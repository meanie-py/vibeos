#!/usr/bin/env python3
"""Point a test exe at a renamed DLL: the C oracle's same-length rename (crypt32 -> crypt3c).

A module the tree loads as a KnownDLL (crypt32, shlwapi) cannot be shadowed by a copy beside
the exe, so the oracle payloads (`*winetestc`) ship the pre-flip C under a same-length name
and patch every way the exe names the module. Every way, not just `name.dll`: Wine's suites
also call `GetModuleHandleA("crypt32")` / `GetModuleHandleA("shlwapi")` and look functions
up through that handle. Renaming only `.dll` strings sent those lookups to the shipping Rust,
so part of each "C" run graded the Rust against itself (crypt32 store.c, shlwapi string.c).

Renamed, case-insensitively, in ASCII and UTF-16LE:
- `name.dll` anywhere (the import table, LoadLibrary/GetModuleHandle strings);
- bare `name` as a whole NUL-terminated string (not inside a longer identifier or path).

Usage: rename_module_refs.py IN.exe OUT.exe OLD NEW   (OLD and NEW the same length)
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


def _ci(text: str, wide: bool) -> bytes:
    """A case-insensitive byte pattern for `text`, ASCII or UTF-16LE."""
    out = b""
    for ch in text:
        alts = {ch.lower(), ch.upper()}
        if wide:
            body = b"|".join(re.escape(a.encode("utf-16le")) for a in sorted(alts))
            out += b"(?:" + body + b")"
        else:
            out += b"[" + b"".join(re.escape(a.encode()) for a in sorted(alts)) + b"]"
    return out


def rename(data: bytes, old: str, new: str) -> tuple[bytes, dict[str, int]]:
    if len(old) != len(new):
        raise ValueError(f"{old!r} -> {new!r}: the rename must keep the length")
    counts: dict[str, int] = {}
    for wide in (False, True):
        nul = b"\x00\x00" if wide else b"\x00"
        enc = "utf-16le" if wide else "ascii"
        dll = _ci(old + ".dll", wide)
        bare = _ci(old, wide)
        # Bare: a whole string. Before it, a NUL (the previous string's terminator or
        # padding) or the start of the data; after it, the terminator.
        bare_rx = re.compile(rb"(?:(?<=" + nul + rb")|^)" + bare + rb"(?=" + nul + rb")")
        dll_rx = re.compile(dll)
        kind = "utf16" if wide else "ascii"
        pre, head = new.encode(enc), len(old.encode(enc))
        data, n_dll = dll_rx.subn(lambda m, pre=pre, head=head: pre + m.group(0)[head:], data)
        data, n_bare = bare_rx.subn(new.encode(enc), data)
        counts[f"{kind} {old}.dll"] = n_dll
        counts[f"{kind} {old}"] = n_bare
    return data, counts


def main(argv: list[str]) -> int:
    if len(argv) != 5:
        print(__doc__, file=sys.stderr)
        return 2
    src, dst, old, new = argv[1:]
    data, counts = rename(Path(src).read_bytes(), old, new)
    if not any(counts.values()):
        print(
            f"[rename_module_refs] no {old} reference to rename; "
            "the oracle would test the shipping module",
            file=sys.stderr,
        )
        return 1
    Path(dst).write_bytes(data)
    print("[rename_module_refs] " + ", ".join(f"{k}: {v}" for k, v in counts.items() if v))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
