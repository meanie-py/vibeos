#!/usr/bin/env bash
# tools/payloads/locking/build.sh — _locking retry contract, both CRTs.
# Usage: tools/payloads/locking/build.sh {i386|amd64|arm64}   Output: locking-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:?arch}"
declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang [i386]=i686-w64-mingw32-clang )
"$TC/${CC[$A]}" -O1 -o "$HERE/locking-$A.exe" "$HERE/locking.c" -I"$HERE" \
    -lmsvcrt-os -lntdll -lkernel32 \
    -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/locking-$A.exe" --tag locking --forbid-dll api-ms-win-crt
echo "[locking] OK"
