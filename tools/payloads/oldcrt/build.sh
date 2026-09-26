#!/usr/bin/env bash
# tools/payloads/oldcrt/build.sh — the legacy CRT DLL load-and-answer probe (i386).
# Usage: tools/payloads/oldcrt/build.sh [i386]   Output: oldcrt-<arch>.exe
# msvcrt20, msvcrt40 and crtdll exist only on i386, so that is the only arch this
# builds for.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:-i386}"
[ "$A" = "i386" ] || { echo "[oldcrt] FAIL: msvcrt20/msvcrt40/crtdll are i386-only; got $A"; exit 1; }
# GUI subsystem + classic msvcrt, the same recipe as ../cpuprof/build.sh.
"$TC/i686-w64-mingw32-gcc" -O2 -o "$HERE/oldcrt-$A.exe" "$HERE/oldcrt.c" \
    -lmsvcrt-os -lntdll -lkernel32 \
    -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
# Headers + import table read directly (tools/payloads/pecheck.py): the old
# `objdump -p | grep -q` under pipefail passed or failed at random on SIGPIPE.
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/oldcrt-$A.exe" --tag oldcrt --forbid-dll api-ms-win-crt
echo "[oldcrt] OK"
