#!/bin/bash
# Read serial output from a run, sanitized to printable ASCII + newlines.
#
# Usage:
#   qemu/tail.sh <run-id>                  # last 50 lines
#   qemu/tail.sh <run-id> --lines 200       # last N lines
#   qemu/tail.sh <run-id> --all              # whole log
#
# The on-disk serial.log stays raw bytes (KDB ANSI escapes, bugcheck binary
# garbage, all preserved). This script sanitizes on read with:
#     tr -cd '\11\12\15\40-\176'
# which keeps tab, LF, CR, and printable ASCII. Without this, agents grep'ing
# the output break in the middle of a match when KDB emits color codes.

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
. "$SCRIPT_DIR/lib/common.sh"
. "$SCRIPT_DIR/lib/manifest.sh"
. "$SCRIPT_DIR/lib/runid.sh"

RUN_ID="${1:-}"
LINES=50
ALL=0
shift 2>/dev/null || true
while [ $# -gt 0 ]; do
    case "$1" in
        --lines) shift; LINES="$1" ;;
        --all)   ALL=1 ;;
        *) die "unknown argument: $1" ;;
    esac
    shift
done

[ -n "$RUN_ID" ] || die "usage: tail.sh <run-id> [--lines N | --all]"
manifest_exists "$RUN_ID" || die "no such run-id: $RUN_ID"

LOG="$(runid_serial_log "$RUN_ID")"
[ -f "$LOG" ] || { log_warn "serial.log does not exist yet"; exit 0; }

if [ "$ALL" -eq 1 ]; then
    tr -cd '\11\12\15\40-\176' < "$LOG"
else
    tail -n "$LINES" "$LOG" | tr -cd '\11\12\15\40-\176'
fi
