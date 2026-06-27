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
    """Build-side arch: build-<base>, ninja, stamp. Strips firmware/test suffix."""
    return arch.replace("-uefi", "").replace("-test", "")


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
            "msafd",
            "mswsock",
            "nettest",
        ),
        ninja_targets_optional=("arm64input", "arm64vid"),
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
