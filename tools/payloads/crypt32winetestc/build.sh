#!/usr/bin/env bash
# tools/payloads/crypt32winetestc/build.sh — the C half of crypt32's before/after.
# Usage: tools/payloads/crypt32winetestc/build.sh <arm64|amd64|i386>
#
# Builds crypt32_c.dll (the pre-flip C, `python -m rustconv.c_oracle dll/win32/crypt32`)
# and a copy of crypt32_winetest.exe whose "crypt32.dll" strings (the import and any
# GetModuleHandle/LoadLibrary lookups) read "crypt3c.dll", the same length. Renaming is the
# only way in: crypt32 is a KnownDLL (boot/bootdata/hivesys.inf), so a crypt32.dll beside
# the exe is ignored. DLLs the suite loads that import crypt32 themselves still bind the
# Rust one; the exe's own calls reach the C.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
A="${1:?usage: build.sh <arch>}"
EXE="$ROOT/build-$A/modules/rostests/winetests/crypt32/crypt32_winetest.exe"
# Both are built here: the payload compiler runs before the main ninja pass.
(cd "$ROOT/build-$A" && bash ../tools/canon-run.sh ninja crypt32_winetest crypt32_c) >&2 \
  || { echo "[crypt32winetestc] ninja failed; fix: ./vibeos-cli build --arch $A --winetests crypt32" >&2; exit 1; }
[ -f "$EXE" ] || { echo "[crypt32winetestc] $EXE still missing after ninja" >&2; exit 1; }
cp "$ROOT/build-$A/dll/win32/crypt32/crypt32_c.dll" "$HERE/crypt32_c-$A.dll"
python3 "$ROOT/tools/payloads/rename_module_refs.py" "$EXE" "$HERE/crypt32_winetest_c-$A.exe" crypt32 crypt3c
