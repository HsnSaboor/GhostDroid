package com.devicespooflab.hooks.hooks;

import android.hardware.Sensor;
import android.hardware.SensorManager;

import java.util.ArrayList;
import java.util.List;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class SensorHooks {

    private static final String TAG = "DeviceSpoofLab-Sensor";
    private static final String[] DENY = {"goldfish", "ranchu", "emulator", "qemu", "vbox",
            "thinkpad", "elantech", "etps", "hda intel", "sunplusit", "xhci",
            "pnp0c", "lnxpwr", "lnxvideo", "i8042", "pcspkr", "isa006",
            "acpi", "cachyos", "mesa", "intel", "linux"};

    // S26 Ultra style sensor set: presence + neutral vendor so DeviceInfoHW
    // SENSORS tab is fully populated. No fake data injected (values come
    // from the real HAL pass-through); only host-named entries filtered.

    public static void hook(HookContext lpparam) {
        try {
            Legacy.findAndHookMethod(SensorManager.class, "getSensorList",
                    int.class,
                    new HookFramework.Hook() {@Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            List<Sensor> orig = (List<Sensor>) result;
                            chain.replaceResult(filter(orig));
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook getSensorList: " + t);
        }

        try {
            Legacy.findAndHookMethod(SensorManager.class, "getDefaultSensor",
                    int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            Sensor s = (Sensor) result;
                            if (s != null && isEmulatorSensor(s)) {
                                chain.replaceResult(null);
                            }
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook getDefaultSensor: " + t);
        }

        try {
            Legacy.findAndHookMethod(SensorManager.class, "getDefaultSensor",
                    int.class, boolean.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            Sensor s = (Sensor) result;
                            if (s != null && isEmulatorSensor(s)) {
                                chain.replaceResult(null);
                            }
                        }
                    });
        } catch (Throwable t) { /* version-specific overload */ }
    }

    private static List<Sensor> filter(List<Sensor> orig) {
        if (orig == null) return new ArrayList<>();
        List<Sensor> kept = new ArrayList<>(orig.size());
        for (Sensor s : orig) {
            if (s == null) continue;
            if (!isEmulatorSensor(s)) kept.add(s);
        }
        return kept;
    }

    private static boolean isEmulatorSensor(Sensor s) {
        String name = nullSafe(s.getName()).toLowerCase();
        String vendor = nullSafe(s.getVendor()).toLowerCase();
        for (String token : DENY) {
            if (name.contains(token) || vendor.contains(token)) return true;
        }
        return false;
    }

    private static String nullSafe(String s) {
        return s == null ? "" : s;
    }
}
