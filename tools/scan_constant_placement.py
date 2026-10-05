#!/usr/bin/env python3
"""Constants that traded places between a C function and its Rust translation.

The fact seal guarantees a translated body holds the C's constants as a SET, which a
permutation satisfies (`a = 10; b = 20;` -> `a = 20; b = 10;`). This runs rustconv's
placement audit (tools/rustconv/rustconv/placement_audit.py) over every Rust function
whose same-named C twin (tools/c_twin.py) lives in a related module, and reports a
ring of constants each sitting where the C puts another one.

It reports only cycles, never a single unexpected placement, and stays silent when a
statement gives no context to compare; see the module docstring for why.

Fix: put each constant back where the C has it. If the Rust is right (a deliberate
divergence from a C bug), mark the function `// nolint: constant-placement (<why>)`
on one of the 6 lines above it.

USAGE
  scan_constant_placement.py [PATH ...]        # report
  scan_constant_placement.py --lint [PATH ...] # one finding per line (vibeos-cli check)
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
sys.path.insert(0, str(Path(__file__).resolve().parent / "rustconv"))
from rustconv.placement_audit import placement_findings

from tools.c_twin import CIndex, affinity, blank_rust, prefer, rust_functions
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
SKIP_PATH_RE = re.compile(r"(/target/|/\.responses/|/bindgen_|/types_[a-z0-9]+\.rs$)")
NOLINT = "nolint: constant-placement"


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    lint = "--lint" in sys.argv
    roots = [Path(a) for a in args] if args else [RUST_ROOT]
    files: list[Path] = []
    for root in roots:
        if root.is_file():
            files.append(root)
        elif root.is_dir():
            files.extend(source_files(root))
        else:
            print(f"scan_constant_placement: no such path: {root}", file=sys.stderr)
            return 2
    funcs = []
    for f in sorted(set(files)):
        if SKIP_PATH_RE.search(str(f)):
            continue
        raw = f.read_text(errors="replace")
        lines = raw.splitlines()
        for fn in rust_functions(blank_rust(raw), f):
            if any(NOLINT in ln for ln in lines[max(0, fn.line - 7) : fn.line]):
                continue
            funcs.append(fn)
    index = CIndex()
    index.load({fn.name for fn in funcs})

    found = 0
    for fn in funcs:
        twins = prefer(fn.path, index.get(fn.name))
        if not twins or affinity(fn.path, twins[0]) <= 0:
            continue
        for finding in placement_findings(twins[0].body, fn.body):
            found += 1
            rel = fn.path.relative_to(REPO) if fn.path.is_relative_to(REPO) else fn.path
            ring = "/".join(finding.ring)
            if lint:
                print(f"{rel}:{fn.line}: constant-placement {fn.name} {ring}")
            else:
                c_rel = twins[0].path.relative_to(REPO)
                print(f"{rel}:{fn.line}: {fn.name} (C: {c_rel}:{twins[0].line})")
                print("    " + finding.message().replace("\n", "\n    "))
    if not lint:
        print(f"\n{found} finding(s) over {len(funcs)} Rust function(s)")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
