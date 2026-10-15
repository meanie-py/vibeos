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

from . import variadic_shim

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
# `unsafe` is INSIDE the match so a dropped block takes it along: matched from `extern`, a
# removed `unsafe extern "C" {...}` left a bare `unsafe` line, and a rewritten one doubled it.
EXTERN_BLOCK_RE = re.compile(rf"(?:unsafe\s+)?extern\s+({ABI})\s*\{{")

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
        # A trailing `// source` comment is not code: resolve writes one after every copied
        # declaration, and `endswith(";")` on the raw line ran each such decl on into the
        # next -- net-rust's whole CRT block parsed as ONE ConMsgPrintfV, copied into attrib
        # as `pub fn ConMsgPrintfV(..); fn _wcsicmp(..); ..` (private shadows of every name).
        code = line.split("//", 1)[0]
        depth += code.count("(") - code.count(")")
        if depth <= 0 and code.rstrip().endswith(";"):
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


_INLINE_GLOB_RE = re.compile(r"pub use super::(\w+)::\*")


def _resolve_inline_module(rust_path: str, crate: Path) -> list[Path]:
    """`ntkrnl_sys::ddk` is no file: lib.rs declares `pub mod ddk { pub use super::bindgen_
    <arch>::*; ... }`. Follow its `super::X::*` globs to the files (amd64's arm only, plus the
    arch-independent ones -- the names are the same on every arch)."""
    segs = rust_path.split("::")
    if len(segs) != 2 or segs[0] in ("crate", "self"):
        return []
    root = crate.parent / segs[0].replace("_", "-") / "src"
    lib = root / "lib.rs"
    if not lib.is_file():
        return []
    m = re.search(rf"^pub mod {re.escape(segs[1])}\s*\{{(.*?)^\}}", lib.read_text(), re.S | re.M)
    if not m:
        return []
    out = []
    for mod in _INLINE_GLOB_RE.findall(m.group(1)):
        if mod.endswith(("_i386", "_arm64")):
            continue
        out += [c for c in (root / f"{mod}.rs", root / mod / "mod.rs") if c.is_file()]
    return out


_PATH_MOD_RE = re.compile(r'#\[path\s*=\s*"([^"]+)"\]\s*mod\s+(\w+)\s*;')


def _path_module_globs(f: Path) -> list[Path]:
    """A dispatcher (`types.rs`) that `#[path]`-includes per-arch files and re-exports one with
    `pub use types_arch::*;`: the names live in the included file (amd64's, as above)."""
    text = f.read_text(errors="replace")
    out = []
    for target, mod in _PATH_MOD_RE.findall(text):
        if re.search(rf"pub use {re.escape(mod)}::\*", text) and not target.endswith(
            ("_i386.rs", "_arm64.rs")
        ):
            cand = f.parent / target
            if cand.is_file():
                out.append(cand)
    return out


def own_reexports(crate: Path, module: str) -> set[str]:
    """Names the prelude re-exports from the crate module `module` (`pub use crate::..::m::X;`)."""
    prelude = Path(crate) / "src" / "prelude.rs"
    if not module or not prelude.is_file():
        return set()
    stem = Path(module).stem
    return set(
        re.findall(
            rf"^pub use crate::(?:\w+::)*{re.escape(stem)}::(\w+)\s*;",
            prelude.read_text(errors="replace"),
            re.M,
        )
    )


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
        for f in _resolve_path(path, crate) or _resolve_inline_module(path, crate):
            names |= _harvest_file(f)
            for g in _path_module_globs(f):
                names |= _harvest_file(g)

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


def _shared_crate_macros(workspace: Path, ident: str) -> set[str]:
    """`#[macro_export]` names a workspace crate (by its Rust identifier) defines."""
    try:
        from tools.source_walk import source_files
    except ModuleNotFoundError:
        import sys

        sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
        from tools.source_walk import source_files

    src = workspace / ident.replace("_", "-") / "src"
    if not src.is_dir():
        return set()
    out: set[str] = set()
    for p in source_files(src, "*.rs"):
        out |= set(
            re.findall(
                r"#\[macro_export\]\s*macro_rules!\s*([A-Za-z_]\w*)", p.read_text(errors="replace")
            )
        )
    return out


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
        names = set(_RE_MACRO_RULES.findall(text))
        # ...and the macros it RE-EXPORTS from a shared crate (`pub use win32macros_rust::{
        # IntToPtr, ZeroMemory, ..}`): a call the model spelled `IntToPtr(x)` needs its `!` just
        # the same, and reading only local `macro_rules!` left access/wined3dcfg calling a macro
        # as a function (E0423).
        for m in re.finditer(r"pub use (\w+)(?:::\w+)*::\{([^}]*)\}", text):
            listed = {n.strip() for n in m.group(2).split(",") if n.strip()}
            names |= listed & _shared_crate_macros(crate.parent, m.group(1))
        found = frozenset(names)
    _MACRO_CACHE[crate] = found
    return set(found)


#: `assembler.py` appends the whole translated `.c` as a trailing comment block under this
#: banner. It is the 1:1 reference that makes the NEXT translation of the file land as a
#: readable diff, so it must stay byte-identical to the C — a rewrite that reaches into it
#: turns `if(!MSVCRT_CHECK_PMT(x))` into Rust-flavoured pseudo-C and destroys exactly that.
#: Every other pass here edits `extern` blocks and `use` lines, which do not occur in the
#: block; a bare-name substitution is the first one that can.
_C_SOURCE_BANNER = "\n// ORIGINAL C SOURCE"


_C_ENUM = re.compile(
    r"(?P<attrs>(?:^[ \t]*#\[[^\]]*\][ \t]*\n)*)^(?P<indent>[ \t]*)pub enum (?P<name>[A-Za-z_]\w*)"
    r"\s*\{(?P<body>[^{}]*)\}[ \t]*\n?",
    re.M,
)
_VARIANT = re.compile(r"^\s*([A-Za-z_]\w*)\s*(?:=\s*(.+?))?\s*$", re.S)


def lower_c_enums(src: str) -> tuple[str, list[str]]:
    """A translated C `enum` -> bindgen's shape: `pub type E = c_uint;` + one const each.

    The preamble pass renders `typedef enum _ADAPTER_TYPE { EisaAdapter, .. }` as a Rust
    `pub enum`, but the C bodies -- and so the translations -- spell the enumerators bare
    (`AdapterTypeMax`) and do integer arithmetic on them, and a Rust enum holding a value
    that is not one of its variants is undefined behaviour. The bindgen islands already use
    `c_uint` + consts (`c_int` when an enumerator is negative), so translated code does too.
    Only FIELDLESS enums are touched; `E::Variant` paths are rewritten to the bare name.
    """
    lowered: list[str] = []

    def repl(m: re.Match[str]) -> str:
        parts = [p for p in _split_top_commas(m.group("body")) if p.strip()]
        variants = []
        for part in parts:
            code = re.sub(r"//[^\n]*|/\*.*?\*/", "", part, flags=re.S)
            if not code.strip():
                continue
            v = _VARIANT.match(code)
            if v is None:
                return m.group(0)  # a data-carrying variant: a real Rust enum, not C's
            variants.append((v.group(1), v.group(2)))
        name, indent = m.group("name"), m.group("indent")
        negative = any(val is not None and val.strip().startswith("-") for _, val in variants)
        ctype = "::core::ffi::c_int" if negative else "::core::ffi::c_uint"
        out = [f"{indent}pub type {name} = {ctype};"]
        prev = None
        for vname, val in variants:
            expr = val.strip() if val is not None else ("0" if prev is None else f"{prev} + 1")
            out.append(f"{indent}pub const {vname}: {name} = {expr};")
            prev = vname
        lowered.append(name)
        return "\n".join(out) + "\n"

    src = _C_ENUM.sub(repl, src)
    if lowered:
        aliases = set(lowered)
        for m in re.finditer(r"pub type (\w+) = (\w+);", src):
            if m.group(2) in aliases:
                aliases.add(m.group(1))
        src = re.sub(rf"\b(?:{'|'.join(map(re.escape, sorted(aliases)))})::(\w+)\b", r"\1", src)
    return src, lowered


def drop_self_imports(src: str, module: str) -> tuple[str, list[str]]:
    """`use crate::setuplib::arcname::X;` inside arcname.rs -> gone.

    The assembler writes a `use` for every sibling-module name a body calls, and a name the
    file defines ITSELF lands there too; beside the definition that is E0255. setuplib had
    12 (arcname's tables, bldrsup's NtosBootLoaders, registry's hive tables).
    """
    if not module:
        return src, []
    dropped: list[str] = []

    def repl(m: re.Match[str]) -> str:
        dropped.append(m.group(1))
        return ""

    src = re.sub(
        rf"^use crate::(?:\w+::)*{re.escape(module)}::(\w+);[ \t]*\n", repl, src, flags=re.M
    )
    return src, dropped


def _split_top_commas(body: str) -> list[str]:
    parts, depth, cur = [], 0, []
    for ch in body:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur))
    return parts


_LITERAL_OR_COMMENT = re.compile(
    r'//[^\n]*|/\*.*?\*/|b?r#*"(?:[^"]|"(?!#))*"#*|b?"(?:\\.|[^"\\])*"|b?\'(?:\\.|[^\'\\])\'',
    re.S,
)


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

        # Matched on a copy with string literals and comments blanked: install.c's
        # `DPRINT1("SpInfFindFirstLine(\"SourceDisksNames\", ...) failed")` had its MESSAGE
        # rewritten to `SpInfFindFirstLine!(`.
        masked = _LITERAL_OR_COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), src)
        count = 0
        for m in reversed(list(pattern.finditer(masked))):
            # `fn NAME(` is a DECLARATION (an extern decl of what the prelude has as a
            # macro): `fn DPRINT1!(` is not Rust. integrate() drops such decls as provided.
            if re.search(r"\bfn\s+$", masked[max(0, m.start() - 16) : m.start()]):
                continue
            src = src[: m.start()] + f"{name}!(" + src[m.end() :]
            count += 1
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
    enums_lowered: list[str] = field(default_factory=list)  # C enum -> c_uint + consts
    kept_unprovided: list[str] = field(default_factory=list)  # REPORT: verify each
    kept_statics: list[str] = field(default_factory=list)  # e.g. IID_* link symbols
    promoted_pub: list[str] = field(default_factory=list)
    macro_calls_fixed: list[str] = field(default_factory=list)  # `NAME(x)` -> `NAME!(x)`
    multichar_folded: list[str] = field(default_factory=list)  # C `'PedI'` -> 0x50656449
    dropped_bad_imports: list[str] = field(default_factory=list)  # `use sys::m::{Nope}`
    indexed_offsets: int = 0  # offset_of!(T, a[i]) -> FIELD_OFFSET!
    dropped_fn_statics: list[str] = field(default_factory=list)  # `static F: FNTYPE = None;`
    dropped_type_defs: list[str] = field(default_factory=list)  # re-declared provided types
    fn_ptr_calls: int = 0  # `.expect("..")(` / `.unwrap()(` on an Option<fn> -> unchecked
    blocks_removed: int = 0

    @property
    def changed(self) -> bool:
        return bool(
            self.substrate_swapped
            or self.prelude_added
            or self.fn_ptr_calls
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


# C's multi-character constant (`'PedI'`, the pool-tag idiom) is an int whose bytes are the
# characters in BIG-endian order (clang/GCC). In Rust it is not a literal at all -- it is a
# parse error that hides every other error in the file -- so fold it to its value. Two to
# four printable ASCII characters between quotes is never valid Rust, so nothing else matches.
# Guarded both ways against lifetimes: `<'a, 'b>` would otherwise read `'a, '` as one.
# A real one-char Rust literal is matched FIRST and kept, so the scan resumes after its closing
# quote: in cmd's `[' ', '%', '%']` the `', '` BETWEEN two char literals read as a multi-char
# constant and became `' 0x00002C20 /* ', ' */%'`, a parse error.
_MULTICHAR_RE = re.compile(
    r"'(?:\\(?:u\{[0-9a-fA-F]{1,6}\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'"
    r"|(?<![<&\w])'([\x20-\x26\x28-\x5b\x5d-\x7e]{2,4})'(?![\w])"
)


def fold_multichar_literals(src: str) -> tuple[str, list[str]]:
    folded: list[str] = []
    out: list[str] = []
    for line in src.splitlines(keepends=True):
        code, sep, comment = line.partition("//")
        # A string could hold the pattern; leave the line to the model. So could the body of
        # a multi-line block comment, which a per-line scan cannot see the opener of (cmd's
        # goto.rs `* any label containing '/?' in it.`).
        if '"' in code or code.lstrip().startswith("*"):
            out.append(line)
            continue

        # Never inside a block comment -- including the `/* 'PedI' */` this fold writes, which a
        # second integrate folded again into `/* 0x50656449 /* 'PedI' */ */` (legide).
        blocks = [b.span() for b in re.finditer(r"/\*.*?\*/", code)]

        def repl(m: re.Match, blocks: list[tuple[int, int]] = blocks) -> str:
            if m.group(1) is None or any(lo <= m.start() < hi for lo, hi in blocks):
                return m.group(0)
            value = 0
            for ch in m.group(1):
                value = (value << 8) | ord(ch)
            folded.append(m.group(1))
            return f"0x{value:08X} /* '{m.group(1)}' */"

        out.append(_MULTICHAR_RE.sub(repl, code) + sep + comment)
    return "".join(out), folded


_SYS_NAMES_CACHE: dict[Path, frozenset[str]] = {}
_SYS_ITEM_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?"
    r"(?:fn|const|static(?:\s+mut)?|struct|union|enum|type|trait)\s+([A-Za-z_]\w*)",
    re.M,
)
# `use path::{a, b};` or, when the header names one item, the brace-less `use path::a;`
# (sil680.rs's `use ntkrnl_sys::ddk::Channel;` -- a field name -- was never pruned).
_EXPLICIT_SYS_USE_RE = re.compile(
    r"^use\s+((?:ntkrnl_sys|ntdll_sys)(?:::\w+)*)::(?:\{([^}]*)\}|(\w+))\s*;[ \t]*\n?", re.M
)


def _sys_names(workspace: Path, crate_ident: str) -> frozenset[str]:
    """Every item name the -sys crate defines anywhere (no module scoping: a cheap filter)."""
    root = (workspace / crate_ident.replace("_", "-") / "src").resolve()
    if root not in _SYS_NAMES_CACHE:
        names: set[str] = set()
        # Imported here, not at module scope: `port` runs rustconv with cwd tools/rustconv,
        # where `tools.` is not importable, and this module is on __main__'s import path.
        try:
            from tools.source_walk import source_files
        except ModuleNotFoundError:
            import sys

            sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
            from tools.source_walk import source_files

        for rs in source_files(root) if root.is_dir() else []:
            names |= set(_SYS_ITEM_RE.findall(rs.read_text(errors="replace")))
        _SYS_NAMES_CACHE[root] = frozenset(names)
    return _SYS_NAMES_CACHE[root]


_MODULE_NAMES_CACHE: dict[tuple[Path, str], frozenset[str]] = {}
_TOP_ITEM_RE = re.compile(
    r"^(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?"
    r"(?:fn|const|static(?:\s+mut)?|struct|union|enum|type|trait)\s+([A-Za-z_]\w*)",
    re.M,
)


def _item_names(f: Path) -> set[str]:
    """Column-0 items plus the fns of its extern blocks -- never an indented METHOD (bindgen's
    bitfield accessors are `pub fn Level(&self)`, which is not an importable item)."""
    text = f.read_text(errors="replace")
    names = set(_TOP_ITEM_RE.findall(text))
    for _, _, body, _ in extern_block_spans(text):
        names |= set(re.findall(r"\bfn\s+([A-Za-z_]\w*)", body))
    return names


def _module_names(workspace: Path, path: str) -> frozenset[str]:
    """Item names THE NAMED MODULE provides on EVERY arch (`ntkrnl_sys::ddk` -> its
    per-arch bindgen re-exports, intersected).

    A crate-wide name set let `Level`/`Mask` through because bitfield accessors are spelled
    `pub fn`, and a one-arch view let `DRIVER_INITIALIZE` through, which i386's ddk lacks:
    an unconditional `use` of either fails the whole statement on some target. Empty when
    the module cannot be located."""
    key = (workspace.resolve(), path)
    if key in _MODULE_NAMES_CACHE:
        return _MODULE_NAMES_CACHE[key]
    segs = path.split("::")
    root = workspace / segs[0].replace("_", "-") / "src"
    per_arch: list[set[str]] = []
    lib = root / "lib.rs"
    m = (
        re.search(rf"^pub mod {re.escape(segs[1])}\s*\{{(.*?)^\}}", lib.read_text(), re.S | re.M)
        if len(segs) == 2 and lib.is_file()
        else None
    )
    if m:
        mods = _INLINE_GLOB_RE.findall(m.group(1))
        common = [x for x in mods if not x.endswith(("_i386", "_amd64", "_arm64"))]
        for arch in ("i386", "amd64", "arm64"):
            names: set[str] = set()
            for mod in [*common, *(x for x in mods if x.endswith(f"_{arch}"))]:
                for c in (root / f"{mod}.rs", root / mod / "mod.rs"):
                    if c.is_file():
                        names |= _item_names(c)
            per_arch.append(names)
    else:
        files = _resolve_path(path, workspace / "__probe__")
        if files:
            per_arch.append(set().union(*(_item_names(f) for f in files)))
    result = frozenset(set.intersection(*per_arch)) if per_arch else frozenset()
    _MODULE_NAMES_CACHE[key] = result
    return result


def drop_unknown_substrate_imports(src: str, workspace: Path) -> tuple[str, list[str]]:
    """Drop names an explicit `use ntkrnl_sys::ddk::{...};` list asks for that do not exist.

    The kernel import header lists C's bare enumerators (`Latched`, `PCIConfiguration`), which
    bindgen spells with their enum prefix; one unknown name fails the WHOLE `use` (E0432) and
    every other name in it with it. A dropped name stays unresolved in the body, where
    `port resolve` aliases the bare enumerator to the prefixed constant.
    """
    dropped: list[str] = []

    def repl(m: re.Match) -> str:
        path, items = m.group(1), m.group(2) if m.group(2) is not None else m.group(3)
        known = _module_names(workspace, path) or _sys_names(workspace, path.split("::")[0])
        if not known:
            return m.group(0)
        keep = []
        for item in (i.strip() for i in items.split(",")):
            if not item:
                continue
            name = item.split()[0]
            if name in known or name == "self":
                keep.append(item)
            else:
                dropped.append(name)
        if not keep:
            return ""
        if len(keep) == len([i for i in items.split(",") if i.strip()]):
            return m.group(0)  # nothing dropped: leave the line exactly as written
        return f"use {path}::{{{', '.join(keep)}}};\n"

    return _EXPLICIT_SYS_USE_RE.sub(repl, src), dropped


_OFFSET_OF_CALL = "core::mem::offset_of!("


def route_indexed_offset_of(src: str) -> tuple[str, int]:
    """`core::mem::offset_of!(T, Arr[i])` -> the index-aware `FIELD_OFFSET!`, as `usize`.

    C's `FIELD_OFFSET(AHCI_COMMAND_LIST, CommandHeader[Slots])` sizes a variable-length
    struct; offset_of! refuses an index ("array indexing not supported in offset_of") and
    the parse error hides every other error in the file. Parens are matched, because the
    index is often an expression (`PrdTable[(*ChanData).MaximumPhysicalPages as usize]`).
    """
    out, pos, count = [], 0, 0
    while True:
        at = src.find(_OFFSET_OF_CALL, pos)
        if at < 0:
            break
        open_ = at + len(_OFFSET_OF_CALL) - 1
        depth, i, comma = 0, open_, -1
        while i < len(src):
            ch = src[i]
            if ch in "([{":
                depth += 1
            elif ch in ")]}":
                depth -= 1
                if depth == 0:
                    break
            elif ch == "," and depth == 1 and comma < 0:
                comma = i
            i += 1
        if i >= len(src) or comma < 0 or "[" not in src[comma:i]:
            out.append(src[pos : open_ + 1])
            pos = open_ + 1
            continue
        ty, path = src[open_ + 1 : comma].strip(), src[comma + 1 : i].strip()
        out.append(src[pos:at])
        out.append(f"(FIELD_OFFSET!({ty}, {path}) as usize)")
        pos = i + 1
        count += 1
    out.append(src[pos:])
    return "".join(out), count


_FN_FORWARD_STATIC_RE = re.compile(
    r"^[ \t]*(?:pub(?:\([^)]*\))?\s+)?static(?:\s+mut)?\s+([A-Za-z_]\w*)"
    r"\s*:[^=;\n]*=\s*None\s*;[ \t]*\n?",
    re.M,
)


def drop_fn_forward_statics(src: str) -> tuple[str, list[str]]:
    """Drop `static Foo: IO_COMPLETION_ROUTINE = None;` beside a real `fn Foo`.

    That is C's function-typed FORWARD DECLARATION (`static IO_COMPLETION_ROUTINE Foo;`)
    read as data: the preamble pass emits it, and next to the definition it is E0428 -- or,
    if it ever won, a NULL where the routine should be (atapi's dev_power.c and scsi.c).
    """
    fns = set(DEFINED_FN_RE.findall(_outside_extern(src)))
    dropped: list[str] = []

    def repl(m: re.Match) -> str:
        if m.group(1) in fns:
            dropped.append(m.group(1))
            return ""
        return m.group(0)

    return _FN_FORWARD_STATIC_RE.sub(repl, src), dropped


_TYPE_DEF_START_RE = re.compile(
    r"^(?:#\[[^\]]*\]\s*\n)*(?:pub(?:\([^)]*\))?\s+)?(struct|union|type)\s+([A-Za-z_]\w*)\b",
    re.M,
)


def drop_provided_type_defs(src: str, provided: set[str]) -> tuple[str, list[str]]:
    """Drop a module-local struct/union/type alias that the prelude already provides.

    The layout of a type the island or the -sys crate provides comes from bindgen; a
    preamble that re-declares it (atapi.c re-emitted `_UNICODE_STRING`, plus the
    self-referential `pub type _UNICODE_STRING = _UNICODE_STRING;`) can only be a second,
    hand-guessed copy. A self-alias is dropped unconditionally: it is never valid Rust.
    """
    dropped: list[str] = []
    out, pos = [], 0
    for m in _TYPE_DEF_START_RE.finditer(src):
        if m.start() < pos:
            continue
        kind, name = m.group(1), m.group(2)
        line_end = src.find("\n", m.end())
        line_end = len(src) if line_end < 0 else line_end
        first_line = src[m.start() : line_end]
        self_alias = kind == "type" and re.search(rf"=\s*{re.escape(name)}\s*;", first_line)
        if not self_alias and name not in provided:
            continue
        if kind == "type":
            end = src.find(";", m.end())
            if end < 0:
                continue
            end += 1
        else:
            brace = src.find("{", m.end())
            semi = src.find(";", m.end())
            if brace < 0 or (0 <= semi < brace):
                end = semi + 1
            else:
                depth, i = 0, brace
                while i < len(src):
                    if src[i] == "{":
                        depth += 1
                    elif src[i] == "}":
                        depth -= 1
                        if depth == 0:
                            break
                    i += 1
                end = i + 1
        while end < len(src) and src[end] == "\n":
            end += 1
        out.append(src[pos : m.start()])
        pos = end
        dropped.append(name)
    out.append(src[pos:])
    return "".join(out), dropped


_PANICKING_FN_CALL = re.compile(r"\.(?:expect\(\s*\"(?:[^\"\\]|\\.)*\"\s*\)|unwrap\(\))(\)?)\s*\(")


def unchecked_fn_pointer_calls(src: str) -> tuple[str, int]:
    """`f.expect("non-null function pointer")(args)` -> `f.unwrap_unchecked()(args)`.

    C calls a function pointer without a check; calling NULL is undefined in both languages,
    and a panic is a crash C does not have. The model writes `.expect(..)` / `.unwrap()` on a
    bindgen `Option<fn>` slot, which compiles, so crate-repair never sees it -- the deferred
    lint did, eleven times in usetup's cabinet and list code. Only the call shape `..)(` is
    rewritten: an `.unwrap()` whose result is not immediately called is left for review.
    """
    out, n = _PANICKING_FN_CALL.subn(lambda m: f".unwrap_unchecked(){m.group(1)}(", src)
    # `Some(X.expect(..)).unwrap_unchecked()(args)`: an earlier repair wrapped the panicking
    # unwrap instead of replacing it (ndis io.rs ISRHandler); the inner one still panics.
    out, k = _REWRAPPED_FN_EXPECT.subn(lambda m: ".unwrap_unchecked()" + m.group(1), out)
    n += k
    # The same call split across a binding (ndis, 11 sites):
    #     let f = (*Drv).Chars.QueryInformationHandler.expect("non-null function pointer");
    #     Status = f(..);
    # c2rust's phrase marks a function-pointer call; rewritten only when the bound name is
    # then CALLED, so a pointer that is stored or compared is left for review.
    pieces, last, m2 = [], 0, 0
    for m in _BOUND_FN_EXPECT.finditer(out):
        stmt = out.rfind(";", 0, m.start())
        stmt = max(stmt, out.rfind("{", 0, m.start()), out.rfind("}", 0, m.start()))
        bound = re.match(r"\s*let\s+(?:mut\s+)?(\w+)\b", out[stmt + 1 : m.start()])
        if bound is None or not re.search(rf"\b{re.escape(bound.group(1))}\s*\(", out[m.end() :]):
            continue
        pieces += [out[last : m.start()], ".unwrap_unchecked()"]
        last = m.end() - 1  # keep the `;`
        m2 += 1
    if m2:
        out = "".join(pieces) + out[last:]
    return out, n + m2


_REWRAPPED_FN_EXPECT = re.compile(
    r"\.expect\(\s*\"non-null function pointer\"\s*\)(\s*,?\s*\)\s*\.unwrap_unchecked\(\)\s*\()"
)
_BOUND_FN_EXPECT = re.compile(r"\.expect\(\s*\"non-null function pointer\"\s*\)\s*;")


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
    # A name the prelude re-exports FROM THIS MODULE (resolve's `pub use crate::cmd::dir::X;`)
    # is this module's own definition, not something provided to it: counting it dropped
    # dir.rs's `_DirSwitchesFlags` the moment resolve wired it, and then nothing defined it.
    provided = provided - own_reexports(crate, module)

    src = _swap_substrate(src, report)
    src, report.multichar_folded = fold_multichar_literals(src)
    src, _ = variadic_shim.lower_va_list_types(src)
    src, report.dropped_bad_imports = drop_unknown_substrate_imports(src, crate.parent)
    src, report.indexed_offsets = route_indexed_offset_of(src)
    src, report.enums_lowered = lower_c_enums(src)
    src, _self_imports = drop_self_imports(src, module)

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
    src, report.dropped_fn_statics = drop_fn_forward_statics(src)
    src, report.dropped_type_defs = drop_provided_type_defs(src, provided)

    # A prelude macro is spelled as a plain call in the C, so it comes back as a plain call.
    # Which names are macros is a fact about the PRELUDE, not about the C — see prelude_macros.
    macros = prelude_macros(crate)
    src, report.macro_calls_fixed = fix_macro_calls(src, macros, defined)
    # ...and the other way: a name the prelude defines as a FUNCTION and not as a macro is a
    # plain call. dxdiag's `IDirectSound8_Release!(p)` was right while the SDK macro was
    # lowered as one, and wrong once the island carried the Vtbl and `com_wrappers` wrote
    # the WIDL C wrapper as a function instead (E: cannot find macro).
    prelude_rs = crate / "src" / "prelude.rs"
    if prelude_rs.is_file():
        fns = set(re.findall(r"(?m)^pub (?:unsafe )?fn (\w+)", prelude_rs.read_text())) - macros
        for name in sorted(fns):
            src, n = re.subn(rf"(?<!\w){re.escape(name)}!\s*\(", f"{name}(", src)
            if n:
                report.macro_calls_fixed.append(f"{name}!→fn x{n}")
    src, report.fn_ptr_calls = unchecked_fn_pointer_calls(src)

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
            elif name in provided or name in macros:
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
    # A leading attribute on the SAME LINE still leaves the item at column 0, and rustconv
    # emits exactly that shape: `#[no_mangle] pub unsafe extern "system" fn Foo(`. Anchoring
    # on `^pub` made every one-line `#[no_mangle] pub fn` invisible to the export index, so a
    # sibling module calling it got `cannot find function` and no import was ever generated --
    # `CRecycleBin_ParseRecycleBinPath_RustBridge` is defined and public in `recycle_bin.rs`,
    # and CRecycleBinFolderViewCB still could not see it.
    r"^(?:#\[[^\]]*\]\s*)*pub\s+(?:unsafe\s+)?(?:extern\s+\"[A-Za-z0-9_-]+\"\s+)?"
    # A C global named after a Rust keyword is defined as a RAW identifier (`r#type`); the
    # name group keeps the `r#`, or the index records it as `r` and imports that.
    r"(fn|struct|enum|union|trait|type|const|static\s+mut|static)\s+((?:r#)?[A-Za-z_][A-Za-z0-9_]*)",
    re.M,
)

#: Never imported ACROSS modules, only ever provided by the island or the prelude. A constant
#: reaching a sibling module is a substrate gap — `GENERIC_WRITE` living in a class module
#: because the island lacks it — and cross-importing it would entrench the duplicate instead of
#: reporting it. Excluding them also kills a self-import: a wide-literal static
#: (`pub static CPP_COM_WSTR_…: [WCHAR; 4] = …`) is invisible to `defined_names`, whose
#: static/const pattern is `:[^;]*=` and so cannot span the `;` inside an array type, and the
#: file was importing its own definition from itself.
#:
#: `static mut` is deliberately NOT in here, and the distinction is the rationale above rather
#: than an exception to it. Substrate is immutable: a `const` or a plain `static` crossing a
#: module boundary means the island is missing something. MUTABLE file-scope state is the
#: opposite — it is the C translation unit's own state, it has exactly one owner by
#: construction, and it must be reachable from every module the TU was split across.
#: `CDirectoryWatcher.cpp` is the shape: its `static HANDLE s_hThreadAPC` is written by the APC
#: callbacks in the free-functions module and read by the class methods in the other, because
#: one rustconv run owns one output file. Excluding it made a TU unable to see its own state.
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


def drop_stale_sibling_imports(
    code: str,
    crate: Path,
    owners: dict[str, set[str]] | None = None,
    provided: set[str] | None = None,
) -> tuple[str, list[str]]:
    """Drop a `use crate::<sibling>::NAME;` whose sibling no longer defines NAME, when the
    prelude supplies it.

    An import this module wrote earlier outlives the definition it pointed at: regedit's
    `use crate::regedit::error::ErrorMessageBox;` was right until the variadic stage turned
    the definition into `ErrorMessageBox_v` plus a prelude extern, and then named only the
    prelude glob error.rs imports privately (E0603). Dropped only when the prelude supplies
    the name, so the reference still resolves -- to the same symbol."""
    crate_src = Path(crate) / "src"
    if owners is None:
        owners = index_crate_exports(crate_src)
    if not owners:
        return code, []
    if provided is None:
        provided = provided_names(crate) | island_names(crate_src)
    modules = set().union(*owners.values())
    notes: list[str] = []

    def _stale(m: re.Match[str]) -> str:
        path, name = m.group(1), m.group(2)
        if path in modules and path not in owners.get(name, set()) and name in provided:
            notes.append(f"{name}: no longer defined in {path}, the prelude provides it")
            return ""
        return m.group(0)

    code = re.sub(
        r"^use (crate::[A-Za-z0-9_:]+)::([A-Za-z_]\w*);[ \t]*\n", _stale, code, flags=re.M
    )
    return code, notes


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

    provided = provided_names(crate) | island_names(crate_src)
    code, stale_notes = drop_stale_sibling_imports(code, crate, owners, provided)
    local = defined_names(code)
    # A name this file EXTERN-DECLARES is already in scope, and importing it too is
    # `E0255: the name ... is defined multiple times`. `defined_names` deliberately skips
    # extern blocks (a sibling's same-named `pub fn` must still look like the owner), so the
    # skip has to be spelled here: filetypes.rs declares `fn ExtractIconExW(...)` in its own
    # `extern "system"` block AND got `use crate::shell32::iconcache::ExtractIconExW;`.
    local |= extern_declared(code)
    already = set(re.findall(r"^\s*use\s+[A-Za-z0-9_:]+::([A-Za-z_][A-Za-z0-9_]*)\s*;", code, re.M))
    # COMMENTS ARE NOT CODE. Scanning the raw text meant a name that appears only in prose
    # got a `use` line generated for it — `enum_merged_folder.rs` mentions
    # "shellutils.h ShellObjectCreator<T>" in a comment explaining its activation, and the
    # assembled file came out with `use crate::drv_def_ext::ShellObjectCreator;`, a module
    # that does not exist in that crate. The failure is loud (unresolved import) but it is
    # generated, so it cannot be fixed in the file — only here.
    words = set(re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\b", _without_comments(code)))

    notes: list[str] = stale_notes
    imports: list[str] = []
    # The same keyword-named global as the bodies spell it: c2rust renames `type` to `type_0`,
    # while the defining preamble wrote `r#type`. ftp's main.c `int type;` is read from four
    # sibling modules that way. Import the raw name under the c2rust one.
    for name in sorted(n for n in owners if n.startswith("r#")):
        alias = f"{name[2:]}_0"
        if alias in words and alias not in local and alias not in already and alias not in provided:
            mods = owners[name]
            if len(mods) == 1:
                imports.append(f"use {next(iter(mods))}::{name} as {alias};")
    raw_used = set(re.findall(r"\br#[A-Za-z_][A-Za-z0-9_]*\b", _without_comments(code)))
    # `defined_names` cannot see a raw identifier, so the defining file would import itself.
    raw_local = {n for _, n in _top_level_pub_items(code)}
    for name in sorted((raw_used & owners.keys()) - local - raw_local):
        if len(owners[name]) == 1:
            imports.append(f"use {next(iter(owners[name]))}::{name};")
    for name in sorted(words & owners.keys()):
        if name in local or name in already or name in provided:
            continue
        # c2rust numbers its anonymous types per transpile: a sibling's `C2RustUnnamed_163`
        # is a different struct that happens to share the counter (pciidex's via.rs got
        # svw_pata's timing struct for its bridge table). `port resolve` takes the right one
        # from the module's own TU.
        if name.startswith("C2RustUnnamed"):
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
