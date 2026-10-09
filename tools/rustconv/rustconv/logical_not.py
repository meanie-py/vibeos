"""C's logical `!x` translated as Rust's BITWISE `!x`, then compared with zero.

`if (!(ChannelBitmap & (1 << i))) continue;` came back from the model as
`if !(ChannelBitmap & (1 << i)) != 0 { continue; }`. It compiles -- `!` on an integer is a
bitwise not in Rust, so `!(x) != 0` is `x != 0xFFFFFFFF` -- and it is true for every channel,
so pciidex enumerated no IDE channels and the `--ata-stack` boot died with BugCheck 0x6B. No
compile error means no repair rule ever sees it.

The shape is decidable from the two sides: if `!(E) != 0` (or `== 0`) compiles, E is an
integer, and the only C that means a bitwise not is `~`. So inside a function whose C never
writes `~` -- or where the C writes `!(E)` with the same operand -- `!(E) != 0` is `(E) == 0`
and `!(E) == 0` is `(E) != 0`. Anything else is reported, never guessed.
"""

from __future__ import annotations

import re
from collections.abc import Mapping
from typing import Protocol


class _Decls(Protocol):
    # Read-only, so a frozen dataclass (pointer_depth.CFunctionDecls) satisfies it.
    @property
    def bitwise_not(self) -> bool: ...
    @property
    def logical_nots(self) -> frozenset[str]: ...


_SITE = re.compile(r"!\s*\(")
_OPERAND_START = re.compile(r"(?:[({,;]|&&|\|\||\bif|\bwhile|\breturn|[^=!<>]=)\s*$")


def _close(text: str, open_idx: int) -> int | None:
    depth = 0
    for i in range(open_idx, len(text)):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return i
    return None


def _fn_bodies(text: str):
    for m in re.finditer(
        r"^(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?fn\s+([A-Za-z_]\w*)\s*\(",
        text,
        re.M,
    ):
        brace = text.find("{", m.end())
        if brace < 0:
            continue
        depth, j = 0, brace
        while j < len(text):
            depth += {"{": 1, "}": -1}.get(text[j], 0)
            if depth == 0:
                break
            j += 1
        yield m.group(1), brace, j


def repair(rust: str, decls: Mapping[str, _Decls]) -> tuple[str, list[str], list[str]]:
    """Returns (text, fixed `fn: count` notes, refused `fn` notes)."""
    edits: list[tuple[int, int, str]] = []
    fixed: list[str] = []
    refused: list[str] = []
    for name, lo, hi in _fn_bodies(rust):
        n = 0
        for m in _SITE.finditer(rust, lo, hi):
            # `!=` is not a not; `!(` after an identifier char is a macro call (`ASSERT!(`).
            if m.start() > 0 and (rust[m.start() - 1].isalnum() or rust[m.start() - 1] == "_"):
                continue
            # Only a `!(..)` that STARTS the compared operand. After a binary operator it is a
            # mask -- `x & !(M) != 0` is `(x & ~M) != 0`, since `&` binds tighter than `!=`.
            if not _OPERAND_START.search(rust[lo : m.start()]):
                continue
            close = _close(rust, m.end() - 1)
            if close is None or close > hi:
                continue
            cmp = re.match(r"\s*(!=|==)\s*0\b(?!\.|x)", rust[close + 1 :])
            if cmp is None:
                continue
            c = decls.get(name)
            if c is None:
                continue
            inner = rust[m.end() : close]
            # The C spelling `!( same operand )` decides the site even where the function
            # uses `~` elsewhere (pciidex's QBR: `Bitmap & ~Seen` two lines above the test).
            same = re.sub(r"\s+", "", re.sub(r"\bas\s+[A-Za-z_]\w*", "", inner))
            if c.bitwise_not and same not in c.logical_nots:
                refused.append(name)
                continue
            op = "==" if cmp.group(1) == "!=" else "!="
            edits.append((m.start(), close + 1 + cmp.end(), f"({inner}) {op} 0"))
            n += 1
        if n:
            fixed.append(f"{name}: {n}")
    for start, end, repl in sorted(edits, reverse=True):
        rust = rust[:start] + repl + rust[end:]
    return rust, fixed, sorted(set(refused))
