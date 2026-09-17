#include "gs_state.h"

#include <atomic>
#include <dlfcn.h>
#include <mutex>

#include <lsplt.hpp>

namespace gs {

namespace {

void* (*orig_dlopen)(const char*, int) = nullptr;
void* (*orig_android_dlopen_ext)(const char*, int, const void*) = nullptr;

std::mutex g_recommit_mutex;
std::atomic<bool> g_in_recommit{false};

void Recommit() {
    if (g_in_recommit.exchange(true)) return;
    {
        std::lock_guard<std::mutex> lk(g_recommit_mutex);
        bool ok = lsplt::CommitHook();
        DS_LOGI("dlopen re-commit: ok=%d", ok);
    }
    g_in_recommit = false;
}

void* my_dlopen(const char* name, int flags) {
    void* h = orig_dlopen ? orig_dlopen(name, flags) : nullptr;
    if (h != nullptr) Recommit();
    return h;
}

void* my_android_dlopen_ext(const char* name, int flags, const void* extinfo) {
    void* h = orig_android_dlopen_ext
            ? orig_android_dlopen_ext(name, flags, extinfo)
            : nullptr;
    if (h != nullptr) Recommit();
    return h;
}

}  // namespace

void InstallDlopenHooks(dev_t dev, ino_t inode) {
    bool ok_d  = lsplt::RegisterHook(dev, inode, "dlopen",
            reinterpret_cast<void*>(&my_dlopen),
            reinterpret_cast<void**>(&orig_dlopen));
    bool ok_ax = lsplt::RegisterHook(dev, inode, "android_dlopen_ext",
            reinterpret_cast<void*>(&my_android_dlopen_ext),
            reinterpret_cast<void**>(&orig_android_dlopen_ext));
    DS_LOGI("dlopen hooks: dlopen=%d android_dlopen_ext=%d", ok_d, ok_ax);
}

}  // namespace gs
