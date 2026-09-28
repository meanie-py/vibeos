//! `misc/perror.cpp` (`perror`) and `misc/wperror.cpp` (`_wperror`): print
//! `errno`'s system message to stderr (fd 2), through the lowio layer
//! directly (`_write_nolock`), not through stdio's buffered `FILE*`.
//!
//! ## Details that are silent if wrong
//!
//! * Both write straight to lowio handle 2 (`_write_nolock`), locked with
//!   `__acrt_lowio_lock_fh`/`__acrt_lowio_unlock_fh` around the whole
//!   sequence -- three separate writes (`"prefix"`, `": "`, message, `"\n"`
//!   as up to four), NOT one buffered `fprintf`. A concurrent writer to fd 2
//!   from another thread can interleave with a `perror` call from stdio-level
//!   functions, but never mid-write with another `perror`/`_wperror` (same
//!   lock).
//! * The system message comes from `ptd.get_raw_ptd()->_terrno` -- the
//!   AMBIENT thread errno at call time, read directly off the ptd, NOT
//!   through the `PtdHost`'s cache (nothing in this call path ever sets the
//!   cache, so the cache is never consulted; this matters if that ever
//!   changes).
//! * `_wperror` with a NULL or empty prefix calls `perror(nullptr)` directly
//!   -- it does NOT go through its own conversion path at all in that case.
//! * `_wperror` with a real prefix converts it narrow via `wcstombs_s`
//!   first (using `_ERRCHECK_EINVAL_ERANGE`, so EINVAL/ERANGE from the
//!   conversion are fatal (Watson), not just a quiet failure) and passes the
//!   result to `perror` -- so `_wperror`'s errno message is subject to the
//!   SAME narrow system-error table as `perror`; the wide message table is
//!   never consulted here (`perror`, not `_wcserror`, does the lookup).
//! * `_wperror` silently returns (does nothing, no output at all) if
//!   `required_count == 0` after the sizing call, or if the allocation or
//!   the second conversion call fails -- none of those paths call `perror`.

use core::ffi::{c_char, c_int, c_void};
use core::ptr::null_mut;

use crate::ptd_host::PtdHost;

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;

extern "C" {
    fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: u32, r: usize) -> !;
    fn __acrt_lowio_lock_fh(fh: c_int);
    fn __acrt_lowio_unlock_fh(fh: c_int);
    /// `int __cdecl _write_nolock(int, void const*, unsigned, __crt_cached_ptd_host&)`
    /// -- the C++ reference becomes a pointer to the layout-identical
    /// [`PtdHost`].
    fn _write_nolock(fh: c_int, buf: *const c_void, max_count: u32, ptd: *mut PtdHost) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    /// `errno_t wcstombs_s(size_t*, char*, size_t, wchar_t const*, size_t)`.
    /// Not declared elsewhere in the crate yet -- no clash to match.
    fn wcstombs_s(
        converted: *mut usize,
        dst: *mut c_char,
        size: usize,
        src: *const u16,
        count: usize,
    ) -> c_int;
}

unsafe fn errcheck_einval_erange(e: c_int) -> c_int {
    if e == 0 || (e != EINVAL && e != ERANGE) {
        e
    } else {
        _invoke_watson(null_mut(), null_mut(), null_mut(), 0, 0);
    }
}

/// `_perror_internal`.
unsafe fn perror_internal(user_prefix: *const c_char, ptd: &mut PtdHost) {
    const FH: c_int = 2;
    __acrt_lowio_lock_fh(FH);

    if !user_prefix.is_null() && *user_prefix != 0 {
        let n = strlen(user_prefix);
        _write_nolock(FH, user_prefix.cast(), n as u32, ptd);
        _write_nolock(FH, b": \0".as_ptr().cast(), 2, ptd);
    }

    let system_message = crate::misc::syserr::get_sys_err_msg((*ptd.raw_ptd()).terrno as usize);
    let n = strlen(system_message);
    _write_nolock(FH, system_message.cast(), n as u32, ptd);
    _write_nolock(FH, b"\n\0".as_ptr().cast(), 1, ptd);

    __acrt_lowio_unlock_fh(FH);
}

/// `perror`.
#[no_mangle]
pub unsafe extern "C" fn perror(user_prefix: *const c_char) {
    let mut ptd = PtdHost::new(null_mut());
    perror_internal(user_prefix, &mut ptd);
}

/// `_wperror`.
#[no_mangle]
pub unsafe extern "C" fn _wperror(wide_user_prefix: *const u16) {
    if wide_user_prefix.is_null() || *wide_user_prefix == 0 {
        return perror(null_mut());
    }

    let mut required_count: usize = 0;
    errcheck_einval_erange(wcstombs_s(
        &mut required_count,
        null_mut(),
        0,
        wide_user_prefix,
        0x7FFF_FFFF, // INT_MAX, exactly as the C passes -- ignored anyway since dst is NULL
    ));
    if required_count == 0 {
        return;
    }

    let narrow = calloc(required_count, 1) as *mut c_char;
    if narrow.is_null() {
        return;
    }

    let conversion_result = errcheck_einval_erange(wcstombs_s(
        null_mut(),
        narrow,
        required_count,
        wide_user_prefix,
        usize::MAX, // _TRUNCATE
    ));

    if conversion_result != 0 {
        free(narrow.cast());
        return;
    }

    perror(narrow);
    free(narrow.cast());
}
