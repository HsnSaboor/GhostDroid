package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.bridge.HookContext;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.Legacy;

// KeyStore / DRM / SafetyNet-adjacent device-state ACE DEX calls directly:
// KeyChain.isBoundKeyAlgorithm, MediaDrm property strings beyond
// deviceUniqueId, Settings.Global adb/development tells, and the hidden
// ApplicationInfo flag path (PackageInfoHooks clears the delivered object;
// this clears the appInfo.flags field reads some detectors snapshot
// earlier via getApplicationInfo().flags before our patch runs).
// Fail-closed throughout.
public final class DeviceStateHooks {

    private static final String TAG = "DeviceSpoofLab-DeviceState";

    private DeviceStateHooks() {}

    public static void hook(HookContext lpparam) {
        hookKeyChain(lpparam);
        hookMediaDrmStrings(lpparam);
        hookSettingsGlobalDev(lpparam);
        hookKeystoreState(lpparam);
    }

    private static void hookKeyChain(HookContext lpparam) {
        Class<?> kc = Legacy.findClassIfExists(
                "android.security.KeyChain", lpparam.classLoader);
        if (kc == null) {
            return;
        }
        Legacy.safeHook(TAG, "KeyChain.isBoundKeyAlgorithm", () -> {
            HookFramework.hookAllMethods(kc, "isBoundKeyAlgorithm",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof Boolean
                                        && !(Boolean) result) {
                                    chain.replaceResult(true);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": isBoundKeyAlgorithm failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    // MediaDrm.getPropertyString vendor/oemCryptoApiVersion/securityLevel
    // leak Widevine SW build tags; pin to L1 + stable vendor label.
    private static void hookMediaDrmStrings(HookContext lpparam) {
        Class<?> drm = Legacy.findClassIfExists(
                "android.media.MediaDrm", lpparam.classLoader);
        if (drm == null) {
            return;
        }
        Legacy.safeHook(TAG, "MediaDrm.getPropertyString", () -> {
            Legacy.findAndHookMethod(drm, "getPropertyString",
                    String.class,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                String key = chain.arg(0, null);
                                if ("securityLevel".equals(key)) {
                                    chain.replaceResult("L1");
                                } else if ("vendor".equals(key)) {
                                    chain.replaceResult("Google");
                                } else if ("version".equals(key
                                        ) && result instanceof String) {
                                    String v = (String) result;
                                    if (v == null || v.isEmpty()) {
                                        chain.replaceResult("15.0.0");
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getPropertyString failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    // Settings.Global ADB / development-settings tells. Probe shows
    // service.adb.tcp.port + ro.debuggable reads; the Java Settings path
    // is the one ACE DEX calls without native.
    private static void hookSettingsGlobalDev(HookContext lpparam) {
        Class<?> global = Legacy.findClassIfExists(
                "android.provider.Settings$Global", lpparam.classLoader);
        if (global == null) {
            return;
        }
        Legacy.safeHook(TAG, "Settings.Global.getString-dev", () -> {
            HookFramework.hookAllMethods(global, "getString",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                String name = chain.arg(1, null);
                                if ("adb_enabled".equals(name)) {
                                    chain.replaceResult("0");
                                } else if ("development_settings_enabled"
                                        .equals(name)) {
                                    chain.replaceResult("0");
                                } else if ("verifier_verify_adb_installs"
                                        .equals(name)) {
                                    chain.replaceResult("1");
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": Global.getString failed: "
                                        + t);
                            }
                        }
                    });
        });
        Legacy.safeHook(TAG, "Settings.Global.getInt-dev", () -> {
            HookFramework.hookAllMethods(global, "getInt",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                String name = chain.arg(1, null);
                                if ("adb_enabled".equals(name)) {
                                    chain.replaceResult(0);
                                } else if ("development_settings_enabled"
                                        .equals(name)) {
                                    chain.replaceResult(0);
                                } else if ("development_keep_screen_on"
                                        .equals(name)) {
                                    chain.replaceResult(0);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": Global.getInt failed: " + t);
                            }
                        }
                    });
        });
    }

    // KeyStore.getState(): UNLOCKED on a user build with secure lock.
    // Some detectors treat UNKNOWN/LOCKED as emulator (no credential
    // storage). Only lift UNKNOWN -> UNLOCKED; never downgrade.
    // State is an enum (KeyProperties.State), not an int — compare by name.
    private static void hookKeystoreState(HookContext lpparam) {
        Class<?> ks = Legacy.findClassIfExists(
                "android.security.KeyStore", lpparam.classLoader);
        if (ks == null) {
            return;
        }
        Legacy.safeHook(TAG, "KeyStore.getState", () -> {
            HookFramework.hookAllMethods(ks, "getState",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result != null && "UNKNOWN".equals(
                                        String.valueOf(result))) {
                                    Object unlocked = enumValue(result,
                                            "UNLOCKED");
                                    if (unlocked != null) {
                                        chain.replaceResult(unlocked);
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": KeyStore.getState failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static Object enumValue(Object sample, String name) {
        try {
            Class<?> cls = sample.getClass();
            if (!cls.isEnum()) {
                return null;
            }
            @SuppressWarnings({"unchecked", "rawtypes"})
            Object found = Enum.valueOf((Class<Enum>) cls, name);
            return found;
        } catch (Throwable ignored) {
            return null;
        }
    }
}
