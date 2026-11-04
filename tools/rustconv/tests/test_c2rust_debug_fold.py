"""c2rust_debug_fold: c2rust's expanded wine TRACE/WARN/FIXME/ERR folded back to the macro."""

from rustconv.c2rust_debug_fold import fold

# As c2rust and rustfmt wrote winhlp32's HLPFILE_BrowseParagraph (2026-10-11): the second
# block's locals are numbered and its casts wrap.
SRC = """
    if buf.offset(0x19 as ::core::ffi::c_int as isize) > end {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"/abs/hlpfile.c\\0" as *const u8 as *const ::core::ffi::c_char,
                b"HLPFILE_BrowseParagraph\\0" as *const u8 as *const ::core::ffi::c_char,
                1236 as ::core::ffi::c_int,
                b"header too small\\n\\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return FALSE;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class =
            __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"/abs/hlpfile.c\\0" as *const u8
                as *const ::core::ffi::c_char,
            b"HLPFILE_BrowseParagraph\\0" as *const u8 as *const ::core::ffi::c_char,
            1260 as ::core::ffi::c_int,
            b"cols=%d type=%x\\n\\0" as *const u8 as *const ::core::ffi::c_char,
            ncol,
            type_0 as ::core::ffi::c_int,
        );
    }
"""


def test_both_blocks_fold_to_the_macro():
    out, n = fold(SRC)
    assert n == 2
    assert 'WINE_WARN!("header too small\\n");' in out
    assert 'WINE_TRACE!("cols=%d type=%x\\n", ncol, type_0 as ::core::ffi::c_int);' in out
    assert "ros_dbg_log" not in out and "/abs/hlpfile.c" not in out


def test_a_named_channel_is_left_alone():
    named = SRC.replace("__wine_dbch___default", "__wine_dbch_richedit")
    assert fold(named) == (named, 0)


def test_fold_module_drops_the_scaffolding_nothing_names_any_more():
    """dsound's mixer.rs (2026-11-02): the blocks came with a local `__WINE_DBCL_TRACE = 4`
    (wine/debug.h says 3) and an extern `ros_dbg_log`. Folded, both are dead -- and the const
    was wrong -- so both go; the prelude's macro carries the real class."""
    from rustconv.c2rust_debug_fold import fold_module

    src = (
        "use crate::prelude::*;\n"
        'extern "C" {\n'
        "    fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, "
        "file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, "
        "line: ::core::ffi::c_int, fmt: *const ::core::ffi::c_char, ... );\n"
        "}\n"
        "\n"
        "const __WINE_DBCL_TRACE: __wine_debug_class = 4;\n"
        'pub unsafe extern "C" fn f(v: *mut c_void) {\n' + SRC + "}\n"
    )
    new, n, dropped = fold_module(src)
    assert n == 2
    assert "ros_dbg_log" not in new and "__WINE_DBCL" not in new
    assert 'extern "C" {' not in new
    assert sorted(dropped) == ["__WINE_DBCL_TRACE", "ros_dbg_log"]


def test_fold_module_keeps_scaffolding_a_named_channel_still_uses():
    from rustconv.c2rust_debug_fold import fold_module

    src = (
        'extern "C" {\n    fn ros_dbg_log(cls: __wine_debug_class, ...);\n}\n'
        "const __WINE_DBCL_TRACE: __wine_debug_class = 3;\n"
        "fn g() { if (*__wine_dbch_other).flags & 1 << __WINE_DBCL_TRACE != 0 "
        "{ ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch_other); } }\n"
    )
    new, n, dropped = fold_module(src)
    assert (new, n, dropped) == (src, 0, [])


def test_model_respelt_blocks_fold_too():
    """dsound mixer.rs: the model kept the block but added parens, addr_of!, `core::ffi`
    without `::`, and `(__dbcl) as u32` -- 26 of 77 blocks missed the literal template."""
    src = """
    if ((*(*core::ptr::addr_of!(__wine_dbch___default))).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = core::ptr::addr_of_mut!(__wine_dbch___default);
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/directx/wine/dsound/mixer.c\\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DSOUND_RecalcFreqAcc\\0" as *const u8 as *const ::core::ffi::c_char,
            150 as ::core::ffi::c_int,
            b"New freqadjust: %04x\\n\\0" as *const u8
                as *const core::ffi::c_char,
            (*dsb).freqAccNext,
        );
    }
"""
    out, n = fold(src)
    assert n == 1
    assert out.strip() == 'WINE_TRACE!("New freqadjust: %04x\\n", (*dsb).freqAccNext);'


def test_a_block_whose_condition_tests_another_class_is_left_alone():
    """The `if` must test the same class the block logs at: a TRACE log behind a WARN test
    is not a plain TRACE and must not fold."""
    src = """
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(__dbcl, __dbch, b"f.c\\0" as *const u8 as *const ::core::ffi::c_char,
            b"F\\0" as *const u8 as *const ::core::ffi::c_char, 1 as ::core::ffi::c_int,
            b"x\\n\\0" as *const u8 as *const ::core::ffi::c_char);
    }
"""
    assert fold(src) == (src, 0)


def test_rust_primitive_spellings_fold_too():
    # dsound's sound3d: `*const i8`, `120 as i32`, `1252i32` instead of c_char / c_int.
    text = (
        "    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {\n"
        "        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;\n"
        "        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;\n"
        "        ros_dbg_log(\n            (__dbcl) as u32,\n            __dbch,\n"
        '            b"dll/directx/wine/dsound/sound3d.c\\0"\n'
        "                as *const u8 as *const i8,\n"
        '            b"VectorBetweenTwoPoints\\0" as *const u8 as *const i8,\n'
        "            1252i32,\n"
        '            b"A (%f)\\n\\0" as *const u8 as *const i8,\n'
        "            (*a).x as f64,\n        );\n    }\n"
    )
    out, n = fold(text)
    assert n == 1
    assert out.strip() == 'WINE_TRACE!("A (%f)\\n", (*a).x as f64);'


def test_duplicate_scaffolding_consts_drop_together():
    # sound3d: the model's const and resolve's copy, each "naming" the other.
    from rustconv.c2rust_debug_fold import fold_module

    text = (
        "use crate::prelude::*;\n"
        "#[allow(non_upper_case_globals)]\n"
        "pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;\n\n"
        "// __WINE_DBCL_TRACE\n"
        "pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;\n"
        "fn f() {}\n"
    )
    out, n, dropped = fold_module(text)
    assert dropped == ["__WINE_DBCL_TRACE"]
    assert out == "use crate::prelude::*;\n\nfn f() {}\n"


def test_as_ptr_spelt_strings_fold():
    text = (
        "if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {\n"
        "    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;\n"
        "    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;\n"
        "    ros_dbg_log((__dbcl) as u32, __dbch,\n"
        '        b"dll/directx/wine/dsound/duplex.c\\0"\n            .as_ptr() as *const i8,\n'
        '        b"F\\0".as_ptr() as *const i8, 209i32,\n'
        '        b"(%p)\\n\\0".as_ptr() as *const i8, This,\n    );\n}\n'
    )
    out, n = fold(text)
    assert n == 1 and out.strip() == 'WINE_TRACE!("(%p)\\n", This);'
