#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// pci.sys — Rust translation of the ReactOS PCI bus driver
// (drivers/bus/pci/{pci,fdo,pdo}.c). crate-type=staticlib → libpci_rust.a, linked
// into the cmake-built pci.sys with the .c files commented out (kept on disk per
// the never-delete rule). Standalone kernel-mode bus driver on the ntkrnl-sys
// substrate, built for all 3 arches.
//
// One C file → one module (rustconv per-file convention). `shared` holds the
// driver-private device extensions (pci.h) used across all three modules, plus
// the module-global DriverExtension / HasDebuggingDevice / PciDebuggingDevice.
pub mod shared;
pub mod pci;
pub mod fdo;
pub mod pdo;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
