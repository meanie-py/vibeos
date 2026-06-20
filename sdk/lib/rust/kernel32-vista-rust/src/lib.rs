#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_parens, unused_mut, unused_assignments)]
#![allow(static_mut_refs)]
#![allow(clippy::all)]

//! kernel32-vista-rust — Rust translation of dll/win32/kernel32/kernel32_vista/*.c
//! (the kernel32_vista_static NT6/Vista+ API set). staticlib → libkernel32_vista_rust.a,
//! linked into BOTH kernel32.dll and kernel32_vista.dll (C kernel32_vista_static dropped).
//! kernel32_vista's DllMain.c stays C (collision: this .a links into kernel32.dll too).
//! See docs/engineering/kernel32-shared-vista-rust-plan.md.

pub mod common;
pub mod localenametolcid_table;

pub mod firmware;
pub mod locale;
pub mod misc;
pub mod sync;

// no_std staticlib floor. Duplicates kernel32-rust's handler when both .a link into
// kernel32.dll — resolved by the link's --allow-multiple-definition (first wins).
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
