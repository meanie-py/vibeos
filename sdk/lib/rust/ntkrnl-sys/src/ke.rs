//! Kernel executive types and extern declarations.
//!
//! Derived from `sdk/include/ndk/ketypes.h` and `sdk/include/xdk/ketypes.h`.

use crate::base::*;

// ============================================================================
// KSPIN_LOCK
// ============================================================================

pub type KSPIN_LOCK = ULONG_PTR;
pub type PKSPIN_LOCK = *mut KSPIN_LOCK;

// ============================================================================
// KGUARDED_MUTEX — opaque, size varies by architecture
// ============================================================================

/// KGUARDED_MUTEX is used for paged pool synchronization.
/// Layout: Count(4) + [pad] + Owner(ptr) + Contention(4) + [pad] + Gate(DISPATCHER_HEADER) + CombinedApcDisable(4) + [pad]
/// We define it as an opaque byte array with correct size and alignment.
///
/// DISPATCHER_HEADER: Lock(4) + SignalState(4) + WaitListHead(2*ptr)
/// On 64-bit: 4+4+8+4+4+24+4+4 = 56 bytes
/// On 32-bit: 4+4+4+16+4 = 32 bytes
#[cfg(target_pointer_width = "64")]
#[repr(C, align(8))]
pub struct KGUARDED_MUTEX {
    pub _opaque: [u8; 56],
}

#[cfg(target_pointer_width = "32")]
#[repr(C, align(4))]
pub struct KGUARDED_MUTEX {
    pub _opaque: [u8; 32],
}

pub type PKGUARDED_MUTEX = *mut KGUARDED_MUTEX;

impl KGUARDED_MUTEX {
    pub const fn zeroed() -> Self {
        #[cfg(target_pointer_width = "64")]
        { Self { _opaque: [0; 56] } }
        #[cfg(target_pointer_width = "32")]
        { Self { _opaque: [0; 32] } }
    }
}

// ============================================================================
// KDPC — Deferred Procedure Call object (opaque)
// ============================================================================

/// KDPC size: 64-bit = 64 bytes, 32-bit = 32 bytes (approximate).
#[cfg(target_pointer_width = "64")]
#[repr(C, align(8))]
pub struct KDPC {
    pub _opaque: [u8; 64],
}

#[cfg(target_pointer_width = "32")]
#[repr(C, align(4))]
pub struct KDPC {
    pub _opaque: [u8; 32],
}

pub type PKDPC = *mut KDPC;

// ============================================================================
// KLOCK_QUEUE_HANDLE — re-export full definition from ddk (bindgen)
// ============================================================================

// Re-export the full struct from ddk so all consumers can access fields.
pub use super::ddk::_KLOCK_QUEUE_HANDLE as KLOCK_QUEUE_HANDLE;

pub type PKLOCK_QUEUE_HANDLE = *mut KLOCK_QUEUE_HANDLE;

// ============================================================================
// KPRCB — re-export full definition from ddk (bindgen)
// ============================================================================

// Re-export the full struct from ddk so KE subsystem code can access fields.
pub use super::ddk::_KPRCB as KPRCB;

pub type PKPRCB = *mut KPRCB;

// ============================================================================
// GENERAL_LOOKASIDE — lookaside list descriptor
// ============================================================================

/// PP_LOOKASIDE_LIST — per-processor lookaside list pair.
#[repr(C)]
pub struct PP_LOOKASIDE_LIST {
    pub P: *mut GENERAL_LOOKASIDE,
    pub L: *mut GENERAL_LOOKASIDE,
}

/// GENERAL_LOOKASIDE — cache-aligned lookaside list.
/// We define the fields the pool allocator accesses directly.
#[repr(C)]
pub struct GENERAL_LOOKASIDE {
    pub ListHead: SLIST_HEADER,
    pub Depth: USHORT,
    pub MaximumDepth: USHORT,
    pub TotalAllocates: ULONG,
    pub AllocateHits: ULONG, // union with AllocateMisses
    pub TotalFrees: ULONG,
    pub FreeHits: ULONG,     // union with FreeMisses
    pub Type: ULONG,         // POOL_TYPE
    pub Tag: ULONG,
    pub Size: ULONG,
    // Remaining fields (function pointers, list entry, etc.) are not accessed
    // by the pool allocator. Pad to the REAL C sizeof (verified by
    // size_check_<arch>.rs): 128 on 64-bit (LOOKASIDE_ALIGN cache line),
    // 72 on i386. (Was over-padded to 192/128 — harmless only because
    // nothing shares arrays of this type with C.)
    #[cfg(target_pointer_width = "64")]
    pub _pad: [u8; 80], // total 128 bytes
    #[cfg(target_pointer_width = "32")]
    pub _pad: [u8; 32], // total 72 bytes
}

pub type PGENERAL_LOOKASIDE = *mut GENERAL_LOOKASIDE;

/// NUMBER_POOL_LOOKASIDE_LISTS — max block size eligible for lookaside.
pub const NUMBER_POOL_LOOKASIDE_LISTS: u16 = 32;

// ============================================================================
// DPC routine type
// ============================================================================

/// PKDEFERRED_ROUTINE — DPC callback function.
pub type PKDEFERRED_ROUTINE = unsafe extern "system" fn(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
);

// ============================================================================
// DISPATCHER_HEADER — minimal definition for process type checking
// ============================================================================

/// ProcessObject type value (from obtypes.h).
pub const ProcessObject: UCHAR = 3;

// ============================================================================
// Extern declarations — Ke functions
// ============================================================================

// FORCEINLINE functions — use extern "C" (cdecl) to match bindgen declaration
extern "C" {
    pub fn KeGetCurrentPrcb() -> PKPRCB;
}

extern "system" {
    pub fn KeBugCheckEx(
        BugCheckCode: ULONG,
        BugCheckParameter1: ULONG_PTR,
        BugCheckParameter2: ULONG_PTR,
        BugCheckParameter3: ULONG_PTR,
        BugCheckParameter4: ULONG_PTR,
    ) -> !;

    // KeGetCurrentIrql — inline on amd64 (reads CR8), extern on arm64/i386
    #[cfg(not(target_arch = "x86_64"))]
    pub fn KeGetCurrentIrql() -> KIRQL;

    // KeAcquireSpinLock/KeReleaseSpinLock — inline wrappers below (C macros)
    // KeAcquireSpinLockForDpc/KeReleaseSpinLockForDpc — fastcall, see below
    // KeAcquireQueuedSpinLock/KeReleaseQueuedSpinLock — fastcall on i386, see below
    // GuardedMutex functions are fastcall — see below

    pub fn KeFlushEntireTb(Invalid: BOOLEAN, AllProcessors: BOOLEAN);

    pub fn KeGenericCallDpc(
        Routine: PKDEFERRED_ROUTINE,
        Context: PVOID,
    );

    pub fn KeSignalCallDpcDone(SystemArgument1: PVOID);

    pub fn KeSignalCallDpcSynchronize(SystemArgument2: PVOID) -> BOOLEAN;

    pub fn KeSetEvent(
        Event: PVOID, // PKEVENT
        Increment: LONG,
        Wait: BOOLEAN,
    ) -> LONG;

    pub fn KeClearEvent(Event: PVOID); // PKEVENT

    pub fn DbgBreakPoint();
}

// KeQueryInterruptTime is an exported NTAPI function on x86, but on amd64/arm64
// it is a macro reading KUSER_SHARED_DATA (no exported symbol). Mirror the C
// headers (xdk/amd64/ke.h, xdk/arm64/ke.h): SharedInterruptTime is at
// KI_USER_SHARED_DATA + 0x8.
#[cfg(target_arch = "x86")]
extern "system" {
    pub fn KeQueryInterruptTime() -> ULONGLONG;
}

#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn KeQueryInterruptTime() -> ULONGLONG {
    core::ptr::read_volatile((crate::mm::KI_USER_SHARED_DATA + 0x8) as *const ULONGLONG)
}

// ============================================================================
// GuardedMutex — fastcall on all architectures
// ============================================================================

#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KeInitializeGuardedMutex(Mutex: PKGUARDED_MUTEX);
    pub fn KeAcquireGuardedMutex(Mutex: PKGUARDED_MUTEX);
    pub fn KeReleaseGuardedMutex(Mutex: PKGUARDED_MUTEX);
}

#[cfg(not(target_arch = "x86"))]
extern "system" {
    pub fn KeInitializeGuardedMutex(Mutex: PKGUARDED_MUTEX);
    pub fn KeAcquireGuardedMutex(Mutex: PKGUARDED_MUTEX);
    pub fn KeReleaseGuardedMutex(Mutex: PKGUARDED_MUTEX);
}

// ============================================================================
// KeGetCurrentIrql — inline on amd64 (reads CR8 register)
// ============================================================================

#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeGetCurrentIrql() -> KIRQL {
    let irql: u64;
    core::arch::asm!("mov {}, cr8", out(reg) irql, options(nomem, nostack));
    irql as KIRQL
}

// ============================================================================
// Spinlock inline wrappers
// ============================================================================
// On arm64/x86_64: KeAcquireSpinLock is a macro → KeAcquireSpinLockRaiseToDpc
// On i386: KeAcquireSpinLock is a macro → KfAcquireSpinLock (fastcall)

// KeAcquireSpinLockRaiseToDpc — available on arm64/x86_64
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn KeAcquireSpinLockRaiseToDpc(SpinLock: PKSPIN_LOCK) -> KIRQL;
}

// KfAcquireSpinLock / KfReleaseSpinLock — available on i386
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KfAcquireSpinLock(SpinLock: PKSPIN_LOCK) -> KIRQL;
    fn KfReleaseSpinLock(SpinLock: PKSPIN_LOCK, NewIrql: KIRQL);
}

/// KeAcquireSpinLock — portable wrapper.
#[inline]
pub unsafe fn KeAcquireSpinLock(spin_lock: PKSPIN_LOCK, old_irql: *mut KIRQL) {
    #[cfg(not(target_arch = "x86"))]
    {
        *old_irql = KeAcquireSpinLockRaiseToDpc(spin_lock);
    }
    #[cfg(target_arch = "x86")]
    {
        *old_irql = KfAcquireSpinLock(spin_lock);
    }
}

/// KeReleaseSpinLock — portable wrapper.
#[inline]
pub unsafe fn KeReleaseSpinLock(spin_lock: PKSPIN_LOCK, new_irql: KIRQL) {
    #[cfg(not(target_arch = "x86"))]
    {
        // KeReleaseSpinLock is a real function on x86_64/arm64
        extern "system" {
            #[link_name = "KeReleaseSpinLock"]
            fn KeReleaseSpinLock_impl(SpinLock: PKSPIN_LOCK, NewIrql: KIRQL);
        }
        KeReleaseSpinLock_impl(spin_lock, new_irql);
    }
    #[cfg(target_arch = "x86")]
    {
        KfReleaseSpinLock(spin_lock, new_irql);
    }
}

// ============================================================================
// QueuedSpinLock — fastcall on i386/arm64 (from HAL), cdecl on x86_64 (ntoskrnl)
// ============================================================================

#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KeAcquireQueuedSpinLock(Number: ULONG) -> KIRQL;
    pub fn KeReleaseQueuedSpinLock(Number: ULONG, OldIrql: KIRQL);
}

#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn KeAcquireQueuedSpinLock(Number: ULONG) -> KIRQL;
    pub fn KeReleaseQueuedSpinLock(Number: ULONG, OldIrql: KIRQL);
}

// ============================================================================
// KeAcquireSpinLockForDpc / KeReleaseSpinLockForDpc — fastcall on all arches
// ============================================================================

#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn KeAcquireSpinLockForDpc(SpinLock: PKSPIN_LOCK) -> KIRQL;
    pub fn KeReleaseSpinLockForDpc(SpinLock: PKSPIN_LOCK, OldIrql: KIRQL);
}

#[cfg(not(target_arch = "x86"))]
extern "system" {
    pub fn KeAcquireSpinLockForDpc(SpinLock: PKSPIN_LOCK) -> KIRQL;
    pub fn KeReleaseSpinLockForDpc(SpinLock: PKSPIN_LOCK, OldIrql: KIRQL);
}

// ============================================================================
// KeAcquireSpinLockAtDpcLevel / KeReleaseSpinLockFromDpcLevel
// On amd64/arm64: regular extern "system".
// On i386: fastcall KefAcquireSpinLockAtDpcLevel / KefReleaseSpinLockFromDpcLevel.
// ============================================================================

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KefAcquireSpinLockAtDpcLevel(SpinLock: PKSPIN_LOCK);
    fn KefReleaseSpinLockFromDpcLevel(SpinLock: PKSPIN_LOCK);
}

#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn KeAcquireSpinLockAtDpcLevel_impl(SpinLock: PKSPIN_LOCK);
    fn KeReleaseSpinLockFromDpcLevel_impl(SpinLock: PKSPIN_LOCK);
}

/// KeAcquireSpinLockAtDpcLevel — portable wrapper.
#[inline]
pub unsafe fn KeAcquireSpinLockAtDpcLevel(spin_lock: PKSPIN_LOCK) {
    #[cfg(not(target_arch = "x86"))]
    {
        extern "system" {
            #[link_name = "KeAcquireSpinLockAtDpcLevel"]
            fn KeAcquireSpinLockAtDpcLevel_impl(SpinLock: PKSPIN_LOCK);
        }
        KeAcquireSpinLockAtDpcLevel_impl(spin_lock);
    }
    #[cfg(target_arch = "x86")]
    {
        KefAcquireSpinLockAtDpcLevel(spin_lock);
    }
}

/// KeReleaseSpinLockFromDpcLevel — portable wrapper.
#[inline]
pub unsafe fn KeReleaseSpinLockFromDpcLevel(spin_lock: PKSPIN_LOCK) {
    #[cfg(not(target_arch = "x86"))]
    {
        extern "system" {
            #[link_name = "KeReleaseSpinLockFromDpcLevel"]
            fn KeReleaseSpinLockFromDpcLevel_impl(SpinLock: PKSPIN_LOCK);
        }
        KeReleaseSpinLockFromDpcLevel_impl(spin_lock);
    }
    #[cfg(target_arch = "x86")]
    {
        KefReleaseSpinLockFromDpcLevel(spin_lock);
    }
}

// ============================================================================
// KDEFERRED_ROUTINE — type alias for DPC callback (Option<fn>)
// On amd64/arm64 bindgen generates both KDEFERRED_ROUTINE and PKDEFERRED_ROUTINE.
// On i386 bindgen only generates PKDEFERRED_ROUTINE (as Option<fn>).
// Provide a unified type here.
// ============================================================================

pub type KDEFERRED_ROUTINE = Option<unsafe extern "system" fn(
    Dpc: PKDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
)>;

// ============================================================================
// KPRCB field accessors
// ============================================================================

// KPRCB lookaside list access:
// PP_LOOKASIDE_LIST fields are at architecture-dependent offsets in KPRCB.
// Rather than defining the entire 10KB+ KPRCB structure, the pool crate
// accesses these via raw pointer arithmetic with known offsets.
