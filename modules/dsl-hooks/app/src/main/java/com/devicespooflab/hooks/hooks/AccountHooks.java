package com.devicespooflab.hooks.hooks;

import android.accounts.Account;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

// AccountManager.getAccounts is hooked; getAccountsByType is intentionally not.
public class AccountHooks {

    private static final String TAG = "DeviceSpoofLab-Account";

    public static void hook(HookContext lpparam) {
        if (!ConfigManager.isHideAccountsEnabled()) {
            return;
        }

        Class<?> am = Legacy.findClassIfExists(
                "android.accounts.AccountManager", lpparam.classLoader);
        if (am == null) return;

        try {
            Legacy.findAndHookMethod(am, "getAccounts",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(new Account[0]);
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook getAccounts: " + t);
        }

        try {
            Legacy.findAndHookMethod(am, "getAccountsAsUser",
                    int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(new Account[0]);
                        }
                    });
        } catch (Throwable t) { /* hidden API; may be missing */ }

        // Sibling enumeration paths that bypass getAccounts():
        // getAccountsByType / getAccountsByTypeForPackage / getAccountsForPackage
        // / getAccountsByTypeAndFeatures. Same empty-account policy.
        // Fail-closed: errors keep the original result.
        for (String name : new String[]{
                "getAccountsByType", "getAccountsByTypeForPackage",
                "getAccountsForPackage", "getAccountsByTypeAndFeatures"}) {
            final String method = name;
            Legacy.safeHook(TAG, "AccountManager." + method, () -> {
                HookFramework.hookAllMethods(am, method,
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    if (result instanceof Account[]) {
                                        Account[] orig = (Account[]) result;
                                        if (orig.length != 0) {
                                            chain.replaceResult(new Account[0]);
                                        }
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": " + method
                                            + " failed: " + t);
                                }
                            }
                        });
            });
        }
    }
}
