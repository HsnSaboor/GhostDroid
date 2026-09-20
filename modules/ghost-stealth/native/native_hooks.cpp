#include "gs_state.h"

#include <atomic>
#include <mutex>

namespace gs {
std::unordered_map<std::string, std::string> g_props;
}  // namespace gs

namespace {

std::atomic<bool> g_installed{false};
std::mutex g_install_mutex;

}  // namespace

namespace gs {

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

std::atomic<int> g_probe_lines{0};
// 4000 lines max per process: full ACE scan fits, logcat survives.
constexpr int kProbeCap = 4000;

bool ProbeSlot() {
    int n = g_probe_lines.fetch_add(1, std::memory_order_relaxed);
    return n < kProbeCap;
}

}  // namespace

void TraceProbeProp(const char* name) {
    if (name == nullptr || !TraceProbes() || !ProbeSlot()) return;
    DS_LOGW("probe prop: %s", name);
}

void TraceProbeFile(const char* op, const char* path) {
    if (op == nullptr || path == nullptr || !TraceProbes() || !ProbeSlot()) return;
    DS_LOGW("probe %s: %s", op, path);
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
