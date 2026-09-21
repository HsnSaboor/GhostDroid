package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class TelephonyHooks {

    public static void hook(HookContext lpparam) {
        Class<?> telephonyManager = Legacy.findClassIfExists(
                "android.telephony.TelephonyManager",
                lpparam.classLoader
        );

        if (telephonyManager == null) {
            return;
        }

        // Identity strings pinned to the profile. hookAllMethods covers
        // every overload at once (no-arg + slot-index + future int/int
        // forms), so per-overload findAndHookMethod clones cannot drift.
        // Every after-hook is fail-closed: try/catch + Legacy.log, original
        // kept on error.
        pinIdentity(telephonyManager, "getDeviceId", ConfigManager.getIMEI());
        pinIdentity(telephonyManager, "getImei", ConfigManager.getIMEI());
        pinIdentity(telephonyManager, "getMeid", ConfigManager.getMEID());
        pinIdentity(telephonyManager, "getSubscriberId", ConfigManager.getIMSI());
        pinIdentity(telephonyManager, "getSimSerialNumber", ConfigManager.getICCID());
        pinIdentity(telephonyManager, "getLine1Number", ConfigManager.getPhoneNumber());
        // getPhoneNumber (API 33+ TelephonyManager direct): same spoofed
        // number; hookAllMethods is a no-op where the method is absent.
        pinIdentity(telephonyManager, "getPhoneNumber", ConfigManager.getPhoneNumber());

        // Operator strings pinned to the profile. TelephonyCellHooks owns
        // the int/subId overloads; hookAllMethods here covers every overload
        // of each name so no-arg and int forms cannot drift.
        pinOperator(telephonyManager, "getNetworkOperator",
                ConfigManager.getSystemProperty("gsm.operator.numeric", null));
        pinOperator(telephonyManager, "getNetworkOperatorName",
                ConfigManager.getSystemProperty("gsm.operator.alpha", null));
        pinOperator(telephonyManager, "getSimOperator",
                ConfigManager.getSystemProperty("gsm.sim.operator.numeric", null));
        pinOperator(telephonyManager, "getSimOperatorName",
                ConfigManager.getSystemProperty("gsm.sim.operator.alpha", null));
        pinOperator(telephonyManager, "getSimCountryIso",
                ConfigManager.getSystemProperty("gsm.sim.operator.iso-country", null));
        pinOperator(telephonyManager, "getNetworkCountryIso",
                ConfigManager.getSystemProperty("gsm.operator.iso-country", null));

        // Android 13+ replaced TelephonyManager.getLine1Number() with
        // SubscriptionManager.getPhoneNumber(int) and getPhoneNumber(int, int).
        hookSubscriptionManager(lpparam);
        hookSimState(telephonyManager);
        hookPhoneCount(telephonyManager);
        hookHasIccCard(telephonyManager);
        hookNetworkOperatorForPhone(telephonyManager);
    }

    // Single shared identity pin: covers no-arg + slot-index overloads
    // via hookAllMethods (delegates to pinTelephonyString).
    // Fail-closed: errors keep the original value.
    private static void pinIdentity(Class<?> tm, String name, String value) {
        pinTelephonyString(tm, name, value);
    }

    private static void pinOperator(Class<?> tm, String name, String value) {
        pinTelephonyString(tm, name, value);
    }

    // Shared TelephonyManager string pin: hookAllMethods covers no-arg
    // + slot-index + future overloads (identity + operator share it).
    // Fail-closed: errors keep the original value.
    private static void pinTelephonyString(Class<?> tm, String name, String value) {
        final String method = name;
        Legacy.safeHook("DeviceSpoofLab-Telephony", method, () -> {
            HookFramework.hookAllMethods(tm, method,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (value != null) {
                                    chain.replaceResult(value);
                                }
                            } catch (Throwable t) {
                                Legacy.log("DeviceSpoofLab-Telephony"
                                        + ": " + method + " failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookSimState(Class<?> telephonyManager) {
        Legacy.safeHook("DeviceSpoofLab-Telephony", "getSimState", () -> {
            HookFramework.hookAllMethods(telephonyManager, "getSimState",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                chain.replaceResult(5);
                            } catch (Throwable t) {
                                Legacy.log("DeviceSpoofLab-Telephony"
                                        + ": getSimState failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookPhoneCount(Class<?> telephonyManager) {
        Legacy.safeHook("DeviceSpoofLab-Telephony", "getPhoneCount", () -> {
            HookFramework.hookAllMethods(telephonyManager, "getPhoneCount",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                chain.replaceResult(2);
                            } catch (Throwable t) {
                                Legacy.log("DeviceSpoofLab-Telephony"
                                        + ": getPhoneCount failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookHasIccCard(Class<?> telephonyManager) {
        Legacy.safeHook("DeviceSpoofLab-Telephony", "hasIccCard", () -> {
            HookFramework.hookAllMethods(telephonyManager, "hasIccCard",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                chain.replaceResult(true);
                            } catch (Throwable t) {
                                Legacy.log("DeviceSpoofLab-Telephony"
                                        + ": hasIccCard failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookNetworkOperatorForPhone(Class<?> telephonyManager) {
        Legacy.safeHook("DeviceSpoofLab-Telephony", "getNetworkOperatorForPhone", () -> {
            HookFramework.hookAllMethods(telephonyManager, "getNetworkOperatorForPhone",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                String mccMnc = ConfigManager.getSystemProperty(
                                        "gsm.operator.numeric", null);
                                if (mccMnc != null) {
                                    chain.replaceResult(mccMnc);
                                }
                            } catch (Throwable t) {
                                Legacy.log("DeviceSpoofLab-Telephony"
                                        + ": getNetworkOperatorForPhone failed: " + t);
                            }
                        }
                    });
        });
    }

    // Single-entry T-Mobile (310260) subscription list so SIM-presence
    // checks pass on Waydroid's SIM-less build. SubscriptionInfo has no
    // public constructor: fill the widest hidden ctor by type, then set
    // carrier fields reflectively. Fail-closed on any error.
    private static void hookSubscriptionManager(HookContext lpparam) {
        Class<?> subscriptionManager = Legacy.findClassIfExists(
                "android.telephony.SubscriptionManager", lpparam.classLoader);
        if (subscriptionManager != null) {
            HookFramework.Hook phoneNumberHook = new HookFramework.Hook() {
                @Override
                public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                    try {
                        String v = ConfigManager.getPhoneNumber();
                        if (v != null) chain.replaceResult(v);
                    } catch (Throwable t) {
                        Legacy.log("DeviceSpoofLab-Telephony"
                                + ": SubscriptionManager.getPhoneNumber failed: " + t);
                    }
                }
            };
            // Exact (int) + (int,int) forms (two overloads only);
            // fail-closed install with try/catch + Legacy.log.
            Legacy.safeHook("DeviceSpoofLab-Telephony",
                    "SubscriptionManager.getPhoneNumber", () -> {
                try {
                    Legacy.findAndHookMethod(subscriptionManager, "getPhoneNumber",
                            int.class, phoneNumberHook);
                } catch (Throwable ignored) {
                }
                try {
                    Legacy.findAndHookMethod(subscriptionManager, "getPhoneNumber",
                            int.class, int.class, phoneNumberHook);
                } catch (Throwable ignored) {
                }
            });

            Legacy.safeHook("DeviceSpoofLab-Telephony",
                    "getActiveSubscriptionInfoList", () -> {
                        HookFramework.hookAllMethods(subscriptionManager,
                                "getActiveSubscriptionInfoList",
                                new HookFramework.Hook() {
                                    @Override
                                    @SuppressWarnings("unchecked")
                                    public void after(HookFramework.HookChain chain,
                                            Object result, Throwable error) {
                                        try {
                                            java.util.List<Object> orig =
                                                    (java.util.List<Object>) result;
                                            if (orig != null && !orig.isEmpty()) {
                                                return;
                                            }
                                            Object info = syntheticSubscriptionInfo(
                                                    lpparam.classLoader);
                                            if (info != null) {
                                                java.util.List<Object> single =
                                                        new java.util.ArrayList<>(1);
                                                single.add(info);
                                                chain.replaceResult(single);
                                            }
                                        } catch (Throwable t) {
                                            Legacy.log("DeviceSpoofLab-Telephony"
                                                    + ": getActiveSubscriptionInfoList failed: "
                                                    + t);
                                        }
                                    }
                                });
                    });

            // Bulk-list siblings some detectors enumerate instead of the
            // base list: same empty -> single synthetic T-Mobile entry.
            for (String bulk : new String[]{
                    "getCompleteActiveSubscriptionInfoList",
                    "getAccessibleSubscriptionInfoList",
                    "getAvailableSubscriptionInfoList"}) {
                final String bulkName = bulk;
                Legacy.safeHook("DeviceSpoofLab-Telephony", bulkName, () -> {
                    HookFramework.hookAllMethods(subscriptionManager, bulkName,
                            new HookFramework.Hook() {
                                @Override
                                @SuppressWarnings("unchecked")
                                public void after(HookFramework.HookChain chain,
                                        Object result, Throwable error) {
                                    try {
                                        java.util.List<Object> orig =
                                                (java.util.List<Object>) result;
                                        if (orig != null && !orig.isEmpty()) {
                                            return;
                                        }
                                        Object info = syntheticSubscriptionInfo(
                                                lpparam.classLoader);
                                        if (info != null) {
                                            java.util.List<Object> single =
                                                    new java.util.ArrayList<>(1);
                                            single.add(info);
                                            chain.replaceResult(single);
                                        }
                                    } catch (Throwable t) {
                                        Legacy.log("DeviceSpoofLab-Telephony"
                                                + ": " + bulkName + " failed: " + t);
                                    }
                                }
                            });
                });
            }

            // Single-entry lookups bypass the list: backfill null with the
            // same synthetic entry so slot-index checks see a SIM present.
            for (String single : new String[]{
                    "getActiveSubscriptionInfo",
                    "getActiveSubscriptionInfoForSimSlotIndex",
                    "getActiveSubscriptionInfoForSubscription"}) {
                final String singleName = single;
                Legacy.safeHook("DeviceSpoofLab-Telephony", singleName, () -> {
                    HookFramework.hookAllMethods(subscriptionManager, singleName,
                            new HookFramework.Hook() {
                                @Override
                                public void after(HookFramework.HookChain chain,
                                        Object result, Throwable error) {
                                    try {
                                        if (result != null) {
                                            return;
                                        }
                                        Object info = syntheticSubscriptionInfo(
                                                lpparam.classLoader);
                                        if (info != null) {
                                            chain.replaceResult(info);
                                        }
                                    } catch (Throwable t) {
                                        Legacy.log("DeviceSpoofLab-Telephony"
                                                + ": " + singleName + " failed: " + t);
                                    }
                                }
                            });
                });
            }
        }
    }

    private static Object syntheticSubscriptionInfo(ClassLoader loader) {
        try {
            Class<?> si = Legacy.findClassIfExists(
                    "android.telephony.SubscriptionInfo", loader);
            if (si == null) {
                return null;
            }
            java.lang.reflect.Constructor<?> widest = null;
            for (java.lang.reflect.Constructor<?> c : si.getDeclaredConstructors()) {
                if (widest == null
                        || c.getParameterTypes().length > widest.getParameterTypes().length) {
                    widest = c;
                }
            }
            if (widest == null) {
                return null;
            }
            Class<?>[] types = widest.getParameterTypes();
            Object[] args = new Object[types.length];
            String iccId = ConfigManager.getICCID();
            if (iccId == null || iccId.isEmpty()) {
                iccId = "8901260422345678901";
            }
            int intSeen = 0;
            for (int i = 0; i < types.length; i++) {
                Class<?> t = types[i];
                if (t == int.class) {
                    intSeen++;
                    if (intSeen == 1) {
                        args[i] = 1;
                    } else if (intSeen == 3) {
                        args[i] = 0;
                    } else {
                        args[i] = 0;
                    }
                } else if (t == long.class) {
                    args[i] = 0L;
                } else if (t == boolean.class) {
                    args[i] = true;
                } else if (t == String.class) {
                    args[i] = iccId;
                } else if (t == CharSequence.class) {
                    args[i] = "T-Mobile";
                } else {
                    args[i] = null;
                }
            }
            Object inst;
            try {
                widest.setAccessible(true);
                inst = widest.newInstance(args);
            } catch (Throwable t) {
                Legacy.log("DeviceSpoofLab-Telephony"
                        + ": SubscriptionInfo ctor failed: " + t);
                return null;
            }
            Legacy.setObjectField(inst, "mDisplayName", "T-Mobile");
            Legacy.setObjectField(inst, "mCarrierName", "T-Mobile");
            Legacy.setObjectField(inst, "mMcc", "310");
            Legacy.setObjectField(inst, "mMnc", "260");
            Legacy.setObjectField(inst, "mCountryIso", "us");
            try {
                Legacy.setIntField(inst, "mId", 1);
                Legacy.setIntField(inst, "mSubscriptionId", 1);
                Legacy.setIntField(inst, "mSimSlotIndex", 0);
            } catch (Throwable ignored) {
            }
            return inst;
        } catch (Throwable t) {
            Legacy.log("DeviceSpoofLab-Telephony"
                    + ": syntheticSubscriptionInfo failed: " + t);
            return null;
        }
    }
}
