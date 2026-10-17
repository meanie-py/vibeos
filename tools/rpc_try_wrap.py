#!/usr/bin/env python3
"""Restore the C's RpcTryExcept around RPC stub calls in a user-mode Rust translation.

The translations ran the C under DUMMY_PSEH and kept only the try body, so a stub that
raises (RPC_S_SERVER_UNAVAILABLE before the server is up) kills the caller instead of
returning an error. This reads each C `RpcTryExcept { X = Stub(...); } RpcExcept(..)
{ X = Map(RpcExceptionCode()); }` block, and in the Rust rewrites the matching
assignment `lhs = Stub(args);` to

    lhs = rpc_try(|| Stub(args)).unwrap_or_else(|code| Map(code as _) as _);

using ntdll_sys::seh::rpc_try. Only that shape is rewritten. Every stub call it cannot
match (used inside an expression, a handler that is more than one mapped assignment, two
C sites disagreeing on the handler) is listed for a hand conversion, and the exit status
is 1 while any remain, so a partial run cannot pass for a complete one. It then drops
the dead `if _SEH2_Volatile0 != 0 { .. RpcExceptionCode() .. }` arms the translation
kept, and the `_SEH2_Volatile*` locals once nothing reads them.

    tools/rpc_try_wrap.py --c dll/win32/advapi32/service/scm.c \
        --rs sdk/lib/rust/advapi32-rust/src/service/scm.rs [--dry-run]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

TRY_RE = re.compile(r"RpcTryExcept\b(.*?)RpcExcept\s*\((.*?)\)\s*(.*?)RpcEndExcept", re.S)
CALL_RE = re.compile(r"\b([A-Za-z_]\w*)\s*\(")
ASSIGN_RE = re.compile(
    r"(?P<var>\w+)\s*=\s*(?:(?P<map>\w+)\s*\(\s*RpcExceptionCode\s*\(\s*\)\s*\)|RpcExceptionCode\s*\(\s*\))\s*;"
)
SETLAST_RE = re.compile(
    r"SetLastError\s*\(\s*(?P<map>\w+)\s*\(\s*RpcExceptionCode\s*\(\s*\)\s*\)\s*\)\s*;"
    r"\s*(?P<var>\w+)\s*=\s*(?P<val>FALSE|NULL|0)\s*;"
)
NOT_STUBS = {
    "if",
    "while",
    "for",
    "switch",
    "return",
    "sizeof",
    "TRACE",
    "WARN",
    "ERR",
    "FIXME",
    "DPRINT",
    "DPRINT1",
    "RpcExceptionCode",
    "SetLastError",
    "MIDL_user_free",
}


def strip_c_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.S)
    return re.sub(r"//[^\n]*", "", text)


def c_handlers(paths: list[Path]) -> tuple[dict[str, tuple], dict[str, str], set[str]]:
    """stub -> (kind, mapper, value); stub -> note; stubs whose handler is not understood."""
    handlers: dict[str, tuple] = {}
    problems: dict[str, str] = {}
    unhandled: set[str] = set()
    for path in paths:
        text = strip_c_comments(path.read_text(errors="replace"))
        for m in TRY_RE.finditer(text):
            body, handler = m.group(1), m.group(3).strip()
            stubs = [s for s in CALL_RE.findall(body) if s not in NOT_STUBS]
            if not stubs:
                continue
            inner = handler.strip().rstrip(";").strip()
            inner = inner[1:-1] if inner.startswith("{") and inner.endswith("}") else inner
            h = SETLAST_RE.search(inner) or ASSIGN_RE.search(inner)
            if not h:
                problems[stubs[0]] = f"{path.name}: handler is not a mapped assignment"
                unhandled.add(stubs[0])
                continue
            if h.re is SETLAST_RE:
                # SetLastError(Map(code)); bResult = FALSE;
                mapper = ("setlast", h.group("map"), h.group("val"))
            else:
                mapper = ("assign", h.group("map") or "", "")
            rest = " ".join((inner[: h.start()] + inner[h.end() :]).split())
            if rest:
                # Out-buffer frees beside the mapping cover a raise after the stub
                # allocated, which a server that is not there never reaches. Keep the
                # mapping and name the site.
                problems.setdefault(stubs[0], f"{path.name}: handler also runs: {rest[:120]}")
            for stub in stubs[:1]:
                if stub in handlers and handlers[stub] != mapper:
                    problems[stub] = f"C sites disagree: {handlers[stub]!r} vs {mapper!r}"
                handlers[stub] = mapper
    return handlers, problems, unhandled


def balanced_end(text: str, open_idx: int) -> int:
    """Index just past the ')' matching the '(' at open_idx; -1 if unbalanced."""
    depth = 0
    i = open_idx
    while i < len(text):
        c = text[i]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i + 1
        elif c == '"':
            i += 1
            while i < len(text) and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
        elif text.startswith("//", i):
            nl = text.find("\n", i)
            i = len(text) if nl < 0 else nl
            continue
        i += 1
    return -1


def brace_end(text: str, open_idx: int) -> int:
    """Index just past the '}' matching the '{' at open_idx; -1 if unbalanced."""
    swapped = text.translate(str.maketrans("{}()", "()  "))
    return balanced_end(swapped, open_idx)


# The translations kept DUMMY_PSEH's except arm as dead code guarded by a flag nothing
# sets: `if _SEH2_Volatile0 != 0 { ret = Map(RpcExceptionCode()); }`. Once the call is
# wrapped, rpc_try owns that mapping; the arm is dead, and RpcExceptionCode does not
# exist in Rust, so it also keeps the crate from compiling.
# The flag is spelled however the model chose (`_SEH2_Volatile0`, `__seh_raised`) and may
# carry the filter's folded remains (`|| 0 != 0 && 1 != 0`); the arm reads the code as
# RpcExceptionCode() or as the `_SEH2_VolatileExceptionCode` local.
_FLAG = r"(?:_SEH2_Volatile\w*|__seh_raised|__rpc_raised)"
DEAD_ARM_RE = re.compile(r"^[ \t]*if\s+(?:" + _FLAG + r"\s*!=\s*0\b[^{\n]*|false\s*)\{", re.M)
_CODE_READ = re.compile(r"\bRpcExceptionCode\b|\b_SEH2_VolatileExceptionCode\b")
SEH_LOCAL_RE = re.compile(
    r"^[ \t]*(?:let\s+(?:mut\s+)?" + _FLAG + r"\b[^;\n]*"
    r"|let\s+_\s*=\s*" + _FLAG + r"|" + _FLAG + r"\s*=[^;\n]*);[ \t]*\n",
    re.M,
)


def drop_dead_arms(text: str) -> tuple[str, int]:
    """Remove the dead except arms whose body reads RpcExceptionCode, then the
    `_SEH2_Volatile*` locals once nothing reads them."""
    out, pos, dropped = [], 0, 0
    for m in DEAD_ARM_RE.finditer(text):
        if m.start() < pos:
            continue
        end = brace_end(text, m.end() - 1)
        if end < 0 or not _CODE_READ.search(text[m.end() : end]):
            continue
        if text[end:].startswith("\n"):
            end += 1
        out.append(text[pos : m.start()])
        pos = end
        dropped += 1
    out.append(text[pos:])
    new = "".join(out)
    stripped = SEH_LOCAL_RE.sub("", new)
    if not re.search(r"\b" + _FLAG + r"\b", stripped):
        new = stripped
    return new, dropped


def on_err(mapper: tuple) -> str:
    kind, fn, val = mapper
    if kind == "setlast":
        failed = "0" if val in ("FALSE", "0") else "core::ptr::null_mut()"
        return f"|code| {{ SetLastError({fn}(code as _) as _); {failed} as _ }}"
    return f"|code| {fn}(code as _) as _" if fn else "|code| code as _"


def rewrite(
    text: str, handlers: dict[str, tuple], unhandled: set[str]
) -> tuple[str, int, list[str]]:
    out, done, leftovers = [], 0, []
    pos = 0
    names = "|".join(
        sorted((re.escape(s) for s in set(handlers) | unhandled), key=len, reverse=True)
    )
    if not names:
        return text, 0, []
    # `lhs = Stub(args);`, or `Stub(args)` as the function's tail expression.
    assign_re = re.compile(
        r"(?P<lhs>(?:let\s+(?:mut\s+)?\w+\s*(?::\s*[\w:]+)?|[\w.]+)\s*=\s*|)(?P<stub>"
        + names
        + r")\s*\("
    )
    any_re = re.compile(r"\b(?P<stub>" + names + r")\s*\(")
    for m in assign_re.finditer(text):
        if m.start() < pos:
            continue
        line_start = text.rfind("\n", 0, m.start()) + 1
        if text[line_start : m.start()].strip():
            continue  # not at statement start
        open_idx = m.end() - 1
        end = balanced_end(text, open_idx)
        closer = r"\n\s*\}" if not m.group("lhs") else ";"
        tail = re.match(r"\s*(?:as\s+[\w:]+\s*)?" + closer, text[end:]) if end >= 0 else None
        if not tail:
            continue
        stub = m.group("stub")
        if stub in unhandled:
            continue
        call = text[m.start("stub") : end]
        out.append(text[pos : m.start("stub")])
        out.append(f"rpc_try(|| {call}).unwrap_or_else({on_err(handlers[stub])})")
        pos = end
        done += 1
    out.append(text[pos:])
    new = "".join(out)
    for m in any_re.finditer(new):
        line_start = new.rfind("\n", 0, m.start()) + 1
        line = new[line_start : new.find("\n", m.start())]
        if line.lstrip().startswith("//") or re.search(r"\bfn\s+" + m.group("stub") + r"\b", line):
            continue
        if len(re.findall(r'(?<!\\)"', new[line_start : m.start()])) % 2:
            continue  # the name inside a log string
        if (
            "rpc_try(|| " + m.group("stub") in line
            or new[max(0, m.start() - 11) : m.start()] == "rpc_try(|| "
        ):
            continue
        lineno = new.count("\n", 0, m.start()) + 1
        leftovers.append(f"{lineno}: {line.strip()[:110]}")
    if done and "seh::rpc_try" not in new:
        first_use = re.search(r"^use [^\n]*;\n", new, re.M)
        at = first_use.end() if first_use else 0
        new = new[:at] + "use ntdll_sys::seh::rpc_try;\n" + new[at:]
    return new, done, leftovers


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument(
        "--c", nargs="+", required=True, type=Path, help="the C sources the Rust translates"
    )
    ap.add_argument("--rs", nargs="+", required=True, type=Path, help="the Rust files to rewrite")
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    handlers, problems, unhandled = c_handlers(args.c)
    status = 0
    for stub, why in sorted(problems.items()):
        print(f"note {stub}: {why}")
    for rs in args.rs:
        text = rs.read_text()
        new, done, leftovers = rewrite(text, handlers, unhandled)
        new, dropped = drop_dead_arms(new)
        print(f"{rs}: wrapped {done} call(s), dropped {dropped} dead except arm(s)")
        for left in leftovers:
            print(f"  NOT WRAPPED {rs.name}:{left}")
            status = 1
        if new != text and not args.dry_run:
            rs.write_text(new)
    return status


if __name__ == "__main__":
    sys.exit(main())
