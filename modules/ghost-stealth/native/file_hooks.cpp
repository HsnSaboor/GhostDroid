#include "gs_state.h"

#include <cstdarg>
#include <cstdio>
#include <cstring>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <link.h>
#include <mutex>
#include <string>
#include <sys/stat.h>
#include <unistd.h>
#include <unordered_set>

#include <dobby.h>

namespace gs {

namespace {

std::mutex g_fakedir_mutex;
std::unordered_set<DIR*> g_fakedir_set;

// FORCE_DENYLIST_UNMOUNT hides /data/adb in targets, so the fake files
// live at world-readable paths service.sh maintains.
#define FAKE_MOUNTS "/data/local/tmp/gs_fake_mounts"
#define FAKE_MODULES "/data/local/tmp/gs_fake_modules"
#define FAKE_INPUT "/data/local/tmp/gs_fake_input_devices"
#define FAKE_ASOUND "/data/local/tmp/gs_fake_asound_cards"
#define FAKE_USB "/data/local/tmp/gs_fake_usb_devices"
#define FAKE_PCI_DEVICES "/data/local/tmp/gs_fake_pci_devices"
#define FAKE_PCI_UEVENT "/data/local/tmp/gs_fake_pci_uevent"
#define FAKE_USB_DRIVERS_LIST "/data/local/tmp/gs_fake_usb_drivers_list"
#define FAKE_PCI_DRIVERS_LIST "/data/local/tmp/gs_fake_pci_drivers_list"
#define FAKE_PLATFORM_DRIVERS_LIST "/data/local/tmp/gs_fake_platform_drivers_list"
#define FAKE_SYS_MODULE_LIST "/data/local/tmp/gs_fake_sys_module_list"
#define FAKE_USB_MANUFACTURER "/data/local/tmp/gs_fake_usb_manufacturer"
#define FAKE_USB_PRODUCT "/data/local/tmp/gs_fake_usb_product"
#define FAKE_USB_SERIAL "/data/local/tmp/gs_fake_usb_serial"
#define FAKE_USB_IDVENDOR "/data/local/tmp/gs_fake_usb_idvendor"
#define FAKE_USB_IDPRODUCT "/data/local/tmp/gs_fake_usb_idproduct"
#define FAKE_USB_VERSION "/data/local/tmp/gs_fake_usb_version"
#define FAKE_USB_BUSNUM "/data/local/tmp/gs_fake_usb_busnum"
#define FAKE_USB_DEVNUM "/data/local/tmp/gs_fake_usb_devnum"
#define FAKE_PCI_VENDOR "/data/local/tmp/gs_fake_pci_vendor"
#define FAKE_PCI_DEVICE "/data/local/tmp/gs_fake_pci_device"
// Battery: DeviceInfoHW reads charge_full*/energy_full* from three
// candidate dirs (bms/, battery/, qcom-battery/) + model/manufacturer.
// Serve 5000mAh Li-ion design values; level/status come from the sticky
// broadcast hook (BatteryIntentHooks, Java).
#define FAKE_BAT_CHARGE_FULL "/data/local/tmp/gs_fake_bat_charge_full"
#define FAKE_BAT_CHARGE_FULL_DESIGN \
    "/data/local/tmp/gs_fake_bat_charge_full_design"
#define FAKE_BAT_MODEL "/data/local/tmp/gs_fake_bat_model"
#define FAKE_BAT_MANUFACTURER "/data/local/tmp/gs_fake_bat_manufacturer"

int (*orig_open)(const char*, int, ...) = nullptr;
int (*orig_openat)(int, const char*, int, ...) = nullptr;
FILE* (*orig_fopen)(const char*, const char*) = nullptr;

// opendir/readdir/getdents path: java.io.File.listFiles() uses
// opendir+readdir, bypassing open/openat entirely. DeviceInfoHW USB tab
// (o1/a: /sys/bus/usb/drivers/usb), DRIVERS tab (b1/E:
// /sys/bus/pci/drivers, b1/F: /sys/bus/platform/drivers) all enumerate
// this way. Hook opendir + readdir + getdents64 and serve synthetic
// single-Qualcomm listings for masked trees.
// /sys/devices/pci0000:00 stays REAL (minigbm needs Intel GPU node).
DIR* (*orig_opendir)(const char*) = nullptr;
struct dirent* (*orig_readdir)(DIR*) = nullptr;
int (*orig_closedir)(DIR*) = nullptr;
void (*orig_rewinddir)(DIR*) = nullptr;
long (*orig_telldir)(DIR*) = nullptr;
void (*orig_seekdir)(DIR*, long) = nullptr;
int (*orig_dirfd)(DIR*) = nullptr;

// Runtime.exec("ls ..."/"cat ...") path: DeviceInfoHW falls back to
// `sh -c "ls <dir>"` (t1/a, P0/a) when listFiles returns null. Redirect
// exec of bare ls/cat/getprop over masked paths to `cat <fake>` so the
// shell fallback also sees the synthetic view.
int (*orig_execve)(const char*, char* const[], char* const[]) = nullptr;

// Synthetic single-entry listings (names only; File.isDirectory on them
// falls through to real stat which fails closed -> treated as file; the
// readers only need names to then read node files we redirect by content).
//
// Existence checks (File.exists -> access/fstatat) on masked roots and on
// synthetic child dirs pass through to the REAL fs (dirs really exist, so
// `exists()` stays true and the app takes the sysfs path and reads our
// fakes). Only the LISTING is synthetic. This matters for b1/F.j():
// `new File(.../bt_power).exists()` must stay false->false (we don't fake
// bt_power itself), while masked roots keep their real existence.
const char* MaskedListFile(const char* p) {
    if (p == nullptr) return nullptr;
    if (strcmp(p, "/sys/bus/usb/drivers/usb") == 0 ||
        strcmp(p, "/sys/bus/usb/drivers/usb/") == 0 ||
        strcmp(p, "/sys/bus/usb/devices") == 0 ||
        strcmp(p, "/sys/bus/usb/devices/") == 0)
        return FAKE_USB_DRIVERS_LIST;
    if (strcmp(p, "/sys/bus/pci/drivers") == 0 ||
        strcmp(p, "/sys/bus/pci/drivers/") == 0)
        return FAKE_PCI_DRIVERS_LIST;
    if (strcmp(p, "/sys/bus/platform/drivers") == 0 ||
        strcmp(p, "/sys/bus/platform/drivers/") == 0)
        return FAKE_PLATFORM_DRIVERS_LIST;
    if (strcmp(p, "/sys/module") == 0 || strcmp(p, "/sys/module/") == 0)
        return FAKE_SYS_MODULE_LIST;
    return nullptr;
}

// Covers both exact paths and per-device node paths (DeviceInfoHW reads
// nodes like /sys/bus/usb/devices/1-0:1.0/descriptors and ALSA card dirs).
bool PathStartsWith(const char* p, const char* prefix) {
    if (p == nullptr || prefix == nullptr) return false;
    size_t n = strlen(prefix);
    return strncmp(p, prefix, n) == 0;
}

bool IsMountsPath(const char* p) {
    if (p == nullptr) return false;
    return strcmp(p, "/proc/mounts") == 0 ||
           strcmp(p, "/proc/self/mounts") == 0 ||
           strcmp(p, "/proc/self/mountinfo") == 0 ||
           strcmp(p, "/proc/mountinfo") == 0;
}

// Forward: translator predicate (defined below) reused by the maps filter.
bool IsTranslatorPath(const char* p);

// Translator cloak (EmulatorDetection.cpp:108-132): the ARM bridge must
// stay LOADABLE for games (PUBG runs translated code via Houdini) but
// must be INVISIBLE to detectors. Any open/openat/fopen probe of the
// translator footprint returns ENOENT so fileExists() reads false:
//
//   Houdini: /system/{lib,lib64}/libhoudini.so, /system/bin/{arm,arm64,
//            houdini,houdini64}, /system/{lib,lib64}/{arm,arm64},
//            /system/etc/{binfmt_misc,init/houdini.rc}
//   NDK:     /system/{lib,lib64}/libndk*.so, ndk_translation runners,
//            /system/etc/ld.config.arm{,64}.txt, init/ndk_translation.rc,
//            cpuinfo.arm{,64}.txt
//
// Only the open-family is cloaked (dlopen/mmap of already-open fds still
// works, so translated games keep running). Property side is cloaked in
// spoof.conf (native.bridge=0, arm-only abilist, x86 isa hidden).
// Maps-content views (/proc/self/maps et al.) are filtered separately in
// OpenFilteredMaps: ACE parses maps text, which dl_iterate_phdr can't cover.

bool IsMapsPath(const char* p) {
    if (p == nullptr) return false;
    if (strcmp(p, "/proc/self/maps") == 0 ||
        strcmp(p, "/proc/self/smaps") == 0 ||
        strcmp(p, "/proc/self/smaps_rollup") == 0)
        return true;
    // /proc/self/task/<tid>/{maps,smaps,smaps_rollup}
    static const char kTask[] = "/proc/self/task/";
    if (strncmp(p, kTask, sizeof(kTask) - 1) == 0) {
        const char* slash = strchr(p + sizeof(kTask) - 1, '/');
        if (slash != nullptr && (strcmp(slash, "/maps") == 0 ||
                                 strcmp(slash, "/smaps") == 0 ||
                                 strcmp(slash, "/smaps_rollup") == 0))
            return true;
    }
    // /proc/<pid>/{maps,smaps,smaps_rollup} only when pid == self
    // (detectors build the path via getpid()).
    static const char kProc[] = "/proc/";
    if (strncmp(p, kProc, sizeof(kProc) - 1) == 0) {
        const char* rest = p + sizeof(kProc) - 1;
        long pid = 0;
        size_t i = 0;
        while (rest[i] >= '0' && rest[i] <= '9') {
            pid = pid * 10 + (rest[i] - '0');
            i++;
        }
        if (i > 0 && pid == (long)::getpid()) {
            const char* tail = rest + i;
            if (strcmp(tail, "/maps") == 0 || strcmp(tail, "/smaps") == 0 ||
                strcmp(tail, "/smaps_rollup") == 0)
                return true;
        }
    }
    return false;
}

// Probe-trace path filter: only detection-relevant opens are logged.
bool IsProbePath(const char* p) {
    if (p == nullptr) return false;
    if (IsMapsPath(p)) return true;
    if (PathStartsWith(p, "/dev/__properties__/")) return true;
    if (strcmp(p, "/proc/cpuinfo") == 0 || strcmp(p, "/proc/version") == 0 ||
        strcmp(p, "/proc/modules") == 0 || strcmp(p, "/proc/bus/input/devices") == 0 ||
        strcmp(p, "/proc/asound/cards") == 0 || strcmp(p, "/sys/bus/usb/devices") == 0 ||
        IsMountsPath(p))
        return true;
    if (strstr(p, "sensor") != nullptr || strstr(p, "houdini") != nullptr ||
        strstr(p, "magisk") != nullptr || strstr(p, "xposed") != nullptr ||
        strstr(p, "supersu") != nullptr || strstr(p, "busybox") != nullptr ||
        strstr(p, "/su") != nullptr)
        return true;
    return false;
}

// Max maps bytes filtered inline (1 MiB pipe). Larger -> fail open (real
// fd) so the game never breaks; detectors just see truth in that case.
#define MAPS_FILTER_MAX (1u << 20)

// Open real maps, strip translator lines, serve via pipe. Returns the
// read-end fd, or -1 to fall back to the real open. Single write into a
// pre-grown pipe buffer, so the caller never blocks with no reader yet.
int OpenFilteredMaps(const char* path, int flags) {
    // Read-only views only; writers / O_PATH / O_TMPFILE fall through real.
    if ((flags & (O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND |
                  O_PATH | O_TMPFILE)) != 0)
        return -1;
    if (orig_open == nullptr) return -1;
    int real = orig_open(path, O_RDONLY | O_CLOEXEC, (mode_t)0);
    if (real < 0) return -1;
    std::string data;
    data.reserve(1u << 18);
    char buf[32768];
    size_t total = 0;
    for (;;) {
        ssize_t n = ::read(real, buf, sizeof(buf));
        if (n < 0) {
            ::close(real);
            return -1;
        }
        if (n == 0) break;
        if (total + (size_t)n > MAPS_FILTER_MAX) {
            ::close(real);
            return -1;
        }
        data.append(buf, (size_t)n);
        total += (size_t)n;
    }
    ::close(real);
    std::string out;
    out.reserve(data.size());
    size_t pos = 0;
    while (pos < data.size()) {
        size_t eol = data.find('\n', pos);
        size_t len = (eol == std::string::npos) ? data.size() - pos
                                                : eol - pos + 1;
        std::string line = data.substr(pos, len);
        // Whole-line substring scan: maps lines carry the mapped pathname,
        // so the dl_phdr predicate applies (libraries stay mapped).
        if (!IsTranslatorPath(line.c_str())) out += line;
        pos += len;
    }
    int fds[2];
    if (::pipe(fds) != 0) return -1;
    ::fcntl(fds[0], F_SETPIPE_SZ, (int)MAPS_FILTER_MAX);
    // Never hand out a pipe the payload can't fit: a blocking write past
    // the buffer with no reader yet would wedge the caller mid-startup.
    long cap = ::fcntl(fds[0], F_GETPIPE_SZ);
    if (cap <= 0 || out.size() > (size_t)cap) {
        ::close(fds[0]);
        ::close(fds[1]);
        return -1;
    }
    size_t written = 0;
    while (written < out.size()) {
        ssize_t n = ::write(fds[1], out.data() + written, out.size() - written);
        if (n <= 0) {
            ::close(fds[0]);
            ::close(fds[1]);
            return -1;
        }
        written += (size_t)n;
    }
    ::close(fds[1]);
    if ((flags & O_CLOEXEC) != 0) ::fcntl(fds[0], F_SETFD, FD_CLOEXEC);
    if ((flags & O_NONBLOCK) != 0) {
        int fl = ::fcntl(fds[0], F_GETFL, 0);
        ::fcntl(fds[0], F_SETFL, fl | O_NONBLOCK);
    }
    return fds[0];
}

bool IsTranslatorPath(const char* p) {
    if (p == nullptr) return false;
    if (strstr(p, "libhoudini") != nullptr) return true;
    if (strstr(p, "libndk_translation") != nullptr) return true;
    if (strstr(p, "ndk_translation") != nullptr) return true;
    if (strstr(p, "libnb") != nullptr) return true;
    if (strstr(p, "tango_translator") != nullptr) return true;
    if (strstr(p, "/system/etc/ld.config.arm") != nullptr) return true;
    if (strstr(p, "/system/etc/cpuinfo.arm") != nullptr) return true;
    if (PathStartsWith(p, "/system/bin/arm")) return true;
    if (strcmp(p, "/system/bin/houdini") == 0 ||
        strcmp(p, "/system/bin/houdini64") == 0) return true;
    if (PathStartsWith(p, "/system/lib/arm")) return true;
    if (PathStartsWith(p, "/system/lib64/arm64")) return true;
    if (PathStartsWith(p, "/system/etc/binfmt_misc")) return true;
    if (strcmp(p, "/system/etc/init/houdini.rc") == 0 ||
        strcmp(p, "/system/etc/init/ndk_translation.rc") == 0) return true;
    return false;
}

const char* Redirect(const char* path) {
    if (IsMountsPath(path)) return FAKE_MOUNTS;
    // DRIVERS tab: /proc/modules lists host-only modules (snd_hda_intel,
    // thinkpad_acpi, i2c_*...). Serve a short virtio-only list instead.
    if (path != nullptr && strcmp(path, "/proc/modules") == 0) return FAKE_MODULES;
    // INPUT tab: /proc/bus/input/devices leaks ThinkPad/Elantech/HDA.
    if (path != nullptr && strcmp(path, "/proc/bus/input/devices") == 0) return FAKE_INPUT;
    // AUDIO tab: /proc/asound/cards leaks HDA Intel PCH.
    if (path != nullptr && strcmp(path, "/proc/asound/cards") == 0) return FAKE_ASOUND;
    // USB tab: /sys/bus/usb/devices leaks xhci/cachyos/SunplusIT.
    // Directory listing itself is per-node; mask the uevent/product/vendor
    // strings via the common parent read fallback below.
    if (path != nullptr && strcmp(path, "/sys/bus/usb/devices") == 0) return FAKE_USB;
    // USB enumeration dir (o1/a lists /sys/bus/usb/drivers/usb via
    // File.listFiles -> opendir, bypassing open): serve synthetic list.
    if (path != nullptr && (strcmp(path, "/sys/bus/usb/drivers/usb") == 0 ||
                             strcmp(path, "/sys/bus/usb/drivers/usb/") == 0))
        return FAKE_USB_DRIVERS_LIST;
    // Sysfs node reads: per-device attribute files DeviceInfoHW reads after
    // listing (manufacturer/product/idVendor/... under usb1; vendor/device
    // under the single PCI node). Serve Qualcomm strings; everything else
    // stays real. Content-level only — no global sysfs tmpfs (GPU-safe).
    if (path != nullptr &&
        (PathStartsWith(path, "/sys/bus/usb/drivers/usb/usb1/") ||
         PathStartsWith(path, "/sys/bus/usb/devices/usb1/"))) {
        const char* slash = strrchr(path, '/');
        const char* node = slash != nullptr ? slash + 1 : path;
        if (strcmp(node, "manufacturer") == 0) return FAKE_USB_MANUFACTURER;
        if (strcmp(node, "product") == 0) return FAKE_USB_PRODUCT;
        if (strcmp(node, "serial") == 0) return FAKE_USB_SERIAL;
        if (strcmp(node, "idVendor") == 0) return FAKE_USB_IDVENDOR;
        if (strcmp(node, "idProduct") == 0) return FAKE_USB_IDPRODUCT;
        if (strcmp(node, "version") == 0) return FAKE_USB_VERSION;
        if (strcmp(node, "busnum") == 0) return FAKE_USB_BUSNUM;
        if (strcmp(node, "devnum") == 0) return FAKE_USB_DEVNUM;
    }
    if (path != nullptr &&
        (PathStartsWith(path, "/sys/bus/pci/devices/0000:00:00.0/") ||
         PathStartsWith(path, "/sys/bus/pci/drivers/pcieport/0000:00:00.0/"))) {
        const char* slash = strrchr(path, '/');
        const char* node = slash != nullptr ? slash + 1 : path;
        if (strcmp(node, "vendor") == 0) return FAKE_PCI_VENDOR;
        if (strcmp(node, "device") == 0) return FAKE_PCI_DEVICE;
    }
    // Battery design counters (c1/c reads bms/, battery/, qcom-battery/
    // charge_full* + model/manufacturer). Serve 5000mAh values.
    if (path != nullptr &&
        (strcmp(path, "/sys/class/power_supply/bms/charge_full") == 0 ||
         strcmp(path, "/sys/class/power_supply/battery/charge_full") == 0 ||
         strcmp(path, "/sys/class/power_supply/qcom-battery/charge_full") == 0))
        return FAKE_BAT_CHARGE_FULL;
    if (path != nullptr &&
        (strcmp(path, "/sys/class/power_supply/bms/charge_full_design") == 0 ||
         strcmp(path, "/sys/class/power_supply/battery/charge_full_design") == 0 ||
         strcmp(path, "/sys/class/power_supply/qcom-battery/charge_full_design") == 0))
        return FAKE_BAT_CHARGE_FULL_DESIGN;
    if (path != nullptr &&
        strcmp(path, "/sys/class/power_supply/battery/model_name") == 0)
        return FAKE_BAT_MODEL;
    if (path != nullptr &&
        (strcmp(path, "/sys/class/power_supply/bms/battery_type") == 0 ||
         strcmp(path, "/sys/class/power_supply/battery/manufacturer") == 0))
        return FAKE_BAT_MANUFACTURER;
    // PCI Wi-Fi/iGPU: /proc/bus/pci/devices leaks Intel IDs, pci uevent
    // leaks DRIVER=iwlwifi. Per-process redirect only — NEVER global tmpfs
    // (minigbm/libdrm need real PCI sysfs for Intel GPU BO alloc).
    if (path != nullptr && strcmp(path, "/proc/bus/pci/devices") == 0) return FAKE_PCI_DEVICES;
    if (path != nullptr && PathStartsWith(path, "/sys/bus/pci/devices/") &&
        strlen(path) >= 7 && strcmp(path + strlen(path) - 7, "/uevent") == 0)
        return FAKE_PCI_UEVENT;
    return path;
}

// NOTE: orig_open/openat MUST be int-returning variadic pointers. An early
// revision declared them `bool`, which truncated every fd to 1 and killed
// every target at ART startup (fdsan double-close SIGABRT crash loop).
int my_open(const char* path, int flags, ...) {
    if (IsTranslatorPath(path)) {
        TraceProbeFile("deny", path);
        errno = ENOENT;
        return -1;
    }
    if (IsProbePath(path)) TraceProbeFile("open", path);
    // ACE parses /proc/self/maps text for translator .so names (dl_phdr
    // hook alone can't cover it). Serve the filtered pipe view.
    if (IsMapsPath(path)) {
        int fd = OpenFilteredMaps(path, flags);
        if (fd >= 0) return fd;
    }
    const char* eff = Redirect(path);
    if (eff != path) {
        // Read-only view: strip write/creat so apps can't corrupt the fake.
        flags &= ~(O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND);
        return orig_open(eff, flags | O_RDONLY, (mode_t)0);
    }
    if ((flags & O_CREAT) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode_t mode = va_arg(ap, mode_t);
        va_end(ap);
        return orig_open(path, flags, mode);
    }
    return orig_open(path, flags);
}

int my_openat(int dirfd, const char* path, int flags, ...) {
    if (dirfd == AT_FDCWD && IsTranslatorPath(path)) {
        TraceProbeFile("deny", path);
        errno = ENOENT;
        return -1;
    }
    if (dirfd == AT_FDCWD && IsProbePath(path)) TraceProbeFile("openat", path);
    if (dirfd == AT_FDCWD && IsMountsPath(path)) {
        flags &= ~(O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND);
        return orig_openat(dirfd, FAKE_MOUNTS, flags | O_RDONLY, (mode_t)0);
    }
    if (dirfd == AT_FDCWD && IsMapsPath(path)) {
        int fd = OpenFilteredMaps(path, flags);
        if (fd >= 0) return fd;
    }
    if (dirfd == AT_FDCWD && path != nullptr) {
        // Same fake-file treatment for the extended redirect table.
        const char* eff = Redirect(path);
        if (eff != path) {
            flags &= ~(O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND);
            return orig_openat(dirfd, eff, flags | O_RDONLY, (mode_t)0);
        }
    }
    if ((flags & O_CREAT) != 0) {
        va_list ap;
        va_start(ap, flags);
        mode_t mode = va_arg(ap, mode_t);
        va_end(ap);
        return orig_openat(dirfd, path, flags, mode);
    }
    return orig_openat(dirfd, path, flags);
}

FILE* my_fopen(const char* path, const char* mode) {
    if (IsTranslatorPath(path)) {
        TraceProbeFile("deny", path);
        errno = ENOENT;
        return nullptr;
    }
    if (IsProbePath(path)) TraceProbeFile("fopen", path);
    // Same filtered view for stdio readers of maps (fscanf/fgets loops).
    if (IsMapsPath(path) && mode != nullptr && strchr(mode, 'r') != nullptr &&
        strchr(mode, 'w') == nullptr && strchr(mode, 'a') == nullptr &&
        strchr(mode, '+') == nullptr) {
        int fd = OpenFilteredMaps(path, O_RDONLY);
        if (fd >= 0) {
            FILE* f = ::fdopen(fd, mode);
            if (f != nullptr) return f;
            ::close(fd);
        }
    }
    const char* eff = Redirect(path);
    if (eff != path) {
        // Force read mode; fake mounts are a read-only view.
        if (mode != nullptr && (strchr(mode, 'w') != nullptr ||
                                strchr(mode, 'a') != nullptr ||
                                strchr(mode, '+') != nullptr)) {
            mode = "r";
        }
        return orig_fopen(eff, mode);
    }
    return orig_fopen(path, mode);
}

bool HookExport(const char* sym, void* replace, void** orig) {
    void* addr = DobbySymbolResolver(nullptr, sym);
    if (addr == nullptr) {
        DS_LOGW("DobbySymbolResolver(%s) -> null, skipping", sym);
        return false;
    }
    int rc = DobbyHook(addr, (dobby_dummy_func_t)replace,
                       (dobby_dummy_func_t*)orig);
    DS_LOGI("DobbyHook %s @ %p rc=%d", sym, addr, rc);
    return rc == 0;
}

// --- opendir/readdir/closedir wrappers ----------------------------------
// Serve synthetic single-entry listings for masked sysfs trees. We open
// the fake list FILE (one name per line) at opendir time and synthesize
// dirents from it; "." and ".." first to keep File.listFiles() happy.
//
// DANGER (Bionic): DIR is an opaque libc struct. A forged heap object cast
// to DIR* would SIGSEGV if any unhooked libc helper (dirfd, rewinddir,
// seekdir, telldir, closedir fallback) touches it. Defense in depth:
//   1. Registry membership check (never range check) in readdir/closedir.
//   2. Also hook rewinddir/telldir/seekdir/dirfd/getdents64 and route
//      registered fakes to safe synthetic behavior / EBADF passthrough.
//   3. my_opendir falls back to the real opendir if the fake list file
//      cannot be opened, so we never forge a handle without content.

struct FakeDir {
    FILE* list = nullptr;     // non-null when synthetic
    char current[256];
    struct dirent synth;
    long synth_off = 0;       // virtual offset for telldir/seekdir
    int stage = 0;            // 0=".", 1="..", 2=entries, 3=eof
};

// Live synthetic-handle registry: real DIR* and FakeDir* both come from
// malloc, so identity must be checked by membership, never by range.
// Mutex-guarded; entries added at opendir, removed at closedir.
bool IsFakeDir(DIR* d) {
    std::lock_guard<std::mutex> lk(g_fakedir_mutex);
    return g_fakedir_set.find(d) != g_fakedir_set.end();
}

void RegisterFakeDir(DIR* d) {
    std::lock_guard<std::mutex> lk(g_fakedir_mutex);
    g_fakedir_set.insert(d);
}

void UnregisterFakeDir(DIR* d) {
    std::lock_guard<std::mutex> lk(g_fakedir_mutex);
    g_fakedir_set.erase(d);
}

DIR* my_opendir(const char* path) {
    // Translator dirs (/system/lib/arm, /system/lib64/arm64, binfmt_misc)
    // enumerate as EMPTY so listFilesInDirectory() finds no bridge files.
    // The loader already has the real fds, so games keep running.
    if (IsTranslatorPath(path)) {
        errno = ENOENT;
        return nullptr;
    }
    // Genuine dirs inside a masked tree (e.g. .../drivers/usb/usb1) must
    // NOT be faked: readers open node dirs then read attribute files we
    // redirect by content. Only the exact masked roots get synthesis.
    // NOTE: FakeDir detection relies on pointer identity — FakeDir objects
    // are heap-allocated (never valid DIR* from libc), so my_readdir /
    // my_closedir can distinguish synthetic handles. Track live handles in
    // a small set to avoid misclassifying a real DIR* that happens to
    // overlap heap range (defensive; real DIR* comes from libc malloc too,
    // so check membership, never range).
    const char* listFile = MaskedListFile(path);
    if (listFile != nullptr && orig_fopen != nullptr) {
        FILE* f = orig_fopen(listFile, "r");
        if (f != nullptr) {
            FakeDir* fd = new FakeDir();
            fd->list = f;
            RegisterFakeDir(reinterpret_cast<DIR*>(fd));
            return reinterpret_cast<DIR*>(fd);
        }
    }
    if (orig_opendir != nullptr) return orig_opendir(path);
    return nullptr;
}

struct dirent* my_readdir(DIR* d) {
    if (d == nullptr) return nullptr;
    if (!IsFakeDir(d)) {
        if (orig_readdir != nullptr) return orig_readdir(d);
        return nullptr;
    }
    FakeDir* fd = reinterpret_cast<FakeDir*>(d);
    if (fd->stage == 0) {
        fd->stage = 1;
        memset(&fd->synth, 0, sizeof(fd->synth));
        strcpy(fd->synth.d_name, ".");
        return &fd->synth;
    }
    if (fd->stage == 1) {
        fd->stage = 2;
        memset(&fd->synth, 0, sizeof(fd->synth));
        strcpy(fd->synth.d_name, "..");
        return &fd->synth;
    }
    if (fd->stage == 3) return nullptr;
    char line[256];
    while (fgets(line, sizeof(line), fd->list) != nullptr) {
        size_t n = strlen(line);
        while (n > 0 && (line[n - 1] == '\n' || line[n - 1] == '\r')) line[--n] = '\0';
        if (n == 0) continue;
        snprintf(fd->current, sizeof(fd->current), "%s", line);
        memset(&fd->synth, 0, sizeof(fd->synth));
        snprintf(fd->synth.d_name, sizeof(fd->synth.d_name), "%s", fd->current);
        return &fd->synth;
    }
    fd->stage = 3;
    return nullptr;
}

int my_closedir(DIR* d) {
    if (d == nullptr) return -1;
    if (!IsFakeDir(d)) {
        if (orig_closedir != nullptr) return orig_closedir(d);
        return -1;
    }
    FakeDir* fd = reinterpret_cast<FakeDir*>(d);
    if (fd->list != nullptr) {
        fclose(fd->list);
        UnregisterFakeDir(d);
        delete fd;
        return 0;
    }
    if (orig_closedir != nullptr) return orig_closedir(d);
    return -1;
}

void my_rewinddir(DIR* d) {
    if (d == nullptr) return;
    if (!IsFakeDir(d)) {
        if (orig_rewinddir != nullptr) orig_rewinddir(d);
        return;
    }
    FakeDir* fd = reinterpret_cast<FakeDir*>(d);
    if (fd->list != nullptr) rewind(fd->list);
    fd->stage = 0;
    fd->synth_off = 0;
}

long my_telldir(DIR* d) {
    if (d == nullptr) return -1;
    if (!IsFakeDir(d)) {
        if (orig_telldir != nullptr) return orig_telldir(d);
        return -1;
    }
    return reinterpret_cast<FakeDir*>(d)->synth_off;
}

void my_seekdir(DIR* d, long loc) {
    if (d == nullptr) return;
    if (!IsFakeDir(d)) {
        if (orig_seekdir != nullptr) orig_seekdir(d, loc);
        return;
    }
    // Only rewind-to-start is meaningful on a generated listing; anything
    // else clamps to EOF (safe, no crash, File.listFiles never seeks).
    FakeDir* fd = reinterpret_cast<FakeDir*>(d);
    if (loc <= 0) {
        if (fd->list != nullptr) rewind(fd->list);
        fd->stage = 0;
    } else {
        fd->stage = 3;
    }
    fd->synth_off = loc;
}

int my_dirfd(DIR* d) {
    if (d == nullptr) return -1;
    if (!IsFakeDir(d)) {
        if (orig_dirfd != nullptr) return orig_dirfd(d);
        return -1;
    }
    // Synthetic handle has no fd. Return the list FILE's fd so fstat()
    // callers get *something* valid instead of EBADF surprises; stat-based
    // isDirectory checks on synthetic children still fail closed (by
    // design — readers only need entry names).
    FakeDir* fd = reinterpret_cast<FakeDir*>(d);
    if (fd->list != nullptr) return fileno(fd->list);
    return -1;
}

// --- stat/fstatat/access path: fileExists() in detectors uses fopen,
// but hardened checks use access()/stat(). In Bionic, stat()/lstat()/
// access() all route through fstatat()/faccessat(), so cloaking those
// two covers every existence probe with ENOENT.
int (*orig_fstatat)(int, const char*, struct stat*, int) = nullptr;
int (*orig_faccessat)(int, const char*, int, int) = nullptr;

int my_fstatat(int dirfd, const char* path, struct stat* buf, int flags) {
    if (dirfd == AT_FDCWD && IsTranslatorPath(path)) {
        errno = ENOENT;
        return -1;
    }
    if (orig_fstatat != nullptr)
        return orig_fstatat(dirfd, path, buf, flags);
    errno = ENOSYS;
    return -1;
}

int my_faccessat(int dirfd, const char* path, int mode, int flags) {
    if (dirfd == AT_FDCWD && IsTranslatorPath(path)) {
        errno = ENOENT;
        return -1;
    }
    if (orig_faccessat != nullptr)
        return orig_faccessat(dirfd, path, mode, flags);
    errno = ENOSYS;
    return -1;
}

// --- dl_iterate_phdr filter: EmulatorDetection.cpp:108-115 walks loaded
// modules via dl_phdr_info looking for a "libhoudini.so" substring.
// Filter translator entries out of the enumeration so the check sees a
// clean module list. The libraries stay mapped (games keep running);
// only this process's *view* of the list is filtered.
int (*orig_dl_iterate_phdr)(
        int (*)(struct dl_phdr_info*, size_t, void*), void*) = nullptr;

int my_dl_iterate_phdr(int (*cb)(struct dl_phdr_info*, size_t, void*),
        void* data) {
    if (cb == nullptr) {
        if (orig_dl_iterate_phdr != nullptr)
            return orig_dl_iterate_phdr(cb, data);
        return 0;
    }
    struct FilterCtx {
        int (*cb)(struct dl_phdr_info*, size_t, void*);
        void* data;
    };
    FilterCtx ctx{cb, data};
    auto filtered = [](struct dl_phdr_info* info, size_t size,
            void* vctx) -> int {
        auto* c = reinterpret_cast<FilterCtx*>(vctx);
        if (info != nullptr && info->dlpi_name != nullptr &&
            IsTranslatorPath(info->dlpi_name)) {
            return 0;  // skip: hide translator module from detectors
        }
        return c->cb(info, size, c->data);
    };
    if (orig_dl_iterate_phdr != nullptr)
        return orig_dl_iterate_phdr(filtered, &ctx);
    return 0;
}

// --- execve redirect ------------------------------------------------------
// `sh -c "ls <masked>"` / `"cat <masked node>"` fallbacks (t1/a, P0/a).
// Rewrite argv to `cat <fake list>` so shell probes see the synthetic view.
// Only exact masked roots + bare `ls <root>` / `cat <root>` forms; every
// other exec passes through untouched (app spawn, dex2oat, etc. safe).
int my_execve(const char* path, char* const argv[], char* const envp[]) {
    if (path != nullptr && argv != nullptr && argv[0] != nullptr &&
        orig_execve != nullptr) {
        const char* base = strrchr(path, '/');
        base = base != nullptr ? base + 1 : path;
        if (strcmp(base, "sh") == 0 && argv[1] != nullptr &&
            strcmp(argv[1], "-c") == 0 && argv[2] != nullptr) {
            const char* cmd = argv[2];
            // "ls <dir>" -> "cat <fake list>"
            if (strncmp(cmd, "ls ", 3) == 0) {
                const char* arg = cmd + 3;
                while (*arg == ' ') arg++;
                char target[512];
                size_t i = 0;
                while (*arg != '\0' && *arg != ' ' && i + 1 < sizeof(target)) {
                    if (strncmp(arg, "-all", 4) == 0) { arg += 4; continue; }
                    target[i++] = *arg++;
                }
                target[i] = '\0';
                // "ls -all <dir>" puts flags first; take last token.
                const char* last = strrchr(cmd, ' ');
                if (last != nullptr && last[1] != '\0' && last[1] != '-') {
                    snprintf(target, sizeof(target), "%s", last + 1);
                }
                const char* fake = MaskedListFile(target);
                if (fake != nullptr) {
                    static thread_local char ncmd[768];
                    snprintf(ncmd, sizeof(ncmd), "cat %s", fake);
                    char* nargv[4];
                    nargv[0] = argv[0];
                    nargv[1] = argv[1];
                    nargv[2] = ncmd;
                    nargv[3] = nullptr;
                    return orig_execve(path, nargv, envp);
                }
            }
            // "cat <node>" where node has fake content -> "cat <fake file>"
            if (strncmp(cmd, "cat ", 4) == 0) {
                const char* last = strrchr(cmd, ' ');
                if (last != nullptr && last[1] != '\0') {
                    const char* eff = Redirect(last + 1);
                    if (eff != last + 1) {
                        static thread_local char ncmd[768];
                        snprintf(ncmd, sizeof(ncmd), "cat %s", eff);
                        char* nargv[4];
                        nargv[0] = argv[0];
                        nargv[1] = argv[1];
                        nargv[2] = ncmd;
                        nargv[3] = nullptr;
                        return orig_execve(path, nargv, envp);
                    }
                }
            }
        }
    }
    if (orig_execve != nullptr) return orig_execve(path, argv, envp);
    return -1;
}

}  // namespace

void InstallFileHooks() {
    bool ok_open = HookExport("open", reinterpret_cast<void*>(&my_open),
                              reinterpret_cast<void**>(&orig_open));
    bool ok_openat = HookExport("openat", reinterpret_cast<void*>(&my_openat),
                                reinterpret_cast<void**>(&orig_openat));
    bool ok_fopen = HookExport("fopen", reinterpret_cast<void*>(&my_fopen),
                               reinterpret_cast<void**>(&orig_fopen));
    // NOTE: no open64/openat64 hook. On LP64 open==open64 (same address);
    // hooking the alias second would chain-replace orig_open with my_open
    // and recurse / corrupt fds.
    // Directory enumeration (File.listFiles -> opendir/readdir): DeviceInfoHW
    // USB (o1/a) + DRIVERS (b1/E, b1/F) tabs bypass open/openat entirely.
    bool ok_opendir = HookExport("opendir", reinterpret_cast<void*>(&my_opendir),
                                 reinterpret_cast<void**>(&orig_opendir));
    bool ok_readdir = HookExport("readdir", reinterpret_cast<void*>(&my_readdir),
                                 reinterpret_cast<void**>(&orig_readdir));
    bool ok_closedir = HookExport("closedir", reinterpret_cast<void*>(&my_closedir),
                                  reinterpret_cast<void**>(&orig_closedir));
    // Bionic DIR companions: without these, any dirfd/rewinddir/telldir/
    // seekdir call on a synthetic handle would dereference a forged DIR*
    // and SIGSEGV. Route registered fakes to safe synthetic behavior.
    bool ok_rewinddir = HookExport("rewinddir", reinterpret_cast<void*>(&my_rewinddir),
                                   reinterpret_cast<void**>(&orig_rewinddir));
    bool ok_telldir = HookExport("telldir", reinterpret_cast<void*>(&my_telldir),
                                 reinterpret_cast<void**>(&orig_telldir));
    bool ok_seekdir = HookExport("seekdir", reinterpret_cast<void*>(&my_seekdir),
                                 reinterpret_cast<void**>(&orig_seekdir));
    bool ok_dirfd = HookExport("dirfd", reinterpret_cast<void*>(&my_dirfd),
                               reinterpret_cast<void**>(&orig_dirfd));
    // Shell fallback (`sh -c "ls <dir>"` via Runtime.exec, t1/a + P0/a):
    // rewrite masked ls/cat to serve fake lists.
    bool ok_execve = HookExport("execve", reinterpret_cast<void*>(&my_execve),
                                reinterpret_cast<void**>(&orig_execve));
    bool ok_fstatat = HookExport("fstatat",
                                  reinterpret_cast<void*>(&my_fstatat),
                                  reinterpret_cast<void**>(&orig_fstatat));
    bool ok_faccessat = HookExport("faccessat",
                                   reinterpret_cast<void*>(&my_faccessat),
                                   reinterpret_cast<void**>(&orig_faccessat));
    // Loaded-module enumeration (dl_phdr_info substring "libhoudini.so"):
    // filter translator entries so checkArmTranslation sees a clean list.
    bool ok_phdr = HookExport("dl_iterate_phdr",
                              reinterpret_cast<void*>(&my_dl_iterate_phdr),
                              reinterpret_cast<void**>(&orig_dl_iterate_phdr));
    DS_LOGI("file hooks: open=%d openat=%d fopen=%d opendir=%d readdir=%d closedir=%d rewinddir=%d telldir=%d seekdir=%d dirfd=%d execve=%d fstatat=%d faccessat=%d phdr=%d",
            ok_open, ok_openat, ok_fopen,
            ok_opendir, ok_readdir, ok_closedir,
            ok_rewinddir, ok_telldir, ok_seekdir, ok_dirfd, ok_execve,
            ok_fstatat, ok_faccessat, ok_phdr);
}

}  // namespace gs
