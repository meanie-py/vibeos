//! rtl-heap: Rust translation of ReactOS `sdk/lib/rtl/heap.c` — the NT
//! process-heap allocator (`Rtl{Create,Allocate,Free,ReAllocate,Size}Heap`
//! and the `Rtlp*` internals).
//!
//! Unlike rtl-strings there is no pure `core_rs` split: the heap is
//! inherently pointer-walking over the in-band `HEAP_ENTRY` headers, so
//! the whole body is `unsafe` cabi. `types.rs` is the byte-exact mirror
//! of `heap.h`; heappage.c / heapdbg.c remain C and share that layout
//! across the cohab boundary (hence the compile-time size assertions).
//!
//! Semantics track ReactOS rtl, not textbook.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod types;

#[cfg(feature = "cabi")]
pub mod heap;
