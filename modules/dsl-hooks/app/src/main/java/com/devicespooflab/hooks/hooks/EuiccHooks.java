package com.devicespooflab.hooks.hooks;

import android.os.Build;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class EuiccHooks {

    private static final String TAG = "DeviceSpoofLab-Euicc";

    public static void hook(HookContext lpparam) {
        hook(lpparam, Build.VERSION.SDK_INT);
    }

    public static void hook(HookContext lpparam, int realDeviceSdk) {
        if (realDeviceSdk < 28) return;

        Class<?> em = Legacy.findClassIfExists(
                "android.telephony.euicc.EuiccManager", lpparam.classLoader);
        if (em == null) return;

        Legacy.safeHook(TAG, "EuiccManager.getEid", () -> {
            HookFramework.hookAllMethods(em, "getEid",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                String v = ConfigManager.getEid();
                                if (v != null) {
                                    chain.replaceResult(v);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getEid failed: " + t);
                            }
                        }
                    });
        });

        // EuiccManager.isEnabled / isSupported: a flagship eSIM phone
        // reports true; Waydroid's SIM-less build reports disabled.
        // EuiccInfo.getEid: object-returning EID sibling some detectors
        // read instead of EuiccManager.getEid — same spoofed EID.
        // Fail-closed: errors keep the original value.
        Legacy.safeHook(TAG, "EuiccManager.isEnabled", () -> {
            HookFramework.hookAllMethods(em, "isEnabled",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (result instanceof Boolean
                                        && !(Boolean) result) {
                                    chain.replaceResult(true);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": isEnabled failed: " + t);
                            }
                        }
                    });
        });
        Legacy.safeHook(TAG, "EuiccManager.isSupported", () -> {
            HookFramework.hookAllMethods(em, "isSupported",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (result instanceof Boolean
                                        && !(Boolean) result) {
                                    chain.replaceResult(true);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": isSupported failed: " + t);
                            }
                        }
                    });
        });
        Class<?> euiccInfo = Legacy.findClassIfExists(
                "android.telephony.euicc.EuiccInfo", lpparam.classLoader);
        if (euiccInfo != null) {
            Legacy.safeHook(TAG, "EuiccInfo.getEid", () -> {
                HookFramework.hookAllMethods(euiccInfo, "getEid",
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    String v = ConfigManager.getEid();
                                    if (v != null) {
                                        chain.replaceResult(v);
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": EuiccInfo.getEid failed: "
                                            + t);
                                }
                            }
                        });
            });
        }
    }
}
