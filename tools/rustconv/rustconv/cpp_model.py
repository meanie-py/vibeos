"""The C++ class model of a module, read from the compiler, not from regexes.

`cpp_com.analyze_class` reads class text with patterns, which is right for the narrow,
proven COM subset it accepts and wrong for an application: mspaint's `CCanvasWindow`,
sndrec32's `audio_membuffer : public audio_receiver, public audio_producer` (pure-virtual
bases in other headers), header-inline method bodies, in-class member initializers,
constructor init lists, and `audio_format UNKNOWN_FORMAT(0, 0, 0);` globals are all things
clang has already resolved. So this asks clang: each TU's real compile command
(`compile_commands.json`) with `-Xclang -ast-dump=json`, keeping only declarations whose
source lives in the module's own directory.

The output is a plain-data model (`ModuleModel`) that `cpp_app` renders into the Rust
skeleton. Bodies are kept as SOURCE TEXT spans: the model translates text, and the span is
how a worker's prompt and its seal find the body.

A TU costs ~20 s and a few hundred MB of JSON (the whole of windows.h and ATL), so the
extracted model is cached per TU, keyed on the TU's include-closure mtimes.
"""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import asdict, dataclass, field
from pathlib import Path

from . import c_data_port as cdp
from . import cpp_lower, cpp_templates
from .cpp_templates import instance_key, instance_name


def _anon_field(anon: ClassModel) -> str:
    """`CTextEventSink__anon0` -> its member name in the class, `__anon0`."""
    return "__" + anon.name.rsplit("__", 1)[1]


class CppModelError(RuntimeError):
    pass


@dataclass
class Param:
    name: str
    c_type: str
    default: str | None = None  # the default argument's text


@dataclass
class Method:
    name: str
    kind: str  # "method" | "ctor" | "dtor" | "operator"
    params: list[Param]
    return_type: str
    virtual: bool = False
    pure: bool = False
    static: bool = False
    const: bool = False
    access: str = "public"
    # Where the BODY is: (file relative to the repo, begin offset, end offset), or None for a
    # declaration with no definition anywhere in the module (pure virtual, or defined in a
    # TU this module does not compile).
    body: tuple[str, int, int] | None = None
    # Constructor member/base initializers, in written order: (member-or-base, init text).
    inits: list[tuple[str, str]] = field(default_factory=list)
    inline: bool = False  # body written inside the class definition
    rust_name: str = ""  # unique within the class: overloads get `__N` (see _assign_names)
    lowered: str | None = None  # the body after cpp_lower, what a worker translates
    lowered_inits: list[tuple[str, str]] = field(default_factory=list)
    unplaced: list[str] = field(default_factory=list)
    # The member calls its lowered body resolved (cpp_lower.Lowered.calls): external reach.
    calls: list[tuple[str, str]] = field(default_factory=list)
    # "system" for a WINAPI/CALLBACK (stdcall) member: a static one is handed to the OS as a
    # callback (`EnumWindows(EnumSingleInstanceCallback, ..)`), so its Rust ABI must match.
    abi: str = ""

    @property
    def signature_key(self) -> str:
        return f"{self.name}({', '.join(p.c_type for p in self.params)})"


@dataclass
class Field:
    name: str
    c_type: str
    access: str = "public"
    static: bool = False
    default_init: str | None = None  # the in-class initializer's text


@dataclass
class Base:
    name: str  # as written, template arguments included (`CWindowImpl<CCanvasWindow>`)
    access: str
    virtual: bool = False

    @property
    def template(self) -> str | None:
        m = re.match(r"^\s*([A-Za-z_][\w:]*)\s*<", self.name)
        return m.group(1).split("::")[-1] if m else None


@dataclass
class MsgMapEntry:
    macro: str  # MESSAGE_HANDLER, COMMAND_ID_HANDLER, NOTIFY_CODE_HANDLER, CHAIN_MSG_MAP, ...
    args: list[str]


@dataclass
class ClassModel:
    name: str
    qualified: str
    tag: str  # class | struct | union
    file: str
    bases: list[Base] = field(default_factory=list)
    fields: list[Field] = field(default_factory=list)
    methods: list[Method] = field(default_factory=list)
    msg_map: list[MsgMapEntry] | None = None
    polymorphic: bool = False
    nested_in: str | None = None
    # `enum { IDD = IDD_MIRRORROTATE };` -- class-scoped constants, by clang's folded value.
    constants: list[tuple[str, int]] = field(default_factory=list)
    # DECLARE_WND_CLASS_EX(name, style, bkgnd) / DECLARE_WND_CLASS(name) arguments, as written.
    wnd_class: list[str] | None = None
    # A class from outside the module the module's classes build on (ATL's CWindow): rendered
    # like a module class, but only the methods the module reaches are translated.
    external: bool = False
    # The class template this class instantiates (`CUiWindow` for `CUiWindow_CTreeView`):
    # inside its own bodies the bare template name is the injected class name, i.e. itself.
    template: str | None = None
    # Its template parameters and the arguments bound to them, `[("T", "CTreeView")]`: a
    # body's text still spells `T` and `GrowthRate`, which mean the arguments here.
    template_args: list[tuple[str, str]] = field(default_factory=list)
    # BEGIN_COM_MAP entries (COM_INTERFACE_ENTRY(IFoo), ...), in order; None without a map.
    com_map: list[MsgMapEntry] | None = None

    def method(self, name: str) -> list[Method]:
        return [m for m in self.methods if m.name == name]


@dataclass
class GlobalVar:
    name: str
    c_type: str
    file: str
    init: str | None  # constructor arguments / initializer text, None for default
    static: bool = False


@dataclass
class EnumModel:
    name: str
    underlying: str  # the C type, `int` when unspecified
    members: list[tuple[str, int]]
    file: str


@dataclass
class FreeFunc:
    name: str
    signature: str  # the declarator text up to the body, as written
    file: str
    lowered: str
    static: bool = False
    unplaced: list[str] = field(default_factory=list)
    # The member calls its lowered body resolved (cpp_lower.Lowered.calls): external reach.
    calls: list[tuple[str, str]] = field(default_factory=list)


@dataclass
class ModuleModel:
    classes: dict[str, ClassModel] = field(default_factory=dict)
    globals: list[GlobalVar] = field(default_factory=list)
    functions: list[FreeFunc] = field(default_factory=list)
    enums: list[EnumModel] = field(default_factory=list)
    # User-declared global allocation operators (cicbase.h's LocalAlloc pair), by role --
    # "new", "new[]", "delete", "delete[]", "new:<Tag>", "new[]:<Tag>" -- to their i386
    # Itanium name without the C decoration underscore (`_ZnwjRK10CicNoThrow`). The
    # generated cpp_alloc/cpp_free call THESE, so a C++ half and a Rust half of one heap
    # never cross allocators mid-flip.
    allocators: dict[str, str] = field(default_factory=dict)
    # role -> the translatable name of the module's OWN definition of that operator
    # (`cxx_operator_new`): cpp_classes exports it under the mangled symbol, so the operator
    # is Rust too and no C++ half has to stay behind to provide it (cicero's cicbase.cpp).
    allocator_fns: dict[str, str] = field(default_factory=dict)
    # File-scope typedefs of the module's .cpp files, as (file, text): main.cpp's
    # `typedef HWND (WINAPI *FN_HtmlHelpW)(..)`. A header's are the island's.
    typedefs: list[tuple[str, str]] = field(default_factory=list)
    # Each spelling of a template instantiation (cpp_templates.instance_key) -> the concrete
    # class that lays it out (`CUiWindow<CTreeView>` -> `CUiWindow_CTreeView`).
    instances: dict[str, str] = field(default_factory=dict)

    def to_json(self) -> str:
        return json.dumps(asdict(self), indent=1, sort_keys=True)

    @classmethod
    def from_json(cls, text: str) -> ModuleModel:
        raw = json.loads(text)
        out = cls()
        for name, c in raw["classes"].items():
            c["bases"] = [Base(**b) for b in c["bases"]]
            c["fields"] = [Field(**f) for f in c["fields"]]
            methods = []
            for m in c["methods"]:
                m["params"] = [Param(**p) for p in m["params"]]
                m["body"] = tuple(m["body"]) if m["body"] else None
                m["inits"] = [tuple(i) for i in m["inits"]]
                m["lowered_inits"] = [tuple(i) for i in m["lowered_inits"]]
                methods.append(Method(**m))
            c["methods"] = methods
            c["constants"] = [tuple(k) for k in c.get("constants", [])]
            c["template_args"] = [tuple(k) for k in c.get("template_args", [])]
            if c["msg_map"] is not None:
                c["msg_map"] = [MsgMapEntry(**e) for e in c["msg_map"]]
            if c.get("com_map") is not None:
                c["com_map"] = [MsgMapEntry(**e) for e in c["com_map"]]
            out.classes[name] = ClassModel(**c)
        out.globals = [GlobalVar(**g) for g in raw["globals"]]
        out.functions = [FreeFunc(**f) for f in raw.get("functions", [])]
        out.enums = [
            EnumModel(e["name"], e["underlying"], [tuple(m) for m in e["members"]], e["file"])
            for e in raw.get("enums", [])
        ]
        out.allocators = raw.get("allocators", {})
        out.allocator_fns = raw.get("allocator_fns", {})
        out.typedefs = [tuple(t) for t in raw.get("typedefs", [])]
        out.instances = raw.get("instances", {})
        return out


_ALLOC_ROLE = [
    (re.compile(r"void \*\(size_t\)"), "{op}"),
    (re.compile(r"void \*\(size_t, const (\w+) &\) noexcept"), "{op}:{tag}"),
    (re.compile(r"void \(void \*\)(?: noexcept)?"), "{op}"),
]


def allocator_role(name: str, qual_type: str) -> str | None:
    """`operator new` + `void *(size_t, const CicNoThrow &) noexcept` -> "new:CicNoThrow".
    None for a shape nothing calls by role (sized delete, placement into a buffer)."""
    op = name.removeprefix("operator ").replace(" ", "")
    if op not in ("new", "new[]", "delete", "delete[]"):
        return None
    for pat, role in _ALLOC_ROLE:
        m = pat.fullmatch(qual_type)
        if m and (op.startswith("new") == ("size_t" in qual_type)):
            return role.format(op=op, tag=m.group(1) if m.groups() else "")
    return None


# ---------------------------------------------------------------------------------------
# AST helpers
# ---------------------------------------------------------------------------------------


_TEMPLATE_USE_RE = re.compile(r"\b(?:\w+::)*\w+\s*<")


def _instance_spellings(written: str, desugared: str) -> list[tuple[str, str]]:
    """Pair each template-id in `written` with its counterpart in `desugared`, in order:
    `CLocalPtr<WCHAR> *` / `CLocalPtr<wchar_t> *` -> [("CLocalPtr<WCHAR>",
    "CLocalPtr<wchar_t>")]. Only when both spell the same templates in the same order."""

    def ids(text: str) -> list[str]:
        out = []
        for m in _TEMPLATE_USE_RE.finditer(text):
            j, depth = m.end(), 1
            while j < len(text) and depth:
                depth += (text[j] == "<") - (text[j] == ">")
                j += 1
            if not depth:
                out.append(text[m.start() : j])
        return out

    a, b = ids(written), ids(desugared)
    if len(a) != len(b) or any(short_name(x) != short_name(y) for x, y in zip(a, b, strict=True)):
        return []
    return list(zip(a, b, strict=True))


def _template_args(spec: dict) -> list[str]:
    """A ClassTemplateSpecializationDecl's arguments as written by clang: a type's spelling,
    an integer's value."""
    out = []
    for a in spec.get("inner", []):
        if a.get("kind") != "TemplateArgument":
            continue
        if "type" in a:
            out.append(a["type"]["qualType"])
        elif "value" in a:
            out.append(str(a["value"]))
        else:
            raise CppModelError(
                f"template {spec.get('name')}: an argument that is neither a type nor a value"
            )
    return out


def _signature_from_ast(n: dict) -> str:
    """`int fnFileClose(INT_PTR hf)`: a FunctionDecl's declarator from its type and parameter
    names, with its calling convention as winnt.h spells it."""
    qt = n.get("type", {}).get("qualType", "")
    params = []
    for i, p in enumerate(x for x in n.get("inner", []) if x.get("kind") == "ParmVarDecl"):
        t, name = p["type"]["qualType"], p.get("name", f"arg{i}")
        # A reference/pointer to an array or function spells its name inside: `(&Base)[260]`.
        # clang writes an instantiated one without the parentheses (`const char &[4]`).
        m = re.fullmatch(r"(.*?)\(?([&*])\)?(\[.*\]|\(.*\))", t)
        params.append(f"{m.group(1)}({m.group(2)}{name}){m.group(3)}" if m else f"{t} {name}")
    cc = " WINAPI" if "stdcall" in qt else ""
    static = "static " if n.get("storageClass") == "static" else ""
    return f"{static}{_return_type(qt)}{cc} {n['name']}({', '.join(params) or 'void'})"


def _template_param_names(tmpl: dict) -> list[str]:
    return [
        p.get("name", f"__param{i}")
        for i, p in enumerate(
            p
            for p in tmpl.get("inner", [])
            if p.get("kind")
            in ("TemplateTypeParmDecl", "NonTypeTemplateParmDecl", "TemplateTemplateParmDecl")
        )
    ]


def _template_defaults(tmpl: dict) -> list[tuple[str, ...] | None]:
    """Each template parameter's default argument -- its spelling then clang's desugared
    form, or an integer's value -- None if it has none."""
    out: list[tuple[str, ...] | None] = []
    for p in tmpl.get("inner", []):
        if p.get("kind") not in ("TemplateTypeParmDecl", "NonTypeTemplateParmDecl"):
            continue
        d = p.get("defaultArg")
        if d is None:
            out.append(None)
        elif "type" in d:
            t = d["type"]
            out.append(tuple(x for x in (t["qualType"], t.get("desugaredQualType")) if x))
        else:
            v = _const_value(p) if p.get("inner") else None
            out.append(None if v is None else (str(v),))
    return out


def _file_of(node: dict, key: str = "loc") -> str | None:
    loc = node.get(key, {})
    if key == "range":
        loc = loc.get("begin", {})
    return cdp._loc(loc).get("_file")


def _span(node: dict) -> tuple[str, int, int] | None:
    """(file, begin, end-exclusive) of a node's source range, at its EXPANSION site."""
    rng = node.get("range")
    if not rng:
        return None
    b, e = cdp._loc(rng["begin"]), cdp._loc(rng["end"])
    if "offset" not in b or "offset" not in e or b.get("_file") != e.get("_file"):
        return None
    return b["_file"], b["offset"], e["offset"] + e.get("tokLen", 0)


class _Sources:
    def __init__(self) -> None:
        self._cache: dict[str, bytes] = {}

    def typedef_text(self, span: tuple[str, int, int] | None) -> str | None:
        """A typedef through its `;`: clang's range for a function-pointer typedef ENDS at
        the name (`typedef HWND (WINAPI *FN_HtmlHelpW`), short of its parameter list."""
        if span is None or self.text(span) is None:
            return None
        f, b, _e = span
        data = self._cache[f]
        depth, i = 0, b
        while i < len(data):
            c = data[i : i + 1]
            depth += (c in (b"(", b"{")) - (c in (b")", b"}"))
            if c == b";" and depth == 0:
                return data[b : i + 1].decode("utf-8", errors="replace")
            i += 1
        return None

    def node_text(self, node: dict) -> str | None:
        span = self.node_span(node)
        return None if span is None else self.text(span)

    def node_span(self, node: dict) -> tuple[str, int, int] | None:
        """A node's source span. A function-like MACRO call (`RGB(255, 255, 255)` as an
        initializer) has its expansion END reported at the macro NAME by clang's JSON, so
        the span is just `RGB`: extend it through the call's balanced parentheses. The
        lowering needs this as much as the text does: a ctor init lowered over the short
        span came out as `m_rgbBack = RGB;`."""
        span = _span(node)
        text = self.text(span)
        if span is None or text is None:
            return span
        end = (node.get("range") or {}).get("end", {})
        if "expansionLoc" not in end or not re.fullmatch(r"[\s\S]*\b\w+", text):
            return span
        f, b, e = span
        data = self._cache[f]
        i = e
        while i < len(data) and data[i : i + 1] in (b" ", b"\t"):
            i += 1
        if data[i : i + 1] != b"(":
            return span
        depth = 0
        while i < len(data):
            c = data[i : i + 1]
            depth += (c == b"(") - (c == b")")
            i += 1
            if depth == 0:
                return f, b, i
        return span

    def text(self, span: tuple[str, int, int] | None) -> str | None:
        if span is None:
            return None
        f, b, e = span
        if f not in self._cache:
            self._cache[f] = Path(f).read_bytes()
        return self._cache[f][b:e].decode("utf-8", errors="replace")


_FN_TYPE_RE = re.compile(r"^(?P<ret>.*?)\s*\((?P<params>.*)\)(?P<quals>[^()]*)$")


def _return_type(qual_type: str) -> str:
    m = _FN_TYPE_RE.match(qual_type)
    return m.group("ret").strip() if m else "void"


def _is_const_method(qual_type: str) -> bool:
    m = _FN_TYPE_RE.match(qual_type)
    return bool(m and re.search(r"\bconst\b", m.group("quals")))


_MSG_MAP_RE = re.compile(r"BEGIN_MSG_MAP\s*\(\s*\w+\s*\)(?P<body>.*?)END_MSG_MAP\s*\(\s*\)", re.S)
_MSG_ENTRY_RE = re.compile(r"\b([A-Z][A-Z0-9_]+)\s*\(((?:[^()]|\([^()]*\))*)\)")


def parse_msg_map(class_text: str) -> list[MsgMapEntry] | None:
    """The BEGIN_MSG_MAP entries, in order. ATL dispatches FIRST MATCH, so order is
    semantics: two handlers for one message means the second is dead unless the first sets
    bHandled = FALSE."""
    m = _MSG_MAP_RE.search(class_text)
    if m is None:
        return None
    body = re.sub(r"//[^\n]*|/\*.*?\*/", "", m.group("body"), flags=re.S)
    return [
        MsgMapEntry(e.group(1), [a.strip() for a in _split_top(e.group(2))])
        for e in _MSG_ENTRY_RE.finditer(body)
    ]


_COM_MAP_RE = re.compile(r"BEGIN_COM_MAP\s*\(\s*\w+\s*\)(?P<body>.*?)END_COM_MAP\s*\(\s*\)", re.S)


def parse_com_map(class_text: str) -> list[MsgMapEntry] | None:
    """The BEGIN_COM_MAP entries, in order: QueryInterface walks them FIRST MATCH, and
    `IID_IUnknown` answers with the first one's interface."""
    m = _COM_MAP_RE.search(class_text)
    if m is None:
        return None
    body = re.sub(r"//[^\n]*|/\*.*?\*/", "", m.group("body"), flags=re.S)
    return [
        MsgMapEntry(e.group(1), [a.strip() for a in _split_top(e.group(2))])
        for e in _MSG_ENTRY_RE.finditer(body)
    ]


# What CComObject<T> declares over a class with a COM map: the IUnknown its vtables call.
_COM_OBJECT_UNKNOWN = (
    ("QueryInterface", [Param("riid", "REFIID"), Param("ppvObject", "void **")], "HRESULT"),
    ("AddRef", [], "ULONG"),
    ("Release", [], "ULONG"),
)


def _add_com_object_unknown(cls: ClassModel) -> None:
    """`CComObject<T>`'s QueryInterface/AddRef/Release, as members of T: body-less (nothing is
    translated), rendered by cpp_app's COM-object glue, found by the interface vtables and by
    a body's `p->AddRef()` like any member."""
    for name, params, ret in _COM_OBJECT_UNKNOWN:
        if not cls.method(name):
            m = Method(name, "method", list(params), ret)
            m.rust_name = name
            cls.methods.append(m)


def _split_top(text: str) -> list[str]:
    """Split on top-level commas. `<>` nests too: a map entry's template argument
    (`CHAIN_MSG_MAP(CBase<CCanvasWindow, x>)`) is one argument."""
    out, depth, cur = [], 0, []
    for ch in text:
        if ch in "([{<":
            depth += 1
        elif ch in ")]}>":
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(cur))
            cur = []
            continue
        cur.append(ch)
    if "".join(cur).strip():
        out.append("".join(cur))
    return out


# Classes outside the module a module class derives from, by the header that defines them.
EXTERNAL_CLASSES = {
    "CWindow": "sdk/lib/atl/atlwin.h",
    # atltypes.h: value types over the SDK structs (CRect : tagRECT), used by value.
    "CRect": "sdk/lib/atl/atltypes.h",
    "CPoint": "sdk/lib/atl/atltypes.h",
    "CSize": "sdk/lib/atl/atltypes.h",
    # atlbase.h: CRegKey holds an HKEY, which is island-NOMINAL: translated per module, it
    # uses that module's own HKEY rather than a shared crate's guess at it.
    "CRegKey": "sdk/lib/atl/atlbase.h",
    # ReactOS's RAII wait cursor: its ctor/dtor swap the cursor, so it must be modeled, not
    # stubbed. Its static members are DECLSPEC_SELECTANY zero/NULL.
    "CWaitCursor": "sdk/include/reactos/ui/CWaitCursor.h",
    # atlimage.h: mspaint calls CImage's STATIC GetImporterFilterString/
    # GetExporterFilterString (GDI+ codec lists); only what is reached is translated.
    "CImage": "sdk/lib/atl/atlimage.h",
    # rosctrls.h: header-only common-control wrappers over CWindow (no data of their own).
    # certmgr hand-ported its slice before these were modeled; rapps derives from them, and
    # CToolbar<TItemData> is a template, laid out per instantiation like a module one.
    "CListView": "sdk/include/reactos/ui/rosctrls.h",
    "CToolbar": "sdk/include/reactos/ui/rosctrls.h",
    "CStatusBar": "sdk/include/reactos/ui/rosctrls.h",
    "CTreeView": "sdk/include/reactos/ui/rosctrls.h",
    "CTooltips": "sdk/include/reactos/ui/rosctrls.h",
    # GDI+'s C++ API (psdk gdiplus*.h): header-only wrappers over the flat Gdip* exports.
    # rapps draws its screenshot preview with Graphics/SolidBrush/Bitmap.
    "GdiplusBase": "sdk/include/psdk/gdiplusbase.h",
    "Graphics": "sdk/include/psdk/gdiplusgraphics.h",
    "Image": "sdk/include/psdk/gdiplusheaders.h",
    "Bitmap": "sdk/include/psdk/gdiplusheaders.h",
    "Brush": "sdk/include/psdk/gdiplusbrush.h",
    "SolidBrush": "sdk/include/psdk/gdiplusbrush.h",
    "Color": "sdk/include/psdk/gdipluscolor.h",
    "GdiplusStartupInput": "sdk/include/psdk/gdiplusinit.h",
}
# ATL's window templates are, for layout and member lookup, their CWindow: the template's own
# machinery (thunk, Create, DefWindowProc, message routing) is atl_rust::win plus the per-class
# glue cpp_app generates, never a translated body.
ATL_BASE_ALIASES = {
    "CWindowImpl": "CWindow",
    "CDialogImpl": "CWindow",
    "CWindowImplBaseT": "CWindow",
    "CDialogImplBaseT": "CWindow",
    # ReactOS rosdlgs.h: `CPropertyPageImpl<T> : CDialogImplBaseT<CWindow>`. Its `m_psp` is not
    # a field here -- comctl32 copies the page at CreatePropertySheetPage, so the glue builds it
    # on the stack (atl_rust::win::create_property_page).
    "CPropertyPageImpl": "CWindow",
}
# Their methods a subclass calls through an implicit `this`: they stay on the object, where the
# generated glue answers them, instead of resolving to CWindow (whose `Create` takes a class
# name -- a different function).
ATL_IMPL_METHODS = cpp_lower.ATL_IMPL_METHODS


def short_name(name: str) -> str:
    return name.split("<")[0].split("::")[-1].strip()


# Which template argument of an ATL window template is its TBase (the class it derives from).
_ATL_TBASE_ARG = {"CWindowImpl": 1, "CDialogImpl": 1, "CWindowImplBaseT": 0}


def atl_base_alias(name: str) -> str:
    """A base's short name for layout and member lookup: an ATL window template is its TBase
    -- CWindow by default, but rapps' `CWindowImpl<CAppsListView, CListView>` derives from
    rosctrls' CListView, whose methods the class calls as its own."""
    t = short_name(name)
    if t not in ATL_BASE_ALIASES:
        return t
    k = _ATL_TBASE_ARG.get(t)
    if k is not None and "<" in name:
        args = _split_top(name[name.index("<") + 1 : name.rindex(">")])
        if len(args) > k:
            return short_name(args[k])
    return ATL_BASE_ALIASES[t]


def _sig(m: Method) -> str:
    """A signature with qualification dropped: `A::B` and `B` name one type here."""
    return re.sub(r"\b\w+::", "", m.signature_key).replace(" ", "")


def _const_value(node: dict) -> int | None:
    """An enumerator's value as clang folded it (a ConstantExpr's `value`), or None."""
    for x in node.get("inner", []):
        if "value" in x:
            try:
                return int(x["value"])
            except ValueError:
                return None
        v = _const_value(x)
        if v is not None:
            return v
    return None


def _assign_names(cls: ClassModel) -> None:
    """Rust has no overloading: the Nth overload of a name is `name__N` (declaration order),
    constructors are `__ctor_N`, the destructor `__dtor`. Unique names stay as written."""
    counts: dict[str, int] = {}
    for m in cls.methods:
        counts[m.name] = counts.get(m.name, 0) + 1
    seen: dict[str, int] = {}
    ctor = 0
    for m in cls.methods:
        if m.kind == "ctor":
            m.rust_name = f"__ctor_{ctor}"
            ctor += 1
        elif m.kind == "dtor":
            m.rust_name = "__dtor"
        elif m.kind == "operator":
            op = m.name[len("operator") :].strip()
            # A conversion operator (`operator HWND`) is `op_to_HWND`; cpp_lower calls it
            # where C++ converts implicitly.
            conv = "op_to_" + re.sub(r"\W+", "_", op).strip("_")
            m.rust_name = _OPERATOR_NAMES.get(
                op, conv if re.match(r"[A-Za-z_]", op) else "op_unknown"
            )
        elif counts[m.name] > 1:
            m.rust_name = f"{m.name}__{seen.get(m.name, 0)}"
        else:
            m.rust_name = m.name
        seen[m.name] = seen.get(m.name, 0) + 1


_OPERATOR_NAMES = {
    "==": "op_eq",
    "!=": "op_ne",
    "=": "op_assign",
    "<": "op_lt",
    ">": "op_gt",
    "[]": "op_index",
    "()": "op_call",
    "+": "op_add",
    "-": "op_sub",
    "+=": "op_add_assign",
    "-=": "op_sub_assign",
    "*": "op_deref",
    "->": "op_arrow",
    "!": "op_not",
}


def _ctor_args(text: str | None, names: tuple[str, ...]) -> str | None:
    """Normalize an initializer's source text to what the C++ WROTE.

    clang's range for a construct-expression covers the declarator: a global
    `audio_format UNKNOWN_FORMAT(0, 0, 0);` reads `UNKNOWN_FORMAT(0, 0, 0)`, and an implicit
    default construction (`OPENFILENAME ofn;`, or a base class the init list never names)
    reads as the bare name. The first becomes `(0, 0, 0)`; the second becomes None, which
    means value-initialized, exactly as the C++ default does.
    """
    if text is None:
        return None
    t = text.strip()
    for n in names:
        if t == n:
            return None
        if t.startswith(n) and t[len(n) :].lstrip().startswith(("(", "{")):
            return t[len(n) :].strip()
    return t


# ---------------------------------------------------------------------------------------
# extraction
# ---------------------------------------------------------------------------------------


def _method_kind(node: dict) -> str:
    k = node["kind"]
    if k == "CXXConstructorDecl":
        return "ctor"
    if k == "CXXDestructorDecl":
        return "dtor"
    if node.get("name", "").startswith("operator"):
        return "operator"
    return "method"


_METHOD_KINDS = ("CXXMethodDecl", "CXXConstructorDecl", "CXXDestructorDecl", "CXXConversionDecl")


class _Extractor:
    def __init__(self, repo: Path, module_dir: Path) -> None:
        self.repo = repo
        self.module_dir = str(module_dir.resolve())
        self.src = _Sources()
        self.model = ModuleModel()
        self.unicode = False  # the TU compiles with -DUNICODE: TEXT("x") is L"x"
        # Per TU: decl ids mean nothing outside the AST that produced them.
        self.by_id: dict[str, str] = {}  # record decl id -> qualified class name
        self.decls: dict[str, cpp_lower.DeclInfo] = {}
        self._anon_seen: set[str] = set()
        self._anon_registered: set[str] = set()
        self._fn_defaults: dict[str, list[str | None]] = {}
        self._overloads: dict[str, list[str]] = {}
        self.method_by_id: dict[str, Method] = {}
        # Every file-scope typedef seen (`WCHAR` -> `wchar_t`): instance keys (canon_key).
        self.typedef_map: dict[str, str] = {}
        # (function template, instantiation type) -> its free function's name.
        self._fn_instances: dict[tuple[str, str], str] = {}
        # (Method | FreeFunc, CompoundStmt node, this class) to lower once the TU is walked:
        # a body can name a member of a class defined LATER in the same TU.
        self.pending: list[tuple[Method | FreeFunc, dict, str | None]] = []
        self.pending_inits: list[tuple[Method, str, dict, str]] = []

    def mine(self, f: str | None) -> bool:
        return bool(f) and str(Path(f).resolve()).startswith(self.module_dir + "/")

    def external_class(self, n: dict) -> bool:
        """ATL's `CWindow` (atlwin.h), which every CWindowImpl/CDialogImpl subclass is."""
        f = _file_of(n) or ""
        return n.get("name") in EXTERNAL_CLASSES and f.replace("\\", "/").endswith(
            EXTERNAL_CLASSES[n["name"]]
        )

    def rel(self, f: str) -> str:
        return str(Path(f).resolve().relative_to(self.repo.resolve()))

    def walk(self, nodes: list, scope: str = "", record: str | None = None) -> None:
        for n in nodes:
            kind = n.get("kind")
            if kind == "TypedefDecl" and record is None and n.get("name"):
                self.typedef_map.setdefault(n["name"], n.get("type", {}).get("qualType", ""))
            if kind in ("NamespaceDecl", "LinkageSpecDecl"):
                inner_scope = f"{scope}{n['name']}::" if n.get("name") else scope
                self.walk(n.get("inner", []), inner_scope)
            elif kind == "CXXRecordDecl" and n.get("completeDefinition"):
                if not n.get("isImplicit") and (self.mine(_file_of(n)) or self.external_class(n)):
                    self.record(n, scope, record)
            elif kind == "ClassTemplateDecl" and record is None:
                self.class_template(n, scope)
            elif kind == "FunctionTemplateDecl" and record is None:
                self.function_template(n)
            elif kind in _METHOD_KINDS and record is None:
                self.out_of_line(n)
            elif (
                kind == "FunctionDecl"
                and record is None
                and n.get("name", "").startswith("operator ")
                and not n.get("isImplicit")
                and ".build-tools" not in (_file_of(n) or "/.build-tools/")
            ):
                role = allocator_role(n["name"], n.get("type", {}).get("qualType", ""))
                if role and n.get("mangledName"):
                    # i386 adds the C decoration underscore; rustc adds it itself.
                    self.model.allocators[role] = "_" + n["mangledName"].lstrip("_")
                if self.mine(_file_of(n)):
                    self.free_function(n)
                    if role:
                        self.model.allocator_fns[role] = self.fn_rust_name(n)
            elif kind == "FunctionDecl" and record is None and self.mine(_file_of(n)):
                self._register_function(n)
                self.free_function(n)
            elif (
                kind == "TypedefDecl"
                and record is None
                and self.mine(_file_of(n))
                and (_file_of(n) or "").endswith((".cpp", ".cxx", ".cc"))
            ):
                text = self.src.typedef_text(_span(n))
                src_file = _file_of(n)
                entry = (self.rel(src_file), text) if text and src_file else None
                if entry and entry not in self.model.typedefs:
                    self.model.typedefs.append(entry)
            elif kind == "EnumDecl" and self.mine(_file_of(n)) and n.get("name"):
                self.enum(n)
            elif kind == "VarDecl" and record is None and self.mine(_file_of(n)):
                self.global_var(n)

    def class_template(self, n: dict, scope: str) -> None:
        """A module class template: each instantiation clang made is a class of its own.

        The pattern itself is never a class -- its layout depends on its arguments
        (`CUiWindow<T> : .., public T`). An instantiation has the arguments substituted in its
        bases, fields and signatures, and carries the bodies this TU USES (another TU may
        instantiate more of them: _register attaches those)."""
        pattern = next((c for c in n.get("inner", []) if c.get("kind") == "CXXRecordDecl"), None)
        if pattern is None or not (self.mine(_file_of(pattern)) or self.external_class(pattern)):
            return
        defaults = _template_defaults(n)
        for spec in n.get("inner", []):
            if spec.get("kind") != "ClassTemplateSpecializationDecl" or not spec.get(
                "completeDefinition"
            ):
                continue
            args = _template_args(spec)
            # An argument equal to its default is bound by the default's SPELLING: clang
            # gives `CToolbar<>`'s argument desugared (`unsigned long`, a width that differs
            # per arch), the header says `DWORD_PTR`.
            for i, a in enumerate(args):
                d = defaults[i] if i < len(defaults) else None
                if d is not None and instance_key(a) in {instance_key(x) for x in d}:
                    args[i] = d[0]
            written = list(args)
            while written and len(written) <= len(defaults):
                d = defaults[len(written) - 1]
                if d is None or written[-1] != d[0]:
                    break
                written.pop()
            name = instance_name(n["name"], written)
            for k in range(len(written), len(args) + 1):
                for form in (args[:k], [self.rewrite(a) for a in args[:k]]):
                    self.model.instances[instance_key(f"{n['name']}<{', '.join(form)}>")] = name
            self.record(spec, scope, None, name=name, template=n["name"])
            params = _template_param_names(n)
            if len(params) != len(args):
                raise CppModelError(f"template {n['name']}: {len(params)} parameters, {args}")
            self.model.classes[name].template_args = list(zip(params, args, strict=True))

    def instance_sugar(self, nodes: list) -> None:
        """Typedef spellings of an instantiation: clang names `CLocalPtr<WCHAR>`'s
        specialization by its desugared argument (`CLocalPtr<wchar_t>`), while the module's
        text says WCHAR. Every type in the module's own declarations that desugars to a known
        instantiation teaches its written spelling."""
        if not self.model.instances:
            return
        stack = [n for n in nodes if self.mine(_file_of(n))]
        while stack:
            n = stack.pop()
            t = n.get("type")
            if isinstance(t, dict) and "<" in t.get("desugaredQualType", ""):
                for written, sugar in _instance_spellings(t["qualType"], t["desugaredQualType"]):
                    hit = self.model.instances.get(instance_key(sugar))
                    if hit is not None:
                        self.model.instances.setdefault(instance_key(written), hit)
            stack.extend(x for x in n.get("inner", []) if isinstance(x, dict))

    def function_template(self, n: dict) -> None:
        """A module function template (rapps' `template <class T> static CStringW
        BuildPath(const T &Base, LPCWSTR Append)`): each instantiation clang made is a free
        function of its own, `BuildPath__T0`, named in first-seen order and keyed by its
        type so every TU agrees, and calls reach it through the decl map as an overload's do."""
        pattern = next((c for c in n.get("inner", []) if c.get("kind") == "FunctionDecl"), None)
        if pattern is None or not self.mine(_file_of(pattern)):
            return
        for inst in n.get("inner", []):
            if inst.get("kind") != "FunctionDecl" or not any(
                x.get("kind") == "TemplateArgument" for x in inst.get("inner", [])
            ):
                continue
            key = (n["name"], inst.get("type", {}).get("qualType", ""))
            rust = self._fn_instances.setdefault(
                key, f"{n['name']}__T{sum(1 for k in self._fn_instances if k[0] == n['name'])}"
            )
            self.decls[inst["id"]] = cpp_lower.DeclInfo("", "function", n["name"], rust)
            body = next((x for x in inst.get("inner", []) if x["kind"] == "CompoundStmt"), None)
            if body is None or any(f.name == rust for f in self.model.functions):
                continue
            whole = _span(inst)
            if whole is None:
                continue
            ff = FreeFunc(
                rust,
                _signature_from_ast(inst).replace(f" {n['name']}(", f" {rust}(", 1),
                self.rel(whole[0]),
                "",
                inst.get("storageClass") == "static",
            )
            self.model.functions.append(ff)
            self.pending.append((ff, body, None))

    def rewrite(self, text: str) -> str:
        return cpp_templates.rewrite_instances(text, self.model.instances, self.canon_key)

    def canon_key(self, key: str) -> str:
        """An instance key with each argument's typedef names resolved, as clang spells a
        specialization's arguments: `CLocalPtr<WCHAR>` -> `CLocalPtr<wchar_t>`."""
        if "<" not in key:
            return key
        head, args = key.split("<", 1)

        def resolve(m: re.Match) -> str:
            name, seen = m.group(0), set()
            while name in self.typedef_map and name not in seen:
                seen.add(name)
                name = self.typedef_map[name]
            return name

        for _ in range(4):  # typedef chains re-expose names: WCHAR -> wchar_t is one step
            args = re.sub(r"\b[A-Za-z_]\w*\b", resolve, args)
        return instance_key(f"{head}<{args}")

    def apply_instances(self) -> None:
        """Every spelled instantiation in the model -> its concrete class name. Run before a
        TU's bodies are lowered (base paths are found by class name) and after (the lowered
        text still spells `CUiWindow<CTreeView> *`)."""
        if not self.model.instances:
            return
        rw = self.rewrite
        for cls in self.model.classes.values():
            for b in cls.bases:
                b.name = rw(b.name)
            for f in cls.fields:
                f.c_type = rw(f.c_type)
            for m in cls.methods:
                m.return_type = rw(m.return_type)
                for p in m.params:
                    p.c_type = rw(p.c_type)
                m.inits = [(rw(a), rw(b)) for a, b in m.inits]
                m.lowered_inits = [(rw(a), rw(b)) for a, b in m.lowered_inits]
                if m.lowered:
                    m.lowered = rw(m.lowered)
                    if cls.template:
                        m.lowered = self._bind_template(cls, m.lowered)
        for g in self.model.globals:
            g.c_type = rw(g.c_type)
            g.init = rw(g.init) if g.init else g.init
        for fn in self.model.functions:
            fn.signature = rw(fn.signature)
            fn.lowered = rw(fn.lowered)
        self.model.typedefs = [(f, rw(t)) for f, t in self.model.typedefs]

    def _bind_template(self, cls: ClassModel, text: str) -> str:
        """An instantiation's body: the template's own names, bound. The injected class name
        (bare `CPointerArray`) is the instance; a parameter is its argument (`GrowthRate` ->
        `10`, `reinterpret_cast<T *>` -> `<CUiPrimitive *>`). A member reached through a
        dependent base, `T::m_hWnd`, was already given its path by the lowering
        (`T::this_->__base2.m_hWnd`): the qualifier is the dependent base's name for it."""
        text = re.sub(rf"\b{re.escape(cls.template or '')}\b(?!\s*<)", cls.name, text)
        for param, arg in cls.template_args:
            text = re.sub(rf"\b{re.escape(param)}::(?=this_->)", "", text)
            text = re.sub(rf"(?<![\w.>:]){re.escape(param)}\b", self.rewrite(arg), text)
        return text

    def record(
        self,
        n: dict,
        scope: str,
        outer: str | None,
        name: str | None = None,
        template: str | None = None,
    ) -> None:
        name = name or n.get("name") or ""
        if not name:
            return  # an anonymous struct/union member: its fields belong to the parent
        qualified = f"{outer}::{name}" if outer else f"{scope}{name}"
        self.by_id[n["id"]] = qualified
        if qualified in self.model.classes:
            # The same header class seen again from another TU: only its ids are new.
            self._register(n, self.model.classes[qualified], scope)
            return
        span = _span(n)
        cls = ClassModel(
            name=name,
            qualified=qualified,
            tag=n.get("tagUsed", "class"),
            file=self.rel(span[0]) if span else "",
            bases=[
                Base(
                    b["type"]["qualType"],
                    b.get("writtenAccess", b.get("access", "public")),
                    bool(b.get("isVirtual")),
                )
                for b in n.get("bases", [])
            ],
            polymorphic=bool(n.get("definitionData", {}).get("isPolymorphic")),
            nested_in=outer,
            template=template,
        )
        text = self.src.text(span) or ""
        cls.msg_map = parse_msg_map(text)
        cls.com_map = parse_com_map(text)
        cls.external = self.external_class(n)
        wc = re.search(r"\bDECLARE_WND_CLASS(_EX)?\s*\(((?:[^()]|\([^()]*\))*)\)", text)
        if wc:
            cls.wnd_class = [a.strip() for a in _split_top(wc.group(2))]
            if self.unicode:  # rapps' DECLARE_WND_CLASS_EX(_T("SplitterWindowClass"), ..)
                cls.wnd_class = [cpp_lower.unicode_text_macros(a) for a in cls.wnd_class]
        access = "private" if cls.tag == "class" else "public"
        self.model.classes[qualified] = cls
        for c in n.get("inner", []):
            k = c.get("kind")
            if k == "AccessSpecDecl":
                access = c["access"]
            elif k == "FieldDecl" and not c.get("name"):
                anon = self._anon_record(n, c, cls)
                if anon is not None:
                    cls.fields.append(Field(_anon_field(anon), anon.name, access, False, None))
            elif k == "FieldDecl":
                init = None
                if c.get("hasInClassInitializer") and c.get("inner"):
                    init = self.src.node_text(c["inner"][-1])
                cls.fields.append(Field(c["name"], c["type"]["qualType"], access, False, init))
            elif k == "EnumDecl" and not c.get("name"):
                for e in c.get("inner", []):
                    v = _const_value(e) if e.get("kind") == "EnumConstantDecl" else None
                    if v is not None:
                        cls.constants.append((e["name"], v))
            elif k == "VarDecl":  # a static data member
                cls.fields.append(Field(c["name"], c["type"]["qualType"], access, True, None))
            elif k in _METHOD_KINDS and not c.get("isImplicit"):
                m = self.method(c, access, external=cls.external)
                if m is not None:
                    m.inline = m.body is not None
                    cls.methods.append(m)
            elif k == "CXXRecordDecl" and c.get("completeDefinition") and not c.get("isImplicit"):
                self.record(c, scope, qualified)
        _assign_names(cls)
        self._register(n, cls, scope, first=True)
        if cls.com_map is not None:
            _add_com_object_unknown(cls)

    def _anon_record(self, n: dict, fdecl: dict, cls: ClassModel) -> ClassModel | None:
        """An anonymous `union { UINT m_uFlags; FN_LAYOUTCHANGE m_fnLayoutChange; };` member
        (msctfime's CTextEventSink) as a generated `{Class}__anonK` union, embedded where C++
        puts it. Skipping it would shift every later field. Its members are reached as
        `this->m_uFlags` in C++, so _register maps them to `__anonK.m_uFlags`."""
        rec = next(
            (
                r
                for r in n.get("inner", [])
                if r.get("kind") == "CXXRecordDecl"
                and not r.get("name")
                and r.get("completeDefinition")
                and f"anonymous {r.get('tagUsed', '?')}" in fdecl["type"]["qualType"]
                and r["id"] not in self._anon_seen
            ),
            None,
        )
        if rec is None:
            raise CppModelError(
                f"{cls.qualified}: an unnamed field whose record cannot be found "
                f"({fdecl['type']['qualType']}); its layout cannot be skipped"
            )
        self._anon_seen.add(rec["id"])
        k = sum(1 for f in cls.fields if f.name.startswith("__anon"))
        anon = ClassModel(
            name=f"{cls.name}__anon{k}",
            qualified=f"{cls.qualified}__anon{k}",
            tag=rec.get("tagUsed", "union"),
            file=cls.file,
        )
        for f in rec.get("inner", []):
            if f.get("kind") == "FieldDecl":
                if not f.get("name"):
                    raise CppModelError(f"{cls.qualified}: nested anonymous member")
                anon.fields.append(Field(f["name"], f["type"]["qualType"]))
        self.model.classes[anon.qualified] = anon
        return anon

    def _register(self, n: dict, cls: ClassModel, scope: str, first: bool = False) -> None:
        """Map this AST's member decl ids to the class model, and queue inline bodies."""
        methods = iter(cls.methods)
        fields = {f.name for f in cls.fields}
        anon_fields = iter(f for f in cls.fields if f.name.startswith("__anon"))
        for c in n.get("inner", []):
            k = c.get("kind")
            if k == "FunctionTemplateDecl":
                # A static member TEMPLATE (atlimagedx.h's get_fn<FN_T>): never generated --
                # it is hand-ported once -- but a bare call inside the class still means
                # `Class::get_fn(..)`. Calls reference the INSTANTIATION's decl id, and clang
                # lists each instantiation under the template, so all of them are mapped.
                for d in c.get("inner", []):
                    if d.get("kind") == "CXXMethodDecl" and d.get("storageClass") == "static":
                        self.decls[d["id"]] = cpp_lower.DeclInfo(
                            cls.qualified, "method", d["name"], d["name"], static=True
                        )
                        if first and not any(m.name == d["name"] for m in cls.methods):
                            # Known to the model (so integrate can qualify a bare call), but
                            # body-less: nothing is generated or translated for a template.
                            cls.methods.append(Method(d["name"], "method", [], "void", static=True))
                            cls.methods[-1].rust_name = d["name"]
                continue
            if k == "EnumDecl" and c.get("name"):
                # A NAMED enum nested in the class: bindgen spells its constants
                # Outer_Enum_CONST (`RegistrySettings_WallpaperStyle_TILED`), and C++ writes
                # them `RegistrySettings::TILED` or, inside the class, bare `TILED`.
                prefix = cls.qualified.replace("::", "_") + "_" + c["name"]
                for e in c.get("inner", []):
                    if e.get("kind") == "EnumConstantDecl":
                        self.decls[e["id"]] = cpp_lower.DeclInfo(
                            cls.qualified, "enumconst", e["name"], f"{prefix}_{e['name']}"
                        )
                continue
            if k == "FieldDecl" and not c.get("name"):
                af = next(anon_fields, None)
                rec = next(
                    (
                        r
                        for r in n.get("inner", [])
                        if r.get("kind") == "CXXRecordDecl"
                        and not r.get("name")
                        and r.get("completeDefinition")
                        and r.get("id") not in self._anon_registered
                    ),
                    None,
                )
                if af is None or rec is None:
                    continue
                self._anon_registered.add(rec["id"])
                for f in rec.get("inner", []):
                    if f.get("kind") == "FieldDecl" and f.get("name"):
                        self.decls[f["id"]] = cpp_lower.DeclInfo(
                            cls.qualified, "field", f["name"], f"{af.name}.{f['name']}"
                        )
                continue
            if k in ("FieldDecl", "VarDecl") and c.get("name") in fields:
                self.decls[c["id"]] = cpp_lower.DeclInfo(
                    cls.qualified, "field", c["name"], c["name"], static=k == "VarDecl"
                )
            elif k in _METHOD_KINDS and not c.get("isImplicit"):
                if self.method(c, "public", probe=True, external=cls.external) is None:
                    continue
                m = next(methods)
                self.method_by_id[c["id"]] = m
                self.decls[c["id"]] = cpp_lower.DeclInfo(
                    cls.qualified,
                    m.kind,
                    m.name,
                    m.rust_name,
                    [p.default for p in m.params],
                    m.static,
                )
                if first and m.body is not None:
                    self._queue(m, c, cls.qualified)
                elif not first and cls.template and m.body is None:
                    # An instantiation's member is instantiated where it is USED: this TU may
                    # have the body an earlier one did not.
                    late = self.method(c, m.access)
                    if late is not None and late.body is not None:
                        m.body, m.inits, m.inline = late.body, late.inits, True
                        self._queue(m, c, cls.qualified)
            elif (
                k == "CXXRecordDecl"
                and c.get("completeDefinition")
                and not c.get("isImplicit")
                and c.get("name")
            ):
                self.by_id[c["id"]] = f"{cls.qualified}::{c['name']}"

    def _queue(self, m: Method, node: dict, owner: str) -> None:
        for x in node.get("inner", []):
            if x["kind"] == "CompoundStmt":
                self.pending.append((m, x, owner))
            elif x["kind"] == "CXXCtorInitializer" and x.get("inner"):
                target = x.get("anyInit", {}).get("name") or x.get("baseInit", {}).get(
                    "qualType", "?"
                )
                if not x["inner"][0].get("isImplicit"):
                    self.pending_inits.append((m, target, x["inner"][0], owner))

    def method(
        self, c: dict, access: str, probe: bool = False, external: bool = False
    ) -> Method | None:
        # A macro-generated method (BEGIN_MSG_MAP's ProcessWindowMessage) is rendered from
        # the map, never translated as text. Not for an EXTERNAL class: CWindow's
        # `SendMessage` is spelled through winuser.h's TCHAR macro, and that filter dropped
        # SendMessage, MessageBox and every other TCHAR-named member mspaint calls.
        # Macro-GENERATED means the whole declaration came from the macro: its name AND its
        # first token are spelled outside the module. A TCHAR-renamed method,
        # `BOOL GetOpenFileName(..)` in mspaint's main.h, has only its NAME spelled in
        # commdlg.h (`#define GetOpenFileName GetOpenFileNameW`): the name-only test dropped
        # CMainWindow's GetOpenFileName/GetSaveFileName/ChooseColor entirely. STDMETHOD(Foo)
        # is the converse (begin in objbase.h, name the module's own) and is kept too.
        begin = (c.get("range") or {}).get("begin", {})
        if (
            not external
            and "spellingLoc" in c.get("loc", {})
            and not self.mine(c["loc"]["spellingLoc"].get("_file"))
            and "spellingLoc" in begin
            and not self.mine(begin["spellingLoc"].get("_file"))
            # ...unless the name is a macro ARGUMENT, i.e. the module's own method renamed by
            # a TCHAR macro inside STDMETHOD_: cicero's `STDMETHOD_(LPCTSTR, GetClassName)()`
            # begins in objbase.h AND spells `GetClassNameW` in winuser.h, and dropping it left
            # CUIFWindow's vtable a slot short of the C++ one.
            and not c["loc"].get("expansionLoc", {}).get("isMacroArgExpansion")
        ):
            return None
        if probe:
            return Method(c["name"], _method_kind(c), [], "void")
        qt = c["type"]["qualType"]
        params = [
            Param(
                p.get("name", f"arg{i}"),
                p["type"]["qualType"],
                self.src.node_text(p["inner"][0]) if p.get("init") and p.get("inner") else None,
            )
            for i, p in enumerate(x for x in c.get("inner", []) if x["kind"] == "ParmVarDecl")
        ]
        body = None
        inits: list[tuple[str, str]] = []
        for x in c.get("inner", []):
            if x["kind"] == "CompoundStmt":
                span = _span(x)
                body = (self.rel(span[0]), span[1], span[2]) if span else None
            elif x["kind"] == "CXXCtorInitializer":
                target = x.get("anyInit", {}).get("name") or x.get("baseInit", {}).get(
                    "qualType", "?"
                )
                if x.get("inner") and not x["inner"][0].get("isImplicit"):
                    text = _ctor_args(self.src.node_text(x["inner"][0]), (target, c["name"]))
                    if text is not None:
                        inits.append((target, text))
        return Method(
            name=c["name"],
            kind=_method_kind(c),
            params=params,
            return_type=_return_type(qt) if c["kind"] == "CXXMethodDecl" else "void",
            virtual=bool(c.get("virtual")),
            pure=bool(c.get("pure")),
            static=c.get("storageClass") == "static",
            const=_is_const_method(qt),
            access=access,
            body=body,
            inits=inits,
            abi=self._abi(c, qt),
        )

    _STDCALL_WORDS = re.compile(r"\b(?:CALLBACK|WINAPI|APIENTRY|__stdcall|PASCAL|NTAPI)\b")

    def _abi(self, c: dict, qt: str) -> str:
        """`system` when the member is stdcall: clang spells it in the type on i386, and on
        64-bit (where it is ignored) the declaration still SAYS so -- the same Rust either
        way, `extern "system"` being stdcall on i386 and the C ABI elsewhere."""
        if "stdcall" in qt:
            return "system"
        text = self.src.node_text(c) or ""
        head = text.split(c.get("name", "\0"), 1)[0]
        return "system" if self._STDCALL_WORDS.search(head) else ""

    def out_of_line(self, n: dict) -> None:
        """`void CCanvasWindow::OnPaint(...) { ... }` in a .cpp: attach the body."""
        if not self.mine(_file_of(n)) or not any(
            x["kind"] == "CompoundStmt" for x in n.get("inner", [])
        ):
            return
        owner = self.by_id.get(n.get("parentDeclContextId", ""))
        if owner is None:
            return  # a method of a class this module does not define (an ATL template)
        cls = self.model.classes[owner]
        defined = self.method(n, "public")
        if defined is None:
            return
        # clang links a definition to its in-class declaration (`previousDecl`); the
        # written types may differ in qualification (`RegistrySettings::WallpaperStyle`).
        prev = self.method_by_id.get(n.get("previousDecl", ""))
        for m in ([prev] if prev is not None else []) + cls.methods:
            if m is prev or (_sig(m) == _sig(defined) and m.kind == defined.kind):
                m.body, m.inits = defined.body, defined.inits or m.inits
                # A definition's parameter NAMES are the ones its body uses; the defaults
                # are written only on the declaration.
                for p, d in zip(m.params, defined.params, strict=False):
                    p.name = d.name
                # A call AFTER this definition names the definition's own decl id (clang
                # refers to the most recent redeclaration), so it resolves too.
                self.decls[n["id"]] = cpp_lower.DeclInfo(
                    owner, m.kind, m.name, m.rust_name, [p.default for p in m.params], m.static
                )
                self._queue(m, n, owner)
                return
        raise CppModelError(
            f"{owner}::{defined.signature_key} is defined in {defined.body and defined.body[0]} "
            "but matches no declaration in the class"
        )

    def enum(self, n: dict) -> None:
        """A module enum. Namespaced ones (`snd::audio_wavein_status`) never reach the
        bindgen island, so the class renderer needs them from here."""
        if any(e.name == n["name"] for e in self.model.enums):
            return
        members: list[tuple[str, int]] = []
        nxt = 0
        for c in n.get("inner", []):
            if c.get("kind") != "EnumConstantDecl":
                continue
            value = _const_value(c)
            nxt = nxt if value is None else value
            members.append((c["name"], nxt))
            nxt += 1
        self.model.enums.append(
            EnumModel(
                n["name"],
                n.get("fixedUnderlyingType", {}).get("qualType", "int"),
                members,
                self.rel(_file_of(n) or ""),
            )
        )

    def scan_overloads(self, nodes: list) -> None:
        """Every module free function's distinct signatures, in first-seen order: drawing.h
        declares `Fill(.., COLORREF)` AND `Fill(.., HBRUSH)`, and keeping one name dropped
        the other overload, so every call went to whichever came first."""
        for n in nodes:
            if n.get("kind") == "FunctionDecl" and self.mine(_file_of(n)):
                sigs = self._overloads.setdefault(n["name"], [])
                qt = n.get("type", {}).get("qualType", "")
                if qt not in sigs:
                    sigs.append(qt)
            elif n.get("kind") in ("NamespaceDecl", "LinkageSpecDecl"):
                self.scan_overloads(n.get("inner", []))

    def fn_rust_name(self, n: dict) -> str:
        """`Fill` alone, or `Fill__0`/`Fill__1` when the module overloads it. An operator is
        `cxx_operator_new[_array]`: `operator new` is no name a C function can have, and
        translate could not extract the body (cicbase.cpp went unbought three times)."""
        sigs = self._overloads.get(n["name"], [])
        base = n["name"]
        if base.startswith("operator "):
            base = "cxx_" + base.replace("operator ", "operator_").replace("[]", "_array")
        if len(sigs) < 2:
            return base
        qt = n.get("type", {}).get("qualType", "")
        return f"{base}__{sigs.index(qt) if qt in sigs else len(sigs)}"

    def _register_function(self, n: dict) -> None:
        """A module free function's declaration (a prototype in dib.h counts), for its
        default arguments: `CopyDIBImage(hbm, cx, cy)` must be called with dib.h's
        `stretchMode = STRETCH_HALFTONE` written out, as for a member call."""
        params = [x for x in n.get("inner", []) if x.get("kind") == "ParmVarDecl"]
        defaults = [
            self.src.node_text(p["inner"][0]) if p.get("init") and p.get("inner") else None
            for p in params
        ]
        # A redeclaration (the definition in dib.cpp) repeats no defaults, but a call may
        # reference it: it carries the prototype's.
        rust = self.fn_rust_name(n)
        key = rust  # an overload's defaults are its own
        if any(d is not None for d in defaults):
            self._fn_defaults[key] = defaults
        known = self._fn_defaults.get(key)
        if known is not None and len(known) == len(defaults):
            self.decls[n["id"]] = cpp_lower.DeclInfo("", "function", n["name"], rust, known)
        elif rust != n["name"]:
            self.decls[n["id"]] = cpp_lower.DeclInfo("", "function", n["name"], rust)

    def free_function(self, n: dict) -> None:
        body = next((x for x in n.get("inner", []) if x["kind"] == "CompoundStmt"), None)
        rust = self.fn_rust_name(n)
        if body is None or any(f.name == rust for f in self.model.functions):
            return
        whole, bspan = _span(n), _span(body)
        if whole is None or bspan is None:
            return
        sig = self.src.text((whole[0], whole[1], bspan[1])) or ""
        # The signature is joined onto one line below: a comment between the declarator and
        # the brace (asyncinet.cpp's `BOOL AsyncInetCleanUp(..) // close all handle..`) would
        # swallow everything after it, the `{` included.
        sig = re.sub(r"//[^\n]*|/\*.*?\*/", " ", sig, flags=re.S)
        if not re.search(rf"\b{re.escape(n['name'])}\s*\(", sig):
            # The declarator is a macro's (cabinet.cpp's `FNCLOSE(fnFileClose)`, fdi.h): the
            # text names no function, so the worker was asked for `FNCLOSE` and the planner
            # missed the `fnFileClose` it correctly wrote. Spell it from clang's type instead.
            sig = _signature_from_ast(n)
        if rust != n["name"]:
            sig = re.sub(rf"\b{re.escape(n['name'])}(?=\s*\()", rust, sig, count=1)
        ff = FreeFunc(
            rust,
            " ".join(sig.split()),
            self.rel(whole[0]),
            "",
            n.get("storageClass") == "static",
        )
        self.model.functions.append(ff)
        self.pending.append((ff, body, None))

    def base_path(self, frm: str, to: str, *, deep: bool = False) -> str:
        """`__baseK.` hops from class `frm` to its (transitive) base `to`, by short name.
        A base the module does not define (CWindowImpl<...>) is reached as the class's
        external base when `to` is not a module class at all.

        `deep` (a plain-struct field reached through a cast, `_plain_base_field`): the
        external base may sit behind module bases -- rapps' CUiMargin reaches RECT via
        CUiRect -- and clang names that cast `tagRECT` where the base list says `RECT`, so
        names compare through the typedef table. It is opt-in because the lowered text is
        the purchase cache key: widening every caller re-keyed 17 of rapps' paid units."""
        if frm == to:
            return ""
        by_short = {q.split("::")[-1]: c for q, c in self.model.classes.items()}
        start = by_short.get(frm)
        if start is None:
            return ""
        target = self._tag(to) if deep else to
        queue = [(start, "")]
        seen = set()
        external: list[str] | None = None  # the NEAREST class with outside bases decides
        while queue:
            cls, path = queue.pop(0)
            outside = []
            for i, b in enumerate(cls.bases):
                short = atl_base_alias(b.name)
                hop = f"{path}__base{i}."
                if short == to or (deep and self._tag(short) == target):
                    return hop
                nxt = by_short.get(short)
                if nxt is None:
                    outside.append(hop)
                elif short not in seen:
                    seen.add(short)
                    queue.append((nxt, hop))
            if external is None and (outside or not deep):
                external = outside  # not deep: only the starting class's own bases count
        if to not in by_short and external is not None and len(external) == 1:
            return external[0]
        return ""

    def _tag(self, name: str) -> str:
        """A type name with its typedef chain resolved: `RECT` -> `tagRECT`."""
        seen = set()
        while name in self.typedef_map and name not in seen:
            seen.add(name)
            name = re.sub(r"^(?:struct|class|union)\s+", "", self.typedef_map[name])
        return name

    def lower_pending(self) -> None:
        low = cpp_lower.Lowerer(
            self.decls, self.model.classes, self.base_path, self.model.instances, self.canon_key
        )
        own = sorted(
            f
            for pat in ("*.h", "*.hpp", "*.cpp")
            for f in Path(self.module_dir).rglob(pat)
            if f.is_file()
        )
        strings = cpp_lower.string_macros([f.read_text(errors="replace") for f in own])
        for target, node, owner in self.pending:
            span = self.src.node_span(node)
            if span is None:
                continue
            out = low.lower(node, span, owner)
            text = cpp_lower.concat_string_macros(out.text, strings)
            text = cpp_lower.ppv_arg_macros(text)
            text = cpp_lower.atl_qualified_glue(text)
            target.lowered = cpp_lower.unicode_text_macros(text) if self.unicode else text
            target.unplaced = out.unplaced
            target.calls = [(a, b) for a, b in out.calls]
        for m, member, node, owner in self.pending_inits:
            span = self.src.node_span(node)
            if span is None:
                continue
            out = low.lower(node, span, owner)
            # `calls` round-trips through JSON as lists: back to (class, method) pairs.
            m.calls = sorted({*((a, b) for a, b in m.calls), *out.calls})
            text = _ctor_args(out.text, (member, owner.split("::")[-1]))
            if text is not None:
                m.lowered_inits.append((member, text))
        for g in self.model.globals:
            # A string initializer gets what translation phases 2, 4 and 6 do to it: line
            # splices joined, TEXT() widened, a module string macro beside a literal folded
            # (rapps' `L"RAPPS [/" CMD_KEY_HELP L"] ..."` usage string).
            if g.init and '"' in g.init:
                text = cpp_lower.concat_string_macros(g.init.replace("\\\n", ""), strings)
                g.init = cpp_lower.unicode_text_macros(text) if self.unicode else text
        self.pending.clear()
        self.pending_inits.clear()

    def global_var(self, n: dict) -> None:
        if n.get("storageClass") == "extern" and not n.get("inner"):
            return  # a declaration; the definition is elsewhere in the module
        name = n["name"]
        init = None
        if n.get("inner"):
            init = _ctor_args(self.src.node_text(n["inner"][0]), (name,))
        if any(g.name == name for g in self.model.globals):
            return
        self.model.globals.append(
            GlobalVar(
                name,
                n["type"]["qualType"],
                self.rel(_file_of(n) or ""),
                init,
                n.get("storageClass") == "static",
            )
        )


def _cache_key(argv: list[str], tu: Path) -> str:
    h = hashlib.sha256(" ".join(argv).encode())
    h.update(tu.read_bytes())
    for f in sorted(tu.parent.glob("*")):
        if f.suffix.lower() in (".h", ".hpp", ".inl"):
            h.update(f.name.encode() + f.read_bytes())
    return h.hexdigest()[:20]


def extract(
    repo: Path, module_dir: Path, tus: list[Path], arch: str = "i386", cache_dir: Path | None = None
) -> ModuleModel:
    """The module's class model, merged across its TUs (headers are seen once per TU)."""
    ex = _Extractor(repo, module_dir)
    for tu in tus:
        argv = cdp.ma.compile_argv(repo, arch, tu)
        if argv is None:
            raise CppModelError(
                f"no compile command for {tu}\nfix: ./vibeos-cli build --arch {arch}"
            )
        ast = cdp.clang_ast(repo, arch, tu)
        ex.unicode = any(a in ("-DUNICODE", "-D_UNICODE") for a in argv)
        # ids are per-AST; record() re-registers them on each TU
        ex.by_id, ex.decls, ex.method_by_id = {}, {}, {}
        ex._anon_seen, ex._anon_registered = set(), set()
        ex.scan_overloads(ast.get("inner", []))
        ex.walk(ast.get("inner", []))
        ex.instance_sugar(ast.get("inner", []))
        ex.apply_instances()
        ex.lower_pending()
        ex.apply_instances()
        del ast
    if cache_dir is not None:
        cache_dir.mkdir(parents=True, exist_ok=True)
        (cache_dir / "cpp_model.json").write_text(ex.model.to_json())
    return ex.model
