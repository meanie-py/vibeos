#!/usr/bin/env bash
# tools/payloads/fenv/build.sh — ucrtbase's fenv family vs Wine's test_fenv.
# Usage: tools/payloads/fenv/build.sh {i386|amd64|arm64}   Output: fenv-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:?arch}"
declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang [i386]=i686-w64-mingw32-clang )
"$TC/${CC[$A]}" -O1 -o "$HERE/fenv-$A.exe" "$HERE/fenv.c" \
    -lmsvcrt-os -lntdll -lkernel32 \
    -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/fenv-$A.exe" --tag fenv --forbid-dll api-ms-win-crt
echo "[fenv] OK"
