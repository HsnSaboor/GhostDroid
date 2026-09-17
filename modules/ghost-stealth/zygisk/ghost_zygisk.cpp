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
#include <dlfcn.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>

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

// Installs gs_native prop map for THIS process only. The .so is loaded
// from the module lib dir (zygisk/<abi>/libgs_native.so, same dir as the
// Zygisk entry zygisk/<abi>.so); lsplt hooks apply to this process's libc.
void InstallForProcess(const std::vector<char> &conf) {
    // Parse key=value into g_props via native entry point.
    // Loaded dynamically to keep this TU dependency-free.
    const char *paths[] = {
        "/data/adb/modules/ghost-stealth/zygisk/arm64-v8a/libgs_native.so",
        "/data/adb/modules/ghost-stealth/zygisk/x86_64/libgs_native.so",
        "/data/adb/modules/ghost-stealth/zygisk/x86/libgs_native.so",
        "/data/adb/modules/ghost-stealth/zygisk/armeabi-v7a/libgs_native.so",
        nullptr,
    };
    typedef int (*InstallFn)(const char *, size_t);
    for (const char **p = paths; *p != nullptr; p++) {
        void *h = dlopen(*p, RTLD_NOW | RTLD_LOCAL);
        if (h == nullptr) continue;
        auto fn = (InstallFn)dlsym(h, "gs_install_buf");
        if (fn != nullptr && !conf.empty()) {
            fn(conf.data(), conf.size());
        }
        return;
    }
}

}  // namespace

class GhostStealth : public zygisk::ModuleBase {
public:
    void onLoad(zygisk::Api *api, JNIEnv *env) override {
        this->api = api;
        this->env = env;
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
        api->setOption(zygisk::FORCE_DENYLIST_UNMOUNT);
        conf_ = ReadFile(SPOOF_CONF);
    }

    void postAppSpecialize(const zygisk::AppSpecializeArgs * /*args*/) override {
        if (conf_.empty()) return;
        InstallForProcess(conf_);
        conf_.clear();
        conf_.shrink_to_fit();
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
