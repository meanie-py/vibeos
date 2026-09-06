#!/usr/bin/env python3
"""rs_walk.py — the one .rs traversal, pruned rather than filtered.

Six source scanners each wrote `for p in root.rglob("*.rs"): if "/target/" in str(p): continue`.
That reads as a filter, but `rglob` has already DESCENDED into `target/` by the time the test
runs, so the exclusion is cosmetic: the walk still stats every one of the ~5,700 directories
under `sdk/lib/rust/target`, and a scanner that matches nothing in that tree still dies in it.
Under the source-early partition the scanners run alongside the three builds instead of after
them, so cargo is deleting directories out from under the walk — the scanner crashes with
FileNotFoundError deep inside pathlib and the gate reports it as "NEW lint findings (exit 1)"
with no findings in it, bouncing a lane for a defect it did not cause.

So: prune during the walk, and survive a directory that vanishes mid-walk anyway (`onerror`).
Pruning is also just much faster — target/ dwarfs the rest of the tree.

Pruning by directory NAME is exactly equivalent to the `"/target/" in str(p)` test it replaces:
that substring needs a separator on both sides, so it can only ever have matched a directory.
"""

from __future__ import annotations

import os
from collections.abc import Iterator
from pathlib import Path

# `target` is cargo's; `.responses` is rustconv's paid-artifact stream (in-repo, gitignored,
# and full of .rs that were never part of the tree).
DEFAULT_PRUNE = ("target", ".responses", ".git")


def iter_rs_files(
    root: str | Path,
    *,
    prune: tuple[str, ...] = DEFAULT_PRUNE,
    skip_generated: bool = False,
) -> Iterator[Path]:
    """Every `*.rs` under `root`, never descending into a pruned directory.

    `skip_generated` also drops `bindgen_*.rs` and `build.rs`, which several scanners exclude
    because they are machine-written and hold none of the hand-translation bugs being hunted.
    """
    for dirpath, dirnames, filenames in os.walk(root, onerror=lambda _e: None):
        dirnames[:] = [d for d in dirnames if d not in prune]
        here = Path(dirpath)
        for name in filenames:
            if not name.endswith(".rs"):
                continue
            if skip_generated and (name.startswith("bindgen_") or name == "build.rs"):
                continue
            yield here / name
