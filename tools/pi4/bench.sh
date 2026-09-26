#!/usr/bin/env bash
# Host side of the Pi 4B bench, driven from WSL. Guide: docs/engineering/pi4-bench.md
#
#   tools/pi4/bench.sh status
#   tools/pi4/bench.sh up       # serial capture listener + TFTP server
#   tools/pi4/bench.sh down
#   tools/pi4/bench.sh stage    # copy the current arm64 build into the TFTP payload
#
# Both services run as WINDOWS processes: the USB-serial port belongs to Windows,
# and WSL2 in NAT mode (172.x) is unreachable from the LAN the Pi is on. `up`
# first copies the repo's versions of the scripts into the bench directory, so
# the repo stays the source of truth and Windows never runs a script over a
# \\wsl.localhost path (execution policy and UNC quirks).
set -u

HERE=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$HERE/../.." && pwd)
BENCH=${PI4_BENCH:-/mnt/d/home/vibeos-rpi4-setup}
PORT=${PI4_COM:-COM5}
PS=/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe
# Windows python (the Microsoft Store stub does not count).
PY=${PI4_WINPY:-'C:\Users\Lambda\AppData\Local\Programs\Python\Python313\python.exe'}

WBENCH=$(wslpath -w "$BENCH")
ps_run() { "$PS" -NoProfile -Command "$1" 2>&1 | tr -d '\r'; }
# Start-Process children inherit powershell's stdout; if that is a pipe, the
# pipe never closes and the caller hangs until the service exits. Detach fully.
ps_launch() { "$PS" -NoProfile -Command "$1" >/dev/null 2>&1 </dev/null; }

# PIDs by command line, so we never touch an unrelated powershell/python.
pids() {
    ps_run "Get-CimInstance Win32_Process | Where-Object { \$_.CommandLine -like '*$1*' -and \$_.CommandLine -notlike '*Get-CimInstance*' } | ForEach-Object { \$_.ProcessId }"
}

status() {
    local s t
    s=$(pids 'Capture-RpiSerial.ps1' | xargs)
    t=$(pids 'serve_tftp.py' | xargs)
    printf 'serial listener : %s\n' "${s:-down}"
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
    printf 'power           : %s\n' "$("$HERE/power.sh" status 2>&1)"
}

up() {
    mkdir -p "$BENCH/payload"
    cp "$HERE/Capture-RpiSerial.ps1" "$HERE/serve_tftp.py" "$BENCH/"
    if [ -z "$(pids 'Capture-RpiSerial.ps1')" ]; then
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
        echo "pi4-bench: serial listener is not LISTENING ($PORT held by another program?)" >&2
        return 1
    }
    [ -n "$(pids 'serve_tftp.py')" ] || {
        echo "pi4-bench: TFTP server exited - see $BENCH/tftp-server.err" >&2
        return 1
    }
}

down() {
    for pat in 'Capture-RpiSerial.ps1' 'serve_tftp.py'; do
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
    for f in "$efi" "$img"; do
        [ -f "$f" ] || { echo "pi4-bench: missing $f - run ./vibeos-cli build --arch arm64" >&2; return 1; }
    done
    mkdir -p "$BENCH/payload/EFI/BOOT"
    cp "$efi" "$BENCH/payload/EFI/BOOT/BOOTAA64.EFI"
    cp "$HERE/freeldr.ini" "$BENCH/payload/freeldr.ini"
    cp "$img" "$BENCH/payload/ramdisk.img"
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
         for r in ('EFI/BOOT/BOOTAA64.EFI', 'freeldr.ini', 'ramdisk.img')]
m = {'status': 'STAGED', 'staged': time.strftime('%Y-%m-%dT%H:%M:%S%z'),
     'source_head': git('rev-parse', 'HEAD'),
     'source_dirty': bool(git('status', '--porcelain', '--untracked-files=no')),
     'files': files}
(payload / 'manifest.json').write_text(json.dumps(m, indent=2))
print(json.dumps(m, indent=2))
PY
}

case "${1:-status}" in
    status) status ;;
    up) up ;;
    down) down ;;
    stage) stage ;;
    *) echo "usage: $0 status|up|down|stage" >&2; exit 2 ;;
esac
