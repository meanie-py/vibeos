"""Deterministic COM vtable-table generation (C → Rust). NO LLM, NO paid calls.

A COM vtbl table is a *positional* struct initializer:

    static const IStorageVtbl StorageImpl_Vtbl =
    {
      StorageBaseImpl_QueryInterface,   /* slot 0 */
      StorageBaseImpl_AddRef,           /* slot 1 */
      ...
    };

Nothing about that needs a language model: the slot *order* is ground truth we
already own (the bindgen'd `I<X>Vtbl` structs in the `com-sys` crate, generated
from the real SDK COM headers), and the slot *values* are just the identifiers
the C lists. Asking an LLM to "translate" it is pure downside — and it bit us:
the paid preamble translation of `dll/win32/ole32/storage32.c` silently dropped
**every** vtbl table, which does not fail the build, it just produces COM objects
with no methods. 23 tables had to be hand-authored across ole32.

So: generate them here, deterministically, and never ask the model.

Field-order ground truth is resolved in layers (`FieldOrderResolver`):

1. **com-sys bindgen** (`sdk/lib/rust/com-sys/src/bindgen_<arch>.rs`) — authoritative
   for every real COM interface. Slots there are
   `Option<unsafe extern "stdcall"|"C" fn(..)>`, so values get wrapped in `Some(..)`
   and a C `NULL` slot becomes `None`.
2. **The DLL's own Rust crate** — for DLL-*internal* vtbl structs that are not COM
   interfaces and so are absent from com-sys. `StorageBaseImplVtbl` (ole32's
   `storage32.h`) is the live example: plain C function pointers, no
   STDMETHODCALLTYPE, and `ole32-rust` declares them **non-`Option`**. We read the
   real Rust declaration so the rendering matches what will actually compile.
3. **The DLL's C headers** — same-directory `*.h` fallback, for a vtbl struct that
   has no Rust declaration yet (a DLL being translated for the first time). Field
   order comes from the C; such plain-C-pointer tables render non-`Option`.

Hard rules (a mis-zipped vtbl is a jump to the *wrong method* — silent and awful):

* **Arity is checked.** C initializer count != struct field count is a `VtblArityError`.
  Never pad, never truncate.
* **Nothing is guessed.** `Option`-ness is read off the actual Rust declaration, not
  assumed; if no field order can be resolved for a type it is a `VtblResolveError`.

Public API:
    find_vtbl_tables(c_path, ...)   -> list[VtblTable]
    render_vtbl_tables(c_path, ...) -> str   (the Rust)
    com_method_names(c_path, ...)   -> set[str]  (authoritative "these are COM methods")
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from functools import lru_cache
from pathlib import Path

import tree_sitter_c as tsc
from tree_sitter import Language, Parser

from .config import REACTOS_ROOT

C_LANG = Language(tsc.language())

REPO_ROOT = REACTOS_ROOT  # the invoked tree, not this file's checkout
COM_SYS_SRC = REPO_ROOT / "sdk" / "lib" / "rust" / "com-sys" / "src"

#: Field order is identical across arches (only the ABI on each slot differs:
#: stdcall on i386, C on amd64/arm64), so any one bindgen file is a valid source
#: of slot order. `test_vtbl_gen.py` pins that invariant.
DEFAULT_ARCH = "i386"


class VtblError(Exception):
    """Base for vtbl-generation failures. Always LOUD — never degrade silently."""


class VtblResolveError(VtblError):
    """No field order could be resolved for a vtbl struct type."""


class VtblArityError(VtblError):
    """C initializer count != Rust/C struct field count. A mis-zipped vtbl would
    dispatch calls to the WRONG method, so this is fatal, never patched over."""


# ---------------------------------------------------------------------------
# Data model
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class VtblField:
    """One slot of a vtbl *struct* (the shape), from the field-order ground truth."""

    name: str
    is_option: bool


@dataclass(frozen=True)
class VtblSlot:
    """One slot of a vtbl *table* (the shape zipped with the C's initializer)."""

    field: str  #: struct field name (from ground truth)
    value: str  #: C initializer text — a function name, or "NULL"
    is_null: bool
    is_option: bool

    def render(self) -> str:
        if self.is_null:
            return f"    {self.field}: None," if self.is_option else f"    {self.field}: NULL_SLOT,"
        if self.is_option:
            return f"    {self.field}: Some({self.value}),"
        return f"    {self.field}: {self.value},"


@dataclass
class VtblTable:
    name: str  #: e.g. "StorageImpl_Vtbl"
    type_name: str  #: e.g. "IStorageVtbl"
    line: int  #: 1-based line in the C source
    slots: list[VtblSlot] = field(default_factory=list)
    origin: str = ""  #: where field order came from (provenance, for the doc comment)

    @property
    def method_names(self) -> list[str]:
        """Function names occupying a slot (NULL slots excluded)."""
        return [s.value for s in self.slots if not s.is_null]

    def render(self, c_name: str | None = None) -> str:
        src = f"{c_name}:{self.line}" if c_name else f"line {self.line}"
        lines = [
            f"/// {src} `static const {self.type_name} {self.name}`  (slot order: {self.origin})",
            f"pub static {self.name}: {self.type_name} = {self.type_name} {{",
            *(s.render() for s in self.slots),
            "};",
        ]
        return "\n".join(lines)


# ---------------------------------------------------------------------------
# Ground truth #1 + #2: Rust struct declarations (com-sys bindgen, DLL crate)
# ---------------------------------------------------------------------------


def _rust_struct_fields(text: str, struct_name: str) -> list[VtblField] | None:
    """Field order + Option-ness of `pub struct <struct_name> { .. }` in Rust text.

    Depth-tracked rather than line-regexed: a bindgen vtbl field's type is a
    multi-line `Option<unsafe extern "stdcall" fn(..) -> HRESULT>` whose inner
    lines also start with identifiers, so only nesting depth reliably tells a
    top-level field from a function parameter.
    """
    m = re.search(rf"^pub struct {re.escape(struct_name)}\s*\{{", text, re.M)
    if not m:
        return None

    # Walk the struct body, tracking (), [], <>, {} nesting. Fields are the
    # `pub <ident>:` occurrences at depth 0 relative to the body.
    i = m.end()
    depth = 0  # braces/brackets/parens
    angle = 0  # generic args
    fields: list[VtblField] = []
    cur: str | None = None
    start = i
    n = len(text)

    def _flush(name: str | None, type_text: str) -> None:
        if name is not None:
            fields.append(VtblField(name=name, is_option="Option<" in type_text))

    while i < n:
        c = text[i]
        if c in "([{":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif c == "}":
            if depth == 0:
                _flush(cur, text[start:i])
                return fields
            depth -= 1
        elif c == "<" and depth == 0:
            angle += 1
        elif c == ">" and depth == 0 and angle:
            # `->` is a return arrow, not a closing angle bracket.
            if text[i - 1] != "-":
                angle -= 1
        elif c == "," and depth == 0 and angle == 0:
            _flush(cur, text[start:i])
            cur, start = None, i + 1
        elif depth == 0 and angle == 0 and cur is None:
            fm = re.match(r"\s*pub\s+(\w+)\s*:", text[i:])
            if fm:
                cur = fm.group(1)
                i += fm.end()
                start = i
                continue
        i += 1
    return fields or None


@lru_cache(maxsize=8)
def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


@lru_cache(maxsize=8)
def _com_sys_text(arch: str) -> str:
    p = COM_SYS_SRC / f"bindgen_{arch}.rs"
    if not p.is_file():
        raise VtblResolveError(
            f"com-sys bindgen not found: {p}\n"
            "Field order for COM interfaces comes from the bindgen'd com-sys crate; "
            "without it no vtbl can be generated safely."
        )
    return _read(p)


# ---------------------------------------------------------------------------
# Ground truth #3: C header struct declarations
# ---------------------------------------------------------------------------


def _ts_parse(src: bytes):
    return Parser(C_LANG).parse(src)


def _field_declarator_name(node, src: bytes) -> str | None:
    """Innermost declarator name — digs through pointer/function/parenthesized
    declarators so `HRESULT (*Flush)(StorageBaseImpl*)` yields "Flush"."""
    if node is None:
        return None
    if node.type in ("field_identifier", "identifier", "type_identifier"):
        return src[node.start_byte : node.end_byte].decode("utf-8", "replace")
    for key in ("declarator",):
        child = node.child_by_field_name(key)
        if child is not None:
            name = _field_declarator_name(child, src)
            if name:
                return name
    for child in node.named_children:
        if child.type in (
            "pointer_declarator",
            "function_declarator",
            "parenthesized_declarator",
            "array_declarator",
            "field_identifier",
            "identifier",
        ):
            name = _field_declarator_name(child, src)
            if name:
                return name
    return None


def _c_struct_fields_in(text: str, struct_name: str) -> list[VtblField] | None:
    """Field order of `struct <struct_name> { .. }` in C text (plain fn pointers)."""
    tree = _ts_parse(text.encode())
    src = text.encode()
    found: list[VtblField] | None = None

    def walk(node):
        nonlocal found
        if found is not None:
            return
        if node.type == "struct_specifier":
            name_node = node.child_by_field_name("name")
            body = node.child_by_field_name("body")
            if name_node is not None and body is not None:
                name = src[name_node.start_byte : name_node.end_byte].decode("utf-8", "replace")
                if name == struct_name:
                    fields: list[VtblField] = []
                    for fd in body.named_children:
                        if fd.type != "field_declaration":
                            continue
                        for d in fd.children_by_field_name("declarator"):
                            fname = _field_declarator_name(d, src)
                            if fname:
                                # Plain C function pointers (no STDMETHODCALLTYPE)
                                # — non-nullable in every table we have seen, and
                                # that is how the Rust side declares them.
                                fields.append(VtblField(name=fname, is_option=False))
                    if fields:
                        found = fields
                        return
        for ch in node.children:
            walk(ch)

    walk(tree.root_node)
    return found


# ---------------------------------------------------------------------------
# Field-order resolver
# ---------------------------------------------------------------------------


class FieldOrderResolver:
    """Resolves `<X>Vtbl` -> ordered fields, from com-sys, then the DLL's Rust
    crate, then the DLL's C headers. Each layer records provenance."""

    def __init__(
        self,
        arch: str = DEFAULT_ARCH,
        rust_sources: list[Path] | None = None,
        c_headers: list[Path] | None = None,
    ) -> None:
        self.arch = arch
        self.rust_sources = list(rust_sources or [])
        self.c_headers = list(c_headers or [])
        self._cache: dict[str, tuple[list[VtblField], str]] = {}

    def resolve(self, type_name: str) -> tuple[list[VtblField], str]:
        if type_name in self._cache:
            return self._cache[type_name]

        # 1. com-sys bindgen (authoritative for COM interfaces)
        fields = _rust_struct_fields(_com_sys_text(self.arch), type_name)
        if fields:
            got = (fields, f"com-sys/bindgen_{self.arch}.rs")
            self._cache[type_name] = got
            return got

        # 2. the DLL's own Rust crate (internal, non-COM vtbl structs)
        for p in self.rust_sources:
            if not p.is_file():
                continue
            fields = _rust_struct_fields(_read(p), type_name)
            if fields:
                got = (fields, f"rust:{p.name}")
                self._cache[type_name] = got
                return got

        # 3. the DLL's C headers (no Rust declaration yet)
        for p in self.c_headers:
            if not p.is_file():
                continue
            fields = _c_struct_fields_in(_read(p), type_name)
            if fields:
                got = (fields, f"c-header:{p.name}")
                self._cache[type_name] = got
                return got

        raise VtblResolveError(
            f"cannot resolve field order for vtbl struct '{type_name}'.\n"
            f"  looked in: com-sys/bindgen_{self.arch}.rs, "
            f"{len(self.rust_sources)} Rust source(s), {len(self.c_headers)} C header(s).\n"
            "  If it is a COM interface, add it to com-sys/wrapper.h and regenerate; "
            "if it is a DLL-internal vtbl, point --rust-sources/--c-headers at its "
            "declaration. Refusing to guess slot order."
        )


# ---------------------------------------------------------------------------
# C side: find the tables
# ---------------------------------------------------------------------------

#: A vtbl struct type, by the universal COM/IDL naming convention (`I<X>Vtbl`)
#: and the DLL-internal convention we see in practice (`StorageBaseImplVtbl`).
_VTBL_TYPE_RE = re.compile(r"^\w+Vtbl$")


def _initializer_values(node, src: bytes) -> list[tuple[str | None, str]]:
    """Flatten an `initializer_list` into [(designated_field | None, value_text)].

    Skips comments (tree-sitter keeps them as siblings), and handles both
    positional (`Foo_AddRef,`) and designated (`.AddRef = Foo_AddRef,`) forms.
    """
    out: list[tuple[str | None, str]] = []
    for child in node.named_children:
        if child.type == "comment":
            continue
        if child.type == "initializer_pair":
            desig = None
            for d in child.children_by_field_name("designator"):
                txt = src[d.start_byte : d.end_byte].decode("utf-8", "replace")
                desig = txt.lstrip(".").strip()
            val_node = child.child_by_field_name("value")
            val = (
                src[val_node.start_byte : val_node.end_byte].decode("utf-8", "replace").strip()
                if val_node is not None
                else ""
            )
            out.append((desig, val))
        else:
            val = src[child.start_byte : child.end_byte].decode("utf-8", "replace").strip()
            out.append((None, val))
    return out


def _is_null(value: str) -> bool:
    v = re.sub(r"\s+", "", value)
    v = re.sub(r"^\((?:void|VOID)\s*\*\)", "", v)  # (void*)0
    return v in ("NULL", "0", "nullptr", "(void*)0")


def find_vtbl_tables(
    c_path: Path,
    arch: str = DEFAULT_ARCH,
    rust_sources: list[Path] | None = None,
    c_headers: list[Path] | None = None,
    resolver: FieldOrderResolver | None = None,
) -> list[VtblTable]:
    """Every file-scope `static const <X>Vtbl <Name> = { .. };` table in `c_path`.

    Forward declarations (`static const IStorageVtbl StorageInternalImpl_Vtbl;`
    — no initializer) are skipped: they carry no slots, and the real definition
    appears later in the same file. `static` without `const` (olepropframe.c) and
    a type/name split across lines (olefont.c:1865) are handled by parsing the
    AST rather than matching line shapes.
    """
    c_path = Path(c_path)
    if rust_sources is None and c_headers is None:
        # Default: look for vtbl struct declarations next to the C file.
        c_headers = sorted(c_path.parent.glob("*.h"))
    resolver = resolver or FieldOrderResolver(
        arch=arch, rust_sources=rust_sources, c_headers=c_headers
    )

    text = _read(c_path)
    src = text.encode()
    tree = _ts_parse(src)
    tables: list[VtblTable] = []

    def type_of(node) -> str | None:
        t = node.child_by_field_name("type")
        if t is None:
            return None
        name = src[t.start_byte : t.end_byte].decode("utf-8", "replace").strip()
        # `struct IRichEditOleCallbackVtbl olecallbackVtbl = {..}` (wordpad) spells the tag:
        # the type node is the whole `struct X` specifier.
        name = re.sub(r"^struct\s+", "", name)
        return name if _VTBL_TYPE_RE.match(name) else None

    for node in tree.root_node.named_children:
        if node.type != "declaration":
            continue
        type_name = type_of(node)
        if not type_name:
            continue
        for decl in node.children_by_field_name("declarator"):
            if decl.type != "init_declarator":
                continue  # forward declaration — no initializer, no slots
            name_node = decl.child_by_field_name("declarator")
            val_node = decl.child_by_field_name("value")
            if name_node is None or val_node is None or val_node.type != "initializer_list":
                continue
            table_name = src[name_node.start_byte : name_node.end_byte].decode("utf-8", "replace")

            fields, origin = resolver.resolve(type_name)
            inits = _initializer_values(val_node, src)
            line = node.start_point[0] + 1

            # ARITY: a mis-zipped vtbl dispatches to the WRONG method. Fatal.
            if len(inits) != len(fields):
                raise VtblArityError(
                    f"{c_path.name}:{line}: vtbl table '{table_name}' of type "
                    f"'{type_name}' has {len(inits)} initializer(s) but the struct "
                    f"has {len(fields)} field(s) (slot order from {origin}).\n"
                    f"  struct fields: {[f.name for f in fields]}\n"
                    f"  C initializers: {[v for _, v in inits]}\n"
                    "  Refusing to pad or truncate — a mis-zipped vtbl is a jump to "
                    "the wrong method."
                )

            by_name = {f.name: f for f in fields}
            slots: list[VtblSlot] = []
            for idx, (desig, value) in enumerate(inits):
                if desig is not None:
                    if desig not in by_name:
                        raise VtblArityError(
                            f"{c_path.name}:{line}: vtbl table '{table_name}' designates "
                            f"unknown field '.{desig}' (struct '{type_name}' has "
                            f"{[f.name for f in fields]})."
                        )
                    f = by_name[desig]
                else:
                    f = fields[idx]
                slots.append(
                    VtblSlot(
                        field=f.name,
                        value=value,
                        is_null=_is_null(value),
                        is_option=f.is_option,
                    )
                )
            tables.append(
                VtblTable(
                    name=table_name,
                    type_name=type_name,
                    line=line,
                    slots=slots,
                    origin=origin,
                )
            )
    return tables


def render_vtbl_tables(
    c_path: Path,
    arch: str = DEFAULT_ARCH,
    rust_sources: list[Path] | None = None,
    c_headers: list[Path] | None = None,
    header: bool = True,
) -> str:
    """The Rust for every vtbl table in `c_path`. Empty string if there are none."""
    c_path = Path(c_path)
    tables = find_vtbl_tables(c_path, arch=arch, rust_sources=rust_sources, c_headers=c_headers)
    if not tables:
        return ""
    parts: list[str] = []
    if header:
        parts.append(
            "// COM vtable tables — GENERATED DETERMINISTICALLY from "
            f"{c_path.name} by rustconv/vtbl_gen.py.\n"
            "// Slot order is taken from the bindgen'd struct definition, NOT from a\n"
            "// language model: an LLM silently dropped all 12 of these once "
            "(method-less COM objects).\n"
            "// Regenerate rather than hand-edit."
        )
    parts.extend(t.render(c_name=c_path.name) for t in tables)
    return "\n\n".join(parts) + "\n"


def com_method_names(
    c_path: Path,
    arch: str = DEFAULT_ARCH,
    rust_sources: list[Path] | None = None,
    c_headers: list[Path] | None = None,
) -> set[str]:
    """The authoritative "these functions are COM methods" set for `c_path`.

    A function that occupies a vtbl slot is called through the COM ABI, so its
    Rust must be `extern "system"` (stdcall on i386) — the ABI enforcer consumes
    this. A slot filled by a function from *another* C file of the same DLL is
    still reported; the name is what matters.
    """
    names: set[str] = set()
    for t in find_vtbl_tables(c_path, arch=arch, rust_sources=rust_sources, c_headers=c_headers):
        names.update(t.method_names)
    return names
