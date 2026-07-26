#!/usr/bin/env python3
"""
scan_bugcheck_stubs.py — cohab stubs that are UNIMPLEMENTED **and** REACHABLE.

THE BUG SHAPE:
A cohab driver keeps a `rust_cohab_stub.c` holding C definitions for symbols the
Rust side still references. Some of those bodies are real shims; some are
`STUB_BUGCHECK()` — i.e. `KeBugCheckEx(...)` on entry, standing in for a function
nobody translated yet. That is a deliberate, honest placeholder. What makes it a
bug is the combination:

    a Rust module DECLARES the symbol  +  the stub body bugchecks
    ------------------------------------------------------------
    = a live crash, waiting for whichever code path calls it first

and nothing in the toolchain says so. The Rust compiles (an `extern` block is
just a claim), the link succeeds (the stub satisfies it), and the failure is
deferred to a boot — one stub per boot, each costing a full build+boot cycle to
discover. fastfat cost three such cycles in one night: FatGenerateFileIdFromFcb,
then FatOpenVolume, then the next one.

THE MACRO SUBCLASS (worse, and strictly a translation defect):
If the name is a `#define NAME(...)` in the driver's own headers, then it was
never a function at all. Translation emitted an `extern fn` for a macro, no
symbol could ever exist, and the stub generator "resolved" it into a bugcheck —
a crash wearing the costume of a link fix. There is no scenario where this is
correct: the fix is always to implement the macro inline in Rust and delete the
stub. fastfat shipped three (FatGenerateFileIdFromFcb,
FatGenerateFileIdFromDirentAndOffset, SizeOfPackedEa). Note this is the exact
inverse of the older `[§9.53]` failure, where a stubbed macro became a silent
no-op instead of a loud crash — same root, opposite symptom.

THE ORPHAN SUBCLASS (looks identical, needs the opposite fix):
A declaration is not a call. fastfat carried `extern` decls for
`fat_check_system_security_access` and `fat_supersede_or_overwrite_file` whose
functions were ALREADY translated in the same file under their PascalCase names
(`FatCheckSystemSecurityAccess`, `FatSupersedeOrOverwriteFile`, `#[no_mangle]`).
Nothing called the snake_case names — but the decls alone were enough to make the
stub generator mint two bugcheck bodies. These cannot fire, and the fix is to
delete the decl and the stub, NOT to translate anything. Reporting them the same
as a live handler is actively harmful: it sent a reader off to re-translate a
370-line function that was already sitting in the file. Hence [ORPHAN_DECL].

The three classes, and what each one costs you:
  [MACRO_AS_EXTERN]  always a defect — the symbol can never exist. Implement inline.
  [BUGCHECK_STUB]    reachable: Rust CALLS it. A live crash on that path. Translate.
  [ORPHAN_DECL]      declared, never called. Delete the decl + stub; check first
                     whether it is already translated under another name.

WHY A LINT AND NOT A CHECKLIST:
The intersection is mechanically computable from files already in the tree, so
paying a build+boot per entry to rediscover it is pure waste. Findings here are
a WORK QUEUE, not necessarily breakage: a bugcheck stub on a path nothing
reaches is fine until something reaches it. Baseline gates NEW ones.

WHAT IS NOT A FINDING:
  - a stub whose body is a real implementation (returns a value, forwards a call)
  - a bugcheck stub NO Rust module declares — not linkable from the Rust side, so
    it cannot fire; it stays off the queue until someone wires it up
  - a call that appears only inside a Rust comment. fastfat keeps the entire
    original C body commented beside each translation, so counting those would
    make every dead stub look reachable.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


def rust_root(root: Path) -> Path:
    """Where the Rust crates live. Falls back to the scan root itself so a control
    fixture can drop a .rs anywhere without mirroring sdk/lib/rust/."""
    real = root / "sdk" / "lib" / "rust"
    return real if real.is_dir() else root


# `TYPE NAME(` at the start of a line — the shape every stub definition takes.
DEF_RE = re.compile(r"(?m)^[A-Za-z_][\w \t\*]*?\b([A-Za-z_]\w*)\s*\(")
# Match the deliberate placeholder markers ONLY — `STUB_BUGCHECK()` or a KeBugCheckEx
# carrying the 0xDEAD marker. Do NOT match a bare `KeBugCheckEx(`: a faithful wrapper
# for a bugcheck MACRO is a real implementation, not a stub. fastfat's FatBugCheck
# (`nodetype.h:104`) is exactly that — it forwards its A/B/C to KeBugCheckEx — and a
# bare-call regex flags it as unimplemented, which is a false positive that would send
# someone to "fix" correct code.
BUGCHECK_RE = re.compile(r"STUB_BUGCHECK\s*\(\s*\)|KeBugCheckEx\s*\([^;]*0[xX][dD][eE][aA][dD]")
# Strip // and /* */ so a commented-out stub is not counted as live.
COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)


def bugcheck_stub_names(text: str) -> list[str]:
    """Names whose definition body contains a bugcheck. Found by locating each
    bugcheck and walking back to the nearest preceding definition header, which
    survives the two-line definitions these files use."""
    src = COMMENT_RE.sub(lambda m: " " * len(m.group(0)), text)
    names: list[str] = []
    for hit in BUGCHECK_RE.finditer(src):
        before = src[: hit.start()]
        # Skip the STUB_BUGCHECK macro's OWN definition: it necessarily contains the
        # marker, and the backwards walk below would otherwise attribute it to whatever
        # function happens to sit above the #define (that mis-flagged FatBugCheck).
        line_start = before.rfind("\n") + 1
        logical = before[line_start:]
        while logical.rstrip().endswith("\\"):
            prev = before.rfind("\n", 0, line_start - 1) + 1
            logical = before[prev:]
            line_start = prev
        if logical.lstrip().startswith("#define"):
            continue
        # The macro's own #define is not a stub definition.
        heads = [
            m
            for m in DEF_RE.finditer(before)
            if not before[: m.start()].rstrip().endswith("#define")
        ]
        if heads:
            names.append(heads[-1].group(1))
    # Preserve first-seen order, drop dupes, and drop the STUB_BUGCHECK macro itself.
    out: list[str] = []
    for n in names:
        if n not in out and n != "STUB_BUGCHECK":
            out.append(n)
    return out


def _show(path: Path, base: Path) -> str:
    """Repo-relative when scanning the repo (keeps the baseline stable and readable);
    absolute otherwise, because the control harness identifies a finding by looking
    for its fixture root in the line."""
    return str(path.relative_to(base)) if base == REPO else str(path)


# A Rust comment. Stripped before counting call sites: fastfat keeps the entire
# original C body commented out beside each translation, so an un-stripped scan
# counts those as live calls and every dead stub looks reachable.
RS_COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)


def rust_declared(
    names: set[str], crate_dir: Path | None, fallback: Path, base: Path
) -> tuple[dict[str, str], dict[str, int]]:
    """Map name -> first Rust file declaring `fn <name>(`, and name -> how many times
    Rust actually CALLS it.

    Declaring and calling are different questions and only the second one means the
    stub can fire. Counting only declarations over-reports: fastfat carried two
    `extern` decls (`fat_check_system_security_access`, `fat_supersede_or_overwrite_file`)
    for functions that were ALREADY translated under their PascalCase names in the same
    file, with zero call sites. They are unreachable orphans — the honest fix is to
    delete the decl and the stub, not to translate anything — but a declaration-only
    scan reports them identically to a live FSCTL handler, which sent a reader
    (me) off to translate a 370-line function that was already there.

    Prefer the driver's own crate; fall back to every Rust crate, because these symbols
    are global and a declaration anywhere is enough to make the stub linkable."""
    roots = [crate_dir] if crate_dir and crate_dir.is_dir() else [fallback]
    decl_pats = {n: re.compile(rf"\bfn\s+{re.escape(n)}\s*\(") for n in names}
    use_pats = {n: re.compile(rf"\b{re.escape(n)}\s*\(") for n in names}
    found: dict[str, str] = {}
    calls: dict[str, int] = dict.fromkeys(names, 0)
    for root in roots:
        for rs in root.rglob("*.rs"):
            try:
                text = rs.read_text(errors="replace")
            except OSError:
                continue
            for n, p in decl_pats.items():
                if n not in found and p.search(text):
                    found[n] = _show(rs, base)
            src = RS_COMMENT_RE.sub(" ", text)
            for n, p in use_pats.items():
                for m in p.finditer(src):
                    # `fn NAME(` is the declaration or the definition, not a call.
                    if not re.search(r"\bfn\s+$", src[: m.start()]):
                        calls[n] += 1
    return found, calls


def header_macros(driver_dir: Path, names: set[str]) -> set[str]:
    """Names defined as function-like macros in the driver's own headers."""
    out: set[str] = set()
    for hdr in driver_dir.glob("*.h"):
        try:
            text = hdr.read_text(errors="replace")
        except OSError:
            continue
        for n in names:
            if re.search(rf"(?m)^\s*#define\s+{re.escape(n)}\s*\(", text):
                out.add(n)
    return out


def main() -> int:
    # Optional scan root, so the control fixtures in tools/test_scanner_controls.py
    # can point this at a throwaway tree instead of the whole repo.
    root = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else REPO
    rroot = rust_root(root)
    findings: list[str] = []
    for stub in sorted(root.glob("**/rust_cohab_stub.c")):
        try:
            text = stub.read_text(errors="replace")
        except OSError:
            continue
        names = bugcheck_stub_names(text)
        if not names:
            continue
        driver_dir = stub.parent
        rel = _show(stub, root)
        crate = rroot / f"{driver_dir.name}-rust"
        declared, calls = rust_declared(set(names), crate if crate.is_dir() else None, rroot, root)
        macros = header_macros(driver_dir, set(names))
        for n in names:
            if n not in declared:
                continue  # not declared in Rust at all — not linkable from there
            if n in macros:
                findings.append(
                    f"{rel}: [MACRO_AS_EXTERN] {n} is a #define in this driver's headers, "
                    f"never a function — the extern decl in {declared[n]} can never resolve, "
                    f"so this stub is a guaranteed bugcheck on first call; implement it inline "
                    f"in Rust and delete the stub"
                )
            elif calls[n]:
                findings.append(
                    f"{rel}: [BUGCHECK_STUB] {n} is untranslated (bugchecks on entry) and Rust "
                    f"CALLS it at {calls[n]} site(s) (declared in {declared[n]}) — reachable, so "
                    f"it crashes whenever that path first runs; translate it or prove the path "
                    f"is dead"
                )
            else:
                findings.append(
                    f"{rel}: [ORPHAN_DECL] {n} is declared in {declared[n]} but never called "
                    f"from Rust, so the stub cannot fire. The decl is what MADE the stub exist; "
                    f"check whether the function is already translated under another name before "
                    f"translating anything, then delete the decl and the stub"
                )
    for f in sorted(findings):
        print(f)
    return 0


if __name__ == "__main__":
    sys.exit(main())
