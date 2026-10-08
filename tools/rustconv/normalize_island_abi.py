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

# Crates that are ALSO compiled for a 32-bit HOST target by a difftest harness
# (`i686-unknown-linux-gnu`). They must keep their literal `stdcall`, because the whole premise
# of this pass — that `system` IS stdcall on x86 — is false off Windows: `"system"` is stdcall
# only on 32-bit WINDOWS, while an explicit `extern "stdcall"` is honoured on x86 whatever the
# OS. `target_arch = "x86"` is true for the host harness too, so normalizing here silently
# turns the harness's Win32 imports cdecl, leaves four bytes on the stack per call, and the
# following `ret` jumps to a stack-resident value: SIGSEGV with the PC inside the stack, no
# symbol, nowhere near the cause. That is what took msvcrt-seh-difftest down
# (`cxx_frame_handler` faulting) with all three product arches perfectly green.
#
# See the field manual, "On a 32-bit HOST harness, `extern "system"` and `extern "stdcall"`
# stop agreeing". Keep in sync with the difftest harnesses' path dependencies.
HOST_HARNESS_CRATES = frozenset(
    {"msvcrt-crt-rust", "msvcrt-sys", "cnt-rust", "cnt-ctype", "rtl-strings"}
)

# The stem regex above says which files COULD be an island; this says which ones ARE. Both
# halves are needed and they close different holes. Naming alone once meant an allowlist of
# stems ("types", "bindgen") that silently skipped every island generated under another name:
# oleaut32's `win_sys_<arch>.rs` (bindgen), ntkrnl-sys's
# `hal_bindgen_<arch>.rs`, shell32's `island_<arch>.rs`, oleaut32's `commacros_<arch>.rs`. A
# skipped island is invisible rather than loud — the slot simply stays literal and the
# mismatch surfaces later as an E0308 in a consumer, which is how oleaut32's PROPSHEETPAGEW
# `pfnDlgProc` survived a repo-wide normalization run.
#
# Keying on the generation banner instead is self-maintaining: a new generator gets covered
# the day it lands, and a hand-written per-arch file (ke_<arch>.rs, seh_<arch>.rs) is left
# alone because it carries no banner.
_GENERATED = re.compile(
    r"generated by rust-bindgen|@generated|GENERATED by|Auto-generated", re.I
)

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

# `pfnContinue: ::core::option::Option<unsafe extern "stdcall" fn(..)>` — a callback passed as
# a PARAMETER of another callback. Same shape as a struct field but with no `pub`, so `_FIELD`
# never saw it, and normalizing the enclosing slot to `system` while this stayed literal makes
# the two signatures differ in one buried argument. IViewObject::Draw's `pfnContinue` is the
# live case: the vtable slot became `system` and DataCache_Draw, whose parameter is `system`
# too, then failed to match the island's half-normalized type.
#
# Keyed by the parameter's own name, projected from i386 exactly like every other slot — the
# name is identical across the three islands because bindgen writes the C parameter name.
_PARAM = re.compile(
    r"(?<!pub )(?P<pname>\w+): (?:::core::option::)?Option<\s*"
    r'unsafe extern "(?P<pabi>\w+)" fn'
)

# `pub field: SomeAlias,` — a struct field typed by a BARE alias name (no `*`, no generics).
# Paired with the enclosing `pub struct`, this maps a slot key onto the alias that carries
# its ABI on this island.
_ALIAS_FIELD = re.compile(r"pub struct (?P<s>\w+)|pub (?P<fname>\w+): (?P<ty>[A-Za-z_]\w*),")

# `unsafe extern "X" { pub fn NAME(` — a function DECLARATION, not a fn-pointer type.
#
# These matter for the same reason the pointer types do, and were missed because they are a
# different shape. When a crate DEFINES a callback that its own island also declares (the
# translator emits `#[no_mangle] extern "system" fn`, the island imports the C prototype),
# the call site resolves to the ISLAND's declaration — so a `stdcall` declaration is what
# rustc sees, and it will not unify with a normalized slot no matter how the definition is
# spelled. afd's four TDI completions and smss's SmpApiLoop failed exactly here.
#
# bindgen emits ONE function per block (verified: 0 of 1,339 blocks across afd's island and
# ntdll-sys carry more than one), so rewriting the block header is a per-symbol edit and no
# block ever has to be split.
_DECL = re.compile(
    r'unsafe extern "(?P<dabi>\w+)" \{\s*\n(?:\s*#\[[^\]]*\]\s*\n)*\s*pub fn (?P<dname>\w+)'
)


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
        # An explicit --stem names the files a generation run just wrote, banner or not.
        # Without one, the banner is what separates a generated island from a hand-written
        # per-arch module that merely shares the naming convention (ke_<arch>.rs,
        # seh_<arch>.rs). The stem regex alone would happily rewrite those.
        if stem is None:
            with p.open(encoding="utf-8", errors="replace") as fh:
                head = "".join(next(fh, "") for _ in range(8))
            if not _GENERATED.search(head):
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
    for m in _PARAM.finditer(text):
        found.setdefault(f"param {m.group('pname')}", m.group("pabi"))
    # Declarations live in their own key space (`fn NAME`) so they can never collide with an
    # alias or a `<struct>::<field>` of the same name.
    for m in _DECL.finditer(text):
        found[f"fn {m.group('dname')}"] = m.group("dabi")
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


# `pub type PKIPI_BROADCAST_WORKER = KIPI_BROADCAST_WORKER;` — one alias naming another.
_ALIAS_CHAIN = re.compile(r"^pub type (?P<src>\w+) = (?P<dst>\w+);$", re.M)


def alias_chains(text: str) -> dict[str, str]:
    """alias -> the alias it forwards to, for bare `pub type A = B;` lines.

    The SAME asymmetry the struct-field projection exists for, one level further out. On i386
    bindgen inlines the fn-pointer into `PKIPI_BROADCAST_WORKER`; on the 64-bit pair that name
    is a bare forward to `KIPI_BROADCAST_WORKER`, which is where the ABI actually lives. The
    i386 witness is keyed on the pointer alias, finds a non-matching bare alias here, and stops
    — so `KIPI_BROADCAST_WORKER` kept `extern "C"` while its own i386 twin said `system`, and
    pcix's `KeIpiGenericCall(Some(PciExecuteCriticalSystemRoutine), ..)` stopped type-checking
    on amd64 only. Following the chain puts the witness where it belongs.
    """
    return {m["src"]: m["dst"] for m in _ALIAS_CHAIN.finditer(text)}


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

    def sub_decl(m: re.Match) -> str:
        nonlocal n
        if f"fn {m.group('dname')}" not in keys or m.group("dabi") != from_abi:
            return m.group(0)
        n += 1
        return m.group(0).replace(f'extern "{from_abi}" {{', 'extern "system" {', 1)

    text = _DECL.sub(sub_decl, text)

    def sub_param(m: re.Match) -> str:
        nonlocal n
        if f"param {m.group('pname')}" not in keys or m.group("pabi") != from_abi:
            return m.group(0)
        n += 1
        return m.group(0).replace(f'extern "{from_abi}" fn', 'extern "system" fn', 1)

    text = _PARAM.sub(sub_param, text)
    return text, n


def normalize_crate(
    crate_src: pathlib.Path, apply: bool, stem: str | None = None
) -> tuple[int, list[str]]:
    # `crate_src` is <crate>/src for a real crate; name the crate from its parent.
    crate_name = (crate_src.parent if crate_src.name == "src" else crate_src).name
    if crate_name in HOST_HARNESS_CRATES:
        return 0, [
            f"    {crate_name}: SKIPPED — compiled for a 32-bit HOST target by a difftest "
            f"harness, where `system` is cdecl and an explicit `stdcall` is not"
        ]
    total, notes = 0, []
    # Per stem, never pooled: the witness set is only valid inside the one bindgen run that
    # produced it (see island_paths).
    for islands in island_paths(crate_src, stem).values():
        if not islands["i386"]:
            continue  # no i386 twin -> no witness -> nothing is knowable about this stem
        # The witness: everything this stem's i386 island records as stdcall — INCLUDING what
        # a previous run already rewrote to `system`, because on i386 `system` IS stdcall and
        # so carries the same evidence. Without that this pass is only idempotent in the
        # trivial sense: the second run finds no witness left on i386 and silently stops
        # projecting onto 64-bit entries it had not reached yet, which is how ntkrnl-sys's
        # KIPI_BROADCAST_WORKER stayed `extern "C"` on amd64 while its i386 twin said
        # `system`. `C` on i386 is still excluded — that is the genuinely-cdecl case and
        # rewriting it would be a real ABI break.
        stdcall: set[str] = set()
        for p in islands["i386"]:
            for key, abi in scan(p.read_text(errors="replace")).items():
                if abi in ("stdcall", "system"):
                    stdcall.add(key)
        if not stdcall:
            continue
        for arch in ARCHES:
            frm = "stdcall" if arch == "i386" else "C"
            for p in islands[arch]:
                text = p.read_text(errors="replace")
                # Project the witness through this island's aliases: a slot i386 inlined as
                # stdcall may be a NAMED alias here (C function TYPES take that shape), and
                # the two spellings share no key of their own.
                keys = stdcall | {
                    ty for slot, ty in alias_fields(text).items() if slot in stdcall
                }
                # Follow bare `pub type A = B;` forwards so a witness keyed on A reaches
                # whichever alias in the chain actually carries the fn pointer. Transitive,
                # with a visited set — a self-referential or mutually-referential pair would
                # otherwise spin.
                chain = alias_chains(text)
                frontier, seen = set(keys), set(keys)
                while frontier:
                    nxt = {chain[k] for k in frontier if k in chain} - seen
                    seen |= nxt
                    frontier = nxt
                keys = seen
                new, n = rewrite(text, keys, frm)
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
        elif notes:
            # A crate deliberately left alone still says so. A silent skip is indistinguishable
            # from "nothing to do", which is exactly how a missing island stays missing.
            print("\n".join(notes))
    verb = "normalized" if args.apply else "would normalize"
    print(f"\n{verb} {grand} callback slot(s)")
    if args.apply:
        widen_crates(repo, crates, args.stem)
    return 0


def widen_crates(repo: pathlib.Path, crates: list[pathlib.Path], stem: str | None) -> None:
    """bindgen drops the `L` of `#define X L"..."`: put the UTF-16 back (widen_island_constants).

    It rides on this pass because this is the one post-step EVERY island generator already
    runs; a fix applied by only some generators is one the next regeneration of any other
    crate silently undoes (ntdll-sys, win32k-sys and six more had shipped narrow constants
    after gen_driver_types.sh alone learned to widen).
    """
    import widen_island_constants as widen

    wide = widen.wide_defines(repo)
    for c in crates:
        src = c / "src" if (c / "src").is_dir() else c
        files = sorted(src.glob(f"{stem}_*.rs" if stem else "*bindgen_*.rs"))
        for path in files:
            text = path.read_text()
            new, widened, refused = widen.widen(text, wide)
            if new != text:
                path.write_text(new)
            if widened or refused:
                note = f", refused: {', '.join(refused)}" if refused else ""
                print(f"widened {len(widened)} L\"..\" constant(s) in {path.name}{note}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
