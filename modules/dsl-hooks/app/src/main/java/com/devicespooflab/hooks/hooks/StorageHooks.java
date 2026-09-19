package com.devicespooflab.hooks.hooks;

import android.os.StatFs;

import com.devicespooflab.hooks.utils.ConfigManager;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class StorageHooks {

    private static final String TAG = "DeviceSpoofLab-Storage";
    private static final long BLOCK_SIZE = 4096L;

    public static void hook(HookContext lpparam) {
        Legacy.safeHook(TAG, "StatFs.getBlockSizeLong", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getBlockSizeLong",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(BLOCK_SIZE);
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getBlockCountLong", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getBlockCountLong",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(ConfigManager.getStorageTotalBytes() / BLOCK_SIZE);
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getAvailableBlocksLong", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getAvailableBlocksLong",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(ConfigManager.getStorageAvailableBytes() / BLOCK_SIZE);
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getFreeBlocksLong", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getFreeBlocksLong",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(ConfigManager.getStorageAvailableBytes() / BLOCK_SIZE);
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getTotalBytes", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getTotalBytes",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(ConfigManager.getStorageTotalBytes());
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getAvailableBytes", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getAvailableBytes",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(ConfigManager.getStorageAvailableBytes());
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getFreeBytes", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getFreeBytes",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            chain.replaceResult(ConfigManager.getStorageAvailableBytes());
                        }
                    });
        });
    }
}
