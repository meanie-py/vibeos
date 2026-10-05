#!/usr/bin/env bash
# The Pi 5's serial listener: tools/pi4/pico_serial.py on its Pico, logging under
# the Pi 5 bench directory (serial-current.json names the live capture).
#
#   tools/pi5/serial.sh up | down | status
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/env.sh"
PY=${PICO_PY:-$HOME/.venvs/picousb/bin/python}
PIDF=$PI4_BENCH/pico-serial.pid
mkdir -p "$PI4_BENCH"

live() {
    local p
    p=$(cat "$PIDF" 2>/dev/null) || return 1
    tr '\0' '\n' <"/proc/$p/cmdline" 2>/dev/null | grep -qx "$PI4_PICO_HOST" && echo "$p"
}

case "${1:-status}" in
    up)
        if p=$(live); then echo "serial listener: $p ($PI4_PICO_HOST)"; exit 0; fi
        setsid nohup "$PY" -u "$HERE/../pi4/pico_serial.py" "$PI4_BENCH" "$PI4_PICO_HOST" \
            >"$PI4_BENCH/pico-serial.log" 2>&1 </dev/null &
        echo $! >"$PIDF"
        sleep 3
        p=$(live) || { echo "pi5-serial: listener died; see $PI4_BENCH/pico-serial.log" >&2; exit 1; }
        echo "serial listener: $p ($PI4_PICO_HOST)"
        ;;
    down) if p=$(live); then kill "$p" && echo "stopped serial listener ($p)"; fi ;;
    status)
        if p=$(live); then echo "serial listener: $p ($PI4_PICO_HOST)"; else echo "serial listener: down"; fi
        cat "$PI4_BENCH/serial-current.json" 2>/dev/null; echo
        ;;
    *) echo "usage: $0 up|down|status" >&2; exit 2 ;;
esac
