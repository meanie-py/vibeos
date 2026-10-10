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
# Flags whose VALUE is a separate argv token.
_FLAG_TAKES_ARG = {"-I", "-isystem", "-include", "-D", "-U", "-iquote", "-idirafter"}


def extract_flags(build_dir: Path, tu_suffix: str) -> list[str]:
    """Pull include/define/target flags for a representative TU out of the
    build's compile_commands.json, rewriting canonical /tmp/vibeos paths back
    to this checkout (layout probing doesn't need the canon bind mount)."""
    db = json.loads((build_dir / "compile_commands.json").read_text())
    entry = next(e for e in db if e["file"].endswith(tu_suffix))
    flags = []
    toks = entry["command"].split()
    i = 0
    while i < len(toks):
        tok = toks[i]
        # `-isystem <path>` / `-I <path>` are TWO tokens when cmake emits them
        # separated. Taking only the first drops the path silently, and the probe
        # then fails on whatever that directory provided -- the 2026-09-20 upstream
        # merge changed one module's spelling and marshalgen lost
        # sdk/lib/pseh/include, i.e. <pseh/pseh2.h>, i.e. every layout.
        if tok in _FLAG_TAKES_ARG and i + 1 < len(toks):
            flags.append(tok)
            flags.append(toks[i + 1].replace("/tmp/vibeos", str(REPO)))
            i += 2
            continue
        if _FLAG_RE.match(tok) and not _FLAG_SKIP.match(tok):
            flags.append(tok.replace("/tmp/vibeos", str(REPO)))
        i += 1
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


def strip_comments(text: str) -> str:
    """Blank out block + line comments (newlines preserved).

    NOT cosmetic: ntuser.h keeps DEAD prototypes inside /* */ — the MSDN-shaped
    4-arg NtUserEnumDisplayMonitors sits commented directly above the REAL
    5-arg ROS one. Parsing a commented-out declaration yields a marshaller with
    the wrong arg count that reads the guest stack at the wrong offsets; only
    the last-definition-wins dict ordering hid it here. marshal32.exe caught
    that class the hard way. Only LIVE declarations may reach the classifier.
    """
    text = re.sub(r"/\*.*?\*/", lambda m: "\n" * m.group(0).count("\n"), text, flags=re.S)
    return re.sub(r"//[^\n]*", "", text)


def extract_prototypes(text: str, name_re: str) -> dict[str, Proto]:
    """Find `<ret>\\n<CONV>\\n?Name(\\n  params...);` shapes for names matching
    name_re. Tolerant of NTSYSCALLAPI/W32KAPI/APIENTRY/NTAPI decoration."""
    text = strip_comments(text)
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
    ("user32_layout_probe.c", "#include <user32.h>\n"),
    # winegdi is Rust now (its .c are commented out of the build); its stub TU keeps the
    # same target flags and include path, which is all the probe borrows.
    ("gdi32/wine/rust_cohab_stub.c", "#include <precomp.h>\n"),
]
# ndk umbrella for the ntdll surface. This MIRRORS dll/ntdll/include/ntdll.h's
# real include set rather than inventing a wrapper: a hand-rolled `<ntdef.h>` +
# `<ndk/ntndk.h>` pair walks straight into a redefinition wall (_FLOAT128,
# _LARGE_INTEGER, _LUID...), which is a fact about the WRAPPER, not the headers
# (field manual §9.108). NTOS_MODE_USER + WIN32_NO_STATUS are the two switches
# that decide which declarations the NDK emits, so they are load-bearing.
NTDLL_TU = (
    "#define WIN32_NO_STATUS\n"
    "#include <windef.h>\n"
    "#include <winbase.h>\n"
    "#include <winreg.h>\n"
    "#include <winuser.h>\n"
    "#define NTOS_MODE_USER\n"
    "#include <ndk/cmfuncs.h>\n"
    "#include <ndk/exfuncs.h>\n"
    "#include <ndk/iofuncs.h>\n"
    "#include <ndk/kefuncs.h>\n"
    "#include <ndk/ldrfuncs.h>\n"
    "#include <ndk/mmfuncs.h>\n"
    "#include <ndk/obfuncs.h>\n"
    "#include <ndk/psfuncs.h>\n"
    "#include <ndk/rtlfuncs.h>\n"
    "#include <ndk/sefuncs.h>\n"
    "#include <ndk/lpcfuncs.h>\n"
    "#include <ndk/pofuncs.h>\n"
    "#include <ndk/dbgkfuncs.h>\n"
    "#include <ndk/umfuncs.h>\n"
)


def needs_manual_from_sources(arch: str, table: str = "win32k") -> list[str]:
    """Syscalls wow64gen stubs as WOW64_NEEDS_MANUAL, from CHECKED-IN SOURCES.

    This used to scrape the BUILT `*_thunks.gen.c`, which made codegen a
    function of build state: adding a name to a manual list and regenerating
    without rebuilding first was a silent no-op, and the workflow that actually
    happens (edit list -> regenerate -> build) had the two steps in the order
    that cannot work. Same argument as compute_thunk_table(), and the artifact
    is not consulted here at all — `gen` legitimately runs BEFORE the build
    that produces it, so a staleness crosscheck would fire on the correct
    sequence.
    """
    return [n for n, live in compute_thunk_table(AUDIT_TABLES[table], arch).items() if not live]


# ---------------------------------------------------------------------------
# 5a. Audit driver — the flat-thunk BLIND SPOT (E10)
# ---------------------------------------------------------------------------
#
# `report` (above) classifies only syscalls ALREADY on the NEEDS_MANUAL list.
# That makes it structurally blind to the class most likely to be silently
# wrong: a syscall whose .spec types every arg flat (`ptr`/`long`) but whose
# REAL prototype takes a pointer to a divergent payload. wow64gen sees only the
# spec, so it emits a flat thunk and never stubs the call — the name never
# enters NEEDS_MANUAL, so `report` never looks at it. Production then hands
# 64-bit win32k the guest's 32-bit struct and the kernel reads/writes it with
# 64-bit eyes.
#
# That is not theoretical. It is the same bug three times:
#   E6  NtUserGetAtomName / NtUserGetClassName  (PUNICODE_STRING, spec `ptr`)
#   E8  NtUserUnregisterClass                   (PUNICODE_STRING, spec `ptr`)
#   E9  NtUserFindWindowEx                      (PUNICODE_STRING, spec `ptr`)
# each a garbage `Buffer` picked up from the wrong offset -> C0000005 in
# win32k's raw_move -> BUGCHECK 0x1E, and each found only because an applet
# happened to reach it. E7 is the same shape one step out: NtUserBuildHwndList
# fills a caller HWND[], and the flat thunk let the kernel write 8-byte handles
# into 4-byte guest slots — a 2x heap overrun that surfaced minutes later.
#
# So this audit inverts `report`'s question. Instead of "are the stubs we
# already flagged marshalled correctly?", it asks: **of every syscall
# production dispatches through an AUTO flat thunk, which ones have a
# prototype whose payload provably diverges 32<->64?** Those are latent
# bugchecks waiting for the first guest to call them.
#
# Ground truth is the GENERATED table, not the spec: an entry is `0` when
# wow64gen stubbed it (hand marshaller or auto-marshaller covers it) and
# `wow64_thunk_<Name>` when the flat thunk is live. We audit exactly the
# latter.

# `{ "NtUserFoo", wow64_thunk_NtUserFoo, 2 }, /* [551] */`  vs the MANUAL form
# `{ "NtUserBar", 0, 15 }, /* [342] MANUAL: ... */`
_TABLE_ENTRY = re.compile(r'^\s*\{\s*"([^"]+)",\s*(\w+),\s*\d+\s*\},')

# A SEMANTIC verdict that only means "marshalgen could not SEE this layout" is
# a hole in the TOOL, not a claim about the syscall. Keep the two apart: the
# gate fires on PROVEN divergence, and the blind spots are reported separately
# so the unknown set can be driven down deliberately instead of drowning the
# real findings.
_BLIND_DETAILS = ("not in dump", "unknown type", "unknown field type")

AUDIT_TABLES: dict[str, dict] = {
    "win32k": {
        "gen": "dll/win32/wow64/wow64_win32k_thunks.gen.c",
        # Checked-in inputs the table is a pure function of — see
        # compute_thunk_table(). Must mirror dll/win32/wow64/CMakeLists.txt.
        "spec": "win32ss/win32u/win32u.spec",
        "sysfuncs": "win32ss/w32ksvc32.h",
        "manual_lists": [
            "tools/wow64gen/manual-win32k.txt",
            "tools/wow64gen/automarshal-win32k.txt",
        ],
        "headers": [
            "win32ss/include/ntuser.h",
            "sdk/include/psdk/ntgdi.h",
            "win32ss/include/ntgdibad.h",
        ],
        "tus": WIN32K_TUS,
    },
    "ntdll": {
        "gen": "dll/win32/wow64/wow64_thunks.gen.c",
        "spec": "dll/ntdll/def/ntdll.spec",
        "sysfuncs": "ntoskrnl/include/sysfuncs.h",
        "manual_lists": ["tools/wow64gen/manual.txt"],
        "headers": [
            "sdk/include/ndk/cmfuncs.h",
            "sdk/include/ndk/dbgkfuncs.h",
            "sdk/include/ndk/exfuncs.h",
            "sdk/include/ndk/iofuncs.h",
            "sdk/include/ndk/kdfuncs.h",
            "sdk/include/ndk/kefuncs.h",
            "sdk/include/ndk/lpcfuncs.h",
            "sdk/include/ndk/mmfuncs.h",
            "sdk/include/ndk/obfuncs.h",
            "sdk/include/ndk/pofuncs.h",
            "sdk/include/ndk/psfuncs.h",
            "sdk/include/ndk/sefuncs.h",
        ],
        # ntdll is ~fully Rust-flipped; these two C stubs are what's left of it
        # in compile_commands.json, and they carry the ntdll include paths the
        # NDK umbrella needs. (If they ever go too, any TU that can `#include
        # <ndk/ntndk.h>` works — the probe only borrows the flags.)
        "tus": [("ntdll/rtl/rtl_um_rust_stub.c", NTDLL_TU)],
    },
}


def load_thunk_table(gen_c: Path) -> dict[str, bool]:
    """-> {syscall name: True if a LIVE auto flat thunk dispatches it}."""
    out: dict[str, bool] = {}
    for line in gen_c.read_text(encoding="utf-8", errors="replace").splitlines():
        m = _TABLE_ENTRY.match(line)
        if m:
            out[m.group(1)] = m.group(2) != "0"
    return out


def _load_wow64gen():
    """Import the sibling generator as a module (it is a script, not a package)."""
    import importlib.util

    path = Path(__file__).with_name("wow64gen.py")
    spec = importlib.util.spec_from_file_location("wow64gen", path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules.setdefault("wow64gen", mod)  # dataclass annotations resolve via sys.modules
    spec.loader.exec_module(mod)
    return mod


def compute_thunk_table(cfg: dict, arch: str) -> dict[str, bool]:
    """The thunk table derived from CHECKED-IN SOURCES, exactly as the build
    derives it — same generator, same inputs as dll/win32/wow64/CMakeLists.txt.

    Why not just read the generated .c: because that is a BUILD ARTIFACT, and a
    lint whose findings depend on build state is not portable across worktrees.
    Reading a stale artifact silently reports syscalls as unmarshalled that were
    fixed commits ago — which is not a hypothetical: it cost another lane a
    re-blessed baseline that accepted 33 already-fixed sites as debt (2026-07-24).
    The table is a pure function of (.spec, sysfuncs.h, the manual lists), all of
    which are checked in, so compute it and never go stale.
    """
    w = _load_wow64gen()
    manual = w.load_manual_list(",".join(str(REPO / m) for m in cfg["manual_lists"]))
    syscalls = w.parse_spec(str(REPO / cfg["spec"]), arch, manual)
    table = w.order_by_sysfuncs(w.parse_sysfuncs(str(REPO / cfg["sysfuncs"])), syscalls)
    return {sc.name: not sc.needs_manual for sc in table}


def crosscheck_artifact(computed: dict[str, bool], gen_c: Path, tname: str) -> None:
    """Compare the computed table against the BUILT one and fail loudly on any
    divergence. The computed table is authoritative (never stale); the artifact
    is what the running wow64.dll actually dispatches. They can only differ if
    the build dir predates a manual-list change — i.e. someone is about to read
    stale results — so say so instead of quietly picking one."""
    built = load_thunk_table(gen_c)
    if not built:
        return
    diffs = [
        (n, computed[n], built[n])
        for n in sorted(computed.keys() & built.keys())
        if computed[n] != built[n]
    ]
    if not diffs:
        return
    lines = "\n".join(
        f"    {n}: sources say {'FLAT' if c else 'marshalled'}, "
        f"built table says {'FLAT' if b else 'marshalled'}"
        for n, c, b in diffs[:12]
    )
    more = f"\n    ... and {len(diffs) - 12} more" if len(diffs) > 12 else ""
    raise RuntimeError(
        f"audit {tname}: the BUILT thunk table disagrees with the checked-in sources on "
        f"{len(diffs)} syscall(s) — {gen_c} is STALE.\n{lines}{more}\n"
        f"Rebuild before trusting this lint (`./vibeos-cli build --arch <a>`). Reporting "
        f"stale findings here is how already-fixed sites get re-blessed into the baseline."
    )


def audit_verdict(cls: str, detail: str) -> str:
    """classify_param() verdict -> DIVERGENT (a finding) | UNKNOWN | OK.

    SCALAR_WIDEN counts as DIVERGENT here, unlike in `report`. In a marshalled
    stub it is the easy case; under a LIVE FLAT THUNK a `HWND *` / `PVOID *`
    out-slot means the 64-bit kernel writes 8 bytes where the guest reserved 4
    — the E7 heap overrun.
    """
    if cls in ("FLAT", "IDENTICAL"):
        return "OK"
    if cls == "SEMANTIC" and any(b in detail for b in _BLIND_DETAILS):
        return "UNKNOWN"
    return "DIVERGENT"


def _resolve_build(spec: str | None, candidates: tuple[str, ...]) -> Path | None:
    """Absolute build dir that has a compile_commands.json, or None.

    With no --build-dir the 64-bit side is probed (amd64 then arm64) so the
    check works in an arm64-only or amd64-only checkout without a flag.
    """
    for cand in (spec,) if spec else candidates:
        p = Path(cand)
        p = p if p.is_absolute() else REPO / p
        if (p / "compile_commands.json").is_file():
            return p
    return None


def cmd_audit(args) -> int:
    b64 = _resolve_build(args.build_dir, ("build-amd64", "build-arm64"))
    b32 = _resolve_build(args.build_dir32, ("build-i386",))
    if b64 is None or b32 is None:
        which = "64-bit host" if b64 is None else "i386 guest"
        print(
            f"marshalgen audit: SKIP (no {which} build with compile_commands.json)",
            file=sys.stderr,
        )
        return 0
    if not CLANG.is_file():
        print("marshalgen audit: SKIP (no clang in .build-tools)", file=sys.stderr)
        return 0

    tables = list(AUDIT_TABLES) if args.table == "both" else [args.table]
    findings: list[str] = []
    unknown: list[str] = []
    noproto: list[str] = []
    audited = 0

    arch = "arm64" if b64.name.endswith("arm64") else "amd64"

    for tname in tables:
        cfg = AUDIT_TABLES[tname]
        # The table comes from CHECKED-IN SOURCES, so this lint's findings do not
        # depend on build freshness and its baseline is portable across lanes.
        # The built artifact is still consulted — but only to prove it AGREES.
        table = compute_thunk_table(cfg, arch)
        gen = b64 / cfg["gen"]
        if gen.is_file():
            crosscheck_artifact(table, gen, tname)

        # A layout dump that fails HERE is coverage silently lost — the build
        # exists, so every finding this table would have produced just vanishes.
        # Let it raise: the check harness treats "nonzero exit, no findings" as
        # the tool failing to run, which is the honest verdict.
        db32: dict[str, Record] = {}
        db64: dict[str, Record] = {}
        for tu_suffix, tu_src in cfg["tus"]:
            for db, bdir in ((db32, b32), (db64, b64)):
                try:
                    flags = extract_flags(bdir, tu_suffix)
                except StopIteration:
                    raise RuntimeError(
                        f"audit {tname}: no TU matching '{tu_suffix}' in "
                        f"{bdir}/compile_commands.json — the layout probe cannot run, so "
                        f"this table would go UNCHECKED. Point cfg['tus'] at a TU that "
                        f"still exists."
                    ) from None
                for k, v in dump_layouts(flags, tu_src).items():
                    db.setdefault(k, v)
        if not db32 or not db64:
            raise RuntimeError(
                f"audit {tname}: layout dump came back empty (32:{len(db32)} 64:{len(db64)}) "
                f"— refusing to report a clean table from no data."
            )

        protos = extract_prototypes(
            "\n".join((REPO / h).read_text(errors="replace") for h in cfg["headers"]), r"Nt\w+"
        )
        print(
            f"marshalgen audit [{tname}]: {len(table)} entries, "
            f"{sum(table.values())} live flat thunks, {len(protos)} prototypes, "
            f"{len(db32)}/{len(db64)} records "
            f"(table from sources{', built table agrees' if gen.is_file() else ''})",
            file=sys.stderr,
        )

        for name, is_auto in sorted(table.items()):
            if not is_auto:
                continue  # stubbed: a hand or auto marshaller owns it
            p = protos.get(name)
            if p is None:
                noproto.append(f"{tname} {name}")
                continue
            audited += 1
            for prm in p.params:
                cls, det = classify_param(prm, db32, db64)
                v = audit_verdict(cls, det)
                line = f"{tname} {name}({prm.name}) {cls} {det}"
                if v == "DIVERGENT":
                    findings.append(line)
                elif v == "UNKNOWN":
                    unknown.append(line)

    # Findings on STDOUT (the harness baselines these line-by-line); everything
    # else on stderr so a clean run adds nothing to the gated set.
    for line in findings:
        print(f"FLAT-DIVERGENT: {line}")

    print(
        f"marshalgen audit: {audited} flat-thunked syscalls audited — "
        f"{len(findings)} DIVERGENT, {len(unknown)} unknown-layout, "
        f"{len(noproto)} no-prototype",
        file=sys.stderr,
    )
    if args.show_unknown:
        for line in unknown:
            print(f"   unknown-layout: {line}", file=sys.stderr)
        for line in noproto:
            print(f"   no-prototype:   {line}", file=sys.stderr)
    if findings:
        print(
            "\nEach line is a syscall production dispatches through a FLAT thunk "
            "whose real prototype takes a payload that provably differs between "
            "32- and 64-bit. The guest's struct reaches the 64-bit kernel unmarshalled.\n"
            "Fix: add the name to tools/wow64gen/manual{,-win32k}.txt (which turns it "
            "into a loud WOW64_AUTOMARSHAL_UNIMPL stub — a clean error, never a bugcheck), "
            "then write the marshaller in dll/win32/wow64/wow64_manual.c or let "
            "`marshalgen gen` emit one.",
            file=sys.stderr,
        )
    return 1 if findings else 0


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


# Names of a leading SELF-DESCRIBING SIZE field: the struct states its own
# sizeof so the callee can version-check it. Under WoW64 that check is made
# against the HOST sizeof, so a field-perfect marshaller still gets the call
# rejected — see _gen_struct_marshaller. Restricted to this list (and to a
# 4-byte field at offset 0) rather than "anything that looks like a size",
# because rewriting a field that is NOT the struct size is a data corruption,
# whereas missing one is only the status quo.
_SIZE_FIELD_NAMES = frozenset({"cbSize", "lStructSize", "dwSize", "Size", "Length"})


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
            # A leading self-describing size is the one field a verbatim copy
            # gets WRONG even though its offset and width match on both sides:
            # the guest wrote ITS sizeof, and the callee compares against the
            # HOST's. win32k rejects the whole call on the mismatch
            # (NtUserTrackMouseEvent / GetCursorInfo / GetGUIThreadInfo /
            # GetComboBoxInfo all bail ERROR_INVALID_PARAMETER), so copying it
            # faithfully is exactly how a "correct" marshaller stays broken.
            if i == 0 and a.offset == 0 and s32 == 4 and a.name in _SIZE_FIELD_NAMES:
                body_in.append(
                    f"    WowAmSizeIn(d + {o64}, s + {o32}, {rec32.size}, {rec64.size});{cmt}"
                )
                body_out.append(
                    f"    WowAmSizeOut(d + {o32}, s + {o64}, {rec32.size}, {rec64.size});{cmt}"
                )
                continue
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

#include "wow64_status.h"

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

/* A leading self-describing size ("cbSize"): the guest states ITS sizeof and
 * the callee compares against the HOST's, so a verbatim copy fails the very
 * version check the field exists for. Rewrite it — but ONLY when it is
 * exactly the 32-bit sizeof, so a struct with more than one legal size
 * (MONITORINFO vs MONITORINFOEXW) still reaches the kernel with the size the
 * guest actually chose, and a garbage value still gets rejected as garbage. */
static void WowAmSizeIn(unsigned char *d, const unsigned char *s,
                        ULONG g32, ULONG h64)
{
    ULONG v;
    WowAmCpy((unsigned char *)&v, s, 4);
    if (v == g32) v = h64;
    WowAmCpy(d, (const unsigned char *)&v, 4);
}

/* ...and undo it on the way back: the kernel restamps cbSize with the host
 * sizeof, which would leave the guest holding a size that does not describe
 * its own struct. */
static void WowAmSizeOut(unsigned char *d, const unsigned char *s,
                         ULONG g32, ULONG h64)
{
    ULONG v;
    WowAmCpy((unsigned char *)&v, s, 4);
    if (v == h64) v = g32;
    WowAmCpy(d, (const unsigned char *)&v, 4);
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
# E13: this table is now shared by BOTH directions — wow64_callback.c uses it
# for the kernel->guest WINDOWPROC payload, and Wow64Manual_NtUserMessageCall
# for the guest->kernel SendMessage lParam. Adding a row fixes both at once,
# and they can no longer disagree about a layout.
CB_TABLE_MESSAGES = [
    ("WM_DRAWITEM", 0x002B, "DRAWITEMSTRUCT"),
    ("WM_MEASUREITEM", 0x002C, "MEASUREITEMSTRUCT"),
    ("WM_HELP", 0x0053, "HELPINFO"),
    ("WM_NEXTMENU", 0x0213, "MDINEXTMENU"),
    # E13: WINDOWPOS was an EXTRA struct only, so the Msg->pair lookup missed
    # it and the callback direction fell through to the verbatim copy (40 host
    # bytes into a buffer the guest reads as 28). M16 had hand-patched just the
    # FNID_DEFWINDOWPROC leg of the OTHER direction; a table row covers both.
    ("WM_WINDOWPOSCHANGING", 0x0046, "WINDOWPOS"),
    ("WM_WINDOWPOSCHANGED", 0x0047, "WINDOWPOS"),
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
    manual = needs_manual_from_sources("arm64" if b64.name.endswith("arm64") else "amd64")
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
        "{\n" + "\n".join(dispatch) + "\n    return WOW64_AUTOMARSHAL_UNIMPL;\n}\n"
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

    manual = needs_manual_from_sources("arm64" if b64.name.endswith("arm64") else "amd64")
    print(f"   {len(manual)} NEEDS_MANUAL stubs (from the checked-in lists)", file=sys.stderr)

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
    rp.add_argument("--tu-suffix", default="user32_layout_probe.c")
    rp.add_argument(
        "--headers",
        default="win32ss/include/ntuser.h,sdk/include/psdk/ntgdi.h,win32ss/include/ntgdibad.h",
    )
    rp.set_defaults(func=cmd_report)
    ap_ = sub.add_parser(
        "audit",
        help="find LIVE flat thunks whose real prototype has a divergent payload",
    )
    ap_.add_argument("--build-dir", default=None, help="64-bit host build (probed if omitted)")
    ap_.add_argument("--build-dir32", default=None)
    ap_.add_argument("--table", default="both", choices=[*AUDIT_TABLES, "both"])
    ap_.add_argument("--show-unknown", action="store_true")
    ap_.set_defaults(func=cmd_audit)
    gp = sub.add_parser("gen")
    gp.add_argument("--build-dir", default="build-arm64")
    gp.add_argument("--build-dir32", default="build-i386")
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
