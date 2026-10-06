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

# CLI --arch choices. Plain i386/amd64 are ALIASES that resolve to the UEFI config;
# amd64-bios is the legacy-BIOS boot of the same build (restored 2026-10-05).
ACCEPTED_ARCHES = ("i386", "i386-uefi", "amd64", "amd64-uefi", "amd64-bios", "arm64")


def base_arch(arch: str) -> str:
    """Build-side arch: build-<base>, ninja, stamp. Strips every smoke-variant
    suffix (firmware/test/net) so all variants map to their base build tree —
    e.g. 'amd64-uefi-net' → 'amd64'. Strip '-net' before '-uefi' so the compound
    'amd64-uefi-net' doesn't leave a stray '-net'. This is THE base-arch mapping;
    conftest imports it rather than keeping a private copy (T2 one-source dedup)."""
    return arch.replace("-net", "").replace("-uefi", "").replace("-bios", "").replace("-test", "")


def vm_arch(arch: str) -> str:
    """Runtime/boot config name as registered in vm.py:ARCH_CONFIGS. x86 boots
    UEFI by default, so i386/amd64 resolve to i386-uefi/amd64-uefi; the legacy-BIOS
    boot is asked for by name."""
    if arch == "amd64-bios":
        return arch
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
        # The same `livecd` DAG as x86: every add_cd_file(... FOR livecd) target,
        # which is exactly the set mkramdisk.sh stages (it lays out the livecd list).
        # This was a hand-picked list of ~45 targets for most of arm64's life, and
        # each file missing from it shipped STALE on the ramdisk -- fastfat,
        # ramdisk.sys, pl011, gdi32, user32, advapi32, shell32, explorer and winlogon
        # were all found that way, one false-green at a time. On arm64 `livecd`
        # builds the files and no ISO (boot/boot_images.cmake).
        ninja_targets=("livecd",),
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
