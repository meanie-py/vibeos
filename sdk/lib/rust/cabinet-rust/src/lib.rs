//! cabinet-rust — Microsoft Cabinet (.cab) FCI/FDI (dll/win32/cabinet/*.c).
//!
//! Mechanical C→Rust transliteration (DeepSeek v4-pro, banked 2026-06-07).
//! Banked pre-grind: drive to `cargo check` green with deepdrive (grind only,
//! do not re-translate). Substrate: ntdll-sys `nt` umbrella.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_assignments, unused_variables, unused_mut)]
#![allow(unused_parens, unused_unsafe, unused_imports)]

// Pull libz-rs-sys's #[export_name] C symbols (deflateInit2/deflate/deflateEnd)
// into the cabinet.dll link — fci.rs calls them via its extern "C" block, but
// nothing references the crate at the Rust level, so force its inclusion.
extern crate libz_rs_sys;

#[path = "cabinet/types.rs"]
mod types;
#[path = "cabinet/cabinet_main.rs"]
mod cabinet_main;
#[path = "cabinet/fci.rs"]
mod fci;
#[path = "cabinet/fdi.rs"]
mod fdi;
#[path = "cabinet/stubs.rs"]
mod stubs;

// no_std staticlib needs its own panic handler + EH personality.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
