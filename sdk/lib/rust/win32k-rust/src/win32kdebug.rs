//! win32k's debug macros — `win32ss/user/ntuser/win32kdebug.h`, centralised.
//!
//! `ERR` is NOT channel-gated. `DBG_PRINT`'s guard is
//! `if ((level == ERR_LEVEL) || (ppi && DBG_IS_CHANNEL_ENABLED(...)))`, so the
//! ERR level short-circuits before the per-process `DbgChannelLevel` is ever
//! read: under `DBG` — and our media is `-DCMAKE_BUILD_TYPE=Debug`, i.e.
//! `-DDBG=1` — every `ERR` prints, always. Translating it to a silent macro
//! per file dropped 44 files' worth of always-on diagnostics while the call
//! sites still read as present.
//!
//! `FIXME`/`WARN`/`TRACE` genuinely do consult `ppi->DbgChannelLevel[ch]`,
//! which starts ERR-only and is opened per process by the `DEBUGCHANNEL`
//! environment variable. Nothing in the boot path sets it, so silence is the
//! faithful boot behaviour for those three; they stay no-ops here on purpose.
//!
//! `DPRINT1` comes from `sdk/include/reactos/debug.h` and sits OUTSIDE that
//! header's `NDEBUG` guard — only `DPRINT` is silenced by `NDEBUG`. Both are
//! `macro_rules!` and never no-op `fn`s: an out-of-line no-op is an
//! optimization barrier that uncloaks latent ABI mismatches (AGENTS.md).
//!
//! The C prefixes the message with `"(%s:%d) "` + `__RELFILE__`/`__LINE__` as
//! two extra varargs; `file!()`/`line!()` fold into the literal via `concat!`
//! instead, for the same output with fewer arguments to get wrong.

/// `ERR(fmt, ...)` — always on under `DBG`.
macro_rules! ERR {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        ::ntkrnl_sys::driver_ddk::DbgPrint(
            concat!("(", file!(), ":", line!(), ") err: ", $fmt, "\0").as_ptr() as *const i8
            $(, $arg)*
        )
    };
}

/// `DPRINT1(fmt, ...)` — always on under `DBG`; `NDEBUG` does not reach it.
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        ::ntkrnl_sys::driver_ddk::DbgPrint(
            concat!("(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const i8
            $(, $arg)*
        )
    };
}

/// Silenced: `DPRINT` is what `NDEBUG` turns off, and every win32k C file this
/// crate translates carries `#define NDEBUG`.
macro_rules! DPRINT {
    ($($t:tt)*) => {
        ()
    };
}

// win32k's FIXME is NOT one of the always-on channels the scanner models it as:
// `DBG_PRINT` short-circuits only for ERR_LEVEL, and FIXME_LEVEL falls through to
// `ppi->DbgChannelLevel[ch]`, which stays ERR-only until a process is launched
// with DEBUGCHANNEL set. Silence is the faithful boot behaviour here, not a
// dropped diagnostic.
/// Channel-gated in the C, and no boot-path process opens the channel.
// nolint: scan_dropped_prints (channel-gated, ERR-only at boot — see above)
macro_rules! FIXME {
    ($($t:tt)*) => {
        ()
    };
}

/// Channel-gated in the C — see the module note.
macro_rules! WARN {
    ($($t:tt)*) => {
        ()
    };
}

/// Channel-gated in the C — see the module note.
macro_rules! TRACE {
    ($($t:tt)*) => {
        ()
    };
}
