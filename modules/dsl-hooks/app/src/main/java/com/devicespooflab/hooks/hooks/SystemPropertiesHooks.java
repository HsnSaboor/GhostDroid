package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.utils.ConfigManager;

import java.util.Collections;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class SystemPropertiesHooks {

    private static final String TAG = "DeviceSpoofLab-SystemProps";
    private static final String SYSTEM_PROPERTIES_CLASS = "android.os.SystemProperties";
    private static final Set<Class<?>> HOOKED_CLASSES =
            Collections.newSetFromMap(new ConcurrentHashMap<Class<?>, Boolean>());

    // Waydroid-identifying properties must never leak: return the caller's
    // default instead. The fake_wifi kill-switch itself stays readable so
    // NetworkHooks can honor persist.waydroid.fake_wifi / fake_wifi.
    static boolean isDeniedProp(String key) {
        if (key == null || key.isEmpty()) {
            return false;
        }
        if (key.equals("fake_wifi") || key.endsWith(".fake_wifi")) {
            return false;
        }
        return key.startsWith("waydroid.")
                || key.startsWith("persist.waydroid.");
    }

    public static void hook(HookContext lpparam) {
        try {
            hookSystemProperties(lpparam.classLoader);

            try {
                ClassLoader systemClassLoader = ClassLoader.getSystemClassLoader();
                if (systemClassLoader != null && systemClassLoader != lpparam.classLoader) {
                    hookSystemProperties(systemClassLoader);
                }
            } catch (Exception ignored) {
            }
        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook SystemProperties: " + e.getMessage());
        }
    }

    private static void hookSystemProperties(ClassLoader classLoader) {
        Class<?> sysPropClass = Legacy.findClassIfExists(SYSTEM_PROPERTIES_CLASS, classLoader);

        if (sysPropClass == null) {
            return;
        }
        if (!HOOKED_CLASSES.add(sysPropClass)) {
            return;
        }

        // Hook get(String key)
        try {
            Legacy.findAndHookMethod(sysPropClass, "get",
                String.class,
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) throws Throwable {
                        try {
                            String key = (String) chain.arg(0, null);
                            if (isDeniedProp(key)) {
                                chain.replaceResult("");
                                return;
                            }
                            String originalValue = (String) result;
                            String spoofedValue = ConfigManager.getSystemProperty(key, null);

                            if (spoofedValue != null) {
                                chain.replaceResult(spoofedValue);
                            }
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": get(String) failed: " + t);
                        }
                    }
                });
        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook get(String): " + e.getMessage());
        }

        // Hook get(String key, String def)
        try {
            Legacy.findAndHookMethod(sysPropClass, "get",
                String.class, String.class,
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) throws Throwable {
                        try {
                            String key = (String) chain.arg(0, null);
                            String defaultValue = (String) chain.arg(1, null);
                            if (isDeniedProp(key)) {
                                chain.replaceResult(defaultValue);
                                return;
                            }
                            String spoofedValue = ConfigManager.getSystemProperty(key, null);

                            if (spoofedValue != null) {
                                chain.replaceResult(spoofedValue);
                            }
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": get(String, String) failed: " + t);
                        }
                    }
                });
        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook get(String, String): " + e.getMessage());
        }

        // Hook getInt(String key, int def)
        try {
            Legacy.findAndHookMethod(sysPropClass, "getInt",
                String.class, int.class,
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) throws Throwable {
                        try {
                            String key = (String) chain.arg(0, null);
                            if (isDeniedProp(key)) {
                                chain.replaceResult(chain.arg(1, 0));
                                return;
                            }
                            String spoofedValue = ConfigManager.getSystemProperty(key, null);

                            if (spoofedValue != null) {
                                try {
                                    int intValue = Integer.parseInt(spoofedValue);
                                    chain.replaceResult(intValue);
                                } catch (NumberFormatException e) {
                                    // Invalid int value, keep original
                                }
                            }
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": getInt failed: " + t);
                        }
                    }
                });
        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook getInt(String, int): " + e.getMessage());
        }

        // Hook getBoolean(String key, boolean def)
        try {
            Legacy.findAndHookMethod(sysPropClass, "getBoolean",
                String.class, boolean.class,
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) throws Throwable {
                        try {
                            String key = (String) chain.arg(0, null);
                            if (isDeniedProp(key)) {
                                chain.replaceResult(chain.arg(1, false));
                                return;
                            }
                            String spoofedValue = ConfigManager.getSystemProperty(key, null);

                            if (spoofedValue != null) {
                                // Handle both "true"/"false" and "1"/"0"
                                boolean boolValue = spoofedValue.equals("1") ||
                                                  spoofedValue.equalsIgnoreCase("true");
                                chain.replaceResult(boolValue);
                            }
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": getBoolean failed: " + t);
                        }
                    }
                });
        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook getBoolean(String, boolean): " + e.getMessage());
        }

        // Hook getLong(String key, long def)
        try {
            Legacy.findAndHookMethod(sysPropClass, "getLong",
                String.class, long.class,
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) throws Throwable {
                        try {
                            String key = (String) chain.arg(0, null);
                            if (isDeniedProp(key)) {
                                chain.replaceResult(chain.arg(1, 0L));
                                return;
                            }
                            String spoofedValue = ConfigManager.getSystemProperty(key, null);

                            if (spoofedValue != null) {
                                try {
                                    long longValue = Long.parseLong(spoofedValue);
                                    chain.replaceResult(longValue);
                                } catch (NumberFormatException e) {
                                    // Invalid long value, keep original
                                }
                            }
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": getLong failed: " + t);
                        }
                    }
                });
        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook getLong(String, long): " + e.getMessage());
        }
    }
}
