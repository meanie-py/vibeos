#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// ntlsalib.a — Rust translation of the kernel LSA client lib
// (sdk/lib/lsalib/lsa.c). crate-type=staticlib → liblsalib_rust.a, linked into
// ntoskrnl in place of the C `ntlsalib` (kept on disk). Own panic_handler absorbed
// by --allow-multiple-definition. DeepSeek+c2rust translation (Batch 4).
pub mod lsa;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
