//! Rust pool allocator — replaces ntoskrnl/mm/ARM3/expool.c and pool.c.
//!
//! This crate provides the kernel pool allocation subsystem:
//! - Executive pool: ExAllocatePoolWithTag, ExFreePoolWithTag, tag tracking
//! - Page-level pool: MiAllocatePoolPages, MiFreePoolPages, pool init
//!
//! Compiled as a `staticlib` and linked directly into ntoskrnl.

#![no_std]
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    clippy::missing_safety_doc
)]

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub mod expool;
pub mod globals;
pub mod pool;
pub mod types;
