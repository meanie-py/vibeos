#!/usr/bin/env bash
# tools/payloads/neonctx/build.sh — cross-compile the arm64 V-register exception round-trip check.
# Usage: tools/payloads/neonctx/build.sh [arm64]    (arm64 only: it is arm64 vector state)
# Output (gitignored): neonctx-arm64.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:-arm64}"
[ "$A" = "arm64" ] || { echo "[neonctx] FAIL: arm64 only (got $A)"; exit 1; }
# GUI subsystem + classic msvcrt, same recipe and reasons as ../regdiff/build.sh.
"$TC/aarch64-w64-mingw32-gcc" -O2 -o "$HERE/neonctx-arm64.exe" "$HERE/neonctx.c" \
    -lmsvcrt-os -lntdll -lkernel32 -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
# Headers + import table read directly (tools/payloads/pecheck.py): the old
# `objdump -p | grep -q` under pipefail passed or failed at random on SIGPIPE.
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/neonctx-arm64.exe" --tag neonctx --forbid-dll api-ms-win-crt
echo "[neonctx] OK"
