#!/usr/bin/env bash
# Put this tree's VibeOS on the Pi 5 and boot it once.
#
#   tools/pi5/stage.sh                # stage over SSH, then `reboot '0 tryboot'`
#   tools/pi5/stage.sh --no-reboot    # stage only
#   tools/pi5/stage.sh --freeldr-ini  # print the Pi 5's freeldr.ini (make-card.sh uses it)
#
# VIBEOS_LOADER_OPTIONS (e.g. "/STORTEST") is appended to every entry's Options=
# line for this staging only, so an opt-in diagnostic needs no edit to freeldr.ini.
#
# The board must be in its Linux stager (any normal boot; see card/config.txt).
# Every file is checked by SHA-256 on the board before the reboot, and the
# payload manifest records which commit went on, as tools/pi4/bench.sh stage does.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$HERE/../.." && pwd)
# shellcheck source=env.sh
. "$HERE/env.sh"

# The Pi 4's freeldr.ini, but the menu times out: nothing at the Pi 5 presses a
# key during an unattended boot. CRLF is kept.
freeldr_ini() {
    local extra=${VIBEOS_LOADER_OPTIONS:-}
    case "$extra" in
        *[\\\&\|]*) echo "pi5-stage: VIBEOS_LOADER_OPTIONS may not contain \\ & or |" >&2; exit 2 ;;
    esac
    sed -e 's/^TimeOut=-1\(\r\{0,1\}\)$/TimeOut=5\1/' \
        -e "s|^\\(Options=[^\r]*\\)\\(\r\\{0,1\\}\\)\$|\\1${extra:+ $extra}\\2|" \
        "$REPO/tools/pi4/freeldr.ini"
}

case "${1:-}" in
    --freeldr-ini) freeldr_ini; exit 0 ;;
    --no-reboot|"") ;;
    *) echo "usage: $0 [--no-reboot|--freeldr-ini]" >&2; exit 2 ;;
esac

log() { echo "pi5-stage: $*" >&2; }
efi=$REPO/build-arm64/boot/freeldr/freeldr/uefildr.efi
img=$REPO/qemu/ramdisk.img
for f in "$efi" "$img"; do
    [ -f "$f" ] || { log "missing $f - run ./vibeos-cli build --arch arm64"; exit 1; }
done

. "$REPO/tools/pi4/lease.sh"
pi4_lease

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/EFI/BOOT"
cp "$efi" "$work/EFI/BOOT/BOOTAA64.EFI"
cp "$img" "$work/ramdisk.img"
freeldr_ini > "$work/freeldr.ini"
cp "$HERE/card/config.txt" "$HERE/card/tryboot.txt" "$work/"
# UEFI's device tree goes in the partition root under the default name, as on a
# UEFI-only card; the stager's own lives in linux/ (config.txt's os_prefix).
uefi_zip=$PI4_BENCH/cache/rpi5-uefi-v0.3.zip
[ -f "$uefi_zip" ] || { log "missing $uefi_zip - run tools/pi5/make-card.sh once"; exit 1; }
unzip -p "$uefi_zip" bcm2712-rpi-5-b.dtb > "$work/bcm2712-rpi-5-b.dtb"
files=(EFI/BOOT/BOOTAA64.EFI freeldr.ini ramdisk.img config.txt tryboot.txt bcm2712-rpi-5-b.dtb)
(cd "$work" && sha256sum "${files[@]}") > "$work/SHA256SUMS"

pi5_ssh true || { log "no SSH to $PI5_SSH_TARGET - is the board up in its Linux stager? (vibeos-cli pi5 doctor)"; exit 1; }
log "copying ${#files[@]} files to $PI5_SSH_TARGET"
pi5_ssh 'rm -rf /tmp/vibeos-stage && mkdir -p /tmp/vibeos-stage/EFI/BOOT'
(cd "$work" && tar cf - "${files[@]}" SHA256SUMS) | pi5_ssh 'tar xf - -C /tmp/vibeos-stage'
# Verify the staged copy, install, then verify what is on the card itself.
pi5_ssh 'set -e
cd /tmp/vibeos-stage && sha256sum -c --quiet SHA256SUMS
cd /boot/firmware
# One-time move of the stager into linux/ (cards built before the os_prefix
# layout carry it in the root). The kernel still in the root is what says so.
if [ -f kernel_2712.img ]; then
    sudo mkdir -p linux
    sudo mv -f kernel_2712.img linux/
    [ -f initramfs_2712 ] && sudo mv -f initramfs_2712 linux/
    sudo mv -f bcm2712-rpi-5-b.dtb linux/
    sudo cp cmdline.txt linux/cmdline.txt
    sudo rm -f rpi5-uefi.dtb
fi
# config.txt sends the next normal boot to linux/: never install it unless the
# stager is complete there.
for f in kernel_2712.img initramfs_2712 bcm2712-rpi-5-b.dtb cmdline.txt; do
    [ -s "linux/$f" ] || { echo "linux/$f missing: not installing config.txt" >&2; exit 1; }
done
cd /tmp/vibeos-stage
for f in EFI/BOOT/BOOTAA64.EFI freeldr.ini ramdisk.img tryboot.txt bcm2712-rpi-5-b.dtb config.txt; do
    sudo install -D -m 0755 "$f" "/boot/firmware/$f"
done
sync
cd /boot/firmware && sha256sum -c --quiet /tmp/vibeos-stage/SHA256SUMS'

commit=$(git -C "$REPO" rev-parse --short=12 HEAD)
dirty=$(git -C "$REPO" status --porcelain --untracked-files=no | head -c1)
mkdir -p "$PI4_BENCH"
python3 - "$work/SHA256SUMS" "$PI4_BENCH/payload.json" "$commit" "${dirty:+dirty}" <<'PY'
import json, sys, time
sums, out, commit, dirty = sys.argv[1:5]
files = [{'sha256': h, 'path': p} for h, p in (l.split() for l in open(sums))]
json.dump({'commit': commit, 'dirty': bool(dirty), 'staged': time.strftime('%Y-%m-%dT%H:%M:%S%z'),
           'files': files}, open(out, 'w'), indent=1)
PY
log "staged $commit${dirty:+ (dirty tree)}; card verified"

[ "${1:-}" = --no-reboot ] && exit 0
# `reboot '0 tryboot'` boots tryboot.txt instead of config.txt, once. The
# SSH session dies with the reboot, so its exit status means nothing here.
log "rebooting into UEFI + VibeOS (tryboot)"
pi5_ssh "sudo systemd-run --on-active=1 /usr/sbin/reboot '0 tryboot'" >/dev/null 2>&1 || true
