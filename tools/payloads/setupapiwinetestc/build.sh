#!/usr/bin/env bash
# tools/payloads/setupapiwinetestc/build.sh — the C half of setupapi's before/after.
# Usage: tools/payloads/setupapiwinetestc/build.sh <arm64|amd64|i386>
#
# Builds setupapi_c.dll (the pre-flip C, `python -m rustconv.c_oracle dll/win32/setupapi`)
# and a copy of setupapi_winetest.exe whose four "setupapi.dll" strings (the import and the
# GetModuleHandleA lookups) read "setupapc.dll", the same length. Renaming is the only
# way in: setupapi is a KnownDLL (boot/bootdata/hivesys.inf), so a setupapi.dll beside the
# exe is ignored, and an ISO graft over system32's copy loses to the build's own entry.
# The exe imports no cfgmgr32 (whose CM_* forward to setupapi), so every call reaches the C.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
A="${1:?usage: build.sh <arch>}"
EXE="$ROOT/build-$A/modules/rostests/winetests/setupapi/setupapi_winetest.exe"
[ -f "$EXE" ] || { echo "[setupapiwinetestc] $EXE missing; fix: ./vibeos-cli build --arch $A --rostests --keep-going" >&2; exit 1; }

(cd "$ROOT/build-$A" && bash ../tools/canon-run.sh ninja setupapi_c)
cp "$ROOT/build-$A/dll/win32/setupapi/setupapi_c.dll" "$HERE/setupapi_c-$A.dll"
python3 - "$EXE" "$HERE/setupapi_winetest_c-$A.exe" <<'PY'
import re, sys
data = open(sys.argv[1], "rb").read()
hits = len(re.findall(rb"(?i)setupapi\.dll", data))
if hits == 0:
    sys.exit("[setupapiwinetestc] no setupapi.dll string to rename; the oracle would test the Rust")
open(sys.argv[2], "wb").write(re.sub(rb"(?i)setupapi\.dll", b"setupapc.dll", data))
print(f"[setupapiwinetestc] renamed {hits} setupapi.dll string(s)")
PY
