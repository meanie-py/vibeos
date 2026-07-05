#!/bin/bash
# tools/coverage/run.sh — one-shot boot coverage map for an arch.
#
#   tools/coverage/run.sh <i386|amd64> [--timeout <s>]
#
# Composes the existing kernel-debug toolkit end to end:
#   1. builds the TCG plugin if missing (build.sh),
#   2. boots the arch's real UEFI livecd via qemu/start.sh with
#      VIBEOS_NO_KVM=1 (plugins need TCG) and the plugin injected through
#      VIBEOS_QEMU_EXTRA,
#   3. waits for the North-Star boot marker ("shell ready") via wait-for.sh
#      (TCG is slow: default timeout 900s),
#   4. stops the run cleanly via stop.sh (SIGTERM → the plugin's atexit
#      flushes cov.txt),
#   5. symbolizes into the per-function coverage report + the never-executed
#      Rust function list (report_coverage.py).
#
# Output lands in the run dir (printed at the end). arm64 is TODO: the
# ramdisk boot works under TCG already, but plugin availability for
# qemu-system-aarch64 + the marker timeout need their own validation pass.
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/../.." && pwd)"

ARCH="${1:-}"
case "$ARCH" in
    i386|amd64) ;;
    arm64) echo "arm64 coverage not validated yet — see header" >&2; exit 2 ;;
    *) echo "usage: $0 <i386|amd64> [--timeout <s>]" >&2; exit 2 ;;
esac
shift
TIMEOUT=900
if [ "${1:-}" = "--timeout" ]; then
    TIMEOUT="${2:?--timeout needs seconds}"
fi

PLUGIN="$SCRIPT_DIR/vibeos_cov.so"
[ -f "$PLUGIN" ] || bash "$SCRIPT_DIR/build.sh"

# Kernel-range filter keeps the set small; user/firmware space is ambiguous
# across address spaces anyway (the plugin sees vaddrs only, no cr3).
case "$ARCH" in
    amd64) FILT="min=0xfffff80000000000" ;;
    i386)  FILT="min=0x80000000" ;;
esac

# start.sh prints the run id on stdout; the run dir holds serial.log etc.
COV_TMP="$(mktemp -t vibeos-cov-XXXX.txt)"
RUN="$(cd "$REPO" && VIBEOS_NO_KVM=1 \
    VIBEOS_QEMU_EXTRA="-plugin $PLUGIN,out=$COV_TMP,$FILT" \
    bash qemu/start.sh "$ARCH")"
echo "[coverage] run=$RUN cov=$COV_TMP (TCG boot, timeout ${TIMEOUT}s)" >&2

MARKER_OK=1
"$REPO/qemu/wait-for.sh" "$RUN" --pattern "shell ready" --timeout "$TIMEOUT" || MARKER_OK=0
# Post-marker settle (default 45s): service startup — where gate/WS contention
# parks threads (KeWaitForGate etc.) — runs AFTER "shell ready". Stopping at
# the marker would leave that entire window out of the map (verified: a
# marker-stop boot records ZERO TBs in KeWaitForGate). Matches the smoke
# verdict's settle-scan semantics.
[ "$MARKER_OK" -eq 1 ] && sleep "${VIBEOS_COV_SETTLE:-45}"
"$REPO/qemu/stop.sh" "$RUN" || true
sleep 1  # let the atexit flush land

if [ ! -s "$COV_TMP" ]; then
    echo "[coverage] FAIL: no coverage data written ($COV_TMP empty) — plugin" \
         "did not run or QEMU died pre-translation. Check the run dir serial." >&2
    exit 1
fi
if [ "$MARKER_OK" -eq 0 ]; then
    echo "[coverage] WARNING: boot marker not reached in ${TIMEOUT}s — the map" \
         "below covers a PARTIAL boot only. Do not read it as the smoke map." >&2
fi

RUN_DIR="$REPO/qemu/runs/$RUN"
[ -d "$RUN_DIR" ] || RUN_DIR="$(dirname "$COV_TMP")"
cp "$COV_TMP" "$RUN_DIR/cov.txt" 2>/dev/null || true

PY="$REPO/.venv/bin/python"
[ -x "$PY" ] || PY="$(ls -d "$REPO"/../*/.venv/bin/python 2>/dev/null | head -1)"
[ -x "$PY" ] || PY=python3
"$PY" "$SCRIPT_DIR/report_coverage.py" --arch "$ARCH" --cov "$COV_TMP" --out-dir "$RUN_DIR"
echo "[coverage] artifacts in $RUN_DIR" >&2
