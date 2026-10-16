//! Shims for the wine debug-channel macros and HRESULT test macros that the
//! rustconv translations declare as `extern "C"` imports (the C had them as
//! macros; the Rust needs real items). Plain Rust fns reached through the
//! `TRACE!`/`WARN!`/`ERR!`/`FIXME!` macros — NOT `#[no_mangle]` link symbols,
//! because nothing links against them (every raw extern decl rustconv emitted is
//! stripped at integration).
//!
//! Policy: TRACE/WARN are winemacros.rs's token-discarding macros, never fns; ERR/FIXME print
//! through winedebug-rust's `DbgPrint` with their arguments, so the debugstr_* a live ERR/FIXME
//! formats are real (win32macros-rust's wine/debug.h port).
//!
//! NOT C-VARIADIC, deliberately — identical to `ole32-rust/src/debug_shims.rs`.
//! The wine originals are variadic and rustconv declared them that way, but the
//! macros above reach these with a SINGLE format pointer, so there is nothing to
//! be variadic FOR. DEFINING a C-variadic fn needs `#![feature(c_variadic)]`,
//! i.e. nightly (or `RUSTC_BOOTSTRAP=1`) — which the stable-toolchain lane removed
//! workspace-wide (see `sdk/lib/rust/varargs-rust`). Re-introducing it to print a
//! format string we never interpolate would pay a real cost for nothing.
//! (DECLARING a variadic extern is fine on stable; only defining one is not.)
use core::ffi::{c_char, c_int, c_uchar};

extern "system" {
    fn OutputDebugStringA(lpOutputString: *const c_char);
}

// ---- wine debug-channel surface (sdk/include/wine/debug.h) ----
//
// The Tier-D modules (connpt/olefont/olepicture/olepropframe/recinfo) were translated
// with `--c2rust`, and c2rust runs the *preprocessor* — so wine's `TRACE(...)` arrives
// already expanded into its underlying channel call:
//
//     ros_dbg_log(__WINE_DBCL_TRACE, &__wine_dbch_ole, file, func, line, fmt, ...)
//
// rather than as a call to the `TRACE` shim above. (That expansion alone accounted for
// ~160 of this crate's compile errors when the surface was missing.)
//
// These are honest DECLARATIONS, not stubs: `ros_dbg_log` is a real exported symbol
// (wine/debug.h:184), so the traces stay live and runtime-filterable. Declaring a
// variadic extern is fine on stable — only *defining* one needs `c_variadic`, which is
// exactly why the shims above are non-variadic and this block can still exist.
// Layouts transcribed from wine/debug.h:49-64; do not guess them.

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
    // cdecl + varargs. DECLARING a variadic extern is stable Rust; only defining one is not.
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

// wine debugstr_*. ERR!/FIXME! interpolate their arguments (winedebug-rust's `DbgPrint`), so
// these must produce the real text: a "?" placeholder here printed "asking for unsupported
// interface ?" for every live FIXME. The formatting is win32macros-rust's port of wine/debug.h
// over libwine (which oleaut32 links). TRACE!/WARN! discard their tokens, so the calls inside
// them are never type-checked or evaluated.
//
// Generic over the pointee, unlike ole32's `*const c_void` twin. The Tier-D modules here were
// translated with `--c2rust`, which expands the C preprocessor — so these are REAL calls, passing
// the C's real argument types (`debugstr_w(LPCWSTR)`, `debugstr_guid(const GUID*)`, and olefont's
// `*const c_void`). Accepting any `*const T` lets the C's own types stand. Safe fns, as the
// placeholders were: they only read what the C's TRACE would have read.
static PLACEHOLDER: &[u8] = b"?\0";

pub fn debugstr_w<T>(p: *const T) -> *const c_char {
    unsafe { win32macros_rust::debugstr_w(p as *const u16) }
}

pub fn debugstr_a<T>(p: *const T) -> *const c_char {
    unsafe { win32macros_rust::debugstr_a(p as *const c_char) }
}

pub fn debugstr_guid<T>(p: *const T) -> *const c_char {
    unsafe { win32macros_rust::debugstr_guid(p) }
}

pub fn debugstr_variant<T>(p: *const T) -> *const c_char {
    unsafe { win32macros_rust::debugstr_variant(p) }
}

/// olepicture.c calls this spelling (wine/debug.h aliases the two).
pub fn wine_dbgstr_variant<T>(p: *const T) -> *const c_char {
    unsafe { win32macros_rust::wine_dbgstr_variant(p) }
}

pub fn debugstr_vt(vt: u16) -> *const c_char {
    unsafe { win32macros_rust::debugstr_vt(vt) }
}

pub fn wine_dbgstr_longlong(ll: u64) -> *const c_char {
    unsafe { win32macros_rust::wine_dbgstr_longlong(ll) }
}

// Reached only inside TRACE!/WARN!, whose tokens are discarded: no live call exists to give
// them a real signature to match (`debugstr_an`/`debugstr_wn` take a length the C passes).
macro_rules! debugstr_stub {
    ($($name:ident),+ $(,)?) => {$(
        pub fn $name<T>(_p: *const T) -> *const c_char {
            PLACEHOLDER.as_ptr() as *const c_char
        }
    )+};
}

debugstr_stub!(debugstr_an, debugstr_wn, debugstr_vf);
