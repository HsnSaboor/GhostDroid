package com.devicespooflab.hooks.hooks;

import android.content.pm.ApplicationInfo;
import android.content.pm.PackageInfo;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

// Install times reported as 60-120 days ago, derived from android_id so the
// per-install value stays stable across reads.
public class PackageInfoHooks {

    private static final String TAG = "DeviceSpoofLab-PackageInfo";
    private static final long DAY_MS = 86_400_000L;

    public static void hook(HookContext lpparam) {
        hookPackageInfoFields(lpparam);
        hookGetInstallerPackageName(lpparam);
        hookGetInstallSourceInfo(lpparam);
    }

    private static void hookPackageInfoFields(HookContext lpparam) {
        Class<?> appPm = Legacy.findClassIfExists(
                "android.app.ApplicationPackageManager", lpparam.classLoader);
        if (appPm == null) {
            return;
        }

        HookFramework.Hook patcher = new HookFramework.Hook() {@Override
            public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                try {
                    if (result instanceof PackageInfo) {
                        patch((PackageInfo) result);
                    }
                } catch (Throwable t) {
                    Legacy.log(TAG + ": PackageInfo patch failed: " + t);
                }
            }
        };

        // getPackageInfo: hook ALL overloads (String,int / String,flags /
        // getPackageInfoAsUser variants). A per-signature hook misses the
        // AsUser path detectors use to bypass the base patch. Every
        // overload returns PackageInfo so one shared patcher covers all.
        // Fail-closed: patch errors keep the original object.
        Legacy.safeHook(TAG, "getPackageInfo", () -> {
            HookFramework.hookAllMethods(appPm, "getPackageInfo", patcher);
        });
        // AsUser sibling bypasses the base patch on multi-user aware
        // detectors: same PackageInfo patcher covers every overload.
        Legacy.safeHook(TAG, "getPackageInfoAsUser", () -> {
            HookFramework.hookAllMethods(appPm, "getPackageInfoAsUser", patcher);
        });

        // ApplicationInfo.FLAG_DEBUGGABLE leaks a debuggable/eng build:
        // clear it on every getApplicationInfo overload (fail-closed).
        Legacy.safeHook(TAG, "getApplicationInfo", () -> {
            HookFramework.hookAllMethods(appPm, "getApplicationInfo",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof ApplicationInfo) {
                                    clearDebuggable((ApplicationInfo) result);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getApplicationInfo flag clear failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookGetInstallerPackageName(HookContext lpparam) {
        Class<?> appPm = Legacy.findClassIfExists(
                "android.app.ApplicationPackageManager", lpparam.classLoader);
        if (appPm == null) return;

        // Single (String) overload; fail-closed with try/catch + log.
        Legacy.safeHook(TAG, "getInstallerPackageName", () -> {
            Legacy.findAndHookMethod(appPm, "getInstallerPackageName",
                    String.class,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                String packageName = (String) chain.arg(0, null);
                                if (shouldSpoofInstaller(lpparam, packageName)) {
                                    chain.replaceResult(ConfigManager.getInstallerPackage());
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getInstallerPackageName failed: " + t);
                            }
                        }
                    });
        });
    }

    private static boolean shouldSpoofInstaller(HookContext lpparam,
                                                String packageName) {
        if (packageName == null || lpparam.packageName == null) {
            return false;
        }
        if (!lpparam.packageName.equals(packageName)) {
            return false;
        }
        if (ConfigManager.isOwnPackageProcess(lpparam.packageName)) {
            return false;
        }
        if (lpparam.appInfo == null) {
            return true;
        }
        int systemFlags = ApplicationInfo.FLAG_SYSTEM | ApplicationInfo.FLAG_UPDATED_SYSTEM_APP;
        return (lpparam.appInfo.flags & systemFlags) == 0;
    }

    private static void hookGetInstallSourceInfo(HookContext lpparam) {
        Class<?> appPm = Legacy.findClassIfExists(
                "android.app.ApplicationPackageManager", lpparam.classLoader);
        if (appPm == null) return;

        Class<?> sourceInfo = Legacy.findClassIfExists(
                "android.content.pm.InstallSourceInfo", lpparam.classLoader);
        if (sourceInfo == null) return;

        // InstallSourceInfo getters — hook each accessor to return Play
        // Store. No-arg accessors; fail-closed with try/catch + log.
        HookFramework.Hook playStoreHook = new HookFramework.Hook() {
            @Override
            public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                try {
                    if (error != null) {
                        return;
                    }
                    chain.replaceResult(ConfigManager.getInstallerPackage());
                } catch (Throwable t) {
                    Legacy.log(TAG + ": InstallSourceInfo getter failed: " + t);
                }
            }
        };

        for (String getter : new String[]{
                "getInstallingPackageName",
                "getInitiatingPackageName",
                "getOriginatingPackageName"
        }) {
            try {
                Legacy.findAndHookMethod(sourceInfo, getter, playStoreHook);
            } catch (Throwable t) { /* getter may be absent on older Android */ }
        }
    }

    private static void patch(PackageInfo pi) {
        if (pi == null) return;
        long install = stableInstallTime();
        pi.firstInstallTime = install;
        // lastUpdateTime: random within 0–14 days after install, stable per app.
        pi.lastUpdateTime = install + (Math.abs(stableHash(pi.packageName)) % (14L * DAY_MS));
        if (pi.applicationInfo != null) {
            clearDebuggable(pi.applicationInfo);
        }
    }

    private static void clearDebuggable(ApplicationInfo ai) {
        try {
            ai.flags &= ~ApplicationInfo.FLAG_DEBUGGABLE;
        } catch (Throwable t) {
            Legacy.log(TAG + ": clearDebuggable failed: " + t);
        }
    }

    private static long stableInstallTime() {
        // 60–120 days before now, stable per android_id.
        long seed = Math.abs(ConfigManager.getFingerprintSeed());
        long offsetDays = 60 + (seed % 61);
        return System.currentTimeMillis() - offsetDays * DAY_MS;
    }

    private static long stableHash(String s) {
        if (s == null) return 0L;
        long h = 1469598103934665603L;
        for (int i = 0; i < s.length(); i++) {
            h ^= s.charAt(i);
            h *= 1099511628211L;
        }
        return h;
    }
}
