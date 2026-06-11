//! rtl-heappage — sdk/lib/rtl/heappage.c (page/debug heap). Banked translation,
//! deepdrive-ground to compile.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#[cfg(feature = "cabi")]
pub mod heappage;
