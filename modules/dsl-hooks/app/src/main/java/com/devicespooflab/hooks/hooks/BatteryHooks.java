package com.devicespooflab.hooks.hooks;

import android.os.BatteryManager;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

// Live charge level is passthrough; only the design counters are overridden.
// Every after-hook is fail-closed: try/catch + Legacy.log, original kept
// on error. getIntProperty/getLongProperty take a single int id (one
// overload each); isCharging/computeChargeTimeRemaining are no-arg.
public class BatteryHooks {

    private static final String TAG = "DeviceSpoofLab-Battery";

    public static void hook(HookContext lpparam) {
        Legacy.safeHook(TAG, "BatteryManager.getIntProperty", () -> {
            Legacy.findAndHookMethod(BatteryManager.class, "getIntProperty",
                    int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                int id = chain.arg(0, -1);
                                if (id == BatteryManager.BATTERY_PROPERTY_CHARGE_COUNTER) {
                                    long capUah = ConfigManager.getBatteryChargeCounterUah();
                                    chain.replaceResult((int) Math.min(Integer.MAX_VALUE, capUah));
                                } else if (id == BatteryManager.BATTERY_PROPERTY_CAPACITY) {
                                    chain.replaceResult(85);
                                } else if (id == BatteryManager.BATTERY_PROPERTY_STATUS) {
                                    chain.replaceResult(BatteryManager.BATTERY_STATUS_DISCHARGING);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getIntProperty failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "BatteryManager.getLongProperty", () -> {
            Legacy.findAndHookMethod(BatteryManager.class, "getLongProperty",
                    int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                int id = chain.arg(0, -1);
                                if (id == BatteryManager.BATTERY_PROPERTY_CHARGE_COUNTER) {
                                    chain.replaceResult(ConfigManager.getBatteryChargeCounterUah());
                                } else if (id == BatteryManager.BATTERY_PROPERTY_ENERGY_COUNTER) {
                                    chain.replaceResult(ConfigManager.getBatteryEnergyCounterNwh());
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getLongProperty failed: " + t);
                            }
                        }
                    });
        });

        // DeviceInfoHW Power profile row reads the design capacity, not the
        // live counter: isCharging()=false + 5000mAh design value.
        Legacy.safeHook(TAG, "BatteryManager.isCharging", () -> {
            Legacy.findAndHookMethod(BatteryManager.class, "isCharging",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(false);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": isCharging failed: " + t);
                            }
                        }
                    });
        });

        // BatteryManager.computeChargeTimeRemaining: host rail reports a
        // bogus remaining-time; a discharging phone with 85% reports -1
        // (unknown). Fail-closed.
        Legacy.safeHook(TAG, "BatteryManager.computeChargeTimeRemaining", () -> {
            Legacy.findAndHookMethod(BatteryManager.class,
                    "computeChargeTimeRemaining",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(-1L);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": computeChargeTimeRemaining failed: " + t);
                            }
                        }
                    });
        });

        // DeviceInfoHW "Power profile" row reads the static design capacity
        // from com.android.internal.os.PowerProfile (power_profile.xml),
        // not from BatteryManager. Report the 5000mAh S26 Ultra cell.
        Legacy.safeHook(TAG, "PowerProfile.getBatteryCapacity", () -> {
            Class<?> profile = Legacy.findClassIfExists(
                    "com.android.internal.os.PowerProfile",
                    lpparam.classLoader);
            if (profile == null) return;
            HookFramework.hookAllMethods(profile, "getBatteryCapacity",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(5000.0);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getBatteryCapacity failed: " + t);
                            }
                        }
                    });
        });
    }
}
