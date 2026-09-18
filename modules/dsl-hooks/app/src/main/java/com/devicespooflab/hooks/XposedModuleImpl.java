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
            // Vector injects legacy de.robv classes via its OWN
            // VectorModuleClassLoader — NOT the module APK classloader
            // (verified 2026-09-19: Class.forName through module loader
            // throws ClassNotFoundException even though vector.dex ships
            // XC_LoadPackage). So resolve through the call stack: walk up
            // from Vector's callback frame (x2.a/s.d/h2.b in the stack)
            // to the ClassLoader that actually hosts de.robv.
            Class<?> lpClass = findLegacyLoadPackageParamClass();
            Object lp = allocateLoadPackageParam(lpClass);
            setField(lpClass, lp, "packageName", param.getPackageName());
            setField(lpClass, lp, "processName", getProcessName(param));
            setField(lpClass, lp, "classLoader", param.getDefaultClassLoader());
            setField(lpClass, lp, "appInfo", param.getApplicationInfo());
            setField(lpClass, lp, "isFirstApplication", param.isFirstPackage());
            invokeMainHookHandleLoadPackage(lpClass, lp);
        } catch (Throwable t) {
            Log.e(TAG, "onPackageLoaded delegation failed: "
                    + t.getClass().getSimpleName() + ": " + t.getMessage(), t);
        }
    }

    // Walk the current thread's stack: Vector calls onPackageLoaded from
    // its own frames (x2.a/s.d/h2.b...), whose classes live in the loader
    // that also hosts de.robv legacy classes. Try each frame's loader in
    // order; fall back to module + app loaders.
    private static Class<?> findLegacyLoadPackageParamClass() throws Exception {
        final String target =
                "de.robv.android.xposed.callbacks.XC_LoadPackage$LoadPackageParam";
        for (StackTraceElement el : Thread.currentThread().getStackTrace()) {
            try {
                Class<?> frame = Class.forName(el.getClassName(), false,
                        XposedModuleImpl.class.getClassLoader());
                if (frame == null) continue;
                ClassLoader l = frame.getClassLoader();
                if (l == null) continue;
                try {
                    return Class.forName(target, false, l);
                } catch (ClassNotFoundException ignored) {
                }
            } catch (Throwable ignored) {
            }
        }
        // Fallbacks: module loader, then thread context loader.
        try {
            return Class.forName(target, false,
                    XposedModuleImpl.class.getClassLoader());
        } catch (ClassNotFoundException ignored) {
        }
        return Class.forName(target, false,
                Thread.currentThread().getContextClassLoader());
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

    private static void invokeMainHookHandleLoadPackage(Class<?> lpClass, Object lp)
            throws Exception {
        Method m = MainHook.class.getMethod("handleLoadPackage", lpClass);
        m.invoke(new MainHook(), lp);
    }
}
