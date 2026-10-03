#!/usr/bin/env bash
# tools/payloads/fpctrl/build.sh — ucrtbase vs msvcrt FP control family (arm64).
# Usage: tools/payloads/fpctrl/build.sh [arm64]   Output: fpctrl-<arch>.exe
# Both DLLs are LoadLibrary'd, so the exe itself imports neither's FP functions.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:-arm64}"
[ "$A" = "arm64" ] || { echo "[fpctrl] FAIL: arm64-only (i386/amd64 never forwarded); got $A"; exit 1; }
"$TC/aarch64-w64-mingw32-clang" -O1 -o "$HERE/fpctrl-$A.exe" "$HERE/fpctrl.c" \
    -lmsvcrt-os -lntdll -lkernel32 \
    -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/fpctrl-$A.exe" --tag fpctrl --forbid-dll api-ms-win-crt
echo "[fpctrl] OK"
