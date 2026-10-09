"""sdk_macros: SDK header macros lowered from the preprocessor's own expansion."""

import pytest

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
