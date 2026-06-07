//! rtl-data: deepdrive batch #3 — banked Opus translations of rtl/ data-structure leaves.
//!
//! Each module is the raw Opus draft on first commit (the grind baseline);
//! DeepSeek drives `cargo check -p rtl-data --features cabi` to green, the
//! frontier driver reviews + smokes + wires it into the build.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod shared;

#[cfg(feature = "cabi")]
pub mod rangelist;
#[cfg(feature = "cabi")]
pub mod rxact;
#[cfg(feature = "cabi")]
pub mod bootdata;
