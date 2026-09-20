#pragma once

#include <mutex>
#include <string>
#include <unordered_map>
#include <android/log.h>

#define DS_LOG_TAG "GhostStealth-Native"
#define DS_LOGI(...) do { if (::gs::IsVerboseLoggingEnabled()) __android_log_print(ANDROID_LOG_INFO, DS_LOG_TAG, __VA_ARGS__); } while (0)
#define DS_LOGW(...) __android_log_print(ANDROID_LOG_WARN,  DS_LOG_TAG, __VA_ARGS__)
#define DS_LOGE(...) __android_log_print(ANDROID_LOG_ERROR, DS_LOG_TAG, __VA_ARGS__)

namespace gs {

// Populated once from Java; read-only after install.
extern std::unordered_map<std::string, std::string> g_props;

bool LookupProperty(const char* name, std::string& out);

bool IsVerboseLoggingEnabled();

// Probe tracing (debug.trace_probes=1 in spoof.conf): logs every property
// key and every sensitive file open from the target process so detection
// scans can be observed instead of guessed. Capped per process.
bool TraceProbes();

// Log one probe line, enforcing the per-process cap. Returns true when the
// caller should skip its own logging (cap reached).
void TraceProbeProp(const char* name);
void TraceProbeFile(const char* op, const char* path);

void InstallPropertyHooks();

void InstallSystemHooks();

void InstallFileHooks();

}  // namespace gs
