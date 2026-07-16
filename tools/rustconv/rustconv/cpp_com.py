"""Deterministic C++/ATL COM-class analysis and Rust scaffold generation.

The language model translates method *bodies*.  It must never invent COM ABI:
interface slot order, vtable signatures, QueryInterface routing, interface-to-object
offset recovery, refcounting, or activation metadata.  This module derives that
surface from two compiler-facing sources:

* the class header (``BEGIN_COM_MAP`` is the authoritative IID/interface map), and
* WIDL's generated C header (the authoritative flattened vtable shape).

Only the deliberately small, proven subset is accepted.  Unsupported ATL map
entries or state fields are reported and make ``require_supported`` fail loudly.
"""

from __future__ import annotations

import ast
import hashlib
import json
import re
from collections import Counter
from dataclasses import asdict, dataclass, field, replace
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .callgraph import FunctionDep


class CppComError(Exception):
    """The deterministic COM preflight could not prove a safe scaffold."""


@dataclass(frozen=True)
class ComMapEntry:
    iid: str
    interface: str
    macro: str


@dataclass(frozen=True)
class InterfaceMethod:
    name: str
    return_type: str
    params: tuple[tuple[str, str], ...]


@dataclass(frozen=True)
class InterfaceSpec:
    name: str
    methods: tuple[InterfaceMethod, ...]
    source: str


@dataclass(frozen=True)
class PlainStateField:
    """One C-layout object field safe to spell directly in the Rust owner."""

    declaration: str
    c_type: str
    name: str


@dataclass
class CppComClass:
    name: str
    source: Path
    header: Path
    com_map: list[ComMapEntry] = field(default_factory=list)
    registry_resource: str | None = None
    not_aggregatable: bool = False
    object_map_clsid: str | None = None
    method_names: list[str] = field(default_factory=list)
    base_classes: list[str] = field(default_factory=list)
    state_fields: list[str] = field(default_factory=list)
    unsupported: list[str] = field(default_factory=list)

    def plain_state_fields(self) -> list[PlainStateField]:
        return [_parse_plain_state_field(item) for item in self.state_fields]

    def require_supported(self) -> None:
        problems = list(self.unsupported)
        if not self.com_map:
            problems.append("no BEGIN_COM_MAP entries found")
        if not self.not_aggregatable:
            problems.append(
                "aggregation policy is unsupported: DECLARE_NOT_AGGREGATABLE is required"
            )
        if not self.object_map_clsid:
            problems.append(
                "no OBJECT_ENTRY activation found; internal COM-object construction is not mapped"
            )
        for declaration in self.state_fields:
            try:
                _parse_plain_state_field(declaration)
            except CppComError as exc:
                problems.append(str(exc))
        if problems:
            raise CppComError(
                f"{self.name}: deterministic COM scaffold preflight failed:\n  - "
                + "\n  - ".join(problems)
            )

    def to_json(self) -> str:
        data = asdict(self)
        for key in ("source", "header"):
            data[key] = str(data[key])
        return json.dumps(data, indent=2, sort_keys=True)


_CLASS_HEAD = re.compile(r"\bclass\s+(?P<name>[A-Za-z_]\w*)\s*(?P<bases>:[^{]+)?\{")
_BEGIN_MAP = re.compile(r"\bBEGIN_COM_MAP\s*\(\s*(\w+)\s*\)")
_END_MAP = re.compile(r"\bEND_COM_MAP\s*\(\s*\)")
_MAP_ENTRY = re.compile(r"\b(COM_INTERFACE_ENTRY(?:2)?(?:_IID)?)\s*\(([^)]*)\)")
_UNSUPPORTED_MAP = re.compile(
    r"\b(COM_INTERFACE_ENTRY_(?:CHAIN|FUNC|FUNC_BLIND|AGGREGATE|BREAK|NOINTERFACE))\s*\("
)
_METHOD_DECL = re.compile(
    r"(?:STDMETHOD(?:IMP)?|virtual\s+[^;()]+|[A-Za-z_]\w*(?:\s+WINAPI)?)\s*"
    r"(?:\([^;{}]*\)|[A-Za-z_]\w*\s*\([^;{}]*\))\s*(?:override\s*)?;"
)


def _balanced_block(text: str, open_brace: int) -> tuple[str, int]:
    depth = 0
    for i in range(open_brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[open_brace + 1 : i], i
    raise CppComError("unbalanced C++ class/interface block")


def _find_class_text(header: Path, class_name: str) -> str:
    text = header.read_text(encoding="utf-8", errors="replace")
    for match in _CLASS_HEAD.finditer(text):
        if match.group("name") == class_name:
            body, _ = _balanced_block(text, match.end() - 1)
            return body
    raise CppComError(f"{class_name}: class definition not found in {header}")


def _class_bases(header: Path, class_name: str) -> list[str]:
    text = header.read_text(encoding="utf-8", errors="replace")
    for match in _CLASS_HEAD.finditer(text):
        if match.group("name") != class_name:
            continue
        raw = (match.group("bases") or "").lstrip(":").strip()
        raw = re.sub(r"/\*.*?\*/|//[^\n]*", " ", raw, flags=re.S)
        return _split_args(raw) if raw else []
    raise CppComError(f"{class_name}: class definition not found in {header}")


def find_class_header(source: Path, class_name: str) -> Path:
    """Find the source/header containing the actual class definition.

    shell32 has both classes defined directly in their ``.cpp`` and forward
    declarations spread across sibling headers. A bare ``class Name`` grep
    confuses those declarations with definitions and makes reruns ambiguous.
    """

    def defines(path: Path) -> bool:
        text = path.read_text(encoding="utf-8", errors="replace")
        return any(match.group("name") == class_name for match in _CLASS_HEAD.finditer(text))

    if defines(source):
        return source

    hits = [header for header in sorted(source.parent.glob("*.h")) if defines(header)]
    if not hits:
        shell32_root = next((p for p in source.parents if p.name == "shell32"), None)
        if shell32_root is not None and shell32_root != source.parent:
            hits = [header for header in sorted(shell32_root.rglob("*.h")) if defines(header)]
    if not hits:
        raise CppComError(f"{class_name}: no defining C++ source/header found from {source}")
    if len(hits) > 1:
        raise CppComError(
            f"{class_name}: ambiguous defining header: " + ", ".join(str(p) for p in hits)
        )
    return hits[0]


def _split_args(text: str) -> list[str]:
    out, start, depth = [], 0, 0
    for i, char in enumerate(text):
        if char in "(<[":
            depth += 1
        elif char in ")>]":
            depth -= 1
        elif char == "," and depth == 0:
            out.append(text[start:i].strip())
            start = i + 1
    tail = text[start:].strip()
    if tail:
        out.append(tail)
    return out


def _parse_com_map(body: str, class_name: str) -> tuple[list[ComMapEntry], list[str]]:
    begin = _BEGIN_MAP.search(body)
    if not begin or begin.group(1) != class_name:
        return [], []
    end = _END_MAP.search(body, begin.end())
    if not end:
        raise CppComError(f"{class_name}: BEGIN_COM_MAP has no END_COM_MAP")
    map_text = body[begin.end() : end.start()]
    map_text = re.sub(r"/\*.*?\*/|//[^\n]*", " ", map_text, flags=re.S)
    unsupported = [
        f"unsupported ATL map macro {m.group(1)}" for m in _UNSUPPORTED_MAP.finditer(map_text)
    ]
    entries: list[ComMapEntry] = []
    for match in _MAP_ENTRY.finditer(map_text):
        macro, args = match.group(1), _split_args(match.group(2))
        if macro == "COM_INTERFACE_ENTRY" and len(args) == 1:
            entries.append(ComMapEntry(f"IID_{args[0]}", args[0], macro))
        elif macro == "COM_INTERFACE_ENTRY_IID" and len(args) == 2:
            entries.append(ComMapEntry(args[0], args[1], macro))
        elif macro == "COM_INTERFACE_ENTRY2" and len(args) == 2:
            entries.append(ComMapEntry(f"IID_{args[0]}", args[0], macro))
        elif macro == "COM_INTERFACE_ENTRY2_IID" and len(args) == 3:
            entries.append(ComMapEntry(args[0], args[1], macro))
        else:
            unsupported.append(f"unrecognized {macro} argument shape: {match.group(2).strip()}")
    return entries, unsupported


def _state_fields(body: str) -> list[str]:
    """Conservatively list class data declarations outside the COM-map/method surface."""
    # Comments are not syntax.  Leaving them in previously made trailing prose such
    # as ``// CControlPanelFolder does it for us`` look like an object field.
    stripped = re.sub(r"/\*.*?\*/|//[^\n]*", " ", body, flags=re.S)
    stripped = re.sub(r"BEGIN_COM_MAP\b.*?END_COM_MAP\s*\(\s*\)", "", stripped, flags=re.S)

    # Remove in-class method definitions before splitting declarations at ';'.
    # A body can itself contain semicolons, and treating those fragments as class
    # declarations produces both false positives and (worse) architecture-dependent
    # layout decisions.  We only consume a balanced block when its declaration
    # prefix contains a parameter list, so nested type declarations remain visible.
    inline_method = re.compile(
        r"(?:^|[;}])\s*(?:template\s*<[^>{}]*>\s*)?[^;{}]*\([^;{}]*\)"
        r"\s*(?:const\s*)?(?:noexcept\s*)?(?:override\s*)?\{",
        re.M,
    )
    while match := inline_method.search(stripped):
        open_brace = match.end() - 1
        _, close_brace = _balanced_block(stripped, open_brace)
        stripped = stripped[: match.start()] + ";" + stripped[close_brace + 1 :]

    stripped = _METHOD_DECL.sub("", stripped)
    stripped = re.sub(r"\b(?:public|private|protected)\s*:\s*", "", stripped)
    stripped = re.sub(r"\bDECLARE_[A-Z0-9_]+\s*(?:\([^)]*\))?", "", stripped)
    fields = []
    for statement in stripped.split(";"):
        statement = " ".join(statement.split())
        if not statement or "(" in statement or ")" in statement or statement == "override":
            continue
        # Access labels and macro debris contain no plausible declarator.
        if re.search(r"\b[A-Za-z_]\w*\s*(?:\[[^]]+\])?$", statement):
            fields.append(statement)
    return fields


def _parse_plain_state_field(declaration: str) -> PlainStateField:
    """Parse the deliberately small C-layout state subset.

    A Rust-owned COM allocation does not contain ATL base-class state, so plain C
    scalar/pointer fields can be represented directly and zero-initialized before
    the translated constructor runs. C++ objects, references, arrays, bitfields,
    initializers, and multi-declarator statements retain hidden lifetime/layout
    semantics and must remain a loud preflight failure.
    """
    text = " ".join(declaration.split())
    if any(token in text for token in ("<", ">", "&", "[", "]", "{", "}", "=", ",", ":")):
        raise CppComError(
            f"state field needs an explicit C++ layout/lifetime mapping: {declaration}"
        )
    match = re.fullmatch(r"(?P<type>.+?)(?P<name>[A-Za-z_]\w*)", text)
    if match is None:
        raise CppComError(f"cannot parse object state declaration: {declaration}")
    c_type = match.group("type").strip()
    name = match.group("name")
    if not c_type or "(" in c_type or ")" in c_type:
        raise CppComError(f"cannot parse object state declaration: {declaration}")
    base = re.sub(r"\b(?:const|volatile)\b", "", c_type).replace("*", "")
    base = " ".join(base.split())
    scalar_types = _RUST_SCALARS | {
        "BOOL": "",
        "BOOLEAN": "",
        "BYTE": "",
        "CHAR": "",
        "DWORD": "",
        "DWORD64": "",
        "HRESULT": "",
        "INT": "",
        "INT_PTR": "",
        "LONG": "",
        "LONG_PTR": "",
        "LONGLONG": "",
        "SIZE_T": "",
        "UINT": "",
        "UINT_PTR": "",
        "ULONG": "",
        "ULONG_PTR": "",
        "ULONGLONG": "",
        "USHORT": "",
        "WCHAR": "",
        "WORD": "",
    }
    pointer_typedef = bool(re.fullmatch(r"(?:L?P(?:C)?|H)[A-Z][A-Z0-9_]*", base))
    if "*" not in c_type and base not in scalar_types and not pointer_typedef:
        raise CppComError(f"state field type is not proven C scalar/pointer state: {declaration}")
    return PlainStateField(declaration=text, c_type=c_type, name=name)


def analyze_class(
    source: Path,
    class_name: str,
    *,
    object_map_source: Path | None = None,
    method_names: list[str] | None = None,
) -> CppComClass:
    source = source.resolve()
    header = find_class_header(source, class_name)
    body = _find_class_text(header, class_name)
    entries, unsupported = _parse_com_map(body, class_name)
    bases = _class_bases(header, class_name)
    interfaces = {entry.interface for entry in entries}
    for raw_base in bases:
        base = re.sub(r"\b(?:public|private|protected|virtual)\b", "", raw_base)
        base = " ".join(base.split())
        if (
            base.startswith("CComCoClass<")
            or base.startswith("CComObjectRootEx<")
            or base in interfaces
        ):
            continue
        unsupported.append(
            f"custom/inherited base class {base} needs deterministic inherited layout mapping"
        )
    for lifecycle in ("FinalConstruct", "FinalRelease"):
        if re.search(rf"\b{lifecycle}\s*\(", body):
            unsupported.append(
                f"custom ATL lifecycle method {lifecycle} needs deterministic factory wiring"
            )
    resource = re.search(r"\bDECLARE_REGISTRY_RESOURCEID\s*\(\s*([^)\s]+)\s*\)", body)
    not_agg = bool(re.search(r"\bDECLARE_NOT_AGGREGATABLE\s*\(", body))
    clsid = None
    if object_map_source and object_map_source.is_file():
        object_map = object_map_source.read_text(encoding="utf-8", errors="replace")
        hit = re.search(
            rf"\bOBJECT_ENTRY\s*\(\s*([^,]+?)\s*,\s*"
            rf"{re.escape(class_name)}(?:RustEntry)?\s*\)",
            object_map,
        )
        if hit:
            clsid = hit.group(1).strip()
    return CppComClass(
        name=class_name,
        source=source,
        header=header,
        com_map=entries,
        registry_resource=resource.group(1) if resource else None,
        not_aggregatable=not_agg,
        object_map_clsid=clsid,
        method_names=method_names or [],
        base_classes=bases,
        state_fields=_state_fields(body),
        unsupported=unsupported,
    )


_VTBL_BLOCK = re.compile(
    r"typedef\s+struct\s+(?P<name>[A-Za-z_]\w*Vtbl)\s*\{(?P<body>.*?)\}\s*(?P=name)\s*;",
    re.S,
)
_VTBL_METHOD = re.compile(
    r"(?P<ret>[A-Za-z_]\w*(?:\s*\*)*)\s*"
    r"\(\s*STDMETHODCALLTYPE\s*\*(?P<name>[A-Za-z_]\w*)\s*\)\s*"
    r"\((?P<params>.*?)\)\s*;",
    re.S,
)
_DECLARE_METHOD = re.compile(
    r"STDMETHOD(?:(?:_\s*\(\s*(?P<ret>[^,]+?)\s*,\s*(?P<name2>\w+)\s*\))"
    r"|(?:\s*\(\s*(?P<name1>\w+)\s*\)))\s*"
    r"\((?P<params>.*?)\)\s*PURE\s*;",
    re.S,
)
_SAL_ANNOTATION = re.compile(r"\b_[A-Za-z][A-Za-z0-9_]*_(?:\s*\([^)]*\))?\s*")


def _parse_param(text: str, index: int) -> tuple[str, str]:
    text = " ".join(text.replace("CONST", "const").split())
    if text in ("", "void"):
        return "", ""
    match = re.match(r"(?P<type>.*?)(?P<name>[A-Za-z_]\w*)$", text)
    if not match:
        return text, f"arg{index}"
    c_type = match.group("type").strip()
    name = match.group("name")
    while name.startswith("*"):
        c_type += " *"
        name = name[1:]
    return c_type, name


def parse_widl_interface(header: Path, interface: str) -> InterfaceSpec:
    """Read an exact flattened interface vtable from a compiler-facing SDK header.

    WIDL's generated ``InterfaceVtbl`` struct is preferred. A small number of
    legacy shell interfaces use ``DECLARE_INTERFACE_`` instead; that C macro
    surface explicitly lists every inherited slot, so it is equally suitable
    deterministic input once SAL/``THIS_`` spelling is normalized.
    """
    text = header.read_text(encoding="utf-8", errors="replace")
    wanted = f"{interface}Vtbl"
    block = next((m for m in _VTBL_BLOCK.finditer(text) if m.group("name") == wanted), None)
    methods = []
    if block is not None:
        for match in _VTBL_METHOD.finditer(block.group("body")):
            params = []
            for i, raw in enumerate(_split_args(match.group("params"))):
                c_type, name = _parse_param(raw, i)
                if c_type:
                    params.append((c_type, name))
            methods.append(
                InterfaceMethod(
                    name=match.group("name"),
                    return_type=" ".join(match.group("ret").split()),
                    params=tuple(params),
                )
            )
    else:
        legacy = re.search(
            rf"DECLARE_INTERFACE_\s*\(\s*{re.escape(interface)}\s*,[^)]*\)\s*"
            r"\{(?P<body>.*?)\}\s*;",
            text,
            flags=re.S,
        )
        if legacy is None:
            raise CppComError(
                f"{interface}: neither {wanted} nor DECLARE_INTERFACE_ definition found in "
                f"SDK header {header}"
            )
        body = re.sub(r"/\*.*?\*/|//[^\n]*", " ", legacy.group("body"), flags=re.S)
        for match in _DECLARE_METHOD.finditer(body):
            raw_params = match.group("params").strip()
            has_this = raw_params == "THIS" or raw_params.startswith("THIS_")
            raw_params = re.sub(r"^THIS_?\s*", "", raw_params)
            raw_params = _SAL_ANNOTATION.sub("", raw_params)
            params = [(f"{interface} *", "This")] if has_this else []
            for i, raw in enumerate(_split_args(raw_params), len(params)):
                c_type, name = _parse_param(raw, i)
                if c_type:
                    params.append((c_type, name))
            methods.append(
                InterfaceMethod(
                    name=match.group("name1") or match.group("name2"),
                    return_type=" ".join((match.group("ret") or "HRESULT").split()),
                    params=tuple(params),
                )
            )
    if not methods:
        raise CppComError(f"{interface}: no vtable methods parsed from {header}")
    # Keep generated output byte-identical across worktrees/build roots.
    return InterfaceSpec(interface, tuple(methods), header.name)


_RUST_SCALARS = {
    "void": "core::ffi::c_void",
    "char": "i8",
    "unsigned char": "u8",
    "short": "i16",
    "unsigned short": "u16",
    "int": "i32",
    "unsigned int": "u32",
    "long": "i32",
    "unsigned long": "u32",
}


def c_type_to_rust(c_type: str) -> str:
    """Mechanical C declarator mapping for generated WIDL method signatures."""
    text = " ".join(c_type.replace("CONST", "const").split())
    is_const = text.startswith("const ")
    text = re.sub(r"^const\s+", "", text)
    stars = text.count("*")
    base = " ".join(text.replace("*", " ").split())
    rust = _RUST_SCALARS.get(base, base)
    for depth in range(stars):
        qualifier = "const" if is_const and depth == 0 else "mut"
        rust = f"*{qualifier} {rust}"
    return rust


_CPP_COM_DEBUG_MACROS = {"ERR", "FIXME", "MESSAGE", "TRACE", "WARN"}
_CPP_COM_LITERAL_WORDS = {"FALSE", "NULL", "TRUE"}
_CPP_COM_TYPE_WORDS = {
    "BOOL",
    "BYTE",
    "DWORD",
    "HRESULT",
    "INT",
    "LONG",
    "LSTATUS",
    "SIZE_T",
    "UINT",
    "ULONG",
    "ULONG_PTR",
    "WCHAR",
    "WORD",
}
_INTEGER_LITERAL = re.compile(
    r"(?<![\w.])(?P<number>0[xX][0-9A-Fa-f]+|0[oO][0-7]+|\d+)"
    r"(?:[uUlL]+|[iu](?:8|16|32|64|128|size))?(?![\w.])"
)
_CPP_WIDE_LITERAL = re.compile(r'L"(?:\\.|[^"\\])*"')
_CPP_WIDE_CHAR = re.compile(r"L'(?:\\.|[^'\\])+'")


def _wide_literal_units(token: str) -> tuple[int, ...]:
    try:
        value = ast.literal_eval(token[1:])
    except (SyntaxError, ValueError) as exc:
        raise CppComError(f"unsupported C wide string literal {token}: {exc}") from exc
    if not isinstance(value, str):
        raise CppComError(f"unsupported C wide string literal {token}")
    encoded = value.encode("utf-16-le", "surrogatepass")
    units = tuple(encoded[i] | (encoded[i + 1] << 8) for i in range(0, len(encoded), 2))
    return (*units, 0)


def _wide_literal_name(token: str) -> str:
    units = _wide_literal_units(token)
    payload = b"".join(unit.to_bytes(2, "little") for unit in units)
    return "CPP_COM_WSTR_" + hashlib.sha256(payload).hexdigest()[:12].upper()


def _wide_char_unit(token: str) -> int:
    try:
        value = ast.literal_eval(token[1:])
    except (SyntaxError, ValueError) as exc:
        raise CppComError(f"unsupported C wide character literal {token}: {exc}") from exc
    if not isinstance(value, str) or len(value) != 1:
        raise CppComError(f"unsupported C wide character literal {token}")
    encoded = value.encode("utf-16-le", "surrogatepass")
    if len(encoded) != 2:
        raise CppComError(f"C wide character literal is not one UTF-16 code unit: {token}")
    return encoded[0] | (encoded[1] << 8)


def _rewrite_wide_literals(text: str) -> str:
    text = _CPP_WIDE_LITERAL.sub(
        lambda match: f"{_wide_literal_name(match.group(0))}.as_ptr()", text
    )
    return _CPP_WIDE_CHAR.sub(lambda match: f"0x{_wide_char_unit(match.group(0)):04X}u16", text)


def render_wide_literal_statics(raw_deps: list[FunctionDep], plan: list[FunctionDep]) -> str:
    """Render only the UTF-16 literals referenced by the selected helper closure."""
    selected = set(
        re.findall(r"\bCPP_COM_WSTR_[0-9A-F]{12}\b", "\n".join(d.body_text for d in plan))
    )
    by_name: dict[str, tuple[int, ...]] = {}
    for dep in raw_deps:
        for match in _CPP_WIDE_LITERAL.finditer(dep.body_text):
            token = match.group(0)
            name = _wide_literal_name(token)
            if name in selected:
                by_name[name] = _wide_literal_units(token)
    missing = selected - set(by_name)
    if missing:
        raise CppComError("lost deterministic wide literal source: " + ", ".join(sorted(missing)))
    lines = ["// Deterministic UTF-16 literals from the selected C++ helper closure."]
    for name, units in sorted(by_name.items()):
        values = ", ".join(f"0x{unit:04X}" for unit in units)
        lines.append(f"pub static {name}: [WCHAR; {len(units)}] = [{values}];")
    return "\n".join(lines) if by_name else ""


def _integer_values(text: str) -> Counter[int]:
    values: Counter[int] = Counter()
    for match in _INTEGER_LITERAL.finditer(text):
        token = match.group("number")
        if token.lower().startswith("0x"):
            base = 16
        elif token.lower().startswith("0o") or (
            len(token) > 1 and token.startswith("0") and not re.search(r"[89]", token)
        ):
            base = 8
        else:
            base = 10
        values[int(token[2:] if token.lower().startswith("0o") else token, base)] += 1
    return values


def normalize_worker_body(rust: str, expected_signature: str | None = None) -> str:
    """Normalize only syntax the deterministic shell32 substrate already owns."""
    names = "|".join(sorted(_CPP_COM_DEBUG_MACROS))
    rust = re.sub(r"^\s*```(?:rust)?\s*|\s*```\s*$", "", rust, flags=re.I)
    # Cheap models sometimes turn a C macro into a variadic extern declaration.
    # The crate supplies macro_rules definitions, so the declaration is both wrong
    # and unnecessary. Accept only a block containing that one declaration.
    rust = re.sub(
        rf'\s*extern\s+"(?:C|system)"\s*\{{\s*fn\s+(?:{names})\s*\([^;]*;\s*\}}\s*',
        "\n    ",
        rust,
        flags=re.S,
    )
    rust = re.sub(rf"\b({names})\s*\(", r"\1!(", rust)
    # Body workers never own FFI declarations. Keep their calls, but remove
    # model-invented local declaration blocks so the deterministic module-level
    # substrate remains the single ABI authority. The audit still rejects any
    # declaration shape this narrow matcher cannot prove is declarations-only.
    rust = re.sub(
        r'\s*extern\s+"(?:C|system)"\s*\{(?:\s*(?:pub\s+)?fn\s+[^;]+;)+\s*\}\s*',
        "\n    ",
        rust,
        flags=re.S,
    )
    rust = re.sub(r"\bcrate::(?:types|shellcom)::", "", rust)
    # HRESULT is a C/Rust scalar alias, not a tuple struct constructor.
    rust = re.sub(r"\bHRESULT\s*\(\s*([A-Z][A-Z0-9_]*)\s*\)", r"\1", rust)
    # A model may name a constant array bound with `let`, which parses but does
    # not compile in a type-level array length. Promote only bindings whose RHS
    # is provably integer/operator/cast syntax and which are actually consumed
    # as an array bound. Runtime-derived bindings remain untouched.
    constant_binding = re.compile(r"\blet\s+([A-Za-z_]\w*)\s*:\s*usize\s*=\s*([^;]+);")
    for match in reversed(list(constant_binding.finditer(rust))):
        name, expression = match.groups()
        tail = rust[match.end() :]
        lowercase_words = re.findall(r"\b[a-z_][A-Za-z0-9_]*\b", expression)
        constant_only = all(word in {"as", "usize"} for word in lowercase_words)
        if constant_only and re.search(rf"\[[^;\]]*;\s*{re.escape(name)}\s*\]", tail):
            replacement = f"const {name}: usize = {expression};"
            rust = rust[: match.start()] + replacement + rust[match.end() :]
    if expected_signature is not None:
        # Rust warns about unused parameters, so cheap models often rename an
        # ABI-owned `param` to `_param`. The deterministic wrapper fixes the
        # worker signature; restore those exact names in signature and body.
        expected_params = re.findall(r"\b([A-Za-z_]\w*)\s*:", expected_signature)
        for name in expected_params:
            rust = re.sub(rf"\b_{re.escape(name)}\b", name, rust)
        expected_name_match = re.search(r"\bfn\s+([A-Za-z_]\w*)\s*\(", expected_signature)
        if expected_name_match is not None:
            expected_name = expected_name_match.group(1)
            actual = re.search(
                rf"\b(?:pub\s+)?(?:unsafe\s+)?fn\s+{re.escape(expected_name)}\s*\(",
                rust,
            )
            if actual is not None:
                opening = rust.find("(", actual.start())
                closing = _matching_paren(rust, opening)
                brace = rust.find("{", closing + 1)
                actual_params = [
                    match.group(1)
                    for item in _split_args(rust[opening + 1 : closing])
                    if (match := re.match(r"\s*([A-Za-z_]\w*)\s*:", item))
                ]
                # The model owns no part of the worker surface. If its parameter
                # list still names the generated slots in order, replace all of
                # its type/return/qualifier spellings with the exact authority.
                if brace >= 0 and actual_params == expected_params:
                    rust = rust[: actual.start()] + expected_signature + " " + rust[brace:]
    return rust


def worker_signature(
    name: str, class_name: str | None, params: list[tuple[str, str]], return_type: str
) -> str:
    """Render the deterministic body-worker signature.

    COM methods receive the recovered object pointer first. Same-file helper
    functions are ordinary Rust workers with signatures derived directly from
    their C definitions; the model owns neither surface.
    """
    rendered = [f"this: *mut {class_name}"] if class_name else []
    rendered.extend(
        f"{param_name or f'arg{i}'}: {c_type_to_rust(c_type)}"
        for i, (c_type, param_name) in enumerate(params)
    )
    ret = c_type_to_rust(return_type) if return_type else "()"
    return f"pub unsafe fn {name}_body({', '.join(rendered)}) -> {ret}"


def _masked_cpp(text: str) -> str:
    """Blank comments and literals while preserving byte/character offsets."""
    out = list(text)
    i = 0
    while i < len(text):
        if text.startswith("//", i):
            end = text.find("\n", i + 2)
            end = len(text) if end < 0 else end
            for pos in range(i, end):
                out[pos] = " "
            i = end
        elif text.startswith("/*", i):
            end = text.find("*/", i + 2)
            end = len(text) if end < 0 else end + 2
            for pos in range(i, end):
                if out[pos] != "\n":
                    out[pos] = " "
            i = end
        elif text[i] in ('"', "'"):
            quote = text[i]
            i += 1
            while i < len(text):
                if text[i] == "\\":
                    out[i] = " "
                    if i + 1 < len(text):
                        out[i + 1] = " "
                    i += 2
                    continue
                if text[i] == quote:
                    i += 1
                    break
                if out[i] != "\n":
                    out[i] = " "
                i += 1
        else:
            i += 1
    return "".join(out)


def _matching_paren(text: str, opening: int) -> int:
    depth = 0
    for pos in range(opening, len(text)):
        if text[pos] == "(":
            depth += 1
        elif text[pos] == ")":
            depth -= 1
            if depth == 0:
                return pos
    raise CppComError("unbalanced helper call argument list")


def strip_debug_statements(text: str) -> str:
    """Remove complete shell32 debug-macro statements from a model-only view."""
    masked = _masked_cpp(text)
    spans = []
    for match in re.finditer(r"\b(?:TRACE|FIXME|WARN|ERR|MESSAGE)\s*\(", masked):
        before = masked[: match.start()].rstrip()
        if before and before[-1] not in "{;}":
            continue
        opening = masked.find("(", match.start())
        try:
            closing = _matching_paren(masked, opening)
        except CppComError:
            continue
        end = closing + 1
        while end < len(masked) and masked[end].isspace():
            end += 1
        if end < len(masked) and masked[end] == ";":
            spans.append((match.start(), end + 1))
    for start, end in reversed(spans):
        text = text[:start] + " " * (end - start) + text[end:]
    return text


def _argument_count(text: str, opening: int, closing: int) -> int:
    inner = text[opening + 1 : closing]
    if not inner.strip() or inner.strip() == "void":
        return 0
    depth = 0
    count = 1
    for char in inner:
        # Parentheses/brackets/braces can contain comma expressions. Do not
        # count angle brackets here: in an expression ``x < y`` is much more
        # common than an explicit-template argument containing a comma. The
        # latter stays outside this narrow arity-only overload subset and will
        # fail preflight instead of silently binding the wrong overload.
        if char in "([{":
            depth += 1
        elif char in ")]}":
            if depth:
                depth -= 1
        elif char == "," and depth == 0:
            count += 1
    return count


def _named_call_spans(text: str, names: set[str]) -> list[tuple[int, int, str, int]]:
    """Return identifier spans and arities for selected free-function calls.

    The same syntax covers a function definition's declarator. That is useful:
    helper definitions and every call site are renamed in one deterministic pass.
    """
    masked = _masked_cpp(text)
    spans = []
    for match in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", masked):
        name = match.group(1)
        if name not in names:
            continue
        opening = masked.find("(", match.start(1) + len(name))
        closing = _matching_paren(masked, opening)
        spans.append(
            (match.start(1), match.end(1), name, _argument_count(masked, opening, closing))
        )
    return spans


def prepare_helper_closure(deps: list[FunctionDep], class_name: str) -> list[FunctionDep]:
    """Select and bind the exact same-file helper closure for one COM class.

    C++ overload resolution is deliberately not reimplemented. The accepted
    subset is stronger and easy to audit: local overloads must have distinct
    arities. Every helper signature comes from the scanner, and every selected
    call is rewritten to its collision-free ``<target>_body`` Rust name before
    the model sees it. Same-name/same-arity overloads hard-fail before payment.
    """
    from .callgraph import FunctionDep, topological_sort

    class_deps = [dep for dep in deps if dep.cpp_class == class_name]
    if not class_deps:
        raise CppComError(f"{class_name}: no extracted method bodies")
    names = {dep.name for dep in class_deps}
    if f"{class_name}_ctor" not in names:
        class_deps.append(
            FunctionDep(
                name=f"{class_name}_ctor",
                cpp_class=class_name,
                cpp_method=class_name,
                body_text=f"{class_name}::{class_name}() {{}}",
            )
        )
    if f"{class_name}_dtor" not in names:
        class_deps.append(
            FunctionDep(
                name=f"{class_name}_dtor",
                cpp_class=class_name,
                cpp_method=f"~{class_name}",
                body_text=f"{class_name}::~{class_name}() {{}}",
            )
        )

    # Only internal-linkage functions belong to a class's body closure. An
    # external-linkage function in the same translation unit is still a module
    # ABI boundary (for example SHCreateShellItem); absorbing it would change
    # ownership, exports, and possibly activation semantics.
    free_deps = [dep for dep in deps if dep.cpp_class is None and dep.is_static]
    by_source: dict[str, list[FunctionDep]] = {}
    for dep in free_deps:
        # ``name`` is the scanner's stable target name. Overload suffixes are
        # assigned in declaration order; recover the spelling from the source
        # declarator instead of guessing by stripping ``_2``.
        match = re.search(r"\b([A-Za-z_]\w*)\s*\(", _masked_cpp(dep.body_text))
        source_name = match.group(1) if match else dep.name
        by_source.setdefault(source_name, []).append(dep)

    binding: dict[tuple[str, int], FunctionDep] = {}
    for source_name, candidates in by_source.items():
        for dep in candidates:
            key = (source_name, len(dep.params))
            if key in binding:
                other = binding[key]
                raise CppComError(
                    f"{class_name}: local overload {source_name}/{len(dep.params)} is "
                    f"ambiguous between {other.name} and {dep.name}; deterministic helper "
                    "binding currently requires distinct arities"
                )
            binding[key] = dep

    helper_names = set(by_source)
    selected: dict[str, FunctionDep] = {}
    queue = list(class_deps)
    while queue:
        caller = queue.pop(0)
        for _, _, source_name, arity in _named_call_spans(caller.body_text, helper_names):
            target = binding.get((source_name, arity))
            if target is None:
                available = sorted(n for name, n in binding if name == source_name)
                raise CppComError(
                    f"{class_name}: cannot bind local helper call {source_name}/{arity}; "
                    f"available arities: {available}"
                )
            if target.name not in selected:
                selected[target.name] = target
                queue.append(target)

    plan = list(selected.values()) + class_deps
    rewritten: list[FunctionDep] = []
    for dep in plan:
        body = _rewrite_wide_literals(dep.body_text)
        local_targets: set[str] = set()
        spans = _named_call_spans(body, helper_names)
        for start, end, source_name, arity in reversed(spans):
            target = binding.get((source_name, arity))
            if target is None:
                continue
            body = body[:start] + f"{target.name}_body" + body[end:]
            # The helper definition's own declarator is not a callee. Removing
            # self is harmless for genuine recursion; _format_callee_context
            # intentionally omits it too.
            if target.name != dep.name:
                local_targets.add(target.name)
        external = {name for name in dep.callees if name not in helper_names}
        rewritten.append(replace(dep, body_text=body, callees=external | local_targets))

    return topological_sort(rewritten)


def apply_interface_method_signatures(
    deps: list[FunctionDep], class_name: str, interfaces: list[InterfaceSpec]
) -> list[FunctionDep]:
    """Replace parsed COM method types with the flattened vtable authority.

    C++ error recovery can find an out-of-line ``WINAPI Class::Method`` body
    while losing its return type. WIDL already defines the exact ABI shape, so
    scanner types must never win for interface methods. Source parameter names
    are retained because the C++ body refers to them.
    """
    methods: dict[str, InterfaceMethod] = {}
    for interface in interfaces:
        for method in interface.methods:
            if method.name in ("QueryInterface", "AddRef", "Release"):
                continue
            prior = methods.get(method.name)
            # An inherited method is repeated in flattened child vtables with
            # only its receiver type changed (IPersist* vs IPersistFile*).
            # Workers receive the owning class pointer, so compare the ABI
            # shape after removing that deterministic interface receiver.
            shape = (method.return_type, method.params[1:])
            prior_shape = None if prior is None else (prior.return_type, prior.params[1:])
            if prior is not None and prior_shape != shape:
                raise CppComError(
                    f"{class_name}: interface method {method.name} has conflicting vtable "
                    "signatures across COM-map interfaces"
                )
            methods[method.name] = method

    out = []
    for dep in deps:
        method = methods.get(dep.cpp_method or "") if dep.cpp_class == class_name else None
        if method is None:
            out.append(dep)
            continue
        vtable_params = list(method.params[1:])
        if len(vtable_params) != len(dep.params):
            raise CppComError(
                f"{class_name}::{dep.cpp_method}: scanner found {len(dep.params)} parameter(s), "
                f"but the flattened vtable requires {len(vtable_params)}"
            )
        params = [
            (vtable_type, source_name or vtable_name)
            for (vtable_type, vtable_name), (_, source_name) in zip(
                vtable_params, dep.params, strict=True
            )
        ]
        out.append(replace(dep, return_type=method.return_type, params=params))
    return out


def lower_trivial_worker(c_body: str, expected_signature: str) -> str | None:
    """Lower exact no-state stub bodies without spending an LLM call.

    This deliberately recognizes only the whole-body shapes it can prove:
    ``UNIMPLEMENTED;`` and ``UNIMPLEMENTED; return SYMBOLIC_HRESULT;``.
    Any additional statement stays on the audited model path.
    """
    open_brace, close_brace = c_body.find("{"), c_body.rfind("}")
    if open_brace < 0 or close_brace <= open_brace:
        return None
    # An empty constructor body is not necessarily empty behavior: its
    # initializer list runs first. The deterministic factory already provides
    # zeroed plain-C state, so only exact zero initializers may collapse to a
    # no-op. Non-zero/member-construction forms stay on the audited model path.
    head = re.sub(r"/\*.*?\*/|//[^\n]*", " ", c_body[:open_brace], flags=re.S)
    initializer_mark = re.search(r"\)\s*:\s*", head)
    if initializer_mark is not None:
        initializer = head[initializer_mark.end() :].strip()
        for item in _split_args(initializer):
            match = re.fullmatch(r"[A-Za-z_]\w*\s*\(\s*([^()]*)\s*\)", item)
            if match is None or match.group(1).strip() not in {"0", "NULL", "nullptr", "FALSE"}:
                return None
    body = c_body[open_brace + 1 : close_brace]
    # TRACE/FIXME/WARN/ERR/MESSAGE are no-op macro_rules in the shell32 Rust
    # substrate. Remove only complete call statements before proving the
    # remainder is one of the closed trivial shapes below.
    body = strip_debug_statements(body)
    body = re.sub(r"/\*.*?\*/|//[^\n]*", " ", body, flags=re.S)
    body = re.sub(r"\bUNIMPLEMENTED\s*;", " ", body)
    body = " ".join(body.split())
    if not body:
        return expected_signature + " {}"
    returned = re.fullmatch(r"return\s+([A-Z][A-Z0-9_]*)\s*;", body)
    if returned and returned.group(1) not in _CPP_COM_LITERAL_WORDS:
        return f"{expected_signature} {{ {returned.group(1)} }}"
    out_assignment = re.fullmatch(
        r"\*\s*([A-Za-z_]\w*)\s*=\s*([A-Z][A-Z0-9_]*)\s*;\s*"
        r"return\s+([A-Z][A-Z0-9_]*)\s*;",
        body,
    )
    if out_assignment:
        pointer, value, result = out_assignment.groups()
        return f"{expected_signature} {{ *{pointer} = {value}; {result} }}"
    return None


def _canonical_signature(signature: str) -> str:
    signature = re.sub(r"\bcrate::(?:types|shellcom)::", "", signature)
    signature = re.sub(r"\s+", " ", signature).strip()
    signature = re.sub(r"\(\s+", "(", signature)
    signature = re.sub(r",\s*\)", ")", signature)
    signature = re.sub(r"\s+\)", ")", signature)
    # An omitted unit return and an explicit `-> ()` are the same Rust signature.
    return re.sub(r"\s*->\s*\(\)\s*$", "", signature)


def audit_worker_body(
    c_body: str, rust: str, expected_name: str, expected_signature: str | None = None
) -> list[str]:
    """Verify the narrow contract for an LLM-translated COM method body.

    This is intentionally not a semantic verifier.  It protects the boundary that
    matters for the hybrid pipeline: the response must remain a plain worker, must
    preserve source-level symbolic constants, and must not attempt to own COM ABI.
    """
    findings: list[str] = []
    if not re.search(rf"\bfn\s+{re.escape(expected_name)}\s*\(", rust):
        findings.append(f"missing exact worker function {expected_name}")
    elif expected_signature is not None:
        match = re.search(
            rf"\bpub\s+unsafe\s+fn\s+{re.escape(expected_name)}\s*\(.*?\)\s*"
            r"(?:->\s*[^\{]+)?(?=\{)",
            rust,
            flags=re.S,
        )
        if not match or _canonical_signature(match.group(0)) != _canonical_signature(
            expected_signature
        ):
            actual = _canonical_signature(match.group(0)) if match else "<unparsed>"
            findings.append(
                "worker signature differs from deterministic contract: "
                f"expected `{_canonical_signature(expected_signature)}`, got `{actual}`"
            )

    forbidden = {
        "#[no_mangle]": "export attribute",
        'extern "': "ABI declaration",
        "QueryInterface": "QueryInterface implementation",
        "CreateInstance": "activation factory",
        ".unwrap()": "Option/panic ABI guess",
    }
    for token, description in forbidden.items():
        if token in rust:
            findings.append(f"worker emitted forbidden {description}: {token}")
    if re.search(r"\b(?:struct|static)\s+[A-Za-z_]\w*Vtbl\b|\b[A-Za-z_]\w*Vtbl\s*\{", rust):
        findings.append("worker emitted forbidden vtable definition")

    # Strings and comments contain printf formats and prose, not program constants.
    open_brace, close_brace = c_body.find("{"), c_body.rfind("}")
    source_body = (
        c_body[open_brace + 1 : close_brace]
        if open_brace >= 0 and close_brace > open_brace
        else c_body
    )
    source = re.sub(r'/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"', " ", source_body, flags=re.S)
    symbolic = set()
    ignored_words = _CPP_COM_DEBUG_MACROS | _CPP_COM_LITERAL_WORDS | _CPP_COM_TYPE_WORDS
    for match in re.finditer(r"\b[A-Z][A-Z0-9_]+\b", source):
        word = match.group(0)
        # Callable macro/function names (FAILED(...), SUCCEEDED(...),
        # HRESULT_FROM_WIN32(...)) are behavior, not constants. The body audit
        # separately preserves their arguments, branches, and numeric literals.
        tail = source[match.end() :]
        callable_token = tail.lstrip().startswith("(")
        start = match.start()
        cast_type = (
            start > 0
            and source[start - 1] == "("
            and (start == 1 or not (source[start - 2].isalnum() or source[start - 2] == "_"))
            and bool(re.match(r"\s*\**\s*\)", tail))
        )
        declaration_type = bool(re.match(r"\s+\**\s*[A-Za-z_]\w*", tail))
        if (
            word not in ignored_words
            and not callable_token
            and not cast_type
            and not declaration_type
        ):
            symbolic.add(word)
    missing = sorted(word for word in symbolic if not re.search(rf"\b{re.escape(word)}\b", rust))
    if missing:
        findings.append("dropped symbolic C constant(s): " + ", ".join(missing))
    redefined = sorted(
        word
        for word in symbolic
        if re.search(rf"\b(?:pub\s+)?const\s+{re.escape(word)}\s*[:=]", rust)
    )
    if redefined:
        findings.append(
            "worker redefined source symbolic constant(s) instead of using substrate: "
            + ", ".join(redefined)
        )

    source_integers = _integer_values(source)
    rust_integers = _integer_values(rust)
    missing_integers = source_integers - rust_integers
    if missing_integers:
        rendered = []
        for value, count in sorted(missing_integers.items()):
            rendered.append(f"{value}" + (f" (x{count})" if count > 1 else ""))
        findings.append("dropped or changed numeric C literal(s): " + ", ".join(rendered))
    return findings


def render_interface(spec: InterfaceSpec) -> str:
    lines = [
        f"// Deterministic from {spec.source}",
        "#[repr(C)]",
        f"pub struct {spec.name} {{",
        f"    pub lpVtbl: *const {spec.name}Vtbl,",
        "}",
        "#[repr(C)]",
        f"pub struct {spec.name}Vtbl {{",
    ]
    for method in spec.methods:
        params = ", ".join(f"{name}: {c_type_to_rust(ty)}" for ty, name in method.params)
        ret = c_type_to_rust(method.return_type)
        lines.append(f'    pub {method.name}: unsafe extern "system" fn({params}) -> {ret},')
    lines.extend(
        [
            "}",
            'extern "C" {',
            f"    pub static IID_{spec.name}: IID;",
            "}",
        ]
    )
    return "\n".join(lines)


def render_class_scaffold(cls: CppComClass, interfaces: list[InterfaceSpec]) -> str:
    """Render the ABI-owned portion. Method worker bodies remain separate LLM output."""
    cls.require_supported()
    by_name = {item.name: item for item in interfaces}
    missing = [entry.interface for entry in cls.com_map if entry.interface not in by_name]
    if missing:
        raise CppComError(f"{cls.name}: missing interface specs: {', '.join(missing)}")

    lines = [
        f"// Deterministic COM scaffold for {cls.name}; method bodies are generated separately.",
        'extern "system" {',
        "    fn GetProcessHeap() -> HANDLE;",
        "    fn HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;",
        "    fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;",
        "    fn Shell32RustObjectLock();",
        "    fn Shell32RustObjectUnlock();",
        "}",
        'extern "C" {',
        "    pub static IID_IUnknown: IID;",
        "}",
        "",
        "#[repr(C)]",
        f"pub struct {cls.name} {{",
    ]
    for entry in cls.com_map:
        lines.append(f"    pub {entry.interface}_iface: {entry.interface},")
    lines.append("    pub ref_count: core::sync::atomic::AtomicI32,")
    for state in cls.plain_state_fields():
        lines.append(f"    pub {state.name}: {c_type_to_rust(state.c_type)},")
    lines.extend(["}", ""])

    for entry in cls.com_map:
        iface = by_name[entry.interface]
        field_name = f"{entry.interface}_iface"
        prefix = f"{cls.name}_{entry.interface}"
        for method in iface.methods:
            params = list(method.params)
            rendered = ", ".join(f"{name}: {c_type_to_rust(ty)}" for ty, name in params)
            ret = c_type_to_rust(method.return_type)
            lines.append(
                f'unsafe extern "system" fn {prefix}_{method.name}({rendered}) -> {ret} {{'
            )
            this_name = params[0][1]
            lines.append(
                f"    let this = ({this_name} as *mut u8)"
                f".sub(core::mem::offset_of!({cls.name}, {field_name})) "
                f"as *mut {cls.name};"
            )
            rest = ", ".join(name for _, name in params[1:])
            if method.name in ("QueryInterface", "AddRef", "Release"):
                target = f"{cls.name}_{method.name}"
            else:
                target = f"{cls.name}_{method.name}_body"
            call = f"{target}(this{', ' if rest else ''}{rest})"
            lines.append(f"    {call}")
            lines.append("}")
        lines.extend(
            [
                "",
                f"pub static {prefix.upper()}_VTBL: {entry.interface}Vtbl = "
                f"{entry.interface}Vtbl {{",
            ]
        )
        for method in iface.methods:
            lines.append(f"    {method.name}: {prefix}_{method.name},")
        lines.extend(["};", ""])

    first = cls.com_map[0]
    lines.extend(
        [
            f"unsafe fn {cls.name}_QueryInterface(",
            f"    this: *mut {cls.name}, riid: REFIID, ppv: *mut *mut core::ffi::c_void,",
            ") -> HRESULT {",
            "    if ppv.is_null() { return E_POINTER; }",
            "    *ppv = core::ptr::null_mut();",
            f"    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || "
            f"cpp_com_iid_equal(riid, core::ptr::addr_of!({first.iid})) {{",
            f"        *ppv = core::ptr::addr_of_mut!((*this).{first.interface}_iface) "
            "as *mut core::ffi::c_void;",
            "    }",
        ]
    )
    for entry in cls.com_map[1:]:
        lines.extend(
            [
                f"    else if cpp_com_iid_equal(riid, core::ptr::addr_of!({entry.iid})) {{",
                f"        *ppv = core::ptr::addr_of_mut!((*this).{entry.interface}_iface) "
                "as *mut core::ffi::c_void;",
                "    }",
            ]
        )
    lines.extend(
        [
            "    else { return E_NOINTERFACE; }",
            f"    {cls.name}_AddRef(this);",
            "    S_OK",
            "}",
            "",
            f"unsafe fn {cls.name}_AddRef(this: *mut {cls.name}) -> ULONG {{",
            "    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1",
            "}",
            "",
            f"unsafe fn {cls.name}_Release(this: *mut {cls.name}) -> ULONG {{",
            "    let refs = (*this).ref_count.fetch_sub(1, "
            "core::sync::atomic::Ordering::SeqCst) - 1;",
            "    if refs == 0 {",
            f"        {cls.name}_dtor_body(this);",
            "        Shell32RustObjectUnlock();",
            "        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);",
            "    }",
            "    refs as ULONG",
            "}",
            "",
            "#[inline]",
            "unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {",
            "    !left.is_null() && !right.is_null()",
            "        && (*left).Data1 == (*right).Data1",
            "        && (*left).Data2 == (*right).Data2",
            "        && (*left).Data3 == (*right).Data3",
            "        && (*left).Data4 == (*right).Data4",
            "}",
            "",
        ]
    )

    lines.extend(
        [
            f"// Activation: {cls.object_map_clsid or '<not found>'} currently maps to "
            f"{cls.name} in shell32.cpp.",
            "// The generated C++ adapter redirects ATL activation to this factory.",
            f'#[no_mangle] pub unsafe extern "system" fn {cls.name}_CreateInstance(',
            "    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,",
            ") -> HRESULT {",
            "    if ppv.is_null() { return E_POINTER; }",
            "    *ppv = core::ptr::null_mut();",
            "    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }",
            f"    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, "
            f"core::mem::size_of::<{cls.name}>()) as *mut {cls.name};",
            "    if this.is_null() { return E_OUTOFMEMORY; }",
            f"    core::ptr::write(this, {cls.name} {{",
        ]
    )
    for entry in cls.com_map:
        prefix = f"{cls.name}_{entry.interface}".upper()
        lines.append(
            f"        {entry.interface}_iface: {entry.interface} {{ "
            f"lpVtbl: core::ptr::addr_of!({prefix}_VTBL) }},"
        )
    lines.extend(
        [
            "        ref_count: core::sync::atomic::AtomicI32::new(1),",
        ]
    )
    for state in cls.plain_state_fields():
        lines.append(f"        {state.name}: core::mem::zeroed(),")
    lines.extend(
        [
            "    });",
            "    Shell32RustObjectLock();",
            f"    {cls.name}_ctor_body(this);",
            f"    let hr = {cls.name}_QueryInterface(this, riid, ppv);",
            f"    {cls.name}_Release(this);",
            "    hr",
            "}",
        ]
    )
    return "\n".join(lines)


def render_object_map_adapter(cls: CppComClass) -> str:
    """C++ glue that keeps ATL registration but delegates object creation to Rust."""
    cls.require_supported()
    # require_supported already proves both the activation entry and the
    # non-aggregatable lifetime contract used by the generated factory.
    adapter = f"{cls.name}RustEntry"
    return "\n".join(
        [
            f'extern "C" HRESULT WINAPI {cls.name}_CreateInstance(',
            "    IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);",
            "",
            f"class {adapter} : public {cls.name}",
            "{",
            "public:",
            "    class _CreatorClass",
            "    {",
            "    public:",
            "        static HRESULT WINAPI CreateInstance(void *pv, REFIID riid, LPVOID *ppv)",
            "        {",
            f"            return {cls.name}_CreateInstance(",
            "                static_cast<IUnknown *>(pv), riid, ppv);",
            "        }",
            "    };",
            "};",
            "",
            f"// ObjectMap replacement: OBJECT_ENTRY({cls.object_map_clsid}, {adapter})",
            "// One-time bridge functions (place after gModule is defined):",
            "#ifndef SHELL32_RUST_OBJECT_LOCK_BRIDGE_DEFINED",
            "#define SHELL32_RUST_OBJECT_LOCK_BRIDGE_DEFINED",
            'extern "C" void WINAPI Shell32RustObjectLock() { gModule.Lock(); }',
            'extern "C" void WINAPI Shell32RustObjectUnlock() { gModule.Unlock(); }',
            "#endif",
        ]
    )


def discover_widl_header(
    repo_root: Path,
    arch: str,
    filename: str = "shobjidl.h",
    *,
    interface: str | None = None,
) -> Path:
    if interface is not None:
        needles = (
            f"typedef struct {interface}Vtbl",
            f"DECLARE_INTERFACE_({interface},",
            f"DECLARE_INTERFACE_ ({interface},",
        )
        roots = [
            repo_root / f"build-{arch}" / "sdk" / "include" / "psdk",
            repo_root / f"build-{arch}-cross" / "sdk" / "include" / "psdk",
            repo_root / "sdk" / "include" / "psdk",
        ]
        hits = []
        for root in roots:
            if not root.is_dir():
                continue
            for candidate in sorted(root.glob("*.h")):
                text = candidate.read_text(encoding="utf-8", errors="replace")
                if any(needle in text for needle in needles):
                    hits.append(candidate)
        if len(hits) == 1:
            return hits[0]
        if len(hits) > 1:
            raise CppComError(
                f"{interface}: flattened vtable is ambiguous across generated headers: "
                + ", ".join(str(path) for path in hits)
            )
        raise CppComError(
            f"{interface}: no flattened vtable definition found in compiler-facing SDK headers "
            f"for {arch}; refusing to infer slot order"
        )

    candidates = [
        repo_root / f"build-{arch}" / "sdk" / "include" / "psdk" / filename,
        repo_root / f"build-{arch}-cross" / "sdk" / "include" / "psdk" / filename,
    ]
    for candidate in candidates:
        if candidate.is_file():
            return candidate
    raise CppComError(
        f"generated SDK header {filename} not found for {arch}; build that arch first or pass "
        "--sdk-header explicitly. Refusing to guess COM slot order."
    )
