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

void InstallPropertyHooks();

void InstallSystemHooks();

}  // namespace gs
