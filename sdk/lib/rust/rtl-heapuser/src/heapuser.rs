use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{HANDLE, PPEB, RTL_CRITICAL_SECTION, RtlEnterCriticalSection, RtlGetCurrentPeb, RtlInitializeCriticalSection, RtlLeaveCriticalSection};

// ---- local type: minimal _HEAP with ProcessHeapsListIndex at the correct offset ----
// sdk/lib/rtl/heap.h line 222-269: struct _HEAP { HEAP_SEGMENT_MEMBERS; ... USHORT ProcessHeapsListIndex; ... }
// HEAP_SEGMENT_MEMBERS spans HEAP_ENTRY(8/16) + 15 fields; the total offset varies by pointer width:
//   i386: 128 bytes   amd64/arm64: 216 bytes
// Verified layout matches rtl-heap/src/types.rs field ordering; only ProcessHeapsListIndex is accessed.
#[cfg(target_pointer_width = "32")]
const _HEAP_PHLI_OFFSET: usize = 128;
#[cfg(target_pointer_width = "64")]
const _HEAP_PHLI_OFFSET: usize = 216;

#[repr(C)]
pub struct _HEAP {
    _pad: [u8; _HEAP_PHLI_OFFSET],
    pub ProcessHeapsListIndex: USHORT,
}
pub type PHEAP = *mut _HEAP;

// ---- file-scope preamble (translated) ----
pub static mut RtlpProcessHeapsListLock: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "system" fn RtlpAddHeapToProcessList(Heap: PHEAP) {
    let Peb: PPEB;

    Peb = RtlGetCurrentPeb();

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(RtlpProcessHeapsListLock));

    if (*Peb).NumberOfHeaps == (*Peb).MaximumNumberOfHeaps {
    }

    *(*Peb).ProcessHeaps.offset((*Peb).NumberOfHeaps as isize) = Heap as *mut _;
    (*Peb).NumberOfHeaps += 1;
    (*Heap).ProcessHeapsListIndex = (*Peb).NumberOfHeaps as USHORT;

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlpProcessHeapsListLock));
}

#[no_mangle]
pub unsafe extern "system" fn RtlpRemoveHeapFromProcessList(Heap: PHEAP) {
    let Peb: PPEB;
    let mut Current: *mut PHEAP;
    let mut Next: *mut PHEAP;
    let mut Count: ULONG;

    /* Get PEB */
    Peb = RtlGetCurrentPeb();

    /* Acquire the lock */
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(RtlpProcessHeapsListLock));

    /* Check if we don't need anything to do */
    if ((*Heap).ProcessHeapsListIndex == 0)
        || ((*Heap).ProcessHeapsListIndex as ULONG > (*Peb).NumberOfHeaps)
        || ((*Peb).NumberOfHeaps == 0)
    {
        /* Release the lock */
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlpProcessHeapsListLock));

        return;
    }

    /* The process actually has more than one heap. */

    Current = core::ptr::addr_of_mut!(
        *(*Peb)
            .ProcessHeaps
            .offset(((*Heap).ProcessHeapsListIndex - 1) as isize)
    ) as *mut PHEAP;
    Next = Current.offset(1);

    /* How many items we need to shift to the left */
    Count = (*Peb).NumberOfHeaps - ((*Heap).ProcessHeapsListIndex as ULONG - 1);

    /* Move them all in a loop */
    loop {
        Count -= 1;
        if Count == 0 {
            break;
        }

        /* Copy it and advance next pointer */
        *Current = *Next;

        /* Update its index */
        (*(*Current as PHEAP)).ProcessHeapsListIndex -= 1;

        /* Advance pointers */
        Current = Current.offset(1);
        Next = Next.offset(1);
    }

    /* Decrease total number of heaps */
    (*Peb).NumberOfHeaps -= 1;

    /* Zero last unused item */
    *(*Peb).ProcessHeaps.offset((*Peb).NumberOfHeaps as isize) = core::ptr::null_mut();
    (*Heap).ProcessHeapsListIndex = 0;

    /* Release the lock */
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlpProcessHeapsListLock));
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeHeapManager() {
    let Peb: PPEB = RtlGetCurrentPeb();

    (*Peb).NumberOfHeaps = 0;

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(RtlpProcessHeapsListLock));
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetProcessHeaps(HeapCount: ULONG, HeapArray: *mut HANDLE) -> ULONG {
    let Peb: PPEB = RtlGetCurrentPeb();

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(RtlpProcessHeapsListLock));
    let nHeaps: ULONG = (*Peb).NumberOfHeaps;

    if !HeapArray.is_null() {
        let limit = if nHeaps < HeapCount { nHeaps } else { HeapCount };
        let mut n: ULONG = 0;
        while n < limit {
            *HeapArray.offset(n as isize) = *(*Peb).ProcessHeaps.offset(n as isize);
            n += 1;
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(RtlpProcessHeapsListLock));

    nHeaps
}
