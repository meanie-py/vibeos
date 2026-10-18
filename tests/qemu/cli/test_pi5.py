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


def test_stortest_readback_reads_a_volume_left_dirty(tmp_path: Path) -> None:
    """The board is power-cut with the volume mounted, so fastfat's FAT16 dirty flag is
    still set (entry 1 = 0x7FFF in both FATs) and plain mtools refuses the image. The
    2026-10-17 Pi 5 run failed its readback exactly so, over the right 43 bytes."""
    import shutil
    import subprocess

    if not (shutil.which("mkfs.fat") and shutil.which("mtype")):
        pytest.skip("dosfstools/mtools not installed")
    img = tmp_path / "vibeusb.img"
    pi5.stortest_image(img)
    wrote = tmp_path / "WRITETST.TXT"
    wrote.write_bytes(pi5.STORTEST_WROTE)
    subprocess.run(["mcopy", "-i", str(img), str(wrote), "::WRITETST.TXT"], check=True)
    data = bytearray(img.read_bytes())
    reserved = int.from_bytes(data[14:16], "little")
    per_fat = int.from_bytes(data[22:24], "little")
    for fat in range(data[16]):
        at = (reserved + fat * per_fat) * 512 + 2
        data[at : at + 2] = b"\xff\x7f"
    img.write_bytes(data)
    plain = subprocess.run(["mtype", "-i", str(img), "::WRITETST.TXT"], capture_output=True)
    assert plain.returncode != 0, "fixture no longer reproduces mtools' refusal"
    assert pi5.stortest_readback(img) == pi5.STORTEST_WROTE


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


def test_a_lost_image_server_is_the_verdict_not_a_stortest_hang(tmp_path: Path) -> None:
    """2026-10-16: the image server's websocket closed a minute into the boot, the Pico
    answered every READ with MEDIUM NOT PRESENT, and the run reported HANG_TIMEOUT 240s
    later. The wait now stops when the server exits and says so."""
    import subprocess
    import sys
    import time

    class FakeRun:
        def __init__(self) -> None:
            self.dir = tmp_path
            self.extra: dict = {}
            self.stages: list = []
            self.classification = ""

        def add(self, stage) -> None:
            self.stages.append(stage)

    log = tmp_path / "serve-image.log"
    log.write_text('Serving media.\n{"ok": false, "error": "other", "exit_code": 1}\n')
    server = subprocess.Popen([sys.executable, "-c", "import sys; sys.exit(1)"])
    watch = pi5.ImageServerWatch(server, log)
    serial = tmp_path / "live.log"
    serial.write_text("[STORTEST] start\n[STORTEST] next-letter Harddisk3 failed (error 3)\n")
    run = FakeRun()
    t0 = time.monotonic()
    pi5._await_stortest(run, serial, 0, watch)  # type: ignore[arg-type]
    assert time.monotonic() - t0 < 30  # not STORTEST_WAIT
    assert run.classification == "BENCH_UNHEALTHY"
    assert run.stages[-1].name == "serve-image"
    assert "image server exited 1" in run.stages[-1].detail


def test_until_reads_on_past_the_desktop_and_says_when_the_line_never_comes(tmp_path: Path) -> None:
    class FakeRun:
        def __init__(self) -> None:
            self.dir = tmp_path
            self.extra: dict = {}
            self.stages: list = []
            self.classification = ""

        def add(self, stage) -> None:
            self.stages.append(stage)

    serial = tmp_path / "live.log"
    serial.write_text("[WIN32K] shell ready\n[BWFM] joining 'x'\n[BWFM] 4-way handshake complete\n")
    run = FakeRun()
    pi5._await_until(run, serial, 0, "[BWFM] 4-way", 5)  # type: ignore[arg-type]
    assert (run.classification, run.stages[-1].status) == ("", "ok")
    assert run.stages[-1].detail == "[BWFM] 4-way handshake complete"
    run = FakeRun()
    pi5._await_until(run, serial, 0, "[BWFM] link up", 0)  # type: ignore[arg-type]
    assert run.classification == "HANG_TIMEOUT"
    assert "not seen" in run.stages[-1].detail and run.stages[-1].salient[-1].startswith(
        "[BWFM] 4-way"
    )
