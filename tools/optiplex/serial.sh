#!/usr/bin/env bash
# The OptiPlex's serial listener: Capture-RpiSerial.ps1 on the Windows COM port
# wired to its COM1. Each start writes <bench>/serial-<timestamp>/serial.bin and
# names the live capture in <bench>/serial-current.json. coreboot logs every boot
# here; VibeOS's kernel debug output follows on the same port.
#
#   tools/optiplex/serial.sh up | down | status
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/env.sh"
PS=/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe
mkdir -p "$PI4_BENCH"
WBENCH=$(wslpath -w "$PI4_BENCH")
SCRIPT="$WBENCH\\Capture-RpiSerial.ps1"

# Our listener only: matched by its full script path, so another bench's
# Capture-RpiSerial.ps1 (a different directory) is never touched.
# A failed query is not "no listener": that misreading once started a second
# listener, which could not open the port and overwrote serial-current.json.
live() {
    local out
    out=$(timeout 20 "$PS" -NoProfile -Command "Get-CimInstance Win32_Process | Where-Object { \$_.CommandLine -like '*$SCRIPT*' -and \$_.CommandLine -notlike '*Get-CimInstance*' } | ForEach-Object { \$_.ProcessId }; 'query-ok'" 2>/dev/null </dev/null | tr -d '\r')
    case "$out" in
        *query-ok*) printf '%s\n' "$out" | grep -v query-ok | head -1 ;;
        *) echo "optiplex-serial: could not list Windows processes (interop); rerun" >&2; exit 1 ;;
    esac
}

case "${1:-status}" in
    up)
        p=$(live) || exit 1
        if [ -n "$p" ]; then echo "serial listener: $p ($OPTIPLEX_COM)"; exit 0; fi
        cp "$HERE/../pi4/Capture-RpiSerial.ps1" "$PI4_BENCH/"
        # The status file is the listener's own report; clear the last one first so a
        # listener that never started cannot read as LISTENING.
        rm -f "$PI4_BENCH/serial-current.json"
        timeout 15 "$PS" -NoProfile -Command "Start-Process -FilePath powershell.exe -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','$SCRIPT','-PortName','$OPTIPLEX_COM' -WindowStyle Hidden" >/dev/null 2>&1 </dev/null
        for _ in 1 2 3 4 5 6 7 8 9 10; do [ -s "$PI4_BENCH/serial-current.json" ] && break; sleep 1; done
        # The listener's own status file says whether it opened the port; asking
        # Windows for its pid as well only adds an interop call that can time out.
        cur="$PI4_BENCH/serial-current.json"
        if ! grep -q '"LISTENING"' "$cur" 2>/dev/null; then
            echo "optiplex-serial: listener is not LISTENING ($OPTIPLEX_COM held by another program?)" >&2
            exit 1
        fi
        echo "serial listener: $(grep -o '"pid": *[0-9]*' "$cur" | grep -o '[0-9]*$') ($OPTIPLEX_COM)"
        ;;
    down)
        p=$(live) || exit 1
        if [ -n "$p" ]; then
            timeout 15 "$PS" -NoProfile -Command "Stop-Process -Id $p" >/dev/null 2>&1
            echo "stopped serial listener ($p)"
        fi
        ;;
    status)
        p=$(live) || exit 1
        if [ -n "$p" ]; then echo "serial listener: $p ($OPTIPLEX_COM)"; else echo "serial listener: down"; fi
        cat "$PI4_BENCH/serial-current.json" 2>/dev/null; echo
        ;;
    *) echo "usage: $0 up|down|status" >&2; exit 2 ;;
esac
