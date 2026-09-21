package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.bridge.HookContext;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.utils.ConfigManager;

// Cell / signal / subscription defaults ACE DEX reads directly.
// TelephonyHooks owns identity strings (IMEI/IMSI/ICCID); this file owns
// radio-state shapes: operator overloads pinned to T-Mobile, signal level
// sane, default subscription ids pinned to the synthetic sub (id 1).
// Java cannot fabricate a full CellInfo graph safely per API level, so
// cell-list contents stay passthrough (fail-closed); only scalar radio
// state is pinned. Every hook is fail-closed.
public final class TelephonyCellHooks {

    private static final String TAG = "DeviceSpoofLab-TelephonyCell";

    private TelephonyCellHooks() {}

    public static void hook(HookContext lpparam) {
        Class<?> tm = Legacy.findClassIfExists(
                "android.telephony.TelephonyManager", lpparam.classLoader);
        if (tm != null) {
            hookCellLists(tm);
            hookCellLocation(tm);
            hookCellLocationExtra(tm);
            hookRadioState(tm);
            hookSignalStrength(tm, lpparam);
            hookExtraIdentity(tm);
            hookServiceState(tm, lpparam);
        }
        hookSubscriptionDefaults(lpparam);
        hookSubscriptionManagerExtra(lpparam);
        hookSignalStrengthClass(lpparam);
    }

    // Service-state shape ACE gates on: SIM-less Waydroid reports no voice/
    // data service; claim in-service + LTE roaming-consistent state so the
    // emulator notice's radio branch cannot fire. Fail-closed throughout.
    private static void hookServiceState(Class<?> tm, HookContext lpparam) {
        Legacy.safeHook(TAG, "TelephonyManager.getServiceState", () -> {
            HookFramework.hookAllMethods(tm, "getServiceState",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                Object patched = patchServiceState(result,
                                        lpparam.classLoader);
                                if (patched != null) {
                                    chain.replaceResult(patched);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getServiceState failed: "
                                        + t);
                            }
                        }
                    });
        });
        // Voice/data registration constants: in-service = 0.
        pinIntMethod(tm, "getVoiceNetworkType", 13);
        pinIntMethod(tm, "getVoiceRegState", 0);
        pinIntMethod(tm, "getDataRegState", 0);
    }

    private static Object patchServiceState(Object state, ClassLoader loader) {
        if (state == null) {
            return null;
        }
        try {
            Legacy.setIntField(state, "mVoiceRegState", 0);
            Legacy.setIntField(state, "mDataRegState", 0);
            Legacy.setIntField(state, "mVoiceNetworkType", 13);
            Legacy.setIntField(state, "mDataNetworkType", 13);
            Legacy.setObjectField(state, "mOperatorAlphaLong", "T-Mobile");
            Legacy.setObjectField(state, "mOperatorAlphaShort", "T-Mobile");
            Legacy.setObjectField(state, "mOperatorNumeric", "310260");
            return state;
        } catch (Throwable t) {
            Legacy.log(TAG + ": patchServiceState failed: " + t);
            return null;
        }
    }

    // SubscriptionManager siblings the synthetic-list hook misses:
    // per-slot default phone / data-enabled / oppor-mode / roaming-guard /
    // carrier-config bundle paths that bypass the single-entry list.
    private static void hookSubscriptionManagerExtra(HookContext lpparam) {
        Class<?> sm = Legacy.findClassIfExists(
                "android.telephony.SubscriptionManager", lpparam.classLoader);
        if (sm == null) {
            return;
        }
        Legacy.safeHook(TAG, "SubscriptionManager.getDefaultSubscriptionId-extra", () -> {
            for (String name : new String[]{
                    "getDefaultSubscription", "getPreferredDataSubscriptionId",
                    "getPreferredVoiceSubscriptionId",
                    "getPreferredSmsSubscriptionId",
                    "getDefaultDataPhoneId",
                    "getPhoneId"}) {
                final String method = name;
                HookFramework.hookAllMethods(sm, method,
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    if (result instanceof Integer
                                            && (Integer) result < 0) {
                                        chain.replaceResult(0);
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": " + method
                                            + " failed: " + t);
                                }
                            }
                        });
            }
        });
        Legacy.safeHook(TAG, "SubscriptionManager.isDataEnabled", () -> {
            HookFramework.hookAllMethods(sm, "isDataRoamingEnabled",
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
                                Legacy.log(TAG + ": isDataRoamingEnabled failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    // Real cell contents cannot be fabricated safely per API level, but an
    // empty getAllCellInfo is a SIM-less tell: reuse the framework's own
    // CellInfoLte shell when available so presence checks see one LTE cell.
    // Fail-closed: any reflection error keeps the empty list.
    private static void hookCellLists(Class<?> tm) {
        hookOperatorOverloads(tm);
        Legacy.safeHook(TAG, "TelephonyManager.getAllCellInfo", () -> {
            HookFramework.hookAllMethods(tm, "getAllCellInfo",
                    new HookFramework.Hook() {
                        @Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (result instanceof java.util.List
                                        && !((java.util.List<?>) result)
                                                .isEmpty()) {
                                    return;
                                }
                                Object cell = syntheticCellInfoLte();
                                if (cell != null) {
                                    java.util.List<Object> single =
                                            new java.util.ArrayList<>(1);
                                    single.add(cell);
                                    chain.replaceResult(single);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getAllCellInfo failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static Object syntheticCellInfoLte() {
        try {
            Class<?> cell =
                    Class.forName("android.telephony.CellInfoLte");
            for (java.lang.reflect.Constructor<?> c
                    : cell.getDeclaredConstructors()) {
                Class<?>[] types = c.getParameterTypes();
                Object[] args = new Object[types.length];
                boolean usable = true;
                for (int i = 0; i < types.length; i++) {
                    if (types[i] == boolean.class) {
                        args[i] = false;
                    } else if (types[i] == long.class) {
                        args[i] = System.nanoTime();
                    } else {
                        args[i] = null;
                    }
                    if (types[i] != boolean.class
                            && types[i] != long.class
                            && !types[i].isPrimitive()
                            && args[i] != null) {
                        usable = false;
                    }
                }
                if (!usable) {
                    continue;
                }
                try {
                    c.setAccessible(true);
                    return c.newInstance(args);
                } catch (Throwable ignored) {
                }
            }
            return null;
        } catch (Throwable t) {
            Legacy.log(TAG + ": syntheticCellInfoLte failed: " + t);
            return null;
        }
    }

    // Cell-location + neighboring-cell + service-state siblings that bypass
    // the operator overloads: null/empty leaks the SIM-less radio.
    // Fail-closed: null results are backfilled only when a neutral value
    // exists; errors keep the original.
    private static void hookCellLocationExtra(Class<?> tm) {
        for (String name : new String[]{
                "getCellLocation", "getNeighboringCellInfo"}) {
            final String method = name;
            Legacy.safeHook(TAG, "TelephonyManager." + method, () -> {
                HookFramework.hookAllMethods(tm, method,
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    if (result instanceof java.util.List
                                            && ((java.util.List<?>) result)
                                                    .isEmpty()) {
                                        chain.replaceResult(
                                                java.util.Collections
                                                        .emptyList());
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

    // Radio-state siblings the operator pin misses: manual/auto selection,
    // roaming, CDMA/EvDo leak branches, and forbidden-PLMN enumeration.
    private static void hookRadioState(Class<?> tm) {
        pinBooleanMethod(tm, "isNetworkRoaming", false);
        pinBooleanMethod(tm, "isDataRoamingEnabled", false);
        for (String name : new String[]{
                "getCdmaMdn", "getCdmaMin", "getCdmaPrlVersion",
                "getForbiddenPlmns", "getEquivalentHomePlmns"}) {
            final String method = name;
            Legacy.safeHook(TAG, "TelephonyManager." + method, () -> {
                HookFramework.hookAllMethods(tm, method,
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    if (result instanceof String
                                            && ((String) result).isEmpty()) {
                                        return;
                                    }
                                    if (result instanceof java.util.List
                                            && ((java.util.List<?>) result)
                                                    .isEmpty()) {
                                        return;
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": " + method
                                            + " failed: " + t);
                                }
                            }
                        });
            });
        }
        Legacy.safeHook(TAG, "TelephonyManager.getNetworkSelectionMode", () -> {
            HookFramework.hookAllMethods(tm, "getNetworkSelectionMode",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (result instanceof Integer
                                        && (Integer) result != 0) {
                                    chain.replaceResult(0);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG
                                        + ": getNetworkSelectionMode failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static void pinBooleanMethod(Class<?> tm, String name,
            boolean value) {
        final String method = name;
        Legacy.safeHook(TAG, "TelephonyManager." + method, () -> {
            HookFramework.hookAllMethods(tm, method,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (result instanceof Boolean
                                        && (Boolean) result != value) {
                                    chain.replaceResult(value);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": " + method
                                        + " failed: " + t);
                            }
                        }
                    });
        });
    }

    // Per-subId operator overloads TelephonyHooks misses: getNetworkOperator
    // (int), getSimOperator(int), getNetworkOperatorName(int),
    // getSimOperatorName(int), getNetworkCountryIso(int),
    // getSimCountryIso(int), getNetworkType(int), getDataNetworkType(int).
    // Pinned to the same T-Mobile / 310260 / LTE values.
    private static void hookOperatorOverloads(Class<?> tm) {
        pinStringMethod(tm, "getNetworkOperator",
                ConfigManager.getSystemProperty("gsm.operator.numeric", "310260"));
        pinStringMethod(tm, "getNetworkOperatorName",
                ConfigManager.getSystemProperty("gsm.operator.alpha", "T-Mobile"));
        pinStringMethod(tm, "getSimOperator",
                ConfigManager.getSystemProperty("gsm.sim.operator.numeric", "310260"));
        pinStringMethod(tm, "getSimOperatorName",
                ConfigManager.getSystemProperty("gsm.sim.operator.alpha", "T-Mobile"));
        pinStringMethod(tm, "getNetworkCountryIso",
                ConfigManager.getSystemProperty("gsm.operator.iso-country", "us"));
        pinStringMethod(tm, "getSimCountryIso",
                ConfigManager.getSystemProperty("gsm.sim.operator.iso-country", "us"));
        pinIntMethod(tm, "getNetworkType", 13);
        pinIntMethod(tm, "getDataNetworkType", 13);
    }

    private static void pinStringMethod(Class<?> tm, String name, String value) {
        final String method = name;
        Legacy.safeHook(TAG, "TelephonyManager." + method, () -> {
            HookFramework.hookAllMethods(tm, method,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (result instanceof String) {
                                    String s = (String) result;
                                    if (s == null || s.isEmpty()
                                            || !s.equals(value)) {
                                        chain.replaceResult(value);
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": " + method
                                        + " failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void pinIntMethod(Class<?> tm, String name, int value) {
        final String method = name;
        Legacy.safeHook(TAG, "TelephonyManager." + method, () -> {
            HookFramework.hookAllMethods(tm, method,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (result instanceof Integer
                                        && (Integer) result != value) {
                                    chain.replaceResult(value);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": " + method
                                        + " failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookCellLocation(Class<?> tm) {
        Legacy.safeHook(TAG, "TelephonyManager.getPhoneType", () -> {
            HookFramework.hookAllMethods(tm, "getPhoneType",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                chain.replaceResult(1);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getPhoneType failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookSignalStrength(Class<?> tm, HookContext lpparam) {
        Legacy.safeHook(TAG, "TelephonyManager.getSignalStrength", () -> {
            HookFramework.hookAllMethods(tm, "getSignalStrength",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result == null) {
                                    Object synth = syntheticSignalStrength(
                                            lpparam.classLoader);
                                    if (synth != null) {
                                        chain.replaceResult(synth);
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getSignalStrength failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookExtraIdentity(Class<?> tm) {
        // Software version / NAI / voicemail / group-id leak build or
        // empty-SIM state; pin to neutral spoofed values.
        Legacy.safeHook(TAG, "TelephonyManager.getDeviceSoftwareVersion", () -> {
            HookFramework.hookAllMethods(tm, "getDeviceSoftwareVersion",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                String inc = ConfigManager
                                        .getBuildVersionIncremental();
                                if (inc != null) {
                                    chain.replaceResult(inc);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDeviceSoftwareVersion failed: "
                                        + t);
                            }
                        }
                    });
        });
        for (String name : new String[]{
                "getNai", "getVoiceMailNumber", "getVoiceMailAlphaTag",
                "getGroupIdLevel1"}) {
            final String method = name;
            Legacy.safeHook(TAG, "TelephonyManager." + method, () -> {
                HookFramework.hookAllMethods(tm, method,
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (result instanceof String) {
                                        String s = (String) result;
                                        if (s == null || s.isEmpty()) {
                                            chain.replaceResult("");
                                        }
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

    // SubscriptionManager default-sub queries ACE uses to resolve slot 0.
    // Pin every default-sub id to 1 / slot 0 so dual-SIM checks agree with
    // the synthetic single-entry list in TelephonyHooks.
    private static void hookSubscriptionDefaults(HookContext lpparam) {
        Class<?> sm = Legacy.findClassIfExists(
                "android.telephony.SubscriptionManager", lpparam.classLoader);
        if (sm == null) {
            return;
        }
        for (String name : new String[]{
                "getDefaultSubId", "getDefaultDataSubId",
                "getDefaultDataSubscriptionId", "getDefaultSmsSubId",
                "getDefaultSmsSubscriptionId", "getDefaultVoiceSubId",
                "getDefaultVoiceSubscriptionId", "getDefaultSubscriptionId",
                "getActiveDataSubId", "getActiveDataSubscriptionId"}) {
            final String method = name;
            Legacy.safeHook(TAG, "SubscriptionManager." + method, () -> {
                HookFramework.hookAllMethods(sm, method,
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    chain.replaceResult(1);
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": " + method
                                            + " failed: " + t);
                                }
                            }
                        });
            });
        }
        Legacy.safeHook(TAG, "SubscriptionManager.getSlotIndex", () -> {
            HookFramework.hookAllMethods(sm, "getSlotIndex",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof Integer
                                        && (Integer) result < 0) {
                                    chain.replaceResult(0);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getSlotIndex failed: " + t);
                            }
                        }
                    });
        });
        Legacy.safeHook(TAG, "SubscriptionManager.getSubId", () -> {
            HookFramework.hookAllMethods(sm, "getSubId",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof int[]) {
                                    int[] orig = (int[]) result;
                                    if (orig.length == 0) {
                                        chain.replaceResult(new int[]{1});
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getSubId failed: " + t);
                            }
                        }
                    });
        });
    }

    // android.telephony.SignalStrength.getLevel: HAL returns 0 on Waydroid's
    // SIM-less radio; report a healthy 4 bars. Constructed instances (above)
    // also flow through here, so both paths agree.
    private static void hookSignalStrengthClass(HookContext lpparam) {
        Class<?> ss = Legacy.findClassIfExists(
                "android.telephony.SignalStrength", lpparam.classLoader);
        if (ss == null) {
            return;
        }
        Legacy.safeHook(TAG, "SignalStrength.getLevel", () -> {
            HookFramework.hookAllMethods(ss, "getLevel",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof Integer
                                        && (Integer) result <= 0) {
                                    chain.replaceResult(4);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getLevel failed: " + t);
                            }
                        }
                    });
        });
    }

    private static Object syntheticSignalStrength(ClassLoader loader) {
        try {
            Class<?> ss = Legacy.findClassIfExists(
                    "android.telephony.SignalStrength", loader);
            if (ss == null) {
                return null;
            }
            for (java.lang.reflect.Constructor<?> c
                    : ss.getDeclaredConstructors()) {
                if (c.getParameterTypes().length == 0) {
                    try {
                        c.setAccessible(true);
                        return c.newInstance();
                    } catch (Throwable ignored) {
                        return null;
                    }
                }
            }
            return null;
        } catch (Throwable t) {
            Legacy.log(TAG + ": syntheticSignalStrength failed: " + t);
            return null;
        }
    }
}
