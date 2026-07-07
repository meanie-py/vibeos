#!/usr/bin/env python3
"""scan_cc_mismatch.py — same-crate extern-decl vs #[no_mangle]-def calling-convention drift.

The class (field manual §9.30, found 2026-07-07): a Rust `extern "X" { fn F(); }`
declaration whose convention doesn't match F's `#[no_mangle] pub unsafe extern "Y" fn`
definition in the SAME link unit is UB to LLVM. On i386 (where stdcall/fastcall/cdecl
genuinely differ) LLVM silently deletes the calling branch — no ud2, no link error,
the code path just vanishes. Real hit: user32-rust declared MessageInit (FASTCALL def)
as extern "system" → the whole ClientThreadSetupHelper init branch folded away →
U32AccelCacheLock never initialized → explorer's desktop thread deadlocked in
LoadAcceleratorsW → no desktop icons on i386.

Scope: same-CRATE pairs only (cross-crate/DLL pairs go through import libs whose
decorated names catch drift at link time; same-crate pairs link silently).
i386 semantics: "system"→stdcall, "cdecl"/"C"→C, fastcall/stdcall as written.
Only decl/def pairs both compiled on i386 are compared (cfg-aware; per-arch
`*_arm64.rs`/`*_amd64.rs` files are skipped).

Output: one line per finding (stable, repo-relative) — baselined by
tools/lint-baseline/scan_cc_mismatch.txt via `vibeos-cli check` (0-NEW gate).
Suppress a verified FP with: // nolint: cc-mismatch (reason)
"""

from __future__ import annotations

import os
import re
import sys
from collections import defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BASE = os.path.join(REPO, "sdk", "lib", "rust")

NORM_I386 = {
    "system": "stdcall",
    "C": "C",
    "cdecl": "C",
    "fastcall": "fastcall",
    "stdcall": "stdcall",
    "thiscall": "thiscall",
}

# cfg prefix (0+ attributes), optionally followed by `unsafe`, then extern block
DECL_BLOCK_RE = re.compile(
    r'((?:#\[cfg[^\]]*\]\s*)*)(?:unsafe\s+)?extern\s+"(\w+)"\s*\{([^}]*)\}', re.S
)
DECL_FN_RE = re.compile(r"\bfn\s+(\w+)\s*\(")
DEF_RE = re.compile(
    r'((?:#\[cfg[^\]]*\]\s*)*)#\[no_mangle\]\s*pub\s+unsafe\s+extern\s+"(\w+)"\s+fn\s+(\w+)'
)
NOLINT_RE = re.compile(r"nolint:\s*cc-mismatch")

ARCH_FILE_RE = re.compile(r"(arm64|amd64|aarch64|x86_64)")


def cfg_excludes_i386(cfg: str) -> bool:
    if 'not(target_arch = "x86")' in cfg:
        return True
    m = re.search(r'#\[cfg\(target_arch = "(\w+)"\)\]', cfg)
    return bool(m and m.group(1) != "x86")


def scan_crate(crate_src: str) -> list[str]:
    defs: dict[str, list[tuple[str, bool]]] = defaultdict(list)
    decls: dict[str, list[tuple[str, str, int]]] = defaultdict(list)
    for root, dirs, files in os.walk(crate_src):
        dirs[:] = [d for d in dirs if d != "target"]
        for fn in files:
            # per-arch non-i386 source files never compile on i386
            if not fn.endswith(".rs") or ARCH_FILE_RE.search(fn):
                continue
            path = os.path.join(root, fn)
            try:
                with open(path, encoding="utf-8", errors="replace") as fh:
                    src = fh.read()
            except OSError:
                continue
            for m in DEF_RE.finditer(src):
                cfg, cc, name = m.groups()
                defs[name].append((cc, cfg_excludes_i386(cfg)))
            src_lines = src.splitlines()
            for m in DECL_BLOCK_RE.finditer(src):
                cfg, cc, body = m.groups()
                if cfg_excludes_i386(cfg):
                    continue
                block_line = src.count("\n", 0, m.start())
                for d in DECL_FN_RE.finditer(body):
                    line = block_line + body.count("\n", 0, d.start()) + 1
                    decl_line = src_lines[line - 1] if line - 1 < len(src_lines) else ""
                    if NOLINT_RE.search(decl_line):
                        continue
                    decls[d.group(1)].append((cc, path, line))
    out = []
    for name, dd in decls.items():
        if name not in defs:
            continue
        defcc = {NORM_I386.get(cc, cc) for cc, excluded in defs[name] if not excluded}
        if not defcc:
            continue
        for cc, path, _line in dd:
            if NORM_I386.get(cc, cc) not in defcc:
                rel = os.path.relpath(path, REPO)
                # NOTE: no line number in the finding — the baseline is keyed on
                # these lines verbatim, and line numbers churn on every edit
                # (the lint_kernel line-shift disease). file+symbol+cc is stable.
                out.append(
                    f'{rel}: [CC_MISMATCH] {name} declared extern "{cc}" '
                    f"but defined {sorted(defcc)} in-crate (i386 UB-fold risk)"
                )
    return out


def main() -> int:
    findings: list[str] = []
    for d in sorted(os.listdir(BASE)):
        crate_src = os.path.join(BASE, d, "src")
        if os.path.isdir(crate_src):
            findings.extend(scan_crate(crate_src))
    for line in sorted(set(findings)):
        print(line)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
