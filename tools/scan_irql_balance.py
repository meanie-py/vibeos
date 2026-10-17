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

Cross-function summaries (ownership-model Layer 3, phase 4a): before the
per-function walk, a workspace pass computes per-function summaries and
propagates them through the (syntactic, direct-call) call graph to a fixpoint:
  * RAISE-WRAPPER — the body nets a raise whose OldIrql is handed back to the
    caller (returned): call sites treat it like `KfRaiseIrql`.
  * LOWER-WRAPPER(i) — the body lowers/releases using its own parameter #i:
    call sites treat argument i like the `KeLowerIrql` argument.
  * MAY-SLEEP — the body (transitively) reaches a wait
    (`KeWaitForSingleObject` / `KeDelayExecutionThread` / CPS `wait_for` ...).
The summaries feed two things: wrapper-aware held-set tracking, and the new
IRQL_SLEEP_WHILE_HELD finding — a call into a may-sleep function while the
held set is non-empty (the static mirror of cswitchdep's SLEEP_WITH_SPINLOCK;
NT's real rule is waits ≤ APC_LEVEL, and a live raise/spinlock breaks it).

CTX_CAPTURE_ACROSS_WAIT (the stable substitute for the REJECTED SwitchSafe
auto-trait — ownership-model Layer 2): a `wait_for`/`wait_for_armed` CPS
continuation capturing a local minted from per-CPU state (KeGetPcr /
KeGetCurrentPrcb / Pcpu<) — stale after the switch; re-derive it inside the
continuation. Borrowck already forbids borrow-captures (guards borrow the
token wait_for consumes); this catches the owned smuggles.
Summaries are keyed by bare name (collisions merge conservatively) and only
cover the Rust workspace — calls into C externs have no summary and propagate
nothing, which bounds the noise and is the honest syntactic limit (function
pointers / DPC routines are phase 4b, MIR).

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
    python3 tools/scan_irql_balance.py --summaries     # dump call-graph facts
Exit 1 if any finding.

NOTE on scoped runs: pointing the scanner at specific files still builds the
summary database from the whole workspace (call-graph facts are global); only
the FINDINGS are limited to the named files.
"""

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.lint_facts import source_facts
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools" / "rustconv"))
import tree_sitter_rust as tsr  # noqa: E402
from tree_sitter import Language, Parser, Query, QueryCursor  # noqa: E402

RUST_WS = REPO / "sdk" / "lib" / "rust"

_LANG = Language(tsr.language())
_PARSER = Parser(_LANG)
_FUNCTIONS = Query(_LANG, "(function_item) @fn")
_SUMMARY_NODES = Query(_LANG, "[(return_expression) (call_expression)] @item")

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

# Seed may-sleep set: functions that park the thread (voluntary switch). The
# fixpoint propagates this through Rust callers; C externs stay opaque.
# `wait_for` is the ntkrnl_sys::token CPS wait (matched by bare name).
SLEEP_SEEDS = {
    "KeWaitForSingleObject",
    "KeWaitForMultipleObjects",
    "KeDelayExecutionThread",
    "KeWaitForGate",
    "wait_for",
    "wait_for_armed",
}
# Functions that reach a wait only on a path their own contract confines to
# PASSIVE_LEVEL, so a caller holding a raise is not sleeping: KeSynchronizeExecution
# takes a KMUTEX only for a CONNECT_MESSAGE_BASED_PASSIVE interrupt (iomgr irq.rs
# IopSynchronizePassiveInterrupt), which Windows lets be synchronized only at PASSIVE.
# Every other interrupt it synchronizes by raising and spinning. They never become
# may-sleep, and so never propagate it to their callers.
SLEEP_CONDITIONAL = {
    "KeSynchronizeExecution",
}

_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
# Path/macro/keyword tokens that wrap an out-param lvalue but are NOT the
# variable — `core::ptr::addr_of_mut!(OldIrql)`, `&mut OldIrql`, `&raw mut x`.
_LVALUE_NOISE = {"core", "ptr", "addr_of_mut", "addr_of", "mut", "raw", "self", "crate"}

# Context-bound value sources: capturing one of these ACROSS a wait_for
# continuation smuggles per-CPU state past a context switch (the stale-PCR
# class). This is the stable substitute for the REJECTED SwitchSafe auto-trait
# (ownership-model Layer 2): borrowck already forbids borrow-captures (guards
# borrow the token that wait_for consumes); this catches the OWNED smuggles —
# a value whose `let` initializer reads per-CPU state. Syntactic, flat-local
# scope (the translated code's shape); deep-struct smuggling is 4b/MIR.
_CTX_BOUND_SRC = re.compile(
    r"\bKeGetPcr\b|\bKeGetCurrentPrcb\b|\bKeGetPcrArm64\b|\bPcpu\s*<|\bget_pcr\b"
)
# The CPS waits whose trailing-closure captures we inspect.
_CPS_WAITS = {"wait_for", "wait_for_armed"}


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
        # A path into the standard library is never a workspace function, but
        # summaries are keyed by bare name: `core::ptr::read` would inherit any
        # workspace `fn read` that sleeps (sdhc's Host::read does).
        path = src[fn.start_byte : fn.end_byte].decode(errors="replace")
        if path.lstrip(":").split("::", 1)[0] in _STD_ROOTS:
            return None
        name = fn.child_by_field_name("name")
        if name is not None:
            return src[name.start_byte : name.end_byte].decode(errors="replace"), args
    return None


_STD_ROOTS = {"core", "std", "alloc"}


# Not arguments: punctuation, and COMMENTS — tree-sitter makes an inline
# `f(a, 1, /* UserMode */ &mut Obj)` comment a child of the argument list, so
# counting it shifts every positional (or last-arg) index by one.
_NOT_AN_ARG = ("(", ")", ",", "line_comment", "block_comment", "comment")


def _arg_idents(args_node, src: bytes, idx: int):
    """Idents appearing in argument `idx` (0-based) of a call's arguments node."""
    if args_node is None:
        return set()
    real = [c for c in args_node.children if c.type not in _NOT_AN_ARG]
    if idx >= len(real):
        return set()
    text = src[real[idx].start_byte : real[idx].end_byte].decode(errors="replace")
    # Drop any `as <type>` cast tail — `&mut OldIrql as *mut KIRQL` is just the
    # lvalue `OldIrql`; the cast tokens (as, KIRQL, _, mut) are not variables.
    text = re.split(r"\bas\b", text, maxsplit=1)[0]
    # Strip the pointer/reference wrapper noise so `addr_of_mut!(OldIrql)` and
    # `&mut OldIrql` both reduce to {OldIrql}.
    return {i for i in _IDENT.findall(text) if i not in _LVALUE_NOISE}


def _find_functions(root, src: bytes):
    """Yield (name, params, body) for every function_item under root."""
    out = []
    nodes = QueryCursor(_FUNCTIONS).captures(root).get("fn", [])
    for n in sorted(nodes, key=lambda node: node.start_byte):
        body = n.child_by_field_name("body")
        name_node = n.child_by_field_name("name")
        if body is not None and name_node is not None:
            name = src[name_node.start_byte : name_node.end_byte].decode(errors="replace")
            out.append((name, _param_names(n, src), body))
    return out


def _param_names(fn_node, src: bytes):
    """Ordered parameter identifier names of a function_item ('' if opaque)."""
    params = fn_node.child_by_field_name("parameters")
    names = []
    if params is None:
        return names
    for c in params.children:
        if c.type not in ("parameter", "self_parameter"):
            continue
        pat = c.child_by_field_name("pattern")
        txt = src[pat.start_byte : pat.end_byte].decode(errors="replace") if pat else ""
        m = _IDENT.search(txt)
        names.append(m.group(0) if m else "")
    return names


# ---------------------------------------------------------------------------
# Cross-function summaries (Layer 3 phase 4a). One extra walk per function
# collects direct facts + the callee list; the fixpoints then run purely on
# the name-keyed graph (no repeated AST work). Wrapper facts (ret-raise /
# lower-param) are DIRECT-only — a wrapper of a wrapper is not propagated
# (rare in the C-shaped translation; phase 4b territory).
# ---------------------------------------------------------------------------


class FnSummary:
    __slots__ = ("calls", "lower_params", "ret_raise")

    def __init__(self):
        self.calls = set()  # callee bare names
        self.ret_raise = False  # nets a raise handed back via return/tail
        self.lower_params = set()  # own-param indexes used as the OldIrql of a lower


def _summarize_function(name, params, body, src: bytes):
    s = FnSummary()
    held: set[str] = set()
    lowered: set[str] = set()
    returned_idents: set[str] = set()

    def visit(node):
        if node.type == "return_expression":
            returned_idents.update(
                _IDENT.findall(src[node.start_byte : node.end_byte].decode(errors="replace"))
            )
        call = _call_name(node, src)
        if call is not None:
            cname, args = call
            s.calls.add(cname)
            if cname in RAISE_OUTPARAM:
                real = [c for c in (args.children if args else []) if c.type not in _NOT_AN_ARG]
                acquire = True
                if cname in RAISE_LEVELED and real:
                    lvl = src[real[0].start_byte : real[0].end_byte].decode(errors="replace")
                    acquire = bool(_CONST_LEVEL.search(lvl))
                if acquire and real:
                    held.update(_arg_idents(args, src, len(real) - 1))
            elif cname in RAISE_RETURN:
                var = _bound_var(node, src)
                if var:
                    held.add(var)
            elif cname in LOWER_ARG:
                ids = _arg_idents(args, src, 0)
                held.difference_update(ids)
                lowered.update(ids)
            elif cname in RELEASE_ARG2:
                ids = _arg_idents(args, src, 1)
                held.difference_update(ids)
                lowered.update(ids)

    nodes = QueryCursor(_SUMMARY_NODES).captures(body).get("item", [])
    for node in sorted(nodes, key=lambda node: (node.start_byte, -node.end_byte)):
        visit(node)

    # Tail expression of the body counts as a return.
    real_tail = [c for c in body.children if c.type not in ("{", "}")]
    if real_tail and real_tail[-1].type not in ("expression_statement", "let_declaration"):
        t = real_tail[-1]
        returned_idents.update(
            _IDENT.findall(src[t.start_byte : t.end_byte].decode(errors="replace"))
        )

    # RAISE-WRAPPER: a net-held var handed back to the caller.
    s.ret_raise = bool(held & returned_idents)
    # LOWER-WRAPPER: own params consumed as an OldIrql.
    for i, p in enumerate(params):
        if p and p in lowered:
            s.lower_params.add(i)
    return s


# Propagation depth for may-sleep: 1 = only direct calls to the seed wait
# APIs; 2 (default) = plus calls through one Rust wrapper layer. UNCAPPED
# propagation is useless here — everything that touches the dispatcher
# (KeSetEvent → KiExitDispatcher → the swap) becomes "may-sleep" even though
# it never parks the CALLER, and the finding count explodes (measured: 13.6K
# may-sleep functions, 762 findings at fixpoint vs a reviewable set at 2).
SLEEP_DEPTH = 2

_RAISE_LOWER_TOKEN = re.compile(
    ("|".join(re.escape(n) for n in (RAISE_OUTPARAM | RAISE_RETURN | LOWER_ARG | RELEASE_ARG2)))
    .join((r"\b(?:", r")\s*\("))
    .encode()
)


def _name_call_re(names):
    return re.compile(
        ("|".join(re.escape(n) for n in sorted(names))).join((r"\b(?:", r")\s*\(")).encode()
    )


_CALL_TOKEN = re.compile(rb"\b([A-Za-z_]\w*)\s*\(")


@source_facts()
def call_tokens(src: bytes):
    # Same textual edges as _name_call_re, but index ONCE rather than matching
    # a growing alternation of thousands of names against every byte each ring.
    return frozenset(m.group(1).decode() for m in _CALL_TOKEN.finditer(src))


@source_facts()
def summary_facts(src: bytes):
    tree = _PARSER.parse(src)
    wrappers = bool(_RAISE_LOWER_TOKEN.search(src))
    out = []
    for name, params, body in _find_functions(tree.root_node, src):
        summary = _summarize_function(name, params, body, src) if wrappers else FnSummary()
        out.append(
            (
                name,
                call_tokens(src[body.start_byte : body.end_byte]),
                summary.ret_raise,
                summary.lower_params,
            )
        )
    return out


def build_summaries(files, tree_cache, sleep_depth=None):
    """files: [Path]. Returns (may_sleep, ret_raise, lower_params), name-keyed;
    collisions merge conservatively (union). The textual call index and wrapper
    facts are content-addressed; global propagation always reruns. tree_cache is
    retained as a compatibility argument for existing callers."""
    if sleep_depth is None:
        sleep_depth = SLEEP_DEPTH
    srcs = {f: f.read_bytes() for f in files}
    calls = {f: call_tokens(src) for f, src in srcs.items()}

    # Wrapper facts: only files that textually touch a raise/lower builtin.
    ret_raise: set[str] = set()
    lower_params: dict[str, set[int]] = {}
    for src in srcs.values():
        if not _RAISE_LOWER_TOKEN.search(src):
            continue
        for name, _body, raises, lowers in summary_facts(src):
            if raises:
                ret_raise.add(name)
            if lowers:
                lower_params.setdefault(name, set()).update(lowers)
    ret_raise -= RAISE_RETURN
    for n in LOWER_ARG | RELEASE_ARG2:
        lower_params.pop(n, None)

    # may-sleep: breadth-first over TEXTUAL caller edges, depth-capped. Each
    # ring only parses the files that mention a frontier name as a call.
    may_sleep = set(SLEEP_SEEDS)
    frontier = set(SLEEP_SEEDS)
    for _depth in range(sleep_depth):
        if not frontier:
            break
        previous = frontier
        frontier = set()
        for f, src in srcs.items():
            if not (previous & calls[f]):
                continue
            for name, body, _raises, _lowers in summary_facts(src):
                if name in may_sleep or name in SLEEP_CONDITIONAL:
                    continue
                if previous & body:
                    may_sleep.add(name)
                    frontier.add(name)

    return may_sleep, ret_raise, lower_params


def _collect_lets(body, src: bytes):
    """(name, value_text, start_byte, end_byte) for every let under `body`."""
    out = []

    def rec(n):
        if n.type == "let_declaration":
            pat = n.child_by_field_name("pattern")
            val = n.child_by_field_name("value")
            if pat is not None and val is not None:
                m = _IDENT.search(src[pat.start_byte : pat.end_byte].decode(errors="replace"))
                if m:
                    out.append(
                        (
                            m.group(0),
                            src[val.start_byte : val.end_byte].decode(errors="replace"),
                            n.start_byte,
                            n.end_byte,
                        )
                    )
        for c in n.children:
            rec(c)

    rec(body)
    return out


def _scan_function(body, src: bytes, lines, summaries=None):
    """Linear held-set walk over one function body. Returns findings.
    `summaries` = (may_sleep, ret_raise, lower_params) from build_summaries;
    None disables the cross-function checks (unit-test convenience)."""
    may_sleep, ret_raise, lower_params = summaries or (frozenset(), frozenset(), {})
    findings = []
    # var -> True if the raise makes a wait ILLEGAL (spinlock / raise above
    # APC_LEVEL). A KeRaiseIrql(APC_LEVEL, ..) hold is leak-checked but NOT
    # sleep-checked — NT's real rule is waits ≤ APC_LEVEL are legal.
    held: dict[str, bool] = {}
    lowers_internally = [False]
    fn_lets = [None]  # lazy _collect_lets cache (only wait_for sites need it)

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
                leaked = held.keys() - ret_ids
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
                real = [c for c in (args.children if args else []) if c.type not in _NOT_AN_ARG]
                acquire = True
                if name in RAISE_LEVELED and real:
                    lvl = src[real[0].start_byte : real[0].end_byte].decode(errors="replace")
                    # Restore-to-saved (level is a variable, not a *_LEVEL const).
                    acquire = bool(_CONST_LEVEL.search(lvl))
                if acquire and real:
                    # KeRaiseIrql(APC_LEVEL, ..) holds are wait-LEGAL; a
                    # spinlock acquire or a raise above APC is not.
                    wait_illegal = name in RAISE_LOCK or (
                        name in RAISE_LEVELED
                        and "APC_LEVEL"
                        not in src[real[0].start_byte : real[0].end_byte].decode(errors="replace")
                    )
                    for i in _arg_idents(args, src, len(real) - 1):
                        held[i] = wait_illegal
            elif name in RAISE_RETURN or name in ret_raise:
                # value bound by an enclosing let/assignment — resolve upward.
                var = _bound_var(node, src)
                if var:
                    held[var] = True  # DPC/SYNCH-level raises and lock wrappers
            elif name in LOWER_ARG:
                for i in _arg_idents(args, src, 0):
                    held.pop(i, None)
                lowers_internally[0] = True
            elif name in RELEASE_ARG2:
                for i in _arg_idents(args, src, 1):
                    held.pop(i, None)
                lowers_internally[0] = True
            elif name in lower_params:
                # LOWER-WRAPPER: the callee lowers with these argument slots.
                for idx in lower_params[name]:
                    for i in _arg_idents(args, src, idx):
                        held.pop(i, None)
                lowers_internally[0] = True

            # SLEEP-WHILE-HELD: a (transitively) parking call under a live
            # spinlock / above-APC raise — the static SLEEP_WITH_SPINLOCK.
            blocking = sorted(v for v, illegal in held.items() if illegal)
            if name in may_sleep and blocking and not suppressed(node):
                findings.append(
                    (
                        line_of(node),
                        "IRQL_SLEEP_WHILE_HELD",
                        f"call to may-sleep `{name}` while holding "
                        f"{', '.join(blocking)} — a wait above APC_LEVEL "
                        f"(park with a live raise/spinlock)",
                    )
                )

            # CTX-CAPTURE-ACROSS-WAIT (stable Layer-2 substitute): the CPS
            # continuation captures a local minted from per-CPU state — stale
            # after the switch resumes (elsewhere / after preemption).
            if name in _CPS_WAITS and args is not None and not suppressed(node):
                real = [c for c in args.children if c.type not in _NOT_AN_ARG]
                if real and real[-1].type == "closure_expression":
                    k = real[-1]
                    k_txt = src[k.start_byte : k.end_byte].decode(errors="replace")
                    k_ids = set(_IDENT.findall(k_txt))
                    if fn_lets[0] is None:
                        fn_lets[0] = _collect_lets(body, src)
                    for lname, lval, lstart, _lend in fn_lets[0]:
                        # declared OUTSIDE the closure, used inside, minted
                        # from per-CPU state.
                        if (
                            lname in k_ids
                            and not (k.start_byte <= lstart < k.end_byte)
                            and _CTX_BOUND_SRC.search(lval)
                        ):
                            findings.append(
                                (
                                    line_of(node),
                                    "CTX_CAPTURE_ACROSS_WAIT",
                                    f"`{name}` continuation captures `{lname}` "
                                    f"(minted from per-CPU state) — stale after "
                                    f"the switch; re-derive it inside the "
                                    f"continuation",
                                )
                            )

        for c in node.children:
            visit(c)

    visit(body)
    # Only trust the HELD-SET analysis for functions that balance internally;
    # a raise-only function is the cross-function API contract, not a leak.
    # CTX_CAPTURE findings don't depend on the held set — always keep them.
    if lowers_internally[0]:
        return findings
    return [f for f in findings if f[1] == "CTX_CAPTURE_ACROSS_WAIT"]


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


def scan_source(src: bytes, summaries=None):
    may_sleep, ret_raise, lower = summaries or (set(), set(), {})
    return scan_facts(
        src,
        tuple(sorted(may_sleep)),
        tuple(sorted(ret_raise)),
        tuple((name, tuple(sorted(indices))) for name, indices in sorted(lower.items())),
    )


@source_facts()
def scan_facts(src, may_sleep, ret_raise, lower):
    summaries = (set(may_sleep), set(ret_raise), {name: set(indices) for name, indices in lower})
    tree = _PARSER.parse(src)
    lines = src.decode(errors="replace").splitlines()
    findings = []
    for _name, _params, body in _find_functions(tree.root_node, src):
        findings.extend(_scan_function(body, src, lines, summaries))
    findings.sort()
    return findings


def _iter_files(paths):
    for p in paths:
        p = Path(p)
        if p.is_dir():
            for f in source_files(p):
                if f.name.startswith("bindgen_") or f.name == "build.rs":
                    continue
                yield f
        elif p.suffix == ".rs":
            yield p


def main(argv, *, summary_root=None):
    # Injectable workspace for self-contained test fixtures. The CLI deliberately
    # exposes no override: production scoped scans still need whole-OS facts.
    workspace = RUST_WS if summary_root is None else Path(summary_root)
    args = [a for a in argv if not a.startswith("--")]
    lint = "--lint" in argv
    dump_summaries = "--summaries" in argv
    roots = [Path(a) for a in args] if args else [workspace]

    # Summary database: ALWAYS the whole workspace (call-graph facts are
    # global), even when findings are limited to the named files.
    tree_cache: dict = {}
    ws_files = sorted(set(_iter_files([workspace])))
    summaries = build_summaries(ws_files, tree_cache)

    if dump_summaries:
        may_sleep, ret_raise, lower_params = summaries
        print(f"may-sleep ({len(may_sleep)}):")
        for n in sorted(may_sleep):
            print(f"  {n}")
        print(f"raise-wrappers ({len(ret_raise)}):")
        for n in sorted(ret_raise):
            print(f"  {n}")
        print(f"lower-wrappers ({len(lower_params)}):")
        for n in sorted(lower_params):
            print(f"  {n} (args {sorted(lower_params[n])})")
        return 0

    # Findings need a non-empty held set, which needs a raise: skip files
    # that neither touch a raise/lower builtin nor call a raise-wrapper.
    _, ret_raise, _ = summaries
    wrapper_pat = _name_call_re(ret_raise) if ret_raise else None

    total = 0
    for f in sorted(set(_iter_files(roots))):
        try:
            src = f.read_bytes()
            if not (_RAISE_LOWER_TOKEN.search(src) or (wrapper_pat and wrapper_pat.search(src))):
                continue
            findings = scan_source(src, summaries)
        except Exception as e:
            raise RuntimeError(f"cannot analyze {f}: {e}") from e
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
