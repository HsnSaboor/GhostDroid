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
    // Emulator-vendor tokens (qemu/goldfish/ranchu/genymotion/bluestacks/
    // vbox/memu/microvirt/gamematrix/cloud-phone stacks) are denied the same
    // way: ACE DEX reads these keys straight from SystemProperties, and a
    // present-but-emulator value is the fingerprint (probe_2594 shows the
    // qemu/goldfish/ranchu + cph/gamematrix/cloud sweeps). Absent keys return
    // the caller default either way; denying only matters when the host
    // actually sets them.
    static boolean isDeniedProp(String key) {
        if (key == null || key.isEmpty()) {
            return false;
        }
        if (key.equals("fake_wifi") || key.endsWith(".fake_wifi")) {
            return false;
        }
        if (key.startsWith("waydroid.")
                || key.startsWith("persist.waydroid.")) {
            return true;
        }
        return isEmulatorVendorProp(key);
    }

    private static boolean isEmulatorVendorProp(String key) {
        String lower = key.toLowerCase(java.util.Locale.US);
        for (String token : EMULATOR_PROP_TOKENS) {
            if (lower.contains(token)) {
                return true;
            }
        }
        return false;
    }

    private static final String[] EMULATOR_PROP_TOKENS = {
            "qemu", "goldfish", "ranchu", "genymotion", "bluestacks",
            "vbox", "memu", "microvirt", "gamematrix", "cloudgame",
            "docker", "lgsys", "cloudvm", "pscloud", "ecalc",
            ".cph", "cph.", "cph_", "hm_", ".hm.",
            "mesa", "debug.gles", "debug.egl", "ro.hwui", "gralloc.gbm",
            "ro.hardware.alter", "ro.bootmode",
            "opporom", ".emui", "miui", "vivo", "meizu", "lenovo",
            "nubia", "aa.romver", "rom.id", "lewa", "gnrom", "tyd.kbstyle",
    };

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
            hookExecFallbacks();
        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook SystemProperties: " + e.getMessage());
        }
    }

    // ByShell fallback (getprop via Runtime.exec / ProcessBuilder): the
    // Java SystemProperties hook never sees these. Rewrite explicit
    // denied-key args to a benign passthrough key before spawn so the
    // child prints an empty/innocent value instead of the waydroid leak.
    // Fail-closed: any error leaves the command untouched.
    // NOT covered here (native-owned, see hole table): bare `getprop`
    // full-dump, popen/__system_property_get, and /proc file reads —
    // those belong to the native libc layer (ghost-stealth).
    private static void hookExecFallbacks() {
        Legacy.safeHook(TAG, "Runtime.exec", () -> {
            HookFramework.hookAllMethods(Runtime.class, "exec",
                    new HookFramework.BeforeHook() {
                        @Override
                        public void before(HookFramework.HookChain chain) {
                            try {
                                rewriteExecArgs(chain);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": Runtime.exec rewrite failed: " + t);
                            }
                        }
                    });
        });
        Legacy.safeHook(TAG, "ProcessBuilder.start", () -> {
            Legacy.findAndHookMethod(ProcessBuilder.class, "start",
                    new HookFramework.BeforeHook() {
                        @Override
                        public void before(HookFramework.HookChain chain) {
                            try {
                                Object self = chain.thisObject();
                                if (self instanceof ProcessBuilder) {
                                    sanitizeCommandList(((ProcessBuilder) self).command());
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": ProcessBuilder rewrite failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void rewriteExecArgs(HookFramework.HookChain chain) {
        for (int i = 0; i < chain.argCount(); i++) {
            Object arg = chain.arg(i, null);
            if (arg instanceof String) {
                String rewritten = sanitizeShellCommand((String) arg);
                if (rewritten != null) {
                    chain.setArg(i, rewritten);
                }
            } else if (arg instanceof String[]) {
                sanitizeCommandArray((String[]) arg);
            } else if (arg instanceof java.util.List) {
                sanitizeCommandList((java.util.List<?>) arg);
            }
        }
    }

    @SuppressWarnings("unchecked")
    private static void sanitizeCommandList(java.util.List<?> cmd) {
        try {
            if (!(cmd instanceof java.util.List)
                    || ((java.util.List<?>) cmd).isEmpty()) {
                return;
            }
            java.util.List<Object> mutable = (java.util.List<Object>) cmd;
            for (int i = 0; i < mutable.size(); i++) {
                Object el = mutable.get(i);
                if (!(el instanceof String)) {
                    continue;
                }
                String token = (String) el;
                if (isDeniedProp(token) && previousIsGetprop(mutable, i)) {
                    mutable.set(i, "persist.sys.timezone");
                }
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": sanitizeCommandList failed: " + t);
        }
    }

    private static void sanitizeCommandArray(String[] cmd) {
        try {
            for (int i = 0; i < cmd.length; i++) {
                if (cmd[i] != null && isDeniedProp(cmd[i])
                        && previousIsGetprop(cmd, i)) {
                    cmd[i] = "persist.sys.timezone";
                }
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": sanitizeCommandArray failed: " + t);
        }
    }

    private static String sanitizeShellCommand(String cmd) {
        try {
            if (cmd == null || !cmd.contains("getprop")) {
                return null;
            }
            String[] tokens = cmd.split("\\s+");
            boolean changed = false;
            for (int i = 0; i < tokens.length; i++) {
                if (isDeniedProp(stripQuotes(tokens[i]))
                        && previousIsGetprop(tokens, i)) {
                    tokens[i] = "persist.sys.timezone";
                    changed = true;
                }
            }
            return changed ? String.join(" ", tokens) : null;
        } catch (Throwable t) {
            Legacy.log(TAG + ": sanitizeShellCommand failed: " + t);
            return null;
        }
    }

    private static boolean previousIsGetprop(java.util.List<Object> tokens, int idx) {
        try {
            for (int i = idx - 1; i >= 0; i--) {
                Object t = tokens.get(i);
                if (!(t instanceof String)) {
                    continue;
                }
                String s = ((String) t).trim();
                if (s.isEmpty() || "-Z".equals(s)) {
                    continue;
                }
                return "getprop".equals(s) || s.endsWith("/getprop");
            }
            return false;
        } catch (Throwable ignored) {
            return false;
        }
    }

    private static boolean previousIsGetprop(String[] tokens, int idx) {
        try {
            for (int i = idx - 1; i >= 0; i--) {
                String s = tokens[i] == null ? "" : tokens[i].trim();
                if (s.isEmpty() || "-Z".equals(s)) {
                    continue;
                }
                return "getprop".equals(s) || s.endsWith("/getprop");
            }
            return false;
        } catch (Throwable ignored) {
            return false;
        }
    }

    private static String stripQuotes(String token) {
        if (token == null) {
            return null;
        }
        if (token.length() >= 2
                && ((token.startsWith("\"") && token.endsWith("\""))
                        || (token.startsWith("'") && token.endsWith("'")))) {
            return token.substring(1, token.length() - 1);
        }
        return token;
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
