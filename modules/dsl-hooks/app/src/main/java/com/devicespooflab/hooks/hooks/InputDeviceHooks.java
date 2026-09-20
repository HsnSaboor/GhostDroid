package com.devicespooflab.hooks.hooks;

import android.view.InputDevice;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class InputDeviceHooks {

    private static final String TAG = "DeviceSpoofLab-InputDevice";

    public static void hook(HookContext lpparam) {
        try {
            Legacy.findAndHookMethod(InputDevice.class, "getName",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String name = (String) result;
                            if (name != null && isEmulator(name)) {
                                chain.replaceResult("Touch");
                            } else if (name != null) {
                                String clean = sanitizeDeviceLabel(name);
                                if (!clean.equals(name)) chain.replaceResult(clean);
                            }
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook InputDevice.getName: " + t);
        }

        try {
            Legacy.findAndHookMethod(InputDevice.class, "getDescriptor",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String desc = (String) result;
                            if (desc != null && isEmulator(desc)) {
                                chain.replaceResult("0");
                            } else if (desc != null) {
                                String clean = sanitizeDeviceLabel(desc);
                                if (!clean.equals(desc)) chain.replaceResult(clean);
                            }
                        }
                    });
        } catch (Throwable t) { /* getDescriptor signature stable */ }

        hookSources();
        hookMotionRanges();
        hookVendorProduct();
    }

    // Flagship touchscreen view: every device reports touchscreen sources.
    private static void hookSources() {
        Legacy.safeHook(TAG, "InputDevice.getSources", () -> {
            Legacy.findAndHookMethod(InputDevice.class, "getSources",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult(InputDevice.SOURCE_CLASS_POINTER
                                        | InputDevice.SOURCE_TOUCHSCREEN);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getSources failed: " + t);
                            }
                        }
                    });
        });
    }

    // Touchscreen axis ranges (X/Y); MotionRange has no public ctor so it is
    // built reflectively and any failure leaves the original result.
    private static void hookMotionRanges() {
        Legacy.safeHook(TAG, "InputDevice.getMotionRange", () -> {
            HookFramework.hookAllMethods(InputDevice.class, "getMotionRange",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (result != null) {
                                    return;
                                }
                                int axis = chain.arg(0, -1);
                                Object range = motionRangeFor(axis);
                                if (range != null) {
                                    chain.replaceResult(range);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getMotionRange failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "InputDevice.getMotionRanges", () -> {
            Legacy.findAndHookMethod(InputDevice.class, "getMotionRanges",
                    new HookFramework.Hook() {@Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                java.util.List<Object> orig = (java.util.List<Object>) result;
                                if (orig != null && !orig.isEmpty()) {
                                    return;
                                }
                                java.util.List<Object> ranges = new java.util.ArrayList<>(2);
                                Object x = motionRangeFor(android.view.MotionEvent.AXIS_X);
                                Object y = motionRangeFor(android.view.MotionEvent.AXIS_Y);
                                if (x != null) {
                                    ranges.add(x);
                                }
                                if (y != null) {
                                    ranges.add(y);
                                }
                                if (!ranges.isEmpty()) {
                                    chain.replaceResult(ranges);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getMotionRanges failed: " + t);
                            }
                        }
                    });
        });
    }

    private static Object motionRangeFor(int axis) {
        try {
            if (axis != android.view.MotionEvent.AXIS_X
                    && axis != android.view.MotionEvent.AXIS_Y) {
                return null;
            }
            float max = axis == android.view.MotionEvent.AXIS_X ? 1080.0f : 2400.0f;
            Class<?> rangeClass =
                    Class.forName("android.view.InputDevice$MotionRange");
            for (java.lang.reflect.Constructor<?> c : rangeClass.getDeclaredConstructors()) {
                Class<?>[] types = c.getParameterTypes();
                if (types.length < 6) {
                    continue;
                }
                if (types[0] != int.class || types[1] != int.class) {
                    continue;
                }
                Object[] args = new Object[types.length];
                args[0] = axis;
                args[1] = InputDevice.SOURCE_TOUCHSCREEN;
                float[] floats = {0.0f, max, 0.0f, 0.0f, 1.0f};
                int fi = 0;
                boolean ok = true;
                for (int i = 2; i < types.length; i++) {
                    if (types[i] == float.class && fi < floats.length) {
                        args[i] = floats[fi++];
                    } else if (types[i] == int.class) {
                        args[i] = 0;
                    } else {
                        ok = false;
                        break;
                    }
                }
                if (!ok) {
                    continue;
                }
                try {
                    c.setAccessible(true);
                    return c.newInstance(args);
                } catch (Throwable ignored) {
                }
            }
            return null;
        } catch (Throwable t) {
            Legacy.log(TAG + ": motionRangeFor failed: " + t);
            return null;
        }
    }

    // Samsung VID (04e8): matches the spoofed Samsung device identity.
    private static void hookVendorProduct() {
        Legacy.safeHook(TAG, "InputDevice.getVendorId", () -> {
            Legacy.findAndHookMethod(InputDevice.class, "getVendorId",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult(0x04e8);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getVendorId failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "InputDevice.getProductId", () -> {
            Legacy.findAndHookMethod(InputDevice.class, "getProductId",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult(0x04e8);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getProductId failed: " + t);
                            }
                        }
                    });
        });
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
