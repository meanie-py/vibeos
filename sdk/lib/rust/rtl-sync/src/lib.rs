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

// atomics + srw/condvar/runonce/locale are in BOTH the kernel/ntdll-main link
// surface (`cabi`) and the standalone ntdll_vista.dll forwarder (`vista`).
#[cfg(any(feature = "cabi", feature = "vista"))]
pub mod atomics;
#[cfg(any(feature = "cabi", feature = "vista"))]
pub mod srw;
#[cfg(any(feature = "cabi", feature = "vista"))]
pub mod condvar;
#[cfg(any(feature = "cabi", feature = "vista"))]
pub mod runonce;
#[cfg(any(feature = "cabi", feature = "vista"))]
pub mod locale;
// name.c (upstream rtl_vista): RtlIsNameInExpression. cabi only: it allocates through
// RtlpAllocateMemory, which ntdll.dll and ntoskrnl provide but ntdll_vista.dll does not.
#[cfg(feature = "cabi")]
pub mod name;

// critical + wait are NOT exported by ntdll_vista.dll — only the main `rtl`
// link surface (`cabi`) compiles them. Under `vista`, condvar imports
// RtlEnter/LeaveCriticalSection from ntdll.dll instead (see condvar.rs).
#[cfg(feature = "cabi")]
pub mod critical;
#[cfg(feature = "cabi")]
pub mod wait;

// waitaddr — RtlWaitOnAddress / RtlWakeAddress{All,Single}, backing Win32
// WaitOnAddress / WakeByAddress* (exported by kernelbase.dll, forwarded here).
// Like srw/condvar/runonce these are -version=0x600+ Rtl exports, so they live
// in BOTH the main ntdll link (`cabi`, stripped at the 0x502 baseline) and the
// standalone ntdll_vista.dll forwarder (`vista`, where they actually export).
#[cfg(any(feature = "cabi", feature = "vista"))]
pub mod waitaddr;

// NOTE: rtl-sync stays rlib-only. The standalone ntdll_vista.dll staticlib +
// its panic_handler live in the sibling `rtl-vista-rust` crate, which re-exports
// these `vista`-gated modules. (A staticlib crate-type here would force cargo to
// build the .a even as rtl-rust's rlib dep, demanding a panic_handler that would
// duplicate ntoskrnl/ntdll's under `cabi`.)
