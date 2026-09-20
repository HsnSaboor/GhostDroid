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

        try {
            Legacy.findAndHookMethod(telephonyManager, "getDeviceId",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getIMEI();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getDeviceId", int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getIMEI();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getImei",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getIMEI();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getImei", int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getIMEI();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getMeid",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getMEID();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getMeid", int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getMEID();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getSubscriberId",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getIMSI();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getSubscriberId", int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getIMSI();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getSimSerialNumber",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getICCID();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getSimSerialNumber", int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getICCID();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getLine1Number",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getPhoneNumber();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getLine1Number", int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getPhoneNumber();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        // Hook network operator methods (MCC/MNC)
        try {
            Legacy.findAndHookMethod(telephonyManager, "getNetworkOperator",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String mccMnc = ConfigManager.getSystemProperty("gsm.operator.numeric", null);
                            if (mccMnc != null) {
                                chain.replaceResult(mccMnc);
                            }
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getNetworkOperatorName",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String operatorName = ConfigManager.getSystemProperty("gsm.operator.alpha", null);
                            if (operatorName != null) {
                                chain.replaceResult(operatorName);
                            }
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getSimOperator",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String simMccMnc = ConfigManager.getSystemProperty("gsm.sim.operator.numeric", null);
                            if (simMccMnc != null) {
                                chain.replaceResult(simMccMnc);
                            }
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getSimOperatorName",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String simOperatorName = ConfigManager.getSystemProperty("gsm.sim.operator.alpha", null);
                            if (simOperatorName != null) {
                                chain.replaceResult(simOperatorName);
                            }
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getSimCountryIso",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String simCountry = ConfigManager.getSystemProperty("gsm.sim.operator.iso-country", null);
                            if (simCountry != null) {
                                chain.replaceResult(simCountry);
                            }
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        try {
            Legacy.findAndHookMethod(telephonyManager, "getNetworkCountryIso",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String networkCountry = ConfigManager.getSystemProperty("gsm.operator.iso-country", null);
                            if (networkCountry != null) {
                                chain.replaceResult(networkCountry);
                            }
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        // Android 13+ replaced TelephonyManager.getLine1Number() with
        // SubscriptionManager.getPhoneNumber(int) and getPhoneNumber(int, int).
        hookSubscriptionManager(lpparam);
        hookSimState(telephonyManager);
        hookPhoneCount(telephonyManager);
        hookHasIccCard(telephonyManager);
        hookNetworkOperatorForPhone(telephonyManager);
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
            try {
                Legacy.findAndHookMethod(subscriptionManager, "getPhoneNumber",
                        int.class, phoneNumberHook);
            } catch (NoSuchMethodError ignored) {
            }
            try {
                Legacy.findAndHookMethod(subscriptionManager, "getPhoneNumber",
                        int.class, int.class, phoneNumberHook);
            } catch (NoSuchMethodError ignored) {
            }

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
