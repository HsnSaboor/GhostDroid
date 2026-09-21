package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

// Spoofs GPU vendor/renderer strings that DeviceInfoHW-style apps read via
// android.opengl GLES glGetString() / EGL eglQueryString(). Without this the
// calls fall through to host Mesa ("Intel" / "Mesa Intel(R) ...").
// WebView WebGL is covered separately by WebViewHooks (JS getParameter).
public class GpuHooks {

    private static final String TAG = "DeviceSpoofLab-Gpu";

    // GL strings (GLES10.GL_VENDOR etc.).
    private static final int GL_VENDOR = 0x1F00;
    private static final int GL_RENDERER = 0x1F01;
    private static final int GL_VERSION = 0x1F02;
    private static final int GL_SHADING_LANGUAGE_VERSION = 0x8B8C;
    private static final int GL_EXTENSIONS = 0x1F03;

    // DEBUG_RENDERER (0x824E): KHR_debug unmasked query that bypasses
    // glGetString. Same Adreno story or the host leaks through.
    private static final int GL_DEBUG_RENDERER = 0x824E;

    // EGL strings (EGL14.EGL_VENDOR).
    private static final int EGL_VENDOR = 0x3053;
    private static final int EGL_VERSION = 0x3054;
    private static final int EGL_EXTENSIONS = 0x3055;

    // Vulkan 1.3.0 packed (variant 0, major 1, minor 3, patch 0).
    private static final int VULKAN_API_1_3 = 0x403000;

    public static void hook(HookContext lpparam) {
        // glGetString is only declared on GLES10/GLES20 (GLES30+ inherit
        // the static from GLES20 — hooking them throws NoSuchMethodError).
        // GLES30/31/32 DO redeclare glGetStringi (indexed extensions):
        // hook the declaration site per class.
        hookGlGetString("android.opengl.GLES10", lpparam.classLoader);
        hookGlGetString("android.opengl.GLES20", lpparam.classLoader);
        hookGlGetStringi("android.opengl.GLES30", lpparam.classLoader);
        hookGlGetStringi("android.opengl.GLES31", lpparam.classLoader);
        hookGlGetStringi("android.opengl.GLES32", lpparam.classLoader);
        hookEglQueryString(lpparam.classLoader);
        hookEglGetString(lpparam.classLoader);
        hookVulkanVersion(lpparam);
    }

    // VkPhysicalDeviceProperties.deviceName / driverVersion read via
    // android.hardware.vulkan.VulkanDeviceInfo. Cosmetic strings only —
    // VkDevice creation path untouched, accel keeps working.
    // S26 Ultra / Adreno 840: Vulkan 1.3 (confirmed Qualcomm SM8850 brief).
    private static void hookVulkanVersion(HookContext lpparam) {
        Class<?> info = Legacy.findClassIfExists(
                "android.hardware.vulkan.VulkanDeviceInfo", lpparam.classLoader);
        if (info == null) return;
        try {
            Legacy.findAndHookMethod(info, "getDeviceName",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult("Adreno (TM) 840");
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": VulkanDeviceInfo.getDeviceName failed: " + t);
                            }
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook VulkanDeviceInfo.getDeviceName: " + t);
        }
        try {
            HookFramework.hookAllMethods(info, "getApiVersion",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult(VULKAN_API_1_3);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": VulkanDeviceInfo.getApiVersion failed: " + t);
                            }
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook VulkanDeviceInfo.getApiVersion: " + t);
        }
    }

    private static void hookGlGetString(String className, ClassLoader loader) {
        Class<?> clazz = Legacy.findClassIfExists(className, loader);
        if (clazz == null) return;
        // glGetString(int) single overload; exact-signature hook correct.
        // Fail-closed: try/catch + Legacy.log, original kept on error.
        Legacy.safeHook(TAG, className + ".glGetString", () -> {
            Legacy.findAndHookMethod(clazz, "glGetString",
                    int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                int name = chain.arg(0, -1);
                                if (name == GL_VENDOR) {
                                    chain.replaceResult(ConfigManager.getGpuVendor());
                                } else if (name == GL_RENDERER) {
                                    chain.replaceResult(ConfigManager.getGpuRenderer());
                                } else if (name == GL_VERSION) {
                                    // String label only — context/config stay real.
                                    chain.replaceResult(ConfigManager.getGpuVersion());
                                } else if (name == GL_SHADING_LANGUAGE_VERSION) {
                                    chain.replaceResult(ConfigManager.getGpuShadingVersion());
                                } else if (name == GL_DEBUG_RENDERER) {
                                    chain.replaceResult(ConfigManager.getGpuRenderer());
                                } else if (name == GL_EXTENSIONS) {
                                    String stripped = stripHostGlTokens((String) result);
                                    if (stripped != null) {
                                        chain.replaceResult(stripped);
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": " + className + ".glGetString failed: " + t);
                            }
                        }
                    });
        });
    }

    // GLES30/31/32 indexed query (glGetStringi): host extension tokens
    // leak the Waydroid driver through per-index reads. Strip host tokens;
    // fail-closed (null keeps the original string).
    private static void hookGlGetStringi(String className, ClassLoader loader) {
        Class<?> clazz = Legacy.findClassIfExists(className, loader);
        if (clazz == null) return;
        Legacy.safeHook(TAG, className + ".glGetStringi", () -> {
            Legacy.findAndHookMethod(clazz, "glGetStringi",
                    int.class, int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null || !(result instanceof String)) {
                                    return;
                                }
                                String stripped = stripHostGlTokens((String) result);
                                if (stripped == null) {
                                    chain.replaceResult("");
                                } else {
                                    chain.replaceResult(stripped);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": " + className + ".glGetStringi failed: " + t);
                            }
                        }
                    });
        });
    }

    // EGL14.eglGetString (unmasked display query): eglQueryString is the
    // EGL15 form; detectors call this one directly. Same vendor/version
    // story. Fail-closed.
    private static void hookEglGetString(ClassLoader loader) {
        Class<?> egl14 = Legacy.findClassIfExists("android.opengl.EGL14", loader);
        Class<?> display = Legacy.findClassIfExists("android.opengl.EGLDisplay", loader);
        if (egl14 == null || display == null) return;
        try {
            Legacy.findAndHookMethod(egl14, "eglGetString",
                    display, int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                int name = chain.arg(1, -1);
                                if (name == EGL_VENDOR) {
                                    chain.replaceResult(ConfigManager.getGpuVendor());
                                } else if (name == EGL_VERSION) {
                                    chain.replaceResult(ConfigManager.getGpuEglVersion());
                                } else if (name == EGL_EXTENSIONS) {
                                    String stripped = stripHostGlTokens((String) result);
                                    if (stripped != null) {
                                        chain.replaceResult(stripped);
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": eglGetString failed: " + t);
                            }
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook EGL14.eglGetString: " + t);
        }
    }

    // Drop host-GPU extension tokens (Mesa/Intel/llvmpipe/swrast) that
    // leak the Waydroid host through eglQueryString(EGL_EXTENSIONS).
    // Fail-closed: null on any error keeps the original string.
    static String stripHostGlTokens(String extensions) {
        try {
            if (extensions == null || extensions.isEmpty()) {
                return null;
            }
            String[] tokens = extensions.split(" ");
            StringBuilder kept = new StringBuilder(extensions.length());
            boolean changed = false;
            for (String token : tokens) {
                if (token.isEmpty()) {
                    continue;
                }
                String lower = token.toLowerCase();
                if (lower.contains("mesa") || lower.contains("intel")
                        || lower.contains("llvmpipe") || lower.contains("swrast")) {
                    changed = true;
                    continue;
                }
                if (kept.length() > 0) {
                    kept.append(' ');
                }
                kept.append(token);
            }
            return changed ? kept.toString() : null;
        } catch (Throwable t) {
            Legacy.log(TAG + ": stripHostGlTokens failed: " + t);
            return null;
        }
    }

    private static void hookEglQueryString(ClassLoader loader) {
        Class<?> egl14 = Legacy.findClassIfExists("android.opengl.EGL14", loader);
        Class<?> display = Legacy.findClassIfExists("android.opengl.EGLDisplay", loader);
        if (egl14 == null || display == null) return;
        try {
            Legacy.findAndHookMethod(egl14, "eglQueryString",
                    display, int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                int name = chain.arg(1, -1);
                                if (name == EGL_VENDOR) {
                                    chain.replaceResult(ConfigManager.getGpuVendor());
                                } else if (name == EGL_VERSION) {
                                    // String label only — EGL init stays real.
                                    chain.replaceResult(ConfigManager.getGpuEglVersion());
                                } else if (name == EGL_EXTENSIONS) {
                                    String stripped = stripHostGlTokens((String) result);
                                    if (stripped != null) {
                                        chain.replaceResult(stripped);
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": eglQueryString failed: " + t);
                            }
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook EGL14.eglQueryString: " + t);
        }

        // UsbManager.getDeviceList(): DeviceInfoHW USB tab reads this AND
        // parses /sys/bus/usb via File.listFiles (opendir/getdents, bypasses
        // libc open hooks — covered natively by dir_hooks). Return empty so
        // the Java path shows a phone-like "no host USB" view.
        // ADB/function state untouched — debugging keeps working.
        // Single no-arg overload. Fail-closed: try/catch + Legacy.log.
        Class<?> usbManager = Legacy.findClassIfExists(
                "android.hardware.usb.UsbManager", loader);
        if (usbManager != null) {
            Legacy.safeHook(TAG, "UsbManager.getDeviceList", () -> {
                Legacy.findAndHookMethod(usbManager, "getDeviceList",
                        new HookFramework.Hook() {@Override
                            public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    chain.replaceResult(new java.util.HashMap<String, Object>());
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": UsbManager.getDeviceList failed: " + t);
                                }
                            }
                        });
            });
        }
    }
}
