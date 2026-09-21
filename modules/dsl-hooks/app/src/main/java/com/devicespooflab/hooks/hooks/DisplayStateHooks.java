package com.devicespooflab.hooks.hooks;

import com.devicespooflab.hooks.bridge.HookContext;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.Legacy;

// Display / window-manager state ACE DEX reads without native:
// Display.getMetrics/getRealMetrics/getRealSize/getSize/getRefreshRate,
// WindowManager.getDefaultDisplay, DisplayMetrics field pinning via
// Resources.getDisplayMetrics (densityDpi pinned to a concrete xxhdpi
// bucket so host-density leaks cannot disagree with DisplayMetrics).
// Resolution values are NOT spoofed: Waydroid owns the real window
// (persist.waydroid.width/height); only leak-shaped fields (host density
// strings, emulator refresh quirks) are normalized.
public final class DisplayStateHooks {

    private static final String TAG = "DeviceSpoofLab-DisplayState";

    private DisplayStateHooks() {}

    public static void hook(HookContext lpparam) {
        hookDisplay(lpparam);
        hookWindowManager(lpparam);
        hookDisplayMetrics(lpparam);
    }

    private static void hookDisplay(HookContext lpparam) {
        Class<?> display = Legacy.findClassIfExists(
                "android.view.Display", lpparam.classLoader);
        if (display == null) {
            return;
        }
        // Display.getSupportedRefreshRates (API 33+ float[]): host panels
        // report odd rates; clamp any out-of-phone-range entry into the
        // 60/90/120 bucket set. Resolution untouched — Waydroid owns it.
        // Fail-closed: errors keep the original array.
        Legacy.safeHook(TAG, "Display.getSupportedRefreshRates", () -> {
            HookFramework.hookAllMethods(display, "getSupportedRefreshRates",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null
                                        || !(result instanceof float[])) {
                                    return;
                                }
                                float[] rates = (float[]) result;
                                float[] clamped = clampRefreshRates(rates);
                                if (clamped != null) {
                                    chain.replaceResult(clamped);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getSupportedRefreshRates failed: "
                                        + t);
                            }
                        }
                    });
        });
        // Display.getSupportedModes (Mode[]): same clamp policy per mode
        // refresh rate; Mode objects are patched in place (no safe ctor).
        // Fail-closed: errors keep the original array.
        Legacy.safeHook(TAG, "Display.getSupportedModes", () -> {
            HookFramework.hookAllMethods(display, "getSupportedModes",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null
                                        || !(result instanceof Object[])) {
                                    return;
                                }
                                clampDisplayModes((Object[]) result);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getSupportedModes failed: "
                                        + t);
                            }
                        }
                    });
        });
        // Refresh rate: normalize absurd host values into a phone bucket.
        // Real panels report 60/90/120; only clamp out-of-range leaks.
        Legacy.safeHook(TAG, "Display.getRefreshRate", () -> {
            HookFramework.hookAllMethods(display, "getRefreshRate",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof Float) {
                                    float hz = (Float) result;
                                    if (hz <= 0.0f || hz > 240.0f) {
                                        chain.replaceResult(60.0f);
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getRefreshRate failed: "
                                        + t);
                            }
                        }
                    });
        });
        Legacy.safeHook(TAG, "Display.getDisplayId", () -> {
            HookFramework.hookAllMethods(display, "getDisplayId",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null) {
                                    return;
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDisplayId failed: " + t);
                            }
                        }
                    });
        });
    }

    private static void hookWindowManager(HookContext lpparam) {
        Class<?> wm = Legacy.findClassIfExists(
                "android.view.WindowManager", lpparam.classLoader);
        if (wm != null) {
            // NOTE: WindowManager.getDefaultDisplay is abstract on the
            // interface (no body to hook) — hooking it throws
            // IllegalArgumentException noise. The concrete impl lives on
            // WindowManagerImpl; Display.* hooks below cover the signal.
            // Deliberately NOT hooked.
            Legacy.safeHook(TAG, "WindowManager.getCurrentWindowMetrics", () -> {
                HookFramework.hookAllMethods(wm, "getCurrentWindowMetrics",
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                } catch (Throwable t) {
                                    Legacy.log(TAG
                                            + ": getCurrentWindowMetrics failed: "
                                            + t);
                                }
                            }
                        });
            });
        }
        // Display.getMetrics/getRealMetrics/getSize/getRealSize/getRectSize:
        // Waydroid owns the real window — never rewrite values, but ensure
        // no host-density string leaks disagreement by normalizing the
        // delivered DisplayMetrics through the same bucket path.
        Class<?> display = Legacy.findClassIfExists(
                "android.view.Display", lpparam.classLoader);
        if (display == null) {
            return;
        }
        for (String name : new String[]{
                "getMetrics", "getRealMetrics", "getSize", "getRealSize",
                "getRectSize"}) {
            final String method = name;
            Legacy.safeHook(TAG, "Display." + method, () -> {
                HookFramework.hookAllMethods(display, method,
                        new HookFramework.Hook() {
                            @Override
                            public void after(HookFramework.HookChain chain,
                                    Object result, Throwable error) {
                                try {
                                    if (error != null) {
                                        return;
                                    }
                                    normalizeDelivered(chain);
                                } catch (Throwable t) {
                                    Legacy.log(TAG + ": " + method
                                            + " failed: " + t);
                                }
                            }
                        });
            });
        }
    }

    private static void normalizeDelivered(HookFramework.HookChain chain) {
        try {
            if (chain.argCount() == 0) {
                return;
            }
            Object out = chain.arg(0, null);
            if (out instanceof android.util.DisplayMetrics) {
                normalizeDensity((android.util.DisplayMetrics) out);
            } else if (out instanceof android.graphics.Point) {
                // Resolution untouched — Waydroid owns it.
            } else if (out instanceof android.graphics.Rect) {
                // Resolution untouched — Waydroid owns it.
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": normalizeDelivered failed: " + t);
        }
    }

    private static void hookDisplayMetrics(HookContext lpparam) {
        Class<?> resources = Legacy.findClassIfExists(
                "android.content.res.Resources", lpparam.classLoader);
        if (resources == null) {
            return;
        }
        // Resources.getDisplayMetrics: pin densityDpi to a concrete bucket
        // derived from the real density so host fractional leaks (e.g.
        // Waydroid 225.5) cannot disagree with Display.getMetrics callers.
        Legacy.safeHook(TAG, "Resources.getDisplayMetrics", () -> {
            HookFramework.hookAllMethods(resources, "getDisplayMetrics",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (result instanceof android.util
                                        .DisplayMetrics) {
                                    normalizeDensity(
                                            (android.util.DisplayMetrics)
                                                    result);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDisplayMetrics failed: "
                                        + t);
                            }
                        }
                    });
        });
    }

    private static void normalizeDensity(android.util.DisplayMetrics dm) {
        try {
            int bucket = bucketFor(dm.densityDpi);
            dm.densityDpi = bucket;
            if (dm.density != 0.0f) {
                dm.density = bucket / 160.0f;
            }
            dm.xdpi = bucket;
            dm.ydpi = bucket;
        } catch (Throwable t) {
            Legacy.log(TAG + ": normalizeDensity failed: " + t);
        }
    }

    private static int bucketFor(int dpi) {
        int[] buckets = {120, 160, 213, 240, 320, 420, 480, 560, 640};
        int best = 320;
        int bestDist = Integer.MAX_VALUE;
        for (int b : buckets) {
            int dist = Math.abs(b - dpi);
            if (dist < bestDist) {
                bestDist = dist;
                best = b;
            }
        }
        return best;
    }

    private static float[] clampRefreshRates(float[] rates) {
        try {
            boolean changed = false;
            float[] out = rates.clone();
            for (int i = 0; i < out.length; i++) {
                float clamped = clampRate(out[i]);
                if (clamped != out[i]) {
                    out[i] = clamped;
                    changed = true;
                }
            }
            return changed ? out : null;
        } catch (Throwable t) {
            Legacy.log(TAG + ": clampRefreshRates failed: " + t);
            return null;
        }
    }

    private static float clampRate(float hz) {
        if (hz <= 0.0f || hz > 240.0f) {
            return 60.0f;
        }
        float[] buckets = {60.0f, 90.0f, 120.0f};
        float best = buckets[0];
        float bestDist = Math.abs(hz - best);
        for (float b : buckets) {
            float dist = Math.abs(hz - b);
            if (dist < bestDist) {
                bestDist = dist;
                best = b;
            }
        }
        return best;
    }

    private static void clampDisplayModes(Object[] modes) {
        try {
            for (Object mode : modes) {
                if (mode == null) {
                    continue;
                }
                float rate;
                try {
                    Object boxed = Legacy.callMethod(mode, "getRefreshRate");
                    if (!(boxed instanceof Float)) {
                        continue;
                    }
                    rate = (Float) boxed;
                } catch (Throwable ignored) {
                    continue;
                }
                float clamped = clampRate(rate);
                if (clamped == rate) {
                    continue;
                }
                // Display.Mode has no public mutator; patch the hidden
                // mRefreshRate field reflectively. Absent field = skip.
                try {
                    Legacy.setObjectField(mode, "mRefreshRate", clamped);
                } catch (Throwable ignored) {
                }
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": clampDisplayModes failed: " + t);
        }
    }
}
