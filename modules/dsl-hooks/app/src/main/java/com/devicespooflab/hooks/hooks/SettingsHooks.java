package com.devicespooflab.hooks.hooks;

import android.content.ContentResolver;
import android.database.Cursor;
import android.database.MatrixCursor;
import android.net.Uri;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class SettingsHooks {

    private static final String TAG = "DeviceSpoofLab-Settings";

    private static final int SPOOF_ANDROID_ID = 1;
    private static final int SPOOF_GSF_ID = 1 << 1;
    private static final int SPOOF_BLUETOOTH_ADDRESS = 1 << 2;
    private static final int SPOOF_DEVICE_NAMES = 1 << 3;

    private static final String ANDROID_ID = "android_id";
    private static final String GSF_ID = "gsf_id";
    private static final String BLUETOOTH_ADDRESS = "bluetooth_address";
    private static final String BLUETOOTH_NAME = "bluetooth_name";
    private static final String DEVICE_NAME = "device_name";

    public static void hook(HookContext lpparam) {
        hookClass(lpparam, "android.provider.Settings$Secure",
                SPOOF_ANDROID_ID | SPOOF_GSF_ID | SPOOF_BLUETOOTH_ADDRESS | SPOOF_DEVICE_NAMES);
        hookClass(lpparam, "android.provider.Settings$System",
                SPOOF_BLUETOOTH_ADDRESS | SPOOF_DEVICE_NAMES);
        hookClass(lpparam, "android.provider.Settings$Global",
                SPOOF_BLUETOOTH_ADDRESS | SPOOF_DEVICE_NAMES);
        hookDirectQuery();
    }

    private static void hookClass(HookContext lpparam,
                                  String className,
                                  int spoofFlags) {
        Class<?> clazz = Legacy.findClassIfExists(className, lpparam.classLoader);
        if (clazz == null) return;

        try {
            Legacy.findAndHookMethod(clazz, "getString",
                    ContentResolver.class, String.class,
                    new HookFramework.BeforeHook() {@Override
                        public void before(HookFramework.HookChain chain) {
                            String name = (String) chain.arg(1, null);
                            applySpoof(chain, name, spoofFlags);
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }

        // Per-user variant used by multi-user aware detectors (same key
        // position: arg 1). Before-hook so the provider is never queried.
        Legacy.safeHook(TAG, className + ".getStringForUser", () -> {
            HookFramework.hookAllMethods(clazz, "getStringForUser",
                    new HookFramework.BeforeHook() {
                        @Override
                        public void before(HookFramework.HookChain chain) {
                            try {
                                applySpoof(chain, chain.arg(1, null), spoofFlags);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getStringForUser spoof failed: " + t);
                            }
                        }
                    });
        });

        // Typed getters delegate to getString internally on most releases,
        // but some builds query the provider directly: coerce the spoofed
        // string to the requested type so both paths agree. Fail-closed.
        hookTypedGetter(clazz, "getInt", spoofFlags);
        hookTypedGetter(clazz, "getLong", spoofFlags);
        hookTypedGetter(clazz, "getFloat", spoofFlags);
    }

    private static void hookTypedGetter(Class<?> clazz, String method, int spoofFlags) {
        Legacy.safeHook(TAG, clazz.getSimpleName() + "." + method, () -> {
            HookFramework.hookAllMethods(clazz, method,
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                String name = chain.arg(1, null);
                                String spoofed = spoofedString(name, spoofFlags);
                                if (spoofed == null) {
                                    return;
                                }
                                if ("getInt".equals(method) && result instanceof Integer) {
                                    try {
                                        chain.replaceResult(Integer.parseInt(spoofed));
                                    } catch (NumberFormatException ignored) {
                                    }
                                } else if ("getLong".equals(method) && result instanceof Long) {
                                    try {
                                        chain.replaceResult(Long.parseLong(spoofed));
                                    } catch (NumberFormatException ignored) {
                                    }
                                } else if ("getFloat".equals(method) && result instanceof Float) {
                                    try {
                                        chain.replaceResult(Float.parseFloat(spoofed));
                                    } catch (NumberFormatException ignored) {
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": " + method + " spoof failed: " + t);
                            }
                        }
                    });
        });
    }

    // Direct ContentResolver.query(content://settings/secure, ...) bypasses
    // Settings.getString entirely. Rewrite name/value rows for spoofed keys
    // via a MatrixCursor copy; any error keeps the original cursor.
    private static void hookDirectQuery() {
        Legacy.safeHook(TAG, "ContentResolver.query", () -> {
            HookFramework.hookAllMethods(ContentResolver.class, "query",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (!(result instanceof Cursor)) {
                                    return;
                                }
                                Uri uri = chain.arg(0, null);
                                int flags = flagsForUri(uri);
                                if (flags == 0) {
                                    return;
                                }
                                Cursor rewritten = rewriteCursor((Cursor) result, flags);
                                if (rewritten != null) {
                                    chain.replaceResult(rewritten);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": query rewrite failed: " + t);
                            }
                        }
                    });
        });
    }

    private static int flagsForUri(Uri uri) {
        try {
            if (uri == null) {
                return 0;
            }
            if (!"settings".equals(uri.getAuthority())) {
                return 0;
            }
            String path = String.valueOf(uri.getPath());
            if (path.contains("secure")) {
                return SPOOF_ANDROID_ID | SPOOF_GSF_ID
                        | SPOOF_BLUETOOTH_ADDRESS | SPOOF_DEVICE_NAMES;
            }
            if (path.contains("system") || path.contains("global")) {
                return SPOOF_BLUETOOTH_ADDRESS | SPOOF_DEVICE_NAMES;
            }
            return 0;
        } catch (Throwable ignored) {
            return 0;
        }
    }

    private static Cursor rewriteCursor(Cursor orig, int flags) {
        try {
            int nameIdx = orig.getColumnIndex("name");
            int valueIdx = orig.getColumnIndex("value");
            if (nameIdx < 0 || valueIdx < 0) {
                return null;
            }
            String[] columns = orig.getColumnNames();
            MatrixCursor out = new MatrixCursor(columns);
            boolean changed = false;
            int count = orig.getCount();
            for (int pos = 0; pos < count; pos++) {
                if (!orig.moveToPosition(pos)) {
                    continue;
                }
                Object[] row = new Object[columns.length];
                for (int i = 0; i < columns.length; i++) {
                    try {
                        row[i] = orig.getString(i);
                    } catch (Throwable ignored) {
                        row[i] = null;
                    }
                }
                String rowName = (nameIdx < row.length && row[nameIdx] instanceof String)
                        ? (String) row[nameIdx] : null;
                String spoofed = spoofedString(rowName, flags);
                if (spoofed != null && valueIdx < row.length) {
                    row[valueIdx] = spoofed;
                    changed = true;
                }
                out.addRow(row);
            }
            if (!changed) {
                out.close();
                return null;
            }
            try {
                orig.close();
            } catch (Throwable ignored) {
            }
            return out;
        } catch (Throwable t) {
            Legacy.log(TAG + ": rewriteCursor failed: " + t);
            return null;
        }
    }

    private static String spoofedString(String name, int spoofFlags) {
        if (name == null) {
            return null;
        }
        try {
            if ((spoofFlags & SPOOF_ANDROID_ID) != 0 && ANDROID_ID.equals(name)) {
                return ConfigManager.getAndroidId();
            }
            if ((spoofFlags & SPOOF_GSF_ID) != 0 && GSF_ID.equals(name)) {
                return ConfigManager.getGSFId();
            }
            if ((spoofFlags & SPOOF_BLUETOOTH_ADDRESS) != 0 && BLUETOOTH_ADDRESS.equals(name)) {
                String mac = ConfigManager.getBluetoothMacAddress();
                return mac != null ? mac.toUpperCase() : null;
            }
            if ((spoofFlags & SPOOF_DEVICE_NAMES) != 0
                    && (BLUETOOTH_NAME.equals(name) || DEVICE_NAME.equals(name))) {
                return ConfigManager.getBuildModel();
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": spoofedString failed: " + t);
        }
        return null;
    }

    private static void applySpoof(HookFramework.HookChain chain, String name, int spoofFlags) {
        String spoofed = spoofedString(name, spoofFlags);
        if (spoofed != null) {
            chain.replaceResult(spoofed);
        }
    }
}
