#!/usr/bin/env bash
# Catch a NON-DETERMINISTIC BUGCHECK live and symbolicate it with the runtime index.
#
# Why this exists: rare post-shell-ready crashes (e.g. the amd64 taskbar 0x50) don't
# reproduce under TCG (too slow + timing change suppresses the race) and KVM doesn't
# fire gdb breakpoints — so run-until/triage can't catch them. This loops FAST KVM boots
# until a BUGCHECK appears in the serial log, then connects gdb (interrupt works under
# KVM even though breakpoints don't), reads the bugcheck params, and resolves the faulting
# PC + paged-in faulting module via qemu/runtime_symbolicate.py (which the crash makes
# resident). Reusable for any non-det kernel crash, not just this one.
#
# Usage: qemu/catch_bugcheck.sh <arch> [max_boots] [shellready_timeout] [crash_window]
set -uo pipefail
cd "$(dirname "$0")/.."
source qemu/lib/runid.sh 2>/dev/null || true

ARCH="${1:-amd64}"
MAX="${2:-10}"
SR_TIMEOUT="${3:-300}"
CRASH_WIN="${4:-75}"
INV="/tmp/pe_inv_${ARCH}.tsv"
NM=".build-tools/llvm-mingw/bin/llvm-nm"

[ -f "$INV" ] || { echo "[catch] building PE inventory -> $INV"; qemu/pe_inventory.py "build-${ARCH}" > "$INV"; }

# Some races only reproduce under smoke's device config (virtio-blk scratch +
# debugcon shift PnP/driver-load timing). VIBEOS_CATCH_SCRATCH=1 matches it.
SCRATCH_DIR="/tmp/vibeos-catch-$$"; mkdir -p "$SCRATCH_DIR"
cleanup_scratch() { rm -rf "$SCRATCH_DIR" 2>/dev/null; }
trap cleanup_scratch EXIT

caught_run=""
for i in $(seq 1 "$MAX"); do
    echo "=== [catch] boot attempt $i/$MAX ($ARCH, KVM+gdb${VIBEOS_CATCH_SCRATCH:+, smoke-config}) ==="
    export VIBEOS_QEMU_EXTRA=""
    if [ -n "${VIBEOS_CATCH_SCRATCH:-}" ]; then
        SCR="$SCRATCH_DIR/scratch_$i.qcow2"
        qemu-img create -f qcow2 "$SCR" 64M >/dev/null 2>&1
        export VIBEOS_QEMU_EXTRA="-drive file=$SCR,format=qcow2,if=none,id=scratch -device virtio-blk-pci,drive=scratch -debugcon file:$SCRATCH_DIR/dbg_$i.bin"
    fi
    RUN=$(bash qemu/start.sh "$ARCH" --gdb 2>/dev/null | tail -1)
    [ -z "$RUN" ] && { echo "[catch] start.sh failed"; continue; }
    echo "[catch]   run=$RUN"
    SERIAL="$(runid_serial_log "$RUN" 2>/dev/null)"

    # Wait for boot to complete (shell ready) OR an early bugcheck.
    bash qemu/wait-for.sh "$RUN" --pattern "shell ready" --timeout "$SR_TIMEOUT" >/dev/null 2>&1
    # Now watch a window for the post-shell-ready crash.
    if bash qemu/wait-for.sh "$RUN" --pattern "BUGCHECK 0x" --timeout "$CRASH_WIN" >/dev/null 2>&1; then
        echo "[catch]   *** BUGCHECK CAUGHT on attempt $i"
        caught_run="$RUN"
        break
    fi
    # also catch a bugcheck that landed before shell ready
    if grep -aq "BUGCHECK 0x" "$SERIAL" 2>/dev/null; then
        echo "[catch]   *** BUGCHECK (pre-shell-ready) on attempt $i"
        caught_run="$RUN"
        break
    fi
    echo "[catch]   clean boot, stopping & retrying"
    bash qemu/stop.sh "$RUN" >/dev/null 2>&1
done

if [ -z "$caught_run" ]; then
    echo "[catch] no BUGCHECK in $MAX boots. Very rare races (the amd64 taskbar 0x50"
    echo "[catch] was seen <1/86) may need a different trigger or a passive trap in"
    echo "[catch] smoke rather than brute-force boots."
    exit 2
fi

RUN="$caught_run"
SERIAL="$(runid_serial_log "$RUN")"
BCLINE=$(grep -aoE "BUGCHECK: 0x[0-9A-Fa-f]+ \([^)]*\)" "$SERIAL" | tail -1)
echo "[catch] $BCLINE"
# Extract params (P1 referenced addr, P3 faulting PC for 0x50)
PARAMS=$(echo "$BCLINE" | grep -oE "0x[0-9A-Fa-f]+" )
P1=$(echo "$PARAMS" | sed -n '2p')
P3=$(echo "$PARAMS" | sed -n '4p')
echo "[catch] referenced=$P1  faultingPC=$P3"

echo "=== [catch] symbolicating (faulting module is resident now) ==="
TARGETS="$P3"
[ -n "$P1" ] && TARGETS="$TARGETS,$P1"
SYMIDX_INV="$INV" SYMIDX_BUILD="build-${ARCH}" SYMIDX_NM="$NM" SYMIDX_DUMP=1 SYMIDX_TARGETS="$TARGETS" \
    bash qemu/inspect.sh "$RUN" --command raw "source qemu/runtime_symbolicate.py" 2>&1 \
    | grep -E "\[idx\]|\[mod\]|!|covers|NOT in" | grep -vE "DWO|warning"

echo "=== [catch] registers at halt ==="
bash qemu/inspect.sh "$RUN" --command regs 2>&1 | grep -ivE "DWO|warning" | head -20
echo "[catch] run $RUN left alive for further inspection (stop: bash qemu/stop.sh $RUN)"
