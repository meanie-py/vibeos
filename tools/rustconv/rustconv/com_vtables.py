"""COM interfaces in a C++ island, given their C layout.

bindgen run over a C++ translation unit sees a WIDL interface as a C++ class: `IFoo : IBar`
with pure virtual members, which it emits as `pub struct IFoo { pub _base: IBar }` down to
`IUnknown { pub vtable_: *const IUnknown__bindgen_vtable }` -- one opaque vptr. Nothing can be
called through that, and nothing can implement it: cleanmgr's `CEmptyVolumeCacheCallBack`
came out as a plain struct handed to COM, which would call through a garbage vptr.

The same header's C section (`#if !defined(__cplusplus) || defined(CINTERFACE)`) has the
flattened `IFooVtbl`, IUnknown's three slots first, which IS the C++ vtable layout: one vptr,
the slots in declaration order, base first. Defining CINTERFACE over the whole island is not
an option -- ATL's `p->Release()` needs the C++ shape -- so gen_driver_types.sh binds just
the interfaces again in C mode and this module splices the result in:

* `names`: the interfaces the C++ island lays out, and every name it already defines;
* `merge`: each interface's C body (`lpVtbl: *mut IFooVtbl`) replaces its C++ body, and
  the C pass's other items (the Vtbl structs, any type they name that the island lacks) are
  appended unless the island already defines the name.

Same size, same alignment, same bytes: only the vptr gets a type.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

_IFACE_RE = re.compile(
    r"^pub struct (I[A-Z]\w*) \{\n\s*pub (?:_base: \w+|vtable_: \*const \w+__bindgen_vtable),\n\}",
    re.M,
)
_NAME_RE = re.compile(r"^pub (?:struct|union|enum|type|const|static(?: mut)?) (\w+)", re.M)


def interfaces(island: str) -> list[str]:
    """Interface structs laid out as C++ classes: a lone `_base` or bindgen vptr."""
    return sorted(set(_IFACE_RE.findall(island)))


def items(text: str) -> list[tuple[str | None, str]]:
    """Top-level items of bindgen output as (defined name or None, text with attributes)."""
    out: list[tuple[str | None, str]] = []
    pending: list[str] = []
    cur: list[str] = []
    depth = 0
    for ln in text.splitlines(keepends=True):
        if not cur:
            if ln.startswith("#[") or ln.startswith("#!["):
                pending.append(ln)
                continue
            if not ln.strip():
                continue
            cur = [*pending, ln]
            pending = []
        else:
            cur.append(ln)
        depth += ln.count("{") - ln.count("}")
        if depth == 0 and ln.rstrip().endswith(("}", ";")):
            body = "".join(cur)
            m = _NAME_RE.search(body)
            out.append((m.group(1) if m else None, body))
            cur = []
    if cur or pending:
        out.append((None, "".join([*cur, *pending])))
    return out


def merge(island: str, c_pass: str) -> tuple[str, list[str]]:
    """The island with each interface's C body, plus the C pass's new items. Returns the
    text and the interfaces left in their C++ shape (no C section found for them)."""
    ifaces = set(interfaces(island))
    c_items = items(c_pass)
    c_bodies = {n: t for n, t in c_items if n in ifaces and re.search(r"\blpVtbl\b", t) is not None}
    out: list[str] = []
    for n, t in items(island):
        out.append(c_bodies.get(n, t) if n else t)
    have = {n for n, _ in items(island) if n}
    added = [t for n, t in c_items if n and n not in have]
    tail = "".join(out) + (
        "\n// COM interfaces' C layout (rustconv.com_vtables)\n" + "".join(added) if added else ""
    )
    return tail, sorted(ifaces - set(c_bodies))


def _split_top(s: str) -> list[str]:
    """Split on commas outside (), <> and []."""
    out, depth, cur = [], 0, []
    for i, ch in enumerate(s):
        if ch in "(<[":
            depth += 1
        elif ch in ")]" or (ch == ">" and s[i - 1 : i] != "-"):  # `->` closes nothing
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
    if "".join(cur).strip():
        out.append("".join(cur).strip())
    return out


_SLOT_RE = re.compile(
    r'^pub (\w+): (?:::core::option::)?Option<\s*unsafe extern "(\w+)" fn'
    r"\((.*)\)(?:\s*->\s*(.+?))?\s*>$"
)


def methods(island: str) -> str:
    """`impl IFoo { pub unsafe fn Slot(&mut self, ..) }` for every interface in the C shape: a
    C++ `p->GetSpaceUsed(a, b)` then reads `(*p).GetSpaceUsed(a, b)`, as the bodies are
    written, and dispatches through the vtable exactly as the C++ call does."""
    out = []
    for n, t in items(island):
        m = re.match(
            r"(?s).*?pub struct (I[A-Z]\w*) \{\s*pub lpVtbl: \*(?:mut|const) (\w+),\s*\}", t
        )
        if not n or not m:
            continue
        vt = next((b for nn, b in items(island) if nn == m.group(2)), None)
        if vt is None:
            continue
        body = " ".join(vt[vt.index("{") + 1 : vt.rindex("}")].split())
        fns = []
        for field in _split_top(body):
            sm = _SLOT_RE.match(field)
            if sm is None:
                continue
            name, _abi, params, ret = sm.groups()
            ps = _split_top(params)[1:]  # the first is `This`
            names = [p.split(":", 1)[0].strip() for p in ps]
            sig = ", ".join(["&mut self", *ps])
            r = f" -> {ret.rstrip(',').strip()}" if ret else ""
            args = ", ".join(["self", *names])
            fns.append(
                f"    pub unsafe fn {name}({sig}){r} {{\n"
                f"        match (*self.lpVtbl).{name} {{\n"
                f"            Some(f) => f({args}),\n"
                f'            None => panic!("{n}::{name}: null vtable slot"),\n'
                "        }\n    }\n"
            )
        if fns:
            out.append(f"impl {n} {{\n{''.join(fns)}}}\n")
    return "".join(out)


METHODS_MARKER = "// COM interface methods through the vtable (rustconv.com_vtables)"


def island_has_methods(src: Path) -> bool:
    """Whether a crate's island carries the per-slot interface methods."""
    return any(
        METHODS_MARKER in f.read_text(errors="replace") for f in sorted(src.glob("types_*.rs"))
    )


def _main(argv: list[str]) -> int:
    if len(argv) == 2 and argv[0] == "names":
        island = Path(argv[1]).read_text()
        print(" ".join(interfaces(island)))
        return 0
    if len(argv) == 2 and argv[0] == "defined":
        island = Path(argv[1]).read_text()
        print("\n".join(sorted(set(_NAME_RE.findall(island)) - set(interfaces(island)))))
        return 0
    if len(argv) == 3 and argv[0] == "merge":
        island = Path(argv[1]).read_text()
        text, missing = merge(island, Path(argv[2]).read_text())
        impls = methods(text)
        if impls:
            text += f"\n{METHODS_MARKER}\n" + impls
        Path(argv[1]).write_text(text)
        if missing:
            print(f"com_vtables: no C section for {', '.join(missing)} (left C++-shaped)")
        return 0
    print("usage: com_vtables names|defined ISLAND | merge ISLAND C_PASS", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(_main(sys.argv[1:]))
