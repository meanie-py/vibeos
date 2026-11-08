"""Headers-only preparation retains build guards and never stamps boot media."""

import subprocess
from contextlib import nullcontext
from pathlib import Path
from types import SimpleNamespace

from . import build


class FakeRun:
    last = None

    def __init__(self, command, arch):
        FakeRun.last = self
        self.command, self.arch = command, arch
        self.classification, self.extra, self.stages = "", {}, []
        self.dir = Path("/tmp/headers-test")
        self.commands = []
        self.finalize_hooks = []

    def add(self, stage):
        self.stages.append(stage)

    def finalize(self, repo):
        return 1 if any(s.status == "fail" for s in self.stages) else 0

    def run(self, cmd, **kwargs):
        self.commands.append((cmd, kwargs))
        return subprocess.CompletedProcess(cmd, 0, "", "")


def test_headers_build_invalidates_stamp_and_skips_full_lifecycle(tmp_path, monkeypatch):
    d = tmp_path / "build-amd64"
    d.mkdir()
    stamp = d / build.provenance.STAMP_NAME
    stamp.write_text("old")
    monkeypatch.setattr(build, "Run", FakeRun)
    monkeypatch.setattr(build.hostlock, "build_lock", lambda *a: nullcontext("admitted"))
    monkeypatch.setattr(build.hostjobs, "ensure", lambda *a: None)
    monkeypatch.setattr(build.diskspace, "reserve", lambda *a: nullcontext())

    def locked(args, repo, run, arch, cfg, build_dir, cold):
        assert not stamp.exists()
        assert args.no_wow64_stage
        return 0

    monkeypatch.setattr(build, "_build_locked", locked)

    def forbidden(*args, **kwargs):
        raise AssertionError("full build operation reached")

    monkeypatch.setattr(build, "_build_supervised", forbidden)
    monkeypatch.setattr(build, "_trim_own_target", forbidden)
    monkeypatch.setattr(build, "_scheduled_store_gc", forbidden)
    args = SimpleNamespace(arch="amd64", clean=False, headers_only=True)
    assert build.build(args, tmp_path) == 0
    assert FakeRun.last is not None
    assert FakeRun.last.command == "build-headers"
    assert FakeRun.last.extra["partial"] == "headers-only"


def test_partial_refuses_incompatible_option_before_lock(tmp_path, monkeypatch):
    monkeypatch.setattr(build, "Run", FakeRun)

    def forbidden(*args, **kwargs):
        raise AssertionError("lock or build reached")

    monkeypatch.setattr(build.hostlock, "build_lock", forbidden)
    args = SimpleNamespace(arch="i386", clean=True, headers_only=True)
    assert build.build(args, tmp_path) == 1
    assert FakeRun.last is not None
    assert FakeRun.last.classification == "HEADERS_OPTIONS_REFUSED"


def test_headers_stops_after_prime(tmp_path, monkeypatch):
    run = FakeRun("build-headers", "amd64")
    args = SimpleNamespace(headers_only=True, no_wow64_stage=True, incremental=False)
    monkeypatch.setattr(build.warmth, "worktree_tree", lambda *a: "tree")
    monkeypatch.setattr(build.canon, "available", lambda: False)
    monkeypatch.setattr(build.cachecfg, "make", lambda *a: None)
    monkeypatch.setattr(build.cachecfg, "cache_enabled", lambda *a: False)
    monkeypatch.setattr(build.hostlock.buildpolicy, "read", lambda *a: None)
    monkeypatch.setattr(build.hostjobs, "environment", lambda *a, **k: {})
    monkeypatch.setattr(build.canon, "is_canonical", lambda *a: False)
    monkeypatch.setattr(build, "source_date_epoch", lambda *a: None)

    assert build._build_locked(args, tmp_path, run, "amd64", None, tmp_path, False) == 0
    # the full build's own steps never ran: one ninja command, no payload or guest stage
    assert len(run.commands) == 1
    assert not {"payloads", "wow64-guests", "storage-config"} & {s.name for s in run.stages}
    assert run.commands[0][0][-len(build._PRIME_TARGETS) :] == list(build._PRIME_TARGETS)
    assert run.commands[0][1]["stage"] == "ninja-prime"
    assert run.stages[-1].name == "headers-only"
    assert run.stages[-1].status == "warn"
