#!/usr/bin/env python3
"""
gen_ioctl_consts.py — deterministically transcribe `CTL_CODE(...)` IOCTL `#define`s
from a C header into Rust consts.

WHY THIS EXISTS
    bindgen cannot emit a function-like macro. Every `IOCTL_*` in the DDK is
        #define IOCTL_SERIAL_CLR_DTR \
            CTL_CODE(FILE_DEVICE_SERIAL_PORT, 10, METHOD_BUFFERED, FILE_ANY_ACCESS)
    so no amount of `--allowlist-file` will ever put it in a bindgen island. A
    translated `match` arm naming one is therefore an UNDEFINED identifier, which
    Rust silently turns into an irrefutable binding that swallows every input
    (field manual §9.112 / §9.112a). serial.sys shipped with all 37 of its
    DeviceIoControl arms dead behind `IOCTL_SERIAL_CLEAR_STATS`.

    The fix is transcription — but transcription by hand is exactly how §9.113
    (61 invented kernel stop codes) happened. So compute the values from the
    header instead of typing them: parse the macro, resolve its arguments through
    the real `#define`s, evaluate CTL_CODE's arithmetic, and emit the result with
    its provenance. Nothing is guessed; anything unevaluable is a hard error.

USAGE
    python3 tools/gen_ioctl_consts.py sdk/include/psdk/ntddser.h --prefix IOCTL_SERIAL_
    python3 tools/gen_ioctl_consts.py sdk/include/psdk/ntddkbd.h --prefix IOCTL_KEYBOARD_ --type u32
    ... --check sdk/lib/rust/serial-rust/src/glue.rs   # verify, don't emit
"""

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SEARCH_ROOTS = ["sdk/include"]

# `#define NAME value` — object-like only (no '(' straight after the name).
RE_OBJ_DEFINE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)[ \t]+(.*?)[ \t]*$", re.M)
# `#define NAME(a, b) body` — function-like. Needed because the interesting
# constants are rarely a bare CTL_CODE: winsock spells its IOCTLs
# `_WSAIORW(IOC_WS2, 6)`, which expands to `(IOC_INOUT|(IOC_WS2)|(6))`. Expanding
# the macro chain generically means CTL_CODE needs no special case either — it is
# just another macro defined in the headers.
RE_FUNC_DEFINE = re.compile(
    r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)\(([^)]*)\)[ \t]+(.*?)[ \t]*$", re.M
)
RE_MACRO_CALL = re.compile(r"^\s*([A-Za-z_]\w*)\s*\((.*)\)\s*$", re.S)


def read_defines(paths):
    """(object-like table, function-like table). Line-continuations are folded.
    The first definition wins, which matches include order closely enough for the
    constant families we transcribe."""
    table = {}
    functable = {}
    for p in paths:
        try:
            text = Path(p).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        text = re.sub(r"\\\n", " ", text)  # fold continuations
        for m in RE_FUNC_DEFINE.finditer(text):
            name = m.group(1)
            params = [a.strip() for a in m.group(2).split(",") if a.strip()]
            body = re.split(r"/\*|//", m.group(3))[0].strip()
            if name not in functable:
                functable[name] = (params, body)
        for m in RE_OBJ_DEFINE.finditer(text):
            name, rhs = m.group(1), m.group(2).strip()
            if rhs.startswith("("):
                # `#define NAME(a) …` already captured above; a genuinely
                # object-like `#define NAME (x|y)` still lands here correctly
                # because RE_FUNC_DEFINE requires no space before '('.
                pass
            rhs = re.split(r"/\*|//", rhs)[0].strip()
            if name not in table:
                table[name] = (rhs, str(p))
    return table, functable


def all_headers():
    out = []
    for root in SEARCH_ROOTS:
        out.extend(sorted((REPO / root).rglob("*.h")))
    return out


class Unevaluable(Exception):
    pass


INT_RE = re.compile(r"^[-+]?(0[xX][0-9a-fA-F]+|\d+)[uUlL]*$")


def expand_call(name, argstr, functable):
    """Textually expand one function-like macro invocation."""
    params, body = functable[name]
    args = split_args(argstr)
    if len(args) != len(params):
        raise Unevaluable(f"{name} takes {len(params)} args, got {len(args)}")
    binding = dict(zip(params, args, strict=True))
    return re.sub(
        r"\b([A-Za-z_]\w*)\b",
        lambda m: f"({binding[m.group(1)]})" if m.group(1) in binding else m.group(1),
        body,
    )


def evaluate(expr, table, functable=None, depth=0):
    """Evaluate a C integer constant expression using `table` for object-like
    symbols and `functable` for function-like macros. Raises Unevaluable rather
    than guessing — a wrong IOCTL value is worse than a failed run."""
    functable = functable or {}
    if depth > 24:
        raise Unevaluable(f"recursion limit on {expr!r}")
    e = expr.strip()
    if not e:
        raise Unevaluable("empty expression")
    # strip redundant outer parens
    while e.startswith("(") and e.endswith(")") and _balanced(e[1:-1]):
        e = e[1:-1].strip()
    if INT_RE.match(e):
        return int(e.rstrip("uUlL"), 0)
    # a bare symbol
    if re.fullmatch(r"[A-Za-z_]\w*", e):
        if e not in table:
            raise Unevaluable(f"undefined symbol {e}")
        return evaluate(table[e][0], table, functable, depth + 1)
    # a cast we can ignore for value purposes, e.g. (ULONG)0x1b
    m = re.match(r"^\(\s*(?:unsigned\s+|signed\s+)?[A-Za-z_]\w*\s*\)\s*(.*)$", e)
    if m and m.group(1):
        return evaluate(m.group(1), table, functable, depth + 1)
    # a function-like macro invocation: expand and re-evaluate
    mc = RE_MACRO_CALL.match(e)
    if mc and mc.group(1) in functable:
        return evaluate(
            expand_call(mc.group(1), mc.group(2), functable), table, functable, depth + 1
        )

    # general arithmetic: substitute symbols then hand to Python, which shares
    # C's semantics for the operators these headers actually use (<< | & + - * ~).
    if not re.fullmatch(r"[0-9A-Za-z_\s()<>|&+\-*/^~]*", e):
        raise Unevaluable(f"unsupported syntax in {e!r}")
    subbed = re.sub(
        r"\b([A-Za-z_]\w*)\b",
        lambda mm: str(evaluate(mm.group(1), table, functable, depth + 1)),
        e,
    )
    subbed = re.sub(r"\b(\d+)[uUlL]+\b", r"\1", subbed)
    try:
        return int(eval(subbed, {"__builtins__": {}}, {}))
    except Exception as exc:
        raise Unevaluable(f"cannot evaluate {e!r} ({exc})") from exc


def _balanced(s):
    d = 0
    for c in s:
        if c == "(":
            d += 1
        elif c == ")":
            d -= 1
            if d < 0:
                return False
    return d == 0


def split_args(s):
    """Split a macro argument list on top-level commas."""
    args, depth, cur = [], 0, ""
    for c in s:
        if c == "," and depth == 0:
            args.append(cur)
            cur = ""
            continue
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
        cur += c
    args.append(cur)
    return [a.strip() for a in args]


def ctl_code(dev, func, method, access):
    """The DDK definition, verbatim:
    ((DeviceType) << 16) | ((Access) << 14) | ((Function) << 2) | (Method)"""
    return ((dev & 0xFFFF) << 16) | ((access & 3) << 14) | ((func & 0xFFF) << 2) | (method & 3)


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("header", help="header owning the IOCTL defines (repo-relative or abs)")
    ap.add_argument(
        "--prefix",
        action="append",
        default=[],
        help="only transcribe names with this prefix (repeatable)",
    )
    ap.add_argument("--type", default="u32", help="Rust type for the emitted consts")
    ap.add_argument(
        "--check",
        metavar="RSFILE",
        help="verify RSFILE's existing consts against the header instead of emitting",
    )
    args = ap.parse_args()

    hdr = Path(args.header)
    if not hdr.is_absolute():
        hdr = REPO / hdr
    if not hdr.is_file():
        sys.exit(f"FATAL: header not found: {hdr}")

    # Symbol table: the owning header first, then every SDK header for the
    # FILE_DEVICE_* / METHOD_* / IOC_* operands and the CTL_CODE / _WSAIO* macros.
    table, functable = read_defines([hdr, *all_headers()])

    hdr_only, _ = read_defines([hdr])
    out, failures, skipped = [], [], 0
    for name, (rhs, _src) in sorted(hdr_only.items()):
        if args.prefix and not any(name.startswith(p) for p in args.prefix):
            continue
        mc = RE_MACRO_CALL.match(rhs)
        if not mc or mc.group(1) not in functable:
            # A plain literal in the same family is fine to emit; anything else
            # (a string, a type name, a cast to a pointer) is not ours.
            if INT_RE.match(rhs):
                out.append((name, int(rhs.rstrip("uUlL"), 0), rhs))
            else:
                skipped += 1
            continue
        try:
            value = evaluate(rhs, table, functable)
        except Unevaluable as exc:
            failures.append(f"{name}: {exc}")
            continue
        out.append((name, value & 0xFFFFFFFF, rhs))

    if failures:
        # Fail loud: a partially-transcribed family is worse than none, because
        # the ones left out stay silent catch-all bindings.
        print("FATAL: could not evaluate these definitions:", file=sys.stderr)
        for f in failures:
            print(f"  {f}", file=sys.stderr)
        sys.exit(2)
    if not out:
        sys.exit(
            f"FATAL: no evaluable macro defines matched in {hdr} "
            f"({skipped} non-numeric define(s) skipped)"
        )

    if args.check:
        rs = Path(args.check)
        if not rs.is_absolute():
            rs = REPO / rs
        text = rs.read_text(encoding="utf-8", errors="replace")
        bad = 0
        for name, value, _ in out:
            m = re.search(rf"\b{re.escape(name)}\s*:\s*\w+\s*=\s*([^;]+);", text)
            if not m:
                continue
            try:
                have = evaluate(m.group(1).replace("_", ""), {}, {})
            except Unevaluable:
                print(f"  ?  {name}: cannot parse existing value {m.group(1).strip()}")
                continue
            if have != value:
                print(f"  MISMATCH {name}: file={have:#x} header={value:#x}")
                bad += 1
        print(f"checked {len(out)} constant(s) against {hdr.name}: {bad} mismatch(es)")
        return 1 if bad else 0

    rel = hdr.relative_to(REPO) if hdr.is_relative_to(REPO) else hdr
    print(f"// Generated by tools/gen_ioctl_consts.py from {rel}.")
    print("// bindgen cannot emit function-like macros, so these CTL_CODE() IOCTLs")
    print("// can never come from the bindgen island (field manual §9.112a). Values")
    print("// are COMPUTED from the header, never hand-typed — see §9.113.")
    for name, value, rhs in out:
        print(f"pub const {name}: {args.type} = {value:#010x}; // {rhs}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
