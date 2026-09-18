#include "gs_state.h"

#include <cstdarg>
#include <cstdio>
#include <cstring>
#include <fcntl.h>
#include <string>

#include <dobby.h>

namespace gs {

namespace {

// FORCE_DENYLIST_UNMOUNT hides /data/adb in targets, so the fake file
// lives at a world-readable path service.sh maintains.
#define FAKE_MOUNTS "/data/local/tmp/gs_fake_mounts"

bool (*orig_open)(const char*, int, ...) = nullptr;
int (*orig_openat)(int, const char*, int, ...) = nullptr;
FILE* (*orig_fopen)(const char*, const char*) = nullptr;

bool IsMountsPath(const char* p) {
    if (p == nullptr) return false;
    return strcmp(p, "/proc/mounts") == 0 ||
           strcmp(p, "/proc/self/mounts") == 0 ||
           strcmp(p, "/proc/self/mountinfo") == 0 ||
           strcmp(p, "/proc/mountinfo") == 0;
}

const char* Redirect(const char* path) {
    return IsMountsPath(path) ? FAKE_MOUNTS : path;
}

int my_open(const char* path, int flags, ...) {
    mode_t mode = 0;
    if ((flags & O_CREAT) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
    }
    const char* eff = Redirect(path);
    if (eff != path) {
        // Read-only view: strip write/creat so apps can't corrupt the fake.
        flags &= ~(O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND);
        return orig_open(eff, flags | O_RDONLY, mode);
    }
    return orig_open(path, flags, mode);
}

int my_openat(int dirfd, const char* path, int flags, ...) {
    mode_t mode = 0;
    if ((flags & O_CREAT) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode = va_arg(ap, mode_t);
        va_end(ap);
    }
    if (dirfd == AT_FDCWD && IsMountsPath(path)) {
        flags &= ~(O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND);
        return orig_openat(dirfd, FAKE_MOUNTS, flags | O_RDONLY, mode);
    }
    return orig_openat(dirfd, path, flags, mode);
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
    // open64/openat64/fopen64 alias to the 64-bit variants on LP64; hook
    // best-effort (missing symbols skipped, no crash).
    bool ok_open64 = HookExport("open64", reinterpret_cast<void*>(&my_open),
                                reinterpret_cast<void**>(&orig_open));
    DS_LOGI("file hooks: open=%d openat=%d fopen=%d open64=%d",
            ok_open, ok_openat, ok_fopen, ok_open64);
}

}  // namespace gs
