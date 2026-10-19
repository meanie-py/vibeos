#!/usr/bin/env bash
# tools/payloads/hostapps/build.sh — cross-compile fsquery (the host apps themselves are
# copied, not built). Usage: build.sh [amd64]   Output: fsquery-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:-amd64}"; [ "$A" = "all" ] && A=amd64
[ "$A" = amd64 ] || { echo "hostapps: amd64 only" >&2; exit 1; }
"$TC/x86_64-w64-mingw32-gcc" -O2 -municode -o "$HERE/fsquery-$A.exe" "$HERE/fsquery.c" \
    -lmsvcrt-os -lntdll -lkernel32
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/fsquery-$A.exe" --tag fsquery --forbid-dll api-ms-win-crt
echo "[hostapps] OK"
