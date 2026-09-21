package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.utils.ConfigManager;

import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.net.NetworkInterface;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Enumeration;
import java.util.List;
import java.util.WeakHashMap;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

// Makes apps believe they are on WiFi + Cellular (LTE) instead of the
// Waydroid virtual Ethernet (eth0 / TRANSPORT_ETHERNET / type 9).
// Mirrors the waydroid_network_spoof reference (.devdocs): WiFi wins when
// both transports are present. Every hook is fail-closed: any reflection
// error leaves the original result untouched and logs once.
public class NetworkHooks {

    private static final String TAG = "DeviceSpoofLab-Network";
    private static final byte[] EMPTY_MAC = new byte[0];

    private static final int TRANSPORT_CELLULAR = 0;
    private static final int TRANSPORT_WIFI = 1;
    private static final int TRANSPORT_ETHERNET = 3;

    private static final int TYPE_WIFI = 1;
    private static final int NETWORK_TYPE_LTE = 13;
    private static final int DATA_CONNECTED = 2;
    private static final int WIFI_STATE_ENABLED = 3;

    private static final String DEFAULT_SSID = "GhostDroid-5G";

    // NetworkInterface.getName/getDisplayName are native (unhookable), so
    // getNetworkInterfaces()/getByName() record overrides here, mirroring
    // the reference module's WeakHashMap approach.
    private static final WeakHashMap<NetworkInterface, String> IFACE_OVERRIDES =
            new WeakHashMap<>();

    public static void hook(HookContext lpparam) {
        hookWifiInfo(lpparam);
        hookWifiManager(lpparam);
        hookBluetoothAdapter(lpparam);
        hookNetworkInterface();
        hookConnectivity(lpparam);
        hookNetworkCapabilities(lpparam);
        hookNetworkInfo(lpparam);
        hookLinkProperties(lpparam);
        hookTelephonyNet(lpparam);
        hookNetworkByName();
    }

    private static boolean fakeWifiEnabled() {
        try {
            String raw = ConfigManager.getRawProperty("fake_wifi");
            return raw == null || !"0".equals(raw.trim());
        } catch (Throwable t) {
            Legacy.log(TAG + ": fake_wifi read failed: " + t);
            return true;
        }
    }

    private static String effectiveSsid() {
        try {
            String raw = ConfigManager.getRawProperty("wifi.ssid");
            if (raw != null && !raw.trim().isEmpty()) {
                return raw.trim();
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": wifi.ssid read failed: " + t);
        }
        return DEFAULT_SSID;
    }

    private static void hookWifiInfo(HookContext lpparam) {
        Class<?> wifiInfo = Legacy.findClassIfExists(
                "android.net.wifi.WifiInfo", lpparam.classLoader);
        if (wifiInfo == null) return;

        pinWifiString(wifiInfo, "getMacAddress", ConfigManager.getWifiMacAddress());
        pinWifiString(wifiInfo, "getBSSID", ConfigManager.getWifiBssid());
        // getSSID / getRssi / getLinkSpeed / getFrequency / getNetworkId:
        // sibling scalars detectors read off the same WifiInfo object.
        // SSID keeps the quoted form; RSSI/link/freq pinned to match the
        // synthetic AP + spoofWifiInfo field injection. Fail-closed.
        Legacy.safeHook(TAG, "WifiInfo.getSSID", () -> {
            HookFramework.hookAllMethods(wifiInfo, "getSSID",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult("\"" + ConfigManager.getWifiSsid() + "\"");
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": WifiInfo.getSSID failed: " + t);
                            }
                        }
                    });
        });
        pinWifiInt(wifiInfo, "getRssi", -55);
        pinWifiInt(wifiInfo, "getLinkSpeed", 866);
        pinWifiInt(wifiInfo, "getFrequency", 5180);
        pinWifiInt(wifiInfo, "getNetworkId", 0);
    }

    private static void pinWifiInt(Class<?> wifiInfo, String name, int value) {
        final String method = name;
        Legacy.safeHook(TAG, "WifiInfo." + method, () -> {
            HookFramework.hookAllMethods(wifiInfo, method,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(value);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": WifiInfo." + method + " failed: " + t);
                            }
                        }
                    });
        });
    }

    // (SSID hook folded into the block above; pinWifiString shared helper
    // lives below.)

    // Single shared WifiInfo string pin: hookAllMethods covers every
    // overload so future (int)-suffixed forms cannot bypass. Fail-closed.
    private static void pinWifiString(Class<?> wifiInfo, String name, String value) {
        final String method = name;
        Legacy.safeHook(TAG, "WifiInfo." + method, () -> {
            HookFramework.hookAllMethods(wifiInfo, method,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (value != null) {
                                    chain.replaceResult(value);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": WifiInfo." + method + " failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookWifiManager(HookContext lpparam) {
        Class<?> wm = Legacy.findClassIfExists(
                "android.net.wifi.WifiManager", lpparam.classLoader);
        if (wm == null) return;

        // getScanResults: hookAllMethods covers all overloads; same
        // single-AP synthetic policy for each.
        Legacy.safeHook(TAG, "WifiManager.getScanResults", () -> {
            HookFramework.hookAllMethods(wm, "getScanResults",
                    new HookFramework.Hook() {@Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (!fakeWifiEnabled()) {
                                    chain.replaceResult(Collections.emptyList());
                                    return;
                                }
                                Object ap = syntheticAccessPoint(lpparam.classLoader);
                                if (ap != null) {
                                    List<Object> single = new ArrayList<>(1);
                                    single.add(ap);
                                    chain.replaceResult(single);
                                } else {
                                    chain.replaceResult(Collections.emptyList());
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getScanResults failed: " + t);
                            }
                        }
                    });
        });

        // isWifiEnabled: hookAllMethods covers all overloads (some
        // releases add a calling-package variant).
        Legacy.safeHook(TAG, "WifiManager.isWifiEnabled", () -> {
            HookFramework.hookAllMethods(wm, "isWifiEnabled",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult(true);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": isWifiEnabled failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "WifiManager.setWifiEnabled", () -> {
            HookFramework.hookAllMethods(wm, "setWifiEnabled",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult(true);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": setWifiEnabled failed: " + t);
                            }
                        }
                    });
        });

        // getWifiState: hookAllMethods covers all overloads.
        Legacy.safeHook(TAG, "WifiManager.getWifiState", () -> {
            HookFramework.hookAllMethods(wm, "getWifiState",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult(WIFI_STATE_ENABLED);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getWifiState failed: " + t);
                            }
                        }
                    });
        });

        // getConnectionInfo: hookAllMethods covers all overloads; field
        // injection on the delivered WifiInfo.
        Legacy.safeHook(TAG, "WifiManager.getConnectionInfo", () -> {
            HookFramework.hookAllMethods(wm, "getConnectionInfo",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (result != null) {
                                    spoofWifiInfo(result);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getConnectionInfo spoof failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "WifiManager.getConfiguredNetworks", () -> {
            HookFramework.hookAllMethods(wm, "getConfiguredNetworks",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                Object cfg = syntheticWifiConfiguration(lpparam.classLoader);
                                if (cfg != null) {
                                    List<Object> single = new ArrayList<>(1);
                                    single.add(cfg);
                                    chain.replaceResult(single);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getConfiguredNetworks failed: " + t);
                            }
                        }
                    });
        });

        // Some apps reach into WifiManager.getCurrentNetwork().getSSID() — those go
        // through WifiInfo, already covered.
    }

    private static void spoofWifiInfo(Object wifiInfo) {
        try {
            Legacy.setObjectField(wifiInfo, "mSSID", "\"" + effectiveSsid() + "\"");
        } catch (Throwable ignored) {
        }
        try {
            String bssid = ConfigManager.getWifiBssid();
            if (bssid != null) {
                Legacy.setObjectField(wifiInfo, "mBSSID", bssid);
            }
        } catch (Throwable ignored) {
        }
        try {
            String mac = ConfigManager.getWifiMacAddress();
            if (mac != null) {
                Legacy.setObjectField(wifiInfo, "mMacAddress", mac);
            }
        } catch (Throwable ignored) {
        }
        try {
            Legacy.setIntField(wifiInfo, "mNetworkId", 0);
            Legacy.setIntField(wifiInfo, "mRssi", -55);
            Legacy.setIntField(wifiInfo, "mLinkSpeed", 866);
            Legacy.setIntField(wifiInfo, "mFrequency", 5180);
        } catch (Throwable ignored) {
        }
    }

    private static Object syntheticAccessPoint(ClassLoader loader) {
        try {
            Class<?> sr = Legacy.findClassIfExists(
                    "android.net.wifi.ScanResult", loader);
            if (sr == null) {
                return null;
            }
            Object inst = newHiddenInstance(sr);
            if (inst == null) {
                return null;
            }
            Legacy.setObjectField(inst, "SSID", effectiveSsid());
            String bssid = ConfigManager.getWifiBssid();
            Legacy.setObjectField(inst, "BSSID",
                    bssid != null ? bssid : "02:00:00:00:00:00");
            Legacy.setObjectField(inst, "capabilities", "[WPA2-PSK-CCMP][ESS]");
            Legacy.setIntField(inst, "level", -55);
            Legacy.setIntField(inst, "frequency", 5180);
            try {
                Legacy.setLongField(inst, "timestamp", System.currentTimeMillis() * 1000L);
            } catch (Throwable ignored) {
            }
            return inst;
        } catch (Throwable t) {
            Legacy.log(TAG + ": synthetic AP build failed: " + t);
            return null;
        }
    }

    private static Object syntheticWifiConfiguration(ClassLoader loader) {
        try {
            Class<?> wc = Legacy.findClassIfExists(
                    "android.net.wifi.WifiConfiguration", loader);
            if (wc == null) {
                return null;
            }
            Object inst = newHiddenInstance(wc);
            if (inst == null) {
                return null;
            }
            Legacy.setObjectField(inst, "SSID", "\"" + effectiveSsid() + "\"");
            try {
                String bssid = ConfigManager.getWifiBssid();
                if (bssid != null) {
                    Legacy.setObjectField(inst, "BSSID", bssid);
                }
            } catch (Throwable ignored) {
            }
            try {
                Legacy.setIntField(inst, "networkId", 0);
                Legacy.setIntField(inst, "status", 2);
            } catch (Throwable ignored) {
            }
            return inst;
        } catch (Throwable t) {
            Legacy.log(TAG + ": synthetic WifiConfiguration build failed: " + t);
            return null;
        }
    }

    private static Object newHiddenInstance(Class<?> cls) {
        try {
            Constructor<?> noArg = cls.getDeclaredConstructor();
            noArg.setAccessible(true);
            return noArg.newInstance();
        } catch (Throwable ignored) {
        }
        Constructor<?> best = null;
        for (Constructor<?> c : cls.getDeclaredConstructors()) {
            if (best == null
                    || c.getParameterTypes().length < best.getParameterTypes().length) {
                best = c;
            }
        }
        if (best == null) {
            return null;
        }
        try {
            best.setAccessible(true);
            Class<?>[] types = best.getParameterTypes();
            Object[] args = new Object[types.length];
            for (int i = 0; i < types.length; i++) {
                args[i] = defaultFor(types[i]);
            }
            return best.newInstance(args);
        } catch (Throwable t) {
            Legacy.log(TAG + ": hidden instance of " + cls.getName()
                    + " failed: " + t);
            return null;
        }
    }

    private static Object defaultFor(Class<?> type) {
        if (type == boolean.class) {
            return false;
        }
        if (type == byte.class) {
            return (byte) 0;
        }
        if (type == char.class) {
            return (char) 0;
        }
        if (type == short.class) {
            return (short) 0;
        }
        if (type == int.class) {
            return 0;
        }
        if (type == long.class) {
            return 0L;
        }
        if (type == float.class) {
            return 0f;
        }
        if (type == double.class) {
            return 0d;
        }
        return null;
    }

    private static void hookBluetoothAdapter(HookContext lpparam) {
        Class<?> ba = Legacy.findClassIfExists(
                "android.bluetooth.BluetoothAdapter", lpparam.classLoader);
        if (ba == null) return;

        pinBtString(ba, "getAddress", true);
        pinBtString(ba, "getName", false);

        // Settings.Secure.bluetooth_address path — settings hook handles strings,
        // but BluetoothAdapter.getAddress hides the well-known reflection too.
    }

    // Single shared BluetoothAdapter string pin: hookAllMethods covers all
    // overloads. MAC upper-cased to match framework form; name verbatim.
    // Fail-closed: errors keep the original value.
    private static void pinBtString(Class<?> ba, String name, boolean isMac) {
        final String method = name;
        final boolean mac = isMac;
        Legacy.safeHook(TAG, "BluetoothAdapter." + method, () -> {
            HookFramework.hookAllMethods(ba, method,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (mac) {
                                    String addr = ConfigManager.getBluetoothMacAddress();
                                    if (addr != null) {
                                        chain.replaceResult(addr.toUpperCase(
                                                java.util.Locale.US));
                                    }
                                } else {
                                    chain.replaceResult(ConfigManager.getBluetoothName());
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": BluetoothAdapter." + method + " failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookNetworkInterface() {
        // getHardwareAddress: hookAllMethods covers all overloads.
        Legacy.safeHook(TAG, "NetworkInterface.getHardwareAddress", () -> {
            HookFramework.hookAllMethods(NetworkInterface.class, "getHardwareAddress",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                NetworkInterface ni = (NetworkInterface) chain.thisObject();
                                String name = (ni == null) ? null : ni.getName();
                                if (name == null) return;
                                // Loopback and dummy interfaces have no MAC; preserve null.
                                byte[] original = (byte[]) result;
                                if (original == null) return;

                                String mac;
                                if (name.startsWith("wlan")) {
                                    mac = ConfigManager.getWifiMacAddress();
                                } else if (name.startsWith("bt") || name.startsWith("bnep")) {
                                    mac = ConfigManager.getBluetoothMacAddress();
                                } else {
                                    // Other interfaces: zero them out rather than leak.
                                    chain.replaceResult(new byte[]{0, 0, 0, 0, 0, 0});
                                    return;
                                }
                                if (mac == null) return;
                                chain.replaceResult(macStringToBytes(mac));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getHardwareAddress failed: " + t);
                            }
                        }
                    });
        });

        // getNetworkInterfaces: hookAllMethods covers all overloads.
        Legacy.safeHook(TAG, "NetworkInterface.getNetworkInterfaces", () -> {
            HookFramework.hookAllMethods(NetworkInterface.class, "getNetworkInterfaces",
                    new HookFramework.Hook() {@Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                Enumeration<NetworkInterface> orig =
                                        (Enumeration<NetworkInterface>) result;
                                if (orig == null) return;

                                // Filter out interfaces named "rmnet*" / "ccmni*" / "p2p*"
                                // which leak modem/p2p details on emulators.
                                List<NetworkInterface> kept = new ArrayList<>();
                                while (orig.hasMoreElements()) {
                                    NetworkInterface ni = orig.nextElement();
                                    String n = (ni == null) ? "" : ni.getName();
                                    if (n == null) continue;
                                    if (n.startsWith("rmnet") || n.startsWith("ccmni")
                                            || n.startsWith("p2p") || n.startsWith("dummy")) {
                                        continue;
                                    }
                                    if (n.startsWith("eth")) {
                                        synchronized (IFACE_OVERRIDES) {
                                            IFACE_OVERRIDES.put(ni, "wlan0");
                                        }
                                    }
                                    kept.add(ni);
                                }
                                chain.replaceResult(Collections.enumeration(kept));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getNetworkInterfaces failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "NetworkInterface.getName", () -> {
            HookFramework.hookAllMethods(NetworkInterface.class, "getName",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                NetworkInterface ni = (NetworkInterface) chain.thisObject();
                                if (ni == null) return;
                                synchronized (IFACE_OVERRIDES) {
                                    String override = IFACE_OVERRIDES.get(ni);
                                    if (override != null) chain.replaceResult(override);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getName override failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "NetworkInterface.getDisplayName", () -> {
            HookFramework.hookAllMethods(NetworkInterface.class, "getDisplayName",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                NetworkInterface ni = (NetworkInterface) chain.thisObject();
                                if (ni == null) return;
                                synchronized (IFACE_OVERRIDES) {
                                    String override = IFACE_OVERRIDES.get(ni);
                                    if (override != null) chain.replaceResult(override);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDisplayName override failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookNetworkByName() {
        Legacy.safeHook(TAG, "NetworkInterface.getByName", () -> {
            HookFramework.hookAllMethods(NetworkInterface.class, "getByName",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                String requested = chain.arg(0, null);
                                if (result instanceof NetworkInterface) {
                                    NetworkInterface ni = (NetworkInterface) result;
                                    String actual = ni.getName();
                                    if (actual != null && actual.startsWith("eth")) {
                                        synchronized (IFACE_OVERRIDES) {
                                            IFACE_OVERRIDES.put(ni, "wlan0");
                                        }
                                    }
                                    return;
                                }
                                if (result == null && "wlan0".equals(requested)) {
                                    try {
                                        NetworkInterface eth =
                                                NetworkInterface.getByName("eth0");
                                        if (eth != null) {
                                            synchronized (IFACE_OVERRIDES) {
                                                IFACE_OVERRIDES.put(eth, "wlan0");
                                            }
                                            chain.replaceResult(eth);
                                        }
                                    } catch (Throwable ignored) {
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getByName override failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookConnectivity(HookContext lpparam) {
        Class<?> cm = Legacy.findClassIfExists(
                "android.net.ConnectivityManager", lpparam.classLoader);
        if (cm == null) {
            return;
        }

        Legacy.safeHook(TAG, "ConnectivityManager.getActiveNetworkInfo", () -> {
            HookFramework.hookAllMethods(cm, "getActiveNetworkInfo",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (result != null) {
                                    spoofNetworkInfo(result);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getActiveNetworkInfo spoof failed: " + t);
                            }
                        }
                    });
        });

        // Per-network and dump-all variants bypass getActiveNetworkInfo:
        // spoof each element so detectors enumerating networks still see
        // WIFI/LTE. getAllNetworks (Network[]) is intentionally untouched —
        // Network tokens cannot be faked safely; capabilities/link-props
        // per Network are already rewritten above.
        Legacy.safeHook(TAG, "ConnectivityManager.getNetworkInfo", () -> {
            HookFramework.hookAllMethods(cm, "getNetworkInfo",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (result != null) {
                                    spoofNetworkInfo(result);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getNetworkInfo spoof failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "ConnectivityManager.getAllNetworkInfo", () -> {
            HookFramework.hookAllMethods(cm, "getAllNetworkInfo",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (result instanceof Object[]) {
                                    for (Object info : (Object[]) result) {
                                        if (info != null) {
                                            spoofNetworkInfo(info);
                                        }
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getAllNetworkInfo spoof failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "ConnectivityManager.getNetworkCapabilities", () -> {
            HookFramework.hookAllMethods(cm, "getNetworkCapabilities",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (result != null) {
                                    setWifiCellularMask(result);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getNetworkCapabilities wrap failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "ConnectivityManager.getLinkProperties", () -> {
            HookFramework.hookAllMethods(cm, "getLinkProperties",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (result != null) {
                                    setWlanInterface(result);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getLinkProperties wrap failed: " + t);
                            }
                        }
                    });
        });

        HookFramework.BeforeHook wrapRequest = new HookFramework.BeforeHook() {
            @Override
            public void before(HookFramework.HookChain chain) {
                try {
                    for (int i = 0; i < chain.argCount(); i++) {
                        Object arg = chain.arg(i, null);
                        if (arg == null) {
                            continue;
                        }
                        if ("android.net.NetworkRequest".equals(arg.getClass().getName())) {
                            addWifiCellularToRequest(arg);
                        }
                    }
                } catch (Throwable t) {
                    Legacy.log(TAG + ": NetworkRequest wrap failed: " + t);
                }
            }
        };
        Legacy.safeHook(TAG, "ConnectivityManager.registerNetworkCallback", () -> {
            HookFramework.hookAllMethods(cm, "registerNetworkCallback", wrapRequest);
        });
        Legacy.safeHook(TAG, "ConnectivityManager.requestNetwork", () -> {
            HookFramework.hookAllMethods(cm, "requestNetwork", wrapRequest);
        });
        Legacy.safeHook(TAG, "ConnectivityManager.registerDefaultNetworkCallback", () -> {
            HookFramework.hookAllMethods(cm, "registerDefaultNetworkCallback", wrapRequest);
        });
    }

    private static void addWifiCellularToRequest(Object request) {
        try {
            Object caps = Legacy.getObjectField(request, "networkCapabilities");
            if (caps != null) {
                setWifiCellularMask(caps);
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": addWifiCellularToRequest failed: " + t);
        }
    }

    // WIFI bit (1<<1) + CELLULAR bit (1<<0); ETHERNET bit (1<<3) cleared.
    private static void setWifiCellularMask(Object networkCapabilities) {
        long mask = (1L << TRANSPORT_CELLULAR) | (1L << TRANSPORT_WIFI);
        try {
            Legacy.setLongField(networkCapabilities, "mTransportTypes", mask);
        } catch (Throwable ignored) {
        }
        try {
            Legacy.setIntField(networkCapabilities, "mTransportTypes", (int) mask);
        } catch (Throwable ignored) {
        }
    }

    private static void spoofNetworkInfo(Object info) {
        try {
            Legacy.setIntField(info, "mType", TYPE_WIFI);
            Legacy.setObjectField(info, "mTypeName", "WIFI");
            Legacy.setIntField(info, "mSubtype", NETWORK_TYPE_LTE);
            Legacy.setObjectField(info, "mSubtypeName", "LTE");
        } catch (Throwable ignored) {
        }
        try {
            Class<?> stateClass = Class.forName("android.net.NetworkInfo$State",
                    false, info.getClass().getClassLoader());
            Class<?> detailedClass = Class.forName("android.net.NetworkInfo$DetailedState",
                    false, info.getClass().getClassLoader());
            Object connected = enumValue(stateClass, "CONNECTED");
            Object detailedConnected = enumValue(detailedClass, "CONNECTED");
            if (connected != null) {
                Legacy.setObjectField(info, "mState", connected);
            }
            if (detailedConnected != null) {
                Legacy.setObjectField(info, "mDetailedState", detailedConnected);
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": NetworkInfo state spoof failed: " + t);
        }
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static Object enumValue(Class<?> enumClass, String name) {
        try {
            return Enum.valueOf((Class<Enum>) enumClass, name);
        } catch (Throwable ignored) {
            return null;
        }
    }

    private static void setWlanInterface(Object linkProperties) {
        try {
            Object current = Legacy.callMethod(linkProperties, "getInterfaceName");
            if (current instanceof String) {
                String name = (String) current;
                if (name.startsWith("eth")) {
                    Legacy.callMethod(linkProperties, "setInterfaceName", "wlan0");
                }
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": LinkProperties iface rewrite failed: " + t);
        }
    }

    private static void hookNetworkCapabilities(HookContext lpparam) {
        Class<?> caps = Legacy.findClassIfExists(
                "android.net.NetworkCapabilities", lpparam.classLoader);
        if (caps == null) {
            return;
        }
        // NetworkCapabilities.hasTransport has (int) and (int,int)
        // overloads on newer releases: hookAllMethods covers both.
        Legacy.safeHook(TAG, "NetworkCapabilities.hasTransport", () -> {
            HookFramework.hookAllMethods(caps, "hasTransport",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                int transport = chain.arg(0, -1);
                                if (transport == TRANSPORT_WIFI
                                        || transport == TRANSPORT_CELLULAR) {
                                    chain.replaceResult(true);
                                } else if (transport == TRANSPORT_ETHERNET) {
                                    chain.replaceResult(false);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": hasTransport failed: " + t);
                            }
                        }
                    });
        });
    }

    // NetworkInfo getters: single no-arg overloads each, but installed
    // via hookAllMethods so signature drift cannot leave a gap.
    // isConnectedOrConnecting / isAvailable / isRoaming / isFailover
    // complete the boolean state shape ACE reads.
    // Fail-closed: errors keep the original value.
    private static void hookNetworkInfo(HookContext lpparam) {
        Class<?> ni = Legacy.findClassIfExists(
                "android.net.NetworkInfo", lpparam.classLoader);
        if (ni == null) {
            return;
        }

        pinNetworkInt(ni, "getType", TYPE_WIFI);
        pinNetworkString(ni, "getTypeName", "WIFI");
        pinNetworkInt(ni, "getSubtype", NETWORK_TYPE_LTE);
        pinNetworkString(ni, "getSubtypeName", "LTE");
        pinNetworkBoolean(ni, "isConnected", true);
        pinNetworkBoolean(ni, "isConnectedOrConnecting", true);
        pinNetworkBoolean(ni, "isAvailable", true);
        pinNetworkBoolean(ni, "isRoaming", false);
        pinNetworkBoolean(ni, "isFailover", false);
    }

    private static void pinNetworkInt(Class<?> ni, String name, int value) {
        final String method = name;
        Legacy.safeHook(TAG, "NetworkInfo." + method, () -> {
            HookFramework.hookAllMethods(ni, method,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(value);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": NetworkInfo." + method + " failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void pinNetworkString(Class<?> ni, String name, String value) {
        final String method = name;
        Legacy.safeHook(TAG, "NetworkInfo." + method, () -> {
            HookFramework.hookAllMethods(ni, method,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(value);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": NetworkInfo." + method + " failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void pinNetworkBoolean(Class<?> ni, String name, boolean value) {
        final String method = name;
        Legacy.safeHook(TAG, "NetworkInfo." + method, () -> {
            HookFramework.hookAllMethods(ni, method,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(value);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": NetworkInfo." + method + " failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookLinkProperties(HookContext lpparam) {
        Class<?> lp = Legacy.findClassIfExists(
                "android.net.LinkProperties", lpparam.classLoader);
        if (lp == null) {
            return;
        }
        Legacy.safeHook(TAG, "LinkProperties.getInterfaceName", () -> {
            HookFramework.hookAllMethods(lp, "getInterfaceName",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (result instanceof String
                                        && ((String) result).startsWith("eth")) {
                                    chain.replaceResult("wlan0");
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": LinkProperties.getInterfaceName failed: " + t);
                            }
                        }
                    });
        });
        // getLinkAddresses / getDnsServers / getRoutes / getDomains /
        // getDhcpServerAddress: host LAN values leak through these lists.
        // Rewrite eth-anchored entries to the phone-plausible 192.168.1.x
        // LAN; fail-closed (errors keep the original list).
        rewriteLinkAddressList(lp, "getLinkAddresses", "192.168.1.50", 24);
        rewriteLinkAddressList(lp, "getDnsServers", "8.8.8.8", -1);
        pinLinkString(lp, "getDomains", "lan");
        pinLinkString(lp, "getDhcpServerAddress", "192.168.1.1");
    }

    // Rewrites an InetAddress-returning LinkProperties getter: replaces any
    // host-leak entry with the spoofed literal. Fail-closed.
    private static void pinLinkString(Class<?> lp, String name, String literal) {
        final String method = name;
        Legacy.safeHook(TAG, "LinkProperties." + method, () -> {
            HookFramework.hookAllMethods(lp, method,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null || result == null) {
                                    return;
                                }
                                Object spoofed = Legacy.callStaticMethod(
                                        java.net.InetAddress.class,
                                        "getByName", literal);
                                if (spoofed != null) {
                                    chain.replaceResult(spoofed);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": LinkProperties." + method
                                        + " failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void rewriteLinkAddressList(Class<?> lp, String name,
            String literal, int prefix) {
        final String method = name;
        Legacy.safeHook(TAG, "LinkProperties." + method, () -> {
            HookFramework.hookAllMethods(lp, method,
                    new HookFramework.Hook() {
                        @Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null
                                        || !(result instanceof java.util.List)) {
                                    return;
                                }
                                java.util.List<?> orig =
                                        (java.util.List<?>) result;
                                if (orig.isEmpty()) {
                                    return;
                                }
                                Object spoofedAddr = Legacy.callStaticMethod(
                                        java.net.InetAddress.class,
                                        "getByName", literal);
                                if (spoofedAddr == null) {
                                    return;
                                }
                                java.util.List<Object> out =
                                        new java.util.ArrayList<>(orig.size());
                                for (Object entry : orig) {
                                    if (entry instanceof java.net.InetAddress) {
                                        out.add(spoofAddrFor(entry,
                                                (java.net.InetAddress) spoofedAddr,
                                                prefix));
                                    } else {
                                        out.add(entry);
                                    }
                                }
                                chain.replaceResult(out);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": LinkProperties." + method
                                        + " failed: " + t);
                            }
                        }
                    });
        });
    }

    // Preserves the entry shape: LinkAddress stays a LinkAddress (with the
    // spoofed prefix), plain InetAddress entries become the spoofed addr.
    private static Object spoofAddrFor(Object entry,
            java.net.InetAddress spoofed, int prefix) {
        try {
            if (prefix >= 0
                    && "android.net.LinkAddress".equals(
                            entry.getClass().getName())) {
                java.lang.reflect.Constructor<?> c =
                        entry.getClass().getDeclaredConstructor(
                                java.net.InetAddress.class, int.class);
                c.setAccessible(true);
                return c.newInstance(spoofed, prefix);
            }
        } catch (Throwable ignored) {
        }
        return spoofed;
    }

    private static void hookTelephonyNet(HookContext lpparam) {
        Class<?> tm = Legacy.findClassIfExists(
                "android.telephony.TelephonyManager", lpparam.classLoader);
        if (tm == null) {
            return;
        }

        HookFramework.Hook lteHook = new HookFramework.Hook() {
            @Override
            public void after(HookFramework.HookChain chain,
                    Object result, Throwable error) {
                try {
                    chain.replaceResult(NETWORK_TYPE_LTE);
                } catch (Throwable t) {
                    Legacy.log(TAG + ": TelephonyManager network type failed: " + t);
                }
            }
        };
        Legacy.safeHook(TAG, "TelephonyManager.getNetworkType", () -> {
            HookFramework.hookAllMethods(tm, "getNetworkType", lteHook);
        });
        Legacy.safeHook(TAG, "TelephonyManager.getDataNetworkType", () -> {
            HookFramework.hookAllMethods(tm, "getDataNetworkType", lteHook);
        });
        Legacy.safeHook(TAG, "TelephonyManager.getDataState", () -> {
            HookFramework.hookAllMethods(tm, "getDataState",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult(DATA_CONNECTED);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": TelephonyManager.getDataState failed: " + t);
                            }
                        }
                    });
        });
    }

    private static byte[] macStringToBytes(String mac) {
        if (mac == null) return EMPTY_MAC;
        String[] parts = mac.split(":");
        if (parts.length != 6) return EMPTY_MAC;
        byte[] out = new byte[6];
        try {
            for (int i = 0; i < 6; i++) {
                out[i] = (byte) Integer.parseInt(parts[i], 16);
            }
        } catch (NumberFormatException e) {
            return EMPTY_MAC;
        }
        return out;
    }
}
