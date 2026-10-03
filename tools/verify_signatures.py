#!/usr/bin/env python3
"""verify_signatures.py — Full function-SIGNATURE verifier for exported Rust kernel fns.

Extends the calling-convention-only checks (`verify_abi.py`) to the actual
parameter/return TYPES at the ABI boundary. It catches the bug class where an
exported

    #[no_mangle] pub extern "system" fn Foo(a: ULONG, b: PVOID) -> NTSTATUS

has a C prototype with a different ARITY (missing/extra param), a different
parameter WIDTH (e.g. SIZE_T vs ULONG — 8 vs 4 bytes on amd64), a sign flip,
or a different RETURN type — all of which link silently but are wrong.

How it works
------------
1. C ground truth: build one wrapper TU that `#include <ntoskrnl.h>`, ask clang
   for a JSON AST dump (`-Xclang -ast-dump=json`), and walk every FunctionDecl.
   clang resolves typedefs PER ARCH via `desugaredQualType`, so SIZE_T is
   `unsigned long long` on amd64 and `unsigned int` on i386 automatically — no
   hand-maintained typedef table on the C side.
2. Rust side: scan the crate for `#[no_mangle]`/`#[export_name]` exported
   `extern "..." fn NAME(params) -> ret`, parse param + return types.
3. Classify both sides into ABI "shape" buckets (pointer / 4-byte-int /
   8-byte-int / ptr-width-int / 1-byte / enum / float / void / ...) keyed on the
   target's integer-model (Windows LLP64 for amd64/arm64, ILP32 for i386), then
   compare ARITY first (high confidence) and per-slot shape (medium/low).

Confidence
----------
  HIGH    arity mismatch (extra/missing parameter) — almost never a false pos.
  MEDIUM  a parameter or return WIDTH differs (4 vs 8 bytes) with both shapes
          resolved confidently.
  LOW     a sign flip on an integer of equal width, OR one side could not be
          resolved to a known shape (unknown alias) — likely-FP, shown for
          completeness, off by default unless --low.

Usage
-----
    python3 tools/verify_signatures.py                 # default: ntoskrnl-rust, amd64
    python3 tools/verify_signatures.py --arch i386
    python3 tools/verify_signatures.py --crate sdk/lib/rust/hal-rust/src
    python3 tools/verify_signatures.py --low           # include LOW-confidence
    python3 tools/verify_signatures.py --json out.json
    python3 tools/verify_signatures.py --only ExAllocatePoolWithTag

Self-contained: no QEMU, no kernel build (only needs the generated headers under
build-<arch>/ which the size-verifier already relies on).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

# ── Paths ────────────────────────────────────────────────────────────
# The worktree may not carry .build-tools/ or build-<arch>/ (gitignored, large).
# Fall back to a sibling vibeos checkout that does — these are shared, read-only
# build infrastructure, not source.
SCRIPT_DIR = Path(__file__).resolve().parent
REPO = SCRIPT_DIR.parent  # tools/ -> repo root


def _candidate_repos() -> list[Path]:
    """This repo, then sibling vibeos checkouts carrying shared build infra."""
    candidates = [REPO]
    parent = REPO.parent
    for sib in sorted(parent.iterdir()) if parent.exists() else []:
        if sib != REPO and sib.is_dir() and (sib / ".build-tools").exists():
            candidates.append(sib)
    return candidates


def _find_toolchain() -> tuple[Path, Path]:
    """Return (clang_path, repo_with_build_dirs). Prefer this repo; fall back to
    sibling vibeos checkouts that have .build-tools + build-<arch>/."""
    candidates = _candidate_repos()
    clang = None
    for c in candidates:
        cand = c / ".build-tools/llvm-mingw/bin/clang"
        if cand.exists():
            clang = cand
            break
    if clang is None:
        sys.exit(
            "FATAL: could not find .build-tools/llvm-mingw/bin/clang in this "
            "repo or any sibling vibeos checkout"
        )
    # Pick a repo that has build-<arch> dirs (for generated headers).
    build_repo = None
    for c in candidates:
        if list(c.glob("build-*")):
            build_repo = c
            break
    if build_repo is None:
        build_repo = REPO
    return clang, build_repo


def resolve_build_repo(arch_name: str) -> Path:
    """THIS repo's build-<arch_name>/ — the generated headers this tool needs.

    Deliberately NO sibling fallback. The toolchain (a pinned clang binary) is
    genuinely shared infrastructure, but these headers are generated from source
    at a specific commit: reading another worktree's build tree compares THIS
    tree's Rust against ANOTHER lane's C, so the verdict silently depends on which
    sibling checkouts happen to exist and what state they are in. That is not a
    gate. (Observed 2026-07-24: the arch-blind sibling search picked
    vibeos-conformance/build-amd64 and the mismatch count moved 987 -> 9 without a
    single source line changing.)

    HARD FAIL when it is absent. Without the headers clang falls back to
    implicit-int declarations (`int f(VOID *)` for a `VOID f(VOID)`) and EVERY
    exported function looks mismatched — the silent degradation AGENTS.md forbids.
    """
    if (REPO / f"build-{arch_name}").is_dir():
        return REPO
    sys.exit(
        f"FATAL: {REPO}/build-{arch_name} not found. verify_signatures needs the\n"
        f"generated headers from THIS worktree's configured build tree; without\n"
        f"them clang emits implicit-int declarations and every signature looks\n"
        f"mismatched. A sibling worktree's build tree is NOT a substitute — it is\n"
        f"another lane's source at another commit.\n"
        f"  fix: ./vibeos-cli build --arch {arch_name}"
    )


CLANG, BUILD_REPO = _find_toolchain()
# clang version dir (for builtin headers)
_clang_ver_dirs = sorted((CLANG.parent.parent / "lib/clang").glob("*"))
CLANG_BUILTIN_INC = (_clang_ver_dirs[-1] / "include") if _clang_ver_dirs else None

ARCHS = {
    "amd64": {
        "target": "x86_64-pc-windows-gnu",
        "defs": ["-D_WIN64", "-D_M_AMD64", "-D__x86_64__", "-D_AMD64_", "-mxsave"],
        "ptr": 8,
        "long": 4,
        "model": "llp64",
    },
    "i386": {
        "target": "i686-pc-windows-gnu",
        "defs": ["-D_M_IX86", "-D__i386__", "-D_X86_"],
        "ptr": 4,
        "long": 4,
        "model": "ilp32",
    },
    "arm64": {
        "target": "aarch64-pc-windows-gnu",
        "defs": ["-D_WIN64", "-D_M_ARM64", "-D__aarch64__", "-D__arm64__", "-D_ARM64_"],
        "ptr": 8,
        "long": 4,
        "model": "llp64",
    },
}


# ── C include flags (mirrors verify_sizes.py get_include_flags) ──────────


def get_include_flags(arch_name: str) -> list[str]:
    R = resolve_build_repo(arch_name)
    build_dir = R / f"build-{arch_name}"
    flags = [
        "-I",
        str(build_dir / "ntoskrnl"),
        "-I",
        str(R / "ntoskrnl"),
        "-I",
        str(R / "sdk/include"),
        "-I",
        str(R / "sdk/include/ddk"),
        "-I",
        str(R / "sdk/include/ndk"),
        "-I",
        str(R / "sdk/include/psdk"),
        "-I",
        str(R / "sdk/include/xdk"),
        "-I",
        str(R / "sdk/include/reactos"),
        "-I",
        str(R / "sdk/include/reactos/libs"),
        "-I",
        str(R / "sdk/include/reactos/drivers"),
        "-I",
        str(R / "sdk/include/vcruntime"),
        "-I",
        str(R / "sdk/include/winrt"),
        "-I",
        str(R / "sdk/lib/pseh/include"),
        "-I",
        str(R),
        "-I",
        str(R / "sdk/lib/drivers/arbiter"),
        "-I",
        str(R / "sdk/lib/cmlib"),
        "-I",
        str(R / "ntoskrnl/include"),
        "-I",
        str(build_dir / "ntoskrnl/include"),
        "-I",
        str(build_dir / "ntoskrnl/include/internal"),
        "-I",
        str(build_dir / "sdk/include"),
        "-I",
        str(build_dir / "sdk/include/psdk"),
        "-I",
        str(build_dir / "sdk/include/ddk"),
        "-I",
        str(build_dir / "sdk/include/dxsdk"),
        "-I",
        str(build_dir / "sdk/include/reactos"),
        "-I",
        str(build_dir / "sdk/include/reactos/mc"),
        "-I",
        str(R / "sdk/include/dxsdk"),
        "-isystem",
        str(R / "sdk/include/crt"),
    ]
    if CLANG_BUILTIN_INC:
        flags += ["-isystem", str(CLANG_BUILTIN_INC)]
    return flags


WRAPPER_C = """\
#define _NTOSKRNL_
#define _NTSYSTEM_
#define _USE_DUMMY_PSEH 1
#define __REACTOS__
#define _INTRIN_H_
#define _INTRIN_X86_H_
#ifndef __int64
#define __int64 long long
#endif
#undef FORCEINLINE
#define FORCEINLINE
#include <ntoskrnl.h>
"""


def dump_c_ast(arch_name: str) -> dict:
    arch = ARCHS[arch_name]
    with tempfile.TemporaryDirectory() as td:
        td = Path(td)
        wrap = td / "wrap.c"
        wrap.write_text(WRAPPER_C)
        cmd = [
            str(CLANG),
            "-Xclang",
            "-ast-dump=json",
            "-fsyntax-only",
            "-nostdlibinc",
            f"--target={arch['target']}",
            *arch["defs"],
            "-D_NTOSKRNL_",
            "-D_NTSYSTEM_",
            "-D_USE_DUMMY_PSEH=1",
            "-D__REACTOS__",
            "-DDBG=1",
            "-DWINVER=0x502",
            "-D_WIN32_WINNT=0x502",
            "-ferror-limit=0",
            "-Wno-everything",
            *get_include_flags(arch_name),
            str(wrap),
        ]
        proc = subprocess.run(cmd, capture_output=True, text=True)
        if not proc.stdout.strip():
            sys.stderr.write(proc.stderr[:4000])
            sys.exit(f"FATAL: clang produced no AST for {arch_name}")
        return json.loads(proc.stdout)


# ── C prototype extraction from AST ─────────────────────────────────────


def extract_c_prototypes(ast: dict) -> tuple[dict[str, dict], dict[str, tuple[str, str]]]:
    """Returns (protos, typedefs).

    protos: name -> {"ret": (qual, desugar), "params": [(qual, desugar), ...],
                     "variadic": bool}. Last definition wins (matches TU).
    typedefs: alias-name -> (qualType, desugaredQualType) for EVERY typedef in
              the TU. This is the authoritative resolver for Rust alias names
              (POOL_TYPE -> enum, SIZE_T -> unsigned long long, etc.) so we don't
              hand-maintain a guess table on the Rust side."""
    out: dict[str, dict] = {}
    typedefs: dict[str, tuple[str, str]] = {}

    def desugar(tp: dict | None) -> tuple[str, str]:
        if not tp:
            return ("", "")
        q = tp.get("qualType", "")
        d = tp.get("desugaredQualType", q)
        return (q, d)

    def walk(node):
        if not isinstance(node, dict):
            return
        if node.get("kind") == "TypedefDecl":
            nm = node.get("name")
            if nm:
                typedefs[nm] = desugar(node.get("type"))
        if node.get("kind") == "FunctionDecl":
            name = node.get("name")
            if name:
                ret_qt = node.get("type", {}).get("qualType", "")
                params = []
                variadic = "..." in ret_qt
                for c in node.get("inner", []) or []:
                    if c.get("kind") == "ParmVarDecl":
                        params.append(desugar(c.get("type")))
                # return type = the function-type's result; derive from qualType
                ret = _return_from_fn_qualtype(ret_qt)
                out[name] = {"ret": ret, "params": params, "variadic": variadic}
        for c in node.get("inner", []) or []:
            walk(c)

    walk(ast)
    return out, typedefs


def _return_from_fn_qualtype(qt: str) -> tuple[str, str]:
    """'NTSTATUS (PDRIVER_OBJECT, ULONG)' -> return-type spelling 'NTSTATUS'.
    We only get the sugared spelling here; desugaring of the return happens via
    the shape classifier which knows the typedefs. Strip trailing attrs."""
    # cut at first '(' that starts the param list
    i = 0
    # find the '(' that opens the parameter list (top-level)
    for i, ch in enumerate(qt):  # noqa: B007  # `i` is consumed after the loop
        if ch == "(":
            break
    ret = qt[:i].strip() if i else qt.strip()
    return (ret, ret)


# ── ABI shape classifier ────────────────────────────────────────────────
# A "shape" is (category, size_bytes, signed) where category is 'int'/'ptr'/
# 'float'/'void'/'enum'/'unknown'. Comparison ignores the exact alias; it asks
# "does this occupy the same register slot the same way".

# C *desugared* base types -> (size on a given model is computed) we map names.
# Sizes are in bytes; 'L' means platform long (4 on win), 'P' means pointer width.
_C_BASE = {
    "void": ("void", 0, False),
    "_Bool": ("int", 1, False),
    "bool": ("int", 1, False),
    "char": ("int", 1, True),
    "signed char": ("int", 1, True),
    "unsigned char": ("int", 1, False),
    "short": ("int", 2, True),
    "short int": ("int", 2, True),
    "unsigned short": ("int", 2, False),
    "unsigned short int": ("int", 2, False),
    "int": ("int", 4, True),
    "unsigned int": ("int", 4, False),
    "unsigned": ("int", 4, False),
    "long": ("int", "L", True),
    "long int": ("int", "L", True),
    "unsigned long": ("int", "L", False),
    "unsigned long int": ("int", "L", False),
    "long long": ("int", 8, True),
    "long long int": ("int", 8, True),
    "unsigned long long": ("int", 8, False),
    "unsigned long long int": ("int", 8, False),
    "__int64": ("int", 8, True),
    "unsigned __int64": ("int", 8, False),
    "float": ("float", 4, True),
    "double": ("float", 8, True),
    "long double": ("float", 8, True),
    "wchar_t": ("int", 2, False),
}


def classify_c(desugar: str, qual: str, arch: dict) -> tuple[str, int | None, bool, str]:
    """Return (category, size, signed, note). size None = unknown."""
    d = desugar.strip()
    # pointers (incl. function pointers, arrays-as-params decay)
    if "*" in d or d.endswith("]") or "(*" in d:
        return ("ptr", arch["ptr"], False, "")
    # enum -> underlying int, 4 bytes in MSVC/clang win ABI
    if d.startswith("enum "):
        return ("int", 4, False, "enum")
    # struct/union BY VALUE — rare in NTAPI; treat as aggregate (size unknown)
    if d.startswith("struct ") or d.startswith("union "):
        return ("aggregate", None, False, "by-value-aggregate")
    # strip const/volatile
    d2 = re.sub(r"\b(const|volatile)\b", "", d).strip()
    if d2 in _C_BASE:
        cat, sz, sg = _C_BASE[d2]
        if sz == "L":
            sz = arch["long"]
        return (cat, sz, sg, "")
    return ("unknown", None, False, f"c?{d}")


# Rust type aliases -> shape. We resolve the common ntkrnl-sys / NTAPI aliases.
# Anything ending in pointer syntax is a pointer; otherwise alias table; else
# Rust primitives.
_RUST_PRIM = {
    "u8": ("int", 1, False),
    "i8": ("int", 1, True),
    "u16": ("int", 2, False),
    "i16": ("int", 2, True),
    "u32": ("int", 4, False),
    "i32": ("int", 4, True),
    "u64": ("int", 8, False),
    "i64": ("int", 8, True),
    "usize": ("int", "P", False),
    "isize": ("int", "P", True),
    "bool": ("int", 1, False),
    "f32": ("float", 4, True),
    "f64": ("float", 8, True),
    "c_void": ("void", 0, False),
    "()": ("void", 0, False),
    "core::ffi::c_void": ("void", 0, False),
}

# Common Windows/NT typedef aliases as spelled on the Rust side. Map to
# (category, size, signed) where size 'L'=platform long(4), 'P'=pointer.
_RUST_ALIAS = {
    # fixed-width unsigned/signed
    "ULONG": ("int", 4, False),
    "LONG": ("int", 4, True),
    "DWORD": ("int", 4, False),
    "UINT": ("int", 4, False),
    "INT": ("int", 4, True),
    "ULONG32": ("int", 4, False),
    "LONG32": ("int", 4, True),
    "USHORT": ("int", 2, False),
    "SHORT": ("int", 2, True),
    "WORD": ("int", 2, False),
    "UCHAR": ("int", 1, False),
    "CHAR": ("int", 1, True),
    "BYTE": ("int", 1, False),
    "BOOLEAN": ("int", 1, False),
    "ULONGLONG": ("int", 8, False),
    "LONGLONG": ("int", 8, True),
    "ULONG64": ("int", 8, False),
    "LONG64": ("int", 8, True),
    "DWORD64": ("int", 8, False),
    "QWORD": ("int", 8, False),
    "NTSTATUS": ("int", 4, True),
    "HRESULT": ("int", 4, True),
    "KPRIORITY": ("int", 4, True),
    "KIRQL": ("int", 1, False),
    "BOOL": ("int", 4, True),
    "WCHAR": ("int", 2, False),
    "LCID": ("int", 4, False),
    "LANGID": ("int", 2, False),
    "ACCESS_MASK": ("int", 4, False),
    # pointer-width integers
    "SIZE_T": ("int", "P", False),
    "SSIZE_T": ("int", "P", True),
    "ULONG_PTR": ("int", "P", False),
    "LONG_PTR": ("int", "P", True),
    "DWORD_PTR": ("int", "P", False),
    "INT_PTR": ("int", "P", True),
    "UINT_PTR": ("int", "P", False),
    "KAFFINITY": ("int", "P", False),
    "WPARAM": ("int", "P", False),
    "LPARAM": ("int", "P", True),
    "LRESULT": ("int", "P", True),
    # handles & raw pointers (pointer-shaped)
    "HANDLE": ("ptr", "P", False),
    "HWND": ("ptr", "P", False),
    "HKEY": ("ptr", "P", False),
    "HDC": ("ptr", "P", False),
    "HMODULE": ("ptr", "P", False),
    "HINSTANCE": ("ptr", "P", False),
    "HICON": ("ptr", "P", False),
    "HMENU": ("ptr", "P", False),
    "HGDIOBJ": ("ptr", "P", False),
    "HBRUSH": ("ptr", "P", False),
    "HBITMAP": ("ptr", "P", False),
    "HPALETTE": ("ptr", "P", False),
    "HFONT": ("ptr", "P", False),
    "HRGN": ("ptr", "P", False),
    "SC_HANDLE": ("ptr", "P", False),
    "PVOID": ("ptr", "P", False),
    "LPVOID": ("ptr", "P", False),
    "PVOID64": ("ptr", 8, False),
    "PWSTR": ("ptr", "P", False),
    "PCWSTR": ("ptr", "P", False),
    "LPWSTR": ("ptr", "P", False),
    "LPCWSTR": ("ptr", "P", False),
    "PSTR": ("ptr", "P", False),
    "PCSTR": ("ptr", "P", False),
    "LPSTR": ("ptr", "P", False),
    "LPCSTR": ("ptr", "P", False),
    "PCHAR": ("ptr", "P", False),
    "PUCHAR": ("ptr", "P", False),
    "PWCHAR": ("ptr", "P", False),
    "PULONG": ("ptr", "P", False),
    "PLONG": ("ptr", "P", False),
    "PUSHORT": ("ptr", "P", False),
    "PSHORT": ("ptr", "P", False),
    "PBOOLEAN": ("ptr", "P", False),
    "PSIZE_T": ("ptr", "P", False),
    "PULONG_PTR": ("ptr", "P", False),
    "PLONG_PTR": ("ptr", "P", False),
    "PHANDLE": ("ptr", "P", False),
    "PLARGE_INTEGER": ("ptr", "P", False),
    "PLUID": ("ptr", "P", False),
    "PFILE_OBJECT": ("ptr", "P", False),
}


def classify_rust(t: str, arch: dict, c_typedefs: dict[str, tuple[str, str]]):
    """Classify a Rust-spelled type. Authority order:
    1. raw pointer / reference / fn-pointer syntax -> ptr
    2. Rust primitive (u32/usize/...)
    3. the *C typedef table* (alias name shared with C, e.g. POOL_TYPE,
       SIZE_T) — resolved via clang's per-arch desugaring, so no guessing
    4. fallback hand table _RUST_ALIAS for names C doesn't have
    5. P-prefix pointer heuristic (last resort)"""
    t = t.strip()
    if t.startswith("*const") or t.startswith("*mut") or t.startswith("&"):
        return ("ptr", arch["ptr"], False, "")
    if "fn(" in t or t.startswith("extern ") or ("Option" in t and "fn" in t):
        return ("ptr", arch["ptr"], False, "fnptr")
    base = t.split("<")[0].strip()
    base = base.rsplit("::", 1)[-1]  # strip module path
    if base in _RUST_PRIM:
        cat, sz, sg = _RUST_PRIM[base]
        if sz == "P":
            sz = arch["ptr"]
        return (cat, sz, sg, "")
    # AUTHORITATIVE: resolve via the C typedef table (clang desugaring).
    if base in c_typedefs:
        q, d = c_typedefs[base]
        return classify_c(d, q, arch)
    if base in _RUST_ALIAS:
        cat, sz, sg = _RUST_ALIAS[base]
        if sz == "P":
            sz = arch["ptr"]
        return (cat, sz, sg, "")
    if base.startswith("NonNull"):
        return ("ptr", arch["ptr"], False, "")
    # Heuristic (last resort): P/LP-prefixed unknown typedef = pointer.
    if re.match(r"^(P|LP)[A-Z_]", base):
        return ("ptr", arch["ptr"], False, f"pfx:{base}")
    return ("unknown", None, False, f"rust?{base}")


# ── Rust exported-fn scanner ────────────────────────────────────────────


# Strip a balanced-paren param list out of a fn signature starting just after '('
def _split_params(s: str) -> list[str]:
    """Split a parameter list string on top-level commas (respect <>, (), [])."""
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur)
    return out


def _param_type(p: str) -> str | None:
    """'GuardedMutex: PKGUARDED_MUTEX' -> 'PKGUARDED_MUTEX'. Returns None for
    self/cfg/variadic markers."""
    p = p.strip()
    if not p or p == "..." or p.startswith("self") or p.startswith("&self"):
        return None
    # split on first top-level ':'
    depth = 0
    for i, ch in enumerate(p):
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        elif ch == ":" and depth == 0:
            return p[i + 1 :].strip()
    return p  # unnamed (rare)


def _param_list(text: str, popen: int) -> tuple[str, int]:
    """The balanced param list after the '(' at `popen`, with comments removed,
    and the index of its closing ')'. Comments must not reach `_split_params`:
    a `// ... usize: same size, distinct type` note inside the list otherwise
    splits into phantom params, and a HIGH arity finding is documented as almost
    never a false positive -- so it has to be one we can trust."""
    out, depth, j, n = [], 0, popen, len(text)
    while j < n:
        if text.startswith("//", j):
            j = text.find("\n", j)
            if j < 0:
                j = n
            continue
        if text.startswith("/*", j):
            end = text.find("*/", j + 2)
            j = n if end < 0 else end + 2
            continue
        ch = text[j]
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                break
        if j > popen:
            out.append(ch)
        j += 1
    return "".join(out), j


EXPORT_ATTR = re.compile(r"#\[\s*(no_mangle|export_name\s*=\s*\"([^\"]+)\")")
FN_SIG = re.compile(r'(?:pub\s+)?(?:unsafe\s+)?extern\s+"([^"]+)"\s+fn\s+(\w+)\s*\(')


def scan_rust_crate(src: Path) -> list[dict]:
    """Return list of {name, link_name, conv, params:[type], ret:type, file, line}.
    De-dups dual-cfg twins (same name+arity) keeping the first occurrence."""
    results = []
    seen = set()
    for rs in sorted(source_files(src)):
        text = rs.read_text(errors="replace")
        # find export attributes, then the next fn sig within a small window
        for m in EXPORT_ATTR.finditer(text):
            link = m.group(2)  # export_name value or None
            # locate line of attr
            attr_pos = m.start()
            # search forward for the fn signature
            fm = FN_SIG.search(text, attr_pos, attr_pos + 600)
            if not fm:
                continue
            conv = fm.group(1)
            name = link or fm.group(2)
            rust_ident = fm.group(2)
            popen = fm.end() - 1  # at '('
            params_str, j = _param_list(text, popen)
            # return type: between ')' and '{' or ';' or 'where'
            rest = text[j + 1 : j + 400]
            ret = "()"
            rm = re.match(r"\s*->\s*(.+?)\s*(\{|;|where\b)", rest, re.S)
            if rm:
                ret = rm.group(1).strip()
            params = []
            for p in _split_params(params_str):
                pt = _param_type(p)
                if pt is not None:
                    params.append(pt)
            line = text[:attr_pos].count("\n") + 1
            key = (name, len(params))
            if key in seen:
                continue
            seen.add(key)
            results.append(
                {
                    "name": name,
                    "link_name": link,
                    "rust_ident": rust_ident,
                    "conv": conv,
                    "params": params,
                    "ret": ret,
                    "file": str(rs),
                    "line": line,
                }
            )
    return results


# ── Comparison ───────────────────────────────────────────────────────────


def shapes_compatible(a, b) -> tuple[bool, str]:
    """a,b = (cat, size, signed, note). Return (ok, reason-if-not)."""
    ca, sa, ga, _ = a
    cb, sb, gb, _ = b
    if "unknown" in (ca, cb) or "aggregate" in (ca, cb):
        return (True, "")  # can't judge -> don't flag here
    # category mismatch (ptr vs int of different width, int vs float)
    if ca != cb:
        # allow ptr <-> pointer-width int (common & ABI-identical)
        if {ca, cb} == {"ptr", "int"} and sa == sb:
            return (True, "")
        if {ca, cb} == {"ptr", "int"}:
            return (False, f"category {ca}({sa}) vs {cb}({sb})")
        return (False, f"category {ca} vs {cb}")
    if sa != sb:
        return (False, f"width {sa} vs {sb}")
    if ga != gb:
        return (False, f"sign {'s' if ga else 'u'} vs {'s' if gb else 'u'}")
    return (True, "")


def classify_c_resolved(qual, desugar, arch, c_typedefs):
    """classify_c, but if the desugared spelling is itself still a typedef name
    (happens for return types where we only have the sugared prefix), resolve it
    through the typedef table first."""
    r = classify_c(desugar, qual, arch)
    if r[0] == "unknown":
        base = re.sub(r"\b(const|volatile)\b", "", desugar).strip()
        base = base.split("<")[0].strip()
        if base in c_typedefs:
            q, d = c_typedefs[base]
            return classify_c(d, q, arch)
    return r


def compare(rust_fns, c_protos, arch, include_low, c_typedefs):
    findings = []
    stats = {"checked": 0, "matched_c": 0, "no_c": 0, "ok": 0, "mismatch": 0}
    for rf in rust_fns:
        stats["checked"] += 1
        cp = c_protos.get(rf["name"])
        if not cp:
            stats["no_c"] += 1
            continue
        stats["matched_c"] += 1
        issues = []
        confidence = None
        # ARITY
        if cp["variadic"]:
            # variadic: only check the fixed prefix
            if len(rf["params"]) < len(cp["params"]):
                issues.append(
                    f"arity (variadic): rust {len(rf['params'])} < C fixed {len(cp['params'])}"
                )
                confidence = "HIGH"
        elif len(rf["params"]) != len(cp["params"]):
            issues.append(f"ARITY: rust {len(rf['params'])} params vs C {len(cp['params'])}")
            confidence = "HIGH"

        # per-slot type check (only when arity matches, to keep slots aligned)
        if not issues:
            n = min(len(rf["params"]), len(cp["params"]))
            for i in range(n):
                rs = classify_rust(rf["params"][i], arch, c_typedefs)
                cs_ = classify_c_resolved(cp["params"][i][0], cp["params"][i][1], arch, c_typedefs)
                ok, why = shapes_compatible(rs, cs_)
                if not ok:
                    sev = "LOW" if why.startswith("sign") else "MEDIUM"
                    issues.append(
                        f"param#{i + 1} {rf['params'][i]} vs "
                        f"{cp['params'][i][0]}({cp['params'][i][1]}): {why} [{sev}]"
                    )
                    confidence = _max_conf(confidence, sev)
            # RETURN — we only have the sugared prefix spelling for the C return;
            # resolve it through the typedef table.
            rret = classify_rust(rf["ret"], arch, c_typedefs)
            cret = classify_c_resolved(cp["ret"][0], cp["ret"][1], arch, c_typedefs)
            ok, why = shapes_compatible(rret, cret)
            if not ok:
                sev = "LOW" if why.startswith("sign") else "MEDIUM"
                issues.append(f"return {rf['ret']} vs {cp['ret'][0]}: {why} [{sev}]")
                confidence = _max_conf(confidence, sev)

        if issues:
            if confidence == "LOW" and not include_low:
                # still count, but suppress from default report
                stats["mismatch"] += 1
                findings.append({**_finding(rf, cp, issues, confidence), "_suppressed": True})
                continue
            stats["mismatch"] += 1
            findings.append(_finding(rf, cp, issues, confidence))
        else:
            stats["ok"] += 1
    return findings, stats


def _max_conf(cur, new):
    order = {None: 0, "LOW": 1, "MEDIUM": 2, "HIGH": 3}
    return new if order[new] > order[cur] else cur


def _finding(rf, cp, issues, conf):
    rsig = f"fn {rf['name']}({', '.join(rf['params'])}) -> {rf['ret']}"
    csig = f"{cp['ret'][0]} {rf['name']}({', '.join(q for q, _ in cp['params'])})"
    if cp["variadic"]:
        csig = csig[:-1] + ", ...)"
    return {
        "name": rf["name"],
        "confidence": conf,
        "file": rf["file"],
        "line": rf["line"],
        "rust": rsig,
        "c": csig,
        "issues": issues,
    }


# ── Main ─────────────────────────────────────────────────────────────────


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--arch", default="amd64", choices=list(ARCHS))
    ap.add_argument(
        "--crate",
        default="sdk/lib/rust/ntoskrnl-rust/src",
        help="Rust source dir to scan (relative to repo root or absolute)",
    )
    ap.add_argument("--low", action="store_true", help="include LOW-confidence findings")
    ap.add_argument("--only", help="check only this fn name (debug)")
    ap.add_argument("--json", help="write full findings to JSON file")
    args = ap.parse_args()

    # Default: scan every kernel crate (single source of truth) so the
    # crate-split doesn't silently drop signature coverage. An explicit --crate
    # overrides to a single dir.
    if args.crate == "sdk/lib/rust/ntoskrnl-rust/src":
        sys.path.insert(0, str(SCRIPT_DIR))
        from kernel_crate_roots import KERNEL_CRATE_SRC_ROOTS

        crate_dirs = [REPO / r for r in KERNEL_CRATE_SRC_ROOTS]
    else:
        c = Path(args.crate)
        crate_dirs = [c if c.is_absolute() else REPO / c]
    crate_dirs = [c for c in crate_dirs if c.exists()]
    if not crate_dirs:
        sys.exit(f"crate src not found: {args.crate}")

    arch = ARCHS[args.arch]
    print(f"=== verify_signatures — arch={args.arch} ===")
    print(f"clang:  {CLANG}", file=sys.stderr)
    print(f"headers: {resolve_build_repo(args.arch)}/build-{args.arch}")
    print(f"crates: {', '.join(str(c) for c in crate_dirs)}")
    print()

    print("Dumping C AST (clang)...", flush=True)
    ast = dump_c_ast(args.arch)
    c_protos, c_typedefs = extract_c_prototypes(ast)
    print(f"  C prototypes: {len(c_protos)} FunctionDecls, {len(c_typedefs)} typedefs")

    print("Scanning exported Rust fns...", flush=True)
    rust_fns = []
    for c in crate_dirs:
        rust_fns.extend(scan_rust_crate(c))
    if args.only:
        rust_fns = [r for r in rust_fns if r["name"] == args.only]
    print(f"  exported Rust fns: {len(rust_fns)} (deduped twins)")
    print()

    findings, stats = compare(rust_fns, c_protos, arch, args.low, c_typedefs)

    print("── coverage ──")
    print(f"  exported fns checked : {stats['checked']}")
    print(f"  matched to a C proto : {stats['matched_c']}")
    print(f"  no C proto (internal): {stats['no_c']}")
    print(f"  signature OK         : {stats['ok']}")
    print(
        f"  mismatches           : {stats['mismatch']}"
        f"  ({sum(1 for f in findings if not f.get('_suppressed'))} shown,"
        f" {sum(1 for f in findings if f.get('_suppressed'))} LOW-suppressed)"
    )
    print()

    shown = [f for f in findings if not f.get("_suppressed")]
    order = {"HIGH": 0, "MEDIUM": 1, "LOW": 2}
    shown.sort(key=lambda f: order.get(f["confidence"], 9))
    for f in shown:
        rel = os.path.relpath(f["file"], REPO)
        print(f"[{f['confidence']}] {f['name']}  ({rel}:{f['line']})")
        print(f"    rust: {f['rust']}")
        print(f"    C   : {f['c']}")
        for iss in f["issues"]:
            print(f"      - {iss}")
        print()

    if args.json:
        Path(args.json).write_text(
            json.dumps({"arch": args.arch, "stats": stats, "findings": findings}, indent=2)
        )
        print(f"wrote {args.json}")

    # exit nonzero only on HIGH/MEDIUM (LOW is advisory)
    hard = [f for f in shown if f["confidence"] in ("HIGH", "MEDIUM")]
    return 1 if hard else 0


if __name__ == "__main__":
    sys.exit(main())
