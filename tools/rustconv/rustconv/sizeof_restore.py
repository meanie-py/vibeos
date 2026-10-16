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
_MACRO_NAME = re.compile(r"[A-Za-z_]\w*!\s*\Z")


def _in_macro_args(text: str, pos: int) -> bool:
    """True when `pos` is an argument (any position) of the innermost enclosing call, and that
    call is a macro: `FIELD_OFFSET!(T, f)` takes its type FIRST, `CONTAINING_RECORD!(p, T, f)`
    SECOND -- an exemption for the first argument only turned mfplat's two CONTAINING_RECORDs
    into `CONTAINING_RECORD!(format, size_of::<T>(), field)` on every integrate."""
    depth = 0
    for i in range(pos - 1, -1, -1):
        c = text[i]
        if c in ")]}":
            depth += 1
        elif c in "([{":
            if depth == 0:
                return c == "(" and bool(_MACRO_NAME.search(text, 0, i))
            depth -= 1
        elif c == ";" and depth == 0:
            return False
    return False


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
            count = 0

            def put_back(m: re.Match, t: str = t) -> str:
                nonlocal count
                # A macro's argument takes a TYPE there by design:
                # `FIELD_OFFSET!(DATATYPES_INFO_1W, pName)` (winprint) is not a sizeof.
                if _in_macro_args(m.string, m.start()):
                    return m.group(0)
                count += 1
                return f"{m.group(1)}core::mem::size_of::<{t}>()"

            new = pat.sub(put_back, new)
            if count:
                fixed.append(f"{name}: sizeof({t}) x{count}")
        if new != body:
            rust = rust.replace(body, new, 1)
    return rust, fixed
