//! Link-level shims for the wine debug-channel macros and HRESULT test macros
//! that the rustconv translations declare as `extern "C"` imports (the C had
//! them as macros; the Rust needs real items). They are plain Rust fns reached through
//! the prelude — NOT #[no_mangle] link symbols, because nothing links against them.
//!
//! Policy (matches oleaut32-rust + the compobj/marshal lanes): TRACE/WARN are the
//! token-discarding `TRACE!`/`WARN!` of winemacros.rs -- never fns here, because an
//! empty fn still evaluates every argument the C macro discards; ERR/FIXME print the raw format string (no interpolation) to the
//! debugger so failures stay loud. debugstr_* return a static placeholder —
//! their results are only ever consumed as TRACE/WARN varargs.
//!
//! NOT C-VARIADIC, deliberately. The wine originals are variadic, and rustconv's raw
//! output declared them that way — but every one of those extern decls is stripped at
//! integration, and the ring reaches these only through the TRACE!/WARN!/ERR!/FIXME!
//! macros, which pass a single format pointer. So there is nothing to be variadic FOR.
//!
//! That matters: DEFINING a C-variadic fn needs `#![feature(c_variadic)]`, i.e. nightly
//! (or RUSTC_BOOTSTRAP). The stable-toolchain lane removed that feature workspace-wide
//! (see sdk/lib/rust/varargs-rust), and re-introducing it here to print a format string
//! we never interpolate would be paying a real cost for nothing. (DECLARING a variadic
//! extern — as crate::crt does for snprintf/sprintf — is fine on stable; only defining
//! one is not.)
//!
//! NOTE these are deliberately NOT shared with compobj/marshal: those lanes use
//! their own `macro_rules! TRACE!` forms and emit no symbols, so there is no
//! duplicate-definition hazard inside libole32_rust.a.
use core::ffi::c_char;

extern "system" {
    fn OutputDebugStringA(lpOutputString: *const c_char);
}

pub unsafe fn ERR(fmt: *const c_char) {
    OutputDebugStringA(b"ole32-rust ERR: \0".as_ptr() as *const c_char);
    if !fmt.is_null() {
        OutputDebugStringA(fmt);
    }
}

pub unsafe fn FIXME(fmt: *const c_char) {
    OutputDebugStringA(b"ole32-rust FIXME: \0".as_ptr() as *const c_char);
    if !fmt.is_null() {
        OutputDebugStringA(fmt);
    }
}

// HRESULT test macros the translations declared as extern fns.
pub fn FAILED(hr: i32) -> bool {
    hr < 0
}

pub fn SUCCEEDED(hr: i32) -> bool {
    hr >= 0
}

// wine debugstr_* — consumed only as (no-op) TRACE/WARN varargs.
static PLACEHOLDER: &[u8] = b"?\0";

macro_rules! debugstr_stub {
    ($($name:ident),+ $(,)?) => {$(
        pub fn $name(_p: *const core::ffi::c_void) -> *const c_char {
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
    wine_dbgstr_guid,
    wine_dbgstr_rect,
);

/// wine/debug.h `wine_dbg_sprintf(fmt, ...)` — formats into a per-thread ring buffer and
/// returns a borrowed pointer. Its result is only ever consumed as a TRACE/WARN vararg,
/// which we drop, so the placeholder is sufficient (and, unlike the real one, allocation-
/// free). Non-variadic for the same reason as the rest of this module.
pub fn wine_dbg_sprintf(_fmt: *const c_char) -> *const c_char {
    PLACEHOLDER.as_ptr() as *const c_char
}
