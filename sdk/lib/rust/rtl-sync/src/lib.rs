//! rtl-sync: Rust translation of the ReactOS RTL synchronization cluster.
//!
//! - `critical`  — RTL critical sections (`RtlEnter/LeaveCriticalSection`,
//!   `RtlInitializeCriticalSectionEx`, keyed-event contention path).
//! - `wait`      — `RtlRegisterWait` / `RtlDeregisterWait` thread-pool waits.
//! - `srw`       — slim reader/writer locks (`RtlAcquireSRWLock*`).      [rtl_vista]
//! - `condvar`   — condition variables (`RtlSleep/WakeConditionVariable`). [rtl_vista]
//! - `runonce`   — `RtlRunOnce*` one-time initialization.                  [rtl_vista]
//!
//! Files are integrated one wave at a time; modules are gated in as each is
//! reconciled + smoke-validated on all three arches.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod shared;

#[cfg(feature = "cabi")]
pub mod critical;
#[cfg(feature = "cabi")]
pub mod wait;
