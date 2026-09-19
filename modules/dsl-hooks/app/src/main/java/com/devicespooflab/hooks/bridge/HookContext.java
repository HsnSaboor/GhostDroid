package com.devicespooflab.hooks.bridge;

import android.content.pm.ApplicationInfo;

// Legacy-free load-package context. Replaces
// de.robv.android.xposed.callbacks.XC_LoadPackage.LoadPackageParam, which is
// NOT present in any classloader reachable from Vector target processes.
// Only the fields MainHook + *Hooks actually read: classLoader (42 uses),
// packageName (10), appInfo (2), processName (1).
public final class HookContext {

    public final String packageName;
    public final String processName;
    public final ClassLoader classLoader;
    public final ApplicationInfo appInfo;
    public final boolean isFirstApplication;

    public HookContext(String packageName, String processName,
            ClassLoader classLoader, Object appInfo, boolean first) {
        this.packageName = packageName;
        this.processName = processName;
        this.classLoader = classLoader;
        this.appInfo = appInfo instanceof ApplicationInfo
                ? (ApplicationInfo) appInfo : null;
        this.isFirstApplication = first;
    }
}
