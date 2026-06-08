#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// libhidparser_km — Rust translation of the HID report-descriptor parser
// (sdk/lib/drivers/hidparser/*.c). crate-type=staticlib → libhidparser_rust.a,
// linked into hidparse.sys (Batch-1 Rust wrapper) in place of the C
// libhidparser_km. Public HIDP_* types come from hid-sys; kernel types from
// ntkrnl-sys. C kept on disk per never-delete. DeepSeek+c2rust (Batch 2).
//
// One C file → one module (rustconv per-file convention).
pub mod parser_h;
pub mod context;
pub mod hidp;
pub mod hidparser;
pub mod parser;
pub mod api;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
