#!/bin/bash
# Show every tracked QEMU running RIGHT NOW, across ALL worktrees/sessions.
#
# Usage:
#   qemu/ps.sh            # compact human table + slot summary
#   qemu/ps.sh --json     # machine-readable array (for agents)
#
# Reads the GLOBAL lockfiles that both launchers drop in
# /tmp/vibeos-qemu-locks/ (tests/qemu/vm.py for smoke, qemu/start.sh for the
# debug toolkit) — so it sees runs from sibling worktrees too, which the
# per-worktree qemu/runs/ manifests can't. One-shot snapshot by design: it
# prints a handful of lines and exits, never streaming (to follow a single
# VM's boot live, tail its log — the WATCH column points at it).
#
# Columns:
#   ARCH  SRC   QPID    OWNER        AGE     TTL      SLOT  WATCH
#   - OWNER  = launcher pid; a trailing ! means that owning session is already
#              dead (a reaper will cull this VM on its next pass)
#   - AGE    = wall-clock the QEMU process has been alive (from /proc)
#   - TTL    = time left before the 30-min hard cap reaps it (OVERDUE if past)
#   - WATCH  = relative path / command to follow this VM's serial live

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"

JSON=0
for arg in "$@"; do
    case "$arg" in
        --json) JSON=1 ;;
        --help|-h) sed -n '2,20p' "$0"; exit 0 ;;
        *) die "unknown argument: $arg" ;;
    esac
done

LOCK_DIR="/tmp/vibeos-qemu-locks"
NOW="$(date +%s)"

pid_alive() { kill -0 "$1" 2>/dev/null; }
proc_is_qemu() { grep -q qemu "/proc/$1/comm" 2>/dev/null; }

# Seconds -> compact "1h03m" / "5m12s" / "47s".
human() {
    local s="$1" neg=""
    if [ "$s" -lt 0 ]; then neg="-"; s=$(( -s )); fi
    if   [ "$s" -ge 3600 ]; then printf '%s%dh%02dm' "$neg" $(( s/3600 )) $(( (s%3600)/60 ))
    elif [ "$s" -ge 60 ];   then printf '%s%dm%02ds' "$neg" $(( s/60 )) $(( s%60 ))
    else printf '%s%ds' "$neg" "$s"; fi
}

# --- gather rows ------------------------------------------------------------
rows_json="[]"
declare -a TABLE=()
running=0
stale=0

if [ -d "$LOCK_DIR" ]; then
    for lf in "$LOCK_DIR"/*.json; do
        [ -f "$lf" ] || continue
        qpid="$(jq -r '.qemu_pid // empty' "$lf" 2>/dev/null)"
        lpid="$(jq -r '.launcher_pid // empty' "$lf" 2>/dev/null)"
        dl="$(jq -r '.deadline // empty' "$lf" 2>/dev/null)"
        slot="$(jq -r '.slot // empty' "$lf" 2>/dev/null)"
        arch="$(jq -r '.arch // empty' "$lf" 2>/dev/null)"
        rid="$(jq -r '.run_id // empty' "$lf" 2>/dev/null)"
        src="$(jq -r '.source // empty' "$lf" 2>/dev/null)"

        # Only show VMs actually alive now; a lockfile whose qemu is gone is a
        # stale lock (the reaper purges it) — count it, don't list it.
        if [ -z "$qpid" ] || [ "$qpid" = "null" ] || ! pid_alive "$qpid" || ! proc_is_qemu "$qpid"; then
            stale=$(( stale + 1 ))
            continue
        fi
        running=$(( running + 1 ))

        # source: vm.py lockfiles have no "source" field → it's a smoke run.
        [ -z "$src" ] || [ "$src" = "null" ] && src="smoke"
        case "$src" in qemu/start.sh) src="start" ;; esac

        owner="$lpid"
        pid_alive "${lpid:-0}" || owner="${lpid}!"

        age="?"
        et="$(ps -o etimes= -p "$qpid" 2>/dev/null | tr -d ' ')"
        [ -n "$et" ] && age="$(human "$et")"

        ttl="-"
        if [ -n "$dl" ] && [ "$dl" != "null" ]; then
            rem=$(( dl - NOW ))
            if [ "$rem" -lt 0 ]; then ttl="OVERDUE"; else ttl="$(human "$rem")"; fi
        fi

        [ -n "$slot" ] && [ "$slot" != "null" ] || slot="-"

        # WATCH: where to follow this VM's serial.
        if [ "$src" = "smoke" ]; then
            watch="qemu/${arch}_smoke_live.log"
        elif [ -n "$rid" ] && [ "$rid" != "null" ]; then
            watch="tail.sh $rid"
        else
            watch="-"
        fi

        TABLE+=("$(printf '%-7s %-6s %-7s %-12s %-8s %-9s %-5s %s' \
            "${arch:-?}" "$src" "$qpid" "$owner" "$age" "$ttl" "$slot" "$watch")")

        if [ "$JSON" -eq 1 ]; then
            rows_json="$(jq \
                --argjson qemu_pid "$qpid" \
                --argjson launcher_pid "${lpid:-null}" \
                --arg arch "${arch:-}" --arg src "$src" \
                --arg run_id "${rid:-}" \
                --argjson deadline "${dl:-null}" \
                --argjson slot "${slot/-/null}" \
                --argjson owner_alive "$(pid_alive "${lpid:-0}" && echo true || echo false)" \
                --argjson age_s "${et:-null}" \
                '. + [{qemu_pid:$qemu_pid, launcher_pid:$launcher_pid, arch:$arch,
                       source:$src, run_id:$run_id, deadline:$deadline, slot:$slot,
                       owner_alive:$owner_alive, age_s:$age_s}]' \
                <<<"$rows_json" 2>/dev/null || echo "$rows_json")"
        fi
    done
fi

# --- slot usage -------------------------------------------------------------
slots_used=0
if [ -d "$QEMU_SLOT_DIR" ]; then
    for sf in "$QEMU_SLOT_DIR"/slot-*; do
        [ -f "$sf" ] || continue
        # If we CAN'T take the lock non-blocking, the slot is held (in use).
        if ( exec 9>>"$sf"; flock -n 9 ); then : ; else slots_used=$(( slots_used + 1 )); fi
    done
fi

# --- emit -------------------------------------------------------------------
if [ "$JSON" -eq 1 ]; then
    jq -n --argjson vms "$rows_json" \
          --argjson running "$running" --argjson stale "$stale" \
          --argjson slots_used "$slots_used" --argjson slots_total "$QEMU_MAX_CONCURRENT" \
          '{running:$running, stale_locks:$stale,
            slots: {used:$slots_used, total:$slots_total}, vms:$vms}'
    exit 0
fi

if [ "$running" -eq 0 ]; then
    echo "no QEMUs running"
else
    printf '%-7s %-6s %-7s %-12s %-8s %-9s %-5s %s\n' \
        ARCH SRC QPID OWNER AGE TTL SLOT WATCH
    for row in "${TABLE[@]}"; do echo "$row"; done
fi
printf 'slots: %d/%d in use' "$slots_used" "$QEMU_MAX_CONCURRENT"
[ "$stale" -gt 0 ] && printf '   (%d stale lock(s) — run reap.sh)' "$stale"
echo
