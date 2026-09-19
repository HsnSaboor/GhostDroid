package com.devicespooflab.hooks;

import android.content.SharedPreferences;
import android.util.Log;

import io.github.libxposed.api.XposedModule;
import io.github.libxposed.api.XposedModuleInterface;

// New-API entry point for Vector LSPosed (JingMatrix). Vector only delivers
// the IXposedService binder — and therefore writable RemotePreferences — to
// modules declared via META-INF/xposed/java_init.list. This class receives
// that binder and installs ALL hooks directly through the modern
// XposedInterface hook API (io.github.libxposed), with ZERO references to
// the legacy de.robv.android.xposed.* classes.
//
// History (do NOT regress): every de.robv class (XC_LoadPackage,
// XposedBridge, XposedHelpers, XC_MethodHook) is compileOnly and ABSENT
// from every reachable classloader inside Vector target processes
// (verified 2026-09-19: ClassNotFoundException on all loaders, including
// stack-walked Vector caller frames). Any static import of de.robv.* in
// ANY class loaded in a target process throws NoClassDefFoundError at
// class-load time and kills the whole module. MainHook + all *Hooks now
// take a plain HookContext (no de.robv types) and hook via
// com.devicespooflab.hooks.bridge.HookFramework.
public final class XposedModuleImpl extends XposedModule {

    private static final String TAG = "DeviceSpoofLab";

    private static volatile XposedModuleImpl sInstance;

    // Vector's libxposed-api runtime instantiates the module via the NO-ARG
    // constructor (getDeclaredConstructor().newInstance()) and injects the
    // framework reference afterwards through XposedInterfaceWrapper
    // .attachFramework(), so inherited getRemotePreferences() works without a
    // constructor argument. A 2-arg ctor here makes Vector throw
    // NoSuchMethodException: <init> [] and the module never loads in targets.
    public XposedModuleImpl() {
        sInstance = this;
    }

    public static XposedModuleImpl get() {
        return sInstance;
    }

    public static SharedPreferences fetchRemotePreferences(String group) {
        XposedModuleImpl instance = sInstance;
        if (instance == null) return null;
        try {
            return instance.getRemotePreferences(group);
        } catch (Throwable t) {
            Log.w(TAG, "fetchRemotePreferences(" + group + ") failed: "
                    + t.getClass().getSimpleName() + ": " + t.getMessage());
            return null;
        }
    }

    @Override
    public void onModuleLoaded(XposedModuleInterface.ModuleLoadedParam param) {
        Log.i(TAG, "XposedModuleImpl onModuleLoaded process=" + param.getProcessName()
                + " systemServer=" + param.isSystemServer());
        // Framework handle for modern-API hooking. attachFramework() has run
        // by now, so `this` is a live XposedInterface.
        com.devicespooflab.hooks.bridge.HookFramework.setHost(this);
        com.devicespooflab.hooks.utils.XposedServiceBridge
                .markAvailableViaNewApi();
    }

    @Override
    public void onPackageLoaded(XposedModuleInterface.PackageLoadedParam param) {
        try {
            String packageName = param.getPackageName();
            ClassLoader appLoader = param.getDefaultClassLoader();
            Object appInfo = param.getApplicationInfo();
            boolean first = param.isFirstPackage();
            // PackageLoadedParam has no process name; the package name is
            // the best available identity (ModuleLoadedParam carries it).
            String process = packageName;
            com.devicespooflab.hooks.bridge.HookContext ctx =
                    new com.devicespooflab.hooks.bridge.HookContext(
                            packageName, process, appLoader, appInfo, first);
            new MainHook().handleLoadPackage(ctx);
        } catch (Throwable t) {
            Log.e(TAG, "onPackageLoaded delegation failed: "
                    + t.getClass().getSimpleName() + ": " + t.getMessage(), t);
        }
    }
}
