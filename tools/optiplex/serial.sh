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
live() {
    timeout 20 "$PS" -NoProfile -Command "Get-CimInstance Win32_Process | Where-Object { \$_.CommandLine -like '*$SCRIPT*' -and \$_.CommandLine -notlike '*Get-CimInstance*' } | ForEach-Object { \$_.ProcessId }" 2>/dev/null | tr -d '\r' | head -1
}

case "${1:-status}" in
    up)
        p=$(live)
        if [ -n "$p" ]; then echo "serial listener: $p ($OPTIPLEX_COM)"; exit 0; fi
        cp "$HERE/../pi4/Capture-RpiSerial.ps1" "$PI4_BENCH/"
        timeout 15 "$PS" -NoProfile -Command "Start-Process -FilePath powershell.exe -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','$SCRIPT','-PortName','$OPTIPLEX_COM' -WindowStyle Hidden" >/dev/null 2>&1 </dev/null
        sleep 3
        p=$(live)
        if [ -z "$p" ] || ! grep -q '"LISTENING"' "$PI4_BENCH/serial-current.json" 2>/dev/null; then
            echo "optiplex-serial: listener is not LISTENING ($OPTIPLEX_COM held by another program?)" >&2
            exit 1
        fi
        echo "serial listener: $p ($OPTIPLEX_COM)"
        ;;
    down)
        p=$(live)
        if [ -n "$p" ]; then
            timeout 15 "$PS" -NoProfile -Command "Stop-Process -Id $p" >/dev/null 2>&1
            echo "stopped serial listener ($p)"
        fi
        ;;
    status)
        p=$(live)
        if [ -n "$p" ]; then echo "serial listener: $p ($OPTIPLEX_COM)"; else echo "serial listener: down"; fi
        cat "$PI4_BENCH/serial-current.json" 2>/dev/null; echo
        ;;
    *) echo "usage: $0 up|down|status" >&2; exit 2 ;;
esac
