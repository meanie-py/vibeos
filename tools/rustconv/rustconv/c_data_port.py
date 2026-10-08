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

Not handled, loudly: wide literals, unions, bit-fields, designated initializers with gaps
clang does not fill, and any initializer kind not listed in `_render`. Each is a
`DataPortError` naming the declaration, never a silent zero.
"""

from __future__ import annotations

import json
import re
import subprocess
from dataclasses import dataclass, field
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


def enum_constants(ast: dict) -> dict[str, tuple[int, str]]:
    """EnumConstantDecl id -> (value, the enum's tag). An explicit initializer carries its
    evaluated value; an implicit one is the previous constant + 1."""
    out: dict[str, tuple[int, str]] = {}
    for d in ast.get("inner", []):
        if d.get("kind") != "EnumDecl":
            continue
        nxt = 0
        for c in d.get("inner", []):
            if c.get("kind") != "EnumConstantDecl":
                continue
            exprs = [i for i in c.get("inner", []) if not i.get("kind", "").endswith("Comment")]
            vals = [i["value"] for i in exprs if "value" in i]
            if exprs and not vals:
                raise DataPortError(f"enum constant {c.get('name')}: no evaluated value in the AST")
            v = int(vals[0]) if vals else nxt
            out[c["id"]] = (v, d.get("name", ""))
            out[c["name"]] = (v, d.get("name", ""))
            nxt = v + 1
    return out


def records(ast: dict) -> dict[str, list[tuple[str, str]]]:
    """typedef or struct name -> its fields (name, qualType), in declaration order."""
    by_id: dict[str, list[tuple[str, str]]] = {}
    named: dict[str, str] = {}
    out: dict[str, list[tuple[str, str]]] = {}
    for d in ast.get("inner", []):
        if d.get("kind") == "RecordDecl" and d.get("completeDefinition"):
            fields = []
            for f in d.get("inner", []):
                if f.get("kind") != "FieldDecl":
                    continue
                if f.get("isBitfield"):
                    fields = None
                    break
                fields.append((f.get("name", ""), f["type"]["qualType"]))
            if fields is not None and d.get("tagUsed") == "struct":
                by_id[d["id"]] = fields
                if d.get("name"):
                    named[f"struct {d['name']}"] = d["id"]
        elif d.get("kind") == "TypedefDecl":
            ids = re.findall(r'"decl":\s*\{"id":\s*"(0x[0-9a-f]+)"', json.dumps(d.get("inner", [])))
            if ids:
                named[d["name"]] = ids[0]
    for name, rid in named.items():
        if rid in by_id:
            out[name] = by_id[rid]
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


def _clang_string_value(node: dict) -> bytes:
    """clang's evaluated value of a StringLiteral (printed with C escapes)."""
    v = node["value"]
    if not (v.startswith('"') and v.endswith('"')):
        raise DataPortError(f"wide or prefixed string literal not handled: {v[:40]}")
    return c_unescape(v[1:-1].encode("latin-1"))


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
    records: dict[str, list[tuple[str, str]]]
    island_consts: dict[str, int]
    string_macros: dict[str, bytes]
    utf8_files: set[str]
    refs: set[str] = field(default_factory=set)  # every object the initializer points at
    where: str = ""

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


_ENUMS: dict[str, tuple[int, str]] = {}
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


def clang_int(node: dict) -> int:
    """clang's integer value of a constant expression tree."""
    n, _ = _strip(node)
    k = n.get("kind")
    if k in ("IntegerLiteral", "CharacterLiteral"):
        return int(n["value"])
    if k == "UnaryOperator":
        v = clang_int(n["inner"][0])
        return {"-": -v, "~": ~v, "+": v, "!": int(not v)}[n["opcode"]]
    if k == "BinaryOperator":
        a, b = clang_int(n["inner"][0]), clang_int(n["inner"][1])
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
        return _ENUMS[ref["id"]][0]
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
            return (
                f"{spelling} as {rt}" if re.fullmatch(r"\w+", spelling) else f"({spelling}) as {rt}"
            )
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
    return f"cstr_cat!({', '.join(pieces)}).as_ptr() as {rt}"


def _check_spelling(ctx: Ctx, got: bytes, want: bytes) -> None:
    """The Rust spelled here, read back as Rust reads it, must be clang's bytes."""
    if got != want:
        raise DataPortError(
            f"{ctx.where}: the Rust literal spells {got[:40]!r}, clang evaluated {want[:40]!r}"
        )


def _render_list(node: dict, qual: str, ctx: Ctx, indent: str) -> str:
    q = qual.strip()
    arr = re.fullmatch(r"(.+?)\s*\[(\d+)\]", q)
    inner = node.get("inner", [])
    if arr:
        elem, n = arr.group(1), int(arr.group(2))
        items = [_render(c, elem, ctx, indent + "    ") for c in inner]
        if len(items) < n:
            if "array_filler" not in node and len(items) != n:
                raise DataPortError(f"{ctx.where}: {len(items)} of {n} elements and no filler")
            items += [_zero(elem)] * (n - len(items))
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
    fields = ctx.records.get(name) or ctx.records.get(f"struct {name}")
    if fields is None:
        raise DataPortError(
            f"{ctx.where}: {q} is not a struct this tool can see (union? bit-field?)"
        )
    if node.get("field") or any(c.get("kind") == "DesignatedInitExpr" for c in inner):
        raise DataPortError(f"{ctx.where}: designated initializer in {q}")
    if len(inner) > len(fields):
        raise DataPortError(
            f"{ctx.where}: {len(inner)} initializers for {len(fields)} fields of {q}"
        )
    vals = [_render(c, ft, ctx, indent + "    ") for c, (_, ft) in zip(inner, fields, strict=False)]
    vals += [_zero(ft) for _, ft in fields[len(vals) :]]
    return (
        f"{rust_type(name)} {{ "
        + ", ".join(f"{fname}: {v}" for (fname, _), v in zip(fields, vals, strict=True))
        + " }"
    )


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


def tables(repo: Path, arch: str, tu: Path, crate_src: Path) -> list[Table]:
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
    ctx = Ctx({}, records(ast), _island_consts(crate_src, arch), macros, set())
    out: list[Table] = []
    for d in ast.get("inner", []):
        if d.get("kind") != "VarDecl" or not d.get("init"):
            continue
        f = _decl_file(d)
        if f is None or not Path(f).resolve().is_relative_to(scope):
            continue
        src = ctx.sources.setdefault(f, Path(f).read_bytes())
        try:
            src.decode("utf-8")
            ctx.utf8_files.add(f)
        except UnicodeDecodeError:
            pass
        ctx.where = f"{Path(f).name}: {d['name']}"
        ctx.refs = set()
        qual = d["type"]["qualType"]
        init = _render(d["inner"][0], qual, ctx, "")
        arr = re.fullmatch(r"(.+?)\s*\[(\d+)\]", qual.strip())
        rust_ty = f"[{rust_type(arr.group(1))}; {arr.group(2)}]" if arr else rust_type(qual)
        out.append(
            Table(
                d["name"],
                f,
                "static" if d.get("storageClass") == "static" else "extern",
                rust_ty,
                int(arr.group(2)) if arr else None,
                init,
                set(ctx.refs),
            )
        )
    lengths = _island_lengths(crate_src, arch)
    for t in out:
        if t.linkage == "extern" and t.name in lengths and lengths[t.name] != t.length:
            raise DataPortError(
                f"{t.name}: the C defines {t.length} elements but the crate island declares "
                f"{lengths[t.name]} ({arch}) -- the island is stale or the TU's flags differ"
            )
    return out


CSTR_CAT = """\
/// C string-literal concatenation through a string MACRO (`" ReactOS " KERNEL_VERSION_STR
/// " Setup "`): a C compiler pastes the pieces; Rust's `concat!` takes only literals, and
/// the macro's value is generated per build and per arch, so it is joined here, at compile
/// time, into one NUL-terminated array. Each piece's own trailing NUL (a `&[u8; N]` C string
/// constant has one) is dropped first.
macro_rules! cstr_cat {
    ($($part:expr),+ $(,)?) => {{
        const PARTS: &[&[u8]] = &[$($part),+];
        const LEN: usize = cstr_cat_len(PARTS);
        const BYTES: [u8; LEN] = cstr_cat_join::<LEN>(PARTS);
        &BYTES
    }};
}

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
"""


def module_name(header: str) -> str:
    return re.sub(r"\W", "_", Path(header).stem).lower()


def render_module(tu: Path, tabs: list[Table], repo: Path) -> dict[str, str]:
    """{relative file name: text} -- `mod.rs` for the TU's own tables plus the module list,
    one file per header the tables come from.

    Each file globs the crate prelude for types and constants. The tables it points at in
    a SIBLING file are imported by name: the crate island also declares the extern tables
    (that is how the crate's code reaches them), so a second glob would make every such
    name ambiguous, while an explicit import -- like a local definition -- outranks a glob."""
    by_file: dict[str, list[Table]] = {}
    for t in tabs:
        by_file.setdefault(t.file, []).append(t)
    files: dict[str, str] = {}
    mods = []
    rel_tu = tu.resolve().relative_to(repo)
    uses_cat = any("cstr_cat!" in t.init for t in tabs)
    for f, ts in sorted(by_file.items()):
        is_tu = Path(f).resolve() == tu.resolve()
        rel = Path(f).resolve().relative_to(repo)
        own = {t.name for t in ts}
        sib = sorted(set().union(*(t.refs for t in ts)) - own)
        if not is_tu and any("cstr_cat!" in t.init for t in ts):
            sib = sorted({*sib, "cstr_cat_join", "cstr_cat_len"})
        head = (
            f"//! `{rel}`'s tables, generated by `rustconv data-port {rel_tu}` from clang's\n"
            "//! AST of the TU's own compile command. Do not hand-edit: re-run the generator.\n\n"
            "#![allow(clippy::all)]\n"
            "#[allow(unused_imports)]\nuse crate::prelude::*;\n"
            + (f"use super::{{{', '.join(sib)}}};\n" if sib and not is_tu else "")
            + "\n"
        )
        body = []
        for t in ts:
            vis = (
                "pub(super) static mut" if t.linkage == "static" else "#[no_mangle]\npub static mut"
            )
            body.append(f"{vis} {t.name}: {t.rust_ty} = {t.init};\n")
        if is_tu:
            files["mod.rs"] = head + "\n".join(body)
        else:
            m = module_name(f)
            mods.append(m)
            files[f"{m}.rs"] = head + "\n".join(body)
    top = files.get("mod.rs") or (
        f"//! `{rel_tu}` as Rust: its data tables, generated by `rustconv data-port` from\n"
        "//! clang's AST of the TU's own compile command. Do not hand-edit: re-run it.\n\n"
    )
    if uses_cat:
        top += "\n" + CSTR_CAT  # textual scope: before the `mod` lines that use it
    top += "\n" + "".join(f"mod {m};\n" for m in mods)
    if mods:
        top += "\n#[allow(unused_imports)]\n" + "".join(f"pub(super) use {m}::*;\n" for m in mods)
    files["mod.rs"] = top
    return files


def port(repo: Path, tu: Path, crate_src: Path, arches: tuple[str, ...] = ARCHES) -> dict[str, str]:
    """Generate on every arch and require one answer."""
    outs = {}
    for arch in arches:
        outs[arch] = render_module(tu, tables(repo, arch, tu, crate_src), repo)
    first = outs[arches[0]]
    for arch in arches[1:]:
        if outs[arch] != first:
            diff = sorted(
                k for k in set(first) | set(outs[arch]) if first.get(k) != outs[arch].get(k)
            )
            raise DataPortError(
                f"{tu.name}: the tables differ between {arches[0]} and {arch} in {diff[:5]} -- "
                "arch-dependent data needs cfg'd variants, which this tool does not write yet"
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
    a = p.parse_args(argv)
    repo = Path(__file__).resolve().parents[3]
    try:
        files = port(
            repo,
            a.tu.resolve(),
            repo / "sdk" / "lib" / "rust" / a.crate / "src",
            tuple(a.arches.split(",")),
        )
    except DataPortError as e:
        print(f"data-port: {e}")
        return 1
    a.out.mkdir(parents=True, exist_ok=True)
    for name, text in files.items():
        (a.out / name).write_text(text)
    print(f"data-port: {len(files)} file(s) in {a.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
