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

    # Items produced by a macro_rules! INVOCATION are invisible to a plain source scan,
    # and missing them is not cosmetic: oleaut32's debug_shims declares its whole
    # `debugstr_a/_w/_guid/...` family through ONE `debugstr_stub!(...)` call. Without
    # this the integrator did not know they existed, so it KEPT each module's local
    # `extern "C" { fn debugstr_w(); }` — and since the shims are plain Rust fns
    # (deliberately NOT `#[no_mangle]`), those decls resolved to no symbol and the DLL
    # failed to LINK. A compile-clean crate that doesn't link.
    names |= _macro_generated_names(text, path)

    # Per-arch bindgen bodies are pulled in by `include!` under a #[cfg]. A name provided on
    # ANY arch counts as provided: the union is the safe direction — the prelude will supply
    # it on the arch that has it, and dropping a decl the prelude provides is the whole point.
    for inc in _INCLUDE_RE.findall(text):
        names |= _harvest_file(path.parent / inc, _seen)

    return names


# `macro_rules! debugstr_stub { ($($name:ident),+ $(,)?) => {$( pub fn $name … )+}; }`
_RE_MACRO_RULES = re.compile(r"macro_rules!\s+([A-Za-z_]\w*)\s*\{")
_RE_FN_META = re.compile(r"\bfn\s+\$([A-Za-z_]\w*)\b")


def _macro_generated_names(text: str, path: Path) -> set[str]:
    """Names a `macro_rules!` emits as items, resolved through its invocations.

    Two shapes, because they resolve differently:

    * POSITIONAL — `impl_handle!(HFOO, size, marshal)`, each metavar mapping to one
      argument slot. Delegated to export_audit's expander, which already models this.
    * REPETITION — `debugstr_stub!(a, b, c)`: a single `$($x:ident),+` emitting
      `pub fn $x` per element, so EVERY argument is a generated name. The positional
      expander resolves only the first one here, which is worse than useless because it
      looks like it worked.
    """
    names: set[str] = set()

    try:  # positional
        from .export_audit import scan_macro_generated_fns

        names |= set(scan_macro_generated_fns(text, path))
    except Exception:  # a scan miss must never take integration down
        pass

    for m in _RE_MACRO_RULES.finditer(text):  # repetition
        mac = m.group(1)
        brace = text.index("{", m.end() - 1)
        depth, end = 0, brace
        for i in range(brace, len(text)):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    end = i
                    break
        body = text[brace : end + 1]
        metas = set(_RE_FN_META.findall(body))
        if len(metas) != 1:
            continue  # not the one-metavar list shape
        meta = metas.pop()
        if not re.search(rf"\$\(\s*\$\s*{re.escape(meta)}\s*:\s*ident", body):
            continue  # metavar isn't inside a repetition — that's the positional case
        for inv in re.finditer(rf"\b{re.escape(mac)}!\s*[(\[]", text):
            open_i = inv.end() - 1
            opc = text[open_i]
            clc = ")" if opc == "(" else "]"
            depth, close = 0, open_i
            for i in range(open_i, len(text)):
                if text[i] == opc:
                    depth += 1
                elif text[i] == clc:
                    depth -= 1
                    if depth == 0:
                        close = i
                        break
            for arg in text[open_i + 1 : close].split(","):
                arg = re.sub(r"//[^\n]*", "", arg).strip()
                if re.fullmatch(r"[A-Za-z_]\w*", arg):
                    names.add(arg)
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


_MACRO_CACHE: dict[Path, frozenset[str]] = {}


def prelude_macros(crate: Path) -> set[str]:
    """Names the crate's prelude provides as `macro_rules!`, not as functions.

    A C macro that MUST stay a macro in Rust — `MSVCRT_CHECK_PMT`, `HIBYTE`, `DPRINT` — is
    spelled `NAME(args)` in the C and therefore comes back from the model spelled
    `NAME(args)`, which is a plain call. That is the model reading the C correctly; which
    names happen to be macros on the Rust side is a property of the PRELUDE, invisible from
    the C, and so not the model's to know.
    """
    crate = Path(crate).resolve()
    if crate in _MACRO_CACHE:
        return set(_MACRO_CACHE[crate])
    prelude = crate / "src" / "prelude.rs"
    found: frozenset[str] = frozenset()
    if prelude.is_file():
        text = prelude.read_text(errors="replace")
        found = frozenset(_RE_MACRO_RULES.findall(text))
    _MACRO_CACHE[crate] = found
    return set(found)


#: `assembler.py` appends the whole translated `.c` as a trailing comment block under this
#: banner. It is the 1:1 reference that makes the NEXT translation of the file land as a
#: readable diff, so it must stay byte-identical to the C — a rewrite that reaches into it
#: turns `if(!MSVCRT_CHECK_PMT(x))` into Rust-flavoured pseudo-C and destroys exactly that.
#: Every other pass here edits `extern` blocks and `use` lines, which do not occur in the
#: block; a bare-name substitution is the first one that can.
_C_SOURCE_BANNER = "\n// ORIGINAL C SOURCE"


def fix_macro_calls(src: str, macros: set[str], defined: set[str]) -> tuple[str, list[str]]:
    """Turn `NAME(args)` into `NAME!(args)` for every prelude macro. Returns (src, fixed).

    Skips a name this file DEFINES as a real function: the prelude's macro would be shadowed
    by the local item, so the plain call is already correct and adding `!` would break it.
    Already-correct `NAME!(…)` sites are left alone, which is what makes this idempotent.
    Never touches the appended original-C block — see `_C_SOURCE_BANNER`.
    """
    cut = src.find(_C_SOURCE_BANNER)
    if cut != -1:
        head, tail = src[:cut], src[cut:]
        head, fixed = fix_macro_calls(head, macros, defined)
        return head + tail, fixed

    fixed: list[str] = []
    for name in sorted(macros - defined):
        # `(?!\s*!)` skips a call that is already a macro invocation — that alone makes this
        # idempotent. The lookbehind is only `\w`, to stay off `other_NAME(`: excluding a
        # preceding `!` too would silently skip `if !NAME(x)`, which is how C spells a
        # boolean test and is 17 of mbcs.c's 42 `MSVCRT_CHECK_PMT` sites.
        pattern = re.compile(rf"(?<!\w){re.escape(name)}(?!\s*!)\s*\(")
        src, count = pattern.subn(f"{name}!(", src)
        if count:
            fixed.append(f"{name}×{count}")
    return src, fixed


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
    macro_calls_fixed: list[str] = field(default_factory=list)  # `NAME(x)` -> `NAME!(x)`
    blocks_removed: int = 0

    @property
    def changed(self) -> bool:
        return bool(
            self.substrate_swapped
            or self.prelude_added
            or self.dropped_provided
            or self.dropped_self
            or self.promoted_pub
            or self.macro_calls_fixed
        )

    def summary(self) -> str:
        return (
            f"{self.module or '<mem>'}: "
            f"-{len(self.dropped_provided)} prelude-provided  "
            f"-{len(self.dropped_self)} self-decl  "
            f"={len(self.kept_statics)} data-static  "
            f"?{len(self.kept_unprovided)} unprovided  "
            f"+{len(self.promoted_pub)} pub"
            + (f"  !{','.join(self.macro_calls_fixed)}" if self.macro_calls_fixed else "")
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

    # A prelude macro is spelled as a plain call in the C, so it comes back as a plain call.
    # Which names are macros is a fact about the PRELUDE, not about the C — see prelude_macros.
    src, report.macro_calls_fixed = fix_macro_calls(src, prelude_macros(crate), defined)

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


# --------------------------------------------------------------------------------------
# In-crate import resolution.
# --------------------------------------------------------------------------------------

#: Never indexed as an import source: the prelude and the bindgen islands are already
#: glob-imported by every translated module, and they are enormous — a single island
#: re-export line runs to tens of kilobytes — so scanning them is pure cost.
_UNINDEXED_STEMS = frozenset({"prelude", "types", "lib"})
_UNINDEXED_PREFIXES = ("island_", "bindgen_", "hal_bindgen_", "types_")


def _is_island(rs: Path) -> bool:
    """True for a generated per-arch bindgen island, which is glob-imported, not imported."""
    return rs.stem.startswith(_UNINDEXED_PREFIXES)


#: Items at column 0 — i.e. genuinely importable module-level items. `defined_names` anchors
#: at `^\s*`, which also matches an INHERENT METHOD inside an `impl` block, and those are not
#: importable at all: indexing recyclebin_support.rs that way proposed
#: `use …::recyclebin_support::len;` for `CStringW::len`. rustfmt puts every top-level item at
#: column 0, so the column is an exact test and needs no brace matching.
_TOP_LEVEL_PUB_RE = re.compile(
    r"^pub\s+(?:unsafe\s+)?(?:extern\s+\"[A-Za-z0-9_-]+\"\s+)?"
    r"(fn|struct|enum|union|trait|type|const|static)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)",
    re.M,
)

#: Never imported ACROSS modules, only ever provided by the island or the prelude. A constant
#: reaching a sibling module is a substrate gap — `GENERIC_WRITE` living in a class module
#: because the island lacks it — and cross-importing it would entrench the duplicate instead of
#: reporting it. Excluding them also kills a self-import: a wide-literal static
#: (`pub static CPP_COM_WSTR_…: [WCHAR; 4] = …`) is invisible to `defined_names`, whose
#: static/const pattern is `:[^;]*=` and so cannot span the `;` inside an array type, and the
#: file was importing its own definition from itself.
_NEVER_IMPORTED_KINDS = frozenset({"const", "static"})


def _top_level_pub_items(text: str) -> set[tuple[str, str]]:
    """(kind, name) for `pub` items at column 0, outside any `extern` block."""
    return set(_TOP_LEVEL_PUB_RE.findall(_outside_extern(text)))


def _top_level_pub_names(text: str) -> set[str]:
    """`pub` items declared at column 0, outside any `extern` block."""
    return {name for _, name in _top_level_pub_items(text)}


def island_names(crate_src: Path) -> set[str]:
    """Every name the crate's bindgen islands export.

    These have to be treated as PROVIDED, not as import candidates, and `provided_names`
    cannot see them: a crate's `types.rs` selects its island with
    `#[cfg(...)] #[path = "types_i386.rs"] mod types_arch;` + `pub use types_arch::*;`, and
    that `#[path]` indirection defeats `_resolve_path`, which resolves module paths on
    disk by name. recyclebin-rust reports 61 provided names as a result, missing every
    island type. Harvesting the island files directly is exact and needs no path guessing.
    """
    names: set[str] = set()
    for rs in sorted(Path(crate_src).rglob("*.rs")):
        if not _is_island(rs):
            continue
        try:
            text = rs.read_text()
        except OSError:
            continue
        # `extern_declared` too: an island states most of its FUNCTIONS inside
        # `unsafe extern "C" { pub fn X(); }`, which `defined_names` deliberately skips.
        # Without them a sibling's same-named `pub fn` would look like the only owner.
        names |= _top_level_pub_names(text) | extern_declared(text)
    return names


_PATH_MOD_RE = re.compile(
    r"#\[path\s*=\s*\"(?P<file>[^\"]+)\"\s*\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(?P<name>\w+)\s*;"
)


def _path_attr_modules(crate_src: Path) -> dict[Path, str]:
    """{resolved file -> `crate::NAME`} for every `#[path = "..."] pub mod NAME;`.

    A module's Rust path is what a `mod` declaration BINDS it to, which for a
    `#[path]`-mapped file has nothing to do with where the file sits. shell32 maps
    `dialogs/drvdefext.rs` to `crate::drv_def_ext` and `shell32/CDefView.rs` to
    `crate::def_view`, both so a rustconv re-run can keep assembling at the C tree's path
    while the crate keeps its own naming. Deriving the module from the FILE PATH emits
    `use crate::dialogs::drvdefext::ShellObjectCreator;` — a module that does not exist.
    It is a compile error rather than a silent wrong call, but it is one the generator
    should not be producing, and every `#[path]` module in the crate is exposed to it.

    Only the declaring file's own prefix is applied (lib.rs -> `crate`), which covers every
    `#[path]` in this tree; a `#[path]` inside a nested `mod.rs` keeps the path-derived
    answer rather than being guessed at.
    """
    mapped: dict[Path, str] = {}
    root = Path(crate_src)
    for decl in (root / "lib.rs", root / "main.rs"):
        try:
            text = decl.read_text()
        except OSError:
            continue
        for match in _PATH_MOD_RE.finditer(text):
            target = (root / match.group("file")).resolve()
            mapped[target] = f"crate::{match.group('name')}"
    return mapped


def _module_path(rs: Path, crate_src: Path) -> str | None:
    """`crate::a::b` for a file under the crate's `src/`; None if it lies outside.

    A `#[path]`-mapped file takes the name its `mod` declaration binds, not its location.
    """
    try:
        rel = rs.resolve().relative_to(crate_src.resolve())
    except ValueError:
        return None
    mapped = _path_attr_modules(crate_src).get(rs.resolve())
    if mapped is not None:
        return mapped
    parts = rel.with_suffix("").parts
    if parts and parts[-1] == "mod":
        parts = parts[:-1]
    return "::".join(("crate", *parts)) if parts else None


def index_crate_exports(crate_src: Path) -> dict[str, set[str]]:
    """name → the set of `crate::…` modules that define it `pub`."""
    owners: dict[str, set[str]] = {}
    for rs in sorted(Path(crate_src).rglob("*.rs")):
        if rs.stem in _UNINDEXED_STEMS or rs.stem.startswith(_UNINDEXED_PREFIXES):
            continue
        module = _module_path(rs, Path(crate_src))
        if module is None:
            continue
        try:
            text = rs.read_text()
        except OSError:
            continue
        # A file no `mod` chain reaches is not importable, and indexing one is worse than
        # useless: recyclebin's superseded whole-TU files still define RecycleBin5_Constructor,
        # so they made the live definition look AMBIGUOUS and suppressed the import, while
        # RecycleBin5Enum_Init resolved to the orphan alone and would have produced a `use`
        # of a module that is not declared. `scan_orphan_modules` guarantees every other file
        # is reachable, which makes this banner an exact test for reachability.
        if "ORPHAN-BY-DESIGN" in text[:2000]:
            continue
        for kind, name in _top_level_pub_items(text):
            if kind in _NEVER_IMPORTED_KINDS:
                continue
            owners.setdefault(name, set()).add(module)
    return owners


def _without_comments(code: str) -> str:
    """`code` with Rust line and block comments blanked out.

    Only for scans that ask "which names does this file USE" — a name in prose is not a use.
    Newlines are preserved so any line-based reasoning downstream still lines up. Doc comments
    (`///`, `//!`) go too: they are the most prose-dense part of a generated file and the most
    likely to name a type the module does not actually reference.
    """
    code = re.sub(r"/\*.*?\*/", lambda m: "\n" * m.group(0).count("\n"), code, flags=re.S)
    return re.sub(r"//[^\n]*", "", code)


def resolve_in_crate_imports(code: str, crate: Path) -> tuple[str, list[str]]:
    """Emit `use crate::…::<name>;` for each name `code` references but does not define.

    Splitting a C++ TU into one module per class turns intra-TU references into
    cross-module ones, and nothing deterministic puts the `use` there: today the only
    thing producing these lines is a PROMPT HINT, i.e. the model writes them. That
    violates the substrate-closure rule that imports are derived, never paid for — and,
    worse, it cannot repair a body that is already banked, because prompt prose is out of
    the resume-cache key, so a prompt fix leaves cached output exactly as broken.

    Names the prelude already provides are SKIPPED. That is the safety property, not an
    optimization: an explicit `use` shadows a glob import, so importing a sibling's
    `pub fn Foo` when the island also exports `Foo` would silently rebind every call in
    the file. Restricting to names the prelude does not supply makes this strictly
    additive — it can only resolve a name that resolves to nothing today.

    Ambiguous names (two sibling modules define the same one) are REPORTED and skipped
    rather than guessed at.

    Order caveat: the index reflects the files that exist right now, so generating a
    whole crate from scratch is order-sensitive — a module written before its sibling
    cannot import from it. It is additive and converges on a second pass, and the failure
    mode is an unresolved-name compile error, never a wrong-symbol bind.
    """
    crate = Path(crate)
    crate_src = crate / "src"
    if not crate_src.is_dir():
        return code, []

    owners = index_crate_exports(crate_src)
    if not owners:
        return code, []

    local = defined_names(code)
    already = set(re.findall(r"^\s*use\s+[A-Za-z0-9_:]+::([A-Za-z_][A-Za-z0-9_]*)\s*;", code, re.M))
    provided = provided_names(crate) | island_names(crate_src)
    # COMMENTS ARE NOT CODE. Scanning the raw text meant a name that appears only in prose
    # got a `use` line generated for it — `enum_merged_folder.rs` mentions
    # "shellutils.h ShellObjectCreator<T>" in a comment explaining its activation, and the
    # assembled file came out with `use crate::drv_def_ext::ShellObjectCreator;`, a module
    # that does not exist in that crate. The failure is loud (unresolved import) but it is
    # generated, so it cannot be fixed in the file — only here.
    words = set(re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\b", _without_comments(code)))

    notes: list[str] = []
    imports: list[str] = []
    for name in sorted(words & owners.keys()):
        if name in local or name in already or name in provided:
            continue
        mods = owners[name]
        if len(mods) > 1:
            notes.append(f"{name}: defined in {', '.join(sorted(mods))} — ambiguous, not imported")
            continue
        imports.append(f"use {next(iter(mods))}::{name};")

    if not imports:
        return code, notes

    block = "\n".join(imports)
    anchor = re.search(r"^use crate::prelude::\*;\n", code, re.M)
    if anchor:
        code = code[: anchor.end()] + block + "\n" + code[anchor.end() :]
    else:
        code = block + "\n" + code
    return code, notes


#: A top-level item that RE-DECLARES a type the crate already owns. `impl` is included
#: because the duplicate carries its methods with it.
_TOP_LEVEL_ITEM_RE = re.compile(
    r"^(?:pub\s+)?(?:struct|enum|union|type)\s+([A-Za-z_][A-Za-z0-9_]*)"
    r"|^impl(?:\s*<[^>]*>)?\s+(?:[A-Za-z0-9_:<>, ]+\s+for\s+)?([A-Za-z_][A-Za-z0-9_]*)"
)


def strip_top_level_items(rust: str, names: set[str]) -> tuple[str, list[str]]:
    """Drop top-level definitions (and `impl` blocks) of `names`, keeping everything else.

    The free-functions path translates the TU's file-scope preamble, and for a C++ class
    target that preamble carries declarations of types the crate ALREADY owns — the
    `--local-raii` lowering above all. You then get both `use …::CStringW;` and a second
    `pub struct CStringW` in one module, which is E0255, and if it compiled it would be
    worse: two distinct Rust types for one C++ class. `cpp_com` mode dodges this by
    skipping the preamble entirely; the free half cannot, because it needs the rest of it.

    Anchored at column 0 so a nested item is never touched.
    """
    if not names:
        return rust, []
    kept: list[str] = []
    dropped: list[str] = []
    lines = rust.splitlines()
    i = 0
    while i < len(lines):
        match = _TOP_LEVEL_ITEM_RE.match(lines[i])
        name = (match.group(1) or match.group(2)) if match else None
        if name not in names:
            kept.append(lines[i])
            i += 1
            continue
        # Carry any attribute/doc lines that were emitted immediately above it.
        while kept and re.match(r"^\s*(?:#\[|///|//!)", kept[-1]):
            kept.pop()
        dropped.append(name)
        depth = 0
        while i < len(lines):
            depth += lines[i].count("{") - lines[i].count("}")
            ends = depth <= 0 and (lines[i].rstrip().endswith((";", "}")) or "{" not in lines[i])
            i += 1
            if ends:
                break
    return "\n".join(kept), dropped
