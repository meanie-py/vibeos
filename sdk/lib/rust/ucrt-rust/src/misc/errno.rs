//! `misc/errno.cpp` -- `_errno`, `__doserrno`, `_set_errno`/`_get_errno`,
//! `_set_doserrno`/`_get_doserrno`, and the OS-error <-> errno mapping table
//! (`__acrt_errno_map_os_error`/`_ptd`, `__acrt_errno_from_os_error`).
//!
//! `__acrt_errno_from_os_error` in particular is load-bearing far outside this
//! file: nearly every other UCRT source that wraps a failing Win32 call reaches
//! it (directly or via `__acrt_errno_map_os_error`) to turn `GetLastError()`
//! into an errno value.
//!
//! ## Details that are silent if wrong
//!
//! * The table is searched linearly, first match wins; codes are NOT sorted by
//!   value in the C, so a binary search would silently disagree on any future
//!   edit (there is exactly one now, so it doesn't matter yet, but the C's
//!   *shape* -- linear scan -- is preserved).
//! * Unmatched codes fall through to two range checks (EACCES range 19..=36,
//!   ENOEXEC range 188..=202) before defaulting to EINVAL -- not the other way
//!   around, and a code inside a range is not also checked against the table
//!   (it can't be, since the table is checked first and the ranges start
//!   above/around table entries but don't overlap any of the table's own
//!   codes: verified none of the 44 table codes not already itself falls in
//!   either the ERROR_WRITE_PROTECT(19)..ERROR_SHARING_BUFFER_EXCEEDED(36) or
//!   ERROR_INVALID_STARTING_CODESEG(188)..ERROR_INFLOOP_IN_RELOC_CHAIN(202)
//!   spans except where the table itself maps them, e.g. 19/33/36 explicitly
//!   listed to EBUSY-family/EACCES/ENOMEM take priority over the range).
//! * `_get_errno`/`_get_doserrno` are "guaranteed to give some answer": they
//!   read straight from `errno`/`_doserrno` (i.e. `_errno()`/`__doserrno()`,
//!   which themselves never fail -- see below), not through the fallible
//!   `__acrt_getptd_noexit` path that `_set_*` uses.
//! * `_set_errno`/`_set_doserrno` DO go through `__acrt_getptd_noexit` and fail
//!   with `ENOMEM` (not touching the value) if no ptd is available -- an
//!   asymmetry with the getters that is transcribed exactly.
//! * `_errno`/`__doserrno` never fail: with no ptd, they return the address of
//!   a static fallback (`ENOMEM` / `ERROR_NOT_ENOUGH_MEMORY` respectively),
//!   NOT null and NOT the real thread's storage -- so writing through the
//!   returned pointer in that (essentially unreachable in practice) case is
//!   silently discarded on the next call.

use core::ffi::{c_int, c_long, c_ulong};

use crate::ptd::AcrtPtd;
use crate::ptd_host::PtdHost;

const ENOMEM: c_int = 12;
const EINVAL: c_int = 22;

extern "C" {
    /// `sdk/lib/ucrt/internal/per_thread_data.cpp` (still C).
    fn __acrt_getptd_noexit() -> *mut AcrtPtd;
    fn _invalid_parameter_noinfo();
}

/// `errentry` (`oscode`, `errnocode`), transcribed verbatim including its
/// comments' original error numbers, for cross-reference against the C.
struct ErrEntry {
    oscode: u32,
    errnocode: c_int,
}

macro_rules! e {
    ($os:expr, $errno:expr) => {
        ErrEntry { oscode: $os, errnocode: $errno }
    };
}

// EACCES=13, EAGAIN=11, EBADF=9, E2BIG=7, EEXIST=17, EILSEQ=42, EINVAL=22,
// ENOENT=2, ENOEXEC=8, ENOMEM=12, ENOTEMPTY=41, EPIPE=32, ESPIPE unused,
// ECHILD=10, EXDEV=18, ENOSPC=28 -- the CRT's own numbering (`errno.h`), not
// the OS codes on the left.
static ERRTABLE: &[ErrEntry] = &[
    e!(1, 22),    // ERROR_INVALID_FUNCTION       -> EINVAL
    e!(2, 2),     // ERROR_FILE_NOT_FOUND         -> ENOENT
    e!(3, 2),     // ERROR_PATH_NOT_FOUND         -> ENOENT
    e!(4, 24),    // ERROR_TOO_MANY_OPEN_FILES    -> EMFILE
    e!(5, 13),    // ERROR_ACCESS_DENIED          -> EACCES
    e!(6, 9),     // ERROR_INVALID_HANDLE         -> EBADF
    e!(7, 12),    // ERROR_ARENA_TRASHED          -> ENOMEM
    e!(8, 12),    // ERROR_NOT_ENOUGH_MEMORY      -> ENOMEM
    e!(9, 12),    // ERROR_INVALID_BLOCK          -> ENOMEM
    e!(10, 7),    // ERROR_BAD_ENVIRONMENT        -> E2BIG
    e!(11, 8),    // ERROR_BAD_FORMAT             -> ENOEXEC
    e!(12, 22),   // ERROR_INVALID_ACCESS         -> EINVAL
    e!(13, 22),   // ERROR_INVALID_DATA           -> EINVAL
    e!(15, 2),    // ERROR_INVALID_DRIVE          -> ENOENT
    e!(16, 13),   // ERROR_CURRENT_DIRECTORY      -> EACCES
    e!(17, 18),   // ERROR_NOT_SAME_DEVICE        -> EXDEV
    e!(18, 2),    // ERROR_NO_MORE_FILES          -> ENOENT
    e!(33, 13),   // ERROR_LOCK_VIOLATION         -> EACCES
    e!(53, 2),    // ERROR_BAD_NETPATH            -> ENOENT
    e!(65, 13),   // ERROR_NETWORK_ACCESS_DENIED  -> EACCES
    e!(67, 2),    // ERROR_BAD_NET_NAME           -> ENOENT
    e!(80, 17),   // ERROR_FILE_EXISTS            -> EEXIST
    e!(82, 13),   // ERROR_CANNOT_MAKE            -> EACCES
    e!(83, 13),   // ERROR_FAIL_I24               -> EACCES
    e!(87, 22),   // ERROR_INVALID_PARAMETER      -> EINVAL
    e!(89, 11),   // ERROR_NO_PROC_SLOTS          -> EAGAIN
    e!(108, 13),  // ERROR_DRIVE_LOCKED           -> EACCES
    e!(109, 32),  // ERROR_BROKEN_PIPE            -> EPIPE
    e!(112, 28),  // ERROR_DISK_FULL              -> ENOSPC
    e!(114, 9),   // ERROR_INVALID_TARGET_HANDLE  -> EBADF
    e!(128, 10),  // ERROR_WAIT_NO_CHILDREN       -> ECHILD
    e!(129, 10),  // ERROR_CHILD_NOT_COMPLETE     -> ECHILD
    e!(130, 9),   // ERROR_DIRECT_ACCESS_HANDLE   -> EBADF
    e!(131, 22),  // ERROR_NEGATIVE_SEEK          -> EINVAL
    e!(132, 13),  // ERROR_SEEK_ON_DEVICE         -> EACCES
    e!(145, 41),  // ERROR_DIR_NOT_EMPTY          -> ENOTEMPTY
    e!(158, 13),  // ERROR_NOT_LOCKED             -> EACCES
    e!(161, 2),   // ERROR_BAD_PATHNAME           -> ENOENT
    e!(164, 11),  // ERROR_MAX_THRDS_REACHED      -> EAGAIN
    e!(167, 13),  // ERROR_LOCK_FAILED            -> EACCES
    e!(183, 17),  // ERROR_ALREADY_EXISTS         -> EEXIST
    e!(206, 2),   // ERROR_FILENAME_EXCED_RANGE   -> ENOENT
    e!(215, 11),  // ERROR_NESTING_NOT_ALLOWED    -> EAGAIN
    e!(1113, 42), // ERROR_NO_UNICODE_TRANSLATION -> EILSEQ
    e!(1816, 12), // ERROR_NOT_ENOUGH_QUOTA       -> ENOMEM
];

/// `MIN_EACCES_RANGE`/`MAX_EACCES_RANGE`: `ERROR_WRITE_PROTECT`(19)..=
/// `ERROR_SHARING_BUFFER_EXCEEDED`(36).
const MIN_EACCES_RANGE: u32 = 19;
const MAX_EACCES_RANGE: u32 = 36;
/// `MIN_EXEC_ERROR`/`MAX_EXEC_ERROR`: `ERROR_INVALID_STARTING_CODESEG`(188)..=
/// `ERROR_INFLOOP_IN_RELOC_CHAIN`(202).
const MIN_EXEC_ERROR: u32 = 188;
const MAX_EXEC_ERROR: u32 = 202;

/// `__acrt_errno_from_os_error`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_errno_from_os_error(oserrno: c_ulong) -> c_int {
    for entry in ERRTABLE {
        if oserrno == entry.oscode {
            return entry.errnocode;
        }
    }

    if oserrno >= MIN_EACCES_RANGE && oserrno <= MAX_EACCES_RANGE {
        13 // EACCES
    } else if oserrno >= MIN_EXEC_ERROR && oserrno <= MAX_EXEC_ERROR {
        8 // ENOEXEC
    } else {
        EINVAL
    }
}

/// `__acrt_errno_map_os_error`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_errno_map_os_error(oserrno: c_ulong) {
    *__doserrno() = oserrno;
    *_errno() = __acrt_errno_from_os_error(oserrno);
}

/// `__acrt_errno_map_os_error_ptd`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_errno_map_os_error_ptd(oserrno: c_ulong, ptd: *mut PtdHost) {
    (*ptd).errno().set(__acrt_errno_from_os_error(oserrno));
    // `ptd.get_doserrno().set(oserrno)`.
    (*ptd).doserrno().set(oserrno);
}


/// `_set_errno`.
#[no_mangle]
pub unsafe extern "C" fn _set_errno(value: c_int) -> c_int {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return ENOMEM;
    }
    (*ptd).terrno = value;
    0
}

/// `_get_errno`. `_VALIDATE_RETURN_NOERRNO(result != nullptr, EINVAL)`: errno
/// is untouched and no handler runs on a NULL `result` -- just EINVAL back.
#[no_mangle]
pub unsafe extern "C" fn _get_errno(result: *mut c_int) -> c_int {
    if result.is_null() {
        return EINVAL;
    }
    *result = *_errno();
    0
}

/// `_set_doserrno`.
#[no_mangle]
pub unsafe extern "C" fn _set_doserrno(value: c_ulong) -> c_int {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return ENOMEM;
    }
    (*ptd).tdoserrno = value as c_long;
    0
}

/// `_get_doserrno`.
#[no_mangle]
pub unsafe extern "C" fn _get_doserrno(result: *mut c_ulong) -> c_int {
    if result.is_null() {
        // `_VALIDATE_RETURN_NOERRNO`: the handler, but errno untouched.
        _invalid_parameter_noinfo();
        return EINVAL;
    }
    *result = *__doserrno();
    0
}

/// Fallback storage for `_errno`/`__doserrno` when no ptd is available --
/// same values the C's file-scope statics use (`ENOMEM`,
/// `ERROR_NOT_ENOUGH_MEMORY`).
static mut ERRNO_NO_MEMORY: c_int = ENOMEM;
static mut DOSERRNO_NO_MEMORY: c_ulong = 8; // ERROR_NOT_ENOUGH_MEMORY

/// `_errno`.
#[no_mangle]
pub unsafe extern "C" fn _errno() -> *mut c_int {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return core::ptr::addr_of_mut!(ERRNO_NO_MEMORY);
    }
    core::ptr::addr_of_mut!((*ptd).terrno)
}

/// `__doserrno`.
#[no_mangle]
pub unsafe extern "C" fn __doserrno() -> *mut c_ulong {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return core::ptr::addr_of_mut!(DOSERRNO_NO_MEMORY);
    }
    // `tdoserrno` is `c_long` (`corecrt_internal.h`'s field is `long`, matching
    // `ptd.rs`); `_doserrno`'s public type is `unsigned long`. Same bit
    // pattern, reinterpreted, exactly as the C's implicit signed/unsigned
    // `long`/`DWORD` aliasing at this one field.
    core::ptr::addr_of_mut!((*ptd).tdoserrno).cast::<c_ulong>()
}
