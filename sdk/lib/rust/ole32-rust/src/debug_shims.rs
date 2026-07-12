//! Link-level shims for the wine debug-channel macros and HRESULT test macros
//! that the rustconv translations declare as `extern "C"` imports (the C had
//! them as macros; the Rust needs real symbols). #[no_mangle] so every ring
//! module's local extern block resolves here at staticlib link.
//!
//! Policy (matches oleaut32-rust + the compobj/marshal lanes): TRACE/WARN are
//! no-ops; ERR/FIXME print the raw format string (no interpolation) to the
//! debugger so failures stay loud. debugstr_* return a static placeholder —
//! their results are only ever consumed as TRACE/WARN varargs.
//!
//! NOTE these are deliberately NOT shared with compobj/marshal: those lanes use
//! their own `macro_rules! TRACE!` forms and emit no symbols, so there is no
//! duplicate-definition hazard inside libole32_rust.a.
use core::ffi::c_char;

extern "system" {
    fn OutputDebugStringA(lpOutputString: *const c_char);
}

#[no_mangle]
pub unsafe extern "C" fn TRACE(_fmt: *const c_char, _args: ...) {}

#[no_mangle]
pub unsafe extern "C" fn WARN(_fmt: *const c_char, _args: ...) {}

#[no_mangle]
pub unsafe extern "C" fn ERR(fmt: *const c_char, _args: ...) {
    OutputDebugStringA(b"ole32-rust ERR: \0".as_ptr() as *const c_char);
    if !fmt.is_null() {
        OutputDebugStringA(fmt);
    }
}

#[no_mangle]
pub unsafe extern "C" fn FIXME(fmt: *const c_char, _args: ...) {
    OutputDebugStringA(b"ole32-rust FIXME: \0".as_ptr() as *const c_char);
    if !fmt.is_null() {
        OutputDebugStringA(fmt);
    }
}

// HRESULT test macros the translations declared as extern fns.
#[no_mangle]
pub extern "C" fn FAILED(hr: i32) -> bool {
    hr < 0
}

#[no_mangle]
pub extern "C" fn SUCCEEDED(hr: i32) -> bool {
    hr >= 0
}

// wine debugstr_* — consumed only as (no-op) TRACE/WARN varargs.
static PLACEHOLDER: &[u8] = b"?\0";

macro_rules! debugstr_stub {
    ($($name:ident),+ $(,)?) => {$(
        #[no_mangle]
        pub extern "C" fn $name(_p: *const core::ffi::c_void) -> *const c_char {
            PLACEHOLDER.as_ptr() as *const c_char
        }
    )+};
}

debugstr_stub!(
    debugstr_a,
    debugstr_w,
    debugstr_an,
    debugstr_wn,
    debugstr_guid,
    debugstr_user_flags,
    debugstr_formatetc,
    wine_dbgstr_longlong,
);
