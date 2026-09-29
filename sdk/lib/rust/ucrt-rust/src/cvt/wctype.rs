//! `convert/wctype.cpp`: `wctype` -- a property name to its `_ctype` mask.
//!
//! A linear `strcmp` scan of twelve names; an unknown name is 0. A NULL name
//! is not validated (the C hands it straight to `strcmp`).

use core::ffi::c_int;

/// corecrt_wctype.h ctype bits.
const CT_UPPER: u16 = 0x01;
const CT_LOWER: u16 = 0x02;
const CT_DIGIT: u16 = 0x04;
const CT_SPACE: u16 = 0x08;
const CT_PUNCT: u16 = 0x10;
const CT_CONTROL: u16 = 0x20;
const CT_BLANK: u16 = 0x40;
const CT_HEX: u16 = 0x80;
const CT_ALPHA: u16 = 0x0100 | CT_UPPER | CT_LOWER;

extern "C" {
    // `mbstring/compare.rs` and `convert_rest/wctrans.rs` declare it `*const u8`.
    fn strcmp(a: *const u8, b: *const u8) -> c_int;
}

/// `tab[]`, in the C's order (the scan is first-match, and order is the only
/// thing that could make it observable).
static TAB: [(&[u8], u16); 12] = [
    (b"alnum\0", CT_ALPHA | CT_DIGIT),
    (b"alpha\0", CT_ALPHA),
    (b"cntrl\0", CT_CONTROL),
    (b"digit\0", CT_DIGIT),
    (b"graph\0", CT_PUNCT | CT_ALPHA | CT_DIGIT),
    (b"lower\0", CT_LOWER),
    (b"print\0", CT_BLANK | CT_PUNCT | CT_ALPHA | CT_DIGIT),
    (b"punct\0", CT_PUNCT),
    (b"blank\0", CT_BLANK),
    (b"space\0", CT_SPACE),
    (b"upper\0", CT_UPPER),
    (b"xdigit\0", CT_HEX),
];

/// `wctype` (wctype.h): `wctype_t` is `unsigned short`.
///
/// # Safety
/// `name` must be a NUL-terminated string (NULL faults, as in the C).
#[no_mangle]
pub unsafe extern "C" fn wctype(name: *const u8) -> u16 {
    for &(s, value) in TAB.iter() {
        if strcmp(s.as_ptr(), name) == 0 {
            return value;
        }
    }
    0
}
