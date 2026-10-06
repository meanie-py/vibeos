#!/usr/bin/env bash
# The Pi 5's Pico (picousb): tools/pi4/pico.sh with the Pi 5's host and lease.
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/env.sh"
exec bash "$HERE/../pi4/pico.sh" "$@"
