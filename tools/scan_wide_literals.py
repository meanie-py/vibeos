#!/usr/bin/env python3
"""Decode every hand-encoded UTF-16 string literal and check it against its intent comment.

WHY THIS EXISTS
---------------
Translating C to Rust, a `L"..."` wide-string constant routinely becomes a hand-encoded array:

    // L"System\\CurrentControlSet\\Services"
    static SERVICES_KEY: [u16; 34] = [b'S' as u16, b'y' as u16, ..., 0];

and the array is sized BY HAND. Get the count wrong and rustc is perfectly happy — the array is
whatever length you declared, and the compiler has no idea what you meant. `SERVICES_KEY` shipped
as `[u16; 33]`, which silently dropped the final `s`: it spelled

    System\\CurrentControlSet\\Service          (no trailing 's')

so services-rust opened a nonexistent key and every service registration failed with
ERROR_FILE_NOT_FOUND. The intent comment directly above it said the RIGHT string, which is exactly
why review eyes slide off it — you read the comment, nod, and never decode the 34 numbers below.
It took a kmtest conformance sweep to find (field manual §9.6-services / SERVICES_KEY).

Nothing we already run can see this class:
  * rustc — the array is well-formed at any length; there is no bug to diagnose.
  * The linker/boot — a wrong-but-valid registry path fails at RUNTIME, as a plain "not found"
    from a subsystem that has a hundred other reasons to say that.
  * verify_constants / scan_magic_offsets — those compare NUMBERS against C headers. This is a
    string whose only specification is a comment.

So the oracle is the comment. That is unusual and worth being honest about: this lint trusts prose.
That is sound *here* only because the comment and the array are two independent encodings of the
same intent, written at different times — they disagree exactly when one of them is wrong, which
is the entire signal. When they disagree the lint does not guess which is right; it prints both.

USAGE
    scan_wide_literals.py [paths...]       # default: sdk/lib/rust
    scan_wide_literals.py --check          # exit 1 on any finding
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# `static NAME: [u16; N] = [ ... ];`  (also `pub static`, `static mut`, `const`)
_DECL = re.compile(
    r"^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?(?:static|const)[ \t]+(?:mut[ \t]+)?"
    r"([A-Za-z_]\w*)[ \t]*:[ \t]*\[[ \t]*u16[ \t]*;[ \t]*(\d+)[ \t]*\][ \t]*=[ \t]*\[",
    re.M,
)

# One array element. Accepts the four spellings that actually occur in the tree:
#   b'X' as u16 | 'X' as u16 | b'\\' as u16 | 0 | 0x5C
_ELEM = re.compile(
    r"""b?'(\\.|[^'])'[ \t]*as[ \t]*u16     # b'X' as u16 / 'X' as u16
      | 0[xX]([0-9a-fA-F]+)                 # 0x5C
      | (\d+)                               # 0, 65
    """,
    re.X,
)

_ESCAPES = {
    "\\\\": "\\",
    "\\'": "'",
    '\\"': '"',
    "\\n": "\n",
    "\\r": "\r",
    "\\t": "\t",
    "\\0": "\0",
}

# The intent: `// L"..."`. A comment may list ALTERNATIVES (`// L"DependOnService" /
# L"DependOnGroup"`), so collect them all and accept a match against ANY — the comment is
# documenting a family, and demanding the first would report correct code.
_INTENT = re.compile(r'L"((?:[^"\\]|\\.)*)"')


def _decode_elem(m: re.Match) -> int | None:
    if m.group(1) is not None:
        ch = m.group(1)
        if ch.startswith("\\"):
            if ch not in _ESCAPES:
                return None  # an escape we do not model — refuse to guess
            return ord(_ESCAPES[ch])
        return ord(ch)
    if m.group(2) is not None:
        return int(m.group(2), 16)
    return int(m.group(3))


def _unescape_intent(s: str) -> str:
    out, i = [], 0
    while i < len(s):
        if s[i] == "\\" and i + 1 < len(s):
            two = s[i : i + 2]
            if two in _ESCAPES:
                out.append(_ESCAPES[two])
                i += 2
                continue
            return ""  # unmodelled escape -> treat as no usable intent
        out.append(s[i])
        i += 1
    return "".join(out)


def _array_body(text: str, open_bracket: int) -> tuple[str, int] | None:
    """Text between the `[` and its matching `]`, bracket-matched (elements never nest, but a
    naive `]` search would still trip over a trailing comment containing one)."""
    depth, i = 0, open_bracket
    while i < len(text):
        if text[i] == "[":
            depth += 1
        elif text[i] == "]":
            depth -= 1
            if depth == 0:
                return text[open_bracket + 1 : i], i
        i += 1
    return None


def _claims_nul(block: list[str]) -> bool:
    """Does the comment block explicitly promise a NUL terminator?

    A blanket "an L\"...\" literal must end in NUL" rule is WRONG here and the corpus says so
    loudly: `// L"\\\\?\\\\" — 4 wide chars, no null needed (Length == MaximumLength)` sits above a
    UNICODE_STRING Buffer, where the terminator is genuinely unnecessary because Length carries
    the size. Five such buffers exist in iomgr/fsrtl/npfs alone, all correct. Flagging them would
    be five false positives against one hypothetical bug — and a lint that cries wolf on correct
    code is how the real findings get ignored. So only fire when the author SAID they wanted one.
    """
    joined = " ".join(block).lower()
    return "nul-terminated" in joined or "null-terminated" in joined or "null terminated" in joined


def _comment_block(text: str, decl_start: int) -> list[str]:
    """The CONTIGUOUS `//` run directly above the declaration — nothing further up.

    Do NOT rstrip newlines off `head` first: that swallows the blank line that separates an
    unrelated comment from this literal, silently adopting it as the spec. `decl_start` sits at
    the start of the declaration's line, so the final split element is the empty string before it;
    drop exactly that one and let a second empty line terminate the walk.
    """
    lines = text[:decl_start].split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    block = []
    for line in reversed(lines):
        s = line.strip()
        if s.startswith("//"):
            block.append(s)
            continue
        break
    return list(reversed(block))


def _intent_for(text: str, decl_start: int) -> list[str]:
    """The `L\"...\"` strings in the comment block immediately above the declaration.

    Only a CONTIGUOUS run of `//` lines counts — a stray L"..." from an unrelated comment further
    up is not this literal's specification.
    """
    out = []
    for line in _comment_block(text, decl_start):
        for m in _INTENT.finditer(line):
            s = m.group(1)
            # `...` is not a spec. Two real shapes produce it, and both would be pure noise:
            #   - prose ABOUT the syntax: "Rust doesn't have `L\"...\"`, so spell them out"
            #   - an ABBREVIATED path: L"\\Registry\\Machine\\...\\Version"
            # Neither states what the array should contain, so there is nothing to check.
            if "..." in s or not s:
                continue
            out.append(s)
    return out


# `b"...".as_ptr() as *const u16` — the OTHER way a wide literal is hand-encoded, and the one
# with no comment to check against. Accepts the pointer spellings that occur: u16 / wchar_t /
# WCHAR / PWSTR-ish, const or mut.
# Both spellings of the cast matter, and so do the ALIASES. Widened after a repo-wide sweep
# fixed 52 sites and the remainder turned out to be the same bug hiding behind `LPCWSTR`
# rather than a bare `u16`, or behind `transmute` rather than `as`. A lint that only sees the
# primitive spelling reports clean while the class is still there.
_WIDE_PTR = r"(?:u16|wchar_t|WCHAR|MSVCRT_wchar_t|[LP]{1,2}C?W(?:STR|CHAR))"
_BYTECAST = re.compile(
    rf"""b"((?:[^"\\]|\\.)*)"       # the byte-string literal
        [ \t\r\n]*(?:\.as_ptr\(\)|\.as_mut_ptr\(\))
        [ \t\r\n]*as[ \t\r\n]*
        (?:\*[ \t\r\n]*(?:const|mut)[ \t\r\n]*{_WIDE_PTR}   # as *const u16
          |{_WIDE_PTR})\b                                   # as LPCWSTR""",
    re.X,
)
# `transmute(b"...".as_ptr())` reaches the same wrong pointer with no `as` in sight. The
# target type is on the binding, not here, so this form is matched on shape alone; a byte
# string transmuted to anything is worth a look, and in practice every hit is a wide string.
_BYTETRANSMUTE = re.compile(
    r"""transmute(?:::<[^>]*>)?\([ \t\r\n]*b"((?:[^"\\]|\\.)*)"
        [ \t\r\n]*(?:\.as_ptr\(\)|\.as_mut_ptr\(\))""",
    re.X,
)

_BYTE_ESCAPES = {"0": 0, "n": 10, "r": 13, "t": 9, "\\": 92, '"': 34, "'": 39}


def _decode_byte_string(s: str) -> bytes | None:
    """The bytes a Rust `b"..."` literal actually contains. None if we cannot decode it."""
    out = bytearray()
    i = 0
    while i < len(s):
        if s[i] != "\\":
            out.append(ord(s[i]) & 0xFF)
            i += 1
            continue
        nxt = s[i + 1 : i + 2]
        if nxt in _BYTE_ESCAPES:
            out.append(_BYTE_ESCAPES[nxt])
            i += 2
        elif nxt == "x":
            try:
                out.append(int(s[i + 2 : i + 4], 16))
            except ValueError:
                return None
            i += 4
        else:
            return None  # \u{...} and friends — not a shape we decode, so not a verdict
    return bytes(out)


def scan_byte_casts(text: str, rel) -> list[str]:
    """Reject a byte-string literal that CANNOT be the UTF-16 string it is cast to.

    Rust's `b"..."` is 8-bit and is not NUL-terminated for you, so three distinct mistakes
    all produce a pointer that compiles, links, and reads out of bounds at runtime:

      * `b" /c \\0" as *const wchar_t` — plain ASCII reinterpreted as UTF-16. `wcscat` reads
        `' '`+`'/'` as ONE code unit 0x2F20 and walks off the end looking for a zero u16.
        (Shipped in msvcrt process.c's `_wsystem`.)
      * `b"P\\0A\\0T\\0H\\0\\0"` — correctly interleaved but ONE terminator byte. A u16 NUL is
        two bytes, so the final code unit's high half comes from the next object in rodata.
        (Shipped in msvcrt process.c's `msvcrt_search_executable`; broke PATH search.)
      * an odd total length — the literal cannot be whole u16 units at all.

    Unlike the array rule above, this one needs no intent comment: it is a self-contained
    claim about the literal's own bytes. All three checks are decidable, so there are no
    judgement calls and no prose to trust.
    """
    findings = []
    stripped = _strip_line_comments(text)
    for m in list(_BYTECAST.finditer(stripped)) + list(_BYTETRANSMUTE.finditer(stripped)):
        raw = _decode_byte_string(m.group(1))
        if raw is None:
            continue
        line = text[: m.start()].count("\n") + 1
        shown = m.group(0).split(".as_")[0]
        if len(raw) % 2:
            findings.append(
                f"{rel}:{line}: {shown} is {len(raw)} bytes — an ODD length cannot be "
                f"whole u16 code units"
            )
        elif not raw.endswith(b"\0\0"):
            findings.append(
                f"{rel}:{line}: {shown} has no u16 NUL terminator — a wide NUL is TWO zero "
                f"bytes and Rust does not add them"
            )
        elif len(raw) > 4 and all(b != 0 for b in raw[1:-2:2]):
            findings.append(
                f"{rel}:{line}: {shown} is 8-bit text cast to a u16 pointer — every second "
                f"byte must be 0 for ASCII UTF-16LE, none is"
            )
    return findings


def _strip_line_comments(text: str) -> str:
    """Blank out `//` comments, preserving offsets so line numbers stay right.

    Needed because several crates carry a comment explaining this very bug and quoting the
    bad form verbatim ("replacing incorrect b\"...\".as_ptr() as *mut u16 casts"). Matching
    inside those turns a fix into a finding.
    """
    out = list(text)
    i, n = 0, len(text)
    while i < n - 1:
        if text[i] == '"':  # skip string literals so a `//` inside one is not a comment
            i += 1
            while i < n and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
            i += 1
            continue
        if text[i] == "/" and text[i + 1] == "/":
            while i < n and text[i] != "\n":
                out[i] = " "
                i += 1
            continue
        i += 1
    return "".join(out)


def scan_file(path: Path) -> tuple[list[str], int, int]:
    """(findings, literals_seen, literals_compared)"""
    text = path.read_text(errors="replace")
    try:
        rel_path = path.relative_to(ROOT)
    except ValueError:
        rel_path = path
    findings, seen, compared = scan_byte_casts(text, rel_path), 0, 0
    for m in _DECL.finditer(text):
        name, declared = m.group(1), int(m.group(2))
        body = _array_body(text, m.end() - 1)
        if body is None:
            continue
        seen += 1
        vals = [_decode_elem(e) for e in _ELEM.finditer(body[0])]
        if any(v is None for v in vals):
            continue  # an element we cannot decode -> not a verdict we can make
        if len(vals) != declared:
            # rustc would reject this, so it means OUR parse is wrong, not the code.
            continue
        intents = _intent_for(text, m.start())
        if not intents:
            continue
        compared += 1

        decoded = "".join(chr(v) for v in vals)
        # Strip ALL trailing NULs, not one: hal usage.rs really ends `0x0000, 0x0000` (a harmless
        # double terminator — RtlInitUnicodeString stops at the first), and a one-NUL strip
        # reported that correct literal as a content mismatch.
        actual = decoded.rstrip("\0")
        nul_terminated = decoded.endswith("\0")

        # The comment may spell the terminator itself: `// L"RXACT\0" / L"Log\0"`. Strip it on
        # this side too, or every such literal reports as "TRUNCATED (dropped '\x00')".
        wants = [_unescape_intent(i).rstrip("\0") for i in intents]
        wants = [w for w in wants if w]
        if not wants:
            compared -= 1
            continue
        if actual in wants and (nul_terminated or not _claims_nul(_comment_block(text, m.start()))):
            continue
        # Report against the closest alternative — a comment may list a family
        # (`L"DependOnService" / L"DependOnGroup"`), and naming the nearest is the useful message.
        best = min(wants, key=lambda w: abs(len(w) - len(actual)))
        try:
            rel = path.relative_to(ROOT)
        except ValueError:
            rel = path  # a file outside the repo (a test fixture) — report it as given
        line = text[: m.start()].count("\n") + 1
        if best.startswith(actual) and len(actual) < len(best):
            dropped = best[len(actual) :]
            findings.append(
                f"{rel}:{line}: {name} [u16; {declared}] is TRUNCATED — decodes to "
                f'"{actual}" but the comment says "{best}" (dropped {dropped!r}; '
                f"need {len(best) + 1} elements)"
            )
        elif not nul_terminated:
            findings.append(
                f'{rel}:{line}: {name} [u16; {declared}] decodes to "{actual}" with NO NUL '
                f'terminator (comment: "{best}")'
            )
        else:
            findings.append(
                f'{rel}:{line}: {name} [u16; {declared}] decodes to "{actual}" but the '
                f'comment says "{best}"'
            )
    return findings, seen, compared


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("paths", nargs="*", default=[])
    ap.add_argument("--check", action="store_true", help="exit 1 on any finding")
    args = ap.parse_args()

    roots = [Path(p) for p in args.paths] or [ROOT / "sdk" / "lib" / "rust"]
    files = []
    for r in roots:
        files.extend([r] if r.is_file() else sorted(r.rglob("*.rs")))

    all_findings, seen, compared = [], 0, 0
    for f in files:
        if "/target/" in str(f):
            continue
        try:
            fnd, s, c = scan_file(f)
        except OSError:
            continue
        all_findings += fnd
        seen += s
        compared += c

    for f in all_findings:
        print(f)
    # ALWAYS report coverage. This lint can only check literals that carry an L"..." intent
    # comment, so "0 findings" over 0 comparisons is meaningless — and indistinguishable from a
    # clean sweep unless the ratio is printed. (verify_c_signatures shipped exactly that green.)
    print(
        f"scan_wide_literals: {len(all_findings)} finding(s); "
        f'{compared}/{seen} literal(s) had an L"..." intent comment to check against',
        file=sys.stderr,
    )
    return 1 if (all_findings and args.check) else 0


if __name__ == "__main__":
    raise SystemExit(main())
