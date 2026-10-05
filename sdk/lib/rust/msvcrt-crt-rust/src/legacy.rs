//! The per-DLL overrides of the two legacy CRTs that link this archive: crtdll.dll
//! (dll/win32/crtdll/dllmain.c) and msvcrt20.dll (dll/win32/msvcrt20/msvcrt20.c). They
//! were each DLL's own C module object, which won first-match over the shared bodies.
//!
//! This archive goes whole into msvcrt.dll, msvcrt20.dll, msvcrt40.dll and crtdll.dll
//! alike, so nothing here may take a name msvcrt.dll exports: every symbol carries a
//! `CRTDLL_`/`MSVCRT20_` prefix, and the DLL's spec file maps its export onto it
//! (`@ cdecl _strinc(str) CRTDLL__strinc`). In the C, `_strdec` .. `_strspnp` and
//! `_fileinfo_dll` were unprefixed, which was safe only while they were crtdll's own
//! objects; here an unprefixed `_strinc` would silently be msvcrt.dll's too.
//!
//! Not carried over: dllmain.c's `SystemFunction036`, an `assert(FALSE)` dummy satisfying the
//! shared rand_s's RtlGenRandom import. Defined here it would also bind msvcrt.dll's rand_s
//! to the dummy, so crtdll delay-imports advapi32 for the real one, as msvcrt20/40 do.
#![cfg(target_os = "windows")]

use crate::prelude::*;
use core::ptr::read_volatile;
use varargs_rust::va_entry;

extern "C" {
    fn strlen(s: *const u8) -> usize;
    fn strspn(s: *const u8, accept: *const u8) -> usize;
    fn strstr(hay: *const u8, needle: *const u8) -> *mut u8;
    fn strtoul(nptr: *const u8, endptr: *mut *mut u8, base: i32) -> u32;
    fn wcstoul(nptr: *const u16, endptr: *mut *mut u16, base: i32) -> u32;
    fn system(command: *const u8) -> i32;
}

// ---- msvcrt20.c ----

/// `__getmainargs` (MSVCRT20.@): msvcrt 2.0 takes `new_mode` by value, not as `int *`.
#[no_mangle]
pub unsafe extern "C" fn MSVCRT20__getmainargs(
    argc: *mut i32,
    argv: *mut *mut *mut i8,
    envp: *mut *mut *mut i8,
    expand_wildcards: i32,
    mut new_mode: i32,
) {
    crate::msvcrt::data::__getmainargs(argc, argv, envp, expand_wildcards, &mut new_mode);
}

/// `__wgetmainargs` (MSVCRT20.@): as above.
#[no_mangle]
pub unsafe extern "C" fn MSVCRT20__wgetmainargs(
    argc: *mut i32,
    wargv: *mut *mut *mut wchar_t,
    wenvp: *mut *mut *mut wchar_t,
    expand_wildcards: i32,
    mut new_mode: i32,
) {
    crate::msvcrt::data::__wgetmainargs(argc, wargv, wenvp, expand_wildcards, &mut new_mode);
}

// ---- crtdll dllmain.c ----

// The `_*_dll` version words. msvcrt_init_args (data.rs) fills the three base* ones in
// every DLL that links this archive, as Wine's data.c does under `_CRTDLL`; only crtdll
// exports them. cpumode/osmode are 1 as in Wine's data.c (dllmain.c had them 0, which no
// crtdll ever reported).
#[no_mangle]
pub static mut CRTDLL__basemajor_dll: u32 = 0;
#[no_mangle]
pub static mut CRTDLL__baseminor_dll: u32 = 0;
#[no_mangle]
pub static mut CRTDLL__baseversion_dll: u32 = 0;
#[no_mangle]
pub static mut CRTDLL__cpumode_dll: u32 = 1;
#[no_mangle]
pub static mut CRTDLL__osmajor_dll: u32 = 0;
#[no_mangle]
pub static mut CRTDLL__osminor_dll: u32 = 0;
#[no_mangle]
pub static mut CRTDLL__osmode_dll: u32 = 1;
#[no_mangle]
pub static mut CRTDLL__osversion_dll: u32 = 0;
#[no_mangle]
pub static mut CRTDLL__fileinfo_dll: i32 = 0;

/// crtdll's `struct stat`: `dev_t` is a `short` there but an `unsigned int` in msvcrt.
#[repr(C)]
pub struct crtdll_stat {
    st_dev: i16,
    st_ino: _ino_t,
    st_mode: u16,
    st_nlink: i16,
    st_uid: i16,
    st_gid: i16,
    st_rdev: i16,
    st_size: _off_t,
    st_atime: time_t,
    st_mtime: time_t,
    st_ctime: time_t,
}

fn convert_struct_stat(dst: &mut crtdll_stat, src: &_stat) {
    dst.st_dev = src.st_dev as i16;
    dst.st_ino = src.st_ino;
    dst.st_mode = src.st_mode;
    dst.st_nlink = src.st_nlink;
    dst.st_uid = src.st_uid;
    dst.st_gid = src.st_gid;
    dst.st_rdev = src.st_rdev as i16;
    dst.st_size = src.st_size;
    dst.st_atime = src.st_atime;
    dst.st_mtime = src.st_mtime;
    dst.st_ctime = src.st_ctime;
}

/// `_fstat` (CRTDLL.@)
#[no_mangle]
pub unsafe extern "C" fn CRTDLL__fstat(fd: i32, buf: *mut crtdll_stat) -> i32 {
    let mut st: _stat = core::mem::zeroed();
    let ret = crate::msvcrt::file::_fstat(fd, &mut st);
    if ret == 0 {
        convert_struct_stat(&mut *buf, &st);
    }
    ret
}

/// `_stat` (CRTDLL.@)
#[no_mangle]
pub unsafe extern "C" fn CRTDLL__stat(path: *const i8, buf: *mut crtdll_stat) -> i32 {
    let mut st: _stat = core::mem::zeroed();
    let ret = crate::msvcrt::file::_stat(path, &mut st);
    if ret == 0 {
        convert_struct_stat(&mut *buf, &st);
    }
    ret
}

// The `_str*` family: crtdll's are byte-only, msvcrt's are the MBCS-aware `_mbs*` twins.

/// `_strdec` (CRTDLL.@)
#[no_mangle]
pub extern "C" fn CRTDLL__strdec(_str1: *const u8, str2: *const u8) -> *mut u8 {
    str2.wrapping_sub(1) as *mut u8
}

/// `_strinc` (CRTDLL.@)
#[no_mangle]
pub extern "C" fn CRTDLL__strinc(str: *const u8) -> *mut u8 {
    str.wrapping_add(1) as *mut u8
}

/// `_strncnt` (CRTDLL.@)
#[no_mangle]
pub unsafe extern "C" fn CRTDLL__strncnt(str: *const u8, maxlen: usize) -> usize {
    strlen(str).min(maxlen)
}

/// `_strnextc` (CRTDLL.@): the C reads a plain (signed) `char`, so a high byte
/// sign-extends — 0xE9 comes back as 0xFFFFFFE9, as crtdll's own did.
#[no_mangle]
pub unsafe extern "C" fn CRTDLL__strnextc(str: *const u8) -> u32 {
    *str as i8 as u32
}

/// `_strninc` (CRTDLL.@)
#[no_mangle]
pub extern "C" fn CRTDLL__strninc(str: *const u8, len: usize) -> *mut u8 {
    str.wrapping_add(len) as *mut u8
}

/// `_strspnp` (CRTDLL.@)
#[no_mangle]
pub unsafe extern "C" fn CRTDLL__strspnp(str1: *const u8, str2: *const u8) -> *mut u8 {
    let p = str1.add(strspn(str1, str2));
    if *p != 0 { p as *mut u8 } else { core::ptr::null_mut() }
}

/// `_mbsstr` (CRTDLL.@): byte-wise, and empty-in-empty is NULL rather than the haystack.
#[no_mangle]
pub unsafe extern "C" fn CRTDLL__mbsstr(str: *const u8, substr: *const u8) -> *mut u8 {
    if *str == 0 && *substr == 0 {
        return core::ptr::null_mut();
    }
    strstr(str, substr)
}

// The wrappers below touch their pointer arguments before delegating: crtdll did no
// parameter validation, so a NULL faulted where msvcrt's bodies return an error, and
// callers (and Wine's crtdll tests) rely on the fault.

// int sprintf(char *buffer, const char *format, ...)
va_entry!(CRTDLL_sprintf => |a| -> i32 {
    let (buffer, format) = (a.get(0) as *mut u8, a.get(1) as *const u8);
    read_volatile(buffer);
    read_volatile(format);
    crate::printf_shells::vsprintf(buffer, format, a.va(2) as _)
});

/// `strtoul` (CRTDLL.@)
#[no_mangle]
pub unsafe extern "C" fn CRTDLL_strtoul(nptr: *const u8, endptr: *mut *mut u8, base: i32) -> u32 {
    read_volatile(nptr);
    strtoul(nptr, endptr, base)
}

/// `wcstoul` (CRTDLL.@)
#[no_mangle]
pub unsafe extern "C" fn CRTDLL_wcstoul(nptr: *const u16, endptr: *mut *mut u16, base: i32) -> u32 {
    read_volatile(nptr);
    wcstoul(nptr, endptr, base)
}

/// `system` (CRTDLL.@): clears errno first, so a successful run leaves it 0.
#[no_mangle]
pub unsafe extern "C" fn CRTDLL_system(command: *const u8) -> i32 {
    if !command.is_null() {
        *crate::msvcrt::errno::_errno() = 0;
    }
    system(command)
}

/// `_mbsnbcat` (CRTDLL.@)
#[no_mangle]
pub unsafe extern "C" fn CRTDLL__mbsnbcat(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if n != 0 {
        read_volatile(dest);
        read_volatile(src);
    }
    crate::msvcrt::mbcs::_mbsnbcat(dest, src, n)
}

/// `_mbsncat` (CRTDLL.@)
#[no_mangle]
pub unsafe extern "C" fn CRTDLL__mbsncat(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if n != 0 {
        read_volatile(dest);
        read_volatile(src);
    }
    crate::msvcrt::mbcs::_mbsncat(dest, src, n)
}

/// `_vsnprintf` (CRTDLL.@): -1 when the output did not fit, and errno always cleared.
#[no_mangle]
pub unsafe extern "C" fn CRTDLL__vsnprintf(
    buffer: *mut u8,
    count: usize,
    format: *const u8,
    argptr: *mut u8,
) -> i32 {
    read_volatile(format);
    if count != 0 {
        read_volatile(buffer);
    }
    let mut ret = crate::printf_shells::_vsnprintf(buffer, count, format, argptr);
    if ret > count as i32 {
        ret = -1;
    }
    *crate::msvcrt::errno::_errno() = 0;
    ret
}

// int _snprintf(char *buffer, size_t count, const char *format, ...)
va_entry!(CRTDLL__snprintf => |a| -> i32 {
    CRTDLL__vsnprintf(a.get(0) as _, a.get(1) as _, a.get(2) as _, a.va(3) as _)
});

/// `_vsnwprintf` (CRTDLL.@): -1 when the output did not fit or there is no buffer.
#[no_mangle]
pub unsafe extern "C" fn CRTDLL__vsnwprintf(
    buffer: *mut u16,
    count: usize,
    format: *const u16,
    argptr: *mut u8,
) -> i32 {
    read_volatile(format);
    let mut ret = crate::printf_shells::_vsnwprintf(buffer, count, format, argptr);
    if ret > count as i32 || buffer.is_null() {
        ret = -1;
    }
    *crate::msvcrt::errno::_errno() = 0;
    ret
}
