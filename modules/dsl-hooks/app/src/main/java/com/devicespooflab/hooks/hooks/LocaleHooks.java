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

    // TimeZone.getDefault / Locale.getDefault(+Category) /
    // LocaleList.getDefault+getAdjustedDefault: hookAllMethods covers
    // every overload (Locale.getDefault has no-arg + Category forms).
    // Fail-closed: try/catch + Legacy.log, original kept on error.
    private static void hookTimeZone() {
        Legacy.safeHook(TAG, "TimeZone.getDefault", () -> {
            HookFramework.hookAllMethods(TimeZone.class, "getDefault",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                String tz = ConfigManager.getSystemProperty(
                                        "persist.sys.timezone", "America/Los_Angeles");
                                if (tz != null && !tz.isEmpty()) {
                                    chain.replaceResult(TimeZone.getTimeZone(tz));
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": TimeZone.getDefault failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookLocale() {
        Legacy.safeHook(TAG, "Locale.getDefault", () -> {
            HookFramework.hookAllMethods(Locale.class, "getDefault",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(buildLocale());
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": Locale.getDefault failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookLocaleList() {
        Legacy.safeHook(TAG, "LocaleList.getDefault", () -> {
            HookFramework.hookAllMethods(LocaleList.class, "getDefault",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(new LocaleList(buildLocale()));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": LocaleList.getDefault failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "LocaleList.getAdjustedDefault", () -> {
            HookFramework.hookAllMethods(LocaleList.class, "getAdjustedDefault",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(new LocaleList(buildLocale()));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": LocaleList.getAdjustedDefault failed: " + t);
                            }
                        }
                    });
        });
    }

    private static Locale buildLocale() {
        return new Locale.Builder()
                .setLanguage(ConfigManager.getLocaleLanguage())
                .setRegion(ConfigManager.getLocaleCountry())
                .build();
    }
}
