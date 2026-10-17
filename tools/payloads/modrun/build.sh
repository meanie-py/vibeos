#!/usr/bin/env bash
# tools/payloads/modrun/build.sh — the modrun runner, its case list, and the build-dir
# targets the suites stage (C oracles, probes).
# Usage: tools/payloads/modrun/build.sh {i386|amd64|arm64}
# Outputs: modrun-<arch>.exe, modrun-<arch>.txt (gitignored), and `ninja <targets>` in build-<arch>.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
TC="$ROOT/.build-tools/llvm-mingw/bin"
A="${1:?arch}"
declare -A CC=( [arm64]=aarch64-w64-mingw32-gcc [amd64]=x86_64-w64-mingw32-gcc [i386]=i686-w64-mingw32-gcc )
# The case list first: it is a staged output, and the stale check reads any file here newer
# than the exe as a source edit.
python3 "$HERE/modrun.py" gen --arch "$A"
# GUI subsystem + classic msvcrt: the apitest sweep's recipe, for the same reasons.
"$TC/${CC[$A]}" -O2 -o "$HERE/modrun-$A.exe" "$HERE/modrun.c" \
    -lmsvcrt-os -lntdll -ladvapi32 -Wl,--subsystem,windows -Wl,-e,mainCRTStartup
python3 "$(dirname "$HERE")/pecheck.py" "$HERE/modrun-$A.exe" --tag modrun --gui --forbid-dll api-ms-win-crt
# Plain-C probes (probes/<name>.c -> probes/<name>-<arch>.exe), for suites that test a module
# through its public API and need nothing from the build tree.
for SRC in "$HERE"/probes/*.c; do
    [ -e "$SRC" ] || continue
    OUT="${SRC%.c}-$A.exe"
    "$TC/${CC[$A]}" -O1 -o "$OUT" "$SRC" -lmsvcrt-os -lnetapi32 -lshell32 -lshlwapi -lole32 -luuid -ladvapi32 -luser32 -lgdi32 -lkernel32 \
        -Wl,--subsystem,console
    python3 "$(dirname "$HERE")/pecheck.py" "$OUT" --tag modrun-probe --forbid-dll api-ms-win-crt
done
TARGETS="$(python3 "$HERE/modrun.py" ninja --arch "$A")"
if [ -n "$TARGETS" ]; then
    # The oracles and probes are EXCLUDE_FROM_ALL: nothing else builds them. Incremental, so
    # this is a no-op when neither the C nor the Rust they link changed.
    if [ ! -f "$ROOT/build-$A/build.ninja" ]; then
        echo "modrun: build-$A is not configured; run ./vibeos-cli build --arch $A first" >&2
        exit 1
    fi
    (cd "$ROOT/build-$A" && bash ../tools/canon-run.sh ninja $TARGETS)
fi
echo "[modrun] OK"
