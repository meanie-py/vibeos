#!/usr/bin/env bash
# Build the vendored FEX-Emu (external/fex) for arm64 Windows with the repo's
# llvm-mingw — OUT-OF-BAND from the ReactOS cmake graph (fex-bringup-plan.md
# rung 1). Produces libwow64fex.dll (x86-guest WoW64 BTCpu plugin, imports
# ntdll+wow64 only) and FEXOfflineCompiler.exe in $BUILD_DIR/Bin/.
#
# Usage: tools/fex/build-fex.sh [builddir]   (default: build-fex-arm64)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FEX_SRC="$REPO_ROOT/external/fex"
BUILD_DIR="${1:-$REPO_ROOT/build-fex-arm64}"
LLVM_MINGW="$REPO_ROOT/.build-tools/llvm-mingw/bin"

[ -x "$LLVM_MINGW/aarch64-w64-mingw32-clang++" ] || {
    echo "FATAL: llvm-mingw not found at $LLVM_MINGW" >&2; exit 1; }
[ -f "$FEX_SRC/CMakeLists.txt" ] || {
    echo "FATAL: vendored FEX not found at $FEX_SRC" >&2; exit 1; }

export PATH="$LLVM_MINGW:$PATH"

mkdir -p "$BUILD_DIR"
cmake -S "$FEX_SRC" -B "$BUILD_DIR" -G Ninja \
    -DCMAKE_TOOLCHAIN_FILE=Data/CMake/toolchain_mingw.cmake \
    -DMINGW_TRIPLE=aarch64-w64-mingw32 \
    -DCMAKE_BUILD_TYPE=Release \
    -DENABLE_LTO=False \
    -DBUILD_TESTING=False \
    -DBUILD_FEXCONFIG=False \
    -DENABLE_ASSERTIONS=False

ninja -C "$BUILD_DIR" wow64fex FEXOfflineCompiler

echo
echo "Built:"
ls -la "$BUILD_DIR/Bin/"
