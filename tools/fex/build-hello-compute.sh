#!/usr/bin/env bash
# Build the rung-2 "hello compute" FEX harness (fexprobe.exe) — a native arm64
# Windows PE linking the prebuilt vendored FEXCore archives. Requires the FEX
# vendor build to exist first (tools/fex/build-fex.sh).
#
# Usage: tools/fex/build-hello-compute.sh [fex-builddir] [harness-builddir]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FEX_SRC="$REPO_ROOT/external/fex"
FEX_BUILD="${1:-$REPO_ROOT/build-fex-arm64}"
HARNESS_SRC="$REPO_ROOT/tools/fex/hello-compute"
HARNESS_BUILD="${2:-$REPO_ROOT/build-fex-hello}"
LLVM_MINGW="$REPO_ROOT/.build-tools/llvm-mingw/bin"

[ -f "$FEX_BUILD/FEXCore/Source/libFEXCore.a" ] || {
    echo "FATAL: FEX not built at $FEX_BUILD — run tools/fex/build-fex.sh first" >&2
    exit 1; }

export PATH="$LLVM_MINGW:$PATH"

mkdir -p "$HARNESS_BUILD"
cmake -S "$HARNESS_SRC" -B "$HARNESS_BUILD" -G Ninja \
    -DCMAKE_TOOLCHAIN_FILE="$FEX_SRC/Data/CMake/toolchain_mingw.cmake" \
    -DMINGW_TRIPLE=aarch64-w64-mingw32 \
    -DCMAKE_BUILD_TYPE=Release \
    -DFEX_SRC="$FEX_SRC" \
    -DFEX_BUILD="$FEX_BUILD"

ninja -C "$HARNESS_BUILD" fexprobe wow64probe

# hello32.exe — the i386 WoW64 Phase C guest (no CRT, no imports). Built with
# a direct clang call because the harness CMake is aarch64-target; this is the
# only i386 artifact here.
# GUI subsystem, NOT console: a CUI process makes kernel32's ConDllInitialize
# allocate a console via csrss, which hangs for a winlogon-spawned WoW64 process
# (no console to inherit). The GUI probes (fexprobe/wow64probe) work precisely
# because they skip console init. See wow64-plan.md Phase C M2.
"$LLVM_MINGW/i686-w64-mingw32-clang" -O1 -nostdlib -Wl,-e,_start \
    -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
    -o "$HARNESS_BUILD/hello32.exe" "$HARNESS_SRC/Hello32.c"

echo
echo "Built:"
ls -la "$HARNESS_BUILD"/fexprobe.exe "$HARNESS_BUILD"/wow64probe.exe "$HARNESS_BUILD"/hello32.exe
