package com.devicespooflab.hooks;

import android.content.SharedPreferences;
import android.util.Log;

import com.devicespooflab.hooks.utils.ConfigManager;
import com.devicespooflab.hooks.utils.XposedServiceBridge;

import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.util.concurrent.CopyOnWriteArraySet;

import io.github.libxposed.api.XposedModule;
import io.github.libxposed.api.XposedModuleInterface;

// New-API entry point for Vector LSPosed (JingMatrix). Vector only delivers
// the IXposedService binder — and therefore writable RemotePreferences — to
// modules declared via META-INF/xposed/java_init.list. This class receives
// that binder; MainHook still does all of the hook installation.
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
        // Read-only RemotePreferences is live now; writes wait for the
        // IXposedService binder from MainHook's Application.attach hook.
        XposedServiceBridge.markAvailableViaNewApi();
    }

    @Override
    public void onPackageLoaded(XposedModuleInterface.PackageLoadedParam param) {
        try {
            // Vector's runtime provides the legacy de.robv classes inside
            // target processes (see framework/vector.dex: XC_LoadPackage,
            // XposedBridge, XposedHelpers all present). Resolve through the
            // module classloader explicitly — the compileOnly api:82 + stub
            // jars are NOT packaged into the APK, so a direct static
            // reference would throw NoClassDefFoundError at class-load time
            // (verified 2026-09-19: onPackageLoaded delegation failed).
            // Reflection keeps this class loadable even if Vector ever drops
            // a legacy symbol; failures degrade to "module loaded, no hooks"
            // instead of a crash.
            ClassLoader loader = XposedModuleImpl.class.getClassLoader();
            Class<?> lpClass = Class.forName(
                    "de.robv.android.xposed.callbacks.XC_LoadPackage$LoadPackageParam",
                    false, loader);
            Object lp = allocateLoadPackageParam(lpClass);
            setField(lpClass, lp, "packageName", param.getPackageName());
            setField(lpClass, lp, "processName", getProcessName(param));
            setField(lpClass, lp, "classLoader", param.getDefaultClassLoader());
            setField(lpClass, lp, "appInfo", param.getApplicationInfo());
            setField(lpClass, lp, "isFirstApplication", param.isFirstPackage());
            invokeMainHookHandleLoadPackage(lp);
        } catch (Throwable t) {
            Log.e(TAG, "onPackageLoaded delegation failed: "
                    + t.getClass().getSimpleName() + ": " + t.getMessage(), t);
        }
    }

    private static String getProcessName(XposedModuleInterface.PackageLoadedParam param) {
        try {
            Method method = param.getClass().getMethod("getProcessName");
            method.setAccessible(true);
            Object value = method.invoke(param);
            if (value instanceof String && !((String) value).isEmpty()) {
                return (String) value;
            }
        } catch (Throwable ignored) {
        }
        return param.getPackageName();
    }

    // Runtime LoadPackageParam takes a CopyOnWriteArraySet; the api-82 stub
    // exposes only a package-private no-arg ctor. Reflect to bypass the mismatch.
    // NOTE: callers must pass the Class resolved from THIS module's loader
    // (Vector injects legacy de.robv classes into target processes).
    private static Object allocateLoadPackageParam(Class<?> lpClass) throws Exception {
        for (Constructor<?> c : lpClass.getDeclaredConstructors()) {
            c.setAccessible(true);
            Class<?>[] params = c.getParameterTypes();
            if (params.length == 0) {
                return c.newInstance();
            }
            if (params.length == 1 && CopyOnWriteArraySet.class.isAssignableFrom(params[0])) {
                return c.newInstance(new CopyOnWriteArraySet<>());
            }
        }
        Class<?> unsafeClass = Class.forName("sun.misc.Unsafe");
        java.lang.reflect.Field field = unsafeClass.getDeclaredField("theUnsafe");
        field.setAccessible(true);
        Object unsafe = field.get(null);
        Method allocateInstance = unsafeClass.getMethod("allocateInstance", Class.class);
        Object value = allocateInstance.invoke(unsafe, lpClass);
        Log.w(TAG, "LoadPackageParam created via Unsafe fallback");
        return value;
    }

    private static void setField(Class<?> cls, Object obj, String name, Object value)
            throws Exception {
        java.lang.reflect.Field f = cls.getField(name);
        f.set(obj, value);
    }

    private static void invokeMainHookHandleLoadPackage(Object lp) throws Exception {
        Method m = MainHook.class.getMethod(
                "handleLoadPackage",
                Class.forName(
                        "de.robv.android.xposed.callbacks.XC_LoadPackage$LoadPackageParam",
                        false, XposedModuleImpl.class.getClassLoader()));
        m.invoke(new MainHook(), lp);
    }
}
