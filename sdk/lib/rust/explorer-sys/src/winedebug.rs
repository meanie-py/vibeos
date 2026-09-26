//! Wine's debug channels (`sdk/include/wine/debug.h`), centralised for the crate.
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

pub mod ffi {
    extern "C" {
        /// ntdll's `DbgPrint` — the serial sink `wine_dbg_log` ends up in anyway.
        pub fn DbgPrint(Format: *const core::ffi::c_char, ...) -> u32;
    }
}

/// `ERR(fmt, ...)` — on by default (`__WINE_DBCL_ERR` is in `default_flags`).
#[macro_export]
macro_rules! ERR {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::winedebug::ffi::DbgPrint(
                concat!("err:(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
}

/// `FIXME(fmt, ...)` — on by default (`__WINE_DBCL_FIXME` is in `default_flags`).
#[macro_export]
macro_rules! FIXME {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::winedebug::ffi::DbgPrint(
                concat!("fixme:(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
}

// nolint: scan_dropped_prints (channel off by default — see the module note)
/// Off unless `DEBUGCHANNEL` opens the channel; silence is the faithful default.
#[macro_export]
macro_rules! WARN {
    ($($t:tt)*) => {
        ()
    };
}

// nolint: scan_dropped_prints (channel off by default — see the module note)
/// Off unless `DEBUGCHANNEL` opens the channel; silence is the faithful default.
#[macro_export]
macro_rules! TRACE {
    ($($t:tt)*) => {
        ()
    };
}
