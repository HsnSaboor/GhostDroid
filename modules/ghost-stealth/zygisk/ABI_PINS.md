// Zygisk ABI/API pins (2026-09, latest verified):
// - Zygisk API v2 (topjohnwu header, unchanged since v2).
// - ReZygisk 1.0.0 (active), Vector 2.2 (API 102).
// - NDK r26+ (DSL pins 26.3.11579264), LSPlt 2.1 (latest).
// - APP_ABI: x86_64 (Waydroid zygote) + x86 + arm64-v8a + armeabi-v7a.
// Ref: /tmp/tfsrc/app/build.gradle.kts (ndkVersion 29.x, c++23, Dobby).
#pragma once
