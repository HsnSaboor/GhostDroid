package com.devicespooflab.hooks.bridge;

import android.util.Log;

// Drop-in runtime for the legacy XposedHelpers/XposedBridge call shapes
// used across all *Hooks, backed by the modern XposedInterface API.
// Zero de.robv imports: every method resolves java.lang.reflect members
// directly, so this class loads fine in Vector target processes.
//
// Supported shapes (only what the codebase actually calls):
//   findAndHookMethod(Class|String, ClassLoader?, name, params..., Hook)
//   findClassIfExists(name, loader)
//   getObjectField / setObjectField / setIntField / getIntField ...
//   callMethod / callStaticMethod
//   XposedBridge.log -> Log.i
public final class Legacy {

    private static final String TAG = "DeviceSpoofLab-Legacy";

    private Legacy() {}

    public static void log(String msg) {
        try {
            Log.i("DeviceSpoofLab", msg);
        } catch (Throwable ignored) {
        }
    }

    public static Class<?> findClassIfExists(String name, ClassLoader loader) {
        if (name == null) return null;
        // App loader first: the target app's classes live here. Never fall
        // back to the system loader — it can resolve a same-named framework
        // stub instead of the app's real class.
        if (loader != null) {
            try {
                return Class.forName(name, false, loader);
            } catch (Throwable ignored) {
            }
        }
        try {
            ClassLoader self = Legacy.class.getClassLoader();
            if (self != null && self != loader) {
                return Class.forName(name, false, self);
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    // findAndHookMethod(Class, name, params..., Hook) -> handle or null.
    public static HookFramework.HookHandle findAndHookMethod(Class<?> cls,
            String name, Object... paramsAndHook) {
        return hookParsed(cls, name, paramsAndHook);
    }

    // findAndHookMethod(String className, ClassLoader, name, params..., Hook)
    public static HookFramework.HookHandle findAndHookMethod(String className,
            ClassLoader loader, String name, Object... paramsAndHook) {
        Class<?> cls = findClassIfExists(className, loader);
        if (cls == null) return null;
        return hookParsed(cls, name, paramsAndHook);
    }

    // findAndHookConstructor(Class, params..., Hook)
    public static HookFramework.HookHandle findAndHookConstructor(Class<?> cls,
            Object... paramsAndHook) {
        if (cls == null || paramsAndHook == null || paramsAndHook.length == 0) {
            return null;
        }
        Object last = paramsAndHook[paramsAndHook.length - 1];
        if (!(last instanceof HookFramework.Hook)) return null;
        HookFramework.Hook hook = (HookFramework.Hook) last;
        Class<?>[] params = classesOf(paramsAndHook, paramsAndHook.length - 1);
        if (params == null) return null;
        try {
            java.lang.reflect.Constructor<?> c =
                    cls.getDeclaredConstructor(params);
            return HookFramework.hook(c, hook);
        } catch (Throwable t) {
            Log.w(TAG, "findAndHookConstructor " + cls.getName()
                    + " failed: " + t.getClass().getSimpleName());
            return null;
        }
    }

    private static HookFramework.HookHandle hookParsed(Class<?> cls,
            String name, Object[] paramsAndHook) {
        if (cls == null || paramsAndHook == null || paramsAndHook.length == 0) {
            return null;
        }
        Object last = paramsAndHook[paramsAndHook.length - 1];
        if (!(last instanceof HookFramework.Hook)) return null;
        HookFramework.Hook hook = (HookFramework.Hook) last;
        Class<?>[] params = classesOf(paramsAndHook, paramsAndHook.length - 1);
        if (params == null) return null;
        return HookFramework.hookMethod(cls, name, params, hook);
    }

    private static Class<?>[] classesOf(Object[] in, int n) {
        Class<?>[] out = new Class<?>[n];
        for (int i = 0; i < n; i++) {
            Object o = in[i];
            if (o instanceof Class) {
                out[i] = (Class<?>) o;
            } else if (o instanceof String) {
                // Primitive / array names as used by XposedHelpers call sites.
                Class<?> c = primitive((String) o);
                if (c == null) return null;
                out[i] = c;
            } else {
                return null;
            }
        }
        return out;
    }

    private static Class<?> primitive(String n) {
        switch (n) {
            case "boolean": return boolean.class;
            case "byte": return byte.class;
            case "char": return char.class;
            case "short": return short.class;
            case "int": return int.class;
            case "long": return long.class;
            case "float": return float.class;
            case "double": return double.class;
            case "void": return void.class;
            default: return null;
        }
    }

    // Central install guard: runs the hook install, logs failures with tag.
    // Replaces per-file try/catch + logFail clones in every *Hooks class.
    public interface HookInstall {
        void install() throws Throwable;
    }

    public static void safeHook(String tag, String what, HookInstall install) {
        try {
            install.install();
        } catch (Throwable t) {
            android.util.Log.w(tag, "failed to hook " + what + ": " + t);
        }
    }

    // ---- field helpers (mirrors XposedHelpers subset in use) ----

    public static Object getObjectField(Object obj, String name) {
        try {
            java.lang.reflect.Field f = field(obj.getClass(), name);
            f.setAccessible(true);
            return f.get(obj);
        } catch (Throwable t) {
            Log.w(TAG, "getObjectField " + name + ": " + t.getMessage());
            return null;
        }
    }

    public static void setObjectField(Object obj, String name, Object v) {
        try {
            java.lang.reflect.Field f = field(obj.getClass(), name);
            f.setAccessible(true);
            f.set(obj, v);
        } catch (Throwable t) {
            Log.w(TAG, "setObjectField " + name + ": " + t.getMessage());
        }
    }

    public static int getIntField(Object obj, String name) {
        try {
            java.lang.reflect.Field f = field(obj.getClass(), name);
            f.setAccessible(true);
            return f.getInt(obj);
        } catch (Throwable t) {
            return 0;
        }
    }

    public static void setIntField(Object obj, String name, int v) {
        try {
            java.lang.reflect.Field f = field(obj.getClass(), name);
            f.setAccessible(true);
            f.setInt(obj, v);
        } catch (Throwable ignored) {
        }
    }

    public static long getLongField(Object obj, String name) {
        try {
            java.lang.reflect.Field f = field(obj.getClass(), name);
            f.setAccessible(true);
            return f.getLong(obj);
        } catch (Throwable t) {
            return 0L;
        }
    }

    public static void setLongField(Object obj, String name, long v) {
        try {
            java.lang.reflect.Field f = field(obj.getClass(), name);
            f.setAccessible(true);
            f.setLong(obj, v);
        } catch (Throwable ignored) {
        }
    }

    public static boolean getBooleanField(Object obj, String name) {
        try {
            java.lang.reflect.Field f = field(obj.getClass(), name);
            f.setAccessible(true);
            return f.getBoolean(obj);
        } catch (Throwable t) {
            return false;
        }
    }

    public static void setBooleanField(Object obj, String name, boolean v) {
        try {
            java.lang.reflect.Field f = field(obj.getClass(), name);
            f.setAccessible(true);
            f.setBoolean(obj, v);
        } catch (Throwable ignored) {
        }
    }

    public static void setStaticObjectField(Class<?> cls, String name, Object v) {
        try {
            java.lang.reflect.Field f = field(cls, name);
            f.setAccessible(true);
            f.set(null, v);
        } catch (Throwable t) {
            Log.w(TAG, "setStaticObjectField " + name + ": " + t.getMessage());
        }
    }

    public static void setStaticIntField(Class<?> cls, String name, int v) {
        try {
            java.lang.reflect.Field f = field(cls, name);
            f.setAccessible(true);
            f.setInt(null, v);
        } catch (Throwable ignored) {
        }
    }

    public static void setStaticLongField(Class<?> cls, String name, long v) {
        try {
            java.lang.reflect.Field f = field(cls, name);
            f.setAccessible(true);
            f.setLong(null, v);
        } catch (Throwable ignored) {
        }
    }

    public static void setStaticBooleanField(Class<?> cls, String name, boolean v) {
        try {
            java.lang.reflect.Field f = field(cls, name);
            f.setAccessible(true);
            f.setBoolean(null, v);
        } catch (Throwable ignored) {
        }
    }

    private static java.lang.reflect.Field field(Class<?> cls, String name)
            throws Exception {
        Class<?> c = cls;
        while (c != null) {
            try {
                return c.getDeclaredField(name);
            } catch (NoSuchFieldException e) {
                c = c.getSuperclass();
            }
        }
        return cls.getField(name);
    }

    // ---- method-call helpers ----

    public static Object callMethod(Object obj, String name, Object... args) {
        try {
            Class<?>[] types = typesOf(args);
            java.lang.reflect.Method m = method(obj.getClass(), name, types);
            m.setAccessible(true);
            return m.invoke(obj, args);
        } catch (Throwable t) {
            Log.w(TAG, "callMethod " + name + ": " + t.getMessage());
            return null;
        }
    }

    public static Object callStaticMethod(Class<?> cls, String name,
            Object... args) {
        try {
            Class<?>[] types = typesOf(args);
            java.lang.reflect.Method m = method(cls, name, types);
            m.setAccessible(true);
            return m.invoke(null, args);
        } catch (Throwable t) {
            Log.w(TAG, "callStaticMethod " + name + ": " + t.getMessage());
            return null;
        }
    }

    private static Class<?>[] typesOf(Object[] args) {
        if (args == null) return new Class<?>[0];
        Class<?>[] out = new Class<?>[args.length];
        for (int i = 0; i < args.length; i++) {
            Object a = args[i];
            out[i] = a == null ? Object.class : a.getClass();
        }
        return out;
    }

    private static java.lang.reflect.Method method(Class<?> cls, String name,
            Class<?>[] types) throws Exception {
        Class<?> c = cls;
        while (c != null) {
            for (java.lang.reflect.Method m : c.getDeclaredMethods()) {
                if (!m.getName().equals(name)) continue;
                if (compatible(m.getParameterTypes(), types)) return m;
            }
            c = c.getSuperclass();
        }
        return cls.getMethod(name, types);
    }

    private static boolean compatible(Class<?>[] formal, Class<?>[] actual) {
        if (formal.length != actual.length) return false;
        for (int i = 0; i < formal.length; i++) {
            if (actual[i] == Object.class) continue;
            if (formal[i].isAssignableFrom(actual[i])) continue;
            if (formal[i].isPrimitive()
                    && boxed(formal[i]).isAssignableFrom(actual[i])) continue;
            return false;
        }
        return true;
    }

    private static Class<?> boxed(Class<?> p) {
        if (p == boolean.class) return Boolean.class;
        if (p == byte.class) return Byte.class;
        if (p == char.class) return Character.class;
        if (p == short.class) return Short.class;
        if (p == int.class) return Integer.class;
        if (p == long.class) return Long.class;
        if (p == float.class) return Float.class;
        if (p == double.class) return Double.class;
        return p;
    }

    // Direct hook of an already-resolved Method or Constructor
    // (AdvertisingId/AppSetId GMS-side discovery resolves Executables).
    public static HookFramework.HookHandle hookMethod(
            java.lang.reflect.Executable target, HookFramework.Hook hook) {
        return HookFramework.hook(target, hook);
    }

    public static HookFramework.HookHandle hookConstructor(
            java.lang.reflect.Constructor<?> c, HookFramework.Hook hook) {
        return HookFramework.hook(c, hook);
    }
}
