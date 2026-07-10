#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// csq.a — Rust translation of the Cancel-Safe IRP Queue library
// (sdk/lib/drivers/csq/csq.c). crate-type=staticlib → libcsq_rust.a, linked into
// ntoskrnl in place of the C `csq` lib (kept on disk per the never-delete rule).
// DeepSeek+c2rust translation (Batch 4 kernel-lib residue).
pub mod csq;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
