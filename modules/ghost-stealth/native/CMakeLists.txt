# gs_native: LSPlt PLT hooks (get/find/read/read_callback, uname,
# gethostname, getifaddrs). Copied MIT from DeviceSpoofLab-Hooks cpp/.
# Pins (2026-09): NDK r26+ (DSL uses 26.3.11579264), cmake 3.22.1,
# cxx17, LSPlt 2.1 (latest upstream, LSPosed/LSPlt). Prefab via AGP;
# standalone NDK: consume lsplt as prefab package (see CI workflow).
cmake_minimum_required(VERSION 3.22.1)
project("gs_native")

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

find_package(lsplt REQUIRED CONFIG)

add_library(gs_native SHARED
    native_hooks.cpp
    property_hooks.cpp
    system_hooks.cpp
    dlopen_hook.cpp
)

target_link_libraries(gs_native
    PRIVATE
    lsplt::lsplt
    log
)

target_compile_options(gs_native PRIVATE
    -Wall
    -Wextra
    -Wno-unused-parameter
    -ffunction-sections
    -fdata-sections
)

target_link_options(gs_native PRIVATE
    -Wl,--gc-sections
    -Wl,--exclude-libs,ALL
)
