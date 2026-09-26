"""Networking launched by smoke must not rebuild the media it just validated."""

import subprocess
from types import SimpleNamespace
from typing import cast

import pytest

from .. import backend
from ..vm import QemuVM, get_arch_config
from . import smoke
from .artifacts import Run


def test_network_stage_preserves_validated_media(tmp_path, monkeypatch):
    test = tmp_path / "tests/qemu/test_net.py"
    test.parent.mkdir(parents=True)
    test.write_text("")
    media = tmp_path / "livecd.iso"
    media.write_bytes(b"validated media")
    stages = []
    logs = []

    def pytest_fixture(cmd, **kwargs):
        assert kwargs["env"]["VIBEOS_ARTIFACT_DIR"] == str(tmp_path / "net")
        # Model the raw-pytest clean-build fixture that rewrites the shared ISO.
        if "--skip-clean-build" not in cmd:
            media.write_bytes(b"partially rewritten ISO")
        return subprocess.CompletedProcess(cmd, 0, "15 passed in 1s", "")

    monkeypatch.setattr(smoke.subprocess, "run", pytest_fixture)
    smoke._net_stage(
        cast(
            Run,
            SimpleNamespace(
                dir=tmp_path, add=stages.append, log=lambda text, **_: logs.append(text)
            ),
        ),
        tmp_path,
        "amd64",
    )
    assert media.read_bytes() == b"validated media"
    assert stages[0].status == "ok"
    assert "15 passed" in logs[0]


def test_same_arch_vms_keep_independent_serial_logs(tmp_path, monkeypatch):
    monkeypatch.delenv("VIBEOS_ARTIFACT_DIR", raising=False)
    first = QemuVM(get_arch_config("amd64-uefi"), tmp_path / "first")
    second = QemuVM(get_arch_config("amd64-uefi"), tmp_path / "second")
    first.live_log_path.write_text("first boot")
    second.live_log_path.write_text("second boot")
    assert first.live_log_path.read_text() == "first boot"
    monkeypatch.setenv("VIBEOS_ARTIFACT_DIR", str(tmp_path / "durable"))
    durable = QemuVM(get_arch_config("amd64-uefi"), tmp_path / "third")
    assert durable.live_log_path == tmp_path / "durable/amd64-uefi_serial.log"


def test_dead_qemu_does_not_drain_serial_for_another_hour():
    vm = cast(
        QemuVM,
        SimpleNamespace(
            serial=SimpleNamespace(read_available=lambda **_: ""),
            _proc=SimpleNamespace(poll=lambda: 1, returncode=1),
        ),
    )
    with pytest.raises(ConnectionError, match="QEMU exited with code 1"):
        QemuVM.serial_output(vm)


def test_staging_avoids_sendfile_and_publishes_complete_media(tmp_path, monkeypatch):
    monkeypatch.setattr(backend, "win_path", str)
    monkeypatch.setattr(backend.os, "sendfile", lambda *a: pytest.fail("bulk sendfile used"))
    source = tmp_path / "boot.img"
    source.write_bytes(bytes(range(256)) * 4096 + b"partial final block")
    staging = tmp_path / "staging"
    destination = backend.stage_file(source, staging)
    assert (staging / "boot.img").read_bytes() == source.read_bytes()
    assert destination == str(staging / "boot.img")
    assert list(staging.iterdir()) == [staging / "boot.img"]


@pytest.mark.parametrize("failure", ["write", "changed-source"])
def test_staging_never_publishes_partial_or_mutating_media(tmp_path, monkeypatch, failure):
    monkeypatch.setattr(backend, "win_path", str)
    source = tmp_path / "boot.img"
    source.write_bytes(b"new complete media")
    staging = tmp_path / "staging"
    staging.mkdir()
    destination = staging / "boot.img"
    destination.write_bytes(b"previous complete media")

    def broken_copy(reader, writer, **kwargs):
        writer.write(reader.read(3))
        if failure == "write":
            raise OSError("injected write failure")
        source.write_bytes(b"changed during copy")

    monkeypatch.setattr(backend.shutil, "copyfileobj", broken_copy)
    with pytest.raises((OSError, RuntimeError)):
        backend.stage_file(source, staging)
    assert destination.read_bytes() == b"previous complete media"
    assert list(staging.iterdir()) == [destination]
