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

from pathlib import Path

from tools import build_infra_compat as native

# Native: tools/vibeos-build/src/warmth.rs (receipt keying, recording, reachability, the
# close-enough rule). The native build records receipts itself; this is the facade for
# `land` and tests. The inert predicate stays the CALLER's (tools/docs_only.py, shared with
# the pre-push hook), so `missing` asks native which paths need judging, judges them here,
# and hands the verdicts back.

RECEIPT_DIRNAME = "vibeos-build-receipts"


def _call(op: str, **kwargs):
    try:
        return native.call("warmth_" + op, kwargs)
    except native.NativeError as exc:
        raise RuntimeError(str(exc)) from exc


def receipts_dir(repo: Path) -> Path:
    return Path(_call("receipts_dir", repo=repo))


def worktree_tree(repo: Path) -> str:
    """The tree `git add -A && git commit` would record right now."""
    return _call("worktree_tree", repo=repo)


def record(repo: Path, arch: str) -> str:
    """Leave a receipt for a SUCCESSFUL build of `arch`. Returns the tree it names."""
    return _call("record", repo=repo, arch=arch)


def build_reachable_python(repo: Path) -> set[str] | None:
    """The tracked tools/ and tests/ `.py` files the build might execute, or None."""
    found = _call("build_reachable_python", repo=repo)
    return None if found is None else set(found)


def missing(
    repo: Path,
    tree: str,
    arches: tuple[str, ...],
    is_inert,
    *,
    master: str | None = None,
    scan: int = 20,
) -> list[str]:
    """The arches with no receipt close enough to `tree` to have warmed the cache for it."""
    common = {"repo": repo, "tree": tree, "arches": list(arches), "scan": scan}
    paths = _call("diff_paths", **common)
    inert = [p for p in paths if is_inert(p)]
    return _call("missing", **common, inert=inert, master=master)
