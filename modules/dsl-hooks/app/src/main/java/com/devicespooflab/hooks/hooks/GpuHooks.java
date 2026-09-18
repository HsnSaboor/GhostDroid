package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.utils.ConfigManager;

import de.robv.android.xposed.XC_MethodHook;
import de.robv.android.xposed.XposedBridge;
import de.robv.android.xposed.XposedHelpers;
import de.robv.android.xposed.callbacks.XC_LoadPackage;

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

    // EGL strings (EGL14.EGL_VENDOR).
    private static final int EGL_VENDOR = 0x3053;
    private static final int EGL_VERSION = 0x3054;

    public static void hook(XC_LoadPackage.LoadPackageParam lpparam) {
        // glGetString is only declared on GLES10/GLES20 (GLES30+ inherit
        // the static from GLES20 — hooking them throws NoSuchMethodError).
        hookGlGetString("android.opengl.GLES10", lpparam.classLoader);
        hookGlGetString("android.opengl.GLES20", lpparam.classLoader);
        hookEglQueryString(lpparam.classLoader);
        hookVulkanVersion(lpparam);
    }

    // VkPhysicalDeviceProperties.deviceName / driverVersion read via
    // android.hardware.vulkan.VulkanDeviceInfo. Cosmetic strings only —
    // VkDevice creation path untouched, accel keeps working.
    // S26 Ultra / Adreno 840: Vulkan 1.3 (confirmed Qualcomm SM8850 brief).
    private static void hookVulkanVersion(XC_LoadPackage.LoadPackageParam lpparam) {
        Class<?> info = XposedHelpers.findClassIfExists(
                "android.hardware.vulkan.VulkanDeviceInfo", lpparam.classLoader);
        if (info == null) return;
        try {
            XposedHelpers.findAndHookMethod(info, "getDeviceName",
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            param.setResult("Adreno (TM) 840");
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook VulkanDeviceInfo.getDeviceName: " + t);
        }
    }

    private static void hookGlGetString(String className, ClassLoader loader) {
        Class<?> clazz = XposedHelpers.findClassIfExists(className, loader);
        if (clazz == null) return;
        try {
            XposedHelpers.findAndHookMethod(clazz, "glGetString",
                    int.class,
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            int name = (Integer) param.args[0];
                            if (name == GL_VENDOR) {
                                param.setResult(ConfigManager.getGpuVendor());
                            } else if (name == GL_RENDERER) {
                                param.setResult(ConfigManager.getGpuRenderer());
                            } else if (name == GL_VERSION) {
                                // String label only — context/config stay real.
                                param.setResult(ConfigManager.getGpuVersion());
                            }
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook " + className + ".glGetString: " + t);
        }
    }

    private static void hookEglQueryString(ClassLoader loader) {
        Class<?> egl14 = XposedHelpers.findClassIfExists("android.opengl.EGL14", loader);
        Class<?> display = XposedHelpers.findClassIfExists("android.opengl.EGLDisplay", loader);
        if (egl14 == null || display == null) return;
        try {
            XposedHelpers.findAndHookMethod(egl14, "eglQueryString",
                    display, int.class,
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            int name = (Integer) param.args[1];
                            if (name == EGL_VENDOR) {
                                param.setResult(ConfigManager.getGpuVendor());
                            } else if (name == EGL_VERSION) {
                                // String label only — EGL init stays real.
                                param.setResult(ConfigManager.getGpuEglVersion());
                            }
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook EGL14.eglQueryString: " + t);
        }

        // UsbManager.getDeviceList(): DeviceInfoHW USB tab reads this AND
        // parses /sys/bus/usb via File.listFiles (opendir/getdents, bypasses
        // libc open hooks — covered natively by dir_hooks). Return empty so
        // the Java path shows a phone-like "no host USB" view.
        // ADB/function state untouched — debugging keeps working.
        Class<?> usbManager = XposedHelpers.findClassIfExists(
                "android.hardware.usb.UsbManager", loader);
        if (usbManager != null) {
            try {
                XposedHelpers.findAndHookMethod(usbManager, "getDeviceList",
                        new XC_MethodHook() {
                            @Override
                            protected void afterHookedMethod(MethodHookParam param) {
                                param.setResult(new java.util.HashMap<String, Object>());
                            }
                        });
            } catch (Throwable t) {
                XposedBridge.log(TAG + ": failed to hook UsbManager.getDeviceList: " + t);
            }
        }
    }
}
