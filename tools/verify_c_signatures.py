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
enumerating that equivalence is a rabbit hole). It compares two things only:
  * INDIRECTION DEPTH — a scalar translated as a pointer, or HANDLE as LPHANDLE.
  * ARCH-VARIABILITY — one side pointer-sized (`DWORD_PTR`, `SIZE_T`, `usize`), the other
    fixed-width. Depth alone is blind to this: `DWORD_PTR` is not a pointer, it is an integer
    that is 4 bytes on i386 and 8 on amd64, so `DWORD` -> `DWORD_PTR` is a silent ABI break
    that this checker ADVERTISED catching while in fact sailing straight past it. A mutation
    test is what exposed that, not review.

THE BASELINE IS TRIAGED, NOT BLESSED (2026-07-14)
-------------------------------------------------
Every line in tools/lint-baseline/verify_c_signatures.txt was read against its C before being
parked. None is a live defect; the reasons differ and are worth knowing:
  * lsasrv LsapRegCreateKey/LsapRegOpenKey param 3 — THE RUST IS RIGHT AND THE C IS WRONG.
    `OUT HANDLE KeyHandle` is under-typed: all ~20 callers pass `&handle`, and the callee
    (ZwCreateKey/NtOpenKey) takes PHANDLE. The C only compiles because `void*` implicitly
    converts to `void**`. Do not "fix" the Rust to match it.
  * cabinet/mscms `HFDI`/`HPROFILE`/`HTRANSFORM` vs `isize`, ntdll ETW `PEVENT_TRACE_PROPERTIES *`
    vs `PVOID` — handle-as-integer / pointer-flattening translation conventions. ABI-identical
    (all one machine word) and cast back at the use site, so no runtime effect; fixing them is
    crate-wide churn for zero behaviour change. Owed as cosmetic, not as a bug.
The one REAL defect this triage found — ws2_32 WPUCompleteOverlappedRequest's DWORD_PTR
dwError/cbTransferred, contradicted by the C, ws2spi.h AND the .spec — is fixed, not baselined.

USAGE
    verify_c_signatures.py --crate ws2_32-rust --csrc dll/win32/ws2_32/src [--check]
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from functools import lru_cache
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

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
# Excluding braces (above) keeps the struct BODY from being read as declarators, but it also
# threw away the declarator LIST that follows the closing brace — which is where most of Win32
# lives. The cost was not a gap but a WRONG ANSWER: with the true `*LPSTARTUPINFOW` invisible,
# cordebug.idl's marshalling placeholder (`typedef SIZE_T LPSTARTUPINFOW;`) was the only
# declaration the checker could see, so it stood unopposed, looked unanimous, resolved to
# depth 0, and reported the CORRECT advapi32 translation as a bug. The ambiguity machinery
# could not save us: it fires on conflicting declarations, and a parser blind to the true one
# sees no conflict. A checker that can only see one source cannot tell that it is the wrong one.
#
# This needs BRACE MATCHING, not a regex. `} NAME;` is genuinely ambiguous out of context:
#     typedef struct { .. } NAME;        <- NAME is a type
#     union { struct { .. } NAME; ... }  <- NAME is a FIELD  (rpcasync.h really does this,
#                                           with a member literally named HWND)
# A `^}` regex reads both the same way, which is how the first cut of this harvest declared
# HWND ambiguous — the same "struct fields harvested as typedef names" bug the flat-typedef
# comment above already warns about, reintroduced in a new costume.
_MAX_TYPEDEF_SPAN = 200_000  # a runaway guard; the largest real one is far under this


def _braced_typedefs(text: str) -> list[str]:
    """Declarator lists of every `typedef <aggregate> { ... } <declarators>;` in `text`.

    Walks from each `typedef` keeping brace depth, so only the closing brace that returns to
    depth 0 ends the aggregate — a nested member's `}` cannot be mistaken for the typedef's.
    """
    out: list[str] = []
    for m in re.finditer(r"\btypedef\b", text):
        depth, decl_start, i = 0, None, m.end()
        end = min(len(text), i + _MAX_TYPEDEF_SPAN)
        while i < end:
            c = text[i]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    decl_start = i + 1
                elif depth < 0:
                    break
            elif depth == 0 and c == "(":
                break  # function-pointer typedef — a different shape, not our business
            elif depth == 0 and c == ";":
                if decl_start is not None:
                    out.append(text[decl_start:i])
                break
            i += 1
    return out


# `DECLARE_HANDLE(HMENU);` -> `typedef struct HMENU__{int unused;} *HMENU` (or `typedef HANDLE
# HMENU` in the non-STRICT branch). BOTH spellings are depth 1, so this is unambiguous — but
# unharvested it left all 82 of them at depth 0, i.e. the entire GDI/USER handle family
# (HDC, HMENU, HFONT, HWND, HBITMAP…) silently mis-typed as scalars.
_DECLARE_HANDLE = re.compile(r"^\s*DECLARE_HANDLE\s*\(\s*([A-Za-z_]\w*)\s*\)", re.M)
# Aggregate TAG names: `struct _FOO {`, `typedef struct _FOO *PFOO;`. A tag is depth 0 by
# definition. Harvesting them is what lets `typedef struct _FOO *PFOO` resolve at all — the
# tag is not itself a typedef, so without this PFOO's pointee is an unknown name and the
# entire P<struct> family becomes unresolvable.
_TAG = re.compile(r"\b(?:struct|union|enum)\s+([A-Za-z_]\w*)")
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
# Words that QUALIFY a type rather than name one. `unsigned long` is one type, not a type
# called `unsigned` applied to a type called `long`, and `_W64`/`__w64` are MSVC portability
# markers that name nothing at all.
_MODIFIER = re.compile(
    r"\b(?:_W64|__w64|unsigned|signed|const|volatile|struct|union|enum|register|"
    r"__declspec|__cdecl|__stdcall|CONST)\b"
)


def _norm_base(s: str) -> str:
    """Reduce a possibly multi-word base type to the one word that names it.

    `_W64 unsigned long` -> `long`; `unsigned __int64` -> `__int64`; `void` -> `void`.
    An all-modifier base (`unsigned`) reduces to the implied `int`.
    """
    words = _MODIFIER.sub(" ", s).split()
    return words[-1] if words else "int"


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
    variants: dict[str, set[tuple[int, str, bool]]] = {}
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
            idl = h.suffix == ".idl"
            text = _COMMENT.sub(" ", text)  # a `typedef` inside a comment is not a typedef
            TAGS.update(_TAG.findall(text))
            # A braced typedef's tail names an aggregate: `} FOO, *LPFOO;`. The struct itself
            # is depth 0 by definition; each `*` declarator is one level above it.
            for tail in _braced_typedefs(text):
                decls = [d.strip() for d in _NOISE.sub(" ", tail).split(",")]
                base = next((d for d in decls if d and not d.startswith("*")), "")
                if not base or not re.fullmatch(r"[A-Za-z_]\w*", base):
                    continue
                variants.setdefault(base, set()).add((0, "", idl))
                stars.setdefault(base, 0)
                for d in decls:
                    m = re.match(r"^(\*+)\s*([A-Za-z_]\w*)$", d)
                    if not m:
                        continue
                    variants.setdefault(m.group(2), set()).add((len(m.group(1)), base, idl))
                    stars.setdefault(m.group(2), len(m.group(1)))
                    alias.setdefault(m.group(2), base)
            for name in _DECLARE_HANDLE.findall(text):
                variants.setdefault(name, set()).add((1, "", idl))
                stars.setdefault(name, 1)
            for body in _TYPEDEF.findall(text):
                body = _NOISE.sub(" ", body).strip()
                parts = [d.strip() for d in body.split(",")]
                if not parts or not parts[0]:
                    continue
                # The base type may be MANY words (`unsigned long`, `_W64 unsigned __int64`),
                # so it is everything up to the LAST identifier; the last identifier is the
                # first declarator. Taking head[0] as the base instead harvested `unsigned` as
                # a typedef name aliased to `_W64` — which poisoned the whole integer tower:
                # SIZE_T -> ULONG_PTR -> unsigned -> _W64 -> __w64 -> unresolvable.
                # Split parts[0] into "everything before the trailing identifier" + that
                # identifier. C has no such thing as a star on a BASE type: in both
                # `void *PVOID` and `void* PVOID` the star belongs to the declarator. Reading
                # the second spelling as base=`void*`/stars=0 made PVOID resolve to depth 0 in
                # stubs.h and depth 1 everywhere else, so the single most common pointer type
                # in the tree was declared AMBIGUOUS and skipped.
                m0 = re.fullmatch(r"(.*?)([A-Za-z_]\w*)\s*", parts[0], re.S)
                if not m0 or not m0.group(1).strip():
                    continue
                prefix, first = m0.group(1), m0.group(2)
                base = _norm_base(prefix.replace("*", " "))
                decls = [("*" * prefix.count("*")) + first, *parts[1:]]
                for d in decls:
                    m = re.fullmatch(r"(\**)\s*([A-Za-z_]\w*)", d.strip())
                    if not m:
                        continue
                    n_stars, name = len(m.group(1)), m.group(2)
                    if name == base:
                        continue
                    variants.setdefault(name, set()).add((n_stars, base, idl))
                    stars.setdefault(name, n_stars)
                    alias.setdefault(name, base)
            # Object-macro aliases: `#define WSAEVENT HANDLE`. We do NOT preprocess, so a
            # header can hand us BOTH the live WIN32 spelling (#define WSAEVENT HANDLE, line
            # 579) and the dead WIN16 one (typedef DWORD WSAEVENT, line 600). Harvesting both
            # makes the name AMBIGUOUS -> skipped and reported, which is the honest outcome.
            # Taking whichever we saw first would have declared 9 correct ws2_32 functions buggy.
            for name, base in _DEFINE_ALIAS.findall(text):
                variants.setdefault(name, set()).add((0, base, idl))
                alias.setdefault(name, base)
                stars.setdefault(name, 0)
            for name, base, st in _DEFINE_PTR.findall(text):
                variants.setdefault(name, set()).add((len(st), base, idl))
                stars.setdefault(name, len(st))
                alias.setdefault(name, base)

    def _depth(t: str, seen: frozenset[str] = frozenset()) -> int:
        t = t.strip()
        if not t or t in seen or t in TAGS:
            return 0
        if t in stars:
            return stars[t] + _depth(alias.get(t, ""), seen | {t})
        if t in alias:
            return _depth(alias[t], seen | {t})
        return 1 if _lp_of(t, stars, alias) else 0

    # A .h DECLARATION OUTRANKS AN .idl ONE. This is not a tie-break heuristic, it is what the
    # comparison is ABOUT: we are checking a Rust translation against the C that the compiler
    # saw, and the compiler that built advapi32/security.c included the psdk headers — it never
    # saw cordebug.idl. An IDL typedef states an RPC WIRE representation, and IDL routinely
    # types a struct it cannot marshal as an opaque integer (`typedef SIZE_T LPSTARTUPINFOW;`).
    # Letting that outvote processthreadsapi.h's real `*LPSTARTUPINFOW` is a category error.
    # IDL is still consulted where it is the ONLY declaration — BSTR and LPUNKNOWN genuinely
    # live nowhere else — and where IDL files disagree among THEMSELVES, ambiguity still wins.
    for n, vs in variants.items():
        if any(not is_idl for _, _, is_idl in vs):
            variants[n] = {v for v in vs if not v[2]}
    # Re-resolve from the .h-only view before judging ambiguity.
    stars.clear()
    alias.clear()
    for n, vs in variants.items():
        st, base, _ = next(iter(vs))
        stars[n] = st
        if base:
            alias[n] = base

    global AMBIGUOUS
    AMBIGUOUS = {
        n
        for n, vs in variants.items()
        if len({st + _depth(b, frozenset({n})) for st, b, _ in vs}) > 1
    }
    for n in AMBIGUOUS:
        stars.pop(n, None)
        alias.pop(n, None)
    return stars, alias


AMBIGUOUS: set[str] = set()
TAGS: set[str] = set()


def _lp_of(t: str, stars: dict[str, int], alias: dict[str, str]) -> str | None:
    """`LPFOO`/`PFOO` -> `FOO`, but ONLY when FOO is a type we actually harvested.

    The P/LP prefix convention is a real convention, but it is a convention about NAMES, and
    plenty of ordinary type names simply begin with a capital P. Applied blind, this rule read
    `POINT` as a pointer to `OINT`, and did the same to PROCESS_INFORMATION, PALETTEENTRY,
    PIXELFORMATDESCRIPTOR, POLYTEXTW, PERF_DATA_BLOCK — every P-initial struct in Win32.
    Requiring the remainder to name a KNOWN type is what separates `PHANDLE` (HANDLE exists ->
    really a pointer) from `POINT` (no type `OINT` -> just a name that starts with P).
    """
    m = re.match(r"^(?:LP|P)([A-Z]\w*)$", t)
    if not m:
        return None
    base = m.group(1)
    return base if (base in stars or base in alias) else None


# Typedefs that LOOK like pointers by name but are scalars. `PASCAL` is a calling convention.
_SCALAR_P_WHITELIST = {"PASCAL"}

# Types that are depth 0 by definition, so an unresolvable name can be distinguished from a
# genuine scalar. Everything else that resolves to nothing is UNKNOWN -> Ambiguous -> skipped
# and counted, rather than silently assumed to be a scalar. Guessing 0 for a name we never
# found is precisely the "plausible wrong default" this checker exists to catch, and it is a
# guess that fails in the FALSE-GREEN direction: an unknown C type and an unknown Rust type
# both land on 0 and compare equal, so the pair is reported as verified when nothing was
# checked at all.
_KNOWN_SCALARS = {
    # C / Win32 primitives and their upper-case spellings
    "void", "char", "short", "int", "long", "float", "double", "signed", "unsigned",
    "size_t", "wchar_t", "__int64", "__int32", "__int16", "__int8", "bool", "_Bool",
    "int8_t", "int16_t", "int32_t", "int64_t", "uint8_t", "uint16_t", "uint32_t",
    "uint64_t", "intptr_t", "uintptr_t", "ptrdiff_t", "intmax_t", "uintmax_t",
    "VOID", "CHAR", "SHORT", "INT", "LONG", "FLOAT", "DOUBLE", "BYTE", "WORD", "DWORD",
    "BOOL", "BOOLEAN", "UCHAR", "USHORT", "UINT", "ULONG", "LONGLONG", "ULONGLONG",
    "INT8", "INT16", "INT32", "INT64", "UINT8", "UINT16", "UINT32", "UINT64",
    "LONG32", "LONG64", "ULONG32", "ULONG64", "SIZE_T", "SSIZE_T", "DWORD_PTR",
    "ULONG_PTR", "LONG_PTR", "UINT_PTR", "INT_PTR", "NTSTATUS", "HRESULT", "WPARAM",
    "LPARAM", "LRESULT", "COLORREF", "ACCESS_MASK", "LCID", "LANGID", "ATOM", "WCHAR",
    "TCHAR", "DWORDLONG", "QWORD", "FARPROC", "PROC", "SOCKET", "GROUP",
    # Rust primitives
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
    "f32", "f64", "c_void", "c_char", "c_schar", "c_uchar", "c_short", "c_ushort",
    "c_int", "c_uint", "c_long", "c_ulong", "c_longlong", "c_ulonglong", "c_float",
    "c_double", "bool_", "char16_t",
}  # fmt: skip


class Unknown(Exception):
    """The name resolves to no declaration we found; we refuse to assume it is a scalar."""


# The POINTER-SIZED integer family. These are not pointers — `DWORD_PTR` is "a DWORD wide
# enough to hold a pointer" — so a DEPTH comparison is structurally blind to them, and the
# `DWORD dwCatalogEntryId` -> `DWORD_PTR` bug that this checker's own docstring advertises
# was in fact sailing straight through it. The defect is WIDTH: 4 bytes vs 8 on amd64.
#
# Width cannot be compared numerically here, because the headers carry BOTH sides of the
# #ifdef (`typedef unsigned __int64 ULONG_PTR` AND `typedef unsigned long ULONG_PTR`) and we
# do not preprocess. But the defect does not need a number: what makes it a bug is that one
# side is ARCH-VARIABLE and the other is FIXED. That is decidable from the name alone.
_PTR_SIZED = {
    "SIZE_T", "SSIZE_T", "DWORD_PTR", "ULONG_PTR", "LONG_PTR", "INT_PTR", "UINT_PTR",
    "HALF_PTR", "UHALF_PTR", "WPARAM", "LPARAM", "LRESULT", "usize", "isize",
    # The C spellings too: `inet_ntop(.., size_t StringBufSize)` is pointer-sized just as
    # much as SIZE_T is, and omitting them made the CORRECT `usize` translation a finding.
    "size_t", "ssize_t", "ptrdiff_t", "intptr_t", "uintptr_t",
}  # fmt: skip


def is_ptr_sized(t: str) -> bool:
    """True if `t` resolves through any pointer-sized integer typedef."""
    bare = _norm_base(strip_sal(t)).split("::")[-1]
    if "*" in t:
        return False  # a real pointer; depth already covers it
    _, alias = _typedefs()
    seen: set[str] = set()
    while bare and bare not in seen:
        if bare in _PTR_SIZED:
            return True
        seen.add(bare)
        bare = alias.get(bare, "")
    return False


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
      * A name we cannot resolve raises Unknown; it does NOT default to 0. See _KNOWN_SCALARS.
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
    bare = _norm_base(t)
    bare = bare.split("::")[-1]  # core::ffi::c_void -> c_void
    if not bare or bare in _SCALAR_P_WHITELIST or bare in _seen:
        return 0
    stars, alias = _typedefs()
    if bare in AMBIGUOUS:
        raise Ambiguous(bare)
    if bare in TAGS:
        return 0  # a struct/union/enum tag is an aggregate, not a pointer
    seen = _seen | {bare}
    if bare in stars:
        return stars[bare] + ptr_depth(alias.get(bare, ""), seen)
    if bare in alias:
        return ptr_depth(alias[bare], seen)
    if bare in _KNOWN_SCALARS:
        return 0
    # Not in the headers we scanned. The P/LP naming convention is only trusted when the
    # remainder names a type we actually found (`PHANDLE` -> HANDLE exists); otherwise this is
    # simply a name we do not know, and we say so rather than assuming it is a scalar.
    if _lp_of(bare, stars, alias):
        return 1 + ptr_depth(_lp_of(bare, stars, alias), seen)
    raise Unknown(bare)


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
        "SEC_ENTRY",  # sspi.h: __stdcall. ksecdd spells it on its own line, so the
        # last token before the name was read as the return type.
        "CDECL",
        "WINAPIV",
        "DECLSPEC_NORETURN",
        # A module-private ALIAS for a convention is still a convention. deskcplx.h:4 has
        # `#define DESK_EXT_CALLBACK CALLBACK`, and without it the return type of
        # `PDEVMODEW DESK_EXT_CALLBACK CDevSettings_GetCurrentMode(..)` reads as the two tokens
        # `PDEVMODEW DESK_EXT_CALLBACK`, which no Rust return type can ever match — a permanent
        # false positive on a correctly-translated function. Same class as netcfgx's STDAPI.
        "DESK_EXT_CALLBACK",
        # vfw.h: `#define VFWAPI WINAPI` -- avicap32's `HWND VFWAPI capCreateCaptureWindowW(..)`.
        "VFWAPI",
        "VFWAPIV",
    )

    def _clean_ret(s: str) -> str:
        """Drop calling conventions / declspecs from a return type, whichever path found it."""
        s = " ".join(w for w in s.replace("*", " * ").split() if w not in CONV)
        return _DECLSPEC.sub("", s).strip()

    for m in _C_DEF.finditer(text):
        lead, name, params = m.group(1), m.group(2), m.group(3)
        if name in ("if", "while", "for", "switch", "return", "sizeof"):
            continue
        # Return type: whatever precedes the name on this line, else the line(s) above.
        ret = _clean_ret(lead)
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
                    # ...and this line needs the SAME cleaning as the inline path. It did not
                    # get it, so a two-line `PVOID\nNTAPI\nRtlPcToFileHeader(..)` yielded the
                    # return type "PVOID NTAPI" — a calling convention read as part of a type.
                    ret = _clean_ret(line)
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
            # dll/ AND drivers/. Pairing only against dll/ silently excluded every
            # translated driver — fastfat, cdfs, classpnp, usbport, the whole storage
            # and USB stack — ~50 crates and 3,000 signature pairs that read as
            # covered because the sweep is spelled `--all`. A kernel-mode extra `*`
            # is worse than a usermode one, not better.
            ddir = None
            for croot in ("dll", "drivers"):
                ddir = next((d for d in (ROOT / croot).rglob(name) if d.is_dir()), None)
                if ddir is not None:
                    break
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
    unknown: Counter[str] = Counter()

    def _differs(ct: str, rt: str) -> bool:
        """True iff both sides resolve AND disagree. Raises to signal a blind spot."""
        if ptr_depth(ct) != ptr_depth(rt):
            return True
        # Same indirection, but one side is arch-variable and the other is not.
        return is_ptr_sized(ct) != is_ptr_sized(rt)

    for rf in sorted(source_files(rdir)):
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
                if cret and _differs(cret, rret):
                    findings.append((rf.name, name, "return", cret, rret))
            except Ambiguous:
                skipped += 1
            except Unknown as u:
                skipped += 1
                unknown[str(u)] += 1
            if len(cparams) != len(rparams):
                continue  # arity drift is a different (louder) bug; rustc/link will catch it
            for i, (ct, rt) in enumerate(zip(cparams, rparams, strict=False)):
                try:
                    if ct and _differs(ct, rt):
                        findings.append((rf.name, name, f"param {i}", ct, rt))
                except Ambiguous:
                    skipped += 1
                except Unknown as u:
                    skipped += 1
                    unknown[str(u)] += 1

    for f, fn, where, ct, rt in findings:
        print(f"{label}/{f}: {fn} {where}: C `{ct}` vs Rust `{rt}`")
    if compared:
        # ALWAYS report coverage AND the blind spots by name. "0 findings" from an oracle that
        # skipped everything is indistinguishable from "0 findings" from one that checked
        # everything, unless it tells you what it could not resolve.
        top = ", ".join(f"{n}x{c}" for n, c in unknown.most_common(5))
        print(
            f"{label}: {len(findings)} finding(s), {compared} pairs compared, "
            f"{skipped} type(s) skipped as unresolvable"
            + (f" (top unknown: {top})" if top else ""),
            file=sys.stderr,
        )
    # ALWAYS report coverage. A checker that silently matched ZERO functions also prints
    # "0 findings", and that is the failure mode that makes a green oracle worthless.
    return 1 if (findings and check) else 0


if __name__ == "__main__":
    raise SystemExit(main())
