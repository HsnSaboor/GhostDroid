#include "gs_state.h"

#include <cstdarg>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <link.h>
#include <mutex>
#include <string>
#include <sys/inotify.h>
#include <sys/ptrace.h>
#include <sys/system_properties.h>
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
// Raw sysfs device tree + Adreno counters (probe log: ACE reads
// /sys/devices/pci0000:00/0000:00:02.0/* directly, bypassing the
// /sys/bus/pci fakes; kgsl gpubusy absence = no-Adreno tell).
#define FAKE_PCI_DEV_UEVENT "/data/local/tmp/gs_fake_pci_dev_uevent"
#define FAKE_PCI_DEV_MODALIAS "/data/local/tmp/gs_fake_pci_dev_modalias"
#define FAKE_KGSL_GPUBUSY "/data/local/tmp/gs_fake_kgsl_gpubusy"
// Battery: DeviceInfoHW reads charge_full*/energy_full* from three
// candidate dirs (bms/, battery/, qcom-battery/) + model/manufacturer.
// Serve 5000mAh Li-ion design values; level/status come from the sticky
// broadcast hook (BatteryIntentHooks, Java).
#define FAKE_BAT_CHARGE_FULL "/data/local/tmp/gs_fake_bat_charge_full"
#define FAKE_BAT_CHARGE_FULL_DESIGN \
    "/data/local/tmp/gs_fake_bat_charge_full_design"
#define FAKE_BAT_MODEL "/data/local/tmp/gs_fake_bat_model"
#define FAKE_BAT_MANUFACTURER "/data/local/tmp/gs_fake_bat_manufacturer"
// Probe log (PUBG pid 3164) direct-read tells: /proc/cpuinfo (Intel, global
// bind misses the target mount ns), /system/build.prop (41x, bypasses prop
// hooks), host kernel truth (/proc/kallsyms+iomem+ioports+misc), per-cpu
// cpufreq counters (host kHz, not Oryon) + cpu online topology.
#define FAKE_CPUINFO "/data/local/tmp/gs_fake_cpuinfo"
#define FAKE_BUILD_PROP "/data/local/tmp/gs_fake_build_prop"
#define FAKE_CPUFREQ_PRIME "/data/local/tmp/gs_fake_cpufreq_prime"
#define FAKE_CPUFREQ_PERF "/data/local/tmp/gs_fake_cpufreq_perf"
#define FAKE_CPUFREQ_MIN "/data/local/tmp/gs_fake_cpufreq_min"
#define FAKE_CPU_ONLINE "/data/local/tmp/gs_fake_cpu_online"
#define FAKE_KALLSYMS "/data/local/tmp/gs_fake_kallsyms"
#define FAKE_IOMEM "/data/local/tmp/gs_fake_iomem"
#define FAKE_IOPORTS "/data/local/tmp/gs_fake_ioports"
#define FAKE_PROC_MISC "/data/local/tmp/gs_fake_proc_misc"

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
// Root file-cloak + hook-framework predicates (defined below, next to the
// translator cloak so all deny/filter tables stay in one place).
bool IsRootPath(const char* p);
bool IsHookFrameworkPath(const char* p);
bool IsHiddenModuleLine(const char* p);
bool IsStatusPath(const char* p);

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

// Numeric pid segment matcher (no allocation): sets lenOut to the digit
// run length. Used by the /proc pid-view predicate below.
bool IsNumericPid(const char* s, size_t* lenOut) {
    if (s == nullptr) return false;
    size_t i = 0;
    while (s[i] >= '0' && s[i] <= '9') i++;
    if (lenOut != nullptr) *lenOut = i;
    return i > 0;
}

// /proc pid-view predicate (DRY, shared by maps + status filters):
// matches /proc/self/<view>, /proc/self/task/<tid>/<view>,
// /proc/<pid>/<view>, /proc/<pid>/task/<tid>/<view> for ANY numeric pid.
// Widened from self-only: probe log shows reads of a sibling's maps
// (/proc/4885/maps). Only translator/framework lines are stripped (maps)
// or TracerPid zeroed (status), so other-pid views stay truthful
// otherwise; oversized views still fail open.
bool IsProcPidView(const char* p, const char* view) {
    if (p == nullptr || view == nullptr || *view != '/') return false;
    static const char kProc[] = "/proc/";
    if (strncmp(p, kProc, sizeof(kProc) - 1) != 0) return false;
    const char* rest = p + sizeof(kProc) - 1;
    if (strncmp(rest, "self/", 5) == 0) {
        const char* after = rest + 5;
        if (strcmp(after, view + 1) == 0) return true;
        static const char kTask[] = "task/";
        if (strncmp(after, kTask, sizeof(kTask) - 1) != 0) return false;
        const char* tid = after + sizeof(kTask) - 1;
        size_t n = 0;
        if (!IsNumericPid(tid, &n)) return false;
        return tid[n] == '/' && strcmp(tid + n, view) == 0;
    }
    size_t n = 0;
    if (!IsNumericPid(rest, &n)) return false;
    const char* after = rest + n;
    if (strcmp(after, view) == 0) return true;
    static const char kTask[] = "/task/";
    if (strncmp(after, kTask, sizeof(kTask) - 1) != 0) return false;
    const char* tid = after + sizeof(kTask) - 1;
    size_t m = 0;
    if (!IsNumericPid(tid, &m)) return false;
    return tid[m] == '/' && strcmp(tid + m, view) == 0;
}

bool IsMapsPath(const char* p) {
    return IsProcPidView(p, "/maps") || IsProcPidView(p, "/smaps") ||
           IsProcPidView(p, "/smaps_rollup");
}

// (Removed: probe tracing now logs every open unconditionally.)

// Max maps bytes filtered inline (1 MiB pipe). Larger -> fail open (real
// fd) so the game never breaks; detectors just see truth in that case.
#define MAPS_FILTER_MAX (1u << 20)

// Open real maps, strip translator lines, serve via pipe. Returns the
// read-end fd, or -1 to fall back to the real open. Single write into a
// pre-grown pipe buffer, so the caller never blocks with no reader yet.
// Pipe-backed read-end serving `content` (shared by the maps + status
// filters). Returns the read-end fd, or -1 to fail open to the real file.
// Never blocks: refuses payloads larger than the grown pipe buffer.
int PipeReadEndFromString(const std::string& content) {
    int fds[2];
    if (::pipe(fds) != 0) return -1;
    ::fcntl(fds[0], F_SETPIPE_SZ, (int)MAPS_FILTER_MAX);
    long cap = ::fcntl(fds[0], F_GETPIPE_SZ);
    if (cap <= 0 || content.size() > (size_t)cap) {
        ::close(fds[0]);
        ::close(fds[1]);
        return -1;
    }
    size_t written = 0;
    while (written < content.size()) {
        ssize_t n = ::write(fds[1], content.data() + written,
                            content.size() - written);
        if (n <= 0) {
            ::close(fds[0]);
            ::close(fds[1]);
            return -1;
        }
        written += (size_t)n;
    }
    ::close(fds[1]);
    return fds[0];
}

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
        if (!IsHiddenModuleLine(line.c_str())) out += line;
        pos += len;
    }
    // Never hand out a pipe the payload can't fit: a blocking write past
    // the buffer with no reader yet would wedge the caller mid-startup
    // (helper fails open with -1 in that case).
    int fd = PipeReadEndFromString(out);
    if (fd < 0) return -1;
    if ((flags & O_CLOEXEC) != 0) ::fcntl(fd, F_SETFD, FD_CLOEXEC);
    if ((flags & O_NONBLOCK) != 0) {
        int fl = ::fcntl(fd, F_GETFL, 0);
        ::fcntl(fd, F_SETFL, fl | O_NONBLOCK);
    }
    return fd;
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
    // x86_64 framework oat (probe log: opens of oat/x86_64/*.art+.vdex).
    // Real S26 Ultra ships arm64-only oat; the x86_64 boot image here is a
    // host-runtime artifact. The runtime holds its fds (zygote-inherited),
    // so denying fresh existence/content probes only blinds detectors.
    if (strstr(p, "oat/x86_64") != nullptr) return true;
    return false;
}

// Basename of a path (no allocation): pointer into p after the last '/'.
const char* Basename(const char* p) {
    if (p == nullptr) return nullptr;
    const char* slash = strrchr(p, '/');
    return slash != nullptr ? slash + 1 : p;
}

// Case-insensitive substring search (no allocation): framework .so names
// appear in mixed case in the wild (XposedBridge, LSPosed, Magisk...).
bool ContainsCI(const char* hay, const char* needle) {
    if (hay == nullptr || needle == nullptr || *needle == '\0') return false;
    for (const char* h = hay; *h != '\0'; h++) {
        const char* hh = h;
        const char* nn = needle;
        while (*nn != '\0' && *hh != '\0' &&
               tolower((unsigned char)*hh) == tolower((unsigned char)*nn)) {
            hh++;
            nn++;
        }
        if (*nn == '\0') return true;
    }
    return false;
}

// Root file-cloak table (fail-CLOSED with ENOENT, mirroring the translator
// cloak above): any existence/open probe of a root tell reads as absent.
//
//   exact:  /system/xbin/su, /system/bin/su, /sbin/su, /su/bin/su,
//           /dev/qemu_pipe, /dev/socket/qemud, goldfish nodes,
//           hawk emulator props/bins/libs, hardware_info.txt
//   prefix: /data/adb/* (magisk dir, modules, ...)
//   basename: su, busybox, daemonsu, Superuser.apk
//   substring: magisk pkgs, eu.chainfire.supersu
//
// Explicitly NOT matched: libbluetooth_jni.so / libvulkan.so (guarded first
// so a future substring can never swallow real game libraries).
bool IsRootPath(const char* p) {
    if (p == nullptr || *p == '\0') return false;
    if (strstr(p, "libbluetooth_jni.so") != nullptr) return false;
    if (strstr(p, "libvulkan.so") != nullptr) return false;
    if (strcmp(p, "/system/xbin/su") == 0 ||
        strcmp(p, "/system/bin/su") == 0 ||
        strcmp(p, "/sbin/su") == 0 ||
        strcmp(p, "/su/bin/su") == 0 ||
        strcmp(p, "/dev/qemu_pipe") == 0 ||
        strcmp(p, "/dev/socket/qemud") == 0 ||
        strcmp(p, "/data/share1/hardware_info.txt") == 0 ||
        strcmp(p, "/system/bin/qemu_props") == 0 ||
        strcmp(p, "/system/bin/androVM-prop") == 0 ||
        strcmp(p, "/system/bin/microvirt-prop") == 0 ||
        strcmp(p, "/system/bin/microvirtd") == 0 ||
        strcmp(p, "/system/bin/nox-prop") == 0 ||
        strcmp(p, "/system/bin/ttVM-prop") == 0 ||
        strcmp(p, "/system/bin/droid4x-prop") == 0 ||
        strcmp(p, "/system/bin/windroyed") == 0 ||
        strcmp(p, "/system/lib/libdroid4x.so") == 0 ||
        strcmp(p, "/system/lib64/libdroid4x.so") == 0 ||
        strcmp(p, "/system/lib/libc_malloc_debug_qemu.so") == 0 ||
        strcmp(p, "/system/lib64/libc_malloc_debug_qemu.so") == 0)
        return true;
    if (strcmp(p, "/data/adb") == 0 ||
        PathStartsWith(p, "/data/adb/"))
        return true;
    const char* base = Basename(p);
    if (strcmp(base, "su") == 0 ||
        strcmp(base, "busybox") == 0 ||
        strcmp(base, "daemonsu") == 0 ||
        strcmp(base, "Superuser.apk") == 0)
        return true;
    if (strstr(p, "magisk") != nullptr) return true;
    if (strstr(p, "eu.chainfire.supersu") != nullptr) return true;
    if (strstr(p, "goldfish") != nullptr) return true;
    if (strstr(p, "qemu_props") != nullptr ||
        strstr(p, "androVM-prop") != nullptr ||
        strstr(p, "microvirt-prop") != nullptr ||
        strstr(p, "microvirtd") != nullptr ||
        strstr(p, "nox-prop") != nullptr ||
        strstr(p, "ttVM-prop") != nullptr ||
        strstr(p, "droid4x-prop") != nullptr ||
        strstr(p, "windroyed") != nullptr ||
        strstr(p, "libdroid4x") != nullptr ||
        strstr(p, "libc_malloc_debug_qemu") != nullptr ||
        strstr(p, "hardware_info.txt") != nullptr ||
        strstr(p, "qemu_pipe") != nullptr ||
        strstr(p, "qemud") != nullptr)
        return true;
    return false;
}

// Hook-framework module predicate for the maps/phdr content filters
// (fail-OPEN: oversized views fall back to real fds, games never break).
// Covers xposed / lsposed / lspd / zygisk / shamiko / riru / frida /
// substrate / edxposed / magisk / lsplant. Our own libs (gs_native, dobby,
// lsplt) are guarded first so we never hide ourselves from our own view.
bool IsHookFrameworkPath(const char* p) {
    if (p == nullptr) return false;
    if (strstr(p, "gs_native") != nullptr) return false;
    if (strstr(p, "libdobby") != nullptr) return false;
    if (strstr(p, "dobby") != nullptr) return false;
    if (strstr(p, "lsplt") != nullptr) return false;
    if (ContainsCI(p, "xposed")) return true;
    if (ContainsCI(p, "lsposed")) return true;
    if (ContainsCI(p, "lspd")) return true;
    if (ContainsCI(p, "zygisk")) return true;
    if (ContainsCI(p, "shamiko")) return true;
    if (ContainsCI(p, "riru")) return true;
    if (ContainsCI(p, "frida")) return true;
    if (ContainsCI(p, "substrate")) return true;
    if (ContainsCI(p, "edxposed")) return true;
    if (ContainsCI(p, "magisk")) return true;
    if (ContainsCI(p, "lsplant")) return true;
    return false;
}

// Single maps/phdr predicate (DRY): translator cloak + hook frameworks.
bool IsHiddenModuleLine(const char* p) {
    return IsTranslatorPath(p) || IsHookFrameworkPath(p);
}

// TracerPid/status views eligible for the pipe-backed rewrite (any pid:
// detectors also read siblings, e.g. /proc/4885/maps; TracerPid of other
// pids is zeroed the same way, everything else passes through verbatim).
bool IsStatusPath(const char* p) {
    return IsProcPidView(p, "/status");
}



// TracerPid/caps/seccomp pipe filter: read the real status, normalize
// debugger + container tells to stock Pixel app values, serve via pipe
// (fail-open: -1 hands back to the real open). Read-only callers only.
//
//   TracerPid -> 0 (anti-debug; probe log: 6x status reads)
//   CapInh/CapPrm/CapEff/CapBnd/CapAmb -> 0 (Waydroid container may carry
//     ambient/bounding caps a stock phone drops for apps; 0 is the app norm)
//   NoNewPrivs -> 1, Seccomp -> 2, Seccomp_filters -> 1 (stock app sandbox;
//     a container with Seccomp: 0 would betray the VMM stack)
// Name/State/Uid/Gid pass through (u0_aXXX app identity is normal).
int OpenFilteredStatus(const char* path, int flags) {
    if ((flags & (O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND |
                  O_PATH | O_TMPFILE)) != 0)
        return -1;
    if (orig_open == nullptr) return -1;
    int real = orig_open(path, O_RDONLY | O_CLOEXEC, (mode_t)0);
    if (real < 0) return -1;
    std::string data;
    data.reserve(4096);
    char buf[1024];
    size_t total = 0;
    for (;;) {
        ssize_t n = ::read(real, buf, sizeof(buf));
        if (n < 0) {
            ::close(real);
            return -1;
        }
        if (n == 0) break;
        if (total + (size_t)n > 65536) {
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
        bool has_nl = (eol != std::string::npos);
        size_t len = has_nl ? eol - pos + 1 : data.size() - pos;
        std::string line = data.substr(pos, len);
        if (strncmp(line.c_str(), "TracerPid:", 10) == 0) {
            out += "TracerPid:\t0";
            out += "\n";
        } else if (strncmp(line.c_str(), "CapInh:", 7) == 0 ||
                   strncmp(line.c_str(), "CapPrm:", 7) == 0 ||
                   strncmp(line.c_str(), "CapEff:", 7) == 0 ||
                   strncmp(line.c_str(), "CapBnd:", 7) == 0 ||
                   strncmp(line.c_str(), "CapAmb:", 7) == 0) {
            out.append(line.c_str(), 7);
            out += "\t0000000000000000\n";
        } else if (strncmp(line.c_str(), "NoNewPrivs:", 11) == 0) {
            out += "NoNewPrivs:\t1\n";
        } else if (strncmp(line.c_str(), "Seccomp_filters:", 16) == 0) {
            out += "Seccomp_filters:\t1\n";
        } else if (strncmp(line.c_str(), "Seccomp:", 8) == 0) {
            out += "Seccomp:\t2\n";
        } else {
            out += line;
        }
        pos += len;
    }
    int fd = PipeReadEndFromString(out);
    if (fd < 0) return -1;
    if ((flags & O_CLOEXEC) != 0) ::fcntl(fd, F_SETFD, FD_CLOEXEC);
    if ((flags & O_NONBLOCK) != 0) {
        int fl = ::fcntl(fd, F_GETFL, 0);
        ::fcntl(fd, F_SETFL, fl | O_NONBLOCK);
    }
    return fd;
}

// Oryon V3 (SM8850) cpufreq view: cpu0-1 prime @4.74GHz, cpu2-7 perf
// @3.62GHz (matches the service.sh global mask). Only freq-counter nodes
// are faked; governor/policy dirs stay real so DVFS readers never break.
// Unknown nodes and cpu>=8 fall through (fail-open).
const char* CpufreqFake(const char* path) {
    static const char kPrefix[] = "/sys/devices/system/cpu/";
    if (!PathStartsWith(path, kPrefix)) return nullptr;
    const char* rest = path + sizeof(kPrefix) - 1;  // "cpu<N>/cpufreq/<node>"
    if (rest[0] != 'c' || rest[1] != 'p' || rest[2] != 'u') return nullptr;
    const char* d = rest + 3;
    int cpu = 0;
    int digits = 0;
    while (*d >= '0' && *d <= '9') {
        cpu = cpu * 10 + (*d - '0');
        d++;
        digits++;
    }
    if (digits == 0 || cpu < 0 || cpu > 7) return nullptr;
    static const char kFreq[] = "/cpufreq/";
    if (strncmp(d, kFreq, sizeof(kFreq) - 1) != 0) return nullptr;
    const char* node = d + sizeof(kFreq) - 1;
    if (strcmp(node, "scaling_cur_freq") == 0 ||
        strcmp(node, "cpuinfo_max_freq") == 0 ||
        strcmp(node, "scaling_max_freq") == 0)
        return cpu < 2 ? FAKE_CPUFREQ_PRIME : FAKE_CPUFREQ_PERF;
    if (strcmp(node, "cpuinfo_min_freq") == 0 ||
        strcmp(node, "scaling_min_freq") == 0)
        return FAKE_CPUFREQ_MIN;
    return nullptr;
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
    // Probe log (PUBG 3164) direct-read tells, per-process only:
    // /proc/cpuinfo leaks Intel (global bind misses the target mount ns);
    // /system/build.prop (41x) bypasses the prop hooks; kallsyms/iomem/
    // ioports/misc expose x86 ranges + Waydroid devices; cpufreq counters
    // report host kHz instead of Oryon V3 clocks.
    if (path != nullptr && strcmp(path, "/proc/cpuinfo") == 0) return FAKE_CPUINFO;
    if (path != nullptr && strcmp(path, "/system/build.prop") == 0) return FAKE_BUILD_PROP;
    if (path != nullptr && strcmp(path, "/proc/kallsyms") == 0) return FAKE_KALLSYMS;
    if (path != nullptr && strcmp(path, "/proc/iomem") == 0) return FAKE_IOMEM;
    if (path != nullptr && strcmp(path, "/proc/ioports") == 0) return FAKE_IOPORTS;
    if (path != nullptr && strcmp(path, "/proc/misc") == 0) return FAKE_PROC_MISC;
    if (path != nullptr && strcmp(path, "/sys/devices/system/cpu/online") == 0)
        return FAKE_CPU_ONLINE;
    if (path != nullptr) {
        const char* cf = CpufreqFake(path);
        if (cf != nullptr) return cf;
    }
    // PCI Wi-Fi/iGPU: /proc/bus/pci/devices leaks Intel IDs, pci uevent
    // leaks DRIVER=iwlwifi. Per-process redirect only — NEVER global tmpfs
    // (minigbm/libdrm need real PCI sysfs for Intel GPU BO alloc).
    if (path != nullptr && strcmp(path, "/proc/bus/pci/devices") == 0) return FAKE_PCI_DEVICES;
    // Raw sysfs device tree: /sys/devices/pci0000:00/<bdf>/{vendor,device,
    // subsystem_vendor,subsystem_device,uevent,modalias} leak Intel i915 +
    // ThinkPad subsystem (probe log: 4-5x each from ACE). Same Qualcomm
    // view as the bus-tree fakes; host graphics stack is unhooked.
    if (path != nullptr && PathStartsWith(path, "/sys/devices/pci0000:00/")) {
        const char* slash = strrchr(path, '/');
        const char* node = slash != nullptr ? slash + 1 : path;
        if (strcmp(node, "vendor") == 0 || strcmp(node, "subsystem_vendor") == 0)
            return FAKE_PCI_VENDOR;
        if (strcmp(node, "device") == 0 || strcmp(node, "subsystem_device") == 0)
            return FAKE_PCI_DEVICE;
        if (strcmp(node, "uevent") == 0) return FAKE_PCI_DEV_UEVENT;
        if (strcmp(node, "modalias") == 0) return FAKE_PCI_DEV_MODALIAS;
    }
    // Adreno busy counters live here on Snapdragon; Intel has no kgsl node
    // (ENOENT = no-Adreno tell). Serve plausible busy/total counters.
    if (path != nullptr && strcmp(path, "/sys/class/kgsl/kgsl-3d0/gpubusy") == 0)
        return FAKE_KGSL_GPUBUSY;
    if (path != nullptr && PathStartsWith(path, "/sys/bus/pci/devices/") &&
        strlen(path) >= 7 && strcmp(path + strlen(path) - 7, "/uevent") == 0)
        return FAKE_PCI_UEVENT;
    return path;
}

// NOTE: orig_open/openat MUST be int-returning variadic pointers. An early
// revision declared them `bool`, which truncated every fd to 1 and killed
// every target at ART startup (fdsan double-close SIGABRT crash loop).
int my_open(const char* path, int flags, ...) {
    if (IsTranslatorPath(path) || IsRootPath(path)) {
        TraceProbeFile("deny", path);
        errno = ENOENT;
        return -1;
    }
    TraceProbeFile("open", path);
    // ACE parses /proc/self/maps text for translator .so names (dl_phdr
    // hook alone can't cover it). Serve the filtered pipe view.
    if (IsMapsPath(path)) {
        int fd = OpenFilteredMaps(path, flags);
        if (fd >= 0) return fd;
    }
    // TracerPid pipe view (fail-open: falls through to real on -1).
    if (IsStatusPath(path)) {
        int fd = OpenFilteredStatus(path, flags);
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
    if (IsTranslatorPath(path) || IsRootPath(path)) {
        TraceProbeFile("deny", path);
        errno = ENOENT;
        return -1;
    }
    if (dirfd == AT_FDCWD) TraceProbeFile("openat", path);
    if (dirfd == AT_FDCWD && IsMountsPath(path)) {
        flags &= ~(O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND);
        return orig_openat(dirfd, FAKE_MOUNTS, flags | O_RDONLY, (mode_t)0);
    }
    if (dirfd == AT_FDCWD && IsMapsPath(path)) {
        int fd = OpenFilteredMaps(path, flags);
        if (fd >= 0) return fd;
    }
    if (dirfd == AT_FDCWD && IsStatusPath(path)) {
        int fd = OpenFilteredStatus(path, flags);
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
    if (IsTranslatorPath(path) || IsRootPath(path)) {
        TraceProbeFile("deny", path);
        errno = ENOENT;
        return nullptr;
    }
    TraceProbeFile("fopen", path);
    // TracerPid pipe view for stdio readers of status (fscanf/fgets loops).
    if (IsStatusPath(path) && mode != nullptr && strchr(mode, 'r') != nullptr &&
        strchr(mode, 'w') == nullptr && strchr(mode, 'a') == nullptr &&
        strchr(mode, '+') == nullptr) {
        int fd = OpenFilteredStatus(path, O_RDONLY);
        if (fd >= 0) {
            FILE* f = ::fdopen(fd, mode);
            if (f != nullptr) return f;
            ::close(fd);
        }
    }
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
    if (IsTranslatorPath(path) || IsRootPath(path)) {
        TraceProbeFile("deny", path);
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
    if (IsTranslatorPath(path) || IsRootPath(path)) {
        TraceProbeFile("deny", path);
        errno = ENOENT;
        return -1;
    }
    if (orig_fstatat != nullptr)
        return orig_fstatat(dirfd, path, buf, flags);
    errno = ENOSYS;
    return -1;
}

int my_faccessat(int dirfd, const char* path, int mode, int flags) {
    if (IsTranslatorPath(path) || IsRootPath(path)) {
        TraceProbeFile("deny", path);
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
            IsHiddenModuleLine(info->dlpi_name)) {
            return 0;  // skip: hide translator/hook module from detectors
        }
        return c->cb(info, size, c->data);
    };
    if (orig_dl_iterate_phdr != nullptr)
        return orig_dl_iterate_phdr(filtered, &ctx);
    return 0;
}

// --- getprop shell bypass -------------------------------------------------
// Detectors dodge libc properties via `sh -c "getprop <key>"`, bare
// `getprop` dumps, or direct exec of the getprop binary (all fork+read
// stdout, bypassing __system_property_get). Serve the same deny-aware
// spoofed view the libc hooks serve:
//
//   single key: denied/missing -> `echo` (empty); spoofed -> `echo 'value'`;
//               unknown keys pass through to the real getprop (fail-open).
//   bare dump:  `printf '%s\\n' '[k]: [v]' ...` built from g_props with
//               denied keys skipped and spoofed values substituted.
//
// Only exact single-command forms are rewritten; compound shell
// (`;`, `|`, `&`, ... past the key) passes through untouched so app
// spawn / installer scripts never break.
//
// NOTE on execl/execv: Bionic routes every exec-family call through
// execve, so this hook plus popen covers execl/execv/execlp/execvp with
// no extra hooks.

const char* ExecBasename(const char* path) {
    if (path == nullptr) return "";
    const char* base = strrchr(path, '/');
    return base != nullptr ? base + 1 : path;
}

const char* SkipSpaces(const char* p) {
    while (p != nullptr && *p == ' ') p++;
    return p;
}

bool IsShellRemainderSafe(const char* p) {
    // Only trailing whitespace allowed after the key token.
    while (p != nullptr && *p != '\0') {
        if (*p != ' ' && *p != '\t' && *p != '\n' && *p != '\r') return false;
        p++;
    }
    return true;
}

// Parse `getprop [key]` out of an `sh -c` command. Returns 1 + key for a
// single-key query, 0 for a bare dump, -1 for anything compound/foreign.
int ParseGetpropShCmd(const char* cmd, char* keyOut, size_t keyCap) {
    if (cmd == nullptr || keyOut == nullptr || keyCap == 0) return -1;
    const char* p = SkipSpaces(cmd);
    static const char kGetprop[] = "getprop";
    if (strncmp(p, kGetprop, sizeof(kGetprop) - 1) != 0) return -1;
    p += sizeof(kGetprop) - 1;
    if (*p != '\0' && *p != ' ' && *p != '\t') return -1;
    p = SkipSpaces(p);
    if (*p == '\0') return 0;  // bare dump
    size_t i = 0;
    while (*p != '\0' && *p != ' ' && *p != '\t' && *p != '\n' &&
           *p != '\r' && *p != ';' && *p != '|' && *p != '&' &&
           *p != '`' && *p != '$' && *p != '(' && *p != ')' &&
           *p != '<' && *p != '>') {
        if (i + 1 < keyCap) keyOut[i++] = *p;
        p++;
    }
    keyOut[i] = '\0';
    if (i == 0) return -1;
    if (!IsShellRemainderSafe(p)) return -1;
    return 1;
}

// Append a single-quoted shell word (keys/values are [a-z0-9._-] plus
// free-form values; `'` becomes `'\''`).
void AppendShellQuoted(std::string& out, const char* v) {
    out += '\'';
    for (const char* q = v; q != nullptr && *q != '\0'; q++) {
        if (*q == '\'') {
            out += "'\\''";
        } else {
            out += *q;
        }
    }
    out += '\'';
}

// Build `echo 'value'` (or bare `echo` for denied/missing) for one key.
// Returns false to fail open to the real getprop (unknown key).
bool BuildGetpropSingle(const char* key, char* out, size_t cap) {
    if (key == nullptr || out == nullptr || cap == 0) return false;
    if (IsDeniedProperty(key)) {
        snprintf(out, cap, "echo");
        return true;
    }
    std::string spoofed;
    if (!LookupProperty(key, spoofed)) return false;
    std::string cmd = "echo ";
    AppendShellQuoted(cmd, spoofed.c_str());
    snprintf(out, cap, "%s", cmd.c_str());
    return true;
}

// Build the bare-dump replacement: `printf '%s\n' '[k]: [v]' ...` over
// g_props minus denied keys. Returns false on overflow (fail-open).
bool BuildGetpropDump(char* out, size_t cap) {
    if (out == nullptr || cap == 0) return false;
    std::string cmd = "printf '%s\n'";
    for (const auto& kv : g_props) {
        if (IsDeniedProperty(kv.first.c_str())) continue;
        if (kv.first == "debug.verbose" ||
            kv.first == "debug.trace_probes")
            continue;
        std::string entry = "[" + kv.first + "]: [" + kv.second + "]";
        std::string word;
        AppendShellQuoted(word, entry.c_str());
        if (cmd.size() + 1 + word.size() + 1 > cap) return false;
        cmd += ' ';
        cmd += word;
    }
    snprintf(out, cap, "%s", cmd.c_str());
    return true;
}

// Popen content builders over the same table (no fork): single key gives
// `value\n` ("" when denied/missing); bare gives the filtered dump.
bool BuildPopenGetprop(const char* cmd, std::string& out) {
    char key[PROP_NAME_MAX] = {0};
    int kind = ParseGetpropShCmd(SkipSpaces(cmd), key, sizeof(key));
    if (kind < 0) return false;
    if (kind == 0) {
        out.clear();
        for (const auto& kv : g_props) {
            if (IsDeniedProperty(kv.first.c_str())) continue;
            if (kv.first == "debug.verbose" ||
                kv.first == "debug.trace_probes")
                continue;
            out += "[" + kv.first + "]: [" + kv.second + "]\n";
        }
        return true;
    }
    if (IsDeniedProperty(key)) {
        out.clear();
        return true;
    }
    std::string spoofed;
    if (!LookupProperty(key, spoofed)) return false;  // fail-open: real popen
    out = spoofed + "\n";
    return true;
}

// Pipe-backed FILE* serving `content` (popen/getprop path). Null on error
// so the caller can fail open to the real popen.
FILE* PipeFileFromString(const std::string& content, const char* mode) {
    int fds[2];
    if (::pipe(fds) != 0) return nullptr;
    size_t written = 0;
    while (written < content.size()) {
        ssize_t n = ::write(fds[1], content.data() + written,
                            content.size() - written);
        if (n <= 0) {
            ::close(fds[0]);
            ::close(fds[1]);
            return nullptr;
        }
        written += (size_t)n;
    }
    ::close(fds[1]);
    FILE* f = ::fdopen(fds[0], mode);
    if (f == nullptr) ::close(fds[0]);
    return f;
}

// --- execve redirect ------------------------------------------------------
// `sh -c "ls <masked>"` / `"cat <masked node>"` fallbacks (t1/a, P0/a).
// Rewrite argv to `cat <fake list>` so shell probes see the synthetic view.
// Only exact masked roots + bare `ls <root>` / `cat <root>` forms; every
// other exec passes through untouched (app spawn, dex2oat, etc. safe).
int my_execve(const char* path, char* const argv[], char* const envp[]) {
    // Trace shell probes (`sh -c "getprop ..."`, `sh -c "ls ..."`) verbatim.
    if (path != nullptr && argv != nullptr && argv[0] != nullptr &&
        argv[1] != nullptr && argv[2] != nullptr) {
        const char* base = ExecBasename(path);
        if (strcmp(base, "sh") == 0 && strcmp(argv[1], "-c") == 0) {
            TraceProbeFile("exec-sh", argv[2]);
        }
    }
    if (path != nullptr && argv != nullptr && argv[0] != nullptr &&
        orig_execve != nullptr) {
        const char* base = ExecBasename(path);
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
            // getprop bypass: serve the deny-aware spoofed view.
            {
                char key[PROP_NAME_MAX] = {0};
                int kind = ParseGetpropShCmd(cmd, key, sizeof(key));
                if (kind == 1) {
                    static thread_local char ncmd[1024];
                    if (BuildGetpropSingle(key, ncmd, sizeof(ncmd))) {
                        TraceProbeFile("exec-getprop", key);
                        char* nargv[4];
                        nargv[0] = argv[0];
                        nargv[1] = argv[1];
                        nargv[2] = ncmd;
                        nargv[3] = nullptr;
                        return orig_execve(path, nargv, envp);
                    }
                } else if (kind == 0) {
                    static thread_local char ncmd[32768];
                    if (BuildGetpropDump(ncmd, sizeof(ncmd))) {
                        TraceProbeFile("exec-getprop", "(dump)");
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
        // Direct exec of the getprop binary (no sh wrapper).
        if (strcmp(base, "getprop") == 0) {
            const char* key = (argv[1] != nullptr) ? argv[1] : nullptr;
            if (key == nullptr || *key == '\0') {
                static thread_local char ncmd[32768];
                if (BuildGetpropDump(ncmd, sizeof(ncmd))) {
                    TraceProbeFile("exec-getprop", "(dump)");
                    static thread_local char sh[] = "/system/bin/sh";
                    static thread_local char dash_c[] = "-c";
                    char* nargv[4];
                    nargv[0] = sh;
                    nargv[1] = dash_c;
                    nargv[2] = ncmd;
                    nargv[3] = nullptr;
                    return orig_execve(sh, nargv, envp);
                }
            } else if (argv[2] == nullptr) {
                // Denied/missing -> bare `echo` (empty); spoofed -> echo
                // with the raw value (no shell quoting needed: direct
                // argv, no sh involved). Unknown keys fall through real.
                if (IsDeniedProperty(key)) {
                    TraceProbeFile("exec-getprop", key);
                    static thread_local char epath[] = "/system/bin/echo";
                    char* nargv[2];
                    nargv[0] = epath;
                    nargv[1] = nullptr;
                    return orig_execve(epath, nargv, envp);
                }
                std::string spoofed;
                if (LookupProperty(key, spoofed)) {
                    TraceProbeFile("exec-getprop", key);
                    static thread_local char epath[] = "/system/bin/echo";
                    static thread_local char val[PROP_VALUE_MAX];
                    snprintf(val, sizeof(val), "%s", spoofed.c_str());
                    char* nargv[3];
                    nargv[0] = epath;
                    nargv[1] = val;
                    nargv[2] = nullptr;
                    return orig_execve(epath, nargv, envp);
                }
            }
        }
    }
    if (orig_execve != nullptr) return orig_execve(path, argv, envp);
    return -1;
}

// --- popen bypass ---------------------------------------------------------
// Same getprop table without the fork: pipe-backed FILE* for getprop
// commands, real popen otherwise (fail-open). Synthetic handles are
// tracked so my_pclose can fclose them instead of waitpid-ing a child
// that never existed.

std::mutex g_popen_mutex;
std::unordered_set<FILE*> g_popen_set;
FILE* (*orig_popen)(const char*, const char*) = nullptr;
int (*orig_pclose)(FILE*) = nullptr;

bool IsSyntheticPopen(FILE* f) {
    std::lock_guard<std::mutex> lk(g_popen_mutex);
    return g_popen_set.find(f) != g_popen_set.end();
}

void RegisterSyntheticPopen(FILE* f) {
    std::lock_guard<std::mutex> lk(g_popen_mutex);
    g_popen_set.insert(f);
}

void UnregisterSyntheticPopen(FILE* f) {
    std::lock_guard<std::mutex> lk(g_popen_mutex);
    g_popen_set.erase(f);
}

FILE* my_popen(const char* cmd, const char* mode) {
    if (cmd != nullptr && mode != nullptr && strchr(mode, 'r') != nullptr &&
        strchr(mode, 'w') == nullptr && strchr(mode, '+') == nullptr) {
        std::string content;
        if (BuildPopenGetprop(cmd, content)) {
            TraceProbeFile("popen-getprop", cmd);
            FILE* f = PipeFileFromString(content, mode);
            if (f != nullptr) {
                RegisterSyntheticPopen(f);
                return f;
            }
            // Pipe failure: fail open to the real popen below.
        }
    }
    if (orig_popen != nullptr) return orig_popen(cmd, mode);
    errno = ENOSYS;
    return nullptr;
}

int my_pclose(FILE* f) {
    if (f != nullptr && IsSyntheticPopen(f)) {
        UnregisterSyntheticPopen(f);
        return ::fclose(f) == 0 ? 0 : -1;
    }
    if (orig_pclose != nullptr) return orig_pclose(f);
    errno = ENOSYS;
    return -1;
}

// --- ptrace guard -----------------------------------------------------------
// Anti-debug: detectors call ptrace(TRACEME) expecting failure under a
// tracer, or ATTACH self to detect refusal. Fake TRACEME success (no real
// call) and refuse ATTACH on our own pid; everything else passes through.
long (*orig_ptrace)(int, pid_t, void*, void*) = nullptr;

long my_ptrace(int request, pid_t pid, void* addr, void* data) {
    if (request == PTRACE_TRACEME) {
        TraceProbeFile("ptrace", "traceme-faked");
        return 0;
    }
    if (request == PTRACE_ATTACH && pid == ::getpid()) {
        TraceProbeFile("ptrace", "attach-self-denied");
        errno = EPERM;
        return -1;
    }
    if (orig_ptrace != nullptr) return orig_ptrace(request, pid, addr, data);
    errno = ENOSYS;
    return -1;
}

// --- inotify watch: log-only --------------------------------------------------
// Detectors watch su/magisk paths via inotify; log the watched path for
// the probe trace. Do NOT deny yet (fail-open by design).
int (*orig_inotify_add_watch)(int, const char*, uint32_t) = nullptr;

int my_inotify_add_watch(int fd, const char* path, uint32_t mask) {
    TraceProbeFile("inotify", path);
    (void)mask;
    if (orig_inotify_add_watch != nullptr)
        return orig_inotify_add_watch(fd, path, mask);
    errno = ENOSYS;
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
    // filter translator + hook-framework entries so checkArmTranslation
    // sees a clean list. Own libs (gs_native/dobby) are guarded inside
    // the predicate and stay visible.
    bool ok_phdr = HookExport("dl_iterate_phdr",
                              reinterpret_cast<void*>(&my_dl_iterate_phdr),
                              reinterpret_cast<void**>(&orig_dl_iterate_phdr));
    // getprop shell bypass (Runtime.exec / popen over the same prop table).
    // Bionic routes execl/execv/execlp/execvp through execve, so the execve
    // hook covers them with no extra hooks.
    bool ok_popen = HookExport("popen", reinterpret_cast<void*>(&my_popen),
                               reinterpret_cast<void**>(&orig_popen));
    bool ok_pclose = HookExport("pclose",
                                reinterpret_cast<void*>(&my_pclose),
                                reinterpret_cast<void**>(&orig_pclose));
    // Anti-debug ptrace guard + log-only inotify watch (never denies).
    bool ok_ptrace = HookExport("ptrace",
                                reinterpret_cast<void*>(&my_ptrace),
                                reinterpret_cast<void**>(&orig_ptrace));
    bool ok_inotify = HookExport("inotify_add_watch",
                                 reinterpret_cast<void*>(&my_inotify_add_watch),
                                 reinterpret_cast<void**>(&orig_inotify_add_watch));
    DS_LOGI("file hooks: open=%d openat=%d fopen=%d opendir=%d readdir=%d closedir=%d rewinddir=%d telldir=%d seekdir=%d dirfd=%d execve=%d fstatat=%d faccessat=%d phdr=%d popen=%d pclose=%d ptrace=%d inotify=%d",
            ok_open, ok_openat, ok_fopen,
            ok_opendir, ok_readdir, ok_closedir,
            ok_rewinddir, ok_telldir, ok_seekdir, ok_dirfd, ok_execve,
            ok_fstatat, ok_faccessat, ok_phdr, ok_popen, ok_pclose,
            ok_ptrace, ok_inotify);
}

}  // namespace gs
