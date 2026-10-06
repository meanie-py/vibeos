#!/usr/bin/env bash
# The Pi 5's plug: tools/pi4/power.sh on PI5_PLUG. Same verbs (status|on|off|cycle).
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/env.sh"
# power.sh falls back to the Pi 4's plug when none is named, so an unset PI5_PLUG
# must stop here: it would power-cycle the OTHER board under whoever is using it.
[ -n "$PI4_PLUG" ] || { echo "pi5-power: PI5_PLUG is not set (the Pi 5's Shelly plug)" >&2; exit 1; }
exec bash "$HERE/../pi4/power.sh" "$@"
