#!/usr/bin/env python3
"""
scan_store_on_failure.py — the C writes an out-parameter only on its SUCCESS path; the
translation writes it and then returns a status that can be a failure.

    C:            if (!chunk) return STATUS_BAD_COMPRESSION_BUFFER;
                  ...
              out:
                  if (final_size) *final_size = dst_cur - dst;
                  return STATUS_SUCCESS;
    translated:   *final_size = result.bytes_written;       <-- clobbered on failure too
                  result.status
    faithful:     if result.status == STATUS_SUCCESS { *final_size = result.bytes_written; }

On failure the C leaves the caller's variable exactly as it was, and callers rely on that:
a retry loop that re-reads its own length, a `*Size = 0; if (!f(&Size)) use(Size)` idiom.
The translation writes on every path, so the first failure hands the caller a value the C
never produced. Smoke cannot see it; difftest-contract found it (compress.json, 2026-09-26)
as the other half of the dropped-null-guard class (scan_dropped_null_guard.py).

HOW. Per Rust function returning a value:
  1. a pointer PARAMETER P is stored through (`*P = …`, `(*P).f = …`), and the value the
     function produces NEXT after that store — the first `return E` or block tail reached —
     is not a success literal;
  2. that value is not a status variable the body success-tests before the store
     (`if st == STATUS_SUCCESS { *P = … }`, `if !NT_SUCCESS(st) { return st; }`); and
  3. the C function of the same name (tools/c_twin.py) stores through the same parameter
     ONLY where the next thing it does is `return <success literal>`, and it has at least
     one non-success `return` — so there IS a failure path that leaves P untouched.
Step 3 is the oracle, as in scan_dropped_null_guard: without it every `*out = x; status`
in the tree is a candidate.

Coarse by design: "next value" is textual, not a CFG. It reads a store followed by a
failure `return` inside a later `if` as storing-then-failing, which is exactly the shape
(C does its checks first; the translation hoisted the store above one).

Usage:
    python3 tools/scan_store_on_failure.py            # report with the C evidence
    python3 tools/scan_store_on_failure.py --lint     # terse (CLI baseline gate)

Suppress a reviewed site with `// nolint: store-on-failure` on the store line.
Exit 1 if any finding.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.c_twin import (
    CIndex,
    Func,
    blank_rust,
    match_brace,
    prefer,
    rust_functions,
    same_module,
)
from tools.scan_dropped_null_guard import (
    GEN_MARKERS,
    SKIP_PATH_RE,
    _match_paren,
    _split_top,
    c_name_for,
    c_param_names,
    is_pointer,
    split_params,
)
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

NOLINT_RE = re.compile(r"//\s*nolint(?::\s*store-on-failure\b|\s*$)")

SUCCESS = {"STATUS_SUCCESS", "S_OK", "TRUE", "ERROR_SUCCESS", "NO_ERROR", "NOERROR", "true"}


def _strip(e: str) -> str:
    e = " ".join(e.split())
    while e.startswith("(") and e.endswith(")"):
        e = e[1:-1].strip()
    e = re.sub(r"\.\s*(?:try_)?into\s*\(\s*\)(?:\s*\.\s*unwrap\s*\(\s*\))?$", "", e)
    return re.sub(r"\s+as\s+[\w:]+$", "", e).strip()


def is_success(e: str) -> bool:
    e = _strip(e)
    last = e.split("::")[-1]
    return last in SUCCESS or last.endswith("_SUCCESS")


# The numeric value of a success literal, so the Rust `1u8` matches the C `TRUE`.
_NUMERIC = {"TRUE": 1, "true": 1, "FALSE": 0, "false": 0}


def literal_value(e: str) -> int | None:
    e = _strip(e)
    last = e.split("::")[-1]
    if last in _NUMERIC:
        return _NUMERIC[last]
    if last in SUCCESS or last.endswith("_SUCCESS"):
        return 0
    m = re.fullmatch(r"(-?)(0x[0-9a-fA-F]+|\d+)(?:_?[iu](?:8|16|32|64|size))?", e)
    if m:
        return int(m.group(2), 0) * (-1 if m.group(1) else 1)
    return None


_RET_RE = re.compile(r"\breturn\b")


def next_value(body: str, pos: int) -> str | None:
    """The first value produced after offset `pos`: a `return E` at any depth below the
    store's block, or the tail expression of the first block the store's block closes into.
    Leaving a block skips what that path cannot reach: an `else` chain after an `if` block,
    the sibling arms after a match arm. None when the path ends in a value-less block."""
    depth = 0
    i = pos
    seg_start = pos
    while i < len(body):
        c = body[i]
        if c == "r" and _RET_RE.match(body, i) and (i == 0 or not body[i - 1].isalnum()):
            end = _stmt_end(body, i + 6)
            return body[i + 6 : end].strip() or None
        if c == "{":
            depth += 1
        elif c == "}":
            if depth == 0:
                tail = body[seg_start:i].strip()
                if tail:
                    return tail
                i = _skip_unreachable(body, i)
                seg_start = i + 1
            else:
                depth -= 1
                if depth == 0:
                    seg_start = i + 1
        elif c == ";" and depth == 0:
            seg_start = i + 1
        i += 1
    return None


def _open_of(body: str, close: int) -> int:
    depth = 0
    for i in range(close, -1, -1):
        if body[i] == "}":
            depth += 1
        elif body[i] == "{":
            depth -= 1
            if depth == 0:
                return i
    return -1


def _skip_unreachable(body: str, close: int) -> int:
    """`close` ends the block the path is leaving. Return the offset of the last `}` that
    path also leaves without executing anything: the rest of an `if … else …` chain, or
    — for a match arm — the remaining arms up to the match's own `}`."""
    op = _open_of(body, close)
    before = body[max(0, op - 3) : op].rstrip() if op > 0 else ""
    if before.endswith("=>"):
        # Match arm: jump to the `}` closing the whole match.
        depth = 0
        for j in range(close + 1, len(body)):
            if body[j] == "{":
                depth += 1
            elif body[j] == "}":
                if depth == 0:
                    return j
                depth -= 1
        return close
    j = close
    while True:
        m = re.compile(r"\s*else\b").match(body, j + 1)
        if not m:
            return j
        k = body.find("{", m.end())
        if k < 0:
            return j
        j = match_brace(body, k)
        if j < 0:
            return close


def _stmt_end(body: str, at: int) -> int:
    depth = 0
    for i in range(at, len(body)):
        c = body[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            if depth == 0:
                return i
            depth -= 1
        elif c == ";" and depth == 0:
            return i
    return len(body)


def success_tested(var: str, body: str, before: int) -> bool:
    """`body[:before]` success-tests `var`, so a store after it sits on the success path."""
    v = re.escape(var)
    return bool(
        re.search(
            r"\b(?:NT_SUCCESS|SUCCEEDED|FAILED|NT_ERROR)!?\s*\(\s*" + v + r"\s*\)"
            r"|\b" + v + r"\s*(?:==|!=|<|>=)\s*(?:[\w:]*_SUCCESS|S_OK|0|TRUE|FALSE)\b"
            r"|\b" + v + r"\s*\.\s*is_(?:ok|err)\s*\(",
            body[:before],
        )
    )


_FAIL_NAME_RE = re.compile(
    r"(?:\w+::)*(?:STATUS_\w+|E_\w+|\w+_E_\w+|ERROR_\w+|CR_\w+|FALSE|false|NULL)"
)


def may_fail(v: str, body: str, store_at: int) -> bool:
    """The Rust value `v` produced after a store can be a failure. Literals are decided
    later against the C's own success literal; unknown constants and calls are not guessed."""
    if literal_value(v) is not None:
        return True
    if is_success(v):
        return False
    if re.fullmatch(r"Err\s*\(.*\)", v, re.S):
        return True
    if re.fullmatch(r"[A-Z][A-Z0-9_]*|(?:\w+::)+[A-Z][A-Z0-9_]*", v):
        return bool(_FAIL_NAME_RE.fullmatch(v))
    if not re.fullmatch(r"[A-Za-z_]\w*(?:\s*\.\s*\w+)*", v):
        return False
    if success_tested(v, body, store_at):
        return False
    # `let dwError: DWORD = ERROR_SUCCESS;` never reassigned is a success literal by name.
    d = re.search(r"\blet\s+(mut\s+)?" + re.escape(v) + r"\s*(?::[^=;]*)?=\s*([^;]+);", body)
    return not (d and not d.group(1) and is_success(d.group(2)))


# --- C side ----------------------------------------------------------------------------

_C_RET_RE = re.compile(r"\breturn\b\s*([^;]*);")


def c_stores(p: str, body: str) -> list[int]:
    """Offsets of writes through C parameter `p`: `*p = …`, `p->f = …`, `p[i] = …`, a
    compound assignment or `++`/`--` (`sym->current++`), and handing P or `&P->f` to a
    call — `RtlSIZETAdd(a, b, ViewSize)` writes *ViewSize."""
    v = re.escape(p)
    lhs = (
        r"(?:(?<![\w)\]])\*\s*\(?\s*" + v + r"\s*\)?"
        r"|" + _bare_lhs(v) + r"\s*(?:->\s*\w+(?:\s*(?:\.|->)\s*\w+)*|\[[^\]]*\]))"
    )
    pat = re.compile(
        lhs + r"\s*(?:[-+*/%&|^]|<<|>>)?=(?!=)|" + lhs + r"\s*(?:\+\+|--)|(?:\+\+|--)\s*" + lhs
    )
    out = [m.start() for m in pat.finditer(body)]
    arg = re.compile(r"&?\s*" + _bare_lhs(v) + r"\b(?:\s*->\s*\w+)*")
    for m in _C_CALL_RE.finditer(body):
        if m.group(1) in _NOT_CALLS:
            continue
        close = _match_paren(body, m.end() - 1)
        if close < 0:
            continue
        if any(arg.fullmatch(a.strip()) for a in _split_top(body[m.end() : close])):
            out.append(m.start())
    return sorted(out)


_C_CALL_RE = re.compile(r"\b([A-Za-z_]\w*)\s*\(")
_NOT_CALLS = {"if", "while", "for", "switch", "return", "sizeof", "defined"} | {
    "ASSERT",
    "NT_ASSERT",
    "DPRINT",
    "DPRINT1",
    "TRACE",
    "WARN",
    "ERR",
    "FIXME",
}


def _bare_lhs(v: str) -> str:
    return r"(?<![\w*.>])" + v


def c_next_return(body: str, at: int) -> str | None:
    m = _C_RET_RE.search(body, at)
    return m.group(1) if m else None


# A statement-level ALL-CAPS macro may expand to a `return` the C text does not show
# (gdi32's HANDLE_EMETAFDC). Anything not in this list after the store makes it unknown.
_C_MACRO_STMT_RE = re.compile(r"(?:^|[;{}])\s*([A-Z][A-Z0-9_]{3,})\s*\(")
_HARMLESS_MACROS = re.compile(
    r"(?:DPRINT\w*|TRACE\w*|WARN\w*|ERR|FIXME\w*|ASSERT\w*|NT_ASSERT\w*|UNREFERENCED\w*|"
    r"DBG\w*|DEBUG\w*|PAGED_CODE|KdPrint\w*|INFO\w*|MSG\w*|RtlZero\w*|SetLastError)"
)


def c_success_only(p: str, twin: Func, index: CIndex, depth: int = 1) -> tuple[str, str] | None:
    """(evidence, success literal) when `twin` writes through `p` only after its last
    failure `return` — i.e. every failure path leaves P untouched — and the return after
    each store is a success literal. One level deep into a same-file callee `twin` returns
    straight from with `p` passed unchanged — compress.c's RtlDecompressFragment is
    `return lznt1_decompress(…, final_size, …)`, and the `out:` label is in the callee."""
    body = twin.body
    at = c_stores(p, body)
    rets_all = list(_C_RET_RE.finditer(body))
    # P handed to a call whose result is RETURNED: the callee decides, or nobody can.
    if not at or all(any(m.start() < a < m.end() for m in rets_all) for a in at):
        return _through_callee(p, twin, index, depth) if depth else None
    if any(m.start() < a < m.end() for a in at for m in rets_all):
        return None  # some paths write P directly, others defer to a callee: unknowable
    rets = [(m.start(), _strip(m.group(1))) for m in rets_all if m.group(1).strip()]
    fails = [(o, r) for o, r in rets if not is_success(r)]
    if not fails or fails[-1][0] > at[0]:
        return None
    succ = []
    for s in at:
        r = c_next_return(body, s)
        if r is None or not is_success(r):
            return None
        succ.append(_strip(r))
    for m in _C_MACRO_STMT_RE.finditer(body, at[0]):
        if not _HARMLESS_MACROS.fullmatch(m.group(1)):
            return None
    lo = body.rfind("\n", 0, at[0]) + 1
    hi = body.find("\n", at[0])
    store = " ".join(body[lo:hi].split())[:80]
    ev = f"{store}  … only after its last failure `return {fails[-1][1]}`"
    return ev, succ[0]


_C_RET_CALL_RE = re.compile(r"\breturn\s*\(?\s*([A-Za-z_]\w*)\s*\(")


def _through_callee(p: str, twin: Func, index: CIndex, depth: int) -> tuple[str, str] | None:
    body = twin.body
    for m in _C_RET_CALL_RE.finditer(body):
        close = _match_paren(body, m.end() - 1)
        if close < 0:
            continue
        args = [a.strip() for a in _split_top(body[m.end() : close])]
        if p not in args:
            continue
        for sub in index.in_file(twin.path, m.group(1)):
            names = c_param_names(sub.params)
            if len(names) == len(args):
                r = c_success_only(names[args.index(p)], sub, index, depth - 1)
                if r:
                    return f"{m.group(1)}(): {r[0]}", r[1]
    return None


@dataclass
class Site:
    path: Path
    line: int
    fn: str
    param: str
    index: int
    params: list[tuple[str, str]]
    value: str
    src: str


def _returns_value(text: str, f: Func) -> bool:
    close = text.rfind(")", 0, f.body_start)
    sig = text[close : f.body_start]
    return "->" in sig and not re.search(r"->\s*(?:\(\s*\)|!)", sig)


def candidates(path: Path) -> list[Site]:
    try:
        raw = path.read_text(errors="replace")
    except OSError:
        return []
    if any(g in raw[:2000] for g in GEN_MARKERS):
        return []
    text = blank_rust(raw)
    lines = raw.split("\n")
    out = []
    for f in rust_functions(text, path):
        if not _returns_value(text, f):
            continue
        params = split_params(f.params)
        ptrs = {n: i for i, (n, t) in enumerate(params) if is_pointer(t)}
        if not ptrs:
            continue
        for m in _all_stores(f.body):
            name, at, end = m
            if name not in ptrs:
                continue
            # `*ppv = NULL` before failing is COM's out-pointer rule (and harmless anywhere):
            # a caller that ignores the status still sees no stale pointer. Exceeding the C.
            if _NULL_STORE_RE.search(f.body, at, end):
                continue
            val = next_value(f.body, end)
            if val is None:
                continue
            head = _strip(val)
            if not may_fail(head, f.body, at):
                continue
            ln = text.count("\n", 0, f.body_start + at) + 1
            if NOLINT_RE.search(lines[ln - 1]):
                continue
            out.append(
                Site(
                    path,
                    ln,
                    f.name,
                    name,
                    ptrs[name],
                    params,
                    head[:60],
                    lines[ln - 1].strip()[:110],
                )
            )
            break  # one finding per function
    return out


_STORE_ANY_RE = re.compile(
    r"(?<![\w)\]])\*\s*(?P<a>[A-Za-z_]\w*)\s*(?:\.\s*(?:add|offset)\s*\([^()]*\)\s*)?=(?!=)"
    r"|\(\s*\*\s*(?P<b>[A-Za-z_]\w*)\s*\)\s*\.\s*\w+(?:\s*\.\s*\w+)*\s*=(?!=)"
)


_NULL_STORE_RE = re.compile(
    r"=\s*(?:core::|std::)?(?:ptr::)?null(?:_mut)?\s*(?:::<[^>]*>)?\s*\(\s*\)\s*;|=\s*NULL\s*;"
)


def _all_stores(body: str):
    for m in _STORE_ANY_RE.finditer(body):
        end = body.find(";", m.end())
        yield (m.group("a") or m.group("b"), m.start(), end + 1 if end >= 0 else m.end())


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    lint = "--lint" in sys.argv
    roots = [Path(a) for a in args] if args else [RUST_ROOT]
    files: list[Path] = []
    for root in roots:
        if root.is_file():
            files.append(root)
        elif root.is_dir():
            files.extend(source_files(root))
        else:
            print(f"scan_store_on_failure: no such path: {root}", file=sys.stderr)
            return 2
    sites = [
        s for f in sorted(set(files)) if not SKIP_PATH_RE.search(str(f)) for s in candidates(f)
    ]
    index = CIndex()
    index.load({s.fn for s in sites})

    findings: list[tuple[str, Site, str]] = []
    for s in sites:
        # The twin must share a CRATE word with the Rust path: a common file stem is not
        # enough (win32k's main.rs `DriverEntry` is not drivers/bus/acpi/main.c's).
        twins: list[Func] = [t for t in prefer(s.path, index.get(s.fn)) if same_module(s.path, t)]
        if not twins:
            continue
        cname = c_name_for(s.params, s.index, c_param_names(twins[0].params))
        got = c_success_only(cname, twins[0], index) if cname else None
        if got is None:
            continue
        evidence, succ = got
        # The Rust produces what the C's success return does (`1u8` for TRUE): no divergence.
        if is_success(s.value) or (
            literal_value(s.value) is not None and literal_value(s.value) == literal_value(succ)
        ):
            continue
        try:
            rel = s.path.relative_to(REPO)
        except ValueError:
            rel = s.path
        findings.append((str(rel), s, evidence))

    for rel, s, evidence in findings:
        if lint:
            print(f"{rel}:{s.line}: store-on-failure {s.fn}({s.param})")
        else:
            print(
                f"{rel}:{s.line}: store-on-failure: {s.fn}: stores `*{s.param}`, "
                f"then produces `{s.value}`"
            )
            print(f"    rust: {s.src}")
            print(f"    c:    {evidence}")
    if not lint:
        print(f"\n{len(findings)} site(s)")
        if findings:
            print("fix: store only on the success path, as the C does; if the Rust contract")
            print("     documents the write, mark `// nolint: store-on-failure`.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
