#!/usr/bin/env python3
"""
scan_type_alias_contradictions.py — one type name, two WIDTHS, both ours.

THE POINT. This is the type-level sibling of `scan_const_contradictions.py`, and
it exists because the constant scanner's whole argument applies unchanged to
`type` aliases — except that a width disagreement does not merely compare wrong,
it CORRUPTS MEMORY.

The worked example is fastfat's VBO (task #46, fixed 2026-07-29). `fat.h:38` is
`typedef ULONG32 VBO` and `FatLocateDirent` is declared `OUT PVBO ByteOffset` —
a pointer to FOUR bytes. Three files had each invented their own alias:

    types_{i386,amd64,arm64}.rs   pub type VBO = ULONG32      (4)  <- authority
    dirsup.rs                     type VBO = LARGE_INTEGER    (8)  <- definition
    dirctrl.rs                    type VBO = u64              (8)
    write.rs                      type VBO = u32              (4)

so the definition stored `(*ByteOffset).QuadPart`, an eight-byte write through a
pointer three callers backed with a `u32` stack local — 4 bytes of the caller's
frame gone on every file open, on the arch that boots FAT as its system volume.

WHY NOTHING ELSE CATCHES IT. Every single-arch build is green: the alias is
consistent *within* each file, and rustc has no reason to care that a different
file spells the same name differently. `verify_c_signatures` compares Rust decls
against the C prototype, but both sides said `PVBO`/`*mut VBO` — the names
agreed and only the resolved widths differed. And the two WRONG aliases here
agreed with each other (both 8), so the definition and its in-file callers were
mutually consistent and only the honest callers were exposed. A lint that reads
the whole tree at once is the only thing positioned to see it.

WHAT IT REPORTS. Only a name whose declarations resolve to two different KNOWN
widths inside one arch bucket. Aliases that do not resolve to a primitive width
are skipped rather than guessed at — an unresolvable RHS is a false-positive
generator, and the constant scanner's §9.113b lesson (486 findings, overwhelmingly
false, from being too eager) is the one to learn from. Pointer-width aliases
(usize/ULONG_PTR/SIZE_T) resolve per arch bucket, so a 4-vs-8 that is really just
i386-vs-amd64 does not flag.

WHY THE DISEASE IS THE DUPLICATION. Same as the constant scanner: a type declared
once and imported cannot disagree with itself. Every entry is a private copy that
should be a single shared definition — for a translated driver that is the
generated `types_<arch>.rs` island, which is correct by construction because
bindgen copied it from the C header. The baseline is the work queue.

Usage:
    python3 tools/scan_type_alias_contradictions.py            # full report
    python3 tools/scan_type_alias_contradictions.py --lint     # terse (baseline gate)
    python3 tools/scan_type_alias_contradictions.py <root>     # scan a fixture tree

Suppress a verified-legitimate divergence on the declaring line:

    type Handle = u32; // nolint: type-alias-contradiction (driver-private cookie)

Output is line-number-free (`<file>: <NAME> = <rhs> (<width>)`) so unrelated
edits above a declaration never churn the baseline.
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.lint_facts import source_facts
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent

SKIP_FILE_RE = re.compile(r"(/target/|/\.responses/)")
CHARSET_GENERIC = frozenset(
    {
        "TCHAR",
        "_TCHAR",
        "TBYTE",
        "BCHAR",
        "PTCHAR",
        "PTBYTE",
        "LPTCH",
        "PTCH",
        "LPTSTR",
        "LPCTSTR",
        "PTSTR",
        "PCTSTR",
        "PBCHAR",
    }
)

NOLINT_RE = re.compile(r"//\s*nolint(?::\s*type-alias-contradiction\b|\s*$|:?\s*\()")

# [pub[(…)]] type NAME = RHS;   — no generics, no `where`; those are not the
# C-typedef shapes this is about.
TYPE_RE = re.compile(r"^\s*(?:pub\s*(?:\([^)]*\)\s*)?)?type\s+([A-Za-z_]\w*)\s*=\s*([^;]+);")

ARCH_RE = re.compile(r"(?:^|[/_])(arm64|aarch64|amd64|x86_64|i386|x86)(?:[/_.]|$)")

# Start of an impl/trait body — see the brace stack in collect().
IMPL_RE = re.compile(r"^\s*(?:pub\s+)?(?:default\s+)?(?:unsafe\s+)?(?:impl|trait)\b")

# Pointer width per arch bucket. "common" code is compiled for all three, so a
# pointer-width alias there is genuinely ambiguous and is treated as unresolvable
# rather than assumed 64-bit.
PTR_WIDTH = {"i386": 4, "amd64": 8, "arm64": 8}

# Primitive widths. The Windows spellings matter more than the Rust ones here:
# translated code overwhelmingly writes ULONG/LONGLONG/LARGE_INTEGER, and those
# are precisely where an invented alias goes wrong.
BASE_WIDTH = {
    "u8": 1,
    "i8": 1,
    "bool": 1,
    "u16": 2,
    "i16": 2,
    "u32": 4,
    "i32": 4,
    "f32": 4,
    "char": 4,
    "u64": 8,
    "i64": 8,
    "f64": 8,
    "u128": 16,
    "i128": 16,
    # Windows / DDK spellings
    "UCHAR": 1,
    "CHAR": 1,
    "BYTE": 1,
    "BOOLEAN": 1,
    "CCHAR": 1,
    "USHORT": 2,
    "SHORT": 2,
    "WORD": 2,
    "WCHAR": 2,
    "LANGID": 2,
    "ULONG": 4,
    "LONG": 4,
    "DWORD": 4,
    "ULONG32": 4,
    "LONG32": 4,
    "UINT": 4,
    "INT": 4,
    "NTSTATUS": 4,
    "HRESULT": 4,
    "BOOL": 4,
    "ULONGLONG": 8,
    "LONGLONG": 8,
    "ULONG64": 8,
    "LONG64": 8,
    "DWORDLONG": 8,
    "LARGE_INTEGER": 8,
    "ULARGE_INTEGER": 8,
    "DWORD64": 8,
}
PTR_NAMES = {
    "usize",
    "isize",
    "ULONG_PTR",
    "LONG_PTR",
    "DWORD_PTR",
    "SIZE_T",
    "SSIZE_T",
    "UINT_PTR",
    "INT_PTR",
    "PVOID",
    "HANDLE",
    "KAFFINITY",
}


def arch_of(rel: str) -> str:
    m = ARCH_RE.search(rel)
    if not m:
        return "common"
    a = m.group(1)
    return {"aarch64": "arm64", "x86_64": "amd64", "x86": "i386"}.get(a, a)


def normalize(rhs: str) -> str:
    """Strip paths/whitespace so `crate::types::ULONG` and `ULONG` compare equal."""
    rhs = rhs.strip()
    rhs = re.sub(r"\s+", " ", rhs)
    # keep only the final path segment of a qualified name
    if "::" in rhs and not rhs.startswith("*"):
        rhs = rhs.rsplit("::", 1)[-1]
    return rhs


def width_of(rhs: str, aliases: dict[str, str], arch: str, depth: int = 0) -> int | None:
    """Resolve an alias RHS to a byte width, or None if we cannot be sure.

    None is the safe answer and is used liberally: a guess here becomes a false
    finding, and a false finding in a whole-tree lint is what gets the lint
    ignored. Raw pointers are deliberately NOT resolved to a width — two aliases
    that are both pointers are the same size whatever they point at, so a
    `*mut A` vs `*mut B` divergence is a different (and much weaker) signal than
    a 4-vs-8.
    """
    if depth > 8:
        return None
    rhs = normalize(rhs)
    if rhs.startswith("*") or rhs.startswith("&"):
        return None
    if rhs in BASE_WIDTH:
        return BASE_WIDTH[rhs]
    if rhs in PTR_NAMES:
        return PTR_WIDTH.get(arch)  # None for "common" — genuinely ambiguous
    nxt = aliases.get(rhs)
    if nxt is not None and normalize(nxt) != rhs:
        return width_of(nxt, aliases, arch, depth + 1)
    return None


@source_facts()
def alias_facts(text: str):
    decls, aliases = [], {}
    lines = text.splitlines()
    # Brace stack recording, for each open block, whether it is an impl/trait
    # body. An ASSOCIATED type is not an alias: `impl Width for Narrow { type
    # Ch = i8 }` and `impl Width for Wide { type Ch = u16 }` are msvcrt's
    # narrow/wide scanf engines, and differing per impl is the entire point of
    # an associated type. Without this every trait in the tree is a false hit,
    # and a lint people learn to ignore is worse than no lint.
    block_is_impl: list[bool] = []
    pending_impl = False
    for idx, line in enumerate(lines):
        m = TYPE_RE.match(line)
        in_impl = any(block_is_impl)
        if IMPL_RE.match(line):
            pending_impl = True
        for brace in re.finditer(r"[{}]", line):
            ch = brace.group()
            if ch == "{":
                block_is_impl.append(pending_impl)
                pending_impl = False
            elif ch == "}" and block_is_impl:
                block_is_impl.pop()
        if not m:
            continue
        if in_impl:
            continue
        name, rhs = m.group(1), normalize(m.group(2))
        # `#[cfg(...)]`-gated aliases legitimately come in mutually exclusive
        # pairs — freeldr's UINTN is u64/u32 by target_pointer_width, and only
        # one is ever compiled. The arch-path bucketing cannot see these
        # because both live in one file. Look back past doc comments and
        # other attributes for a cfg. (Same rule as scan_const_contradictions;
        # without it, every pointer-width pair in the tree is a false hit.)
        gated = False
        for back in range(1, 4):
            if idx - back < 0:
                break
            prev = lines[idx - back].strip()
            if prev.startswith("#[cfg("):
                gated = True
                break
            if not (prev.startswith("//") or prev.startswith("#[")):
                break
        if gated:
            aliases.setdefault(name, rhs)
            continue
        # Resolution table takes every declaration, including nolint'd and
        # generic ones — it is only used to follow chains.
        aliases.setdefault(name, rhs)
        if NOLINT_RE.search(line):
            continue
        if "<" in rhs or "dyn " in rhs or "impl " in rhs or "fn(" in rhs:
            continue
        # Charset-generic typedefs follow each module's own UNICODE define (winnt.h's TCHAR,
        # winddi.h's BCHAR): a UNICODE module's island says WCHAR, an ANSI one's BYTE/CHAR.
        # Both are right for their module, and the islands are generated, so a per-line
        # nolint would not survive a regen (imm32 vs bmfd, 26 "contradictions").
        if name in CHARSET_GENERIC:
            continue
        decls.append((name, rhs))
    return decls, aliases


def collect(root: Path):
    """-> (decls: name -> [(rel, rhs, arch)], aliases_by_arch: arch -> {name: rhs})"""
    rust_root = root / "sdk" / "lib" / "rust"
    if not rust_root.is_dir():
        rust_root = root
    decls: dict[str, list[tuple[str, str, str]]] = defaultdict(list)
    aliases_by_arch: dict[str, dict[str, str]] = defaultdict(dict)
    for path in sorted(source_files(rust_root)):
        if SKIP_FILE_RE.search(str(path)):
            continue
        text = path.read_text(errors="replace")
        # Repo-relative when scanning the repo (keeps the baseline stable and
        # readable); absolute otherwise, because the control harness identifies a
        # finding by looking for its fixture root in the line. Same rule, and the
        # same reason, as scan_bugcheck_stubs._show().
        if root == REPO:
            try:
                rel = str(path.relative_to(root))
            except ValueError:
                rel = str(path)
        else:
            rel = str(path)
        arch = arch_of(rel)
        local_decls, aliases = alias_facts(text)
        for name, rhs in aliases.items():
            aliases_by_arch[arch].setdefault(name, rhs)
        for name, rhs in local_decls:
            decls[name].append((rel, rhs, arch))
    return decls, aliases_by_arch


def contradictions(decls, aliases_by_arch):
    """[(name, rel, rhs, width)] where one arch bucket holds two known widths."""
    out = []
    common = aliases_by_arch.get("common", {})
    tables = {arch: {**common, **aliases} for arch, aliases in aliases_by_arch.items()}
    for name, sites in decls.items():
        if len(sites) < 2:
            continue
        by_arch: dict[str, set] = defaultdict(set)
        resolved: list[tuple[str, str, str, int]] = []
        for rel, rhs, arch in sites:
            # An alias chain is resolved within its own arch bucket, falling back
            # to "common" — that is how a per-arch island's ULONG32 resolves for
            # a hand-written file that carries no arch in its path.
            table = tables.get(arch, common)
            w = width_of(rhs, table, arch if arch != "common" else "common")
            if w is None:
                continue
            by_arch[arch].add(w)
            resolved.append((name, rel, rhs, w))
        # Cross-bucket check too: a hand-written "common" file and the per-arch
        # islands must agree, and that pairing is exactly how VBO went wrong.
        common_w = {w for _n, r, _h, w in resolved if arch_of(r) == "common"}
        bad = {a for a, v in by_arch.items() if len(v) > 1}
        for a, widths in by_arch.items():
            if a == "common" or not common_w:
                continue
            if len(widths | common_w) > 1:
                bad.add(a)
                bad.add("common")
        if not bad:
            continue
        for n, rel, rhs, w in resolved:
            if arch_of(rel) in bad:
                out.append((n, rel, rhs, w))
    return sorted(set(out))


def main() -> int:
    argv = [a for a in sys.argv[1:] if not a.startswith("--")]
    lint = "--lint" in sys.argv
    root = Path(argv[0]).resolve() if argv else REPO
    decls, aliases = collect(root)
    hits = contradictions(decls, aliases)

    for name, rel, rhs, w in hits:
        print(f"{rel}: {name} = {rhs} ({w} bytes)")

    if not lint:
        names = sorted({h[0] for h in hits})
        print(f"\n{len(hits)} declaration(s) across {len(names)} contradictory name(s)")
        if hits:
            print(
                "each name resolves to two different WIDTHS in one arch bucket — at most\n"
                "one can be right, and the loser corrupts memory rather than merely\n"
                "comparing wrong (fastfat VBO, task #46: an 8-byte store through a\n"
                "4-byte out-param, on every file open).\n"
                "fix: resolve against the C typedef, then delete the private copies in\n"
                "     favour of the generated types_<arch>.rs island; suppress a genuine\n"
                "     driver-local type with // nolint: type-alias-contradiction (reason)"
            )
    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main())
