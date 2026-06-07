//! rtl-diag: Rust translation of small SEH-free RTL diagnostic / misc leaves.
//!
//! - `assert`       — `RtlAssert`.
//! - `appverifier`  — `RtlApplicationVerifierStop`.
//! - `trace`        — `RtlGetUnloadEventTrace` + the heap-trace database.
//! - `wine_debug`   — wine debug-channel + allocation-leak tracking helpers.
//!
//! Files are integrated one wave at a time; modules are gated in as each is
//! reconciled + smoke-validated on all three arches.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod shared;

#[cfg(feature = "cabi")]
pub mod assert;
#[cfg(feature = "cabi")]
pub mod appverifier;
#[cfg(feature = "cabi")]
pub mod trace;
// wine_debug — WIP: needs c_variadic (unstable) + the wine debug-channel
// substrate (__wine_dbg_* / debugstr_*); deferred to its own focused wave.
// #[cfg(feature = "cabi")]
// pub mod wine_debug;
