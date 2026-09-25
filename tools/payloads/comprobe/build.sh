#!/usr/bin/env bash
# tools/payloads/comprobe/build.sh — cross-compile the COM marshalling / SHObjectProperties probe.
# Usage: tools/payloads/comprobe/build.sh [arm64|amd64|i386|all]   Output: comprobe-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
ARCHES="${1:-all}"; [ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"
declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )
for A in $ARCHES; do
    # GUI subsystem + classic msvcrt, same recipe and reasons as ../regdiff/build.sh.
    "$TC/${CC[$A]}" -O2 -o "$HERE/comprobe-$A.exe" "$HERE/comprobe.c" \
        -lmsvcrt-os -lntdll -lole32 -luuid -lshell32 -ladvapi32 -lkernel32 -luser32 -lkernel32 \
        -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
    "$TC/${CC[$A]%-gcc}-objdump" -p "$HERE/comprobe-$A.exe" | grep -qi "api-ms-win-crt" \
        && { echo "[comprobe] FAIL($A): links UCRT api-sets"; exit 1; } || true
done
echo "[comprobe] OK"
