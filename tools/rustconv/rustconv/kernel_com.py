"""Rust ownership for the bounded CUnknownImpl<one-interface> kernel model.

Private C++ layout is deliberately not reproduced: only the SDK interface escapes.
The generated owner embeds that interface at offset zero, with its compiler-derived
vtable type. Constructor state, destructor frees and the atomic base are checked
before emitting anything. QI/factories remain visible Rust implementation hooks.
Unsupported inheritance or lifetime code is an error, never a C++ runtime fallback.
"""

import re

from .cpp_bridge import BridgeError
from .cpp_com import _balanced_block, _state_fields, c_type_to_rust
from .lex_mask import mask_code


def _tokens(text):
    return re.sub(r"\s+", "", mask_code(text))


_BASE = """class CUnknownImpl : public Interfaces... {
private: volatile LONG m_Ref;
protected: CUnknownImpl() : m_Ref(0) {} virtual ~CUnknownImpl() {}
public:
STDMETHODIMP_(ULONG) AddRef() {
 ULONG Ref = InterlockedIncrement(&m_Ref); ASSERT(Ref < 0x10000); return Ref;
}
STDMETHODIMP_(ULONG) Release() {
 ULONG Ref = InterlockedDecrement(&m_Ref); ASSERT(Ref < 0x10000);
 if (!Ref) { delete this; return 0; } return Ref;
}
}"""


def generate_owner(source, base_source, scan, class_name, bridge_header):
    """Return Rust; imports/hooks are explicit so missing implementations stay red."""
    masked = mask_code(source)
    head = re.search(
        r"\bclass\s+"
        + re.escape(class_name)
        + r"\s*:\s*public\s+CUnknownImpl\s*<\s*(\w+)\s*>\s*\{",
        masked,
    )
    if head is None:
        raise BridgeError("kernel owner requires exactly one CUnknownImpl interface base")
    interface = head[1]
    _, end = _balanced_block(masked, head.end() - 1)
    body = source[head.end() : end]
    base_mask = mask_code(base_source)
    base_head = re.search(r"\bclass\s+CUnknownImpl\s*:", base_mask)
    if base_head is None:
        raise BridgeError("missing CUnknownImpl definition")
    opening = base_mask.index("{", base_head.end())
    _, base_end = _balanced_block(base_mask, opening)
    if _tokens(base_source[base_head.start() : base_end + 1]) != _tokens(_BASE):
        raise BridgeError("unsupported CUnknownImpl atomic/lifetime implementation")
    fields = {}
    for declaration in _state_fields(body):
        if declaration.strip() == "IMP_" + interface:
            continue
        field = re.fullmatch(r"(\w+)\s+(m_\w+)", declaration.strip())
        if field is None or field[1] not in {"PUNKNOWN", "POOL_TYPE", "PCM_RESOURCE_LIST", "ULONG"}:
            raise BridgeError(f"unsupported kernel state declaration: {declaration}")
        fields[field[2]] = field[1]
    ctor = re.search(
        re.escape(class_name) + r"\s*\(\s*IUnknown\s*\*\s*(\w+)\s*\)\s*:\s*([^{}]+)\{\s*\}",
        mask_code(body),
    )
    if ctor is None:
        raise BridgeError("unsupported kernel constructor")
    initializers = re.findall(r"(m_\w+)\s*\(\s*(\w+)\s*\)", ctor[2])
    residue = re.sub(r"m_\w+\s*\(\s*\w+\s*\)", "", ctor[2]).replace(",", "").strip()
    if residue or len(initializers) != len(fields) or set(dict(initializers)) != set(fields):
        raise BridgeError("constructor must initialize each state field exactly once")
    values = {}
    for name, value in initializers:
        if value == "0":
            values[name] = (
                "core::ptr::null_mut()"
                if fields[name].startswith("P") and fields[name] != "POOL_TYPE"
                else "0"
            )
        elif value == ctor[1]:
            values[name] = "outer"
        elif value == "NonPagedPool" and fields[name] == "POOL_TYPE":
            values[name] = "NonPagedPool"
        else:
            raise BridgeError(f"unsupported constructor initializer {name}({value})")
    destructor = re.search(re.escape(class_name + "::~" + class_name) + r"\s*\(\s*\)\s*\{", masked)
    if destructor is None:
        raise BridgeError("missing destructor")
    _, dtor_end = _balanced_block(masked, destructor.end() - 1)
    dtor = masked[destructor.end() : dtor_end]
    free_pattern = r"if\s*\(\s*(m_\w+)\s*\)\s*\{\s*FreeItem\s*\(\s*\1\s*,\s*(\w+)\s*\)\s*;\s*\}"
    frees = re.findall(free_pattern, dtor)
    if re.sub(free_pattern, "", dtor).strip() or any(
        fields.get(name) != "PCM_RESOURCE_LIST" for name, _ in frees
    ):
        raise BridgeError("unsupported destructor: expected conditional tagged resource frees")
    context_fields = re.findall(r"\b(m_\w+)\s*;", bridge_header)
    if not context_fields or not set(context_fields) <= fields.keys():
        raise BridgeError("bridge context is not a subset of owned state")
    prefix = f"RustBridge_{class_name}"
    methods = [
        f
        for f in scan.functions
        if f.cpp_class == class_name and f.return_type and f.name != "QueryInterface"
    ]
    if not methods or len({f.name for f in methods}) != len(methods):
        raise BridgeError("missing or overloaded ordinary interface methods")
    if any(method.name in {"AddRef", "Release"} for method in methods):
        raise BridgeError("class overrides the supported atomic base lifetime")
    for method in methods:
        if prefix + "_" + method.name + "(" not in bridge_header:
            raise BridgeError(f"method lacks a retained worker: {method.name}")
    lines = [
        "// @generated by rustconv.kernel_com; private storage is Rust-owned.",
        "use crate::types::*;",
        "use ntkrnl_sys::ddk::*;",
        "use ntkrnl_sys::mm::NonPagedPool;",
        "use crate::resource_support::*;",
        "use core::sync::atomic::{AtomicI32, Ordering};",
        "#[repr(C)]",
        f"pub struct {class_name} {{",
        f"    pub interface: {interface},",
        "    m_Ref: AtomicI32,",
    ]
    lines += [f"    pub {name}: {typ}," for name, typ in fields.items()]
    lines += [
        "}",
        f"const _: () = assert!(core::mem::offset_of!({class_name}, interface) == 0);",
        f"static mut VTABLE: {interface}Vtbl = {interface}Vtbl {{",
        "    QueryInterface: Some(query_interface), AddRef: Some(add_ref), Release: Some(release),",
    ]
    lines += [f"    {m.name}: Some({m.name})," for m in methods]
    lines += [
        "};",
        "pub unsafe fn construct(pool: POOL_TYPE, tag: ULONG, outer: PUNKNOWN) "
        f"-> *mut {class_name} {{",
        f"    let object = allocate_object(pool, core::mem::size_of::<{class_name}>(), tag) "
        f"as *mut {class_name};",
        "    if object.is_null() { return object; }",
        f"    object.write({class_name} {{ interface: {interface} {{ lpVtbl: &raw mut VTABLE }}, "
        "m_Ref: AtomicI32::new(0),",
    ]
    lines += [f"        {name}: {value}," for name, value in values.items()]
    lines += ["    });", "    object", "}", f"pub unsafe fn destroy(object: *mut {class_name}) {{"]
    lines += [
        f"    if !(*object).{name}.is_null() {{ free_item((*object).{name}.cast(), {tag}); }}"
        for name, tag in frees
    ]
    lines += [
        "    free_object(object.cast());",
        "}",
        f'pub unsafe extern "system" fn add_ref(this: *mut {interface}) -> ULONG {{',
        f"    let object = this.cast::<{class_name}>();",
        "    let count = (*object).m_Ref.fetch_add(1, Ordering::SeqCst).wrapping_add(1) as ULONG;",
        "    kernel_assert!(count < 0x10000);",
        "    count",
        "}",
        f'pub unsafe extern "system" fn release(this: *mut {interface}) -> ULONG {{',
        f"    let object = this.cast::<{class_name}>();",
        "    let count = (*object).m_Ref.fetch_sub(1, Ordering::SeqCst).wrapping_sub(1) as ULONG;",
        "    kernel_assert!(count < 0x10000);",
        "    if count == 0 { destroy(object); }",
        "    count",
        "}",
        f"unsafe fn context(this: *mut {interface}) -> {prefix}_Context {{",
        f"    let object = this.cast::<{class_name}>();",
        f"    {prefix}_Context {{ object: object.cast(),",
    ]
    lines += [f"        {name}: &raw mut (*object).{name}," for name in context_fields]
    lines += ["    }", "}"]
    for method in methods:
        parameters = [f"{name}: {c_type_to_rust(typ)}" for typ, name in method.params]
        arguments = [name for _, name in method.params]
        lines += [
            f'pub unsafe extern "system" fn {method.name}('
            + ", ".join([f"this: *mut {interface}", *parameters])
            + f") -> {c_type_to_rust(method.return_type)} {{",
            "    let mut ctx = context(this);",
            f"    crate::resource::{prefix}_{method.name}("
            + ", ".join(["&raw mut ctx", *arguments])
            + ")",
            "}",
        ]
    return "\n".join(lines) + "\n"
