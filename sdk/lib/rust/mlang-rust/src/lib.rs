//! mlang-rust — MLang multi-language support (dll/win32/mlang/mlang.c).
//!
//! Mechanical C→Rust transliteration (DeepSeek v4-pro, banked 2026-06-07).
//! Banked pre-grind: drive to `cargo check` green with deepdrive (grind only,
//! do not re-translate). Substrate: ntdll-sys `nt` umbrella.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_assignments, unused_variables, unused_mut)]
#![allow(unused_parens, unused_unsafe, unused_imports)]

// Wine debug channel macros — translated as no-op `extern "C" fn`s, converted back
// to `macro_rules!` per AGENTS.md §"Translating a plain-C module", step 2.
// Defined at crate root (before module declarations) so they're in scope for all modules.
macro_rules! TRACE {
    ($($arg:tt)*) => {
        ()
    };
}
macro_rules! WARN {
    ($($arg:tt)*) => {
        ()
    };
}

#[path = "mlang/mlang.rs"]
mod mlang;
mod debug_shims;

// no_std staticlib needs its own panic handler + EH personality.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}

// ---- MIMECONTF constants (from mlang.h tagMIMECONTF enum) ----
pub type MIMECONTF = u32;
pub const MIMECONTF_MAILNEWS: MIMECONTF = 0x00000001;
pub const MIMECONTF_BROWSER: MIMECONTF = 0x00000002;
pub const MIMECONTF_MINIMAL: MIMECONTF = 0x00000004;
pub const MIMECONTF_IMPORT: MIMECONTF = 0x00000008;
pub const MIMECONTF_SAVABLE_MAILNEWS: MIMECONTF = 0x00000100;
pub const MIMECONTF_SAVABLE_BROWSER: MIMECONTF = 0x00000200;
pub const MIMECONTF_EXPORT: MIMECONTF = 0x00000400;
pub const MIMECONTF_PRIVCONVERTER: MIMECONTF = 0x00010000;
pub const MIMECONTF_VALID: MIMECONTF = 0x00020000;
pub const MIMECONTF_VALID_NLS: MIMECONTF = 0x00040000;
pub const MIMECONTF_MIME_IE4: MIMECONTF = 0x10000000;
pub const MIMECONTF_MIME_LATEST: MIMECONTF = 0x20000000;
pub const MIMECONTF_MIME_REGISTRY: MIMECONTF = 0x40000000;
