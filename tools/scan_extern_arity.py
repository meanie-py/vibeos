#!/usr/bin/env python3
"""
scan_extern_arity.py — a crate forward-declares a function with a DIFFERENT NUMBER
of parameters than the definition it links to.

WHY THIS ONE AND NOT THE REST OF THE CLASH TRIAGE. `clashing_extern_declarations`
covers ~1,458 findings, and the 2026-07-17 triage sorted them: pointer-name and
const/mut differences are ABI-identical noise, scalar width differences are mostly
benign return-register width, and calling-convention differences are an i386-only
hazard that needs the C prototype to adjudicate. But a different ARG COUNT is none
of those — it is the caller and the callee disagreeing about the shape of the call,
which no ABI makes safe. The triage called it "the highest-value needles". This
scanner isolates exactly that bucket, from source, for free.

It found cdrom's `PORT_ALLOCATED_SENSE`: declared and CALLED with two arguments
(autorun.rs:973) against a one-parameter shim, so the callee bound `Srb` to the
DeviceExtension. Harmless only because that shim ignores its parameter.

WHAT IS NOT A FINDING (each of these was in the first run's output, and none is a bug):
  * an address-only forward decl — `fn BaseThreadStartup();` exists so kernel32 can
    write `BaseThreadStartup as ULONG_PTR` into a new thread's Rip. It is never
    called through, so its arity is not a claim about any call.
  * a uniform syscall-table shape — ntoskrnl's SSDT spells every entry
    `(_: usize, ...)` on purpose.
  * a variadic C decl against a fixed-arity Rust shim (`...` in either signature).
  * a declaration in a file no `mod` chain reaches (see scan_orphan_modules) — dead
    code cannot make a wrong call. freeldr's parked directui.rs is the case.
The first two cannot be told apart from a real bug by signature alone, so they are
BASELINED with a per-symbol reason rather than guessed at.

PARSING NOTES, both of which produced wrong answers first:
  * count against COMMENT-MASKED text. `RaiseOnError: u8, // BOOLEAN` and
    `Flag: u8, /* BOOLEAN -> 1 */` contribute their own commas and `>`s otherwise.
  * neutralise `->` BEFORE depth-tracking `<`/`>`. `Option<unsafe extern "system"
    fn(usize) -> usize>` drives depth negative at the arrow, after which every
    top-level comma is invisible; that alone reported 3,790 phantom findings.
"""

import argparse
import os
import re
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "rustconv"))
from rustconv.crate_repair import _mask_lexical, _match_forward

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from lint_facts import source_facts
from rust_module_tree import crate_live_files, crate_root_of

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_ROOT = os.path.join(REPO, "sdk", "lib", "rust")

# Matched against MASKED bytes, where _mask_lexical blanks the ABI string literal
# INCLUDING its quotes (`"C"` -> three spaces). Match `extern <no brace> {`.
RE_EXTERN_BLOCK = re.compile(rb"(?:unsafe\s+)?extern\s+[^{;()]*\{")
RE_DECL = re.compile(rb"\bfn\s+(\w+)\s*\(")
RE_DEF = re.compile(
    rb"#\[no_mangle\]\s*(?:#\[[^\]]*\]\s*)*pub\s+(?:unsafe\s+)?"
    rb'(?:extern\s+"[A-Za-z-]+"\s+)?fn\s+(\w+)\s*\('
)
# What a declaration/definition actually LINKS to, when it is not the Rust identifier.
# Matched against raw bytes, not `masked` — _mask_lexical blanks string literals, so the
# symbol is only readable in the original. Anchored at the end of the text preceding `fn`.
RE_LINK_NAME = re.compile(rb'#\s*\[\s*link_name\s*=\s*"([^"]+)"\s*\]\s*(?:pub\s+)?(?:unsafe\s+)?$')
RE_EXPORT_NAME = re.compile(rb'#\s*\[\s*export_name\s*=\s*"([^"]+)"\s*\]')


def _symbol(region: bytes, rust_name: bytes, attr_re) -> str:
    """The symbol a decl/def resolves to: an explicit link_name/export_name, else the name.

    Comparing a declaration against a definition that happens to share its RUST identifier is
    wrong whenever the declaration redirects: shell32's five folder classes declare a 5-param
    `SHELL32_CoCreateInitSF` carrying `#[link_name = "Shell32RustCoCreateInitSF"]`, which is a
    WINAPI C++ trampoline — not the 6-param `SHELL32_CoCreateInitSF` that shlfolder.rs defines.
    Keying on the identifier reported all five as arity bugs.
    """
    m = attr_re.search(region)
    return (m.group(1) if m else rust_name).decode()


def nargs(masked_params: str) -> int:
    """Top-level parameter count of a comment-masked parameter list."""
    s = masked_params.replace("->", "→")  # see module docstring
    parts, depth, cur = [], 0, []
    for ch in s:
        if ch in "<([{":
            depth += 1
        elif ch in ">)]}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur))
    return len([x for x in parts if x.strip()])


def _params(data: bytes, masked: bytes, open_paren: int):
    close = _match_forward(masked, open_paren)
    if close is None:
        return None
    return (
        data[open_paren + 1 : close - 1].decode("utf-8", "replace"),
        masked[open_paren + 1 : close - 1].decode("utf-8", "replace"),
        _return_kind(masked, close),
    )


def _return_kind(masked: bytes, after_params: int) -> str:
    """'unit' or 'value' — whether the function returns anything at all.

    DELIBERATELY NOT finer than this. A pointer-vs-scalar split looks far more useful —
    returning a POINTER where the declaration promises a BOOLEAN means every non-null
    result reads as TRUE, which is exactly how classpnp's ScsiGetSenseDescriptor sent
    callers down the "descriptor found" branch. But it cannot be decided syntactically:
    `PDWORD`, `PVOID` and `PIRP_CONTEXT` are pointer TYPEDEFS that a name-based classifier
    files as scalars. Trying it produced 293 findings whose first three hand-checked samples
    were ALL false (advapi32 `-> *mut DWORD` vs `-> PDWORD` is the same type twice), and a
    checker that flags everything catches every bug the same way a broken clock is right.
    That is the same lesson as the `SIZE_T` alias bug in crate_repair's panicking_cast the
    same night: A TYPE NAME IS NOT A TYPE. Deciding it properly needs the bindgen islands,
    which is a different tool.

    unit-vs-value needs no type knowledge — it is the presence or absence of `->`.
    """
    tail = masked[after_params : after_params + 200].decode("utf-8", "replace")
    tail = tail.split("{")[0].split(";")[0]
    if "->" not in tail:
        return "unit"
    ret = tail.split("->", 1)[1].strip()
    return "unit" if not ret or ret == "()" else "value"


@source_facts("tools/rustconv/rustconv/crate_repair.py")
def arity_facts(data: bytes):
    masked = _mask_lexical(data)
    entry = {"decls": {}, "defs": {}}
    for bm in RE_EXTERN_BLOCK.finditer(masked):
        end = _match_forward(masked, bm.end() - 1)
        if end is None:
            continue
        prev = bm.end()
        for dm in RE_DECL.finditer(masked, bm.end(), end):
            sig = _params(data, masked, dm.end() - 1)
            if sig:
                sym = _symbol(data[prev : dm.start()], dm.group(1), RE_LINK_NAME)
                entry["decls"].setdefault(sym, []).append((dm.group(1).decode(), sig))
            prev = dm.end()
    for fm in RE_DEF.finditer(data):
        sig = _params(data, masked, fm.end() - 1)
        if sig:
            sym = _symbol(data[fm.start() : fm.end()], fm.group(1), RE_EXPORT_NAME)
            entry["defs"][sym] = sig
    return entry


def scan(root):
    crates = {}
    for dirpath, dirnames, filenames in os.walk(root):
        # SORTED, not just filtered. `defs[sym] = sig` is last-write-wins, so when a
        # symbol is defined under several arch dirs (ke/amd64/stubs.rs vs ke/arm64/
        # stubs.rs) the one NAMED in the finding is whichever os.walk happened to visit
        # last -- i.e. raw readdir order. That is stable on a given tree and changes when
        # the tree is recreated: restoring this repo on 2026-09-14 flipped the reported
        # arch to amd64, so a baselined finding read as "1 new" with no source change and
        # a red `check` on master. A baseline cannot be stable if the scanner's output
        # depends on directory iteration order.
        dirnames[:] = sorted(d for d in dirnames if d not in ("target", ".git"))
        for fn in sorted(filenames):
            if not fn.endswith(".rs"):
                continue
            path = os.path.join(dirpath, fn)
            crate = crate_root_of(path)
            if crate is None:
                continue
            try:
                with open(path, "rb") as fh:
                    data = fh.read()
            except OSError:
                continue
            # An `//! ORPHAN-BY-DESIGN:` file is in no `mod` chain, so neither its
            # declarations nor its definitions can take part in a call — the exemption
            # this module's docstring already states, now enforced instead of baselined.
            # It also stops a parked translation from "disagreeing" with the live
            # declaration it exists to REPLACE.
            if b"ORPHAN-BY-DESIGN:" in data[:2048]:
                continue
            if b"extern" not in data and b"no_mangle" not in data:
                continue
            facts = arity_facts(data)
            entry = crates.setdefault(crate, {"decls": {}, "defs": {}})
            for sym, sites in facts["decls"].items():
                entry["decls"].setdefault(sym, []).extend((path, *site) for site in sites)
            for sym, sig in facts["defs"].items():
                entry["defs"][sym] = (path, sig)

    findings = []
    for crate, e in crates.items():
        live = {os.path.realpath(x) for x in crate_live_files(crate)}
        for sym, sites in sorted(e["decls"].items()):
            if sym not in e["defs"]:
                continue
            dpath, dsig = e["defs"][sym]
            for path, name, sig in sites:
                if "..." in sig[0] or "..." in dsig[0]:
                    continue
                # A decl in an unreachable file cannot make a wrong call.
                if live and os.path.realpath(path) not in live:
                    continue
                a, b = nargs(sig[1]), nargs(dsig[1])
                if a != b:
                    findings.append(("ARITY", path, name, a, b, dpath))
                elif sig[2] == "value" and dsig[2] == "unit":
                    # Only this DIRECTION is a hazard. Declared-unit/defined-value means the
                    # caller simply ignores a result; declared-value/defined-unit means the
                    # caller READS a register the callee never set.
                    findings.append(("RETVOID", path, name, sig[2], dsig[2], dpath))
    return sorted(findings)


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("paths", nargs="*", help="dirs to scan (default: sdk/lib/rust)")
    args = ap.parse_args()
    findings = []
    for root in args.paths or [DEFAULT_ROOT]:
        findings += scan(root)
    for kind, path, name, a, b, dpath in findings:
        rel = os.path.relpath(path, REPO)
        drel = os.path.relpath(dpath, REPO)
        if kind == "ARITY":
            print(
                f"{rel}:1: [ARITY] {name}: declared with {a} parameter(s), defined with "
                f"{b} in {drel} — the caller and callee disagree about the call shape"
            )
        else:
            print(
                f"{rel}:1: [RETVOID] {name}: declared as returning a value, defined as "
                f"returning nothing in {drel} — the caller reads a register the callee "
                f"never set"
            )
    n_arity = sum(1 for f in findings if f[0] == "ARITY")
    print(f"\nSUMMARY: arity-mismatches={n_arity}  retvoid-mismatches={len(findings) - n_arity}")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
