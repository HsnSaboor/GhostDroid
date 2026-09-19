package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class CameraHooks {

    private static final String TAG = "DeviceSpoofLab-Camera";

    public static void hook(HookContext lpparam) {
        Class<?> cm = Legacy.findClassIfExists(
                "android.hardware.camera2.CameraManager", lpparam.classLoader);
        if (cm == null) return;

        try {
            Legacy.findAndHookMethod(cm, "getCameraIdList",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String[] ids = (String[]) result;
                            if (ids == null) return;
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
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook getCameraIdList: " + t);
        }
    }
}
