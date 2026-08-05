#!/usr/bin/env python3
"""Diff a whole-file translation's STRING LITERALS against the C it came from.

The oracle nothing else provides. A translated body can satisfy every check we run --
`cargo check` on three arches, the sealed-constant envelope, `verify_cohab_coverage`, a
green boot -- while carrying a literal that is real, is spelled correctly, and is in the
WRONG SLOT. That is the recall-permutation failure mode: the model writes a famous body
from memory instead of from the text, and a closed-SET invariant cannot see a permutation
because the set is unchanged.

Worked example, and the reason this file exists. `undname.c`'s calling-convention switch:

    case 'C': case 'D': *call_conv = "pascal";      /* the C */
    b'C' | b'D' => *call_conv = b"__stdcall\0"      /* what shipped in the translation */

`__stdcall` is a genuine member of that very switch eight lines further down. Nothing
about it is invented, nothing fails to compile, and `?f@@YCXXZ` simply demangles wrong
forever. As a MULTISET the difference is loud: one `pascal` missing, one `__stdcall` too
many. This is that comparison, per file.

It is deliberately a multiset and not a set: a literal moved from one arm to another
changes no set but changes the counts on both sides.

    python3 tools/verify_translation_literals.py            # report
    python3 tools/verify_translation_literals.py --check    # exit 1 on any finding

Pairing is by the repo's own "one module per C file" convention (see
`msvcrt-crt-rust/src/msvcrt/mod.rs`): `src/msvcrt/<name>.rs` is the translation of
`dll/win32/msvcrt/<name>.c`. A module with no matching .c is skipped and named, so a new
whole-file module is covered the day it lands without touching this file.

WHAT IS FILTERED, AND WHY EACH IS SAFE

  * wine/debug.h channel calls (TRACE/WARN/ERR/FIXME/MESSAGE). Every translated msvcrt
    module lowers these to no-op `macro_rules!`, which drops their format strings and
    their arguments. That is a deliberate, uniform convention, not a per-file accident.
  * `#include "..."` names, which are not literals of the program at all.
  * Rust items under `#[cfg(any())]` -- the repo's spelling for "this C block is not
    compiled in this configuration" (math.c's `#ifndef __REACTOS__` blocks, undname.c's
    `#ifdef _UCRT` arms). Their literals are inert on BOTH sides, so they are dropped
    from the Rust and expected to be absent from the C.

Anything else that differs is reported. Known-and-accepted differences belong in the lint
baseline, where they are visible, rather than in a filter here, where they would not be.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
PAIRS = [
    # (Rust module dir, C source dir)
    ("sdk/lib/rust/msvcrt-crt-rust/src/msvcrt", "dll/win32/msvcrt"),
]

DEBUG_CHANNELS = ("TRACE", "WARN", "ERR", "FIXME", "MESSAGE")
# Assembly is not a translation of the C's text -- `__ASM_GLOBAL_FUNC`'s operand string and
# a `global_asm!` body say the same thing in different syntaxes, so comparing them as
# literals is noise. The instructions themselves are covered by the offset/ABI verifiers.
C_ASM_MACROS = (
    "__ASM_GLOBAL_FUNC",
    "__ASM_STDCALL_FUNC",
    "__ASM_DEFINE_FUNC",
    # `__asm__ __volatile__(...)` puts the qualifier between the
    # keyword and the paren, so the qualifier is the token to match.
    "__volatile__",
    "__asm__",
    "__asm",
    "asm",
)
RUST_ASM_MACROS = ("global_asm!", "asm!", "core::arch::global_asm!", "core::arch::asm!")
RUST_NOOP_CHANNELS = ("TRACE!", "WARN!", "ERR!", "FIXME!", "MESSAGE!")


def strip_c_comments(src: str) -> str:
    # Blank them rather than delete, so nothing on either side of a comment joins up.
    src = re.sub(r"/\*.*?\*/", " ", src, flags=re.S)
    return re.sub(r"//[^\n]*", " ", src)


def strip_calls(src: str, names: tuple[str, ...]) -> str:
    """Remove `NAME(...)` calls, arguments and all, by balanced-paren scan."""
    pat = re.compile(
        r"(?<![\w])(" + "|".join(re.escape(n).replace(r"\!", "!") for n in names) + r")\s*[\(\[\{]"
    )
    out = []
    i = 0
    while i < len(src):
        m = pat.search(src, i)
        if not m:
            out.append(src[i:])
            break
        out.append(src[i : m.start()])
        opens, closes = "([{", ")]}"
        depth = 0
        j = m.end() - 1
        in_str: str | None = None
        while j < len(src):
            ch = src[j]
            if in_str:
                if ch == "\\":
                    j += 2
                    continue
                if ch == in_str:
                    in_str = None
            elif ch in "\"'":
                in_str = ch
            elif ch in opens:
                depth += 1
            elif ch in closes:
                depth -= 1
                if depth == 0:
                    break
            j += 1
        i = j + 1
    return "".join(out)


# The build's own settings for the conditionals that actually gate msvcrt's sources. A C
# literal inside a NOT-TAKEN arm is not part of the program, so comparing it against the
# Rust reports a difference that is only in the source text: `asctime`'s `"%s %s %2d ..."`
# is the `_MSVCR_VER>=140` arm and the shipping one is `%02d`, and reading that as a bug
# is a false positive that costs a real investigation.
#
# Deliberately a SHORT list of exactly what this module is compiled with (see the
# `VER_DEFS` that `rustconv.profile_derive` derives from CMake). Anything not named here
# leaves BOTH arms in, so an unrecognised conditional over-reports rather than hiding a
# literal -- the safe direction for a checker.
C_DEFINES = {"__REACTOS__": 1, "_MSVCR_VER": 0, "__UCRTSUPPORT__": 1}
C_UNDEFINED = {"_UCRT", "_CRTDLL"}


def _eval_cond(expr: str) -> bool | None:
    """True/False when the build settles this condition, None when it does not."""
    expr = expr.strip()
    m = re.fullmatch(r"defined\s*\(?\s*(\w+)\s*\)?", expr)
    if m:
        if m.group(1) in C_DEFINES:
            return True
        if m.group(1) in C_UNDEFINED:
            return False
        return None
    m = re.fullmatch(r"!\s*defined\s*\(?\s*(\w+)\s*\)?", expr)
    if m:
        inner = _eval_cond(f"defined({m.group(1)})")
        return None if inner is None else not inner
    m = re.fullmatch(r"(\w+)\s*(==|!=|<=|>=|<|>)\s*(\d+)", expr)
    if m and m.group(1) in C_DEFINES:
        lhs, op, rhs = C_DEFINES[m.group(1)], m.group(2), int(m.group(3))
        return {
            "==": lhs == rhs,
            "!=": lhs != rhs,
            "<=": lhs <= rhs,
            ">=": lhs >= rhs,
            "<": lhs < rhs,
            ">": lhs > rhs,
        }[op]
    return None


def strip_untaken_arms(src: str) -> str:
    """Blank the arms of #if/#ifdef/#ifndef chains the build does not compile."""
    out, stack = [], []  # stack entries: (decided, keeping)
    for line in src.split("\n"):
        s = line.strip()
        m = re.match(r"#\s*(ifdef|ifndef|if|elif|else|endif)\b(.*)", s)
        if m:
            kind, rest = m.group(1), m.group(2)
            if kind in ("ifdef", "ifndef", "if"):
                cond = (
                    f"defined({rest.strip()})"
                    if kind == "ifdef"
                    else f"!defined({rest.strip()})"
                    if kind == "ifndef"
                    else rest
                )
                v = _eval_cond(cond)
                stack.append((v is not None, True if v is None else v))
            elif kind == "elif" and stack:
                decided, keeping = stack[-1]
                v = _eval_cond(rest)
                # Once a decided chain has taken an arm, later arms are dead.
                stack[-1] = (
                    decided and v is not None,
                    True if (not decided or v is None) else (v and not keeping),
                )
            elif kind == "else" and stack:
                decided, keeping = stack[-1]
                stack[-1] = (decided, (not keeping) if decided else True)
            elif kind == "endif" and stack:
                stack.pop()
            out.append("")
            continue
        out.append(line if all(k for _, k in stack) else "")
    return "\n".join(out)


def c_literals(path: Path) -> Counter:
    src = strip_c_comments(path.read_text(errors="replace"))
    src = strip_untaken_arms(src)
    src = re.sub(r'^\s*#\s*include\s+"[^"]*"', " ", src, flags=re.M)
    src = strip_calls(src, DEBUG_CHANNELS + C_ASM_MACROS)
    return Counter(re.findall(r'"((?:[^"\\\n]|\\.)*)"', src))


# A wide literal spelled as interleaved bytes: `C\0O\0M\0S\0P\0E\0C\0\0` is the C's
# `L"COMSPEC"`. Collapsing it is what lets a wide string be compared with the C at all;
# without this every `L"..."` reads as one missing and one extra literal.
_WIDE = re.compile(r"^(?:(?:[^\\]|\\[^0])\\0)+$")


def _dewiden(lit: str) -> str:
    # `\0` and `\x00` are the same byte; the assembler emits one and hand-written bodies the
    # other. Without this, every NUL-terminated literal reads as a mismatched pair.
    lit = lit.replace(r"\x00", r"\0")
    # A WIDE literal's terminator is two NUL bytes, a narrow one's is one. Try the wide
    # shape first -- stripping only one NUL leaves a trailing `\0` that fails the interleave
    # test, which is how this silently reported every `L"..."` as a mismatched pair.
    body = lit
    if lit.endswith(r"\0\0") and _WIDE.match(lit[:-4]):
        body = lit[:-4]
    elif lit.endswith(r"\0"):
        body = lit[:-2]
    if not body or not _WIDE.match(body):
        return body
    out, i = [], 0
    while i < len(body):
        if body[i] == "\\":
            out.append(body[i : i + 2])
            i += 2
        else:
            out.append(body[i])
            i += 1
        if body[i : i + 2] != r"\0":
            return body
        i += 2
    return "".join(out)


def _skip_cfg_any_items(text: str) -> str:
    """Blank out each item/arm carrying `#[cfg(any())]` -- the not-compiled-here marker."""
    out = []
    i = 0
    for m in re.finditer(r"#\[cfg\(any\(\)\)\]", text):
        out.append(text[i : m.start()])
        # Walk to the end of the item this attribute decorates: either the first balanced
        # `{...}` that opens after it, or the line's terminating `,`/`;` if there is none.
        j = m.end()
        brace = text.find("{", j)
        semi = min((x for x in (text.find(";", j), text.find(",\n", j)) if x >= 0), default=-1)
        if brace < 0 or (semi >= 0 and semi < brace):
            i = (semi + 1) if semi >= 0 else j
            continue
        depth = 0
        k = brace
        while k < len(text):
            if text[k] == "{":
                depth += 1
            elif text[k] == "}":
                depth -= 1
                if depth == 0:
                    break
            k += 1
        i = k + 1
    out.append(text[i:])
    return "".join(out)


def strip_rust_comments(text: str) -> str:
    """Drop `//` and `/* */`, respecting string and char literals.

    Not cosmetic: `_skip_cfg_any_items` looks for the `;` or `,` that ends an attributed
    item, and a `;` inside the attribute's own trailing comment stops it at the wrong
    place -- which silently un-skips a `#[cfg(any())]` block.
    """
    out, i, n = [], 0, len(text)
    while i < n:
        c = text[i]
        if c == "'" and not re.match(r"'(?:\\.|[^'\\])'", text[i : i + 4]):
            # A LIFETIME or loop label (`'fail:`), not a char literal. Treating it as one
            # scans to the next quote and desynchronises the rest of the file — which is
            # how a `//`-commented line downstream leaked a fragment of its string.
            out.append(c)
            i += 1
        elif c == '"' or c == "'":
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == c or text[j] == "\n":
                    break
                j += 1
            out.append(text[i : j + 1])
            i = j + 1
        elif text.startswith("//", i):
            j = text.find("\n", i)
            i = n if j < 0 else j
        elif text.startswith("/*", i):
            j = text.find("*/", i)
            i = n if j < 0 else j + 2
        else:
            out.append(c)
            i += 1
    return "".join(out)


def strip_rust_attributes(text: str) -> str:
    """Drop `#[...]` / `#![...]`, which carry Rust's own vocabulary (`target_arch = "x86"`)."""
    out, i, n = [], 0, len(text)
    while i < n:
        m = re.compile(r"#!?\[").search(text, i)
        if not m:
            out.append(text[i:])
            break
        out.append(text[i : m.start()])
        depth, j = 0, m.end() - 1
        while j < n:
            if text[j] == "[":
                depth += 1
            elif text[j] == "]":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        i = j + 1
    return "".join(out)



_ARCH_ARM = re.compile(
    r'#\[cfg\((not\()?target_arch\s*=\s*"[^"]+"\)?\)\]\s*\n'
    r'(?:\s*#\[[^\]]*\]\s*\n)*'
    r'\s*(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]*"\s+)?fn\s+(\w+)'
)


def _collapse_arch_duals(text: str) -> str:
    """Blank the `not(target_arch)` twin of a per-arch DUAL, so its literals count once.

    A dual is two definitions of the SAME function name, one under
    `#[cfg(target_arch = "x86")]` and one under `#[cfg(not(target_arch = "x86"))]`, holding
    the same body. The C has one body, so counting both makes every string in it look
    duplicated: msvcrt's cpp.c reported `'bad cast' appears 2x here and 1x in cpp.c` for
    thirty-two C++ methods, none of which is a defect.

    Only a matched PAIR is collapsed. A lone `#[cfg(not(target_arch = ...))]` item is real
    arch-conditional code with its own `#ifdef` in the C (cpp.c's RVA-only RTTI arm is one),
    and blanking that would invent a MISSING finding for every string in it.
    """
    spans = {}
    for m in _ARCH_ARM.finditer(text):
        spans.setdefault(m.group(2), []).append((bool(m.group(1)), m.start()))
    kill = []
    for _name, arms in spans.items():
        if len(arms) != 2:
            continue
        kinds = {is_not for is_not, _ in arms}
        if kinds != {True, False}:
            continue
        start = next(pos for is_not, pos in arms if is_not)
        brace = text.find("{", start)
        if brace < 0:
            continue
        depth, k = 0, brace
        while k < len(text):
            if text[k] == "{":
                depth += 1
            elif text[k] == "}":
                depth -= 1
                if depth == 0:
                    break
            k += 1
        kill.append((start, k + 1))
    out, i = [], 0
    for s, e in sorted(kill):
        if s < i:
            continue
        out.append(text[i:s])
        i = e
    out.append(text[i:])
    return "".join(out)


def rust_literals(path: Path) -> Counter:
    text = strip_rust_comments(path.read_text(errors="replace"))
    text = _skip_cfg_any_items(text)
    text = _collapse_arch_duals(text)
    # The no-op debug macros keep their format string as a token even though they expand to
    # nothing; the C side has the matching call stripped, so strip it here too.
    text = strip_calls(text, RUST_NOOP_CHANNELS + RUST_ASM_MACROS)
    # `extern "C"`, `#[cfg(target_arch = "x86")]` and `cfg!(...)` are Rust's own vocabulary,
    # not translated content -- they have no counterpart in the C by construction.
    text = re.sub(r'\bextern\s+"[^"]*"', " ", text)
    text = strip_rust_attributes(text)
    text = strip_calls(text, ("cfg!", "cfg_attr!"))

    lits = []
    for m in re.findall(r'b"((?:[^"\\\n]|\\.)*)"', text):
        # C string literals are NUL-terminated by the compiler; the Rust spells the NUL.
        lits.append(_dewiden(m))
    for m in re.findall(r'(?<![b\w])"((?:[^"\\\n]|\\.)*)"', text):
        lits.append(_dewiden(m))
    return Counter(lits)


def check_pair(rs: Path, c: Path, companions: list[Path] | None = None) -> list[str]:
    cl, rl = c_literals(c), rust_literals(rs)
    findings = []
    # A hand-written SUBSTRATE COMPANION (`<stem>_<suffix>.rs` with no `.c` of its own) is
    # part of the same port and can legitimately carry a literal the C spells in <stem>.c:
    # cpp.c's `DEFINE_RTTI_DATA1(bad_cast, …, ".?AVbad_cast@@")` lines are macro invocations,
    # so their mangled names live in the hand-written cpp_rtti.rs beside the descriptors they
    # initialise, not in the translated cpp.rs.
    #
    # It only ever CANCELS a MISSING -- a companion's own literals are never counted as EXTRA.
    # That asymmetry is deliberate: the companion exists precisely because its content comes
    # from MACROS IN HEADERS (cppexcept.h's "Unknown exception"), which `c_literals(<stem>.c)`
    # cannot see, so its extras are unverifiable against this pair rather than wrong.
    rl_seen = rl.copy()  # rl + companions: used ONLY to cancel MISSING, never to add EXTRA
    for comp in companions or []:
        rl_seen = rl_seen + rust_literals(comp)
    # Repo-relative when it can be (findings must match a baseline captured in any
    # worktree), raw otherwise -- a fixture pair under /tmp is how the controls exercise
    # this, and a checker that cannot be pointed at a fixture cannot be shown to work.
    rel = rs.relative_to(REPO) if rs.is_relative_to(REPO) else rs
    for lit in sorted(cl - rl_seen):
        findings.append(
            f"MISSING: {rel}: {lit!r} appears {cl[lit]}x in {c.name} and {rl_seen[lit]}x here"
        )
    for lit in sorted(rl - cl):
        findings.append(f"EXTRA: {rel}: {lit!r} appears {rl[lit]}x here and {cl[lit]}x in {c.name}")
    return findings


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--check", action="store_true", help="exit 1 on any finding")
    ap.add_argument("--only", help="restrict to one module name (e.g. undname)")
    args = ap.parse_args()

    findings: list[str] = []
    checked = skipped = 0
    for rs_dir, c_dir in PAIRS:
        for rs in sorted((REPO / rs_dir).glob("*.rs")):
            if rs.name == "mod.rs":
                continue
            if args.only and rs.stem != args.only:
                continue
            c = REPO / c_dir / f"{rs.stem}.c"
            if not c.is_file():
                print(f"  skip {rs.relative_to(REPO)} — no {c_dir}/{rs.stem}.c", file=sys.stderr)
                skipped += 1
                continue
            # `<stem>_*.rs` files with no `.c` of their own are this pair's substrate
            # companions -- see check_pair.
            companions = [
                o
                for o in sorted(rs.parent.glob(f"{rs.stem}_*.rs"))
                if not (REPO / c_dir / f"{o.stem}.c").is_file()
            ]
            checked += 1
            findings += check_pair(rs, c, companions)

    for f in findings:
        print(f)
    print(
        f"verify_translation_literals: {checked} pair(s) checked, {skipped} skipped, "
        f"{len(findings)} finding(s)",
        file=sys.stderr,
    )
    return 1 if (args.check and findings) else 0


if __name__ == "__main__":
    raise SystemExit(main())
