//! usbaudio-rust — Rust translation of the USB Audio class driver (drivers/usb/usbaudio).
//! KS/PortCls miniport; cohab usbaudio.sys. NOT boot-loaded -> link-clean x3 is the bar.
//! DeepSeek-chat (V3) + c2rust-anchored; constant-safeguard verified clean
//! (verify_constants 0 mismatches, scan_inline_flag_literals clean, Haiku audit CLEAN x4).
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]

// Coerce an `extern "C"` Rust fn item into a bindgen NTAPI callback slot (`Option<unsafe
// extern "stdcall" fn ...>` on i386; `extern "C"` on amd64/arm64). The fn-pointer bit pattern
// is identical; `Some(f) as PSLOT` is an invalid Option→Option cast, so transmute the address.
#[allow(unused_macros)]
macro_rules! cb {
    ($f:expr) => {
        core::mem::transmute($f as usize)
    };
}
pub(crate) use cb;

pub mod types;
pub mod glue;

// Translated function modules (DeepSeek+c2rust). The KS/PortCls dispatch tables +
// #[no_mangle] DriverEntry/AddDevice take over the cohab usbaudio.sys link.
pub mod pool;
pub mod usbaudio;
pub mod pin;
pub mod filter;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
