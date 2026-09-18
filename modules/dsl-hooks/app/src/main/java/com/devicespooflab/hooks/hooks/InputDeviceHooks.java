package com.devicespooflab.hooks.hooks;

import android.view.InputDevice;

import de.robv.android.xposed.XC_MethodHook;
import de.robv.android.xposed.XposedBridge;
import de.robv.android.xposed.XposedHelpers;
import de.robv.android.xposed.callbacks.XC_LoadPackage;

public class InputDeviceHooks {

    private static final String TAG = "DeviceSpoofLab-InputDevice";

    public static void hook(XC_LoadPackage.LoadPackageParam lpparam) {
        try {
            XposedHelpers.findAndHookMethod(InputDevice.class, "getName",
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            String name = (String) param.getResult();
                            if (name != null && isEmulator(name)) {
                                param.setResult("Touch");
                            } else if (name != null) {
                                String clean = sanitizeDeviceLabel(name);
                                if (!clean.equals(name)) param.setResult(clean);
                            }
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook InputDevice.getName: " + t);
        }

        try {
            XposedHelpers.findAndHookMethod(InputDevice.class, "getDescriptor",
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            String desc = (String) param.getResult();
                            if (desc != null && isEmulator(desc)) {
                                param.setResult("0");
                            } else if (desc != null) {
                                String clean = sanitizeDeviceLabel(desc);
                                if (!clean.equals(desc)) param.setResult(clean);
                            }
                        }
                    });
        } catch (Throwable t) { /* getDescriptor signature stable */ }
    }

    private static boolean isEmulator(String s) {
        String lower = s.toLowerCase();
        return lower.contains("goldfish") || lower.contains("qemu")
                || lower.contains("ranchu") || lower.contains("vbox")
                || lower.contains("thinkpad") || lower.contains("elantech")
                || lower.contains("etps/2") || lower.contains("hda intel")
                || lower.contains("sunplusit") || lower.contains("xhci")
                || lower.contains("pnp0c") || lower.contains("lnxpwr")
                || lower.contains("lnxvideo") || lower.contains("i8042")
                || lower.contains("pcspkr") || lower.contains("isa006")
                || lower.contains("cachyos");
    }

    // Audio/USB descriptor strings read from the same leak surface
    // (/proc/bus/input/devices, /sys/bus/usb). Java-side rename so
    // DeviceInfoHW INPUT/USB tabs show neutral names; real audio route
    // and USB function stay untouched.
    public static String sanitizeDeviceLabel(String s) {
        if (s == null) return null;
        String lower = s.toLowerCase();
        if (lower.contains("hda intel")) return "Audio Device";
        if (lower.contains("sunplusit")) return "Camera Device";
        if (lower.contains("xhci")) return "USB Controller";
        if (lower.contains("thinkpad")) return "Keyboard Device";
        if (lower.contains("elantech") || lower.contains("etps/2")) return "Touch Device";
        if (lower.contains("cachyos")) return "USB Device";
        return s;
    }
}
