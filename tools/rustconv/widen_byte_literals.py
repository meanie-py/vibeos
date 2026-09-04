#!/usr/bin/env python3
"""widen_byte_literals.py — restore a C `L"..."` that came back spelled some other way.

Two shapes, one oracle. The first is `b"...".as_ptr() as LPCWSTR`; the second is the
ELEMENTWISE array, `['S' as u16, 'Y' as u16, ..., 0].as_ptr()`, which syssetup's wizard.c
produced 54 of. The elementwise shape compiles in a `static` (Rust promotes the array to a
`'static` allocation) but not in a function body, where the array is a TEMPORARY and the
pointer dangles the moment the statement ends — and either way it is 40 lines of unreadable
`as u16` that nothing can diff against the C.

THE BUG. A C `L"..."` constant routinely comes back as a Rust BYTE string cast to a wide
pointer. That is wrong in up to three independent ways at once:

  * ALIGNMENT. `b"..."` has alignment 1. A UTF-16 read from an odd address is a 0x1E
    STATUS_DATATYPE_MISALIGNMENT on amd64 and silently wrong elsewhere.
  * TERMINATOR. A wide NUL is TWO zero bytes. Rust appends neither, so `b"Foo\\0"` ends in a
    single zero byte and the string has no terminator at all — the callee reads past it.
  * CONTENT. Worst and commonest: the literal is often NARROW ASCII behind the wide pointer
    (`b"DeviceType\\0"`), so every other code unit is garbage and the string is not the string
    the C passed. `RegQueryValueExW` looks up a key nobody named.

THE ORACLE IS THE C. Every replacement is checked back against the module's own .c: the decoded
text must appear there as `L"<text>"` (or inside a TEXT()/wide literal). A literal that does not
round-trip is REPORTED AND LEFT ALONE — the point of this pass is to remove a silent corruption,
not to introduce a confident one.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

# `b"...".as_ptr() as LPCWSTR` and friends, including a `(b"..." as *const u8)` middle step.
_LIT = re.compile(
    r'b"(?P<body>(?:[^"\\]|\\.)*)"\s*(?:as\s+\*const\s+u8\s*)?\.as_ptr\(\)\s*(?:as\s+\*const\s+u8\s*)?'
    r"as\s+(?P<ty>LPCWSTR|PCWSTR|LPWSTR|PWSTR|LPCTSTR|LPTSTR|\*const u16|\*mut u16)"
)


# The ELEMENTWISE shape: `['S' as u16, 'Y' as u16, ..., 0].as_ptr()`. Elements are Rust char
# literals cast to u16, or bare integers (the NULs, and the odd `0x5C`). Requiring at least one
# `as u16` keeps `[0; 260]` and other genuine numeric arrays out.
_WCHAR_ELEM = r"(?:'(?:\\.|[^'\\])'\s*as\s+u16|0[xX][0-9A-Fa-f]+|\d+)"
_WIDE_ARR = re.compile(
    r"\[\s*(?P<body>" + _WCHAR_ELEM + r"(?:\s*,\s*" + _WCHAR_ELEM + r")*)\s*,?\s*\]\s*\.as_ptr\(\)"
)
_ELEM = re.compile(_WCHAR_ELEM)

_RUST_CHAR_ESC = {"n": 0x0A, "r": 0x0D, "t": 0x09, "0": 0x00, "\\": 0x5C, "'": 0x27, '"': 0x22}


def decode_units(body: str) -> str | None:
    """The text an elementwise `['x' as u16, .., 0]` array spells, or None if it isn't one."""
    units: list[int] = []
    for tok in _ELEM.findall(body):
        if tok.startswith("'"):
            ch = tok[1 : tok.index("'", 1)]
            if ch.startswith("\\"):
                if ch[1] in ("x", "u"):
                    return None  # \xNN / \u{..} — rare enough to hand-check rather than guess
                if ch[1] not in _RUST_CHAR_ESC:
                    return None
                units.append(_RUST_CHAR_ESC[ch[1]])
            elif len(ch) == 1:
                units.append(ord(ch))
            else:
                return None
        else:
            units.append(int(tok, 0))
    if not units:
        return None
    if any(u > 0xFFFF for u in units):
        return None
    # Strip exactly ONE trailing zero — the terminator. Any further zero is CONTENT: the C's
    # `L"Terminal Server\0"` is a double-NUL-terminated list, and eating its embedded NUL would
    # silently change what the string means to the callee.
    if units and units[-1] == 0:
        units = units[:-1]
    try:
        return "".join(chr(u) for u in units)
    except ValueError:
        return None


def unescape(body: str) -> bytes:
    """The literal's actual bytes. Rust escapes we can meet in generated code."""
    out = bytearray()
    i = 0
    while i < len(body):
        c = body[i]
        if c != "\\":
            out += c.encode("utf-8")
            i += 1
            continue
        n = body[i + 1]
        if n == "x":
            out.append(int(body[i + 2 : i + 4], 16))
            i += 4
        elif n == "0":
            out.append(0)
            i += 2
        elif n == "n":
            out.append(0x0A)
            i += 2
        elif n == "r":
            out.append(0x0D)
            i += 2
        elif n == "t":
            out.append(0x09)
            i += 2
        else:  # \\ and \"
            out += n.encode("utf-8")
            i += 2
    return bytes(out)


def decode(raw: bytes) -> str | None:
    """The text the literal was meant to be, whether it was UTF-16-encoded or plain ASCII."""
    if not raw:
        return ""
    # NEVER rstrip the RAW bytes: the high byte of the last UTF-16 unit is itself 0x00, so
    # stripping trailing zeros eats half of the final character ('s' -> 0x73,0x00 -> 0x73) and
    # leaves an odd length that then fails to decode. Decode first, strip CODE UNITS after.
    # The TERMINATOR bug in its commonest shape: the body is UTF-16LE and then ONE trailing zero
    # byte, because the C's `L"foo"` was transliterated as `b"f\0o\0o\0\0"` — half a wide NUL. The
    # byte count is then ODD, which is not decodable as UTF-16 at all, so the pass used to report
    # it and walk away, leaving the very corruption it exists to remove. Peel the lone byte and
    # decode the prefix: the whole point is that the terminator is WRONG.
    if len(raw) % 2 == 1 and raw[-1] == 0:
        raw = raw[:-1]
    if len(raw) % 2 == 0 and all(b == 0 for b in raw[1::2]):
        try:
            return raw.decode("utf-16-le").rstrip("\x00")
        except UnicodeDecodeError:
            return None
    # WRONG BYTE ORDER. `b"\0F\0a\0i\0l\0e\0d"` puts the zero BEFORE each character: that is
    # UTF-16BE, and reading it little-endian (which is what the callee does) yields CJK garbage,
    # not the string. A separate bug from the terminator and from narrowness, and the only clue
    # is which half of each pair is zero.
    if len(raw) % 2 == 0 and raw and all(b == 0 for b in raw[0::2]):
        try:
            return raw.decode("utf-16-be").rstrip("\x00")
        except UnicodeDecodeError:
            return None
    narrow = raw.rstrip(b"\x00")
    # \t \r \n are ordinary content in a format string — logfile.c's L"..\r\n" came back as a
    # NARROW string behind a PCWSTR, and rejecting it for holding a CR left the corruption in.
    if narrow and all(0x20 <= b < 0x7F or b in (0x09, 0x0A, 0x0D) for b in narrow):
        return narrow.decode("ascii")  # narrow ASCII behind a wide pointer — the content bug
    return None


def _c_escape(text: str) -> str:
    """The text as the C SOURCE spells it — control characters back to their escapes.

    The oracle compares against source, not against bytes: a decoded "\\r\\n" is written `\\r\\n`
    in the .c, so comparing the raw characters never matches and every format string with a
    newline reads as "the C never spells this".
    """
    out = text.replace("\\", "\\\\").replace('"', '\\"')
    for ch, esc in (("\t", "\\t"), ("\r", "\\r"), ("\n", "\\n"), ("\0", "\\0")):
        out = out.replace(ch, esc)
    return out


def c_has_literal(c_text: str, text: str) -> bool:
    """Does the C spell this exact string as a wide literal?

    The EMPTY string is checked like any other. `[0].as_ptr()` is `L""` in the sound-event
    tables, but a bare `[0]` is also what a one-element numeric array looks like, and the only
    thing that tells them apart is whether the C spells `L""` at all.
    """
    esc = _c_escape(text)
    return f'L"{esc}"' in c_text or f'TEXT("{esc}")' in c_text or f'_T("{esc}")' in c_text


def convert(rs: pathlib.Path, c_text: str, apply: bool) -> tuple[int, list[str]]:
    text = rs.read_text(encoding="utf-8")
    notes: list[str] = []
    n = 0

    def repl(m: re.Match[str]) -> str:
        nonlocal n
        raw = unescape(m.group("body"))
        got = decode(raw)
        line = text[: m.start()].count("\n") + 1
        if got is None:
            notes.append(
                f"  ! {rs.name}:{line} not decodable as text, left alone: {m.group('body')[:50]!r}"
            )
            return m.group(0)
        if not c_has_literal(c_text, got):
            notes.append(
                f"  ! {rs.name}:{line} decodes to {got!r}, which the C never spells — left alone"
            )
            return m.group(0)
        n += 1
        # Rust spells \t \r \n the same way C does, so the source-form escape serves both the
        # oracle and the emitted literal. Emitting the raw control character would break the line.
        return f'w!("{_c_escape(got)}") as {m.group("ty")}'

    stage1 = _LIT.sub(repl, text)

    def repl_arr(m: re.Match[str]) -> str:
        nonlocal n
        got = decode_units(m.group("body"))
        line = stage1[: m.start()].count("\n") + 1
        if got is None:
            return m.group(0)  # not a string array at all — say nothing, it was never a finding
        if not c_has_literal(c_text, got):
            notes.append(
                f"  ! {rs.name}:{line} decodes to {got!r}, which the C never spells — left alone"
            )
            return m.group(0)
        n += 1
        return f'w!("{_c_escape(got)}")'

    new = _WIDE_ARR.sub(repl_arr, stage1)
    if n and apply:
        rs.write_text(new, encoding="utf-8")
    return n, notes


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("crate_src", help="the crate's module dir, e.g. sdk/lib/rust/foo-rust/src/foo")
    ap.add_argument("c_dir", help="the module's C source dir — the oracle")
    ap.add_argument("--apply", action="store_true")
    args = ap.parse_args(argv)

    src, c_dir = pathlib.Path(args.crate_src), pathlib.Path(args.c_dir)
    total, all_notes = 0, []
    for rs in sorted(src.glob("*.rs")):
        c = c_dir / (rs.stem + ".c")
        if not c.exists():
            hits = list(c_dir.rglob(rs.stem + ".c"))
            if not hits:
                continue
            c = hits[0]
        n, notes = convert(rs, c.read_text(encoding="utf-8", errors="replace"), args.apply)
        total += n
        all_notes += notes
        if n:
            print(f"  {rs.name}: {n} literal(s)")
    print("\n".join(all_notes))
    print(
        f"\n{'converted' if args.apply else 'would convert'} {total}; {len(all_notes)} left alone"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
