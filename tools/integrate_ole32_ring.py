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

FN_DECL_RE = re.compile(r"^\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")
STATIC_DECL_RE = re.compile(r"^\s*(?:pub\s+)?static\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:")


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
DEFINED_FN_RE = re.compile(
    r"^\s*(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"[a-z]+\"\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"
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


def process(src: str) -> tuple[str, dict]:
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
            fm, sm = FN_DECL_RE.search(d), STATIC_DECL_RE.search(d)
            if fm:
                name = fm.group(1)
                if name in defined:
                    stats["dropped_self"].append(name)
                elif COBJMACRO_RE.match(name):
                    stats["dropped_cobj"].append(name)
                elif name in PRELUDE_FNS:
                    stats["dropped_prelude"].append(name)
                elif name in CRT_FNS:
                    keep.append(d)
                else:
                    stats["kept_other"].append(name)
                    keep.append(d)
            elif sm:
                stats["kept_static"].append(sm.group(1))
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
    args = ap.parse_args()

    outdir = Path(args.out)
    outdir.mkdir(parents=True, exist_ok=True)

    all_other = set()
    for f in args.files:
        p = Path(f)
        new, st = process(p.read_text())
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
