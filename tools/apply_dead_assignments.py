#!/usr/bin/env python3
"""
apply_dead_assignments.py — remove the `unused_assignments` sites the C proves harmless.

INPUT is `rank_unused_assignments.py --json`. Only rows whose verdict says the C has the
same dead write (C_DEAD) or no initializer at all (C_NO_INIT) are touched; everything the
ranker could not clear is left for a human.

WHAT IT DOES, per shape:

  DEAD_INIT   `let mut x: T = <pure>;` -> `let mut x: T;`. rustc's definite-initialization
              check then proves it -- `x`, `&x` and `&mut x` before a write are E0381 -- on
              every arch that still compiles, EXCEPT through a raw borrow: `addr_of_mut!(x)`
              / `&raw mut x` of an uninitialized local COMPILES, and the callee it is handed
              to may read the initial value (an in/out size like samsrv's `&raw mut Length`).
              So a variable raw-borrowed before it is written keeps its initializer (SKIP).
              "Written" means a plain `x = ..;` statement in the `let`'s own block ahead
              of every raw borrow: straight-line code there runs before anything after it
              on every path, so no borrow can see the initial value. A write inside a
              branch or loop does not count (acpi's `old_status`, scsiport's `UlongData`
              were removable by this rule and refused by the older fn-wide one).
              A site that no longer compiles goes back by hand (or `--skip`).
  DEAD_STORE  `x = <pure>;` -> deleted.
  LOST_CALL   `x = f(..);` -> `f(..);` (the call stays; only the dead store goes, along with
              any cast of the result). A pure-looking DEAD_STORE whose rhs is a call is
              treated the same way.

DEAD_STORE / LOST_CALL are NOT proven by the compiler: rustc's liveness cannot see a read
through a raw pointer that already exists (cdfs's SEH cleanup flags are read as
`*p_UnlockVcb` from a finally block; cdrom's `fourByte = &allocationLength as *mut _` is
read after the write it calls dead). So a store is only removed when no pointer to the
variable can be live at it: none taken earlier in the fn, none anywhere in a loop around
it. A borrow AFTER the write needs no guard -- rustc counts it as the read. The rest are
SKIP for a human, never guessed; in practice they are where to look for aliasing UB (a
direct write to a local while a raw pointer to it is live).

Only PURE right-hand sides are removed -- literals, paths, casts/negations of those,
`null[_mut]()`, `zeroed()`, struct/array/tuple literals of pure parts. A call's side effect
is never dropped.

Usage:
    python tools/rank_unused_assignments.py <full.log x3> --json > rank.json
    python tools/apply_dead_assignments.py rank.json [--dry-run] [--skip path:line ...]
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from collections.abc import Callable
from itertools import pairwise
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
from rank_unused_assignments import parser, text

ROOT = Path(__file__).resolve().parent.parent
RUST = ROOT / "sdk" / "lib" / "rust"
HARMLESS = {"C_DEAD", "C_NO_INIT"}
PURE_CALLS = re.compile(r"(^|::)(null|null_mut|zeroed|uninit)$")
LOOPS = {"loop_expression", "while_expression", "for_expression"}
PURE_METHODS = {
    "add",
    "sub",
    "offset",
    "byte_add",
    "byte_sub",
    "wrapping_add",
    "wrapping_sub",
    "wrapping_offset",
    "cast",
    "cast_mut",
    "cast_const",
}


def pure(n: Any) -> bool:
    t = n.type
    if t in {
        "integer_literal",
        "float_literal",
        "boolean_literal",
        "char_literal",
        "string_literal",
        "identifier",
        "scoped_identifier",
        "field_identifier",
        "unit_expression",
    }:
        return True
    if t in {
        "parenthesized_expression",
        "unary_expression",
        "type_cast_expression",
        "binary_expression",
    }:
        # `-x` / `!x` / `(x)` / `x as T` / `a + b`: the operands decide (`*p` is a deref -- a
        # read, but a read has no side effect, so it is still safe to drop).
        return all(pure(c) for c in n.named_children if c.type not in _TYPE_NODES)
    if t == "field_expression":
        return pure(n.child_by_field_name("value"))
    if t == "call_expression":
        fn = n.child_by_field_name("function")
        args = n.child_by_field_name("arguments")
        argl = [] if args is None else args.named_children
        if fn is not None and fn.type == "field_expression":
            # `p.wrapping_add(n)` / `(p as usize).add(..)`: pointer arithmetic, no effect.
            meth = fn.child_by_field_name("field")
            recv = fn.child_by_field_name("value")
            return (
                meth is not None
                and text(meth) in PURE_METHODS
                and recv is not None
                and pure(recv)
                and all(pure(a) for a in argl)
            )
        return (
            fn is not None
            and PURE_CALLS.search(re.sub(r"::<.*>$", "", text(fn))) is not None
            and not argl
        )
    if t in {"generic_function"}:
        return False
    if t in {"array_expression", "tuple_expression"}:
        return all(pure(c) for c in n.named_children)
    if t == "struct_expression":
        body = n.child_by_field_name("body")
        return body is not None and all(pure(c) for c in body.named_children)
    if t == "field_initializer":
        v = n.child_by_field_name("value")
        return v is not None and pure(v)
    if t == "shorthand_field_initializer":
        return True
    if t == "base_field_initializer":
        return all(pure(c) for c in n.named_children)
    if t == "unsafe_block":
        blk = n.named_children[-1] if n.named_children else None
        return blk is not None and blk.type == "block" and _pure_block(blk)
    if t == "block":
        return _pure_block(n)
    return False


_TYPE_NODES = {
    "primitive_type",
    "type_identifier",
    "scoped_type_identifier",
    "pointer_type",
    "generic_type",
    "reference_type",
    "array_type",
    "tuple_type",
}


def _pure_block(b: Any) -> bool:
    kids = [c for c in b.named_children if c.type not in {"line_comment", "block_comment"}]
    return len(kids) == 1 and pure(kids[0])


def _spans(n: Any, line: int) -> bool:
    return n.start_point[0] <= line - 1 <= n.end_point[0]


def _reparse(src: bytes, tt: Any) -> Any | None:
    """A macro's token tree as Rust, PADDED so byte offsets and lines are the file's own:
    `stdmethod! { fn f(..) { .. } }` hides a whole COM method from tree-sitter. None when
    the body is not plain Rust (a `macro_rules!` body with `$x` metavariables)."""
    s, e = tt.start_byte + 1, tt.end_byte - 1
    pad = bytes(b if b == 0x0A else 0x20 for b in src[:s])
    root = parser("rust").parse(pad + src[s:e]).root_node
    return None if root.has_error else root


def _find(src: bytes, root: Any, line: int, ok: Callable[[Any], bool]) -> list[Any]:
    """Nodes spanning `line` that satisfy `ok`, looking inside macro token trees too."""
    out: list[Any] = []

    def walk(n: Any) -> None:
        if not _spans(n, line):
            return
        if ok(n):
            out.append(n)
        if n.type == "macro_invocation":
            tt = next((c for c in n.children if c.type == "token_tree"), None)
            sub = _reparse(src, tt) if tt is not None else None
            if sub is not None:
                walk(sub)
            return
        if n.type == "macro_definition":
            return
        for c in n.children:
            walk(c)

    walk(root)
    return out


def _enclosing_fn(n: Any) -> Any:
    p = n.parent
    while p is not None and p.type != "function_item":
        p = p.parent
    return p


def _lhs_is(lhs: Any, var: str) -> bool:
    """`x = ..` or a field store into it, `x.a.b = ..` (rustc names the local)."""
    while lhs.type == "field_expression":
        lhs = lhs.child_by_field_name("value")
    return lhs.type == "identifier" and text(lhs) == var


def _call_of(n: Any) -> Any | None:
    """The call a store's rhs IS, through parens and an `as` cast: `(F)(..) as u8`."""
    while n.type in {"parenthesized_expression", "type_cast_expression"}:
        n = n.named_children[0]
    return n if n.type == "call_expression" else None


def _borrows(var: str) -> re.Pattern[bytes]:
    v = re.escape(var).encode()
    return re.compile(
        rb"&\s*(mut\s+|raw\s+(const|mut)\s+)?" + v + rb"\b(?!\s*[(\[])"
        rb"|addr_of(_mut)?!\s*\(\s*" + v + rb"\b"
    )


def _raw_borrows(var: str) -> re.Pattern[bytes]:
    v = re.escape(var).encode()
    return re.compile(rb"&raw\s+(const|mut)\s+" + v + rb"\b|addr_of(_mut)?!\s*\(\s*" + v + rb"\b")


def address_taken(fn_src: str, var: str) -> bool:
    """Does `fn_src` take `var`'s address in any spelling?"""
    return _borrows(var).search(fn_src.encode()) is not None


def raw_borrow_sees_init(fn: Any, let: Any, var: str) -> bool:
    """Could a raw borrow of `var` observe the value its `let` initialized it with? Not if
    a plain `var = ..;` statement in the `let`'s own block precedes every raw borrow in the
    fn: that statement dominates everything after it."""
    borrows = [fn.start_byte + m.start() for m in _raw_borrows(var).finditer(fn.text)]
    if not borrows:
        return False
    first = min(borrows)
    block = let.parent
    for st in block.named_children if block is not None else []:
        if st.start_byte <= let.end_byte:
            continue
        if st.start_byte >= first:
            break
        e = (
            st.named_children[0]
            if st.type == "expression_statement" and st.named_children
            else None
        )
        lhs = (
            e.child_by_field_name("left")
            if e is not None and e.type == "assignment_expression"
            else None
        )
        if lhs is not None and text(lhs) == var:
            return False
    return True


def pointer_live_at(fn: Any, write: Any, var: str) -> bool:
    """Can a raw pointer to `var` exist when `write` runs? Only if one was taken BEFORE it
    in the source, or anywhere in a loop around it (taken on an earlier iteration). A
    borrow after the write is itself a read rustc counts -- it would not have warned."""
    starts = [fn.start_byte + m.start() for m in _borrows(var).finditer(fn.text)]
    if any(s < write.start_byte for s in starts):
        return True
    p = write.parent
    while p is not None and p.id != fn.id:
        if p.type in LOOPS and any(p.start_byte <= s < p.end_byte for s in starts):
            return True
        p = p.parent
    return False


def plan_file(path: Path, rows: list[dict]) -> tuple[list[tuple[int, int, bytes, dict]], list[str]]:
    src = path.read_bytes()
    root = parser("rust").parse(src).root_node
    edits: list[tuple[int, int, bytes, dict]] = []
    skips: list[str] = []

    def skip(r: dict, why: str) -> None:
        skips.append(f"SKIP {r['path']}:{r['line']} `{r['var']}` {r['shape']}: {why}")

    for r in rows:
        var, line, shape = r["var"], r["line"], r["shape"]
        if shape == "DEAD_INIT":

            def is_let(n: Any, var: str = var, line: int = line) -> bool:
                p = n.child_by_field_name("pattern") if n.type == "let_declaration" else None
                v = n.child_by_field_name("value") if p is not None else None
                return (
                    p is not None
                    and re.fullmatch(rf"(mut\s+)?{re.escape(var)}", text(p)) is not None
                    # rustc points into the VALUE; a `let` whose value is a block holding
                    # an inner `let` of the same name must not match the inner line.
                    and (
                        n.start_point[0] == line - 1
                        or (v is not None and v.start_point[0] == line - 1)
                    )
                )

            lets = _find(src, root, line, is_let)
            if len(lets) != 1:
                skip(r, f"{len(lets)} matching `let`s at the line")
                continue
            n = lets[0]
            val = n.child_by_field_name("value")
            if val is None or n.child_by_field_name("alternative") is not None:
                skip(r, "no initializer / let-else")
                continue
            if not pure(val):
                skip(r, f"initializer not pure: {text(val)[:60]}")
                continue
            fn = _enclosing_fn(n)
            if fn is not None and raw_borrow_sees_init(fn, n, var):
                skip(
                    r, "raw-borrowed before any write -- `addr_of_mut!` of an uninit local compiles"
                )
                continue
            # Cut from the end of the pattern/type to the end of the value: ` = <init>`.
            ty = n.child_by_field_name("type")
            pat = n.child_by_field_name("pattern")
            assert pat is not None
            edits.append(((ty or pat).end_byte, val.end_byte, b"", r))
        elif shape in {"DEAD_STORE", "LOST_CALL"}:

            def is_store(n: Any, var: str = var, line: int = line) -> bool:
                if n.type != "expression_statement" or not n.named_child_count:
                    return False
                a = n.named_children[0]
                lhs = a.child_by_field_name("left") if a.type == "assignment_expression" else None
                return lhs is not None and a.start_point[0] == line - 1 and _lhs_is(lhs, var)

            stmts = _find(src, root, line, is_store)
            if len(stmts) != 1:
                skip(r, f"{len(stmts)} matching assignments at the line")
                continue
            st = stmts[0]
            fn = _enclosing_fn(st)
            if fn is None:
                skip(r, "no enclosing fn")
                continue
            if pointer_live_at(fn, st, var):
                skip(r, "a pointer to it may be live -- rustc cannot see a read through it")
                continue
            a = st.named_children[0]
            rhs = a.child_by_field_name("right")
            assert rhs is not None
            call = _call_of(rhs)
            if pure(rhs):
                s, e = st.start_byte, st.end_byte
                # Take the whole line when the statement is alone on it.
                ls = src.rfind(b"\n", 0, s) + 1
                le = src.find(b"\n", e)
                le = len(src) if le < 0 else le
                if not src[ls:s].strip() and not src[e:le].strip():
                    s, e = ls, min(le + 1, len(src))
                edits.append((s, e, b"", r))
            elif call is not None:
                # Keep the call, drop the store (and any cast of its result).
                edits.append((a.start_byte, a.end_byte, src[call.start_byte : call.end_byte], r))
            else:
                skip(r, f"rhs neither pure nor a call: {text(rhs)[:60]}")
        else:
            skip(r, f"shape {shape} is done by hand")
    return edits, skips


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("rank_json", type=Path)
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--skip", nargs="*", default=[], help="path:line sites to leave alone")
    args = ap.parse_args(argv)

    rows = [r for r in json.loads(args.rank_json.read_text()) if r["verdict"] in HARMLESS]
    skipset = set(args.skip)
    by_file: dict[str, list[dict]] = defaultdict(list)
    for r in rows:
        if f"{r['path']}:{r['line']}" not in skipset:
            by_file[r["path"]].append(r)

    applied = 0
    skips: list[str] = []
    for rel, rs in sorted(by_file.items()):
        path = RUST / rel
        edits, sk = plan_file(path, rs)
        skips += sk
        # Overlapping edits would mean two rows matched the same node.
        edits.sort(key=lambda e: e[0])
        for a, b in pairwise(edits):
            if a[1] > b[0]:
                print(f"OVERLAP in {rel}: {a[3]['line']} / {b[3]['line']}", file=sys.stderr)
                return 1
        src = path.read_bytes()
        for s, e, rep, r in reversed(edits):
            if args.dry_run:
                print(f"{rel}:{r['line']} {r['shape']}: -{src[s:e].decode()!r}")
            src = src[:s] + rep + src[e:]
        if not args.dry_run and edits:
            path.write_bytes(src)
        applied += len(edits)

    for s in skips:
        print(s)
    verb = "would apply" if args.dry_run else "applied"
    print(f"{verb} {applied}, skipped {len(skips)} of {len(rows)} harmless sites")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
