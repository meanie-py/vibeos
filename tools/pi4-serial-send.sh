#!/usr/bin/env bash
# Send a line to the Pi's UEFI Shell through the running Capture-RpiSerial.ps1
# listener, slowly enough that the firmware does not drop characters.
#
# Sending a whole command at once at 115200 with no flow control silently loses
# characters: "freeldr.ini fs0:\tftp-test.tmp" arrived as "freelt.tmp" and the
# command ran anyway, against the wrong filename. Corruption, not an error.
#
# The listener consumes serial-send.bin once per poll iteration, so writing one
# small chunk per iteration is self-pacing: wait for the file to disappear
# before writing the next chunk.
#
# Usage: pi4-serial-send.sh "ifconfig -l eth0"        # appends CR
#        pi4-serial-send.sh --raw $'\033OP'           # send bytes verbatim
set -u

BENCH=${PI4_BENCH:-/mnt/d/home/vibeos-rpi4-setup}
SEND="$BENCH/serial-send.bin"
CHUNK=${PI4_CHUNK:-4}
RAW=0
if [ "${1:-}" = "--raw" ]; then RAW=1; shift; fi
[ $# -ge 1 ] || { echo "usage: $0 [--raw] <text>" >&2; exit 2; }

payload=$1
[ "$RAW" -eq 1 ] || payload="$payload"$'\r'

wait_consumed() {
    # The listener deletes the file after writing it to the port.
    for _ in $(seq 1 60); do
        [ -e "$SEND" ] || return 0
        sleep 0.25
    done
    echo "pi4-serial-send: listener never consumed $SEND (is it running?)" >&2
    return 1
}

wait_consumed || exit 1
len=${#payload}
offset=0
while [ "$offset" -lt "$len" ]; do
    # Write then rename: a partially written serial-send.bin is a sharing
    # violation for the listener, and the listener used to die on it.
    printf '%s' "${payload:offset:CHUNK}" > "$SEND.tmp"
    mv -f "$SEND.tmp" "$SEND"
    wait_consumed || exit 1
    offset=$((offset + CHUNK))
done
