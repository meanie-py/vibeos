#!/bin/bash
# play.sh — boot arm64 vibeos, run x86 Doom under WoW64/FEX, drive live keyboard
# input via QMP, and capture a screenshot sequence proving it renders AND responds.
#
# Prereqs:
#   - arm64 build present (./vibeos-cli build --arch arm64)
#   - tools/doom/doom-i386.exe + doom1.wad present (tools/doom/build.sh i386)
# The ramdisk is rebuilt with VIBEOS_FEX=1 so syswow64/doom.exe (the i386 PE) is
# staged. qemu/start.sh now attaches virtio kbd+mouse by default, so no extra QEMU
# args are needed for input.
#
# Output: /tmp/doom-play-{0..4}.png  (demo -> menu -> new game -> level -> action)
set -u
cd "$(dirname "$0")/../.." || exit 1
export RUNS_DIR=${RUNS_DIR:-/tmp/vbr}

echo "[*] rebuilding FEX ramdisk (WoW64 x86 Doom in syswow64)"
rm -f qemu/ramdisk.img qemu/boot.img
VIBEOS_FEX=1 bash qemu/mkramdisk.sh >/tmp/doom-mkramdisk.log 2>&1 \
    || { echo "mkramdisk FAILED"; tail -5 /tmp/doom-mkramdisk.log; exit 1; }

echo "[*] booting arm64 (start.sh attaches virtio kbd+mouse by default)"
RUN=$(VIBEOS_QEMU_MAX_LIFETIME=600 bash qemu/start.sh arm64 2>/dev/null)
echo "RUN=$RUN"
echo "[*] waiting for Doom + its render loop"
timeout 180 bash qemu/wait-for.sh "$RUN" --pattern "x86 DOOM under WoW64 created" --timeout 170 2>&1 | tail -1
timeout 160 bash qemu/wait-for.sh "$RUN" --pattern "NtGdiStretchDIBitsInternal(0x1124) enter" --timeout 150 2>&1 | tail -1
sleep 6

SOCK="$RUNS_DIR/$RUN/qmp.sock"
for _ in $(seq 1 20); do [ -S "$SOCK" ] && break; sleep 1; done
[ -S "$SOCK" ] || { echo "no QMP socket at $SOCK"; exit 1; }

python3 - "$SOCK" "$RUN" <<'PY'
import socket, json, sys, time
sock, run = sys.argv[1], sys.argv[2]
s=socket.socket(socket.AF_UNIX); s.connect(sock); f=s.makefile('rw')
def cmd(d):
    f.write(json.dumps(d)+'\n'); f.flush()
    while True:
        l=f.readline()
        if not l: return
        r=json.loads(l)
        if 'return' in r or 'error' in r: return r
f.readline(); cmd({"execute":"qmp_capabilities"})
def key(k): cmd({"execute":"send-key","arguments":{"keys":[{"type":"qcode","data":k}]}})
def shot(n): cmd({"execute":"screendump","arguments":{"filename":f"/tmp/vbr/{run}/{n}.ppm"}})
shot("play-0-demo")
key("esc"); time.sleep(1.8); shot("play-1-menu")      # esc TOGGLES the menu — send once
key("ret"); time.sleep(1.5); shot("play-2-newgame")   # New Game
key("ret"); time.sleep(3.5); shot("play-3-level")     # skill -> level loads
for _ in range(3): key("ctrl"); time.sleep(0.3)       # fire
for _ in range(8): key("up");   time.sleep(0.15)      # move forward
time.sleep(1.2); shot("play-4-action")
print("driven")
PY

for n in play-0-demo play-1-menu play-2-newgame play-3-level play-4-action; do
    convert "$RUNS_DIR/$RUN/$n.ppm" "/tmp/doom-$n.png" 2>/dev/null && echo "png: /tmp/doom-$n.png"
done
echo "RUN=$RUN done"
