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

from .cpp_core import c_type_to_rust
from .cpp_model import ATL_BASE_ALIASES, ATL_IMPL_METHODS, ClassModel, Method, ModuleModel


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
    n = name.split("<")[0].split("::")[-1].strip()
    return ATL_BASE_ALIASES.get(n, n)


def atl_template(cls: ClassModel) -> str | None:
    """`CWindowImpl` / `CDialogImpl` when the class derives from one (its first base)."""
    for b in cls.bases:
        t = b.name.split("<")[0].split("::")[-1].strip()
        if t in ("CWindowImpl", "CDialogImpl"):
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
    return [(p.name or f"a{i}", rust_type(p.c_type, model)) for i, p in enumerate(m.params)]


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
    model: ModuleModel, island_structs: frozenset[str] = frozenset(), island: str = ""
) -> str:
    """`island_structs`: names the crate's bindgen island already lays out. A plain struct
    (no methods, no bases, no vtable) the island has is the island's -- its C layout is the
    authority, and a second definition makes every use of the name ambiguous (E0659).
    `island`: its text, for the COM interfaces a class implements (their IFooVtbl)."""
    h = Hierarchy(model, parse_com_vtbls(island))
    h.island_names = frozenset(
        re.findall(r"^\s*pub (?:type|struct|union|enum) (\w+)", island, re.M)
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
    out.append(_render_globals(model, h))
    return "\n".join(out) + "\n"


def _plain(cls: ClassModel) -> bool:
    return not cls.methods and not cls.bases and not cls.polymorphic


def _render_class(
    cls: ClassModel, h: Hierarchy, model: ModuleModel, island_owned: bool = False
) -> str:
    n = cls.name
    # A class nothing destroys meaningfully is a C++ trivially-copyable value: Copy gives
    # Rust the implicit copy constructor's semantics, so `fmt = A44100_16BIT_STEREO` is a
    # copy rather than a move out of a static.
    derive = "" if has_drop(cls, h) else "#[derive(Clone, Copy)]\n"
    kw = "union" if cls.tag == "union" else "struct"
    lines = [f"// ---- {cls.qualified} ({cls.file})", f"{derive}#[repr(C)]", f"pub {kw} {n} {{"]
    if h.has_own_vtbl(cls):
        lines.append(f"    pub __vtbl: *const {n}__vtbl,")
    for i, b in enumerate(cls.bases):
        bn = short(b.name)
        if bn in h.by_short:
            lines.append(f"    pub __base{i}: {bn},")
        else:
            lines.append(f"    pub __base{i}: {_external_base(b.name)},")
    for f in cls.fields:
        if f.static:
            continue
        ty = _nested_names(rust_type(f.c_type, model, field=True), cls, h.island_names)
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
            lines.append(
                f"pub static mut {n}__{f.name}: {rust_type(f.c_type, model, field=True)} = "
                "unsafe { core::mem::zeroed() };"
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
                thunks.append(
                    f"unsafe fn {n}__vt_{tag}_{m.rust_name}({sig}){_ret(m, model)} {{ {body} }}"
                )
                out.append(f"{m.rust_name}: {n}__vt_{tag}_{m.rust_name}")
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
    if atl_template(cls):
        lines.append(_render_atl_glue(cls, h, model))
    elif cls.bases and short(cls.bases[0].name) not in h.by_short and not h.com_bases(cls):
        # An SDK struct base (CRect : tagRECT): C++ reaches its fields as the class's own
        # (`rc.top`), so the class derefs to it.
        base = _external_base(cls.bases[0].name)
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
)


def has_drop(cls: ClassModel, h: Hierarchy) -> bool:
    """A destructor with a body that does something, here or in any base or member."""
    if any(m.kind == "dtor" and m.body and not _empty_body(m) for m in cls.methods):
        return True
    if any(not f.static and _ATL_OWNING.search(f.c_type) for f in cls.fields):
        return True
    if any(has_drop(b, h) for _, b in h.module_bases(cls)):
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
    for slot in h.com[iface]:
        impl = next((m for m in cls.methods if m.name == slot.name), None)
        owner = cls
        if impl is None:
            for _, b in h.module_bases(cls):
                impl = next((m for m in b.methods if m.name == slot.name), None)
                if impl is not None:
                    owner = b
                    break
        if impl is None:
            raise CppAppError(f"{cls.qualified} implements {iface} but defines no {slot.name}")
        via = h_path(h, cls, owner).rstrip(".")
        target = f"(*{_down(n, f'__base{k}.', 'This')})" + (f".{via}" if via else "")
        sig = ", ".join([f"This: *mut {iface}", *(f"{a}: {t}" for a, t in slot.params)])
        args = ", ".join(a for a, _ in slot.params)
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


def _handler(cls: ClassModel, name: str, arity: int) -> str:
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
                f"            *result = self.{_handler(cls, func, arity)}({call}, &mut handled) "
                "as isize;"
            )
            out.append("            if handled != 0 { return true; }")
            out.append("        }")
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


def _nested_names(ty: str, cls: ClassModel, island_names: frozenset[str]) -> str:
    """A type named bare inside its class (`FN_Shutdown` in CImageDx::SHARED) is the
    island's `CImageDx_FN_Shutdown`: bindgen spells a nested name Outer_Inner. Innermost
    enclosing scope first, as C++ lookup goes; a name the island has bare is left alone."""
    scopes = cls.qualified.split("::")
    prefixes = ["_".join(scopes[:k]) for k in range(len(scopes), 0, -1)]

    def fix(m: re.Match) -> str:
        w = m.group(0)
        if w in island_names:
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


def _render_atl_glue(cls: ClassModel, h: Hierarchy, model: ModuleModel) -> str:
    n, t = cls.name, atl_template(cls)
    base = next(f"__base{i}" for i, b in enumerate(cls.bases) if short(b.name) == "CWindow")
    lines = [
        f"impl core::ops::Deref for {n} {{ type Target = CWindow; "
        f"fn deref(&self) -> &CWindow {{ &self.{base} }} }}",
        f"impl core::ops::DerefMut for {n} {{ "
        f"fn deref_mut(&mut self) -> &mut CWindow {{ &mut self.{base} }} }}",
        f"impl atl_rust::win::MessageMap for {n} {{",
        "    unsafe fn process_window_message(&mut self, hwnd: atl_rust::win::HWND, msg: u32, "
        "w: usize, l: isize, result: &mut isize, map_id: u32) -> bool {",
        *_render_msg_map(cls, h),
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
        lines.append(
            "    /// ATL `CWindowImpl::Create(hWndParent, rect, szWindowName, dwStyle, dwExStyle, "
            "MenuOrID)`."
        )
        lines.append(
            "    pub unsafe fn Create(&mut self, parent: HWND, rect: *mut RECT, name: LPCWSTR, "
            "style: DWORD, ex_style: DWORD, menu_or_id: UINT) -> HWND {"
        )
        lines.append(
            "        atl_rust::win::create_window("
            f"self as *mut Self, &{n}__WND_CLASS, parent as _, "
            "rect as *const i32, name as *const u16, style as u32, ex_style as u32, "
            "menu_or_id as usize as *mut core::ffi::c_void) as HWND"
        )
        lines.append("    }")
        lines.append("    pub unsafe fn SubclassWindow(&mut self, hwnd: HWND) -> BOOL {")
        lines.append("        atl_rust::win::subclass_window(self as *mut Self, hwnd as _) as BOOL")
        lines.append("    }")
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
    lines[glue_start:] = _drop_hidden_glue(lines[glue_start:], own)
    lines.append("}")
    if t == "CWindowImpl":
        lines.extend(_wnd_class_info(cls))
    return "\n".join(lines)


def _external_base(name: str) -> str:
    """A base class the module does not define. ATL's window templates never get here (they
    alias to the module's CWindow); what does is a plain C struct from the SDK --
    `BITMAPINFOEX : BITMAPINFO` -- which is embedded by value as the island lays it out. A
    zero-size placeholder here would shift every field after it, silently."""
    if "<" in name:
        raise CppAppError(f"external template base {name} has no Rust layout; port it first")
    return name.split("::")[-1].strip()


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
            f"{_ident(f.name)}: ({_strip_ns(v, model)}) as _"
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


def _render_globals(model: ModuleModel, h: Hierarchy) -> str:
    """Module globals of CLASS type, zeroed until `__cxx_global_init` constructs them."""
    out = []
    for g in model.globals:
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
    return head + "\n".join(out) + "\n"


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


def synthetic_tus(model: ModuleModel, island: str = "") -> dict[str, str]:
    """{body file (repo-relative): C-like text}: every lowered body, grouped by the file
    its text lives in, as free functions a worker can translate one at a time. `island` as
    for render_classes: a COM interface method is a plain body, not a `__body` behind a
    dispatcher -- the two must agree or assemble's names miss the vtable's."""
    h = Hierarchy(model, parse_com_vtbls(island))
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
        if aggregate_static(g, h, model) is not None:
            continue  # rendered in cpp_classes.rs, field by field
        decl = _c_declarator(g.c_type, g.name)
        init = f" = {g.init}" if g.init and not g.init.startswith("(") else ""
        heads.setdefault(g.file, []).append(
            _strip_ns(f"{'static ' if g.static else ''}{decl}{init};", model)
        )
    for file, decls in heads.items():
        by_file[file] = ["\n".join(decls) + "\n", *by_file.get(file, [])]
    init = _global_init(model, h)
    for f in model.functions:
        body = f.lowered
        if init and f.name in _ENTRY_POINTS:
            # C++ runs static constructors before the entry point; the Rust entry runs them
            # first thing instead (there is no CRT initializer table for a Rust staticlib).
            brace = body.index("{")
            body = body[: brace + 1] + "\n    __cxx_global_init();" + body[brace + 1 :]
        by_file.setdefault(f.file, []).append(_strip_ns(f"{f.signature}\n{body}\n", model))
    if init:
        entry = next((f.file for f in model.functions if f.name in _ENTRY_POINTS), None)
        if entry is None:
            raise CppAppError("module globals need construction but no entry point was found")
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


_CALL_RE = re.compile(r"(?:\.|->)\s*([A-Za-z_]\w*)\s*\(")


def external_reached(model: ModuleModel) -> set[tuple[str, str]]:
    """(class, method) pairs of EXTERNAL classes (ATL's CWindow: 134 inline methods) that the
    module's code calls, directly or through another reached method. Only these are
    translated -- mspaint reaches about 45 -- and a name shared with a module method is
    included rather than missed."""
    ext = [c for c in model.classes.values() if c.external]
    if not ext:
        return set()
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
    called |= set(re.findall(r"\b\w+::\s*([A-Za-z_]\w*)\s*\(", joined))
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


def _c_declarator(c_type: str, name: str) -> str:
    """`TCHAR[100]` + `szTitle` -> `TCHAR szTitle[100]`; `void (*)(int)` -> `void (*name)(int)`."""
    arr = re.match(r"^(.*?)\s*((?:\[[^\]]*\])+)$", c_type)
    if arr:
        return f"{arr.group(1)} {name}{arr.group(2)}"
    if "(*)" in c_type:
        return c_type.replace("(*)", f"(*{name})", 1)
    return f"{c_type} {name}"


_ENTRY_POINTS = ("wWinMain", "WinMain", "_tWinMain", "wmain", "main", "_tmain", "DllMain")


def _global_init(model: ModuleModel, h: Hierarchy) -> str:
    """`audio_format UNKNOWN_FORMAT(0, 0, 0);` -> a constructor call in __cxx_global_init,
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
            and not any(f.default_init for f in cls.fields)
        ):
            continue  # zeroed storage IS its value-initialized state
        arg = g.init if g.init and g.init.startswith("(") else "()"
        idx = _match_ctor(cls, arg) if ctors else 0
        calls.append(f"    {g.name}.__ctor_{idx}{arg};")
    if not calls:
        return ""
    return _strip_ns(
        "// C++ static initialization of the module's global objects\n"
        "void __cxx_global_init(void)\n{\n" + "\n".join(calls) + "\n}\n",
        model,
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
    return lines


def _ctor_call(base: ClassModel, arg: str) -> tuple[int, str]:
    """(constructor index, argument list) for constructing `base` with `arg` as written,
    trailing DEFAULT arguments filled in: a base left to default-construct through
    `CWindow(HWND hWnd = NULL)` is `__ctor_0(NULL)`, never `__ctor_0()`."""
    written = [] if arg.strip() in ("()", "") else _split(arg.strip()[1:-1])
    ctors = [m for m in base.methods if m.kind == "ctor"]
    for i, m in enumerate(ctors):
        n = len(written)
        if len(m.params) >= n and all(p.default is not None for p in m.params[n:]):
            full = [w.strip() for w in written] + [p.default for p in m.params[n:]]
            return i, "(" + ", ".join(full) + ")"
    return _match_ctor(base, arg), arg


def _match_ctor(base: ClassModel, arg: str) -> int:
    n = 0 if arg.strip() in ("()", "") else len(_split(arg.strip()[1:-1]))
    ctors = [m for m in base.methods if m.kind == "ctor"]
    for i, m in enumerate(ctors):
        if len(m.params) == n:
            return i
    return 0


def _synthetic_method(cls: ClassModel, m: Method, h: Hierarchy, model: ModuleModel) -> str:
    n = cls.name
    params = ", ".join([f"{n} *this_", *(f"{p.c_type} {p.name}" for p in m.params)])
    if m.static:
        params = ", ".join(f"{p.c_type} {p.name}" for p in m.params) or "void"
    suffix = (
        "__body" if (h.is_virtual(cls, m) and m.kind not in ("ctor",)) or m.kind == "dtor" else ""
    )
    ret = m.return_type if m.kind == "method" or m.kind == "operator" else "void"
    body = m.lowered or "{}"
    if m.kind == "ctor":
        pro = _ctor_prologue(cls, m, h, model)
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
    r"(?P<name>[A-Za-z_]\w*)\s*\(\s*(?P<this>this_\s*:\s*\*(?:mut|const)\s+[\w:]+)?\s*,?\s*"
)


def assemble(rust: str, model: ModuleModel) -> str:
    """Move each translated `<Class>__<name>(this_: *mut Class, ..)` into `impl <Class>`."""
    classes = {c.name for c in model.classes.values()}
    out: list[str] = []
    impls: dict[str, list[str]] = {}
    pos = 0
    for m in _FN_HEAD_RE.finditer(rust):
        name = m.group("name")
        cls = next((c for c in classes if name.startswith(c + "__")), None)
        if cls is None:
            continue
        end = _fn_end(rust, m.end())
        out.append(rust[pos : m.start()])
        method = name[len(cls) + 2 :]
        text = rust[m.start() : end]
        head_end = m.end() - m.start()
        rest = text[head_end:]
        if m.group("this"):
            sig = f"    pub unsafe fn {method}(&mut self" + (
                "" if rest.lstrip().startswith(")") else ", "
            )
            brace = rest.index("{")
            rest = rest[: brace + 1] + "\n    let this_: *mut Self = self;" + rest[brace + 1 :]
        else:
            sig = f"    pub unsafe fn {method}("
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
            parts.append(
                f"\nRequired signature: `unsafe fn {worker}(...)` with exactly the C's "
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
        lines.append(f"    pub __base{i}: {bn if bn in h.by_short else _external_base(b.name)},")
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
