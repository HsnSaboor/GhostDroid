package com.devicespooflab.hooks.hooks;

import android.app.ActivityManager;
import android.os.Debug;

import com.devicespooflab.hooks.utils.ConfigManager;

import java.util.Collections;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicBoolean;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class HardwareHooks {

    private static final String TAG = "DeviceSpoofLab-Hardware";
    private static final AtomicBoolean RUNTIME_CORES_HOOKED = new AtomicBoolean(false);
    private static final AtomicBoolean DEBUG_MEMORY_HOOKED = new AtomicBoolean(false);
    private static final Set<Class<?>> HOOKED_ACTIVITY_MANAGER_CLASSES =
            Collections.newSetFromMap(new ConcurrentHashMap<Class<?>, Boolean>());

    public static void hook(HookContext lpparam) {
        try {
            hookRuntimeCores();
            hookActivityManagerMemory(lpparam);
            hookRunningTasks(lpparam);
            hookDebugMemory();
            hookDebuggerState();
            hookTestHarness(lpparam);
            if (ConfigManager.isVerboseLoggingEnabled()) {
                Legacy.log(TAG + ": Successfully hooked hardware specs");
            }
        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook hardware: " + e.getMessage());
        }
    }

    // Activity.isInMultiWindowMode / UiModeManager car-desk tells: ACE DEX
    // reads these without native. A flagship phone is fullscreen, not in
    // multi-window, and uiMode is normal (1). Fail-closed.
    private static void hookRunningTasks(HookContext lpparam) {
        Class<?> activity = Legacy.findClassIfExists(
                "android.app.Activity", lpparam.classLoader);
        if (activity != null) {
            Legacy.safeHook(TAG, "Activity.isInMultiWindowMode", () -> {
                HookFramework.hookAllMethods(activity, "isInMultiWindowMode",
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    if (result instanceof Boolean
                                            && (Boolean) result) {
                                        chain.replaceResult(false);
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": isInMultiWindowMode failed: "
                                            + t);
                                }
                            }
                        });
            });
            Legacy.safeHook(TAG, "Activity.isInPictureInPictureMode", () -> {
                HookFramework.hookAllMethods(activity,
                        "isInPictureInPictureMode",
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    if (result instanceof Boolean
                                            && (Boolean) result) {
                                        chain.replaceResult(false);
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": isInPictureInPictureMode failed: "
                                            + t);
                                }
                            }
                        });
            });
        }
        Class<?> uiMode = Legacy.findClassIfExists(
                "android.app.UiModeManager", lpparam.classLoader);
        if (uiMode != null) {
            Legacy.safeHook(TAG, "UiModeManager.getCurrentModeType", () -> {
                HookFramework.hookAllMethods(uiMode, "getCurrentModeType",
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    if (result instanceof Integer
                                            && (Integer) result != 1) {
                                        chain.replaceResult(1);
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": getCurrentModeType failed: "
                                            + t);
                                }
                            }
                        });
            });
        }
    }

    // Build.IS_DEBUGGABLE-adjacent Java tell: ActivityManager.isRunningInTestHarness
    // and isRunningInUserTestHarness return true on engineering/test builds.
    // A user build is never in a test harness. Fail-closed.
    private static void hookTestHarness(HookContext lpparam) {
        Class<?> am = Legacy.findClassIfExists(
                "android.app.ActivityManager", lpparam.classLoader);
        if (am == null) {
            return;
        }
        for (String name : new String[]{
                "isRunningInTestHarness", "isRunningInUserTestHarness"}) {
            final String method = name;
            Legacy.safeHook(TAG, "ActivityManager." + method, () -> {
                HookFramework.hookAllMethods(am, method,
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    if (result instanceof Boolean
                                            && (Boolean) result) {
                                        chain.replaceResult(false);
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": " + method
                                            + " failed: " + t);
                                }
                            }
                        });
            });
        }
    }

    private static void hookRuntimeCores() {
        if (!RUNTIME_CORES_HOOKED.compareAndSet(false, true)) {
            return;
        }
        // availableProcessors: single no-arg overload; hookAllMethods
        // covers uniformly. Fail-closed: try/catch + Legacy.log.
        Legacy.safeHook(TAG, "Runtime.availableProcessors", () -> {
            HookFramework.hookAllMethods(Runtime.class, "availableProcessors",
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                        try {
                            if (error != null) {
                                return;
                            }
                            chain.replaceResult(ConfigManager.getCpuCoreCount());
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": availableProcessors failed: " + t);
                        }
                    }
                });
        });
    }

    private static void hookActivityManagerMemory(HookContext lpparam) {
        Class<?> activityManagerClass = Legacy.findClassIfExists(
            "android.app.ActivityManager", lpparam.classLoader);

        if (activityManagerClass == null) {
            return;
        }
        if (!HOOKED_ACTIVITY_MANAGER_CLASSES.add(activityManagerClass)) {
            return;
        }

        // getMemoryInfo(MemoryInfo) single overload; no-arg memory-class
        // getters likewise. Fail-closed: try/catch + Legacy.log inside
        // every after-hook, installs via safeHook.
        Legacy.safeHook(TAG, "ActivityManager.getMemoryInfo", () -> {
            Legacy.findAndHookMethod(activityManagerClass, "getMemoryInfo",
                ActivityManager.MemoryInfo.class,
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                        try {
                            if (error != null) {
                                return;
                            }
                            ActivityManager.MemoryInfo memInfo = (ActivityManager.MemoryInfo) chain.arg(0, null);
                            if (memInfo != null) {
                                long originalTotal = memInfo.totalMem;
                                long configuredTotal = Math.max(0L, ConfigManager.getMemoryTotalBytes());
                                long configuredAvailable = Math.max(0L, ConfigManager.getMemoryAvailableKb() * 1024L);
                                memInfo.totalMem = configuredTotal;
                                if (originalTotal > 0) {
                                    long originalAvailable = Math.max(0L,
                                            Math.min(originalTotal, memInfo.availMem));
                                    double availableRatio = (double) originalAvailable / originalTotal;
                                    memInfo.availMem = Math.min(configuredTotal,
                                            (long) (configuredTotal * availableRatio));
                                } else {
                                    memInfo.availMem = Math.min(configuredTotal, configuredAvailable);
                                }
                            }
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": getMemoryInfo failed: " + t);
                        }
                    }
                });
        });

        Legacy.safeHook(TAG, "ActivityManager.getMemoryClass", () -> {
            Legacy.findAndHookMethod(activityManagerClass, "getMemoryClass",
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                        try {
                            if (error != null) {
                                return;
                            }
                            chain.replaceResult(ConfigManager.getMemoryClassMb());
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": getMemoryClass failed: " + t);
                        }
                    }
                });
        });

        Legacy.safeHook(TAG, "ActivityManager.getLargeMemoryClass", () -> {
            Legacy.findAndHookMethod(activityManagerClass, "getLargeMemoryClass",
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                        try {
                            if (error != null) {
                                return;
                            }
                            chain.replaceResult(ConfigManager.getLargeMemoryClassMb());
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": getLargeMemoryClass failed: " + t);
                        }
                    }
                });
        });
    }

    private static void hookDebuggerState() {
        // ACE DEX anti-debug: isDebuggerConnected / waitingForDebugger leak
        // the LSPosed/Vector JDWP session. A user build has no debugger.
        // Single no-arg overloads each; hookAllMethods keeps the install
        // uniform. Fail-closed: errors keep the original value.
        Legacy.safeHook(TAG, "Debug.isDebuggerConnected", () -> {
            HookFramework.hookAllMethods(Debug.class, "isDebuggerConnected",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof Boolean
                                        && (Boolean) result) {
                                    chain.replaceResult(false);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": isDebuggerConnected failed: "
                                        + t);
                            }
                        }
                    });
        });
        Legacy.safeHook(TAG, "Debug.waitingForDebugger", () -> {
            HookFramework.hookAllMethods(Debug.class, "waitingForDebugger",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof Boolean
                                        && (Boolean) result) {
                                    chain.replaceResult(false);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": waitingForDebugger failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static void hookDebugMemory() {
        if (!DEBUG_MEMORY_HOOKED.compareAndSet(false, true)) {
            return;
        }
        // getNativeHeapSize: single no-arg overload. Fail-closed.
        Legacy.safeHook(TAG, "Debug.getNativeHeapSize", () -> {
            Legacy.findAndHookMethod(Debug.class, "getNativeHeapSize",
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                        try {
                            if (error != null
                                    || !(result instanceof Long)) {
                                return;
                            }
                            long originalSize = (Long) result;
                            chain.replaceResult(originalSize * Math.max(1, ConfigManager.getNativeHeapScale()));
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": getNativeHeapSize failed: " + t);
                        }
                    }
                });
        });
    }
}
