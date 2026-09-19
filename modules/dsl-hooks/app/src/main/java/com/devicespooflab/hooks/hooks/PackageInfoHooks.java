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
        if (appPm == null) return;

        HookFramework.Hook patcher = new HookFramework.Hook() {@Override
            public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                                if (result instanceof PackageInfo) {
                    patch((PackageInfo) result);
                }
            }
        };

        // getPackageInfo(String, int) and getPackageInfo(String, PackageInfoFlags)
        Legacy.safeHook(TAG, "getPackageInfo(String,int)", () -> {
            Legacy.findAndHookMethod(appPm, "getPackageInfo",
                    String.class, int.class, patcher);
        });

        try {
            Class<?> flags = Legacy.findClassIfExists(
                    "android.content.pm.PackageManager$PackageInfoFlags", lpparam.classLoader);
            if (flags != null) {
                Legacy.findAndHookMethod(appPm, "getPackageInfo",
                        String.class, flags, patcher);
            }
        } catch (Throwable t) { /* Android 13+ overload */ }
    }

    private static void hookGetInstallerPackageName(HookContext lpparam) {
        Class<?> appPm = Legacy.findClassIfExists(
                "android.app.ApplicationPackageManager", lpparam.classLoader);
        if (appPm == null) return;

        Legacy.safeHook(TAG, "getInstallerPackageName", () -> {
            Legacy.findAndHookMethod(appPm, "getInstallerPackageName",
                    String.class,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String packageName = (String) chain.arg(0, null);
                            if (shouldSpoofInstaller(lpparam, packageName)) {
                                chain.replaceResult(ConfigManager.getInstallerPackage());
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

        // InstallSourceInfo getters — hook each accessor to return Play Store.
        HookFramework.Hook playStoreHook = new HookFramework.Hook() {
            @Override
            public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                chain.replaceResult(ConfigManager.getInstallerPackage());
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
