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


def test_a_va_start_local_is_the_callers_list_in_the_twin():
    # regedit's InfoMessageBox: `va_list args; va_start(args, lpMessage);` forwarded twice.
    src = """\
pub unsafe extern "C" fn Info(h: HWND, lpMessage: LPCWSTR, mut args: ...) -> c_int {
    let mut args: va_list = core::mem::zeroed();
    va_start!(args, lpMessage);
    let n = _vscwprintf(lpMessage, (args).cast());
    _vsnwprintf(buf, n as usize, lpMessage, args);
    va_end!(args);
    0
}
"""
    r = v.rewrite(src)
    assert r.names == ["Info"] and not r.refused
    assert "let mut args: va_list = *(ap_list as *mut va_list);" in r.text
    assert "va_start!" not in r.text and "va_end!" not in r.text
    assert "_vsnwprintf(buf, n as usize, lpMessage, args);" in r.text


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


def test_an_uninitialised_valistimpl_local_forwarded_by_as_va_list():
    # regedit's edit.c error(): `let mut ap: VaListImpl; va_start!(ap, resId);
    # _vsnwprintf(.., ap.as_va_list()); va_end!(ap);`
    src = """\
pub unsafe extern "C" fn error(hwnd: HWND, resId: INT, mut args: ...) {
    let mut ap: ::core::ffi::VaListImpl;
    va_start!(ap, resId);
    _vsnwprintf(s, 9, f, ap.as_va_list());
    va_end!(ap);
}
"""
    r = v.rewrite(src)
    assert r.names == ["error"] and not r.refused
    assert "let mut ap: va_list = *(ap_list as *mut va_list);" in r.text
    assert "_vsnwprintf(s, 9, f, ap);" in r.text and "va_start!" not in r.text


def test_a_cloned_list_only_handed_on_is_forwarded():
    # regedit's edit.c warning(): `ap = args.clone(); StringCbVPrintfW(.., (ap).cast());`
    src = """\
pub unsafe extern "C" fn warning(hwnd: HWND, resId: INT, mut args: ...) {
    let mut ap: ::core::ffi::VaListImpl;
    ap = args.clone();
    StringCbVPrintfW(s, 9, f, (ap).cast());
}
"""
    r = v.rewrite(src)
    assert r.names == ["warning"] and not r.refused
    assert "let mut ap: va_list = *(ap_list as *mut va_list);" in r.text
    assert "from_ptr" not in r.text and "StringCbVPrintfW(s, 9, f, (ap).cast());" in r.text


def test_a_static_variadics_twin_is_still_exported():
    # regedit's `static void txt_fprintf(FILE *, LPCWSTR, ...)`: the shim calls the twin by name.
    r = v.rewrite('unsafe extern "C" fn txt(fp: *mut FILE, mut args: ...) {\n    g(args.clone());\n}\n'.replace("g(args.clone())", "let mut ap: ::core::ffi::VaListImpl; ap = args.clone(); ap.arg::<i32>()"))
    assert "#[no_mangle]\nunsafe extern \"C\" fn txt_v(" in r.text


def test_a_wrapped_list_passed_by_address_is_the_callers_va_list_pointer():
    # regedit's ErrorMessageBox: FormatMessageW(.., &args) wants a `va_list *` -- ap_list.
    src = """\
pub unsafe extern "C" fn Emb(h: HWND, mut args: ...) -> c_int {
    let mut args_0: ::core::ffi::VaListImpl = args.clone();
    FormatMessageW(0, p, 1, 2, b, 0, (&raw mut args_0).cast());
    drop(args_0);
    0
}
"""
    r = v.rewrite(src)
    assert r.names == ["Emb"] and not r.refused
    assert "FormatMessageW(0, p, 1, 2, b, 0, ((ap_list as *mut va_list)).cast());" in r.text
    assert "args_0" not in r.text


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


def test_an_unnamed_ellipsis_is_still_variadic():
    # net.exe's PrintMessageStringV: the C's bare `...` survived translation unnamed.
    src = (
        "#[no_mangle]\n"
        'pub unsafe extern "C" fn PrintMessageStringV(dwMessage: DWORD, ...) {\n'
        "    let mut args: va_list;\n"
        "    va_start!(args, dwMessage);\n"
        "    ConMsgPrintfV(StdOut, 0, h, dwMessage, 0, core::ptr::addr_of_mut!(args));\n"
        "    va_end!(args);\n"
        "}\n"
    )
    out = v.rewrite(src)
    assert out.names == ["PrintMessageStringV"] and not out.refused
    assert (
        "fn PrintMessageStringV_v(dwMessage: DWORD, ap_list: *mut ::core::ffi::c_void)" in out.text
    )
    assert "let mut args: va_list = *(ap_list as *mut va_list);" in out.text
    assert "pub fn PrintMessageStringV(dwMessage: DWORD, ...);" in out.text


def test_the_parameter_itself_va_started_is_forwarded():
    # mode's UnderlinedResPrintf: va_start!(args, uID) on the `...` parameter.
    src = """\
pub unsafe extern "C" fn Urp(Stream: PCON_STREAM, uID: UINT, mut args: ...) {
    let mut Len: INT = 0;
    va_start!(args, uID);
    Len = ConResPrintfV(Stream, uID, args.as_va_list());
    va_end!(args);
}
"""
    r = v.rewrite(src)
    assert r.names == ["Urp"] and not r.refused
    assert "let mut args: va_list = *(ap_list as *mut va_list);" in r.text
    assert "ConResPrintfV(Stream, uID, args);" in r.text and "va_start!" not in r.text
