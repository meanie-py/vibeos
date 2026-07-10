#!/usr/bin/env bash
# tools/payloads/httpd/build.sh — cross-compile vibehttpd for the httpd payload.
# Usage: tools/payloads/httpd/build.sh [arm64|amd64|i386|all]   # default: all
# Outputs: tools/payloads/httpd/httpd-<arch>.exe (gitignored, ~80KB each;
# consumed via payload.yaml when VIBEOS_PAYLOAD=httpd).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"

ARCHES="${1:-all}"
[ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"

declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )

for A in $ARCHES; do
    echo "[httpd] building httpd-$A.exe"
    # Link recipe from tools/doom/build.sh: GUI subsystem (no CSRSS console
    # attach from the explorer startup walk) + classic msvcrt.dll (the UCRT
    # api-ms-win-crt-* default is NOT shipped by reactos — loader death with
    # zero serial evidence). Guarded below.
    "$TC/${CC[$A]}" -O2 -o "$HERE/httpd-$A.exe" "$HERE/vibehttpd.c" \
        -lmsvcrt-os -lws2_32 -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
    objd="$TC/${CC[$A]%-gcc}-objdump"
    ss=$("$objd" -p "$HERE/httpd-$A.exe" | awk '$1=="Subsystem" && !p {print $2; p=1}')
    [ "$ss" = "00000002" ] || { echo "[httpd] FAIL($A): subsystem=$ss (want GUI)"; exit 1; }
    "$objd" -p "$HERE/httpd-$A.exe" | grep -qi "api-ms-win-crt" \
        && { echo "[httpd] FAIL($A): links UCRT api-sets (want classic msvcrt.dll)"; exit 1; } \
        || true
done
echo "[httpd] done. Enable with: VIBEOS_PAYLOAD=httpd ./vibeos-cli build --arch <arch>"
