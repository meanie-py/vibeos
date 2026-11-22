#!/usr/bin/env bash
# tools/payloads/k32winetest/build.sh — regenerate the k32winetest sweep lists.
# Usage: tools/payloads/k32winetest/build.sh [arm64|amd64|i386|all]   # default: all
# Outputs: winetest-suites-<arch>.txt and winetest-skip-<arch>.txt (gitignored), from
# olewinetest's gen_suites.py pointed at kernel32.
#
# The payload builder runs this when those lists are stale against their inputs (after a
# master merge touches the generator, for instance). Without it the build stopped at
# "stale or missing output and no .../build.sh" and the fix was a remembered command.
#
# The --only subset is the caller's choice, so it is KEPT: read back from the existing
# list's "RESTRICTED by --only to:" header. K32WINETEST_ONLY overrides it ("" for every
# subtest). With neither, payload.yaml's build_hint default applies: kernel32:version.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"

ARCHES="${1:-all}"
[ "$ARCHES" = "all" ] && ARCHES="arm64 amd64 i386"

for A in $ARCHES; do
    if [ "${K32WINETEST_ONLY+set}" = set ]; then
        ONLY="$K32WINETEST_ONLY"
    elif [ -f "$HERE/winetest-suites-$A.txt" ]; then
        ONLY="$(sed -n 's/^# RESTRICTED by --only to: //p' "$HERE/winetest-suites-$A.txt" | head -n 1)"
    else
        ONLY="kernel32:version"
    fi
    echo "[k32winetest] $A: regenerating sweep lists${ONLY:+ (--only $ONLY)}"
    python3 "$ROOT/tools/payloads/olewinetest/gen_suites.py" --arch "$A" --suites kernel32 \
        ${ONLY:+--only "$ONLY"} --register '' --out-dir "$HERE"
done
echo "[k32winetest] OK"
