"""Opt-in xHCI keyboard delivery regression (VIBEOS_QEMU_XHCI=1)."""

from __future__ import annotations

import os
import time

import pytest

from tests.qemu.cli.provenance import check_staleness
from tests.qemu.test_smoke import _FATAL_MARKERS
from tests.qemu.vm import PROJECT_DIR, QemuVM


@pytest.fixture(scope="session", autouse=True)
def xhci_preflight(arch: str) -> None:
    if os.environ.get("VIBEOS_QEMU_XHCI") != "1":
        pytest.skip("requires the explicit xHCI keyboard/mouse rig")
    media_arch = {"i386-uefi": "i386", "amd64-uefi": "amd64"}.get(arch, arch)
    freshness = check_staleness(PROJECT_DIR, media_arch)
    assert freshness.fresh, f"Rebuild {media_arch} media: {freshness.reasons}"


@pytest.fixture(scope="session")
def vm(xhci_preflight: None, vm: QemuVM) -> QemuVM:
    # Explicit dependency: check media before the parent fixture starts QEMU.
    return vm


def test_xhci_keyboard_delivery(vm: QemuVM) -> None:
    serial = vm._boot_output + vm.serial_output(timeout=1)
    assert "[XHCI] controller running" in serial
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        for down in (True, False):
            response = vm.qmp.execute(
                "input-send-event",
                events=[
                    {
                        "type": "key",
                        "data": {
                            "down": down,
                            "key": {"type": "qcode", "data": "a"},
                        },
                    }
                ],
            )
            assert "error" not in response, response
            serial += vm.serial_output(timeout=0.2)
        assert not any(marker in serial for marker in _FATAL_MARKERS), serial[-4000:]
        if "[KBDHID] read completion" in serial and "[KBDCLASS] first key delivered" in serial:
            break
    else:
        pytest.fail(f"xHCI did not deliver a keyboard report:\n{serial[-6000:]}")
    assert "shell ready" in serial
    assert vm.qmp.execute("query-status")["return"]["status"] == "running"
    vm.screendump(vm.work_dir / "xhci-keyboard.ppm")


def test_xhci_mouse_delivery(vm: QemuVM) -> None:
    boot = vm._boot_output + vm.serial_output(timeout=1)
    assert "[XHCI] controller running" in boot
    assert "[MOUHID] mouse connected to class driver" in boot
    serial = ""
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        response = vm.qmp.execute(
            "input-send-event",
            events=[
                {"type": "rel", "data": {"axis": "x", "value": 20}},
                {"type": "rel", "data": {"axis": "y", "value": 10}},
            ],
        )
        assert "error" not in response, response
        serial += vm.serial_output(timeout=0.5)
        assert not any(marker in serial for marker in _FATAL_MARKERS), serial[-4000:]
        if "[MOUHID] report delivered dx=20 dy=10" in serial:
            break
    else:
        pytest.fail(f"xHCI mouse report did not reach mouclass:\n{serial[-6000:]}")
    assert vm.qmp.execute("query-status")["return"]["status"] == "running"
    vm.screendump(vm.work_dir / "xhci-mouse.ppm")
