"""Opt-in NVMe regression (VIBEOS_QEMU_NVME=1): nvme.sys brings the controller up,
disk.sys and fastfat mount its namespace, and the desktop reads a file off it and
writes a copy that the host then reads back out of the image."""

from __future__ import annotations

import os
import re
import time

import pytest

from tests.qemu.cli.provenance import check_staleness
from tests.qemu.test_smoke import _FATAL_MARKERS
from tests.qemu.test_xhci import _stick_file, _tap, _type
from tests.qemu.vm import PROJECT_DIR, USB_STICK_FILE, USB_STICK_TEXT, QemuVM, smp_count

_COPY = "COPY.TXT"
# Which letter the namespace gets depends on what else the arch attaches (on
# amd64 it is C:, ahead of the IDE temp disk), so the guest copies on each
# likely one; the drives without the file just print an error.
_LETTERS = "CDEFG"


@pytest.fixture(scope="session", autouse=True)
def nvme_preflight(arch: str) -> None:
    if os.environ.get("VIBEOS_QEMU_NVME") != "1":
        pytest.skip("requires the explicit NVMe rig (VIBEOS_QEMU_NVME=1)")
    media_arch = {"i386-uefi": "i386", "amd64-uefi": "amd64"}.get(arch, arch)
    freshness = check_staleness(PROJECT_DIR, media_arch)
    assert freshness.fresh, f"Rebuild {media_arch} media: {freshness.reasons}"
    # A quiet boot: without it winlogon spends minutes after the desktop launching
    # its test ladder's applets (osk, magnify, ...), and each one takes the focus
    # the Run dialog's keystrokes need.
    tripwire = os.environ.get("VIBEOS_TRIPWIRE", "")
    if "QUIET" not in tripwire.replace(";", " ").split():
        os.environ["VIBEOS_TRIPWIRE"] = f"{tripwire} QUIET".strip()
    # arm64 types through the xHCI rig's USB keyboard. Through the default virtio
    # keyboard the Run dialog never opened (2026-10-06: win32k saw the first key,
    # and nothing after it reached a window).
    if media_arch == "arm64":
        os.environ["VIBEOS_QEMU_XHCI"] = "1"


@pytest.fixture(scope="session")
def vm(nvme_preflight: None, vm: QemuVM) -> QemuVM:
    # Explicit dependency: check media before the parent fixture starts QEMU.
    return vm


def test_nvme_controller_and_namespace(vm: QemuVM) -> None:
    """Bring-up, identify, and the start-time read self-test, from the serial log."""
    serial = vm._boot_output + vm.serial_output(timeout=1)
    for line in (
        "[NVME] controller 0: namespace 1: 131072 blocks of 512 B (64 MiB)",
        "[NVME] controller 0: self-test: partition 1 type 0x06 at block 2048",
        "[NVME] controller 0: namespace 1 published as a disk",
        "[NVME] controller 0: interrupts working",
    ):
        assert line in serial, f"missing {line!r}:\n{serial[-6000:]}"
    # One message for the admin queue and one per CPU for an I/O queue each (at
    # most 8), every one connected.
    messages = 1 + min(smp_count() or 1, 8)
    for line in (
        f"MSI-X enabled for 1b36:0010: {messages} of 65 table entries",
        f"[NVME] controller 0: MSI-X: {messages} message(s) connected",
    ):
        assert line in serial, f"missing {line!r}:\n{serial[-6000:]}"
    queues = re.findall(r"\[NVME\] controller 0: I/O queue \d+ on message \d+", serial)
    assert len(queues) == messages - 1, serial[-6000:]
    assert "self-test: read 1024 KiB" in serial, serial[-6000:]
    # Both messages fired: admin completions on one, the self-test's reads on the other.
    taken = re.search(r"MSI-X messages taken: (\d+) admin, (\d+) I/O", serial)
    assert taken and int(taken[1]) > 0 and int(taken[2]) > 0, serial[-6000:]
    # The megabyte went as one request, cut into commands that were all in flight
    # at once, and moved by DMA to the reader's pages rather than the bounce buffer.
    depth = re.search(r"up to (\d+) commands in flight.*?(\d+) commands by DMA", serial)
    assert depth and int(depth[1]) >= 2 and int(depth[2]) > 0, serial[-6000:]


def test_nvme_read_and_write_from_the_desktop(vm: QemuVM) -> None:
    """nvme -> disk -> partmgr -> mountmgr -> fastfat, end to end: the desktop
    copies the namespace's file to a new one, and the host reads the copy back."""
    image = vm.scratch_path.parent / "nvmedisk.qcow2"
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
    vm.screendump(vm.work_dir / "nvme-copy.ppm")
    assert copied == USB_STICK_TEXT, f"{_COPY} on the namespace: {copied!r}\n{serial[-6000:]}"
    assert vm.qmp.execute("query-status")["return"]["status"] == "running"
