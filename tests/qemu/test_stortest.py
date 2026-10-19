"""Opt-in stortest regression (VIBEOS_QEMU_STORTEST=1): the USB mass-storage chain from
the top, in QEMU, with no rebuild.

stortest.exe runs only with the STORTEST loader option and finds its stick by the label
VIBEUSB (base/applications/stortest). Before this, only `vibeos-cli pi5 boot --stortest`
reached it, on the bench. Here vm.stortest_selected_iso puts the option on a copy of the
media, and vm.py attaches a VIBEUSB stick holding MARKER.TXT. The guest reads the marker,
writes WRITETST.TXT and reads it back; the verdict is graded from serial, then the write
is read off the stick image on the host, so the medium itself confirms the bytes landed.

With VIBEOS_STORTEST_RESULT=<path> (the gate's stor:<arch> lane) the test also writes its
verdict there as JSON, {classification, salient}, so a red lane names what failed
(BUGCHECK, HANG_TIMEOUT, STORTEST_FAIL, STORTEST_MISMATCH) instead of "pytest exit 1".
A boot that never reaches the desktop fails in the vm fixture and writes nothing; the
gate classifies that one from the console.
"""

from __future__ import annotations

import json
import os
import time
from pathlib import Path

import pytest

from tests.qemu.cli import triage
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
    if not freshness.fresh:
        _record("STALE_MEDIA", list(freshness.reasons))
    assert freshness.fresh, f"Rebuild {media_arch} media: {freshness.reasons}"


@pytest.fixture(scope="session")
def vm(stortest_preflight: None, vm: QemuVM) -> QemuVM:
    # Explicit dependency: check media before the parent fixture starts QEMU.
    return vm


def _record(classification: str, salient: list[str]) -> None:
    out = os.environ.get("VIBEOS_STORTEST_RESULT")
    if out:
        Path(out).write_text(json.dumps({"classification": classification, "salient": salient}))


def _grade(vm: QemuVM) -> tuple[str, list[str], str]:
    """(classification, salient, serial): "" when stortest passed and its write landed."""
    serial = vm._boot_output + vm.serial_output(timeout=1)
    verdict, lines = stortest_verdict(serial)
    deadline = time.monotonic() + _VERDICT_WAIT
    while verdict is None and time.monotonic() < deadline:
        serial += vm.serial_output(timeout=2)
        if any(marker in serial for marker in _FATAL_MARKERS):
            v = triage.classify(serial, reached=True, fail_marker=None, debugcon=None)
            return v["classification"] or "BUGCHECK", list(v.get("salient", [])), serial
        verdict, lines = stortest_verdict(serial)
    if not lines:
        return "STORTEST_NOT_STARTED", ["no [STORTEST] line: did winlogon start it?"], serial
    if verdict is None:
        return "HANG_TIMEOUT", [f"no stortest verdict {_VERDICT_WAIT}s in", *lines[-4:]], serial
    if verdict != "ok":
        return "STORTEST_FAIL", lines[-6:], serial
    written = _stick_file(vm.scratch_path.parent / "usbdisk.qcow2", "WRITETST.TXT")
    if written != _WRITE_CONTENT:
        return "STORTEST_MISMATCH", [f"WRITETST.TXT on the stick image: {written!r}"], serial
    return "", [lines[-1]], serial


def test_stortest_reads_and_writes_the_stick(vm: QemuVM) -> None:
    classification, salient, serial = _grade(vm)
    _record(classification, salient)
    assert not classification, f"{classification}: " + "\n".join(salient) + "\n" + serial[-4000:]
