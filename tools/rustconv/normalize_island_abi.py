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

...with ONE asymmetry, which is why the witness is also projected THROUGH aliases. A C
function TYPE (`typedef NTSTATUS (NTAPI SpInitializeFn)(...)` — no star, ntsecpkg.h's whole
SECPKG_FUNCTION_TABLE surface is written this way) is emitted by bindgen as a NAMED alias on
amd64/arm64 but INLINED into the struct field on i386. The two spellings then share no key,
so the i386 stdcall witness could not reach the 64-bit alias and 31 of msv1_0's SECPKG slots
stayed `extern "C"` while the translation spelled them `system`. Slot identity is the struct
FIELD, not the spelling: when `<struct>::<field>` is a stdcall witness and this island types
that field by a bare alias name, the alias is rewritten too.

Idempotent: entries already spelled `system` are left alone, so it is safe to re-run and
safe to wire into generation.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

ARCHES = ("i386", "amd64", "arm64")
ISLAND_STEMS = ("types", "bindgen")

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

# `pub field: SomeAlias,` — a struct field typed by a BARE alias name (no `*`, no generics).
# Paired with the enclosing `pub struct`, this maps a slot key onto the alias that carries
# its ABI on this island.
_ALIAS_FIELD = re.compile(r"pub struct (?P<s>\w+)|pub (?P<fname>\w+): (?P<ty>[A-Za-z_]\w*),")


def island_paths(crate_src: pathlib.Path, stem: str | None = None) -> dict[str, list[pathlib.Path]]:
    """arch -> the crate's island files for that arch.

    `stem` restricts the match to exactly `<stem>_<arch>.rs`. gen_driver_types.sh generates
    ONE stem per invocation (`TYPES_STEM`, several islands per crate — netapi32 carries six),
    so a generation run must normalize the three files it just wrote and nothing else.
    Without this the pass would also rewrite a sibling stem that this run never regenerated,
    which is a tool mutating output it does not own."""
    pattern = (
        rf"{re.escape(stem)}_({'|'.join(ARCHES)})\.rs$"
        if stem
        else rf"(?:{'|'.join(ISLAND_STEMS)})\w*_({'|'.join(ARCHES)})\.rs$"
    )
    out: dict[str, list[pathlib.Path]] = {a: [] for a in ARCHES}
    for p in sorted(crate_src.rglob("*.rs")):
        m = re.match(pattern, p.name)
        if m:
            out[m.group(1)].append(p)
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


def alias_fields(text: str) -> dict[str, str]:
    """`<struct>::<field>` -> the bare alias name that types that field on this island."""
    out: dict[str, str] = {}
    struct = ""
    for m in _ALIAS_FIELD.finditer(text):
        if m.group("s"):
            struct = m.group("s")
        else:
            out[f"{struct}::{m.group('fname')}"] = m.group("ty")
    return out


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
    islands = island_paths(crate_src, stem)
    if not islands["i386"]:
        return 0, []
    # The witness: everything the i386 island records as stdcall.
    stdcall: set[str] = set()
    for p in islands["i386"]:
        for key, abi in scan(p.read_text(errors="replace")).items():
            if abi == "stdcall":
                stdcall.add(key)
    if not stdcall:
        return 0, []
    total, notes = 0, []
    for arch in ARCHES:
        frm = "stdcall" if arch == "i386" else "C"
        for p in islands[arch]:
            text = p.read_text(errors="replace")
            # Project the witness through this island's aliases: a slot i386 inlined as
            # stdcall may be a NAMED alias here (C function TYPES take that shape), and the
            # two spellings share no key of their own.
            keys = stdcall | {
                ty for slot, ty in alias_fields(text).items() if slot in stdcall
            }
            new, n = rewrite(text, keys, frm)
            if n and apply:
                p.write_text(new)
            if n:
                total += n
                notes.append(f"    {p.relative_to(crate_src.parent.parent)}: {n} ({frm} -> system)")
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
