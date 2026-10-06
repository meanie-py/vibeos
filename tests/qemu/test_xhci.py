"""Opt-in xHCI regressions (VIBEOS_QEMU_XHCI=1): keyboard and mouse delivery,
and with VIBEOS_QEMU_USB_DISK=1 a USB stick read and written from the desktop."""

from __future__ import annotations

import os
import subprocess
import tempfile
import time
from pathlib import Path

import pytest

from tests.qemu.cli.provenance import check_staleness
from tests.qemu.test_smoke import _FATAL_MARKERS
from tests.qemu.vm import PROJECT_DIR, USB_STICK_FILE, USB_STICK_TEXT, QemuVM


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
        # kbdclass accepting the key is not enough: win32k must be reading that
        # class device. On arm64 it once was not (arm64input owned KeyboardClass0).
        if (
            "[KBDHID] read completion" in serial
            and "[KBDCLASS] first key delivered" in serial
            and "[WIN32K] RIT first keyboard input" in serial
        ):
            break
    else:
        pytest.fail(f"xHCI did not deliver a keyboard report to the desktop:\n{serial[-6000:]}")
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


# US-layout qcodes for the characters the stick test types.
_QCODE = {
    " ": ["spc"],
    ":": ["shift", "semicolon"],
    "\\": ["backslash"],
    ".": ["dot"],
    "/": ["slash"],
    "&": ["shift", "7"],
    ">": ["shift", "dot"],
}


# Every letter a stick can be given: two sticks plus the ramdisk if it ever
# takes one of these instead of X:.
_STICK_LETTERS = "CDE"


def _tap(vm: QemuVM, keys: list[str]) -> None:
    for down, order in ((True, keys), (False, list(reversed(keys)))):
        for k in order:
            event = {"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": k}}}
            response = vm.qmp.execute("input-send-event", events=[event])
            assert "error" not in response, response
            time.sleep(0.04)
    time.sleep(0.06)


def _type(vm: QemuVM, text: str) -> None:
    for ch in text:
        if ch in _QCODE:
            _tap(vm, _QCODE[ch])
        elif ch.isupper():
            _tap(vm, ["shift", ch.lower()])
        else:
            _tap(vm, [ch])


def _stick_file(image: Path, name: str) -> bytes | None:
    """Read one file off the stick image while QEMU runs (-U: shared read)."""
    with tempfile.TemporaryDirectory() as td:
        raw = Path(td) / "stick.raw"
        subprocess.run(
            ["qemu-img", "convert", "-U", "-O", "raw", str(image), str(raw)],
            check=True,
            capture_output=True,
        )
        out = Path(td) / name
        r = subprocess.run(
            ["mcopy", "-n", "-i", f"{raw}@@1M", f"::/{name}", str(out)], capture_output=True
        )
        return out.read_bytes() if r.returncode == 0 else None


def test_usb_stick_read_and_write(vm: QemuVM) -> None:
    """usbstor -> disk -> partmgr -> mountmgr -> fastfat, end to end: the desktop
    copies the stick's file to a new one, and the host reads the copy back."""
    if os.environ.get("VIBEOS_QEMU_USB_DISK") != "1":
        pytest.skip("requires VIBEOS_QEMU_USB_DISK=1 (the stick)")
    serial = vm._boot_output + vm.serial_output(timeout=1)
    assert "(drv \\Driver\\usbstor)" in serial, serial[-6000:]
    # Two sticks (vm.py): one behind a full-speed hub, one SuperSpeed on a USB3
    # root port, lettered in whichever order they arrive. The boot ramdisk is NOT
    # C: -- a MININT boot puts it at X: and fstub hands C: to the first hard disk
    # -- so try every letter a stick can get (_STICK_LETTERS); the ramdisk has no
    # source file, so a copy there fails harmlessly.
    assert "speed 4" in serial, "no SuperSpeed device was addressed:\n" + serial[-6000:]
    images = [vm.scratch_path.parent / "usbdisk.qcow2", vm.scratch_path.parent / "usbdisk-ss.qcow2"]
    _tap(vm, ["meta_l", "r"])
    time.sleep(3)
    _type(
        vm,
        "cmd /c "
        + " & ".join(f"copy {d}:\\{USB_STICK_FILE} {d}:\\COPY.TXT" for d in _STICK_LETTERS),
    )
    _tap(vm, ["ret"])
    copied: list[bytes | None] = [None, None]
    deadline = time.monotonic() + 60
    while time.monotonic() < deadline and None in copied:
        serial += vm.serial_output(timeout=2)
        assert not any(marker in serial for marker in _FATAL_MARKERS), serial[-4000:]
        copied = [_stick_file(image, "COPY.TXT") for image in images]
    vm.screendump(vm.work_dir / "xhci-stick.ppm")
    assert copied == [USB_STICK_TEXT, USB_STICK_TEXT], (
        f"COPY.TXT on (hub, SuperSpeed): {copied!r}\n{serial[-6000:]}"
    )
    assert vm.qmp.execute("query-status")["return"]["status"] == "running"


def test_usb_stick_cmd_redirection(vm: QemuVM) -> None:
    """An interactive cmd's `>` lands a file on the stick and cmd takes the next
    line. On arm64 (the Pi 4) `echo hi > F:\\x.txt` typed at the prompt once left
    cmd parked in NtRequestWaitReplyPort with no file created, while `copy` to
    the same volume worked: redirection, not storage."""
    if os.environ.get("VIBEOS_QEMU_USB_DISK") != "1":
        pytest.skip("requires VIBEOS_QEMU_USB_DISK=1 (the stick)")
    serial = vm._boot_output + vm.serial_output(timeout=1)
    images = [vm.scratch_path.parent / "usbdisk.qcow2", vm.scratch_path.parent / "usbdisk-ss.qcow2"]
    _tap(vm, ["meta_l", "r"])
    time.sleep(3)
    _type(vm, "cmd")
    _tap(vm, ["ret"])
    time.sleep(5)
    # Each stick must get both files. The second round is typed only after the
    # first redirections, so it proves cmd came back to its prompt.
    for name in ("REDIR.TXT", "AGAIN.TXT"):
        for letter in _STICK_LETTERS:
            _type(vm, f"echo REDIR>{letter}:\\{name}")
            _tap(vm, ["ret"])
            time.sleep(1)
    want = [b"REDIR\r\n"] * 2
    got: list[list[bytes | None]] = [[None, None], [None, None]]
    deadline = time.monotonic() + 60
    while time.monotonic() < deadline and got != [want, want]:
        serial += vm.serial_output(timeout=2)
        assert not any(marker in serial for marker in _FATAL_MARKERS), serial[-4000:]
        got = [[_stick_file(image, n) for n in ("REDIR.TXT", "AGAIN.TXT")] for image in images]
    vm.screendump(vm.work_dir / "xhci-redirect.ppm")
    assert got == [want, want], f"(REDIR, AGAIN) on (hub, SuperSpeed): {got!r}\n{serial[-6000:]}"
    assert vm.qmp.execute("query-status")["return"]["status"] == "running"
