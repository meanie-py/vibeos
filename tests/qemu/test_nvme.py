"""Opt-in NVMe regression (VIBEOS_QEMU_NVME=1): nvme.sys brings the controller up,
disk.sys and fastfat mount its namespace, and the desktop reads a file off it and
writes a copy that the host then reads back out of the image."""

from __future__ import annotations

import os
import time

import pytest

from tests.qemu.cli.provenance import check_staleness
from tests.qemu.test_smoke import _FATAL_MARKERS
from tests.qemu.test_xhci import _QCODE, _stick_file, _tap
from tests.qemu.vm import PROJECT_DIR, USB_STICK_FILE, USB_STICK_TEXT, QemuVM

# The guest command looks for the file on every likely drive letter, since
# which one the namespace gets depends on what else the arch attaches.
_KEYS = {**_QCODE, "%": ["shift", "5"], "(": ["shift", "9"], ")": ["shift", "0"]}
_COPY = "COPY.TXT"


@pytest.fixture(scope="session", autouse=True)
def nvme_preflight(arch: str) -> None:
    if os.environ.get("VIBEOS_QEMU_NVME") != "1":
        pytest.skip("requires the explicit NVMe rig (VIBEOS_QEMU_NVME=1)")
    media_arch = {"i386-uefi": "i386", "amd64-uefi": "amd64"}.get(arch, arch)
    freshness = check_staleness(PROJECT_DIR, media_arch)
    assert freshness.fresh, f"Rebuild {media_arch} media: {freshness.reasons}"


@pytest.fixture(scope="session")
def vm(nvme_preflight: None, vm: QemuVM) -> QemuVM:
    # Explicit dependency: check media before the parent fixture starts QEMU.
    return vm


def _type(vm: QemuVM, text: str) -> None:
    for ch in text:
        if ch in _KEYS:
            _tap(vm, _KEYS[ch])
        elif ch.isupper():
            _tap(vm, ["shift", ch.lower()])
        else:
            _tap(vm, [ch])


def test_nvme_controller_and_namespace(vm: QemuVM) -> None:
    """Bring-up, identify, and the start-time read self-test, from the serial log."""
    serial = vm._boot_output + vm.serial_output(timeout=1)
    for line in (
        "[NVME] controller 0: namespace 1: 131072 blocks of 512 B (64 MiB)",
        "[NVME] controller 0: self-test: partition 1 type 0x06 at block 2048",
        "[NVME] controller 0: namespace 1 published as a disk",
    ):
        assert line in serial, f"missing {line!r}:\n{serial[-6000:]}"
    assert "self-test: read 1024 KiB" in serial, serial[-6000:]


def test_nvme_read_and_write_from_the_desktop(vm: QemuVM) -> None:
    """nvme -> disk -> partmgr -> mountmgr -> fastfat, end to end: the desktop
    copies the namespace's file to a new one, and the host reads the copy back."""
    image = vm.scratch_path.parent / "nvmedisk.qcow2"
    serial = vm._boot_output + vm.serial_output(timeout=1)
    _tap(vm, ["meta_l", "r"])
    time.sleep(3)
    _type(vm, f"cmd /c for %d in (C D E F G H) do copy %d:\\{USB_STICK_FILE} %d:\\{_COPY}")
    _tap(vm, ["ret"])
    copied = None
    deadline = time.monotonic() + 90
    while time.monotonic() < deadline and copied is None:
        serial += vm.serial_output(timeout=2)
        assert not any(marker in serial for marker in _FATAL_MARKERS), serial[-4000:]
        copied = _stick_file(image, _COPY)
    vm.screendump(vm.work_dir / "nvme-copy.ppm")
    assert copied == USB_STICK_TEXT, f"{_COPY} on the namespace: {copied!r}\n{serial[-6000:]}"
    assert vm.qmp.execute("query-status")["return"]["status"] == "running"
