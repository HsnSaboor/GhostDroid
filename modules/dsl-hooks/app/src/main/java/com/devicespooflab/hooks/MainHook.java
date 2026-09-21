package com.devicespooflab.hooks;

import android.os.Build;
import android.util.Log;

import com.devicespooflab.hooks.hooks.AccountHooks;
import com.devicespooflab.hooks.hooks.AdvertisingIdHooks;
import com.devicespooflab.hooks.hooks.ApplistHooks;
import com.devicespooflab.hooks.hooks.AppSetIdHooks;
import com.devicespooflab.hooks.hooks.BatteryHooks;
import com.devicespooflab.hooks.hooks.BatteryIntentHooks;
import com.devicespooflab.hooks.hooks.BuildHooks;
import com.devicespooflab.hooks.hooks.CameraHooks;
import com.devicespooflab.hooks.hooks.DeviceStateHooks;
import com.devicespooflab.hooks.hooks.DisplayStateHooks;
import com.devicespooflab.hooks.hooks.EuiccHooks;
import com.devicespooflab.hooks.hooks.GpuHooks;
import com.devicespooflab.hooks.hooks.HardwareHooks;
import com.devicespooflab.hooks.hooks.InputDeviceHooks;
import com.devicespooflab.hooks.hooks.LocaleHooks;
import com.devicespooflab.hooks.hooks.MediaDrmHooks;
import com.devicespooflab.hooks.hooks.NetworkHooks;
import com.devicespooflab.hooks.hooks.PackageInfoHooks;
import com.devicespooflab.hooks.hooks.PackageManagerHooks;
import com.devicespooflab.hooks.hooks.PackageQueryHooks;
import com.devicespooflab.hooks.hooks.PowerHooks;
import com.devicespooflab.hooks.hooks.ProcessHooks;
import com.devicespooflab.hooks.hooks.SensorHooks;
import com.devicespooflab.hooks.hooks.SettingsHooks;
import com.devicespooflab.hooks.hooks.StorageHooks;
import com.devicespooflab.hooks.hooks.SysfsListHooks;
import com.devicespooflab.hooks.hooks.SystemPropertiesHooks;
import com.devicespooflab.hooks.hooks.TelephonyCellHooks;
import com.devicespooflab.hooks.hooks.TelephonyHooks;
import com.devicespooflab.hooks.hooks.WebViewHooks;
import com.devicespooflab.hooks.hooks.WirelessHooks;
import com.devicespooflab.hooks.utils.ConfigManager;
import com.devicespooflab.hooks.utils.XposedServiceBridge;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class MainHook {

    private static final String TAG = "DeviceSpoofLab";

    public void handleLoadPackage(HookContext lpparam) {
        try {
            ConfigManager.init();
        } catch (Exception e) {
            Log.e(TAG, "Failed to init config: " + e.getMessage(), e);
            Legacy.log(TAG + ": Failed to init config: " + e.getMessage());
            return;
        }

        boolean verbose = ConfigManager.isVerboseLoggingEnabled();
        Log.i(TAG, "handleLoadPackage start pkg=" + lpparam.packageName
                + " verbose=" + verbose
                + " imei=" + ConfigManager.getIdentifierValue("imei")
                + " gaid=" + ConfigManager.getIdentifierValue("gaid"));
        logInfo(verbose, TAG + ": Loading hooks for " + lpparam.packageName);
        final int realDeviceSdk = Build.VERSION.SDK_INT;

        // XposedServiceHelper has independent static state in the app loader vs
        // the module loader. Defer init until Application.attach so the bridge
        // can bind through the app loader and see the same mListener that
        // XposedProvider notifies on SEND_BINDER.
        final boolean isOwnPackage = ConfigManager.isOwnPackageProcess(lpparam.processName)
                || "com.devicespooflab.hooks".equals(lpparam.packageName);

        final Runnable onBinderReady = new Runnable() {
            @Override
            public void run() {
                if (isOwnPackage) {
                    // Fallback only: with the module out of its own scope this
                    // branch never runs (MainActivity publishes directly). It
                    // survives for users who re-scope the module to itself, e.g.
                    // on stock LSPosed. The new-API RemotePreferences is
                    // read-only, so publishing needs the writable SEND_BINDER.
                    if (!XposedServiceBridge.isServiceWritable()) return;
                    ConfigManager.publishToRemotePreferences();
                } else {
                    boolean loaded = ConfigManager.loadFromRemotePreferences();
                    if (loaded) {
                        BuildHooks.refreshStaticFields(lpparam.classLoader);
                    }
                    // Vector's read-only RemotePreferences caches a frozen
                    // snapshot at construction and never fires the change
                    // listener for daemon writes, so we poll _generation.
                    installRemoteRefreshLoop(lpparam.classLoader);
                    Log.i(TAG, "RemotePreferences load=" + loaded
                            + " imei=" + ConfigManager.getIdentifierValue("imei")
                            + " gaid=" + ConfigManager.getIdentifierValue("gaid"));
                }
            }
        };

        try {
            Legacy.findAndHookMethod("android.app.Application",
                    lpparam.classLoader, "attach",
                    android.content.Context.class, new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                android.content.Context ctx =
                                        (android.content.Context) chain.arg(0, null);
                                XposedServiceBridge.init(ctx, onBinderReady);
                                // The own UI process is no longer in scope, so it
                                // never reaches here; it publishes directly from
                                // MainActivity via the binder delivered to
                                // XposedProvider. This path now only serves target
                                // apps: seed from RemotePreferences when the new-API
                                // channel is up but config is still empty.
                                if (!isOwnPackage
                                        && ConfigManager.getIdentifierValue("imei").isEmpty()
                                        && XposedServiceBridge.isServiceAvailable()) {
                                    if (ConfigManager.loadFromRemotePreferences()) {
                                        BuildHooks.refreshStaticFields(lpparam.classLoader);
                                    }
                                }
                                Log.i(TAG, "After Application.attach pkg=" + lpparam.packageName
                                        + " imei=" + ConfigManager.getIdentifierValue("imei")
                                        + " gaid=" + ConfigManager.getIdentifierValue("gaid"));
                            } catch (Throwable t) {
                                Log.w(TAG, "Application.attach hook failed: "
                                        + t.getMessage());
                            }
                        }
                    });
        } catch (Throwable t) {
            Log.w(TAG, "Failed to hook Application.attach: " + t.getMessage());
        }

        // Table-driven dispatch: {name, hook, minSdk, skipInOwnProcess}.
        // Replaces ~20 repetitive try/catch + logInfo blocks.
        installAll(lpparam, verbose, realDeviceSdk, isOwnPackage);

        logInfo(verbose, TAG + ": All hooks initialized for " + lpparam.packageName);
    }

    interface SubHook {
        void apply(HookContext ctx) throws Throwable;
    }

    static final class Entry {
        final String name;
        final SubHook hook;
        final int minSdk;
        final boolean skipInOwnProcess;

        Entry(String name, SubHook hook, int minSdk, boolean skipInOwnProcess) {
            this.name = name;
            this.hook = hook;
            this.minSdk = minSdk;
            this.skipInOwnProcess = skipInOwnProcess;
        }
    }

    private static void installAll(HookContext ctx, boolean verbose,
            int realDeviceSdk, boolean isOwnPackage) {
        Entry[] entries = {
            // BuildHooks first so direct Build.* reads pick up spoofed values.
            new Entry("BuildHooks", BuildHooks::hook, 0, false),
            new Entry("SystemPropertiesHooks", SystemPropertiesHooks::hook, 0, true),
            new Entry("HardwareHooks", HardwareHooks::hook, 0, false),
            new Entry("TelephonyHooks", TelephonyHooks::hook, 0, false),
            new Entry("TelephonyCellHooks", TelephonyCellHooks::hook, 0, false),
            new Entry("ProcessHooks", ProcessHooks::hook, 0, false),
            new Entry("PowerHooks", PowerHooks::hook, 0, false),
            new Entry("DeviceStateHooks", DeviceStateHooks::hook, 0, false),
            new Entry("SettingsHooks", SettingsHooks::hook, 0, false),
            new Entry("AdvertisingIdHooks", AdvertisingIdHooks::hook, 0, false),
            new Entry("AppSetIdHooks", c -> AppSetIdHooks.hook(c, realDeviceSdk), 30, false),
            new Entry("MediaDrmHooks", MediaDrmHooks::hook, 0, false),
            new Entry("WebViewHooks", WebViewHooks::hook, 0, false),
            new Entry("PackageManagerHooks", PackageManagerHooks::hook, 0, false),
            new Entry("NetworkHooks", NetworkHooks::hook, 0, false),
            new Entry("SensorHooks", SensorHooks::hook, 0, false),
            new Entry("GpuHooks", GpuHooks::hook, 0, false),
            // File.listFiles/list sysfs enumeration (USB/DRIVERS tabs): Java
            // opendir/getdents bypasses libc open hooks, so mask listings here
            // (single Qualcomm node). PCI sysfs dirs stay real for minigbm.
            new Entry("SysfsListHooks", SysfsListHooks::hook, 0, false),
            new Entry("CameraHooks", CameraHooks::hook, 0, false),
            new Entry("StorageHooks", StorageHooks::hook, 0, false),
            new Entry("AccountHooks", AccountHooks::hook, 0, false),
            new Entry("LocaleHooks", LocaleHooks::hook, 0, true),
            new Entry("PackageInfoHooks", PackageInfoHooks::hook, 0, false),
            new Entry("PackageQueryHooks", PackageQueryHooks::hook, 0, false),
            new Entry("ApplistHooks", ApplistHooks::hook, 0, false),
            new Entry("WirelessHooks", WirelessHooks::hook, 0, false),
            new Entry("DisplayStateHooks", DisplayStateHooks::hook, 0, false),
            new Entry("BatteryHooks", BatteryHooks::hook, 0, false),
            // Sticky ACTION_BATTERY_CHANGED broadcast: Level/Status/Health/
            // Power source/Technology/Temperature/Voltage rows. Rewritten to
            // S26 Ultra discharging values (BatteryManager counters alone
            // don't cover the broadcast path).
            new Entry("BatteryIntentHooks", BatteryIntentHooks::hook, 0, false),
            new Entry("EuiccHooks", c -> EuiccHooks.hook(c, realDeviceSdk), 28, false),
            new Entry("InputDeviceHooks", InputDeviceHooks::hook, 0, false),
        };
        // DisplayHooks REMOVED: any Java display/resolution spoof breaks
        // persist.waydroid.width/height. Waydroid owns the real res.
        logInfo(verbose, TAG + ": DisplayHooks removed (Waydroid owns res)");
        for (Entry e : entries) {
            if (realDeviceSdk < e.minSdk) continue;
            if (e.skipInOwnProcess && isOwnPackage) {
                logInfo(verbose, TAG + ": " + e.name + " skipped for module process");
                continue;
            }
            if ("AccountHooks".equals(e.name) && !ConfigManager.isHideAccountsEnabled()) {
                logInfo(verbose, TAG + ": AccountHooks skipped (hooks.hide_accounts=0)");
                continue;
            }
            try {
                e.hook.apply(ctx);
                logInfo(verbose, TAG + ": " + e.name + " loaded");
            } catch (Throwable t) {
                Log.w(TAG, e.name + " failed: " + t.getMessage());
            }
        }
        if (!isOwnPackage) {
            // Java-only: NativeHooks.tryInstall always returns false
            // (ghost-stealth owns native libc via Dobby). Log once.
            logInfo(verbose, TAG + ": NativeHooks unavailable (Java-only spoofing active)");
        } else {
            logInfo(verbose, TAG + ": NativeHooks skipped for module process");
        }
    }

    private static void logInfo(boolean verbose, String message) {
        if (verbose) {
            Legacy.log(message);
        }
    }

    private static final long REFRESH_INTERVAL_MS = 2_000L;
    private static final java.util.concurrent.atomic.AtomicBoolean sRefreshInstalled =
            new java.util.concurrent.atomic.AtomicBoolean(false);
    private static android.os.HandlerThread sRefreshThread;
    private static android.os.Handler sRefreshHandler;

    private static void installRemoteRefreshLoop(final ClassLoader classLoader) {
        if (!sRefreshInstalled.compareAndSet(false, true)) return;
        try {
            sRefreshThread = new android.os.HandlerThread("spoof-refresh",
                    android.os.Process.THREAD_PRIORITY_BACKGROUND);
            sRefreshThread.start();
            sRefreshHandler = new android.os.Handler(sRefreshThread.getLooper());
            final Runnable tick = new Runnable() {
                @Override
                public void run() {
                    try {
                        ConfigManager.refreshFromRemoteIfNewer(classLoader);
                    } catch (Throwable t) {
                        Log.w(TAG, "Remote refresh tick failed: " + t.getMessage());
                    } finally {
                        sRefreshHandler.postDelayed(this, REFRESH_INTERVAL_MS);
                    }
                }
            };
            sRefreshHandler.postDelayed(tick, REFRESH_INTERVAL_MS);
            Log.i(TAG, "Remote refresh loop installed (interval="
                    + REFRESH_INTERVAL_MS + "ms)");
        } catch (Throwable t) {
            sRefreshInstalled.set(false);
            Log.w(TAG, "installRemoteRefreshLoop failed: " + t.getMessage());
        }
    }
}
