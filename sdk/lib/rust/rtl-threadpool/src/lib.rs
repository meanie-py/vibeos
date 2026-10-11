//! rtl-threadpool: Rust translation of the ReactOS RTL thread-pool cluster.
//!
//! - `threadpool` — the Wine object-oriented thread pool (`TpAlloc*`,
//!   `TpPostWork`, the new timer/wait/io queues) plus the old
//!   `RtlQueueWorkItem` / `RtlSetIoCompletionCallback` / `RtlCreateTimer`
//!   entry points layered on top of it.
//! - `workitem` — the legacy worker-thread pool (`RtlpStartThreadFunc` /
//!   `RtlpExitThreadFunc` globals, IO worker threads).
//! - `timerqueue` — `RtlCreateTimerQueue` / `RtlCreateTimer` / `RtlDeleteTimer`.
//!
//! The internal pool/object/timer structs are file-private (heap-allocated,
//! cast from the opaque public `TP_*` handles); `types` mirrors the PUBLIC
//! winnt `TP_*` types byte-exact since callers pass them across the ABI.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// Wine's ERR/FIXME/WARN/TRACE, shared by every Wine-derived crate (only the `cabi`
// modules log).
#[cfg(feature = "cabi")]
#[macro_use]
extern crate winedebug_rust;

pub mod types;
#[macro_use]
pub mod wine_list;

#[cfg(feature = "cabi")]
pub mod threadpool;
#[cfg(feature = "cabi")]
pub mod workitem;
#[cfg(feature = "cabi")]
pub mod timerqueue;
