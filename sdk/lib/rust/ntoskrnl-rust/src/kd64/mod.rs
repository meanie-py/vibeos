//! kd64 subsystem — translated from C.

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
