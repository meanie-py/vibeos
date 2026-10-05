#!/usr/bin/env bash
# Assemble the Pi 5 bench card's boot partition (one time, or to rebuild a card).
#
#   tools/pi5/make-card.sh <out-dir> <partuuid>   # partuuid: the card's MBR id, e.g. 8b744144
#
# The card is an existing Raspberry Pi OS install (its ext4 root, partition 2, is
# kept). This builds the FAT partition that boots it as the bench's Linux stager
# and, under tryboot, UEFI + VibeOS: see card/config.txt and card/tryboot.txt. Copy the result onto the
# card with tools/pi5/Deploy-Card.ps1. After that, `vibeos-cli pi5 boot` keeps the
# VibeOS files current over SSH and the card never has to leave the board.
#
# Inputs are pinned by SHA-256 and cached under <bench>/cache:
#   - Raspberry Pi OS Lite 2025-12-04: kernel_2712.img, initramfs_2712 and the
#     Pi 5 device tree. Its kernel must match the root filesystem's modules; the
#     bench card was installed from this release (same kernel_2712.img).
#   - worproject rpi5-uefi v0.3 (archived; its last release).
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$HERE/../.." && pwd)
OUT=${1:?usage: make-card.sh <out-dir> <partuuid>}
PARTUUID=${2:?usage: make-card.sh <out-dir> <partuuid>}
BENCH=${PI5_BENCH:-/mnt/d/home/vibeos-rpi5-setup}
CACHE=$BENCH/cache
KEY=${PI5_SSH_PUBKEY:-$HOME/.ssh/id_ed25519.pub}

PIOS_URL=https://downloads.raspberrypi.com/raspios_lite_arm64/images/raspios_lite_arm64-2025-12-04/2025-12-04-raspios-trixie-arm64-lite.img.xz
PIOS_SHA=$(curl -sfL "$PIOS_URL.sha256" | awk '{print $1}') || true
UEFI_URL=https://github.com/worproject/rpi5-uefi/releases/download/v0.3/RPi5_UEFI_Release_v0.3.zip
UEFI_SHA=2337df318845448ebcc8a6a587d15dccec44b54c665ece21250d2f6170732724

die() { echo "pi5-card: $*" >&2; exit 1; }
[[ $PARTUUID =~ ^[0-9a-f]{8}$ ]] || die "partuuid must be 8 hex digits (the card's MBR disk id)"
[ -f "$KEY" ] || die "no SSH public key at $KEY (set PI5_SSH_PUBKEY)"
for f in build-arm64/boot/freeldr/freeldr/uefildr.efi qemu/ramdisk.img; do
    [ -f "$REPO/$f" ] || die "missing $f - run ./vibeos-cli build --arch arm64"
done

fetch() { # url sha file
    if [ ! -f "$3" ] || ! echo "$2  $3" | sha256sum -c --quiet 2>/dev/null; then
        curl -sfLo "$3.part" "$1" || die "download failed: $1"
        mv "$3.part" "$3"
    fi
    echo "$2  $3" | sha256sum -c --quiet || die "checksum mismatch: $3"
}

mkdir -p "$CACHE" "$OUT"
[ -n "$PIOS_SHA" ] || PIOS_SHA=$(cat "$CACHE/pios.sha256" 2>/dev/null) || die "cannot fetch the Pi OS checksum"
echo "$PIOS_SHA" > "$CACHE/pios.sha256"
fetch "$PIOS_URL" "$PIOS_SHA" "$CACHE/pios.img.xz"
fetch "$UEFI_URL" "$UEFI_SHA" "$CACHE/rpi5-uefi-v0.3.zip"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
xz -dc "$CACHE/pios.img.xz" > "$work/pios.img"
# The boot partition is the first MBR entry; mtools reads it in place.
start=$(python3 -c "import struct,sys; d=open(sys.argv[1],'rb').read(512); print(struct.unpack('<I', d[454:458])[0] * 512)" "$work/pios.img")
for f in kernel_2712.img initramfs_2712 bcm2712-rpi-5-b.dtb; do
    MTOOLS_SKIP_CHECK=1 mcopy -n -i "$work/pios.img@@$start" "::/$f" "$OUT/$f"
done
unzip -oq "$CACHE/rpi5-uefi-v0.3.zip" RPI_EFI.fd bcm2712-rpi-5-b.dtb -d "$work/uefi"
cp "$work/uefi/RPI_EFI.fd" "$OUT/"
cp "$work/uefi/bcm2712-rpi-5-b.dtb" "$OUT/rpi5-uefi.dtb"

cp "$HERE/card/config.txt" "$HERE/card/tryboot.txt" "$HERE/card/meta-data" "$HERE/card/network-config" "$OUT/"
sed "s/@PARTUUID@/$PARTUUID/" "$HERE/card/cmdline.txt" > "$OUT/cmdline.txt"
pub=$(cat "$KEY")
sed "s|@SSH_PUBKEY@|$pub|" "$HERE/card/user-data.in" > "$OUT/user-data"

mkdir -p "$OUT/EFI/BOOT"
cp "$REPO/build-arm64/boot/freeldr/freeldr/uefildr.efi" "$OUT/EFI/BOOT/BOOTAA64.EFI"
bash "$HERE/stage.sh" --freeldr-ini > "$OUT/freeldr.ini"
cp "$REPO/qemu/ramdisk.img" "$OUT/ramdisk.img"

(cd "$OUT" && find . -type f ! -name card.sha256 -exec sha256sum {} + | sort -k2 > card.sha256)
du -sh "$OUT"
echo "card tree: $OUT (deploy with tools/pi5/Deploy-Card.ps1)"
