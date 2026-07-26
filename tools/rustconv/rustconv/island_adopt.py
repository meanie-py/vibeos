#!/usr/bin/env python3
"""Deterministic repair for the classes that appear when a TU adopts a bindgen ISLAND.

`crate_repair` closes the numeric/pointer-cast roots.  This closes the *different* set that
only shows up when a translated TU stops globbing a hand-written substrate module and starts
globbing the island instead — measured on shell32's first two Wave-5 leaves, then applied to
the 21K-line `shell32/` batch.  Every rule here is driven by a rustc diagnostic, edits one byte
range, and REFUSES rather than guesses when the shape is not the one it recognises.

Why these are not in `crate_repair`: each is a consequence of the island being *more faithful*
than the substrate it replaced, not of a translation defect.

  * BITFIELD_ACCESSOR (E0615) — bindgen renders a C `:1` field as `x.f()` / `x.set_f(v)`; a
    hand-written substrate had flattened it to a plain `w0: u32` word the translation read
    directly.  The rule must refuse a multi-line right-hand side: taking "the rest of the line"
    as the argument corrupts the call (it did, on recycler_prop.cpp:464, and the file stopped
    parsing).
  * UNINIT_LOCAL (E0381) — C declares a local and lets the callee fill it through a pointer.
    Rust wants it initialised before the address is taken.
  * MACRO_CALL_SYNTAX (E0423) — the translation emits `_countof(x)`, but the item is a
    `macro_rules!`, so the call needs a `!`.  Only macros the crate actually exports qualify.
  * VARIADIC_PROMOTION (E0617) — C promotes a `wchar_t`/`short` through `...`; Rust refuses,
    so the argument needs an explicit `as c_int`.
  * REF_AS_INT (E0606) — `&mut x as LPARAM` is not a valid cast; the address must be taken
    first.  `addr_of_mut!` and not `&mut x as *mut _ as _`, because the latter creates a
    reference to a possibly-uninitialised or aliased place on the way.

Pure core (`propose`) so the rules are testable without a cargo run.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

RULE_BITFIELD = "bitfield_accessor"
RULE_UNINIT = "uninit_local"
RULE_MACRO_CALL = "macro_call_syntax"
RULE_VARIADIC = "variadic_promotion"
RULE_REF_AS_INT = "ref_as_int"
RULE_COM_CALL = "com_method_call"
ALL_RULES = (
    RULE_BITFIELD,
    RULE_UNINIT,
    RULE_MACRO_CALL,
    RULE_VARIADIC,
    RULE_REF_AS_INT,
    RULE_COM_CALL,
)

#: `pub struct <Name>Vtbl {` — an interface rendered in the C `lpVtbl` idiom.
_VTBL_STRUCT = re.compile(r"^pub struct (\w+)Vtbl\s*\{", re.M)


def com_interfaces(*sources: str) -> frozenset[str]:
    """Interface names that carry an `lpVtbl`, derived from the crate's own vtable module.

    The COM rule is INERT without this: rewriting `x.M()` into a vtable dispatch is only
    correct for an interface in that shape, and a translated crate also has ordinary Rust
    structs whose methods rustc reports the same way.
    """
    names: set[str] = set()
    for text in sources:
        names.update(m.group(1) for m in _VTBL_STRUCT.finditer(text))
    return frozenset(names)


def _matching_open(data: bytes, close: int) -> int | None:
    """Index of the `(` matching the `)` at ``close``."""
    depth = 0
    for i in range(close, -1, -1):
        if data[i : i + 1] == b")":
            depth += 1
        elif data[i : i + 1] == b"(":
            depth -= 1
            if depth == 0:
                return i
    return None


@dataclass(frozen=True)
class Edit:
    file: str
    line: int
    start: int  # byte offset
    end: int
    replacement: str
    original: str
    rule: str
    authority: str


@dataclass(frozen=True)
class Refused:
    file: str
    line: int
    rule: str
    reason: str
    detail: str = ""


def _errors(diagnostics):
    for d in diagnostics:
        if d.get("level") == "error":
            yield d


def _primary(d):
    return next((s for s in (d.get("spans") or []) if s.get("is_primary")), None)


def _next_code_byte(data: bytes, pos: int) -> bytes:
    """The next byte that is not whitespace, or b'' at end of input."""
    while pos < len(data) and data[pos : pos + 1] in (b" ", b"\t", b"\n", b"\r"):
        pos += 1
    return data[pos : pos + 1]


def _line_bounds(data: bytes, offset: int) -> tuple[int, int]:
    start = data.rfind(b"\n", 0, offset) + 1
    end = data.find(b"\n", offset)
    return start, (len(data) if end < 0 else end)


def propose(diagnostics, read_file, *, rules=ALL_RULES, interfaces=frozenset()):
    """PURE: rustc diagnostics + a file reader -> (edits, refusals).

    ``interfaces`` is the set of COM interface names carrying an `lpVtbl` (see
    ``com_interfaces``); the COM rule does nothing without it.
    """
    edits: dict[tuple, Edit] = {}
    refused: list[Refused] = []
    cache: dict[str, bytes | None] = {}

    def load(name):
        if name not in cache:
            cache[name] = read_file(name)
        return cache[name]

    for d in _errors(diagnostics):
        code = (d.get("code") or {}).get("code") or ""
        msg = d.get("message") or ""
        span = _primary(d)
        if span is None:
            continue
        fname = span.get("file_name") or ""
        line = int(span.get("line_start") or 0)
        bs, be = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        data = load(fname)
        if data is None:
            continue

        def refuse(rule, reason, detail=""):
            refused.append(Refused(fname, line, rule, reason, detail))

        def add(rule, start, end, repl, authority):
            orig = data[start:end].decode("utf-8", "replace")
            edits.setdefault(
                (fname, start, end, rule),
                Edit(fname, line, start, end, repl, orig, rule, authority),
            )

        # --- bitfield accessors -------------------------------------------------------
        if code == "E0615" and RULE_BITFIELD in rules:
            m = re.search(r"take value of method `(\w+)`", msg)
            if not m:
                refuse(RULE_BITFIELD, "unrecognised_message", msg[:70])
                continue
            name = m.group(1)
            if data[bs:be].decode("utf-8", "replace") != name:
                refuse(RULE_BITFIELD, "span_is_not_the_field", data[bs:be].decode("utf-8", "replace"))
                continue
            tail = data[be:]
            assign = re.match(rb"[ \t]*=(?!=)", tail)
            if not assign:
                add(
                    RULE_BITFIELD, be, be, "()",
                    f"`{name}` is a bindgen bitfield ACCESSOR on this struct, so the read is a call",
                )
                continue
            # Assignment -> setter. The RHS must terminate on this line, or the argument we
            # would splice is a fragment. Refuse instead of corrupting the call.
            _, line_end = _line_bounds(data, be)
            rhs = data[be + assign.end() - 1 + 1 : line_end].decode("utf-8", "replace").strip()
            if not rhs.endswith(";"):
                refuse(RULE_BITFIELD, "multiline_rhs", f"`{name} = {rhs[:40]}...`")
                continue
            depth = rhs.count("(") - rhs.count(")")
            if depth != 0:
                refuse(RULE_BITFIELD, "unbalanced_rhs", f"`{name} = {rhs[:40]}`")
                continue
            add(
                RULE_BITFIELD, bs, line_end, f"set_{name}({rhs[:-1].strip()});",
                f"`{name}` is a bindgen bitfield field, so the write goes through `set_{name}`",
            )
            continue

        # --- COM method call written in C++ syntax -------------------------------------
        if code == "E0599" and RULE_COM_CALL in rules and interfaces:
            m = re.search(r"no method named `(\w+)` found for struct `([\w:]+)`", msg)
            if m and m.group(2).rsplit("::", 1)[-1] in interfaces:
                name, iface = m.group(1), m.group(2).rsplit("::", 1)[-1]
                if data[bs:be].decode("utf-8", "replace") != name:
                    refuse(RULE_COM_CALL, "span_is_not_the_method", name)
                    continue
                # The shape the translation emits is `(*RECV).Method(args)`. Anything else
                # (a chained call, a field access, a `&` receiver) is refused: guessing the
                # receiver here would splice a dispatch onto the wrong pointer.
                dot = bs - 1
                while dot > 0 and data[dot : dot + 1] in (b" ", b"\t", b"\n"):
                    dot -= 1
                if data[dot : dot + 1] != b".":
                    refuse(RULE_COM_CALL, "receiver_not_dotted", name)
                    continue
                close = dot - 1
                if data[close : close + 1] != b")":
                    refuse(RULE_COM_CALL, "receiver_not_parenthesised", name)
                    continue
                open_ = _matching_open(data, close)
                if open_ is None:
                    refuse(RULE_COM_CALL, "unbalanced_receiver", name)
                    continue
                inner = data[open_ + 1 : close].decode("utf-8", "replace").strip()
                if not inner.startswith("*"):
                    refuse(RULE_COM_CALL, "receiver_not_a_deref", inner[:40])
                    continue
                recv = inner[1:].strip()
                call = re.match(rb"[ \t\n]*\(", data[be:])
                if not call:
                    refuse(RULE_COM_CALL, "method_not_called", name)
                    continue
                arg_open = be + call.end() - 1
                nxt = _next_code_byte(data, arg_open + 1)
                if nxt == b")":
                    sep = ""
                else:
                    # A multi-line argument list starts on the NEXT line, so a trailing space
                    # after the comma is trailing whitespace — which `git diff --check` rejects
                    # and the repo's own hooks treat as a defect.
                    gap = data[arg_open + 1 :].split(nxt, 1)[0]
                    sep = "," if b"\n" in gap else ", "
                add(
                    RULE_COM_CALL,
                    open_,
                    arg_open + 1,
                    f"((*(*{recv}).lpVtbl).{name})({recv}{sep}",
                    f"`{iface}` is a COM interface in the `lpVtbl` shape, so `{name}` is a "
                    "vtable slot taking the interface pointer as its first argument, not a "
                    "Rust method",
                )
                continue

        # --- uninitialised locals -----------------------------------------------------
        if code == "E0381" and RULE_UNINIT in rules:
            m = re.search(r"binding `(\w+)`", msg)
            if not m:
                refuse(RULE_UNINIT, "unrecognised_message", msg[:70])
                continue
            name = m.group(1)
            decl = re.search(
                rf"^([ \t]*)let mut {re.escape(name)}: ([A-Za-z_][\w:<>, \[\]]*);$".encode(),
                data,
                re.M,
            )
            if not decl:
                refuse(RULE_UNINIT, "no_bare_declaration", name)
                continue
            ty = decl.group(2).decode()
            add(
                RULE_UNINIT, decl.start(), decl.end(),
                f"{decl.group(1).decode()}let mut {name}: {ty} = core::mem::zeroed();",
                "C declares this local and lets the callee fill it through a pointer; Rust "
                "requires an initialiser before the address is taken",
            )
            continue

        # --- macro called with fn syntax ----------------------------------------------
        if code in ("E0423", "E0425") and RULE_MACRO_CALL in rules:
            m = re.search(r"`(\w+)`", msg)
            if not m or "macro" not in msg:
                continue
            name = m.group(1)
            if data[bs:be].decode("utf-8", "replace") != name:
                refuse(RULE_MACRO_CALL, "span_is_not_the_name", name)
                continue
            if not re.match(rb"[ \t]*\(", data[be:]):
                refuse(RULE_MACRO_CALL, "not_a_call", name)
                continue
            add(
                RULE_MACRO_CALL, be, be, "!",
                f"`{name}` is a `macro_rules!`, so the call needs a `!`",
            )
            continue

        # --- variadic promotion --------------------------------------------------------
        if code == "E0617" and RULE_VARIADIC in rules:
            m = re.search(r"can't pass `(\w+)` to variadic function", msg)
            if not m:
                refuse(RULE_VARIADIC, "unrecognised_message", msg[:70])
                continue
            add(
                RULE_VARIADIC, be, be, " as ::core::ffi::c_int",
                f"C promotes a `{m.group(1)}` to `int` through `...`; Rust does not, so the "
                "promotion is written out",
            )
            continue

        # --- `&mut x as <int>` ---------------------------------------------------------
        if code == "E0606" and RULE_REF_AS_INT in rules:
            text = data[bs:be].decode("utf-8", "replace")
            m = re.match(r"&(mut\s+)?([A-Za-z_]\w*)\s+as\s+(.+)$", text.strip())
            if not m:
                refuse(RULE_REF_AS_INT, "unrecognised_shape", text[:60])
                continue
            of = "addr_of_mut!" if m.group(1) else "addr_of!"
            add(
                RULE_REF_AS_INT, bs, be,
                f"core::ptr::{of}({m.group(2)}) as {m.group(3)}",
                "a reference cannot be cast to an integer; the address is taken first, without "
                "creating a reference to the place",
            )
            continue

    ordered = sorted(edits.values(), key=lambda e: (e.file, e.start))
    return ordered, refused


def select_non_overlapping(edits):
    kept, dropped = [], []
    by_file: dict[str, list[Edit]] = {}
    for e in edits:
        by_file.setdefault(e.file, []).append(e)
    for f, group in sorted(by_file.items()):
        last = None
        for e in sorted(group, key=lambda e: (e.start, -e.end)):
            if last is not None and e.start < last:
                dropped.append(Refused(f, e.line, e.rule, "overlapping_span", e.original[:40]))
                continue
            kept.append(e)
            last = e.end
    return kept, dropped


def apply(edits, root: Path):
    by_file: dict[str, list[Edit]] = {}
    for e in edits:
        by_file.setdefault(e.file, []).append(e)
    applied = 0
    for f, group in by_file.items():
        p = root / f
        data = p.read_bytes()
        for e in sorted(group, key=lambda e: -e.start):
            if data[e.start : e.end].decode("utf-8", "replace") != e.original:
                continue
            data = data[: e.start] + e.replacement.encode() + data[e.end :]
            applied += 1
        p.write_bytes(data)
    return applied


def cargo_check(crate: str, target: str, root: Path):
    out = subprocess.run(
        ["cargo", "check", "-p", crate, "--target", target, "--message-format=json"],
        cwd=root, capture_output=True, text=True,
    )
    diags = []
    for line in out.stdout.splitlines():
        try:
            m = json.loads(line).get("message")
        except json.JSONDecodeError:
            continue
        if m:
            diags.append(m)
    return diags


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("crate")
    ap.add_argument("--target", default="i686-pc-windows-gnu")
    ap.add_argument("--root", type=Path, default=Path("sdk/lib/rust"))
    ap.add_argument("--passes", type=int, default=6)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument(
        "--vtable-module",
        action="append",
        default=[],
        help="path (relative to --root) of a module defining `<Iface>Vtbl` structs; enables "
        "the COM rule",
    )
    args = ap.parse_args()
    root = args.root.resolve()

    sources = []
    for rel in args.vtable_module:
        p = root / rel
        if not p.is_file():
            print(f"error: --vtable-module {rel} not found under {root}", file=sys.stderr)
            return 2
        sources.append(p.read_text(errors="replace"))
    ifaces = com_interfaces(*sources)
    if args.vtable_module:
        print(f"COM rule armed: {len(ifaces)} interfaces from {len(sources)} module(s)")

    for i in range(1, args.passes + 1):
        diags = cargo_check(args.crate, args.target, root)
        n_err = sum(1 for _ in _errors(diags))
        edits, refused = propose(
            diags,
            lambda f: (root / f).read_bytes() if (root / f).is_file() else None,
            interfaces=ifaces,
        )
        edits, dropped = select_non_overlapping(edits)
        refused += dropped
        print(f"pass {i}: {n_err} errors, {len(edits)} edits, {len(refused)} refused")
        for r in refused[:10]:
            print(f"    ! {r.rule}/{r.reason} {r.file}:{r.line} {r.detail}")
        if not edits or args.dry_run:
            break
        apply(edits, root)
    diags = cargo_check(args.crate, args.target, root)
    print(f"final: {sum(1 for _ in _errors(diags))} errors")
    return 0


if __name__ == "__main__":
    sys.exit(main())
