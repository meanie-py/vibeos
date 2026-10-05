#!/usr/bin/env bash
# tools/payloads/fpctrl/build.sh — ucrtbase vs msvcrt FP control family (arm64, amd64).
# Usage: tools/payloads/fpctrl/build.sh [arm64|amd64]   Output: fpctrl-<arch>.exe
# Both DLLs are LoadLibrary'd, so the exe itself imports neither's FP functions.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:-arm64}"
# amd64 joined when ucrtbase's MXCSR family moved from sdk/lib/crt's C to ucrt-rust's
# fpctrl_amd64.rs (msvcrt's is Wine's _setfp, already Rust). i386 is not covered here.
declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang )
[ -n "${CC[$A]:-}" ] || { echo "[fpctrl] FAIL: arm64/amd64 only; got $A"; exit 1; }
"$TC/${CC[$A]}" -O1 -o "$HERE/fpctrl-$A.exe" "$HERE/fpctrl.c" \
    -lmsvcrt-os -lntdll -lkernel32 \
    -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/fpctrl-$A.exe" --tag fpctrl --forbid-dll api-ms-win-crt
echo "[fpctrl] OK"
