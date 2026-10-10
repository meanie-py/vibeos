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


@dataclass(frozen=True)
class _Entry:
    crate: str
    header: str


#: Template name -> where its argument-independent generic layout lives.
REGISTRY: dict[str, _Entry] = {
    "CicArray": _Entry("cicero-rust", "base/ctf/cicero/cicarray.h"),
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
    return TemplateBase(f"{m.group(1)}<{args}>", entry.crate)
