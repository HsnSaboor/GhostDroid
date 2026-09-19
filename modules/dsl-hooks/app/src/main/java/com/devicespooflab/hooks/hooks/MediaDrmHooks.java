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

        try {
            Legacy.findAndHookMethod(mediaDrmClass, "getPropertyByteArray",
                    String.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String propertyName = (String) chain.arg(0, null);

                            if (DEVICE_UNIQUE_ID.equals(propertyName)) {
                                byte[] v = ConfigManager.getMediaDrmId();
                                if (v != null) chain.replaceResult(v);
                            }
                        }
                    });
        } catch (NoSuchMethodError ignored) {
        }
    }
}
