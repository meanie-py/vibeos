//! Wine-debug macro shims (TRACE / FIXME / WARN / ERR), isolated in their own
//! crate on purpose — the drvtrace-shim recipe applied to usermode (§9.99).
//!
//! kernel32-rust call sites declare these C-variadic (the Wine originals are
//! macros); the defs are non-variadic no-ops. That is machine-safe (cdecl
//! caller-clean) but UB to LLVM when def and variadic decl share one LLVM
//! module: instcombine folds every such call to `ud2`. The workspace builds
//! with `lto = false`, so a separate crate is a separate LLVM module and the
//! pattern is fold-proof. Do NOT move these back in-crate, do NOT enable LTO
//! over them.
//!
//! Deliberately linked ONLY by wine-heritage usermode crates that do not
//! define their own TRACE family — the names are too generic for drvtrace-shim
//! (kernel drivers carry their own DPRINT/TRACE variants).
#![no_std]
#![allow(non_snake_case)]

macro_rules! wine_noop {
    ($($n:ident),* $(,)?) => { $(
        #[no_mangle]
        pub unsafe extern "C" fn $n(_fmt: *const i8) {}
    )* }
}

wine_noop!(TRACE, FIXME, WARN, ERR);
