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

`--fix` moves each mismatching DECL into an extern block spelled with the DEFINITION's
convention. That direction is deliberate, and it is only correct because of what the
scanner's scope statement already says: these are SAME-CRATE pairs, so the `#[no_mangle]`
definition IS the symbol LLVM emits and the decl is just a local description of it. A decl
cannot be the authority for a symbol it does not define.

The one case where the definition really is the wrong side is a function bound to a TYPED
FUNCTION POINTER that a foreign caller dereferences — there the pointer's type is a hard
external contract. Checked before writing this: of 256 mismatching symbols only 7 are ever
bound to a function pointer, 6 of them buslogic's SCSI miniport callbacks (whose definitions
were fixed by hand, as a real bug, in the commit that added this mode) and the 7th
(`ForwardIrpAndForget`) already matching its `PDRIVER_DISPATCH` type. So --fix does not need
to reason about pointer types; if that ever stops being true the oracle to re-run is
"which of these symbols appear in `= Some(F)` / `= cb!(F)` / `transmute(F as ...)`".

GENERATED BINDGEN ISLANDS ARE NEVER EDITED, and this is the rule that matters most here.
An island decl is not a local description of our symbol — it is bindgen's transcript of the
C prototype, so it is EVIDENCE, and "fix the decl" against evidence is not a fix, it is
tampering with the oracle. Learned the expensive way: an earlier mechanical pass in this
lane (e33a810c9ad) canonicalised decl->def across the tree and rewrote 8 island blocks from
`extern "stdcall"` to `extern "C"` — silencing the lint on buslogic's 6 SCSI miniport
callbacks and cdrom's 2 WDF cleanup callbacks, every one of which the C declares NTAPI /
STDCALL and our Rust wrongly defined cdecl. The scanner's live count fell 305 -> 16 and read
as progress. So: a mismatch whose decl lives in an island means the DEFINITION is wrong;
--fix leaves it alone and reports it for a human, because deciding it needs the C header.
"""

from __future__ import annotations

import argparse
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

# Generated bindgen islands. `--fix` refuses to write these (see the module docstring): the
# decls in them are bindgen's transcript of the C prototypes, i.e. the evidence the finding
# is judged against, so editing them to match a definition destroys the only authority that
# could say which side is wrong.
ISLAND_RE = re.compile(r"(?:^|/)(?:types|bindgen)\w*\.rs$")


def cfg_excludes_i386(cfg: str) -> bool:
    if 'not(target_arch = "x86")' in cfg:
        return True
    m = re.search(r'#\[cfg\(target_arch = "(\w+)"\)\]', cfg)
    return bool(m and m.group(1) != "x86")


# `#[cfg(target_arch = "x86_64")] mod ke_amd64_exports { ... }` — a per-arch MODULE. The
# attribute is on the `mod`, not on the items inside it, so cfg_excludes_i386 (which only
# ever sees the attributes directly preceding a def) counts every item in such a module as
# present on i386. That is not cosmetic: shim_exports.rs defines KeQueryInterruptHandler and
# KeRegisterInterruptHandler `extern "C"` inside an x86_64-only module, and treating those as
# the i386 authority made this tool's --fix rewrite ntoskrnl's *correct* `extern "system"`
# decls to match a definition that does not exist on i386. A cfg'd-out def is not evidence.
RE_CFG_MOD = re.compile(r"#\[cfg\(([^\]]*)\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{")


def excluded_spans(src: str) -> list[tuple[int, int]]:
    """Byte ranges of `#[cfg(...)] mod` blocks that are NOT compiled on i386."""
    spans = []
    for m in RE_CFG_MOD.finditer(src):
        if not cfg_excludes_i386(f"#[cfg({m.group(1)})]"):
            continue
        depth, i = 0, m.end() - 1
        while i < len(src):
            if src[i] == "{":
                depth += 1
            elif src[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        spans.append((m.start(), i))
    return spans


def in_spans(pos: int, spans: list[tuple[int, int]]) -> bool:
    return any(a <= pos <= b for a, b in spans)


def walk_i386_sources(crate_src: str):
    """Every .rs in the crate that is actually compiled on i386, as (path, text).

    Shared by the scanner and `--fix` on purpose: a fixer that walked a different file set
    than the scanner would either miss findings or edit files the gate never looks at."""
    for root, dirs, files in os.walk(crate_src):
        # skip build dirs AND per-arch non-i386 source dirs (ke/arm64/, amd64/, …)
        dirs[:] = [d for d in dirs if d != "target" and not ARCH_FILE_RE.fullmatch(d)]
        for fn in sorted(files):
            # per-arch non-i386 source files never compile on i386
            if not fn.endswith(".rs") or ARCH_FILE_RE.search(fn):
                continue
            path = os.path.join(root, fn)
            try:
                with open(path, encoding="utf-8", errors="replace") as fh:
                    yield path, fh.read()
            except OSError:
                continue


def scan_crate(crate_src: str) -> list[str]:
    defs: dict[str, list[tuple[str, bool]]] = defaultdict(list)
    decls: dict[str, list[tuple[str, str, int]]] = defaultdict(list)
    for path, src in walk_i386_sources(crate_src):
        spans = excluded_spans(src)
        for m in DEF_RE.finditer(src):
            cfg, cc, name = m.groups()
            defs[name].append((cc, cfg_excludes_i386(cfg) or in_spans(m.start(), spans)))
        src_lines = src.splitlines()
        for m in DECL_BLOCK_RE.finditer(src):
            cfg, cc, body = m.groups()
            if cfg_excludes_i386(cfg) or in_spans(m.start(), spans):
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


def crate_def_conventions(crate_src: str) -> dict[str, set[str]]:
    """symbol -> the convention(s) its i386-compiled `#[no_mangle]` definitions are written
    with, AS SPELLED (not i386-normalized) — the new block has to be spelled, not normalized."""
    defs: dict[str, set[str]] = defaultdict(set)
    for path, src in walk_i386_sources(crate_src):
        del path
        spans = excluded_spans(src)
        for m in DEF_RE.finditer(src):
            cfg, cc, name = m.groups()
            if not cfg_excludes_i386(cfg) and not in_spans(m.start(), spans):
                defs[name].add(cc)
    return defs


def split_items(body: str) -> list[str]:
    """Split an extern block body into whole items, each ending at its top-level `;`.

    Depth counts `(`/`[`/`{` only. It deliberately does NOT track `<`/`>`: the only `;` that
    can hide inside a Rust type is the one in an array length (`[T; 1]`), which `[` already
    covers — and pulling `<`/`>` in would re-import the `->` trap that made an earlier
    scanner in this repo report 3,790 phantom findings (a `->` drives `>` depth negative,
    after which every top-level separator is invisible).

    An item does NOT end at its `;` — it ends at the END OF THAT LINE, so a trailing
    comment stays with the declaration it annotates. Cutting at the `;` instead put
    `// nolint: cc-mismatch (...)` into the FOLLOWING chunk, which both destroyed a valid
    suppression (the scanner reads nolint off the decl's line, so scanner and fixer
    disagreed about the very same decl) and left the rewritten block misformatted."""
    items, depth, cur, i = [], 0, [], 0
    while i < len(body):
        ch = body[i]
        cur.append(ch)
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == ";" and depth == 0:
            j = body.find("\n", i)
            rest = body[i + 1 : len(body) if j < 0 else j + 1]
            # Only absorb a trailing COMMENT (or plain whitespace) — never a second item
            # that happens to share the line.
            if not rest.strip() or rest.lstrip().startswith("//"):
                cur.append(rest)
                i = len(body) if j < 0 else j
            items.append("".join(cur))
            cur = []
        i += 1
    tail = "".join(cur)
    if tail.strip():
        items.append(tail)
    return items


def fix_file(path: str, defs: dict[str, set[str]]) -> list[str]:
    """Move every mismatching decl in `path` into a block spelled with its def's
    convention. Returns one description per moved decl."""
    with open(path, encoding="utf-8", errors="replace") as fh:
        src = fh.read()
    moved: list[str] = []
    spans = excluded_spans(src)
    out, last = [], 0
    for m in DECL_BLOCK_RE.finditer(src):
        cfg, cc, body = m.groups()
        if cfg_excludes_i386(cfg) or in_spans(m.start(), spans):
            continue
        keep, relocate = [], defaultdict(list)
        for item in split_items(body):
            d = DECL_FN_RE.search(item)
            name = d.group(1) if d else None
            spellings = defs.get(name or "", set())
            # Ambiguity is not something to guess at: a symbol whose i386 definitions
            # disagree with EACH OTHER, or which has no in-crate definition, is left alone
            # for a human. Same for an explicit nolint.
            if (
                not name
                or len(spellings) != 1
                or NOLINT_RE.search(item)
                or NORM_I386.get(cc, cc) in {NORM_I386.get(s, s) for s in spellings}
            ):
                keep.append(item)
                continue
            target = next(iter(spellings))
            relocate[target].append(item)
            moved.append(f"{os.path.relpath(path, REPO)}: {name} {cc!r} -> {target!r}")
        if not relocate:
            continue
        indent = re.match(r"[ \t]*", src[src.rfind("\n", 0, m.start()) + 1 : m.start()]).group(0)
        blocks = []
        if any(i.strip() for i in keep):
            blocks.append(f'{cfg}extern "{cc}" {{{"".join(keep)}}}')
        for target, items in sorted(relocate.items()):
            blocks.append(f'{cfg}extern "{target}" {{{"".join(items)}}}')
        out.append(src[last : m.start()])
        out.append(f"\n\n{indent}".join(blocks))
        last = m.end()
    if not moved:
        return []
    out.append(src[last:])
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("".join(out))
    return moved


def fix_crate(crate_src: str) -> tuple[list[str], list[str]]:
    """(moved, refused) — `refused` names island decls whose DEFINITION a human must judge."""
    defs = crate_def_conventions(crate_src)
    moved, refused = [], []
    for path, src in walk_i386_sources(crate_src):
        if ISLAND_RE.search(path.replace(os.sep, "/")):
            spans = excluded_spans(src)
            for m in DECL_BLOCK_RE.finditer(src):
                cfg, cc, body = m.groups()
                if cfg_excludes_i386(cfg) or in_spans(m.start(), spans):
                    continue
                for d in DECL_FN_RE.finditer(body):
                    spellings = defs.get(d.group(1), set())
                    if spellings and NORM_I386.get(cc, cc) not in {
                        NORM_I386.get(s, s) for s in spellings
                    }:
                        refused.append(
                            f"{os.path.relpath(path, REPO)}: {d.group(1)} — C says "
                            f"{cc!r}, our definition says {sorted(spellings)}; the "
                            f"DEFINITION is the suspect side. Not auto-fixed."
                        )
            continue
        moved.extend(fix_file(path, defs))
    return moved, refused


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--fix",
        action="store_true",
        help="move mismatching decls into a block spelled with the DEFINITION's convention",
    )
    args = ap.parse_args()
    crates = [
        os.path.join(BASE, d, "src")
        for d in sorted(os.listdir(BASE))
        if os.path.isdir(os.path.join(BASE, d, "src"))
    ]
    if args.fix:
        moved, refused = [], []
        for c in crates:
            m, r = fix_crate(c)
            moved += m
            refused += r
        for line in sorted(moved):
            print(line)
        for line in sorted(refused):
            print(f"REFUSED {line}")
        print(
            f"\nmoved {len(moved)} declaration(s); refused {len(refused)} in generated "
            f"islands (their DEFINITION is the suspect side — judge against the C header)",
            file=sys.stderr,
        )
        return 0
    findings = [line for c in crates for line in scan_crate(c)]
    for line in sorted(set(findings)):
        print(line)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
