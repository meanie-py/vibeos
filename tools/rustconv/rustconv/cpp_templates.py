"""Rust layouts for C++ class TEMPLATES a module uses as a base, a field or a global.

`port cppapp` models a module's own classes from clang's AST, but a template instantiation
defined elsewhere (cicero's `CicArray<CUIFObject *>`, the base of `CUIFObjectArray`) has no
class of its own in the model. Without a layout `_external_base` must refuse, because a
zero-size placeholder would silently shift every field after it.

Each entry here names a template whose layout is INDEPENDENT of its arguments and a generic
Rust struct, written once by hand in the crate that owns the header, that lays it out.
`CicArray<T>` is `class CicArray : protected CicArrayBase` with no fields of its own: a
vptr, then `LPBYTE m_pb; size_t m_cItems, m_cbItem, m_cCapacity`, whatever T is. The Rust
spelling is the bare generic name; the owning crate defines it, and every consumer's prelude
re-exports it from `crate` (the dependency to add).

A template whose layout DOES depend on its arguments (a `T m_value` member, a `T[N]` array)
does not belong here: it needs per-instantiation layout from clang, which this does not do.
"""

from __future__ import annotations

import re
from collections.abc import Callable
from dataclasses import dataclass


@dataclass(frozen=True)
class TemplateBase:
    #: The Rust type to embed, e.g. `CicArray<*mut CUIFObject>`.
    rust_type: str
    #: The crate that defines the generic (a consumer's dependency), e.g. `cicero-rust`.
    crate: str
    #: The template carries a vptr (CicArrayBase's virtual destructor), which a class
    #: deriving from it shares instead of growing a second one.
    polymorphic: bool = False
    #: The template's destructor does work (frees `m_pb`), so the Rust generic implements
    #: Drop and nothing embedding it may derive Copy.
    drops: bool = False


@dataclass(frozen=True)
class _Entry:
    crate: str
    header: str
    #: The module in `crate` holding the hand-written generic.
    module: str
    polymorphic: bool = False
    drops: bool = False


#: Template name -> where its argument-independent generic layout lives.
REGISTRY: dict[str, _Entry] = {
    "CicArray": _Entry(
        "cicero-rust",
        "base/ctf/cicero/cicarray.h",
        "cicero::cicarray_t",
        polymorphic=True,
        drops=True,
    ),
    # `T_DATA *m_pIMCC` is the only T-dependent member: a pointer, the same for every T.
    "CIC_IMCC_LOCK": _Entry("cicero-rust", "base/ctf/cicero/cicimc.h", "cicero::cicimc_t"),
    "CicIMCCLock": _Entry(
        "cicero-rust", "base/ctf/cicero/cicimc.h", "cicero::cicimc_t", drops=True
    ),
}

_TEMPLATE_RE = re.compile(r"^(?:(?:class|struct)\s+)?(?:\w+::)*(\w+)\s*<(.*)>$")


def _split_args(text: str) -> list[str]:
    out, depth, cur = [], 0, []
    for ch in text:
        if ch in "(<[":
            depth += 1
        elif ch in ")>]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(ch)
    out.append("".join(cur).strip())
    return [a for a in out if a]


def template_base(name: str, map_type: Callable[[str], str]) -> TemplateBase | None:
    """The registered layout for template instantiation `name`, or None if unregistered.

    `map_type` turns each C++ template argument into its Rust spelling (cpp_app.rust_type).
    """
    m = _TEMPLATE_RE.match(" ".join(name.split()))
    if m is None:
        return None
    entry = REGISTRY.get(m.group(1))
    if entry is None:
        return None
    args = ", ".join(map_type(a) for a in _split_args(m.group(2)))
    return TemplateBase(f"{m.group(1)}<{args}>", entry.crate, entry.polymorphic, entry.drops)


def drops(name: str) -> bool:
    """Whether a registered template instantiation `name` has a destructor that does work."""
    tb = template_base(name, lambda a: a)
    return tb is not None and tb.drops


def island_reexport(name: str, crate: str) -> str | None:
    """The `pub use` that replaces bindgen's copy of registered template `name` in `crate`'s
    island, or None if unregistered. Bindgen can only spell one non-generic `struct CicArray`;
    left in place, integrate strips the hand-written generic as a duplicate of it."""
    entry = REGISTRY.get(name)
    if entry is None:
        return None
    root = "crate" if crate == entry.crate else entry.crate.replace("-", "_")
    return f"pub use {root}::{entry.module}::{name}; // cpp_templates owns the generic"


_TYPEDEF_RE = re.compile(r"\btypedef\s+((\w+)\s*<[^;]*>)\s+(\w+)\s*;")


def retype_aliases(island: str, sources: str, map_type: Callable[[str], str]) -> str:
    """`pub type CDispAttrPropCache = CicArray;` -> `... = CicArray<GUID>;`.

    Bindgen spells a typedef of a template instantiation with the bare template name, which
    against the generic is a missing-argument error. The arguments come from the C++
    `typedef CicArray<GUID> CDispAttrPropCache;` in the module's own headers."""
    for m in _TYPEDEF_RE.finditer(sources):
        if m.group(2) not in REGISTRY:
            continue
        tb = template_base(m.group(1), map_type)
        if tb is None:
            continue
        island = re.sub(
            rf"^(pub type {m.group(3)} = ){m.group(2)};$",
            rf"\g<1>{tb.rust_type};",
            island,
            flags=re.M,
        )
    return island
