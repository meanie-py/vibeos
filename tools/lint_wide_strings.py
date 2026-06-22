#!/usr/bin/env python3
"""Static linter for mis-encoded UTF-16 (wide) byte-string literals in Rust
translations.

rustconv hand-encodes UTF-16LE wide strings as Rust byte-string literals
(`b"i\\0m\\0m\\03\\02\\0.\\0d\\0l\\0l\\0\\0\\0"` == L"imm32.dll"). Two silent
mis-encodings have bitten boot (user32 Wave 4):

  1. C-OCTAL CONFUSION. The author writes a C octal escape (`\\000`, `\\0NN`)
     but Rust has NO octal escapes: `\\0` is NUL and the following digits are
     LITERAL. So `b"...m\\0003\\0002..."` is `m \\0 '0' '0' '3' ...` — garbage,
     not L"...m32...". (A single `\\0<digit>`, e.g. `\\03`, is fine: it's the
     NUL high-byte of one char followed by the next char's low byte. Only
     `\\0` + TWO-OR-MORE digits is the bug.)

  2. MISSING UTF-16 NUL TERMINATOR. A wide literal must end in the two bytes
     `00 00` (a u16 NUL) and have even length. `b"\\\\\\0"` (L"\\\\") is `[5C 00]`
     — the '\\' char with NO terminator; read as a PCWSTR it walks off the end
     of the literal into adjacent .rodata. `b"\\0"` (intended empty L"") is one
     byte `[00]` — odd length, half a code unit. Correct forms: `b"\\\\\\0\\0\\0"`
     and `b"\\0\\0"`.

Both are invisible to the compiler (the bytes are well-formed Rust) and only
surface as a wrong runtime string — a failed LoadLibraryW, a bogus registry
path, etc. This linter decodes every byte-string literal and flags wide ones
that are mis-encoded, BEFORE the boot regression.

Usage:
    python3 tools/lint_wide_strings.py                       # scan all rust crates
    python3 tools/lint_wide_strings.py path/to/file.rs ...   # specific files
    python3 tools/lint_wide_strings.py --check               # exit 1 if any found (CI)
"""

import re
import sys
from pathlib import Path

# Pointer types that mean "this byte literal is consumed as a UTF-16 wide string".
WIDE_CAST_TYPES = (
    "PCWSTR",
    "LPCWSTR",
    "PWSTR",
    "LPWSTR",
    "LPWCH",
    "PWCH",
    "PWCHAR",
    "LPCWCH",
    "*const u16",
    "*mut u16",
)
# A byte-string literal: b"..." with Rust escape handling (\" and \\ respected).
# Capture a leading '*' (array deref, e.g. *b"Tag4" — NOT a string pointer) and any
# immediately-following `.as_ptr() as <wide-type>` cast that BINDS to this literal.
_BYTESTR_RE = re.compile(
    r'(?P<deref>\*?)b"(?P<body>(?:\\.|[^"\\])*)"'
    r"(?P<cast>\s*(?:\.as_ptr\(\)\s*)?as\s+(?:"
    + "|".join(re.escape(t) for t in WIDE_CAST_TYPES)
    + r"))?"
)

# Source-text signature of the C-octal-confusion bug: \0 followed by 2+ digits.
_OCTAL_RE = re.compile(r"\\0[0-9][0-9]")

# Simple Rust byte-string escapes -> byte value.
_SIMPLE_ESC = {
    "n": 0x0A,
    "r": 0x0D,
    "t": 0x09,
    "\\": 0x5C,
    "0": 0x00,
    "'": 0x27,
    '"': 0x22,
}


def decode_bytestr(body: str):
    """Decode the inside of a b"..." literal to raw bytes (Rust escape rules).

    Returns (bytes, ok). ok=False if an escape couldn't be decoded (we then
    skip validation rather than guess)."""
    out = bytearray()
    i = 0
    n = len(body)
    while i < n:
        c = body[i]
        if c != "\\":
            # Non-ASCII source chars would be multi-byte UTF-8; byte literals
            # only allow ASCII + escapes, so treat as a single byte best-effort.
            out.append(ord(c) & 0xFF)
            i += 1
            continue
        if i + 1 >= n:
            return bytes(out), False
        e = body[i + 1]
        if e == "x":
            if i + 3 >= n:
                return bytes(out), False
            try:
                out.append(int(body[i + 2 : i + 4], 16))
            except ValueError:
                return bytes(out), False
            i += 4
            continue
        if e == "\n":  # line-continuation escape
            i += 2
            continue
        if e in _SIMPLE_ESC:
            out.append(_SIMPLE_ESC[e])
            i += 2
            continue
        return bytes(out), False  # unknown escape — don't guess
    return bytes(out), True


def strip_comments(line: str, in_block: bool):
    """Blank out `//` line-comments and `/* */` block-comments (respecting string
    literals) so byte-string examples inside comments aren't scanned. Returns
    (code_only_line, in_block_after). Char literals aren't tracked (rare edge)."""
    out = []
    i, n = 0, len(line)
    in_str = False
    while i < n:
        c = line[i]
        if in_block:
            if c == "*" and i + 1 < n and line[i + 1] == "/":
                in_block = False
                out.append("  ")
                i += 2
                continue
            out.append(" ")
            i += 1
            continue
        if in_str:
            out.append(c)
            if c == "\\" and i + 1 < n:
                out.append(line[i + 1])
                i += 2
                continue
            if c == '"':
                in_str = False
            i += 1
            continue
        if c == '"':
            in_str = True
            out.append(c)
            i += 1
            continue
        if c == "/" and i + 1 < n and line[i + 1] == "/":
            break  # rest of line is a comment
        if c == "/" and i + 1 < n and line[i + 1] == "*":
            in_block = True
            out.append("  ")
            i += 2
            continue
        out.append(c)
        i += 1
    return "".join(out), in_block


def check_file(path: Path):
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return []
    findings = []
    in_block = False
    for lineno, raw_line in enumerate(text.splitlines(), 1):
        line, in_block = strip_comments(raw_line, in_block)
        for m in _BYTESTR_RE.finditer(line):
            body = m.group("body")
            # A leading '*' makes this a byte-array deref (e.g. u32::from_be_bytes(*b"Tag4"))
            # — a tag/blob, never a wide string pointer. Skip.
            if m.group("deref") == "*":
                continue
            # Only literals DIRECTLY cast to a wide pointer type are wide-intended.
            # This binds the cast to *this* literal, avoiding ANSI strings that merely
            # sit near an unrelated `as PWSTR`/`as PWCHAR` on the same line.
            if m.group("cast") is None:
                continue
            raw, ok = decode_bytestr(body)
            if not ok:
                continue
            lit = 'b"' + body + '"'
            # Rule 1: C-octal confusion (source-text signature).
            if _OCTAL_RE.search(body):
                findings.append(
                    (
                        lineno,
                        "octal-escape",
                        lit,
                        r"`\0`+digits is NUL+literal digits (Rust has no octal)",
                    )
                )
                continue
            # Rule 2a: odd byte length — not a whole number of UTF-16 units.
            if len(raw) % 2 != 0:
                findings.append(
                    (
                        lineno,
                        "odd-length",
                        lit,
                        f"{len(raw)} bytes — not a whole number of UTF-16 "
                        "code units (ANSI cast to wide?)",
                    )
                )
                continue
            # Rule 2b: missing UTF-16 NUL terminator (must end in 00 00).
            if raw[-2:] != b"\x00\x00":
                findings.append(
                    (lineno, "no-terminator", lit, "wide string does not end in a u16 NUL (00 00)")
                )
    return findings


def main():
    check_mode = "--check" in sys.argv
    args = [a for a in sys.argv[1:] if not a.startswith("--")]

    roots = [Path(a) for a in args] if args else [Path("sdk/lib/rust")]
    files: list[Path] = []
    for p in roots:
        if p.is_dir():
            files.extend(
                f
                for f in p.rglob("*.rs")
                if "/target/" not in str(f) and "/.responses/" not in str(f)
            )
        elif p.exists():
            files.append(p)
        else:
            print(f"ERROR: {p} not found. Run from vibeos/ root.")
            sys.exit(2)
    files = sorted(set(files))

    by_file: dict[str, list] = {}
    total = 0
    for f in files:
        found = check_file(f)
        if found:
            by_file[str(f)] = found
            total += len(found)

    if total == 0:
        print("No mis-encoded UTF-16 byte-string literals found.")
        return

    print(f"Found {total} mis-encoded wide byte-string literal(s) in {len(by_file)} file(s):\n")
    for filepath, found in sorted(by_file.items()):
        print(f"  {filepath}:")
        for lineno, kind, lit, why in found:
            print(f"    L{lineno:<5d} [{kind}] {lit}")
            print(f"            -> {why}")
        print()
    print(f"Total: {total} finding(s) in {len(by_file)} file(s)")
    if check_mode:
        sys.exit(1)


if __name__ == "__main__":
    main()
