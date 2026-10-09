#!/usr/bin/env python3
"""
scan_call_literal_vs_c.py — a call passes a number where its own C passes a named constant.

    C (kept in the file):  CmpCmdHiveOpen(SourceFile, ..., CM_CHECK_REGISTRY_PURGE_VOLATILES);
    Rust:                  CmpCmdHiveOpen(SourceFile, ..., 0x00000001u32);   // the value is 2

The model replaced the name with a number it remembered, and the number was wrong: bit 0 is
no check flag at all, so CmCheckRegistry answered INVALID_PARAMETER and every NtLoadKey of a
hive file failed REGISTRY_CORRUPT. Setup could never connect its hives (the install smoke).
Nothing compiles differently, and the call returns an ordinary error status.

The oracle is the C the translator left at the bottom of the same file ("ORIGINAL C
SOURCE"). For each Rust call with a bare integer at argument i, every same-arity call of
that function in the C must pass a named constant there: an ALL_CAPS macro, or an enum
member (`HalRebootRoutine`, `KeyValuePartialInformation`). Its value comes from the bindgen
islands first -- `<EnumType>_<Member>` as bindgen writes it, the C header's own transcript --
and only then from the crate's `const` items, which can themselves be invented (fastfat had
`MmFlushForWrite = 2`; it is 1). A literal that differs is reported (MISMATCH). A macro
nothing defines is reported UNRESOLVED -- the literal may be right, but nothing in the tree
says so; name it.

The enum half found po's shutdown handling every reboot as a power-off (POWER_ACTION 5/6
swapped, so setup's reboot halted), ZwQueryValueKey asked for KeyValueFullInformation by five
callers that parse the PARTIAL layout, a fastfat wait in UserMode on a stack event, and the
balance manager waiting WaitAll for WaitAny.

Usage:
    python3 tools/scan_call_literal_vs_c.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_call_literal_vs_c.py --lint
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_wide_byte_copy import SKIP_FILE_RE
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
MARKER = "ORIGINAL C SOURCE"
NOLINT_RE = re.compile(r"nolint:\s*call-literal-vs-c\b")
CALL_RE = re.compile(r"(?<![\w.:!])([A-Za-z_]\w*)\s*\(")  # Rust: `foo!(` is a macro
C_CALL_RE = re.compile(r"(?<![\w.])([A-Za-z_]\w*)\s*\(")  # C: `!foo(` is a call
INT_RE = re.compile(
    r"^\(?\s*(-?)\s*(0x[0-9a-fA-F_]+|\d[\d_]*)(?:[iu](?:8|16|32|64|size))?\s*\)?"
    r"(?:\s+as\s+[\w:]+)*$"
)
C_CONST_RE = re.compile(r"^\(?\s*(?:\([A-Z_]+\)\s*)?([A-Za-z][A-Za-z0-9_]{3,})\s*\)?$")
ALL_CAPS_RE = re.compile(r"[A-Z][A-Z0-9_]{3,}")
ISLAND_ENUM_RE = re.compile(r"^pub const (\w+): (\w+) = (-?\d+);", re.M)
ISLAND_GLOB = "*-sys/src/*bindgen_*.rs"
NOT_CONSTANTS = {"TRUE", "FALSE", "NULL", "VOID", "ULONG", "USHORT", "UCHAR", "BOOLEAN"}
CONST_RE = re.compile(
    r"\bconst\s+([A-Za-z_]\w*)\s*:\s*[\w:]+\s*=\s*\(?\s*(-?)\s*(0x[0-9a-fA-F_]+|\d[\d_]*)"
    r"(?:[iu](?:8|16|32|64|128|size))?\s*\)?\s*(?:as\s+[\w:]+\s*)?;"
)
KEYWORDS = {"if", "while", "for", "match", "return", "switch", "sizeof", "fn", "loop"}


def _args(text: str, open_paren: int) -> list[str] | None:
    depth, cur, out = 0, [], []
    for ch in text[open_paren:]:
        if ch in "([{":
            depth += 1
            if depth == 1:
                continue
        elif ch in ")]}":
            depth -= 1
            if depth == 0:
                out.append("".join(cur).strip())
                if out and out[-1] == "":
                    out.pop()  # Rust's trailing comma, or `f()`
                return out
        elif ch == "," and depth == 1:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(ch)
    return None


def _calls(text: str, call_re: re.Pattern = CALL_RE):
    for m in call_re.finditer(text):
        name = m.group(1)
        if name in KEYWORDS:
            continue
        args = _args(text, m.end() - 1)
        if args is not None:
            yield m.start(), name, args


_ISLAND: dict[str, set[int]] | None = None


def _island_enums() -> dict[str, set[int]]:
    """Enum member -> its values across every bindgen island (a member is
    `pub const <EnumType>_<Member>: <EnumType> = N;`). Two values means two enums share the
    member name: ambiguous, so the caller ignores it."""
    global _ISLAND
    if _ISLAND is None:
        out: dict[str, set[int]] = defaultdict(set)
        for f in sorted(RUST_ROOT.glob(ISLAND_GLOB)):
            for m in ISLAND_ENUM_RE.finditer(f.read_text(errors="replace")):
                name, enum_type, val = m.groups()
                if name.startswith(enum_type + "_"):
                    out[name[len(enum_type) + 1 :]].add(int(val))
        _ISLAND = dict(out)
    return _ISLAND


def _crate_of(path: Path) -> Path:
    for parent in path.parents:
        if (parent / "Cargo.toml").is_file():
            return parent
    return path.parent


def _crate_consts(crate: Path, cache: dict) -> dict[str, set[int]]:
    if crate not in cache:
        values: dict[str, set[int]] = defaultdict(set)
        for f in source_files(crate):
            if f.suffix != ".rs":
                continue
            try:
                t = f.read_text(errors="replace")
            except OSError:
                continue
            for m in CONST_RE.finditer(t):
                v = int(m.group(3).replace("_", ""), 0)
                values[m.group(1)].add(-v if m.group(2) else v)
        cache[crate] = values
    return cache[crate]


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    findings = []
    consts_cache: dict = {}
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
            c_text = "\n".join(re.sub(r"^\s*//\s?", "", ln) for ln in text[cut:].splitlines())
            c_by_name: dict[tuple[str, int], list[list[str]]] = defaultdict(list)
            for _pos, name, args in _calls(c_text, C_CALL_RE):
                c_by_name[(name, len(args))].append(args)
            rust = text[:cut]
            code = re.sub(
                r"//[^\n]*|/\*.*?\*/", lambda m: re.sub(r"[^\n]", " ", m.group(0)), rust, flags=re.S
            )
            lines = rust.splitlines()
            for pos, name, args in _calls(code):
                c_calls = c_by_name.get((name, len(args)))
                if not c_calls:
                    continue
                for i, arg in enumerate(args):
                    lm = INT_RE.match(arg)
                    if not lm:
                        continue
                    names = set()
                    for ca in c_calls:
                        cm = C_CONST_RE.match(ca[i])
                        if not cm or cm.group(1) in NOT_CONSTANTS:
                            names = set()
                            break
                        names.add(cm.group(1))
                    if len(names) != 1:
                        continue
                    cname = next(iter(names))
                    lit = int(lm.group(2).replace("_", ""), 0) * (-1 if lm.group(1) else 1)
                    island = _island_enums().get(cname)
                    if island is not None and len(island) == 1:
                        known = island  # the header's own value outranks a hand copy
                    elif ALL_CAPS_RE.fullmatch(cname):
                        known = _crate_consts(_crate_of(f), consts_cache).get(cname)
                    else:
                        continue  # a camel-case name with no single island value: a variable
                    if known and lit in known:
                        continue
                    lineno = rust.count("\n", 0, pos) + 1
                    if NOLINT_RE.search(lines[lineno - 1]):
                        continue
                    kind = "MISMATCH" if known else "UNRESOLVED"
                    want = f"{cname}={sorted(known)}" if known else cname
                    findings.append(
                        (f, lineno, f"{kind} {name} arg{i}: {arg} where C passes {want}")
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
        tag = "call-literal-vs-c" if lint else "literal where the C names a constant:"
        print(f"{rel}:{lineno}: {tag} {what}")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): pass the C's constant by name.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
