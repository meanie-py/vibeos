#!/bin/bash
#
# Populate the QEMU boot disk image with ReactOS ARM64 files.
#
# Copies freeldr.ini, kernel (ntoskrnl.exe), HAL (hal.dll), KD transport
# (kdcom.dll), NLS codepage files, and the SYSTEM registry hive onto
# the FAT32 boot image so FreeLDR can load them.
#
# Usage: ./populate-disk.sh [--rebuild]
#

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build-arm64"
BOOT_IMG="$SCRIPT_DIR/boot.img"

# Parse arguments
for arg in "$@"; do
    case "$arg" in
        --rebuild)
            echo "[*] Rebuilding required targets..."
            ninja -C "$BUILD_DIR" ntoskrnl hal kdcom uefildr livecd_hives
            ninja -C "$BUILD_DIR" c_437.nls c_1252.nls 2>/dev/null || \
                ninja -C "$BUILD_DIR" media/nls/c_437.nls media/nls/c_1252.nls 2>/dev/null || true
            ;;
    esac
done

# Verify boot image exists
if [ ! -f "$BOOT_IMG" ]; then
    echo "[!] Boot image not found: $BOOT_IMG"
    exit 1
fi

# Source files
FREELDR_INI="$SCRIPT_DIR/freeldr.ini"
NTOSKRNL="$BUILD_DIR/ntoskrnl/ntoskrnl.exe"
HAL="$BUILD_DIR/hal/halarm64/hal.dll"
KDCOM="$BUILD_DIR/drivers/base/kdrosdbg/kdcom.dll"
SYSTEM_HIVE="$BUILD_DIR/boot/bootdata/system"
C_437="$BUILD_DIR/media/nls/c_437.nls"
C_1252="$BUILD_DIR/media/nls/c_1252.nls"
L_INTL="$PROJECT_DIR/reactos/media/nls/l_intl.nls"

# Verify all files exist
MISSING=0
for f in "$FREELDR_INI" "$NTOSKRNL" "$HAL" "$KDCOM" "$SYSTEM_HIVE" "$C_437" "$C_1252" "$L_INTL"; do
    if [ ! -f "$f" ]; then
        echo "[!] Missing: $f"
        MISSING=1
    fi
done
if [ "$MISSING" -eq 1 ]; then
    echo "[!] Build the missing files first: ninja -C $BUILD_DIR ntoskrnl hal kdcom livecd_hives"
    exit 1
fi

echo "[*] Populating boot image: $BOOT_IMG"

# Create directory structure on FAT image
# mmd returns error if dir exists, so ignore failures
mmd -i "$BOOT_IMG" ::/reactos 2>/dev/null || true
mmd -i "$BOOT_IMG" ::/reactos/system32 2>/dev/null || true
mmd -i "$BOOT_IMG" ::/reactos/system32/config 2>/dev/null || true

# Copy freeldr.ini to root
echo "  freeldr.ini -> /"
mcopy -o -i "$BOOT_IMG" "$FREELDR_INI" ::/freeldr.ini

# Copy kernel binaries to \reactos\system32
echo "  ntoskrnl.exe -> /reactos/system32/"
mcopy -o -i "$BOOT_IMG" "$NTOSKRNL" ::/reactos/system32/ntoskrnl.exe

echo "  hal.dll -> /reactos/system32/"
mcopy -o -i "$BOOT_IMG" "$HAL" ::/reactos/system32/hal.dll

echo "  kdcom.dll -> /reactos/system32/"
mcopy -o -i "$BOOT_IMG" "$KDCOM" ::/reactos/system32/kdcom.dll

# Copy NLS files
echo "  c_437.nls -> /reactos/system32/"
mcopy -o -i "$BOOT_IMG" "$C_437" ::/reactos/system32/c_437.nls

echo "  c_1252.nls -> /reactos/system32/"
mcopy -o -i "$BOOT_IMG" "$C_1252" ::/reactos/system32/c_1252.nls

echo "  l_intl.nls -> /reactos/system32/"
mcopy -o -i "$BOOT_IMG" "$L_INTL" ::/reactos/system32/l_intl.nls

# Copy SYSTEM registry hive
echo "  SYSTEM hive -> /reactos/system32/config/"
mcopy -o -i "$BOOT_IMG" "$SYSTEM_HIVE" ::/reactos/system32/config/system

echo "[*] Done. Disk contents:"
mdir -i "$BOOT_IMG" ::/
mdir -i "$BOOT_IMG" ::/reactos/system32/
mdir -i "$BOOT_IMG" ::/reactos/system32/config/
