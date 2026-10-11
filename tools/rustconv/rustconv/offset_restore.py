"""Put back the `FIELD_OFFSET`s c2rust evaluated into numbers, from the C table they came from.

c2rust translates with ONE target's layout, and folds `FIELD_OFFSET(DRIVER_INFO_3W, pName)`
to that target's number: localspl's `dwDriverInfo3Offsets` came out `[8, 16, 24, ..]`, the
64-bit offsets. On i386 the pointers are 4 bytes, so `PackStrings` wrote each string pointer
into the wrong slot, past the end of the struct, and left `pDependentFiles` (0x1c) zero:
`ToMultiSz(NULL)` crashed spoolsv on the first `GetPrinterDriver` level 3 (wordpad's print
setup). amd64 and arm64 share the layout, so nothing else saw it.

Decidable from the two texts: a file-scope C array whose initializer is a list of
`FIELD_OFFSET(T, f)` / `offsetof(T, f)` entries, and the Rust static of the same name with
the same number of entries. Each such entry becomes `core::mem::offset_of!(T, f) as <elem>`,
which every target computes for itself. Any other entry (a `MAXDWORD` terminator) is kept.
A count mismatch, or an entry that is not a literal number, is reported and left alone.
"""

from __future__ import annotations

import re
from pathlib import Path

_C_TABLE = re.compile(
    r"^[ \t]*(?:static\s+)?(?:const\s+)?[A-Za-z_]\w*\s+(?:const\s+)?([A-Za-z_]\w*)\s*\[[^\]]*\]"
    r"\s*=\s*\{(.*?)\}\s*;",
    re.M | re.S,
)
_OFFSET = re.compile(r"^(?:FIELD_OFFSET|offsetof)\s*\(\s*([A-Za-z_]\w*)\s*,\s*([\w.]+)\s*\)$")
_RUST_TABLE = re.compile(
    r"\bstatic\s+(?:mut\s+)?([A-Za-z_]\w*)\s*:\s*\[\s*([A-Za-z_][\w:]*)\s*;[^\]]*\]\s*=\s*\["
)
# A number c2rust folded, under any chain of casts:
# `8 as ::core::ffi::c_ulonglong as LONG as DWORD`.
_BAKED = re.compile(r"^\(?\s*\d+\s*\)?(?:\s+as\s+[\w:]+)*$")
_KEYWORDS = {"type", "match", "fn", "self", "Self", "ref", "move", "loop", "box", "use", "mod"}


def _split_top(s: str) -> list[str]:
    out, depth, cur = [], 0, []
    for ch in s:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    if "".join(cur).strip():
        out.append("".join(cur))
    return out


def _strip_c_comments(s: str) -> str:
    return re.sub(r"/\*.*?\*/|//[^\n]*", " ", s, flags=re.S)


def c_offset_tables(c_text: str) -> dict[str, list[tuple[str, str] | None]]:
    """name -> per-entry (type, field) for FIELD_OFFSET entries, None for any other entry."""
    out: dict[str, list[tuple[str, str] | None]] = {}
    for m in _C_TABLE.finditer(_strip_c_comments(c_text)):
        entries = [e.strip() for e in _split_top(m.group(2)) if e.strip()]
        parsed = [
            (lambda o: (o.group(1), o.group(2)) if o else None)(_OFFSET.match(e)) for e in entries
        ]
        if any(parsed):
            out[m.group(1)] = parsed
    return out


def _field(f: str) -> str:
    # bindgen's spelling of a keyword-named C field, per path component.
    return ".".join(p + "_" if p in _KEYWORDS else p for p in f.split("."))


def restore(rust: str, c_text: str) -> tuple[str, list[str], list[str]]:
    """(new text, restored table names, refused table names with why)."""
    tables = c_offset_tables(c_text)
    if not tables:
        return rust, [], []
    fixed: list[str] = []
    refused: list[str] = []
    for m in reversed(list(_RUST_TABLE.finditer(rust))):
        name, elem = m.group(1), m.group(2)
        want = tables.get(name)
        if want is None:
            continue
        open_at = m.end() - 1
        depth, close = 0, -1
        for i in range(open_at, len(rust)):
            depth += {"[": 1, "]": -1}.get(rust[i], 0)
            if depth == 0:
                close = i
                break
        if close < 0:
            continue
        entries = _split_top(rust[open_at + 1 : close])
        if len(entries) != len(want):
            refused.append(f"{name} ({len(entries)} Rust entries, {len(want)} in the C)")
            continue
        new_entries, changed = [], False
        for raw, w in zip(entries, want, strict=True):  # lengths checked above
            if w is None:
                new_entries.append(raw)
                continue
            body = raw.strip()
            if body.startswith(("core::mem::offset_of!", "::core::mem::offset_of!")):
                new_entries.append(raw)
                continue
            if not _BAKED.match(body):
                refused.append(f"{name} (entry {body!r} is not a folded number)")
                new_entries = []
                break
            lead = raw[: len(raw) - len(raw.lstrip())]
            new_entries.append(f"{lead}core::mem::offset_of!({w[0]}, {_field(w[1])}) as {elem}")
            changed = True
        if not new_entries or not changed:
            continue
        tail = "," if rust[open_at + 1 : close].rstrip().endswith(",") else ""
        indent = re.search(r"\n([ \t]*)\]\s*;?\s*$", rust[: close + 1])
        closing = "\n" + (indent.group(1) if indent else "") if "\n" in rust[open_at:close] else ""
        rust = rust[: open_at + 1] + ",".join(new_entries) + tail + closing + rust[close:]
        fixed.append(name)
    return rust, sorted(fixed), refused


def restore_file(rs: Path, c_file: Path) -> tuple[list[str], list[str]]:
    text = rs.read_text()
    new, fixed, refused = restore(text, c_file.read_text(errors="replace"))
    if new != text:
        rs.write_text(new)
    return fixed, refused
