//! concfg-rust — Console settings (settings.rs) + font cache (font.rs).
//!
//! Translated from win32ss/user/winsrv/concfg/{settings,font}.c (Opus 4.8).
//! concfg is a SHARED leaf static lib: linked into BOTH winsrv.dll (via consrv)
//! and dll/cpl/console/console.dll (the Console CPL applet). It therefore lives
//! in its own crate so the `concfg` cmake static lib can hand the Rust symbols
//! to every consumer (libconcfg_rust.a), while winsrv-rust depends on it as an
//! rlib for consrv's direct (`use crate::concfg::...`) calls.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

pub mod font;
pub mod settings;

// The standalone staticlib (libconcfg_rust.a → Console CPL) needs its own panic
// handler + eh_personality. When concfg-rust is instead an rlib dependency of
// winsrv-rust, THAT crate provides them — so gate these behind the `standalone`
// feature to avoid a duplicate-lang-item (panic_impl) / duplicate-symbol.
#[cfg(feature = "standalone")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(feature = "standalone")]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
