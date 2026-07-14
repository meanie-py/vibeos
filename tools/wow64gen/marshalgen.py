#!/usr/bin/env python3
"""marshalgen — wow64gen stage 2: bulk 32<->64 deep-marshaller generation.

wow64gen (stage 1) emits flat width-widening thunks and flags syscalls whose
args defeat widening as WOW64_NEEDS_MANUAL stubs (~158 today). This tool eats
that residue mechanically:

  1. LAYOUTS: compile one TU through the real header stack TWICE
     (--target i686-w64-mingw32 and aarch64-w64-mingw32) with
     `-Xclang -fdump-record-layouts-complete`, parsing clang's authoritative
     per-field offsets. No hand-maintained struct tables, no drift: the same
     compiler that builds the guest and host DLLs reports the layouts.
  2. PROTOTYPES: extract Nt* parameter lists (type per arg) from the same
     headers textually (SAL is stripped by mingw's no_sal, so direction is
     heuristic — see below).
  3. DIFF+CLASSIFY each pointed-to struct across the two widths:
       IDENTICAL    same size + same field offsets, no divergent members
                    -> the flat thunk was already correct; no marshal needed.
       WIDENING     field names align 1:1 and every divergence is a 4->8
                    pointer/handle/SIZE_T widening -> fully auto-generatable
                    field-copy marshaller (the WINDOWPOS pattern).
       SEMANTIC     unions / bitfields / flex arrays / mismatched field lists
                    -> stays manual; report prints the layout diff so a human
                    (or a smarter generator pass) starts from facts.
  4. (codegen pass) emit Wow64AutoMarshal_<Struct>_{In,Out} pairs + thunk
     bodies for the WIDENING class.

DIRECTION: without SAL we marshal WIDENING structs bidirectionally (copy-in,
call, copy-back). That is always semantically correct: the 64-bit temp was
built from the guest's 32-bit data, so copying back is a no-op unless the
kernel wrote — exactly when a copy-back is required. `const` pointees skip
the copy-back.

Usage (classification report):
    python3 tools/wow64gen/marshalgen.py report \
        --build-dir build-arm64 --build-dir32 build-i386
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent
CLANG = REPO / ".build-tools/llvm-mingw/bin/clang"

# ---------------------------------------------------------------------------
# 1. Compile-flag extraction from compile_commands.json
# ---------------------------------------------------------------------------

# Flags worth forwarding to the -fsyntax-only layout probe.
_FLAG_RE = re.compile(r"^(-I|-D|-isystem|--target=|-f|-W|-m)")
# Flags that only matter for real codegen / would break -fsyntax-only reuse.
_FLAG_SKIP = re.compile(r"^(-fdiagnostics|-fcolor|-flto|-ffile-prefix|-fdebug)")


def extract_flags(build_dir: Path, tu_suffix: str) -> list[str]:
    """Pull include/define/target flags for a representative TU out of the
    build's compile_commands.json, rewriting canonical /tmp/vibeos paths back
    to this checkout (layout probing doesn't need the canon bind mount)."""
    db = json.loads((build_dir / "compile_commands.json").read_text())
    entry = next(e for e in db if e["file"].endswith(tu_suffix))
    flags = []
    for tok in entry["command"].split():
        if _FLAG_RE.match(tok) and not _FLAG_SKIP.match(tok):
            flags.append(tok.replace("/tmp/vibeos", str(REPO)))
    return flags


# ---------------------------------------------------------------------------
# 2. Record-layout dump + parse
# ---------------------------------------------------------------------------


@dataclass
class Field:
    offset: int  # byte offset (bitfields: byte of the storage unit)
    depth: int  # 1 = top-level member of the record
    ctype: str
    name: str  # "" for anonymous struct/union members
    is_bitfield: bool


@dataclass
class Record:
    name: str  # e.g. "struct tagWINDOWPOS" / "union _FOO"
    size: int
    align: int
    fields: list[Field] = field(default_factory=list)

    @property
    def has_union(self) -> bool:
        return self.name.startswith("union") or any(
            f.ctype.startswith("union") for f in self.fields
        )

    @property
    def has_bitfield(self) -> bool:
        return any(f.is_bitfield for f in self.fields)

    def top_fields(self) -> list[Field]:
        return [f for f in self.fields if f.depth == 1]


_REC_HDR = re.compile(r"^\s*0 \| ((?:struct|union|class) .+)$")
_REC_FOOT = re.compile(r"^\s*\| \[sizeof=(\d+), align=(\d+)")
_FIELD = re.compile(r"^\s*(\d+)(:\d+-\d+)? \| (  +)(.+)$")


def parse_layout_dump(text: str) -> dict[str, Record]:
    records: dict[str, Record] = {}
    cur: Record | None = None
    for line in text.splitlines():
        if line.startswith("*** Dumping AST Record Layout"):
            cur = None
            continue
        m = _REC_HDR.match(line)
        if m and cur is None:
            cur = Record(name=m.group(1).strip(), size=0, align=0)
            continue
        if cur is None:
            continue
        m = _REC_FOOT.match(line)
        if m:
            cur.size, cur.align = int(m.group(1)), int(m.group(2))
            # last complete definition wins (dumps can repeat)
            records[cur.name] = cur
            cur = None
            continue
        m = _FIELD.match(line)
        if m:
            offset, bits, indent, decl = (int(m.group(1)), m.group(2), m.group(3), m.group(4))
            depth = len(indent) // 2
            # decl is "TYPE name" or a nested "struct X name"; split the
            # trailing identifier off (may be absent for anonymous members).
            parts = decl.rsplit(" ", 1)
            if len(parts) == 2 and re.fullmatch(r"[A-Za-z_]\w*(\[\d+\])*", parts[1]):
                ctype, fname = parts
            else:
                ctype, fname = decl, ""
            cur.fields.append(Field(offset, depth, ctype.strip(), fname, bits is not None))
    return records


def dump_layouts(flags: list[str], tu_source: str) -> dict[str, Record]:
    with tempfile.NamedTemporaryFile("w", suffix=".c", delete=False) as f:
        f.write(tu_source)
        tu = f.name
    cmd = [str(CLANG), *flags, "-fsyntax-only", "-Xclang", "-fdump-record-layouts-complete", tu]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        errs = "\n".join(ln for ln in r.stderr.splitlines() if "error:" in ln)[:2000]
        raise RuntimeError(f"layout TU failed to compile:\n{errs}")
    return parse_layout_dump(r.stdout)


# ---------------------------------------------------------------------------
# 3. Prototype extraction (textual — headers are pre-SAL-stripped mingw)
# ---------------------------------------------------------------------------


@dataclass
class Param:
    ctype: str  # normalized, e.g. "PWINDOWPOS", "DWORD", "RECT *"
    name: str
    sal: str  # "", "in", "out", "inout" (best effort)


@dataclass
class Proto:
    name: str
    ret: str
    params: list[Param]


# SAL2 (`_In_opt_`, `_Out_writes_(n)`, `_At_(...)`) and old-style
# (`__in`, `__out_ecount(n)`) annotation tokens.
_ANN_TOK = re.compile(r"_[A-Z][A-Za-z0-9_]*_$|__[a-z][a-z0-9_]*$")
_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def strip_annotations(raw: str) -> tuple[str, str]:
    """Remove SAL/old-style annotations (with their argument lists) from one
    parameter declaration. -> (clean_decl, direction)."""
    out, i, n, sal = [], 0, len(raw), ""
    while i < n:
        m = _IDENT.match(raw, i)
        if not m:
            out.append(raw[i])
            i += 1
            continue
        tok = m.group(0)
        if _ANN_TOK.match(tok):
            t = tok.lower()
            if "inout" in t or "updates" in t:
                sal = "inout"
            elif "out" in t or "writes" in t:
                sal = sal or "out"
            elif "in" in t or "reads" in t:
                sal = sal or "in"
            i = m.end()
            # consume the annotation's balanced-paren argument list
            while i < n and raw[i].isspace():
                i += 1
            if i < n and raw[i] == "(":
                depth = 0
                while i < n:
                    if raw[i] == "(":
                        depth += 1
                    elif raw[i] == ")":
                        depth -= 1
                        if depth == 0:
                            i += 1
                            break
                    i += 1
        else:
            out.append(tok)
            i = m.end()
    return "".join(out), sal


def extract_prototypes(text: str, name_re: str) -> dict[str, Proto]:
    """Find `<ret>\\n<CONV>\\n?Name(\\n  params...);` shapes for names matching
    name_re. Tolerant of NTSYSCALLAPI/W32KAPI/APIENTRY/NTAPI decoration."""
    protos: dict[str, Proto] = {}
    pat = re.compile(r"\b(" + name_re + r")\s*\(", re.M)
    for m in pat.finditer(text):
        name = m.group(1)
        # walk to the matching ');'
        depth, i = 1, m.end()
        while i < len(text) and depth:
            if text[i] == "(":
                depth += 1
            elif text[i] == ")":
                depth -= 1
            i += 1
        if depth or i >= len(text) or text[i] != ";":
            continue  # a call site or macro, not a prototype
        arglist = re.sub(r"/\*.*?\*/", " ", text[m.end() : i - 1], flags=re.S)
        arglist = re.sub(r"//[^\n]*", " ", arglist)
        # return type: scan the few lines above for the last type-ish token
        pre = text[: m.start()].rsplit(";", 1)[-1]
        toks = [
            t
            for t in re.split(r"[\s]+", pre)
            if t
            and t
            not in (
                "NTSYSCALLAPI",
                "NTSYSAPI",
                "W32KAPI",
                "APIENTRY",
                "NTAPI",
                "WINAPI",
                "__stdcall",
                "__kernel_entry",
                "DECLSPEC_ADDRSAFE",
            )
        ]
        ret = toks[-1] if toks else "?"
        params: list[Param] = []
        if arglist.strip() not in ("", "VOID", "void"):
            for raw in _split_params(arglist):
                clean, sal = strip_annotations(raw)
                # old-style annotation macros (ntuser.h/ntgdi.h use these)
                for tok, d in (("INOUT", "inout"), ("IN", "in"), ("OUT", "out"), ("OPTIONAL", "")):
                    if re.search(rf"\b{tok}\b", clean):
                        sal = sal or d
                        clean = re.sub(rf"\b{tok}\b", " ", clean)
                clean = re.sub(r"\s+", " ", clean).strip()
                pm = re.match(r"(.+?)\s*(\**)\s*([A-Za-z_]\w*)(\[\s*\d*\s*\])?$", clean)
                if pm:
                    ctype = (pm.group(1) + (" " + pm.group(2) if pm.group(2) else "")).strip()
                    pname = pm.group(3)
                    if pm.group(4):
                        ctype += " *"
                else:
                    ctype, pname = clean, ""
                params.append(Param(ctype, pname, sal))
        protos[name] = Proto(name, ret, params)
    return protos


def _split_params(arglist: str) -> list[str]:
    out, depth, cur = [], 0, []
    for ch in arglist:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    if cur and "".join(cur).strip():
        out.append("".join(cur))
    return out


# ---------------------------------------------------------------------------
# 4. Type resolution + struct diff + classification
# ---------------------------------------------------------------------------

# scalar pointees whose width diverges 32/64 (auto-generatable: 64-bit local,
# narrow copy-back)
DIVERGENT_SCALARS = {
    "SIZE_T",
    "PSIZE_T",
    "ULONG_PTR",
    "LONG_PTR",
    "SSIZE_T",
    "INT_PTR",
    "UINT_PTR",
    "DWORD_PTR",
    "HANDLE",
    "PVOID",
    "LPVOID",
    "WPARAM",
    "LPARAM",
    "LRESULT",
    "KAFFINITY",
    "HWND",
    "HMENU",
    "HICON",
    "HCURSOR",
    "HBRUSH",
    "HDC",
    "HBITMAP",
    "HRGN",
    "HFONT",
    "HPALETTE",
    "HMODULE",
    "HINSTANCE",
    "HDESK",
    "HWINSTA",
    "HHOOK",
    "HKL",
    "HMONITOR",
    "HRAWINPUT",
    "HIMC",
}
# scalar pointees that are width-identical -> flat thunk already correct
SAFE_SCALARS = {
    "VOID",
    "CHAR",
    "UCHAR",
    "BOOLEAN",
    "BYTE",
    "WORD",
    "USHORT",
    "SHORT",
    "WCHAR",
    "INT",
    "UINT",
    "LONG",
    "ULONG",
    "DWORD",
    "BOOL",
    "FLOAT",
    "LONGLONG",
    "ULONGLONG",
    "LARGE_INTEGER",
    "ULARGE_INTEGER",
    "DWORD64",
    "UINT64",
    "INT64",
    "NTSTATUS",
    "ACCESS_MASK",
    "COLORREF",
    "LCID",
    "LANGID",
    "ATOM",
    "HRESULT",
    # C builtins (Windows LLP64: long is 4 bytes on both widths)
    "int",
    "unsigned",
    "long",
    "short",
    "char",
    "void",
    "float",
    "double",
    "unsigned int",
    "unsigned long",
    "unsigned short",
    "unsigned char",
    "signed char",
    "long long",
    "unsigned long long",
    "wchar_t",
    "__int64",
}


def _norm_type(ctype: str) -> str:
    t = re.sub(r"/\*.*?\*/", " ", ctype)
    toks = [
        k
        for k in t.split()
        if k
        not in ("IN", "OUT", "OPTIONAL", "CONST", "const", "FAR", "NEAR", "volatile", "__unaligned")
    ]
    return " ".join(toks).strip()


TAG_ALIASES = {
    "DEVMODEW": "struct _devicemodeW",
    "DEVMODEA": "struct _devicemodeA",
    "SECURITY_QUALITY_OF_SERVICE": "struct _SECURITY_QUALITY_OF_SERVICE",
}


def resolve_exact(ctype: str, records: dict[str, Record]) -> Record | None:
    """Resolve a NON-pointer type name to a dumped record (no P/LP peeling)."""
    t = re.sub(r"\s*(\*+|\[\d*\])\s*$", "", _norm_type(ctype)).strip()
    if t.startswith(("struct ", "union ", "class ")):
        return records.get(t)
    if t in TAG_ALIASES:
        return records.get(TAG_ALIASES[t])
    for kind in ("struct", "union"):
        for tag in (f"{kind} _{t}", f"{kind} tag{t}", f"{kind} _tag{t}", f"{kind} {t}"):
            if tag in records:
                return records[tag]
    return None


def resolve_pointee(ctype: str, records: dict[str, Record]) -> Record | None:
    """Resolve a pointer type's pointee record: `FOO *` or a P/LP typedef."""
    t = _norm_type(ctype)
    if "*" in t:
        return resolve_exact(t, records)
    for prefix in ("LPC", "PC", "LP", "P"):
        if t.startswith(prefix):
            r = resolve_exact(t[len(prefix) :], records)
            if r:
                return r
    return None


@dataclass
class StructVerdict:
    cls: str  # IDENTICAL | WIDENING | SEMANTIC
    detail: str
    rec32: Record | None = None
    rec64: Record | None = None


_STRING_PTEES = {"STR", "CSTR", "WSTR", "CWSTR", "TSTR", "CTSTR"}


def _scalar_base(t: str) -> str:
    """Peel P/LP from a scalar-pointer typedef name: PULONG -> ULONG."""
    for prefix in ("LPC", "PC", "LP", "P"):
        if t.startswith(prefix):
            return t[len(prefix) :]
    return t


def _field_class(ctype: str, db32, db64, _seen: set[str]) -> tuple[str, str]:
    """Classify one struct FIELD by its type (not by offset arithmetic).
    -> (SAME | WIDEN | SEMANTIC, detail)."""
    t = re.sub(r"\[\d*\]$", "", _norm_type(ctype)).strip()  # arrays: element
    # nested record?
    r = resolve_exact(t, db32)
    if r:
        sub = diff_struct(t, db32, db64, _seen)
        if sub.cls == "IDENTICAL":
            return ("SAME", "")
        if sub.cls == "WIDENING":
            # nested divergent struct: only generatable when embedded flat
            # (codegen recurses); arrays of divergent elements: v1
            if ctype.rstrip().endswith("]"):
                return ("SEMANTIC", f"array of divergent {t}")
            return ("WIDEN", f"nested {t}")
        return ("SEMANTIC", f"nested {t}: {sub.detail}")
    if "*" in t:
        # pointer field: 4->8 widening; pointee holds a guest VA (valid host
        # VA) — but if the POINTEE layout diverges, the kernel would read it
        # with 64-bit eyes -> must be marshalled too (flag for codegen).
        pr = resolve_pointee(t, db32)
        if pr:
            sub = diff_struct(pr.name, db32, db64, _seen)
            if sub.cls == "SEMANTIC":
                return ("SEMANTIC", f"ptr to divergent {pr.name}: {sub.detail}")
            if sub.cls == "WIDENING":
                return ("WIDEN", f"ptr to divergent {pr.name} (deep)")
        return ("WIDEN", "")
    base = t
    if base in SAFE_SCALARS or base in (
        "float",
        "double",
        "int",
        "unsigned",
        "long",
        "char",
        "short",
        "unsigned int",
        "unsigned long",
        "unsigned short",
        "unsigned char",
    ):
        return ("SAME", "")
    if base in DIVERGENT_SCALARS:
        return ("WIDEN", "")
    sb = _scalar_base(base)
    if sb in SAFE_SCALARS or sb in _STRING_PTEES:
        return ("WIDEN", "")  # pointer typedef to width-identical data
    if sb in DIVERGENT_SCALARS:
        return ("WIDEN", f"ptr to {sb} (deep scalar)")
    pr = resolve_pointee(base, db32)
    if pr:
        sub = diff_struct(pr.name, db32, db64, _seen)
        if sub.cls == "SEMANTIC":
            return ("SEMANTIC", f"ptr to divergent {pr.name}: {sub.detail}")
        if sub.cls == "WIDENING":
            return ("WIDEN", f"ptr to divergent {pr.name} (deep)")
        return ("WIDEN", "")
    return ("SEMANTIC", f"unknown field type '{t}'")


def diff_struct(
    name: str, db32: dict[str, Record], db64: dict[str, Record], _seen: set[str] | None = None
) -> StructVerdict:
    _seen = _seen if _seen is not None else set()
    r32 = resolve_exact(name, db32)
    r64 = resolve_exact(name, db64)
    if not r32 or not r64:
        return StructVerdict("SEMANTIC", f"layout of '{name}' not in dump (add header to TU?)")
    if r32.name in _seen:
        return StructVerdict("IDENTICAL", "recursion cut")
    _seen.add(r32.name)

    f32, f64 = r32.top_fields(), r64.top_fields()
    same_layout = r32.size == r64.size and [
        (-1 if f.is_bitfield else f.offset, f.name) for f in f32
    ] == [(-1 if f.is_bitfield else f.offset, f.name) for f in f64]
    if [f.name for f in f32] != [f.name for f in f64]:
        return StructVerdict("SEMANTIC", "field lists differ across widths", r32, r64)
    if r32.has_bitfield or r64.has_bitfield or r32.has_union or r64.has_union:
        if same_layout:
            return StructVerdict("IDENTICAL", "same layout (union/bitfield)", r32, r64)
        return StructVerdict("SEMANTIC", "union/bitfield with divergent layout", r32, r64)

    def span(fields: list[Field], i: int, total: int) -> tuple[int, int]:
        nxt = fields[i + 1].offset if i + 1 < len(fields) else total
        return (fields[i].offset, nxt - fields[i].offset)

    if same_layout:
        # width-identical. A field with identical byte-span across widths
        # cannot be a pointer (4 vs 8 always differ), so no divergent pointee
        # can hide behind an unknown typedef — IDENTICAL, no type knowledge
        # needed. (Guest VAs are valid host VAs for any same-width pointers
        # smuggled in DWORDs; those are the caller's problem, not layout's.)
        return StructVerdict("IDENTICAL", "", r32, r64)

    # Layouts diverge. Hybrid per-field check: identical-span fields are
    # fine as-is (can't be pointers); divergent-span fields must be a known
    # widening shape (4->8 scalar/pointer, or a nested WIDENING struct).
    for i, (a, _b) in enumerate(zip(f32, f64, strict=False)):
        (_o32, s32), (_o64, s64) = span(f32, i, r32.size), span(f64, i, r64.size)
        if s32 == s64:
            continue  # same span: not a pointer; done
        if (s32, s64) == (4, 8):
            # some pointer/handle/uintptr — widens by sign-extension. If the
            # POINTEE is itself a divergent struct the marshaller must
            # recurse; check when the type resolves, trust the span if not.
            pr = resolve_pointee(a.ctype, db32)
            if pr:
                sub = diff_struct(pr.name, db32, db64, _seen)
                if sub.cls == "SEMANTIC":
                    return StructVerdict(
                        "SEMANTIC",
                        f"field {a.name}: ptr to divergent {pr.name} ({sub.detail})",
                        r32,
                        r64,
                    )
            continue
        # other divergence: nested struct or array — classify by type
        cls, det = _field_class(a.ctype, db32, db64, _seen)
        if cls == "SEMANTIC":
            return StructVerdict("SEMANTIC", f"field {a.name} ({s32}->{s64}): {det}", r32, r64)
    return StructVerdict("WIDENING", "", r32, r64)


def classify_param(p: Param, db32, db64) -> tuple[str, str]:
    """-> (class, detail) for one parameter."""
    t = _norm_type(p.ctype)
    # 1) by-value record (POINT point, LARGE_STRING str, ...)
    r = resolve_exact(t, db32)
    if r and "*" not in t:
        v = diff_struct(t, db32, db64)
        if v.cls == "IDENTICAL":
            return ("FLAT", f"by-value {t}, width-identical")
        return (v.cls, f"by-value {t}: {v.detail}")
    # 2) plain scalar (incl. HANDLE/SIZE_T — the flat thunk sign-extends)
    if "*" not in t and t in SAFE_SCALARS | DIVERGENT_SCALARS:
        return ("FLAT", "scalar")
    # 3) pointer (either `FOO *` or P/LP typedef)
    pointee = None
    if "*" in t:
        base = re.sub(r"\s*\*+$", "", t).strip()
        if base in SAFE_SCALARS or base in (
            "void",
            "char",
            "wchar_t",
            "int",
            "unsigned",
            "long",
            "short",
        ):
            return ("FLAT", f"-> {base}: width-identical pointee")
        if base in DIVERGENT_SCALARS:
            return ("SCALAR_WIDEN", f"-> {base}: 4->8 scalar pointee")
        pointee = base
    else:
        sb = _scalar_base(t)
        if sb != t:
            if sb in SAFE_SCALARS or sb in _STRING_PTEES:
                return ("FLAT", f"-> {sb}: width-identical pointee")
            if sb in DIVERGENT_SCALARS:
                return ("SCALAR_WIDEN", f"-> {sb}: 4->8 scalar pointee")
            pointee = sb
    if pointee is None and "*" not in t:
        return ("SEMANTIC", f"unknown type '{t}'")
    pr = resolve_pointee(t, db32)
    if pr:
        v = diff_struct(pr.name, db32, db64)
        return (v.cls, f"-> {pr.name}: {v.detail}" if v.detail else f"-> {pr.name}")
    return ("SEMANTIC", f"-> {pointee}: layout not in dump (add header to TU?)")


PARAM_RANK = {"FLAT": 0, "IDENTICAL": 0, "SCALAR_WIDEN": 1, "WIDENING": 2, "SEMANTIC": 3}


# ---------------------------------------------------------------------------
# 5. Report driver
# ---------------------------------------------------------------------------

# (tu_suffix used to look up flags in compile_commands.json, TU source)
WIN32K_TUS = [
    ("user32_stubs.c", "#include <user32.h>\n"),
    ("gdi32/wine/emfdc.c", "#include <precomp.h>\n"),
]
# ndk umbrella for the ntdll surface
NTDLL_TU = (
    "#include <ntdef.h>\n"
    "#define WIN32_NO_STATUS\n"
    "#include <windef.h>\n"
    "#include <winbase.h>\n"
    "#include <ndk/ntndk.h>\n"
)


def load_needs_manual(gen_c: Path) -> list[str]:
    txt = gen_c.read_text()
    return re.findall(r"/\* (\w+): WOW64_NEEDS_MANUAL", txt)


# ---------------------------------------------------------------------------
# 5b. Codegen — flat thunks for FLAT-OK, field-copy marshallers for WIDENING
# ---------------------------------------------------------------------------

_8BYTE_SCALARS = {
    "LONGLONG",
    "ULONGLONG",
    "DWORD64",
    "UINT64",
    "INT64",
    "LARGE_INTEGER",
    "ULARGE_INTEGER",
    "double",
    "__int64",
    "long long",
    "unsigned long long",
}


def _param_slots32(p: Param, db32) -> int:
    """Guest (i686 stdcall) stack dwords this parameter consumes."""
    t = _norm_type(p.ctype)
    if "*" in t:
        return 1
    r = resolve_exact(t, db32)
    if r:
        return max(1, (r.size + 3) // 4)
    if t in _8BYTE_SCALARS:
        return 2
    return 1


def _ctag(rec: Record) -> str:
    """struct tagMSG -> tagMSG (identifier-safe)."""
    return rec.name.split(None, 1)[1]


_SCALAR_SIZES = {
    **dict.fromkeys(
        ("CHAR", "UCHAR", "BOOLEAN", "BYTE", "char", "unsigned char", "signed char"), 1
    ),
    **dict.fromkeys(
        (
            "WORD",
            "USHORT",
            "SHORT",
            "WCHAR",
            "wchar_t",
            "ATOM",
            "LANGID",
            "short",
            "unsigned short",
        ),
        2,
    ),
    **dict.fromkeys(
        (
            "INT",
            "UINT",
            "LONG",
            "ULONG",
            "DWORD",
            "BOOL",
            "FLOAT",
            "NTSTATUS",
            "ACCESS_MASK",
            "COLORREF",
            "LCID",
            "HRESULT",
            "int",
            "unsigned",
            "unsigned int",
            "long",
            "unsigned long",
            "float",
        ),
        4,
    ),
    **dict.fromkeys(
        (
            "LONGLONG",
            "ULONGLONG",
            "DWORD64",
            "UINT64",
            "INT64",
            "LARGE_INTEGER",
            "ULARGE_INTEGER",
            "double",
            "__int64",
            "long long",
            "unsigned long long",
        ),
        8,
    ),
}


def _esize(ctype: str, db: dict[str, Record], ptr_width: int) -> int | None:
    """True size of a field's type at one width, or None if unknown."""
    t = _norm_type(ctype)
    am = re.search(r"\[(\d+)\]$", t)
    if am:
        elem = _esize(re.sub(r"\[\d+\]$", "", t).strip(), db, ptr_width)
        return elem * int(am.group(1)) if elem else None
    if "*" in t:
        return ptr_width
    r = resolve_exact(t, db)
    if r:
        return r.size
    if t in _SCALAR_SIZES:
        return _SCALAR_SIZES[t]
    if t in DIVERGENT_SCALARS:
        return ptr_width
    sb = _scalar_base(t)
    if sb != t and (
        sb in _SCALAR_SIZES
        or sb in _STRING_PTEES
        or sb in SAFE_SCALARS
        or sb in DIVERGENT_SCALARS
        or resolve_exact(sb, db)
    ):
        return ptr_width  # P/LP typedef pointer
    return None


def _gen_struct_marshaller(
    rec32: Record, rec64: Record, db32, db64, emitted: dict[str, tuple[int, int]], out: list[str]
) -> bool:
    """Emit In/Out field-copy pair for one WIDENING struct (recursing into
    nested divergent structs). Returns False if an unmarshallable field shape
    is hit (deep pointee, unknown divergence)."""
    tag = _ctag(rec32)
    if tag in emitted:
        return True
    emitted[tag] = (rec32.size, rec64.size)  # reserve (cycle guard)

    f32, f64 = rec32.top_fields(), rec64.top_fields()

    def span(fields, i, total):
        nxt = fields[i + 1].offset if i + 1 < len(fields) else total
        return fields[i].offset, nxt - fields[i].offset

    body_in, body_out = [], []
    for i, (a, b) in enumerate(zip(f32, f64, strict=False)):
        (o32, s32) = span(f32, i, rec32.size)
        (o64, s64) = span(f64, i, rec64.size)
        # prefer the field's TRUE type size over the span (spans absorb
        # alignment padding: UNICODE_STRING.MaximumLength spans 2 vs 6)
        e32 = _esize(a.ctype, db32, 4)
        e64 = _esize(b.ctype, db64, 8)
        if e32 is not None and e64 is not None:
            s32, s64 = e32, e64
        cmt = f" /* {a.name} */"
        if s32 == s64:
            body_in.append(f"    WowAmCpy(d + {o64}, s + {o32}, {s32});{cmt}")
            body_out.append(f"    WowAmCpy(d + {o32}, s + {o64}, {s32});{cmt}")
            continue
        if (s32, s64) == (4, 8):
            pr = resolve_pointee(a.ctype, db32)
            if pr:
                sub = diff_struct(pr.name, db32, db64)
                if sub.cls == "WIDENING":
                    del emitted[tag]
                    return False  # deep pointee: not generatable (v0)
                if sub.cls == "SEMANTIC":
                    del emitted[tag]
                    return False
            body_in.append(f"    WowAmSx(d + {o64}, s + {o32});{cmt}")
            body_out.append(f"    WowAmNx(d + {o32}, s + {o64});{cmt}")
            continue
        # nested divergent struct?
        nr32 = resolve_exact(a.ctype, db32)
        nr64 = resolve_exact(a.ctype, db64)
        if nr32 and nr64 and (nr32.size, nr64.size) == (s32, s64):
            if not _gen_struct_marshaller(nr32, nr64, db32, db64, emitted, out):
                del emitted[tag]
                return False
            ntag = _ctag(nr32)
            body_in.append(f"    WowAmIn_{ntag}(s + {o32}, d + {o64});{cmt}")
            body_out.append(f"    WowAmOut_{ntag}(s + {o64}, d + {o32});{cmt}")
            continue
        del emitted[tag]
        return False
    out.append(
        f"/* {rec32.name}: {rec32.size} -> {rec64.size} bytes */\n"
        f"static void WowAmIn_{tag}(const unsigned char *s, unsigned char *d)\n"
        "{\n"
        + f"    WowAmZero(d, {rec64.size});\n"
        + "\n".join(body_in)
        + "\n}\n\n"
        + f"static void WowAmOut_{tag}(const unsigned char *s, unsigned char *d)\n"
        "{\n" + "\n".join(body_out) + "\n}\n"
    )
    return True


_GEN_PRELUDE = """\
/*
 * wow64_automarshal.c — GENERATED by tools/wow64gen/marshalgen.py. DO NOT EDIT.
 *
 * 32->64 syscall thunks that wow64gen (stage 1) could not derive from the
 * .spec alone: flat thunks recovered from real header prototypes, and
 * field-copy deep marshallers generated from clang record-layout diffs
 * (i686 vs aarch64) through the real header stack.
 *
 * Regenerate:  python3 tools/wow64gen/marshalgen.py gen
 * (requires configured build-arm64/ + build-i386/ for compile_commands.json)
 */

#include <stdarg.h>
#define WIN32_NO_STATUS
#include <windef.h>
#include <winnt.h>
#include <ndk/ntndk.h>

#define WOW64_MANUAL_UNIMPL ((NTSTATUS)0xC0BBBBBB)

static void WowAmCpy(unsigned char *d, const unsigned char *s, unsigned n)
{
    while (n--) *d++ = *s++;
}

static void WowAmZero(unsigned char *d, unsigned n)
{
    while (n--) *d++ = 0;
}

/* 4 -> 8 sign-extend (covers pointers/handles: guest VAs < 2GB, and
 * pseudo-handles (-1, -2) which are only correct sign-extended) */
static void WowAmSx(unsigned char *d, const unsigned char *s)
{
    LONG v;
    LONGLONG w;
    WowAmCpy((unsigned char *)&v, s, 4);
    w = (LONGLONG)v;
    WowAmCpy(d, (const unsigned char *)&w, 8);
}

/* 8 -> 4 narrow */
static void WowAmNx(unsigned char *d, const unsigned char *s)
{
    WowAmCpy(d, s, 4);
}

static int WowAmStrEq(const char *a, const char *b)
{
    while (*a && *a == *b) { a++; b++; }
    return *a == *b;
}

"""


def _flat_arg_exprs(proto: Proto, db32) -> tuple[list[str], int] | None:
    """Build the 64-bit call argument expressions for a flat/widening thunk.
    Returns (exprs, total_slots) or None if a param shape defeats flat
    passing (by-value struct > 16 bytes)."""
    exprs, slot = [], 0
    for p in proto.params:
        n = _param_slots32(p, db32)
        if n == 1:
            exprs.append(f"(ULONGLONG)(LONGLONG)(LONG)a[{slot}]")
        elif n == 2:
            exprs.append(f"((ULONGLONG)a[{slot}] | ((ULONGLONG)a[{slot + 1}] << 32))")
        elif n <= 4:
            # by-value 9..16-byte struct: two AAPCS64 register halves
            exprs.append(f"((ULONGLONG)a[{slot}] | ((ULONGLONG)a[{slot + 1}] << 32))")
            exprs.append(
                f"((ULONGLONG)a[{slot + 2}] | ((ULONGLONG)a[{slot + 3}] << 32))"
                if n == 4
                else f"(ULONGLONG)a[{slot + 2}]"
            )
        else:
            return None
        slot += n
    return exprs, slot


# ---------------------------------------------------------------------------
# 5c. Callback-payload codegen (the KeUserModeCallback WINDOWPROC direction)
# ---------------------------------------------------------------------------
#
# win32k packs certain window messages' lParam payload into the callback
# argument blob (g_MsgMemory in win32k message.rs is the authoritative list).
# wow64 must convert that payload 64->32 on the way DOWN to the guest and
# (for MMS_FLAG_READWRITE messages) 32->64 back UP after the guest wndproc
# returns.
#
# Direction rules differ from the syscall marshallers:
#   DOWN (64->32): every divergent field converts; embedded DEREFERENCEABLE
#     pointers (lppos, lpData) just narrow — guest user32 recomputes them
#     from the appended tail (message.c MsgiKMToUMMessage fixups), so the
#     value is scratch.
#   UP (32->64): writes into the kernel's ORIGINAL packed buffer, so
#     dereferenceable pointer fields must be PRESERVED (a sign-extended
#     guest VA would corrupt the kernel's packed lppos and redirect
#     UnpackParam's copy source). HANDLES and uintptr VALUE fields
#     (hmenuNext, itemData, dwContextId) DO copy back — returning them is
#     the message's purpose (e.g. WM_NEXTMENU).
#
# Messages with a simple one-struct payload are table-driven; messages with
# appended tails (NCCALCSIZE_PARAMS+WINDOWPOS, COPYDATASTRUCT+data,
# CREATESTRUCT+strings) keep their layout logic in wow64_callback.c, calling
# the exported per-struct pairs. CREATESTRUCT keeps its hand-written repack
# (offset-encoded strings).

# (WM name, value, struct) — payload is exactly one struct; auto-dispatched.
CB_TABLE_MESSAGES = [
    ("WM_DRAWITEM", 0x002B, "DRAWITEMSTRUCT"),
    ("WM_MEASUREITEM", 0x002C, "MEASUREITEMSTRUCT"),
    ("WM_HELP", 0x0053, "HELPINFO"),
    ("WM_NEXTMENU", 0x0213, "MDINEXTMENU"),
]
# extra per-struct pairs exported for the hand-coded special layouts
CB_EXTRA_STRUCTS = ["NCCALCSIZE_PARAMS", "WINDOWPOS", "COPYDATASTRUCT"]

_CB_PRELUDE = """\
/*
 * wow64_cb_automarshal.c — GENERATED by tools/wow64gen/marshalgen.py. DO NOT EDIT.
 *
 * 64<->32 marshallers for win32k WINDOWPROC callback payloads (the
 * KeUserModeCallback direction). Field offsets come from clang record-layout
 * dumps at both -target i686-w64-mingw32 and aarch64-w64-mingw32.
 * See wow64_callback.c for how Down/Up are dispatched around the guest call.
 *
 * Regenerate:  python3 tools/wow64gen/marshalgen.py gen-callback
 */

typedef unsigned int ULONG;
typedef int LONG;
typedef long long LONGLONG;

static void WowCbCpy(unsigned char *d, const unsigned char *s, unsigned n)
{
    while (n--) *d++ = *s++;
}

static void WowCbZero(unsigned char *d, unsigned n)
{
    while (n--) *d++ = 0;
}

/* 4 -> 8 sign-extend (handles: pseudo/special values like HWND_TOPMOST(-1)
 * are only correct sign-extended) */
static void WowCbSx(unsigned char *d, const unsigned char *s)
{
    LONG v;
    LONGLONG w;
    WowCbCpy((unsigned char *)&v, s, 4);
    w = (LONGLONG)v;
    WowCbCpy(d, (const unsigned char *)&w, 8);
}

/* 8 -> 4 narrow */
static void WowCbNx(unsigned char *d, const unsigned char *s)
{
    WowCbCpy(d, s, 4);
}

"""


def _is_deref_pointer(ctype: str, db32) -> bool:
    """A field whose value the KERNEL may dereference after copy-back (real
    memory pointer), as opposed to a handle/uintptr VALUE field."""
    t = re.sub(r"\[\d*\]$", "", _norm_type(ctype)).strip()
    if t in DIVERGENT_SCALARS:  # HWND/HANDLE/ULONG_PTR/... value fields
        return False
    if "*" in t:
        return True
    sb = _scalar_base(t)
    # P/LP typedef -> pointer
    return bool(
        sb != t
        and (
            sb in SAFE_SCALARS
            or sb in _STRING_PTEES
            or sb in DIVERGENT_SCALARS
            or resolve_exact(sb, db32)
        )
    )


def _gen_cb_pair(
    rec32: Record, rec64: Record, db32, db64, emitted: dict[str, tuple[int, int]], out: list[str]
) -> bool:
    """Emit WowCbDown_<tag> (64->32) / WowCbUp_<tag> (32->64, pointer fields
    preserved) for one struct. Nested divergent structs recurse."""
    tag = _ctag(rec32)
    if tag in emitted:
        return True
    emitted[tag] = (rec32.size, rec64.size)

    f32, f64 = rec32.top_fields(), rec64.top_fields()

    def span(fields, i, total):
        nxt = fields[i + 1].offset if i + 1 < len(fields) else total
        return fields[i].offset, nxt - fields[i].offset

    body_down, body_up = [], []
    for i, (a, b) in enumerate(zip(f32, f64, strict=False)):
        (o32, s32) = span(f32, i, rec32.size)
        (o64, s64) = span(f64, i, rec64.size)
        e32 = _esize(a.ctype, db32, 4)
        e64 = _esize(b.ctype, db64, 8)
        if e32 is not None and e64 is not None:
            s32, s64 = e32, e64
        cmt = f" /* {a.name} */"
        if s32 == s64:
            body_down.append(f"    WowCbCpy(d + {o32}, s + {o64}, {s32});{cmt}")
            body_up.append(f"    WowCbCpy(d + {o64}, s + {o32}, {s32});{cmt}")
            continue
        if (s32, s64) == (4, 8):
            body_down.append(f"    WowCbNx(d + {o32}, s + {o64});{cmt}")
            if _is_deref_pointer(a.ctype, db32):
                body_up.append(
                    f"    /* {a.name}: kernel pointer preserved (guest value is scratch) */"
                )
            else:
                body_up.append(f"    WowCbSx(d + {o64}, s + {o32});{cmt}")
            continue
        nr32 = resolve_exact(a.ctype, db32)
        nr64 = resolve_exact(b.ctype, db64)
        if nr32 and nr64:
            if not _gen_cb_pair(nr32, nr64, db32, db64, emitted, out):
                del emitted[tag]
                return False
            ntag = _ctag(nr32)
            body_down.append(f"    WowCbDown_{ntag}(s + {o64}, d + {o32});{cmt}")
            body_up.append(f"    WowCbUp_{ntag}(s + {o32}, d + {o64});{cmt}")
            continue
        del emitted[tag]
        return False
    out.append(
        f"/* {rec32.name}: 32-bit {rec32.size}b <-> 64-bit {rec64.size}b */\n"
        f"void WowCbDown_{tag}(const unsigned char *s, unsigned char *d)\n"
        "{\n"
        + f"    WowCbZero(d, {rec32.size});\n"
        + "\n".join(body_down)
        + "\n}\n\n"
        + f"void WowCbUp_{tag}(const unsigned char *s, unsigned char *d)\n"
        "{\n" + "\n".join(body_up) + "\n}\n\n"
    )
    return True


def cmd_gen_callback(args) -> int:
    b64, b32 = Path(args.build_dir), Path(args.build_dir32)
    db32: dict[str, Record] = {}
    db64: dict[str, Record] = {}
    for tu_suffix, tu_src in WIN32K_TUS:
        for db, bdir in ((db32, b32), (db64, b64)):
            for k, v in dump_layouts(extract_flags(bdir, tu_suffix), tu_src).items():
                db.setdefault(k, v)

    emitted: dict[str, tuple[int, int]] = {}
    pairs_out: list[str] = []
    table_rows: list[str] = []
    failed: list[str] = []

    want = [(nm, val, st) for nm, val, st in CB_TABLE_MESSAGES]
    for nm, val, st in want:
        r32, r64 = resolve_exact(st, db32), resolve_exact(st, db64)
        if not r32 or not r64:
            failed.append(f"{st}: not in layout dump")
            continue
        v = diff_struct(r32.name, db32, db64)
        if v.cls == "SEMANTIC":
            failed.append(f"{st}: SEMANTIC ({v.detail})")
            continue
        if not _gen_cb_pair(r32, r64, db32, db64, emitted, pairs_out):
            failed.append(f"{st}: pair not generatable")
            continue
        tag = _ctag(r32)
        table_rows.append(
            f"    {{ 0x{val:04X} /* {nm} */, {r32.size}, {r64.size}, "
            f"WowCbDown_{tag}, WowCbUp_{tag} }},"
        )
    for st in CB_EXTRA_STRUCTS:
        r32, r64 = resolve_exact(st, db32), resolve_exact(st, db64)
        if not r32 or not r64:
            failed.append(f"{st}: not in layout dump")
            continue
        if not _gen_cb_pair(r32, r64, db32, db64, emitted, pairs_out):
            failed.append(f"{st}: pair not generatable")
    if failed:
        raise RuntimeError("gen-callback failed on:\n  " + "\n  ".join(failed))

    hdr_decls = "\n".join(
        f"void WowCbDown_{t}(const unsigned char *s, unsigned char *d);\n"
        f"void WowCbUp_{t}(const unsigned char *s, unsigned char *d);\n"
        f"#define WOW64_CB_SIZE32_{t} {sz[0]}\n"
        f"#define WOW64_CB_SIZE64_{t} {sz[1]}"
        for t, sz in emitted.items()
    )
    header = (
        "/* wow64_cb_automarshal.h — GENERATED by marshalgen. DO NOT EDIT. */\n"
        "#ifndef WOW64_CB_AUTOMARSHAL_H\n#define WOW64_CB_AUTOMARSHAL_H\n\n"
        "typedef struct _WOW64_CB_MARSHAL {\n"
        "    unsigned int Msg;\n"
        "    unsigned int Size32;\n"
        "    unsigned int Size64;\n"
        "    void (*Down)(const unsigned char *, unsigned char *);\n"
        "    void (*Up)(const unsigned char *, unsigned char *);\n"
        "} WOW64_CB_MARSHAL;\n\n"
        "const WOW64_CB_MARSHAL *Wow64CbFindMarshal(unsigned int Msg);\n\n"
        + hdr_decls
        + "\n\n#endif /* WOW64_CB_AUTOMARSHAL_H */\n"
    )

    body = (
        _CB_PRELUDE + "".join(pairs_out) + "typedef struct _WOW64_CB_MARSHAL {\n"
        "    unsigned int Msg;\n"
        "    unsigned int Size32;\n"
        "    unsigned int Size64;\n"
        "    void (*Down)(const unsigned char *, unsigned char *);\n"
        "    void (*Up)(const unsigned char *, unsigned char *);\n"
        "} WOW64_CB_MARSHAL;\n\n"
        "static const WOW64_CB_MARSHAL Wow64CbMarshalTable[] = {\n"
        + "\n".join(table_rows)
        + "\n};\n\n"
        "const WOW64_CB_MARSHAL *Wow64CbFindMarshal(unsigned int Msg)\n"
        "{\n"
        "    unsigned int i;\n"
        "    for (i = 0; i < sizeof(Wow64CbMarshalTable) /\n"
        "                    sizeof(Wow64CbMarshalTable[0]); i++)\n"
        "        if (Wow64CbMarshalTable[i].Msg == Msg)\n"
        "            return &Wow64CbMarshalTable[i];\n"
        "    return 0;\n"
        "}\n"
    )
    (REPO / args.out).write_text(body)
    (REPO / args.out_header).write_text(header)
    print(
        f"wrote {args.out} + {args.out_header}: "
        f"{len(table_rows)} table messages, {len(emitted)} struct pairs",
        file=sys.stderr,
    )
    return 0


def cmd_gen(args) -> int:
    b64, b32 = Path(args.build_dir), Path(args.build_dir32)
    db32: dict[str, Record] = {}
    db64: dict[str, Record] = {}
    for tu_suffix, tu_src in WIN32K_TUS:
        for db, bdir in ((db32, b32), (db64, b64)):
            for k, v in dump_layouts(extract_flags(bdir, tu_suffix), tu_src).items():
                db.setdefault(k, v)
    proto_text = "\n".join((REPO / h).read_text() for h in args.headers.split(","))
    protos = extract_prototypes(proto_text, r"Nt\w+")
    manual = load_needs_manual(Path(args.gen_c))
    linkable = set(
        re.findall(r"(?m)^@ stdcall (?:-arch=\S+ )?(\w+)\(", (REPO / args.export_spec).read_text())
    )
    hand_written = {
        ln.split()[0]
        for ln in (REPO / args.manual_list).read_text().splitlines()
        if ln.strip() and not ln.startswith("#")
    }

    emitted_structs: dict[str, tuple[int, int]] = {}
    struct_out: list[str] = []
    thunks: list[str] = []
    dispatch: list[str] = []
    skipped: list[str] = []

    for name in manual:
        if name in hand_written:
            continue
        p = protos.get(name)
        if not p:
            skipped.append(f"{name}: no prototype")
            continue
        if name not in linkable:
            skipped.append(f"{name}: not exported by {args.export_spec}")
            continue

        # classify each param
        plan = []  # (param, class, rec32, rec64)
        ok = True
        for prm in p.params:
            cls, _det = classify_param(prm, db32, db64)
            if cls in ("FLAT", "IDENTICAL"):
                plan.append((prm, "FLAT", None, None))
            elif cls == "WIDENING":
                pr32 = resolve_pointee(prm.ctype, db32) or resolve_exact(prm.ctype, db32)
                pr64 = resolve_pointee(prm.ctype, db64) or resolve_exact(prm.ctype, db64)
                if not pr32 or not pr64:
                    ok = False
                    skipped.append(f"{name}: {prm.name} pointee unresolved")
                    break
                if not _gen_struct_marshaller(pr32, pr64, db32, db64, emitted_structs, struct_out):
                    ok = False
                    skipped.append(f"{name}: {_ctag(pr32)} not generatable (deep pointee)")
                    break
                plan.append((prm, "WIDEN", pr32, pr64))
            else:
                ok = False
                skipped.append(f"{name}: param {prm.name} is {cls}")
                break
        if not ok:
            continue

        flat = _flat_arg_exprs(p, db32)
        if flat is None:
            skipped.append(f"{name}: by-value struct > 16 bytes")
            continue
        exprs, _slots = flat

        # patch WIDEN params: local 64-bit buffer + in/out marshal
        decls, pre, post = [], [], []
        slot = 0
        for i, (prm, kind, pr32, pr64) in enumerate(plan):
            n = _param_slots32(prm, db32)
            if kind == "WIDEN":
                tag = _ctag(pr32)
                is_const = "CONST" in prm.ctype or "const" in prm.ctype or prm.sal == "in"
                decls.append(f"    unsigned char b{i}[{pr64.size}] __attribute__((aligned(16)));")
                decls.append(
                    f"    const unsigned char *g{i} = (const unsigned char *)(ULONG_PTR)a[{slot}];"
                )
                pre.append(f"    if (g{i}) WowAmIn_{tag}(g{i}, b{i});")
                if not is_const:
                    post.append(
                        f"    if (g{i}) WowAmOut_{tag}(b{i}, "
                        f"(unsigned char *)(ULONG_PTR)a[{slot}]);"
                    )
                exprs[i] = f"(g{i} ? (ULONGLONG)(ULONG_PTR)b{i} : 0)"
            slot += n
        nargs = len(exprs)
        proto64 = ", ".join(["ULONGLONG"] * nargs) if nargs else "void"
        call = f"{name}({', '.join(exprs)})"
        thunks.append(
            f"static NTSTATUS WowAmThunk_{name}(ULONG *a)\n"
            "{\n"
            f"    extern ULONGLONG {name}({proto64});\n"
            + ("\n".join(decls) + "\n" if decls else "")
            + "    ULONGLONG r;\n"
            "    (void)a;\n"
            + ("\n".join(pre) + "\n" if pre else "")
            + f"    r = {call};\n"
            + ("\n".join(post) + "\n" if post else "")
            + "    return (NTSTATUS)(ULONG)r;\n"
            "}\n"
        )
        dispatch.append(
            f'    if (WowAmStrEq(Name, "{name}"))\n        return WowAmThunk_{name}(Arguments);'
        )

    out_path = REPO / args.out
    parts = [_GEN_PRELUDE]
    parts.extend(struct_out)
    parts.append("\n")
    parts.extend(thunks)
    parts.append(
        "\nNTSTATUS Wow64AutoMarshalDispatch(const char *Name, ULONG *Arguments)\n"
        "{\n" + "\n".join(dispatch) + "\n    return WOW64_MANUAL_UNIMPL;\n}\n"
    )
    if skipped:
        parts.append(
            "\n/* NOT generated (still loud NEEDS_MANUAL stubs):\n"
            + "\n".join(f" *   {s}" for s in sorted(skipped))
            + "\n */\n"
        )
    out_path.write_text("".join(parts))
    print(
        f"wrote {out_path}: {len(thunks)} thunks, "
        f"{len(emitted_structs)} struct marshaller pairs, "
        f"{len(skipped)} skipped",
        file=sys.stderr,
    )
    return 0


def cmd_report(args) -> int:
    b64, b32 = Path(args.build_dir), Path(args.build_dir32)

    print("== dumping layouts (i686 + aarch64) ...", file=sys.stderr)
    db32: dict[str, Record] = {}
    db64: dict[str, Record] = {}
    for tu_suffix, tu_src in WIN32K_TUS:
        for db, bdir in ((db32, b32), (db64, b64)):
            recs = dump_layouts(extract_flags(bdir, tu_suffix), tu_src)
            for k, v in recs.items():
                db.setdefault(k, v)  # first TU wins on duplicates
    print(f"   {len(db32)} / {len(db64)} records", file=sys.stderr)

    proto_text = "\n".join((REPO / h).read_text() for h in args.headers.split(","))
    protos = extract_prototypes(proto_text, r"Nt\w+")
    print(f"   {len(protos)} prototypes", file=sys.stderr)

    manual = load_needs_manual(Path(args.gen_c))
    print(f"   {len(manual)} NEEDS_MANUAL stubs in {args.gen_c}", file=sys.stderr)

    buckets: dict[str, list[str]] = {
        k: [] for k in ("FLAT-OK", "SCALAR_WIDEN", "WIDENING", "SEMANTIC", "NO-PROTO")
    }
    lines = []
    for name in manual:
        p = protos.get(name)
        if not p:
            buckets["NO-PROTO"].append(name)
            lines.append(f"{'NO-PROTO':13} {name}")
            continue
        worst, details = "FLAT", []
        for prm in p.params:
            cls, det = classify_param(prm, db32, db64)
            if PARAM_RANK[cls if cls != "IDENTICAL" else "FLAT"] > PARAM_RANK[worst]:
                worst = cls
            if cls not in ("FLAT", "IDENTICAL"):
                details.append(f"{prm.name}:{cls}{det}")
        key = {
            "FLAT": "FLAT-OK",
            "SCALAR_WIDEN": "SCALAR_WIDEN",
            "WIDENING": "WIDENING",
            "SEMANTIC": "SEMANTIC",
        }[worst]
        buckets[key].append(name)
        lines.append(f"{key:13} {name:40} {'; '.join(details)}")

    lines.sort()
    print("\n".join(lines))
    print()
    total = sum(len(v) for v in buckets.values())
    for k, v in buckets.items():
        print(f"{k:13} {len(v):4}  ({100 * len(v) // max(total, 1)}%)")
    print(f"{'TOTAL':13} {total:4}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    rp = sub.add_parser("report")
    rp.add_argument("--build-dir", default="build-arm64")
    rp.add_argument("--build-dir32", default="build-i386")
    rp.add_argument("--tu-suffix", default="user32_stubs.c")
    rp.add_argument("--gen-c", default="build-arm64/dll/win32/wow64/wow64_win32k_thunks.gen.c")
    rp.add_argument(
        "--headers",
        default="win32ss/include/ntuser.h,sdk/include/psdk/ntgdi.h,win32ss/include/ntgdibad.h",
    )
    rp.set_defaults(func=cmd_report)
    gp = sub.add_parser("gen")
    gp.add_argument("--build-dir", default="build-arm64")
    gp.add_argument("--build-dir32", default="build-i386")
    gp.add_argument("--gen-c", default="build-arm64/dll/win32/wow64/wow64_win32k_thunks.gen.c")
    gp.add_argument(
        "--headers",
        default="win32ss/include/ntuser.h,sdk/include/psdk/ntgdi.h,win32ss/include/ntgdibad.h",
    )
    gp.add_argument("--export-spec", default="win32ss/win32u/win32u.spec")
    gp.add_argument("--manual-list", default="tools/wow64gen/manual-win32k.txt")
    gp.add_argument("--out", default="dll/win32/wow64/wow64_automarshal.c")
    gp.set_defaults(func=cmd_gen)
    cp = sub.add_parser("gen-callback")
    cp.add_argument("--build-dir", default="build-arm64")
    cp.add_argument("--build-dir32", default="build-i386")
    cp.add_argument("--out", default="dll/win32/wow64/wow64_cb_automarshal.c")
    cp.add_argument("--out-header", default="dll/win32/wow64/wow64_cb_automarshal.h")
    cp.set_defaults(func=cmd_gen_callback)
    args = ap.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
