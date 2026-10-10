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
from . import cpp_lower


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
            if c["msg_map"] is not None:
                c["msg_map"] = [MsgMapEntry(**e) for e in c["msg_map"]]
            out.classes[name] = ClassModel(**c)
        out.globals = [GlobalVar(**g) for g in raw["globals"]]
        out.functions = [FreeFunc(**f) for f in raw.get("functions", [])]
        out.enums = [
            EnumModel(e["name"], e["underlying"], [tuple(m) for m in e["members"]], e["file"])
            for e in raw.get("enums", [])
        ]
        out.allocators = raw.get("allocators", {})
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
}
# ATL's window templates are, for layout and member lookup, their CWindow: the template's own
# machinery (thunk, Create, DefWindowProc, message routing) is atl_rust::win plus the per-class
# glue cpp_app generates, never a translated body.
ATL_BASE_ALIASES = {
    "CWindowImpl": "CWindow",
    "CDialogImpl": "CWindow",
    "CWindowImplBaseT": "CWindow",
    "CDialogImplBaseT": "CWindow",
}
# Their methods a subclass calls through an implicit `this`: they stay on the object, where the
# generated glue answers them, instead of resolving to CWindow (whose `Create` takes a class
# name -- a different function).
ATL_IMPL_METHODS = cpp_lower.ATL_IMPL_METHODS


def short_name(name: str) -> str:
    return name.split("<")[0].split("::")[-1].strip()


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
            m.rust_name = _OPERATOR_NAMES.get(m.name[len("operator") :].strip(), "op_unknown")
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


_METHOD_KINDS = ("CXXMethodDecl", "CXXConstructorDecl", "CXXDestructorDecl")


class _Extractor:
    def __init__(self, repo: Path, module_dir: Path) -> None:
        self.repo = repo
        self.module_dir = str(module_dir.resolve())
        self.src = _Sources()
        self.model = ModuleModel()
        # Per TU: decl ids mean nothing outside the AST that produced them.
        self.by_id: dict[str, str] = {}  # record decl id -> qualified class name
        self.decls: dict[str, cpp_lower.DeclInfo] = {}
        self._anon_seen: set[str] = set()
        self._anon_registered: set[str] = set()
        self.method_by_id: dict[str, Method] = {}
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
            if kind in ("NamespaceDecl", "LinkageSpecDecl"):
                inner_scope = f"{scope}{n['name']}::" if n.get("name") else scope
                self.walk(n.get("inner", []), inner_scope)
            elif kind == "CXXRecordDecl" and n.get("completeDefinition"):
                if not n.get("isImplicit") and (self.mine(_file_of(n)) or self.external_class(n)):
                    self.record(n, scope, record)
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
            elif kind == "FunctionDecl" and record is None and self.mine(_file_of(n)):
                self.free_function(n)
            elif kind == "EnumDecl" and self.mine(_file_of(n)) and n.get("name"):
                self.enum(n)
            elif kind == "VarDecl" and record is None and self.mine(_file_of(n)):
                self.global_var(n)

    def record(self, n: dict, scope: str, outer: str | None) -> None:
        name = n.get("name") or ""
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
        )
        text = self.src.text(span) or ""
        cls.msg_map = parse_msg_map(text)
        cls.external = self.external_class(n)
        wc = re.search(r"\bDECLARE_WND_CLASS(_EX)?\s*\(((?:[^()]|\([^()]*\))*)\)", text)
        if wc:
            cls.wnd_class = [a.strip() for a in _split_top(wc.group(2))]
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
                    init = self.src.text(_span(c["inner"][-1]))
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
                    cls.qualified, "field", c["name"], c["name"]
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
        if (
            not external
            and "spellingLoc" in c.get("loc", {})
            and not self.mine(c["loc"]["spellingLoc"].get("_file"))
        ):
            return None
        if probe:
            return Method(c["name"], _method_kind(c), [], "void")
        qt = c["type"]["qualType"]
        params = [
            Param(
                p.get("name", f"arg{i}"),
                p["type"]["qualType"],
                self.src.text(_span(p["inner"][0])) if p.get("init") and p.get("inner") else None,
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
                    text = _ctor_args(self.src.text(_span(x["inner"][0])), (target, c["name"]))
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
        )

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

    def free_function(self, n: dict) -> None:
        body = next((x for x in n.get("inner", []) if x["kind"] == "CompoundStmt"), None)
        if body is None or any(f.name == n["name"] for f in self.model.functions):
            return
        whole, bspan = _span(n), _span(body)
        if whole is None or bspan is None:
            return
        sig = self.src.text((whole[0], whole[1], bspan[1])) or ""
        ff = FreeFunc(
            n["name"],
            " ".join(sig.split()),
            self.rel(whole[0]),
            "",
            n.get("storageClass") == "static",
        )
        self.model.functions.append(ff)
        self.pending.append((ff, body, None))

    def base_path(self, frm: str, to: str) -> str:
        """`__baseK.` hops from class `frm` to its (transitive) base `to`, by short name.
        A base the module does not define (CWindowImpl<...>) is reached as the class's
        external base when `to` is not a module class at all."""
        if frm == to:
            return ""
        by_short = {q.split("::")[-1]: c for q, c in self.model.classes.items()}
        start = by_short.get(frm)
        if start is None:
            return ""
        queue = [(start, "")]
        seen = set()
        while queue:
            cls, path = queue.pop(0)
            for i, b in enumerate(cls.bases):
                short = b.name.split("<")[0].split("::")[-1].strip()
                short = ATL_BASE_ALIASES.get(short, short)
                hop = f"{path}__base{i}."
                if short == to:
                    return hop
                nxt = by_short.get(short)
                if nxt is not None and short not in seen:
                    seen.add(short)
                    queue.append((nxt, hop))
        if to not in by_short:
            external = [
                i
                for i, b in enumerate(start.bases)
                if ATL_BASE_ALIASES.get(short_name(b.name), short_name(b.name)) not in by_short
            ]
            if len(external) == 1:
                return f"__base{external[0]}."
        return ""

    def lower_pending(self) -> None:
        low = cpp_lower.Lowerer(self.decls, self.model.classes, self.base_path)
        for target, node, owner in self.pending:
            span = _span(node)
            if span is None:
                continue
            out = low.lower(node, span, owner)
            target.lowered = out.text
            target.unplaced = out.unplaced
        for m, member, node, owner in self.pending_inits:
            span = _span(node)
            if span is None:
                continue
            out = low.lower(node, span, owner)
            text = _ctor_args(out.text, (member, owner.split("::")[-1]))
            if text is not None:
                m.lowered_inits.append((member, text))
        self.pending.clear()
        self.pending_inits.clear()

    def global_var(self, n: dict) -> None:
        if n.get("storageClass") == "extern" and not n.get("inner"):
            return  # a declaration; the definition is elsewhere in the module
        name = n["name"]
        init = None
        if n.get("inner"):
            init = _ctor_args(self.src.text(_span(n["inner"][0])), (name,))
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
        # ids are per-AST; record() re-registers them on each TU
        ex.by_id, ex.decls, ex.method_by_id = {}, {}, {}
        ex._anon_seen, ex._anon_registered = set(), set()
        ex.walk(ast.get("inner", []))
        ex.lower_pending()
        del ast
    if cache_dir is not None:
        cache_dir.mkdir(parents=True, exist_ok=True)
        (cache_dir / "cpp_model.json").write_text(ex.model.to_json())
    return ex.model
