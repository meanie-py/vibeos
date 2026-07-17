//! vdm-rust — Virtual DOS Machine / 16-bit support (Vdm*), extracted from
//! ntoskrnl-rust (crate-split Phase 1b). i386-ONLY: the modules are gated on
//! `target_arch = "x86"`, so on amd64/arm64 this is a valid empty `#![no_std]` rlib
//! (kernel-rust's `extern crate vdm_rust` is itself cfg-gated to x86 to match).
//! NB: `#![no_std]` is UNCONDITIONAL — a crate-level `#![cfg(target_arch="x86")]`
//! strips `#![no_std]` on non-x86, making the empty crate demand std (E0463) — which
//! broke the amd64 build. Gate the modules, never the crate.
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_unsafe)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

#[cfg(target_arch = "x86")]
pub mod global_barrier;
#[cfg(target_arch = "x86")]
pub mod vdmexec;
#[cfg(target_arch = "x86")]
pub mod vdmmain;
