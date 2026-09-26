# FEX rung-2 "hello compute" harness — standalone native-arm64 PE that links the
# PREBUILT vendored FEXCore static archives (vendor/fex, built out-of-band by
# tools/fex/build-fex.sh). Kept OUT of the ReactOS cmake graph, same as the FEX
# vendor build. Configure with the same mingw toolchain + these cache vars:
#   FEX_SRC   = <repo>/vendor/fex
#   FEX_BUILD = <the FEX build dir produced by build-fex.sh>
# See tools/fex/build-hello-compute.sh for the exact invocation.
cmake_minimum_required(VERSION 3.14)
project(fex_hello_compute CXX)

if(NOT FEX_SRC OR NOT FEX_BUILD)
  message(FATAL_ERROR "Set -DFEX_SRC=<vendor/fex> and -DFEX_BUILD=<fex build dir>")
endif()

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(fexprobe Main.cpp)

# armv8-a floor — vibeos QEMU runs -cpu cortex-a72 (ARMv8.0, no FEAT_LRCPC), so
# ARMv8.3 LDAPR/RCpc ops (which clang otherwise emits for C++ static-init guards)
# fault as illegal-instruction. Matches the kernel + the FEX vendor build.
target_compile_options(fexprobe PRIVATE -march=armv8-a)

# Include roots — the source trees plus the generated headers under FEX_BUILD
# (git-version / config headers configure-generated). Mirrors the compile db of
# FEX's own FEXOfflineCompiler.
target_include_directories(fexprobe PRIVATE
  ${FEX_SRC}/Source
  ${FEX_BUILD}/Source
  ${FEX_SRC}/Source/Windows/include
  ${FEX_SRC}/Source/Windows
  ${FEX_SRC}/Source/Tools/CommonTools
  ${FEX_SRC}/Source/Common/cpp-optparse
  ${FEX_BUILD}/FEXCore/Source
  ${FEX_SRC}/FEXCore/include
  ${FEX_BUILD}/include
  ${FEX_SRC}/FEXHeaderUtils
  ${FEX_SRC}/CodeEmitter
  ${FEX_SRC}/External/fmt/include
  ${FEX_SRC}/External/xxhash/cmake_unofficial/..
  ${FEX_SRC}/External/cephes/include
  ${FEX_SRC}/External/SoftFloat-3e/include
  ${FEX_SRC}/External/SoftFloat-3e/include/SoftFloat-3e
  ${FEX_SRC}/External/unordered_dense/include
  ${FEX_SRC}/External/tiny-json
  ${FEX_SRC}/External/range-v3/include
  ${FEX_SRC}/External/rpmalloc)

# Prebuilt archive link order — lifted verbatim from FEX's FEXOfflineCompiler
# LINK_LIBRARIES (repeats are intentional: static-archive symbol resolution).
target_link_libraries(fexprobe PRIVATE
  ${FEX_BUILD}/Source/Common/libCommon.a
  ${FEX_BUILD}/Source/Tools/CommonTools/libCommonTools.a
  ${FEX_BUILD}/FEXCore/Source/libFEXCore.a
  ${FEX_BUILD}/FEXCore/Source/libJemallocLibs.a
  ${FEX_BUILD}/External/fmt/libfmt.a
  ${FEX_BUILD}/Source/Windows/Common/libCommonWindows.a
  ${FEX_BUILD}/Source/Windows/libntdll_ex.a
  ${FEX_BUILD}/External/tiny-json/libtiny-json.a
  ${FEX_BUILD}/External/rpmalloc/librpmalloc.a
  ${FEX_BUILD}/FEXCore/Source/libFEXCore_Base.a
  ${FEX_BUILD}/External/fmt/libfmt.a
  ${FEX_BUILD}/External/xxhash/cmake_unofficial/libxxhash.a
  ${FEX_BUILD}/External/cephes/libcephes_128bit.a
  ${FEX_BUILD}/External/SoftFloat-3e/libsoftfloat_3e.a
  -lsynchronization
  -lntdll)

# Match FEX's PE flags (page-aligned sections, loop-align workaround).
# GUI subsystem (windows) + explicit mainCRTStartup entry: the harness has NO
# console I/O (it logs to serial via ntdll DbgPrint), so it must not be a console
# app — a console-subsystem PE makes kernel32 DllMain run ConDllInitialize, which
# wedges attaching a console (fex-bringup-plan.md rung 2). GUI apps skip console
# allocation entirely.
target_link_options(fexprobe PRIVATE
  -static -static-libgcc -static-libstdc++
  -Wl,--file-alignment=4096,/mllvm:-align-loops=1
  -Wl,--subsystem,windows -Wl,-e,mainCRTStartup)

# wow64probe — WOW64 Phase B harness (docs/engineering/wow64-plan.md). Drives
# the PREBUILT libwow64fex.dll through its BTCpu* contract; deliberately does
# NOT link FEXCore (the dll's exports are the interface under test). Same
# GUI-subsystem/serial-only conventions as fexprobe.
add_executable(wow64probe Wow64Probe.cpp)
target_compile_options(wow64probe PRIVATE -march=armv8-a)
target_link_libraries(wow64probe PRIVATE -lntdll)
target_link_options(wow64probe PRIVATE
  -static -static-libgcc -static-libstdc++
  -Wl,--subsystem,windows -Wl,-e,mainCRTStartup)
