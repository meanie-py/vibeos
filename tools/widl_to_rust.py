#!/usr/bin/env python3
"""widl_to_rust — translate a widl/MIDL-generated RPC *server* stub (`<iface>_s.c`)
into Rust that builds the same `repr(C)` static graph against the `midl-rt`
crate, with no C.

The widl server stub is almost entirely *data*: two byte-array format strings
(interpreted at runtime by rpcrt4's NDR engine), a graph of cross-referencing
`MIDL_STUB_DESC` / `MIDL_SERVER_INFO` / `RPC_SERVER_INTERFACE` /
`RPC_DISPATCH_TABLE` statics, function-address tables (server routines,
`NdrServerCall2` dispatch fns, rundowns, ExprEvals), and the one exported
`<iface>_v<M>_<m>_s_ifspec` handle. The only executable code is a couple of tiny
`ExprEval` conformance routines that poke `MIDL_STUB_MESSAGE`.

This is a faithful 1:1 transliteration — same data, same NDR engine — emitted
against `midl-rt`'s ABI-asserted type mirror. It does NOT reimplement widl; it
consumes widl's C output, plus the *compiled* C stub object (`<iface>_s.c.obj`,
which cmake already builds) to read the EXACT per-arch external symbol
decorations — so the Rust externs match services-rust / rpcrt4 bit-for-bit
without re-deriving i386 stdcall `@N` (which by-value struct params make
non-trivial to compute from the prototype).

Usage:  widl_to_rust.py <iface>_s.c --symobj <iface>_s.c.obj --arch <a> \
            --nm <llvm-nm> -o <iface>_s.rs
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

# ---- helpers ---------------------------------------------------------------


def strip_comments(text: str) -> str:
    """Drop /* ... */ and // ... comments (the stub has no string literal that
    contains these sequences except protseq endpoints, which we parse before
    stripping)."""
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.DOTALL)
    text = re.sub(r"//[^\n]*", "", text)
    return text


def brace_block(text: str, start: int) -> str:
    """Return the balanced { ... } substring beginning at the first '{' at or
    after `start` (inclusive of the braces)."""
    i = text.index("{", start)
    depth = 0
    for j in range(i, len(text)):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return text[i : j + 1]
    raise ValueError("unbalanced braces")


def fail(msg: str):
    sys.stderr.write(f"widl_to_rust: ERROR: {msg}\n")
    sys.exit(2)


# ---- parse: interface identity ---------------------------------------------


def parse_iface(c: str):
    m = re.search(r"\b(\w+)_v(\d+)_(\d+)_s_ifspec\b", c)
    if not m:
        fail("could not find <iface>_v<M>_<m>_s_ifspec")
    return m.group(1), int(m.group(2)), int(m.group(3))


# ---- parse: RPC_SERVER_INTERFACE (the two syntax identifiers + flags) -------


# `{{0x..,0x..,0x..,{0x..,0x..,0x..,0x..,0x..,0x..,0x..,0x..}},{maj,min}}`
SYNTAX_ID_RE = re.compile(
    r"\{\s*\{\s*(0x[0-9a-fA-F]+)\s*,\s*(0x[0-9a-fA-F]+)\s*,\s*(0x[0-9a-fA-F]+)\s*,"
    r"\s*\{([^}]*)\}\s*\}\s*,\s*\{\s*(\d+)\s*,\s*(\d+)\s*\}\s*\}"
)


def parse_syntax_id(m):
    d1, d2, d3 = int(m.group(1), 0), int(m.group(2), 0), int(m.group(3), 0)
    d4 = [int(x.strip(), 0) for x in m.group(4).split(",")]
    if len(d4) != 8:
        fail(f"syntax id data4 expected 8 bytes, got {len(d4)}")
    return (d1, d2, d3, d4), (int(m.group(5)), int(m.group(6)))


def parse_server_interface(c: str, iface: str):
    m = re.search(r"RPC_SERVER_INTERFACE\s+" + iface + r"___RpcServerInterface\s*=", c)
    if not m:
        fail("RPC_SERVER_INTERFACE not found")
    block = brace_block(c, m.end())
    ids = list(SYNTAX_ID_RE.finditer(block))
    if len(ids) != 2:
        fail(f"expected 2 syntax identifiers, found {len(ids)}")
    interface_id = parse_syntax_id(ids[0])
    transfer_syntax = parse_syntax_id(ids[1])
    # flags = last top-level scalar before the closing brace
    tail = strip_comments(block).rstrip().rstrip("}").rstrip().rstrip(",")
    flags = int(tail.split(",")[-1].strip() or "0", 0)
    # protseq endpoint count: the scalar right after the DispatchTable ref
    mcount = re.search(r"DispatchTable\s*,\s*(\d+)\s*,", block)
    pcount = int(mcount.group(1)) if mcount else 1
    return interface_id, transfer_syntax, flags, pcount


# ---- parse: protseq endpoints ----------------------------------------------


def parse_protseq(c: str, iface: str):
    m = re.search(r"\b" + iface + r"__RpcProtseqEndpoint\s*\[\s*\]\s*\[\s*2\s*\]\s*=", c)
    if not m:
        return []
    block = brace_block(c, m.end())
    rows = []
    for row in re.findall(r"\{\s*([^{}]*?)\s*\}", block):
        strs = re.findall(r'"((?:[^"\\]|\\.)*)"', row)
        if len(strs) == 2:
            rows.append((strs[0], strs[1]))
    return rows


# ---- parse: simple identifier-list arrays ----------------------------------


def parse_named_array(c: str, decl_regex: str):
    m = re.search(decl_regex, c)
    if not m:
        return None
    block = brace_block(c, m.end())
    inner = strip_comments(block)[1:-1]
    items = [t.strip() for t in inner.split(",")]
    return [t for t in items if t]


def parse_u16_array(c: str, name: str):
    m = re.search(r"unsigned short\s+" + name + r"\s*\[\s*\]\s*=", c)
    if not m:
        fail(f"{name} not found")
    inner = strip_comments(brace_block(c, m.end()))[1:-1]
    return [int(t.strip(), 0) for t in inner.split(",") if t.strip()]


# ---- parse: MIDL_STUB_DESC scalar fields -----------------------------------


def parse_stub_desc(c: str, iface: str):
    m = re.search(r"MIDL_STUB_DESC\s+" + iface + r"_StubDesc\s*=", c)
    if not m:
        fail("MIDL_STUB_DESC definition not found")
    block = brace_block(c, m.end())

    def grab(anchor, default=0):
        mm = re.search(r"(0x[0-9a-fA-F]+|\d+)\s*,\s*/\*[^*]*" + anchor, block)
        return int(mm.group(1), 0) if mm else default

    f_check_bounds = grab(r"bounds_check")
    version = grab(r"Ndr library version")
    midl_version = grab(r"MIDL Version")
    m_flags = grab(r"Flags")
    has_rundown = "RundownRoutines" in block
    has_expr = "ExprEvalRoutines" in block
    return dict(
        f_check_bounds=f_check_bounds,
        version=version,
        midl_version=midl_version,
        m_flags=m_flags,
        has_rundown=has_rundown,
        has_expr=has_expr,
    )


# ---- parse: dispatch table (count + entries) -------------------------------


def parse_dispatch(c: str, iface: str):
    m = re.search(r"RPC_DISPATCH_FUNCTION\s+" + iface + r"_table\s*\[\s*\]\s*=", c)
    if not m:
        fail("dispatch function table not found")
    inner = strip_comments(brace_block(c, m.end()))[1:-1]
    entries = [t.strip() for t in inner.split(",") if t.strip()]
    md = re.search(r"RPC_DISPATCH_TABLE\s+" + iface + r"_v\d+_\d+_DispatchTable\s*=", c)
    count = None
    if md:
        binner = strip_comments(brace_block(c, md.end()))[1:-1]
        count = int(binner.split(",")[0].strip(), 0)
    return entries, count


# ---- parse: server routine table + param counts from the .h ----------------


def parse_server_routines(c: str, iface: str):
    m = re.search(r"SERVER_ROUTINE\s+" + iface + r"_ServerRoutineTable\s*\[\s*\]\s*=", c)
    if not m:
        fail("server routine table not found")
    inner = strip_comments(brace_block(c, m.end()))[1:-1]
    names = re.findall(r"\(\s*void\s*\*\s*\)\s*(\w+)", inner)
    return names


def build_decoration_map(symobj: str, nm: str, arch: str):
    """nm the compiled C stub object and map each external (undefined) symbol's
    BASE name → its exact link name (with i386 `_..@N` decoration). This is the
    ground truth the Rust externs must match — services-rust / rpcrt4 define
    exactly these symbols, since the C stub links against them today."""
    out = subprocess.run([nm, symobj], capture_output=True, text=True).stdout
    mapping = {}
    for line in out.splitlines():
        m = re.match(r"\s*U\s+(\S+)$", line)
        if not m:
            continue
        sym = m.group(1)
        link = sym
        # On i686-pc-windows-gnu the toolchain PREPENDS a leading `_` to an
        # `extern "C"` `#[link_name]` (the i386-gnu decoration quirk). So the
        # link name must carry the `@N` stdcall suffix but NOT the leading `_`
        # (the toolchain re-adds it). amd64/arm64 names are bare — no change.
        if arch == "i386" and link.startswith("_"):
            link = link[1:]
        base = link.split("@", 1)[0]
        mapping[base] = link
    return mapping


def linkname(mapping, name):
    if name in mapping:
        return mapping[name]
    fail(
        f"external symbol {name!r} not found in compiled stub object — cannot "
        f"determine its link decoration"
    )


# ---- parse: ExprEval routines ----------------------------------------------

# C param-struct field types → Rust. Pointers-to-DWORD that get dereferenced
# become *mut u32; plain scalars u32; everything else an opaque pointer.
DWORD_PTR_TYPES = {"LPDWORD", "PDWORD", "PULONG", "LPLONG", "PLONG"}
SCALAR_TYPES = {"DWORD", "ULONG", "LONG", "UINT", "INT", "BOOL"}


def map_field_type(ctype: str) -> str:
    t = ctype.strip()
    if t.endswith("*"):
        base = t[:-1].strip()
        return "*mut u32" if base in ("DWORD", "ULONG", "LONG") else "*const core::ffi::c_void"
    if t in DWORD_PTR_TYPES:
        return "*mut u32"
    if t in SCALAR_TYPES:
        return "u32"
    return "*const core::ffi::c_void"


# Exact-width scalar map for synthesizing a `repr(C)` *view* of a real conformant
# type whose leading fields an ExprEval reads (e.g. `LSAPR_ACL.AclSize`). Unlike
# `map_field_type` (which is fine collapsing param-struct scalars to u32), the
# view must reproduce the C layout byte-for-byte so the read lands at the right
# offset — so widths are exact and anything unknown hard-fails.
VIEW_SCALAR = {
    "BYTE": "u8",
    "UCHAR": "u8",
    "CHAR": "u8",
    "CCHAR": "u8",
    "BOOLEAN": "u8",
    "WORD": "u16",
    "USHORT": "u16",
    "SHORT": "u16",
    "WCHAR": "u16",
    "DWORD": "u32",
    "ULONG": "u32",
    "LONG": "u32",
    "UINT": "u32",
    "INT": "u32",
    "BOOL": "u32",
    "HRESULT": "u32",
    "ULONG64": "u64",
    "LONGLONG": "u64",
    "ULONGLONG": "u64",
    "DWORD64": "u64",
    # Raw C spellings — widl-generated headers emit these (e.g. SAMPR_LOGON_HOURS
    # `unsigned short UnitsPerWeek`), not the SDK typedefs. Whitespace-normalized.
    "char": "u8",
    "signed char": "u8",
    "unsigned char": "u8",
    "short": "u16",
    "short int": "u16",
    "unsigned short": "u16",
    "unsigned short int": "u16",
    "int": "u32",
    "unsigned": "u32",
    "unsigned int": "u32",
    "long": "u32",
    "long int": "u32",
    "unsigned long": "u32",
    "unsigned long int": "u32",
    "long long": "u64",
    "unsigned long long": "u64",
}


def synth_view_from_header(type_name: str, header: str, fname: str):
    """Build a `repr(C)` view of `type_name`'s LEADING scalar fields from its
    definition in the companion `<iface>_s.h`. Used by the named-type ExprEval
    idiom (`TYPE *pS = (TYPE *)(pStubMsg->StackTop - N)`), where widl reads a
    real conformant type's header instead of a synthetic `_PARAM_STRUCT`. We
    only model the leading scalar prefix (stop at the first array/pointer/
    aggregate — the conformant tail), and hard-fail on anything non-scalar so a
    layout mismatch can never silently corrupt a buffer size."""
    found = None
    for m in re.finditer(r"typedef\s+struct\s+\w*\s*(?=\{)", header):
        blk = brace_block(header, m.end())
        after = header[header.index(blk, m.end()) + len(blk) :]
        tm = re.match(r"\s*([A-Za-z_]\w*)", after)
        if tm and tm.group(1) == type_name:
            found = blk
            break
    if found is None:
        fail(f"{fname}: ExprEval cast type {type_name} not found in companion header")
    fields = []
    for line in found[1:-1].split(";"):
        line = line.strip()
        if not line:
            continue
        if "*" in line:
            # pointer field — conformant tail (e.g. PUCHAR LogonHours); scalar header ends here
            break
        # Greedy type capture: field name is the last identifier, the C type is
        # everything before it (may be multi-word, e.g. `unsigned short`).
        mm = re.match(r"(.+?)\s+([A-Za-z_]\w*)\s*(\[[^\]]*\])?$", line)
        if not mm:
            fail(f"{fname}: ExprEval view field unparseable {line!r}")
        ctype, fld, arr = re.sub(r"\s+", " ", mm.group(1).strip()), mm.group(2), mm.group(3)
        if arr:
            break  # conformant/array tail — leading scalar header ends here
        rty = VIEW_SCALAR.get(ctype)
        if rty is None:
            fail(f"{fname}: ExprEval view field {ctype} {fld} not a known scalar")
        fields.append((fld, rty))
    if not fields:
        fail(f"{fname}: ExprEval view of {type_name} has no leading scalar fields")
    return fields


def translate_expr_eval(body: str, fname: str, header: str = ""):
    """Translate one ExprEval routine body into a Rust unsafe extern fn body.
    Only the bounded set of MIDL conformance idioms is accepted; anything else
    hard-fails (a wrong ExprEval is a silent runtime buffer-size corruption).

    Two pointer-init idioms are recognized:
      (a) inline    : `struct _PARAM_STRUCT { .. } *pS = (..)pStubMsg->StackTop;`
      (b) named-type : `TYPE *pS = (TYPE *)(pStubMsg->StackTop [+-] N);`
          (widl emits this when the conformant size_is references a real type's
          field, e.g. `[size_is(AclSize - 4)]` inside `LSAPR_ACL`)."""
    ms = re.search(r"struct\s+_PARAM_STRUCT\s*", body)
    if ms:
        # (a) inline _PARAM_STRUCT — StackTop with no offset, fields inline.
        sblock = brace_block(body, ms.end())
        fields = []
        for line in sblock[1:-1].split(";"):
            line = line.strip()
            if not line:
                continue
            mm = re.match(r"(.*?)([A-Za-z_]\w*)\s*$", line)
            if not mm or not mm.group(1).strip():
                fail(f"{fname}: bad param field {line!r}")
            ctype, fname_f = mm.group(1).strip(), mm.group(2)
            fields.append((fname_f, map_field_type(ctype)))
        stack_off = 0
    else:
        # (b) named-type cast over (StackTop [+-] N) — synthesize the view.
        mi = re.search(
            r"(\w+)\s*\*\s*pS\s*=\s*\(\s*\w+\s*\*\s*\)\s*\(?\s*pStubMsg->StackTop"
            r"\s*(?:([+-])\s*(0x[0-9a-fA-F]+|\d+))?\s*\)?\s*;",
            body,
        )
        if not mi:
            fail(f"{fname}: no _PARAM_STRUCT and no recognized named-type StackTop cast")
        type_name = mi.group(1)
        sign, num = mi.group(2), mi.group(3)
        stack_off = (int(num, 0) * (-1 if sign == "-" else 1)) if num else 0
        fields = synth_view_from_header(type_name, header, fname)

    # statements after the `*pS = (...)pStubMsg->StackTop...;`, up to the closing
    # brace of the function body.
    inner = body[: body.rindex("}")]
    stmts_src = inner[inner.index(";", inner.index("StackTop")) + 1 :]
    out_stmts = []
    for stmt in stmts_src.split(";"):
        s = stmt.strip()
        if not s:
            continue
        out_stmts.append(translate_expr_stmt(s, fname))

    ps_init = "(*p_stub_msg).stack_top"
    if stack_off:
        ps_init += f".offset({stack_off})"
    rust_fields = "\n".join(f"        {fn}: {rty}," for fn, rty in fields)
    body_rs = "\n".join(f"    {st}" for st in out_stmts)
    return f"""unsafe extern "system" fn {fname}(p_stub_msg: *mut midl_rt::MIDL_STUB_MESSAGE) {{
    #[repr(C)]
    struct ParamStruct {{
{rust_fields}
    }}
    let ps = {ps_init} as *const ParamStruct;
    let _ = &ps;
{body_rs}
}}"""


def translate_expr_stmt(s: str, fname: str):
    # pStubMsg->Offset = 0;
    m = re.match(r"pStubMsg->Offset\s*=\s*(\w+)$", s)
    if m:
        return f"(*p_stub_msg).offset = {int(m.group(1), 0)};"
    # pStubMsg->MaxCount = (ULONG_PTR)( EXPR );
    m = re.match(r"pStubMsg->MaxCount\s*=\s*\(\s*ULONG_PTR\s*\)\s*\((.*)\)$", s)
    if m:
        return f"(*p_stub_msg).max_count = ({translate_rhs(m.group(1).strip(), fname)}) as usize;"
    # pStubMsg->MaxCount = (ULONG_PTR)EXPR;  (no inner parens)
    m = re.match(r"pStubMsg->MaxCount\s*=\s*\(\s*ULONG_PTR\s*\)\s*(.*)$", s)
    if m:
        return f"(*p_stub_msg).max_count = ({translate_rhs(m.group(1).strip(), fname)}) as usize;"
    fail(f"{fname}: unhandled ExprEval statement {s!r}")


def _strip_outer_parens(e: str) -> str:
    """Strip a fully-enclosing balanced paren pair (repeatedly)."""
    e = e.strip()
    while e.startswith("(") and e.endswith(")"):
        depth = 0
        wraps = True
        for i, ch in enumerate(e):
            if ch == "(":
                depth += 1
            elif ch == ")":
                depth -= 1
                if depth == 0 and i != len(e) - 1:
                    wraps = False
                    break
        if wraps:
            e = e[1:-1].strip()
        else:
            break
    return e


def _split_top_op(e: str, ops: str):
    """(index, op) of the LAST top-level (paren-depth 0) binary operator in `ops`,
    skipping a leading-unary occurrence (depth-0 op at the very start, e.g. the
    `*` of `*pS->X`). None if no such operator."""
    depth = 0
    found = None
    for i, ch in enumerate(e):
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        elif depth == 0 and ch in ops and e[:i].strip():
            # don't mistake the `-` of a `->` member-access arrow for subtraction
            if ch == "-" and i + 1 < len(e) and e[i + 1] == ">":
                continue
            found = (i, ch)
    return found


def translate_rhs(e: str, fname: str):
    # All sub-expressions are normalized to u32 so mixed-width field arithmetic
    # (e.g. a u16 UnitsPerWeek count) can't type-mismatch; the result is cast to
    # usize at the MaxCount site. Precedence-correct: split at the lowest-binding
    # top-level operator first (additive, then multiplicative).
    e = _strip_outer_parens(e)
    # ternary:  pS->X ? *pS->X : 0   /  pS->X ? <a> : <b>
    m = re.match(r"(.+?)\s*\?\s*(.+?)\s*:\s*(.+)$", e)
    if m:
        cond, a, b = m.group(1).strip(), m.group(2).strip(), m.group(3).strip()
        # cond is a pointer truthiness test
        cm = re.match(r"pS->(\w+)$", cond)
        if not cm:
            fail(f"{fname}: unhandled ternary cond {cond!r}")
        return (
            f"if !(*ps).{cm.group(1)}.is_null() {{ {translate_rhs(a, fname)} }} "
            f"else {{ {translate_rhs(b, fname)} }}"
        )
    # additive (lowest precedence): last top-level + or -
    op = _split_top_op(e, "+-")
    if op:
        i, o = op
        fn = "wrapping_add" if o == "+" else "wrapping_sub"
        return f"({translate_rhs(e[:i], fname)}).{fn}({translate_rhs(e[i + 1 :], fname)})"
    # multiplicative: last top-level * or /
    op = _split_top_op(e, "*/")
    if op:
        i, o = op
        fn = "wrapping_mul" if o == "*" else "wrapping_div"
        return f"({translate_rhs(e[:i], fname)}).{fn}({translate_rhs(e[i + 1 :], fname)})"
    # *pS->FIELD
    m = re.match(r"\*\s*pS->(\w+)$", e)
    if m:
        return f"(*(*ps).{m.group(1)} as u32)"
    # pS->FIELD
    m = re.match(r"pS->(\w+)$", e)
    if m:
        return f"((*ps).{m.group(1)} as u32)"
    # integer literal
    if re.match(r"^(0x[0-9a-fA-F]+|\d+)$", e):
        return f"{int(e, 0)}u32"
    fail(f"{fname}: unhandled ExprEval expr {e!r}")


def parse_expr_evals(c: str, header: str = ""):
    routines = []
    for m in re.finditer(
        r"static\s+void\s+__RPC_USER\s+(\w+)\s*\(\s*PMIDL_STUB_MESSAGE\s+pStubMsg\s*\)",
        c,
    ):
        fname = m.group(1)
        body = brace_block(c, m.end())
        routines.append((fname, translate_expr_eval(body, fname, header)))
    return routines


# ---- parse: format strings → byte arrays -----------------------------------


def parse_format_string(c: str, struct_type: str, var: str):
    m = re.search(struct_type + r"\s+" + var + r"\s*=", c)
    if not m:
        fail(f"{var} not found")
    block = brace_block(c, m.end())
    # block = { <pad>, { <bytes...> } }  — take the inner (Format[]) brace
    inner_start = block.index("{", 1)
    fmt_block = brace_block(block, inner_start)
    text = strip_comments(fmt_block)[1:-1]
    out = []
    for tok in re.findall(
        r"NdrFcShort\(\s*(0x[0-9a-fA-F]+|\d+)\s*\)"
        r"|NdrFcLong\(\s*(0x[0-9a-fA-F]+|\d+)\s*\)"
        r"|(0x[0-9a-fA-F]+|\d+)",
        text,
    ):
        sh, lo, by = tok
        if sh:
            v = int(sh, 0) & 0xFFFFFFFF
            out += [v & 0xFF, (v >> 8) & 0xFF]
        elif lo:
            v = int(lo, 0) & 0xFFFFFFFF
            out += [v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF, (v >> 24) & 0xFF]
        elif by:
            out.append(int(by, 0) & 0xFF)
    return out


# ---- emit ------------------------------------------------------------------


def fmt_bytes(b):
    lines = []
    for i in range(0, len(b), 16):
        chunk = ", ".join(f"0x{x:02x}" for x in b[i : i + 16])
        lines.append("    " + chunk + ",")
    return "\n".join(lines)


def cstr_bytes(s: str) -> str:
    # s is the C string literal contents (with C escapes); re-emit as a Rust
    # byte string literal with a NUL terminator.
    out = s.replace("\\\\", "\\")  # widl already escaped backslashes
    # Rust byte string: keep backslashes escaped
    esc = out.replace("\\", "\\\\").replace('"', '\\"')
    return f'b"{esc}\\0"'


def emit(iface, maj, minr, parsed, deco, out_path):
    (
        interface_id,
        transfer_syntax,
        flags,
        pcount,
        protseq,
        offset_table,
        sd,
        disp_entries,
        disp_count,
        routines,
        rundowns,
        expr_names,
        expr_funcs,
        proc_fmt,
        type_fmt,
    ) = parsed

    L = []
    A = L.append
    A("// @generated by tools/widl_to_rust.py — DO NOT EDIT.")
    A("// Faithful Rust transliteration of the widl/MIDL server stub for the")
    A(f"// `{iface}` interface (v{maj}.{minr}). Builds the same repr(C) static")
    A("// graph against `midl-rt`; the rpcrt4 NDR engine walks it at dispatch.")
    A("// (crate-level allow attrs live in the stub crate's lib.rs — this file is")
    A("//  include!'d, where inner attributes are not permitted.)")
    A("use core::ffi::c_void;")
    A("use midl_rt::*;")
    A("")

    # extern routines — declared `extern "C"` with the EXACT per-arch link name
    # read from the compiled C stub object (so i386 stdcall `@N`, including
    # by-value struct params, is verbatim ground truth). Each is only ever
    # address-taken into a table, never called, so a `fn()` signature suffices.
    externals = []
    externals += routines
    externals += rundowns
    seen = set(routines)
    for e in disp_entries:
        if e != "0" and e not in seen:
            seen.add(e)
            externals.append(e)
    externals += ["MIDL_user_allocate", "MIDL_user_free"]
    A('extern "C" {')
    for name in externals:
        ln = linkname(deco, name)
        if ln != name:
            A(f'    #[link_name = "{ln}"]')
        A(f"    fn {name}();")
    A("}")
    A("")

    # ExprEval functions
    for _, src in expr_funcs:
        A(src)
        A("")

    # ExprEval table
    if expr_names:
        items = ", ".join(f"{n} as *const c_void" for n in expr_names)
        A(f"static EXPR_EVAL_ROUTINES: PtrTable<{len(expr_names)}> = PtrTable([{items}]);")
    # Rundown table
    if rundowns:
        items = ", ".join(f"{n} as *const c_void" for n in rundowns)
        A(f"static RUNDOWN_ROUTINES: PtrTable<{len(rundowns)}> = PtrTable([{items}]);")
    A("")

    # protseq endpoints
    if protseq:
        rows = ", ".join(
            f"[{cstr_bytes(p)}.as_ptr(), {cstr_bytes(ep)}.as_ptr()]" for p, ep in protseq
        )
        A(f"static PROTSEQ: ProtseqEndpoints<{len(protseq)}> = ProtseqEndpoints([{rows}]);")
    A("")

    # format strings
    A(f"static PROC_FORMAT_STRING: MidlFormatString<{len(proc_fmt)}> = MidlFormatString {{")
    A("    pad: 0,")
    A("    format: [")
    A(fmt_bytes(proc_fmt))
    A("    ],")
    A("};")
    A(f"static TYPE_FORMAT_STRING: MidlFormatString<{len(type_fmt)}> = MidlFormatString {{")
    A("    pad: 0,")
    A("    format: [")
    A(fmt_bytes(type_fmt))
    A("    ],")
    A("};")
    A("")

    # format string offset table
    items = ", ".join(str(x) for x in offset_table)
    A(f"static FMT_OFFSET: [u16; {len(offset_table)}] = [{items}];")
    A("")

    # server routine table
    sr_items = ", ".join(f"{n} as *const c_void" for n in routines)
    A(f"static SERVER_ROUTINES: PtrTable<{len(routines)}> = PtrTable([{sr_items}]);")

    # dispatch function table (NdrServerCall2 ... + trailing null)
    d_items = ", ".join(
        "core::ptr::null()" if e == "0" else f"{e} as *const c_void" for e in disp_entries
    )
    A(f"static DISPATCH_FNS: PtrTable<{len(disp_entries)}> = PtrTable([{d_items}]);")
    A("")

    # RPC_DISPATCH_TABLE
    A("static DISPATCH_TABLE: RPC_DISPATCH_TABLE = RPC_DISPATCH_TABLE {")
    A(f"    dispatch_table_count: {disp_count},")
    A("    dispatch_table: DISPATCH_FNS.0.as_ptr() as *const c_void,")
    A("    reserved: 0,")
    A("};")
    A("")

    # RPC_SERVER_INTERFACE
    def guid_lit(g):
        d1, d2, d3, d4 = g
        d4s = ", ".join(f"0x{x:02x}" for x in d4)
        return f"GUID {{ data1: 0x{d1:08x}, data2: 0x{d2:04x}, data3: 0x{d3:04x}, data4: [{d4s}] }}"

    def sid_lit(sid):
        guid, ver = sid
        return (
            f"RPC_SYNTAX_IDENTIFIER {{ syntax_guid: {guid_lit(guid)}, "
            f"syntax_version: RPC_VERSION {{ major: {ver[0]}, minor: {ver[1]} }} }}"
        )

    A("static RPC_SERVER_INTERFACE_: RPC_SERVER_INTERFACE = RPC_SERVER_INTERFACE {")
    A("    length: core::mem::size_of::<RPC_SERVER_INTERFACE>() as u32,")
    A(f"    interface_id: {sid_lit(interface_id)},")
    A(f"    transfer_syntax: {sid_lit(transfer_syntax)},")
    A("    dispatch_table: &DISPATCH_TABLE as *const _ as *const c_void,")
    A(f"    rpc_protseq_endpoint_count: {pcount},")
    if protseq:
        A("    rpc_protseq_endpoint: &PROTSEQ as *const _ as *const c_void,")
    else:
        A("    rpc_protseq_endpoint: core::ptr::null(),")
    A("    default_manager_epv: core::ptr::null(),")
    A("    interpreter_info: &SERVER_INFO as *const _ as *const c_void,")
    A(f"    flags: {flags},")
    A("};")
    A("")

    # MIDL_STUB_DESC
    A("static STUB_DESC: MIDL_STUB_DESC = MIDL_STUB_DESC {")
    A("    rpc_interface_information: &RPC_SERVER_INTERFACE_ as *const _ as *const c_void,")
    A("    pfn_allocate: MIDL_user_allocate as *const c_void,")
    A("    pfn_free: MIDL_user_free as *const c_void,")
    A("    implicit_handle_info: core::ptr::null(),")
    if sd["has_rundown"] and rundowns:
        A("    apfn_ndr_rundown_routines: RUNDOWN_ROUTINES.0.as_ptr() as *const c_void,")
    else:
        A("    apfn_ndr_rundown_routines: core::ptr::null(),")
    A("    a_generic_binding_routine_pairs: core::ptr::null(),")
    if sd["has_expr"] and expr_names:
        A("    apfn_expr_eval: EXPR_EVAL_ROUTINES.0.as_ptr() as *const c_void,")
    else:
        A("    apfn_expr_eval: core::ptr::null(),")
    A("    a_xmit_quintuple: core::ptr::null(),")
    A("    p_format_types: TYPE_FORMAT_STRING.format.as_ptr() as *const c_void,")
    A(f"    f_check_bounds: {sd['f_check_bounds']},")
    A(f"    version: 0x{sd['version']:x},")
    A("    p_malloc_free_struct: core::ptr::null(),")
    A(f"    midl_version: 0x{sd['midl_version']:x},")
    A("    comm_fault_offsets: core::ptr::null(),")
    A("    a_user_marshal_quadruple: core::ptr::null(),")
    A("    notify_routine_table: core::ptr::null(),")
    A(f"    m_flags: {sd['m_flags']},")
    A("    cs_routine_tables: core::ptr::null(),")
    A("    proxy_server_info: core::ptr::null(),")
    A("    p_expr_info: core::ptr::null(),")
    A("};")
    A("")

    # MIDL_SERVER_INFO
    A("static SERVER_INFO: MIDL_SERVER_INFO = MIDL_SERVER_INFO {")
    A("    p_stub_desc: &STUB_DESC as *const _ as *const c_void,")
    A("    dispatch_table: SERVER_ROUTINES.0.as_ptr() as *const c_void,")
    A("    proc_string: PROC_FORMAT_STRING.format.as_ptr() as *const c_void,")
    A("    fmt_string_offset: FMT_OFFSET.as_ptr() as *const c_void,")
    A("    thunk_table: core::ptr::null(),")
    A("    p_transfer_syntax: core::ptr::null(),")
    A("    n_count: 0,")
    A("    p_syntax_info: core::ptr::null(),")
    A("};")
    A("")

    # exported ifspec handle
    A("#[no_mangle]")
    A(
        f"pub static {iface}_v{maj}_{minr}_s_ifspec: RpcIfHandle = "
        f"RpcIfHandle(&RPC_SERVER_INTERFACE_ as *const _ as *const c_void);"
    )
    A("")

    with open(out_path, "w") as f:
        f.write("\n".join(L))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("c_file")
    ap.add_argument(
        "--symobj", required=True, help="compiled <iface>_s.c object (for exact symbol decorations)"
    )
    ap.add_argument("--arch", required=True, choices=["i386", "amd64", "arm64"])
    ap.add_argument("--nm", default="nm", help="nm binary")
    ap.add_argument("-o", "--out", required=True)
    args = ap.parse_args()

    c = Path(args.c_file).read_text()
    # Companion <iface>_s.h — needed only for the named-type ExprEval idiom, to
    # synthesize a layout-faithful view of the conformant type it reads.
    hdr = Path(args.c_file).with_suffix(".h")
    header = hdr.read_text() if hdr.exists() else ""
    deco = build_decoration_map(args.symobj, args.nm, args.arch)

    iface, maj, minr = parse_iface(c)
    interface_id, transfer_syntax, flags, pcount = parse_server_interface(c, iface)
    protseq = parse_protseq(c, iface)
    offset_table = parse_u16_array(c, iface + "_FormatStringOffsetTable")
    sd = parse_stub_desc(c, iface)
    disp_entries, disp_count = parse_dispatch(c, iface)
    routines = parse_server_routines(c, iface)
    rundowns = parse_named_array(c, r"NDR_RUNDOWN\s+RundownRoutines\s*\[\s*\]\s*=") or []
    expr_names = parse_named_array(c, r"EXPR_EVAL\s+ExprEvalRoutines\s*\[\s*\]\s*=") or []
    expr_funcs = parse_expr_evals(c, header)
    proc_fmt = parse_format_string(c, "MIDL_PROC_FORMAT_STRING", "__MIDL_ProcFormatString")
    type_fmt = parse_format_string(c, "MIDL_TYPE_FORMAT_STRING", "__MIDL_TypeFormatString")

    if disp_count is None:
        disp_count = len(routines)

    parsed = (
        interface_id,
        transfer_syntax,
        flags,
        pcount,
        protseq,
        offset_table,
        sd,
        disp_entries,
        disp_count,
        routines,
        rundowns,
        expr_names,
        expr_funcs,
        proc_fmt,
        type_fmt,
    )
    emit(iface, maj, minr, parsed, deco, args.out)
    sys.stderr.write(
        f"widl_to_rust: {iface} v{maj}.{minr} → {args.out} "
        f"({len(routines)} routines, proc={len(proc_fmt)}B type={len(type_fmt)}B)\n"
    )


if __name__ == "__main__":
    main()
