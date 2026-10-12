// framebuf-rust — Rust translation of the generic framebuffer GDI display
// driver (win32ss/drivers/displays/framebuf/*.c).
//
// Cohabitation staticlib (sysfb-rust / hal-rust style): the C sources are
// commented out of win32ss/drivers/displays/framebuf/CMakeLists.txt and the
// #[no_mangle] symbols below take over. Links win32k (GDI engine exports:
// Eng*/PALOBJ_*/DDI types) + libcntpr at the cmake link.
//
// Each module is a self-contained translation of one C file: it carries its
// own #[repr(C)] PDEV and ALLOC_TAG. Cross-file calls (Int* helpers) go
// through extern FFI (raw pointers), so the duplicate repr(C) PDEV defs are
// ABI-compatible and need no shared type.
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(clippy::all)]

pub mod framebuf;
pub mod enable;
pub mod palette;
pub mod pointer;
pub mod screen;
pub mod surface;


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
