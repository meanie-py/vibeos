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
