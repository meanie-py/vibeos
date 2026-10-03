//! `misc/chdir.cpp` -- `_chdir`/`_wchdir`.
//!
//! Calls the real (still-C, exported, non-template) internal helpers
//! `__acrt_SetCurrentDirectoryA`/`__acrt_SetEnvironmentVariableA` directly for
//! the narrow path (`sdk/lib/ucrt/internal/SetCurrentDirectoryA.cpp`,
//! `SetEnvironmentVariableA.cpp` -- outside this batch, not templates, so
//! callable by their real C names) rather than re-deriving their
//! codepage-conversion logic a third time; the wide path calls
//! `SetCurrentDirectoryW`/`SetEnvironmentVariableW` directly, as the C does.
//! Reuses [`crate::misc::getcwd::grow_into`] for the internal (dynamically
//! growable) current-directory fetch.
//!
//! ## Details that are silent if wrong
//!
//! * A path is treated as UNC (skip the environment-variable update
//!   entirely, return 0) when its first two characters are both `\` or both
//!   `/` -- i.e. `path[0]` is a slash AND `path[0] == path[1]`; `\/` or `/\`
//!   are NOT UNC by this check even though Windows itself is slash-agnostic.
//! * The environment variable name is `"=" + toupper(path[0]) + ":"` --
//!   e.g. `=D:` -- built from the NEW directory's first character, not
//!   queried from the OS.
//! * The value written to that variable is NOT `path` itself but the FULL
//!   current directory as reported by `GetCurrentDirectoryW`/-A AFTER the
//!   `SetCurrentDirectory` call succeeded (so relative or short input paths
//!   get expanded before being stored).
//! * If `SetCurrentDirectory` succeeds but reading back the current
//!   directory fails, `_chdir` still returns -1 (propagating that failure),
//!   even though the OS's cwd HAS already changed -- the directory change is
//!   not rolled back.
//! * `_VALIDATE_CLEAR_OSSERR_RETURN(path != nullptr, EINVAL, -1)`: a NULL
//!   path clears `_doserrno` to 0 FIRST, then sets errno=EINVAL and invokes
//!   the (noinfo) handler -- unlike plain `_VALIDATE_RETURN`, which never
//!   touches `_doserrno`.

use core::ffi::{c_char, c_int, c_void};

const EINVAL: c_int = 22;
const MAX_PATH: usize = 260;

extern "system" {
    fn SetCurrentDirectoryW(path: *const u16) -> c_int;
    fn SetEnvironmentVariableW(name: *const u16, value: *const u16) -> c_int;
    fn GetCurrentDirectoryW(len: u32, buf: *mut u16) -> u32;
    fn GetLastError() -> u32;
}

extern "C" {
    fn _invalid_parameter_noinfo();
    fn __acrt_errno_map_os_error(err: u32);
    fn __acrt_SetCurrentDirectoryA(path: *const c_char) -> c_int;
    fn __acrt_SetEnvironmentVariableA(name: *const c_char, value: *const c_char) -> c_int;
    fn free(p: *mut c_void);
}

/// `_VALIDATE_CLEAR_OSSERR_RETURN(expr, err, ret)`: `_doserrno = 0` first,
/// then the ordinary `_VALIDATE_RETURN` sequence.
unsafe fn validate_clear_osserr(ok: bool, err: c_int) {
    if !ok {
        *crate::misc::errno::__doserrno() = 0;
        *crate::misc::errno::_errno() = err;
        _invalid_parameter_noinfo();
    }
}

/// `set_cwd_environment_variable<wchar_t>`.
unsafe fn set_cwd_env_wide(path: *const u16) -> c_int {
    if *path == b'\\' as u16 || *path == b'/' as u16 {
        if *path == *path.add(1) {
            return 0;
        }
    }
    let drive_letter = to_upper_ascii(*path);
    let name: [u16; 4] = [b'=' as u16, drive_letter, b':' as u16, 0];

    if SetEnvironmentVariableW(name.as_ptr(), path) != 0 {
        return 0;
    }
    __acrt_errno_map_os_error(GetLastError());
    -1
}

/// `set_cwd_environment_variable<char>`.
unsafe fn set_cwd_env_narrow(path: *const c_char) -> c_int {
    let p0 = *path as u8;
    if p0 == b'\\' || p0 == b'/' {
        if p0 == *path.add(1) as u8 {
            return 0;
        }
    }
    let drive_letter = to_upper_ascii(p0 as u16) as u8;
    let name: [u8; 4] = [b'=', drive_letter, b':', 0];

    if __acrt_SetEnvironmentVariableA(name.as_ptr().cast(), path) != 0 {
        return 0;
    }
    __acrt_errno_map_os_error(GetLastError());
    -1
}

/// `toupper(static_cast<char>(path[0]))`: the C's plain `toupper` from the
/// "C" locale here (not a `_l`/locale-aware call), which only ever touches
/// ASCII `a`..`z`.
fn to_upper_ascii(c: u16) -> u16 {
    if (b'a' as u16..=b'z' as u16).contains(&c) {
        c - 32
    } else {
        c
    }
}

/// `common_chdir<wchar_t>`.
unsafe fn common_chdir_wide(path: *const u16) -> c_int {
    if path.is_null() {
        validate_clear_osserr(false, EINVAL);
        return -1;
    }

    if SetCurrentDirectoryW(path) == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return -1;
    }

    // `__crt_internal_win32_buffer<Character>` starting at `MAX_PATH + 1` --
    // growing via `malloc` if that isn't enough (`grow_into`, shared with
    // `getcwd.rs`).
    let mut stack = [0u16; MAX_PATH + 1];
    let call = |buf: *mut u16, cap: u32| GetCurrentDirectoryW(cap, buf);
    match crate::misc::getcwd::grow_into(stack.as_mut_ptr(), MAX_PATH + 1, call) {
        Ok((ptr, _len, was_malloced)) => {
            let r = set_cwd_env_wide(ptr);
            if was_malloced {
                free(ptr.cast());
            }
            r
        }
        Err(_) => -1, // errno already set by `grow_into`
    }
}

/// `common_chdir<char>`.
unsafe fn common_chdir_narrow(path: *const c_char) -> c_int {
    if path.is_null() {
        validate_clear_osserr(false, EINVAL);
        return -1;
    }

    if __acrt_SetCurrentDirectoryA(path) == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return -1;
    }

    // Mirrors the wide path: fetch the (possibly expanded) new current
    // directory back, this time narrow via the wide API + codepage
    // conversion (`getcwd.rs`'s `wcs_to_mbs_alloc`/`file_codepage`), since
    // there is no exported, non-template narrow "get current directory"
    // helper to call directly.
    let mut stack = [0u16; MAX_PATH + 1];
    let call = |buf: *mut u16, cap: u32| GetCurrentDirectoryW(cap, buf);
    let wide = match crate::misc::getcwd::grow_into(stack.as_mut_ptr(), MAX_PATH + 1, call) {
        Ok((ptr, _len, was_malloced)) => {
            if !was_malloced {
                // Copy off the stack buffer before it goes out of scope --
                // `wcs_to_mbs_alloc` only needs to read it once, so borrow it
                // directly instead.
                let cp = crate::misc::getcwd::file_codepage();
                let r = match crate::misc::getcwd::wcs_to_mbs_alloc(ptr, cp) {
                    Ok((narrow, _n)) => {
                        let r = set_cwd_env_narrow(narrow);
                        free(narrow.cast());
                        r
                    }
                    Err(e) => {
                        *crate::misc::errno::_errno() = e;
                        -1
                    }
                };
                return r;
            }
            ptr
        }
        Err(_) => return -1,
    };

    let cp = crate::misc::getcwd::file_codepage();
    let r = match crate::misc::getcwd::wcs_to_mbs_alloc(wide, cp) {
        Ok((narrow, _n)) => {
            let r = set_cwd_env_narrow(narrow);
            free(narrow.cast());
            r
        }
        Err(e) => {
            *crate::misc::errno::_errno() = e;
            -1
        }
    };
    free(wide.cast());
    r
}

/// `_chdir`.
#[no_mangle]
pub unsafe extern "C" fn _chdir(path: *const c_char) -> c_int {
    common_chdir_narrow(path)
}

/// `_wchdir`.
#[no_mangle]
pub unsafe extern "C" fn _wchdir(path: *const u16) -> c_int {
    common_chdir_wide(path)
}
