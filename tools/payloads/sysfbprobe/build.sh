#!/usr/bin/env bash
# tools/payloads/sysfbprobe/build.sh — cross-compile the sysfb PnP probe.
# Usage: tools/payloads/sysfbprobe/build.sh [arm64|amd64|i386|all]   Output: sysfbprobe-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
ARCHES="${1:-all}"; [ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"
declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )
for A in $ARCHES; do
    # GUI subsystem + classic msvcrt, same recipe and reasons as ../regdiff/build.sh. The CM_*
    # calls bind to setupapi, which implements them (cfgmgr32 only forwards there).
    "$TC/${CC[$A]}" -O2 -o "$HERE/sysfbprobe-$A.exe" "$HERE/sysfbprobe.c" \
        -lmsvcrt-os -lsetupapi -ladvapi32 -luser32 -lgdi32 -lntdll -lkernel32 \
        -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/sysfbprobe-$A.exe" --tag sysfbprobe --forbid-dll api-ms-win-crt
done
echo "[sysfbprobe] OK"
