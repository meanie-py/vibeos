import os
from pathlib import Path
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


def test_retention_budget_never_overrides_protection():
    def candidate(name, size, eligible):
        path = Path("/fixture") / name
        return prune.Candidate(path, [(path / "build-amd64", size * 1024**3)], eligible, None)

    candidates = [
        candidate("integration", 40, False),
        candidate("old", 30, True),
        candidate("warm", 20, True),
    ]
    result = prune.select_budget(candidates, 60)
    assert [c.worktree.name for c in result if c.eligible] == ["old"]
    assert not candidates[2].skip_reason  # planner does not mutate its inputs
    result = prune.select_budget(candidates, 0)
    assert [c.worktree.name for c in result if c.eligible] == ["old", "warm"]
    for budget in (-1, float("inf"), float("nan")):
        with pytest.raises(ValueError):
            prune.select_budget(candidates, budget)


def test_retention_age_sees_nested_object_write(tmp_path):
    root = tmp_path / "build-amd64"
    obj = root / "nested" / "file.o"
    obj.parent.mkdir(parents=True)
    obj.write_bytes(b"object")
    os.utime(obj, (1000, 1000))
    os.utime(obj.parent, (10, 10))
    os.utime(root, (10, 10))
    assert prune._newest_mtime(tmp_path) == 1000


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


def test_orphan_canonical_compiler_protects_fleet(tmp_path, monkeypatch):
    proc = tmp_path / "1001"
    proc.mkdir()
    (proc / "comm").write_text("rustc\n")
    (proc / "cwd").symlink_to("/tmp/vibeos/sdk/lib/rust")
    original = Path.iterdir
    monkeypatch.setattr(
        Path, "iterdir", lambda path: iter([proc]) if path == Path("/proc") else original(path)
    )
    monkeypatch.setattr(prune.canon, "canon_path", lambda: "/tmp/vibeos")
    lanes = [tmp_path / "one", tmp_path / "two"]
    assert prune.active_build_worktrees(lanes) == set(lanes)


def test_canonical_compiler_is_attributed_by_its_bind_source(tmp_path, monkeypatch):
    lanes = [tmp_path / "one", tmp_path / "two"]
    for lane in lanes:
        lane.mkdir()
    proc = tmp_path / "1001"
    proc.mkdir()
    (proc / "comm").write_text("ninja\n")
    (proc / "cwd").symlink_to("/tmp/vibeos/build-i386")
    (proc / "mountinfo").write_text(
        "22 1 8:48 / / rw - ext4 /dev/sdd rw\n"
        f"1063 1015 8:48 {lanes[1]} /tmp/vibeos rw,relatime - ext4 /dev/sdd rw\n"
    )
    host = tmp_path / "host-mountinfo"
    host.write_text("22 1 8:48 / / rw - ext4 /dev/sdd rw\n")
    original = Path.iterdir
    monkeypatch.setattr(
        Path, "iterdir", lambda path: iter([proc]) if path == Path("/proc") else original(path)
    )
    monkeypatch.setattr(prune.canon, "canon_path", lambda: "/tmp/vibeos")
    real = prune.canonical_bind_source
    monkeypatch.setattr(
        prune, "canonical_bind_source", lambda mi, canon: real(mi, canon, host_mountinfo=host)
    )
    assert prune.active_build_worktrees(lanes) == {lanes[1].resolve()}


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


def test_zombie_compiler_protects_nothing(tmp_path, monkeypatch):
    proc = tmp_path / "1001"
    proc.mkdir()
    (proc / "comm").write_text("cc1\n")
    (proc / "status").write_text("Name:\tcc1\nState:\tZ (zombie)\n")
    original = Path.iterdir
    monkeypatch.setattr(
        Path, "iterdir", lambda path: iter([proc]) if path == Path("/proc") else original(path)
    )
    assert prune.active_build_worktrees([tmp_path / "one"]) == set()
