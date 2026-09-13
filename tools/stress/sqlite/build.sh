#!/usr/bin/env bash
# tools/stress/sqlite/build.sh — fetch SQLite and cross-compile sqlite32.exe,
# an i386 WoW64 stress guest. Follows tools/doom/build.sh: the third-party
# source is FETCHED, never vendored, so nothing large lands in the repo.
#
# Usage:  tools/stress/sqlite/build.sh
# Output: tools/stress/sqlite/sqlite32.exe   (staged to syswow64 like the
#         other guests; see qemu/mkramdisk.sh and dll/win32/wow64cpu/CMakeLists.txt)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
AMALG="$HERE/amalg"
VER="3450100"
URL="https://sqlite.org/2024/sqlite-amalgamation-${VER}.zip"

[ -x "$TC/i686-w64-mingw32-clang" ] || {
    echo "FATAL: llvm-mingw not found at $TC" >&2; exit 1; }

if [ ! -f "$AMALG/sqlite3.c" ]; then
    echo "[sqlite32] fetching SQLite amalgamation $VER..."
    mkdir -p "$AMALG"
    curl -fsSL -o "$AMALG/src.zip" "$URL"
    ( cd "$AMALG" && unzip -qo src.zip && mv sqlite-amalgamation-*/sqlite3.[ch] . && rm -rf sqlite-amalgamation-* src.zip )
fi

# Trimmed feature set: this is a substrate stress test, not a SQL demo, and
# every feature left out is code we do not have to emulate at TCG speed.
DEFS=(
  -DSQLITE_OS_WIN=1
  -DSQLITE_THREADSAFE=0
  -DSQLITE_OMIT_LOAD_EXTENSION
  -DSQLITE_DEFAULT_MEMSTATUS=0
  -DSQLITE_MAX_EXPR_DEPTH=0
)

echo "[sqlite32] compiling i386 PE..."
# -lmsvcrt-os binds CLASSIC msvcrt.dll. Without it this toolchain defaults to
# the UCRT api-sets (api-ms-win-crt-*), which the tree does not ship as i386 --
# the guest would then fail to load with a missing-DLL error far from the cause.
# Same reason tools/doom/build.sh does it. -e mainCRTStartup matches.
"$TC/i686-w64-mingw32-gcc" -O1 -o "$HERE/sqlite32.exe" \
    -I"$AMALG" "${DEFS[@]}" \
    -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
    -Wl,-e,mainCRTStartup \
    "$HERE/Sqlite32.c" "$AMALG/sqlite3.c" \
    -lmsvcrt-os -lkernel32 -luser32

# Guards, same shape as doom's: a silent CRT flip is what this catches.
# `|| true` on the no-match (GOOD) case -- under `set -e` a failing grep would
# otherwise kill the script exactly when everything is fine.
OBJD="$TC/i686-w64-mingw32-objdump"
"$OBJD" -p "$HERE/sqlite32.exe" | grep -qi "api-ms-win-crt" \
    && { echo "[sqlite32] FATAL: links UCRT api-sets, want classic msvcrt.dll" >&2; exit 1; } \
    || true
"$OBJD" -p "$HERE/sqlite32.exe" | grep -qi "DLL Name: msvcrt.dll" \
    || { echo "[sqlite32] FATAL: does not import msvcrt.dll" >&2; exit 1; }

ls -la "$HERE/sqlite32.exe"
"$OBJD" -p "$HERE/sqlite32.exe" | grep "DLL Name"
