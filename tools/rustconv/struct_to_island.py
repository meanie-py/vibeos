#!/usr/bin/env python3
"""Retire a crate's hand-written copy of a struct its bindgen island already renders.

    python3 tools/rustconv/struct_to_island.py <crate src dir> [--dry-run] [--names A,B]

A translated module that predates its island wrote the structs it needed by hand:
shell32 had 240 records -- `MENUITEMINFOW` in types.rs, `WIN32_FIND_DATAW` in ten modules --
each a second Rust type for one C layout. Every island function declaration then rejects
the copy at the call (`expected *mut tagMENUITEMINFOW, found &mut MENUITEMINFOW`), and the
only mechanical bridge, `.cast()`, would compile a copy that disagrees with the header without
a word. The island is generated from the module's real headers, so it is the authority, and
the copy becomes `pub use crate::island::NAME;`.

A copy is replaced only when the swap is a rename: on EVERY arch the island resolves NAME
(through its `pub use` globs and `pub type NAME = tagNAME;` aliases) to a struct with the same
field names in the same order, the copy derives nothing the island's does not, and nothing in
the crate `impl`s it. A `repr` that differs is swapped too and reported as a CORRECTION: it is
the header's packing, which a hand copy drops. Anything else is listed and left for a human -- a
field-list difference is either bindgen's union/bitfield spelling (`__bindgen_anon_1`) or a copy
that is wrong, and telling those apart is reading, not matching. COM interfaces and classes are
never touched: the island blocklists them, `shellcom` owns vtable ABI.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from extern_to_use import GENERATED, module_files, module_path

RUST_ROOT = Path(__file__).resolve().parents[2] / "sdk" / "lib" / "rust"
ARCHES = ("i386", "amd64", "arm64")
STRUCT = re.compile(
    r"^(?P<attrs>(?:[ \t]*#\[[^\n]*\][ \t]*\n)*)"
    r"(?P<vis>pub(?:\([^)]*\))?\s+)?struct\s+(?P<name>\w+)\s*\{",
    re.M,
)
ALIAS = re.compile(r"^pub type (\w+) = (?:[\w:]+::)?(\w+);", re.M)
GLOB = re.compile(r"^pub use (\w+)::(?:\w+::)*\*;", re.M)
FIELD = re.compile(r"\bpub(?:\([^)]*\))?\s+(\w+)\s*:")


def _body_end(text: str, brace: int) -> int:
    """Offset just past the `}` matching the `{` at `brace`."""
    depth = 0
    for i in range(brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i + 1
    raise ValueError("unbalanced struct body")


def _fields(body: str) -> list[str]:
    return FIELD.findall(re.sub(r"//[^\n]*", "", body))


def _repr(attrs: str) -> str:
    m = re.search(r"#\[repr\(([^)]*)\)\]", attrs)
    return ",".join(sorted(p.strip() for p in m.group(1).split(","))) if m else ""


def _derives(attrs: str) -> set[str]:
    return {
        d.strip()
        for m in re.finditer(r"#\[derive\(([^)]*)\)\]", attrs)
        for d in m.group(1).split(",")
    }


class Island:
    """One arch's island: its own file plus the `-sys` crates it globs."""

    def __init__(self, files: list[str]):
        self.structs: dict[str, tuple[str, str]] = {}
        self.alias: dict[str, str] = {}
        for text in files:
            for m in STRUCT.finditer(text):
                brace = m.end() - 1
                self.structs.setdefault(
                    m.group("name"), (m.group("attrs"), text[brace : _body_end(text, brace)])
                )
            for a, b in ALIAS.findall(text):
                self.alias.setdefault(a, b)

    def resolve_name(self, name: str) -> str | None:
        """The struct NAME's aliases end at, or None."""
        for _ in range(6):
            if name in self.structs:
                return name
            if name not in self.alias:
                return None
            name = self.alias[name]
        return None

    def resolve(self, name: str) -> tuple[str, str] | None:
        for _ in range(6):
            if name in self.structs:
                return self.structs[name]
            if name not in self.alias:
                return None
            name = self.alias[name]
        return None


def load_islands(src: Path, module: str) -> dict[str, Island]:
    out = {}
    for arch in ARCHES:
        own = src / f"{module}_{arch}.rs"
        if not own.is_file():
            raise SystemExit(f"struct_to_island: no {own}; is `{module}` this crate's island?")
        text = own.read_text(errors="replace")
        files = [text]
        for crate in GLOB.findall(text):
            if crate in ("crate", "self", "super"):
                continue
            sys_src = RUST_ROOT / crate.replace("_", "-") / "src"
            files += [
                p.read_text(errors="replace")
                for p in (sys_src / f"bindgen_{arch}.rs", sys_src / "bindgen_common.rs")
                if p.is_file()
            ]
        out[arch] = Island(files)
    return out


FIELD_TYPE = re.compile(r"\bpub(?:\([^)]*\))?\s+\w+\s*:\s*([^,]+?),?\s*(?:\n|$)")


def _field_type_names(body: str) -> set[str]:
    return {
        n
        for ty in FIELD_TYPE.findall(re.sub(r"//[^\n]*", "", body))
        for n in re.findall(r"[A-Za-z_]\w*", ty)
    }


def plan(
    src: Path, module: str, names: set[str] | None
) -> tuple[dict[str, str], list[str], list[str]]:
    """({module key: new text}, replaced lines, kept lines)."""
    islands = load_islands(src, module)
    files = module_files(src)
    hand = {k: t for k, t in files.items() if not GENERATED.search(t[:2048])}
    impls = set(
        re.findall(
            r"^\s*(?:unsafe\s+)?impl\b[^{;]*?\b(?:for\s+)?(\w+)\s*(?:<[^{]*>)?\s*\{",
            "\n".join(hand.values()),
            re.M,
        )
    )
    # (key, start, end, name, where, use, note, island field-type names)
    cands = []
    kept: list[str] = []
    local_names: set[str] = set()
    for key, text in hand.items():
        for m in STRUCT.finditer(text):
            name = m.group("name")
            local_names.add(name)
            if names is not None and name not in names:
                continue
            if re.match(r"[IC][A-Z]", name):
                continue
            got = [islands[a].resolve(name) for a in ARCHES]
            if any(g is None for g in got):
                continue
            end = _body_end(text, m.end() - 1)
            body = text[m.end() - 1 : end]
            line = text.count("\n", 0, m.start()) + 1
            where = f"{module_path(src, key).relative_to(src)}:{line}: {name}"
            mine = _fields(body)
            if any(_fields(b) != mine for _, b in got):
                kept.append(
                    f"{where}: fields differ from the island's ({', '.join(_fields(got[1][1]))})"
                )
                continue
            extra = _derives(m.group("attrs")) - set.intersection(*(_derives(a) for a, _ in got))
            if extra:
                kept.append(f"{where}: derives {sorted(extra)} the island does not")
                continue
            if name in impls:
                kept.append(f"{where}: the crate implements it")
                continue
            # A different repr is the header's packing, which the hand copy dropped: shellapi.h
            # is `pshpack1` on 32-bit only, so shell32's unpacked SHQUERYRBINFO was 24 bytes on
            # i386 against the caller's 20, and the exported SHQueryRecycleBinW refused every
            # real caller. Same field names, so the swap takes the header's layout; it is listed.
            mine_repr = _repr(m.group("attrs"))
            theirs = sorted(
                {
                    f"{arch}:{_repr(a) or '-'}"
                    for arch, (a, _) in zip(ARCHES, got, strict=True)
                    if _repr(a) != mine_repr
                }
            )
            note = f"repr({mine_repr or '-'}) -> {', '.join(theirs)}" if theirs else ""
            vis = (m.group("vis") or "").strip()
            use = f"{vis + ' ' if vis else ''}use crate::{module}::{name};"
            ftypes = set().union(*(_field_type_names(b) for _, b in got))
            cands.append((key, m.start(), end, name, where, use, note, ftypes))

    # A swap is a rename only if the island's FIELDS are the types the code already uses. A
    # field whose island type is a bindgen anonymous record (`tagPIDLDATA__bindgen_ty_1`, a
    # `__BindgenUnionField` union) changes every access path; one whose type the crate keeps its
    # own copy of (SHELLDETAILS.str: the island's `_STRRET`, the code's `shellcom::STRRET`, a
    # COM interface) changes what the field IS. Either keeps the copy, and a kept copy keeps
    # every struct that embeds it -- so iterate to a fixpoint.
    def canon(n: str) -> str:
        return next((s for s in (islands["amd64"].resolve_name(n),) if s), n)

    swapped = {c[3] for c in cands}
    while True:
        held = {canon(n) for n in local_names - swapped} | (local_names - swapped)
        drop = set()
        for c in cands:
            if c[3] not in swapped:
                continue
            anon = sorted(n for n in c[7] if "__bindgen_ty" in n or n == "__BindgenUnionField")
            mixed = sorted(n for n in c[7] if n in held or canon(n) in held)
            if anon or mixed:
                drop.add(c[3])
        if not drop:
            break
        swapped -= drop

    changed, done = {}, []
    by_key: dict[str, list] = {}
    for c in cands:
        if c[3] in swapped:
            by_key.setdefault(c[0], []).append(c)
            done.append(f"{c[4]}: {c[6]}" if c[6] else c[4])
        else:
            anon = sorted(n for n in c[7] if "__bindgen_ty" in n or n == "__BindgenUnionField")
            why = (
                f"a field is a bindgen anonymous record ({anon[0]})"
                if anon
                else "a field's type is one the crate keeps its own copy of"
            )
            kept.append(f"{c[4]}: {why}")
    for key, edits in by_key.items():
        text = hand[key]
        for _, start, end, _, _, use, _, _ in sorted(edits, key=lambda e: e[1], reverse=True):
            text = text[:start] + use + text[end:]
        changed[key] = text
    return changed, done, kept


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("src", type=Path, help="the crate's src/ directory")
    ap.add_argument("--island", default="island", help="the island module (island_<arch>.rs)")
    ap.add_argument("--names", help="comma-separated struct names (default: every candidate)")
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args(argv)
    names = set(a.names.split(",")) if a.names else None
    changed, done, kept = plan(a.src, a.island, names)
    for line in done:
        print(f"use  {line}")  # a trailing `repr(..) -> ..` is a layout the copy had wrong
    for line in kept:
        print(f"KEPT {line}")
    print(
        f"{len(done)} struct(s) now re-export the island in {len(changed)} module(s); "
        f"{len(kept)} kept",
        file=sys.stderr,
    )
    if not a.dry_run:
        for key, text in changed.items():
            module_path(a.src, key).write_text(text)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
