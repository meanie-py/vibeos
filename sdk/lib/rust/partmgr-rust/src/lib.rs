#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(clippy::all)]

// partmgr.sys — Rust translation of the partition manager driver
// (drivers/storage/partmgr). staticlib -> libpartmgr_rust.a, cohab-linked into the
// cmake partmgr.sys with the translated .c (partition.c/partmgr.c/utils.c) commented
// out. guid.c stays C (DEFINE_GUID data symbols, declared extern in guids.rs).

pub mod types;
pub mod guids;
pub mod glue;
pub mod inlines;
pub mod ddk_shims;

pub mod partition;
pub mod partmgr;
pub mod utils;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
