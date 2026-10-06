#!/bin/bash
#
# Build ramdisk.img and boot.img for the ReactOS ARM64 ramdisk boot.
#
# The ramdisk holds the SAME files as the x86 livecd: boot/livecd.$CONFIG.lst, the
# list every add_cd_file(... FOR livecd) feeds and `mkisofs -path-list` consumes on
# x86, is laid out by tools/ramdisk_tree.py and copied into the FAT image in one
# `mcopy -s`. On top of it go only the things that are arm64's own (the overlay below)
# and the opt-in payloads.
#
# This file used to stage ~250 files by hand. Every file the livecd gained after that
# had to be noticed and copied here as well, and most were not: setup.exe, syssetup.dll
# and every .inf were missing, so the `setup -mini` the livecd hive runs on first boot
# failed silently on every arm64 boot, and nothing it sets up (COM registrations,
# Tcpip/Dhcp/Dnscache, device installs, PlugPlay) ever happened. ucrtbase.dll, the
# sam/security hives, c_20127.nls, the winsxs store and %SystemRoot%\TEMP had each
# gone missing the same way and been found one at a time.
#
# Usage:
#   ./mkramdisk.sh              # build ramdisk.img + boot.img from build-arm64
#   ./mkramdisk.sh --up         # boot-select the UP kernel (opt-out; MP is default)
#   ./mkramdisk.sh --refresh    # accepted for old callers; the image is always rebuilt
#
# Build first with `./vibeos-cli build --arch arm64` (it builds `livecd`).

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build-arm64"
QEMU_DIR="$SCRIPT_DIR"

# Both images are built under temporary names and renamed into place at the very end, so a
# reader never sees a half-written one. The merge-queue gate runs tooling while its arm64
# build rewrites these: test_recordreplay's qemu-img, backing a qcow2 on boot.img, failed on
# a green master (2026-10-04), and a concurrent smoke would boot a partial image.
RAMDISK_IMG_FINAL="$QEMU_DIR/ramdisk.img"
BOOT_IMG_FINAL="$QEMU_DIR/boot.img"
RAMDISK_IMG="$QEMU_DIR/.ramdisk.img.partial-$$"
BOOT_IMG="$QEMU_DIR/.boot.img.partial-$$"
trap 'rm -f "$RAMDISK_IMG" "$BOOT_IMG"' EXIT
UEFILDR="$BUILD_DIR/boot/freeldr/freeldr/uefildr.efi"
FREELDR_INI="$QEMU_DIR/freeldr.ini"
LIVECD_LST="$BUILD_DIR/boot/livecd.Debug.lst"
STAGE_DIR="$BUILD_DIR/boot/ramdisk"

UP=""    # --up: boot-select the UP entry. The MP pair is the DEFAULT (2026-09-12);
         # this is the only way to get a UP boot, and nothing automated should use it.

# Debug-stripped PE twins, the same pass the x86 livecd target runs before mkisofs:
# DWARF sections are never mapped and are most of each file's bytes. It HARD-FAILS
# on a strip error (never silently stages an unstripped file).
STRIP_CACHE="$BUILD_DIR/boot/stripped"
STRIP_OBJCOPY="$PROJECT_DIR/.build-tools/llvm-mingw/bin/llvm-objcopy"

# mcopy cannot create parent directories, so every deep destination needs its
# path components made first. Probe-then-create, because `mmd` prompts on an
# existing directory when stdout is a tty and vibeos-cli runs this under a PTY.
MMDIR_P() {
    local dir partial comp
    dir="$(dirname "$1")"
    [ "$dir" != "." ] && [ "$dir" != "/" ] || return 0
    partial=""
    # No pipe: a `while read` on the right of one runs in a subshell, and this
    # has to accumulate across iterations.
    local IFS=/
    for comp in $dir; do
        [ -n "$comp" ] || continue
        partial="${partial:+$partial/}$comp"
        mdir -i "$RAMDISK_IMG" "::/$partial" >/dev/null 2>&1 \
            || mmd -i "$RAMDISK_IMG" "::/$partial"
    done
}

for arg in "$@"; do
    case "$arg" in
        --up)
            UP=1
            ;;
        --refresh)
            ;;
        --help|-h)
            sed -n '2,24p' "$0"
            exit 0
            ;;
        *)
            echo "[!] unknown argument: $arg"
            exit 1
            ;;
    esac
done

for f in "$LIVECD_LST" "$UEFILDR"; do
    if [ ! -f "$f" ]; then
        echo "[!] Missing: $f"
        echo "    Run: ./vibeos-cli build --arch arm64"
        exit 1
    fi
done

# --- The livecd list -----------------------------------------------------------
# Its paths are written against the canonical build root (tools/canon-run.sh), which
# is only mounted while a build runs; map them back to this worktree.
LST_DIR="$BUILD_DIR/boot"
sed "s#=/tmp/vibeos/#=$PROJECT_DIR/#; s#^/tmp/vibeos/#$PROJECT_DIR/#" "$LIVECD_LST" > "$LST_DIR/ramdisk-livecd.lst"
python3 "$PROJECT_DIR/tools/strip_cd_media.py" --lst "$LST_DIR/ramdisk-livecd.lst" \
    --out "$LST_DIR/ramdisk-livecd.stripped.lst" \
    --cache "$STRIP_CACHE" --objcopy "$STRIP_OBJCOPY" || exit 1

# --- The arm64 overlay ---------------------------------------------------------
# Only what has no livecd counterpart belongs here. Later lists win, so an entry
# here replaces the livecd's copy of the same destination.
OVERLAY_LST="$LST_DIR/ramdisk-overlay.lst"
: > "$OVERLAY_LST"

# csrss.exe with SizeOfStackCommit raised to 256KB. The PE default (4KB) was too
# small for the deep DLL-initialisation chains of the arm64 bring-up, where
# delay-imports became regular imports and every DLL loaded at once.
CSRSS_PATCHED="$LST_DIR/csrss-stack256k.exe"
cp "$BUILD_DIR/subsystems/csr/csrss/csrss.exe" "$CSRSS_PATCHED"
python3 "$SCRIPT_DIR/patch_pe_stack.py" "$CSRSS_PATCHED" 0x40000
echo "reactos/system32/csrss.exe=$CSRSS_PATCHED" >> "$OVERLAY_LST"

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
    echo "reactos/system32/libwow64fex.dll=$WOW64FEX_DEFAULT" >> "$OVERLAY_LST"
else
    echo "[!] FEX CPU layer MISSING ($WOW64FEX_DEFAULT) — this image has NO 32-bit"
    echo "    support at all. Build it with: tools/fex/build-fex.sh"
fi

# The x86-64 CPU layer (BTCpu64, docs/engineering/wow64-x64-guest-design.md
# Fork B) rides the same rule for the same reason: free until an x64 process
# starts, and loud when absent. Its OS half, wowx64.dll, is an ordinary cmake
# target staged through the livecd list like any system DLL.
WOWX64FEX_DEFAULT="${VIBEOS_WOWX64FEX_DLL:-$PROJECT_DIR/build-fex-hello/wowx64fex/libwowx64fex.dll}"
if [ -f "$WOWX64FEX_DEFAULT" ]; then
    echo "[*] staging libwowx64fex.dll into system32 (x86-64 CPU layer, default-on)"
    echo "reactos/system32/libwowx64fex.dll=$WOWX64FEX_DEFAULT" >> "$OVERLAY_LST"
else
    echo "[!] x86-64 CPU layer MISSING ($WOWX64FEX_DEFAULT) — this image cannot run"
    echo "    x64 apps. Build it with: tools/fex/build-fex.sh && tools/fex/build-hello-compute.sh"
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
# They are the last list, so a payload replaces a stock file of the same name.
PAYLOAD_LST="$LST_DIR/ramdisk-payloads.lst"
python3 "$PROJECT_DIR/tools/payloads/compile.py" manifest \
    --arch arm64 --payloads-auto "$BUILD_DIR" > "$PAYLOAD_LST"
if [ -s "$PAYLOAD_LST" ]; then
    echo "[*] payloads (VIBEOS_PAYLOAD=${VIBEOS_PAYLOAD:-<stamp>}${VIBEOS_DOOM:+ VIBEOS_DOOM=$VIBEOS_DOOM}):"
    sed 's/^/[*]   /; s/=.*//' "$PAYLOAD_LST"
fi

# --- Lay it out and build the image -------------------------------------------
TREE_BYTES=$(python3 "$PROJECT_DIR/tools/ramdisk_tree.py" --out "$STAGE_DIR" \
    "$LST_DIR/ramdisk-livecd.stripped.lst" "$OVERLAY_LST" "$PAYLOAD_LST") || exit 1

# Sized to the content: the FAT overhead at one sector per cluster is ~2%, and the
# headroom is real free space, because %SystemRoot% is this writable volume (hives,
# %TEMP%, logs). The FEX guest set below is copied after the tree and needs its own.
HEADROOM_MB=64
[ "${VIBEOS_FEX:-0}" = "1" ] && HEADROOM_MB=$((HEADROOM_MB + 192))
# ...plus the amd64 guest set (sysamd64, ~40MB stripped) when there is one.
[ "${VIBEOS_FEX:-0}" = "1" ] && [ -d "$PROJECT_DIR/build-amd64" ] && HEADROOM_MB=$((HEADROOM_MB + 64))
RAMDISK_SIZE_MB=$(( TREE_BYTES * 105 / 100 / 1048576 + HEADROOM_MB ))

echo "[*] Creating ramdisk.img (${RAMDISK_SIZE_MB}MB FAT32, $((TREE_BYTES / 1048576))MB of files)..."
rm -f "$RAMDISK_IMG"
dd if=/dev/zero of="$RAMDISK_IMG" bs=1M count=$RAMDISK_SIZE_MB 2>/dev/null
# One sector per cluster (mformat -F's default): -c 8 would shrink the FAT, but
# FreeLDR's FAT reader cannot read it.
# -m 0xf8 on both images: at a size that is an exact CHS multiple (315MB is 640 x 16 x
# 63), mformat otherwise picks a floppy media byte and drive number (0xF0, 0x00). See
# BOOT_SIZE_MB below for the half of that which stopped arm64 booting.
mformat -i "$RAMDISK_IMG" -F -m 0xf8 -v REACTOS ::
mcopy -s -i "$RAMDISK_IMG" "$STAGE_DIR"/* ::/

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
        # BTCpu64 M1: the first x86-64 guest, beside the arm64 probes in
        # system32 (an x64 exe is not a WoW64 guest; its DLLs are in sysamd64).
        HELLO64="${VIBEOS_HELLO64_EXE:-$PROJECT_DIR/build-fex-hello/hello64.exe}"
        if [ -f "$HELLO64" ]; then
            echo "[*] VIBEOS_FEX=1 — staging hello64.exe (x86-64 guest, BTCpu64 M1)"
            mcopy -o -i "$RAMDISK_IMG" "$HELLO64" ::/reactos/system32/hello64.exe
        else
            echo "[!] VIBEOS_FEX=1: hello64.exe not built ($HELLO64) — x86-64 guest skipped"
        fi
        # M2: the rest of the x86-64 ladder (the 32-bit ladder's sources rebuilt
        # for x86-64), same directory, same staged-if-built rule.
        for G in gui64 win64 file64 async64 marshal64 fault64 thread64 selfmod64 reg64 spawn64; do
            if [ -f "$PROJECT_DIR/build-fex-hello/$G.exe" ]; then
                mcopy -o -i "$RAMDISK_IMG" "$PROJECT_DIR/build-fex-hello/$G.exe" "::/reactos/system32/$G.exe"
            else
                echo "[!] VIBEOS_FEX=1: $G.exe not built — that x86-64 rung will no-op"
            fi
        done
        # BTCpu64 M2: the x86-64 guest's own system DLLs — amd64 ntdll and the
        # whole import closure of a stock amd64 GUI app, out of build-amd64,
        # into sysamd64. wowx64.dll maps the guest ntdll from here and
        # redirects the guest's every system32 open here, as WoW64 does with
        # syswow64. Complete closure or nothing, for the reason the SysWOW64
        # block below spells out: a half-bound module crashes at first call.
        TESTGUI64="${VIBEOS_TESTGUI64_EXE:-$PROJECT_DIR/build-amd64/base/applications/testgui/testgui.exe}"
        if [ -f "$TESTGUI64" ]; then
            echo "[*] VIBEOS_FEX=1 — staging the sysamd64 DLL closure (amd64 guest system DLLs)"
            mmd -i "$RAMDISK_IMG" ::/reactos/sysamd64 2>/dev/null || true
            CLOSURE64=$(python3 "$PROJECT_DIR/tools/fex/wow64-closure.py" --include-prestaged \
                          --build-dir "$PROJECT_DIR/build-amd64" "$TESTGUI64") || {
                echo "[!] VIBEOS_FEX=1: sysamd64 import closure is INCOMPLETE (see above)."
                echo "[!]   Refusing to stage a partial set. Build build-amd64 first."
                exit 1
            }
            echo "$CLOSURE64" | while IFS=$'\t' read -r name path; do
                [ -n "$name" ] || continue
                mcopy -o -i "$RAMDISK_IMG" "$path" "::/reactos/sysamd64/$name"
            done
            echo "[*]   staged $(echo "$CLOSURE64" | wc -l) sysamd64 modules"
        else
            echo "[!] VIBEOS_FEX=1: no build-amd64 ($TESTGUI64) — x86-64 guests that import"
            echo "[!]   anything (all but hello64) will fail at their first DLL. Build amd64 first."
        fi
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
            # (stripped) on top of the guest headroom the image is sized with.
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

echo "[*] Ramdisk: $(mdir -i "$RAMDISK_IMG" :: 2>/dev/null | tail -1 | sed 's/^ *//')"

# --- Create boot.img ---
# uefildr + freeldr.ini + the ramdisk image, with a little slack.
BOOT_SIZE_MB=$((RAMDISK_SIZE_MB + 16))
# Never a whole number of 16-head/63-sector cylinders (1008 sectors): at such a size
# (315MB) mformat's partition entry spans the entire disk instead of stopping short at a
# cylinder boundary, and AAVMF never offers boot.img as a boot device -- the boot falls
# through to PXE and HTTP and smoke:arm64 hangs with no freeldr output (2026-10-04).
while [ $(( BOOT_SIZE_MB * 2048 % 1008 )) -eq 0 ]; do BOOT_SIZE_MB=$((BOOT_SIZE_MB + 1)); done
echo "[*] Creating boot.img (${BOOT_SIZE_MB}MB FAT32)..."
rm -f "$BOOT_IMG"
dd if=/dev/zero of="$BOOT_IMG" bs=1M count=$BOOT_SIZE_MB 2>/dev/null
mformat -i "$BOOT_IMG" -F -m 0xf8 -v ROSBOOT ::

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

mv -f "$RAMDISK_IMG" "$RAMDISK_IMG_FINAL"
mv -f "$BOOT_IMG" "$BOOT_IMG_FINAL"

echo ""
echo "[*] Done! Boot with: bash qemu/run.sh"
echo "[*] Ramdisk: $(du -h "$RAMDISK_IMG_FINAL" | cut -f1)"
echo "[*] Boot image: $(du -h "$BOOT_IMG_FINAL" | cut -f1)"
