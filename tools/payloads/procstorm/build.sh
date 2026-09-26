#!/usr/bin/env bash
# tools/payloads/procstorm/build.sh — cross-compile the #39 process-storm amplifier.
# Usage: tools/payloads/procstorm/build.sh [arm64|amd64|i386|all]   # default: all
# Outputs: tools/payloads/procstorm/procstorm-<arch>.exe (gitignored;
# consumed via payload.yaml when VIBEOS_PAYLOAD=procstorm).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"

ARCHES="${1:-all}"
[ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"

declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )

for A in $ARCHES; do
    echo "[procstorm] building procstorm-$A.exe"
    # Same link recipe as ../kmtest/build.sh: GUI subsystem (no CSRSS console
    # attach from the explorer startup walk) + classic msvcrt.dll (the UCRT
    # api-ms-win-crt-* default is NOT shipped by reactos — loader death with
    # zero serial evidence). DbgPrint comes from ntdll.
    "$TC/${CC[$A]}" -O2 -o "$HERE/procstorm-$A.exe" "$HERE/storm.c" \
        -lmsvcrt-os -lntdll -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
    # Headers + import table read directly (tools/payloads/pecheck.py): the old
    # `objdump -p | grep -q` under pipefail passed or failed at random on SIGPIPE.
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/procstorm-$A.exe" --tag procstorm --gui --forbid-dll api-ms-win-crt
done
echo "[procstorm] OK"
