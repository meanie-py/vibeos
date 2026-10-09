use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    EXCEPTION_RECORD, HEAP_INFORMATION_CLASS, HEAP_LOCK, KPROCESSOR_MODE, PPEB, _TEB,
    MEMORY_BASIC_INFORMATION, PHEAP_ENUMERATION_ROUTINE, PHEAP_LOCK, PRTL_HEAP_PARAMETERS,
    PRTL_HEAP_TAG_INFO, PRTL_HEAP_USAGE, RTL_HEAP_PARAMETERS, RtlCompareMemory,
    RtlCompareMemoryUlong, RtlGetNtGlobalFlags,
    RtlRaiseException, SYSTEM_BASIC_INFORMATION, ZwAllocateVirtualMemory, ZwFreeVirtualMemory,
    ZwQuerySystemInformation, ZwQueryVirtualMemory,
    // bindgen module-style enum variants (aliased to the bare names the body uses)
    _HEAP_INFORMATION_CLASS_HeapCompatibilityInformation as HeapCompatibilityInformation,
    _MEMORY_INFORMATION_CLASS_MemoryBasicInformation as MemoryBasicInformation,
    _SYSTEM_INFORMATION_CLASS_SystemBasicInformation as SystemBasicInformation,
};
// The heap.h type family (HEAP / HEAP_ENTRY / HEAP_SEGMENT / … + the
// HEAP_* constants) — byte-exact local substrate, not in ntkrnl-sys.
use crate::types::*;
// NtCurrentProcess()/NtCurrentThread() pseudo-handles.
use ntkrnl_sys::macros::NtCurrentProcess;

extern "C" {
    // heap.h global, defined in rtl-heappage (heappage.rs, #[no_mangle]). i386: `_RtlpPageHeapEnabled`.
    static RtlpPageHeapEnabled: BOOLEAN;
}

// RtlpHeapIsSpecial is FORCEINLINE in heap.h (no exported symbol) — translate.
#[inline]
unsafe fn RtlpHeapIsSpecial(Flags: ULONG) -> BOOLEAN {
    if Flags & HEAP_SKIP_VALIDATION_CHECKS != 0 {
        return 0;
    }
    if Flags
        & (HEAP_FLAG_PAGE_ALLOCS
            | HEAP_VALIDATE_ALL_ENABLED
            | HEAP_VALIDATE_PARAMETERS_ENABLED
            | HEAP_CAPTURE_STACK_BACKTRACES
            | HEAP_CREATE_ENABLE_TRACING)
        != 0
    {
        return 1;
    }
    0
}

// NtCurrentPeb() (FORCEINLINE: NtCurrentTeb()->ProcessEnvironmentBlock).
#[inline]
unsafe fn NtCurrentPeb() -> PPEB {
    (*(ntkrnl_sys::inline::ke::NtCurrentTeb() as *mut _TEB)).ProcessEnvironmentBlock
}

// LOBYTE/HIBYTE on a USHORT (winnt.h macros).
#[inline]
fn LOBYTE(w: USHORT) -> UCHAR {
    (w & 0xff) as UCHAR
}
#[inline]
fn HIBYTE(w: USHORT) -> UCHAR {
    (w >> 8) as UCHAR
}
extern "system" {
    fn RtlpPageHeapLock(Heap: PHEAP) -> BOOLEAN;
    fn RtlpPageHeapUnlock(Heap: PHEAP) -> BOOLEAN;
    // rtl heap-lock primitives (rtlp.h; defined in librtl C, stay C)
    fn RtlEnterHeapLock(Lock: PHEAP_LOCK, Exclusive: BOOLEAN) -> NTSTATUS;
    fn RtlLeaveHeapLock(Lock: PHEAP_LOCK) -> NTSTATUS;
    fn RtlInitializeHeapLock(Lock: *mut PHEAP_LOCK) -> NTSTATUS;
    fn RtlDeleteHeapLock(Lock: PHEAP_LOCK) -> NTSTATUS;
    fn RtlpGetMode() -> KPROCESSOR_MODE;
    fn RtlpSetHeapParameters(Parameters: PRTL_HEAP_PARAMETERS);
    // page-heap / debug-heap (heappage.c, heapdbg.c — stay C)
    fn RtlpPageHeapCreate(
        Flags: ULONG,
        Addr: PVOID,
        TotalSize: SIZE_T,
        CommitSize: SIZE_T,
        Lock: PVOID,
        Parameters: PRTL_HEAP_PARAMETERS,
    ) -> PVOID;
    fn RtlpPageHeapDestroy(HeapPtr: PVOID) -> PVOID;
    fn RtlpDebugPageHeapValidate(HeapPtr: PVOID, Flags: ULONG, Block: PVOID) -> BOOLEAN;
    fn RtlpAddHeapToProcessList(Heap: PHEAP);
    fn RtlpRemoveHeapFromProcessList(Heap: PHEAP);
    // NTAPI helpers (i386 stdcall = extern "system", NOT "C")
    fn RtlSetLastWin32ErrorAndNtStatusFromNtStatus(Status: NTSTATUS);
    // RtlFindSetBits over mm::RTL_BITMAP (rtl module lacks it; the bitmap
    // crate provides the symbol). Keeps the bitmap API on ONE type.
    fn RtlFindSetBits(BitMapHeader: PRTL_BITMAP, NumberToFind: ULONG, HintIndex: ULONG) -> ULONG;
    // debug-heap (heapdbg.c — stays C)
    fn RtlDebugCreateHeap(
        Flags: ULONG,
        Addr: PVOID,
        TotalSize: SIZE_T,
        CommitSize: SIZE_T,
        Lock: PVOID,
        Parameters: PRTL_HEAP_PARAMETERS,
    ) -> PVOID;
    fn RtlDebugDestroyHeap(HeapPtr: PVOID) -> BOOLEAN;
    fn RtlDebugAllocateHeap(HeapPtr: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlDebugReAllocateHeap(HeapPtr: PVOID, Flags: ULONG, Ptr: PVOID, Size: SIZE_T) -> PVOID;
    fn RtlDebugFreeHeap(HeapPtr: PVOID, Flags: ULONG, Ptr: PVOID) -> BOOLEAN;
    fn RtlDebugGetUserInfoHeap(
        HeapHandle: PVOID,
        Flags: ULONG,
        BaseAddress: PVOID,
        UserValue: *mut PVOID,
        UserFlags: *mut ULONG,
    ) -> BOOLEAN;
    fn RtlDebugSizeHeap(HeapPtr: PVOID, Flags: ULONG, Ptr: PVOID) -> SIZE_T;
    fn RtlDebugSetUserValueHeap(
        HeapHandle: PVOID,
        Flags: ULONG,
        BaseAddress: PVOID,
        UserValue: PVOID,
    ) -> BOOLEAN;
    fn RtlDebugSetUserFlagsHeap(
        HeapHandle: PVOID,
        Flags: ULONG,
        BaseAddress: PVOID,
        UserFlagsReset: ULONG,
        UserFlagsSet: ULONG,
    ) -> BOOLEAN;
}

// PAGE_ROUND_UP(x) macro (ndk/mmtypes.h).
#[inline]
fn PAGE_ROUND_UP(x: ULONG_PTR) -> ULONG_PTR {
    (x + PAGE_SIZE as ULONG_PTR - 1) & !(PAGE_SIZE as ULONG_PTR - 1)
}

// HEAP.FreeHints is a flexible array member (`[PLIST_ENTRY; 1]` for layout;
// the heap over-allocates DeCommitFreeBlockThreshold slots). Index it via
// raw pointer arithmetic — Rust would bounds-check `FreeHints[idx]` and
// panic for idx >= 1. Returns the address of slot `idx`.
#[inline]
unsafe fn free_hint_slot(heap: PHEAP, idx: usize) -> *mut PLIST_ENTRY {
    (core::ptr::addr_of_mut!((*heap).FreeHints) as *mut PLIST_ENTRY).add(idx)
}

// RtlFillMemoryUlong (rtl/mem.c) — fill `length` bytes with the 32-bit
// `pattern`, ULONG at a time. Inlined locally: the real symbol's provider
// is arch-divergent (i386 asm / amd64 CRT / arm64 mem-crate) and the amd64
// CRT copy isn't reachable from ntdll's static link in cargo link order.
// Trivial body, only used on the heap debug-fill paths.
#[inline]
unsafe fn rtl_fill_memory_ulong(dest: PVOID, length: usize, pattern: ULONG) {
    let mut d = dest as *mut ULONG;
    let mut n = length / core::mem::size_of::<ULONG>();
    while n != 0 {
        *d = pattern;
        d = d.add(1);
        n -= 1;
    }
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

pub static RtlpBitsClearLow: [UCHAR; 256] =
[
    8,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    5,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    6,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    5,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    7,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    5,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    6,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    5,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0,
    4,0,1,0,2,0,1,0,3,0,1,0,2,0,1,0
];

// HEAP_ENTRY_SIZE is arch-dependent (8 i386 / 16 amd64/arm64), so repeat
// the fill byte rather than a fixed 8-element literal.
pub static FillPattern: [UCHAR; HEAP_ENTRY_SIZE as usize] =
    [HEAP_TAIL_FILL; HEAP_ENTRY_SIZE as usize];
// FORCEINLINE heap.c-internal helper (no external callers) — private fn.
unsafe fn RtlpFindLeastSetBit(Bits: ULONG) -> UCHAR {
    if Bits & 0xFFFF != 0 {
        if Bits & 0xFF != 0 {
            RtlpBitsClearLow[(Bits & 0xFF) as usize]
        } else {
            RtlpBitsClearLow[((Bits >> 8) & 0xFF) as usize] + 8
        }
    } else {
        if (Bits >> 16) & 0xFF != 0 {
            RtlpBitsClearLow[((Bits >> 16) & 0xFF) as usize] + 16
        } else {
            RtlpBitsClearLow[((Bits >> 24) & 0xFF) as usize] + 24
        }
    }
}

unsafe fn RtlpIsLastCommittedEntry(mut Entry: PHEAP_ENTRY) -> BOOLEAN {
    if ((*Entry).Flags & HEAP_ENTRY_LAST_ENTRY) != 0 {
        return 1u8;
    }

    Entry = Entry.add((*Entry).Size as usize);

    if ((*Entry).Flags != (HEAP_ENTRY_BUSY | HEAP_ENTRY_LAST_ENTRY)) || ((*Entry).Size != 1) {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpInitializeHeap(
    Heap: PHEAP,
    mut Flags: ULONG,
    mut Lock: PHEAP_LOCK,
    Parameters: PRTL_HEAP_PARAMETERS,
) -> NTSTATUS {
    let NumUCRs: ULONG = 8;
    let mut Index: ULONG;
    let mut HeaderSize: SIZE_T;
    let Status: NTSTATUS;
    let UcrDescriptor: PHEAP_UCR_DESCRIPTOR;
    let DeCommitFreeBlockThreshold: SIZE_T;

    /* Preconditions */
    if Heap.is_null() || Parameters.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    /* Make sure we're not doing stupid things */
    DeCommitFreeBlockThreshold =
        ((*Parameters).DeCommitFreeBlockThreshold >> HEAP_ENTRY_SHIFT) as usize;
    /* Start out with the size of a plain Heap header + our hints of free entries + the bitmap */
    HeaderSize = (core::mem::offset_of!(HEAP, FreeHints)
        + DeCommitFreeBlockThreshold as usize * core::mem::size_of::<PLIST_ENTRY>())
        + (ROUND_UP(DeCommitFreeBlockThreshold as usize, RTL_BITS_OF_ULONG as usize)
            / RTL_BITS_OF_ULONG as usize) as SIZE_T
            * core::mem::size_of::<ULONG>() as SIZE_T;

    /* Check if space needs to be added for the Heap Lock */
    if (Flags & HEAP_NO_SERIALIZE) == 0 {
        if !Lock.is_null() {
            /* The user manages the Heap Lock */
            Flags |= HEAP_LOCK_USER_ALLOCATED;
        } else if RtlpGetMode() == 1i32 as KPROCESSOR_MODE {
            /* In user mode, the Heap Lock trails the Heap header */
            Lock = ((Heap as ULONG_PTR) + HeaderSize as ULONG_PTR) as PHEAP_LOCK;
            HeaderSize += core::mem::size_of::<HEAP_LOCK>() as SIZE_T;
        }
    }

    /* Add space for the initial Heap UnCommitted Range Descriptor list */
    UcrDescriptor =
        ((Heap as ULONG_PTR) + HeaderSize as ULONG_PTR) as PHEAP_UCR_DESCRIPTOR;
    HeaderSize += NumUCRs as SIZE_T * core::mem::size_of::<HEAP_UCR_DESCRIPTOR>() as SIZE_T;

    HeaderSize = ROUND_UP(HeaderSize as usize, HEAP_ENTRY_SIZE as usize) as SIZE_T;

    /* Initialise the Heap Entry header containing the Heap header */
    (*Heap).Entry.Size = (HeaderSize >> HEAP_ENTRY_SHIFT) as USHORT;
    (*Heap).Entry.Flags = HEAP_ENTRY_BUSY as UCHAR;
    (*Heap).Entry.SmallTagIndex = (LOBYTE((*Heap).Entry.Size)
        ^ HIBYTE((*Heap).Entry.Size)
        ^ (*Heap).Entry.Flags) as UCHAR;
    (*Heap).Entry.PreviousSize = 0;
    (*Heap).Entry.SegmentOffset = 0;
    (*Heap).Entry.UnusedBytes = 0;

    /* Initialise the Heap header */
    (*Heap).Signature = HEAP_SIGNATURE;
    (*Heap).Flags = Flags;
    (*Heap).ForceFlags = Flags
        & (HEAP_NO_SERIALIZE
            | HEAP_GENERATE_EXCEPTIONS
            | HEAP_ZERO_MEMORY
            | HEAP_REALLOC_IN_PLACE_ONLY
            | HEAP_VALIDATE_PARAMETERS_ENABLED
            | HEAP_VALIDATE_ALL_ENABLED
            | HEAP_TAIL_CHECKING_ENABLED
            | HEAP_CREATE_ALIGN_16
            | HEAP_FREE_CHECKING_ENABLED);

    /* Initialise the Heap parameters */
    (*Heap).VirtualMemoryThreshold = (ROUND_UP(
        (*Parameters).VirtualMemoryThreshold as usize,
        core::mem::size_of::<HEAP_ENTRY>() as usize,
    ) >> HEAP_ENTRY_SHIFT) as ULONG;
    (*Heap).SegmentReserve = (*Parameters).SegmentReserve as usize;
    (*Heap).SegmentCommit = (*Parameters).SegmentCommit as usize;
    (*Heap).DeCommitFreeBlockThreshold = DeCommitFreeBlockThreshold;
    (*Heap).DeCommitTotalFreeThreshold =
        ((*Parameters).DeCommitTotalFreeThreshold >> HEAP_ENTRY_SHIFT) as usize;
    (*Heap).MaximumAllocationSize = (*Parameters).MaximumAllocationSize as usize;
    (*Heap).CommitRoutine = (*Parameters).CommitRoutine;

    /* Initialise the Heap validation info */
    (*Heap).HeaderValidateCopy = core::ptr::null_mut();
    (*Heap).HeaderValidateLength = HeaderSize as USHORT;

    /* Initialise the Heap Lock */
    if (Flags & HEAP_NO_SERIALIZE) == 0 && (Flags & HEAP_LOCK_USER_ALLOCATED) == 0 {
        Status = RtlInitializeHeapLock(&mut Lock);
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }
    (*Heap).LockVariable = Lock;

    /* Initialise the Heap alignment info */
    if (Flags & HEAP_CREATE_ALIGN_16) != 0 {
        (*Heap).AlignMask = !15usize;
        (*Heap).AlignRound = 15 + core::mem::size_of::<HEAP_ENTRY>();
    } else {
        (*Heap).AlignMask = !(core::mem::size_of::<HEAP_ENTRY>() - 1);
        (*Heap).AlignRound = 2 * core::mem::size_of::<HEAP_ENTRY>() - 1;
    }

    if (Flags & HEAP_TAIL_CHECKING_ENABLED) != 0 {
        (*Heap).AlignRound += core::mem::size_of::<HEAP_ENTRY>();
    }

    /* Initialise the Heap Segment list */
    Index = 0;
    while (Index as usize) < HEAP_SEGMENTS {
        (*Heap).Segments[Index as usize] = core::ptr::null_mut();
        Index += 1;
    }

    /* Initialise the free entry lists. */
    InitializeListHead(core::ptr::addr_of_mut!((*Heap).FreeLists));
    RtlInitializeBitMap(
        core::ptr::addr_of_mut!((*Heap).FreeHintBitmap),
        free_hint_slot(Heap, DeCommitFreeBlockThreshold as usize)
            as PULONG,
        DeCommitFreeBlockThreshold as ULONG,
    );
    RtlClearAllBits(core::ptr::addr_of_mut!((*Heap).FreeHintBitmap));
    RtlZeroMemory(
        free_hint_slot(Heap, 0) as *mut _,
        core::mem::size_of_val(&(*free_hint_slot(Heap, 0))) * DeCommitFreeBlockThreshold as usize,
    );

    /* Initialise the Heap Virtual Allocated Blocks list */
    InitializeListHead(core::ptr::addr_of_mut!((*Heap).VirtualAllocdBlocks));

    /* Initialise the Heap UnCommitted Region lists */
    InitializeListHead(core::ptr::addr_of_mut!((*Heap).UCRSegments));
    InitializeListHead(core::ptr::addr_of_mut!((*Heap).UCRList));

    /* Register the initial Heap UnCommitted Region Descriptors */
    Index = 0;
    while Index < NumUCRs {
        InsertTailList(
            core::ptr::addr_of_mut!((*Heap).UCRList),
            core::ptr::addr_of_mut!((*UcrDescriptor.add(Index as usize)).ListEntry),
        );
        Index += 1;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlpInsertFreeBlockHelper(
    Heap: PHEAP,
    FreeEntry: PHEAP_FREE_ENTRY,
    BlockSize: SIZE_T,
    NoFill: BOOLEAN,
) {
    let HintIndex: ULONG;
    let NextHintIndex: ULONG;

    if NoFill == 0 {
        (*FreeEntry).Flags &= !((HEAP_ENTRY_FILL_PATTERN
            | HEAP_ENTRY_EXTRA_PRESENT
            | HEAP_ENTRY_BUSY) as UCHAR);

        if (*Heap).Flags & HEAP_FREE_CHECKING_ENABLED != 0 {
            rtl_fill_memory_ulong(
                FreeEntry.add(1) as PVOID,
                ((BlockSize << HEAP_ENTRY_SHIFT) - core::mem::size_of::<_HEAP_FREE_ENTRY>()) as _,
                ARENA_FREE_FILLER,
            );

            (*FreeEntry).Flags |= HEAP_ENTRY_FILL_PATTERN as UCHAR;
        }
    } else {
        (*FreeEntry).Flags &= HEAP_ENTRY_LAST_ENTRY as UCHAR;
    }

    if BlockSize > (*Heap).DeCommitFreeBlockThreshold as SIZE_T {
        let mut ListEntry: PLIST_ENTRY = *free_hint_slot(Heap, 0) ;

        if ListEntry.is_null() {
            (*free_hint_slot(Heap, 0)) = core::ptr::addr_of_mut!((*FreeEntry).FreeList);
            RtlSetBit(core::ptr::addr_of_mut!((*Heap).FreeHintBitmap), 0);
            InsertTailList(
                core::ptr::addr_of_mut!((*Heap).FreeLists),
                core::ptr::addr_of_mut!((*FreeEntry).FreeList),
            );
            return;
        }

        while ListEntry != core::ptr::addr_of_mut!((*Heap).FreeLists) {
            let PreviousEntry = (ListEntry as *mut u8)
                .sub(core::mem::offset_of!(_HEAP_FREE_ENTRY, FreeList))
                as PHEAP_FREE_ENTRY;
            if (*PreviousEntry).Size as SIZE_T >= BlockSize {
                break;
            }

            ListEntry = (*ListEntry).Flink;
        }

        InsertTailList(ListEntry, core::ptr::addr_of_mut!((*FreeEntry).FreeList));

        if (*free_hint_slot(Heap, 0)) == ListEntry {
            (*free_hint_slot(Heap, 0)) = core::ptr::addr_of_mut!((*FreeEntry).FreeList);
        }

        return;
    }

    HintIndex = (BlockSize - 1) as ULONG;

    if !(*free_hint_slot(Heap, HintIndex as usize)).is_null() {
        InsertHeadList(
            *free_hint_slot(Heap, HintIndex as usize) ,
            core::ptr::addr_of_mut!((*FreeEntry).FreeList),
        );

        return;
    }

    if IsListEmpty(core::ptr::addr_of_mut!((*Heap).FreeLists)) {
        InsertHeadList(
            core::ptr::addr_of_mut!((*Heap).FreeLists),
            core::ptr::addr_of_mut!((*FreeEntry).FreeList),
        );
        RtlSetBit(core::ptr::addr_of_mut!((*Heap).FreeHintBitmap), HintIndex);
        (*free_hint_slot(Heap, HintIndex as usize)) = core::ptr::addr_of_mut!((*FreeEntry).FreeList);
        return;
    }

    NextHintIndex = RtlFindSetBits(core::ptr::addr_of_mut!((*Heap).FreeHintBitmap), 1, HintIndex);
    if (NextHintIndex == 0) || (NextHintIndex > HintIndex) {
        InsertTailList(
            *free_hint_slot(Heap, NextHintIndex as usize) ,
            core::ptr::addr_of_mut!((*FreeEntry).FreeList),
        );
    } else {
        InsertTailList(
            core::ptr::addr_of_mut!((*Heap).FreeLists),
            core::ptr::addr_of_mut!((*FreeEntry).FreeList),
        );
    }

    RtlSetBit(core::ptr::addr_of_mut!((*Heap).FreeHintBitmap), HintIndex);
    (*free_hint_slot(Heap, HintIndex as usize)) = core::ptr::addr_of_mut!((*FreeEntry).FreeList);
}

#[no_mangle]
pub unsafe extern "system" fn RtlpInsertFreeBlock(
    Heap: PHEAP,
    mut FreeEntry: PHEAP_FREE_ENTRY,
    mut BlockSize: SIZE_T,
) -> PHEAP_FREE_ENTRY {
    let mut Size: USHORT;
    let mut PreviousSize: USHORT;
    let SegmentOffset: UCHAR;
    let Flags: UCHAR;
    let Segment: PHEAP_SEGMENT;
    let mut LastEntry: PHEAP_FREE_ENTRY;

    (*Heap).TotalFreeSize += BlockSize;

    LastEntry = FreeEntry;
    Flags = (*FreeEntry).Flags;
    PreviousSize = (*FreeEntry).PreviousSize;
    SegmentOffset = (*FreeEntry).SegmentOffset;
    Segment = (*Heap).Segments[SegmentOffset as usize];

    while BlockSize != 0 {
        if BlockSize > HEAP_MAX_BLOCK_SIZE as SIZE_T {
            Size = HEAP_MAX_BLOCK_SIZE as USHORT;

            if BlockSize == (HEAP_MAX_BLOCK_SIZE + 1) as SIZE_T {
                Size -= 16;
            }

            (*FreeEntry).Flags = 0;
        } else {
            Size = BlockSize as USHORT;
            (*FreeEntry).Flags = Flags;
        }

        (*FreeEntry).Size = Size;
        (*FreeEntry).PreviousSize = PreviousSize;
        (*FreeEntry).SegmentOffset = SegmentOffset;

        RtlpInsertFreeBlockHelper(Heap, FreeEntry, Size as SIZE_T, 0);

        PreviousSize = Size;
        BlockSize -= Size as SIZE_T;

        LastEntry = FreeEntry;
        FreeEntry = (FreeEntry as PHEAP_ENTRY).add(Size as usize) as PHEAP_FREE_ENTRY;

        if (FreeEntry as PHEAP_ENTRY) >= (*Segment).LastValidEntry {
            return LastEntry;
        }
    }

    if (Flags & HEAP_ENTRY_LAST_ENTRY as UCHAR) == 0 {
        (*FreeEntry).PreviousSize = PreviousSize;
    }

    LastEntry
}

unsafe fn RtlpRemoveFreeBlock(Heap: PHEAP, FreeEntry: PHEAP_FREE_ENTRY, NoFill: BOOLEAN) {
    let Result: SIZE_T;
    let RealSize: SIZE_T;
    let HintIndex: ULONG;

    if (*FreeEntry).Size as usize > (*Heap).DeCommitFreeBlockThreshold {
        HintIndex = 0;
    } else {
        HintIndex = (*FreeEntry).Size as ULONG - 1;
    }

    if (*free_hint_slot(Heap, HintIndex as usize)) == core::ptr::addr_of_mut!((*FreeEntry).FreeList) {
        let mut NewHintEntry: PHEAP_FREE_ENTRY = core::ptr::null_mut();
        if (*FreeEntry).FreeList.Flink != core::ptr::addr_of_mut!((*Heap).FreeLists) {
            NewHintEntry = (((*FreeEntry).FreeList.Flink as *mut u8)
                .offset(-(core::mem::offset_of!(_HEAP_FREE_ENTRY, FreeList) as isize)))
                as PHEAP_FREE_ENTRY;
            if HintIndex != 0 && (*NewHintEntry).Size != (*FreeEntry).Size {
                NewHintEntry = core::ptr::null_mut();
            }
        }

        if !NewHintEntry.is_null() {
            (*free_hint_slot(Heap, HintIndex as usize)) = core::ptr::addr_of_mut!((*NewHintEntry).FreeList);
        } else {
            (*free_hint_slot(Heap, HintIndex as usize)) = core::ptr::null_mut();
            RtlClearBit(core::ptr::addr_of_mut!((*Heap).FreeHintBitmap), HintIndex);
        }
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*FreeEntry).FreeList));

    if NoFill == 0 && ((*FreeEntry).Flags & HEAP_ENTRY_FILL_PATTERN as u8) != 0 {
        RealSize = (((*FreeEntry).Size as SIZE_T) << HEAP_ENTRY_SHIFT)
            - core::mem::size_of::<_HEAP_FREE_ENTRY>();

        let mut RealSize = RealSize;
        if ((*FreeEntry).Flags & HEAP_ENTRY_EXTRA_PRESENT as u8) != 0
            && RealSize > core::mem::size_of::<HEAP_FREE_ENTRY_EXTRA>()
        {
            RealSize -= core::mem::size_of::<HEAP_FREE_ENTRY_EXTRA>();
        }

        Result = RtlCompareMemoryUlong(
            FreeEntry.add(1) as *mut core::ffi::c_void,
            RealSize as _,
            ARENA_FREE_FILLER as ULONG,
        ) as usize;

        let _ = Result;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpGetSizeOfBigBlock(HeapEntry: PHEAP_ENTRY) -> SIZE_T {
    let VirtualEntry: PHEAP_VIRTUAL_ALLOC_ENTRY = (HeapEntry as *mut u8)
        .offset(-(core::mem::offset_of!(_HEAP_VIRTUAL_ALLOC_ENTRY, BusyBlock) as isize))
        as PHEAP_VIRTUAL_ALLOC_ENTRY;

    (*VirtualEntry).CommitSize - (*HeapEntry).Size as SIZE_T
}
#[no_mangle]
pub unsafe extern "system" fn RtlpCreateUnCommittedRange(Segment: PHEAP_SEGMENT) -> PHEAP_UCR_DESCRIPTOR {
    let Entry: PLIST_ENTRY;
    let mut UcrDescriptor: PHEAP_UCR_DESCRIPTOR;
    let mut UcrSegment: PHEAP_UCR_SEGMENT;
    let Heap: PHEAP = (*Segment).Heap;
    let mut ReserveSize: SIZE_T = (16 * PAGE_SIZE) as SIZE_T;
    let mut CommitSize: SIZE_T = (1 * PAGE_SIZE) as SIZE_T;
    let mut Status: NTSTATUS;

    if IsListEmpty(core::ptr::addr_of_mut!((*Heap).UCRList)) {
        UcrSegment = ((*Heap).UCRSegments.Flink as PCHAR)
            .offset(-(core::mem::offset_of!(_HEAP_UCR_SEGMENT, ListEntry) as isize))
            as PHEAP_UCR_SEGMENT;

        if IsListEmpty(core::ptr::addr_of_mut!((*Heap).UCRSegments))
            || (*UcrSegment).ReservedSize == (*UcrSegment).CommittedSize
        {
            UcrSegment = core::ptr::null_mut();
            Status = ZwAllocateVirtualMemory(
                -1isize as HANDLE,
                core::ptr::addr_of_mut!(UcrSegment) as *mut PVOID,
                0,
                core::ptr::addr_of_mut!(ReserveSize) as *mut _,
                MEM_RESERVE,
                PAGE_READWRITE,
            );

            if !NT_SUCCESS(Status) {
                return core::ptr::null_mut();
            }

            Status = ZwAllocateVirtualMemory(
                -1isize as HANDLE,
                core::ptr::addr_of_mut!(UcrSegment) as *mut PVOID,
                0,
                core::ptr::addr_of_mut!(CommitSize) as *mut _,
                MEM_COMMIT,
                PAGE_READWRITE,
            );

            if !NT_SUCCESS(Status) {
                ZwFreeVirtualMemory(
                    -1isize as HANDLE,
                    core::ptr::addr_of_mut!(UcrSegment) as *mut PVOID,
                    core::ptr::addr_of_mut!(ReserveSize) as *mut _,
                    MEM_RELEASE,
                );
                return core::ptr::null_mut();
            }

            (*UcrSegment).ReservedSize = ReserveSize;
            (*UcrSegment).CommittedSize = CommitSize;

            InsertHeadList(
                core::ptr::addr_of_mut!((*Heap).UCRSegments),
                core::ptr::addr_of_mut!((*UcrSegment).ListEntry),
            );

            UcrDescriptor = UcrSegment.offset(1) as PHEAP_UCR_DESCRIPTOR;
        } else {
            UcrDescriptor =
                (UcrSegment as PCHAR).offset((*UcrSegment).CommittedSize as isize)
                    as PHEAP_UCR_DESCRIPTOR;
            Status = ZwAllocateVirtualMemory(
                -1isize as HANDLE,
                core::ptr::addr_of_mut!(UcrDescriptor) as *mut PVOID,
                0,
                core::ptr::addr_of_mut!(CommitSize) as *mut _,
                MEM_COMMIT,
                PAGE_READWRITE,
            );

            if !NT_SUCCESS(Status) {
                return core::ptr::null_mut();
            }

            (*UcrSegment).CommittedSize += CommitSize;
        }

        while (UcrDescriptor.offset(1) as PCHAR)
            <= (UcrSegment as PCHAR).offset((*UcrSegment).CommittedSize as isize)
        {
            InsertTailList(
                core::ptr::addr_of_mut!((*Heap).UCRList),
                core::ptr::addr_of_mut!((*UcrDescriptor).ListEntry),
            );
            UcrDescriptor = UcrDescriptor.offset(1);
        }
    }

    Entry = RemoveHeadList(core::ptr::addr_of_mut!((*Heap).UCRList));
    UcrDescriptor = (Entry as PCHAR)
        .offset(-(core::mem::offset_of!(_HEAP_UCR_DESCRIPTOR, ListEntry) as isize))
        as PHEAP_UCR_DESCRIPTOR;
    UcrDescriptor
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDestroyUnCommittedRange(
    Segment: PHEAP_SEGMENT,
    UcrDescriptor: PHEAP_UCR_DESCRIPTOR,
) {
    (*UcrDescriptor).Address = core::ptr::null_mut();
    (*UcrDescriptor).Size = 0;

    InsertHeadList(
        core::ptr::addr_of_mut!((*(*Segment).Heap).UCRList),
        core::ptr::addr_of_mut!((*UcrDescriptor).ListEntry),
    );
}

#[no_mangle]
pub unsafe extern "system" fn RtlpInsertUnCommittedPages(
    Segment: PHEAP_SEGMENT,
    mut Address: ULONG_PTR,
    mut Size: SIZE_T,
) {
    let mut Current: PLIST_ENTRY;
    let mut UcrDescriptor: PHEAP_UCR_DESCRIPTOR;

    Current = (*Segment).UCRSegmentList.Flink;
    while Current != core::ptr::addr_of_mut!((*Segment).UCRSegmentList) {
        UcrDescriptor = (Current as PCHAR)
            .offset(-(core::mem::offset_of!(_HEAP_UCR_DESCRIPTOR, SegmentEntry) as isize))
            as PHEAP_UCR_DESCRIPTOR;

        if (*UcrDescriptor).Address as ULONG_PTR > Address {
            if (Address + Size) == ((*UcrDescriptor).Address as ULONG_PTR) {
                (*UcrDescriptor).Address = Address as PVOID;
                (*UcrDescriptor).Size += Size;
                return;
            }

            break;
        } else if (((*UcrDescriptor).Address as ULONG_PTR) + (*UcrDescriptor).Size) == Address {
            Address = (*UcrDescriptor).Address as ULONG_PTR;
            Size += (*UcrDescriptor).Size;

            Current = (*Current).Flink;

            RemoveEntryList(core::ptr::addr_of_mut!((*UcrDescriptor).SegmentEntry));
            RtlpDestroyUnCommittedRange(Segment, UcrDescriptor);

            (*Segment).NumberOfUnCommittedRanges -= 1;
        } else {
            Current = (*Current).Flink;
        }
    }

    UcrDescriptor = RtlpCreateUnCommittedRange(Segment);
    if UcrDescriptor.is_null() {
        return;
    }

    (*UcrDescriptor).Address = Address as PVOID;
    (*UcrDescriptor).Size = Size;

    InsertTailList(Current, core::ptr::addr_of_mut!((*UcrDescriptor).SegmentEntry));

    (*Segment).NumberOfUnCommittedRanges += 1;
}

unsafe fn RtlpFindAndCommitPages(
    Heap: PHEAP,
    Segment: PHEAP_SEGMENT,
    Size: PSIZE_T,
    AddressRequested: PVOID,
) -> PHEAP_FREE_ENTRY {
    let mut Current: PLIST_ENTRY;
    let Status: NTSTATUS;

    Current = (*Segment).UCRSegmentList.Flink;
    while Current != core::ptr::addr_of_mut!((*Segment).UCRSegmentList) {
        let UcrDescriptor: PHEAP_UCR_DESCRIPTOR = (Current as *mut u8)
            .offset(-(core::mem::offset_of!(_HEAP_UCR_DESCRIPTOR, SegmentEntry) as isize))
            as PHEAP_UCR_DESCRIPTOR;

        if (*UcrDescriptor).Size >= *Size
            && ((*UcrDescriptor).Address == AddressRequested || AddressRequested.is_null())
        {
            let mut GuardEntry: PHEAP_ENTRY;
            let FreeEntry: PHEAP_ENTRY;
            let mut Address: PVOID = (*UcrDescriptor).Address;

            if let Some(CommitRoutine) = (*Heap).CommitRoutine {
                Status = CommitRoutine(Heap as PVOID, core::ptr::addr_of_mut!(Address), Size as *mut _);
            } else {
                Status = ZwAllocateVirtualMemory(
                    NtCurrentProcess(),
                    core::ptr::addr_of_mut!(Address),
                    0,
                    Size as *mut _,
                    MEM_COMMIT as u32,
                    PAGE_READWRITE as u32,
                );
            }

            if !NT_SUCCESS(Status) {
                return core::ptr::null_mut();
            }

            (*Segment).NumberOfUnCommittedPages -= (*Size / PAGE_SIZE as usize) as u32;

            (*UcrDescriptor).Address =
                ((*UcrDescriptor).Address as usize + *Size) as PVOID;
            (*UcrDescriptor).Size -= *Size;

            GuardEntry = (Address as PHEAP_ENTRY).offset(-1);

            if (*GuardEntry).PreviousSize == 1 {
                GuardEntry = GuardEntry.offset(-1);
                *Size += core::mem::size_of::<_HEAP_ENTRY>();
            }

            FreeEntry = GuardEntry;

            (*FreeEntry).Flags &= !((HEAP_ENTRY_BUSY | HEAP_ENTRY_LAST_ENTRY) as u8);
            (*FreeEntry).Size = ((*Size) >> HEAP_ENTRY_SHIFT) as u16;

            if (*UcrDescriptor).Size == 0 {
                (*FreeEntry).Size += 1;
                *Size += core::mem::size_of::<_HEAP_ENTRY>();

                if (*UcrDescriptor).Address == (*Segment).LastValidEntry as PVOID {
                    (*FreeEntry).Flags = HEAP_ENTRY_LAST_ENTRY as u8;
                } else {
                    let NextEntry: PHEAP_ENTRY = (*UcrDescriptor).Address as PHEAP_ENTRY;
                    (*NextEntry).PreviousSize = (*FreeEntry).Size;
                }

                RemoveEntryList(core::ptr::addr_of_mut!((*UcrDescriptor).SegmentEntry));

                RtlpDestroyUnCommittedRange(Segment, UcrDescriptor);
                (*Segment).NumberOfUnCommittedRanges -= 1;
            } else {
                GuardEntry = ((*UcrDescriptor).Address as PHEAP_ENTRY).offset(-1);
                (*GuardEntry).Flags = (HEAP_ENTRY_LAST_ENTRY | HEAP_ENTRY_BUSY) as u8;
                (*GuardEntry).Size = 1;
                (*GuardEntry).PreviousSize = (*FreeEntry).Size;
                (*GuardEntry).SegmentOffset = (*FreeEntry).SegmentOffset;
            }

            return FreeEntry as PHEAP_FREE_ENTRY;
        }

        Current = (*Current).Flink;
    }

    core::ptr::null_mut()
}

unsafe fn RtlpDeCommitFreeBlock(Heap: PHEAP, FreeEntry: PHEAP_FREE_ENTRY, Size: SIZE_T) {
    let Segment: PHEAP_SEGMENT;
    let NextEntry: PHEAP_ENTRY;
    let _GuardEntry: PHEAP_ENTRY;
    let _UcrDescriptor: PHEAP_UCR_DESCRIPTOR;
    let mut PrecedingSize: SIZE_T;
    let _DecommitSize: SIZE_T;
    let mut DecommitBase: ULONG_PTR;
    let DecommitEnd: ULONG_PTR;
    let _Status: NTSTATUS;

    /* We can't decommit if there is a commit routine! */
    if (*Heap).CommitRoutine.is_some() {
        /* Just add it back the usual way */
        RtlpInsertFreeBlock(Heap, FreeEntry, Size);
        return;
    }

    /* Get the segment */
    Segment = (*Heap).Segments[(*FreeEntry).SegmentOffset as usize];

    /* Get the preceding entry */
    DecommitBase = ROUND_UP(FreeEntry as ULONG_PTR, PAGE_SIZE as ULONG_PTR);
    PrecedingSize = ((DecommitBase as PHEAP_ENTRY).offset_from(FreeEntry as PHEAP_ENTRY)) as SIZE_T;

    if PrecedingSize == 0 {
        /* We need some space in order to insert our guard entry */
        DecommitBase += PAGE_SIZE as ULONG_PTR;
        PrecedingSize += (PAGE_SIZE >> HEAP_ENTRY_SHIFT) as SIZE_T;
    }

    /* Do we really have a next entry */
    if RtlpIsLastCommittedEntry(FreeEntry as PHEAP_ENTRY) != 0 {
        /* No, Decommit till the next UCR. */
        DecommitEnd = PAGE_ROUND_UP(
            (FreeEntry as PHEAP_ENTRY).add((*FreeEntry).Size as usize) as ULONG_PTR,
        );
        NextEntry = core::ptr::null_mut();
    } else {
        NextEntry = (FreeEntry as PHEAP_ENTRY).add(Size as usize);
        DecommitEnd = PAGE_ROUND_DOWN(NextEntry as ULONG_PTR);

        /* Can we make a free entry out of what's left ? */
        if NextEntry.offset_from(DecommitEnd as PHEAP_ENTRY) == 1 {
            /* Nope. Let's keep one page before this */
            return RtlpDeCommitFreeBlockTail(
                Heap, FreeEntry, Size, Segment, NextEntry, DecommitBase, PrecedingSize,
                DecommitEnd - PAGE_SIZE as ULONG_PTR,
            );
        }
    }

    RtlpDeCommitFreeBlockTail(
        Heap, FreeEntry, Size, Segment, NextEntry, DecommitBase, PrecedingSize, DecommitEnd,
    )
}

unsafe fn RtlpDeCommitFreeBlockTail(
    Heap: PHEAP,
    mut FreeEntry: PHEAP_FREE_ENTRY,
    Size: SIZE_T,
    Segment: PHEAP_SEGMENT,
    NextEntry: PHEAP_ENTRY,
    mut DecommitBase: ULONG_PTR,
    mut PrecedingSize: SIZE_T,
    DecommitEnd: ULONG_PTR,
) {
    let UcrDescriptor: PHEAP_UCR_DESCRIPTOR;
    let mut DecommitSize: SIZE_T;
    let Status: NTSTATUS;
    let mut GuardEntry: PHEAP_ENTRY;

    if DecommitEnd <= DecommitBase {
        /* There's nothing left to decommit. */
        RtlpInsertFreeBlock(Heap, FreeEntry, Size);
        return;
    }

    DecommitSize = DecommitEnd - DecommitBase;

    /* A decommit is necessary. Create a UCR descriptor */
    UcrDescriptor = RtlpCreateUnCommittedRange(Segment);
    if UcrDescriptor.is_null() {
        RtlpInsertFreeBlock(Heap, FreeEntry, PrecedingSize);
        return;
    }

    /* Decommit the memory */
    Status = ZwFreeVirtualMemory(
        NtCurrentProcess(),
        core::ptr::addr_of_mut!(DecommitBase) as *mut PVOID,
        core::ptr::addr_of_mut!(DecommitSize) as *mut _,
        MEM_DECOMMIT,
    );

    /* Delete that UCR. This is needed to assure there is an unused UCR entry in the list */
    RtlpDestroyUnCommittedRange(Segment, UcrDescriptor);

    if !NT_SUCCESS(Status) {
        RtlpInsertFreeBlock(Heap, FreeEntry, Size);
        return;
    }

    /* Insert uncommitted pages */
    RtlpInsertUnCommittedPages(Segment, DecommitBase, DecommitSize);
    (*Segment).NumberOfUnCommittedPages += (DecommitSize / PAGE_SIZE as SIZE_T) as ULONG;

    /* Insert our guard entry before this */
    GuardEntry = (DecommitBase as PHEAP_ENTRY).offset(-1);
    (*GuardEntry).Size = 1;
    (*GuardEntry).Flags = (HEAP_ENTRY_BUSY | HEAP_ENTRY_LAST_ENTRY) as UCHAR;
    (*GuardEntry).SegmentOffset = (*FreeEntry).SegmentOffset;

    /* Now see what's really behind us */
    PrecedingSize -= 1;
    match PrecedingSize {
        1 => {
            /* No space left for a free entry. Make this another guard entry */
            (*GuardEntry).PreviousSize = 1;
            GuardEntry = GuardEntry.offset(-1);
            (*GuardEntry).Size = 1;
            (*GuardEntry).Flags = (HEAP_ENTRY_BUSY | HEAP_ENTRY_LAST_ENTRY) as UCHAR;
            (*GuardEntry).SegmentOffset = (*FreeEntry).SegmentOffset;
            /* Fall-through */
            (*GuardEntry).PreviousSize = (*FreeEntry).PreviousSize;
        }
        0 => {
            /* There was just enough space four our guard entry */
            (*GuardEntry).PreviousSize = (*FreeEntry).PreviousSize;
        }
        _ => {
            /* We can insert this as a free entry */
            (*GuardEntry).PreviousSize = PrecedingSize as USHORT;
            (*FreeEntry).Size = PrecedingSize as USHORT;
            (*FreeEntry).Flags &= !(HEAP_ENTRY_LAST_ENTRY as UCHAR);
            FreeEntry = RtlpCoalesceFreeBlocks(
                Heap,
                FreeEntry,
                core::ptr::addr_of_mut!(PrecedingSize),
                0u8,
            );
            RtlpInsertFreeBlock(Heap, FreeEntry, PrecedingSize);
        }
    }

    /* Now the next one */
    if !NextEntry.is_null() {
        let mut NextSize: SIZE_T = NextEntry.offset_from(DecommitEnd as PHEAP_ENTRY) as SIZE_T;
        if NextSize != 0 {
            let mut NextFreeEntry: PHEAP_FREE_ENTRY = DecommitEnd as PHEAP_FREE_ENTRY;

            /* Adjust size of this free entry and insert it */
            (*NextFreeEntry).Flags = 0;
            (*NextFreeEntry).PreviousSize = 0;
            (*NextFreeEntry).SegmentOffset = (*Segment).Entry.SegmentOffset;
            (*NextFreeEntry).Size = NextSize as USHORT;

            (*NextEntry).PreviousSize = NextSize as USHORT;

            NextFreeEntry = RtlpCoalesceFreeBlocks(
                Heap,
                NextFreeEntry,
                core::ptr::addr_of_mut!(NextSize),
                0u8,
            );
            RtlpInsertFreeBlock(Heap, NextFreeEntry, NextSize);
        } else {
            (*NextEntry).PreviousSize = 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlpInitializeHeapSegment(
    Heap: PHEAP,
    Segment: PHEAP_SEGMENT,
    SegmentIndex: UCHAR,
    SegmentFlags: ULONG,
    SegmentReserve: SIZE_T,
    SegmentCommit: SIZE_T,
) -> NTSTATUS {
    if (Heap as PHEAP_SEGMENT) != Segment {
        (*Segment).Entry.Size = (ROUND_UP(
            core::mem::size_of::<HEAP_SEGMENT>() as SIZE_T,
            core::mem::size_of::<HEAP_ENTRY>() as SIZE_T,
        ) >> HEAP_ENTRY_SHIFT) as USHORT;
        (*Segment).Entry.Flags = HEAP_ENTRY_BUSY as UCHAR;
        (*Segment).Entry.SmallTagIndex = (((*Segment).Entry.Size & 0xff) as UCHAR)
            ^ ((((*Segment).Entry.Size >> 8) & 0xff) as UCHAR)
            ^ (*Segment).Entry.Flags;
        (*Segment).Entry.PreviousSize = 0;
        (*Segment).Entry.SegmentOffset = SegmentIndex;
        (*Segment).Entry.UnusedBytes = 0;
    }

    (*Segment).SegmentSignature = HEAP_SEGMENT_SIGNATURE;
    (*Segment).SegmentFlags = SegmentFlags;
    (*Segment).Heap = Heap;
    (*Heap).Segments[SegmentIndex as usize] = Segment;

    (*Segment).BaseAddress = Segment as PVOID;
    (*Segment).NumberOfPages = (SegmentReserve >> PAGE_SHIFT) as ULONG;

    (*Segment).FirstEntry = core::ptr::addr_of_mut!((*Segment).Entry)
        .offset((*Segment).Entry.Size as isize);
    (*Segment).LastValidEntry = ((Segment as ULONG_PTR) + SegmentReserve) as PHEAP_ENTRY;

    (*Segment).NumberOfUnCommittedPages =
        ((SegmentReserve - SegmentCommit) >> PAGE_SHIFT) as ULONG;
    (*Segment).NumberOfUnCommittedRanges = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*Segment).UCRSegmentList));

    if (((*Segment).Entry.Size as SIZE_T) << HEAP_ENTRY_SHIFT) < SegmentCommit {
        let mut FreeEntry: PHEAP_ENTRY;

        if (*Segment).NumberOfUnCommittedPages != 0 {
            let mut GuardEntry: PHEAP_ENTRY = core::ptr::addr_of_mut!((*Segment).Entry)
                .offset((SegmentCommit >> HEAP_ENTRY_SHIFT) as isize - 1);
            let PreviousSize: SIZE_T;

            (*GuardEntry).Size = 1;
            (*GuardEntry).Flags = (HEAP_ENTRY_BUSY | HEAP_ENTRY_LAST_ENTRY) as UCHAR;
            (*GuardEntry).SegmentOffset = SegmentIndex;
            PreviousSize = GuardEntry.offset_from((*Segment).FirstEntry) as SIZE_T;

            match PreviousSize {
                1 => {
                    (*GuardEntry).PreviousSize = PreviousSize as USHORT;

                    GuardEntry = GuardEntry.offset(-1);
                    (*GuardEntry).Size = 1;
                    (*GuardEntry).Flags = (HEAP_ENTRY_BUSY | HEAP_ENTRY_LAST_ENTRY) as UCHAR;
                    (*GuardEntry).SegmentOffset = SegmentIndex;

                    (*GuardEntry).PreviousSize = (*Segment).Entry.Size;
                }
                0 => {
                    (*GuardEntry).PreviousSize = (*Segment).Entry.Size;
                }
                _ => {
                    FreeEntry = (*Segment).FirstEntry;
                    (*FreeEntry).PreviousSize = (*Segment).Entry.Size;
                    (*FreeEntry).SegmentOffset = SegmentIndex;
                    (*FreeEntry).Size = PreviousSize as USHORT;
                    (*FreeEntry).Flags = 0;

                    FreeEntry = RtlpInsertFreeBlock(
                        Heap,
                        FreeEntry as PHEAP_FREE_ENTRY,
                        PreviousSize,
                    ) as PHEAP_ENTRY;
                    (*GuardEntry).PreviousSize = (*FreeEntry).Size;
                }
            }
        } else {
            FreeEntry = (*Segment).FirstEntry;
            (*FreeEntry).PreviousSize = (*Segment).Entry.Size;
            (*FreeEntry).SegmentOffset = SegmentIndex;
            (*FreeEntry).Flags = HEAP_ENTRY_LAST_ENTRY as UCHAR;
            (*FreeEntry).Size =
                ((SegmentCommit >> HEAP_ENTRY_SHIFT) as USHORT) - (*Segment).Entry.Size;

            RtlpInsertFreeBlock(
                Heap,
                FreeEntry as PHEAP_FREE_ENTRY,
                (*FreeEntry).Size as SIZE_T,
            );
        }
    }

    if (*Segment).NumberOfUnCommittedPages != 0 {
        RtlpInsertUnCommittedPages(
            Segment,
            (Segment as ULONG_PTR) + SegmentCommit,
            SegmentReserve - SegmentCommit,
        );
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlpDestroyHeapSegment(Segment: PHEAP_SEGMENT) {
    let mut Size: SIZE_T = 0;

    if (*Segment).SegmentFlags & HEAP_USER_ALLOCATED != 0 {
        return;
    }

    let mut BaseAddress: PVOID = (*Segment).BaseAddress;

    let _Status = ZwFreeVirtualMemory(
        -1isize as HANDLE,
        &mut BaseAddress,
        core::ptr::addr_of_mut!(Size) as *mut _,
        MEM_RELEASE,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RtlpCoalesceHeap(_Heap: PHEAP) -> PHEAP_FREE_ENTRY {
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn RtlpCoalesceFreeBlocks(
    Heap: PHEAP,
    mut FreeEntry: PHEAP_FREE_ENTRY,
    FreeSize: PSIZE_T,
    mut Remove: BOOLEAN,
) -> PHEAP_FREE_ENTRY {
    let CurrentEntry: PHEAP_FREE_ENTRY;
    let NextEntry: PHEAP_FREE_ENTRY;
    let mut SegmentOffset: UCHAR;

    /* Get the previous entry */
    CurrentEntry = ((FreeEntry as PHEAP_ENTRY).offset(-((*FreeEntry).PreviousSize as isize)))
        as PHEAP_FREE_ENTRY;

    /* Check it */
    if CurrentEntry != FreeEntry
        && ((*CurrentEntry).Flags & HEAP_ENTRY_BUSY) == 0
        && (*FreeSize + (*CurrentEntry).Size as SIZE_T) <= HEAP_MAX_BLOCK_SIZE as SIZE_T
    {
        /* Remove it if asked for */
        if Remove != 0 {
            RtlpRemoveFreeBlock(Heap, FreeEntry, 0u8);
            (*Heap).TotalFreeSize -= (*FreeEntry).Size as SIZE_T;

            /* Remove it only once! */
            Remove = 0u8;
        }

        /* Remove previous entry too */
        RtlpRemoveFreeBlock(Heap, CurrentEntry, 0u8);

        /* Copy flags */
        (*CurrentEntry).Flags = (*FreeEntry).Flags & HEAP_ENTRY_LAST_ENTRY;

        /* Advance FreeEntry and update sizes */
        FreeEntry = CurrentEntry;
        *FreeSize = *FreeSize + (*CurrentEntry).Size as SIZE_T;
        (*Heap).TotalFreeSize -= (*CurrentEntry).Size as SIZE_T;
        (*FreeEntry).Size = *FreeSize as USHORT;

        /* Also update previous size if needed */
        if ((*FreeEntry).Flags & HEAP_ENTRY_LAST_ENTRY) == 0 {
            (*((FreeEntry as PHEAP_ENTRY).offset(*FreeSize as isize))).PreviousSize =
                *FreeSize as USHORT;
        } else {
            SegmentOffset = (*FreeEntry).SegmentOffset;
            let _ = SegmentOffset;
        }
    }

    /* Check the next block if it exists */
    if ((*FreeEntry).Flags & HEAP_ENTRY_LAST_ENTRY) == 0 {
        NextEntry = ((FreeEntry as PHEAP_ENTRY).offset(*FreeSize as isize)) as PHEAP_FREE_ENTRY;

        if ((*NextEntry).Flags & HEAP_ENTRY_BUSY) == 0
            && (*NextEntry).Size as SIZE_T + *FreeSize <= HEAP_MAX_BLOCK_SIZE as SIZE_T
        {
            /* Remove it if asked for */
            if Remove != 0 {
                RtlpRemoveFreeBlock(Heap, FreeEntry, 0u8);
                (*Heap).TotalFreeSize -= (*FreeEntry).Size as SIZE_T;
            }

            /* Copy flags */
            (*FreeEntry).Flags = (*NextEntry).Flags & HEAP_ENTRY_LAST_ENTRY;

            /* Remove next entry now */
            RtlpRemoveFreeBlock(Heap, NextEntry, 0u8);

            /* Update sizes */
            *FreeSize = *FreeSize + (*NextEntry).Size as SIZE_T;
            (*Heap).TotalFreeSize -= (*NextEntry).Size as SIZE_T;
            (*FreeEntry).Size = *FreeSize as USHORT;

            /* Also update previous size if needed */
            if ((*FreeEntry).Flags & HEAP_ENTRY_LAST_ENTRY) == 0 {
                (*((FreeEntry as PHEAP_ENTRY).offset(*FreeSize as isize))).PreviousSize =
                    *FreeSize as USHORT;
            } else {
                SegmentOffset = (*FreeEntry).SegmentOffset;
                let _ = SegmentOffset;
            }
        }
    }

    FreeEntry
}

unsafe fn RtlpExtendHeap(Heap: PHEAP, Size: SIZE_T) -> PHEAP_FREE_ENTRY {
    let Pages: ULONG;
    let mut Index: UCHAR;
    let mut EmptyIndex: UCHAR;
    let mut FreeSize: SIZE_T;
    let mut CommitSize: SIZE_T;
    let mut ReserveSize: SIZE_T;
    let mut Segment: PHEAP_SEGMENT;
    let mut FreeEntry: PHEAP_FREE_ENTRY;
    let mut Status: NTSTATUS;

    /* Calculate amount in pages */
    Pages = ((Size + PAGE_SIZE as SIZE_T - 1) / PAGE_SIZE as SIZE_T) as ULONG;
    FreeSize = Pages as usize * PAGE_SIZE;

    /* Find an empty segment */
    EmptyIndex = HEAP_SEGMENTS as UCHAR;
    Index = 0;
    while (Index as usize) < HEAP_SEGMENTS {
        Segment = (*Heap).Segments[Index as usize];

        /* Check if its size suits us */
        if !Segment.is_null() && Pages <= (*Segment).NumberOfUnCommittedPages {
            /* Commit needed amount */
            FreeEntry = RtlpFindAndCommitPages(
                Heap,
                Segment,
                core::ptr::addr_of_mut!(FreeSize),
                core::ptr::null_mut(),
            );

            /* Coalesce it with adjacent entries */
            if !FreeEntry.is_null() {
                FreeSize = FreeSize >> HEAP_ENTRY_SHIFT;
                FreeEntry = RtlpCoalesceFreeBlocks(
                    Heap,
                    FreeEntry,
                    core::ptr::addr_of_mut!(FreeSize),
                    0u8,
                );
                RtlpInsertFreeBlock(Heap, FreeEntry, FreeSize);
                return FreeEntry;
            }
        } else if Segment.is_null() && EmptyIndex == HEAP_SEGMENTS as UCHAR {
            /* Remember the first unused segment index */
            EmptyIndex = Index;
        }

        Index += 1;
    }

    /* No luck, need to grow the heap */
    if ((*Heap).Flags & HEAP_GROWABLE) != 0 && EmptyIndex != HEAP_SEGMENTS as UCHAR {
        Segment = core::ptr::null_mut();

        /* Reserve the memory */
        if (Size + PAGE_SIZE as SIZE_T) <= (*Heap).SegmentReserve {
            ReserveSize = (*Heap).SegmentReserve;
        } else {
            ReserveSize = Size + PAGE_SIZE as SIZE_T;
        }

        Status = ZwAllocateVirtualMemory(
            NtCurrentProcess(),
            core::ptr::addr_of_mut!(Segment) as *mut PVOID,
            0,
            core::ptr::addr_of_mut!(ReserveSize) as *mut _,
            MEM_RESERVE,
            PAGE_READWRITE,
        );

        /* If it failed, retry again with a half division algorithm */
        while !NT_SUCCESS(Status) && ReserveSize != Size + PAGE_SIZE as SIZE_T {
            ReserveSize /= 2;

            if ReserveSize < (Size + PAGE_SIZE as SIZE_T) {
                ReserveSize = Size + PAGE_SIZE as SIZE_T;
            }

            Status = ZwAllocateVirtualMemory(
                NtCurrentProcess(),
                core::ptr::addr_of_mut!(Segment) as *mut PVOID,
                0,
                core::ptr::addr_of_mut!(ReserveSize) as *mut _,
                MEM_RESERVE,
                PAGE_READWRITE,
            );
        }

        /* Proceed only if it's success */
        if NT_SUCCESS(Status) {
            (*Heap).SegmentReserve += ReserveSize;

            /* Now commit the memory */
            if (Size + PAGE_SIZE as SIZE_T) <= (*Heap).SegmentCommit {
                CommitSize = (*Heap).SegmentCommit;
            } else {
                CommitSize = Size + PAGE_SIZE as SIZE_T;
            }

            Status = ZwAllocateVirtualMemory(
                NtCurrentProcess(),
                core::ptr::addr_of_mut!(Segment) as *mut PVOID,
                0,
                core::ptr::addr_of_mut!(CommitSize) as *mut _,
                MEM_COMMIT,
                PAGE_READWRITE,
            );

            /* Initialize heap segment if commit was successful */
            if NT_SUCCESS(Status) {
                Status = RtlpInitializeHeapSegment(
                    Heap,
                    Segment,
                    EmptyIndex,
                    0,
                    ReserveSize,
                    CommitSize,
                );
            }

            /* If everything worked - cool */
            if NT_SUCCESS(Status) {
                return (*Segment).FirstEntry as PHEAP_FREE_ENTRY;
            }

            /* Nope, we failed. Free memory */
            ZwFreeVirtualMemory(
                NtCurrentProcess(),
                core::ptr::addr_of_mut!(Segment) as *mut PVOID,
                core::ptr::addr_of_mut!(ReserveSize) as *mut _,
                MEM_RELEASE,
            );
        }
    }

    if RtlpGetMode() == 1i32 as KPROCESSOR_MODE {
        /* If coalescing on free is disabled in usermode, then do it here */
        if ((*Heap).Flags & HEAP_DISABLE_COALESCE_ON_FREE) != 0 {
            FreeEntry = RtlpCoalesceHeap(Heap);

            /* If it's a suitable one - return it */
            if !FreeEntry.is_null() && (*FreeEntry).Size as SIZE_T >= Size {
                return FreeEntry;
            }
        }
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlCreateHeap(
    mut Flags: ULONG,
    Addr: PVOID,
    mut TotalSize: SIZE_T,
    mut CommitSize: SIZE_T,
    Lock: PVOID,
    mut Parameters: PRTL_HEAP_PARAMETERS,
) -> HANDLE {
    let mut CommittedAddress: PVOID = core::ptr::null_mut();
    let UncommittedAddress: PVOID;
    let mut Heap: PHEAP = core::ptr::null_mut();
    let mut SafeParams: RTL_HEAP_PARAMETERS = core::mem::zeroed();
    let MaximumUserModeAddress: ULONG_PTR;
    let mut SystemInformation: SYSTEM_BASIC_INFORMATION = core::mem::zeroed();
    let mut MemoryInfo: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
    let NtGlobalFlags: ULONG = RtlGetNtGlobalFlags();
    let mut HeapSegmentFlags: ULONG = 0;
    let mut Status: NTSTATUS;
    let MaxBlockSize: ULONG;

    /* Check for a special heap */
    if RtlpPageHeapEnabled != 0 && Addr.is_null() && Lock.is_null() {
        Heap = RtlpPageHeapCreate(Flags, Addr, TotalSize, CommitSize, Lock, Parameters) as PHEAP;
        if !Heap.is_null() {
            return Heap as HANDLE;
        }

        /* Reset a special Parameters == -1 hack */
        if Parameters as ULONG_PTR == (-1isize) as ULONG_PTR {
            Parameters = core::ptr::null_mut();
        }
    }

    /* Check validation flags */
    if (Flags & HEAP_SKIP_VALIDATION_CHECKS) == 0 && (Flags & !HEAP_CREATE_VALID_MASK) != 0 {
        Flags &= HEAP_CREATE_VALID_MASK;
    }

    /* Capture parameters */
    if !Parameters.is_null() {
        if (*Parameters).Length as usize == core::mem::size_of::<RTL_HEAP_PARAMETERS>() {
            RtlCopyMemory(
                core::ptr::addr_of_mut!(SafeParams) as PVOID,
                Parameters as PVOID,
                core::mem::size_of::<RTL_HEAP_PARAMETERS>() as SIZE_T,
            );
        }
    }

    Parameters = core::ptr::addr_of_mut!(SafeParams);

    /* Check global flags */
    if NtGlobalFlags & FLG_HEAP_DISABLE_COALESCING != 0 {
        Flags |= HEAP_DISABLE_COALESCE_ON_FREE;
    }

    if NtGlobalFlags & FLG_HEAP_ENABLE_FREE_CHECK != 0 {
        Flags |= HEAP_FREE_CHECKING_ENABLED;
    }

    if NtGlobalFlags & FLG_HEAP_ENABLE_TAIL_CHECK != 0 {
        Flags |= HEAP_TAIL_CHECKING_ENABLED;
    }

    if RtlpGetMode() == 1i32 as KPROCESSOR_MODE {
        if NtGlobalFlags & FLG_HEAP_VALIDATE_ALL != 0 {
            Flags |= HEAP_VALIDATE_ALL_ENABLED;
        }

        if NtGlobalFlags & FLG_HEAP_VALIDATE_PARAMETERS != 0 {
            Flags |= HEAP_VALIDATE_PARAMETERS_ENABLED;
        }

        if NtGlobalFlags & FLG_USER_STACK_TRACE_DB != 0 {
            Flags |= HEAP_CAPTURE_STACK_BACKTRACES;
        }
    }

    /* Set tunable parameters */
    RtlpSetHeapParameters(Parameters);

    /* Get the max um address */
    Status = ZwQuerySystemInformation(
        SystemBasicInformation,
        core::ptr::addr_of_mut!(SystemInformation) as PVOID,
        core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    MaximumUserModeAddress = SystemInformation.MaximumUserModeAddress as usize;

    /* Calculate max alloc size */
    if (*Parameters).MaximumAllocationSize == 0 {
        (*Parameters).MaximumAllocationSize =
            (MaximumUserModeAddress - 0x10000 as ULONG_PTR - PAGE_SIZE as ULONG_PTR) as _;
    }

    MaxBlockSize = (0x80000 - PAGE_SIZE) as u32;

    if (*Parameters).VirtualMemoryThreshold == 0
        || (*Parameters).VirtualMemoryThreshold > MaxBlockSize as _
    {
        (*Parameters).VirtualMemoryThreshold = MaxBlockSize as _;
    }

    if (*Parameters).DeCommitFreeBlockThreshold != PAGE_SIZE as _ {
        (*Parameters).DeCommitFreeBlockThreshold = PAGE_SIZE as _;
    }

    /* Check reserve/commit sizes and set default values */
    if CommitSize == 0 {
        CommitSize = PAGE_SIZE as SIZE_T;
        if TotalSize != 0 {
            TotalSize = ROUND_UP(TotalSize, PAGE_SIZE as SIZE_T);
        } else {
            TotalSize = 64 * PAGE_SIZE as SIZE_T;
        }
    } else {
        CommitSize = ROUND_UP(CommitSize, PAGE_SIZE as SIZE_T);

        if TotalSize != 0 {
            TotalSize = ROUND_UP(TotalSize, PAGE_SIZE as SIZE_T);
        } else {
            TotalSize = ROUND_UP(CommitSize, 16 * PAGE_SIZE as SIZE_T);
        }
    }

    /* Call special heap */
    if RtlpHeapIsSpecial(Flags) != 0 {
        return RtlDebugCreateHeap(Flags, Addr, TotalSize, CommitSize, Lock, Parameters);
    }

    /* Without serialization, a lock makes no sense */
    if (Flags & HEAP_NO_SERIALIZE) != 0 && !Lock.is_null() {
        return core::ptr::null_mut();
    }

    /* See if we are already provided with an address for the heap */
    if !Addr.is_null() {
        if (*Parameters).CommitRoutine.is_some() {
            if (Flags & HEAP_GROWABLE) != 0
                || (*Parameters).InitialCommit == 0
                || (*Parameters).InitialReserve == 0
                || (*Parameters).InitialCommit > (*Parameters).InitialReserve
            {
                return core::ptr::null_mut();
            }

            CommittedAddress = Addr;
            UncommittedAddress = (Addr as PCHAR).add((*Parameters).InitialCommit as usize) as PVOID;
            TotalSize = (*Parameters).InitialReserve as usize;

            RtlZeroMemory(CommittedAddress, PAGE_SIZE as SIZE_T);
        } else {
            Status = ZwQueryVirtualMemory(
                (-1isize) as HANDLE,
                Addr,
                MemoryBasicInformation,
                core::ptr::addr_of_mut!(MemoryInfo) as PVOID,
                core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as _,
                core::ptr::null_mut(),
            );

            if !NT_SUCCESS(Status) {
                return core::ptr::null_mut();
            }

            if MemoryInfo.BaseAddress != Addr || MemoryInfo.State == MEM_FREE {
                return core::ptr::null_mut();
            }

            CommittedAddress = Addr;

            if MemoryInfo.State == MEM_COMMIT {
                RtlZeroMemory(CommittedAddress, PAGE_SIZE as SIZE_T);

                CommitSize = MemoryInfo.RegionSize as usize;
                TotalSize = CommitSize;
                UncommittedAddress = (Addr as PCHAR).add(CommitSize as usize) as PVOID;

                Status = ZwQueryVirtualMemory(
                    (-1isize) as HANDLE,
                    UncommittedAddress,
                    MemoryBasicInformation,
                    core::ptr::addr_of_mut!(MemoryInfo) as PVOID,
                    core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as _,
                    core::ptr::null_mut(),
                );

                if NT_SUCCESS(Status) && MemoryInfo.State == MEM_RESERVE {
                    TotalSize += MemoryInfo.RegionSize as usize;
                }
            } else {
                CommitSize = PAGE_SIZE as SIZE_T;
                UncommittedAddress = Addr;
            }
        }

        HeapSegmentFlags = HEAP_USER_ALLOCATED;
        Heap = Addr as PHEAP;
    } else {
        if (*Parameters).CommitRoutine.is_some() {
            return core::ptr::null_mut();
        }

        Status = ZwAllocateVirtualMemory(
            (-1isize) as HANDLE,
            core::ptr::addr_of_mut!(Heap) as *mut PVOID,
            0,
            core::ptr::addr_of_mut!(TotalSize) as *mut _,
            MEM_RESERVE,
            PAGE_READWRITE,
        );

        if !NT_SUCCESS(Status) {
            return core::ptr::null_mut();
        }

        CommittedAddress = Heap as PVOID;
        UncommittedAddress = Heap as PVOID;
    }

    /* Check if we need to commit something */
    if CommittedAddress == UncommittedAddress {
        Status = ZwAllocateVirtualMemory(
            (-1isize) as HANDLE,
            core::ptr::addr_of_mut!(CommittedAddress),
            0,
            core::ptr::addr_of_mut!(CommitSize) as *mut _,
            MEM_COMMIT,
            PAGE_READWRITE,
        );

        if !NT_SUCCESS(Status) {
            if Addr.is_null() {
                ZwFreeVirtualMemory(
                    (-1isize) as HANDLE,
                    core::ptr::addr_of_mut!(Heap) as *mut PVOID,
                    core::ptr::addr_of_mut!(TotalSize) as *mut _,
                    MEM_RELEASE,
                );
            }

            return core::ptr::null_mut();
        }

    }

    /* Initialize the heap */
    Status = RtlpInitializeHeap(Heap, Flags, Lock as PHEAP_LOCK, Parameters);
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    /* Initialize heap's first segment */
    Status = RtlpInitializeHeapSegment(
        Heap,
        Heap as PHEAP_SEGMENT,
        0,
        HeapSegmentFlags,
        TotalSize,
        CommitSize,
    );
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    /* Add heap to process list in case of usermode heap */
    if RtlpGetMode() == 1i32 as KPROCESSOR_MODE {
        RtlpAddHeapToProcessList(Heap);
    }

    Heap as HANDLE
}
#[no_mangle]
pub unsafe extern "system" fn RtlDestroyHeap(HeapPtr: HANDLE) -> HANDLE {
    let Heap: PHEAP = HeapPtr as PHEAP;
    let mut Current: PLIST_ENTRY;
    let mut UcrSegment: PHEAP_UCR_SEGMENT;
    let mut VirtualEntry: PHEAP_VIRTUAL_ALLOC_ENTRY;
    let mut BaseAddress: PVOID;
    let mut Size: SIZE_T;
    let mut i: LONG;
    let mut Segment: PHEAP_SEGMENT;

    if HeapPtr.is_null() {
        return core::ptr::null_mut();
    }

    /* Call page heap routine if required */
    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpPageHeapDestroy(HeapPtr);
    }

    /* Call special heap */
    if RtlpHeapIsSpecial((*Heap).Flags) != 0 {
        if RtlDebugDestroyHeap(Heap as PVOID) == 0 {
            return HeapPtr;
        }
    }

    /* Check for a process heap */
    if RtlpGetMode() == 1i32 as KPROCESSOR_MODE
        && HeapPtr == (*NtCurrentPeb()).ProcessHeap
    {
        return HeapPtr;
    }

    /* Free up all big allocations */
    Current = (*Heap).VirtualAllocdBlocks.Flink;
    while Current != core::ptr::addr_of_mut!((*Heap).VirtualAllocdBlocks) {
        VirtualEntry = (Current as usize
            - core::mem::offset_of!(_HEAP_VIRTUAL_ALLOC_ENTRY, Entry))
            as PHEAP_VIRTUAL_ALLOC_ENTRY;
        BaseAddress = VirtualEntry as PVOID;
        Current = (*Current).Flink;
        Size = 0;
        ZwFreeVirtualMemory(
            NtCurrentProcess(),
            &mut BaseAddress,
            core::ptr::addr_of_mut!(Size) as *mut _,
            MEM_RELEASE,
        );
    }

    /* Delete tags and remove heap from the process heaps list in user mode */
    if RtlpGetMode() == 1i32 as KPROCESSOR_MODE {
        // FIXME DestroyTags
        RtlpRemoveHeapFromProcessList(Heap);
    }

    /* Delete the heap lock */
    if (*Heap).Flags & HEAP_NO_SERIALIZE == 0 {
        /* Delete it if it wasn't user allocated */
        if (*Heap).Flags & HEAP_LOCK_USER_ALLOCATED == 0 {
            RtlDeleteHeapLock((*Heap).LockVariable);
        }

        /* Clear out the lock variable */
        (*Heap).LockVariable = core::ptr::null_mut();
    }

    /* Free UCR segments if any were created */
    Current = (*Heap).UCRSegments.Flink;
    while Current != core::ptr::addr_of_mut!((*Heap).UCRSegments) {
        UcrSegment = (Current as usize
            - core::mem::offset_of!(_HEAP_UCR_SEGMENT, ListEntry))
            as PHEAP_UCR_SEGMENT;

        /* Advance to the next descriptor */
        Current = (*Current).Flink;

        BaseAddress = UcrSegment as PVOID;
        Size = 0;

        /* Release that memory */
        ZwFreeVirtualMemory(
            NtCurrentProcess(),
            &mut BaseAddress,
            core::ptr::addr_of_mut!(Size) as *mut _,
            MEM_RELEASE,
        );
    }

    /* Go through segments and destroy them */
    i = HEAP_SEGMENTS as LONG - 1;
    while i >= 0 {
        Segment = (*Heap).Segments[i as usize];
        if !Segment.is_null() {
            RtlpDestroyHeapSegment(Segment);
        }
        i -= 1;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlpSplitEntry(
    Heap: PHEAP,
    Flags: ULONG,
    FreeBlock: PHEAP_FREE_ENTRY,
    AllocationSize: SIZE_T,
    Index: SIZE_T,
    Size: SIZE_T,
) -> PHEAP_ENTRY {
    let SplitBlock: PHEAP_FREE_ENTRY;
    let SplitBlock2: PHEAP_FREE_ENTRY;
    let mut FreeFlags: UCHAR;
    let mut EntryFlags: UCHAR = HEAP_ENTRY_BUSY as UCHAR;
    let InUseEntry: PHEAP_ENTRY;
    let mut FreeSize: SIZE_T;
    let SegmentOffset: UCHAR;

    if (Flags & HEAP_EXTRA_FLAGS_MASK) != 0 || (*Heap).PseudoTagEntries != core::ptr::null_mut() {
        EntryFlags |= HEAP_ENTRY_EXTRA_PRESENT as UCHAR;
    }

    EntryFlags |= ((Flags & HEAP_SETTABLE_USER_FLAGS) >> 4) as UCHAR;

    FreeFlags = (*FreeBlock).Flags;
    (*Heap).TotalFreeSize -= (*FreeBlock).Size as SIZE_T;

    InUseEntry = FreeBlock as PHEAP_ENTRY;
    (*InUseEntry).Flags = EntryFlags;
    (*InUseEntry).SmallTagIndex = 0;

    FreeSize = (*InUseEntry).Size as SIZE_T - Index;

    (*InUseEntry).Size = Index as USHORT;
    (*InUseEntry).UnusedBytes = (AllocationSize - Size) as UCHAR;

    if FreeSize != 0 {
        if FreeSize == 1 {
            (*InUseEntry).Size += 1;
            (*InUseEntry).UnusedBytes += core::mem::size_of::<HEAP_ENTRY>() as UCHAR;
        } else {
            SplitBlock = InUseEntry.add(Index) as PHEAP_FREE_ENTRY;

            (*SplitBlock).Flags = FreeFlags;
            (*SplitBlock).SegmentOffset = (*InUseEntry).SegmentOffset;
            (*SplitBlock).Size = FreeSize as USHORT;
            (*SplitBlock).PreviousSize = Index as USHORT;

            if (FreeFlags & HEAP_ENTRY_LAST_ENTRY as UCHAR) != 0 {
                RtlpInsertFreeBlockHelper(Heap, SplitBlock, FreeSize, 0u8);
                (*Heap).TotalFreeSize += FreeSize;
            } else {
                SplitBlock2 = (SplitBlock as PHEAP_ENTRY).add(FreeSize) as PHEAP_FREE_ENTRY;

                if (*SplitBlock2).Flags & HEAP_ENTRY_BUSY as UCHAR != 0 {
                    (*SplitBlock2).PreviousSize = FreeSize as USHORT;
                    RtlpInsertFreeBlockHelper(Heap, SplitBlock, FreeSize, 0u8);
                    (*Heap).TotalFreeSize += FreeSize;
                } else {
                    (*SplitBlock).Flags = (*SplitBlock2).Flags;

                    RtlpRemoveFreeBlock(Heap, SplitBlock2, 0u8);

                    FreeSize += (*SplitBlock2).Size as SIZE_T;
                    (*Heap).TotalFreeSize -= (*SplitBlock2).Size as SIZE_T;

                    if FreeSize <= HEAP_MAX_BLOCK_SIZE as SIZE_T {
                        (*SplitBlock).Size = FreeSize as USHORT;

                        if (*SplitBlock).Flags & HEAP_ENTRY_LAST_ENTRY as UCHAR == 0 {
                            (*((SplitBlock as PHEAP_ENTRY).add(FreeSize) as PHEAP_FREE_ENTRY))
                                .PreviousSize = FreeSize as USHORT;
                        }

                        RtlpInsertFreeBlockHelper(Heap, SplitBlock, FreeSize as USHORT as SIZE_T, 0u8);

                        (*Heap).TotalFreeSize += FreeSize;
                    } else {
                        RtlpInsertFreeBlock(Heap, SplitBlock, FreeSize);
                    }
                }
            }

            FreeFlags = 0;
            if (*SplitBlock).Flags & HEAP_ENTRY_LAST_ENTRY as UCHAR != 0 {
                SegmentOffset = (*SplitBlock).SegmentOffset;
                let _ = SegmentOffset;
            }
        }
    }

    if FreeFlags & HEAP_ENTRY_LAST_ENTRY as UCHAR != 0 {
        (*InUseEntry).Flags |= HEAP_ENTRY_LAST_ENTRY as UCHAR;
    }

    InUseEntry
}

unsafe fn RtlpAllocateNonDedicated(
    Heap: PHEAP,
    Flags: ULONG,
    Size: SIZE_T,
    AllocationSize: SIZE_T,
    Index: SIZE_T,
    HeapLocked: BOOLEAN,
) -> PVOID {
    let FreeBlock: PHEAP_FREE_ENTRY;

    /* Extend the heap */
    FreeBlock = RtlpExtendHeap(Heap, AllocationSize);

    /* Use the new biggest entry we've got */
    if !FreeBlock.is_null() {
        let InUseEntry: PHEAP_ENTRY;
        let Extra: PHEAP_ENTRY_EXTRA;

        RtlpRemoveFreeBlock(Heap, FreeBlock, 1u8);

        /* Split it */
        InUseEntry = RtlpSplitEntry(Heap, Flags, FreeBlock, AllocationSize, Index, Size);

        /* Release the lock */
        if HeapLocked != 0 {
            RtlLeaveHeapLock((*Heap).LockVariable);
        }

        /* Zero memory if that was requested */
        if (Flags & HEAP_ZERO_MEMORY) != 0 {
            RtlZeroMemory(InUseEntry.add(1) as PVOID, Size);
        } else if ((*Heap).Flags & HEAP_FREE_CHECKING_ENABLED) != 0 {
            /* Fill this block with a special pattern */
            rtl_fill_memory_ulong(InUseEntry.add(1) as PVOID, (Size & !0x3) as _, ARENA_INUSE_FILLER);
        }

        /* Fill tail of the block with a special pattern too if requested */
        if ((*Heap).Flags & HEAP_TAIL_CHECKING_ENABLED) != 0 {
            core::ptr::write_bytes(
                (InUseEntry.add(1) as *mut u8).add(Size),
                HEAP_TAIL_FILL as u8,
                core::mem::size_of::<HEAP_ENTRY>(),
            );
            (*InUseEntry).Flags |= HEAP_ENTRY_FILL_PATTERN as UCHAR;
        }

        /* Prepare extra if it's present */
        if ((*InUseEntry).Flags & HEAP_ENTRY_EXTRA_PRESENT as UCHAR) != 0 {
            Extra = RtlpGetExtraStuffPointer(InUseEntry);
            RtlZeroMemory(Extra as PVOID, core::mem::size_of::<HEAP_ENTRY_EXTRA>());

            // TODO: Tagging
        }

        /* Return pointer to the */
        return InUseEntry.add(1) as PVOID;
    }

    /* Really unfortunate, out of memory condition */
    RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_NO_MEMORY);

    /* Generate an exception */
    if (Flags & HEAP_GENERATE_EXCEPTIONS) != 0 {
        let mut ExceptionRecord: EXCEPTION_RECORD = core::mem::zeroed();

        ExceptionRecord.ExceptionCode = STATUS_NO_MEMORY;
        ExceptionRecord.ExceptionRecord = core::ptr::null_mut();
        ExceptionRecord.NumberParameters = 1;
        ExceptionRecord.ExceptionFlags = 0;
        ExceptionRecord.ExceptionInformation[0] = AllocationSize as _;

        RtlRaiseException(core::ptr::addr_of_mut!(ExceptionRecord));
    }

    /* Release the lock */
    if HeapLocked != 0 {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlAllocateHeap(
    HeapPtr: PVOID,
    mut Flags: ULONG,
    Size: SIZE_T,
) -> PVOID {
    let Heap: PHEAP = HeapPtr as PHEAP;
    let mut AllocationSize: SIZE_T;
    let Index: SIZE_T;
    let mut EntryFlags: UCHAR = HEAP_ENTRY_BUSY as UCHAR;
    let mut HeapLocked: BOOLEAN = 0u8;
    let mut VirtualBlock: PHEAP_VIRTUAL_ALLOC_ENTRY = core::ptr::null_mut();
    let Extra: PHEAP_ENTRY_EXTRA;
    let Status: NTSTATUS;

    /* Force flags */
    Flags |= (*Heap).ForceFlags;

    /* Call special heap */
    if RtlpHeapIsSpecial(Flags) != 0 {
        return RtlDebugAllocateHeap(Heap as PVOID, Flags, Size);
    }

    /* Check for the maximum size */
    if Size >= 0x80000000 {
        RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_NO_MEMORY);
        return core::ptr::null_mut();
    }

    /* Calculate allocation size and index */
    if Size != 0 {
        AllocationSize = Size;
    } else {
        AllocationSize = 1;
    }
    AllocationSize = (AllocationSize + (*Heap).AlignRound) & (*Heap).AlignMask;

    /* Add extra flags in case of settable user value feature is requested,
       or there is a tag (small or normal) or there is a request to
       capture stack backtraces */
    if (Flags & HEAP_EXTRA_FLAGS_MASK) != 0 || !(*Heap).PseudoTagEntries.is_null() {
        EntryFlags |= HEAP_ENTRY_EXTRA_PRESENT as UCHAR;
        AllocationSize += core::mem::size_of::<HEAP_ENTRY_EXTRA>() as SIZE_T;
    }

    /* Add settable user flags, if any */
    EntryFlags |= ((Flags & HEAP_SETTABLE_USER_FLAGS) >> 4) as UCHAR;

    Index = AllocationSize >> HEAP_ENTRY_SHIFT;

    /* Acquire the lock if necessary */
    if (Flags & HEAP_NO_SERIALIZE) == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = 1u8;
    }

    /* Depending on the size, the allocation is going to be done from dedicated,
       non-dedicated lists or a virtual block of memory */
    if Index <= (*Heap).VirtualMemoryThreshold as SIZE_T {
        let InUseEntry: PHEAP_ENTRY;
        let mut FreeEntry: PHEAP_FREE_ENTRY;

        /* First quick check: Anybody here ? */
        if IsListEmpty(core::ptr::addr_of_mut!((*Heap).FreeLists)) {
            return RtlpAllocateNonDedicated(Heap, Flags, Size, AllocationSize, Index, HeapLocked);
        }

        /* Second quick check: Is there someone for us ? */
        FreeEntry = (((*Heap).FreeLists.Blink as *mut u8)
            .sub(core::mem::offset_of!(_HEAP_FREE_ENTRY, FreeList)))
            as PHEAP_FREE_ENTRY;
        if (*FreeEntry).Size < Index as USHORT {
            return RtlpAllocateNonDedicated(Heap, Flags, Size, AllocationSize, Index, HeapLocked);
        }

        if Index > (*Heap).DeCommitFreeBlockThreshold as SIZE_T {
            /* Find an entry from the non dedicated list */
            FreeEntry = (((*free_hint_slot(Heap, 0)) as *mut u8)
                .sub(core::mem::offset_of!(_HEAP_FREE_ENTRY, FreeList)))
                as PHEAP_FREE_ENTRY;

            while (*FreeEntry).Size < Index as USHORT {
                FreeEntry = (((*FreeEntry).FreeList.Flink as *mut u8)
                    .sub(core::mem::offset_of!(_HEAP_FREE_ENTRY, FreeList)))
                    as PHEAP_FREE_ENTRY;
            }
        } else {
            /* Get the free entry from the hint */
            let HintIndex: ULONG = RtlFindSetBits(
                core::ptr::addr_of_mut!((*Heap).FreeHintBitmap),
                1,
                (Index - 1) as ULONG,
            );
            FreeEntry = (((*free_hint_slot(Heap, HintIndex as usize)) as *mut u8)
                .sub(core::mem::offset_of!(_HEAP_FREE_ENTRY, FreeList)))
                as PHEAP_FREE_ENTRY;
        }

        /* Remove the free block, split, profit. */
        RtlpRemoveFreeBlock(Heap, FreeEntry, 0u8);
        InUseEntry = RtlpSplitEntry(Heap, Flags, FreeEntry, AllocationSize, Index, Size);

        /* Release the lock */
        if HeapLocked != 0 {
            RtlLeaveHeapLock((*Heap).LockVariable);
        }

        /* Zero memory if that was requested */
        if (Flags & HEAP_ZERO_MEMORY) != 0 {
            RtlZeroMemory(InUseEntry.add(1) as PVOID, Size);
        } else if ((*Heap).Flags & HEAP_FREE_CHECKING_ENABLED) != 0 {
            rtl_fill_memory_ulong(InUseEntry.add(1) as PVOID, (Size & !0x3) as _, ARENA_INUSE_FILLER);
        }

        /* Fill tail of the block with a special pattern too if requested */
        if ((*Heap).Flags & HEAP_TAIL_CHECKING_ENABLED) != 0 {
            core::ptr::write_bytes(
                (InUseEntry.add(1) as *mut u8).add(Size),
                HEAP_TAIL_FILL as u8,
                core::mem::size_of::<HEAP_ENTRY>(),
            );
            (*InUseEntry).Flags |= HEAP_ENTRY_FILL_PATTERN as UCHAR;
        }

        /* Prepare extra if it's present */
        if ((*InUseEntry).Flags & HEAP_ENTRY_EXTRA_PRESENT as UCHAR) != 0 {
            Extra = RtlpGetExtraStuffPointer(InUseEntry);
            RtlZeroMemory(Extra as PVOID, core::mem::size_of::<HEAP_ENTRY_EXTRA>() as SIZE_T);
        }

        /* User data starts right after the entry's header */
        return InUseEntry.add(1) as PVOID;
    }

    if ((*Heap).Flags & HEAP_GROWABLE) != 0 {
        AllocationSize += (core::mem::size_of::<HEAP_VIRTUAL_ALLOC_ENTRY>()
            - core::mem::size_of::<HEAP_ENTRY>()) as SIZE_T;

        Status = ZwAllocateVirtualMemory(
            -1isize as HANDLE,
            core::ptr::addr_of_mut!(VirtualBlock) as *mut PVOID,
            0,
            core::ptr::addr_of_mut!(AllocationSize) as *mut _,
            MEM_COMMIT,
            PAGE_READWRITE,
        );

        if !NT_SUCCESS(Status) {
            if HeapLocked != 0 {
                RtlLeaveHeapLock((*Heap).LockVariable);
            }
            return core::ptr::null_mut();
        }

        /* Initialize the newly allocated block */
        (*VirtualBlock).BusyBlock.Size = (AllocationSize - Size) as USHORT;
        (*VirtualBlock).BusyBlock.Flags =
            EntryFlags | HEAP_ENTRY_VIRTUAL_ALLOC as UCHAR | HEAP_ENTRY_EXTRA_PRESENT as UCHAR;
        (*VirtualBlock).CommitSize = AllocationSize;
        (*VirtualBlock).ReserveSize = AllocationSize;

        /* Insert it into the list of virtual allocations */
        InsertTailList(
            core::ptr::addr_of_mut!((*Heap).VirtualAllocdBlocks),
            core::ptr::addr_of_mut!((*VirtualBlock).Entry),
        );

        /* Release the lock */
        if HeapLocked != 0 {
            RtlLeaveHeapLock((*Heap).LockVariable);
        }

        /* Return pointer to user data */
        return VirtualBlock.add(1) as PVOID;
    }

    /* Generate an exception */
    if (Flags & HEAP_GENERATE_EXCEPTIONS) != 0 {
        let mut ExceptionRecord: EXCEPTION_RECORD = core::mem::zeroed();

        ExceptionRecord.ExceptionCode = STATUS_NO_MEMORY;
        ExceptionRecord.ExceptionRecord = core::ptr::null_mut();
        ExceptionRecord.NumberParameters = 1;
        ExceptionRecord.ExceptionFlags = 0;
        ExceptionRecord.ExceptionInformation[0] = AllocationSize as _;

        RtlRaiseException(core::ptr::addr_of_mut!(ExceptionRecord));
    }

    RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_BUFFER_TOO_SMALL);

    /* Release the lock */
    if HeapLocked != 0 {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlFreeHeap(
    HeapPtr: HANDLE,
    mut Flags: ULONG,
    Ptr: PVOID,
) -> BOOLEAN {
    let Heap: PHEAP;
    let mut HeapEntry: PHEAP_ENTRY;
    let TagIndex: USHORT = 0;
    let mut BlockSize: SIZE_T;
    let VirtualEntry: PHEAP_VIRTUAL_ALLOC_ENTRY;
    let mut Locked: BOOLEAN = 0u8;
    let Status: NTSTATUS;

    /* Freeing NULL pointer is a legal operation */
    if Ptr.is_null() {
        return 1u8;
    }

    /* Get pointer to the heap and force flags */
    Heap = HeapPtr as PHEAP;
    Flags |= (*Heap).ForceFlags;

    /* Call special heap */
    if RtlpHeapIsSpecial(Flags) != 0 {
        return RtlDebugFreeHeap(Heap as PVOID, Flags, Ptr);
    }

    /* Get pointer to the heap entry */
    HeapEntry = (Ptr as PHEAP_ENTRY).offset(-1);

    /* Protect with SEH in case the pointer is not valid */
    if ((*HeapEntry).Flags & HEAP_ENTRY_BUSY as UCHAR) == 0
        || ((Ptr as ULONG_PTR) & 0x7) != 0
        || (*HeapEntry).SegmentOffset >= HEAP_SEGMENTS as UCHAR
    {
        /* This is an invalid block */
        RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_INVALID_PARAMETER);
        return 0u8;
    }

    /* Lock if necessary */
    if (Flags & HEAP_NO_SERIALIZE) == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        Locked = 1u8;
    }

    if ((*HeapEntry).Flags & HEAP_ENTRY_VIRTUAL_ALLOC as UCHAR) != 0 {
        /* Big allocation */
        VirtualEntry = (HeapEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_HEAP_VIRTUAL_ALLOC_ENTRY, BusyBlock) as isize))
            as PHEAP_VIRTUAL_ALLOC_ENTRY;

        /* Remove it from the list */
        RemoveEntryList(core::ptr::addr_of_mut!((*VirtualEntry).Entry));

        BlockSize = 0;
        let mut VirtualPtr = VirtualEntry as PVOID;
        Status = ZwFreeVirtualMemory(
            (-1isize) as HANDLE,
            core::ptr::addr_of_mut!(VirtualPtr),
            core::ptr::addr_of_mut!(BlockSize) as *mut _,
            MEM_RELEASE,
        );

        if !NT_SUCCESS(Status) {
            RtlSetLastWin32ErrorAndNtStatusFromNtStatus(Status);
        }
    } else {
        /* Normal allocation */
        BlockSize = (*HeapEntry).Size as SIZE_T;

        /* Coalesce in kernel mode, and in usermode if it's not disabled */
        if RtlpGetMode() == 0i32 as KPROCESSOR_MODE
            || (RtlpGetMode() == 1i32 as KPROCESSOR_MODE
                && ((*Heap).Flags & HEAP_DISABLE_COALESCE_ON_FREE) == 0)
        {
            HeapEntry = RtlpCoalesceFreeBlocks(
                Heap,
                HeapEntry as PHEAP_FREE_ENTRY,
                core::ptr::addr_of_mut!(BlockSize) as *mut _,
                0u8,
            ) as PHEAP_ENTRY;
        }

        /* See if we should decommit this block */
        if BlockSize >= (*Heap).DeCommitFreeBlockThreshold as SIZE_T
            || (*Heap).TotalFreeSize + BlockSize >= (*Heap).DeCommitTotalFreeThreshold as SIZE_T
        {
            RtlpDeCommitFreeBlock(Heap, HeapEntry as PHEAP_FREE_ENTRY, BlockSize);
        } else {
            /* Insert into the free list */
            RtlpInsertFreeBlock(Heap, HeapEntry as PHEAP_FREE_ENTRY, BlockSize);

            if RtlpGetMode() == 1i32 as KPROCESSOR_MODE && TagIndex != 0 {
                // FIXME: Tagging
            }
        }
    }

    /* Release the heap lock */
    if Locked != 0 {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    1u8
}
#[no_mangle]
pub unsafe extern "system" fn RtlpGrowBlockInPlace(
    Heap: PHEAP,
    Flags: ULONG,
    InUseEntry: PHEAP_ENTRY,
    Size: SIZE_T,
    mut Index: SIZE_T,
) -> BOOLEAN {
    let EntryFlags: UCHAR;
    let mut RememberFlags: UCHAR;
    let mut FreeEntry: PHEAP_FREE_ENTRY;
    let UnusedEntry: PHEAP_FREE_ENTRY;
    let FollowingEntry: PHEAP_FREE_ENTRY;
    let mut FreeSize: SIZE_T;
    let PrevSize: SIZE_T;
    let mut TailPart: SIZE_T;
    let mut AddedSize: SIZE_T = 0;
    let OldExtra: PHEAP_ENTRY_EXTRA;
    let NewExtra: PHEAP_ENTRY_EXTRA;
    let SegmentOffset: UCHAR;

    /* We can't grow beyond specified threshold */
    if Index > (*Heap).VirtualMemoryThreshold as SIZE_T {
        return 0u8;
    }

    /* Get entry flags */
    EntryFlags = (*InUseEntry).Flags;

    if RtlpIsLastCommittedEntry(InUseEntry) != 0 {
        /* There is no next block, just uncommitted space. Calculate how much is needed */
        FreeSize = (Index - (*InUseEntry).Size as SIZE_T) << HEAP_ENTRY_SHIFT;
        FreeSize = ROUND_UP(FreeSize as usize, PAGE_SIZE as usize) as SIZE_T;

        /* Find and commit those pages */
        FreeEntry = RtlpFindAndCommitPages(
            Heap,
            (*Heap).Segments[(*InUseEntry).SegmentOffset as usize],
            &mut FreeSize,
            PAGE_ROUND_UP(InUseEntry.add((*InUseEntry).Size as usize) as ULONG_PTR) as PVOID,
        );

        /* Fail if it failed... */
        if FreeEntry.is_null() {
            return 0u8;
        }

        /* It was successful, perform coalescing */
        FreeSize = FreeSize >> HEAP_ENTRY_SHIFT;
        FreeEntry = RtlpCoalesceFreeBlocks(Heap, FreeEntry, &mut FreeSize, 0u8);

        /* Check if it's enough */
        if FreeSize + ((*InUseEntry).Size as SIZE_T) < Index {
            /* Still not enough */
            RtlpInsertFreeBlock(Heap, FreeEntry, FreeSize);
            (*Heap).TotalFreeSize += FreeSize;
            return 0u8;
        }

        /* Remember flags of this free entry */
        RememberFlags = (*FreeEntry).Flags;

        /* Sum up sizes */
        FreeSize += (*InUseEntry).Size as SIZE_T;
    } else {
        FreeEntry = InUseEntry.add((*InUseEntry).Size as usize) as PHEAP_FREE_ENTRY;

        /* The next block indeed exists. Check if it's free or in use */
        if (*FreeEntry).Flags & HEAP_ENTRY_BUSY != 0 {
            return 0u8;
        }

        /* Next entry is free, check if it can fit the block we need */
        FreeSize = (*InUseEntry).Size as SIZE_T + (*FreeEntry).Size as SIZE_T;
        if FreeSize < Index {
            return 0u8;
        }

        /* Remember flags of this free entry */
        RememberFlags = (*FreeEntry).Flags;

        /* Remove this block from the free list */
        RtlpRemoveFreeBlock(Heap, FreeEntry, 0u8);
        (*Heap).TotalFreeSize -= (*FreeEntry).Size as SIZE_T;
    }

    PrevSize = (((*InUseEntry).Size as SIZE_T) << HEAP_ENTRY_SHIFT)
        - (*InUseEntry).UnusedBytes as SIZE_T;
    FreeSize -= Index;

    /* Don't produce too small blocks */
    if FreeSize <= 2 {
        Index += FreeSize;
        FreeSize = 0;
    }

    /* Process extra stuff */
    if EntryFlags & HEAP_ENTRY_EXTRA_PRESENT != 0 {
        /* Calculate pointers */
        OldExtra = InUseEntry.add((*InUseEntry).Size as usize).offset(-1) as PHEAP_ENTRY_EXTRA;
        NewExtra = InUseEntry.add(Index as usize).offset(-1) as PHEAP_ENTRY_EXTRA;

        /* Copy contents */
        *NewExtra = *OldExtra;
    }

    /* Update sizes */
    (*InUseEntry).Size = Index as USHORT;
    (*InUseEntry).UnusedBytes = ((Index << HEAP_ENTRY_SHIFT) - Size) as UCHAR;

    /* Check if there is a free space remaining after merging those blocks */
    if FreeSize == 0 {
        /* Update flags and sizes */
        (*InUseEntry).Flags |= RememberFlags & HEAP_ENTRY_LAST_ENTRY as UCHAR;

        /* Either update previous size of the next entry or mark it as a last
           entry in the segment */
        if RememberFlags & HEAP_ENTRY_LAST_ENTRY == 0 {
            (*InUseEntry.add((*InUseEntry).Size as usize)).PreviousSize = (*InUseEntry).Size;
        } else {
            SegmentOffset = (*InUseEntry).SegmentOffset;
            let _ = SegmentOffset;
        }
    } else {
        /* Complex case, we need to split the block to give unused free space
           back to the heap */
        UnusedEntry = InUseEntry.add(Index as usize) as PHEAP_FREE_ENTRY;
        (*UnusedEntry).PreviousSize = Index as USHORT;
        (*UnusedEntry).SegmentOffset = (*InUseEntry).SegmentOffset;

        /* Update the following block or set the last entry in the segment */
        if RememberFlags & HEAP_ENTRY_LAST_ENTRY != 0 {
            SegmentOffset = (*UnusedEntry).SegmentOffset;
            let _ = SegmentOffset;

            /* Set flags and size */
            (*UnusedEntry).Flags = RememberFlags;
            (*UnusedEntry).Size = FreeSize as USHORT;

            /* Insert it to the heap and update total size */
            RtlpInsertFreeBlockHelper(Heap, UnusedEntry, FreeSize, 0u8);
            (*Heap).TotalFreeSize += FreeSize;
        } else {
            /* There is a block after this one */
            FollowingEntry =
                (UnusedEntry as PHEAP_ENTRY).add(FreeSize as usize) as PHEAP_FREE_ENTRY;

            if (*FollowingEntry).Flags & HEAP_ENTRY_BUSY != 0 {
                /* Update flags and set size of the unused space entry */
                (*UnusedEntry).Flags = RememberFlags & !(HEAP_ENTRY_LAST_ENTRY as UCHAR);
                (*UnusedEntry).Size = FreeSize as USHORT;

                /* Update previous size of the following entry */
                (*FollowingEntry).PreviousSize = FreeSize as USHORT;

                /* Insert it to the heap and update total free size */
                RtlpInsertFreeBlockHelper(Heap, UnusedEntry, FreeSize, 0u8);
                (*Heap).TotalFreeSize += FreeSize;
            } else {
                /* That following entry is also free, what a fortune! */
                RememberFlags = (*FollowingEntry).Flags;

                /* Remove it */
                RtlpRemoveFreeBlock(Heap, FollowingEntry, 0u8);
                (*Heap).TotalFreeSize -= (*FollowingEntry).Size as SIZE_T;

                /* And make up a new combined block */
                FreeSize += (*FollowingEntry).Size as SIZE_T;
                (*UnusedEntry).Flags = RememberFlags;

                /* Check where to put it */
                if FreeSize <= HEAP_MAX_BLOCK_SIZE as SIZE_T {
                    /* Fine for a dedicated list */
                    (*UnusedEntry).Size = FreeSize as USHORT;

                    if RememberFlags & HEAP_ENTRY_LAST_ENTRY == 0 {
                        (*(UnusedEntry as PHEAP_ENTRY).add(FreeSize as usize)).PreviousSize =
                            FreeSize as USHORT;
                    } else {
                        SegmentOffset = (*UnusedEntry).SegmentOffset;
                        let _ = SegmentOffset;
                    }

                    /* Insert it back and update total size */
                    RtlpInsertFreeBlockHelper(Heap, UnusedEntry, FreeSize, 0u8);
                    (*Heap).TotalFreeSize += FreeSize;
                } else {
                    /* The block is very large, leave all the hassle to the insertion routine */
                    RtlpInsertFreeBlock(Heap, UnusedEntry, FreeSize);
                }
            }
        }
    }

    /* Properly "zero out" (and fill!) the space */
    if Flags & HEAP_ZERO_MEMORY != 0 {
        RtlZeroMemory(
            (InUseEntry.add(1) as PCHAR).add(PrevSize as usize) as PVOID,
            Size - PrevSize,
        );
    } else if (*Heap).Flags & HEAP_FREE_CHECKING_ENABLED != 0 {
        /* Calculate tail part which we need to fill */
        TailPart = PrevSize & (core::mem::size_of::<ULONG>() as SIZE_T - 1);

        /* "Invert" it as usual */
        if TailPart != 0 {
            TailPart = 4 - TailPart;
        }

        if Size > PrevSize + TailPart {
            AddedSize =
                (Size - (PrevSize + TailPart)) & !(core::mem::size_of::<ULONG>() as SIZE_T - 1);
        }

        if AddedSize != 0 {
            rtl_fill_memory_ulong(
                (InUseEntry.add(1) as PCHAR).add((PrevSize + TailPart) as usize) as PVOID,
                AddedSize as _,
                ARENA_INUSE_FILLER,
            );
        }
    }

    /* Fill the new tail */
    if (*Heap).Flags & HEAP_TAIL_CHECKING_ENABLED != 0 {
        core::ptr::write_bytes(
            (InUseEntry.add(1) as PCHAR).add(Size as usize),
            HEAP_TAIL_FILL as u8,
            HEAP_ENTRY_SIZE as usize,
        );
    }

    /* Copy user settable flags */
    (*InUseEntry).Flags &= !(HEAP_ENTRY_SETTABLE_FLAGS as UCHAR);
    (*InUseEntry).Flags |= ((Flags & HEAP_SETTABLE_USER_FLAGS) >> 4) as UCHAR;

    /* Return success */
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpGetExtraStuffPointer(HeapEntry: PHEAP_ENTRY) -> PHEAP_ENTRY_EXTRA {
    if ((*HeapEntry).Flags & HEAP_ENTRY_VIRTUAL_ALLOC) != 0 {
        let VirtualEntry = (HeapEntry as *mut u8)
            .sub(core::mem::offset_of!(_HEAP_VIRTUAL_ALLOC_ENTRY, BusyBlock))
            as PHEAP_VIRTUAL_ALLOC_ENTRY;

        core::ptr::addr_of_mut!((*VirtualEntry).ExtraStuff)
    } else {
        HeapEntry.add((*HeapEntry).Size as usize).sub(1) as PHEAP_ENTRY_EXTRA
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlReAllocateHeap(
    HeapPtr: HANDLE,
    mut Flags: ULONG,
    mut Ptr: PVOID,
    Size: SIZE_T,
) -> PVOID {
    let Heap: PHEAP = HeapPtr as PHEAP;
    let InUseEntry: PHEAP_ENTRY;
    let NewInUseEntry: PHEAP_ENTRY;
    let mut OldExtra: PHEAP_ENTRY_EXTRA;
    let NewExtra: PHEAP_ENTRY_EXTRA;
    let mut AllocationSize: SIZE_T;
    let mut FreeSize: SIZE_T;
    let _DecommitSize: SIZE_T;
    let mut HeapLocked: BOOLEAN = 0u8;
    let NewBaseAddress: PVOID;
    let SplitBlock: PHEAP_FREE_ENTRY;
    let SplitBlock2: PHEAP_FREE_ENTRY;
    let OldSize: SIZE_T;
    let mut Index: SIZE_T;
    let OldIndex: SIZE_T;
    let FreeFlags: UCHAR;
    let Status: NTSTATUS;
    let mut DecommitBase: PVOID;
    let mut RemainderBytes: SIZE_T;
    let ExtraSize: SIZE_T;
    let VirtualAllocBlock: PHEAP_VIRTUAL_ALLOC_ENTRY;
    let mut ExceptionRecord: EXCEPTION_RECORD = core::mem::zeroed();

    /* Return success in case of a null pointer */
    if Ptr.is_null() {
        RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_SUCCESS);
        return core::ptr::null_mut();
    }

    /* Force heap flags */
    Flags |= (*Heap).ForceFlags;

    /* Call special heap */
    if RtlpHeapIsSpecial(Flags) != 0 {
        return RtlDebugReAllocateHeap(Heap as PVOID, Flags, Ptr, Size);
    }

    /* Make sure size is valid */
    if Size >= 0x80000000 {
        RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_NO_MEMORY);
        return core::ptr::null_mut();
    }

    /* Calculate allocation size and index */
    if Size != 0 {
        AllocationSize = Size;
    } else {
        AllocationSize = 1;
    }
    AllocationSize = (AllocationSize + (*Heap).AlignRound as SIZE_T) & (*Heap).AlignMask as SIZE_T;

    /* Add up extra stuff, if it is present anywhere */
    if (((*(Ptr as PHEAP_ENTRY).sub(1)).Flags & HEAP_ENTRY_EXTRA_PRESENT) != 0)
        || (Flags & HEAP_EXTRA_FLAGS_MASK) != 0
        || !(*Heap).PseudoTagEntries.is_null()
    {
        AllocationSize += core::mem::size_of::<HEAP_ENTRY_EXTRA>() as SIZE_T;
    }

    /* Acquire the lock if necessary */
    if (Flags & HEAP_NO_SERIALIZE) == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = 1u8;
        Flags |= HEAP_NO_SERIALIZE;
    }

    /* Get the pointer to the in-use entry */
    InUseEntry = (Ptr as PHEAP_ENTRY).sub(1);

    /* If that entry is not really in-use, we have a problem */
    if ((*InUseEntry).Flags & HEAP_ENTRY_BUSY) == 0 {
        RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_INVALID_PARAMETER);

        if HeapLocked != 0 {
            RtlLeaveHeapLock((*Heap).LockVariable);
        }
        return Ptr;
    }

    if ((*InUseEntry).Flags & HEAP_ENTRY_VIRTUAL_ALLOC) != 0 {
        /* This is a virtually allocated block. Get its size */
        OldSize = RtlpGetSizeOfBigBlock(InUseEntry);

        /* Convert it to an index */
        OldIndex = (OldSize + (*InUseEntry).Size as SIZE_T) >> HEAP_ENTRY_SHIFT;

        /* Calculate new allocation size and round it to the page size */
        AllocationSize +=
            core::mem::offset_of!(_HEAP_VIRTUAL_ALLOC_ENTRY, BusyBlock) as SIZE_T;
        AllocationSize = ROUND_UP(AllocationSize as ULONG_PTR, PAGE_SIZE as ULONG_PTR) as SIZE_T;
    } else {
        /* Usual entry */
        OldIndex = (*InUseEntry).Size as SIZE_T;

        OldSize = (OldIndex << HEAP_ENTRY_SHIFT) - (*InUseEntry).UnusedBytes as SIZE_T;
    }

    /* Calculate new index */
    Index = AllocationSize >> HEAP_ENTRY_SHIFT;

    /* Check for 4 different scenarios */
    if Index <= OldIndex {
        /* Difference must be greater than 1 */
        if Index + 1 == OldIndex {
            Index += 1;
            AllocationSize += core::mem::size_of::<HEAP_ENTRY>() as SIZE_T;
        }

        /* Calculate new size */
        if ((*InUseEntry).Flags & HEAP_ENTRY_VIRTUAL_ALLOC) != 0 {
            (*InUseEntry).Size = (AllocationSize - Size) as USHORT;
        } else if ((*InUseEntry).Flags & HEAP_ENTRY_EXTRA_PRESENT) != 0 {
            OldExtra = InUseEntry.add((*InUseEntry).Size as usize).sub(1) as PHEAP_ENTRY_EXTRA;
            NewExtra = InUseEntry.add(Index as usize).sub(1) as PHEAP_ENTRY_EXTRA;
            *NewExtra = *OldExtra;

            (*InUseEntry).UnusedBytes = (AllocationSize - Size) as UCHAR;
        } else {
            (*InUseEntry).UnusedBytes = (AllocationSize - Size) as UCHAR;
        }

        /* If new size is bigger than the old size */
        if Size > OldSize {
            if (Flags & HEAP_ZERO_MEMORY) != 0 {
                RtlZeroMemory((Ptr as PCHAR).add(OldSize as usize) as PVOID, Size - OldSize);
            } else if ((*Heap).Flags & HEAP_FREE_CHECKING_ENABLED) != 0 {
                RemainderBytes = OldSize & (core::mem::size_of::<ULONG>() as SIZE_T - 1);

                if RemainderBytes != 0 {
                    RemainderBytes = 4 - RemainderBytes;
                }

                if Size > (OldSize + RemainderBytes) {
                    ExtraSize = (Size - (OldSize + RemainderBytes))
                        & !(core::mem::size_of::<ULONG>() as SIZE_T - 1);

                    if ExtraSize != 0 {
                        rtl_fill_memory_ulong(
                            (InUseEntry.add(1) as PCHAR)
                                .add(OldSize as usize + RemainderBytes as usize)
                                as PVOID,
                            ExtraSize as _,
                            ARENA_INUSE_FILLER,
                        );
                    }
                }
            }
        }

        /* Fill tail of the heap entry if required */
        if ((*Heap).Flags & HEAP_TAIL_CHECKING_ENABLED) != 0 {
            core::ptr::write_bytes(
                (InUseEntry.add(1) as PCHAR).add(Size as usize),
                HEAP_TAIL_FILL as u8,
                HEAP_ENTRY_SIZE as usize,
            );
        }

        /* Check if the difference is significant or not */
        if Index != OldIndex {
            FreeFlags = (*InUseEntry).Flags & !(HEAP_ENTRY_BUSY as UCHAR);

            if (FreeFlags & HEAP_ENTRY_VIRTUAL_ALLOC) != 0 {
                VirtualAllocBlock = (InUseEntry as *mut u8).sub(
                    core::mem::offset_of!(_HEAP_VIRTUAL_ALLOC_ENTRY, BusyBlock),
                ) as PHEAP_VIRTUAL_ALLOC_ENTRY;

                DecommitBase = (VirtualAllocBlock as PCHAR).add(AllocationSize as usize) as PVOID;
                let mut DecommitSizeLocal: SIZE_T =
                    (OldIndex << HEAP_ENTRY_SHIFT) - AllocationSize;

                Status = ZwFreeVirtualMemory(
                    -1isize as HANDLE,
                    core::ptr::addr_of_mut!(DecommitBase),
                    core::ptr::addr_of_mut!(DecommitSizeLocal) as *mut _,
                    MEM_RELEASE,
                );

                if !NT_SUCCESS(Status) {
                } else {
                    (*VirtualAllocBlock).CommitSize -= DecommitSizeLocal;
                }
            } else {
                SplitBlock = InUseEntry.add(Index as usize) as PHEAP_FREE_ENTRY;

                (*SplitBlock).Flags = FreeFlags;
                (*SplitBlock).PreviousSize = Index as USHORT;
                (*SplitBlock).SegmentOffset = (*InUseEntry).SegmentOffset;

                FreeSize = (*InUseEntry).Size as SIZE_T - Index;

                (*InUseEntry).Size = Index as USHORT;
                (*InUseEntry).Flags &= !(HEAP_ENTRY_LAST_ENTRY as UCHAR);

                if (FreeFlags & HEAP_ENTRY_LAST_ENTRY) != 0 {
                    (*SplitBlock).Size = FreeSize as USHORT;
                    RtlpInsertFreeBlockHelper(Heap, SplitBlock, FreeSize, 0u8);

                    (*Heap).TotalFreeSize += FreeSize;
                } else {
                    SplitBlock2 =
                        (SplitBlock as PHEAP_ENTRY).add(FreeSize as usize) as PHEAP_FREE_ENTRY;

                    if ((*SplitBlock2).Flags & HEAP_ENTRY_BUSY) != 0 {
                        (*SplitBlock).Size = FreeSize as USHORT;

                        (*((SplitBlock as PHEAP_ENTRY).add(FreeSize as usize) as PHEAP_FREE_ENTRY))
                            .PreviousSize = FreeSize as USHORT;

                        RtlpInsertFreeBlockHelper(Heap, SplitBlock, FreeSize, 0u8);

                        (*Heap).TotalFreeSize += FreeSize;
                    } else {
                        (*SplitBlock).Flags = (*SplitBlock2).Flags;

                        RtlpRemoveFreeBlock(Heap, SplitBlock2, 0u8);
                        (*Heap).TotalFreeSize -= (*SplitBlock2).Size as SIZE_T;

                        FreeSize += (*SplitBlock2).Size as SIZE_T;

                        if FreeSize <= HEAP_MAX_BLOCK_SIZE as SIZE_T {
                            (*SplitBlock).Size = FreeSize as USHORT;

                            if ((*SplitBlock).Flags & HEAP_ENTRY_LAST_ENTRY) == 0 {
                                (*((SplitBlock as PHEAP_ENTRY).add(FreeSize as usize)
                                    as PHEAP_FREE_ENTRY))
                                    .PreviousSize = FreeSize as USHORT;
                            }

                            RtlpInsertFreeBlockHelper(Heap, SplitBlock, FreeSize, 0u8);
                            (*Heap).TotalFreeSize += FreeSize;
                        } else {
                            RtlpInsertFreeBlock(Heap, SplitBlock, FreeSize);
                        }
                    }
                }
            }
        }
    } else {
        /* We're growing the block */
        if ((*InUseEntry).Flags & HEAP_ENTRY_VIRTUAL_ALLOC) != 0
            || RtlpGrowBlockInPlace(Heap, Flags, InUseEntry, Size, Index) == 0
        {
            if (Flags & HEAP_REALLOC_IN_PLACE_ONLY) != 0 {
                Ptr = core::ptr::null_mut();
            } else {
                Flags &= !HEAP_TAG_MASK;

                if ((*InUseEntry).Flags & HEAP_ENTRY_EXTRA_PRESENT) != 0 {
                    Flags &= !HEAP_SETTABLE_USER_FLAGS;

                    Flags |= HEAP_SETTABLE_USER_VALUE
                        | ((((*InUseEntry).Flags & HEAP_ENTRY_SETTABLE_FLAGS) as u32) << 4);

                    OldExtra = RtlpGetExtraStuffPointer(InUseEntry);

                    if (*OldExtra).TagIndex != 0
                        && ((*OldExtra).TagIndex as u32 & HEAP_PSEUDO_TAG_FLAG) == 0
                    {
                        Flags |= ((*OldExtra).TagIndex as u32) << HEAP_TAG_SHIFT;
                    }
                } else if (*InUseEntry).SmallTagIndex != 0 {
                    Flags |= ((*InUseEntry).SmallTagIndex as u32) << HEAP_TAG_SHIFT;
                }

                NewBaseAddress = RtlAllocateHeap(HeapPtr, Flags & !HEAP_ZERO_MEMORY, Size);

                if !NewBaseAddress.is_null() {
                    NewInUseEntry = (NewBaseAddress as PHEAP_ENTRY).sub(1);

                    if ((*NewInUseEntry).Flags & HEAP_ENTRY_EXTRA_PRESENT) != 0 {
                        let NewExtra2 = RtlpGetExtraStuffPointer(NewInUseEntry);

                        if ((*InUseEntry).Flags & HEAP_ENTRY_EXTRA_PRESENT) != 0 {
                            OldExtra = RtlpGetExtraStuffPointer(InUseEntry);
                            (*NewExtra2).Settable = (*OldExtra).Settable;
                        } else {
                            RtlZeroMemory(
                                NewExtra2 as PVOID,
                                core::mem::size_of::<HEAP_ENTRY_EXTRA>() as SIZE_T,
                            );
                        }
                    }

                    if Size < OldSize {
                        core::ptr::copy(Ptr, NewBaseAddress, Size as usize);
                    } else {
                        core::ptr::copy(Ptr, NewBaseAddress, OldSize as usize);
                    }

                    if Size > OldSize && (Flags & HEAP_ZERO_MEMORY) != 0 {
                        RtlZeroMemory(
                            (NewBaseAddress as PCHAR).add(OldSize as usize) as PVOID,
                            Size - OldSize,
                        );
                    }

                    RtlFreeHeap(HeapPtr, Flags, Ptr);
                }

                Ptr = NewBaseAddress;
            }
        }
    }

    /* Did resizing fail? */
    if Ptr.is_null() && (Flags & HEAP_GENERATE_EXCEPTIONS) != 0 {
        ExceptionRecord.ExceptionCode = STATUS_NO_MEMORY;
        ExceptionRecord.ExceptionRecord = core::ptr::null_mut();
        ExceptionRecord.NumberParameters = 1;
        ExceptionRecord.ExceptionFlags = 0;
        ExceptionRecord.ExceptionInformation[0] = AllocationSize as _;

        RtlRaiseException(core::ptr::addr_of_mut!(ExceptionRecord));
    }

    if HeapLocked != 0 {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    Ptr
}

#[no_mangle]
pub unsafe extern "system" fn RtlCompactHeap(_Heap: HANDLE, _Flags: ULONG) -> ULONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlLockHeap(HeapPtr: HANDLE) -> BOOLEAN {
    let Heap = HeapPtr as PHEAP;

    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpPageHeapLock(Heap);
    }

    if (*Heap).Signature != HEAP_SIGNATURE {
        return 0u8;
    }

    if (*Heap).Flags & HEAP_NO_SERIALIZE == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnlockHeap(HeapPtr: HANDLE) -> BOOLEAN {
    let Heap = HeapPtr as PHEAP;

    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpPageHeapUnlock(Heap);
    }

    if (*Heap).Signature != HEAP_SIGNATURE {
        return 0u8;
    }

    if (*Heap).Flags & HEAP_NO_SERIALIZE == 0 {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlSizeHeap(
    HeapPtr: HANDLE,
    mut Flags: ULONG,
    Ptr: PVOID,
) -> SIZE_T {
    let Heap: PHEAP = HeapPtr as PHEAP;
    let HeapEntry: PHEAP_ENTRY;
    let EntrySize: SIZE_T;

    // FIXME This is a hack around missing SEH support!
    if Heap.is_null() {
        RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_INVALID_HANDLE);
        return -1i64 as SIZE_T;
    }

    /* Force flags */
    Flags |= (*Heap).ForceFlags;

    /* Call special heap */
    if RtlpHeapIsSpecial(Flags) != 0 {
        return RtlDebugSizeHeap(Heap as PVOID, Flags, Ptr);
    }

    /* Get the heap entry pointer */
    HeapEntry = (Ptr as PHEAP_ENTRY).offset(-1);

    /* Return -1 if that entry is free */
    if (*HeapEntry).Flags & HEAP_ENTRY_BUSY == 0 {
        RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_INVALID_PARAMETER);
        return -1i64 as SIZE_T;
    }

    /* Get size of this block depending if it's a usual or a big one */
    if (*HeapEntry).Flags & HEAP_ENTRY_VIRTUAL_ALLOC != 0 {
        EntrySize = RtlpGetSizeOfBigBlock(HeapEntry);
    } else {
        /* Calculate it */
        EntrySize = (((*HeapEntry).Size as SIZE_T) << HEAP_ENTRY_SHIFT)
            - (*HeapEntry).UnusedBytes as SIZE_T;
    }

    /* Return calculated size */
    EntrySize
}

#[no_mangle]
pub unsafe extern "system" fn RtlpCheckInUsePattern(HeapEntry: PHEAP_ENTRY) -> BOOLEAN {
    let Size: SIZE_T;
    let Result: SIZE_T;
    let TailPart: PCHAR;

    if (*HeapEntry).Flags & HEAP_ENTRY_VIRTUAL_ALLOC as u8 != 0 {
        Size = RtlpGetSizeOfBigBlock(HeapEntry);
    } else {
        Size = (((*HeapEntry).Size as SIZE_T) << HEAP_ENTRY_SHIFT) - (*HeapEntry).UnusedBytes as SIZE_T;
    }

    TailPart = (HeapEntry.offset(1) as PCHAR).add(Size as usize);

    Result = RtlCompareMemory(
        TailPart as *const core::ffi::c_void,
        FillPattern.as_ptr() as *const core::ffi::c_void,
        HEAP_ENTRY_SIZE as _,
    ) as usize;

    if Result != HEAP_ENTRY_SIZE as SIZE_T {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpValidateHeapHeaders(
    _Heap: PHEAP,
    _Recalculate: BOOLEAN,
) -> BOOLEAN {
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpValidateHeapEntry(
    Heap: PHEAP,
    HeapEntry: PHEAP_ENTRY,
) -> BOOLEAN {
    let BigAllocation: BOOLEAN;
    let mut EntryFound: BOOLEAN = 0u8;
    let mut Segment: PHEAP_SEGMENT;
    let mut SegmentOffset: ULONG;

    'invalid_entry: {
        if HeapEntry.is_null() {
            break 'invalid_entry;
        }
        if (HeapEntry as ULONG_PTR) & (HEAP_ENTRY_SIZE as ULONG_PTR - 1) != 0 {
            break 'invalid_entry;
        }
        if (*HeapEntry).Flags & HEAP_ENTRY_BUSY as u8 == 0 {
            break 'invalid_entry;
        }

        BigAllocation = (*HeapEntry).Flags & HEAP_ENTRY_VIRTUAL_ALLOC as u8;
        Segment = (*Heap).Segments[(*HeapEntry).SegmentOffset as usize];

        if BigAllocation != 0
            && (((HeapEntry as ULONG_PTR) & (PAGE_SIZE as ULONG_PTR - 1))
                != core::mem::offset_of!(HEAP_VIRTUAL_ALLOC_ENTRY, BusyBlock))
        {
            break 'invalid_entry;
        }

        if BigAllocation == 0
            && ((*HeapEntry).SegmentOffset as usize >= HEAP_SEGMENTS
                || Segment.is_null()
                || HeapEntry < (*Segment).FirstEntry
                || HeapEntry >= (*Segment).LastValidEntry)
        {
            break 'invalid_entry;
        }

        if (*HeapEntry).Flags & HEAP_ENTRY_FILL_PATTERN as u8 != 0
            && RtlpCheckInUsePattern(HeapEntry) == 0
        {
            break 'invalid_entry;
        }

        if (*HeapEntry).Flags & HEAP_ENTRY_VIRTUAL_ALLOC as u8 != 0 {
            return 1u8;
        }

        SegmentOffset = 0;
        while (SegmentOffset as usize) < HEAP_SEGMENTS {
            Segment = (*Heap).Segments[SegmentOffset as usize];
            if Segment.is_null() {
                SegmentOffset += 1;
                continue;
            }

            if HeapEntry >= (*Segment).FirstEntry && HeapEntry < (*Segment).LastValidEntry {
                EntryFound = 1u8;
                break;
            }
            SegmentOffset += 1;
        }

        return EntryFound;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpValidateHeapSegment(
    Heap: PHEAP,
    Segment: PHEAP_SEGMENT,
    SegmentOffset: UCHAR,
    FreeEntriesCount: PULONG,
    TotalFreeSize: PSIZE_T,
    TagEntries: PSIZE_T,
    _PseudoTagEntries: PSIZE_T,
) -> BOOLEAN {
    let mut UcrDescriptor: PHEAP_UCR_DESCRIPTOR;
    let mut UcrEntry: PLIST_ENTRY;
    let mut ByteSize: SIZE_T;
    let mut Size: SIZE_T;
    let mut Result: SIZE_T;
    let mut CurrentEntry: PHEAP_ENTRY;
    let mut UnCommittedPages: ULONG;
    let mut UnCommittedRanges: ULONG;
    let mut PreviousSize: ULONG;

    UnCommittedPages = 0;
    UnCommittedRanges = 0;

    if IsListEmpty(core::ptr::addr_of_mut!((*Segment).UCRSegmentList)) {
        UcrEntry = core::ptr::null_mut();
        UcrDescriptor = core::ptr::null_mut();
    } else {
        UcrEntry = (*Segment).UCRSegmentList.Flink;
        UcrDescriptor = (UcrEntry as PCHAR)
            .offset(-(core::mem::offset_of!(_HEAP_UCR_DESCRIPTOR, SegmentEntry) as isize))
            as PHEAP_UCR_DESCRIPTOR;
    }

    if (*Segment).BaseAddress == Heap as PVOID {
        CurrentEntry = core::ptr::addr_of_mut!((*Heap).Entry);
    } else {
        CurrentEntry = core::ptr::addr_of_mut!((*Segment).Entry);
    }

    while CurrentEntry < (*Segment).LastValidEntry {
        if !UcrDescriptor.is_null() && (CurrentEntry as PVOID >= (*UcrDescriptor).Address) {
            return 0u8;
        }

        PreviousSize = 0;

        while CurrentEntry < (*Segment).LastValidEntry {
            if PreviousSize != (*CurrentEntry).PreviousSize as ULONG {
                return 0u8;
            }

            PreviousSize = (*CurrentEntry).Size as ULONG;
            Size = ((*CurrentEntry).Size as SIZE_T) << HEAP_ENTRY_SHIFT;

            if (*CurrentEntry).Flags & HEAP_ENTRY_BUSY as u8 != 0 {
                if !TagEntries.is_null() {
                    // UNIMPLEMENTED
                }

                if (*CurrentEntry).Flags & HEAP_ENTRY_FILL_PATTERN as u8 != 0 {
                    if RtlpCheckInUsePattern(CurrentEntry) == 0 {
                        return 0u8;
                    }
                }
            } else {
                *FreeEntriesCount = *FreeEntriesCount + 1;
                *TotalFreeSize += (*CurrentEntry).Size as SIZE_T;

                if (*Heap).Flags & HEAP_FREE_CHECKING_ENABLED != 0
                    && (*CurrentEntry).Flags & HEAP_ENTRY_FILL_PATTERN as u8 != 0
                {
                    ByteSize = Size - core::mem::size_of::<HEAP_FREE_ENTRY>() as SIZE_T;

                    if (*CurrentEntry).Flags & HEAP_ENTRY_EXTRA_PRESENT as u8 != 0
                        && ByteSize > core::mem::size_of::<HEAP_FREE_ENTRY_EXTRA>() as SIZE_T
                    {
                        ByteSize -= core::mem::size_of::<HEAP_FREE_ENTRY_EXTRA>() as SIZE_T;
                    }

                    Result = RtlCompareMemoryUlong(
                        (CurrentEntry as PHEAP_FREE_ENTRY).offset(1) as PVOID,
                        ByteSize as _,
                        ARENA_FREE_FILLER,
                    ) as usize;

                    if Result != ByteSize {
                        return 0u8;
                    }
                }
            }

            if (*CurrentEntry).SegmentOffset != SegmentOffset {
                return 0u8;
            }

            if (*CurrentEntry).Flags & HEAP_ENTRY_LAST_ENTRY as u8 != 0 {
                CurrentEntry = (CurrentEntry as PCHAR).add(Size as usize) as PHEAP_ENTRY;

                if UcrDescriptor.is_null() {
                    if CurrentEntry != (*Segment).LastValidEntry {
                        return 0u8;
                    }
                } else if CurrentEntry as PVOID != (*UcrDescriptor).Address {
                    return 0u8;
                } else {
                    UnCommittedPages += ((*UcrDescriptor).Size / PAGE_SIZE as SIZE_T) as ULONG;
                    UnCommittedRanges += 1;

                    CurrentEntry = ((*UcrDescriptor).Address as PCHAR)
                        .add((*UcrDescriptor).Size as usize)
                        as PHEAP_ENTRY;

                    UcrEntry = (*UcrEntry).Flink;
                    if UcrEntry == core::ptr::addr_of_mut!((*Segment).UCRSegmentList) {
                        UcrEntry = core::ptr::null_mut();
                        UcrDescriptor = core::ptr::null_mut();
                    } else {
                        UcrDescriptor = (UcrEntry as PCHAR)
                            .offset(-(core::mem::offset_of!(_HEAP_UCR_DESCRIPTOR, SegmentEntry) as isize))
                            as PHEAP_UCR_DESCRIPTOR;
                    }
                }

                break;
            }

            CurrentEntry = (CurrentEntry as PCHAR).add(Size as usize) as PHEAP_ENTRY;
        }
    }

    if (*Segment).NumberOfUnCommittedPages != UnCommittedPages {
        return 0u8;
    }

    if (*Segment).NumberOfUnCommittedRanges != UnCommittedRanges {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlpValidateHeap(
    Heap: PHEAP,
    ForceValidation: BOOLEAN,
) -> BOOLEAN {
    let mut SegmentOffset: UCHAR;
    let mut TotalFreeSize: SIZE_T;
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut FreeBlocksCount: ULONG;
    let mut FreeListEntriesCount: ULONG;
    let mut HintIndex: ULONG;

    /* Check headers */
    if RtlpValidateHeapHeaders(Heap, 0u8) == 0 {
        return 0u8;
    }

    /* Skip validation if it's not needed */
    if ForceValidation == 0 && ((*Heap).Flags & HEAP_VALIDATE_ALL_ENABLED) == 0 {
        return 1u8;
    }

    /* Check free list */
    FreeListEntriesCount = 0;
    ListHead = core::ptr::addr_of_mut!((*Heap).FreeLists);
    NextEntry = (*ListHead).Flink;

    while NextEntry != ListHead {
        let FreeEntry: PHEAP_FREE_ENTRY = (NextEntry as PCHAR)
            .offset(-(core::mem::offset_of!(_HEAP_FREE_ENTRY, FreeList) as isize))
            as PHEAP_FREE_ENTRY;

        NextEntry = (*NextEntry).Flink;

        if NextEntry != ListHead {
            let NextFreeEntry: PHEAP_FREE_ENTRY = (NextEntry as PCHAR)
                .offset(-(core::mem::offset_of!(_HEAP_FREE_ENTRY, FreeList) as isize))
                as PHEAP_FREE_ENTRY;
            /* Free entries must be sorted */
            let _ = NextFreeEntry;
        }

        /* Check that the hint is there */
        if ((*FreeEntry).Size as usize) > (*Heap).DeCommitFreeBlockThreshold {
            if (*free_hint_slot(Heap, 0)).is_null() {
            }
            if RtlTestBit(core::ptr::addr_of_mut!((*Heap).FreeHintBitmap), 0) == 0 {
            }
        } else {
            if (*free_hint_slot(Heap, ((*FreeEntry).Size - 1) as usize)).is_null() {
            }
            if RtlTestBit(
                core::ptr::addr_of_mut!((*Heap).FreeHintBitmap),
                ((*FreeEntry).Size - 1) as ULONG,
            ) == 0
            {
            }
        }

        /* If there is an in-use entry in a free list - that's quite a big problem */
        if (*FreeEntry).Flags & HEAP_ENTRY_BUSY as u8 != 0 {
            return 0u8;
        }

        /* Add up to the total amount of free entries */
        FreeListEntriesCount += 1;
    }

    /* Check free list hints */
    HintIndex = 0;
    while (HintIndex as usize) < (*Heap).DeCommitFreeBlockThreshold {
        if !(*free_hint_slot(Heap, HintIndex as usize)).is_null() {
            let FreeEntry: PHEAP_FREE_ENTRY = ((*free_hint_slot(Heap, HintIndex as usize)) as PCHAR)
                .offset(-(core::mem::offset_of!(_HEAP_FREE_ENTRY, FreeList) as isize))
                as PHEAP_FREE_ENTRY;

            if RtlTestBit(core::ptr::addr_of_mut!((*Heap).FreeHintBitmap), HintIndex) == 0 {
            }

            if HintIndex == 0 {
                if (*FreeEntry).Size as usize <= (*Heap).DeCommitFreeBlockThreshold {
                }
            } else {
                if HintIndex != ((*FreeEntry).Size - 1) as ULONG {
                }

                if (*FreeEntry).FreeList.Blink != core::ptr::addr_of_mut!((*Heap).FreeLists) {
                    /* The entry right before the hint must be smaller. */
                    let PreviousFreeEntry: PHEAP_FREE_ENTRY = ((*FreeEntry).FreeList.Blink as PCHAR)
                        .offset(-(core::mem::offset_of!(_HEAP_FREE_ENTRY, FreeList) as isize))
                        as PHEAP_FREE_ENTRY;
                    let _ = PreviousFreeEntry;
                }
            }
        } else if RtlTestBit(core::ptr::addr_of_mut!((*Heap).FreeHintBitmap), HintIndex) != 0 {
        }

        HintIndex += 1;
    }

    /* Check big allocations */
    ListHead = core::ptr::addr_of_mut!((*Heap).VirtualAllocdBlocks);
    NextEntry = (*ListHead).Flink;

    while ListHead != NextEntry {
        let VirtualAllocBlock: PHEAP_VIRTUAL_ALLOC_ENTRY = (NextEntry as PCHAR)
            .offset(-(core::mem::offset_of!(_HEAP_VIRTUAL_ALLOC_ENTRY, Entry) as isize))
            as PHEAP_VIRTUAL_ALLOC_ENTRY;

        /* We can only check the fill pattern */
        if (*VirtualAllocBlock).BusyBlock.Flags & HEAP_ENTRY_FILL_PATTERN as u8 != 0 {
            if RtlpCheckInUsePattern(core::ptr::addr_of_mut!((*VirtualAllocBlock).BusyBlock)) == 0 {
                return 0u8;
            }
        }

        NextEntry = (*NextEntry).Flink;
    }

    /* Check all segments */
    FreeBlocksCount = 0;
    TotalFreeSize = 0;

    SegmentOffset = 0;
    while (SegmentOffset as usize) < HEAP_SEGMENTS {
        let Segment: PHEAP_SEGMENT = (*Heap).Segments[SegmentOffset as usize];

        /* Go to the next one if there is no segment */
        if Segment.is_null() {
            SegmentOffset += 1;
            continue;
        }

        if RtlpValidateHeapSegment(
            Heap,
            Segment,
            SegmentOffset,
            core::ptr::addr_of_mut!(FreeBlocksCount),
            core::ptr::addr_of_mut!(TotalFreeSize),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            return 0u8;
        }

        SegmentOffset += 1;
    }

    if FreeListEntriesCount != FreeBlocksCount {
        return 0u8;
    }

    if (*Heap).TotalFreeSize != TotalFreeSize {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlValidateHeap(
    HeapPtr: HANDLE,
    mut Flags: ULONG,
    Block: PVOID,
) -> BOOLEAN {
    let Heap: PHEAP = HeapPtr as PHEAP;
    let mut HeapLocked: BOOLEAN = 0u8;
    let HeapValid: BOOLEAN;

    /* Check for page heap */
    if (*Heap).ForceFlags & HEAP_FLAG_PAGE_ALLOCS != 0 {
        return RtlpDebugPageHeapValidate(HeapPtr, Flags, Block);
    }

    /* Check signature */
    if (*Heap).Signature != HEAP_SIGNATURE {
        return 0u8;
    }

    /* Force flags */
    Flags |= (*Heap).ForceFlags;

    /* Acquire the lock if necessary */
    if Flags & HEAP_NO_SERIALIZE == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = 1u8;
    }

    /* Either validate whole heap or just one entry */
    if Block.is_null() {
        HeapValid = RtlpValidateHeap(Heap, 1u8);
    } else {
        HeapValid = RtlpValidateHeapEntry(Heap, (Block as PHEAP_ENTRY).offset(-1));
    }

    /* Unlock if it's lockable */
    if HeapLocked != 0 {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    HeapValid
}

#[no_mangle]
pub unsafe extern "system" fn RtlEnumProcessHeaps(
    _HeapEnumerationRoutine: PHEAP_ENUMERATION_ROUTINE,
    _lParam: PVOID,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlValidateProcessHeaps() -> BOOLEAN {
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlZeroHeap(
    _HeapHandle: PVOID,
    _Flags: ULONG,
) -> BOOLEAN {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetUserValueHeap(
    HeapHandle: PVOID,
    mut Flags: ULONG,
    BaseAddress: PVOID,
    UserValue: PVOID,
) -> BOOLEAN {
    let Heap = HeapHandle as PHEAP;
    let HeapEntry: PHEAP_ENTRY;
    let Extra: PHEAP_ENTRY_EXTRA;
    let mut HeapLocked: BOOLEAN = 0u8;
    let mut ValueSet: BOOLEAN = 0u8;

    /* Force flags */
    Flags |= (*Heap).ForceFlags;

    /* Call special heap */
    if RtlpHeapIsSpecial(Flags) != 0 {
        return RtlDebugSetUserValueHeap(Heap as PVOID, Flags, BaseAddress, UserValue);
    }

    /* Lock if it's lockable */
    if ((*Heap).Flags & HEAP_NO_SERIALIZE) == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = 1u8;
    }

    /* Get a pointer to the entry */
    HeapEntry = (BaseAddress as PHEAP_ENTRY).sub(1);

    /* If it's a free entry - return error */
    if ((*HeapEntry).Flags & HEAP_ENTRY_BUSY) == 0 {
        RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_INVALID_PARAMETER);

        /* Release the heap lock if it was acquired */
        if HeapLocked != 0 {
            RtlLeaveHeapLock((*Heap).LockVariable);
        }

        return 0u8;
    }

    /* Check if this entry has an extra stuff associated with it */
    if ((*HeapEntry).Flags & HEAP_ENTRY_EXTRA_PRESENT) != 0 {
        /* Use extra to store the value */
        Extra = RtlpGetExtraStuffPointer(HeapEntry);
        (*Extra).Settable = UserValue as ULONG_PTR;

        /* Indicate that value was set */
        ValueSet = 1u8;
    }

    /* Release the heap lock if it was acquired */
    if HeapLocked != 0 {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    ValueSet
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetUserFlagsHeap(
    HeapHandle: PVOID,
    mut Flags: ULONG,
    BaseAddress: PVOID,
    UserFlagsReset: ULONG,
    UserFlagsSet: ULONG,
) -> BOOLEAN {
    let Heap = HeapHandle as PHEAP;
    let HeapEntry: PHEAP_ENTRY;
    let mut HeapLocked: BOOLEAN = 0u8;

    /* Force flags */
    Flags |= (*Heap).ForceFlags;

    /* Call special heap */
    if RtlpHeapIsSpecial(Flags) != 0 {
        return RtlDebugSetUserFlagsHeap(Heap as PVOID, Flags, BaseAddress, UserFlagsReset, UserFlagsSet);
    }

    /* Lock if it's lockable */
    if (Flags & HEAP_NO_SERIALIZE) == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = 1u8;
    }

    /* Get a pointer to the entry */
    HeapEntry = (BaseAddress as PHEAP_ENTRY).offset(-1);

    /* If it's a free entry - return error */
    if ((*HeapEntry).Flags & HEAP_ENTRY_BUSY as UCHAR) == 0 {
        RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_INVALID_PARAMETER);

        if HeapLocked != 0 {
            RtlLeaveHeapLock((*Heap).LockVariable);
        }

        return 0u8;
    }

    /* Set / reset flags */
    (*HeapEntry).Flags &= !((UserFlagsReset >> 4) as UCHAR);
    (*HeapEntry).Flags |= (UserFlagsSet >> 4) as UCHAR;

    if HeapLocked != 0 {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetUserInfoHeap(
    HeapHandle: PVOID,
    mut Flags: ULONG,
    BaseAddress: PVOID,
    UserValue: *mut PVOID,
    UserFlags: PULONG,
) -> BOOLEAN {
    let Heap = HeapHandle as PHEAP;
    let HeapEntry: PHEAP_ENTRY;
    let Extra: PHEAP_ENTRY_EXTRA;
    let mut HeapLocked: BOOLEAN = 0u8;

    Flags |= (*Heap).ForceFlags;

    if RtlpHeapIsSpecial(Flags) != 0 {
        return RtlDebugGetUserInfoHeap(Heap as PVOID, Flags, BaseAddress, UserValue, UserFlags);
    }

    if (Flags & HEAP_NO_SERIALIZE) == 0 {
        RtlEnterHeapLock((*Heap).LockVariable, 1u8);
        HeapLocked = 1u8;
    }

    HeapEntry = (BaseAddress as PHEAP_ENTRY).sub(1);

    if ((*HeapEntry).Flags & HEAP_ENTRY_BUSY) == 0 {
        RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_INVALID_PARAMETER);

        if HeapLocked != 0 {
            RtlLeaveHeapLock((*Heap).LockVariable);
        }

        return 0u8;
    }

    if ((*HeapEntry).Flags & HEAP_ENTRY_EXTRA_PRESENT) != 0 {
        Extra = RtlpGetExtraStuffPointer(HeapEntry);

        if !UserValue.is_null() {
            *UserValue = (*Extra).Settable as PVOID;
        }
    }

    if !UserFlags.is_null() {
        *UserFlags = (((*HeapEntry).Flags & HEAP_ENTRY_SETTABLE_FLAGS) as u32) << 4;
    }

    if HeapLocked != 0 {
        RtlLeaveHeapLock((*Heap).LockVariable);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn RtlUsageHeap(
    _Heap: HANDLE,
    _Flags: ULONG,
    _Usage: PRTL_HEAP_USAGE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryTagHeap(
    _HeapHandle: PVOID,
    _Flags: ULONG,
    _TagIndex: USHORT,
    _ResetCounters: BOOLEAN,
    _HeapTagInfo: PRTL_HEAP_TAG_INFO,
) -> PWSTR {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlExtendHeap(
    _Heap: HANDLE,
    _Flags: ULONG,
    _P: PVOID,
    _Size: SIZE_T,
) -> ULONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlCreateTagHeap(
    _HeapHandle: HANDLE,
    _Flags: ULONG,
    _TagName: PWSTR,
    _TagSubName: PWSTR,
) -> ULONG {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlWalkHeap(_HeapHandle: HANDLE, _HeapEntry: PVOID) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlProtectHeap(_HeapHandle: PVOID, _ReadOnly: BOOLEAN) -> PVOID {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn RtlSetHeapInformation(
    _HeapHandle: HANDLE,
    HeapInformationClass: HEAP_INFORMATION_CLASS,
    HeapInformation: PVOID,
    HeapInformationLength: SIZE_T,
) -> NTSTATUS {
    if HeapInformationClass == HeapCompatibilityInformation {
        if HeapInformationLength < core::mem::size_of::<ULONG>() as SIZE_T {
            return STATUS_BUFFER_TOO_SMALL;
        }

        if *(HeapInformation as PULONG) != 2 {
            return STATUS_UNSUCCESSFUL;
        }

        return STATUS_SUCCESS;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryHeapInformation(
    HeapHandle: HANDLE,
    HeapInformationClass: HEAP_INFORMATION_CLASS,
    HeapInformation: PVOID,
    HeapInformationLength: SIZE_T,
    ReturnLength: PSIZE_T,
) -> NTSTATUS {
    let Heap: PHEAP = HeapHandle as PHEAP;

    if HeapInformationClass == HeapCompatibilityInformation {
        if !ReturnLength.is_null() {
            *ReturnLength = core::mem::size_of::<ULONG>() as SIZE_T;
        }

        if HeapInformationLength < core::mem::size_of::<ULONG>() as SIZE_T {
            return STATUS_BUFFER_TOO_SMALL;
        }

        *(HeapInformation as PULONG) = (*Heap).FrontEndHeapType as ULONG;

        return STATUS_SUCCESS;
    }

    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn RtlMultipleAllocateHeap(
    HeapHandle: PVOID,
    Flags: ULONG,
    Size: SIZE_T,
    Count: ULONG,
    Array: *mut PVOID,
) -> ULONG {
    let mut Index: ULONG = 0;
    let mut ExceptionRecord: EXCEPTION_RECORD = core::mem::zeroed();

    while Index < Count {
        *Array.add(Index as usize) = RtlAllocateHeap(HeapHandle, Flags, Size);
        if (*Array.add(Index as usize)).is_null() {
            RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_NO_MEMORY);

            if Flags & HEAP_GENERATE_EXCEPTIONS != 0 {
                ExceptionRecord.ExceptionCode = STATUS_NO_MEMORY;
                ExceptionRecord.ExceptionRecord = core::ptr::null_mut();
                ExceptionRecord.NumberParameters = 0;
                ExceptionRecord.ExceptionFlags = 0;

                RtlRaiseException(&mut ExceptionRecord);
            }
            break;
        }
        Index += 1;
    }

    Index
}

#[no_mangle]
pub unsafe extern "system" fn RtlMultipleFreeHeap(
    HeapHandle: PVOID,
    Flags: ULONG,
    Count: ULONG,
    Array: *mut PVOID,
) -> ULONG {
    let mut Index: ULONG = 0;

    while Index < Count {
        if (*Array.offset(Index as isize)).is_null() {
            Index += 1;
            continue;
        }

        if RtlFreeHeap(HeapHandle, Flags, *Array.offset(Index as isize)) == 0u8 {
            /* ERROR_INVALID_PARAMETER */
            RtlSetLastWin32ErrorAndNtStatusFromNtStatus(STATUS_INVALID_PARAMETER);
            break;
        }

        Index += 1;
    }

    Index
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryProcessHeapInformation(
    _DebugBuffer: *mut _DEBUG_BUFFER,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
