#!/usr/bin/env python3
"""
scan_fnptr_abi_transmute.py — a Rust-ABI fn smuggled into a C callback slot.

    Some(core::mem::transmute(GuiInitFrontEnd as *const ()))

`f as *const ()` erases the fn's ABI, and the transmute re-types it as whatever
the slot declares (`extern "system" fn`, a DLGPROC, a thread start routine).
rustc can no longer check that the two agree, so a plain `unsafe fn` (the Rust
ABI) lands in a stdcall slot. On amd64/arm64 a one- or two-pointer Rust fn
happens to be called like the C ABI, so it works and gets copied. On i386 the
callee reads its arguments from the wrong place. A BOOL slot filled by a fn
returning u8 leaves the upper register bits as garbage on every arch.

Found 2026-09-23: consrv's GuiVtbl filled every slot this way under a comment
claiming the impls were `extern "system"`. 22 of 25 were not, and six returned
u8 for C's BOOL. It stayed invisible until console creation first worked on
i386, and then csrss died (BUGCHECK 0xF4). The same shape was in msgina's dialog
procs and advapi32's crypt provider callbacks.

FIX: declare the fn `extern "system"` (or whatever the slot's ABI is) and pass
it DIRECTLY, e.g. `Some(f)`. rustc then checks the ABI and the signature. Keep a
transmute only when the types differ in spelling alone, and never for ABI.

The check is per crate: a name counts as Rust-ABI when every definition of it
in the crate is a bodied `fn` without `extern "..."`. Foreign declarations
(body-less `fn x(..);` inside an `extern` block) are foreign, never Rust-ABI.

Usage:
    python3 tools/scan_fnptr_abi_transmute.py [roots...]   # default: sdk/lib/rust
    python3 tools/scan_fnptr_abi_transmute.py --lint       # terse (CLI baseline gate)

Suppress a verified false positive on its line:
    ... transmute(f as *const ()) ... // nolint: fnptr-abi-transmute (reason)
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

SKIP_FILE_RE = re.compile(
    r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)"
)
NOLINT_RE = re.compile(r"//\s*nolint(?::\s*fnptr-abi-transmute\b|\s*$|:?\s*\()")

FN_RE = re.compile(
    r"^[ \t]*(?:#\[[^\]\n]*\][ \t]*)*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:unsafe\s+)?"
    r'(extern\s+"[A-Za-z0-9_-]+"\s+)?fn\s+(\w+)\s*(?:<[^>{;]*>)?\s*\(',
    re.M,
)
TRANSMUTE_RE = re.compile(
    r"transmute(?:::<[^>]*>)?\(\s*(\w+)\s+as\s+\*(?:const|mut)\s*"
    r"(?:\(\)|c_void|core::ffi::c_void|std::ffi::c_void)\s*\)"
)


def crate_of(path: Path, roots: list[Path]) -> Path:
    """Nearest ancestor holding a Cargo.toml, else the scan root it came from."""
    for parent in path.parents:
        if (parent / "Cargo.toml").is_file():
            return parent
        if parent in roots:
            return parent
    return path.parent


def _is_foreign(text: str, sig_open: int) -> bool:
    """True when the fn whose '(' is at sig_open ends in ';' (no body)."""
    depth = 1
    k = sig_open + 1
    n = len(text)
    while k < n and depth:
        c = text[k]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
        k += 1
    while k < n and text[k] not in "{;":
        k += 1
    return k < n and text[k] == ";"


def scan(roots: list[Path]) -> list[tuple[Path, int, str]]:
    by_crate: dict[Path, list[Path]] = defaultdict(list)
    for root in roots:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if SKIP_FILE_RE.search(str(f)):
                continue
            by_crate[crate_of(f, roots)].append(f)

    findings: list[tuple[Path, int, str]] = []
    for files in by_crate.values():
        texts: dict[Path, str] = {}
        for f in files:
            try:
                texts[f] = f.read_text(errors="replace")
            except OSError:
                continue
        abi: dict[str, set[str]] = defaultdict(set)
        for text in texts.values():
            for m in FN_RE.finditer(text):
                foreign = _is_foreign(text, m.end() - 1)
                abi[m.group(2)].add("ext" if (m.group(1) or foreign) else "rust")
        for f, text in texts.items():
            lines = text.splitlines()
            for m in TRANSMUTE_RE.finditer(text):
                name = m.group(1)
                if abi.get(name) != {"rust"}:
                    continue
                lineno = text.count("\n", 0, m.start()) + 1
                if NOLINT_RE.search(lines[lineno - 1]):
                    continue
                findings.append((f, lineno, name))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, name in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        if lint:
            print(f"{rel}:{lineno}: fnptr-abi-transmute {name}")
        else:
            print(f"{rel}:{lineno}: Rust-ABI fn `{name}` transmuted into a fn-pointer slot")
    if findings and not lint:
        print(
            f'\n{len(findings)} finding(s): declare each fn extern "system" and pass it '
            "directly (Some(f)) so rustc checks the ABI."
        )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
