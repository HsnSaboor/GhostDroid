#include "gs_state.h"

#include <atomic>
#include <cstdio>
#include <cstring>
#include <dlfcn.h>
#include <fcntl.h>
#include <mutex>
#include <sys/syscall.h>
#include <unistd.h>

namespace gs {
std::unordered_map<std::string, std::string> g_props;
}  // namespace gs

namespace {

std::atomic<bool> g_installed{false};
std::mutex g_install_mutex;

}  // namespace

namespace gs {

void* SafeResolve(const char* sym) {
    if (sym == nullptr || *sym == '\0') return nullptr;
    dlerror();
    void* addr = dlsym(RTLD_DEFAULT, sym);
    (void)dlerror();
    return addr;
}

bool LookupProperty(const char* name, std::string& out) {
    if (name == nullptr) return false;
    auto it = g_props.find(name);
    if (it == g_props.end()) return false;
    out = it->second;
    return true;
}

bool IsVerboseLoggingEnabled() {
    auto it = g_props.find("debug.verbose");
    if (it == g_props.end()) return false;
    const std::string& v = it->second;
    return v == "1" || v == "true" || v == "TRUE" || v == "yes"
            || v == "YES" || v == "on" || v == "ON";
}

bool TraceProbes() {
    auto it = g_props.find("debug.trace_probes");
    if (it == g_props.end()) return false;
    const std::string& v = it->second;
    return v == "1" || v == "true" || v == "TRUE" || v == "yes"
            || v == "YES" || v == "on" || v == "ON";
}

namespace {

std::mutex g_probe_mutex;
int g_probe_fd = -2;  // -2 = unopened, -1 = failed/off

// Raw-syscall file sink: bypasses libc (and our own open hook), so tracing
// can never recurse or feed back into itself.
int ProbeFd() {
    if (g_probe_fd != -2) return g_probe_fd;
    char path[64];
    snprintf(path, sizeof(path), "/data/local/tmp/gs_probe_%d.log",
             (int)::getpid());
    long fd = syscall(SYS_openat, AT_FDCWD, path,
                      O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC, 0644);
    g_probe_fd = (fd < 0) ? -1 : (int)fd;
    return g_probe_fd;
}

void ProbeWrite(const char* kind, const char* a, const char* b) {
    std::lock_guard<std::mutex> lk(g_probe_mutex);
    int fd = ProbeFd();
    if (fd < 0) return;
    char line[1056];
    int n;
    if (b != nullptr) {
        n = snprintf(line, sizeof(line), "%s %s %s\n", kind, a, b);
    } else {
        n = snprintf(line, sizeof(line), "%s %s\n", kind, a);
    }
    if (n <= 0) return;
    size_t total = (size_t)n < sizeof(line) ? (size_t)n : sizeof(line) - 1;
    const char* p = line;
    while (total > 0) {
        long w = syscall(SYS_write, fd, p, total);
        if (w <= 0) break;
        p += w;
        total -= (size_t)w;
    }
}

}  // namespace

void TraceProbeProp(const char* name) {
    if (name == nullptr || !TraceProbes()) return;
    ProbeWrite("prop", name, nullptr);
}

void TraceProbeFile(const char* op, const char* path) {
    if (op == nullptr || path == nullptr || !TraceProbes()) return;
    ProbeWrite("file", op, path);
}

}  // namespace gs

extern "C" int
gs_install_buf(const char *buf, size_t len) {
    std::lock_guard<std::mutex> lock(g_install_mutex);
    if (g_installed.exchange(true)) {
        DS_LOGI("gs_install_buf: already installed; ignoring");
        return 0;
    }
    std::unordered_map<std::string, std::string> tmp;
    // key=value lines, '#' comments, '\r' tolerated.
    std::string key, val;
    bool in_key = true, in_comment = false, have_key = false;
    auto flush = [&]() {
        if (have_key && !key.empty()) tmp.emplace(key, val);
        key.clear();
        val.clear();
        in_key = true;
        have_key = false;
    };
    for (size_t i = 0; i <= len; i++) {
        char c = (i < len) ? buf[i] : '\n';
        if (c == '\r') continue;
        if (in_comment) {
            if (c == '\n') in_comment = false;
            continue;
        }
        if (c == '#' && key.empty() && val.empty() && in_key) {
            in_comment = true;
            continue;
        }
        if (c == '\n') {
            flush();
            continue;
        }
        if (in_key && c == '=') {
            in_key = false;
            have_key = true;
            continue;
        }
        if (in_key) key.push_back(c);
        else val.push_back(c);
    }
    gs::g_props = std::move(tmp);
    DS_LOGI("gs_install_buf: parsed %zu entries",
            (size_t)gs::g_props.size());

    gs::InstallPropertyHooks();
    return (int)gs::g_props.size();
}
