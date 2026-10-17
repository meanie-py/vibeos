"""c_data_port — a C DATA translation unit as Rust statics, deterministically, for $0.

Some C is not code at all. usetup's MUI language tables are ~70,000 lines across 30
`lang/*.h` headers of `MUI_ENTRY { X, Y, "text", flags, id }` initializers; diskpart's and
many drivers' are smaller copies of the same shape. Buying them from a model is paying to
retype data and inviting it to "fix" a string, and c2rust renders a narrow literal as a
`[u8; N]` array per string -- several times the C, and unreadable. So usetup's tables were
left in C (`mui_tables.c`) as an owed item. This is the tool that pays it.

Everything that decides a byte comes from clang, run with the TU's own compile command:
which declarations exist (so `#ifdef LANGUAGE_*` and every other guard is the build's),
their types, struct field order, array lengths, and the evaluated value of every
initializer. The SOURCE contributes only spelling, so the Rust reads like the C:

  * a string is spelled from its source tokens -- the literal pieces, and a string macro
    such as `KERNEL_VERSION_STR` kept BY NAME (its value is generated per build and per
    arch: "0.4.17-x86-dev"), concatenated at compile time by `cstr_cat!`. The spelled
    bytes are then checked against clang's evaluated literal, byte for byte. The lang
    headers are in five encodings (UTF-8, ISO-8859, raw OEM bytes); a narrow C literal
    copies source bytes verbatim, and so does this.
  * an integer keeps its spelling (`TEXT_STYLE_NORMAL | TEXT_ALIGN_CENTER`) when every
    name in it is a constant of the crate's own island and the expression evaluates, from
    the island's values, to what clang evaluated. Otherwise the clang value is emitted,
    with the spelling as a comment: correct first, readable when it can be proven.

Output is generated once per built arch and must be byte-identical across them -- data
whose layout differs per arch is reported, not guessed. Each non-`static` C object is a
`#[no_mangle] pub static mut` (C linkage, the C's mutability), and its length is checked
against the crate island's `extern` declaration of it, which is how the crate's code
already reaches it. A C `static` stays module-private in spirit (`pub(super)`), visible to
the sibling tables that point at it.

Also handled (2026-10-08): designated initializers (clang's JSON is the SEMANTIC form,
already in field order with implicit zeros); unions, through the member clang says the
initializer names, and anonymous struct/union members as bindgen's `__bindgen_anon_N` of type
`Parent__bindgen_ty_N`; bit-fields, packed into `__BindgenBitfieldUnit::new([..])` from the
offsets the island's own bindgen accessors state; wide literals as `wstr!("...")`, checked
unit for unit; and `char`/`WCHAR` arrays initialized by a string. Not handled, loudly: an
anonymous member initialized as a union's active member, a bit-field struct the island does
not define, and any initializer kind not listed in `_render`. Each is a `DataPortError`
naming the declaration, never a silent zero.
"""

from __future__ import annotations

import json
import re
import subprocess
from dataclasses import dataclass, field, replace
from pathlib import Path

from . import macro_authority as ma


class DataPortError(Exception):
    pass


ARCHES = ("i386", "amd64", "arm64")

# C scalar types a field can be spelled with, when it is not a typedef the island has.
_C_SCALARS = {
    "char": "core::ffi::c_char",
    "signed char": "i8",
    "unsigned char": "u8",
    "short": "i16",
    "unsigned short": "u16",
    "int": "i32",
    "unsigned int": "u32",
    "long": "i32",  # LLP64: every target here is Windows
    "unsigned long": "u32",
    "long long": "i64",
    "unsigned long long": "u64",
}


# --------------------------------------------------------------------------------------
# clang
# --------------------------------------------------------------------------------------


def clang_ast(repo: Path, arch: str, tu: Path) -> dict:
    """The TU's AST as clang's JSON, from its real compile command on `arch`."""
    argv = ma.compile_argv(repo, arch, tu)
    if argv is None:
        raise DataPortError(
            f"no compile command for {tu} on {arch}\nfix: ./vibeos-cli build --arch {arch}"
        )
    proc = subprocess.run(
        [*ma._without_outputs(argv), "-fsyntax-only", "-Xclang", "-ast-dump=json"],
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        raise DataPortError(f"clang failed on {tu.name} ({arch}):\n{proc.stderr.decode()[:800]}")
    ast = json.loads(proc.stdout)
    _attach_files(ast)
    return ast


def _attach_files(node: object) -> None:
    """Give every source location its `_file`. clang's JSON prints a location's file only
    when it differs from the previous location PRINTED, so it is recovered by walking the
    dump in print order (dict order is print order)."""
    cur: list[str | None] = [None]

    def walk(n: object) -> None:
        if isinstance(n, dict):
            if "offset" in n:
                if "file" in n:
                    cur[0] = n["file"]
                n["_file"] = cur[0]
            for k, v in n.items():
                if k != "includedFrom":
                    walk(v)
        elif isinstance(n, list):
            for v in n:
                walk(v)

    walk(node)


def _loc(loc: dict) -> dict:
    """The expansion location: where the text sits in the file the reader sees."""
    return loc.get("expansionLoc", loc)


def _decl_file(decl: dict) -> str | None:
    return _loc(decl.get("loc", {})).get("_file")


def enum_constants(ast: dict) -> dict[str, tuple[int | None, str]]:
    """EnumConstantDecl id -> (value, the enum's tag). An explicit initializer carries its
    evaluated value; an implicit one is the previous constant + 1.

    A value clang did not evaluate is None, and so is every implicit successor of it. That is
    an error only where a table USES the constant (`_int_value`). Failing on the whole include
    closure refused cdrom's data.c and guid.c over iotypes.h's `ContinueCompletion =
    STATUS_CONTINUE_COMPLETION`, a cast that clang leaves unevaluated and neither file reads.
    """
    out: dict[str, tuple[int | None, str]] = {}
    for d in ast.get("inner", []):
        if d.get("kind") != "EnumDecl":
            continue
        nxt: int | None = 0
        for c in d.get("inner", []):
            if c.get("kind") != "EnumConstantDecl":
                continue
            exprs = [i for i in c.get("inner", []) if not i.get("kind", "").endswith("Comment")]
            vals = [i["value"] for i in exprs if "value" in i]
            v = int(vals[0]) if vals else (None if exprs else nxt)
            out[c["id"]] = (v, d.get("name", ""))
            out[c["name"]] = (v, d.get("name", ""))
            nxt = None if v is None else v + 1
    return out


@dataclass
class CField:
    name: str | None  # None: an anonymous struct/union member
    qual: str
    bits: int | None = None  # a bit-field's width


@dataclass
class Rec:
    kind: str  # "struct" | "union"
    fields: list[CField]
    rust: str | None = None  # bindgen's name, for an anonymous record (`A__bindgen_ty_1`)


def records(ast: dict) -> dict[str, Rec]:
    """Every struct and union by the names an initializer's qualType can use: its typedef,
    `struct X`/`union X`, and for an anonymous member's record the qualType clang gives it
    (`union A::(anonymous at f.c:4:28)`), together with bindgen's name for that record
    (`A__bindgen_ty_1`: the parent's name, then the anonymous records in declaration order)."""
    by_id: dict[str, dict] = {}
    names: dict[str, str] = {}  # record id -> bindgen's name, for top-level records
    aliases: dict[str, str] = {}  # typedef name -> record id
    for d in ast.get("inner", []):
        if d.get("kind") == "RecordDecl" and d.get("completeDefinition"):
            by_id[d["id"]] = d
            if d.get("name"):
                names.setdefault(d["id"], d["name"])
        elif d.get("kind") == "TypedefDecl":
            inner = json.dumps(d.get("inner", []))
            owned = re.search(r'"ownedTagDecl":\s*\{"id":\s*"(0x[0-9a-f]+)"', inner)
            if owned and owned.group(1) not in names:
                names[owned.group(1)] = d["name"]  # an anonymous struct: bindgen names it so
            ids = re.findall(r'"decl":\s*\{"id":\s*"(0x[0-9a-f]+)"', inner)
            if ids:
                aliases[d["name"]] = ids[0]
    out: dict[str, Rec] = {}

    def walk(rec: dict, rust: str) -> Rec:
        fields: list[CField] = []
        nested = 0
        last: Rec | None = None  # the anonymous record the next field's type names
        for f in rec.get("inner", []):
            k = f.get("kind")
            if k == "RecordDecl" and f.get("completeDefinition") and not f.get("name"):
                nested += 1
                last = walk(f, f"{rust}__bindgen_ty_{nested}")
                last.rust = f"{rust}__bindgen_ty_{nested}"
            elif k == "FieldDecl":
                q = f["type"]["qualType"]
                if last is not None and ("(anonymous" in q or "(unnamed" in q):
                    out[q] = last  # clang spells the member's type this way, exactly
                bits = (
                    int(f["inner"][0]["value"]) if f.get("isBitfield") and f.get("inner") else None
                )
                fields.append(CField(f.get("name") or None, q, bits))
        return Rec(rec.get("tagUsed", "struct"), fields)

    for rid, d in by_id.items():
        rust = names.get(rid)
        if rust:
            r = walk(d, rust)
            out[rust] = r
            out[f"{d.get('tagUsed', 'struct')} {rust}"] = r
    for alias, rid in aliases.items():
        rust = names.get(rid)
        if rust in out:
            out.setdefault(alias, out[rust])
    return out


# --------------------------------------------------------------------------------------
# C literals
# --------------------------------------------------------------------------------------

_SIMPLE_ESC = {
    "n": 10, "t": 9, "r": 13, "a": 7, "b": 8, "f": 12, "v": 11,
    "\\": 92, "'": 39, '"': 34, "?": 63, "0": 0,
}  # fmt: skip


def c_unescape(body: bytes) -> bytes:
    """The bytes a narrow C string literal's body (between the quotes) denotes."""
    out = bytearray()
    i = 0
    while i < len(body):
        c = body[i]
        if c != 0x5C:  # backslash
            out.append(c)
            i += 1
            continue
        nxt = chr(body[i + 1])
        if nxt == "x":
            j = i + 2
            while j < len(body) and chr(body[j]) in "0123456789abcdefABCDEF":
                j += 1
            out.append(int(body[i + 2 : j], 16) & 0xFF)
            i = j
        elif nxt in "01234567":
            j = i + 1
            while j < len(body) and j < i + 4 and chr(body[j]) in "01234567":
                j += 1
            out.append(int(body[i + 1 : j], 8) & 0xFF)
            i = j
        elif nxt in _SIMPLE_ESC:
            out.append(_SIMPLE_ESC[nxt])
            i += 2
        else:
            raise DataPortError(f"unknown escape \\{nxt} in a C literal")
    return bytes(out)


def _split_printed(body: str) -> list[str]:
    """clang prints a string whose escape could swallow the next character as TWO adjacent
    literals (`L"\\x2DD""DOPPELAKUT"`): the pieces of a printed value, split at every
    UNESCAPED quote. Decoding the joined text instead lets the hex escape eat the `D`s."""
    pieces, cur, i = [], [], 0
    while i < len(body):
        c = body[i]
        if c == "\\":
            cur.append(body[i : i + 2])
            i += 2
        elif c == '"':
            pieces.append("".join(cur))
            cur = []
            i += 1
        else:
            cur.append(c)
            i += 1
    pieces.append("".join(cur))
    return pieces


def _clang_string_value(node: dict) -> bytes:
    """clang's evaluated value of a narrow StringLiteral (printed with C escapes)."""
    v = node["value"]
    if not (v.startswith('"') and v.endswith('"')):
        raise DataPortError(f"prefixed string literal where a narrow one was expected: {v[:40]}")
    return b"".join(c_unescape(p.encode("latin-1")) for p in _split_printed(v[1:-1]))


def wide_units(body: str) -> list[int]:
    """The code units a wide literal's body denotes, as clang prints or a source spells it:
    an escape is one unit (`\\351`, `\\x263A`), `\\u`/`\\U` a code point, text its UTF-16."""
    out: list[int] = []
    i = 0
    while i < len(body):
        c = body[i]
        if c != "\\":
            out += _utf16(ord(c))
            i += 1
            continue
        e = body[i + 1]
        if e == "x":
            j = i + 2
            while j < len(body) and body[j] in "0123456789abcdefABCDEF":
                j += 1
            out.append(int(body[i + 2 : j], 16) & 0xFFFF)
            i = j
        elif e in "01234567":
            j = i + 1
            while j < len(body) and j < i + 4 and body[j] in "01234567":
                j += 1
            out.append(int(body[i + 1 : j], 8))
            i = j
        elif e in "uU":
            n = 4 if e == "u" else 8
            out += _utf16(int(body[i + 2 : i + 2 + n], 16))
            i += 2 + n
        elif e in _SIMPLE_ESC:
            out.append(_SIMPLE_ESC[e])
            i += 2
        else:
            raise DataPortError(f"unknown escape \\{e} in a C wide literal")
    return out


def _utf16(cp: int) -> list[int]:
    if cp < 0x10000:
        return [cp]
    cp -= 0x10000
    return [0xD800 | (cp >> 10), 0xDC00 | (cp & 0x3FF)]


def _rust_str(units: list[int]) -> str | None:
    """The Rust `"..."` whose UTF-16 is `units`, or None when they are not valid UTF-16 (an
    unpaired surrogate): printable ASCII as itself, everything else `\\u{..}`."""
    try:
        text = bytes(b for u in units for b in u.to_bytes(2, "little")).decode("utf-16-le")
    except UnicodeDecodeError:
        return None
    out = []
    for ch in text:
        if ch in '"\\':
            out.append("\\" + ch)
        elif 0x20 <= ord(ch) < 0x7F:
            out.append(ch)
        else:
            out.append(f"\\u{{{ord(ch):x}}}")
    return '"' + "".join(out) + '"'


_STR_TOKEN = re.compile(rb'"(?:[^"\\\n]|\\.)*"')
_TOKEN = re.compile(rb'\s+|/\*.*?\*/|//[^\n]*|"(?:[^"\\\n]|\\.)*"|[A-Za-z_]\w*|.', re.S)


def rust_byte_string(data: bytes, *, prefix: str) -> str:
    """`data` as a Rust `c"..."` / `b"..."` literal: printable ASCII as itself, the rest
    as `\\xNN` -- byte-exact for any encoding."""
    out = []
    for b in data:
        if b == 0x22:
            out.append('\\"')
        elif b == 0x5C:
            out.append("\\\\")
        elif 0x20 <= b < 0x7F:
            out.append(chr(b))
        else:
            out.append(f"\\x{b:02x}")
    return f'{prefix}"{"".join(out)}"'


def rust_literal_bytes(lit: str) -> bytes:
    """What a Rust `c"..."`/`b"..."` literal emitted here denotes (without the NUL)."""
    body = lit[2:-1]
    out = bytearray()
    i = 0
    while i < len(body):
        if body[i] == "\\":
            e = body[i + 1]
            if e == "x":
                out.append(int(body[i + 2 : i + 4], 16))
                i += 4
                continue
            out.append({"\\": 92, '"': 34, "n": 10, "t": 9, "r": 13, "0": 0}[e])
            i += 2
        else:
            out += body[i].encode("utf-8")
            i += 1
    return bytes(out)


def spell_literal(body: bytes, *, utf8: bool, prefix: str) -> str:
    """A C narrow literal's body (between its quotes) as a Rust literal, keeping the
    author's spelling: a C escape stays an escape (`\\x07`, `\\304` -> `\\xc4`), and raw
    text stays text when the file is UTF-8 -- the Rust literal is then the same bytes. A
    byte-string (`b`) piece must be ASCII, so its raw text is escaped too. Spelling from
    the BYTES instead read cs-CZ's escaped cp852 `\\xd8\\x9f` as the UTF-8 for an Arabic
    question mark: the right bytes, displayed as the wrong text."""
    out = []
    i = 0
    while i < len(body):
        c = body[i]
        if c == 0x5C:  # an escape: decode it, then spell the byte it denotes
            j = i + 2
            if chr(body[i + 1]) == "x":
                while j < len(body) and chr(body[j]) in "0123456789abcdefABCDEF":
                    j += 1
            elif chr(body[i + 1]) in "01234567":
                j = i + 1
                while j < len(body) and j < i + 4 and chr(body[j]) in "01234567":
                    j += 1
            out.append(rust_byte_string(c_unescape(body[i:j]), prefix="")[1:-1])
            i = j
        elif c < 0x80:
            out.append(rust_byte_string(bytes([c]), prefix="")[1:-1])
            i += 1
        else:
            n = 2 if c >> 5 == 0b110 else 3 if c >> 4 == 0b1110 else 4 if c >> 3 == 0b11110 else 1
            chunk = body[i : i + n]
            try:
                text = chunk.decode("utf-8") if utf8 and prefix == "c" else None
            except UnicodeDecodeError:
                text = None
            out.append(text if text is not None else rust_byte_string(chunk, prefix="")[1:-1])
            i += len(chunk)
    return f'{prefix}"{"".join(out)}"'


# --------------------------------------------------------------------------------------
# rendering
# --------------------------------------------------------------------------------------


@dataclass
class Ctx:
    """What rendering one TU needs to know beyond the node in hand."""

    sources: dict[str, bytes]
    records: dict[str, Rec]
    island_consts: dict[str, int]
    string_macros: dict[str, bytes]
    utf8_files: set[str]
    refs: set[str] = field(default_factory=set)  # every object the initializer points at
    where: str = ""
    island: str = ""  # the crate island's text: bindgen's spelling of bit-field units
    helpers: set[str] = field(default_factory=set)  # const helpers the output needs
    flex: dict[str, str] = field(default_factory=dict)  # flexible-array struct name -> its text
    layouts: dict[str, dict] = field(default_factory=dict)  # clang's record layouts (bytes)

    def text(self, node: dict) -> tuple[str | None, bytes]:
        r = node.get("range", {})
        b, e = _loc(r.get("begin", {})), _loc(r.get("end", {}))
        f = b.get("_file")
        if f is None or f != e.get("_file") or "offset" not in b or "offset" not in e:
            return None, b""
        src = self.sources.setdefault(f, Path(f).read_bytes())
        return f, src[b["offset"] : e["offset"] + e.get("tokLen", 0)]


def rust_type(qual: str) -> str:
    """A C qualType as the Rust type the island spells it with."""
    q = qual.strip()
    m = re.fullmatch(r"(const\s+)?(.+?)\s*\*\s*(const)?", q)
    if m:
        return f"*{'const' if m.group(1) else 'mut'} {rust_type(m.group(2))}"
    q = re.sub(r"^(?:const|volatile)\s+", "", q)
    q = re.sub(r"^struct\s+", "", q)
    return _C_SCALARS.get(q, q)


def _strip(node: dict) -> tuple[dict, set[str]]:
    """Peel implicit casts and parens; return the inner node and the cast kinds seen."""
    kinds: set[str] = set()
    while node.get("kind") in ("ImplicitCastExpr", "ParenExpr", "ConstantExpr", "CStyleCastExpr"):
        if node.get("castKind"):
            kinds.add(node["castKind"])
        if node.get("kind") == "CStyleCastExpr":
            kinds.add("CStyle")
        node = node["inner"][0]
    return node, kinds


_ENUMS: dict[str, tuple[int | None, str]] = {}
_TYPEDEFS: dict[str, str] = {}


def typedefs(ast: dict) -> dict[str, str]:
    """typedef name -> the C type it names (one level; `desugar` follows the chain)."""
    return {
        d["name"]: d["type"]["qualType"]
        for d in ast.get("inner", [])
        if d.get("kind") == "TypedefDecl" and "type" in d
    }


def desugar(qual: str) -> str:
    """`PCSTR` -> `const CHAR *` -> `const char *`: through the TU's typedefs."""
    q = qual.strip()
    for _ in range(16):
        m = re.fullmatch(r"((?:const\s+|volatile\s+)*)([A-Za-z_]\w*)(.*)", q)
        if not m or m.group(2) not in _TYPEDEFS:
            return q
        q = f"{m.group(1)}{_TYPEDEFS[m.group(2)]}{m.group(3)}".strip()
    return q


_INT_BITS = {
    "char": (8, True), "signed char": (8, True), "unsigned char": (8, False),
    "short": (16, True), "unsigned short": (16, False), "int": (32, True),
    "unsigned int": (32, False), "long": (32, True), "unsigned long": (32, False),
    "long long": (64, True), "unsigned long long": (64, False), "_Bool": (8, False),
}  # fmt: skip

# The arch under translation (pointer width) and the clang record layouts of the TU, for
# `sizeof`: set by `tables`, read by `c_sizeof`.
_ARCH: dict = {"ptr": 4, "layouts": None, "ctx": None}


def _int_type(t: dict) -> tuple[int, bool] | None:
    """(bits, signed) of an integer type node, through typedefs; None for anything else."""
    for q in (t.get("desugaredQualType"), t.get("qualType")):
        if q:
            d = desugar(q).replace("const ", "").strip()
            if d in _INT_BITS:
                return _INT_BITS[d]
    return None


def _wrap(v: int, bits: int, signed: bool) -> int:
    v &= (1 << bits) - 1
    return v - (1 << bits) if signed and v >> (bits - 1) else v


def c_sizeof(qual: str) -> int:
    """sizeof a C type, as clang laid it out: records from the TU's own record-layout dump,
    arrays by their length, pointers by the arch; anything else is an error, not a guess."""
    q = desugar(qual.strip())
    arr = re.fullmatch(r"(.+?)\s*\[(\d+)\]", q)
    if arr:
        return int(arr.group(2)) * c_sizeof(arr.group(1))
    if q.endswith("*"):
        return _ARCH["ptr"]
    base = re.sub(r"^(?:const|volatile)\s+", "", q)
    if base in _INT_BITS:
        return _INT_BITS[base][0] // 8
    lay = _ARCH["layouts"]
    if lay is not None and base in lay:
        return lay[base]["size"]
    raise DataPortError(f"sizeof({qual}): no clang layout for that type")


_LAYOUT_BLOCK = re.compile(
    r"Type: ((?:struct|union) [^\n]+?)\s*\n\s*Layout: <ASTRecordLayout\s+Size:(\d+)"
    r"\s+DataSize:\d+\s+Alignment:(\d+)\s+FieldOffsets: \[([\d, ]*)\]"
)


def record_layouts(argv: list[str]) -> dict[str, dict]:
    """`struct _X` -> {size, align, offsets} in BYTES (bit-field offsets stay in bits via
    `bit_offsets`), from clang's `-fdump-record-layouts-simple` of the TU's own command."""
    proc = subprocess.run(
        [
            *ma._without_outputs(argv),
            "-fsyntax-only",
            "-Xclang",
            "-fdump-record-layouts-complete",
            "-Xclang",
            "-fdump-record-layouts-simple",
        ],
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        raise DataPortError(f"clang record-layout dump failed:\n{proc.stderr.decode()[:600]}")
    out: dict[str, dict] = {}
    for m in _LAYOUT_BLOCK.finditer(proc.stdout.decode(errors="replace")):
        bits = [int(x) for x in m.group(4).split(",") if x.strip()]
        out[m.group(1)] = {
            "size": int(m.group(2)) // 8,
            "align": int(m.group(3)) // 8,
            "bit_offsets": bits,
        }
    return out


def _call_text(n: dict, ctx: Ctx) -> str:
    """The source text of the call a node begins: its name and the balanced parentheses after
    it. A function-like macro's expansion range ends at the macro NAME, not its `)`."""
    b = _loc(n.get("range", {}).get("begin", {}))
    f = b.get("_file")
    if f is None or "offset" not in b:
        raise DataPortError(f"{ctx.where}: an expression with no source location")
    src = ctx.sources.setdefault(f, Path(f).read_bytes())
    i = b["offset"]
    m = re.compile(rb"[A-Za-z_]\w*\s*\(").match(src, i)
    if m is None:
        raise DataPortError(f"{ctx.where}: expected a call at {Path(f).name}:{b.get('line', '?')}")
    depth, j = 1, m.end()
    while j < len(src) and depth:
        depth += {40: 1, 41: -1}.get(src[j], 0)
        j += 1
    return " ".join(src[i:j].decode("latin-1").split())


def _anon_layout(lay: dict, key: str) -> dict | None:
    """The layout of an anonymous member's record: the AST calls it `struct (unnamed struct
    at F:L:C)`, the layout dump `struct _S::(unnamed at F:L:C)` (or `anonymous`)."""
    m = re.fullmatch(r"(struct|union) \((?:unnamed|anonymous) (?:struct |union )?at (.+)\)", key)
    if m is None:
        return None
    for k, v in lay.items():
        if k.startswith(m.group(1) + " ") and k.endswith(f"at {m.group(2)})") and "::(" in k:
            return v
    return None


def _offsetof(n: dict) -> int:
    """`offsetof(T, a.b)` / `FIELD_OFFSET(T, a.b)`: clang's JSON prints an OffsetOfExpr with
    no value and no operands, so the TYPE and MEMBER PATH are read from the call's source text
    and the number from clang's own record layout of that type (bit-fields are refused)."""
    ctx: Ctx | None = _ARCH["ctx"]
    lay = _ARCH["layouts"]
    if ctx is None or lay is None:
        raise DataPortError("offsetof outside a TU render: no clang layouts to read")
    text = _call_text(n, ctx)
    m = re.search(r"\(\s*(.+?)\s*,\s*([A-Za-z_]\w*(?:\s*\.\s*[A-Za-z_]\w*)*)\s*\)\s*\)?$", text)
    if m is None:
        raise DataPortError(f"{ctx.where}: cannot read the type and member of `{text[:80]}`")
    qual = m.group(1)
    total = 0
    for member in re.split(r"\s*\.\s*", m.group(2)):
        key = desugar(qual)
        rec = ctx.records.get(qual) or ctx.records.get(key)
        info = lay.get(key) or _anon_layout(lay, key)
        if rec is None or info is None:
            raise DataPortError(f"{ctx.where}: offsetof into {qual!r}: no record layout")
        names = [f.name for f in rec.fields]
        if member not in names or len(info["bit_offsets"]) != len(names):
            raise DataPortError(f"{ctx.where}: offsetof({qual}, {member}): not a laid-out field")
        i = names.index(member)
        if rec.fields[i].bits is not None or info["bit_offsets"][i] % 8:
            raise DataPortError(f"{ctx.where}: offsetof({qual}, {member}) is a bit-field")
        total += info["bit_offsets"][i] // 8
        qual = rec.fields[i].qual
    return total


def clang_int(node: dict, root: bool = True) -> int:
    """clang's integer value of a constant expression tree. An integral cast WRAPS to its
    type (`(WORD)0x12345` is 0x2345, as MAKELONG's pieces rely on), except an implicit one
    at the root: that is the field's own conversion, which `_render` states with `as`."""
    k0 = node.get("kind")
    if k0 in ("ImplicitCastExpr", "CStyleCastExpr") and node.get("castKind") == "IntegralCast":
        it = _int_type(node["type"])
        if it is not None and not (root and k0 == "ImplicitCastExpr"):
            return _wrap(clang_int(node["inner"][0], False), *it)
    if k0 in ("ParenExpr", "ConstantExpr", "ImplicitCastExpr", "CStyleCastExpr"):
        return clang_int(node["inner"][0], root and k0 != "CStyleCastExpr")
    n = node
    k = n.get("kind")
    if k == "UnaryExprOrTypeTraitExpr" and n.get("name") == "sizeof":
        t = n.get("argType") or n["inner"][0]["type"]
        return c_sizeof(t.get("desugaredQualType") or t["qualType"])
    if k == "OffsetOfExpr":
        return _offsetof(n)
    if k in ("IntegerLiteral", "CharacterLiteral"):
        return int(n["value"])
    if k == "ImplicitValueInitExpr":  # a member a designated initializer left out: C's zero
        return 0
    if k == "UnaryOperator":
        v = clang_int(n["inner"][0], False)
        return {"-": -v, "~": ~v, "+": v, "!": int(not v)}[n["opcode"]]
    if k == "BinaryOperator":
        a, b = clang_int(n["inner"][0], False), clang_int(n["inner"][1], False)
        if n["opcode"] in ("/", "%"):
            if b == 0:
                raise DataPortError("division by zero in a constant initializer")
            q = abs(a) // abs(b) * (-1 if (a < 0) != (b < 0) else 1)  # C truncates toward 0
            return q if n["opcode"] == "/" else a - q * b
        ops = {
            "|": a | b, "&": a & b, "^": a ^ b, "+": a + b, "-": a - b, "*": a * b,
            "<<": a << b, ">>": a >> b,
        }  # fmt: skip
        if n["opcode"] not in ops:
            raise DataPortError(f"operator {n['opcode']} in a constant initializer")
        return ops[n["opcode"]]
    if k == "DeclRefExpr" and n.get("referencedDecl", {}).get("kind") == "EnumConstantDecl":
        ref = n["referencedDecl"]
        if ref["id"] not in _ENUMS:
            raise DataPortError(f"enum constant {ref.get('name')}: its EnumDecl is not top-level")
        value = _ENUMS[ref["id"]][0]
        if value is None:
            raise DataPortError(f"enum constant {ref.get('name')}: no evaluated value in the AST")
        return value
    raise DataPortError(f"{k} is not an integer constant")


_SPELLED_INT = re.compile(r"[\sA-Za-z0-9_|()+\-~<>&^x]+")


def _eval_spelling(spelling: str, consts: dict[str, int]) -> int | None:
    names = re.findall(r"\b[A-Za-z_]\w*\b", spelling)
    if any(n not in consts for n in names if not re.fullmatch(r"0[xX][0-9a-fA-F]+|\d+[uUlL]*", n)):
        return None
    expr = re.sub(r"\b(0[xX][0-9a-fA-F]+|\d+)[uUlL]+\b", r"\1", spelling)
    expr = re.sub(r"\b[A-Za-z_]\w*\b", lambda m: str(consts.get(m.group(0), m.group(0))), expr)
    try:
        return int(eval(expr, {"__builtins__": {}}, {}))
    except Exception:
        return None


def _render(node: dict, qual: str, ctx: Ctx, indent: str) -> str:
    kind = node.get("kind")
    if kind == "InitListExpr":
        return _render_list(node, qual, ctx, indent)
    if kind == "ImplicitValueInitExpr":
        return _zero(qual)
    inner, casts = _strip(node)
    ik = inner.get("kind")
    if "NullToPointer" in casts:
        return _zero(qual)
    if ik == "StringLiteral":
        if re.fullmatch(r".+?\s*\[\d+\]", qual.strip()):
            return _render_string_array(inner, qual, ctx)  # `char s[8] = "abc"`
        if inner["value"].startswith(('L"', 'u"')):
            return _render_wide(inner, qual, ctx)
        return _render_string(inner, qual, ctx)
    if ik in ("DeclRefExpr", "UnaryOperator") and (
        "ArrayToPointerDecay" in casts or inner.get("opcode") == "&"
    ):
        ref = inner if ik == "DeclRefExpr" else _strip(inner["inner"][0])[0]
        name = ref.get("referencedDecl", {}).get("name")
        if ref.get("kind") != "DeclRefExpr" or not name:
            raise DataPortError(f"{ctx.where}: address of something other than a named object")
        ctx.refs.add(name)
        return f"&raw const {name} as {rust_type(qual)}"
    value = clang_int(node)
    if ik == "UnaryExprOrTypeTraitExpr" and inner.get("name") == "sizeof":
        # `sizeof(x[0])` of a record: the island's own type, whose layout bindgen states, so
        # the value follows the arch; clang's number is checked against it by the cargo
        # build's offset/size assertions of the island, and stated here.
        t = inner.get("argType") or inner["inner"][0]["type"]
        tq = t["qualType"].strip()
        d = desugar(t.get("desugaredQualType") or tq)
        if re.fullmatch(r"[A-Za-z_]\w*", tq) and d.startswith(("struct ", "union ")):
            return f"core::mem::size_of::<{tq}>() as {rust_type(qual)} /* {value} */"
    _, raw = ctx.text(node)
    spelling = " ".join(raw.decode("latin-1").split())
    rt = rust_type(qual)
    if re.fullmatch(r"-?\d+", spelling) and int(spelling) == value:
        return (
            spelling if rt not in ("u8", "u16", "u32", "u64") or value >= 0 else f"{value} as {rt}"
        )
    # bindgen names an enum's constants `<tag>_<NAME>` (`_PAGE_NUMBER_SETUP_INIT_PAGE`).
    if re.fullmatch(r"[A-Za-z_]\w*", spelling) and spelling not in ctx.island_consts:
        tagged = f"{_ENUMS.get(spelling, (0, ''))[1]}_{spelling}"
        if spelling in _ENUMS and ctx.island_consts.get(tagged) == value:
            spelling = tagged
    if spelling and _SPELLED_INT.fullmatch(spelling):
        ev = _eval_spelling(spelling, ctx.island_consts)
        width = 64
        if ev is not None and (ev - value) % (1 << width) == 0:
            # Rust has no `0X1F`, `5UL`: drop the C-only spellings (the value was checked).
            rs = re.sub(r"\b(0[xX][0-9a-fA-F]+|\d+)[uUlL]+\b", r"\1", spelling)
            rs = re.sub(r"\b0X", "0x", rs)
            return f"{rs} as {rt}" if re.fullmatch(r"\w+", rs) else f"({rs}) as {rt}"
    if re.fullmatch(r"[A-Za-z_]\w*", spelling):  # a function-like macro's range stops at its name
        try:
            spelling = _call_text(inner if ik == "OffsetOfExpr" else node, ctx)
        except DataPortError:
            pass
    if "/*" in spelling or "*/" in spelling or len(spelling) > 72:
        spelling = ""  # a macro call spanning comments or a whole table row is no label
    note = f" /* {spelling} */" if spelling else ""
    return f"{value} as {rt}{note}"


def _zero(qual: str) -> str:
    """C's zero of `qual` -- a NULL, a 0 -- spelled for the field's own type: `PCSTR` is a
    `*const` and wants `null()`, which only the typedef chain says."""
    real = rust_type(desugar(qual))
    if real.startswith("*const"):
        return "core::ptr::null()"
    if real.startswith("*mut"):
        return "core::ptr::null_mut()"
    if real in _C_SCALARS.values() or re.fullmatch(r"[iu](8|16|32|64)|core::ffi::c_char", real):
        return "0"
    return "unsafe { core::mem::zeroed() }"


def _render_string(lit: dict, qual: str, ctx: Ctx) -> str:
    want = _clang_string_value(lit)
    if want.endswith(b"\0"):
        want = want[:-1]  # clang's value is the array, NUL included only if written
    f, raw = ctx.text(lit)
    rt = rust_type(qual)
    parts: list[tuple[str, bytes]] = []  # ("lit", raw token body) | ("macro", name)
    if f is not None:
        for m in _TOKEN.finditer(raw):
            tok = m.group(0)
            if tok.isspace() or tok.startswith((b"/*", b"//")):
                continue
            if tok.startswith(b'"'):
                parts.append(("lit", tok[1:-1]))
            elif re.fullmatch(rb"[A-Za-z_]\w*", tok) and tok.decode() in ctx.string_macros:
                parts.append(("macro", tok))
            else:
                parts = []
                break
    spelled = b"".join(
        c_unescape(p) if k == "lit" else ctx.string_macros[p.decode()] for k, p in parts
    )
    if not parts or spelled != want:
        # Spelling could not be proven (a macro this TU's command does not define as a
        # string, an odd token): the clang value is right by construction.
        return f"{rust_byte_string(want, prefix='c')}.as_ptr() as {rt}"
    utf8 = f in ctx.utf8_files
    if all(k == "lit" for k, _ in parts):
        rust = 'c"' + "".join(spell_literal(p, utf8=utf8, prefix="c")[2:-1] for _, p in parts) + '"'
        _check_spelling(ctx, rust_literal_bytes(rust), want)
        return f"{rust}.as_ptr() as {rt}"
    pieces = [
        spell_literal(p, utf8=False, prefix="b") if k == "lit" else p.decode() for k, p in parts
    ]
    _check_spelling(
        ctx,
        b"".join(
            rust_literal_bytes(x) if x.startswith('b"') else ctx.string_macros[x] for x in pieces
        ),
        want,
    )
    ctx.helpers.add("cstr_cat")
    return f"cstr_cat!({', '.join(pieces)}).as_ptr() as {rt}"


def _wide_value(lit: dict) -> list[int]:
    v = lit["value"]
    if not (v[:2] in ('L"', 'u"') and v.endswith('"')):
        raise DataPortError(f"string literal prefix not handled: {v[:40]}")
    units = [u for piece in _split_printed(v[2:-1]) for u in wide_units(piece)]
    return units[:-1] if units and units[-1] == 0 else units


def _wide_spelling(lit: dict, ctx: Ctx, want: list[int]) -> str:
    """`wstr!("...")` spelled from the source's L"..." pieces when they denote exactly
    clang's units, else from clang's units (still a `wstr!` when they are valid UTF-16)."""
    f, raw = ctx.text(lit)
    pieces = []
    if f is not None:
        for m in _TOKEN.finditer(raw):
            tok = m.group(0)
            if tok.isspace() or tok.startswith((b"/*", b"//")):
                continue
            if tok in (b"L", b"u") or tok.startswith(b'"'):
                pieces.append(tok)
                continue
            pieces = []
            break
    text = b"".join(pieces).replace(b'L"', b'"').replace(b'u"', b'"')
    bodies = re.findall(rb'"((?:[^"\\]|\\.)*)"', text)
    try:
        spelled = [u for b in bodies for u in wide_units(b.decode("utf-8"))] if bodies else None
    except (UnicodeDecodeError, DataPortError):
        spelled = None
    rust = _rust_str(want)
    if rust is None:
        return "(&[" + ", ".join(f"0x{u:04x}u16" for u in [*want, 0]) + "])"
    _check_wide(ctx, rust, want)
    del spelled  # the spelling decides nothing a Rust `\u{..}` cannot say; checked below
    return f"wstr!({rust})"


def _check_wide(ctx: Ctx, rust: str, want: list[int]) -> None:
    got = list(_rust_str_value(rust).encode("utf-16-le"))
    units = [got[i] | got[i + 1] << 8 for i in range(0, len(got), 2)]
    if units != want:
        raise DataPortError(
            f"{ctx.where}: the Rust wide literal spells {units[:20]}, clang {want[:20]}"
        )


def _rust_str_value(rust: str) -> str:
    body, out, i = rust[1:-1], [], 0
    while i < len(body):
        if body[i] == "\\":
            if body[i + 1] == "u":
                j = body.index("}", i)
                out.append(chr(int(body[i + 3 : j], 16)))
                i = j + 1
                continue
            out.append(body[i + 1])
            i += 2
        else:
            out.append(body[i])
            i += 1
    return "".join(out)


def _render_wide(lit: dict, qual: str, ctx: Ctx) -> str:
    want = _wide_value(lit)
    spelled = _wide_spelling(lit, ctx, want)
    ctx.helpers.add("wstr")
    return f"{spelled}.as_ptr() as {rust_type(qual)}"


def _render_string_array(lit: dict, qual: str, ctx: Ctx) -> str:
    """A char or WCHAR ARRAY initialized by a string: the units, zero-padded to the length
    (C drops the NUL when the string exactly fills the array)."""
    m = re.fullmatch(r"(.+?)\s*\[(\d+)\]", qual.strip())
    assert m is not None  # _render routed only an array type here
    elem, n = m.group(1), int(m.group(2))
    rt = rust_type(desugar(elem))
    if lit["value"].startswith(('L"', 'u"')):
        units = _wide_value(lit)
        if len(units) > n:
            raise DataPortError(f"{ctx.where}: {len(units)} units for {qual}")
        rust = _rust_str(units)
        if rust is None:
            vals = [*units, *[0] * (n - len(units))]
            return "[" + ", ".join(f"0x{u:04x}" for u in vals) + "]"
        _check_wide(ctx, rust, units)
        ctx.helpers.add("wstr")
        return f"data_port::wstr_units::<{n}>({rust})"
    data = _clang_string_value(lit)
    if data.endswith(b"\0") and len(data) > n:
        data = data[:-1]
    if len(data) > n:
        raise DataPortError(f"{ctx.where}: {len(data)} bytes for {qual}")
    lit_rs = rust_byte_string(data, prefix="b")
    ctx.helpers.add("cstr_array")
    arr = f"data_port::cstr_array::<{n}>({lit_rs})"
    return (
        arr
        if rt in ("u8", "UCHAR", "BYTE")
        else f"unsafe {{ core::mem::transmute::<[u8; {n}], [{rust_type(elem)}; {n}]>({arr}) }}"
    )


def _check_spelling(ctx: Ctx, got: bytes, want: bytes) -> None:
    """The Rust spelled here, read back as Rust reads it, must be clang's bytes."""
    if got != want:
        raise DataPortError(
            f"{ctx.where}: the Rust literal spells {got[:40]!r}, clang evaluated {want[:40]!r}"
        )


def _list_items(node: dict) -> tuple[list[dict], dict | None]:
    """An InitListExpr's elements and its filler. clang's JSON prints the explicit elements
    INSIDE `array_filler`, after the filler, and omits `inner` -- reading only `inner` made
    `int x[4] = {1, 2}` come out as four zeros."""
    if "array_filler" in node and "inner" not in node:
        fill = node["array_filler"]
        return fill[1:], fill[0]
    return node.get("inner", []), (node.get("array_filler") or [None])[0]


def _render_list(node: dict, qual: str, ctx: Ctx, indent: str) -> str:
    q = qual.strip()
    arr = re.fullmatch(r"(.+?)\s*\[(\d+)\]", q)
    inner, filler = _list_items(node)
    if arr:
        elem, n = arr.group(1), int(arr.group(2))
        items = [_render(c, elem, ctx, indent + "    ") for c in inner]
        if len(items) < n:
            if filler is None:
                raise DataPortError(f"{ctx.where}: {len(items)} of {n} elements and no filler")
            pad = _render(filler, elem, ctx, indent + "    ")
            items += [pad] * (n - len(items))
        if "[" in elem:
            body = ",\n".join(f"{indent}    {it}" for it in items)
            return f"[\n{body},\n{indent}]"
        if (
            all(len(it) < 60 and "\n" not in it for it in items)
            and len(items) <= 16
            and "{" not in "".join(items)
        ):
            return f"[{', '.join(items)}]"
        body = "".join(f"{indent}    {it},\n" for it in items)
        return f"[\n{body}{indent}]"
    name = re.sub(r"^(?:const|volatile)\s+", "", q)
    rec = ctx.records.get(q) or ctx.records.get(name) or ctx.records.get(f"struct {name}")
    if rec is None:
        raise DataPortError(f"{ctx.where}: {q} is not a struct or union this tool can see")
    if any(c.get("kind") == "DesignatedInitExpr" for c in inner):
        # The JSON is clang's SEMANTIC form, already in field order; a designator here would
        # mean the syntactic one, which this reader does not interpret.
        raise DataPortError(f"{ctx.where}: a syntactic designated initializer in {q}")
    rust_name = rec.rust or rust_type(name)
    if rec.kind == "union":
        if not inner:
            return _zero(q)
        member = node.get("field", {}).get("name")
        mtype = node.get("field", {}).get("type", {}).get("qualType")
        if not member or mtype is None:
            raise DataPortError(f"{ctx.where}: union {q} initialized through an anonymous member")
        return f"{rust_name} {{ {member}: {_render(inner[0], mtype, ctx, indent + '    ')} }}"
    if rec.fields and re.fullmatch(r".+\[\]", rec.fields[-1].qual.strip()):
        return _render_flex(node, q, rec, rust_name, inner, ctx, indent)
    if len(inner) > len(rec.fields):
        raise DataPortError(
            f"{ctx.where}: {len(inner)} initializers for {len(rec.fields)} fields of {q}"
        )
    vals: list[tuple[str, str]] = []
    bits: dict[str, int] = {}
    anon = 0
    for i, f in enumerate(rec.fields):
        node_i = inner[i] if i < len(inner) else None
        if f.bits is not None:
            bits[f.name or ""] = clang_int(node_i) if node_i is not None else 0
            continue
        rname = f.name
        if rname is None:
            anon += 1
            rname = f"__bindgen_anon_{anon}"
        v = _render(node_i, f.qual, ctx, indent + "    ") if node_i is not None else _zero(f.qual)
        vals.append((rname, v))
    if bits:
        vals = _with_bitfields(rust_name, rec, vals, bits, ctx)
    return f"{rust_name} {{ " + ", ".join(f"{n}: {v}" for n, v in vals) + " }"


def _render_flex(
    node: dict, q: str, rec: Rec, rust_name: str, inner: list[dict], ctx: Ctx, indent: str
) -> str:
    """A struct ending in a flexible array member, initialized in place (`MODIFIERS`'s
    `BYTE ModNumber[]`). Neither C-- nor bindgen's `__IncompleteArrayField` can hold the
    elements, so the object gets a `#[repr(C)]` twin of the island struct whose tail is
    `[T; N]` for the N clang counted in the initializer. The table's pointers still name the
    island type, so every fixed field's offset (and the alignment) is asserted equal to the
    island's at compile time (the island's own offsets are libclang's, via bindgen)."""
    fixed, tail = rec.fields[:-1], rec.fields[-1]
    if any(f.bits is not None or f.name is None for f in rec.fields) or rec.kind != "struct":
        raise DataPortError(
            f"{ctx.where}: flexible array struct {q} with bit-fields or anonymous members"
        )
    if len(inner) > len(rec.fields):
        raise DataPortError(
            f"{ctx.where}: {len(inner)} initializers for {len(rec.fields)} fields of {q}"
        )
    elem = re.fullmatch(r"(.+?)\s*\[\]", tail.qual.strip()).group(1)  # type: ignore[union-attr]
    tail_node = inner[len(fixed)] if len(inner) > len(fixed) else None
    n = 0
    if tail_node is not None and tail_node.get("kind") == "ImplicitValueInitExpr":
        tail_node = None  # the initializer stopped before the tail: zero elements
    if tail_node is not None:
        tq = tail_node.get("type", {}).get("qualType", "")
        m = re.fullmatch(r".+?\s*\[(\d+)\]", tq.strip())
        if tail_node.get("kind") != "InitListExpr" or not m:
            raise DataPortError(
                f"{ctx.where}: the tail of {q} is initialized by {tail_node.get('kind')} "
                f"of type {tq!r}, not a braced list clang gave a length"
            )
        n = int(m.group(1))
    twin = f"{rust_name}_x{n}"
    vals = []
    for i, f in enumerate(fixed):
        v = _render(inner[i], f.qual, ctx, indent + "    ") if i < len(inner) else _zero(f.qual)
        vals.append((f.name, v))
    tail_qual = f"{elem}[{n}]"
    vals.append(
        (tail.name, _render(tail_node, tail_qual, ctx, indent + "    ") if tail_node else "[]")
    )
    if twin not in ctx.flex:
        decl = "".join(f"    pub {f.name}: {rust_type(f.qual)},\n" for f in fixed)
        decl += f"    pub {tail.name}: [{rust_type(elem)}; {n}],\n"
        checks = [
            f"    assert!(core::mem::offset_of!({twin}, {f.name}) "
            f"== core::mem::offset_of!({rust_name}, {f.name}));\n"
            for f in rec.fields
        ]
        checks.append(
            f"    assert!(core::mem::align_of::<{twin}>()"
            f" == core::mem::align_of::<{rust_name}>());\n"
        )
        ctx.flex[twin] = (
            f"/// `{q}` with its flexible `{tail.name}` initialized in place: {n} element(s).\n"
            f"#[repr(C)]\npub struct {twin} {{\n{decl}}}\n"
            f"const _: () = {{\n{''.join(checks)}}};\n"
        )
    return f"{twin} {{ " + ", ".join(f"{a}: {b}" for a, b in vals) + " }"


def _with_bitfields(
    rust_name: str, rec: Rec, vals: list[tuple[str, str]], bits: dict[str, int], ctx: Ctx
) -> list[tuple[str, str]]:
    """The struct's fields in bindgen's order, its bit-field units packed from the values.

    bindgen groups bit-fields into `_bitfield_N: __BindgenBitfieldUnit<[u8; K]>` units (with
    `_bitfield_align_N` and padding beside them) and says where each one lives in its
    accessor: `self._bitfield_1.get_const::<OFFSETusize, WIDTHu8>()`. That accessor IS the
    layout, so it is read from the island rather than re-derived from C's rules."""
    m = re.search(rf"pub struct {re.escape(rust_name)} \{{(.*?)\n\}}", ctx.island, re.S)
    impl = re.search(rf"impl {re.escape(rust_name)} \{{(.*?)\n\}}\n", ctx.island, re.S)
    if m is None or impl is None:
        raise DataPortError(
            f"{ctx.where}: {rust_name} has bit-fields and the crate island does not define it "
            "(its bindgen accessors are what say how the fields pack)"
        )
    where: dict[str, tuple[int, int, int]] = {}
    for a in re.finditer(
        r"pub fn (\w+)\(&self\)[^{]*\{[^}]*?self\._bitfield_(\d+)\.get"
        r"(?:_const::<(\d+)usize, (\d+)u8>\(\)|\((\d+)usize, (\d+)u8\))",
        impl.group(1),
        re.S,
    ):
        where[a.group(1)] = (
            int(a.group(2)),
            int(a.group(3) or a.group(5)),
            int(a.group(4) or a.group(6)),
        )
    units: dict[int, int] = {}
    for name, value in bits.items():
        if name not in where:
            raise DataPortError(
                f"{ctx.where}: no bindgen accessor for bit-field {rust_name}.{name}"
            )
        unit, off, width = where[name]
        units[unit] = units.get(unit, 0) | ((value & ((1 << width) - 1)) << off)
    by_name = dict(vals)
    out: list[tuple[str, str]] = []
    for fm in re.finditer(r"pub (\w+)\s*:\s*([^,\n]+),", m.group(1)):
        fname, ftype = fm.group(1), fm.group(2).strip()
        u = re.fullmatch(r"_bitfield_(\d+)", fname)
        if u:
            sm = re.search(r"\[u8;\s*(\d+)", ftype)
            if sm is None:
                raise DataPortError(f"{ctx.where}: bindgen unit {rust_name}.{fname} is {ftype}")
            size = int(sm.group(1))
            data = units.get(int(u.group(1)), 0).to_bytes(size, "little")
            shown = ", ".join(
                f"{n}={bits[n]}" for n, w in where.items() if w[0] == int(u.group(1)) and n in bits
            )
            out.append(
                (
                    fname,
                    f"__BindgenBitfieldUnit::new([{', '.join(map(str, data))}]) /* {shown} */",
                )
            )
        elif fname.startswith("_bitfield_align_"):
            out.append((fname, "[]"))
        elif fname.startswith("__bindgen_padding_"):
            out.append((fname, "unsafe { core::mem::zeroed() }"))
        elif fname in by_name:
            out.append((fname, by_name[fname]))
        else:
            raise DataPortError(
                f"{ctx.where}: island field {rust_name}.{fname} has no C initializer"
            )
    return out


# --------------------------------------------------------------------------------------
# a whole TU
# --------------------------------------------------------------------------------------


@dataclass
class Table:
    name: str
    file: str
    linkage: str  # "static" | "extern"
    rust_ty: str
    length: int | None
    init: str
    refs: set[str]
    helpers: set[str] = field(default_factory=set)
    defs: list[str] = field(default_factory=list)  # type definitions the init needs
    cfg: str | None = None  # a `#[cfg(..)]` predicate when the table differs per arch


def _island_consts(crate_src: Path, arch: str) -> dict[str, int]:
    out: dict[str, int] = {}
    for f in (crate_src / f"types_{arch}.rs", crate_src / "prelude.rs"):
        if not f.is_file():
            continue
        for m in re.finditer(
            r"pub const (\w+): [\w:]+ = (-?(?:0x[0-9a-fA-F_]+|[0-9_]+))(?:[iu]\d+|usize|isize)?;",
            f.read_text(),
        ):
            out[m.group(1)] = int(m.group(2).replace("_", ""), 0)
    return out


def _island_lengths(crate_src: Path, arch: str) -> dict[str, int]:
    f = crate_src / f"types_{arch}.rs"
    text = f.read_text() if f.is_file() else ""
    return {
        m.group(1): int(m.group(2))
        for m in re.finditer(r"pub static mut (\w+): \[\w+; (\d+)usize\];", text)
    }


_GUID_TYPES = {"const GUID", "const IID", "const CLSID", "const struct _GUID"}


def _crate_references(crate_src: Path) -> set[str]:
    """Identifiers the crate's translated modules use (islands and the prelude excluded)."""
    from tools.source_walk import source_files

    names: set[str] = set()
    for f in source_files(crate_src):
        if f.parent == crate_src:
            continue  # lib.rs, prelude.rs, types_*.rs
        names |= set(re.findall(r"\b[A-Za-z_]\w*\b", f.read_text(errors="replace")))
    return names


def tables(
    repo: Path,
    arch: str,
    tu: Path,
    crate_src: Path,
    headers: tuple[str, ...] = (),
    scope_tables: bool = True,
) -> list[Table]:
    """The TU's data tables on `arch`. `headers` (path suffixes such as `marshalling/forms.h`)
    adds the statics DEFINED in those headers of the TU's include closure; `scope_tables=False`
    drops the TU's own directory, leaving only them."""
    ast = clang_ast(repo, arch, tu)
    _ENUMS.clear()
    _ENUMS.update(enum_constants(ast))
    _TYPEDEFS.clear()
    _TYPEDEFS.update(typedefs(ast))
    tu = tu.resolve()
    scope = tu.parent
    # The string macros this TU sees on `arch`, by its preprocessor: from compile_argv, so a
    # TU flipped out of the build (no command of its own) borrows its target sibling's, as
    # clang_ast does. One plain literal only; a macro pasting several is left to clang.
    argv = ma.compile_argv(repo, arch, tu)
    assert argv is not None  # clang_ast has already refused a TU with no command
    macros = {
        m.group(1): c_unescape(m.group(2).strip()[1:-1].encode("latin-1"))
        for m in ma._STRING_MACRO.finditer(ma._preprocess_defines(argv))
        if re.fullmatch(r'"(?:[^"\\]|\\.)*"', m.group(2).strip())
    }
    island = crate_src / f"types_{arch}.rs"
    ctx = Ctx({}, records(ast), _island_consts(crate_src, arch), macros, set())
    ctx.layouts = record_layouts(argv)
    _ARCH["layouts"] = ctx.layouts
    _ARCH["ctx"] = ctx
    _ARCH["ptr"] = 8 if arch in ("amd64", "arm64") else 4
    ctx.island = island.read_text() if island.is_file() else ""
    out: list[Table] = []
    referenced = _crate_references(crate_src)
    for d in ast.get("inner", []):
        if d.get("kind") != "VarDecl" or not d.get("init"):
            continue
        f = _decl_file(d)
        if f is None:
            continue
        in_header = any(Path(f).resolve().as_posix().endswith("/" + h.lstrip("/")) for h in headers)
        # A header DEFINE_GUID that `#include <initguid.h>` turns into a definition in THIS
        # TU: ndis's guid.c is nothing else, and without it GUID_BUS_INTERFACE_STANDARD was an
        # unresolved import. Only the ones the crate uses -- wdmguid.h alone instantiates dozens.
        guid = d["type"].get("qualType") in _GUID_TYPES and d["name"] in referenced
        if (
            not in_header
            and not guid
            and not (scope_tables and Path(f).resolve().is_relative_to(scope))
        ):
            continue
        src = ctx.sources.setdefault(f, Path(f).read_bytes())
        try:
            src.decode("utf-8")
            ctx.utf8_files.add(f)
        except UnicodeDecodeError:
            pass
        ctx.where = f"{Path(f).name}: {d['name']}"
        ctx.refs = set()
        ctx.helpers = set()
        qual = d["type"]["qualType"]
        known = set(ctx.flex)
        init = _render(d["inner"][0], qual, ctx, "")
        arr = re.fullmatch(r"(.+?)\s*\[(\d+)\]", qual.strip())
        rust_ty = f"[{rust_type(arr.group(1))}; {arr.group(2)}]" if arr else rust_type(qual)
        for twin in ctx.flex:  # a flexible-array struct's object is its twin, not the island type
            if init.startswith(f"{twin} {{"):
                rust_ty = twin
        out.append(
            Table(
                d["name"],
                f,
                "static" if d.get("storageClass") == "static" else "extern",
                rust_ty,
                int(arr.group(2)) if arr else None,
                init,
                set(ctx.refs),
                set(ctx.helpers),
                [t for n, t in ctx.flex.items() if n not in known or n == rust_ty],
            )
        )
    lengths = _island_lengths(crate_src, arch)
    for t in out:
        if t.linkage == "extern" and _island_length_disagrees(lengths.get(t.name), t.length):
            raise DataPortError(
                f"{t.name}: the C defines {t.length} elements but the crate island declares "
                f"{lengths[t.name]} ({arch}) -- the island is stale or the TU's flags differ"
            )
    return out


def _island_length_disagrees(island: int | None, defined: int | None) -> bool:
    """Whether the island's array length contradicts the C definition's. A 0 is bindgen's
    spelling of an incomplete declaration (`extern T x[];`, cdrom.h's CdromHackItems), which
    any definition completes; only a nonzero length can disagree."""
    return island not in (None, 0) and island != defined


# Each helper is a macro (textual scope: emitted at the top of mod.rs, before every table
# and `mod` line that uses it) and/or const fns, which live in a private `data_port` module:
# a C table named `w` or `n` must not meet a helper's local of the same name.
HELPER_MACROS = {
    "cstr_cat": """\
/// C string-literal concatenation through a string MACRO (`" ReactOS " KERNEL_VERSION_STR
/// " Setup "`): a C compiler pastes the pieces; Rust's `concat!` takes only literals, and
/// the macro's value is generated per build and per arch, so it is joined here, at compile
/// time, into one NUL-terminated array. Each piece's own trailing NUL (a `&[u8; N]` C string
/// constant has one) is dropped first.
macro_rules! cstr_cat {
    ($($part:expr),+ $(,)?) => {{
        const PARTS: &[&[u8]] = &[$($part),+];
        const LEN: usize = data_port::cstr_cat_len(PARTS);
        const BYTES: [u8; LEN] = data_port::cstr_cat_join::<LEN>(PARTS);
        &BYTES
    }};
}
""",
    "wstr": """\
/// A C wide literal (`L"..."`) as Rust: the UTF-16 of a `str`, NUL-terminated, built at
/// compile time. `data_port::wstr_units::<N>` is the same zero-padded into a `WCHAR s[N]`.
macro_rules! wstr {
    ($s:expr) => {{
        const S: &str = $s;
        const N: usize = data_port::wstr_len(S);
        const W: [u16; N] = data_port::wstr_units::<N>(S);
        &W
    }};
}
""",
}

HELPER_FNS = {
    "cstr_cat": """\
    const fn cstr_piece(p: &[u8]) -> usize {
        if !p.is_empty() && p[p.len() - 1] == 0 {
            p.len() - 1
        } else {
            p.len()
        }
    }

    pub(super) const fn cstr_cat_len(parts: &[&[u8]]) -> usize {
        let mut n = 1;
        let mut i = 0;
        while i < parts.len() {
            n += cstr_piece(parts[i]);
            i += 1;
        }
        n
    }

    pub(super) const fn cstr_cat_join<const N: usize>(parts: &[&[u8]]) -> [u8; N] {
        let mut out = [0u8; N];
        let (mut i, mut o) = (0, 0);
        while i < parts.len() {
            let p = parts[i];
            let mut j = 0;
            while j < cstr_piece(p) {
                out[o] = p[j];
                o += 1;
                j += 1;
            }
            i += 1;
        }
        out
    }
""",
    "wstr": """\
    pub(super) const fn wstr_len(s: &str) -> usize {
        let b = s.as_bytes();
        let (mut i, mut n) = (0, 1);
        while i < b.len() {
            let c = b[i];
            let w = if c < 0x80 { 1 } else if c < 0xE0 { 2 } else if c < 0xF0 { 3 } else { 4 };
            n += if w == 4 { 2 } else { 1 };
            i += w;
        }
        n
    }

    pub(super) const fn wstr_units<const N: usize>(s: &str) -> [u16; N] {
        let b = s.as_bytes();
        let mut out = [0u16; N];
        let (mut i, mut o) = (0, 0);
        while i < b.len() {
            let c = b[i] as u32;
            let (cp, w) = if c < 0x80 {
                (c, 1)
            } else if c < 0xE0 {
                (((c & 0x1F) << 6) | (b[i + 1] as u32 & 0x3F), 2)
            } else if c < 0xF0 {
                (((c & 0x0F) << 12) | ((b[i + 1] as u32 & 0x3F) << 6) | (b[i + 2] as u32 & 0x3F), 3)
            } else {
                let v = ((c & 0x07) << 18)
                    | ((b[i + 1] as u32 & 0x3F) << 12)
                    | ((b[i + 2] as u32 & 0x3F) << 6)
                    | (b[i + 3] as u32 & 0x3F);
                (v, 4)
            };
            if cp >= 0x10000 {
                out[o] = (0xD800 | ((cp - 0x10000) >> 10)) as u16;
                out[o + 1] = (0xDC00 | ((cp - 0x10000) & 0x3FF)) as u16;
                o += 2;
            } else {
                out[o] = cp as u16;
                o += 1;
            }
            i += w;
        }
        out
    }
""",
    "cstr_array": """\
    /// A C `char s[N] = "..."`: the bytes zero-padded to N (a string that fills the array
    /// exactly has no NUL, as in C).
    pub(super) const fn cstr_array<const N: usize>(s: &[u8]) -> [u8; N] {
        let mut out = [0u8; N];
        let mut i = 0;
        while i < s.len() && i < N {
            out[i] = s[i];
            i += 1;
        }
        out
    }
""",
}


def helpers_text(used: set[str]) -> str:
    if not used:
        return ""
    macros = "".join(HELPER_MACROS[h] + "\n" for h in sorted(used) if h in HELPER_MACROS)
    fns = "\n".join(HELPER_FNS[h] for h in sorted(used) if h in HELPER_FNS)
    return macros + "mod data_port {\n" + fns + "}\n"


def module_name(header: str) -> str:
    return re.sub(r"\W", "_", Path(header).stem).lower()


def render_module(tu: Path, tabs: list[Table], repo: Path, public: bool = False) -> dict[str, str]:
    """{relative file name: text} -- `mod.rs` for the TU's own tables plus the module list,
    one file per header the tables come from.

    Each file globs the crate prelude for types and constants. The tables it points at in
    a SIBLING file are imported by name: the crate island also declares the extern tables
    (that is how the crate's code reaches them), so a second glob would make every such
    name ambiguous, while an explicit import -- like a local definition -- outranks a glob.

    `public`: a C `static` becomes `pub static mut` (the crate's other modules name it),
    where the default is `pub(super)`. A table with a `cfg` is written under that predicate."""
    by_file: dict[str, list[Table]] = {}
    for t in tabs:
        by_file.setdefault(t.file, []).append(t)
    files: dict[str, str] = {}
    mods = []
    rel_tu = tu.resolve().relative_to(repo)
    helpers = set().union(*(t.helpers for t in tabs)) if tabs else set()
    for f, ts in sorted(by_file.items()):
        is_tu = Path(f).resolve() == tu.resolve()
        rel = Path(f).resolve().relative_to(repo)
        own = {t.name for t in ts}
        sib = sorted(set().union(*(t.refs for t in ts)) - own)
        if not is_tu and set().union(*(t.helpers for t in ts)):
            sib = sorted({*sib, "data_port"})
        head = (
            f"//! `{rel}`'s tables, generated by `rustconv data-port {rel_tu}` from clang's\n"
            "//! AST of the TU's own compile command. Do not hand-edit: re-run the generator.\n\n"
            "#![allow(clippy::all)]\n"
            "#[allow(unused_imports)]\nuse crate::prelude::*;\n"
            + (
                (
                    f"use super::{sib[0]};\n"
                    if len(sib) == 1
                    else f"use super::{{{', '.join(sib)}}};\n"
                )
                if sib and not is_tu
                else ""
            )
            + "\n"
        )
        if is_tu and helpers:
            head += helpers_text(helpers) + "\n"  # before the first table that expands one
        body = []
        for t in ts:
            for d in t.defs:  # a layout-twin's text is arch-free: written once, ungated
                if d not in body:
                    body.append(d)
        for t in ts:
            gate = f"#[cfg({t.cfg})]\n" if t.cfg else ""
            if t.linkage == "static":
                vis = "pub static mut" if public else "pub(super) static mut"
            else:
                vis = "#[no_mangle]\npub static mut"
            body.append(f"{gate}{vis} {t.name}: {t.rust_ty} = {t.init};\n")
        if is_tu:
            files["mod.rs"] = head + "\n".join(body)
        else:
            m = module_name(f)
            mods.append(m)
            files[f"{m}.rs"] = head + "\n".join(body)
    top = files.get("mod.rs") or (
        f"//! `{rel_tu}` as Rust: its data tables, generated by `rustconv data-port` from\n"
        "//! clang's AST of the TU's own compile command. Do not hand-edit: re-run it.\n\n"
        + (helpers_text(helpers) if helpers else "")
    )
    top += "\n" + "".join(f"mod {m};\n" for m in mods)
    if mods:
        top += "\n#[allow(unused_imports)]\n" + "".join(f"pub(super) use {m}::*;\n" for m in mods)
    files["mod.rs"] = top
    # One trailing newline: with no header modules, `top` ended in a blank line, which
    # `git diff --check` refuses (cdrom's data.c).
    return {name: text.rstrip("\n") + "\n" for name, text in files.items()}


_CFG_ARCH = {"i386": "x86", "amd64": "x86_64", "arm64": "aarch64"}


def merge_arches(per: dict[str, list[Table]]) -> list[Table]:
    """One table list from the per-arch ones: a table every arch renders identically is
    written once; one that differs (pointer-sized offsets, `sizeof(void *)` fields) is written
    once per group of arches that agree, each under its `#[cfg(target_arch = ..)]`."""
    arches = list(per)
    order: list[tuple[str, str]] = []
    seen: dict[tuple[str, str], dict[str, Table]] = {}
    for a in arches:
        for t in per[a]:
            key = (t.file, t.name)
            if key not in seen:
                order.append(key)
            seen.setdefault(key, {})[a] = t
    out: list[Table] = []
    for key in order:
        groups: list[tuple[list[str], Table]] = []
        for a in arches:
            t = seen[key].get(a)
            if t is None:
                continue
            sig = (t.linkage, t.rust_ty, t.length, t.init, tuple(t.defs))
            for ga, gt in groups:
                if (gt.linkage, gt.rust_ty, gt.length, gt.init, tuple(gt.defs)) == sig:
                    ga.append(a)
                    break
            else:
                groups.append(([a], t))
        for ga, gt in groups:
            if len(ga) == len(arches):
                cfg = None
            else:
                parts = [f'target_arch = "{_CFG_ARCH[x]}"' for x in ga]
                cfg = parts[0] if len(parts) == 1 else f"any({', '.join(parts)})"
            out.append(replace(gt, cfg=cfg))
    return out


def port(
    repo: Path,
    tu: Path,
    crate_src: Path,
    arches: tuple[str, ...] = ARCHES,
    headers: tuple[str, ...] = (),
    scope_tables: bool = True,
    per_arch: bool = False,
    public: bool = False,
) -> dict[str, str]:
    """Generate on every arch and require one answer -- or, with `per_arch`, one answer per
    group of agreeing arches under `#[cfg(target_arch)]` (never a silent pick of one arch)."""
    if per_arch:
        per = {a: tables(repo, a, tu, crate_src, headers, scope_tables) for a in arches}
        return render_module(tu, merge_arches(per), repo, public)
    outs = {}
    for arch in arches:
        outs[arch] = render_module(
            tu, tables(repo, arch, tu, crate_src, headers, scope_tables), repo, public
        )
    first = outs[arches[0]]
    for arch in arches[1:]:
        if outs[arch] != first:
            diff = sorted(
                k for k in set(first) | set(outs[arch]) if first.get(k) != outs[arch].get(k)
            )
            raise DataPortError(
                f"{tu.name}: the tables differ between {arches[0]} and {arch} in {diff[:5]} -- "
                "pass per_arch=True (--per-arch) to write cfg'd variants"
            )
    return first


def main(argv: list[str] | None = None) -> int:
    """`python -m rustconv.c_data_port <tu.c> --crate <name> --out <dir>`: the generator
    alone, for a crate `rustconv port` does not manage (`port data` also wires the module
    into lib.rs and flips the CMakeLists)."""
    import argparse

    p = argparse.ArgumentParser(description=main.__doc__)
    p.add_argument("tu", type=Path, help="the C data TU")
    p.add_argument("--crate", required=True, help="the crate whose island types the tables")
    p.add_argument(
        "--out", type=Path, required=True, help="directory for mod.rs + per-header files"
    )
    p.add_argument("--arches", default=",".join(ARCHES))
    p.add_argument(
        "--header",
        action="append",
        default=[],
        help="also port the statics defined in this header (path suffix, repeatable)",
    )
    p.add_argument(
        "--headers-only", action="store_true", help="port ONLY --header statics, not the TU's own"
    )
    p.add_argument("--per-arch", action="store_true", help="cfg'd variants where arches differ")
    p.add_argument("--public", action="store_true", help="C statics become `pub`, not `pub(super)`")
    p.add_argument(
        "--no-mod",
        action="store_true",
        help="do not write mod.rs (the caller owns the module list)",
    )
    a = p.parse_args(argv)
    repo = Path(__file__).resolve().parents[3]
    try:
        files = port(
            repo,
            a.tu.resolve(),
            repo / "sdk" / "lib" / "rust" / a.crate / "src",
            tuple(a.arches.split(",")),
            tuple(a.header),
            not a.headers_only,
            a.per_arch,
            a.public,
        )
    except DataPortError as e:
        print(f"data-port: {e}")
        return 1
    a.out.mkdir(parents=True, exist_ok=True)
    for name, text in files.items():
        if name == "mod.rs" and a.no_mod:
            continue
        (a.out / name).write_text(text)
    print(f"data-port: {len(files)} file(s) in {a.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
