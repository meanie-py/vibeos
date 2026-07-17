//! kd64-rust — the WinDbg KD protocol stack (Kd*/Kdp*), extracted from ntoskrnl-rust
//! (crate-split Phase 1b). Active on all 3 arches. #[no_mangle] symbols link via
//! `extern crate kd64_rust`. No per-crate barrier (see kd-rust note).
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

pub use dbgprint_rust::kdp_dprintf;

// abi_cast! — cast an extern "system" fn to the target callback ABI (i386 stdcall /
// amd64+arm64 C). Crate-local copy of ntoskrnl-rust's (not exported).
#[allow(unused_macros)]
macro_rules! abi_cast {
    ($func:expr) => {
        unsafe { core::mem::transmute($func as usize) }
    };
}

#[cfg(target_arch = "x86")]
pub mod i386;
#[cfg(target_arch = "x86_64")]
pub mod amd64;

pub mod kdapi;
pub mod kdbreak;
pub mod kddata;
pub mod kdinit;
pub mod kdlock;
pub mod kdprint;
pub mod kdtrap;
