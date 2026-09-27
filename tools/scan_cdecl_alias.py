#!/usr/bin/env python3
"""scan_cdecl_alias.py — a bare (cdecl) i386 symbol ALIASED onto a stdcall body.

The class (found 2026-09-26): on i386 a helper that exists as a stdcall body
`_X@N` and must also be reachable under the bare cdecl name `_X` was "bridged"
with a symbol alias:

    .set _KeInitializeSpinLock, _KeInitializeSpinLock@4

That resolves the link and is wrong at run time. The two conventions disagree
about who pops the N bytes of arguments: the stdcall body does `ret N`, and the
cdecl caller then pops them again, so every call leaves ESP N bytes too high.
A caller with an ESP-relative epilogue then restores its registers from the
wrong slots and returns through whatever sits above its own frame. fastfat's
DriverEntry made two such calls (+8), popped its own return address into EBX,
and `ret`-ed into its RegistryPath argument on the caller's stack: BUGCHECK 0x1E
with EIP on the stack, the first time a FAT volume made fs_rec load it on i386.

An alias is sound only for `@0`, where the two conventions are the same code.
For `@N` with N > 0 the bare name needs a real thunk that re-pushes the
arguments and calls the stdcall body (drivers/filesystems/fastfat/
rust_cohab_forceinline.c, `CDECL_TO_STDCALL`).

Scans C, assembly and Rust (`global_asm!`) sources. Output is one line per
finding, baselined by tools/lint-baseline/scan_cdecl_alias.txt (EMPTY) via
`vibeos-cli check`.
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ROOTS = (
    "boot",
    "base",
    "dll",
    "drivers",
    "hal",
    "modules",
    "ntoskrnl",
    "sdk",
    "subsystems",
    "win32ss",
)
EXTS = {".c", ".h", ".s", ".S", ".asm", ".inc", ".rs", ".cpp"}
# `.set _X, _X@N` / `.equ _X, _X@N`, also inside C/Rust asm string literals.
ALIAS_RE = re.compile(r"\.(?:set|equ)\s+_(\w+)\s*,\s*_(\w+)@(\d+)\b")


def _skipped(name: str) -> bool:
    return name.startswith((".", "build-")) or name in {"target", "node_modules"}


def iter_sources(paths: list[Path]):
    for root in paths:
        if root.is_file():
            yield root
            continue
        for dirpath, dirnames, filenames in os.walk(root):
            dirnames[:] = [d for d in dirnames if not _skipped(d)]
            for fn in filenames:
                if os.path.splitext(fn)[1] in EXTS:
                    yield Path(dirpath) / fn


def scan(paths: list[Path]) -> list[tuple[Path, int, str, int]]:
    findings = []
    for f in iter_sources(paths):
        try:
            text = f.read_text(errors="replace")
        except OSError:
            continue
        if ".set" not in text and ".equ" not in text:
            continue
        for lineno, line in enumerate(text.splitlines(), 1):
            for m in ALIAS_RE.finditer(line):
                bare, target, nbytes = m.group(1), m.group(2), int(m.group(3))
                if bare == target and nbytes > 0:
                    findings.append((f, lineno, bare, nbytes))
    return findings


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [REPO / r for r in ROOTS]
    findings = scan(paths)
    for f, lineno, name, nbytes in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        if lint:
            print(f"{rel}:{lineno}: cdecl-alias {name}@{nbytes}")
        else:
            print(f"{rel}:{lineno}: bare `_{name}` aliased onto stdcall `_{name}@{nbytes}`")
    if findings and not lint:
        print(
            f"\n{len(findings)} finding(s): a cdecl caller of the bare name pops the arguments a "
            "second time. Replace the alias with a thunk that re-pushes them and calls the "
            "stdcall body (see CDECL_TO_STDCALL in drivers/filesystems/fastfat/"
            "rust_cohab_forceinline.c)."
        )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
