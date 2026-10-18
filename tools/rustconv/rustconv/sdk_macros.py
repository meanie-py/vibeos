"""sdk_macros.py -- SDK header macros a crate calls, lowered from the compiler's own expansion.

`port macros` buys the module's OWN function-like macros from the model. SDK header macros
(`PropSheet_Changed`, `ListView_InsertColumn`, `TreeView_GetSelection`, `MAKEWPARAM`) are a
different problem: the control panels call ~40 of them, every one is a `SendMessage` with casts,
and win32macros-rust cannot hold them because their types (WPARAM, HWND, LV_COLUMNW) are each
consumer's island types. Re-porting them by hand per crate is how `LOWORD` ended up in eight
spellings.

So nothing here is written by anyone. The module's own preprocessor, with the TU's real compile
command on every arch, expands `NAME(__rcv_p0__, ..)`; the expansion is parsed as a C expression
and printed as Rust. What it cannot lower -- a statement block, a string literal, a construct the
grammar below does not cover, or an expansion that differs between arches -- is REFUSED by name
and left for a hand port, never approximated.

Two C semantics are kept explicitly:
- an argument passed to a call without a cast gets C's implicit conversion, spelled `as _` so
  Rust infers the callee's parameter type (`PropSheet_PressButton(d, i)` passes an int as WPARAM);
- a pointer cast directly under an integer cast is dropped: `(LPARAM)(const LV_COLUMN *)(p)`
  reinterprets no bits, and keeping it would demand an island type the crate may not carry.
"""

from __future__ import annotations

import re
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path

from . import generated_items
from .macro_authority import _ARCHES, _without_outputs, compile_argv


class LowerError(ValueError):
    """The expansion is outside what this lowering covers."""


# C type keywords that may start a cast.
_C_TYPE_WORDS = {
    "void",
    "char",
    "short",
    "int",
    "long",
    "signed",
    "unsigned",
    "float",
    "double",
    "const",
    "volatile",
    "struct",
    "union",
    "enum",
    "_Bool",
    "__int64",
}
_C_PRIM = {
    ("void",): "c_void",
    ("char",): "c_char",
    ("signed", "char"): "i8",
    ("unsigned", "char"): "c_uchar",
    ("short",): "c_short",
    ("short", "int"): "c_short",
    ("unsigned", "short"): "c_ushort",
    ("unsigned", "short", "int"): "c_ushort",
    ("int",): "c_int",
    ("signed",): "c_int",
    ("signed", "int"): "c_int",
    ("unsigned",): "c_uint",
    ("unsigned", "int"): "c_uint",
    ("long",): "c_long",
    ("long", "int"): "c_long",
    ("unsigned", "long"): "c_ulong",
    ("unsigned", "long", "int"): "c_ulong",
    ("long", "long"): "i64",
    ("unsigned", "long", "long"): "u64",
    ("__int64",): "i64",
    ("unsigned", "__int64"): "u64",
}

_TOKEN = re.compile(
    r"\s*(?:(?P<num>0[xX][0-9a-fA-F]+[uUlL]*|\d+[uUlL]*)"
    r"|(?P<id>[A-Za-z_]\w*)"
    r"|(?P<str>L?\"(?:[^\"\\]|\\.)*\"|L?'(?:[^'\\]|\\.)*')"
    r"|(?P<op>->|<<|>>|<=|>=|==|!=|&&|\|\||[-+*/%&|^~!<>=?:,.()\[\]{};]))"
)


def cxx_ref_alias(name: str, expansion: str) -> str | None:
    """A type macro's C++ spelling, `const IID &` (REFIID/REFCLSID in a C++ unit, acppage's
    DllGetClassObject): a reference is a pointer at the C ABI these exports have."""
    m = re.fullmatch(r"\s*const\s+([A-Za-z_]\w*)\s*&\s*", expansion)
    return f"pub type {name} = *const {m.group(1)};" if m else None


def tokenize(text: str) -> list[str]:
    out, pos = [], 0
    text = text.strip()
    while pos < len(text):
        m = _TOKEN.match(text, pos)
        if m is None or m.end() == pos:
            raise LowerError(f"cannot tokenize at {text[pos : pos + 20]!r}")
        if m.group("str"):
            lit = m.group("str")
            # A CHARACTER literal is an `int` in C (`sndAlias('S','D')`): its code point.
            ch = re.fullmatch(r"L?'(\\.|[^'\\])'", lit)
            if ch is None:
                raise LowerError("a string literal (needs the wide-literal seal)")
            body = ch.group(1)
            code = _C_ESCAPES.get(body[1]) if body.startswith("\\") else ord(body)
            if code is None:
                raise LowerError(f"character literal {lit}")
            out.append(str(code))
        else:
            out.append(m.group(m.lastgroup or 0))
        pos = m.end()
        while pos < len(text) and text[pos].isspace():
            pos += 1
    return out


@dataclass
class Node:
    kind: str  # num ident param cast call member index unary binary ternary comma assign null
    text: str = ""
    kids: tuple = ()
    ctype: tuple = ()  # cast: (rust type, is_pointer)
    # cast: a pointer to volatile (`(volatile ULONG *)p`), whose dereference must stay a
    # volatile access -- ndis.h's NdisReadRegisterUlong reads a device register that way.
    volatile: bool = False


_BIN_PREC = [
    ("||",),
    ("&&",),
    ("|",),
    ("^",),
    ("&",),
    ("==", "!="),
    ("<", ">", "<=", ">="),
    ("<<", ">>"),
    ("+", "-"),
    ("*", "/", "%"),
]


class _Parser:
    def __init__(self, toks: list[str], types: set[str]):
        self.t = toks
        self.i = 0
        self.types = types
        self.last_volatile = False  # set by type_name

    def peek(self, k: int = 0) -> str | None:
        j = self.i + k
        return self.t[j] if j < len(self.t) else None

    def take(self, want: str | None = None) -> str:
        tok = self.peek()
        if tok is None or (want is not None and tok != want):
            raise LowerError(f"expected {want!r}, found {tok!r}")
        self.i += 1
        return tok

    def parse(self) -> Node:
        if self.peek() == "{":
            return self.block()
        node = self.comma()
        if self.peek() is not None:
            raise LowerError(f"trailing tokens from {self.peek()!r}")
        return node

    def block(self) -> Node:
        """`{ T x; x.f = a; f(&x); }` -- commctrl.h's ListView_SetItemText family: local
        declarations of ONE plain type name, then expression statements. Anything else
        (control flow, initialisers, arrays) is out of the grammar."""
        self.take("{")
        stmts: list[Node] = []
        while self.peek() != "}":
            if self.peek() is None:
                raise LowerError("unterminated block")
            if (
                re.fullmatch(r"[A-Za-z_]\w*", self.peek() or "")
                and re.fullmatch(r"[A-Za-z_]\w*", self.peek(1) or "")
                and self.peek(2) == ";"
                and not (self.peek() or "").startswith("__rcv_p")
            ):
                ty, name = self.take(), self.take()
                self.take(";")
                stmts.append(Node("decl", name, ctype=(ty, False)))
                continue
            if self.peek() in ("if", "for", "while", "do", "switch", "return"):
                raise LowerError(f"`{self.peek()}` in a statement block")
            stmts.append(self.comma())
            self.take(";")
        self.take("}")
        if self.peek() is not None:
            raise LowerError(f"trailing tokens from {self.peek()!r}")
        return Node("block", kids=tuple(stmts))

    def comma(self) -> Node:
        node = self.assign()
        items = [node]
        while self.peek() == ",":
            self.take()
            items.append(self.assign())
        return node if len(items) == 1 else Node("comma", kids=tuple(items))

    def assign(self) -> Node:
        lhs = self.ternary()
        if self.peek() == "=":
            self.take()
            return Node("assign", kids=(lhs, self.assign()))
        return lhs

    def ternary(self) -> Node:
        cond = self.binary(0)
        if self.peek() == "?":
            self.take()
            a = self.comma()
            self.take(":")
            b = self.ternary()
            return Node("ternary", kids=(cond, a, b))
        return cond

    def binary(self, level: int) -> Node:
        if level == len(_BIN_PREC):
            return self.unary()
        node = self.binary(level + 1)
        while self.peek() in _BIN_PREC[level]:
            op = self.take()
            node = Node("binary", op, (node, self.binary(level + 1)))
        return node

    def _is_type_start(self) -> bool:
        """Is the `(` at the cursor a cast? A C keyword type, an island type, or the SHAPE
        `( Ident *... )` followed by an operand -- the base Win32 typedefs (WPARAM, DWORD)
        live in ntdll-sys, not the island, so the type set alone misses most casts."""
        tok = self.peek(1)
        if tok is None or tok.startswith("__rcv_p"):
            return False
        if tok in _C_TYPE_WORDS or tok in self.types:
            return True
        if not re.fullmatch(r"[A-Za-z_]\w*", tok):
            return False
        j = 2
        while self.peek(j) == "*":
            j += 1
        if self.peek(j) != ")":
            return False
        after = self.peek(j + 1)
        if after in ("-", "+", "*", "&"):
            # `(ULONG_PTR)-0x10000` (TVI_ROOT): binary-or-unary is ambiguous, so only a
            # name KNOWN to be a type makes it a cast -- or a `*` inside the parens, which
            # no expression can end with (`(LVITEMW *)&_ms_lvi`).
            return tok in _INT or j > 2
        if after == "(" and j == 2 and not re.fullmatch(r"[A-Z][A-Z0-9_]*", tok):
            # `(SendMessageW)(h, ..)` -- windowsx.h's SetWindowFont, a PARENTHESIZED CALLEE.
            # Win32 type names are ALL CAPS (WPARAM, HWND, DWORD_PTR); a mixed-case one with
            # no `*` before `)` followed by `(` is a function.
            return False
        return after is not None and (
            after in ("(", "~", "!") or re.fullmatch(r"\w+", after) is not None
        )

    def type_name(self) -> tuple[str, bool]:
        words: list[str] = []
        const = False
        self.last_volatile = False
        while self.peek() not in (None, "*", ")"):
            w = self.take()
            if w in ("const", "volatile"):
                const = const or w == "const"
                self.last_volatile = self.last_volatile or w == "volatile"
                continue
            if w in ("struct", "union", "enum"):
                continue
            words.append(w)
        stars = 0
        while self.peek() == "*":
            self.take()
            stars += 1
            while self.peek() in ("const", "volatile"):
                self.take()
        # Only the POINTEE's qualifier makes the access volatile (`volatile T *`).
        self.last_volatile = self.last_volatile and stars == 1
        base = _C_PRIM.get(tuple(words))
        if base is None:
            if len(words) != 1:
                raise LowerError(f"type {' '.join(words)!r}")
            base = words[0]
        if base == "c_void" and stars == 0:
            return "()", False
        ty = base
        for n in range(stars):
            ty = ("*const " if const and n == 0 else "*mut ") + ty
        return ty, stars > 0 or base in _POINTER_TYPES

    def unary(self) -> Node:
        tok = self.peek()
        if tok == "(" and self._is_type_start():
            self.take("(")
            ty = self.type_name()
            vol = self.last_volatile
            self.take(")")
            return Node("cast", kids=(self.unary(),), ctype=ty, volatile=vol)
        if tok in ("-", "~", "!", "*", "&", "+"):
            self.take()
            return Node("unary", tok, (self.unary(),))
        if tok == "sizeof":
            self.take()
            # Inside sizeof, `( Name *... )` is a type unless Name is a macro parameter: an
            # expansion's other bare names are typedefs (`sizeof(RECT)`).
            j = 2
            while self.peek(j) == "*":
                j += 1
            named = self.peek(1) is not None and re.fullmatch(r"[A-Za-z_]\w*", self.peek(1) or "")
            if self.peek() == "(" and (
                self._is_type_start()
                or (
                    named and not (self.peek(1) or "").startswith("__rcv_p") and self.peek(j) == ")"
                )
            ):
                self.take("(")
                ty = self.type_name()
                self.take(")")
                return Node("sizeof_type", ctype=ty)
            return Node("sizeof_expr", kids=(self.unary(),))
        return self.postfix()

    def postfix(self) -> Node:
        node = self.primary()
        while True:
            tok = self.peek()
            if tok == "(":
                self.take()
                args: list[Node] = []
                if self.peek() != ")":
                    args.append(self.assign())
                    while self.peek() == ",":
                        self.take()
                        args.append(self.assign())
                self.take(")")
                node = Node("call", kids=(node, *args))
            elif tok in ("->", "."):
                self.take()
                node = Node("member", tok, (node, Node("ident", self.take())))
            elif tok == "[":
                self.take()
                idx = self.comma()
                self.take("]")
                node = Node("index", kids=(node, idx))
            else:
                return node

    def primary(self) -> Node:
        tok = self.take()
        if tok == "(":
            node = self.comma()
            self.take(")")
            return node
        if re.fullmatch(r"0[xX][0-9a-fA-F]+[uUlL]*|\d+[uUlL]*", tok):
            return Node("num", re.sub(r"[uUlL]+$", "", tok))
        param = re.fullmatch(r"__rcv_p(\d+)__", tok)
        if param is not None:
            return Node("param", "$p" + param.group(1))
        if "__rcv_p" in tok:
            raise LowerError(f"token pasting onto a parameter ({tok})")
        if re.fullmatch(r"[A-Za-z_]\w*", tok):
            return Node("ident", tok)
        raise LowerError(f"unexpected token {tok!r}")


def _is_integer_literal(node: Node) -> bool:
    """`1`, `-1`, `-1L`, `(-(1))`: an integer constant expression of literals only."""
    if node.kind == "num":
        return not re.search(r"[.eE]", node.text) or node.text.lower().startswith("0x")
    if node.kind == "unary" and node.text in ("-", "+", "~"):
        return all(_is_integer_literal(k) for k in node.kids)
    return False


def _is_null(node: Node) -> bool:
    """`((void *)0)` -- the expansion of NULL."""
    return (
        node.kind == "cast"
        and node.ctype == ("*mut c_void", True)
        and node.kids[0].kind == "num"
        and node.kids[0].text == "0"
    )


# Integer types the expansions cast through: (width, signed). "P" is pointer width, which is
# 32 on i386 and 64 elsewhere -- a conversion whose outcome depends on it is refused.
_INT = {
    "c_char": (8, True),
    "i8": (8, True),
    "c_uchar": (8, False),
    "BYTE": (8, False),
    "UCHAR": (8, False),
    "CHAR": (8, True),
    "c_short": (16, True),
    "SHORT": (16, True),
    "c_ushort": (16, False),
    "WORD": (16, False),
    "USHORT": (16, False),
    "WCHAR": (16, False),
    "ATOM": (16, False),
    "LANGID": (16, False),
    "c_int": (32, True),
    "INT": (32, True),
    "BOOL": (32, True),
    "LONG": (32, True),
    "c_long": (32, True),
    "HRESULT": (32, True),
    "c_uint": (32, False),
    "UINT": (32, False),
    "DWORD": (32, False),
    "ULONG": (32, False),
    "c_ulong": (32, False),
    "COLORREF": (32, False),
    "LCID": (32, False),
    "i64": (64, True),
    "LONGLONG": (64, True),
    "u64": (64, False),
    "ULONGLONG": (64, False),
    "DWORD64": (64, False),
    "WPARAM": ("P", False),
    "UINT_PTR": ("P", False),
    "ULONG_PTR": ("P", False),
    "DWORD_PTR": ("P", False),
    "SIZE_T": ("P", False),
    "LPARAM": ("P", True),
    "LRESULT": ("P", True),
    "LONG_PTR": ("P", True),
    "INT_PTR": ("P", True),
}


#: `_emit`'s type for an expression cast to a pointer type: arithmetic on it is element-wise.
_PTR = "<pointer>"


def _promoted(ty: str | None) -> str | None:
    """C's integer promotion: anything narrower than int computes as int."""
    if ty is None or ty not in _INT:
        return ty
    width, _signed = _INT[ty]
    return "c_int" if width != "P" and width < 32 else ty


def _const_eval(node: Node, ptr_bits: int) -> int | None:
    """The C value of a numbers-and-casts expression at a given pointer width, as an
    unsigned bit pattern of that width, or None when it involves anything else."""
    mask = (1 << ptr_bits) - 1

    def width(ty: str, is_ptr: bool) -> tuple[int, bool] | None:
        if is_ptr or ty not in _INT:
            return (ptr_bits, False)  # a pointer or opaque handle: pointer-width bits
        w, signed = _INT[ty]
        return (ptr_bits if w == "P" else w, signed)

    def ev(n: Node) -> int | None:
        if n.kind == "num":
            return int(n.text, 0)
        if n.kind == "cast":
            if n.ctype[0] == "()":
                return None
            v = ev(n.kids[0])
            ws = width(*n.ctype)
            if v is None or ws is None:
                return None
            w, signed = ws
            v &= (1 << w) - 1
            if signed and v >> (w - 1):
                v -= 1 << w
            return v
        if n.kind == "unary" and n.text in ("-", "~", "+"):
            v = ev(n.kids[0])
            return None if v is None else {"-": -v, "~": ~v, "+": v}[n.text]
        if n.kind == "binary" and n.text in ("+", "-", "*", "&", "|", "^", "<<", ">>"):
            a, b = ev(n.kids[0]), ev(n.kids[1])
            if a is None or b is None:
                return None
            return {
                "+": a + b,
                "-": a - b,
                "*": a * b,
                "&": a & b,
                "|": a | b,
                "^": a ^ b,
                "<<": a << b,
                ">>": a >> b,
            }[n.text]
        return None

    v = ev(node)
    return None if v is None else v & mask


def _common(a: str, b: str) -> str:
    """C's usual arithmetic conversions over two PROMOTED integer types."""
    if a == b:
        return a
    (wa, sa), (wb, _sb) = _INT[a], _INT[b]
    if "P" in (wa, wb) and wa != wb:
        # 32 vs pointer-width: the answer differs between i386 and the 64-bit targets.
        other = wb if wa == "P" else wa
        if other == 64:
            return a if wa == 64 else b
        ptr, narrow = (a, b) if wa == "P" else (b, a)
        if not _INT[ptr][1] or _INT[narrow][1]:
            return ptr  # unsigned pointer-width, or both signed: the P type on every arch
        raise LowerError(f"`{a}` and `{b}` convert differently on i386 and 64-bit")
    if wa == wb:
        return a if not sa else b  # same rank: the unsigned one
    big, small = (a, b) if (wa == "P" or wa > wb) else (b, a)  # type: ignore[operator]
    if not _INT[big][1] or _INT[small][1]:
        return big
    return big  # signed and wider than the unsigned: represents every value


def _c_literal(text: str) -> str:
    """An unsuffixed C integer literal spelled with the type C gives it (LLP64): `int` if it
    fits, else for hex/octal `unsigned int`, else `long long` / `unsigned long long`."""
    v = int(text, 0)
    hexish = text.lower().startswith("0x") or (text.startswith("0") and len(text) > 1)
    if v <= 0x7FFFFFFF:
        return text
    if hexish and v <= 0xFFFFFFFF:
        return f"{text}u32"
    if v <= 0x7FFFFFFFFFFFFFFF:
        return f"{text}i64"
    return f"{text}u64"


def emit(node: Node) -> str:
    return _emit(node)[0]


def _cast_to(text: str, have: str | None, want: str) -> str:
    return text if have == want else f"(({text}) as {want})"


def _emit(node: Node) -> tuple[str, str | None]:
    """(Rust text, its C integer type when known). Unknown types are never guessed: rustc
    then reports a mismatch, which is the loud outcome."""
    k = node.kind
    if k == "num":
        return node.text, None
    if k in ("ident", "param"):
        return node.text, None
    if _is_null(node):
        return "core::ptr::null_mut::<c_void>()", None
    if k == "decl":
        # C leaves the rest of the struct uninitialised; zeroed is the defined superset.
        return f"let mut {node.text}: {node.ctype[0]} = core::mem::zeroed();", None
    if k == "block":
        body = " ".join(_emit(x)[0] if x.kind == "decl" else f"{_emit(x)[0]};" for x in node.kids)
        return f"{{ {body} }}", None
    if k == "sizeof_type":
        return f"core::mem::size_of::<{node.ctype[0]}>()", "SIZE_T"
    if k == "sizeof_expr":
        # `sizeof(a)` measures the OBJECT: an array is its whole storage, as size_of_val
        # of a reference to it is (never a decayed pointer).
        return f"core::mem::size_of_val(&{_emit(node.kids[0])[0]})", "SIZE_T"
    if k == "cast":
        ty, _ptr = node.ctype
        inner = node.kids[0]
        # A pointer cast directly under a non-pointer cast changes no bits; drop it.
        while not _ptr and inner.kind == "cast" and inner.ctype[1] and not _is_null(inner):
            inner = inner.kids[0]
        if ty == "()":
            return f"{{ let _ = {_emit(inner)[0]}; }}", None
        # A cast TO a callback typedef (`Option<fn>`) is a coercion in Rust -- `as` cannot
        # convert `Some(CompareFunc)` -- and a callback cast on to an integer goes through its
        # bits: taskmgr's ListView_SortItems is `(LPARAM)(PFNLVCOMPARE)(pfn)`. `Option<fn>`
        # is pointer-sized with None at 0, the C pointer's own representation.
        if ty in _OPTION_TYPES and not _ptr:
            if _is_integer_literal(inner) and not (inner.kind == "num" and inner.text == "0"):
                # An INTEGER cast to a callback (wininet's INTERNET_INVALID_STATUS_CALLBACK,
                # `(INTERNET_STATUS_CALLBACK)(-1L)`): a sentinel, never a fn. Rust has no
                # constant for it -- const evaluation rejects a fn pointer that points at no
                # fn -- so refuse: the call site compares the pointer's bits by hand.
                raise LowerError(
                    f"an integer cast to callback type {ty} is a sentinel with no Rust constant"
                )
            return f"({{ let __cb: {ty} = {_emit(inner)[0]}; __cb }})", ty
        itext, ity = _emit(inner)
        if ity in _OPTION_TYPES and ty not in _OPTION_TYPES:
            return f"((core::mem::transmute::<{ity}, usize>({itext})) as {ty})", (
                _PTR if _ptr else ty
            )
        if inner.kind == "num":
            # Give the literal its C type: Rust infers it from the cast TARGET, so
            # `0x80000001 as LONG` is an out-of-range i32 literal (HKEY_CURRENT_USER), where C
            # reads an `unsigned int` and then converts.
            return f"(({_c_literal(inner.text)}) as {ty})", (_PTR if _ptr else ty)
        return f"(({_emit(inner)[0]}) as {ty})", (_PTR if _ptr else ty)
    if k == "call":
        fn, *args = node.kids
        if fn.kind == "ident" and fn.text in ("__builtin_offsetof", "offsetof"):
            # Its first operand is a TYPE and its second a member PATH, neither an expression:
            # read as a call it came out as `__builtin_offsetof(size_of::<T>(), Member)`
            # (ntoskrnl_vista's REPARSE_DATA_BUFFER_HEADER_SIZE). The path may also run
            # through an anonymous union bindgen names `__bindgen_anon_N`. Refused, by name.
            raise LowerError("offsetof (a type and a member path, not a call)")
        params = (_SIGS.get(fn.text) or _tree_signature(fn.text)) if fn.kind == "ident" else []
        rendered = []
        for i, a in enumerate(args):
            # C's implicit argument conversion: an uncast operand gets the parameter's type.
            # Except a callback slot (`DLGPROC`, an `Option<fn>`): `as _` cannot convert
            # `Some(ApplyProc)` there, while the bare argument coerces (console's DialogBoxW).
            text = _emit(a)[0]
            callback = i < len(params) and params[i] in _OPTION_TYPES
            rendered.append(text if a.kind == "cast" or callback else f"(({text}) as _)")
        # A call through a struct member (`(*Block->StatusHandler)(..)`, ndis.h's
        # NdisMIndicateStatus; a WIDL `This->lpVtbl->Release(This)`): bindgen types every
        # function-pointer field Option<fn>, and C calling a NULL one is undefined, so
        # unwrap_unchecked -- never `*` (E0614), never a bare `.Release(` method lookup
        # (E0599), never unwrap().
        target = fn.kids[0] if fn.kind == "unary" and fn.text == "*" else fn
        if target.kind == "member":
            return f"({_emit(target)[0]}).unwrap_unchecked()({', '.join(rendered)})", None
        return f"{_emit(fn)[0]}({', '.join(rendered)})", None
    if k == "member":
        base, field = node.kids
        # A parameter as the MEMBER (oleaut32's `V_UNION(A,B) ((A)->n1.n2.n3.B)`) is a field
        # name, which render_macro makes an `ident` fragment.
        param = re.fullmatch(r"__rcv_p(\d+)__", field.text)
        name = f"$p{param.group(1)}" if param else field.text
        if node.text == "->":
            return f"(*{_emit(base)[0]}).{name}", None
        return f"{_emit(base)[0]}.{name}", None
    if k == "index":
        base, ty = _emit(node.kids[0])
        if ty == _PTR:
            # C's `p[i]` on a POINTER is `*(p + i)`; Rust indexes only arrays and slices.
            # commctrl's DPA_FastGetPtr is `DPA_GetPtrPtr(hdpa)[i]`.
            return f"(*({base}).add(({_emit(node.kids[1])[0]}) as usize))", None
        return f"{base}[({_emit(node.kids[1])[0]}) as usize]", None
    if k == "unary":
        op, (x,) = node.text, node.kids
        text, ty = _emit(x)
        if op == "~":
            ty = _promoted(ty)
            return f"(!{_cast_to(text, _emit(x)[1], ty) if ty else text})", ty
        if op == "!":
            return f"((({text}) as isize == 0) as c_int)", "c_int"
        if op == "&":
            return f"core::ptr::addr_of_mut!({text})", None
        if op == "*":
            if x.kind == "cast" and x.volatile:
                return f"core::ptr::read_volatile({text})", None
            # Through a cast to `T **..`, what is read is still a pointer.
            deeper = x.kind == "cast" and x.ctype[0].startswith(("*mut *", "*const *"))
            return f"(*{text})", (_PTR if deeper else None)
        if ty is None and x.kind == "num":
            # A C integer literal is an `int` when it fits, else `unsigned int`: `-0x10000`
            # (TVI_ROOT) must negate an int, and Rust would infer the literal from the
            # enclosing cast instead (`cannot apply unary operator - to u64`).
            ty = "c_int" if int(x.text, 0) <= 0x7FFFFFFF else "c_uint"
            if op == "-" and ty == "c_uint":
                return f"(({text} as c_uint).wrapping_neg())", ty
            return f"({op}({text} as {ty}))", ty
        ty = _promoted(ty)
        return f"({op}{_cast_to(text, _emit(x)[1], ty) if ty else text})", ty
    if k == "binary":
        op, (a, b) = node.text, node.kids
        (ta, ya), (tb, yb) = _emit(a), _emit(b)
        if op in ("&&", "||"):
            # C: each operand is tested against zero, and the result is an int 0/1.
            return f"((((({ta}) as isize) != 0) {op} ((({tb}) as isize) != 0)) as c_int)", "c_int"
        if _PTR in (ya, yb) and op in ("+", "-"):
            # C pointer arithmetic: `(PUCHAR)p + n` steps n elements of the pointee, as
            # `wrapping_add` does (ndis's NDIS_GET_PACKET_STATUS: `p + NdisPacketOobOffset`).
            # pointer - pointer is a distance, not this; refused rather than guessed.
            if ya == _PTR and yb != _PTR:
                step = "wrapping_add" if op == "+" else "wrapping_sub"
                return f"({ta}).{step}(({tb}) as usize)", _PTR
            if yb == _PTR and ya != _PTR and op == "+":
                return f"({tb}).wrapping_add(({ta}) as usize)", _PTR
            raise LowerError("pointer - pointer")
        pa, pb = _promoted(ya), _promoted(yb)
        if op in ("==", "!=", "<", ">", "<=", ">="):
            if pa in _INT and pb in _INT:
                common = _common(pa, pb)
                ta, tb = _cast_to(ta, ya, common), _cast_to(tb, yb, common)
            return f"(({ta} {op} {tb}) as c_int)", "c_int"
        if op in ("<<", ">>"):
            # The result has the promoted LEFT type; Rust takes any integer on the right.
            return f"({_cast_to(ta, ya, pa) if pa else ta} {op} {tb})", pa
        if pa in _INT and pb in _INT:
            common = _common(pa, pb)
            return f"({_cast_to(ta, ya, common)} {op} {_cast_to(tb, yb, common)})", common
        known = pa or pb
        if known in _INT:
            # The other side is a literal or untyped: promote the typed side and let the
            # literal infer. A typed-vs-untyped mix that disagrees fails in rustc, loudly.
            ta = _cast_to(ta, ya, pa) if pa else ta
            tb = _cast_to(tb, yb, pb) if pb else tb
            return f"({ta} {op} {tb})", known
        return f"({ta} {op} {tb})", None
    if k == "ternary":
        c, a, b = node.kids
        return (
            f"(if ({_emit(c)[0]}) as isize != 0 {{ {_emit(a)[0]} }} else {{ {_emit(b)[0]} }})",
            None,
        )
    if k == "comma":
        *first, last = node.kids
        return "{ " + "".join(f"{_emit(x)[0]}; " for x in first) + _emit(last)[0] + " }", None
    if k == "assign":
        lhs, rhs = node.kids
        if (
            lhs.kind == "unary"
            and lhs.text == "*"
            and lhs.kids[0].kind == "cast"
            and lhs.kids[0].volatile
        ):
            ptr = _emit(lhs.kids[0])[0]
            return f"core::ptr::write_volatile({ptr}, (({_emit(rhs)[0]}) as _));", None
        return f"{{ {_emit(lhs)[0]} = (({_emit(rhs)[0]}) as _); }}", None
    raise LowerError(f"node {k}")


# Set by `load_signatures` for one crate: callee -> parameter type names, and the type
# names that are `Option<fn>` callbacks. Module state because `_emit` is recursive and pure
# otherwise; lower_for_crate loads it before lowering.
_SIGS: dict[str, list[str]] = {}
_OPTION_TYPES: set[str] = set()
_POINTER_TYPES: set[str] = set()
_FN_DECL = re.compile(r"\bpub fn ([A-Za-z_]\w*)\s*\(([^)]*)\)")
_TYPE_DECL = re.compile(r"^pub type (\w+) = ([^;]+);", re.M)


_TREE_SIGS: dict[str, list[str]] = {}
_CRATE_SRC: list[Path] = []


def _tree_signature(name: str) -> list[str]:
    """Parameter types of a function the crate does not declare YET, from any other crate's
    bindgen island. `port macros` runs before the last `port resolve`, so runonce's prelude had
    no DialogBoxParamW when its DialogBox was lowered: the DLGPROC slot got `as _`, which
    cannot convert a function item (E0605). The island is the header's transcript, so the
    first declaration found is the header's."""
    if name in _TREE_SIGS:
        return _TREE_SIGS[name]
    params: list[str] = []
    if _CRATE_SRC:
        needle = f"pub fn {name}("
        for island in sorted(_CRATE_SRC[0].parents[1].glob("*/src/types_i386.rs")):
            text = island.read_text(errors="replace")
            i = text.find(needle)
            if i < 0:
                continue
            m = _FN_DECL.match(text, i)
            if m:
                params = [x.split(":", 1)[1].strip() for x in m.group(2).split(",") if ":" in x]
            break
    _TREE_SIGS[name] = params
    return params


def load_signatures(crate_src: Path) -> None:
    """Read the crate's extern declarations and the bindgen islands it sees."""
    sigs: dict[str, list[str]] = {}
    types: dict[str, str] = {}
    sys_island = crate_src.parents[1] / "ntdll-sys" / "src" / "bindgen_amd64.rs"
    for p in [*sorted(crate_src.glob("*.rs")), sys_island]:
        if not p.is_file():
            continue
        text = p.read_text(errors="replace")
        for m in _FN_DECL.finditer(text):
            params = [x.split(":", 1)[1].strip() for x in m.group(2).split(",") if ":" in x]
            sigs.setdefault(m.group(1), params)
        for m in _TYPE_DECL.finditer(text):
            types.setdefault(m.group(1), " ".join(m.group(2).split()))
    options = {n for n, v in types.items() if v.startswith(("::core::option::Option<", "Option<"))}
    changed = True
    while changed:  # aliases of callback types (`FONTENUMPROCW = OLDFONTENUMPROCW`)
        new = {n for n, v in types.items() if v in options and n not in options}
        options |= new
        changed = bool(new)
    _SIGS.clear()
    _SIGS.update(sigs)
    _TREE_SIGS.clear()
    _CRATE_SRC[:] = [crate_src]
    _OPTION_TYPES.clear()
    _OPTION_TYPES.update(options)
    # Pointer typedefs (`PUCHAR = *mut UCHAR`), aliases followed: a cast to one is a pointer
    # cast, so `(PUCHAR)p + n` is pointer arithmetic, which a bare `+` is not in Rust.
    pointers = {n for n, v in types.items() if v.startswith(("*mut ", "*const "))}
    changed = True
    while changed:
        new = {n for n, v in types.items() if v in pointers and n not in pointers}
        pointers |= new
        changed = bool(new)
    _POINTER_TYPES.clear()
    _POINTER_TYPES.update(pointers)


def lower_expansion(expansion: str, types: set[str]) -> str:
    # A C++ TU expands windowsx.h's macros to `::SendMessageW(..)`: the global-scope
    # qualifier names the same function, and Rust has no use for it.
    expansion = re.sub(r"(?<![\w:>])::(?=[A-Za-z_])", "", expansion)
    # A call through a struct member -- a WIDL vtable slot (`(This)->lpVtbl->Release(This)`,
    # dxdiag's IDirectSound8_Release) or an NDIS handler -- unwraps its Option<fn> in _emit.
    return emit(_Parser(tokenize(expansion), types).parse())


_DEFINE = re.compile(r"^#define ([A-Za-z_]\w*)\(([^)]*)\)", re.M)


_ALIAS = re.compile(r"^#define ([A-Za-z_]\w*) ([A-Za-z_]\w*)$", re.M)


def macro_params(dump: str) -> dict[str, list[str]]:
    """Function-like macros in a `-dM` dump -> their parameter lists, including an
    object-like ALIAS of one (`#define MAKEINTRESOURCE MAKEINTRESOURCEW`), which a call
    expands through exactly as if it were the target."""
    params = {
        m.group(1): [p.strip() for p in m.group(2).split(",") if p.strip()]
        for m in _DEFINE.finditer(dump)
    }
    aliases = dict(_ALIAS.findall(dump))
    for name in aliases:
        seen, target = {name}, aliases[name]
        while target in aliases and target not in seen:
            seen.add(target)
            target = aliases[target]
        if target in params and name not in params:
            params[name] = params[target]
    return params


def _preprocess(argv: list[str], source: Path, probe: str) -> str:
    """`argv` run with -E over `#include <source>` followed by `probe`."""
    with tempfile.TemporaryDirectory(prefix="rcv-sdkmacro-") as d:
        p = Path(d) / "probe.c"
        # `#line` gives the probe one name on every arch. Each arch's probe lives in its own
        # temp dir, so a macro that expands `__FILE__` read as "expands differently per arch"
        # (mmebuddy_debug.h's SND_ERR) when the real refusal is its string literal.
        p.write_text(f'#include "{source}"\n#line 1 "rcv-sdkmacro-probe.c"\n{probe}\n')
        args = [str(p) if a == str(source) else a for a in _without_outputs(argv)]
        if str(p) not in args:
            raise LowerError(f"compile command does not name {source}")
        res = subprocess.run([*args, "-E", "-P"], capture_output=True, text=True, check=False)
        if res.returncode != 0:
            raise LowerError("preprocessor failed: " + res.stderr.strip()[:300])
        return res.stdout


def _dump(argv: list[str]) -> str:
    res = subprocess.run(
        [*_without_outputs(argv), "-E", "-dM"], capture_output=True, text=True, check=False
    )
    if res.returncode != 0:
        raise LowerError("preprocessor failed: " + res.stderr.strip()[:300])
    return res.stdout


def expansions(repo: Path, source: Path, names: set[str]) -> dict[str, tuple[int, dict[str, str]]]:
    """name -> (arity, {arch: expansion}) for every name in `names` that is function-like on
    `source`'s compile command on EVERY arch. Variadic macros are skipped."""
    per_arch: dict[str, dict[str, str]] = {}
    arity: dict[str, int] = {}
    for arch in _ARCHES:
        argv = compile_argv(repo, arch, source)
        if argv is None:
            return {}
        params = macro_params(_dump(argv))
        here = {
            n: len(params[n])
            for n in names
            if n in params and not any(p.endswith("...") for p in params[n])
        }
        if not here:
            return {}
        probe = "\n".join(
            f"__RCV_BEGIN_{n}__ {n}({', '.join(f'__rcv_p{i}__' for i in range(k))}) __RCV_END__"
            for n, k in sorted(here.items())
        )
        out = _preprocess(argv, source, probe)
        for n, k in here.items():
            m = re.search(rf"__RCV_BEGIN_{n}__(.*?)__RCV_END__", out, re.S)
            if m is None:
                continue
            per_arch.setdefault(n, {})[arch] = " ".join(m.group(1).split())
            arity[n] = k
    return {n: (arity[n], ex) for n, ex in per_arch.items() if len(ex) == len(_ARCHES)}


_TYPE_ITEM = re.compile(r"^pub (?:type|struct|union|enum) (\w+)", re.M)
_SYS_GLOB = re.compile(r"^pub use (\w+)_sys(?:::\w+)*::\*;", re.M)


def island_types(crate_src: Path) -> set[str]:
    """Type names the crate's island sees: its own, plus every `-sys` crate it glob-imports.

    A user-mode island takes `LPWSTR` from `pub use ntdll_sys::nt::*;`. Without those, a cast
    to it is not a cast to the parser: commctrl's `LPSTR_TEXTCALLBACKW ((LPWSTR)-1L)` read
    as `(LPWSTR) - 1L`, a subtraction, and taskmgr/regedit got no constant at all."""
    out: set[str] = set()
    sys_crates: set[str] = set()
    for p in crate_src.glob("types_*.rs"):
        text = p.read_text(errors="replace")
        out |= set(_TYPE_ITEM.findall(text))
        sys_crates |= set(_SYS_GLOB.findall(text))
    for name in sorted(sys_crates):
        src = crate_src.parents[1] / f"{name.replace('_', '-')}-sys" / "src"
        for p in (src / "bindgen_amd64.rs", src / "bindgen_common.rs"):
            if p.is_file():
                out |= set(_TYPE_ITEM.findall(p.read_text(errors="replace")))
    return out


def render_macro(name: str, arity: int, body: str) -> str:
    fields = set(re.findall(r"\.\$p(\d+)\b", body))
    for i in fields:
        if len(re.findall(rf"\$p{i}\b", body)) != len(re.findall(rf"\.\$p{i}\b", body)):
            raise LowerError(f"parameter {i} is both a field name and an expression")
    pat = ", ".join(f"$p{i}:{'ident' if str(i) in fields else 'expr'}" for i in range(arity))
    pat = f"{pat} $(,)?" if pat else ""
    return f"#[macro_export] macro_rules! {name} {{ ({pat}) => {{ {body} }}; }}"


BEGIN = (
    "// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion "
    "(generated -- re-run `rustconv port macros`) ===="
)
END = "// ==== end SDK header macros ===="


def render_items(macros: dict[str, str], items: dict[str, str] | None = None) -> dict[str, str]:
    """Each item as it stands in the block, `gen:`-stamped (generated_items.py)."""
    return {n: generated_items.stamp(t) for n, t in {**(items or {}), **macros}.items()}


def render_block(macros: dict[str, str], items: dict[str, str] | None = None) -> str:
    items = items or {}
    if not macros and not items:
        return ""
    stamped = render_items(macros, items)
    lines = [BEGIN, *(stamped[n] for n in sorted(macros)), *(stamped[n] for n in sorted(items))]
    if macros:
        lines.append(f"pub use crate::{{{', '.join(sorted(macros))}}};")
    lines.append(END)
    return "\n".join(lines) + "\n"


_BLOCK_ITEM = re.compile(
    r"macro_rules!\s+(\w+)|^pub\s+(?:unsafe\s+)?(?:fn|const|type|static(?:\s+mut)?)\s+(\w+)", re.M
)


# `#[inline]`, `#[cfg(target_arch = "x86")]` -- possibly carrying the item's `gen:` stamp.
_BARE_ATTR = re.compile(r"#\[[^\]]*\]\s*(?://.*)?")


def split_block(body: str) -> dict[str, str]:
    """A block body back into {name: item text}, the inverse of render_items.

    An item starts at a line opening with `#[` or `pub ` unless the line before is a bare
    attribute; a COM wrapper's per-arch `#[cfg]` copies share a name and are one item. The
    trailing `pub use crate::{...}` is the block's own, not an item.
    """
    chunks: list[list[str]] = []
    for line in body.split("\n"):
        if not line.strip() or line.startswith("pub use crate::{"):
            continue
        bare_attr = bool(chunks) and _BARE_ATTR.fullmatch(chunks[-1][-1]) is not None
        if chunks and (bare_attr or not line.startswith(("#[", "pub "))):
            chunks[-1].append(line)
        else:
            chunks.append([line])
    out: dict[str, str] = {}
    for chunk in chunks:
        item = "\n".join(chunk)
        m = _BLOCK_ITEM.search(item)
        if m:
            name = m.group(1) or m.group(2)
            out[name] = f"{out[name]}\n{item}" if name in out else item
    return out


def reexport(name: str, item: str) -> str:
    """What a `#[macro_export]` item moved out of the block needs: its own `pub use`."""
    return f"pub use crate::{name};\n" if "macro_rules!" in item else ""


def write_block(prelude: Path, block: str) -> bool:
    text = prelude.read_text()
    pat = re.compile(re.escape(BEGIN) + r".*?" + re.escape(END) + r"\n", re.S)
    new = (
        pat.sub(lambda _m: block, text) if pat.search(text) else text.rstrip("\n") + "\n\n" + block
    )
    if new != text:
        prelude.write_text(new)
        return True
    return False


def existing(prelude: Path) -> set[str]:
    text = prelude.read_text() if prelude.is_file() else ""
    m = re.search(re.escape(BEGIN) + r"(.*?)" + re.escape(END), text, re.S)
    return set(re.findall(r"macro_rules! (\w+)", m.group(1))) if m else set()


def outside_block_names(prelude: Path) -> set[str]:
    """Items the prelude defines OUTSIDE every `// ==== ... ====` generated block."""
    text = prelude.read_text() if prelude.is_file() else ""
    text = re.sub(r"(?ms)^// ==== (?!end).*?^// ==== end[^\n]*\n", "", text)
    return set(
        re.findall(
            r"^\s*(?:#\[macro_export\]\s*)?(?:pub(?:\([^)]*\))?\s+)?"
            r"(?:const|static(?:\s+mut)?|type|fn|struct|union)\s+([A-Za-z_]\w*)",
            text,
            re.M,
        )
    ) | set(re.findall(r"macro_rules!\s*([A-Za-z_]\w*)", text))


def outside_sdk_block_names(prelude: Path) -> set[str]:
    """Items the prelude defines anywhere but THIS block -- other generated blocks included.

    ndis: after a replay `port macros` ran before `port resolve`, so the island's
    `NDIS_STATUS_*` were briefly unresolved and got lowered here; resolve then put them in
    its facts block, and because block items are re-wanted on every run, the duplicates
    (E0428) never went away. A name another block owns is not this block's to keep.
    """
    text = prelude.read_text() if prelude.is_file() else ""
    start, end = text.find(BEGIN), text.find(END)
    if start >= 0 and end > start:
        text = text[:start] + text[end + len(END) :]
    return set(
        re.findall(
            r"^\s*(?:#\[macro_export\]\s*)?(?:pub(?:\([^)]*\))?\s+)?"
            r"(?:const|static(?:\s+mut)?|type|fn|struct|union)\s+([A-Za-z_]\w*)",
            text,
            re.M,
        )
    ) | set(re.findall(r"macro_rules!\s*([A-Za-z_]\w*)", text))


def existing_items(prelude: Path) -> dict[str, set[str]]:
    """The block's object-like items: {"type": names, "value": names}."""
    text = prelude.read_text() if prelude.is_file() else ""
    m = re.search(re.escape(BEGIN) + r"(.*?)" + re.escape(END), text, re.S)
    body = m.group(1) if m else ""
    return {
        "type": set(re.findall(r"^pub type (\w+)", body, re.M)),
        "value": set(re.findall(r"^pub const (\w+)", body, re.M)),
        "fn": set(re.findall(r"^pub unsafe fn (\w+)", body, re.M)),
    }


_ISLAND_ARCH = {"i386": "x86", "amd64": "x86_64", "arm64": "aarch64"}


def _render_wrapper(
    name: str,
    iface: str,
    method: str,
    sig: tuple[list[tuple[str, str]], str | None] | None,
    cfg: str = "",
) -> str:
    assert sig is not None
    args, ret = sig
    params = ", ".join([f"This: *mut {iface}", *(f"{n}: {t}" for n, t in args)])
    call = ", ".join(["This", *(n for n, _ in args)])
    return (
        f"{cfg}#[inline]\npub unsafe fn {name}({params})"
        + (f" -> {ret}" if ret else "")
        + f" {{ ((*(*This).lpVtbl).{method}.unwrap_unchecked())({call}) }}"
        + " // widl C wrapper"
    )


def com_wrappers(crate_src: Path, wanted: set[str]) -> dict[str, str]:
    """WIDL's C wrappers (`IFoo_Method(This, ...)`) the crate calls, from the island's Vtbl.

    Under `__WINESRC__` widl headers define WIDL_C_INLINE_WRAPPERS, so `IFoo_Method` is a
    `static inline` function -- no macro for the preprocessor lowering to find, and no DLL
    exports it -- and a dozen crates carry hand copies of IUnknown_Release & co. The body is
    always `This->lpVtbl->Method(This, ...)`; it is generated with tools/gen_commacros.py's
    parser and body (`unwrap_unchecked`: a NULL slot faults at the call, as in C). A method
    whose callback parameter's ABI differs per arch gets one `#[cfg(target_arch)]` copy each.
    """
    try:
        from tools.gen_commacros import parse_vtbls
    except ImportError:
        import sys

        sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
        from tools.gen_commacros import parse_vtbls

    per_arch: dict[str, dict[str, dict[str, tuple[list[tuple[str, str]], str | None]]]] = {}
    for arch in _ISLAND_ARCH:
        p = crate_src / f"types_{arch}.rs"
        if not p.is_file():
            return {}
        per_arch[arch] = {
            iface: {m: (args, ret) for m, args, ret in methods}
            for iface, methods in parse_vtbls(p.read_text(errors="replace")).items()
        }
    out: dict[str, str] = {}
    for name in sorted(wanted):
        parts = name.split("_")
        for cut in range(1, len(parts)):
            iface, method = "_".join(parts[:cut]), "_".join(parts[cut:])
            sigs = {a: v.get(iface, {}).get(method) for a, v in per_arch.items()}
            if any(sig is None for sig in sigs.values()):
                continue

            if len({repr(sig) for sig in sigs.values()}) == 1:
                out[name] = _render_wrapper(name, iface, method, next(iter(sigs.values())))
            else:
                out[name] = "\n".join(
                    _render_wrapper(
                        name, iface, method, sig, f'#[cfg(target_arch = "{_ISLAND_ARCH[a]}")]\n'
                    )
                    for a, sig in sorted(sigs.items())
                )
            break
    return out


_OBJ_DEFINE = re.compile(r"^#define ([A-Za-z_]\w*) (.+)$", re.M)


def object_items(
    repo: Path, tus: list[Path], crate_src: Path, values: set[str], types_wanted: set[str]
) -> tuple[dict[str, str], dict[str, str]]:
    """Object-like SDK macros the crate uses as a TYPE or a pointer-typed VALUE.

    - `#define LV_ITEM LVITEMW` used as a type -> `pub type LV_ITEM = LVITEMW;` (the A/W
      switch for types; resolve covers it for functions only).
    - `#define TVI_ROOT ((HTREEITEM)(ULONG_PTR)-0x10000)` used as a value -> a `pub const`
      of the expression's outermost cast type. Plain integer constants are resolve's
      (clang-evaluated facts), so only a cast-topped expression is taken here.
    Every arch must agree; anything else is refused by name.
    """
    island = island_types(crate_src)
    out: dict[str, str] = {}
    refused: dict[str, str] = {}
    left_v, left_t = set(values), set(types_wanted)
    for tu in tus:
        if not (left_v or left_t):
            break
        dumps: dict[str, dict[str, str]] = {}
        for arch in _ARCHES:
            argv = compile_argv(repo, arch, tu)
            if argv is None:
                dumps = {}
                break
            dumps[arch] = dict(_OBJ_DEFINE.findall(_dump(argv)))
        if not dumps:
            continue
        want = sorted(n for n in left_v | left_t if all(n in d for d in dumps.values()))
        if not want:
            continue
        probes: dict[str, set[str]] = {}
        for arch in _ARCHES:
            argv = compile_argv(repo, arch, tu)
            assert argv is not None
            probe = "\n".join(f"__RCV_BEGIN_{n}__ {n} __RCV_END__" for n in want)
            text = _preprocess(argv, tu, probe)
            for n in want:
                m = re.search(rf"__RCV_BEGIN_{n}__(.*?)__RCV_END__", text, re.S)
                if m:
                    probes.setdefault(n, set()).add(" ".join(m.group(1).split()))
        for n in want:
            as_type = n in left_t
            left_v.discard(n)
            left_t.discard(n)
            ex = probes.get(n, set())
            if len(ex) != 1:
                refused[n] = "expands differently per arch" if ex else "no expansion"
                continue
            expansion = ex.pop()
            # A type alias through the A/W switch (`LV_ITEM` -> `LVITEM` ->
            # `__MINGW_NAME_AW(LVITEM)` -> `LVITEMW`): the expansion is the final name. A
            # value use of a type (`PROPSHEETHEADER` where the model dropped its sizeof)
            # gets the alias too, so the sizeof pass can see it is a type.
            # ...but a C type KEYWORD is not a Rust name: `#define VOID void` became
            # `pub type VOID = void;` (audiosrv). Those take the type parser below.
            if (
                re.fullmatch(r"[A-Za-z_]\w*", expansion)
                and expansion not in _C_TYPE_WORDS
                and (as_type or expansion in island)
            ):
                out[n] = f"pub type {n} = {expansion};"
                continue
            ref = cxx_ref_alias(n, expansion)
            if as_type and ref is not None:
                out[n] = ref
                continue
            if as_type:
                # `#define REFIID const IID* const` -- a C type NAME, not a single identifier.
                try:
                    tp = _Parser(tokenize(expansion), island)
                    rust_ty, _ptr = tp.type_name()
                    if tp.peek() is not None or rust_ty == "()":
                        raise LowerError("not a whole type")
                    out[n] = f"pub type {n} = {rust_ty};"
                except LowerError:
                    refused[n] = f"not a type alias: `{expansion[:80]}`"
                continue
            wide = wide_literal(expansion)
            if wide is not None:
                units = ", ".join(f"0x{u:04x}" for u in wide)
                out[n] = (
                    f"pub const {n}: *const u16 = (&[{units}u16]).as_ptr(); // {expansion[:100]}"
                )
                continue
            try:
                node = _Parser(tokenize(expansion), island).parse()
            except LowerError as e:
                refused[n] = f"{e} -- in `{expansion[:120]}`"
                continue
            while node.kind == "comma" and len(node.kids) == 1:
                node = node.kids[0]
            if node.kind != "cast" or node.ctype[0] == "()":
                continue  # an integer constant: resolve's facts own it
            # A pointer-typed constant built only from numbers and casts (RT_GROUP_ICON =
            # MAKEINTRESOURCE((ULONG_PTR)RT_ICON + 11)) casts a POINTER back to an integer on
            # the way, which const evaluation refuses. Fold it to its value when the 32- and
            # 64-bit evaluations agree (a resource id), else keep the expression.
            if node.ctype[1] or node.ctype[0] not in _INT:
                v32, v64 = _const_eval(node, 32), _const_eval(node, 64)
                if v32 is not None and v32 == v64:
                    out[n] = f"pub const {n}: {node.ctype[0]} = {v32:#x}usize as {node.ctype[0]};"
                    continue
            try:
                out[n] = f"pub const {n}: {node.ctype[0]} = {emit(node)};"
            except LowerError as e:
                refused[n] = f"{e} -- in `{expansion[:120]}`"
    return out, refused


def object_expansions(repo: Path, source: Path, names: set[str]) -> dict[str, str]:
    """`NAME` -> its full expansion on `source`'s compile command, for each name that is an
    object-like macro expanding identically on EVERY arch. Reaches through the function-like
    layers a `#define X Y` table cannot (`StringCchCopy` is `__MINGW_NAME_AW(StringCchCopy)`).
    """
    names = set(names)
    per: dict[str, set[str]] = {}
    for arch in _ARCHES:
        argv = compile_argv(repo, arch, source)
        if argv is None:
            return {}
        defined = dict(_OBJ_DEFINE.findall(_dump(argv)))
        # A name that is a macro on only SOME arches is not an alias: winuser.h makes
        # `SetWindowLongPtrW` `SetWindowLongW` on i386 alone, and probing only the arches
        # where it is defined collected one expansion and called it unanimous -- notepad's
        # 64-bit window-proc subclassing went through the 32-bit SetWindowLongW.
        names = {n for n in names if n in defined}
        want = sorted(names)
        if not want:
            return {}
        probe = "\n".join(f"__RCV_BEGIN_{n}__ {n} __RCV_END__" for n in want)
        text = _preprocess(argv, source, probe)
        for n in want:
            m = re.search(rf"__RCV_BEGIN_{n}__(.*?)__RCV_END__", text, re.S)
            per.setdefault(n, set()).add(" ".join(m.group(1).split()) if m else "")
    return {n: ex.pop() for n, ex in per.items() if n in names and len(ex) == 1 and "" not in ex}


_C_ESCAPES = {"n": 10, "t": 9, "r": 13, "0": 0, "\\": 92, '"': 34, "'": 39, "a": 7, "b": 8}


def wide_literal(expansion: str) -> list[int] | None:
    """`L"a" L"b"` (adjacent pieces concatenate) -> UTF-16 units with the NUL, else None."""
    pieces = re.findall(r'L"((?:[^"\\]|\\.)*)"', expansion)
    if not pieces or re.sub(r'L"(?:[^"\\]|\\.)*"', "", expansion).strip():
        return None
    out: list[int] = []
    for piece in pieces:
        i = 0
        while i < len(piece):
            ch = piece[i]
            if ch != "\\":
                out.extend(
                    int.from_bytes(ch.encode("utf-16-le")[k : k + 2], "little")
                    for k in range(0, len(ch.encode("utf-16-le")), 2)
                )
                i += 1
                continue
            nxt = piece[i + 1]
            if nxt == "x":
                m = re.match(r"[0-9a-fA-F]+", piece[i + 2 :])
                if m is None:
                    return None
                out.append(int(m.group(0), 16))
                i += 2 + len(m.group(0))
            elif nxt in _C_ESCAPES:
                out.append(_C_ESCAPES[nxt])
                i += 2
            else:
                return None
    return [*out, 0]


def widening_macros(repo: Path, tus: list[Path], names: set[str]) -> set[str]:
    """Which of `names` are C's TEXT/_T: `NAME("x")` expands to `L"x"` on every arch."""
    if not names:
        return set()
    # Per NAME, across TUs: `_T` exists only in the TU that includes <tchar.h> (main's
    # mouse.c), and stopping at the first TU that preprocesses at all missed it.
    # One probe PER NAME: a candidate of another arity (cmd's two-argument
    # `__WINE_IS_DEBUG_ON`) makes `NAME("rcv")` a preprocessor error, and in a shared probe
    # that error failed every TU for every name -- `_T` was never widened (725 call sites).
    widening: set[str] = set()
    for n in sorted(names):
        for tu in tus:
            seen: set[str] = set()
            ok = True
            for arch in _ARCHES:
                argv = compile_argv(repo, arch, tu)
                if argv is None:
                    ok = False
                    break
                try:
                    text = _preprocess(argv, tu, f'__RCV_BEGIN__ {n}("rcv") __RCV_END__')
                except LowerError:
                    ok = False
                    break
                m = re.search(r"__RCV_BEGIN__(.*?)__RCV_END__", text, re.S)
                seen.add(m.group(1).strip() if m else "")
            # Keep looking past a TU where NAME is simply undefined (`_T` exists only where
            # <tchar.h> is included -- main's mouse.c).
            if ok and seen == {'L"rcv"'}:
                widening.add(n)
                break
    return widening


_STR_CALL = r'"((?:[^"\\]|\\.)*)"'


def widen_call_sites(rust: str, names: set[str]) -> tuple[str, int]:
    """`TEXT("abc")` -> a NUL-terminated UTF-16 array pointer, `TEXT('0')` -> `('0' as u16)`.

    The argument is the Rust string literal the model kept; it is re-read with C's escapes,
    which Rust's string escapes match for everything these macros carry."""
    count = 0
    for n in sorted(names):

        def repl_s(m: re.Match[str], n: str = n) -> str:
            nonlocal count
            units = wide_literal(f'L"{m.group(1)}"')
            if units is None:
                return m.group(0)
            count += 1
            body = ", ".join(f"0x{u:04x}" for u in units)
            # The note is a block comment: a literal holding `*/` (cmd's set.c
            # `_T("*/%+-&^|")`) closed it early and left the rest as an unterminated string.
            note = m.group(1)[:60].replace("*/", "*\\/").replace("/*", "/\\*")
            return f'(&[{body}u16]).as_ptr() /* {n} "{note}" */'

        rust = re.sub(rf"\b{re.escape(n)}!?\(\s*{_STR_CALL}\s*\)", repl_s, rust)

        def repl_c(m: re.Match[str]) -> str:
            nonlocal count
            count += 1
            return f"({m.group(1)} as u16)"

        rust = re.sub(rf"\b{re.escape(n)}!?\(\s*('(?:[^'\\]|\\.)')\s*\)", repl_c, rust)
        # What is left wraps an expression the model ALREADY made wide (`TEXT([..u16].as_ptr())`,
        # `TEXT(API_NAME.as_ptr())` -- usrmgr): TEXT of a wide value is the identity.
        while (m := re.search(rf"(?<![\w:]){re.escape(n)}!?\(", rust)) is not None:
            depth, i = 0, m.end() - 1
            for i in range(m.end() - 1, len(rust)):
                if rust[i] == "(":
                    depth += 1
                elif rust[i] == ")":
                    depth -= 1
                    if depth == 0:
                        break
            if depth != 0:
                break
            rust = rust[: m.start()] + "(" + rust[m.end() : i] + ")" + rust[i + 1 :]
            count += 1
    return rust, count


def lower_for_crate(
    repo: Path, tus: list[Path], crate_src: Path, wanted: set[str]
) -> tuple[dict[str, str], dict[str, str]]:
    """(lowered name -> macro_rules text, refused name -> reason)."""
    load_signatures(crate_src)
    types = island_types(crate_src)
    lowered: dict[str, str] = {}
    refused: dict[str, str] = {}
    left = set(wanted)
    for tu in tus:
        if not left:
            break
        try:
            found = expansions(repo, tu, left)
        except LowerError as e:
            refused.update({n: f"{tu.name}: {e}" for n in left})
            continue
        for name, (arity, per_arch) in found.items():
            left.discard(name)
            refused.pop(name, None)
            if len(set(per_arch.values())) != 1:
                refused[name] = "expands differently per arch: " + "; ".join(
                    f"{a}: {x[:80]}" for a, x in sorted(per_arch.items())
                )
                continue
            try:
                lowered[name] = render_macro(
                    name, arity, lower_expansion(next(iter(per_arch.values())), types)
                )
            except LowerError as e:
                refused[name] = f"{e} -- in `{next(iter(per_arch.values()))[:140]}`"
    return lowered, refused
