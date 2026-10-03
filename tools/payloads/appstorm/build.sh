#!/usr/bin/env bash
# tools/payloads/appstorm/build.sh — cross-compile the GUI app and applet sweep.
# Usage: [APPSTORM_ONLY=name,name] [APPSTORM_PAGEHEAP=1] tools/payloads/appstorm/build.sh [arm64|amd64|i386|all]
# Output: appstorm-<arch>.exe. APPSTORM_ONLY compiles in a subset of the app table.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
ARCHES="${1:-all}"; [ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"
declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )
for A in $ARCHES; do
    # GUI subsystem + classic msvcrt, same recipe and reasons as ../regdiff/build.sh.
    "$TC/${CC[$A]}" -O2 ${APPSTORM_ONLY:+"-DAPPSTORM_ONLY=\"$APPSTORM_ONLY\""} ${APPSTORM_PAGEHEAP:+-DAPPSTORM_PAGEHEAP} -o "$HERE/appstorm-$A.exe" "$HERE/appstorm.c" \
        -lmsvcrt-os -lntdll -luser32 -ladvapi32 -lkernel32 \
        -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
    # Headers + import table read directly (tools/payloads/pecheck.py): the old
    # `objdump -p | grep -q` under pipefail passed or failed at random on SIGPIPE.
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/appstorm-$A.exe" --tag appstorm --forbid-dll api-ms-win-crt
done
echo "[appstorm] OK"
