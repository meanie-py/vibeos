#!/usr/bin/env bash
# The OptiPlex's hdmi: tools/pi4/hdmi.sh with the OptiPlex profile (env.sh).
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/env.sh"
exec bash "$HERE/../pi4/hdmi.sh" "$@"
