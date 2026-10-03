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
    # Headers + import table read directly (tools/payloads/pecheck.py): the old
    # `objdump -p | grep -q` under pipefail passed or failed at random on SIGPIPE.
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/cplreuse-$A.exe" --tag cplreuse --forbid-dll api-ms-win-crt
done
echo "[cplreuse] OK"
