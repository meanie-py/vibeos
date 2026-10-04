#!/usr/bin/env bash
# tools/payloads/wsprintf/build.sh — _wsprintf retry contract, both CRTs.
# Usage: tools/payloads/wsprintf/build.sh {i386|amd64|arm64}   Output: wsprintf-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:?arch}"
declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang [i386]=i686-w64-mingw32-clang )
"$TC/${CC[$A]}" -O1 -o "$HERE/wsprintf-$A.exe" "$HERE/wsprintf.c" -I"$HERE" \
    -lmsvcrt-os -luser32 -lntdll -lkernel32 \
    -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/wsprintf-$A.exe" --tag wsprintf --forbid-dll api-ms-win-crt
echo "[wsprintf] OK"
