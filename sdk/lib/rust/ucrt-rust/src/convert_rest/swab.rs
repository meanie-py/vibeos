//! `convert/swab.cpp`: `_swab`.

use super::common::EINVAL;
use core::ffi::{c_char, c_int};

extern "C" {
    fn _invalid_parameter_noinfo();
    fn _errno() -> *mut c_int;
}

/// `_VALIDATE_RETURN_VOID(e, err)`: set errno, invoke the no-info invalid
/// parameter handler, and return (void, so no return value to also set).
unsafe fn validate_void(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}

/// `_swab`. Faithful quirk: odd `bytes` truncates down (the `while bytes > 1`
/// loop stops with one unswapped byte left over, silently dropped).
///
/// # Safety
/// `source`/`destination` must be valid, non-overlapping, for `bytes` (or
/// `bytes - 1` if odd) bytes.
#[no_mangle]
pub unsafe extern "C" fn _swab(mut source: *mut c_char, mut destination: *mut c_char, bytes: c_int) {
    if !validate_void(!source.is_null(), EINVAL) {
        return;
    }
    if !validate_void(!destination.is_null(), EINVAL) {
        return;
    }
    if !validate_void(bytes >= 0, EINVAL) {
        return;
    }

    let mut bytes = bytes;
    while bytes > 1 {
        let b1 = *source;
        source = source.add(1);
        let b2 = *source;
        source = source.add(1);

        *destination = b2;
        destination = destination.add(1);
        *destination = b1;
        destination = destination.add(1);

        bytes -= 2;
    }
}
