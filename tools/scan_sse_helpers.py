#!/usr/bin/env python3
"""
scan_sse_helpers.py — a re-ported section-segment-entry (SSE) helper that disagrees with mm.h.

mm.h packs a segment page entry as: bit 0 swap, bit 1 dirty, bit 2 write-in-progress, bits
3..11 share count, the page frame above. Its `*_SSE` macros are one-liners, and the Rust tree
re-ported them per module. Two copies drifted:

    ntkrnl_sys::mm::IS_DIRTY_SSE   tested 0x8 -- the low bit of the SHARE COUNT. Every page
                                   mapped an odd number of times looked dirty, so
                                   MiShutdownSystem looped "while dirty" forever and the
                                   install's reboot never came.
    ARM3/section.rs                a whole private set: swap bit inverted, share count at bit 2.

Nothing compiles differently, and the first one sat harmless until a path exercised it.

This evaluates every Rust `fn` named like an mm.h `*_SSE` macro against the macro itself,
on the same sample entries, and reports any input where they disagree (truthiness for the
IS_* predicates, the value otherwise). Gated at zero.

Usage:
    python3 tools/scan_sse_helpers.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_sse_helpers.py --lint
"""

from __future__ import annotations

import itertools
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_wide_byte_copy import SKIP_FILE_RE
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
MM_H = REPO / "ntoskrnl" / "include" / "internal" / "mm.h"
MARKER = "ORIGINAL C SOURCE"
MASK = (1 << 64) - 1
DEFINE_RE = re.compile(r"^#define\s+(\w+_SSE|SHARE_COUNT_FROM_SSE)\((\w+)(?:,\s*(\w+))?\)\s+(.+)$")
# Entries a 64-bit kernel actually holds: page frames below 1 TiB (mm.h's _WIN64
# PAGE_FROM_SSE keeps 40 bits; a wider Rust mask agrees on all of these).
PAGES = [0, 0x1000, 0x12345000, 0xFD4000, 0x3D0AD000]
SAMPLES1 = [p | (c << 3) | f for p in PAGES for c in range(6) for f in range(8)]
SAMPLES2 = list(itertools.product([0x1000, 0x12345000], range(6)))


def _c_to_py(body: str) -> str:
    body = re.sub(r"\((?:PFN_NUMBER|ULONG_PTR|ULONG|SWAPENTRY)\)", "", body)
    body = re.sub(r"(0x[0-9A-Fa-f]+|\d+)(?:ULL|UL|U|L)\b", r"\1", body)
    body = body.replace("&&", " and ").replace("||", " or ")
    return body.replace("PAGE_SHIFT", "12")


def mm_h_macros() -> dict[str, tuple[int, object]]:
    """name -> (arity, python callable). The _WIN64 branch where mm.h has two."""
    defs: dict[str, tuple[list[str], str]] = {}
    text = MM_H.read_text(errors="replace").replace("\\\n", " ")
    in_else = False
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("#else"):
            in_else = True
            continue
        if s.startswith("#endif"):
            in_else = False
            continue
        m = DEFINE_RE.match(s)
        if m and not in_else:
            params = [p for p in (m.group(2), m.group(3)) if p]
            defs.setdefault(m.group(1), (params, _c_to_py(m.group(4))))
    env: dict[str, object] = {}
    out = {}
    for name, (params, body) in defs.items():
        src = f"lambda {', '.join(params)}: ({body})"
        try:
            env[name] = eval(src, env)
            out[name] = (len(params), env[name])
        except SyntaxError:
            pass
    return out


def _rust_to_py(body: str) -> str | None:
    body = re.sub(r"//[^\n]*", "", body)
    # cfg'd alternatives: keep the 64-bit one.
    cfg64 = re.search(r'#\[cfg\(target_pointer_width\s*=\s*"64"\)\]\s*\{(.*?)\}', body, re.S)
    if cfg64:
        body = cfg64.group(1)
    body = body.strip().rstrip(";")
    if any(tok in body for tok in ("let ", "return", "if ", "match ", "unsafe")):
        return None
    body = re.sub(r"\bas\s+[\w:]+", "", body)
    body = re.sub(
        r"(0x[0-9A-Fa-f_]+|\b\d[\d_]*)(?:u8|u16|u32|u64|usize|i32|i64|isize)?\b",
        lambda m: m.group(1).replace("_", ""),
        body,
    )
    body = re.sub(r"!(?!=)", "~", body)
    body = body.replace("&&", " and ").replace("||", " or ").replace("PAGE_SHIFT", "12")
    return body


FN_RE = re.compile(r"\bfn\s+(\w+)\s*\(([^)]*)\)\s*(?:->\s*[\w:]+\s*)?\{")


def _body(text: str, open_brace: int) -> str:
    depth = 0
    for j in range(open_brace, len(text)):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return text[open_brace + 1 : j]
    return ""


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    macros = mm_h_macros()
    findings = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if f.suffix != ".rs" or SKIP_FILE_RE.search(str(f)):
                continue
            text = f.read_text(errors="replace")
            cut = text.find(MARKER)
            if cut >= 0:
                text = text[:cut]
            for m in FN_RE.finditer(text):
                name = m.group(1)
                if name not in macros:
                    continue
                arity, ref = macros[name]
                params = [p.split(":")[0].strip() for p in m.group(2).split(",") if p.strip()]
                if len(params) != arity:
                    continue
                py = _rust_to_py(_body(text, m.end() - 1))
                if py is None:
                    continue
                try:
                    # A sibling helper the body calls is bound to mm.h's; each sibling is
                    # checked against mm.h on its own.
                    env = {k: v[1] for k, v in macros.items()}
                    impl = eval(f"lambda {', '.join(params)}: ({py})", env)
                except SyntaxError:
                    continue
                samples = SAMPLES1 if arity == 1 else SAMPLES2
                for args in samples:
                    args = args if isinstance(args, tuple) else (args,)
                    want, got = ref(*args), impl(*args)  # type: ignore[operator]
                    if name.startswith("IS_") or name.startswith("MM_IS_"):
                        same = bool(want) == bool(got)
                    else:
                        same = (int(want) & MASK) == (int(got) & MASK)
                    if not same:
                        line = text.count("\n", 0, m.start()) + 1
                        shown = ", ".join(hex(a) for a in args)
                        findings.append(
                            (f, line, f"{name}({shown}) = {int(got):#x}, mm.h says {int(want):#x}")
                        )
                        break
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, what in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        tag = "sse-helper-drift" if lint else "SSE helper disagrees with mm.h:"
        print(f"{rel}:{lineno}: {tag} {what}")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): match ntoskrnl/include/internal/mm.h.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
