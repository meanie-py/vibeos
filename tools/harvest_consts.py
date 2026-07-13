#!/usr/bin/env python3
"""harvest_consts.py — resolve C constant names to their values from the SDK headers and
emit Rust `pub const` declarations.

WHY THIS EXISTS: a constant transcribed from memory produces NO compile error when it is
wrong — it just silently makes the code do something else. (The ole32 ring had exactly
this: a missing CF_ENHMETAFILE turned a `match` arm into a catch-all *binding*, so every
clipboard format took the enhanced-metafile path.) So constants are never hand-typed here;
they are looked up in the headers that the C itself compiles against, and the emitted Rust
cites the file:line it came from.

Sources searched, in order:
  1. sdk/include/**            — the checked-in PSDK/DDK headers
  2. build-<arch>/sdk/include/ — the widl-GENERATED headers (objidl.h, oleidl.h, ...),
                                where interface enums like TYMED/DVASPECT/STGC actually
                                live; they are not in the checked-in tree.

Recognized shapes:
  #define NAME 0x1234        /  #define NAME 1234  /  #define NAME (0x12 | 0x34)
  enum { NAME = 0x10, ... }  /  enum { NAME, NEXT, ... }   (implicit C numbering)

Usage:
  tools/harvest_consts.py CF_ENHMETAFILE GENERIC_READ ...        # names on argv
  tools/harvest_consts.py --from-errors errs.txt                 # scrape rustc E0425s
  tools/harvest_consts.py --type DWORD ...                       # emitted Rust type
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


def header_files() -> list[Path]:
    out: list[Path] = []
    for root in [REPO / "sdk" / "include"] + sorted(REPO.glob("build-*/sdk/include")):
        if root.is_dir():
            out += sorted(root.rglob("*.h"))
    return out


def find_define(name: str, files: list[Path]) -> tuple[str, str] | None:
    pat = re.compile(rf"^\s*#\s*define\s+{re.escape(name)}\s+(\(?[^/\r\n]+?)\s*(?:/\*.*)?$")
    for f in files:
        try:
            for i, line in enumerate(f.read_text(errors="ignore").splitlines(), 1):
                m = pat.match(line)
                if m:
                    val = m.group(1).strip()
                    if val and not val.startswith(("__", "DECLSPEC")):
                        return val, f"{f.relative_to(REPO)}:{i}"
        except OSError:
            pass
    return None


def find_enum_member(name: str, files: list[Path]) -> tuple[str, str] | None:
    """Resolve NAME inside a C enum, honouring implicit numbering."""
    for f in files:
        try:
            text = f.read_text(errors="ignore")
        except OSError:
            continue
        if re.search(rf"\b{re.escape(name)}\b", text) is None:
            continue
        for m in re.finditer(r"enum\s+\w*\s*\{(.*?)\}", text, re.S):
            body = m.group(1)
            if not re.search(rf"\b{re.escape(name)}\b\s*(=|,|$)", body):
                continue
            nxt = 0
            for member in body.split(","):
                member = re.sub(r"/\*.*?\*/", "", member, flags=re.S).strip()
                if not member:
                    continue
                if "=" in member:
                    mname, mval = (x.strip() for x in member.split("=", 1))
                    try:
                        nxt = int(mval, 0)
                        val = mval
                    except ValueError:
                        val, nxt = mval, None
                else:
                    mname = member
                    if nxt is None:
                        break
                    val = str(nxt)
                if mname == name:
                    line = text[: m.start()].count("\n") + 1
                    return val, f"{f.relative_to(REPO)}:~{line} (enum)"
                if nxt is not None:
                    nxt += 1
    return None


SCALARS = (
    "DWORD|ULONG|ULONGLONG|UINT|USHORT|WORD|BYTE|LONG|SHORT|INT|CHAR|"
    "unsigned long|unsigned int|unsigned|long|int|short|char"
)


def to_rust(val: str) -> str:
    """C constant expression -> Rust.

    Not just literals: real headers define constants in terms of other constants
    (`#define UPDFCACHE_ALLBUTNODATACACHE (UPDFCACHE_ALL & (DWORD)~UPDFCACHE_NODATACACHE)`).
    Emitting that verbatim yields Rust that does not even PARSE — and a parse error in this
    module is catastrophic rather than local: the whole `win_consts` module fails, every
    constant it exports disappears, and ~800 unrelated E0425s erupt across the crate. So
    translate the C operators rather than passing them through.
    """
    v = val.strip()
    while v.startswith("(") and v.endswith(")") and v.count("(") == v.count(")"):
        inner = v[1:-1].strip()
        if inner.count("(") == inner.count(")"):
            v = inner
        else:
            break
    # integer-literal suffixes: 0x10UL -> 0x10
    v = re.sub(r"\b(0x[0-9a-fA-F]+|\d+)[uUlL]+\b", r"\1", v)
    # C casts are noise here — every emitted const is already typed by its declaration.
    v = re.sub(rf"\(\s*(?:{SCALARS})\s*\)\s*", "", v)
    # C bitwise-NOT is `~`; Rust's is `!`.
    v = v.replace("~", "!")
    return v.strip()


def referenced_names(val: str) -> set[str]:
    """Uppercase identifiers a constant's value depends on — harvest those too, or the
    emitted Rust references a name that doesn't exist."""
    return {n for n in re.findall(r"\b[A-Z][A-Z0-9_]{2,}\b", val)}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("names", nargs="*")
    ap.add_argument("--from-errors", help="rustc log; scrape `cannot find value \\`X\\``")
    ap.add_argument("--type", default="DWORD", help="Rust type for the emitted consts")
    args = ap.parse_args()

    names = list(args.names)
    if args.from_errors:
        log = Path(args.from_errors).read_text()
        names += re.findall(r"cannot find value `([A-Z][A-Z0-9_]+)`", log)
    names = sorted(set(names))
    if not names:
        print("no names given", file=sys.stderr)
        return 2

    files = header_files()
    print(f"// harvested by tools/harvest_consts.py from {len(files)} SDK headers", file=sys.stderr)

    # Resolve transitively: a value may name other constants, and those must be emitted too.
    resolved: dict[str, tuple[str, str]] = {}
    missing = []
    queue = list(names)
    seen = set(queue)
    while queue:
        n = queue.pop(0)
        hit = find_define(n, files) or find_enum_member(n, files)
        if not hit:
            missing.append(n)
            continue
        val, where = hit
        rust = to_rust(val)
        resolved[n] = (rust, where)
        for dep in referenced_names(rust):
            if dep not in seen:
                seen.add(dep)
                queue.append(dep)

    for n in sorted(resolved):
        rust, where = resolved[n]
        print(f"/// {where}")
        print(f"pub const {n}: {args.type} = {rust};")

    if missing:
        print("\nNOT FOUND in any header (resolve by hand, citing a source):",
              file=sys.stderr)
        for n in missing:
            print(f"  {n}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
