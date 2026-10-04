#!/usr/bin/env bash
# tools/payloads/c99math/build.sh — ucrtbase's C99 math functions, value + errno.
# expected.h is regenerated from cases.h with the host glibc libm before each build.
# Usage: tools/payloads/c99math/build.sh {i386|amd64|arm64}   Output: c99math-<arch>.exe
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:?arch}"
GEN="$(mktemp -d)"
cc -O0 -fno-builtin -o "$GEN/gen" "$HERE/gen_expected.c" -lm
"$GEN/gen" > "$HERE/expected.h"
rm -rf "${GEN:?}"
declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang [i386]=i686-w64-mingw32-clang )
"$TC/${CC[$A]}" -O1 -o "$HERE/c99math-$A.exe" "$HERE/c99math.c" -I"$HERE" \
    -lmsvcrt-os -lntdll -lkernel32 \
    -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/c99math-$A.exe" --tag c99math --forbid-dll api-ms-win-crt
echo "[c99math] OK"
