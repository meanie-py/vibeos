#!/usr/bin/env python3
"""
scan_hresult_literals.py — bare HRESULT literals that the C function never returns.

THE CLASS. The translator often spelled an HRESULT as a bare number,
`return 0x80030006u32 as HRESULT;`, with no name beside it. When that number is
wrong nothing anywhere notices: it is a valid HRESULT, it has the right severity
bit, and usually the right facility. ole32's storage32.rs carried six of them in two
functions alone (2026-10): StorageBaseImpl_CreateStream returned STG_E_INVALIDHANDLE
and E_INVALIDARG where the C returns STG_E_INVALIDNAME and STG_E_INVALIDPARAMETER,
and BlockChainStream_WriteAt returned STG_E_TOOMANYOPENFILES for STG_E_READFAULT
*and* STG_E_WRITEFAULT. Neither scan_inline_flag_literals.py (it needs a
`/* NAME */` annotation to check against) nor scan_invented_constants.py (it needs
a `const`) can see a literal like that.

THE ORACLE is the C function the Rust function translates: same name, found in the
C module the crate translates (scan_invented_constants.c_dirs_for, deliberately
narrow). A literal passes when ANY name winerror.h gives its value, or the literal
itself, appears in that C function's body. Otherwise it is a MISMATCH and the
report lists the HRESULT names the C function does use. A Rust function with no
same-named C function is not judged (helpers, renames), nor is one whose C body
names no HRESULT at all: that is an NTSTATUS function (STATUS_* codes share the
0x8000xxxx range) or a C stub the Rust has since outgrown. An ALL-CAPS name is a
macro shim, not a translated function. This scanner prefers silence to guessing.

Usage:
    python3 tools/scan_hresult_literals.py                 # all crates
    python3 tools/scan_hresult_literals.py --crate ole32-rust
    python3 tools/scan_hresult_literals.py --lint          # terse, exit 1 on findings

Suppress a verified false positive on the line: `// nolint: hresult-literal (why)`.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.scan_invented_constants import c_dirs_for  # noqa: E402
from tools.source_walk import source_files  # noqa: E402

RUST_ROOT = REPO / "sdk" / "lib" / "rust"
WINERROR = REPO / "sdk" / "include" / "psdk" / "winerror.h"

HR_LIT_RE = re.compile(r"\b0x(8[0-9a-fA-F]{7})(?:u32)?\s+as\s+(?:HRESULT|i32|LONG)\b")
# A function DEFINITION (also the one-line `stdmethod! { fn X(` form); a line ending in
# `;` is an extern declaration and must not steal the attribution.
RUST_FN_RE = re.compile(
    r"^\s*(?:stdmethod!\s*\{\s*)?(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?"
    r"(?:extern\s+\"[^\"]+\"\s+)?fn\s+(\w+)"
)
# `const E_FAIL: HRESULT = 0x80004005...` NAMES its value: verify_constants' job, not ours.
RUST_CONST_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?const\s+\w+\s*:")
# A C function definition: a name, a parameter list that may hold one level of nested
# parens (function-pointer params), then `{` with no `;` in between.
C_DEF_RE = re.compile(r"\b(\w+)\s*\(((?:[^;{}()]|\([^;{}()]*\))*)\)\s*\{")
C_COMMENT_RE = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
C_KEYWORDS = {"if", "for", "while", "switch", "return", "sizeof", "do", "else"}
NAME_RE = re.compile(r"\b(?:[A-Z][A-Z0-9]*_E_\w+|E_[A-Z][A-Z0-9_]*|[A-Z]+_S_\w+)\b")
NOLINT = "nolint: hresult-literal"


def hresult_names() -> dict[int, set[str]]:
    names: dict[int, set[str]] = {}
    rx = re.compile(r"#define\s+(\w+)\s+_HRESULT_TYPEDEF_\((0x[0-9A-Fa-f]+)L?\)")
    for line in WINERROR.read_text(errors="replace").splitlines():
        m = rx.match(line)
        if m:
            names.setdefault(int(m.group(2), 16), set()).add(m.group(1))
    return names


def c_functions(c_dirs: list[Path]) -> dict[str, list[str]]:
    """name -> bodies of every same-named C function definition in the module."""
    out: dict[str, list[str]] = {}
    for d in c_dirs:
        for path in sorted(source_files(d, "*.c")):
            # Comments OUT first: wine puts `/* [out] */` between a signature's `)` and
            # its `{`, which hid storage32's CreateStream from the definition regex.
            text = C_COMMENT_RE.sub(" ", path.read_text(errors="replace"))
            for m in C_DEF_RE.finditer(text):
                name = m.group(1)
                if name in C_KEYWORDS:
                    continue
                i, depth = m.end(), 1
                while depth and i < len(text):
                    c = text[i]
                    if c == "{":
                        depth += 1
                    elif c == "}":
                        depth -= 1
                    i += 1
                out.setdefault(name, []).append(text[m.start() : i])
    return out


def scan_files(
    rs_files: list[Path], c_dirs: list[Path], names: dict[int, set[str]]
) -> list[tuple[str, int, str, str, str]]:
    funcs = c_functions(c_dirs)
    findings = []
    for rs in rs_files:
        fn = None
        for lineno, line in enumerate(rs.read_text(errors="replace").splitlines(), 1):
            m = RUST_FN_RE.match(line)
            if m and not line.rstrip().endswith(";"):
                fn = None if m.group(1).isupper() else m.group(1)
            if not fn or NOLINT in line or RUST_CONST_RE.match(line):
                continue
            for hm in HR_LIT_RE.finditer(line):
                value = int(hm.group(1), 16)
                bodies = funcs.get(fn)
                if not bodies:
                    continue
                body = "\n".join(bodies)
                c_has = sorted(set(NAME_RE.findall(body)))
                if not c_has:
                    continue
                valnames = names.get(value, set())
                if any(re.search(r"\b" + n + r"\b", body) for n in valnames):
                    continue
                if re.search(r"0x0*" + hm.group(1), body, re.I):
                    continue
                findings.append(
                    (
                        str(rs.relative_to(REPO)) if rs.is_relative_to(REPO) else str(rs),
                        lineno,
                        fn,
                        f"0x{value:08X} ({'/'.join(sorted(valnames)) or 'unnamed'})",
                        ", ".join(c_has) or "(none)",
                    )
                )
    return findings


def scan_crate(crate: Path, names: dict[int, set[str]]) -> list[tuple[str, int, str, str, str]]:
    c_dirs = c_dirs_for(crate.name)
    if not c_dirs:
        return []
    return scan_files(sorted(source_files(crate / "src")), c_dirs, names)


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--crate", action="append", help="crate dir name, e.g. ole32-rust")
    ap.add_argument("--lint", action="store_true", help="terse output; exit 1 on findings")
    args = ap.parse_args()

    names = hresult_names()
    crates = (
        [RUST_ROOT / c for c in args.crate]
        if args.crate
        else sorted(p for p in RUST_ROOT.iterdir() if p.is_dir() and (p / "src").is_dir())
    )
    findings = []
    for crate in crates:
        findings += scan_crate(crate, names)

    for path, lineno, fn, lit, c_has in findings:
        if args.lint:
            print(f"{path}:{lineno}: {fn}: {lit} not returned by the C function")
        else:
            print(f"MISMATCH {path}:{lineno} {fn}: {lit}\n    C function uses: {c_has}")
    if not args.lint:
        print(f"\n{len(findings)} finding(s)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
