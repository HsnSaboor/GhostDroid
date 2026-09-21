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

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class BatteryIntentHooks {

    private static final String TAG = "DeviceSpoofLab-BatteryIntent";

    public static void hook(HookContext lpparam) {
        hookRegisterReceiver(lpparam);
        hookStickyBroadcast(lpparam);
    }

    private static void hookRegisterReceiver(HookContext lpparam) {
        HookFramework.Hook rewrite = new HookFramework.Hook() {@Override
            public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                rewriteBatteryIntent((Intent) result);
            }
        };
        // registerReceiver overloads vary by SDK — hook all, not each one.
        // Context itself is abstract (no body to hook); the concrete
        // ContextWrapper/ContextImpl overrides carry the implementation.
        Class<?> wrapper = Legacy.findClassIfExists(
                "android.content.ContextWrapper", lpparam.classLoader);
        if (wrapper != null) {
            HookFramework.hookAllMethods(wrapper, "registerReceiver", rewrite);
            HookFramework.hookAllMethods(wrapper, "registerReceiverAsUser", rewrite);
        }
        Class<?> impl = Legacy.findClassIfExists(
                "android.app.ContextImpl", lpparam.classLoader);
        if (impl != null) {
            HookFramework.hookAllMethods(impl, "registerReceiver", rewrite);
            HookFramework.hookAllMethods(impl, "registerReceiverAsUser", rewrite);
        }
        // Async path: apps that register a NON-null receiver get the sticky
        // via onReceive(), never via the registerReceiver return. Hook the
        // Intent extras themselves so both delivery paths see spoofed values.
        hookBatteryIntentExtras(lpparam);
    }

    // Rewrite ACTION_BATTERY_CHANGED extras at read time. Covers async
    // onReceive delivery (DeviceInfoHW Battery tab: Status/Power source
    // still leaked Charging/AC with only the registerReceiver hook).
    private static void hookBatteryIntentExtras(HookContext lpparam) {
        Class<?> intentClass = Legacy.findClassIfExists(
                "android.content.Intent", lpparam.classLoader);
        if (intentClass == null) return;
        HookFramework.Hook intSpoof = new HookFramework.Hook() {@Override
            public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                Object self = chain.thisObject();
                if (!(self instanceof Intent)) return;
                Intent intent = (Intent) self;
                if (!Intent.ACTION_BATTERY_CHANGED.equals(intent.getAction())) return;
                String key = (String) chain.arg(0, null);
                Integer spoofed = spoofedIntExtra(key);
                if (spoofed != null) chain.replaceResult(spoofed);
            }
        };
        Legacy.safeHook(TAG, "Intent.getIntExtra", () -> {
            Legacy.findAndHookMethod(intentClass, "getIntExtra",
                    String.class, int.class, intSpoof);
        });
        HookFramework.Hook stringSpoof = new HookFramework.Hook() {@Override
            public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                Object self = chain.thisObject();
                if (!(self instanceof Intent)) return;
                Intent intent = (Intent) self;
                if (!Intent.ACTION_BATTERY_CHANGED.equals(intent.getAction())) return;
                String key = (String) chain.arg(0, null);
                if (BatteryManager.EXTRA_TECHNOLOGY.equals(key)) {
                    chain.replaceResult("Li-ion");
                }
            }
        };
        Legacy.safeHook(TAG, "Intent.getStringExtra", () -> {
            Legacy.findAndHookMethod(intentClass, "getStringExtra",
                    String.class, stringSpoof);
        });
    }

    private static Integer spoofedIntExtra(String key) {
        if (key == null) return null;
        if (BatteryManager.EXTRA_LEVEL.equals(key)) return 85;
        if (BatteryManager.EXTRA_SCALE.equals(key)) return 100;
        if (BatteryManager.EXTRA_STATUS.equals(key)) {
            return BatteryManager.BATTERY_STATUS_DISCHARGING;
        }
        if (BatteryManager.EXTRA_HEALTH.equals(key)) {
            return BatteryManager.BATTERY_HEALTH_GOOD;
        }
        if (BatteryManager.EXTRA_PLUGGED.equals(key)) return 0;
        if (BatteryManager.EXTRA_TEMPERATURE.equals(key)) return 350;
        if (BatteryManager.EXTRA_VOLTAGE.equals(key)) return 3600;
        return null;
    }

    private static void hookStickyBroadcast(HookContext lpparam) {
        // Covered by hookAllMethods above (registerReceiverAsUser included).
        if (lpparam == null) {
            return;
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
