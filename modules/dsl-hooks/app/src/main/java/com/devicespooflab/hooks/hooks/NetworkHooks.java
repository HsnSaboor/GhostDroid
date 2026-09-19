package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.utils.ConfigManager;

import java.lang.reflect.Method;
import java.net.NetworkInterface;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Enumeration;
import java.util.List;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class NetworkHooks {

    private static final String TAG = "DeviceSpoofLab-Network";
    private static final byte[] EMPTY_MAC = new byte[0];

    public static void hook(HookContext lpparam) {
        hookWifiInfo(lpparam);
        hookWifiManager(lpparam);
        hookBluetoothAdapter(lpparam);
        hookNetworkInterface();
    }

    private static void hookWifiInfo(HookContext lpparam) {
        Class<?> wifiInfo = Legacy.findClassIfExists(
                "android.net.wifi.WifiInfo", lpparam.classLoader);
        if (wifiInfo == null) return;

        Legacy.safeHook(TAG, "WifiInfo.getMacAddress", () -> {
            Legacy.findAndHookMethod(wifiInfo, "getMacAddress",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getWifiMacAddress();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        });

        Legacy.safeHook(TAG, "WifiInfo.getBSSID", () -> {
            Legacy.findAndHookMethod(wifiInfo, "getBSSID",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getWifiBssid();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        });

        Legacy.safeHook(TAG, "WifiInfo.getSSID", () -> {
            Legacy.findAndHookMethod(wifiInfo, "getSSID",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult("\"" + ConfigManager.getWifiSsid() + "\"");
                        }
                    });
        });
    }

    private static void hookWifiManager(HookContext lpparam) {
        Class<?> wm = Legacy.findClassIfExists(
                "android.net.wifi.WifiManager", lpparam.classLoader);
        if (wm == null) return;

        Legacy.safeHook(TAG, "WifiManager.getScanResults", () -> {
            Legacy.findAndHookMethod(wm, "getScanResults",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            // Scan-result MAC addresses are equally fingerprintable;
                            // empty list is the safest spoof.
                            chain.replaceResult(Collections.emptyList());
                        }
                    });
        });

        // Some apps reach into WifiManager.getCurrentNetwork().getSSID() — those go
        // through WifiInfo, already covered.
    }

    private static void hookBluetoothAdapter(HookContext lpparam) {
        Class<?> ba = Legacy.findClassIfExists(
                "android.bluetooth.BluetoothAdapter", lpparam.classLoader);
        if (ba == null) return;

        Legacy.safeHook(TAG, "BluetoothAdapter.getAddress", () -> {
            Legacy.findAndHookMethod(ba, "getAddress",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String mac = ConfigManager.getBluetoothMacAddress();
                            if (mac != null) chain.replaceResult(mac.toUpperCase());
                        }
                    });
        });

        Legacy.safeHook(TAG, "BluetoothAdapter.getName", () -> {
            Legacy.findAndHookMethod(ba, "getName",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(ConfigManager.getBluetoothName());
                        }
                    });
        });

        // Settings.Secure.bluetooth_address path — settings hook handles strings,
        // but BluetoothAdapter.getAddress hides the well-known reflection too.
    }

    private static void hookNetworkInterface() {
        Legacy.safeHook(TAG, "NetworkInterface.getHardwareAddress", () -> {
            Legacy.findAndHookMethod(NetworkInterface.class, "getHardwareAddress",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
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
                        }
                    });
        });

        Legacy.safeHook(TAG, "NetworkInterface.getNetworkInterfaces", () -> {
            Legacy.findAndHookMethod(NetworkInterface.class, "getNetworkInterfaces",
                    new HookFramework.Hook() {@Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
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
                                kept.add(ni);
                            }
                            chain.replaceResult(Collections.enumeration(kept));
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
