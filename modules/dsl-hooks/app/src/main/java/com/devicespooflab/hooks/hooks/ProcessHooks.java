package com.devicespooflab.hooks.hooks;

import android.app.ActivityManager;

import com.devicespooflab.hooks.bridge.HookContext;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.utils.DenyTokens;

import java.util.ArrayList;
import java.util.List;

// Running-process / service / task lists ACE DEX enumerates directly.
// Strips root/cloak tooling (Magisk/Xposed/Frida/Shizuku) via the shared
// DenyTokens set so the deny list cannot drift from ApplistHooks.
// Fail-closed: any reflection error keeps the original list.
public final class ProcessHooks {

    private static final String TAG = "DeviceSpoofLab-Process";

    private ProcessHooks() {}

    public static void hook(HookContext lpparam) {
        Class<?> am = Legacy.findClassIfExists(
                "android.app.ActivityManager", lpparam.classLoader);
        if (am == null) {
            return;
        }
        hookRunningAppProcesses(am);
        hookRunningServices(am);
        hookRecentTasks(am);
        hookRunningTasks(am);
        hookAppTasks(am);
        hookProcessMemory(am, lpparam);
    }

    private static String processOf(Object entry) {
        try {
            Object name = Legacy.getObjectField(entry, "processName");
            if (name instanceof String) {
                return (String) name;
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    private static String serviceClassOf(Object entry) {
        try {
            Object cn = Legacy.getObjectField(entry, "service");
            if (cn instanceof android.content.ComponentName) {
                return ((android.content.ComponentName) cn).getPackageName();
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    private static String taskBaseOf(Object entry) {
        try {
            Object cn = Legacy.getObjectField(entry, "baseActivity");
            if (cn instanceof android.content.ComponentName) {
                return ((android.content.ComponentName) cn).getPackageName();
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    private static void hookRunningAppProcesses(Class<?> am) {
        Legacy.safeHook(TAG, "ActivityManager.getRunningAppProcesses", () -> {
            HookFramework.hookAllMethods(am, "getRunningAppProcesses",
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
                                    if (!DenyTokens.isDeniedProcess(
                                            processOf(entry))) {
                                        kept.add(entry);
                                    }
                                }
                                if (kept.size() != orig.size()) {
                                    chain.replaceResult(kept);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getRunningAppProcesses failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static void hookRunningServices(Class<?> am) {
        Legacy.safeHook(TAG, "ActivityManager.getRunningServices", () -> {
            HookFramework.hookAllMethods(am, "getRunningServices",
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
                                    String pkg = serviceClassOf(entry);
                                    if (pkg == null) {
                                        pkg = processOf(entry);
                                    }
                                    if (!DenyTokens.isDeniedProcess(pkg)
                                            && !DenyTokens.isDeniedPackage(pkg)) {
                                        kept.add(entry);
                                    }
                                }
                                if (kept.size() != orig.size()) {
                                    chain.replaceResult(kept);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getRunningServices failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookRecentTasks(Class<?> am) {
        Legacy.safeHook(TAG, "ActivityManager.getRecentTasks", () -> {
            HookFramework.hookAllMethods(am, "getRecentTasks",
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
                                    if (!DenyTokens.isDeniedPackage(
                                            taskBaseOf(entry))) {
                                        kept.add(entry);
                                    }
                                }
                                if (kept.size() != orig.size()) {
                                    chain.replaceResult(kept);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getRecentTasks failed: " + t);
                            }
                        }
                    });
        });
    }

    // Deprecated getRunningTasks (API < 21 remnant ACE still calls via
    // reflection): same deny filter as getRecentTasks. Fail-closed.
    private static void hookRunningTasks(Class<?> am) {
        Legacy.safeHook(TAG, "ActivityManager.getRunningTasks", () -> {
            HookFramework.hookAllMethods(am, "getRunningTasks",
                    new HookFramework.Hook() {
                        @Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (!(result instanceof java.util.List)) {
                                    return;
                                }
                                java.util.List<?> orig =
                                        (java.util.List<?>) result;
                                java.util.List<Object> kept =
                                        new ArrayList<>(orig.size());
                                for (Object entry : orig) {
                                    if (entry == null) {
                                        continue;
                                    }
                                    if (!DenyTokens.isDeniedPackage(
                                            taskBaseOf(entry))) {
                                        kept.add(entry);
                                    }
                                }
                                if (kept.size() != orig.size()) {
                                    chain.replaceResult(kept);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getRunningTasks failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    // getProcessMemoryInfo / getMyMemoryState: per-pid memory snapshots.
    // Debug.MemoryInfo carries memory counters only (no pid/host fields),
    // so there is nothing host-specific to scrub — and trimming the array
    // would break the parallel-to-pids contract. Passthrough guard retained
    // so the surface is audited and fail-closed. Real numbers stay real.
    private static void hookProcessMemory(Class<?> am, HookContext lpparam) {
        // lpparam scopes future per-caller filtering; nothing to pin today.
        if (lpparam == null || lpparam.classLoader == null) {
            return;
        }
        Legacy.safeHook(TAG, "ActivityManager.getProcessMemoryInfo", () -> {
            HookFramework.hookAllMethods(am, "getProcessMemoryInfo",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getProcessMemoryInfo failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static String appTaskPackage(Object task) {
        try {
            Object info = Legacy.callMethod(task, "getTaskInfo");
            if (info != null) {
                Object cn = Legacy.getObjectField(info, "baseActivity");
                if (cn instanceof android.content.ComponentName) {
                    return ((android.content.ComponentName) cn).getPackageName();
                }
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    private static void hookAppTasks(Class<?> am) {
        Legacy.safeHook(TAG, "ActivityManager.getAppTasks", () -> {
            HookFramework.hookAllMethods(am, "getAppTasks",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (!(result instanceof List)) {
                                    return;
                                }
                                // AppTask carries no package directly; resolve
                                // via getTaskInfo() when available, else keep.
                                List<?> orig = (List<?>) result;
                                List<Object> kept = new ArrayList<>(orig.size());
                                for (Object task : orig) {
                                    if (task == null) {
                                        continue;
                                    }
                                    String pkg = appTaskPackage(task);
                                    if (!DenyTokens.isDeniedPackage(pkg)) {
                                        kept.add(task);
                                    }
                                }
                                if (kept.size() != orig.size()) {
                                    chain.replaceResult(kept);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getAppTasks failed: " + t);
                            }
                        }
                    });
        });
    }
}
