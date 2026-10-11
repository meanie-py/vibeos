//! afd-rust — drivers/network/afd/afd → Rust (Sonnet translate).
//!
//! The Ancillary Function Driver: the kernel bridge from usermode Winsock
//! (ws2_32 → msafd) down to TDI, which netstack now provides
//! (\Device\Tcp|Udp|Ip). Keystone of the netstack→winsock vertical. Cohab flip
//! per .c file: each file's #[no_mangle] fns take over from the commented-out C,
//! cross-file calls + the C tdihelpers (tdi.c/tdiconn.c, kept C) resolve as
//! `extern "C"`. afd-private struct layouts come from `types` (per-arch bindgen);
//! kernel/TDI types from ntkrnl-sys's single ddk identity.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

// UNREFERENCED_PARAMETER is a macro in the DDK headers; an out-of-line no-op fn is an
// optimization barrier that can uncloak an ABI mismatch.
#[allow(unused_macros)]
macro_rules! UNREFERENCED_PARAMETER {
    ($($p:expr),* $(,)?) => {{ $( let _ = $p; )* }};
}

// ASSERT is debug_assert! (ntkrnl_sys::dbgassert): checked in a debug build, never
// evaluated in release -- the kernel's convention. It was an empty fn here, which
// never checked yet still evaluated its argument on every call.
macro_rules! ASSERT { ($($t:tt)*) => { ntkrnl_sys::nt_assert!($($t)*) }; }

pub mod types; // afd-private + TDI/shared struct layouts (gen_driver_types.sh, per-arch)
#[macro_use] // AFD_DbgPrint! no-op macro must be in scope for the modules below
pub mod glue; // #define consts/type-aliases/macros bindgen doesn't expand

// The 10 compiled translation units, mirroring drivers/network/afd/CMakeLists.txt
// SOURCE exactly. NOTE: event.c is NOT in the C build (vestigial: AfdRegisterEventHandlers
// is defined only there and called nowhere; the live event wiring is TdiSetEventHandler
// in connect/listen). event.rs is kept as the paid translation but is intentionally NOT
// a module here — matching the C build, not dodging a regression. The TDI helper TUs
// (tdihelpers/tdi.c, tdiconn.c) stay C and resolve via extern "C".
pub mod bind;
pub mod connect;
pub mod context;
pub mod info;
pub mod listen;
pub mod lock;
pub mod main;
pub mod read;
pub mod select;
pub mod write;

// staticlib (libafd_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them. Mirrors cdfs-rust/disk-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
