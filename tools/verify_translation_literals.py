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

WHAT THIS CANNOT SEE, stated plainly because the gap is in the middle of its own purpose.
A literal the Rust HOISTS into a named static is compared by presence, not count (see
`check_pair`) -- the count is uninformative there. So a permutation AMONG HOISTED CONSTANTS
is invisible: swap `pszShellFoldersKey` and `pszUserShellFoldersKey` in a table of
`NAME.as_ptr()` and every literal is still present exactly once. That is not hypothetical.
userenv's setup.rs shipped a second copy of StandardProfiles whose two `hRootKey` fields
were HKEY_LOCAL_MACHINE and HKEY_CLASSES_ROOT where the C says HKEY_USERS and
HKEY_LOCAL_MACHINE -- a permutation in a table of hoisted values, found by reading the two
tables side by side, not by this checker. For tables built out of named constants the
oracle is still a field-by-field diff against the C.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
# (Rust module dir, C source dir), one row per whole-file-translated module directory.
#
# HAND-VERIFIED, never derived. Pairing modules by "which C dir holds the most matching
# stems" looks like it would keep this list current for free, and it is wrong often enough
# to be dangerous: a one-file module votes on a single common stem, so `wsock32` pairs to
# `sdk/lib/crt/misc`, `snmpapi` to `base/applications/network/net`, and
# `shell32-rust/src/dialogs` to `ntoskrnl/cc`. Diffing a translation against an unrelated
# C file reports every literal in BOTH as a finding, and the noise is what gets a checker
# switched off. A row that has to be added by hand is a row someone looked at.
PAIRS = [
    ("sdk/lib/rust/msvcrt-crt-rust/src/msvcrt", "dll/win32/msvcrt"),
    # Tier-3 native userland (batches A-H) — every module that has been cohab-flipped.
    ("sdk/lib/rust/aclui-rust/src/aclui", "dll/win32/aclui"),
    ("sdk/lib/rust/authz-rust/src/authz", "dll/win32/authz"),
    ("sdk/lib/rust/browser-rust/src/browser", "base/services/browser"),
    ("sdk/lib/rust/desk-rust/src/desk", "dll/cpl/desk"),
    ("sdk/lib/rust/intl-rust/src/intl", "dll/cpl/intl"),
    ("sdk/lib/rust/mprapi-rust/src/mprapi", "dll/win32/mprapi"),
    ("sdk/lib/rust/msv1_0-rust/src/msv1_0", "dll/win32/msv1_0"),
    ("sdk/lib/rust/msv1_0-rust/src/ntlm", "dll/win32/msv1_0/ntlm"),
    ("sdk/lib/rust/mswsock-rust/src/mswsock", "dll/win32/mswsock"),
    ("sdk/lib/rust/netcfgx-rust/src/netcfgx", "dll/win32/netcfgx"),
    ("sdk/lib/rust/netid-rust/src/netid", "dll/win32/netid"),
    ("sdk/lib/rust/ntmarta-rust/src/ntmarta", "dll/win32/ntmarta"),
    ("sdk/lib/rust/rasadhlp-rust/src/rasadhlp", "dll/win32/rasadhlp"),
    ("sdk/lib/rust/rasapi32-rust/src/rasapi32", "dll/win32/rasapi32"),
    ("sdk/lib/rust/rasdlg-rust/src/rasdlg", "dll/win32/rasdlg"),
    ("sdk/lib/rust/rasman-rust/src/rasman", "dll/win32/rasman"),
    ("sdk/lib/rust/samlib-rust/src/samlib", "dll/win32/samlib"),
    ("sdk/lib/rust/snmpapi-rust/src/snmpapi", "dll/win32/snmpapi"),
    ("sdk/lib/rust/sysdm-rust/src/sysdm", "dll/cpl/sysdm"),
    ("sdk/lib/rust/syssetup-rust/src/syssetup", "dll/win32/syssetup"),
    ("sdk/lib/rust/umpnpmgr-rust/src/umpnpmgr", "base/services/umpnpmgr"),
    ("sdk/lib/rust/userenv-rust/src/userenv", "dll/win32/userenv"),
    ("sdk/lib/rust/wkssvc-rust/src/wkssvc", "base/services/wkssvc"),
    ("sdk/lib/rust/wlanapi-rust/src/wlanapi", "dll/win32/wlanapi"),
    ("sdk/lib/rust/wlansvc-rust/src/wlansvc", "base/services/wlansvc"),
    ("sdk/lib/rust/wldap32-rust/src/wldap32", "dll/win32/wldap32"),
    ("sdk/lib/rust/wshirda-rust/src/wshirda", "dll/win32/wshirda"),
    ("sdk/lib/rust/wshtcpip-rust/src/wshtcpip", "dll/win32/wshtcpip"),
    ("sdk/lib/rust/wsock32-rust/src/wsock32", "dll/win32/wsock32"),
    # Applications (base/applications), ported with `rustconv port` 2026-10-09/10.
    ("sdk/lib/rust/notepad-rust/src/notepad", "base/applications/notepad"),
    ("sdk/lib/rust/cacls-rust/src/cacls", "base/applications/cacls"),
    ("sdk/lib/rust/charmap-rust/src/charmap", "base/applications/charmap"),
    ("sdk/lib/rust/clipbrd-rust/src/clipbrd", "base/applications/clipbrd"),
    ("sdk/lib/rust/control-rust/src/control", "base/applications/control"),
    ("sdk/lib/rust/dxdiag-rust/src/dxdiag", "base/applications/dxdiag"),
    ("sdk/lib/rust/extrac32-rust/src/extrac32", "base/applications/extrac32"),
    ("sdk/lib/rust/findstr-rust/src/findstr", "base/applications/findstr"),
    ("sdk/lib/rust/fontview-rust/src/fontview", "base/applications/fontview"),
    ("sdk/lib/rust/logoff-rust/src/logoff", "base/applications/logoff"),
    ("sdk/lib/rust/magnify-rust/src/magnify", "base/applications/magnify"),
    ("sdk/lib/rust/mmc-rust/src/mmc", "base/applications/mmc"),
    ("sdk/lib/rust/mplay32-rust/src/mplay32", "base/applications/mplay32"),
    ("sdk/lib/rust/msconfig-rust/src/msconfig", "base/applications/msconfig"),
    ("sdk/lib/rust/mstsc-rust/src/mstsc", "base/applications/mstsc"),
    ("sdk/lib/rust/osk-rust/src/osk", "base/applications/osk"),
    ("sdk/lib/rust/runas-rust/src/runas", "base/applications/runas"),
    ("sdk/lib/rust/sc-rust/src/sc", "base/applications/sc"),
    ("sdk/lib/rust/sdbinst-rust/src/sdbinst", "base/applications/sdbinst"),
    ("sdk/lib/rust/shutdown-rust/src/shutdown", "base/applications/shutdown"),
    ("sdk/lib/rust/winver-rust/src/winver", "base/applications/winver"),
]

DEBUG_CHANNELS = (
    "TRACE",
    "WARN",
    "ERR",
    "FIXME",
    "MESSAGE",
    # ReactOS's own channel, which the native modules use instead of wine/debug.h. Stripped
    # for the SAME reason and on the same terms: a translated body either lowers these to a
    # no-op `macro_rules!` or drops the call outright, uniformly, by convention.
    #
    # This is NOT the dropped-diagnostic class going unwatched -- `scan_dropped_prints` owns
    # it and carries its own baseline of the functions that lost their DPRINT. Counting it
    # HERE as well buried this checker's actual signal: across the tier-3 modules the DPRINT
    # formats were ~900 of ~950 findings, and underneath them sat real ones nobody would
    # have reached -- userenv desktop.c's `%HOMEDRIVE%%HOMEPATH%` absent entirely, and its
    # `.lnk` appearing 4x in the C against 2x in the Rust.
    "DPRINT",
    "DPRINT1",
    "DPRINT2",
    "DbgPrint",
    "DbgPrintEx",
)
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
RUST_NOOP_CHANNELS = (
    "TRACE!",
    "WARN!",
    "ERR!",
    "FIXME!",
    "MESSAGE!",
    "DPRINT!",
    "DPRINT1!",
    "DPRINT2!",
    "DbgPrint!",
    "DbgPrintEx!",
)


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
    # `#if 0` is how C parks a block without deleting it, and the translation correctly drops
    # it. umpnpmgr's rpcserver.c parks the 2k/XP `\pipe\ntsvcs` endpoint that way; leaving both
    # arms in reported the Rust as having lost an RPC endpoint it was right not to register.
    if re.fullmatch(r"\d+", expr):
        return bool(int(expr))
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


# A raw control character and its escape are the SAME string; C and Rust just disagree on
# which to spell. sysdm's virtmem.c writes `L"%s  [%s]\t%s"` and the translation embedded a
# literal TAB byte, which read as one MISSING and one EXTRA for a pair that is identical at
# runtime. Normalise to the escape on both sides.
_RAW_CTRL = {"\t": "\\t", "\n": "\\n", "\r": "\\r"}


def _norm_escapes(lit: str) -> str:
    return "".join(_RAW_CTRL.get(ch, ch) for ch in lit)


_C_STR = re.compile(r'"((?:[^"\\\n]|\\.)*)"')


def _c_strings(src: str) -> list[str]:
    """Every C string literal, with ADJACENT ones concatenated as the compiler concatenates them.

    C joins string literals separated only by whitespace into one object, and translations
    quite reasonably emit the joined result. userenv's profile.c spells one message as three
    adjacent literals across three lines; the Rust holds the single joined string, so a
    literal-by-literal compare reported three MISSING and one EXTRA for a translation that is
    exactly right. Merging here is not a filter -- the joined string is what the C program
    actually contains.
    """
    out: list[str] = []
    prev_end = None
    for m in _C_STR.finditer(src):
        # The gap holds whitespace and, for a WIDE literal, the next one's `L` prefix -- which
        # is why a whitespace-only test silently merged nothing in this tree: every literal
        # worth comparing here is an `L"..."`.
        gap = src[prev_end : m.start()].strip() if prev_end is not None else None
        if gap is not None and gap in ("", "L", "u", "U", "u8"):
            out[-1] += m.group(1)
        else:
            out.append(m.group(1))
        prev_end = m.end()
    return out


def c_literals(path: Path) -> Counter:
    src = strip_c_comments(path.read_text(errors="replace"))
    src = strip_untaken_arms(src)
    src = re.sub(r'^\s*#\s*include\s+"[^"]*"', " ", src, flags=re.M)
    src = strip_calls(src, DEBUG_CHANNELS + C_ASM_MACROS)
    # Strip one trailing NUL on THIS side too. `_dewiden` already does it for the Rust, and
    # doing it on only one side is a false positive generator: netcfgx's `L"0.0.0.0\0"` (an
    # embedded NUL, REG_MULTI_SZ style) compared against a Rust `w!("0.0.0.0\0")` that means
    # the identical thing read as one MISSING plus one EXTRA.
    return Counter(_norm_escapes(_strip_trailing_nul(s)) for s in _c_strings(src))


# A wide literal spelled as interleaved bytes: `C\0O\0M\0S\0P\0E\0C\0\0` is the C's
# `L"COMSPEC"`. Collapsing it is what lets a wide string be compared with the C at all;
# without this every `L"..."` reads as one missing and one extra literal.
_WIDE = re.compile(r"^(?:(?:[^\\]|\\[^0])\\0)+$")


def _is_nul_escape_at(lit: str, i: int) -> bool:
    r"""Is `lit[i:i+2]` the escape `\0`, rather than a backslash followed by the DIGIT zero?

    Both spell `\0` in source text and only the backslash count tells them apart. sysdm's
    general.c opens `HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0` -- the CPU-0 subkey,
    whose last three characters are escaped-backslash then '0'. Peeling that as a terminator
    left `...CentralProcessor\` and reported the translation as having corrupted a registry
    path it had copied perfectly.
    """
    if lit[i : i + 2] != "\\0":
        return False
    n, j = 0, i - 1
    while j >= 0 and lit[j] == "\\":
        n += 1
        j -= 1
    return n % 2 == 0


def _strip_trailing_nul(lit: str) -> str:
    """Drop one trailing NUL escape, the terminator a C literal does not spell in source."""
    if len(lit) >= 2 and _is_nul_escape_at(lit, len(lit) - 2):
        return lit[:-2]
    return lit


def _dewiden(lit: str) -> str:
    # `\0` and `\x00` are the same byte; the assembler emits one and hand-written bodies the
    # other. Without this, every NUL-terminated literal reads as a mismatched pair.
    lit = lit.replace(r"\x00", r"\0")
    # A WIDE literal's terminator is two NUL bytes, a narrow one's is one. Try the wide
    # shape first -- stripping only one NUL leaves a trailing `\0` that fails the interleave
    # test, which is how this silently reported every `L"..."` as a mismatched pair.
    body = lit
    if lit.endswith(r"\0\0") and _is_nul_escape_at(lit, len(lit) - 4) and _WIDE.match(lit[:-4]):
        body = lit[:-4]
    elif _is_nul_escape_at(lit, len(lit) - 2):
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
    r"(?:\s*#\[[^\]]*\]\s*\n)*"
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
    lits += _u16_array_literals(text)
    return Counter(_norm_escapes(x) for x in lits)


# `static NAME: [u16; N] = [b'S' as u16, ..., 0];` -- the THIRD way a wide literal is spelled,
# alongside `b"S\0y\0"` and a plain `"..."`. Reading only the two string forms made this
# checker report every array-spelled literal as MISSING: userenv alone raised 55 findings and
# all but two were `static ENV_USERPROFILE: [u16; 12]` sitting under its own `// L"USERPROFILE"`
# comment. That is the failure mode worth naming -- a checker whose blind spot manifests as a
# WALL of confident findings costs more than one that stays quiet, because the real two are in
# there and nobody digs to them.
#
# The array's ELEMENTS are decoded, not the intent comment above it. The comment is prose and
# `scan_wide_literals` already owns checking one against the other; trusting it here would make
# a wrong array agree with the C it disagrees with.
# Matched by SHAPE, not by declaration syntax. Keying on the declaration meant a new finding
# every time the corpus used a spelling the regex had not met -- `static NAME: [u16; N]`, then
# `const NAME: &[u16]`, then `let NAME: [u16; 8]`, then wkssvc's untyped
# `let ncacn_np = &[0x6Eu16, ...]` (suffix on the first element), then sysdm's bare
# `['S' as u16, 't' as u16, 0].as_ptr()` with no binding at all. Each gap presented as a
# confident MISSING against code that was present and usually correct.
#
# So: any bracketed list of integer-ish elements that is NUL-terminated and otherwise entirely
# printable ASCII is a wide string literal, however it is bound. The terminator plus the
# printable-only rule is what keeps ordinary numeric tables out -- they do not end in 0 and do
# not read as text.
_ARRAY_ELEM = re.compile(
    r"""b?'(\\.|[^'])'\s*as\s*u16|0[xX]([0-9a-fA-F]+)(?:u\d+)?|(\d+)(?:u\d+)?"""
)
_ARRAY_ELEM_ONLY = re.compile(
    r"""\A\s*(?:(?:b?'(?:\\.|[^'])'\s*as\s*u16|0[xX][0-9a-fA-F]+(?:u\d+)?|\d+(?:u\d+)?)"""
    r"""\s*,\s*)*(?:b?'(?:\\.|[^'])'\s*as\s*u16|0[xX][0-9a-fA-F]+(?:u\d+)?|\d+(?:u\d+)?)?\s*\Z"""
)
_ELEM_ESCAPES = {"\\\\": "\\", "\\'": "'", '\\"': '"', "\\n": "\n", "\\r": "\r", "\\t": "\t"}
# Re-escape a decoded char back to the SOURCE spelling both sides are captured in. `c_literals`
# and `rust_literals` both keep escapes uninterpreted (a C `"a\\b"` is captured as the four
# characters `a\\b`), so a decoded array has to be written the same way or it never matches.
_REESCAPE = {"\\": "\\\\", '"': '\\"', "\n": "\\n", "\r": "\\r", "\t": "\\t"}


def _u16_array_literals(text: str) -> list[str]:
    """Every NUL-terminated printable-ASCII array in `text`, decoded to its source spelling."""
    out = []
    for start in (mm.start() for mm in re.finditer(r"\[", text)):
        depth, i = 0, start
        while i < len(text):
            if text[i] == "[":
                depth += 1
            elif text[i] == "]":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        else:
            continue
        body = text[start + 1 : i]
        if not _ARRAY_ELEM_ONLY.match(body):
            continue
        codes, ok = [], True
        for e in _ARRAY_ELEM.finditer(body):
            if e.group(1) is not None:
                ch = e.group(1)
                if ch.startswith("\\"):
                    if ch not in _ELEM_ESCAPES:
                        ok = False  # an escape we do not model -- refuse to guess
                        break
                    codes.append(ord(_ELEM_ESCAPES[ch]))
                else:
                    codes.append(ord(ch))
            else:
                codes.append(int(e.group(2), 16) if e.group(2) is not None else int(e.group(3)))
        # Must be NUL-terminated: that is the wide-string signature, and what separates a
        # string from an ordinary numeric table.
        # Two elements is the floor, not three: a one-character wide string is `[c, 0]`, and
        # userenv spells the path separator exactly that way eight times.
        if not ok or len(codes) < 2 or codes[-1] != 0:
            continue
        # Strip ALL trailing NULs, not one. An array is routinely declared longer than its
        # string and zero-padded (desk's `g_CPColors: [u16; 23]` holds 20 characters and three
        # zeros), and treating the padding as an interior NUL rejected the whole literal --
        # reporting a correct, present string as missing.
        while codes and codes[-1] == 0:
            codes.pop()
        # Text only, and no interior NUL. Tab/LF/CR count as text -- userenv's profile-load
        # failure message embeds `\n`, and excluding it dropped the whole string. An interior
        # NUL means this is not a plain string (a REG_MULTI_SZ, or the DOUBLE-INTERLEAVED shape
        # wkssvc shipped); skipping it is right, because claiming it holds the whole string is
        # precisely the bug.
        if not codes or any(c not in (0x09, 0x0A, 0x0D) and not 0x20 <= c <= 0x7E for c in codes):
            continue
        out.append("".join(_REESCAPE.get(chr(c), chr(c)) for c in codes))
    return out


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
    # A literal the Rust HOISTS into a named `[u16; N]` static is present exactly once on this
    # side however many times the C spells it, because each use site is `NAME.as_ptr()` and not
    # a literal at all. So for those the COUNT carries no information -- it is 1 whether the
    # translation kept all four uses or none -- and comparing it only manufactured findings:
    # userenv's `SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList` read as "10x in the
    # C and 5x here" purely because the C repeats the string and the Rust names it.
    #
    # Dropping to PRESENCE for these loses no signal. It is not a weakening of the permutation
    # check either: a permuted literal (undname's `pascal` -> `__stdcall`) sits inline in a
    # match arm, is never hoisted, and is still compared by count.
    hoisted = set(_u16_array_literals(strip_rust_comments(rs.read_text(errors="replace"))))
    for lit in sorted(cl - rl_seen):
        if lit in hoisted and rl_seen[lit] >= 1:
            continue
        findings.append(
            f"MISSING: {rel}: {lit!r} appears {cl[lit]}x in {c.name} and {rl_seen[lit]}x here"
        )
    for lit in sorted(rl - cl):
        if lit in hoisted and cl[lit] >= 1:
            continue
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
