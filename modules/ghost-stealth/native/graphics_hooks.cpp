// graphics_hooks.cpp: native GLES string spoof (Adreno 840 / Qualcomm).
//
// Java GpuHooks cannot intercept C-to-C glGetString calls from libUE4.so /
// libanogs.so into libGLESv2.so, so ACE saw "Mesa Intel(R) UHD Graphics 620"
// on a device claiming Galaxy S26 Ultra. This module hooks the exported
// glGetString and returns S26-consistent strings for RENDERER/VENDOR/
// VERSION/SHADING. Every other enum and every failure path falls through
// to the original: fail-open rendering, fail-closed identity strings.
//
// RESOLVER-ONLY, SAFE FROM preAppSpecialize. Resolution is a single
// DobbySymbolResolver(nullptr, "glGetString") over already-loaded exports:
// zero linker work, zero locks taken. There is deliberately NO fallback
// that loads a library and NO trap on the dynamic loader:
//
// FORBIDDEN HISTORY (do NOT regress):
//   - Hooking the loader entry point SEGVs Mesa during EGL init
//     (gbm_mesa_bo_import tombstone, verified). This file must never hook
//     it, reference it, or wrap it.
//   - Forcing a load of libGLESv2.so with RTLD_NOW inside preAppSpecialize
//     wedges Zygote linker locks; the app RenderThread then wedges in
//     futex_wait at EGL init (ANR, verified 2026-09-21).
//
// If the GL driver is not loaded yet when InstallGraphicsHooks() runs, the
// install is a no-op miss and the real strings show until the next process
// start. That fail-open miss is the accepted cost of never touching the
// linker. Must mirror GpuHooks.java + spoof.conf gles.* entries.

#include "gs_state.h"

#include <dlfcn.h>

#include <cstring>

#include <dobby.h>

namespace gs {

namespace {

using GlStrFn = const unsigned char* (*)(unsigned int);

GlStrFn g_orig_gl_get_string = nullptr;

constexpr unsigned int kGlVendor = 0x1F00;
constexpr unsigned int kGlRenderer = 0x1F01;
constexpr unsigned int kGlVersion = 0x1F02;
constexpr unsigned int kGlExtensions = 0x1F03;
constexpr unsigned int kGlShadingVersion = 0x8B8C;

// S26 story: Snapdragon 8 Elite / Adreno 840, OpenGL ES 3.2.
constexpr const char kSpoofVendor[] = "Qualcomm";
constexpr const char kSpoofRenderer[] = "Adreno (TM) 840";
constexpr const char kSpoofVersion[] =
    "OpenGL ES 3.2 V@0615.0 (GIT@0c6347c6a9, I3e7929d012)";
constexpr const char kSpoofShading[] = "GLSL ES 3.20";

const unsigned char* my_glGetString(unsigned int name) {
    // Caller-sensitive: Android's own HwUI (libhwui.so / libEGL_mesa /
    // libGLESv2 internals) builds its grContext from the REAL driver
    // strings; feeding it Adreno strings on Intel Mesa trips
    // "Assertion failed: !grContext.get()" and aborts the process
    // (verified 2026-09-21). Only game/ACE callers (libUE4.so, libanogs.so,
    // libPubGAMS.so, application JNI) see spoofed strings. Everyone else —
    // including unknown callers — gets the real driver: fail-open rendering,
    // fail-closed identity only where ACE actually reads.
    Dl_info info{};
    bool isGameCaller = false;
    if (dladdr(__builtin_return_address(0), &info) &&
        info.dli_fname != nullptr) {
        const char* f = info.dli_fname;
        isGameCaller =
            strstr(f, "libUE4") != nullptr ||
            strstr(f, "libanogs") != nullptr ||
            strstr(f, "libPubGAMS") != nullptr ||
            strstr(f, "libgcloud") != nullptr ||
            strstr(f, "libTDataMaster") != nullptr ||
            strstr(f, "libapp") != nullptr;
    }
    if (isGameCaller) {
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
                break;
            default:
                break;
        }
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

}  // namespace

void InstallGraphicsHooks() {
    // Idempotent: TryHookResolved short-circuits once hooked, so this is
    // safe to call on every install path including preAppSpecialize.
    bool ok = TryHookResolved();
    DS_LOGI("graphics hooks: glGetString=%d", ok ? 1 : 0);
}

}  // namespace gs
