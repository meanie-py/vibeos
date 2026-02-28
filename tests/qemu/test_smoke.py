"""Smoke tests: verify QEMU boots, serial works, QMP works, snapshots work."""

from __future__ import annotations

import subprocess
from pathlib import Path

import pytest

from tests.qemu.vm import QemuVM, PROJECT_DIR


def test_boot_and_serial(vm: QemuVM) -> None:
    """Verify QEMU boots and produces serial output."""
    # wait_for_boot() in the session fixture succeeded without timeout,
    # meaning the kernel booted and produced serial output.
    assert vm._boot_output, "No boot output captured"
    threshold = vm.config.min_boot_bytes
    assert len(vm._boot_output) >= threshold, (
        f"Too little boot output: {len(vm._boot_output)} bytes (expected >= {threshold})"
    )
    # Verify ReactOS boot output appears
    assert "ReactOS" in vm._boot_output or "UEFI" in vm._boot_output, (
        "Neither ReactOS nor UEFI string found in boot output"
    )


def test_qmp_status(vm: QemuVM) -> None:
    """Verify QMP reports VM as running."""
    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True


def test_snapshot_restore(vm: QemuVM, arch: str) -> None:
    """Verify snapshot save/restore cycle works.

    The autouse restore_vm fixture already restored to 'ready'.
    Save a second snapshot, restore it, and verify VM is still alive.
    Skipped on i386 — no scratch disk for savevm (FreeLDR limitation).
    """
    if arch == "i386":
        pytest.skip("i386 has no scratch disk for savevm")
    vm.save_snapshot("test_snap")
    vm.restore_snapshot("test_snap")

    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True


def test_screenshot(vm: QemuVM, arch: str) -> None:
    """Capture a screenshot of the desktop at steady state.

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
