#!/usr/bin/env bash
# tools/payloads/chrome/build.sh — cross-compile the sweep runner and pipeprobe (Chrome itself is
# copied, not built). Usage: build.sh [amd64]
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:-amd64}"; [ "$A" = "all" ] && A=amd64
[ "$A" = amd64 ] || { echo "chrome: amd64 only" >&2; exit 1; }
bash "$ROOT/tools/payloads/apitest/build.sh" "$A"
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/pipeprobe-$A.exe" "$HERE/pipeprobe.c" -lmsvcrt-os -lntdll -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/catfile-$A.exe" "$HERE/catfile.c" -lmsvcrt-os -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/dumpinfo-$A.exe" "$HERE/dumpinfo.c" -lmsvcrt-os -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -shared -o "$HERE/tlsdll-$A.dll" "$HERE/tlsdll.c" -lmsvcrt-os -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/tlsprobe-$A.exe" "$HERE/tlsprobe.c" -lmsvcrt-os -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/secprobe-$A.exe" "$HERE/secprobe.c" -lmsvcrt-os -lkernel32
for P in pipeprobe catfile dumpinfo tlsprobe secprobe; do
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/$P-$A.exe" --tag $P --forbid-dll api-ms-win-crt
done
echo "[chrome] OK"
