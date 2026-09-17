# gs_native: LSPlt PLT hooks (get/find/read/read_callback, uname,
# gethostname, getifaddrs). Copied MIT from DeviceSpoofLab-Hooks cpp/.
# Pins (2026-09): NDK r26+ (DSL uses 26.3.11579264), cmake 3.22.1,
# cxx17, LSPlt 2.1 (latest upstream, LSPosed/LSPlt).
# LSPlt source layout (verified .devdocs/LSPlt clone):
#   lsplt/src/main/jni/{lsplt.cc,elf_util.cc,include/lsplt.hpp}.
# Standalone NDK: add_subdirectory LSPlt source, link lsplt::lsplt.
cmake_minimum_required(VERSION 3.22.1)
project("gs_native")

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# LSPLT_SOURCE_DIR: path to LSPosed/LSPlt checkout (CI clones tag 2.1).
# LSPlt upstream target is plain `lsplt` (+ `lsplt_static`), no namespaced
# alias in standalone source builds (verified .devdocs/LSPlt clone).
if(DEFINED LSPLT_SOURCE_DIR)
  # LSPlt standalone writes debug symbols under DEBUG_SYMBOLS_PATH
  # (defaults to <bindir>/symbols). Pin it explicitly: with an empty
  # value it expands to "/<ABI>" and the link fails (CI 35198938544).
  set(DEBUG_SYMBOLS_PATH ${CMAKE_BINARY_DIR}/symbols CACHE PATH "")
  add_subdirectory(${LSPLT_SOURCE_DIR}/lsplt/src/main/jni lsplt)
  # Static link: liblsplt.so is NOT shipped in the module zip, so a shared
  # NEEDED entry fails dlopen inside the target process (silent no-op).
  add_library(lsplt::lsplt ALIAS lsplt_static)
else()
  find_package(lsplt REQUIRED CONFIG)
endif()

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
