#!/usr/bin/env python3
"""gen_commacros.py — emit COBJMACROS-shaped call helpers from com-sys bindgen.

The wine-derived C sources call COM methods through the COBJMACROS macros from
the SDK headers (`IMalloc_Free(p, mem)` → `p->lpVtbl->Free(p, mem)`). rustconv's
raw translations reproduce those call sites verbatim, but emit the helpers as
`extern "C"` declarations — which link to nothing. This generator produces the
real thing: inlinable Rust fns that call straight through the vtbl slot.

The wrapper body never names a calling convention: it invokes the `Option<fn>` slot,
and com-sys's bindgen already encodes STDMETHODCALLTYPE per arch (extern "stdcall" on
i386, extern "C" on amd64/arm64).

BUT THE GENERATED FILE IS STILL PER-ARCH, and this is not optional. A method whose
PARAMETER is itself a function pointer (e.g. `IViewObject::Draw`'s `pfnContinue`
continue-callback) has that callback's calling convention baked into the parameter
TYPE — and bindgen spells it per-arch too. Generating once from the amd64 bindgen and
sharing it makes i386 fail with:

    expected "stdcall" fn, found "C" fn
    expected `Option<unsafe extern "stdcall" fn(_) -> _>`
       found `Option<unsafe extern "C" fn(_) -> _>`

So: run this once per arch against the matching bindgen_<arch>.rs and cfg-select the
outputs, exactly as com-sys does for its own bindings.

`unwrap_unchecked()` on the Option keeps C semantics exactly — a NULL vtbl slot
faults at the call, same as C.

Usage (per arch):
    for a in i386 amd64 arm64; do
      tools/gen_commacros.py --bindgen sdk/lib/rust/com-sys/src/bindgen_$a.rs \
          -o sdk/lib/rust/ole32-rust/src/commacros_$a.rs
    done
    # optionally restrict to the interfaces a crate actually uses:
    tools/gen_commacros.py ... --only IMalloc,IMoniker,IBindCtx
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

OPEN2CLOSE = {"(": ")", "<": ">", "{": "}", "[": "]"}


def _is_arrow_gt(text: str, i: int) -> bool:
    """True if text[i] is the '>' of a `->` return arrow, not a closing angle bracket."""
    return text[i] == ">" and i > 0 and text[i - 1] == "-"


def match_balanced(text: str, start: int) -> int:
    """Index of the delimiter closing the one at `start` (which must be an opener).

    Rust makes this fiddly: a `>` may be the tail of a `->` return arrow rather than
    a closing angle bracket (`Option<unsafe extern "C" fn(..) -> HRESULT>`). Counting
    that `>` as a closer truncates the payload — so skip it explicitly.
    """
    opener = text[start]
    closer = OPEN2CLOSE[opener]
    depth = 0
    i = start
    while i < len(text):
        c = text[i]
        if c == ">" and _is_arrow_gt(text, i):
            i += 1
            continue
        if c == opener:
            depth += 1
        elif c == closer:
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ValueError(f"unbalanced {opener!r} at {start}")


def split_top_level(text: str, sep: str = ",") -> list[str]:
    """Split on `sep`, ignoring separators nested inside (), <>, [], {} (arrow-aware)."""
    parts, depth, cur = [], 0, []
    for i, c in enumerate(text):
        if c in "(<[{":
            depth += 1
        elif c in ")>]}" and not _is_arrow_gt(text, i):
            depth -= 1
        if c == sep and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(c)
    if "".join(cur).strip():
        parts.append("".join(cur))
    return [p.strip() for p in parts if p.strip()]


VTBL_RE = re.compile(r"pub struct (I[A-Za-z0-9_]*)Vtbl\s*\{")
FIELD_RE = re.compile(r"pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*::core::option::Option\s*<")


def parse_vtbls(src: str) -> dict[str, list[tuple[str, list[tuple[str, str]], str | None]]]:
    """-> {iface: [(method, [(argname, argtype)], ret_or_None)]} (This excluded from args)."""
    out: dict[str, list] = {}
    for m in VTBL_RE.finditer(src):
        iface = m.group(1)
        body_open = src.index("{", m.end() - 1)
        body = src[body_open + 1 : match_balanced(src, body_open)]

        methods = []
        for fm in FIELD_RE.finditer(body):
            name = fm.group(1)
            opt_open = body.index("<", fm.end() - 1)
            payload = body[opt_open + 1 : match_balanced(body, opt_open)]

            fn_idx = payload.find("fn")
            if fn_idx < 0:
                continue
            paren_open = payload.index("(", fn_idx)
            paren_close = match_balanced(payload, paren_open)
            args_src = payload[paren_open + 1 : paren_close]

            tail = payload[paren_close + 1 :].strip()
            ret = None
            if tail.startswith("->"):
                ret = tail[2:].strip().rstrip(",").strip()

            args = []
            for a in split_top_level(args_src):
                if ":" not in a:
                    continue
                an, at = a.split(":", 1)
                args.append((an.strip(), at.strip()))
            if not args:
                continue  # every COM method takes `This`
            methods.append((name, args[1:], ret))  # drop This
        if methods:
            out[iface] = methods
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--bindgen", required=True, help="path to com-sys bindgen_<arch>.rs")
    ap.add_argument("-o", "--output", required=True)
    ap.add_argument("--only", help="comma list of interfaces to emit (default: all)")
    args = ap.parse_args()

    src = Path(args.bindgen).read_text()
    vtbls = parse_vtbls(src)
    if not vtbls:
        print("error: no I*Vtbl structs parsed — did the bindgen format change?", file=sys.stderr)
        return 1

    wanted = None
    if args.only:
        wanted = {s.strip() for s in args.only.split(",") if s.strip()}
        missing = wanted - vtbls.keys()
        if missing:
            print(f"error: interfaces not in bindgen: {sorted(missing)}", file=sys.stderr)
            return 1

    lines = [
        "//! GENERATED by tools/gen_commacros.py from com-sys bindgen — do not hand-edit.",
        "//! COBJMACROS-shaped call helpers (`IFoo_Method(p, ...)`), the role the C macros",
        "//! from the COBJMACROS headers played for the wine sources. Bodies call straight",
        "//! through the vtbl slot; the fn-pointer type carries the per-arch calling",
        "//! convention (com-sys bindgen: stdcall on i386, C on amd64/arm64), so this file",
        "//! is ABI-correct on all three targets without a single cfg.",
        "//! `unwrap_unchecked` on the Option<fn> keeps C semantics exactly (a NULL slot",
        "//! faults at the call, same as C).",
        "#![allow(clippy::all)]",
        "use crate::prelude::*;",
        "",
    ]

    def emit(iface, fn_name, slot, margs, ret):
        params = ", ".join(f"{a}: {t}" for a, t in margs)
        params = f"This: *mut {iface}" + (f", {params}" if params else "")
        rett = f" -> {ret}" if ret else ""
        callargs = ", ".join(["This"] + [a for a, _ in margs])
        lines.append("#[inline]")
        lines.append(f"pub unsafe fn {iface}_{fn_name}({params}){rett} {{")
        lines.append(f"    ((*(*This).lpVtbl).{slot}.unwrap_unchecked())({callargs})")
        lines.append("}")

    n_if = n_fn = n_alias = 0
    for iface in sorted(vtbls):
        if wanted is not None and iface not in wanted:
            continue
        n_if += 1
        lines.append(f"// ---- {iface} ----")
        names = {m for m, _, _ in vtbls[iface]}
        for method, margs, ret in vtbls[iface]:
            n_fn += 1
            emit(iface, method, method, margs, ret)

            # A/W-macro pollution: the Windows SDK's ANSI/wide macros (GetObject →
            # GetObjectA, TranslateAccelerator → ...A, GetClassInfo → ...A) expand
            # INSIDE the COM interface definitions, so bindgen sees a suffixed vtbl
            # slot. The C sources call the UNSUFFIXED COBJMACRO name (in C the same
            # macro renames both sides, so it's self-consistent there) — and so do
            # the translations derived from them. Emit an unsuffixed alias onto the
            # real slot. Skip when the base name already exists, or when both the A
            # and W variants exist (a genuine ANSI/wide pair, where an unsuffixed
            # alias would be ambiguous).
            if method[-1:] in ("A", "W") and len(method) > 1:
                base = method[:-1]
                sibling = base + ("W" if method.endswith("A") else "A")
                if base not in names and sibling not in names:
                    n_alias += 1
                    lines.append(f"// alias: C call sites use the unsuffixed COBJMACRO name")
                    emit(iface, base, method, margs, ret)
        lines.append("")

    Path(args.output).write_text("\n".join(lines) + "\n")
    print(f"wrote {args.output}: {n_fn} helpers (+{n_alias} A/W aliases) across {n_if} interfaces")
    return 0


if __name__ == "__main__":
    sys.exit(main())
