"""Opt-in virtio-mmio disk regression (VIBEOS_QEMU_VIRTIO_MMIO_DISK=1, arm64): the
HAL publishes the slot from ACPI, vioblk's PnP path brings it up on its interrupt,
disk.sys and fastfat mount it, and the desktop reads a file off it and writes a
copy that the host then reads back out of the image."""

from __future__ import annotations

import os
import re
import time

import pytest

from tests.qemu.cli.provenance import check_staleness
from tests.qemu.test_smoke import _FATAL_MARKERS
from tests.qemu.test_xhci import _stick_file, _tap, _type
from tests.qemu.vm import PROJECT_DIR, USB_STICK_FILE, USB_STICK_TEXT, QemuVM

_COPY = "COPY.TXT"
# The letter depends on what else is mounted first, so the guest copies on each
# likely one; the drives without the file just print an error.
_LETTERS = "CDEFG"


@pytest.fixture(scope="session", autouse=True)
def virtio_mmio_preflight(arch: str) -> None:
    if not arch.startswith("arm64"):
        pytest.skip("virtio-mmio disks are arm64's (QEMU virt)")
    if os.environ.get("VIBEOS_QEMU_VIRTIO_MMIO_DISK") != "1":
        pytest.skip("requires the explicit rig (VIBEOS_QEMU_VIRTIO_MMIO_DISK=1)")
    freshness = check_staleness(PROJECT_DIR, "arm64")
    assert freshness.fresh, f"Rebuild arm64 media: {freshness.reasons}"
    # A quiet boot and the xHCI keyboard, for the reasons test_nvme.py gives.
    tripwire = os.environ.get("VIBEOS_TRIPWIRE", "")
    if "QUIET" not in tripwire.replace(";", " ").split():
        os.environ["VIBEOS_TRIPWIRE"] = f"{tripwire} QUIET".strip()
    os.environ["VIBEOS_QEMU_XHCI"] = "1"


@pytest.fixture(scope="session")
def vm(virtio_mmio_preflight: None, vm: QemuVM) -> QemuVM:
    # Explicit dependency: check media before the parent fixture starts QEMU.
    return vm


def test_virtio_mmio_disk_up_on_interrupts(vm: QemuVM) -> None:
    """Publication, bring-up and the start-time read self-test, from the serial log."""
    serial = vm._boot_output + vm.serial_output(timeout=1)
    assert "[HAL] ACPI virtio-mmio block (LNRO0005 #" in serial, serial[-6000:]
    assert ": 131072 sectors (64 MiB)" in serial, serial[-6000:]
    # The rig's disk is the one with a partition table; the blank scratch disk
    # reports none.
    rig = re.search(
        r"virtio-mmio (0x[0-9a-f]+): self-test: read 64 KiB, block 0 holds an MBR", serial
    )
    assert rig is not None, serial[-6000:]
    assert f"[VIOBLK] virtio-mmio {rig.group(1)}: interrupts working" in serial, serial[-6000:]


def test_virtio_mmio_read_and_write_from_the_desktop(vm: QemuVM) -> None:
    """vioblk -> disk -> partmgr -> mountmgr -> fastfat, end to end: the desktop
    copies the disk's file to a new one, and the host reads the copy back."""
    image = vm.scratch_path.parent / "viommiodisk.qcow2"
    serial = vm._boot_output + vm.serial_output(timeout=1)
    _tap(vm, ["meta_l", "r"])
    time.sleep(3)
    _type(
        vm, "cmd /c " + " & ".join(f"copy {d}:\\{USB_STICK_FILE} {d}:\\{_COPY}" for d in _LETTERS)
    )
    _tap(vm, ["ret"])
    copied = None
    deadline = time.monotonic() + 90
    while time.monotonic() < deadline and copied is None:
        serial += vm.serial_output(timeout=2)
        assert not any(marker in serial for marker in _FATAL_MARKERS), serial[-4000:]
        copied = _stick_file(image, _COPY)
    vm.screendump(vm.work_dir / "viommio-copy.ppm")
    assert copied == USB_STICK_TEXT, f"{_COPY} on the disk: {copied!r}\n{serial[-6000:]}"
    assert vm.qmp.execute("query-status")["return"]["status"] == "running"
