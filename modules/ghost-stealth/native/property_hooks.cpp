#include "gs_state.h"

#include <sys/system_properties.h>
#include <cstring>
#include <memory>
#include <mutex>
#include <unordered_map>
#include <vector>

#include <dobby.h>

namespace gs {

namespace {

int  (*orig_sp_get)(const char*, char*) = nullptr;

const prop_info* (*orig_sp_find)(const char*) = nullptr;

int  (*orig_sp_read)(const prop_info*, char*, char*) = nullptr;

void (*orig_sp_read_callback)(
        const prop_info*,
        void (*)(void*, const char*, const char*, uint32_t),
        void*) = nullptr;

// Opaque prop_info* we return from __system_property_find for spoofed keys.
// The real layout is libc-internal; our read hook recognises these pointers
// and short-circuits before they reach the real read code.
struct SyntheticPi {
    std::string name;
};

std::mutex g_synth_mutex;
std::vector<std::unique_ptr<SyntheticPi>> g_synth_storage;
std::unordered_map<const prop_info*, SyntheticPi*> g_synth_index;
std::unordered_map<std::string, const prop_info*> g_synth_by_name;

bool IsSynthetic(const prop_info* pi, std::string& nameOut) {
    if (pi == nullptr) return false;
    std::lock_guard<std::mutex> lk(g_synth_mutex);
    auto it = g_synth_index.find(pi);
    if (it == g_synth_index.end()) return false;
    nameOut = it->second->name;
    return true;
}

const prop_info* MakeSynthetic(const char* name) {
    if (name == nullptr) return nullptr;
    std::lock_guard<std::mutex> lk(g_synth_mutex);
    auto existing = g_synth_by_name.find(name);
    if (existing != g_synth_by_name.end()) {
        return existing->second;
    }

    auto p = std::make_unique<SyntheticPi>();
    p->name = name;
    const prop_info* opaque = reinterpret_cast<const prop_info*>(p.get());

    g_synth_index.emplace(opaque, p.get());
    g_synth_by_name.emplace(p->name, opaque);
    g_synth_storage.push_back(std::move(p));
    return opaque;
}

int my_sp_get(const char* name, char* value) {
    std::string spoofed;
    if (LookupProperty(name, spoofed)) {
        size_t n = spoofed.size();
        if (n > PROP_VALUE_MAX - 1) n = PROP_VALUE_MAX - 1;
        if (value != nullptr) {
            memcpy(value, spoofed.data(), n);
            value[n] = '\0';
        }
        return (int)n;
    }
    if (orig_sp_get) return orig_sp_get(name, value);
    return 0;
}

const prop_info* my_sp_find(const char* name) {
    if (name != nullptr) {
        std::string spoofed;
        if (LookupProperty(name, spoofed)) {
            return MakeSynthetic(name);
        }
    }
    if (orig_sp_find) return orig_sp_find(name);
    return nullptr;
}

int my_sp_read(const prop_info* pi, char* name, char* value) {
    std::string synthName;
    if (IsSynthetic(pi, synthName)) {
        std::string spoofed;
        if (LookupProperty(synthName.c_str(), spoofed)) {
            if (name != nullptr) {
                size_t nlen = synthName.size();
                if (nlen > PROP_NAME_MAX - 1) nlen = PROP_NAME_MAX - 1;
                memcpy(name, synthName.data(), nlen);
                name[nlen] = '\0';
            }
            size_t vlen = spoofed.size();
            if (vlen > PROP_VALUE_MAX - 1) vlen = PROP_VALUE_MAX - 1;
            if (value != nullptr) {
                memcpy(value, spoofed.data(), vlen);
                value[vlen] = '\0';
            }
            return (int)vlen;
        }
    }

    int rc = 0;
    if (orig_sp_read != nullptr) {
        rc = orig_sp_read(pi, name, value);
    }
    if (name != nullptr && value != nullptr) {
        std::string spoofed;
        if (LookupProperty(name, spoofed)) {
            size_t vlen = spoofed.size();
            if (vlen > PROP_VALUE_MAX - 1) vlen = PROP_VALUE_MAX - 1;
            memcpy(value, spoofed.data(), vlen);
            value[vlen] = '\0';
            return (int)vlen;
        }
    }
    return rc;
}

// read_callback invokes the user callback synchronously, so this can live on
// the caller's stack.
struct CbWrapper {
    void (*orig)(void*, const char*, const char*, uint32_t);
    void* orig_cookie;
    const char* synth_name;
};

void CbTrampoline(void* cookie, const char* name, const char* value,
                  uint32_t serial) {
    auto* w = reinterpret_cast<CbWrapper*>(cookie);
    const char* effective_name = (w->synth_name != nullptr) ? w->synth_name : name;

    std::string spoofed;
    if (LookupProperty(effective_name, spoofed)) {
        w->orig(w->orig_cookie, effective_name, spoofed.c_str(), serial);
        return;
    }
    w->orig(w->orig_cookie, name, value, serial);
}

void my_sp_read_callback(const prop_info* pi,
                         void (*callback)(void*, const char*, const char*,
                                          uint32_t),
                         void* cookie) {
    if (callback == nullptr) {
        if (orig_sp_read_callback) orig_sp_read_callback(pi, callback, cookie);
        return;
    }

    std::string synthName;
    if (IsSynthetic(pi, synthName)) {
        std::string spoofed;
        if (LookupProperty(synthName.c_str(), spoofed)) {
            callback(cookie, synthName.c_str(), spoofed.c_str(), /*serial=*/0);
            return;
        }
    }

    CbWrapper w{};
    w.orig = callback;
    w.orig_cookie = cookie;
    w.synth_name = nullptr;

    if (orig_sp_read_callback) {
        orig_sp_read_callback(pi, &CbTrampoline, &w);
    }
}

// Dobby export-side hook: patches the symbol in libc itself, so libraries
// loaded later (e.g. libemulatordetector.so via System.loadLibrary) call
// the replacement too. LSPlt PLT-patching only covered already-loaded
// callers, which is why reveny still saw real values.
bool HookExport(const char *sym, void *replace, void **orig) {
    void *addr = DobbySymbolResolver(nullptr, sym);
    if (addr == nullptr) {
        DS_LOGW("DobbySymbolResolver(%s) -> null, skipping", sym);
        return false;
    }
    int rc = DobbyHook(addr, (dobby_dummy_func_t)replace,
                       (dobby_dummy_func_t *)orig);
    DS_LOGI("DobbyHook %s @ %p rc=%d", sym, addr, rc);
    return rc == 0;
}

}  // namespace

void InstallPropertyHooks() {
    bool ok_get  = HookExport("__system_property_get",
            reinterpret_cast<void*>(&my_sp_get),
            reinterpret_cast<void**>(&orig_sp_get));
    bool ok_find = HookExport("__system_property_find",
            reinterpret_cast<void*>(&my_sp_find),
            reinterpret_cast<void**>(&orig_sp_find));
    bool ok_read = HookExport("__system_property_read",
            reinterpret_cast<void*>(&my_sp_read),
            reinterpret_cast<void**>(&orig_sp_read));
    bool ok_cb   = HookExport("__system_property_read_callback",
            reinterpret_cast<void*>(&my_sp_read_callback),
            reinterpret_cast<void**>(&orig_sp_read_callback));

    DS_LOGI("Hook(prop): get=%d find=%d read=%d read_callback=%d",
            ok_get, ok_find, ok_read, ok_cb);

    InstallSystemHooks();

    InstallFileHooks();

    DS_LOGI("installed  spoofed_keys=%zu  orig_get=%p orig_find=%p "
            "orig_read=%p orig_cb=%p",
            (size_t)g_props.size(), (void*)orig_sp_get,
            (void*)orig_sp_find, (void*)orig_sp_read,
            (void*)orig_sp_read_callback);
}

}  // namespace gs
