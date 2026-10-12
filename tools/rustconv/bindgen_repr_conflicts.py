#!/usr/bin/env python3
"""Print the bindgen types rustc cannot lay out, one per line, for a second `--opaque-type` pass.

A C struct under `#pragma pack(N)` that holds a `DECLSPEC_ALIGN(M)` member is legal C (MSVC
lets the member's alignment win), but Rust has no spelling for it: bindgen emits
`#[repr(C, packed(N))]` beside `#[repr(align(M))]` (E0587), or a packed struct containing an
aligned type (E0588). dbghelp.h's MINIDUMP callback structs hold a CONTEXT, which is
`align(16)` on amd64, and one of them fails the whole island of every module that includes
dbghelp.h (drwtsn32). Re-running bindgen with these names as `--opaque-type` keeps their
size and alignment exactly (clang computes them) and drops the field layout Rust cannot
express. Every OTHER type keeps its fields: the set is computed, not guessed.

Usage: bindgen_repr_conflicts.py <island.rs>
"""

from __future__ import annotations

import re
import sys

_ITEM = re.compile(r"((?:^#\[[^\n]*\]\n)+)pub (?:struct|union) (\w+)\s*\{(.*?)^\}", re.M | re.S)
_FIELD_TYPE = re.compile(r"^\s*pub \w+:\s*(.+?),\s*$", re.M)


def _value_types(field_type: str) -> list[str]:
    """Types a field holds BY VALUE (arrays included, pointers and references not)."""
    t = field_type.strip()
    # The RAW island still wraps union members (`__BindgenUnionField<MINIDUMP_THREAD_CALLBACK>`);
    # REAL_UNIONS rewrites them later. The member is held by value either way.
    while m := re.fullmatch(r"(?:\w+::)*(?:__BindgenUnionField|ManuallyDrop)<(.+)>", t):
        t = m.group(1).strip()
    while True:
        m = re.fullmatch(r"\[(.+);\s*[^;\]]+\]", t)
        if not m:
            break
        t = m.group(1).strip()
    if t.startswith(("*", "&")) or "fn(" in t:
        return []
    return [re.sub(r"<.*$", "", t).split("::")[-1]]


def conflicts(text: str) -> list[str]:
    # `pub type MINIDUMP_THREAD_CALLBACK = _MINIDUMP_THREAD_CALLBACK;`: fields name the alias.
    alias = dict(re.findall(r"^pub type (\w+) = (\w+);", text, re.M))

    def resolve(t: str) -> str:
        seen = set()
        while t in alias and t not in seen:
            seen.add(t)
            t = alias[t]
        return t

    packed: dict[str, list[str]] = {}
    aligned: set[str] = set()
    for attrs, name, body in _ITEM.findall(text):
        fields = [resolve(v) for f in _FIELD_TYPE.findall(body) for v in _value_types(f)]
        if re.search(r"\bpacked\b", attrs):
            packed[name] = fields
        if re.search(r"\balign\(", attrs):
            aligned.add(name)
    bad = {n for n in packed if n in aligned}  # E0587
    # E0588, transitively: an aligned type's alignment propagates to whatever holds it.
    holds_align = set(aligned)
    changed = True
    while changed:
        changed = False
        for attrs, name, body in _ITEM.findall(text):
            if name in holds_align:
                continue
            fields = [resolve(v) for f in _FIELD_TYPE.findall(body) for v in _value_types(f)]
            if any(f in holds_align for f in fields):
                holds_align.add(name)
                changed = True
    bad |= {n for n, fields in packed.items() if any(f in holds_align for f in fields)}
    return sorted(bad)


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2
    with open(sys.argv[1], encoding="utf-8") as f:
        for name in conflicts(f.read()):
            print(name)
    return 0


if __name__ == "__main__":
    sys.exit(main())
