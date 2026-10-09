#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

//! acpi-rust — Rust translation of the ReactOS ACPI driver glue
//! (drivers/bus/acpi). Phase 2 of the ACPICA FFI-to-blob plan: Intel's ACPICA
//! reference impl stays a C static lib (libacpica.a); only the OS-services glue
//! becomes Rust. This crate begins with the OS Services Layer (osl.c) — the 46
//! `AcpiOs*` callbacks ACPICA calls into.
//!
//! crate-type=staticlib → libacpi_rust.a, linked into the cmake-built acpi.sys
//! with osl.c commented out of ACPI_SOURCE (processr/pl011-style cohab; the C is
//! kept on disk per the never-delete rule). ACPI types/exports come from
//! acpica-sys; kernel calls from ntkrnl-sys.
//!
//! `AcpiOsPrintf` (C-variadic) stays a tiny C shim (acpi_printf_shim.c); under
//! the driver's NDEBUG build `AcpiOsVprintf` is a no-op anyway.

// ASSERT is debug_assert! (ntkrnl_sys::dbgassert): checked in a debug build, never
// evaluated in release -- the kernel's convention. It was an empty fn here, which
// never checked yet still evaluated its argument on every call.
macro_rules! ASSERTMSG { ($($t:tt)*) => { ntkrnl_sys::nt_assertmsg!($($t)*) }; }

// PAGED_CODE() is the DDK's "pageable code" IRQL assertion (xdk/kefuncs.h): NOP_FUNCTION in
// a free build, and a no-op here. It is a macro, never a fn -- an out-of-line no-op call is an
// optimization barrier, and an `extern` one resolved by whichever crate happened to export it.
macro_rules! PAGED_CODE {
    () => {};
}

// DPRINT/DPRINT1 for the whole crate; `#[macro_use]` must precede the modules
// that call them (macro_rules! is textually scoped).
#[macro_use]
mod dprint;

pub mod types;
pub mod shared;
pub mod osl;
pub mod bus;
pub mod buspdo;
// Phase 3b glue leaves (hand-translated): the remaining ReactOS ACPI driver
// glue. #[no_mangle] symbols replace the cohab-flipped C files in acpi.sys.
pub mod interface;
pub mod power;
pub mod acpienum;
pub mod button;
pub mod system;
pub mod pnp;
pub mod utils;
pub mod busmgr_power;
pub mod eval;
pub mod driver_main;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
