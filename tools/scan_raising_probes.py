#!/usr/bin/env python3
"""
scan_raising_probes.py — Rust kernel code that calls a RAISING user-pointer probe.

THE POINT. The C kernel probes user pointers with ProbeForRead / ProbeForWrite and
the typed probe.h macros (ProbeForWriteHandle, ProbeForReadLargeInteger, ...), all of
which RAISE STATUS_ACCESS_VIOLATION / STATUS_DATATYPE_MISALIGNMENT, always inside a
`_SEH2_TRY` whose `_SEH2_EXCEPT` turns the raise into a returned status. This kernel
has no SEH (DUMMY_PSEH), so in Rust a raising probe on a bad user pointer is an
unhandled exception: BUGCHECK 0x1E where the C returned an error. Any unprivileged
process could take the machine down with one bad syscall argument.

The fix shape is always the same: the `Result` probes in `ntkrnl_sys::probe`
(`probe_user_buffer`, `probe_read_generic`, `probe_write_generic`,
`safe_copy_from_user`, ...) with the C handler's cleanup run on `Err`. 2026-10-04
drove the class to zero (about 160 sites across 20 crates), so this gates at ZERO
rather than against a baseline: a baseline would be a place for the next one to hide.

WHAT IS FLAGGED. An unqualified call to one of the raising names below. Qualified
calls (`ntkrnl_sys::inline::common::ProbeForWriteHandle(..)`) are not: those are the
C-ABI re-exports in ntoskrnl-rust (`common_inlines.rs`, `shim_exports.rs`), which
must keep the raising contract because C callers wrap them in their own SEH.
Definitions and `extern` declarations are not calls. Files that DEFINE the raising
primitives are skipped (EXEMPT_FILES).

A local helper that shadows a typed name and returns a `Result` (`fn
ProbeForReadUnicodeString(..) -> Result<..>`) is fine and is not flagged: its name
is not in RAISING unless the crate imports the raising one.

`MmProbeAndLockPages` raises the same way and is flagged however it is spelled,
qualified or not (`ntkrnl_sys::ddk::MmProbeAndLockPages(..)` is a real call, not a
re-export). Use `ntkrnl_sys::probe::MmTryProbeAndLockPages`, which returns the
status. A call inside a `seh::try_except(..)` or `seh::try_finally(..)` body is not
flagged: those are real SEH frames (`ntkrnl_sys::seh`, the C trampolines in
ntoskrnl/sehtry.c). A `try_except` catches the raise; a `try_finally` runs its
cleanup and lets the raise reach the caller's `try_except`, which is how the FSDs
(cdfs, fastfat) are written in the C too.

Suppress a reviewed site with `// nolint: raising-probe <reason>` on the line.

Usage:
    python3 tools/scan_raising_probes.py              # scan sdk/lib/rust
    python3 tools/scan_raising_probes.py --lint       # gate: exit 1 on any finding
    python3 tools/scan_raising_probes.py <path ...>   # files or directories
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
DEFAULT_ROOT = REPO / "sdk" / "lib" / "rust"

# The raising exports and the typed probe.h helpers that raise (ntkrnl-sys
# inline/common.rs, inline/mm.rs, macros.rs).
RAISING = (
    "ProbeForRead",
    "ProbeForWrite",
    "ProbeForWriteHandle",
    "ProbeForWriteUlong",
    "ProbeForWritePointer",
    "ProbeForWriteBoolean",
    "ProbeForWriteLangId",
    "ProbeForWriteSize_t",
    "ProbeForWriteIoStatusBlock",
    "ProbeForReadLargeInteger",
    "ProbeForReadUchar",
    "ProbeForReadUshort",
)

# Raising names with no qualified re-export to spare, so any spelling is a call.
RAISING_ANY_PATH = ("MmProbeAndLockPages",)

CALL_RE = re.compile(
    r"(?<![\w:])(" + "|".join(RAISING) + r")\s*\("
    r"|(?<!\w)(" + "|".join(RAISING_ANY_PATH) + r")\s*\("
)
DECL_RE = re.compile(r"\bfn\s+(" + "|".join(RAISING + RAISING_ANY_PATH) + r")\b")
TRY_EXCEPT_RE = re.compile(r"\btry_(?:except|finally)\s*\(")

# Files that define the raising primitives themselves.
EXEMPT_FILES = (
    "ex-rust/src/exintrin.rs",
    "ntkrnl-sys/src/inline/common.rs",
    "ntkrnl-sys/src/inline/mm.rs",
    "ntkrnl-sys/src/macros.rs",
    "ntkrnl-sys/src/probe.rs",
)

NOLINT_RE = re.compile(r"nolint:\s*(raising-probe|raising_probe|scan_raising_probes|all)\b")


def _strip_comments(text: str) -> list[tuple[str, str]]:
    """Return (code, raw) per line with // and /* */ comments blanked out."""
    out: list[tuple[str, str]] = []
    in_block = False
    for raw in text.splitlines():
        code = []
        i = 0
        in_str = False
        while i < len(raw):
            if in_block:
                end = raw.find("*/", i)
                if end < 0:
                    i = len(raw)
                    break
                in_block = False
                i = end + 2
                continue
            ch = raw[i]
            if in_str:
                code.append(ch)
                if ch == "\\":
                    if i + 1 < len(raw):
                        code.append(raw[i + 1])
                    i += 2
                    continue
                if ch == '"':
                    in_str = False
                i += 1
                continue
            if raw.startswith("//", i):
                break
            if raw.startswith("/*", i):
                in_block = True
                i += 2
                continue
            if ch == '"':
                in_str = True
                code.append(" ")
                i += 1
                continue
            code.append(ch)
            i += 1
        out.append(("".join(code), raw))
    return out


def _try_except_lines(lines: list[str]) -> set[int]:
    """1-based line numbers inside the argument list of a `try_except(`/`try_finally(` call."""
    inside: set[int] = set()
    for start, code in enumerate(lines):
        for m in TRY_EXCEPT_RE.finditer(code):
            depth = 0
            row, col = start, m.end() - 1
            while row < len(lines):
                line = lines[row]
                while col < len(line):
                    if line[col] == "(":
                        depth += 1
                    elif line[col] == ")":
                        depth -= 1
                        if depth == 0:
                            break
                    col += 1
                inside.add(row + 1)
                if depth == 0 and col < len(line):
                    break
                row, col = row + 1, 0
    return inside


def scan_file(path: Path) -> list[str]:
    rel = path.as_posix()
    if any(rel.endswith(e) for e in EXEMPT_FILES):
        return []
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return []
    if not CALL_RE.search(text):
        return []
    findings = []
    stripped = _strip_comments(text)
    guarded = _try_except_lines([code for code, _ in stripped])
    for lineno, (code, raw) in enumerate(stripped, 1):
        if DECL_RE.search(code) or NOLINT_RE.search(raw) or lineno in guarded:
            continue
        m = CALL_RE.search(code)
        if m:
            name = m.group(1) or m.group(2)
            findings.append(
                f"{rel}:{lineno}: raising {name}() — no SEH here; use ntkrnl_sys::probe"
            )
    return findings


def _files(paths: list[Path]) -> list[Path]:
    out: list[Path] = []
    for p in paths:
        if p.is_dir():
            out.extend(sorted(source_files(p)))
        elif p.suffix == ".rs":
            out.append(p)
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--lint", action="store_true", help="gate: exit 1 on any finding")
    ap.add_argument("paths", nargs="*", type=Path)
    args = ap.parse_args(argv)

    findings: list[str] = []
    for f in _files(args.paths or [DEFAULT_ROOT]):
        findings.extend(scan_file(f))
    for f in findings:
        print(f)
    if findings:
        print(
            f"\n{len(findings)} raising probe call(s). Convert each to the Result probes in "
            "ntkrnl_sys::probe and run the C _SEH2_EXCEPT handler's cleanup on Err.",
            file=sys.stderr,
        )
        return 1
    if args.lint:
        print("OK — 0 raising probe calls.", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
