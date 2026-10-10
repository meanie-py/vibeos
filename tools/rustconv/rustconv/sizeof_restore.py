"""Put back a `sizeof` the model dropped around a TYPE name, from the C it came from.

Haiku hands back a sealed `sizeof(PROPSHEETPAGE)` as the bare token, so the body reads
`psp.dwSize = PROPSHEETPAGE as DWORD;` (appwiz, then hdwwiz six times). New purchases are
repaired at unseal (fact_seal `in_sizeof`); this pass repairs bodies already banked, whose
seal context is gone. It is deliberately narrow: the name must be a TYPE the crate knows
(a type is never a value, so a bare one in value position is always this mistake), the
function's own C must take `sizeof` of exactly that name, and the use must sit in a value
slot -- after `=`, `(` or `,`, and before `as`, `,`, `)` or `;`.
"""

from __future__ import annotations

import re
from pathlib import Path

from .crate_fix import c_function_text
from .function_coverage import rust_definition_bodies

_SIZEOF = re.compile(r"\bsizeof\s*\(\s*([A-Za-z_]\w*)\s*\)")


def crate_types(crate_src: Path) -> set[str]:
    names: set[str] = set()
    for p in [*crate_src.glob("types_*.rs"), crate_src / "prelude.rs"]:
        if p.is_file():
            names |= set(
                re.findall(
                    r"^\s*pub (?:type|struct|union) (\w+)", p.read_text(errors="replace"), re.M
                )
            )
    return names


def restore(rust: str, c_file: Path, types: set[str]) -> tuple[str, list[str]]:
    cache: dict = {}
    fixed: list[str] = []
    for name, body in rust_definition_bodies(rust).items():
        c = c_function_text(c_file, name, cache)
        if not c:
            continue
        wanted = set(_SIZEOF.findall(c)) & types
        if not wanted:
            continue
        new = body
        for t in sorted(wanted):
            pat = re.compile(rf"(?<=[=(,])(\s*){re.escape(t)}(?=\s*(?:as\b|,|\)|;))")

            def sub(m: re.Match[str], t: str = t, text: str = new) -> str:
                # A macro's first argument is a TYPE slot (`FIELD_OFFSET!(METARECORD, f)`,
                # `offset_of!(T, f)`): rewriting it handed winegdi and both font drivers a
                # `size_of::<T>()` where the macro wants the type, breaking the parse.
                if re.search(r"!\s*\($", text[: m.start()].rstrip(" \t")):
                    return m.group(0)
                return f"{m.group(1)}core::mem::size_of::<{t}>()"

            spelled = f"core::mem::size_of::<{t}>()"
            before = new.count(spelled)
            new = pat.sub(sub, new)
            n = new.count(spelled) - before
            if n:
                fixed.append(f"{name}: sizeof({t}) x{n}")
        if new != body:
            rust = rust.replace(body, new, 1)
    return rust, fixed
