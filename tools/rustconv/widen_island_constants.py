#!/usr/bin/env python3
"""widen_island_constants.py — a bindgen island's `L"..."` macro constants, made wide again.

THE BUG. bindgen emits a string macro as a byte string and DROPS the `L`:

    #define DD_ATA_REG_MAX_TARGET_ID  L"MaxTargetId"          (ata_user.h)
    pub const DD_ATA_REG_MAX_TARGET_ID: &[u8; 12] = b"MaxTargetId\\0";

so the island's constant is NARROW where the C's is UTF-16. Passed where a `PCWSTR` is wanted
it is a type error (atapi's AtaSetPortRegistryKey caught it); passed through a PVOID or a cast
it is a registry value name nobody wrote. Across the tree's islands 400-odd constants had this
shape (`DD_*_DEVICE_NAME_U`, `*_CLASSW`, `SERVICES_ACTIVE_DATABASEW`, ...).

THE ORACLE IS THE C. A constant is widened only when the tree defines that name as `L"<text>"`
with ONE text everywhere, and that text is exactly what the narrow byte string holds. Anything
else (a name `#define`d two ways, an escape this does not decode) is left alone and reported.

The wide form is `&[u16; N]` with the NUL, so `X.as_ptr()` is the C's `PCWSTR` and the array
decay crate-repair already performs (`&[u16; N]` -> `*const u16`) applies unchanged.

Usage: widen_island_constants.py [--repo DIR] FILE.rs [FILE.rs ...]   (rewrites in place)
"""

from __future__ import annotations

import argparse
import pathlib
import re
import subprocess
import sys

_DEFINE = re.compile(r'^\s*#\s*define\s+([A-Za-z_]\w*)\s+L"((?:[^"\\]|\\\\)*)"\s*(?://.*|/\*.*)?$')
_NARROW = re.compile(
    r"^(?P<head>pub const (?P<name>[A-Za-z_]\w*): )&\[u8; (?P<n>\d+)\] = "
    r'b"(?P<body>(?:[^"\\]|\\.)*)";',
    re.M,
)


def wide_defines(repo: pathlib.Path) -> dict[str, str | None]:
    """`NAME -> text` for every `#define NAME L"text"` in the tree; None when two disagree."""
    out = subprocess.run(
        ["git", "grep", "-h", "-E", r'^\s*#\s*define\s+[A-Za-z_]\w*\s+L"', "--", "*.h", "*.c"],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    seen: dict[str, str | None] = {}
    for line in out.splitlines():
        m = _DEFINE.match(line)
        if not m:
            continue
        text = m.group(2).replace("\\\\", "\\")
        if m.group(1) in seen and seen[m.group(1)] != text:
            seen[m.group(1)] = None
        else:
            seen.setdefault(m.group(1), text)
    return seen


_ALIAS = re.compile(r"^\s*#\s*define\s+([A-Za-z_]\w*)\s+([A-Za-z_]\w*[AW])\s*(?://.*|/\*.*)?$")


def alias_defines(repo: pathlib.Path) -> dict[str, set[str]]:
    """`NAME -> {targets}` for every `#define NAME XxxA|XxxW` in the tree's headers: the SDK's
    A/W switch (`#define FINDMSGSTRING FINDMSGSTRINGW` under UNICODE, `...A` otherwise)."""
    out = subprocess.run(
        [
            "git",
            "grep",
            "-h",
            "-E",
            r"^\s*#\s*define\s+[A-Za-z_]\w*\s+[A-Za-z_]\w*[AW]\s*($|/)",
            "--",
            "*.h",
        ],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    seen: dict[str, set[str]] = {}
    for line in out.splitlines():
        m = _ALIAS.match(line)
        if m:
            seen.setdefault(m.group(1), set()).add(m.group(2))
    return seen


def island_charset(text: str) -> str | None:
    """ "W" when the island was generated UNICODE, "A" when not, None when it does not say.

    The island's own `TCHAR` is the evidence: a UNICODE module UNBLOCKs the TCHAR family so
    bindgen emits it from the module's -DUNICODE compile."""
    m = re.search(r"^pub type TCHAR = (\w+);", text, re.M)
    if m is None:
        return None
    return "W" if m.group(1) in ("WCHAR", "u16", "wchar_t") else "A"


def _narrow_text(body: str) -> str | None:
    """The bytes a Rust `b"..."` body spells, as text; None for an escape not decoded here."""
    out, i = [], 0
    while i < len(body):
        ch = body[i]
        if ch != "\\":
            out.append(ch)
            i += 1
            continue
        nxt = body[i + 1 : i + 2]
        if nxt in ("\\", '"', "'"):
            out.append(nxt)
            i += 2
        elif nxt == "0":
            out.append("\0")
            i += 2
        else:
            return None
    return "".join(out)


def widen(
    text: str,
    wide: dict[str, str | None],
    aliases: dict[str, set[str]] | None = None,
    charset: str | None = None,
) -> tuple[str, list[str], list[str]]:
    """Rewrite each narrowed constant the C defines wide. Returns (text, widened, refused).

    With `aliases`, a UNICODE island's A/W switch names are widened through their W target:
    bindgen evaluates `FINDMSGSTRING` -> `FINDMSGSTRINGW` -> `L"commdlg_FindReplace"` and
    drops the `L` just the same, and notepad registered its Find/Replace message from those
    narrow bytes read as UTF-16 -- Find Next never fired."""
    widened: list[str] = []
    refused: list[str] = []
    # The island's own TCHAR is the best evidence; `charset` (from the module's compile
    # flags) answers when the island does not say -- kbswitch's allowlisted headers use no
    # TCHAR, so its island had none and INDICATOR_CLASS stayed the narrow INDICATOR_CLASSA
    # bytes in a -DUNICODE module (five type errors, and a class name nobody registered).
    charset = (island_charset(text) or charset) if aliases else None

    def one(m: re.Match[str]) -> str:
        name = m.group("name")
        if name not in wide and aliases and charset == "W":
            targets = {t for t in aliases.get(name, set()) if t.endswith("W")}
            if len(targets) == 1 and next(iter(targets)) in wide:
                name = next(iter(targets))
        if name not in wide:
            return m.group(0)
        c_text = wide[name]
        narrow = _narrow_text(m.group("body"))
        if c_text is None or narrow != c_text + "\0" or not c_text.isascii():
            refused.append(name)
            return m.group(0)
        units = [ord(c) for c in c_text] + [0]
        body = ", ".join(f"0x{u:04X}" for u in units)
        widened.append(name)
        return f'{m.group("head")}&[u16; {len(units)}] = &[{body}]; // L"{m.group("body")[:-2]}"'

    return _NARROW.sub(one, text), widened, refused


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument(
        "--repo", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[2]
    )
    ap.add_argument("files", nargs="+", type=pathlib.Path)
    args = ap.parse_args(argv)
    wide = wide_defines(args.repo)
    aliases = alias_defines(args.repo)
    for path in args.files:
        text = path.read_text()
        new, widened, refused = widen(text, wide, aliases)
        if new != text:
            path.write_text(new)
        print(f"widen_island_constants: {path.name}: {len(widened)} widened", end="")
        print(f", refused (C defines it otherwise): {', '.join(refused)}" if refused else "")
    return 0


if __name__ == "__main__":
    sys.exit(main())
