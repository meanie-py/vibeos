//! skiplist-rust — Rust translation of sdk/lib/skiplist/skiplist.c.
//! The C lib is built as `skiplist16` with `-DSKIPLIST_LEVELS=16`.
//! DeepSeek mechanical translation, deepdrive-ground to compile.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod skiplist;

// staticlib (lib<name>_rust.a) needs its own panic_handler/eh_personality;
// absorbed by --allow-multiple-definition at the consumer link (cohab pattern,
// mirrors lsalib-rust). Gated on not(test) so host unit tests stay std.
#[cfg(not(test))]
#[panic_handler]
fn rust_cohab_panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
