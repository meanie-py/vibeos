"""Fold c2rust's expansion of wine/debug.h's TRACE/WARN/FIXME/ERR back into the macro.

c2rust sees the preprocessed TU, so every `WARN("header too small\\n")` arrives as ~18 lines:
a test of the channel's flag bits, two locals, and a `ros_dbg_log(...)` call carrying the
builder's absolute source path and line. A c2rust body banked by hand (one too large for the
model's output budget -- winhlp32's HLPFILE_BrowseParagraph, 1,569 lines, 26 such blocks)
should read like the rest of the crate, which calls win32macros-rust's `WINE_WARN!(..)`:

    if (*__wine_dbch___default).flags as c_int & (1 as c_int) << __WINE_DBCL_WARN as c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
        ros_dbg_log(__dbcl, __dbch, b"/abs/path.c\\0" .., b"Func\\0" .., 1236 as c_int,
                    b"header too small\\n\\0" as *const u8 as *const c_char, args..);
    }
  ->
    WINE_WARN!("header too small\\n", args..);

Only the exact shape is folded; anything else is left alone. The default channel only: a
named channel's block is left as c2rust wrote it.
"""

from __future__ import annotations

import re

# Written with single spaces; each space then matches any run of whitespace, since rustfmt
# breaks c2rust's long lines wherever the width runs out (`let __dbcl_7: __wine_debug_class =`
# with its value on the next line).
_TEMPLATE = (
    r"if \(\*__wine_dbch___default\)\.flags as ::core::ffi::c_int "
    r"& \(1 as ::core::ffi::c_int\) << __WINE_DBCL_(?P<cls>TRACE|WARN|FIXME|ERR) "
    r"as ::core::ffi::c_int != 0 \{ "
    # c2rust numbers the locals of the second and later blocks (`__dbch_0`, `__dbcl_1`).
    r"let (?P<ch>__dbch(?:_\d+)?): \*mut __wine_debug_channel = __wine_dbch___default; "
    r"let (?P<cl>__dbcl(?:_\d+)?): __wine_debug_class = __WINE_DBCL_(?P=cls); "
    r"ros_dbg_log\( (?P=cl), (?P=ch), "
    r"b\"[^\"]*\\0\" as \*const u8 as \*const ::core::ffi::c_char, "
    r"b\"[^\"]*\\0\" as \*const u8 as \*const ::core::ffi::c_char, "
    r"\d+ as ::core::ffi::c_int, "
    r"b\"(?P<fmt>(?:\\.|[^\"\\])*)\\0\" as \*const u8 as \*const ::core::ffi::c_char,?"
    r"(?P<args>[^;]*?)\); \}"
)
_BLOCK = re.compile(_TEMPLATE.replace(" ", r"\s*"), re.S)


def fold(text: str) -> tuple[str, int]:
    """(new text, number of blocks folded)."""

    def one(m: re.Match) -> str:
        args = m.group("args").strip().rstrip(",").strip()
        args = re.sub(r"\s*\n\s*", " ", args)
        tail = f", {args}" if args else ""
        return f'WINE_{m.group("cls")}!("{m.group("fmt")}"{tail});'

    return _BLOCK.subn(one, text)
