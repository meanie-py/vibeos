#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// condrv.sys — Rust translation of the Console Driver
// (drivers/base/condrv/{condrv,control,dispatch}.c). crate-type=staticlib →
// libcondrv_rust.a, linked into condrv.sys by condrv-cargo (reactos-link,
// ModuleKind::KernelDll, entry=DriverEntry@8, no exports). The C sources are
// kept on disk (never-delete rule). One C file → one module.

pub mod condrv_h;
pub mod condrv;
pub mod control;
pub mod dispatch;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
