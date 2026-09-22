#include "gs_state.h"

#include <cstring>
#include <ifaddrs.h>
#include <linux/if_packet.h>
#include <net/if.h>
#include <netinet/in.h>
#include <string>
#include <sys/socket.h>
#include <sys/utsname.h>
#include <unistd.h>

#include <dobby.h>

namespace gs {

namespace {

int (*orig_uname)(struct utsname*) = nullptr;
int (*orig_gethostname)(char*, size_t) = nullptr;
int (*orig_getifaddrs)(struct ifaddrs**) = nullptr;

std::string LookupOr(const char* key, const char* fallback) {
    std::string out;
    if (LookupProperty(key, out) && !out.empty()) return out;
    return fallback ? fallback : "";
}

int my_uname(struct utsname* u) {
    int rc = orig_uname ? orig_uname(u) : -1;
    if (u != nullptr) {
        std::string release = LookupOr("kernel.osrelease", "5.15.148-android13-4-00003-gabcdef123456-ab11223344");
        std::string version = LookupOr("kernel.version",
                                       "#1 SMP PREEMPT Wed May 22 18:00:00 UTC 2024");
        std::string nodename = LookupOr("kernel.hostname", "localhost");
        std::string machine = LookupOr("kernel.machine", "aarch64");

        // sysname left as Linux; machine forced aarch64 (DeviceInfoHW SoC
        // tab leaked x86_64 because u->machine was untouched).
        snprintf(u->release, sizeof(u->release), "%s", release.c_str());
        snprintf(u->version, sizeof(u->version), "%s", version.c_str());
        snprintf(u->nodename, sizeof(u->nodename), "%s", nodename.c_str());
        snprintf(u->machine, sizeof(u->machine), "%s", machine.c_str());
    }
    return rc;
}

int my_gethostname(char* name, size_t len) {
    if (name == nullptr || len == 0) {
        return orig_gethostname ? orig_gethostname(name, len) : -1;
    }
    std::string h = LookupOr("kernel.hostname", "localhost");
    size_t n = h.size();
    if (n + 1 > len) n = len - 1;
    memcpy(name, h.data(), n);
    name[n] = '\0';
    return 0;
}

bool ParseMac(const std::string& s, uint8_t out[6]) {
    if (s.size() < 17) return false;
    auto hex = [](char c, int& v) {
        if (c >= '0' && c <= '9') { v = c - '0'; return true; }
        if (c >= 'a' && c <= 'f') { v = c - 'a' + 10; return true; }
        if (c >= 'A' && c <= 'F') { v = c - 'A' + 10; return true; }
        return false;
    };
    for (int i = 0; i < 6; i++) {
        int hi = 0, lo = 0;
        if (!hex(s[i*3 + 0], hi)) return false;
        if (!hex(s[i*3 + 1], lo)) return false;
        out[i] = (uint8_t)((hi << 4) | lo);
    }
    return true;
}

int my_getifaddrs(struct ifaddrs** ifap) {
    int rc = orig_getifaddrs ? orig_getifaddrs(ifap) : -1;
    if (rc != 0 || ifap == nullptr || *ifap == nullptr) return rc;

    std::string wifiMac = LookupOr("wifi.mac", "");
    std::string btMac   = LookupOr("bluetooth.mac", "");
    uint8_t wifiBytes[6] = {0}, btBytes[6] = {0};
    bool haveWifi = !wifiMac.empty() && ParseMac(wifiMac, wifiBytes);
    bool haveBt   = !btMac.empty()   && ParseMac(btMac,   btBytes);

    // Same-or-shorter in-place rename (2026-09-22): libc's ifa_name buffer
    // is exactly strlen(orig)+1, so only strcpy an alias with
    // strlen(alias) <= strlen(orig) — never past the original NUL. Longer
    // phone names ("wlan0" 5B over "eth0" 5B incl. NUL) SEGV'd gralloc/EGL
    // (2026-09-21 tombstone in gbm_mesa_bo_import). Mapping keeps the
    // lo/wlan*/rmnet_data* family used by the /proc/net/dev fake:
    //   eth0 (4)      -> "wlan"  (4, exact fit)
    //   ethN len>=5   -> "wlan0" (5, fits)
    //   veth*         -> "wlan0" (veth names are >=5, fits; guarded anyway)
    //   waydroid* (9) -> "wlan0" (5 fits; "rmnet_data0" 11 would NOT fit)
    //   docker*       -> "wlan0" (fits; guarded anyway)
    // Rename runs before MAC handling, so a renamed entry matches the
    // "wlan" prefix below and receives the spoofed wifi MAC (looks like a
    // real phone wlan); anything else non-lo gets zeroed as before.
    // Duplicate names across entries are legal (multi-addr interfaces).
    for (struct ifaddrs* cur = *ifap; cur != nullptr; cur = cur->ifa_next) {
        if (cur->ifa_name == nullptr) continue;
        const char* n = cur->ifa_name;
        const char* alias = nullptr;
        if (strncmp(n, "waydroid", 8) == 0) {
            alias = "wlan0";
        } else if (strncmp(n, "veth", 4) == 0) {
            alias = "wlan0";
        } else if (strncmp(n, "docker", 6) == 0) {
            alias = "wlan0";
        } else if (strncmp(n, "eth", 3) == 0) {
            alias = (strlen(n) >= 5) ? "wlan0" : "wlan";
        }
        if (alias != nullptr && strlen(alias) <= strlen(n)) {
            strcpy(cur->ifa_name, alias);
            n = cur->ifa_name;
        }
        // (fall through to MAC handling.)
        if (cur->ifa_addr == nullptr) continue;
        if (cur->ifa_addr->sa_family != AF_PACKET) continue;
        auto* sll = reinterpret_cast<struct sockaddr_ll*>(cur->ifa_addr);
        if (sll->sll_halen != 6) continue;

        if (haveWifi && (strncmp(n, "wlan", 4) == 0)) {
            memcpy(sll->sll_addr, wifiBytes, 6);
        } else if (haveBt && (strncmp(n, "bt", 2) == 0 || strncmp(n, "bnep", 4) == 0)) {
            memcpy(sll->sll_addr, btBytes, 6);
        } else if (strncmp(n, "lo", 2) == 0) {
            // loopback
        } else {
            memset(sll->sll_addr, 0, 6);
        }
    }
    return rc;
}

}  // namespace

void InstallSystemHooks() {
    // SafeResolve (plain dlsym): install-time resolution never touches the
    // Dobby maps parser, so preAppSpecialize/zygote early init stays clean.
    void *u = SafeResolve("uname");
    void *gh = SafeResolve("gethostname");
    void *gi = SafeResolve("getifaddrs");
    bool ok_uname = false, ok_gh = false, ok_gi = false;
    if (u != nullptr)
        ok_uname = DobbyHook(u, (dobby_dummy_func_t)&my_uname,
                             (dobby_dummy_func_t *)&orig_uname) == 0;
    if (gh != nullptr)
        ok_gh = DobbyHook(gh, (dobby_dummy_func_t)&my_gethostname,
                          (dobby_dummy_func_t *)&orig_gethostname) == 0;
    if (gi != nullptr)
        ok_gi = DobbyHook(gi, (dobby_dummy_func_t)&my_getifaddrs,
                          (dobby_dummy_func_t *)&orig_getifaddrs) == 0;
    DS_LOGI("system hooks: uname=%d gethostname=%d getifaddrs=%d",
            ok_uname, ok_gh, ok_gi);
}

}  // namespace gs
