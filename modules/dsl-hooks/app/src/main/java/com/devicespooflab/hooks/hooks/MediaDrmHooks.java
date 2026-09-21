package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;
import com.devicespooflab.hooks.bridge.Legacy;

public class MediaDrmHooks {

    private static final String DEVICE_UNIQUE_ID = "deviceUniqueId";

    public static void hook(HookContext lpparam) {
        Class<?> mediaDrmClass = Legacy.findClassIfExists(
                "android.media.MediaDrm",
                lpparam.classLoader
        );

        if (mediaDrmClass == null) {
            return;
        }

        // getPropertyByteArray has no overloads but hookAllMethods keeps
        // the install uniform with the sibling hooks (fail-closed).
        Legacy.safeHook("DeviceSpoofLab-MediaDrm", "getPropertyByteArray", () -> {
            HookFramework.hookAllMethods(mediaDrmClass, "getPropertyByteArray",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                String propertyName = (String) chain.arg(0, null);

                                if (DEVICE_UNIQUE_ID.equals(propertyName)) {
                                    byte[] v = ConfigManager.getMediaDrmId();
                                    if (v != null) chain.replaceResult(v);
                                }
                            } catch (Throwable t) {
                                Legacy.log("DeviceSpoofLab-MediaDrm"
                                        + ": getPropertyByteArray failed: " + t);
                            }
                        }
                    });
        });
        // getPropertyString: vendor/securityLevel/oemCryptoApiVersion leak
        // Widevine SW tags; pin to L1 + stable vendor label. Fail-closed.
        Legacy.safeHook("DeviceSpoofLab-MediaDrm", "getPropertyString", () -> {
            HookFramework.hookAllMethods(mediaDrmClass, "getPropertyString",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                String key = chain.arg(0, null);
                                if ("securityLevel".equals(key)) {
                                    chain.replaceResult("L1");
                                } else if ("vendor".equals(key)) {
                                    chain.replaceResult("Google");
                                } else if ("oemCryptoApiVersion".equals(key)
                                        && result instanceof String) {
                                    String v = (String) result;
                                    if (v == null || v.isEmpty()) {
                                        chain.replaceResult("15");
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log("DeviceSpoofLab-MediaDrm"
                                        + ": getPropertyString failed: " + t);
                            }
                        }
                    });
        });
    }
}
