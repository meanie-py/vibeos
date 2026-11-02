//! Wine's debug channels (`sdk/include/wine/debug.h`), shared by every Wine-derived crate.
//!
//! Thirty-one crates carried a byte-identical copy of this file. A consumer takes the
//! macros crate-wide with `#[macro_use] extern crate winedebug_rust;` in its `lib.rs`.
//!
//! `ERR` and `FIXME` are ON by default: libwine's `default_flags` is
//! `(1 << __WINE_DBCL_ERR) | (1 << __WINE_DBCL_FIXME)` (`sdk/lib/libwine/debug.c`),
//! and `WINE_DEFAULT_DEBUG_CHANNEL` initialises a channel to `~0` so it takes those
//! defaults. Only `DEBUGCHANNEL` in the environment turns one off. Translating all four
//! macros to silence therefore dropped every ERR and FIXME in the crate while the call
//! sites still read as present.
//!
//! `WARN` and `TRACE` are the reverse — off unless a channel is explicitly opened — so
//! they stay silent here, deliberately.
//!
//! These are `macro_rules!` and never no-op `fn`s: an out-of-line no-op is an
//! optimization barrier that uncloaks latent ABI mismatches (AGENTS.md, field manual
//! §9.65). The C prefixes each line with the class and `__FILE__`/`__LINE__`;
//! `file!()`/`line!()` fold into the literal via `concat!` for the same output with
//! fewer varargs to get wrong.

#![no_std]

pub mod ffi {
    extern "C" {
        /// ntdll's `DbgPrint` — the serial sink `wine_dbg_log` ends up in anyway.
        pub fn DbgPrint(Format: *const core::ffi::c_char, ...) -> u32;
    }
}

/// (win32macros-rust has the twin, `DBG_TYPECHECK`; this crate takes no dependencies.)
#[doc(hidden)]
/// The arguments of a debug macro whose channel is OFF (`TRACE`/`WARN`, a stubbed `DPRINT`),
/// TYPE-CHECKED but never evaluated or emitted. They sit in a closure that is never called:
/// the closure body is name-resolved and type-checked like any code, but no call means no codegen,
/// so a debug helper that is declared and never linked still links. A swallowing
/// `($($t:tt)*) => {{}}` compiles nothing, so a stale argument (a renamed local, a field
/// that moved) rots silently, and every value computed only for a trace reads as unused.
/// No catch-all arm: a call that is not `fmt, args..` must fail, not vanish.
#[macro_export]
macro_rules! __WINEDEBUG_TYPECHECK {
    () => {{}};
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        let _ = || {
            // wine/debug.h's `debugstr_*`, as SIGNATURES only, so `TRACE!("%s", debugstr_w(x))`
            // checks in a crate that never imported one. Items, not a `$crate::` module: macro
            // hygiene leaves items visible to the caller's tokens, and the difftest harness mounts
            // this file as a submodule, where a crate-root path does not exist. They sit inside
            // a never-called closure, so they are never codegened and link nothing.
            #[allow(dead_code)]
            unsafe fn debugstr_w(_s: *const u16) -> *const ::core::ffi::c_char { ::core::ptr::null() }
            #[allow(dead_code)]
            unsafe fn debugstr_wn(_s: *const u16, _n: i32) -> *const ::core::ffi::c_char { ::core::ptr::null() }
            #[allow(dead_code)]
            unsafe fn debugstr_a(_s: *const ::core::ffi::c_char) -> *const ::core::ffi::c_char { ::core::ptr::null() }
            #[allow(dead_code)]
            unsafe fn debugstr_an(_s: *const ::core::ffi::c_char, _n: i32) -> *const ::core::ffi::c_char { ::core::ptr::null() }
            #[allow(dead_code)]
            unsafe fn debugstr_guid<T>(_id: *const T) -> *const ::core::ffi::c_char { ::core::ptr::null() }
            #[allow(unused_unsafe)]
            unsafe {
                // `let _ = place;` binds nothing: no read, no move, no borrow, so a packed
                // field or a moved local is fine, and the name still resolves and type-checks.
                let _ = $fmt;
                $(let _ = $arg;)*
            }
        };
    }};
}

/// `ERR(fmt, ...)` — on by default (`__WINE_DBCL_ERR` is in `default_flags`).
#[macro_export]
macro_rules! ERR {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::ffi::DbgPrint(
                concat!("err:(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
    // A format the translation already made a C pointer (`b"..\0".as_ptr()`): the prefix
    // cannot be concatenated onto it, so it goes out as its own line first.
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::ffi::DbgPrint(concat!("err:(", file!(), ":", line!(), ") \0").as_ptr() as *const _);
            $crate::ffi::DbgPrint($fmt as *const _ $(, $arg)*)
        }
    };
}

/// `FIXME(fmt, ...)` — on by default (`__WINE_DBCL_FIXME` is in `default_flags`).
#[macro_export]
macro_rules! FIXME {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::ffi::DbgPrint(
                concat!("fixme:(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
    // A format the translation already made a C pointer (`b"..\0".as_ptr()`): the prefix
    // cannot be concatenated onto it, so it goes out as its own line first.
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::ffi::DbgPrint(concat!("fixme:(", file!(), ":", line!(), ") \0").as_ptr() as *const _);
            $crate::ffi::DbgPrint($fmt as *const _ $(, $arg)*)
        }
    };
}

// nolint: scan_dropped_prints (channel off by default — see the module note)
/// Off unless `DEBUGCHANNEL` opens the channel; silence is the faithful default.
#[macro_export]
macro_rules! WARN {
    ($($t:tt)*) => {
        $crate::__WINEDEBUG_TYPECHECK!($($t)*)
    };
}

// nolint: scan_dropped_prints (channel off by default — see the module note)
/// Off unless `DEBUGCHANNEL` opens the channel; silence is the faithful default.
#[macro_export]
macro_rules! TRACE {
    ($($t:tt)*) => {
        $crate::__WINEDEBUG_TYPECHECK!($($t)*)
    };
}
