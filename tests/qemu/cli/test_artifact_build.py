"""Artifact supervision must survive gates and invalidate rejected build stamps."""

import json
import subprocess
from contextlib import contextmanager
from types import SimpleNamespace
from typing import cast

import pytest

from tools import artifact_runtime

from . import build, gate, provenance
from .artifacts import Run


@pytest.mark.parametrize(
    "arch,restage,expected",
    [
        ("amd64", False, True),
        ("amd64", True, False),
        ("i386", False, True),
        ("arm64", True, False),
    ],
)
def test_gate_forwards_store_without_reverifying_restage(tmp_path, arch, restage, expected):
    calls = []

    def execute(cmd, **kwargs):
        calls.append(cmd)
        return subprocess.CompletedProcess(cmd, 0, "")

    args = SimpleNamespace(artifact_store=tmp_path / "store", artifact_store_verify=True)
    run = cast(Run, SimpleNamespace(run=execute))
    result = gate._build_once(tmp_path, run, arch, args, restage=restage)
    assert result["ok"]
    assert "--artifact-store" in calls[0]
    assert ("--artifact-store-verify" in calls[0]) == expected


def test_failed_postcheck_removes_successful_build_stamp(tmp_path, monkeypatch):
    _no_provisioning(monkeypatch)
    supervisor = tmp_path / "supervisor.json"
    supervisor.write_text(json.dumps({"toolchain_key": "fixture"}))
    output = tmp_path / "build"
    output.mkdir()
    stamp = output / provenance.STAMP_NAME

    @contextmanager
    def supervise(*args, **kwargs):
        yield supervisor
        raise RuntimeError("toolchain changed")

    def compiled(*args, **kwargs):
        stamp.write_text("otherwise valid")
        return 0

    monkeypatch.setattr(artifact_runtime, "supervise", supervise)
    monkeypatch.setattr(artifact_runtime, "record_outer_result", lambda *args: None)
    monkeypatch.setattr(build, "_build_locked", compiled)
    monkeypatch.delenv("VIBEOS_RUST_INCREMENTAL", raising=False)
    stages = []
    run = SimpleNamespace(dir=tmp_path, extra={}, add=stages.append, finalize=lambda repo: 2)
    args = SimpleNamespace(artifact_store=tmp_path / "store", artifact_store_verify=True)
    assert build._build_supervised(args, tmp_path, run, "amd64", None, output, False) == 2
    assert run.classification == "ARTIFACT_STORE_FAIL"
    assert not stamp.exists()
    assert "toolchain changed" in stages[-1].detail


def _no_provisioning(monkeypatch):
    """The supervised build provisions the toolchain and broker first; not here."""
    from . import build

    monkeypatch.setattr(build.canon, "ensure_toolchain", lambda repo: True)
    monkeypatch.setattr(build.hostjobs, "ensure", lambda repo: None)
