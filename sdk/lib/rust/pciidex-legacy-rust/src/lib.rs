#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(clippy::all)]

// UNREFERENCED_PARAMETER is a macro in the DDK headers; an out-of-line no-op fn is an
// optimization barrier that can uncloak an ABI mismatch.
#[allow(unused_macros)]
macro_rules! UNREFERENCED_PARAMETER {
    ($($p:expr),* $(,)?) => {{ $( let _ = $p; )* }};
}

// PAGED_CODE() is the DDK's "pageable code" IRQL assertion (xdk/kefuncs.h): NOP_FUNCTION in
// a free build, and a no-op here. It is a macro, never a fn -- an out-of-line no-op call is an
// optimization barrier, and an `extern` one resolved by whichever crate happened to export it.
macro_rules! PAGED_CODE {
    () => {};
}

// pciidex.sys — Rust translation of the PCI IDE bus extension (drivers/storage/ide/
// pciidex). staticlib -> libpciidex_rust.a, cohab-linked into the cmake pciidex.sys with
// the 5 .c commented out + spec2def kept. EXPORT driver: PciIdeXInitialize/GetBusData/
// SetBusData/DebugPrint come from the #[no_mangle] symbols here. Boot-loaded + exercised
// via uniata.sys / pciide.sys PciIdeX* imports.
//
// The three C-variadic entry points (RtlStringCbPrintfW / RtlStringCchPrintfExW /
// PciIdeXDebugPrint) live in drivers/storage/ide/pciidex/rust_varargs_shim.c — stable Rust
// cannot `va_start` (c_variadic, rust-lang#44930). Each is a 3-line trampoline into the
// `*_v` twin here; all logic stays in Rust. This crate builds on PLAIN STABLE rustc — no
// RUSTC_BOOTSTRAP, no nightly feature gate.

pub mod types;
pub mod glue;
pub mod inlines;
pub mod ddk_shims;

pub mod fdo;
pub mod miniport;
pub mod pciidex;
pub mod pdo;
pub mod power;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
