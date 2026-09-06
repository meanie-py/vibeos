//! `DPRINT` / `DPRINT1` — the ReactOS debug macros (`sdk/include/reactos/debug.h`),
//! defined once for the crate rather than per module.
//!
//! `pdo.rs` carried `// DPRINT / DPRINT1 / DBGPRINT are no-ops under NDEBUG`
//! over three empty macros, and `pci.rs`/`fdo.rs` dropped their calls outright
//! on the same reasoning. It only holds for `DPRINT`: `#define NDEBUG` silences
//! that one, while `DPRINT1` sits outside the NDEBUG guard in debug.h and is
//! live whenever `DBG` is set. Fifteen pci.sys diagnostics — wrong BAR probe
//! sizes, failed range-length reads, unknown IOCTLs — were going nowhere.
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

/// Silenced: pci.c, fdo.c and pdo.c all carry `#define NDEBUG`.
macro_rules! DPRINT {
    ($($t:tt)*) => {
        ()
    };
}
