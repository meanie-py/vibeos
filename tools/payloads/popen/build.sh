#!/usr/bin/env bash
# tools/payloads/popen/build.sh — msvcrt _popen errno probe (misc.c:375).
# Usage: tools/payloads/popen/build.sh {i386|amd64|arm64}   Output: popen-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:?arch}"
declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang [i386]=i686-w64-mingw32-clang )
"$TC/${CC[$A]}" -O1 -o "$HERE/popen-$A.exe" "$HERE/popen.c" -I"$HERE" \
    -lmsvcrt-os -lntdll -lkernel32 \
    -Wl,--subsystem,console -Wl,-e,mainCRTStartup
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/popen-$A.exe" --tag popen --forbid-dll api-ms-win-crt
echo "[popen] OK"
