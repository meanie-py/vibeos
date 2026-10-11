"""Index C flexible arrays the way C does: no bounds check.

bindgen spells a trailing `T Arr[ANYSIZE_ARRAY]` / `T Arr[1]` as `[T; 1]`, so a translated
`(*data).MountPoints[i]` bounds-checks and PANICS for every i >= 1 while the C deliberately
over-allocates. kernel32's FindNextVolumeW did that on the second mount point, so every
diskpart -- the C one too -- died at startup; setuplib's partition list did it on a disk's
second layout entry (both found by tools/payloads/modrun, 2026-10-07).

A field is flexible when some bindgen struct declares it `[T; 1]` as its LAST field. Each
`<place>.Field[i]` with a non-zero index becomes

    (*core::ptr::addr_of!(<place>.Field[0]).add((i) as usize))       // read
    (*core::ptr::addr_of_mut!(<place>.Field[0]).add((i) as usize))   // written, or &mut

`[0]` of a `[T; 1]` cannot panic, so the element type is inferred and no helper is needed.
Matching is by field NAME, so a same-named real array in another struct is rewritten too;
that is still exactly the C's semantics (C checks no bound), it only drops a Rust check the
C never had.

    python -m rustconv.flex_index sdk/lib/rust/kernel32-rust [--check]
"""

from __future__ import annotations

import argparse
import re
import sys
from functools import cache
from pathlib import Path

from .config import REACTOS_ROOT

REPO = REACTOS_ROOT  # the invoked tree, not this file's checkout
RUST = REPO / "sdk" / "lib" / "rust"
_STRUCT = re.compile(r"pub struct (\w+)\s*\{(.*?)\n\}", re.S)
_FIELD = re.compile(r"pub (\w+)\s*:\s*([^,\n]+),")
_FLEX_TYPE = re.compile(r"^\[.+;\s*1(?:usize)?\s*\]$")


@cache
def field_shapes(root: Path = RUST) -> tuple[frozenset[str], frozenset[str]]:
    """(flexible, ambiguous) field names over every bindgen struct under `root`.

    Flexible: some struct ends with it as a `[T; 1]`. Ambiguous: also declared somewhere as
    a longer array -- kernel32's `cStreamName` is `[WCHAR; 296]` in WIN32_FIND_STREAM_DATA
    and lzexpand's `table` a real buffer -- so a name alone cannot say which struct a site
    indexes, and those are reported, not rewritten.
    """
    flex: set[str] = set()
    longer: set[str] = set()
    # Imported here: `tools` is the repo root's package, absent when rustconv is imported on its
    # own (port.py imports this module, and check-tooling runs from tools/rustconv).
    from tools.source_walk import source_files

    for path in source_files(root, "*.rs"):
        text = path.read_text(errors="replace")
        if "pub struct" not in text:
            continue
        for m in _STRUCT.finditer(text):
            fields = _FIELD.findall(m.group(2))
            for i, (name, ty) in enumerate(fields):
                ty = ty.strip()
                if not ty.startswith("["):
                    continue
                if _FLEX_TYPE.match(ty):
                    if i == len(fields) - 1:
                        flex.add(name)
                else:
                    longer.add(name)
    return frozenset(flex - longer), frozenset(flex & longer)


def _place_start(text: str, dot: int) -> int:
    """Start of the postfix place expression that ends just before `.Field` at `dot`."""
    i = dot
    while i > 0:
        c = text[i - 1]
        if c in ")]":
            depth, j = 0, i - 1
            opener = "(" if c == ")" else "["
            while j >= 0:
                if text[j] == c:
                    depth += 1
                elif text[j] == opener:
                    depth -= 1
                    if depth == 0:
                        break
                j -= 1
            i = j
            continue
        if c.isalnum() or c in "_.":
            i -= 1
            continue
        if c == ":" and i >= 2 and text[i - 2] == ":":
            i -= 2
            continue
        # A chain rustfmt broke across lines (`(*R).List[0]\n    .PartialResourceList\n
        # .PartialDescriptors[i]`, pciidex's resource walk): whitespace before a `.` is still
        # inside the place.
        if c.isspace() and i < len(text) and text[i] == ".":
            j = i - 1
            while j > 0 and text[j - 1].isspace():
                j -= 1
            i = j
            continue
        break
    return i


def _index_end(text: str, open_bracket: int) -> int:
    depth = 0
    for j in range(open_bracket, len(text)):
        if text[j] == "[":
            depth += 1
        elif text[j] == "]":
            depth -= 1
            if depth == 0:
                return j
    return -1


def _written(text: str, start: int, end: int) -> bool:
    """Is the place at text[start:end] assigned, or borrowed mutably?"""
    before = text[max(0, start - 40) : start]
    if re.search(r"(&\s*mut\s*|addr_of_mut!\s*\(\s*|&raw\s+mut\s+)$", before):
        return True
    after = text[end : end + 200]
    # `x.F[i] = ..`, `x.F[i].y += ..`, `x.F[i].y.z = ..` (but not `==`)
    return (
        re.match(r"(\s*\.\s*\w+(\s*\[[^\]]*\])?)*\s*([-+*/%&|^]|<<|>>)?=(?!=)", after) is not None
    )


def rewrite(text: str, fields: frozenset[str], *, verified: bool = False) -> tuple[str, int]:
    """Rewrite each `<place>.F[i]` for F in `fields`. `verified`: the form that only
    type-checks when <place>.F really is a `[T; 1]` (for a name that is not decisive)."""
    if not fields:
        return text, 0
    pat = re.compile(r"\.\s*(" + "|".join(sorted(map(re.escape, fields))) + r")\s*\[")
    out, pos, n = [], 0, 0
    for m in pat.finditer(text):
        if m.start() < pos:
            continue
        open_b = m.end() - 1
        close = _index_end(text, open_b)
        if close < 0:
            continue
        index = text[open_b + 1 : close].strip()
        if (
            re.fullmatch(r"\(?\s*0(?:usize|u32|i32)?\s*(?:as\s+\w+)?\s*\)?", index)
            or "\n" in text[m.start() : open_b]
        ):
            continue
        line_start = text.rfind("\n", 0, m.start()) + 1
        if text[line_start : m.start()].lstrip().startswith("//"):
            continue
        start = _place_start(text, m.start())
        place = text[start : m.start()]
        if not place or place.endswith("::"):
            continue
        field = m.group(1)
        mac = "addr_of_mut" if _written(text, start, close + 1) else "addr_of"
        if not re.search(r"\bas\s+usize\s*$", index):
            index = f"({index}) as usize"
        out.append(text[pos:start])
        if verified:
            ptr = "*mut" if mac == "addr_of_mut" else "*const"
            out.append(
                f"(*{{ let flex: {ptr} [_; 1] = core::ptr::{mac}!({place}.{field}); "
                f"core::ptr::{mac}!((*flex)[0]) }}.add({index}))"
            )
        else:
            out.append(f"(*core::ptr::{mac}!({place}.{field}[0]).add({index}))")
        pos = close + 1
        n += 1
    out.append(text[pos:])
    return "".join(out), n


_VERIFIED = re.compile(
    r"\(\*\{ let flex: \*(?:const|mut) \[_; 1\] = core::ptr::addr_of(?:_mut)?!\((?P<arr>.+?)\); "
    r"core::ptr::addr_of(?:_mut)?!\(\(\*flex\)\[0\]\) \}\.add\((?P<idx>.+?)\)\)(?=[^)]|$)"
)


def _unwrap(index: str) -> str:
    """`(i) as usize` -> `i`: undo the cast rewrite() added, when the parens are its own."""
    m = re.fullmatch(r"\((.*)\) as usize", index)
    if m is None:
        return index
    depth = 0
    for c in m.group(1):
        depth += c == "("
        depth -= c == ")"
        if depth < 0:
            return index
    return m.group(1) if depth == 0 else index


def reject(sites: list[str]) -> int:
    """Turn the self-checking sites rustc rejected (`file.rs:LINE[:COL]`, as rustc prints them,
    relative to sdk/lib/rust or the repo) back into plain indexing: those arrays are real."""
    by_file: dict[Path, set[int]] = {}
    for s in sites:
        parts = s.split(":")
        path = next((b / parts[0] for b in (RUST, REPO) if (b / parts[0]).is_file()), None)
        if path is None:
            raise SystemExit(f"flex_index: no such file {parts[0]}")
        by_file.setdefault(path, set()).add(int(parts[1]))
    n = 0
    for path, lines in by_file.items():
        text = path.read_text().split("\n")
        for ln in lines:
            line, k = _VERIFIED.subn(
                lambda m: f"{m.group('arr')}[{_unwrap(m.group('idx'))}]", text[ln - 1]
            )
            text[ln - 1] = line
            n += k
        path.write_text("\n".join(text))
    print(f"flex_index: {n} site(s) returned to plain indexing")
    return n


def run(paths: list[Path], *, check: bool = False, ambiguous_too: bool = False) -> int:
    fields, ambiguous = field_shapes()
    total = 0
    for root in paths:
        from tools.source_walk import source_files

        files = [root] if root.is_file() else sorted(source_files(root, "*.rs"))
        for f in files:
            text = f.read_text()
            new, n = rewrite(text, fields)
            rel = f.relative_to(REPO) if f.is_relative_to(REPO) else f
            new2, maybe = rewrite(new, ambiguous, verified=True)
            if maybe and ambiguous_too:
                new, n = new2, n + maybe
                print(
                    f"flex_index: {rel}: {maybe} name-ambiguous site(s) in the self-checking form "
                    "(a compile error there means a real array: revert that site)"
                )
            elif maybe:
                print(
                    f"flex_index: {rel}: {maybe} index(es) of a name that is flexible in "
                    "one struct "
                    "and a real array in another: --ambiguous rewrites them in a form the "
                    "compiler rejects for a real array"
                )
            if n:
                total += n
                print(f"flex_index: {rel}: {n} site(s)")
                if not check:
                    f.write_text(new)
    return total


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=(__doc__ or "").splitlines()[0])
    ap.add_argument("paths", nargs="*", type=Path)
    ap.add_argument(
        "--check", action="store_true", help="report, do not rewrite; exit 1 on any site"
    )
    ap.add_argument(
        "--ambiguous",
        action="store_true",
        help="also rewrite name-ambiguous sites, in the form only a [T; 1] compiles",
    )
    ap.add_argument(
        "--reject",
        nargs="+",
        metavar="FILE:LINE",
        help="sites rustc rejected: plain indexing again",
    )
    args = ap.parse_args(argv)
    if args.reject:
        reject(args.reject)
        return 0
    n = run([p.resolve() for p in args.paths], check=args.check, ambiguous_too=args.ambiguous)
    print(f"flex_index: {n} site(s) {'found' if args.check else 'rewritten'}")
    return 1 if (args.check and n) else 0


if __name__ == "__main__":
    sys.exit(main())
