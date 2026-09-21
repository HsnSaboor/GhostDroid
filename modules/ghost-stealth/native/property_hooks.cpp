#include "gs_state.h"

#include <dlfcn.h>
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

int (*orig_sp_foreach)(void (*)(const prop_info*, void*), void*) = nullptr;

const prop_info* (*orig_sp_find_nth)(unsigned) = nullptr;

// Container tell deny-list: key NAMES alone betray Waydroid even when
// values are spoofed (probe log: waydroid.host.uid queried hundreds of
// times). Local IsDeniedProp forwards to the shared gs-scope
// IsDeniedProperty (declared in gs_state.h so the exec/popen bypass
// reuses the same list).
bool IsDeniedProp(const char* name);

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

// Spoofed value for a synthetic prop_info*. The spoof table is read-only
// post-install, so a miss is impossible; serve fail-closed empty instead
// of handing our opaque pointer to the real libc read path (crash/garbage).
std::string SyntheticValue(const std::string& synthName) {
    std::string spoofed;
    LookupProperty(synthName.c_str(), spoofed);
    return spoofed;
}

// Truncated copy with NUL termination. cap includes NUL (PROP_*_MAX).
// dst may be null (length query): no write, still returns truncated len.
int CopyPropString(const std::string& s, char* dst, size_t cap) {
    size_t n = s.size();
    if (cap == 0) return 0;
    if (n > cap - 1) n = cap - 1;
    if (dst != nullptr) {
        memcpy(dst, s.data(), n);
        dst[n] = '\0';
    }
    return (int)n;
}

// Deny wins, then spoof, else the original rc. value==nullptr is a
// length query: denied/spoofed keys report 0 without touching libc,
// so existence never leaks through a length oracle.
int ServeValue(const char* name, char* value, int origRc) {
    if (value == nullptr) {
        // Length query: denied reads as absent (0); spoofed reports its
        // truncated length without writing; else the original rc.
        if (IsDeniedProperty(name)) return 0;
        std::string probe;
        if (LookupProperty(name, probe)) {
            return CopyPropString(probe, nullptr, PROP_VALUE_MAX);
        }
        return origRc;
    }
    if (IsDeniedProperty(name)) {
        value[0] = '\0';
        return 0;
    }
    std::string spoofed;
    if (LookupProperty(name, spoofed)) {
        return CopyPropString(spoofed, value, PROP_VALUE_MAX);
    }
    return origRc;
}

int my_sp_get(const char* name, char* value) {
    TraceProbeProp(name);
    if (name == nullptr) return 0;
    // Lazy GLES + sensor hook: libGLESv2.so / libandroid.so map after
    // preAppSpecialize, so retry the resolver on every prop probe (single
    // pointer check once hooked). Lands glGetString + NDK sensor list the
    // moment the game loads the drivers.
    if (!TryHookGraphicsResolved()) { /* not loaded yet; keep probing */ }
    TryHookSensorResolved();
    // ro.hardware.gralloc DISABLED 2026-09-21: masking it to empty inside
    // the game process SEGVs GraphicBufferAllocator (verified tombstone:
    // gbm_mesa_bo_import via CrosGralloc4Mapper::importBuffer). The
    // allocator queries the prop mid-allocate and mis-selects the mapper
    // backend when the value is absent. Falls through to real
    // minigbm_gbm_mesa until a backend-safe spoof (kaanapali-with-mapper?)
    // is proven. DO NOT deny without a mapper shim.
    if (IsDeniedProp(name)) {
        if (value != nullptr) value[0] = '\0';
        return 0;
    }
    std::string spoofed;
    if (LookupProperty(name, spoofed)) {
        return CopyPropString(spoofed, value, PROP_VALUE_MAX);
    }
    if (orig_sp_get) return orig_sp_get(name, value);
    return 0;
}

const prop_info* my_sp_find(const char* name) {
    TraceProbeProp(name);
    if (name == nullptr) return nullptr;
    if (IsDeniedProp(name)) return nullptr;
    std::string spoofed;
    if (LookupProperty(name, spoofed)) {
        // Empty spoof = absent on retail: null, not a handle.
        if (spoofed.empty()) return nullptr;
        return MakeSynthetic(name);
    }
    if (orig_sp_find) return orig_sp_find(name);
    return nullptr;
}

int my_sp_read(const prop_info* pi, char* name, char* value) {
    std::string synthName;
    if (IsSynthetic(pi, synthName)) {
        if (IsDeniedProp(synthName.c_str())) {
            CopyPropString(synthName, name, PROP_NAME_MAX);
            if (value != nullptr) value[0] = '\0';
            return 0;
        }
        const std::string spoofed = SyntheticValue(synthName);
        CopyPropString(synthName, name, PROP_NAME_MAX);
        return CopyPropString(spoofed, value, PROP_VALUE_MAX);
    }

    int rc = 0;
    if (orig_sp_read != nullptr) {
        rc = orig_sp_read(pi, name, value);
    }
    // name==nullptr gives no key to evaluate: fall through to orig rc.
    if (name == nullptr) return rc;
    if (name[0] == '\0') {
        // Real read hands back handles without names for some indices;
        // resolve against the area instead of leaking rc/value as-is.
        if (orig_sp_find_nth == nullptr) return rc;
        for (unsigned i = 0; i < 8192; i++) {
            const prop_info* cand = orig_sp_find_nth(i);
            if (cand == nullptr) break;
            if (cand != pi) continue;
            char rname[PROP_NAME_MAX] = {};
            char rval[PROP_VALUE_MAX] = {};
            if (orig_sp_read(cand, rname, rval) < 0) return rc;
            return ServeValue(rname, value, rc);
        }
        return rc;
    }
    return ServeValue(name, value, rc);
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
    if (w == nullptr || w->orig == nullptr) return;
    const char* effective_name = (w->synth_name != nullptr) ? w->synth_name : name;

    if (IsDeniedProperty(effective_name)) {
        w->orig(w->orig_cookie, effective_name, "", serial);
        return;
    }
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
        if (IsDeniedProp(synthName.c_str())) {
            callback(cookie, synthName.c_str(), "", /*serial=*/0);
            return;
        }
        const std::string spoofed = SyntheticValue(synthName);
        callback(cookie, synthName.c_str(), spoofed.c_str(), /*serial=*/0);
        return;
    }

    CbWrapper w{};
    w.orig = callback;
    w.orig_cookie = cookie;
    w.synth_name = nullptr;

    if (orig_sp_read_callback) {
        orig_sp_read_callback(pi, &CbTrampoline, &w);
    }
}

// Enumeration filter: __system_property_foreach hands out raw prop_info
// pointers, so denied key NAMES would leak even though get/find spoof
// values. Re-read each name via the ORIGINAL read and skip denied keys;
// spoofed values still flow through the user's callback via our read hook.
struct ForeachCtx {
    void (*user)(const prop_info*, void*);
    void* cookie;
};

void ForeachTrampoline(const prop_info* pi, void* vctx) {
    auto* c = reinterpret_cast<ForeachCtx*>(vctx);
    if (c == nullptr || c->user == nullptr) return;
    if (orig_sp_read == nullptr) {
        c->user(pi, c->cookie);
        return;
    }
    char name[PROP_NAME_MAX] = {};
    char value[PROP_VALUE_MAX] = {};
    // orig read with a value buffer: libc only writes name when it can
    // also deliver a value; guarantees name is NUL-set on success.
    if (orig_sp_read(pi, name, value) < 0) return;
    if (IsDeniedProp(name)) return;
    c->user(pi, c->cookie);
}

int my_sp_foreach(void (*propfn)(const prop_info*, void*), void* cookie) {
    if (propfn == nullptr || orig_sp_foreach == nullptr) {
        if (orig_sp_foreach != nullptr) return orig_sp_foreach(propfn, cookie);
        return -1;
    }
    ForeachCtx ctx{propfn, cookie};
    return orig_sp_foreach(ForeachTrampoline, &ctx);
}

// Indexed enumeration (seen in ACE imports as __system_property_find_nth):
// remap logical indices past denied keys so numbering stays dense.
const prop_info* my_sp_find_nth(unsigned n) {
    if (orig_sp_find_nth == nullptr) return nullptr;
    unsigned seen = 0;
    for (unsigned i = 0; i < 8192; i++) {
        const prop_info* pi = orig_sp_find_nth(i);
        if (pi == nullptr) return nullptr;
        if (orig_sp_read != nullptr) {
            char name[PROP_NAME_MAX] = {};
            char value[PROP_VALUE_MAX] = {};
            if (orig_sp_read(pi, name, value) < 0) continue;
            if (IsDeniedProp(name)) continue;
        }
        if (seen == n) return pi;
        seen++;
    }
    return nullptr;
}

}  // namespace

// Shared deny-list predicate (declared in gs_state.h): the exec/popen
// getprop bypass reuses the same deny-aware view as the libc hooks.
// Container tells read as unset: key NAMES alone betray Waydroid even when
// values are spoofed (probe log: 856x waydroid.host.uid).
// persist.waydroid.* is fully denied (incl. fake_wifi/fake_touch — the
// platform FakeWifi path reads them outside target processes; in-target
// readers get empty, same as a phone without those keys).
// Probe-grounded (PUBG pid 3164, 26k-line take): every prefix/exact below
// was QUERIED by the game and is absent on a real S26, so deny (empty)
// rather than invent a value.
bool IsDeniedProperty(const char* name) {
    if (name == nullptr || name[0] == '\0') return false;
    // Vendor/container namespace prefixes: any key under these betrays
    // the runtime even with a spoofed value.
    static const char* const kDeniedPrefixes[] = {
        "waydroid.",                 // Waydroid container (600x+ queried)
        "qemu.",                     // goldfish/ranchu kernel props
        "ro.kernel.qemu",            // ro.kernel.qemu.* paravirt tells
        "ro.boot.qemu",              // ro.boot.qemu.* (ACE sweep, probe 2594)
        "ro.qemu.",                  // ro.qemu.initrc
        "vendor.qemu.",              // vendor.qemu.vport.*, sf.fake_camera
        "init.svc.qemu",             // init.svc.qemu-*, qemud*
        "init.svc.goldfish-",        // init.svc.goldfish-setup/logcat
        "init.svc.ranchu-",          // init.svc.ranchu-setup/net
        "init.svc.vbox86-",          // init.svc.vbox86-setup
        "init.svc.ld",               // init.svc.ldinit (LDPlayer)
        "init.svc.cloud",            // cloudAppEngine/cloudcheck
        "init.svc.cph",              // init.svc.cph_logger (Redfinger cph)
        "init.svc.ecalc",            // init.svc.ecalcMediaCtl/setup
        "init.svc.lg",               // init.svc.lgserver (LG cloud)
        "init.svc_debug_pid.",       // init.svc_debug_pid.cloudAppEngine
        "com.cph.",                  // Redfinger cloud-phone client keys
        "persist.com.cph.",          // persist.com.cph.adbd/bg_killer
        "ro.com.cph.",               // ro.com.cph.cloud_app_engine/mac
        "docker.",                   // docker.fps.*/storage
        "ro.docker.",                // ro.docker.Gateway/IPAddress
        "microvirt.",                // microvirt.inited/memu_version (MEmu)
        "nemud.",                    // nemud.player_package (MEmu)
        "persist.nox.",              // persist.nox.simulator_version
        "persist.ld.",               // persist.ld.user.identified
        "persist.gamematrix.",       // persist.gamematrix.instance
        "javaprop.gamematrix.",      // javaprop.gamematrix.gameid
        "vendor.gamematrix.",        // vendor.gamematrix.cluster/hw_type
        "debug.gamematrix.",         // debug.gamematrix.deviceid
        "dev.cloudgame.",            // dev.cloudgame.heartbeat
        "ro.cloud.",                 // ro.cloud.gaming/rentable
        "ro.tenc.",                  // ro.tenc.cloudgame.gameid
        "ro.pscloud.",               // ro.pscloud.vm.name
        "ro.boottime.cloud",         // ro.boottime.cloudAppEngine/vm_srv
        "ro.boottime.cph",           // ro.boottime.cph_logger
        "ro.boottime.ecalc",         // ro.boottime.ecalcMediaCtl/setup
        "ro.boottime.lg",            // ro.boottime.lgserver
        "target.cloud.",             // target.cloud.type
        "storage.cloud.",            // storage.cloud.mode
        "vendor.cloudgame.",         // vendor.cloudgame.game
        "hm.",                       // Haimawan hm.custom.*/service.id
        "hm_",                       // hm_rtc_*/hm_wm_size_*
        "persist.hm.",               // persist.hm.device.info
        "ro.hm.",                    // ro.hm.device.type
        "ro.ecalc.",                 // ro.ecalc.otherSystemSize
        "camera.ecalc.",             // camera.ecalc.camera_facing
        "ro.rk.",                    // ro.rk.bt_enable/ethernet/hdmi (RK boxes)
        "ro.rksdk.",                 // ro.rksdk.version
        "ro.lgsys.",                 // ro.lgsys.sid/tid
        "ro.boot.ro.lgsys.",         // ro.boot.ro.lgsys.tid
        "lg.huang.",                 // lg.huang.addrule/init/network
        "lgctrl.",                   // lgctrl.properties.pid
        "vm.lgsys.",                 // vm.lgsys.init
        "ro.build.nubia.",           // ro.build.nubia.rom.name
        "ro.build.tyd.",             // ro.build.tyd.kbstyle_version
        "ro.gn.",                    // ro.gn.gnromvernumber (GIONEE)
        "ro.lenovo.",                // ro.lenovo.series
        "ro.lewa.",                  // ro.lewa.version
        "ro.meizu.",                 // ro.meizu.product.model
        "ro.miui.",                  // ro.miui.ui.version.name (no MIUI on S26)
        "ro.vivo.",                  // ro.vivo.os.*
        "ro.gfx.driver",             // ro.gfx.driver.0/1/_build_time
        "ro.kernel.mac80211_hwsim.",  // Linux wifi-simulator channels
        "persist.sys.bd.",           // persist.sys.bd.keep_alive/su_white
        "persist.sys.byte_cloud_env",  // byte-dance cloud env flag
        "phone.",                    // phone.area/aws/productname
        "bst.",                      // bst.bluestacks_eb_url
        "com.taoxinyun.",            // com.taoxinyun.android.channel
        "omx_vp8_",                  // omx_vp8_max/min_qp (soft-codec tell)
        "ro.bootmode",               // bootmode (absent on retail user builds)
        "ro.hardware.power",         // power HAL key (absent on retail dumps)
        "ro.surface_flinger.",       // has_HDR/wide_color (absent on retail S26 dumps)
        "adjust.",                   // adjust.preinstall.*
        "vendor.cf.",                // vendor.cf.address (Cuttlefish)
        "vclusters.",                // vclusters.virtual.camera
        "gralloc.gbm.device",        // mesa/gbm software-render tell
        "mesa.",                     // mesa.* debug knobs (absent on retail)
        "debug.mesa.",               // debug.mesa.* (absent on retail)
        "vendor.mesa.",              // vendor.mesa.* (absent on retail)
        "dalvik.vm.dexopt.",         // dexopt tuning knobs (absent on retail user builds)
        "dalvik.vm.my-feature-test.",  // feature-test keys (absent on retail)
        "debug.allocTracker.",       // alloc tracker knobs (absent on retail)
        "debug.am.",                   // am trim knobs (absent on retail, probe 8955/10750)
        "debug.sqlite.",               // sqlite debug knobs (absent on retail, probe 8955/10750)
        "debug.hwui.",                 // hwui debug knobs (absent on retail, probe 8955/10750)
        "dalvik.vm.metrics.",          // metrics knobs (absent on retail, probe 8955/10750)
        "media.metrics.",              // metrics flag (absent on retail, probe 8955/10750)
        "debug.atrace.",             // atrace app flags (absent on retail)
        "debug.egl.",                // egl debug knobs (absent on retail)
        "debug.firebase.",           // firebase debug keys (absent on retail)
        "debug.gles.",               // gles layer knobs (absent on retail)
        "debug.layout",              // layout debug flag (absent on retail)
        "debug.second-display.",     // second-display pkg (absent on retail)
        "framework.pause_bg_animations.",  // bg-animation flag (absent on retail)
        "heapprofd.",                // heapprofd profiler knobs (absent on retail)
        "hw_sc.",                    // hw_sc platform keys (absent on retail S26)
        "libc.debug.gwp_asan.",      // gwp_asan debug knobs (absent on retail)
        "persist.device_config.runtime_native.",  // runtime device_config (absent on retail)
        "persist.libc.debug.gwp_asan.",  // persisted gwp_asan knobs (absent on retail)
        "persist.media.metrics.",    // media metrics flag (absent on retail)
        "renderthread.skia.",        // skia renderthread knobs (absent on retail)
        "ro.hwui.",                  // hwui renderer keys (absent on retail S26)
        "viewroot.",                 // viewroot profiling flags (absent on retail)
    };
    for (const char* prefix : kDeniedPrefixes) {
        size_t len = strlen(prefix);
        if (strncmp(name, prefix, len) == 0) return true;
    }
    if (strncmp(name, "persist.waydroid.", 17) == 0)
        return true;
    // One-off emulator/cloud tells: real S26 leaves these absent.
    static const char* const kDeniedExact[] = {
        "ro.hardware.camera",        // v4l2 camera HAL key, VMM-only
        "ro.hardware.alter",         // libcubehawk probe; absent on retail S26
        "ro.hardware.wifi",          // unsourceable wifi HAL key (absent on retail)
        "ro.input.resampling",       // input-resampling knob (absent on retail)
        "ro.hardware.fps.cph",       // Redfinger fps HAL key
        "ro.adb.qemud",              // adb-over-qemud pipe flag
        "ro.kernel.android.qemud",   // android.qemud kernel driver
        "ro.dalvik.vm.isa.x86_64",   // x86_64 ABI tell on arm64 profile
        "ro.genymotion.version",     // Genymotion build stamp
        "ro.ddy.webrtc",             // dayu-cloud webrtc flag
        "ro.aa.romver",              // third-party ROM version stamp
        "ro.bd.product.model",       // box-stack product model
        "ro.build.rom.id",           // third-party ROM id
        "ro.build.version.emui",     // EMUI version (no EMUI on S26)
        "ro.build.version.opporom",  // Oppo ROM version
        "ro.global.scene",           // scene-sdk leftover key
        "ro.vendor.rk_sdk",          // Rockchip vendor SDK flag
        "ro.vendor.redirect_socket_calls",  // vsock-redirect tell
        "ro.vendor.product.brand",   // vendor product overlay (Waydroid)
        "ro.vendor.product.name",    // vendor product overlay (Waydroid)
        "ro.product.base_version",   // base-version overlay (Waydroid)
        "vendor.rild.libpath",       // emulator rild path (unsourceable)
        "gsm.version.baseband",      // emulator baseband (unsourceable)
        "drm.gpu.vendor_name",       // drm/gbm software-render tell
        "service.adb.tcp.port",      // tcp-adb tell (unset on retail S26)
        "persist.adb.tls_server.enable",  // adb tls-server flag
        "persist.sys.byte_cloud_env",  // byte-dance cloud env flag (queried, probe 2594)
        "persist.sys.strictmode.disable",  // probe 2594: raw true on container, absent on retail user builds
        "debug.force_rtl",  // probe 2594: raw false on container, absent on retail user builds
        "ro.boot.vbmeta.digest",     // test-keys vbmeta digest (unsourceable)
        "ro.boot.enable_dm_verity",  // verity-mode tell (unset on retail)
        "sys.tencent.model",         // tencent channel leftover
        "sys.prop.writewifissid",    // wifi-ssid writer flag
        "sys.wmwidth",               // haimawan cloud-phone window size
        "sys.wmheight",              // (same sweep, absent on retail)
        "wifi_name",                 // cooler-style wifi key
        "phone_type",                // phone-type leftover key
        "su_white_list.1",           // bd su-whitelist tell
        "keep_alive_whitelist",      // bd keep-alive tell
        "init.svc.android-hardware-media-c2-goldfish-hal-1-0",
    };
    for (const char* key : kDeniedExact) {
        if (strcmp(name, key) == 0) return true;
    }
    return false;
}

namespace {
bool IsDeniedProp(const char* name) { return IsDeniedProperty(name); }

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
    bool ok_foreach = HookExport("__system_property_foreach",
            reinterpret_cast<void*>(&my_sp_foreach),
            reinterpret_cast<void**>(&orig_sp_foreach));
    bool ok_nth = HookExport("__system_property_find_nth",
            reinterpret_cast<void*>(&my_sp_find_nth),
            reinterpret_cast<void**>(&orig_sp_find_nth));

    DS_LOGI("Hook(prop): get=%d find=%d read=%d read_callback=%d foreach=%d find_nth=%d",
            ok_get, ok_find, ok_read, ok_cb, ok_foreach, ok_nth);

    InstallSystemHooks();

    InstallFileHooks();

    // Resolver-only: DobbySymbolResolver over already-loaded exports, zero
    // linker work, no load trap. Safe from the preAppSpecialize path; a miss
    // when the GL driver is not loaded yet just leaves real strings.
    InstallGraphicsHooks();

    // NOTE: InstallSensorHooks deliberately NOT called here.
    // DobbySymbolResolver -> GetProcessModuleMap crashes inside
    // preAppSpecialize/zygote early init (tombstone 10057: single-frame
    // crash in the maps parser, not recursion). Same reason the graphics
    // hook tolerates install-time miss: lazy retry in my_sp_get/my_open/
    // my_fopen lands it once the app process is fully up.

    DS_LOGI("installed  spoofed_keys=%zu  orig_get=%p orig_find=%p "
            "orig_read=%p orig_cb=%p",
            (size_t)g_props.size(), (void*)orig_sp_get,
            (void*)orig_sp_find, (void*)orig_sp_read,
            (void*)orig_sp_read_callback);
    if (TraceProbes()) {
        DS_LOGW("probe tracing ON: full take -> /data/local/tmp/gs_probe_<pid>.log");
    }
}

}  // namespace gs
