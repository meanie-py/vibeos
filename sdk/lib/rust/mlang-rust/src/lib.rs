//! mlang-rust — MLang multi-language support (dll/win32/mlang/mlang.c).
//!
//! Mechanical C→Rust transliteration (DeepSeek v4-pro, banked 2026-06-07).
//! Banked pre-grind: drive to `cargo check` green with deepdrive (grind only,
//! do not re-translate). Substrate: ntdll-sys `nt` umbrella.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_assignments, unused_variables, unused_mut)]
#![allow(unused_parens, unused_unsafe, unused_imports)]

#[path = "mlang/mlang.rs"]
mod mlang;

// no_std staticlib needs its own panic handler + EH personality.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}
