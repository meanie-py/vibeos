#!/usr/bin/env bash
# tools/payloads/regcopyprobe/build.sh — RegCopyTreeW over a control set.
# Usage: tools/payloads/regcopyprobe/build.sh {i386|amd64|arm64|all}   Output: regcopyprobe-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
ARCHES="${1:?arch}"; [ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"
declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang [i386]=i686-w64-mingw32-clang )
for A in $ARCHES; do
    "$TC/${CC[$A]}" -O1 -o "$HERE/regcopyprobe-$A.exe" "$HERE/regcopyprobe.c" \
        -lmsvcrt-os -lntdll -lkernel32 -ladvapi32 \
        -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/regcopyprobe-$A.exe" --tag regcopyprobe --forbid-dll api-ms-win-crt
done
echo "[regcopyprobe] OK"
