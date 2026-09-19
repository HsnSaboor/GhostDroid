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
        Class<?> subscriptionManager = Legacy.findClassIfExists(
                "android.telephony.SubscriptionManager", lpparam.classLoader);
        if (subscriptionManager != null) {
            HookFramework.Hook phoneNumberHook = new HookFramework.Hook() {
                @Override
                public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                    String v = ConfigManager.getPhoneNumber();
                    if (v != null) chain.replaceResult(v);
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
        }
    }
}
