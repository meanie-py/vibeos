#!/bin/bash
# Stop a QEMU run. Idempotent — safe to call on already-stopped runs.
#
# Usage:
#   qemu/stop.sh <run-id>
#
# Side effects:
#   - SIGTERM, then SIGKILL if needed, the qemu process from the manifest
#   - Records stopped_at in manifest
#   - Preserves serial.log, manifest.json, gdb.log

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"
. "$SCRIPT_DIR/lib/manifest.sh"
. "$SCRIPT_DIR/lib/runid.sh"

RUN_ID="${1:-}"
[ -n "$RUN_ID" ] || die "usage: stop.sh <run-id>"
manifest_exists "$RUN_ID" || die "no such run-id: $RUN_ID"

PID="$(manifest_get "$RUN_ID" qemu_pid)"
if [ -z "$PID" ] || [ "$PID" = "null" ]; then
    log_info "no qemu_pid recorded for $RUN_ID — nothing to do"
    manifest_set "$RUN_ID" stopped_at "$(date -u +%FT%TZ)"
    exit 0
fi

if ! kill -0 "$PID" 2>/dev/null; then
    log_info "qemu pid $PID is already gone"
    manifest_set "$RUN_ID" stopped_at "$(date -u +%FT%TZ)"
    exit 0
fi

log_info "sending SIGTERM to QEMU pid $PID"
kill -TERM "$PID" 2>/dev/null || true

# Up to 5s grace period for clean exit.
for _ in 1 2 3 4 5 6 7 8 9 10; do
    kill -0 "$PID" 2>/dev/null || break
    sleep 0.5
done

if kill -0 "$PID" 2>/dev/null; then
    log_warn "QEMU pid $PID didn't exit on SIGTERM, sending SIGKILL"
    kill -KILL "$PID" 2>/dev/null || true
    sleep 0.2
fi

manifest_set "$RUN_ID" stopped_at "$(date -u +%FT%TZ)"

# Purge the shared lockfile if start.sh wrote one. Best-effort; the reaper
# also cleans up stale locks naturally.
OWNER_FIELD="$(manifest_get "$RUN_ID" owner_pid 2>/dev/null || echo null)"
if [ -n "$OWNER_FIELD" ] && [ "$OWNER_FIELD" != "null" ]; then
    rm -f "/tmp/vibeos-qemu-locks/${OWNER_FIELD}-${PID}.json" 2>/dev/null || true
fi

log_info "QEMU stopped (run-id=$RUN_ID)"
