"""Lower a C++ method or function body to the C-like text a worker translates.

The model is good at C and unreliable at the parts of C++ that are invisible in the text:
which member an unqualified `bits_psample` names, which base class it lives in, which of two
`alloc_seconds` overloads a call picks, which default argument it silently passes, what
`new audio_membuffer(fmt)` constructs and what `delete p` destroys. clang has already
resolved every one of those, so they are rewritten here, from the AST, into explicit text:

    bytes_played_ = 0;                 ->  this_->__base1.bytes_played_ = 0;
    alloc_seconds(secs);   // (float)  ->  this_->alloc_seconds__1(secs);
    buffer_secs();         // default  ->  this_->buffer_secs__0();
    new audio_waveout(fmt, *buf)       ->  audio_waveout__new_0(fmt, *buf)
    delete AUD_OUT                     ->  audio_waveout__delete(AUD_OUT)
    this                               ->  this_

Every rewrite is an EDIT (offset, length, replacement) against the body's own source text,
located from the AST node's own token, so the body keeps its comments, macros and layout.
A node whose token sits inside a macro expansion has no single place in the text; such an
edit is skipped and reported, never guessed (`Lowered.unplaced`).

The names this produces (`__baseK`, `<method>__N`, `<Class>__new_N`, `<Class>__delete`,
`<Class>__make_N`) are the ones `cpp_app` renders, and nothing else knows them.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field

from . import c_data_port as cdp
from . import cpp_templates

# ATL template methods a CWindowImpl/CDialogImpl subclass calls through an implicit `this`.
# cpp_model re-exports this one set (it imports this module, so it cannot own it).
ATL_IMPL_METHODS = frozenset(
    {
        "Create",
        "DefWindowProc",
        "SubclassWindow",
        "UnsubclassWindow",
        "DoModal",
        "EndDialog",
        "SetModified",
        "WinMain",
        "RegisterClassObjects",
        "RevokeClassObjects",
        "SetWizardButtons",
        "GetWndClassInfo",
        "GetWindowProc",
    }
)


# atlwin.h's defaults for the methods _render_atl_glue implements, by (name, full arity).
# CWindowImpl::Create's 7th parameter (lpCreateParam) has no glue slot: its default is None,
# so a call that relies on it is left for the audit rather than silently mis-arity'd.
_ATL_GLUE_DEFAULTS: dict[tuple[str, int], list[str | None]] = {
    ("Create", 7): [None, "NULL", "NULL", "0", "0", "0U", None],
    # CDialogImpl::Create/DoModal's dwInitParam is an LPARAM: `0`, not a pointer NULL.
    ("Create", 2): [None, "0"],
    ("DoModal", 2): ["GetActiveWindow()", "0"],
}


_TEXT_MACRO = re.compile(
    r"\b(?:TEXT|_TEXT|__TEXT|_T)\(\s*(\"(?:\\.|[^\"\\])*\"|'(?:\\.|[^'\\])*')\s*\)"
)


_ATL_QUALIFIED_GLUE = re.compile(r"\b(CWindowImpl|CDialogImpl)(?:<[^;{}()]*?>)?::this_->(\w+)\s*\(")


_COM_OBJECT_CREATE = re.compile(r"\b_?CComObject\s*<\s*(\w+)\s*>\s*::\s*CreateInstance\s*\(")
_COM_OBJECT_TYPE = re.compile(r"\b_?CComObject\s*<\s*(\w+)\s*>")


def com_object_create(text: str) -> str:
    """`_CComObject<T>::CreateInstance(&p)` -> `ComObject_CreateInstance<T>(&p)`, and the
    `CComObject<T> *p` it fills -> `T *p`: atl_rust gives the class itself CComObject's
    QueryInterface/AddRef/Release, so the object IS a T. Left as written, ntobjshex's model
    invented a `_CComObject_CFolderViewCB` type and a bare `CreateInstance`."""
    text = _COM_OBJECT_CREATE.sub(lambda m: f"ComObject_CreateInstance<{m.group(1)}>(", text)
    return _COM_OBJECT_TYPE.sub(lambda m: m.group(1), text)


def atl_qualified_glue(text: str) -> str:
    """`CWindowImpl::Create(..)` -> `this_->CWindowImpl_Create(..)`. A class's own `Create`
    hides the ATL template's in C++ name lookup, and the qualified call is how rapps reaches it
    anyway; the generated glue answers it under that qualified name (cpp_app)."""
    return _ATL_QUALIFIED_GLUE.sub(lambda m: f"this_->{m.group(1)}_{m.group(2)}(", text)


def unicode_text_macros(text: str) -> str:
    """`TEXT("x")` / `_T('x')` -> `L"x"` / `L'x'`, what winnt.h/tchar.h expand them to in a
    -DUNICODE compile. Lowered bodies keep the source's macro spelling, and an ATL header's
    `TEXT(';')` reached the model as a call it cannot resolve."""
    return _TEXT_MACRO.sub(lambda m: "L" + m.group(1), text)


_STR_LIT = re.compile(r'(?:\bL)?"(?:[^"\\\n]|\\.)*"')
_STRING_DEFINE = re.compile(
    r'^[ \t]*#[ \t]*define[ \t]+(\w+)[ \t]+((?:L)?"(?:[^"\\\n]|\\.)*")[ \t]*$', re.M
)


def string_macros(sources: list[str]) -> dict[str, str]:
    """A module's own object-like macros whose whole expansion is one string literal
    (drwtsn32.h `#define NEWLINE "\\r\\n"`)."""
    out: dict[str, str] = {}
    for text in sources:
        for m in _STRING_DEFINE.finditer(text):
            out.setdefault(m.group(1), m.group(2))
    return out


def concat_string_macros(text: str, macros: dict[str, str]) -> str:
    """`"%5d: %ls" NEWLINE` -> `"%5d: %ls\\r\\n"`: what translation phases 4 and 6 do. Left as
    written, the model dropped the literal and kept the macro -- every drwtsn32 report line
    came back as `xfprintf(output, NEWLINE, ...)`, which compiles and prints nothing useful.
    Only a macro NEXT TO a string literal is folded; a standalone use stays a name (it
    translates fine, and leaving it keeps every such body's response-cache key unchanged)."""
    if not macros:
        return text
    names = "|".join(map(re.escape, sorted(macros, key=len, reverse=True)))
    lit = _STR_LIT.pattern
    unit = re.compile(rf"(?:{lit}|\b(?:{names})\b)(?:\s*(?:{lit}|\b(?:{names})\b))+")

    def fold(m: re.Match) -> str:
        parts: list[str] = [p.group(0) for p in re.finditer(rf"{lit}|\b(?:{names})\b", m.group(0))]
        if not any(p.endswith('"') for p in parts):
            return m.group(0)  # macro beside macro with no literal: not this hazard
        lits: list[str] = [macros.get(p, p) for p in parts]
        wide = any(x.startswith("L") for x in lits)
        bodies = [x[x.index('"') + 1 : -1] for x in lits]
        for left in bodies[:-1]:
            if re.search(r"\\(?:x[0-9A-Fa-f]*|[0-7]{1,2})$", left):
                return m.group(0)  # a hex/octal escape would absorb the next chars
        return ("L" if wide else "") + '"' + "".join(bodies) + '"'

    return unit.sub(fold, text)


_PPV_ARG = re.compile(r"\b(IID_PPV_ARG|IID_NULL_PPV_ARG)\s*\(")


_DEFINE_LINE = re.compile(r"^[ \t]*#[ \t]*define[ \t]+(\w+)(?![\w(])[ \t]*([^\n]*)$", re.M)
_COND_LINE = re.compile(r"^[ \t]*#[ \t]*(if|ifdef|ifndef|elif|else|endif)\b(.*)$")


def module_defines(sources: list[str]) -> dict[str, str]:
    """The object-like `#define NAME VALUE` a module's own sources make (value text, comments
    stripped; the first definition wins)."""
    out: dict[str, str] = {}
    for text in sources:
        for m in _DEFINE_LINE.finditer(text):
            out.setdefault(m.group(1), re.sub(r"//.*|/\*.*?\*/", "", m.group(2)).strip())
    return out


def _eval_condition(cond: str, defines: dict[str, str]) -> bool | None:
    """A `#if` condition over the module's own macros: True/False, or None when it names
    anything else (an SDK or arch macro), which is then left for the reader."""
    names = set(re.findall(r"\b[A-Za-z_]\w*\b", cond)) - {"defined"}
    if any(n not in defines for n in names):
        return None
    expr = re.sub(r"\bdefined\s*\(\s*(\w+)\s*\)|\bdefined\s+(\w+)", lambda m: "1", cond)

    def value(m: re.Match[str]) -> str:
        v = defines[m.group(0)]
        return v if re.fullmatch(r"\(?-?\d+[uUlL]*\)?", v or "x") else ("1" if v == "" else "@")

    expr = re.sub(r"\b[A-Za-z_]\w*\b", value, expr)
    if "@" in expr or not re.fullmatch(r"[\d\s()!&|<>=+\-*uUlL]*", expr):
        return None
    expr = re.sub(r"(\d+)[uUlL]+", r"\1", expr)
    expr = expr.replace("&&", " and ").replace("||", " or ")
    expr = re.sub(r"!(?!=)", " not ", expr)
    try:
        return bool(eval(expr, {"__builtins__": {}}, {}))  # digits and operators only
    except Exception:
        return None


def resolve_module_conditionals(text: str, defines: dict[str, str]) -> str:
    """A body's `#if USE_CUSTOM_CONMGR .. #else .. #endif` decided by the module's own
    macros before the model sees it. netshell's precomp.h sets USE_CUSTOM_CONMGR 1, and the
    model translated the `#else` arm: an unregistered CLSID, so the tray never started. A
    block whose condition names any macro the module does not define is left whole."""
    out: list[str] = []
    # Each frame: [decided?, taking current arm?, an arm already taken?]; an undecided
    # frame passes its lines (and its directives) through.
    stack: list[list[bool | None]] = []
    for line in text.splitlines(keepends=True):
        m = _COND_LINE.match(line)
        if m is None:
            if all((not f[0]) or f[1] for f in stack):
                out.append(line)
            continue
        kw, rest = m.group(1), m.group(2).strip()
        live = all((not f[0]) or f[1] for f in stack)
        if kw in ("if", "ifdef", "ifndef"):
            cond = rest if kw == "if" else (
                f"defined({rest})" if kw == "ifdef" else f"!defined({rest})"
            )
            if kw != "if" and rest not in defines:
                verdict = None  # an undefined name may be the SDK's
            else:
                verdict = _eval_condition(cond, defines) if live else None
            stack.append([verdict is not None, bool(verdict), bool(verdict)])
            if verdict is None:
                out.append(line)
        elif kw == "elif":
            f = stack[-1] if stack else None
            if f is None or not f[0]:
                out.append(line)
                continue
            v = _eval_condition(rest, defines)
            if v is None:
                return text  # mixed decidable/undecidable chain: leave the body whole
            f[1] = (not f[2]) and v
            f[2] = bool(f[2] or v)
        elif kw == "else":
            f = stack[-1] if stack else None
            if f is None or not f[0]:
                out.append(line)
                continue
            f[1] = not f[2]
            f[2] = True
        else:  # endif
            f = stack.pop() if stack else None
            if f is None or not f[0]:
                out.append(line)
    return "".join(out)


_LOCAL_ENUM = re.compile(r"\benum\s+(?:class\s+)?(\w+\s*)?\{([^{}]*)\}\s*;")


def local_enums_to_consts(text: str) -> str:
    """A body's own `enum { IDC_EXTRACT, IDC_EXTRACTALL };` as `const int` declarations.

    The island cannot carry a function-local enum, and the model given one wrote
    `let _ = (IDC_EXTRACT, IDC_EXTRACTALL);` with the names undefined, so every `case`
    on them became a catch-all binding (cabview's ItemMenuCallback). An implicit value is
    the previous member plus one, spelled so, which is what C++ means by it. A named enum
    keeps its name as an `int` typedef for the declarations that use it."""

    def lower(m: re.Match[str]) -> str:
        out = [f"typedef int {m.group(1).strip()};"] if m.group(1) else []
        prev = None
        for item in (i.strip() for i in m.group(2).split(",")):
            if not item:
                continue
            name, _, value = (p.strip() for p in item.partition("="))
            if not re.fullmatch(r"\w+", name):
                return m.group(0)  # not a plain enumerator list; leave it to the model
            value = value or ("0" if prev is None else f"{prev} + 1")
            out.append(f"const int {name} = {value};")
            prev = name
        return " ".join(out)

    return _LOCAL_ENUM.sub(lower, text)


def ppv_arg_macros(text: str) -> str:
    """shellutils.h `IID_PPV_ARG(Itype, ppType)` -> `&IID_Itype, (void**)(ppType)`, its C
    spelling (`IID_NULL_PPV_ARG` adds the NULL outer unknown). It expands to TWO arguments, so
    no Rust item can stand in for it at the call site: the model wrote `IID_PPV_ARG(T, p)` as
    a call and every CoCreateInstance came out one argument short."""
    out, pos = [], 0
    for m in _PPV_ARG.finditer(text):
        if m.start() < pos:
            continue
        depth, i = 1, m.end()
        while i < len(text) and depth:
            depth += {"(": 1, ")": -1}.get(text[i], 0)
            i += 1
        if depth:
            break  # unbalanced: leave the rest as written
        inner = text[m.end() : i - 1]
        comma = _top_comma(inner)
        if comma < 0:
            continue
        itype, pp = inner[:comma].strip(), inner[comma + 1 :].strip()
        null = ", NULL" if m.group(1) == "IID_NULL_PPV_ARG" else ""
        out.append(text[pos : m.start()] + f"&IID_{itype}{null}, (void**)({pp})")
        pos = i
    return "".join(out) + text[pos:]


def _top_comma(s: str) -> int:
    depth = 0
    for i, ch in enumerate(s):
        if ch in "(<[":
            depth += 1
        elif ch in ")>]":
            depth -= 1
        elif ch == "," and depth == 0:
            return i
    return -1


@dataclass
class Lowered:
    text: str
    unplaced: list[str] = field(default_factory=list)
    # (class short name, rust method name) of every member call the AST resolved: what the
    # body really calls, for external reach. ("*", name) where the callee was not resolved
    # (an inherited ATL template member through an implicit `this`).
    calls: list[tuple[str, str]] = field(default_factory=list)


@dataclass
class DeclInfo:
    """What a member/constructor declaration id resolves to, in the CURRENT TU's AST."""

    owner: str  # qualified class name
    kind: str  # field | method | ctor | dtor
    name: str
    rust_name: str
    defaults: list[str | None] = field(default_factory=list)  # per param
    static: bool = False


def _tok(loc: dict, body_file: str, lo: int, hi: int) -> tuple[int, int] | None:
    """(offset, tokLen) of a location's token if it is literally inside the body text."""
    for cand in (loc.get("spellingLoc"), loc.get("expansionLoc"), loc):
        if not cand or "offset" not in cand:
            continue
        if cand.get("_file") == body_file and lo <= cand["offset"] < hi:
            # A spelling location inside a macro DEFINITION is not in the body; the check
            # above already excludes it (different file or outside the span).
            return cand["offset"], cand.get("tokLen", 0)
    return None


# Namespaces that wrap a C API (extern "C" functions): a call through one is the bare extern.
_C_API_NAMESPACES = frozenset({"DllExports"})


def _strip_type(qt: str) -> str:
    t = re.sub(r"\b(const|volatile|struct|class)\b", "", qt)
    t = t.replace("*", "").replace("&", "").strip()
    return t.split("::")[-1].strip()


class Lowerer:
    """Per-TU: decl ids are only meaningful inside the AST that produced them."""

    def __init__(
        self,
        decls: dict[str, DeclInfo],
        classes: dict,
        base_path,
        instances: dict[str, str] | None = None,
        canon=None,
    ) -> None:
        self.decls = decls
        self.classes = classes  # qualified name -> ClassModel
        self.base_path = base_path  # (from_class_short, to_class_short) -> "__base1." | ""
        # A template instantiation's spelling -> its concrete class (cpp_templates).
        self.instances = instances or {}
        self.canon = canon or (lambda k: k)
        self.ctor_index = {}  # (class short, ctorType qualType) -> N
        self._span: tuple[str, int, int] = ("", 0, 0)

    def _token_text(self, loc: dict) -> str | None:
        f, lo, hi = self._span
        t = _tok(loc, f, lo, hi)
        return None if t is None else cdp_text(f, t[0], t[0] + t[1])

    def lower(self, node: dict, span: tuple[str, int, int], this_class: str | None) -> Lowered:
        self._span = span
        f, lo, hi = span
        text = cdp_text(f, lo, hi)
        edits: list[tuple[int, int, str]] = []
        unplaced: list[str] = []
        self._calls: set[tuple[str, str]] = set()

        placed: set[tuple[int, int, str]] = set()

        # Object-like macros over a caller's local are inlined FIRST (_local_in_macro): every
        # node of their expansion is located at the macro-name token, and a member rename
        # there (`rc.bottom` -> rewrite at `Y_MARGIN`) would overwrite the macro name.
        def find_inlined(n: object) -> None:
            if isinstance(n, list):
                for x in n:
                    find_inlined(x)
            elif isinstance(n, dict):
                if n.get("kind") == "DeclRefExpr":
                    self._local_in_macro(n, f, lo, hi, edits)
                find_inlined(n.get("inner", []))

        find_inlined(node)
        inlined = {(e[0], e[0] + e[1]) for e in edits}

        def put(loc: dict, length: int | None, repl: str, what: str) -> None:
            t = _tok(loc, f, lo, hi)
            if t is None:
                unplaced.append(what)
                return
            if any(a <= t[0] - lo < b for a, b in inlined):
                return  # inside an inlined macro: its text is the definition's, as written
            edit = (t[0] - lo, t[1] if length is None else length, repl)
            # A macro that repeats its argument (`min(a, b)` expands `b` twice) yields one
            # AST node per copy, all spelled at the same token: one edit, not one per copy.
            # history.cpp's `min(pt.x, GetWidth())` became `this_->this_->this_->this_->`.
            if edit in placed:
                return
            placed.add(edit)
            edits.append(edit)

        def walk(n: object, parent: dict | None = None) -> None:
            if isinstance(n, list):
                for x in n:
                    walk(x, parent)
                return
            if not isinstance(n, dict):
                return
            k = n.get("kind")
            if k == "CXXThisExpr" and not _implicit(n):
                put(n["range"]["begin"], 4, "this_", "this")
            elif k == "MemberExpr" and self._static_via_object(n, f, lo, hi, edits):
                return  # `obj.f` is now `Class::f`: the object is gone, so is its walk
            elif k == "MemberExpr":
                self._member(n, put, this_class)
            elif k == "DeclRefExpr":
                self._static_ref(n, f, lo, hi, edits)
            elif k == "CallExpr" and self._fn_pointer_member_call(n, f, lo, hi, edits):
                self._defaults(n, put, f, lo, hi, late)
            elif k in ("CXXMemberCallExpr", "CallExpr"):
                # Into `late`: a default argument goes after the LAST written argument, and
                # that argument's own edits at the same offset (a conversion's
                # `).op_to_HWND()`) must land in front of it, i.e. be discovered first.
                self._defaults(n, put, f, lo, hi, late)
            elif k == "CXXNewExpr":
                self._new(n, put, f, lo, hi, edits)
            elif k == "CXXDeleteExpr":
                self._delete(n, put, f, lo, hi, edits)
            elif k == "ImplicitCastExpr" and n.get("castKind") in (
                "DerivedToBase",
                "UncheckedDerivedToBase",
            ):
                # The object of a member access is _member's: it writes the base path at the
                # member name, and wrapping the object too would walk the path twice.
                if not (parent and parent.get("kind") == "MemberExpr"):
                    self._upcast(n, f, lo, hi, edits, unplaced)
            elif k == "ImplicitCastExpr" and n.get("castKind") == "UserDefinedConversion":
                obj = self._conversion(n, f, lo, hi, edits)
                if obj is not None:
                    walk(obj, None)  # the object only: the implicit member is ours
                    return
            elif k == "VarDecl":
                self._local(n, f, lo, hi, edits)
            elif k in ("CXXTemporaryObjectExpr", "CXXFunctionalCastExpr"):
                self._temporary(n, f, lo, hi, edits)
            elif k == "MaterializeTemporaryExpr":
                self._implicit_construct(n, f, lo, hi, edits, late)
            for key in ("inner",):
                walk(n.get(key, []), n)

        late: list = []
        walk(node)
        edits.extend(late)
        return Lowered(_apply(text, edits), unplaced, sorted(self._calls))

    # -- rules -------------------------------------------------------------------------

    def _member(self, n: dict, put, this_class: str | None) -> None:
        if not n.get("name"):
            return  # the implicit hop through an anonymous union: the field's rename spells it
        info = self.decls.get(n.get("referencedMemberDecl", ""))
        if info is not None and info.kind != "field":
            self._calls.add((info.owner.split("::")[-1], info.rust_name))
        elif info is None and n.get("name"):
            self._calls.add(("*", n["name"]))
        base = (n.get("inner") or [{}])[0]
        while base.get("kind") == "MemberExpr" and not base.get("name"):
            base = (base.get("inner") or [{}])[0]
        this_node = _unwrap_casts(base)
        implicit_this = this_node.get("kind") == "CXXThisExpr" and _implicit(this_node)
        if info is None and not implicit_this and self._plain_base_field(n, base, put):
            return
        if info is None:
            # A member of a class the module does not define. Through an implicit `this`
            # it is inherited from an EXTERNAL base (`Invalidate(FALSE)` in a CWindowImpl
            # subclass): qualify it, or the model reads a member call as a free function.
            if implicit_this and this_class:
                name = n.get("name", "?")
                # ATL's template machinery (Create, DefWindowProc, DoModal...) is answered by
                # the class's own generated glue: keep the call on the object.
                path = (
                    ""
                    if name in ATL_IMPL_METHODS
                    else self.base_path(this_class.split("::")[-1], "\0external")
                )
                put(n["range"]["end"], 0, "this_->" + path, name)
            return
        # The static type BEFORE any derived-to-base cast: the base path is ours to write.
        static_type = self._ty(this_node.get("type", {}).get("qualType", ""))
        if implicit_this:
            static_type = (this_class or "").split("::")[-1]
        path = self.base_path(static_type, info.owner.split("::")[-1]) if static_type else ""
        own = {q.split("::")[-1] for q, c in self.classes.items() if not c.external}
        if info.name in ATL_IMPL_METHODS and _short_owner(info.owner) not in own:
            # An explicit `_AtlModule.WinMain(..)` / `dlg.DoModal(..)`: the ATL template's member
            # is the class's generated glue, on the object itself -- the same rule as through an
            # implicit `this` above. The base path would land on CWindow / the base's data.
            path = ""
        # clang's JSON gives a MemberExpr no `loc`; its range ENDS on the member-name token
        # (`p->name`, and for an implicit `this` the range is that one token).
        name_loc = n["range"]["end"]
        prefix = ("this_->" if implicit_this else "") + path
        # Rename when the WRITTEN token differs from the Rust name: an overload
        # (`alloc_seconds` -> `alloc_seconds__1`), or a TCHAR macro spelling whose declaration
        # is the W function (`SendMessage` written, CWindow::SendMessageW declared).
        written = self._token_text(name_loc)
        renamed = info.rust_name if written is not None and written != info.rust_name else None
        if prefix or renamed:
            put(name_loc, None if renamed else 0, prefix + (renamed or ""), info.name)

    def _local_in_macro(self, n: dict, f: str, lo: int, hi: int, edits) -> None:
        """palette.cpp's `#define Y_MARGIN ((rcClient.bottom / 2) - CXY_COLORBOX)` names the
        CALLER's local `rcClient`. No Rust item can (a `macro_rules!` is hygienic, a const has
        no locals), so such an object-like macro is inlined where it is expanded, exactly as
        the preprocessor did. The tell is in the AST: a variable reference SPELLED inside a
        `#define` line and EXPANDED at a token in this body."""
        ref = n.get("referencedDecl", {})
        if ref.get("kind") not in ("VarDecl", "ParmVarDecl"):
            return
        begin = n.get("range", {}).get("begin", {})
        spell, exp = begin.get("spellingLoc"), begin.get("expansionLoc")
        if not spell or not exp or exp.get("isMacroArgExpansion") or "offset" not in spell:
            return
        if exp.get("_file") != f or not lo <= exp.get("offset", -1) < hi:
            return
        src_file = spell.get("_file")
        if not src_file or src_file.startswith("<"):
            return  # `<scratch space>`: a `##`-pasted token, spelled on no `#define` line
        cdp_text(src_file, 0, 0)  # load into _SRC: offsets are BYTES
        data = _SRC[src_file]
        line_start = data.rfind(b"\n", 0, spell["offset"]) + 1
        line_end = data.find(b"\n", spell["offset"])
        line = data[line_start : line_end if line_end >= 0 else len(data)].decode(
            "utf-8", errors="replace"
        )
        m = re.match(r"\s*#\s*define\s+([A-Za-z_]\w*)(?![\w(])\s+(.*?)\s*(?:/\*.*)?$", line)
        name = cdp_text(f, exp["offset"], exp["offset"] + exp.get("tokLen", 0))
        if m is None or m.group(1) != name:
            return  # a function-like macro, or a nested expansion: not this shape
        edits.append((exp["offset"] - lo, len(name), f"({m.group(2).strip()})"))

    def _plain_base_field(self, n: dict, base: dict, put) -> bool:
        """`m_rcOld.left` where CRect DERIVES from RECT: clang wraps the object in a
        DerivedToBase cast to the plain struct, whose fields are not in the decl table. The
        member lives in the base at `__base0`; without the path, Rust reached it through the
        generated Deref, a reference made through a raw pointer (dangerous_implicit_autorefs)
        where C++ just named the base subobject."""
        if base.get("kind") != "ImplicitCastExpr" or base.get("castKind") not in (
            "DerivedToBase",
            "UncheckedDerivedToBase",
        ):
            return False
        if n.get("name") in ATL_IMPL_METHODS:
            # `_AtlModule.WinMain(..)` reaches CAtlExeModuleT through the same cast, but an ATL
            # template's member is the class's own generated glue, never the base's.
            return False
        obj = (base.get("inner") or [{}])[0]
        static_type = self._ty(obj.get("type", {}).get("qualType", ""))
        if not static_type or self._class(static_type) is None:
            return False
        target = self._ty(base.get("type", {}).get("qualType", ""))
        path = self.base_path(static_type, target, deep=True) or self.base_path(
            static_type, "\0external", deep=True
        )
        if not path:
            return False
        put(n["range"]["end"], 0, path, n.get("name", "?"))
        return True

    def _static_via_object(self, n: dict, f: str, lo: int, hi: int, edits) -> bool:
        """`toolsModel.CreateBrush(..)` where CreateBrush is STATIC: C++ allows the object
        form, Rust's associated fn does not -> `ToolsModel::CreateBrush(..)`. Only for an
        object with no side effects (a name or member chain); C++ evaluates the object
        anyway, so dropping a call there would drop behaviour."""
        info = self.decls.get(n.get("referencedMemberDecl", ""))
        if info is None or not info.static or info.kind != "method":
            return False
        obj = (n.get("inner") or [{}])[0]
        node = obj
        while node.get("kind") in ("MemberExpr", "ImplicitCastExpr", "ParenExpr"):
            node = (node.get("inner") or [{}])[0]
        if node.get("kind") not in ("DeclRefExpr", "CXXThisExpr"):
            return False
        span = _node_span(n, f, lo, hi)
        name = _tok(n["range"]["end"], f, lo, hi)
        if span is None or name is None:
            return False
        cls = info.owner.split("::")[-1]
        self._calls.add((cls, info.rust_name))
        edits.append((span[0], name[0] - lo - span[0] + name[1], f"{cls}::{info.rust_name}"))
        return True

    def _fn_pointer_member_call(self, n: dict, f: str, lo: int, hi: int, edits) -> bool:
        """`p->m_fn(args)` where `m_fn` is a DATA member of function-pointer type
        (atlimagedx.h's `_shared()->m_GetImageHorizontalResolution(..)`, CInitGDIPlus's
        Startup/...): a call THROUGH the field, written `(p->m_fn)(args)` so it does not
        read as a method call -- a worker turned it into `(*p).m_fn(..)`, a method that
        does not exist."""
        callee = (n.get("inner") or [{}])[0]
        while callee.get("kind") in ("ImplicitCastExpr", "ParenExpr") and callee.get("inner"):
            if callee.get("kind") == "ParenExpr":
                return False  # already `(p->m_fn)(..)`
            callee = callee["inner"][0]
        if callee.get("kind") != "MemberExpr":
            return False
        qt = callee.get("type", {}).get("qualType", "")
        if "(*)" not in qt and not re.search(r"\bFN_|\bFUN_|PROC\b", qt):
            return False
        span = _node_span(callee, f, lo, hi)
        if span is None:
            return False
        edits.append((span[0], 0, "("))
        edits.append((span[1], 0, ")"))
        return True

    def _conversion(self, n: dict, f: str, lo: int, hi: int, edits) -> dict | None:
        """C++'s implicit user-defined conversion, `IsWindowVisible(textEditWindow)` through
        `CWindow::operator HWND()`, made an explicit call: `(textEditWindow).op_to_HWND()`.
        Returns the object node (whose own edits still apply), or None if not this shape."""
        call = _unwrap_casts((n.get("inner") or [{}])[0])
        if call.get("kind") != "CXXMemberCallExpr":
            return None
        me = (call.get("inner") or [{}])[0]
        if me.get("kind") != "MemberExpr":
            return None
        info = self.decls.get(me.get("referencedMemberDecl", ""))
        obj = (me.get("inner") or [None])[0]
        span = _node_span(obj, f, lo, hi) if obj else None
        if info is None or span is None:
            return None
        self._calls.add((info.owner.split("::")[-1], info.rust_name))
        edits.append((span[0], 0, "("))
        edits.append((span[1], 0, f").{info.rust_name}()"))
        # A method call needs no upcast: autoderef reaches CWindow's op_to_HWND through the
        # generated Deref, and _upcast's `.__base0` would land outside the call.
        while (
            obj is not None
            and obj.get("kind") == "ImplicitCastExpr"
            and obj.get("castKind")
            in (
                "DerivedToBase",
                "UncheckedDerivedToBase",
                "NoOp",
            )
        ):
            obj = (obj.get("inner") or [obj])[0]
        return obj

    def _static_ref(self, n: dict, f: str, lo: int, hi: int, edits) -> None:
        """A STATIC member named without its class (`_shared()` inside CImageDx): after
        assembly it is an associated fn, reachable only as `CImageDx::_shared()`."""
        info = self.decls.get(n.get("referencedDecl", {}).get("id", ""))
        if info is None and n.get("referencedDecl", {}).get("kind") == "FunctionDecl":
            # gdiplus's `DllExports::GdipCreateSolidFill(..)`: a namespace around the flat
            # extern "C" API, whose Rust is the bare extern. Left qualified, every GDI+
            # wrapper body named a module that does not exist.
            span = _node_span(n, f, lo, hi)
            if span is not None:
                text = cdp_text(f, lo + span[0], lo + span[1])
                m = re.fullmatch(r"(?:::)?((?:\w+::)+)(\w+)", text.strip())
                if m and m.group(1).rstrip(":").split("::")[-1] in _C_API_NAMESPACES:
                    edits.append((span[0], span[1] - span[0], m.group(2)))
            return
        if info is not None and info.kind == "field" and info.static:
            # A static data member, `s_nLock` or `CWaitCursor::s_nLock`: cpp_classes.rs
            # renders it as the static `CWaitCursor__s_nLock`.
            span = _node_span(n, f, lo, hi)
            if span is not None:
                cls = info.owner.split("::")[-1]
                edits.append((span[0], span[1] - span[0], f"{cls}__{info.name}"))
            return
        if info is not None and info.kind == "function":
            # An overloaded module function (`Fill` -> `Fill__1`): the decl clang resolved
            # the call to names the overload.
            t = _tok(n["range"]["end"], f, lo, hi)
            if t is not None and info.rust_name != info.name:
                edits.append((t[0] - lo, t[1], info.rust_name))
            return
        if info is not None and info.kind == "enumconst":
            # The whole reference, qualifier included, is the island's flat name.
            span = _node_span(n, f, lo, hi)
            if span is not None:
                edits.append((span[0], span[1] - span[0], info.rust_name))
            return
        if info is None or not info.static or info.kind != "method":
            return
        self._calls.add((info.owner.split("::")[-1], info.rust_name))
        # The range END is the name token: `RegistrySettings::SetWallpaper(..)` BEGINS
        # at its qualifier, and qualifying that token wrote `X::SetWallpaper::SetWallpaper`.
        t = _tok(n["range"]["end"], f, lo, hi)
        if t is None:
            return
        before = cdp_text(f, max(lo, t[0] - 2), t[0])
        if before.endswith("::"):
            return  # already `Class::name`
        cls = info.owner.split("::")[-1]
        written = cdp_text(f, t[0], t[0] + t[1])
        repl = info.rust_name if written != info.rust_name else written
        edits.append((t[0] - lo, t[1], f"{cls}::{repl}"))

    def _defaults(self, n: dict, put, f: str, lo: int, hi: int, edits) -> None:
        inner = n.get("inner", [])
        if not inner:
            return
        callee = inner[0]
        while callee.get("kind") in ("ImplicitCastExpr", "ParenExpr") and callee.get("inner"):
            callee = callee["inner"][0]
        info = self.decls.get(callee.get("referencedMemberDecl", "")) or self.decls.get(
            callee.get("referencedDecl", {}).get("id", "")
        )
        args = inner[1:]
        missing = [a for a in args if a.get("kind") == "CXXDefaultArgExpr"]
        if info is None and missing and callee.get("kind") == "MemberExpr":
            # An ATL template method the generated glue implements (CWindowImpl::Create):
            # no decl id of ours, so its defaults come from atlwin.h's own declarations,
            # told apart by full arity (written + defaulted).
            table = _ATL_GLUE_DEFAULTS.get((callee.get("name"), len(args)))
            if table is not None:
                info = DeclInfo("", "method", callee["name"], callee["name"], table)
        if info is None or not missing:
            return
        written = len(args) - len(missing)
        texts = info.defaults[written : written + len(missing)]
        while texts and texts[-1] is None and info.owner == "" and info.kind == "method":
            texts = texts[:-1]  # an ATL glue table's slot the glue does not have
        if any(t is None for t in texts):
            return
        written_texts = [t for t in texts if t is not None]
        end = n["range"]["end"]
        t = _tok(end, f, lo, hi)
        if t is None:
            return
        sep = ", " if written else ""
        edits.append((t[0] - lo, 0, sep + ", ".join(written_texts)))  # before the closing paren

    def _new(self, n: dict, put, f: str, lo: int, hi: int, edits) -> None:
        span = _node_span(n, f, lo, hi)
        if span is None:
            return
        t = cdp_text(f, lo + span[0], lo + span[1])
        if n.get("isPlacement"):
            # `new(cicNoThrow) T(args)`: a TAG-dispatched allocator (`operator new(size_t,
            # const CicNoThrow&) noexcept`, CTF's idiom) is allocate-or-NULL, i.e. a plain
            # `new` whose result the caller null-checks -- and the generated __new_N returns
            # NULL rather than constructing into it. A placement into a BUFFER (`new(buf) T`,
            # `operator new(size_t, void*)`) constructs in place: a different operation, left
            # unlowered so the audit rejects it loudly instead of allocating fresh memory.
            sig = n.get("operatorNewDecl", {}).get("type", {}).get("qualType", "")
            tm = re.fullmatch(r"void \*\(size_t, const (\w+) &\) noexcept", sig)
            pm = re.match(r"new\s*\((?:[^()]|\([^()]*\))*\)\s*", t)
            if tm is None or pm is None:
                return
            tag = tm.group(1)  # its allocator: cpp_alloc_<Tag> (cpp_app._allocator_helpers)
            t = "new " + t[pm.end() :]
        else:
            tag = None
        alloc = self._ty(n.get("type", {}).get("qualType", ""))
        if n.get("isArray"):
            m = re.match(r"new\s+(.+?)\s*\[(.*)\]\s*$", t, re.S)
            if m:
                # The untagged spelling is what every banked body was bought against: a
                # change of it alone would re-buy them all.
                fn = f"cpp_new_array_{tag}::" if tag else "cpp_new_array"
                edits.append((span[0], span[1] - span[0], f"{fn}<{m.group(1)}>({m.group(2)})"))
            return
        cls = self._class(alloc)
        if cls is None:
            m = re.match(r"new\s+([\w:\s\*]+?)\s*(\((.*)\))?\s*$", t, re.S)
            if m:
                ty, v = m.group(1), m.group(3) or ""
                repl = (
                    f"cpp_new_in(cpp_alloc_{tag}::<{ty}>(), {v})" if tag else f"cpp_new<{ty}>({v})"
                )
                edits.append((span[0], span[1] - span[0], repl))
            return
        construct = next((x for x in n.get("inner", []) if "Construct" in x.get("kind", "")), None)
        idx = self._ctor_n(cls, construct)
        m = re.match(r"new\s+[\w:]+\s*", t)
        if m is None:
            return
        rest = t[m.end() :] or "()"
        if tag:
            inner = rest.strip()[1:-1].strip()
            args = f"cpp_alloc_{tag}::<{cls.name}>()" + (f", {inner}" if inner else "")
            edits.append((span[0], span[1] - span[0], f"{cls.name}__new_in_{idx}({args})"))
            return
        edits.append((span[0], span[1] - span[0], f"{cls.name}__new_{idx}{rest}"))

    def _delete(self, n: dict, put, f: str, lo: int, hi: int, edits) -> None:
        span = _node_span(n, f, lo, hi)
        if span is None:
            return
        t = cdp_text(f, lo + span[0], lo + span[1])
        m = re.match(r"delete\s*(\[\s*\])?\s*(.*)$", t, re.S)
        if m is None:
            return
        # Surgical: the `delete` keyword becomes the call's head and `)` closes it, so the
        # operand keeps its OWN edits. Replacing the whole expression with the operand's
        # source text dropped them and left the tail behind: rapps' `delete RichEdit;`
        # (an implicit-this member) lowered to `CAppRichEdit__delete(RichEdit)ichEdit;`.
        head = m.start(2)
        if m.group(1):
            edits.append((span[0], head, "cpp_delete_array("))
            edits.append((span[1], 0, ")"))
            return
        target = n.get("inner", [{}])[0].get("type", {}).get("qualType", "")
        cls = self._class(self._ty(target))
        edits.append((span[0], head, f"{cls.name}__delete(" if cls else "cpp_delete("))
        edits.append((span[1], 0, ")"))

    def _upcast(self, n: dict, f: str, lo: int, hi: int, edits, unplaced) -> None:
        inner = (n.get("inner") or [{}])[0]
        this_node = _unwrap_casts(inner)
        if this_node.get("kind") == "CXXThisExpr" and _implicit(this_node):
            return  # an implicit `this` has no text to wrap: _member writes its base path
        # An EXPLICIT `this` converted to a base pointer -- `return this;` from rosui.h's
        # `CUiBox *AsBox()`, or `this` passed as a base -- is the base SUBOBJECT's address.
        # Left as `this_`, the model wrote `this_ as *mut CUiBox`: CUiBox sits 16 bytes in, so
        # every layout call went through the wrong vtable and rapps' teardown read RichEdit
        # from an HWND. (A member access through `this` never gets here: the caller skips a
        # cast whose parent is the MemberExpr.)
        src = self._ty(inner.get("type", {}).get("qualType", ""))
        dst = self._ty(n.get("type", {}).get("qualType", ""))
        path = self.base_path(src, dst)
        if not path:
            return  # the primary base: the same address, a cast is enough
        span = _node_span(inner, f, lo, hi)
        if span is None:
            unplaced.append(f"upcast {src}->{dst}")
            return
        pointer = "*" in n.get("type", {}).get("qualType", "")
        if pointer:
            edits.append((span[0], 0, "(&("))
            edits.append((span[1], 0, ")->" + path.rstrip(".") + ")"))
        else:
            edits.append((span[0], 0, "("))
            edits.append((span[1], 0, ")." + path.rstrip(".")))

    def _local(self, n: dict, f: str, lo: int, hi: int, edits) -> None:
        """`audio_membuffer buf(fmt);` -> `audio_membuffer buf = audio_membuffer__make_1(fmt);`

        Surgical: one insertion after the declarator name, so the argument text keeps its
        own edits (an upcast of an argument, a member path inside it)."""
        cls = self._class(self._ty(n.get("type", {}).get("qualType", "")))
        if cls is None or "*" in n["type"]["qualType"] or "&" in n["type"]["qualType"]:
            return
        if "[" in n["type"]["qualType"]:
            return  # an array of objects: rendered element-wise, not here
        init = (n.get("inner") or [None])[0]
        if init is not None and init.get("kind") not in ("CXXConstructExpr",):
            return  # `T x = expr;` -- a copy, already an expression the model can read
        name = _tok(n["loc"], f, lo, hi)
        if name is None:
            return
        idx = self._ctor_n(cls, init)
        after = name[0] - lo + name[1]
        tail = cdp_text(f, name[0] + name[1], hi)
        rest = tail.lstrip()
        if rest.startswith("("):
            edits.append((after, 0, f" = {cls.name}__make_{idx}"))
        elif re.match(r"(=\s*)?\{", rest):
            # List-initialization, `CRect rc = { 0, 0, w, h };`: the braces are the
            # constructor's argument list. Swap them for parens; the arguments keep their
            # own edits (a member path inside one), so only the two delimiters change.
            m = re.match(r"\s*(=\s*)?\{", tail)
            if m is None:
                return
            depth, i = 0, m.end() - 1
            while i < len(tail):
                depth += (tail[i] == "{") - (tail[i] == "}")
                if depth == 0:
                    break
                i += 1
            edits.append((after, m.end(), f" = {cls.name}__make_{idx}("))
            edits.append((after + i, 1, ")"))
        else:
            edits.append((after, 0, f" = {cls.name}__make_{idx}()"))

    def _temporary(self, n: dict, f: str, lo: int, hi: int, edits) -> None:
        """`audio_format(44100, 16, 2)` as an expression -> `audio_format__make_N(...)`."""
        cls = self._class(self._ty(n.get("type", {}).get("qualType", "")))
        span = _node_span(n, f, lo, hi)
        if span is None:
            return
        t = cdp_text(f, lo + span[0], lo + span[1])
        if cls is None:
            # A functional cast to a SCALAR type, dib.cpp's `INT(cx * eTan)`: written as C's
            # `((INT)(cx * eTan))`, or the model calls `INT(..)` like a function.
            m = re.match(r"([A-Za-z_]\w*)\s*\(", t)
            if (
                n.get("kind") == "CXXFunctionalCastExpr"
                and m is not None
                and t.rstrip().endswith(")")
                and n.get("castKind") != "ConstructorConversion"
            ):
                edits.append((span[0], m.end(), f"(({m.group(1)})("))
                edits.append((span[1], 0, ")"))
            return
        m = re.match(r"[\w:]+(?=\s*[({])", t)
        if m is None:
            return
        construct = (
            n
            if n.get("kind") == "CXXTemporaryObjectExpr"
            else next((x for x in n.get("inner", []) if x.get("kind") == "CXXConstructExpr"), None)
        )
        edits.append((span[0], m.end(), f"{cls.name}__make_{self._ctor_n(cls, construct)}"))

    def _implicit_construct(self, n: dict, f: str, lo: int, hi: int, edits, late) -> None:
        """An implicit converting-constructor temporary, written out: rapps' `fnFileOpen`
        passes a `char *` to `const CStringA &`, and with no `CStringA(` in the text the
        model cast the pointer. Spelled `Cls__make_N(p)` for a modeled class and `Type(p)`
        for an external one, the way `_temporary` leaves a written one. A string literal
        stays as written: no body has been seen to need it, and spelling one would move the
        lowered text (the purchase cache key) of every literal passed to a CString."""
        c = n.get("inner", [{}])[0]
        while c.get("kind") == "ImplicitCastExpr" and c.get("castKind") in (
            "ConstructorConversion",
            "NoOp",
        ):
            c = (c.get("inner") or [{}])[0]
        args = c.get("inner") or []
        if c.get("kind") != "CXXConstructExpr" or len(args) != 1:
            return
        arg = _unwrap_casts(args[0])
        if arg.get("kind") == "StringLiteral":
            return
        cls_ty = self._ty(c.get("type", {}).get("qualType", ""))
        if self._ty(args[0].get("type", {}).get("qualType", "")) == cls_ty:
            return  # a copy or move, not a conversion
        span, arg_span = _node_span(c, f, lo, hi), _node_span(args[0], f, lo, hi)
        if span is None or span != arg_span:
            return  # written (`CStringA(p)`): its own text already says so
        cls = self._class(cls_ty)
        if cls is not None:
            head = f"{cls.name}__make_{self._ctor_n(cls, c)}("
        else:
            head = re.sub(r"\b(const|volatile|struct|class)\b|&", "", c["type"]["qualType"])
            head = " ".join(head.split()) + "("
        edits.append((span[0], 0, head))
        late.append((span[1], 0, ")"))

    # -- helpers -----------------------------------------------------------------------

    def _ty(self, qt: str) -> str:
        """`_strip_type`, with a template instantiation named by its concrete class."""
        t = re.sub(r"\b(const|volatile|struct|class)\b", "", qt).replace("*", "").replace("&", "")
        if "<" in t:
            key = cpp_templates.instance_key(t)
            hit = self.instances.get(key) or self.instances.get(self.canon(key))
            if hit is not None:
                return hit
        return _strip_type(qt)

    def _class(self, short: str):
        for q, c in self.classes.items():
            if q.split("::")[-1] == short:
                return c
        return None

    def _ctor_n(self, cls, construct: dict | None) -> int:
        if construct is None:
            return 0
        sig = construct.get("ctorType", {}).get("qualType", "")
        ctors = [m for m in cls.methods if m.kind == "ctor"]
        for i, m in enumerate(ctors):
            want = "(" + ", ".join(p.c_type for p in m.params) + ")"
            if sig.replace(" ", "").startswith("void" + want.replace(" ", "")):
                return i
        return 0


def _implicit(n: dict) -> bool:
    # clang's JSON spells it `implicit` on a CXXThisExpr and `isImplicit` on declarations.
    return bool(n.get("implicit") or n.get("isImplicit"))


def _short_owner(owner: str) -> str:
    return owner.split("<")[0].split("::")[-1].strip()


def _unwrap_casts(n: dict) -> dict:
    while n.get("kind") in ("ImplicitCastExpr", "ParenExpr") and n.get("inner"):
        n = n["inner"][0]
    return n


_SRC: dict[str, bytes] = {}


def cdp_text(f: str, lo: int, hi: int) -> str:
    if f not in _SRC:
        with open(f, "rb") as fh:
            _SRC[f] = fh.read()
    return _SRC[f][lo:hi].decode("utf-8", errors="replace")


def _node_span(n: dict, f: str, lo: int, hi: int) -> tuple[int, int] | None:
    rng = n.get("range")
    if not rng:
        return None
    b = _tok(rng["begin"], f, lo, hi)
    e = _tok(rng["end"], f, lo, hi)
    if b is None or e is None:
        return None
    return b[0] - lo, e[0] - lo + e[1]


def _apply(text: str, edits: list[tuple[int, int, str]]) -> str:
    """Apply right to left. Two insertions at one offset keep their discovery order: the
    later one is applied first, so the earlier one ends up in front of it."""
    out = text
    # An identical REPLACEMENT twice is one macro argument seen once per expansion copy
    # (`min(a, b)` repeats `b`): applying it twice would replace text it already replaced.
    # Identical insertions can be legitimate (nested conversions each insert `(`), so only
    # `put` dedupes those, for the member-path edits that are known to come from copies.
    seen: set[tuple[int, int, str]] = set()
    kept = []
    for e in edits:
        if e[1] > 0 and e in seen:
            continue
        seen.add(e)
        kept.append(e)
    edits = kept
    for i in sorted(range(len(edits)), key=lambda i: (edits[i][0], i), reverse=True):
        off, length, repl = edits[i]
        out = out[:off] + repl + out[off + length :]
    return out


__all__ = ["DeclInfo", "Lowered", "Lowerer", "cdp"]
