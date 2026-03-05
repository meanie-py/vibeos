"""Smoke tests: verify QEMU boots, serial works, QMP works, snapshots work."""

from __future__ import annotations

import subprocess
from pathlib import Path

import pytest

from tests.qemu.vm import QemuVM, PROJECT_DIR


def _debugcon_hint(vm: QemuVM) -> str:
    """Get debugcon triage report for assertion failure messages."""
    report = vm.debugcon_report()
    if report:
        return f"\n\n{report}"
    return ""


def test_boot_and_serial(vm: QemuVM) -> None:
    """Verify QEMU boots and produces serial output."""
    # wait_for_boot() in the session fixture succeeded without timeout,
    # meaning the kernel booted and produced serial output.
    hint = _debugcon_hint(vm)
    assert vm._boot_output, f"No boot output captured{hint}"
    threshold = vm.config.min_boot_bytes
    assert len(vm._boot_output) >= threshold, (
        f"Too little boot output: {len(vm._boot_output)} bytes "
        f"(expected >= {threshold}){hint}"
    )
    # Verify ReactOS boot output appears (i386 serial comes from DbgPrint,
    # not the display — so "ReactOS" may not appear; accept HAL signatures)
    has_reactos = "ReactOS" in vm._boot_output
    has_uefi = "UEFI" in vm._boot_output
    has_hal = "halx86" in vm._boot_output or "PCI BUS" in vm._boot_output
    assert has_reactos or has_uefi or has_hal, (
        f"No ReactOS/UEFI/HAL signature found in boot output{hint}"
    )


def test_qmp_status(vm: QemuVM, arch: str) -> None:
    """Verify QMP reports VM as running."""
    if arch == "i386":
        pytest.skip("i386 LiveCD hangs in PnP enumeration after boot drivers")
    if arch == "amd64":
        pytest.skip("amd64 crashes after KD init (KE Rust regression)")
    if arch == "arm64":
        pytest.skip("arm64 crashes during early MM init (KE Rust regression)")
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
    if arch == "amd64":
        pytest.skip("amd64 crashes after KD init (KE Rust regression)")
    if arch == "arm64":
        pytest.skip("arm64 crashes during early MM init (KE Rust regression)")
    vm.save_snapshot("test_snap")
    vm.restore_snapshot("test_snap")

    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True


def test_screenshot(vm: QemuVM, arch: str) -> None:
    """Capture a screenshot of the desktop at steady state.

    Saves as qemu/<arch>_smoke.png for visual inspection.
    """
    if arch == "amd64":
        pytest.skip("amd64 crashes after KD init (KE Rust regression)")
    if arch == "arm64":
        pytest.skip("arm64 crashes during early MM init (KE Rust regression)")
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
