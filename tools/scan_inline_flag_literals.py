#!/usr/bin/env python3
"""
scan_inline_flag_literals.py — catch WRONG inlined flag/constant literals (AST-aware).

verify_constants.py only checks `const NAME: T = VALUE;` definitions. The LLM grind
also inlines BARE literals annotated with the intended name in a comment, e.g.

    SetFlag(&Flags, 0x2 /* IRP_CONTEXT_FLAG_TOP_LEVEL */);   // WRONG: 0x2 = MORE_PROCESSING
    PALETTE_AllocPalette(0x4 /* PAL_BITFIELDS */ | PAL_DIBSECTION)  // WRONG: PAL_BITFIELDS=0x2

Those are invisible to verify_constants.py (no named const). This finds them by
parsing the Rust with tree-sitter (NOT regex — adjacency of a literal to a comment
across nested `|`/parens/casts/suffixes is not a regular language), then for each
integer_literal node finds the constant-name comment that annotates it and checks the
value against the C headers.

Usage:
    python3 tools/scan_inline_flag_literals.py <file.rs ...> [--headers DIR ...]
    python3 tools/scan_inline_flag_literals.py sdk/lib/rust/cdfs-rust/src/*.rs \
        --headers drivers/filesystems/cdfs
    python3 tools/scan_inline_flag_literals.py            # no paths -> sweep all rust
    python3 tools/scan_inline_flag_literals.py --lint     # terse (CLI baseline gate)

Catalog = kernel headers verify_constants.py uses + sdk/include/{xdk,ndk,psdk} +
ntoskrnl/include/internal + every *.h under each --headers dir (driver-local flags).
Exit 1 if any mismatch is found. Wired into `vibeos-cli check` (baseline-diff gate).

Suppress a verified false positive at the source with a `nolint` comment on the line:
    PALETTE_AllocPalette(0x4 /* PAL_BITFIELDS */ ...)  // nolint: inline-literal (why)
Bare `// nolint` or `// nolint: inline-literal` suppress; a `nolint` scoped to another
tool does not. Prefer this (local, reviewable, refactor-stable) over the baseline.
"""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools" / "rustconv"))
import tree_sitter_rust as tsr  # noqa: E402
from rustconv.verify_constants import build_c_catalog, parse_c_hex  # noqa: E402
from tree_sitter import Language, Parser  # noqa: E402

DEFINE_RE = re.compile(r"^#define\s+([A-Z_][A-Z0-9_]*)\s+(.+?)(?:\s*//.*)?$")
_SUFFIX_RE = re.compile(r"(u8|u16|u32|u64|usize|i8|i16|i32|i64|isize)$")
# A comment segment that "looks like a flag annotation": just an UPPER_CASE name,
# optionally followed by a short parenthetical / punctuation — NOT prose. This keeps
# `/* NAME */`, `// NAME (grind had 0x2)` but rejects `// NULL / no '=' => -1`.
_NAME_SEG_RE = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*(?:\(.*\))?\s*$")
# Per-line suppression an agent adds after verifying a FP, e.g.
#   PALETTE_AllocPalette(0x4 /* PAL_BITFIELDS */ ...)  // nolint: inline-literal (checked)
# Bare `nolint` (no id) or scoped to this lint's ids both suppress; a `nolint` scoped to
# some OTHER tool does not. Reason text after the id is encouraged but not required.
_NOLINT_RE = re.compile(r"\bnolint\b\s*[:(\[]?\s*([a-z0-9_-]*)", re.I)
_NOLINT_IDS = {
    "",
    "inline_literal",
    "inline-literal",
    "inline_flag_literal",
    "inline-flag-literal",
    "scan_inline_flag_literals",
    "all",
}


def suppressed(line_text):
    m = _NOLINT_RE.search(line_text)
    return m is not None and m.group(1).lower() in _NOLINT_IDS


def parse_c_value(val):
    """Parse a #define RHS: 0x10, 16, (0x10), ((NODE_TYPE_CODE)0x0308), (ULONG)5."""
    s = val.strip()
    while s.startswith("(") and s.endswith(")"):
        s = s[1:-1].strip()
    s = re.sub(r"^\(\s*[A-Za-z_][A-Za-z0-9_ ]*\)\s*", "", s).strip()
    s = _SUFFIX_RE.sub("", s).rstrip("uUlL")
    try:
        return int(s, 16) if s.lower().startswith("0x") else int(s)
    except ValueError:
        return parse_c_hex(s)


def lit_value(s):
    """Value of a Rust integer_literal node's text (0x2, 0u8, 512, 0x00000040)."""
    s = _SUFFIX_RE.sub("", s.strip().replace("_", ""))
    try:
        return int(s, 16) if s.lower().startswith("0x") else int(s)
    except ValueError:
        return None


def add_headers(catalog, header_dir):
    for hf in Path(header_dir).rglob("*.h"):
        try:
            text = hf.read_text(errors="ignore")
        except OSError:
            continue
        text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
        for line in text.splitlines():
            m = DEFINE_RE.match(line.strip())
            if m:
                v = parse_c_value(m.group(2).strip())
                if v is not None:
                    catalog.setdefault(m.group(1), v)  # curated value wins collisions


def comment_names(text):
    """Constant names a comment annotates, in order, or [] if it's prose.

    `/* A | B */` -> ['A','B']; `// NAME (grind had 0x2)` -> ['NAME'];
    `// NULL / no '=' => -1` -> [] (prose, not a value annotation)."""
    body = text.strip()
    if body.startswith("/*"):
        body = body[2:-2] if body.endswith("*/") else body[2:]
    elif body.startswith("//"):
        body = body[2:]
    names = []
    for seg in body.split("|"):
        m = _NAME_SEG_RE.match(seg)
        if not m:
            return []  # any non-clean segment => treat whole comment as prose
        names.append(m.group(1))
    return names


# ── AST walk ─────────────────────────────────────────────────────────
_PARSER = Parser(Language(tsr.language()))


def collect(node, lits, comments):
    if node.type == "integer_literal":
        lits.append(node)
    elif node.type in ("line_comment", "block_comment"):
        comments.append(node)
    for c in node.children:
        collect(c, lits, comments)


def scan_file(path, catalog):
    src = path.read_bytes()
    root = _PARSER.parse(src).root_node
    lits, comments = [], []
    collect(root, lits, comments)

    def txt(n):
        return src[n.start_byte : n.end_byte].decode("utf8", "replace")

    lines = src.split(b"\n")

    out = []  # (line, litval, name, want, linetext)
    for c in comments:
        names = comment_names(txt(c))
        if not names:
            continue
        cline = c.start_point[0]
        line_src = lines[cline].decode("utf8", "replace").strip()
        if len(names) >= 2:
            # |-list: pair positionally with the literals on the comment's line,
            # in source order, that sit before the comment (e.g. `A | B // N1 | N2`).
            row = sorted(
                (L for L in lits if L.start_point[0] == cline and L.start_byte < c.start_byte),
                key=lambda n: n.start_byte,
            )
            pairs = list(zip(row, names, strict=False))
        else:
            # single name: the adjacent literal — the closest one touching the comment
            # (trailing `0x4 /* N */`, or leading `/* N */ 0x4`), same line only.
            name = names[0]
            before = [L for L in lits if L.end_byte <= c.start_byte and L.start_point[0] == cline]
            after = [
                L for L in lits if L.start_byte >= c.end_byte and L.start_point[0] == c.end_point[0]
            ]
            side = before or after
            # If the correct value appears ANYWHERE among the line's candidates, the
            # flag is present and correct — don't be fooled by a nearer `0` in
            # `& FLAG) != 0 // FLAG` or `== 0`. Only the closest is the annotation.
            if name in catalog and catalog[name] in [lit_value(txt(L)) for L in side]:
                pairs = []
            elif before:
                pairs = [(max(before, key=lambda n: n.end_byte), name)]  # closest preceding
            elif after:
                pairs = [(min(after, key=lambda n: n.start_byte), name)]  # closest following
            else:
                pairs = []
        for lit, name in pairs:
            if lit is None or name not in catalog:
                continue
            # COLOR_* are GetSysColor INDICES, near-universally used as a default-RGB
            # table labeled by slot (not value==index) — a pure FP family here.
            if name.startswith("COLOR_"):
                continue
            lv = lit_value(txt(lit))
            if lv is None or lv == catalog[name]:
                continue
            want = catalog[name]
            # Skip the "value table labeled by index" pattern: a big literal (e.g. an
            # 0xRRGGBB color) annotated with a tiny enum/index name (COLOR_*, etc.).
            # The comment names which SLOT the value fills, not value==constant.
            if lv > 0xFFFF and want <= 0xFF:
                continue
            # Per-line `// nolint` an agent added after verifying this is a FP.
            lit_line = lines[lit.start_point[0]].decode("utf8", "replace")
            if suppressed(lit_line) or suppressed(line_src):
                continue
            out.append((lit.start_point[0] + 1, lv, name, want, line_src))
    return out


def main():
    args = sys.argv[1:]
    files, header_dirs, lint, i = [], [], False, 0
    while i < len(args):
        if args[i] == "--headers":
            i += 1
            header_dirs.append(args[i])
        elif args[i] == "--lint":
            lint = True  # terse: only `file:line:` finding lines (for the CLI baseline gate)
        else:
            files.append(args[i])
        i += 1
    if not files:
        # No paths given → sweep the whole Rust workspace (broad catalog, no
        # driver --headers). This is how the CLI lint gate invokes it.
        files = [str(p) for p in (REPO / "sdk" / "lib" / "rust").rglob("*.rs")]

    catalog = build_c_catalog(REPO, include_enums=True)
    for d in (
        "sdk/include/xdk",
        "sdk/include/ndk",
        "sdk/include/psdk",
        "ntoskrnl/include/internal",
    ):
        p = REPO / d
        if p.exists():
            add_headers(catalog, str(p))
    for d in header_dirs:
        add_headers(catalog, d)

    mismatches = 0
    for f in files:
        p = Path(f)
        if not p.exists() or "/tests/" in f or "/target/" in f:
            continue  # test fixtures use intentional non-canonical values
        try:
            hits = scan_file(p, catalog)
        except Exception as e:
            print(f"{f}: parse error: {e}", file=sys.stderr)
            continue
        for ln, lv, name, want, line_src in hits:
            print(
                f"{f}:{ln}: literal {hex(lv)} tagged /* {name} */ but {name} = {hex(want)} ({want})"
            )
            if not lint:  # terse mode emits ONLY finding lines (baseline-friendly)
                print(f"    {line_src}")
            mismatches += 1
    if lint:
        return 1 if mismatches else 0
    if mismatches:
        print(f"\n{mismatches} inlined-literal mismatch(es) — fix to the header value.")
        return 1
    print("OK: no inlined flag-literal mismatches.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
