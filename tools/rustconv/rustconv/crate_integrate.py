"""crate_integrate — the last mile of a translation: land a self-contained rustconv
file onto its target crate's shared *prelude*.

rustconv translates every C file in isolation, so it emits a self-contained module:
a generic substrate glob (`use ntdll_sys::nt::*;`) plus a per-file `extern "C" { ... }`
block hand-declaring everything the C called. Both are wrong once the file is a module
of a real DLL crate:

  1. **Wrong import surface.** The crate's other modules `use crate::prelude::*;` — the
     prelude globs the bindgen sys crate (com-sys), the crate's own shared modules, its
     macros. Without the swap you get a wall of `cannot find type HRESULT`.

  2. **Redeclared externs with the WRONG ABI — the dangerous one.** The file re-declares
     `HeapAlloc`/`HeapFree`/... as `extern "C"`. WINAPI is `__stdcall` on i386 (`_Foo@N`),
     so a C-ABI decl asks for a symbol that does not exist: links fine on amd64/arm64,
     breaks i386. And because an *item* in the module shadows a glob import, the local
     decl silently wins over the prelude's correct `extern "system"` one. So every decl
     the prelude already provides must be DROPPED, not kept.

Everything else in an extern block is KEPT and REPORTED, so an unrecognized symbol can't
slip through as a silent undefined-at-link surprise: each one is either a real gap in the
prelude (add it there, with the right ABI) or a name the model invented.

Generalized from the one-off `tools/integrate_ole32_ring.py`. The one hardcoded thing in
that script — a literal `PRELUDE_FNS` set — is here *computed* from the crate's actual
`src/prelude.rs` (see `provided_names`), because a hardcoded list rots the moment the
prelude grows a symbol.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

# --------------------------------------------------------------------------------------
# Regexes. Two of these encode bugs that cost real debugging time in the ole32 run — the
# comments are load-bearing, do not "simplify" them away.
# --------------------------------------------------------------------------------------

# re.M matters: split_decls glues any preceding comment lines onto the decl, so a
# `^`-anchored non-multiline pattern silently fails to classify every commented decl —
# and an unclassified decl is KEPT, which is the failure direction that hurts (a stale
# `extern "C"` decl shadows the prelude and breaks i386). Comment lines are stripped
# before matching so a `//`-quoted signature can't false-match either.
FN_DECL_RE = re.compile(r"^\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", re.M)
STATIC_DECL_RE = re.compile(
    r"^\s*(?:pub\s+)?static\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:", re.M
)
COMMENT_LINE_RE = re.compile(r"^\s*//.*$", re.M)

# The ABI string class is `[A-Za-z0-9_-]+`, NOT `[a-z]+`: the overwhelmingly common
# spelling here is `extern "C"` — uppercase. A lowercase-only class silently fails to see
# every `pub unsafe extern "C" fn`, i.e. nearly every definition in the crate, which makes
# the "already provided" check quietly useless. (It was: that class matched exactly one
# name in ole32's dictionary.rs.)
ABI = r'"[A-Za-z0-9_-]+"'

# A DEFINITION has a body: `... fn name(...) ... {`. An extern DECL ends in `;`. Matching
# on the trailing `{` is what separates them — critical, because the two look identical up
# to the parameter list, and misreading a decl as a definition would silently delete a
# genuinely-external symbol.
DEFINED_FN_RE = re.compile(
    rf"^\s*(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+{ABI}\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"
    r"[^;{]*\{",
    re.M | re.S,
)

# `extern "<abi>" {` — the `{` must follow the ABI string, which is what distinguishes a
# block from a *type* (`x as unsafe extern "C" fn(..) -> HRESULT`) or a definition.
EXTERN_BLOCK_RE = re.compile(rf"extern\s+({ABI})\s*\{{")

# Substrate globs rustconv emits (the generic sys-crate surface), to be swapped for the
# crate prelude. `ntdll_sys::nt`, `ntkrnl_sys::ddk`, `win32k_sys::*`, …
SUBSTRATE_USE_RE = re.compile(
    r"^use\s+(?:ntdll_sys|ntkrnl_sys|win32k_sys|winsrv_sys|winlogon_sys)"
    r"(?:::[A-Za-z0-9_]+)*::\*\s*;[ \t]*\n?",
    re.M,
)
PRELUDE_USE = "use crate::prelude::*;"


def decl_code(d: str) -> str:
    """A decl with its comment lines removed — what the classifiers match against."""
    return COMMENT_LINE_RE.sub("", d)


def split_decls(block: str) -> list[str]:
    """Split an extern-block body into individual decls (a decl may span lines)."""
    decls, cur, depth = [], [], 0
    for line in block.splitlines():
        cur.append(line)
        depth += line.count("(") - line.count(")")
        if depth <= 0 and line.rstrip().endswith(";"):
            decls.append("\n".join(cur))
            cur, depth = [], 0
    if cur and "".join(cur).strip():
        decls.append("\n".join(cur))
    return decls


def extern_block_spans(src: str) -> list[tuple[int, int, str, str]]:
    """-> [(start, end, body, abi)] for each `extern "<abi>" { ... }` block.

    `abi` is carried so a rewritten block keeps its own ABI (rebuilding every block as
    `extern "C"` would corrupt an `extern "system"` one — an i386 decoration bug of
    exactly the class this module exists to prevent).

    Matches inside a `//` comment are skipped: `assemble_file` appends the original C
    source as trailing comments, and a C++-guarded header quoted there really does
    contain `extern "C" {`.
    """
    spans = []
    for m in EXTERN_BLOCK_RE.finditer(src):
        line_start = src.rfind("\n", 0, m.start()) + 1
        if "//" in src[line_start : m.start()]:
            continue
        brace = src.index("{", m.end() - 1)
        i, depth = brace, 0
        while i < len(src):
            if src[i] == "{":
                depth += 1
            elif src[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        spans.append((m.start(), i + 1, src[brace + 1 : i], m.group(1)))
    return spans


def _outside_extern(src: str) -> str:
    """The source with every extern block excised — so a decl can never be mistaken for
    a definition of itself."""
    out, prev = [], 0
    for s, e, _, _ in extern_block_spans(src):
        out.append(src[prev:s])
        prev = e
    out.append(src[prev:])
    return "".join(out)


def defined_names(src: str, pub_only: bool = False) -> set[str]:
    """Names this source DEFINES (fns, statics, consts, types, macros), counting only text
    OUTSIDE extern blocks.

    `pub_only` matters for the cross-module ("provided by the prelude") query: a private
    `fn` is NOT re-exported by a glob, so counting it would make us drop a decl that then
    resolves to nothing. It must stay False for the self-decl query, where a file's own
    private definitions are exactly what we're looking for.
    """
    body = _outside_extern(src)
    pub = r"pub\s+" if pub_only else r"(?:pub\s+)?"
    names = set(
        re.findall(
            rf"^\s*{pub}(?:unsafe\s+)?(?:extern\s+{ABI}\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"
            r"[^;{]*\{",
            body,
            re.M | re.S,
        )
    )
    names |= set(
        re.findall(
            rf"^\s*{pub}(?:static|const)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:[^;]*=",
            body,
            re.M,
        )
    )
    names |= set(
        re.findall(
            rf"^\s*{pub}(?:struct|enum|union|trait|type)\s+([A-Za-z_][A-Za-z0-9_]*)", body, re.M
        )
    )
    # #[macro_export] macros are crate-root items and always reachable.
    names |= set(re.findall(r"^\s*macro_rules!\s+([A-Za-z_][A-Za-z0-9_]*)", body, re.M))
    return names


def extern_declared(src: str) -> set[str]:
    """Names this source DECLARES in an extern block (i.e. expects from elsewhere)."""
    out = set()
    for _, _, body, _ in extern_block_spans(src):
        for d in split_decls(body):
            code = decl_code(d)
            m = FN_DECL_RE.search(code) or STATIC_DECL_RE.search(code)
            if m:
                out.add(m.group(1))
    return out


def promote_pub(src: str, names: set[str]) -> tuple[str, list[str]]:
    """Make `pub` any definition of a name that some OTHER translation unit declares.

    These are the C's `DECLSPEC_HIDDEN` cross-TU helpers: non-`static` in C, so shared
    between TUs, but rustconv — translating one file in isolation — has no reason to make
    them `pub`. Left module-private, the prelude's glob cannot re-export them, and the
    sibling that calls them is left with an unresolved name (or, worse, keeps its own
    `extern "C"` decl and links against the wrong i386 decoration).
    """
    # Only DEFINITIONS (a body: `... fn n(..) {`), and only OUTSIDE extern blocks — inside
    # one, `fn n(..);` is a decl about to be stripped, and "promoting" it would just make
    # the report lie about what was actually defined here. (The first cut of this in the
    # ole32 script over-fired and promoted decls inside extern blocks, including FAILED.)
    spans = extern_block_spans(src)

    def in_extern(pos: int) -> bool:
        return any(s <= pos < e for s, e, _, _ in spans)

    promoted = []
    for n in sorted(names):
        pat = re.compile(
            rf"^(\s*)((?:unsafe\s+)?(?:extern\s+{ABI}\s+)?fn\s+{re.escape(n)}\s*\([^;{{]*\{{)",
            re.M | re.S,
        )
        hits = [m for m in pat.finditer(src) if not in_extern(m.start())]
        if not hits:
            continue
        for m in reversed(hits):  # reversed: earlier offsets stay valid as we splice
            src = src[: m.start()] + m.group(1) + "pub " + m.group(2) + src[m.end() :]
        promoted.append(n)
        spans = extern_block_spans(src)  # offsets shifted
    return src, promoted


# --------------------------------------------------------------------------------------
# provided_names — computed from the crate's real prelude, never hardcoded.
# --------------------------------------------------------------------------------------

_PROVIDED_CACHE: dict[Path, frozenset[str]] = {}

_GLOB_USE_RE = re.compile(r"^\s*pub\s+use\s+([A-Za-z0-9_:]+?)\s*::\s*\*\s*;", re.M)
_LIST_USE_RE = re.compile(r"^\s*pub\s+use\s+([A-Za-z0-9_:]+?)\s*::\s*\{([^}]*)\}\s*;", re.M)
_ITEM_USE_RE = re.compile(r"^\s*pub\s+use\s+([A-Za-z0-9_:]+)\s*;", re.M)
_INCLUDE_RE = re.compile(r'include!\s*\(\s*"([^"]+)"\s*\)')


def _harvest_file(path: Path, _seen: set[Path] | None = None) -> set[str]:
    """Public names a Rust source file surfaces: `pub` items + `pub fn`/`pub static` inside
    its extern blocks, plus whatever it `include!`s.

    This is a name scan, not a Rust parse — deliberately. The bindgen files it has to chew
    through are ~19k lines each and we only ever ask "is this identifier reachable?".
    """
    _seen = _seen if _seen is not None else set()
    path = path.resolve()
    if path in _seen or not path.is_file():
        return set()
    _seen.add(path)

    text = path.read_text(errors="replace")
    names = defined_names(text, pub_only=True)
    for _, _, body, _ in extern_block_spans(text):
        for d in split_decls(body):
            code = decl_code(d)
            m = re.search(r"^\s*pub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)", code, re.M) or re.search(
                r"^\s*pub\s+static\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)", code, re.M
            )
            if m:
                names.add(m.group(1))

    # Per-arch bindgen bodies are pulled in by `include!` under a #[cfg]. A name provided on
    # ANY arch counts as provided: the union is the safe direction — the prelude will supply
    # it on the arch that has it, and dropping a decl the prelude provides is the whole point.
    for inc in _INCLUDE_RE.findall(text):
        names |= _harvest_file(path.parent / inc, _seen)

    return names


def _resolve_path(rust_path: str, crate: Path) -> list[Path]:
    """Resolve a `use` path prefix (`crate::hash`, `com_sys`, `com_sys::foo`) to the file(s)
    that define it. Returns [] if it isn't a module/crate we can find on disk (e.g.
    `crate::stdmethod`, which names a macro, not a module — the caller falls back to
    treating the last segment as a plain item name)."""
    segs = rust_path.split("::")
    if segs[0] in ("crate", "self"):
        root = crate / "src"
        segs = segs[1:]
    else:
        # A sibling workspace crate: `com_sys` → sdk/lib/rust/com-sys/.
        workspace = crate.parent
        for cand in (segs[0].replace("_", "-"), segs[0]):
            if (workspace / cand / "src" / "lib.rs").is_file():
                root = workspace / cand / "src"
                break
        else:
            return []
        segs = segs[1:]
        if not segs:
            return [root / "lib.rs"]

    if not segs:
        return []
    p = root.joinpath(*segs)
    out = [c for c in (p.with_suffix(".rs"), p / "mod.rs") if c.is_file()]
    return out


def provided_names(crate: Path) -> set[str]:
    """Every name reachable in a module that does `use crate::prelude::*;`.

    Computed from the crate's actual `src/prelude.rs`:
      * items the prelude itself defines (`pub type LPVARIANT = ...`) and DECLARES in its
        own `extern "..." { pub fn X(); }` blocks — the correctly-ABI'd Win32 surface;
      * every `pub use <path>::*;` glob, resolved on disk (a crate module `src/<m>.rs` or
        `src/<m>/mod.rs`, or a sibling workspace crate like `com_sys`) and name-scanned;
      * every explicit re-export (`pub use crate::{TRACE, WARN};`, `pub use x::Y;`).

    Empty set if the crate has no prelude — the caller must then leave the file alone
    rather than invent one.
    """
    crate = crate.resolve()
    if crate in _PROVIDED_CACHE:
        return set(_PROVIDED_CACHE[crate])

    prelude = crate / "src" / "prelude.rs"
    if not prelude.is_file():
        _PROVIDED_CACHE[crate] = frozenset()
        return set()

    text = prelude.read_text(errors="replace")
    names = _harvest_file(prelude)

    for path in _GLOB_USE_RE.findall(text):
        for f in _resolve_path(path, crate):
            names |= _harvest_file(f)

    for _path, items in _LIST_USE_RE.findall(text):
        for item in items.split(","):
            item = item.strip()
            if not item or item == "*":
                continue
            # `Foo as Bar` re-exports under Bar.
            names.add(item.split()[-1] if " as " in item else item)

    for path in _ITEM_USE_RE.findall(text):
        seg = path.split("::")[-1]
        if seg not in ("*",):
            names.add(seg)

    _PROVIDED_CACHE[crate] = frozenset(names)
    return set(names)


# --------------------------------------------------------------------------------------
# integrate
# --------------------------------------------------------------------------------------


@dataclass
class Report:
    """What integrate() did — the `kept_unprovided` list is the interesting one: each name
    there is either a real gap in the prelude or a symbol the model invented."""

    module: str = ""
    substrate_swapped: bool = False
    prelude_added: bool = False
    dropped_provided: list[str] = field(default_factory=list)  # prelude already has it
    dropped_self: list[str] = field(default_factory=list)  # this file defines it
    kept_unprovided: list[str] = field(default_factory=list)  # REPORT: verify each
    kept_statics: list[str] = field(default_factory=list)  # e.g. IID_* link symbols
    promoted_pub: list[str] = field(default_factory=list)
    blocks_removed: int = 0

    @property
    def changed(self) -> bool:
        return bool(
            self.substrate_swapped
            or self.prelude_added
            or self.dropped_provided
            or self.dropped_self
            or self.promoted_pub
        )

    def summary(self) -> str:
        return (
            f"{self.module or '<mem>'}: "
            f"-{len(self.dropped_provided)} prelude-provided  "
            f"-{len(self.dropped_self)} self-decl  "
            f"={len(self.kept_statics)} data-static  "
            f"?{len(self.kept_unprovided)} unprovided  "
            f"+{len(self.promoted_pub)} pub"
        )


def _swap_substrate(src: str, report: Report) -> str:
    """Replace the first substrate glob with the prelude glob and delete the rest."""
    if PRELUDE_USE in src:
        # Idempotent: already integrated. Still drop any lingering substrate glob.
        new, n = SUBSTRATE_USE_RE.subn("", src)
        report.substrate_swapped = n > 0
        return new

    matches = list(SUBSTRATE_USE_RE.finditer(src))
    if matches:
        src = SUBSTRATE_USE_RE.sub("", src)
        first = matches[0].start()
        src = src[:first] + PRELUDE_USE + "\n" + src[first:]
        report.substrate_swapped = True
        return src

    # No substrate glob at all (a hand-written or already-half-integrated file): insert the
    # prelude after the leading inner-attributes / `//!` docs, which must stay first.
    lines = src.splitlines(keepends=True)
    i = 0
    while i < len(lines) and (
        not lines[i].strip()
        or lines[i].lstrip().startswith(("//!", "#!["))
        or lines[i].lstrip().startswith("//")
    ):
        i += 1
    lines.insert(i, PRELUDE_USE + "\n")
    report.prelude_added = True
    return "".join(lines)


def integrate(
    src: str,
    crate: Path,
    *,
    module: str = "",
    provided: set[str] | None = None,
) -> tuple[str, Report]:
    """Land one translated file onto `crate`'s prelude. Idempotent.

    * swaps the substrate glob for `use crate::prelude::*;` — but ONLY if the crate really
      has a `src/prelude.rs`. No prelude → return the source untouched (fail-safe: never
      invent an import surface that doesn't exist).
    * drops every extern decl whose name the prelude already provides (WRONG-ABI class) or
      that this file itself defines (E0428 duplicate class), deleting blocks left empty.
    * keeps + REPORTS everything else.
    * promotes to `pub` the definitions that sibling modules declare as extern (the C's
      DECLSPEC_HIDDEN cross-TU helpers).

    `provided` may be passed to override the computed set (batch integration: a name defined
    by another file landing in the same pass is provided too).
    """
    report = Report(module=module)
    crate = Path(crate)
    if not (crate / "src" / "prelude.rs").is_file():
        return src, report  # not a prelude crate — leave it alone.

    if provided is None:
        provided = provided_names(crate)

    src = _swap_substrate(src, report)

    # Cross-TU helper promotion: a name that some OTHER module in the crate declares extern
    # must be `pub` where it's defined. Exclude anything the prelude already provides —
    # rustconv sometimes emits a local copy of a shared helper, and promoting that copy puts
    # two same-named items in scope through two globs (E0659 ambiguity). Left private, the
    # local copy just shadows the glob inside its own file, which is harmless.
    wanted: set[str] = set()
    for rs in sorted((crate / "src").rglob("*.rs")):
        if module and rs.stem == module:
            continue  # a previous run's copy of the file we're re-integrating
        if rs.name == "prelude.rs":
            continue
        wanted |= extern_declared(rs.read_text(errors="replace"))
    wanted -= provided
    src, report.promoted_pub = promote_pub(src, wanted)

    # Functions/statics THIS file defines — computed outside the extern blocks. rustconv
    # translates each function independently, so a caller translated before its callee can
    # get an `extern` decl for something defined in the very same file: a hard E0428.
    defined = defined_names(src, pub_only=False)

    out, pos = [], 0
    for start, end, body, abi in extern_block_spans(src):
        keep = []
        for d in split_decls(body):
            # split_decls hands a decl its preceding blank/comment lines. Blank ones are
            # stripped at the edges so a re-run can't keep re-inflating the block (that is
            # what idempotency means here); the comment lines ride along with the decl.
            d = d.strip("\n")
            code = decl_code(d)
            fm, sm = FN_DECL_RE.search(code), STATIC_DECL_RE.search(code)
            m = fm or sm
            if m is None:
                if d.strip():
                    keep.append(d)  # unclassifiable (a type alias?) — never silently drop
                continue
            name = m.group(1)
            if name in defined:
                report.dropped_self.append(name)
            elif name in provided:
                report.dropped_provided.append(name)
            elif sm:
                # A data static (IID_*/CLSID_*, …) the prelude doesn't have: a genuine
                # link-time symbol. Keep it — redefining it in Rust is a silent-corruption
                # source — but say so.
                report.kept_statics.append(name)
                keep.append(d)
            else:
                report.kept_unprovided.append(name)
                keep.append(d)

        out.append(src[pos:start])
        if keep:
            out.append(f"extern {abi} {{\n" + "\n".join(keep) + "\n}")
        else:
            report.blocks_removed += 1
            # Eat the newline the now-deleted block would have left behind.
            while end < len(src) and src[end] == "\n":
                end += 1
        pos = end
    out.append(src[pos:])
    return "".join(out), report


def integrate_path(path: Path, crate: Path) -> Report:
    """In-place `integrate` of a file already sitting in the crate."""
    path = Path(path)
    new, report = integrate(path.read_text(), crate, module=path.stem)
    if new != path.read_text():
        path.write_text(new)
    return report
