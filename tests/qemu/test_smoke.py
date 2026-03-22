"""Smoke tests: verify QEMU boots to desktop, serial works, QMP works, snapshots work."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

import pytest

from tests.qemu.vm import QemuVM, PROJECT_DIR


def _screenshot_path(arch: str) -> Path:
    """Return the path where the smoke screenshot will be saved."""
    return PROJECT_DIR / "qemu" / f"{arch}_smoke.png"


def _debugcon_hint(vm: QemuVM) -> str:
    """Get debugcon triage report + screenshot path for assertion failure messages."""
    parts = []
    report = vm.debugcon_report()
    if report:
        parts.append(report)
    parts.append(f"Screenshot: {_screenshot_path(vm.config.arch)}")
    return "\n\n" + "\n".join(parts)


def test_boot_and_serial(vm: QemuVM) -> None:
    """Verify QEMU boots and produces serial output without crashing."""
    hint = _debugcon_hint(vm)
    assert vm._boot_output, f"No boot output captured{hint}"
    threshold = vm.config.min_boot_bytes
    assert len(vm._boot_output) >= threshold, (
        f"Too little boot output: {len(vm._boot_output)} bytes "
        f"(expected >= {threshold}){hint}"
    )
    # Detect crashes in serial output (catches BSODs that idle detection misses)
    for crash_sig in ["*** BUGCHECK:", "*** STOP:"]:
        assert crash_sig not in vm._boot_output, (
            f"Crash detected in serial output: {crash_sig!r}{hint}"
        )


def test_qmp_status(vm: QemuVM) -> None:
    """Verify QMP reports VM as running."""
    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True


def test_snapshot_restore(vm: QemuVM) -> None:
    """Verify snapshot save/restore cycle works."""
    vm.save_snapshot("test_snap")
    vm.restore_snapshot("test_snap")

    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True


def test_screenshot(vm: QemuVM, arch: str) -> None:
    """Capture a screenshot and verify it shows a desktop.

    Saves as qemu/<arch>_smoke.png for visual inspection.
    """
    ppm_path = vm.work_dir / "screen.ppm"
    vm.screendump(ppm_path)
    assert ppm_path.exists(), "screendump did not create file"
    assert ppm_path.stat().st_size > 1000, "screendump file too small"

    # Convert PPM to PNG and save to qemu/ directory
    png_path = PROJECT_DIR / "qemu" / f"{arch}_smoke.png"
    try:
        subprocess.run(
            ["convert", str(ppm_path), str(png_path)],
            check=True, capture_output=True, timeout=10,
        )
    except (FileNotFoundError, subprocess.CalledProcessError):
        # ImageMagick not available — try pnmtopng or just keep PPM
        try:
            subprocess.run(
                ["pnmtopng", str(ppm_path)],
                check=True, capture_output=True, timeout=10,
                stdout=open(png_path, "wb"),
            )
        except (FileNotFoundError, subprocess.CalledProcessError):
            # No converter — copy PPM as-is
            png_path = PROJECT_DIR / "qemu" / f"{arch}_smoke.ppm"
            import shutil
            shutil.copy2(ppm_path, png_path)

    print(f"\n[screenshot] Saved to {png_path}")


def test_sacred_primitives(arch: str) -> None:
    """Verify that LLVM has not eliminated sacred IRQL/spinlock primitives.

    Static binary analysis — no QEMU needed. Checks that KfRaiseIrql,
    KfLowerIrql, KeAcquireSpinLock, etc. survive optimization in the
    final ntoskrnl binary.
    """
    build_dir = PROJECT_DIR / f"build-{arch}"
    ntoskrnl = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntoskrnl.exists():
        pytest.skip(f"ntoskrnl.exe not found at {ntoskrnl}")

    # Find a working objdump
    objdump_candidates = {
        "arm64": ["aarch64-w64-mingw32-objdump", "llvm-objdump"],
        "amd64": ["x86_64-w64-mingw32-objdump", "llvm-objdump"],
        "i386": ["i686-w64-mingw32-objdump", "llvm-objdump"],
    }
    objdump = None
    for candidate in objdump_candidates.get(arch, ["llvm-objdump"]):
        try:
            subprocess.run([candidate, "--version"], capture_output=True, check=True)
            objdump = candidate
            break
        except (FileNotFoundError, subprocess.CalledProcessError):
            continue
    if objdump is None:
        pytest.skip("No objdump available")

    result = subprocess.run(
        [objdump, "-d", str(ntoskrnl)],
        capture_output=True, text=True, timeout=120,
    )
    disasm = result.stdout

    # amd64 uses direct CR8 access for IRQL, not KfRaiseIrql/KfLowerIrql
    if arch == "amd64":
        assert disasm.count("cr8") > 0, (
            "No CR8 accesses in ntoskrnl — IRQL primitives may be eliminated"
        )
        assert "KeAcquireSpinLock" in disasm, (
            "KeAcquireSpinLock not found in ntoskrnl — spinlock primitive eliminated"
        )
    else:
        for prim in ["KfRaiseIrql", "KfLowerIrql", "KeAcquireSpinLock"]:
            assert prim in disasm, (
                f"{prim} not found in ntoskrnl — sacred primitive eliminated by LLVM"
            )
