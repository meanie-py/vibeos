#!/usr/bin/env bash
# Drive a Pi 4B bring-up cycle end to end, with nobody at the bench.
#
#   tools/pi4/netboot.sh [--no-power]                       # sanboot (default, ~100 s to desktop)
#   tools/pi4/netboot.sh --card [--no-launch] [--ramdisk]   # old path: TFTP onto the SD card
#   tools/pi4/netboot.sh --shell                            # stop at the UEFI Shell prompt; drive it
#                                                           # with tools/pi4/serial-send.sh
#
# Power-cycles the board through the bench's smart plug (power.sh), catches the
# firmware splash (a ~5 second window that is easy to miss by hand -
# missing it means sitting through ~90s of PXE/HTTP timeouts and then booting
# whatever stale loader is on the card), enters the UEFI Shell, brings up DHCP,
# TFTPs the freshly built loader onto the card, and launches it.
#
# The default is sanboot: only iPXE touches the card, and iPXE sanboots
# payload/boot.img over HTTP (bench.sh http), so a kernel change costs ~100 s
# power-on to desktop instead of the ~23 minutes of card write + read that
# --card takes. Needs the TCP 8069 rule in docs/engineering/pi4-bench.md.
#
# Requires `tools/pi4/bench.sh up` (serial listener + TFTP server) and a payload
# staged with `tools/pi4/bench.sh stage`. Guide: docs/engineering/pi4-bench.md
set -u

BENCH=${PI4_BENCH:-/mnt/d/home/vibeos-rpi4-setup}
HOST_IP=${PI4_HOST_IP:-192.168.1.183}
# RFC 7440 window: blocks sent per ACK. 1 = classic lockstep TFTP.
TFTP_WINDOW=${PI4_TFTP_WINDOW:-32}
TFTP_BLKSIZE=${PI4_TFTP_BLKSIZE:-1468}
SEND="$(dirname "$0")/serial-send.sh"
LAUNCH=1
POWER=1
RAMDISK=0
SANBOOT=1
SHELL_ONLY=0
for arg in "$@"; do
    case "$arg" in
        --no-launch) LAUNCH=0 ;;
        # Leave power alone and wait for someone to cycle the board by hand.
        --no-power) POWER=0 ;;
        # Only needed when kernel-side sources changed: the loader is 300 KB and
        # takes seconds, the ramdisk is 140 MB and takes ~13.5 minutes (card writes).
        --ramdisk) RAMDISK=1; SANBOOT=0 ;;
        --card) SANBOOT=0 ;;
        --sanboot) SANBOOT=1 ;;
        # Power-cycle into the firmware's UEFI Shell and stop there: for reading
        # bench media (the card, a USB stick) with a second FAT implementation.
        --shell) SHELL_ONLY=1 ;;
        *) echo "usage: $0 [--no-power] | --card [--no-power] [--no-launch] [--ramdisk] | --shell [--no-power]" >&2; exit 2 ;;
    esac
done

log() { printf '[pi4] %s\n' "$*" >&2; }

serial_file() {
    python3 - "$BENCH" <<'PY'
import json, sys, pathlib
bench = pathlib.Path(sys.argv[1])
state = json.loads((bench / 'serial-current.json').read_text())
if state.get('status') != 'LISTENING':
    sys.exit('capture listener is not LISTENING: ' + str(state.get('status')))
d = state['directory']
# Capture-RpiSerial.ps1 records a Windows path; pico_serial.py a WSL one.
if not d.startswith('/'):
    d = d.replace('D:\\home', '/mnt/d/home').replace('\\', '/')
print(pathlib.Path(d) / 'serial.bin')
PY
}

F=$(serial_file) || exit 1
log "watching $F"

# Wait for PATTERN to appear at/after byte OFFSET. Echoes nothing; returns 1 on
# timeout so the caller can report which step actually stalled.
wait_for() {
    local pattern=$1 offset=$2 timeout=${3:-120} waited=0
    while [ "$waited" -lt "$timeout" ]; do
        if tail -c "+$((offset + 1))" "$F" 2>/dev/null | strings | grep -qF "$pattern"; then
            return 0
        fi
        sleep 2
        waited=$((waited + 2))
    done
    return 1
}

size() { stat -c %s "$F" 2>/dev/null || echo 0; }

start=$(size)
if [ "$POWER" -eq 1 ]; then
    log "power-cycling the board"
    "$(dirname "$0")/power.sh" cycle 5 >/dev/null || exit 1
else
    log "waiting for the firmware splash (power-cycle the Pi now)"
fi
if ! wait_for 'ENTER (boot)' "$start" "${PI4_SPLASH_TIMEOUT:-180}"; then
    log "no splash seen in ${PI4_SPLASH_TIMEOUT:-180}s - is the Pi powered on?"
    exit 1
fi

# F1 = UEFI Shell. VT100 PF1; the firmware's terminal maps ESC O P.
log "splash seen, requesting the UEFI Shell"
mark=$(size)
"$SEND" --raw $'\033OP'
if ! wait_for 'Shell>' "$mark" 90; then
    log "never reached the Shell prompt"
    exit 1
fi

# The SD card's filesystem mapping. Not fs0: a USB stick with a FAT partition
# (a Ventoy stick's VTOYEFI) maps ahead of the card, and fs0 then wrote the
# loader onto the stick. The Shell prints its mapping table before the prompt;
# the card is the one FS whose device path does not go through USB(...).
card_fs() {
    tail -c "+$((1 + $1))" "$F" | python3 -c '
import re, sys
text = re.sub(r"\x1b\[[0-9;]*[A-Za-z]", "", sys.stdin.buffer.read().decode("latin1"))
table = text[text.rfind("Mapping table"):]
fs = {}
cur = None
for line in table.splitlines():
    m = re.match(r"\s*(FS\d+):", line)
    if m:
        cur = m.group(1)
        fs[cur] = ""
    elif cur and line.startswith(" " * 10):
        fs[cur] += line.strip()
    elif not line.strip() or re.match(r"\s*BLK\d+:", line):
        cur = None
card = [k for k, path in fs.items() if path and "USB(" not in path]
if len(card) != 1:
    sys.exit("card filesystem: want exactly one non-USB FS, found %r" % fs)
print(card[0].lower())
'
}
CARD=$(card_fs "$mark") || { log "cannot tell which Shell filesystem is the SD card; refusing to write"; exit 1; }
log "SD card is $CARD: in the Shell mapping table"
if [ "$SHELL_ONLY" -eq 1 ]; then
    log "at the Shell prompt (--shell); send commands with tools/pi4/serial-send.sh"
    exit 0
fi

log "in the Shell, bringing up DHCP on eth0"
mark=$(size)
"$SEND" 'ifconfig -s eth0 dhcp'
sleep 15
mark=$(size)
"$SEND" 'ifconfig -l eth0'
if ! wait_for '192.168.' "$mark" 30; then
    log "eth0 did not get a DHCP address"
    exit 1
fi

# tftp_to_card REMOTE CARDPATH [TIMEOUT]
tftp_to_card() {
    local mark
    mark=$(size)
    "$SEND" "tftp -s $TFTP_BLKSIZE -w $TFTP_WINDOW $HOST_IP $1 $2"
    if ! wait_for 'Shell>' "$mark" "${3:-120}"; then
        log "TFTP of $1 did not finish"
        exit 1
    fi
    if tail -c "+$((mark + 1))" "$F" | strings | grep -qF 'TFTP Error'; then
        log "TFTP of $1 reported an error - is serve_tftp.py running (bench.sh status) and the firewall rule present?"
        exit 1
    fi
}

if [ "$SANBOOT" -eq 1 ]; then
    log "TFTP: ipxe.efi + autoexec.ipxe onto the card"
    tftp_to_card ipxe.efi "$CARD:\\ipxe.efi"
    tftp_to_card autoexec.ipxe "$CARD:\\autoexec.ipxe"
    log "launching iPXE (sanboot http://$HOST_IP:${PI4_HTTP_PORT:-8069}/boot.img)"
    mark=$(size)
    "$SEND" "$CARD:\\ipxe.efi"
    if ! wait_for 'Switching stack' "$mark" 300; then
        log "loader never started - see the capture for iPXE's error (HTTP rule? bench.sh http?)"
        exit 1
    fi
    log "loader at its menu; selecting the default entry"
    "$SEND" --raw $'\r'
    log "booting - watch the capture for [FLDR]/[KERNEL] output"
    exit 0
fi

log "TFTP: BOOTAA64.EFI from $HOST_IP onto the card"
mark=$(size)
"$SEND" "tftp -s $TFTP_BLKSIZE -w $TFTP_WINDOW $HOST_IP EFI/BOOT/BOOTAA64.EFI $CARD:\\EFI\\BOOT\\BOOTAA64.EFI"
if ! wait_for 'Shell>' "$mark" 120; then
    log "TFTP did not finish"
    exit 1
fi
if tail -c "+$((mark + 1))" "$F" | strings | grep -qF 'TFTP Error'; then
    log "TFTP reported an error - is serve_tftp.py running (bench.sh status) and the firewall rule present?"
    exit 1
fi

if [ "$RAMDISK" -eq 1 ]; then
    log "TFTP: ramdisk.img (140 MB, ~13.5 min - the SD card write is the limit)"
    mark=$(size)
    "$SEND" "tftp -s $TFTP_BLKSIZE -w $TFTP_WINDOW $HOST_IP ramdisk.img $CARD:\\ramdisk.img"
    if ! wait_for 'Shell>' "$mark" 900; then
        log "ramdisk TFTP did not finish"
        exit 1
    fi
    if tail -c "+$((mark + 1))" "$F" | strings | grep -qF 'TFTP Error'; then
        log "ramdisk TFTP reported an error"
        exit 1
    fi
fi

if [ "$LAUNCH" -eq 0 ]; then
    log "loader updated; stopping at the Shell as asked"
    exit 0
fi

log "launching the loader"
mark=$(size)
"$SEND" "$CARD:\\EFI\\BOOT\\BOOTAA64.EFI"
if ! wait_for 'Switching stack' "$mark" 120; then
    log "loader did not reach its menu"
    exit 1
fi

# freeldr.ini on this card uses TimeOut=-1, so the menu waits for a key.
log "loader at its menu; selecting the default entry"
"$SEND" --raw $'\r'
log "booting - watch the capture for [FLDR]/[KERNEL] output"
