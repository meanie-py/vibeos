#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_parens, unused_mut, unused_assignments)]
#![allow(clippy::all)]

//! kernel32-vista-rust — Rust translation of dll/win32/kernel32/kernel32_vista/*.c
//! (the kernel32_vista_static NT6/Vista+ API set). staticlib → libkernel32_vista_rust.a,
//! linked into BOTH kernel32.dll and kernel32_vista.dll (C kernel32_vista_static dropped).
//! kernel32_vista's DllMain.c stays C (collision: this .a links into kernel32.dll too).
//! See docs/engineering/kernel32-shared-vista-rust-plan.md.

// Crate-wide `w!("...")` — compile-time NUL-terminated UTF-16 literal (same as kernel32-rust).
#[macro_export]
macro_rules! w {
    ($s:literal) => {{
        const __B: &[u8] = $s.as_bytes();
        const __N: usize = __B.len();
        static __W: [u16; __N + 1] = {
            let mut out = [0u16; __N + 1];
            let mut i = 0;
            while i < __N {
                out[i] = __B[i] as u16;
                i += 1;
            }
            out
        };
        &__W
    }};
}

pub mod common;
pub mod localenametolcid_table;

pub mod enumlocales;
pub mod fileinfo;
pub mod finalpath;
pub mod firmware;
pub mod locale;
pub mod misc;
pub mod sync;
pub mod threadpool;
pub mod vista;

// no_std staticlib floor. Duplicates kernel32-rust's handler when both .a link into
// kernel32.dll — resolved by the link's --allow-multiple-definition (first wins).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
