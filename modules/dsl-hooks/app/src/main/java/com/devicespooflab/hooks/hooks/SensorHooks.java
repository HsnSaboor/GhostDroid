package com.devicespooflab.hooks.hooks;

import android.hardware.Sensor;
import android.hardware.SensorManager;

import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.List;

import com.devicespooflab.hooks.bridge.Legacy;
import com.devicespooflab.hooks.bridge.HookFramework;
import com.devicespooflab.hooks.bridge.HookContext;

public class SensorHooks {

    private static final String TAG = "DeviceSpoofLab-Sensor";
    private static final String[] DENY = {"goldfish", "ranchu", "emulator", "qemu", "vbox",
            "thinkpad", "elantech", "etps", "hda intel", "sunplusit", "xhci",
            "pnp0c", "lnxpwr", "lnxvideo", "i8042", "pcspkr", "isa006",
            "acpi", "cachyos", "mesa", "intel", "linux"};

    // S26 Ultra style sensor set: Waydroid exposes ZERO sensors, and an
    // empty getSensorList is a textbook emulator tell (PUBG matchmaking
    // notice). Synthesize presence-only Sensor instances (no HAL behind
    // them; registerListener stays false) so presence checks pass while
    // real data paths are untouched. Fail-closed: any reflection error
    // leaves the filtered list as-is and the game keeps running.

    // {type, stringType, name, vendor, maxRange, resolution}
    // IMU/magnetometer mirror Galaxy S25 Ultra teardown parts (iFixit:
    // STMicro LSM6DSV 6-axis + AKM magnetometer) — closest verified
    // Samsung flagship reference; software sensors carry Samsung vendor.
    private static final Object[][] SYNTHETIC = {
        {Sensor.TYPE_ACCELEROMETER, "android.sensor.accelerometer",
                "LSM6DSV Accelerometer", "STMicroelectronics", 156.9064f, 0.0012f},
        {Sensor.TYPE_GYROSCOPE, "android.sensor.gyroscope",
                "LSM6DSV Gyroscope", "STMicroelectronics", 34.9066f, 0.0011f},
        {Sensor.TYPE_MAGNETIC_FIELD, "android.sensor.magnetic_field",
                "AK09918 Magnetometer", "AKM", 2500.0f, 0.15f},
        {Sensor.TYPE_PROXIMITY, "android.sensor.proximity",
                "S26 Proximity", "Samsung", 8.0f, 1.0f},
        {Sensor.TYPE_LIGHT, "android.sensor.light",
                "S26 Light", "Samsung", 43000.0f, 1.0f},
        {Sensor.TYPE_GRAVITY, "android.sensor.gravity",
                "S26 Gravity", "Samsung", 19.6133f, 0.01f},
        {Sensor.TYPE_LINEAR_ACCELERATION, "android.sensor.linear_acceleration",
                "S26 Linear Acceleration", "Samsung", 19.6133f, 0.01f},
        {Sensor.TYPE_ROTATION_VECTOR, "android.sensor.rotation_vector",
                "S26 Rotation Vector", "Samsung", 1.0f, 0.0001f},
        {Sensor.TYPE_PRESSURE, "android.sensor.pressure",
                "BMP390 Pressure", "Samsung", 1100.0f, 0.01f},
        {Sensor.TYPE_STEP_COUNTER, "android.sensor.step_counter",
                "S26 Step Counter", "Samsung", 100000.0f, 1.0f},
        {Sensor.TYPE_STEP_DETECTOR, "android.sensor.step_detector",
                "S26 Step Detector", "Samsung", 1.0f, 1.0f},
    };

    private static float sStepCount = 1234.0f;

    private static List<Sensor> syntheticCache = null;

    public static void hook(HookContext lpparam) {
        try {
            Legacy.findAndHookMethod(SensorManager.class, "getSensorList",
                    int.class,
                    new HookFramework.Hook() {@Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                List<Sensor> orig = (List<Sensor>) result;
                                java.util.List<Object> args = chain.args();
                                int type = (args != null && !args.isEmpty() && args.get(0) instanceof Integer)
                                        ? (Integer) args.get(0) : Sensor.TYPE_ALL;
                                chain.replaceResult(withSynthetic(filter(orig), type));
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getSensorList filter failed: " + t);
                            }
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook getSensorList: " + t);
        }

        try {
            Legacy.findAndHookMethod(SensorManager.class, "getDefaultSensor",
                    int.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                Sensor s = (Sensor) result;
                                if (s != null && isEmulatorSensor(s)) {
                                    s = null;
                                }
                                if (s == null) {
                                    Sensor synth = syntheticFor(requestedType(chain, 0));
                                    if (synth != null) chain.replaceResult(synth);
                                    else if (result != null) chain.replaceResult(null);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDefaultSensor filter failed: " + t);
                            }
                        }
                    });
        } catch (Throwable t) {
            Legacy.log(TAG + ": failed to hook getDefaultSensor: " + t);
        }

        try {
            Legacy.findAndHookMethod(SensorManager.class, "getDefaultSensor",
                    int.class, boolean.class,
                    new HookFramework.Hook() {@Override
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            try {
                                Sensor s = (Sensor) result;
                                if (s != null && isEmulatorSensor(s)) {
                                    s = null;
                                }
                                if (s == null) {
                                    Sensor synth = syntheticFor(requestedType(chain, 0));
                                    if (synth != null) chain.replaceResult(synth);
                                    else if (result != null) chain.replaceResult(null);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDefaultSensor(wakeUp) filter failed: " + t);
                            }
                        }
                    });
        } catch (Throwable t) { /* version-specific overload */ }

        hookRegisterListener();
        hookUnregisterListener();
        hookDynamicSensorList();
        hookSensorDetails(lpparam);
    }

    // Synthetic sensors have no HAL behind them, so the real
    // registerListener would return false and games would conclude the
    // device has no motion path. Claim success for synthetic handles and
    // post one synthetic event; real sensors pass through untouched.
    // Fail-closed: any error leaves the original result as-is.
    private static void hookRegisterListener() {
        Legacy.safeHook(TAG, "registerListener", () -> {
            HookFramework.hookAllMethods(SensorManager.class, "registerListener",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                Sensor sensor = sensorArg(chain);
                                if (sensor == null || !isSyntheticHandle(sensor)) {
                                    return;
                                }
                                Object listener = listenerArg(chain);
                                android.os.Handler handler = handlerArg(chain);
                                if (listener != null) {
                                    postSyntheticEvent(listener, sensor, handler);
                                }
                                chain.replaceResult(true);
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": registerListener synth post failed: " + t);
                            }
                        }
                    });
        });
    }

    // Mirror of registerListener: unregistering a synthetic handle must
    // report success (boolean overloads) instead of leaking "no such
    // sensor". Void overloads pass through untouched. Fail-closed.
    private static void hookUnregisterListener() {
        Legacy.safeHook(TAG, "unregisterListener", () -> {
            HookFramework.hookAllMethods(SensorManager.class, "unregisterListener",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                Sensor sensor = sensorArg(chain);
                                if (sensor == null || !isSyntheticHandle(sensor)) {
                                    return;
                                }
                                if (result instanceof Boolean) {
                                    chain.replaceResult(true);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": unregisterListener synth ack failed: " + t);
                            }
                        }
                    });
        });
    }

    // Dynamic sensors (USB/BT hot-plug): phones normally report none, so
    // only strip emulator tokens here — never append static synthetics.
    // Fail-closed: any error leaves the original list as-is.
    private static void hookDynamicSensorList() {
        Legacy.safeHook(TAG, "getDynamicSensorList", () -> {
            HookFramework.hookAllMethods(SensorManager.class, "getDynamicSensorList",
                    new HookFramework.Hook() {
                        @Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (!(result instanceof List)) {
                                    return;
                                }
                                List<Sensor> filtered = filter((List<Sensor>) result);
                                if (filtered.size() != ((List<?>) result).size()) {
                                    chain.replaceResult(filtered);
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": getDynamicSensorList filter failed: " + t);
                            }
                        }
                    });
        });
    }

    // Sensor scalar accessors detectors call per-sensor: getName /
    // getVendor / getStringType / getMaximumRange carry host leak strings
    // on real (non-synthetic) sensors. Synthetics already carry Samsung
    // names — leave them untouched; scrub only emulator-token values on
    // real sensors to the matching synthetic's neutral label. Fail-closed.
    private static void hookSensorDetails(HookContext lpparam) {
        Legacy.safeHook(TAG, "Sensor.getName", () -> {
            HookFramework.hookAllMethods(Sensor.class, "getName",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null
                                        || !(result instanceof String)) {
                                    return;
                                }
                                Object self = chain.thisObject();
                                if (!(self instanceof Sensor)
                                        || isSyntheticHandle((Sensor) self)) {
                                    return;
                                }
                                String name = (String) result;
                                if (isEmulatorToken(name)) {
                                    Sensor synth = syntheticFor(
                                            ((Sensor) self).getType());
                                    if (synth != null) {
                                        chain.replaceResult(synth.getName());
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": Sensor.getName failed: " + t);
                            }
                        }
                    });
            HookFramework.hookAllMethods(Sensor.class, "getVendor",
                    new HookFramework.Hook() {
                        @Override
                        public void after(HookFramework.HookChain chain,
                                Object result, Throwable error) {
                            try {
                                if (error != null
                                        || !(result instanceof String)) {
                                    return;
                                }
                                Object self = chain.thisObject();
                                if (!(self instanceof Sensor)
                                        || isSyntheticHandle((Sensor) self)) {
                                    return;
                                }
                                String vendor = (String) result;
                                if (isEmulatorToken(vendor)) {
                                    Sensor synth = syntheticFor(
                                            ((Sensor) self).getType());
                                    if (synth != null) {
                                        chain.replaceResult(synth.getVendor());
                                    }
                                }
                            } catch (Throwable t) {
                                Legacy.log(TAG + ": Sensor.getVendor failed: " + t);
                            }
                        }
                    });
        });
    }

    private static boolean isEmulatorToken(String value) {
        if (value == null) {
            return false;
        }
        String lower = value.toLowerCase(java.util.Locale.US);
        for (String token : DENY) {
            if (lower.contains(token)) {
                return true;
            }
        }
        return false;
    }

    private static Sensor sensorArg(HookFramework.HookChain chain) {
        try {
            for (Object arg : chain.args()) {
                if (arg instanceof Sensor) {
                    return (Sensor) arg;
                }
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    private static Object listenerArg(HookFramework.HookChain chain) {
        try {
            for (Object arg : chain.args()) {
                if (arg instanceof android.hardware.SensorEventListener) {
                    return arg;
                }
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    private static android.os.Handler handlerArg(HookFramework.HookChain chain) {
        try {
            for (Object arg : chain.args()) {
                if (arg instanceof android.os.Handler) {
                    return (android.os.Handler) arg;
                }
            }
        } catch (Throwable ignored) {
        }
        return null;
    }

    private static boolean isSyntheticHandle(Sensor sensor) {
        try {
            int handle = Legacy.getIntField(sensor, "mHandle");
            return handle <= -1000 && handle > -2000;
        } catch (Throwable ignored) {
            return false;
        }
    }

    private static void postSyntheticEvent(Object listener, Sensor sensor,
            android.os.Handler handler) {
        try {
            final android.hardware.SensorEventListener target =
                    (android.hardware.SensorEventListener) listener;
            final android.hardware.SensorEvent event =
                    syntheticEvent(sensor);
            if (event == null) {
                return;
            }
            Runnable deliver = new Runnable() {
                @Override
                public void run() {
                    try {
                        target.onSensorChanged(event);
                    } catch (Throwable t) {
                        Legacy.log(TAG + ": synth deliver failed: " + t);
                    }
                }
            };
            boolean posted = false;
            if (handler != null) {
                try {
                    posted = handler.post(deliver);
                } catch (Throwable ignored) {
                }
            }
            if (!posted) {
                try {
                    deliver.run();
                } catch (Throwable t) {
                    Legacy.log(TAG + ": synth direct deliver failed: " + t);
                }
            }
        } catch (Throwable t) {
            Legacy.log(TAG + ": postSyntheticEvent failed: " + t);
        }
    }

    private static android.hardware.SensorEvent syntheticEvent(Sensor sensor) {
        try {
            float[] values = valuesFor(sensor.getType());
            Object inst = allocateSensorEvent(sensor, values);
            return (android.hardware.SensorEvent) inst;
        } catch (Throwable t) {
            Legacy.log(TAG + ": syntheticEvent build failed: " + t);
            return null;
        }
    }

    private static float noise() {
        return (float) ((Math.random() - 0.5) * 0.08);
    }

    private static synchronized float nextSteps() {
        sStepCount += 1.0f;
        return sStepCount;
    }

    private static float[] valuesFor(int type) {
        switch (type) {
            case Sensor.TYPE_ACCELEROMETER:
                return new float[]{noise(), noise(), 9.81f + noise()};
            case Sensor.TYPE_GYROSCOPE:
                return new float[]{noise() * 0.1f, noise() * 0.1f, noise() * 0.1f};
            case Sensor.TYPE_MAGNETIC_FIELD:
                return new float[]{20.0f + noise(), -30.0f + noise(), 25.0f + noise()};
            case Sensor.TYPE_LIGHT:
                return new float[]{1000.0f};
            case Sensor.TYPE_PROXIMITY:
                return new float[]{8.0f};
            case Sensor.TYPE_GRAVITY:
                return new float[]{noise() * 0.1f, noise() * 0.1f, 9.81f + noise() * 0.1f};
            case Sensor.TYPE_LINEAR_ACCELERATION:
                return new float[]{noise() * 0.1f, noise() * 0.1f, noise() * 0.1f};
            case Sensor.TYPE_ROTATION_VECTOR:
                return new float[]{0.0f, 0.0f, 0.0f, 1.0f};
            case Sensor.TYPE_PRESSURE:
                return new float[]{1013.25f + noise()};
            case Sensor.TYPE_STEP_COUNTER:
                return new float[]{nextSteps()};
            case Sensor.TYPE_STEP_DETECTOR:
                return new float[]{1.0f};
            default:
                return new float[]{0.0f, 0.0f, 0.0f};
        }
    }

    // SensorEvent has no public constructor; allocate without calling one
    // (Unsafe) and inject sensor/values/accuracy/timestamp. Any failure
    // returns null and registerListener still reports true.
    private static Object allocateSensorEvent(Sensor sensor, float[] values) {
        try {
            Class<?> eventClass = Class.forName("android.hardware.SensorEvent");
            Object unsafe = unsafeInstance();
            if (unsafe == null) {
                return null;
            }
            java.lang.reflect.Method allocate =
                    unsafe.getClass().getMethod("allocateInstance", Class.class);
            Object event = allocate.invoke(unsafe, eventClass);
            Legacy.setObjectField(event, "sensor", sensor);
            Legacy.setObjectField(event, "values", values);
            Legacy.setIntField(event, "accuracy", 3);
            try {
                Legacy.setLongField(event, "timestamp", System.nanoTime());
            } catch (Throwable ignored) {
            }
            return event;
        } catch (Throwable t) {
            Legacy.log(TAG + ": allocateSensorEvent failed: " + t);
            return null;
        }
    }

    private static Object unsafeInstance() {
        try {
            Class<?> unsafeClass = Class.forName("sun.misc.Unsafe");
            Field theUnsafe = unsafeClass.getDeclaredField("theUnsafe");
            theUnsafe.setAccessible(true);
            return theUnsafe.get(null);
        } catch (Throwable t) {
            Legacy.log(TAG + ": Unsafe unavailable: " + t);
            return null;
        }
    }

    private static int requestedType(HookFramework.HookChain chain, int idx) {
        try {
            java.util.List<Object> args = chain.args();
            if (args != null && args.size() > idx && args.get(idx) instanceof Integer) {
                return (Integer) args.get(idx);
            }
        } catch (Throwable ignored) { /* fall through */ }
        return Sensor.TYPE_ALL;
    }

    private static List<Sensor> filter(List<Sensor> orig) {
        if (orig == null) return new ArrayList<>();
        List<Sensor> kept = new ArrayList<>(orig.size());
        for (Sensor s : orig) {
            if (s == null) continue;
            if (!isEmulatorSensor(s)) kept.add(s);
        }
        return kept;
    }

    // Append synthetic presence sensors missing from the kept list, scoped
    // to the requested type (TYPE_ALL or one concrete type).
    private static List<Sensor> withSynthetic(List<Sensor> kept, int type) {
        List<Sensor> synthetics = synthetics();
        if (synthetics.isEmpty()) return kept;
        List<Sensor> out = new ArrayList<>(kept);
        for (Sensor s : synthetics) {
            if (type != Sensor.TYPE_ALL && s.getType() != type) continue;
            if (hasType(out, s.getType())) continue;
            out.add(s);
        }
        return out;
    }

    private static boolean hasType(List<Sensor> list, int type) {
        for (Sensor s : list) {
            if (s != null && s.getType() == type) return true;
        }
        return false;
    }

    private static Sensor syntheticFor(int type) {
        for (Sensor s : synthetics()) {
            if (s.getType() == type) return s;
        }
        return null;
    }

    private static synchronized List<Sensor> synthetics() {
        if (syntheticCache != null) return syntheticCache;
        List<Sensor> made = new ArrayList<>(SYNTHETIC.length);
        for (Object[] def : SYNTHETIC) {
            Sensor s = makeSensor((Integer) def[0], (String) def[1], (String) def[2],
                    (String) def[3], (Float) def[4], (Float) def[5]);
            if (s != null) made.add(s);
        }
        syntheticCache = made;
        if (!made.isEmpty()) Legacy.log(TAG + ": synthesized " + made.size() + " sensors");
        return made;
    }

    // Build a presence-only Sensor via the hidden no-arg constructor plus
    // field injection (userdebug permits hidden-API reflection with a
    // warning; everything is guarded so failure just yields no sensors).
    private static Sensor makeSensor(int type, String stringType, String name, String vendor,
            float maxRange, float resolution) {
        try {
            Constructor<?> ctor = Sensor.class.getDeclaredConstructor();
            ctor.setAccessible(true);
            Sensor s = (Sensor) ctor.newInstance();
            setField(s, "mName", name);
            setField(s, "mVendor", vendor);
            setField(s, "mVersion", 1);
            setField(s, "mType", type);
            setField(s, "mStringType", stringType);
            setField(s, "mHandle", -(1000 + type));
            setField(s, "mMinDelay", 5000);
            setField(s, "mMaxDelay", 200000);
            setField(s, "mResolution", resolution);
            setField(s, "mPower", 0.5f);
            setField(s, "mMaxRange", maxRange);
            return s;
        } catch (Throwable t) {
            Legacy.log(TAG + ": sensor synthesis unavailable: " + t);
            return null;
        }
    }

    private static void setField(Object target, String field, Object value) {
        try {
            Field f = Sensor.class.getDeclaredField(field);
            f.setAccessible(true);
            Class<?> ft = f.getType();
            if (ft == int.class) f.setInt(target, ((Number) value).intValue());
            else if (ft == float.class) f.setFloat(target, ((Number) value).floatValue());
            else f.set(target, value);
        } catch (Throwable ignored) { /* field absent on this API level */ }
    }

    private static boolean isEmulatorSensor(Sensor s) {
        String name = nullSafe(s.getName()).toLowerCase();
        String vendor = nullSafe(s.getVendor()).toLowerCase();
        for (String token : DENY) {
            if (name.contains(token) || vendor.contains(token)) return true;
        }
        return false;
    }

    private static String nullSafe(String s) {
        return s == null ? "" : s;
    }
}
