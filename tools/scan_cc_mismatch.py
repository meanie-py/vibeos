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
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.lint_facts import source_facts

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

# Every group in these four patterns is NAMED, and every read of them goes through
# .group("...") — never .groups() or .group(N). Positional access is what broke this
# file once already: adding the #[export_name] group to DEF_RE renumbered cc/name, so
# `cfg, cc, name = m.groups()` began raising ValueError, and DECL_FN_RE's new #[link_name]
# group turned the fixer's `d.group(1)` into None for every ordinary declaration — which
# would have made --fix a SILENT no-op that still reported success. A scanner and a fixer
# that read the same regex differently is the specific failure this guards against.
# cfg prefix (0+ attributes), optionally followed by `unsafe`, then extern block
DECL_BLOCK_RE = re.compile(
    r'(?P<cfg>(?:#\[cfg[^\]]*\]\s*)*)(?:unsafe\s+)?extern\s+"(?P<cc>\w+)"\s*\{(?P<body>[^}]*)\}',
    re.S,
)
# A declaration may be retargeted by #[link_name], and a definition renamed by
# #[export_name]. The SYMBOL is what the linker pairs up, so capture both and key
# on the effective name — otherwise a #[link_name] decl is compared against an
# unrelated same-named definition (false positive), and a genuinely mismatched
# retargeted pair is never compared at all (false negative).
#
# The visibility/`unsafe` run between the attribute and `fn` is NOT optional decoration:
# every #[link_name] in this tree is spelled `#[link_name = "_wcsicmp"] pub fn wcsicmp(...)`
# (rtl-actctx/src/shim.rs, ps-rust/src/kill.rs). A pattern that admits only `\s*` there
# matches none of them, so the `link` group is always None and the retargeting support
# silently degrades to identifier pairing — present in the source, dead in the tree.
DECL_FN_RE = re.compile(
    r'(?:#\[link_name\s*=\s*"(?P<link>[^"]+)"\]\s*)?'
    r"(?:(?:pub(?:\([^)]*\))?|unsafe)\s+)*"
    r"\bfn\s+(?P<name>\w+)\s*\("
)
# `pub` is OPTIONAL. #[no_mangle] alone makes the symbol global to the LINKER regardless of
# Rust visibility, and crates deliberately leave it off: usbhub-rust/src/glue.rs defines
# InterlockedIncrement/Decrement as bare `#[no_mangle] unsafe extern "system" fn` — its own
# comment says PRIVATE (no `pub`) so they don't enter the glob and clash (E0659). Requiring
# `pub` here made both this scanner and scan_import_cc.py treat those as UNDEFINED, i.e. as
# imports of ntoskrnl's fastcall exports, and report a link failure against a symbol the
# crate defines itself two files over.
DEF_RE = re.compile(
    r"(?P<cfg>(?:#\[cfg[^\]]*\]\s*)*)"
    r'(?:#\[no_mangle\]|#\[export_name\s*=\s*"(?P<export>[^"]+)"\])\s*'
    r'(?:pub(?:\([^)]*\))?\s+)?unsafe\s+extern\s+"(?P<cc>\w+)"\s+fn\s+(?P<name>\w+)'
)
NOLINT_RE = re.compile(r"nolint:\s*cc-mismatch")


def def_symbol(m: re.Match) -> str:
    """The symbol the LINKER emits for a definition — #[export_name] outranks the identifier."""
    return m.group("export") or m.group("name")


def decl_symbol(d: re.Match) -> str:
    """The symbol the LINKER resolves a declaration to — #[link_name] outranks the identifier.

    Scanner and fixer MUST pair on the same symbol. When `crate_def_conventions` keyed on the
    Rust identifier while `scan_crate` keyed on the symbol, --fix looked up a name the scanner
    had never recorded and quietly relocated nothing."""
    return d.group("link") or d.group("name")


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
#
# The same reasoning covers a cfg'd BLOCK EXPRESSION — `#[cfg(target_arch = "aarch64")] { ...
# extern "C" { fn KfRaiseIrql(..); } .. }` — which is how a per-arch body inside one function
# is written (ntkrnl-sys/src/guards.rs picks fastcall on x86 and cdecl on aarch64 exactly like
# this, and is right on both). The attribute sits on the block, not on the extern inside it, so
# without this the aarch64 arm reads as an i386 declaration and its correct cdecl spelling
# reports as a link failure against hal.spec's FASTCALL KfRaiseIrql.
RE_CFG_MOD = re.compile(r"#\[cfg\(([^\]]*)\)\]\s*(?:(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*)?\{")


def excluded_spans(src: str) -> list[tuple[int, int]]:
    """Byte ranges of `#[cfg(...)] mod`/`#[cfg(...)] { .. }` that are NOT compiled on i386."""
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
    than the scanner would either miss findings or edit files the gate never looks at.

    An `//! ORPHAN-BY-DESIGN:` file is not in any `mod` chain, so nothing compiles it and
    its definitions cannot fold — the hazard this lint reports ("i386 UB-fold risk") does
    not exist until the file is wired, and it re-fires the moment it is. Honouring the same
    banner `scan_orphan_modules` defines keeps a parked translation from reporting a
    conflict against the live declaration it is going to REPLACE at wiring time."""
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
                    src = fh.read()
            except OSError:
                continue
            if "ORPHAN-BY-DESIGN:" in src[:2048]:
                continue
            yield path, src


@source_facts()
def convention_facts(src: str):
    """Shared declaration/definition index for local and imported ABI checks."""
    spans = excluded_spans(src)
    definitions, declarations = [], []
    for m in DEF_RE.finditer(src):
        definitions.append(
            (
                def_symbol(m),
                m.group("cc"),
                cfg_excludes_i386(m.group("cfg")) or in_spans(m.start(), spans),
            )
        )
    lines = src.splitlines()
    for m in DECL_BLOCK_RE.finditer(src):
        cfg, cc, body = m.group("cfg"), m.group("cc"), m.group("body")
        if cfg_excludes_i386(cfg) or in_spans(m.start(), spans):
            continue
        block_line = src.count("\n", 0, m.start())
        for d in DECL_FN_RE.finditer(body):
            # A commented-out line is not a declaration: section.rs keeps
            # `// DUPLICATE: fn KeRaiseIrql(..)` inside an extern "system" block, and reading
            # it made KeRaiseIrql look declared two ways.
            if "//" in body[body.rfind("\n", 0, d.start()) + 1 : d.start()]:
                continue
            line = block_line + body.count("\n", 0, d.start()) + 1
            text = lines[line - 1] if line - 1 < len(lines) else ""
            declarations.append((decl_symbol(d), cc, line, text))
    return definitions, declarations


# A `-sys` crate's declaration reaches this crate's LLVM module the moment the crate calls it,
# and rustc then merges it with any local declaration of the same symbol exactly as it merges
# two local ones. fastfat's pnp.rs imported the cdecl IoCopyCurrentIrpStackLocationToNext from
# ntkrnl_sys::ddk while flush.rs declared it "system"; the stdcall call linked to the cdecl
# thunk and FatCommonFlushBuffers' finally returned into its own stack on every flush it
# passed down (i386 0x23/0x50/0x1E). Only local extern blocks were indexed, so it was invisible.
SYS_USE_RE = re.compile(
    r"\buse\s+(?P<crate>\w+_sys)(?P<path>(?:::\w+)*)::(?P<tail>\{[^;]*\}|\w+)\s*;"
)
SYS_PATH_CALL_RE = re.compile(r"\b(?P<crate>\w+_sys)(?P<path>(?:::\w+)*)::(?P<name>\w+)\s*\(")


def sys_crate_decls(crate_ident: str) -> dict[str, str]:
    """symbol -> i386 convention, for every extern fn the `-sys` crate declares on i386."""
    if crate_ident not in _SYS_CACHE:
        src_dir = os.path.join(BASE, crate_ident.replace("_", "-"), "src")
        out: dict[str, str] = {}
        if os.path.isdir(src_dir):
            for _path, src in walk_i386_sources(src_dir):
                for name, cc, _line, _text in convention_facts(src)[1]:
                    out.setdefault(name, cc)
        _SYS_CACHE[crate_ident] = out
    return _SYS_CACHE[crate_ident]


_SYS_CACHE: dict[str, dict[str, str]] = {}


def sys_module_defines_rust_fn(crate_ident: str, path: str, name: str) -> bool:
    """Whether `crate::path` is a module whose own source defines a Rust `fn name` with a body.

    `ntkrnl_sys::inline::ke::KeInitializeSpinLock(..)` calls ke.h's FORCEINLINE ported to
    Rust, not the bindgen extern of the same name elsewhere in the crate. Attributing that
    call to the extern's convention made every kernel prelude that wraps the inline read as
    a DECL_CC_SPLIT against the island's (correct) extern "system" declaration."""
    segs = [seg for seg in path.split("::") if seg]
    if not segs:
        return False
    src_dir = os.path.join(BASE, crate_ident.replace("_", "-"), "src")
    for cand in (os.path.join(src_dir, *segs) + ".rs", os.path.join(src_dir, *segs, "mod.rs")):
        if os.path.isfile(cand):
            with open(cand, encoding="utf-8", errors="replace") as f:
                text = f.read()
            body = rf"^\s*pub\s+(?:unsafe\s+)?fn\s+{re.escape(name)}\s*\([^;{{]*\)[^;{{]*\{{"
            return re.search(body, text, re.M) is not None
    return False


def sys_imports(src: str) -> list[tuple[str, str, int]]:
    """(symbol, convention, line) for each extern fn this file pulls in from a `-sys` crate,
    by a named `use` or a qualified call. Glob imports are not followed: a local declaration
    shadows the glob in its own file, and which other file calls the glob's copy cannot be
    told from the text."""
    out = []
    for m in SYS_USE_RE.finditer(src):
        tail = m.group("tail")
        names = re.findall(r"\b(\w+)\b(?:\s+as\s+\w+)?", tail.strip("{}"))
        for name in names:
            if sys_module_defines_rust_fn(m.group("crate"), m.group("path"), name):
                continue
            cc = sys_crate_decls(m.group("crate")).get(name)
            if cc:
                out.append((name, cc, src.count("\n", 0, m.start()) + 1))
    for m in SYS_PATH_CALL_RE.finditer(src):
        if sys_module_defines_rust_fn(m.group("crate"), m.group("path"), m.group("name")):
            continue
        cc = sys_crate_decls(m.group("crate")).get(m.group("name"))
        if cc:
            out.append((m.group("name"), cc, src.count("\n", 0, m.start()) + 1))
    return out


def scan_crate(crate_src: str) -> list[str]:
    defs: dict[str, list[tuple[str, bool]]] = defaultdict(list)
    decls: dict[str, list[tuple[str, str, int]]] = defaultdict(list)
    for path, src in walk_i386_sources(crate_src):
        definitions, declarations = convention_facts(src)
        for name, cc, excluded in definitions:
            defs[name].append((cc, excluded))
        for name, cc, line, text in declarations:
            if not NOLINT_RE.search(text):
                decls[name].append((cc, path, line))
        lines = src.splitlines()
        for name, cc, line in sys_imports(src):
            if not NOLINT_RE.search(lines[line - 1] if line - 1 < len(lines) else ""):
                decls[name].append((cc, path, line))
    out = []
    for name, dd in decls.items():
        if name not in defs:
            out.extend(decl_split_findings(name, dd))
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


def decl_split_findings(name: str, dd: list[tuple[str, str, int]]) -> list[str]:
    """[DECL_CC_SPLIT]: one crate declares an UNDEFINED symbol under two i386 conventions.

    rustc keeps one LLVM declaration per symbol per crate, so every call site whose own view
    disagrees with the surviving one is UB. On i386 it need not fold to ud2 -- fastfat's
    fileinfo.rs declared IoGetCurrentIrpStackLocation "system" while cachesup/create/dirctrl
    said "C": fileinfo's calls went to the cdecl thunk with stdcall's "callee pops" bookkeeping,
    esp came back 4 bytes off, and the next esp-relative load read a garbage pointer
    (BUGCHECK 0x50 writing a file on FAT). The existing check needs an in-crate DEFINITION to
    judge against; this class needs none -- two answers to one question are already wrong.
    One line per declaring file and spelling, so the baseline keys stay stable."""
    ccs = {NORM_I386.get(cc, cc) for cc, _p, _l in dd}
    if len(ccs) < 2:
        return []
    seen, out = set(), []
    for cc, path, _line in dd:
        rel = os.path.relpath(path, REPO)
        if (rel, cc) in seen:
            continue
        seen.add((rel, cc))
        out.append(
            f'{rel}: [DECL_CC_SPLIT] {name} declared extern "{cc}" here; the crate also '
            f"declares it {sorted(ccs - {NORM_I386.get(cc, cc)})} (i386 esp/UB risk)"
        )
    return out


def crate_def_conventions(crate_src: str) -> dict[str, set[str]]:
    """symbol -> the convention(s) its i386-compiled `#[no_mangle]` definitions are written
    with, AS SPELLED (not i386-normalized) — the new block has to be spelled, not normalized.

    Keyed by `def_symbol`, exactly as `scan_crate` keys its own `defs`: the fixer must look up
    the same symbol the scanner reported on, or it silently finds nothing to move."""
    defs: dict[str, set[str]] = defaultdict(set)
    for path, src in walk_i386_sources(crate_src):
        del path
        for name, cc, excluded in convention_facts(src)[0]:
            if not excluded:
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
        cfg, cc, body = m.group("cfg"), m.group("cc"), m.group("body")
        if cfg_excludes_i386(cfg) or in_spans(m.start(), spans):
            continue
        keep, relocate = [], defaultdict(list)
        for item in split_items(body):
            d = DECL_FN_RE.search(item)
            name = decl_symbol(d) if d else None
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
                cfg, cc, body = m.group("cfg"), m.group("cc"), m.group("body")
                if cfg_excludes_i386(cfg) or in_spans(m.start(), spans):
                    continue
                for d in DECL_FN_RE.finditer(body):
                    symbol = decl_symbol(d)
                    spellings = defs.get(symbol, set())
                    if spellings and NORM_I386.get(cc, cc) not in {
                        NORM_I386.get(s, s) for s in spellings
                    }:
                        refused.append(
                            f"{os.path.relpath(path, REPO)}: {symbol} — C says "
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
