//! Shims for the wine debug-channel macros and HRESULT test macros that the
//! rustconv translations declare as `extern "C"` imports (the C had them as
//! macros; the Rust needs real items). Plain Rust fns reached through the
//! `TRACE!`/`WARN!`/`ERR!`/`FIXME!` macros — NOT `#[no_mangle]` link symbols,
//! because nothing links against them (every raw extern decl rustconv emitted is
//! stripped at integration).
//!
//! Policy: TRACE/WARN are no-ops; ERR/FIXME print the raw format string (no
//! interpolation) to the debugger so failures stay loud. debugstr_* return a
//! static placeholder — their results are only ever consumed as TRACE/WARN args.
//!
//! NOT C-VARIADIC, deliberately — identical to `ole32-rust/src/debug_shims.rs`.
//! The wine originals are variadic and rustconv declared them that way, but the
//! macros above reach these with a SINGLE format pointer, so there is nothing to
//! be variadic FOR. DEFINING a C-variadic fn needs `#![feature(c_variadic)]`,
//! i.e. nightly (or `RUSTC_BOOTSTRAP=1`) — which the stable-toolchain lane removed
//! workspace-wide (see `sdk/lib/rust/varargs-rust`). Re-introducing it to print a
//! format string we never interpolate would pay a real cost for nothing.
//! (DECLARING a variadic extern is fine on stable; only defining one is not.)
use core::ffi::c_char;

extern "system" {
    fn OutputDebugStringA(lpOutputString: *const c_char);
}

pub unsafe fn TRACE(_fmt: *const c_char) {}

pub unsafe fn WARN(_fmt: *const c_char) {}

pub unsafe fn ERR(fmt: *const c_char) {
    OutputDebugStringA(b"oleaut32-rust ERR: \0".as_ptr() as *const c_char);
    if !fmt.is_null() {
        OutputDebugStringA(fmt);
    }
}

pub unsafe fn FIXME(fmt: *const c_char) {
    OutputDebugStringA(b"oleaut32-rust FIXME: \0".as_ptr() as *const c_char);
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

// wine debugstr_* — consumed only as (no-op) TRACE/WARN args.
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
    debugstr_vt,
    debugstr_vf,
    debugstr_variant,
    wine_dbgstr_longlong,
);
