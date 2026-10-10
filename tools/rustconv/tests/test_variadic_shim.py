import pytest

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
    assert "let mut args_va: va_list = *(ap_list as *mut va_list);" in out.text
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


def test_a_module_local_pointer_typedef_spells_void_star():
    # cmd's DirPrintf(LPDIRSWITCHFLAGS, ...): the typedef lives in dir.rs, not the island.
    # integrate lists the twin's own file FIRST, so it resolves and shadows the island.
    local = "pub type LPDIRSWITCHFLAGS = *mut DIRSWITCHFLAGS;\npub type T = u16;\n"
    island = "pub type T = u32;\n"
    maps = v.alias_maps([[local, island]] * 3)
    assert v.c_primitive("LPDIRSWITCHFLAGS", maps) == "void *"
    assert v.c_primitive("T", maps) == v.c_primitive("u16", maps)
    with pytest.raises(ValueError, match="stuck at `LPDIRSWITCHFLAGS`"):
        v.c_primitive("LPDIRSWITCHFLAGS", v.alias_maps([[island]] * 3))


def test_a_va_list_parameter_or_field_type_is_the_pointer_it_is_on_windows():
    # cmd's ConPrintfVPaging(.., arg_ptr: VaList) and Wine's dbg_vlog field: c_variadic.
    src = (
        'pub dbg_vlog: Option<unsafe extern "C" fn(*const c_char, ::core::ffi::VaList) -> c_int>,\n'
        "fn P(p: BOOL, arg_ptr: ::core::ffi::VaList<'_, '_>) -> BOOL {\n"
        "    let mut ap: ::core::ffi::VaListImpl;\n}\n"
    )
    out, n = v.lower_va_list_types(src)
    assert n == 2
    assert "fn(*const c_char, *mut ::core::ffi::c_void)" in out
    assert "arg_ptr: *mut ::core::ffi::c_void)" in out
    assert "VaListImpl" in out  # a twin body's walking is the variadic rewrite's
    # cmd's dir.rs `as *const ::core::ffi::VaList`: never split the path's own `::`.
    out, n = v.lower_va_list_types("read(p as *const ::core::ffi::VaList)")
    assert (out, n) == ("read(p as *const *mut ::core::ffi::c_void)", 1)


def test_an_existing_twin_loses_its_va_start_even_when_it_is_not_adjacent():
    # cmd's ErrorMessage_v after the paid fix stage: the list is started inside an `if`.
    src = (
        'pub unsafe extern "C" fn ErrorMessage_v(e: DWORD, szFormat: PCWSTR, '
        "ap_list: *mut ::core::ffi::c_void) {\n"
        "    let mut arg_ptr: va_list;\n"
        "    if !szFormat.is_null() {\n"
        "        va_start!(arg_ptr, szFormat);\n"
        "        _vswprintf(buf, szFormat, arg_ptr);\n"
        "        va_end!(arg_ptr);\n"
        "    }\n}\n"
    )
    out, n = v.lower_twin_va_starts(src)
    assert n == 1
    assert "arg_ptr = *(ap_list as *mut va_list);" in out
    assert "va_start!" not in out and "va_end!" not in out
    assert "let mut arg_ptr: va_list;" in out
    assert v.lower_twin_va_starts(out) == (out, 0)


def test_a_static_variadics_twin_is_still_unmangled():
    # cmd's `static BOOL DirPrintf(LPDIRSWITCHFLAGS, LPTSTR, ...)`: the trampoline calls
    # DirPrintf_v by name, so the twin needs #[no_mangle] the definition never had.
    src = (
        'unsafe extern "C" fn DirPrintf(f: LPDIRSWITCHFLAGS, s: LPTSTR, mut args: ...)'
        " -> BOOL {\n    0\n}\n"
    )
    out = v.rewrite(src)
    assert out.names == ["DirPrintf"]
    assert '#[no_mangle]\nunsafe extern "C" fn DirPrintf_v(' in out.text
