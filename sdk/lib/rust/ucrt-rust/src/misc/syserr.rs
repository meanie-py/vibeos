//! `misc/syserr.cpp` -- the System Error List: `_sys_errlist`, `_sys_nerr`,
//! `_sys_posix_errlist`, `_sys_first_posix_error`/`_sys_last_posix_error`, and
//! their accessor functions `__sys_nerr()`/`__sys_errlist()`.
//!
//! `_get_sys_err_msg` (`corecrt_internal.h`, `__inline`) is not itself an
//! exported symbol -- it is a header-inline helper every other C file in this
//! directory calls by value, so it is ported here (not `#[no_mangle]`) and
//! re-exported to the sibling submodules that need it (`strerror.rs`,
//! `perror.rs`).
//!
//! ## Details that are silent if wrong
//!
//! * `_sys_errlist` has 44 entries (errno 0..=43); `_sys_nerr` is 43 (COUNT-1,
//!   i.e. the index of the last, "Unknown error" catch-all entry -- NOT the
//!   count of real messages).
//! * `_sys_posix_errlist` starts at errno 100 (`_sys_first_posix_error`) and
//!   its own last real entry is 140 (`EWOULDBLOCK`); index 141-100=41 is that
//!   table's own out-of-range catch-all, at `_sys_last_posix_error` (140).
//! * `_get_sys_err_msg(m)`'s ordering matters: it checks `m >
//!   _sys_last_posix_error` (i.e. above BOTH tables, including a POSIX errno
//!   past 140) OR `(m in the dead zone between _sys_nerr and
//!   _sys_first_posix_error, i.e. 44..=99)` FIRST and returns the catch-all --
//!   only then does it split on `m <= _sys_nerr` (narrow table) vs. else
//!   (POSIX table, offset by `_sys_first_posix_error`). A negative `errno`
//!   (cast to `size_t`) becomes a huge value and hits the first branch, same
//!   as the C's implicit `int` -> `size_t` conversion.

use core::ffi::{c_char, c_int};

/// `_sys_errlist[0..=43]`. `char const* const` in the C: an array of pointers
/// to string literals, not embedded byte arrays -- so the Rust mirrors the
/// pointer-array shape, one `*const c_char` per message, each pointing at a
/// NUL-terminated static string. This is NOT byte/relocation-compared against
/// the C's `.obj` (`tools/compare_coff_data.py`): the C emits an array of
/// *pointers* into `.rdata`, so the object's data section holds the pointer
/// table plus deduplicated/merged string bytes at whatever addresses the
/// linker chose -- there is no stable byte-for-byte target to diff against
/// (unlike a plain data table of fixed-width records). Verified instead by
/// STRING CONTENT: every message below is copied verbatim, in order, from
/// `misc/syserr.cpp`.
macro_rules! msgs {
    ($($s:literal),* $(,)?) => {
        SysErrTable([$(concat!($s, "\0").as_ptr().cast::<c_char>()),*])
    };
}

/// `#[repr(transparent)]` over `[*const c_char; N]` -- identical layout and
/// address to the bare array (so the exported symbol is byte-for-byte the
/// C's `char const* const foo[]`), but a named type `Sync` can be `unsafe
/// impl`'d for: raw-pointer arrays are not `Sync` by default, which a plain
/// (non-`mut`) `static` requires. Every pointer inside is `'static` (points
/// at the binary's read-only string-literal storage) and never mutated, so
/// the impl is sound.
#[repr(transparent)]
pub struct SysErrTable<const N: usize>([*const c_char; N]);
unsafe impl<const N: usize> Sync for SysErrTable<N> {}
impl<const N: usize> core::ops::Index<usize> for SysErrTable<N> {
    type Output = *const c_char;
    fn index(&self, i: usize) -> &*const c_char {
        &self.0[i]
    }
}

/// `_countof(_sys_errlist)` / `_countof(_sys_posix_errlist)` -- named so the
/// arithmetic below (`_sys_nerr`, `_sys_last_posix_error`) never has to read
/// a `static`'s value from another `static`'s initializer.
const NARROW_LEN: usize = 44;
const POSIX_LEN: usize = 42;

#[no_mangle]
pub static _sys_errlist: SysErrTable<NARROW_LEN> = msgs![
    "No error",
    "Operation not permitted",
    "No such file or directory",
    "No such process",
    "Interrupted function call",
    "Input/output error",
    "No such device or address",
    "Arg list too long",
    "Exec format error",
    "Bad file descriptor",
    "No child processes",
    "Resource temporarily unavailable",
    "Not enough space",
    "Permission denied",
    "Bad address",
    "Unknown error", // 15 ENOTBLK -- not POSIX
    "Resource device",
    "File exists",
    "Improper link",
    "No such device",
    "Not a directory",
    "Is a directory",
    "Invalid argument",
    "Too many open files in system",
    "Too many open files",
    "Inappropriate I/O control operation",
    "Unknown error", // 26 ETXTBSY -- not POSIX
    "File too large",
    "No space left on device",
    "Invalid seek",
    "Read-only file system",
    "Too many links",
    "Broken pipe",
    "Domain error",
    "Result too large",
    "Unknown error", // 35 EUCLEAN -- not POSIX
    "Resource deadlock avoided",
    "Unknown error", // 37 UNKNOWN
    "Filename too long",
    "No locks available",
    "Function not implemented",
    "Directory not empty",
    "Illegal byte sequence",
    "Unknown error", // 43
];

#[no_mangle]
pub static _sys_posix_errlist: SysErrTable<POSIX_LEN> = msgs![
    "address in use",                 // 100 EADDRINUSE
    "address not available",          // 101 EADDRNOTAVAIL
    "address family not supported",   // 102 EAFNOSUPPORT
    "connection already in progress", // 103 EALREADY
    "bad message",                    // 104 EBADMSG
    "operation canceled",             // 105 ECANCELED
    "connection aborted",             // 106 ECONNABORTED
    "connection refused",             // 107 ECONNREFUSED
    "connection reset",               // 108 ECONNRESET
    "destination address required",   // 109 EDESTADDRREQ
    "host unreachable",               // 110 EHOSTUNREACH
    "identifier removed",             // 111 EIDRM
    "operation in progress",          // 112 EINPROGRESS
    "already connected",              // 113 EISCONN
    "too many symbolic link levels",  // 114 ELOOP
    "message size",                   // 115 EMSGSIZE
    "network down",                   // 116 ENETDOWN
    "network reset",                  // 117 ENETRESET
    "network unreachable",            // 118 ENETUNREACH
    "no buffer space",                // 119 ENOBUFS
    "no message available",           // 120 ENODATA
    "no link",                        // 121 ENOLINK
    "no message",                     // 122 ENOMSG
    "no protocol option",             // 123 ENOPROTOOPT
    "no stream resources",            // 124 ENOSR
    "not a stream",                   // 125 ENOSTR
    "not connected",                  // 126 ENOTCONN
    "state not recoverable",          // 127 ENOTRECOVERABLE
    "not a socket",                   // 128 ENOTSOCK
    "not supported",                  // 129 ENOTSUP
    "operation not supported",        // 130 EOPNOTSUPP
    "Unknown error",                  // 131 EOTHER
    "value too large",                // 132 EOVERFLOW
    "owner dead",                     // 133 EOWNERDEAD
    "protocol error",                 // 134 EPROTO
    "protocol not supported",         // 135 EPROTONOSUPPORT
    "wrong protocol type",            // 136 EPROTOTYPE
    "stream timeout",                 // 137 ETIME
    "timed out",                      // 138 ETIMEDOUT
    "text file busy",                 // 139 ETXTBSY
    "operation would block",          // 140 EWOULDBLOCK
    "Unknown error",                  // 141
];

/// `_sys_first_posix_error`.
#[no_mangle]
pub static _sys_first_posix_error: usize = 100;
/// `_sys_last_posix_error` = `_sys_first_posix_error + _countof(_sys_posix_errlist) - 1`
/// = 100 + 42 - 1 = 141.
#[no_mangle]
pub static _sys_last_posix_error: usize = 100 + POSIX_LEN - 1;
/// `_sys_nerr` = `_countof(_sys_errlist) - 1` = 43.
#[no_mangle]
pub static _sys_nerr: c_int = NARROW_LEN as c_int - 1;

/// `__sys_nerr()`.
#[no_mangle]
pub unsafe extern "C" fn __sys_nerr() -> *mut c_int {
    core::ptr::addr_of!(_sys_nerr).cast_mut()
}

/// `__sys_errlist()`.
#[no_mangle]
pub unsafe extern "C" fn __sys_errlist() -> *mut *mut c_char {
    core::ptr::addr_of!(_sys_errlist).cast::<*mut c_char>().cast_mut()
}

/// `_get_sys_err_msg` (`corecrt_internal.h:132`, `__inline`, not itself
/// exported). `m` is the C's `size_t` -- callers pass `errno` (an `int`,
/// possibly negative), and the sign-extending `as usize` cast at call sites
/// reproduces the C's implicit `(size_t)` conversion of a negative `int`
/// exactly: a huge value that fails `m <= _sys_last_posix_error` and falls
/// into the catch-all.
pub(crate) unsafe fn get_sys_err_msg(m: usize) -> *const c_char {
    if m > _sys_last_posix_error || ((_sys_nerr as usize) < m && m < _sys_first_posix_error) {
        return _sys_errlist[_sys_nerr as usize];
    }
    if m <= _sys_nerr as usize {
        _sys_errlist[m]
    } else {
        _sys_posix_errlist[m - _sys_first_posix_error]
    }
}
