//! Rust pool allocator — replaces ntoskrnl/mm/ARM3/expool.c and pool.c.
//!
//! This crate provides the kernel pool allocation subsystem:
//! - Executive pool: ExAllocatePoolWithTag, ExFreePoolWithTag, tag tracking
//! - Page-level pool: MiAllocatePoolPages, MiFreePoolPages, pool init
//!
//! Compiled as an rlib and linked into ntoskrnl via the kernel-rust umbrella crate.

#![no_std]
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    clippy::missing_safety_doc
)]

pub mod expool;
pub mod globals;
pub mod pool;
pub mod types;

