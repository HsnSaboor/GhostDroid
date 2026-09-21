package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.bridge.HookContext;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.utils.ConfigManager;

// WiFi / Bluetooth state siblings NetworkHooks misses:
// WifiManager.getWifiApState (host AP off), getVerboseLoggingLevel,
// getCountryCode (US to match 310260), startScan (claim success),
// BluetoothAdapter.isEnabled/isDiscovering/getState/getScanMode/
// getBondedDevices (empty set: no host-paired devices), and
// BluetoothDevice.getName/getAddress/getType/getBondState per-device
// pinning. NFC NfcAdapter.isEnabled passthrough guard included.
// Fail-closed throughout.
public final class WirelessHooks {

    private static final String TAG = "DeviceSpoofLab-Wireless";

    private WirelessHooks() {}

    public static void hook(HookContext lpparam) {
        hookWifiExtras(lpparam);
        hookBluetoothState(lpparam);
        hookBluetoothDevice(lpparam);
        hookNfc(lpparam);
    }

    private static void hookWifiExtras(HookContext lpparam) {
        Class<?> wm = Legacy.findClassIfExists(
                "android.net.wifi.WifiManager", lpparam.classLoader);
        if (wm == null) {
            return;
        }
        // WIFI_AP_STATE_DISABLED = 11.
        pinInt(wm, "getWifiApState", 11);
        pinInt(wm, "getVerboseLoggingLevel", 0);
        hookDhcpInfo(wm, lpparam);
        hookWifiFeatureExtras(wm);
        Legacy.safeHook(TAG, "WifiManager.getCountryCode", () -> {
            HookFramework.hookAllMethods(wm, "getCountryCode",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                String cc = ConfigManager.getLocaleCountry();
                                if (cc != null && !cc.isEmpty()) {
                                    chain.replaceResult(cc.toUpperCase(
                                            java.util.Locale.US));
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getCountryCode failed: " + t);
                            }
                        }
                    });
        });
        Legacy.safeHook(TAG, "WifiManager.startScan", () -> {
            HookFramework.hookAllMethods(wm, "startScan",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof Boolean
                                        && !(Boolean) result) {
                                    chain.replaceResult(true);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": startScan failed: " + t);
                            }
                        }
                    });
        });
    }

    // WifiManager siblings ACE DEX calls without native:
    // isScanAlwaysAvailable / is5GHzBandSupported / is6GHzBandSupported /
    // is60GHzBandSupported / isWpa3SaeSupported / isWpa3SuiteBSupported /
    // isEnhancedOpenSupported / isStaApConcurrencySupported / isTdlsSupported.
    // A 2025 flagship supports all of these. Fail-closed.
    private static void hookWifiFeatureExtras(Class<?> wm) {
        pinBoolean(wm, "isScanAlwaysAvailable", true);
        pinBoolean(wm, "is5GHzBandSupported", true);
        pinBoolean(wm, "is6GHzBandSupported", true);
        pinBoolean(wm, "is60GHzBandSupported", true);
        pinBoolean(wm, "isWpa3SaeSupported", true);
        pinBoolean(wm, "isWpa3SuiteBSupported", true);
        pinBoolean(wm, "isEnhancedOpenSupported", true);
        pinBoolean(wm, "isStaApConcurrencySupported", true);
        pinBoolean(wm, "isTdlsSupported", true);
        // API 29+ randomized-MAC sibling: Waydroid reports the real eth0
        // MAC; pin to the spoofed WiFi MAC so scans cannot disagree with
        // WifiInfo.getMacAddress. Fail-closed.
        Legacy.safeHook(TAG, "WifiManager.getFactoryMacAddresses", () -> {
            HookFramework.hookAllMethods(wm, "getFactoryMacAddresses",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                String mac = ConfigManager.getWifiMacAddress();
                                if (mac != null && !mac.isEmpty()) {
                                    chain.replaceResult(new String[]{mac});
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getFactoryMacAddresses failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static void hookBluetoothState(HookContext lpparam) {
        Class<?> ba = Legacy.findClassIfExists(
                "android.bluetooth.BluetoothAdapter", lpparam.classLoader);
        if (ba == null) {
            return;
        }
        pinBoolean(ba, "isEnabled", true);
        pinBoolean(ba, "isDiscovering", false);
        // STATE_ON = 12.
        pinInt(ba, "getState", 12);
        // SCAN_MODE_NONE = 20.
        pinInt(ba, "getScanMode", 20);
        // Bluetooth LE / dual-mode / offload tells on a flagship phone.
        pinBoolean(ba, "isLeEnabled", true);
        pinBoolean(ba, "isMultipleAdvertisementSupported", true);
        pinBoolean(ba, "isOffloadedFilteringSupported", true);
        pinBoolean(ba, "isOffloadedScanBatchingSupported", true);
        pinInt(ba, "getLeState", 12);
        pinInt(ba, "getLeAccess", 0);
        // No host-paired devices leak through getBondedDevices.
        Legacy.safeHook(TAG, "BluetoothAdapter.getBondedDevices", () -> {
            HookFramework.hookAllMethods(ba, "getBondedDevices",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof java.util.Set
                                        && !((java.util.Set<?>) result)
                                                .isEmpty()) {
                                    chain.replaceResult(
                                            java.util.Collections.emptySet());
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getBondedDevices failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static void hookBluetoothDevice(HookContext lpparam) {
        Class<?> bd = Legacy.findClassIfExists(
                "android.bluetooth.BluetoothDevice", lpparam.classLoader);
        if (bd == null) {
            return;
        }
        Legacy.safeHook(TAG, "BluetoothDevice.getName", () -> {
            HookFramework.hookAllMethods(bd, "getName",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                chain.replaceResult(
                                        ConfigManager.getBluetoothName());
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": BTDevice.getName failed: "
                                        + t);
                            }
                        }
                    });
        });
        Legacy.safeHook(TAG, "BluetoothDevice.getAddress", () -> {
            HookFramework.hookAllMethods(bd, "getAddress",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                String mac = ConfigManager
                                        .getBluetoothMacAddress();
                                if (mac != null) {
                                    chain.replaceResult(mac.toUpperCase(
                                            java.util.Locale.US));
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": BTDevice.getAddress failed: "
                                        + t);
                            }
                        }
                    });
        });
        // DEVICE_TYPE_CLASSIC = 1; BOND_NONE = 10.
        pinInt(bd, "getType", 1);
        pinInt(bd, "getBondState", 10);
        // BluetoothClass.getDeviceClass service bits: Waydroid exposes a
        // computer-class (laptop) device class that DeviceInfoHW BLUETOOTH
        // tab reads; a phone reports PHONE_SMART (536). Fail-closed.
        Legacy.safeHook(TAG, "BluetoothClass.getDeviceClass", () -> {
            Class<?> bc;
            try {
                bc = Class.forName("android.bluetooth.BluetoothClass");
            } catch (Throwable ignored) {
                return;
            }
            HookFramework.hookAllMethods(bc, "getDeviceClass",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null
                                        || !(result instanceof Integer)) {
                                    return;
                                }
                                if ((Integer) result != 536) {
                                    chain.replaceResult(536);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": BTClass.getDeviceClass failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static void hookNfc(HookContext lpparam) {
        Class<?> nfc = Legacy.findClassIfExists(
                "android.nfc.NfcAdapter", lpparam.classLoader);
        if (nfc == null) {
            return;
        }
        // Real S26 Ultra has NFC; Waydroid reports no adapter (null from
        // getDefaultAdapter). Only guard isEnabled when an adapter exists —
        // never fabricate one.
        pinBoolean(nfc, "isEnabled", true);
    }

    private static void hookDhcpInfo(Class<?> wm, HookContext lpparam) {
        // getDhcpInfo(): Waydroid eth0 DHCP leaks host gateway/DNS. Rewrite
        // to a phone-plausible 192.168.1.x LAN via field injection on the
        // returned DhcpInfo (public int fields, no ctor needed). Android IP
        // ints are little-endian: 192.168.1.50 = 0x3201A8C0,
        // 192.168.1.1 = 0x0101A8C0. Never derive one address from another
        // with integer arithmetic (octet carries break); use literals.
        Legacy.safeHook(TAG, "WifiManager.getDhcpInfo", () -> {
            Legacy.findAndHookMethod(wm, "getDhcpInfo",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result == null) {
                                    return;
                                }
                                Legacy.setIntField(result, "ipAddress",
                                        0x3201A8C0);
                                Legacy.setIntField(result, "gateway",
                                        0x0101A8C0);
                                Legacy.setIntField(result, "netmask",
                                        0x00FFFFFF);
                                Legacy.setIntField(result, "dns1",
                                        0x08080808);
                                Legacy.setIntField(result, "dns2",
                                        0x08080404);
                                Legacy.setIntField(result, "serverAddress",
                                        0x0101A8C0);
                                Legacy.setIntField(result, "leaseDuration",
                                        86400);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDhcpInfo failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void pinInt(Class<?> cls, String name, int value) {
        Legacy.safeHook(TAG, cls.getSimpleName() + "." + name, () -> {
            HookFramework.hookAllMethods(cls, name,
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
                                Legacy.log(TAG + ": " + name + " failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void pinBoolean(Class<?> cls, String name, boolean value) {
        Legacy.safeHook(TAG, cls.getSimpleName() + "." + name, () -> {
            HookFramework.hookAllMethods(cls, name,
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
                                Legacy.log(TAG + ": " + name + " failed: " + t);
                            }
                        }
                    });
        });
    }
}
