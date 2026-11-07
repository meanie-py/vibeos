from types import SimpleNamespace

import pytest

from . import hostlock, maintenance, prune


@pytest.fixture
def pinned(tmp_path, monkeypatch):
    locks = tmp_path / "host"
    locks.mkdir()
    monkeypatch.setattr(hostlock, "HOSTLOCK_DIR", locks)
    (locks / "maintenance-owner").write_text(str(tmp_path))
    return tmp_path, locks


def test_activity_readers_coexist_cleanup_excludes(pinned):
    repo, _ = pinned
    with (
        maintenance.activity(repo),
        maintenance.activity(repo),
        pytest.raises(maintenance.MaintenanceError, match="in progress"),
        maintenance.activity(repo, exclusive=True),
    ):
        pytest.fail("cleanup raced active reader")
    with (
        maintenance.activity(repo, exclusive=True),
        pytest.raises(maintenance.MaintenanceError, match="in progress"),
        maintenance.activity(repo),
    ):
        pytest.fail("reader raced cleanup")
    with maintenance.activity(repo):
        pass  # exclusive lock released


def test_cleanup_requires_pinned_copy_and_obeys_kill_switch(pinned):
    repo, locks = pinned
    with (
        pytest.raises(maintenance.MaintenanceError, match="pinned owner"),
        maintenance.activity(repo / "sibling", exclusive=True),
    ):
        pass
    (locks / "maintenance-disabled").touch()
    with pytest.raises(maintenance.MaintenanceError, match="kill switch"):
        maintenance.check_owner(repo)
    (locks / "maintenance-disabled").unlink()
    (locks / "maintenance-owner").unlink()
    with pytest.raises(maintenance.MaintenanceError, match="prune disabled"):
        maintenance.check_owner(repo)


def test_prune_apply_only_generated_dirs_and_pin_required(pinned, monkeypatch):
    repo, locks = pinned
    lane = repo / "lane"
    generated = lane / "build-amd64"
    generated.mkdir(parents=True)
    (generated / "fixture.o").write_bytes(b"regenerable")
    paid = lane / "paid.rs"
    paid.write_text("paid translation")
    monkeypatch.setattr(prune, "git_worktrees", lambda _: [lane])
    monkeypatch.setattr(prune, "active_build_worktrees", lambda _: set())
    monkeypatch.setattr(prune, "live_qemu_worktrees", lambda _: set())
    args = SimpleNamespace(apply=True, self=False, idle_hours=0, retain_gib=None)
    (locks / "maintenance-owner").unlink()
    assert prune.prune(args, repo) == 2
    assert generated.exists()
    (locks / "maintenance-owner").write_text(str(repo))
    assert prune.prune(args, repo) == 0
    assert not generated.exists()
    assert paid.read_text() == "paid translation"


def test_prune_refuses_symlinked_generated_parent(pinned, monkeypatch):
    repo, _ = pinned
    lane = repo / "lane"
    lane.mkdir()
    foreign = repo / "foreign"
    foreign.mkdir()
    (foreign / "fixture.o").write_bytes(b"must survive")
    (lane / "build-amd64").symlink_to(foreign, target_is_directory=True)
    monkeypatch.setattr(prune, "git_worktrees", lambda _: [lane])
    monkeypatch.setattr(prune, "active_build_worktrees", lambda _: set())
    monkeypatch.setattr(prune, "live_qemu_worktrees", lambda _: set())
    args = SimpleNamespace(apply=True, self=False, idle_hours=0, retain_gib=None)
    assert prune.prune(args, repo) == 2
    assert (foreign / "fixture.o").read_bytes() == b"must survive"


def test_prune_only_limits_the_fleet_and_refuses_unknown_names(tmp_path, monkeypatch, capsys):
    lanes = [tmp_path / "vibeos-a", tmp_path / "vibeos-b"]
    for lane in lanes:
        (lane / "build-i386").mkdir(parents=True)
        (lane / "build-i386" / "x.o").write_bytes(b"x" * 4096)
    monkeypatch.setattr(prune, "git_worktrees", lambda _: lanes)
    monkeypatch.setattr(prune, "live_qemu_worktrees", lambda _: set())
    monkeypatch.setattr(prune, "active_build_worktrees", lambda _: set())
    args = SimpleNamespace(apply=False, only=["vibeos-b"], idle_hours=0)
    assert prune.prune(args, tmp_path) == 0
    out = capsys.readouterr().out
    assert "vibeos-b" in out and "vibeos-a" not in out
    args.only = ["vibeos-nope"]
    assert prune.prune(args, tmp_path) == 2
    assert "--only names no worktree: vibeos-nope" in capsys.readouterr().out
