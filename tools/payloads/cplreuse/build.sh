#!/usr/bin/env bash
# tools/payloads/cplreuse/build.sh — cross-compile the DLL VA-reuse probe.
# Usage: tools/payloads/cplreuse/build.sh [arm64|amd64|i386|all]   Output: cplreuse-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
ARCHES="${1:-all}"; [ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"
declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )
for A in $ARCHES; do
    # GUI subsystem + classic msvcrt, same recipe and reasons as ../regdiff/build.sh.
    "$TC/${CC[$A]}" -O2 -o "$HERE/cplreuse-$A.exe" "$HERE/cplreuse.c" \
        -lmsvcrt-os -lntdll -lkernel32 \
        -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
    "$TC/${CC[$A]%-gcc}-objdump" -p "$HERE/cplreuse-$A.exe" | grep -qi "api-ms-win-crt" \
        && { echo "[cplreuse] FAIL($A): links UCRT api-sets"; exit 1; } || true
done
echo "[cplreuse] OK"
