//! cryptlib-rust — Rust translation of sdk/lib/cryptlib.
//! Cohabitation staticlib (libcryptlib_rust.a): the cryptlib cmake target
//! REMOVE_ITEMs each flipped .c and links this archive.
//!
//! 8 modules mirror the 8 .c source files:
//!   aes/des (LibTomCrypt, cdecl), md4/md5/sha1 (NTAPI/system), rc4 (cdecl),
//!   mvAesAlg (Marvell Rijndael ref impl, cdecl), util (byteReverse, LE no-op).
//!
//! Mirrors rossym-rust / cmlib-rust: one #[panic_handler] + eh_personality here.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals,
         unused_variables, dead_code, unused_mut, unused_unsafe)]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

mod types;
mod util;
mod aes;
mod des;
mod md4;
mod md5;
mod sha1;
mod rc4;
mod mvaesalg;
