from rustconv import variadic_shim as v

RUST = """\
#[no_mangle]
pub unsafe extern "C" fn Notify(
    Kind: KIND,
    Ctx: PVOID,
    mut args: ...
) -> ULONG {
    let mut ap: ::core::ffi::VaListImpl;
    ap = args.clone();
    Use(ap.arg::<ULONG>())
}

pub unsafe extern "C" fn Plain(a: u32) {}
"""

C = """\
#include "mod.h"

ULONG
__cdecl
Notify(
    _In_ KIND Kind,
    _In_ PVOID Ctx,
    ...)
{
    return 0;
}
"""


def test_a_walking_variadic_becomes_a_twin_and_a_declaration():
    r = v.rewrite(RUST)
    assert r.names == ["Notify"] and not r.refused
    assert 'extern "C" fn Notify_v(' in r.text and "ap_list: *mut ::core::ffi::c_void" in r.text
    assert "VaArgs::from_ptr(ap_list)" in r.text and "VaListImpl" not in r.text
    assert "pub fn Notify(Kind: KIND, Ctx: PVOID, ...) -> ULONG;" in r.text
    assert "fn Plain(a: u32)" in r.text
    assert v.rewrite(r.text).names == []  # idempotent


def test_a_forwarded_list_is_refused():
    fwd = RUST.replace("Use(ap.arg::<ULONG>())", "vsnprintf(p, n, f, args.as_va_list())")
    r = v.rewrite(fwd)
    assert r.names == [] and "Notify" in r.refused and r.text == fwd


def test_the_trampoline_is_primitive_c_read_off_the_twin():
    twin = v.twins(v.rewrite(RUST).text)
    assert twin == {"Notify": ([("Kind", "KIND"), ("Ctx", "PVOID")], "ULONG")}
    island = "pub type KIND = _KIND;\npub type _KIND = ::core::ffi::c_uint;\n"
    i386 = island + "pub type PVOID = *mut c_void;\npub type ULONG = ::core::ffi::c_ulong;\n"
    maps = v.alias_maps([[i386], [i386], [i386]])
    t = v.c_trampoline("Notify", *twin["Notify"], maps)
    assert "extern unsigned long Notify_v(unsigned int Kind, void * Ctx, void *ap);" in t
    assert "unsigned long __cdecl Notify(unsigned int Kind, void * Ctx, ...)" in t
    assert "va_start(ap, Ctx);" in t and "r = Notify_v(Kind, Ctx, (void *)&ap);" in t
    # The C is the oracle that it IS variadic, and of what arity.
    assert v.c_variadic_arity(C, "Notify") == 2
    assert v.c_variadic_arity(C.replace("    ...)", "    int x)"), "Notify") is None


def test_a_type_whose_abi_differs_by_arch_is_refused():
    import pytest

    narrow = ["pub type T = ::core::ffi::c_uint;\n"]
    wide = ["pub type T = ::core::ffi::c_ulonglong;\n"]
    with pytest.raises(ValueError):
        v.c_primitive("T", v.alias_maps([narrow, wide, wide]))
    # ...except the one pointer-sized pair (ULONG_PTR).
    ptr = v.alias_maps([["pub type U = ::core::ffi::c_ulong;\n"], ["pub type U = u64;\n"]])
    assert v.c_primitive("U", ptr) == "__UINTPTR_TYPE__"


def test_a_forwarded_va_list_starts_from_the_trampolines():
    """progman's PrintString: C `va_start(args, szStr); PrintStringV(szStr, args); va_end(args);`,
    translated with a local va_list and the lowered va_start!/va_end!, which mean nothing in a
    Rust twin. The trampoline already did va_start, so the local is `*ap_list`."""
    from rustconv import variadic_shim

    src = (
        "#[no_mangle]\n"
        'pub unsafe extern "C" fn PrintString(szStr: LPCWSTR, args: ...) {\n'
        "    let mut args_va: va_list;\n"
        "    va_start!(args_va, szStr);\n"
        "    PrintStringV(szStr, args_va);\n"
        "    va_end!(args_va);\n"
        "}\n"
    )
    out = variadic_shim.rewrite(src)
    assert out.names == ["PrintString"]
    assert "let args_va: va_list = *(ap_list as *mut va_list);" in out.text
    assert "va_start!" not in out.text and "va_end!" not in out.text


def test_the_extern_keeps_a_mut_pointer_type():
    # ftfd's sprintf: stripping every `mut` declared `buf: *i8`, which does not parse.
    src = (
        'pub unsafe extern "C" fn sprintf(mut buf: *mut i8, fmt: *const i8, mut args: ...)'
        " -> i32 {\n    0\n}\n"
    )
    out = v.rewrite(src).text
    assert "pub fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;" in out


def test_a_sal_annotated_ellipsis_is_still_variadic():
    # bmfd's own DbgPrint: integrate refused to generate its trampoline.
    c = "ULONG\nDbgPrint(IN PCHAR Format, IN ...)\n{\n    return 0;\n}\n"
    assert v.c_variadic_arity(c, "DbgPrint") == 1
