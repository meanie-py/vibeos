#!/usr/bin/env python3
"""scan_swprintf_count.py — a counted `swprintf` declared against msvcrt's legacy export.

THE INCIDENT
    C's `swprintf(buf, count, fmt, ...)` is not an export at all: sdk/include/crt/stdio.h
    defines it as a static inline over `_vsnwprintf`. msvcrt.dll's real `swprintf` export
    is the legacy, non-conforming `(buf, fmt, ...)` (msvcrt.spec: `swprintf(ptr wstr)`).
    A translation that declares the C prototype in an `extern` block therefore links the
    legacy export and hands it the COUNT as the format string. mfplat's Rust crashed in
    msvcrt's printf on its first `MFTRegister` (pointer 0x27 read as a wide string), and
    the same declaration, copied between preludes by `port resolve`'s tree consensus, was
    already shipping in ole32, msiexec (its custom-action pipe name), wscript and the
    recycle bin.

WHAT COUNTS AS A FINDING
    * COUNTED_UNLINKED: an extern `fn swprintf`/`fn vswprintf` DECLARATION (no body) whose
      second parameter is an integer count, without `#[link_name = "_snwprintf"]`
      (`_vsnwprintf`, or the `_swprintf_c` family).
    * LEGACY_RELINKED: the two-argument `(buf, fmt, ...)` form linked to a counted export,
      which is the same mismatch the other way round.

    A Rust function DEFINITION of that name is not a finding: it is not the export.
    Findings go to stdout for the baseline comparison; the tally goes to stderr.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

COUNTED_EXPORTS = {"_snwprintf", "_vsnwprintf", "_swprintf_c", "_vswprintf_c"}
_DECL = re.compile(
    r"(?P<attrs>(?:[ \t]*#\[[^\]]*\]\s*)*)[ \t]*(?:pub(?:\([^)]*\))?\s+)?fn\s+"
    r"(?P<name>v?swprintf)\s*\((?P<params>[^)]*)\)\s*(?:->\s*[^;{]+)?;"
)
_LINK = re.compile(r'link_name\s*=\s*"(\w+)"')
_COUNT_TYPE = re.compile(
    r"^(?:usize|u32|u64|c_uint|c_ulong|::core::ffi::c_uint|size_t|SIZE_T|DWORD|UINT|ULONG|"
    r"c_int|i32|INT)$"
)


def scan_text(text: str) -> list[tuple[int, str, str]]:
    """(line, function, finding kind) for each mismatched declaration."""
    out = []
    for m in _DECL.finditer(text):
        params = [p.strip() for p in m.group("params").split(",") if p.strip()]
        named = [p for p in params if p != "..."]
        link = _LINK.search(m.group("attrs"))
        target = link.group(1) if link else m.group("name")
        second = named[1].split(":", 1)[-1].strip() if len(named) >= 2 else ""
        counted = len(named) >= 3 and bool(_COUNT_TYPE.match(second))
        line = text.count("\n", 0, m.start("name")) + 1
        if counted and target not in COUNTED_EXPORTS:
            out.append((line, m.group("name"), "COUNTED_UNLINKED"))
        elif not counted and len(named) == 2 and target in COUNTED_EXPORTS:
            out.append((line, m.group("name"), "LEGACY_RELINKED"))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        action="append",
        default=None,
        help="directory to scan (repeatable); defaults to sdk/",
    )
    args = ap.parse_args()
    repo = Path(__file__).resolve().parent.parent
    findings = files = 0
    for root in [repo / r for r in (args.root or ["sdk"])]:
        for path in sorted(source_files(root)):
            files += 1
            for lineno, name, kind in scan_text(path.read_text(encoding="utf-8", errors="replace")):
                print(f"{path.relative_to(repo)}:{lineno}: {kind} `{name}`")
                findings += 1
    print(f"scanned {files} .rs files, {findings} findings", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
