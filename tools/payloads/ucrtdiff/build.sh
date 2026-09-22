#!/usr/bin/env bash
# tools/payloads/ucrtdiff/build.sh — build the Tier 2 ucrt difftest harness.
# Usage: tools/payloads/ucrtdiff/build.sh [arm64|amd64|i386|all]   # default: all
# Outputs: tools/payloads/ucrtdiff/ucrtdiff-<arch>.exe (gitignored; consumed via
# payload.yaml when VIBEOS_PAYLOAD=ucrtdiff).
#
# The exe links TWO implementations of the same exports:
#   libucrt_ref.a        genuine Microsoft C, exports renamed ref_* by
#                        tools/ucrt_tier2_refset.py
#   libucrtbase.a        the import lib for our SHIPPING ucrtbase.dll
# so `ref_strtol` is Microsoft's body and `strtol` is whatever we ship.
#
# Needs a built tree: the reference set is made from the objects that built
# ucrtbase.dll, taken from ninja's own link line. Hard-fails if it is missing
# rather than emitting a harness that silently grades nothing.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"

ARCHES="${1:-all}"
[ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"

declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang [i386]=i686-w64-mingw32-clang )
declare -A RT=( [arm64]=aarch64 [amd64]=x86_64 [i386]=i386 )

for A in $ARCHES; do
    B="$ROOT/build-$A"
    [ -d "$B" ] || { echo "[ucrtdiff] FAIL($A): no build-$A — fix: ./vibeos-cli build --arch $A"; exit 1; }

    echo "[ucrtdiff] building the $A reference set"
    python3 "$ROOT/tools/ucrt_tier2_refset.py" --arch "$A"
    REF="$B/libucrt_ref.a"
    [ -s "$REF" ] || { echo "[ucrtdiff] FAIL($A): $REF missing"; exit 1; }

    # The THIRD side: our existing msvcrt Rust bodies, renamed rust_*. This is what
    # makes the harness an oracle rather than a control -- while a family is still
    # answered by the vendored C, ref_ and the DLL are the SAME code and agreeing
    # proves nothing about us.
    echo "[ucrtdiff] building the $A msvcrt-rust set"
    python3 "$ROOT/tools/ucrt_tier2_refset.py" --arch "$A" --side msvcrt-rust
    RUST="$B/libucrt_rust.a"
    [ -s "$RUST" ] || { echo "[ucrtdiff] FAIL($A): $RUST missing"; exit 1; }

    # The ucrtbase IMPORT LIB is what makes the bare side resolve to our shipping
    # DLL. It is a normal build target, but only gets built when something links
    # it -- on arm64 nothing does (ucrtbase_winetest is an i386/amd64 rostest),
    # so it is routinely absent there. Build that one target rather than failing:
    # it is an import lib generated from the .def and costs seconds.
    IMPLIB="$B/dll/win32/ucrtbase/CMakeFiles/libucrtbase.dir/libucrtbase.a"
    if [ ! -s "$IMPLIB" ]; then
        echo "[ucrtdiff] $A: libucrtbase import lib absent, building it"
        (cd "$ROOT" && bash tools/canon-run.sh ninja -C "build-$A" libucrtbase) \
            || { echo "[ucrtdiff] FAIL($A): could not build libucrtbase"; exit 1; }
        [ -s "$IMPLIB" ] || { echo "[ucrtdiff] FAIL($A): libucrtbase built but $IMPLIB is missing"; exit 1; }
    fi

    # compiler-rt lives beside the toolchain, not in it: the build's own link
    # line names an absolute path, so read it from there rather than guessing.
    BUILTINS=$(cd "$ROOT" && bash tools/canon-run.sh ninja -C "build-$A" -t commands ucrtbase 2>/dev/null \
        | tail -1 | tr ' ' '\n' | grep -m1 'libclang_rt.builtins' || true)
    [ -n "$BUILTINS" ] || { echo "[ucrtdiff] FAIL($A): no compiler-rt builtins on ucrtbase's link line"; exit 1; }

    # Regenerate the case list every time: it is derived from the burn-down (which moves
    # as families are taken over) and from the public headers (which move with upstream).
    # A stale .inc would keep grading exports that are no longer Microsoft's.
    echo "[ucrtdiff] generating $A cases"
    python3 "$ROOT/tools/gen_ucrt_tier2_cases.py" --arch "$A" --out "$HERE/generated.inc"

    echo "[ucrtdiff] linking ucrtdiff-$A.exe"
    # -fno-builtin is LOAD-BEARING, not hygiene: clang knows what isdigit, strlen
    # and friends mean and will fold or inline them, so the "ours" side silently
    # stops being a call into ucrtbase.dll and the row grades the COMPILER against
    # Microsoft. isdigit vanished from the import table exactly this way.
    "$TC/${CC[$A]}" -O2 -fno-builtin -nostdlib -o "$HERE/ucrtdiff-$A.exe" "$HERE/ucrtdiff.c" \
        -Wl,--start-group \
        "$REF" \
        "$RUST" \
        "$IMPLIB" \
        "$B/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a" \
        "$B/dll/ntdll/CMakeFiles/libntdll.dir/libntdll.a" \
        "$B/sdk/lib/vcstartup/libvcstartup.a" \
        "$B/sdk/lib/vcruntime/libvcruntime.a" \
        "$B/sdk/lib/crt/libcrtmath.a" \
        "$B/sdk/lib/crt/libchkstk.a" \
        "$B/sdk/lib/pseh/libpseh.a" \
        "$B/sdk/lib/libwine/libwine.a" \
        "$B/dll/win32/advapi32/CMakeFiles/libadvapi32.dir/libadvapi32.a" \
        "$B/dll/win32/kernel32/kernel32_vista/CMakeFiles/libkernel32_vista.dir/libkernel32_vista.a" \
        "$B/win32ss/user/user32/CMakeFiles/libuser32.dir/libuser32.a" \
        "$BUILTINS" \
        -Wl,--end-group \
        -Wl,--allow-multiple-definition \
        -Wl,--subsystem,windows -Wl,-e,mainCRTStartup

    objd="$TC/$(echo "${CC[$A]}" | sed 's/-clang$//')-objdump"
    # GUI subsystem: a console exe launched from explorer's startup walk gets no
    # CSRSS console attach and dies without a word on serial (kmtest's lesson).
    ss=$("$objd" -p "$HERE/ucrtdiff-$A.exe" | awk '$1=="Subsystem" && !p {print $2; p=1}')
    [ "$ss" = "00000002" ] || { echo "[ucrtdiff] FAIL($A): subsystem=$ss (want GUI)"; exit 1; }
    # It must import ucrtbase.dll directly. If it ever links the api-ms-win-crt-*
    # sets instead, the bare side stops being OUR DLL and the harness silently
    # grades Microsoft's C against Microsoft's C.
    "$objd" -p "$HERE/ucrtdiff-$A.exe" | grep -qi "ucrtbase.dll" \
        || { echo "[ucrtdiff] FAIL($A): does not import ucrtbase.dll"; exit 1; }
    "$objd" -p "$HERE/ucrtdiff-$A.exe" | grep -qi "api-ms-win-crt" \
        && { echo "[ucrtdiff] FAIL($A): links UCRT api-sets (want ucrtbase.dll direct)"; exit 1; } \
        || true
    # EVERY graded function must appear in the ucrtbase import table. A row whose
    # "ours" side got inlined or constant-folded still PASSES -- it just compares
    # Microsoft's body against the compiler's idea instead of against our DLL, and
    # nothing in the output says so. This caught isdigit before -fno-builtin.
    imports=$("$objd" -p "$HERE/ucrtdiff-$A.exe" | sed -n '/ucrtbase.dll/,/^$/p')
    missing=""
    # Only real DECLARATIONS: `__cdecl ref_name(...);`. Matching ref_ anywhere would
    # also pick up the prose in this file's comments.
    for fn in $(grep -oE '__cdecl[[:space:]]+\*?ref_[A-Za-z_][A-Za-z0-9_]*[[:space:]]*\(' \
                "$HERE/ucrtdiff.c" | sed 's/.*ref_//; s/[[:space:]]*($//; s/(//' | sort -u); do
        echo "$imports" | grep -qE "[[:space:]]$fn\$" || missing="$missing $fn"
    done
    [ -z "$missing" ] || {
        echo "[ucrtdiff] FAIL($A): graded but NOT imported from ucrtbase.dll:$missing"
        echo "[ucrtdiff]   those rows would grade the compiler, not our DLL (see -fno-builtin)"
        exit 1
    }
    echo "[ucrtdiff] ok($A): $(stat -c%s "$HERE/ucrtdiff-$A.exe") bytes, $(echo "$missing" | wc -w) ungraded"
done
