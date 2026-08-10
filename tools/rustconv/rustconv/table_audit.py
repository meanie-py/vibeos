"""Verify a translated file's file-scope DATA TABLES against the C, element by element.

The pipeline's function bodies get read by a human and exercised by a boot; a
lookup table does neither. It compiles, it links, it boots — and then formats a
date wrong forever. A single mistranscribed byte in

    static const BYTE fmtGeneralDate[0x0a] = { FMT_GENERALDATE, 0x2, ... };

is invisible to every other check we run. `c2rust` gives us compiler-exact data
for files it can transpile (`c2rust_anchor.build_preamble`), but it panics on
some ASTs (oleaut32's varformat.c, c2rust 0.22.1 `convert_type.rs:518`), and
those files fall back to a pure-LLM preamble with no oracle at all. This module
is that oracle: parse both sides, compare the elements, and report.

Deliberately conservative — it only compares tables it can parse on BOTH sides,
and only elements that are literals on both sides. A symbolic element (an enum
name, a macro) is reported as `unchecked`, never as a pass. The counts it prints
are therefore a floor on what was verified, which is the honest direction.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

from .scanner import _DEFAULT_PP_DEFINES, scan_file

_COND_RE = re.compile(r"^\s*#\s*(ifdef|ifndef|if|elif|else|endif)\b(.*)$")
_DEFINED_RE = re.compile(r"\bdefined\s*\(?\s*([A-Za-z_]\w*)\s*\)?")


def _blank_dead_branches(text: str) -> str:
    """Blank out preprocessor-dead lines, PRESERVING line structure.

    Without this the audit compares against code the compiler never sees. ole32's
    compobj.c defines `classes_rootW` twice — `#ifdef __REACTOS__` picks the
    uppercase L"..." literal, and the `#else` is Wine's mixed-case char array. The
    Rust (correctly) translated the live branch; a preprocessor-blind audit read
    the dead one and reported 7 bogus "wrong character" diffs.

    Undecidable conditions keep BOTH branches — never blank code we can't prove dead.
    """
    out: list[str] = []
    stack: list[bool | None] = []  # per-nesting: is THIS branch live? None = unknown

    def live() -> bool:
        return all(s is not False for s in stack)

    for line in text.splitlines():
        m = _COND_RE.match(line)
        if not m:
            out.append(line if live() else re.sub(r"[^\n]", " ", line))
            continue
        kw, expr = m.group(1), m.group(2).strip()
        if kw in ("ifdef", "ifndef"):
            name = expr.split()[0] if expr else ""
            known = name in _DEFAULT_PP_DEFINES
            stack.append(known if kw == "ifdef" else not known)
        elif kw == "if":
            names = _DEFINED_RE.findall(expr)
            if names and all(n in _DEFAULT_PP_DEFINES for n in names) and "&&" not in expr:
                stack.append(True)
            elif names and not any(n in _DEFAULT_PP_DEFINES for n in names):
                stack.append(False)
            else:
                stack.append(None)  # can't decide — keep the branch
        elif kw == "elif":
            if stack:
                stack[-1] = None  # too fiddly to track precisely — keep it
        elif kw == "else":
            if stack:
                cur = stack[-1]
                stack[-1] = None if cur is None else (not cur)
        elif kw == "endif":
            if stack:
                stack.pop()
        out.append(re.sub(r"[^\n]", " ", line))  # the directive line itself
    return "\n".join(out)


# `static const BYTE fmtGeneralDate[0x0a] = {` — the size is optional, and the
# type may be multi-token (`unsigned char`, `const WCHAR`).
_C_TABLE_RE = re.compile(
    r"^\s*static\s+(?:const\s+)?"
    r"(?:[A-Za-z_]\w*\s+)+?"  # type tokens (non-greedy — the last ident is the NAME)
    r"([A-Za-z_]\w*)\s*"  # name
    r"((?:\[[^\]]*\])+)"  # one or more [dims]
    r"\s*=\s*\{",
    re.M,
)

# `pub static FMT_GENERAL_DATE: [u8; 10] = <rhs>` — `const`/`let` too, because the
# translator may lower a file-scope C table to a function-local binding (wine's
# VarMul does exactly this). Matching only `static` would report those MISSING, and
# a false "missing" trains you to ignore the whole report.
# The type may not exclude `;` — a Rust array type CONTAINS one (`[u8; 8]`), and
# excluding it silently unmatches every array static, i.e. every table we care about.
_RUST_TABLE_RE = re.compile(
    r"^\s*(?:pub\s+)?(?:static|const|let)\s+(?:mut\s+)?([A-Za-z_]\w*)\s*"
    r"(?::\s*[^=\n]*)?=\s*",  # the type annotation is optional (`let x = wide!(..)`)
    re.M,
)

# `wide!("OleMainThreadWndClass")` — our UTF-16 literal macro. It expands to
# `[u16; len+1]`, NUL-terminated, i.e. exactly the C `{'a','b','\0'}` shape, so it
# compares element-for-element once expanded.
_WIDE_MACRO_RE = re.compile(r'^\s*(?:wide|w)!\s*\(\s*"((?:[^"\\]|\\.)*)"\s*\)')


class TableParseError(Exception):
    """A table's initializer could not be balanced — never guess, say so."""


@dataclass
class TableDiff:
    name: str
    kind: str  # "missing" | "count" | "value" | "ok"
    detail: str


def _skip_literal(text: str, i: int) -> int | None:
    """Index just past the literal starting at ``text[i]``, or None if none starts there.

    THE BUG THIS EXISTS FOR. Both scanners below used to track only CHAR literals, toggling on
    every ``'``. sysdm's SMBIOS vendor table contains ``L"INT'L"`` — an apostrophe inside a
    STRING — so the toggle flipped mid-table, every ``[``/``]`` after it stopped counting, and
    the whole initializer came back "unbalanced". On the Rust side that is a hard
    ``TableParseError`` that aborts assembly of an already-paid file; on the C side it is worse,
    because a mis-split element list is compared rather than refused.

    A string is scanned to its closing quote (``b"..."``/``L"..."`` need nothing special — the
    prefix is an identifier character before the quote). A ``'`` opens a char literal ONLY when
    a closing ``'`` follows within one escape-or-character; otherwise it is data, or a Rust
    lifetime (``&'a str``), and must be treated as an ordinary character.
    """
    c = text[i]
    if c == '"':
        j = i + 1
        while j < len(text):
            if text[j] == "\\":
                j += 2
                continue
            if text[j] == '"':
                return j + 1
            j += 1
        return None  # unterminated — let the caller's balance check report it
    if c == "'":
        j = i + 1
        if j < len(text) and text[j] == "\\":
            j += 2
        elif j < len(text):
            j += 1
        if j < len(text) and text[j] == "'":
            return j + 1
        return None  # a lifetime, or an apostrophe in prose/data
    return None


def _match_balanced(text: str, open_idx: int, opc: str, clc: str) -> int:
    """Index just past the closer matching text[open_idx]; -1 if unbalanced."""
    depth = 0
    i = open_idx
    while i < len(text):
        after = _skip_literal(text, i)
        if after is not None:
            i = after
            continue
        c = text[i]
        if c == opc:
            depth += 1
        elif c == clc:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return -1


def _strip_comments(s: str) -> str:
    """Blank out comments while PRESERVING line structure.

    Collapsing a multi-line /* */ to one space shifts every line after it, which
    silently desynchronises the offsets from the scanner's function line-spans —
    the "is this table function-local?" test then reads the wrong lines.
    """

    def _blank(m: re.Match[str]) -> str:
        return re.sub(r"[^\n]", " ", m.group(0))  # keep the newlines, drop the text

    s = re.sub(r"/\*.*?\*/", _blank, s, flags=re.S)
    return re.sub(r"//[^\n]*", _blank, s)


def _split_elements(body: str) -> list[str]:
    """Split a brace/bracket initializer body on TOP-LEVEL commas.

    Nested initializers (2-D tables) are flattened — we compare the leaf
    sequence, which is what actually lands in .rodata.

    Literals are consumed WHOLE (see ``_skip_literal``), so a bracket or comma inside a string
    is data and not structure — ``L"INT'L"`` used to flip a char-literal toggle and desynchronise
    every element after it, and a string containing a comma would have split one element into
    two long before that.
    """
    out: list[str] = []
    depth = 0
    cur: list[str] = []
    i = 0
    while i < len(body):
        after = _skip_literal(body, i)
        if after is not None:
            cur.append(body[i:after])
            i = after
            continue
        c = body[i]
        if c in "{[":
            depth += 1  # flatten: drop the nesting punctuation itself
        elif c in "}]":
            depth -= 1
        elif c == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
        else:
            cur.append(c)
        i += 1
    tail = "".join(cur).strip()
    if tail:
        out.append(tail)
    return [e for e in out if e]


_CHAR_ESCAPES = {
    "\\0": 0,
    "\\n": 10,
    "\\r": 13,
    "\\t": 9,
    "\\\\": 92,
    "\\'": 39,
    '\\"': 34,
}


def normalize_element(e: str) -> int | None:
    """A literal's integer value, or None if it is symbolic (→ 'unchecked')."""
    e = e.strip()
    # A RUST width suffix, before the C one. `rstrip("uUlL")` cannot take `u16` — it ends in a
    # digit — so `0x0043u16` fell through to `int(e, 0)`, failed, and reported as SYMBOLIC: an
    # element silently exempted from the very check this function exists to perform. Third
    # sighting of this class (crate_repair's `1usize`, scan_wide_literals' `u16`), and the one
    # that matters most now that `--seal-wide` types its restored arrays by suffixing the first
    # element. The lookbehind keeps `0x16` and plain `16` intact.
    e = re.sub(r"(?<=[0-9a-fA-F])(?:[ui](?:8|16|32|64|128|size))$", "", e)
    e = e.rstrip("uUlL")
    if not e:
        return None
    # Char literal, C or Rust: 'x' 'b'x'' '\0' '\n' '\005' (C octal) '\x41' (Rust hex).
    # The octal form is not a curiosity — stg_prop's property-set names are
    # `{'\005','S','u','m',…}`, and without it most of the table reads as symbolic
    # and silently goes unverified.
    m = re.fullmatch(r"[bB]?'(\\.|\\[0-7]{1,3}|\\x[0-9a-fA-F]{1,2}|[^'\\])'", e)
    if m:
        body = m.group(1)
        if body in _CHAR_ESCAPES:
            return _CHAR_ESCAPES[body]
        if re.fullmatch(r"\\x[0-9a-fA-F]{1,2}", body):
            return int(body[2:], 16)
        if re.fullmatch(r"\\[0-7]{1,3}", body):
            return int(body[1:], 8)
        if len(body) == 1:
            return ord(body)
        return None
    try:
        return int(e, 0)  # 0x.., 0b.., 0o.., decimal
    except ValueError:
        return None


def parse_c_tables(c_path: Path) -> dict[str, list[str]]:
    """name -> raw element list, for every FILE-SCOPE `static T name[] = {..}`.

    File-scope only, and deliberately so. Function-local statics are not in scope
    here: they sit inside the body the translator is handed, so they get read in
    context — and worse, they collide. wine's variant.c defines a *different*
    `prio2vt` inside VarAdd (14 entries) and VarMul (10); keeping the last writer
    silently compared VarMul's C table against VarAdd's Rust one and reported a
    bug that did not exist. A collision here is a tool defect, so it raises.
    """
    text = _blank_dead_branches(_strip_comments(c_path.read_text(errors="replace")))
    scan = scan_file(c_path)
    in_fn: set[int] = set()
    for f in scan.functions:
        in_fn.update(range(f.line_start, f.line_end + 1))

    out: dict[str, list[str]] = {}
    for m in _C_TABLE_RE.finditer(text):
        if text.count("\n", 0, m.start()) + 1 in in_fn:
            continue  # function-local — not our business
        name = m.group(1)
        brace = text.index("{", m.end() - 1)
        end = _match_balanced(text, brace, "{", "}")
        if end < 0:
            raise TableParseError(f"{c_path.name}: unbalanced initializer for `{name}`")
        if name in out:
            raise TableParseError(
                f"{c_path.name}: two file-scope tables named `{name}` — refusing to guess "
                "which one the Rust corresponds to"
            )
        elems = _split_elements(text[brace + 1 : end - 1])
        out[name] = _zero_fill(elems, m.group(2))
    return out


def _zero_fill(elems: list[str], dims: str) -> list[str]:
    """Expand a partially-initialised C table to its declared length.

    C zero-fills the tail: ole32's `guid_conv_table[256]` spells only 103
    elements and lets the compiler supply 153 zeros. Rust has no such rule — the
    translation must spell all 256 — so comparing the C's *written* count against
    the Rust's would report a phantom mismatch, and (worse) a translation that
    really did emit `[u8; 103]` would look correct. Model the zero-fill so both
    mistakes surface.

    Non-literal dims (`[]`, `[N_ELEMS]`) are left alone — we compare what's written.
    """
    sizes = re.findall(r"\[([^\]]*)\]", dims)
    total = 1
    for s in sizes:
        try:
            total *= int(s.strip(), 0)
        except ValueError:
            return elems  # symbolic / empty dim — no reliable declared length
    if total > len(elems):
        return elems + ["0"] * (total - len(elems))
    return elems


_RUST_ESCAPES = {"\\n": 10, "\\r": 13, "\\t": 9, "\\0": 0, "\\\\": 92, '\\"': 34}


def _expand_wide(s: str) -> list[str]:
    """`wide!("ab")` -> ['97','98','0'] — chars plus the NUL the macro appends."""
    out: list[str] = []
    i = 0
    while i < len(s):
        if s[i] == "\\" and i + 1 < len(s):
            esc = s[i : i + 2]
            out.append(str(_RUST_ESCAPES.get(esc, ord(s[i + 1]))))
            i += 2
        else:
            out.append(str(ord(s[i])))
            i += 1
    out.append("0")  # wide! is NUL-terminated ([u16; len+1])
    return out


def parse_rust_tables(rust: str) -> tuple[dict[str, list[list[str]]], set[str]]:
    """(name -> EVERY definition's element list, names present but not comparable).

    A name maps to a LIST of candidates, not one. Rust re-uses a name across
    scopes exactly as C does: varformat has a file-scope `szPercent` ("Percent",
    8) and a function-local `szPercent` ("%", 2), and both translations are
    correct. Keeping only the last definition compared the C's file-scope table
    against the Rust's function-local one and reported a bug in correct code —
    the same last-writer-wins mistake this module already fixes on the C side.
    A C table is satisfied if ANY candidate matches it.

    The `unparsed` set is the honest half: `guid_conv_table: [u8; 256] = {
    ...const block... }` IS translated — we just cannot evaluate it without
    running rustc. Calling that "missing" would be exactly the false alarm that
    gets a whole report ignored, so it gets its own class.
    """
    text = _strip_comments(rust)
    out: dict[str, list[list[str]]] = {}
    unparsed: set[str] = set()
    for m in _RUST_TABLE_RE.finditer(text):
        name = m.group(1)
        rhs = text[m.end() :]
        wm = _WIDE_MACRO_RE.match(rhs)
        if wm:
            out.setdefault(name, []).append(_expand_wide(wm.group(1)))
            continue
        if not rhs.lstrip().startswith("["):
            unparsed.add(name)  # const block, fn call, `&[..]` behind a cast, …
            continue
        brk = text.index("[", m.end())
        end = _match_balanced(text, brk, "[", "]")
        if end < 0:
            raise TableParseError(f"unbalanced initializer for Rust `{name}`")
        out.setdefault(name, []).append(_split_elements(text[brk + 1 : end - 1]))
    return out, unparsed


def audit_tables(c_path: Path, rust: str) -> list[TableDiff]:
    """Compare every C data table against its Rust twin. Report every divergence.

    Name match is exact (rustconv preserves the C name), then case-insensitive
    (the LLM likes to SCREAMING_CASE a `static const`).
    """
    c_tables = parse_c_tables(c_path)
    r_tables, r_unparsed = parse_rust_tables(rust)
    r_by_lower = {k.lower(): k for k in r_tables}
    unparsed_by_lower = {k.lower(): k for k in r_unparsed}

    diffs: list[TableDiff] = []
    for name, c_elems in sorted(c_tables.items()):
        rname = name if name in r_tables else r_by_lower.get(name.lower())
        if rname is None:
            if name in r_unparsed or name.lower() in unparsed_by_lower:
                diffs.append(
                    TableDiff(
                        name,
                        "unparsed",
                        f"present in the Rust but not a literal array ({len(c_elems)} C "
                        "elements) — NOT verified; read it by hand",
                    )
                )
            else:
                diffs.append(
                    TableDiff(name, "missing", f"{len(c_elems)} C elements, absent from the Rust")
                )
            continue
        candidates = r_tables[rname]

        # Satisfied if ANY definition of that name matches — Rust re-uses names
        # across scopes just as C does (see parse_rust_tables). Otherwise report
        # against the CLOSEST candidate by length, so the message points at the
        # table the reader most likely means.
        best: TableDiff | None = None
        best_delta = 1 << 30
        for r_elems in candidates:
            d = _compare(name, rname, c_elems, r_elems)
            if d.kind == "ok":
                best, best_delta = d, -1
                break
            delta = abs(len(r_elems) - len(c_elems))
            if delta < best_delta:
                best, best_delta = d, delta
        assert best is not None
        diffs.append(best)
    return diffs


def _compare(name: str, rname: str, c_elems: list[str], r_elems: list[str]) -> TableDiff:
    if len(c_elems) != len(r_elems):
        return TableDiff(
            name,
            "count",
            f"C has {len(c_elems)} elements, Rust `{rname}` has {len(r_elems)}",
        )

    bad: list[str] = []
    unchecked = 0
    # strict=True is safe: the length check above already returned on a mismatch.
    for i, (ce, re_) in enumerate(zip(c_elems, r_elems, strict=True)):
        cv, rv = normalize_element(ce), normalize_element(re_)
        if cv is None or rv is None:
            unchecked += 1
            continue
        if cv != rv:
            bad.append(f"[{i}] C={ce!r}({cv}) vs Rust={re_!r}({rv})")
    if bad:
        return TableDiff(
            name,
            "value",
            f"{len(bad)} element(s) differ: "
            + "; ".join(bad[:6])
            + (f" … +{len(bad) - 6} more" if len(bad) > 6 else ""),
        )
    checked = len(c_elems) - unchecked
    return TableDiff(
        name,
        "ok",
        f"{checked}/{len(c_elems)} elements verified"
        + (f", {unchecked} symbolic (unchecked)" if unchecked else ""),
    )


def format_report(diffs: list[TableDiff]) -> str:
    bad = [d for d in diffs if d.kind in ("missing", "count", "value")]
    unparsed = [d for d in diffs if d.kind == "unparsed"]
    ok = [d for d in diffs if d.kind == "ok"]
    lines = []
    for d in bad:
        lines.append(f"  ✗ {d.kind.upper():8s} {d.name}: {d.detail}")
    for d in unparsed:
        lines.append(f"  ? UNPARSED {d.name}: {d.detail}")
    for d in ok:
        lines.append(f"  ✓ {d.name}: {d.detail}")
    lines.append(
        f"\n{len(ok)} table(s) verified, {len(bad)} divergence(s), "
        f"{len(unparsed)} not comparable (verify by hand)."
    )
    return "\n".join(lines)
