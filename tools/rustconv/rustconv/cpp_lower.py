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
        "GetWndClassInfo",
        "GetWindowProc",
    }
)


@dataclass
class Lowered:
    text: str
    unplaced: list[str] = field(default_factory=list)


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


def _strip_type(qt: str) -> str:
    t = re.sub(r"\b(const|volatile|struct|class)\b", "", qt)
    t = t.replace("*", "").replace("&", "").strip()
    return t.split("::")[-1].strip()


class Lowerer:
    """Per-TU: decl ids are only meaningful inside the AST that produced them."""

    def __init__(self, decls: dict[str, DeclInfo], classes: dict, base_path) -> None:
        self.decls = decls
        self.classes = classes  # qualified name -> ClassModel
        self.base_path = base_path  # (from_class_short, to_class_short) -> "__base1." | ""
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

        def put(loc: dict, length: int | None, repl: str, what: str) -> None:
            t = _tok(loc, f, lo, hi)
            if t is None:
                unplaced.append(what)
                return
            edits.append((t[0] - lo, t[1] if length is None else length, repl))

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
            elif k == "MemberExpr":
                self._member(n, put, this_class)
            elif k == "DeclRefExpr":
                self._static_ref(n, f, lo, hi, edits)
            elif k in ("CXXMemberCallExpr", "CallExpr"):
                self._defaults(n, put, f, lo, hi, edits)
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
            elif k == "VarDecl":
                self._local(n, f, lo, hi, edits)
            elif k in ("CXXTemporaryObjectExpr", "CXXFunctionalCastExpr"):
                self._temporary(n, f, lo, hi, edits)
            for key in ("inner",):
                walk(n.get(key, []), n)

        walk(node)
        return Lowered(_apply(text, edits), unplaced)

    # -- rules -------------------------------------------------------------------------

    def _member(self, n: dict, put, this_class: str | None) -> None:
        if not n.get("name"):
            return  # the implicit hop through an anonymous union: the field's rename spells it
        info = self.decls.get(n.get("referencedMemberDecl", ""))
        base = (n.get("inner") or [{}])[0]
        while base.get("kind") == "MemberExpr" and not base.get("name"):
            base = (base.get("inner") or [{}])[0]
        this_node = _unwrap_casts(base)
        implicit_this = this_node.get("kind") == "CXXThisExpr" and _implicit(this_node)
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
        static_type = _strip_type(this_node.get("type", {}).get("qualType", ""))
        if implicit_this:
            static_type = (this_class or "").split("::")[-1]
        path = self.base_path(static_type, info.owner.split("::")[-1]) if static_type else ""
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

    def _static_ref(self, n: dict, f: str, lo: int, hi: int, edits) -> None:
        """A STATIC member named without its class (`_shared()` inside CImageDx): after
        assembly it is an associated fn, reachable only as `CImageDx::_shared()`."""
        info = self.decls.get(n.get("referencedDecl", {}).get("id", ""))
        if info is not None and info.kind == "enumconst":
            # The whole reference, qualifier included, is the island's flat name.
            span = _node_span(n, f, lo, hi)
            if span is not None:
                edits.append((span[0], span[1] - span[0], info.rust_name))
            return
        if info is None or not info.static or info.kind != "method":
            return
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
        if info is None or not missing:
            return
        written = len(args) - len(missing)
        texts = info.defaults[written : written + len(missing)]
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
        alloc = _strip_type(n.get("type", {}).get("qualType", ""))
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
        operand = m.group(2).strip()
        if m.group(1):
            edits.append((span[0], span[1] - span[0], f"cpp_delete_array({operand})"))
            return
        target = n.get("inner", [{}])[0].get("type", {}).get("qualType", "")
        cls = self._class(_strip_type(target))
        repl = f"{cls.name}__delete({operand})" if cls else f"cpp_delete({operand})"
        edits.append((span[0], span[1] - span[0], repl))

    def _upcast(self, n: dict, f: str, lo: int, hi: int, edits, unplaced) -> None:
        inner = (n.get("inner") or [{}])[0]
        if _unwrap_casts(inner).get("kind") == "CXXThisExpr":
            return  # `this` as a member's object: _member writes the base path itself
        src = _strip_type(inner.get("type", {}).get("qualType", ""))
        dst = _strip_type(n.get("type", {}).get("qualType", ""))
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
        cls = self._class(_strip_type(n.get("type", {}).get("qualType", "")))
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
        cls = self._class(_strip_type(n.get("type", {}).get("qualType", "")))
        span = _node_span(n, f, lo, hi)
        if cls is None or span is None:
            return
        t = cdp_text(f, lo + span[0], lo + span[1])
        m = re.match(r"[\w:]+(?=\s*[({])", t)
        if m is None:
            return
        construct = (
            n
            if n.get("kind") == "CXXTemporaryObjectExpr"
            else next((x for x in n.get("inner", []) if x.get("kind") == "CXXConstructExpr"), None)
        )
        edits.append((span[0], m.end(), f"{cls.name}__make_{self._ctor_n(cls, construct)}"))

    # -- helpers -----------------------------------------------------------------------

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
    for i in sorted(range(len(edits)), key=lambda i: (edits[i][0], i), reverse=True):
        off, length, repl = edits[i]
        out = out[:off] + repl + out[off + length :]
    return out


__all__ = ["DeclInfo", "Lowered", "Lowerer", "cdp"]
