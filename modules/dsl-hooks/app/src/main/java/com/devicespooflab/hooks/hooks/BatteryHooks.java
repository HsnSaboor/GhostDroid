package com.devicespooflab.hooks.hooks;

import android.os.BatteryManager;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

// Live charge level is passthrough; only the design counters are overridden.
public class BatteryHooks {

    private static final String TAG = "DeviceSpoofLab-Battery";

    public static void hook(HookContext lpparam) {
        Legacy.safeHook(TAG, "BatteryManager.getIntProperty", () -> {
            Legacy.findAndHookMethod(BatteryManager.class, "getIntProperty",
                    int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            int id = chain.arg(0, -1);
                            if (id == BatteryManager.BATTERY_PROPERTY_CHARGE_COUNTER) {
                                long capUah = ConfigManager.getBatteryChargeCounterUah();
                                chain.replaceResult((int) Math.min(Integer.MAX_VALUE, capUah));
                            } else if (id == BatteryManager.BATTERY_PROPERTY_CAPACITY) {
                                chain.replaceResult(85);
                            } else if (id == BatteryManager.BATTERY_PROPERTY_STATUS) {
                                chain.replaceResult(BatteryManager.BATTERY_STATUS_DISCHARGING);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "BatteryManager.getLongProperty", () -> {
            Legacy.findAndHookMethod(BatteryManager.class, "getLongProperty",
                    int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            int id = chain.arg(0, -1);
                            if (id == BatteryManager.BATTERY_PROPERTY_CHARGE_COUNTER) {
                                chain.replaceResult(ConfigManager.getBatteryChargeCounterUah());
                            } else if (id == BatteryManager.BATTERY_PROPERTY_ENERGY_COUNTER) {
                                chain.replaceResult(ConfigManager.getBatteryEnergyCounterNwh());
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
                            chain.replaceResult(false);
                        }
                    });
        });
    }
}
