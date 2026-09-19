package com.devicespooflab.hooks.bridge;

import android.util.Log;

import java.lang.reflect.Constructor;
import java.lang.reflect.Executable;
import java.lang.reflect.Method;
import java.util.concurrent.atomic.AtomicReference;

import io.github.libxposed.api.XposedInterface;

// Host holder for the modern hook framework. The XposedModuleImpl instance
// (a live XposedInterface after attachFramework) is registered in
// onModuleLoaded; hook helpers call HookFramework.hook(...) instead of the
// legacy XposedHelpers.findAndHookMethod. Zero de.robv references.
public final class HookFramework {

    private static final String TAG = "DeviceSpoofLab-HookFW";

    private static final AtomicReference<XposedInterface> sHost =
            new AtomicReference<>();

    private HookFramework() {}

    public static void setHost(XposedInterface host) {
        sHost.set(host);
    }

    public static XposedInterface host() {
        return sHost.get();
    }

    public static boolean available() {
        return sHost.get() != null;
    }

    // Hook an instance/static method by exact parameter types.
    // Returns a handle when the hook is installed, null otherwise.
    public static HookHandle hookMethod(Class<?> cls, String name,
            Class<?>[] params, Hook hook) {
        XposedInterface host = sHost.get();
        if (host == null || cls == null) return null;
        try {
            Method m = cls.getDeclaredMethod(name, params);
            m.setAccessible(true);
            return hook(m, hook);
        } catch (Throwable t) {
            Log.w(TAG, "hookMethod " + simple(cls) + "." + name
                    + " failed: " + t.getClass().getSimpleName());
            return null;
        }
    }

    // Hook ALL overloads of a method name (used where signatures vary by
    // SDK, e.g. registerReceiver 2/3/4-arg forms).
    public static int hookAllMethods(Class<?> cls, String name, Hook hook) {
        XposedInterface host = sHost.get();
        if (host == null || cls == null) return 0;
        int ok = 0;
        for (Method m : cls.getDeclaredMethods()) {
            if (!m.getName().equals(name)) continue;
            try {
                m.setAccessible(true);
                if (hook(m, hook) != null) ok++;
            } catch (Throwable ignored) {
            }
        }
        return ok;
    }

    // Hook all constructors (InputDevice-style Constructor hooks).
    public static int hookAllConstructors(Class<?> cls, Hook hook) {
        XposedInterface host = sHost.get();
        if (host == null || cls == null) return 0;
        int ok = 0;
        for (Constructor<?> c : cls.getDeclaredConstructors()) {
            try {
                c.setAccessible(true);
                if (hook(c, hook) != null) ok++;
            } catch (Throwable ignored) {
            }
        }
        return ok;
    }

    // Unified entry: hooks any Executable (Method or Constructor),
    // honoring BeforeHook.before() before the original runs.
    // Uses the HookBuilder form (hook(target).intercept(hooker)) so it
    // compiles against both API 101 and 102 (102 adds setId/replaceHook
    // but keeps intercept()).
    public static HookHandle hook(Executable target, final Hook hook) {
        XposedInterface host = sHost.get();
        if (host == null || target == null || hook == null) return null;
        try {
            target.setAccessible(true);
            XposedInterface.HookHandle h = host.hook(target).intercept(
                    new XposedInterface.Hooker() {
                @Override
                public Object intercept(XposedInterface.Chain chain)
                        throws Throwable {
                    HookChain c = new HookChain(chain);
                    // Before-phase (only for BeforeHook): may mutate args
                    // or replaceResult() to skip the original entirely.
                    if (hook instanceof BeforeHook) {
                        ((BeforeHook) hook).before(c);
                        if (c.hasReplacement()) {
                            return c.replacement();
                        }
                    }
                    Object result;
                    try {
                        result = chain.proceed();
                    } catch (Throwable t) {
                        hook.after(c, null, t);
                        throw t;
                    }
                    hook.after(c, result, null);
                    return c.hasReplacement()
                            ? c.replacement() : result;
                }
            });
            return new HookHandle(h);
        } catch (Throwable t) {
            Log.w(TAG, "hook " + target + " failed: "
                    + t.getClass().getSimpleName() + ": " + t.getMessage());
            return null;
        }
    }

    private static String simple(Class<?> cls) {
        try {
            return cls.getSimpleName();
        } catch (Throwable ignored) {
            return "?";
        }
    }

    // Single after-hook interface: inspect/mutate args, replace result.
    // Throwing from after() propagates (use sparingly; prefer replacement).
    public interface Hook {
        void after(HookChain chain, Object result, Throwable error)
                throws Throwable;
    }

    // Before-hook interface: mutate args before the original runs.
    // before() runs FIRST; if it calls chain.replaceResult(), the original
    // is SKIPPED and the replacement returned (Xposed beforeHookedMethod +
    // setResult semantics). Otherwise the original runs and after() fires.
    public interface BeforeHook extends Hook {
        void before(HookChain chain) throws Throwable;

        @Override
        default void after(HookChain chain, Object result, Throwable error) {
        }
    }

    // Unhook handle wrapper (libxposed 101 stub lacks getId/replaceHook).
    public static final class HookHandle {
        private final XposedInterface.HookHandle inner;

        HookHandle(XposedInterface.HookHandle inner) {
            this.inner = inner;
        }

        public void unhook() {
            try {
                if (inner != null) inner.unhook();
            } catch (Throwable ignored) {
            }
        }
    }

    // Chain adapter: thisObject + args (+ typed arg access), result replace.
    // Arg mutation: libxposed Chain.getArgs() returns a LIVE mutable list
    // (proceed() reads it back), so setArg writes straight through.
    // args() returns the LIVE list (not a copy): index writes through
    // setArg; arg(i, fallback) is the typed read.
    public static final class HookChain {
        private final XposedInterface.Chain chain;
        private boolean replaced;
        private Object replacement;

        HookChain(XposedInterface.Chain chain) {
            this.chain = chain;
        }

        public Object thisObject() {
            try {
                return chain.getThisObject();
            } catch (Throwable ignored) {
                return null;
            }
        }

        @SuppressWarnings("unchecked")
        public java.util.List<Object> args() {
            try {
                return chain.getArgs();
            } catch (Throwable ignored) {
                return java.util.Collections.emptyList();
            }
        }

        public int argCount() {
            try {
                return chain.getArgs().size();
            } catch (Throwable ignored) {
                return 0;
            }
        }

        // Mutable arg write (beforeHookedMethod param.args[i] = v).
        public void setArg(int i, Object v) {
            try {
                java.util.List<Object> list = chain.getArgs();
                if (i >= 0 && i < list.size()) list.set(i, v);
            } catch (Throwable ignored) {
            }
        }

        @SuppressWarnings("unchecked")
        public <T> T arg(int i, T fallback) {
            try {
                Object v = chain.getArg(i);
                return v != null ? (T) v : fallback;
            } catch (Throwable ignored) {
                return fallback;
            }
        }

        public void replaceResult(Object v) {
            replaced = true;
            replacement = v;
        }

        boolean hasReplacement() {
            return replaced;
        }

        Object replacement() {
            return replacement;
        }
    }
}
