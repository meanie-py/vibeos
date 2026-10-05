#!/usr/bin/env bash
# One frame from the Pi 5's capture card: tools/pi4/hdmi.sh on the other card.
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/env.sh"
exec bash "$HERE/../pi4/hdmi.sh" "$@"
