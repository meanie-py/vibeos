//! ntkrnl-sys — FFI declarations for the ReactOS/NT kernel.
//!
//! Provides `#[repr(C)]` types and `extern "system"` function declarations
//! for kernel subsystems. Used as a compile-time dependency by Rust kernel
//! components (pool allocator, etc.).
//!
//! Types were derived from ReactOS NDK headers (`sdk/include/ndk/ntndk.h`)
//! and internal kernel headers (`ntoskrnl/mm/ARM3/miarm.h`).
//!
//! NOT built as a standalone static library — this is an `rlib` dependency.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)]

pub mod base;
pub mod ex;
pub mod interlocked;
pub mod ke;
pub mod mm;
pub mod ob;
pub mod ps;
pub mod rtl;

// Re-export commonly used types at crate root
pub use base::*;
