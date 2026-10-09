#!/usr/bin/env python3
"""
scan_formatless_printf.py — a `#[no_mangle]` printf that takes nothing after its format.

Stable Rust cannot DEFINE a C variadic, so a printf-family symbol exported from Rust has
to receive its arguments some other way: a `va_list`, or fixed slots replayed as one
(`KdIoPrintf(Format, a0, .., a11)`). A definition whose parameter list ENDS at the format
has neither. Whatever it does, it cannot format -- the arguments the caller pushed are
never read -- and the usual body copies the format string verbatim:

    RtlStringCbPrintfW(dst, cb, fmt)  ->  dst = "%S\\%S"

It links, it returns STATUS_SUCCESS, and every string built with it is wrong. Three times
so far: freeldr's RtlStringCbPrintfA (`multi(0)disk(0)cdrom(%u)` reached ArcOpen, so
rosload never loaded), KdIoPrintf (every kdio.c message printed its specifiers), and
freeldr's RtlStringCbPrintfW (setupldr registered every boot driver as "%S\\%S", then
panicked stripping ".sys" from "%S" -- the first step of any install from the bootcd).

Fix: define the symbol in C beside its narrow twin (boot/freeldr/freeldr/lib/
strsafe_shim.c forwards to libcntpr's _vsnprintf/_vsnwprintf), or take fixed slots and
replay them.

Usage:
    python3 tools/scan_formatless_printf.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_formatless_printf.py --lint
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_wide_byte_copy import SKIP_FILE_RE
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
# The exported definition: attribute, then (possibly after other attributes) the fn.
DEF_RE = re.compile(
    r"#\[(?:unsafe\()?no_mangle\)?\]\s*(?:#\[[^\]]*\]\s*)*"
    r"pub\s+(?:unsafe\s+)?extern\s+\"[^\"]+\"\s+fn\s+(\w+)\s*\(([^)]*)\)",
    re.S,
)
# printf-family names that take a variadic tail in C. The v-forms take a va_list and are
# the RIGHT shape for a Rust def, so they are excluded by name and by parameter type.
PRINTF_NAME_RE = re.compile(r"printf", re.I)
VFORM_RE = re.compile(r"(^|_)v\w*printf|VPrintf", re.I)
FORMAT_PARAM_RE = re.compile(r"^(?:mut\s+)?(\w*(?:fmt|format)\w*)\s*:", re.I)


def _code(text: str) -> str:
    return COMMENT_RE.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def _params(raw: str) -> list[str]:
    return [p.strip() for p in raw.split(",") if p.strip()]


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    findings = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if f.suffix != ".rs" or SKIP_FILE_RE.search(str(f)):
                continue
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            for m in DEF_RE.finditer(_code(text)):
                name, params = m.group(1), _params(m.group(2))
                if not PRINTF_NAME_RE.search(name) or VFORM_RE.search(name):
                    continue
                fmt_at = next((i for i, p in enumerate(params) if FORMAT_PARAM_RE.match(p)), None)
                if fmt_at is None or fmt_at != len(params) - 1:
                    continue
                findings.append((f, text.count("\n", 0, m.start()) + 1, name))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, name in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        if lint:
            print(f"{rel}:{lineno}: formatless-printf {name}")
        else:
            print(f"{rel}:{lineno}: {name} takes no arguments after its format: it cannot format")
    if findings and not lint:
        print(
            f"\n{len(findings)} finding(s): define it in C over _vsnprintf/_vsnwprintf, "
            "or take fixed slots and replay them."
        )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
