//! What `rustconv data-port`'s generated modules (src/data) import as `crate::prelude::*`:
//! the island's types, and the kernel base types from `ntkrnl_sys::ddk` -- the same glob
//! the island itself resolves them through, so ULONG_PTR is the island's fixed-width
//! integer, not ntkrnl-sys `base`'s usize. cdrom-rust predates the pipeline's prelude; its
//! hand-ported modules keep their own `use` lists.
#![allow(unused_imports)]
pub use crate::types::*;
pub use kmdf_sys::*;
pub use ntkrnl_sys::ddk::*;
/// The C `size_t` the generated WDF context sizes are cast to.
pub type size_t = usize;
