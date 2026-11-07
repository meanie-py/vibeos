#!/usr/bin/env bash
# tools/payloads/chrome/build.sh — cross-compile the sweep runner and the probes (Chrome itself is
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
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/iocpprobe-$A.exe" "$HERE/iocpprobe.c" -lmsvcrt-os -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/dupprobe-$A.exe" "$HERE/dupprobe.c" -lmsvcrt-os -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/msgprobe-$A.exe" "$HERE/msgprobe.c" -lmsvcrt-os -lkernel32 -luser32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/waitprobe-$A.exe" "$HERE/waitprobe.c" -lmsvcrt-os -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/servepage-$A.exe" "$HERE/servepage.c" -lmsvcrt-os -lws2_32 -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/sockprobe-$A.exe" "$HERE/sockprobe.c" -lmsvcrt-os -lws2_32 -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/grepfile-$A.exe" "$HERE/grepfile.c" -lmsvcrt-os -lkernel32
"$TC/x86_64-w64-mingw32-gcc" -O2 -o "$HERE/gdiprobe-$A.exe" "$HERE/gdiprobe.c" -lmsvcrt-os -lkernel32 -luser32 -lgdi32
# And 32-bit, to go through WOW64 like the app whose DC leak wedged long boots.
# A GUI-subsystem image like the stock apps: a console one died in WOW64 start-up (activation
# context, RtlReAllocateHeap raising STATUS_NO_MEMORY). It still writes to the std handles given.
"$ROOT/.build-tools/llvm-mingw/bin/i686-w64-mingw32-gcc" -O2 -mwindows -o "$HERE/gdiprobe-i386.exe" "$HERE/gdiprobe.c" -lmsvcrt-os -lkernel32 -luser32 -lgdi32
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/gdiprobe-i386.exe" --tag gdiprobe32 --forbid-dll api-ms-win-crt
for P in pipeprobe catfile dumpinfo tlsprobe secprobe iocpprobe dupprobe msgprobe waitprobe servepage sockprobe grepfile gdiprobe; do
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/$P-$A.exe" --tag $P --forbid-dll api-ms-win-crt
done
echo "[chrome] OK"
