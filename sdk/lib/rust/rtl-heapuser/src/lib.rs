//! rtl-heapuser — sdk/lib/rtl/heapuser.c (heap process-list libsupp). Banked
//! translation, deepdrive-ground. Note: kernel/user libsupp split.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#[cfg(feature = "cabi")]
pub mod heapuser;
