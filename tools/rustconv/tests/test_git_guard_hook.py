"""Regression tests for the mainline-rewind guard (tools/git-hooks/reference-transaction).

The guard blocks non-fast-forward updates / deletes of `master` (the ~81-commit
rewind footgun) while allowing normal FF landings AND ref-storage maintenance.
The maintenance carve-out is load-bearing: without it, `git pack-refs` (run by
auto-`gc` on every commit/fetch) is vetoed because migrating a loose ref into
packed-refs looks like a delete (new=0000), making every git op spew errors.

These pin BOTH directions so a future edit can't silently (a) re-block pack-refs
or (b) stop blocking a real rewind — the guard moved master exactly once, during
its own development, when the carve-out over-reached into the caller's shell.
"""

import os
import shutil
import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[3]
HOOK_SRC = REPO / "tools" / "git-hooks" / "reference-transaction"

pytestmark = pytest.mark.skipif(not shutil.which("git"), reason="git not on PATH")


def _git(repo, *args, **kw):
    return subprocess.run(["git", *args], cwd=repo, capture_output=True, text=True, **kw)


@pytest.fixture
def repo(tmp_path):
    r = tmp_path / "r"
    r.mkdir()
    _git(r, "init", "-q")
    _git(r, "config", "user.email", "t@t")
    _git(r, "config", "user.name", "t")
    _git(r, "commit", "-q", "--allow-empty", "-m", "c1")
    _git(r, "commit", "-q", "--allow-empty", "-m", "c2")
    hook_dir = r / ".git" / "hooks"
    hook_dir.mkdir(parents=True, exist_ok=True)
    dst = hook_dir / "reference-transaction"
    dst.write_bytes(HOOK_SRC.read_bytes())
    dst.chmod(0o755)
    return r


def _master(repo):
    return _git(repo, "rev-parse", "master").stdout.strip()


def test_fast_forward_allowed(repo):
    before = _master(repo)
    r = _git(repo, "commit", "-q", "--allow-empty", "-m", "c3")
    assert r.returncode == 0
    assert _master(repo) != before  # FF landing went through


def test_non_fast_forward_blocked_and_master_intact(repo):
    cur = _master(repo)
    par = _git(repo, "rev-parse", "master~1").stdout.strip()
    _git(repo, "checkout", "-q", "--detach")  # dodge git's own checked-out guard
    # reset --hard's internal path: a non-FF update-ref of master
    r = _git(repo, "update-ref", "refs/heads/master", par, cur)
    assert r.returncode != 0
    assert "git-guard: BLOCKED" in r.stderr
    assert _master(repo) == cur  # never moved


def test_delete_blocked(repo):
    cur = _master(repo)
    _git(repo, "checkout", "-q", "--detach")
    r = _git(repo, "update-ref", "-d", "refs/heads/master")
    assert r.returncode != 0
    assert "git-guard" in r.stderr
    assert _master(repo) == cur  # still there


def test_pack_refs_allowed(repo):
    """The regression: ref-packing must NOT be vetoed as a delete."""
    cur = _master(repo)
    r = _git(repo, "pack-refs", "--all")
    assert r.returncode == 0, r.stderr
    assert "git-guard" not in r.stderr
    assert _master(repo) == cur  # value preserved in packed-refs


def test_gc_allowed(repo):
    """auto-gc delegates ref-packing to a child pack-refs — must not be vetoed."""
    r = _git(repo, "gc", "--quiet")
    assert r.returncode == 0, r.stderr
    assert "git-guard" not in r.stderr


def test_escape_hatch_allows_rewind(repo):
    cur = _master(repo)
    par = _git(repo, "rev-parse", "master~1").stdout.strip()
    _git(repo, "checkout", "-q", "--detach")
    r = subprocess.run(
        ["git", "update-ref", "refs/heads/master", par, cur],
        cwd=repo,
        capture_output=True,
        text=True,
        env={**os.environ, "VIBEOS_ALLOW_MAINLINE_REWIND": "1"},
    )
    assert r.returncode == 0
    assert _master(repo) == par  # intentional rewind honored
