//! `sdk/lib/ucrt/env/` in full: the CRT's two environment tables and every
//! function over them. Replaces all seven C files:
//!
//! | C file | here |
//! |---|---|
//! | `environment_initialization.cpp` | [`init`] -- the four tables, `__p__environ`/`__p__wenviron`, `_initialize_*_environment`, `_get_initial_*_environment`, `__dcrt_get_or_create_*_environment_nolock`, `__dcrt_uninitialize_environments_nolock` |
//! | `get_environment_from_os.cpp` | [`from_os`] -- `__dcrt_get_{narrow,wide}_environment_from_os` |
//! | `setenv.cpp` | [`setenv`] -- `__dcrt_set_variable_in_{narrow,wide}_environment_nolock` |
//! | `getenv.cpp` | [`getenv`] -- `getenv`/`_wgetenv`, `getenv_s`/`_wgetenv_s`, `_dupenv_s`/`_wdupenv_s` |
//! | `putenv.cpp` | [`putenv`] -- `_putenv`/`_wputenv`, `_putenv_s`/`_wputenv_s` |
//! | `searchenv.cpp` | [`searchenv`] -- `_searchenv(_s)`/`_wsearchenv(_s)` |
//! | `getpath.cpp` | [`getpath`] -- `__acrt_getpath`/`__acrt_wgetpath` |
//!
//! Each C file is a template over `Character` instantiated for `char` and
//! `wchar_t`; here that is ONE generic body over [`EnvChar`], whose two impls
//! play `__crt_char_traits<Character>`: every traits call the C makes by
//! mapped name (`traits::set_variable_in_environment_nolock` ->
//! `__dcrt_set_variable_in_narrow_environment_nolock`, ...) is routed to the
//! function with that exact C name, so the call graph is the C's.
//!
//! ## Names other code reads
//!
//! State isolation is compiled out (`state_index_count == 1`, see timeset.rs),
//! so each `dual_state_global<T**>` is exactly one pointer. `_environ_table`,
//! `_wenviron_table`, `__dcrt_initial_narrow_environment` and
//! `__dcrt_initial_wide_environment` keep their C names and types:
//! `__dcrt_initial_narrow_environment` is itself a ucrtbase export (as data),
//! and `internal/initialization.cpp` (still C, or its Rust port) calls
//! `_initialize_narrow_environment`, `_get_initial_narrow_environment` and
//! `__dcrt_uninitialize_environments_nolock`. `exec.rs` calls
//! `__dcrt_get_{narrow,wide}_environment_from_os`, `__acrt_{w,}getpath` and
//! `_{w,}dupenv_s`; `time/tzset.rs` calls `_wgetenv_s`; the stdio temp-name
//! family calls `_{w,}dupenv_s`.
//!
//! ## Details that are silent if wrong
//!
//! * **The `=C:` per-drive variables never enter the CRT tables.**
//!   `create_environment` skips every OS string starting with `=`, and so does
//!   the count it sizes the array with. `_putenv("=C:=x")` is refused
//!   (`equal_sign == option`, EINVAL, no handler).
//! * **The initial environment is never modified in place.** The first
//!   modification clones it (`copy_environment`, which `abort()`s -- not
//!   fails -- on allocation failure), because that array was handed to
//!   `main`.
//! * **The other-width table is created lazily**, and only when the other one
//!   exists: `_wgetenv` in a program that has only the narrow table re-reads
//!   the OS block (NOT the narrow table) to build the wide one; cloning the
//!   narrow table through `CP_ACP` is only the fallback when that fails.
//! * **`_putenv("X=")` deletes**, from the CRT table AND the OS (a NULL value
//!   to `SetEnvironmentVariable`), but a removal of a name that is not there
//!   returns 0 WITHOUT touching the OS.
//! * **A name containing `=`**: `_putenv_s("A=B", "c")` builds `"A=B=c"`, and
//!   the FIRST `=` splits it, so it sets `A` to `"B=c"`.
//! * **`common_putenv_nolock` returns -1 with errno untouched when neither
//!   table exists** -- before it validates `name`, so `_putenv(NULL)` then
//!   does not even reach the invalid-parameter handler.
//! * **Removing a variable shrinks with `_recalloc(env, remaining)`**, and a
//!   failed (NULL) result keeps the old block. Transcribed as is, including
//!   the case where `remaining` is 0.
//! * **`getenv` matching is `_strnicoll`, not `_strnicmp`**: locale
//!   collation, case-insensitive, over exactly the name's length, then the
//!   entry's next character must be `=`.
//! * **`_searchenv` sizes its second-chance buffer with `path_count -
//!   file_name_length` (no `- 1`)**, one more than the first try; kept.
//! * **`_searchenv_s` with a NULL `environment_variable` is a Watson
//!   termination**, not an error return: `_ERRCHECK_EINVAL` over `_dupenv_s`.

#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_void};

pub mod from_os;
pub mod getenv;
pub mod getpath;
pub mod init;
pub mod putenv;
pub mod searchenv;
pub mod setenv;

// ── constants ────────────────────────────────────────────────────────────────

pub(crate) const EINVAL: c_int = 22;
pub(crate) const ENOENT: c_int = 2;
pub(crate) const ENOMEM: c_int = 12;
pub(crate) const ERANGE: c_int = 34;
pub(crate) const EILSEQ: c_int = 42;

/// `_MAX_ENV` (stdlib.h).
pub(crate) const MAX_ENV: usize = 32767;
/// `_MAX_PATH`.
pub(crate) const MAX_PATH: usize = 260;
/// `CP_ACP`.
pub(crate) const CP_ACP: u32 = 0;

/// `__acrt_lock_id::__acrt_environment_lock` (corecrt_internal.h: the twelfth
/// enumerator -- heap, debug, exit, signal, locale, multibyte_cp, time,
/// lowio_index, stdio_index, conio, popen, environment).
pub(crate) const ACRT_ENVIRONMENT_LOCK: c_int = 11;

// ── externs ──────────────────────────────────────────────────────────────────

extern "C" {
    pub(crate) fn _errno() -> *mut c_int;
    pub(crate) fn _invalid_parameter_noinfo();
    pub(crate) fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: u32, r: usize) -> !;
    pub(crate) fn abort() -> !;
    pub(crate) fn __acrt_lock(lock: c_int);
    pub(crate) fn __acrt_unlock(lock: c_int);

    pub(crate) fn malloc(size: usize) -> *mut c_void;
    pub(crate) fn calloc(count: usize, size: usize) -> *mut c_void;
    pub(crate) fn free(p: *mut c_void);
    pub(crate) fn _recalloc(block: *mut c_void, count: usize, size: usize) -> *mut c_void;

    pub(crate) fn strlen(s: *const c_char) -> usize;
    pub(crate) fn wcslen(s: *const u16) -> usize;
    pub(crate) fn strnlen(s: *const c_char, n: usize) -> usize;
    pub(crate) fn wcsnlen(s: *const u16, n: usize) -> usize;
    pub(crate) fn strchr(s: *const u8, c: c_int) -> *mut u8;
    pub(crate) fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub(crate) fn strcpy_s(dst: *mut u8, destsz: usize, src: *const u8) -> c_int;
    pub(crate) fn wcscpy_s(dst: *mut u16, destsz: usize, src: *const u16) -> c_int;
    pub(crate) fn _strnicoll(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    pub(crate) fn _wcsnicoll(a: *const u16, b: *const u16, n: usize) -> c_int;

    pub(crate) fn _access_s(path: *const u8, mode: c_int) -> c_int;
    pub(crate) fn _waccess_s(path: *const u16, mode: c_int) -> c_int;
    pub(crate) fn _fullpath(buf: *mut c_char, path: *const c_char, size: usize) -> *mut c_char;
    pub(crate) fn _wfullpath(buf: *mut u16, path: *const u16, size: usize) -> *mut u16;

    pub(crate) fn __acrt_SetEnvironmentVariableA(name: *const c_char, value: *const c_char) -> c_int;
    pub(crate) fn __acrt_MultiByteToWideChar(
        cp: u32,
        flags: u32,
        src: *const c_char,
        src_len: c_int,
        dst: *mut u16,
        dst_len: c_int,
    ) -> c_int;
    pub(crate) fn __acrt_WideCharToMultiByte(
        cp: u32,
        flags: u32,
        src: *const u16,
        src_len: c_int,
        dst: *mut c_char,
        dst_len: c_int,
        default_char: *const c_char,
        used_default: *mut c_int,
    ) -> c_int;
}

pub(crate) use crate::locale_abi::__acrt_initialize_multibyte;

extern "system" {
    pub(crate) fn SetEnvironmentVariableW(name: *const u16, value: *const u16) -> c_int;
    pub(crate) fn GetEnvironmentStringsW() -> *mut u16;
    pub(crate) fn FreeEnvironmentStringsW(block: *mut u16) -> c_int;
}

// ── small shared helpers ─────────────────────────────────────────────────────

#[inline]
pub(crate) unsafe fn set_errno(e: c_int) {
    *_errno() = e;
}

#[inline]
pub(crate) unsafe fn get_errno() -> c_int {
    *_errno()
}

/// `_VALIDATE_RETURN*`'s failure arm: errno, then the handler.
#[cold]
pub(crate) unsafe fn invalid(e: c_int) {
    set_errno(e);
    _invalid_parameter_noinfo();
}

/// `_ERRCHECK(e)`: Watson on any nonzero errno_t.
#[inline]
pub(crate) unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }
}

/// `_ERRCHECK_EINVAL(e)`: Watson only on EINVAL, otherwise `e` passes through.
#[inline]
pub(crate) unsafe fn errcheck_einval(e: c_int) -> c_int {
    if e == EINVAL {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }
    e
}

/// `__crt_unique_heap_ptr<T>`: `free` at scope exit unless detached.
pub(crate) struct HeapPtr<T>(pub(crate) *mut T);

impl<T> HeapPtr<T> {
    pub(crate) fn get(&self) -> *mut T {
        self.0
    }
    pub(crate) fn is_null(&self) -> bool {
        self.0.is_null()
    }
    pub(crate) fn detach(&mut self) -> *mut T {
        core::mem::replace(&mut self.0, core::ptr::null_mut())
    }
}

impl<T> Drop for HeapPtr<T> {
    fn drop(&mut self) {
        unsafe { free(self.0.cast()) }
    }
}

/// `_calloc_crt_t(T, n)`.
#[inline]
pub(crate) unsafe fn calloc_t<T>(n: usize) -> *mut T {
    calloc(n, core::mem::size_of::<T>()).cast()
}

// ── the character trait (`__crt_char_traits<Character>`) ─────────────────────

pub(crate) trait EnvChar: Copy + PartialEq + 'static {
    type Other: EnvChar;
    const WIDE: bool;

    fn ch(c: u8) -> Self;
    fn is(self, c: u8) -> bool {
        self == Self::ch(c)
    }

    /// `get_environment_nolock(Character())`: `&_environ_table.value()`.
    fn table() -> *mut *mut *mut Self;
    /// `get_initial_environment(Character())`.
    fn initial() -> *mut *mut *mut Self;

    unsafe fn tcslen(s: *const Self) -> usize;
    unsafe fn tcsnlen(s: *const Self, max: usize) -> usize;
    unsafe fn tcschr(s: *const Self, c: u8) -> *mut Self;
    unsafe fn tcscpy_s(dst: *mut Self, n: usize, src: *const Self) -> c_int;
    unsafe fn tcsnicoll(a: *const Self, b: *const Self, n: usize) -> c_int;
    unsafe fn taccess_s(path: *const Self, mode: c_int) -> c_int;
    unsafe fn tfullpath(buf: *mut Self, path: *const Self, n: usize) -> *mut Self;
    unsafe fn tgetpath(paths: *const Self, buf: *mut Self, n: usize) -> *mut Self;
    unsafe fn tdupenv_s_crt(buf: *mut *mut Self, count: *mut usize, name: *const Self) -> c_int;
    unsafe fn set_environment_variable(name: *const Self, value: *const Self) -> c_int;
    unsafe fn get_environment_from_os() -> *mut Self;
    unsafe fn get_or_create_environment_nolock() -> *mut *mut Self;
    unsafe fn set_variable_in_environment_nolock(option: *mut Self, is_top_level_call: c_int) -> c_int;
    /// `pre_initialize(Character())`.
    unsafe fn pre_initialize();

    /// `__crt_compute_required_transform_buffer_count(CP_ACP, s)`: the size,
    /// in `Other` units and including the terminator, of `s` converted.
    unsafe fn required_transform_count(s: *const Self) -> usize;
    /// `__crt_transform_string(CP_ACP, s, buffer, count)`.
    unsafe fn transform_string(s: *const Self, buffer: *mut Self::Other, count: usize) -> usize;
}

impl EnvChar for u8 {
    type Other = u16;
    const WIDE: bool = false;

    fn ch(c: u8) -> u8 {
        c
    }
    fn table() -> *mut *mut *mut u8 {
        core::ptr::addr_of_mut!(init::_environ_table)
    }
    fn initial() -> *mut *mut *mut u8 {
        core::ptr::addr_of_mut!(init::__dcrt_initial_narrow_environment)
    }
    unsafe fn tcslen(s: *const u8) -> usize {
        strlen(s.cast())
    }
    unsafe fn tcsnlen(s: *const u8, max: usize) -> usize {
        strnlen(s.cast(), max)
    }
    unsafe fn tcschr(s: *const u8, c: u8) -> *mut u8 {
        strchr(s, c as c_int)
    }
    unsafe fn tcscpy_s(dst: *mut u8, n: usize, src: *const u8) -> c_int {
        strcpy_s(dst, n, src)
    }
    unsafe fn tcsnicoll(a: *const u8, b: *const u8, n: usize) -> c_int {
        _strnicoll(a.cast(), b.cast(), n)
    }
    unsafe fn taccess_s(path: *const u8, mode: c_int) -> c_int {
        _access_s(path, mode)
    }
    unsafe fn tfullpath(buf: *mut u8, path: *const u8, n: usize) -> *mut u8 {
        _fullpath(buf.cast(), path.cast(), n).cast()
    }
    unsafe fn tgetpath(paths: *const u8, buf: *mut u8, n: usize) -> *mut u8 {
        getpath::__acrt_getpath(paths, buf, n)
    }
    unsafe fn tdupenv_s_crt(buf: *mut *mut u8, count: *mut usize, name: *const u8) -> c_int {
        getenv::_dupenv_s(buf, count, name)
    }
    unsafe fn set_environment_variable(name: *const u8, value: *const u8) -> c_int {
        __acrt_SetEnvironmentVariableA(name.cast(), value.cast())
    }
    unsafe fn get_environment_from_os() -> *mut u8 {
        from_os::__dcrt_get_narrow_environment_from_os()
    }
    unsafe fn get_or_create_environment_nolock() -> *mut *mut u8 {
        init::__dcrt_get_or_create_narrow_environment_nolock()
    }
    unsafe fn set_variable_in_environment_nolock(option: *mut u8, is_top_level_call: c_int) -> c_int {
        setenv::__dcrt_set_variable_in_narrow_environment_nolock(option, is_top_level_call)
    }
    unsafe fn pre_initialize() {
        // The result is discarded, as in the C.
        let _ = __acrt_initialize_multibyte();
    }
    unsafe fn required_transform_count(s: *const u8) -> usize {
        __acrt_MultiByteToWideChar(CP_ACP, 0, s.cast(), -1, core::ptr::null_mut(), 0) as usize
    }
    unsafe fn transform_string(s: *const u8, buffer: *mut u16, count: usize) -> usize {
        __acrt_MultiByteToWideChar(CP_ACP, 0, s.cast(), -1, buffer, count as c_int) as usize
    }
}

impl EnvChar for u16 {
    type Other = u8;
    const WIDE: bool = true;

    fn ch(c: u8) -> u16 {
        c as u16
    }
    fn table() -> *mut *mut *mut u16 {
        core::ptr::addr_of_mut!(init::_wenviron_table)
    }
    fn initial() -> *mut *mut *mut u16 {
        core::ptr::addr_of_mut!(init::__dcrt_initial_wide_environment)
    }
    unsafe fn tcslen(s: *const u16) -> usize {
        wcslen(s)
    }
    unsafe fn tcsnlen(s: *const u16, max: usize) -> usize {
        wcsnlen(s, max)
    }
    unsafe fn tcschr(s: *const u16, c: u8) -> *mut u16 {
        wcschr(s, c as u16)
    }
    unsafe fn tcscpy_s(dst: *mut u16, n: usize, src: *const u16) -> c_int {
        wcscpy_s(dst, n, src)
    }
    unsafe fn tcsnicoll(a: *const u16, b: *const u16, n: usize) -> c_int {
        _wcsnicoll(a, b, n)
    }
    unsafe fn taccess_s(path: *const u16, mode: c_int) -> c_int {
        _waccess_s(path, mode)
    }
    unsafe fn tfullpath(buf: *mut u16, path: *const u16, n: usize) -> *mut u16 {
        _wfullpath(buf, path, n)
    }
    unsafe fn tgetpath(paths: *const u16, buf: *mut u16, n: usize) -> *mut u16 {
        getpath::__acrt_wgetpath(paths, buf, n)
    }
    unsafe fn tdupenv_s_crt(buf: *mut *mut u16, count: *mut usize, name: *const u16) -> c_int {
        getenv::_wdupenv_s(buf, count, name)
    }
    unsafe fn set_environment_variable(name: *const u16, value: *const u16) -> c_int {
        SetEnvironmentVariableW(name, value)
    }
    unsafe fn get_environment_from_os() -> *mut u16 {
        from_os::__dcrt_get_wide_environment_from_os()
    }
    unsafe fn get_or_create_environment_nolock() -> *mut *mut u16 {
        init::__dcrt_get_or_create_wide_environment_nolock()
    }
    unsafe fn set_variable_in_environment_nolock(option: *mut u16, is_top_level_call: c_int) -> c_int {
        setenv::__dcrt_set_variable_in_wide_environment_nolock(option, is_top_level_call)
    }
    unsafe fn pre_initialize() {}
    unsafe fn required_transform_count(s: *const u16) -> usize {
        __acrt_WideCharToMultiByte(CP_ACP, 0, s, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut())
            as usize
    }
    unsafe fn transform_string(s: *const u16, buffer: *mut u8, count: usize) -> usize {
        __acrt_WideCharToMultiByte(
            CP_ACP,
            0,
            s,
            -1,
            buffer.cast(),
            count as c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as usize
    }
}

/// `__acrt_lock(__acrt_environment_lock)` ... `__finally { __acrt_unlock }`.
#[inline]
pub(crate) unsafe fn with_environment_lock<R>(f: impl FnOnce() -> R) -> R {
    __acrt_lock(ACRT_ENVIRONMENT_LOCK);
    let r = f();
    __acrt_unlock(ACRT_ENVIRONMENT_LOCK);
    r
}
