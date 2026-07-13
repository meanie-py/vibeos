//! Link-level shims for the wine debug-channel macros and HRESULT test macros
//! that the rustconv translations declare as `extern "C"` imports (the C had
//! them as macros; the Rust needs real symbols). #[no_mangle] so every module's
//! local extern block resolves here at staticlib link.
//!
//! Policy (matches Lane A/B): TRACE/WARN are no-ops; ERR/FIXME print the raw
//! format string (no interpolation) to the debugger so failures stay loud.
//! debugstr_* return a static placeholder — their results are only ever
//! consumed as TRACE/WARN varargs.
use core::ffi::{c_char, c_int, c_uchar};

extern "system" {
    fn OutputDebugStringA(lpOutputString: *const c_char);
}

// ---- wine debug-channel surface (sdk/include/wine/debug.h) ----
//
// `rustconv translate --c2rust` anchors each function on c2rust's transpilation,
// and c2rust runs the *preprocessor* — so wine's `TRACE(...)` macro arrives already
// expanded into its underlying channel call:
//
//     ros_dbg_log(__WINE_DBCL_TRACE, &__wine_dbch_ole, file, func, line, fmt, ...)
//
// rather than as a call to the `TRACE` shim above. That expansion alone accounted
// for ~160 of this crate's compile errors. `ros_dbg_log` is a REAL exported symbol
// (debug.h:184), not a macro, so these are honest declarations, not stubs — the
// traces stay live. Layouts are transcribed from debug.h:49-64; do not guess them.

#[repr(C)]
pub struct __wine_debug_channel {
    pub flags: c_uchar,
    pub name: [c_char; 15],
}

pub type __wine_debug_class = c_int;
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0;
pub const __WINE_DBCL_ERR: __wine_debug_class = 1;
pub const __WINE_DBCL_WARN: __wine_debug_class = 2;
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3;
pub const __WINE_DBCL_INIT: __wine_debug_class = 0x7;

/// wine/debug.h:375-379 — `WINE_DECLARE_DEBUG_CHANNEL(ch)` / `WINE_DEFAULT_DEBUG_CHANNEL(ch)`
/// initialise the channel struct as `{ (unsigned char)~0, #ch }`, i.e. **all trace classes
/// enabled at compile time**. That is deliberate in wine: the static flags only gate the
/// *call* into `ros_dbg_log`, which then applies the real runtime filter
/// (`__wine_dbg_get_channel_flags`, libwine/debug.c:314) before printing anything.
///
/// So 0xFF is the faithful value — it preserves C behaviour exactly, and traces stay
/// runtime-controllable instead of being compiled out. (msgina-rust/src/prelude.rs:36-38
/// takes the other road: a `wine_debug` cargo feature defaulting to 0, hard-silencing the
/// channel. If oleaut32's hot VARIANT paths ever show the per-site flag test in a profile,
/// that is the knob to copy — but do it deliberately, not by accident.)
pub const WINE_DBG_DEFAULT_FLAGS: c_uchar = 0xFF;

extern "C" {
    // cdecl + varargs — declaring a variadic extern is stable Rust (only *defining*
    // one needs #![feature(c_variadic)]).
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const c_char,
        func: *const c_char,
        line: c_int,
        format: *const c_char,
        ...
    ) -> c_int;

    /// wine/debug.h:180 — `extern int wine_dbg_printf( const char *format, ... )`.
    /// A REAL exported symbol (sdk/lib/rtl/wine_debug.c), not a macro: `MESSAGE`/`WINE_DPRINTF`
    /// are `#define`d straight to it (debug.h:381-382). cdecl + varargs.
    pub fn wine_dbg_printf(format: *const c_char, ...) -> c_int;
}

#[no_mangle]
pub unsafe extern "C" fn TRACE(_fmt: *const c_char, _args: ...) {}

#[no_mangle]
pub unsafe extern "C" fn WARN(_fmt: *const c_char, _args: ...) {}

#[no_mangle]
pub unsafe extern "C" fn ERR(fmt: *const c_char, _args: ...) {
    OutputDebugStringA(b"oleaut32-rust ERR: \0".as_ptr() as *const c_char);
    if !fmt.is_null() {
        OutputDebugStringA(fmt);
    }
}

#[no_mangle]
pub unsafe extern "C" fn FIXME(fmt: *const c_char, _args: ...) {
    OutputDebugStringA(b"oleaut32-rust FIXME: \0".as_ptr() as *const c_char);
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
    debugstr_vt,
    debugstr_vf,
    debugstr_variant,
    // olepicture.c calls this spelling (wine/debug.h aliases the two); without a
    // shim it is an undefined symbol at link, not a compile error — so it would
    // only surface at the very end of the build.
    wine_dbgstr_variant,
    wine_dbgstr_longlong,
);
