#!/bin/bash
#
# Create ramdisk.img and update boot.img for ReactOS ARM64 ramdisk boot
#
# Usage:
#   ./mkramdisk.sh              # Create ramdisk and update boot image
#   ./mkramdisk.sh --rebuild    # Rebuild kernel first
#

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build-arm64"
QEMU_DIR="$SCRIPT_DIR"
SOURCE_DIR="$PROJECT_DIR"

RAMDISK_IMG="$QEMU_DIR/ramdisk.img"
BOOT_IMG="$QEMU_DIR/boot.img"
UEFILDR="$BUILD_DIR/boot/freeldr/freeldr/uefildr.efi"
FREELDR_INI="$QEMU_DIR/freeldr.ini"

# Ramdisk size in MB (must hold all system files ~120MB + overhead)
RAMDISK_SIZE_MB=140
# Boot image size in MB (must hold uefildr + freeldr.ini + ramdisk.img)
BOOT_SIZE_MB=280

# --refresh: fast re-stage of freshly-built artifacts into the EXISTING
# ramdisk.img (skip the slow dd/mformat + FAT (re)creation), then fold into
# boot.img. This is the arm64 structural equivalent of `ninja livecd` for
# i386/amd64 and is what the smoke conftest calls so arm64 never boots stale
# binaries. RMC() centralises the ramdisk mcopy so a single `-o` makes every
# populate call idempotent (harmless on a fresh image, overwrite on refresh)
# — zero list drift vs the full build path (same calls run either way).
REFRESH=""

# Stage debug-stripped PE twins into the ramdisk (same rationale as the livecd
# strip pass in boot/boot_images.cmake): DWARF sections are never mapped, and
# they'd bloat the ramdisk ~3x. The strip tool prints the twin path for PE
# files with debug sections, or the original path otherwise; it HARD-FAILS on
# a strip error (never silently stages an unstripped file).
STRIP_CACHE="$BUILD_DIR/boot/stripped"
STRIP_OBJCOPY="$PROJECT_DIR/.build-tools/llvm-mingw/bin/llvm-objcopy"
RMC() {
    local src
    src="$(python3 "$PROJECT_DIR/tools/strip_cd_media.py" --file "$1" \
            --cache "$STRIP_CACHE" --objcopy "$STRIP_OBJCOPY")" || exit 1
    shift
    mcopy -o -i "$RAMDISK_IMG" "$src" "$@"
}

# Parse arguments
for arg in "$@"; do
    case "$arg" in
        --rebuild)
            echo "[*] Rebuilding kernel..."
            ninja -C "$BUILD_DIR" ntoskrnl hal uefildr ramdisk fastfat
            ;;
        --refresh)
            REFRESH=1
            ;;
        --help|-h)
            echo "Usage: $0 [--rebuild] [--refresh]"
            echo "  --rebuild  Build kernel, HAL, bootloader, ramdisk.sys, fastfat.sys first"
            echo "  --refresh  Re-stage built artifacts into the existing ramdisk.img"
            echo "             (skip image re-creation), then fold into boot.img. Fast;"
            echo "             used by the smoke harness to avoid stale-boot-media passes."
            exit 0
            ;;
    esac
done

# Verify required files exist
echo "[*] Checking build artifacts..."
REQUIRED_FILES=(
    "$BUILD_DIR/ntoskrnl/ntoskrnl.exe"
    "$BUILD_DIR/hal/halarm64/hal.dll"
    "$BUILD_DIR/drivers/base/kdrosdbg/kdcom.dll"
    "$BUILD_DIR/drivers/base/bootvid/bootvid.dll"
    "$BUILD_DIR/dll/ntdll/ntdll.dll"
    "$BUILD_DIR/boot/bootdata/system"
    "$BUILD_DIR/drivers/storage/class/ramdisk/ramdisk.sys"
    "$BUILD_DIR/drivers/filesystems/fastfat/fastfat.sys"
    "$BUILD_DIR/base/system/smss/smss.exe"
    "$BUILD_DIR/subsystems/csr/csrss/csrss.exe"
    "$BUILD_DIR/subsystems/csr/csrsrv/csrsrv.dll"
    "$BUILD_DIR/subsystems/win/basesrv/basesrv.dll"
    "$BUILD_DIR/media/nls/c_437.nls"
    "$BUILD_DIR/media/nls/c_1252.nls"
    "$BUILD_DIR/dll/win32/kernel32/kernel32.dll"
    "$BUILD_DIR/win32ss/user/user32/user32.dll"
    "$BUILD_DIR/win32ss/gdi/gdi32/gdi32.dll"
    "$BUILD_DIR/dll/win32/advapi32/advapi32.dll"
    "$BUILD_DIR/dll/win32/rpcrt4/rpcrt4.dll"
    "$BUILD_DIR/dll/win32/ws2_32/ws2_32.dll"
    "$BUILD_DIR/dll/win32/ws2help/ws2help.dll"
    # netstack usermode winsock chain (#92): afd.sys (Start=1 TDI client, loaded
    # explicitly on arm64 — see iomgr/driver.rs) + msafd/mswsock providers +
    # nettest.exe (winlogon spawns it; the ws2_32->msafd->AFD->netstack proof).
    "$BUILD_DIR/drivers/network/afd/afd.sys"
    "$BUILD_DIR/dll/win32/msafd/msafd.dll"
    "$BUILD_DIR/dll/win32/mswsock/mswsock.dll"
    "$BUILD_DIR/base/applications/network/nettest/nettest.exe"
    "$BUILD_DIR/dll/win32/msvcrt/msvcrt.dll"
    "$BUILD_DIR/dll/win32/kernel32/kernel32_vista/kernel32_vista.dll"
    # kernelbase.dll — minimal Vista-scoped synch surface; api-ms-win-core-synch
    # -l1-2-0 redirects here (FEX rung 2). Forwards to kernel32_vista/ntdll.
    "$BUILD_DIR/dll/win32/kernelbase/kernelbase.dll"
    "$BUILD_DIR/dll/ntdll/nt_0600/ntdll_vista.dll"
    "$BUILD_DIR/base/system/winlogon/winlogon.exe"
    "$BUILD_DIR/win32ss/win32k.sys"
    "$BUILD_DIR/win32ss/drivers/font/ftfd/ftfd.dll"
    "$BUILD_DIR/base/applications/testchild/testchild.exe"
    "$BUILD_DIR/base/system/sehtest/sehtest.exe"
    "$BUILD_DIR/base/applications/testgui/testgui.exe"
    "$BUILD_DIR/base/applications/notepad/notepad.exe"
    "$BUILD_DIR/dll/keyboard/kbdus.dll"
    "$BUILD_DIR/dll/win32/userenv/userenv.dll"
    "$BUILD_DIR/dll/win32/secur32/secur32.dll"
    "$BUILD_DIR/dll/win32/mpr/mpr.dll"
    "$BUILD_DIR/dll/win32/netapi32/netapi32.dll"
    "$BUILD_DIR/dll/win32/iphlpapi/iphlpapi.dll"
    "$BUILD_DIR/base/shell/cmd/cmd.exe"
    "$BUILD_DIR/win32ss/user/winsrv/winsrv.dll"
    "$BUILD_DIR/drivers/serial/pl011/pl011.sys"
    "$BUILD_DIR/drivers/network/netstack/netstack.sys"
    "$BUILD_DIR/drivers/input/arm64input/arm64input.sys"
    "$BUILD_DIR/drivers/video/arm64vid/arm64vid.sys"
    # PCI + USB + HID input stack (input-unification: EHCI-on-PCIe -> usbport ->
    # usbhub -> hidusb -> hidclass -> kbdhid/mouhid -> kbdclass/mouclass). The
    # PCI root devnode is seeded by hivesys_arm64.inf (Root\*PNP0A03).
    "$BUILD_DIR/drivers/bus/pci/pci.sys"
    "$BUILD_DIR/drivers/usb/usbd/usbd.sys"
    "$BUILD_DIR/drivers/usb/usbport/usbport.sys"
    "$BUILD_DIR/drivers/usb/usbehci/usbehci.sys"
    "$BUILD_DIR/drivers/usb/usbhub/usbhub.sys"
    "$BUILD_DIR/drivers/hid/hidparse/hidparse.sys"
    "$BUILD_DIR/drivers/hid/hidclass/hidclass.sys"
    "$BUILD_DIR/drivers/hid/hidusb/hidusb.sys"
    "$BUILD_DIR/drivers/hid/kbdhid/kbdhid.sys"
    "$BUILD_DIR/drivers/hid/mouhid/mouhid.sys"
    "$BUILD_DIR/drivers/input/kbdclass/kbdclass.sys"
    "$BUILD_DIR/drivers/input/mouclass/mouclass.sys"
    "$BUILD_DIR/win32ss/drivers/displays/framebuf/framebuf.dll"
    # Explorer.exe + shell DLLs
    "$BUILD_DIR/base/shell/explorer/explorer.exe"
    "$BUILD_DIR/dll/win32/comctl32/comctl32.dll"
    "$BUILD_DIR/dll/win32/ole32/ole32.dll"
    "$BUILD_DIR/dll/win32/oleaut32/oleaut32.dll"
    "$BUILD_DIR/dll/win32/shell32/shell32.dll"
    "$BUILD_DIR/dll/win32/browseui/browseui.dll"
    "$BUILD_DIR/base/shell/rshell/rshell.dll"
    "$BUILD_DIR/dll/win32/shlwapi/shlwapi.dll"
    "$BUILD_DIR/dll/win32/version/version.dll"
    "$BUILD_DIR/dll/win32/uxtheme/uxtheme.dll"
    # Level 2 transitive deps
    "$BUILD_DIR/dll/win32/winmm/winmm.dll"
    "$BUILD_DIR/dll/win32/usp10/usp10.dll"
    "$BUILD_DIR/win32ss/user/imm32/imm32.dll"
    "$BUILD_DIR/dll/win32/urlmon/urlmon.dll"
    "$BUILD_DIR/dll/win32/windowscodecs/windowscodecs.dll"
    "$BUILD_DIR/dll/win32/powrprof/powrprof.dll"
    "$BUILD_DIR/dll/win32/devmgr/devmgr.dll"
    "$BUILD_DIR/win32ss/printing/base/winspool/winspool.drv"
    "$BUILD_DIR/dll/win32/fmifs/fmifs.dll"
    "$BUILD_DIR/dll/win32/comdlg32/comdlg32.dll"
    "$BUILD_DIR/dll/win32/shdocvw/shdocvw.dll"
    "$BUILD_DIR/dll/win32/mlang/mlang.dll"
    "$BUILD_DIR/dll/win32/msimg32/msimg32.dll"
    # Level 3+ transitive deps
    "$BUILD_DIR/dll/win32/advpack/advpack.dll"
    "$BUILD_DIR/dll/win32/wininet/wininet.dll"
    "$BUILD_DIR/dll/3rdparty/libjpeg/libjpeg.dll"
    "$BUILD_DIR/dll/3rdparty/libpng/libpng.dll"
    "$BUILD_DIR/dll/3rdparty/libtiff/libtiff.dll"
    "$BUILD_DIR/dll/win32/advapi32_vista/advapi32_vista.dll"
    "$BUILD_DIR/dll/win32/propsys/propsys.dll"
    "$BUILD_DIR/dll/win32/setupapi/setupapi.dll"
    "$BUILD_DIR/dll/win32/newdev/newdev.dll"
    "$BUILD_DIR/dll/win32/crypt32/crypt32.dll"
    "$BUILD_DIR/dll/win32/cryptui/cryptui.dll"
    "$BUILD_DIR/base/services/dhcpcsvc/dhcpcsvc.dll"
    "$BUILD_DIR/dll/nls/normaliz/normaliz.dll"
    "$BUILD_DIR/dll/win32/wintrust/wintrust.dll"
    "$BUILD_DIR/dll/win32/cryptnet/cryptnet.dll"
    "$BUILD_DIR/dll/win32/bcrypt/bcrypt.dll"
    "$BUILD_DIR/dll/win32/imagehlp/imagehlp.dll"
    "$BUILD_DIR/dll/3rdparty/mbedtls/mbedtls.dll"
    "$BUILD_DIR/dll/win32/dbghelp/dbghelp.dll"
    "$BUILD_DIR/dll/win32/psapi/psapi.dll"
    "$UEFILDR"
)

for f in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$f" ]; then
        echo "[!] Missing: $f"
        echo "    Run: ninja -C $BUILD_DIR ntoskrnl hal uefildr ramdisk fastfat"
        exit 1
    fi
done

# l_intl.nls: try build dir first, fall back to source tree
L_INTL_NLS=""
if [ -f "$BUILD_DIR/media/nls/l_intl.nls" ]; then
    L_INTL_NLS="$BUILD_DIR/media/nls/l_intl.nls"
elif [ -f "$SOURCE_DIR/media/nls/l_intl.nls" ]; then
    L_INTL_NLS="$SOURCE_DIR/media/nls/l_intl.nls"
else
    echo "[!] Warning: l_intl.nls not found, ramdisk may lack Unicode case table"
fi

echo "[*] All build artifacts found."

# --- Create ramdisk.img (skipped under --refresh: reuse existing FAT image) ---
if [ -n "$REFRESH" ]; then
    if [ ! -f "$RAMDISK_IMG" ]; then
        echo "[!] --refresh needs an existing $RAMDISK_IMG (run without --refresh once)"
        exit 1
    fi
    echo "[*] --refresh: reusing existing ramdisk.img (re-staging artifacts only)"
else
    echo "[*] Creating ramdisk.img (${RAMDISK_SIZE_MB}MB FAT32)..."
    rm -f "$RAMDISK_IMG"
    # Create empty image
    dd if=/dev/zero of="$RAMDISK_IMG" bs=1M count=$RAMDISK_SIZE_MB 2>/dev/null
    # Format as FAT32
    # Note: mformat -F defaults to 1 sector/cluster (~96K clusters for 48MB).
    # The kernel FAT scan of ~96K entries takes ~3 minutes on QEMU TCG.
    # -c 8 (4KB clusters) would be faster but breaks FreeLDR's FAT reader.
    mformat -i "$RAMDISK_IMG" -F -v REACTOS ::

    # Create directory structure
    mmd -i "$RAMDISK_IMG" ::/reactos
    mmd -i "$RAMDISK_IMG" ::/reactos/system32
    mmd -i "$RAMDISK_IMG" ::/reactos/system32/config
    mmd -i "$RAMDISK_IMG" ::/reactos/system32/drivers
fi

# Copy system files
echo "[*] Populating ramdisk..."
RMC "$BUILD_DIR/ntoskrnl/ntoskrnl.exe"                        ::/reactos/system32/ntoskrnl.exe
RMC "$BUILD_DIR/hal/halarm64/hal.dll"                          ::/reactos/system32/hal.dll
RMC "$BUILD_DIR/drivers/base/kdrosdbg/kdcom.dll"               ::/reactos/system32/kdcom.dll
RMC "$BUILD_DIR/drivers/base/bootvid/bootvid.dll"              ::/reactos/system32/bootvid.dll
RMC "$BUILD_DIR/dll/ntdll/ntdll.dll"                           ::/reactos/system32/ntdll.dll
RMC "$BUILD_DIR/boot/bootdata/system"                          ::/reactos/system32/config/system
RMC "$BUILD_DIR/boot/bootdata/default"                         ::/reactos/system32/config/default
RMC "$BUILD_DIR/boot/bootdata/software"                        ::/reactos/system32/config/software
RMC "$BUILD_DIR/drivers/storage/class/ramdisk/ramdisk.sys"     ::/reactos/system32/drivers/ramdisk.sys
RMC "$BUILD_DIR/drivers/filesystems/fastfat/fastfat.sys"       ::/reactos/system32/drivers/fastfat.sys
RMC "$BUILD_DIR/drivers/network/netstack/netstack.sys"         ::/reactos/system32/drivers/netstack.sys
RMC "$BUILD_DIR/drivers/network/afd/afd.sys"                   ::/reactos/system32/drivers/afd.sys
RMC "$BUILD_DIR/base/system/smss/smss.exe"                   ::/reactos/system32/smss.exe
# Patch csrss.exe SizeOfStackCommit to 256KB — the PE default (4KB) is too small
# for deep DLL initialization call chains on ARM64 where delay-imports become
# regular imports (all DLLs load at once via recursive LdrpLoadDll).
CSRSS_TMP=$(mktemp)
cp "$BUILD_DIR/subsystems/csr/csrss/csrss.exe" "$CSRSS_TMP"
python3 "$SCRIPT_DIR/patch_pe_stack.py" "$CSRSS_TMP" 0x40000
RMC "$CSRSS_TMP"                                         ::/reactos/system32/csrss.exe
rm -f "$CSRSS_TMP"
RMC "$BUILD_DIR/subsystems/csr/csrsrv/csrsrv.dll"           ::/reactos/system32/csrsrv.dll
RMC "$BUILD_DIR/subsystems/win/basesrv/basesrv.dll"         ::/reactos/system32/basesrv.dll
RMC "$BUILD_DIR/dll/win32/kernel32/kernel32.dll"               ::/reactos/system32/kernel32.dll
RMC "$BUILD_DIR/win32ss/user/user32/user32.dll"               ::/reactos/system32/user32.dll
RMC "$BUILD_DIR/win32ss/gdi/gdi32/gdi32.dll"                  ::/reactos/system32/gdi32.dll
RMC "$BUILD_DIR/dll/win32/advapi32/advapi32.dll"              ::/reactos/system32/advapi32.dll
RMC "$BUILD_DIR/dll/win32/rpcrt4/rpcrt4.dll"                  ::/reactos/system32/rpcrt4.dll
RMC "$BUILD_DIR/dll/win32/ws2_32/ws2_32.dll"                  ::/reactos/system32/ws2_32.dll
RMC "$BUILD_DIR/dll/win32/ws2help/ws2help.dll"                ::/reactos/system32/ws2help.dll
RMC "$BUILD_DIR/dll/win32/msafd/msafd.dll"                    ::/reactos/system32/msafd.dll
RMC "$BUILD_DIR/dll/win32/mswsock/mswsock.dll"                ::/reactos/system32/mswsock.dll
RMC "$BUILD_DIR/base/applications/network/nettest/nettest.exe" ::/reactos/system32/nettest.exe
RMC "$BUILD_DIR/dll/win32/msvcrt/msvcrt.dll"                  ::/reactos/system32/msvcrt.dll
RMC "$BUILD_DIR/dll/win32/kernel32/kernel32_vista/kernel32_vista.dll"  ::/reactos/system32/kernel32_vista.dll
RMC "$BUILD_DIR/dll/win32/kernelbase/kernelbase.dll"          ::/reactos/system32/kernelbase.dll
# wow64.dll — WOW64 host-layer skeleton (docs/engineering/wow64-plan.md);
# libwow64fex.dll imports it, so it must stage wherever the FEX layer does.
RMC "$BUILD_DIR/dll/win32/wow64/wow64.dll"                    ::/reactos/system32/wow64.dll
RMC "$BUILD_DIR/dll/ntdll/nt_0600/ntdll_vista.dll"           ::/reactos/system32/ntdll_vista.dll
RMC "$BUILD_DIR/base/system/winlogon/winlogon.exe"            ::/reactos/system32/winlogon.exe
RMC "$BUILD_DIR/win32ss/win32k.sys"                           ::/reactos/system32/win32k.sys
RMC "$BUILD_DIR/win32ss/drivers/font/ftfd/ftfd.dll"           ::/reactos/system32/ftfd.dll
RMC "$BUILD_DIR/base/applications/testchild/testchild.exe"   ::/reactos/system32/testchild.exe
# crash-value: user-mode SEH boot self-test (spawned by winlogon; [SEH-USER] on serial)
RMC "$BUILD_DIR/base/system/sehtest/sehtest.exe"              ::/reactos/system32/sehtest.exe
RMC "$BUILD_DIR/base/applications/testgui/testgui.exe"     ::/reactos/system32/testgui.exe
RMC "$BUILD_DIR/base/applications/notepad/notepad.exe"    ::/reactos/system32/notepad.exe
RMC "$BUILD_DIR/dll/keyboard/kbdus.dll"                  ::/reactos/system32/kbdus.dll
RMC "$BUILD_DIR/dll/win32/userenv/userenv.dll"                ::/reactos/system32/userenv.dll
RMC "$BUILD_DIR/dll/win32/secur32/secur32.dll"                ::/reactos/system32/secur32.dll
RMC "$BUILD_DIR/dll/win32/mpr/mpr.dll"                        ::/reactos/system32/mpr.dll
RMC "$BUILD_DIR/dll/win32/netapi32/netapi32.dll"              ::/reactos/system32/netapi32.dll
RMC "$BUILD_DIR/dll/win32/samlib/samlib.dll"                  ::/reactos/system32/samlib.dll
RMC "$BUILD_DIR/dll/win32/iphlpapi/iphlpapi.dll"              ::/reactos/system32/iphlpapi.dll
RMC "$BUILD_DIR/base/shell/cmd/cmd.exe"                     ::/reactos/system32/cmd.exe
RMC "$BUILD_DIR/win32ss/user/winsrv/winsrv.dll"              ::/reactos/system32/winsrv.dll
RMC "$BUILD_DIR/drivers/serial/pl011/pl011.sys"              ::/reactos/system32/drivers/pl011.sys
RMC "$BUILD_DIR/drivers/input/arm64input/arm64input.sys" ::/reactos/system32/drivers/arm64input.sys
RMC "$BUILD_DIR/drivers/video/arm64vid/arm64vid.sys"       ::/reactos/system32/drivers/arm64vid.sys
# PCI + USB + HID input stack (input-unification quest)
RMC "$BUILD_DIR/drivers/bus/pci/pci.sys"                   ::/reactos/system32/drivers/pci.sys
RMC "$BUILD_DIR/drivers/usb/usbd/usbd.sys"                 ::/reactos/system32/drivers/usbd.sys
RMC "$BUILD_DIR/drivers/usb/usbport/usbport.sys"           ::/reactos/system32/drivers/usbport.sys
RMC "$BUILD_DIR/drivers/usb/usbehci/usbehci.sys"           ::/reactos/system32/drivers/usbehci.sys
RMC "$BUILD_DIR/drivers/usb/usbhub/usbhub.sys"             ::/reactos/system32/drivers/usbhub.sys
RMC "$BUILD_DIR/drivers/hid/hidparse/hidparse.sys"         ::/reactos/system32/drivers/hidparse.sys
RMC "$BUILD_DIR/drivers/hid/hidclass/hidclass.sys"         ::/reactos/system32/drivers/hidclass.sys
RMC "$BUILD_DIR/drivers/hid/hidusb/hidusb.sys"             ::/reactos/system32/drivers/hidusb.sys
RMC "$BUILD_DIR/drivers/hid/kbdhid/kbdhid.sys"             ::/reactos/system32/drivers/kbdhid.sys
RMC "$BUILD_DIR/drivers/hid/mouhid/mouhid.sys"             ::/reactos/system32/drivers/mouhid.sys
RMC "$BUILD_DIR/drivers/input/kbdclass/kbdclass.sys"       ::/reactos/system32/drivers/kbdclass.sys
RMC "$BUILD_DIR/drivers/input/mouclass/mouclass.sys"       ::/reactos/system32/drivers/mouclass.sys
RMC "$BUILD_DIR/win32ss/drivers/displays/framebuf/framebuf.dll"  ::/reactos/system32/framebuf.dll

# Explorer.exe + shell DLLs
RMC "$BUILD_DIR/base/shell/explorer/explorer.exe"              ::/reactos/system32/explorer.exe
# winlogon.c builds the explorer path as %SystemRoot%\explorer.exe (= X:\reactos\explorer.exe on this ramdisk),
# so also place a copy at that location. The system32 copy is kept for any code path that still searches there.
RMC "$BUILD_DIR/base/shell/explorer/explorer.exe"              ::/reactos/explorer.exe
RMC "$BUILD_DIR/dll/win32/comctl32/comctl32.dll"               ::/reactos/system32/comctl32.dll
RMC "$BUILD_DIR/dll/win32/ole32/ole32.dll"                     ::/reactos/system32/ole32.dll
RMC "$BUILD_DIR/dll/win32/oleaut32/oleaut32.dll"               ::/reactos/system32/oleaut32.dll
RMC "$BUILD_DIR/dll/win32/shell32/shell32.dll"                 ::/reactos/system32/shell32.dll
RMC "$BUILD_DIR/dll/win32/browseui/browseui.dll"               ::/reactos/system32/browseui.dll
RMC "$BUILD_DIR/base/shell/rshell/rshell.dll"                  ::/reactos/system32/rshell.dll
RMC "$BUILD_DIR/dll/win32/shlwapi/shlwapi.dll"                 ::/reactos/system32/shlwapi.dll
RMC "$BUILD_DIR/dll/win32/version/version.dll"                 ::/reactos/system32/version.dll
RMC "$BUILD_DIR/dll/win32/uxtheme/uxtheme.dll"                 ::/reactos/system32/uxtheme.dll
# Level 2 transitive deps
RMC "$BUILD_DIR/dll/win32/winmm/winmm.dll"                    ::/reactos/system32/winmm.dll
RMC "$BUILD_DIR/dll/win32/usp10/usp10.dll"                    ::/reactos/system32/usp10.dll
RMC "$BUILD_DIR/win32ss/user/imm32/imm32.dll"                 ::/reactos/system32/imm32.dll
RMC "$BUILD_DIR/dll/win32/urlmon/urlmon.dll"                   ::/reactos/system32/urlmon.dll
RMC "$BUILD_DIR/dll/win32/windowscodecs/windowscodecs.dll"     ::/reactos/system32/windowscodecs.dll
RMC "$BUILD_DIR/dll/win32/powrprof/powrprof.dll"               ::/reactos/system32/powrprof.dll
RMC "$BUILD_DIR/dll/win32/devmgr/devmgr.dll"                  ::/reactos/system32/devmgr.dll
RMC "$BUILD_DIR/win32ss/printing/base/winspool/winspool.drv"   ::/reactos/system32/winspool.drv
RMC "$BUILD_DIR/dll/win32/fmifs/fmifs.dll"                    ::/reactos/system32/fmifs.dll
RMC "$BUILD_DIR/dll/win32/comdlg32/comdlg32.dll"              ::/reactos/system32/comdlg32.dll
RMC "$BUILD_DIR/dll/win32/shdocvw/shdocvw.dll"                ::/reactos/system32/shdocvw.dll
RMC "$BUILD_DIR/dll/win32/mlang/mlang.dll"                    ::/reactos/system32/mlang.dll
RMC "$BUILD_DIR/dll/win32/msimg32/msimg32.dll"                ::/reactos/system32/msimg32.dll
# Level 3+ transitive deps
RMC "$BUILD_DIR/dll/win32/advpack/advpack.dll"                ::/reactos/system32/advpack.dll
RMC "$BUILD_DIR/dll/win32/wininet/wininet.dll"                ::/reactos/system32/wininet.dll
RMC "$BUILD_DIR/dll/3rdparty/libjpeg/libjpeg.dll"             ::/reactos/system32/libjpeg.dll
RMC "$BUILD_DIR/dll/3rdparty/libpng/libpng.dll"               ::/reactos/system32/libpng.dll
RMC "$BUILD_DIR/dll/3rdparty/libtiff/libtiff.dll"             ::/reactos/system32/libtiff.dll
RMC "$BUILD_DIR/dll/win32/advapi32_vista/advapi32_vista.dll"   ::/reactos/system32/advapi32_vista.dll
RMC "$BUILD_DIR/dll/win32/propsys/propsys.dll"                ::/reactos/system32/propsys.dll
RMC "$BUILD_DIR/dll/win32/setupapi/setupapi.dll"              ::/reactos/system32/setupapi.dll
RMC "$BUILD_DIR/dll/win32/newdev/newdev.dll"                  ::/reactos/system32/newdev.dll
RMC "$BUILD_DIR/dll/win32/crypt32/crypt32.dll"                ::/reactos/system32/crypt32.dll
RMC "$BUILD_DIR/dll/win32/cryptui/cryptui.dll"                ::/reactos/system32/cryptui.dll
RMC "$BUILD_DIR/base/services/dhcpcsvc/dhcpcsvc.dll"          ::/reactos/system32/dhcpcsvc.dll
RMC "$BUILD_DIR/dll/nls/normaliz/normaliz.dll"                ::/reactos/system32/normaliz.dll
RMC "$BUILD_DIR/dll/win32/wintrust/wintrust.dll"              ::/reactos/system32/wintrust.dll
RMC "$BUILD_DIR/dll/win32/cryptnet/cryptnet.dll"              ::/reactos/system32/cryptnet.dll
RMC "$BUILD_DIR/dll/win32/bcrypt/bcrypt.dll"                  ::/reactos/system32/bcrypt.dll
RMC "$BUILD_DIR/dll/win32/imagehlp/imagehlp.dll"              ::/reactos/system32/imagehlp.dll
RMC "$BUILD_DIR/dll/3rdparty/mbedtls/mbedtls.dll"             ::/reactos/system32/mbedtls.dll
RMC "$BUILD_DIR/dll/win32/dbghelp/dbghelp.dll"                ::/reactos/system32/dbghelp.dll
RMC "$BUILD_DIR/dll/win32/psapi/psapi.dll"                    ::/reactos/system32/psapi.dll

RMC "$BUILD_DIR/media/nls/c_437.nls"                           ::/reactos/system32/c_437.nls
RMC "$BUILD_DIR/media/nls/c_1252.nls"                          ::/reactos/system32/c_1252.nls
if [ -n "$L_INTL_NLS" ]; then
    RMC "$L_INTL_NLS"                                          ::/reactos/system32/l_intl.nls
fi

# Fonts (loaded by IntLoadSystemFonts from \SystemRoot\Fonts\)
# Under --refresh, mmd can prompt on an existing directory when stdout is a tty.
# Probe first so refresh stays non-interactive under vibeos-cli.
mdir -i "$RAMDISK_IMG" ::reactos/Fonts >/dev/null 2>&1 || mmd -i "$RAMDISK_IMG" ::reactos/Fonts
mcopy -o -i "$RAMDISK_IMG" "$SOURCE_DIR/media/fonts/FreeSans.ttf"                   ::reactos/Fonts/FreeSans.ttf
# winlogon explicitly tries to load these names; without them
# Failed to lock the DC + SYSTEMCUR(ARROW) cascades and the taskbar
# never paints text/glyphs.
for FONT in tahoma.ttf tahomabd.ttf Marlett.ttf arial.ttf arialbd.ttf cour.ttf times.ttf symbol.ttf; do
    if [ -f "$SOURCE_DIR/media/fonts/$FONT" ]; then
        mcopy -o -i "$RAMDISK_IMG" "$SOURCE_DIR/media/fonts/$FONT"                  "::reactos/Fonts/$FONT"
    fi
done

# --- Payloads (opt-in, VIBEOS_PAYLOAD=name[,name...]) -----------------------
# Declarative test/demo app staging — docs/engineering/payload-harness-design.md.
# OFF by default: with no payloads selected the compiler prints nothing and the
# ramdisk is byte-identical. compile.py owns spec resolution + fail-loud checks
# (missing assets / dest collisions abort the build); the legacy VIBEOS_DOOM=1
# flag is aliased to the `doom` payload inside compile.py so the mapping lives
# in ONE place. `set -e` aborts on a compiler failure via the substitution.
# --payloads-auto: env wins when set; else the build-dir stamp — so env-less
# rebuilds (conftest, raw ninja) reproduce the same media instead of silently
# reverting the payload set (the 2026-07-07 doom-clobber incident).
PAYLOAD_MANIFEST=$(python3 "$PROJECT_DIR/tools/payloads/compile.py" manifest \
    --arch arm64 --payloads-auto "$BUILD_DIR")
if [ -n "$PAYLOAD_MANIFEST" ]; then
    echo "[*] staging payloads (VIBEOS_PAYLOAD=${VIBEOS_PAYLOAD:-<stamp>}${VIBEOS_DOOM:+ VIBEOS_DOOM=$VIBEOS_DOOM})"
    printf '%s\n' "$PAYLOAD_MANIFEST" | while IFS='=' read -r DEST SRC; do
        # mcopy can't create parent dirs — mmd each path component first
        # (probe-then-create, same non-interactive pattern as the Fonts dir).
        DIR="$(dirname "$DEST")"
        if [ "$DIR" != "." ]; then
            PARTIAL=""
            printf '%s\n' "$DIR" | tr '/' '\n' | while read -r COMP; do
                PARTIAL="${PARTIAL:+$PARTIAL/}$COMP"
                mdir -i "$RAMDISK_IMG" "::/$PARTIAL" >/dev/null 2>&1 \
                    || mmd -i "$RAMDISK_IMG" "::/$PARTIAL"
            done
        fi
        echo "[*]   $(basename "$SRC") -> ::/$DEST"
        mcopy -o -i "$RAMDISK_IMG" "$SRC" "::/$DEST"
    done
fi

# --- FEX hello-compute probe (opt-in, VIBEOS_FEX=1) ------------------------
# Rung-2 bring-up (docs/engineering/fex-bringup-plan.md): stage the native-arm64
# FEXCore harness fexprobe.exe + its UCRT runtime. OFF by default (winlogon's
# GetFileAttributesW check makes the launch a silent no-op without it).
# Build fexprobe first: tools/fex/build-fex.sh && tools/fex/build-hello-compute.sh
if [ "${VIBEOS_FEX:-0}" = "1" ]; then
    FEXPROBE="${VIBEOS_FEX_EXE:-$PROJECT_DIR/build-fex-hello/fexprobe.exe}"
    UCRTBASE="$BUILD_DIR/dll/win32/ucrtbase/ucrtbase.dll"
    if [ -f "$FEXPROBE" ] && [ -f "$UCRTBASE" ]; then
        echo "[*] VIBEOS_FEX=1 — staging fexprobe.exe + ucrtbase.dll into system32"
        mcopy -o -i "$RAMDISK_IMG" "$FEXPROBE" ::/reactos/system32/fexprobe.exe
        mcopy -o -i "$RAMDISK_IMG" "$UCRTBASE" ::/reactos/system32/ucrtbase.dll
        # WOW64 Phase A (docs/engineering/wow64-plan.md): stage the FEX CPU
        # layer so fexprobe's load-test (and later wow64probe) can resolve it.
        WOW64FEX="${VIBEOS_WOW64FEX_DLL:-$PROJECT_DIR/build-fex-arm64/Bin/libwow64fex.dll}"
        if [ -f "$WOW64FEX" ]; then
            echo "[*] VIBEOS_FEX=1 — staging libwow64fex.dll into system32"
            mcopy -o -i "$RAMDISK_IMG" "$WOW64FEX" ::/reactos/system32/libwow64fex.dll
        fi
        # WOW64 Phase B (docs/engineering/wow64-plan.md): wow64probe.exe drives
        # the BTCpu contract; it maps the i386 ntdll staged into syswow64
        # (Phase C's SysWOW64 set starts here).
        WOW64PROBE="${VIBEOS_WOW64PROBE_EXE:-$PROJECT_DIR/build-fex-hello/wow64probe.exe}"
        NTDLL32="${VIBEOS_NTDLL32:-$PROJECT_DIR/build-i386/dll/ntdll/ntdll.dll}"
        if [ -f "$WOW64PROBE" ] && [ -f "$NTDLL32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging wow64probe.exe + syswow64/ntdll.dll"
            mcopy -o -i "$RAMDISK_IMG" "$WOW64PROBE" ::/reactos/system32/wow64probe.exe
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$NTDLL32" ::/reactos/syswow64/ntdll.dll
        else
            echo "[!] VIBEOS_FEX=1: wow64probe not staged ($WOW64PROBE / $NTDLL32 missing) — Phase B probe skipped"
        fi
    else
        echo "[!] VIBEOS_FEX=1 but artifacts missing ($FEXPROBE / $UCRTBASE)."
        echo "[!]   Build: tools/fex/build-fex.sh && tools/fex/build-hello-compute.sh; ninja ucrtbase"
        exit 1
    fi
fi

echo "[*] Ramdisk contents:"
mdir -i "$RAMDISK_IMG" -/ :: 2>&1 | grep -E '(\.exe|\.dll|\.sys|\.nls|system|bytes)'

# --- Create boot.img ---
echo "[*] Creating boot.img (${BOOT_SIZE_MB}MB FAT32)..."
rm -f "$BOOT_IMG"
dd if=/dev/zero of="$BOOT_IMG" bs=1M count=$BOOT_SIZE_MB 2>/dev/null
mformat -i "$BOOT_IMG" -F -v ROSBOOT ::

# Create EFI directory
mmd -i "$BOOT_IMG" ::/EFI
mmd -i "$BOOT_IMG" ::/EFI/BOOT

# Copy bootloader
mcopy -i "$BOOT_IMG" "$UEFILDR" ::/EFI/BOOT/BOOTAA64.EFI

# Copy freeldr.ini
mcopy -i "$BOOT_IMG" "$FREELDR_INI" ::/freeldr.ini

# Copy ramdisk image
echo "[*] Copying ramdisk.img to boot.img..."
mcopy -i "$BOOT_IMG" "$RAMDISK_IMG" ::/ramdisk.img

echo "[*] Boot image contents:"
mdir -i "$BOOT_IMG" -/ :: 2>&1 | grep -E '(\.EFI|\.efi|\.ini|\.img|bytes free)'

echo ""
echo "[*] Done! Boot with: bash qemu/run.sh"
echo "[*] Ramdisk: $(du -h "$RAMDISK_IMG" | cut -f1)"
echo "[*] Boot image: $(du -h "$BOOT_IMG" | cut -f1)"
