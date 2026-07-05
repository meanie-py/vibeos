#!/usr/bin/env python3
"""freeldr_linkpromote — finish partial cohabitation ports from the
cold-link undefined-symbol oracle.

After a freeldr unit's .c is commented (cohabitation flip), the link
needs every symbol that .c provided to be a real exported symbol in the
matching .rs. rustconv often left the translated fn PRIVATE
(`unsafe fn X` / `pub unsafe fn X`) — compiles in the staticlib but
exports nothing, so the cold link reports `undefined symbol: _X`
(cdecl) or `_X@N` (i386 stdcall/NTAPI).

This pass takes the linker's undefined list and, for each symbol, finds
its DEFINITION in freeldr-rust/src/*.rs (a `fn <sym>(` that is NOT
inside an `extern "..." { }` block and NOT already `#[no_mangle]`) and
rewrites it to:
    #[no_mangle]
    pub unsafe extern "<abi>" fn <sym>(
where <abi> = "system" if the undefined symbol carried an `@N` suffix
(i386 stdcall decoration → extern "system" emits `_X@N`), else "C"
(cdecl → `_X`). FLDRAPI is empty on the non-MSVC build so freeldr's own
APIs are cdecl; the Nt*/Ke*/Ex*/Rtl* kernel-ABI ones are stdcall (@N).

Input: undefined symbols, one per line on stdin OR as args, in either
`_Name`, `Name`, `_Name@N`, or raw `ld.lld: error: undefined symbol: _X`
log-line form (the parser is tolerant). Deterministic + idempotent
(re-running after a promote is a noop for that symbol). Reusable for
every freeldr unit + freeldr_pe AND uefildr cold-link passes.

Usage:
  grep 'undefined symbol' link.log | python3 freeldr_linkpromote.py
  python3 freeldr_linkpromote.py _ExFreePool@4 _AddMemoryDescriptor ...
"""
import re
import sys
from pathlib import Path

SRC = Path(__file__).resolve().parents[2] / "sdk/lib/rust/freeldr-rust/src"


def parse_syms(lines):
    out = {}
    for ln in lines:
        for m in re.finditer(r'\b_?([A-Za-z_]\w*)(@\d+)?\b', ln):
            name, at = m.group(1), m.group(2)
            if name in ("undefined", "symbol", "error", "ld", "lld",
                        "referenced", "by", "note"):
                continue
            # first plausible identifier per line wins
            out.setdefault(name, "system" if at else "C")
            break
    return out


def promote(name, abi, arch=None):
    # find the .rs that DEFINES it (a fn item, not an extern decl).
    # ARCH-AWARE: arch-suffixed modules (winldr_i386/amd64/arm64.rs) are
    # `#[cfg(target_arch=...)]`-gated — only one compiles per --target.
    # Globbing alphabetically would promote the WRONG arch's copy (a
    # no-op for the failing link + a stray cross-arch edit). If `arch`
    # is given, skip the other arches' `*_<otherarch>.rs` files.
    ARCH_SUFFIX = {"i386": "_i386", "amd64": "_amd64", "arm64": "_arm64"}
    def _wrong_arch(fname):
        if arch is None:
            return False
        keep = ARCH_SUFFIX.get(arch)
        for a, suf in ARCH_SUFFIX.items():
            if a != arch and fname.endswith(suf + ".rs"):
                return True
        return False
    for p in sorted(SRC.glob("*.rs")):
        if _wrong_arch(p.name):
            continue
        s = p.read_text()
        # definition regex: optional pub / unsafe / extern "abi", then
        # `fn name(`, at statement start (line-leading whitespace only).
        defre = re.compile(
            r'^([ \t]*)((?:#\[no_mangle\]\s*)?)'
            r'(pub\s+)?(unsafe\s+)?(extern\s+"[^"]*"\s+)?fn\s+'
            + re.escape(name) + r'\s*(<[^>]*>)?\s*\(', re.M)
        m = defre.search(s)
        if not m:
            continue
        # A generic fn (rustconv stubs e.g. `fn UNREFERENCED_PARAMETER<T>`)
        # CANNOT be #[no_mangle] extern — it has no single mangled symbol.
        # Skip + report so it's hand-fixed (give it a concrete C-ABI arg),
        # never emit a broken `fn X(_: T)` generic export.
        if m.group(6):
            return p.name, "generic-skip(hand-fix)"
        # skip if this match is inside an `extern "..." { ... }` block
        head = s[:m.start()]
        if head.count("extern \"") and head.rfind("extern \"") > head.rfind("}") \
           and re.search(r'extern\s+"[^"]*"\s*\{[^}]*$', head):
            continue
        if m.group(2):  # already #[no_mangle]
            cur_abi = m.group(5)  # `extern "X" ` or None
            want = f'extern "{abi}" '
            if cur_abi and cur_abi.strip() == f'extern "{abi}"':
                return p.name, "already"
            # ABI mismatch: link wants _X@N (stdcall→"system") but the
            # def is extern "C" (emits _X) — or vice-versa. Rewrite the
            # signature's extern clause in place.
            sig_re = re.compile(
                r'((?:pub\s+)?(?:unsafe\s+)?)(extern\s+"[^"]*"\s+)?(fn\s+'
                + re.escape(name) + r'\b)')
            s2 = s[:m.start()] + sig_re.sub(
                lambda mm: f'{mm.group(1)}{want}{mm.group(3)}',
                s[m.start():], count=1)
            if s2 != s:
                p.write_text(s2)
                return p.name, f"abi->{abi}"
            return p.name, "already"
        indent = m.group(1)
        new_head = (f'{indent}#[no_mangle]\n'
                    f'{indent}pub unsafe extern "{abi}" fn {name}(')
        # rebuild from the `fn name(` token precisely
        fn_tok = re.compile(r'(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+"[^"]*"\s+)?'
                            r'fn\s+' + re.escape(name) + r'\s*(?:<[^>]*>)?\s*\(')
        s2 = s[:m.start()] + indent + "#[no_mangle]\n" + indent + \
            fn_tok.sub(f'pub unsafe extern "{abi}" fn {name}(',
                       s[m.start():], count=1)
        p.write_text(s2)
        return p.name, "promoted"
    return None, "notfound"


if __name__ == "__main__":
    args = sys.argv[1:]
    arch = None
    if args and args[0] == "--arch":
        arch = args[1]
        args = args[2:]
    lines = args if args else sys.stdin.read().splitlines()
    syms = parse_syms(lines)
    for name, abi in sorted(syms.items()):
        where, st = promote(name, abi, arch)
        print(f"{name:32} {abi:7} -> {where or '?'} ({st})")
