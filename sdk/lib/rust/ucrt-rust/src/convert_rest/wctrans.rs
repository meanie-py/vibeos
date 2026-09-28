//! `convert/wctrans.cpp`: `towctrans` and `wctrans`.
//!
//! `wctrans_t`/`wctype_t` are `unsigned short` in UCRT; the two named
//! transforms are `"tolower" -> 2`, `"toupper" -> 1` (note: NOT the bit
//! values `_LOWER`/`_UPPER` from the ctype table -- these are wctrans's own
//! private enumeration, unrelated). An unrecognised name (including a null
//! `value` of 0 reaching `towctrans`) falls through to `towlower`, exactly as
//! the C's `value == 1 ? towupper(c) : towlower(c)` does for any value other
//! than 1.

use core::ffi::{c_char, c_int};

extern "C" {
    // Matches tocase.rs's `towupper`/`towlower` prototypes exactly (`wint_t`
    // is `u16`), so this compiles whether or not `tocase` is also enabled --
    // if it isn't, these resolve to the vendored C's exports instead.
    fn towupper(c: u16) -> u16;
    fn towlower(c: u16) -> u16;
    // `mbstring/compare.rs` declares this as `*const u8`, not `*const
    // c_char` -- matched exactly.
    fn strcmp(a: *const u8, b: *const u8) -> c_int;
}

struct WcTab {
    s: *const u8,
    value: u16,
}

// Kept as `\0`-terminated byte strings rather than Rust `&str` so `strcmp`
// (the C's own comparator, byte-for-byte) is what decides matches.
static TOLOWER_NAME: &[u8] = b"tolower\0";
static TOUPPER_NAME: &[u8] = b"toupper\0";

fn tab() -> [WcTab; 2] {
    [
        WcTab { s: TOLOWER_NAME.as_ptr().cast(), value: 2 },
        WcTab { s: TOUPPER_NAME.as_ptr().cast(), value: 1 },
    ]
}

/// `towctrans`.
///
/// # Safety
/// No pointer arguments; always safe to call.
#[no_mangle]
pub unsafe extern "C" fn towctrans(c: u16, value: u16) -> u16 {
    if value == 1 {
        towupper(c)
    } else {
        towlower(c)
    }
}

/// `wctrans`.
///
/// # Safety
/// `name`, if non-null, must be a valid NUL-terminated C string. A null
/// `name` is UB in the C too (`strcmp` would dereference it) -- not guarded
/// here either, faithfully.
#[no_mangle]
pub unsafe extern "C" fn wctrans(name: *const c_char) -> u16 {
    for entry in tab() {
        if strcmp(entry.s, name.cast()) == 0 {
            return entry.value;
        }
    }
    0
}
