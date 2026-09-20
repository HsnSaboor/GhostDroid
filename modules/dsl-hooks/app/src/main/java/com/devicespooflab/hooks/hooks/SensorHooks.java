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

    // {type, stringType, name, vendor}
    private static final Object[][] SYNTHETIC = {
        {Sensor.TYPE_ACCELEROMETER, "android.sensor.accelerometer", "S26 Accelerometer", "Samsung"},
        {Sensor.TYPE_GYROSCOPE, "android.sensor.gyroscope", "S26 Gyroscope", "Samsung"},
        {Sensor.TYPE_MAGNETIC_FIELD, "android.sensor.magnetic_field", "S26 Magnetic Field", "Samsung"},
        {Sensor.TYPE_PROXIMITY, "android.sensor.proximity", "S26 Proximity", "Samsung"},
        {Sensor.TYPE_LIGHT, "android.sensor.light", "S26 Light", "Samsung"},
        {Sensor.TYPE_GRAVITY, "android.sensor.gravity", "S26 Gravity", "Samsung"},
        {Sensor.TYPE_LINEAR_ACCELERATION, "android.sensor.linear_acceleration",
                "S26 Linear Acceleration", "Samsung"},
        {Sensor.TYPE_ROTATION_VECTOR, "android.sensor.rotation_vector",
                "S26 Rotation Vector", "Samsung"},
    };

    private static List<Sensor> syntheticCache = null;

    public static void hook(HookContext lpparam) {
        try {
            Legacy.findAndHookMethod(SensorManager.class, "getSensorList",
                    int.class,
                    new HookFramework.Hook() {@Override
                        @SuppressWarnings("unchecked")
                        public void after(HookFramework.HookChain chain, Object result, Throwable error) {
                            List<Sensor> orig = (List<Sensor>) result;
                            java.util.List<Object> args = chain.args();
                            int type = (args != null && !args.isEmpty() && args.get(0) instanceof Integer)
                                    ? (Integer) args.get(0) : Sensor.TYPE_ALL;
                            chain.replaceResult(withSynthetic(filter(orig), type));
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
                            Sensor s = (Sensor) result;
                            if (s != null && isEmulatorSensor(s)) {
                                s = null;
                            }
                            if (s == null) {
                                Sensor synth = syntheticFor(requestedType(chain, 0));
                                if (synth != null) chain.replaceResult(synth);
                                else if (result != null) chain.replaceResult(null);
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
                            Sensor s = (Sensor) result;
                            if (s != null && isEmulatorSensor(s)) {
                                s = null;
                            }
                            if (s == null) {
                                Sensor synth = syntheticFor(requestedType(chain, 0));
                                if (synth != null) chain.replaceResult(synth);
                                else if (result != null) chain.replaceResult(null);
                            }
                        }
                    });
        } catch (Throwable t) { /* version-specific overload */ }
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
                    (String) def[3]);
            if (s != null) made.add(s);
        }
        syntheticCache = made;
        if (!made.isEmpty()) Legacy.log(TAG + ": synthesized " + made.size() + " sensors");
        return made;
    }

    // Build a presence-only Sensor via the hidden no-arg constructor plus
    // field injection (userdebug permits hidden-API reflection with a
    // warning; everything is guarded so failure just yields no sensors).
    private static Sensor makeSensor(int type, String stringType, String name, String vendor) {
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
            setField(s, "mResolution", 0.01f);
            setField(s, "mPower", 0.5f);
            setField(s, "mMaxRange", 40.0f);
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
