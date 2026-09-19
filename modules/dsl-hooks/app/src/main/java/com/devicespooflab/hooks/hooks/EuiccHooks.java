package com.devicespooflab.hooks.hooks;

import android.os.Build;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class EuiccHooks {

    private static final String TAG = "DeviceSpoofLab-Euicc";

    public static void hook(HookContext lpparam) {
        hook(lpparam, Build.VERSION.SDK_INT);
    }

    public static void hook(HookContext lpparam, int realDeviceSdk) {
        if (realDeviceSdk < 28) return;

        Class<?> em = Legacy.findClassIfExists(
                "android.telephony.euicc.EuiccManager", lpparam.classLoader);
        if (em == null) return;

        try {
            Legacy.findAndHookMethod(em, "getEid",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            String v = ConfigManager.getEid();
                            if (v != null) chain.replaceResult(v);
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook EuiccManager.getEid: " + t);
        }
    }
}
