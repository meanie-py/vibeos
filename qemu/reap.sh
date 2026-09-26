#!/bin/bash
# Reap orphan QEMUs whose owning shell/launcher is dead.
#
# An "orphan" is a QEMU process whose tracked owner — the shell that ran
# qemu/start.sh, or the pytest process that invoked tests/qemu/vm.py — has
# exited. A QEMU whose owner is still alive — possibly another agent's
# worktree session — is NEVER touched.
#
# Walks two tracking systems:
#   1. $RUNS_DIR/*/manifest.json     (kernel-debug toolkit, owner_pid)
#   2. /tmp/vibeos-qemu-locks/*.json (pytest smoke harness, launcher_pid)
#
# Both fields are conceptually the same thing: the first long-lived non-shell
# ancestor of the launcher (claude / tmux / sshd / terminal-server). The kernel
# debug toolkit resolves it via qemu/lib/common.sh:owner_pid_for. vm.py just
# uses os.getpid() because pytest itself is the long-lived launcher.
#
# Usage: qemu/reap.sh [--dry-run] [--verbose]
#
# Safe to invoke from any worktree concurrently; idempotent. Output:
#   - one line per action (kept/skipped/reaped) to stderr if --verbose
#   - the final count of reaped processes to stdout

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"
. "$SCRIPT_DIR/lib/manifest.sh"

DRY_RUN=0
VERBOSE=0

while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run) DRY_RUN=1 ;;
        --verbose|-v) VERBOSE=1 ;;
        --help|-h) sed -n '2,15p' "$0"; exit 0 ;;
        *) die "unknown argument: $1" ;;
    esac
    shift
done

SMOKE_LOCK_DIR="/tmp/vibeos-qemu-locks"
REAPED=0

vlog() { [ "$VERBOSE" -eq 1 ] && printf '[reap] %s\n' "$*" >&2 || true; }

pid_alive() { kill -0 "$1" 2>/dev/null; }

# One JSON field, or empty. python3 rather than jq, deliberately: this script runs
# at the start of EVERY pytest session fleet-wide (tests/qemu/conftest.py), jq is
# not installed on the dev host, and `jq ... 2>/dev/null` failing silently made
# every lock read as "no qemu_pid" -- so each new smoke purged EVERY live VM's
# lock, unregistering them from `vibeos-cli ps`/`kill` and the lifetime cap.
# Exit status 2 means the file itself could not be read as JSON, which callers
# must treat as "unknown", never as "stale".
jfield() {
    python3 - "$1" "$2" <<'PY'
import json, sys
try:
    with open(sys.argv[1]) as f:
        d = json.load(f)
except (OSError, ValueError):
    sys.exit(2)
v = d.get(sys.argv[2]) if isinstance(d, dict) else None
if v is not None:
    print(v)
PY
}

proc_is_qemu() {
    local p="$1"
    [ -r "/proc/$p/comm" ] || return 1
    grep -q qemu "/proc/$p/comm" 2>/dev/null
}

# Kill a qemu PID's process group. We launch QEMUs in their own process group
# (start.sh via setsid, vm.py via preexec_fn=os.setpgrp), so pgid == pid in
# practice — but we read the real pgid defensively to avoid ever broadcasting
# a kill to the caller's shell pgroup if somebody changes the launcher.
kill_qemu_pgroup() {
    local qpid="$1" reason="$2"
    local pgid
    pgid="$(ps -o pgid= -p "$qpid" 2>/dev/null | tr -d ' ')"
    if [ -z "$pgid" ] || [ "$pgid" = "0" ]; then
        vlog "qemu pid=$qpid: no pgid (gone?); skipping"
        return 1
    fi
    if [ "$DRY_RUN" -eq 1 ]; then
        printf '[reap] DRY-RUN would reap qemu pid=%s pgid=%s — %s\n' \
            "$qpid" "$pgid" "$reason" >&2
        return 0
    fi
    kill -TERM -- "-$pgid" 2>/dev/null || true
    for _ in 1 2 3 4 5 6 7 8 9 10; do
        pid_alive "$qpid" || break
        sleep 0.2
    done
    if pid_alive "$qpid"; then
        kill -KILL -- "-$pgid" 2>/dev/null || true
        sleep 0.2
    fi
    if pid_alive "$qpid"; then
        printf '[reap] FAILED to kill qemu pid=%s pgid=%s — %s\n' \
            "$qpid" "$pgid" "$reason" >&2
        return 1
    fi
    printf '[reap] killed orphan qemu pid=%s pgid=%s — %s\n' \
        "$qpid" "$pgid" "$reason" >&2
    REAPED=$((REAPED + 1))
    return 0
}

# --- Walk 1: kernel-debug toolkit manifests ---------------------------------
if [ -d "$RUNS_DIR" ]; then
    for run_dir in "$RUNS_DIR"/*/; do
        [ -d "$run_dir" ] || continue
        rid="$(basename "$run_dir")"
        manifest="$run_dir/manifest.json"
        [ -f "$manifest" ] || continue

        stopped="$(jfield "$manifest" stopped_at)"
        if [ -n "$stopped" ]; then
            vlog "manifest $rid: stopped_at=$stopped — skipping"
            continue
        fi

        qpid="$(jfield "$manifest" qemu_pid)"
        opid="$(jfield "$manifest" owner_pid)"
        # Backwards compat with the brief parent_pid intermediate schema.
        [ -z "$opid" ] || [ "$opid" = "null" ] && \
            opid="$(jfield "$manifest" parent_pid)"

        if [ -z "$qpid" ] || [ "$qpid" = "null" ]; then
            vlog "manifest $rid: no qemu_pid — skipping"
            continue
        fi
        if ! pid_alive "$qpid"; then
            vlog "manifest $rid: qemu pid=$qpid already dead — skipping"
            continue
        fi
        if ! proc_is_qemu "$qpid"; then
            vlog "manifest $rid: pid=$qpid is not a qemu process (pid reuse?) — skipping"
            continue
        fi
        # Hard lifetime cap takes precedence over owner liveness: a VM past its
        # deadline is reaped even if its owner is still alive (it has overrun
        # any legitimate lifetime). Owner liveness only protects within the cap.
        dl="$(jfield "$manifest" deadline)"
        dl="${dl%.*}"   # tolerate a float deadline — `[ -ge ]` errors on one
        if [ -n "$dl" ] && [ "$dl" != "null" ] && [ "$(date +%s)" -ge "$dl" ] 2>/dev/null; then
            kill_qemu_pgroup "$qpid" "manifest $rid exceeded lifetime cap (deadline=$dl)"
            continue
        fi
        if [ -z "$opid" ] || [ "$opid" = "null" ]; then
            vlog "manifest $rid: no owner_pid recorded (legacy manifest), within cap — skipping"
            continue
        fi
        if pid_alive "$opid"; then
            vlog "manifest $rid: owner_pid=$opid alive, within cap — skipping (owner still here)"
            continue
        fi
        kill_qemu_pgroup "$qpid" "manifest $rid owner pid=$opid dead"
    done
fi

# --- Walk 2: pytest smoke-harness lockfiles ---------------------------------
if [ -d "$SMOKE_LOCK_DIR" ]; then
    for lf in "$SMOKE_LOCK_DIR"/*.json; do
        [ -f "$lf" ] || continue
        if ! python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$lf" 2>/dev/null; then
            vlog "lock $(basename "$lf"): unreadable as JSON -- skipping (unknown is not stale)"
            continue
        fi
        lpid="$(jfield "$lf" launcher_pid)"
        qpid="$(jfield "$lf" qemu_pid)"
        dl="$(jfield "$lf" deadline)"
        dl="${dl%.*}"   # tolerate a float deadline (smoke lockfiles) — see Walk 1
        if [ -z "$qpid" ] || [ "$qpid" = "null" ]; then
            vlog "lock $(basename "$lf"): no qemu_pid — purging stale lock"
            [ "$DRY_RUN" -eq 0 ] && rm -f "$lf"
            continue
        fi
        # Hard lifetime cap precedes launcher liveness — see Walk 1.
        expired=0
        if [ -n "$dl" ] && [ "$dl" != "null" ] && [ "$(date +%s)" -ge "$dl" ] 2>/dev/null; then
            expired=1
        fi
        if [ "$expired" -eq 0 ] && [ -n "$lpid" ] && [ "$lpid" != "null" ] && pid_alive "$lpid"; then
            vlog "lock $(basename "$lf"): launcher_pid=$lpid alive, within cap — skipping"
            continue
        fi
        if ! pid_alive "$qpid"; then
            vlog "lock $(basename "$lf"): qemu pid=$qpid already dead — purging"
            [ "$DRY_RUN" -eq 0 ] && rm -f "$lf"
            continue
        fi
        if ! proc_is_qemu "$qpid"; then
            vlog "lock $(basename "$lf"): pid=$qpid is not a qemu (pid reuse?) — purging"
            [ "$DRY_RUN" -eq 0 ] && rm -f "$lf"
            continue
        fi
        if [ "$expired" -eq 1 ]; then
            reason="lockfile exceeded lifetime cap (deadline=$dl)"
        else
            reason="lockfile launcher pid=$lpid dead"
        fi
        if kill_qemu_pgroup "$qpid" "$reason"; then
            [ "$DRY_RUN" -eq 0 ] && rm -f "$lf"
        fi
    done
fi

printf '%s\n' "$REAPED"
exit 0
