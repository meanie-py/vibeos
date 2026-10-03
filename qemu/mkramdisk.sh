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

# Ramdisk size in MB (must hold all system files ~120MB + overhead).
#
# VIBEOS_FEX=1 needs a BIGGER image, and 140 is not enough for it any more: that
# image now carries the stock-app closure (ramdisk-apps.txt, ~27 MB stripped,
# added 2026-09-28) AND the SysWOW64 closure (~34.5 MB, 58 modules), which never
# had to coexist before. At 140 MB the FEX stage dies mid-copy with mtools'
# "Disk full" after staging only part of the closure — and a HALF-STAGED SysWOW64
# is the exact failure the closure check upstream of here exists to prevent,
# because a half-bound module crashes at first call far from the missing DLL.
# Non-FEX arm64 boots keep 140 MB: the ramdisk is loaded into RAM and the arm64
# VM only has 1G, so the extra is charged only to the runs that need it.
if [ "${VIBEOS_FEX:-0}" = "1" ]; then
    RAMDISK_SIZE_MB=200
else
    RAMDISK_SIZE_MB=140
fi
# Boot image must hold uefildr + freeldr.ini + ramdisk.img. DERIVED, not a second
# constant to keep in sync: the two drifted apart is how a ramdisk bump would
# silently produce a boot.img too small to hold it.
BOOT_SIZE_MB=$((RAMDISK_SIZE_MB + 140))

# --refresh: fast re-stage of freshly-built artifacts into the EXISTING
# ramdisk.img (skip the slow dd/mformat + FAT (re)creation), then fold into
# boot.img. This is the arm64 structural equivalent of `ninja livecd` for
# i386/amd64 and is what the smoke conftest calls so arm64 never boots stale
# binaries. RMC() centralises the ramdisk mcopy so a single `-o` makes every
# populate call idempotent (harmless on a fresh image, overwrite on refresh)
# — zero list drift vs the full build path (same calls run either way).
REFRESH=""
UP=""    # --up: boot-select the UP entry. The MP pair is the DEFAULT (2026-09-12);
         # this is the only way to get a UP boot, and nothing automated should use it.

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
        --up)
            UP=1
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
    # The MP pair is REQUIRED, not optional, because the ramdisk now stages it on
    # every build and the shipped freeldr.ini offers a [ReactOS_Arm64_MP] entry
    # that would otherwise name files that aren't there. Same contract as the x86
    # livecd, which has always staged all four binaries. `ntkrnlmp` and `halmp` are
    # best-effort NINJA targets (archcfg.ninja_targets_optional) — so if either one
    # failed to build, say so here instead of quietly shipping media whose MP entry
    # cannot boot.
    "$BUILD_DIR/ntoskrnl/ntkrnlmp/ntkrnlmp.exe"
    "$BUILD_DIR/hal/halarm64/halmp.dll"
    "$BUILD_DIR/drivers/base/kdrosdbg/kdcom.dll"
    "$BUILD_DIR/drivers/base/bootvid/bootvid.dll"
    "$BUILD_DIR/dll/ntdll/ntdll.dll"
    # All five CM hives. A hive that is missing here does NOT fail the boot: the
    # kernel fabricates an empty volatile one in its place (see the copy list
    # below). So every hive the livecd stages into system32/config must be listed
    # here too, and tools/scan_boot_media_drift.py gates exactly that.
    "$BUILD_DIR/boot/bootdata/system"
    "$BUILD_DIR/boot/bootdata/default"
    "$BUILD_DIR/boot/bootdata/software"
    "$BUILD_DIR/boot/bootdata/sam"
    "$BUILD_DIR/boot/bootdata/security"
    "$BUILD_DIR/drivers/storage/class/ramdisk/ramdisk.sys"
    "$BUILD_DIR/drivers/filesystems/fastfat/fastfat.sys"
    "$BUILD_DIR/base/system/smss/smss.exe"
    "$BUILD_DIR/subsystems/csr/csrss/csrss.exe"
    "$BUILD_DIR/subsystems/csr/csrsrv/csrsrv.dll"
    "$BUILD_DIR/subsystems/win/basesrv/basesrv.dll"
    "$BUILD_DIR/media/nls/c_437.nls"
    "$BUILD_DIR/media/nls/c_1252.nls"
    "$BUILD_DIR/media/nls/c_20127.nls"
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
    # usbhub [-> usbccgp for composite devices] -> hidusb -> hidclass -> kbdhid/mouhid -> kbdclass/mouclass; xHCI via usbxhci). The
    # PCI root devnode is seeded by hivesys_arm64.inf (Root\*PNP0A03).
    "$BUILD_DIR/drivers/bus/pci/pci.sys"
    "$BUILD_DIR/drivers/usb/usbd/usbd.sys"
    "$BUILD_DIR/drivers/usb/usbport/usbport.sys"
    "$BUILD_DIR/drivers/usb/usbehci/usbehci.sys"
    "$BUILD_DIR/drivers/usb/usbxhci/usbxhci.sys"
    "$BUILD_DIR/drivers/usb/usbhub/usbhub.sys"
    "$BUILD_DIR/drivers/usb/usbccgp/usbccgp.sys"
    # USB mass storage: usbstor -> disk (classpnp) with the partmgr class filter,
    # and mountmgr for drive letters. The hive already names all of these as boot
    # drivers; without the files the loader drops them and a stick enumerates
    # with no driver.
    "$BUILD_DIR/drivers/usb/usbstor/usbstor.sys"
    "$BUILD_DIR/drivers/storage/class/classpnp/classpnp.sys"
    "$BUILD_DIR/drivers/storage/class/disk/disk.sys"
    "$BUILD_DIR/drivers/storage/partmgr/partmgr.sys"
    "$BUILD_DIR/drivers/storage/mountmgr/mountmgr.sys"
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
    "$BUILD_DIR/dll/branding/rosbrand/rosbrand.dll"
    "$BUILD_DIR/dll/win32/browseui/browseui.dll"
    "$BUILD_DIR/base/shell/rshell/rshell.dll"
    "$BUILD_DIR/dll/win32/shlwapi/shlwapi.dll"
    "$BUILD_DIR/dll/win32/version/version.dll"
    "$BUILD_DIR/dll/win32/uxtheme/uxtheme.dll"
    # Level 2 transitive deps
    "$BUILD_DIR/dll/win32/winmm/winmm.dll"
    "$BUILD_DIR/dll/win32/usp10/usp10.dll"
    # gdi32's LoadLPK retries LoadLibraryW(L"lpk.dll") on EVERY ExtTextOut /
    # GetTextExtentExPoint until it succeeds, so without it each glyph run
    # walks the whole DLL search path (seen on a Pi 4: explorer spent its tray
    # clock repaints in RtlDosSearchPath_U).
    "$BUILD_DIR/dll/win32/lpk/lpk.dll"
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

# Which payload dests are in ramdisk.img right now. --refresh only ever ADDS
# files, so without this a payload staged once stayed on every later boot of
# this tree: a payload-less arm64 build still autostarted kmtest-sweep.exe
# (and a regdiff.exe from a different day), whose 91-test sweep ran through
# every boot the start-menu gate saw. The payload stage below deletes what
# this lists and the current selection does not.
PAYLOAD_RECORD="$QEMU_DIR/ramdisk.payloads"
if [ -n "$REFRESH" ] && [ -f "$RAMDISK_IMG" ] && [ ! -f "$PAYLOAD_RECORD" ]; then
    echo "[*] --refresh: no $PAYLOAD_RECORD for the existing ramdisk.img, so its payloads are unknown -- recreating it"
    REFRESH=""
fi

# --- Create ramdisk.img (skipped under --refresh: reuse existing FAT image) ---
if [ -n "$REFRESH" ]; then
    if [ ! -f "$RAMDISK_IMG" ]; then
        echo "[!] --refresh needs an existing $RAMDISK_IMG (run without --refresh once)"
        exit 1
    fi
    # A cached image of the WRONG SIZE must not be reused. RAMDISK_SIZE_MB is
    # conditional (see above), so flipping VIBEOS_FEX changes the required size
    # while --refresh would happily keep the old file and then fail with mtools'
    # "Disk full" — which names neither the size nor the cache as the cause.
    # Detect it here instead of asking anyone to remember to rm the image.
    WANT_BYTES=$((RAMDISK_SIZE_MB * 1024 * 1024))
    HAVE_BYTES=$(stat -c %s "$RAMDISK_IMG")
    if [ "$HAVE_BYTES" != "$WANT_BYTES" ]; then
        echo "[*] --refresh: existing ramdisk.img is ${HAVE_BYTES}B but this run needs" \
             "${WANT_BYTES}B (${RAMDISK_SIZE_MB}MB, VIBEOS_FEX=${VIBEOS_FEX:-0}) — recreating"
        REFRESH=""
    else
        echo "[*] --refresh: reusing existing ramdisk.img (re-staging artifacts only)"
    fi
fi
if [ -z "$REFRESH" ]; then
    echo "[*] Creating ramdisk.img (${RAMDISK_SIZE_MB}MB FAT32)..."
    rm -f "$RAMDISK_IMG"
    : > "$PAYLOAD_RECORD"
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

# Profile tree — livecd parity. The x86 LiveCD stages Profiles/ onto the ISO
# (boot/bootdata/CMakeLists.txt) and the livecd hives point ProfilesDirectory
# at %SystemDrive%\Profiles; shell32's CSIDL resolution (and therefore
# explorer's start menu — CStartMenu GetMergedFolder) needs the directories
# to actually exist. Without these the start menu object silently fails to
# build (E_FAIL) and Ctrl+Esc opens nothing. Idempotent: mmd errors on
# existing dirs are expected on --refresh of an existing image.
echo "[*] Staging profile tree..."
while IFS= read -r d; do
    # `mmd` prompts when the directory already exists and stdout is a tty.
    # Refresh runs under vibeos-cli's PTY, so probe first just like Fonts below.
    mdir -i "$RAMDISK_IMG" "$d" >/dev/null 2>&1 \
        || mmd -i "$RAMDISK_IMG" "$d"
done <<'PROFILES'
::/Profiles
::/Profiles/All Users
::/Profiles/All Users/Desktop
::/Profiles/All Users/Start Menu
::/Profiles/All Users/Start Menu/Programs
::/Profiles/All Users/Start Menu/Programs/Administrative Tools
::/Profiles/All Users/Start Menu/Programs/Accessories
::/Profiles/Default User
::/Profiles/Default User/Desktop
::/Profiles/Default User/My Documents
::/Profiles/Default User/Start Menu
::/Profiles/Default User/Start Menu/Programs
PROFILES

# Copy system files
echo "[*] Populating ramdisk..."
RMC "$BUILD_DIR/ntoskrnl/ntoskrnl.exe"                        ::/reactos/system32/ntoskrnl.exe
# SMP PAIR (M5): ntkrnlmp.exe (lives in the ntkrnlmp/ subdir) + halmp.dll, the
# hal-rust `smp` twin. Staged UNCONDITIONALLY, exactly like the x86 livecd stages
# all four binaries on every build: the freeldr.ini shipped below carries both a
# [ReactOS_Arm64_UP] and a [ReactOS_Arm64_MP] entry, so which kernel boots is a
# DefaultOS= pick against fixed media rather than a property of how the media was
# built. That is what lets `smoke/conformance --smp N` boot-select the MP pair by
# poking a copy of boot.img (vm.smp_selected_iso) instead of demanding a rebuild.
#
# Conditional staging was its own hazard: forgetting --smp produced media that
# looked identical and silently booted UP, which is the exact false-green
# _smp_check exists to catch.
#
# Staging BOTH is mandatory on arm64, not a nicety: arm64 has no MISMATCHED_HAL
# handshake, so ntkrnlmp.exe on the UP hal.dll boots to a full desktop and
# silently runs on one cpu (HalStartNextProcessor is the FALSE stub). That trap
# cost a debug round already — see smp-bringup-quest.md M5c.
RMC "$BUILD_DIR/ntoskrnl/ntkrnlmp/ntkrnlmp.exe"               ::/reactos/system32/ntkrnlmp.exe
RMC "$BUILD_DIR/hal/halarm64/halmp.dll"                       ::/reactos/system32/halmp.dll
RMC "$BUILD_DIR/hal/halarm64/hal.dll"                          ::/reactos/system32/hal.dll
RMC "$BUILD_DIR/drivers/base/kdrosdbg/kdcom.dll"               ::/reactos/system32/kdcom.dll
RMC "$BUILD_DIR/drivers/base/bootvid/bootvid.dll"              ::/reactos/system32/bootvid.dll
RMC "$BUILD_DIR/dll/ntdll/ntdll.dll"                           ::/reactos/system32/ntdll.dll
# All FIVE hives mkhive emits, matching the livecd's add_cd_file list
# (sdk/cmake/CMakeMacros.cmake). sam and security were missing here for a long
# time and the failure is SILENT: CmpLoadHiveThread can't open the file, sets
# Allocate, and CmpLinkHiveToMaster fabricates an empty volatile hive whose root
# key gets CmpHiveRootSecurityDescriptor(). So \REGISTRY\MACHINE\SAM existed and
# opened fine on arm64 while holding no accounts at all -- and it read as
# *more* correct than x86 in CmSecurity, because the fabricated descriptor is
# exactly what that test expects. Staging drift here does not announce itself.
RMC "$BUILD_DIR/boot/bootdata/system"                          ::/reactos/system32/config/system
RMC "$BUILD_DIR/boot/bootdata/default"                         ::/reactos/system32/config/default
RMC "$BUILD_DIR/boot/bootdata/software"                        ::/reactos/system32/config/software
RMC "$BUILD_DIR/boot/bootdata/sam"                             ::/reactos/system32/config/sam
RMC "$BUILD_DIR/boot/bootdata/security"                        ::/reactos/system32/config/security
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
# ucrtbase.dll — staged UNCONDITIONALLY, same as msvcrt.dll above. It used to be
# staged only under VIBEOS_FEX=1, which meant a normal arm64 boot had no ucrtbase on
# the media at all: the DLL is built and IS in the livecd graft list, but arm64 boots
# boot.img and never the livecd, so nothing noticed. Any exe importing it then failed
# to start with NOTHING on serial -- the loader's ERROR_MOD_NOT_FOUND arrives before
# the first instruction. Cost a night to find; the reproducer is a bare-entry exe that
# runs with no ucrtbase import and dies silently with one.
RMC "$BUILD_DIR/dll/win32/ucrtbase/ucrtbase.dll"              ::/reactos/system32/ucrtbase.dll
RMC "$BUILD_DIR/dll/win32/kernel32/kernel32_vista/kernel32_vista.dll"  ::/reactos/system32/kernel32_vista.dll
RMC "$BUILD_DIR/dll/win32/kernelbase/kernelbase.dll"          ::/reactos/system32/kernelbase.dll
# wow64.dll — WOW64 host-layer skeleton (docs/engineering/wow64-plan.md);
# libwow64fex.dll imports it, so it must stage wherever the FEX layer does.
RMC "$BUILD_DIR/dll/win32/wow64/wow64.dll"                    ::/reactos/system32/wow64.dll
# win32u.dll — native NtGdi*/NtUser* syscall stubs; wow64.dll's win32k thunk
# table imports it (M4/wow64win).
RMC "$BUILD_DIR/win32ss/win32u/win32u.dll"                    ::/reactos/system32/win32u.dll
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
RMC "$BUILD_DIR/drivers/usb/usbxhci/usbxhci.sys"           ::/reactos/system32/drivers/usbxhci.sys
RMC "$BUILD_DIR/drivers/usb/usbhub/usbhub.sys"             ::/reactos/system32/drivers/usbhub.sys
RMC "$BUILD_DIR/drivers/usb/usbccgp/usbccgp.sys"           ::/reactos/system32/drivers/usbccgp.sys
RMC "$BUILD_DIR/drivers/usb/usbstor/usbstor.sys"           ::/reactos/system32/drivers/usbstor.sys
RMC "$BUILD_DIR/drivers/storage/class/classpnp/classpnp.sys" ::/reactos/system32/drivers/classpnp.sys
RMC "$BUILD_DIR/drivers/storage/class/disk/disk.sys"       ::/reactos/system32/drivers/disk.sys
RMC "$BUILD_DIR/drivers/storage/partmgr/partmgr.sys"       ::/reactos/system32/drivers/partmgr.sys
RMC "$BUILD_DIR/drivers/storage/mountmgr/mountmgr.sys"     ::/reactos/system32/drivers/mountmgr.sys
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
# Side-by-side store, mirroring comctl32's add_cd_file rules for the livecd. comctl32
# creates its v5/v6 activation contexts from %windir%\winsxs\manifests, and apps whose
# manifest asks for Common-Controls 6.0 resolve it here. Without it every
# CreateActCtxW in comctl32 failed on arm64.
# %SystemRoot%\TEMP: hivesys.inf points the system TEMP/TMP there, and x86 setup
# creates it, but nothing made it on the arm64 ramdisk -- so every app writing a
# temp file failed with ERROR_PATH_NOT_FOUND (found by the ucrt Tier 2 findfile
# fixture, whose CreateDirectoryW under %TEMP% failed with error 3).
mdir -i "$RAMDISK_IMG" ::/reactos/TEMP >/dev/null 2>&1 || mmd -i "$RAMDISK_IMG" ::/reactos/TEMP
SXS_CC="microsoft.windows.common-controls_6595b64144ccf1df"
for d in winsxs winsxs/manifests \
         "winsxs/arm64_${SXS_CC}_5.82.2600.2982_none_deadbeef" \
         "winsxs/arm64_${SXS_CC}_6.0.2600.2982_none_deadbeef"; do
    mdir -i "$RAMDISK_IMG" "::/reactos/$d" >/dev/null 2>&1 || mmd -i "$RAMDISK_IMG" "::/reactos/$d"
done
for v in 5.82.2600.2982 6.0.2600.2982; do
    RMC "$BUILD_DIR/dll/win32/comctl32/comctl32.dll" "::/reactos/winsxs/arm64_${SXS_CC}_${v}_none_deadbeef/comctl32.dll"
    mcopy -o -i "$RAMDISK_IMG" "$SOURCE_DIR/dll/win32/comctl32/arm64_${SXS_CC}_${v}_none_deadbeef.manifest" \
        "::/reactos/winsxs/manifests/arm64_${SXS_CC}_${v}_none_deadbeef.manifest"
done
mcopy -o -i "$RAMDISK_IMG" "$SOURCE_DIR/media/systemcompatible.manifest" ::/reactos/winsxs/manifests/systemcompatible.manifest
RMC "$BUILD_DIR/dll/win32/ole32/ole32.dll"                     ::/reactos/system32/ole32.dll
RMC "$BUILD_DIR/dll/win32/oleaut32/oleaut32.dll"               ::/reactos/system32/oleaut32.dll
RMC "$BUILD_DIR/dll/win32/shell32/shell32.dll"                 ::/reactos/system32/shell32.dll
RMC "$BUILD_DIR/dll/branding/rosbrand/rosbrand.dll"            ::/reactos/system32/rosbrand.dll
RMC "$BUILD_DIR/dll/win32/browseui/browseui.dll"               ::/reactos/system32/browseui.dll
RMC "$BUILD_DIR/base/shell/rshell/rshell.dll"                  ::/reactos/system32/rshell.dll
RMC "$BUILD_DIR/dll/win32/shlwapi/shlwapi.dll"                 ::/reactos/system32/shlwapi.dll
RMC "$BUILD_DIR/dll/win32/version/version.dll"                 ::/reactos/system32/version.dll
RMC "$BUILD_DIR/dll/win32/uxtheme/uxtheme.dll"                 ::/reactos/system32/uxtheme.dll
# Level 2 transitive deps
RMC "$BUILD_DIR/dll/win32/winmm/winmm.dll"                    ::/reactos/system32/winmm.dll
RMC "$BUILD_DIR/dll/win32/usp10/usp10.dll"                    ::/reactos/system32/usp10.dll
RMC "$BUILD_DIR/dll/win32/lpk/lpk.dll"                        ::/reactos/system32/lpk.dll
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
# US-ASCII. msvcrt's _setmbcp(_MB_CP_SBCS) selects it (Wine's mapping), and without the
# table every MultiByteToWideChar(20127) fails -- so every _ismbc*/_mbcto* call answered
# 0 even for 'A' on arm64, the only arch whose media is curated here (x86 ships all NLS).
RMC "$BUILD_DIR/media/nls/c_20127.nls"                         ::/reactos/system32/c_20127.nls
if [ -n "$L_INTL_NLS" ]; then
    RMC "$L_INTL_NLS"                                          ::/reactos/system32/l_intl.nls
fi

# Fonts (loaded by IntLoadSystemFonts from \SystemRoot\Fonts\)
# Under --refresh, mmd can prompt on an existing directory when stdout is a tty.
# Probe first so refresh stays non-interactive under vibeos-cli.
mdir -i "$RAMDISK_IMG" ::reactos/Fonts >/dev/null 2>&1 || mmd -i "$RAMDISK_IMG" ::reactos/Fonts
mcopy -o -i "$RAMDISK_IMG" "$SOURCE_DIR/media/fonts/FreeSans.ttf"                   ::reactos/Fonts/FreeSans.ttf
# win32k loads every file in this directory at boot. The shell needs the UI
# faces (without them the taskbar never paints text); consrv needs "VGA"
# (UniVGA16.ttf), its default face, or no console window can be created, and
# the failed creation wedges csrss. Lucida Console is the one console TrueType
# face the hive lists.
for FONT in tahoma.ttf tahomabd.ttf Marlett.ttf arial.ttf arialbd.ttf cour.ttf times.ttf symbol.ttf \
            UniVGA16.ttf lucon.ttf; do
    if [ -f "$SOURCE_DIR/media/fonts/$FONT" ]; then
        mcopy -o -i "$RAMDISK_IMG" "$SOURCE_DIR/media/fonts/$FONT"                  "::reactos/Fonts/$FONT"
    fi
done

# --- kmtest corpus (the arm64 twin of add_rostests_file) --------------------
# x86 stages the kmtests onto the LiveCD declaratively: every kmtest driver
# calls add_rostests_file(TARGET x), which is add_cd_file(... DESTINATION
# "reactos/bin") (sdk/cmake/CMakeMacros.cmake). arm64 has no cd-file pass, so
# without this block `vibeos-cli conformance --arch arm64` boots an image with
# the sweep runner but no corpus for it to run -> [KMTEST-SWEEP] launch FAILED.
#
# reactos/bin, and ALL of it in that ONE directory, is load-bearing rather than
# cosmetic: KmtpCreateService resolves each driver by taking
# GetModuleFileName(kmtest_.exe), truncating at the last backslash, and
# appending "<name>_drv.sys" (kmtest/service.c:428-439, which then
# GetFileAttributes()-checks it). A driver staged anywhere else is simply never
# found, so the flat layout mirrors x86 exactly.
#
# Globbed, not enumerated, so STAGING can't drift (same rationale as RMC
# above). Note the glob can only stage what got BUILT, and that list is still
# hand-maintained upstream: kmtest_all -> kmtest_drivers in kmtests/
# CMakeLists.txt, which today omits the cc*/fltmgr drivers (x86 doesn't notice
# — its livecd DAG builds every add_rostests_file target). That's harmless
# while no visible test needs them (none of the 3 sweep lists reference a
# Cc*/Flt* test), but if you make one visible, add its driver to
# kmtest_drivers or arm64 will silently sweep without it.
# Self-gating on the artifacts: they exist only when the build dir is
# configured -DENABLE_ROSTESTS(_KMTESTS)=ON, which is precisely when x86's
# add_rostests_file fires — so an unconfigured build stages nothing and the
# ramdisk stays byte-identical. archcfg.py builds `kmtest_all` (optional) so
# what lands here is freshly built, never a stale leftover.
# npfs.sys + services.exe — the SCM chain the kmtest corpus needs.
# Every kmtest loads its driver via CreateService/StartService, which is
# advapi32 -> RPC ncacn_np "\pipe\ntsvcs" -> services.exe (scm.c:42-44,
# services/rpcserver.c:115). So the sweep needs BOTH the named-pipe FS and the
# SCM itself, neither of which arm64 has ever staged: without them every test
# raises RPC_S_SERVER_UNAVAILABLE (0x6BA) before its first print and the sweep
# only sees a 180s timeout. Npfs is already registered Start=1 in the arm64
# SYSTEM hive; it just needs the binary here + an entry in iomgr's arm64
# STAGED_SVCS (the arm64 boot hand-loads an explicit list instead of the bulk
# Start=1 enumeration). services.exe's whole import set (advapi32, rpcrt4,
# user32, userenv, msvcrt, ntdll, kernel32, advapi32_vista) is already staged
# above. lsass is deliberately NOT staged: kmtest needs the SCM, not the LSA,
# and lsass would drag in lsasrv/samsrv.
RMC "$BUILD_DIR/drivers/filesystems/npfs/npfs.sys"             ::/reactos/system32/drivers/npfs.sys
RMC "$BUILD_DIR/base/system/services/services.exe"             ::/reactos/system32/services.exe
# lsass is NOT optional once services.exe runs: ScmCreateServiceDatabase ends in
# ScmWaitForLsa() = WaitForSingleObject(LSA_RPC_SERVER_ACTIVE, INFINITE), and
# only lsass.exe ever signals that event (services-rust/src/services.rs:253). An
# SCM without an LSA hangs forever mid-database (measured: stuck 420s+ after
# "LastKnownGood OK"), so SC_AutoStartComplete never fires and every kmtest times
# out. lsasrv/samsrv are the only 2 of lsass's 15 transitive DLLs not already
# staged above.
RMC "$BUILD_DIR/base/system/lsass/lsass.exe"                   ::/reactos/system32/lsass.exe
RMC "$BUILD_DIR/dll/win32/lsasrv/lsasrv.dll"                   ::/reactos/system32/lsasrv.dll
RMC "$BUILD_DIR/dll/win32/samsrv/samsrv.dll"                   ::/reactos/system32/samsrv.dll
# msv1_0 = the MSV1_0 authentication package. lsasrv loads it BY NAME at runtime
# (the Lsa "Authentication Packages" registry value), so it is invisible to an
# import-table walk — lsass.exe's every STATIC import resolves and it still died
# with ExitStatus=0xC0000135 (STATUS_DLL_NOT_FOUND) propagated as its exit code.
RMC "$BUILD_DIR/dll/win32/msv1_0/msv1_0.dll"                   ::/reactos/system32/msv1_0.dll

# The stock apps and control-panel applets (qemu/ramdisk-apps.txt), with their whole
# import closure, at their livecd destinations. Derived, not hand-listed: before this the
# ramdisk carried notepad/cmd/explorer only and appstorm found 36 of 43 apps ABSENT on
# arm64. `vibeos-cli build --arch arm64` builds the closure; ramdisk_apps.py HARD-FAILS on
# an unbuilt or unresolvable member rather than stage a half-bound module. Modules staged
# by hand above are left out of its list, so the curated copy stays authoritative.
echo "[*] Staging stock apps (qemu/ramdisk-apps.txt closure)..."
APPS_LST="$BUILD_DIR/boot/ramdisk-apps.lst"
python3 "$PROJECT_DIR/tools/ramdisk_apps.py" --stage --build-dir "$BUILD_DIR" > "$APPS_LST" || {
    echo "[!] ramdisk app closure is incomplete (see above) -- run ./vibeos-cli build --arch arm64"
    exit 1
}
python3 "$PROJECT_DIR/tools/strip_cd_media.py" --lst "$APPS_LST" --out "$APPS_LST.stripped" \
    --cache "$STRIP_CACHE" --objcopy "$STRIP_OBJCOPY" || exit 1
while IFS='=' read -r dest src; do
    [ -n "$dest" ] || continue
    mcopy -o -i "$RAMDISK_IMG" "$src" "::/$dest"
done < "$APPS_LST.stripped"

KMTESTS_DIR="$BUILD_DIR/modules/rostests/kmtests"
if [ -f "$KMTESTS_DIR/kmtest_.exe" ]; then
    echo "[*] Staging kmtest corpus..."
    mdir -i "$RAMDISK_IMG" ::/reactos/bin >/dev/null 2>&1 \
        || mmd -i "$RAMDISK_IMG" ::/reactos/bin
    RMC "$KMTESTS_DIR/kmtest_.exe" ::/reactos/bin/kmtest_.exe
    # Flatten: co-located next to kmtest_.exe regardless of source subdir.
    while IFS= read -r drv; do
        RMC "$drv" "::/reactos/bin/$(basename "$drv")"
    done < <(find "$KMTESTS_DIR" -name '*_drv.sys' | sort)
fi

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
# Drop payloads staged by an earlier run that the current selection omits.
PAYLOAD_DESTS=$(printf '%s\n' "$PAYLOAD_MANIFEST" | sed -n 's/=.*//p')
while IFS= read -r OLD; do
    [ -n "$OLD" ] || continue
    if ! printf '%s\n' "$PAYLOAD_DESTS" | grep -qxF -- "$OLD"; then
        echo "[*]   removing stale payload ::/$OLD"
        mdel -i "$RAMDISK_IMG" "::/$OLD" 2>/dev/null || true
    fi
done < "$PAYLOAD_RECORD"
# Recorded before the copies, so an interrupted stage still lists what it wrote.
printf '%s\n' "$PAYLOAD_DESTS" | sed '/^$/d' > "$PAYLOAD_RECORD"
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

# --- FEX CPU layer (DEFAULT ON) --------------------------------------------
# libwow64fex.dll is staged on EVERY arm64 image, not behind VIBEOS_FEX=1.
#
# Owner decision 2026-09-13: "if even the fex lane is forgetting to stage and
# smoke fex then the build env-var is just a way for us to embarrass ourselves."
# The gate was hiding the lane from itself.
#
# This half is free: 4.5M of ramdisk and ZERO boot time, because nothing maps
# the CPU layer until a 32-bit process starts. It is deliberately separated from
# staging the GUESTS, which is NOT free -- guests add ~85s of ladder drain to
# every arm64 gate for every lane, and until the FEX code-cache fault (roadmap:
# LookupCache::FindBlock / InvalidateThreadCachedCodeRange) has a measured flake
# rate, that cost comes with a known-faulting emulator attached. Guests stay
# behind VIBEOS_FEX=1 below until that number exists.
#
# Existence-gated, and the skip is LOUD: a missing CPU layer now means every
# arm64 image silently loses 32-bit support, which is exactly the failure the
# env-var used to cause on purpose.
WOW64FEX_DEFAULT="${VIBEOS_WOW64FEX_DLL:-$PROJECT_DIR/build-fex-arm64/Bin/libwow64fex.dll}"
if [ -f "$WOW64FEX_DEFAULT" ]; then
    echo "[*] staging libwow64fex.dll into system32 (FEX CPU layer, default-on)"
    mcopy -o -i "$RAMDISK_IMG" "$WOW64FEX_DEFAULT" ::/reactos/system32/libwow64fex.dll
else
    echo "[!] FEX CPU layer MISSING ($WOW64FEX_DEFAULT) — this image has NO 32-bit"
    echo "    support at all. Build it with: tools/fex/build-fex.sh"
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
        # WOW64 Phase A: the FEX CPU layer is staged unconditionally now (see the
        # default-on block above), so there is nothing to do here.
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
        # M3b: the guest loader always maps kernel32 (BaseThreadInitThunk et al).
        # \KnownDlls32 doesn't exist on arm64, so the loader falls back to the
        # file path, which wow64.dll redirects system32 -> syswow64 — stage the
        # i386 kernel32 + its ntdll_vista dep there.
        KERNEL32_32="${VIBEOS_KERNEL32_32:-$PROJECT_DIR/build-i386/dll/win32/kernel32/kernel32.dll}"
        NTDLLVISTA32="${VIBEOS_NTDLLVISTA32:-$PROJECT_DIR/build-i386/dll/ntdll/nt_0600/ntdll_vista.dll}"
        if [ -f "$KERNEL32_32" ] && [ -f "$NTDLLVISTA32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/kernel32.dll + ntdll_vista.dll (i386)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$KERNEL32_32" ::/reactos/syswow64/kernel32.dll
            mcopy -o -i "$RAMDISK_IMG" "$NTDLLVISTA32" ::/reactos/syswow64/ntdll_vista.dll
        else
            echo "[!] VIBEOS_FEX=1: i386 kernel32/ntdll_vista missing ($KERNEL32_32 / $NTDLLVISTA32) — guest loader will stall at kernel32"
        fi
        # M4/wow64win: the 32-bit GUI-DLL set for gui32.exe (gdi32 + user32 +
        # their import closure) — the guest loader pulls whatever the image
        # imports; missing deps show up as loud NtOpenFile c0000034 lines.
        GDI32_32="${VIBEOS_GDI32_32:-$PROJECT_DIR/build-i386/win32ss/gdi/gdi32/gdi32.dll}"
        USER32_32="${VIBEOS_USER32_32:-$PROJECT_DIR/build-i386/win32ss/user/user32/user32.dll}"
        if [ -f "$GDI32_32" ] && [ -f "$USER32_32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/gdi32.dll + user32.dll (i386)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$GDI32_32" ::/reactos/syswow64/gdi32.dll
            mcopy -o -i "$RAMDISK_IMG" "$USER32_32" ::/reactos/syswow64/user32.dll
        else
            echo "[!] VIBEOS_FEX=1: i386 gdi32/user32 missing — gui32 guest will stall at import"
        fi
        # M11: the FULL SysWOW64 DLL set — the transitive import closure of the
        # stock i386 guest (testgui.exe), computed at stage time.
        #
        # This SUPERSEDES the M8 rule ("do NOT stage advapi32/rpcrt4/msvcrt —
        # they leave unbound IATs"). That rule was right about the hazard and
        # wrong about the cure: when a DLL is staged but one of ITS imports is
        # missing, the loader prints "Skipping missing import" and leaves that
        # module half-bound; the first call through an unbound slot jumps to
        # garbage (guest AV @0x76AA4, 2026-07-11). Not-staging avoided it only
        # because nothing on the M8 path called into those DLLs. A CRT-linked
        # binary DOES call them, so the answer is the COMPLETE closure — no
        # module is ever left half-bound. A bigger PARTIAL set has the same
        # landmine, just later. wow64-closure.py hard-fails rather than emit a
        # partial set, and the loader's "Skipping missing import" line is the
        # standing oracle that the closure is still complete.
        TESTGUI32="${VIBEOS_TESTGUI32_EXE:-$PROJECT_DIR/build-i386/base/applications/testgui/testgui.exe}"
        if [ -f "$TESTGUI32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging the SysWOW64 DLL closure (i386)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            # Roots-only literal scan, NOT --library-literals: that adds ~25 MB
            # (stripped) and this ramdisk is a fixed ${RAMDISK_SIZE_MB} MB image.
            CLOSURE=$(python3 "$PROJECT_DIR/tools/fex/wow64-closure.py" \
                          --build-dir "$PROJECT_DIR/build-i386" "$TESTGUI32") || {
                echo "[!] VIBEOS_FEX=1: SysWOW64 import closure is INCOMPLETE (see above)."
                echo "[!]   Refusing to stage a partial set — a half-bound module crashes"
                echo "[!]   at first call, far from the missing DLL. Build the module first."
                exit 1
            }
            echo "$CLOSURE" | while IFS=$'\t' read -r name path; do
                [ -n "$name" ] || continue
                mcopy -o -i "$RAMDISK_IMG" "$path" "::/reactos/syswow64/$name"
            done
            echo "[*]   staged $(echo "$CLOSURE" | wc -l) SysWOW64 modules"
            # testgui.exe — the M11 guest: a STOCK, unmodified ReactOS i386 GUI
            # app (real msvcrt CRT startup, real import table, real window +
            # message loop). Every earlier guest was one we wrote to dodge the
            # gaps; this one can't dodge anything.
            echo "[*] VIBEOS_FEX=1 — staging syswow64/testgui.exe (M11 stock-binary guest)"
            mcopy -o -i "$RAMDISK_IMG" "$TESTGUI32" ::/reactos/syswow64/testgui.exe
            # The NATIVE arm64 build of the same app, into system32. This is the
            # A/B control: same source, same user32-rust, one running native and
            # one under WoW64. Without it there is no way to tell a WoW64 bug
            # from a plain user32 bug that would crash natively too.
            TESTGUI_NATIVE="${VIBEOS_TESTGUI_NATIVE:-$PROJECT_DIR/build-arm64/base/applications/testgui/testgui.exe}"
            if [ -f "$TESTGUI_NATIVE" ]; then
                echo "[*] VIBEOS_FEX=1 — staging system32/testgui.exe (native A/B control)"
                mcopy -o -i "$RAMDISK_IMG" "$TESTGUI_NATIVE" ::/reactos/system32/testgui.exe
            fi

            # M12 — THE NORTH STAR: x86 Doom under WoW64. doomgeneric's Win32/GDI
            # backend built for i386 (tools/doom/build.sh i386): a genuinely foreign
            # mingw binary. Its import closure is the SAME 53 modules staged above,
            # so nothing extra is needed beyond the exe + the IWAD.
            DOOM32="${VIBEOS_DOOM32_EXE:-$PROJECT_DIR/tools/doom/doom-i386.exe}"
            DOOMWAD="${VIBEOS_DOOM_WAD:-$PROJECT_DIR/tools/doom/doom1.wad}"
            if [ -f "$DOOM32" ] && [ -f "$DOOMWAD" ]; then
                echo "[*] VIBEOS_FEX=1 — staging syswow64/doom.exe + doom1.wad (M12: x86 DOOM)"
                mcopy -o -i "$RAMDISK_IMG" "$DOOM32" ::/reactos/syswow64/doom.exe
                mcopy -o -i "$RAMDISK_IMG" "$DOOMWAD" ::/reactos/syswow64/doom1.wad
                # A/B control, same idea as the native testgui: the NATIVE arm64
                # build of the very same doomgeneric. VIBEOS_DOOM_AB=1 stages it
                # into system32 so both run side by side and a rendering failure
                # can be attributed to the WoW64 layer or to win32k.
                DOOM_NATIVE="${VIBEOS_DOOM_NATIVE:-$PROJECT_DIR/tools/doom/doom-arm64.exe}"
                if [ "${VIBEOS_DOOM_AB:-0}" = "1" ] && [ -f "$DOOM_NATIVE" ]; then
                    echo "[*] VIBEOS_DOOM_AB=1 — staging system32/doom.exe + doom1.wad (native A/B)"
                    mcopy -o -i "$RAMDISK_IMG" "$DOOM_NATIVE" ::/reactos/system32/doom.exe
                    mcopy -o -i "$RAMDISK_IMG" "$DOOMWAD" ::/reactos/system32/doom1.wad
                fi
            fi
        fi
        # gui32.exe — the win32k-thunk guest (winlogon TryStartWow64Gui).
        GUI32="${VIBEOS_GUI32_EXE:-$PROJECT_DIR/build-fex-hello/gui32.exe}"
        if [ -f "$GUI32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/gui32.exe (wow64win guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$GUI32" ::/reactos/syswow64/gui32.exe
        fi
        # win32.exe — the M8 window-creation guest (winlogon TryStartWow64Win32).
        WIN32APP="${VIBEOS_WIN32_EXE:-$PROJECT_DIR/build-fex-hello/win32.exe}"
        if [ -f "$WIN32APP" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/win32.exe (M8 window guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$WIN32APP" ::/reactos/syswow64/win32.exe
        fi
        # marshal32.exe — the M18 marshaller-verification guest (winlogon
        # TryStartWow64Marshal): TBI/IOCP/monitor marshallers, self-verified.
        MARSHAL32="${VIBEOS_MARSHAL32_EXE:-$PROJECT_DIR/build-fex-hello/marshal32.exe}"
        if [ -f "$MARSHAL32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/marshal32.exe (M18 marshaller guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$MARSHAL32" ::/reactos/syswow64/marshal32.exe
        fi
        # file32.exe — the M10 file-I/O guest (winlogon TryStartWow64File).
        FILE32="${VIBEOS_FILE32_EXE:-$PROJECT_DIR/build-fex-hello/file32.exe}"
        if [ -f "$FILE32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/file32.exe (M10 file-I/O guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$FILE32" ::/reactos/syswow64/file32.exe
        fi
        # async32.exe — the M15b overlapped-I/O guest (winlogon TryStartWow64Async):
        # kernel-side IO_STATUS_BLOCK32 completion oracle.
        ASYNC32="${VIBEOS_ASYNC32_EXE:-$PROJECT_DIR/build-fex-hello/async32.exe}"
        if [ -f "$ASYNC32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/async32.exe (M15b overlapped-I/O guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$ASYNC32" ::/reactos/syswow64/async32.exe
        fi
        # thread32.exe — the #64 threading guest (winlogon TryStartWow64Thread):
        # the per-thread WoW64 entry, driven here through FEX's BTCpu contract
        # rather than amd64's wow64cpu. Staging it on arm64 is the point — the
        # per-thread CPU area and TEB32 stack bounds are arch-neutral code whose
        # only other coverage is the amd64 CPU layer.
        THREAD32="${VIBEOS_THREAD32_EXE:-$PROJECT_DIR/build-fex-hello/thread32.exe}"
        if [ -f "$THREAD32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/thread32.exe (#64 threading guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$THREAD32" ::/reactos/syswow64/thread32.exe
        fi
        # fault32.exe — the E5/E18 guest-SEH guest (winlogon TryStartWow64Fault):
        # a hardware AV caught by a raw fs:[0] handler with the LIVE faulting
        # context, a RaiseException round-trip, and DBG_PRINTEXCEPTION
        # consumption. On arm64 the fault arrives as a HOST fault inside FEX's
        # JIT, so this exercises a completely different arrival path from
        # amd64's compat-mode trap.
        FAULT32="${VIBEOS_FAULT32_EXE:-$PROJECT_DIR/build-fex-hello/fault32.exe}"
        if [ -f "$FAULT32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/fault32.exe (E18 guest-SEH guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$FAULT32" ::/reactos/syswow64/fault32.exe
        fi
        # selfmod32.exe — the E17 runtime-codegen guest (winlogon
        # TryStartWow64Selfmod): the CPU layer's memory-notification contract.
        # This one is arm64-FIRST by nature — a native host has nothing to
        # invalidate, so FEX is the only place the contract does any work.
        SELFMOD32="${VIBEOS_SELFMOD32_EXE:-$PROJECT_DIR/build-fex-hello/selfmod32.exe}"
        if [ -f "$SELFMOD32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/selfmod32.exe (E17 codegen guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$SELFMOD32" ::/reactos/syswow64/selfmod32.exe
        fi
        # reg32.exe and spawn32.exe — the last two PARITY guests, MISSING here
        # until 2026-09-19 while the x86 livecd staged them, so reg32 had NO
        # arm64 coverage from the day it landed and spawn32 would have had none.
        #
        # KEEP THIS LIST IN STEP WITH ladder.LADDER. The livecd cannot drift like
        # this because CMake's _wow64_ladder is all-or-nothing; here every mcopy
        # stands alone, so a guest is added to the parity set by editing two
        # files and forgetting one of them is silent.
        #
        # What the drift cost, measured: ladder.media_has_guests asks whether ANY
        # guest is aboard (deliberately -- a partial stage must opt IN so the gap
        # cannot pass quietly), so the gate ran the ladder and then waited out its
        # entire 420s drain for two verdicts that could never arrive. It failed as
        # a TIMEOUT rather than naming the two files. With them staged the same
        # boot drains in 87.1s.
        REG32="${VIBEOS_REG32_EXE:-$PROJECT_DIR/build-fex-hello/reg32.exe}"
        if [ -f "$REG32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/reg32.exe (registry-view guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$REG32" ::/reactos/syswow64/reg32.exe
        fi
        SPAWN32="${VIBEOS_SPAWN32_EXE:-$PROJECT_DIR/build-fex-hello/spawn32.exe}"
        if [ -f "$SPAWN32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/spawn32.exe (process-creation guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$SPAWN32" ::/reactos/syswow64/spawn32.exe
        fi
        # sqlite32.exe — a THIRD-PARTY stress guest (tools/stress/sqlite). The
        # other guests each probe one mechanism on purpose; this one is a large
        # C program that never heard of us, leaning on allocator churn, Win32
        # file I/O with locking and truncation, and long compute between
        # syscalls. Built on demand by tools/stress/sqlite/build.sh, so it is
        # absent unless someone asked for it -- like doom, staged only if built.
        SQLITE32="${VIBEOS_SQLITE32_EXE:-$PROJECT_DIR/tools/stress/sqlite/sqlite32.exe}"
        if [ -f "$SQLITE32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/sqlite32.exe (stress guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$SQLITE32" ::/reactos/syswow64/sqlite32.exe
        fi
        # lua32.exe — the second THIRD-PARTY stress guest (tools/stress/lua).
        # Where sqlite32 stresses the DATA path, this one stresses the CONTROL
        # path: Lua implements both error propagation and coroutine switching
        # with setjmp/longjmp, so every rung drives a non-local jump across
        # frames FEX has already translated -- machinery that SEH-based fault32
        # does not reach. Built on demand by tools/stress/lua/build.sh, so it is
        # absent unless someone asked for it -- like doom, staged only if built.
        LUA32="${VIBEOS_LUA32_EXE:-$PROJECT_DIR/tools/stress/lua/lua32.exe}"
        if [ -f "$LUA32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/lua32.exe (stress guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$LUA32" ::/reactos/syswow64/lua32.exe
        fi
        # WOW64 Phase C: the real i386 guest exe, launched through CreateProcessW
        # by winlogon's TryStartWow64Hello (silent no-op unless staged here).
        HELLO32="${VIBEOS_HELLO32_EXE:-$PROJECT_DIR/build-fex-hello/hello32.exe}"
        if [ -f "$HELLO32" ]; then
            echo "[*] VIBEOS_FEX=1 — staging syswow64/hello32.exe (Phase C guest)"
            mmd -i "$RAMDISK_IMG" ::/reactos/syswow64 2>/dev/null || true
            mcopy -o -i "$RAMDISK_IMG" "$HELLO32" ::/reactos/syswow64/hello32.exe
        else
            echo "[!] VIBEOS_FEX=1: hello32.exe missing ($HELLO32) — Phase C probe skipped"
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

# Copy freeldr.ini. BOTH boot entries always ship (see the file itself); the MP
# entry is the DEFAULT, and --up moves DefaultOS= back to the UP one, which is
# the same thing -DVIBEOS_UP_DEFAULT_BOOT=ON does to the x86 livecd.ini. The Options= lines are
# never rewritten here any more — the MP entry carries its own
# /KERNEL=ntkrnlmp.exe /HAL=halmp.dll, so the media is identical either way apart
# from that one selector.
if [ -n "$UP" ]; then
    UP_INI="$(mktemp)"
    # freeldr.ini is CRLF, so a bare `...MP$` anchor never matched and every --up
    # build failed here; keep the CR the loader expects.
    sed 's#^DefaultOS=ReactOS_Arm64_MP\(\r\{0,1\}\)$#DefaultOS=ReactOS_Arm64_UP\1#' "$FREELDR_INI" > "$UP_INI"
    # Fail loud rather than shipping UP-defaulted media from an --smp build: that
    # is a false-green (the boot looks fine and runs the wrong kernel).
    if ! tr -d '\r' < "$UP_INI" | grep -qx 'DefaultOS=ReactOS_Arm64_UP'; then
        echo "[!] freeldr.ini: no 'DefaultOS=ReactOS_Arm64_MP' line to flip — entry names drifted"
        rm -f "$UP_INI"
        exit 1
    fi
    echo "[*] UP: freeldr.ini DefaultOS -> ReactOS_Arm64_UP (ntoskrnl.exe + hal.dll) — OPT-OUT"
    mcopy -o -i "$BOOT_IMG" "$UP_INI" ::/freeldr.ini
    rm -f "$UP_INI"
else
    mcopy -i "$BOOT_IMG" "$FREELDR_INI" ::/freeldr.ini
fi

# Copy ramdisk image
echo "[*] Copying ramdisk.img to boot.img..."
mcopy -i "$BOOT_IMG" "$RAMDISK_IMG" ::/ramdisk.img

echo "[*] Boot image contents:"
mdir -i "$BOOT_IMG" -/ :: 2>&1 | grep -E '(\.EFI|\.efi|\.ini|\.img|bytes free)'

echo ""
echo "[*] Done! Boot with: bash qemu/run.sh"
echo "[*] Ramdisk: $(du -h "$RAMDISK_IMG" | cut -f1)"
echo "[*] Boot image: $(du -h "$BOOT_IMG" | cut -f1)"
