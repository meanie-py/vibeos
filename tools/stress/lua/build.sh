#!/usr/bin/env bash
# tools/stress/lua/build.sh — fetch Lua and cross-compile lua32.exe, an i386
# WoW64 stress guest. Follows tools/stress/sqlite/build.sh and tools/doom/build.sh:
# the third-party source is FETCHED, never vendored, so nothing large lands in
# the repo.
#
# Usage:  tools/stress/lua/build.sh
# Output: tools/stress/lua/lua32.exe   (staged to syswow64 like the other
#         guests; see qemu/mkramdisk.sh and dll/win32/wow64cpu/CMakeLists.txt)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
SRC="$HERE/src"
VER="5.4.7"
URL="https://www.lua.org/ftp/lua-${VER}.tar.gz"

[ -x "$TC/i686-w64-mingw32-clang" ] || {
    echo "FATAL: llvm-mingw not found at $TC" >&2; exit 1; }

if [ ! -f "$SRC/lua.h" ]; then
    echo "[lua32] fetching Lua $VER..."
    mkdir -p "$SRC"
    curl -fsSL -o "$SRC/src.tar.gz" "$URL"
    tar -xzf "$SRC/src.tar.gz" -C "$SRC" --strip-components=2 "lua-${VER}/src"
    rm -f "$SRC/src.tar.gz"
fi

# lua.c and luac.c each carry their own main(); ours is Lua32.c, so exclude
# both. Everything else is the interpreter core plus the standard library --
# we deliberately keep the WHOLE stdlib rather than trimming it, because the
# point of a third-party stress guest is to run code that was not tailored to
# us. (Contrast sqlite32, which trims features: there the untrimmed build is
# mostly SQL surface we would only be emulating at TCG speed for no extra
# substrate coverage.)
LUA_C=()
for f in "$SRC"/*.c; do
    case "$(basename "$f")" in
        lua.c|luac.c) continue ;;
    esac
    LUA_C+=("$f")
done
[ ${#LUA_C[@]} -gt 20 ] || { echo "[lua32] FATAL: only ${#LUA_C[@]} core sources found in $SRC" >&2; exit 1; }

echo "[lua32] compiling i386 PE (${#LUA_C[@]} Lua sources)..."
# -lmsvcrt-os binds CLASSIC msvcrt.dll. Without it this toolchain defaults to
# the UCRT api-sets (api-ms-win-crt-*), which the tree does not ship as i386 --
# the guest would then fail to load with a missing-DLL error far from the cause.
# Same reason tools/doom/build.sh and tools/stress/sqlite/build.sh do it.
# -e mainCRTStartup matches.
#
# -O1, not -O2: Lua's interpreter loop is the hot spot and -O2 turns it into a
# computed-goto dispatch big enough to dominate FEX's translation time on the
# first pass. The control-flow stress we are after is in setjmp/longjmp, which
# -O1 preserves exactly.
"$TC/i686-w64-mingw32-gcc" -O1 -o "$HERE/lua32.exe" \
    -I"$SRC" \
    -Wl,--subsystem,windows:5.1 -Wl,--major-os-version,5 -Wl,--minor-os-version,1 \
    -Wl,-e,mainCRTStartup \
    "$HERE/Lua32.c" "${LUA_C[@]}" \
    -lmsvcrt-os -lkernel32 -luser32

# Guards, same shape as sqlite32's and doom's: a silent CRT flip is what this
# catches. `|| true` on the no-match (GOOD) case -- under `set -e` a failing
# grep would otherwise kill the script exactly when everything is fine.
OBJD="$TC/i686-w64-mingw32-objdump"
"$OBJD" -p "$HERE/lua32.exe" | grep -qi "api-ms-win-crt" \
    && { echo "[lua32] FATAL: links UCRT api-sets, want classic msvcrt.dll" >&2; exit 1; } \
    || true
"$OBJD" -p "$HERE/lua32.exe" | grep -qi "DLL Name: msvcrt.dll" \
    || { echo "[lua32] FATAL: does not import msvcrt.dll" >&2; exit 1; }

ls -la "$HERE/lua32.exe"
"$OBJD" -p "$HERE/lua32.exe" | grep "DLL Name"
