#!/usr/bin/env bash
# Stage this tree's amd64 livecd for the OptiPlex's network boot.
#
# The 9020 boots GRUB -> iPXE -> `sanboot http://<host>/vibeos-amd64.iso`
# (docs/engineering/optiplex-bench.md). iPXE's SAN disk is an INT 13h device,
# which freeldr can read and the kernel cannot see, so the plain livecd loads
# its kernel and then has no boot volume. This builds a RAM-disk image instead:
# the livecd's loader plus the whole livecd.iso, with a freeldr.ini whose
# default entry reads `livecd\livecd.iso` into memory and hands it to the kernel
# as a CD (/RDPATH ... /RDEXPORTASCD, as hybridcd.ini does).
#
# `stage` also writes the iPXE script the board chains (optiplex.ipxe).
#
# Keep sector 0 of the image blank (no isohybrid MBR): iPXE's SAN drive takes
# BIOS drive 80h, and isoboot.bin reads 80h's MBR; finding a signature there it
# offers "Press any key to boot from the ReactOS medium" and, unanswered, boots
# that disk instead. (memdisk's CD emulation leaves the NVMe at 80h, so it hits
# exactly that; see optiplex-bench.md.)
#
#   tools/optiplex/netboot.sh [stage]   build it and copy it to the payload dir
#   tools/optiplex/netboot.sh build     build it only (prints the path)
#
# OPTIPLEX_TRIPWIRE=<cfg> arms the kernel's tripwire for that image, as smoke
# --tripwire does in QEMU, with commas between tokens: SPECIALPOOL=*,QUIET.
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
. "$HERE/env.sh"
ROOT=$(cd "$HERE/../.." && pwd)
LIVECD=${OPTIPLEX_LIVECD:-$ROOT/build-amd64/livecd.iso}
OUT=${OPTIPLEX_NETBOOT_ISO:-$ROOT/build-amd64/optiplex-netboot.iso}

build() {
    [ -f "$LIVECD" ] || { echo "netboot: no $LIVECD (./vibeos-cli build --arch amd64)" >&2; exit 1; }
    local work
    work=$(mktemp -d "${TMPDIR:-/tmp}/optiplex-netboot.XXXXXX")
    trap 'rm -rf "$work"' RETURN
    # Same kernel/HAL pair and debug port as the livecd's default entry (LiveCD_SMP).
    local tripwire=""
    [ -n "${OPTIPLEX_TRIPWIRE:-}" ] && tripwire=" /TRIPWIRE=$OPTIPLEX_TRIPWIRE"
    cat > "$work/freeldr.ini" <<EOF
[FREELOADER]
DefaultOS=LiveCD_RamDisk_SMP
TimeOut=0
Debug=DEBUGPORT=COM1 BAUDRATE=115200

[Display]
TitleText=VibeOS netboot (RAM disk)
MinimalUI=Yes

[Operating Systems]
LiveCD_RamDisk_SMP="LiveCD in RAM, SMP kernel (Debug)"

[LiveCD_RamDisk_SMP]
BootType=Windows2003
SystemPath=ramdisk(0)\reactos
Options=/KERNEL=ntkrnlmp.exe /HAL=halmp.dll /DEBUG /DEBUGPORT=COM1 /BAUDRATE=115200 /SOS /FASTDETECT /MININT /RDPATH=livecd\\livecd.iso /RDEXPORTASCD$tripwire
EOF
    xorriso -osirrox on -indev "$LIVECD" -extract /loader "$work/loader" >/dev/null 2>&1
    chmod -R u+w "$work/loader"
    rm -f "$OUT"
    xorriso -outdev "$OUT" -volid VIBEOS_NETBOOT \
        -map "$work/loader" /loader \
        -map "$work/freeldr.ini" /freeldr.ini \
        -map "$LIVECD" /livecd/livecd.iso \
        -boot_image any bin_path=/loader/isoboot.bin \
        -boot_image any emul_type=no_emulation \
        -boot_image any load_size=2048 \
        >/dev/null 2>&1
    [ -s "$OUT" ] || { echo "netboot: xorriso wrote no $OUT" >&2; exit 1; }
    echo "$OUT"
}

# Copy one file into the payload dir, check it arrived intact, print its sha256.
put() {
    local src=$1 dest="$OPTIPLEX_PAYLOAD/$2" want got
    # dd in 16 KB blocks: under host memory pressure drvfs fails a large
    # write with ENOMEM (`cp` of the 400 MB image; dd at 4M, 256K and 64K,
    # 2026-10-10), and the PowerShell Copy-Item tried before dies whenever WSL
    # interop's vsock times out. Written beside the target and renamed, so the
    # HTTP server never serves a half-copied image.
    dd if="$src" of="$dest.part" bs=16K status=none ||
        { echo "netboot: copying $2 to the payload dir failed" >&2; exit 1; }
    mv -f "$dest.part" "$dest"
    want=$(sha256sum "$src" | cut -d' ' -f1)
    # Reading it back can hit the same transient ENOMEM; that is not a mismatch.
    for _ in 1 2 3; do
        got=$(sha256sum "$dest" 2>/dev/null | cut -d' ' -f1) && [ -n "$got" ] && break
        sleep 2
    done
    [ "$want" = "$got" ] || { echo "netboot: staged $2 differs ($got vs $want)" >&2; exit 1; }
    echo "$want"
}

case "${1:-stage}" in
    build)
        build
        ;;
    stage)
        iso=$(build)
        script=$(mktemp "${TMPDIR:-/tmp}/optiplex-ipxe.XXXXXX")
        # Relative URLs resolve against the script's own, so the host is named
        # only in the board's embedded chain script. That script re-chains this
        # one on failure, and images a failed attempt fetched stay registered:
        # free them first so a retry repeats the same attempt, and hold the
        # error on screen for a minute.
        printf '%s\n' '#!ipxe' \
            'imgfree' \
            'echo [ipxe] optiplex.ipxe: sanboot the VibeOS ISO' \
            'sanboot vibeos-amd64.iso || goto failed' \
            ':failed' \
            'echo [ipxe] optiplex.ipxe failed: ${errno}' \
            'imgfree' \
            'sleep 60' \
            'exit 1' > "$script"
        put "$script" optiplex.ipxe >/dev/null
        rm -f "$script"
        sum=$(put "$iso" vibeos-amd64.iso)
        echo "staged $(du -h "$iso" | cut -f1) RAM-disk image as $OPTIPLEX_PAYLOAD/vibeos-amd64.iso ($sum), with optiplex.ipxe"
        ;;
    *)
        echo "usage: $0 [stage|build]" >&2
        exit 2
        ;;
esac
