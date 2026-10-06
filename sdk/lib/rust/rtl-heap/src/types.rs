//! Byte-exact Rust mirror of `sdk/lib/rtl/heap.h` — the NT process-heap
//! type family. heappage.c / heapdbg.c stay C and share these layouts
//! across the cohab boundary, so every field offset and struct size must
//! match the C exactly. The `C_ASSERT`s from heap.h are reproduced as
//! `const _: () = assert!(...)` at the bottom.
//!
//! The `_HEAP_COMMON_ENTRY` union is flattened: the field interpretations
//! the allocator actually touches (`Size/Flags/SmallTagIndex` in bytes
//! 0..4 and `PreviousSize/SegmentOffset/UnusedBytes` in the trailing
//! bytes) do not overlap, so a flat `#[repr(C)]` struct reproduces them
//! as direct fields while keeping the exact size/alignment. Three arch
//! variants: i386 = 8 bytes (align 4), amd64 = 16 bytes with the leading
//! `PreviousBlockPrivateData`, arm64 = 16 bytes (align 8) without it
//! (heap.h gates the prefix on `_M_AMD64`, not `_WIN64`).

use ntkrnl_sys::base::{
    LIST_ENTRY, PLIST_ENTRY, PVOID, SIZE_T, UCHAR, ULONG, ULONG_PTR, USHORT,
    WCHAR,
};
use ntkrnl_sys::ddk::{PHEAP_LOCK, PRTL_HEAP_COMMIT_ROUTINE};
// Use the hand-written mm::RTL_BITMAP (what ntkrnl_sys::rtl's bitmap API
// takes) so FreeHintBitmap is type-compatible with RtlInitializeBitMap /
// RtlSetBit / RtlClearAllBits etc. (layout-identical to ddk::_RTL_BITMAP).
use ntkrnl_sys::mm::RTL_BITMAP;

// ── core constants (heap.h) ─────────────────────────────────────────
pub const HEAP_SEGMENTS: usize = 64;

#[cfg(target_pointer_width = "64")]
pub const HEAP_ENTRY_SHIFT: ULONG = 4;
#[cfg(target_pointer_width = "32")]
pub const HEAP_ENTRY_SHIFT: ULONG = 3;

pub const HEAP_ENTRY_SIZE: ULONG = core::mem::size_of::<HEAP_ENTRY>() as ULONG;
// HEAP_MAX_BLOCK_SIZE = (0x80000 - PAGE_SIZE) >> HEAP_ENTRY_SHIFT, PAGE_SIZE = 0x1000.
pub const HEAP_MAX_BLOCK_SIZE: ULONG = (0x80000 - 0x1000) >> HEAP_ENTRY_SHIFT;

pub const ARENA_INUSE_FILLER: ULONG = 0xBAADF00D;
pub const ARENA_FREE_FILLER: ULONG = 0xFEEEFEEE;
pub const HEAP_TAIL_FILL: UCHAR = 0xab;

pub const HEAP_GLOBAL_TAG: ULONG = 0x0800;
pub const HEAP_PSEUDO_TAG_FLAG: ULONG = 0x8000;

// Heap entry flags — applied to HEAP_ENTRY.Flags (UCHAR), so typed u8.
pub const HEAP_ENTRY_BUSY: UCHAR = 0x01;
pub const HEAP_ENTRY_EXTRA_PRESENT: UCHAR = 0x02;
pub const HEAP_ENTRY_FILL_PATTERN: UCHAR = 0x04;
pub const HEAP_ENTRY_VIRTUAL_ALLOC: UCHAR = 0x08;
pub const HEAP_ENTRY_LAST_ENTRY: UCHAR = 0x10;
pub const HEAP_ENTRY_SETTABLE_FLAG1: UCHAR = 0x20;
pub const HEAP_ENTRY_SETTABLE_FLAG2: UCHAR = 0x40;
pub const HEAP_ENTRY_SETTABLE_FLAG3: UCHAR = 0x80;
pub const HEAP_ENTRY_SETTABLE_FLAGS: UCHAR =
    HEAP_ENTRY_SETTABLE_FLAG1 | HEAP_ENTRY_SETTABLE_FLAG2 | HEAP_ENTRY_SETTABLE_FLAG3;

// Signatures
pub const HEAP_SIGNATURE: ULONG = 0xeefeeff;
pub const HEAP_SEGMENT_SIGNATURE: ULONG = 0xffeeffee;

// Segment flags
pub const HEAP_USER_ALLOCATED: ULONG = 0x1;

// ── HEAP_ENTRY (flattened _HEAP_COMMON_ENTRY) ───────────────────────

#[cfg(target_arch = "x86")]
#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct HEAP_ENTRY {
    pub Size: USHORT,         // @0
    pub Flags: UCHAR,         // @2
    pub SmallTagIndex: UCHAR, // @3
    pub PreviousSize: USHORT, // @4
    pub SegmentOffset: UCHAR, // @6 (union with LFHFlags)
    pub UnusedBytes: UCHAR,   // @7
}

// Every 64-bit target, as heap.h's `#ifdef _WIN64` has it: the private-data
// pointer FIRST. aarch64 used to have its own order here (Size at +0,
// PreviousSize at +8) that the C never had. That was invisible while each
// process ran one architecture's heap code, and it is not now: an x86-64 guest
// process on arm64 holds the guest's amd64 ntdll and the native arm64 one in a
// single PEB, both reaching the process heap through PEB->ProcessHeap, so the
// two builds must agree on what a heap block IS. They did not -- the guest's
// RtlpRemoveFreeBlock read a free block whose Size the native side had written
// at +0, found 0 at +8, and indexed its free list with 0xFFFFFFFF.
#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_ENTRY {
    pub PreviousBlockPrivateData: PVOID, // @0 (_M_AMD64 only)
    pub Size: USHORT,                    // @8
    pub Flags: UCHAR,                    // @10
    pub SmallTagIndex: UCHAR,            // @11
    pub PreviousSize: USHORT,            // @12
    pub SegmentOffset: UCHAR,            // @14
    pub UnusedBytes: UCHAR,              // @15
}

pub type PHEAP_ENTRY = *mut HEAP_ENTRY;

// ── HEAP_FREE_ENTRY = common entry + FreeList (flattened) ────────────
// Same common-entry prefix as HEAP_ENTRY, then a LIST_ENTRY. Casts
// between PHEAP_ENTRY and PHEAP_FREE_ENTRY rely on the shared prefix.

#[cfg(target_arch = "x86")]
#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct HEAP_FREE_ENTRY {
    pub Size: USHORT,
    pub Flags: UCHAR,
    pub SmallTagIndex: UCHAR,
    pub PreviousSize: USHORT,
    pub SegmentOffset: UCHAR,
    pub UnusedBytes: UCHAR,
    pub FreeList: LIST_ENTRY,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_FREE_ENTRY {
    pub PreviousBlockPrivateData: PVOID,
    pub Size: USHORT,
    pub Flags: UCHAR,
    pub SmallTagIndex: UCHAR,
    pub PreviousSize: USHORT,
    pub SegmentOffset: UCHAR,
    pub UnusedBytes: UCHAR,
    pub FreeList: LIST_ENTRY,
}

pub type PHEAP_FREE_ENTRY = *mut HEAP_FREE_ENTRY;

// ── tag / counter / tuning structs ──────────────────────────────────
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_TAG_ENTRY {
    pub Allocs: ULONG,
    pub Frees: ULONG,
    pub Size: ULONG,
    pub TagIndex: USHORT,
    pub CreatorBackTraceIndex: USHORT,
    pub TagName: [WCHAR; 24],
}
pub type PHEAP_TAG_ENTRY = *mut HEAP_TAG_ENTRY;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_PSEUDO_TAG_ENTRY {
    pub Allocs: ULONG,
    pub Frees: ULONG,
    pub Size: SIZE_T,
}
pub type PHEAP_PSEUDO_TAG_ENTRY = *mut HEAP_PSEUDO_TAG_ENTRY;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_COUNTERS {
    pub TotalMemoryReserved: SIZE_T,
    pub TotalMemoryCommitted: SIZE_T,
    pub TotalMemoryLargeUCR: SIZE_T,
    pub TotalSizeInVirtualBlocks: SIZE_T,
    pub TotalSegments: ULONG,
    pub TotalUCRs: ULONG,
    pub CommittOps: ULONG,
    pub DeCommitOps: ULONG,
    pub LockAcquires: ULONG,
    pub LockCollisions: ULONG,
    pub CommitRate: ULONG,
    pub DecommittRate: ULONG,
    pub CommitFailures: ULONG,
    pub InBlockCommitFailures: ULONG,
    pub CompactHeapCalls: ULONG,
    pub CompactedUCRs: ULONG,
    pub InBlockDeccommits: ULONG,
    pub InBlockDeccomitSize: SIZE_T,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_TUNING_PARAMETERS {
    pub CommittThresholdShift: ULONG,
    pub MaxPreCommittThreshold: SIZE_T,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_LIST_LOOKUP {
    pub ExtendedLookup: *mut HEAP_LIST_LOOKUP,
    pub ArraySize: ULONG,
    pub ExtraItem: ULONG,
    pub ItemCount: ULONG,
    pub OutOfRangeItems: ULONG,
    pub BaseIndex: ULONG,
    pub ListHead: PLIST_ENTRY,
    pub ListsInUseUlong: *mut ULONG,
    pub ListHints: *mut PLIST_ENTRY,
}
pub type PHEAP_LIST_LOOKUP = *mut HEAP_LIST_LOOKUP;

// ── HEAP (HEAP_SEGMENT_MEMBERS macro inlined) ───────────────────────
#[repr(C)]
pub struct HEAP {
    // -- HEAP_SEGMENT_MEMBERS --
    pub Entry: HEAP_ENTRY,
    pub SegmentSignature: ULONG,
    pub SegmentFlags: ULONG,
    pub SegmentListEntry: LIST_ENTRY,
    pub Heap: *mut HEAP,
    pub BaseAddress: PVOID,
    pub NumberOfPages: ULONG,
    pub FirstEntry: PHEAP_ENTRY,
    pub LastValidEntry: PHEAP_ENTRY,
    pub NumberOfUnCommittedPages: ULONG,
    pub NumberOfUnCommittedRanges: ULONG,
    pub SegmentAllocatorBackTraceIndex: USHORT,
    pub Reserved: USHORT,
    pub UCRSegmentList: LIST_ENTRY,
    // -- HEAP-specific --
    pub Flags: ULONG,
    pub ForceFlags: ULONG,
    pub CompatibilityFlags: ULONG,
    pub EncodeFlagMask: ULONG,
    pub Encoding: HEAP_ENTRY,
    pub PointerKey: ULONG_PTR,
    pub Interceptor: ULONG,
    pub VirtualMemoryThreshold: ULONG,
    pub Signature: ULONG,
    pub SegmentReserve: SIZE_T,
    pub SegmentCommit: SIZE_T,
    pub DeCommitFreeBlockThreshold: SIZE_T,
    pub DeCommitTotalFreeThreshold: SIZE_T,
    pub TotalFreeSize: SIZE_T,
    pub MaximumAllocationSize: SIZE_T,
    pub ProcessHeapsListIndex: USHORT,
    pub HeaderValidateLength: USHORT,
    pub HeaderValidateCopy: PVOID,
    pub NextAvailableTagIndex: USHORT,
    pub MaximumTagIndex: USHORT,
    pub TagEntries: PHEAP_TAG_ENTRY,
    pub UCRList: LIST_ENTRY,
    pub UCRSegments: LIST_ENTRY,
    pub AlignRound: ULONG_PTR,
    pub AlignMask: ULONG_PTR,
    pub VirtualAllocdBlocks: LIST_ENTRY,
    pub SegmentList: LIST_ENTRY,
    pub Segments: [*mut HEAP_SEGMENT; HEAP_SEGMENTS],
    pub AllocatorBackTraceIndex: USHORT,
    pub NonDedicatedListLength: ULONG,
    pub BlocksIndex: PVOID,
    pub UCRIndex: PVOID,
    pub PseudoTagEntries: PHEAP_PSEUDO_TAG_ENTRY,
    pub FreeLists: LIST_ENTRY,
    pub LockVariable: PHEAP_LOCK,
    pub CommitRoutine: PRTL_HEAP_COMMIT_ROUTINE,
    pub FrontEndHeap: PVOID,
    pub FrontHeapLockCount: USHORT,
    pub FrontEndHeapType: UCHAR,
    pub Counters: HEAP_COUNTERS,
    pub TuningParameters: HEAP_TUNING_PARAMETERS,
    pub FreeHintBitmap: RTL_BITMAP,
    pub FreeHints: [PLIST_ENTRY; 1], // ANYSIZE_ARRAY
}
pub type PHEAP = *mut HEAP;

#[repr(C)]
pub struct HEAP_SEGMENT {
    pub Entry: HEAP_ENTRY,
    pub SegmentSignature: ULONG,
    pub SegmentFlags: ULONG,
    pub SegmentListEntry: LIST_ENTRY,
    pub Heap: *mut HEAP,
    pub BaseAddress: PVOID,
    pub NumberOfPages: ULONG,
    pub FirstEntry: PHEAP_ENTRY,
    pub LastValidEntry: PHEAP_ENTRY,
    pub NumberOfUnCommittedPages: ULONG,
    pub NumberOfUnCommittedRanges: ULONG,
    pub SegmentAllocatorBackTraceIndex: USHORT,
    pub Reserved: USHORT,
    pub UCRSegmentList: LIST_ENTRY,
}
pub type PHEAP_SEGMENT = *mut HEAP_SEGMENT;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_UCR_DESCRIPTOR {
    pub ListEntry: LIST_ENTRY,
    pub SegmentEntry: LIST_ENTRY,
    pub Address: PVOID,
    pub Size: SIZE_T,
}
pub type PHEAP_UCR_DESCRIPTOR = *mut HEAP_UCR_DESCRIPTOR;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_UCR_SEGMENT {
    pub ListEntry: LIST_ENTRY,
    pub ReservedSize: SIZE_T,
    pub CommittedSize: SIZE_T,
}
pub type PHEAP_UCR_SEGMENT = *mut HEAP_UCR_SEGMENT;

// _HEAP_ENTRY_EXTRA: union { struct { USHORT; USHORT; ULONG_PTR Settable }; UINT64 ZeroInit }.
// Flattened to the named struct interpretation (AllocatorBackTraceIndex /
// TagIndex / Settable); ZeroInit overlaps the whole thing — exposed as a
// method. On amd64/arm64 ULONG_PTR is 8 → struct is 16 (> the 8-byte
// ZeroInit), matching the C union which is sized by its largest member.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_ENTRY_EXTRA {
    pub AllocatorBackTraceIndex: USHORT,
    pub TagIndex: USHORT,
    pub Settable: ULONG_PTR,
}
pub type PHEAP_ENTRY_EXTRA = *mut HEAP_ENTRY_EXTRA;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_VIRTUAL_ALLOC_ENTRY {
    pub Entry: LIST_ENTRY,
    pub ExtraStuff: HEAP_ENTRY_EXTRA,
    pub CommitSize: SIZE_T,
    pub ReserveSize: SIZE_T,
    pub BusyBlock: HEAP_ENTRY,
}
pub type PHEAP_VIRTUAL_ALLOC_ENTRY = *mut HEAP_VIRTUAL_ALLOC_ENTRY;

// ── public heap flags (winnt.h / xdk mmtypes.h / ndk rtltypes.h) ────
// Exact C values — wrong flags corrupt the heap.
pub const HEAP_NO_SERIALIZE: ULONG = 0x00000001;
pub const HEAP_GROWABLE: ULONG = 0x00000002;
pub const HEAP_GENERATE_EXCEPTIONS: ULONG = 0x00000004;
pub const HEAP_ZERO_MEMORY: ULONG = 0x00000008;
pub const HEAP_REALLOC_IN_PLACE_ONLY: ULONG = 0x00000010;
pub const HEAP_TAIL_CHECKING_ENABLED: ULONG = 0x00000020;
pub const HEAP_FREE_CHECKING_ENABLED: ULONG = 0x00000040;
pub const HEAP_DISABLE_COALESCE_ON_FREE: ULONG = 0x00000080;
pub const HEAP_SETTABLE_USER_VALUE: ULONG = 0x00000100;
pub const HEAP_SETTABLE_USER_FLAGS: ULONG = 0x00000E00;
pub const HEAP_CREATE_ALIGN_16: ULONG = 0x00010000;
pub const HEAP_CREATE_ENABLE_TRACING: ULONG = 0x00020000;
pub const HEAP_CREATE_ENABLE_EXECUTE: ULONG = 0x00040000;
pub const HEAP_FLAG_PAGE_ALLOCS: ULONG = 0x01000000;
pub const HEAP_PROTECTION_ENABLED: ULONG = 0x02000000;
pub const HEAP_BREAK_WHEN_OUT_OF_VM: ULONG = 0x04000000;
pub const HEAP_NO_ALIGNMENT: ULONG = 0x08000000;
pub const HEAP_CAPTURE_STACK_BACKTRACES: ULONG = 0x08000000;
pub const HEAP_SKIP_VALIDATION_CHECKS: ULONG = 0x10000000;
pub const HEAP_VALIDATE_ALL_ENABLED: ULONG = 0x20000000;
pub const HEAP_VALIDATE_PARAMETERS_ENABLED: ULONG = 0x40000000;
pub const HEAP_LOCK_USER_ALLOCATED: ULONG = 0x80000000;

pub const HEAP_TAG_SHIFT: ULONG = 18;
pub const HEAP_MAXIMUM_TAG: ULONG = 0x0FFF;
pub const HEAP_TAG_MASK: ULONG = HEAP_MAXIMUM_TAG << HEAP_TAG_SHIFT;
pub const HEAP_TAGS_MASK: ULONG = HEAP_TAG_MASK ^ (0xFF << HEAP_TAG_SHIFT);
pub const HEAP_EXTRA_FLAGS_MASK: ULONG =
    HEAP_CAPTURE_STACK_BACKTRACES | HEAP_SETTABLE_USER_VALUE | HEAP_TAGS_MASK;

pub const HEAP_CREATE_VALID_MASK: ULONG = HEAP_NO_SERIALIZE
    | HEAP_GROWABLE
    | HEAP_GENERATE_EXCEPTIONS
    | HEAP_ZERO_MEMORY
    | HEAP_REALLOC_IN_PLACE_ONLY
    | HEAP_TAIL_CHECKING_ENABLED
    | HEAP_FREE_CHECKING_ENABLED
    | HEAP_DISABLE_COALESCE_ON_FREE
    | HEAP_CREATE_ALIGN_16
    | HEAP_CREATE_ENABLE_TRACING
    | HEAP_CREATE_ENABLE_EXECUTE;

// RTL_BITS_OF(ULONG) = 8 * sizeof(ULONG).
pub const RTL_BITS_OF_ULONG: ULONG = 8 * core::mem::size_of::<ULONG>() as ULONG;

// MEM_DECOMMIT (ZwFreeVirtualMemory) — not in ntkrnl-sys::base.
pub const MEM_DECOMMIT: ULONG = 0x4000;

// NtGlobalFlag bits (FLG_*) the heap reads via RtlGetNtGlobalFlags.
pub const FLG_HEAP_ENABLE_TAIL_CHECK: ULONG = 0x00000010;
pub const FLG_HEAP_ENABLE_FREE_CHECK: ULONG = 0x00000020;
pub const FLG_HEAP_VALIDATE_PARAMETERS: ULONG = 0x00000040;
pub const FLG_HEAP_VALIDATE_ALL: ULONG = 0x00000080;
pub const FLG_USER_STACK_TRACE_DB: ULONG = 0x00001000;
pub const FLG_HEAP_DISABLE_COALESCING: ULONG = 0x00200000;

// _DEBUG_BUFFER — only ever used as `*mut _DEBUG_BUFFER` in this file
// (RtlQueryProcessHeapInformation is a stub); opaque is ABI-correct.
pub type _DEBUG_BUFFER = core::ffi::c_void;

// ── bindgen-style `_`-prefixed aliases (the draft references these) ──
pub type _HEAP = HEAP;
pub type _HEAP_ENTRY = HEAP_ENTRY;
pub type _HEAP_FREE_ENTRY = HEAP_FREE_ENTRY;
pub type _HEAP_SEGMENT = HEAP_SEGMENT;
pub type _HEAP_UCR_DESCRIPTOR = HEAP_UCR_DESCRIPTOR;
pub type _HEAP_UCR_SEGMENT = HEAP_UCR_SEGMENT;
pub type _HEAP_VIRTUAL_ALLOC_ENTRY = HEAP_VIRTUAL_ALLOC_ENTRY;
pub type _HEAP_ENTRY_EXTRA = HEAP_ENTRY_EXTRA;
pub type HEAP_FREE_ENTRY_EXTRA = HEAP_ENTRY_EXTRA;
pub type PHEAP_FREE_ENTRY_EXTRA = *mut HEAP_ENTRY_EXTRA;

// ── compile-time layout assertions (heap.h C_ASSERTs + key prefixes) ─
use core::mem::size_of;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<HEAP_ENTRY>() == 16);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(size_of::<HEAP_ENTRY>() == 8);
const _: () = assert!((1usize << HEAP_ENTRY_SHIFT) == size_of::<HEAP_ENTRY>());
const _: () = assert!((2usize << HEAP_ENTRY_SHIFT) == size_of::<HEAP_FREE_ENTRY>());
