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

# Parse arguments
for arg in "$@"; do
    case "$arg" in
        --rebuild)
            echo "[*] Rebuilding kernel..."
            ninja -C "$BUILD_DIR" ntoskrnl hal uefildr ramdisk fastfat
            ;;
        --help|-h)
            echo "Usage: $0 [--rebuild]"
            echo "  --rebuild  Build kernel, HAL, bootloader, ramdisk.sys, fastfat.sys first"
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
    "$BUILD_DIR/dll/win32/msvcrt/msvcrt.dll"
    "$BUILD_DIR/dll/win32/kernel32/kernel32_vista/kernel32_vista.dll"
    "$BUILD_DIR/dll/ntdll/nt_0600/ntdll_vista.dll"
    "$BUILD_DIR/base/system/winlogon/winlogon.exe"
    "$BUILD_DIR/win32ss/win32k.sys"
    "$BUILD_DIR/win32ss/drivers/font/ftfd/ftfd.dll"
    "$BUILD_DIR/base/applications/testchild/testchild.exe"
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
    "$BUILD_DIR/drivers/input/arm64input/arm64input.sys"
    "$BUILD_DIR/drivers/video/arm64vid/arm64vid.sys"
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

# --- Create ramdisk.img ---
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

# Copy system files
echo "[*] Populating ramdisk..."
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/ntoskrnl/ntoskrnl.exe"                        ::/reactos/system32/ntoskrnl.exe
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/hal/halarm64/hal.dll"                          ::/reactos/system32/hal.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/drivers/base/kdrosdbg/kdcom.dll"               ::/reactos/system32/kdcom.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/drivers/base/bootvid/bootvid.dll"              ::/reactos/system32/bootvid.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/ntdll/ntdll.dll"                           ::/reactos/system32/ntdll.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/boot/bootdata/system"                          ::/reactos/system32/config/system
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/boot/bootdata/default"                         ::/reactos/system32/config/default
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/boot/bootdata/software"                        ::/reactos/system32/config/software
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/drivers/storage/class/ramdisk/ramdisk.sys"     ::/reactos/system32/drivers/ramdisk.sys
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/drivers/filesystems/fastfat/fastfat.sys"       ::/reactos/system32/drivers/fastfat.sys
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/base/system/smss/smss.exe"                   ::/reactos/system32/smss.exe
# Patch csrss.exe SizeOfStackCommit to 256KB — the PE default (4KB) is too small
# for deep DLL initialization call chains on ARM64 where delay-imports become
# regular imports (all DLLs load at once via recursive LdrpLoadDll).
CSRSS_TMP=$(mktemp)
cp "$BUILD_DIR/subsystems/csr/csrss/csrss.exe" "$CSRSS_TMP"
python3 "$SCRIPT_DIR/patch_pe_stack.py" "$CSRSS_TMP" 0x40000
mcopy -i "$RAMDISK_IMG" "$CSRSS_TMP"                                         ::/reactos/system32/csrss.exe
rm -f "$CSRSS_TMP"
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/subsystems/csr/csrsrv/csrsrv.dll"           ::/reactos/system32/csrsrv.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/subsystems/win/basesrv/basesrv.dll"         ::/reactos/system32/basesrv.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/kernel32/kernel32.dll"               ::/reactos/system32/kernel32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/win32ss/user/user32/user32.dll"               ::/reactos/system32/user32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/win32ss/gdi/gdi32/gdi32.dll"                  ::/reactos/system32/gdi32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/advapi32/advapi32.dll"              ::/reactos/system32/advapi32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/rpcrt4/rpcrt4.dll"                  ::/reactos/system32/rpcrt4.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/ws2_32/ws2_32.dll"                  ::/reactos/system32/ws2_32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/ws2help/ws2help.dll"                ::/reactos/system32/ws2help.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/msvcrt/msvcrt.dll"                  ::/reactos/system32/msvcrt.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/kernel32/kernel32_vista/kernel32_vista.dll"  ::/reactos/system32/kernel32_vista.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/ntdll/nt_0600/ntdll_vista.dll"           ::/reactos/system32/ntdll_vista.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/base/system/winlogon/winlogon.exe"            ::/reactos/system32/winlogon.exe
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/win32ss/win32k.sys"                           ::/reactos/system32/win32k.sys
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/win32ss/drivers/font/ftfd/ftfd.dll"           ::/reactos/system32/ftfd.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/base/applications/testchild/testchild.exe"   ::/reactos/system32/testchild.exe
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/base/applications/testgui/testgui.exe"     ::/reactos/system32/testgui.exe
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/base/applications/notepad/notepad.exe"    ::/reactos/system32/notepad.exe
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/keyboard/kbdus.dll"                  ::/reactos/system32/kbdus.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/userenv/userenv.dll"                ::/reactos/system32/userenv.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/secur32/secur32.dll"                ::/reactos/system32/secur32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/mpr/mpr.dll"                        ::/reactos/system32/mpr.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/netapi32/netapi32.dll"              ::/reactos/system32/netapi32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/iphlpapi/iphlpapi.dll"              ::/reactos/system32/iphlpapi.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/base/shell/cmd/cmd.exe"                     ::/reactos/system32/cmd.exe
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/win32ss/user/winsrv/winsrv.dll"              ::/reactos/system32/winsrv.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/drivers/serial/pl011/pl011.sys"              ::/reactos/system32/drivers/pl011.sys
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/drivers/input/arm64input/arm64input.sys" ::/reactos/system32/drivers/arm64input.sys
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/drivers/video/arm64vid/arm64vid.sys"       ::/reactos/system32/drivers/arm64vid.sys
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/win32ss/drivers/displays/framebuf/framebuf.dll"  ::/reactos/system32/framebuf.dll

# Explorer.exe + shell DLLs
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/base/shell/explorer/explorer.exe"              ::/reactos/system32/explorer.exe
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/comctl32/comctl32.dll"               ::/reactos/system32/comctl32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/ole32/ole32.dll"                     ::/reactos/system32/ole32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/oleaut32/oleaut32.dll"               ::/reactos/system32/oleaut32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/shell32/shell32.dll"                 ::/reactos/system32/shell32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/browseui/browseui.dll"               ::/reactos/system32/browseui.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/base/shell/rshell/rshell.dll"                  ::/reactos/system32/rshell.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/shlwapi/shlwapi.dll"                 ::/reactos/system32/shlwapi.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/version/version.dll"                 ::/reactos/system32/version.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/uxtheme/uxtheme.dll"                 ::/reactos/system32/uxtheme.dll
# Level 2 transitive deps
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/winmm/winmm.dll"                    ::/reactos/system32/winmm.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/usp10/usp10.dll"                    ::/reactos/system32/usp10.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/win32ss/user/imm32/imm32.dll"                 ::/reactos/system32/imm32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/urlmon/urlmon.dll"                   ::/reactos/system32/urlmon.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/windowscodecs/windowscodecs.dll"     ::/reactos/system32/windowscodecs.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/powrprof/powrprof.dll"               ::/reactos/system32/powrprof.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/devmgr/devmgr.dll"                  ::/reactos/system32/devmgr.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/win32ss/printing/base/winspool/winspool.drv"   ::/reactos/system32/winspool.drv
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/fmifs/fmifs.dll"                    ::/reactos/system32/fmifs.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/comdlg32/comdlg32.dll"              ::/reactos/system32/comdlg32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/shdocvw/shdocvw.dll"                ::/reactos/system32/shdocvw.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/mlang/mlang.dll"                    ::/reactos/system32/mlang.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/msimg32/msimg32.dll"                ::/reactos/system32/msimg32.dll
# Level 3+ transitive deps
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/advpack/advpack.dll"                ::/reactos/system32/advpack.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/wininet/wininet.dll"                ::/reactos/system32/wininet.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/3rdparty/libjpeg/libjpeg.dll"             ::/reactos/system32/libjpeg.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/3rdparty/libpng/libpng.dll"               ::/reactos/system32/libpng.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/3rdparty/libtiff/libtiff.dll"             ::/reactos/system32/libtiff.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/advapi32_vista/advapi32_vista.dll"   ::/reactos/system32/advapi32_vista.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/propsys/propsys.dll"                ::/reactos/system32/propsys.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/setupapi/setupapi.dll"              ::/reactos/system32/setupapi.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/newdev/newdev.dll"                  ::/reactos/system32/newdev.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/crypt32/crypt32.dll"                ::/reactos/system32/crypt32.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/cryptui/cryptui.dll"                ::/reactos/system32/cryptui.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/base/services/dhcpcsvc/dhcpcsvc.dll"          ::/reactos/system32/dhcpcsvc.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/nls/normaliz/normaliz.dll"                ::/reactos/system32/normaliz.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/wintrust/wintrust.dll"              ::/reactos/system32/wintrust.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/cryptnet/cryptnet.dll"              ::/reactos/system32/cryptnet.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/bcrypt/bcrypt.dll"                  ::/reactos/system32/bcrypt.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/imagehlp/imagehlp.dll"              ::/reactos/system32/imagehlp.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/3rdparty/mbedtls/mbedtls.dll"             ::/reactos/system32/mbedtls.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/dbghelp/dbghelp.dll"                ::/reactos/system32/dbghelp.dll
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/dll/win32/psapi/psapi.dll"                    ::/reactos/system32/psapi.dll

mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/media/nls/c_437.nls"                           ::/reactos/system32/c_437.nls
mcopy -i "$RAMDISK_IMG" "$BUILD_DIR/media/nls/c_1252.nls"                          ::/reactos/system32/c_1252.nls
if [ -n "$L_INTL_NLS" ]; then
    mcopy -i "$RAMDISK_IMG" "$L_INTL_NLS"                                          ::/reactos/system32/l_intl.nls
fi

# Fonts (loaded by IntLoadSystemFonts from \SystemRoot\Fonts\)
mmd -i "$RAMDISK_IMG" ::reactos/Fonts
mcopy -o -i "$RAMDISK_IMG" "$SOURCE_DIR/media/fonts/FreeSans.ttf"                   ::reactos/Fonts/FreeSans.ttf

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
