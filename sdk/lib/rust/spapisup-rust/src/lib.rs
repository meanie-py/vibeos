//! spapisup — Rust translation of base/setup/usetup/spapisup/{cabinet,fileqsup}.c.
//!
//! usetup.exe compiles the two files; reactos.exe `#include`s them
//! (base/setup/reactos/spapisup/fileqsup.c), so they were translated into both crates. This
//! rlib is the one copy: usetup-rust and reactos-rust link it, and setuplib reaches the queue
//! through the `SpFileExports` table it defines. The queue functions are not `#[no_mangle]`:
//! only the table names them, and reactos.exe also imports setupapi's functions of the same
//! names (its C renamed its copies `_SetupOpenFileQueue` & co. for that reason).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh spapisup)
#[macro_use]
pub mod prelude;

pub mod spapisup {
    pub mod cabinet;
    pub mod fileqsup;
}

pub use spapisup::fileqsup::SpFileExports;
