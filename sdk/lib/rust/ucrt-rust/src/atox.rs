//! `atoi` and its 15 siblings. Replaces `sdk/lib/ucrt/convert/atox.cpp` in full.
//!
//! Every export there is `parse_integer_from_string<T>(string, nullptr, 10, locale)`
//! -- the same template as `strtox.cpp`, base 10, no end pointer -- so each is a
//! call into the port in `strtox.rs`. That means the `atoi` family inherits every
//! UCRT detail of `strtol` for free, including the ones that surprise: `atoi`
//! SATURATES at INT_MIN/INT_MAX and sets ERANGE (it is `long`-typed underneath,
//! 32 bits on Windows), and a NULL string raises the invalid-parameter handler
//! and returns 0 with errno EINVAL rather than faulting.

use crate::locale::CrtLocalePointers;
use crate::strtox::parse;
use core::ffi::c_int;

macro_rules! atox {
    ($ch:ty, $ret:ty, $bits:expr, $plain:ident, $l:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $l(string: *const $ch, locale: *mut CrtLocalePointers) -> $ret {
            parse(string, core::ptr::null_mut(), 10, locale, $bits, true) as $ret
        }
        #[no_mangle]
        pub unsafe extern "C" fn $plain(string: *const $ch) -> $ret {
            parse(string, core::ptr::null_mut(), 10, core::ptr::null_mut(), $bits, true) as $ret
        }
    };
}

// `int` and `long` both parse as `long`: 32 bits on every Windows target.
atox!(u8, c_int, 32, atoi, _atoi_l);
atox!(u8, i32, 32, atol, _atol_l);
atox!(u8, i64, 64, atoll, _atoll_l);
atox!(u8, i64, 64, _atoi64, _atoi64_l);
atox!(u16, c_int, 32, _wtoi, _wtoi_l);
atox!(u16, i32, 32, _wtol, _wtol_l);
atox!(u16, i64, 64, _wtoll, _wtoll_l);
atox!(u16, i64, 64, _wtoi64, _wtoi64_l);
