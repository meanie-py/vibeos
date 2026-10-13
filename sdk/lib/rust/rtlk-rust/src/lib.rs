//! rtlk-rust — kernel RTL glue (Rtl* path/libsupp/misc), extracted from ntoskrnl-rust
//! (crate-split Phase 1b). Named rtlk to avoid colliding with the user-mode rtl-rust
//! algorithm umbrella. #[no_mangle] Rtl* symbols link via `extern crate rtlk_rust`.
//! No per-crate barrier (see kd-rust note). path.rs is carried unwired (it is dead in
//! the monolith too — no `mod path` anywhere; its Rtl* come from rtl_misc/rtl_strings).
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

pub mod libsupp;
pub mod misc;
pub mod feature;
#[cfg(target_arch = "x86")]
pub mod prefetch_asm;
