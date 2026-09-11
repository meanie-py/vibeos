//! `DPRINT` / `DPRINT1` — the ReactOS debug macros (`sdk/include/reactos/debug.h`),
//! defined once for the crate instead of re-declared in nine modules.
//!
//! Every module used to carry `// DPRINT/DPRINT1/... are no-ops under the
//! driver's NDEBUG build` over a pair of empty macros. Half of that is wrong:
//! `#define NDEBUG` silences `DPRINT` only. `DPRINT1` sits outside the NDEBUG
//! guard in debug.h and is live whenever `DBG` is set — which is exactly how
//! acpi.sys's failure paths reach the debug log, and how you find out why ACPI
//! enumeration went wrong. Translating it to nothing deleted 44 diagnostics.
//!
//! Both stay `macro_rules!`. A C debug macro lowered to an out-of-line `fn` is
//! an optimisation barrier that uncloaks latent ABI mismatches (AGENTS.md).
//!
//! The C macro builds its `"(%s:%d) "` prefix from `__RELFILE__`/`__LINE__` as
//! two extra varargs at every call. `file!()`/`line!()` fold into the literal
//! at compile time instead, so the call site passes exactly the varargs the C
//! did and the format string needs no `%s:%d` of its own.

/// Unconditional under `DBG` — `NDEBUG` does not reach it (debug.h).
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        ::ntkrnl_sys::driver_ddk::DbgPrint(
            concat!("(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const i8
            $(, $arg)*
        )
    };
}

/// Silenced: every acpi C file this crate translates carries `#define NDEBUG`.
macro_rules! DPRINT {
    ($($t:tt)*) => {
        ()
    };
}
