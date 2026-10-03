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

# UCRTDIFF_SHARD=k/N builds a harness that grades only exports k, k+N, k+2N...
# One boot used to be one shot at all 220 exports, so whatever stopped a long run
# cost every export after it -- and the stopping point moved with emission order,
# which made each table a different PREFIX of the truth. A shard finishes well
# inside any observed wall, and N boots cover the set. Default 0/1 = everything.
# UCRTDIFF_ONLY=rename,_mktemp grades just those exports (any shard split still applies).
SHARD="${UCRTDIFF_SHARD:-0/1}"
SHARD_K="${SHARD%%/*}"; SHARD_N="${SHARD##*/}"
case "$SHARD_K$SHARD_N" in *[!0-9]*|"") echo "[ucrtdiff] FAIL: UCRTDIFF_SHARD=$SHARD is not k/N"; exit 1;; esac
[ "$SHARD_N" -ge 1 ] && [ "$SHARD_K" -lt "$SHARD_N" ] \
    || { echo "[ucrtdiff] FAIL: UCRTDIFF_SHARD=$SHARD needs 0 <= k < N"; exit 1; }
# The harness's own grading deadline. Hitting it prints a `done` line saying
# partial=<n>, which is what makes a cut-short run a parseable result rather than a
# serial log that simply stops.
BUDGET_MS="${UCRTDIFF_BUDGET_MS:-600000}"
echo "[ucrtdiff] shard=$SHARD_K/$SHARD_N budget=${BUDGET_MS}ms"

declare -A CC=( [arm64]=aarch64-w64-mingw32-clang [amd64]=x86_64-w64-mingw32-clang [i386]=i686-w64-mingw32-clang )
declare -A RT=( [arm64]=aarch64 [amd64]=x86_64 [i386]=i386 )

for A in $ARCHES; do
    B="$ROOT/build-$A"
    [ -d "$B" ] || { echo "[ucrtdiff] FAIL($A): no build-$A — fix: ./vibeos-cli build --arch $A"; exit 1; }

    REF="$B/libucrt_ref.a"
    RUST="$B/libucrt_rust.a"
    OURS="$B/libucrt_ours.a"
    # UCRTDIFF_REUSE_INPUTS=1 keeps the reference set, the rust set and the case list
    # from the previous run instead of rebuilding them. None of the three depends on
    # the shard (only the -D flags below do), and a TAKEOVER CHECK needs them frozen:
    # all three are derived from the live build, so once the first shard's media build
    # has dropped a deleted family's objects, rebuilding them would remove Microsoft's
    # bodies from ref_ AND (the burn-down now calling those exports ours) drop the
    # exports from the case list -- every later shard grading less, with no error.
    if [ "${UCRTDIFF_REUSE_INPUTS:-0}" = "1" ]; then
        for f in "$REF" "$RUST" "$OURS" "$HERE/generated.inc" "$HERE/generated-refs.h"; do
            [ -s "$f" ] || { echo "[ucrtdiff] FAIL($A): UCRTDIFF_REUSE_INPUTS=1 but $f is missing"; exit 1; }
        done
        echo "[ucrtdiff] $A: REUSING the reference set, rust set and case list (frozen)"
    else
    echo "[ucrtdiff] building the $A reference set"
    python3 "$ROOT/tools/ucrt_tier2_refset.py" --arch "$A"
    [ -s "$REF" ] || { echo "[ucrtdiff] FAIL($A): $REF missing"; exit 1; }

    # The THIRD side: our existing msvcrt Rust bodies, renamed rust_*. This is what
    # makes the harness an oracle rather than a control -- while a family is still
    # answered by the vendored C, ref_ and the DLL are the SAME code and agreeing
    # proves nothing about us.
    echo "[ucrtdiff] building the $A msvcrt-rust set"
    python3 "$ROOT/tools/ucrt_tier2_refset.py" --arch "$A" --side msvcrt-rust
    [ -s "$RUST" ] || { echo "[ucrtdiff] FAIL($A): $RUST missing"; exit 1; }

    # The internals a takeover moved to Rust that the REMAINING Microsoft objects still
    # call (the locale hooks, __acrt_CompareStringA...). Only those keep their names;
    # see build_internals_set.
    echo "[ucrtdiff] building the $A reference-internals set"
    python3 "$ROOT/tools/ucrt_tier2_refset.py" --arch "$A" --side ucrt-internals
    [ -s "$OURS" ] || { echo "[ucrtdiff] FAIL($A): $OURS missing"; exit 1; }
    fi
    # ref_X for a function export a takeover removed from the reference set: an alias of
    # our Rust copy in the reference instance (see build_internals_set). The file may be
    # empty, but it must exist -- a missing one means the internals set is stale.
    [ -f "$B/libucrt_ours.refalias" ] || { echo "[ucrtdiff] FAIL($A): $B/libucrt_ours.refalias missing -- rebuild without UCRTDIFF_REUSE_INPUTS"; exit 1; }
    REFALIAS=()
    while IFS= read -r a; do [ -n "$a" ] && REFALIAS+=("-Wl,-Xlink=-alternatename:$a"); done < "$B/libucrt_ours.refalias"

    # The ucrtbase IMPORT LIB is what makes the bare side resolve to our shipping
    # DLL. It is a normal build target, but only gets built when something links
    # it -- on arm64 nothing does (ucrtbase_winetest is an i386/amd64 rostest),
    # so it is routinely absent there. Build that one target rather than failing:
    # it is an import lib generated from the .def and costs seconds.
    IMPLIB="$B/dll/win32/ucrtbase/CMakeFiles/libucrtbase.dir/libucrtbase.a"
    if [ ! -s "$IMPLIB" ]; then
        echo "[ucrtdiff] $A: libucrtbase import lib absent, building it"
        # Build the OUTPUT PATH, not a target name: `libucrtbase` exists as a named
        # target only where something links it (i386/amd64, via ucrtbase_winetest).
        # The path is a target on every arch.
        (cd "$ROOT" && bash tools/canon-run.sh ninja -C "build-$A" \
            "dll/win32/ucrtbase/CMakeFiles/libucrtbase.dir/libucrtbase.a") \
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
    # UCRTDIFF_REVERSE=1 emits the exports back-to-front. Diagnostic only, and it has
    # already done its job: FORWARD the old sweep stopped after 114 exports, REVERSED
    # after 53, so the wall was neither a fixed export count nor one bad body. Counting
    # the corpus products for those two prefixes then killed the last cheap theory --
    # ~5,263 cases vs ~2,182, so not a fixed case count either. Sharding makes the
    # remaining question moot rather than answering it; keep this for when it matters.
    GENFLAGS=""; [ "${UCRTDIFF_REVERSE:-0}" = "1" ] && GENFLAGS="--reverse"
    if [ "${UCRTDIFF_REUSE_INPUTS:-0}" != "1" ]; then
        echo "[ucrtdiff] generating $A cases"
        python3 "$ROOT/tools/gen_ucrt_tier2_cases.py" --arch "$A" --out "$HERE/generated.inc" $GENFLAGS
    fi

    echo "[ucrtdiff] linking ucrtdiff-$A.exe"
    # -fno-builtin is LOAD-BEARING, not hygiene: clang knows what isdigit, strlen
    # and friends mean and will fold or inline them, so the "ours" side silently
    # stops being a call into ucrtbase.dll and the row grades the COMPILER against
    # Microsoft. isdigit vanished from the import table exactly this way.
    "$TC/${CC[$A]}" -O2 -fno-builtin -nostdlib -o "$HERE/ucrtdiff-$A.exe" "$HERE/ucrtdiff.c" "$HERE/stubs.c" \
        -DUCRTDIFF_SHARD_K="$SHARD_K" -DUCRTDIFF_SHARD_N="$SHARD_N" \
        -DUCRTDIFF_BUDGET_MS="$BUDGET_MS" -DUCRTDIFF_ONLY="\"${UCRTDIFF_ONLY:-}\"" \
        "${REFALIAS[@]}" \
        -Wl,--start-group \
        "$REF" \
        "$RUST" \
        "$OURS" \
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
        -Wl,--subsystem,windows -Wl,-e,ucrtdiff_entry

    # Three checks, read straight from the PE by tools/payloads/pecheck.py (the old
    # `objdump -p | grep -q` under pipefail passed or failed at random on SIGPIPE):
    #  * GUI subsystem: a console exe launched from explorer's startup walk gets no
    #    CSRSS console attach and dies without a word on serial (kmtest's lesson).
    #  * ucrtbase.dll imported directly, no api-ms-win-crt-* sets: otherwise the bare
    #    side stops being OUR DLL and the harness grades Microsoft's C against itself.
    #  * EVERY graded function imported from ucrtbase.dll. A row whose "ours" side got
    #    inlined or constant-folded still PASSES -- it compares Microsoft's body against
    #    the compiler's idea instead of our DLL. This caught isdigit before -fno-builtin.
    #    A reference a takeover removed (generated-refs.h says 0) is compiled out.
    python3 "$(dirname "$HERE")/pecheck.py" "$HERE/ucrtdiff-$A.exe" --tag ucrtdiff \
        --gui --require-dll ucrtbase.dll --forbid-dll api-ms-win-crt \
        --graded-from ucrtbase.dll --graded-decls "$HERE/ucrtdiff.c" \
        --refs-header "$HERE/generated-refs.h"
    echo "[ucrtdiff] ok($A): $(stat -c%s "$HERE/ucrtdiff-$A.exe") bytes, 0 ungraded"
done
