"""One walk over the Rust tree, shared by every scanner that needs `*.rs`.

Why this exists: forty tools had each hand-rolled `root.rglob("*.rs")` followed by
`if "/target/" in str(path): continue`. That filters AFTER the walk has already
descended, so pathlib still enumerates `sdk/lib/rust/target` — and a concurrent
cargo build deletes directories out from under it. `Path.rglob` propagates the
resulting `FileNotFoundError` (verified on this interpreter, 3.10.12), so the
scanner dies with a traceback on stderr and nothing on stdout.

That was harmless while source scanners ran after the builds. The `source-early`
partition now runs them AT t=0, alongside the builds, which made it reachable —
and the gate reported the crash as "NEW lint findings", sending the lane hunting
a source defect that does not exist. Pruning during the walk removes the race
rather than catching it, and `onerror` keeps any other directory vanishing
mid-walk from being fatal. It is also much faster: `target/` dwarfs the sources.

Order matches `sorted(rglob(...))` exactly, so lint baselines do not move.
"""

from __future__ import annotations

import os
from collections.abc import Callable, Iterable, Iterator
from pathlib import Path

#: Directory NAMES pruned wherever they appear. `target` is cargo's output (the
#: racing one); `.responses` is rustconv's paid-artifact spool, which holds `.rs`
#: that is not in the tree yet.
PRUNE_DIRS = frozenset({"target", ".responses", ".git"})


def walk_rs(
    root: Path | str,
    *,
    skip: Callable[[Path], bool] | None = None,
    prune: Iterable[str] = PRUNE_DIRS,
) -> list[Path]:
    """Every `*.rs` under `root`, sorted, with `prune` directories never entered.

    `skip` is an extra per-file predicate (bindgen islands, `build.rs`, …); it
    runs on paths that survived pruning, so it can never re-introduce the walk.
    A `root` that is itself a `.rs` file yields just that file.
    """
    root = Path(root)
    if root.is_file():
        return [root] if root.suffix == ".rs" and not (skip and skip(root)) else []
    pruned = frozenset(prune)
    out: list[Path] = []
    for dirpath, dirnames, filenames in os.walk(root, onerror=lambda _e: None):
        dirnames[:] = [d for d in dirnames if d not in pruned]
        base = Path(dirpath)
        for fn in filenames:
            if not fn.endswith(".rs"):
                continue
            p = base / fn
            if skip and skip(p):
                continue
            out.append(p)
    out.sort()
    return out


def iter_rs(roots: Iterable[Path | str], **kw) -> Iterator[Path]:
    """`walk_rs` over several roots, in the order given."""
    for r in roots:
        yield from walk_rs(r, **kw)
