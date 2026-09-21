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

// Container-tell deny-list predicate shared by the property hooks and the
// exec/popen getprop bypass (deny-aware serving). Defined in
// property_hooks.cpp; fail-open (false) on null.
bool IsDeniedProperty(const char* name);

bool IsVerboseLoggingEnabled();

// Probe tracing (debug.trace_probes=1 in spoof.conf): appends EVERY
// property key and EVERY file open from the target process to
// /data/local/tmp/gs_probe_<pid>.log. File sink (raw syscalls, no libc):
// no logcat feedback loop, no drops, no cap. One-time install notice goes
// to logcat so runs are identifiable.
bool TraceProbes();

// Log one probe line to the per-process trace file. No-op unless tracing.
void TraceProbeProp(const char* name);
void TraceProbeFile(const char* op, const char* path);

void InstallPropertyHooks();

void InstallSystemHooks();

void InstallFileHooks();

void InstallGraphicsHooks();

bool TryHookGraphicsResolved();

}  // namespace gs
