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
    offset: int          # byte offset (bitfields: byte of the storage unit)
    depth: int           # 1 = top-level member of the record
    ctype: str
    name: str            # "" for anonymous struct/union members
    is_bitfield: bool


@dataclass
class Record:
    name: str            # e.g. "struct tagWINDOWPOS" / "union _FOO"
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
            offset, bits, indent, decl = (
                int(m.group(1)), m.group(2), m.group(3), m.group(4)
            )
            depth = len(indent) // 2
            # decl is "TYPE name" or a nested "struct X name"; split the
            # trailing identifier off (may be absent for anonymous members).
            parts = decl.rsplit(" ", 1)
            if len(parts) == 2 and re.fullmatch(r"[A-Za-z_]\w*(\[\d+\])*", parts[1]):
                ctype, fname = parts
            else:
                ctype, fname = decl, ""
            cur.fields.append(Field(offset, depth, ctype.strip(), fname,
                                    bits is not None))
    return records


def dump_layouts(flags: list[str], tu_source: str) -> dict[str, Record]:
    with tempfile.NamedTemporaryFile("w", suffix=".c", delete=False) as f:
        f.write(tu_source)
        tu = f.name
    cmd = [str(CLANG), *flags, "-fsyntax-only",
           "-Xclang", "-fdump-record-layouts-complete", tu]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        errs = "\n".join(l for l in r.stderr.splitlines() if "error:" in l)[:2000]
        raise RuntimeError(f"layout TU failed to compile:\n{errs}")
    return parse_layout_dump(r.stdout)


# ---------------------------------------------------------------------------
# 3. Prototype extraction (textual — headers are pre-SAL-stripped mingw)
# ---------------------------------------------------------------------------

@dataclass
class Param:
    ctype: str           # normalized, e.g. "PWINDOWPOS", "DWORD", "RECT *"
    name: str
    sal: str             # "", "in", "out", "inout" (best effort)


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
        arglist = re.sub(r"/\*.*?\*/", " ", text[m.end():i - 1], flags=re.S)
        arglist = re.sub(r"//[^\n]*", " ", arglist)
        # return type: scan the few lines above for the last type-ish token
        pre = text[:m.start()].rsplit(";", 1)[-1]
        toks = [t for t in re.split(r"[\s]+", pre)
                if t and t not in ("NTSYSCALLAPI", "NTSYSAPI", "W32KAPI",
                                   "APIENTRY", "NTAPI", "WINAPI", "__stdcall",
                                   "__kernel_entry", "DECLSPEC_ADDRSAFE")]
        ret = toks[-1] if toks else "?"
        params: list[Param] = []
        if arglist.strip() not in ("", "VOID", "void"):
            for raw in _split_params(arglist):
                clean, sal = strip_annotations(raw)
                # old-style annotation macros (ntuser.h/ntgdi.h use these)
                for tok, d in (("INOUT", "inout"), ("IN", "in"), ("OUT", "out"),
                               ("OPTIONAL", "")):
                    if re.search(rf"\b{tok}\b", clean):
                        sal = sal or d
                        clean = re.sub(rf"\b{tok}\b", " ", clean)
                clean = re.sub(r"\s+", " ", clean).strip()
                pm = re.match(r"(.+?)\s*(\**)\s*([A-Za-z_]\w*)(\[\s*\d*\s*\])?$",
                              clean)
                if pm:
                    ctype = (pm.group(1) + (" " + pm.group(2) if pm.group(2)
                                            else "")).strip()
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
    "SIZE_T", "PSIZE_T", "ULONG_PTR", "LONG_PTR", "SSIZE_T", "INT_PTR",
    "UINT_PTR", "DWORD_PTR", "HANDLE", "PVOID", "LPVOID", "WPARAM", "LPARAM",
    "LRESULT", "KAFFINITY", "HWND", "HMENU", "HICON", "HCURSOR", "HBRUSH",
    "HDC", "HBITMAP", "HRGN", "HFONT", "HPALETTE", "HMODULE", "HINSTANCE",
    "HDESK", "HWINSTA", "HHOOK", "HKL", "HMONITOR", "HRAWINPUT", "HIMC",
}
# scalar pointees that are width-identical -> flat thunk already correct
SAFE_SCALARS = {
    "VOID", "CHAR", "UCHAR", "BOOLEAN", "BYTE", "WORD", "USHORT", "SHORT",
    "WCHAR", "INT", "UINT", "LONG", "ULONG", "DWORD", "BOOL", "FLOAT",
    "LONGLONG", "ULONGLONG", "LARGE_INTEGER", "ULARGE_INTEGER", "DWORD64",
    "UINT64", "INT64", "NTSTATUS", "ACCESS_MASK", "COLORREF", "LCID", "LANGID",
    "ATOM", "HRESULT",
    # C builtins (Windows LLP64: long is 4 bytes on both widths)
    "int", "unsigned", "long", "short", "char", "void", "float", "double",
    "unsigned int", "unsigned long", "unsigned short", "unsigned char",
    "signed char", "long long", "unsigned long long", "wchar_t", "__int64",
}


def _norm_type(ctype: str) -> str:
    t = re.sub(r"/\*.*?\*/", " ", ctype)
    toks = [k for k in t.split()
            if k not in ("IN", "OUT", "OPTIONAL", "CONST", "const", "FAR",
                         "NEAR", "volatile", "__unaligned")]
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
        for tag in (f"{kind} _{t}", f"{kind} tag{t}", f"{kind} _tag{t}",
                    f"{kind} {t}"):
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
            r = resolve_exact(t[len(prefix):], records)
            if r:
                return r
    return None


@dataclass
class StructVerdict:
    cls: str             # IDENTICAL | WIDENING | SEMANTIC
    detail: str
    rec32: Record | None = None
    rec64: Record | None = None


_STRING_PTEES = {"STR", "CSTR", "WSTR", "CWSTR", "TSTR", "CTSTR"}


def _scalar_base(t: str) -> str:
    """Peel P/LP from a scalar-pointer typedef name: PULONG -> ULONG."""
    for prefix in ("LPC", "PC", "LP", "P"):
        if t.startswith(prefix):
            return t[len(prefix):]
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
    if base in SAFE_SCALARS or base in ("float", "double", "int", "unsigned",
                                        "long", "char", "short",
                                        "unsigned int", "unsigned long",
                                        "unsigned short", "unsigned char"):
        return ("SAME", "")
    if base in DIVERGENT_SCALARS:
        return ("WIDEN", "")
    sb = _scalar_base(base)
    if sb in SAFE_SCALARS or sb in _STRING_PTEES:
        return ("WIDEN", "")          # pointer typedef to width-identical data
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


def diff_struct(name: str, db32: dict[str, Record],
                db64: dict[str, Record],
                _seen: set[str] | None = None) -> StructVerdict:
    _seen = _seen if _seen is not None else set()
    r32 = resolve_exact(name, db32)
    r64 = resolve_exact(name, db64)
    if not r32 or not r64:
        return StructVerdict("SEMANTIC", f"layout of '{name}' not in dump "
                                         "(add header to TU?)")
    if r32.name in _seen:
        return StructVerdict("IDENTICAL", "recursion cut")
    _seen.add(r32.name)

    f32, f64 = r32.top_fields(), r64.top_fields()
    same_layout = (r32.size == r64.size and
                   [(-1 if f.is_bitfield else f.offset, f.name) for f in f32]
                   == [(-1 if f.is_bitfield else f.offset, f.name) for f in f64])
    if [f.name for f in f32] != [f.name for f in f64]:
        return StructVerdict("SEMANTIC", "field lists differ across widths",
                             r32, r64)
    if r32.has_bitfield or r64.has_bitfield or r32.has_union or r64.has_union:
        if same_layout:
            return StructVerdict("IDENTICAL", "same layout (union/bitfield)",
                                 r32, r64)
        return StructVerdict("SEMANTIC", "union/bitfield with divergent layout",
                             r32, r64)
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
    for i, (a, b) in enumerate(zip(f32, f64)):
        (o32, s32), (o64, s64) = span(f32, i, r32.size), span(f64, i, r64.size)
        if s32 == s64:
            continue                       # same span: not a pointer; done
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
                        f"field {a.name}: ptr to divergent {pr.name} "
                        f"({sub.detail})", r32, r64)
            continue
        # other divergence: nested struct or array — classify by type
        cls, det = _field_class(a.ctype, db32, db64, _seen)
        if cls == "SEMANTIC":
            return StructVerdict(
                "SEMANTIC",
                f"field {a.name} ({s32}->{s64}): {det}", r32, r64)
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
        if base in SAFE_SCALARS or base in ("void", "char", "wchar_t", "int",
                                            "unsigned", "long", "short"):
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
        return (v.cls, f"-> {pr.name}: {v.detail}" if v.detail
                else f"-> {pr.name}")
    return ("SEMANTIC", f"-> {pointee}: layout not in dump (add header to TU?)")


PARAM_RANK = {"FLAT": 0, "IDENTICAL": 0, "SCALAR_WIDEN": 1, "WIDENING": 2,
              "SEMANTIC": 3}


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


def cmd_report(args) -> int:
    b64, b32 = Path(args.build_dir), Path(args.build_dir32)

    print("== dumping layouts (i686 + aarch64) ...", file=sys.stderr)
    db32: dict[str, Record] = {}
    db64: dict[str, Record] = {}
    for tu_suffix, tu_src in WIN32K_TUS:
        for db, bdir in ((db32, b32), (db64, b64)):
            recs = dump_layouts(extract_flags(bdir, tu_suffix), tu_src)
            for k, v in recs.items():
                db.setdefault(k, v)   # first TU wins on duplicates
    print(f"   {len(db32)} / {len(db64)} records", file=sys.stderr)

    proto_text = "\n".join(
        (REPO / h).read_text()
        for h in args.headers.split(",")
    )
    protos = extract_prototypes(proto_text, r"Nt\w+")
    print(f"   {len(protos)} prototypes", file=sys.stderr)

    manual = load_needs_manual(Path(args.gen_c))
    print(f"   {len(manual)} NEEDS_MANUAL stubs in {args.gen_c}", file=sys.stderr)

    buckets: dict[str, list[str]] = {k: [] for k in
                                     ("FLAT-OK", "SCALAR_WIDEN", "WIDENING",
                                      "SEMANTIC", "NO-PROTO")}
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
        key = {"FLAT": "FLAT-OK", "SCALAR_WIDEN": "SCALAR_WIDEN",
               "WIDENING": "WIDENING", "SEMANTIC": "SEMANTIC"}[worst]
        buckets[key].append(name)
        lines.append(f"{key:13} {name:40} {'; '.join(details)}")

    lines.sort()
    print("\n".join(lines))
    print()
    total = sum(len(v) for v in buckets.values())
    for k, v in buckets.items():
        print(f"{k:13} {len(v):4}  ({100*len(v)//max(total,1)}%)")
    print(f"{'TOTAL':13} {total:4}")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    rp = sub.add_parser("report")
    rp.add_argument("--build-dir", default="build-arm64")
    rp.add_argument("--build-dir32", default="build-i386")
    rp.add_argument("--tu-suffix", default="user32_stubs.c")
    rp.add_argument("--gen-c",
                    default="build-arm64/dll/win32/wow64/wow64_win32k_thunks.gen.c")
    rp.add_argument("--headers",
                    default="win32ss/include/ntuser.h,sdk/include/psdk/ntgdi.h,win32ss/include/ntgdibad.h")
    rp.set_defaults(func=cmd_report)
    args = ap.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
