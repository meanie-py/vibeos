#!/usr/bin/env python3
"""scan_ptr_highword_mask.py — `ptr & 0xFFFF0000 == 0` used as "ptr is at most 0xFFFF".

THE INCIDENT
    ReactOS' `RtlpCheckIntegerAtom` decides whether an atom NAME is really a MAKEINTATOM
    integer with

        if (!((ULONG_PTR)AtomName & 0xFFFF0000))

    which is "the pointer is at most 0xFFFF" only while pointers are 32 bits wide. On a
    64-bit kernel it tests bits 16-31 alone, so a real string at 0xFFFF....0000xxxx passes
    as integer atom xxxx. NtAddAtom/NtFindAtom copy every caller's name into a pool block;
    whenever that block sat at such an address, every name became the same atom for the
    rest of the boot, unrelated window properties collided (uxtheme read shell32's
    `CPLFlags` value 1 as its WND_DATA pointer), and arm64 control-panel applets died with
    no window in about half of all appstorm boots. The rtl resource walker carried the same
    test (`find_entry_by_name`: a name string looked up as a numeric id).

    The translation preserved the C faithfully, so nothing downstream had an opinion: no
    rustc lint, no difftest (the host harness never maps a string at ...0000_xxxx), and the
    symptom surfaced three DLLs away from the cause.

WHAT COUNTS AS A FINDING
    `(<operand> & 0xFFFF0000) == 0` or `!= 0` (either spelling of the mask, with or without
    a `_` and an integer suffix) where the operand contains a cast to a pointer-width type:
    `as usize`, `as isize`, `as ULONG_PTR`, `as UINT_PTR`, `as DWORD_PTR`, `as LONG_PTR`,
    `as INT_PTR`, `as SIZE_T` -- or the operand is a bare identifier that the same file
    declares with one of those types (`ptr_bits: usize`, the shape of the atom incident).
    That type is what says "this was a pointer"; a mask over a `u32` raster op or a version
    DWORD is a different thing and is never reported.

    Deliberately NOT reported: `== 0xFFFF0000`. user32/win32k's `IsCallProcHandle` uses that
    form, and its looseness is load-bearing: a call-proc handle that crossed a 32-bit LONG
    arrives sign-extended, and the bits-16-31 test is what still accepts it.

    The fix is `operand & !0xFFFF == 0` (Windows' `IS_INTRESOURCE`), which reads the whole
    pointer. Comments and literals are stripped first (the octal scanner's tokenizer), so the
    C quoted in a doc comment is not a finding.

BASELINE
    Empty. Both instances in the tree were fixed when this landed.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.lint_facts import source_facts
from tools.scan_octal_literals import strip_noise
from tools.source_walk import source_files

# `& 0xFFFF0000)` then `== 0` / `!= 0` — the mask closes the parenthesised operand.
MASK_TEST = re.compile(
    r"&\s*0x[fF]{4}_?0{4}(?:u32|u64|usize)?\s*\)\s*[!=]=\s*0(?:u32|u64|usize)?(?![\wxX])"
)

PTR_TYPES = r"(?:usize|isize|ULONG_PTR|UINT_PTR|DWORD_PTR|LONG_PTR|INT_PTR|SIZE_T)"
PTR_CAST = re.compile(r"\bas\s+" + PTR_TYPES + r"\b")
BARE_IDENT = re.compile(r"^\s*([A-Za-z_]\w*)\s*$")


def _operand(line: str, amp: int) -> str | None:
    """The text inside the parenthesis that the `&` at `amp` sits in, up to the `&`."""
    depth = 0
    for i in range(amp - 1, -1, -1):
        c = line[i]
        if c == ")":
            depth += 1
        elif c == "(":
            if depth == 0:
                return line[i + 1 : amp]
            depth -= 1
    return None


@source_facts()
def scan_text(text: str) -> list[tuple[int, str]]:
    """Return (line, matched expression) for each pointer-width high-word test."""
    out: list[tuple[int, str]] = []
    stripped = strip_noise(text)

    def pointer_width(operand: str) -> bool:
        if PTR_CAST.search(operand):
            return True
        ident = BARE_IDENT.match(operand)
        return bool(
            ident
            and re.search(r"\b" + re.escape(ident.group(1)) + r"\s*:\s*" + PTR_TYPES + r"\b", stripped)
        )

    for lineno, line in enumerate(stripped.split("\n"), 1):
        for m in MASK_TEST.finditer(line):
            operand = _operand(line, m.start())
            if operand is not None and pointer_width(operand):
                out.append((lineno, (operand + line[m.start() : m.end()]).strip()))
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
    roots = [repo / r for r in (args.root or ["sdk"])]

    findings = 0
    files = 0
    for root in roots:
        for path in sorted(source_files(root)):
            text = path.read_text(encoding="utf-8", errors="replace")
            files += 1
            for lineno, expr in scan_text(text):
                rel = path.relative_to(repo) if path.is_relative_to(repo) else path
                print(
                    f"{rel}:{lineno}: PTR_HIGHWORD_MASK `{expr}` tests bits 16-31 only; on "
                    "64-bit a real pointer at ...0000_xxxx passes. Write `p & !0xFFFF == 0`."
                )
                findings += 1

    print(f"scanned {files} .rs files, {findings} findings", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
