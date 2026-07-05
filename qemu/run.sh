#!/bin/bash
#
# ReactOS QEMU launcher (arm64, i386, amd64)
#
# Usage:
#   ./run.sh                      # ARM64 boot (serial output to terminal)
#   ./run.sh --arch i386          # i386 boot from bootcd.iso
#   ./run.sh --arch amd64         # amd64 boot from bootcd.iso
#   ./run.sh --gui                # GUI mode: QEMU window + virtio mouse/keyboard
#   ./run.sh --livecd             # Boot from livecd.iso (desktop, no install)
#   ./run.sh --debug              # Pause at start, wait for GDB on :1234
#   ./run.sh --log                # Boot with QEMU tracing to qemu.log
#   ./run.sh --rebuild            # Rebuild before launching
#

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
QEMU_DIR="$SCRIPT_DIR"

# Default architecture
QEMU_ARCH="arm64"

# Parse arguments
DEBUG_FLAGS=""
LOG_FLAGS=""
REBUILD=0
GUI_MODE=0
LIVECD=0

while [ $# -gt 0 ]; do
    case "$1" in
        --arch)
            shift
            QEMU_ARCH="$1"
            ;;
        --gui)
            GUI_MODE=1
            echo "[*] GUI mode: QEMU window with virtio mouse/keyboard"
            ;;
        --livecd)
            LIVECD=1
            echo "[*] LiveCD mode: boot to desktop (no installation)"
            ;;
        --debug)
            DEBUG_FLAGS="-s -S"
            echo "[*] Debug mode: QEMU paused, waiting for GDB on localhost:1234"
            ;;
        --log)
            LOG_FLAGS="-d int,cpu_reset,guest_errors -D $QEMU_DIR/qemu.log"
            echo "[*] Logging enabled: $QEMU_DIR/qemu.log"
            ;;
        --rebuild)
            REBUILD=1
            ;;
        --help|-h)
            echo "Usage: $0 [--arch i386|amd64|arm64] [--gui] [--debug] [--log] [--rebuild]"
            echo ""
            echo "  --arch ARCH  Target architecture (default: arm64)"
            echo "  --gui        Open QEMU window with virtio mouse/keyboard (arm64)"
            echo "  --debug      Pause at startup, GDB server on :1234"
            echo "  --log        Enable QEMU exception/interrupt logging"
            echo "  --rebuild    Run ninja to rebuild before launching"
            exit 0
            ;;
    esac
    shift
done

# Architecture-specific configuration
case "$QEMU_ARCH" in
    arm64)
        BUILD_DIR="$PROJECT_DIR/build-arm64"
        UEFI_FW="$QEMU_DIR/QEMU_EFI.fd"
        EFIVARS="$QEMU_DIR/efivars.fd"
        BOOT_IMG="$QEMU_DIR/boot.img"

        # Rebuild if requested
        if [ "$REBUILD" -eq 1 ]; then
            echo "[*] Rebuilding uefildr.efi..."
            ninja -C "$BUILD_DIR" uefildr
        fi

        # NOTE: Do NOT mcopy uefildr.efi into boot.img here!
        # Updating a file in the FAT32 boot.img after ramdisk.img was placed there
        # can corrupt the ramdisk data. Use mkramdisk.sh to rebuild boot.img instead.
        if [ ! -f "$BOOT_IMG" ]; then
            echo "[!] boot.img not found at: $BOOT_IMG"
            echo "    Run: bash qemu/mkramdisk.sh"
            exit 1
        fi

        # Build display/input flags based on mode
        if [ "$GUI_MODE" -eq 1 ]; then
            DISPLAY_FLAGS="-device virtio-keyboard-device -device virtio-tablet-device"
        else
            DISPLAY_FLAGS="-nographic"
        fi

        echo "[*] Launching QEMU ARM64 (Ctrl-A X to quit)..."
        echo "---"

        qemu-system-aarch64 \
            -M virt \
            -cpu cortex-a72 \
            -m 1G \
            -drive if=pflash,format=raw,file="$UEFI_FW",readonly=on \
            -drive if=pflash,format=raw,file="$EFIVARS" \
            -drive file="$BOOT_IMG",format=raw,if=virtio \
            -device ramfb \
            -chardev socket,id=ser0,path="$QEMU_DIR/serial.sock",server=on,wait=off \
            -serial chardev:ser0 \
            -monitor telnet:localhost:4444,server,nowait \
            $DISPLAY_FLAGS \
            $DEBUG_FLAGS \
            $LOG_FLAGS
        ;;
    i386)
        BUILD_DIR="$PROJECT_DIR/build-i386"
        HDD_IMG="$QEMU_DIR/i386_hdd.img"

        if [ "$LIVECD" -eq 1 ]; then
            ISO="$BUILD_DIR/livecd.iso"
            if [ "$REBUILD" -eq 1 ]; then
                echo "[*] Rebuilding livecd..."
                ninja -C "$BUILD_DIR" livecd
            fi
            if [ ! -f "$ISO" ]; then
                echo "[!] livecd.iso not found at: $ISO"
                echo "    Run: ninja -C $BUILD_DIR livecd"
                exit 1
            fi
            HDD_FLAGS=""
            DISPLAY_FLAGS="-device VGA -serial stdio"
        else
            ISO="$BUILD_DIR/bootcd.iso"
            if [ "$REBUILD" -eq 1 ]; then
                echo "[*] Rebuilding bootcd..."
                ninja -C "$BUILD_DIR" bootcd
            fi
            if [ ! -f "$ISO" ]; then
                echo "[!] bootcd.iso not found at: $ISO"
                echo "    Run: ninja -C $BUILD_DIR bootcd"
                exit 1
            fi
            # Create HDD image if it doesn't exist
            if [ ! -f "$HDD_IMG" ]; then
                echo "[*] Creating 2GB HDD image: $HDD_IMG"
                qemu-img create -f raw "$HDD_IMG" 2G
            fi
            # Boot order: CD first, then HDD
            HDD_FLAGS="-drive file=$HDD_IMG,format=raw,if=ide -boot order=dc"
            DISPLAY_FLAGS="-nographic"
        fi

        echo "[*] Launching QEMU i386..."
        echo "[*] CD: $ISO"
        echo "---"

        # Use KVM if available for much faster I/O
        KVM_FLAGS=""
        if [ -r /dev/kvm ]; then
            KVM_FLAGS="-accel kvm -cpu host"
        fi

        qemu-system-i386 \
            -M pc \
            ${KVM_FLAGS:--cpu Westmere} \
            -m 512M \
            -cdrom "$ISO" \
            $HDD_FLAGS \
            $DISPLAY_FLAGS \
            -no-reboot \
            $DEBUG_FLAGS \
            $LOG_FLAGS
        ;;
    amd64)
        BUILD_DIR="$PROJECT_DIR/build-amd64"
        HDD_IMG="$QEMU_DIR/amd64_hdd.img"

        if [ "$LIVECD" -eq 1 ]; then
            ISO="$BUILD_DIR/livecd.iso"
            if [ "$REBUILD" -eq 1 ]; then
                echo "[*] Rebuilding livecd..."
                ninja -C "$BUILD_DIR" livecd
            fi
            if [ ! -f "$ISO" ]; then
                echo "[!] livecd.iso not found at: $ISO"
                echo "    Run: ninja -C $BUILD_DIR livecd"
                exit 1
            fi
            # LiveCD: VGA + serial debug output + telnet monitor
            HDD_FLAGS=""
            DISPLAY_FLAGS="-display none -vga std -serial stdio -monitor telnet:localhost:4444,server,nowait"
        else
            ISO="$BUILD_DIR/bootcd.iso"
            if [ "$REBUILD" -eq 1 ]; then
                echo "[*] Rebuilding bootcd..."
                ninja -C "$BUILD_DIR" bootcd
            fi
            if [ ! -f "$ISO" ]; then
                echo "[!] bootcd.iso not found at: $ISO"
                echo "    Run: ninja -C $BUILD_DIR bootcd"
                exit 1
            fi
            # Create HDD image if it doesn't exist
            if [ ! -f "$HDD_IMG" ]; then
                echo "[*] Creating 2GB HDD image: $HDD_IMG"
                qemu-img create -f raw "$HDD_IMG" 2G
            fi
            HDD_FLAGS="-drive file=$HDD_IMG,format=raw,if=ide -boot order=dc"
            DISPLAY_FLAGS="-nographic"
        fi

        echo "[*] Launching QEMU amd64..."
        echo "[*] CD: $ISO"
        echo "---"

        qemu-system-x86_64 \
            -M pc \
            -accel kvm -cpu host \
            -m 512M \
            -cdrom "$ISO" \
            $HDD_FLAGS \
            $DISPLAY_FLAGS \
            -no-reboot \
            $DEBUG_FLAGS \
            $LOG_FLAGS
        ;;
    *)
        echo "[!] Unknown architecture: $QEMU_ARCH"
        echo "    Supported: i386, amd64, arm64"
        exit 1
        ;;
esac
