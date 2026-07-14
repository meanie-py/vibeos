#!/usr/bin/env python3
"""Compare each translated Rust function's signature against the C prototype it came from.

WHY THIS EXISTS
---------------
Translating C to Rust, the model reaches for a *plausible* typedef rather than the one the C
actually wrote — and it reaches for the POINTER flavour of a scalar with depressing regularity.
Every one of these shipped into ws2_32-rust and had to be caught by hand:

    C: INT      WSARemoveServiceClass(..)      Rust: -> LPINT        (6 functions)
    C: SIZE_T   ComputeQuerySetSize(..)        Rust: -> PSIZE_T      (3 functions)
    C: HANDLE   hLookup                        Rust: LPHANDLE        (2 exported fns)
    C: DWORD    dwCatalogEntryId               Rust: DWORD_PTR       (WPUModifyIFSHandle)
    C: INT      size1 = wcslen(..) + 1         Rust: LPINT           (a string length!)

None of these is caught by anything we already run:
  * rustc cannot see it — the C prototype is not in the Rust program.
  * verify_ffi.py compares a module's decls against ITS OWN .spec, and a .spec records only
    the name and the calling convention. It has no parameter types to check.
  * A boot test cannot see it — on 64-bit, LPINT/INT and DWORD_PTR/DWORD and HANDLE/LPHANDLE
    are all the same WIDTH, so the wrong one links, runs, and returns the right answer. The
    bug only bites on i386, or the day someone dereferences the "pointer".

So the oracle has to be the C source itself. That is what this does: for every `#[no_mangle]`
Rust function, find the C function of the same name in the paired .c file, and compare the
declared types position by position. It is a TYPE-NAME comparison, not a semantic one — it
flags a scalar translated as a pointer (and vice versa), which is precisely the failure mode.

WHAT IT WILL NOT DO
-------------------
It does not try to prove two type names are equivalent (INT vs c_int vs i32 are all fine, and
enumerating that equivalence is a rabbit hole). It only reports the asymmetry that matters:
one side is a POINTER and the other is not. That keeps the false-positive rate at zero on the
cohab fleet while still catching every bug in the list above.

USAGE
    verify_c_signatures.py --crate ws2_32-rust --csrc dll/win32/ws2_32/src [--check]
"""

from __future__ import annotations

import argparse
import re
import sys
from functools import lru_cache
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# A type is a POINTER if it is spelled as one, or is a Win32 typedef whose name says so.
# (The `P`/`LP` prefix convention is load-bearing in the Win32 headers and is exactly the
# convention the model gets wrong.)
_PTR_PREFIX = re.compile(r"^(?:LP|P)[A-Z]")

# Pointer typedefs are HARVESTED FROM THE HEADERS, not hand-listed.
#
# The hand-listed version cried wolf spectacularly: on its first fleet run it reported 77
# "bugs" in ole32/oleaut32 that were all its own ignorance — REFIID really is `const IID*`,
# BSTR really is `OLECHAR*`, LPUNKNOWN really is `IUnknown*`. A checker that does not know the
# type system it is checking produces a list of its own gaps and calls them defects. So resolve
# the typedefs the same way verify_conv_macros.py resolves convention aliases: read the headers
# and take the transitive closure.
_INC = [
    ROOT / "sdk" / "include" / "psdk",
    ROOT / "sdk" / "include" / "reactos",
    ROOT / "sdk" / "include" / "xdk",  # HANDLE/LPWSTR/PHANDLE live HERE, not psdk
]
# `typedef void *HANDLE, **PHANDLE;`  — a DECLARATOR LIST, and the interesting names are
# routinely not the first one. Parsing only a single declarator missed HANDLE, LPWSTR, PWSTR…
# One line, and NO braces: `[^;()]+?` with re.M happily ran across newlines into a
# `typedef struct { HANDLE QualityManager; ... }` body and harvested the STRUCT FIELDS as
# typedef names — which is how "HANDLE" ended up aliased to "{" and resolving to depth 0.
_TYPEDEF = re.compile(r"^[ \t]*typedef[ \t]+([^;(){}\n]+?)[ \t]*;[ \t]*$", re.M)
# `#define REFIID const IID* __MIDL_CONST`
_DEFINE_PTR = re.compile(
    r"^\s*#\s*define\s+([A-Za-z_]\w*)\s+(?:const\s+|CONST\s+)*([A-Za-z_]\w*)\s*(\*+)\s*(?:const|__MIDL_CONST|CONST)?\s*$",
    re.M,
)
# IDL attributes (`[unique]`, `[wire_marshal(wireBSTR)]`, `[string]`) sit exactly where the
# base type goes, so without stripping them BSTR resolves to "long" and LPUNKNOWN to nothing.
_DEFINE_ALIAS = re.compile(r"^\s*#\s*define\s+([A-Za-z_]\w*)\s+([A-Za-z_]\w*)\s*$", re.M)
_NOISE = re.compile(
    r"\[[^\]]*\]"  # IDL attributes
    r"|\b(?:const|CONST|volatile|struct|union|enum|__MIDL_CONST|FAR|NEAR)\b"
    r"|_[A-Za-z_]+_\([^()]*\)|\b_[A-Za-z_]+_\b"  # SAL
)


@lru_cache(maxsize=1)
def _typedefs() -> tuple[dict[str, int], dict[str, str]]:
    """(name -> stars, name -> base). Names declared INCONSISTENTLY across headers are dropped.

    `BSTR` is declared four times: three say `OLECHAR*`, and stdole2.idl says `long`. Picking
    one silently is the same "plausible wrong default" that this whole checker exists to catch,
    so a name whose declarations disagree on depth is marked AMBIGUOUS and excluded from the
    comparison — a blind spot we report, rather than a verdict we invent.
    """
    # (stars, base) per declaration. Ambiguity must be judged on the RESOLVED depth, not
    # the raw star count: WSAEVENT is declared `#define WSAEVENT HANDLE` (live, depth 1)
    # AND `typedef DWORD WSAEVENT` (dead WIN16 branch, depth 0) — both with ZERO stars.
    variants: dict[str, set[tuple[int, str]]] = {}
    stars: dict[str, int] = {}
    alias: dict[str, str] = {}
    for inc in _INC:
        if not inc.is_dir():
            continue
        for h in list(inc.rglob("*.h")) + list(inc.rglob("*.idl")):
            try:
                text = h.read_text(errors="replace")
            except OSError:
                continue
            for body in _TYPEDEF.findall(text):
                body = _NOISE.sub(" ", body).strip()
                parts = [d.strip() for d in body.split(",")]
                if not parts or not parts[0]:
                    continue
                head = parts[0].split()
                if len(head) < 2:
                    continue
                base = head[0]
                # first declarator keeps any stars glued to the name: `void *HANDLE`
                decls = [" ".join(head[1:]), *parts[1:]]
                for d in decls:
                    m = re.match(r"^(\**)\s*([A-Za-z_]\w*)", d.strip())
                    if not m:
                        continue
                    n_stars, name = len(m.group(1)), m.group(2)
                    if name == base:
                        continue
                    variants.setdefault(name, set()).add((n_stars, base))
                    stars.setdefault(name, n_stars)
                    alias.setdefault(name, base)
            # Object-macro aliases: `#define WSAEVENT HANDLE`. We do NOT preprocess, so a
            # header can hand us BOTH the live WIN32 spelling (#define WSAEVENT HANDLE, line
            # 579) and the dead WIN16 one (typedef DWORD WSAEVENT, line 600). Harvesting both
            # makes the name AMBIGUOUS -> skipped and reported, which is the honest outcome.
            # Taking whichever we saw first would have declared 9 correct ws2_32 functions buggy.
            for name, base in _DEFINE_ALIAS.findall(text):
                variants.setdefault(name, set()).add((0, base))
                alias.setdefault(name, base)
                stars.setdefault(name, 0)
            for name, base, st in _DEFINE_PTR.findall(text):
                variants.setdefault(name, set()).add((len(st), base))
                stars.setdefault(name, len(st))
                alias.setdefault(name, base)

    def _depth(t: str, seen: frozenset[str] = frozenset()) -> int:
        t = t.strip()
        if not t or t in seen:
            return 0
        if t in stars:
            return stars[t] + _depth(alias.get(t, ""), seen | {t})
        if t in alias:
            return _depth(alias[t], seen | {t})
        m = re.match(r"^(?:LP|P)([A-Z]\w*)$", t)
        return 1 + _depth(m.group(1), seen | {t}) if m else 0

    global AMBIGUOUS
    AMBIGUOUS = {
        n for n, vs in variants.items() if len({st + _depth(b, frozenset({n})) for st, b in vs}) > 1
    }
    for n in AMBIGUOUS:
        stars.pop(n, None)
        alias.pop(n, None)
    return stars, alias


AMBIGUOUS: set[str] = set()


# Typedefs that LOOK like pointers by name but are scalars. `PASCAL` is a calling convention.
_SCALAR_P_WHITELIST = {"PASCAL"}


class Ambiguous(Exception):
    """The headers disagree about this typedef's indirection; we refuse to guess."""


def ptr_depth(t: str, _seen: frozenset[str] = frozenset()) -> int:
    """Levels of indirection, resolved THROUGH typedefs.

    Two lessons are baked in here, both learned by this checker being wrong:
      * Depth, not pointer-vs-scalar. The first version only asked "is one a pointer and the
        other not"; a mutation test proved it blind to HANDLE-vs-LPHANDLE, the bug that
        motivated it, because BOTH are pointers. The defect is a depth difference.
      * Compositional, not a star-count. `*mut LPVOID` is `void**` (depth 2). Counting `*`
        characters says 1, and that mis-read is what produced dozens of phantom findings.
    """
    t = strip_sal(t).strip()
    if not t:
        return 0
    if "*" in t:
        # Rust `*mut *const T` / C `OLECHAR **` — the stars, PLUS whatever the pointee is.
        stars = t.count("*")
        pointee = re.sub(r"[*]|\bmut\b|\bconst\b", " ", t).split()
        pointee = pointee[-1] if pointee else ""
        return stars + (ptr_depth(pointee, _seen) if pointee else 0)
    bare = re.sub(r"\b(?:const|volatile|struct|CONST)\b", " ", t).strip()
    bare = bare.split("::")[-1]  # core::ffi::c_void -> c_void
    if not bare or bare in _SCALAR_P_WHITELIST or bare in _seen:
        return 0
    stars, alias = _typedefs()
    if bare in AMBIGUOUS:
        raise Ambiguous(bare)
    seen = _seen | {bare}
    if bare in stars:
        return stars[bare] + ptr_depth(alias.get(bare, ""), seen)
    if bare in alias:
        return ptr_depth(alias[bare], seen)
    # Not in the headers we scanned (many COM types live in .idl we do not parse). Fall back to
    # the Win32 naming convention, which is the whole reason the P/LP prefixes exist.
    m = re.match(r"^(?:LP|P)([A-Z]\w*)$", bare)
    if m:
        return 1 + ptr_depth(m.group(1), seen)
    return 0


def is_pointer(t: str) -> bool:
    return ptr_depth(t) > 0


# ---- C side -----------------------------------------------------------------------------

# Two C definition styles live in this tree and the checker must read BOTH, or it silently
# reports "0 findings" against ~0 coverage (which is exactly what it did for ole32/oleaut32
# on its first fleet run — a green from a blind oracle):
#   ReactOS:  INT\nWSAAPI\nWSALookupServiceNextW(IN HANDLE h, ..)\n{     <- name at column 0
#   Wine:     HRESULT WINAPI CoInitialize(LPVOID pvReserved)\n{           <- all on one line
# Anchor on "a line starting at column 0 that ends in a param list followed by `{`", and take
# the LAST identifier before the paren as the name.
_C_DEF = re.compile(r"^((?:[A-Za-z_][\w \t]*[ \t\*]+)?)(\w+)\s*\(([^;{]*?)\)\s*\n?\{", re.S | re.M)
_C_RET = re.compile(r"^([A-Za-z_][\w ]*?)\s*$", re.M)
# SAL comes in two shapes and you need both: the bare annotation (`_In_`, `IN`, `OPTIONAL`)
# and the PARAMETERISED one (`_Out_writes_bytes_(sizeof(IN_ADDR6))`). Stripping only the bare
# form leaves `_Out_writes_(StringBufSize) PSTR`, which does not look like a pointer to any
# name-based test — the checker's first run reported 6 of these as bugs. They were not.
_COMMENT = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
_DECLSPEC = re.compile(
    r"\b(?:DECLSPEC_\w+|WINAPI|WSAAPI|WSPAPI|APIENTRY|__RPC_USER|__RPC_STUB|RPC_ENTRY|WINAPIV)\b"
)
_SAL_PARAM = re.compile(r"\b_[A-Za-z_]+_\s*\([^()]*(?:\([^()]*\)[^()]*)*\)")
_SAL_BARE = re.compile(r"\b(?:_[A-Za-z_]+_|IN|OUT|OPTIONAL|FAR|NEAR|CONST)\b")


def strip_sal(t: str) -> str:
    t = _COMMENT.sub(" ", t)
    t = _DECLSPEC.sub(" ", t)
    return _SAL_BARE.sub("", _SAL_PARAM.sub("", t)).strip()


def c_functions(path: Path) -> dict[str, tuple[str, list[str]]]:
    """name -> (return type, [param types])"""
    text = _COMMENT.sub(" ", path.read_text(errors="replace"))
    out: dict[str, tuple[str, list[str]]] = {}
    CONV = (
        "WSAAPI",
        "WSPAPI",
        "WINAPI",
        "NTAPI",
        "APIENTRY",
        "PASCAL",
        "FAR",
        "CALLBACK",
        "STDMETHODCALLTYPE",
        "inline",
        "static",
        "__cdecl",
        "__stdcall",
        "__RPC_USER",
        "__RPC_STUB",
        "RPC_ENTRY",
        "CDECL",
        "WINAPIV",
        "DECLSPEC_NORETURN",
    )
    for m in _C_DEF.finditer(text):
        lead, name, params = m.group(1), m.group(2), m.group(3)
        if name in ("if", "while", "for", "switch", "return", "sizeof"):
            continue
        # Return type: whatever precedes the name on this line, else the line(s) above.
        ret = " ".join(w for w in lead.replace("*", " * ").split() if w not in CONV).strip()
        ret = _DECLSPEC.sub("", ret).strip()
        if not ret:
            head = text[: m.start()].rstrip().split("\n")
            for line in reversed(head[-6:]):
                line = line.strip()
                if (
                    not line
                    or line in CONV
                    or re.fullmatch(r"DECLSPEC_\w+|__declspec\s*\(.*\)", line)
                ):
                    continue
                if re.fullmatch(r"[A-Za-z_][\w *]*", line):
                    ret = line
                break
        ptypes = []
        for p in params.split(","):
            p = strip_sal(p)
            if not p or p == "void":
                continue
            # strip the parameter NAME (last identifier) to leave the type; an ARRAY
            # parameter decays to a pointer, so it carries one extra level of indirection.
            decayed = bool(re.search(r"\[\s*\d*\s*\]\s*$", p))
            p = re.sub(r"\b\w+\s*(\[\s*\d*\s*\])?$", "", p).strip()
            ptypes.append(p + " *" if decayed else p)
        out[name] = (ret.strip(), ptypes)
    return out


# ---- Rust side --------------------------------------------------------------------------

_R_DEF = re.compile(
    r"#\[no_mangle\]\s*\npub unsafe extern \"[^\"]+\" fn (\w+)"
    r"\(([^)]*)\)\s*(?:->\s*([^{]+?))?\s*\{",
    re.S,
)


def rust_functions(path: Path) -> dict[str, tuple[str, list[str]]]:
    out: dict[str, tuple[str, list[str]]] = {}
    for m in _R_DEF.finditer(path.read_text()):
        name, params, ret = m.group(1), m.group(2), (m.group(3) or "").strip()
        ptypes = []
        for p in params.split(","):
            p = p.strip()
            if not p:
                continue
            if ":" in p:
                ptypes.append(p.split(":", 1)[1].strip())
        out[name] = (ret, ptypes)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--crate", help="e.g. ws2_32-rust")
    ap.add_argument("--csrc", help="e.g. dll/win32/ws2_32")
    ap.add_argument(
        "--all", action="store_true", help="sweep every cohab crate that pairs with a dll/ dir"
    )
    ap.add_argument("--check", action="store_true", help="exit 1 on any finding")
    args = ap.parse_args()

    if args.all:
        rc = 0
        for cdir_rs in sorted((ROOT / "sdk" / "lib" / "rust").glob("*-rust")):
            name = cdir_rs.name[:-5]
            ddir = next((d for d in (ROOT / "dll").rglob(name) if d.is_dir()), None)
            if ddir is None:
                continue
            rc |= _run(cdir_rs / "src", ddir, name, args.check)
        return rc

    if not args.crate or not args.csrc:
        print("FAIL: need --crate and --csrc, or --all", file=sys.stderr)
        return 2
    rdir = ROOT / "sdk" / "lib" / "rust" / args.crate / "src"
    cdir = ROOT / args.csrc
    if not rdir.is_dir() or not cdir.is_dir():
        print(f"FAIL: missing {rdir if not rdir.is_dir() else cdir}", file=sys.stderr)
        return 2

    return _run(rdir, cdir, args.crate, args.check)


def _run(rdir: Path, cdir: Path, label: str, check: bool) -> int:
    # Crate module layouts differ across the fleet (ws2_32 uses src/src/, msafd uses src/misc/,
    # ole32 is flat), so pair by FILE STEM anywhere under each tree rather than by fixed path.
    cfiles = {c.stem: c for c in cdir.rglob("*.c")}

    findings = []
    compared = 0
    skipped = 0
    for rf in sorted(rdir.rglob("*.rs")):
        cf = cfiles.get(rf.stem)
        if cf is None:
            continue
        cfns, rfns = c_functions(cf), rust_functions(rf)
        for name, (rret, rparams) in rfns.items():
            if name not in cfns:
                continue
            compared += 1
            cret, cparams = cfns[name]
            try:
                if cret and ptr_depth(cret) != ptr_depth(rret):
                    findings.append((rf.name, name, "return", cret, rret))
            except Ambiguous:
                skipped += 1
            if len(cparams) != len(rparams):
                continue  # arity drift is a different (louder) bug; rustc/link will catch it
            for i, (ct, rt) in enumerate(zip(cparams, rparams, strict=False)):
                try:
                    if ct and ptr_depth(ct) != ptr_depth(rt):
                        findings.append((rf.name, name, f"param {i}", ct, rt))
                except Ambiguous:
                    skipped += 1

    for f, fn, where, ct, rt in findings:
        print(f"{label}/{f}: {fn} {where}: C `{ct}` vs Rust `{rt}`")
    if compared:
        print(
            f"{label}: {len(findings)} finding(s), {compared} pairs compared, "
            f"{skipped} ambiguous type(s) skipped",
            file=sys.stderr,
        )
    # ALWAYS report coverage. A checker that silently matched ZERO functions also prints
    # "0 findings", and that is the failure mode that makes a green oracle worthless.
    return 1 if (findings and check) else 0


if __name__ == "__main__":
    raise SystemExit(main())
