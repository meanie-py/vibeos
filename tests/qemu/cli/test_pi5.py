"""The Pi 5 bench: its lease is separate from the Pi 4's, and its HDMI verdict."""

from pathlib import Path

import pytest

from . import pi4lease, pi5


@pytest.fixture(autouse=True)
def lock_dir(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path))
    return tmp_path


def test_each_board_has_its_own_lease(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("VIBEOS_BENCH_BOARD", "pi4")
    assert pi4lease.take("/lanes/fex", 10)[0]
    monkeypatch.setenv("VIBEOS_BENCH_BOARD", "pi5")
    assert pi4lease.current() == {}
    assert pi4lease.take("/lanes/graviton", 10)[0]
    monkeypatch.setenv("VIBEOS_BENCH_BOARD", "pi4")
    assert pi4lease.current()["owner"] == "/lanes/fex"
    monkeypatch.setenv("VIBEOS_BENCH_BOARD", "pi6")
    with pytest.raises(ValueError, match="pi6"):
        pi4lease.board()


def test_desktop_share(tmp_path: Path) -> None:
    from PIL import Image

    desk = tmp_path / "desk.png"
    im = Image.new("RGB", (1920, 1080), (255, 255, 255))
    im.paste(pi5.DESKTOP_RGB, (0, 0, 1920, 540))  # half the frame is desktop
    im.save(desk)
    share = pi5.desktop_share(desk)
    assert share is not None and abs(share - 0.5) < 0.02 and share >= pi5.DESKTOP_SHARE
    dark = tmp_path / "dark.png"
    Image.new("RGB", (1920, 1080), (7, 7, 7)).save(dark)
    assert pi5.desktop_share(dark) == 0.0
    assert pi5.desktop_share(tmp_path / "missing.png") is None


def test_stortest_image_is_the_volume_stortest_looks_for(tmp_path: Path) -> None:
    import shutil
    import subprocess

    if not (shutil.which("mkfs.fat") and shutil.which("mtype")):
        pytest.skip("dosfstools/mtools not installed")
    img = tmp_path / "vibeusb.img"
    pi5.stortest_image(img)
    label = subprocess.run(["mlabel", "-i", str(img), "-s", "::"], capture_output=True, text=True)
    assert pi5.STORTEST_LABEL in label.stdout
    marker = subprocess.run(["mtype", "-i", str(img), "::MARKER.TXT"], capture_output=True)
    assert b"vibeos usb e2e marker" in marker.stdout
    assert pi5.stortest_readback(img) is None  # stortest has not written yet


def test_stortest_verdict() -> None:
    running = "[STORTEST] start\n[STORTEST] found VIBEUSB at E:\n"
    assert pi5.stortest_verdict(running)[0] is None
    ok = running + "[STORTEST] write+readback OK (43 bytes)\n[STORTEST] ALL OK drive E:\n"
    assert pi5.stortest_verdict(ok) == ("ok", [*running.split("\n")[:2], *ok.split("\n")[2:4]])
    assert pi5.stortest_verdict("[STORTEST] FAILURES (see above)\n")[0] == "fail"
    assert (
        pi5.stortest_verdict("[STORTEST] FAIL: no VIBEUSB volume found after 120s\n")[0] == "fail"
    )


def test_stage_appends_loader_options_to_every_entry() -> None:
    import os
    import subprocess

    repo = Path(__file__).resolve().parents[3]
    stage = repo / "tools" / "pi5" / "stage.sh"

    def ini(**env: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            ["bash", str(stage), "--freeldr-ini"],
            env={**os.environ, **env},
            capture_output=True,
        )

    # Bytes, not text: the file is CRLF and the CR must survive the edit.
    plain = [ln for ln in ini().stdout.split(b"\n") if ln.startswith(b"Options=")]
    extra = [
        ln
        for ln in ini(VIBEOS_LOADER_OPTIONS="/STORTEST").stdout.split(b"\n")
        if ln.startswith(b"Options=")
    ]
    assert plain and len(extra) == len(plain)
    assert all(e == p[:-1] + b" /STORTEST\r" for p, e in zip(plain, extra, strict=True))
    assert ini(VIBEOS_LOADER_OPTIONS="/X|Y").returncode == 2
