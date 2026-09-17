package com.devicespooflab.hooks;

import java.io.File;
import java.lang.reflect.Method;
import java.util.HashMap;
import java.util.Map;

import com.devicespooflab.hooks.utils.ConfigManager;

import de.robv.android.xposed.XposedBridge;

public final class NativeHooks {

    private static final String TAG = "DeviceSpoofLab-Native";
    private static final String LIB_NAME = "ds_native";

    private static volatile boolean libraryLoaded = false;
    private static volatile boolean installed = false;

    private NativeHooks() {}

    private static native int nativeInstall(HashMap<String, String> props);

    private static native String nativeQuery(String key);

    public static synchronized boolean tryInstall(Map<String, String> props) {
        // GhostDroid Option A: Java-only. ghost-stealth owns native libc
        // (__system_property_*, uname) via Dobby; a second LSPlt hook set on
        // the same symbols collides (SEGV_ACCERR crash loop). Never load.
        return false;
    }

    public static String debugQuery(String key) {
        if (!installed) return null;
        try {
            return nativeQuery(key);
        } catch (Throwable t) {
            return null;
        }
    }

    private static boolean loadLibrary() {
        if (libraryLoaded) return true;

        try {
            System.loadLibrary(LIB_NAME);
            libraryLoaded = true;
            return true;
        } catch (UnsatisfiedLinkError ignored) {
        }

        // Fallback: look the .so up via the module classloader, then a small
        // set of known absolute paths. Runs during app startup; no filesystem walk.
        String absPath = locateLibraryOnDisk();
        if (absPath == null) {
            XposedBridge.log(TAG + ": could not locate " + LIB_NAME + " .so");
            return false;
        }
        try {
            System.load(absPath);
            libraryLoaded = true;
            if (ConfigManager.isVerboseLoggingEnabled()) {
                XposedBridge.log(TAG + ": loaded native lib via System.load(" + absPath + ")");
            }
            return true;
        } catch (Throwable t) {
            XposedBridge.log(TAG + ": System.load(" + absPath + ") failed: " + t);
            return false;
        }
    }

    private static String locateLibraryOnDisk() {
        String modulePkg = "com.devicespooflab.hooks";
        String soName = System.mapLibraryName(LIB_NAME);

        String classLoaderPath = findLibraryFromClassLoader();
        if (isUsableLibraryPath(classLoaderPath, soName)) {
            return classLoaderPath;
        }

        String[] exactPaths = new String[] {
                "/data/data/" + modulePkg + "/lib/" + soName,
                "/data/user/0/" + modulePkg + "/lib/" + soName,
        };
        for (String path : exactPaths) {
            if (path.contains(modulePkg) && isUsableLibraryPath(path, soName)) {
                return path;
            }
        }

        return null;
    }

    private static String findLibraryFromClassLoader() {
        ClassLoader loader = NativeHooks.class.getClassLoader();
        if (loader == null) {
            return null;
        }
        Class<?> clazz = loader.getClass();
        while (clazz != null) {
            try {
                Method method = clazz.getDeclaredMethod("findLibrary", String.class);
                method.setAccessible(true);
                Object value = method.invoke(loader, LIB_NAME);
                if (value instanceof String && !((String) value).isEmpty()) {
                    return (String) value;
                }
                return null;
            } catch (NoSuchMethodException e) {
                clazz = clazz.getSuperclass();
            } catch (Throwable ignored) {
                return null;
            }
        }
        return null;
    }

    private static boolean isUsableLibraryPath(String path, String soName) {
        if (path == null || path.isEmpty()) {
            return false;
        }
        if (!path.endsWith("/" + soName)) {
            return false;
        }
        try {
            File file = new File(path);
            return file.isFile() && file.canRead();
        } catch (Throwable ignored) {
            return false;
        }
    }
}
