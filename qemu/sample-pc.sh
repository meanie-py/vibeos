#!/bin/bash
# Drive a sampling profiler against a running QEMU run.
#
# Usage:
#   qemu/sample-pc.sh <run-id> [--samples N] [--delay-ms MS]
#
# Each sample = one inspect.sh invocation that reads $pc and the current
# thread's process->ImageFileName.  Between samples we sleep delay_ms.
# The QEMU target runs freely between gdb attach/detach cycles, so the
# delay actually corresponds to elapsed runtime.
#
# Output: per-sample lines + two histograms (by symbol, by process).
#
# Defaults: 20 samples 250ms apart (~5s wall clock + ~1s/sample gdb
# overhead = ~25s total).
#
# Trade-off: each gdb spawn costs ~0.5–1.5s, so the effective sampling
# rate is ~1Hz — fine for "what is the system doing right now" but won't
# catch sub-second-duration spikes.  For tighter sampling, we'd need to
# drive gdb interactively (left as future work).

set -u

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

RUN_ID="${1:-}"
[ -n "$RUN_ID" ] || { echo "usage: $0 <run-id> [--samples N] [--delay-ms MS]"; exit 2; }
shift

SAMPLES=20
DELAY_MS=250
while [ $# -gt 0 ]; do
    case "$1" in
        --samples)  shift; SAMPLES="$1" ;;
        --delay-ms) shift; DELAY_MS="$1" ;;
        *) echo "unknown arg: $1"; exit 2 ;;
    esac
    shift
done

DELAY_S=$(awk -v d="$DELAY_MS" 'BEGIN{printf "%.3f", d/1000}')

SAMPLE_LOG="$(mktemp -t sample-pc.XXXXXX)"
trap 'rm -f "$SAMPLE_LOG"' EXIT

echo "=== sample-pc.sh $RUN_ID  samples=$SAMPLES delay=${DELAY_MS}ms  $(date -u +%FT%TZ) ==="
echo

for ((i=1; i<=SAMPLES; i++)); do
    line="$("$SCRIPT_DIR/inspect.sh" "$RUN_ID" --command sample-pc 2>/dev/null \
            | grep '^SAMPLE ' | head -1)"
    if [ -z "$line" ]; then
        printf "[%3d/%d] <no sample>\n" "$i" "$SAMPLES"
    else
        printf "[%3d/%d] %s\n" "$i" "$SAMPLES" "$line"
        echo "$line" >> "$SAMPLE_LOG"
    fi
    if [ "$i" -lt "$SAMPLES" ]; then
        sleep "$DELAY_S"
    fi
done

n_samples=$(wc -l < "$SAMPLE_LOG")
[ "$n_samples" = "0" ] && { echo; echo "(no samples captured)"; exit 1; }

echo
echo "=== histogram by symbol (n=$n_samples) ==="
sed -nE 's/.*sym=(.*)$/\1/p' "$SAMPLE_LOG" \
    | sort | uniq -c | sort -k1,1nr \
    | awk -v n="$n_samples" '{
        cnt = $1; $1 = ""; sub(/^ +/, "");
        printf "  %4d  %5.1f%%  %s\n", cnt, 100*cnt/n, $0
    }'

echo
echo "=== histogram by process (n=$n_samples) ==="
sed -nE 's/.*proc=([^ ]+).*/\1/p' "$SAMPLE_LOG" \
    | sort | uniq -c | sort -k1,1nr \
    | awk -v n="$n_samples" '{
        cnt = $1; $1 = ""; sub(/^ +/, "");
        printf "  %4d  %5.1f%%  %s\n", cnt, 100*cnt/n, $0
    }'
