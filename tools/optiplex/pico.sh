#!/usr/bin/env bash
# The OptiPlex's pico: tools/pi4/pico.sh with the OptiPlex profile (env.sh).
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/env.sh"
exec bash "$HERE/../pi4/pico.sh" "$@"
