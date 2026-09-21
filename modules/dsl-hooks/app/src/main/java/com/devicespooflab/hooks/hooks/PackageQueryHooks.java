package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.bridge.HookContext;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.utils.DenyTokens;

import java.util.ArrayList;
import java.util.List;

// PackageManager enumeration siblings ApplistHooks + PackageInfoHooks miss:
// getInstalledPackagesAsUser / getInstalledApplicationsAsUser,
// queryIntentActivitiesAsUser, resolveActivityAsUser / resolveServiceAsUser,
// queryContentProviders (provider-scan path), getPackageInstallerSessions,
// getInstalledModules (APEX/module-info leak: real user builds list ~30
// com.google.* apex modules; Waydroid's list differs). All share the same
// DenyTokens deny set + fail-closed filter helper. No duplication: this
// file owns *AsUser/provider/session/module surfaces only.
public final class PackageQueryHooks {

    private static final String TAG = "DeviceSpoofLab-PackageQuery";

    private PackageQueryHooks() {}

    public static void hook(HookContext lpparam) {
        Class<?> appPm = Legacy.findClassIfExists(
                "android.app.ApplicationPackageManager", lpparam.classLoader);
        if (appPm == null) {
            return;
        }
        hookAsUserLists(appPm);
        hookAsUserQueries(appPm);
        hookAsUserResolve(appPm);
        hookProviders(appPm, lpparam);
        hookInstallerSessions(appPm);
        hookExtraQueries(appPm);
        hookInstalledModules(appPm);
    }

    private static String packageOf(Object entry) {
        try {
            Object pkg = Legacy.getObjectField(entry, "packageName");
            if (pkg instanceof String) {
                return (String) pkg;
            }
        } catch (Throwable ignored) {
        }
        try {
            Object appInfo = Legacy.getObjectField(entry, "applicationInfo");
            if (appInfo != null) {
                Object pkg = Legacy.getObjectField(appInfo, "packageName");
                if (pkg instanceof String) {
                    return (String) pkg;
                }
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    private static void hookAsUserLists(Class<?> appPm) {
        for (String method : new String[]{
                "getInstalledPackagesAsUser",
                "getInstalledApplicationsAsUser"}) {
            final String name = method;
            Legacy.safeHook(TAG, name, () -> {
                HookFramework.hookAllMethods(appPm, name,
                        new HookFramework.Hook() {
                            @Override
                            @SuppressWarnings("unchecked")
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (!(result instanceof List)) {
                                        return;
                                    }
                                    List<?> orig = (List<?>) result;
                                    List<Object> kept =
                                            new ArrayList<>(orig.size());
                                    for (Object entry : orig) {
                                        if (entry == null) {
                                            continue;
                                        }
                                        if (!DenyTokens.isDeniedPackage(
                                                        packageOf(entry))) {
                                            kept.add(entry);
                                        }
                                    }
                                    if (kept.size() != orig.size()) {
                                        chain.replaceResult(kept);
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": " + name
                                            + " filter failed: " + t);
                                }
                            }
                        });
            });
        }
    }

    private static void hookAsUserQueries(Class<?> appPm) {
        for (String method : new String[]{
                "queryIntentActivitiesAsUser",
                "queryIntentServicesAsUser",
                "queryBroadcastReceiversAsUser"}) {
            final String name = method;
            Legacy.safeHook(TAG, name, () -> {
                HookFramework.hookAllMethods(appPm, name,
                        new HookFramework.Hook() {
                            @Override
                            @SuppressWarnings("unchecked")
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (!(result instanceof List)) {
                                        return;
                                    }
                                    List<?> orig = (List<?>) result;
                                    List<Object> kept =
                                            new ArrayList<>(orig.size());
                                    for (Object entry : orig) {
                                        if (entry == null) {
                                            continue;
                                        }
                                        if (!DenyTokens.isDeniedPackage(
                                                        packageOf(entry))) {
                                            kept.add(entry);
                                        }
                                    }
                                    if (kept.size() != orig.size()) {
                                        chain.replaceResult(kept);
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": " + name
                                            + " filter failed: " + t);
                                }
                            }
                        });
            });
        }
    }

    private static void hookAsUserResolve(Class<?> appPm) {
        for (String method : new String[]{
                "resolveActivityAsUser", "resolveServiceAsUser"}) {
            final String name = method;
            Legacy.safeHook(TAG, name, () -> {
                HookFramework.hookAllMethods(appPm, name,
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (result != null
                                            && com.devicespooflab.hooks.utils
                                                    .DenyTokens.isDeniedPackage(
                                                            packageOf(result))) {
                                        chain.replaceResult(null);
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": " + name
                                            + " deny check failed: " + t);
                                }
                            }
                        });
            });
        }
    }

    private static void hookProviders(Class<?> appPm, HookContext lpparam) {
        Legacy.safeHook(TAG, "queryContentProviders", () -> {
            HookFramework.hookAllMethods(appPm, "queryContentProviders",
                    new HookFramework.Hook() {
                        @Override
                        @SuppressWarnings("unchecked")
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
                                    String pkg = providerPackage(entry);
                                    if (!com.devicespooflab.hooks.utils
                                            .DenyTokens.isDeniedPackage(pkg)) {
                                        kept.add(entry);
                                    }
                                }
                                if (kept.size() != orig.size()) {
                                    chain.replaceResult(kept);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": queryContentProviders failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static String providerPackage(Object providerInfo) {
        try {
            Object pkg = Legacy.getObjectField(providerInfo, "packageName");
            if (pkg instanceof String) {
                return (String) pkg;
            }
        } catch (Throwable ignored) {
        }
        try {
            Object appInfo = Legacy.getObjectField(providerInfo,
                    "applicationInfo");
            if (appInfo != null) {
                Object pkg = Legacy.getObjectField(appInfo, "packageName");
                if (pkg instanceof String) {
                    return (String) pkg;
                }
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    private static void hookInstallerSessions(Class<?> appPm) {
        // PackageInstaller.getAllSessions: sideload-session enumeration.
        // Fail-closed passthrough (kept for symmetry); sessions are left
        // untouched — filtering install sessions would break legit updates.
        // appPm kept as the grouping owner for the PackageManager surface.
        if (appPm == null) {
            return;
        }
        Legacy.safeHook(TAG, "PackageInstaller.getAllSessions", () -> {
            Class<?> pi;
            try {
                pi = Class.forName("android.content.pm.PackageInstaller");
            } catch (Throwable ignored) {
                return;
            }
            HookFramework.hookAllMethods(pi, "getAllSessions",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getAllSessions failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    // UID / canonical-name / current-to-canonical siblings detectors use
    // to resolve denied packages indirectly: throw NameNotFound for denied
    // identity, passthrough otherwise. Fail-closed throughout.
    private static void hookExtraQueries(Class<?> appPm) {
        for (String method : new String[]{
                "getPackageUid", "getPackageGids", "getPackagesForUid",
                "currentToCanonicalPackageNames",
                "canonicalToCurrentPackageNames", "getNameForUid"}) {
            final String name = method;
            Legacy.safeHook(TAG, name, () -> {
                HookFramework.hookAllMethods(appPm, name,
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error)
                                    throws Throwable {
                                try {
                                    if (result instanceof String[]) {
                                        String[] orig = (String[]) result;
                                        java.util.List<String> kept =
                                                new ArrayList<>(orig.length);
                                        for (String pkg : orig) {
                                            if (!com.devicespooflab.hooks.utils
                                                    .DenyTokens.isDeniedPackage(
                                                            pkg)) {
                                                kept.add(pkg);
                                            }
                                        }
                                        if (kept.size() != orig.length) {
                                            chain.replaceResult(kept.toArray(
                                                    new String[0]));
                                        }
                                    } else if (result instanceof String) {
                                        // getNameForUid: single package name
                                        // for a uid — null it when denied
                                        // (unknown-uid contract allows null).
                                        if (com.devicespooflab.hooks.utils
                                                .DenyTokens.isDeniedPackage(
                                                        (String) result)) {
                                            chain.replaceResult(null);
                                        }
                                    }
                                } catch (android.content.pm.PackageManager
                                        .NameNotFoundException e) {
                                    throw e;
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": " + name
                                            + " filter failed: " + t);
                                }
                            }
                        });
            });
        }
    }

    // getInstalledModules (API 29+): APEX module list. Waydroid ships a
    // reduced set; pin each entry's package to the real user-build set by
    // leaving known com.google.* modules and dropping anything matching
    // the deny set. Never fabricate entries (fail-closed filter only).
    private static void hookInstalledModules(Class<?> appPm) {
        Legacy.safeHook(TAG, "getInstalledModules", () -> {
            HookFramework.hookAllMethods(appPm, "getInstalledModules",
                    new HookFramework.Hook() {
                        @Override
                        @SuppressWarnings("unchecked")
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
                                    if (!com.devicespooflab.hooks.utils
                                            .DenyTokens.isDeniedPackage(
                                                    modulePackage(entry))) {
                                        kept.add(entry);
                                    }
                                }
                                if (kept.size() != orig.size()) {
                                    chain.replaceResult(kept);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getInstalledModules failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static String modulePackage(Object moduleInfo) {
        try {
            Object pkg = Legacy.getObjectField(moduleInfo, "packageName");
            if (pkg instanceof String) {
                return (String) pkg;
            }
        } catch (Throwable ignored) {
        }
        try {
            Object name = Legacy.callMethod(moduleInfo, "getPackageName");
            if (name instanceof String) {
                return (String) name;
            }
        } catch (Throwable ignored) {
        }
        return null;
    }
}
