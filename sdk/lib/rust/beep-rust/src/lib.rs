#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// beep.sys — Rust translation of the Beep device driver
// (drivers/base/beep/beep.c). crate-type=staticlib → libbeep_rust.a, linked
// into beep.sys by beep-cargo (reactos-link, ModuleKind::KernelDll,
// entry=DriverEntry@8, no exports). The C source is kept on disk (never-delete
// rule). One C file → one module (rustconv per-file convention).

pub mod beep_h;
pub mod beep;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
