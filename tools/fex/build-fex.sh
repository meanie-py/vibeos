#!/usr/bin/env bash
# Build the vendored FEX-Emu (vendor/fex) for arm64 Windows with the repo's
# llvm-mingw — OUT-OF-BAND from the ReactOS cmake graph (fex-bringup-plan.md
# rung 1). Produces libwow64fex.dll (x86-guest WoW64 BTCpu plugin, imports
# ntdll+wow64 only) and FEXOfflineCompiler.exe in $BUILD_DIR/Bin/.
#
# Usage: tools/fex/build-fex.sh [builddir]   (default: build-fex-arm64)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FEX_SRC="$REPO_ROOT/vendor/fex"
BUILD_DIR="${1:-$REPO_ROOT/build-fex-arm64}"
LLVM_MINGW="$REPO_ROOT/.build-tools/llvm-mingw/bin"

[ -x "$LLVM_MINGW/aarch64-w64-mingw32-clang++" ] || {
    echo "FATAL: llvm-mingw not found at $LLVM_MINGW" >&2; exit 1; }
[ -f "$FEX_SRC/CMakeLists.txt" ] || {
    echo "FATAL: vendored FEX not found at $FEX_SRC" >&2; exit 1; }

export PATH="$LLVM_MINGW:$PATH"

# A CMake cache pins its source dir. FEX moved from external/fex to vendor/fex
# (2026-09-26), and a build dir configured before that refuses `-S` with "does not
# match the source used to generate cache". Reset only the cache, loudly; the
# objects are rebuilt from the new source either way.
CACHE="$BUILD_DIR/CMakeCache.txt"
if [ -f "$CACHE" ]; then
    CACHED_SRC="$(sed -n 's/^CMAKE_HOME_DIRECTORY:INTERNAL=//p' "$CACHE")"
    if [ -n "$CACHED_SRC" ] && [ "$CACHED_SRC" != "$FEX_SRC" ]; then
        echo "[build-fex] $BUILD_DIR was configured from $CACHED_SRC;" \
             "resetting its CMake cache to configure from $FEX_SRC" >&2
        rm -rf "$CACHE" "$BUILD_DIR/CMakeFiles"
    fi
fi

mkdir -p "$BUILD_DIR"
cmake -S "$FEX_SRC" -B "$BUILD_DIR" -G Ninja \
    -DCMAKE_TOOLCHAIN_FILE=Data/CMake/toolchain_mingw.cmake \
    -DMINGW_TRIPLE=aarch64-w64-mingw32 \
    -DCMAKE_BUILD_TYPE=Release \
    -DENABLE_LTO=False \
    -DBUILD_TESTING=False \
    -DBUILD_FEXCONFIG=False \
    -DENABLE_ASSERTIONS=False \
    -DTUNE_ARCH=armv8-a \
    -DTUNE_CPU=none
    # armv8-a floor (NOT the default TUNE_CPU=native, which on this x86->aarch64
    # cross-compile picks a high-baseline -mcpu that emits ARMv8.3 LDAPR/RCpc
    # instructions). vibeos QEMU runs -cpu cortex-a72 (ARMv8.0, no FEAT_LRCPC),
    # so RCpc ops fault as illegal-instruction — matches the kernel's
    # -march=armv8-a (sdk/cmake/config.cmake OARCH). fex-bringup-plan.md rung 2.

ninja -C "$BUILD_DIR" wow64fex FEXOfflineCompiler

echo
echo "Built:"
ls -la "$BUILD_DIR/Bin/"
