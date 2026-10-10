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
    python3 tools/scan_inline_flag_literals.py --no-cache # force full re-scan (skip content cache)

Results are cached per content-hash under tools/.cache/ (gitignored): a file's
findings depend only on its bytes + the header catalog + this script, so an
unchanged blob is never re-parsed. A header edit or a change to this file rolls
the cache generation automatically. `--no-cache` / VIBEOS_SCAN_NO_CACHE=1 bypasses.

Catalog = kernel headers verify_constants.py uses + sdk/include/{xdk,ndk,psdk} +
ntoskrnl/include/internal + every *.h under each --headers dir (driver-local flags).
Exit 1 if any mismatch is found. Wired into `vibeos-cli check` (baseline-diff gate).

Suppress a verified false positive at the source with a `nolint` comment on the line:
    PALETTE_AllocPalette(0x4 /* PAL_BITFIELDS */ ...)  // nolint: inline-literal (why)
Bare `// nolint` or `// nolint: inline-literal` suppress; a `nolint` scoped to another
tool does not. Prefer this (local, reviewable, refactor-stable) over the baseline.
"""

import hashlib
import json
import os
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools" / "rustconv"))
import tree_sitter_rust as tsr  # noqa: E402
from rustconv.verify_constants import build_c_catalog, parse_c_hex  # noqa: E402
from tree_sitter import Language, Parser, Query, QueryCursor  # noqa: E402

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files  # noqa: E402

DEFINE_RE = re.compile(r"^#define\s+([A-Z_][A-Z0-9_]*)\s+(.+?)(?:\s*//.*)?$")
_SUFFIX_RE = re.compile(r"(u8|u16|u32|u64|usize|i8|i16|i32|i64|isize)$")
# A comment segment that "looks like a flag annotation": just an UPPER_CASE name,
# optionally followed by a short parenthetical / punctuation — NOT prose. This keeps
# `/* NAME */`, `// NAME (grind had 0x2)` but rejects `// NULL / no '=' => -1`.
# The parenthetical is a SINGLE paren group ([^()]*): the greedy `.*` used to make
# `// NonPagedPoolNx (512) + CacheAligned (4)` parse as name=NonPagedPoolNx with one
# giant parenthetical — an arithmetic-prose comment misread as an annotation (FP).
_LONE_LIT_RE = re.compile(r"^-?(?:0x[0-9a-fA-F_]+|\d[\d_]*)\w*\s*[,;]?$")
_NAME_SEG_RE = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*(?:\([^()]*\))?\s*$")
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


def lit_values(node, src):
    """All plausible interpretations of a literal node: {magnitude, and — when the
    node sits under a unary minus — the 32/64-bit two's complements}. tree-sitter's
    integer_literal EXCLUDES the sign, so `-1073741670` used to read as 0x3FFFFF66
    and every negative NTSTATUS/HFILE_ERROR return was a false mismatch."""
    raw = src[node.start_byte : node.end_byte].decode("utf8", "replace")
    v = lit_value(raw)
    if v is None:
        return set()
    vals = {v}
    p = node.parent
    if p is not None and p.type == "unary_expression":
        ptxt = src[p.start_byte : p.end_byte].decode("utf8", "replace").lstrip()
        if ptxt.startswith("-"):
            vals.add((-v) & 0xFFFFFFFF)
            vals.add((-v) & 0xFFFFFFFFFFFFFFFF)
    return vals


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
    for seg in re.split(r"\|\|?", body):  # `A | B` (a mask) or `A || B` (alternatives)
        m = _NAME_SEG_RE.match(seg)
        if not m:
            return []  # any non-clean segment => treat whole comment as prose
        names.append(m.group(1))
    return names


# ── AST walk ─────────────────────────────────────────────────────────
_LANG = Language(tsr.language())
_PARSER = Parser(_LANG)
# One native query pulls every integer_literal + comment in a single pass — an
# order-of-magnitude cheaper than a Python-level recursive descent over the ~14M
# nodes across the workspace (the `collect` recursion was ~17s of the ~35s sweep).
_QUERY = Query(_LANG, "(integer_literal) @lit (line_comment) @c (block_comment) @c")


def collect(root):
    """(lits, comments) in source order, via the native tree-sitter query.

    Sorted by start_byte so the result is byte-identical to a pre-order DFS —
    the pairing logic below (and thus the finding order / baseline) is stable."""
    caps = QueryCursor(_QUERY).captures(root)
    lits = sorted(caps.get("lit", []), key=lambda n: n.start_byte)
    comments = sorted(caps.get("c", []), key=lambda n: n.start_byte)
    return lits, comments


def scan_file(path, catalog):
    src = path.read_bytes()
    root = _PARSER.parse(src).root_node
    lits, comments = collect(root)

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
        opener = False
        if line_src.startswith(("//", "/*")):
            # A comment alone on its line annotates either the lone literal BELOW it (a
            # block's value: `// DBG_CONTROL_C` / `0`, kernel32's console handler raised
            # exception code 0) or the block opener ABOVE it, as rustconv writes a C
            # condition: `if Type == 5 {` / `// LPC_CONNECTION_REQUEST` (se-rust's SRM
            # loop had three, and 5 is LPC_PORT_CLOSED).
            nxt = (
                lines[cline + 1].decode("utf8", "replace").strip() if cline + 1 < len(lines) else ""
            )
            prev = (
                lines[cline - 1].decode("utf8", "replace").split("//")[0].rstrip() if cline else ""
            )
            if _LONE_LIT_RE.match(nxt):
                cline += 1
                line_src = nxt
            elif prev.endswith("{"):
                cline -= 1
                line_src = prev.strip()
                opener = True
        if len(names) >= 2:
            # |-list: pair positionally with the literals on the comment's line,
            # in source order, that sit before the comment (e.g. `A | B // N1 | N2`).
            # ...and only within the comment's own FIELD: a struct-initializer line carries
            # several (`VSC_VK { Vsc: 0x10 as BYTE, Vk: 433 as USHORT /* VK_X | KBDEXT */ }`,
            # c_data_port's keyboard tables), and pairing the scan code 0x10 with VK_X
            # raised 4108 false findings. The field starts after the last `,`/`{`/`(`/`;`.
            # A comment alone under a block opener annotates that whole line instead.
            field_start = (
                -1
                if cline != c.start_point[0]
                else max(src.rfind(sep, 0, c.start_byte) for sep in (b",", b"{", b"(", b";"))
            )
            row = sorted(
                (
                    L
                    for L in lits
                    if L.start_point[0] == cline and field_start < L.start_byte < c.start_byte
                ),
                key=lambda n: n.start_byte,
            )
            if len(row) == 1 and len(names) >= 2 and all(n in catalog for n in names):
                # ONE literal annotated by an |-list is a composite annotation
                # (`const LPTR = 0x40; // LMEM_FIXED | LMEM_ZEROINIT`): compare
                # against the OR of the names, not name-by-name positionally.
                or_val = 0
                for n in names:
                    or_val |= catalog[n]
                composite_ok = or_val in lit_values(row[0], src)
                pairs = [] if composite_ok else list(zip(row, names, strict=False))
            elif len(row) == 1:
                # ONE literal, an |-list naming something the catalog does not know (kbd.h's
                # `VK_HOME | KNUMS`): the OR cannot be evaluated, and pairing the literal with
                # the first name alone is not what the comment says. Undecidable: no finding.
                pairs = []
            else:
                pairs = list(zip(row, names, strict=False))
        else:
            # single name: the adjacent literal — the closest one touching the comment
            # (trailing `0x4 /* N */`, or leading `/* N */ 0x4`), same line only.
            name = names[0]
            before = [L for L in lits if L.end_byte <= c.start_byte and L.start_point[0] == cline]
            after = [L for L in lits if L.start_byte >= c.end_byte and L.start_point[0] == cline]
            if opener:
                # `if f(..) == 0 {` / `// ERROR_X` names what the block DOES, not the 0:
                # a zero in an opener is a boolean test (rtl-heap's RtlFreeHeap check).
                before = [L for L in before if 0 not in lit_values(L, src)]
            side = before or after
            # If the correct value appears ANYWHERE among the line's candidates, the
            # flag is present and correct — don't be fooled by a nearer `0` in
            # `& FLAG) != 0 // FLAG` or `== 0`. Only the closest is the annotation.
            if name in catalog and any(catalog[name] in lit_values(L, src) for L in side):
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
            lvs = lit_values(lit, src)
            want = catalog[name]
            # Negative CATALOG values (HFILE_ERROR = -1) must match the same
            # two's-complement forms lit_values() produces for negative literals.
            wants = {want} if want >= 0 else {want, want & 0xFFFFFFFF, want & 0xFFFFFFFFFFFFFFFF}
            if not lvs or lvs & wants:
                continue
            lv = lit_value(txt(lit))
            # A single-BIT flag contained in a composite mask is a correct
            # annotation of one flag in an OR (GHND=0x42 /* GMEM_MOVEABLE */,
            # UNW version-and-flags 9 /* UNW_FLAG_EHANDLER */): the literal
            # includes the named bit. Enum-valued names (multi-bit or zero)
            # keep exact matching — containment is meaningless for them.
            if want and (want & (want - 1)) == 0 and lv is not None and lv & want == want:
                continue
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


# ── content-hash cache ───────────────────────────────────────────────
# The gate re-runs this on a near-unchanged tree every build; re-parsing all
# ~2400 .rs each time is wasteful. A file's findings are a pure function of
# (its bytes, the catalog, this script's logic), so we cache per content-hash
# and skip the tree-sitter parse+walk for any blob we've already scanned.
# Keyed by a "generation" fingerprint (catalog + this file's source) so a header
# edit or a logic change transparently invalidates. Failures degrade to a full
# scan — the cache is an optimization, never a correctness dependency.
_CACHE_VERSION = 1
_CACHE_PATH = REPO / "tools" / ".cache" / "scan_inline_flag_literals.json"
_CACHE_MAX_GENS = 4  # keep a few catalog generations (no-headers gate + ad-hoc --headers)


def _generation_fp(catalog) -> str:
    h = hashlib.sha1()
    h.update(Path(__file__).read_bytes())  # logic version
    h.update(repr(sorted(catalog.items())).encode())  # constant values
    return h.hexdigest()


def _load_cache(fp: str) -> dict:
    """Return the {blob_sha: findings} entries for generation `fp`, or {}."""
    try:
        blob = json.loads(_CACHE_PATH.read_text())
    except (OSError, ValueError):
        return {}
    if blob.get("v") != _CACHE_VERSION:
        return {}
    for gen in blob.get("gens", []):
        if gen.get("fp") == fp:
            return gen.get("entries", {})
    return {}


def _save_cache(fp: str, entries: dict) -> None:
    """Persist `entries` as generation `fp`, most-recent-first, LRU-capped.
    Atomic (temp+replace); any failure is swallowed (best-effort cache)."""
    try:
        blob = json.loads(_CACHE_PATH.read_text())
        if blob.get("v") != _CACHE_VERSION:
            blob = {}
    except (OSError, ValueError):
        blob = {}
    gens = [g for g in blob.get("gens", []) if g.get("fp") != fp]
    gens.insert(0, {"fp": fp, "entries": entries})
    out = {"v": _CACHE_VERSION, "gens": gens[:_CACHE_MAX_GENS]}
    try:
        _CACHE_PATH.parent.mkdir(parents=True, exist_ok=True)
        tmp = _CACHE_PATH.with_suffix(f".{os.getpid()}.tmp")
        tmp.write_text(json.dumps(out))
        os.replace(tmp, _CACHE_PATH)
    except OSError:
        pass


def main():
    args = sys.argv[1:]
    files, header_dirs, lint, no_cache, i = [], [], False, False, 0
    while i < len(args):
        if args[i] == "--headers":
            i += 1
            header_dirs.append(args[i])
        elif args[i] == "--lint":
            lint = True  # terse: only `file:line:` finding lines (for the CLI baseline gate)
        elif args[i] == "--no-cache":
            no_cache = True  # force a full re-scan (ignore the content-hash cache)
        else:
            files.append(args[i])
        i += 1
    full_sweep = not files
    if not files:
        # No paths given → sweep the whole Rust workspace (broad catalog, no
        # driver --headers). This is how the CLI lint gate invokes it.
        files = [str(p) for p in source_files(REPO / "sdk" / "lib" / "rust")]

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

    env_no_cache = os.environ.get("VIBEOS_SCAN_NO_CACHE", "").lower() in ("1", "true", "yes")
    no_cache = no_cache or env_no_cache
    fp = _generation_fp(catalog)
    cache = {} if no_cache else _load_cache(fp)
    seen: dict = {}  # every blob_sha encountered this run -> findings (for persistence)

    mismatches = 0
    for f in files:
        p = Path(f)
        if not p.exists() or "/tests/" in f or "/target/" in f:
            continue  # test fixtures use intentional non-canonical values
        try:
            raw = p.read_bytes()
        except OSError as e:
            print(f"{f}: read error: {e}", file=sys.stderr)
            continue
        sha = hashlib.sha1(raw).hexdigest()
        if sha in seen:
            hits = seen[sha]
        elif sha in cache:
            hits = cache[sha]
        else:
            try:
                hits = [list(t) for t in scan_file(p, catalog)]
            except Exception as e:
                print(f"{f}: parse error: {e}", file=sys.stderr)
                continue
        seen[sha] = hits
        for ln, lv, name, want, line_src in hits:
            print(
                f"{f}:{ln}: literal {hex(lv)} tagged /* {name} */ but {name} = {hex(want)} ({want})"
            )
            if not lint:  # terse mode emits ONLY finding lines (baseline-friendly)
                print(f"    {line_src}")
            mismatches += 1
    if not no_cache:
        # Full sweep → persist exactly this run's blobs (drops stale entries so the
        # cache can't grow without bound). Partial run (explicit files) → merge so a
        # single-file scan doesn't evict the gate's warm full-sweep generation.
        _save_cache(fp, seen if full_sweep else {**cache, **seen})
    if lint:
        return 1 if mismatches else 0
    if mismatches:
        print(f"\n{mismatches} inlined-literal mismatch(es) — fix to the header value.")
        return 1
    print("OK: no inlined flag-literal mismatches.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
