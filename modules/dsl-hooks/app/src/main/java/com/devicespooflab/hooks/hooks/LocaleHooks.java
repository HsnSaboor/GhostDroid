package com.devicespooflab.hooks.hooks;

import android.os.Build;
import android.os.LocaleList;

import com.devicespooflab.hooks.utils.ConfigManager;

import java.util.Locale;
import java.util.TimeZone;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class LocaleHooks {

    private static final String TAG = "DeviceSpoofLab-Locale";

    public static void hook(HookContext lpparam) {
        hookTimeZone();
        hookLocale();
        if (Build.VERSION.SDK_INT >= 24) {
            hookLocaleList();
        }
    }

    private static void hookTimeZone() {
        try {
            Legacy.findAndHookMethod(TimeZone.class, "getDefault",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String tz = ConfigManager.getSystemProperty(
                                    "persist.sys.timezone", "America/Los_Angeles");
                            if (tz != null && !tz.isEmpty()) {
                                chain.replaceResult(TimeZone.getTimeZone(tz));
                            }
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook TimeZone.getDefault: " + t);
        }
    }

    private static void hookLocale() {
        try {
            Legacy.findAndHookMethod(Locale.class, "getDefault",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(buildLocale());
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook Locale.getDefault: " + t);
        }

        try {
            Legacy.findAndHookMethod(Locale.class, "getDefault",
                    Locale.Category.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(buildLocale());
                        }
                    });
        } catch (Throwable t) { /* Category overload is API 24+ */ }
    }

    private static void hookLocaleList() {
        try {
            Legacy.findAndHookMethod(LocaleList.class, "getDefault",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(new LocaleList(buildLocale()));
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook LocaleList.getDefault: " + t);
        }

        try {
            Legacy.findAndHookMethod(LocaleList.class, "getAdjustedDefault",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(new LocaleList(buildLocale()));
                        }
                    });
        } catch (Throwable t) { /* may be missing on some forks */ }
    }

    private static Locale buildLocale() {
        return new Locale.Builder()
                .setLanguage(ConfigManager.getLocaleLanguage())
                .setRegion(ConfigManager.getLocaleCountry())
                .build();
    }
}
