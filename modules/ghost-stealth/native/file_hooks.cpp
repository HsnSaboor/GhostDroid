#include "gs_state.h"

#include <cstdarg>
#include <cstdio>
#include <cstring>
#include <fcntl.h>
#include <string>

#include <dobby.h>

namespace gs {

namespace {

// FORCE_DENYLIST_UNMOUNT hides /data/adb in targets, so the fake files
// live at world-readable paths service.sh maintains.
#define FAKE_MOUNTS "/data/local/tmp/gs_fake_mounts"
#define FAKE_MODULES "/data/local/tmp/gs_fake_modules"
#define FAKE_INPUT "/data/local/tmp/gs_fake_input_devices"
#define FAKE_ASOUND "/data/local/tmp/gs_fake_asound_cards"
#define FAKE_USB "/data/local/tmp/gs_fake_usb_devices"
#define FAKE_PCI_DEVICES "/data/local/tmp/gs_fake_pci_devices"
#define FAKE_PCI_UEVENT "/data/local/tmp/gs_fake_pci_uevent"

int (*orig_open)(const char*, int, ...) = nullptr;
int (*orig_openat)(int, const char*, int, ...) = nullptr;
FILE* (*orig_fopen)(const char*, const char*) = nullptr;

// Covers both exact paths and per-device node paths (DeviceInfoHW reads
// nodes like /sys/bus/usb/devices/1-0:1.0/descriptors and ALSA card dirs).
bool PathStartsWith(const char* p, const char* prefix) {
    if (p == nullptr || prefix == nullptr) return false;
    size_t n = strlen(prefix);
    return strncmp(p, prefix, n) == 0;
}

bool IsMountsPath(const char* p) {
    if (p == nullptr) return false;
    return strcmp(p, "/proc/mounts") == 0 ||
           strcmp(p, "/proc/self/mounts") == 0 ||
           strcmp(p, "/proc/self/mountinfo") == 0 ||
           strcmp(p, "/proc/mountinfo") == 0;
}

const char* Redirect(const char* path) {
    if (IsMountsPath(path)) return FAKE_MOUNTS;
    // DRIVERS tab: /proc/modules lists host-only modules (snd_hda_intel,
    // thinkpad_acpi, i2c_*...). Serve a short virtio-only list instead.
    if (path != nullptr && strcmp(path, "/proc/modules") == 0) return FAKE_MODULES;
    // INPUT tab: /proc/bus/input/devices leaks ThinkPad/Elantech/HDA.
    if (path != nullptr && strcmp(path, "/proc/bus/input/devices") == 0) return FAKE_INPUT;
    // AUDIO tab: /proc/asound/cards leaks HDA Intel PCH.
    if (path != nullptr && strcmp(path, "/proc/asound/cards") == 0) return FAKE_ASOUND;
    // USB tab: /sys/bus/usb/devices leaks xhci/cachyos/SunplusIT.
    // Directory listing itself is per-node; mask the uevent/product/vendor
    // strings via the common parent read fallback below.
    if (path != nullptr && strcmp(path, "/sys/bus/usb/devices") == 0) return FAKE_USB;
    // PCI Wi-Fi/iGPU: /proc/bus/pci/devices leaks Intel IDs, pci uevent
    // leaks DRIVER=iwlwifi. Per-process redirect only — NEVER global tmpfs
    // (minigbm/libdrm need real PCI sysfs for Intel GPU BO alloc).
    if (path != nullptr && strcmp(path, "/proc/bus/pci/devices") == 0) return FAKE_PCI_DEVICES;
    if (path != nullptr && PathStartsWith(path, "/sys/bus/pci/devices/") &&
        strlen(path) >= 7 && strcmp(path + strlen(path) - 7, "/uevent") == 0)
        return FAKE_PCI_UEVENT;
    return path;
}

// NOTE: orig_open/openat MUST be int-returning variadic pointers. An early
// revision declared them `bool`, which truncated every fd to 1 and killed
// every target at ART startup (fdsan double-close SIGABRT crash loop).
int my_open(const char* path, int flags, ...) {
    const char* eff = Redirect(path);
    if (eff != path) {
        // Read-only view: strip write/creat so apps can't corrupt the fake.
        flags &= ~(O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND);
        return orig_open(eff, flags | O_RDONLY, (mode_t)0);
    }
    if ((flags & O_CREAT) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode_t mode = va_arg(ap, mode_t);
        va_end(ap);
        return orig_open(path, flags, mode);
    }
    return orig_open(path, flags);
}

int my_openat(int dirfd, const char* path, int flags, ...) {
    if (dirfd == AT_FDCWD && IsMountsPath(path)) {
        flags &= ~(O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND);
        return orig_openat(dirfd, FAKE_MOUNTS, flags | O_RDONLY, (mode_t)0);
    }
    if (dirfd == AT_FDCWD && path != nullptr) {
        // Same fake-file treatment for the extended redirect table.
        const char* eff = Redirect(path);
        if (eff != path) {
            flags &= ~(O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND);
            return orig_openat(dirfd, eff, flags | O_RDONLY, (mode_t)0);
        }
    }
    if ((flags & O_CREAT) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode_t mode = va_arg(ap, mode_t);
        va_end(ap);
        return orig_openat(dirfd, path, flags, mode);
    }
    return orig_openat(dirfd, path, flags);
}

FILE* my_fopen(const char* path, const char* mode) {
    const char* eff = Redirect(path);
    if (eff != path) {
        // Force read mode; fake mounts are a read-only view.
        if (mode != nullptr && (strchr(mode, 'w') != nullptr ||
                                strchr(mode, 'a') != nullptr ||
                                strchr(mode, '+') != nullptr)) {
            mode = "r";
        }
        return orig_fopen(eff, mode);
    }
    return orig_fopen(path, mode);
}

bool HookExport(const char* sym, void* replace, void** orig) {
    void* addr = DobbySymbolResolver(nullptr, sym);
    if (addr == nullptr) {
        DS_LOGW("DobbySymbolResolver(%s) -> null, skipping", sym);
        return false;
    }
    int rc = DobbyHook(addr, (dobby_dummy_func_t)replace,
                       (dobby_dummy_func_t*)orig);
    DS_LOGI("DobbyHook %s @ %p rc=%d", sym, addr, rc);
    return rc == 0;
}

}  // namespace

void InstallFileHooks() {
    bool ok_open = HookExport("open", reinterpret_cast<void*>(&my_open),
                              reinterpret_cast<void**>(&orig_open));
    bool ok_openat = HookExport("openat", reinterpret_cast<void*>(&my_openat),
                                reinterpret_cast<void**>(&orig_openat));
    bool ok_fopen = HookExport("fopen", reinterpret_cast<void*>(&my_fopen),
                               reinterpret_cast<void**>(&orig_fopen));
    // NOTE: no open64/openat64 hook. On LP64 open==open64 (same address);
    // hooking the alias second would chain-replace orig_open with my_open
    // and recurse / corrupt fds.
    DS_LOGI("file hooks: open=%d openat=%d fopen=%d",
            ok_open, ok_openat, ok_fopen);
}

}  // namespace gs
