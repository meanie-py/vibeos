#!/usr/bin/env python3
"""The hand-transcribed `va_start` trampolines must stay faithful to the C they copy.

Two pairs today: `rust_varargs_shim.c` vs `wcs.c` (printf) and `rust_scanf_shim.c`
vs `scanf.c` (scanf). Both exist for the same reason and carry the same risk.

msvcrt's printf family is Rust except the `...` entry points, which cannot be:
stable Rust has no `va_start`. They live in `dll/win32/msvcrt/rust_varargs_shim.c`
as three-line `va_start` trampolines over the Rust engine, hand-copied from the
genuine `wcs.c` bodies — and they must live in their own TU, because left in wcs.c
clang keeps inlining wcs.c's C engine into them and the flip is silently undone
(symbol replacement flips a TU's callers, not its housemates).

**Why a source scanner and not a differential test.** These 30 functions cannot be
executed on a SysV host at all, by either instantiation the msvcrt difftest owns:

  * the `pfflat_*` TU compiles under a flat `char *` `<stdarg.h>` — the Windows
    model, and the only one msvcrt's plumbing is coherent under — so its `va_start`
    resolves to `(char *)&last_named + sizeof(last_named)`. That is exactly right
    on i386 cdecl and on Windows, and wrong on x86-64, where the variadic arguments
    arrive in registers and never touch that address.
  * the `ref_*` TU compiles under the host's real `<stdarg.h>`, where a `va_list` is
    `__va_list_tag[1]`. wcs.c hands `&valist` to the engine, which does
    `va_arg(*valist, T)` — an idiom that only type-checks when a `va_list` IS a
    pointer. Under SysV it is off by one indirection and segfaults.

So there is no host process in which a real `va_start` and msvcrt's engine agree,
and any harness that appeared to grade these would be grading something else. What
IS checkable, statically and exactly, is the property that actually carries risk:
30 near-identical three-line bodies where a wrong delegate or a swapped argument
compiles, links, boots, and truncates only for small counts.

Every rewrite this tolerates is PROVEN FROM THE SOURCE, never asserted here: the
scanner reads the one-line forwarder in wcs.c that makes the substitution an
identity, and fails if that forwarder ever stops saying so.

Exit 1 on any drift. `--lint` prints one finding per line for the gate.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
MSVCRT = REPO / "dll/win32/msvcrt"
WCS = MSVCRT / "wcs.c"
SHIM = MSVCRT / "rust_varargs_shim.c"
STDIO = REPO / "sdk/include/crt/stdio.h"

#: Every (genuine C, hand-transcribed trampoline TU) pair in msvcrt. Both exist for
#: the same reason — stable Rust cannot `va_start`, so the `...` entry points stay C —
#: and both carry the same risk, 30-40 near-identical three-line bodies where a wrong
#: delegate or a swapped argument compiles, links and boots. scanf's pair has a second
#: reason to exist that printf's does not: scanf.c's 16 engines are `static`, so a Rust
#: engine cannot replace them by first-match at all and the trampolines MUST live in a
#: TU the linker prefers.
PAIRS = [
    (WCS, SHIM),
    (MSVCRT / "scanf.c", MSVCRT / "rust_scanf_shim.c"),
    # file.c's 20: the 16 fprintf/printf/fwprintf/wprintf shells, plus the four
    # _open/_sopen ones whose `...` carries an optional pmode rather than a format.
    # Those four are the reason VA_MACROS exists below — their first call-shaped
    # token is `va_arg`, not the delegate.
    (MSVCRT / "file.c", MSVCRT / "rust_file_varargs_shim.c"),
]

#: `va_arg` is call-shaped and appears BEFORE the real delegate in the four
#: `_open`-family bodies (`pmode = va_arg(ap, int);` precedes `return _sopen(...)`).
#: Taking it as the delegate would still compare equal — the shim is a verbatim copy —
#: but it would compare the wrong thing, leaving the `_sopen`/`_wsopen_dispatch` call
#: and its argument order unchecked. That is the exact drift this scanner exists to
#: catch, so skip the stdarg macros and keep looking.
VA_MACROS = {"va_arg", "va_start", "va_end", "va_copy"}


def strip_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.S)
    return re.sub(r"//[^\n]*", " ", text)


def split_args(s: str) -> list[str]:
    """Top-level comma split (no nesting deeper than parens appears here)."""
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def body_of(text: str, open_brace: int) -> tuple[str, int]:
    depth, i = 0, open_brace
    while True:
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[open_brace + 1 : i], i
        i += 1


# The delegate call is either assigned (`retval = _vsnprintf(...)`) or returned
# directly (`return vswprintf_p_l_opt(...)`, which is how the one-line forwarders
# this scanner proves its rewrites from are written).
CALL_RE = re.compile(r"(?:=|\breturn)\s*([A-Za-z_]\w*)\s*\(")


def shells(path: Path, keyword: str) -> dict[str, tuple[list[str], str, list[str]]]:
    """name -> (parameter names, delegate, delegate argument expressions).

    Functions inside an `_MSVCR_VER` gate this build does not compile are skipped:
    they exist in neither binary, and requiring them on one side only would be
    noise. `gated_shells()` cross-checks that both files gate the SAME set.
    """
    text = strip_comments(path.read_text(errors="replace"))
    skip = gated_in(text)
    found = {}
    for m in re.finditer(r"^int " + keyword + r" (\w+)\s*\(", text, re.M):
        name = m.group(1)
        if name in skip:
            continue
        # parameter list: from the open paren to its match
        p_open = m.end() - 1
        depth, i = 0, p_open
        while True:
            if text[i] == "(":
                depth += 1
            elif text[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        params = text[p_open + 1 : i]
        pnames = []
        for a in split_args(params):
            a = a.strip()
            if a in ("...", "void", ""):
                continue
            w = re.findall(r"\w+", a)
            if w:
                pnames.append(w[-1])
        brace = text.index("{", i)
        body, _ = body_of(text, brace)
        # Skip the stdarg macros: they are call-shaped but they are not the delegate.
        pos = 0
        while True:
            cm = CALL_RE.search(body, pos)
            if not cm:
                break
            if cm.group(1) not in VA_MACROS:
                break
            pos = cm.end(1)
        if not cm:
            continue
        c_open = body.index("(", cm.start(1))
        depth, j = 0, c_open
        while True:
            if body[j] == "(":
                depth += 1
            elif body[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        found[name] = (pnames, cm.group(1), split_args(body[c_open + 1 : j]))
    return found


def canon(args: list[str], pnames: list[str], valist: str) -> list[str]:
    """Rename the local va_list cursor to V; leave parameter names alone (they are
    the thing under test — a swapped `len`/`count` must stay visible)."""
    return [re.sub(r"\b" + re.escape(valist) + r"\b", "V", a).strip() for a in args]


def local_valist(path: Path, name: str) -> str:
    """Whichever of `valist` / `ap` this body declared — a pure spelling choice."""
    return _body_facts(path, name)[0]


def va_start_anchor(path: Path, name: str) -> str | None:
    """The LAST NAMED PARAMETER `va_start` is anchored on.

    Not cosmetic and not implied by the delegate call: `_snprintf` anchors on
    `format` and `_snprintf_l` on `locale`, and getting it wrong shifts every
    variadic argument by one slot while still compiling and linking.
    """
    return _body_facts(path, name)[1]


def _body_facts(path: Path, name: str) -> tuple[str, str | None]:
    text = strip_comments(path.read_text(errors="replace"))
    m = re.search(r"^int WINAPIV " + re.escape(name) + r"\s*\(", text, re.M)
    if not m:
        return "valist", None
    brace = text.index("{", m.end())
    body, _ = body_of(text, brace)
    dm = re.search(r"va_list\s+(\w+)\s*;", body)
    sm = re.search(r"va_start\s*\(\s*\w+\s*,\s*(\w+)\s*\)", body)
    return (dm.group(1) if dm else "valist"), (sm.group(1) if sm else None)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--lint", action="store_true", help="one finding per line, for the gate")
    ap.add_argument(
        "root",
        nargs="?",
        help="directory holding wcs.c and rust_varargs_shim.c (default: dll/win32/msvcrt). "
        "Exists so the scanner is drivable on a fixture pair — a lint nobody can point at "
        "a known-bad input is a lint nobody can prove is not blind.",
    )
    args = ap.parse_args()

    global STDIO
    pairs = PAIRS
    if args.root:
        root = Path(args.root)
        STDIO = root / "stdio.h"
        # A fixture supplies whichever pair it is exercising; skip the ones it omits
        # rather than reporting them missing.
        pairs = [(root / r.name, root / m.name) for r, m in PAIRS]
        pairs = [(r, m) for r, m in pairs if r.is_file() and m.is_file()]
        if not pairs:
            print(f"{root}: no (reference, shim) pair found")
            return 2

    findings: list[str] = []
    checked = 0
    for ref_c, shim_c in pairs:
        findings += check_pair(ref_c, shim_c)
        checked += len(shells(shim_c, "WINAPIV"))

    # Every finding is about a shim, so lead with its path: a lint line that does not
    # say WHERE is one the reader has to go hunting for, and the control harness locates
    # a finding by looking for the fixture path in the output.
    if args.lint:
        for f in findings:
            print(f)
    else:
        for f in findings:
            print(f"DRIFT {f}")
        print(f"{checked} shells checked across {len(pairs)} pair(s), {len(findings)} finding(s)")
    return 1 if findings else 0


def check_pair(WCS: Path, SHIM: Path) -> list[str]:
    """Compare one transcribed trampoline TU against the C it was copied from."""
    findings: list[str] = []

    for p in (WCS, SHIM):
        if not p.is_file():
            return [f"{p}: missing"]

    ref = shells(WCS, "WINAPIV")
    sub = shells(SHIM, "WINAPIV")

    # ── the rewrite table, each entry PROVEN from wcs.c's own one-line forwarder ──
    #
    # The shim cannot call wcs.c's file-static `vswprintf_p_l_opt`, so it calls the
    # exported wrappers instead. That is only sound while those wrappers remain the
    # pass-options-0 forwarders they are today, so read them and check.
    fwd = shells(WCS, "CDECL")
    rewrites: dict[tuple[str, ...], tuple[str, ...]] = {}
    for wrapper, want in (
        ("_vswprintf_p_l", ("buffer", "length", "format", "0", "locale", "args")),
        ("_vscwprintf_p_l", ("NULL", "INT_MAX", "format", "0", "locale", "args")),
    ):
        if wrapper not in fwd:
            # Not a finding on its own: if nothing in wcs.c routes through the
            # file-static, there is no substitution to justify. A call site that
            # DOES need it and finds no rule still fails, as a delegate mismatch
            # naming the shell — which is the locatable message anyway.
            continue
        pnames, callee, cargs = fwd[wrapper]
        if callee != "vswprintf_p_l_opt" or tuple(cargs) != want:
            findings.append(
                f"{wrapper}: {WCS.name} no longer forwards to vswprintf_p_l_opt{want} "
                f"(now {callee}({', '.join(cargs)})) — the shim's substitution is no "
                f"longer an identity"
            )
            continue
        # The wrapper forwards its OWN parameters into `want` and hardcodes the rest,
        # so the positions to keep are wherever its parameters land — in its parameter
        # order — and the remaining positions are the fixed values that identify which
        # wrapper a given call site corresponds to.
        try:
            keep = tuple(want.index(p) for p in pnames)
        except ValueError:
            findings.append(f"{wrapper}: forwards something other than its own parameters")
            continue
        fixed = {i: want[i] for i in range(len(want)) if i not in keep}
        rewrites[("vswprintf_p_l_opt", wrapper)] = (keep, fixed)

    # `vsnprintf` is stdio.h's macro for `_vsnprintf`; the shim spells the target out.
    # Only wcs.c relies on that identity — scanf.c never writes the bare name.
    needs_vsnprintf = any(c == "vsnprintf" for _, c, _ in ref.values())
    if (
        needs_vsnprintf
        and STDIO.is_file()
        and "#define vsnprintf _vsnprintf" not in STDIO.read_text(errors="replace")
    ):
        findings.append(
            "sdk/include/crt/stdio.h no longer defines vsnprintf -> _vsnprintf; "
            "the shim calls _vsnprintf directly where wcs.c writes vsnprintf"
        )

    for name, (pnames, callee, cargs) in sorted(sub.items()):
        if name not in ref:
            findings.append(f"{name}: defined in the shim but not in {WCS.name}")
            continue
        rnames, rcallee, rargs = ref[name]
        if pnames != rnames:
            findings.append(
                f"{name}: parameter names differ — shim({', '.join(pnames)}) vs "
                f"{WCS.name}({', '.join(rnames)})"
            )
            continue
        rc, ra = rcallee, list(rargs)
        # apply the proven rewrites to the REFERENCE so both sides speak the shim's
        # vocabulary
        if rc == "vsnprintf":
            rc = "_vsnprintf"
        # Most-specific first: `_vscwprintf_p_l` pins NULL/INT_MAX where `_vswprintf_p_l`
        # only pins options=0, so the counting call site matches BOTH and the tighter
        # wrapper is the one the shim is entitled to name. (Both are the same call, so
        # this picks the canonical spelling, not the correct semantics.)
        ordered = sorted(rewrites.items(), key=lambda kv: -len(kv[1][1]))
        for (src, dst), (keep, fixed) in ordered:
            # A call site matches a wrapper iff every position the wrapper hardcodes
            # carries that exact value; the rest are what the wrapper would forward.
            if (
                rc == src
                and len(ra) == len(keep) + len(fixed)
                and all(ra[i] == v for i, v in fixed.items())
            ):
                rc, ra = dst, [ra[i] for i in keep]
                break
        v_sub = local_valist(SHIM, name)
        v_ref = local_valist(WCS, name)
        a_sub = canon(cargs, pnames, v_sub)
        a_ref = canon(ra, rnames, v_ref)
        anchor_sub, anchor_ref = va_start_anchor(SHIM, name), va_start_anchor(WCS, name)
        if anchor_sub != anchor_ref:
            findings.append(
                f"{name}: va_start anchored on {anchor_sub}, {WCS.name} anchors on {anchor_ref} "
                f"— every variadic argument shifts by one slot"
            )
        if rc != callee:
            findings.append(f"{name}: delegates to {callee}, {WCS.name} delegates to {rc}")
        elif a_sub != a_ref:
            findings.append(
                f"{name}: argument drift — shim {callee}({', '.join(a_sub)}) vs "
                f"{WCS.name} {rc}({', '.join(a_ref)})"
            )

    # The shim must not silently LOSE a shell either: every wcs.c WINAPIV shell that
    # this vintage compiles has to be present, or its export falls back to wcs.o's
    # copy and quietly keeps running the C engine.
    gated = gated_shells()
    for name in sorted(ref):
        if name not in sub:
            findings.append(
                f"{name}: in {WCS.name} but MISSING from the shim — "
                f"its export stays on the C engine"
            )

    # Both files must gate the SAME shells. A shell gated on one side only either adds
    # a symbol the C build never had, or drops one the DLL still exports.
    gated_sub = gated_in(strip_comments(SHIM.read_text(errors="replace")))
    shim_defines = set(
        re.findall(r"^int WINAPIV (\w+)\s*\(", SHIM.read_text(errors="replace"), re.M)
    )
    for name in sorted(gated & shim_defines):
        if name not in gated_sub:
            findings.append(
                f"{name}: _MSVCR_VER-gated in {WCS.name} but UNGATED in the shim — "
                f"adds a symbol this vintage does not have"
            )
    for name in sorted(gated_sub - gated):
        findings.append(f"{name}: gated in the shim but not in {WCS.name} — export lost")

    # Lead every finding with the shim's path — main() prints them unchanged, and the
    # control harness locates a finding by looking for the fixture path in the output.
    return [f"{SHIM}: {f}" for f in findings]


def gated_in(text: str) -> set[str]:
    """Shells behind an `_MSVCR_VER` gate this build does not compile.

    msvcrt builds at `_MSVCR_VER=0`, so a shell inside `#if _MSVCR_VER>=80` does not
    exist here and must NOT be in the shim — defining it would add a symbol the C
    build never had, and it is in no .spec. (`_sprintf_p` and `_scwprintf_p` were
    exactly that, caught before landing.)
    """
    out, stack = set(), []
    for line in text.splitlines():
        s = line.strip()
        if re.match(r"#\s*if", s):
            stack.append("_MSVCR_VER" in s and not re.search(r"_MSVCR_VER\s*==\s*0", s))
        elif s.startswith("#else") and stack:
            stack[-1] = not stack[-1]
        elif s.startswith("#endif") and stack:
            stack.pop()
        m = re.match(r"int (?:WINAPIV|CDECL) (\w+)\s*\(", s)
        if m and any(stack):
            out.add(m.group(1))
    return out


def gated_shells() -> set[str]:
    return gated_in(strip_comments(WCS.read_text(errors="replace")))


if __name__ == "__main__":
    sys.exit(main())
