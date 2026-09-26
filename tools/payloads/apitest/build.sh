#!/usr/bin/env bash
# tools/payloads/apitest/build.sh — cross-compile the apitest sweep runner.
# Usage: tools/payloads/apitest/build.sh [arm64|amd64|i386|all]   # default: all
# Outputs: tools/payloads/apitest/apitest-sweep-<arch>.exe (gitignored;
# consumed via payload.yaml when VIBEOS_PAYLOAD=apitest).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"

ARCHES="${1:-all}"
[ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"

declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )

for A in $ARCHES; do
    echo "[apitest] building apitest-sweep-$A.exe"
    # Same link recipe as ../kmtest/build.sh, and for the same two reasons:
    # GUI subsystem (no CSRSS console attach from the explorer startup walk)
    # and classic msvcrt.dll (reactos ships no UCRT api-ms-win-crt-* set, so
    # the UCRT default dies in the loader with zero serial evidence).
    "$TC/${CC[$A]}" -O2 -o "$HERE/apitest-sweep-$A.exe" "$HERE/sweep.c" \
        -lmsvcrt-os -lntdll -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
    # Headers + import table read directly (tools/payloads/pecheck.py): the old
    # `objdump -p | grep -q` under pipefail passed or failed at random on SIGPIPE.
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/apitest-sweep-$A.exe" --tag apitest --gui --forbid-dll api-ms-win-crt
done
echo "[apitest] OK"
