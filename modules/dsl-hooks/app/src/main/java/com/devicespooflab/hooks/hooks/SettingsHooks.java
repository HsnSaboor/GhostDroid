package com.devicespooflab.hooks.hooks;

import android.content.ContentResolver;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class SettingsHooks {

    private static final int SPOOF_ANDROID_ID = 1;
    private static final int SPOOF_GSF_ID = 1 << 1;
    private static final int SPOOF_BLUETOOTH_ADDRESS = 1 << 2;
    private static final int SPOOF_DEVICE_NAMES = 1 << 3;

    private static final String ANDROID_ID = "android_id";
    private static final String GSF_ID = "gsf_id";
    private static final String BLUETOOTH_ADDRESS = "bluetooth_address";
    private static final String BLUETOOTH_NAME = "bluetooth_name";
    private static final String DEVICE_NAME = "device_name";

    public static void hook(HookContext lpparam) {
        hookClass(lpparam, "android.provider.Settings$Secure",
                SPOOF_ANDROID_ID | SPOOF_GSF_ID | SPOOF_BLUETOOTH_ADDRESS | SPOOF_DEVICE_NAMES);
        hookClass(lpparam, "android.provider.Settings$System",
                SPOOF_BLUETOOTH_ADDRESS | SPOOF_DEVICE_NAMES);
        hookClass(lpparam, "android.provider.Settings$Global",
                SPOOF_BLUETOOTH_ADDRESS | SPOOF_DEVICE_NAMES);
    }

    private static void hookClass(HookContext lpparam,
                                  String className,
                                  int spoofFlags) {
        Class<?> clazz = Legacy.findClassIfExists(className, lpparam.classLoader);
        if (clazz == null) return;

        try {
            Legacy.findAndHookMethod(clazz, "getString",
                    ContentResolver.class, String.class,
                    new HookFramework.BeforeHook() {@Override
                        public void before(HookFramework.HookChain chain) {
                            String name = (String) chain.arg(1, null);
                            applySpoof(chain, name, spoofFlags);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(clazz, "getString",
                    ContentResolver.class, String.class, String.class,
                    new HookFramework.BeforeHook() {@Override
                        public void before(HookFramework.HookChain chain) {
                            String name = (String) chain.arg(1, null);
                            applySpoof(chain, name, spoofFlags);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(clazz, "getStringForUser",
                    ContentResolver.class, String.class, int.class,
                    new HookFramework.BeforeHook() {@Override
                        public void before(HookFramework.HookChain chain) {
                            String name = (String) chain.arg(1, null);
                            applySpoof(chain, name, spoofFlags);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }
    }

    private static void applySpoof(HookFramework.HookChain chain, String name, int spoofFlags) {
        if (name == null) return;

        if ((spoofFlags & SPOOF_ANDROID_ID) != 0 && ANDROID_ID.equals(name)) {
            String v = ConfigManager.getAndroidId();
            if (v != null) chain.replaceResult(v);
            return;
        }

        if ((spoofFlags & SPOOF_GSF_ID) != 0 && GSF_ID.equals(name)) {
            String v = ConfigManager.getGSFId();
            if (v != null) chain.replaceResult(v);
            return;
        }

        if ((spoofFlags & SPOOF_BLUETOOTH_ADDRESS) != 0 && BLUETOOTH_ADDRESS.equals(name)) {
            String mac = ConfigManager.getBluetoothMacAddress();
            if (mac != null) chain.replaceResult(mac.toUpperCase());
            return;
        }

        if ((spoofFlags & SPOOF_DEVICE_NAMES) != 0
                && (BLUETOOTH_NAME.equals(name) || DEVICE_NAME.equals(name))) {
            String model = ConfigManager.getBuildModel();
            if (model != null) chain.replaceResult(model);
        }
    }
}
