import json
import time
from pathlib import Path

import pytest

from . import pi4, pi4lease


@pytest.fixture(autouse=True)
def lock_dir(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path))
    monkeypatch.delenv("VIBEOS_PI4_OWNER", raising=False)
    return tmp_path


def test_lease_take_refuse_renew_drop() -> None:
    ok, lease = pi4lease.take("/lanes/a", 10, "booting")
    assert ok and lease["owner"] == "/lanes/a"
    ok, held = pi4lease.take("/lanes/b", 10)
    assert not ok and held["owner"] == "/lanes/a"
    assert pi4lease.check("/lanes/b") == (False, held)
    assert pi4lease.check("/lanes/a")[0]
    # A renewal keeps the reason and never shortens the lease.
    ok, renewed = pi4lease.take("/lanes/a", 1)
    assert ok and renewed["why"] == "booting" and renewed["expires"] >= lease["expires"]
    assert pi4lease.drop("/lanes/b") == (False, renewed)
    assert pi4lease.drop("/lanes/a")[0]
    assert pi4lease.current() == {} and pi4lease.describe({}) == "free"


def test_expired_or_garbage_lease_is_free(lock_dir: Path) -> None:
    pi4lease.lease_file().write_text(json.dumps({"owner": "/x", "expires": time.time() - 1}))
    assert pi4lease.current() == {}
    assert pi4lease.take("/lanes/b", 5)[0]
    pi4lease.lease_file().write_text("{not json")
    assert pi4lease.current() == {}


def test_force_drop_releases_another_lane() -> None:
    pi4lease.take("/lanes/a", 10)
    assert pi4lease.drop("/lanes/b", force=True)[0]
    assert pi4lease.current() == {}


def test_script_entry_point(monkeypatch: pytest.MonkeyPatch, capsys) -> None:
    monkeypatch.setenv("VIBEOS_PI4_OWNER", "/lanes/a")
    assert pi4lease.main(["pi4lease.py", "touch"]) == 0
    monkeypatch.setenv("VIBEOS_PI4_OWNER", "/lanes/b")
    assert pi4lease.main(["pi4lease.py", "check"]) == pi4lease.REFUSED
    assert pi4lease.main(["pi4lease.py", "touch"]) == pi4lease.REFUSED
    assert "held by /lanes/a" in capsys.readouterr().err
    assert pi4lease.main(["pi4lease.py", "show"]) == 0
    assert pi4lease.main(["pi4lease.py", "bogus"]) == 2


def test_parse_usb_and_power() -> None:
    usb = pi4.parse_usb("uptime=623s mounted=True suspended=False resets=2 completed=[3, 0, 0]")
    assert usb["mounted"] is True and usb["suspended"] is False and usb["uptime"] == "623s"
    assert usb["completed"] == "[3, 0, 0]"
    bridged = pi4.parse_usb("uptime=5s mounted=True suspended=False target_powered=False")
    assert bridged["target_powered"] is False
    assert pi4.parse_usb('{"ok": false, "error": "unreachable", "exit_code": 3}') == {
        "error": "unreachable"
    }
    assert "error" in pi4.parse_usb("")
    assert pi4.parse_power("on  4.0 W") == {"on": True, "watts": 4.0}
    assert pi4.parse_power("off 0.0 W") == {"on": False, "watts": 0.0}
    assert pi4.parse_power("curl: (7) failed") == {}


def test_serial_file_accepts_both_listeners() -> None:
    windows = {"directory": "D:\\home\\vibeos-rpi4-setup\\serial-1"}
    assert pi4.serial_file(windows) == Path("/mnt/d/home/vibeos-rpi4-setup/serial-1/serial.bin")
    wsl = {"directory": "/mnt/d/home/vibeos-rpi4-setup/serial-2"}
    assert pi4.serial_file(wsl) == Path("/mnt/d/home/vibeos-rpi4-setup/serial-2/serial.bin")
    assert pi4.serial_file({}) is None


def test_wait_for_desktop_reads_only_this_boot(tmp_path: Path) -> None:
    serial = tmp_path / "serial.bin"
    serial.write_bytes(b"old boot shell ready\n")
    start = serial.stat().st_size
    with open(serial, "ab") as f:
        f.write(b"[KERNEL] up\n[EXPLORER] shell ready\n")
    text, reached = pi4.wait_for_desktop(serial, start, timeout=1)
    assert reached and "old boot" not in text
    serial.write_bytes(b"shell ready from before\n")
    text, reached = pi4.wait_for_desktop(serial, serial.stat().st_size, timeout=0.1)
    assert not reached and text == ""
