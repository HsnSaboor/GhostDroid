package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.bridge.HookContext;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.Legacy;

// Power / device-state queries ACE reads directly:
// PowerManager.isPowerSaveMode (probe line 6783 cache_key.is_power_save_mode),
// isDeviceIdleMode / isIgnoringBatteryOptimizations / isInteractive /
// isScreenOn / getLocationMode-style low-power tells. A flagship gaming
// phone is not in power-save: report full-power state everywhere.
// Fail-closed: any reflection error keeps the original value.
public final class PowerHooks {

    private static final String TAG = "DeviceSpoofLab-Power";

    private PowerHooks() {}

    public static void hook(HookContext lpparam) {
        hookPowerManager(lpparam);
        hookBatteryManagerExtra(lpparam);
    }

    private static void hookPowerManager(HookContext lpparam) {
        Class<?> pm = Legacy.findClassIfExists(
                "android.os.PowerManager", lpparam.classLoader);
        if (pm == null) {
            return;
        }
        pinBoolean(pm, "isPowerSaveMode", false);
        pinBoolean(pm, "isDeviceIdleMode", false);
        pinBoolean(pm, "isIgnoringBatteryOptimizations", true);
        pinBoolean(pm, "isInteractive", true);
        // Deprecated alias of isInteractive on older releases.
        pinBoolean(pm, "isScreenOn", true);
    }

    private static void hookBatteryManagerExtra(HookContext lpparam) {
        Class<?> bm = Legacy.findClassIfExists(
                "android.os.BatteryManager", lpparam.classLoader);
        if (bm == null) {
            return;
        }
        // Charging-current tells: a discharging S26 Ultra has no charge
        // current; report 0 rather than leaking the host rail value.
        Legacy.safeHook(TAG, "BatteryManager.getIntProperty-extra", () -> {
            Legacy.findAndHookMethod(bm, "getIntProperty", int.class,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                int id = chain.arg(0, -1);
                                // CURRENT_NOW (2) / CURRENT_AVERAGE (3) only.
                                // STATUS (6) belongs to BatteryHooks — never
                                // touch it here.
                                if (id == 2 || id == 3) {
                                    chain.replaceResult(0);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": current-now/avg failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void pinBoolean(Class<?> pm, String name, boolean value) {
        Legacy.safeHook(TAG, "PowerManager." + name, () -> {
            HookFramework.hookAllMethods(pm, name,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (result instanceof Boolean
                                        && (Boolean) result != value) {
                                    chain.replaceResult(value);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": " + name + " failed: " + t);
                            }
                        }
                    });
        });
    }
}
