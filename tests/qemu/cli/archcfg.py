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
        ninja_targets=("ntoskrnl", "hal", "uefildr"),
        ninja_targets_optional=("arm64input", "arm64vid"),
        needs_mkramdisk=True,
        boot_media_rel="qemu/boot.img",
        staged_bins_rel=(
            "ntoskrnl/ntoskrnl.exe",
            "hal/halarm64/hal.dll",
            "boot/freeldr/uefildr/uefildr.efi",
            "drivers/input/arm64input/arm64input.sys",
            "drivers/video/arm64vid/arm64vid.sys",
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
