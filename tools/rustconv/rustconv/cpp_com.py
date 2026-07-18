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
import subprocess
from collections import Counter, defaultdict
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
    base: str | None = None


@dataclass(frozen=True)
class PlainStateField:
    """One C-layout object field safe to spell directly in the Rust owner."""

    declaration: str
    c_type: str
    name: str
    com_ptr_interface: str | None = None
    owned_cstring: bool = False
    array_len: int | None = None

    @property
    def rust_type(self) -> str:
        if self.com_ptr_interface is not None:
            return f"*mut {self.com_ptr_interface}"
        rust = c_type_to_rust(self.c_type)
        return f"[{rust}; {self.array_len}]" if self.array_len is not None else rust

    @property
    def assignment_helper(self) -> str | None:
        if self.com_ptr_interface is None:
            return None
        return f"cpp_com_assign_{self.name}"

    @property
    def string_assignment_helper(self) -> str | None:
        if not self.owned_cstring:
            return None
        return f"cpp_com_assign_{self.name}"


@dataclass(frozen=True)
class PodStateType:
    """A source-proven C POD embedded by value in the COM owner."""

    name: str
    fields: tuple[PlainStateField, ...]
    packed: bool = False


@dataclass(frozen=True)
class EnumStateType:
    """A source-proven nested C/C++ enum used by object-owned state."""

    name: str
    variants: tuple[tuple[str, int], ...]


@dataclass
class CppComClass:
    name: str
    source: Path
    header: Path
    com_map: list[ComMapEntry] = field(default_factory=list)
    registry_resource: str | None = None
    not_aggregatable: bool = False
    object_map_clsid: str | None = None
    final_construct: bool = False
    method_names: list[str] = field(default_factory=list)
    base_classes: list[str] = field(default_factory=list)
    state_fields: list[str] = field(default_factory=list)
    pod_state_types: list[PodStateType] = field(default_factory=list)
    enum_state_types: list[EnumStateType] = field(default_factory=list)
    array_constants: dict[str, int] = field(default_factory=dict)
    unsupported: list[str] = field(default_factory=list)

    def plain_state_fields(self) -> list[PlainStateField]:
        pod_names = {item.name for item in self.pod_state_types}
        return [
            _parse_plain_state_field(
                item, allowed_pod_types=pod_names, array_constants=self.array_constants
            )
            for item in self.state_fields
        ]

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
                _parse_plain_state_field(
                    declaration,
                    allowed_pod_types={item.name for item in self.pod_state_types},
                    array_constants=self.array_constants,
                )
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


_CLASS_HEAD = re.compile(r"\b(?:class|struct)\s+(?P<name>[A-Za-z_]\w*)\s*(?P<bases>:[^{]+)?\{")
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
    stripped = re.sub(r"^\s*#\s*include[^\n]*$", "", stripped, flags=re.M)
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

    # Nested POD declarations define a type; their members are not additional
    # fields of the outer COM object.  Keep the actual ``volume_info volume;``
    # declaration while removing the nested type block before semicolon
    # splitting.  The type block is independently captured by
    # ``_discover_pod_state_types`` from the unmodified class text.
    nested_type = re.compile(r"\b(?:struct|union|enum)\s+[A-Za-z_]\w*\s*\{")
    while match := nested_type.search(stripped):
        open_brace = stripped.find("{", match.start())
        _, close_brace = _balanced_block(stripped, open_brace)
        semicolon = stripped.find(";", close_brace)
        if semicolon < 0:
            break
        stripped = stripped[: match.start()] + stripped[semicolon + 1 :]

    stripped = _METHOD_DECL.sub("", stripped)
    stripped = re.sub(r"\b(?:public|private|protected)\s*:\s*", "", stripped)
    stripped = re.sub(r"\bDECLARE_[A-Z0-9_]+\s*(?:\([^)]*\))?", "", stripped)
    fields = []
    for statement in stripped.split(";"):
        statement = " ".join(statement.split())
        if not statement or "(" in statement or ")" in statement or statement == "override":
            continue
        # Static declarations do not contribute to an instance layout.  Macro
        # removal can also leave a destructor's leading `~` or disabled `#if`
        # method debris joined to the following static declaration.
        if statement.startswith(("static ", "~", "#")) or " #" in statement:
            continue
        # Access labels and macro debris contain no plausible declarator.
        if re.search(r"\b[A-Za-z_]\w*\s*(?:\[[^]]+\])?$", statement):
            # C permits ``UINT first, second``. There is one base type and two
            # independent scalar fields; expand that proven shape before the
            # single-declarator layout parser. Pointer stars, initializers,
            # bitfields, and per-declarator arrays remain loud.
            multiple = re.fullmatch(
                r"(?P<type>[A-Za-z_]\w*(?:\s+[A-Za-z_]\w*)*)\s+"
                r"(?P<names>[A-Za-z_]\w*(?:\s*,\s*[A-Za-z_]\w*)+)",
                statement,
            )
            if multiple is not None:
                fields.extend(
                    f"{multiple.group('type')} {name.strip()}"
                    for name in multiple.group("names").split(",")
                )
            else:
                fields.append(statement)
    return fields


_POD_LEAF_TYPES = frozenset({"CLSID", "FILETIME", "GUID", "IID", "LARGE_INTEGER", "ULARGE_INTEGER"})
_RUST_KEYWORDS = frozenset(
    {
        "as",
        "break",
        "const",
        "continue",
        "crate",
        "else",
        "enum",
        "extern",
        "false",
        "final",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "match",
        "mod",
        "move",
        "mut",
        "pub",
        "ref",
        "return",
        "self",
        "Self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "type",
        "unsafe",
        "use",
        "where",
        "while",
    }
)


def _parse_plain_state_field(
    declaration: str,
    *,
    allowed_pod_types: set[str] | frozenset[str] = frozenset(),
    array_constants: dict[str, int] | None = None,
) -> PlainStateField:
    """Parse the deliberately small C-layout state subset.

    A Rust-owned COM allocation does not contain ATL base-class state, so plain C
    scalar/pointer fields can be represented directly and zero-initialized before
    the translated constructor runs. C++ objects, references, arrays, bitfields,
    initializers, and multi-declarator statements retain hidden lifetime/layout
    semantics and must remain a loud preflight failure.
    """
    text = " ".join(declaration.split())
    com_ptr = re.fullmatch(r"CComPtr\s*<\s*([A-Za-z_]\w*)\s*>\s+([A-Za-z_]\w*)", text)
    if com_ptr is not None:
        interface, name = com_ptr.groups()
        return PlainStateField(
            declaration=text,
            c_type=f"{interface} *",
            name=name,
            com_ptr_interface=interface,
        )
    cstring = re.fullmatch(r"CStringW\s+([A-Za-z_]\w*)", text)
    if cstring is not None:
        return PlainStateField(
            declaration=text,
            c_type="CStringW",
            name=cstring.group(1),
            owned_cstring=True,
        )
    fixed_array = re.fullmatch(
        r"(?P<type>[A-Za-z_]\w*(?:\s+[A-Za-z_]\w*)*)\s+"
        r"(?P<name>[A-Za-z_]\w*)\s*\[\s*(?P<count>[A-Za-z_]\w*|\d+)\s*\]",
        text,
    )
    if fixed_array is not None:
        c_type = fixed_array.group("type")
        base = " ".join(re.sub(r"\b(?:const|volatile)\b", "", c_type).split())
        scalar_types = set(_RUST_SCALARS) | {
            "BOOL",
            "BOOLEAN",
            "BYTE",
            "CHAR",
            "DWORD",
            "DWORD64",
            "HRESULT",
            "INT",
            "INT_PTR",
            "LONG",
            "LONG_PTR",
            "LONGLONG",
            "SIZE_T",
            "UINT",
            "UINT_PTR",
            "ULONG",
            "ULONG_PTR",
            "ULONGLONG",
            "USHORT",
            "WCHAR",
            "WORD",
        }
        pointer_typedef = bool(re.fullmatch(r"(?:L?P(?:C)?|H)[A-Z][A-Z0-9_]*", base))
        if base not in scalar_types and not pointer_typedef and base not in allowed_pod_types:
            raise CppComError(f"state field array element type is not proven scalar: {declaration}")
        count_text = fixed_array.group("count")
        if count_text.isdigit():
            count = int(count_text)
        else:
            constants = array_constants or {}
            if count_text not in constants:
                raise CppComError(
                    f"state field array bound is not a proven numeric macro: {declaration}"
                )
            count = constants[count_text]
        return PlainStateField(
            declaration=text,
            c_type=c_type,
            name=fixed_array.group("name"),
            array_len=count,
        )
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
        "INT8": "",
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
    if (
        "*" not in c_type
        and base not in scalar_types
        and base not in allowed_pod_types
        and base not in _POD_LEAF_TYPES
        and not pointer_typedef
    ):
        raise CppComError(f"state field type is not proven C scalar/pointer state: {declaration}")
    return PlainStateField(declaration=text, c_type=c_type, name=name)


def _numeric_array_constants(source: Path, declarations: list[str]) -> dict[str, int]:
    """Resolve symbolic fixed-array bounds from compiler-facing headers.

    Every discovered spelling must agree.  This intentionally accepts only a
    single decimal literal, not expressions or architecture-dependent macros.
    """
    names = {
        match.group(1)
        for declaration in declarations
        if (match := re.search(r"\[\s*([A-Za-z_]\w*)\s*\]", declaration))
    }
    if not names:
        return {}
    repo_root = next(
        (parent for parent in source.parents if (parent / "sdk" / "include").is_dir()), None
    )
    if repo_root is None:
        return {}
    values: dict[str, int] = {}
    for name in names:
        found: set[int] = set()
        pattern = re.compile(rf"^\s*#\s*define\s+{re.escape(name)}\s+(\d+)\b", re.M)
        for header in (repo_root / "sdk" / "include").rglob("*.[hi]"):
            text = header.read_text(encoding="utf-8", errors="replace")
            found.update(int(item) for item in pattern.findall(text))
        if len(found) == 1:
            values[name] = found.pop()
    return values


def _pod_definition(text: str, name: str) -> tuple[str, bool] | None:
    patterns = (
        re.compile(
            rf"typedef\s+struct(?:\s+[A-Za-z_]\w*)?\s*\{{(?P<body>[^{{}}]*)\}}\s*"
            rf"(?P<aliases>[^;]*\b{re.escape(name)}\b[^;]*)\s*;",
            re.S,
        ),
        re.compile(rf"\bstruct\s+{re.escape(name)}\s*\{{(?P<body>[^{{}}]*)\}}\s*;", re.S),
    )
    for pattern in patterns:
        match = pattern.search(text)
        if match is None:
            continue
        prefix = text[: match.start()]
        packed = prefix.rfind("pshpack1.h") > prefix.rfind("poppack.h")
        return match.group("body"), packed
    return None


def _discover_pod_state_types(
    source: Path, header: Path, declarations: list[str]
) -> tuple[list[PodStateType], list[EnumStateType]]:
    """Find simple structs/enums reachable from class-owned state.

    Pointer-owned records matter too: translated workers dereference them even
    though they do not contribute inline bytes to the COM owner.
    """
    candidates = []
    for declaration in declarations:
        match = re.fullmatch(r"\s*(?P<type>[A-Za-z_]\w*)\s*(?:\*+\s*)?[A-Za-z_]\w*\s*", declaration)
        if match:
            c_type = match.group("type")
            pointer_typedef = bool(re.fullmatch(r"(?:L?P(?:C)?|H)[A-Z][A-Z0-9_]*", c_type))
            if (
                c_type not in _RUST_SCALARS
                and c_type not in _POD_LEAF_TYPES
                and not pointer_typedef
            ):
                candidates.append(c_type)
    if not candidates:
        return [], []
    local_texts = [
        path.read_text(encoding="utf-8", errors="replace")
        for path in dict.fromkeys((source, header))
    ]
    repo_root = next(
        (parent for parent in source.parents if (parent / "sdk" / "include").is_dir()), None
    )
    definitions: dict[str, tuple[str, bool]] = {}
    for name in dict.fromkeys(candidates):
        definition = next(
            (found for text in local_texts if (found := _pod_definition(text, name)) is not None),
            None,
        )
        if definition is None and repo_root is not None:
            for candidate in (repo_root / "sdk" / "include").rglob("*.[hi]"):
                text = candidate.read_text(encoding="utf-8", errors="replace")
                if not re.search(rf"\b{re.escape(name)}\b", text):
                    continue
                definition = _pod_definition(text, name)
                if definition is not None:
                    break
        if definition is None:
            continue
        definitions[name] = definition

    enum_names: set[str] = set()
    for body, _ in definitions.values():
        for raw in body.split(";"):
            declaration = " ".join(raw.split())
            field = re.match(r"(?:const\s+)?([A-Za-z_]\w*)\b", declaration)
            if field:
                enum_names.add(field.group(1))
    enums: list[EnumStateType] = []
    all_local_text = "\n".join(local_texts)
    for name in sorted(enum_names):
        match = re.search(
            rf"\benum\s+{re.escape(name)}\s*\{{(?P<body>[^{{}}]*)\}}\s*;",
            all_local_text,
            re.S,
        )
        if match is None:
            continue
        value = -1
        variants: list[tuple[str, int]] = []
        valid = True
        for raw in _split_args(match.group("body")):
            item = re.fullmatch(
                r"([A-Za-z_]\w*)(?:\s*=\s*(-?(?:0[xX][0-9A-Fa-f]+|\d+)))?",
                raw.strip(),
            )
            if item is None:
                valid = False
                break
            value = int(item.group(2), 0) if item.group(2) is not None else value + 1
            variants.append((item.group(1), value))
        if valid and variants:
            enums.append(EnumStateType(name, tuple(variants)))

    specs = []
    allowed_local = set(definitions) | {item.name for item in enums}
    for name, (body, packed) in definitions.items():
        body = re.sub(r"/\*.*?\*/|//[^\n]*|#\s*include[^\n]*", " ", body, flags=re.S)
        fields = []
        valid = True
        for raw in body.split(";"):
            declaration = " ".join(raw.split())
            if not declaration:
                continue
            try:
                fields.append(
                    _parse_plain_state_field(
                        declaration,
                        allowed_pod_types=_POD_LEAF_TYPES | allowed_local,
                    )
                )
            except CppComError:
                valid = False
                break
        if valid and fields:
            specs.append(PodStateType(name=name, fields=tuple(fields), packed=packed))
    return specs, enums


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
    state_fields = _state_fields(body)
    pod_state_types, enum_state_types = _discover_pod_state_types(source, header, state_fields)
    array_constants = _numeric_array_constants(source, state_fields)
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
    final_construct = bool(re.search(r"\bFinalConstruct\s*\(", body))
    if re.search(r"\bFinalRelease\s*\(", body):
        unsupported.append("custom ATL lifecycle method FinalRelease needs deterministic wiring")
    resource = re.search(r"\bDECLARE_REGISTRY_RESOURCEID\s*\(\s*([^)\s]+)\s*\)", body)
    not_agg = bool(re.search(r"\bDECLARE_(?:CENTRAL_INSTANCE_)?NOT_AGGREGATABLE\s*\(", body))
    clsid = None
    activation_sources = [source]
    if header != source:
        activation_sources.append(header)
    if object_map_source and object_map_source.is_file():
        activation_sources.append(object_map_source.resolve())
    for activation_source in dict.fromkeys(activation_sources):
        object_map = activation_source.read_text(encoding="utf-8", errors="replace")
        hit = re.search(
            rf"\bOBJECT_ENTRY(?:_AUTO)?\s*\(\s*([^,]+?)\s*,\s*"
            rf"{re.escape(class_name)}(?:RustEntry)?\s*\)",
            object_map,
        )
        if hit:
            clsid = hit.group(1).strip()
            break
    return CppComClass(
        name=class_name,
        source=source,
        header=header,
        com_map=entries,
        registry_resource=resource.group(1) if resource else None,
        not_aggregatable=not_agg,
        object_map_clsid=clsid,
        final_construct=final_construct,
        method_names=method_names or [],
        base_classes=bases,
        state_fields=state_fields,
        pod_state_types=pod_state_types,
        enum_state_types=enum_state_types,
        array_constants=array_constants,
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
    array = re.match(
        r"(?P<type>.*?)(?P<name>[A-Za-z_]\w*)\s*\[[^]]*\]$",
        text,
    )
    if array:
        # Function-parameter arrays are adjusted to pointers by C/C++ before
        # ABI lowering. The bound, when present, is not part of the vtable type.
        return f"{array.group('type').strip()} *", array.group("name")
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
    # The C++ half of WIDL's generated header is also compiler-facing authority
    # for interface inheritance.  Preserve it so a COM map that exposes both a
    # derived interface and one of its bases uses the single physical base
    # subobject that C++ actually lays out (for example IContextMenu2 and
    # IContextMenu).
    inheritance = re.search(
        rf"\b{re.escape(interface)}\s*:\s*public\s+([A-Za-z_]\w*)",
        text,
    )
    base = inheritance.group(1) if inheritance is not None else None
    # Keep generated output byte-identical across worktrees/build roots.
    return InterfaceSpec(interface, tuple(methods), header.name, base)


_RUST_SCALARS = {
    "void": "core::ffi::c_void",
    "VOID": "()",
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
    # Headers use both ``const T *`` and ``T const *``.  A qualifier left in
    # the base spelling becomes invalid Rust (``*mut T const``), so normalize
    # either C spelling before building the pointer declarator.
    is_const = re.search(r"\bconst\b", text) is not None
    text = re.sub(r"\bconst\b", "", text)
    text = " ".join(text.split())
    is_reference = text.endswith("&")
    if is_reference:
        text = text[:-1].rstrip()
    stars = text.count("*")
    base = " ".join(text.replace("*", " ").split())
    # WIDL preserves explicit C tag spelling in a few signatures
    # (``struct tagNMHDR *``). Rust names the tag type directly; retaining the
    # C keyword produces invalid ``*mut struct tagNMHDR`` syntax.
    base = re.sub(r"^(?:struct|enum|union)\s+", "", base)
    # C++ nested PODs are emitted as module-level Rust structs by the
    # deterministic scaffold (``CShellLink::volume_info`` -> ``volume_info``).
    base = re.sub(r"\b[A-Za-z_]\w*::", "", base)
    rust = _RUST_SCALARS.get(base, base)
    for depth in range(stars):
        qualifier = "const" if is_const and depth == 0 else "mut"
        rust = f"*{qualifier} {rust}"
    if is_reference:
        rust = f"&{'' if is_const else 'mut '}{rust}"
    return rust


_CPP_COM_DEBUG_MACROS = {"ASSERT", "ERR", "FIXME", "MESSAGE", "TRACE", "WARN", "pdump"}
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
        r'\s*extern\s+"(?:C|system)"\s*\{(?:\s*(?:pub\s+)?(?:fn\s+[^;]+|'
        r"static\s+(?:mut\s+)?[A-Za-z_]\w*\s*:\s*[^;]+);)+\s*\}\s*",
        "\n    ",
        rust,
        flags=re.S,
    )
    rust = re.sub(r"\bcrate::(?:types|shellcom)::", "", rust)
    rust = re.sub(r"\bcrate::driver_ddk::([A-Z][A-Z0-9_]*)\b", r"\1", rust)
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
        expected_opening = expected_signature.find("(")
        expected_closing = _matching_paren(expected_signature, expected_opening)
        expected_params = [
            match.group(1)
            for item in _split_args(expected_signature[expected_opening + 1 : expected_closing])
            if (match := re.match(r"\s*(?:mut\s+)?([A-Za-z_]\w*)\s*:", item))
        ]
        for name in expected_params:
            rust = re.sub(rf"\b_{re.escape(name)}\b", name, rust)
        expected_name_match = re.search(r"\bfn\s+([A-Za-z_]\w*)\s*\(", expected_signature)
        if expected_name_match is None:
            raise CppComError(f"invalid deterministic worker signature: {expected_signature}")
        expected_name = expected_name_match.group(1)
        actual = re.search(
            rf"\b(?:pub\s+)?(?:unsafe\s+)?fn\s+{re.escape(expected_name)}\s*\(",
            rust,
        )
        if actual is not None:
            opening = rust.find("(", actual.start())
            closing = _matching_paren(rust, opening)
            brace = rust.find("{", closing + 1)
            actual_param_text = re.sub(
                r"/\*.*?\*/|//[^\n]*", " ", rust[opening + 1 : closing], flags=re.S
            )
            actual_items = _split_args(actual_param_text)
            actual_params = [
                match.group(1)
                for item in actual_items
                if (match := re.match(r"\s*(?:mut\s+)?([A-Za-z_]\w*)\s*:", item))
            ]
            mutable_params = {
                match.group(1)
                for item in actual_items
                if (match := re.match(r"\s*mut\s+([A-Za-z_]\w*)\s*:", item))
            }
            # The model owns no part of the worker surface. If its parameter
            # list still names the generated slots in order, replace all of
            # its type/return/qualifier spellings with the exact authority.
            if brace >= 0 and actual_params == expected_params:
                authoritative = expected_signature
                for name in mutable_params:
                    authoritative = re.sub(
                        rf"(?<=[(,])\s*{re.escape(name)}\s*:",
                        lambda match, param=name: match.group(0).replace(param, f"mut {param}"),
                        authoritative,
                        count=1,
                    )
                rust = rust[: actual.start()] + authoritative + " " + rust[brace:]
        # The model is contracted for exactly one body worker. Deterministically
        # discard crate attributes, imports, shadow types, and trailing helpers
        # outside that function before any response can enter the cache/module.
        exact = re.search(rf"\bpub\s+unsafe\s+fn\s+{re.escape(expected_name)}\s*\(", rust)
        if exact is not None:
            opening = rust.find("(", exact.start())
            closing = _matching_paren(rust, opening)
            brace = rust.find("{", closing + 1)
            if brace >= 0:
                _, body_end = _balanced_block(_masked_cpp(rust), brace)
                rust = rust[exact.start() : body_end + 1]
    return rust


def normalize_worker_semantics(source: str, rust: str) -> str:
    """Lower compiler-known C/ATL conveniences after a cached/model worker returns.

    These rewrites depend only on syntax present in the original worker.  They are
    deliberately downstream of the response cache: improving a deterministic
    lowering must never require buying the same model response again.
    """
    rust = strip_debug_statements(rust, {"pdump"})
    rust = re.sub(r"\bcrate::driver_ddk::([A-Z][A-Z0-9_]*)\b", r"\1", rust)
    rust = re.sub(
        r"&mut\s+(\(\*[A-Za-z_]\w*\)\.u\.[A-Za-z_]\w*)\s+as\s+\*mut\s+"
        r"([A-Za-z_]\w*)",
        r"core::ptr::addr_of_mut!(\1).cast::<\2>()",
        rust,
    )
    self_guards = set()
    for owner, name in re.findall(
        r"\bCComPtr\s*<\s*([A-Za-z_]\w*)\s*>\s+([A-Za-z_]\w*)\s*=\s*this\s*;",
        source,
    ):
        self_guards.add(name)
        if re.search(
            rf"let\s+mut\s+{re.escape(name)}\s*:\s*\*mut\s+IUnknown\s*=\s*this\.cast\(\)",
            rust,
        ):
            continue
        model_guard = re.compile(
            r"(?s)\s*let\s+[A-Za-z_]\w*\s*=\s*this\s+as\s+\*mut\s+IUnknown\s*;"
            r".*?let\s+_[A-Za-z_]\w*\s*=\s*[A-Za-z_]\w*\s*\([^;]+\)\s*;"
        )
        replacement = (
            f"\n    let mut {name}: *mut IUnknown = this.cast();\n"
            f"    ((*(*{name}).lpVtbl).AddRef)({name});\n"
            f"    let _{name}_cpp_com_guard = CppComPtrGuard::new("
            f"core::ptr::addr_of_mut!({name}));"
        )
        rust, replaced = model_guard.subn(replacement, rust, count=1)
        if replaced != 1:
            raise CppComError(f"CComPtr self guard {owner} {name}: model guard not recognized")
    rust = re.sub(
        r"\*([A-Za-z_]\w*(?:pidl|Pidl)[A-Za-z_]*)\s*=\s*0\s*;",
        r"*\1 = core::ptr::null_mut();",
        rust,
    )
    rust = re.sub(r"\bIsEqualIID\s*\(", "cpp_com_iid_equal(", rust)
    if "SHCreateWorkerWindowW" in source and "WorkerWndProc" in source:
        rust = re.sub(
            r"([A-Za-z_]\w*WorkerWndProc)\s+as\s+unsafe\s+extern\s+"
            r'"system"\s+fn\s*\(\.\.\.\)\s*->\s*LRESULT',
            r"Some(\1)",
            rust,
        )
        rust = rust.replace("Some(WorkerWndProc)", "Some(CUserNotification_WorkerWndProc)")
        rust = rust.replace("return 0 as isize;", "return 0 as LRESULT;")
        rust = rust.replace(
            "(*this).m_hWorkerWnd = 0 as isize;",
            "(*this).m_hWorkerWnd = core::ptr::null_mut();",
        )
        rust = re.sub(
            r"(wParam\s*==\s*ID_[A-Za-z0-9_]+)\s+as\s+usize",
            r"\1 as WPARAM",
            rust,
        )
        rust = rust.replace(
            "KillTimer(hWnd, wParam as isize)",
            "KillTimer(hWnd, wParam as UINT_PTR)",
        )
    rust = re.sub(
        r"\(\*this\)\.(m_[A-Za-z0-9_]+)\s+as\s+\*const\s+u16",
        r"(*this).\1.as_ptr()",
        rust,
    )
    rust = re.sub(
        r"\(\*this\)\.(m_h[A-Za-z0-9_]+)\s*!=\s*0\b",
        r"!(*this).\1.is_null()",
        rust,
    )
    rust = re.sub(r"\bhIcon\s*!=\s*0\b", "!hIcon.is_null()", rust)
    if "CUserNotification" in source:
        rust = rust.replace("Some(WorkerWndProc)", "Some(CUserNotification_WorkerWndProc)")
    rust = re.sub(
        r"(SetTimer\([^;]*?),\s*(?:::)?core::ptr::null_mut\(\)\s*\)",
        r"\1, None)",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"&nid\s+as\s+\*const\s+NOTIFYICONDATAW",
        r"&mut nid",
        rust,
    )
    rust = re.sub(
        r"\bSHELL32_CompareDetails\s*\(\s*this\s*,",
        "SHELL32_CompareDetails(core::ptr::addr_of_mut!((*this).IShellFolder2_iface),",
        rust,
    )
    rust = re.sub(
        r"(pshf:\s*)this\b",
        r"\1core::ptr::addr_of_mut!((*this).IShellFolder_iface)",
        rust,
    )
    rust = re.sub(
        r"(\*pcsFlags\s*=\s*[A-Za-z_]\w*\[[^]]+\]\.colstate)(\s*;)",
        r"\1 as SHCOLSTATEF\2",
        rust,
    )
    rust = re.sub(
        r"((?:\(\*[A-Za-z_]\w*\)|[A-Za-z_]\w*)\.(?:fmt|cxChar)\s*=\s*"
        r"[A-Za-z_]\w*\[[^]]+\]\.(?:fmt|cxChar))(\s*;)",
        r"\1 as INT\2",
        rust,
    )
    rust = re.sub(
        r"cpp_com_iid_equal\(([^()]*(?:\([^()]*\)[^()]*)*)\)\s*!=\s*0",
        r"cpp_com_iid_equal(\1)",
        rust,
    )
    # A worker may use an SDK type named by the C source, but it may not shadow
    # that deterministic module-level authority with a guessed local layout.
    # Remove only repr(C) declarations whose names occur in the source and are
    # not themselves declared by that source worker. Source-local PODs remain
    # intact and are layout-checked by ``audit_worker_body``.
    source_local_types = _source_local_pod_fields(source)
    shadow_heads = list(
        re.finditer(
            r"#\s*\[\s*repr\s*\(\s*C\s*\)\s*\]\s*struct\s+([A-Za-z_]\w*)\s*\{",
            rust,
        )
    )
    for shadow in reversed(shadow_heads):
        type_name = shadow.group(1)
        if type_name in source_local_types or not re.search(rf"\b{re.escape(type_name)}\b", source):
            continue
        open_brace = rust.find("{", shadow.start())
        try:
            _, close_brace = _balanced_block(rust, open_brace)
        except CppComError:
            continue
        rust = rust[: shadow.start()] + rust[close_brace + 1 :]

    # Rust reserves ``final`` for future use, while C/C++ permits it as an
    # ordinary local name. Raw identifiers preserve that source spelling.
    if re.search(r"\bfinal\s*(?:=|;|\[)", _masked_cpp(source)):
        masked_rust = _masked_cpp(rust)
        matches = list(re.finditer(r"(?<![#\w])final\b", masked_rust))
        for match in reversed(matches):
            rust = rust[: match.start()] + "r#final" + rust[match.end() :]

    # no_std shell modules never own a `std` dependency.
    rust = rust.replace("std::", "core::")
    sfv_create = re.search(
        r"\bSFV_CREATE\s+([A-Za-z_]\w*)\s*=\s*\{\s*sizeof\s*\(\s*SFV_CREATE\s*\)\s*,\s*"
        r"([A-Za-z_]\w*)\s*\}\s*;",
        source,
    )
    if sfv_create is not None:
        local_name, folder_name = sfv_create.groups()
        # The SDK declaration is the ABI authority. A model-local shadow type is
        # never accepted, even if its guessed prefix happens to compile.
        rust = re.sub(
            r"(?s)(?:#\[repr\(C\)\]\s*)?struct\s+SFV_CREATE\s*\{[^{}]*\}\s*",
            "",
            rust,
        )
        initializer = (
            f"let {local_name} = SFV_CREATE {{\n"
            "        cbSize: core::mem::size_of::<SFV_CREATE>() as UINT,\n"
            f"        pshf: {folder_name},\n"
            "        psvOuter: core::ptr::null_mut(),\n"
            "        psfvcb: core::ptr::null_mut(),\n"
            "    };"
        )
        rust, replacements = re.subn(
            rf"(?s)let\s+(?:mut\s+)?{re.escape(local_name)}(?:\s*:[^=]+)?\s*=\s*"
            r"SFV_CREATE\s*\{.*?\}\s*;",
            initializer,
            rust,
            count=1,
        )
        if replacements != 1:
            rust, replacements = re.subn(
                rf"(?s)let\s+(?:mut\s+)?{re.escape(local_name)}\s*:\s*SFV_CREATE\s*=\s*"
                rf"core::mem::zeroed\(\)\s*;\s*{re.escape(local_name)}\.cbSize\s*=.*?;\s*"
                rf"{re.escape(local_name)}\.pshf\s*=.*?;",
                initializer,
                rust,
                count=1,
            )
        if replacements != 1:
            raise CppComError(f"SFV_CREATE local {local_name}: worker initializer not found")
    # CComPtr locals have scope-exit semantics on every C++ return path.  Keep
    # the model's raw pointer spelling (which makes vtable calls simple), but
    # attach a deterministic guard to its slot.  This closes failure exits the
    # model may overlook and makes ownership independent of control-flow style.
    com_ptr_locals = re.findall(
        r"\bCComPtr\s*<\s*([A-Za-z_]\w*)\s*>\s+([A-Za-z_]\w*)\s*[;=]",
        source,
    )
    com_drop = re.search(r"\bstruct\s+ComDrop\s*<[^>]+>\s*\{", rust)
    if com_drop is not None:
        first_open = rust.find("{", com_drop.start())
        _inner, first_end = _balanced_block(rust, first_open)
        drop_impl = re.search(
            r"impl\s*<[^>]+>\s+Drop\s+for\s+ComDrop\s*<[^>]+>\s*\{",
            rust[first_end + 1 :],
        )
        if drop_impl is None:
            raise CppComError("model ComDrop helper has no matching Drop implementation")
        impl_start = first_end + 1 + drop_impl.start()
        impl_open = rust.find("{", impl_start)
        _inner, impl_end = _balanced_block(rust, impl_open)
        rust = rust[: com_drop.start()] + rust[impl_end + 1 :]
    for interface, name in com_ptr_locals:
        if name in self_guards:
            continue
        if len(re.findall(rf"\b{re.escape(name)}\b", source)) == 1:
            # An unused ATL local has no observable construction/destruction;
            # omitting it is the exact deterministic lowering. If the model
            # emitted an immutable raw-null placeholder, remove that placeholder
            # and its defensive Release debris. A mutable slot may still be an
            # out-parameter owner and follows the ordinary guard path below.
            immutable = re.compile(
                rf"^[ \t]*let\s+{re.escape(name)}\s*:\s*\*mut\s+"
                rf"{re.escape(interface)}\s*=\s*core::ptr::null_mut\(\)\s*;\s*",
                flags=re.M,
            )
            if immutable.search(rust):
                rust = immutable.sub("", rust)
                rust = re.sub(
                    rf"\s*if\s+!{re.escape(name)}\.is_null\(\)\s*\{{\s*"
                    rf"\(\(\*\(\*{re.escape(name)}\)\.lpVtbl\)\.Release\)"
                    rf"\({re.escape(name)}\);\s*\}}",
                    "",
                    rust,
                )
                continue
            if not re.search(rf"\b{re.escape(name)}\b", rust):
                continue
        # The model sometimes reproduces ATL's wrapper syntax even though the
        # worker contract exposes CComPtr storage as a scaffold-owned raw slot.
        # Lower only the exact null/default construction shape; Attach/Detach,
        # non-null construction, or any other ownership API remains a failure.
        rust = re.sub(
            rf"let\s+mut\s+{re.escape(name)}\s*:\s*CComPtr\s*<\s*"
            rf"{re.escape(interface)}\s*>\s*=\s*CComPtr::new\(\)\s*;",
            f"let mut {name}: *mut {interface} = core::ptr::null_mut();",
            rust,
            count=1,
        )
        rust = re.sub(
            rf"let\s+mut\s+{re.escape(name)}\s*=\s*ComDrop\s*::\s*<\s*"
            rf"{re.escape(interface)}\s*>\s*\{{\s*ptr:\s*core::ptr::null_mut\(\)\s*,?\s*\}}\s*;",
            f"let mut {name}: *mut {interface} = core::ptr::null_mut();",
            rust,
            count=1,
        )
        rust = re.sub(rf"\b{re.escape(name)}\.ptr\b", name, rust)
        copied_value = re.search(
            rf"\bCComPtr\s*<\s*{re.escape(interface)}\s*>\s+{re.escape(name)}\s*=\s*"
            r"(?P<this>this->)?(?P<value>[A-Za-z_]\w*)\s*;",
            source,
        )
        if copied_value is not None:
            value = copied_value.group("value")
            member_value = f"(*this).{value}"
            rust_value = (
                member_value
                if copied_value.group("this")
                or re.search(
                    rf"let\s+(?:mut\s+)?{re.escape(name)}(?:\s*:[^=]+)?\s*=\s*"
                    rf"{re.escape(member_value)}\s*;",
                    rust,
                )
                else value
            )
            rust = re.sub(
                rf"let\s+(?:mut\s+)?{re.escape(name)}(?:\s*:\s*\*mut\s+"
                rf"{re.escape(interface)})?\s*=\s*{re.escape(rust_value)}\s*;",
                f"let mut {name}: *mut {interface} = {rust_value};",
                rust,
                count=1,
            )
            # ATL's copy constructor owns one AddRef.  Normalize a model-spelled
            # inherited-IUnknown cast to the concrete vtable signature, or add
            # the call deterministically when the model omitted it.
            add_ref = re.compile(
                rf"\(\(\*\(\*{re.escape(name)}\)\.lpVtbl\)\.AddRef\)"
                rf"\({re.escape(name)}(?:\s+as\s+\*mut\s+IUnknown)?\)"
            )
            add_refs = list(add_ref.finditer(rust))
            if len(add_refs) > 1:
                raise CppComError(f"CComPtr local {name}: multiple copy AddRef calls")
            rust = add_ref.sub(f"((*(*{name}).lpVtbl).AddRef)({name})", rust, count=1)
        rust = re.sub(rf"\b{re.escape(name)}\.0\b", name, rust)
        declaration = re.compile(
            rf"(?P<indent>^[ \t]*)let\s+mut\s+{re.escape(name)}\s*:\s*"
            rf"\*mut\s+{re.escape(interface)}\s*=\s*(?:core::ptr::null_mut\(\)|"
            rf"\(\*this\)\.[A-Za-z_]\w*|[A-Za-z_]\w*)\s*;",
            re.M,
        )
        matches = list(declaration.finditer(rust))
        if len(matches) != 1:
            raise CppComError(
                f"CComPtr local {name}: expected one null raw-pointer declaration in worker, "
                f"found {len(matches)}"
            )
        match = matches[0]
        guard_name = f"_{name}_cpp_com_guard"
        if not re.search(rf"\blet\s+{re.escape(guard_name)}\s*=", rust):
            guard = (
                match.group(0)
                + "\n"
                + match.group("indent")
                + (
                    f"if !{name}.is_null() {{ ((*(*{name}).lpVtbl).AddRef)({name}); }}\n"
                    + match.group("indent")
                    if copied_value is not None and not add_refs
                    else ""
                )
                + f"let {guard_name} = CppComPtrGuard::new("
                + f"core::ptr::addr_of_mut!({name}));"
            )
            rust = rust[: match.start()] + guard + rust[match.end() :]

        # DeepSeek commonly emits explicit CComPtr-destructor releases on some
        # branches.  Null the watched slot after those releases so the guard is
        # idempotent while retaining the exact point at which C++ destroyed it.
        release = re.compile(
            rf"(?P<indent>^[ \t]*)\(\(\*\(\*{re.escape(name)}\)\.lpVtbl\)\.Release\)"
            rf"\({re.escape(name)}(?:\s+as\s+\*mut\s+IUnknown)?\);"
            rf"(?!\s*{re.escape(name)}\s*=\s*core::ptr::null_mut\(\)\s*;)",
            re.M,
        )
        rust = release.sub(
            lambda item, local_name=name: (
                item.group(0)
                + "\n"
                + item.group("indent")
                + f"{local_name} = core::ptr::null_mut();"
            ),
            rust,
        )
    # CComPtr object state is likewise represented by a raw pointer in the
    # generated owner. `.0` is a model artifact from an imagined Rust wrapper.
    rust = re.sub(r"(\(\*this\)\.[A-Za-z_]\w*)\.0\b", r"\1", rust)
    # IID_PPV_ARG(T, &out) is a header macro, not a callable ABI symbol.
    rust = re.sub(
        r"let\s+([A-Za-z_]\w*)\s*=\s*IID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)\s*;\s*"
        r"let\s+([A-Za-z_]\w*)\s*=\s*&mut\s+(\(\*this\)\.[A-Za-z_]\w*|"
        r"[A-Za-z_]\w*)\s*\)\s*;",
        r"let \1 = core::ptr::addr_of!(IID_\2); "
        r"let \3 = core::ptr::addr_of_mut!(\4).cast();",
        rust,
    )
    iid_pair = re.search(
        r"let\s*\(\s*([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*\)\s*=\s*"
        r"IID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)\s*,\s*&mut\s+([A-Za-z_]\w*)\s*\)\s*;",
        rust,
    )
    if iid_pair is not None:
        iid_name, output_name, interface, output = iid_pair.groups()
        rust = rust[: iid_pair.start()] + rust[iid_pair.end() :]
        rust = re.sub(
            rf"\b{re.escape(iid_name)}\s*,\s*{re.escape(output_name)}\b",
            f"core::ptr::addr_of!(IID_{interface}), core::ptr::addr_of_mut!({output}).cast()",
            rust,
            count=1,
        )
    rust = re.sub(
        r"\bIID_PPV_ARG\s*\(\s*(?:&IID_)?([A-Za-z_]\w*)\s*,\s*"
        r"&mut\s+([A-Za-z_]\w*)(?:\s+as\s+[^,)]+(?:<[^>]+>)?)?\s*,?\s*\)",
        r"core::ptr::addr_of!(IID_\1), core::ptr::addr_of_mut!(\2).cast()",
        rust,
    )
    source_iid_args = re.findall(
        r"\bIID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*\)", source
    )
    for interface, output in source_iid_args:
        rust = re.sub(
            rf"\bIID_PPV_ARG\s*\(\s*{re.escape(output)}\s*\)",
            f"core::ptr::addr_of!(IID_{interface}), {output}.cast()",
            rust,
        )
    # C++ sibling calls have an implicit receiver. The prepared source names
    # every such worker explicitly, so receiver insertion is syntax-authoritative.
    owner = re.search(r"\b([A-Za-z_]\w*)::[~A-Za-z_]\w*\s*\(", source)
    if owner is None:
        # Inline ATL methods lose the qualified `Class::Method` spelling in
        # the prepared source, but the generated worker signature still owns
        # the receiver type.  Recover that deterministic owner instead.
        owner = re.search(
            r"\bfn\s+[A-Za-z_]\w*_body\s*\(\s*this\s*:\s*\*mut\s+([A-Za-z_]\w*)",
            rust,
        )
    if owner is not None:
        class_name = owner.group(1)
        for nested in set(re.findall(rf"\b{re.escape(class_name)}::([A-Za-z_]\w*)\b", source)):
            rust = re.sub(
                rf"\b(?:{re.escape(class_name)}_|{re.escape(class_name)}::)"
                rf"{re.escape(nested)}\b",
                nested,
                rust,
            )
    sibling_calls = (
        set(re.findall(rf"\b({re.escape(owner.group(1))}_[A-Za-z_]\w*_body)\s*\(", source))
        if owner is not None
        else set()
    )
    sibling_arities: dict[str, set[int]] = {}
    masked_source = _masked_cpp(source)
    for _start, end, callee, arity in _named_call_spans(source, sibling_calls):
        opening = masked_source.find("(", end)
        tail = masked_source[opening + 1 :].lstrip()
        source_has_this = tail.startswith("this") and not re.match(r"this[A-Za-z0-9_]", tail)
        sibling_arities.setdefault(callee, set()).add(arity - 1 if source_has_this else arity)
    insertions = []
    masked_rust = _masked_cpp(rust)
    for _start, end, callee, arity in _named_call_spans(rust, sibling_calls):
        if arity not in sibling_arities.get(callee, set()):
            continue
        opening = masked_rust.find("(", end)
        tail = masked_rust[opening + 1 :].lstrip()
        if tail.startswith("this") and not re.match(r"this[A-Za-z0-9_]", tail):
            continue
        insertions.append((opening + 1, "this" + (", " if arity else "")))
    for position, insertion in reversed(insertions):
        rust = rust[:position] + insertion + rust[position:]
    # Header-only predicates and HRESULT helpers are deterministic expressions.
    rust = re.sub(r"\bFAILED_UNEXPECTEDLY\s*\(([^()]+)\)", r"((\1) < 0)", rust)
    rust = re.sub(r"\bSCODE_CODE\s*\(([^()]+)\)", r"(((\1) as u32) & 0xffff)", rust)
    rust = re.sub(
        r"\bIsEqualGUID\s*\(\s*([^,]+),\s*&([A-Za-z_]\w*)\s+as\s+\*const\s+GUID\s*\)\s*!=\s*0",
        r"cpp_com_iid_equal(\1, core::ptr::addr_of!(\2))",
        rust,
    )
    rust = re.sub(
        r"\bIsEqualGUID\s*\(\s*([^,]+),\s*((?:BHID|CLSID|IID)_[A-Za-z_]\w*)\s*\)"
        r"(?:\s*!=\s*0)?",
        r"cpp_com_iid_equal(\1, core::ptr::addr_of!(\2))",
        rust,
    )
    rust = re.sub(r"if\s+_ILIsDesktop\s*\((.*?)\)\s*\{", r"if _ILIsDesktop(\1) != 0 {", rust)
    rust = re.sub(
        r"if\s+!_ILIsSpecialFolder\s*\((.*?)\)\s*\{",
        r"if _ILIsSpecialFolder(\1) == 0 {",
        rust,
    )
    rust = re.sub(r"(?m)^\s*ATLASSERT\s*\([^;]*\)\s*;\s*", "", rust)
    rust = re.sub(
        r"\bSHStrDupW\s*\(\s*INVALID_FILETITLE_CHARACTERSW\s*,",
        "SHStrDupW(INVALID_FILETITLE_CHARACTERSW.as_ptr(),",
        rust,
    )
    iid_alias = re.search(
        r"let\s+([A-Za-z_]\w*)\s*=\s*&mut\s+\(\*this\)\.([A-Za-z_]\w*)\s*;",
        rust,
    )
    if iid_alias is not None:
        alias, field = iid_alias.groups()
        iid_use = re.compile(rf"\bIID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)\s*,\s*{re.escape(alias)}\s*\)")
        matched_use = iid_use.search(rust)
        if matched_use is not None:
            interface = matched_use.group(1)
            replacement = (
                f"core::ptr::addr_of!(IID_{interface}), "
                f"core::ptr::addr_of_mut!((*this).{field}).cast()"
            )
            rust = rust[: iid_alias.start()] + rust[iid_alias.end() :]
            rust = iid_use.sub(replacement, rust, count=1)
    rust = re.sub(
        r"let\s*\(\s*([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*\)\s*=\s*"
        r"([^,;]+),\s*([^;]+);",
        r"let \1 = \3; let \2 = \4;",
        rust,
    )
    rust = re.sub(
        r"(\.CompareIDs\)\([^,]+,)\s*([A-Za-z_]\w*)\s*,",
        r"\1 \2 as LPARAM,",
        rust,
    )
    rust = re.sub(r"\bGET_SHGDN_RELATION\s*\(([^()]+)\)", r"((\1 as DWORD) & 0xff)", rust)
    rust = re.sub(r"\bGET_SHGDN_FOR\s*\(([^()]+)\)", r"((\1 as DWORD) & 0xff00)", rust)
    rust = re.sub(r"\(\*this\)\.m_pisfInner\.as_ptr\(\)", "(*this).m_pisfInner", rust)
    rust = re.sub(r"\(\*this\)\.m_pisfInner\.p\b", "(*this).m_pisfInner", rust)
    rust = re.sub(
        r"\bIID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)\s*,\s*"
        r"&mut\s+\(\*this\)\.([A-Za-z_]\w*)\s*\)",
        r"core::ptr::addr_of!(IID_\1), core::ptr::addr_of_mut!((*this).\2).cast()",
        rust,
    )
    iid_binding = re.search(
        r"let\s+([A-Za-z_]\w*)\s*=\s*IID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)\s*,\s*"
        r"&mut\s+\(\*this\)\.([A-Za-z_]\w*)\s*\)\s*;",
        rust,
    )
    if iid_binding is not None:
        binding, interface, field = iid_binding.groups()
        expansion = (
            f"core::ptr::addr_of!(IID_{interface}), core::ptr::addr_of_mut!((*this).{field}).cast()"
        )
        rust = rust[: iid_binding.start()] + rust[iid_binding.end() :]
        rust = re.sub(rf"\b{re.escape(binding)}\s*,", expansion + ",", rust, count=1)
    transformed_binding = re.search(
        r"let\s+([A-Za-z_]\w*)\s*=\s*"
        r"(core::ptr::addr_of!\(IID_[A-Za-z_]\w*\))\s*,\s*"
        r"(core::ptr::addr_of_mut!\(\(\*this\)\.[A-Za-z_]\w*\)\.cast\(\))\s*;",
        rust,
    )
    if transformed_binding is not None:
        binding, iid, output = transformed_binding.groups()
        rust = rust[: transformed_binding.start()] + rust[transformed_binding.end() :]
        rust = re.sub(rf"\b{re.escape(binding)}\s*,", f"{iid}, {output},", rust, count=1)
    rust = re.sub(
        r"\bSHFree\s*\(\s*(\(\*this\)\.[A-Za-z_]\w*)\s*\)",
        r"SHFree((\1) as LPVOID)",
        rust,
    )
    # Win32 error APIs use LONG in HRESULT_FROM_WIN32, while most producers spell
    # the value DWORD.  The C macro performs this scalar conversion implicitly.
    rust = re.sub(
        r"\bHRESULT_FROM_WIN32\s*\(\s*([A-Za-z_]\w*)\s*\)", r"HRESULT_FROM_WIN32(\1 as LONG)", rust
    )
    # void-pointer arguments accept any object pointer in C.
    rust = re.sub(
        r"\bSetClipboard_body\s*\(([^,]+),\s*([A-Za-z_]\w*)\s*,\s*([^\)]+)\)",
        r"SetClipboard_body(\1, \2 as *const core::ffi::c_void, \3)",
        rust,
    )
    # ERROR_SUCCESS is canonically LONG in the shared substrate; DWORD-returning
    # helpers receive the same zero bit pattern through C's usual conversion.
    rust = re.sub(r"\{\s*ERROR_SUCCESS\s*\}\s*else", "{ ERROR_SUCCESS as DWORD } else", rust)
    # CStringW(MAKEINTRESOURCEW(id)) invokes ATL's resource-string constructor.
    # Cheap models commonly mistake the integer resource token for a string pointer.
    resource = re.search(
        r"CStringW\s+([A-Za-z_]\w*)\s*\(\s*MAKEINTRESOURCEW\s*\(\s*([A-Za-z_]\w*)\s*\)\s*\)",
        source,
    )
    if resource is not None:
        source_name, resource_id = resource.groups()
        binding = re.search(
            rf"let\s+([A-Za-z_]\w*)\s*:\s*\*const\s+u16\s*=\s*{re.escape(resource_id)}[^;]*;",
            rust,
        )
        if binding is not None:
            rust_name = binding.group(1)
            rust = (
                rust[: binding.start()]
                + f"let mut {rust_name} = CStringW::from_resource({resource_id} as UINT);"
                + rust[binding.end() :]
            )
            rust = re.sub(
                rf"\b{re.escape(rust_name)}\s+as\s+\*mut\s+u16", f"{rust_name}.as_mut_ptr()", rust
            )
        else:
            rust = re.sub(
                rf"let\s+(?:mut\s+)?{re.escape(source_name)}\s*=\s*CStringW::new\([^;]*\);",
                f"let mut {source_name} = CStringW::from_resource({resource_id} as UINT);",
                rust,
                count=1,
            )
        rust = re.sub(
            rf"\bSHStrDupW\s*\(\s*{re.escape(source_name)}\s*,",
            f"SHStrDupW({source_name}.as_ptr(),",
            rust,
        )
    # Fixed C arrays retain their extent in generated Rust object state.
    rust = re.sub(r"\b_countof\s*\(([^()]+|\(\*this\)\.[A-Za-z_]\w*)\)", r"\1.len()", rust)
    rust = re.sub(r"&mut\s+([A-Za-z_]\w*)\s+as\s+\*mut\s+_\.len\(\)", r"\1.len()", rust)
    # Finish arbitrary member/parenthesized forms with a balanced transform;
    # regex-only lowering cannot see `_countof((*p).field)` correctly.
    while True:
        masked = _masked_cpp(rust)
        calls = list(re.finditer(r"\b_countof!?\s*\(", masked))
        if not calls:
            break
        call = calls[-1]
        opening = masked.find("(", call.start())
        closing = _matching_paren(masked, opening)
        expression = rust[opening + 1 : closing].strip()
        if expression.startswith("&"):
            expression = expression[1:].strip()
        replacement = expression if expression.endswith(".len()") else f"({expression}).len()"
        rust = rust[: call.start()] + replacement + rust[closing + 1 :]
    # A model sometimes heapifies a fixed C stack buffer. Restore the exact
    # source-derived extent; shell32 workers are no_std and this also preserves
    # the original allocation/failure behavior.
    for fixed in re.finditer(
        r"\b(?P<type>WCHAR|CHAR|BYTE)\s+(?P<name>[A-Za-z_]\w*)\s*"
        r"\[\s*(?P<count>[A-Za-z_]\w*|\d+)\s*\]",
        source,
    ):
        c_type, name, count = fixed.group("type", "name", "count")
        rust = re.sub(
            rf"(?:const\s+[A-Za-z_]\w*\s*:\s*usize\s*=\s*{re.escape(count)}(?:\s+as\s+usize)?\s*;\s*)?"
            rf"let\s+mut\s+{re.escape(name)}\s*:\s*Vec<[^>]+>\s*=\s*"
            rf"vec!\[\s*0(?:u\d+)?\s*;\s*[A-Za-z_]\w*\s*\]\s*;",
            f"let mut {name}: [{c_type}; {count} as usize] = [0; {count} as usize];",
            rust,
        )
    # C field spellings can be Rust keywords. Raw identifiers preserve layout
    # and keep source-derived member access mechanical.
    rust = re.sub(r"\.type\b", ".r#type", rust)
    rust = re.sub(r"(\b[A-Za-z_]\w*_body\(this,)\s+", r"\1 ", rust)
    rust = re.sub(
        r"let\s+([A-Za-z_]\w*)\s*:\s*BOOL\s*=\s*if\s+([^{};]+)\s*"
        r"\{\s*1\s*\}\s*else\s*\{\s*0\s*\}\s*;",
        r"let \1: BOOL = (\2) as BOOL;",
        rust,
    )
    rust = re.sub(r"\bCShellLink::volume_info\b", "volume_info", rust)
    rust = re.sub(
        r"\bHeapFree\s*\(\s*GetProcessHeap\(\)\s*,\s*0\s*,\s*"
        r"(\(\*this\)\.[A-Za-z_]\w*)\s*,?\s*\)",
        r"HeapFree(GetProcessHeap(), 0, \1.cast())",
        rust,
    )
    rust = re.sub(
        r"\bStringCchCopyW\s*\(\s*(\(\*this\)\.[A-Za-z_]\w*)\s*,",
        r"StringCchCopyW(\1.as_mut_ptr(),",
        rust,
    )
    rust = re.sub(
        r"\bSHEmptyRecycleBinW\s*\(\s*([^,]+),\s*(\(\*this\)\.[A-Za-z_]\w*)\s*,",
        r"SHEmptyRecycleBinW(\1, \2.as_ptr(),",
        rust,
    )
    # COM callback syntax is fixed by the generated interface vtable.  Some
    # models add one dereference around the interface before selecting lpVtbl
    # and omit the call-parentheses required for a Rust function-pointer field.
    rust = re.sub(
        r"\(\(\*\(\*([A-Za-z_]\w*)\)\)\.lpVtbl\)\.([A-Za-z_]\w*)\(",
        r"((*(*\1).lpVtbl).\2)(",
        rust,
    )
    if re.search(r"\*pdwlSpaceUsed\s*=\s*bin\.i64Size", source):
        rust = re.sub(
            r"\bbin\.i64Size\b(?!\s+as\s+DWORDLONG|\s*=)",
            "bin.i64Size as DWORDLONG",
            rust,
        )
    if re.search(r"\bSIZE_T\s+[A-Za-z_]\w*\s*=\s*lstrlenW", source):
        rust = re.sub(
            r"let\s+cch\s*=\s*lstrlenW\(([^)]+)\)\s*\+\s*1\s*;",
            r"let cch: SIZE_T = lstrlenW(\1) as SIZE_T + 1;",
            rust,
        )
    if re.search(r"CStringW\s*&\s*[A-Za-z_]\w*", source):
        rust = re.sub(r"(?m)^(\s*)paths\s*\+=", r"\1*paths +=", rust)
    # File-scope C arrays decay to pointers at call sites.
    rust = re.sub(
        r"SHELL_MapContextMenuVerbToCmdId\s*\(([^,]+),\s*g_VerbMap\s*\)",
        r"SHELL_MapContextMenuVerbToCmdId(\1, core::ptr::addr_of!(g_VerbMap).cast())",
        rust,
    )
    rust = re.sub(
        r"&g_VerbMap\s+as\s+\*const\s+u8\s+as\s+\*const\s+core::ffi::c_void",
        "core::ptr::addr_of!(g_VerbMap).cast()",
        rust,
    )
    rust = re.sub(r"\bcmd_id\s*==\s*(IDC_[A-Za-z_]\w*)", r"cmd_id == \1 as INT", rust)
    # C null/unsigned initializer semantics are source-authoritative; cheap
    # models often spell them as integer operations that Rust rejects.
    for field in re.findall(r"\b(m_[A-Za-z_]\w*)\s*\(\s*NULL\s*\)", source):
        rust = re.sub(
            rf"(\(\*this\)\.{re.escape(field)}\s*=\s*)0\s*;",
            r"\1core::ptr::null_mut();",
            rust,
        )
    for field in re.findall(r"\b(m_[A-Za-z_]\w*)\s*\(\s*-1\s*\)", source):
        rust = re.sub(
            rf"(\(\*this\)\.{re.escape(field)}\s*=\s*)-1\s*;",
            r"\1UINT::MAX;",
            rust,
        )
    for field in re.findall(r"\bDestroyIcon\s*\(\s*(m_[A-Za-z_]\w*)\s*\)", source):
        rust = re.sub(
            rf"\(\*this\)\.{re.escape(field)}\s*!=\s*0",
            f"!(*this).{field}.is_null()",
            rust,
        )
    bool_fields = set(re.findall(r"\bBOOL\s+([A-Za-z_]\w*)\s*;", source))
    for field in bool_fields:
        rust = re.sub(rf"\b{re.escape(field)}\s*:\s*false\b", f"{field}: FALSE", rust)
        rust = re.sub(rf"\b{re.escape(field)}\s*:\s*true\b", f"{field}: TRUE", rust)
    rust = re.sub(
        r"(StringCbPrintfW\(\s*[^,]+,\s*core::mem::size_of_val\([^)]*\))\s+as\s+u32",
        r"\1 as usize",
        rust,
    )
    rust = re.sub(r"\bStringCbPrintfW\s*\(", "StringCbPrintfW!(", rust)
    for alias, target in _CPP_SUPPORT_GUID_ALIASES.items():
        rust = re.sub(rf"\b{re.escape(alias)}\b", target, rust)
    owner = re.search(r"\b([A-Za-z_]\w*)::[~A-Za-z_]\w*\s*\(", source)
    if owner is not None:
        class_name = owner.group(1)
        explicit_receivers: dict[str, set[str]] = defaultdict(set)
        for match in re.finditer(r"\b([A-Za-z_]\w*)\s*(?:->|\.)\s*([A-Za-z_]\w*)\s*\(", source):
            explicit_receivers[match.group(2)].add(match.group(1))

        def insert_receiver(match: re.Match) -> str:
            target = match.group("target")
            first = match.group("first") or ""
            if re.search(r"\bfn\s*$", rust[max(0, match.start() - 8) : match.start()]):
                return match.group(0)
            method = target.removeprefix(f"{class_name}_").removesuffix("_body")
            if first == "this" or first in explicit_receivers.get(method, set()):
                return match.group(0)
            return f"{target}(this, " + first

        rust = re.sub(
            rf"\b(?P<target>{re.escape(class_name)}_[A-Za-z_]\w*_body)\s*\(\s*"
            r"(?P<first>[A-Za-z_]\w*)?",
            insert_receiver,
            rust,
        )
    rust = re.sub(
        r"let\s+(?P<name>[A-Za-z_]\w*)\s*=\s*"
        r"(?P<iid>core::ptr::addr_of!\(IID_[A-Za-z_]\w*\))\s*,\s*"
        r"core::ptr::addr_of_mut!\([^;]+?\)\.cast\(\)\s*;",
        r"let \g<name> = \g<iid>;",
        rust,
    )
    rust = re.sub(r"\(\(([^()]+)\)\s*<\s*0\)\s*(?:!=|<)\s*0", r"((\1) < 0)", rust)
    rust = re.sub(r"\bCComHeapPtr::new\(\s*\)", "CComHeapPtr::empty()", rust)
    rust = re.sub(
        r"alloc::boxed::Box::from_raw\(\s*([A-Za-z_]\w*)\s*\)\s*;",
        r"SENDTO_ITEM_delete(\1);",
        rust,
    )
    rust = re.sub(r"\bSENDTO_ITEM\s*\(", "SENDTO_ITEM_new(", rust)
    heap_ptr_names = set(re.findall(r"\bCComHeapPtr\s*<[^;=]+>\s+([A-Za-z_]\w*)", source)) & set(
        re.findall(r"\b([A-Za-z_]\w*)\s*:\s*CComHeapPtr\s*<", rust)
    )
    for name in heap_ptr_names:
        rust = re.sub(rf"&(?:raw\s+)?mut\s+{re.escape(name)}\b", f"{name}.as_mut_slot()", rust)
    com_ptr_names = set(re.findall(r"\bCComPtr\s*<[^;=]+>\s+([A-Za-z_]\w*)", source))
    for name in com_ptr_names:
        rust = re.sub(rf"\b{re.escape(name)}\.as_raw\(\)", name, rust)
    for field in re.findall(r"\bCComPtr\s*<[^;=]+>\s+(m_[A-Za-z_]\w*)", source):
        rust = re.sub(rf"(\(\*this\)\.{re.escape(field)})\.as_raw\(\)", r"\1", rust)
        rust = re.sub(
            rf"\(\*this\)\.{re.escape(field)}\.Release\(\)\s*;",
            f"if !(*this).{field}.is_null() {{\n"
            f"        let value = (*this).{field} as *mut IUnknown;\n"
            f"        ((*(*value).lpVtbl).Release)(value);\n"
            f"        (*this).{field} = core::ptr::null_mut();\n"
            "    }",
            rust,
        )
    rust = re.sub(r"(\(\*this\)\.m_[A-Za-z_]\w*)\.as_raw\(\)", r"\1", rust)
    state_releases = set(re.findall(r"\(\*this\)\.(m_[A-Za-z_]\w*)\.Release\(\)", rust))
    for field in state_releases:
        rust = re.sub(
            rf"\(\*this\)\.{re.escape(field)}\.Release\(\)\s*;",
            f"if !(*this).{field}.is_null() {{\n"
            f"        let value = (*this).{field} as *mut IUnknown;\n"
            f"        ((*(*value).lpVtbl).Release)(value);\n"
            f"        (*this).{field} = core::ptr::null_mut();\n"
            "    }",
            rust,
        )
    rust = re.sub(r"GetAsyncKeyState\((VK_[A-Za-z_]\w*)\)", r"GetAsyncKeyState(\1 as INT)", rust)
    rust = re.sub(r"\bCSIDL_SENDTO\b(?!\s+as\s+INT)", "CSIDL_SENDTO as INT", rust)
    if re.search(
        r"GetSpecialFolder(?:_body)?\s*\(\s*NULL\s*,[^;]+CSIDL_SENDTO\s*\)\s*;",
        source,
    ):
        rust = re.sub(
            r"(CSIDL_SENDTO\s+as\s+INT\s*,?\s*)\)",
            r"\1core::ptr::null_mut())",
            rust,
            count=1,
        )
    rust = re.sub(
        r"((?:InsertMenuW|AppendMenuW)\([^;]*?)\bidCmd\s*,",
        r"\1idCmd as UINT_PTR,",
        rust,
        flags=re.S,
    )
    rust = re.sub(r"let\s+(idCmd)\s*:\s*WORD\s*=\s*LOWORD!", r"let \1: UINT = LOWORD!", rust)
    rust = re.sub(
        r"&\(\*([A-Za-z_]\w*)\)\.pidlChild\s+as\s+\*const\s+PITEMID_CHILD",
        r"core::ptr::addr_of!((*\1).pidlChild).cast::<PCUITEMID_CHILD>()",
        rust,
    )
    rust = re.sub(r"(itemWidth\s*[<=]\s*)(cxMenuCheck)\b", r"\1\2 as UINT", rust)
    rust = re.sub(r"(itemWidth\s*=\s*)(cxMenuCheck)\b", r"\1\2 as UINT", rust)
    rust = re.sub(r"(itemHeight\s*[<=]\s*)(cySmall)\b", r"\1\2 as UINT", rust)
    rust = re.sub(r"(itemHeight\s*=\s*)(cySmall)\b", r"\1\2 as UINT", rust)
    rust = re.sub(r"\b_wcsdup\(([A-Za-z_]\w*\.[A-Za-z_]\w*)\)", r"_wcsdup(\1.as_ptr())", rust)
    rust = re.sub(r"\bLOWORD\s*\(", "LOWORD!(", rust)
    if re.search(r"\*\s*plResult\s*=\s*TRUE", source):
        rust = re.sub(r"\*plResult\s*=\s*TRUE", "*plResult = TRUE as LRESULT", rust)
    rust = re.sub(
        r"&mut\s+(wsz[A-Za-z_]\w*)\s+as\s+\*mut\s+_",
        r"\1.as_mut_ptr().cast()",
        rust,
    )
    rust = re.sub(r"(?m)^(\s*)pData,(\s*)$", r"\1pData.cast(),\2", rust)
    rust = re.sub(r"\b(fi\.hIcon)\s*!=\s*0", r"!\1.is_null()", rust)
    rust = re.sub(
        r"(RegEnumKeyW\([^;]*?),\s*MAX_PATH(\s*\))",
        r"\1, MAX_PATH as DWORD\2",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"(RegEnumKeyW\([^;]*?),\s*MAX_PATH(\s*,\s*\))",
        r"\1, MAX_PATH as DWORD\2",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"(ExpandEnvironmentStringsW\([^;]*?),\s*MAX_PATH(\s*\))",
        r"\1, MAX_PATH as DWORD\2",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"if\s+!([A-Za-z_]\w*_body\([^;{}]*\))",
        r"if \1 == FALSE",
        rust,
    )
    rust = re.sub(
        r"(LoadStringW\([^;]*?,\s*)([A-Za-z_]\w*\.len\(\))(\s*,?\s*\))",
        r"\1\2 as INT\3",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"(\bmii\.cch\s*=\s*wcslen\([^;]+\))\s*;",
        r"\1 as UINT;",
        rust,
    )
    rust = re.sub(
        r"(CNewMenu_SelectNewItem_body\(\s*this\s*,\s*)(SHCNE_[A-Za-z_]\w*)",
        r"\1\2 as LONG",
        rust,
    )
    rust = re.sub(
        r"(CNewMenu_NewItemByNonCommand_body\([^;]*?wszName\.as_mut_ptr\(\),\s*)"
        r"wszName\.len\(\)",
        r"\1wszName.len() as DWORD",
        rust,
        flags=re.S,
    )
    rust = re.sub(
        r"\blet\s+mut\s+([A-Za-z_]\w*)\s*:\s*HKEY\s*;",
        r"let mut \1: HKEY = core::ptr::null_mut();",
        rust,
    )
    rust = re.sub(r"\blet\s+(wszName)\s*:\s*\*mut", r"let mut \1: *mut", rust)
    if "CreateProcessW" in source:
        rust = rust.replace("core::ptr::null::<u16>()", "core::ptr::null()")
    rust = re.sub(
        r"MessageBoxW\(\s*0\s*,",
        "MessageBoxW(core::ptr::null_mut(),",
        rust,
    )
    if "switch" in source:
        match_start = rust.find("match ")
        if match_start >= 0:
            opening = rust.find("{", match_start)
            try:
                _inner, closing = _balanced_block(rust, opening)
            except CppComError:
                closing = -1
            if closing >= 0 and "_ =>" not in rust[opening:closing]:
                rust = rust[:closing] + "    _ => {}\n" + rust[closing:]
    rust = re.sub(
        r"(pshf:\s*)this\b",
        r"\1core::ptr::addr_of_mut!((*this).IShellFolder_iface)",
        rust,
    )
    rust = rust.replace("(*psd).str", "(*psd).str_")
    rust = rust.replace("(*psd).str_.cStr", "(*psd).str_.u.cStr")
    rust = re.sub(r"(\.cStr\[[^]]+\]\s*=\s*)0(?:u16)?\b", r"\g<1>0i8", rust)
    rust = re.sub(
        r"SHSetStrRet\(\s*&raw\s+mut\s+([^,]+),\s*"
        r"([A-Za-z_]\w*\[[^]]+\]\.colnameid)\s*\)",
        r"cpp_com_SHSetStrRetResource(&raw mut \1, \2 as DWORD)",
        rust,
    )
    rust = re.sub(
        r"SHSetStrRet\(\s*([A-Za-z_]\w*)\s*,\s*"
        r"\(\*([A-Za-z_]\w*)\)\.([A-Za-z_]\w*)\s*\)",
        r"cpp_com_SHSetStrRetW(\1, core::ptr::addr_of!((*\2).\3).cast::<WCHAR>())",
        rust,
    )
    return rust


_CPP_SUPPORT_EXTERNS = {
    "CloseHandle": "    fn CloseHandle(handle: HANDLE) -> BOOL;",
    "CopyFileW": "    fn CopyFileW(existing: LPCWSTR, new: LPCWSTR, fail_if_exists: BOOL) -> BOOL;",
    "CreateFileW": (
        "    fn CreateFileW(name: LPCWSTR, access: DWORD, share: DWORD, security: LPVOID, "
        "creation: DWORD, flags: DWORD, template: HANDLE) -> HANDLE;"
    ),
    "CreateMenu": "    fn CreateMenu() -> HMENU;",
    "DestroyMenu": "    fn DestroyMenu(menu: HMENU) -> BOOL;",
    "GetAsyncKeyState": "    fn GetAsyncKeyState(key: INT) -> i16;",
    "InsertMenuW": (
        "    fn InsertMenuW(menu: HMENU, position: UINT, flags: UINT, id: UINT_PTR, "
        "text: LPCWSTR) -> BOOL;"
    ),
    "AppendMenuW": (
        "    fn AppendMenuW(menu: HMENU, flags: UINT, id: UINT_PTR, text: LPCWSTR) -> BOOL;"
    ),
    "SetMenuItemInfoW": (
        "    fn SetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, "
        "info: *const MENUITEMINFOW) -> BOOL;"
    ),
    "CreateProcessW": (
        "    fn CreateProcessW(app: LPCWSTR, command: LPWSTR, process_attrs: LPCVOID, "
        "thread_attrs: LPCVOID, inherit: BOOL, flags: DWORD, environment: LPCVOID, "
        "directory: LPCWSTR, startup: *const STARTUPINFOW, info: *mut PROCESS_INFORMATION) -> BOOL;"
    ),
    "DestroyIcon": "    fn DestroyIcon(icon: HICON) -> BOOL;",
    "CopyIcon": "    fn CopyIcon(icon: HICON) -> HICON;",
    "DestroyWindow": "    fn DestroyWindow(window: HWND) -> BOOL;",
    "DefWindowProcW": (
        "    fn DefWindowProcW(window: HWND, message: UINT, wparam: WPARAM, "
        "lparam: LPARAM) -> LRESULT;"
    ),
    "DispatchMessageW": "    fn DispatchMessageW(message: *const MSG) -> LRESULT;",
    "GetMessageW": (
        "    fn GetMessageW(message: *mut MSG, window: HWND, first: UINT, last: UINT) -> BOOL;"
    ),
    "GetWindowLongPtrW": (
        '    #[cfg_attr(target_pointer_width = "32", link_name = "GetWindowLongW")]\n'
        "    fn GetWindowLongPtrW(window: HWND, index: INT) -> LONG_PTR;"
    ),
    "KillTimer": "    fn KillTimer(window: HWND, id: UINT_PTR) -> BOOL;",
    "LoadIcon": (
        '    #[link_name = "LoadIconW"]\n'
        "    fn LoadIcon(instance: HINSTANCE, name: LPCWSTR) -> HICON;"
    ),
    "LoadIconW": "    fn LoadIconW(instance: HINSTANCE, name: LPCWSTR) -> HICON;",
    "PlaySoundW": ("    fn PlaySoundW(sound: LPCWSTR, module: HANDLE, flags: DWORD) -> BOOL;"),
    "PostQuitMessage": "    fn PostQuitMessage(exit_code: INT);",
    "SetTimer": (
        "    fn SetTimer(window: HWND, id: UINT_PTR, timeout: UINT, callback: "
        'Option<unsafe extern "system" fn(HWND, UINT, UINT_PTR, DWORD)>) -> UINT_PTR;'
    ),
    "SetWindowLongPtrW": (
        '    #[cfg_attr(target_pointer_width = "32", link_name = "SetWindowLongW")]\n'
        "    fn SetWindowLongPtrW(window: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;"
    ),
    "SHCreateWorkerWindowW": (
        "    fn SHCreateWorkerWindowW(callback: WNDPROC, parent: HWND, ex_style: DWORD, "
        "style: DWORD, menu: HMENU, extra: LONG_PTR) -> HWND;"
    ),
    "Shell_NotifyIconW": (
        "    fn Shell_NotifyIconW(message: DWORD, data: *mut NOTIFYICONDATAW) -> BOOL;"
    ),
    "TranslateMessage": "    fn TranslateMessage(message: *const MSG) -> BOOL;",
    "DrawIconEx": (
        "    fn DrawIconEx(dc: HDC, x: INT, y: INT, icon: HICON, width: INT, height: INT, "
        "step: UINT, brush: HANDLE, flags: UINT) -> BOOL;"
    ),
    "ExpandEnvironmentStringsW": (
        "    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;"
    ),
    "ExtractIconExW": (
        "    fn ExtractIconExW(file: LPCWSTR, index: INT, large: *mut HICON, small: *mut HICON, "
        "icons: UINT) -> UINT;"
    ),
    "GetMenuItemInfoW": (
        "    fn GetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, "
        "info: *mut MENUITEMINFOW) -> BOOL;"
    ),
    "GetSystemMetrics": "    fn GetSystemMetrics(index: UINT) -> INT;",
    "IUnknown_QueryService": (
        "    fn IUnknown_QueryService(object: *mut IUnknown, service: REFGUID, riid: REFIID, "
        "out: *mut *mut core::ffi::c_void) -> HRESULT;"
    ),
    "LoadImage": (
        '    #[link_name = "LoadImageW"]\n'
        "    fn LoadImage(instance: HINSTANCE, name: LPCWSTR, kind: UINT, width: INT, "
        "height: INT, flags: UINT) -> HANDLE;"
    ),
    "MessageBoxW": (
        "    fn MessageBoxW(owner: HWND, text: LPCWSTR, caption: LPCWSTR, kind: UINT) -> INT;"
    ),
    "PathYetAnotherMakeUniqueName": (
        "    fn PathYetAnotherMakeUniqueName(buffer: LPWSTR, path: LPCWSTR, short: LPCWSTR, "
        "long: LPCWSTR) -> BOOL;"
    ),
    "RegCloseKey": "    fn RegCloseKey(key: HKEY) -> LONG;",
    "RegCreateKeyEx": (
        '    #[link_name = "RegCreateKeyExW"]\n'
        "    fn RegCreateKeyEx(key: HKEY, subkey: LPCWSTR, reserved: DWORD, class: LPWSTR, "
        "options: DWORD, access: REGSAM, security: LPVOID, result: PHKEY, "
        "disposition: LPDWORD) -> LONG;"
    ),
    "RegEnumKeyW": (
        "    fn RegEnumKeyW(key: HKEY, index: DWORD, name: LPWSTR, capacity: DWORD) -> LONG;"
    ),
    "RegGetValueW": (
        "    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, "
        "kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;"
    ),
    "RegOpenKeyExW": (
        "    fn RegOpenKeyExW(key: HKEY, subkey: LPCWSTR, options: DWORD, access: REGSAM, "
        "result: PHKEY) -> LONG;"
    ),
    "RegQueryValueExW": (
        "    fn RegQueryValueExW(key: HKEY, value: LPCWSTR, reserved: LPDWORD, kind: LPDWORD, "
        "data: PBYTE, bytes: LPDWORD) -> LONG;"
    ),
    "RegSetValueExW": (
        "    fn RegSetValueExW(key: HKEY, value: LPCWSTR, reserved: DWORD, kind: DWORD, "
        "data: *const BYTE, bytes: DWORD) -> LONG;"
    ),
    "SHChangeNotify": (
        "    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);"
    ),
    "SHCreateDirectory": "    fn SHCreateDirectory(owner: HWND, path: LPCWSTR) -> INT;",
    "SHGetFileInfoW": (
        "    fn SHGetFileInfoW(path: LPCWSTR, attributes: DWORD, info: *mut SHFILEINFOW, "
        "bytes: UINT, flags: UINT) -> DWORD_PTR;"
    ),
    "SHGetPathFromIDListW": (
        "    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, path: LPWSTR) -> BOOL;"
    ),
    "SHILCreateFromPathW": (
        "    fn SHILCreateFromPathW(path: LPCWSTR, pidl: *mut LPITEMIDLIST, "
        "attributes: LPDWORD) -> HRESULT;"
    ),
    "StrRetToStrW": (
        "    fn StrRetToStrW(value: *mut STRRET, pidl: PCUITEMID_CHILD, out: *mut LPWSTR) "
        "-> HRESULT;"
    ),
    "ILCombine": (
        "    fn ILCombine(parent: PCIDLIST_ABSOLUTE, child: PCUITEMID_CHILD) -> PIDLIST_ABSOLUTE;"
    ),
    "WideCharToMultiByte": (
        "    fn WideCharToMultiByte(page: UINT, flags: DWORD, wide: LPCWSTR, wide_len: INT, "
        "multi: LPSTR, multi_len: INT, default_char: LPCSTR, used_default: *mut BOOL) -> INT;"
    ),
    "WriteFile": (
        "    fn WriteFile(file: HANDLE, buffer: LPCVOID, bytes: DWORD, written: LPDWORD, "
        "overlapped: LPVOID) -> BOOL;"
    ),
    "GlobalAlloc": "    fn GlobalAlloc(flags: UINT, bytes: SIZE_T) -> HGLOBAL;",
    "GlobalLock": "    fn GlobalLock(memory: HGLOBAL) -> LPVOID;",
    "GlobalUnlock": "    fn GlobalUnlock(memory: HGLOBAL) -> BOOL;",
    "GlobalFree": "    fn GlobalFree(memory: HGLOBAL) -> HGLOBAL;",
    "OpenClipboard": "    fn OpenClipboard(owner: HWND) -> BOOL;",
    "EmptyClipboard": "    fn EmptyClipboard() -> BOOL;",
    "SetClipboardData": "    fn SetClipboardData(format: UINT, memory: HANDLE) -> HANDLE;",
    "CloseClipboard": "    fn CloseClipboard() -> BOOL;",
    "GetLastError": "    fn GetLastError() -> DWORD;",
    "lstrlenW": "    fn lstrlenW(text: LPCWSTR) -> INT;",
    "CoTaskMemAlloc": "    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;",
    "LoadStringW": (
        "    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;"
    ),
    "CPrintersExtractIconW_CreateInstane": (
        '    #[link_name = "CPrintersExtractIconW_RustBridge"]\n'
        "    fn CPrintersExtractIconW_CreateInstane(pidl: LPCITEMIDLIST, riid: REFIID, "
        "out: *mut LPVOID) -> HRESULT;"
    ),
    "CPrintersEnum_CreateInstance": (
        "    fn CPrintersEnum_CreateInstance(owner: HWND, flags: DWORD, "
        "out: *mut *mut IEnumIDList) -> HRESULT;"
    ),
    "StrChrW": "    fn StrChrW(text: LPCWSTR, ch: WCHAR) -> LPWSTR;",
    "CoTaskMemFree": "    fn CoTaskMemFree(memory: LPVOID);",
    "SHStrDupW": "    fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;",
    "ILClone": "    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;",
    "ILFindLastID": "    fn ILFindLastID(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;",
    "ILFree": "    fn ILFree(pidl: LPITEMIDLIST);",
    "ILRemoveLastID": "    fn ILRemoveLastID(pidl: LPITEMIDLIST) -> BOOL;",
    "SHGetDesktopFolder": ("    fn SHGetDesktopFolder(folder: *mut *mut IShellFolder) -> HRESULT;"),
    "SHGetNameFromIDList": (
        "    fn SHGetNameFromIDList(pidl: PCIDLIST_ABSOLUTE, sigdn: SIGDN, "
        "name: *mut PWSTR) -> HRESULT;"
    ),
    "SHCreateShellItem": (
        "    fn SHCreateShellItem(parent: PCIDLIST_ABSOLUTE, folder: *mut IShellFolder, "
        "child: LPCITEMIDLIST, item: *mut *mut IShellItem) -> HRESULT;"
    ),
    "SHCreateShellFolderView": (
        "    fn SHCreateShellFolderView(create: *const SFV_CREATE, "
        "view: *mut *mut IShellView) -> HRESULT;"
    ),
    "SHQueryRecycleBinW": (
        "    fn SHQueryRecycleBinW(root: LPCWSTR, info: *mut SHQUERYRBINFO) -> HRESULT;"
    ),
    "SHEmptyRecycleBinW": (
        "    fn SHEmptyRecycleBinW(owner: HWND, root: LPCWSTR, flags: DWORD) -> HRESULT;"
    ),
    "SHGetSpecialFolderLocation": (
        "    fn SHGetSpecialFolderLocation(owner: HWND, csidl: INT, "
        "pidl: *mut LPITEMIDLIST) -> HRESULT;"
    ),
    "ShellExecuteExW": "    fn ShellExecuteExW(info: *mut SHELLEXECUTEINFOW) -> BOOL;",
    "ILGetNext": "    fn ILGetNext(pidl: PCUIDLIST_RELATIVE) -> LPITEMIDLIST;",
    "SHELL32_CoCreateInitSF": (
        '    #[link_name = "Shell32RustCoCreateInitSF"]\n'
        "    fn SHELL32_CoCreateInitSF(root: LPCITEMIDLIST, clsid: *const GUID, "
        "csidl: INT, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;"
    ),
    "InsertMenuItemW": (
        "    fn InsertMenuItemW(menu: HMENU, item: UINT, by_position: BOOL, "
        "info: *const MENUITEMINFOW) -> BOOL;"
    ),
    "SHCreateShellItemArrayFromDataObject": (
        "    fn SHCreateShellItemArrayFromDataObject(data: *mut IDataObject, "
        "riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;"
    ),
    "SHELL_MapContextMenuVerbToCmdId": (
        '    #[link_name = "Shell32RustMapContextMenuVerbToCmdId"]\n'
        "    fn SHELL_MapContextMenuVerbToCmdId(info: LPCMINVOKECOMMANDINFO, "
        "map: *const CMVERBMAP) -> INT;"
    ),
    "SHELL_GetCommandStringImpl": (
        '    #[link_name = "Shell32RustGetCommandStringImpl"]\n'
        "    fn SHELL_GetCommandStringImpl(command: SIZE_T, flags: UINT, buffer: LPSTR, "
        "capacity: UINT, map: *const CMVERBMAP) -> HRESULT;"
    ),
}

_CPP_SUPPORT_C_EXTERNS = {
    "SHELL_GetUIObjectOfAbsoluteItem": (
        "    fn SHELL_GetUIObjectOfAbsoluteItem(owner: HWND, pidl: PCIDLIST_ABSOLUTE, "
        "riid: REFIID, out: *mut LPVOID) -> HRESULT;"
    ),
    "HLM_GetIconW": (
        "    fn HLM_GetIconW(index: UINT, destination: LPWSTR, length: DWORD, "
        "icon_index: *mut INT) -> BOOL;"
    ),
    "_ILIsDesktop": "    fn _ILIsDesktop(pidl: LPCITEMIDLIST) -> BOOL;",
    "_ILIsSpecialFolder": "    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;",
    "_ILGetDataPointer": "    fn _ILGetDataPointer(pidl: LPCITEMIDLIST) -> *mut PIDLDATA;",
    "SHELL32_CompareDetails": (
        '    #[link_name = "CPrinterFolder_CompareDetails_RustBridge"]\n'
        "    fn SHELL32_CompareDetails(folder: *mut IShellFolder2, lparam: LPARAM, "
        "left: PCUIDLIST_RELATIVE, right: PCUIDLIST_RELATIVE) -> HRESULT;"
    ),
    "_ILCreateMyDocuments": "    fn _ILCreateMyDocuments() -> LPITEMIDLIST;",
    "memcpy": (
        '    #[link_name = "memcpy"]\n'
        "    fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, "
        "bytes: usize) -> *mut core::ffi::c_void;"
    ),
    "malloc": "    fn malloc(bytes: usize) -> *mut ();",
    "wcsstr": "    fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR;",
    "_wcsdup": "    fn _wcsdup(text: LPCWSTR) -> LPWSTR;",
    "_wcsicmp": "    fn _wcsicmp(left: LPCWSTR, right: LPCWSTR) -> INT;",
}

_CPP_SUPPORT_GUIDS = {
    "BHID_DataObject",
    "BHID_SFObject",
    "BHID_SFUIObject",
    "BHID_SFViewObject",
    "CLSID_ShellItem",
    "CLSID_AdminFolderShortcut",
    "CLSID_FontsFolderShortcut",
    "CLSID_ShellFSFolder",
    "CLSID_MyDocuments",
    "IID_IDataObject",
    "IID_IShellFolder",
    "IID_IShellFolder2",
}
_CPP_SUPPORT_GUID_ALIASES = {"SID_SFolderView": "IID_IFolderView"}


def _render_cmverbmap(source: str) -> str:
    match = re.search(
        r"static\s+const\s+CMVERBMAP\s+([A-Za-z_]\w*)\s*\[\s*\]\s*=\s*\{(?P<body>.*?)\}\s*;",
        source,
        flags=re.S,
    )
    if match is None:
        return ""
    entries = []
    for item in re.finditer(
        r"\{\s*(?P<verb>\"(?:\\.|[^\"])*\"|NULL)\s*(?:,\s*(?P<cmd>[A-Za-z_]\w*|\d+))?\s*\}",
        match.group("body"),
    ):
        token, command = item.group("verb"), item.group("cmd") or "0"
        verb = "core::ptr::null()" if token == "NULL" else f'b{token[:-1]}\\0".as_ptr() as LPCSTR'
        entries.append(f"    CMVERBMAP {{ Verb: {verb}, CmdId: {command} as WORD }},")
    if not entries:
        raise CppComError("CMVERBMAP table was found but no aggregate entries were parsed")
    return "\n".join(
        [
            "#[repr(C)]",
            "pub struct CMVERBMAP { pub Verb: LPCSTR, pub CmdId: WORD }",
            f"pub static mut {match.group(1)}: [CMVERBMAP; {len(entries)}] = [",
            *entries,
            "];",
        ]
    )


def _render_shvheader_tables(source: str, used_words: set[str]) -> str:
    """Render shell folder column tables from their file-scope C authority."""
    chunks = []
    table = re.compile(
        r"static\s+(?:const\s+)?shvheader\s+([A-Za-z_]\w*)\s*\[\s*\]\s*=\s*"
        r"\{(?P<body>.*?)\}\s*;",
        re.S,
    )
    for match in table.finditer(source):
        name = match.group(1)
        if name not in used_words:
            continue
        entries = []
        for item in re.finditer(r"\{([^{}]+)\}", match.group("body")):
            fields = [part.strip() for part in item.group(1).split(",")]
            if len(fields) != 4 or any(not field for field in fields):
                raise CppComError(f"{name}: shvheader entry is not a four-field aggregate")
            entries.append(
                "    shvheader { colnameid: "
                + f"({fields[0]}) as WORD, colstate: ({fields[1]}) as WORD, "
                + f"fmt: ({fields[2]}) as WORD, cxChar: ({fields[3]}) as WORD }},"
            )
        if not entries:
            raise CppComError(f"{name}: shvheader table has no parsed entries")
        chunks.extend(
            [
                "#[repr(C)]",
                "pub struct shvheader {",
                "    pub colnameid: WORD, pub colstate: WORD,",
                "    pub fmt: WORD, pub cxChar: WORD,",
                "}",
                f"pub static {name}: [shvheader; {len(entries)}] = [",
                *entries,
                "];",
            ]
        )
    return "\n".join(chunks)


_CPP_SUPPORT_CONSTANTS = {
    "CSIDL_BITBUCKET": ("INT", "0x000a"),
    "CSIDL_PERSONAL": ("INT", "0x0005"),
    "EVCCBF_LASTNOTIFICATION": ("DWORD", "0x0001"),
    "E_UNEXPECTED": ("HRESULT", "0x8000_FFFFu32 as HRESULT"),
    "EVCF_HASSETTINGS": ("DWORD", "0x0001"),
    "IDI_ERROR": ("LPCWSTR", "32513usize as LPCWSTR"),
    "IDI_INFORMATION": ("LPCWSTR", "32516usize as LPCWSTR"),
    "IDI_WARNING": ("LPCWSTR", "32515usize as LPCWSTR"),
    "IDI_WINLOGO": ("LPCWSTR", "32517usize as LPCWSTR"),
    "MAX_PATH": ("usize", "260"),
    "NIN_BALLOONHIDE": ("DWORD", "WM_USER + 3"),
    "NIN_BALLOONSHOW": ("DWORD", "WM_USER + 2"),
    "NIN_BALLOONTIMEOUT": ("DWORD", "WM_USER + 4"),
    "NIN_BALLOONUSERCLICK": ("DWORD", "WM_USER + 5"),
    "SEE_MASK_IDLIST": ("ULONG", "0x0000_0004"),
    "SHERB_NOCONFIRMATION": ("DWORD", "0x0000_0001"),
    "SHERB_NOPROGRESSUI": ("DWORD", "0x0000_0002"),
    "SHERB_NOSOUND": ("DWORD", "0x0000_0004"),
    "SW_SHOWNORMAL": ("INT", "1"),
    "WM_TRAYNOTIFY": ("UINT", "WM_USER + 100"),
    "SFGAO_CANCOPY": ("SFGAOF", "0x0000_0001"),
    "SFGAO_CANLINK": ("SFGAOF", "0x0000_0004"),
    "SFGAO_CANRENAME": ("SFGAOF", "0x0000_0010"),
    "SFGAO_CANDELETE": ("SFGAOF", "0x0000_0020"),
    "SFGAO_HASPROPSHEET": ("SFGAOF", "0x0000_0040"),
}


def _sdk_numeric_definitions(source_path: Path, used_words: set[str]) -> dict[str, str]:
    """Resolve simple referenced SDK numeric facts in one indexed header pass.

    Only architecture-invariant literal definitions are admitted. Expressions,
    aliases, casts, and conflicting spellings remain unresolved and visible to
    the substrate planner.
    """
    repo_root = next(
        (parent for parent in source_path.parents if (parent / "sdk/include").is_dir()), None
    )
    if repo_root is None:
        return {}
    pattern = (
        r"^\s*#\s*define\s+[A-Za-z_]\w*\s+(?:0[xX][0-9A-Fa-f]+|[0-9]+)[uUlL]*\b"
        r"|^\s*[A-Za-z_]\w*\s*=\s*(?:0[xX][0-9A-Fa-f]+|[0-9]+)[uUlL]*\s*,"
    )
    search_roots = [repo_root / "sdk/include"] + [
        repo_root / f"build-{arch}" / "sdk" / "include" / "psdk"
        for arch in ("i386", "amd64", "arm64")
    ]
    process = subprocess.run(
        [
            "rg",
            "-n",
            "--no-heading",
            "--glob",
            "*.h",
            pattern,
            *(str(root) for root in search_roots if root.is_dir()),
        ],
        capture_output=True,
        text=True,
    )
    if process.returncode not in (0, 1):
        return {}
    values: dict[str, set[str]] = defaultdict(set)
    definition = re.compile(
        r"^\s*(?:#\s*define\s+)?(?P<name>[A-Za-z_]\w*)\s*(?:=\s*|\s+)"
        r"(?P<value>0[xX][0-9A-Fa-f]+|[0-9]+)[uUlL]*\b"
    )
    for line in process.stdout.splitlines():
        parts = line.split(":", 2)
        if len(parts) != 3:
            continue
        match = definition.match(parts[2])
        if (
            match is not None
            and match.group("name") in used_words
            and match.group("name") not in {"false", "true"}
        ):
            values[match.group("name")].add(match.group("value"))
    return {name: next(iter(found)) for name, found in values.items() if len(found) == 1}


def _shared_substrate_type_names(repo_root: Path) -> set[str]:
    names = set()
    for path in (
        repo_root / "sdk/lib/rust/shell32-rust/src/types.rs",
        repo_root / "sdk/lib/rust/shell32-rust/src/shellcom.rs",
    ):
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        names.update(
            re.findall(r"\bpub\s+(?:struct|union|enum|type)\s+(?:r#)?([A-Za-z_]\w*)", text)
        )
    return names


def _shared_substrate_value_names(repo_root: Path) -> set[str]:
    names = set()
    for path in (
        repo_root / "sdk/lib/rust/shell32-rust/src/types.rs",
        repo_root / "sdk/lib/rust/shell32-rust/src/shellcom.rs",
    ):
        if path.is_file():
            names.update(
                re.findall(
                    r"\bpub\s+(?:const|static(?:\s+mut)?)\s+([A-Za-z_]\w*)",
                    path.read_text(encoding="utf-8", errors="replace"),
                )
            )
    return names


def _sdk_pod_support(source_path: Path, used_words: set[str]) -> str:
    """Generate the simple referenced SDK POD closure from header authority."""
    repo_root = next(
        (parent for parent in source_path.parents if (parent / "sdk/include").is_dir()), None
    )
    if repo_root is None:
        return ""
    shared = _shared_substrate_type_names(repo_root)
    variants: dict[str, set[tuple[str, tuple[tuple[str, bool], ...], bool]]] = defaultdict(set)
    typedef = re.compile(
        r"typedef\s+struct(?:\s+[A-Za-z_]\w*)?\s*\{(?P<body>[^{}]*)\}\s*"
        r"(?P<aliases>[^;]+);",
        re.S,
    )
    header_roots = [repo_root / "sdk/include"] + [
        repo_root / f"build-{arch}" / "sdk" / "include" / "psdk"
        for arch in ("i386", "amd64", "arm64")
    ]
    headers = sorted(path for root in header_roots if root.is_dir() for path in root.rglob("*.h"))
    for header in headers:
        text = header.read_text(encoding="utf-8", errors="replace")
        for match in typedef.finditer(text):
            aliases = []
            for raw in match.group("aliases").split(","):
                token = " ".join(raw.split())
                alias = re.fullmatch(r"(?P<pointer>\*+\s*)?(?P<name>[A-Za-z_]\w*)", token)
                if alias is not None:
                    aliases.append((alias.group("name"), bool(alias.group("pointer"))))
            base = next((name for name, pointer in aliases if not pointer), None)
            if base is None:
                continue
            prefix = text[: match.start()]
            packed = prefix.rfind("pshpack1.h") > prefix.rfind("poppack.h")
            body = " ".join(match.group("body").split())
            variants[base].add((body, tuple(aliases), packed))

    # Identical repeated declarations are harmless; conflicting layouts or
    # aliases are not. Keep ambiguity visible to the compiler/substrate plan.
    records = {
        base: (body, list(aliases), packed)
        for base, found in variants.items()
        if len(found) == 1
        for body, aliases, packed in found
    }

    alias_to_base = {
        alias: base for base, (_body, aliases, _packed) in records.items() for alias, _ in aliases
    }
    selected = {
        alias_to_base[name]
        for name in used_words - shared
        if name in alias_to_base and alias_to_base[name] not in shared
    }
    # This SDK record contains optional interface pointers whose pointee types
    # are not consistently exposed as compiler-facing vtables.  Its layout is
    # stable and owned by the explicit support block below, which represents
    # those ABI-equivalent optional pointers as c_void.
    selected.discard("SFV_CREATE")
    # Embedded SDK records are part of the same deterministic layout closure.
    changed = True
    while changed:
        changed = False
        for base in list(selected):
            body = records[base][0]
            for dependency in set(re.findall(r"\b[A-Za-z_]\w*\b", body)) & alias_to_base.keys():
                target = alias_to_base[dependency]
                if target not in selected and target not in shared:
                    selected.add(target)
                    changed = True

    all_bounds = {
        bound
        for base in selected
        for bound in re.findall(r"\[\s*([A-Za-z_]\w*)\s*\]", records[base][0])
    }
    numeric = {
        name: int(value, 0)
        for name, value in _sdk_numeric_definitions(source_path, all_bounds).items()
    }
    chunks = []
    for base in sorted(selected):
        body, aliases, packed = records[base]
        declarations = [
            re.sub(r"\b_[A-Za-z0-9_]+_\b(?:\s*\([^)]*\))?\s*", "", " ".join(item.split()))
            for item in body.split(";")
            if item.strip()
        ]
        fields = []
        try:
            for declaration in declarations:
                fields.append(
                    _parse_plain_state_field(
                        declaration,
                        allowed_pod_types=set(records) | _POD_LEAF_TYPES,
                        array_constants=numeric,
                    )
                )
        except CppComError:
            continue
        representation = "#[repr(C, packed)]" if packed else "#[repr(C)]"
        lines = [representation, f"pub struct {base} {{"]
        for state_field in fields:
            field_name = (
                f"r#{state_field.name}" if state_field.name in _RUST_KEYWORDS else state_field.name
            )
            lines.append(f"    pub {field_name}: {state_field.rust_type},")
        lines.append("}")
        for alias, pointer in aliases:
            if alias != base and (alias in used_words or alias in shared):
                target = f"*mut {base}" if pointer else base
                lines.append(f"pub type {alias} = {target};")
        chunks.append("\n".join(lines))
    return "\n\n".join(chunks)


def _sdk_interface_support(
    source_path: Path, used_words: set[str], declared_types: set[str]
) -> tuple[str, set[str], set[str]]:
    """Render the flattened SDK interface closure referenced by worker bodies.

    Slot order and signatures come only from compiler-facing WIDL headers. All
    available architecture headers must agree before one shared Rust vtable is
    emitted.
    """
    repo_root = next(
        (parent for parent in source_path.parents if (parent / "sdk/include").is_dir()), None
    )
    if repo_root is None:
        return "", set(), set()
    indexes: dict[str, dict[str, Path]] = {}
    shared_found: dict[str, list[Path]] = defaultdict(list)
    shared_root = repo_root / "sdk" / "include" / "reactos"
    if shared_root.is_dir():
        for header in sorted(shared_root.glob("*.h")):
            text = header.read_text(encoding="utf-8", errors="replace")
            names = set(re.findall(r"typedef\s+struct\s+([A-Za-z_]\w*)Vtbl\b", text))
            names.update(re.findall(r"DECLARE_INTERFACE_\s*\(\s*([A-Za-z_]\w*)\s*,", text))
            for name in names:
                shared_found[name].append(header)
    shared_index = {name: paths[0] for name, paths in shared_found.items() if len(paths) == 1}
    for arch in ("i386", "amd64", "arm64"):
        root = repo_root / f"build-{arch}" / "sdk" / "include" / "psdk"
        if not root.is_dir():
            continue
        found: dict[str, list[Path]] = defaultdict(list)
        for header in sorted(root.glob("*.h")):
            text = header.read_text(encoding="utf-8", errors="replace")
            names = set(re.findall(r"typedef\s+struct\s+([A-Za-z_]\w*)Vtbl\b", text))
            names.update(re.findall(r"DECLARE_INTERFACE_\s*\(\s*([A-Za-z_]\w*)\s*,", text))
            for name in names:
                found[name].append(header)
        index = {name: paths[0] for name, paths in found.items() if len(paths) == 1}
        for name, header in shared_index.items():
            index.setdefault(name, header)
        indexes[arch] = index
    if not indexes:
        return "", set(), set()
    reference_arch = "amd64" if "amd64" in indexes else next(iter(indexes))
    reference = indexes[reference_arch]
    queue = sorted((used_words & reference.keys()) - declared_types)
    specs: dict[str, InterfaceSpec] = {}
    while queue:
        name = queue.pop(0)
        if name in specs or name in declared_types:
            continue
        spec = parse_widl_interface(reference[name], name)
        for arch, index in indexes.items():
            if arch == reference_arch or name not in index:
                continue
            other = parse_widl_interface(index[name], name)
            if other.methods != spec.methods:
                raise CppComError(
                    f"{name}: compiler-facing WIDL vtable differs on {reference_arch}/{arch}"
                )
        specs[name] = spec
        # Include return and parameter interface pointees recursively.
        dependencies: set[str] = set()
        for method in spec.methods:
            dependencies.update(
                token
                for c_type, _param in method.params
                for token in re.findall(r"\bI[A-Z][A-Za-z0-9_]*\b", c_type)
            )
            dependencies.update(re.findall(r"\bI[A-Z][A-Za-z0-9_]*\b", method.return_type))
        queue.extend(sorted((dependencies & reference.keys()) - specs.keys() - declared_types))
    rendered = "\n\n".join(render_interface(specs[name]) for name in sorted(specs))
    interface_words = set(re.findall(r"\b[A-Za-z_]\w*\b", rendered))
    return rendered, {f"IID_{name}" for name in specs}, interface_words


def render_cpp_support(
    source_path: Path,
    exclude_guids: set[str] | None = None,
    additional_source: str = "",
    exclude_types: set[str] | None = None,
) -> str:
    """Render the deterministic non-class support closure selected by one C++ file."""
    source = source_path.read_text(encoding="utf-8", errors="replace")
    chunks = ["// Deterministic C++ support closure selected from source identifiers."]

    # Local numeric macros and shell32 resource IDs are compiler-facing facts.
    definitions = dict(
        re.findall(
            r"^\s*#define\s+([A-Za-z_]\w*)\s+\(?\s*"
            r"([0-9]+|0[xX][0-9A-Fa-f]+)\s*\)?\s*(?://[^\n]*)?$",
            source,
            re.M,
        )
    )
    resource_header = next(
        (
            parent / "shresdef.h"
            for parent in source_path.parents
            if (parent / "shresdef.h").is_file()
        ),
        None,
    )
    if resource_header is not None:
        resource_text = resource_header.read_text(encoding="utf-8", errors="replace")
        definitions.update(
            re.findall(
                r"^\s*#define\s+([A-Za-z_]\w*)\s+\(?\s*"
                r"([0-9]+|0[xX][0-9A-Fa-f]+)\s*\)?\s*$",
                resource_text,
                re.M,
            )
        )
    selection_source = source + "\n" + additional_source
    used_words = set(re.findall(r"\b[A-Za-z_]\w*\b", _masked_cpp(selection_source)))
    if re.search(r"\bshell32_hInstance\b", selection_source):
        used_words.add("shell32_hInstance")
    used_words.update(
        f"IID_{interface}"
        for interface in re.findall(r"\bIID_PPV_ARG\s*\(\s*([A-Za-z_]\w*)", source)
    )
    if "ShellObjectCreatorInit<CPrintersEnum>" in source:
        used_words.add("CPrintersEnum_CreateInstance")
    declared_types = set(
        re.findall(
            r"\bpub\s+(?:struct|union|enum|type)\s+(?:r#)?([A-Za-z_]\w*)",
            additional_source,
        )
    )
    repo_root = next(
        (parent for parent in source_path.parents if (parent / "sdk/include").is_dir()), None
    )
    if repo_root is not None:
        declared_types.update(_shared_substrate_type_names(repo_root))
    shared_values = _shared_substrate_value_names(repo_root) if repo_root is not None else set()
    # File-scope PODs are source authority, not model work. This is shared by C
    # and C++ inputs; only simple, fully parsed layouts are emitted.
    source_pod_names = set(re.findall(r"\bstruct\s+([A-Za-z_]\w*)\s*\{", source))
    referenced_pods = {
        name
        for name in source_pod_names
        if len(re.findall(rf"\b{re.escape(name)}\b", source)) > 1
        or re.search(rf"\b{re.escape(name)}\b", additional_source)
    }
    for name in sorted(referenced_pods - (exclude_types or set())):
        definition = _pod_definition(source, name)
        if definition is None:
            continue
        body, packed = definition
        declarations = [" ".join(item.split()) for item in body.split(";") if item.strip()]
        constants = _numeric_array_constants(source_path, declarations)
        fields = []
        try:
            for declaration in declarations:
                fields.append(
                    _parse_plain_state_field(
                        declaration,
                        allowed_pod_types=source_pod_names | _POD_LEAF_TYPES,
                        array_constants=constants,
                    )
                )
        except CppComError:
            continue
        representation = "#[repr(C, packed)]" if packed else "#[repr(C)]"
        lines = [representation, f"pub struct {name} {{"]
        for state_field in fields:
            field_name = (
                f"r#{state_field.name}" if state_field.name in _RUST_KEYWORDS else state_field.name
            )
            lines.append(f"    pub {field_name}: {state_field.rust_type},")
        lines.append("}")
        chunks.append("\n".join(lines))
    for name in sorted((used_words & definitions.keys()) - shared_values):
        chunks.append(f"pub const {name}: UINT = {definitions[name]};")
    for name in sorted((used_words & _CPP_SUPPORT_CONSTANTS.keys()) - shared_values):
        rust_type, value = _CPP_SUPPORT_CONSTANTS[name]
        chunks.append(f"pub const {name}: {rust_type} = {value};")
    owned_constants = definitions.keys() | _CPP_SUPPORT_CONSTANTS.keys() | shared_values
    for name, value in sorted(_sdk_numeric_definitions(source_path, used_words).items()):
        if name not in owned_constants:
            chunks.append(f"pub const {name}: DWORD = {value};")
    sdk_pods = _sdk_pod_support(source_path, used_words)
    if sdk_pods:
        chunks.append(sdk_pods)
        # SDK POD fields can introduce interface/type dependencies that did not
        # occur in the selected workers themselves (SFV_CREATE's callback is a
        # representative shell32 case).  Feed those deterministic dependencies
        # back into interface discovery rather than leaving a dangling type.
        used_words.update(re.findall(r"\b[A-Za-z_]\w*\b", sdk_pods))

    # Header-owned constant UTF-16 arrays are source facts just like literals
    # inside method bodies. Emit only referenced, fully literal definitions.
    for match in re.finditer(
        r"\b(?:static\s+)?const\s+WCHAR\s+([A-Za-z_]\w*)\s*\[\s*\]\s*=\s*"
        r"(L\"(?:\\.|[^\"\\])*\")\s*;",
        selection_source,
    ):
        name, token = match.groups()
        if name not in used_words:
            continue
        units = _wide_literal_units(token)
        values = ", ".join(f"0x{unit:04X}" for unit in units)
        chunks.append(
            f"pub static {name}_WIDE: [WCHAR; {len(units)}] = [{values}];\n"
            f"pub const {name}: LPCWSTR = {name}_WIDE.as_ptr();"
        )

    interface_support, interface_iids, interface_words = _sdk_interface_support(
        source_path, used_words, declared_types
    )
    if interface_support:
        chunks.append(interface_support)
        used_words.update(interface_words)
        interface_pods = _sdk_pod_support(source_path, interface_words)
        if interface_pods:
            chunks.append(interface_pods)
        for name, value in sorted(_sdk_numeric_definitions(source_path, interface_words).items()):
            if name not in owned_constants:
                chunks.append(f"pub const {name}: DWORD = {value};")

    if "INVALID_FILETITLE_CHARACTERSW" in used_words:
        chunks.append(
            "pub static INVALID_FILETITLE_CHARACTERSW: [WCHAR; 10] = "
            "[0x005c, 0x002f, 0x003a, 0x002a, 0x003f, 0x0022, 0x003c, 0x003e, 0x007c, 0];"
        )

    if "SFV_CREATE" in used_words and not re.search(r"\bstruct\s+SFV_CREATE\b", "\n".join(chunks)):
        chunks.append(
            "#[repr(C)]\n"
            "pub struct SFV_CREATE {\n"
            "    pub cbSize: UINT,\n"
            "    pub pshf: *mut IShellFolder,\n"
            "    pub psvOuter: *mut core::ffi::c_void,\n"
            "    pub psfvcb: *mut core::ffi::c_void,\n"
            "}"
        )

    if "NOTIFYICONDATAW" in used_words:
        chunks.append(
            "pub type WPARAM = UINT_PTR;\n"
            "pub type LRESULT = LONG_PTR;\n"
            "#[repr(C)]\n"
            "pub struct MSG {\n"
            "    pub hwnd: HWND, pub message: UINT, pub wParam: WPARAM,\n"
            "    pub lParam: LPARAM, pub time: DWORD, pub pt: POINT,\n"
            "}\n"
            'pub type WNDPROC = Option<unsafe extern "system" fn(\n'
            "    HWND, UINT, WPARAM, LPARAM,\n"
            ") -> LRESULT>;\n"
            "#[repr(C)]\n"
            "pub struct NOTIFYICONDATAW {\n"
            "    pub cbSize: DWORD, pub hWnd: HWND, pub uID: UINT, pub uFlags: UINT,\n"
            "    pub uCallbackMessage: UINT, pub hIcon: HICON, pub szTip: [WCHAR; 128],\n"
            "    pub dwState: DWORD, pub dwStateMask: DWORD, pub szInfo: [WCHAR; 256],\n"
            "    pub uTimeout: UINT, pub szInfoTitle: [WCHAR; 64], pub dwInfoFlags: DWORD,\n"
            "    pub guidItem: GUID, pub hBalloonIcon: HICON,\n"
            "}\n"
            "pub type PNOTIFYICONDATAW = *mut NOTIFYICONDATAW;\n"
            "pub const NOTIFYICONDATAW_V3_SIZE: DWORD =\n"
            "    core::mem::offset_of!(NOTIFYICONDATAW, hBalloonIcon) as DWORD;"
        )

    if "ASSERT" in used_words:
        chunks.append("macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }")

    if "CUserNotification::WorkerWndProc" in source:
        chunks.append(
            'unsafe extern "system" fn CUserNotification_WorkerWndProc(\n'
            "    window: HWND, message: UINT, wparam: WPARAM, lparam: LPARAM,\n"
            ") -> LRESULT {\n"
            "    let this = GetWindowLongPtrW(window, 0) as *mut CUserNotification;\n"
            "    CUserNotification_WorkerWndProc_body(\n"
            "        this, window, message, wparam, lparam,\n"
            "    )\n"
            "}"
        )

    if "DWORDLONG" in used_words:
        chunks.append("pub type DWORDLONG = u64;")
    if "SVSIF" in used_words:
        chunks.append("pub type SVSIF = UINT;")
    if "LPTBBUTTONSB" in used_words:
        chunks.append("pub type LPTBBUTTONSB = *mut core::ffi::c_void;")
    if "LOWORD" in used_words:
        chunks.append(
            "macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }"
        )
    if "MAKEINTRESOURCE" in used_words:
        chunks.append(
            "#[inline]\nconst fn MAKEINTRESOURCE(value: UINT) -> LPCWSTR {\n"
            "    value as usize as LPCWSTR\n}"
        )
    if "EXPPS" in used_words:
        chunks.append("pub type EXPPS = UINT;")
    if "LPFNSVADDPROPSHEETPAGE" in used_words:
        chunks.append(
            'pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn('
            "HPROPSHEETPAGE, LPARAM) -> BOOL>;"
        )
    if "SHQUERYRBINFO" in used_words:
        chunks.append(
            "#[repr(C)]\n"
            "pub struct SHQUERYRBINFO {\n"
            "    pub cbSize: DWORD,\n"
            "    pub i64Size: i64,\n"
            "    pub i64NumItems: i64,\n"
            "}"
        )
    if "SHELLEXECUTEINFOW" in used_words:
        chunks.append(
            "#[repr(C)]\n"
            "pub struct SHELLEXECUTEINFOW {\n"
            "    pub cbSize: DWORD, pub fMask: ULONG, pub hwnd: HWND,\n"
            "    pub lpVerb: LPCWSTR, pub lpFile: LPCWSTR, pub lpParameters: LPCWSTR,\n"
            "    pub lpDirectory: LPCWSTR, pub nShow: INT, pub hInstApp: HINSTANCE,\n"
            "    pub lpIDList: LPVOID, pub lpClass: LPCWSTR, pub hkeyClass: HKEY,\n"
            "    pub dwHotKey: DWORD, pub icon_or_monitor: HANDLE, pub hProcess: HANDLE,\n"
            "}"
        )

    table = _render_cmverbmap(source)
    if table:
        chunks.append(table)
    column_tables = _render_shvheader_tables(source, used_words)
    if column_tables:
        chunks.append(column_tables)

    if "CComHeapPtr" in used_words:
        used_words.add("CoTaskMemFree")
    if "SENDTO_ITEM" in used_words:
        used_words.update(("CoTaskMemFree", "DestroyIcon"))
    if "CStringW" in used_words:
        used_words.update(("lstrlenW", "memcpy", "StringCbPrintfW"))
    if "StringCchCopyW" in used_words:
        used_words.update(("lstrlenW", "memcpy"))
    if "SHSetStrRet" in used_words:
        used_words.update(
            (
                "CoTaskMemAlloc",
                "LoadStringW",
                "lstrlenW",
                "memcpy",
                "MAX_PATH",
                "shell32_hInstance",
            )
        )
    if "CopyMemory" in used_words or "ZeroMemory" in used_words:
        used_words.add("memcpy")
    if "ITEMIDLIST_ABSOLUTE" in used_words:
        chunks.append("pub type ITEMIDLIST_ABSOLUTE = ITEMIDLIST;")
    if "ITEMID_CHILD" in used_words:
        chunks.append("pub type ITEMID_CHILD = ITEMIDLIST;")
    drop_effects = {
        "DROPEFFECT_NONE": 0,
        "DROPEFFECT_COPY": 1,
        "DROPEFFECT_MOVE": 2,
        "DROPEFFECT_LINK": 4,
    }
    selected_effects = [
        f"pub const {name}: DWORD = {value};"
        for name, value in drop_effects.items()
        if name in used_words
    ]
    if selected_effects:
        chunks.append("\n".join(selected_effects))
    selected = [decl for name, decl in _CPP_SUPPORT_EXTERNS.items() if name in used_words]
    if selected:
        chunks.append('extern "system" {\n' + "\n".join(selected) + "\n}")
    selected_c = [decl for name, decl in _CPP_SUPPORT_C_EXTERNS.items() if name in used_words]
    if selected_c:
        chunks.append('extern "C" {\n' + "\n".join(selected_c) + "\n}")
    if "StringCbPrintfW" in used_words:
        chunks.append(
            "unsafe fn cpp_com_string_cb_printf_w(\n"
            "    dest: LPWSTR, bytes: usize, format: LPCWSTR, args: &[LPCWSTR],\n"
            ") -> HRESULT {\n"
            "    let capacity = bytes / core::mem::size_of::<WCHAR>();\n"
            "    if dest.is_null() || format.is_null() || capacity == 0 { return E_FAIL; }\n"
            "    let (mut out, mut input, mut arg_index) = (0usize, 0usize, 0usize);\n"
            "    while *format.add(input) != 0 {\n"
            "        let unit = *format.add(input);\n"
            "        if unit == b'%' as WCHAR && *format.add(input + 1) == b's' as WCHAR {\n"
            "            if arg_index >= args.len() { return E_FAIL; }\n"
            "            let arg = args[arg_index]; arg_index += 1; input += 2;\n"
            "            if arg.is_null() { continue; }\n"
            "            let mut index = 0usize;\n"
            "            while *arg.add(index) != 0 {\n"
            "                if out + 1 >= capacity { *dest.add(capacity - 1) = 0; "
            "return 0x8007007Au32 as HRESULT; }\n"
            "                *dest.add(out) = *arg.add(index); out += 1; index += 1;\n"
            "            }\n"
            "            continue;\n"
            "        }\n"
            "        if out + 1 >= capacity { *dest.add(capacity - 1) = 0; "
            "return 0x8007007Au32 as HRESULT; }\n"
            "        *dest.add(out) = unit; out += 1; input += 1;\n"
            "    }\n"
            "    *dest.add(out) = 0; S_OK\n"
            "}\n"
            "macro_rules! StringCbPrintfW {\n"
            "    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr $(,)?) => {\n"
            "        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1])\n"
            "    };\n"
            "    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr, $arg2:expr $(,)?) => {\n"
            "        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1, $arg2])\n"
            "    };\n"
            "}"
        )
    if "memcpy" in used_words:
        chunks.append(
            "#[inline]\nunsafe fn memcpy<D, S, N: CppComSize>("
            "dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {\n"
            "    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())\n}"
        )
    if "SHFree" in used_words:
        chunks.append(
            'extern "system" {\n    #[link_name = "SHFree"]\n'
            "    fn cpp_com_SHFree(memory: LPVOID);\n}\n"
            "#[inline]\nunsafe fn SHFree<T>(memory: *mut T) { cpp_com_SHFree(memory.cast()); }"
        )
    if "free" in used_words:
        chunks.append(
            'extern "C" {\n    #[link_name = "free"]\n'
            "    fn cpp_com_free(memory: *mut core::ffi::c_void);\n}\n"
            "#[inline]\nunsafe fn free<T>(memory: *mut T) { cpp_com_free(memory.cast()); }"
        )
    if "CComHeapPtr" in used_words:
        chunks.append(_CPP_COM_HEAP_PTR_SUPPORT)
    if "SENDTO_ITEM" in used_words:
        chunks.append(_CPP_SENDTO_ITEM_SUPPORT)
    guid_words = {
        word
        for word in used_words
        if word != "IID_PPV_ARG" and re.fullmatch(r"(?:IID|SID|CLSID)_[A-Za-z_]\w*", word)
    }
    for alias, target in _CPP_SUPPORT_GUID_ALIASES.items():
        if alias in used_words:
            used_words.add(target)
            guid_words.discard(alias)
            guid_words.add(target)
    selected_guids = sorted(
        ((used_words & _CPP_SUPPORT_GUIDS) | guid_words) - (exclude_guids or set()) - interface_iids
    )
    if selected_guids:
        chunks.append(
            'extern "C" {\n'
            + "\n".join(f"    static {name}: GUID;" for name in selected_guids)
            + "\n}"
        )

    if "CopyMemory" in used_words or "ZeroMemory" in used_words:
        chunks.append(
            'extern "C" {\n'
            "    fn memset(dst: *mut core::ffi::c_void, value: INT,\n"
            "        bytes: usize) -> *mut core::ffi::c_void;\n"
            "}\n"
            "#[inline]\nunsafe fn CopyMemory<D, S, N: CppComSize>(\n"
            "    dst: *mut D, src: *const S, bytes: N) {\n"
            "    let _ = memcpy(dst, src, bytes);\n}\n"
            "#[inline]\nunsafe fn ZeroMemory<T, N: CppComSize>(dst: *mut T, bytes: N) {\n"
            "    let _ = memset(dst.cast(), 0, bytes.cpp_com_size());\n}"
        )

    if "StringCchCopyW" in used_words:
        chunks.append(
            "#[inline]\n"
            "unsafe fn StringCchCopyW(dst: LPWSTR, capacity: usize, src: LPCWSTR) -> HRESULT {\n"
            "    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }\n"
            "    let len = lstrlenW(src) as usize;\n"
            "    if len >= capacity { *dst = 0; return E_FAIL; }\n"
            "    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();\n"
            "    let _ = memcpy(dst, src, bytes);\n"
            "    S_OK\n"
            "}"
        )
    if "SHSetStrRet" in used_words:
        chunks.append(
            "unsafe fn cpp_com_SHSetStrRetW(out: *mut STRRET, value: LPCWSTR) -> HRESULT {\n"
            "    let len = lstrlenW(value) as usize;\n"
            "    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();\n"
            "    let buffer = CoTaskMemAlloc(bytes) as LPWSTR;\n"
            "    if buffer.is_null() { return E_OUTOFMEMORY; }\n"
            "    let _ = memcpy(buffer, value, bytes);\n"
            "    (*out).uType = STRRET_WSTR; (*out).u.pOleStr = buffer; S_OK\n"
            "}\n"
            "unsafe fn cpp_com_SHSetStrRetResource(out: *mut STRRET, id: DWORD) -> HRESULT {\n"
            "    let mut buffer = [0u16; MAX_PATH];\n"
            "    if LoadStringW(shell32_hInstance, id, buffer.as_mut_ptr(), MAX_PATH as INT) == 0 "
            "{ return E_FAIL; }\n"
            "    cpp_com_SHSetStrRetW(out, buffer.as_ptr())\n"
            "}"
        )
    if "CStringW" in used_words:
        chunks.append(_CPP_CSTRINGW_SUPPORT)
    if "CComPtr" in used_words:
        chunks.append(_CPP_COMPTR_GUARD_SUPPORT)
    if "shell32_hInstance" in used_words:
        chunks.append('extern "C" {\n    static mut shell32_hInstance: HINSTANCE;\n}')
    return "\n\n".join(chunks)


_CPP_COMPTR_GUARD_SUPPORT = r"""pub struct CppComPtrGuard<T> { slot: *mut *mut T }
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self { Self { slot } }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}"""


_CPP_COM_HEAP_PTR_SUPPORT = r"""#[repr(transparent)]
pub struct CComHeapPtr<T> { ptr: *mut T }
impl<T> CComHeapPtr<T> {
    pub const fn empty() -> Self { Self { ptr: core::ptr::null_mut() } }
    pub const fn new(ptr: *mut T) -> Self { Self { ptr } }
    pub const fn as_raw(&self) -> *mut T { self.ptr }
    pub fn as_mut_slot(&mut self) -> *mut *mut T { core::ptr::addr_of_mut!(self.ptr) }
    pub unsafe fn Attach(&mut self, ptr: *mut T) {
        if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); }
        self.ptr = ptr;
    }
    pub fn Detach(&mut self) -> *mut T {
        let ptr = self.ptr; self.ptr = core::ptr::null_mut(); ptr
    }
}
impl<T> Drop for CComHeapPtr<T> {
    fn drop(&mut self) {
        unsafe { if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); } }
    }
}"""


_CPP_SENDTO_ITEM_SUPPORT = r"""#[repr(C)]
pub struct SENDTO_ITEM {
    pub pNext: *mut SENDTO_ITEM,
    pub pidlChild: PITEMID_CHILD,
    pub pszText: LPWSTR,
    pub hIcon: HICON,
}
unsafe fn SENDTO_ITEM_new(
    child: PITEMID_CHILD, text: LPWSTR, icon: HICON,
) -> *mut SENDTO_ITEM {
    let item = HeapAlloc(
        GetProcessHeap(), 0, core::mem::size_of::<SENDTO_ITEM>(),
    ) as *mut SENDTO_ITEM;
    if !item.is_null() {
        item.write(SENDTO_ITEM {
            pNext: core::ptr::null_mut(), pidlChild: child, pszText: text, hIcon: icon,
        });
    }
    item
}
unsafe fn SENDTO_ITEM_delete(item: *mut SENDTO_ITEM) {
    if item.is_null() { return; }
    CoTaskMemFree((*item).pidlChild.cast());
    CoTaskMemFree((*item).pszText.cast());
    if !(*item).hIcon.is_null() { DestroyIcon((*item).hIcon); }
    HeapFree(GetProcessHeap(), 0, item.cast::<core::ffi::c_void>());
}"""


_CPP_CSTRINGW_SUPPORT = r"""extern "system" {
    fn HeapReAlloc(heap: HANDLE, flags: DWORD, memory: LPVOID, bytes: usize) -> LPVOID;
    fn GetModuleHandleW(name: LPCWSTR) -> HINSTANCE;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
}

pub struct CStringW { buffer: LPWSTR, len: usize, capacity: usize }
impl CStringW {
    pub const fn new() -> Self { Self { buffer: core::ptr::null_mut(), len: 0, capacity: 0 } }
    pub unsafe fn from_resource(id: UINT) -> Self {
        let mut value = Self::new();
        let mut stack = [0u16; 256];
        let count = LoadStringW(
            GetModuleHandleW(core::ptr::null()), id, stack.as_mut_ptr(), stack.len() as INT,
        );
        if count > 0 { value.append_ptr_len(stack.as_ptr(), count as usize); }
        value
    }
    pub unsafe fn new_from_lpcwstr(text: LPCWSTR) -> Self {
        if (text as usize) <= 0xffff { return Self::from_resource(text as usize as UINT); }
        let mut value = Self::new(); value.append_ptr(text); value
    }
    pub fn as_ptr(&self) -> LPCWSTR {
        if self.buffer.is_null() { CPP_COM_EMPTY_WSTR.as_ptr() } else { self.buffer }
    }
    pub fn as_mut_ptr(&mut self) -> LPWSTR { self.as_ptr() as LPWSTR }
    unsafe fn reserve(&mut self, needed: usize) -> bool {
        if needed <= self.capacity { return true; }
        let capacity = needed.next_power_of_two().max(16);
        let bytes = capacity * core::mem::size_of::<WCHAR>();
        let next = if self.buffer.is_null() {
            HeapAlloc(GetProcessHeap(), 0, bytes)
        } else {
            HeapReAlloc(GetProcessHeap(), 0, self.buffer.cast(), bytes)
        } as LPWSTR;
        if next.is_null() { return false; }
        self.buffer = next; self.capacity = capacity; true
    }
    unsafe fn append_ptr_len(&mut self, text: LPCWSTR, count: usize) {
        if text.is_null() || !self.reserve(self.len + count + 1) { return; }
        let _ = memcpy(
            self.buffer.add(self.len), text,
            count * core::mem::size_of::<WCHAR>(),
        );
        self.len += count; *self.buffer.add(self.len) = 0;
    }
    unsafe fn append_ptr(&mut self, text: LPCWSTR) {
        if !text.is_null() { self.append_ptr_len(text, lstrlenW(text) as usize); }
    }
    unsafe fn append_char(&mut self, ch: WCHAR) { self.append_ptr_len(core::ptr::addr_of!(ch), 1); }
    pub unsafe fn concat(&mut self, text: LPCWSTR) { self.append_ptr(text); }
    pub unsafe fn assign(&mut self, text: LPCWSTR) {
        self.len = 0;
        if !self.buffer.is_null() { *self.buffer = 0; }
        self.append_ptr(text);
    }
    pub unsafe fn format(&mut self, format: LPCWSTR, arg: LPCWSTR) {
        let mut resource = [0u16; 256];
        let format = if (format as usize) <= 0xffff {
            LoadStringW(GetModuleHandleW(core::ptr::null()), format as usize as UINT,
                resource.as_mut_ptr(), resource.len() as INT);
            resource.as_ptr()
        } else { format };
        self.len = 0;
        if self.reserve(512) {
            let _ = StringCbPrintfW!(self.buffer, self.capacity * 2, format, arg);
            self.len = lstrlenW(self.buffer) as usize;
        }
    }
    pub unsafe fn format_message(&mut self, arg: LPCWSTR) {
        let mut format = [0u16; 512];
        let count = lstrlenW(self.as_ptr()).max(0) as usize;
        let count = count.min(format.len() - 1);
        let _ = memcpy(format.as_mut_ptr(), self.as_ptr(), count * 2);
        self.format(format.as_ptr(), arg);
    }
}
impl core::ops::AddAssign<LPCWSTR> for CStringW {
    fn add_assign(&mut self, rhs: LPCWSTR) { unsafe { self.append_ptr(rhs) } }
}
impl core::ops::AddAssign<WCHAR> for CStringW {
    fn add_assign(&mut self, rhs: WCHAR) { unsafe { self.append_char(rhs) } }
}
impl Drop for CStringW {
    fn drop(&mut self) {
        unsafe {
            if !self.buffer.is_null() { HeapFree(GetProcessHeap(), 0, self.buffer); }
        }
    }
}
static CPP_COM_EMPTY_WSTR: [WCHAR; 1] = [0];"""


def worker_signature(
    name: str, class_name: str | None, params: list[tuple[str, str]], return_type: str
) -> str:
    """Render the deterministic body-worker signature.

    COM methods receive the recovered object pointer first. Same-file helper
    functions are ordinary Rust workers with signatures derived directly from
    their C definitions; the model owns neither surface.
    """
    rendered = [f"this: *mut {class_name}"] if class_name else []
    for i, (c_type, param_name) in enumerate(params):
        # The generic scanner intentionally preserves declarator punctuation,
        # but for C++ references clang-style source often attaches `&`/`*` to
        # the name (`CStringW`, `&paths`). Move it back to the type before the
        # deterministic Rust mapping.
        declarator = param_name or f"arg{i}"
        while declarator.startswith(("&", "*")):
            c_type = f"{c_type} {declarator[0]}"
            declarator = declarator[1:]
        rendered.append(f"{declarator}: {c_type_to_rust(c_type)}")
    ret = "()" if not return_type or return_type.strip() == "void" else c_type_to_rust(return_type)
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


def strip_debug_statements(text: str, names: set[str] | None = None) -> str:
    """Remove complete shell32 debug-macro statements from a model-only view."""
    masked = _masked_cpp(text)
    spans = []
    debug_names = "|".join(re.escape(name) for name in sorted(names or _CPP_COM_DEBUG_MACROS))
    for match in re.finditer(rf"\b(?:{debug_names})\s*!?\s*\(", masked):
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


def _expand_default_helper_arguments(
    deps: list[FunctionDep], by_source: dict[str, list[FunctionDep]]
) -> list[FunctionDep]:
    """Materialize trailing C++ helper defaults before overload binding/prompting."""
    defaults: dict[str, tuple[int, tuple[str, ...]]] = {}
    for source_name, candidates in by_source.items():
        if len(candidates) != 1:
            continue
        dep = candidates[0]
        masked = _masked_cpp(dep.body_text)
        name_match = re.search(rf"\b{re.escape(source_name)}\s*\(", masked)
        if name_match is None:
            continue
        opening = masked.find("(", name_match.start())
        closing = _matching_paren(masked, opening)
        pieces = _split_args(dep.body_text[opening + 1 : closing])
        trailing: list[str] = []
        for piece in reversed(pieces):
            if "=" not in piece:
                break
            trailing.append(piece.split("=", 1)[1].strip())
        if trailing:
            trailing.reverse()
            defaults[source_name] = (len(pieces) - len(trailing), tuple(trailing))

    rewritten = []
    for dep in deps:
        body = dep.body_text
        definition_brace = body.find("{")
        for start, end, source_name, arity in reversed(_named_call_spans(body, set(defaults))):
            candidates = by_source[source_name]
            if dep in candidates and start < definition_brace:
                continue
            required, default_values = defaults[source_name]
            full_arity = required + len(default_values)
            if not required <= arity < full_arity:
                continue
            opening = body.find("(", end)
            closing = _matching_paren(_masked_cpp(body), opening)
            missing = default_values[arity - required :]
            insertion = (", " if arity else "") + ", ".join(missing)
            body = body[:closing] + insertion + body[closing:]
        rewritten.append(replace(dep, body_text=body))
    return rewritten


def _rewrite_com_ptr_state(
    body: str, class_name: str, state_fields: list[PlainStateField]
) -> tuple[str, set[str]]:
    """Make CComPtr assignment/refcount behavior explicit before model translation."""
    callees = set()
    for state in state_fields:
        helper = state.assignment_helper
        if helper is None:
            continue
        qualified_helper = f"{class_name}_{helper}"
        # CComPtr::operator=(T*) is the only accepted mutation shape. Compound,
        # address-taking, Attach/Detach, and member calls remain untouched and
        # therefore fail later audits/compile rather than losing ownership.
        assignment = re.compile(
            rf"(?<![\w.>])(?:this\s*->\s*)?{re.escape(state.name)}\s*=\s*([^;]+);"
        )
        body, count = assignment.subn(rf"{qualified_helper}(this, \1);", body)
        if count:
            callees.add(qualified_helper)
        body = re.sub(rf"(?<![\w.>]){re.escape(state.name)}\b", f"this->{state.name}", body)
    return body, callees


def _rewrite_owned_string_state(
    body: str, class_name: str, state_fields: list[PlainStateField]
) -> tuple[str, set[str]]:
    """Make CStringW replacement assignment explicit before model translation."""
    callees = set()
    for state in state_fields:
        helper = state.string_assignment_helper
        if helper is None:
            continue
        qualified_helper = f"{class_name}_{helper}"
        assignment = re.compile(
            rf"(?<![\w.>])(?:this\s*->\s*)?{re.escape(state.name)}\s*=\s*([^;]+);"
        )
        body, count = assignment.subn(rf"{qualified_helper}(this, \1);", body)
        if count:
            callees.add(qualified_helper)
        body = re.sub(rf"(?<![\w.>]){re.escape(state.name)}\b", f"this->{state.name}", body)
    return body, callees


def prepare_helper_closure(
    deps: list[FunctionDep],
    class_name: str,
    state_fields: list[PlainStateField] | None = None,
) -> list[FunctionDep]:
    """Select and bind the exact same-file helper closure for one COM class.

    C++ overload resolution is deliberately not reimplemented. Calls are bound
    from scanner-owned parameter/local types and must have one uniquely best
    candidate. Every selected call is rewritten to its collision-free
    ``<target>_body`` Rust name before the model sees it. Ambiguity hard-fails
    before payment.
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

    expanded = _expand_default_helper_arguments(class_deps + free_deps, by_source)
    class_deps = [dep for dep in expanded if dep.cpp_class == class_name]
    free_deps = [dep for dep in expanded if dep.cpp_class is None]
    by_source = {}
    for dep in free_deps:
        match = re.search(r"\b([A-Za-z_]\w*)\s*\(", _masked_cpp(dep.body_text))
        source_name = match.group(1) if match else dep.name
        by_source.setdefault(source_name, []).append(dep)

    helper_returns: dict[str, str] = {}
    for source_name, candidates in by_source.items():
        return_types = {
            _abi_c_type_key(candidate.return_type)
            for candidate in candidates
            if candidate.return_type
        }
        if len(return_types) == 1:
            helper_returns[source_name] = candidates[0].return_type
        for candidate in candidates:
            if candidate.return_type:
                helper_returns[f"{candidate.name}_body"] = candidate.return_type

    binding: dict[tuple[str, int], list[FunctionDep]] = {}
    for source_name, candidates in by_source.items():
        for dep in candidates:
            key = (source_name, len(dep.params))
            binding.setdefault(key, []).append(dep)

    helper_names = set(by_source)
    method_targets: dict[str, list[FunctionDep]] = {}
    for dep in class_deps:
        method = dep.cpp_method
        if method in (None, class_name, f"~{class_name}"):
            continue
        method_targets.setdefault(method, []).append(dep)
    method_names = set(method_targets)
    selected: dict[str, FunctionDep] = {}
    queue = list(class_deps)
    while queue:
        caller = queue.pop(0)
        definition_brace = caller.body_text.find("{")
        for start, end, source_name, arity in _named_call_spans(caller.body_text, helper_names):
            candidates = binding.get((source_name, arity), [])
            if not candidates:
                available = sorted({n for name, n in binding if name == source_name})
                raise CppComError(
                    f"{class_name}: cannot bind local helper call {source_name}/{arity}; "
                    f"available arities: {available}"
                )
            target = (
                caller
                if caller in candidates and start < definition_brace
                else _bind_sibling_overload(
                    caller.body_text,
                    start,
                    end,
                    candidates,
                    caller,
                    state_fields or [],
                    class_name,
                    source_name,
                    helper_returns,
                )
            )
            if target.name not in selected:
                selected[target.name] = target
                queue.append(target)

    plan = list(selected.values()) + class_deps
    rewritten: list[FunctionDep] = []
    for dep in plan:
        body = _rewrite_wide_literals(dep.body_text)
        body, state_callees = _rewrite_com_ptr_state(body, class_name, state_fields or [])
        body, string_state_callees = _rewrite_owned_string_state(
            body, class_name, state_fields or []
        )
        state_callees |= string_state_callees
        local_targets: set[str] = set()
        spans = _named_call_spans(body, helper_names)
        definition_brace = body.find("{")
        for start, end, source_name, arity in reversed(spans):
            candidates = binding.get((source_name, arity), [])
            if not candidates:
                continue
            target = (
                dep
                if dep in candidates and start < definition_brace
                else _bind_sibling_overload(
                    body,
                    start,
                    end,
                    candidates,
                    dep,
                    state_fields or [],
                    class_name,
                    source_name,
                    helper_returns,
                )
            )
            body = body[:start] + f"{target.name}_body" + body[end:]
            # The helper definition's own declarator is not a callee. Removing
            # self is harmless for genuine recursion; _format_callee_context
            # intentionally omits it too.
            if target.name != dep.name:
                local_targets.add(f"{target.name}_body")
        method_spans = _named_call_spans(body, method_names)
        definition_brace = body.find("{")
        for start, end, source_name, _arity in reversed(method_spans):
            receiver = re.search(r"([A-Za-z_]\w*)\s*(?:->|\.)\s*$", body[:start])
            if receiver is not None and receiver.group(1) != "this":
                # Calls through another COM object are not implicit sibling
                # calls merely because their method spelling collides.
                continue
            # Keep this worker's C++ definition declarator recognizable; only
            # sibling calls become deterministic Rust worker calls.
            if source_name == dep.cpp_method and start < definition_brace:
                continue
            all_candidates = method_targets[source_name]
            candidates = [
                candidate for candidate in all_candidates if len(candidate.params) == _arity
            ]
            # IID_PPV_ARG and similar source macros expand one apparent C++
            # argument into multiple ABI arguments later in normalization.
            # A non-overloaded method remains unambiguous despite the lexical
            # arity mismatch.
            if not candidates and len(all_candidates) == 1:
                candidates = all_candidates
            target = _bind_sibling_overload(
                body,
                start,
                end,
                candidates,
                dep,
                state_fields or [],
                class_name,
                source_name,
                helper_returns,
            )
            body = body[:start] + f"{target.name}_body" + body[end:]
            if target.name != dep.name:
                local_targets.add(f"{target.name}_body")
        external = {
            name for name in dep.callees if name not in helper_names and name not in method_names
        }
        rewritten.append(
            replace(dep, body_text=body, callees=external | local_targets | state_callees)
        )

    return topological_sort(rewritten)


def _bind_sibling_overload(
    body: str,
    start: int,
    end: int,
    candidates: list[FunctionDep],
    caller: FunctionDep,
    state_fields: list[PlainStateField],
    class_name: str,
    source_name: str,
    call_returns: dict[str, str] | None = None,
) -> FunctionDep:
    """Bind an implicit ``this`` call using source-visible argument types."""
    if len(candidates) == 1:
        return candidates[0]
    opening = body.find("(", end)
    closing = _matching_paren(_masked_cpp(body), opening)
    arguments = _split_args(body[opening + 1 : closing])
    symbols = {name.lstrip("&*"): c_type for c_type, name in caller.params if name}
    symbols.update({field.name: field.c_type for field in state_fields})
    prefix = body[:start]
    for declaration in re.finditer(
        r"\b(?P<type>[A-Za-z_]\w*(?:\s*\*)?)\s+(?P<name>[A-Za-z_]\w*)\s*"
        r"(?P<array>\[[^]]*\])?\s*(?:=|;)",
        prefix,
    ):
        c_type = " ".join(declaration.group("type").split())
        if declaration.group("array"):
            if c_type == "WCHAR":
                c_type = "LPWSTR"
            elif c_type == "CHAR":
                c_type = "LPSTR"
        symbols.setdefault(declaration.group("name"), c_type)
    for com_ptr in re.finditer(
        r"\bCComPtr\s*<\s*(?P<interface>I[A-Za-z0-9_]*)\s*>\s+"
        r"(?P<name>[A-Za-z_]\w*)\b",
        prefix,
    ):
        symbols[com_ptr.group("name")] = f"{com_ptr.group('interface')} *"
    for argument in arguments:
        identifier = re.fullmatch(r"(?:this\s*->\s*)?([A-Za-z_]\w*)", argument.strip())
        if identifier is None or identifier.group(1) in symbols:
            continue
        name = identifier.group(1)
        com_ptr = re.search(
            rf"\bCComPtr\s*<\s*(?P<interface>I[A-Za-z0-9_]*)\s*>\s+"
            rf"{re.escape(name)}\b",
            prefix,
        )
        if com_ptr is not None:
            symbols[name] = f"{com_ptr.group('interface')} *"
            continue
        declaration = re.search(
            rf"\b(?P<type>[A-Za-z_]\w*(?:\s*\*)?)\s+{re.escape(name)}\s*"
            rf"(?P<array>\[[^]]*\])?\s*(?:=|;)",
            prefix,
        )
        if declaration is not None:
            c_type = " ".join(declaration.group("type").split())
            if declaration.group("array"):
                if c_type == "WCHAR":
                    c_type = "LPWSTR"
                elif c_type == "CHAR":
                    c_type = "LPSTR"
            symbols[name] = c_type

    scores: list[tuple[int, FunctionDep]] = []
    for candidate in candidates:
        score = 0
        for argument, (expected, _name) in zip(arguments, candidate.params, strict=True):
            actual = _argument_c_type(argument, symbols, call_returns)
            if actual is None:
                continue
            if _abi_types_compatible(actual, expected):
                score += 1
        scores.append((score, candidate))
    if scores:
        best = max(score for score, _ in scores)
        winners = [candidate for score, candidate in scores if score == best]
        if best > 0 and len(winners) == 1:
            return winners[0]
    choices = ", ".join(
        f"{candidate.name}({', '.join(c_type for c_type, _ in candidate.params)})"
        for candidate in candidates
    )
    raise CppComError(
        f"{class_name}: cannot bind overloaded sibling call {source_name}"
        f"({', '.join(argument.strip() for argument in arguments)}) in {caller.name}; "
        f"candidates: {choices or '<none>'}"
    )


def _argument_c_type(
    argument: str, symbols: dict[str, str], call_returns: dict[str, str] | None = None
) -> str | None:
    text = argument.strip()
    explicit_cast = re.search(r"\(\s*(?P<type>[A-Za-z_]\w*(?:\s+[A-Za-z_]\w*)*\s*\*+)\s*\)", text)
    if explicit_cast is not None:
        return " ".join(explicit_cast.group("type").split())
    call = re.match(r"(?P<name>[A-Za-z_]\w*)\s*\(", text)
    if call is not None and call_returns is not None:
        return_type = call_returns.get(call.group("name"))
        if return_type is not None:
            return return_type
    text = re.sub(r"^\([^()]+\)\s*", "", text)
    conditional = re.fullmatch(r"[A-Za-z_]\w*\s*\?\s*([A-Za-z_]\w*)\s*:\s*(.+)", text, flags=re.S)
    if conditional is not None:
        true_type = symbols.get(conditional.group(1))
        false_text = conditional.group(2).strip()
        false_type = (
            "LPCWSTR"
            if re.fullmatch(r"CPP_COM_WSTR_[A-F0-9_]+\.as_ptr\(\)", false_text)
            else _argument_c_type(false_text, symbols, call_returns)
        )
        if true_type is not None and (
            false_type is None or _abi_types_compatible(true_type, false_type)
        ):
            return true_type
    if re.fullmatch(r'L"(?:\\.|[^"\\])*"', text):
        return "LPCWSTR"
    if re.fullmatch(r'"(?:\\.|[^"\\])*"', text):
        return "LPCSTR"
    identifier = re.fullmatch(r"(?:this\s*->\s*)?([A-Za-z_]\w*)", text)
    if identifier is None:
        return None
    c_type = symbols.get(identifier.group(1))
    if c_type == "CStringW":
        return "LPCWSTR"
    if c_type == "CStringA":
        return "LPCSTR"
    return c_type


def _abi_types_compatible(actual: str, expected: str) -> bool:
    left, right = _abi_c_type_key(actual), _abi_c_type_key(expected)
    if left == right:
        return True
    if all("PIDL" in item or "ITEMID" in item for item in (left, right)):
        return True
    return {left, right} in ({"LPWSTR", "LPCWSTR"}, {"LPSTR", "LPCSTR"})


def require_supported_worker_raii(deps: list) -> None:
    """Reject body-local C++ owners whose lifetime is not deterministic yet."""
    unsupported = []
    for dep in deps:
        if re.search(r"\bCDrop\s+[A-Za-z_]\w*\s*\(", dep.body_text):
            unsupported.append(f"{dep.name}: CDrop local")
    if unsupported:
        raise CppComError(
            "unsupported body-local C++ RAII requires deterministic lowering before payment: "
            + ", ".join(unsupported)
        )


def apply_interface_method_signatures(
    deps: list[FunctionDep], class_name: str, interfaces: list[InterfaceSpec]
) -> list[FunctionDep]:
    """Replace parsed COM method types with the flattened vtable authority.

    C++ error recovery can find an out-of-line ``WINAPI Class::Method`` body
    while losing its return type. WIDL already defines the exact ABI shape, so
    scanner types must never win for interface methods. Source parameter names
    are retained because the C++ body refers to them.
    """
    methods: dict[str, list[InterfaceMethod]] = {}
    for interface in interfaces:
        for method in interface.methods:
            if method.name in ("QueryInterface", "AddRef", "Release"):
                continue
            variants = methods.setdefault(method.name, [])
            shape = _interface_method_shape(method)
            if all(_interface_method_shape(prior) != shape for prior in variants):
                variants.append(method)

    out = []
    for dep in deps:
        variants = methods.get(dep.cpp_method or "") if dep.cpp_class == class_name else None
        if not variants:
            out.append(dep)
            continue
        source_shape = tuple(_abi_c_type_key(c_type) for c_type, _ in dep.params)
        matching = [
            (index, method)
            for index, method in enumerate(variants)
            if tuple(_abi_c_type_key(c_type) for c_type, _ in method.params[1:]) == source_shape
        ]
        if len(matching) != 1:
            same_arity = [
                (index, method)
                for index, method in enumerate(variants)
                if len(method.params) - 1 == len(dep.params)
            ]
            if len(same_arity) == 1:
                matching = same_arity
        if len(matching) != 1:
            choices = ", ".join(
                "(" + ", ".join(c_type for c_type, _ in method.params[1:]) + ")"
                for method in variants
            )
            raise CppComError(
                f"{class_name}::{dep.cpp_method}: cannot bind scanner signature "
                f"({', '.join(c_type for c_type, _ in dep.params)}) to vtable variants: "
                f"{choices}"
            )
        variant_index, method = matching[0]
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
        target_name = f"{class_name}_{dep.cpp_method}"
        if len(variants) > 1:
            target_name += f"_v{variant_index + 1}"
        out.append(replace(dep, name=target_name, return_type=method.return_type, params=params))
    return out


def _interface_method_shape(method: InterfaceMethod) -> tuple[str, tuple[str, ...]]:
    """Canonical ABI shape excluding the per-interface receiver pointer."""
    return (
        _abi_c_type_key(method.return_type),
        tuple(_abi_c_type_key(c_type) for c_type, _ in method.params[1:]),
    )


def _abi_c_type_key(c_type: str) -> str:
    """Collapse spelling-only Win32 aliases for overload binding, not codegen."""
    text = " ".join(c_type.replace("CONST", "const").split())
    aliases = {
        "int": "INT",
        "unsigned int": "UINT",
        "PSTR": "LPSTR",
        "PWSTR": "LPWSTR",
        "PCSTR": "LPCSTR",
        "PCWSTR": "LPCWSTR",
        "LPCOLESTR": "LPCWSTR",
        "LPOLESTR": "LPWSTR",
        "LPVOID": "void *",
        "PVOID": "void *",
    }
    for spelling, canonical in aliases.items():
        text = re.sub(rf"\b{re.escape(spelling)}\b", canonical, text)
    return re.sub(r"\s*\*\s*", "*", text)


def _interface_method_target(
    class_name: str, method: InterfaceMethod, interfaces: list[InterfaceSpec]
) -> str:
    variants: list[InterfaceMethod] = []
    for interface in interfaces:
        for candidate in interface.methods:
            if candidate.name != method.name or candidate.name in (
                "QueryInterface",
                "AddRef",
                "Release",
            ):
                continue
            if all(
                _interface_method_shape(prior) != _interface_method_shape(candidate)
                for prior in variants
            ):
                variants.append(candidate)
    target = f"{class_name}_{method.name}"
    if len(variants) > 1:
        shape = _interface_method_shape(method)
        variant = next(
            i for i, item in enumerate(variants, 1) if _interface_method_shape(item) == shape
        )
        target += f"_v{variant}"
    return target


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
    printer_enum = re.fullmatch(
        r"return\s+ShellObjectCreatorInit\s*<\s*CPrintersEnum\s*>\s*\(\s*"
        r"([A-Za-z_]\w*)\s*,\s*([A-Za-z_]\w*)\s*,\s*"
        r"IID_PPV_ARG\s*\(\s*IEnumIDList\s*,\s*([A-Za-z_]\w*)\s*\)\s*\)\s*;",
        body,
    )
    if printer_enum is not None:
        owner, flags, output = printer_enum.groups()
        return (
            expected_signature + f" {{ CPrintersEnum_CreateInstance({owner}, {flags}, {output}) }}"
        )
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
    signature = re.sub(r"(?<=[(,])\s*mut\s+", " ", signature)
    signature = re.sub(r"\s+", " ", signature).strip()
    signature = re.sub(r"\(\s+", "(", signature)
    signature = re.sub(r",\s*\)", ")", signature)
    signature = re.sub(r"\s+\)", ")", signature)
    # An omitted unit return and an explicit `-> ()` are the same Rust signature.
    return re.sub(r"\s*->\s*\(\)\s*$", "", signature)


def _source_local_pod_fields(c_body: str) -> dict[str, tuple[str, ...]]:
    """Return the field names of POD layouts declared inside one C worker.

    A body-local helper such as ``Stream_ReadChunk::sized_chunk`` is not COM or
    public ABI owned by the model: its authoritative definition is part of the
    snippet being translated.  Keep this deliberately narrow.  Anonymous types,
    unions, enums, nested declarations, bitfields, and multi-declarators remain
    outside the accepted worker contract.
    """
    result: dict[str, tuple[str, ...]] = {}
    for match in re.finditer(r"\bstruct\s+([A-Za-z_]\w*)\s*\{", c_body):
        name = match.group(1)
        open_brace = c_body.find("{", match.start())
        try:
            inner, _ = _balanced_block(c_body, open_brace)
        except CppComError:
            continue
        fields: list[str] = []
        valid = True
        for declaration in inner.split(";"):
            declaration = " ".join(declaration.split())
            if not declaration:
                continue
            field = re.search(r"([A-Za-z_]\w*)\s*(?:\[[^]]+\])?\s*$", declaration)
            if field is None or any(token in declaration for token in (",", ":", "{")):
                valid = False
                break
            fields.append(field.group(1))
        if valid and fields:
            result[name] = tuple(fields)
    return result


def _source_anonymous_pod_fields(c_body: str) -> set[tuple[str, ...]]:
    """Return field sequences for simple body-local anonymous C records.

    C commonly spells a one-use lookup-table row as ``struct { ... } Rows[]``.
    Rust must name that local type, but the chosen name has no ABI or source
    semantics. Its field sequence is still source authority and is audited just
    like a named local POD.
    """
    result = set()
    for match in re.finditer(
        r"\bstruct\s*\{(?P<body>[^{}]*)\}\s*[A-Za-z_]\w*\s*(?:\[[^]]*\])?\s*=",
        c_body,
        flags=re.S,
    ):
        fields = []
        valid = True
        for declaration in match.group("body").split(";"):
            declaration = " ".join(declaration.split())
            if not declaration:
                continue
            field = re.search(r"([A-Za-z_]\w*)\s*(?:\[[^]]+\])?\s*$", declaration)
            if field is None or any(token in declaration for token in (",", ":", "{")):
                valid = False
                break
            fields.append(field.group(1))
        if valid and fields:
            result.add(tuple(fields))
    return result


def _rust_local_pod_fields(rust: str) -> dict[str, tuple[str, ...]]:
    result: dict[str, tuple[str, ...]] = {}
    for match in re.finditer(
        r"#\s*\[\s*repr\s*\(\s*C\s*\)\s*\]\s*struct\s+([A-Za-z_]\w*)\s*\{", rust
    ):
        name = match.group(1)
        open_brace = rust.find("{", match.start())
        try:
            inner, _ = _balanced_block(rust, open_brace)
        except CppComError:
            continue
        fields = tuple(
            field.group(1)
            for field in re.finditer(r"(?:^|,)\s*(?:pub\s+)?(?:r#)?([A-Za-z_]\w*)\s*:", inner)
        )
        result[name] = fields
    return result


def _rust_local_struct_fields(rust: str) -> dict[str, tuple[str, ...]]:
    """Return fields of any simple Rust local struct, irrespective of repr.

    This is used only for a source-anonymous, local-only record whose Rust name
    and layout representation are unobservable. Named C PODs continue to require
    the stricter `repr(C)` map above.
    """
    result = {}
    for match in re.finditer(r"\bstruct\s+([A-Za-z_]\w*)\s*\{", rust):
        name = match.group(1)
        open_brace = rust.find("{", match.start())
        try:
            inner, _ = _balanced_block(rust, open_brace)
        except CppComError:
            continue
        result[name] = tuple(
            field.group(1)
            for field in re.finditer(r"(?:^|,)\s*(?:pub\s+)?(?:r#)?([A-Za-z_]\w*)\s*:", inner)
        )
    return result


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

    executable_rust = re.sub(r'/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"', " ", rust, flags=re.S)

    # The opaque call tokens are occurrence-specific. Reject a reversed nesting
    # relation after unsealing, so a cheap model cannot turn
    # `HeapFree(GetProcessHeap(), ...)` into `GetProcessHeap(HeapFree(), ...)`
    # while still satisfying the closed call-count set. Benign hoisting such as
    # `let heap = GetProcessHeap(); HeapAlloc(heap, ...)` remains allowed.
    from .call_seal import _EXCLUDED as call_excluded

    def nested_calls(text: str) -> set[tuple[str, str]]:
        open_brace, close_brace = text.find("{"), text.rfind("}")
        body = text[open_brace + 1 : close_brace] if 0 <= open_brace < close_brace else text
        masked = _masked_cpp(body)
        excluded = (
            call_excluded
            | _CPP_COM_TYPE_WORDS
            | {
                "if",
                "for",
                "return",
                "switch",
                "while",
            }
        )
        calls = []
        for match in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", masked):
            name = match.group(1)
            if name in excluded:
                continue
            opening = masked.find("(", match.end(1))
            try:
                closing = _matching_paren(masked, opening)
            except CppComError:
                continue
            calls.append((name, opening, closing))
        return {
            (outer_name, inner_name)
            for outer_name, outer_open, outer_close in calls
            for inner_name, inner_open, inner_close in calls
            if outer_open < inner_open and inner_close < outer_close
        }

    source_nesting = nested_calls(strip_debug_statements(c_body))
    rust_nesting = nested_calls(executable_rust)
    reversed_pairs = sorted(
        (outer, inner)
        for outer, inner in source_nesting
        if outer != inner and (inner, outer) in rust_nesting
    )
    if reversed_pairs:
        findings.append(
            "runtime call nesting reversed from deterministic envelope: "
            + ", ".join(f"{outer}({inner}(...))" for outer, inner in reversed_pairs)
        )
    forbidden = {
        "#[no_mangle]": "export attribute",
        ".unwrap()": "Option/panic ABI guess",
        "scopeguard::": "model-owned ownership guard",
    }
    for token, description in forbidden.items():
        if token in executable_rust:
            findings.append(f"worker emitted forbidden {description}: {token}")
    abi_surface = re.sub(r"/\*.*?\*/|//[^\n]*", " ", rust, flags=re.S)
    if re.search(
        r'\bextern\s+"(?:C|system)"\s*\{|'
        r'\b(?:pub\s+)?(?:unsafe\s+)?extern\s+"(?:C|system)"\s+fn\s+[A-Za-z_]\w*',
        abi_surface,
    ):
        findings.append('worker emitted forbidden ABI declaration: extern "')
    if re.search(r"(?m)\bgoto\s+[A-Za-z_]\w*\s*;|^\s*[A-Za-z_]\w*\s*:\s*$", executable_rust):
        findings.append("worker emitted raw C goto/label syntax")
    if re.search(r"\?[^;{}\n]+:\s*[^:={}]", executable_rust):
        findings.append("worker emitted raw C ternary syntax")
    # Calling QueryInterface through an input object's deterministic vtable is
    # ordinary method-body behavior.  Only a model-owned implementation would
    # compete with the generated COM scaffold.
    if re.search(r"\bfn\s+[A-Za-z_]\w*QueryInterface\s*\(", executable_rust):
        findings.append("worker emitted forbidden QueryInterface implementation")
    if re.search(r"\bfn\s+[A-Za-z_]\w*CreateInstance\s*\(", executable_rust):
        findings.append("worker emitted forbidden activation factory implementation")
    if re.search(
        r"\b(?:struct|static)\s+[A-Za-z_]\w*Vtbl\b|\b[A-Za-z_]\w*Vtbl\s*\{",
        executable_rust,
    ):
        findings.append("worker emitted forbidden vtable definition")
    declared_types = re.findall(r"\b(?:struct|union|enum)\s+([A-Za-z_]\w*)\b", executable_rust)
    if declared_types:
        source_local = _source_local_pod_fields(c_body)
        anonymous_local = _source_anonymous_pod_fields(c_body)
        rust_local = _rust_local_pod_fields(executable_rust)
        rust_any_local = _rust_local_struct_fields(executable_rust)
        invalid_types = [
            name
            for name in declared_types
            if (
                (name not in source_local or rust_local.get(name) != source_local[name])
                and rust_any_local.get(name) not in anonymous_local
            )
        ]
        if invalid_types:
            findings.append(
                "worker emitted forbidden ABI type declaration: " + ", ".join(invalid_types)
            )

    # Strings and comments contain printf formats and prose, not program constants.
    audited_c_body = strip_debug_statements(c_body)
    open_brace, close_brace = audited_c_body.find("{"), audited_c_body.rfind("}")
    source_body = (
        audited_c_body[open_brace + 1 : close_brace]
        if open_brace >= 0 and close_brace > open_brace
        else audited_c_body
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
        declaration_type = bool(re.match(r"(?:\s+|\s*\*+\s*)[A-Za-z_]\w*", tail))
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
        if re.search(rf"(?<!\*)\b(?:pub\s+)?const\s+{re.escape(word)}\s*[:=]", rust)
    )
    if redefined:
        findings.append(
            "worker redefined source symbolic constant(s) instead of using substrate: "
            + ", ".join(redefined)
        )

    source_integers = _integer_values(source)
    rust_integers = _integer_values(rust)
    for pointer in re.findall(r"\*\s*([A-Za-z_]\w*)\s*=\s*0\s*;", source):
        if (
            re.search(rf"\*\s*{re.escape(pointer)}\s*=\s*core::ptr::null_mut\(\)\s*;", rust)
            and source_integers[0] > 0
        ):
            source_integers[0] -= 1
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
    # COM maps may expose several IIDs through one inherited interface slot.
    # Same-interface aliases are direct (for example IID_IOleWindow routed to
    # IDeskBand), while a derived interface and its named base also share one
    # physical C++ base subobject (IContextMenu2/IContextMenu).  WIDL's declared
    # inheritance is the authority; never infer this merely from vtable-prefix
    # similarity because unrelated interfaces all share IUnknown's prefix.
    mapped_interfaces = {entry.interface for entry in cls.com_map}

    def physical_interface(name: str) -> str:
        candidates = []
        for candidate in sorted(mapped_interfaces):
            current = by_name[candidate]
            while current.base is not None and current.base in by_name:
                if current.base == name:
                    candidates.append(candidate)
                    break
                current = by_name[current.base]
        if not candidates:
            return name
        longest = max(len(by_name[item].methods) for item in candidates)
        owners = [item for item in candidates if len(by_name[item].methods) == longest]
        if len(owners) != 1:
            raise CppComError(
                f"{cls.name}: IID for {name} has ambiguous mapped derived slots: "
                + ", ".join(owners)
            )
        return owners[0]

    physical_by_interface = {name: physical_interface(name) for name in mapped_interfaces}
    layout_entries = [
        next(entry for entry in cls.com_map if entry.interface == name)
        for name in dict.fromkeys(physical_by_interface[entry.interface] for entry in cls.com_map)
    ]

    lines = []
    for enum in cls.enum_state_types:
        lines.append(f"pub type {enum.name} = INT;")
        lines.extend(f"pub const {name}: {enum.name} = {value};" for name, value in enum.variants)
        lines.append("")
    for pod in cls.pod_state_types:
        representation = "#[repr(C, packed)]" if pod.packed else "#[repr(C)]"
        lines.extend([representation, f"pub struct {pod.name} {{"])
        for state in pod.fields:
            field_name = f"r#{state.name}" if state.name in _RUST_KEYWORDS else state.name
            lines.append(f"    pub {field_name}: {state.rust_type},")
        lines.extend(["}", ""])

    lines.extend(
        [
            f"// Deterministic COM scaffold for {cls.name}; "
            "method bodies are generated separately.",
            'extern "system" {',
            "    fn GetProcessHeap() -> HANDLE;",
            '    #[link_name = "HeapAlloc"]',
            "    fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) "
            "-> *mut core::ffi::c_void;",
            '    #[link_name = "HeapFree"]',
            "    fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, "
            "memory: *mut core::ffi::c_void) -> BOOL;",
            "    fn Shell32RustObjectLock();",
            "    fn Shell32RustObjectUnlock();",
            "}",
            "trait CppComSize { fn cpp_com_size(self) -> usize; }",
            "impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }",
            "impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }",
            "#[inline]",
            "unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) "
            "-> *mut core::ffi::c_void {",
            "    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())",
            "}",
            "#[inline]",
            "unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {",
            "    cpp_com_HeapFree(heap, flags, memory.cast())",
            "}",
            *(
                []
                if any(interface.name == "IUnknown" for interface in interfaces)
                else ['extern "C" {', "    pub static IID_IUnknown: IID;", "}"]
            ),
            "",
            "#[repr(C)]",
            f"pub struct {cls.name} {{",
        ]
    )
    for entry in layout_entries:
        lines.append(f"    pub {entry.interface}_iface: {entry.interface},")
    lines.append("    pub ref_count: core::sync::atomic::AtomicI32,")
    for state in cls.plain_state_fields():
        lines.append(f"    pub {state.name}: {state.rust_type},")
    lines.extend(["}", ""])

    for state in cls.plain_state_fields():
        if state.com_ptr_interface is None:
            continue
        helper = f"{cls.name}_{state.assignment_helper}"
        interface = state.com_ptr_interface
        lines.extend(
            [
                f"unsafe fn {helper}(this: *mut {cls.name}, value: *mut {interface}) {{",
                f"    let old = (*this).{state.name};",
                "    if old == value { return; }",
                "    if !value.is_null() {",
                "        let unknown = value as *mut IUnknown;",
                '        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =',
                "            core::mem::transmute((*(*unknown).lpVtbl).AddRef);",
                "        add_ref(unknown);",
                "    }",
                f"    (*this).{state.name} = value;",
                "    if !old.is_null() {",
                "        let unknown = old as *mut IUnknown;",
                '        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =',
                "            core::mem::transmute((*(*unknown).lpVtbl).Release);",
                "        release(unknown);",
                "    }",
                "}",
                "",
            ]
        )

    for state in cls.plain_state_fields():
        if not state.owned_cstring:
            continue
        helper = f"{cls.name}_{state.string_assignment_helper}"
        lines.extend(
            [
                f"unsafe fn {helper}(this: *mut {cls.name}, value: LPCWSTR) {{",
                f"    (*this).{state.name}.assign(value);",
                "}",
                "",
            ]
        )

    for entry in layout_entries:
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
                target = f"{_interface_method_target(cls.name, method, interfaces)}_body"
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
    first_physical = physical_by_interface[first.interface]
    lines.extend(
        [
            f"unsafe fn {cls.name}_QueryInterface(",
            f"    this: *mut {cls.name}, riid: REFIID, ppv: *mut *mut core::ffi::c_void,",
            ") -> HRESULT {",
            "    if ppv.is_null() { return E_POINTER; }",
            "    *ppv = core::ptr::null_mut();",
            f"    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || "
            f"cpp_com_iid_equal(riid, core::ptr::addr_of!({first.iid})) {{",
            f"        *ppv = core::ptr::addr_of_mut!((*this).{first_physical}_iface) "
            "as *mut core::ffi::c_void;",
            "    }",
        ]
    )
    for entry in cls.com_map[1:]:
        physical = physical_by_interface[entry.interface]
        lines.extend(
            [
                f"    else if cpp_com_iid_equal(riid, core::ptr::addr_of!({entry.iid})) {{",
                f"        *ppv = core::ptr::addr_of_mut!((*this).{physical}_iface) "
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
        ]
    )
    for state in cls.plain_state_fields():
        if state.assignment_helper is not None:
            lines.append(
                f"        {cls.name}_{state.assignment_helper}(this, core::ptr::null_mut());"
            )
        if state.owned_cstring:
            lines.append(f"        core::ptr::drop_in_place(&mut (*this).{state.name});")
    lines.extend(
        [
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
    for entry in layout_entries:
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
        value = "CStringW::new()" if state.owned_cstring else "core::mem::zeroed()"
        lines.append(f"        {state.name}: {value},")
    lines.extend(
        [
            "    });",
            "    Shell32RustObjectLock();",
            f"    {cls.name}_ctor_body(this);",
        ]
    )
    if cls.final_construct:
        lines.extend(
            [
                f"    let construct_hr = {cls.name}_FinalConstruct_body(this);",
                "    if construct_hr < 0 {",
                f"        {cls.name}_Release(this);",
                "        return construct_hr;",
                "    }",
            ]
        )
    lines.extend(
        [
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
    lines = [
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
    source = cls.source.read_text(encoding="utf-8", errors="replace")
    if "SHELL_MapContextMenuVerbToCmdId" in source:
        lines.extend(
            [
                "",
                'extern "C" INT WINAPI Shell32RustMapContextMenuVerbToCmdId(',
                "    LPCMINVOKECOMMANDINFO pInfo, const CMVERBMAP *pMap)",
                "{",
                "    return SHELL_MapContextMenuVerbToCmdId(pInfo, pMap);",
                "}",
            ]
        )
    if "SHELL_GetCommandStringImpl" in source:
        lines.extend(
            [
                "",
                'extern "C" HRESULT WINAPI Shell32RustGetCommandStringImpl(',
                "    SIZE_T CmdId, UINT uFlags, LPSTR Buf, UINT cchBuf, const CMVERBMAP *pMap)",
                "{",
                "    return SHELL_GetCommandStringImpl(CmdId, uFlags, Buf, cchBuf, pMap);",
                "}",
            ]
        )
    if "SHELL32_CoCreateInitSF" in source:
        lines.extend(
            [
                "",
                'extern "C" HRESULT WINAPI Shell32RustCoCreateInitSF(',
                "    LPCITEMIDLIST pidlRoot, const GUID *clsid, INT csidl,",
                "    REFIID riid, LPVOID *ppvOut)",
                "{",
                "    return SHELL32_CoCreateInitSF(pidlRoot, clsid, csidl, riid, ppvOut);",
                "}",
            ]
        )
    return "\n".join(lines)


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
            repo_root / "sdk" / "include" / "reactos",
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
