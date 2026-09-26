#!/bin/bash
# Read-only status of a run. No side effects.
#
# Usage:
#   qemu/status.sh <run-id>           # human-readable
#   qemu/status.sh <run-id> --json    # JSON (for agent consumption)

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"
. "$SCRIPT_DIR/lib/manifest.sh"
. "$SCRIPT_DIR/lib/runid.sh"

RUN_ID="${1:-}"
JSON=0
shift 2>/dev/null || true
for arg in "$@"; do
    case "$arg" in
        --json) JSON=1 ;;
        *) die "unknown argument: $arg" ;;
    esac
done

[ -n "$RUN_ID" ] || die "usage: status.sh <run-id> [--json]"
manifest_exists "$RUN_ID" || die "no such run-id: $RUN_ID"

PID="$(manifest_get "$RUN_ID" qemu_pid)"
GDB_PORT="$(manifest_get "$RUN_ID" gdb_port)"
STOPPED_AT="$(manifest_get "$RUN_ID" stopped_at)"
SERIAL_LOG="$(runid_serial_log "$RUN_ID")"

# Process state.
ALIVE=false
if [ -n "$PID" ] && [ "$PID" != "null" ] && kill -0 "$PID" 2>/dev/null; then
    ALIVE=true
fi

# Serial log size.
SERIAL_BYTES=0
if [ -f "$SERIAL_LOG" ]; then
    SERIAL_BYTES="$(stat -c %s "$SERIAL_LOG" 2>/dev/null || echo 0)"
fi

# gdb attach state — best-effort: try to connect briefly. Empty when no
# gdb_port is allocated.
GDB_REACHABLE=null
if [ -n "$GDB_PORT" ] && [ "$GDB_PORT" != "null" ]; then
    if timeout 0.2 bash -c "exec 3<>/dev/tcp/127.0.0.1/$GDB_PORT" 2>/dev/null; then
        GDB_REACHABLE=true
        exec 3>&- 3<&- 2>/dev/null || true
    else
        GDB_REACHABLE=false
    fi
fi

if [ "$JSON" -eq 1 ]; then
    vjson obj \
        --arg run_id "$RUN_ID" \
        --argjson alive "$ALIVE" \
        --argnum qemu_pid "$PID" \
        --argnum gdb_port "$GDB_PORT" \
        --argjson gdb_reachable "$GDB_REACHABLE" \
        --argjson serial_bytes "$SERIAL_BYTES" \
        --argopt stopped_at "$STOPPED_AT"
else
    cat <<EOF
run-id:        $RUN_ID
alive:         $ALIVE
qemu pid:      $PID
gdb port:      ${GDB_PORT:-none}
gdb reachable: $GDB_REACHABLE
serial bytes:  $SERIAL_BYTES
stopped at:    ${STOPPED_AT:-—}
EOF
fi
