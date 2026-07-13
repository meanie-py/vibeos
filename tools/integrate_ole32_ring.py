#!/usr/bin/env python3
"""integrate_ole32_ring.py — normalize a rustconv ole32 translation onto the crate's
shared prelude.

rustconv emits every file self-contained: `use ntdll_sys::nt::*;` (not an ole32-rust
dependency) plus a per-file `extern "C" { ... }` block hand-declaring everything the
C called. Three classes of decl in there are actively WRONG once the file is linked
into ole32.dll, and this script rewrites them:

  1. COBJMACRO helpers (`IFoo_Bar`) declared as extern symbols. In C these were
     macros expanding to a vtbl call — there is no such symbol to link against.
     -> drop the decl; crate::commacros_gen provides a real inline vtbl call.

  2. Win32 APIs declared `extern "C"`. WINAPI is __stdcall on i386 (`_Foo@N`), so a
     C-ABI decl asks for a symbol that does not exist. Links fine on amd64/arm64,
     breaks i386 — the nastiest kind of arch-specific bug.
     -> drop the decl; crate::win32 declares them `extern "system"`.

  3. Wine debug/HRESULT macros (TRACE/WARN/ERR/FIXME/FAILED/SUCCEEDED/debugstr_*)
     and GUID-compare inlines (IsEqualGUID/IsEqualIID/IsEqualCLSID).
     -> drop the decl; the prelude provides them.

What it deliberately KEEPS: `static IID_*/CLSID_*: GUID` externs. Those are genuine
link-time data symbols (uuid.lib + ole32's guid.c), and re-defining them in Rust
would be a silent-corruption source.

Anything else in an extern block is left alone and REPORTED, so an unrecognized
symbol can't slip through silently as an undefined-at-link surprise.

Usage:  tools/integrate_ole32_ring.py <staging.rs>... --out sdk/lib/rust/ole32-rust/src/ring
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# Provided by the prelude — drop the local decl.
PRELUDE_FNS = {
    # debug / HRESULT / guid-compare (debug_shims.rs + winemacros.rs)
    "TRACE", "WARN", "ERR", "FIXME", "FAILED", "SUCCEEDED",
    "debugstr_a", "debugstr_w", "debugstr_an", "debugstr_wn", "debugstr_guid",
    "debugstr_user_flags", "debugstr_formatetc", "wine_dbgstr_longlong",
    "wine_dbgstr_guid", "wine_dbgstr_rect",
    "IsEqualGUID", "IsEqualIID", "IsEqualCLSID",
    # winemacros.rs inlines
    "HIWORD", "LOWORD", "HIBYTE", "LOBYTE", "MAKELONG",
    "InterlockedIncrement", "InterlockedDecrement", "InterlockedExchange",
    "ZeroMemory", "HRESULT_FROM_WIN32",
    # win32.rs (extern "system")
    "GetProcessHeap", "HeapAlloc", "HeapReAlloc", "HeapFree", "HeapSize", "HeapValidate",
    "LocalAlloc", "LocalReAlloc", "LocalFree",
    "GlobalAlloc", "GlobalReAlloc", "GlobalFree", "GlobalLock", "GlobalUnlock", "GlobalSize",
    "InitializeCriticalSection", "DeleteCriticalSection",
    "EnterCriticalSection", "LeaveCriticalSection",
    "GetThreadLocale", "IsBadReadPtr", "IsBadWritePtr", "IsBadCodePtr",
    "GetLastError", "SetLastError",
    "lstrlenW", "lstrcpyW", "lstrcmpW", "lstrcmpiW",
    # wine/heap.h inlines (win32.rs)
    "heap_alloc", "heap_alloc_zero", "heap_realloc", "heap_free",
    # wine/list.h intrusive list (wine_list.rs)
    "list_init", "list_add_after", "list_add_before", "list_add_head", "list_add_tail",
    "list_remove", "list_next", "list_prev", "list_head", "list_tail", "list_empty",
    "list_count", "list_move_tail",
}

# CRT memory intrinsics: keep as extern "C" (they genuinely ARE cdecl).
CRT_FNS = {"memcpy", "memset", "memmove", "memcmp", "strlen", "wcslen", "wcscmp", "wcscpy"}

COBJMACRO_RE = re.compile(r"^I[A-Z][A-Za-z0-9]*_[A-Za-z0-9_]+$")

# re.M matters: split_decls glues any preceding comment lines onto the decl, so a
# `^`-anchored non-multiline pattern silently fails to classify every commented decl —
# and an unclassified decl is KEPT, which is the failure direction that hurts (a stale
# `extern "C"` decl shadows the prelude and breaks i386). Comment lines are stripped
# before matching so a `//`-quoted signature can't false-match either.
FN_DECL_RE = re.compile(r"^\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", re.M)
STATIC_DECL_RE = re.compile(
    r"^\s*(?:pub\s+)?static\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:", re.M)
COMMENT_LINE_RE = re.compile(r"^\s*//.*$", re.M)


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


# A DEFINITION has a body: `... fn name(...) ... {`. An extern DECL ends in `;`.
# Matching on the trailing `{` is what separates them — critical, because these two
# look identical up to the parameter list, and misreading a decl as a definition
# would silently delete a genuinely-external symbol.
#
# The ABI string is `[A-Za-z-]+`, NOT `[a-z]+`: the overwhelmingly common spelling here is
# `extern "C"` — uppercase. A lowercase-only class silently fails to see every
# `pub unsafe extern "C" fn`, i.e. nearly every definition in the crate, which makes the
# "already provided" check quietly useless. (It was: this class matched exactly one name
# in dictionary.rs.)
DEFINED_FN_RE = re.compile(
    r"^\s*(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"[A-Za-z-]+\"\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"
    r"[^;{]*\{", re.M | re.S)


def extern_block_spans(src: str) -> list[tuple[int, int, str]]:
    """-> [(start, end, body)] for each `extern "..." { ... }` block."""
    spans = []
    for m in re.finditer(r'extern\s+"(?:C|system)"\s*\{', src):
        brace = src.index("{", m.start())
        i, depth = brace, 0
        while i < len(src):
            if src[i] == "{":
                depth += 1
            elif src[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        spans.append((m.start(), i + 1, src[brace + 1 : i]))
    return spans


def defined_names(src: str, pub_only: bool = False) -> set[str]:
    """Names this source DEFINES (fns + statics + consts), counting only text outside
    extern blocks.

    `pub_only` matters for the cross-module ("provided by the prelude") query: a private
    `fn` is NOT re-exported by a glob, so counting it would make us drop a decl that then
    resolves to nothing. It must stay False for the self-decl query, where a file's own
    private definitions are exactly what we're looking for.
    """
    spans = extern_block_spans(src)
    outside, prev = [], 0
    for s, e, _ in spans:
        outside.append(src[prev:s])
        prev = e
    outside.append(src[prev:])
    body = "".join(outside)

    pub = r"pub\s+" if pub_only else r"(?:pub\s+)?"
    names = set(re.findall(
        rf"^\s*{pub}(?:unsafe\s+)?(?:extern\s+\"[A-Za-z-]+\"\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"
        r"[^;{]*\{", body, re.M | re.S))
    names |= set(re.findall(
        rf"^\s*{pub}(?:static|const)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:[^;]*=", body, re.M))
    names |= set(re.findall(
        rf"^\s*{pub}(?:struct|enum|union|trait|type)\s+([A-Za-z_][A-Za-z0-9_]*)", body, re.M))
    # #[macro_export] macros are crate-root items and always reachable.
    names |= set(re.findall(r"^\s*macro_rules!\s+([A-Za-z_][A-Za-z0-9_]*)", body, re.M))
    return names


def extern_declared(src: str) -> set[str]:
    """Function names this source DECLARES in an extern block (i.e. expects from elsewhere)."""
    out = set()
    for _, _, body in extern_block_spans(src):
        for d in split_decls(body):
            fm = FN_DECL_RE.search(decl_code(d))
            if fm:
                out.add(fm.group(1))
    return out


def promote_pub(src: str, names: set[str]) -> tuple[str, list[str]]:
    """Make `pub` any definition of a name that some OTHER translation unit declares.

    These are the C's `DECLSPEC_HIDDEN` cross-TU helpers — StorageUtl_ReadWord,
    StorageBaseImpl_AddStream, ... — which are ole32-internal but shared between TUs.
    rustconv, translating one file in isolation, has no reason to make them `pub`, so
    they land module-private and the prelude's glob cannot re-export them: the sibling
    that calls them would be left with an unresolved name (or, worse, keep its own
    `extern "C"` decl and link against the wrong i386 decoration).
    """
    # Only DEFINITIONS (a body: `... fn n(..) {`), and only OUTSIDE extern blocks — inside
    # one, `fn n(..);` is a decl about to be stripped, and "promoting" it would just make
    # the report lie about what was actually defined here.
    spans = extern_block_spans(src)

    def in_extern(pos: int) -> bool:
        return any(s <= pos < e for s, e, _ in spans)

    promoted = []
    for n in sorted(names):
        pat = re.compile(
            rf"^(\s*)((?:unsafe\s+)?(?:extern\s+\"[A-Za-z-]+\"\s+)?fn\s+{re.escape(n)}\s*\("
            r"[^;{]*\{)", re.M | re.S)
        hits = [m for m in pat.finditer(src) if not in_extern(m.start())]
        if not hits:
            continue
        for m in reversed(hits):  # reversed: earlier offsets stay valid as we splice
            src = src[: m.start()] + m.group(1) + "pub " + m.group(2) + src[m.end() :]
        promoted.append(n)
        spans = extern_block_spans(src)  # offsets shifted
    return src, promoted


def process(src: str, provided: set[str]) -> tuple[str, dict]:
    """`provided` = names already reachable through the prelude (defined by a sibling
    ring module, by a shared surface module, or by another file in this same batch).
    A decl for one of those is not just redundant — it is WRONG: the sibling is a Rust
    `extern "system"` definition, and a local `extern "C"` decl of the same name both
    shadows it ambiguously and asks for the wrong i386 decoration."""
    stats = {"dropped_cobj": [], "dropped_prelude": [], "dropped_self": [],
             "kept_static": [], "kept_other": []}

    src = re.sub(r"^use ntdll_sys::nt::\*;\s*$", "use crate::prelude::*;",
                 src, count=1, flags=re.M)
    if "use crate::prelude::*;" not in src:
        src = "use crate::prelude::*;\n" + src

    spans = extern_block_spans(src)

    # Functions this file DEFINES — computed from the source OUTSIDE the extern
    # blocks, so an extern decl can never be mistaken for a definition of itself.
    # rustconv translates each function independently, so a caller translated before
    # its callee gets an `extern` decl for a function that ends up defined in the very
    # same file — a hard duplicate-definition error in Rust (E0428). Drop those decls.
    outside, prev = [], 0
    for s, e, _ in spans:
        outside.append(src[prev:s])
        prev = e
    outside.append(src[prev:])
    defined = set(DEFINED_FN_RE.findall("".join(outside)))

    out, pos = [], 0
    for start, end, body in spans:

        keep = []
        for d in split_decls(body):
            code = decl_code(d)
            fm, sm = FN_DECL_RE.search(code), STATIC_DECL_RE.search(code)
            if fm:
                name = fm.group(1)
                if name in defined:
                    stats["dropped_self"].append(name)
                elif COBJMACRO_RE.match(name):
                    stats["dropped_cobj"].append(name)
                elif name in PRELUDE_FNS or name in provided:
                    stats["dropped_prelude"].append(name)
                elif name in CRT_FNS:
                    keep.append(d)
                else:
                    stats["kept_other"].append(name)
                    keep.append(d)
            elif sm:
                name = sm.group(1)
                if name in defined or name in provided:
                    stats["dropped_self"].append(name)
                else:
                    stats["kept_static"].append(name)
                    keep.append(d)
            elif d.strip():
                keep.append(d)

        out.append(src[pos:start])
        if keep:
            out.append('extern "C" {\n' + "\n".join(keep) + "\n}")
        pos = end
    out.append(src[pos:])
    return "".join(out), stats


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="+")
    ap.add_argument("--out", required=True)
    ap.add_argument("--crate-src", default="sdk/lib/rust/ole32-rust/src",
                    help="crate src root; every name defined under it is 'provided'")
    args = ap.parse_args()

    outdir = Path(args.out)
    outdir.mkdir(parents=True, exist_ok=True)

    inputs = [Path(f) for f in args.files]
    sources = {p: p.read_text() for p in inputs}

    # Everything reachable through the prelude. Three sources, and all three matter:
    #
    #   (a) DEFINED by a shared-surface module (winemacros, wine_list, commacros_*,
    #       debug_shims) or by a ring module already in the crate.
    #   (b) DECLARED by one of the prelude's extern-surface modules (win32/gdi32/user32/
    #       crt/ole32_exports/iids). A local re-decl of one of these is not harmless:
    #       an item in the module SHADOWS a glob import, so the file would silently link
    #       against the local `extern "C"` spelling — undecorated on i386, where the real
    #       symbol is `_Foo@N`. That is exactly the arch-specific break this script exists
    #       to prevent, so these must be dropped, not kept.
    #   (c) DEFINED by another file in the batch being integrated right now — storage32
    #       defines the StorageBaseImpl_*/StorageUtl_* that stg_prop/stg_stream call.
    #
    # Only src/*.rs and src/ring/*.rs are scanned: compobj/ and marshal/ are NOT globbed
    # by the prelude, so their fns are reachable only via ole32_exports and must not be
    # counted as provided.
    provided: set[str] = set()
    crate_src = Path(args.crate_src)
    batch_names = {p.name for p in inputs}
    scan = sorted(crate_src.glob("*.rs")) + sorted((crate_src / "ring").glob("*.rs"))
    for rs in scan:
        if rs.name in batch_names:
            continue  # a previous run's copy of a file we're re-integrating
        text = rs.read_text()
        provided |= defined_names(text, pub_only=True)                     # (a)
        for _, _, body in extern_block_spans(text):                        # (b)
            for d in split_decls(body):
                code = decl_code(d)
                mm = FN_DECL_RE.search(code) or STATIC_DECL_RE.search(code)
                if mm:
                    provided.add(mm.group(1))
    # DECLSPEC_HIDDEN promotion. A name that some OTHER TU declares as extern must be
    # `pub` where it is defined, or the prelude's glob cannot re-export it to the caller.
    # Exclude anything the prelude ALREADY provides: rustconv sometimes emits a local copy
    # of a shared helper (datacache re-defines `FAILED`), and promoting that copy would put
    # two same-named items in scope through two globs — an E0659 ambiguity. Left private,
    # the local copy simply shadows the glob inside its own file, which is harmless.
    wanted: set[str] = set()
    for src in sources.values():
        wanted |= extern_declared(src)
    for rs in sorted((crate_src / "ring").glob("*.rs")):
        if rs.name not in batch_names:
            wanted |= extern_declared(rs.read_text())
    wanted -= provided | PRELUDE_FNS | CRT_FNS
    for p in inputs:
        sources[p], promoted = promote_pub(sources[p], wanted)
        if promoted:
            print(f"{p.name}: +pub on cross-TU helpers: {', '.join(promoted)}")

    for src in sources.values():                                           # (c)
        provided |= defined_names(src, pub_only=True)

    all_other = set()
    for p in inputs:
        new, st = process(sources[p], provided)
        (outdir / p.name).write_text(new)
        all_other.update(st["kept_other"])
        print(f"{p.name}: -{len(st['dropped_cobj'])} cobjmacro  "
              f"-{len(st['dropped_self'])} self-decl  "
              f"-{len(st['dropped_prelude'])} prelude  "
              f"={len(st['kept_static'])} guid-static  "
              f"?{len(st['kept_other'])} unrecognized")

    if all_other:
        print("\nUNRECOGNIZED externs kept as-is (verify each is a real link symbol "
              "with the right ABI, or add it to win32.rs/PRELUDE_FNS):")
        for n in sorted(all_other):
            print(f"  {n}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
