#!/usr/bin/env bash
# The OptiPlex's power: tools/pi4/power.sh with the OptiPlex profile (env.sh).
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/env.sh"
exec bash "$HERE/../pi4/power.sh" "$@"
