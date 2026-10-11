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
