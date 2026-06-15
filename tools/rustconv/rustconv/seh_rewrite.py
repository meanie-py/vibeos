"""Win32k pretranslation Pass 1 — mechanical probe-site lowering.

Deterministic (no LLM, ~$0) rewrite of the category-(a) PROBE_LOWER SEH
sites Pass 0 (`seh_inventory`) identified. Transforms

    _SEH2_TRY {
        ProbeForRead(p, n, a);
        <body that touches p>
    } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
        <handler>
    } _SEH2_END;

into

    Status = SafeProbeForRead(p, n, a);
    if (!NT_SUCCESS(Status)) { <handler'> }
    <body that touches p>

against the C `safe_user_io` shim the winsta.c spike proved (builds +
links into win32k.sys). Only Pass-0 category-'a' sites are touched.

Handler shapes (the five from the spike) all reduce to one rule — turn
the EXCEPT body into the probe-failure branch:

  * `_SEH2_GetExceptionCode()` → `Status` (already set by Safe*)
  * `_SEH2_YIELD(stmt);`       → `stmt;`
  * `NOTHING;` / empty         → swallow form `if (NT_SUCCESS(Safe*)) {body}`
  * ERR()/SetLastNtError/return/goto → preserved verbatim in the branch

SAFETY (audit risk #1 — multi/conditional probe mis-rewrite): v1 rewrites
ONLY a single, unconditional, generic 3-arg ProbeForRead/Write. Anything
else — >1 probe, a brace-less `if/for/while` guarding the probe, typed
probe, nested SEH, `_SEH2_LEAVE`, FINALLY, Status-not-in-scope (non-
swallow) — is left verbatim and reported for manual handling (this is
exactly what the spike did by hand for winsta L1242).

`--oracle`: on pre-spike winsta.c, rewrite the 4 clean (a) sites
(L709/L825/L2157/L2190), flag the multi-probe (a) site (L1242) manual,
and leave the 2 (e) + 1 (d) untouched.
"""

from __future__ import annotations

import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

from .seh_classifier import _find_brace_matched_block
from .seh_inventory import Category, _strip_literals, classify_span

RE_TRY = re.compile(r"\b_SEH2_TRY\b")
RE_PROBE_STMT = re.compile(r"\bProbeFor(Read|Write)\s*\(")
RE_PROBE_ANY = re.compile(r"\bProbeFor(Read|Write)\w*\s*\(")
RE_GETCODE = re.compile(r"_SEH2_GetExceptionCode\s*\(\s*\)")
RE_YIELD_OPEN = re.compile(r"_SEH2_YIELD\s*\(")
RE_BRACELESS_GUARD = re.compile(r"\b(if|for|while)\s*\([^{;]*\)\s*ProbeFor(Read|Write)\b")
RE_INCLUDE_W32K = re.compile(r'^#include\s+[<"]win32k\.h[>"]', re.MULTILINE)


@dataclass
class SehSpan:
    start: int
    end: int
    try_body: str
    handler_body: str
    is_finally: bool
    line: int


def _split_args(s: str) -> list[str]:
    out, depth, cur = [], 0, []
    for ch in s:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
    if "".join(cur).strip():
        out.append("".join(cur).strip())
    return out


def _balanced_call_args(text: str, open_paren: int) -> tuple[str, int]:
    """Given index of '(', return (inner_args, index_past_matching_')')."""
    depth, i = 1, open_paren + 1
    while i < len(text) and depth:
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
        i += 1
    return text[open_paren + 1 : i - 1], i


def _find_seh_spans(text: str) -> list[SehSpan]:
    spans: list[SehSpan] = []
    for m in RE_TRY.finditer(text):
        tb = _find_brace_matched_block(text, m.end())
        if tb is None:
            continue
        try_body = text[tb[0] + 1 : tb[1]]
        after = tb[1] + 1
        zone = text[after : after + 600]
        em = re.search(r"_SEH2_EXCEPT\s*\(", zone)
        fm = re.search(r"_SEH2_FINALLY\b", zone)
        handler_body, is_finally, hb_end = "", False, after
        if em:
            _, past = _balanced_call_args(text, after + em.end() - 1)
            hb = _find_brace_matched_block(text, past)
            if hb:
                handler_body = text[hb[0] + 1 : hb[1]]
                hb_end = hb[1] + 1
        elif fm:
            is_finally = True
            hb = _find_brace_matched_block(text, after + fm.end())
            if hb:
                handler_body = text[hb[0] + 1 : hb[1]]
                hb_end = hb[1] + 1
        # `_SEH2_END` may or may not be followed by `;` in win32k
        # (the macro ends in `}`); consume the optional semicolon so the
        # whole TRY..END token sequence is replaced, not left dangling.
        endm = re.compile(r"\s*_SEH2_END[ \t]*;?").match(text, hb_end)
        if endm is None:
            # No _SEH2_END found where expected → span shape we don't
            # understand; skip this site rather than corrupt the source.
            continue
        spans.append(
            SehSpan(
                start=m.start(),
                end=endm.end() if endm else hb_end,
                try_body=try_body,
                handler_body=handler_body,
                is_finally=is_finally,
                line=text[: m.start()].count("\n") + 1,
            )
        )
    return spans


def _line_indent(text: str, off: int) -> str:
    ls = text.rfind("\n", 0, off) + 1
    return text[ls:off]


def _dedent_to(block: str, indent: str) -> str:
    """Strip the try-body's extra nesting, re-indent to `indent`."""
    lines = block.split("\n")
    # smallest indent among non-blank lines
    base = min((len(ln) - len(ln.lstrip()) for ln in lines if ln.strip()), default=0)
    out = []
    for ln in lines:
        out.append(indent + ln[base:] if ln.strip() else "")
    return "\n".join(out).strip("\n")


def _unwrap_yield(h: str) -> str:
    """`_SEH2_YIELD(<stmt>);` -> `<stmt>;` with balanced-paren extraction
    (the inner stmt may itself contain parens, e.g.
    `_SEH2_YIELD(return _SEH2_GetExceptionCode();)`)."""
    while True:
        m = RE_YIELD_OPEN.search(h)
        if not m:
            return h
        op = m.end() - 1  # index of '('
        inner, past = _balanced_call_args(h, op)
        # absorb an optional trailing ';' after the _SEH2_YIELD(...)
        tail = past
        while tail < len(h) and h[tail] in " \t":
            tail += 1
        if tail < len(h) and h[tail] == ";":
            tail += 1
        stmt = inner.strip().rstrip(";").strip()
        h = h[: m.start()] + stmt + ";" + h[tail:]


def _transform_handler(handler: str, var: str = "Status") -> str:
    h = _unwrap_yield(handler)
    h = RE_GETCODE.sub(var, h)
    h = re.sub(rf"^\s*{re.escape(var)}\s*=\s*{re.escape(var)}\s*;\s*$", "", h, flags=re.MULTILINE)
    h = re.sub(r"\bNOTHING\s*;", "", h)
    return h.strip()


def _handler_transfers(handler: str) -> bool:
    """Does the EXCEPT body unconditionally leave the block (return/goto)?
    If so, code after it is reached only on probe success."""
    t = _transform_handler(handler, "X")
    return bool(re.search(r"\b(return|goto)\b", t))


def _is_swallow(handler: str) -> bool:
    h = re.sub(r"/\*.*?\*/", "", handler, flags=re.S)
    h = re.sub(r"//[^\n]*", "", h)
    h = h.replace("NOTHING", "").replace("(void)0", "").replace(";", "").strip()
    return h == ""


@dataclass
class RewriteResult:
    text: str
    rewritten: int
    skipped: list[tuple[int, str]]


def _enclosing_func(text: str, start: int) -> str:
    """Text of the enclosing function up to `start` (bounded by the
    previous top-level `}` on its own line — win32k brace style)."""
    m = None
    for mm in re.finditer(r"\n\}\s*\n", text[:start]):
        m = mm
    return text[(m.end() if m else 0) : start]


def _has_status_lvalue(func_text: str) -> bool:
    """Does the enclosing function have an NTSTATUS Status lvalue we can
    assign the Safe* result to?"""
    return bool(
        re.search(r"\bNTSTATUS\b[\s\w,\*]*\bStatus\b", func_text)
        or re.search(r"\bStatus\s*=", func_text)
    )


def _why_skip(sp: SehSpan, has_status: bool) -> str | None:
    if sp.is_finally:
        return "_SEH2_FINALLY (Pass 2)"
    if "_SEH2_TRY" in sp.try_body:
        return "nested _SEH2_TRY"
    if "_SEH2_LEAVE" in sp.try_body:
        return "_SEH2_LEAVE in try (manual goto lowering)"
    stripped = _strip_literals(sp.try_body)
    generic = RE_PROBE_STMT.findall(stripped)
    typed = [m for m in RE_PROBE_ANY.findall(stripped)]
    if not generic and typed:
        return "typed probe only (v2)"
    if not generic:
        return "no generic ProbeForRead/Write"
    if RE_BRACELESS_GUARD.search(stripped):
        # winsta L1242 shape — a probe is the sole stmt of a brace-less
        # if/for/while. Sequencing it safely needs re-bracing; defer.
        return "brace-less conditional probe — manual (audit risk #1)"
    if len(generic) > 1:
        # multi-probe → handled by _build_multiprobe (short-circuit chain),
        # but only when we can carry/short-circuit the status:
        if not _is_swallow(sp.handler_body) and not has_status:
            return "multi-probe, no Status lvalue — manual (audit risk #1)"
        return None
    if (
        not _is_swallow(sp.handler_body)
        and not has_status
        and not _handler_transfers(sp.handler_body)
    ):
        # no Status lvalue AND handler doesn't transfer control → can
        # neither carry the code nor guarantee body-skip-on-failure
        return "non-swallow, no Status, no control transfer — manual"
    return None


def _extract_all_probes(body: str):
    """Return ([(kind,p,n,a),...], rest) where rest is `body` with every
    generic ProbeForRead/Write statement removed (order preserved)."""
    probes, keep, pos = [], [], 0
    for m in RE_PROBE_STMT.finditer(body):
        op = body.index("(", m.end() - 1)
        args_s, past = _balanced_call_args(body, op)
        args = _split_args(args_s)
        end = body.find(";", past)
        if len(args) != 3 or end == -1:
            return None, None
        probes.append((m.group(1), args[0], args[1], args[2]))
        keep.append(body[pos : m.start()])
        pos = end + 1
    keep.append(body[pos:])
    return probes, "".join(keep)


def _safe_call(kind: str, p: str, n: str, a: str) -> str:
    fn = "SafeProbeForRead" if kind == "Read" else "SafeProbeForWrite"
    return f"{fn}({p}, {n}, {a})"


def _build_multiprobe(
    sp: SehSpan, indent: str, has_status: bool, probes: list, rest: str
) -> str | None:
    """Short-circuit sequence for >1 unconditional probe. A later probe
    must never overwrite an earlier failure, so chain on NT_SUCCESS."""
    body = _dedent_to(rest, indent + "    ") if rest.strip() else ""
    calls = [_safe_call(k, p, n, a) for (k, p, n, a) in probes]

    if _is_swallow(sp.handler_body):
        cond = (" &&\n" + indent + "    ").join(f"NT_SUCCESS({c})" for c in calls)
        out = [f"{indent}if ({cond})", f"{indent}{{"]
        if body:
            out.append(body)
        out.append(f"{indent}}}")
        return "\n".join(out)

    # non-swallow: has_status guaranteed by _why_skip for multi.
    fail = _transform_handler(sp.handler_body)
    out = [f"{indent}Status = {calls[0]};"]
    for c in calls[1:]:
        out.append(f"{indent}if (NT_SUCCESS(Status))")
        out.append(f"{indent}    Status = {c};")
    if fail:
        out += [f"{indent}if (!NT_SUCCESS(Status))", f"{indent}{{"]
        for fl in fail.split("\n"):
            out.append(f"{indent}    {fl.strip()}" if fl.strip() else "")
        out.append(f"{indent}}}")
    if body:
        out += [f"{indent}if (NT_SUCCESS(Status))", f"{indent}{{", body, f"{indent}}}"]
    return "\n".join(out)


def _build_replacement(sp: SehSpan, indent: str, has_status: bool) -> str | None:
    body = sp.try_body
    _allp, _rest = _extract_all_probes(body)
    if _allp is None or _rest is None:
        return None
    if len(_allp) > 1:
        return _build_multiprobe(sp, indent, has_status, _allp, _rest)
    pm = RE_PROBE_STMT.search(_strip_literals(body))
    if pm is None:
        return None
    # locate the real probe call in the unstripped body
    real = RE_PROBE_STMT.search(body)
    if real is None:
        return None
    kind = real.group(1)
    open_paren = body.index("(", real.end() - 1)
    args_s, past = _balanced_call_args(body, open_paren)
    args = _split_args(args_s)
    if len(args) != 3:
        return None
    p, n, a = args
    stmt_end = body.find(";", past)
    if stmt_end == -1:
        return None
    rest = body[: real.start()] + body[stmt_end + 1 :]
    rest = _dedent_to(rest, indent) if rest.strip() else ""

    fn = "SafeProbeForRead" if kind == "Read" else "SafeProbeForWrite"
    call = f"{fn}({p}, {n}, {a})"

    if _is_swallow(sp.handler_body):
        inner = _dedent_to(rest, indent + "    ") if rest.strip() else ""
        lines = [f"{indent}if (NT_SUCCESS({call}))", f"{indent}{{"]
        if inner:
            lines.append(inner)
        lines.append(f"{indent}}}")
        return "\n".join(lines)

    # Non-swallow, function has no NTSTATUS Status, but the handler
    # transfers control (return/goto): use a block-local temp. The body
    # after the block is reached only on probe success (failure left via
    # the return/goto), so it needs no extra guard.
    if not has_status:
        ft = _transform_handler(sp.handler_body, "__SehStatus")
        out = [
            f"{indent}{{",
            f"{indent}    NTSTATUS __SehStatus = {call};",
            f"{indent}    if (!NT_SUCCESS(__SehStatus))",
            f"{indent}    {{",
        ]
        for fl in ft.split("\n"):
            out.append(f"{indent}        {fl.strip()}" if fl.strip() else "")
        out += [f"{indent}    }}", f"{indent}}}"]
        if rest.strip():
            out.append(_dedent_to(rest, indent))
        return "\n".join(out)

    # Non-swallow: probe; on failure run the handler's side effects; the
    # original try body must run ONLY on probe success (otherwise a bad
    # pointer is still dereferenced — defeats the probe). Guard with else.
    fail = _transform_handler(sp.handler_body)
    body_inner = _dedent_to(rest, indent + "    ") if rest.strip() else ""

    if not fail:
        # pure SET_STATUS (Status already holds the Safe* code): the body
        # simply runs on success. Cleanest faithful form (== spike).
        out = [f"{indent}Status = {call};"]
        if body_inner:
            out += [f"{indent}if (NT_SUCCESS(Status))", f"{indent}{{", body_inner, f"{indent}}}"]
        return "\n".join(out)

    out = [f"{indent}Status = {call};", f"{indent}if (!NT_SUCCESS(Status))", f"{indent}{{"]
    for fl in fail.split("\n"):
        out.append(f"{indent}    {fl.strip()}" if fl.strip() else "")
    out.append(f"{indent}}}")
    if body_inner:
        out += [f"{indent}else", f"{indent}{{", body_inner, f"{indent}}}"]
    return "\n".join(out)


def rewrite_text(text: str, src_name: str) -> RewriteResult:
    spans = _find_seh_spans(text)
    skipped: list[tuple[int, str]] = []
    edits: list[tuple[int, int, str]] = []
    for sp in spans:
        cat = classify_span(sp.try_body, sp.handler_body, sp.is_finally)
        if cat != Category.PROBE_LOWER:
            continue
        has_status = _has_status_lvalue(_enclosing_func(text, sp.start))
        why = _why_skip(sp, has_status)
        if why:
            skipped.append((sp.line, why))
            continue
        indent = _line_indent(text, sp.start)
        repl = _build_replacement(sp, indent, has_status)
        if repl is None:
            skipped.append((sp.line, "unsupported probe/handler shape"))
            continue
        edits.append((sp.start, sp.end, repl.lstrip()))

    new = text
    for start, end, repl in sorted(edits, key=lambda e: -e[0]):
        new = new[:start] + repl + new[end:]
    if edits and "safe_user_io.h" not in new:
        m = RE_INCLUDE_W32K.search(new)
        if m:
            new = new[: m.end()] + "\n#include <safe_user_io.h>" + new[m.end() :]
    return RewriteResult(new, len(edits), skipped)


def rewrite_file(path: Path) -> RewriteResult:
    return rewrite_text(path.read_text(errors="replace"), path.name)


def run_oracle(winsta_path: Path) -> int:
    txt = winsta_path.read_text(errors="replace")
    if "_SEH2_TRY" not in txt:
        repo = subprocess.run(
            ["git", "-C", str(winsta_path.parent), "rev-parse", "--show-toplevel"],
            capture_output=True,
            text=True,
        ).stdout.strip()
        rel = winsta_path.resolve().relative_to(repo)
        txt = subprocess.run(
            ["git", "-C", repo, "show", f"HEAD:{rel.as_posix()}"], capture_output=True, text=True
        ).stdout
        print(f"(oracle: using git HEAD:{rel})")

    before = txt.count("_SEH2_TRY")
    res = rewrite_text(txt, "winsta.c")
    after = res.text.count("_SEH2_TRY")
    n_safe = len(re.findall(r"\bSafeProbeFor(Read|Write)\b", res.text))
    inc = "safe_user_io.h" in res.text
    multi = any("multi-probe" in w or "conditional" in w for _, w in res.skipped)
    checks = [
        ("8 sites originally", before == 8),
        ("4 clean (a) rewritten", res.rewritten == 4),
        ("multi-probe (a) site flagged manual", multi),
        ("4 _SEH2_TRY remain (1a-manual + 2e + 1d)", after == 4),
        ("safe_user_io.h injected", inc),
        ("SafeProbeFor* emitted (>=4)", n_safe >= 4),
        ("brace balanced", res.text.count("{") == res.text.count("}")),
    ]
    ok = True
    print(
        f"winsta.c: _SEH2_TRY {before}->{after}; rewritten={res.rewritten}; "
        f"SafeProbeFor*={n_safe}; include={'yes' if inc else 'NO'}"
    )
    for name, cond in checks:
        print(f"  [{'OK' if cond else 'FAIL'}] {name}")
        ok &= cond
    if res.skipped:
        print("  skipped (left verbatim):")
        for ln, why in res.skipped:
            print(f"    L{ln}: {why}")
    print("ORACLE PASS" if ok else "ORACLE FAIL")
    return 0 if ok else 1
