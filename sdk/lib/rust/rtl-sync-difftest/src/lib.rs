//! The shipping rtl-sync SRW lock, compiled for the host against `fake-nt` (see Cargo.toml).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#![allow(clippy::all)]

#[path = "../../rtl-sync/src/atomics.rs"]
pub mod atomics;
#[path = "../../rtl-sync/src/srw.rs"]
pub mod srw;

pub use ntkrnl_sys::ddk::{KEYED_RELEASES, KEYED_WAITS, RTL_SRWLOCK};
