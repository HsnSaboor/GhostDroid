package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.utils.ConfigManager;

import java.util.HashSet;
import java.util.Set;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class BuildHooks {

    private static final String TAG = "DeviceSpoofLab-Build";

    public static void hook(HookContext lpparam) {
        try {
            Class<?> buildClass = findBuildClass(lpparam.classLoader);
            if (buildClass == null) {
                Legacy.log(TAG + ": Build class not found");
                return;
            }

            spoofBuildFields(buildClass);
            hookGetSerial(buildClass);
            hookGetRadioVersion(buildClass);
            hookPartitionMethods(lpparam.classLoader);
            spoofVersionFields(lpparam.classLoader);

            if (ConfigManager.isVerboseLoggingEnabled()) {
                Legacy.log(TAG + ": Successfully spoofed Build static fields and methods");
            }

        } catch (Exception e) {
            Legacy.log(TAG + ": Failed to hook Build methods: " + e.getMessage());
        }
    }

    public static void refreshStaticFields(ClassLoader classLoader) {
        try {
            Class<?> buildClass = findBuildClass(classLoader);
            if (buildClass == null) {
                Legacy.log(TAG + ": Build class not found during refresh");
                return;
            }

            spoofBuildFields(buildClass);
            spoofVersionFields(classLoader);

            if (ConfigManager.isVerboseLoggingEnabled()) {
                Legacy.log(TAG + ": Refreshed Build static fields");
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": Failed to refresh Build static fields: " + t.getMessage());
        }
    }

    private static Class<?> findBuildClass(ClassLoader classLoader) {
        Class<?> buildClass = Legacy.findClassIfExists("android.os.Build", classLoader);
        if (buildClass == null) {
            buildClass = Legacy.findClassIfExists("android.os.Build", ClassLoader.getSystemClassLoader());
        }
        return buildClass;
    }

    private static void spoofBuildFields(Class<?> buildClass) {
        setStringField(buildClass, "BOARD", ConfigManager.getBuildBoard());
        setStringField(buildClass, "BOOTLOADER", ConfigManager.getBuildBootloader());
        setStringField(buildClass, "BRAND", ConfigManager.getBuildBrand());
        setStringField(buildClass, "CPU_ABI", firstAbi(prop("ro.product.cpu.abi", "arm64-v8a")));
        setStringField(buildClass, "CPU_ABI2", secondAbi(prop("ro.product.cpu.abilist", "arm64-v8a,armeabi-v7a,armeabi")));
        setStringField(buildClass, "DEVICE", ConfigManager.getBuildDevice());
        setStringField(buildClass, "DISPLAY", ConfigManager.getBuildDisplay());
        setStringField(buildClass, "FINGERPRINT", ConfigManager.getBuildFingerprint());
        setStringField(buildClass, "HARDWARE", ConfigManager.getBuildHardware());
        setStringField(buildClass, "HOST", prop("ro.build.host", "android-build"));
        setStringField(buildClass, "ID", ConfigManager.getBuildId());
        setStringField(buildClass, "MANUFACTURER", ConfigManager.getBuildManufacturer());
        setStringField(buildClass, "MODEL", ConfigManager.getBuildModel());
        setStringField(buildClass, "ODM_SKU", prop("ro.boot.product.hardware.sku", ""));
        setStringField(buildClass, "PRODUCT", ConfigManager.getBuildProduct());
        setStringField(buildClass, "RADIO", getRadioVersion());
        setStringField(buildClass, "SERIAL", ConfigManager.getSerial());
        setStringField(buildClass, "SKU", prop("ro.boot.hardware.sku", ""));
        setStringField(buildClass, "SOC_MANUFACTURER", prop("ro.soc.manufacturer", ConfigManager.getBuildManufacturer()));
        setStringField(buildClass, "SOC_MODEL", prop("ro.soc.model", "SM8850"));
        setStringField(buildClass, "TAGS", ConfigManager.getBuildTags());
        setLongField(buildClass, "TIME", getBuildTimeMillis());
        setStringField(buildClass, "TYPE", ConfigManager.getBuildType());
        setStringField(buildClass, "UNKNOWN", "unknown");
        setStringField(buildClass, "USER", prop("ro.build.user", "android-build"));

        setStringArrayField(buildClass, "SUPPORTED_ABIS", splitCsv(prop("ro.product.cpu.abilist", "arm64-v8a,armeabi-v7a,armeabi")));
        setStringArrayField(buildClass, "SUPPORTED_64_BIT_ABIS", splitCsv(prop("ro.product.cpu.abilist64", "arm64-v8a")));
        setStringArrayField(buildClass, "SUPPORTED_32_BIT_ABIS", splitCsv(prop("ro.product.cpu.abilist32", "armeabi-v7a,armeabi")));

        String buildType = ConfigManager.getBuildType();
        setBooleanField(buildClass, "IS_DEBUGGABLE", propBoolean("ro.debuggable", false));
        setBooleanField(buildClass, "IS_EMULATOR", false);
        setBooleanField(buildClass, "IS_ENG", "eng".equals(buildType));
        setBooleanField(buildClass, "IS_TREBLE_ENABLED", propBoolean("ro.treble.enabled", true));
        setBooleanField(buildClass, "IS_USER", "user".equals(buildType));
        setBooleanField(buildClass, "IS_USERDEBUG", "userdebug".equals(buildType));
    }

    private static void spoofVersionFields(ClassLoader classLoader) {
        Class<?> versionClass = Legacy.findClassIfExists("android.os.Build$VERSION", classLoader);
        if (versionClass == null) {
            versionClass = Legacy.findClassIfExists("android.os.Build$VERSION", ClassLoader.getSystemClassLoader());
        }
        if (versionClass == null) {
            return;
        }

        int sdk = ConfigManager.getBuildVersionSdk();
        int previewSdk = propInt("ro.build.version.preview_sdk", 0);

        setStringField(versionClass, "BASE_OS", prop("ro.build.version.base_os", ""));
        setStringField(versionClass, "CODENAME", ConfigManager.getBuildVersionCodename());
        setStringField(versionClass, "INCREMENTAL", ConfigManager.getBuildVersionIncremental());
        setIntField(versionClass, "MEDIA_PERFORMANCE_CLASS", propInt("ro.odm.build.media_performance_class", 0));
        setIntField(versionClass, "MIN_SUPPORTED_TARGET_SDK_INT", propInt("ro.build.version.min_supported_target_sdk", 26));
        setIntField(versionClass, "PREVIEW_SDK_INT", previewSdk);
        setStringField(versionClass, "PREVIEW_SDK_FINGERPRINT", prop("ro.build.version.preview_sdk_fingerprint", "REL"));
        setStringField(versionClass, "RELEASE", ConfigManager.getBuildVersionRelease());
        setStringField(versionClass, "RELEASE_OR_CODENAME", prop(
                "ro.build.version.release_or_codename",
                ConfigManager.getBuildVersionRelease()
        ));
        setStringField(versionClass, "RELEASE_OR_PREVIEW_DISPLAY", prop(
                "ro.build.version.release_or_preview_display",
                prop("ro.build.version.release_or_codename", ConfigManager.getBuildVersionRelease())
        ));
        setIntField(versionClass, "RESOURCES_SDK_INT", previewSdk > 0 ? sdk + 1 : sdk);
        setStringField(versionClass, "SDK", String.valueOf(sdk));
        setIntField(versionClass, "SDK_INT", sdk);
        setStringField(versionClass, "SECURITY_PATCH", ConfigManager.getBuildVersionSecurityPatch());
        setStringArrayField(versionClass, "ACTIVE_CODENAMES", new String[0]);

        Set<String> knownCodenames = new HashSet<>();
        knownCodenames.add(ConfigManager.getBuildVersionCodename());
        setObjectField(versionClass, "KNOWN_CODENAMES", knownCodenames);
    }

    // getSerial / getRadioVersion: single no-arg overloads each (API 26+
    // / 14+); hookAllMethods covers uniformly. Fail-closed: before-hook
    // wrapped in try/catch + Legacy.log, install via safeHook.
    private static void hookGetSerial(Class<?> buildClass) {
        Legacy.safeHook(TAG, "Build.getSerial", () -> {
            HookFramework.hookAllMethods(buildClass, "getSerial",
                new HookFramework.BeforeHook() {@Override
                    public void before(HookFramework.HookChain chain) {
                        try {
                            String v = ConfigManager.getSerial();
                            if (v != null) chain.replaceResult(v);
                        } catch (Throwable t) {
                            Legacy.log(TAG + ": getSerial failed: " + t);
                        }
                    }
                });
        });
    }

    private static void hookGetRadioVersion(Class<?> buildClass) {
        Legacy.safeHook(TAG, "Build.getRadioVersion", () -> {
            HookFramework.hookAllMethods(buildClass, "getRadioVersion",
                    new HookFramework.BeforeHook() {@Override
                        public void before(HookFramework.HookChain chain) {
                            try {
                                chain.replaceResult(getRadioVersion());
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getRadioVersion failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookPartitionMethods(ClassLoader classLoader) {
        Class<?> found = Legacy.findClassIfExists("android.os.Build$Partition", classLoader);
        if (found == null) {
            found = Legacy.findClassIfExists("android.os.Build$Partition", ClassLoader.getSystemClassLoader());
        }
        if (found == null) {
            return;
        }
        final Class<?> partitionClass = found;

        // Partition.getFingerprint / getBuildTimeMillis: single no-arg
        // overloads each; hookAllMethods covers uniformly. Fail-closed:
        // after-hooks wrapped in try/catch + Legacy.log, installs via
        // safeHook.
        Legacy.safeHook(TAG, "Partition.getFingerprint", () -> {
            HookFramework.hookAllMethods(partitionClass, "getFingerprint",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                String partitionName = getPartitionName(chain.thisObject());
                                String spoofedValue = getPartitionFingerprint(partitionName);
                                if (spoofedValue != null) {
                                    chain.replaceResult(spoofedValue);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": Partition.getFingerprint failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "Partition.getBuildTimeMillis", () -> {
            HookFramework.hookAllMethods(partitionClass, "getBuildTimeMillis",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(getBuildTimeMillis());
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": Partition.getBuildTimeMillis failed: " + t);
                            }
                        }
                    });
        });
    }

    private static String getPartitionName(Object partition) {
        try {
            Object name = Legacy.callMethod(partition, "getName");
            return name instanceof String ? (String) name : null;
        } catch (Throwable ignored) {
            return null;
        }
    }

    private static String getPartitionFingerprint(String partitionName) {
        if (partitionName == null || partitionName.isEmpty()) {
            return ConfigManager.getBuildFingerprint();
        }

        String value = ConfigManager.getSystemProperty("ro." + partitionName + ".build.fingerprint", null);
        return value != null ? value : ConfigManager.getBuildFingerprint();
    }

    private static void setStringField(Class<?> clazz, String fieldName, String value) {
        if (value == null) {
            return;
        }

        try {
            Legacy.setStaticObjectField(clazz, fieldName, value);
        } catch (NoSuchFieldError ignored) {
        } catch (Throwable t) {
            Legacy.log(TAG + ": Failed to set " + clazz.getName() + "." + fieldName + ": " + t.getMessage());
        }
    }

    private static void setStringArrayField(Class<?> clazz, String fieldName, String[] value) {
        if (value == null) {
            return;
        }

        try {
            Legacy.setStaticObjectField(clazz, fieldName, value);
        } catch (NoSuchFieldError ignored) {
        } catch (Throwable t) {
            Legacy.log(TAG + ": Failed to set " + clazz.getName() + "." + fieldName + ": " + t.getMessage());
        }
    }

    private static void setIntField(Class<?> clazz, String fieldName, int value) {
        try {
            Legacy.setStaticIntField(clazz, fieldName, value);
        } catch (NoSuchFieldError ignored) {
        } catch (Throwable t) {
            Legacy.log(TAG + ": Failed to set " + clazz.getName() + "." + fieldName + ": " + t.getMessage());
        }
    }

    private static void setBooleanField(Class<?> clazz, String fieldName, boolean value) {
        try {
            Legacy.setStaticBooleanField(clazz, fieldName, value);
        } catch (NoSuchFieldError ignored) {
        } catch (Throwable t) {
            Legacy.log(TAG + ": Failed to set " + clazz.getName() + "." + fieldName + ": " + t.getMessage());
        }
    }

    private static void setLongField(Class<?> clazz, String fieldName, long value) {
        try {
            Legacy.setStaticLongField(clazz, fieldName, value);
        } catch (NoSuchFieldError ignored) {
        } catch (Throwable t) {
            Legacy.log(TAG + ": Failed to set " + clazz.getName() + "." + fieldName + ": " + t.getMessage());
        }
    }

    private static void setObjectField(Class<?> clazz, String fieldName, Object value) {
        if (value == null) {
            return;
        }

        try {
            Legacy.setStaticObjectField(clazz, fieldName, value);
        } catch (NoSuchFieldError ignored) {
        } catch (Throwable t) {
            Legacy.log(TAG + ": Failed to set " + clazz.getName() + "." + fieldName + ": " + t.getMessage());
        }
    }

    private static String prop(String key, String defaultValue) {
        String value = ConfigManager.getSystemProperty(key, null);
        return (value == null || value.isEmpty()) ? defaultValue : value;
    }

    private static int propInt(String key, int defaultValue) {
        String value = prop(key, String.valueOf(defaultValue));
        try {
            return Integer.parseInt(value);
        } catch (NumberFormatException e) {
            return defaultValue;
        }
    }

    private static boolean propBoolean(String key, boolean defaultValue) {
        String value = prop(key, String.valueOf(defaultValue));
        return "1".equals(value) || "true".equalsIgnoreCase(value);
    }

    private static long propLong(String key, long defaultValue) {
        String value = prop(key, String.valueOf(defaultValue));
        try {
            return Long.parseLong(value);
        } catch (NumberFormatException e) {
            return defaultValue;
        }
    }

    private static long getBuildTimeMillis() {
        long seconds = propLong("ro.build.date.utc", 1733356800L);
        return seconds > 100000000000L ? seconds : seconds * 1000L;
    }

    private static String getRadioVersion() {
        return prop("gsm.version.baseband", "unknown");
    }

    private static String firstAbi(String value) {
        String[] values = splitCsv(value);
        return values.length > 0 ? values[0] : "";
    }

    private static String secondAbi(String value) {
        String[] values = splitCsv(value);
        return values.length > 1 ? values[1] : "";
    }

    private static String[] splitCsv(String value) {
        if (value == null || value.trim().isEmpty()) {
            return new String[0];
        }

        String[] rawValues = value.split(",");
        int count = 0;
        for (String rawValue : rawValues) {
            if (!rawValue.trim().isEmpty()) {
                count++;
            }
        }

        String[] values = new String[count];
        int index = 0;
        for (String rawValue : rawValues) {
            String trimmed = rawValue.trim();
            if (!trimmed.isEmpty()) {
                values[index++] = trimmed;
            }
        }
        return values;
    }
}
