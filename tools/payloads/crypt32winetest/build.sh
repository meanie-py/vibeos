#!/usr/bin/env bash
# tools/payloads/crypt32winetest/build.sh — the suite and skip lists for Wine's crypt32 tests.
# Usage: tools/payloads/crypt32winetest/build.sh <arm64|amd64|i386>
#
# The payload compiler runs this when winetest-{suites,skip}-<arch>.txt are missing or older
# than this directory's sources. The lists are olewinetest's generator pointed at crypt32; it
# reads the suite names out of crypt32_winetest.exe, so this builds that exe first: the
# compiler runs payloads before the main ninja pass, so on a fresh arch it does not exist yet.
# (apitest-sweep-<arch>.exe is apitest's payload output; the compiler builds it as a borrow.)
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
A="${1:?usage: build.sh <arch>}"
EXE="$ROOT/build-$A/modules/rostests/winetests/crypt32/crypt32_winetest.exe"
(cd "$ROOT/build-$A" && bash ../tools/canon-run.sh ninja crypt32_winetest) >&2 \
  || { echo "[crypt32winetest] ninja crypt32_winetest failed; fix: ./vibeos-cli build --arch $A --winetests crypt32" >&2; exit 1; }
[ -f "$EXE" ] || { echo "[crypt32winetest] $EXE still missing after ninja" >&2; exit 1; }

cd "$ROOT"
python3 tools/payloads/olewinetest/gen_suites.py --arch "$A" --suites crypt32 --register '' --out-dir "$HERE"
