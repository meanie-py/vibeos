#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// pciide.sys — Rust translation of the PCI IDE bus mini-driver
// (drivers/storage/ide/pciide/pciide.c). crate-type=staticlib →
// libpciide_rust.a, linked into pciide.sys by pciide-cargo (reactos-link,
// ModuleKind::KernelDll, entry=DriverEntry@8, no exports). The C source is kept
// on disk (never-delete rule).

pub mod ide_h;
pub mod pciide;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
