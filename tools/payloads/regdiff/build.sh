#!/usr/bin/env bash
# tools/payloads/regdiff/build.sh — cross-compile the registration differ and stage the
# baseline DLL it compares against.
# Usage: REGDIFF_BASE=<worktree> tools/payloads/regdiff/build.sh [arm64|amd64|i386|all]
#   REGDIFF_BASE is a checkout whose build-<arch>/ holds the BASELINE shell32.dll (e.g. a
#   detached worktree at the merge base, built with ./vibeos-cli build). The DLL under test is
#   whatever this tree stages as system32\shell32.dll.
# Outputs (gitignored): regdiff-<arch>.exe, regdiff-base-<arch>.dll, regdiff-base-<arch>.txt
# (the baseline's commit, so a result names what it was compared with).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
: "${REGDIFF_BASE:?set REGDIFF_BASE to the baseline checkout (see header)}"

ARCHES="${1:-all}"
[ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"

declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )

for A in $ARCHES; do
    base="$REGDIFF_BASE/build-$A/dll/win32/shell32/shell32.dll"
    [ -f "$base" ] || { echo "[regdiff] FAIL($A): no baseline DLL at $base"; exit 1; }
    cp "$base" "$HERE/regdiff-base-$A.dll"
    git -C "$REGDIFF_BASE" log -1 --format='%H %s' > "$HERE/regdiff-base-$A.txt"
    echo "[regdiff] building regdiff-$A.exe (baseline $(cut -c1-12 "$HERE/regdiff-base-$A.txt"))"
    # GUI subsystem + classic msvcrt, same recipe and reasons as ../procstorm/build.sh.
    "$TC/${CC[$A]}" -O2 -o "$HERE/regdiff-$A.exe" "$HERE/regdiff.c" \
        -lmsvcrt-os -lntdll -ladvapi32 -lole32 -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
    objd="$TC/${CC[$A]%-gcc}-objdump"
    "$objd" -p "$HERE/regdiff-$A.exe" | grep -qi "api-ms-win-crt" \
        && { echo "[regdiff] FAIL($A): links UCRT api-sets (want classic msvcrt.dll)"; exit 1; } \
        || true
    "$objd" -p "$HERE/regdiff-$A.exe" | grep -qi "DLL Name: shell32" \
        && { echo "[regdiff] FAIL($A): imports shell32 (the children must load it themselves)"; exit 1; } \
        || true
done
echo "[regdiff] OK"
