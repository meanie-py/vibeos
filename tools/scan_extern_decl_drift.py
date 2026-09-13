#!/usr/bin/env python3
"""scan_extern_decl_drift.py — an `extern` DECLARATION that contradicts the
`#[no_mangle]` DEFINITION of the same symbol, in the same workspace.

WHY THIS EXISTS
---------------
rustc compares a call against the DECLARATION in scope. It never compares that
declaration against the DEFINITION, because the definition lives in another
module (or another crate) and the two are only ever joined by the LINKER, which
checks nothing but the name. So this is a bug class no compiler in the pipeline
can see:

    // allocsup.rs
    extern "system" { fn FatGetNextMcbEntry(.., Vbo: *mut u64, ..); }
    // fsctrl.rs
    #[no_mangle] pub unsafe extern "system" fn FatGetNextMcbEntry(.., Vbo: *mut u32, ..)

Both files compile. Both are internally consistent. The caller hands over a
4-byte object and the callee stores 8 bytes into it. Real instances, all from a
translation writing one file's idea of a prototype:

  * fastfat FatUnpinRepinnedBcbs — volinfo declared `*mut PIRP_CONTEXT` against
    cachesup's `*mut IRP_CONTEXT`, so the callee read a pointer-to-pointer as
    the struct itself.
  * freeldr TuiDisplayMenu — the decl carried an 11th parameter the definition
    (and tui.h) do not have, so every argument from MenuItemList on was passed
    one slot early and the callee read a u8 as its item-list pointer.
  * fastfat FatLocateDirent — param 8 declared `*mut *mut DIRENT` where the
    definition (and the C) say `PBOOLEAN`. Latent only because the single call
    site passes NULL.

Neither verify_signatures nor verify_c_signatures covers it: both compare Rust
against a C prototype, so a symbol with no C prototype left — which is every
fully-translated module — has no oracle at all.

WHAT IT COMPARES, AND WHY ONLY THAT
-----------------------------------
ARITY, pointer INDIRECTION DEPTH per parameter, and the WIDTH of the base type
underneath. Not type names: `PEPROCESS`, `ntkrnl_sys::ddk::PEPROCESS` and
`*mut _EPROCESS` are the same thing and enumerating that equivalence is a
rabbit hole. Depth and width are what actually break at runtime, and both are
decidable from the alias graph.

Depth catches a missing `*`. Width catches the 4-vs-8 store you cannot see by
eye — `*mut u64` against `*mut VBO` is one pointer on both sides, and that IS
the FatGetNextMcbEntry bug that motivated this tool.

By-value NARROWING is deliberately not reported. A callee reading fewer bits
than the caller passed is well-defined on all three of our ABIs — the argument
occupies a whole register and the callee reads a subset of bits somebody set —
and it is how `BOOL` against `BOOLEAN` is spelled tree-wide. The reported
direction is the callee consuming bits nobody wrote.

Getting here took EIGHT rounds, every one a defect in THIS tool that
manufactured noise or lost coverage rather than finding bugs. The count went
2682 -> 113 -> 39 -> 10, and then, once width landed, 138 -> 97:

  * An alias spelled differently across bindgen islands (`*mut BOOLEAN` vs
    `*mut c_uchar`) is NOT ambiguous — only a disagreement in resolved DEPTH is.
    Dropping on textual difference buried the real findings under 2,670 fake
    ones about types that agree with themselves.
  * A path-qualified alias (`ntkrnl_sys::ddk::PEPROCESS`) must reduce to its
    last segment before lookup, or it never resolves and every use is a finding.
  * `#[link_name = "X"] fn Y()` declares X. Pairing by the Rust identifier
    compares two unrelated functions (shell32's Shell32RustCoCreateInitSF vs
    SHELL32_CoCreateInitSF). Same lesson scan_cc_mismatch already learned.
  * Only a decl that is CALLED governs an ABI. A decl that exists to take an
    ADDRESS is deliberately not the function's signature: ntoskrnl's SSDT
    declares every syscall `fn(_: usize, ..)` to count stack slots, and
    kernel32's thread-start thunks are invoked by the kernel, not through the
    decl.
  * An identifier that is neither an alias nor an aggregate we have SEEN is
    UNKNOWN, not depth 0. Generated bindgen islands live in build-<arch>/ and
    are outside this sweep.
  * `pub type c_void = core::ffi::c_void;` is a re-export; normalised it reads
    `c_void -> c_void`, a cycle, which resolves to undecidable — silently
    taking PVOID and most of the tree's types with it. Failure mode: silence.
  * A function pointer is ONE indirection however it is spelled, and
    `Option<T>` is T. Otherwise every callback declared `*mut c_void` reads as
    drift against the typedef naming that very pointer.
  * A pointer spelled as a pointer-WIDTH integer (`isize`, `ULONG_PTR`) is the
    same register holding the same address. `u32` against `HWND` is not, and
    still reports.

`&mut T` counts as one level of indirection: it is a pointer at the ABI.

Findings gate on NEW-vs-baseline (tools/lint-baseline/scan_extern_decl_drift.txt).
Suppress a verified false positive with a trailing:
    fn Foo(..);  // nolint: decl-drift (reason)

Usage:
    python3 tools/scan_extern_decl_drift.py            # sweep the workspace
    python3 tools/scan_extern_decl_drift.py --lint     # terse (baseline gate)
    python3 tools/scan_extern_decl_drift.py <dir>      # a fixture tree
Exit 1 if any finding.
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.lint_facts import source_facts
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools" / "rustconv"))
import tree_sitter_rust as tsr  # noqa: E402
from tree_sitter import Language, Parser, Query, QueryCursor  # noqa: E402

_LANG = Language(tsr.language())
_PARSER = Parser(_LANG)
_FACT_NODES = Query(
    _LANG,
    """[
    (call_expression) (function_signature_item) (function_item)
    (type_item) (use_as_clause) (struct_item) (enum_item) (union_item)
] @fact""",
)
RUST_WS = REPO / "sdk" / "lib" / "rust"

_IDENT_ONLY = re.compile(r"[A-Za-z_]\w*")
_LINKNAME = re.compile(r'(?:link_name|export_name)\s*=\s*"([^"]+)"')
_PATH_QUAL = re.compile(r"\b(?:[A-Za-z_]\w*::)+([A-Za-z_]\w*)")
_PTR_HEAD = re.compile(r"^(?:\*\s*(?:mut|const)|&\s*(?:mut\s+)?)\s*(.*)$", re.S)
_FN_PTR = re.compile(r"\bfn\s*\(")
_OPTION = re.compile(r"^(?:::)?(?:core::)?(?:option::)?Option\s*<(.*)>$", re.S)
_MAX_ALIAS_HOPS = 12

# Widths, for the SECOND half of the comparison: a `*mut u64` and a `*mut ULONG32`
# are both one pointer, and the callee still stores four bytes past the caller's
# object. Depth catches a missing `*`; width catches the silent 4-vs-8 store,
# which is the one you cannot see by eye. Shared spellings with
# scan_type_alias_contradictions.py, deliberately duplicated rather than imported:
# that tool resolves per-arch bindgen islands and this one merges them.
BASE_WIDTH = {
    "u8": 1,
    "i8": 1,
    "bool": 1,
    "u16": 2,
    "i16": 2,
    "u32": 4,
    "i32": 4,
    "f32": 4,
    "char": 4,
    "u64": 8,
    "i64": 8,
    "f64": 8,
    "u128": 16,
    "i128": 16,
    "c_char": 1,
    "c_uchar": 1,
    "c_schar": 1,
    "c_short": 2,
    "c_ushort": 2,
    "c_int": 4,
    "c_uint": 4,
    "c_long": 4,
    "c_ulong": 4,
    "c_longlong": 8,
    "c_ulonglong": 8,
    # Windows / DDK spellings
    "UCHAR": 1,
    "CHAR": 1,
    "BYTE": 1,
    "BOOLEAN": 1,
    "CCHAR": 1,
    "USHORT": 2,
    "SHORT": 2,
    "WORD": 2,
    "WCHAR": 2,
    "LANGID": 2,
    "ULONG": 4,
    "LONG": 4,
    "DWORD": 4,
    "ULONG32": 4,
    "LONG32": 4,
    "UINT": 4,
    "INT": 4,
    "NTSTATUS": 4,
    "HRESULT": 4,
    "BOOL": 4,
    "ULONGLONG": 8,
    "LONGLONG": 8,
    "ULONG64": 8,
    "LONG64": 8,
    "DWORDLONG": 8,
    "DWORD64": 8,
    "LARGE_INTEGER": 8,
    "ULARGE_INTEGER": 8,
}
# Pointer-width. Not an integer, because it IS 4 on i386 and 8 on amd64/arm64 —
# so `ULONG` against `ULONG_PTR` is a real bug on two of our three arches and
# correct on the third. Reported, with the arch dependence named in the message.
PTR = "ptr"
PTR_NAMES = {
    "usize",
    "isize",
    "ULONG_PTR",
    "LONG_PTR",
    "DWORD_PTR",
    "SIZE_T",
    "SSIZE_T",
    "UINT_PTR",
    "INT_PTR",
    "KAFFINITY",
}


# Terminal spellings that are neither an aggregate we parse nor a width we know.
_TERMINAL_EXTRA = {"c_void", "void"}
_CYCLE = object()  # memo marker: this name is being resolved further up the stack


@dataclass(frozen=True)
class Types:
    aliases: dict[str, set[str]]
    aggregates: frozenset[str]
    # resolve() fans out over every RHS a name is ever given, and names like
    # PVOID have dozens across the bindgen islands, so the unmemoised walk is
    # exponential in chain length — the sweep went from 0.2s to not finishing.
    cache: dict = field(default_factory=dict)


def _txt(n, src: bytes) -> str:
    return src[n.start_byte : n.end_byte].decode(errors="replace")


def _nolint(line_text: str) -> bool:
    return bool(
        re.search(r"//\s*nolint(?::\s*decl-drift\b|\s*$|\s*\()", line_text)
        or re.search(r"//\s*nolint\b(?!\s*:)", line_text)
    )


def norm(t: str) -> str:
    t = re.sub(r"\s+", " ", t).strip()
    return _PATH_QUAL.sub(r"\1", t)


def _strip_ptrs(t: str) -> tuple[int, str]:
    """(levels of indirection stripped, the base spelling underneath)

    `Option<T>` unwraps to T. A nullable callback is spelled `Option<DLGPROC>`
    on one side and `DLGPROC` on the other all over this tree, and both are one
    pointer — the niche optimisation is exactly why the Option is free. Leaving
    the wrapper on made the two sides differ by a level and reported every
    property-sheet and menu-filter callback as drift.
    """
    t = t.strip()
    d = 0
    while True:
        m = _PTR_HEAD.match(t)
        if m:
            d += 1
            t = m.group(1).strip()
            continue
        m = _OPTION.match(t)
        if m:
            t = m.group(1).strip()
            continue
        return d, t


def _terminal(t: str, types: Types) -> bool:
    """True when `t` is a name whose depth-0-ness we actually KNOW."""
    return t in types.aggregates or t in BASE_WIDTH or t in PTR_NAMES or t in _TERMINAL_EXTRA


def resolve(t: str, types: Types, stack: tuple = ()) -> tuple[int | None, int | str | None]:
    """(pointer depth, byte width of the base) with alias chains chased.

    The two halves are decided INDEPENDENTLY, and either may come back None.
    Coupling them costs real findings: `PCSTR` is `*const i8` in one bindgen
    island and `*const CHAR` in another, which is one answer about depth and two
    about spelling — demanding the base agree before trusting the depth silently
    dropped NtLdrGetOptionExN, FatLocateDirent and HidParser_GetCollectionDescription,
    all three of them genuine. None means undecidable, and undecidable means
    SKIP: an answer we had to guess at is worse than no answer, because it
    becomes a finding somebody has to triage.
    """
    d, t = _strip_ptrs(t)
    if not _IDENT_ONLY.fullmatch(t):
        # A function pointer IS one level of indirection, however it is spelled
        # (`fn(..)`, `Option<unsafe extern "C" fn(..)>`). Counting it as depth 0
        # reported every callback parameter declared `*mut c_void` — DLGPROC,
        # DRAWSTATEPROC, PFNFDINOTIFY, PDLL_INIT_ROUTINE — as drift against the
        # typedef that names the very same pointer.
        return (d + 1, PTR) if _FN_PTR.search(t) else (d, None)
    if t in BASE_WIDTH:
        return d, BASE_WIDTH[t]
    if t in PTR_NAMES:
        return d, PTR
    if t in stack or len(stack) > _MAX_ALIAS_HOPS:
        return None, None
    hit = types.cache.get(t)
    if hit is not None:
        return (None, None) if hit == _CYCLE else (None if hit[0] is None else d + hit[0], hit[1])
    if t in types.aggregates and t in types.aliases:
        # The alias map is workspace-global, but `use X as Y` is scoped to ONE
        # module. ramdisk's `use PDEVICE_OBJECT as _PDEVOBJ` redefined the name
        # of a win32k-sys bindgen STRUCT, so win32k's PPDEVOBJ resolved one
        # level too deep and both MouseSafetyOnDraw* were reported as drift
        # against the type they already agree with. A name that is both an
        # aggregate and an alias is ambiguous here: SKIP, never guess.
        return None, None
    rhs = types.aliases.get(t)
    if not rhs:
        # An identifier that is neither an alias nor a struct/enum/union we have
        # SEEN is not depth 0 — it is UNKNOWN. Generated bindgen islands live in
        # build-<arch>/ and are outside this sweep, so mscms' `PPROFILEHEADER`
        # (== PROFILEHEADER*) read as a bare struct and its use was reported as
        # drift against the very type it names. Guessing depth 0 for an
        # unresolvable name manufactures a finding per use; skipping it loses
        # only what we could not have decided anyway.
        return (d if _terminal(t, types) else None), None
    types.cache[t] = _CYCLE
    depths, widths = set(), set()
    for r in rhs:
        sd, sw = resolve(r, types, (*stack, t))
        depths.add(sd)
        widths.add(sw)
    own = (
        depths.pop() if len(depths) == 1 else None,
        widths.pop() if len(widths) == 1 else None,
    )
    types.cache[t] = own
    return (None if own[0] is None else d + own[0], own[1])


def _bytes(w: int | str) -> int:
    """PTR as its widest, which is the arch where a mismatch against it bites."""
    return 8 if w == PTR else w


def _ptr_as_integer(cd: int, cw, dd: int, dw) -> bool:
    """One side spells a pointer as a pointer-width INTEGER: same ABI, not drift.

    `co_UserSetCapture(hwnd: isize)` against a definition taking `HWND` is one
    register holding one address either way. Passing handles as `isize` /
    `ULONG_PTR` is idiomatic across this tree, and reporting it drowns the
    version that does break — `co_IntGetPeekMessage` declaring an `HWND`
    parameter `u32`, which discards the top half of a handle on both 64-bit
    arches. That one survives this rule, because `u32` is not pointer-width.
    """
    shallow_w = cw if cd < dd else dw
    return abs(cd - dd) == 1 and shallow_w == PTR


def _reads_undefined(decl_w: int | str, def_w: int | str) -> bool:
    """True when the definition consumes more bits than the declaration passes."""
    return _bytes(def_w) > _bytes(decl_w)


def depth_of(t: str, types: Types) -> int | None:
    return resolve(t, types)[0]


def width_of(t: str, types: Types) -> int | str | None:
    """Byte width of the base type under any pointers; PTR when pointer-sized.

    None when undecidable — an aggregate (we compare depth, never layout), an
    unresolved name, or a name meaning two widths in different islands.
    """
    return resolve(t, types)[1]


def _symbol(n, src: bytes, ident: str) -> str:
    """The symbol a decl/def binds. #[link_name]/#[export_name] wins over the ident."""
    a = n.prev_sibling
    while a is not None and a.type == "attribute_item":
        m = _LINKNAME.search(_txt(a, src))
        if m:
            return m.group(1)
        a = a.prev_sibling
    for c in n.children:
        if c.type == "attribute_item":
            m = _LINKNAME.search(_txt(c, src))
            if m:
                return m.group(1)
    return ident


def _sig(n, src: bytes) -> tuple[list[str], bool]:
    """([param type texts], is_variadic)"""
    ps = n.child_by_field_name("parameters")
    params: list[str] = []
    variadic = False
    if ps is not None:
        for c in ps.children:
            if c.type == "parameter":
                t = c.child_by_field_name("type")
                if t is not None:
                    params.append(norm(_txt(t, src)))
                else:
                    variadic = True  # `self`/unnamed: do not compare positionally
            elif c.type in ("variadic_parameter", "..."):
                variadic = True
    return params, variadic


def _owning_crate(rf: Path, crates: frozenset[Path]) -> Path | None:
    for p in rf.parents:
        if p in crates:
            return p
    return None


def _in_foreign(node):
    parent = node.parent
    while parent is not None:
        if parent.type == "foreign_mod_item":
            return True
        parent = parent.parent
    return False


@source_facts()
def file_facts(src: bytes, include_functions: bool):
    seen = defaultdict(set)
    aggregates = set()
    defs, decls, called = {}, defaultdict(list), set()
    lines = src.decode(errors="replace").splitlines() if include_functions else []
    tree = _PARSER.parse(src)
    nodes = QueryCursor(_FACT_NODES).captures(tree.root_node).get("fact", [])
    for n in sorted(nodes, key=lambda node: node.start_byte):
        if include_functions:
            if n.type == "call_expression":
                fn = n.child_by_field_name("function")
                if fn is not None and fn.type == "identifier":
                    called.add(_txt(fn, src))
            if n.type == "function_signature_item" and _in_foreign(n):
                nm = n.child_by_field_name("name")
                if nm is not None:
                    ln = n.start_point[0]
                    if not (ln < len(lines) and _nolint(lines[ln])):
                        ident = _txt(nm, src)
                        decls[_symbol(n, src, ident)].append((ln + 1, _sig(n, src), ident))
            if n.type == "function_item":
                prev, attrs = n.prev_sibling, []
                while prev is not None and prev.type == "attribute_item":
                    attrs.append(_txt(prev, src))
                    prev = prev.prev_sibling
                if any("no_mangle" in a or "export_name" in a for a in attrs):
                    nm = n.child_by_field_name("name")
                    if nm is not None:
                        defs[_symbol(n, src, _txt(nm, src))] = _sig(n, src)
        if n.type == "type_item":
            nm = n.child_by_field_name("name")
            ty = n.child_by_field_name("type")
            if nm is not None and ty is not None:
                name, rhs = _txt(nm, src), norm(_txt(ty, src))
                # `pub type c_void = core::ffi::c_void;` is a RE-EXPORT, and
                # norm() strips the path, so it lands here as `c_void ->
                # c_void`. Recorded as an alias it is a cycle, and a cycle
                # resolves to None — which silently made `*mut c_void`
                # undecidable, and with it PVOID and every one of the ~60
                # types spelled through it. Coverage vanishes with no error.
                if rhs != name:
                    seen[name].add(rhs)
        # `use ntkrnl_sys::ddk::PKSPIN_LOCK as DdkPKSPIN_LOCK;` is an alias too.
        # Missing it left the rename unresolvable at depth 0 and reported every
        # use of it as drift against the very type it renames.
        if n.type == "use_as_clause":
            path, alias = n.child_by_field_name("path"), n.child_by_field_name("alias")
            if path is not None and alias is not None:
                seen[_txt(alias, src)].add(norm(_txt(path, src)))
        if n.type in ("struct_item", "enum_item", "union_item"):
            nm = n.child_by_field_name("name")
            if nm is not None:
                aggregates.add(_txt(nm, src))
    return dict(seen), aggregates, defs, dict(decls), called


def sweep(root_dir: Path, crate_dirs: list[Path]):
    """Rebuild global relations from content-addressed per-file facts."""
    seen = defaultdict(set)
    aggregates = set()
    crates = frozenset(crate_dirs)
    per = {c: ({}, defaultdict(list), defaultdict(set)) for c in crate_dirs}
    for rf in sorted(source_files(root_dir)):
        crate = _owning_crate(rf, crates)
        aliases, structs, definitions, declarations, calls = file_facts(
            rf.read_bytes(), crate is not None
        )
        for name, values in aliases.items():
            seen[name].update(values)
        aggregates.update(structs)
        if crate is not None:
            defs, decls, called = per[crate]
            for name, sig in definitions.items():
                defs[name] = (rf, sig)
            for name, sites in declarations.items():
                decls[name].extend((rf, *site) for site in sites)
            called[rf].update(calls)
    return Types(aliases=seen, aggregates=frozenset(aggregates)), per


def scan(root_dir: Path, crate_dirs: list[Path]) -> list[tuple[Path, int, str]]:
    types, per = sweep(root_dir, crate_dirs)
    findings: list[tuple[Path, int, str]] = []
    for src_dir in crate_dirs:
        defs, decls, called = per[src_dir]
        for sym, (dfile, (dparams, dvar)) in defs.items():
            for cfile, cline, (cparams, cvar), ident in decls.get(sym, []):
                if cvar or dvar:
                    continue
                # Only a CALLED decl governs a real ABI; an address-taken decl
                # (SSDT slot counts, kernel-invoked thunks) deliberately differs.
                if ident not in called.get(cfile, ()):
                    continue
                if len(cparams) != len(dparams):
                    findings.append(
                        (
                            cfile,
                            cline,
                            f"{sym}: ARITY decl {len(cparams)} vs def {len(dparams)} "
                            f"({dfile.name})",
                        )
                    )
                    continue
                for i, (ct, dt) in enumerate(zip(cparams, dparams, strict=True)):
                    (cd, cw), (dd, dw) = resolve(ct, types), resolve(dt, types)
                    if cd is None or dd is None:
                        continue
                    if cd != dd and _ptr_as_integer(cd, cw, dd, dw):
                        continue
                    if cd != dd:
                        findings.append(
                            (
                                cfile,
                                cline,
                                f"{sym}: param {i} decl `{ct}` (*{cd}) vs def `{dt}` "
                                f"(*{dd}) ({dfile.name})",
                            )
                        )
                        continue
                    # Same depth, different width: the store that overruns the
                    # caller's object without any type looking wrong.
                    if cw is None or dw is None or cw == dw:
                        continue
                    if not cd and not _reads_undefined(cw, dw):
                        # A BY-VALUE parameter the definition reads NARROWER than
                        # the declaration passes is well-defined on all three of
                        # our ABIs: the argument occupies a whole register and the
                        # callee reads a subset of bits the caller definitely set.
                        # That is how `BOOL` against `BOOLEAN` is spelled across
                        # the tree, and reporting it buries the direction that
                        # actually breaks — the callee reading bits nobody wrote.
                        continue
                    findings.append(
                        (
                            cfile,
                            cline,
                            f"{sym}: param {i} WIDTH decl `{ct}` ({cw}) vs def `{dt}` "
                            f"({dw}) {'pointee' if cd else 'value'}"
                            f"{' [pointer-sized: i386 differs]' if PTR in (cw, dw) else ''} "
                            f"({dfile.name})",
                        )
                    )
    return sorted(findings, key=lambda f: (str(f[0]), f[1], f[2]))


def main() -> int:
    lint = "--lint" in sys.argv[1:]
    args = [a for a in sys.argv[1:] if a != "--lint"]
    if args:
        root = Path(args[0]).resolve()
        crate_dirs = [root]
    else:
        root = RUST_WS
        crate_dirs = [c / "src" for c in sorted(RUST_WS.glob("*-rust")) if (c / "src").is_dir()]

    findings = scan(root, crate_dirs)
    for f, line, msg in findings:
        try:
            shown = f.relative_to(REPO)
        except ValueError:
            shown = f
        print(f"{shown}:{line}: {msg}")
    # A TALLY is presentation, not a finding. Under --lint the gate diffs stdout
    # against the baseline line by line, so a count that moves reads as "1 new"
    # and turns the gate red with nothing wrong — which is exactly what this line
    # did on its first run. Every sibling scanner suppresses it the same way.
    if not lint:
        print(f"\n{len(findings)} extern-decl drift finding(s)")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
