#!/usr/bin/env bash
# tools/payloads/matherrs/build.sh — Wine's ucrtbase test_math_errors, traced per call.
# Usage: tools/payloads/matherrs/build.sh {i386|amd64|arm64}   Output: matherrs-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:?arch}"
declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang [i386]=i686-w64-mingw32-clang )
"$TC/${CC[$A]}" -O1 -o "$HERE/matherrs-$A.exe" "$HERE/matherrs.c" -I"$HERE" \
    -lmsvcrt-os -lntdll -lkernel32 \
    -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/matherrs-$A.exe" --tag matherrs --forbid-dll api-ms-win-crt
echo "[matherrs] OK"
