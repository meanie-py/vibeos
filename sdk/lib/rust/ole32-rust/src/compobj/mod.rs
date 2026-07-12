//! ole32 Lane A — compobj.c translation (dll/win32/ole32/compobj.c).
//!
//! Module wiring for the integrator: these four files are siblings under one
//! `ole32` module of the future `ole32-rust` crate. `com.rs` is the single
//! swap-target module for the future bindgen `com-sys` crate (interface vtbls,
//! IIDs, HRESULTs); `ffi.rs` is the Win32/NT/CRT import surface + tiny helpers;
//! `types.rs` mirrors compobj_private.h private types + wine/list.h.
//!
//! See ~/vibeos-rust-staging/ole32-lane-a/MANIFEST.md for the full ledger.

pub mod com;
pub mod ffi;
pub mod types;
pub mod compobj;
