#!/usr/bin/env bash
# Host side of the Pi 4B bench, driven from WSL. Guide: docs/engineering/pi4-bench.md
#
#   tools/pi4/bench.sh status
#   tools/pi4/bench.sh up       # serial capture listener + TFTP + sanboot HTTP servers
#   tools/pi4/bench.sh down
#   tools/pi4/bench.sh stage    # copy the current arm64 build into the TFTP payload
#   tools/pi4/bench.sh http     # just the sanboot HTTP server (needs the TCP 8069 rule)
#   tools/pi4/bench.sh serial-restart  # replace a listener whose capture has stalled
#
# The serial listener is PI4_SERIAL=pico (default): pico_serial.py in WSL, talking
# to the Pico 2 W's GPIO UART bridge over Wi-Fi. PI4_SERIAL=com runs the old
# Capture-RpiSerial.ps1 on a Windows USB-serial port instead. Both write the same
# serial-current.json / serial.bin / serial-send.bin contract.
#
# The TFTP/HTTP services run as WINDOWS processes: WSL2 in NAT mode (172.x) is
# unreachable from the LAN the Pi is on. `up`
# first copies the repo's versions of the scripts into the bench directory, so
# the repo stays the source of truth and Windows never runs a script over a
# \\wsl.localhost path (execution policy and UNC quirks).
set -u

HERE=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$HERE/../.." && pwd)
BENCH=${PI4_BENCH:-/mnt/d/home/vibeos-rpi4-setup}
PORT=${PI4_COM:-COM5}
SERIAL=${PI4_SERIAL:-pico}
PICO_PY=${PICO_PY:-$HOME/.venvs/picousb/bin/python}
PICO_PID="$BENCH/pico-serial.pid"
PS=/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe
HTTP_PORT=${PI4_HTTP_PORT:-8069}
HOST_IP=${PI4_HOST_IP:-192.168.1.183}
# Windows python (the Microsoft Store stub does not count).
PY=${PI4_WINPY:-'C:\Users\Lambda\AppData\Local\Programs\Python\Python313\python.exe'}

WBENCH=$(wslpath -w "$BENCH")
ps_run() { "$PS" -NoProfile -Command "$1" 2>&1 | tr -d '\r'; }
# Start-Process children inherit powershell's stdout; if that is a pipe, the
# pipe never closes and the caller hangs until the service exits. Detach fully.
# Start-Process can also leave powershell.exe running after the child is up
# (seen 2026-10-03: `up` sat 7 minutes with TFTP already serving), so bound it;
# every caller re-checks the service by PID afterwards.
ps_launch() { timeout 15 "$PS" -NoProfile -Command "$1" >/dev/null 2>&1 </dev/null; }

# PIDs by command line, so we never touch an unrelated powershell/python.
pids() {
    ps_run "Get-CimInstance Win32_Process | Where-Object { \$_.CommandLine -like '*$1*' -and \$_.CommandLine -notlike '*Get-CimInstance*' } | ForEach-Object { \$_.ProcessId }"
}

pico_serial_pid() {
    local p
    p=$(cat "$PICO_PID" 2>/dev/null) || return 1
    # -a: cmdline is NUL-separated, and grep calls that binary and reports no match.
    grep -qa pico_serial.py "/proc/$p/cmdline" 2>/dev/null && echo "$p"
}

serial_pids() {
    if [ "$SERIAL" = pico ]; then pico_serial_pid; else pids 'Capture-RpiSerial.ps1'; fi
}

status() {
    local s t
    s=$(serial_pids | xargs)
    t=$(pids 'serve_tftp.py' | xargs)
    printf 'serial listener : %s (%s)\n' "${s:-down}" "$SERIAL"
    python3 - "$BENCH" <<'PY'
import json, pathlib, sys
p = pathlib.Path(sys.argv[1]) / 'serial-current.json'
try:
    s = json.loads(p.read_text(encoding='utf-8-sig'))
    print('  last state    :', s.get('status'), s.get('directory', ''), s.get('error', ''))
except Exception as e:
    print('  last state    : unreadable', e)
PY
    printf 'tftp server     : %s\n' "${t:-down}"
    printf 'http server     : %s\n' "$(pids 'serve_http.py' | xargs)"
    printf 'power           : %s\n' "$(bash "$HERE/power.sh" status 2>&1)"
}

up() {
    mkdir -p "$BENCH/payload"
    cp "$HERE/Capture-RpiSerial.ps1" "$HERE/serve_tftp.py" "$BENCH/"
    if [ "$SERIAL" = pico ]; then
        # A COM listener run in between rewrites serial-current.json, leaving a
        # live pico listener that the state file no longer names: restart it.
        local live
        live=$(pico_serial_pid)
        if [ -n "$live" ] && ! grep -q "\"pid\": $live," "$BENCH/serial-current.json" 2>/dev/null; then
            kill "$live" && echo "restarting pico_serial.py ($live): serial-current.json names another listener"
            sleep 2
        fi
        if [ -z "$(pico_serial_pid)" ]; then
            setsid nohup "$PICO_PY" -u "$HERE/pico_serial.py" "$BENCH" \
                >"$BENCH/pico-serial.log" 2>&1 </dev/null &
            echo $! > "$PICO_PID"
            sleep 3
        fi
    elif [ -z "$(pids 'Capture-RpiSerial.ps1')" ]; then
        ps_launch "Start-Process -FilePath powershell.exe -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','$WBENCH\\Capture-RpiSerial.ps1','-PortName','$PORT' -WindowStyle Hidden"
        sleep 3
    fi
    if [ -z "$(pids 'serve_tftp.py')" ]; then
        ps_launch "Start-Process -FilePath '$PY' -ArgumentList '-u','$WBENCH\\serve_tftp.py','$WBENCH\\payload','0.0.0.0' -WindowStyle Hidden -RedirectStandardOutput '$WBENCH\\tftp-server.log' -RedirectStandardError '$WBENCH\\tftp-server.err'"
        sleep 2
    fi
    status
    # A listener that started and then died leaves FAILED behind; say so loudly.
    grep -q '"LISTENING"' "$BENCH/serial-current.json" 2>/dev/null || {
        if [ "$SERIAL" = pico ]; then
            echo "pi4-bench: pico serial listener is not LISTENING - see $BENCH/pico-serial.log" >&2
        else
            echo "pi4-bench: serial listener is not LISTENING ($PORT held by another program?)" >&2
        fi
        return 1
    }
    [ -n "$(pids 'serve_tftp.py')" ] || {
        echo "pi4-bench: TFTP server exited - see $BENCH/tftp-server.err" >&2
        return 1
    }
    http
}

# The sanboot server is started only on request: a first TCP listen from a new
# program can raise a Windows Firewall prompt on the desktop, and without an
# inbound rule for the port the Pi cannot reach it anyway (docs/engineering/pi4-bench.md).
http() {
    cp "$HERE/serve_http.py" "$BENCH/"
    if [ -z "$(pids 'serve_http.py')" ]; then
        # This Start-Process never returns on its own (python is serving within
        # a second; powershell.exe stays). Bound it and trust the PID check below.
        timeout 10 "$PS" -NoProfile -Command "Start-Process -FilePath '$PY' -ArgumentList '-u','$WBENCH\\serve_http.py','$WBENCH\\payload','$HOST_IP','$HTTP_PORT' -WindowStyle Hidden -RedirectStandardOutput '$WBENCH\\http-server.log' -RedirectStandardError '$WBENCH\\http-server.err'" >/dev/null 2>&1 </dev/null
        sleep 2
    fi
    [ -n "$(pids 'serve_http.py')" ] || {
        echo "pi4-bench: HTTP server exited - see $BENCH/http-server.err" >&2
        return 1
    }
    echo "http server on $HOST_IP:$HTTP_PORT"
}

down() {
    local p
    if p=$(pico_serial_pid); then
        kill "$p" && echo "stopped pico_serial.py ($p)"
    fi
    for pat in 'Capture-RpiSerial.ps1' 'serve_tftp.py' 'serve_http.py'; do
        for p in $(pids "$pat"); do
            ps_run "Stop-Process -Id $p -Force" >/dev/null && echo "stopped $pat ($p)"
        done
    done
}

# The payload is what the Pi pulls: the loader, the Pi's freeldr.ini, the ramdisk.
# The manifest records what was staged from where, so a boot result can always be
# tied back to a commit instead of to "whatever was on the card".
stage() {
    local efi="$REPO/build-arm64/boot/freeldr/freeldr/uefildr.efi"
    local img="$REPO/qemu/ramdisk.img"
    for f in "$efi" "$img" "$REPO/qemu/boot.img"; do
        [ -f "$f" ] || { echo "pi4-bench: missing $f - run ./vibeos-cli build --arch arm64" >&2; return 1; }
    done
    mkdir -p "$BENCH/payload/EFI/BOOT"
    cp "$efi" "$BENCH/payload/EFI/BOOT/BOOTAA64.EFI"
    # The MP pair (ntkrnlmp.exe + halmp.dll) is the default; PI4_KERNEL=up boots
    # the UP pair instead. Only the staged copy's DefaultOS= changes; the sed
    # keeps the CRLF line ending.
    case "${PI4_KERNEL:-mp}" in
        mp) cp "$HERE/freeldr.ini" "$BENCH/payload/freeldr.ini" ;;
        up) sed 's/^DefaultOS=ReactOS_Arm64_MP\(\r\{0,1\}\)$/DefaultOS=ReactOS_Arm64_UP\1/' \
                "$HERE/freeldr.ini" > "$BENCH/payload/freeldr.ini"
            grep -q '^DefaultOS=ReactOS_Arm64_UP' "$BENCH/payload/freeldr.ini" || {
                echo "pi4-bench: could not flip DefaultOS to the UP entry" >&2; return 1; } ;;
        *) echo "pi4-bench: PI4_KERNEL must be up or mp" >&2; return 1 ;;
    esac
    cp "$img" "$BENCH/payload/ramdisk.img"
    # sanboot payload: the QEMU boot image (loader + ramdisk) with the Pi's
    # freeldr.ini, which iPXE serves to the board as a block device.
    cp "$REPO/qemu/boot.img" "$BENCH/payload/boot.img.tmp"
    mcopy -o -i "$BENCH/payload/boot.img.tmp" "$BENCH/payload/freeldr.ini" ::/freeldr.ini
    mv -f "$BENCH/payload/boot.img.tmp" "$BENCH/payload/boot.img"
    printf '#!ipxe\necho [ipxe] sanboot http://%s:%s/boot.img\ndhcp\nsanboot http://%s:%s/boot.img\n' \
        "$HOST_IP" "$HTTP_PORT" "$HOST_IP" "$HTTP_PORT" > "$BENCH/payload/autoexec.ipxe"
    # iPXE itself is upstream's prebuilt arm64 ipxe.efi (NOT snponly.efi, which
    # sees no NIC when launched from the card), kept in <bench>/ipxe/.
    if [ -f "$BENCH/ipxe/ipxe.efi" ]; then
        cp "$BENCH/ipxe/ipxe.efi" "$BENCH/payload/ipxe.efi"
    else
        echo "pi4-bench: no $BENCH/ipxe/ipxe.efi - --sanboot needs https://boot.ipxe.org/arm64-efi/ipxe.efi there" >&2
    fi
    python3 - "$BENCH/payload" "$REPO" <<'PY'
import hashlib, json, pathlib, subprocess, sys, time
payload, repo = pathlib.Path(sys.argv[1]), sys.argv[2]
def sha(p):
    h = hashlib.sha256()
    with open(p, 'rb') as f:
        for c in iter(lambda: f.read(1 << 20), b''):
            h.update(c)
    return h.hexdigest()
git = lambda *a: subprocess.check_output(['git', '-C', repo, *a], text=True).strip()
files = [{'path': r, 'bytes': (payload / r).stat().st_size, 'sha256': sha(payload / r)}
         for r in ('EFI/BOOT/BOOTAA64.EFI', 'freeldr.ini', 'ramdisk.img', 'boot.img', 'autoexec.ipxe')]
m = {'status': 'STAGED', 'kernel': __import__('os').environ.get('PI4_KERNEL', 'mp'), 'staged': time.strftime('%Y-%m-%dT%H:%M:%S%z'),
     'source_head': git('rev-parse', 'HEAD'),
     'source_dirty': bool(git('status', '--porcelain', '--untracked-files=no')),
     'files': files}
(payload / 'manifest.json').write_text(json.dumps(m, indent=2))
print(json.dumps(m, indent=2))
PY
}

. "$HERE/lease.sh"
case "${1:-status}" in stage|down|serial-restart) pi4_lease ;; esac

# A pico_serial.py whose stream has stalled still reports LISTENING/connected, and `up`
# keeps a live listener, so it needs replacing explicitly (netboot names this command).
serial_restart() {
    local p
    p=$(pico_serial_pid)
    if [ -n "$p" ]; then
        kill "$p" && echo "stopped pico_serial.py ($p)"
        for _ in 1 2 3 4 5; do [ -z "$(pico_serial_pid)" ] && break; sleep 1; done
    fi
    up
}

case "${1:-status}" in
    status) status ;;
    up) up ;;
    down) down ;;
    stage) stage ;;
    http) http ;;
    serial-restart) serial_restart ;;
    *) echo "usage: $0 status|up|down|stage|http|serial-restart" >&2; exit 2 ;;
esac
