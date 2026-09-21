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
        // StatFs getters are single-overload each; exact-signature hooks
        // are correct here (no overloads exist). Every after-hook is
        // fail-closed: try/catch + Legacy.log, original kept on error.
        Legacy.safeHook(TAG, "StatFs.getBlockSizeLong", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getBlockSizeLong",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(BLOCK_SIZE);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getBlockSizeLong failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getBlockCountLong", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getBlockCountLong",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(ConfigManager.getStorageTotalBytes() / BLOCK_SIZE);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getBlockCountLong failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getAvailableBlocksLong", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getAvailableBlocksLong",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(ConfigManager.getStorageAvailableBytes() / BLOCK_SIZE);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getAvailableBlocksLong failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getFreeBlocksLong", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getFreeBlocksLong",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(ConfigManager.getStorageAvailableBytes() / BLOCK_SIZE);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getFreeBlocksLong failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getTotalBytes", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getTotalBytes",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(ConfigManager.getStorageTotalBytes());
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getTotalBytes failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getAvailableBytes", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getAvailableBytes",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(ConfigManager.getStorageAvailableBytes());
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getAvailableBytes failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getFreeBytes", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getFreeBytes",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                chain.replaceResult(ConfigManager.getStorageAvailableBytes());
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getFreeBytes failed: " + t);
                            }
                        }
                    });
        });

        hookStorageVolume(lpparam);
        hookStatFsIntVariants();
    }

    // StorageManager.getStorageVolumes / getPrimaryStorageVolume /
    // isEncrypted: total-size and encryption-state tells ACE reads without
    // StatFs. Fail-closed throughout.
    private static void hookStorageVolume(HookContext lpparam) {
        Class<?> sm = Legacy.findClassIfExists(
                "android.os.storage.StorageManager", lpparam.classLoader);
        if (sm == null) {
            return;
        }
        Legacy.safeHook(TAG, "StorageManager.isEncrypted", () -> {
            HookFramework.hookAllMethods(sm, "isEncrypted",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                                if (result instanceof Boolean
                                        && !(Boolean) result) {
                                    chain.replaceResult(true);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": isEncrypted failed: " + t);
                            }
                        }
                    });
        });
    }

    // Deprecated int variants ACE DEX still calls on older branches:
    // same values narrowed to int (256GB/4K blocks fit).
    private static void hookStatFsIntVariants() {
        Legacy.safeHook(TAG, "StatFs.getBlockSize", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getBlockSize",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult((int) BLOCK_SIZE);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getBlockSize failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getBlockCount", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getBlockCount",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult((int) (ConfigManager.getStorageTotalBytes() / BLOCK_SIZE));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getBlockCount failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getAvailableBlocks", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getAvailableBlocks",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult((int) (ConfigManager.getStorageAvailableBytes() / BLOCK_SIZE));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getAvailableBlocks failed: " + t);
                            }
                        }
                    });
        });

        Legacy.safeHook(TAG, "StatFs.getFreeBlocks", () -> {
            Legacy.findAndHookMethod(StatFs.class, "getFreeBlocks",
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                chain.replaceResult((int) (ConfigManager.getStorageAvailableBytes() / BLOCK_SIZE));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getFreeBlocks failed: " + t);
                            }
                        }
                    });
        });
    }
}
