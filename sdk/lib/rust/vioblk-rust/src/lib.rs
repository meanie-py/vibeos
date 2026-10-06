//! vioblk-rust — the virtio-blk SCSIPORT miniport (vioblk.sys). See Cargo.toml.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, improper_ctypes, improper_ctypes_definitions)]
// miniport::ffi redeclares the cdecl-varargs ScsiPortNotification on purpose: the
// substrate emits it inside an `extern "system"` block, the wrong ABI for i386 varargs.
#![allow(clashing_extern_declarations)]

pub mod hal;
pub mod miniport;
pub mod pci;
#[allow(unused_imports, unused_variables, unused_mut, clippy::all)]
pub mod types;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
