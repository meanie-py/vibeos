"""port integrate: WINE_DEFAULT_DEBUG_CHANNEL comes from the prelude macro, not the preamble."""

from rustconv.port import _strip_channel_items

PREAMBLES = """use crate::prelude::*;

extern "C" {
    fn assert(expr: INT);
    static mut __wine_dbch___default: *mut __wine_debug_channel;
}
// WINE_DEFAULT_DEBUG_CHANNEL(commctrl)
pub static __wine_dbch___default: [u8; 9] = *b"commctrl\\0";

#[no_mangle]
pub static mut __wine_dbch_taskdialog: __wine_debug_channel = __wine_debug_channel {
    flags: 0,
    name: [b't' as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};
pub const KEEP: u32 = 1;
"""


def test_every_shape_a_preamble_gave_the_channel_is_removed():
    out = _strip_channel_items(PREAMBLES)
    assert "__wine_dbch" not in out
    assert "WINE_DEFAULT_DEBUG_CHANNEL" not in out and "#[no_mangle]" not in out
    assert "fn assert(expr: INT);" in out and "pub const KEEP: u32 = 1;" in out
    assert 'extern "C" {' in out


def test_uses_of_the_channel_are_left_alone():
    body = "fn f() {\n    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;\n}\n"
    assert _strip_channel_items(body) == body
