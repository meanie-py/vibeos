import json
import os
from pathlib import Path
from types import SimpleNamespace

import pytest

from . import hostlock, maintenance, prune, retention


def args(**kw):
    return SimpleNamespace(
        **dict(
            {
                "enable": False,
                "disable": False,
                "run": False,
                "retain_gib": 100,
                "min_free_gib": 150,
                "idle_hours": 24,
            },
            **kw,
        )
    )


@pytest.fixture
def fleet(tmp_path, monkeypatch):
    repo = tmp_path / "repo"
    code = repo / "tests/qemu/cli"
    code.mkdir(parents=True)
    (code / "maintenance.py").write_text("fixture interlock")
    (code / "__main__.py").write_text("with maintenance.activity(REPO):\n    pass\n")
    monkeypatch.setattr(hostlock, "HOSTLOCK_DIR", tmp_path / "locks")
    monkeypatch.setattr(Path, "home", lambda: tmp_path / "home")
    monkeypatch.setattr(prune, "git_worktrees", lambda _: [repo])
    monkeypatch.setattr(prune, "active_build_worktrees", lambda _: set())
    monkeypatch.setattr(prune, "live_qemu_worktrees", lambda _: set())
    real_run = retention.subprocess.run
    monkeypatch.setattr(
        retention.subprocess,
        "run",
        lambda cmd, **kw: None if cmd[0] == "systemctl" else real_run(cmd, **kw),
    )
    return repo


def test_activation_refuses_busy_and_old_clients_without_installing(fleet, monkeypatch):
    with maintenance.activity(fleet):
        assert retention.command(args(enable=True), fleet) == 2
    assert not retention.policy_path().exists()
    old = fleet.parent / "old"
    old.mkdir()
    monkeypatch.setattr(prune, "git_worktrees", lambda _: [fleet, old])
    assert retention.command(args(enable=True), fleet) == 2
    assert not (hostlock.HOSTLOCK_DIR / "maintenance-owner").exists()
    assert not (Path.home() / ".config/systemd/user").exists()


def test_enable_disable_reenable_and_idle_run(fleet):
    generated = fleet / "build-amd64"
    generated.mkdir()
    obj = generated / "object.o"
    obj.write_bytes(b"disposable")
    for p in (obj, generated):
        os.utime(p, (100, 100))
    source = fleet / "paid.rs"
    source.write_text("paid Rust")
    assert retention.command(args(enable=True, retain_gib=0), fleet) == 0
    assert retention.read_policy(fleet)["min_free_gib"] == 150
    assert retention.command(args(disable=True), fleet) == 0
    assert retention.command(args(run=True), fleet) == 2
    assert generated.exists()
    assert retention.command(args(enable=True, retain_gib=0), fleet) == 0
    assert retention.command(args(run=True), fleet) == 0
    assert not generated.exists()
    assert source.read_text() == "paid Rust"


def test_failed_scheduler_install_restores_controls(fleet, monkeypatch):
    def fail(*a, **k):
        raise OSError("systemd unavailable")

    monkeypatch.setattr(retention.subprocess, "run", fail)
    assert retention.command(args(enable=True), fleet) == 2
    assert not retention.policy_path().exists()
    assert not (hostlock.HOSTLOCK_DIR / "maintenance-owner").exists()
    assert not list((Path.home() / ".config/systemd/user").glob("vibeos-retention.*"))


def test_headroom_selects_extra_outputs_but_recent_outputs_survive(fleet, monkeypatch):
    old = fleet.parent / "old"
    recent = fleet.parent / "recent"
    for lane in (old, recent):
        d = lane / "build-amd64"
        d.mkdir(parents=True)
        (d / "obj").write_text("disposable")
    monkeypatch.setattr(prune, "git_worktrees", lambda _: [old, recent])
    monkeypatch.setattr(prune, "_newest_mtime", lambda p: 0 if p == old else float("inf"))
    monkeypatch.setattr(prune.shutil, "disk_usage", lambda _: SimpleNamespace(free=149 * 1024**3))
    monkeypatch.setattr(prune, "_dir_size", lambda _: 2 * 1024**3)
    hostlock.HOSTLOCK_DIR.mkdir()
    (hostlock.HOSTLOCK_DIR / "maintenance-owner").write_text(str(fleet))
    request = SimpleNamespace(
        apply=True, self=False, retain_gib=100, min_free_gib=150, idle_hours=24
    )
    assert prune.prune(request, fleet) == 0
    assert not (old / "build-amd64").exists()
    assert (recent / "build-amd64/obj").is_file()


def test_policy_rejects_bad_budgets(fleet):
    hostlock.HOSTLOCK_DIR.mkdir()
    policy = {
        "version": 1,
        "owner": str(fleet),
        "retain_gib": 100,
        "min_free_gib": 150,
        "idle_hours": 24,
    }
    for key, value in (("retain_gib", True), ("min_free_gib", -1), ("idle_hours", 0)):
        retention.policy_path().write_text(json.dumps(dict(policy, **{key: value})))
        with pytest.raises(ValueError):
            retention.read_policy(fleet)


def test_integration_requires_explicit_opt_in(fleet):
    integration = fleet.parent / prune.queue.INTEGRATION_DIRNAME

    def reason(include=False, active=False):
        return prune.classify(
            integration,
            current=fleet,
            live_qemu=set(),
            active_build={integration} if active else set(),
            allow_self=False,
            min_age_hours=24,
            now=1000000,
            newest_mtime=0,
            include_integration=include,
        )

    assert reason()
    assert reason(include=True) is None
    assert reason(include=True, active=True) == "build in flight"
