//! `_fullpath`/`_wfullpath`. Replaces `sdk/lib/ucrt/filesystem/fullpath.cpp`.
//! The `_DEBUG`-only `_fullpath_dbg`/`_wfullpath_dbg` forms are not exported
//! by a release CRT and are not ported.
//!
//! Reuses `crate::findfile`'s `file_codepage`/`mbs_to_wcs`/`wcs_to_mbs`/
//! `os_errno`/`Buf` for the parts that match its contract exactly (see
//! below for the one place that genuinely needs different code). GetFullPathNameW
//! resolution itself -- the call-twice-and-grow pattern, and the no-resize
//! fixed-buffer variant -- has no counterpart in `findfile.rs` (it does no
//! path resolution at all), so both live here.
//!
//! ## Where this deliberately does NOT reuse `findfile::Buf::alloc`
//!
//! `Buf::alloc` is a private associated fn of `findfile::Buf<C>` (not
//! `pub(crate)`), so it cannot be called from this module. `resolve_wide`'s
//! allocation (a plain "get me N wide chars of heap, self-freeing" -- nothing
//! to do with narrow<->wide conversion) instead constructs a `findfile::Buf`
//! directly through its `pub(crate)` tuple field, via the same
//! `_malloc_base` the private `alloc` uses (`corecrt_internal_win32_buffer.h`'s
//! `__crt_win32_buffer_internal_dynamic_resizing::allocate` is `_malloc_crt`,
//! which is plain `malloc`, which is `_malloc_base` -- see `findfile.rs`'s
//! own comment on the same fact). `detach` is likewise a local free function
//! over the same `pub(crate)` field, since `findfile::Buf` has no `detach`
//! method (nothing in `findfile.rs` ever hands an allocation to a caller).
//!
//! ## Details that are silent if wrong
//!
//! * **An empty or NULL `path` skips path resolution entirely** and returns
//!   the process's OWN current directory via `_getcwd`/`_wgetcwd` (which
//!   itself allocates when `user_buffer` is NULL) -- it does NOT resolve `""`
//!   through `GetFullPathName` against the cwd.
//! * **A caller-supplied buffer NEVER grows.** `_wfullpath` calls
//!   `GetFullPathNameW` exactly once into `user_buffer`/`max_count`; if the
//!   resolved path (plus NUL) doesn't fit, this returns NULL with
//!   `errno=ERANGE` and the buffer's contents are left as whatever
//!   `GetFullPathNameW` wrote (which, per its own contract, is nothing on
//!   this failure path). Only a NULL user buffer allocates -- sized to the
//!   exact resolved length, freeable with `free()`.
//! * **The narrow form's intermediate WIDE resolution is never capped by
//!   `max_count`**: `_fullpath` converts the input to wide, resolves it via
//!   an internally growable buffer (starting at `_MAX_PATH`, exactly like
//!   `_findfirst`'s conversions), and only the FINAL wide-to-narrow
//!   conversion respects `max_count`/NULL-allocates. A path whose wide form
//!   needs an enormous buffer still resolves; only the narrow re-encoding can
//!   fail on size.
//! * **`max_count`/the getcwd length is clamped to `INT_MAX`**, since
//!   `_getcwd`/`_wgetcwd` take a signed `int` size.
//! * **The narrow re-encoding's empty-string special case is mirrored on the
//!   FIXED-buffer path too**, matching `__acrt_convert_wcs_mbs_cp`'s own
//!   branching (it checks `null_terminated_input_string[0] == '\0'` BEFORE
//!   looking at the resize policy at all, so a no-resize destination with
//!   `capacity() > 0` still just writes `'\0'` and succeeds regardless of
//!   `max_count`). Unreachable in practice -- `GetFullPathNameW` never
//!   resolves to an empty string -- but kept for exact fidelity since
//!   `findfile::wcs_to_mbs` has no fixed-capacity variant to inherit this
//!   from automatically.

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use crate::findfile::{file_codepage, mbs_to_wcs, os_errno, wcs_to_mbs, Buf};

use super::helpers::{__acrt_WideCharToMultiByte, _getcwd, _wgetcwd, wide_ptr, GetFullPathNameW, ERANGE, _errno};

const ENOMEM: c_int = 12;

extern "C" {
    fn _malloc_base(size: usize) -> *mut c_void;
}

/// See the module doc: `Buf::alloc` is private to `findfile.rs`, so a plain
/// "N wide chars of self-freeing heap" allocation (nothing to do with a
/// narrow<->wide conversion) is built here the same way `alloc` itself is,
/// through the `pub(crate)` tuple field.
unsafe fn alloc_buf(count: usize) -> Result<Buf<u16>, c_int> {
    let p = _malloc_base(count * core::mem::size_of::<u16>()) as *mut u16;
    if p.is_null() {
        return Err(ENOMEM);
    }
    Ok(Buf(p))
}

/// `__crt_win32_buffer::detach`, restricted to the always-dynamic case this
/// file only ever uses it for: hand the allocation to the caller (who now
/// owns it and must eventually `free()` it) without freeing it here.
/// `findfile::Buf` has no such method of its own -- see the module doc.
fn detach(mut b: Buf<u16>) -> *mut u16 {
    let p = b.0;
    b.0 = null_mut();
    p
}

/// `_MAX_PATH`: the initial (growable) capacity for the wide resolution
/// buffer, matching `__acrt_get_full_path_name_wide`'s own default stack
/// buffer size.
const INITIAL_WIDE_CAPACITY: usize = 260;

/// `__crt_win32_buffer<wchar_t, internal_dynamic_resizing>::call_win32_function`
/// applied to `GetFullPathNameW`: try `initial_cap` first, grow once (to
/// whatever Win32 reports is needed, plus the C's own redundant `+ 1`) if
/// that's not enough.
unsafe fn resolve_wide(path: *const u16) -> Result<Buf<u16>, c_int> {
    let mut buf = alloc_buf(INITIAL_WIDE_CAPACITY)?;
    let need = GetFullPathNameW(path, INITIAL_WIDE_CAPACITY as u32, buf.0, null_mut());
    if need == 0 {
        return Err(os_errno());
    }
    if (need as usize) <= INITIAL_WIDE_CAPACITY {
        return Ok(buf);
    }

    // NOTE (quirk, transcribed): the C adds its own "+1" on top of whatever
    // GetFullPathNameW already reported as the required size (which, on this
    // failure path, already includes room for the NUL) -- one byte of
    // harmless slack.
    let cap2 = need as usize + 1;
    buf = alloc_buf(cap2)?;
    let need2 = GetFullPathNameW(path, cap2 as u32, buf.0, null_mut());
    if need2 == 0 {
        return Err(os_errno());
    }
    Ok(buf)
}

/// `__crt_win32_buffer<wchar_t, no_resizing>::call_win32_function`: a single
/// call into the caller's own fixed buffer. Too small becomes ERANGE with no
/// second attempt.
unsafe fn resolve_wide_fixed(path: *const u16, buffer: *mut u16, cap: usize) -> Result<(), c_int> {
    let need = GetFullPathNameW(path, cap as u32, buffer, null_mut());
    if need == 0 {
        return Err(os_errno());
    }
    if (need as usize) <= cap {
        return Ok(());
    }
    *_errno() = ERANGE;
    Err(ERANGE)
}

/// `_wfullpath`.
#[no_mangle]
pub unsafe extern "C" fn _wfullpath(
    user_buffer: *mut u16,
    path: *const u16,
    max_count: usize,
) -> *mut u16 {
    // If the path is empty, we have no work to do:
    if path.is_null() || *path == 0 {
        let n = if max_count > i32::MAX as usize { i32::MAX } else { max_count as c_int };
        return _wgetcwd(user_buffer, n);
    }

    if !user_buffer.is_null() {
        match resolve_wide_fixed(path, user_buffer, max_count) {
            Ok(()) => user_buffer,
            Err(_) => null_mut(),
        }
    } else {
        match resolve_wide(path) {
            Ok(buf) => detach(buf),
            Err(_) => null_mut(),
        }
    }
}

/// `__acrt_convert_wcs_mbs_cp`'s no-resize instantiation: the empty-string
/// special case (write `'\0'`, succeed, regardless of `cap`) mirrored from
/// `findfile::wcs_to_mbs`, plus the size-then-write shape a fixed
/// destination needs (a too-small destination must never be written into,
/// only sized first).
unsafe fn wcs_to_mbs_fixed(src: *const u16, dst: *mut u8, cap: usize, cp: u32) -> Result<(), c_int> {
    if *src == 0 {
        if cap == 0 {
            *_errno() = ERANGE;
            return Err(ERANGE);
        }
        *dst = 0;
        return Ok(());
    }
    let need = __acrt_WideCharToMultiByte(cp, 0, src, -1, null_mut(), 0, core::ptr::null(), null_mut());
    if need == 0 {
        return Err(os_errno());
    }
    if need as usize > cap {
        *_errno() = ERANGE;
        return Err(ERANGE);
    }
    let n = __acrt_WideCharToMultiByte(cp, 0, src, -1, dst.cast(), cap as c_int, core::ptr::null(), null_mut());
    if n == 0 {
        return Err(os_errno());
    }
    Ok(())
}

/// `_fullpath`.
#[no_mangle]
pub unsafe extern "C" fn _fullpath(
    user_buffer: *mut u8,
    path: *const u8,
    max_count: usize,
) -> *mut u8 {
    // If the path is empty, we have no work to do:
    if path.is_null() || *path == 0 {
        let n = if max_count > i32::MAX as usize { i32::MAX } else { max_count as c_int };
        return _getcwd(user_buffer, n);
    }

    let code_page = file_codepage();

    let wide_path = match mbs_to_wcs(path, code_page) {
        Ok(w) => w,
        Err(_) => return null_mut(),
    };

    let resolved = match resolve_wide(wide_ptr(&wide_path)) {
        Ok(b) => b,
        Err(_) => return null_mut(),
    };

    if !user_buffer.is_null() {
        // __crt_no_alloc_win32_buffer<char>: the size is known up front, so a
        // too-small destination becomes ERANGE without ever writing.
        match wcs_to_mbs_fixed(resolved.0, user_buffer, max_count, code_page) {
            Ok(()) => user_buffer,
            Err(_) => null_mut(),
        }
    } else {
        // __crt_public_win32_buffer<char>: exactly `findfile::wcs_to_mbs`'s
        // own contract (including its empty-string special case), so this
        // reuses it directly rather than re-deriving the same three steps.
        match wcs_to_mbs(resolved.0, code_page) {
            Ok(out) => detach_narrow(out),
            Err(_) => null_mut(),
        }
    }
}

/// `detach`'s narrow twin, over `findfile::Buf<u8>` (the type `wcs_to_mbs`
/// returns).
fn detach_narrow(mut b: Buf<u8>) -> *mut u8 {
    let p = b.0;
    b.0 = null_mut();
    p
}
