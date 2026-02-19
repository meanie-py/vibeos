"""Smoke tests: verify QEMU boots, serial works, QMP works, snapshots work."""

from __future__ import annotations

from tests.qemu.vm import QemuVM


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


def test_snapshot_restore(vm: QemuVM) -> None:
    """Verify snapshot save/restore cycle works.

    The autouse restore_vm fixture already restored to 'ready'.
    Save a second snapshot, restore it, and verify VM is still alive.
    """
    vm.save_snapshot("test_snap")
    vm.restore_snapshot("test_snap")

    status = vm.qmp.execute("query-status")
    assert status["return"]["running"] is True
