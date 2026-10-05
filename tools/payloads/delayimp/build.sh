#!/usr/bin/env bash
# tools/payloads/delayimp/build.sh — the three sdk delay-import apitests (nohook, globalhook,
# runtimehook) against the delay-load runtime, which is Rust (sdk/lib/rust/delayimp-rust).
# Usage: tools/payloads/delayimp/build.sh [i386]   Output: delayimp_*_apitest.exe + sweep
# i386 only: the rostests corpus builds there (same constraint as ../ucrtwine).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
A="${1:-i386}"
[ "$A" = "i386" ] || { echo "[delayimp] FAIL: the rostests corpus builds on i386 only; got $A"; exit 1; }
B="$ROOT/build-$A"
[ -d "$B" ] || { echo "[delayimp] FAIL: no $B -- ./vibeos-cli build --arch $A first"; exit 1; }
T="delayimp_nohook_apitest delayimp_globalhook_apitest delayimp_runtimehook_apitest"
( cd "$B" && bash ../tools/canon-run.sh cmake -DENABLE_ROSTESTS=ON . >/dev/null \
    && bash ../tools/canon-run.sh ninja $T )
for t in $T; do cp "$B/modules/rostests/apitests/sdk/$t.exe" "$HERE/"; done
bash "$ROOT/tools/payloads/apitest/build.sh" "$A"
cp "$ROOT/tools/payloads/apitest/apitest-sweep-$A.exe" "$HERE/sweep-$A.exe"
echo "[delayimp] OK (the next plain build turns ENABLE_ROSTESTS back off; the copies here stay)"
