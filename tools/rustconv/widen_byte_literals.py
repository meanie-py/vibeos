#!/usr/bin/env python3
"""widen_byte_literals.py — replace `b"...".as_ptr() as LPCWSTR` with `w!("...")`.

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
    if len(raw) % 2 == 0 and all(b == 0 for b in raw[1::2]):
        try:
            return raw.decode("utf-16-le").rstrip("\x00")
        except UnicodeDecodeError:
            return None
    narrow = raw.rstrip(b"\x00")
    if narrow and all(0x20 <= b < 0x7F for b in narrow):
        return narrow.decode("ascii")  # narrow ASCII behind a wide pointer — the content bug
    return None


def c_has_literal(c_text: str, text: str) -> bool:
    """Does the C spell this exact string as a wide literal?"""
    if not text:
        return True
    esc = text.replace("\\", "\\\\").replace('"', '\\"')
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
            notes.append(f"  ! {rs.name}:{line} not decodable as text, left alone: {m.group('body')[:50]!r}")
            return m.group(0)
        if not c_has_literal(c_text, got):
            notes.append(f"  ! {rs.name}:{line} decodes to {got!r}, which the C never spells — left alone")
            return m.group(0)
        n += 1
        esc = got.replace("\\", "\\\\").replace('"', '\\"')
        return f'w!("{esc}") as {m.group("ty")}'

    new = _LIT.sub(repl, text)
    if n and apply:
        rs.write_text(new, encoding="utf-8")
    return n, notes


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
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
    print(f"\n{'converted' if args.apply else 'would convert'} {total}; {len(all_notes)} left alone")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
