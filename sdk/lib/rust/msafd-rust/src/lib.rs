//! msafd.dll — Rust translation of dll/win32/msafd (the AFD Winsock service
//! provider). staticlib → libmsafd_rust.a, cohab-linked into the cmake-built
//! msafd.dll with the translated .c commented out. The next rung up the
//! netstack→winsock vertical above afd.sys: ws2_32's WSP* calls land here and
//! become IOCTL_AFD_* DeviceIoControls down to afd.sys. Nt*/Rtl* FFI from
//! ntdll-sys; winsock2/WSP + AFD shared.h IOCTL types from the `types` module.
//!
//! Per-file cohab flip (same model as kernel32-rust/afd-rust): each translated
//! misc/*.c file becomes a module here and its #[no_mangle] WSP*/helper symbols
//! take over from the commented-out C. Modules are wired in as each file lands.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

// Translated msafd modules + the winsock/WSP + AFD-shared substrate get declared
// here during integration (after the rustconv translation lands in src/).
// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate.
#[macro_use]
extern crate winedebug_rust;

pub mod types; // winsock2/WSP + wsahelp + AFD shared.h + msafd-private struct layouts
#[macro_use]
pub mod glue; // #define consts / type aliases / macro shims the substrate needs
pub mod misc; // the 5 translated translation units

// staticlib (libmsafd_rust.a) needs a panic handler + eh personality (no_std cohab),
// gated off for `cargo test` where std provides them. Mirrors afd-rust/cdfs-rust.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
