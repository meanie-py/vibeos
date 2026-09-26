#!/usr/bin/env bash
# tools/payloads/shelltail/build.sh — cross-compile the shell32-glue A/B checker and stage the
# baseline shell32 it compares against.
# Usage: SHELLTAIL_BASE=<worktree> tools/payloads/shelltail/build.sh [arm64|amd64|i386|all]
#   SHELLTAIL_BASE is a checkout whose build-<arch>/ holds a BASELINE shell32.dll that still
#   carries the C/C++ glue (anything before 2026-09-24). The DLL under test is whatever this
#   tree stages as system32\shell32.dll.
# Outputs (gitignored): shelltail-<arch>.exe, shelltail-base-<arch>.dll,
# shelltail-base-<arch>.txt (the baseline's commit, so a result names what it was compared with).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
: "${SHELLTAIL_BASE:?set SHELLTAIL_BASE to the baseline checkout (see header)}"

ARCHES="${1:-all}"
[ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"

declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )

for A in $ARCHES; do
    base="$SHELLTAIL_BASE/build-$A/dll/win32/shell32/shell32.dll"
    [ -f "$base" ] || { echo "[shelltail] FAIL($A): no baseline DLL at $base"; exit 1; }
    cp "$base" "$HERE/shelltail-base-$A.dll"
    git -C "$SHELLTAIL_BASE" log -1 --format='%H %s' > "$HERE/shelltail-base-$A.txt"
    echo "[shelltail] building shelltail-$A.exe (baseline $(cut -c1-12 "$HERE/shelltail-base-$A.txt"))"
    # GUI subsystem + classic msvcrt, same recipe and reasons as ../regdiff/build.sh.
    "$TC/${CC[$A]}" -O2 -o "$HERE/shelltail-$A.exe" "$HERE/shelltail.c" \
        -lmsvcrt-os -lntdll -luser32 -lkernel32 -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
    # Headers + import table read directly (tools/payloads/pecheck.py): the old
    # `objdump -p | grep -q` under pipefail passed or failed at random on SIGPIPE.
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/shelltail-$A.exe" --tag shelltail --forbid-dll api-ms-win-crt --forbid-dll shell32
done
echo "[shelltail] OK"
