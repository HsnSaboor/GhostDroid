// graphics_hooks.cpp: native GLES string spoof (Adreno 840 / Qualcomm).
//
// Java GpuHooks cannot intercept C-to-C glGetString calls from libUE4.so /
// libanogs.so into libGLESv2.so, so ACE saw "Mesa Intel(R) UHD Graphics 620"
// on a device claiming Galaxy S26 Ultra. This module hooks the exported
// glGetString in libGLESv2.so (+ libGLESv2_angle.so fallback) and returns
// S26-consistent strings for RENDERER/VENDOR/VERSION. Every other enum and
// every failure path falls through to the original: fail-open rendering,
// fail-closed identity strings.

#include "gs_state.h"

#include <dlfcn.h>

#include <dobby.h>

namespace gs {

namespace {

using GlStrFn = const unsigned char* (*)(unsigned int);

GlStrFn g_orig_gl_get_string = nullptr;

constexpr unsigned int kGlVendor = 0x1F00;
constexpr unsigned int kGlRenderer = 0x1F01;
constexpr unsigned int kGlVersion = 0x1F02;
constexpr unsigned int kGlShadingVersion = 0x8B8C;
constexpr unsigned int kGlExtensions = 0x1F03;

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

bool HookInLibrary(const char* lib) {
    void* handle = dlopen(lib, RTLD_NOW | RTLD_NOLOAD);
    if (handle == nullptr) handle = dlopen(lib, RTLD_NOW);
    if (handle == nullptr) return false;
    void* sym = dlsym(handle, "glGetString");
    if (sym == nullptr) return false;
    if (g_orig_gl_get_string != nullptr) return true;  // already patched
    return DobbyHook(sym, (dobby_dummy_func_t)&my_glGetString,
                     (dobby_dummy_func_t*)&g_orig_gl_get_string) == 0;
}

}  // namespace

void InstallGraphicsHooks() {
    bool ok = HookInLibrary("libGLESv2.so");
    if (!ok) ok = HookInLibrary("libGLESv2_angle.so");
    DS_LOGI("graphics hooks: glGetString=%d", ok ? 1 : 0);
}

}  // namespace gs
