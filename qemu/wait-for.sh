#!/bin/bash
# Poll the serial log for a regex pattern. On match, exit 0 with the matched
# line on stdout. On timeout, exit nonzero with a structured recovery report.
#
# Usage:
#   qemu/wait-for.sh <run-id> --pattern <regex> --timeout <secs> [--json]
#
# Sanitizes serial output via tr filter before matching, so KDB ANSI escapes
# and bugcheck binary garbage don't break grep mid-match.
#
# Timeout response (--json) shape:
#   {
#     "matched": false,
#     "elapsed_secs": 60,
#     "serial_bytes": 12345,
#     "alive": true,
#     "tail": [".....last 10 sanitized lines....."]
#   }

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"
. "$SCRIPT_DIR/lib/manifest.sh"
. "$SCRIPT_DIR/lib/runid.sh"

RUN_ID="${1:-}"
PATTERN=""
TIMEOUT=60
JSON=0
shift 2>/dev/null || true
while [ $# -gt 0 ]; do
    case "$1" in
        --pattern) shift; PATTERN="$1" ;;
        --timeout) shift; TIMEOUT="$1" ;;
        --json)    JSON=1 ;;
        *) die "unknown argument: $1" ;;
    esac
    shift
done

[ -n "$RUN_ID" ]  || die "usage: wait-for.sh <run-id> --pattern <regex> --timeout <secs>"
[ -n "$PATTERN" ] || die "--pattern required"
manifest_exists "$RUN_ID" || die "no such run-id: $RUN_ID"

LOG="$(runid_serial_log "$RUN_ID")"
PID="$(manifest_get "$RUN_ID" qemu_pid)"

start_ts="$(date +%s)"
deadline=$((start_ts + TIMEOUT))

# Poll loop. We re-scan from byte 0 each iteration — serial.log is append-only
# and small enough that this is cheap. (We could track a cursor, but a 1MB
# log scanned once a second is well under the noise floor.)
while :; do
    now="$(date +%s)"
    if [ "$now" -ge "$deadline" ]; then
        break
    fi

    if [ -f "$LOG" ]; then
        match="$(tr -cd '\11\12\15\40-\176' < "$LOG" | grep -E -m 1 "$PATTERN" || true)"
        if [ -n "$match" ]; then
            elapsed=$((now - start_ts))
            if [ "$JSON" -eq 1 ]; then
                vjson obj -c --argjson matched true --argjson elapsed_secs "$elapsed" \
                    --arg line "$match"
            else
                echo "$match"
            fi
            exit 0
        fi
    fi

    # Bail early if QEMU died — no point waiting for output that will never come.
    if [ -n "$PID" ] && [ "$PID" != "null" ] && ! kill -0 "$PID" 2>/dev/null; then
        log_warn "QEMU pid $PID is no longer alive — pattern still not seen"
        break
    fi

    sleep 1
done

# Timeout (or QEMU died) — assemble recovery context.
elapsed=$(( $(date +%s) - start_ts ))
serial_bytes=0
[ -f "$LOG" ] && serial_bytes="$(stat -c %s "$LOG" 2>/dev/null || echo 0)"
alive=false
if [ -n "$PID" ] && [ "$PID" != "null" ] && kill -0 "$PID" 2>/dev/null; then
    alive=true
fi

# Last 10 lines, sanitized.
tail_lines="$(tail -n 10 "$LOG" 2>/dev/null | tr -cd '\11\12\15\40-\176' || true)"

if [ "$JSON" -eq 1 ]; then
    vjson obj -c \
        --argjson matched false \
        --argjson elapsed_secs "$elapsed" \
        --argjson serial_bytes "$serial_bytes" \
        --argjson alive "$alive" \
        --arglines tail "$tail_lines"
else
    cat <<EOF
[TIMEOUT] pattern '$PATTERN' not seen in ${elapsed}s
serial bytes: $serial_bytes
alive:        $alive
last 10 lines:
$tail_lines
EOF
fi

exit 1
