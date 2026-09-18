#include "gs_state.h"

#include <cstdarg>
#include <cstdio>
#include <cstring>
#include <dirent.h>
#include <fcntl.h>
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
    if (dirfd == AT_FDCWD && IsMountsPath(path)) {
        flags &= ~(O_WRONLY | O_RDWR | O_CREAT | O_TRUNC | O_APPEND);
        return orig_openat(dirfd, FAKE_MOUNTS, flags | O_RDONLY, (mode_t)0);
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
    DS_LOGI("file hooks: open=%d openat=%d fopen=%d opendir=%d readdir=%d closedir=%d rewinddir=%d telldir=%d seekdir=%d dirfd=%d execve=%d",
            ok_open, ok_openat, ok_fopen,
            ok_opendir, ok_readdir, ok_closedir,
            ok_rewinddir, ok_telldir, ok_seekdir, ok_dirfd, ok_execve);
}

}  // namespace gs
