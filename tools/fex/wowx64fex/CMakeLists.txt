# libwowx64fex.dll — the BTCpu64 CPU layer (x86-64 guests on arm64), built OUT
# OF TREE against the prebuilt vendored FEX (tools/fex/build-fex.sh), the same
# arrangement as tools/fex/hello-compute. vendor/fex is never modified and never
# add_subdirectory'd: 25 ${CMAKE_SOURCE_DIR} references inside it would rebind
# to this project (docs/engineering/wow64-x64-guest-design.md, "four practical
# constraints"). Configure with:
#   FEX_SRC    = <repo>/vendor/fex
#   FEX_BUILD  = <the FEX build dir produced by build-fex.sh>
#   VIBEOS_SRC = <repo>   (for sdk/include/reactos/wowx64/btcpu64.h)
# See tools/fex/build-hello-compute.sh for the exact invocation.
cmake_minimum_required(VERSION 3.14)
project(wowx64fex CXX)

if(NOT FEX_SRC OR NOT FEX_BUILD OR NOT VIBEOS_SRC)
  message(FATAL_ERROR "Set -DFEX_SRC=<vendor/fex> -DFEX_BUILD=<fex build dir> -DVIBEOS_SRC=<repo>")
endif()

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(wowx64fex SHARED Module.cpp libwowx64fex.def)
set_target_properties(wowx64fex PROPERTIES PREFIX "lib" OUTPUT_NAME "wowx64fex")

# Same defines + flags FEX compiles its own WOW64 plugin module with (read off
# build-fex-arm64/build.ninja): the FEXCore headers are not neutral to them.
target_compile_definitions(wowx64fex PRIVATE
  ARCHITECTURE_arm64=1
  FEXCORE_HAS_PRESERVE_ALL_ATTR=0 FEXCORE_PRESERVE_ALL_ATTR=
  FEX_HAS_PRESERVE_ALL_ATTR=0 FEX_PRESERVE_ALL_ATTR=)
# armv8-a floor: the A72 (QEMU and the real Pi 4) has no FEAT_LRCPC, and clang
# otherwise emits LDAPR for C++ static-init guards. Matches build-fex.sh.
target_compile_options(wowx64fex PRIVATE -march=armv8-a -fno-strict-aliasing -Wall
  -Wno-deprecated-enum-enum-conversion)

target_include_directories(wowx64fex PRIVATE
  ${VIBEOS_SRC}/sdk/include/reactos/wowx64
  ${FEX_SRC}/Source
  ${FEX_BUILD}/Source
  ${FEX_SRC}/Source/Windows/include
  ${FEX_SRC}/Source/Windows
  ${FEX_BUILD}/FEXCore/Source
  ${FEX_SRC}/FEXCore/include
  ${FEX_BUILD}/include
  ${FEX_SRC}/External/fmt/include
  ${FEX_SRC}/External/xxhash/cmake_unofficial/..
  ${FEX_SRC}/FEXHeaderUtils
  ${FEX_SRC}/CodeEmitter
  ${FEX_SRC}/External/cephes/include
  ${FEX_SRC}/External/SoftFloat-3e/include
  ${FEX_SRC}/External/SoftFloat-3e/include/SoftFloat-3e
  ${FEX_SRC}/External/unordered_dense/include
  ${FEX_SRC}/Source/Common/cpp-optparse
  ${FEX_SRC}/External/tiny-json
  ${FEX_SRC}/External/range-v3/include
  ${FEX_SRC}/Source/Tools/CommonTools
  ${FEX_SRC}/External/rpmalloc)

execute_process(
  COMMAND ${CMAKE_CXX_COMPILER} -print-libgcc-file-name -rtlib=compiler-rt
  OUTPUT_VARIABLE WOWX64_BUILTINS OUTPUT_STRIP_TRAILING_WHITESPACE)

# FEX's own libwow64fex.dll link, archive for archive, with two changes:
# libFEXCore.a stands in for $<TARGET_OBJECTS:FEXCore_object> (an object
# library cannot cross projects), and libwow64_ex.a (the wow64.dll import lib)
# is gone. The group is not optional: without it static-archive ordering
# leaves strlen/memcpy/memmove undefined although FEX's CRT provides them.
target_link_libraries(wowx64fex PRIVATE
  -Wl,--start-group
  ${FEX_BUILD}/FEXCore/Source/libFEXCore.a
  ${FEX_BUILD}/FEXCore/Source/libFEXCore_Base.a
  ${FEX_BUILD}/Source/Common/libCommon.a
  ${FEX_BUILD}/Source/Tools/CommonTools/libCommonTools.a
  ${FEX_BUILD}/Source/Windows/Common/libCommonWindows.a
  ${FEX_BUILD}/Source/Windows/Common/libCommonWindowsRuntime.a
  ${FEX_BUILD}/Source/Windows/libntdll_ex.a
  ${FEX_BUILD}/Source/Common/cpp-optparse/libcpp-optparse.a
  ${FEX_BUILD}/External/tiny-json/libtiny-json.a
  ${FEX_BUILD}/External/fmt/libfmt.a
  ${FEX_BUILD}/External/xxhash/cmake_unofficial/libxxhash.a
  ${FEX_BUILD}/External/cephes/libcephes_128bit.a
  ${FEX_BUILD}/External/SoftFloat-3e/libsoftfloat_3e.a
  ${FEX_BUILD}/FEXCore/Source/libJemallocLibs.a
  ${FEX_BUILD}/External/rpmalloc/librpmalloc.a
  ${WOWX64_BUILTINS}
  -lsynchronization
  -Wl,--end-group)

target_link_options(wowx64fex PRIVATE
  -static -static-libgcc -static-libstdc++
  -Wl,--file-alignment=4096,/mllvm:-align-loops=1
  -nostdlib -nostartfiles -nodefaultlibs -lc++ -lc++abi -lunwind)
