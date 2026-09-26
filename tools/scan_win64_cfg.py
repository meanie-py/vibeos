#!/usr/bin/env python3
"""
scan_win64_cfg.py — the C says `#ifdef _WIN64`; the translation says `target_arch = "x86_64"`.

    C:            #ifdef _WIN64
                      switch (nIndex) { case GCLP_HICON: ... SetLastError(ERROR_INVALID_INDEX); }
                  #else
                      Ret = IntGetClassLongW(Wnd, Class, nIndex);
                  #endif
    translated:   #[cfg(target_arch = "x86_64")]  { match nIndex { ... } }
                  #[cfg(not(target_arch = "x86_64"))] { Ret = IntGetClassLongW(...) }
    faithful:     #[cfg(target_pointer_width = "64")] / not(...)

`_WIN64` means "pointers are 64 bits", which is amd64 AND arm64. Spelling it as one arch
silently gives arm64 the 32-bit path: user32's GetClassLongA/W returned a truncated
pointer for the GCLP_* indices on arm64 instead of ERROR_INVALID_INDEX. It builds and
boots on every arch, and amd64 is correct, so nothing notices. July's per-arch cfg audit
found the same shape in the kernel (the PXE 4-level cluster, gated to one of the two
4-level arches); this is the $0 check for it.

HOW. Per Rust function whose BODY carries `#[cfg(target_arch = "x86_64")]` or its `not(…)`:
the C function of the same name (tools/c_twin.py, same module) has an `_WIN64`
conditional in its body and names no specific 64-bit arch (`_M_AMD64`, `_M_ARM64`, …) —
so the C's split is pointer width, not architecture. A C body that names an arch
explicitly (`#if defined(_WIN64) && !defined(_M_ARM64)`) made its choice on purpose.

Usage:
    python3 tools/scan_win64_cfg.py            # report
    python3 tools/scan_win64_cfg.py --lint     # terse (CLI baseline gate)

Suppress a reviewed site with `// nolint: win64-cfg` on the cfg line. Exit 1 if any finding.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.c_twin import CIndex, Func, blank_rust, prefer, rust_functions, same_module
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

X64_CFG = re.compile(r'#\[cfg\((?:not\()?\s*target_arch\s*=\s*"x86_64"\s*\)?\)\]')
C_WIN64 = re.compile(
    r"#\s*if(?:def\s+_WIN64\b|\s+!?\s*defined\s*\(?\s*_WIN64\b)|#\s*ifndef\s+_WIN64\b"
)
C_NAMES_ARCH = re.compile(r"_M_AMD64|_AMD64_|__x86_64__|_M_X64|_M_ARM64|_ARM64_|__aarch64__")
NOLINT = "nolint: win64-cfg"


def _rel(p: Path) -> Path:
    try:
        return p.relative_to(REPO)
    except ValueError:
        return p


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    lint = "--lint" in sys.argv
    roots = [Path(a) for a in args] if args else [RUST_ROOT]
    sites: list[tuple[Path, Func, int, str]] = []
    for root in roots:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            raw = f.read_text(errors="replace")
            if "x86_64" not in raw:
                continue
            lines = raw.split("\n")
            for fn in rust_functions(blank_rust(raw), f):
                # Attributes are code, so blanking keeps their offsets; read them raw.
                body = raw[fn.body_start : fn.body_start + len(fn.body)]
                m = X64_CFG.search(body)
                if not m:
                    continue
                ln = raw.count("\n", 0, fn.body_start + m.start()) + 1
                if NOLINT in lines[ln - 1]:
                    continue
                sites.append((f, fn, ln, m.group(0)))
    index = CIndex()
    index.load({fn.name for _, fn, _, _ in sites})
    findings = []
    for f, fn, ln, cfg in sites:
        twins = [t for t in prefer(f, index.get(fn.name)) if same_module(f, t)]
        if not twins:
            continue
        c = twins[0].path.read_text(errors="replace")
        cbody = c[twins[0].body_start : twins[0].body_start + len(twins[0].body)]
        m = C_WIN64.search(cbody)
        if not m or C_NAMES_ARCH.search(cbody):
            continue
        findings.append((_rel(f), ln, fn.name, cfg, _rel(twins[0].path), m.group(0).strip()))
    for rel, ln, name, cfg, crel, cond in findings:
        if lint:
            print(f"{rel}:{ln}: win64-cfg {name}")
        else:
            print(f"{rel}:{ln}: win64-cfg: {name}: `{cfg}` where {crel} says `{cond}`")
    if not lint:
        print(f"\n{len(findings)} site(s)")
        if findings:
            print('fix: `_WIN64` is `target_pointer_width = "64"` (amd64 AND arm64).')
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
