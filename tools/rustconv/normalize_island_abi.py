#!/usr/bin/env python3
"""normalize_island_abi.py — spell ABI-identical callback conventions `extern "system"`
across a crate's three bindgen islands, using the i386 island as the witness.

WHY. Rust does not unify `extern "system" fn` with a literal `extern "stdcall" fn` (i386)
or `extern "C" fn` (amd64/arm64) in a function-POINTER type, even though the ABI is
identical on that arch. Translations spell their callbacks `extern "system"` because that
is the only portable spelling, so every assignment into a bindgen callback slot is an
E0308 — and the historical workaround was to transmute the address:

    macro_rules! cb { ($f:expr) => { core::mem::transmute($f as usize) } }

That transmute erases the ENTIRE type, including the calling convention, which is how
buslogic shipped six SCSI miniport callbacks defined `extern "C"` into `PHW_*` slots the
port driver dereferences as stdcall: on i386 the caller pops nothing and the callee pops
nothing, so 8 bytes leak per SRB dispatch. Nothing could see it — rustc had been told not
to look. Normalizing the slot types lets `Some(f)` be assigned directly, which makes that
same bug a compile error.

THE WITNESS. On amd64/arm64 there is only ONE convention, so bindgen flattens every NTAPI
to `extern "C"` and the decoration is simply gone (repo-wide: 11,809 `stdcall` blocks on
i386, ZERO on either 64-bit twin). The i386 island is therefore the only place that still
records which callbacks are stdcall, and it is what this pass projects onto the twins.

THE RULE, and why it is not a blanket rewrite:

    i386 says stdcall  -> `system` on all three arches   (ABI-identical everywhere)
    i386 says C        -> leave `C` on all three         (genuinely cdecl)
    i386 says fastcall -> leave `fastcall`               (NOT the same ABI)

The pre-existing `CALLBACK_SYSTEM=1` sed in gen_driver_types.sh does
`s/extern "C" fn/extern "system" fn/g` unconditionally, which on **i386** promotes a
genuinely-cdecl callback to stdcall — a real ABI break in the opposite direction. There
are 32 alias + 2 field cdecl fn-pointer types on i386 that must not be touched. On 64-bit
a blanket rewrite would be ABI-safe but would leave those same types spelled `system` on
one arch and `C` on another, so the projection is keyed per symbol either way.

KEYS. bindgen writes type *names* rather than expanded layouts, so an island entry is
textually identical across the three arches apart from the ABI string. Correspondence is
therefore exact on `pub type <NAME> =` for aliases and `<struct>::<field>` for inline
callback slots.

Idempotent: entries already spelled `system` are left alone, so it is safe to re-run and
safe to wire into generation.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

ARCHES = ("i386", "amd64", "arm64")

# An island is `<stem>_<arch>.rs`, and the stem is whatever that crate's `TYPES_STEM` said —
# it is NOT drawn from a fixed vocabulary. Discovering stems from the filenames rather than
# listing them is the whole point: an earlier version hardcoded ("types", "bindgen"), and when
# shell32 landed its COM island as TYPES_STEM=island the tree-wide sweep skipped 342 slots in
# a 44k-line island without a word. A tool whose coverage silently depends on someone
# remembering to extend a literal is a tool that reports a clean number for work it never did.
# `\w+` is greedy and the arch is anchored at the end, so `types_rpc_i386.rs` -> stem
# `types_rpc`. Over-matching a non-island (`size_check_i386.rs`) is harmless: the rewrite is
# keyed on callback slots cross-checked against that same stem's i386 witness, so a file
# carrying none is a no-op. Under-matching is the bug this shape exists to prevent.
_ISLAND_FILE = re.compile(rf"^(?P<stem>\w+)_(?P<arch>{'|'.join(ARCHES)})\.rs$")

# `pub type NAME = ::core::option::Option<unsafe extern "X" fn(...)`  (the `Option<` and
# the `unsafe extern` routinely land on separate lines, hence the tolerant whitespace).
_ALIAS = (
    r"pub type (?P<aname>\w+)\s*=\s*(?:::core::option::)?Option<\s*"
    r'unsafe extern "(?P<aabi>\w+)" fn'
)
# `pub field: ::core::option::Option<unsafe extern "X" fn(...)` inside a struct.
_FIELD = (
    r"pub (?P<fname>\w+):\s*(?:::core::option::)?Option<\s*"
    r'unsafe extern "(?P<fabi>\w+)" fn'
)
# One pass over all three shapes: `pub struct` only updates the enclosing-struct cursor
# that gives inline callback slots a key stable across the three islands. Group names must
# be distinct — Python's re rejects a duplicate name across an alternation.
_ANY = re.compile(r"pub struct (?P<s>\w+)|" + _ALIAS + "|" + _FIELD)


def island_paths(
    crate_src: pathlib.Path, stem: str | None = None
) -> dict[str, dict[str, list[pathlib.Path]]]:
    """stem -> arch -> the crate's island files.

    Grouping by stem is a correctness requirement, not tidiness. A crate can carry several
    islands (netapi32 has `types`, `types_rpc`, `types_browser`) because its TUs have
    mutually-irreconcilable header sets, and each is an INDEPENDENT bindgen run. Pooling them
    into one witness set would let a stdcall entry in one island promote a same-named entry in
    another that bindgen recorded as cdecl — projecting across a boundary where the
    name correspondence this pass relies on does not hold. Within a stem the correspondence is
    exact, because the three arches are the same bindgen invocation over the same headers.

    `stem` restricts the result to exactly that one. gen_driver_types.sh writes ONE stem per
    invocation, so a generation run must normalize the three files it just wrote and nothing
    else — rewriting a sibling stem it never regenerated is a tool mutating output it does not
    own."""
    out: dict[str, dict[str, list[pathlib.Path]]] = {}
    for p in sorted(crate_src.rglob("*.rs")):
        m = _ISLAND_FILE.match(p.name)
        if not m or (stem is not None and m.group("stem") != stem):
            continue
        out.setdefault(m.group("stem"), {a: [] for a in ARCHES})[m.group("arch")].append(p)
    return out


def scan(text: str) -> dict[str, str]:
    """key -> ABI, for every callback slot in one island."""
    found: dict[str, str] = {}
    struct = ""
    for m in _ANY.finditer(text):
        if m.group("s"):
            struct = m.group("s")
        elif m.group("aname"):
            found[m.group("aname")] = m.group("aabi")
        elif m.group("fname"):
            found[f"{struct}::{m.group('fname')}"] = m.group("fabi")
    return found


def rewrite(text: str, keys: set[str], from_abi: str) -> tuple[str, int]:
    """Spell `system` for every slot in `keys` currently written `from_abi`."""
    n = 0
    struct = ""

    def sub(m: re.Match) -> str:
        nonlocal n, struct
        if m.group("s"):
            struct = m.group("s")
            return m.group(0)
        if m.group("aname"):
            key, abi = m.group("aname"), m.group("aabi")
        else:
            key, abi = f"{struct}::{m.group('fname')}", m.group("fabi")
        if key not in keys or abi != from_abi:
            return m.group(0)
        n += 1
        return m.group(0).replace(f'extern "{abi}" fn', 'extern "system" fn')

    text = _ANY.sub(sub, text)
    return text, n


def normalize_crate(
    crate_src: pathlib.Path, apply: bool, stem: str | None = None
) -> tuple[int, list[str]]:
    total, notes = 0, []
    for islands in island_paths(crate_src, stem).values():
        if not islands["i386"]:
            continue  # no i386 twin -> no witness -> nothing is knowable about this stem
        # The witness: everything this stem's i386 island records as stdcall.
        stdcall: set[str] = set()
        for p in islands["i386"]:
            for key, abi in scan(p.read_text(errors="replace")).items():
                if abi == "stdcall":
                    stdcall.add(key)
        if not stdcall:
            continue
        for arch in ARCHES:
            frm = "stdcall" if arch == "i386" else "C"
            for p in islands[arch]:
                text = p.read_text(errors="replace")
                new, n = rewrite(text, stdcall, frm)
                if n and apply:
                    p.write_text(new)
                if n:
                    total += n
                    notes.append(
                        f"    {p.relative_to(crate_src.parent.parent)}: {n} ({frm} -> system)"
                    )
    return total, notes


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("crates", nargs="*", help="crate dirs (default: every sdk/lib/rust/*)")
    ap.add_argument("--apply", action="store_true", help="write (default: report only)")
    ap.add_argument(
        "--stem",
        help="only normalize <stem>_<arch>.rs (gen_driver_types.sh writes one stem per run)",
    )
    args = ap.parse_args(argv)

    repo = pathlib.Path(__file__).resolve().parents[2]
    base = repo / "sdk" / "lib" / "rust"
    crates = [pathlib.Path(c) for c in args.crates] or sorted(
        d for d in base.iterdir() if (d / "src").is_dir()
    )
    grand = 0
    for c in crates:
        src = c / "src" if (c / "src").is_dir() else c
        n, notes = normalize_crate(src, args.apply, args.stem)
        if n:
            print(f"{c.name}: {n}")
            print("\n".join(notes))
            grand += n
    verb = "normalized" if args.apply else "would normalize"
    print(f"\n{verb} {grand} callback slot(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
