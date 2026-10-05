#!/usr/bin/env bash
# tools/payloads/vaentry/build.sh — the CRT `...` exports (va_entry! thunks), msvcrt + ucrtbase.
# Usage: tools/payloads/vaentry/build.sh {i386|amd64|arm64}   Output: vaentry-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:?arch}"
declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang [i386]=i686-w64-mingw32-clang )
"$TC/${CC[$A]}" -O1 -o "$HERE/vaentry-$A.exe" "$HERE/vaentry.c" -I"$HERE" \
    -lmsvcrt-os -lntdll -lkernel32 \
    -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/vaentry-$A.exe" --tag vaentry --forbid-dll api-ms-win-crt
echo "[vaentry] OK"
