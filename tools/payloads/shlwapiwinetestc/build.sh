#!/usr/bin/env bash
# tools/payloads/shlwapiwinetestc/build.sh — the C half of shlwapi's before/after.
# Usage: tools/payloads/shlwapiwinetestc/build.sh <arm64|amd64|i386>
#
# Builds shlwapi_c.dll (the pre-flip C, `python -m rustconv.c_oracle dll/win32/shlwapi`)
# and a copy of shlwapi_winetest.exe whose "shlwapi.dll" strings (the import and any
# GetModuleHandle/LoadLibrary lookups) read "shlwapc.dll", the same length. Renaming is the
# only way in: shlwapi is a KnownDLL (boot/bootdata/hivesys.inf), so a shlwapi.dll beside
# the exe is ignored. DLLs the suite loads that import shlwapi themselves still bind the
# Rust one; the exe's own calls reach the C.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
A="${1:?usage: build.sh <arch>}"
EXE="$ROOT/build-$A/modules/rostests/winetests/shlwapi/shlwapi_winetest.exe"
[ -f "$EXE" ] || { echo "[shlwapiwinetestc] $EXE missing; fix: ./vibeos-cli build --arch $A --winetests shlwapi" >&2; exit 1; }

(cd "$ROOT/build-$A" && bash ../tools/canon-run.sh ninja shlwapi_c)
cp "$ROOT/build-$A/dll/win32/shlwapi/shlwapi_c.dll" "$HERE/shlwapi_c-$A.dll"
python3 - "$EXE" "$HERE/shlwapi_winetest_c-$A.exe" <<'PY'
import re, sys
data = open(sys.argv[1], "rb").read()
hits = len(re.findall(rb"(?i)shlwapi\.dll", data))
if hits == 0:
    sys.exit("[shlwapiwinetestc] no shlwapi.dll string to rename; the oracle would test the Rust")
open(sys.argv[2], "wb").write(re.sub(rb"(?i)shlwapi\.dll", b"shlwapc.dll", data))
print(f"[shlwapiwinetestc] renamed {hits} shlwapi.dll string(s)")
PY
