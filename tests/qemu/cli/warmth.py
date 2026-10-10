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

So is a difference in Python the build never runs. A receipt answers ONE question -- would
the gate's compiler caches be cold? -- and a `.py` under tools/ or tests/ can only move them
if the build executes it (a generator) or something it executes imports it. Without this
rule the common sequence "build, see a lint finding in your new tool, fix it, commit" was
refused as "not BUILT" and cost three no-op rebuilds (smp-bringup, 2026-10-09: four .py
files touched after a green build, nothing compiled changed). See build_reachable_python.
"""

from __future__ import annotations

import json
import os
import re
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


# How the build names a Python file: a path or a bare basename (often behind a CMake
# variable, `${REACTOS_SOURCE_DIR}/tools/gen_cflags.py`), or a module (`-m rustconv.c_oracle`).
_PY_REF = re.compile(r"([A-Za-z0-9_-]+)\.py\b|-m\s+([A-Za-z_][\w.]*)")
# The same shape in POSIX ERE, for `git grep` (no \\b, \\s or \\w there).
_GREP_PY_REF = r"[A-Za-z0-9_-]+\.py|-m[[:space:]]+[A-Za-z_][A-Za-z0-9_.]*"
_IMPORT = re.compile(r"^\s*(?:from\s+\.*([\w.]*)\s+import\s+([\w, ()]+)|import\s+([\w., ]+))", re.M)
NEUTRAL_ROOTS = ("tools/", "tests/")
# Tracked files that can make the build run Python: CMake, cargo build scripts and the
# reactos-link crate they share, sdk/tools (build-time Python itself), shell drivers.
# Comments in these count too -- harmless, since a false reference only reports cold.
BUILD_GRAPH = (
    "*CMakeLists.txt",
    "*.cmake",
    "*build.rs",
    "sdk/lib/rust/reactos-link",
    "sdk/tools",
    "*.sh",
    "*.cmd",
)


def _names(dotted: str) -> set[str]:
    return {part for part in re.split(r"[.,\s()]+", dotted) if part and part != "as"}


def build_reachable_python(repo: Path) -> set[str] | None:
    """The tracked tools/ and tests/ `.py` files the build might execute, or None if that
    cannot be worked out (then nothing is treated as neutral).

    Roots are the Python every BUILD_GRAPH file names, plus this worktree's own generated
    build-*/build.ninja. Then everything a reachable file imports,
    to a fixpoint. Matching is by NAME, so a same-named file anywhere makes every file of
    that name reachable: over-approximation only ever reports cold, the safe direction."""
    try:
        tracked = _git(repo, "ls-files", "--", "tools/*.py", "tests/*.py").splitlines()
        cp = subprocess.run(
            [
                "git",
                "grep",
                "-h",
                "-o",
                "-I",
                "-E",
                _GREP_PY_REF,
                "--",
                *BUILD_GRAPH,
            ],
            cwd=str(repo),
            capture_output=True,
            text=True,
            timeout=600,
        )
    except (RuntimeError, OSError, subprocess.SubprocessError):
        return None
    if cp.returncode not in (0, 1):  # 1 is "no match": a real answer
        return None
    refs = cp.stdout
    text = [refs]
    for ninja in sorted(repo.glob("build-*/build.ninja")):  # this worktree's own, never a sibling's
        try:
            text.append(ninja.read_text("utf8", "replace"))
        except OSError:
            return None
    by_name: dict[str, list[str]] = {}
    for p in tracked:
        parts = p[: -len(".py")].split("/")
        for key in {parts[-1]} | (
            {parts[-2]} if parts[-1] == "__init__" and len(parts) > 1 else set()
        ):
            by_name.setdefault(key, []).append(p)
    wanted: set[str] = set()
    for m in _PY_REF.finditer("\n".join(text)):
        wanted |= {m.group(1)} if m.group(1) else _names(m.group(2))
    reachable: set[str] = set()
    pending = [p for n in wanted for p in by_name.get(n, [])]
    while pending:
        p = pending.pop()
        if p in reachable:
            continue
        reachable.add(p)
        try:
            src = (repo / p).read_text("utf8", "replace")
        except OSError:
            continue
        for m in _IMPORT.finditer(src):
            for name in _names(" ".join(g for g in m.groups() if g)):
                pending.extend(q for q in by_name.get(name, []) if q not in reachable)
    return reachable


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

    A receipt covers `tree` when every path that differs between them is inert (docs), or
    Python the build cannot reach (build_reachable_python), or identical to `master` in
    `tree`. The last rule is the merge case: a lane that built, then merged origin/master,
    now differs from its receipt by everything that landed meanwhile -- but that code was
    built by master's own gate, so it is already in the cache. Only the lane's OWN changes
    can be cold."""
    own = _changed(repo, master, tree) if master else None
    reach: set[str] | bool | None = False  # computed on first need: it costs a repo grep

    def neutral(p: str) -> bool:
        nonlocal reach
        if not (p.endswith(".py") and p.startswith(NEUTRAL_ROOTS)):
            return False
        if reach is False:
            reach = build_reachable_python(repo)
        return isinstance(reach, set) and p not in reach

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
            if all(is_inert(p) or (own is not None and p not in own) or neutral(p) for p in diff):
                warm = True
                break
        if not warm:
            out.append(arch)
    return out
