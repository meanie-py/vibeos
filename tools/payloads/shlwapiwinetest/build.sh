#!/usr/bin/env bash
# tools/payloads/shlwapiwinetest/build.sh — the suite and skip lists for Wine's shlwapi tests.
# Usage: tools/payloads/shlwapiwinetest/build.sh <arm64|amd64|i386>
#
# The payload compiler runs this when winetest-{suites,skip}-<arch>.txt are missing or older
# than this directory's sources. The lists are olewinetest's generator pointed at shlwapi; it
# reads the suite names out of shlwapi_winetest.exe, so this builds that exe first: the
# compiler runs payloads before the main ninja pass, so on a fresh arch it does not exist yet.
# (apitest-sweep-<arch>.exe is apitest's payload output; the compiler builds it as a borrow.)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
A="${1:?usage: build.sh <arch>}"
EXE="$ROOT/build-$A/modules/rostests/winetests/shlwapi/shlwapi_winetest.exe"
(cd "$ROOT/build-$A" && bash ../tools/canon-run.sh ninja shlwapi_winetest) >&2 \
  || { echo "[shlwapiwinetest] ninja shlwapi_winetest failed; fix: ./vibeos-cli build --arch $A --winetests shlwapi" >&2; exit 1; }
[ -f "$EXE" ] || { echo "[shlwapiwinetest] $EXE still missing after ninja" >&2; exit 1; }

cd "$ROOT"
python3 tools/payloads/olewinetest/gen_suites.py --arch "$A" --suites shlwapi --register '' --out-dir "$HERE"
