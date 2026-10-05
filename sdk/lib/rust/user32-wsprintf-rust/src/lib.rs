//! user32's wsprintf family (sdk/lib/crt `user32_wsprintf`, `-D_USER32_WSPRINTF`).
//!
//! * `streamout`/`wstreamout`: cnt-rust's engine, compiled here under `cfg(streamout_user32)`.
//! * `wvsprintfA/W`, `wvsnprintfA/W`: printf/_sxprintf.c's non-secure path into a string
//!   stream. stdcall on i386 (`USER32_WSPRINTF && _M_IX86`), `extern "system"` here.
//! * `wsprintfA/W`: the C-variadic exports, as naked thunks (`varargs` below).
//! * `strnlen`/`wcsnlen` and the NT-flavoured `mbtowc`/`mbstowcs`/`wctomb`/`wcstombs`
//!   (string/*_nt.c over the Rtl conversion APIs), which the engine's `%S`/`%s`
//!   cross-width conversions call.
#![no_std]
#![allow(non_snake_case, clippy::missing_safety_doc)]

#[allow(dead_code)]
#[path = "../../cnt-rust/src/cabi/streamout.rs"]
mod streamout;
mod varargs;

use streamout::File;

const INT_MAX: usize = 0x7fff_ffff;
const _IOWRT: i32 = 0x0002;
const _IOSTRG: i32 = 0x0040;

/// printf/_sxprintf.c, `!IS_SECAPI`: a string stream over `buffer`, NUL-terminated only if room
/// is left. `count` is in characters.
unsafe fn sxprintf<T: Copy>(
    buffer: *mut T,
    count: usize,
    format: *const T,
    argptr: *mut u8,
    engine: unsafe extern "C" fn(*mut File, *const T, *mut u8) -> i32,
    nul: T,
) -> i32 {
    let size_of_t = core::mem::size_of::<T>();
    let mut stream = File {
        _ptr: buffer as *mut u8,
        _cnt: (count * size_of_t) as i32,
        _base: buffer as *mut u8,
        _flag: _IOSTRG | _IOWRT,
        _file: 0,
        _charbuf: 0,
        _bufsiz: 0,
        _tmpfname: core::ptr::null_mut(),
    };
    let result = engine(&mut stream, format, argptr);
    // `if ((stream._cnt >= sizeof(_TCHAR)) && (stream._ptr))` -- sizeof is size_t, so the
    // C compares unsigned: INT_MAX * sizeof(WCHAR) overflowed _cnt to a negative int, which
    // is a huge size_t, and the unbounded wide path still terminates.
    if (stream._cnt as isize as usize) >= size_of_t && !stream._ptr.is_null() {
        core::ptr::write_volatile(stream._ptr as *mut T, nul);
    }
    result
}

/// `int wvsnprintfA(LPSTR, size_t, LPCSTR, va_list)` -- printf/wvsnprintfA.c.
#[no_mangle]
pub unsafe extern "system" fn wvsnprintfA(buffer: *mut u8, count: usize, format: *const u8, argptr: *mut u8) -> i32 {
    sxprintf(buffer, count, format, argptr, streamout::streamout, 0u8)
}

/// `int wvsnprintfW(LPWSTR, size_t, LPCWSTR, va_list)` -- printf/wvsnprintfW.c.
#[no_mangle]
pub unsafe extern "system" fn wvsnprintfW(buffer: *mut u16, count: usize, format: *const u16, argptr: *mut u8) -> i32 {
    sxprintf(buffer, count, format, argptr, streamout::wstreamout, 0u16)
}

/// `int wvsprintfA(LPSTR, LPCSTR, va_list)` -- printf/wvsprintfA.c (count INT_MAX).
#[no_mangle]
pub unsafe extern "system" fn wvsprintfA(buffer: *mut u8, format: *const u8, argptr: *mut u8) -> i32 {
    sxprintf(buffer, INT_MAX, format, argptr, streamout::streamout, 0u8)
}

/// `int wvsprintfW(LPWSTR, LPCWSTR, va_list)` -- printf/wvsprintfW.c (count INT_MAX).
#[no_mangle]
pub unsafe extern "system" fn wvsprintfW(buffer: *mut u16, format: *const u16, argptr: *mut u8) -> i32 {
    sxprintf(buffer, INT_MAX, format, argptr, streamout::wstreamout, 0u16)
}

/// The body of `wsprintfA(LPSTR, LPCSTR, ...)` once a thunk has made its `...` a `va_list`.
pub(crate) unsafe extern "C" fn wsprintf_va_a(buffer: *mut u8, format: *const u8, argptr: *mut u8) -> i32 {
    sxprintf(buffer, INT_MAX, format, argptr, streamout::streamout, 0u8)
}

/// The body of `wsprintfW(LPWSTR, LPCWSTR, ...)`.
pub(crate) unsafe extern "C" fn wsprintf_va_w(buffer: *mut u16, format: *const u16, argptr: *mut u8) -> i32 {
    sxprintf(buffer, INT_MAX, format, argptr, streamout::wstreamout, 0u16)
}

// ── string/strnlen.c, string/wcsnlen.c ───────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn strnlen(s: *const u8, maxlen: usize) -> usize {
    let mut n = 0;
    while n < maxlen && *s.add(n) != 0 {
        n += 1;
    }
    n
}

#[no_mangle]
pub unsafe extern "C" fn wcsnlen(s: *const u16, maxlen: usize) -> usize {
    let mut n = 0;
    while n < maxlen && *s.add(n) != 0 {
        n += 1;
    }
    n
}

// ── string/mbstowcs_nt.c, string/wcstombs_nt.c ──────────────────────────────────────────────

#[link(name = "ntdll")]
extern "system" {
    fn RtlAnsiCharToUnicodeChar(source: *mut *mut u8) -> u16;
    fn RtlMultiByteToUnicodeSize(size: *mut u32, mb: *const u8, len: u32) -> i32;
    fn RtlMultiByteToUnicodeN(dst: *mut u16, max: u32, size: *mut u32, src: *const u8, len: u32) -> i32;
    fn RtlUnicodeToMultiByteSize(size: *mut u32, src: *const u16, len: u32) -> i32;
    fn RtlUnicodeToMultiByteN(dst: *mut u8, max: u32, size: *mut u32, src: *const u16, len: u32) -> i32;
}
extern "C" {
    fn strlen(s: *const u8) -> usize;
    fn wcslen(s: *const u16) -> usize;
}

/// mbstowcs_nt.c: `MB_CUR_MAX` is redefined to 2 there.
#[no_mangle]
pub unsafe extern "C" fn mbtowc(wchar: *mut u16, mbchar: *const u8, count: usize) -> i32 {
    let mut mbarr = [0u8; 2];
    if mbchar.is_null() || wchar.is_null() {
        return 0;
    }
    core::ptr::copy_nonoverlapping(mbchar, mbarr.as_mut_ptr(), count.min(2));
    let mut mbs = mbarr.as_mut_ptr();
    let wc = RtlAnsiCharToUnicodeChar(&mut mbs);
    if wc == b' ' as u16 && mbarr[0] != b' ' {
        return -1;
    }
    *wchar = wc;
    mbs.offset_from(mbarr.as_ptr()) as i32
}

#[no_mangle]
pub unsafe extern "C" fn mbstowcs(wcstr: *mut u16, mbstr: *const u8, count: usize) -> usize {
    let length = strlen(mbstr) as u32;
    let mut size: u32 = 0;
    if wcstr.is_null() {
        RtlMultiByteToUnicodeSize(&mut size, mbstr, length);
        return (size / 2) as usize;
    }
    if RtlMultiByteToUnicodeN(wcstr, (count as u32).wrapping_mul(2), &mut size, mbstr, length) < 0 {
        return usize::MAX; // (size_t)-1
    }
    (size / 2) as usize
}

#[no_mangle]
pub unsafe extern "C" fn wctomb(mbchar: *mut u8, wchar: u16) -> i32 {
    if mbchar.is_null() {
        return 0;
    }
    let mut size: u32 = 0;
    if RtlUnicodeToMultiByteN(mbchar, 1, &mut size, &wchar, 2) < 0 {
        return -1;
    }
    size as i32
}

#[no_mangle]
pub unsafe extern "C" fn wcstombs(mbstr: *mut u8, wcstr: *const u16, count: usize) -> usize {
    let length = wcslen(wcstr) as u32;
    let mut size: u32 = 0;
    if mbstr.is_null() {
        RtlUnicodeToMultiByteSize(&mut size, wcstr, length * 2);
        return size as usize;
    }
    if RtlUnicodeToMultiByteN(mbstr, count as u32, &mut size, wcstr, length * 2) < 0 {
        return usize::MAX;
    }
    size as usize
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
