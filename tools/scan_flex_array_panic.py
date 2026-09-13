#!/usr/bin/env python3
"""Find bindgen flexible-array bounds checks that survived into a built object.

bindgen models a C flexible array (`T arr[ANYSIZE_ARRAY]` / `arr[1]`) as `[T; 1]`,
so a Rust `arr[i]` with a runtime `i` emits a bounds check that PANICS for every
i >= 1 while the C intentionally over-allocates. Source grep cannot separate that
from an index into a genuinely 1-element array, and it misses the shape that is
hardest to see by eye:

    *RootCell = addr_of_mut!((*p).List[i]) as PHCELL_INDEX

which reads as address-taking. It is not: the index inside the place expression
still bounds-checks, and `&raw mut` does not suppress it. cmlib's CmpSelectLeaf
shipped three of those in the kernel.

The OBJECT CODE can tell them apart. A `panic_bounds_check` call is preceded by a
push of the ARRAY LENGTH, so a length of 1 names the flexible-array shape and any
larger length is a real fixed array to leave alone.

It grades the BUILT ARTIFACT, so a stale `build-<arch>` reports the state of the
last build and not of the tree -- do a build before believing a drop to zero.

A HIT IS A SHAPE, NOT A BUG. The discriminator this reader cannot see is whether
the C over-allocates: `PRIVILEGE_SET`/`LPCP_DATA_INFO` do (real bugs), while
`PKNODE KeNodeBlock[1] = { &KiNode0 };` really is one element and its check merely
survives because the guard is a runtime variable. Read the C declaration first.

There is one SYSTEMATIC false-positive source, and it is a direct consequence of
this being i386-only: an array sized by an arch-dependent constant that happens to
equal 1 on 32-bit. `MmSystemPageDirectory[PPE_PER_PAGE]` is `[PFN_NUMBER; 512]` on
amd64/arm64 and `[PFN_NUMBER; 1]` on i386 (mm.rs:1265), so it is flagged here and
would not be flagged at all on a 64-bit read. Check the declaration's LENGTH
EXPRESSION before the length: a literal `1` is the bindgen flexible-array shape, a
named constant is an arch-sized real array. Three of the first six sites graded
fell this way.

CORPUS IS DECLARED, NOT DISCOVERED. Globbing `build-<arch>` would grade whatever
binaries a worktree happens to carry -- a verdict that depends on state outside
the diff (field manual, "A gate can be RED for reasons that are not in the tree
at all"). The corpus is the set of workspace crates that declare a staticlib, and
a member missing from the build dir is reported, never skipped silently.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RUST_WS = REPO / "sdk" / "lib" / "rust"
OBJDUMP = REPO / ".build-tools" / "llvm-mingw" / "bin" / "llvm-objdump"

# The immediate pushed just before the call is the array length. i386 pushes it;
# on other arches the length lands in an argument register, so this reader is
# i386-only by construction and says so rather than silently grading nothing.
_LEN_PUSH = re.compile(r"pushl\s+\$0x([0-9a-f]+)$")
_SYMBOL = re.compile(r"^[0-9a-f]+ <(.+)>:$")
_CARRY = re.compile(r"pushl|calll|IMAGE_REL|movl|leal")
_PANIC = "panic_bounds_check"


def declared_corpus() -> list[str]:
    """Crate names that declare a staticlib -- the workspace IS the declaration."""
    out = []
    for cargo in sorted(RUST_WS.glob("*/Cargo.toml")):
        text = cargo.read_text()
        if "staticlib" not in text:
            continue
        m = re.search(r'^\s*name\s*=\s*"([^"]+)"', text, re.M)
        if m:
            out.append(m.group(1))
    return out


def find_artifact(build: Path, crate: str) -> Path | None:
    want = "lib" + crate.replace("-", "_") + ".a"
    for dirpath, dirnames, filenames in os.walk(build):
        # A crate's own cargo target dir under the build tree is intermediate
        # output, not the artifact the link consumes. PRUNE it rather than
        # filtering its paths afterwards: a post-filter has already descended
        # into the one tree a concurrent build is rewriting, so it can raise
        # FileNotFoundError out of the walk before it ever inspects a path.
        dirnames[:] = sorted(d for d in dirnames if not d.endswith("-target"))
        if want in filenames:
            return Path(dirpath) / want
    return None


def scan_object(path: Path) -> list[tuple[str, int]]:
    """[(symbol, array_len)] for every bounds check whose array length is 1."""
    proc = subprocess.run(
        [str(OBJDUMP), "-dr", str(path)], capture_output=True, text=True, check=False
    )
    if proc.returncode != 0:
        raise RuntimeError(f"{OBJDUMP} failed on {path}: {proc.stderr.strip()[:200]}")
    sym = "?"
    pending: int | None = None
    hits = []
    for line in proc.stdout.splitlines():
        m = _SYMBOL.match(line)
        if m:
            sym = m.group(1)
            pending = None
            continue
        m = _LEN_PUSH.search(line)
        if m:
            pending = int(m.group(1), 16)
            continue
        if _PANIC in line:
            if pending == 1:
                hits.append((sym, pending))
            pending = None
            continue
        if not _CARRY.search(line):
            pending = None
    return hits


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", default="i386", help="only i386 encodes the length as a push")
    ap.add_argument("--lint", action="store_true", help="exit 1 when any site is found")
    args = ap.parse_args(argv)

    if args.arch != "i386":
        print(
            f"scan_flex_array_panic: {args.arch} passes the array length in a register, "
            "not a push; this reader is i386-only. Refusing to grade nothing.",
            file=sys.stderr,
        )
        return 2
    if not OBJDUMP.is_file():
        print(f"MISSING TOOL: {OBJDUMP}", file=sys.stderr)
        return 2
    build = REPO / f"build-{args.arch}"
    if not build.is_dir():
        print(
            f"MISSING BUILD: {build}\nfix: ./vibeos-cli build --arch {args.arch}", file=sys.stderr
        )
        return 2

    corpus = declared_corpus()
    missing, findings = [], []
    for crate in corpus:
        art = find_artifact(build, crate)
        if art is None:
            missing.append(crate)
            continue
        for sym, _ in scan_object(art):
            findings.append((crate, sym))

    for crate, sym in sorted(set(findings)):
        print(f"{crate}: {sym}: bounds check on a 1-element array (bindgen flexible array?)")
    print(
        f"\n{len(set(findings))} site(s) in {len({c for c, _ in findings})} module(s); "
        f"corpus {len(corpus)} declared, {len(missing)} not built",
        file=sys.stderr,
    )
    if missing:
        # Not a silent skip: an unbuilt member is missing COVERAGE, and saying so
        # is the difference between "clean" and "did not look".
        print(f"NOT BUILT (no coverage): {' '.join(sorted(missing))}", file=sys.stderr)
    return 1 if (args.lint and findings) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
