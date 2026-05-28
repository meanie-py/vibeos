//! Constants and FFI struct layout for `sdk/lib/rtl/resource.c`.
//!
//! Wave RES is pure plumbing — every fn chains `Rtl*CriticalSection`
//! and `Nt*Semaphore` / `NtClose` calls. There's no algorithmic
//! content to host in `core_rs`; this module just centralises the
//! `RTL_RESOURCE` layout and the few constants the cabi shims need.
//!
//! Faithful to ReactOS `sdk/lib/rtl/resource.c` (Wine-derived; Marcus
//! Meissner / Alex Korobka).

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

/// `SEMAPHORE_ALL_ACCESS` (winnt.h) — `STANDARD_RIGHTS_REQUIRED
/// (0xF0000) | SYNCHRONIZE (0x100000) | 0x3` = `0x1F0003`.
pub const SEMAPHORE_ALL_ACCESS: u32 = 0x1F_0003;

/// `STATUS_WAIT_0` — `NtWaitForSingleObject` success sentinel used by
/// `RtlAcquireResourceShared`'s "already woken by `RtlReleaseResource`"
/// branch (set when waking out of the shared-semaphore queue).
pub const STATUS_WAIT_0: i32 = 0;

/// `RTL_CRITICAL_SECTION` layout-match (ndk/rtltypes.h, NDK baseline).
/// `Lock` is embedded by value in `RTL_RESOURCE`; we just need the
/// right footprint + alignment for the per-arch packing of the
/// containing struct. ReactOS treats this as opaque from C too —
/// callers only pass `&Lock` to `RtlInitializeCriticalSection` etc.
#[repr(C)]
pub struct RTL_CRITICAL_SECTION_OPAQUE {
    pub DebugInfo: *mut u8,
    pub LockCount: i32,
    pub RecursionCount: i32,
    pub OwningThread: *mut u8,
    pub LockSemaphore: *mut u8,
    pub SpinCount: usize,
}

/// `RTL_RESOURCE` layout-match (ndk/rtltypes.h:1535). Lock at offset 0
/// is critical — `&Resource->Lock` is passed to the critsec APIs.
#[repr(C)]
pub struct RTL_RESOURCE {
    pub Lock: RTL_CRITICAL_SECTION_OPAQUE,
    pub SharedSemaphore: *mut u8,
    pub SharedWaiters: u32,
    pub ExclusiveSemaphore: *mut u8,
    pub ExclusiveWaiters: u32,
    pub NumberActive: i32,
    pub OwningThread: *mut u8,
    pub TimeoutBoost: u32,
    pub DebugInfo: *mut u8,
}

// --- TEB.ClientId.UniqueThread offset — per-arch. ClientId at
// TEB+0x20 (i386) / TEB+0x40 (amd64+arm64); UniqueThread is the
// second HANDLE in CLIENT_ID (+ pointer-width into ClientId).
#[cfg(target_arch = "x86")]
pub const TEB_UNIQUE_THREAD_OFFSET: usize = 0x20 + 4;
#[cfg(target_arch = "x86_64")]
pub const TEB_UNIQUE_THREAD_OFFSET: usize = 0x40 + 8;
#[cfg(target_arch = "aarch64")]
pub const TEB_UNIQUE_THREAD_OFFSET: usize = 0x40 + 8;
