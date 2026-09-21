# gs_native: Dobby export-side inline hooks (get/find/read/read_callback,
# uname, gethostname, getifaddrs, open-family + maps-content filter).
# Export-side patching covers libraries
# LSPlt PLT-patching only covered already-loaded callers, so reveny still
# saw real abilist/native.bridge values.
# Pins (2026-09): NDK r26+ (DSL uses 26.3.11579264), cmake 3.22.1,
# cxx17, Dobby (JingMatrix fork, same as TargetedFix).
# Dobby builds libdobby.so shared (upstream has no namespaced static alias
# in standalone source builds); the module zip must ship libdobby.so next
# to libgs_native.so, or dlopen fails inside the target process.
cmake_minimum_required(VERSION 3.22.1)
project("gs_native")

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# DOBBY_SOURCE_DIR: path to Dobby checkout (CI clones JingMatrix fork).
# SymbolResolver plugin (default ON) provides DobbySymbolResolver used for
# export-side lookup; BionicLinkerUtil enables linker-soinfo resolution.
if(NOT DEFINED DOBBY_SOURCE_DIR)
  message(FATAL_ERROR "DOBBY_SOURCE_DIR must point at a Dobby checkout")
endif()
add_subdirectory(${DOBBY_SOURCE_DIR} dobby)

add_library(gs_native SHARED
    native_hooks.cpp
    property_hooks.cpp
    system_hooks.cpp
    file_hooks.cpp
    graphics_hooks.cpp
)

target_include_directories(gs_native PRIVATE ${DOBBY_SOURCE_DIR}/include)

target_link_libraries(gs_native
    PRIVATE
    dobby
    log
    dl
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
