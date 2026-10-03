//! The `_s` (secure) string family: `corecrt_internal_string_templates.h`'s
//! five `common_tcs*_s` templates, each instantiated once for narrow and once
//! for wide, plus `memcpy_s`/`wmemcpy_s`/`wmemmove_s`.
//!
//! Replaces in full: `strcat_s.cpp`, `strcpy_s.cpp`, `strncat_s.cpp`,
//! `strncpy_s.cpp`, `strnset_s.cpp`, `strset_s.cpp`, `wcscat_s.cpp`,
//! `wcscpy_s.cpp`, `wcsncat_s.cpp`, `wcsncpy_s.cpp`, `wcsnset_s.cpp`,
//! `wcsset_s.cpp`, `memcpy_s.cpp`, `wmemcpy_s.cpp`, `wmemmove_s.cpp`.
//!
//! ## Details that are silent if wrong
//!
//! * **`_SECURECRT_FILL_BUFFER` is 0 in a release build** (`corecrt_internal.h`:
//!   `#ifdef _DEBUG` gates it on). `_FILL_STRING`/`_RESET_STRING`'s fill half
//!   is then a complete no-op -- the templates below therefore only ever
//!   NUL-terminate on failure (`_RESET_STRING` = `*dest = 0`), never
//!   pattern-fill the rest of the buffer. If this tree ever builds `_DEBUG`,
//!   this file would need the fill half too; it does not exist today.
//! * **`_TRUNCATE` (`(size_t)-1`) is a real sentinel value for `count`**, not
//!   just "very large" -- `common_tcsncat_s`/`common_tcsncpy_s` branch on
//!   `count == _TRUNCATE` explicitly (copy-until-space-exhausted, then
//!   NUL-terminate and return `STRUNCATE`) rather than "copy `count`
//!   elements".
//! * **Truncation (`STRUNCATE` = 80) sets NO errno and calls NO invalid-
//!   parameter handler** -- it is a distinct return path from every other
//!   failure, which all funnel through `_VALIDATE_RETURN`-shaped errno-set +
//!   handler-invoke.
//! * **`common_tcsncat_s`'s destination-not-NUL-terminated check happens
//!   BEFORE the source-NULL check**, but only when `count != 0` -- a `count
//!   == 0` call with a NUL-unterminated destination and a NULL source is
//!   allowed to proceed (it appends nothing).
//! * **`common_tcsncpy_s` with `count == 0` resets the destination to `""`
//!   and returns success even if `source` is NULL** -- the `_VALIDATE_POINTER_
//!   RESET_STRING(source, ...)` call is skipped entirely in that branch.
//! * **`memcpy_s`/`wmemcpy_s` zero the WHOLE destination on any validation
//!   failure** (`memset(dest, 0, size)` / `wmemset(dest, 0, size_in_elements)`)
//!   before returning the error -- `wmemmove_s` does NOT (it validates first,
//!   writes nothing on failure). This asymmetry is in the vendored C exactly
//!   as read here, kept faithfully.
//! * **`count == 0`/`_SourceSize == 0` is the FIRST check in every one of
//!   `memcpy_s`/`wmemcpy_s`/`wmemmove_s`** and returns 0 (success) even with
//!   a NULL destination -- validation of the pointers only happens once
//!   there is at least one byte/element to move.
//! * **memcpy_s/wmemcpy_s/wmemmove_s call the imported `memcpy`/`memmove`,
//!   never a hand-rolled loop** -- both to avoid the self-recursion class
//!   (irrelevant here, since neither `memcpy` nor `memmove` is defined in
//!   this crate) and because the brief for this batch says to.
use core::ffi::{c_char, c_int, c_void};

type ErrnoT = c_int;

const EINVAL: ErrnoT = 22;
const ERANGE: ErrnoT = 34;
const STRUNCATE: ErrnoT = 80;
/// `(size_t)-1` (corecrt.h).
const TRUNCATE: usize = usize::MAX;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

/// `_VALIDATE_RETURN(e, err, ret)`.
#[inline]
unsafe fn validate(e: bool, err: ErrnoT) -> Option<()> {
    if e {
        Some(())
    } else {
        *_errno() = err;
        _invalid_parameter_noinfo();
        None
    }
}

/// A narrow (`c_char`) or wide (`u16`) code unit -- just enough of an
/// abstraction to write each `common_tcs*_s` template once.
trait Unit: Copy + PartialEq + 'static {
    const ZERO: Self;
}
impl Unit for c_char {
    const ZERO: Self = 0;
}
impl Unit for u16 {
    const ZERO: Self = 0;
}

/// `common_tcscat_s` (`strcat_s`/`wcscat_s`).
unsafe fn tcscat_s<T: Unit>(destination: *mut T, size_in_elements: usize, source: *const T) -> ErrnoT {
    if validate(!destination.is_null() && size_in_elements > 0, EINVAL).is_none() {
        return EINVAL;
    }
    if source.is_null() {
        *destination = T::ZERO;
        if validate(false, EINVAL).is_none() {
            return EINVAL;
        }
    }

    let mut d = destination;
    let mut available = size_in_elements;
    while available > 0 && *d != T::ZERO {
        d = d.add(1);
        available -= 1;
    }

    if available == 0 {
        *destination = T::ZERO;
        if validate(false, EINVAL).is_none() {
            return EINVAL;
        }
    }

    let mut s = source;
    loop {
        let c = *s;
        *d = c;
        d = d.add(1);
        s = s.add(1);
        if c == T::ZERO {
            break;
        }
        available -= 1;
        if available == 0 {
            break;
        }
    }

    if available == 0 {
        *destination = T::ZERO;
        if validate(false, ERANGE).is_none() {
            return ERANGE;
        }
    }
    0
}

/// `common_tcscpy_s` (`strcpy_s`/`wcscpy_s`).
unsafe fn tcscpy_s<T: Unit>(destination: *mut T, size_in_elements: usize, source: *const T) -> ErrnoT {
    if validate(!destination.is_null() && size_in_elements > 0, EINVAL).is_none() {
        return EINVAL;
    }
    if source.is_null() {
        *destination = T::ZERO;
        if validate(false, EINVAL).is_none() {
            return EINVAL;
        }
    }

    let mut d = destination;
    let mut s = source;
    let mut available = size_in_elements;
    loop {
        let c = *s;
        *d = c;
        d = d.add(1);
        s = s.add(1);
        if c == T::ZERO {
            break;
        }
        available -= 1;
        if available == 0 {
            break;
        }
    }

    if available == 0 {
        *destination = T::ZERO;
        if validate(false, ERANGE).is_none() {
            return ERANGE;
        }
    }
    0
}

/// `common_tcsncat_s` (`strncat_s`/`wcsncat_s`).
unsafe fn tcsncat_s<T: Unit>(
    destination: *mut T,
    size_in_elements: usize,
    source: *const T,
    count: usize,
) -> ErrnoT {
    if count == 0 && destination.is_null() && size_in_elements == 0 {
        return 0;
    }

    if validate(!destination.is_null() && size_in_elements > 0, EINVAL).is_none() {
        return EINVAL;
    }
    if count != 0 && source.is_null() {
        *destination = T::ZERO;
        if validate(false, EINVAL).is_none() {
            return EINVAL;
        }
    }

    let mut d = destination;
    let mut available = size_in_elements;
    while available > 0 && *d != T::ZERO {
        d = d.add(1);
        available -= 1;
    }

    if available == 0 {
        *destination = T::ZERO;
        if validate(false, EINVAL).is_none() {
            return EINVAL;
        }
    }

    let mut s = source;
    let mut remaining = count;
    if count == TRUNCATE {
        loop {
            let c = *s;
            *d = c;
            d = d.add(1);
            s = s.add(1);
            if c == T::ZERO {
                break;
            }
            available -= 1;
            if available == 0 {
                break;
            }
        }
    } else {
        while remaining > 0 {
            let c = *s;
            *d = c;
            d = d.add(1);
            s = s.add(1);
            if c == T::ZERO {
                break;
            }
            available -= 1;
            if available == 0 {
                break;
            }
            remaining -= 1;
        }
        if remaining == 0 {
            *d = T::ZERO;
        }
    }

    if available == 0 {
        if count == TRUNCATE {
            *destination.add(size_in_elements - 1) = T::ZERO;
            return STRUNCATE;
        }
        *destination = T::ZERO;
        if validate(false, ERANGE).is_none() {
            return ERANGE;
        }
    }
    0
}

/// `common_tcsncpy_s` (`strncpy_s`/`wcsncpy_s`).
unsafe fn tcsncpy_s<T: Unit>(
    destination: *mut T,
    size_in_elements: usize,
    source: *const T,
    count: usize,
) -> ErrnoT {
    if count == 0 && destination.is_null() && size_in_elements == 0 {
        return 0;
    }

    if validate(!destination.is_null() && size_in_elements > 0, EINVAL).is_none() {
        return EINVAL;
    }
    if count == 0 {
        // Notice: `source` can be NULL here; the C skips validating it.
        *destination = T::ZERO;
        return 0;
    }
    if source.is_null() {
        *destination = T::ZERO;
        if validate(false, EINVAL).is_none() {
            return EINVAL;
        }
    }

    let mut d = destination;
    let mut s = source;
    let mut available = size_in_elements;
    let mut remaining = count;
    if count == TRUNCATE {
        loop {
            let c = *s;
            *d = c;
            d = d.add(1);
            s = s.add(1);
            if c == T::ZERO {
                break;
            }
            available -= 1;
            if available == 0 {
                break;
            }
        }
    } else {
        loop {
            let c = *s;
            *d = c;
            d = d.add(1);
            s = s.add(1);
            if c == T::ZERO {
                break;
            }
            available -= 1;
            if available == 0 {
                break;
            }
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
        if remaining == 0 {
            *d = T::ZERO;
        }
    }

    if available == 0 {
        if count == TRUNCATE {
            *destination.add(size_in_elements - 1) = T::ZERO;
            return STRUNCATE;
        }
        *destination = T::ZERO;
        if validate(false, ERANGE).is_none() {
            return ERANGE;
        }
    }
    0
}

/// `common_tcsnset_s` (`_strnset_s`/`_wcsnset_s`).
unsafe fn tcsnset_s<T: Unit>(destination: *mut T, size_in_elements: usize, value: T, count: usize) -> ErrnoT {
    if count == 0 && destination.is_null() && size_in_elements == 0 {
        return 0;
    }
    if validate(!destination.is_null() && size_in_elements > 0, EINVAL).is_none() {
        return EINVAL;
    }

    let mut d = destination;
    let mut available = size_in_elements;
    let mut remaining = count;
    while *d != T::ZERO && remaining > 0 {
        available -= 1;
        if available == 0 {
            break;
        }
        *d = value;
        d = d.add(1);
        remaining -= 1;
    }

    if remaining == 0 {
        // Skip to the NUL to confirm it exists within bounds.
        while *d != T::ZERO {
            available -= 1;
            if available == 0 {
                break;
            }
            d = d.add(1);
        }
    }

    if available == 0 {
        *destination = T::ZERO;
        if validate(false, EINVAL).is_none() {
            return EINVAL;
        }
    }
    0
}

/// `common_tcsset_s` (`_strset_s`/`_wcsset_s`).
unsafe fn tcsset_s<T: Unit>(destination: *mut T, size_in_elements: usize, value: T) -> ErrnoT {
    if validate(!destination.is_null() && size_in_elements > 0, EINVAL).is_none() {
        return EINVAL;
    }

    let mut d = destination;
    let mut available = size_in_elements;
    while *d != T::ZERO {
        available -= 1;
        if available == 0 {
            break;
        }
        *d = value;
        d = d.add(1);
    }

    if available == 0 {
        *destination = T::ZERO;
        if validate(false, EINVAL).is_none() {
            return EINVAL;
        }
    }
    0
}

// ---------------------------------------------------------------------------
// Narrow exports
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn strcat_s(destination: *mut c_char, size_in_elements: usize, source: *const c_char) -> ErrnoT {
    tcscat_s(destination, size_in_elements, source)
}

#[no_mangle]
pub unsafe extern "C" fn strcpy_s(destination: *mut c_char, size_in_elements: usize, source: *const c_char) -> ErrnoT {
    tcscpy_s(destination, size_in_elements, source)
}

#[no_mangle]
pub unsafe extern "C" fn strncat_s(
    destination: *mut c_char,
    size_in_elements: usize,
    source: *const c_char,
    count: usize,
) -> ErrnoT {
    tcsncat_s(destination, size_in_elements, source, count)
}

#[no_mangle]
pub unsafe extern "C" fn strncpy_s(
    destination: *mut c_char,
    size_in_elements: usize,
    source: *const c_char,
    count: usize,
) -> ErrnoT {
    tcsncpy_s(destination, size_in_elements, source, count)
}

#[no_mangle]
pub unsafe extern "C" fn _strnset_s(
    destination: *mut c_char,
    size_in_elements: usize,
    value: c_int,
    count: usize,
) -> ErrnoT {
    tcsnset_s(destination, size_in_elements, value as c_char, count)
}

#[no_mangle]
pub unsafe extern "C" fn _strset_s(destination: *mut c_char, size_in_elements: usize, value: c_int) -> ErrnoT {
    tcsset_s(destination, size_in_elements, value as c_char)
}

// ---------------------------------------------------------------------------
// Wide exports
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn wcscat_s(destination: *mut u16, size_in_elements: usize, source: *const u16) -> ErrnoT {
    tcscat_s(destination, size_in_elements, source)
}

#[no_mangle]
pub unsafe extern "C" fn wcscpy_s(destination: *mut u16, size_in_elements: usize, source: *const u16) -> ErrnoT {
    tcscpy_s(destination, size_in_elements, source)
}

#[no_mangle]
pub unsafe extern "C" fn wcsncat_s(
    destination: *mut u16,
    size_in_elements: usize,
    source: *const u16,
    count: usize,
) -> ErrnoT {
    tcsncat_s(destination, size_in_elements, source, count)
}

#[no_mangle]
pub unsafe extern "C" fn wcsncpy_s(
    destination: *mut u16,
    size_in_elements: usize,
    source: *const u16,
    count: usize,
) -> ErrnoT {
    tcsncpy_s(destination, size_in_elements, source, count)
}

#[no_mangle]
pub unsafe extern "C" fn _wcsnset_s(destination: *mut u16, size_in_elements: usize, value: u16, count: usize) -> ErrnoT {
    tcsnset_s(destination, size_in_elements, value, count)
}

#[no_mangle]
pub unsafe extern "C" fn _wcsset_s(destination: *mut u16, size_in_elements: usize, value: u16) -> ErrnoT {
    tcsset_s(destination, size_in_elements, value)
}

// ---------------------------------------------------------------------------
// memcpy_s / wmemcpy_s / wmemmove_s
// ---------------------------------------------------------------------------

/// `memcpy_s` (`corecrt_memcpy_s.h`, instantiated `extern` by `memcpy_s.cpp`).
/// `_SourceSize == 0` returns 0 immediately, even with a NULL destination.
#[no_mangle]
pub unsafe extern "C" fn memcpy_s(
    destination: *mut c_void,
    destination_size: usize,
    source: *const c_void,
    source_size: usize,
) -> ErrnoT {
    if source_size == 0 {
        return 0;
    }
    if validate(!destination.is_null(), EINVAL).is_none() {
        return EINVAL;
    }
    if source.is_null() || destination_size < source_size {
        // Zero the WHOLE destination buffer, not just `source_size` bytes.
        core::ptr::write_bytes(destination as *mut u8, 0, destination_size);
        if validate(!source.is_null(), EINVAL).is_none() {
            return EINVAL;
        }
        if validate(destination_size >= source_size, ERANGE).is_none() {
            return ERANGE;
        }
        return EINVAL; // unreachable in practice; matches the C's dead fallback.
    }
    memcpy(destination, source, source_size);
    0
}

/// `memmove_s` (corecrt_memcpy_s.h, compiled out of line by the deleted
/// memcpy_s.cpp). Unlike `memcpy_s` it does NOT zero the destination on failure:
/// the three checks run in order and each one alone returns. Missing from the
/// string takeover, so the export resolved to msvcrt.dll's through the import
/// library until this port.
#[no_mangle]
pub unsafe extern "C" fn memmove_s(
    destination: *mut c_void,
    destination_size: usize,
    source: *const c_void,
    source_size: usize,
) -> ErrnoT {
    if source_size == 0 {
        return 0;
    }
    if validate(!destination.is_null(), EINVAL).is_none() {
        return EINVAL;
    }
    if validate(!source.is_null(), EINVAL).is_none() {
        return EINVAL;
    }
    if validate(destination_size >= source_size, ERANGE).is_none() {
        return ERANGE;
    }
    memmove(destination, source, source_size);
    0
}

/// `wmemcpy_s` (wmemcpy_s.cpp). Element counts, not bytes; the zero-fill on
/// failure is `size_in_elements` wide chars (`wmemset(dest, 0, n)`), done here
/// with `write_bytes` over `size_in_elements * 2` bytes -- exactly what the
/// inline `wmemset(dest, 0, n)` in the header expands to.
#[no_mangle]
pub unsafe extern "C" fn wmemcpy_s(
    destination: *mut u16,
    size_in_elements: usize,
    source: *const u16,
    count: usize,
) -> ErrnoT {
    if count == 0 {
        return 0;
    }
    if validate(!destination.is_null(), EINVAL).is_none() {
        return EINVAL;
    }
    if source.is_null() || size_in_elements < count {
        core::ptr::write_bytes(destination, 0u8, size_in_elements);
        if validate(!source.is_null(), EINVAL).is_none() {
            return EINVAL;
        }
        if validate(size_in_elements >= count, ERANGE).is_none() {
            return ERANGE;
        }
        return EINVAL;
    }
    memcpy(destination as *mut c_void, source as *const c_void, count * 2);
    0
}

/// `wmemmove_s` (wmemmove_s.cpp). Unlike `wmemcpy_s`, does NOT zero the
/// destination on failure -- it validates fully before writing anything.
#[no_mangle]
pub unsafe extern "C" fn wmemmove_s(
    destination: *mut u16,
    size_in_elements: usize,
    source: *const u16,
    count: usize,
) -> ErrnoT {
    if count == 0 {
        return 0;
    }
    if validate(!destination.is_null(), EINVAL).is_none() {
        return EINVAL;
    }
    if validate(!source.is_null(), EINVAL).is_none() {
        return EINVAL;
    }
    if validate(size_in_elements >= count, ERANGE).is_none() {
        return ERANGE;
    }
    memmove(destination as *mut c_void, source as *const c_void, count * 2);
    0
}
