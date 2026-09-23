"""Per-arch BUILD/media facts — the single source of truth for how each arch
compiles and packages its boot media. (Runtime/boot config lives in vm.py's
ArchConfig; this is the build side.)

Keeping this here, not scattered through shell, is what prevents the
"x86 is ninja-livecd but arm64 needs mkramdisk --refresh" footgun and the
q35-vs-pc class of divergence.
"""

from __future__ import annotations

from dataclasses import dataclass

ARCHES = ("i386", "amd64", "arm64")  # base arches (build side)

# CLI --arch choices. BIOS x86 is dead; smoke registers only the -uefi configs,
# so plain i386/amd64 are accepted as ALIASES that resolve to the UEFI config.
ACCEPTED_ARCHES = ("i386", "i386-uefi", "amd64", "amd64-uefi", "arm64")


def base_arch(arch: str) -> str:
    """Build-side arch: build-<base>, ninja, stamp. Strips every smoke-variant
    suffix (firmware/test/net) so all variants map to their base build tree —
    e.g. 'amd64-uefi-net' → 'amd64'. Strip '-net' before '-uefi' so the compound
    'amd64-uefi-net' doesn't leave a stray '-net'. This is THE base-arch mapping;
    conftest imports it rather than keeping a private copy (T2 one-source dedup)."""
    return arch.replace("-net", "").replace("-uefi", "").replace("-test", "")


def vm_arch(arch: str) -> str:
    """Runtime/boot config name as registered in vm.py:ARCH_CONFIGS. x86 boots
    UEFI now, so i386/amd64 resolve to i386-uefi/amd64-uefi (BIOS deregistered)."""
    return {"i386": "i386-uefi", "amd64": "amd64-uefi", "arm64": "arm64"}[base_arch(arch)]


@dataclass(frozen=True)
class ArchBuild:
    arch: str
    # ninja targets to build (in order). For x86 the single `livecd` target's
    # DAG pulls in everything and re-rolls the iso; arm64 builds the staged set
    # then re-stages via mkramdisk (see needs_mkramdisk).
    ninja_targets: tuple[str, ...]
    # Best-effort targets — absent on some branches; don't fail the build if
    # ninja can't find them.
    ninja_targets_optional: tuple[str, ...] = ()
    needs_mkramdisk: bool = False
    # boot media path, relative to repo root. For arm64 this lives under qemu/,
    # not build-<arch>/.
    boot_media_rel: str = ""
    # Built binaries that get packaged into the boot media; their mtimes are
    # compared against boot media mtime (arm64 re-stage trap). Relative to
    # build-<arch>/. Empty on x86 (ninja's livecd DAG already tracks this).
    staged_bins_rel: tuple[str, ...] = ()


_CFGS = {
    "i386": ArchBuild(
        arch="i386",
        ninja_targets=("livecd",),
        boot_media_rel="build-i386/livecd.iso",
    ),
    "amd64": ArchBuild(
        arch="amd64",
        ninja_targets=("livecd",),
        boot_media_rel="build-amd64/livecd.iso",
    ),
    "arm64": ArchBuild(
        arch="arm64",
        # win32k + kernel32 are boot-critical AND frequently edited, but unlike
        # x86 (whose `livecd` DAG re-rolls everything) arm64 builds a hand-picked
        # staged set. Omitting them meant a win32k/kernel32 edit smoked via the
        # standard `build --arch arm64` rebuilt neither and re-staged a STALE
        # binary → false green (every arm64 gate this session worked around it
        # with manual `ninja win32k`/`rust_kernel32` + mcopy). They're in the
        # staged set below too, so staleness now also catches an un-rebuilt copy.
        # livecd_hives builds boot/bootdata/system (the SYSTEM hive) so Start=1
        # driver registrations (e.g. netstack) reach the arm64 boot — the arm64
        # build used to skip it, staging a stale netstack-less hive. netstack is
        # the cargo-driven netstack.sys; both are mkramdisk-staged below.
        # afd/msafd/mswsock/nettest = the netstack usermode-winsock chain (#92):
        # afd.sys is Start=1 (loaded explicitly on arm64, see iomgr/driver.rs),
        # msafd/mswsock are the winsock providers, nettest.exe is the proof app
        # winlogon spawns. Building+staging them here ends the per-gate manual
        # `ninja <target>` + mcopy inject dance (the same false-green footgun the
        # staged-set was created to close). mkramdisk.sh copies them.
        # fastfat is THE arm64 boot filesystem (the ramdisk is FAT32) — same trap:
        # it was NOT in this set, so a fastfat source fix landed on mainline (e.g.
        # arm64-love's FAT_DIRTY_VOLUME constant 0x0FFFFFF7→0xF0000001) never
        # reached the boot — the staged ramdisk kept shipping a STALE fastfat.sys,
        # whose old dirty-volume constant mismatched FatSetFatEntry's check and
        # raised a spurious STATUS_FILE_CORRUPT_ERROR (0xC0000102) → BUGCHECK 0x1E
        # during the registry hive-create. Rebuilding+staging fastfat clears it.
        # gdi32/user32 are the usermode GDI+windowing client DLLs (both being C→Rust
        # cohab-translated, user32 100% Rust, gdi32 in progress). Same trap as
        # win32k/kernel32: NOT in this set → `build --arch arm64` rebuilt neither, so
        # mkramdisk staged an 8-day-STALE gdi32.dll (.text=0x27e10, the old C) and a
        # stale user32.dll → arm64 gates false-greened the Rust gdi32/user32 (they
        # were boot-exercised on i386+amd64 but the arm64 boot ran the C binaries).
        # rust_gdi32/rust_user32 build the cohab staticlibs; the .dll targets relink
        # them in. mkramdisk.sh already copies both (lines 232-233).
        ninja_targets=(
            "ntoskrnl",
            "hal",
            "uefildr",
            "win32k",
            "rust_kernel32",
            "dll/win32/kernel32/kernel32.dll",
            "rust_gdi32",
            "win32ss/gdi/gdi32/gdi32.dll",
            "rust_user32",
            "win32ss/user/user32/user32.dll",
            # advapi32 is boot-critical on arm64 (lsass/services/winlogon link it) and
            # its body is 100% Rust (advapi32-rust cohab). Same stale-.dll trap as
            # gdi32/user32: NOT rebuilt by `build --clean --arch arm64` unless listed →
            # the gate would stage a stale C advapi32.dll and false-green the Rust one.
            "rust_advapi32",
            "dll/win32/advapi32/advapi32.dll",
            # shell32 is the usermode Shell client DLL (explorer.exe links it → boot-
            # critical). Its wine-derived .c half is being C→Rust cohab-translated
            # (shell32-rust staticlib). Same stale-DLL trap as gdi32/user32: NOT in this
            # set → `build --arch arm64` wouldn't relink shell32.dll, so mkramdisk would
            # stage a stale shell32 and the arm64 gate would false-green the Rust cohab.
            # rust_shell32 builds the staticlib; the .dll target relinks it in.
            # mkramdisk.sh already copies it (line 272).
            "rust_shell32",
            "dll/win32/shell32/shell32.dll",
            # rosbrand.dll holds the About banner since upstream's "[SHELL32] Use new branding
            # library" moved IDB_BRAND_BANNER / _BANNERLINE out of shell32's own resources;
            # shell32 LoadLibraryEx's it as a datafile. Not built → not staged → no banner.
            "dll/branding/rosbrand/rosbrand.dll",
            # explorer.exe is the boot marker's own process and its body is Rust
            # (explorer-rust cohab staticlib). Same stale trap as the DLLs above:
            # NOT in this set → a merge touching explorer-rust/*.rs leaves
            # libexplorer_rust.a stale and mkramdisk stages an old explorer.exe
            # (caught by artifact-check, gate 2026-07-18). rust_explorer builds
            # the staticlib; the .exe target relinks it in. winlogon.exe is the
            # other mkramdisk-staged Rust-bodied .exe (winlogon-rust) — same
            # trap, listed for the same reason (both lanes hit this the same
            # night; this entry is the union of the two fixes).
            "winlogon",
            "rust_explorer",
            "base/shell/explorer/explorer.exe",
            "fastfat",
            # ramdisk is THE arm64 boot device (the FAT32 image lives in RAM, exposed
            # as \Device\Ramdisk{GUID} via \ArcName\ramdisk(0)) — same stale-driver trap
            # as fastfat above. It was NOT in this set, so the 2026-06-21 ramdisk.c->Rust
            # cohab flip (1c492f8adb9, link-clean-only bar) was NEVER boot-exercised: the
            # gate kept shipping the pre-flip STALE C ramdisk.sys and false-greened arm64
            # while a clean build's Rust ramdisk-rust 0x64'd (garbage FSCTL/IOCTL consts).
            # pl011 is the PL011 UART serial driver (Rust cohab, BLT-loaded) — same risk.
            # These two + fastfat + arm64input/arm64vid (optional) = all 5 arm64
            # boot-critical Rust-cohab drivers now actually rebuilt+staged by the gate.
            "ramdisk",
            "pl011",
            "netstack",
            "livecd_hives",
            "afd",
            # npfs = the named-pipe FS, services = the SCM. arm64 had NEITHER,
            # which is why the kmtest sweep couldn't run: every test's
            # CreateService is advapi32 -> RPC over \pipe\ntsvcs -> services.exe,
            # so with no npfs and no SCM each test raised RPC_S_SERVER_UNAVAILABLE
            # (0x6BA) before its first print. Both already build for arm64 and
            # mkramdisk.sh stages them; npfs is already Start=1-registered in the
            # arm64 SYSTEM hive and is hand-loaded via iomgr's STAGED_SVCS.
            "npfs",
            "services",
            # lsass + its 2 unstaged DLLs. NOT optional once services runs:
            # ScmCreateServiceDatabase ends in ScmWaitForLsa() =
            # WaitForSingleObject(LSA_RPC_SERVER_ACTIVE, INFINITE), and only
            # lsass.exe signals that event — an SCM without an LSA hangs forever
            # mid-database, so SC_AutoStartComplete never fires and every kmtest
            # times out. lsasrv/samsrv are the only 2 of lsass's 15 transitive
            # DLLs not already staged for the boot.
            "lsass",
            "lsasrv",
            "samsrv",
            # msv1_0 = the MSV1_0 auth package lsasrv loads BY NAME at runtime
            # (Lsa "Authentication Packages"), so no import-table walk finds it:
            # lsass's every static import resolved and it still exited
            # 0xC0000135 (STATUS_DLL_NOT_FOUND) without it.
            "msv1_0",
            "msafd",
            "mswsock",
            "nettest",
        ),
        # ntkrnlmp + halmp = the arm64 SMP PAIR (smp-bringup-quest.md M5): the MP
        # kernel and its matching HAL (hal-rust `smp` feature). mkramdisk stages
        # BOTH on every build and the shipped freeldr.ini offers both a
        # [ReactOS_Arm64_UP] and a [ReactOS_Arm64_MP] entry, so which kernel boots
        # is a DefaultOS= pick against fixed media — no rebuild, exactly like x86.
        # Since 2026-09-12 the MP pair is the DEFAULT (qemu/freeldr.ini line 2 says
        # DefaultOS=ReactOS_Arm64_MP) and `--up-kernel` is the opt-out; nothing
        # automated opts in. Do not read "MP" as "several cpus": the default boot
        # is the MP KERNEL on ONE vcpu, which is a different thing from --smp N. They are listed as
        # best-effort ninja targets but mkramdisk REQUIRES the outputs, so a build
        # that cannot produce them fails there rather than shipping media whose MP
        # entry names files that are not on it.
        # BOTH or neither: arm64 has no MISMATCHED_HAL handshake, so an MP kernel
        # on the UP hal.dll boots to a desktop and silently runs on one cpu.
        # kmtest_all (kmtest_ .exe + the standalone *_drv.sys corpus) is the
        # `vibeos-cli conformance --arch arm64` payload; mkramdisk.sh stages it
        # to reactos/bin. It is the SAME stale-staged-binary trap as every entry
        # above: x86's `livecd` DAG re-rolls the corpus on every build (each
        # driver calls add_rostests_file -> add_cd_file), but arm64's hand-picked
        # staged set rebuilt none of it, so the ramdisk would ship whatever
        # *_drv.sys happened to be left in build-arm64/ from some older ninja —
        # i.e. a conformance sweep silently grading STALE test binaries.
        # OPTIONAL, not required: the corpus only exists when the build dir is
        # configured -DENABLE_ROSTESTS(_KMTESTS)=ON (both cmake options default
        # OFF), and a kmtests-off dir must still build normally. The loud failure
        # for "asked for conformance without the corpus" already lives at the
        # right layer — conformance.py refuses up front with the configure line.
        # (Both are best-effort and INDEPENDENT: build.py runs one ninja per
        # optional target, so an absent kmtest_all can't take ntkrnlmp down with
        # it — which the old single batched `ninja a b c` would have done.)
        # That isolation is about the BUILD and does not extend to the BOOT.
        # Once staged, the kmtests RUN in every arm64 boot — [KMTEST:BEGIN ...]
        # is in any arm64 smoke serial — while `smoke` never grades them (zero
        # kmtest references in smoke.py; `conformance` is the subcommand that
        # parses and compares). So a kmtest that merely FAILS runs past unread,
        # but one that BUGCHECKS takes the smoke down and presents as a boot
        # failure with no visible connection to conformance. Check the serial for
        # KMTEST markers before attributing an arm64 boot failure to whatever you
        # were actually changing.
        ninja_targets_optional=("arm64input", "arm64vid", "ntkrnlmp", "halmp", "kmtest_all"),
        needs_mkramdisk=True,
        boot_media_rel="qemu/boot.img",
        staged_bins_rel=(
            "ntoskrnl/ntoskrnl.exe",
            "hal/halarm64/hal.dll",
            "boot/freeldr/uefildr/uefildr.efi",
            "win32ss/win32k.sys",
            "dll/win32/kernel32/kernel32.dll",
            "win32ss/gdi/gdi32/gdi32.dll",
            "win32ss/user/user32/user32.dll",
            "dll/win32/advapi32/advapi32.dll",
            "dll/win32/shell32/shell32.dll",
            "drivers/filesystems/fastfat/fastfat.sys",
            "drivers/storage/class/ramdisk/ramdisk.sys",
            "drivers/serial/pl011/pl011.sys",
            "drivers/input/arm64input/arm64input.sys",
            "drivers/video/arm64vid/arm64vid.sys",
            "drivers/network/netstack/netstack.sys",
            "drivers/network/afd/afd.sys",
            "dll/win32/msafd/msafd.dll",
            "dll/win32/mswsock/mswsock.dll",
            "base/applications/network/nettest/nettest.exe",
            "boot/bootdata/system",
        ),
    ),
}


def arch_build(arch: str) -> ArchBuild:
    b = base_arch(arch)
    if b not in _CFGS:
        raise ValueError(f"unknown arch {arch!r}; choose from {ACCEPTED_ARCHES}")
    return _CFGS[b]


# Rust workspace source root (one hash of this answers "did any rust source
# change since the build").
RUST_SRC_DIRS = ("sdk/lib/rust",)
