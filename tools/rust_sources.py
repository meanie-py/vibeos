"""One pruned, crash-proof walk over the tree's Rust sources.

THIS MODULE IS A SCAR, and the incident is worth stating because the failure it
prevents does not look like a walk bug from the outside.

Every scanner used to enumerate its inputs as `root.rglob("*.rs")` and then drop
the build outputs with `if "/target/" in str(p): continue`. That reads as correct
and, for years, was: `rglob` had already finished descending into
`sdk/lib/rust/target` before the filter ever saw a path, but nothing was deleting
those directories while a lint ran, so nobody paid for it.

`source-early` changed the timing, not the code. Audited source scanners now run
at t=0 CONCURRENTLY WITH THE BUILDS instead of after them, so cargo is unlinking
directories underneath a walk that is still descending into them. `rglob` raises
from inside the generator, the scanner dies with a traceback, and the gate reports
`lint:source-early: NEW lint findings (exit 1)` — a source verdict, with ZERO
findings in the artifact. A lane then hunts a source defect it did not introduce
and cannot find. One queue round trip was lost to exactly this before the class
was understood; the report names the wrong thing, so the second lane to hit it
pays the same cost as the first.

Two properties matter, and a caller that keeps the post-filter gets neither:

  * PRUNE, don't filter. `dirnames[:] = [...]` stops `os.walk` from ever entering
    the directory, so a concurrent `rm -rf` inside it cannot be observed. Filtering
    after the fact is too late by construction — the descent already happened.
  * SURVIVE the race anyway. `onerror` swallows a directory that vanishes between
    the readdir that named it and the one that reads it. Pruning removes the big
    predictable racer (`target/` dwarfs the rest of the tree); `onerror` covers
    everything else, including build dirs a future partition move makes reachable.

Pruning by directory NAME is exactly equivalent to the `"/target/" in path` filter
it replaces, not merely close to it: `sdk/lib/rust/target` is the only directory of
that name in the tree, and zero git-tracked files live under any `/target/` path.
Each migrated scanner's baseline was diffed byte-identical across the change.

Do not re-introduce a bare `rglob` for Rust sources; `scan_unpruned_walk` gates it.
"""

from __future__ import annotations

import os
from collections.abc import Iterable, Iterator
from pathlib import Path

# Directory NAMES never descended into. These hold build products, VCS metadata and
# vendored trees — never a Rust source the scanners are meant to judge — and they are
# also where a concurrent build does its deleting.
SKIP_DIR_NAMES = frozenset({"target", ".git", "node_modules", ".venv", "__pycache__"})

# Directory name PREFIXES, for the per-arch build trees (`build-i386`, ...).
SKIP_DIR_PREFIXES = ("build-",)


def _keep_dir(name: str) -> bool:
    return name not in SKIP_DIR_NAMES and not name.startswith(SKIP_DIR_PREFIXES)


def walk_files(
    roots: Iterable[Path | str],
    suffix: str = ".rs",
    *,
    extra_skip_dirs: Iterable[str] = (),
) -> Iterator[Path]:
    """Yield every `*<suffix>` file under `roots`, build trees pruned during descent.

    A root that is itself a file yields that file, so callers can accept a mix of
    files and directories on the command line the way they always have. Order is
    deterministic (sorted at every level): a scanner's output feeds a checked-in
    baseline, so a walk whose order drifts with the filesystem would show up as
    phantom baseline churn.
    """
    skip = set(extra_skip_dirs)
    for root in roots:
        root = Path(root)
        if root.is_file():
            if root.suffix == suffix:
                yield root
            continue
        for dirpath, dirnames, filenames in os.walk(root, onerror=lambda _e: None):
            dirnames[:] = sorted(d for d in dirnames if _keep_dir(d) and d not in skip)
            here = Path(dirpath)
            for fn in sorted(filenames):
                if fn.endswith(suffix):
                    yield here / fn


def rust_sources(
    roots: Iterable[Path | str],
    *,
    skip_bindgen: bool = False,
    skip_build_rs: bool = False,
) -> Iterator[Path]:
    """`walk_files` with the two file-level exclusions the scanners share.

    Both are generated or build-time code rather than translated output: `bindgen_*.rs`
    is machine-emitted FFI and `build.rs` runs on the host, so a scanner judging the
    shipped translation would only ever report noise from them. They stay opt-in
    because not every caller excluded them before, and this migration changes no
    scanner's verdict.
    """
    for f in walk_files(roots):
        if skip_bindgen and f.name.startswith("bindgen_"):
            continue
        if skip_build_rs and f.name == "build.rs":
            continue
        yield f
