#!/usr/bin/env python3
"""
scan_irql_balance.py — find leaked IRQL raises / spinlock acquires on early-exit
paths (AST-aware). The static-analysis answer to "we translate C's manual
`OldIrql = KeRaiseIrql(); ...; KeLowerIrql(OldIrql)` verbatim, so an early
`return`/`?` between the raise and the lower silently leaks the raise."

A leaked raise is exactly what stranded arm64 soft-IRQL at HIGH_LEVEL (field
manual §9.24): once IRQL is stuck high, the next spinlock acquire "raises" DOWN
to DISPATCH, opening a preemption window mid-critical-section → corruption → a
wild-pointer BUGCHECK far from the leak. RAII (`ntkrnl_sys::guards::IrqlGuard` /
`SpinLockGuard`) makes this class impossible by construction; until adoption is
complete, this lint catches the manual-pattern leaks.

Model (intraprocedural, per function body):
  * Walk statements in source order, tracking a set of HELD resources:
      - RAISE(var):  `KeRaiseIrql(X, &mut var)` / `let var = KfRaiseIrql(X)` /
                     `var = KeRaiseIrqlToDpcLevel()` / `KeAcquireSpinLock(l,&mut var)`
                     → var becomes held.
      - LOWER(var):  `KeLowerIrql(var)` / `KfLowerIrql(var)` /
                     `KeReleaseSpinLock(l, var)` → var released.
  * At an EARLY `return` / `?` (try) with a non-empty held set → LEAK, UNLESS the
    return hands the held var back to the caller (`return old;` / tail `old` —
    ownership transfer, the legitimate raise-and-return-OldIrql contract).
  * Only functions that lower SOMETHING internally are checked — a function that
    only ever raises is the cross-function API pattern (KeAcquireSpinLock-style),
    not a leak, and flagging it floods false positives.

This is a linear held-set approximation, not full CFG dominance: a raise nested
in one `if` arm with a `return` in a sibling arm can misattribute. That residue
is what the BASELINE-diff gate is for — existing findings are baselined; the lint
fails only on NEW leaks (a fresh early-return-past-a-raise), which is the thing
worth blocking. Reduce a verified false positive with a same-line annotation:
    return;  // nolint: irql-balance (raise is conditional / lowered in callee)

Usage:
    python3 tools/scan_irql_balance.py                 # sweep the workspace
    python3 tools/scan_irql_balance.py path/to/file.rs # specific files
    python3 tools/scan_irql_balance.py --lint          # terse (baseline gate)
Exit 1 if any finding.
"""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools" / "rustconv"))
import tree_sitter_rust as tsr  # noqa: E402
from tree_sitter import Language, Parser  # noqa: E402

RUST_WS = REPO / "sdk" / "lib" / "rust"

_LANG = Language(tsr.language())
_PARSER = Parser(_LANG)

# Raise builtins that PRODUCE an OldIrql (either into a `&mut var` out-param or
# as the return value bound to a local).
# KeRaiseIrql(level, &mut old): the LEVEL is arg 0 — only an ACQUIRE when that
# level is a CONSTANT (`DISPATCH_LEVEL`), not a saved variable. `KeRaiseIrql(old,
# &mut old)` is a RESTORE-to-saved (the lower-then-restore pattern), not a leak.
RAISE_LEVELED = {"KeRaiseIrql"}
# Spinlock acquires: arg 0 is the lock, the OldIrql out-param is the LAST arg;
# always an acquire (no level argument to check).
RAISE_LOCK = {"KeAcquireSpinLock", "KeAcquireInStackQueuedSpinLock"}
RAISE_OUTPARAM = RAISE_LEVELED | RAISE_LOCK
_CONST_LEVEL = re.compile(r"\b[A-Z][A-Z0-9_]*_LEVEL\b")
RAISE_RETURN = {
    "KfRaiseIrql",
    "KeRaiseIrqlToDpcLevel",
    "KeRaiseIrqlToSynchLevel",
    "KeAcquireSpinLockRaiseToSynch",
}
LOWER_ARG = {"KeLowerIrql", "KfLowerIrql"}  # first arg is the OldIrql var
RELEASE_ARG2 = {"KeReleaseSpinLock"}  # second arg is the OldIrql var

_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
# Path/macro/keyword tokens that wrap an out-param lvalue but are NOT the
# variable — `core::ptr::addr_of_mut!(OldIrql)`, `&mut OldIrql`, `&raw mut x`.
_LVALUE_NOISE = {"core", "ptr", "addr_of_mut", "addr_of", "mut", "raw", "self", "crate"}


def _nolint(line_text: str) -> bool:
    return bool(
        re.search(r"//\s*nolint(?::\s*irql-balance\b|\s*$|\s*\()", line_text)
        or re.search(r"//\s*nolint\b(?!\s*:)", line_text)
    )


def _call_name(node, src: bytes):
    """If `node` is a call_expression, return (fn_name, args_node) else None."""
    if node.type != "call_expression":
        return None
    fn = node.child_by_field_name("function")
    if fn is None:
        return None
    args = node.child_by_field_name("arguments")
    # bare `Foo(...)` or path `a::b::Foo(...)`
    if fn.type == "identifier":
        return src[fn.start_byte : fn.end_byte].decode(errors="replace"), args
    if fn.type == "scoped_identifier":
        name = fn.child_by_field_name("name")
        if name is not None:
            return src[name.start_byte : name.end_byte].decode(errors="replace"), args
    return None


def _arg_idents(args_node, src: bytes, idx: int):
    """Idents appearing in argument `idx` (0-based) of a call's arguments node."""
    if args_node is None:
        return set()
    real = [c for c in args_node.children if c.type not in ("(", ")", ",")]
    if idx >= len(real):
        return set()
    text = src[real[idx].start_byte : real[idx].end_byte].decode(errors="replace")
    # Drop any `as <type>` cast tail — `&mut OldIrql as *mut KIRQL` is just the
    # lvalue `OldIrql`; the cast tokens (as, KIRQL, _, mut) are not variables.
    text = re.split(r"\bas\b", text, maxsplit=1)[0]
    # Strip the pointer/reference wrapper noise so `addr_of_mut!(OldIrql)` and
    # `&mut OldIrql` both reduce to {OldIrql}.
    return {i for i in _IDENT.findall(text) if i not in _LVALUE_NOISE}


def _find_functions(root):
    out = []

    def rec(n):
        if n.type == "function_item":
            body = n.child_by_field_name("body")
            if body is not None:
                out.append(body)
        for c in n.children:
            rec(c)

    rec(root)
    return out


def _scan_function(body, src: bytes, lines):
    """Linear held-set walk over one function body. Returns findings."""
    findings = []
    held: set[str] = set()
    lowers_internally = [False]

    def line_of(node):
        return node.start_point[0] + 1

    def suppressed(node):
        ln = node.start_point[0]
        return ln < len(lines) and _nolint(lines[ln])

    def visit(node):
        # RETURN / TRY: check for held resources leaking on this exit.
        if node.type == "return_expression" or node.type == "try_expression":
            if held and not suppressed(node):
                # Ownership transfer? `return old;` / `old?` hands the var back.
                ret_txt = src[node.start_byte : node.end_byte].decode(errors="replace")
                ret_ids = set(_IDENT.findall(ret_txt))
                leaked = held - ret_ids
                if leaked:
                    is_ret = node.type == "return_expression"
                    kind = "IRQL_LEAK_RETURN" if is_ret else "IRQL_LEAK_TRY"
                    findings.append(
                        (
                            line_of(node),
                            kind,
                            f"early {'return' if node.type == 'return_expression' else '`?`'} "
                            f"while holding {', '.join(sorted(leaked))} — raise/acquire not "
                            f"lowered on this path (use IrqlGuard/SpinLockGuard)",
                        )
                    )
            # keep walking children (a return value can contain calls)
            for c in node.children:
                visit(c)
            return

        call = _call_name(node, src)
        if call is not None:
            name, args = call
            if name in RAISE_OUTPARAM:
                # OldIrql is the LAST arg, an `&mut var`.
                real = [c for c in (args.children if args else []) if c.type not in ("(", ")", ",")]
                acquire = True
                if name in RAISE_LEVELED and real:
                    lvl = src[real[0].start_byte : real[0].end_byte].decode(errors="replace")
                    # Restore-to-saved (level is a variable, not a *_LEVEL const).
                    acquire = bool(_CONST_LEVEL.search(lvl))
                if acquire and real:
                    ids = _arg_idents(args, src, len(real) - 1)
                    held.update(ids)
            elif name in RAISE_RETURN:
                # value bound by an enclosing let/assignment — resolve upward.
                var = _bound_var(node, src)
                if var:
                    held.add(var)
            elif name in LOWER_ARG:
                held.difference_update(_arg_idents(args, src, 0))
                lowers_internally[0] = True
            elif name in RELEASE_ARG2:
                held.difference_update(_arg_idents(args, src, 1))
                lowers_internally[0] = True

        for c in node.children:
            visit(c)

    visit(body)
    # Only trust the analysis for functions that balance internally; a
    # raise-only function is the cross-function API contract, not a leak.
    return findings if lowers_internally[0] else []


def _bound_var(call_node, src: bytes):
    """For `let v = KfRaiseIrql(..)` or `v = KfRaiseIrql(..)`, return `v`."""
    _STOP = ("let_declaration", "assignment_expression", "block", "function_item")
    p = call_node.parent
    while p is not None and p.type not in _STOP:
        p = p.parent
    if p is None:
        return None
    if p.type == "let_declaration":
        pat = p.child_by_field_name("pattern")
        if pat is not None:
            m = _IDENT.search(src[pat.start_byte : pat.end_byte].decode(errors="replace"))
            return m.group(0) if m else None
    if p.type == "assignment_expression":
        left = p.child_by_field_name("left")
        if left is not None:
            ltxt = src[left.start_byte : left.end_byte].decode(errors="replace").strip()
            # Only a PLAIN LOCAL is intraprocedurally balanced. A field/deref
            # store (`(*FastMutex).OldIrql = KfRaiseIrql(..)`) stashes the IRQL
            # for a PAIRED release function (ExReleaseFastMutex) — that's a
            # cross-call contract, not a leak. Reject anything but a bare ident.
            if _IDENT.fullmatch(ltxt):
                return ltxt
    return None


def scan_source(src: bytes):
    tree = _PARSER.parse(src)
    lines = src.decode(errors="replace").splitlines()
    findings = []
    for body in _find_functions(tree.root_node):
        findings.extend(_scan_function(body, src, lines))
    findings.sort()
    return findings


def _iter_files(paths):
    for p in paths:
        p = Path(p)
        if p.is_dir():
            for f in p.rglob("*.rs"):
                s = str(f)
                if "/target/" in s or f.name.startswith("bindgen_") or f.name == "build.rs":
                    continue
                yield f
        elif p.suffix == ".rs":
            yield p


def main(argv):
    args = [a for a in argv if not a.startswith("--")]
    lint = "--lint" in argv
    roots = [Path(a) for a in args] if args else [RUST_WS]

    total = 0
    for f in sorted(set(_iter_files(roots))):
        try:
            findings = scan_source(f.read_bytes())
        except Exception as e:
            print(f"{f}: parse error: {e}", file=sys.stderr)
            continue
        try:
            rel = f.resolve().relative_to(REPO)
        except ValueError:
            rel = f
        for ln, kind, msg in findings:
            total += 1
            if lint:
                print(f"{rel}:{ln}: {kind}")
            else:
                print(f"{rel}:{ln}: [{kind}] {msg}")

    if not lint:
        print(f"\n{total} IRQL-balance finding(s)")
    return 1 if total else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
