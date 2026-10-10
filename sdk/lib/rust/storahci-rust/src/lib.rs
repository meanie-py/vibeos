#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]

//! storahci.sys — AHCI storport miniport, faithful C→Rust cohab translation
//! (rustconv opus + c2rust anchor) of drivers/storage/port/storahci/storahci.c.

// NT_ASSERT is a checked-build assertion (wdm.h): its operand is never evaluated in a free
// build. It was an extern `fn`, so every call evaluated its argument out of line (cachesup.rs
// documents a fault that caused). `debug_assert!` type-checks it and evaluates nothing here.
#[macro_export]
macro_rules! NT_ASSERT { ($($t:tt)*) => { ntkrnl_sys::nt_assert!($($t)*) }; }

// UNREFERENCED_PARAMETER is a macro in the DDK headers; an out-of-line no-op fn is an
// optimization barrier that can uncloak an ABI mismatch.
#[allow(unused_macros)]
macro_rules! UNREFERENCED_PARAMETER {
    ($($p:expr),* $(,)?) => {{ $( let _ = $p; )* }};
}

/// `AhciDebugPrint(format, ...)` — in C this was never a function at all, just a
/// DbgPrint macro (`storahci.h:85`). The translation turned it into a no-op
/// C-variadic *fn*, which stable Rust cannot define (`c_variadic`,
/// rust-lang#44930). A macro is the honest shape: identical no-op semantics,
/// arguments still evaluated, zero runtime cost — and no C shim needed, so
/// storahci stays free of hand-written C.
macro_rules! AhciDebugPrint {
    ($($arg:expr),* $(,)?) => {{ $( let _ = &$arg; )* }};
}

pub mod types;
pub mod glue;
pub mod storahci;

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
