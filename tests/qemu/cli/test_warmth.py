"""Warm receipts (warmth.py) and the `land` preflight that reads them.

The incident: upstream-sync (2026-09-26) submitted a merge no build had seen, the queue's
cold gate compiled it from scratch for hours, hit BUILD_TIMEOUT, and held nine lanes for
~80 minutes. These pin the mechanism that makes "build before you land" a check.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
from pathlib import Path

import pytest

from . import land as land_mod
from . import warmth
from .artifacts import Run

REPO = Path(__file__).resolve().parents[3]
ARCHES = ("i386", "amd64", "arm64")


def _g(repo: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=repo, check=True, capture_output=True, text=True
    ).stdout.strip()


def _is_inert(path: str) -> bool:
    return path.endswith(".md")


@pytest.fixture
def repo(tmp_path: Path) -> Path:
    r = tmp_path / "lane"
    r.mkdir()
    _g(r, "init", "-q", "-b", "master")
    _g(r, "config", "user.email", "t@t")
    _g(r, "config", "user.name", "t")
    (r / "code.c").write_text("int a;\n")
    _g(r, "add", "-A")
    _g(r, "commit", "-qm", "base")
    return r


def _build_all(r: Path) -> None:
    for arch in ARCHES:
        warmth.record(r, arch)


def _commit_all(r: Path) -> str:
    _g(r, "add", "-A")
    _g(r, "commit", "-qm", "work")
    return _g(r, "rev-parse", "HEAD^{tree}")


def test_a_receipt_names_the_tree_you_then_commit_even_when_built_dirty(repo):
    """Agents build with uncommitted edits and commit afterwards. A HEAD-keyed receipt
    would never match; the working-tree content does."""
    (repo / "code.c").write_text("int a;\nint b;\n")
    (repo / "new.c").write_text("int c;\n")  # untracked at build time
    _build_all(repo)
    tree = _commit_all(repo)
    assert warmth.missing(repo, tree, ARCHES, _is_inert) == []


def test_computing_the_tree_never_touches_the_real_index(repo):
    (repo / "code.c").write_text("int z;\n")
    before = _g(repo, "status", "--porcelain")
    warmth.worktree_tree(repo)
    assert _g(repo, "status", "--porcelain") == before == "M code.c"


def test_code_no_build_has_seen_is_reported_per_missing_arch(repo):
    _build_all(repo)
    warmth_tree_before = _g(repo, "rev-parse", "HEAD^{tree}")
    assert warmth.missing(repo, warmth_tree_before, ARCHES, _is_inert) == []
    (repo / "code.c").write_text("int changed;\n")
    tree = _commit_all(repo)
    assert warmth.missing(repo, tree, ARCHES, _is_inert) == list(ARCHES)
    warmth.record(repo, "amd64")
    assert warmth.missing(repo, tree, ARCHES, _is_inert) == ["i386", "arm64"]


def test_a_docs_only_difference_from_a_built_tree_is_still_warm(repo):
    """Markdown does not cool a compiler cache: a doc commit after the build must not
    force three rebuilds."""
    _build_all(repo)
    (repo / "NOTES.md").write_text("note\n")
    tree = _commit_all(repo)
    assert warmth.missing(repo, tree, ARCHES, _is_inert) == []


def test_merging_master_after_the_build_does_not_make_the_lane_cold(repo):
    """Code that arrived from master was built by master's own gate, so it is warm: only
    the lane's OWN changes need a receipt."""
    base = _g(repo, "rev-parse", "HEAD")
    (repo / "mine.c").write_text("int mine;\n")
    _build_all(repo)
    _commit_all(repo)
    lane = _g(repo, "rev-parse", "HEAD")
    _g(repo, "checkout", "-q", base)  # master moves on, with CODE
    (repo / "theirs.c").write_text("int theirs;\n")
    master_tree = _commit_all(repo)
    _g(repo, "checkout", "-q", lane)
    _g(repo, "merge", "-q", "--no-edit", "HEAD@{1}")
    tree = _g(repo, "rev-parse", "HEAD^{tree}")
    assert warmth.missing(repo, tree, ARCHES, _is_inert) == list(ARCHES)  # naive: cold
    assert warmth.missing(repo, tree, ARCHES, _is_inert, master=master_tree) == []


def test_the_lanes_own_unbuilt_change_is_still_cold_after_a_merge(repo):
    base = _g(repo, "rev-parse", "HEAD")
    _build_all(repo)
    (repo / "mine.c").write_text("int never_built;\n")
    _commit_all(repo)
    lane = _g(repo, "rev-parse", "HEAD")
    _g(repo, "checkout", "-q", base)
    (repo / "theirs.c").write_text("int theirs;\n")
    master_tree = _commit_all(repo)
    _g(repo, "checkout", "-q", lane)
    _g(repo, "merge", "-q", "--no-edit", "HEAD@{1}")
    tree = _g(repo, "rev-parse", "HEAD^{tree}")
    assert warmth.missing(repo, tree, ARCHES, _is_inert, master=master_tree) == list(ARCHES)


# ── the land preflight ──────────────────────────────────────────────────────


@pytest.fixture
def lane(repo: Path) -> Path:
    """A lane with the real docs_only predicate and an origin/master to compare against."""
    (repo / "tools").mkdir()
    shutil.copy2(REPO / "tools" / "docs_only.py", repo / "tools" / "docs_only.py")
    _commit_all(repo)
    origin = repo.parent / "origin.git"
    subprocess.run(["git", "init", "-q", "--bare", str(origin)], check=True, capture_output=True)
    _g(repo, "remote", "add", "origin", str(origin))
    _g(repo, "push", "-q", "origin", "master")
    _g(repo, "fetch", "-q", "origin")
    return repo


def _preflight(r: Path, **flags) -> Run:
    run = Run("land", "test")
    land_mod._warm_preflight(r, run, argparse.Namespace(**flags))
    return run


def _stage(run: Run) -> str:
    return next(s.status for s in run.stages if s.name == "warm")


def test_land_refuses_code_no_build_has_seen(lane):
    (lane / "code.c").write_text("int cold;\n")
    _commit_all(lane)
    with pytest.raises(land_mod.LandAbort) as ei:
        _preflight(lane)
    msg = str(ei.value)
    assert "./vibeos-cli build --arch i386" in msg and "--cold" in msg


def test_land_passes_a_lane_that_built_what_it_submits(lane):
    (lane / "code.c").write_text("int warm;\n")
    _build_all(lane)
    _commit_all(lane)
    assert _stage(_preflight(lane)) == "ok"


def test_land_cold_overrides_loudly(lane):
    (lane / "code.c").write_text("int cold;\n")
    _commit_all(lane)
    assert _stage(_preflight(lane, cold=True)) == "warn"


def test_a_docs_only_lane_needs_no_build(lane):
    """The conductor does not gate a docs-only batch at all, so there is no cache to warm."""
    (lane / "README.md").write_text("docs\n")
    _commit_all(lane)
    assert _stage(_preflight(lane)) == "skip"


# ── Python the build never runs does not cool the cache ───────────────────────


def test_a_tools_script_the_build_never_runs_is_still_warm(repo):
    """smp-bringup, 2026-10-09: a lint finding in a NEW tool, fixed after a green build,
    was refused as "not BUILT" -- three no-op rebuilds for a file no compiler sees."""
    (repo / "tools").mkdir()
    (repo / "tools" / "mytool.py").write_text("print('v1')\n")
    _build_all(repo)
    (repo / "tools" / "mytool.py").write_text("print('v2')\n")
    tree = _commit_all(repo)
    assert warmth.missing(repo, tree, ARCHES, _is_inert) == []


def test_a_generator_the_build_runs_is_cold(repo):
    (repo / "tools").mkdir()
    (repo / "tools" / "gen_x.py").write_text("print('v1')\n")
    (repo / "CMakeLists.txt").write_text(
        "add_custom_command(COMMAND python ${SRC}/tools/gen_x.py)\n"
    )
    _build_all(repo)
    (repo / "tools" / "gen_x.py").write_text("print('v2')\n")
    tree = _commit_all(repo)
    assert warmth.missing(repo, tree, ARCHES, _is_inert) == list(ARCHES)


def test_a_helper_a_generator_imports_is_cold(repo):
    """Reachability follows imports: changing what a generator imports changes its output."""
    (repo / "tools").mkdir()
    (repo / "tools" / "gen_x.py").write_text("from tools.helper import f\n")
    (repo / "tools" / "helper.py").write_text("def f(): return 1\n")
    (repo / "build.rs").write_text('fn main() { run("-m tools.gen_x"); }\n')
    _build_all(repo)
    (repo / "tools" / "helper.py").write_text("def f(): return 2\n")
    tree = _commit_all(repo)
    assert warmth.missing(repo, tree, ARCHES, _is_inert) == list(ARCHES)


@pytest.mark.parametrize("referenced", [True, False])
def test_this_worktrees_build_ninja_counts_as_a_reference(repo, referenced):
    """The control (`referenced=False`) is what makes this meaningful: the same edit is
    warm when the generated build graph does not name the script."""
    (repo / ".gitignore").write_text("build-*/\n")
    (repo / "tools").mkdir()
    (repo / "tools" / "post.py").write_text("print('v1')\n")
    (repo / "build-amd64").mkdir()
    cmd = "python /x/tools/post.py" if referenced else "ninja -t restat"
    (repo / "build-amd64" / "build.ninja").write_text(f"  command = {cmd}\n")
    _commit_all(repo)
    _build_all(repo)
    (repo / "tools" / "post.py").write_text("print('v2')\n")
    tree = _commit_all(repo)
    assert warmth.missing(repo, tree, ARCHES, _is_inert) == (list(ARCHES) if referenced else [])
