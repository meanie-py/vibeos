#!/bin/bash
# soak.sh — boot x86 Doom under WoW64/FEX on arm64 and assert it renders without
# the M14 residual startup death ([UEXC-KILL]) or a BUGCHECK. Doubles as
#   (a) the FEX/DOOM regression check for automation (default: 1 sequential boot),
#   (b) the contention soak for hunting the startup flake (--concurrent K [--rounds R]).
#
# The residual flake is a FEX JIT `blr` to a null host block (PC=0) at doom
# startup, seen only under multi-VM host contention (field manual §9.85 cousin;
# wow64-plan.md M14 residual). The kernel [UEXC-KILL] path now dumps the guest
# EIP/ESP/return address (GuestEip=/GuestRet=), so any hit here is root-cause-grade.
#
# Usage:
#   tools/doom/soak.sh [N]                 # N sequential boots (default 1)
#   tools/doom/soak.sh --concurrent K      # K boots at once (faithful contention)
#            [--rounds R]                   #   repeat the concurrent round R times
#            [--stress]                     # also pin host cores with a CPU fan
#            [--keep]                       # keep passing-boot logs too
#
# Exit 0 iff every boot rendered cleanly; 1 on any [UEXC-KILL]/BUGCHECK or
# render-fail. Fault logs land in tools/doom/soak-fails/.
set -u
cd "$(dirname "$0")/../.." || exit 1
export RUNS_DIR=${RUNS_DIR:-/tmp/vbr}

N=1; CONC=1; ROUNDS=1; STRESS=0; KEEP=0
while [ $# -gt 0 ]; do
    case "$1" in
        --concurrent) CONC=$2; shift 2 ;;
        --rounds)     ROUNDS=$2; shift 2 ;;
        --stress)     STRESS=1; shift ;;
        --keep)       KEEP=1; shift ;;
        [0-9]*)       N=$1; shift ;;
        *) echo "unknown arg: $1" >&2; exit 2 ;;
    esac
done

FAILDIR="tools/doom/soak-fails"; mkdir -p "$FAILDIR"
MEDIA="/tmp/soak-media"
PASS=0; FAULT=0; RENDER_FAIL=0

# --- stage the FEX ramdisk once (x86 doom.exe in syswow64) ------------------
echo "[soak] staging FEX ramdisk (VIBEOS_FEX=1)"
rm -f qemu/ramdisk.img qemu/boot.img
if ! VIBEOS_FEX=1 bash qemu/mkramdisk.sh >/tmp/soak-mkramdisk.log 2>&1; then
    echo "[soak] mkramdisk FAILED"; tail -8 /tmp/soak-mkramdisk.log; exit 1
fi

# --- optional CPU contention fan --------------------------------------------
FAN_PIDS=()
start_fan() {
    [ "$STRESS" = 1 ] || return 0
    local cores; cores=$(( $(nproc) - 6 )); [ "$cores" -lt 1 ] && cores=1
    echo "[soak] arming CPU fan on $cores cores"
    for _ in $(seq 1 "$cores"); do ( while :; do :; done ) & FAN_PIDS+=($!); done
}
stop_fan() { [ "${#FAN_PIDS[@]}" -gt 0 ] && kill "${FAN_PIDS[@]}" 2>/dev/null; FAN_PIDS=(); }
trap 'stop_fan; rm -rf "$MEDIA"' EXIT
start_fan

# classify one finished run's serial log; updates PASS/FAULT/RENDER_FAIL
classify() {
    local run="$1" log="$RUNS_DIR/$1/serial.log" kills blits
    kills=$(grep -cE 'UEXC-KILL|BUGCHECK|UEXC-STKOVERFLOW' "$log" 2>/dev/null); kills=${kills:-0}
    blits=$(grep -cE 'NtGdiStretchDIBitsInternal.*enter' "$log" 2>/dev/null); blits=${blits:-0}
    if [ "$kills" -gt 0 ]; then
        echo "[soak]   $run: *** FAULT *** ($kills markers)"
        grep -E 'UEXC-KILL|BUGCHECK|GuestEip|GuestEsp|GuestEax|GuestRet|Proc=' "$log" 2>/dev/null | tail -20
        cp "$log" "$FAILDIR/fault-$run.log" 2>/dev/null; FAULT=$((FAULT+1))
    elif [ "$blits" -lt 20 ]; then
        echo "[soak]   $run: render-fail ($blits blits — reaped/hung, not a crash)"
        cp "$log" "$FAILDIR/render-$run.log" 2>/dev/null; RENDER_FAIL=$((RENDER_FAIL+1))
    else
        echo "[soak]   $run: PASS ($blits blits)"
        [ "$KEEP" = 1 ] && cp "$log" "$FAILDIR/pass-$run.log" 2>/dev/null
        PASS=$((PASS+1))
    fi
}

if [ "$CONC" -gt 1 ]; then
    # ---- concurrent mode: K private-media boots at once, R rounds -----------
    for r in $(seq 1 "$ROUNDS"); do
        echo "[soak] === concurrent round $r/$ROUNDS ($CONC VMs) ==="
        rm -rf "$MEDIA"; mkdir -p "$MEDIA"
        RUNS=()
        for k in $(seq 1 "$CONC"); do
            cp qemu/boot.img "$MEDIA/boot-$k.img"
            cp qemu/efivars.fd "$MEDIA/efivars-$k.fd"
            run=$(VIBEOS_QEMU_MAX_LIFETIME=360 \
                  VIBEOS_BOOT_IMG="$MEDIA/boot-$k.img" \
                  VIBEOS_EFIVARS="$MEDIA/efivars-$k.fd" \
                  bash qemu/start.sh arm64 2>/dev/null)
            [ -n "$run" ] && { RUNS+=("$run"); echo "[soak]   launched $run (vm $k)"; }
        done
        # All K now contend. Wait for each to reach doom + render (or time out).
        for run in "${RUNS[@]}"; do
            timeout 260 bash qemu/wait-for.sh "$run" --pattern "NtGdiStretchDIBitsInternal.*enter" --timeout 250 >/dev/null 2>&1
        done
        sleep 6
        for run in "${RUNS[@]}"; do classify "$run"; done
        for run in "${RUNS[@]}"; do bash qemu/stop.sh "$run" >/dev/null 2>&1; done
    done
else
    # ---- sequential mode ----------------------------------------------------
    for i in $(seq 1 "$N"); do
        echo "[soak] === boot $i/$N ==="
        run=$(VIBEOS_QEMU_MAX_LIFETIME=300 bash qemu/start.sh arm64 2>/dev/null)
        [ -n "$run" ] || { echo "[soak] start.sh gave no run id"; FAULT=$((FAULT+1)); continue; }
        timeout 200 bash qemu/wait-for.sh "$run" --pattern "x86 DOOM under WoW64 created" --timeout 190 >/dev/null 2>&1
        timeout 90  bash qemu/wait-for.sh "$run" --pattern "NtGdiStretchDIBitsInternal.*enter" --timeout 80 >/dev/null 2>&1
        sleep 4
        classify "$run"
        bash qemu/stop.sh "$run" >/dev/null 2>&1
    done
fi

stop_fan; rm -rf "$MEDIA"
TOTAL=$((PASS+FAULT+RENDER_FAIL))
echo "[soak] ==== DONE: $PASS pass / $FAULT fault / $RENDER_FAIL render-fail (of $TOTAL) ===="
[ "$FAULT" -eq 0 ] || exit 1
exit 0
