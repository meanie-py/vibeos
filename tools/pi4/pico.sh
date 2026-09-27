#!/usr/bin/env bash
# pico.sh — drive the Pico 2 W USB input device plugged into the Pi 4.
#
# The Pico presents a boot keyboard, boot mouse, mass storage and an absolute
# pointer to the Pi, and reports what the Pi's USB host did to it (configured,
# bus resets, per-endpoint report completions). That report is the oracle for
# usbxhci bring-up: "mounted" plus a growing completion count means the Pi's
# driver enumerated the device and is polling it.
#
#   tools/pi4/pico.sh status            # full JSON
#   tools/pi4/pico.sh usb               # one line: mounted / resets / completions
#   tools/pi4/pico.sh type 'hello' --delay 30
#   tools/pi4/pico.sh key F1 --hold 40 --repeat 20 --interval 100
#   tools/pi4/pico.sh pointer click 16384 16384
#
# Anything else is passed to pico-devbox unchanged. The client is the Pico
# project's own (D:\home\Projects\picousb, host/), installed into a venv:
#   python3 -m venv ~/.venvs/picousb && ~/.venvs/picousb/bin/pip install -e <picousb>/host
#
# Environment:
#   PICO_URL               default ws://picousb:8080/ws (router DNS; mDNS does not work in WSL)
#   PICO_DEVICE_TOKEN_FILE default ~/.config/picousb/token. Created by the bench owner;
#                          agents pass the path and never read the file.
#   PICO_DEVBOX            default ~/.venvs/picousb/bin/pico-devbox
# Exit codes are pico-devbox's: 3 unreachable, 4 auth, 5 USB not mounted,
# 6 host not polling, 7 USB reset during the operation.
set -euo pipefail

PICO_URL="${PICO_URL:-ws://picousb:8080/ws}"
export PICO_DEVICE_TOKEN_FILE="${PICO_DEVICE_TOKEN_FILE:-$HOME/.config/picousb/token}"
PICO_DEVBOX="${PICO_DEVBOX:-$HOME/.venvs/picousb/bin/pico-devbox}"

if [ ! -x "$PICO_DEVBOX" ]; then
    echo "pico.sh: $PICO_DEVBOX not found; install the client (see header)" >&2
    exit 1
fi
if [ ! -f "$PICO_DEVICE_TOKEN_FILE" ]; then
    echo "pico.sh: token file $PICO_DEVICE_TOKEN_FILE missing; the bench owner creates it" >&2
    exit 4
fi

if [ "${1:-}" = "usb" ]; then
    out=$("$PICO_DEVBOX" --url "$PICO_URL" status </dev/null) || exit $?
    printf '%s' "$out" | python3 -c '
import json, sys
u = json.load(sys.stdin)["usb"]
print("uptime=%ds mounted=%s suspended=%s resets=%d set_config=%d completed=%s" % (
    u["uptime_ms"] // 1000, u["mounted"], u["suspended"], u["bus_resets"],
    u["set_configuration_count"], [h["lifetime_completed"] for h in u["hid"]]))'
    exit 0
fi

exec "$PICO_DEVBOX" --url "$PICO_URL" "$@" </dev/null
