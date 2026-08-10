#!/usr/bin/env python3
"""
scan_ob_refcount.py — find leaked Ob object references on early-exit paths
(AST-aware). Sibling of scan_irql_balance.py (same held-set engine, field
manual §9.26), pointed at the NEXT lifecycle class: `ObReferenceObject*` /
`ObDereferenceObject` balance.

A leaked reference never crashes a smoke boot — it's a slow death: the object
(and its backing pool) can never be freed, so hours-in the system exhausts
pool. No gate will ever see it; only static analysis (or a very long soak)
can.

Model (intraprocedural, per function body — deliberately HIGH-PRECISION /
low-recall, like the IRQL scanner that shipped with exactly one real find):
  * ACQUIRE: `ObReferenceObjectByHandle(.., &mut Obj, ..)` (out-param),
    `ObReferenceObject(p)` / `ObfReferenceObject(p)` /
    `ObReferenceObjectByPointer(p, ..)` (refcount++ on existing pointer)
    → the object var becomes HELD, paired with the bound STATUS var (if any).
  * RELEASE: `ObDereferenceObject(p)` / `Obf..` / `..DeferDelete` → released.
  * ESCAPE (ownership left this function — NOT a leak):
      - the var appears in a `return` expression / tail expr;
      - the var is stored through a deref/field/index lvalue
        (`*OutObject = obj`, `(*Fcb).FileObject = obj`);
      - the var is passed to ANY other call (conservative: a use may be a
        transfer — IoQueueWorkItem-style — so we drop it rather than guess).
  * STATUS-GUARD: an early `return` inside an `if` whose condition mentions
    the acquire's paired status var is the failure/success check — the
    reference's existence is conditional there, so it is exempt (both arms;
    ambiguity is accepted for precision).
  * Only functions that ObDereference SOMETHING internally are checked — a
    reference-and-return function is the API contract, not a leak.

Findings gate on NEW-vs-baseline (tools/lint-baseline/scan_ob_refcount.txt).
Reduce a verified false positive with:
    return;  // nolint: ob-refcount (reason)

Usage:
    python3 tools/scan_ob_refcount.py                 # sweep the workspace
    python3 tools/scan_ob_refcount.py path/to/file.rs # specific files
    python3 tools/scan_ob_refcount.py --lint          # terse (baseline gate)
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

# Acquires whose OBJECT is an out-param at a FIXED argument index (0-based).
ACQ_OUT_IDX = {
    "ObReferenceObjectByHandle": 4,  # (Handle, Access, Type, Mode, *Object, *HandleInfo)
    "ObReferenceObjectByName": 7,  # (Path, Attr, AccessState, Access, Type, Mode, Ctx, *Object)
    "ObReferenceFileObjectForWrite": 2,  # (Handle, Mode, *FileObject, *HandleInfo)
}
# Acquires that bump the refcount of an EXISTING pointer (arg 0).
ACQ_ARG0 = {
    "ObReferenceObject",
    "ObfReferenceObject",
    "ObReferenceObjectByPointer",
    "ObReferenceObjectSafe",
}
DEREF_ARG0 = {
    "ObDereferenceObject",
    "ObfDereferenceObject",
    "ObDereferenceObjectEx",
    "ObDereferenceObjectDeferDelete",
    "ObDereferenceObjectDeferDeleteWithTag",
}

_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
_LVALUE_NOISE = {"core", "ptr", "addr_of_mut", "addr_of", "mut", "raw", "self", "crate"}


def _nolint(line_text: str) -> bool:
    return bool(
        re.search(r"//\s*nolint(?::\s*ob-refcount\b|\s*$|\s*\()", line_text)
        or re.search(r"//\s*nolint\b(?!\s*:)", line_text)
    )


def _call_name(node, src: bytes):
    if node.type != "call_expression":
        return None
    fn = node.child_by_field_name("function")
    if fn is None:
        return None
    args = node.child_by_field_name("arguments")
    if fn.type == "identifier":
        return src[fn.start_byte : fn.end_byte].decode(errors="replace"), args
    if fn.type == "scoped_identifier":
        name = fn.child_by_field_name("name")
        if name is not None:
            return src[name.start_byte : name.end_byte].decode(errors="replace"), args
    return None


# Not arguments: punctuation, and COMMENTS — tree-sitter makes an inline
# `f(a, 1, /* UserMode */ &mut Obj)` comment a child of the argument list, so
# counting it shifts every FIXED out-param index by one. That mis-tracked
# fastfat's ObReferenceObjectByHandle onto the comment's own text and reported
# a leak of a variable named `UserMode`.
_NOT_AN_ARG = ("(", ")", ",", "line_comment", "block_comment", "comment")


def _real_args(args_node):
    if args_node is None:
        return []
    return [c for c in args_node.children if c.type not in _NOT_AN_ARG]


def _arg_idents(args_node, src: bytes, idx: int):
    real = _real_args(args_node)
    if idx >= len(real):
        return set()
    text = src[real[idx].start_byte : real[idx].end_byte].decode(errors="replace")
    text = re.split(r"\bas\b", text, maxsplit=1)[0]
    return {i for i in _IDENT.findall(text) if i not in _LVALUE_NOISE}


def _bound_var(call_node, src: bytes):
    """For `let v = Call(..)` / `v = Call(..)`, the bound PLAIN-LOCAL name."""
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
            if _IDENT.fullmatch(ltxt):
                return ltxt
    return None


def _enclosing_condition_idents(node, src: bytes, stop):
    """Idents in every `if`/`while`/`match` condition between node and stop —
    the guards an early return sits under (for the STATUS-GUARD exemption)."""
    ids: set[str] = set()
    p = node.parent
    while p is not None and p is not stop:
        if p.type in ("if_expression", "while_expression"):
            cond = p.child_by_field_name("condition")
            if cond is not None:
                ids |= set(
                    _IDENT.findall(src[cond.start_byte : cond.end_byte].decode(errors="replace"))
                )
        elif p.type == "match_expression":
            val = p.child_by_field_name("value")
            if val is not None:
                ids |= set(
                    _IDENT.findall(src[val.start_byte : val.end_byte].decode(errors="replace"))
                )
        p = p.parent
    return ids


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


def _enclosing_if_with_condition(call_node, body):
    """The nearest if/while ancestor whose CONDITION contains this call — the
    `if ObReferenceObjectByHandle(..) != 0 { return; }` shape: every exit
    inside that if_expression is the acquire's own success/failure handling."""
    n = call_node
    p = call_node.parent
    while p is not None and p is not body:
        if p.type in ("if_expression", "while_expression"):
            cond = p.child_by_field_name("condition")
            if cond is not None and cond.start_byte <= n.start_byte <= cond.end_byte:
                return p
        p = p.parent
    return None


def _within(node, container):
    return (
        container is not None
        and container.start_byte <= node.start_byte
        and node.end_byte <= container.end_byte
    )


def _scan_function(body, src: bytes, lines):
    findings = []
    # canonical obj var -> (status var | None, guard if_expression | None)
    held: dict[str, tuple] = {}
    alias: dict[str, str] = {}  # alias var -> canonical held var
    derefs_internally = [False]

    def canon(v):
        return alias.get(v, v)

    def drop(v):
        # A name can be BOTH a held root (out-param in one branch) and an alias
        # (assigned from another ref in a sibling branch) — clear both.
        for r in {v, canon(v)}:
            held.pop(r, None)
            for a in [k for k, t in alias.items() if t == r]:
                del alias[a]
        alias.pop(v, None)

    def line_of(node):
        return node.start_point[0] + 1

    def suppressed(node):
        ln = node.start_point[0]
        return ln < len(lines) and _nolint(lines[ln])

    def visit(node):
        if node.type in ("return_expression", "try_expression"):
            if held and not suppressed(node):
                ret_txt = src[node.start_byte : node.end_byte].decode(errors="replace")
                ret_ids = {canon(i) for i in _IDENT.findall(ret_txt)}
                guards = _enclosing_condition_idents(node, src, body)
                leaked = []
                for var, (status, guard_if) in held.items():
                    if var in ret_ids:
                        continue  # ownership transferred to caller
                    if status is not None and status in guards:
                        continue  # inside the acquire's success/failure check
                    if _within(node, guard_if):
                        continue  # acquire was IN this if's condition
                    leaked.append(var)
                if leaked:
                    kind = (
                        "OBREF_LEAK_RETURN"
                        if node.type == "return_expression"
                        else "OBREF_LEAK_TRY"
                    )
                    findings.append(
                        (
                            line_of(node),
                            kind,
                            f"early exit while holding Ob reference(s) "
                            f"{', '.join(sorted(leaked))} — not dereferenced on this "
                            f"path (object + backing pool can never be freed)",
                        )
                    )
            for c in node.children:
                visit(c)
            return

        call = _call_name(node, src)
        if call is not None:
            name, args = call
            if name in ACQ_OUT_IDX or name in ACQ_ARG0:
                idx = ACQ_OUT_IDX.get(name, 0)
                ids = _arg_idents(args, src, idx)
                # ByHandle/ByName/ByPointer return NTSTATUS — the reference is
                # conditional on it. Two guard shapes: bound status var, or the
                # call sitting directly in an if/while condition.
                # ByHandle/ByName return NTSTATUS; ByPointer can fail the type
                # check; Safe returns a success BOOLEAN — all conditional.
                statusful = name in ACQ_OUT_IDX or name in (
                    "ObReferenceObjectByPointer",
                    "ObReferenceObjectSafe",
                )
                status = _bound_var(node, src) if statusful else None
                guard_if = _enclosing_if_with_condition(node, body) if statusful else None
                # Only a SINGLE plain ident is intraprocedurally trackable; a
                # field expression (`&mut (*Port).__bindgen_anon_1.ClientPort`)
                # stores the ref into a structure — escaped at birth.
                if len(ids) == 1:
                    held[next(iter(ids))] = (status, guard_if)
            elif name in DEREF_ARG0:
                for i in _arg_idents(args, src, 0):
                    drop(i)
                derefs_internally[0] = True
            else:
                # ESCAPE: a held var passed to ANY other call may be a transfer
                # (work item, list insert, handle-table store) — drop it rather
                # than guess. High precision beats recall here.
                if held:
                    for idx in range(len(_real_args(args))):
                        for i in _arg_idents(args, src, idx):
                            if canon(i) in held:
                                drop(i)

        # ALIAS: `Process = TargetProcess;` / `let fo = obj as *mut T;` — the
        # same reference under a second name; releasing/returning EITHER counts.
        if node.type in ("assignment_expression", "let_declaration") and held:
            if node.type == "assignment_expression":
                left = node.child_by_field_name("left")
                right = node.child_by_field_name("right")
            else:
                left = node.child_by_field_name("pattern")
                right = node.child_by_field_name("value")
            if left is not None and right is not None:
                ltxt = src[left.start_byte : left.end_byte].decode(errors="replace").strip()
                rtxt = src[right.start_byte : right.end_byte].decode(errors="replace")
                rtxt = re.split(r"\bas\b", rtxt, maxsplit=1)[0].strip()
                lm = _IDENT.fullmatch(ltxt.removeprefix("mut ").strip())
                if _IDENT.fullmatch(rtxt) and canon(rtxt) in held:
                    if lm:
                        alias[lm.group(0)] = canon(rtxt)
                    else:
                        # stored through a deref/field/index lvalue → escaped
                        drop(rtxt)
                elif lm and canon(lm.group(0)) in held:
                    # OVERWRITE: a held var reassigned from something that is
                    # not the held object no longer names it — kill the hold
                    # (`connection_port = (*port).ConnectionPort` after an
                    # earlier acquire under the same name).
                    drop(lm.group(0))

        for c in node.children:
            visit(c)

    visit(body)
    return findings if derefs_internally[0] else []


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
        print(f"\n{total} Ob-refcount finding(s)")
    return 1 if total else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
