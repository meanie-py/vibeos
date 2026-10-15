//! The PathCch* / PathAlloc* family (api-ms-win-core-path-l1-1-0): a faithful port of Wine's
//! dlls/kernelbase/path.c. Windows exports these from kernelbase.dll; our api-set table
//! routes that api-set to kernel32.dll (our kernelbase is a thin shim), so they are exported
//! from here by name.
//!
//! All functions return HRESULT (PathCchIsRoot and PathIsUNCEx return BOOL). Allocating
//! variants return LocalAlloc'd buffers that the caller releases with LocalFree.
use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ptr;
use crate::client::heapmem::{LocalAlloc, LocalFree};

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

const S_OK_: HRESULT = 0;
const S_FALSE_: HRESULT = 1;
const E_INVALIDARG_: HRESULT = 0x8007_0057_u32 as HRESULT;
const E_OUTOFMEMORY_: HRESULT = 0x8007_000E_u32 as HRESULT;
const STRSAFE_E_INSUFFICIENT_BUFFER_: HRESULT = 0x8007_007A_u32 as HRESULT;
/// HRESULT_FROM_WIN32(ERROR_FILENAME_EXCED_RANGE = 206)
const HRESULT_FILENAME_EXCED_RANGE: HRESULT = 0x8007_00CE_u32 as HRESULT;

const MAX_PATH_: usize = 260;
const PATHCCH_MAX_CCH_: usize = 0x8000;

const PATHCCH_ALLOW_LONG_PATHS: u32 = 0x01;
const PATHCCH_FORCE_ENABLE_LONG_NAME_PROCESS: u32 = 0x02;
const PATHCCH_FORCE_DISABLE_LONG_NAME_PROCESS: u32 = 0x04;
const PATHCCH_DO_NOT_NORMALIZE_SEGMENTS: u32 = 0x08;
const PATHCCH_ENSURE_IS_EXTENDED_LENGTH_PATH: u32 = 0x10;
const PATHCCH_ENSURE_TRAILING_SLASH: u32 = 0x20;

const LMEM_ZEROINIT_: u32 = 0x40;

const BS: u16 = b'\\' as u16;
const DOT: u16 = b'.' as u16;
const STAR: u16 = b'*' as u16;
const COLON: u16 = b':' as u16;
const QMARK: u16 = b'?' as u16;

/// NUL-terminated UTF-16 array from an ASCII literal.
macro_rules! wlit {
    ($s:literal) => {{
        const N: usize = $s.len() + 1;
        const A: [u16; N] = {
            let b = $s.as_bytes();
            let mut a = [0u16; N];
            let mut i = 0;
            while i < b.len() {
                a[i] = b[i] as u16;
                i += 1;
            }
            a
        };
        A
    }};
}

#[inline]
fn is_alpha(c: u16) -> bool {
    (c >= b'A' as u16 && c <= b'Z' as u16) || (c >= b'a' as u16 && c <= b'z' as u16)
}

#[inline]
fn is_xdigit(c: u16) -> bool {
    (c >= b'0' as u16 && c <= b'9' as u16)
        || (c >= b'A' as u16 && c <= b'F' as u16)
        || (c >= b'a' as u16 && c <= b'f' as u16)
}

#[inline]
unsafe fn is_drive_spec(s: *const u16) -> bool {
    is_alpha(*s) && *s.add(1) == COLON
}

/// `\\?\UNC\` (case-insensitive)
unsafe fn is_prefixed_unc(s: *const u16) -> bool {
    let pat = wlit!("\\\\?\\UNC\\");
    _wcsnicmp(s, pat.as_ptr(), 8) == 0
}

/// `\\?\X:`
unsafe fn is_prefixed_disk(s: *const u16) -> bool {
    *s == BS && *s.add(1) == BS && *s.add(2) == QMARK && *s.add(3) == BS && is_drive_spec(s.add(4))
}

/// `\\?\Volume{GUID}`
unsafe fn is_prefixed_volume(s: *const u16) -> bool {
    let pat = wlit!("\\\\?\\Volume");
    if _wcsnicmp(s, pat.as_ptr(), 10) != 0 {
        return false;
    }
    let guid = s.add(10);
    let mut i = 0usize;
    while i <= 37 {
        let c = *guid.add(i);
        match i {
            0 => {
                if c != b'{' as u16 {
                    return false;
                }
            }
            9 | 14 | 19 | 24 => {
                if c != b'-' as u16 {
                    return false;
                }
            }
            37 => {
                if c != b'}' as u16 {
                    return false;
                }
            }
            _ => {
                if !is_xdigit(c) {
                    return false;
                }
            }
        }
        i += 1;
    }
    true
}

/// Advance past the current segment; returns whether the segment ended in a backslash.
unsafe fn get_next_segment(mut next: *const u16, next_segment: *mut *const u16) -> bool {
    while *next != 0 && *next != BS {
        next = next.add(1);
    }
    if *next == BS {
        *next_segment = next.add(1);
        true
    } else {
        *next_segment = next;
        false
    }
}

/// The last character of the root of `path`, or NULL if it has none.
unsafe fn get_root_end(path: *const u16) -> *const u16 {
    if is_prefixed_volume(path) {
        if *path.add(48) == BS { path.add(48) } else { path.add(47) }
    } else if is_prefixed_unc(path) {
        path.add(7)
    } else if is_prefixed_disk(path) {
        if *path.add(6) == BS { path.add(6) } else { path.add(5) }
    } else if *path == BS && *path.add(1) == BS {
        path.add(1)
    } else if *path == BS {
        path
    } else if is_drive_spec(path) {
        if *path.add(2) == BS { path.add(2) } else { path.add(1) }
    } else {
        ptr::null()
    }
}

/// wcsnlen
unsafe fn wnlen(s: *const u16, max: usize) -> usize {
    let mut n = 0usize;
    while n < max && *s.add(n) != 0 {
        n += 1;
    }
    n
}

#[no_mangle]
pub unsafe extern "system" fn PathAllocCanonicalize(
    path_in: PCWSTR,
    mut flags: DWORD,
    path_out: *mut *mut u16,
) -> HRESULT {
    if path_in.is_null()
        || path_out.is_null()
        || (flags & PATHCCH_FORCE_ENABLE_LONG_NAME_PROCESS != 0
            && flags & PATHCCH_FORCE_DISABLE_LONG_NAME_PROCESS != 0)
        || (flags & (PATHCCH_FORCE_ENABLE_LONG_NAME_PROCESS | PATHCCH_FORCE_DISABLE_LONG_NAME_PROCESS) != 0
            && flags & PATHCCH_ALLOW_LONG_PATHS == 0)
        || (flags & PATHCCH_ENSURE_IS_EXTENDED_LENGTH_PATH != 0 && flags & PATHCCH_ALLOW_LONG_PATHS != 0)
    {
        if !path_out.is_null() {
            *path_out = ptr::null_mut();
        }
        return E_INVALIDARG_;
    }

    let mut length = wcslen(path_in);
    if (length + 1 > MAX_PATH_
        && flags & (PATHCCH_ALLOW_LONG_PATHS | PATHCCH_ENSURE_IS_EXTENDED_LENGTH_PATH) == 0)
        || length + 1 > PATHCCH_MAX_CCH_
    {
        *path_out = ptr::null_mut();
        return HRESULT_FILENAME_EXCED_RANGE;
    }

    // PATHCCH_ENSURE_IS_EXTENDED_LENGTH_PATH implies PATHCCH_DO_NOT_NORMALIZE_SEGMENTS
    if flags & PATHCCH_ENSURE_IS_EXTENDED_LENGTH_PATH != 0 {
        flags |= PATHCCH_DO_NOT_NORMALIZE_SEGMENTS;
    }

    // path length + possible \\?\ addition + possible \ addition + NUL
    let buffer_chars = length + 6;
    let buffer = LocalAlloc(LMEM_ZEROINIT_, (buffer_chars * 2) as SIZE_T) as *mut u16;
    if buffer.is_null() {
        *path_out = ptr::null_mut();
        return E_OUTOFMEMORY_;
    }

    let mut src: *const u16 = path_in;
    let mut dst: *mut u16 = buffer;

    let mut root_end: *const u16 = get_root_end(path_in);
    if !root_end.is_null() {
        root_end = buffer.offset(root_end.offset_from(path_in)) as *const u16;
    }

    // Copy path root
    if !root_end.is_null() {
        let root_len = root_end.offset_from(buffer as *const u16) as usize + 1;
        ptr::copy_nonoverlapping(src, dst, root_len);
        src = src.add(root_len);
        if PathCchStripPrefix(dst, length + 6) == S_OK_ {
            // Fill in \ in X:\ if the \ is missing
            if is_drive_spec(dst) && *dst.add(2) != BS {
                *dst.add(2) = BS;
                *dst.add(3) = 0;
            }
            dst = buffer.add(wcslen(buffer));
            root_end = dst;
        } else {
            dst = dst.add(root_len);
        }
    }

    while *src != 0 {
        if *src == DOT {
            if *src.add(1) == DOT {
                // Keep one . after *
                if dst > buffer && *dst.sub(1) == STAR {
                    *dst = *src;
                    dst = dst.add(1);
                    src = src.add(1);
                    continue;
                }

                // Keep the .. if not surrounded by \
                if (*src.add(2) != BS && *src.add(2) != 0) || (dst > buffer && *dst.sub(1) != BS) {
                    *dst = *src;
                    *dst.add(1) = *src.add(1);
                    dst = dst.add(2);
                    src = src.add(2);
                    continue;
                }

                // Remove the \ before .. if the \ is not part of root
                if dst > buffer && *dst.sub(1) == BS && (root_end.is_null() || dst.sub(1) as *const u16 > root_end) {
                    dst = dst.sub(1);
                    *dst = 0;
                    // Remove characters until a \ is encountered
                    while dst > buffer {
                        if *dst.sub(1) == BS {
                            dst = dst.sub(1);
                            *dst = 0;
                            break;
                        } else {
                            dst = dst.sub(1);
                            *dst = 0;
                        }
                    }
                }
                // Remove the extra \ after .. if the \ before .. wasn't deleted
                else if *src.add(2) == BS {
                    src = src.add(1);
                }

                src = src.add(2);
            } else {
                // Keep the . if not surrounded by \
                if (*src.add(1) != BS && *src.add(1) != 0) || (dst > buffer && *dst.sub(1) != BS) {
                    *dst = *src;
                    dst = dst.add(1);
                    src = src.add(1);
                    continue;
                }

                // Remove the \ before . if the \ is not part of root
                if dst > buffer && *dst.sub(1) == BS && (root_end.is_null() || dst.sub(1) as *const u16 > root_end) {
                    dst = dst.sub(1);
                }
                // Remove the extra \ after . if the \ before . wasn't deleted
                else if *src.add(1) == BS {
                    src = src.add(1);
                }

                src = src.add(1);
            }

            // If X:\ is not complete, then complete it
            if is_drive_spec(buffer) && *buffer.add(2) != BS {
                root_end = buffer.add(2);
                dst = buffer.add(3);
                *buffer.add(2) = BS;
                // If next character is \, use the \ to fill in
                if *src == BS {
                    src = src.add(1);
                }
            }
        } else {
            // Copy over
            *dst = *src;
            dst = dst.add(1);
            src = src.add(1);
        }
    }
    // End the path
    *dst = 0;

    // Strip multiple trailing .
    if flags & PATHCCH_DO_NOT_NORMALIZE_SEGMENTS == 0 {
        while dst > buffer && *dst.sub(1) == DOT {
            // Keep a . after *
            if dst.sub(1) > buffer && *dst.sub(2) == STAR {
                break;
            }
            // If . follow a : at the second character, remove the . and add a \
            else if dst.sub(1) > buffer && *dst.sub(2) == COLON && dst.sub(2) == buffer.add(1) {
                dst = dst.sub(1);
                *dst = BS;
            } else {
                dst = dst.sub(1);
                *dst = 0;
            }
        }
    }

    // If result path is empty, fill in \
    if *buffer == 0 {
        *buffer = BS;
        *buffer.add(1) = 0;
    }

    // Extend the path if needed
    length = wcslen(buffer);
    if ((length + 1 > MAX_PATH_ && is_drive_spec(buffer))
        || (is_drive_spec(buffer) && flags & PATHCCH_ENSURE_IS_EXTENDED_LENGTH_PATH != 0))
        && flags & PATHCCH_FORCE_ENABLE_LONG_NAME_PROCESS == 0
    {
        ptr::copy(buffer, buffer.add(4), length + 1);
        *buffer = BS;
        *buffer.add(1) = BS;
        *buffer.add(2) = QMARK;
        *buffer.add(3) = BS;
    }

    // Add a trailing backslash to the path if needed
    if flags & PATHCCH_ENSURE_TRAILING_SLASH != 0 {
        PathCchAddBackslash(buffer, buffer_chars);
    }

    *path_out = buffer;
    S_OK_
}

#[no_mangle]
pub unsafe extern "system" fn PathAllocCombine(
    mut path1: PCWSTR,
    mut path2: PCWSTR,
    flags: DWORD,
    out: *mut *mut u16,
) -> HRESULT {
    let mut add_backslash = false;

    if (path1.is_null() && path2.is_null()) || out.is_null() {
        if !out.is_null() {
            *out = ptr::null_mut();
        }
        return E_INVALIDARG_;
    }

    if path1.is_null() || path2.is_null() {
        return PathAllocCanonicalize(if !path1.is_null() { path1 } else { path2 }, flags, out);
    }

    // If path2 is fully qualified, use path2 only
    if is_drive_spec(path2) || (*path2 == BS && *path2.add(1) == BS) {
        path1 = path2;
        path2 = ptr::null();
        add_backslash = (is_drive_spec(path1) && *path1.add(2) == 0)
            || (is_prefixed_disk(path1) && *path1.add(6) == 0);
    }

    let length2 = if path2.is_null() { 0 } else { wcslen(path2) };
    // path1 length + path2 length + possible backslash + NUL
    let combined_length = wcslen(path1) + length2 + 2;

    let combined_path = LocalAlloc(0, (combined_length * 2) as SIZE_T) as *mut u16;
    if combined_path.is_null() {
        *out = ptr::null_mut();
        return E_OUTOFMEMORY_;
    }

    wcscpy(combined_path, path1);
    PathCchStripPrefix(combined_path, combined_length);
    if add_backslash {
        PathCchAddBackslashEx(combined_path, combined_length, ptr::null_mut(), ptr::null_mut());
    }

    if !path2.is_null() && *path2 != 0 {
        if *path2 == BS && *path2.add(1) != BS {
            PathCchStripToRoot(combined_path, combined_length);
            path2 = path2.add(1);
        }

        PathCchAddBackslashEx(combined_path, combined_length, ptr::null_mut(), ptr::null_mut());
        wcscat(combined_path, path2);
    }

    let hr = PathAllocCanonicalize(combined_path, flags, out);
    LocalFree(combined_path as HLOCAL);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn PathCchAddBackslash(path: *mut u16, size: usize) -> HRESULT {
    PathCchAddBackslashEx(path, size, ptr::null_mut(), ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn PathCchAddBackslashEx(
    path: *mut u16,
    size: usize,
    endptr: *mut *mut u16,
    remaining: *mut usize,
) -> HRESULT {
    let mut length = wcslen(path);
    let needs_termination = size != 0 && length != 0 && *path.add(length - 1) != BS;

    if length >= (if needs_termination { size - 1 } else { size }) {
        if !endptr.is_null() {
            *endptr = ptr::null_mut();
        }
        if !remaining.is_null() {
            *remaining = 0;
        }
        return STRSAFE_E_INSUFFICIENT_BUFFER_;
    }

    if !needs_termination {
        if !endptr.is_null() {
            *endptr = path.add(length);
        }
        if !remaining.is_null() {
            *remaining = size - length;
        }
        return S_FALSE_;
    }

    *path.add(length) = BS;
    length += 1;
    *path.add(length) = 0;

    if !endptr.is_null() {
        *endptr = path.add(length);
    }
    if !remaining.is_null() {
        *remaining = size - length;
    }
    S_OK_
}

#[no_mangle]
pub unsafe extern "system" fn PathCchAddExtension(path: *mut u16, size: usize, extension: PCWSTR) -> HRESULT {
    if path.is_null() || size == 0 || size > PATHCCH_MAX_CCH_ || extension.is_null() {
        return E_INVALIDARG_;
    }

    let mut next = extension;
    while *next != 0 {
        if (*next == DOT && next > extension) || *next == b' ' as u16 || *next == BS {
            return E_INVALIDARG_;
        }
        next = next.add(1);
    }

    let has_dot = *extension == DOT;

    let mut existing_extension: *const u16 = ptr::null();
    let hr = PathCchFindExtension(path, size, &mut existing_extension);
    if hr < 0 {
        return hr;
    }
    if *existing_extension != 0 {
        return S_FALSE_;
    }

    let mut path_length = wnlen(path, size);
    let dot_length = if has_dot { 0 } else { 1 };
    let extension_length = wcslen(extension);

    if path_length + dot_length + extension_length + 1 > size {
        return STRSAFE_E_INSUFFICIENT_BUFFER_;
    }

    // If extension is empty or only dot, return S_OK with path unchanged
    if *extension == 0 || (*extension == DOT && *extension.add(1) == 0) {
        return S_OK_;
    }

    if !has_dot {
        *path.add(path_length) = DOT;
        path_length += 1;
    }

    wcscpy(path.add(path_length), extension);
    S_OK_
}

#[no_mangle]
pub unsafe extern "system" fn PathCchAppend(path1: *mut u16, size: usize, path2: PCWSTR) -> HRESULT {
    PathCchAppendEx(path1, size, path2, 0)
}

#[no_mangle]
pub unsafe extern "system" fn PathCchAppendEx(
    path1: *mut u16,
    size: usize,
    mut path2: PCWSTR,
    flags: DWORD,
) -> HRESULT {
    if path1.is_null() || size == 0 {
        return E_INVALIDARG_;
    }

    // A temporary result keeps path1 unchanged on failure (PathCchCombineEx writes an empty
    // string on error, so path1 cannot be its output directly).
    let result = LocalAlloc(0, (size * 2) as SIZE_T) as *mut u16;
    if result.is_null() {
        return E_OUTOFMEMORY_;
    }

    // Avoid the single backslash behavior with PathCchCombineEx when appending
    if !path2.is_null() && *path2 == BS && *path2.add(1) != BS {
        path2 = path2.add(1);
    }

    let hr = PathCchCombineEx(result, size, path1, path2, flags);
    if hr >= 0 {
        ptr::copy_nonoverlapping(result, path1, size);
    }

    LocalFree(result as HLOCAL);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn PathCchCanonicalize(out: *mut u16, size: usize, in_: PCWSTR) -> HRESULT {
    // Not X:\ and path > MAX_PATH - 4: HRESULT_FROM_WIN32(ERROR_FILENAME_EXCED_RANGE)
    if wcslen(in_) > MAX_PATH_ - 4 && !(is_drive_spec(in_) && *in_.add(2) == BS) {
        return HRESULT_FILENAME_EXCED_RANGE;
    }

    PathCchCanonicalizeEx(out, size, in_, 0)
}

#[no_mangle]
pub unsafe extern "system" fn PathCchCanonicalizeEx(
    out: *mut u16,
    size: usize,
    in_: PCWSTR,
    flags: DWORD,
) -> HRESULT {
    if size == 0 {
        return E_INVALIDARG_;
    }

    let mut buffer: *mut u16 = ptr::null_mut();
    let mut hr = PathAllocCanonicalize(in_, flags, &mut buffer);
    if hr < 0 {
        return hr;
    }

    let length = wcslen(buffer);
    if size < length + 1 {
        // No root and path > MAX_PATH - 4: HRESULT_FROM_WIN32(ERROR_FILENAME_EXCED_RANGE)
        if length > MAX_PATH_ - 4 && !(*in_ == BS || (is_drive_spec(in_) && *in_.add(2) == BS)) {
            hr = HRESULT_FILENAME_EXCED_RANGE;
        } else {
            hr = STRSAFE_E_INSUFFICIENT_BUFFER_;
        }
    }

    if hr >= 0 {
        ptr::copy_nonoverlapping(buffer, out, length + 1);

        // Fill a backslash at the end of X:
        if is_drive_spec(out) && *out.add(2) == 0 && size > 3 {
            *out.add(2) = BS;
            *out.add(3) = 0;
        }
    }

    LocalFree(buffer as HLOCAL);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn PathCchCombine(
    out: *mut u16,
    size: usize,
    path1: PCWSTR,
    path2: PCWSTR,
) -> HRESULT {
    PathCchCombineEx(out, size, path1, path2, 0)
}

#[no_mangle]
pub unsafe extern "system" fn PathCchCombineEx(
    out: *mut u16,
    size: usize,
    path1: PCWSTR,
    path2: PCWSTR,
    flags: DWORD,
) -> HRESULT {
    if out.is_null() || size == 0 || size > PATHCCH_MAX_CCH_ {
        return E_INVALIDARG_;
    }

    let mut buffer: *mut u16 = ptr::null_mut();
    let hr = PathAllocCombine(path1, path2, flags, &mut buffer);
    if hr < 0 {
        *out = 0;
        return hr;
    }

    let length = wcslen(buffer);
    if length + 1 > size {
        *out = 0;
        LocalFree(buffer as HLOCAL);
        STRSAFE_E_INSUFFICIENT_BUFFER_
    } else {
        ptr::copy_nonoverlapping(buffer, out, length + 1);
        LocalFree(buffer as HLOCAL);
        S_OK_
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathCchFindExtension(
    path: PCWSTR,
    size: usize,
    extension: *mut *const u16,
) -> HRESULT {
    if extension.is_null() {
        return E_INVALIDARG_;
    }
    if path.is_null() || size == 0 || size > PATHCCH_MAX_CCH_ {
        *extension = ptr::null();
        return E_INVALIDARG_;
    }

    let mut p = path;
    let mut lastpoint: *const u16 = ptr::null();
    let mut counter = 0usize;
    while *p != 0 {
        if *p == BS || *p == b' ' as u16 {
            lastpoint = ptr::null();
        } else if *p == DOT {
            lastpoint = p;
        }

        p = p.add(1);
        counter += 1;
        if counter == size || counter == PATHCCH_MAX_CCH_ {
            *extension = ptr::null();
            return E_INVALIDARG_;
        }
    }

    *extension = if !lastpoint.is_null() { lastpoint } else { p };
    S_OK_
}

#[no_mangle]
pub unsafe extern "system" fn PathCchIsRoot(path: PCWSTR) -> BOOL {
    if path.is_null() || *path == 0 {
        return FALSE;
    }

    let root_end = get_root_end(path);
    if root_end.is_null() {
        return FALSE;
    }

    let is_unc = is_prefixed_unc(path);
    if is_unc || (*path == BS && *path.add(1) == BS && *path.add(2) != QMARK) {
        let mut next = root_end.add(1);
        // No extra segments
        if *next == 0 {
            return TRUE;
        }

        // Has first segment with an ending backslash but no remaining characters
        if get_next_segment(next, &mut next) && *next == 0 {
            return FALSE;
        }
        // Has first segment with no ending backslash
        else if *next == 0 {
            return TRUE;
        }
        // Has first segment with an ending backslash and has remaining characters
        else {
            // Second segment must have no backslash and no remaining characters
            return (!get_next_segment(next, &mut next) && *next == 0) as BOOL;
        }
    } else if *root_end == BS && *root_end.add(1) == 0 {
        TRUE
    } else {
        FALSE
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathCchRemoveBackslash(path: *mut u16, path_size: usize) -> HRESULT {
    let mut path_end: *mut u16 = ptr::null_mut();
    let mut free_size: usize = 0;
    PathCchRemoveBackslashEx(path, path_size, &mut path_end, &mut free_size)
}

#[no_mangle]
pub unsafe extern "system" fn PathCchRemoveBackslashEx(
    path: *mut u16,
    path_size: usize,
    path_end: *mut *mut u16,
    free_size: *mut usize,
) -> HRESULT {
    if path_size == 0 || path_end.is_null() || free_size.is_null() {
        if !path_end.is_null() {
            *path_end = ptr::null_mut();
        }
        if !free_size.is_null() {
            *free_size = 0;
        }
        return E_INVALIDARG_;
    }

    let path_length = wnlen(path, path_size);
    if path_length == path_size && *path.add(path_length) == 0 {
        return E_INVALIDARG_;
    }

    let root_end = get_root_end(path);
    if path_length > 0 && *path.add(path_length - 1) == BS {
        *path_end = path.add(path_length - 1);
        *free_size = path_size - path_length + 1;
        // If the last character is beyond end of root
        if root_end.is_null() || path.add(path_length - 1) as *const u16 > root_end {
            *path.add(path_length - 1) = 0;
            S_OK_
        } else {
            S_FALSE_
        }
    } else {
        *path_end = path.add(path_length);
        *free_size = path_size - path_length;
        S_FALSE_
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathCchRemoveExtension(path: *mut u16, size: usize) -> HRESULT {
    if path.is_null() || size == 0 || size > PATHCCH_MAX_CCH_ {
        return E_INVALIDARG_;
    }

    let mut extension: *const u16 = ptr::null();
    let hr = PathCchFindExtension(path, size, &mut extension);
    if hr < 0 {
        return hr;
    }

    let mut next = path.offset(extension.offset_from(path as *const u16));
    while (next.offset_from(path) as usize) < size && *next != 0 {
        *next = 0;
        next = next.add(1);
    }

    if next as *const u16 == extension { S_FALSE_ } else { S_OK_ }
}

#[no_mangle]
pub unsafe extern "system" fn PathCchRemoveFileSpec(path: *mut u16, size: usize) -> HRESULT {
    if path.is_null() || size == 0 || size > PATHCCH_MAX_CCH_ {
        return E_INVALIDARG_;
    }

    let mut root_end_c: *const u16 = ptr::null();
    let mut root_end: *mut u16 = if PathCchSkipRoot(path, &mut root_end_c) < 0 {
        path
    } else {
        root_end_c as *mut u16
    };

    // The backslash at the end of UNC and \\* are not considered part of root in this case
    if root_end > path
        && *root_end.sub(1) == BS
        && ((is_prefixed_unc(path) && *path.add(8) != 0)
            || (*path == BS && *path.add(1) == BS && *path.add(2) != 0 && *path.add(2) != QMARK))
    {
        root_end = root_end.sub(1);
    }

    let mut last = wcsrchr(root_end, BS);
    if last.is_null() {
        last = root_end;
    }
    if last > root_end && *last.sub(1) == BS && *last.add(1) != QMARK {
        last = last.sub(1);
    }
    if last.offset_from(path) as usize >= size {
        return E_INVALIDARG_;
    }
    if *last == 0 {
        return S_FALSE_;
    }
    *last = 0;
    S_OK_
}

#[no_mangle]
pub unsafe extern "system" fn PathCchRenameExtension(path: *mut u16, size: usize, extension: PCWSTR) -> HRESULT {
    let hr = PathCchRemoveExtension(path, size);
    if hr < 0 {
        return hr;
    }

    let hr = PathCchAddExtension(path, size, extension);
    if hr < 0 { hr } else { S_OK_ }
}

#[no_mangle]
pub unsafe extern "system" fn PathCchSkipRoot(path: PCWSTR, root_end: *mut *const u16) -> HRESULT {
    if path.is_null() || *path == 0 || root_end.is_null() {
        return E_INVALIDARG_;
    }
    let prefix = wlit!("\\\\?");
    if _wcsnicmp(path, prefix.as_ptr(), 3) == 0
        && !is_prefixed_volume(path)
        && !is_prefixed_unc(path)
        && !is_prefixed_disk(path)
    {
        return E_INVALIDARG_;
    }

    *root_end = get_root_end(path);
    if !(*root_end).is_null() {
        *root_end = (*root_end).add(1);
        if is_prefixed_unc(path) {
            get_next_segment(*root_end, root_end);
            get_next_segment(*root_end, root_end);
        } else if *path == BS && *path.add(1) == BS && *path.add(2) != QMARK {
            // Skip share server
            get_next_segment(*root_end, root_end);
            // If mount point is empty, don't skip over mount point
            if **root_end != BS {
                get_next_segment(*root_end, root_end);
            }
        }
    }

    if (*root_end).is_null() { E_INVALIDARG_ } else { S_OK_ }
}

#[no_mangle]
pub unsafe extern "system" fn PathCchStripPrefix(path: *mut u16, size: usize) -> HRESULT {
    if path.is_null() || size == 0 || size > PATHCCH_MAX_CCH_ {
        return E_INVALIDARG_;
    }

    if is_prefixed_unc(path) {
        // \\?\UNC\a -> \\a
        if size < wcslen(path.add(8)) + 3 {
            return E_INVALIDARG_;
        }
        // Overlapping move: the tail slides down over the prefix.
        let n = wcslen(path.add(8)) + 1;
        ptr::copy(path.add(8), path.add(2), n);
        S_OK_
    } else if is_prefixed_disk(path) {
        // \\?\C:\ -> C:\
        if size < wcslen(path.add(4)) + 1 {
            return E_INVALIDARG_;
        }
        let n = wcslen(path.add(4)) + 1;
        ptr::copy(path.add(4), path, n);
        S_OK_
    } else {
        S_FALSE_
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathCchStripToRoot(path: *mut u16, size: usize) -> HRESULT {
    if path.is_null() || *path == 0 || size == 0 || size > PATHCCH_MAX_CCH_ {
        return E_INVALIDARG_;
    }

    let mut root_end: *const u16 = ptr::null();
    if PathCchSkipRoot(path, &mut root_end) == S_OK_ {
        if !root_end.is_null()
            && root_end > path as *const u16
            && *root_end.sub(1) == BS
            && ((is_prefixed_unc(path) && *path.add(8) != 0)
                || (*path == BS && *path.add(1) == BS && *path.add(2) != 0 && *path.add(2) != QMARK))
        {
            root_end = root_end.sub(1);
        }
        if root_end.offset_from(path as *const u16) as usize >= size {
            return E_INVALIDARG_;
        }

        let segment_end = path.offset(root_end.offset_from(path as *const u16));
        if *segment_end == 0 {
            return S_FALSE_;
        }

        *segment_end = 0;
        S_OK_
    } else {
        *path = 0;
        E_INVALIDARG_
    }
}

#[no_mangle]
pub unsafe extern "system" fn PathIsUNCEx(path: PCWSTR, server: *mut *const u16) -> BOOL {
    let mut result: *const u16 = ptr::null();

    if is_prefixed_unc(path) {
        result = path.add(8);
    } else if *path == BS && *path.add(1) == BS && *path.add(2) != QMARK {
        result = path.add(2);
    }

    if !server.is_null() {
        *server = result;
    }
    (!result.is_null()) as BOOL
}
