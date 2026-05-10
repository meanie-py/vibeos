//! Folded back from standalone `pool` crate 2026-05-09.
//! Was: `sdk/lib/rust/pool/`. Now: a submodule of ntoskrnl-rust::ex.
//! Rust pool allocator — replaces ntoskrnl/mm/ARM3/expool.c and pool.c.
//!
//! This crate provides the kernel pool allocation subsystem:
//! - Executive pool: ExAllocatePoolWithTag, ExFreePoolWithTag, tag tracking
//! - Page-level pool: MiAllocatePoolPages, MiFreePoolPages, pool init
//!


pub mod expool;
pub mod globals;
pub mod pool;
pub mod types;

