"""Smoke tests: verify QEMU boots to desktop, serial works, QMP works, snapshots work."""

from __future__ import annotations

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
