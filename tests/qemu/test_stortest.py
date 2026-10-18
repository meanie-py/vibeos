"""Opt-in stortest regression (VIBEOS_QEMU_STORTEST=1): the USB mass-storage chain from
the top, in QEMU, with no rebuild.

stortest.exe runs only with the STORTEST loader option and finds its stick by the label
VIBEUSB (base/applications/stortest). Before this, only `vibeos-cli pi5 boot --stortest`
reached it, on the bench. Here vm.stortest_selected_iso puts the option on a copy of the
media, and vm.py attaches a VIBEUSB stick holding MARKER.TXT. The guest reads the marker,
writes WRITETST.TXT and reads it back; the verdict is graded from serial, then the write
is read off the stick image on the host, so the medium itself confirms the bytes landed.
"""

from __future__ import annotations

import os
import time

import pytest

from tests.qemu.cli.pi5 import stortest_verdict
from tests.qemu.cli.provenance import check_staleness
from tests.qemu.test_smoke import _FATAL_MARKERS
from tests.qemu.test_xhci import _stick_file
from tests.qemu.vm import PROJECT_DIR, QemuVM

# stortest.c hunts for the volume for 120 s; allow for the boot's tail on top.
_VERDICT_WAIT = 240
_WRITE_CONTENT = b"vibeos wrote this through fastfat+usbstor\r\n"


@pytest.fixture(scope="session", autouse=True)
def stortest_preflight(arch: str) -> None:
    if os.environ.get("VIBEOS_QEMU_STORTEST") != "1":
        pytest.skip("requires VIBEOS_QEMU_STORTEST=1 (the STORTEST option and its stick)")
    media_arch = {"i386-uefi": "i386", "amd64-uefi": "amd64"}.get(arch, arch)
    freshness = check_staleness(PROJECT_DIR, media_arch)
    assert freshness.fresh, f"Rebuild {media_arch} media: {freshness.reasons}"


@pytest.fixture(scope="session")
def vm(stortest_preflight: None, vm: QemuVM) -> QemuVM:
    # Explicit dependency: check media before the parent fixture starts QEMU.
    return vm


def test_stortest_reads_and_writes_the_stick(vm: QemuVM) -> None:
    serial = vm._boot_output + vm.serial_output(timeout=1)
    verdict, lines = stortest_verdict(serial)
    deadline = time.monotonic() + _VERDICT_WAIT
    while verdict is None and time.monotonic() < deadline:
        serial += vm.serial_output(timeout=2)
        assert not any(marker in serial for marker in _FATAL_MARKERS), serial[-4000:]
        verdict, lines = stortest_verdict(serial)
    assert lines, "stortest never started: no [STORTEST] line\n" + serial[-6000:]
    assert verdict == "ok", "\n".join(lines) + "\n" + serial[-4000:]
    written = _stick_file(vm.scratch_path.parent / "usbdisk.qcow2", "WRITETST.TXT")
    assert written == _WRITE_CONTENT, f"WRITETST.TXT on the stick image: {written!r}"
