//! Interlocked operations — extern declarations and inline wrappers.
//!
//! These map to compiler intrinsics or Rtl* functions depending on architecture.

use crate::base::*;

// ============================================================================
// Basic interlocked operations — inline using core::sync::atomic
// ============================================================================
// These are compiler intrinsics in C (not real exported functions).
// On i386 they're fastcall exports in ntoskrnl.spec, but using inline
// atomics is simpler and avoids calling convention issues.

/// InterlockedIncrement — returns the NEW value.
#[inline]
pub unsafe fn InterlockedIncrement(Addend: *mut LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Addend as *const AtomicI32);
    atomic.fetch_add(1, Ordering::SeqCst).wrapping_add(1)
}

/// InterlockedDecrement — returns the NEW value.
#[inline]
pub unsafe fn InterlockedDecrement(Addend: *mut LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Addend as *const AtomicI32);
    atomic.fetch_sub(1, Ordering::SeqCst).wrapping_sub(1)
}

/// InterlockedOr / InterlockedAnd / InterlockedXor — return the OLD value, as the MSVC
/// `_Interlocked{Or,And,Xor}` intrinsics do. The new ATA stack (atapi's port state machine,
/// pciidex's AHCI slots) sets and clears event and slot bits with them.
#[inline]
pub unsafe fn InterlockedOr(Destination: *mut LONG, Value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    (&*(Destination as *const AtomicI32)).fetch_or(Value, Ordering::SeqCst)
}

#[inline]
pub unsafe fn InterlockedAnd(Destination: *mut LONG, Value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    (&*(Destination as *const AtomicI32)).fetch_and(Value, Ordering::SeqCst)
}

#[inline]
pub unsafe fn InterlockedXor(Destination: *mut LONG, Value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    (&*(Destination as *const AtomicI32)).fetch_xor(Value, Ordering::SeqCst)
}

/// InterlockedExchangeAdd — returns the OLD value.
#[inline]
pub unsafe fn InterlockedExchangeAdd(Addend: *mut LONG, Value: LONG) -> LONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Addend as *const AtomicI32);
    atomic.fetch_add(Value, Ordering::SeqCst)
}

/// InterlockedExchangePointer — atomically exchange a pointer, returns OLD value.
#[inline]
pub unsafe fn InterlockedExchangePointer(Target: *mut PVOID, Value: PVOID) -> PVOID {
    use core::sync::atomic::{AtomicPtr, Ordering};
    let atomic = &*(Target as *const AtomicPtr<core::ffi::c_void>);
    atomic.swap(Value, Ordering::SeqCst)
}

/// InterlockedCompareExchangePointer — CAS on a pointer, returns OLD value.
#[inline]
pub unsafe fn InterlockedCompareExchangePointer(
    Destination: *mut PVOID,
    Exchange: PVOID,
    Comparand: PVOID,
) -> PVOID {
    use core::sync::atomic::{AtomicPtr, Ordering};
    let atomic = &*(Destination as *const AtomicPtr<core::ffi::c_void>);
    match atomic.compare_exchange(Comparand, Exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => old,
        Err(old) => old,
    }
}

// ============================================================================
// SList operations — name mapping to Rtl* symbols
// ============================================================================

// On all architectures, the actual symbols are RtlInterlockedPopEntrySList
// and RtlInterlockedPushEntrySList. The Windows "InterlockedPopEntrySList"
// names are DLL exports that forward to these.

// Pop: 1 parameter on all architectures
#[cfg(not(target_arch = "x86"))]
extern "system" {
    #[link_name = "RtlInterlockedPopEntrySList"]
    pub fn InterlockedPopEntrySList(ListHead: PSLIST_HEADER) -> PSLIST_ENTRY;
}

#[cfg(target_arch = "x86")]
extern "stdcall" {
    #[link_name = "RtlInterlockedPopEntrySList"]
    fn RtlInterlockedPopEntrySList_i386(ListHead: PSLIST_HEADER) -> PSLIST_ENTRY;
}

#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn InterlockedPopEntrySList(list_head: PSLIST_HEADER) -> PSLIST_ENTRY {
    RtlInterlockedPopEntrySList_i386(list_head)
}

// Push: 2 params on all architectures (Lock param removed — Rust SList ignores it)
#[cfg(not(target_arch = "x86"))]
extern "system" {
    #[link_name = "RtlInterlockedPushEntrySList"]
    pub fn InterlockedPushEntrySList(
        ListHead: PSLIST_HEADER,
        ListEntry: PSLIST_ENTRY,
    ) -> PSLIST_ENTRY;
}

#[cfg(target_arch = "x86")]
extern "stdcall" {
    #[link_name = "RtlInterlockedPushEntrySList"]
    fn RtlInterlockedPushEntrySList_i386(
        ListHead: PSLIST_HEADER,
        ListEntry: PSLIST_ENTRY,
    ) -> PSLIST_ENTRY;
}

#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn InterlockedPushEntrySList(
    list_head: PSLIST_HEADER,
    list_entry: PSLIST_ENTRY,
) -> PSLIST_ENTRY {
    RtlInterlockedPushEntrySList_i386(list_head, list_entry)
}

// ============================================================================
// ExQueryDepthSList — inline (reads low 16 bits of header)
// ============================================================================

/// ExQueryDepthSList — returns the depth of an SList.
/// On all architectures, the depth is stored in the low 16 bits of the header.
#[inline]
pub unsafe fn ExQueryDepthSList(ListHead: PSLIST_HEADER) -> USHORT {
    // SLIST_HEADER._opaque[0] has depth in its low 16 bits on both 32-bit and 64-bit.
    // On 64-bit: _opaque is [u64; 2], low 16 bits of first u64 = depth.
    // On 32-bit: _opaque is [u32; 2], low 16 bits of first u32 = depth.
    #[cfg(target_pointer_width = "64")]
    { (*ListHead)._opaque[0] as USHORT }
    #[cfg(target_pointer_width = "32")]
    { (*ListHead)._opaque[0] as USHORT }
}

// ============================================================================
// InterlockedExchangeAddSizeT — inline using core::sync::atomic
// ============================================================================

/// Atomic exchange-add on a SIZE_T (pointer-width) value.
#[inline]
pub unsafe fn InterlockedExchangeAddSizeT(addend: *mut isize, value: isize) -> isize {
    use core::sync::atomic::{AtomicIsize, Ordering};
    let atomic = &*(addend as *const AtomicIsize);
    atomic.fetch_add(value, Ordering::SeqCst)
}

/// InterlockedIncrementSizeT — atomic increment on isize (LONG_PTR), returns NEW value.
#[inline]
pub unsafe fn InterlockedIncrementSizeT(Addend: *mut isize) -> isize {
    use core::sync::atomic::{AtomicIsize, Ordering};
    let atomic = &*(Addend as *const AtomicIsize);
    atomic.fetch_add(1, Ordering::SeqCst).wrapping_add(1)
}

/// InterlockedDecrementSizeT — atomic decrement on isize (LONG_PTR), returns NEW value.
#[inline]
pub unsafe fn InterlockedDecrementSizeT(Addend: *mut isize) -> isize {
    use core::sync::atomic::{AtomicIsize, Ordering};
    let atomic = &*(Addend as *const AtomicIsize);
    atomic.fetch_sub(1, Ordering::SeqCst).wrapping_sub(1)
}
