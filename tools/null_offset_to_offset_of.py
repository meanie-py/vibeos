#!/usr/bin/env python3
"""Rewrite CONTAINING_RECORD's offsetof-through-null into `core::mem::offset_of!`.

    python3 tools/null_offset_to_offset_of.py [paths] [--apply]
    python3 tools/null_offset_to_offset_of.py --lint      # the gate: any null place is NEW

The C `CONTAINING_RECORD(p, T, Field)` / Wine `LIST_ENTRY(elem, T, field)` subtracts
`offsetof(T, Field)`, and the translations spelled offsetof the way C macros used to:

    &raw mut (*core::ptr::null_mut::<T>()).Field as ULONG_PTR
    core::ptr::addr_of_mut!((*core::ptr::null_mut::<T>()).Field) as ULONG_PTR

That is undefined behaviour in Rust even though only an address is taken: a field
projection is an in-bounds offset of its base, and null has no allocation to be in bounds
of (Miri rejects it; rustc's `deref_nullptr` flags only some spellings, 7 of about 100
in the tree on 2026-10-16). `core::mem::offset_of!(T, Field)` is the defined spelling and
the same number.

Only a site whose value is immediately cast to an integer (`as ULONG_PTR`, `as usize`,
`as c_ulonglong`) is rewritten, so `usize` replacing the pointer changes nothing
downstream. Anything else is listed and left alone. Without --apply, prints what it would
do. Exit 1 if a site was skipped.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.source_walk import source_files  # noqa: E402

_NULL = r"\(\s*\*\s*(?:::)?(?:core::ptr::|ptr::)?null(?:_mut)?::<\s*(?P<ty>[A-Za-z_][\w:]*)\s*>\s*\(\s*\)\s*\)"  # noqa: E501
_FIELDS = r"(?P<fields>(?:\s*\.\s*[A-Za-z_]\w*)+)"
SITE = re.compile(
    r"(?P<raw>&raw\s+(?:mut|const)\s+)" + _NULL + _FIELDS
    + r"|(?P<mac>(?:(?:::)?core::ptr::|ptr::)?addr_of(?:_mut)?!\s*\(\s*)" + _NULL.replace("ty>", "ty2>")  # noqa: E501
    + _FIELDS.replace("fields>", "fields2>") + r"\s*\)"
)  # fmt: skip
CAST_AFTER = re.compile(r"\s*as\s+(?:::)?[\w:]+")
# Any place expression through a null pointer, whatever surrounds it (the --lint net).
NULL_PLACE = re.compile(
    r"\(\s*\*\s*(?:::)?(?:core::ptr::|ptr::|std::ptr::)?null(?:_mut)?(?:::<[^>]*>)?\s*\(\s*\)"
)


def _in_comment(text: str, at: int) -> bool:
    line_start = text.rfind("\n", 0, at) + 1
    return "//" in text[line_start:at]


def rewrite(text: str) -> tuple[str, int, list[str]]:
    """(new text, sites rewritten, [skipped site descriptions])."""
    out: list[str] = []
    pos = 0
    n = 0
    skipped: list[str] = []
    for m in SITE.finditer(text):
        ty = m.group("ty") or m.group("ty2")
        fields = re.sub(r"\s+", "", m.group("fields") or m.group("fields2")).lstrip(".")
        line = text.count("\n", 0, m.start()) + 1
        if not CAST_AFTER.match(text, m.end()):
            skipped.append(f"{line}: not cast to an integer: {' '.join(m.group(0).split())}")
            continue
        out.append(text[pos : m.start()])
        out.append(f"core::mem::offset_of!({ty}, {fields})")
        pos = m.end()
        n += 1
    out.append(text[pos:])
    return "".join(out), n, skipped


def main(argv: list[str]) -> int:
    apply = "--apply" in argv
    lint = "--lint" in argv
    args = [Path(a) for a in argv if not a.startswith("--")] or [REPO / "sdk" / "lib" / "rust"]
    files = [f for a in args for f in ([a] if a.is_file() else sorted(source_files(a)))]
    total = 0
    bad = 0
    for f in files:
        if "/target" in f.as_posix():
            continue
        text = f.read_text(errors="replace")
        if "null" not in text:
            continue
        if lint:
            try:
                rel = f.resolve().relative_to(REPO)
            except ValueError:
                rel = f
            hits = [m for m in NULL_PLACE.finditer(text) if not _in_comment(text, m.start())]
            for _ in hits:
                print(f"{rel}: [NULL_PLACE]")
            bad += len(hits)
            continue
        new, n, skipped = rewrite(text)
        try:
            rel = f.resolve().relative_to(REPO)
        except ValueError:
            rel = f
        for s in skipped:
            print(f"SKIP {rel}:{s}")
        bad += len(skipped)
        if n:
            print(f"{rel}: {n}")
            total += n
            if apply:
                f.write_text(new)
    if lint:
        return 1 if bad else 0
    print(
        f"{'rewrote' if apply else 'would rewrite'} {total} site(s); skipped {bad}", file=sys.stderr
    )
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
