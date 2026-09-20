package com.devicespooflab.hooks.hooks;

import android.content.pm.PackageManager;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;
import com.devicespooflab.hooks.utils.ConfigManager;

// Hides root/cloak tooling from applist scans: strips deny-set packages
// from list-returning PackageManager APIs and throws
// NameNotFoundException for direct getPackageInfo(denied). Fail-closed:
// any reflection error leaves the original result untouched.
public class ApplistHooks {

    private static final String TAG = "DeviceSpoofLab-Applist";

    private static final Set<String> DENY = new HashSet<>(Arrays.asList(
            "com.topjohnwu.magisk",
            "io.github.huskydg.magisk",
            "com.hv.magisk",
            "org.lsposed.manager",
            "org.lsposed.lspatch",
            "org.frknkrc44.hma_oss",
            "com.tsng.hidemyapplist",
            "de.robv.android.xposed.installer",
            "re.frida.server",
            "com.frida.server",
            "org.maus.lspd",
            "moe.shizuku.privileged.api"));

    public static void hook(HookContext lpparam) {
        if (!ConfigManager.isIdentifierEnabled("applist_hide")) {
            return;
        }
        Class<?> appPm = Legacy.findClassIfExists(
                "android.app.ApplicationPackageManager", lpparam.classLoader);
        if (appPm == null) {
            return;
        }
        hookInstalledPackages(appPm);
        hookInstalledApplications(appPm);
        hookQueryIntentActivities(appPm);
        hookGetPackageInfo(appPm);
    }

    private static boolean denied(String pkg) {
        if (pkg == null) {
            return false;
        }
        String lower = pkg.toLowerCase();
        for (String d : DENY) {
            if (lower.equals(d) || lower.startsWith(d + ".")) {
                return true;
            }
        }
        return lower.contains("magisk")
                || lower.contains("lsposed")
                || lower.contains("lspd")
                || lower.contains("shamiko")
                || lower.contains("frida");
    }

    private static String packageOf(Object entry) {
        try {
            String pkg = (String) Legacy.getObjectField(entry, "packageName");
            if (pkg != null) {
                return pkg;
            }
        } catch (Throwable ignored) {
        }
        try {
            Object appInfo = Legacy.getObjectField(entry, "applicationInfo");
            if (appInfo != null) {
                return (String) Legacy.getObjectField(appInfo, "packageName");
            }
        } catch (Throwable ignored) {
        }
        try {
            Object activityInfo = Legacy.getObjectField(entry, "activityInfo");
            if (activityInfo != null) {
                return (String) Legacy.getObjectField(activityInfo, "packageName");
            }
        } catch (Throwable ignored) {
        }
        try {
            Object serviceInfo = Legacy.getObjectField(entry, "serviceInfo");
            if (serviceInfo != null) {
                return (String) Legacy.getObjectField(serviceInfo, "packageName");
            }
        } catch (Throwable ignored) {
        }
        try {
            Object providerInfo = Legacy.getObjectField(entry, "providerInfo");
            if (providerInfo != null) {
                return (String) Legacy.getObjectField(providerInfo, "packageName");
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    private static void hookInstalledPackages(Class<?> appPm) {
        Legacy.safeHook(TAG, "getInstalledPackages", () -> {
            HookFramework.hookAllMethods(appPm, "getInstalledPackages",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (!(result instanceof List)) {
                                    return;
                                }
                                List<?> orig = (List<?>) result;
                                List<Object> kept = new ArrayList<>(orig.size());
                                for (Object entry : orig) {
                                    if (entry == null) {
                                        continue;
                                    }
                                    if (!denied(packageOf(entry))) {
                                        kept.add(entry);
                                    }
                                }
                                if (kept.size() != orig.size()) {
                                    chain.replaceResult(kept);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getInstalledPackages filter failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookInstalledApplications(Class<?> appPm) {
        Legacy.safeHook(TAG, "getInstalledApplications", () -> {
            HookFramework.hookAllMethods(appPm, "getInstalledApplications",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (!(result instanceof List)) {
                                    return;
                                }
                                List<?> orig = (List<?>) result;
                                List<Object> kept = new ArrayList<>(orig.size());
                                for (Object entry : orig) {
                                    if (entry == null) {
                                        continue;
                                    }
                                    if (!denied(packageOf(entry))) {
                                        kept.add(entry);
                                    }
                                }
                                if (kept.size() != orig.size()) {
                                    chain.replaceResult(kept);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getInstalledApplications filter failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookQueryIntentActivities(Class<?> appPm) {
        Legacy.safeHook(TAG, "queryIntentActivities", () -> {
            HookFramework.hookAllMethods(appPm, "queryIntentActivities",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (!(result instanceof List)) {
                                    return;
                                }
                                List<?> orig = (List<?>) result;
                                List<Object> kept = new ArrayList<>(orig.size());
                                for (Object entry : orig) {
                                    if (entry == null) {
                                        continue;
                                    }
                                    if (!denied(packageOf(entry))) {
                                        kept.add(entry);
                                    }
                                }
                                if (kept.size() != orig.size()) {
                                    chain.replaceResult(kept);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": queryIntentActivities filter failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookGetPackageInfo(Class<?> appPm) {
        Legacy.safeHook(TAG, "getPackageInfo", () -> {
            HookFramework.hookAllMethods(appPm, "getPackageInfo",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) throws Throwable {
                            try {
                                String pkg = chain.arg(0, null);
                                if (denied(pkg)) {
                                    throw new PackageManager.NameNotFoundException(pkg);
                                }
                            } catch (PackageManager.NameNotFoundException e) {
                                throw e;
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getPackageInfo deny check failed: " + t);
                            }
                        }
                    });
        });
    }
}
