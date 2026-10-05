#!/usr/bin/env bash
# tools/payloads/crtmath/build.sh — the i386 math entry points through every CRT DLL.
# Usage: tools/payloads/crtmath/build.sh [i386]   Output: crtmath-<arch>.exe
# _ftol, _CI* and __libm_sse2_* exist only on i386, so that is the only arch this
# builds for.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:-i386}"
[ "$A" = "i386" ] || { echo "[crtmath] FAIL: the _CI*/_ftol/__libm_sse2_* entry points are i386-only; got $A"; exit 1; }
# GUI subsystem + classic msvcrt, the same recipe as ../cpuprof/build.sh.
"$TC/i686-w64-mingw32-gcc" -O2 -msse2 -o "$HERE/crtmath-$A.exe" "$HERE/crtmath.c" \
    -lmsvcrt-os -lntdll -lkernel32 \
    -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
# Headers + import table read directly (tools/payloads/pecheck.py): the old
# `objdump -p | grep -q` under pipefail passed or failed at random on SIGPIPE.
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/crtmath-$A.exe" --tag crtmath --forbid-dll api-ms-win-crt
echo "[crtmath] OK"
