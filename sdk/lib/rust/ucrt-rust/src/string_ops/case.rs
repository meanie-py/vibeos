//! String-wide case mapping: `strlwr`/`strupr`/`wcslwr`/`wcsupr`, each with
//! its `_l`, `_s` and `_s_l` forms -- replaces `strlwr.cpp`, `strupr.cpp`,
//! `wcslwr.cpp`, `wcsupr.cpp` in full (16 exports).
//!
//! NOT the same problem as `tocase.rs`: that file maps ONE character
//! (`tolower`/`towlower`); this file maps a whole NUL-terminated string in
//! place, via `LCMapStringA`/`W` when the locale isn't "C". `strupr`/`wcsupr`
//! are `strlwr`/`wcslwr` with `LCMAP_UPPERCASE` and the ASCII range flipped --
//! confirmed with `diff` against the vendored `.cpp` pair, byte-for-byte
//! identical otherwise -- so both directions share one generic body per width.
//!
//! ## Details that are silent if wrong
//!
//! * **The "C locale" test here is `locale_name[LC_CTYPE] == NULL`**, checked
//!   INSIDE the `_s_l` worker against the ALREADY-RESOLVED locale -- not
//!   `__acrt_locale_changed_data`. The plain (non-`_l`) entry points use
//!   `__acrt_locale_changed_data == 0` as their OWN, separate fast-path gate
//!   before ever reaching the worker (so a thread that changed the locale
//!   once always pays the `_s_l` call, even if it changed it back to "C" by
//!   name -- same "never un-latches" caveat as `tocase.rs`'s doc).
//! * **`_strlwr`/`_strupr`'s C-locale fast path validates `string != NULL`
//!   itself** (`_VALIDATE_RETURN` before touching it); the locale-changed
//!   path does NOT re-validate -- it hands `string` straight to
//!   `_strlwr_s_l`/`_strupr_s_l`, whose OWN validation catches a NULL. Both
//!   ultimately behave the same for NULL (return NULL / leave `string`
//!   unchanged), but the C's shape is kept exactly (validate-then-loop vs.
//!   delegate-then-validate).
//! * **`_strlwr_l`/`_wcslwr_l` (and the upper twins) DISCARD the `errno_t`**
//!   from their `_s_l` call and unconditionally return the original pointer,
//!   even when the `_s_l` call failed and left the string untouched (or
//!   reset to `""` -- see next point). This looks like it swallows the error,
//!   and it does: that is the documented, load-bearing contract ("the
//!   original string is returned unchanged on any error, and errno is set" --
//!   the ERRNO carries the failure, the return value never does for the
//!   non-`_s` forms).
//! * **On `_RETURN_DEST_NOT_NULL_TERMINATED`/`_RETURN_BUFFER_TOO_SMALL`, the
//!   worker RESETS the string to `""`** (`*string = 0`) before returning the
//!   error code -- so `_strlwr_l`/`_wcslwr_l` (which ignore that code) return
//!   a pointer to a now-EMPTY string, not the original content. This is the
//!   one way the "unchanged on error" doc comment is wrong about its own
//!   code, and it is kept faithfully (verified against the `.cpp`: `_RESET_
//!   STRING` unconditionally executes on both failure paths before the
//!   return).
//! * **`stringlen = strnlen(string, sizeInBytes)` (not `strlen`)**: a
//!   non-NUL-terminated buffer of exactly `sizeInBytes` bytes is caught as
//!   `stringlen >= sizeInBytes`, the same "not NUL terminated" error path,
//!   rather than reading past the buffer.
//! * **The two-call LCMapString shape**: first call with a NULL/zero dest to
//!   size the transformed string (which can be LONGER than the source, e.g.
//!   German ß -> SS under some locales); only if `sizeInBytes`/`sizeInWords`
//!   is large enough does the second, real call happen. A too-small
//!   destination is `_RETURN_BUFFER_TOO_SMALL` (ERANGE) -- the string is
//!   reset to `""`, NOT left holding a truncated transform.
//! * **The final copy uses `strcpy_s`/`wcscpy_s` (this crate's own,
//!   `safe.rs`)**, propagating whatever it returns -- which cannot fail here
//!   since the destination was already proven big enough, but is propagated
//!   verbatim rather than assumed to be 0, exactly as the C does.
//! * **Scratch buffer**: the C uses `_malloca_crt_t` (stack for a small
//!   request, heap above a threshold) as a perf detail with NO effect on the
//!   RESULT -- a caller cannot observe whether the temporary buffer for the
//!   transformed string lived on the stack or the heap. This port always
//!   heap-allocates (`malloc`/`free`) and maps `malloc` failure to `ENOMEM`,
//!   matching the C's own `ENOMEM` branch for `_malloca_crt_t` failure.
use crate::locale::CRT_LC_CTYPE;
use crate::locale_abi::{LocaleT, __acrt_LCMapStringA, __acrt_LCMapStringW};
use crate::locale_update::{LocaleUpdate, __acrt_locale_changed_data};
use core::ffi::{c_char, c_int, c_void};

type ErrnoT = c_int;

const EINVAL: ErrnoT = 22;
const ENOMEM: ErrnoT = 12;
const EILSEQ: ErrnoT = 42;
const ERANGE: ErrnoT = 34;
const LCMAP_LOWERCASE: u32 = 0x0100;
const LCMAP_UPPERCASE: u32 = 0x0200;
/// `(size_t)(-1)`, what `_strlwr`/`_strlwr_l` pass as `sizeInBytes` to the
/// `_s_l` worker -- "assume the buffer is big enough" (the plain forms take
/// no size parameter at all).
const UNBOUNDED: usize = usize::MAX;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn malloc(size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn strnlen(s: *const c_char, n: usize) -> usize;
    fn wcsnlen(s: *const u16, n: usize) -> usize;
}

#[inline]
unsafe fn validate_errcode(e: bool, err: ErrnoT) -> Result<(), ErrnoT> {
    if e {
        Ok(())
    } else {
        *_errno() = err;
        _invalid_parameter_noinfo();
        Err(err)
    }
}

/// `_strlwr_s_l_stat`/`_strupr_s_l_stat`, generic over direction
/// (`to_upper`) -- narrow. `plocinfo` is already resolved (the caller's
/// `LocaleUpdate::locale_t()`).
unsafe fn narrow_case_s_l(
    string: *mut c_char,
    size_in_bytes: usize,
    plocinfo: LocaleT,
    to_upper: bool,
) -> ErrnoT {
    if let Err(e) = validate_errcode(!string.is_null(), EINVAL) {
        return e;
    }
    let stringlen = strnlen(string, size_in_bytes);
    if stringlen >= size_in_bytes {
        *string = 0;
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return EINVAL;
    }

    let locinfo = (*plocinfo).locinfo;
    if (*locinfo).locale_name[CRT_LC_CTYPE as usize].is_null() {
        // "C" locale: plain ASCII range flip, in place.
        let mut cp = string;
        while *cp != 0 {
            let c = *cp as u8;
            if to_upper {
                if c.is_ascii_lowercase() {
                    *cp = (c - b'a' + b'A') as c_char;
                }
            } else if c.is_ascii_uppercase() {
                *cp = (c - b'A' + b'a') as c_char;
            }
            cp = cp.add(1);
        }
        return 0;
    }

    let map_flag = if to_upper { LCMAP_UPPERCASE } else { LCMAP_LOWERCASE };
    let locale_name = (*locinfo).locale_name[CRT_LC_CTYPE as usize];
    let code_page = (*(*locinfo).public_ref()).locale_lc_codepage as c_int;

    let dstsize = __acrt_LCMapStringA(
        plocinfo,
        locale_name,
        map_flag,
        string,
        -1,
        core::ptr::null_mut(),
        0,
        code_page,
        1,
    );
    if dstsize == 0 {
        *_errno() = EILSEQ;
        return EILSEQ;
    }

    if size_in_bytes < dstsize as usize {
        *string = 0;
        *_errno() = ERANGE;
        _invalid_parameter_noinfo();
        return ERANGE;
    }

    let dst = malloc(dstsize as usize) as *mut c_char;
    if dst.is_null() {
        *_errno() = ENOMEM;
        return ENOMEM;
    }
    let mapped = __acrt_LCMapStringA(plocinfo, locale_name, map_flag, string, -1, dst, dstsize, code_page, 1);
    let result = if mapped != 0 {
        crate::string_ops::safe::strcpy_s(string, size_in_bytes, dst)
    } else {
        *_errno() = EILSEQ;
        EILSEQ
    };
    free(dst as *mut c_void);
    result
}

/// `_wcslwr_s_l_stat`/`_wcsupr_s_l_stat` -- wide twin. Note: unlike the
/// narrow form, `__acrt_LCMapStringW` takes no `_locale_t`/codepage (the
/// wide Win32 call is codepage-independent), matching the header's own
/// narrower signature (see `locale_abi.rs`).
unsafe fn wide_case_s_l(string: *mut u16, size_in_words: usize, plocinfo: LocaleT, to_upper: bool) -> ErrnoT {
    if let Err(e) = validate_errcode(!string.is_null(), EINVAL) {
        return e;
    }
    let stringlen = wcsnlen(string, size_in_words);
    if stringlen >= size_in_words {
        *string = 0;
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return EINVAL;
    }

    let locinfo = (*plocinfo).locinfo;
    if (*locinfo).locale_name[CRT_LC_CTYPE as usize].is_null() {
        let mut p = string;
        while *p != 0 {
            let c = *p;
            if to_upper {
                if (b'a' as u16..=b'z' as u16).contains(&c) {
                    *p = c - b'a' as u16 + b'A' as u16;
                }
            } else if (b'A' as u16..=b'Z' as u16).contains(&c) {
                *p = c - b'A' as u16 + b'a' as u16;
            }
            p = p.add(1);
        }
        return 0;
    }

    let map_flag = if to_upper { LCMAP_UPPERCASE } else { LCMAP_LOWERCASE };
    let locale_name = (*locinfo).locale_name[CRT_LC_CTYPE as usize];

    let dstsize = __acrt_LCMapStringW(locale_name, map_flag, string, -1, core::ptr::null_mut(), 0);
    if dstsize == 0 {
        *_errno() = EILSEQ;
        return EILSEQ;
    }

    if size_in_words < dstsize as usize {
        *string = 0;
        *_errno() = ERANGE;
        _invalid_parameter_noinfo();
        return ERANGE;
    }

    let dst = malloc(dstsize as usize * 2) as *mut u16;
    if dst.is_null() {
        *_errno() = ENOMEM;
        return ENOMEM;
    }
    let mapped = __acrt_LCMapStringW(locale_name, map_flag, string, -1, dst, dstsize);
    let result = if mapped != 0 {
        crate::string_ops::safe::wcscpy_s(string, size_in_words, dst)
    } else {
        *_errno() = EILSEQ;
        EILSEQ
    };
    free(dst as *mut c_void);
    result
}

// ---------------------------------------------------------------------------
// strlwr family
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _strlwr_s_l(string: *mut c_char, size_in_bytes: usize, plocinfo: LocaleT) -> ErrnoT {
    let mut lu = LocaleUpdate::new(plocinfo);
    narrow_case_s_l(string, size_in_bytes, lu.locale_t(), false)
}

#[no_mangle]
pub unsafe extern "C" fn _strlwr_s(string: *mut c_char, size_in_bytes: usize) -> ErrnoT {
    _strlwr_s_l(string, size_in_bytes, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _strlwr_l(string: *mut c_char, plocinfo: LocaleT) -> *mut c_char {
    _strlwr_s_l(string, UNBOUNDED, plocinfo);
    string
}

#[no_mangle]
pub unsafe extern "C" fn _strlwr(string: *mut c_char) -> *mut c_char {
    if __acrt_locale_changed_data == 0 {
        if validate_errcode(!string.is_null(), EINVAL).is_err() {
            return core::ptr::null_mut();
        }
        let mut cp = string;
        while *cp != 0 {
            let c = *cp as u8;
            if c.is_ascii_uppercase() {
                *cp = (c - b'A' + b'a') as c_char;
            }
            cp = cp.add(1);
        }
        string
    } else {
        _strlwr_s_l(string, UNBOUNDED, core::ptr::null_mut());
        string
    }
}

// ---------------------------------------------------------------------------
// strupr family
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _strupr_s_l(string: *mut c_char, size_in_bytes: usize, plocinfo: LocaleT) -> ErrnoT {
    let mut lu = LocaleUpdate::new(plocinfo);
    narrow_case_s_l(string, size_in_bytes, lu.locale_t(), true)
}

#[no_mangle]
pub unsafe extern "C" fn _strupr_s(string: *mut c_char, size_in_bytes: usize) -> ErrnoT {
    _strupr_s_l(string, size_in_bytes, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _strupr_l(string: *mut c_char, plocinfo: LocaleT) -> *mut c_char {
    _strupr_s_l(string, UNBOUNDED, plocinfo);
    string
}

#[no_mangle]
pub unsafe extern "C" fn _strupr(string: *mut c_char) -> *mut c_char {
    if __acrt_locale_changed_data == 0 {
        if validate_errcode(!string.is_null(), EINVAL).is_err() {
            return core::ptr::null_mut();
        }
        let mut cp = string;
        while *cp != 0 {
            let c = *cp as u8;
            if c.is_ascii_lowercase() {
                *cp = (c - b'a' + b'A') as c_char;
            }
            cp = cp.add(1);
        }
        string
    } else {
        _strupr_s_l(string, UNBOUNDED, core::ptr::null_mut());
        string
    }
}

// ---------------------------------------------------------------------------
// wcslwr family
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _wcslwr_s_l(wsrc: *mut u16, size_in_words: usize, plocinfo: LocaleT) -> ErrnoT {
    let mut lu = LocaleUpdate::new(plocinfo);
    wide_case_s_l(wsrc, size_in_words, lu.locale_t(), false)
}

#[no_mangle]
pub unsafe extern "C" fn _wcslwr_s(wsrc: *mut u16, size_in_words: usize) -> ErrnoT {
    _wcslwr_s_l(wsrc, size_in_words, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _wcslwr_l(wsrc: *mut u16, plocinfo: LocaleT) -> *mut u16 {
    _wcslwr_s_l(wsrc, UNBOUNDED, plocinfo);
    wsrc
}

#[no_mangle]
pub unsafe extern "C" fn _wcslwr(wsrc: *mut u16) -> *mut u16 {
    if __acrt_locale_changed_data == 0 {
        if validate_errcode(!wsrc.is_null(), EINVAL).is_err() {
            return core::ptr::null_mut();
        }
        let mut p = wsrc;
        while *p != 0 {
            let c = *p;
            if (b'A' as u16..=b'Z' as u16).contains(&c) {
                *p = c - b'A' as u16 + b'a' as u16;
            }
            p = p.add(1);
        }
        wsrc
    } else {
        _wcslwr_s_l(wsrc, UNBOUNDED, core::ptr::null_mut());
        wsrc
    }
}

// ---------------------------------------------------------------------------
// wcsupr family
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn _wcsupr_s_l(wsrc: *mut u16, size_in_words: usize, plocinfo: LocaleT) -> ErrnoT {
    let mut lu = LocaleUpdate::new(plocinfo);
    wide_case_s_l(wsrc, size_in_words, lu.locale_t(), true)
}

#[no_mangle]
pub unsafe extern "C" fn _wcsupr_s(wsrc: *mut u16, size_in_words: usize) -> ErrnoT {
    _wcsupr_s_l(wsrc, size_in_words, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _wcsupr_l(wsrc: *mut u16, plocinfo: LocaleT) -> *mut u16 {
    _wcsupr_s_l(wsrc, UNBOUNDED, plocinfo);
    wsrc
}

#[no_mangle]
pub unsafe extern "C" fn _wcsupr(wsrc: *mut u16) -> *mut u16 {
    if __acrt_locale_changed_data == 0 {
        if validate_errcode(!wsrc.is_null(), EINVAL).is_err() {
            return core::ptr::null_mut();
        }
        let mut p = wsrc;
        while *p != 0 {
            let c = *p;
            if (b'a' as u16..=b'z' as u16).contains(&c) {
                *p = c - b'a' as u16 + b'A' as u16;
            }
            p = p.add(1);
        }
        wsrc
    } else {
        _wcsupr_s_l(wsrc, UNBOUNDED, core::ptr::null_mut());
        wsrc
    }
}
