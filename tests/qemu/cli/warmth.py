"""Build receipts: has this lane's code been built, so the gate's compiler caches are warm?

WHY (2026-09-26): upstream-sync submitted a merge that forced a full kernel recompile. The
merge queue's gate builds COLD (`--clean`) and relies on the shared, content-addressed
ccache/sccache to make that cheap -- normally ~8 minutes. With nothing in the cache for the
new code it took hours per arch, hit BUILD_TIMEOUT, and held nine lanes behind it for ~80
minutes. The lane's fix was to build in its own tree first, which fills the same shared
cache, and resubmit. That is a habit every lane needs and none can remember, so `land`
checks for it: every successful `vibeos-cli build` leaves a receipt, and `land` wants one
per arch for the tree it submits.

WHAT A RECEIPT IS KEYED ON. The content of the WORKING TREE at build time -- the tree a full
`git add -A` would commit -- not HEAD. Agents build with uncommitted edits and commit
afterwards; a HEAD key would almost never match what they then submit. It is computed in a
throwaway index (seeded from the real one, so git's stat cache still applies), and the
real index is never touched.

CLOSE ENOUGH COUNTS. A receipt for a tree whose difference from the submitted one is
documentation only is accepted: Markdown does not cool a compiler cache. The predicate is
`tools/docs_only.py`, the same one the conductor uses to skip gating docs-only batches.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

RECEIPT_DIRNAME = "vibeos-build-receipts"
MAX_AGE_S = 14 * 24 * 3600


def _git(repo: Path, *args: str, env: dict | None = None) -> str:
    cp = subprocess.run(
        ["git", *args], cwd=str(repo), capture_output=True, text=True, env=env, timeout=600
    )
    if cp.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {cp.stderr.strip()}")
    return cp.stdout.strip()


def receipts_dir(repo: Path) -> Path:
    common = Path(_git(repo, "rev-parse", "--git-common-dir"))
    common = common if common.is_absolute() else (repo / common).resolve()
    return common / RECEIPT_DIRNAME


def worktree_tree(repo: Path) -> str:
    """The tree `git add -A && git commit` would record right now."""
    index = Path(_git(repo, "rev-parse", "--git-path", "index"))
    index = index if index.is_absolute() else (repo / index)
    with tempfile.TemporaryDirectory(prefix="vibeos-warmth-") as tmp:
        scratch = Path(tmp) / "index"
        if index.is_file():
            shutil.copy2(index, scratch)
        env = dict(os.environ, GIT_INDEX_FILE=str(scratch))
        _git(repo, "add", "-A", env=env)
        return _git(repo, "write-tree", env=env)


def record(repo: Path, arch: str) -> str:
    """Leave a receipt for a SUCCESSFUL build of `arch`. Returns the tree it names."""
    tree = worktree_tree(repo)
    d = receipts_dir(repo)
    d.mkdir(parents=True, exist_ok=True)
    rec = {"tree": tree, "arch": arch, "worktree": str(repo.resolve()), "at": int(time.time())}
    tmp = d / f".{tree}.{arch}.tmp"
    tmp.write_text(json.dumps(rec))
    tmp.replace(d / f"{tree}.{arch}.json")
    _prune(d)
    return tree


def _prune(d: Path) -> None:
    now = time.time()
    for p in d.glob("*.json"):
        try:
            if now - p.stat().st_mtime > MAX_AGE_S:
                p.unlink()
        except OSError:
            pass


def _receipt_trees(repo: Path, arch: str) -> list[str]:
    """Trees with a receipt for `arch`, newest first."""
    try:
        files = sorted(
            receipts_dir(repo).glob(f"*.{arch}.json"), key=lambda p: p.stat().st_mtime, reverse=True
        )
    except OSError:
        return []
    return [p.name.split(".", 1)[0] for p in files]


def _changed(repo: Path, a: str, b: str) -> set[str] | None:
    try:
        return {p for p in _git(repo, "diff", "--name-only", a, b).splitlines() if p.strip()}
    except RuntimeError:
        return None


def missing(
    repo: Path,
    tree: str,
    arches: tuple[str, ...],
    is_inert,
    *,
    master: str | None = None,
    scan: int = 20,
) -> list[str]:
    """The arches with no receipt close enough to `tree` to have warmed the cache for it.

    A receipt covers `tree` when every path that differs between them is either inert
    (docs) or identical to `master` in `tree`. The second rule is the merge case: a lane
    that built, then merged origin/master, now differs from its receipt by everything that
    landed meanwhile -- but that code was built by master's own gate, so it is already in
    the cache. Only the lane's OWN changes can be cold."""
    own = _changed(repo, master, tree) if master else None
    out = []
    for arch in arches:
        trees = _receipt_trees(repo, arch)
        if tree in trees:
            continue
        warm = False
        for t in trees[:scan]:
            diff = _changed(repo, t, tree)
            if diff is None:
                continue
            if all(is_inert(p) or (own is not None and p not in own) for p in diff):
                warm = True
                break
        if not warm:
            out.append(arch)
    return out
