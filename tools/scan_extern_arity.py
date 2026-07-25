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
from rustconv.crate_repair import _mask_lexical, _match_forward  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from rust_module_tree import crate_live_files, crate_root_of  # noqa: E402

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
    )


def scan(root):
    crates = {}
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in ("target", ".git")]
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
            if b"extern" not in data and b"no_mangle" not in data:
                continue
            masked = _mask_lexical(data)
            entry = crates.setdefault(crate, {"decls": {}, "defs": {}})
            for bm in RE_EXTERN_BLOCK.finditer(masked):
                end = _match_forward(masked, bm.end() - 1)
                if end is None:
                    continue
                for dm in RE_DECL.finditer(masked, bm.end(), end):
                    sig = _params(data, masked, dm.end() - 1)
                    if sig:
                        entry["decls"].setdefault(dm.group(1).decode(), []).append((path, sig))
            for fm in RE_DEF.finditer(data):
                sig = _params(data, masked, fm.end() - 1)
                if sig:
                    entry["defs"][fm.group(1).decode()] = (path, sig)

    findings = []
    for crate, e in crates.items():
        live = crate_live_files(crate)
        for name, sites in sorted(e["decls"].items()):
            if name not in e["defs"]:
                continue
            dpath, dsig = e["defs"][name]
            for path, sig in sites:
                if "..." in sig[0] or "..." in dsig[0]:
                    continue
                # A decl in an unreachable file cannot make a wrong call.
                if live and os.path.realpath(path) not in {os.path.realpath(x) for x in live}:
                    continue
                a, b = nargs(sig[1]), nargs(dsig[1])
                if a != b:
                    findings.append((path, name, a, b, dpath))
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
    for path, name, a, b, dpath in findings:
        rel = os.path.relpath(path, REPO)
        drel = os.path.relpath(dpath, REPO)
        print(
            f"{rel}:1: [ARITY] {name}: declared with {a} parameter(s), defined with "
            f"{b} in {drel} — the caller and callee disagree about the call shape"
        )
    print(f"\nSUMMARY: arity-mismatches={len(findings)}")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
