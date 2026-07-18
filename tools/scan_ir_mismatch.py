#!/usr/bin/env python3
"""Scan LLVM IR modules (.ll) for same-module disagreements about one symbol.

The ud2-bomb detector, at SOURCE-side IR level (field manual §9.99): any
same-LLVM-module disagreement about a symbol's calling convention or signature
makes calls through the "wrong" view UB — instcombine replaces the call with
`unreachable` (→ ud2 on x86, silent fallthrough on arm64) and deletes the
symbol reference, so the LINK PASSES precisely because the code is broken.
scan_ud2.py catches the surviving folds in the linked binary (post-opt,
post-inline); this tool catches ALL armed sites pre-fold, with exact symbol
names, because dev-profile IR still shows both sides of every mismatch.

Reported classes:
  CC   — call-site calling convention != the in-module define/declare's CC.
         Folds on BOTH defs and declarations (LLVM trusts a declare's CC).
  SIG  — call-site arg types / arity / void-vs-value return != the DEFINE's
         (BOOLEAN i8 vs BOOL i32 crossings, dropped/added params, ret drift).
         Only def-authority SIG findings are reported: against a mere declare
         LLVM cannot prove the true signature, so decl-vs-decl SIG drift does
         not fold (it is also a deliberate pattern — ke-macros' per-shape
         `#[link_name = "DbgPrint"]` aliases). Pass --all to see those too.
  VAR  — variadic call sites against a FIXED-arity in-module define (the
         classpnp `TracePrint` class — machine-ABI-safe on cdecl, LLVM-UB in
         one module; durable fix = move the def to its own crate, see
         drvtrace-shim)

Usage:
    # emit dev IR for a crate, then scan it
    cargo rustc -p <crate> --target <triple> -- --emit=llvm-ir
    python3 tools/scan_ir_mismatch.py target/<triple>/debug/deps/<crate>-*.ll
    python3 tools/scan_ir_mismatch.py --check <files...>   # exit 1 on findings

Notes / honest limits:
  - Only SAME-MODULE views are compared (that is the fold domain under the
    workspace `lto=false` firewall). Cross-crate disagreement is machine-ABI
    territory — verify_ffi.py / the nm oracle cover that.
  - Not every SIG finding folds today (instcombine repairs some compatible
    shapes, e.g. discarding a value return); every finding is still a live
    landmine — CGU/inlining shifts decide, not you.
  - int-width-equal arg pairs (i8 vs i32 on the stack) are machine-safe on
    i386 yet still LLVM-visible mismatches; they are reported.
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

CC_TOKENS = (
    "x86_fastcallcc",
    "x86_stdcallcc",
    "x86_thiscallcc",
    "x86_vectorcallcc",
    "fastcc",
    "coldcc",
)
_CC_ALT = re.compile(r"\b(?:" + "|".join(CC_TOKENS) + r")\b")

# parameter attributes that may precede a type token in an arg list
ATTR_NOISE = re.compile(
    r"\b(noundef|nonnull|signext|zeroext|inreg|returned|nofree|noalias|"
    r"nocapture|readonly|writeonly|writable|dead_on_unwind|dead_on_return|"
    r"immarg|allocptr|swiftself|swifterror)\b"
    r"|align \d+"
    r"|byval\([^)]*\)|byref\([^)]*\)|sret\([^)]*\)|inalloca\([^)]*\)"
    r"|preallocated\([^)]*\)|dereferenceable(?:_or_null)?\(\d+\)"
    r"|captures\([^)]*\)|range\([^)]*\)|initializes\([^)]*\)"
)

KEYWORD_NOISE = re.compile(
    r"\b(tail|musttail|notail|call|invoke|callbr|define|declare|internal|"
    r"private|external|available_externally|linkonce\w*|weak\w*|appending|"
    r"common|extern_weak|hidden|protected|default|dso_local|dso_preemptable|"
    r"unnamed_addr|local_unnamed_addr|dllimport|dllexport|thread_local|"
    r"noundef|signext|zeroext|fast|nnan|ninf|nsz|arcp|contract|afn|reassoc)\b"
)


def _cc_of(text: str) -> str:
    for t in CC_TOKENS:
        if re.search(r"\b" + t + r"\b", text):
            return t
    return "ccc"


def _split_top(s: str) -> list[str]:
    out: list[str] = []
    depth = 0
    cur = ""
    for ch in s:
        if ch in "(<[{":
            depth += 1
        elif ch in ")>]}":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def _norm_ty(tok: str) -> str:
    tok = ATTR_NOISE.sub("", tok).strip()
    parts = tok.split()
    return parts[0] if parts else "?"


def _match_paren(s: str, open_idx: int) -> int:
    """Index just past the ')' matching the '(' at open_idx."""
    depth = 0
    i = open_idx
    while i < len(s):
        if s[i] == "(":
            depth += 1
        elif s[i] == ")":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return len(s)


def _rmatch_paren(s: str, close_idx: int) -> int:
    """Index of the '(' matching the ')' at close_idx."""
    depth = 0
    i = close_idx
    while i >= 0:
        if s[i] == ")":
            depth += 1
        elif s[i] == "(":
            depth -= 1
            if depth == 0:
                return i
        i -= 1
    return 0


SYM_RE = re.compile(r'@("[^"]+"|[-\w$.]+)\s*\(')


def parse_def(rest: str):
    """Parse 'define'/'declare' remainder: attrs cc <retty> @sym(<params>)."""
    m = SYM_RE.search(rest)
    if not m:
        return None
    sym = m.group(1).strip('"')
    pre = rest[: m.start()]
    cc = _cc_of(pre)
    pre = KEYWORD_NOISE.sub("", _CC_ALT.sub("", pre))
    pre = ATTR_NOISE.sub("", pre)
    ret = pre.split()[-1] if pre.split() else "?"
    end = _match_paren(rest, m.end() - 1)
    params = _split_top(rest[m.end() : end - 1])
    variadic = bool(params) and params[-1] == "..."
    if variadic:
        params = params[:-1]
    return sym, cc, ret, tuple(_norm_ty(p) for p in params if p), variadic


CALL_RE = re.compile(r"\b(call|invoke)\b")


def parse_call(line: str):
    """Parse a call/invoke: ... call attrs cc <ty-or-fnty> @sym(<args>) ...

    When the callee type is spelled as a fn type — mandatory for variadic
    callees: `call void (ptr, ...) @f(...)` — the parenthesized group right
    before @sym holds the DECLARED param types and the variadic flag; use it.
    Otherwise derive arg types from the call arguments themselves.
    """
    m = CALL_RE.search(line)
    if not m:
        return None
    rest = line[m.end() :]
    sm = SYM_RE.search(rest)
    if not sm:
        return None
    sym = sm.group(1).strip('"')
    pre = rest[: sm.start()].rstrip()
    cc = _cc_of(pre)
    variadic_spelled = False
    if pre.endswith(")"):
        # fn-type group before @sym: '<retty> (params...)'
        o = _rmatch_paren(pre, len(pre) - 1)
        fnty_params = _split_top(pre[o + 1 : len(pre) - 1])
        variadic_spelled = bool(fnty_params) and fnty_params[-1] == "..."
        head = KEYWORD_NOISE.sub("", _CC_ALT.sub("", pre[:o])).strip()
        ret = head.split()[-1] if head.split() else "?"
        params = [p for p in fnty_params if p != "..."]
        args = tuple(_norm_ty(p) for p in params if p)
    else:
        head = KEYWORD_NOISE.sub("", _CC_ALT.sub("", pre)).strip()
        ret = head.split()[-1] if head.split() else "?"
        end = _match_paren(rest, sm.end() - 1)
        raw_args = _split_top(rest[sm.end() : end - 1])
        args = tuple(_norm_ty(a) for a in raw_args if a and a != "...")
    return sym, cc, ret, args, variadic_spelled


def scan_module(path: Path, include_decl_sig: bool = False):
    defs: dict[str, tuple] = {}
    decls: dict[str, tuple] = {}
    calls: dict[str, set] = defaultdict(set)
    sample: dict[tuple, int] = {}
    with open(path, encoding="utf-8", errors="replace") as fh:
        for ln, line in enumerate(fh, 1):
            s = line.strip()
            if s.startswith("define "):
                p = parse_def(s[7:])
                if p:
                    defs[p[0]] = p[1:]
            elif s.startswith("declare "):
                p = parse_def(s[8:])
                if p:
                    decls[p[0]] = p[1:]
            elif "@" in line and CALL_RE.search(line):
                p = parse_call(line)
                if p:
                    key = p[1:]
                    calls[p[0]].add(key)
                    sample.setdefault((p[0], *key), ln)

    findings = []
    for sym, shapes in sorted(calls.items()):
        is_def = sym in defs
        auth = defs.get(sym) or decls.get(sym)
        if auth is None:
            continue
        a_cc, a_ret, a_args, a_var = auth
        kind = "def" if is_def else "decl"
        for c_cc, c_ret, c_args, c_var in shapes:
            row = None
            if c_cc != a_cc:
                row = ("CC", f"authority {a_cc} ({kind}), call {c_cc}")
            elif not is_def and not include_decl_sig:
                continue  # decl-vs-decl SIG/VAR drift cannot fold — see header
            elif (c_var or len(c_args) > len(a_args)) and not a_var and is_def:
                row = (
                    "VAR",
                    f"variadic/extra-arg call ({list(c_args)}) against fixed-arity "
                    f"def ({list(a_args)})",
                )
            elif a_var:
                if list(c_args[: len(a_args)]) != list(a_args):
                    row = (
                        "SIG",
                        f"fixed prefix {list(a_args)} ({kind}), call {list(c_args)}",
                    )
            elif c_args != a_args:
                row = ("SIG", f"args {list(a_args)} ({kind}), call {list(c_args)}")
            elif ("void" in (c_ret, a_ret)) and c_ret != a_ret:
                row = ("SIG", f"ret {a_ret} ({kind}), call ret {c_ret}")
            if row:
                findings.append(
                    (
                        row[0],
                        sym,
                        row[1],
                        sample.get((sym, c_cc, c_ret, c_args, c_var), 0),
                    )
                )
    return findings


def main() -> int:
    argv = sys.argv[1:]
    check = "--check" in argv
    include_decl_sig = "--all" in argv
    paths = [Path(a) for a in argv if a not in ("--check", "--all")]
    if not paths:
        print(__doc__)
        return 2
    total = 0
    for path in paths:
        if not path.is_file():
            print(f"ERROR: {path} not found")
            return 2
        findings = scan_module(path, include_decl_sig)
        if findings:
            print(f"== {path.name}: {len(findings)} same-module mismatches ==")
            for cls, sym, detail, ln in findings:
                print(f"  [{cls}] {sym}: {detail}  (e.g. line {ln})")
            total += len(findings)
        else:
            print(f"== {path.name}: clean ==")
    if total:
        print(f"\nTOTAL: {total} same-module mismatches — each is a §9.99 landmine")
        return 1 if check else 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
