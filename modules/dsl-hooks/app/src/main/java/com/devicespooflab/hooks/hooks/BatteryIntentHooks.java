// DeviceInfoHW BATTERY tab reads the sticky ACTION_BATTERY_CHANGED
// broadcast (Level/Health/Status/Power source/Technology/Temperature/
// Voltage). The sticky intent is cached in ActivityManagerService — Java
// per-app hooks can't rewrite it. Spoof here at the receiver: rewrite
// extras to S26 Ultra values (discharging, 85%, 5000mAh Li-ion).
//
// Call order safety: Application.attach fires before any receiver is
// queried, so values are fixed before DeviceInfoHW reads them.
// Real charging hardware untouched (we only rewrite the Intent extras
// delivered to this process).
package com.devicespooflab.hooks.hooks;

import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.os.BatteryManager;

import com.devicespooflab.hooks.utils.ConfigManager;

import de.robv.android.xposed.XC_MethodHook;
import de.robv.android.xposed.XposedBridge;
import de.robv.android.xposed.XposedHelpers;
import de.robv.android.xposed.callbacks.XC_LoadPackage;

public class BatteryIntentHooks {

    private static final String TAG = "DeviceSpoofLab-BatteryIntent";

    public static void hook(XC_LoadPackage.LoadPackageParam lpparam) {
        hookRegisterReceiver(lpparam);
        hookStickyBroadcast(lpparam);
    }

    private static void hookRegisterReceiver(XC_LoadPackage.LoadPackageParam lpparam) {
        try {
            XposedHelpers.findAndHookMethod(Context.class, "registerReceiver",
                    BroadcastReceiver.class, IntentFilter.class,
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            Intent intent = (Intent) param.getResult();
                            rewriteBatteryIntent(intent);
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook Context.registerReceiver: " + t);
        }
        try {
            XposedHelpers.findAndHookMethod(Context.class, "registerReceiver",
                    BroadcastReceiver.class, IntentFilter.class, String.class,
                    android.os.Handler.class,
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            Intent intent = (Intent) param.getResult();
                            rewriteBatteryIntent(intent);
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook Context.registerReceiver(4-arg): " + t);
        }
    }

    private static void hookStickyBroadcast(XC_LoadPackage.LoadPackageParam lpparam) {
        try {
            XposedHelpers.findAndHookMethod(Context.class, "registerReceiverAsUser",
                    BroadcastReceiver.class, android.os.UserHandle.class,
                    IntentFilter.class, String.class, android.os.Handler.class,
                    new XC_MethodHook() {
                        @Override
                        protected void afterHookedMethod(MethodHookParam param) {
                            Intent intent = (Intent) param.getResult();
                            rewriteBatteryIntent(intent);
                        }
                    });
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": failed to hook Context.registerReceiverAsUser: " + t);
        }
    }

    static void rewriteBatteryIntent(Intent intent) {
        if (intent == null) return;
        if (!Intent.ACTION_BATTERY_CHANGED.equals(intent.getAction())) return;
        try {
            intent.putExtra(BatteryManager.EXTRA_LEVEL, 85);
            intent.putExtra(BatteryManager.EXTRA_SCALE, 100);
            intent.putExtra(BatteryManager.EXTRA_STATUS,
                    BatteryManager.BATTERY_STATUS_DISCHARGING);
            intent.putExtra(BatteryManager.EXTRA_HEALTH,
                    BatteryManager.BATTERY_HEALTH_GOOD);
            intent.putExtra(BatteryManager.EXTRA_PLUGGED, 0);
            intent.putExtra(BatteryManager.EXTRA_TECHNOLOGY, "Li-ion");
            intent.putExtra(BatteryManager.EXTRA_TEMPERATURE, 350);
            intent.putExtra(BatteryManager.EXTRA_VOLTAGE, 3600);
            intent.putExtra(BatteryManager.EXTRA_PRESENT, true);
        } catch (Throwable ignored) {
        }
    }

    @SuppressWarnings("unused")
    static long batteryCapacityUah() {
        try {
            return ConfigManager.getBatteryCapacityUah();
        } catch (Throwable ignored) {
            return 5000000L;
        }
    }
}
