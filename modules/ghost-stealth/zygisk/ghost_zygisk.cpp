// Zygisk entry: per-process gate + native install.
//
// Pattern copied from TargetedFix main.cpp (companion fd protocol,
// DLCLOSE non-targets, FORCE_DENYLIST_UNMOUNT targets). Props come from
// config/spoof.conf (same map native_hooks.cpp consumes). CPU/file masks
// live in service.sh (bind mounts, global). Per-process prop view only:
// host Houdini + base.prop untouched.
//
// Ref: /tmp/tfsrc/app/src/main/cpp/main.cpp,
// .devdocs/Android-Emulator-Detection/.../EmulatorDetection.cpp.
#include <android/log.h>
#include <cstdio>
#include <dlfcn.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>

#define GS_TAG "GhostStealth"
#define GS_LOGI(...) __android_log_print(ANDROID_LOG_INFO, GS_TAG, __VA_ARGS__)
#define GS_LOGE(...) __android_log_print(ANDROID_LOG_ERROR, GS_TAG, __VA_ARGS__)

#include <string>
#include <string_view>
#include <vector>

#include "zygisk.hpp"

#define SPOOF_CONF "/data/adb/modules/ghost-stealth/config/spoof.conf"
#define TARGET_LIST "/data/adb/modules/ghost-stealth/config/target.txt"

namespace {

bool IsTarget(const std::vector<char> &targets, std::string_view process) {
    size_t start = 0;
    for (size_t i = 0; i <= targets.size(); i++) {
        if (i == targets.size() || targets[i] == '\n') {
            size_t a = start, b = i;
            while (a < b && (targets[a] == ' ' || targets[a] == '\r' || targets[a] == '\t')) a++;
            while (b > a && (targets[b - 1] == ' ' || targets[b - 1] == '\r' || targets[b - 1] == '\t')) b--;
            if (a < b && targets[a] != '#') {
                std::string_view line(targets.data() + a, b - a);
                if (line == process) return true;
            }
            start = i + 1;
        }
    }
    return false;
}

std::vector<char> ReadFile(const char *path) {
    std::vector<char> out;
    int fd = open(path, O_RDONLY);
    if (fd < 0) return out;
    struct stat st {};
    if (fstat(fd, &st) == 0 && st.st_size > 0 && st.st_size < 1024 * 1024) {
        out.resize((size_t)st.st_size);
        ssize_t n = read(fd, out.data(), out.size());
        if (n < 0) out.clear();
        else out.resize((size_t)n);
    }
    close(fd);
    return out;
}

// Installs gs_native prop map for THIS process. MUST run in
// preAppSpecialize: FORCE_DENYLIST_UNMOUNT hides /data/adb during
// specialization, so postAppSpecialize cannot dlopen the module lib
// ("not found" for every ABI). Hooks target this process's libc and
// persist — specialization sandboxes, it does not re-exec.
void InstallForProcess(const std::vector<char> &conf) {
    // Parse key=value into g_props via native entry point.
    // Loaded dynamically to keep this TU dependency-free.
    // libgs_native links libdobby.so (DT_NEEDED, same dir) but the loader
    // does not resolve same-dir deps for absolute-path dlopen, so preload
    // libdobby first from the same directory.
    const char *dirs[] = {
        "/data/adb/modules/ghost-stealth/zygisk/arm64-v8a",
        "/data/adb/modules/ghost-stealth/zygisk/x86_64",
        "/data/adb/modules/ghost-stealth/zygisk/x86",
        "/data/adb/modules/ghost-stealth/zygisk/armeabi-v7a",
        nullptr,
    };
    typedef int (*InstallFn)(const char *, size_t);
    char dep[256], lib[256];
    for (const char **d = dirs; *d != nullptr; d++) {
        snprintf(dep, sizeof(dep), "%s/libdobby.so", *d);
        snprintf(lib, sizeof(lib), "%s/libgs_native.so", *d);
        void *dep_h = dlopen(dep, RTLD_NOW | RTLD_GLOBAL);
        if (dep_h == nullptr) {
            GS_LOGE("dlopen %s failed: %s", dep, dlerror());
            continue;
        }
        void *h = dlopen(lib, RTLD_NOW | RTLD_LOCAL);
        if (h == nullptr) {
            GS_LOGE("dlopen %s failed: %s", lib, dlerror());
            continue;
        }
        auto fn = (InstallFn)dlsym(h, "gs_install_buf");
        if (fn == nullptr) {
            GS_LOGE("dlsym gs_install_buf in %s failed: %s", lib, dlerror());
            return;
        }
        if (!conf.empty()) {
            int n = fn(conf.data(), conf.size());
            GS_LOGI("installed %d spoof keys from %s", n, lib);
        }
        return;
    }
    GS_LOGE("no libgs_native.so found in module zygisk dirs");
}

}  // namespace

class GhostStealth : public zygisk::ModuleBase {
public:
    void onLoad(zygisk::Api *api, JNIEnv *env) override {
        this->api = api;
        this->env = env;
        GS_LOGI("ghost-stealth loaded");
    }

    void preAppSpecialize(zygisk::AppSpecializeArgs *args) override {
        const char *raw = env->GetStringUTFChars(args->nice_name, nullptr);
        std::string_view process(raw != nullptr ? raw : "");
        auto targets = ReadFile(TARGET_LIST);
        bool hit = IsTarget(targets, process);
        if (raw != nullptr) env->ReleaseStringUTFChars(args->nice_name, raw);
        if (!hit) {
            api->setOption(zygisk::DLCLOSE_MODULE_LIBRARY);
            return;
        }
        GS_LOGI("target hit: %.*s", (int)process.size(), process.data());
        api->setOption(zygisk::FORCE_DENYLIST_UNMOUNT);
        conf_ = ReadFile(SPOOF_CONF);
        InstallForProcess(conf_);
        conf_.clear();
        conf_.shrink_to_fit();
    }

    void postAppSpecialize(const zygisk::AppSpecializeArgs * /*args*/) override {
    }

    void preServerSpecialize(zygisk::ServerSpecializeArgs * /*args*/) override {
        api->setOption(zygisk::DLCLOSE_MODULE_LIBRARY);
    }

private:
    zygisk::Api *api = nullptr;
    JNIEnv *env = nullptr;
    std::vector<char> conf_;
};

REGISTER_ZYGISK_MODULE(GhostStealth)
