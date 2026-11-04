"""Rust byte-string literals may hold only ASCII; C narrow literals hold any bytes.

A C narrow literal written in UTF-8 (`"... — guest SEH dispatch disabled\\n"`) is a
sequence of bytes, so the faithful Rust is `b"... \\xe2\\x80\\x94 ..."`. The model copies
the character instead, and `b"—"` does not parse ("non-ASCII character in byte string
literal"): wow64's `Wow64LdrpInitialize` was rejected outright for one em dash in a
DbgPrint format. Escaping the UTF-8 bytes is exact, so it is a normalization, not a guess.
"""

from __future__ import annotations

import re

# `b"..."` not preceded by an identifier character (so `ab"` / `rb"` are left alone) and
# not a raw `br"..."`. Escapes inside are skipped as a unit so `\\"` never ends the match.
_BYTE_STR = re.compile(r'(?<![A-Za-z0-9_])b"((?:[^"\\]|\\.)*)"', re.S)


def _escape(body: str) -> str:
    return "".join(
        c if ord(c) < 0x80 else "".join(f"\\x{b:02x}" for b in c.encode("utf-8")) for c in body
    )


def escape_nonascii_byte_strings(rust: str) -> str:
    """Every non-ASCII character inside a `b"..."` literal as its UTF-8 `\\xNN` bytes."""
    return _BYTE_STR.sub(
        lambda m: f'b"{_escape(m.group(1))}"' if not m.group(1).isascii() else m.group(0), rust
    )
