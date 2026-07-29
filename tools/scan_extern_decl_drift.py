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

WHAT IT DOES NOT CATCH (yet)
----------------------------
A WIDTH difference at the SAME indirection depth. `*mut u64` against `*mut VBO`
(ULONG32) is one level of pointer on both sides, so the fastfat
FatGetNextMcbEntry bug that motivated this tool would NOT be reported by it —
that one was found while fixing compile errors, and is fixed. Adding width
resolution (the BASE_WIDTH table in scan_type_alias_contradictions.py is the
machinery) is owed, and is the more valuable half: a silent 4-vs-8 store is
harder to spot by eye than a missing `*`.

WHAT IT COMPARES, AND WHY ONLY THAT
-----------------------------------
ARITY, and pointer INDIRECTION DEPTH per parameter. Not type names: `PEPROCESS`,
`ntkrnl_sys::ddk::PEPROCESS` and `*mut _EPROCESS` are the same thing and
enumerating that equivalence is a rabbit hole. Depth is the property that
actually breaks at runtime, and it is decidable.

Getting there took four rounds, each a defect in THIS tool that manufactured
noise rather than finding bugs — the count went 2682 -> 113 -> 39 -> 10:

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
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools" / "rustconv"))
import tree_sitter_rust as tsr  # noqa: E402
from tree_sitter import Language, Parser  # noqa: E402

_LANG = Language(tsr.language())
_PARSER = Parser(_LANG)
RUST_WS = REPO / "sdk" / "lib" / "rust"

_IDENT_ONLY = re.compile(r"[A-Za-z_]\w*")
_LINKNAME = re.compile(r'(?:link_name|export_name)\s*=\s*"([^"]+)"')
_PATH_QUAL = re.compile(r"\b(?:[A-Za-z_]\w*::)+([A-Za-z_]\w*)")
_PTR_HEAD = re.compile(r"^(?:\*\s*(?:mut|const)|&\s*(?:mut\s+)?)\s*(.*)$", re.S)
_MAX_ALIAS_HOPS = 12


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


def depth_of(t: str, aliases: dict[str, set[str]], stack: tuple = ()) -> int | None:
    """Pointer depth after resolving alias chains; None when undecidable."""
    t = t.strip()
    d = 0
    while True:
        m = _PTR_HEAD.match(t)
        if not m:
            break
        d += 1
        t = m.group(1).strip()
    if not _IDENT_ONLY.fullmatch(t):
        return d  # array / fn-pointer / primitive: the depth we counted is it
    if t in stack or len(stack) > _MAX_ALIAS_HOPS:
        return None
    rhs = aliases.get(t)
    if not rhs:
        return d
    depths = set()
    for r in rhs:
        sub = depth_of(r, aliases, (*stack, t))
        if sub is None:
            return None
        depths.add(sub)
    if len(depths) != 1:
        return None  # the same name means different depths somewhere: SKIP, never guess
    return d + depths.pop()


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


def alias_map(root_dir: Path) -> dict[str, set[str]]:
    """name -> every RHS it is given anywhere under root_dir."""
    seen: dict[str, set[str]] = defaultdict(set)
    for rf in sorted(root_dir.rglob("*.rs")):
        if "/target/" in str(rf):
            continue
        src = rf.read_bytes()

        def walk(n, src=src):
            if n.type == "type_item":
                nm = n.child_by_field_name("name")
                ty = n.child_by_field_name("type")
                if nm is not None and ty is not None:
                    seen[_txt(nm, src)].add(norm(_txt(ty, src)))
            # `use ntkrnl_sys::ddk::PKSPIN_LOCK as DdkPKSPIN_LOCK;` is an alias too.
            # Missing it left the rename unresolvable at depth 0 and reported every
            # use of it as drift against the very type it renames.
            if n.type == "use_as_clause":
                path, alias = n.child_by_field_name("path"), n.child_by_field_name("alias")
                if path is not None and alias is not None:
                    seen[_txt(alias, src)].add(norm(_txt(path, src)))
            for c in n.children:
                walk(c)

        walk(_PARSER.parse(src).root_node)
    return seen


def collect(src_dir: Path):
    """(defs, decls, called-idents-per-file) for one crate."""
    defs: dict[str, tuple] = {}
    decls: dict[str, list] = defaultdict(list)
    called: dict[Path, set[str]] = defaultdict(set)
    for rf in sorted(src_dir.rglob("*.rs")):
        src = rf.read_bytes()
        lines = src.decode(errors="replace").splitlines()
        root = _PARSER.parse(src).root_node

        def note_calls(n, src=src, rf=rf):
            if n.type == "call_expression":
                fn = n.child_by_field_name("function")
                if fn is not None and fn.type == "identifier":
                    called[rf].add(_txt(fn, src))
            for c in n.children:
                note_calls(c)

        note_calls(root)

        def walk(n, in_foreign=False, src=src, rf=rf, lines=lines):
            if n.type == "function_signature_item" and in_foreign:
                nm = n.child_by_field_name("name")
                if nm is not None:
                    ln = n.start_point[0]
                    if not (ln < len(lines) and _nolint(lines[ln])):
                        ident = _txt(nm, src)
                        decls[_symbol(n, src, ident)].append((rf, ln + 1, _sig(n, src), ident))
            if n.type == "function_item":
                prev, attrs = n.prev_sibling, []
                while prev is not None and prev.type == "attribute_item":
                    attrs.append(_txt(prev, src))
                    prev = prev.prev_sibling
                if any("no_mangle" in a or "export_name" in a for a in attrs):
                    nm = n.child_by_field_name("name")
                    if nm is not None:
                        defs[_symbol(n, src, _txt(nm, src))] = (rf, _sig(n, src))
            for c in n.children:
                walk(c, in_foreign or n.type == "foreign_mod_item")

        walk(root)
    return defs, decls, called


def scan(root_dir: Path, crate_dirs: list[Path]) -> list[tuple[Path, int, str]]:
    aliases = alias_map(root_dir)
    findings: list[tuple[Path, int, str]] = []
    for src_dir in crate_dirs:
        defs, decls, called = collect(src_dir)
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
                    cd, dd = depth_of(ct, aliases), depth_of(dt, aliases)
                    if cd is None or dd is None or cd == dd:
                        continue
                    findings.append(
                        (
                            cfile,
                            cline,
                            f"{sym}: param {i} decl `{ct}` (*{cd}) vs def `{dt}` "
                            f"(*{dd}) ({dfile.name})",
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
