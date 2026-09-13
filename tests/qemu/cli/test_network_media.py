"""Networking launched by smoke must not rebuild the media it just validated."""

import subprocess
from types import SimpleNamespace
from typing import cast

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
        # Model the raw-pytest clean-build fixture that rewrites the shared ISO.
        if "--skip-clean-build" not in cmd:
            media.write_bytes(b"partially rewritten ISO")
        return subprocess.CompletedProcess(cmd, 0, "15 passed in 1s", "")

    monkeypatch.setattr(smoke.subprocess, "run", pytest_fixture)
    smoke._net_stage(
        cast(Run, SimpleNamespace(add=stages.append, log=lambda text, **_: logs.append(text))),
        tmp_path,
        "amd64",
    )
    assert media.read_bytes() == b"validated media"
    assert stages[0].status == "ok"
    assert "15 passed" in logs[0]
