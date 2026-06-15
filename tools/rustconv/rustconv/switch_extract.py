"""Win32k pretranslation — tree-sitter switch-dispatcher de-monstering.

`SpiGetSet` (1029 LOC) / `IntDefWindowProc` (727) are giant *single
functions* whose body is essentially one huge `switch`. `translate`
mode chunks function-by-function, so a 1000-line function is still one
oversized prompt — it WILL bite rustconv. But a thin dispatch `switch`
is the *best-case* AST for a real parser: each arm is independent. So,
unlike arbitrary logic extraction (fragile locals→params dataflow),
this is tree-sitter-tractable and per-arm *provably* safe.

Conservative v1 — extract only the proven-safe arms, leave the rest
inline (Pass-0 discipline: never gamble):

A case-group (fall-through labels + the body-bearing case) is SAFE iff
  * the body unconditionally leaves via `return` (last top-level stmt
    is a return_statement) — so the extracted helper `return`s the same;
  * NO `break` / `goto` / `continue` / `_SEH2_*` anywhere in the body
    (break/goto would change meaning once moved out of the switch);
  * every `identifier` it uses is a function PARAMETER, a file-scope
    symbol (global / other fn / macro / enum constant), or a local
    declared *within the group's own body* — i.e. it references no
    function-scope local that lives outside it.
SAFE → `static <ret> <fn>_<tag>(<used params>) { <body> }` + the arm
becomes `case L1: case L2: …: return <fn>_<tag>(<args>);`.
UNSAFE / `default` / break-terminated → left byte-verbatim.

Net: a ~1000-line monster → a small regular dispatch shell + N tiny
independent helpers (same TU/file, `static`, zero CMake change) — each
a trivial rustconv unit. Guards: every original case label still
present & routed (no drop/dup); output re-parses clean; build + nm
(no pre-existing symbol lost) + smoke gate.

CLI: `python -m rustconv switch-extract <file> --function NAME
      [--apply] [--oracle]`.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

from tree_sitter import Parser

from .scanner import C_LANG

_KW = {
    "if",
    "else",
    "for",
    "while",
    "do",
    "switch",
    "case",
    "default",
    "return",
    "sizeof",
    "break",
    "continue",
    "goto",
    "const",
    "static",
    "struct",
    "union",
    "enum",
    "typedef",
    "void",
    "int",
    "unsigned",
    "signed",
    "char",
    "short",
    "long",
    "float",
    "double",
}


@dataclass
class Group:
    labels: list[str]  # case label source text (or "default")
    body_start: int  # byte offset of first body stmt
    body_end: int  # byte offset after last body stmt
    span_start: int  # byte offset of the whole case-group
    span_end: int
    safe: bool
    reason: str
    used_params: list[str] = field(default_factory=list)
    tag: str = ""


def _txt(src: bytes, n) -> str:
    return src[n.start_byte : n.end_byte].decode("utf8", "replace")


def _find_function(root, src: bytes, name: str):
    out = []

    def walk(n):
        if n.type == "function_definition":
            d = n.child_by_field_name("declarator")
            # descend to the function_declarator's identifier
            q = d
            while q is not None and q.type != "identifier":
                q = q.child_by_field_name("declarator") or (q.children[0] if q.children else None)
            if q is not None and _txt(src, q) == name:
                out.append(n)
        for c in n.children:
            walk(c)

    walk(root)
    return out[0] if out else None


def _idents(n, src: bytes):
    acc = []

    def walk(m):
        if m.type == "identifier":
            acc.append(_txt(src, m))
        for c in m.children:
            walk(c)

    walk(n)
    return acc


def _has_type(n, types: set[str]) -> bool:
    if n.type in types:
        return True
    return any(_has_type(c, types) for c in n.children)


@dataclass
class ExtractResult:
    path: Path
    function: str
    groups: list[Group]
    new_text: str
    note: str = ""


def analyze(path: Path, func: str) -> ExtractResult:
    src = path.read_bytes()
    tree = Parser(C_LANG).parse(src)
    fn = _find_function(tree.root_node, src, func)
    if fn is None:
        return ExtractResult(path, func, [], "", f"function {func} not found")
    body = fn.child_by_field_name("body")
    # the switch must be the (a) statement of the function body
    sw = next((c for c in body.children if c.type == "switch_statement"), None)
    if sw is None:
        return ExtractResult(path, func, [], "", "no top-level switch in function body")
    ret_type = _txt(src, fn.child_by_field_name("type")).strip()

    # function-scope params + locals (locals = declarations directly in
    # the function body, NOT inside the switch).
    params = []
    pl = fn.child_by_field_name("declarator")
    for d in pl.children if pl else []:
        if d.type == "parameter_list":
            for p in d.children:
                if p.type == "parameter_declaration":
                    pd = p.child_by_field_name("declarator")
                    ids = _idents(pd, src) if pd else []
                    if ids:
                        params.append(ids[-1])
    fn_locals = set()
    for c in body.children:
        if c.type == "declaration":
            for idn in _idents(c, src):
                fn_locals.add(idn)

    swbody = next(c for c in sw.children if c.type == "compound_statement")
    cases = [c for c in swbody.children if c.type == "case_statement"]

    groups: list[Group] = []
    pending_labels: list[str] = []
    pending_start = None
    for cs in cases:
        if pending_start is None:
            pending_start = cs.start_byte
        # label text: child after 'case' (value) or 'default'
        is_default = any(ch.type == "default" for ch in cs.children)
        label = None
        if not is_default:
            for ch in cs.children:
                if (
                    ch.type not in ("case", ":")
                    and ch.prev_sibling
                    and ch.prev_sibling.type == "case"
                ):
                    label = _txt(src, ch)
                    break
        # body statements = children after the ':'
        colon = next((i for i, ch in enumerate(cs.children) if ch.type == ":"), None)
        stmts = cs.children[colon + 1 :] if colon is not None else []
        pending_labels.append("default" if is_default else (label or "?"))
        if not stmts:
            continue  # pure fall-through label → accumulate
        # this case_statement carries the group body
        b0, b1 = stmts[0].start_byte, stmts[-1].end_byte
        g = Group(
            labels=pending_labels[:],
            body_start=b0,
            body_end=b1,
            span_start=pending_start,
            span_end=cs.end_byte,
            safe=False,
            reason="",
        )
        # ---- safety classification ----
        last = stmts[-1]
        bad_kw = any(
            _has_type(s, {"break_statement", "goto_statement", "continue_statement"}) for s in stmts
        )
        seh = any("_SEH2_" in _txt(src, s) for s in stmts)
        returns = last.type == "return_statement"
        if "default" in g.labels:
            g.reason = "default arm — left inline"
        elif bad_kw:
            g.reason = "break/goto/continue in body — left inline"
        elif seh:
            g.reason = "_SEH2_ in body — left inline"
        elif not returns:
            g.reason = "body does not end in return — left inline"
        else:
            # locals declared within this body are fine; identifiers must
            # be param / file-scope / body-local only.
            body_local = set()
            for s in stmts:
                for dn in [s] if s.type == "declaration" else []:
                    body_local |= set(_idents(dn, src))

                def collect_decls(m, body_local=body_local):
                    if m.type == "declaration":
                        for i in _idents(m, src):
                            body_local.add(i)
                    for c in m.children:
                        collect_decls(c)

                collect_decls(s)
            used = set()
            leak = None
            for s in stmts:
                for idn in _idents(s, src):
                    if idn in _KW:
                        continue
                    if idn in params:
                        used.add(idn)
                    elif idn in fn_locals and idn not in body_local:
                        leak = idn
                        break
                if leak:
                    break
            if leak:
                g.reason = f"uses function-local '{leak}' — left inline"
            else:
                g.safe = True
                g.used_params = [p for p in params if p in used]
                first = next((lbl for lbl in g.labels if lbl != "default"), g.labels[0])
                g.tag = re.sub(r"[^A-Za-z0-9_]", "_", first).strip("_")
                g.reason = "safe"
        groups.append(g)
        pending_labels = []
        pending_start = None

    safe = [g for g in groups if g.safe]
    if not safe:
        return ExtractResult(path, func, groups, "", f"0/{len(groups)} groups extractable")

    # ---- emit: helpers + rewritten arms — ALL splicing in BYTES.
    # tree-sitter offsets are *byte* offsets; splicing into a decoded
    # str desyncs after the first non-ASCII char (copyright comments).
    helpers: list[bytes] = []
    edits: list[tuple[int, int, bytes]] = []
    for g in safe:
        bodytxt = src[g.body_start : g.body_end].decode("utf8", "replace")
        # wrap bare (non-brace) bodies so multi-stmt is well-formed
        if not bodytxt.lstrip().startswith("{"):
            bodytxt = "{\n        " + bodytxt + "\n    }"
        sig_params = ", ".join(f"{_param_type(src, fn, p)} {p}" for p in g.used_params) or "void"
        helpers.append((f"static {ret_type}\n{func}_{g.tag}({sig_params})\n{bodytxt}\n").encode())
        labels = "\n        ".join(f"case {lbl}:" for lbl in g.labels)
        call = f"{func}_{g.tag}({', '.join(g.used_params)})"
        edits.append((g.span_start, g.span_end, f"{labels}\n            return {call};".encode()))

    new = src
    for s, e, rep in sorted(edits, key=lambda x: -x[0]):
        new = new[:s] + rep + new[e:]
    insert_at = fn.start_byte
    # map fn.start_byte (bytes) to char index — ASCII source, equal.
    block = (
        (
            "/* pretranslation: switch arms extracted to helpers so "
            f"rustconv gets tiny units instead of one {func} monster. "
            "See docs/engineering/win32k-pretranslation-audit.md */\n"
        ).encode()
        + b"\n".join(helpers)
        + b"\n"
    )
    new = new[:insert_at] + block + new[insert_at:]

    # ---- oracle: every original case label still present & re-parses ----
    def labels_of(b: bytes) -> list[str]:
        s = b.decode("utf8", "replace")
        return sorted(
            re.findall(r"\bcase\s+([A-Za-z_]\w*)\s*:", s)
            + (["default"] if re.search(r"\bdefault\s*:", s) else [])
        )

    if labels_of(src) != labels_of(new):
        return ExtractResult(path, func, groups, "", "ORACLE FAIL: case-label set changed")

    # tree-sitter's lenient C parser emits ERROR/missing nodes on valid
    # win32k C (unresolved macros: APIENTRY/SAL/#if). Absolute
    # zero-errors is unachievable here, so the invariant is *differential*
    # — the transform must not INTRODUCE new parse errors. (Real
    # correctness gate is build + nm + smoke, not tree-sitter.)
    def _errs(b: bytes) -> int:
        t = Parser(C_LANG).parse(b)
        n = [0]

        def w(x):
            if x.type == "ERROR" or x.is_missing:
                n[0] += 1
            for c in x.children:
                w(c)

        w(t.root_node)
        return n[0]

    if _errs(new) > _errs(src):
        return ExtractResult(
            path,
            func,
            groups,
            "",
            f"ORACLE FAIL: transform introduced new parse errors ({_errs(src)}→{_errs(new)})",
        )
    return ExtractResult(
        path,
        func,
        groups,
        new.decode("utf8", "replace"),
        f"{len(safe)}/{len(groups)} arms extracted, {len(groups) - len(safe)} left inline",
    )


def _param_type(src: bytes, fn, pname: str) -> str:
    pl = fn.child_by_field_name("declarator")
    for d in pl.children if pl else []:
        if d.type != "parameter_list":
            continue
        for p in d.children:
            if p.type != "parameter_declaration":
                continue
            pd = p.child_by_field_name("declarator")
            if pd and _idents(pd, src) and _idents(pd, src)[-1] == pname:
                t = p.child_by_field_name("type")
                pre = src[t.start_byte : pd.start_byte].decode("utf8", "replace").strip()
                return pre or "void"
    return "void"


def run_oracle(path: Path, func: str) -> int:
    r = analyze(path, func)
    print(f"{path.name}:{func} — {r.note}")
    ok = bool(r.new_text) and "FAIL" not in r.note
    for g in r.groups[:6]:
        print(f"  [{'SAFE' if g.safe else 'inline'}] {'/'.join(g.labels[:3])}… → {g.reason}")
    print("ORACLE PASS" if ok else "ORACLE FAIL")
    return 0 if ok else 1
