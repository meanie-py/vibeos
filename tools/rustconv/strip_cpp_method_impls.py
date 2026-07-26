#!/usr/bin/env python3
"""Strip bindgen's C++ METHOD-WRAPPER impl blocks from a generated island.

WHY. For a C++ class, bindgen emits two things: an `extern "C"` declaration of each member
function under its flattened name (``CFSFolder_FormatSize``), and an inherent ``impl`` block
of thin wrappers that call them::

    impl CFSFolder {
        #[inline]
        pub unsafe fn FormatSize(size: LONGLONG, Buf: LPWSTR, cchBuf: UINT) -> HRESULT {
            CFSFolder_FormatSize(size, Buf, cchBuf)
        }
    }

A cohab crate blocklists the declaration, because the TRANSLATION defines that symbol — that
is the whole point of BLOCKLIST_FUNCS_FROM_CRATE. The wrapper survives the blocklist and then
calls a function that is no longer declared, so the island fails to compile on its own output:
138 × ``E0425: cannot find function`` on shell32's first run with functions enabled.

The wrappers are unwanted regardless. A translated module calls the free function directly;
nothing in the tree calls `Type::method()` through a bindgen wrapper.

WHAT IT MUST NOT REMOVE. Two other kinds of `impl` are load-bearing:

  * bindgen's own generic helpers — ``impl<T> __BindgenUnionField<T>``, ``impl<T> Default
    for __BindgenOpaqueArray<T, N>``. These carry type parameters and/or are trait impls.
  * BITFIELD ACCESSORS — ``impl SHELLSTATE { pub fn fShowAllObjects(&self) -> BOOL { ...
    self._bitfield_1.get(0usize, 1u8) ... } }``. These are the only correct way to read a C
    ``:1`` field, and the field manual has a standing rule against hand-rolling them.

So the discriminator is not "inherent impl". It is "inherent impl that calls the flattened
member-function form ``<Type>_<name>(``" — bitfield accessors never do, which is checked
against the real output rather than assumed.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

#: `impl Foo {` — no generics, no trait, i.e. an inherent impl on a plain named type.
_INHERENT_IMPL = re.compile(r"^impl\s+([A-Za-z_]\w*)\s*\{", re.M)


def _match_brace(text: str, open_index: int) -> int | None:
    """Index of the `}` closing the `{` at ``open_index``, or None."""
    depth = 0
    for i in range(open_index, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i
    return None


def strip(text: str) -> tuple[str, list[str]]:
    """Return (new_text, names of the types whose wrapper impl was removed)."""
    removed: list[str] = []
    out = text
    while True:
        for match in _INHERENT_IMPL.finditer(out):
            name = match.group(1)
            end = _match_brace(out, match.end() - 1)
            if end is None:
                continue
            body = out[match.end() : end]
            # The tell: a call to the flattened member form `<Type>_<something>(`.
            if not re.search(rf"\b{re.escape(name)}_\w+\s*\(", body):
                continue
            out = out[: match.start()] + out[end + 1 :].lstrip("\n")
            removed.append(name)
            break
        else:
            return out, removed


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: strip_cpp_method_impls.py <island.rs>", file=sys.stderr)
        return 2
    path = Path(argv[1])
    text = path.read_text()
    new, removed = strip(text)
    if removed:
        path.write_text(new)
    print(f"stripped {len(removed)} C++ method-wrapper impl block(s): {', '.join(removed)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
