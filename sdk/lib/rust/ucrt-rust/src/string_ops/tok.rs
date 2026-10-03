//! `strtok`/`strtok_s`/`wcstok`/`wcstok_s` -- replaces `strtok.cpp`,
//! `strtok_s.cpp`, `wcstok.cpp`, `wcstok_s.cpp` in full.
//!
//! ## Details that are silent if wrong
//!
//! * **`strtok`/`wcstok` (no context) use the PER-THREAD token slot**
//!   (`AcrtPtd::strtok_token`/`wcstok_token`, pinned in `ptd.rs`), reached via
//!   `__acrt_getptd()`. `wcstok` additionally accepts an explicit `context`
//!   pointer (a 3-arg form, matching the vendored header, which is NOT the
//!   2-arg POSIX `wcstok`); when `context` is non-NULL it is used directly
//!   and the PTD slot is never touched.
//! * **`__acrt_strtok_s_novalidation`'s control-character map is a 256-bit
//!   bitmap** (`unsigned char map[32]`), built fresh on every call by walking
//!   `control` to termination -- NOT cached. The NUL byte is always in the
//!   map because the control-string walk's `do { ... } while (*p++)`
//!   processes the terminating NUL as if it were a control character before
//!   stopping.
//! * **`strtok_s`/`wcstok_s` validate `context` and `control` unconditionally**
//!   (`EINVAL`, return NULL), and additionally require `string != NULL ||
//!   *context != NULL` -- i.e. a continuation call (`string == NULL`) with an
//!   uninitialized/NULL `*context` is rejected, unlike the bare `strtok`/
//!   `wcstok`, which have no such check and would dereference `*context`
//!   regardless (the C's `__acrt_strtok_s_novalidation` reads `*context`
//!   whenever `string == NULL`, with no validation at all -- carried
//!   faithfully: calling `strtok(NULL, ctrl)` before any prior `strtok` call
//!   on this thread reads whatever garbage sits in `_strtok_token`, exactly
//!   as the C does).
//! * **A "no token found" return is NULL**, which happens when the iterator
//!   never advances past the leading delimiters (it lands directly on `\0`).
use crate::ptd::AcrtPtd;
use core::ffi::{c_char, c_int};

const EINVAL: c_int = 22;

extern "C" {
    fn _errno() -> *mut c_int;
    fn __acrt_getptd() -> *mut AcrtPtd;
}

/// `__acrt_strtok_s_novalidation` (shared by `strtok`/`strtok_s`).
#[no_mangle]
pub unsafe extern "C" fn __acrt_strtok_s_novalidation(
    string: *mut c_char,
    control: *const c_char,
    context: *mut *mut c_char,
) -> *mut c_char {
    let mut map = [0u8; 32];
    let mut p = control as *const u8;
    loop {
        let b = *p;
        map[(b >> 3) as usize] |= 1 << (b & 7);
        if b == 0 {
            break;
        }
        p = p.add(1);
    }

    let mut it: *mut u8 = if !string.is_null() {
        string as *mut u8
    } else {
        *context as *mut u8
    };

    while (map[(*it >> 3) as usize] & (1 << (*it & 7))) != 0 && *it != 0 {
        it = it.add(1);
    }

    let token_first = it;

    while *it != 0 {
        if map[(*it >> 3) as usize] & (1 << (*it & 7)) != 0 {
            *it = 0;
            it = it.add(1);
            break;
        }
        it = it.add(1);
    }

    *context = it as *mut c_char;

    if it != token_first {
        token_first as *mut c_char
    } else {
        core::ptr::null_mut()
    }
}

/// `strtok` (strtok.cpp): the per-thread-context spelling.
#[no_mangle]
pub unsafe extern "C" fn strtok(string: *mut c_char, control: *const c_char) -> *mut c_char {
    let ptd = __acrt_getptd();
    __acrt_strtok_s_novalidation(string, control, &mut (*ptd).strtok_token as *mut *mut u8 as *mut *mut c_char)
}

/// `strtok_s` (strtok_s.cpp): validated, caller-owned context.
#[no_mangle]
pub unsafe extern "C" fn strtok_s(
    string: *mut c_char,
    control: *const c_char,
    context: *mut *mut c_char,
) -> *mut c_char {
    if context.is_null() {
        *_errno() = EINVAL;
        invalid_parameter_noinfo();
        return core::ptr::null_mut();
    }
    if control.is_null() {
        *_errno() = EINVAL;
        invalid_parameter_noinfo();
        return core::ptr::null_mut();
    }
    if string.is_null() && (*context).is_null() {
        *_errno() = EINVAL;
        invalid_parameter_noinfo();
        return core::ptr::null_mut();
    }
    __acrt_strtok_s_novalidation(string, control, context)
}

/// `__acrt_wcstok_s_novalidation` (shared by `wcstok`/`wcstok_s`). Unlike the
/// narrow twin, this uses a linear scan of `control` per character rather
/// than a bitmap -- the C's own choice, kept.
#[no_mangle]
pub unsafe extern "C" fn __acrt_wcstok_s_novalidation(
    string: *mut u16,
    control: *const u16,
    context: *mut *mut u16,
) -> *mut u16 {
    let mut it: *mut u16 = if !string.is_null() { string } else { *context };

    while *it != 0 {
        let mut c = control;
        while *c != 0 && *c != *it {
            c = c.add(1);
        }
        if *c == 0 {
            break;
        }
        it = it.add(1);
    }

    let token_first = it;

    while *it != 0 {
        let mut c = control;
        while *c != 0 && *c != *it {
            c = c.add(1);
        }
        if *c != 0 {
            *it = 0;
            it = it.add(1);
            break;
        }
        it = it.add(1);
    }

    *context = it;

    if it != token_first {
        token_first
    } else {
        core::ptr::null_mut()
    }
}

/// `wcstok` (wcstok.cpp): the 3-arg vendored-header form -- an explicit
/// non-NULL `context` bypasses the per-thread slot entirely.
#[no_mangle]
pub unsafe extern "C" fn wcstok(string: *mut u16, control: *const u16, context: *mut *mut u16) -> *mut u16 {
    let context_to_use = if !context.is_null() {
        context
    } else {
        let ptd = __acrt_getptd();
        &mut (*ptd).wcstok_token
    };
    __acrt_wcstok_s_novalidation(string, control, context_to_use)
}

/// `wcstok_s` (wcstok_s.cpp): validated, caller-owned context.
#[no_mangle]
pub unsafe extern "C" fn wcstok_s(string: *mut u16, control: *const u16, context: *mut *mut u16) -> *mut u16 {
    if context.is_null() {
        *_errno() = EINVAL;
        invalid_parameter_noinfo();
        return core::ptr::null_mut();
    }
    if control.is_null() {
        *_errno() = EINVAL;
        invalid_parameter_noinfo();
        return core::ptr::null_mut();
    }
    if string.is_null() && (*context).is_null() {
        *_errno() = EINVAL;
        invalid_parameter_noinfo();
        return core::ptr::null_mut();
    }
    __acrt_wcstok_s_novalidation(string, control, context)
}

extern "C" {
    fn _invalid_parameter_noinfo();
}
#[inline]
unsafe fn invalid_parameter_noinfo() {
    _invalid_parameter_noinfo();
}
