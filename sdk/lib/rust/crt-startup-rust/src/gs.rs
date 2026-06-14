//! startup/gs_support.c — the stack security cookie.
//!
//! `__security_init_cookie` (called from every entry point) is reproduced
//! faithfully. `__report_gsfailure` (only reachable on an actual
//! /GS-detected stack-buffer overrun — never on the boot path) preserves
//! the security-critical behaviour (report + terminate); the precise
//! CONTEXT register capture the C does for the debugger is simplified.

use crate::winapi::*;
use core::ffi::c_void;

#[cfg(target_pointer_width = "64")]
const DEFAULT_SECURITY_COOKIE: usize = 0x0000_2B99_2DDF_A232;
#[cfg(target_pointer_width = "32")]
const DEFAULT_SECURITY_COOKIE: usize = 0xBB40_E64E;

#[no_mangle]
pub static mut __security_cookie: usize = DEFAULT_SECURITY_COOKIE;
#[no_mangle]
pub static mut __security_cookie_complement: usize = !DEFAULT_SECURITY_COOKIE;

#[no_mangle]
pub unsafe extern "C" fn __security_init_cookie() {
    if __security_cookie != DEFAULT_SECURITY_COOKIE {
        __security_cookie_complement = !__security_cookie;
        return;
    }

    let mut systime = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    GetSystemTimeAsFileTime(&mut systime);

    let mut cookie: usize;
    #[cfg(target_pointer_width = "64")]
    {
        cookie = (((systime.dwHighDateTime as u64) << 32) | systime.dwLowDateTime as u64) as usize;
    }
    #[cfg(target_pointer_width = "32")]
    {
        cookie = systime.dwLowDateTime as usize;
        cookie ^= systime.dwHighDateTime as usize;
    }

    cookie ^= GetCurrentProcessId() as usize;
    cookie ^= GetCurrentThreadId() as usize;
    cookie ^= GetTickCount() as usize;

    let mut perfctr = LARGE_INTEGER { QuadPart: 0 };
    QueryPerformanceCounter(&mut perfctr);
    #[cfg(target_pointer_width = "64")]
    {
        cookie ^= perfctr.QuadPart as usize;
        cookie &= 0x0000_ffff_ffff_ffff;
    }
    #[cfg(target_pointer_width = "32")]
    {
        cookie ^= perfctr.low() as usize;
        cookie ^= perfctr.high() as usize;
    }

    if cookie == DEFAULT_SECURITY_COOKIE {
        cookie = DEFAULT_SECURITY_COOKIE + 1;
    }
    __security_cookie = cookie;
    __security_cookie_complement = !cookie;
}

// ---- GS failure path (off the boot path) ----------------------------
#[cfg(target_arch = "x86_64")]
const CONTEXT_SIZE: usize = 0x4D0;
#[cfg(target_arch = "x86")]
const CONTEXT_SIZE: usize = 0x2CC;
#[cfg(target_arch = "aarch64")]
const CONTEXT_SIZE: usize = 0x390;

#[repr(C, align(16))]
struct ContextBuf([u8; CONTEXT_SIZE]);

#[no_mangle]
pub static mut GS_ExceptionRecord: EXCEPTION_RECORD = EXCEPTION_RECORD {
    ExceptionCode: 0,
    ExceptionFlags: 0,
    ExceptionRecord: core::ptr::null_mut(),
    ExceptionAddress: core::ptr::null_mut(),
    NumberParameters: 0,
    ExceptionInformation: [0; EXCEPTION_MAXIMUM_PARAMETERS],
};
#[no_mangle]
static mut GS_ContextRecord: ContextBuf = ContextBuf([0; CONTEXT_SIZE]);

#[no_mangle]
pub static mut GS_ExceptionPointers: EXCEPTION_POINTERS = EXCEPTION_POINTERS {
    ExceptionRecord: core::ptr::addr_of_mut!(GS_ExceptionRecord),
    ContextRecord: core::ptr::addr_of_mut!(GS_ContextRecord) as *mut c_void,
};

#[no_mangle]
pub unsafe extern "C" fn __report_gsfailure(_stack_cookie: usize) -> ! {
    #[cfg(target_arch = "x86_64")]
    RtlCaptureContext(core::ptr::addr_of_mut!(GS_ContextRecord) as *mut c_void);

    // The precise faulting PC/SP capture (for a debugger) is omitted — the
    // security behaviour is "report + terminate", which is preserved.
    GS_ExceptionRecord.ExceptionAddress = core::ptr::null_mut();
    GS_ExceptionRecord.ExceptionCode = STATUS_STACK_BUFFER_OVERRUN;
    GS_ExceptionRecord.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
    GS_ExceptionRecord.NumberParameters = 0;

    SetUnhandledExceptionFilter(None);
    UnhandledExceptionFilter(core::ptr::addr_of!(GS_ExceptionPointers) as *mut EXCEPTION_POINTERS);
    TerminateProcess(GetCurrentProcess(), STATUS_STACK_BUFFER_OVERRUN);
    abort()
}
