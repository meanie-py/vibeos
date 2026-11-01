#!/usr/bin/env bash
# tools/payloads/hostapps/build.sh — cross-compile fsquery, regsd, w11api and w11tok (the host apps themselves are
# copied, not built). Usage: build.sh [amd64]   Output: fsquery-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:-amd64}"; [ "$A" = "all" ] && A=amd64
[ "$A" = amd64 ] || { echo "hostapps: amd64 only" >&2; exit 1; }
"$TC/x86_64-w64-mingw32-gcc" -O2 -municode -o "$HERE/fsquery-$A.exe" "$HERE/fsquery.c" \
    -lmsvcrt-os -lntdll -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -municode -o "$HERE/regsd-$A.exe" "$HERE/regsd.c" \
    -lmsvcrt-os -lntdll -ladvapi32 -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -municode -o "$HERE/w11api-$A.exe" "$HERE/w11api.c" -lmsvcrt-os -lntdll -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/w11tok-$A.exe" "$HERE/w11tok.c" -lmsvcrt-os -lntdll -ladvapi32 -lkernel32
for P in fsquery regsd w11api w11tok; do
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/$P-$A.exe" --tag $P --forbid-dll api-ms-win-crt
done
echo "[hostapps] OK"
