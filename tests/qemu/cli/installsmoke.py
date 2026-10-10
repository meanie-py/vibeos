"""vibeos-cli install-smoke: install from the bootcd onto a blank disk, then boot the disk.

Every other boot starts from the livecd, an image laid down at build time, so setup itself
-- usetup (the bootcd's smss.exe), setuplib's partitioning, formatting, file copy, hive
creation and boot-loader install, and the fastfat/Cc/Cm write paths under them -- runs in no
other gate. This verb drives tests/qemu/test_install.py (two VMs over one disk image; see
its docstring) and reports one verdict + summary.json.

amd64-bios, amd64-uefi and i386-uefi (`amd64`/`i386` alias to UEFI, as for smoke): setup
installs FreeLdr's boot sectors under BIOS and uefildr.efi under UEFI. arm64 has no IDE bus
for the two-disk layout yet. Under TCG the install runs 10-60 minutes, so it is its own verb
rather than a gate stage.
"""

from __future__ import annotations

import sys
from pathlib import Path

from .archcfg import vm_arch
from .artifacts import Run, Stage

_TEST = "tests/qemu/test_install.py"
# Two stages of up to an hour each (test_install.py's budgets) plus the ISO build.
_TIMEOUT = 2 * 3600 + 900


def _salient(stdout: str) -> list[str]:
    """The assertion lines and the live-serial pointers -- what says where it stopped."""
    keep = [ln.strip() for ln in stdout.splitlines() if ln.startswith("E  ") or "live serial" in ln]
    return keep[-8:]


def install_smoke(args, repo: Path) -> int:
    arch = vm_arch(args.arch)
    run = Run("install-smoke", arch)
    if arch not in ("amd64-bios", "amd64-uefi", "i386-uefi"):
        run.add(
            Stage(
                "install",
                status="fail",
                detail="amd64-bios, amd64-uefi or i386-uefi (arm64 has no IDE bus for the disks)",
            )
        )
        return run.finalize(repo)
    cmd = [
        sys.executable,
        "-m",
        "pytest",
        _TEST,
        "--arch",
        arch,
        "-x",
        "-s",
        "-p",
        "no:cacheprovider",
        "--skip-clean-build",
        f"--basetemp={run.dir / 'pytest'}",
    ]
    cp = run.run(cmd, cwd=repo, stage="install", timeout=_TIMEOUT)
    ok = cp.returncode == 0
    run.add(
        Stage(
            "install",
            exit=cp.returncode,
            status="ok" if ok else "fail",
            detail="installed, rebooted, desktop reached" if ok else "install or first boot failed",
            salient=_salient(cp.stdout or ""),
        )
    )
    return run.finalize(repo)
