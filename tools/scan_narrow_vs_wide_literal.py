#!/usr/bin/env python3
"""
scan_narrow_vs_wide_literal.py — a narrow b"..." where the file's own C passes L"...".

    C:     RtlCompareMemory(FilePattern.Buffer, L"*.*", 6) == 6
    Rust:  RtlCompareMemory(FilePattern.Buffer as PVOID, b"*.*\\0".as_ptr() as PVOID, 6) == 6

The C compares six bytes of UTF-16; the Rust compared them with four bytes of ASCII, so
the test could never match. kernel32's FindFirstFileExW never shortened "*.*" to "*", sent it
through the DOS-wildcard rewrite instead, and that rejects "." and "..": FindFirstFileW on
any empty directory failed with ERROR_FILE_NOT_FOUND, and GUI setup stopped at
"CopySystemProfile() failed". The cast to PVOID hides the element type from the compiler.

The oracle is the C left in the file ("ORIGINAL C SOURCE"): for each Rust call with a
`b"TEXT"` argument, a same-arity call of the same function in the C passing `L"TEXT"` at
that position is reported. Gated at zero.

Usage:
    python3 tools/scan_narrow_vs_wide_literal.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_narrow_vs_wide_literal.py --lint
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_call_literal_vs_c import C_CALL_RE, _calls
from tools.scan_wide_byte_copy import SKIP_FILE_RE
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
MARKER = "ORIGINAL C SOURCE"
NARROW_RE = re.compile(r'\bb"((?:\\.|[^"\\])*)"')
NOLINT_RE = re.compile(r"nolint:\s*narrow-vs-wide\b")
WIDE_RE = re.compile(r'^\(?\s*L"((?:\\.|[^"\\])*)"')


def _call_end(code: str, pos: int) -> int:
    """Index just past the `)` closing the call that starts at `pos`."""
    depth = 0
    for j in range(code.index("(", pos), len(code)):
        if code[j] == "(":
            depth += 1
        elif code[j] == ")":
            depth -= 1
            if depth == 0:
                return j + 1
    return len(code)


def _text(lit: str) -> str:
    return re.sub(r"\\0$", "", lit)


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    findings = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if f.suffix != ".rs" or SKIP_FILE_RE.search(str(f)):
                continue
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            cut = text.find(MARKER)
            if cut < 0:
                continue
            c = "\n".join(re.sub(r"^\s*//\s?", "", ln) for ln in text[cut:].splitlines())
            wide: dict[tuple[str, int, int], set[str]] = {}
            for _pos, fn, args in _calls(c, C_CALL_RE):
                for i, a in enumerate(args):
                    m = WIDE_RE.match(a.strip())
                    if m:
                        wide.setdefault((fn, len(args), i), set()).add(m.group(1))
            if not wide:
                continue
            rust = re.sub(r"//[^\n]*", lambda m: " " * len(m.group(0)), text[:cut])
            for pos, fn, args in _calls(rust):
                for i, a in enumerate(args):
                    m = NARROW_RE.search(a)
                    if not m or re.search(r"!\s*\(\s*$", a[: m.start()]):
                        continue  # wstr!(b"..") and friends convert to UTF-16 themselves
                    if _text(m.group(1)) in wide.get((fn, len(args), i), set()):
                        line = rust.count("\n", 0, pos) + 1
                        span = text[pos : _call_end(rust, pos)]
                        if NOLINT_RE.search(span):
                            continue
                        findings.append(
                            (f, line, f'{fn} arg{i}: b"{_text(m.group(1))}" where C passes L"..."')
                        )
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, what in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        tag = "narrow-vs-wide-literal" if lint else "narrow literal where the C is wide:"
        print(f"{rel}:{lineno}: {tag} {what}")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): use a UTF-16 literal, as the C does.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
