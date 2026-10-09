#!/usr/bin/env python3
"""
rank_unused_assignments.py — rank rustc's `unused_assignments` sites by whether the C READ them.

THE QUESTION. rustc's `value assigned to `x` is never read` is a fact about the Rust only.
Most of the ~1,000 sites in a translated tree are the C's own habits carried over
faithfully (`NTSTATUS Status = STATUS_SUCCESS;` then assigned on every path), and they are
noise. A few are the TRACE of a translation bug: the C read the value -- an
`if (!NT_SUCCESS(Status))`, a `goto Exit` with the error code, a `return Status` on a path
the Rust restructured -- and the Rust lost that read. The warning cannot tell the two apart;
the C can. So this tool asks the C the same question rustc asked the Rust.

HOW. For each site it finds the enclosing Rust fn (tree-sitter-rust), the C function of
the same name (tree-sitter-c / -cpp, outside `sdk/lib/rust/`), the C write that
corresponds to the Rust one -- same callee, same literal, else the same ordinal among the
variable's writes -- and runs a structured backward liveness over the C function from
that write: if/else, loops (to a fixed point, with `break`/`continue`), `switch`
fall-through, `goto` to its label, `#if` blocks (both arms possible). Verdicts:

  C_READS     the C reads the value on some path; the Rust lost that read. RANKED FIRST:
              this is the dropped error check / lost output.
  C_DEAD      the C has the same dead store. A faithful upstream dead store: harmless.
  C_NO_INIT   (dead initializers) the C declares the variable WITHOUT a value; the Rust
              added one for definite-initialization. Harmless.
  NO_MATCH    the C function exists but no write lines up with this one. Read by hand.
  AMBIGUOUS   several same-named C functions disagree. Read by hand.
  NO_C_FN     no C function of that name (Rust-only helper, C++ method spelled differently,
              a vendored Wine name...). Read by hand.

The liveness is LEXICAL-STRUCTURAL, not a compiler's: macros are opaque calls (a macro
argument naming the variable is a read), an unparseable region that names the variable is
a read, `&x` is a read. Every approximation errs toward C_READS -- a false C_READS costs a
reading, a false C_DEAD would bury a bug.

Within a verdict, sites rank by shape (a call's result overwritten > a dead store > a dead
accumulation > a dead initializer), then a status-like variable name, then arch spread.

Usage:
    python tools/rank_unused_assignments.py /tmp/vibeos-cli/<ts>-build-{i386,amd64,arm64}/full.log
        [--verdict C_READS] [--crate afd-rust] [--json] [--limit N]

Use logs from `vibeos-cli build --clean`: a warm log only carries recompiled crates.
Line numbers are re-located by the warning's snippet when the tree moved since the build;
a site whose snippet cannot be found is reported STALE, never guessed.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import defaultdict
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
import rust_warning_census as census

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
LINT = "unused_assignments"

VERDICT_ORDER = [
    "C_READS",
    "C_READS_ADDR",
    "NO_MATCH",
    "C_READS_DEBUG",
    "AMBIGUOUS",
    "NO_C_FN",
    "STALE",
    "C_DEAD",
    "C_NO_INIT",
]
SHAPE_ORDER = ["LOST_CALL", "DEAD_STORE", "DEAD_COMPOUND", "PARAM", "DEAD_INIT"]
STATUS_LIKE = re.compile(
    r"^(n?t?status|hr|hres|hresult|ret|retval|rc|result|res|err|error|success|bret|bresult|ok)$"
)

# `UNREFERENCED_PARAMETER(Status)` names a variable to SILENCE the C compiler's unused
# warning: it is the opposite of a read.
UNREF_MACROS = frozenset(
    {
        "UNREFERENCED_PARAMETER",
        "UNREFERENCED_LOCAL_VARIABLE",
        "DBG_UNREFERENCED_PARAMETER",
        "DBG_UNREFERENCED_LOCAL_VARIABLE",
        "UNUSED",
        "UNUSED_PARAM",
        "UNUSED_VARIABLE",
    }
)
# A read that exists only to be printed or asserted. The C read is real on a checked build,
# but a Rust that lost it lost a diagnostic, not behaviour -- ranked below C_READS.
DEBUG_MACROS = frozenset(
    {
        "DPRINT",
        "DPRINT1",
        "DPRINTT",
        "DPRINT_ERR",
        "DbgPrint",
        "DbgPrintEx",
        "KdPrint",
        "KdPrintEx",
        "ASSERT",
        "ASSERTMSG",
        "NT_ASSERT",
        "NT_VERIFY",
        "assert",
        "ok",
        "TRACE",
        "WARN",
        "ERR",
        "FIXME",
        "TRACE_",
        "WARN_",
        "ERR_",
        "TraceLog",
        "TracePrint",
        "TraceEvents",
        "DebugPrint",
        "DBGPRINT",
        "DBGTRACE",
        "DPRINTF",
        "FsRtlDbgPrint",
        "ACPI_DEBUG_PRINT",
        "dprintf",
        "STUB",
        "UNIMPLEMENTED",
        "UNIMPLEMENTED_DBGBREAK",
        "DBG_PRINT",
        "TRACE_ON",
        "LOG",
        "ERROR_DBGBREAK",
        "VideoDebugPrint",
        "VideoPortDebugPrint",
    }
)

# Iteration macros that ASSIGN their first argument (the cursor) on every pass.
FOREACH_MACRO = re.compile(r"^(?i:list_for_each\w*|for_each\w*|LIST_FOR_EACH\w*|FOR_EACH\w*)$")

# Debug output by naming convention, beyond the list: `AFD_DbgPrint`, `CdromDbgPrint`, `INFO`.
DEBUG_RE = re.compile(
    r"(?i)(dbg_?print|debug_?print|dprint|^trace|^info$|^warn$|^err$|^fixme$|assert)"
)


def is_debug(callee: str) -> bool:
    return callee in DEBUG_MACROS or bool(DEBUG_RE.search(callee))


Node = Any  # tree_sitter.Node; kept loose so the module imports without tree-sitter

# --------------------------------------------------------------------------- parsers

_PARSERS: dict[str, Any] = {}


def parser(lang: str) -> Any:
    if lang not in _PARSERS:
        from tree_sitter import Language, Parser

        if lang == "rust":
            import tree_sitter_rust as m
        elif lang == "c":
            import tree_sitter_c as m
        else:
            import tree_sitter_cpp as m
        _PARSERS[lang] = Parser(Language(m.language()))
    return _PARSERS[lang]


def text(n: Node) -> str:
    return n.text.decode("utf8", "replace")


def canon(name: str) -> str:
    """Spelling-insensitive key: `Status` == `status`, `list_count_index` == `ListCountIndex`."""
    return name.replace("_", "").lower()


def npath(expr: str) -> str:
    """`(*p).a.b` / `p->a.b` / `p.a.b` -> `p.a.b`, spelling kept EXACT. Comparing case- or
    underscore-insensitively here made the type `FEATURE_HEADER` (in a `sizeof`) a read of
    the variable `featureHeader`; the insensitive match is only for resolving a Rust name
    to its C spelling (`resolve_c_path`), once per function."""
    s = re.sub(r"\s+", "", expr)
    s = s.replace("->", ".")
    s = re.sub(r"[()*&]", "", s)
    return ".".join(p for p in s.split(".") if p)


def canon_path(expr: str) -> str:
    """`(*p).a.b` / `p->a.b` / `p.a.b` -> `p.a.b` (canonicalized per component)."""
    s = re.sub(r"\s+", "", expr)
    s = s.replace("->", ".")
    s = re.sub(r"[()*&]", "", s)
    return ".".join(canon(p) for p in s.split(".") if p)


# --------------------------------------------------------------------------- RHS keys

_INT = re.compile(r"^-?(0x[0-9a-fA-F]+|\d+)")
_ZERO_CONSTS = {"STATUS_SUCCESS", "S_OK", "NO_ERROR", "ERROR_SUCCESS", "NOERROR", "AE_OK"}
_NULLISH = {"null", "nullmut", "null_mut", "NULL", "nullptr"}


def _strip_parens(s: str) -> str:
    """Drop parentheses that wrap the WHOLE expression, and only those."""
    s = s.strip()
    while s.startswith("(") and s.endswith(")"):
        depth = 0
        for i, ch in enumerate(s):
            depth += ch == "("
            depth -= ch == ")"
            if depth == 0 and i != len(s) - 1:
                return s
        s = s[1:-1].strip()
    return s


def rhs_key(expr: str) -> str:
    """A spelling-insensitive key for what a write stores: the callee of a call, a
    constant's name, an integer's value. Two writes with equal keys are the same write."""
    s = expr.strip().rstrip(";").strip()
    # Rust casts / C casts
    s = re.sub(r"\s+as\s+[\w:<>*\s]+$", "", s)
    s = re.sub(r"^\(\s*[A-Z_][\w\s*]*\)\s*", "", s)
    s = _strip_parens(s)
    if re.fullmatch(r"(core::|std::)?(ptr::)?null(_mut)?(::<[^>]*>)?\(\)", s) or s in _NULLISH:
        return "0"
    if re.fullmatch(r"\{\s*0?\s*\}", s):
        return "0"  # C `= {0}`
    if re.fullmatch(
        r"(core::|std::)?(mem::)?zeroed(::<[^>]*>)?\(\)|(\w+::)*[Dd]efault(::default)?\(\)", s
    ):
        return "0"  # an all-zero default: what the C spells `= {0}` / `= 0` / nothing
    if s.split("::")[-1] in _ZERO_CONSTS:
        return "0"  # `= STATUS_SUCCESS` and `= 0 // STATUS_SUCCESS` are one write
    if s in ("TRUE", "true"):
        return "1"
    if s in ("FALSE", "false"):
        return "0"
    m = _INT.match(s)
    if m and re.fullmatch(r"-?(0x[0-9a-fA-F]+|\d+)[a-zA-Z0-9]*", s):
        v = m.group(0)
        try:
            return str(int(v, 16) if "0x" in v.lower() else int(v))
        except ValueError:
            return v
    m = re.match(r"^([\w:]+?)(?:::<[^>]*>)?\s*\(", s)
    if m:
        return "call:" + canon(m.group(1).split("::")[-1])
    m = re.match(r"^.*\.(\w+)\s*\([^()]*\)\s*$", s)  # method call: x.foo(..)
    if m:
        if re.fullmatch(r"add|sub|offset|byte_add|byte_offset|wrapping_\w+", m.group(1)):
            return "expr"  # pointer arithmetic, the C's `p + n`
        return "call:" + canon(m.group(1))
    if re.fullmatch(r"[\w:]+", s):
        return "id:" + canon(s.split("::")[-1])
    return "expr"


# --------------------------------------------------------------------------- Rust side


@dataclass
class RustWrite:
    shape: str
    lhs: str  # canonical lvalue path
    key: str  # rhs_key
    op: str  # "=", "+=", "init", "param"
    ordinal: int  # index among this lvalue's writes in the fn, source order
    nwrites: int
    key_ordinal: int = 0  # index among the writes storing the same key
    key_count: int = 1


def _rust_fn(node: Node) -> Node | None:
    while node is not None and node.type != "function_item":
        node = node.parent
    return node


def _rust_writes(fn: Node, lhs: str) -> list[Node]:
    out = []

    def walk(n: Node) -> None:
        if n.type == "let_declaration":
            pat = n.child_by_field_name("pattern")
            named = pat is not None and npath(re.sub(r"^mut\s+", "", text(pat))) == lhs
            if named and n.child_by_field_name("value") is not None:
                out.append(n)
        elif n.type in ("assignment_expression", "compound_assignment_expr"):
            left = n.child_by_field_name("left")
            if left is not None and npath(text(left)) == lhs:
                out.append(n)
        elif n.type == "parameter":
            pat = n.child_by_field_name("pattern")
            if pat is not None and npath(re.sub(r"^mut\s+", "", text(pat))) == lhs:
                out.append(n)
        for c in n.children:
            if c.type not in ("function_item", "closure_expression"):
                walk(c)

    walk(fn)
    return sorted(out, key=lambda n: n.start_byte)


_WRITE_NODES = ("let_declaration", "assignment_expression", "compound_assignment_expr", "parameter")


def _write_at(root: Node, pt: tuple[int, int]) -> tuple[Node | None, Node | None]:
    """(the write node enclosing `pt`, the outermost macro body on the way, if any)."""
    n = root.descendant_for_point_range(pt, pt)
    macro_body = None
    while n is not None and n.type not in _WRITE_NODES:
        if n.type == "token_tree" and n.parent is not None and n.parent.type == "macro_invocation":
            macro_body = n
        n = n.parent
    return n, macro_body


def rust_site(src: bytes, line: int, col: int) -> tuple[str, RustWrite] | str:
    """(fn name, the write) for the warning at 1-based line:col, or an error string."""
    pt = (line - 1, max(col - 1, 0))
    n, macro_body = _write_at(parser("rust").parse(src).root_node, pt)
    if n is None and macro_body is not None:
        # A fn defined inside a macro call (`com_vtbl! { fn .. }`) is one token tree to the
        # parser. Blank everything outside the macro's braces -- newlines kept, so every
        # point stays put -- and parse the body as items.
        lo, hi = macro_body.start_byte + 1, macro_body.end_byte - 1
        inner = bytes(b if (lo <= i < hi or b == 0x0A) else 0x20 for i, b in enumerate(src))
        n, _ = _write_at(parser("rust").parse(inner).root_node, pt)
    if n is None:
        return "no assignment at site"
    fn = _rust_fn(n)
    if fn is None or fn.child_by_field_name("name") is None:
        return "no enclosing fn"
    name = text(fn.child_by_field_name("name"))
    if n.type == "let_declaration":
        pat, val = n.child_by_field_name("pattern"), n.child_by_field_name("value")
        if pat is None or val is None:
            return "let without value"
        lhs, op, key = npath(re.sub(r"^mut\s+", "", text(pat))), "init", rhs_key(text(val))
        # `let mut status = RtlCreateAcl(..)` is a call's result dropped, not a C-style default
        shape = "LOST_CALL" if key.startswith("call:") else "DEAD_INIT"
    elif n.type == "parameter":
        pat = n.child_by_field_name("pattern")
        lhs, op, key, shape = (
            npath(re.sub(r"^mut\s+", "", text(pat))),
            "param",
            "param",
            "PARAM",
        )
    else:
        left, right = n.child_by_field_name("left"), n.child_by_field_name("right")
        lhs = npath(text(left))
        op = "=" if n.type == "assignment_expression" else text(n.child_by_field_name("operator"))
        key = rhs_key(text(right))
        if op != "=":
            shape, key = "DEAD_COMPOUND", op + key
        elif key.startswith("call:") and not re.search(
            r"\.(add|sub|offset|wrapping_\w+)\s*\(", text(right)
        ):
            shape = "LOST_CALL"
        else:
            shape = "DEAD_STORE"
    writes = _rust_writes(fn, lhs)
    ordinal = next((i for i, w in enumerate(writes) if w.start_byte == n.start_byte), -1)
    same = [w for w in writes if _rust_key(w) == key]
    k_ord = next((i for i, w in enumerate(same) if w.start_byte == n.start_byte), 0)
    return name, RustWrite(shape, lhs, key, op, ordinal, len(writes), k_ord, max(len(same), 1))


def _rust_key(n: Node) -> str:
    if n.type == "let_declaration":
        v = n.child_by_field_name("value")
        return rhs_key(text(v)) if v is not None else ""
    if n.type == "parameter":
        return "param"
    right = n.child_by_field_name("right")
    k = rhs_key(text(right)) if right is not None else ""
    if n.type == "compound_assignment_expr":
        return text(n.child_by_field_name("operator")) + k
    return k


# --------------------------------------------------------------------------- C side: liveness


class CLive:
    """Backward may-liveness of one lvalue path inside one C function body.

    `live_in(stmt, out)` answers: entering `stmt` with `out` the liveness after it, may the
    current value be read before it is overwritten? `after(node)` gives the liveness at the
    point just past `node`, climbing the tree for its continuation."""

    LOOPS = ("while_statement", "for_statement", "do_statement", "for_range_loop")

    def __init__(
        self, fn: Node, path: str, ignore_debug: bool = False, ignore_addr: bool = False
    ) -> None:
        self.fn = fn
        self.ignore_debug = ignore_debug
        self.ignore_addr = ignore_addr
        self.path = path
        self.base = path.split(".")[0]
        self.labels: dict[str, Node] = {}
        self._collect_labels(fn)
        self._after: dict[int, bool] = {}
        self._label_live: dict[str, bool] = {}

    def _collect_labels(self, n: Node) -> None:
        if n.type == "labeled_statement":
            lab = n.child_by_field_name("label")
            if lab is not None:
                self.labels[text(lab)] = n
        for c in n.children:
            self._collect_labels(c)

    # -- expressions

    def _is_target(self, n: Node) -> bool:
        return npath(text(n)) == self.path

    def _lhs_write(self, n: Node) -> bool:
        """`n` is a plain `=` assignment (or declarator) whose target is our path."""
        if n.type == "assignment_expression":
            op = n.child_by_field_name("operator")
            left = n.child_by_field_name("left")
            return op is not None and text(op) == "=" and left is not None and self._is_target(left)
        return False

    def _callee(self, n: Node) -> str:
        f = n.child_by_field_name("function")
        return text(f).split("::")[-1].split(".")[-1].split("->")[-1] if f is not None else ""

    def _hit(self, n: Node) -> bool:
        """`n` itself reads the path (not counting its children)."""
        if n.type not in (
            "identifier",
            "field_expression",
            "pointer_expression",
            "subscript_expression",
        ):
            return False
        t = npath(text(n))
        if t == self.path or t.startswith(self.path + "."):
            return True
        if n.type == "identifier" and text(n) == self.base and self.base != self.path:
            # the base used whole (passed, copied, address taken) reads every field of it --
            # unless it heads a field chain, which _scan judges at the chain.
            p = n.parent
            return not (
                p is not None
                and p.type == "field_expression"
                and p.child_by_field_name("argument") == n
            )
        return False

    def _scan(self, n: Node | None) -> str | None:
        """Evaluate `n` in order: 'R' if the path is read before any kill, 'K' if it is
        overwritten first, None if neither. A kill on a conditionally-evaluated operand
        (`&&`, `||`, `?:`) is not a kill; a read there still is a read."""
        if n is None:
            return None
        t = n.type
        if self._lhs_write(n):
            r = self._scan(n.child_by_field_name("right"))
            return r or "K"
        if t == "call_expression":
            callee = self._callee(n)
            if callee in UNREF_MACROS:
                return None
            if self.ignore_debug and is_debug(callee):
                return None
            if FOREACH_MACRO.match(callee):
                # `list_for_each(entry, &head)` assigns its cursor: the first argument is a
                # write, the rest are reads
                args = n.child_by_field_name("arguments")
                kids = [
                    c
                    for c in (args.named_children if args is not None else [])
                    if c.type != "comment"
                ]
                if kids and self._is_target(kids[0]):
                    return "R" if any(self._scan(k) == "R" for k in kids[1:]) else "K"
        if t == "field_identifier" or t in ("comment", "string_literal", "char_literal"):
            return None
        if t in ("sizeof_expression", "alignof_expression", "offsetof_expression"):
            return None  # unevaluated: `sizeof(*InputBuffer)` reads nothing
        if self.ignore_addr and t == "pointer_expression" and text(n).lstrip().startswith("&"):
            arg = n.child_by_field_name("argument")
            if arg is not None and npath(text(arg)).startswith(self.path):
                return "K"  # strict mode: assume `&x` is an out-param that overwrites it
        if self._hit(n):
            return "R"
        if t == "binary_expression":
            op = n.child_by_field_name("operator")
            if op is not None and text(op) in ("&&", "||"):
                left = self._scan(n.child_by_field_name("left"))
                if left:
                    return left
                return "R" if self._scan(n.child_by_field_name("right")) == "R" else None
        if t == "conditional_expression":
            c = self._scan(n.child_by_field_name("condition"))
            if c:
                return c
            a = self._scan(n.child_by_field_name("consequence"))
            b = self._scan(n.child_by_field_name("alternative"))
            if "R" in (a, b):
                return "R"
            return "K" if a == b == "K" else None
        if t == "field_expression":
            arg = n.child_by_field_name("argument")
            return self._scan(arg) if arg is not None and arg.type != "identifier" else None
        for c in n.children:
            r = self._scan(c)
            if r:
                return r
        return None

    def reads(self, n: Node | None) -> bool:
        return self._scan(n) == "R"

    def _eff(self, expr: Node | None, out: bool) -> bool:
        r = self._scan(expr)
        return True if r == "R" else False if r == "K" else out

    # -- statements

    def _decl(self, n: Node, out: bool) -> bool:
        decls = [c for c in n.named_children if c.type in ("init_declarator",)]
        for d in reversed(decls):
            name = d.child_by_field_name("declarator")
            val = d.child_by_field_name("value")
            if name is not None and npath(text(name).lstrip("*&")) == self.path:
                out = self.reads(val)
            else:
                out = self._eff(val, out)
        return out

    def live_seq(self, stmts: list[Node], out: bool, ctx: dict[str, Callable[[], bool]]) -> bool:
        for s in reversed(stmts):
            out = self.live_in(s, out, ctx)
        return out

    def _stmts(self, n: Node) -> list[Node]:
        return [c for c in n.named_children if c.type not in ("comment",)]

    def _loop_head(self, n: Node, out: bool, ctx: dict[str, Callable[[], bool]]) -> bool:
        cond = n.child_by_field_name("condition")
        body = n.child_by_field_name("body")
        upd = n.child_by_field_name("update")
        h = False
        for _ in range(4):
            cont = self._eff(upd, h) if n.type == "for_statement" else h
            inner = dict(ctx, brk=lambda o=out: o, cont=lambda c=cont: c)
            body_in = self.live_in(body, cont, inner)
            # `while (TRUE)` / `for (;;)` never leave through the condition, and
            # `do { } while (0)` never goes round again
            exit_live = False if _const(cond, n.type) is True else out
            again = False if _const(cond, n.type) is False else body_in
            if n.type == "do_statement":
                new = body_in
            else:
                new = self._eff(cond, (body_in if again else False) or exit_live)
            if n.type == "do_statement":
                # h here is the liveness at the condition: cond, then body again, or exit
                cond_live = self._eff(cond, again or exit_live)
                if cond_live == h:
                    return new
                h = cond_live
                continue
            if new == h:
                return h
            h = new
        return True

    def live_in(self, n: Node | None, out: bool, ctx: dict[str, Callable[[], bool]]) -> bool:
        if n is None:
            return out
        t = n.type
        if t == "compound_statement":
            return self.live_seq(self._stmts(n), out, ctx)
        if t == "expression_statement":
            return self._eff(n.named_children[0] if n.named_children else None, out)
        if t == "declaration":
            return self._decl(n, out)
        if t == "if_statement":
            alt = n.child_by_field_name("alternative")
            if alt is not None and alt.type == "else_clause":
                alt = alt.named_children[0] if alt.named_children else None
            cons = n.child_by_field_name("consequence")
            if self.ignore_debug and _debug_only(cons) and _debug_only(alt):
                return out  # `if (!x) ASSERT(FALSE);` is a diagnostic, its test included
            branches = self.live_in(cons, out, ctx) or (
                self.live_in(alt, out, ctx) if alt is not None else out
            )
            return self._eff(n.child_by_field_name("condition"), branches)
        if t in self.LOOPS:
            init = n.child_by_field_name("initializer")
            head = self._loop_head(n, out, ctx)
            if init is None:
                return head
            if init.type == "declaration":
                return self._decl(init, head)
            return self._eff(init, head)
        if t == "switch_statement":
            body = n.child_by_field_name("body")
            inner = dict(ctx, brk=lambda o=out: o)
            cases = [
                c
                for c in (body.named_children if body is not None else [])
                if c.type == "case_statement"
            ]
            live_at_case = []
            o = out
            has_default = False
            for c in reversed(cases):
                if c.child_by_field_name("value") is None:
                    has_default = True
                stmts = [
                    s
                    for s in c.named_children
                    if s != c.child_by_field_name("value") and s.type != "comment"
                ]
                o = self.live_seq(stmts, o, inner)
                live_at_case.append(o)
            entry = any(live_at_case) or (not has_default and out)
            return self._eff(n.child_by_field_name("condition"), entry)
        if t == "return_statement":
            return any(self.reads(c) for c in n.named_children)  # nothing is live after it
        if t == "break_statement":
            return ctx["brk"]() if "brk" in ctx else out
        if t == "continue_statement":
            return ctx["cont"]() if "cont" in ctx else out
        if t == "goto_statement":
            lab = n.child_by_field_name("label")
            return self.label_live(text(lab)) if lab is not None else True
        if t == "labeled_statement":
            kids = [c for c in n.named_children if c.type != "statement_identifier"]
            return self.live_seq(kids, out, ctx)
        if t in ("preproc_if", "preproc_ifdef", "preproc_elif", "preproc_else", "preproc_elifdef"):
            alt = n.child_by_field_name("alternative")
            kids = [
                c
                for c in n.named_children
                if c != alt
                and c != n.child_by_field_name("condition")
                and c != n.child_by_field_name("name")
                and c.type not in ("comment", "preproc_arg")
            ]
            arm = _pp_arm(n)
            here = self.live_seq(kids, out, ctx)
            other = self.live_in(alt, out, ctx) if alt is not None else out
            if arm == "this":
                return here
            if arm == "other":
                return other
            return here or other
        if t in (
            "comment",
            ";",
            "empty_statement",
            "preproc_call",
            "preproc_def",
            "preproc_function_def",
        ):
            return out
        # anything else (ERROR, SEH macro soup, try/catch...): opaque, read if it names us
        return self._eff(n, out)

    # -- continuations

    def _ctx_at(self, n: Node) -> dict[str, Callable[[], bool]]:
        ctx: dict[str, Callable[[], bool]] = {}
        p = n.parent
        while p is not None and p != self.fn:
            if p.type in self.LOOPS and "cont" not in ctx:
                loop = p
                ctx.setdefault("brk", lambda loop=loop: self.after(loop))
                ctx["cont"] = lambda loop=loop: self._loop_head(
                    loop, self.after(loop), self._ctx_at(loop)
                )
            elif p.type == "switch_statement" and "brk" not in ctx:
                sw = p
                ctx["brk"] = lambda sw=sw: self.after(sw)
            p = p.parent
        return ctx

    def label_live(self, label: str) -> bool:
        if label in self._label_live:
            return self._label_live[label]
        node = self.labels.get(label)
        if node is None:
            return True
        self._label_live[label] = False  # cycle guard (iterated once below)
        v = self.live_in(node, self.after(node), self._ctx_at(node))
        self._label_live[label] = v
        return v

    def after(self, n: Node) -> bool:
        key = n.id
        if key in self._after:
            return self._after[key]
        self._after[key] = False  # cycle guard
        v = self._after_uncached(n)
        self._after[key] = v
        return v

    def _after_uncached(self, n: Node) -> bool:
        p = n.parent
        if p is None or n == self.fn or p == self.fn:
            return False
        if p.type == "compound_statement" and p.parent == self.fn:
            return self.live_seq(self._following(p, n), False, self._ctx_at(n))
        if p.type in (
            "compound_statement",
            "labeled_statement",
            "translation_unit",
        ) or p.type.startswith("preproc_"):
            return self.live_seq(self._following(p, n), self.after(p), self._ctx_at(n))
        if p.type == "case_statement":
            rest = self._following(p, n)
            sw_body = p.parent
            later = []
            if sw_body is not None:
                cs = [c for c in sw_body.named_children if c.type == "case_statement"]
                if p in cs:
                    for c in cs[cs.index(p) + 1 :]:
                        later += [
                            s for s in c.named_children if s != c.child_by_field_name("value")
                        ]
            sw = sw_body.parent if sw_body is not None else None
            return self.live_seq(
                rest + later, self.after(sw) if sw is not None else False, self._ctx_at(n)
            )
        if p.type == "else_clause":
            return self.after(p.parent)
        if p.type == "if_statement":
            if n == p.child_by_field_name("condition"):
                alt = p.child_by_field_name("alternative")
                if alt is not None and alt.type == "else_clause":
                    alt = alt.named_children[0] if alt.named_children else None
                out = self.after(p)
                ctx = self._ctx_at(p)
                return self.live_in(p.child_by_field_name("consequence"), out, ctx) or (
                    self.live_in(alt, out, ctx) if alt is not None else out
                )
            return self.after(p)
        if p.type in self.LOOPS:
            out = self.after(p)
            body = p.child_by_field_name("body")
            ctx = self._ctx_at(p)
            head = self._loop_head(p, out, ctx)
            if n == body:
                if p.type == "for_statement":
                    return self._eff(p.child_by_field_name("update"), head)
                if p.type == "do_statement":
                    return self._eff(p.child_by_field_name("condition"), head or out)
                return head
            if n == p.child_by_field_name("condition"):
                inner = dict(ctx, brk=lambda o=out: o, cont=lambda h=head: h)
                return self.live_in(body, head, inner) or out
            if n == p.child_by_field_name("initializer"):
                return head
            if n == p.child_by_field_name("update"):
                return head
            return head
        if p.type == "switch_statement":
            if n == p.child_by_field_name("condition"):
                return self.live_in(p, self.after(p), self._ctx_at(p)) or self.after(p)
            return self.after(p)
        if p.type in (
            "expression_statement",
            "return_statement",
            "declaration",
            "init_declarator",
            "comma_expression",
            "parenthesized_expression",
        ):
            # the rest of the enclosing expression, then the statement's continuation
            if p.type == "return_statement":
                return self._later(p, n, False)
            return self._later(p, n, None)
        # a write nested inside a larger expression: the rest of it, then climb
        return self._later(p, n, None)

    def _later(self, p: Node, n: Node, tail: bool | None) -> bool:
        """Liveness after `n` inside expression-ish parent `p`: its later operands in
        order, then (tail None) the parent's own continuation."""
        for c in p.children:
            if c.start_byte >= n.end_byte:
                r = self._scan(c)
                if r:
                    return r == "R"
        return self.after(p) if tail is None else tail

    def _following(self, parent: Node, child: Node) -> list[Node]:
        kids = [c for c in parent.named_children if c.type != "comment"]
        # child may be nested deeper than a direct kid only via ERROR recovery
        for i, k in enumerate(kids):
            if k.start_byte <= child.start_byte and child.end_byte <= k.end_byte:
                return kids[i + 1 :]
        return []


def _const(cond: Node | None, loop: str) -> bool | None:
    """A loop condition that is a constant: True for `TRUE`/`1`/absent (`for (;;)`),
    False for `FALSE`/`0`; None when it depends on anything."""
    if cond is None:
        return True if loop == "for_statement" else None
    t = re.sub(r"[\s()]", "", text(cond))
    if t in ("TRUE", "1", "true"):
        return True
    if t in ("FALSE", "0", "false"):
        return False
    return None


def _debug_only(n: Node | None) -> bool:
    """`n` does nothing but debug output / asserts (or nothing at all)."""
    if n is None or n.type in ("comment", "empty_statement", ";"):
        return True
    if n.type == "compound_statement":
        return all(_debug_only(c) for c in n.named_children)
    if n.type == "expression_statement":
        e = n.named_children[0] if n.named_children else None
        if e is None:
            return True
        if e.type == "call_expression":
            f = e.child_by_field_name("function")
            return f is not None and is_debug(text(f))
    return False


# The build always defines __REACTOS__: an `#ifdef __REACTOS__` arm is the code, its
# `#else` is Wine's, and `#if 0` is nobody's.
def _pp_arm(n: Node) -> str | None:
    head = text(n).lstrip()
    name = n.child_by_field_name("name")
    if n.type == "preproc_ifdef" and name is not None and text(name) == "__REACTOS__":
        return "other" if head.startswith("#ifndef") else "this"
    cond = n.child_by_field_name("condition")
    if n.type in ("preproc_if", "preproc_elif") and cond is not None:
        c = re.sub(r"\s+", "", text(cond))
        if c in ("0", "defined(__REACTOS__)==0", "!defined(__REACTOS__)", "!defined__REACTOS__"):
            return "other"
        if c in ("1", "defined(__REACTOS__)", "defined__REACTOS__"):
            return "this"
    return None


def c_fn_name(fn: Node) -> str | None:
    d = fn.child_by_field_name("declarator")
    while d is not None and d.type not in ("function_declarator",):
        d = d.child_by_field_name("declarator")
    if d is None:
        return None
    nm = d.child_by_field_name("declarator")
    if nm is None:
        return None
    return re.split(r"::", text(nm))[-1].strip()


def c_writes(fn: Node, path: str) -> list[tuple[Node, str, str]]:
    """(node, op, rhs_key) for every write to `path` in source order."""
    out: list[tuple[Node, str, str]] = []

    def walk(n: Node) -> None:
        if n.type == "assignment_expression":
            left, op, right = (n.child_by_field_name(f) for f in ("left", "operator", "right"))
            if left is not None and npath(text(left)) == path and op is not None:
                o = text(op)
                k = rhs_key(text(right)) if right is not None else "expr"
                out.append((n, o, k if o == "=" else o + k))
        elif n.type == "init_declarator":
            nm, val = n.child_by_field_name("declarator"), n.child_by_field_name("value")
            if nm is not None and npath(text(nm).lstrip("*&")) == path:
                out.append((n, "init", rhs_key(text(val)) if val is not None else "expr"))
        elif n.type == "update_expression":
            arg = n.child_by_field_name("argument")
            if arg is not None and npath(text(arg)) == path:
                o = "+=" if "++" in text(n) else "-="
                out.append((n, o, o + "1"))
        for c in n.children:
            walk(c)

    walk(fn.child_by_field_name("body") or fn)
    return out


def c_declared_uninit(fn: Node, path: str) -> bool:
    """The variable is declared in this function without an initializer."""
    found = False

    def walk(n: Node) -> None:
        nonlocal found
        if n.type == "declaration":
            for c in n.named_children:
                kinds = ("identifier", "pointer_declarator", "array_declarator")
                if c.type in kinds and npath(text(c).lstrip("*&")) == path:
                    found = True
        for c in n.children:
            walk(c)

    walk(fn)
    return found


def _graded(fn: Node, path: str, probe: Callable[[CLive], bool]) -> str:
    """C_READS if the C reads it outright; C_READS_ADDR if only by passing `&x` somewhere
    (an out-param or a real read -- the callee decides); C_READS_DEBUG if only a
    DPRINT/ASSERT reads it; else C_DEAD."""
    if probe(CLive(fn, path, ignore_debug=True, ignore_addr=True)):
        return "C_READS"
    if probe(CLive(fn, path, ignore_debug=True)):
        return "C_READS_ADDR"
    return "C_READS_DEBUG" if probe(CLive(fn, path)) else "C_DEAD"


def resolve_c_path(fn: Node, rust_path: str) -> str | None:
    """The C spelling of a Rust lvalue: itself if the C function names it, else the ONE C
    spelling equal to it case- and underscore-insensitively (`status` -> `Status`,
    `list_count_index` -> `ListCountIndex`); None when several C names would fit."""
    seen: set[str] = set()

    def walk(n: Node) -> None:
        if n.type in ("identifier", "field_expression"):
            seen.add(npath(text(n)))
        for c in n.children:
            walk(c)

    walk(fn)
    if rust_path in seen:
        return rust_path
    fits = {c for c in seen if canon_path(c) == canon_path(rust_path)}
    if len(fits) == 1:
        return fits.pop()
    return rust_path if not fits else None


def c_verdict(fn: Node, w: RustWrite) -> tuple[str, int]:
    """(verdict, 1-based C line of the matched write or 0)."""
    c_path = resolve_c_path(fn, w.lhs)
    if c_path is None:
        return "NO_MATCH", 0
    w = RustWrite(w.shape, c_path, w.key, w.op, w.ordinal, w.nwrites, w.key_ordinal, w.key_count)
    if w.op == "param":
        body = fn.child_by_field_name("body")
        if body is None:
            return "NO_MATCH", 0
        return _graded(fn, w.lhs, lambda lv: lv.live_in(body, False, {})), fn.start_point[0] + 1
    writes = c_writes(fn, w.lhs)
    if w.op == "init" and not w.key.startswith("call:"):
        cands = [x for x in writes if x[1] == "init"]
        if not cands:
            if c_declared_uninit(fn, w.lhs):
                return "C_NO_INIT", 0
            return "NO_MATCH", 0
        same = [x for x in cands if x[2] == w.key]
        if not same:
            # the C initializes it to something else: typically a C99 declaration further
            # down (`PWCHAR typeName = GetPeripheralTypeW(..)`) that the Rust hoisted to the
            # top with a default, which then has no C counterpart. But `= STATUS_SUCCESS`
            # against `= 0` is the same value spelled two ways, so only a C initializer
            # that is itself dead lets this be called harmless.
            if any(_graded(fn, w.lhs, lambda lv, x=x: lv.after(x[0])) != "C_DEAD" for x in cands):
                return "NO_MATCH", cands[0][0].start_point[0] + 1
            return "C_NO_INIT", 0
        node = same[0][0]
    else:
        keyed = [x for x in writes if x[2] == w.key]
        if keyed and len(keyed) == w.key_count:
            # the same write, the same number of times on both sides: pair by ordinal
            node = keyed[w.key_ordinal][0]
        elif keyed:
            # the counts differ (a hoisted or merged write): a pairing would be a guess,
            # unless every candidate gives the same answer
            vs = {_graded(fn, w.lhs, lambda lv, x=x: lv.after(x[0])) for x in keyed}
            if len(vs) == 1 and w.key_count <= len(keyed):
                return vs.pop(), keyed[0][0].start_point[0] + 1
            return "NO_MATCH", 0
        elif w.key == "expr" and w.nwrites == len(writes) and 0 <= w.ordinal < len(writes):
            node = writes[w.ordinal][0]
        else:
            return "NO_MATCH", 0
    return _graded(fn, w.lhs, lambda lv: lv.after(node)), node.start_point[0] + 1


# --------------------------------------------------------------------------- C function index


def c_candidates(names: set[str]) -> dict[str, list[Path]]:
    """name -> C/C++ files (outside sdk/lib/rust) that contain `name(`."""
    hits: dict[str, set[Path]] = defaultdict(set)
    names_l = sorted(n for n in names if re.fullmatch(r"\w+", n))
    for i in range(0, len(names_l), 200):
        chunk = names_l[i : i + 200]
        pat = r"\b(" + "|".join(chunk) + r")\s*\("
        r = subprocess.run(
            [
                "git",
                "-C",
                str(REPO),
                "grep",
                "-n",
                "-I",
                "-E",
                pat,
                "--",
                "*.c",
                "*.cpp",
                "*.cxx",
                "*.cc",
                "*.h",
                "*.hpp",
                "*.inl",
                ":!sdk/lib/rust/",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if r.returncode not in (0, 1):
            raise SystemExit(f"git grep failed: {r.stderr.strip()}")
        rx = re.compile(pat)
        for ln in r.stdout.splitlines():
            path, _, rest = ln.split(":", 2)
            for m in rx.finditer(rest):
                hits[m.group(1)].add(REPO / path)
    return {k: sorted(v) for k, v in hits.items()}


_C_FNS: dict[Path, dict[str, list[Node]]] = {}


def c_functions(path: Path) -> dict[str, list[Node]]:
    if path not in _C_FNS:
        lang = "c" if path.suffix in (".c", ".h") else "cpp"
        tree = parser(lang).parse(path.read_bytes())
        out: dict[str, list[Node]] = defaultdict(list)

        def walk(n: Node) -> None:
            if n.type == "function_definition":
                nm = c_fn_name(n)
                if nm:
                    out[nm].append(n)
            for c in n.children:
                if (
                    c.type
                    in (
                        "translation_unit",
                        "declaration_list",
                        "namespace_definition",
                        "class_specifier",
                        "field_declaration_list",
                        "linkage_specification",
                        "preproc_if",
                        "preproc_ifdef",
                        "preproc_else",
                        "preproc_elif",
                        "function_definition",
                        "template_declaration",
                    )
                    or n.type == "translation_unit"
                ):
                    walk(c)

        walk(tree.root_node)
        _C_FNS[path] = out
    return _C_FNS[path]


def _affinity(rust_path: str, c_path: Path) -> int:
    crate = rust_path.split("/")[0]
    toks = {t for t in re.split(r"[-_]", crate.lower()) if t not in ("rust", "legacy", "lib", "")}
    parts = [p.lower() for p in c_path.relative_to(REPO).parts]
    score = sum(2 for t in toks if t in parts) + sum(1 for t in toks if any(t in p for p in parts))
    if c_path.stem.lower() == Path(rust_path).stem.lower():
        score += 3
    return score


# --------------------------------------------------------------------------- driver


@dataclass
class Row:
    path: str
    line: int
    var: str
    arches: list[str]
    fn: str = ""
    shape: str = ""
    verdict: str = ""
    c_loc: str = ""
    stmt: str = ""
    note: str = ""
    extra: dict[str, str] = field(default_factory=dict)

    def rank_key(self) -> tuple:
        return (
            VERDICT_ORDER.index(self.verdict) if self.verdict in VERDICT_ORDER else 99,
            SHAPE_ORDER.index(self.shape) if self.shape in SHAPE_ORDER else 99,
            0 if STATUS_LIKE.match(canon(self.var)) else 1,
            -len(self.arches),
            self.path,
            self.line,
        )


def _snip(snippet: str) -> str:
    """rustc's display line minus its gutter: a leading `/` or `|` (multi-line span) and the
    `...` it puts on either end of a line too long to show whole."""
    s = snippet.strip().lstrip("/|").strip()
    s = s.removeprefix("...").removesuffix("...").strip()
    return s


def relocate(lines: list[str], line: int, snippet: str, exact: bool = False) -> int | None:
    """The current line of a site the log put at `line`.

    `exact`: `line` was already mapped through `git diff <built tree>` (`line_map`), so the
    snippet must be ON it. Otherwise this is a guess by text, and a guess is only taken
    when it is unique: in a file that shifted by k lines, the k-th `hRes = IStream_Write(`
    below a site matches the site's OLD line number, and the nearest-match search would
    hand one site's verdict to its neighbour (two rows on one line, one line never read)."""
    s = _snip(snippet)
    if not s:
        return line
    here = 0 < line <= len(lines) and s in lines[line - 1]
    if exact:
        return line if here else None
    hits = [i + 1 for i in range(max(0, line - 400), min(len(lines), line + 399)) if s in lines[i]]
    if len(hits) == 1:
        return hits[0]
    return line if here and not hits else None


def built_tree(log: Path) -> str | None:
    """The tree the build that wrote `log` compiled: the `warm-receipt` stage of its
    summary.json, which `vibeos-cli build` records as `git write-tree` of the worktree.
    None when the build predates receipts or did not get that far."""
    try:
        stages = json.loads((log.parent / "summary.json").read_text())["stages"]
    except (OSError, ValueError, KeyError):
        return None
    for st in stages:
        m = re.match(r"tree ([0-9a-f]{7,40})$", st.get("detail", ""))
        if st.get("name") == "warm-receipt" and st.get("status") == "ok" and m:
            cp = subprocess.run(
                ["git", "-C", str(REPO), "rev-parse", "--verify", "-q", m.group(1) + "^{tree}"],
                capture_output=True,
                text=True,
            )
            return cp.stdout.strip() or None
    return None


_HUNK = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", re.M)


def line_map(tree: str, rel: str) -> Callable[[int], int | None]:
    """old line (in `tree`) -> line in the working tree, or None if the line itself changed."""
    cp = subprocess.run(
        [
            "git",
            "-C",
            str(REPO),
            "diff",
            "-U0",
            "--no-color",
            "--no-ext-diff",
            tree,
            "--",
            str((RUST_ROOT / rel).relative_to(REPO)),
        ],
        capture_output=True,
        text=True,
    )
    hunks = [(int(a), int(b or 1), int(d or 1)) for a, b, _c, d in _HUNK.findall(cp.stdout)]

    def f(line: int) -> int | None:
        off = 0
        for a, b, d in hunks:
            # `-a,0` inserts AFTER old line a; `-a,b` replaces old lines a..a+b-1.
            first = a + 1 if b == 0 else a
            if line < first:
                break
            if b and line < a + b:
                return None
            off += d - b
        return line + off

    return f


def map_sites(sites: list[census.Site], tree: str) -> None:
    """Move each site's line from `tree` to the working tree; 0 marks a changed line."""
    maps: dict[str, Callable[[int], int | None]] = {}
    for s in sites:
        if s.path not in maps:
            maps[s.path] = line_map(tree, s.path)
        s.line = maps[s.path](s.line) or 0


def analyze(sites: list[census.Site], exact: bool = False) -> list[Row]:
    rows: list[Row] = []
    pending: list[tuple[Row, RustWrite]] = []
    for s in sites:
        m = re.search(r"`([^`]+)`", s.msg)
        row = Row(s.path, s.line, m.group(1) if m else "?", sorted(s.arches))
        rows.append(row)
        f = RUST_ROOT / s.path
        if not f.is_file():
            row.verdict, row.note = "STALE", "file gone"
            continue
        if exact and not s.line:
            row.verdict, row.note = "STALE", "line changed since the build"
            continue
        src = f.read_bytes()
        lines = src.decode("utf8", "replace").split("\n")
        ln = relocate(lines, s.line, s.snippet, exact)
        if ln is None:
            row.verdict, row.note = "STALE", "snippet not found (or not unique)"
            continue
        col = s.col if ln == s.line else (lines[ln - 1].find(_snip(s.snippet)[:20]) + 1)
        row.line = ln
        row.stmt = lines[ln - 1].strip()[:100]
        got = rust_site(src, ln, max(col, 1))
        if isinstance(got, str):
            row.verdict, row.note = "STALE", got
            continue
        row.fn, w = got
        row.shape = w.shape
        pending.append((row, w))

    index = c_candidates({r.fn for r, _ in pending})
    for row, w in pending:
        defs: list[tuple[int, Path, Node]] = []
        for p in index.get(row.fn, []):
            try:
                fns = c_functions(p).get(row.fn, [])
            except (OSError, ValueError):
                continue
            defs += [(_affinity(row.path, p), p, fn) for fn in fns]
        if not defs:
            row.verdict = "NO_C_FN"
            continue
        best = max(d[0] for d in defs)
        top = [d for d in defs if d[0] == best]
        verdicts = []
        for _, p, fn in top:
            v, cl = c_verdict(fn, w)
            verdicts.append((v, f"{p.relative_to(REPO)}:{cl}" if cl else str(p.relative_to(REPO))))
        kinds = {v for v, _ in verdicts}
        if len(kinds) == 1:
            row.verdict, row.c_loc = verdicts[0]
        else:
            row.verdict = "AMBIGUOUS"
            row.c_loc = "; ".join(f"{v}@{loc}" for v, loc in verdicts)
    rows.sort(key=Row.rank_key)
    return rows


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "logs", nargs="+", type=Path, help="vibeos-cli build full.log file(s), ideally --clean ×3"
    )
    ap.add_argument("--verdict", choices=VERDICT_ORDER, help="list one verdict only")
    ap.add_argument("--crate", help="restrict to one crate (path prefix, e.g. afd-rust)")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--limit", type=int, default=0, help="list at most N rows")
    args = ap.parse_args(argv)

    per_arch: dict[str, list[census.Site]] = defaultdict(list)
    exact = True
    for log in args.logs:
        if not log.is_file():
            print(f"MISSING LOG: {log}", file=sys.stderr)
            return 2
        sites, _ = census.parse_log(log.read_text(errors="replace"))
        census.attribute(sites)
        # Map each log through ITS OWN built tree before merging: the per-arch builds of
        # one batch can be of different trees, and merge keys on the line.
        tree = built_tree(log)
        if tree:
            map_sites(sites, tree)
        else:
            exact = False
            print(
                f"note: {log} has no warm-receipt tree; lines are relocated by snippet, and a "
                "snippet that is not unique nearby is STALE",
                file=sys.stderr,
            )
        per_arch[census.arch_of(log)].extend(sites)
    merged = census.merge(per_arch)
    census.attribute(merged)
    sites = [s for s in merged if s.lint == LINT and (not args.crate or s.crate == args.crate)]
    if not sites:
        print(f"no {LINT} sites in the given logs", file=sys.stderr)
        return 2
    rows = analyze(sites, exact)

    counts: dict[tuple[str, str], int] = defaultdict(int)
    for r in rows:
        counts[(r.verdict, r.shape)] += 1
    shown = [r for r in rows if not args.verdict or r.verdict == args.verdict]
    if args.limit:
        shown = shown[: args.limit]

    if args.json:
        json.dump([r.__dict__ for r in shown], sys.stdout, indent=1)
        print()
        return 0

    print(f"{len(rows)} {LINT} sites\n")
    print(f"{'verdict':<11}" + "".join(f"{s:>14}" for s in SHAPE_ORDER) + f"{'total':>8}")
    for v in VERDICT_ORDER:
        n = [counts.get((v, s), 0) for s in SHAPE_ORDER]
        tot = sum(c for (vv, _), c in counts.items() if vv == v)
        if tot:
            print(f"{v:<11}" + "".join(f"{c:>14}" for c in n) + f"{tot:>8}")
    print()
    for i, r in enumerate(shown, 1):
        print(f"{i:4}. {r.verdict:<9} {r.shape:<13} {r.path}:{r.line}  fn {r.fn}  `{r.var}`")
        if r.stmt:
            print(f"        rust: {r.stmt}")
        if r.c_loc or r.note:
            print(f"        c:    {r.c_loc or r.note}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
