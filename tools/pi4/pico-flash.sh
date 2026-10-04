#!/usr/bin/env bash
# Update the Pico 2 W's firmware with nobody at the bench.
#
#   tools/pi4/pico-flash.sh [picousb.uf2]   # default: picousb's build/firmware/picousb.uf2
#
# The Pico's USB is plugged into the Pi, so `pico.sh bootsel` makes its BOOTSEL
# volume appear on the PI. The Pi's UEFI Shell does the copy: netboot.sh
# --pico-flash TFTPs the UF2 and picoflash.nsh onto the SD card, starts the
# script (it waits, rescans USB and copies onto the volume holding
# INFO_UF2.TXT) and then reboots the Pico into BOOTSEL over Wi-Fi. The Pico
# flashes itself, reboots into the new firmware and rejoins Wi-Fi; this script
# waits for that and checks its uptime restarted.
#
# EXPERIMENTAL, ATTENDED ONLY. If the copy fails the Pico waits in BOOTSEL,
# and with VSYS feeding it a Pi power cycle does not reset it: someone has to
# pull its power. On 2026-10-03 a failed copy (fixed since: volumes are now
# found by content) was followed by a power-up straight into BOOTSEL, i.e.
# possibly an unbootable flash. Hence PI4_PICO_FLASH_ATTENDED=1 is required.
# The safe path stays the PC: Pico USB on the host, `pico-devbox bootsel`,
# copy the stripped UF2 to the RP2350 drive.
#
# Only public firmware UF2s: never a config or provisioned image. picousb's
# firmware_uf2.py check refuses one that would reach the config sector (a
# drag-and-drop of picotool's unstripped UF2 deprovisions the board).
set -u
HERE=$(cd "$(dirname "$0")" && pwd)
BENCH=${PI4_BENCH:-/mnt/d/home/vibeos-rpi4-setup}
PICOUSB=${PICOUSB_REPO:-/mnt/d/home/Projects/picousb}
UF2=${1:-$PICOUSB/build/firmware/picousb.uf2}
if [ "${PI4_PICO_FLASH_ATTENDED:-0}" != 1 ]; then
    echo "pico-flash: attended only - a failed copy leaves the Pico in BOOTSEL until someone" >&2
    echo "pulls its power. Set PI4_PICO_FLASH_ATTENDED=1 with someone at the bench." >&2
    exit 2
fi
. "$HERE/lease.sh"
pi4_lease

log() { printf '[pico-flash] %s\n' "$*" >&2; }

[ -f "$UF2" ] || { log "no UF2 at $UF2 (build it: bash tools/build.sh in picousb)"; exit 1; }
python3 "$PICOUSB/tools/firmware_uf2.py" check "$UF2" || exit 1

before=$("$HERE/pico.sh" usb) || { log "the Pico is not answering; nothing to update"; exit 1; }
log "before: $before"
up_before=$(printf '%s' "$before" | sed -n 's/^uptime=\([0-9]*\)s.*/\1/p')

mkdir -p "$BENCH/payload"
cp "$UF2" "$BENCH/payload/pico.uf2"
# CRLF: the Shell's script parser is DOS-flavoured.
sed 's/$/\r/' > "$BENCH/payload/picoflash.nsh" <<'NSH'
@echo -off
# Written by vibeos tools/pi4/pico-flash.sh. Volumes are found by CONTENT after
# every rescan: `map -r` renumbers them once the Pico's BOOTSEL volume appears
# (2026-10-03 the card went fs1 -> fs2 and a USB stick took fs1).
for %t run (1 8)
  stall 5000000
  connect -r
  map -r
  for %a in fs0 fs1 fs2 fs3 fs4 fs5 fs6 fs7
    if exist %a:\INFO_UF2.TXT then
      for %b in fs0 fs1 fs2 fs3 fs4 fs5 fs6 fs7
        if exist %b:\picoflash.nsh then
          echo [pico-flash] copying %b:\pico.uf2 to the BOOTSEL volume %a:
          cp -q %b:\pico.uf2 %a:\pico.uf2
          goto finished
        endif
      endfor
    endif
  endfor
endfor
echo [pico-flash] no BOOTSEL volume appeared
:finished
NSH

"$HERE/netboot.sh" --pico-flash || exit 1

log "waiting for the Pico to come back on its new firmware"
for _ in $(seq 1 60); do
    sleep 5
    if after=$("$HERE/pico.sh" usb 2>/dev/null); then
        up=$(printf '%s' "$after" | sed -n 's/^uptime=\([0-9]*\)s.*/\1/p')
        # Its uptime restarting is the proof it rebooted, i.e. the copy finished.
        if [ -n "$up" ] && [ "$up" -lt "${up_before:-0}" ]; then
            log "after: $after"
            exit 0
        fi
    fi
done
log "the Pico did not come back within 5 minutes."
log "It is most likely still in BOOTSEL on the Pi (its flash is untouched until a copy"
log "completes). The HDMI frame shows the Shell's side: tools/pi4/hdmi.sh. To free it,"
log "remove the Pico's power - VSYS and USB - for a moment; a Pi power cycle is not enough."
exit 1
