//! Inline implementations of C FORCEINLINE/macro functions.
//!
//! These functions replace the `*_rust_shims.c` C trampoline files with native
//! Rust implementations. Each submodule corresponds to a kernel subsystem.
//! Export wrappers (`#[no_mangle]`) live in `ntoskrnl-rust/src/shim_exports/`.

pub mod se;
pub mod common;
pub mod ke;
pub mod mm;
pub mod ex;
pub mod ob;
pub mod ps;
pub mod tier1;
pub mod ke_amd64;
pub mod ke_i386;
