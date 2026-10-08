#!/usr/bin/env bash
# tools/payloads/ucrtwine/build.sh — stage the ucrtbase conformance suites (i386).
# Usage: tools/payloads/ucrtwine/build.sh [i386]
# The two suites are rostests targets; this builds just them (not the ~128-exe
# --rostests corpus, whose media build can be blocked by any one unrelated
# suite) and copies them here beside the apitest sweep runner.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
A="${1:-i386}"
[ "$A" = "i386" ] || { echo "[ucrtwine] FAIL: the rostests corpus builds on i386 only; got $A"; exit 1; }
B="$ROOT/build-$A"
[ -d "$B" ] || { echo "[ucrtwine] FAIL: no $B -- ./vibeos-cli build --arch $A first"; exit 1; }
( cd "$B" && bash ../tools/canon-run.sh cmake -DENABLE_ROSTESTS=ON . >/dev/null \
    && bash ../tools/canon-run.sh ninja ucrtbase_winetest ucrtbase_apitest )
cp "$B/modules/rostests/winetests/ucrtbase/ucrtbase_winetest.exe" "$HERE/"
cp "$B/modules/rostests/apitests/ucrtbase/ucrtbase_apitest.exe" "$HERE/"
bash "$ROOT/tools/payloads/apitest/build.sh" "$A"
cp "$ROOT/tools/payloads/apitest/apitest-sweep-$A.exe" "$HERE/sweep-$A.exe"
echo "[ucrtwine] OK (the next plain build turns ENABLE_ROSTESTS back off; the copies here stay)"
