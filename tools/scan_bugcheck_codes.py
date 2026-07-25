#!/usr/bin/env python3
"""
scan_bugcheck_codes.py — keep magic numbers out of KeBugCheck/KeBugCheckEx.

THE BUG CLASS (2026-07-24, field manual §9.113). Translated C
`KeBugCheck(SYMBOL)` calls came back from the LLM as `KeBugCheck(0xNN)` with a
number it made up. A sweep found 61 of ~115 stop codes wrong across ex, se, po,
cc, fsrtl, config, classpnp, iomgr, mm and ke — every one of them plausible-
looking and internally consistent, which is exactly why review slid off them:

    ke/thrdschd.rs  INVALID_AFFINITY_SET          0x3  -> 0x48 (an IRP bugcheck)
    ke/wait.rs      MAXIMUM_WAIT_OBJECTS_EXCEEDED 0xC  -> 0x36
    ke/i386/irqobj  TRAP_CAUSE_UNKNOWN            0x12 -> 0x7F
    ex/init.rs      the whole *_INITIALIZATION_FAILED family, shifted

Nothing else can catch this. rustc sees a valid ULONG; the code only shows up
when the kernel dies, and then it names a subsystem nobody touched — the bugcheck
screen actively lies about where to look.

The fix is the AGENTS.md substrate rule: constants never go through the LLM. Pass
a named constant from `ntkrnl_sys::bugcodes` (generated from bugcodes.mc by
tools/gen_bugcodes.py) so the value is defined in exactly one place:

    KeBugCheckEx(bugcodes::INVALID_AFFINITY_SET, 0, 0, 0, 0);

Usage:
    python3 tools/scan_bugcheck_codes.py [paths...]   # default: sweep sdk/lib/rust
    python3 tools/scan_bugcheck_codes.py --lint       # terse (CLI baseline gate)

Suppress a verified false positive on the call's first line:

    KeBugCheckEx(0x101, ...); // nolint: bugcheck-code (no ReactOS symbol exists)

Baseline is EMPTY and must stay that way — every kernel stop code in the tree is
symbolic. Exit 1 if any finding.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

SKIP_FILE_RE = re.compile(r"(bindgen_[a-z0-9_]+\.rs$|/target/|/\.responses/|/tests?/)")
NOLINT_RE = re.compile(r"//\s*nolint(?::\s*bugcheck-code\b|\s*$|:?\s*\()")

CALL_RE = re.compile(r"\bKeBugCheck(?:Ex)?\s*\(")
# A Rust integer literal may carry a type suffix (0x35u32, 5usize, 0x1E_u32) and
# `_` digit separators. The original pattern required the digits to be followed
# directly by `,` or `)`, so `KeBugCheckEx(0x35u32, ...)` in classpnp-rust hid
# from this gate for six weeks while the scanner reported a clean 0. A lint that
# under-matches reads exactly like a lint that passes — match the whole literal.
NUM_RE = re.compile(r"^\s*(0x[0-9A-Fa-f_]+|\d[\d_]*)(?:[ui](?:8|16|32|64|128|size))?\s*[,)]")


def scan_file(path: Path) -> list[tuple[int, str, str]]:
    """[(line_no, code_literal, source_line)] for numeric-literal stop codes."""
    try:
        lines = path.read_text(errors="replace").splitlines()
    except OSError:
        return []
    out = []
    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith("//") or stripped.startswith("*"):
            continue
        for m in CALL_RE.finditer(line):
            # The code may sit on the call line or wrap to the next ones.
            rest = line[m.end() :]
            probe, probe_line = rest, i
            for lookahead in range(1, 3):
                if probe.strip():
                    break
                if i + lookahead >= len(lines):
                    break
                probe_line = i + lookahead
                probe = lines[probe_line]
            hit = NUM_RE.match(probe)
            if not hit:
                continue
            if NOLINT_RE.search(line) or NOLINT_RE.search(lines[probe_line]):
                continue
            out.append((i + 1, hit.group(1), stripped[:110]))
    return out


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    lint = "--lint" in sys.argv
    roots = [Path(a) for a in args] if args else [RUST_ROOT]

    files: list[Path] = []
    for root in roots:
        if root.is_file():
            files.append(root)
        elif root.is_dir():
            files.extend(root.rglob("*.rs"))

    findings = []
    for f in sorted(set(files)):
        if SKIP_FILE_RE.search(str(f)):
            continue
        for ln, code, src in scan_file(f):
            try:
                rel = f.relative_to(REPO)
            except ValueError:
                rel = f
            findings.append((str(rel), ln, code, src))

    for rel, ln, code, src in findings:
        if lint:
            print(f"{rel}:{ln}: bugcheck-code {code}")
        else:
            print(f"{rel}:{ln}: numeric stop code {code} — use ntkrnl_sys::bugcodes::NAME")
            print(f"    {src}")

    if not lint:
        print(f"\n{len(findings)} numeric bugcheck code(s)")
        if findings:
            print("fix: pass a named constant from ntkrnl_sys::bugcodes (tools/gen_bugcodes.py)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
