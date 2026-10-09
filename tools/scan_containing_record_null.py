#!/usr/bin/env python3
"""
scan_containing_record_null.py — a CONTAINING_RECORD whose field became a null pointer.

    C:     Dope = CONTAINING_RECORD(NextEntry, DEVICE_OBJECT_POWER_EXTENSION, Volume);
    Rust:  dope = CONTAINING_RECORD(next_entry as PVOID, core::ptr::null_mut(),
                                    core::ptr::null_mut()) as PDEVICE_OBJECT_POWER_EXTENSION;

The macro's type and field are not values, so the translator declared an extern taking two
PVOIDs and passed null for both. The symbol it links against is ntoskrnl's
`CONTAINING_RECORD(address, type_size, field_offset)`, so the offset is 0: right only when
the field happens to be first. `Volume` is the LAST field of the power extension, so po's
volume flush read a DEVICE_OBJECT out of the list links and bugchecked 0x1E at setup's
reboot; kdbg's symbol loader walked InInitializationOrderLinks at offset 0. Nothing
compiles differently.

Every call with a null last argument is reported -- pass `core::mem::offset_of!(T, Field)`
even when the field is first, so the layout is stated rather than assumed. Gated at zero.

Usage:
    python3 tools/scan_containing_record_null.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_containing_record_null.py --lint
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_wide_byte_copy import SKIP_FILE_RE
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
MARKER = "ORIGINAL C SOURCE"
CALL_RE = re.compile(r"\bCONTAINING_RECORD\w*(?:::<[^>]*>)?\s*\(")
NULL_LAST_RE = re.compile(r"(?:core::ptr::|ptr::)?null(?:_mut)?\(\)\s*(?:as\s+[\w:]+\s*)?,?\s*$")


def _args_text(text: str, open_paren: int) -> str | None:
    depth = 0
    for j in range(open_paren, len(text)):
        if text[j] == "(":
            depth += 1
        elif text[j] == ")":
            depth -= 1
            if depth == 0:
                return text[open_paren + 1 : j]
    return None


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    findings = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if f.suffix != ".rs" or SKIP_FILE_RE.search(str(f)):
                continue
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            cut = text.find(MARKER)
            if cut >= 0:
                text = text[:cut]
            code = re.sub(
                r"//[^\n]*|/\*.*?\*/", lambda m: re.sub(r"[^\n]", " ", m.group(0)), text, flags=re.S
            )
            for m in CALL_RE.finditer(code):
                args = _args_text(code, m.end() - 1)
                if args is None or "," not in args:
                    continue  # a declaration's parameter list has no call shape worth checking
                if code[: m.start()].rstrip().endswith("fn"):
                    continue
                if NULL_LAST_RE.search(args.strip()):
                    lineno = code.count("\n", 0, m.start()) + 1
                    findings.append((f, lineno, " ".join(args.split())[:100]))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, what in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        tag = "containing-record-null" if lint else "CONTAINING_RECORD with a null field:"
        print(f"{rel}:{lineno}: {tag} ({what})")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): pass core::mem::offset_of!(T, Field).")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
