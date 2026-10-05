#!/usr/bin/env bash
# Grab one frame of the Pi's HDMI output from the capture card.
#
#   tools/pi4/hdmi.sh [out.png]     # default: <bench>/hdmi-now.png
#
# The card is a DirectShow device ("USB Video") that only one program can hold.
# If the Windows Camera app has it open this fails - close Camera, don't fall
# back to a screen grab of its window: a gdigrab of that window came back black
# and was once nearly read as "the Pi shows nothing".
set -u

BENCH=${PI4_BENCH:-/mnt/d/home/vibeos-rpi4-setup}
OUT=${1:-$BENCH/hdmi-now.png}
FFMPEG=${PI4_FFMPEG:-$(ls -d /mnt/c/Users/*/AppData/Local/Microsoft/WinGet/Packages/Gyan.FFmpeg*/ffmpeg-*/bin/ffmpeg.exe 2>/dev/null | head -1)}
[ -n "$FFMPEG" ] || { echo "pi4-hdmi: ffmpeg.exe not found (winget install Gyan.FFmpeg, or set PI4_FFMPEG)" >&2; exit 1; }
# Both benches' cards call themselves "USB Video", so pick ours by its USB vendor
# id (Pi 4: 534d; tools/pi5/env.sh sets the Pi 5's). A name alone grabs whichever
# Windows lists first, which can be the other board's screen.
VID=${PI4_HDMI_VID:-534d}
DEVICE=${PI4_HDMI_DEVICE:-$("$FFMPEG" -hide_banner -list_devices true -f dshow -i dummy 2>&1 </dev/null |
    grep -o "@device_pnp_[^\"]*vid_${VID}[^\"]*" | head -1)}
[ -n "$DEVICE" ] || { echo "pi4-hdmi: no capture card with USB vendor id $VID is plugged in" >&2; exit 1; }

# ffmpeg.exe is a Windows program: hand it a Windows path, write via a temp name
# so a failed grab never leaves a stale frame looking current.
tmp="$OUT.tmp.png"
rm -f "$tmp"
# The card's first MJPEG frames after opening are often garbage; decode 30 and
# keep only the last (-update 1 overwrites the one output file).
"$FFMPEG" -hide_banner -loglevel error -f dshow -i "video=$DEVICE" \
    -frames:v 30 -update 1 -y "$(wslpath -w "$tmp")" </dev/null
if [ ! -s "$tmp" ]; then
    echo "pi4-hdmi: no frame captured - is the Camera app holding \"$DEVICE\"?" >&2
    exit 1
fi
mv -f "$tmp" "$OUT"
echo "$OUT"
