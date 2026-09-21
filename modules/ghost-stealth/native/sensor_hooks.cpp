// sensor_hooks.cpp: native NDK sensor spoof (ASensorManager).
//
// Java SensorHooks covers android.hardware.SensorManager, but ACE reads
// sensors natively via ASensorManager_getSensorList (libcubehawk links
// only __system_property_get/open/fopen — the GPU/battery/sensor signal
// arrives through NDK + sysfs, not libc strings). Waydroid exposes ZERO
// NDK sensors; an empty list is a textbook emulator tell.
//
// Synthesize a phone-plausible list: return S26-Ultra vendor/name
// pointers for the standard types, backed by static storage (never freed,
// fail-open: null list on any error keeps the original).
//
// Resolver-only: DobbySymbolResolver over already-loaded exports, zero
// linker work, safe from preAppSpecialize. Fail-closed identity,
// fail-open rendering (callers only read name/vendor/type).

#include "gs_state.h"

#include <cstring>

#include <dobby.h>

namespace gs {

namespace {

struct FakeSensor {
    const char* name;
    const char* vendor;
    int type;
};

// S26 Ultra style set: STMicro LSM6DSV IMU + AKM magnetometer (S25 Ultra
// teardown reference), Samsung software sensors. Mirrors the Java
// SensorHooks SYNTHETIC table — both layers must agree.
const FakeSensor kFakeSensors[] = {
    {"LSM6DSV Accelerometer", "STMicroelectronics", 1},   // ASENSOR_TYPE_ACCELEROMETER
    {"LSM6DSV Gyroscope", "STMicroelectronics", 4},       // ASENSOR_TYPE_GYROSCOPE
    {"AK09918 Magnetometer", "AKM", 2},                   // ASENSOR_TYPE_MAGNETIC_FIELD
    {"S26 Proximity", "Samsung", 8},                      // ASENSOR_TYPE_PROXIMITY
    {"S26 Light", "Samsung", 5},                          // ASENSOR_TYPE_LIGHT
    {"S26 Gravity", "Samsung", 9},                        // ASENSOR_TYPE_GRAVITY
    {"S26 Linear Acceleration", "Samsung", 10},           // ASENSOR_TYPE_LINEAR_ACCELERATION
    {"S26 Rotation Vector", "Samsung", 11},               // ASENSOR_TYPE_ROTATION_VECTOR
    {"S26 Pressure", "Samsung", 6},                       // ASENSOR_TYPE_PRESSURE
    {"S26 Step Counter", "Samsung", 19},                  // ASENSOR_TYPE_STEP_COUNTER
};
constexpr int kFakeSensorCount =
    (int)(sizeof(kFakeSensors) / sizeof(kFakeSensors[0]));

// Opaque handle table: callers treat entries as const void*; index into
// the static array. Stable for process lifetime.
const void* g_sensorHandles[sizeof(kFakeSensors) / sizeof(kFakeSensors[0])];
bool g_sensorTableInit = false;

void EnsureSensorTable() {
    if (g_sensorTableInit) return;
    for (int i = 0; i < kFakeSensorCount; i++) {
        g_sensorHandles[i] = (const void*)&kFakeSensors[i];
    }
    g_sensorTableInit = true;
}

using GetListFn = int (*)(void*, const void***);
GetListFn g_orig_get_sensor_list = nullptr;

int my_ASensorManager_getSensorList(void* manager, const void*** list) {
    // Cheap: serve the synthetic list when the host reports empty.
    // Non-empty host lists pass through (fail-open: real data untouched).
    if (g_orig_get_sensor_list == nullptr) return 0;
    int n = g_orig_get_sensor_list(manager, list);
    if (n > 0) return n;
    EnsureSensorTable();
    if (list != nullptr) *list = g_sensorHandles;
    return kFakeSensorCount;
}

using GetVendorFn = const char* (*)(const void*);
using GetNameFn = const char* (*)(const void*);
GetVendorFn g_orig_get_vendor = nullptr;
GetNameFn g_orig_get_name = nullptr;

bool IsFakeHandle(const void* s) {
    if (s == nullptr) return false;
    for (int i = 0; i < kFakeSensorCount; i++) {
        if (s == (const void*)&kFakeSensors[i]) return true;
    }
    return false;
}

const char* my_ASensor_getVendor(const void* sensor) {
    if (IsFakeHandle(sensor)) {
        return ((const FakeSensor*)sensor)->vendor;
    }
    if (g_orig_get_vendor != nullptr) return g_orig_get_vendor(sensor);
    return nullptr;
}

const char* my_ASensor_getName(const void* sensor) {
    if (IsFakeHandle(sensor)) {
        return ((const FakeSensor*)sensor)->name;
    }
    if (g_orig_get_name != nullptr) return g_orig_get_name(sensor);
    return nullptr;
}

bool TryHookOne(const char* sym, void* replace, void** orig) {
    // Reentrancy guard: DobbySymbolResolver opens /proc/self/maps via
    // fopen internally. Our my_fopen/my_open retry the lazy hooks, so an
    // unguarded resolve recurses (resolver -> fopen -> resolve -> ...)
    // until the stack overflows (verified tombstone 2026-09-22:
    // TryHookSensorResolved <-> GetProcessModuleMap cycle). Skip the
    // resolve while already inside one; the next probe retries.
    static thread_local bool s_in_resolve = false;
    if (s_in_resolve) return false;
    void* addr = nullptr;
    {
        struct Guard {
            bool& f;
            Guard(bool& f) : f(f) { f = true; }
            ~Guard() { f = false; }
        } g(s_in_resolve);
        addr = DobbySymbolResolver(nullptr, sym);
    }
    if (addr == nullptr) return false;
    if (*orig != nullptr) return true;
    return DobbyHook(addr, (dobby_dummy_func_t)replace,
                     (dobby_dummy_func_t*)orig) == 0;
}

}  // namespace

void InstallSensorHooks() {
    // Idempotent: TryHookOne short-circuits once hooked. Misses when
    // libandroid.so is not mapped yet — lazy retry lands it later.
    bool ok_list = TryHookOne("ASensorManager_getSensorList",
                              (void*)&my_ASensorManager_getSensorList,
                              (void**)&g_orig_get_sensor_list);
    bool ok_vendor = TryHookOne("ASensor_getVendor",
                                (void*)&my_ASensor_getVendor,
                                (void**)&g_orig_get_vendor);
    bool ok_name = TryHookOne("ASensor_getName", (void*)&my_ASensor_getName,
                              (void**)&g_orig_get_name);
    DS_LOGI("sensor hooks: list=%d vendor=%d name=%d", ok_list ? 1 : 0,
            ok_vendor ? 1 : 0, ok_name ? 1 : 0);
}

bool TryHookSensorResolved() {
    if (g_orig_get_sensor_list != nullptr) return true;
    void* sym = DobbySymbolResolver(nullptr, "ASensorManager_getSensorList");
    if (sym == nullptr) return false;
    InstallSensorHooks();
    return g_orig_get_sensor_list != nullptr;
}

}  // namespace gs
