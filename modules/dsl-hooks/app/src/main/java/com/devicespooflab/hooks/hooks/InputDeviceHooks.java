package com.devicespooflab.hooks.hooks;

import android.view.InputDevice;

import java.util.ArrayList;
import java.util.List;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

// Peripheral-leak cloak: Waydroid exposes wayland_pointer (SOURCE_MOUSE)
// + wayland_keyboard (SOURCE_KEYBOARD). PUBG InputManager reads sources
// directly -> emulator matchmaking. Every such device must present as a
// Samsung sec_touchscreen touchscreen with no mouse/keyboard bits.
public class InputDeviceHooks {

    private static final String TAG = "DeviceSpoofLab-InputDevice";

    static final String CLOAKED_NAME = "sec_touchscreen";
    static final int SAMSUNG_VENDOR = 0x04e8;
    static final int SAMSUNG_PRODUCT = 0x04e8;

    public static void hook(HookContext lpparam) {
        hookNames();
        hookSources();
        hookDeviceInventory(lpparam);
        hookInputManager(lpparam);
        hookVirtualFlag();
        hookMotionRanges();
        hookVendorProduct();
    }

    // ---- name / descriptor rename ------------------------------------

    private static void hookNames() {
        try {
            Legacy.findAndHookMethod(InputDevice.class, "getName",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                String name = (String) result;
                                if (name == null) {
                                    return;
                                }
                                String cloaked = cloakedName(name);
                                if (!cloaked.equals(name)) {
                                    chain.replaceResult(cloaked);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getName failed: " + t);
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
                            try {
                                if (error != null) {
                                    return;
                                }
                                String desc = (String) result;
                                if (desc == null) {
                                    return;
                                }
                                String clean = sanitizeDeviceLabel(desc);
                                if (!clean.equals(desc)) {
                                    chain.replaceResult(clean);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDescriptor failed: " + t);
                            }
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook InputDevice.getDescriptor: " + t);
        }
    }

    // Device-inventory enumeration ACE DEX walks directly:
    // getDeviceIds (int[] incl. -1 virtual keyboard), getDevice(id).
    // Fail-closed throughout: any error leaves the original inventory.
    private static void hookDeviceInventory(HookContext lpparam) {
        Legacy.safeHook(TAG, "InputDevice.getDeviceIds", () -> {
            HookFramework.hookAllMethods(InputDevice.class, "getDeviceIds",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null || !(result instanceof int[])) {
                                    return;
                                }
                                chain.replaceResult(filterDeviceIds(lpparam, (int[]) result));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDeviceIds failed: " + t);
                            }
                        }
                    });
        });
        Legacy.safeHook(TAG, "InputDevice.getDevice", () -> {
            HookFramework.hookAllMethods(InputDevice.class, "getDevice",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null || result == null) {
                                    return;
                                }
                                String name = (String) Legacy.callMethod(
                                        result, "getName");
                                // getName hook already cloaks Waydroid names;
                                // a surviving emulator name means the hook
                                // missed (e.g. cached instance): null it.
                                if (name != null && isPeripheralLeak(name)) {
                                    chain.replaceResult(null);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDevice failed: " + t);
                            }
                        }
                    });
        });
    }

    // Drop ids whose live device still reports a peripheral leak or stays
    // virtual after cloaking (e.g. -1 virtual keyboard). Pure ids carry no
    // host strings, so names resolve via getDevice(id) in-target; any
    // resolution failure keeps the id (fail-closed, inventory intact).
    // Negative ids (virtual keyboard) never exist on retail enumerations.
    static int[] filterDeviceIds(HookContext lpparam, int[] ids) {
        if (ids == null) {
            return new int[0];
        }
        Class<?> clazz = resolveInputDevice(lpparam);
        if (clazz == null) {
            return ids;
        }
        List<Integer> kept = new ArrayList<>(ids.length);
        for (int id : ids) {
            try {
                if (id < 0) {
                    continue;
                }
                Object device = Legacy.callStaticMethod(clazz, "getDevice", id);
                if (device == null) {
                    kept.add(id);
                    continue;
                }
                String name = (String) Legacy.callMethod(device, "getName");
                if (name != null && isPeripheralLeak(name)) {
                    continue;
                }
                Object virtualFlag = Legacy.callMethod(device, "isVirtual");
                if (Boolean.TRUE.equals(virtualFlag)) {
                    continue;
                }
                kept.add(id);
            } catch (Throwable t) {
                Legacy.log(TAG + ": filterDeviceIds kept id " + id + ": " + t);
                kept.add(id);
            }
        }
        int[] out = new int[kept.size()];
        for (int i = 0; i < kept.size(); i++) {
            out[i] = kept.get(i);
        }
        return out;
    }

    private static Class<?> resolveInputDevice(HookContext lpparam) {
        try {
            if (lpparam != null && lpparam.classLoader != null) {
                Class<?> viaLoader = Legacy.findClassIfExists(
                        "android.view.InputDevice", lpparam.classLoader);
                if (viaLoader != null) {
                    return viaLoader;
                }
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": resolveInputDevice failed: " + t);
        }
        return InputDevice.class;
    }

    // InputManager.getInputDevice()/getInputDeviceIds() is the path PUBG
    // actually walks (InputManager, not the static InputDevice facade).
    // Same cloak: strip peripheral leaks at resolution, filter ids.
    // Fail-closed: errors keep the original inventory/device.
    private static void hookInputManager(HookContext lpparam) {
        Class<?> manager = Legacy.findClassIfExists(
                "android.hardware.input.InputManager", lpparam.classLoader);
        if (manager == null) {
            return;
        }
        Legacy.safeHook(TAG, "InputManager.getInputDevice", () -> {
            HookFramework.hookAllMethods(manager, "getInputDevice",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null || result == null) {
                                    return;
                                }
                                String name = (String) Legacy.callMethod(
                                        result, "getName");
                                if (name != null && isPeripheralLeak(name)) {
                                    chain.replaceResult(null);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": InputManager.getInputDevice failed: " + t);
                            }
                        }
                    });
        });
        Legacy.safeHook(TAG, "InputManager.getInputDeviceIds", () -> {
            HookFramework.hookAllMethods(manager, "getInputDeviceIds",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null || !(result instanceof int[])) {
                                    return;
                                }
                                chain.replaceResult(filterDeviceIds(lpparam, (int[]) result));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": InputManager.getInputDeviceIds failed: " + t);
                            }
                        }
                    });
        });
    }

    // Virtual devices (-1 virtual keyboard) betray software injection.
    // Real phone enumerations expose no virtual keyboards; report false.
    private static void hookVirtualFlag() {
        Legacy.safeHook(TAG, "InputDevice.isVirtual", () -> {
            Legacy.findAndHookMethod(InputDevice.class, "isVirtual",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (!Boolean.FALSE.equals(result)) {
                                    chain.replaceResult(false);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": isVirtual failed: " + t);
                            }
                        }
                    });
        });
    }

    // Flagship touchscreen view: every device reports touchscreen sources.
    // Peripheral bits (MOUSE/KEYBOARD/TRACKBALL/...) are stripped so
    // wayland_pointer/_keyboard never surface SOURCE_MOUSE/SOURCE_KEYBOARD.
    // Non-touch devices gain SOURCE_TOUCHSCREEN; error/null fall back to
    // pure SOURCE_TOUCHSCREEN (fail-closed to the phone story).
    private static void hookSources() {
        Legacy.safeHook(TAG, "InputDevice.getSources", () -> {
            Legacy.findAndHookMethod(InputDevice.class, "getSources",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                int sources = result instanceof Integer
                                        ? (Integer) result
                                        : 0;
                                chain.replaceResult(cloakedSources(sources));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getSources failed: " + t);
                            }
                        }
                    });
        });
    }

    static int cloakedSources(int sources) {
        int stripped = sources
                & ~InputDevice.SOURCE_MOUSE
                & ~InputDevice.SOURCE_KEYBOARD
                & ~InputDevice.SOURCE_TRACKBALL
                & ~InputDevice.SOURCE_DPAD
                & ~InputDevice.SOURCE_GAMEPAD
                & ~InputDevice.SOURCE_JOYSTICK
                & ~InputDevice.SOURCE_STYLUS
                & ~InputDevice.SOURCE_BLUETOOTH_STYLUS
                & ~InputDevice.SOURCE_HDMI;
        return stripped | InputDevice.SOURCE_TOUCHSCREEN;
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
                                List<Object> orig = (List<Object>) result;
                                if (orig != null && !orig.isEmpty()) {
                                    return;
                                }
                                List<Object> ranges = new ArrayList<>(2);
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
                                if (error != null) {
                                    return;
                                }
                                if (!Integer.valueOf(SAMSUNG_VENDOR).equals(result)) {
                                    chain.replaceResult(SAMSUNG_VENDOR);
                                }
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
                                if (error != null) {
                                    return;
                                }
                                if (!Integer.valueOf(SAMSUNG_PRODUCT).equals(result)) {
                                    chain.replaceResult(SAMSUNG_PRODUCT);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getProductId failed: " + t);
                            }
                        }
                    });
        });
    }

    // Waydroid peripheral pair + classic host-input tells. Wayland names
    // carry no host-vendor token, so they match first, before the generic
    // emulator table below.
    static boolean isPeripheralLeak(String s) {
        if (s == null) {
            return false;
        }
        String lower = s.toLowerCase();
        return lower.contains("wayland_pointer")
                || lower.contains("wayland_keyboard")
                || lower.contains("wayland")
                || isEmulator(lower);
    }

    static String cloakedName(String name) {
        if (name == null) {
            return CLOAKED_NAME;
        }
        if (isPeripheralLeak(name)) {
            return CLOAKED_NAME;
        }
        return sanitizeDeviceLabel(name);
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
