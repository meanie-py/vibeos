//! `getpath.cpp`: pull the next element out of a `;`-separated path list.
//!
//! Leading `;`s are skipped, a `"..."` span is copied without its quotes
//! (so it may hold a `;`), at most `result_count - 1` characters are copied,
//! and the return is the start of the NEXT element (runs of `;` skipped) --
//! or NULL if nothing was consumed, or with `errno = ERANGE` if the element
//! did not fit (the buffer then holds the truncated prefix).

use core::ptr::null_mut;

use super::{set_errno, EnvChar, EINVAL, ERANGE};

unsafe fn common_getpath<C: EnvChar>(delimited_paths: *const C, result: *mut C, result_count: usize) -> *mut C {
    // _VALIDATE_RETURN_NOEXC: errno only.
    if result.is_null() {
        set_errno(EINVAL);
        return null_mut();
    }

    if result_count > 0 {
        *result = C::ch(0);
    }

    if result_count <= 1 {
        set_errno(EINVAL);
        return null_mut();
    }

    let mut source_it = delimited_paths;

    while (*source_it).is(b';') {
        source_it = source_it.add(1);
    }

    let source_first = source_it;

    let mut result_it = result;
    let result_last = result.add(result_count - 1); // room for the terminator

    while !(*source_it).is(0) && !(*source_it).is(b';') {
        if (*source_it).is(b'"') {
            source_it = source_it.add(1); // past the opening quote

            while !(*source_it).is(0) && !(*source_it).is(b'"') {
                *result_it = *source_it;
                result_it = result_it.add(1);
                source_it = source_it.add(1);
                if result_it == result_last {
                    *result_it = C::ch(0);
                    set_errno(ERANGE);
                    return null_mut();
                }
            }

            if !(*source_it).is(0) {
                source_it = source_it.add(1); // past the closing quote
            }
        } else {
            *result_it = *source_it;
            result_it = result_it.add(1);
            source_it = source_it.add(1);
            if result_it == result_last {
                *result_it = C::ch(0);
                set_errno(ERANGE);
                return null_mut();
            }
        }
    }

    while (*source_it).is(b';') {
        source_it = source_it.add(1);
    }

    *result_it = C::ch(0);
    if source_it == source_first {
        null_mut()
    } else {
        source_it as *mut C
    }
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_getpath(delimited_paths: *const u8, result: *mut u8, result_count: usize) -> *mut u8 {
    common_getpath(delimited_paths, result, result_count)
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_wgetpath(
    delimited_paths: *const u16,
    result: *mut u16,
    result_count: usize,
) -> *mut u16 {
    common_getpath(delimited_paths, result, result_count)
}
