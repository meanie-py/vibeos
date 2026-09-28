//! `tempnam.cpp`.

use core::ffi::{c_char, c_int};
use core::ptr::null_mut;

use super::{calloc, free};
use crate::stdio::TEMPNAM_LOCK;
use crate::stdio::{path_requires_backslash_narrow, path_requires_backslash_wide};

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn wcslen(s: *const u16) -> usize;
    fn strcat_s(dst: *mut u8, destsz: usize, src: *const u8) -> c_int;
    fn wcscat_s(dst: *mut u16, size: usize, src: *const u16) -> c_int;
    fn _access_s(path: *const u8, mode: c_int) -> c_int;
    fn _waccess_s(path: *const u16, mode: c_int) -> c_int;
    fn _dupenv_s(buf: *mut *mut u8, len: *mut usize, name: *const u8) -> c_int;
    fn _wdupenv_s(buf: *mut *mut u16, len: *mut usize, name: *const u16) -> c_int;
    fn _ultoa_s(value: u32, buffer: *mut u8, size: usize, radix: c_int) -> c_int;
    fn _ultow_s(value: u32, buffer: *mut u16, size: usize, radix: c_int) -> c_int;
    fn _errno() -> *mut c_int;
    fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: u32, r: usize) -> !;
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
}

const CRT_EACCES: c_int = 13;
/// `_TMP_MAX_S` (`TMP_MAX`, `_CRT_INT_MAX`).
const TMP_MAX_S: u32 = i32::MAX as u32;

#[cold]
unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }
}

// ── strip_quotes<Character> ─────────────────────────────────────────────────

/// Returns a newly `calloc`'d, quote-stripped copy of `source`, or null if
/// `source` has no `"` characters (matching the C's "no quotes, no problem"
/// short-circuit exactly, including that a null return there means "use
/// `source` unchanged", not "allocation failed").
unsafe fn strip_quotes_narrow(source: *const u8) -> *mut u8 {
    let mut quote_count = 0usize;
    let mut source_length = 0usize;
    let mut it = source;
    while *it != 0 {
        if *it == b'"' {
            quote_count += 1;
        }
        source_length += 1;
        it = it.add(1);
    }
    if quote_count == 0 {
        return null_mut();
    }

    let destination_length = source_length - quote_count + 1;
    let destination = calloc(destination_length, 1) as *mut u8;
    if destination.is_null() {
        return null_mut();
    }

    let mut dst = destination;
    let mut src = source;
    while *src != 0 {
        if *src != b'"' {
            *dst = *src;
            dst = dst.add(1);
        }
        src = src.add(1);
    }
    *dst = 0;
    destination
}

unsafe fn strip_quotes_wide(source: *const u16) -> *mut u16 {
    let mut quote_count = 0usize;
    let mut source_length = 0usize;
    let mut it = source;
    while *it != 0 {
        if *it == b'"' as u16 {
            quote_count += 1;
        }
        source_length += 1;
        it = it.add(1);
    }
    if quote_count == 0 {
        return null_mut();
    }

    let destination_length = source_length - quote_count + 1;
    let destination = calloc(destination_length, 2) as *mut u16;
    if destination.is_null() {
        return null_mut();
    }

    let mut dst = destination;
    let mut src = source;
    while *src != 0 {
        if *src != b'"' as u16 {
            *dst = *src;
            dst = dst.add(1);
        }
        src = src.add(1);
    }
    *dst = 0;
    destination
}

// ── get_directory<Character> ────────────────────────────────────────────────

/// The result of [`get_directory_narrow`]/[`get_directory_wide`]: the
/// directory to use, and (if the caller must free it) the pointer to free.
struct Directory<T> {
    directory: *const T,
    owned: *mut T,
}

unsafe fn get_directory_narrow(alternative: *const u8) -> Directory<u8> {
    static ROOT: [u8; 2] = *b"\\\0";
    static CWD: [u8; 2] = *b".\0";

    let mut tmp_value: *mut u8 = null_mut();
    let _ = _dupenv_s(&mut tmp_value, null_mut(), b"TMP\0".as_ptr());

    if !tmp_value.is_null() {
        if _access_s(tmp_value, 0) == 0 {
            return Directory { directory: tmp_value, owned: tmp_value };
        }
        let unquoted = strip_quotes_narrow(tmp_value);
        if !unquoted.is_null() {
            if _access_s(unquoted, 0) == 0 {
                free(tmp_value.cast());
                return Directory { directory: unquoted, owned: unquoted };
            }
            free(unquoted.cast());
        }
        free(tmp_value.cast());
    }

    if !alternative.is_null() && _access_s(alternative, 0) == 0 {
        return Directory { directory: alternative, owned: null_mut() };
    }

    if _access_s(ROOT.as_ptr(), 0) == 0 {
        return Directory { directory: ROOT.as_ptr(), owned: null_mut() };
    }
    Directory { directory: CWD.as_ptr(), owned: null_mut() }
}

unsafe fn get_directory_wide(alternative: *const u16) -> Directory<u16> {
    static ROOT: [u16; 2] = [b'\\' as u16, 0];
    static CWD: [u16; 2] = [b'.' as u16, 0];
    static TMP_NAME: [u16; 4] = [b'T' as u16, b'M' as u16, b'P' as u16, 0];

    let mut tmp_value: *mut u16 = null_mut();
    let _ = _wdupenv_s(&mut tmp_value, null_mut(), TMP_NAME.as_ptr());

    if !tmp_value.is_null() {
        if _waccess_s(tmp_value, 0) == 0 {
            return Directory { directory: tmp_value, owned: tmp_value };
        }
        let unquoted = strip_quotes_wide(tmp_value);
        if !unquoted.is_null() {
            if _waccess_s(unquoted, 0) == 0 {
                free(tmp_value.cast());
                return Directory { directory: unquoted, owned: unquoted };
            }
            free(unquoted.cast());
        }
        free(tmp_value.cast());
    }

    if !alternative.is_null() && _waccess_s(alternative, 0) == 0 {
        return Directory { directory: alternative, owned: null_mut() };
    }

    if _waccess_s(ROOT.as_ptr(), 0) == 0 {
        return Directory { directory: ROOT.as_ptr(), owned: null_mut() };
    }
    Directory { directory: CWD.as_ptr(), owned: null_mut() }
}

// ── compute_name<Character> ─────────────────────────────────────────────────

/// `compute_name`: appends a unique numeric suffix (base 10, via the SHARED
/// `_tempoff` counter -- distinct from `tmpnam`'s own `_tmpoff`, despite the
/// similar name, per the C's own comment) to `path_buffer` until a name that
/// does not already exist is found, resetting the counter to 1 whenever the
/// prefix has grown (otherwise a long-prefix directory that is entirely full
/// of temp files could loop forever).
unsafe fn compute_name_narrow(path_buffer: *const u8, suffix_pointer: *mut u8, suffix_count: usize, prefix_length: usize) -> bool {
    use super::tmp::{_old_pfxlen, _tempoff};

    __acrt_lock(TEMPNAM_LOCK);

    if _old_pfxlen < prefix_length as u32 {
        _tempoff = 1;
    }
    _old_pfxlen = prefix_length as u32;

    let first = _tempoff;
    let saved_errno = *_errno();
    let mut ok = false;

    loop {
        _tempoff += 1;
        if _tempoff.wrapping_sub(first) > TMP_MAX_S {
            *_errno() = saved_errno;
            break;
        }

        errcheck(_ultoa_s(_tempoff, suffix_pointer, suffix_count, 10));
        *_errno() = 0;

        if !(_access_s(path_buffer, 0) == 0 || *_errno() == CRT_EACCES) {
            *_errno() = saved_errno;
            ok = true;
            break;
        }
    }

    __acrt_unlock(TEMPNAM_LOCK);
    ok
}

unsafe fn compute_name_wide(path_buffer: *const u16, suffix_pointer: *mut u16, suffix_count: usize, prefix_length: usize) -> bool {
    use super::tmp::{_old_pfxlen, _tempoff};

    __acrt_lock(TEMPNAM_LOCK);

    if _old_pfxlen < prefix_length as u32 {
        _tempoff = 1;
    }
    _old_pfxlen = prefix_length as u32;

    let first = _tempoff;
    let saved_errno = *_errno();
    let mut ok = false;

    loop {
        _tempoff += 1;
        if _tempoff.wrapping_sub(first) > TMP_MAX_S {
            *_errno() = saved_errno;
            break;
        }

        errcheck(_ultow_s(_tempoff, suffix_pointer, suffix_count, 10));
        *_errno() = 0;

        if !(_waccess_s(path_buffer, 0) == 0 || *_errno() == CRT_EACCES) {
            *_errno() = saved_errno;
            ok = true;
            break;
        }
    }

    __acrt_unlock(TEMPNAM_LOCK);
    ok
}

// ── common_tempnam<Character> ───────────────────────────────────────────────

unsafe fn common_tempnam_narrow(alternative: *const u8, prefix: *const u8) -> *mut u8 {
    let dir = get_directory_narrow(alternative);

    let prefix_length = if !prefix.is_null() { strlen(prefix.cast()) } else { 0 };
    let buffer_size = strlen(dir.directory.cast()) + prefix_length + 12;

    let result = calloc(buffer_size, 1) as *mut u8;
    if result.is_null() {
        free(dir.owned.cast());
        return null_mut();
    }

    *result = 0;
    errcheck(strcat_s(result, buffer_size, dir.directory));

    if path_requires_backslash_narrow(dir.directory.cast()) {
        errcheck(strcat_s(result, buffer_size, b"\\\0".as_ptr()));
    }
    if !prefix.is_null() {
        errcheck(strcat_s(result, buffer_size, prefix));
    }

    free(dir.owned.cast());

    let ptr = result.add(strlen(result.cast()));
    let ptr_size = buffer_size - (ptr.offset_from(result) as usize);

    if !compute_name_narrow(result, ptr, ptr_size, prefix_length) {
        free(result.cast());
        return null_mut();
    }

    result
}

unsafe fn common_tempnam_wide(alternative: *const u16, prefix: *const u16) -> *mut u16 {
    let dir = get_directory_wide(alternative);

    let prefix_length = if !prefix.is_null() { wcslen(prefix) } else { 0 };
    let buffer_size = wcslen(dir.directory) + prefix_length + 12;

    let result = calloc(buffer_size, 2) as *mut u16;
    if result.is_null() {
        free(dir.owned.cast());
        return null_mut();
    }

    *result = 0;
    errcheck(wcscat_s(result, buffer_size, dir.directory));

    if path_requires_backslash_wide(dir.directory) {
        static BACKSLASH: [u16; 2] = [b'\\' as u16, 0];
        errcheck(wcscat_s(result, buffer_size, BACKSLASH.as_ptr()));
    }
    if !prefix.is_null() {
        errcheck(wcscat_s(result, buffer_size, prefix));
    }

    free(dir.owned.cast());

    let ptr = result.add(wcslen(result));
    let ptr_size = buffer_size - (ptr.offset_from(result) as usize);

    if !compute_name_wide(result, ptr, ptr_size, prefix_length) {
        free(result.cast());
        return null_mut();
    }

    result
}

#[no_mangle]
pub unsafe extern "C" fn _tempnam(alternative: *const c_char, prefix: *const c_char) -> *mut c_char {
    common_tempnam_narrow(alternative.cast(), prefix.cast()).cast()
}

#[no_mangle]
pub unsafe extern "C" fn _wtempnam(alternative: *const u16, prefix: *const u16) -> *mut u16 {
    common_tempnam_wide(alternative, prefix)
}
