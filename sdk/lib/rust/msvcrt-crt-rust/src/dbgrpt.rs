//! The debug-report family msvcrt.dll exports (`_CrtDbgReport[W][V]`, `_VCrtDbgReport[A|W]`,
//! `_CrtSetReportMode`, `_CrtSetReportFile`), ported from sdk/lib/crt/misc/dbgrpt.cpp, which
//! msvcrt linked as the `dbgrpt` C++ library. The C++ templates over `char`/`wchar_t` become
//! one generic over [`Ch`]; the two `...` entry points are varargs-rust `va_entry!` thunks.
//!
//! One divergence, on purpose: the C `strcat`s the formatted message after a
//! "file(line) : " prefix into a 4096-element buffer it never bounds, so a long file name
//! plus a near-maximum message wrote past the end of the stack buffer. Here every append is
//! truncated at the buffer's end.
#![cfg(target_os = "windows")]

use crate::prelude::*;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicI32, AtomicPtr, Ordering};
use varargs_rust::va_entry;

/// `L"..."` for the ASCII literals below: a NUL-terminated `&'static [u16]`, built at compile
/// time (a `[u16]` constant is naturally 2-aligned).
const fn ascii_utf16<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    let mut out = [0u16; N];
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i] as u16;
        i += 1;
    }
    out
}
macro_rules! wz {
    ($s:literal) => {{
        const S: &str = $s;
        const W: [u16; S.len() + 1] = ascii_utf16::<{ S.len() + 1 }>(S);
        &W
    }};
}

const _CRT_ASSERT: i32 = 2;
const _CRT_ERRCNT: i32 = 3;
const _CRTDBG_MODE_FILE: i32 = 0x1;
const _CRTDBG_MODE_DEBUG: i32 = 0x2;
const _CRTDBG_MODE_WNDW: i32 = 0x4;
const _CRTDBG_REPORT_MODE: i32 = -1;
const _CRTDBG_INVALID_HFILE: isize = -1;
const _CRTDBG_FILE_STDOUT: isize = -4;
const _CRTDBG_FILE_STDERR: isize = -5;
const _CRTDBG_REPORT_FILE: isize = -6;
const IDABORT: i32 = 3;
const IDRETRY: i32 = 4;
const SIGABRT: i32 = 22;
const DBGRPT_MAX_BUFFER_SIZE: usize = 4096;
/// `_countof(DBGRPT_ASSERT_PREFIX_MESSAGE)`, terminator included.
const ASSERT_PREFIX_COUNT: usize = "Assertion failed: ".len() + 1;

type MessageBoxW = unsafe extern "system" fn(*mut c_void, *const u16, *const u16, u32) -> i32;

extern "system" {
    fn OutputDebugStringA(s: *const i8);
    fn OutputDebugStringW(s: *const u16);
    fn IsDebuggerPresent() -> i32;
    fn LoadLibraryExW(name: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
    fn FreeLibrary(module: *mut c_void) -> i32;
    fn GetProcAddress(module: *mut c_void, name: *const i8) -> *mut c_void;
    fn GetStdHandle(which: u32) -> *mut c_void;
    fn WriteFile(h: *mut c_void, buf: *const c_void, n: u32, written: *mut u32, ov: *mut c_void) -> i32;
}

static CRT_IN_ASSERT: AtomicI32 = AtomicI32::new(-1);
static mut CRT_MODE_OUTPUT_FORMAT: [i32; 3] = [_CRTDBG_MODE_DEBUG, _CRTDBG_MODE_WNDW, _CRTDBG_MODE_WNDW];
static mut CRT_REPORT_FILES: [isize; 3] = [_CRTDBG_INVALID_HFILE; 3];
static CRT_MODE_MESSAGES: [&[u16]; 3] = [
    wz!("Warning"),
    wz!("Error"),
    wz!("Assertion Failed"),
];
/// INVALID_HANDLE_VALUE marks "tried and failed", as in the C.
const INVALID: *mut c_void = usize::MAX as *mut c_void;
static CRT_USER32: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static CRT_MESSAGE_BOX: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

/// `dbgrpt_char_traits<char_t>`.
trait Ch: Copy + Default + Eq + 'static {
    const ASSERTION_MESSAGE: &'static [u16];
    const EMPTY: &'static [Self];
    const UNKNOWN_FILE: &'static [Self];
    fn from_ascii(b: u8) -> Self;
    unsafe fn output_debug_string(s: *const Self);
    unsafe fn vsnprintf(buf: *mut Self, count: usize, format: *const Self, args: *mut c_void) -> i32;
}
impl Ch for u8 {
    const ASSERTION_MESSAGE: &'static [u16] = wz!(
        "Debug %s!\n%s%hs%s%hs%s%s%s%hs\n\n(Press Retry to debug the application)"
    );
    const EMPTY: &'static [u8] = b"\0";
    const UNKNOWN_FILE: &'static [u8] = b"<unknown file>\0";
    fn from_ascii(b: u8) -> u8 {
        b
    }
    unsafe fn output_debug_string(s: *const u8) {
        OutputDebugStringA(s as _)
    }
    unsafe fn vsnprintf(buf: *mut u8, count: usize, format: *const u8, args: *mut c_void) -> i32 {
        crate::printf_shells::_vsnprintf(buf as _, count as _, format as _, args as _) as i32
    }
}
impl Ch for u16 {
    const ASSERTION_MESSAGE: &'static [u16] = wz!(
        "Debug %s!\n%s%ws%s%ws%s%s%s%ws\n\n(Press Retry to debug the application)"
    );
    const EMPTY: &'static [u16] = &[0];
    const UNKNOWN_FILE: &'static [u16] = wz!("<unknown file>");
    fn from_ascii(b: u8) -> u16 {
        b as u16
    }
    unsafe fn output_debug_string(s: *const u16) {
        OutputDebugStringW(s)
    }
    unsafe fn vsnprintf(buf: *mut u16, count: usize, format: *const u16, args: *mut c_void) -> i32 {
        crate::printf_shells::_vsnwprintf(buf as _, count as _, format as _, args as _) as i32
    }
}

unsafe fn len<C: Ch>(s: *const C) -> usize {
    let mut n = 0;
    while *s.add(n) != C::default() {
        n += 1;
    }
    n
}

/// A NUL-terminated buffer that never writes past its end (the divergence in the docs).
struct Buf<C: Ch> {
    b: [C; DBGRPT_MAX_BUFFER_SIZE],
    n: usize,
}
impl<C: Ch> Buf<C> {
    fn new() -> Self {
        Buf { b: [C::default(); DBGRPT_MAX_BUFFER_SIZE], n: 0 }
    }
    fn push(&mut self, c: C) {
        if self.n + 1 < DBGRPT_MAX_BUFFER_SIZE {
            self.b[self.n] = c;
            self.n += 1;
            self.b[self.n] = C::default();
        }
    }
    fn push_ascii(&mut self, s: &str) {
        for b in s.bytes() {
            self.push(C::from_ascii(b));
        }
    }
    unsafe fn push_z(&mut self, s: *const C) {
        for i in 0..len(s) {
            self.push(*s.add(i));
        }
    }
    fn ptr(&self) -> *const C {
        self.b.as_ptr()
    }
}

/// `_itoa`/`_itow` into an ASCII string, as the C formats the line number.
fn decimal(mut v: i32, out: &mut [u8; 12]) -> &str {
    let neg = v < 0;
    let mut i = out.len();
    loop {
        i -= 1;
        out[i] = b'0' + (v % 10).unsigned_abs() as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    if neg {
        i -= 1;
        out[i] = b'-';
    }
    core::str::from_utf8(&out[i..]).unwrap_or("")
}

unsafe fn crt_get_user32() -> *mut c_void {
    if CRT_USER32.load(Ordering::SeqCst).is_null() {
        let mut module = LoadLibraryExW(wz!("user32.dll").as_ptr(), null_mut(), 0);
        if module.is_null() {
            module = INVALID;
        }
        if CRT_USER32.compare_exchange(null_mut(), module, Ordering::SeqCst, Ordering::SeqCst).is_err()
            && module != INVALID
        {
            FreeLibrary(module);
        }
    }
    let m = CRT_USER32.load(Ordering::SeqCst);
    if m != INVALID { m } else { null_mut() }
}

unsafe fn crt_get_message_box() -> Option<MessageBoxW> {
    let module = crt_get_user32();
    if CRT_MESSAGE_BOX.load(Ordering::SeqCst).is_null() && module != INVALID {
        let mut proc_ = GetProcAddress(module, c"MessageBoxW".as_ptr() as _);
        if proc_.is_null() {
            proc_ = INVALID;
        }
        let _ = CRT_MESSAGE_BOX.compare_exchange(null_mut(), proc_, Ordering::SeqCst, Ordering::SeqCst);
    }
    let p = CRT_MESSAGE_BOX.load(Ordering::SeqCst);
    if p.is_null() || p == INVALID { None } else { Some(core::mem::transmute::<*mut c_void, MessageBoxW>(p)) }
}

unsafe fn crt_dbg_report_window<C: Ch>(
    report_type: i32,
    mut filename: *const C,
    linenumber: i32,
    mut module_name: *const C,
    mut message: *const C,
) -> i32 {
    let mut complete = [0u16; DBGRPT_MAX_BUFFER_SIZE];
    let mut line = [0u16; 20];
    if !filename.is_null() && *filename == C::default() {
        filename = core::ptr::null();
    }
    if !module_name.is_null() && *module_name == C::default() {
        module_name = core::ptr::null();
    }
    if !message.is_null() && *message == C::default() {
        message = core::ptr::null();
    }
    if linenumber != 0 {
        let mut digits = [0u8; 12];
        for (i, b) in decimal(linenumber, &mut digits).bytes().enumerate() {
            line[i] = b as u16;
        }
    }
    let empty_w: *const u16 = [0u16].as_ptr();
    let or_empty = |p: *const C| if p.is_null() { C::EMPTY.as_ptr() } else { p };
    _snwprintf(
        complete.as_mut_ptr(),
        complete.len() - 1,
        C::ASSERTION_MESSAGE.as_ptr(),
        CRT_MODE_MESSAGES[report_type as usize].as_ptr(),
        if module_name.is_null() { empty_w } else { wz!("\nModule: ").as_ptr() },
        or_empty(module_name),
        if filename.is_null() { empty_w } else { wz!("\nFile: ").as_ptr() },
        or_empty(filename),
        if line[0] == 0 { empty_w } else { wz!("\nLine: ").as_ptr() },
        if line[0] == 0 { empty_w } else { line.as_ptr() },
        if message.is_null() { empty_w } else { wz!("\n\n").as_ptr() },
        or_empty(message),
    );
    if IsDebuggerPresent() != 0 {
        OutputDebugStringW(complete.as_ptr());
    }
    let Some(message_box) = crt_get_message_box() else {
        return if IsDebuggerPresent() != 0 { IDRETRY } else { IDABORT };
    };
    // MB_ABORTRETRYIGNORE | MB_ICONHAND | MB_SETFOREGROUND | MB_TASKMODAL
    message_box(
        null_mut(),
        complete.as_ptr(),
        wz!("ReactOS C++ Runtime Library").as_ptr(),
        0x2 | 0x10 | 0x1_0000 | 0x2000,
    )
}

/// `__debugbreak()`.
#[inline(always)]
unsafe fn crt_dbg_break() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("int3", options(nomem, nostack));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("brk #0xf000", options(nomem, nostack));
}

unsafe fn crt_enter_dbg_report<C: Ch>(report_type: i32, filename: *const C, linenumber: i32) -> bool {
    if !(0.._CRT_ERRCNT).contains(&report_type) {
        return false;
    }
    if report_type == _CRT_ASSERT && CRT_IN_ASSERT.fetch_add(1, Ordering::SeqCst) + 1 > 0 {
        let mut digits = [0u8; 12];
        let mut line = [0u8; 13];
        let d = decimal(linenumber, &mut digits);
        line[..d.len()].copy_from_slice(d.as_bytes());
        OutputDebugStringA(c"Nested Assert from File: ".as_ptr() as _);
        C::output_debug_string(if filename.is_null() { C::UNKNOWN_FILE.as_ptr() } else { filename });
        OutputDebugStringA(c", Line: ".as_ptr() as _);
        OutputDebugStringA(line.as_ptr() as _);
        OutputDebugStringA(c"\n".as_ptr() as _);
        crt_dbg_break();
        CRT_IN_ASSERT.fetch_sub(1, Ordering::SeqCst);
        return false;
    }
    true
}

fn crt_leave_dbg_report(report_type: i32) {
    if report_type == _CRT_ASSERT {
        CRT_IN_ASSERT.fetch_sub(1, Ordering::SeqCst);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _CrtSetReportMode(report_type: i32, report_mode: i32) -> i32 {
    if !(0.._CRT_ERRCNT).contains(&report_type) {
        return 0;
    }
    let old = CRT_MODE_OUTPUT_FORMAT[report_type as usize];
    if report_mode != _CRTDBG_REPORT_MODE {
        CRT_MODE_OUTPUT_FORMAT[report_type as usize] = report_mode;
    }
    old
}

#[no_mangle]
pub unsafe extern "C" fn _CrtSetReportFile(report_type: i32, report_file: *mut c_void) -> *mut c_void {
    if !(0.._CRT_ERRCNT).contains(&report_type) {
        return null_mut();
    }
    let old = CRT_REPORT_FILES[report_type as usize];
    if report_file as isize != _CRTDBG_REPORT_FILE {
        CRT_REPORT_FILES[report_type as usize] = report_file as isize;
    }
    old as *mut c_void
}

unsafe fn crt_dbg_report_to_file<C: Ch>(file: isize, msg: *const C) -> bool {
    if file == _CRTDBG_INVALID_HFILE || file == 0 {
        return false;
    }
    let h = match file {
        _CRTDBG_FILE_STDOUT => GetStdHandle(-11i32 as u32),
        _CRTDBG_FILE_STDERR => GetStdHandle(-12i32 as u32),
        _ => file as *mut c_void,
    };
    let mut cb = (len(msg) * core::mem::size_of::<C>()) as u32;
    WriteFile(h, msg as _, cb, &mut cb, null_mut()) != 0
}

unsafe fn crt_handle_dbg_report<C: Ch>(
    report_type: i32,
    complete: *const C,
    formatted: *const C,
    filename: *const C,
    linenumber: i32,
    module_name: *const C,
) -> i32 {
    let mode = CRT_MODE_OUTPUT_FORMAT[report_type as usize];
    if mode & _CRTDBG_MODE_FILE != 0 {
        crt_dbg_report_to_file(CRT_REPORT_FILES[report_type as usize], complete);
    }
    if mode & _CRTDBG_MODE_DEBUG != 0 {
        C::output_debug_string(complete);
    }
    if mode & _CRTDBG_MODE_WNDW != 0 {
        match crt_dbg_report_window(report_type, filename, linenumber, module_name, formatted) {
            IDRETRY => return 1,
            IDABORT => {
                raise(SIGABRT);
                _exit(3);
            }
            _ => return 0, // IDIGNORE and anything else
        }
    }
    0
}

unsafe fn vcrt_dbg_report<C: Ch>(
    report_type: i32,
    filename: *const C,
    linenumber: i32,
    module_name: *const C,
    format: *const C,
    arglist: *mut c_void,
) -> i32 {
    // Check for recursive _CrtDbgReport calls, and validate reportType.
    if !crt_enter_dbg_report(report_type, filename, linenumber) {
        return -1;
    }
    let mut formatted = Buf::<C>::new(); // the user's message
    let mut complete = Buf::<C>::new(); // the output for debug / file
    if !filename.is_null() {
        // "%s(%d) : "
        let mut digits = [0u8; 12];
        complete.push_z(filename);
        complete.push_ascii("(");
        complete.push_ascii(decimal(linenumber, &mut digits));
        complete.push_ascii(") : ");
    }
    if !format.is_null() {
        let n = C::vsnprintf(
            formatted.b.as_mut_ptr(),
            DBGRPT_MAX_BUFFER_SIZE - 2 - ASSERT_PREFIX_COUNT,
            format,
            arglist,
        );
        if n < 0 {
            formatted = Buf::new();
            formatted.push_ascii("_CrtDbgReport: String too long");
        }
        if report_type == _CRT_ASSERT {
            complete.push_ascii("Assertion failed: ");
        }
        complete.push_z(formatted.ptr());
    } else if report_type == _CRT_ASSERT {
        complete.push_ascii("Assertion failed!");
    }
    if report_type == _CRT_ASSERT {
        if CRT_MODE_OUTPUT_FORMAT[report_type as usize] & _CRTDBG_MODE_FILE != 0 {
            complete.push_ascii("\r");
        }
        complete.push_ascii("\n");
    }
    // FIXME (as in the C): user report hooks.
    let result =
        crt_handle_dbg_report(report_type, complete.ptr(), formatted.ptr(), filename, linenumber, module_name);
    crt_leave_dbg_report(report_type);
    result
}

#[no_mangle]
pub unsafe extern "C" fn _VCrtDbgReportA(
    report_type: i32,
    filename: *const u8,
    linenumber: i32,
    module_name: *const u8,
    format: *const u8,
    arglist: *mut c_void,
) -> i32 {
    vcrt_dbg_report(report_type, filename, linenumber, module_name, format, arglist)
}
#[no_mangle]
pub unsafe extern "C" fn _VCrtDbgReportW(
    report_type: i32,
    filename: *const u16,
    linenumber: i32,
    module_name: *const u16,
    format: *const u16,
    arglist: *mut c_void,
) -> i32 {
    vcrt_dbg_report(report_type, filename, linenumber, module_name, format, arglist)
}
#[no_mangle]
pub unsafe extern "C" fn _CrtDbgReportV(
    report_type: i32,
    filename: *const u8,
    linenumber: i32,
    module_name: *const u8,
    format: *const u8,
    arglist: *mut c_void,
) -> i32 {
    vcrt_dbg_report(report_type, filename, linenumber, module_name, format, arglist)
}
#[no_mangle]
pub unsafe extern "C" fn _CrtDbgReportWV(
    report_type: i32,
    filename: *const u16,
    linenumber: i32,
    module_name: *const u16,
    format: *const u16,
    arglist: *mut c_void,
) -> i32 {
    vcrt_dbg_report(report_type, filename, linenumber, module_name, format, arglist)
}

// int _CrtDbgReport(int reportType, const char *filename, int linenumber,
//                   const char *moduleName, const char *format, ...)
va_entry!(_CrtDbgReport => |a| -> i32 {
    vcrt_dbg_report::<u8>(a.get(0) as i32, a.get(1) as _, a.get(2) as i32, a.get(3) as _, a.get(4) as _, a.va(5))
});
// int _CrtDbgReportW(... const wchar_t * ..., ...)
va_entry!(_CrtDbgReportW => |a| -> i32 {
    vcrt_dbg_report::<u16>(a.get(0) as i32, a.get(1) as _, a.get(2) as i32, a.get(3) as _, a.get(4) as _, a.va(5))
});
