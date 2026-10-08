#![no_std]
// pci/id.rs translates id.c's PciIdPrintf/PciIdPrintfAppend, which are true C variadic
// functions (va_start/vsprintf). Stable Rust cannot DEFINE a C-variadic fn (`c_variadic`,
// rust-lang#44930), so their variadic ENTRY points live in
// drivers/bus/pcix/rust_varargs_shim.c as 3-line trampolines into the `*_v` twins in
// pci/id.rs — all logic stays in Rust. This crate builds on PLAIN STABLE rustc: no
// RUSTC_BOOTSTRAP, no nightly feature gate, no build.rs cfg probe.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(clippy::all)]

// PAGED_CODE() is the DDK's "pageable code" IRQL assertion (xdk/kefuncs.h): NOP_FUNCTION in
// a free build, and a no-op here. It is a macro, never a fn -- an out-of-line no-op call is an
// optimization barrier, and an `extern` one resolved by whichever crate happened to export it.
macro_rules! PAGED_CODE {
    () => {};
}

// pcix.sys — Rust translation of the full MS-architecture PCI bus driver
// (drivers/bus/pcix). staticlib -> libpcix_rust.a, cohab-linked into the cmake pcix.sys
// with the translated .c commented out. NOT loaded at boot; bar = link-clean ×3.
// guid.c + the 4 empty stub TUs (usage/ideintrf/pci-devhere/pci-romimage) stay C.

pub mod types;
pub mod glue;
pub mod guids;
pub mod ddk_shims;

// subsystem dirs
pub mod arb;
pub mod intrface;
pub mod pci;

// top-level modules
pub mod debug;
pub mod device;
pub mod dispatch;
pub mod enumr;
pub mod fdo;
pub mod hookhal;
pub mod init;
pub mod pcivrify;
pub mod pdo;
pub mod power;
pub mod utils;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
