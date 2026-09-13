#!/usr/bin/env python3
"""Ratchet on `rglob`/`glob("**/…")` walks that descend into build output.

THE CLASS. A walk written as `root.rglob("*.rs")` with `if "/target/" in str(p):
continue` is correct on a quiet tree and unsound on a busy one. `rglob` finishes
descending into `sdk/lib/rust/target` before the filter sees its first path, so
while a build is deleting directories underneath it the walk raises from inside
the generator and the scanner dies with a traceback.

WHY IT IS GATED RATHER THAN JUST FIXED. The defect is invisible until something
changes the TIMING, and nothing about the walk itself changes when it does. Moving
the audited source scanners into `source-early` — concurrent with the builds
instead of after them — made six of them reachable without touching a line of
their code. The failure then reports as `lint:source-early: NEW lint findings
(exit 1)` with ZERO findings in the artifact, so the lane that pays for it goes
hunting a source defect that does not exist. Any future partition move re-arms
whichever walks are still written this way, which is why the fix has to be a
standing gate and not a one-time sweep.

THE SECOND TREE. The rule started out as "Rust sources, because those are the walks
that cross a live cargo target dir". That was too narrow: a BUILD dir is the most-written
tree in the repo, and on 2026-09-13 three separate tools were found walking one with a
post-filter — `scan_ud2`'s driver sweep, `scan_self_recursive`, and
`scan_flex_array_panic.find_artifact`, which post-filtered on "-target/", the exact tell
named above. None was a `*.rs` walk, so none of them tripped the original rule. The
extension matches on the RECEIVER (`build`, `build_dir`, `self.build`) rather than on the
glob pattern, which is what keeps it narrow enough to gate: 4 sites, where "every
recursive walk in tools+tests" would be ~64 and mostly harmless single-level or
source-tree walks. The root being a build dir IS the mutable-tree criterion, made
greppable.

THE FIX at each site is `tools/source_walk.py` — `source_files(root)`, or
`source_walk(root)` when you need the dirnames. It excludes the mutable tree BEFORE
scandir rather than after, so a concurrent `rm -rf` inside it cannot be observed.
Note it deliberately does NOT swallow errors (`onerror=_raise`): a vanishing SOURCE
directory is an incomplete check, not an empty corpus. That is only a safe trade
because the churning tree is pruned first — the two halves go together.

BASELINE. The six scanners in EARLY_SOURCE_CHECKS are already migrated (they were
fixed on master by the lane that introduced source_walk; this scanner is what stops
a SEVENTH being written the old way tomorrow — the migration closed the six, not the
class). The rest are
latent, not harmless: they are in the baseline so the class cannot GROW while they
are burned down, and each is a real hazard the moment its caller runs beside a
build. A new row here means a new walk was written the old way — migrate it rather
than baselining it.

Not every hit is a lint target: one-shot `fix_*`/`migrate_*` refactor tools are
run by hand on a quiet tree. They are still listed, because "this tool is only
ever run by hand" is a property of today's caller, not of the code.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
BASELINE = REPO / "tools" / "lint-baseline" / "scan_unpruned_walk.txt"

# `.rglob("*.rs")` and the `.glob("**/*.rs")` spelling of the same descent. Anchored
# on the dot so a bare `glob.glob` module reference is not a hit, and restricted to
# Rust sources because those are the walks that cross a live cargo target dir.
WALK_RE = re.compile(r"\.(?:rglob\(\s*[\"']\*\.rs[\"']|glob\(\s*[\"']\*\*/\*\.rs[\"'])")

# The SECOND mutable tree, and the busier one. The rule above says "restricted to Rust
# sources because those are the walks that cross a live cargo target dir" -- too narrow.
# A BUILD dir is the most-written tree in the repo, and three separate tools walked one
# with a post-filter: scan_ud2's driver sweep and scan_self_recursive (both fixed
# 2026-09-13), and scan_flex_array_panic, whose `find_artifact` even post-filtered on
# "-target/" -- the exact tell this file's docstring names. None was a `*.rs` walk, so
# none of them tripped the rule above.
#
# Matching on the RECEIVER rather than the pattern is what makes this narrow enough to
# be worth gating: `build`/`build_dir`/`self.build` recursive walks are 4 sites in the
# tree, where "every recursive walk" would be ~64 and mostly harmless. The root being a
# build dir IS the mutable-tree criterion, made concrete and greppable.
BUILD_WALK_RE = re.compile(r"\b\w*build\w*\s*\.\s*(?:rglob\(|glob\(\s*[\"']\*\*/)")

# Files that necessarily CONTAIN the bad spelling as data rather than executing it:
# the module implementing the safe walk names the unsafe one in its docstring, this
# scanner states the pattern it matches, and the control harness stores a known-BAD
# fixture as a string literal for every gate lint in the tree. Exempting whole files is
# right here — in all three the pattern is subject matter, not a walk that ever runs.
EXEMPT = {"rust_sources.py", "scan_unpruned_walk.py", "test_scanner_controls.py"}

SEARCH_ROOTS = ("tools", "tests")

# An inline opt-out for the one legitimate use: a test that must exercise the OLD
# spelling to prove it really does fail the race. Baselining that line instead would
# be wrong — a baseline row is debt to be burned down, and this line must never be.
NOLINT = "unpruned-walk: intentional"


def findings(roots: list[Path] | None = None) -> list[str]:
    """Rows for `roots`, defaulting to the repo's own tool + harness trees.

    Explicit roots exist so the scanner is drivable over a fixture directory: a gate
    lint that has never been shown to FIRE is a lint nobody has tested, and this repo
    requires a positive control for exactly that reason.
    """
    out: list[str] = []
    for root in roots if roots is not None else [REPO / r for r in SEARCH_ROOTS]:
        for py in sorted(Path(root).rglob("*.py")):
            if py.name in EXEMPT or "/target/" in str(py):
                continue
            try:
                text = py.read_text(errors="replace")
            except OSError:
                continue
            rel = py.relative_to(REPO) if py.is_relative_to(REPO) else py
            for i, line in enumerate(text.splitlines(), 1):
                if NOLINT in line:
                    continue
                if WALK_RE.search(line):
                    out.append(f"{rel}:{i}: unpruned rust walk: {line.strip()}")
                elif BUILD_WALK_RE.search(line):
                    out.append(f"{rel}:{i}: unpruned build walk: {line.strip()}")
    return out


def load_baseline() -> set[str]:
    if not BASELINE.is_file():
        return set()
    return {
        ln.strip()
        for ln in BASELINE.read_text().splitlines()
        if ln.strip() and not ln.startswith("#")
    }


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--lint", action="store_true", help="report only rows NEW vs the baseline")
    ap.add_argument("--write-baseline", action="store_true")
    ap.add_argument("roots", nargs="*", type=Path, help="directories to scan (default: the repo)")
    args = ap.parse_args(argv)

    found = findings(args.roots or None)
    if args.write_baseline:
        BASELINE.parent.mkdir(parents=True, exist_ok=True)
        BASELINE.write_text("".join(f"{f}\n" for f in found))
        print(f"wrote {len(found)} rows to {BASELINE.relative_to(REPO)}")
        return 0

    if args.lint:
        # Identity is file:line, so an edit above a hit shifts it and reads as NEW.
        # Match on the file plus the offending text instead — the walk is what is
        # being gated, not where it happens to sit today.
        def key(row: str) -> tuple[str, str]:
            f, _, rest = row.partition(":")
            return f, rest.split(": ", 1)[-1]

        base = {key(b) for b in load_baseline()}
        new = [f for f in found if key(f) not in base]
        for f in new:
            print(f)
        if new:
            print(f"\n{len(new)} NEW unpruned walk(s). Use tools/source_walk.py source_files().")
        return 1 if new else 0

    for f in found:
        print(f)
    print(f"\n{len(found)} unpruned walk(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
