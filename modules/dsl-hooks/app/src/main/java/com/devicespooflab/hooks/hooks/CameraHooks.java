package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class CameraHooks {

    private static final String TAG = "DeviceSpoofLab-Camera";

    // getCameraIdList: single no-arg overload; hookAllMethods covers
    // uniformly. Fail-closed: try/catch + Legacy.log, original kept.
    public static void hook(HookContext lpparam) {
        Class<?> cm = Legacy.findClassIfExists(
                "android.hardware.camera2.CameraManager", lpparam.classLoader);
        if (cm == null) return;

        Legacy.safeHook(TAG, "CameraManager.getCameraIdList", () -> {
            HookFramework.hookAllMethods(cm, "getCameraIdList",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null
                                        || !(result instanceof String[])) {
                                    return;
                                }
                                String[] ids = (String[]) result;
                                int kept = 0;
                                for (String id : ids) {
                                    if (id != null && !id.toLowerCase().contains("emulator")) {
                                        ids[kept++] = id;
                                    }
                                }
                                if (kept != ids.length) {
                                    String[] trimmed = new String[kept];
                                    System.arraycopy(ids, 0, trimmed, 0, kept);
                                    chain.replaceResult(trimmed);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getCameraIdList failed: " + t);
                            }
                        }
                    });
        });
    }
}
