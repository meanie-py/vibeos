#!/usr/bin/env python3
"""Give a C++-mode island's COM interfaces their C (`CINTERFACE`) shape.

A C++ module's island is bindgen'd with `-x c++`, where WIDL's headers declare every
interface as a C++ class: bindgen renders `ITfSource` as `{ _base: IUnknown }` down to an
opaque `IUnknown { vtable_: *const IUnknown__bindgen_vtable }`. Nothing can call through
that, and cppapp builds a COM class's vtable from WIDL's flattened `IFooVtbl`, which only
the C view declares. Rendered against the opaque shape, a class implementing
`ITfThreadMgrEventSink` came out with no COM vtable at all: it compiles, and the first call
TSF makes through it jumps nowhere.

The module's own headers do not compile with CINTERFACE (they `override` the interface's
methods), so the C view comes from a second bindgen of the SDK headers alone (`COMRAW`).
Both shapes are one pointer, so swapping `IFoo`'s body for `{ lpVtbl: *mut IFooVtbl }` and
adding `IFooVtbl` keeps every layout. Only interfaces the island already has are touched,
and only when the C view has their Vtbl.

Slots are rewritten to `extern "system"`: bindgen spells STDMETHODCALLTYPE `"stdcall"` on
i386 and `"C"` elsewhere, and `"system"` is exactly that pair, so one spelling serves all
three islands (and is what cppapp's vtable parser reads).

Usage: bindgen_com_cinterface.py FINAL COMRAW
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

_STRUCT = re.compile(r"((?:#\[[^\n]*\]\n)*)pub struct (\w+) \{\n(.*?)\n\}\n", re.S)


def blocks(text: str) -> dict[str, tuple[int, int, str]]:
    """name -> (start, end, body) of every `pub struct` block, attributes included."""
    return {m.group(2): (m.start(), m.end(), m.group(3)) for m in _STRUCT.finditer(text)}


def merge(final: str, comraw: str) -> tuple[str, list[str]]:
    com = blocks(comraw)
    have = blocks(final)
    opaque = re.compile(r"\s*pub (?:_base: \w+|vtable_: \*const \w+__bindgen_vtable),?\s*")
    swaps: list[tuple[int, int, str]] = []
    done: list[str] = []
    for name, (s, e, body) in have.items():
        vt = com.get(f"{name}Vtbl")
        iface = com.get(name)
        if vt is None or iface is None or not opaque.fullmatch(body):
            continue
        if f"{name}Vtbl" in have:
            raise SystemExit(f"{name}Vtbl is already in the island; refusing to define it twice")
        if not re.fullmatch(rf"\s*pub lpVtbl: \*(?:mut|const) {name}Vtbl,?\s*", iface[2]):
            raise SystemExit(f"C view of {name} is not a lone lpVtbl: {iface[2]!r}")
        vbody = re.sub(r'unsafe extern "(?:stdcall|C)" fn', 'unsafe extern "system" fn', vt[2])
        text = (
            "#[repr(C)]\n#[derive(Debug, Copy, Clone)]\n"
            f"pub struct {name} {{\n    pub lpVtbl: *mut {name}Vtbl,\n}}\n"
            "#[repr(C)]\n#[derive(Debug, Copy, Clone)]\n"
            f"pub struct {name}Vtbl {{\n{vbody}\n}}\n"
        )
        swaps.append((s, e, text))
        done.append(name)
    for s, e, text in sorted(swaps, reverse=True):
        final = final[:s] + text + final[e:]
    return final, sorted(done)


def main() -> int:
    final_path, com_path = sys.argv[1], sys.argv[2]
    final = Path(final_path).read_text()
    out, done = merge(final, Path(com_path).read_text())
    Path(final_path).write_text(out)
    print(f"bindgen_com_cinterface: {len(done)} interface(s) given their C vtable shape")
    return 0


if __name__ == "__main__":
    sys.exit(main())
