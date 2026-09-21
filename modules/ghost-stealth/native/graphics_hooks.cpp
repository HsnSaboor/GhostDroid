// graphics_hooks.cpp: native GLES string spoof (Adreno 840 / Qualcomm).
//
// Java GpuHooks cannot intercept C-to-C glGetString calls from libUE4.so /
// libanogs.so into libGLESv2.so, so ACE saw "Mesa Intel(R) UHD Graphics 620"
// on a device claiming Galaxy S26 Ultra. This module hooks the exported
// glGetString and returns S26-consistent strings for RENDERER/VENDOR/
// VERSION/SHADING. Every other enum and every failure path falls through
// to the original: fail-open rendering, fail-closed identity strings.
//
// DEADLOCK HISTORY (do NOT regress): calling dlopen("libGLESv2.so",
// RTLD_NOW) inside preAppSpecialize forced Bionic's linker to load the
// graphics HAL while holding Zygote locks; the app RenderThread then wedged
// in futex_wait at EGL init (ANR, 2026-09-21). This file therefore NEVER
// dlopens with RTLD_NOW. Resolution order:
//   1. DobbySymbolResolver (already-loaded export, zero linker work).
//   2. dlopen hook trap: when the game engine loads libGLESv2/libEGL for
//      the first time post-specialize, hook glGetString immediately.

#include "gs_state.h"

#include <dlfcn.h>

#include <cstring>

#include <dobby.h>

namespace gs {

namespace {

using GlStrFn = const unsigned char* (*)(unsigned int);
using DlopenFn = void* (*)(const char*, int);

GlStrFn g_orig_gl_get_string = nullptr;
DlopenFn g_orig_dlopen = nullptr;

constexpr unsigned int kGlVendor = 0x1F00;
constexpr unsigned int kGlRenderer = 0x1F01;
constexpr unsigned int kGlVersion = 0x1F02;
constexpr unsigned int kGlExtensions = 0x1F03;
constexpr unsigned int kGlShadingVersion = 0x8B8C;

// S26 story: Snapdragon 8 Elite / Adreno 840, OpenGL ES 3.2. Must mirror
// GpuHooks.java + spoof.conf gles.* entries.
constexpr const char kSpoofVendor[] = "Qualcomm";
constexpr const char kSpoofRenderer[] = "Adreno (TM) 840";
constexpr const char kSpoofVersion[] =
    "OpenGL ES 3.2 V@0615.0 (GIT@0c6347c6a9, I3e7929d012)";
constexpr const char kSpoofShading[] = "OpenGL ES GLSL ES 3.20";

const unsigned char* my_glGetString(unsigned int name) {
    switch (name) {
        case kGlVendor:
            return reinterpret_cast<const unsigned char*>(kSpoofVendor);
        case kGlRenderer:
            return reinterpret_cast<const unsigned char*>(kSpoofRenderer);
        case kGlVersion:
            return reinterpret_cast<const unsigned char*>(kSpoofVersion);
        case kGlShadingVersion:
            return reinterpret_cast<const unsigned char*>(kSpoofShading);
        case kGlExtensions:
            // Extensions list is driver-build specific; pass through rather
            // than invent a fingerprintable subset.
            break;
        default:
            break;
    }
    if (g_orig_gl_get_string != nullptr) return g_orig_gl_get_string(name);
    return nullptr;
}

bool TryHookResolved() {
    if (g_orig_gl_get_string != nullptr) return true;
    void* sym = DobbySymbolResolver(nullptr, "glGetString");
    if (sym == nullptr) return false;
    return DobbyHook(sym, (dobby_dummy_func_t)&my_glGetString,
                     (dobby_dummy_func_t*)&g_orig_gl_get_string) == 0;
}

bool IsGlLibrary(const char* name) {
    if (name == nullptr) return false;
    return strstr(name, "libGLESv2") != nullptr ||
           strstr(name, "libEGL") != nullptr;
}

void* my_dlopen(const char* filename, int flags) {
    void* handle = nullptr;
    if (g_orig_dlopen != nullptr) {
        handle = g_orig_dlopen(filename, flags);
    } else {
        handle = dlopen(filename, flags);
    }
    // Post-load trap: the linker has finished loading the GL driver, so
    // resolving + patching glGetString here cannot wedge Zygote locks.
    // Fail-open: a miss just leaves the real strings for this load.
    if (handle != nullptr && IsGlLibrary(filename)) {
        TryHookResolved();
    }
    return handle;
}

}  // namespace

void InstallGraphicsHooks() {
    // Pass 1: hook the already-resolved export without touching the linker.
    bool ok = TryHookResolved();
    // Pass 2: trap future loads of the GL driver (game engine loads EGL
    // lazily post-specialize). dlopen/dlopen_exten both funnel through the
    // "dlopen" export for DobbySymbolResolver.
    if (g_orig_dlopen == nullptr) {
        void* sym = DobbySymbolResolver(nullptr, "dlopen");
        if (sym != nullptr) {
            bool hooked =
                DobbyHook(sym, (dobby_dummy_func_t)&my_dlopen,
                          (dobby_dummy_func_t*)&g_orig_dlopen) == 0;
            DS_LOGI("graphics hooks: glGetString=%d dlopen_trap=%d", ok ? 1 : 0,
                    hooked ? 1 : 0);
            return;
        }
    }
    DS_LOGI("graphics hooks: glGetString=%d dlopen_trap=0", ok ? 1 : 0);
}

}  // namespace gs
