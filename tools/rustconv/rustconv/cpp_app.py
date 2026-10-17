"""Render a C++ application module's classes as Rust, deterministically.

The split is the one the COM path established: the model translates method BODIES, and
never invents layout, dispatch or lifetime. Here the source of truth is `cpp_model` (clang's
own view of the classes) and the target is what the C++ compiler itself builds:

* a class is a `#[repr(C)]` struct; each base class is an embedded field `__baseK` (the
  primary base first, so a pointer to the derived object is a pointer to it), and a class
  that starts a polymorphic hierarchy carries `__vtbl` first;
* each virtual method is a slot in that vtable, filled per concrete class with the final
  overrider, through a `this`-adjusting thunk for a secondary base (`offset_of!`);
* a virtual method `m` is a generated dispatcher, `m(&mut self, ..)`, that calls through the
  vtable; its translated body is `m__body`. A non-virtual method is its body;
* the destructor body becomes `Drop`, so Rust destroys bases and members after it, as C++
  does; `delete` of a polymorphic object goes through a vtable `__delete` slot;
* constructors are `__ctor_N(&mut self, ..)`; `T__make_N(..) -> T`, `T__new_N(..) -> *mut T`
  and `T__zeroed() -> T` are the free functions the lowered text (`cpp_lower`) calls.

Bodies come from translating SYNTHETIC TUs: every lowered method as a free function
`RET <Class>__<name>(Class *this_, ..)`, every lowered free function as itself. `assemble`
then moves each translated `<Class>__<name>` into `impl <Class>` as `name(&mut self, ..)`
with `let this_: *mut Self = self;` — a textual move, so the model's own types survive and
nothing here re-states a signature the model already wrote.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

from . import cpp_templates
from .cpp_core import c_type_to_rust
from .cpp_model import (
    ATL_IMPL_METHODS,
    ClassModel,
    Method,
    ModuleModel,
    atl_base_alias,
)


class CppAppError(RuntimeError):
    pass


# ---------------------------------------------------------------------------------------
# types
# ---------------------------------------------------------------------------------------

_FNPTR_RE = re.compile(
    r"^(?P<ret>.+?)\s*\(\s*(?P<cc>__stdcall|__cdecl|CALLBACK|WINAPI)?\s*\*\s*\)\s*\((?P<args>.*)\)$"
)


def short(name: str) -> str:
    """A class's short name; ATL's window templates are their CWindow (cpp_model)."""
    return atl_base_alias(name)


def atl_template(cls: ClassModel) -> str | None:
    """`CWindowImpl` / `CDialogImpl` / `CPropertyPageImpl` when the class derives from one."""
    for b in cls.bases:
        t = b.name.split("<")[0].split("::")[-1].strip()
        if t in ("CWindowImpl", "CDialogImpl", "CPropertyPageImpl"):
            return t
    return None


def rust_type(c: str, model: ModuleModel, *, field: bool = False) -> str:
    """C++ declarator -> Rust type. Module classes and enums by their short name; a
    reference FIELD is a pointer (a struct cannot hold a borrow), a reference parameter or
    return is `&mut T` / `&T`."""
    c = " ".join(c.split())
    for ns in _namespaces(model):
        c = c.replace(f"{ns}::", "")
    m = _FNPTR_RE.match(c)
    if m:
        args = [a.strip() for a in _split(m.group("args")) if a.strip() not in ("", "void")]
        abi = "system" if m.group("cc") in ("__stdcall", "CALLBACK", "WINAPI") else "C"
        ret = m.group("ret").strip()
        r = "" if ret == "void" else f" -> {rust_type(ret, model)}"
        return (
            f'Option<unsafe extern "{abi}" fn({", ".join(rust_type(a, model) for a in args)}){r}>'
        )
    arr = re.match(r"^(.*?)\s*\[(.+)\]$", c)
    if arr:
        return f"[{rust_type(arr.group(1), model)}; {arr.group(2).strip()} as usize]"
    if c == "bool":
        return "bool"
    if c in ("float",):
        return "f32"
    if c in ("double",):
        return "f64"
    if "<" in c:
        # A field or global of a registered template type (`CicArray<GUID> m_array`).
        tb = cpp_templates.template_base(c, lambda a: rust_type(a, model))
        if tb is not None:
            return tb.rust_type
    t = c_type_to_rust(c)
    if field and t.startswith("&"):
        t = (
            "*"
            + ("const " if t.startswith("&") and not t.startswith("&mut") else "mut ")
            + t.lstrip("&").removeprefix("mut ").strip()
        )
    return t


def _split(text: str) -> list[str]:
    out, depth, cur = [], 0, []
    for ch in text:
        if ch in "(<[":
            depth += 1
        elif ch in ")>]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(cur))
            cur = []
            continue
        cur.append(ch)
    out.append("".join(cur))
    return out


def _namespaces(model: ModuleModel) -> list[str]:
    ns = set()
    for q in model.classes:
        parts = q.split("::")
        for i in range(1, len(parts)):
            ns.add("::".join(parts[:i]))
    return sorted(ns, key=len, reverse=True)


# ---------------------------------------------------------------------------------------
# hierarchy
# ---------------------------------------------------------------------------------------


@dataclass
class ComSlot:
    name: str
    params: list[tuple[str, str]]  # after `This`
    ret: str  # "" for unit
    optional: bool = True  # bindgen wraps each slot in Option<>; a hand-written table may not


_VTBL_RE = re.compile(r"pub struct (\w+)Vtbl \{(.*?)\n\}", re.S)
_VTBL_FIELD_RE = re.compile(
    r"pub (\w+):\s*((?:::core::option::)?Option<)?\s*unsafe extern \"system\" fn\s*"
    r"\((.*?)\)\s*(?:->\s*([^,>]+(?:<[^>]*>)?))?\s*>?,",
    re.S,
)


def parse_com_vtbls(island: str) -> dict[str, list[ComSlot]]:
    """`IFooVtbl` structs from the island (WIDL's flattened vtable: IUnknown first)."""
    out: dict[str, list[ComSlot]] = {}
    for m in _VTBL_RE.finditer(island):
        slots = []
        for f in _VTBL_FIELD_RE.finditer(m.group(2)):
            params = [p.strip() for p in _split(f.group(3)) if p.strip()]
            pairs = [tuple(x.strip() for x in p.split(":", 1)) for p in params[1:]]
            slots.append(
                ComSlot(
                    f.group(1),
                    [(a, b) for a, b in pairs],
                    (f.group(4) or "").strip(),
                    optional=bool(f.group(2)),
                )
            )
        if slots:
            out[m.group(1)] = slots
    return out


@dataclass
class Slot:
    owner: str  # the class whose vtable struct declares this slot (short name)
    method: Method


class Hierarchy:
    def __init__(self, model: ModuleModel, com: dict[str, list[ComSlot]] | None = None) -> None:
        self.model = model
        self.by_short: dict[str, ClassModel] = {short(q): c for q, c in model.classes.items()}
        # COM interfaces the island lays out (`IFooVtbl`), by interface name.
        self.com = com or {}
        # Every type name the island defines (render_classes sets it): _nested_names.
        self.island_names: frozenset[str] = frozenset()
        self.dep_classes: frozenset[str] = frozenset()
        self.hand_methods: dict[str, set[str]] = {}

    def com_bases(self, cls: ClassModel) -> list[tuple[int, str]]:
        return [
            (i, short(b.name))
            for i, b in enumerate(cls.bases)
            if short(b.name) not in self.by_short and short(b.name) in self.com
        ]

    def implements_com(self, cls: ClassModel, m: Method) -> bool:
        """`m` is the class's implementation of a COM interface method: dispatched through
        that interface's vtable (the island's IFooVtbl), never a slot of a vtable of ours."""
        for _, iface in self.com_bases(cls):
            if any(slot.name == m.name for slot in self.com[iface]):
                return True
        return any(self.implements_com(b, m) for _, b in self.module_bases(cls))

    def module_bases(self, cls: ClassModel) -> list[tuple[int, ClassModel]]:
        return [
            (i, self.by_short[short(b.name)])
            for i, b in enumerate(cls.bases)
            if short(b.name) in self.by_short
        ]

    def has_own_vtbl(self, cls: ClassModel) -> bool:
        """A class owns `__vtbl` when it declares virtuals and its primary base does not
        already carry a vtable pointer it can share."""
        if not cls.polymorphic:
            return False
        if self.com_bases(cls) and not self.new_slots(cls):
            # Every virtual is an interface method: the interface's lpVtbl IS the vptr.
            return False
        if cls.bases and short(cls.bases[0].name) in self.by_short:
            return not self.by_short[short(cls.bases[0].name)].polymorphic
        if cls.bases:
            tb = cpp_templates.template_base(cls.bases[0].name, lambda a: a)
            if tb is not None and tb.polymorphic:
                # The vptr lives inside the template base (CUIFObjectArray : CicArray<T>), so
                # a second one here would shift the whole layout by a pointer. Sharing it
                # with the base's table is only right while the class adds nothing that table
                # would have to describe: no slots of its own and no fields to destroy.
                # (Dropped by the per-instantiation rework and restored: cicero's
                # regeneration gave CUIFObjectArray its own `__vtbl`.)
                if self.new_slots(cls) or any(not f.static for f in cls.fields):
                    raise CppAppError(
                        f"{cls.name} extends polymorphic template {cls.bases[0].name} with "
                        "virtuals or fields; its vtable would need the template's slots"
                    )
                return False
        return not cls.bases or short(cls.bases[0].name) not in self.by_short

    def vtbl_root(self, cls: ClassModel) -> ClassModel:
        """The class whose `__vtbl` field this class's primary vtable pointer is."""
        while not self.has_own_vtbl(cls):
            cls = self.by_short[short(cls.bases[0].name)]
        return cls

    def is_virtual(self, cls: ClassModel, m: Method) -> bool:
        if self.implements_com(cls, m):
            return False
        if m.virtual:
            return True
        return any(self._virtual_above(b, m) for _, b in self.module_bases(cls))

    def _virtual_above(self, b: ClassModel, m: Method) -> bool:
        """`m` overrides a virtual of `b` or of ANY class above it. A base that does not
        redeclare the method passes the question up: RubberTool : SmoothDrawTool : ToolBase
        with OnSpecialTweak declared only in ToolBase made the unit ask for a plain
        `RubberTool__OnSpecialTweak` while the vtable thunk called `OnSpecialTweak__body`."""
        bm = next(
            (x for x in b.methods if x.name == m.name and x.signature_key == m.signature_key),
            None,
        )
        if bm is not None:
            return self.is_virtual(b, bm)
        return any(self._virtual_above(bb, m) for _, bb in self.module_bases(b))

    def slots(self, root: ClassModel) -> list[Method]:
        """The vtable of a root class: its virtual methods in declaration order, the
        destructor first if virtual (as `__delete`)."""
        return [m for m in root.methods if m.virtual and m.kind != "dtor"]

    def final_overrider(self, cls: ClassModel, m: Method) -> tuple[ClassModel, Method] | None:
        for x in cls.methods:
            if x.name == m.name and x.signature_key == m.signature_key and not x.pure:
                return cls, x
        for _, b in self.module_bases(cls):
            r = self.final_overrider(b, m)
            if r is not None:
                return r
        return None

    def subobjects(self, cls: ClassModel, path: str = "") -> list[tuple[str, ClassModel]]:
        """Every polymorphic base subobject that owns a vtable pointer, by field path."""
        out = []
        if self.has_own_vtbl(cls):
            out.append((path, cls))
        for i, b in self.module_bases(cls):
            out.extend(self.subobjects(b, f"{path}__base{i}."))
        return out

    def primary(self, cls: ClassModel) -> ClassModel | None:
        if cls.bases and short(cls.bases[0].name) in self.by_short:
            return self.by_short[short(cls.bases[0].name)]
        return None

    def overrides_base(self, cls: ClassModel, m: Method) -> bool:
        return any(
            any(
                bm.name == m.name and bm.signature_key == m.signature_key and self.is_virtual(b, bm)
                for bm in b.methods
            )
            or self.overrides_base(b, m)
            for _, b in self.module_bases(cls)
        )

    def new_slots(self, cls: ClassModel) -> list[Method]:
        """Virtual methods this class INTRODUCES (overriding nothing in a base), in order."""
        return [
            m
            for m in cls.methods
            if m.virtual
            and m.kind not in ("ctor", "dtor")
            and not m.static
            and not self.overrides_base(cls, m)
            and not self.implements_com(cls, m)
        ]

    def owns_vt_type(self, cls: ClassModel) -> bool:
        """A class with its own vtable STRUCT: a root, or one that extends its primary
        base's table with slots of its own (mspaint's SmoothDrawTool adds OnDraw over
        ToolBase) -- the layout C++ compilers use: the derived table begins with the base's."""
        return cls.polymorphic and (self.has_own_vtbl(cls) or bool(self.new_slots(cls)))

    def vt_type(self, cls: ClassModel) -> ClassModel:
        while not self.owns_vt_type(cls):
            nxt = self.primary(cls)
            if nxt is None:
                raise CppAppError(f"{cls.qualified}: polymorphic with no vtable on its chain")
            cls = nxt
        return cls

    def chain(self, top: ClassModel) -> list[ClassModel]:
        """`top`, its primary base, that one's primary base... down to the vtable root."""
        out = [top]
        while not self.has_own_vtbl(out[-1]):
            nxt = self.primary(out[-1])
            if nxt is None:
                break
            out.append(nxt)
        return out

    def has_chain(self, cls: ClassModel) -> bool:
        """A vtable of OURS on this class's primary chain (a class whose only virtuals are
        COM interface methods has none: the interface's lpVtbl is its vptr)."""
        if self.has_own_vtbl(cls):
            return True
        p = self.primary(cls)
        return p is not None and p.polymorphic and self.has_chain(p)

    def tops(
        self, cls: ClassModel, path: str = "", own: bool = True
    ) -> list[tuple[str, ClassModel]]:
        """Every vtable-bearing subobject, as (path, the class whose primary chain it is):
        the object itself, then each NON-primary polymorphic base, recursively (a primary
        base shares its derived class's table)."""
        out = [(path, cls)] if own and self.has_chain(cls) else []
        for i, b in self.module_bases(cls):
            out.extend(self.tops(b, f"{path}__base{i}.", own=i != 0))
        return out

    def root_path(self, path: str, top: ClassModel) -> str:
        """The field path from the complete object to `top`'s `__vtbl` owner."""
        return path + "__base0." * (len(self.chain(top)) - 1)

    def virtual_dtor(self, cls: ClassModel) -> bool:
        if any(m.kind == "dtor" and m.virtual for m in cls.methods):
            return True
        return any(self.virtual_dtor(b) for _, b in self.module_bases(cls))


# ---------------------------------------------------------------------------------------
# rendering
# ---------------------------------------------------------------------------------------


def _params(m: Method, model: ModuleModel) -> list[tuple[str, str]]:
    # `_ident`: certmgr's `CStoreList(StoreType type)` reached its __make/__new wrappers bare.
    return [
        (_ident(p.name) if p.name else f"a{i}", rust_type(p.c_type, model))
        for i, p in enumerate(m.params)
    ]


def _ret(m: Method, model: ModuleModel) -> str:
    return "" if m.return_type in ("void", "") else f" -> {rust_type(m.return_type, model)}"


def drop_generated_statics(text: str, generated: set[str]) -> str:
    """Remove a translated unit's own definition of a static that cpp_classes.rs renders
    (aggregate_static): two definitions are E0428, and the model's copy is the one that
    guessed the field names."""
    out, i = [], 0
    for m in re.finditer(r"^pub static (?:mut )?(\w+)\s*:", text, re.M):
        if m.group(1) not in generated or m.start() < i:
            continue
        depth, j = 0, m.end()
        while j < len(text):
            c = text[j]
            depth += (c in "([{") - (c in ")]}")
            if c == ";" and depth == 0:
                break
            j += 1
        out.append(text[i : m.start()])
        i = j + 1
        if text[i : i + 1] == "\n":
            i += 1
    return "".join(out) + text[i:]


def render_classes(
    model: ModuleModel,
    island_structs: frozenset[str] = frozenset(),
    island: str = "",
    hand_methods: dict[str, set[str]] | None = None,
    hand_names: frozenset[str] = frozenset(),
) -> str:
    """`island_structs`: names the crate's bindgen island already lays out. A plain struct
    (no methods, no bases, no vtable) the island has is the island's -- its C layout is the
    authority, and a second definition makes every use of the name ambiguous (E0659).
    `island`: its text, for the COM interfaces a class implements (their IFooVtbl).
    `hand_methods` / `hand_names`: what the crate's hand-written modules already define --
    methods per class, and the names its prelude re-exports from them. The hand version wins:
    mmc_new's MDI frame needs ATL's `m_pfnSuperWindowProc` honoured (DefFrameProc /
    DefMDIChildProc), which the generic glue's DefWindowProc does not do, and its
    `CWndProcThunk` is a `{ proc, this }` pair, not the island's instruction bytes. Rendering
    both is E0592/E0252, and dropping the hand one silently loses the MDI behaviour."""
    h = Hierarchy(model, parse_com_vtbls(island))
    h.island_names = island_type_names(island)
    h.hand_methods = hand_methods or {}
    # Classes a path dependency owns (`pub use cicero_rust::cicero::cpp_classes::X;`, from
    # port's _dep_classes). Whether one drops is not knowable from here.
    h.dep_classes = frozenset(
        re.findall(r"^pub use \w+(?:::\w+)*::cpp_classes::(\w+);", island, re.M)
    )
    out: list[str] = [
        "//! The module's C++ classes: layout, vtables, dispatch, construction, destruction.",
        "//! GENERATED by rustconv.cpp_app from the clang class model (cpp_model) -- do not edit;",
        "//! regenerate. Method BODIES are translated and assembled into the `impl` blocks of the",
        "//! per-TU modules; nothing here re-states a signature the model wrote.",
        "#![allow(non_camel_case_types, non_snake_case, dead_code, unused_unsafe)]",
        "use crate::prelude::*;",
        "",
        _HELPERS,
        _allocator_helpers(model),
    ]
    for e in model.enums:
        if re.search(rf"^\s*pub type {re.escape(e.name)}\b", island, re.M):
            # The island has it (mspaint's HITTEST, as c_uint): an i32 copy here made every
            # prelude alias of its members a type error, 255 of them.
            continue
        ty = rust_type(e.underlying, model)
        out.append(f"pub type {e.name} = {ty};")
        for name, val in e.members:
            out.append(f"pub const {name}: {e.name} = {val};")
        out.append("")
    for cls in model.classes.values():
        out.append(_render_class(cls, h, model, cls.name in island_structs and _plain(cls)))
    if any(c.method("GetWndClassInfo") for c in model.classes.values() if not c.external):
        out.append(_START_WINDOW_PROC)
    out.append(_render_globals(model, h))
    out.append(_render_global_init(model, h))
    out.append(_render_island_glue(model, island, hand_names))
    return "\n".join(out) + "\n"


# atl_rust's own spellings of ATL types, re-exported by every C++ crate's prelude.
_ATL_RUST_PROVIDED = frozenset(
    {
        "CStringW",
        "CStringA",
        "CString",
        "CSimpleString",
        "CComPtr",
        "CHeapPtr",
        "CComHeapPtr",
        "CSimpleArray",
        "CSimpleMap",
        "CAtlList",
        "POSITION",
        "CLocalAllocator",
        "CCRTAllocator",
        "CPath",
        "CPathW",
        "CRegKey",
        # CStringW's own template machinery: a module's `CStringT<..>` spelling IS CStringW.
        "CStringT",
        "CSimpleStringT",
        "ChTraitsCRT",
        "StrTraitATL",
    }
)


# C++ namespaces whose types bindgen prefixes and whose members the module names bare.
_ISLAND_NAMESPACES = ("ATL", "Gdiplus")
# Never re-exported bare: a const named `Ok` would shadow Result::Ok across the crate.
_RUST_PRELUDE = frozenset({"Ok", "Err", "Some", "None", "Option", "Result", "Box", "Vec"})


def _model_words(model: ModuleModel) -> frozenset[str]:
    """Every identifier in the module's lowered bodies and declared types."""
    texts: list[str] = []
    for c in model.classes.values():
        texts += [f.c_type for f in c.fields]
        for m in c.methods:
            texts += [m.lowered or "", m.return_type, *(p.c_type for p in m.params)]
    texts += [f.lowered + f.signature for f in model.functions]
    texts += [g.c_type for g in model.globals]
    return frozenset(re.findall(r"[A-Za-z_]\w*", "\n".join(texts)))


def _render_island_glue(
    model: ModuleModel, island: str, hand_names: frozenset[str] = frozenset()
) -> str:
    """Two C++ spellings bindgen's island has no name for.

    A class-scoped anonymous enum (`class CUIFWindow { enum { USER_TIMER_ID = 0x5461 }; }`)
    is `CUIFWindow_USER_TIMER_ID` in the island, but the class's own methods say
    `USER_TIMER_ID`: re-export it bare when exactly one class has that member and nothing
    else owns the name. And a C++ struct base (`struct INPUTCONTEXTDX : INPUTCONTEXT`) is a
    `_base` field, while C++ reaches its members as the struct's own (`pIC->hCandInfo`):
    the struct derefs to it, as a class with an SDK base already does.
    """
    island_names = frozenset(re.findall(r"^\s*pub (?:type|struct|union|enum) (\w+)", island, re.M))
    lines: list[str] = []
    owners: dict[str, list[tuple[str, str]]] = {}
    for cls in model.classes:
        for m in re.finditer(rf"^pub const {re.escape(cls)}_(\w+): (\w+) = ", island, re.M):
            owners.setdefault(m.group(1), []).append((cls, m.group(2)))
    for name, found in sorted(owners.items()):
        if (
            len(found) == 1
            and name not in island_names
            and not re.search(rf"^pub const {re.escape(name)}\b", island, re.M)
        ):
            cls, ty = found[0]
            lines.append(f"pub const {name}: {ty} = {cls}_{name};")
    # The same for a TYPE nested in a class: member code names `FN_COMPARE`, bindgen emits
    # `CActiveLanguageProfileNotifySink_FN_COMPARE`. Aliased only when one class owns it.
    types: dict[str, list[str]] = {}
    for cls in model.classes:
        pat = rf"^pub (?:type|struct|union|enum) {re.escape(cls)}_(\w+)\b"
        for m in re.finditer(pat, island, re.M):
            types.setdefault(m.group(1), []).append(cls)
    rendered = {c.name for c in model.classes.values()}  # cppapp lays these out itself
    for name, found in sorted(types.items()):
        if len(found) == 1 and name not in island_names and name not in rendered:
            lines.append(f"pub type {name} = {found[0]}_{name};")
    # And a type in ATL's namespace: bindgen spells `ATL::CWndClassInfo` `ATL_CWndClassInfo`,
    # the module's own `GetWndClassInfo()` says `CWndClassInfo`. Only names the module uses
    # and atl_rust does not provide (its CStringW/CComPtr/... are the prelude's).
    # GDI+'s C++ API lives in `namespace Gdiplus` the same way (`Gdiplus_Status`), and so do
    # its enum constants (`Gdiplus_Unit_UnitPixel`, which the bodies say `UnitPixel`).
    used = _model_words(model)
    aliased: set[str] = set()
    for ns in _ISLAND_NAMESPACES:
        # Not a GENERIC one (`ATL_StrTraitATL<..>`): a bare alias of it is missing arguments.
        for m in re.finditer(rf"^pub (?:type|struct|union) {ns}_(\w+)\b(?!\s*<)", island, re.M):
            name = m.group(1)
            if (
                name in used
                and name not in island_names
                and name not in rendered
                and name not in _ATL_RUST_PROVIDED
                and name not in hand_names
                and name not in aliased
            ):
                aliased.add(name)
                lines.append(f"pub type {name} = {ns}_{name};")
        for m in re.finditer(rf"^pub const {ns}_(\w+?)_(\w+): ({ns}_\1) = ", island, re.M):
            name = m.group(2)
            if (
                name in used
                and name not in island_names
                and name not in _RUST_PRELUDE
                and name not in aliased
                and not re.search(rf"^pub const {re.escape(name)}\b", island, re.M)
            ):
                aliased.add(name)
                lines.append(f"pub const {name}: {m.group(3)} = {ns}_{m.group(1)}_{name};")
    for m in re.finditer(
        r"((?:^#\[[^\n]*\]\n)*)^pub struct (\w+) \{\n\s*pub _base: (\w+),", island, re.M
    ):
        attrs, name, base = m.groups()
        if name in model.classes:
            continue
        if "packed" in attrs:
            # A struct under `#pragma pack` (shlobj.h's IShellFolder & co): a reference to its
            # `_base` field is unaligned (E0793), so it gets no Deref -- the 36 such impls
            # were msctf's entire E0793 count.
            continue
        lines.append(
            f"impl core::ops::Deref for {name} {{ type Target = {base}; "
            f"fn deref(&self) -> &{base} {{ &self._base }} }}"
        )
        lines.append(
            f"impl core::ops::DerefMut for {name} {{ "
            f"fn deref_mut(&mut self) -> &mut {base} {{ &mut self._base }} }}"
        )
    return "\n".join(lines) + "\n" if lines else ""


def _plain(cls: ClassModel) -> bool:
    """A struct the island may lay out. Not one holding an ATL type by value: bindgen lays
    rapps' `CStringW DownloadFileName` out as its own `ATL_CStringW` and derives Copy over it;
    cppapp's layout uses atl_rust's CStringW, which owns and frees its buffer."""
    return (
        not cls.methods
        and not cls.bases
        and not cls.polymorphic
        and not any(not f.static and _ATL_OWNING.search(f.c_type) for f in cls.fields)
    )


def _render_class(
    cls: ClassModel, h: Hierarchy, model: ModuleModel, island_owned: bool = False
) -> str:
    n = cls.name
    # A class nothing destroys meaningfully is a C++ trivially-copyable value: Copy gives
    # Rust the implicit copy constructor's semantics, so `fmt = A44100_16BIT_STEREO` is a
    # copy rather than a move out of a static.
    # A base or by-value member that a DEPENDENCY's cppapp laid out (msctfime's
    # CDefCompFrameGripper : cicero's CUIFGripper) may own a destructor this module cannot see,
    # and is not Copy there; deriving Copy over it fails to compile. Leaving Copy off is safe.
    dep = h.dep_classes
    from_dep = any(short(b.name) in dep for b in cls.bases) or any(
        not f.static and short(f.c_type.replace("const", "").strip()) in dep for f in cls.fields
    )
    derive = "" if has_drop(cls, h) or from_dep else "#[derive(Clone, Copy)]\n"
    kw = "union" if cls.tag == "union" else "struct"
    lines = [f"// ---- {cls.qualified} ({cls.file})", f"{derive}#[repr(C)]", f"pub {kw} {n} {{"]
    if h.has_own_vtbl(cls):
        lines.append(f"    pub __vtbl: *const {n}__vtbl,")
    for i in _base_layout_order(cls, h):
        b = cls.bases[i]
        bn = short(b.name)
        if bn in h.by_short:
            lines.append(f"    pub __base{i}: {bn},")
        else:
            lines.append(f"    pub __base{i}: {_external_base(b.name, model)},")
    for f in cls.fields:
        if f.static:
            continue
        ty = _nested_names(
            rust_type(f.c_type, model, field=True), cls, h.island_names, frozenset(h.by_short)
        )
        lines.append(f"    pub {_ident(f.name)}: {ty},")
    lines.append("}")
    lines.append("")
    if island_owned:
        lines = [f"// ---- {cls.qualified}: laid out by the bindgen island (types_<arch>.rs)"]
    # vtable struct: a root's, or an extension of the primary base's
    if h.owns_vt_type(cls):
        root = h.vtbl_root(cls)
        lines.append("#[repr(C)]")
        lines.append(f"pub struct {n}__vtbl {{")
        if h.has_own_vtbl(cls):
            lines.append(f"    pub __delete: unsafe fn(*mut {n}),")
            lines.append(f"    pub __drop: unsafe fn(*mut {n}),")
        else:
            base_vt = h.vt_type(h.primary(cls))  # type: ignore[arg-type]
            lines.append(f"    pub __base: {base_vt.name}__vtbl,")
        for m in h.new_slots(cls):
            args = ", ".join([f"*mut {root.name}", *(t for _, t in _params(m, model))])
            lines.append(f"    pub {m.rust_name}: unsafe fn({args}){_ret(m, model)},")
        lines.append("}")
        lines.append("")
    # static members
    for f in cls.fields:
        if f.static:
            ty = rust_type(f.c_type, model, field=True)
            lines.append(
                f"pub static mut {n}__{f.name}: {ty} = {_static_member_init(cls, f, ty, model)};"
            )
    # vtable instances, one per vtable-bearing subobject (every class: abstract ones are never
    # built, but their constructors install their own table, as C++ does mid-construction)
    impl = [f"impl {n} {{"]
    vt_install = []
    for path, top in h.tops(cls):
        root = h.vtbl_root(top)
        rn = root.name
        rp = h.root_path(path, top)
        tag = rp.replace(".", "").replace("__base", "b") or "own"
        thunks = [
            f"unsafe fn {n}__vt_delete_{tag}(p: *mut {rn}) {{ let d = {_down(n, rp, 'p')}; "
            f"core::ptr::drop_in_place(d); cpp_free(d as *mut core::ffi::c_void); }}",
            f"unsafe fn {n}__vt_drop_{tag}(p: *mut {rn}) {{ "
            f"core::ptr::drop_in_place({_down(n, rp, 'p')}); }}",
        ]

        def slot_entries(
            x: ClassModel,
            rn: str = rn,
            rp: str = rp,
            tag: str = tag,
            thunks: list[str] = thunks,
        ) -> list[str]:
            out = []
            for m in h.new_slots(x):
                fo = h.final_overrider(cls, m)
                params = _params(m, model)
                args = ", ".join(f"a{i}" for i in range(len(params)))
                sig = ", ".join(
                    [f"p: *mut {rn}", *(f"a{i}: {t}" for i, (_, t) in enumerate(params))]
                )
                if fo is None:
                    body = f'cpp_pure_virtual_call("{cls.qualified}::{m.name}")'
                else:
                    owner, om = fo
                    opath = h_path(h, cls, owner)
                    via = "." + opath.rstrip(".") if opath else ""
                    body = f"(*{_down(n, rp, 'p')}){via}.{om.rust_name}__body({args})"
                thunk = f"{n}__vt_{tag}_{m.rust_name}"
                if any(t.startswith(f"unsafe fn {thunk}(") for t in thunks):
                    # A new slot that HIDES an inherited one of the same name: cicero's
                    # CUIFWindow adds `OnTimer(WPARAM)` over CUIFObject's `OnTimer()`. Each
                    # vtable struct holds its own field, but both thunks were one fn (E0428).
                    thunk += f"__{x.name}"
                thunks.append(f"unsafe fn {thunk}({sig}){_ret(m, model)} {{ {body} }}")
                out.append(f"{m.rust_name}: {thunk}")
            return out

        def literal(x: ClassModel, tag: str = tag) -> str:
            if h.has_own_vtbl(x):
                fields = [f"__delete: {n}__vt_delete_{tag}", f"__drop: {n}__vt_drop_{tag}"]
                return f"{x.name}__vtbl {{ {', '.join(fields + slot_entries(x))} }}"
            inner = literal(h.primary(x))  # type: ignore[arg-type]
            if not h.owns_vt_type(x):
                return inner
            fields = [f"__base: {inner}", *slot_entries(x)]
            return f"{x.name}__vtbl {{ {', '.join(fields)} }}"

        lit = literal(top)
        vt = h.vt_type(top).name
        lines.extend(thunks)
        lines.append(f"pub static {n}__VTBL_{tag}: {vt}__vtbl = {lit};")
        vt_install.append(
            f"        (*this).{rp}__vtbl = "
            f"&{n}__VTBL_{tag} as *const {vt}__vtbl as *const {rn}__vtbl;"
        )
    for k, iface in h.com_bases(cls):
        lines.extend(_com_vtable(cls, k, iface, h))
        vt_install.append(
            f"        (*this).__base{k}.lpVtbl = "
            f"&{n}__{iface}_VTBL as *const {iface}Vtbl as *mut {iface}Vtbl;"
        )
    impl.append(
        "    /// Point every vtable this object contains at this class's tables: what a C++"
    )
    impl.append("    /// constructor does after its bases are built and before its members are.")
    impl.append("    pub unsafe fn __init_vtables(&mut self) {")
    impl.append("        let this: *mut Self = self;")
    impl.extend(vt_install or ["        let _ = this;"])
    impl.append("    }")
    # dispatchers
    for m in cls.methods:
        if m.kind in ("ctor", "dtor") or m.static:
            continue
        if h.is_virtual(cls, m):
            rp, root, decl = _slot_home(h, cls, m)
            params = _params(m, model)
            sig = ", ".join(["&mut self", *(f"{a}: {t}" for a, t in params)])
            args = ", ".join(a for a, _ in params)
            sub = f"core::ptr::addr_of_mut!((*this).{rp.rstrip('.')})" if rp else "this"
            impl.append(
                f"    /// virtual `{m.name}`: dispatch through the vtable to the final overrider."
            )
            impl.append(f"    pub unsafe fn {m.rust_name}({sig}){_ret(m, model)} {{")
            impl.append("        let this: *mut Self = self;")
            impl.append(f"        let sub = {sub} as *mut {root.name};")
            impl.append(f"        let vt = (*sub).__vtbl as *const {decl.name}__vtbl;")
            impl.append(f"        ((*vt).{m.rust_name})(sub{', ' + args if args else ''})")
            impl.append("    }")
    impl.append("}")
    lines.extend(impl)
    lines.append("")
    if cls.com_map is not None:
        lines.append(_render_com_object_glue(cls, h, model))
    win = atl_window(cls, h)
    if win is not None:
        lines.append(_render_atl_glue(cls, h, model, win))
    elif _exe_module_base(cls) is not None:
        lines.append(_render_exe_module_glue(cls, model))
    elif cls.bases and not h.com_bases(cls):
        # C++ reaches a base's members as the class's own: an SDK struct base's fields
        # (CRect : tagRECT, `rc.top`), and a base CLASS's methods -- msctfime's
        # CDefCompFrameWindow calls CUIFWindow's `Move` through its own base
        # CCompFrameWindow. Deref to the primary base gives exactly that lookup, chained up
        # the hierarchy; a method the class defines itself still wins, as in C++. Virtual
        # ones stay correct either way: the base's dispatcher reads the vptr this object's
        # constructor installed.
        own = h.by_short.get(short(cls.bases[0].name))
        base = own.name if own is not None else _external_base(cls.bases[0].name, model)
        lines.append(
            f"impl core::ops::Deref for {n} {{ type Target = {base}; "
            f"fn deref(&self) -> &{base} {{ &self.__base0 }} }}"
        )
        lines.append(
            f"impl core::ops::DerefMut for {n} {{ "
            f"fn deref_mut(&mut self) -> &mut {base} {{ &mut self.__base0 }} }}"
        )
    # construction / destruction (locals are `__`-named: a ctor parameter `t` or `p`,
    # CRect(l, t, r, b), would shadow them)
    ctors = [m for m in cls.methods if m.kind == "ctor"]
    lines.append(f"pub unsafe fn {n}__zeroed() -> {n} {{ core::mem::zeroed() }}")
    if not ctors:
        lines.append(
            f"pub unsafe fn {n}__make_0() -> {n} {{ "
            f"let mut __obj = {n}__zeroed(); __obj.__ctor_0(); __obj }}"
        )
        lines.append(
            f"pub unsafe fn {n}__new_in_0(__p: *mut {n}) -> *mut {n} "
            f"{{ if __p.is_null() {{ return __p; }} (*__p).__ctor_0(); __p }}"
        )
        lines.append(
            f"pub unsafe fn {n}__new_0() -> *mut {n} {{ {n}__new_in_0(cpp_alloc::<{n}>()) }}"
        )
    for i, m in enumerate(ctors):
        if m.body is None:
            # Declared, never defined: SelectionModel's private `SelectionModel(const
            # SelectionModel&);` is the pre-C++11 "not copyable". Nothing to construct with.
            continue
        params = _params(m, model)
        sig = ", ".join(f"{a}: {t}" for a, t in params)
        args = ", ".join(a for a, _ in params)
        lines.append(
            f"pub unsafe fn {n}__make_{i}({sig}) -> {n} {{ let mut __obj = {n}__zeroed(); "
            f"__obj.__ctor_{i}({args}); __obj }}"
        )
        lines.append(
            f"pub unsafe fn {n}__new_in_{i}(__p: *mut {n}{', ' if sig else ''}{sig}) -> *mut {n} "
            f"{{ if __p.is_null() {{ return __p; }} (*__p).__ctor_{i}({args}); __p }}"
        )
        lines.append(
            f"pub unsafe fn {n}__new_{i}({sig}) -> *mut {n} {{ "
            f"{n}__new_in_{i}(cpp_alloc::<{n}>(){', ' if args else ''}{args}) }}"
        )
    if h.virtual_dtor(cls):
        root_path, root = _vtbl_path(h, cls)
        field = root_path.rstrip(".")
        sub = f"core::ptr::addr_of_mut!((*p).{field})" if field else "p"
        lines.append(
            f"pub unsafe fn {n}__delete(p: *mut {n}) {{ if p.is_null() {{ return; }} "
            f"let sub = {sub} as *mut {root.name}; ((*(*sub).__vtbl).__delete)(sub); }}"
        )
    else:
        lines.append(
            f"pub unsafe fn {n}__delete(p: *mut {n}) {{ if p.is_null() {{ return; }} "
            f"core::ptr::drop_in_place(p); cpp_free(p as *mut core::ffi::c_void); }}"
        )
    if any(m.kind == "dtor" and m.body and not _empty_body(m) for m in cls.methods):
        lines.append(f"impl Drop for {n} {{")
        if cls.polymorphic:
            # A C++ destructor runs with the object's vtables pointing at ITS OWN class:
            # CPointerArray's destructor calls OnRemoveItem through DPA_DestroyCallback and
            # gets the BASE version, which deletes nothing. Dispatching to the derived
            # CUiCollection::OnRemoveItem (`delete ptr`) freed rapps' CAppInfoDisplay a
            # second time after ~CApplicationView had. Each base's own Drop resets in turn.
            lines.append(
                "    fn drop(&mut self) { unsafe { self.__init_vtables(); self.__dtor__body() } }"
            )
        else:
            lines.append("    fn drop(&mut self) { unsafe { self.__dtor__body() } }")
        lines.append("}")
    lines.append("")
    return "\n".join(lines)


def _empty_body(m: Method) -> bool:
    text = re.sub(r"//[^\n]*|/\*.*?\*/", "", m.lowered or "", flags=re.S)
    return not text.replace("{", "").replace("}", "").strip()


# atl_rust types that own something (they implement Drop): a member of one makes its class
# non-Copy, exactly as a member with a user destructor does in C++.
_ATL_OWNING = re.compile(
    r"\b(?:CStringW|CAtlStringW|CString|CAtlString|CComPtr|CComHeapPtr|CHeapPtr|"
    r"CSimpleArray|CSimpleMap|CAtlList|CRegKey)\b"
    # atl_rust::stl's containers (drwtsn32's `std::string ModuleName;`)
    r"|\bstd::(?:w?string|vector|map)\b"
)


def has_drop(cls: ClassModel, h: Hierarchy) -> bool:
    """A destructor with a body that does something, here or in any base or member."""
    if any(m.kind == "dtor" and m.body and not _empty_body(m) for m in cls.methods):
        return True
    if any(not f.static and _ATL_OWNING.search(f.c_type) for f in cls.fields):
        return True
    if any(_ATL_OWNING.match(b.name) for b in cls.bases):  # rapps' CLocalPtr<T> : CHeapPtr
        return True
    if any(has_drop(b, h) for _, b in h.module_bases(cls)):
        return True
    # A hand-written generic that frees what it owns (cicero's CicArray<T>): not Copy.
    if any(cpp_templates.drops(b.name) for b in cls.bases):
        return True
    if any(not f.static and cpp_templates.drops(f.c_type) for f in cls.fields):
        return True
    return any(
        has_drop(h.by_short[t], h)
        for f in cls.fields
        if not f.static and (t := short(f.c_type.replace("const", "").strip())) in h.by_short
    )


def h_path(h: Hierarchy, frm: ClassModel, to: ClassModel) -> str:
    if frm is to:
        return ""
    for i, b in h.module_bases(frm):
        sub = h_path(h, b, to)
        if sub is not None and (b is to or sub):
            return f"__base{i}." + sub
        if b is to:
            return f"__base{i}."
    return ""


def _com_vtable(cls: ClassModel, k: int, iface: str, h: Hierarchy) -> list[str]:
    """The island's IFooVtbl filled with this class's implementations, each behind a thunk
    that turns `This` (the interface subobject) back into the object -- C++'s adjustor."""
    n = cls.name
    entries, thunks = [], []

    def pick(methods: list) -> Method | None:
        # Overloads across interfaces (mmc_new's CConsoleWnd: IConsole2::Expand(item, bExpand)
        # beside IConsoleNameSpace2::Expand(item)): the slot's arity chooses.
        cands = [m for m in methods if m.name == slot.name]
        return next((m for m in cands if len(m.params) == len(slot.params)), None) or next(
            iter(cands), None
        )

    for slot in h.com[iface]:
        impl = pick(cls.methods)
        owner = cls
        if impl is None:
            for _, b in h.module_bases(cls):
                impl = pick(b.methods)
                if impl is not None:
                    owner = b
                    break
        if impl is None:
            raise CppAppError(f"{cls.qualified} implements {iface} but defines no {slot.name}")
        if len(impl.params) > len(slot.params):
            raise CppAppError(
                f"{cls.qualified}: {owner.name}::{slot.name} takes {len(impl.params)} "
                f"parameter(s), but {iface}'s slot has {len(slot.params)}"
            )
        via = h_path(h, cls, owner).rstrip(".")
        target = f"(*{_down(n, f'__base{k}.', 'This')})" + (f".{via}" if via else "")
        sig = ", ".join([f"This: *mut {iface}", *(f"{a}: {t}" for a, t in slot.params)])
        args = ", ".join(
            _ref_arg(a, p) for (a, _), p in zip(slot.params, _pad(impl.params, slot), strict=True)
        )
        ret = f" -> {slot.ret}" if slot.ret else ""
        name = f"{n}__{iface}_{slot.name}"
        thunks.append(
            f'unsafe extern "system" fn {name}({sig}){ret} {{ {target}.{impl.rust_name}({args}) }}'
        )
        entries.append(f"{slot.name}: Some({name})" if slot.optional else f"{slot.name}: {name}")
    return [
        *thunks,
        f"pub static {n}__{iface}_VTBL: {iface}Vtbl = {iface}Vtbl {{ {', '.join(entries)} }};",
    ]


def _pad(params: list, slot) -> list:
    return [*params, *([None] * (len(slot.params) - len(params)))]


def _ref_arg(arg: str, param) -> str:
    """The slot hands a POINTER where the C++ implementation takes a REFERENCE (`REFIID riid`,
    `const T&`): reborrow it. `&*p` also coerces back to `*const T`, so the thunk is right
    whichever spelling the implementation's Rust signature uses."""
    c = " ".join((param.c_type if param is not None else "").split())
    if c.endswith("&") or re.fullmatch(r"REF[A-Z]+", c):
        mutable = c.endswith("&") and not c.startswith("const ")
        return f"&mut *{arg}" if mutable else f"&*{arg}"
    return arg


def _slot_home(h: Hierarchy, cls: ClassModel, m: Method) -> tuple[str, ClassModel, ClassModel]:
    """Where slot `m` lives for `cls`: (field path to the vtable owner, the root class, the
    class whose vtable struct DECLARES the slot -- the type to read the table as)."""
    for path, top in h.tops(cls):
        for x in h.chain(top):
            if any(s.name == m.name and s.signature_key == m.signature_key for s in h.new_slots(x)):
                return h.root_path(path, top), h.vtbl_root(top), x
    raise CppAppError(f"{cls.qualified}::{m.name} is virtual but no vtable declares it")


def _vtbl_path(h: Hierarchy, cls: ClassModel) -> tuple[str, ClassModel]:
    tops = h.tops(cls)
    if not tops:
        raise CppAppError(f"{cls.qualified} has a virtual destructor but no vtable")
    path, top = tops[0]
    return h.root_path(path, top), h.vtbl_root(top)


def _down(n: str, path: str, p: str) -> str:
    """A pointer to the subobject at `path` back to the complete `n` (the C++ thunk). A
    nested path is a nested `offset_of!` (stable since Rust 1.82)."""
    if not path:
        return f"({p} as *mut {n})"
    field = path.rstrip(".")
    return f"(({p} as *mut u8).sub(core::mem::offset_of!({n}, {field})) as *mut {n})"


def _ident(name: str) -> str:
    return f"r#{name}" if name in _RUST_KEYWORDS else name


_RUST_KEYWORDS = {
    "type",
    "match",
    "fn",
    "mod",
    "use",
    "ref",
    "move",
    "loop",
    "impl",
    "self",
    "box",
    "in",
    "where",
}


# ---- ATL window glue ------------------------------------------------------------------

_CMD = "msg == WM_COMMAND as u32"
_NFY = "msg == WM_NOTIFY as u32"
_LO = "(w & 0xFFFF) as u32"  # LOWORD(wParam)
_HI = "((w >> 16) & 0xFFFF) as u32"  # HIWORD(wParam)
_ID_FROM = "(*(l as *const NMHDR)).idFrom as usize"
_CODE = "(*(l as *const NMHDR)).code as i64"

# atlwin.h's message-map macros: (condition over msg/w/l, handler call arguments). The handler
# returns LRESULT and takes `BOOL& bHandled` last; ATL sets bHandled = TRUE before the call and
# the entry answers only if the handler left it TRUE.
_MSG_ENTRY = {
    "MESSAGE_HANDLER": (1, "msg == ({0}) as u32", "msg as _, w as _, l as _"),
    "MESSAGE_RANGE_HANDLER": (
        2,
        "msg >= ({0}) as u32 && msg <= ({1}) as u32",
        "msg as _, w as _, l as _",
    ),
    "COMMAND_HANDLER": (
        2,
        f"{_CMD} && {_LO} == ({{0}}) as u32 && {_HI} == ({{1}}) as u32",
        "((w >> 16) & 0xFFFF) as _, (w & 0xFFFF) as _, l as _",
    ),
    "COMMAND_ID_HANDLER": (
        1,
        f"{_CMD} && {_LO} == ({{0}}) as u32",
        "((w >> 16) & 0xFFFF) as _, (w & 0xFFFF) as _, l as _",
    ),
    "COMMAND_CODE_HANDLER": (
        1,
        f"{_CMD} && {_HI} == ({{0}}) as u32",
        "((w >> 16) & 0xFFFF) as _, (w & 0xFFFF) as _, l as _",
    ),
    "COMMAND_RANGE_HANDLER": (
        2,
        f"{_CMD} && {_LO} >= ({{0}}) as u32 && {_LO} <= ({{1}}) as u32",
        "((w >> 16) & 0xFFFF) as _, (w & 0xFFFF) as _, l as _",
    ),
    "NOTIFY_HANDLER": (
        2,
        f"{_NFY} && {_ID_FROM} == ({{0}}) as usize && {_CODE} == ({{1}}) as i64",
        "w as _, l as _",
    ),
    "NOTIFY_ID_HANDLER": (
        1,
        f"{_NFY} && {_ID_FROM} == ({{0}}) as usize",
        "w as _, l as _",
    ),
    "NOTIFY_CODE_HANDLER": (
        1,
        f"{_NFY} && {_CODE} == ({{0}}) as i64",
        "w as _, l as _",
    ),
}


def _handler(cls: ClassModel, name: str, arity: int, h: Hierarchy | None = None) -> str:
    """The receiver path and method a message-map entry calls, relative to `self`."""
    if "." in name:
        # `MESSAGE_HANDLER(WM_SIZE, m_Splitter.OnSize)` (certmgr): a member object's handler.
        member, _, meth = name.partition(".")
        fld = next((f for f in cls.fields if f.name == member and not f.static), None)
        owner = h.by_short.get(short(fld.c_type.strip())) if (fld and h) else None
        if owner is None:
            raise CppAppError(
                f"{cls.qualified}: message map names `{name}`, and `{member}` is not a member "
                "of a module class"
            )
        return f"{member}.{_handler(owner, meth, arity)}"
    cands = [m for m in cls.methods if m.name == name]
    pick = next((m for m in cands if len(m.params) == arity), cands[0] if cands else None)
    if pick is None:
        raise CppAppError(f"{cls.qualified}: message map names `{name}`, which the class lacks")
    return pick.rust_name + ("__body" if pick.virtual else "")


def _render_msg_map(cls: ClassModel, h: Hierarchy) -> list[str]:
    out = ["        let mut handled: BOOL = 1;"]
    for e in cls.msg_map or []:
        if e.macro in _MSG_ENTRY:
            nargs, cond, call = _MSG_ENTRY[e.macro]
            keys, func = e.args[:nargs], e.args[nargs]
            arity = call.count(" as _") + 1
            cond = cond.format(*keys)
            out.append(f"        // {e.macro}({', '.join(e.args)})")
            out.append(f"        if {cond} {{")
            out.append("            handled = 1;")
            out.append(
                f"            *result = self.{_handler(cls, func, arity, h)}({call}, &mut handled) "
                "as isize;"
            )
            out.append("            if handled != 0 { return true; }")
            out.append("        }")
        elif e.macro == "CHAIN_MSG_MAP" and _template_name(e.args[0]) == "CPropertyPageImpl":
            # The base's own map: MESSAGE_HANDLER(WM_NOTIFY, OnNotify), dispatching PSN_* to
            # the class's overridables through its `PropertyPage` impl.
            out.append(f"        // CHAIN_MSG_MAP({e.args[0]})")
            out.append(
                "        if atl_rust::win::property_page_map(self as *mut Self, hwnd, msg, l, "
                "result) { return true; }"
            )
        elif e.macro == "CHAIN_MSG_MAP":
            base = short(e.args[0])
            out.append(f"        // CHAIN_MSG_MAP({e.args[0]})")
            path = h_path(h, cls, h.by_short[base]) if base in h.by_short else None
            if path is None:
                raise CppAppError(
                    f"{cls.qualified}: CHAIN_MSG_MAP({e.args[0]}) is not a module base"
                )
            out.append(
                "        if atl_rust::win::MessageMap::process_window_message("
                f"&mut self.{path.rstrip('.')}, "
                "hwnd, msg, w, l, result, map_id) { return true; }"
            )
        elif e.macro == "CHAIN_MSG_MAP_MEMBER":
            out.append(f"        // CHAIN_MSG_MAP_MEMBER({e.args[0]})")
            out.append(
                "        if atl_rust::win::MessageMap::process_window_message("
                f"&mut self.{e.args[0]}, "
                "hwnd, msg, w, l, result, map_id) { return true; }"
            )
        else:
            raise CppAppError(f"{cls.qualified}: message-map entry {e.macro} is not rendered yet")
    out.append("        let _ = (hwnd, map_id, &mut handled);")
    out.append("        false")
    return out


def _template_name(name: str) -> str:
    return name.split("<")[0].split("::")[-1].strip()


def _base_layout_order(cls: ClassModel, h: Hierarchy) -> list[int]:
    """The order base subobjects are LAID OUT in (their `__baseK` names keep declaration
    order). C++ puts the primary base -- the first one with a vptr -- at offset 0, ahead of
    non-polymorphic bases declared before it. That matters for ATL's COM roots: rapps'
    `CDownloadManager : CComCoClass<..>, CComObjectRootEx<..>, IUnknown` has its IUnknown
    vptr at offset 0, and a `CComPtr<CDownloadManager>` releases through offset 0. Only
    those roots are moved: an ATL window base, non-polymorphic in this model, is polymorphic
    in C++ and already first."""
    order = list(range(len(cls.bases)))
    roots = {i for i, b in enumerate(cls.bases) if _template_name(b.name) in _ATL_COM_ROOTS}
    if not roots:
        return order
    first = next(
        (
            i
            for i, b in enumerate(cls.bases)
            if i not in roots
            and (
                short(b.name) in h.com
                or (short(b.name) in h.by_short and h.by_short[short(b.name)].polymorphic)
            )
        ),
        None,
    )
    if first is None or any(i not in roots for i in range(first)):
        return order
    return [first, *(i for i in order if i != first)]


def _iface_path(h: Hierarchy, cls: ClassModel, iface: str) -> str | None:
    """The field path to `cls`'s subobject for COM interface `iface`, through module bases."""
    for i, b in enumerate(cls.bases):
        if short(b.name) == iface:
            return f"__base{i}"
    for i, b in h.module_bases(cls):
        sub = _iface_path(h, b, iface)
        if sub is not None:
            return f"__base{i}.{sub}"
    return None


def _render_com_object_glue(cls: ClassModel, h: Hierarchy, model: ModuleModel) -> str:
    """What `CComObject<T>` adds over a class with a COM map, given to the class itself:
    QueryInterface over the map, AddRef/Release on CComObjectRootEx's count (destroying at
    0 the way `CComObject`'s destructor does: FinalRelease, module Unlock, then the class's
    own destructor), and `CComObject<T>::CreateInstance` as atl_rust::com::ComObject."""
    n = cls.name
    root = next(
        (i for i, b in enumerate(cls.bases) if _template_name(b.name) == "CComObjectRootEx"), None
    )
    if root is None:
        raise CppAppError(f"{cls.qualified}: a COM map, but no CComObjectRootEx base of its own")
    unk = {slot.name: slot for slot in h.com.get("IUnknown", [])}
    if set(unk) < {"QueryInterface", "AddRef", "Release"}:
        raise CppAppError(f"{cls.qualified}: the island lays out no IUnknownVtbl")
    qi = unk["QueryInterface"]
    entries = []
    for e in cls.com_map or []:
        if e.macro == "COM_INTERFACE_ENTRY" and len(e.args) == 1:
            iid, via = f"IID_{e.args[0]}", e.args[0]
        elif e.macro == "COM_INTERFACE_ENTRY_IID" and len(e.args) == 2:
            iid, via = e.args[0], e.args[1]
        elif e.macro == "COM_INTERFACE_ENTRY2" and len(e.args) == 2:
            iid, via = f"IID_{e.args[0]}", e.args[1]
        else:
            raise CppAppError(f"{cls.qualified}: COM map entry {e.macro}({', '.join(e.args)})")
        path = _iface_path(h, cls, short(via))
        if path is None:
            raise CppAppError(f"{cls.qualified}: {e.macro} names {via}, which it does not derive")
        entries.append(
            f"atl_rust::com::ComMapEntry {{ iid: core::ptr::addr_of!({iid}) as *const c_void, "
            f"offset: core::mem::offset_of!({n}, {path}) }}"
        )
    ctor = next(
        (i for i, m in enumerate(m for m in cls.methods if m.kind == "ctor") if not m.params),
        None if any(m.kind == "ctor" for m in cls.methods) else 0,
    )
    if ctor is None:
        raise CppAppError(f"{cls.qualified}: CComObject<T> needs a default constructor")
    (a_riid, t_riid), (a_ppv, t_ppv) = qi.params
    fc = cls.method("FinalConstruct")
    fr = cls.method("FinalRelease")
    construct = f"(*p).{fc[0].rust_name}() as i32" if fc else "0"
    release = f"(*p).{fr[0].rust_name}(); " if fr else ""
    return "\n".join(
        [
            f"impl {n} {{",
            f"    /// `CComObject<{n}>::QueryInterface`: the COM map, first match.",
            f"    pub unsafe fn QueryInterface(&mut self, {a_riid}: {t_riid}, {a_ppv}: {t_ppv})"
            f" -> {qi.ret} {{",
            f"        let map: [atl_rust::com::ComMapEntry; {len(entries)}] = "
            f"[{', '.join(entries)}];",
            "        atl_rust::com::internal_query_interface(self as *mut Self as *mut c_void, "
            f"&map, {a_riid} as *const c_void, {a_ppv} as *mut *mut c_void) as {qi.ret}",
            "    }",
            f"    pub unsafe fn AddRef(&mut self) -> {unk['AddRef'].ret} {{",
            f"        atl_rust::com::internal_add_ref(&mut self.__base{root}) as "
            f"{unk['AddRef'].ret}",
            "    }",
            f"    pub unsafe fn Release(&mut self) -> {unk['Release'].ret} {{",
            f"        let n = atl_rust::com::internal_release(&mut self.__base{root});",
            f"        if n == 0 {{ {n}__com_destroy(self); }}",
            f"        n as {unk['Release'].ret}",
            "    }",
            "}",
            f"/// `delete` of a `CComObject<{n}>`: its destructor's FinalRelease and module "
            f"Unlock, then {n}'s own.",
            f"unsafe fn {n}__com_destroy(p: *mut {n}) {{ {release}"
            f"atl_rust::com::module_unlock(); {n}__delete(p); }}",
            f"impl atl_rust::com::ComObject for {n} {{",
            "    unsafe fn create_instance(pp: *mut *mut Self) -> i32 {",
            "        *pp = core::ptr::null_mut();",
            f"        let p = {n}__new_{ctor}();",
            "        if p.is_null() { return atl_rust::com::E_OUTOFMEMORY; }",
            "        atl_rust::com::module_lock();",
            f"        let hr: i32 = {construct};",
            f"        if hr != 0 {{ {n}__com_destroy(p); return hr; }}",
            "        *pp = p;",
            "        hr",
            "    }",
            "}",
        ]
    )


# rosdlgs.h `CPropertyPageImpl<T>` overridable -> (atl_rust::win::PropertyPage method, its
# parameters, the arguments passed on, the return cast or None for a `void`).
_PROPERTY_PAGE_HOOKS = {
    "OnPageCreate": ("on_page_create", "", "", "i32"),
    "OnPageAddRef": ("on_page_add_ref", "", "", None),
    "OnPageRelease": ("on_page_release", "", "", None),
    "OnSetActive": ("on_set_active", "", "", "i32"),
    "OnKillActive": ("on_kill_active", "", "", "i32"),
    "OnApply": ("on_apply", "", "", "i32"),
    "OnReset": ("on_reset", "", "", None),
    "OnHelp": ("on_help", "", "", None),
    "OnWizardBack": ("on_wizard_back", "", "", "i32"),
    "OnWizardNext": ("on_wizard_next", "", "", "i32"),
    "OnWizardFinish": ("on_wizard_finish", "", "", "i32"),
    "OnQueryCancel": ("on_query_cancel", "", "", "i32"),
    "OnGetObject": ("on_get_object", ", notify: atl_rust::win::LPARAM", "notify as _", "i32"),
    "OnTranslateAccelerator": (
        "on_translate_accelerator",
        ", msg: atl_rust::win::LPARAM",
        "msg as _",
        "i32",
    ),
    "OnQueryInitialFocus": (
        "on_query_initial_focus",
        ", focus: atl_rust::win::HWND",
        "focus as _",
        "atl_rust::win::HWND",
    ),
}


def _property_page_glue(cls: ClassModel, idd: str, base: str) -> list[str]:
    """`CPropertyPageImpl<T>`'s members, and the class's overrides of its overridables: the
    C++ base calls them through `static_cast<T*>(this)`, i.e. the derived class's own when it
    declares one -- exactly a trait default and its override."""
    lines = [
        "    /// rosdlgs.h `CPropertyPageImpl::Create`: this page's HPROPSHEETPAGE.",
        "    pub unsafe fn Create(&mut self) -> HPROPSHEETPAGE {",
        "        atl_rust::win::create_property_page(self as *mut Self, "
        f"{idd} as u16, core::ptr::null()) as HPROPSHEETPAGE",
        "    }",
        "    /// A page is not a modal dialog: the C++ `EndDialog` is `return FALSE`.",
        "    pub unsafe fn EndDialog(&mut self, code: INT_PTR) -> BOOL {",
        "        let _ = code; 0",
        "    }",
        "    pub unsafe fn SetModified(&mut self, changed: BOOL) {",
        f"        atl_rust::win::property_page_set_modified(self.{base}.m_hWnd as _, changed != 0)",
        "    }",
        "    pub unsafe fn SetWizardButtons(&mut self, flags: DWORD) {",
        "        // PSM_SETWIZBUTTONS",
        f"        PostMessageW(GetParent(self.{base}.m_hWnd), 0x0470, 0, flags as _);",
        "    }",
        "}",
        f"impl atl_rust::win::PropertyPage for {cls.name} {{",
    ]
    for name, (hook, params, args, ret) in _PROPERTY_PAGE_HOOKS.items():
        if not cls.method(name):
            continue
        sig = f"    unsafe fn {hook}(&mut self{params})"
        call = f"self.{_handler(cls, name, 1 if args else 0)}({args})"
        if ret is None:
            lines.append(f"{sig} {{ {call}; }}")
        else:
            lines.append(f"{sig} -> {ret} {{ {call} as {ret} }}")
    return lines


# An external ATL template base whose data atl_rust lays out.
_ATL_RUST_BASES = {
    "CAtlExeModuleT": "atl_rust::exe::CAtlExeModule",
    # atlcom.h's COM object roots: the reference count, and CComCoClass's empty base.
    "CComObjectRootEx": "atl_rust::com::CComObjectRootEx",
    "CComCoClass": "atl_rust::com::CComCoClass",
}
_ATL_COM_ROOTS = ("CComObjectRootEx", "CComCoClass")

_REPO = Path(__file__).resolve().parents[3]


def _exe_module_base(cls: ClassModel) -> int | None:
    """The index of a `CAtlExeModuleT<T>` base, if the class derives from one."""
    for i, b in enumerate(cls.bases):
        if _template_name(b.name) == "CAtlExeModuleT":
            return i
    return None


# atlbase.h `CAtlExeModuleT<T>` overridable -> (atl_rust::exe::ExeModule method, parameters,
# arguments, the call's rendering of its result). `{}` is the call.
_EXE_MODULE_HOOKS = {
    "ParseCommandLine": (
        "parse_command_line",
        "&mut self, cmd_line: *const u16, ret_code: *mut i32",
        "cmd_line as _, ret_code as _",
        "-> bool { ({}) as i32 != 0 }",
    ),
    "Run": ("run", "&mut self, show: i32", "show as _", "-> i32 { {} as i32 }"),
    "PreMessageLoop": (
        "pre_message_loop",
        "&mut self, show: i32",
        "show as _",
        "-> i32 { {} as i32 }",
    ),
    "RunMessageLoop": ("run_message_loop", "&mut self", "", "{ {}; }"),
    "PostMessageLoop": ("post_message_loop", "&mut self", "", "-> i32 { {} as i32 }"),
    "InitializeCom": ("initialize_com", "", "", "-> i32 where Self: Sized { {} as i32 }"),
    "UninitializeCom": ("uninitialize_com", "", "", "where Self: Sized { {}; }"),
}


def _render_exe_module_glue(cls: ClassModel, model: ModuleModel) -> str:
    """`CAtlExeModuleT<T>`: its `WinMain` and class-object members on the class, and the
    class's overrides as an `ExeModule` impl. atl_rust has no COM object map, so a module that
    declares one cannot take this glue: its class objects would silently never register."""
    files = {c.file for c in model.classes.values() if not c.external}
    for f in sorted(files):
        p = _REPO / f
        text = p.read_text("latin-1") if p.is_file() else ""
        if re.search(r"\b(OBJECT_ENTRY\w*|BEGIN_OBJECT_MAP)\b", text):
            raise CppAppError(
                f"{cls.qualified}: {f} declares a COM object map, which atl_rust::exe does not "
                "register; port the object map first"
            )
    n, base = cls.name, f"__base{_exe_module_base(cls)}"
    lines = [
        f"impl {n} {{",
        "    /// `CAtlExeModuleT::WinMain(nShowCmd)`.",
        "    pub unsafe fn WinMain(&mut self, show: c_int) -> c_int {",
        f"        if self.{base}.m_dwMainThreadID == 0 {{",
        "            // The C++ base constructor's GetCurrentThreadId(): a global module is built",
        "            // on the thread that then runs WinMain, so this is the same thread.",
        f"            self.{base} = atl_rust::exe::CAtlExeModule::new();",
        "        }",
        "        atl_rust::exe::win_main(self as *mut Self, show as i32) as c_int",
        "    }",
        "    pub unsafe fn RegisterClassObjects(&mut self, ctx: DWORD, flags: DWORD) -> HRESULT {",
        "        let _ = (ctx, flags); atl_rust::exe::register_class_objects() as HRESULT",
        "    }",
        "    pub unsafe fn RevokeClassObjects(&mut self) -> HRESULT {",
        "        atl_rust::exe::revoke_class_objects() as HRESULT",
        "    }",
        "}",
        f"impl atl_rust::exe::ExeModule for {n} {{",
    ]
    own = {m.rust_name for m in cls.methods if m.kind == "method"}
    lines[1:-2] = _drop_hidden_glue(lines[1:-2], own)
    for name, (hook, params, args, body) in _EXE_MODULE_HOOKS.items():
        ms = cls.method(name)
        if not ms:
            continue
        recv = "Self::" if ms[0].static else "self."
        call = f"{recv}{_handler(cls, name, args.count(' as _'))}({args})"
        lines.append(f"    unsafe fn {hook}({params}) {body.replace('{}', call)}")
    lines.append("}")
    return "\n".join(lines)


def _wnd_class_info(cls: ClassModel) -> list[str]:
    """DECLARE_WND_CLASS_EX(name, style, bkgnd) / DECLARE_WND_CLASS(name) as a static."""
    args = cls.wnd_class
    n = cls.name
    if args:
        name = args[0]
        style = args[1] if len(args) > 1 else "CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS"
        bkgnd = args[2] if len(args) > 2 else "COLOR_WINDOW"
    else:
        name, style, bkgnd = f'L"ATL:{n}"', "CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS", "COLOR_WINDOW"
    m = re.fullmatch(r'L"((?:[^"\\]|\\.)*)"', name.strip())
    if m is None:
        if name.strip() not in ("NULL", "0"):
            raise CppAppError(f"{cls.qualified}: window class name {name} is not a wide literal")
        # A NULL name: ATL generates "ATL:<address>"; a fixed per-class name serves the same
        # purpose (one registration per class) without the address.
        literal = f"ATL:{n}"
    else:
        literal = m.group(1)
    text = literal.encode().decode("unicode_escape") + "\0"
    units = ", ".join(str(ord(c)) for c in text)
    return [
        f"pub static {n}__CLASS_NAME: [u16; {len(text)}] = [{units}];",
        f"pub static {n}__WND_CLASS: atl_rust::win::WndClassInfo = atl_rust::win::WndClassInfo {{",
        f"    class_name: {n}__CLASS_NAME.as_ptr(),",
        f"    style: ({style}) as u32,",
        f"    background_color: ({bkgnd}) as i32,",
        "};",
    ]


def _nested_names(
    ty: str, cls: ClassModel, island_names: frozenset[str], own: frozenset[str] = frozenset()
) -> str:
    """A type named bare inside its class (`FN_Shutdown` in CImageDx::SHARED) is the
    island's `CImageDx_FN_Shutdown`: bindgen spells a nested name Outer_Inner. Innermost
    enclosing scope first, as C++ lookup goes; a name the island has bare is left alone.
    A class cpp_classes.rs renders itself (`own`) is never the island's: ToolsModel's
    `VirtualBrush m_fgBrush` became bindgen's method-less `ToolsModel_VirtualBrush`."""
    scopes = cls.qualified.split("::")
    prefixes = ["_".join(scopes[:k]) for k in range(len(scopes), 0, -1)]

    def fix(m: re.Match) -> str:
        w = m.group(0)
        if w in island_names or w in own:
            return w
        return next((f"{p}_{w}" for p in prefixes if f"{p}_{w}" in island_names), w)

    return re.sub(r"\b[A-Za-z_]\w*\b", fix, ty)


def _drop_hidden_glue(lines: list[str], own: set[str]) -> list[str]:
    """Remove each `pub unsafe fn NAME(` glue method (with its preceding /// lines) whose
    NAME the class declares itself."""
    out: list[str] = []
    skip = False
    for ln in lines:
        m = re.match(r"\s*pub unsafe fn (\w+)\(", ln)
        if m:
            skip = m.group(1) in own
            if skip:
                while out and out[-1].lstrip().startswith("///"):
                    out.pop()
        if not skip:
            out.append(ln)
        elif ln.rstrip() == "    }":
            skip = False
    return out


@dataclass
class AtlWindow:
    """Where a class's ATL window template is: which one, the field path to its CWindow
    subobject, and (CWindowImpl) its window-traits argument."""

    template: str
    path: str
    traits: str | None = None


_ATL_WINDOW_TEMPLATES = ("CWindowImpl", "CDialogImpl", "CPropertyPageImpl")


def _template_args(name: str) -> list[str]:
    if "<" not in name:
        return []
    return [a.strip() for a in _split(name[name.index("<") + 1 : name.rindex(">")]) if a.strip()]


def atl_window(cls: ClassModel, h: Hierarchy) -> AtlWindow | None:
    """The ATL window template `cls` IS (CRTP: `CWindowImpl<cls, ..>`), found among its bases
    or, through a template instance, deeper: rapps' `CAppInfoDisplay :
    CUiWindow<CWindowImpl<CAppInfoDisplay>>` is the window, at `__base0.__base2`, while the
    instance `CUiWindow_CWindowImpl_CAppInfoDisplay` is not one (its CRTP argument is the
    derived class, whose message map ATL calls)."""

    def to_cwindow(name: str, seen: frozenset[str] = frozenset()) -> str | None:
        if name == "CWindow":
            return ""
        c = h.by_short.get(name)
        if c is None or name in seen:
            return None
        for i, b in enumerate(c.bases):
            sub = to_cwindow(atl_base_alias(b.name), seen | {name})
            if sub is not None:
                return f"__base{i}" + (f".{sub}" if sub else "")
        return None

    def search(c: ClassModel, prefix: str) -> AtlWindow | None:
        for i, b in enumerate(c.bases):
            t = _template_name(b.name)
            if t not in _ATL_WINDOW_TEMPLATES:
                continue
            args = _template_args(b.name)
            if args and short(args[0]) != cls.name:
                continue
            sub = to_cwindow(atl_base_alias(b.name))
            if sub is None:
                raise CppAppError(f"{cls.qualified}: {b.name} reaches no CWindow")
            traits = args[2] if t == "CWindowImpl" and len(args) > 2 else None
            return AtlWindow(t, f"{prefix}__base{i}" + (f".{sub}" if sub else ""), traits)
        for i, b in h.module_bases(c):
            r = search(b, f"{prefix}__base{i}.")
            if r is not None:
                return r
        return None

    return search(cls, "")


# atlwin.h window traits -> (dwStyle, dwExStyle) that `T::GetWndStyle(0)` /
# `GetWndExStyle(0)` substitute for a zero style. CWindowImpl's default is CControlWinTraits.
_WIN_TRAITS = {
    "CControlWinTraits": ("WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS", "0"),
    "CFrameWinTraits": (
        "WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS",
        "WS_EX_APPWINDOW | WS_EX_WINDOWEDGE",
    ),
    "CMDIChildWinTraits": (
        "WS_OVERLAPPEDWINDOW | WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS",
        "WS_EX_MDICHILD",
    ),
}

_START_WINDOW_PROC = """/// ATL `CWindowImplBaseT::StartWindowProc`, which a module's own
/// `GetWndClassInfo()` names as its class's window procedure. ATL's is the thunk's
/// entry; atl_rust routes the object by window property instead and registers its own
/// procedure in its place
/// (`atl_rust::win::create_window_info`), so this one is never the class's procedure.
pub unsafe extern "system" fn StartWindowProc(
    hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM,
) -> LRESULT {
    DefWindowProcW(hwnd, msg, w, l)
}
"""


def _render_process_window_message(cls: ClassModel, h: Hierarchy) -> list[str]:
    """The `MessageMap` body: from BEGIN_MSG_MAP, or the class's hand-written
    `ProcessWindowMessage` (rapps' CUiSplitPanel, CAppInfoDisplay), which IS the map."""
    if cls.msg_map is not None:
        return _render_msg_map(cls, h)
    own = cls.method("ProcessWindowMessage")
    if not own:
        raise CppAppError(
            f"{cls.qualified}: an ATL window with neither BEGIN_MSG_MAP nor ProcessWindowMessage"
        )
    return [
        "        let mut r: LRESULT = 0;",
        f"        let handled = self.{own[0].rust_name}(hwnd as _, msg as _, w as _, l as _, "
        "&mut r, map_id as _) != 0;",
        "        *result = r as isize;",
        "        handled",
    ]


def _render_atl_glue(cls: ClassModel, h: Hierarchy, model: ModuleModel, win: AtlWindow) -> str:
    n, t = cls.name, win.template
    base = win.path
    lines = [
        f"impl core::ops::Deref for {n} {{ type Target = CWindow; "
        f"fn deref(&self) -> &CWindow {{ &self.{base} }} }}",
        f"impl core::ops::DerefMut for {n} {{ "
        f"fn deref_mut(&mut self) -> &mut CWindow {{ &mut self.{base} }} }}",
        f"impl atl_rust::win::MessageMap for {n} {{",
        "    unsafe fn process_window_message(&mut self, hwnd: atl_rust::win::HWND, msg: u32, "
        "w: usize, l: isize, result: &mut isize, map_id: u32) -> bool {",
        *_render_process_window_message(cls, h),
        "    }",
        "    unsafe fn hwnd_slot(&mut self) -> *mut atl_rust::win::HWND {",
        f"        core::ptr::addr_of_mut!(self.{base}.m_hWnd) as *mut atl_rust::win::HWND",
        "    }",
    ]
    if cls.method("OnFinalMessage"):
        lines.append(
            "    unsafe fn on_final_message(&mut self, hwnd: atl_rust::win::HWND) "
            f"{{ self.{_handler(cls, 'OnFinalMessage', 1)}(hwnd as _) }}"
        )
    lines.append("}")
    lines.append(f"impl {n} {{")
    glue_start = len(lines)
    lines.append(
        "    /// ATL `DefWindowProc(uMsg, wParam, lParam)`: the superclass procedure, else the "
        "default."
    )
    lines.append(
        "    pub unsafe fn DefWindowProc(&mut self, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {"
    )
    lines.append(
        f"        atl_rust::win::def_window_proc(self.{base}.m_hWnd as _, msg as u32, "
        "w as usize, l as isize) as LRESULT"
    )
    lines.append("    }")
    for k, v in cls.constants:
        lines.append(f"    pub const {k}: i32 = {v};")
    if t == "CWindowImpl":
        traits = _template_name(win.traits) if win.traits else "CControlWinTraits"
        if traits not in _WIN_TRAITS:
            raise CppAppError(f"{cls.qualified}: window traits {win.traits}")
        style, ex_style = _WIN_TRAITS[traits]
        params = (
            "parent: HWND, rect: *mut RECT, name: LPCWSTR, style: DWORD, ex_style: DWORD, "
            "menu_or_id: impl atl_rust::win::MenuOrId"
        )
        lines.append(
            "    /// ATL `CWindowImpl::Create(hWndParent, rect, szWindowName, dwStyle, dwExStyle, "
            f"MenuOrID)`; a zero style is {traits}'s (`T::GetWndStyle`). Named as the C++ "
            "qualifies it, since a class's own `Create` hides it."
        )
        lines.append(f"    pub unsafe fn CWindowImpl_Create(&mut self, {params}) -> HWND {{")
        lines.append(
            f"        let style = if style == 0 {{ ({style}) as DWORD }} else {{ style }};"
        )
        lines.append(
            f"        let ex_style = if ex_style == 0 {{ ({ex_style}) as DWORD }} "
            "else { ex_style };"
        )
        if cls.method("GetWndClassInfo"):
            # The class registers its own CWndClassInfo (icon, menu, cursor), as ATL's
            # CWindowImpl::Create does through T::GetWndClassInfo().Register().
            lines.append(
                "        atl_rust::win::create_window_info(self as *mut Self, "
                "atl_rust::win::class_info_ptr(Self::GetWndClassInfo() as *mut _), parent as _, "
                "rect as *const i32, name as *const u16, style as u32, ex_style as u32, "
                "menu_or_id.menu_or_id()) as HWND"
            )
        else:
            lines.append(
                "        atl_rust::win::create_window("
                f"self as *mut Self, &{n}__WND_CLASS, parent as _, "
                "rect as *const i32, name as *const u16, style as u32, ex_style as u32, "
                "menu_or_id.menu_or_id()) as HWND"
            )
        lines.append("    }")
        lines.append(f"    pub unsafe fn Create(&mut self, {params}) -> HWND {{")
        lines.append(
            "        self.CWindowImpl_Create(parent, rect, name, style, ex_style, menu_or_id)"
        )
        lines.append("    }")
        lines.append("    pub unsafe fn SubclassWindow(&mut self, hwnd: HWND) -> BOOL {")
        lines.append("        atl_rust::win::subclass_window(self as *mut Self, hwnd as _) as BOOL")
        lines.append("    }")
    elif t == "CPropertyPageImpl":
        idd = dict(cls.constants).get("IDD")
        if idd is None:
            raise CppAppError(
                f"{cls.qualified}: a CPropertyPageImpl without `enum {{ IDD = ... }}`"
            )
        lines.extend(_property_page_glue(cls, str(idd), base))
    else:
        idd = dict(cls.constants).get("IDD")
        if idd is None:
            raise CppAppError(f"{cls.qualified}: a CDialogImpl without `enum {{ IDD = ... }}`")
        lines.append("    /// ATL `CDialogImpl::DoModal(hWndParent, dwInitParam)`.")
        lines.append(
            "    pub unsafe fn DoModal(&mut self, parent: HWND, init: LPARAM) -> INT_PTR {"
        )
        lines.append(
            f"        atl_rust::win::do_modal(self as *mut Self, {idd}, parent as _, "
            "init as isize) as INT_PTR"
        )
        lines.append("    }")
        lines.append("    pub unsafe fn Create(&mut self, parent: HWND, init: LPARAM) -> HWND {")
        lines.append(
            "        let _ = init; atl_rust::win::create_dialog("
            f"self as *mut Self, {idd}, parent as _) as HWND"
        )
        lines.append("    }")
        lines.append("    pub unsafe fn EndDialog(&mut self, code: INT_PTR) -> BOOL {")
        lines.append(f"        EndDialog(self.{base}.m_hWnd, code as _)")
        lines.append("    }")
    # C++ name hiding: a class that declares its own `Create(HWND)` (CTextEditWindow) hides
    # every inherited Create, so the glue of that name is not rendered (E0592 otherwise).
    own = {m.rust_name for m in cls.methods if m.kind == "method"}
    own |= h.hand_methods.get(n, set())
    lines[glue_start:] = _drop_hidden_glue(lines[glue_start:], own)
    lines.append("}")
    if t == "CWindowImpl" and not cls.method("GetWndClassInfo"):
        lines.extend(_wnd_class_info(cls))
    return "\n".join(lines)


def _external_base(name: str, model: ModuleModel | None = None) -> str:
    """A base class the module does not define. ATL's window templates never get here (they
    alias to the module's CWindow); what does is a plain C struct from the SDK --
    `BITMAPINFOEX : BITMAPINFO` -- which is embedded by value as the island lays it out. A
    zero-size placeholder here would shift every field after it, silently."""
    if _template_name(name) in _ATL_RUST_BASES:
        return _ATL_RUST_BASES[_template_name(name)]
    if "<" in name:
        tb = cpp_templates.template_base(name, lambda a: rust_type(a, model) if model else a)
        if tb is not None:
            return tb.rust_type
        if _ATL_OWNING.match(name.strip()) and model is not None:
            # An atl_rust generic (`CLocalPtr<T> : CHeapPtr<T, CLocalAllocator>`): the same
            # layout a field of that type gets.
            return rust_type(name, model)
        raise CppAppError(f"external template base {name} has no Rust layout; port it first")
    return name.split("::")[-1].strip()


def _is_reference(c_type: str) -> bool:
    """`T &` / `const T &`, or the SDK's reference typedefs (REFGUID, REFIID, REFCLSID, ...)."""
    t = c_type.strip()
    return t.endswith("&") or re.fullmatch(r"(?:const\s+)?REF[A-Z]+", t) is not None


_ZERO_INIT = re.compile(
    r"(?:NULL|nullptr|FALSE|false|0+[uUlL]*|0[xX]0+[uUlL]*|\{\s*(?:NULL|nullptr|0)?\s*\})"
)


def _static_member_init(cls, f, ty: str, model: ModuleModel) -> str:
    """A static data member's value before any code runs: its definition's initializer
    (`DWORD TLS::s_dwTlsIndex = (DWORD)-1;`), rendered as a Rust constant. Zero-like and
    absent initializers are the zeroed storage C++ also starts from. Anything else this
    cannot render is refused: a static silently left at zero is a wrong value nobody sees."""
    v = (f.default_init or "").strip()
    if not v or _ZERO_INIT.fullmatch(v):
        return "unsafe { core::mem::zeroed() }"
    if v in ("TRUE", "true"):
        return "true" if ty == "bool" else "1"
    m = re.fullmatch(
        r"(?:\(\s*([A-Za-z_][\w ]*?)\s*\))?\s*\(?\s*(-?)\s*(0[xX][0-9A-Fa-f]+|\d+)[uUlL]*\s*\)?", v
    )
    if m is not None:
        cast, neg, num = m.groups()
        lit = f"({neg}{num}i64)"
        if cast is not None and " ".join(cast.split()) != " ".join(f.c_type.split()):
            lit = f"({lit} as {rust_type(cast, model)})"  # `(BYTE)-1` is 0xFF in any field
        return f"{lit} as {ty}"
    raise CppAppError(
        f"{cls.qualified}::{f.name} = {v}: a static member initializer this cannot render "
        "(cpp_app._static_member_init); extend it rather than leave the static zeroed"
    )


def aggregate_static(g, h: Hierarchy, model: ModuleModel) -> str | None:
    """A module struct (or array of them) with a brace initializer, as a Rust static:
    positional C values zipped with the struct's DECLARED field order. The model, given
    `{ID_FREESEL, TOOL_FREESEL}`, invented the field names (`nCommand`, `nTool` for `id`,
    `tool`). None when the initializer is not plain enough to zip here (strings, nesting
    deeper than one struct, a class with constructors or bases): the model gets it."""
    if not g.init or not g.init.lstrip().startswith("{"):
        return None
    t = " ".join(g.c_type.split())
    const = t.startswith("const ")
    t = t.removeprefix("const ").strip()
    arr = re.fullmatch(r"(\w+)\s*\[(\d*)\]", t)
    cls = h.by_short.get(arr.group(1) if arr else t)
    if cls is None or cls.bases or any(m.kind == "ctor" for m in cls.methods):
        return None
    fields = [f for f in cls.fields if not f.static]
    if '"' in g.init or "'" in g.init:
        return None

    def split(text: str) -> list[str]:  # top-level commas; braces nest too
        out, depth, cur = [], 0, []
        for ch in text:
            depth += (ch in "({[") - (ch in ")}]")
            if ch == "," and depth == 0:
                out.append("".join(cur))
                cur = []
            else:
                cur.append(ch)
        return [*out, "".join(cur)]

    def one(text: str) -> str | None:
        text = text.strip()
        if not (text.startswith("{") and text.endswith("}")):
            return None
        vals = [v.strip() for v in split(text[1:-1]) if v.strip()]
        if len(vals) > len(fields) or any("{" in v for v in vals):
            return None
        parts = [
            # A REFERENCE member (`REFGUID m_guid;`, msctfime's MODEBIAS) is laid out as a
            # pointer and binds to its initializer's object: the address, never a cast of
            # the value (`GUID as *const GUID` does not even compile).
            f"{_ident(f.name)}: core::ptr::addr_of!({_strip_ns(v, model)}) as _"
            if _is_reference(f.c_type)
            else f"{_ident(f.name)}: ({_strip_ns(v, model)}) as _"
            for f, v in zip(fields, vals, strict=False)
        ]
        if len(vals) < len(fields):
            parts.append("..unsafe { core::mem::zeroed() }")
        return f"{cls.name} {{ {', '.join(parts)} }}"

    body = g.init.strip()
    mut = "" if const else "mut "
    if arr:
        items = [v for v in split(body[1:-1]) if v.strip()]
        rendered = [one(v) for v in items]
        if any(r is None for r in rendered):
            return None
        n = arr.group(2) or str(len(rendered))
        lines = ",\n    ".join(r for r in rendered if r)
        return f"pub static {mut}{g.name}: [{cls.name}; {n}] = [\n    {lines},\n];"
    single = one(body)
    return None if single is None else f"pub static {mut}{g.name}: {cls.name} = {single};"


_C_ESCAPES = {"n": 10, "t": 9, "r": 13, "0": 0, "a": 7, "b": 8, "f": 12, "v": 11}


def _wide_units(body: str) -> list[int] | None:
    """A wide literal's body as UTF-16 code units, C escapes decoded; None for an escape
    this does not know (the model gets the global instead)."""
    out: list[int] = []
    i = 0
    while i < len(body):
        c = body[i]
        if c != "\\":
            out.extend(
                int.from_bytes(c.encode("utf-16-le")[k : k + 2], "little")
                for k in range(0, len(c.encode("utf-16-le")), 2)
            )
            i += 1
            continue
        nxt = body[i + 1 : i + 2]
        if nxt in ("\\", '"', "'", "?"):
            out.append(ord(nxt))
            i += 2
        elif nxt == "x":
            m = re.match(r"[0-9A-Fa-f]+", body[i + 2 :])
            if m is None:
                return None
            out.append(int(m.group(0), 16) & 0xFFFF)
            i += 2 + m.end()
        elif nxt in "1234567":
            m = re.match(r"[0-7]{1,3}", body[i + 1 :])
            out.append(int(m.group(0), 8))  # type: ignore[union-attr]
            i += 1 + m.end()  # type: ignore[union-attr]
        elif nxt in _C_ESCAPES:
            out.append(_C_ESCAPES[nxt])
            i += 2
        else:
            return None
    return out


def wide_string_static(g) -> str | None:
    """`const WCHAR UsageString[] = L"..";` as a static `[u16; N]`, units and NUL padding
    exactly as the C array holds them. A wide literal needs no model, and a header-only
    unit holding nothing else is one translate refuses."""
    t = " ".join(g.c_type.split())
    m = re.fullmatch(r"(const )?(?:WCHAR|wchar_t)\s*\[(\d*)\]", t)
    if m is None or not g.init:
        return None
    lit = re.fullmatch(r'L"((?:[^"\\]|\\.)*)"', g.init.strip(), re.S)
    units = _wide_units(lit.group(1)) if lit else None
    if units is None:
        return None
    n = int(m.group(2)) if m.group(2) else len(units) + 1
    if len(units) > n:
        return None
    vals = ", ".join(str(u) for u in [*units, *([0] * (n - len(units)))])
    return f"pub static {'' if m.group(1) else 'mut '}{g.name}: [u16; {n}] = [{vals}];"


def _render_globals(model: ModuleModel, h: Hierarchy) -> str:
    """Module globals of CLASS type, zeroed until `__cxx_global_init` constructs them."""
    out = []
    for g in model.globals:
        wide = wide_string_static(g)
        if wide is not None:
            out.append(wide)
            continue
        agg = aggregate_static(g, h, model)
        if agg is not None:
            out.append(agg)
            continue
        cls = h.by_short.get(short(g.c_type.replace("*", "").strip()))
        if cls is None or "*" in g.c_type:
            continue
        out.append(f"pub static mut {g.name}: {cls.name} = unsafe {{ core::mem::zeroed() }};")
    return "\n".join(out)


_HELPERS = """// ---- C++ runtime pieces the lowered bodies call (cpp_lower) ----
extern "C" {
    fn malloc(n: usize) -> *mut core::ffi::c_void;
    fn calloc(n: usize, size: usize) -> *mut core::ffi::c_void;
    fn free(p: *mut core::ffi::c_void);
}
/// `new T(v)` for a non-class T, into storage from an allocator (NULL stays NULL).
pub unsafe fn cpp_new_in<T>(p: *mut T, v: T) -> *mut T {
    if p.is_null() { return p; } // a nothrow new's NULL; never construct into it
    core::ptr::write(p, v);
    p
}
pub unsafe fn cpp_new<T>(v: T) -> *mut T { cpp_new_in(cpp_alloc::<T>(), v) }
pub unsafe fn cpp_delete<T>(p: *mut T) {
    if !p.is_null() { core::ptr::drop_in_place(p); cpp_free(p as *mut _); }
}
/// atlbase.h's ATLASSERT: a DEBUG assertion, so a no-op MACRO (never an out-of-line no-op fn,
/// which is an optimization barrier that has uncloaked ABI bugs before).
macro_rules! ATLASSERT { ($($t:tt)*) => {}; }
pub(crate) use ATLASSERT;
/// atltrace.h's ATLTRACE: debug output, compiled out in a release ReactOS build. No-op macro.
macro_rules! ATLTRACE { ($($t:tt)*) => {}; }
pub(crate) use ATLTRACE;
/// A pure virtual reached at runtime: the C++ runtime's `_purecall` terminates the process.
pub fn cpp_pure_virtual_call(what: &str) -> ! { panic!("pure virtual call: {}", what) }
"""


# ---------------------------------------------------------------------------------------
# synthetic TUs
# ---------------------------------------------------------------------------------------


def _link_name(i386: str, alias: str, sig: str) -> str:
    """An extern for an Itanium-mangled operator: `size_t` is `j` on i386, `y` on 64-bit."""
    wide = re.sub(r"^(_Z(?:nw|na))j", r"\1y", i386)
    wide = re.sub(r"^(_Zd[la]Pv)j$", r"\1y", wide)
    return (
        f'    #[cfg_attr(target_pointer_width = "32", link_name = "{i386}")]\n'
        f'    #[cfg_attr(target_pointer_width = "64", link_name = "{wide}")]\n'
        f"    fn {alias}{sig};"
    )


def _allocator_helpers(model: ModuleModel) -> str:
    """cpp_alloc/cpp_free and friends. Where the module (or a library it links, cicero's
    cicbase.h) REPLACES a global allocation operator, they call that operator by its mangled
    name -- what the C++ calls -- so an object a C++ half allocates and a Rust half frees
    (or the reverse) stays on one heap while a static lib's users flip one at a time. The
    defaults are msvcrt's calloc/free, zeroed as before."""
    a = model.allocators
    ext: list[str] = []
    out: list[str] = []
    void = "*mut core::ffi::c_void"
    if "new" in a:
        ext.append(_link_name(a["new"], "__cxx_op_new", f"(n: usize) -> {void}"))
        alloc = "__cxx_op_new(core::mem::size_of::<T>())"
    else:
        alloc = "calloc(1, core::mem::size_of::<T>())"
    out.append("/// `operator new` for a class: storage, constructed by the caller.")
    out.append(f"pub unsafe fn cpp_alloc<T>() -> *mut T {{ {alloc} as *mut T }}")
    if "delete" in a:
        ext.append(_link_name(a["delete"], "__cxx_op_delete", f"(p: {void})"))
        out.append(f"pub unsafe fn cpp_free(p: {void}) {{ __cxx_op_delete(p) }}")
    else:
        out.append(f"pub unsafe fn cpp_free(p: {void}) {{ free(p) }}")
    if "new[]" in a:
        ext.append(_link_name(a["new[]"], "__cxx_op_new_array", f"(n: usize) -> {void}"))
        arr = "__cxx_op_new_array(n.max(1) * core::mem::size_of::<T>())"
    else:
        arr = "calloc(n.max(1), core::mem::size_of::<T>())"
    out.append("/// `new T[n]` (trivial T: no cookie, no per-element construction).")
    out.append(
        "pub unsafe fn cpp_new_array<T>(n: impl TryInto<usize>) -> *mut T { "
        f"let n: usize = n.try_into().unwrap_or(0); {arr} as *mut T }}"
    )
    free_arr = "__cxx_op_delete_array(p as *mut _)" if "delete[]" in a else "free(p as *mut _)"
    if "delete[]" in a:
        ext.append(_link_name(a["delete[]"], "__cxx_op_delete_array", f"(p: {void})"))
    out.append(
        f"pub unsafe fn cpp_delete_array<T>(p: *mut T) {{ if !p.is_null() {{ {free_arr}; }} }}"
    )
    tag_ptr = "core::ptr::NonNull::<u8>::dangling().as_ptr() as *const core::ffi::c_void"
    for role, mangled in sorted(a.items()):
        op, _, tag = role.partition(":")
        if not tag:
            continue
        alias = f"__cxx_op_{'new_array' if op == 'new[]' else 'new'}_{tag}"
        ext.append(
            _link_name(mangled, alias, f"(n: usize, tag: *const core::ffi::c_void) -> {void}")
        )
        if op == "new":
            out.append(f"/// `new({tag}{{}}) T`: {tag}'s allocator, NULL on failure.")
            out.append(
                f"pub unsafe fn cpp_alloc_{tag}<T>() -> *mut T {{ "
                f"{alias}(core::mem::size_of::<T>(), {tag_ptr}) as *mut T }}"
            )
        else:
            out.append(
                f"pub unsafe fn cpp_new_array_{tag}<T>(n: impl TryInto<usize>) -> *mut T {{ "
                "let n: usize = n.try_into().unwrap_or(0); "
                f"{alias}(n.max(1) * core::mem::size_of::<T>(), {tag_ptr}) as *mut T }}"
            )
    head = 'extern "C" {\n' + "\n".join(ext) + "\n}\n" if ext else ""
    return head + "\n".join(out) + "\n" + _allocator_exports(model)


def _allocator_exports(model: ModuleModel) -> str:
    """The module's own operators, translated as `cxx_operator_*`, exported under the
    Itanium symbol every caller links against (the externs above, and any C++ left). The
    tag argument is an empty object nobody reads; transmute passes it whichever way the
    translation spelled the reference (`&CicNoThrow` or `*const CicNoThrow`)."""
    out = []
    void = "*mut core::ffi::c_void"
    for role, fn in sorted(model.allocator_fns.items()):
        i386 = model.allocators.get(role)
        if not i386:
            continue
        wide = re.sub(r"^(_Z(?:nw|na))j", r"\1y", i386)
        op, _, tag = role.partition(":")
        slug = re.sub(r"\W", "_", role)
        if op.startswith("new"):
            sig = (
                f"n: usize, tag: *const core::ffi::c_void) -> {void}"
                if tag
                else f"n: usize) -> {void}"
            )
            call = (
                f"{fn}(n as _, core::mem::transmute(tag)) as {void}"
                if tag
                else f"{fn}(n as _) as {void}"
            )
        else:
            sig, call = f"p: {void})", f"{fn}(p as _)"
        out.append(
            f'#[cfg_attr(target_pointer_width = "32", export_name = "{i386}")]\n'
            f'#[cfg_attr(target_pointer_width = "64", export_name = "{wide}")]\n'
            f'pub unsafe extern "C" fn __cxx_export_{slug}({sig} {{ {call} }}'
        )
    return "\n".join(out) + "\n" if out else ""


def _strip_ns(text: str, model: ModuleModel) -> str:
    """Drop namespace qualifiers (`snd::audio_membuffer`): Rust names a class by its short
    name. A CLASS that merely encloses another (`CImageDx::SHARED`) loses its qualifier only
    before a nested type -- `CImageDx::_shared()` is a static call and keeps it, or it reads
    as a free function that does not exist (33 mspaint errors)."""
    for ns in _namespaces(model):
        if ns in model.classes:
            nested = sorted(
                (q[len(ns) + 2 :] for q in model.classes if q.startswith(ns + "::")),
                key=len,
                reverse=True,
            )
            if nested:
                alt = "|".join(re.escape(n) for n in nested)
                text = re.sub(rf"\b{re.escape(ns)}::(?=(?:{alt})\b)", "", text)
            continue
        text = re.sub(rf"\b{re.escape(ns)}::", "", text)
    return text


#: atl_rust's own types, as the generated prelude re-exports them (profile_derive's ATL block).
ATL_RUST_TYPES = frozenset(
    {
        "CStringW",
        "CStringA",
        "CSimpleString",
        "CAtlStringW",
        "CString",
        "CAtlString",
        "CComPtr",
        "CHeapPtr",
        "CComHeapPtr",
        "CSimpleArray",
        "CSimpleMap",
        "CAtlList",
        "CPath",
        "POSITION",
    }
)


def island_type_names(island: str) -> frozenset[str]:
    """The type names a bindgen island defines (nested ones spelled Outer_Inner)."""
    return frozenset(re.findall(r"^\s*pub (?:type|struct|union|enum) (\w+)", island, re.M))


def synthetic_tus(model: ModuleModel, island: str = "") -> dict[str, str]:
    """{body file (repo-relative): C-like text}: every lowered body, grouped by the file
    its text lives in, as free functions a worker can translate one at a time. `island` as
    for render_classes: a COM interface method is a plain body, not a `__body` behind a
    dispatcher -- the two must agree or assemble's names miss the vtable's."""
    h = Hierarchy(model, parse_com_vtbls(island))
    # The signature/body renames (_synthetic_method) need the island's type names.
    h.island_names = island_type_names(island)
    reached = external_reached(model)
    by_file: dict[str, list[str]] = {}
    for cls in model.classes.values():
        for m in cls.methods:
            if m.body is None or m.lowered is None:
                continue
            if m.kind == "dtor" and _empty_body(m):
                continue  # no Drop is rendered for it (has_drop)
            if (
                cls.external
                and m.kind not in ("ctor", "dtor")  # Drop calls the dtor: always reached
                and (cls.name, m.rust_name) not in reached
            ):
                continue  # an ATL method the module never calls: never translated
            by_file.setdefault(m.body[0], []).append(_synthetic_method(cls, m, h, model))
        if not any(x.kind == "ctor" for x in cls.methods):
            by_file.setdefault(cls.file, []).append(_implicit_ctor(cls, h, model))
    # File-scope globals that are not class OBJECTS (those live in cpp_classes.rs): `HWND
    # main_win;`, `snd::audio_membuffer *AUD_BUF;`. They head their file's unit, where the
    # translator's file-scope preamble pass turns them into statics.
    heads: dict[str, list[str]] = {}
    for file, text in model.typedefs:  # before the globals that use them (s_pHtmlHelpW)
        heads.setdefault(file, []).append(text)
    for g in model.globals:
        cls = h.by_short.get(short(g.c_type))
        if cls is not None and "*" not in g.c_type and "[" not in g.c_type:
            continue
        if aggregate_static(g, h, model) is not None or wide_string_static(g) is not None:
            continue  # rendered in cpp_classes.rs, field by field / unit by unit
        decl = _c_declarator(g.c_type, g.name)
        init = f" = {g.init}" if g.init and not g.init.startswith("(") else ""
        heads.setdefault(g.file, []).append(
            _strip_ns(f"{'static ' if g.static else ''}{decl}{init};", model)
        )
    for file, decls in heads.items():
        by_file[file] = ["\n".join(decls) + "\n", *by_file.get(file, [])]
    init = _global_init(model, h)
    constructs = bool(_global_init_calls(model, h))
    for f in model.functions:
        body = f.lowered
        if constructs and f.name in _ENTRY_POINTS:
            # C++ runs static constructors before the entry point; the Rust entry runs them
            # first thing instead (there is no CRT initializer table for a Rust staticlib).
            brace = body.index("{")
            body = body[: brace + 1] + "\n    __cxx_global_init();" + body[brace + 1 :]
        by_file.setdefault(f.file, []).append(_strip_ns(f"{f.signature}\n{body}\n", model))
    if constructs:
        entry = next((f.file for f in model.functions if f.name in _ENTRY_POINTS), None)
        if entry is None:
            raise CppAppError("module globals need construction but no entry point was found")
        if init:
            by_file.setdefault(entry, []).append(init)
    names = frozenset(
        re.findall(r"^\s*pub (?:type|struct|union|enum|const|static(?: mut)?) (\w+)", island, re.M)
    )
    return {
        k: "\n".join(_flatten_island_ns(_apply_using(c, names), names) for c in v)
        for k, v in by_file.items()
    }


def _apply_using(chunk: str, island_names: frozenset[str]) -> str:
    """A function-scope `using namespace Gdiplus;` (atlimagedx.h's LoadDx): its bare names
    (`GpBitmap`, `Status`) are the island's `Gdiplus_GpBitmap`, `Gdiplus_Status`. One chunk
    is one function, which is exactly the directive's scope; the directive is dropped."""
    spaces = re.findall(r"\busing\s+namespace\s+([\w:]+)\s*;", chunk)
    if not spaces:
        return chunk
    chunk = re.sub(r"[ \t]*\busing\s+namespace\s+[\w:]+\s*;[ \t]*\n?", "", chunk)
    prefixes = [n.replace("::", "_") for n in spaces]

    def fix(m: re.Match) -> str:
        w = m.group(0)
        if w in island_names:
            return w
        return next((f"{p}_{w}" for p in prefixes if f"{p}_{w}" in island_names), w)

    return re.sub(r"(?<![\w:.>])[A-Za-z_]\w*\b", fix, chunk)


def _flatten_island_ns(text: str, island_names: frozenset[str]) -> str:
    """`Gdiplus::GpImage` -> the island's `Gdiplus_GpImage` (bindgen flattens a namespace
    path with `_`). Only where the island has that exact name; the rest is untouched."""

    def fix(m: re.Match) -> str:
        flat = m.group(0).replace("::", "_")
        return flat if flat in island_names else m.group(0)

    return re.sub(r"\b[A-Za-z_]\w*(?:::[A-Za-z_]\w*)+\b", fix, text)


_QUALIFIED_CALL_RE = re.compile(r"\b\w+::\s*([A-Za-z_]\w*)\s*\(")
_CALL_RE = re.compile(r"(?:\.|->)\s*([A-Za-z_]\w*)\s*\(")


def external_reached(model: ModuleModel) -> set[tuple[str, str]]:
    """(class, method) pairs of EXTERNAL classes (ATL's CWindow: 134 inline methods) that the
    module's code calls, directly or through another reached method. Only these are
    translated -- mspaint reaches about 45 -- and a name shared with a module method is
    included rather than missed."""
    ext = [c for c in model.classes.values() if c.external]
    if not ext:
        return set()
    if any(m.calls for c in model.classes.values() for m in c.methods) or any(
        f.calls for f in model.functions
    ):
        return _reached_by_calls(model, ext)
    texts = [m.lowered or "" for c in model.classes.values() if not c.external for m in c.methods]
    texts += [t for c in model.classes.values() for m in c.methods for _, t in m.lowered_inits]
    texts += [f.lowered for f in model.functions]
    # CWindow's own Create/DefWindowProc/... (below): every module call of those names is a
    # CWindowImpl/CDialogImpl subclass's, answered by its generated glue. Counting them
    # translated ATL's raw CWindow::Create (_U_RECT, _AtlBaseModule) for nothing. A call on a
    # PLAIN CWindow would then be missing, which fails to compile -- loud, never silent.
    joined = "\n".join(texts)
    called = set(_CALL_RE.findall(joined))
    # A static call is `Class::name(` (CWaitCursor::IsWaiting()), not `.name(`.
    called |= set(_QUALIFIED_CALL_RE.findall(joined))
    # A module class on PLAIN CWindow (CPaintToolBar : CWindow) calls ATL's own Create; only
    # CWindowImpl/CDialogImpl subclasses have glue answering those names.
    plain_cwindow = any(
        not c.external and not atl_template(c) and any(short(b.name) == "CWindow" for b in c.bases)
        for c in model.classes.values()
    )
    glue_only = frozenset() if plain_cwindow else ATL_IMPL_METHODS
    reached: set[tuple[str, str]] = set()
    while True:
        new = {
            (c.name, m.rust_name)
            for c in ext
            for m in c.methods
            if m.rust_name in called
            # Only CWindow: CRegKey::Create is a real call (registry.cpp).
            and not (c.name == "CWindow" and m.rust_name in glue_only)
            and (c.name, m.rust_name) not in reached
        }
        if not new:
            return reached
        reached |= new
        for c in ext:
            for m in c.methods:
                if (c.name, m.rust_name) in new:
                    called |= set(_CALL_RE.findall(m.lowered or ""))
                    # and its static calls, lowered `CImage::InitGDIPlus()`
                    called |= set(_QUALIFIED_CALL_RE.findall(m.lowered or ""))


def _c_declarator(c_type: str, name: str) -> str:
    """`TCHAR[100]` + `szTitle` -> `TCHAR szTitle[100]`; `void (*)(int)` -> `void (*name)(int)`."""
    arr = re.match(r"^(.*?)\s*((?:\[[^\]]*\])+)$", c_type)
    if arr:
        return f"{arr.group(1)} {name}{arr.group(2)}"
    if "(*)" in c_type:
        return c_type.replace("(*)", f"(*{name})", 1)
    return f"{c_type} {name}"


_ENTRY_POINTS = ("wWinMain", "WinMain", "_tWinMain", "wmain", "main", "_tmain", "DllMain")


def _global_init_calls(model: ModuleModel, h: Hierarchy) -> list[tuple[str, int, str]]:
    """`audio_format UNKNOWN_FORMAT(0, 0, 0);` -> (global, constructor index, argument text),
    in definition order (the order C++ guarantees within one TU)."""
    calls = []
    for g in model.globals:
        cls = h.by_short.get(short(g.c_type))
        if cls is None or "*" in g.c_type or "[" in g.c_type:
            continue
        ctors = [m for m in cls.methods if m.kind == "ctor"]
        if (
            g.init is None
            and not ctors
            and not cls.polymorphic
            and not any(f.default_init for f in cls.fields if not f.static)
        ):
            continue  # zeroed storage IS its value-initialized state
        arg = g.init if g.init and g.init.startswith("(") else "()"
        idx = _match_ctor(cls, arg) if ctors else 0
        calls.append((g.name, idx, arg))
    return calls


def _global_init_is_rendered(calls: list[tuple[str, int, str]]) -> bool:
    """No constructor ARGUMENTS anywhere (`CCertMgrModule _AtlModule;`): cpp_classes.rs renders
    the whole __cxx_global_init, so nothing is bought. Twice (cleanmgr, certmgr) the model
    turned the C stub's `_AtlModule.__ctor_0();` into an invented `_AtlModule___ctor_0`
    extern that failed the link. With arguments, those are C expressions to translate."""
    return bool(calls) and all(arg == "()" for _, _, arg in calls)


def _global_init(model: ModuleModel, h: Hierarchy) -> str:
    """The synthetic C `__cxx_global_init` for the model -- or nothing, when cpp_classes.rs
    renders it (the entry point still calls it: see `_global_init_calls` at the call site)."""
    calls = _global_init_calls(model, h)
    if not calls:
        return ""
    if _global_init_is_rendered(calls):
        # Nothing at all, not even a prototype: a new preamble line would be a unit the
        # response cache never bought (certmgr's replay refused on exactly that).
        return ""
    return _strip_ns(
        "// C++ static initialization of the module's global objects\n"
        "void __cxx_global_init(void)\n{\n"
        + "\n".join(f"    {n}.__ctor_{i}{a};" for n, i, a in calls)
        + "\n}\n",
        model,
    )


def _render_global_init(model: ModuleModel, h: Hierarchy) -> str:
    calls = _global_init_calls(model, h)
    if not _global_init_is_rendered(calls):
        return ""
    body = "\n".join(f"    (*core::ptr::addr_of_mut!({n})).__ctor_{i}();" for n, i, _ in calls)
    return (
        "/// C++ static initialization of the module's global objects, in definition order. The\n"
        "/// entry point calls it first thing (no CRT initializer table runs for a staticlib).\n"
        "#[no_mangle]\n"
        f'pub unsafe extern "C" fn __cxx_global_init() {{\n{body}\n}}'
    )


def _ctor_prologue(
    cls: ClassModel, m: Method | None, h: Hierarchy, model: ModuleModel
) -> list[str]:
    """What a C++ constructor does before its body, as explicit statements: bases in
    declaration order, then the vtable pointers, then every member in DECLARATION order
    (not init-list order) from its init-list entry, else its in-class initializer."""
    inits = dict(m.lowered_inits) if m else {}
    lines = []
    for i, b in enumerate(cls.bases):
        bn = short(b.name)
        base = h.by_short.get(bn)
        if base is None:
            continue
        arg = inits.get(b.name) or inits.get(bn)
        idx, call = _ctor_call(base, arg or "()")
        lines.append(f"    this_->__base{i}.__ctor_{idx}{call};")
    if cls.polymorphic:
        lines.append("    this_->__init_vtables();")
    for f in cls.fields:
        if f.static:
            continue
        val = inits.get(f.name)
        if val is not None:
            v = val[1:-1] if val.startswith("(") and val.endswith(")") else val
            lines.append(f"    this_->{f.name} = {v};")
        elif f.default_init is not None:
            v = f.default_init.lstrip("=").strip()
            lines.append(f"    this_->{f.name} = {v};")
        else:
            # A class-typed member with no initializer is DEFAULT-CONSTRUCTED. rapps'
            # CUiContainer (no constructor of its own) left `m_Children` zeroed, so its
            # CPointerArray never ran DPA_Create, and the first GetCount read address 0 on
            # all three arches.
            member = None if re.search(r"[*&\[]", f.c_type) else h.by_short.get(short(f.c_type))
            if member is not None and _needs_construction(member, h):
                idx, call = _ctor_call(member, "()")
                lines.append(f"    this_->{f.name}.__ctor_{idx}{call};")
    return lines


def _needs_construction(cls: ClassModel, h: Hierarchy, seen: frozenset = frozenset()) -> bool:
    """Whether default-constructing `cls` does anything zeroed storage does not already
    hold: a constructor, a vtable, an in-class initializer, or a base or by-value member
    that needs one."""
    if cls.name in seen:
        return False
    seen = seen | {cls.name}
    if cls.polymorphic or any(m.kind == "ctor" for m in cls.methods):
        return True
    if any(f.default_init is not None for f in cls.fields if not f.static):
        return True
    parts = [h.by_short.get(short(b.name)) for b in cls.bases] + [
        h.by_short.get(short(f.c_type))
        for f in cls.fields
        if not f.static and not re.search(r"[*&\[]", f.c_type)
    ]
    return any(p is not None and _needs_construction(p, h, seen) for p in parts)


def _ctor_call(base: ClassModel, arg: str) -> tuple[int, str]:
    """(constructor index, argument list) for constructing `base` with `arg` as written,
    trailing DEFAULT arguments filled in: a base left to default-construct through
    `CWindow(HWND hWnd = NULL)` is `__ctor_0(NULL)`, never `__ctor_0()`."""
    written = [] if arg.strip() in ("()", "") else _split(arg.strip()[1:-1])
    ctors = [m for m in base.methods if m.kind == "ctor"]
    for i, m in enumerate(ctors):
        n = len(written)
        if len(m.params) >= n and all(p.default is not None for p in m.params[n:]):
            full = [w.strip() for w in written] + [p.default or "" for p in m.params[n:]]
            return i, "(" + ", ".join(full) + ")"
    return _match_ctor(base, arg), arg


def _match_ctor(base: ClassModel, arg: str) -> int:
    n = 0 if arg.strip() in ("()", "") else len(_split(arg.strip()[1:-1]))
    ctors = [m for m in base.methods if m.kind == "ctor"]
    for i, m in enumerate(ctors):
        if len(m.params) == n:
            return i
    return 0


_QUALIFIED_BASE_CALL = re.compile(r"\b(\w+)::this_->((?:__base\d+\.)+)(\w+)(\s*\()")


def _qualified_base_calls(body: str, h: Hierarchy) -> str:
    """`Base::M(..)` from a derived class -- the lowering leaves `Base::this_->__baseK.M(` --
    is a NON-virtual call to Base's own M. Spelled `this_->__baseK.M(` it reaches the virtual
    dispatcher, i.e. the derived override that is making the call: rapps' CUiWindow<CToolbar>
    ::ComputeMinimalSize (`return CUiBox::ComputeMinimalSize(size);`) spun forever, and
    cicero's CUIFWindow::Initialize shipped the same recursion. A virtual M is called as its
    body, `M__body`; a non-virtual one just loses the qualifier. Only when the lowering gave
    a base path: without one, dropping the qualifier would name the derived class's own M."""

    def fix(mt: re.Match) -> str:
        qual, path, name, paren = mt.group(1), mt.group(2), mt.group(3), mt.group(4)
        start = h.by_short.get(qual)
        if start is None:
            return mt.group(0)
        queue, seen = [start], set()
        while queue:
            c = queue.pop(0)
            if c.name in seen:
                continue
            seen.add(c.name)
            decl = next((x for x in c.methods if x.rust_name == name), None)
            if decl is not None:
                virtual = h.is_virtual(c, decl) and decl.body is not None and not decl.pure
                return f"this_->{path}{name}{'__body' if virtual else ''}{paren}"
            queue += [b for _, b in h.module_bases(c)]
        return mt.group(0)

    return _QUALIFIED_BASE_CALL.sub(fix, body)


def _synthetic_method(cls: ClassModel, m: Method, h: Hierarchy, model: ModuleModel) -> str:
    n = cls.name
    # atl_rust's types are named bare on purpose (the prelude maps CSimpleString to CStringW);
    # ATL::CImage's `CSimpleString&` must not become bindgen's opaque ATL_CSimpleString.
    own = frozenset(h.by_short) | ATL_RUST_TYPES

    def ty(c_type: str) -> str:
        # A nested type named bare in the declaration (`WallpaperStyle style` inside
        # RegistrySettings) is the island's RegistrySettings_WallpaperStyle, as for fields.
        return _nested_names(c_type, cls, h.island_names, own)

    params = ", ".join([f"{n} *this_", *(f"{ty(p.c_type)} {p.name}" for p in m.params)])
    if m.static:
        params = ", ".join(f"{ty(p.c_type)} {p.name}" for p in m.params) or "void"
    suffix = (
        "__body" if (h.is_virtual(cls, m) and m.kind not in ("ctor",)) or m.kind == "dtor" else ""
    )
    ret = ty(m.return_type) if m.kind == "method" or m.kind == "operator" else "void"
    # The body too: atlimagedx.h's `FN_Startup Startup = ..` inside CImageDx is bindgen's
    # CImageDx_FN_Startup. Only names whose Outer_name the island HAS are rewritten.
    body = _qualified_base_calls(ty(m.lowered or "{}"), h)
    if m.kind == "ctor":
        pro = _ctor_prologue(cls, m, h, model)
        if not pro and re.fullmatch(r"\{\s*\}", body.strip()):
            # `CUIFSystemInfo() { }` wrapped as `{ { } }` reads to tree-sitter-cpp as a
            # declaration with a brace initializer: the scanner dropped the definition, and
            # every `CUIFSystemInfo__make_0` called a `__ctor_0` that was never translated.
            body = "{\n}"
        else:
            body = "{\n" + "\n".join(pro) + "\n    " + body + "\n}"
    head = f"// {cls.qualified}::{m.name} -- {m.body[0] if m.body else ''}"
    return _strip_ns(f"{head}\n{ret} {n}__{m.rust_name}{suffix}({params})\n{body}\n", model)


def _implicit_ctor(cls: ClassModel, h: Hierarchy, model: ModuleModel) -> str:
    pro = _ctor_prologue(cls, None, h, model)
    body = "{\n" + "\n".join(pro) + "\n}" if pro else "{\n}"
    return _strip_ns(
        f"// {cls.qualified}: the implicit default constructor\n"
        f"void {cls.name}____ctor_0({cls.name} *this_)\n{body}\n",
        model,
    )


# ---------------------------------------------------------------------------------------
# assembly
# ---------------------------------------------------------------------------------------

_FN_HEAD_RE = re.compile(
    r"(?P<attrs>(?:#\[[^\]]*\]\s*)*)(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?fn\s+"
    # `<'a>`: a worker's accessor returning a reference names its lifetime
    # (mmc_new's `CRecentFileEntry__FileName<'a>(this_..) -> &'a CAtlString`); without it here
    # the item stayed a free fn, and every caller's `entry.FileName()` failed.
    r"(?P<name>[A-Za-z_]\w*)\s*(?P<gen><[^>(]*>)?\s*\(\s*"
    r"(?P<this>this_\s*:\s*\*(?:mut|const)\s+[\w:]+)?\s*,?\s*"
)


def assemble(rust: str, model: ModuleModel) -> str:
    """Move each translated `<Class>__<name>(this_: *mut Class, ..)` into `impl <Class>`."""
    classes = {c.name for c in model.classes.values()}
    out: list[str] = []
    impls: dict[str, list[str]] = {}
    pos = 0
    externs = _extern_block_spans(rust)
    for m in _FN_HEAD_RE.finditer(rust):
        name = m.group("name")
        # The LONGEST class the name starts with: a nested class's name contains its outer
        # class's and `__` (`CTextEventSink__anon0____ctor_0`), and the first match out of a
        # set put the union's constructor on CTextEventSink as `anon0____ctor_0`.
        cls = max((c for c in classes if name.startswith(c + "__")), key=len, default=None)
        if cls is None or m.start() < pos:
            continue
        # A DECLARATION is not a method: inside an `extern {}` block, or a head whose `;`
        # comes before any `{`. A worker that declared `CImage__s_gdiplus_X(..);` in an
        # extern block had it moved into `impl CImage`, tearing the block apart.
        if any(lo <= m.start() < hi for lo, hi in externs):
            continue
        semi, brace = rust.find(";", m.end()), rust.find("{", m.end())
        if brace < 0 or (0 <= semi < brace and "(" not in rust[semi:brace]):
            continue
        end = _fn_end(rust, m.end())
        out.append(rust[pos : m.start()])
        method = name[len(cls) + 2 :]
        text = rust[m.start() : end]
        head_end = m.end() - m.start()
        rest = text[head_end:]
        gen = m.group("gen") or ""
        if m.group("this"):
            sig = f"    pub unsafe fn {method}{gen}(&mut self" + (
                "" if rest.lstrip().startswith(")") else ", "
            )
            brace = rest.index("{")
            rest = rest[: brace + 1] + "\n    let this_: *mut Self = self;" + rest[brace + 1 :]
        else:
            # A stdcall static member is a callback the OS calls: keep its ABI (the model's
            # own `extern "system"`, or the model's record of the C++ declaration).
            target = next((c for c in model.classes.values() if c.name == cls), None)
            decl = next(
                (x for x in (target.methods if target else []) if x.rust_name == method), None
            )
            system = (decl is not None and decl.abi == "system") or (
                'extern "system"' in rust[m.start() : m.end()]
            )
            abi = 'extern "system" ' if system else ""
            sig = f"    pub unsafe {abi}fn {method}{gen}("
        impls.setdefault(cls, []).append(sig + rest.replace("\n", "\n    "))
        pos = end
    out.append(rust[pos:])
    for cls, fns in impls.items():
        out.append(f"\nimpl {cls} {{\n" + "\n".join(fns) + "\n}\n")
    return "".join(out)


def unthis_globals(rust: str, model: ModuleModel) -> str:
    """`(*this_).selectionModel` -> `selectionModel` where `selectionModel` is a module
    GLOBAL and no class has a field of that name: the lowered C says `selectionModel.x`,
    and a worker that invents the member (mspaint's OnCommand, 100+ sites) is wrong in a
    way that has exactly one reading."""
    fields = {f.name for c in model.classes.values() for f in c.fields}
    names = {g.name for g in model.globals} - fields
    if not names:
        return rust
    alt = "|".join(sorted(map(re.escape, names), key=len, reverse=True))
    return re.sub(rf"\(\*this_\)\.(?=(?:{alt})\b)", "", rust)


def qualify_static_calls(rust: str, model: ModuleModel) -> str:
    """A bare `_shared()` where `_shared` is a static method of exactly one class and
    nothing else in the module (no free function, no other class's method) is that class's
    `CImageDx::_shared()`: the lowered C said so, and a worker inside `impl CImageDx`
    dropped the qualifier, which Rust does not allow for an associated fn."""
    owners: dict[str, set[str]] = {}
    for c in model.classes.values():
        for m in c.methods:
            owners.setdefault(m.rust_name or m.name, set()).add(c.name if m.static else "")
    free = {f.name for f in model.functions} | {g.name for g in model.globals}
    names = {n for n, o in owners.items() if len(o) == 1 and "" not in o and n not in free}
    if not names:
        return rust
    alt = "|".join(sorted(map(re.escape, names), key=len, reverse=True))

    def fix(m: re.Match) -> str:
        if re.search(r"\bfn\s+$", rust[max(0, m.start() - 8) : m.start()]):
            return m.group(0)  # the definition itself
        return f"{next(iter(owners[m.group(1)]))}::{m.group(0)}"

    return re.sub(rf"(?<![\w.:>])({alt})\s*\(", fix, rust)


def _reached_by_calls(model: ModuleModel, ext: list[ClassModel]) -> set[tuple[str, str]]:
    """External reach from the calls the AST RESOLVED (cpp_lower records them), not from
    names: `img.Attach(..)` on a CImageDx no longer reaches CImage::Attach. mspaint reached
    21 CImage methods by name and needs 8. An unresolved call ("*", name) -- an inherited
    ATL window-template member through an implicit `this` -- is CWindow's."""
    ext_names = {c.name for c in ext}
    by = {(c.name, m.rust_name): m for c in ext for m in c.methods}
    plain_cwindow = any(
        not c.external and not atl_template(c) and any(short(b.name) == "CWindow" for b in c.bases)
        for c in model.classes.values()
    )
    glue_only = frozenset() if plain_cwindow else ATL_IMPL_METHODS

    def targets(calls) -> set[tuple[str, str]]:
        out: set[tuple[str, str]] = set()
        for cls, name in calls:
            if cls == "*":
                # Unresolved only through an ATL window template base: CWindow's member.
                out |= {k for k in by if k[0] == "CWindow" and k[1] == name}
            elif cls in ext_names:
                out.add((cls, name))
        return {k for k in out if k in by and not (k[0] == "CWindow" and k[1] in glue_only)}

    todo = set()
    for c in model.classes.values():
        if not c.external:
            for m in c.methods:
                todo |= targets(m.calls)
    for f in model.functions:
        todo |= targets(f.calls)
    # An external class's ctors and dtor are always translated (synthetic_tus: Drop calls
    # the dtor), so what THEY call is reached too: CImage's dtor calls Destroy.
    for c in ext:
        for m in c.methods:
            if m.kind in ("ctor", "dtor"):
                todo |= targets(m.calls)
    reached: set[tuple[str, str]] = set()
    while todo:
        k = todo.pop()
        if k in reached:
            continue
        reached.add(k)
        todo |= targets(by[k].calls) - reached
    return reached


def static_paths(rust: str, model: ModuleModel) -> str:
    """`CPaintToolBar__ToolBarWndProc` (the synthetic unit's name for a static method, used
    as a VALUE -- a window procedure pointer) -> `CPaintToolBar::ToolBarWndProc`, its name
    after assembly. Only for static methods: an instance method has no such value form."""
    pairs = {
        f"{c.name}__{m.rust_name or m.name}": f"{c.name}::{m.rust_name or m.name}"
        for c in model.classes.values()
        for m in c.methods
        if m.static
    }
    if not pairs:
        return rust
    alt = "|".join(sorted(map(re.escape, pairs), key=len, reverse=True))

    def fix(m: re.Match) -> str:
        if re.search(r"\bfn\s+$", rust[max(0, m.start() - 8) : m.start()]):
            return m.group(0)
        return pairs[m.group(0)]

    return re.sub(rf"\b(?:{alt})\b", fix, rust)


def publish_header_statics(rust: str) -> str:
    """A `static inline` function in a module HEADER (toolsmodel.h's `UnZoomed`) is one copy
    per including TU in C++, and every TU calls it. Translated, it is one private `fn` in the
    header's unit, which the crate prelude re-exports: E0603. One copy visible to the crate is
    the same program. `pub`, not `pub(crate)`: the prelude's `pub use` of a `pub(crate)` item
    is E0364. Top-level items only; already-public ones are left as they are."""
    return re.sub(
        r'^(?:pub\(crate\) )?((?:unsafe )?(?:extern "[\w-]+" )?fn )', r"pub \1", rust, flags=re.M
    )


def drop_global_qualifier(rust: str) -> str:
    """C++'s `::IsWindow(m_hWnd)` (the global function, not CWindow::IsWindow) copied into
    Rust, where a leading `::` names a CRATE: `cannot find crate IsWindow`. Dropped unless
    the name really is a crate (`::core::ptr`, `::atl_rust::`)."""

    def fix(m: re.Match) -> str:
        name = m.group(1)
        if name in ("core", "std", "alloc") or name.endswith(("_rust", "_sys")):
            return m.group(0)
        return name

    return re.sub(r"(?<![\w:>)\]])::([A-Za-z_]\w*)(?=\s*[(!])", fix, rust)


def _extern_block_spans(text: str) -> list[tuple[int, int]]:
    """(start, end) of every `extern "..." {` block."""
    spans = []
    for m in re.finditer(r'\bextern\s*(?:"[^"]*"\s*)?\{', text):
        depth, i = 0, m.end() - 1
        while i < len(text):
            depth += (text[i] == "{") - (text[i] == "}")
            if depth == 0:
                spans.append((m.start(), i + 1))
                break
            i += 1
    return spans


def _fn_end(text: str, start: int) -> int:
    i = text.index("{", start)
    depth = 0
    while i < len(text):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise CppAppError("unbalanced braces in a translated function")


# ---------------------------------------------------------------------------------------
# the worker contract
# ---------------------------------------------------------------------------------------

_CONVENTIONS = """
## C++ class context (deterministic — the classes are generated, and in scope)
This C++ was lowered to C before you saw it: a member function is a free function whose
first parameter `this_` is the object pointer, every member access is explicit, overloads
are already resolved to distinct names, and object creation is an explicit call.
Mandatory mapping:
- `this_->field` → `(*this_).field`; `this_->m(args)` → `(*this_).m(args)` — every class
  method is a Rust method taking `&mut self`. `p->m(args)` → `(*p).m(args)`, `o.m(args)` →
  `o.m(args)`.
- `__baseK` is the K-th base class subobject: `this_->__base1.x` → `(*this_).__base1.x`.
- `x.__ctor_N(args)` constructs `x` in place — keep it as a method call, never drop it.
- `T__make_N(args)` (an object by value), `T__new_N(args)` (heap), `T__delete(p)`,
  `T__zeroed()` are free functions in scope: call them by exactly these names.
- `cpp_new_array<T>(n)` → `cpp_new_array::<T>(n)`; `cpp_delete_array(p)`, `cpp_delete(p)`,
  `cpp_new::<T>(v)` likewise. Never use Box, Vec or alloc.
- A distinct name like `alloc_seconds__1` IS the overload the C++ called: keep it exactly.
- A C++ reference parameter `T &x` → `x: &mut T`, `const T &x` → `x: &T`.
- OUTPUT SHAPE: a free function with the C function's own name and its `this_: *mut T`
  first parameter, exactly as given. Do NOT write it as a method (`&mut self`): rustconv
  moves it into the class itself, by that name.
"""


def worker_contract(model: ModuleModel, worker: str, body: str) -> str:
    """The class context for one synthetic-TU worker: conventions, plus the Rust shape of
    each module class the worker's text names (its own class first)."""
    h = Hierarchy(model)
    names = sorted(
        (
            c
            for c in h.by_short.values()
            if re.search(rf"\b{re.escape(c.name)}\b", body + " " + worker)
        ),
        key=lambda c: (not worker.startswith(c.name + "__"), c.name),
    )
    if not names and "this_" not in body:
        return ""
    parts = [_CONVENTIONS]
    own = next((c for c in names if worker.startswith(c.name + "__")), None)
    if own is not None:
        member = worker[len(own.name) + 2 :].removesuffix("__body")
        static = any(m.static and m.rust_name == member for m in own.methods)
        if static:
            # A STATIC member has no object: told to take `this_`, the model turned
            # audio_wavein::recording_procedure (a CreateThread routine) into a method.
            system = any(
                m.static and m.rust_name == member and m.abi == "system" for m in own.methods
            )
            abi = 'extern "system" ' if system else ""
            parts.append(
                f"\nRequired signature: `unsafe {abi}fn {worker}(...)` with exactly the C's "
                "parameters -- a STATIC member function: there is no `this_`.\n"
            )
        else:
            parts.append(
                f"\nRequired signature: `unsafe fn {worker}(this_: *mut {own.name}, ...)` "
                "with the remaining parameters as in the C.\n"
            )
    for cls in names[:6]:
        parts.append(f"\n`{cls.name}`:\n```rust\n{_class_api(cls, h, model)}\n```\n")
    return "".join(parts)


def _class_api(cls: ClassModel, h: Hierarchy, model: ModuleModel) -> str:
    lines = [f"pub struct {cls.name} {{"]
    if h.has_own_vtbl(cls):
        lines.append(f"    pub __vtbl: *const {cls.name}__vtbl,")
    for i, b in enumerate(cls.bases):
        bn = short(b.name)
        lines.append(
            f"    pub __base{i}: {bn if bn in h.by_short else _external_base(b.name, model)},"
        )
    for f in cls.fields:
        if not f.static:
            lines.append(f"    pub {_ident(f.name)}: {rust_type(f.c_type, model, field=True)},")
    lines.append("}")
    lines.append(f"// methods, called on a `{cls.name}` object `o` as `o.m(args)`:")
    reached = external_reached(model) if cls.external else set()
    for m in cls.methods:
        if m.kind == "dtor" or m.static:
            continue
        if cls.external and (cls.name, m.rust_name) not in reached:
            continue
        params = ", ".join(f"{a}: {t}" for a, t in _params(m, model))
        lines.append(f"//   {m.rust_name}({params}){_ret(m, model)}")
    return "\n".join(lines)
