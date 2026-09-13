#!/usr/bin/env python3
"""Duplicate `#[no_mangle]` symbol ratchet for the kernel link.

WHY THIS EXISTS
---------------
The kernel PE is linked from ~11 INDEPENDENT Rust staticlib graphs (the
`kernel-rust` umbrella + rtl / cmlib / guids / cport / csq / arbiter / lsalib /
rossym / cnt-rust-lib / pseh) plus a C floor. `-Wl,--allow-multiple-definition`
(reactos-link, "common to every module kind") lets lld resolve any symbol
defined in >1 of those archives FIRST-MATCH instead of erroring.

That flag is LOAD-BEARING and cannot simply be dropped:
  * Every independent Rust staticlib graph emits its OWN copy of the panic /
    eh_personality / compiler-builtins runtime — IRREDUCIBLE while N>1
    staticlibs feed one link.
  * Merging them into one graph is partly IMPOSSIBLE: rtl-rust / cnt-rust-lib
    build a DIFFERENT rustc target (user-mode hardfloat) than the kernel-json
    softfloat kernel-rust — they physically cannot share a crate graph.

So we keep the flag but RATCHET it: freeze the set of Rust `#[no_mangle]`
symbols that are defined in more than one kernel-linked crate, and FAIL if a
NEW one appears. That stops a future translation from silently re-defining a
symbol another crate already owns — the exact near-miss `IsEqualGUID` was.

WHAT IT MEASURES (and its limits)
---------------------------------
A SOURCE scan: it finds every `#[no_mangle]` fn/static DEFINITION (an attribute
that only ever precedes a real definition — never an `extern {}` import, so
import decls are correctly excluded) across the kernel-link crate group, and
reports any symbol name defined in >=2 of those crates.

This is a deliberately DIFFERENTIAL check, not an exact model of lld's link:
  * It does NOT try to know which archive members lld actually pulls (only the
    linker knows; rustc hides linker output on a successful link). So the
    absolute count includes benign entries — those freeze into the baseline.
  * It does NOT see compiler-emitted runtime symbols (memcpy, eh_personality)
    or C-side cohab — both out of scope here.
What it DOES reliably catch is the thing that matters: a NEW cross-crate
`#[no_mangle]` collision introduced by a translation. Everything present today
is baselined; only net-new collisions fail the gate.

USAGE
-----
  scan_duplicate_symbols.py            # human-readable census
  scan_duplicate_symbols.py --lint     # bare `SYM\\tcrates` finding lines

Wired into `vibeos-cli check` as a source lint: the harness baseline-diffs the
`--lint` output against tools/lint-baseline/scan_duplicate_symbols.txt and gates
NEW lines only (regenerate with `vibeos-cli check --update-baseline
scan_duplicate_symbols`). The baseline doubles as the accepted-cohab registry.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent

# The crates whose object code is linked into the kernel PE (ntoskrnl-cargo /
# ntkrnlmp-cargo `.rust_lib`/`.rust_staticlib` set). A `#[no_mangle]` symbol
# defined in two of these collides at link and is masked by the flag.
KERNEL_LINK_CRATES = [
    "kernel-rust",  # umbrella staticlib (pulls ntoskrnl-rust + extracted crates)
    "ntoskrnl-rust",
    "lpc-rust",
    "fsrtl-rust",
    "dbgk-rust",
    "wmi-rust",
    "dbgprint-rust",
    "cc-rust",
    "iomgr-rust",
    "kdbg-rust",
    "config-rust",
    "ex-rust",
    "ob-rust",
    "ps-rust",
    "kd-rust",
    "kd64-rust",
    "vf-rust",
    "rtlk-rust",
    "po-rust",
    "inbv-rust",
    "vdm-rust",
    "fstub-rust",
    "se-rust",
    "pnpmgr-rust",
    "guids-rust",
    "rtl-rust",
    "cnt-rust-lib",
    "cmlib-rust",
    "cport-rust",
    "csq-rust",
    "arbiter-rust",
    "lsalib-rust",
    "rossym-rust",
    "pseh-rust",
]
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

# `#[no_mangle]` attribute, then (skipping further attrs / cfg / doc lines) the
# defined item. Matches `pub? (unsafe)? (extern "..")? fn NAME` and
# `pub? static (mut)? NAME`. An attribute can't sit on an extern-block decl, so
# this only ever pairs with real definitions.
# Anchored to line start so a `#[no_mangle]` mentioned inside a `//` comment
# (e.g. "provide inlines (NOT #[no_mangle])") is NOT taken as an attribute.
NO_MANGLE = re.compile(r"^\s*#\s*\[\s*no_mangle\s*\]")
FN_DEF = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)")
STATIC_DEF = re.compile(r"\bstatic\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)")
# lines that may legitimately sit between the attr and the item
ATTR_OR_DOC = re.compile(r"^\s*(#\s*\[|//|/\*|\*)")


def symbols_in_crate(crate: str) -> set[str]:
    src = RUST_ROOT / crate / "src"
    if not src.is_dir():
        return set()
    found: set[str] = set()
    for rs in source_files(src):
        try:
            lines = rs.read_text(errors="replace").splitlines()
        except OSError:
            continue
        for i, ln in enumerate(lines):
            if not NO_MANGLE.search(ln):
                continue
            # walk forward to the first non-attr/doc line = the defined item
            for j in range(i + 1, min(i + 8, len(lines))):
                nxt = lines[j]
                if ATTR_OR_DOC.match(nxt) or not nxt.strip():
                    continue
                m = FN_DEF.search(nxt) or STATIC_DEF.search(nxt)
                if m:
                    found.add(m.group(1))
                break
    return found


# Structural runtime: symbols each independent Rust staticlib graph legitimately
# defines its own copy of (the panic/eh runtime). Irreducible by the multi-
# staticlib design, so they are NOT ratchet debt — excluded from the census.
STRUCTURAL = re.compile(
    r"^(rust_eh_personality|rust_begin_unwind|rust_oom|__rust_[a-z_]+|_fltused)$"
)


def census() -> dict[str, list[str]]:
    owners: dict[str, set[str]] = {}
    for crate in KERNEL_LINK_CRATES:
        for sym in symbols_in_crate(crate):
            if STRUCTURAL.match(sym):
                continue
            owners.setdefault(sym, set()).add(crate)
    return {s: sorted(c) for s, c in owners.items() if len(c) >= 2}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--lint",
        action="store_true",
        help="emit bare `SYM<TAB>crates` finding lines for the "
        "harness baseline-diff (vibeos-cli check)",
    )
    args = ap.parse_args()

    dups = census()

    if args.lint:
        for s in sorted(dups):
            print(f"{s}\t{','.join(dups[s])}")
        return 0

    print("=== duplicate #[no_mangle] census (kernel link group) ===")
    print(f"collisions: {len(dups)}")
    for s in sorted(dups):
        print(f"  {s}\t{','.join(dups[s])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
