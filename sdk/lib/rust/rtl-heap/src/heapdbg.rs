//! Translation of `sdk/lib/rtl/heapdbg.c` — the debug-heap wrappers
//! (`RtlDebug*`). Each one validates the heap (`RtlpValidateHeap*`),
//! forwards to the real `Rtl*Heap`, and (where the C does) re-validates.
//! They are reached only via the `RtlpHeapIsSpecial` fast-path in heap.rs
//! when heap debug/validation flags are set, so they are inert on a normal
//! boot. Faithful to ReactOS heapdbg.c (Aleksey Bragin, 2010).
//!
//! All the real allocator entry points, the validators, and the heap-lock
//! helpers are exported `#[no_mangle]` symbols from the sibling `heap`
//! module / its externs; the page-heap forwarders live in heappage.c (C).
//! We declare them `extern "system"` (NTAPI) and let the linker resolve.

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{
    MEMORY_BASIC_INFORMATION, PHEAP_LOCK, PPEB, PRTL_HEAP_PARAMETERS,
    ZwFreeVirtualMemory, ZwQueryVirtualMemory,
    _MEMORY_INFORMATION_CLASS_MemoryBasicInformation as MemoryBasicInformation,
};
use ntkrnl_sys::macros::NtCurrentProcess;
// The heap.h type family + HEAP_* constants (byte-exact local substrate).
use crate::types::*;

extern "system" {
    // --- real allocator entry points (sibling heap.rs #[no_mangle]) ---
    fn RtlCreateHeap(
        Flags: ULONG,
        Addr: PVOID,
        TotalSize: SIZE_T,
        CommitSize: SIZE_T,
        Lock: PVOID,
        Parameters: PRTL_HEAP_PARAMETERS,
    ) -> HANDLE;
    fn RtlAllocateHeap(HeapPtr: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlReAllocateHeap(HeapPtr: PVOID, Flags: ULONG, Ptr: PVOID, Size: SIZE_T) -> PVOID;
    fn RtlFreeHeap(HeapPtr: PVOID, Flags: ULONG, Ptr: PVOID) -> BOOLEAN;
    fn RtlSizeHeap(HeapPtr: PVOID, Flags: ULONG, Ptr: PVOID) -> SIZE_T;
    fn RtlGetUserInfoHeap(
        HeapHandle: PVOID,
        Flags: ULONG,
        BaseAddress: PVOID,
        UserValue: *mut PVOID,
        UserFlags: PULONG,
    ) -> BOOLEAN;
    fn RtlSetUserValueHeap(
        HeapHandle: PVOID,
        Flags: ULONG,
        BaseAddress: PVOID,
        UserValue: PVOID,
    ) -> BOOLEAN;
    fn RtlSetUserFlagsHeap(
        HeapHandle: PVOID,
        Flags: ULONG,
        BaseAddress: PVOID,
        UserFlagsReset: ULONG,
        UserFlagsSet: ULONG,
    ) -> BOOLEAN;

    // --- validators (sibling heap.rs #[no_mangle]) ---
    fn RtlpValidateHeap(Heap: PHEAP, ForceValidation: BOOLEAN) -> BOOLEAN;
    fn RtlpValidateHeapEntry(Heap: PHEAP, HeapEntry: PHEAP_ENTRY) -> BOOLEAN;
    fn RtlpValidateHeapHeaders(Heap: PHEAP, Recalculate: BOOLEAN) -> BOOLEAN;

    // --- heap lock + PEB (rtlp.h / rtl exports) ---
    fn RtlEnterHeapLock(Lock: PHEAP_LOCK, Exclusive: BOOLEAN) -> NTSTATUS;
    fn RtlLeaveHeapLock(Lock: PHEAP_LOCK) -> NTSTATUS;
    fn RtlGetCurrentPeb() -> PPEB;

    // --- page heap (heappage.c, stays C) ---
    fn RtlpPageHeapAllocate(HeapPtr: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlpPageHeapReAllocate(HeapPtr: PVOID, Flags: ULONG, Ptr: PVOID, Size: SIZE_T) -> PVOID;
    fn RtlpPageHeapFree(HeapPtr: PVOID, Flags: ULONG, Ptr: PVOID) -> BOOLEAN;
    fn RtlpPageHeapGetUserInfo(
        HeapHandle: PVOID,
        Flags: ULONG,
        BaseAddress: PVOID,
        UserValue: *mut PVOID,
        UserFlags: PULONG,
    ) -> BOOLEAN;
    fn RtlpPageHeapSetUserValue(
        HeapHandle: PVOID,
        Flags: ULONG,
        BaseAddress: PVOID,
        UserValue: PVOID,
    ) -> BOOLEAN;
    fn RtlpPageHeapSetUserFlags(
        HeapHandle: PVOID,
        Flags: ULONG,
        BaseAddress: PVOID,
        UserFlagsReset: ULONG,
        UserFlagsSet: ULONG,
    ) -> BOOLEAN;
    fn RtlpPageHeapSize(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> SIZE_T;
}

#[no_mangle]
pub unsafe extern "system" fn RtlDebugCreateHeap(
    mut Flags: ULONG,
    Addr: PVOID,
    ReserveSize: SIZE_T,
    CommitSize: SIZE_T,
    Lock: PVOID,
    Parameters: PRTL_HEAP_PARAMETERS,
) -> HANDLE {
    let mut MemoryInfo: MEMORY_BASIC_INFORMATION = core::mem::zeroed();

    /* Validate parameters */
    if ReserveSize <= HEAP_ENTRY_SIZE as SIZE_T {
        return core::ptr::null_mut();
    }

    if ReserveSize < CommitSize {
        return core::ptr::null_mut();
    }

    if Flags & HEAP_NO_SERIALIZE != 0 && !Lock.is_null() {
        return core::ptr::null_mut();
    }

    /* If the address is specified, check it's virtual memory */
    if !Addr.is_null() {
        let Status = ZwQueryVirtualMemory(
            NtCurrentProcess(),
            Addr,
            MemoryBasicInformation,
            core::ptr::addr_of_mut!(MemoryInfo) as PVOID,
            core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as _,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) {
            return core::ptr::null_mut();
        }

        if MemoryInfo.BaseAddress != Addr {
            return core::ptr::null_mut();
        }

        if MemoryInfo.State == MEM_FREE {
            return core::ptr::null_mut();
        }
    }

    /* All validation performed, now call the real routine with skip validation check flag */
    Flags |= HEAP_SKIP_VALIDATION_CHECKS | HEAP_TAIL_CHECKING_ENABLED | HEAP_FREE_CHECKING_ENABLED;

    let Heap = RtlCreateHeap(Flags, Addr, ReserveSize, CommitSize, Lock, Parameters) as PHEAP;
    if Heap.is_null() {
        return core::ptr::null_mut();
    }

    // FIXME: Capture stack backtrace

    RtlpValidateHeapHeaders(Heap, 1u8);

    Heap as HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn RtlDebugDestroyHeap(HeapPtr: HANDLE) -> BOOLEAN {
    let mut Size: SIZE_T = 0;
    let Heap = HeapPtr as PHEAP;

    if Heap as PVOID == (*RtlGetCurrentPeb()).ProcessHeap {
        return 0u8;
    }

    if (*Heap).Signature != HEAP_SIGNATURE {
        return 0u8;
    }

    if RtlpValidateHeap(Heap, 0u8) == 0 {
        return 0u8;
    }

    /* Make heap invalid by zeroing its signature */
    (*Heap).Signature = 0;

    /* Free validate headers copy if it was existing */
    if !(*Heap).HeaderValidateCopy.is_null() {
        ZwFreeVirtualMemory(
            NtCurrentProcess(),
            core::ptr::addr_of_mut!((*Heap).HeaderValidateCopy) as *mut PVOID,
            core::ptr::addr_of_mut!(Size) as *mut _,
            MEM_RELEASE,
        );
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlDebugAllocateHeap(
    HeapPtr: PVOID,
    mut Flags: ULONG,
    Size: SIZE_T,
) -> PVOID {
    let Heap = HeapPtr as PHEAP;
    let mut AllocSize: SIZE_T = 1;
    let mut HeapLocked = false;

    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpPageHeapAllocate(HeapPtr, Flags, Size);
    }

    if (*Heap).Signature != HEAP_SIGNATURE {
        return core::ptr::null_mut();
    }

    /* Add settable user value flag */
    Flags |= (*Heap).ForceFlags | HEAP_SETTABLE_USER_VALUE | HEAP_SKIP_VALIDATION_CHECKS;

    /* Calculate size */
    if Size != 0 {
        AllocSize = Size;
    }
    AllocSize = ((AllocSize + (*Heap).AlignRound as SIZE_T) & (*Heap).AlignMask as SIZE_T)
        + core::mem::size_of::<HEAP_ENTRY_EXTRA>();

    /* Check if size didn't exceed max one */
    if AllocSize < Size || AllocSize > (*Heap).MaximumAllocationSize {
        return core::ptr::null_mut();
    }

    /* Lock the heap ourselves */
    if Flags & HEAP_NO_SERIALIZE == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = true;

        /* Add no serialize flag so that the main routine won't try to acquire the lock again */
        Flags |= HEAP_NO_SERIALIZE;
    }

    /* Validate the heap if necessary */
    RtlpValidateHeap(Heap, 0u8);

    /* Call main routine to do the stuff */
    let Result = RtlAllocateHeap(HeapPtr, Flags, Size);

    /* Validate heap headers */
    RtlpValidateHeapHeaders(Heap, 1u8);

    if !Result.is_null() && (*Heap).Flags & HEAP_VALIDATE_ALL_ENABLED != 0 {
        RtlpValidateHeap(Heap, 0u8);
    }

    /* Release the lock */
    if HeapLocked {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn RtlDebugReAllocateHeap(
    HeapPtr: HANDLE,
    mut Flags: ULONG,
    Ptr: PVOID,
    Size: SIZE_T,
) -> PVOID {
    let Heap = HeapPtr as PHEAP;
    let mut AllocSize: SIZE_T = 1;
    let mut HeapLocked = false;
    let mut Result: PVOID = core::ptr::null_mut();

    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpPageHeapReAllocate(HeapPtr, Flags, Ptr, Size);
    }

    if (*Heap).Signature != HEAP_SIGNATURE {
        return core::ptr::null_mut();
    }

    /* Add settable user value flag */
    Flags |= (*Heap).ForceFlags | HEAP_SETTABLE_USER_VALUE | HEAP_SKIP_VALIDATION_CHECKS;

    /* Calculate size */
    if Size != 0 {
        AllocSize = Size;
    }
    AllocSize = ((AllocSize + (*Heap).AlignRound as SIZE_T) & (*Heap).AlignMask as SIZE_T)
        + core::mem::size_of::<HEAP_ENTRY_EXTRA>();

    /* Check if size didn't exceed max one */
    if AllocSize < Size || AllocSize > (*Heap).MaximumAllocationSize {
        return core::ptr::null_mut();
    }

    /* Lock the heap ourselves */
    if Flags & HEAP_NO_SERIALIZE == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = true;

        Flags |= HEAP_NO_SERIALIZE;
    }

    /* Validate the heap if necessary */
    RtlpValidateHeap(Heap, 0u8);

    /* Get the existing heap entry */
    let HeapEntry = (Ptr as PHEAP_ENTRY).offset(-1);

    /* Validate it */
    if RtlpValidateHeapEntry(Heap, HeapEntry) != 0 {
        /* Call main routine to do the stuff */
        Result = RtlReAllocateHeap(HeapPtr, Flags, Ptr, Size);

        if !Result.is_null() {
            /* Validate heap headers and then heap itself */
            RtlpValidateHeapHeaders(Heap, 1u8);
            RtlpValidateHeap(Heap, 0u8);
        }
    }

    /* Release the lock */
    if HeapLocked {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn RtlDebugFreeHeap(
    HeapPtr: HANDLE,
    mut Flags: ULONG,
    Ptr: PVOID,
) -> BOOLEAN {
    let Heap = HeapPtr as PHEAP;
    let mut HeapLocked = false;
    let mut Result: BOOLEAN = 0u8;

    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpPageHeapFree(HeapPtr, Flags, Ptr);
    }

    if (*Heap).Signature != HEAP_SIGNATURE {
        return 0u8;
    }

    /* Add skip validation flag */
    Flags |= (*Heap).ForceFlags | HEAP_SKIP_VALIDATION_CHECKS;

    /* Lock the heap ourselves */
    if Flags & HEAP_NO_SERIALIZE == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = true;

        Flags |= HEAP_NO_SERIALIZE;
    }

    /* Validate the heap if necessary */
    RtlpValidateHeap(Heap, 0u8);

    /* Get the existing heap entry */
    let HeapEntry = (Ptr as PHEAP_ENTRY).offset(-1);

    /* Validate it */
    if RtlpValidateHeapEntry(Heap, HeapEntry) != 0 {
        /* If it succeeded - call the main routine */
        Result = RtlFreeHeap(HeapPtr, Flags, Ptr);

        /* Validate heap headers and then heap itself */
        RtlpValidateHeapHeaders(Heap, 1u8);
        RtlpValidateHeap(Heap, 0u8);
    }

    /* Release the lock */
    if HeapLocked {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn RtlDebugGetUserInfoHeap(
    HeapHandle: PVOID,
    mut Flags: ULONG,
    BaseAddress: PVOID,
    UserValue: *mut PVOID,
    UserFlags: PULONG,
) -> BOOLEAN {
    let Heap = HeapHandle as PHEAP;
    let mut HeapLocked = false;
    let mut Result: BOOLEAN = 0u8;

    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpPageHeapGetUserInfo(HeapHandle, Flags, BaseAddress, UserValue, UserFlags);
    }

    if (*Heap).Signature != HEAP_SIGNATURE {
        return 0u8;
    }

    /* Add skip validation flag */
    Flags |= (*Heap).ForceFlags | HEAP_SKIP_VALIDATION_CHECKS;

    /* Lock the heap ourselves */
    if Flags & HEAP_NO_SERIALIZE == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = true;

        Flags |= HEAP_NO_SERIALIZE;
    }

    /* Validate the heap if necessary */
    RtlpValidateHeap(Heap, 0u8);

    /* Get the existing heap entry */
    let HeapEntry = (BaseAddress as PHEAP_ENTRY).offset(-1);

    /* Validate it */
    if RtlpValidateHeapEntry(Heap, HeapEntry) != 0 {
        /* If it succeeded - call the main routine */
        Result = RtlGetUserInfoHeap(HeapHandle, Flags, BaseAddress, UserValue, UserFlags);
    }

    /* Release the lock */
    if HeapLocked {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn RtlDebugSetUserValueHeap(
    HeapHandle: PVOID,
    mut Flags: ULONG,
    BaseAddress: PVOID,
    UserValue: PVOID,
) -> BOOLEAN {
    let Heap = HeapHandle as PHEAP;
    let mut HeapLocked = false;
    let mut Result: BOOLEAN = 0u8;

    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpPageHeapSetUserValue(HeapHandle, Flags, BaseAddress, UserValue);
    }

    if (*Heap).Signature != HEAP_SIGNATURE {
        return 0u8;
    }

    /* Add skip validation flag */
    Flags |= (*Heap).ForceFlags | HEAP_SKIP_VALIDATION_CHECKS;

    /* Lock the heap ourselves */
    if Flags & HEAP_NO_SERIALIZE == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = true;

        Flags |= HEAP_NO_SERIALIZE;
    }

    /* Validate the heap if necessary */
    RtlpValidateHeap(Heap, 0u8);

    /* Get the existing heap entry */
    let HeapEntry = (BaseAddress as PHEAP_ENTRY).offset(-1);

    /* Validate it */
    if RtlpValidateHeapEntry(Heap, HeapEntry) != 0 {
        /* If it succeeded - call the main routine */
        Result = RtlSetUserValueHeap(HeapHandle, Flags, BaseAddress, UserValue);

        /* Validate the heap */
        RtlpValidateHeap(Heap, 0u8);
    }

    /* Release the lock */
    if HeapLocked {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn RtlDebugSetUserFlagsHeap(
    HeapHandle: PVOID,
    mut Flags: ULONG,
    BaseAddress: PVOID,
    UserFlagsReset: ULONG,
    UserFlagsSet: ULONG,
) -> BOOLEAN {
    let Heap = HeapHandle as PHEAP;
    let mut HeapLocked = false;
    let mut Result: BOOLEAN = 0u8;

    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpPageHeapSetUserFlags(
            HeapHandle,
            Flags,
            BaseAddress,
            UserFlagsReset,
            UserFlagsSet,
        );
    }

    /* Check if this heap allows flags to be set at all */
    if UserFlagsSet & !HEAP_SETTABLE_USER_FLAGS != 0
        || UserFlagsReset & !HEAP_SETTABLE_USER_FLAGS != 0
    {
        return 0u8;
    }

    if (*Heap).Signature != HEAP_SIGNATURE {
        return 0u8;
    }

    /* Add skip validation flag */
    Flags |= (*Heap).ForceFlags | HEAP_SKIP_VALIDATION_CHECKS;

    /* Lock the heap ourselves */
    if Flags & HEAP_NO_SERIALIZE == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = true;

        Flags |= HEAP_NO_SERIALIZE;
    }

    /* Validate the heap if necessary */
    RtlpValidateHeap(Heap, 0u8);

    /* Get the existing heap entry */
    let HeapEntry = (BaseAddress as PHEAP_ENTRY).offset(-1);

    /* Validate it */
    if RtlpValidateHeapEntry(Heap, HeapEntry) != 0 {
        /* If it succeeded - call the main routine */
        Result = RtlSetUserFlagsHeap(HeapHandle, Flags, BaseAddress, UserFlagsReset, UserFlagsSet);

        /* Validate the heap */
        RtlpValidateHeap(Heap, 0u8);
    }

    /* Release the lock */
    if HeapLocked {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn RtlDebugSizeHeap(
    HeapPtr: HANDLE,
    mut Flags: ULONG,
    Ptr: PVOID,
) -> SIZE_T {
    let Heap = HeapPtr as PHEAP;
    let mut HeapLocked = false;
    let mut Result: SIZE_T = !(0 as SIZE_T);

    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpPageHeapSize(HeapPtr, Flags, Ptr);
    }

    /* Check heap signature */
    if (*Heap).Signature != HEAP_SIGNATURE {
        return 0;
    }

    /* Add skip validation flag */
    Flags |= (*Heap).ForceFlags | HEAP_SKIP_VALIDATION_CHECKS;

    /* Lock the heap ourselves */
    if Flags & HEAP_NO_SERIALIZE == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = true;

        Flags |= HEAP_NO_SERIALIZE;
    }

    /* Validate the heap if necessary */
    RtlpValidateHeap(Heap, 0u8);

    /* Get the existing heap entry */
    let HeapEntry = (Ptr as PHEAP_ENTRY).offset(-1);

    /* Validate it */
    if RtlpValidateHeapEntry(Heap, HeapEntry) != 0 {
        /* If it succeeded - call the main routine */
        Result = RtlSizeHeap(HeapPtr, Flags, Ptr);
    }

    /* Release the lock */
    if HeapLocked {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    Result
}
