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
            hookDebugMemory();
            if (ConfigManager.isVerboseLoggingEnabled()) {
                Legacy.log(TAG + ": Successfully hooked hardware specs");
            }
        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook hardware: " + e.getMessage());
        }
    }

    private static void hookRuntimeCores() {
        if (!RUNTIME_CORES_HOOKED.compareAndSet(false, true)) {
            return;
        }
        try {
            Legacy.findAndHookMethod(Runtime.class, "availableProcessors",
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) throws Throwable {
                        chain.replaceResult(ConfigManager.getCpuCoreCount());
                    }
                });
        } catch (Exception e) {
            RUNTIME_CORES_HOOKED.set(false);
            Legacy.log(TAG + ": Failed to hook Runtime.availableProcessors(): " + e.getMessage());
        }
    }

    private static void hookActivityManagerMemory(HookContext lpparam) {
        try {
            Class<?> activityManagerClass = Legacy.findClassIfExists(
                "android.app.ActivityManager", lpparam.classLoader);

            if (activityManagerClass == null) {
                return;
            }
            if (!HOOKED_ACTIVITY_MANAGER_CLASSES.add(activityManagerClass)) {
                return;
            }

            Legacy.findAndHookMethod(activityManagerClass, "getMemoryInfo",
                ActivityManager.MemoryInfo.class,
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) throws Throwable {
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
                    }
                });

            Legacy.findAndHookMethod(activityManagerClass, "getMemoryClass",
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) throws Throwable {
                        chain.replaceResult(ConfigManager.getMemoryClassMb());
                    }
                });

            Legacy.findAndHookMethod(activityManagerClass, "getLargeMemoryClass",
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) throws Throwable {
                        chain.replaceResult(ConfigManager.getLargeMemoryClassMb());
                    }
                });

        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook ActivityManager memory: " + e.getMessage());
        }
    }

    private static void hookDebugMemory() {
        if (!DEBUG_MEMORY_HOOKED.compareAndSet(false, true)) {
            return;
        }
        try {
            Legacy.findAndHookMethod(Debug.class, "getNativeHeapSize",
                new HookFramework.Hook() {@Override
                    public void after(HookFramework.HookChain chain, Object result, Throwable error) throws Throwable {
                        long originalSize = (Long) result;
                        chain.replaceResult(originalSize * Math.max(1, ConfigManager.getNativeHeapScale()));
                    }
                });
        } catch (Exception e) {
            DEBUG_MEMORY_HOOKED.set(false);
        }
    }
}
