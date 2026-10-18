"""sdk_macros: SDK header macros lowered from the preprocessor's own expansion."""

import re

import pytest

from rustconv import sdk_macros
from rustconv.sdk_macros import LowerError, lower_expansion, macro_params, render_macro


def low(text: str) -> str:
    return lower_expansion(text, set())


def test_makewparam_keeps_cs_integer_promotion():
    """MAKELONG ORs a WORD with a DWORD: the WORD promotes, so both sides are DWORD."""
    out = low(
        "((WPARAM)(DWORD)((LONG)(((WORD)(((DWORD_PTR)(__rcv_p0__)) & 0xffff)) | "
        "((DWORD)((WORD)(((DWORD_PTR)(__rcv_p1__)) & 0xffff))) << 16)))"
    )
    assert "as WORD)) as DWORD) |" in out
    assert out.endswith("as WPARAM)")


def test_a_narrow_left_shift_operand_is_promoted():
    """`(WORD)x << 16` is an int shift in C; a u16 shift in Rust would drop every bit."""
    assert low("((WORD)(__rcv_p0__)) << 16") == "((((($p0) as WORD)) as c_int) << 16)"


def test_uncast_call_arguments_get_cs_implicit_conversion():
    out = low("SendMessageW(__rcv_p0__,(1024 +112),__rcv_p1__,0)")
    assert out == "SendMessageW((($p0) as _), (((1024 + 112)) as _), (($p1) as _), ((0) as _))"


def test_a_pointer_cast_under_an_integer_cast_is_dropped():
    out = low("(LPARAM)(const LVCOLUMNW *)(__rcv_p0__)")
    assert out == "(($p0) as LPARAM)"


def test_null_and_void():
    assert low("(HTREEITEM)((void *)0)") == "((core::ptr::null_mut::<c_void>()) as HTREEITEM)"
    assert low("((void)f(__rcv_p0__))") == "{ let _ = f((($p0) as _)); }"


def test_logical_operators_yield_a_c_int():
    out = low("((__rcv_p0__)->SystemS1 || (__rcv_p0__)->SystemS2)")
    assert out == (
        "((((((*$p0).SystemS1) as isize) != 0) || ((((*$p0).SystemS2) as isize) != 0)) as c_int)"
    )


@pytest.mark.parametrize(
    "text",
    [
        "L__rcv_p0__",  # TEXT's token pasting
        'f(L"x")',  # string literal
    ],
)
def test_out_of_grammar_is_refused(text):
    with pytest.raises(LowerError):
        low(text)


def test_an_object_like_alias_takes_its_targets_parameters():
    dump = (
        "#define MAKEINTRESOURCEW(i) ((LPWSTR)((ULONG_PTR)((WORD)(i))))\n"
        "#define MAKEINTRESOURCE MAKEINTRESOURCEW\n"
    )
    assert macro_params(dump)["MAKEINTRESOURCE"] == ["i"]


def test_render_accepts_a_trailing_comma():
    assert (
        render_macro("F", 1, "$p0")
        == "#[macro_export] macro_rules! F { ($p0:expr $(,)?) => { $p0 }; }"
    )


def test_text_call_sites_widen_with_a_nul():
    from rustconv.sdk_macros import widen_call_sites

    out, n = widen_call_sites("f(TEXT(\"ab\"), _T('0'))", {"TEXT", "_T"})
    assert n == 2
    assert out.startswith("f((&[0x0061, 0x0062, 0x0000u16]).as_ptr()")
    assert out.endswith("('0' as u16))")


def test_array_size_measures_the_object():
    out = low("(sizeof(__rcv_p0__) / sizeof((__rcv_p0__)[0]))")
    assert out == "(core::mem::size_of_val(&$p0) / core::mem::size_of_val(&$p0[(0) as usize]))"
    assert low("sizeof(RECT)") == "core::mem::size_of::<RECT>()"


def test_a_big_hex_literal_keeps_its_c_type_through_a_cast():
    """HKEY_CURRENT_USER: `(HKEY)(LONG_PTR)(LONG)0x80000001` -- sign-extended, not out of range."""
    assert low("((HKEY)(LONG_PTR)(LONG)0x80000001)") == (
        "((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY)"
    )


def test_a_statement_block_macro_declares_a_zeroed_local():
    out = low(
        "{ LVITEMW _ms_lvi; _ms_lvi.iSubItem = __rcv_p2__; "
        "SendMessageW((__rcv_p0__),(0x1000 +116),(WPARAM)(__rcv_p1__),"
        "(LPARAM)(LVITEMW *)&_ms_lvi);}"
    )
    assert out.startswith("{ let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.iSubItem")
    assert "core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM" in out


def test_a_character_literal_is_its_code():
    """mmsys SND_ALIAS_SYSTEMDEFAULT = sndAlias('S', 'D')."""
    out = low("(0 +((DWORD)(BYTE)('S')|((DWORD)(BYTE)('D')<<8)))")
    assert "((83) as BYTE)" in out and "((68) as BYTE)" in out


def test_text_around_an_already_wide_value_is_the_identity():
    from rustconv.sdk_macros import widen_call_sites

    out, n = widen_call_sites("f(TEXT(API_NAME.as_ptr()), x)", {"TEXT"})
    assert (out, n) == ("f((API_NAME.as_ptr()), x)", 1)


ISLAND_VTBL = """
#[repr(C)]
pub struct IZoneVtbl {
    pub QueryInterface: ::core::option::Option< unsafe extern "system" fn( This: *mut IZone, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void, ) -> HRESULT, >,
    pub Release: ::core::option::Option<unsafe extern "system" fn(This: *mut IZone) -> ULONG>,
    pub Destroy: ::core::option::Option<unsafe extern "system" fn(This: *mut IZone, dwEnum: DWORD)>,
}
"""


def test_widl_c_wrappers_come_from_the_island_vtbl(tmp_path):
    for arch in ("amd64", "arm64", "i386"):
        (tmp_path / f"types_{arch}.rs").write_text(ISLAND_VTBL)
    out = sdk_macros.com_wrappers(
        tmp_path, {"IZone_Release", "IZone_QueryInterface", "IZone_Destroy", "IZone_Nope", "x_y"}
    )
    assert set(out) == {"IZone_Release", "IZone_QueryInterface", "IZone_Destroy"}
    assert (
        "pub unsafe fn IZone_Release(This: *mut IZone) -> ULONG "
        "{ ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) }" in out["IZone_Release"]
    )
    assert "(This, riid, ppvObject)" in out["IZone_QueryInterface"]
    assert "-> " not in out["IZone_Destroy"].split("{")[0]


def test_a_widl_wrapper_the_arches_disagree_on_is_generated_per_arch(tmp_path):
    for arch in ("amd64", "arm64"):
        (tmp_path / f"types_{arch}.rs").write_text(ISLAND_VTBL)
    (tmp_path / "types_i386.rs").write_text(ISLAND_VTBL.replace("dwEnum: DWORD", "dwEnum: u64"))
    out = sdk_macros.com_wrappers(tmp_path, {"IZone_Destroy", "IZone_Release"})
    assert "cfg" not in out["IZone_Release"]
    destroy = out["IZone_Destroy"]
    assert destroy.count("pub unsafe fn IZone_Destroy") == 3
    assert (
        '#[cfg(target_arch = "x86")]\n#[inline]\npub unsafe fn IZone_Destroy(This: *mut IZone, dwEnum: u64)'
        in destroy
    )


def test_a_callback_parameter_arrow_does_not_end_the_slot(tmp_path):
    vtbl = ISLAND_VTBL.replace(
        "dwEnum: DWORD)>",
        'cb: ::core::option::Option<unsafe extern "C" fn(x: DWORD) -> BOOL>, n: DWORD) -> HRESULT>',
    )
    for arch in ("amd64", "arm64", "i386"):
        (tmp_path / f"types_{arch}.rs").write_text(vtbl)
    out = sdk_macros.com_wrappers(tmp_path, {"IZone_Destroy"})["IZone_Destroy"]
    assert "(This, cb, n)" in out and out.split("{")[0].rstrip().endswith("-> HRESULT")


def test_pointer_typedef_arithmetic_steps_elements(monkeypatch):
    """ndis's NDIS_GET_PACKET_STATUS: `(PUCHAR)p + NdisPacketOobOffset` is pointer
    arithmetic; a bare `+` on `*mut u8` does not compile, and PUCHAR names no `*`."""
    monkeypatch.setattr(sdk_macros, "_POINTER_TYPES", {"PUCHAR", "PNDIS_PACKET_OOB_DATA"})
    out = lower_expansion(
        "((PNDIS_PACKET_OOB_DATA)((PUCHAR)(__rcv_p0__) + "
        "(__rcv_p0__)->Private.NdisPacketOobOffset))->Status",
        {"PUCHAR", "PNDIS_PACKET_OOB_DATA"},
    )
    assert ".wrapping_add(((*$p0).Private.NdisPacketOobOffset) as usize)" in out
    assert " + (*$p0)" not in out


def test_a_callback_cast_coerces_and_reaches_an_integer_through_its_bits(monkeypatch):
    """taskmgr's ListView_SortItems: `(LPARAM)(PFNLVCOMPARE)(pfn)`. `Some(F) as PFNLVCOMPARE`
    and `Option<fn> as LPARAM` are both E0605; a coercion and a transmute are not."""
    monkeypatch.setattr(sdk_macros, "_OPTION_TYPES", {"PFNLVCOMPARE"})
    out = lower_expansion("(LPARAM)(PFNLVCOMPARE)(__rcv_p0__)", {"PFNLVCOMPARE", "LPARAM"})
    assert "let __cb: PFNLVCOMPARE = $p0; __cb" in out
    assert "core::mem::transmute::<PFNLVCOMPARE, usize>(" in out and " as PFNLVCOMPARE" not in out


def test_an_integer_cast_to_a_callback_is_refused(monkeypatch):
    """wininet's INTERNET_INVALID_STATUS_CALLBACK, `(INTERNET_STATUS_CALLBACK)(-1L)`: a
    sentinel pointer, not a fn: no Rust constant can hold it (const eval rejects a fn pointer
    to no fn), and `let __cb: Option<fn> = -1` does not compile either."""
    monkeypatch.setattr(sdk_macros, "_OPTION_TYPES", {"INTERNET_STATUS_CALLBACK"})
    with pytest.raises(LowerError):
        lower_expansion("((INTERNET_STATUS_CALLBACK)(-1L))", {"INTERNET_STATUS_CALLBACK"})


def test_pointer_minus_pointer_is_refused(monkeypatch):
    monkeypatch.setattr(sdk_macros, "_POINTER_TYPES", {"PUCHAR"})
    with pytest.raises(LowerError):
        lower_expansion("(PUCHAR)(__rcv_p0__) - (PUCHAR)(__rcv_p1__)", {"PUCHAR"})


def test_object_expansion_needs_the_macro_on_every_arch(monkeypatch, tmp_path):
    """winuser.h makes `SetWindowLongPtrW` a macro on i386 only. One arch's expansion is not
    unanimity: notepad's 64-bit subclassing was routed through the 32-bit SetWindowLongW."""
    dumps = {
        "i386": "#define SetWindowLongPtrW SetWindowLongW\n#define StringCchCopy StringCchCopyW\n",
        "amd64": "#define StringCchCopy StringCchCopyW\n",
        "arm64": "#define StringCchCopy StringCchCopyW\n",
    }
    monkeypatch.setattr(sdk_macros, "compile_argv", lambda _r, arch, _s: [arch])
    monkeypatch.setattr(sdk_macros, "_dump", lambda argv: dumps[argv[0]])

    def preprocess(argv, _source, probe):
        defined = dict(sdk_macros._OBJ_DEFINE.findall(dumps[argv[0]]))
        return re.sub(
            r"(__RCV_BEGIN_\w+__) (\w+) (__RCV_END__)",
            lambda m: f"{m.group(1)} {defined.get(m.group(2), m.group(2))} {m.group(3)}",
            probe,
        )

    monkeypatch.setattr(sdk_macros, "_preprocess", preprocess)
    got = sdk_macros.object_expansions(
        tmp_path, tmp_path / "x.c", {"SetWindowLongPtrW", "StringCchCopy"}
    )
    assert got == {"StringCchCopy": "StringCchCopyW"}


def test_a_cpp_global_scope_qualifier_is_dropped():
    """windowsx.h's Edit_SetModify, expanded in a C++ TU: `::SendMessageW(..)`."""
    from rustconv.sdk_macros import lower_expansion

    out = lower_expansion("((void)::SendMessageW((h),185,(WPARAM)(UINT)(m),0))", {"WPARAM", "UINT"})
    assert "SendMessageW(" in out and "::SendMessageW" not in out


def test_a_parenthesized_callee_is_a_call_not_a_cast():
    """windowsx.h's SetWindowFont in a C++ TU: `(void)(::SendMessageW)((h),48,(WPARAM)(f),0)`."""
    from rustconv.sdk_macros import lower_expansion

    out = lower_expansion(
        "(void)(::SendMessageW)(((h)),48,(WPARAM)(HFONT)((f)),(LPARAM)(BOOL)((r)))",
        {"WPARAM", "LPARAM", "HFONT", "BOOL"},
    )
    assert "SendMessageW(" in out and "as SendMessageW" not in out, out


def test_a_vtable_call_goes_through_the_option_slot():
    out = low("(__rcv_p0__)->lpVtbl->Release(__rcv_p0__)")
    assert ".lpVtbl).Release).unwrap_unchecked()(" in out


def test_a_read_through_a_volatile_pointer_stays_volatile():
    """ndis.h's NdisReadRegisterUlong reads a device register through `volatile ULONG *`;
    a plain `*` is a load LLVM may merge or hoist out of a poll loop."""
    out = low("*(__rcv_p1__) = *((volatile ULONG * const) (__rcv_p0__))")
    assert out == "{ (*$p1) = ((core::ptr::read_volatile((($p0) as *mut ULONG))) as _); }"


def test_a_write_through_a_volatile_pointer_stays_volatile():
    out = low("*((volatile ULONG *)(__rcv_p0__)) = (__rcv_p1__)")
    assert out == "core::ptr::write_volatile((($p0) as *mut ULONG), (($p1) as _));"


def test_a_volatile_pointer_to_pointer_is_not_a_volatile_access():
    """`volatile T **`: the outer load reads an ordinary pointer."""
    assert "read_volatile" not in low("*((volatile ULONG **)(__rcv_p0__))")


def test_a_call_through_a_member_function_pointer_unwraps_the_option():
    """ndis.h's NdisMIndicateStatus calls `(*Block->StatusHandler)(..)`; bindgen types the
    field Option<fn>, which `*` cannot call."""
    out = low("(*((PNDIS_MINIPORT_BLOCK)(__rcv_p0__))->StatusHandler)(__rcv_p0__, __rcv_p1__)")
    assert out.startswith("((*(($p0) as PNDIS_MINIPORT_BLOCK)).StatusHandler).unwrap_unchecked()(")


def test_island_types_include_the_sys_crate_the_island_globs(tmp_path):
    """taskmgr: `LPWSTR` comes from `pub use ntdll_sys::nt::*;`, and without it
    `((LPWSTR)-1L)` parsed as a subtraction and LPSTR_TEXTCALLBACK was never lowered."""
    crate = tmp_path / "rust" / "x-rust" / "src"
    crate.mkdir(parents=True)
    (crate / "types_amd64.rs").write_text("pub use ntdll_sys::nt::*;\npub struct LOCAL { }\n")
    sys_src = tmp_path / "rust" / "ntdll-sys" / "src"
    sys_src.mkdir(parents=True)
    (sys_src / "bindgen_amd64.rs").write_text("pub type LPWSTR = *mut WCHAR;\n")
    types = sdk_macros.island_types(crate)
    assert {"LOCAL", "LPWSTR"} <= types
    node = sdk_macros._Parser(sdk_macros.tokenize("((LPWSTR)-1L)"), types).parse()
    while node.kind == "comma" and len(node.kids) == 1:
        node = node.kids[0]
    assert node.kind == "cast"


def test_a_callee_the_crate_does_not_declare_yet_is_read_from_a_sibling_island(tmp_path):
    """runonce's DialogBox was lowered before `port resolve` declared DialogBoxParamW, so
    the DLGPROC slot got `as _` (E0605 on a function item). Any island has the header's
    signature; the callback slot must be passed through."""
    mine = tmp_path / "runonce-rust" / "src"
    other = tmp_path / "aclui-rust" / "src"
    mine.mkdir(parents=True)
    other.mkdir(parents=True)
    (mine / "types_i386.rs").write_text(
        'pub type DLGPROC = ::core::option::Option<unsafe extern "system" fn(a: HWND) -> INT_PTR>;\n'
    )
    (other / "types_i386.rs").write_text(
        'extern "system" {\n    pub fn DialogBoxParamW(\n        arg1: HINSTANCE,\n'
        "        arg2: LPCWSTR,\n        arg3: HWND,\n        arg4: DLGPROC,\n"
        "        arg5: LPARAM,\n    ) -> INT_PTR;\n}\n"
    )
    sdk_macros.load_signatures(mine)
    out = lower_expansion(
        "DialogBoxParamW(__rcv_p0__, __rcv_p1__, __rcv_p2__, __rcv_p3__, 0L)",
        {"DLGPROC"},
    )
    assert ", $p3," in out and "($p3) as _" not in out


def test_offsetof_is_refused_not_read_as_a_call():
    """FIELD_OFFSET(REPARSE_DATA_BUFFER, GenericReparseBuffer): a type and a member path."""
    with pytest.raises(LowerError, match="offsetof"):
        low("((LONG)__builtin_offsetof(REPARSE_DATA_BUFFER, GenericReparseBuffer))")


def test_a_cxx_reference_type_macro_is_a_pointer_alias():
    from rustconv.sdk_macros import cxx_ref_alias

    assert cxx_ref_alias("REFIID", "const IID &") == "pub type REFIID = *const IID;"
    assert cxx_ref_alias("REFCLSID", "const CLSID&") == "pub type REFCLSID = *const CLSID;"
    assert cxx_ref_alias("X", "const IID * const") is None


def test_a_parameter_used_as_a_member_name_is_an_ident_fragment():
    """oleaut32's `V_UNION(A,B) ((A)->n1.n2.n3.B)`: B is a field name, never an expression."""
    body = low("((__rcv_p0__)->n1.n2.n3.__rcv_p1__)")
    assert body.endswith(".n1.n2.n3.$p1")
    assert "$p0:expr, $p1:ident" in render_macro("V_UNION", 2, body)
    with pytest.raises(LowerError, match="both a field name"):
        render_macro("X", 1, "(*$p0).$p0")


def test_a_subscript_on_a_pointer_is_pointer_arithmetic():
    """commctrl's DPA_FastGetPtr: `DPA_GetPtrPtr(hdpa)[i]`, a pointer read through `**`."""
    out = low("(*((void * **)(((BYTE*)(__rcv_p0__)) + sizeof(void *))))[__rcv_p1__]")
    assert out.startswith("(*((*(") and out.endswith(
        " as *mut *mut *mut c_void))).add(($p1) as usize))"
    )
