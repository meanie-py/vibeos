#!/usr/bin/env bash
# Switch the Pi 4B bench's mains power through its Shelly plug (Gen2+ local RPC).
#
#   tools/pi4/power.sh status        # on/off and watts
#   tools/pi4/power.sh on | off
#   tools/pi4/power.sh cycle [secs]  # off, then the PLUG turns it back on
#
# `cycle` uses the plug's own toggle_after timer, so the "back on" half runs on
# the device: if this script, WSL or the host dies mid-cycle the board still
# comes back instead of being left dark.
#
# Never cycle while the board is writing the SD card (a TFTP transfer onto fs0:)
# - cutting power mid-write can corrupt the FAT volume the firmware boots from.
set -u

# The Pi 4's plug since 2026-10-18 (it was the Pi 5's; e8f60a7df904, the Pi 4's
# first, now feeds the OptiPlex, ~40 W, and tools/optiplex/env.sh names it).
PLUG=${PI4_PLUG:-shellyplugusg4-acebe6f18e28}
# Digest auth, when the plug has a password set (Shelly's user is always admin).
AUTH=()
[ -n "${PI4_PLUG_PASS:-}" ] && AUTH=(--digest -u "admin:$PI4_PLUG_PASS")

rpc() {
    curl -sS -m 5 --fail "${AUTH[@]}" "http://$PLUG/rpc/$1" || {
        echo "pi4-power: plug $PLUG did not answer $1" >&2
        return 1
    }
}

status() {
    rpc 'Switch.GetStatus?id=0' | python3 -c '
import json, sys
s = json.load(sys.stdin)
print(("on " if s["output"] else "off"), "%.1f W" % s.get("apower", 0.0))'
}

. "$(dirname "${BASH_SOURCE[0]}")/lease.sh"
case "${1:-status}" in status) ;; *) pi4_lease ;; esac

case "${1:-status}" in
    status) status ;;
    on)  rpc 'Switch.Set?id=0&on=true' >/dev/null && status ;;
    off) rpc 'Switch.Set?id=0&on=false' >/dev/null && status ;;
    cycle)
        secs=${2:-5}
        # The PSU's capacitors hold the board up for a moment; a too-short gap
        # is a brown-out, not a reset.
        rpc "Switch.Set?id=0&on=false&toggle_after=$secs" >/dev/null || exit 1
        echo "off; the plug turns it back on in ${secs}s" >&2
        sleep "$((secs + 1))"
        status
        ;;
    *) echo "usage: $0 status|on|off|cycle [secs]" >&2; exit 2 ;;
esac
