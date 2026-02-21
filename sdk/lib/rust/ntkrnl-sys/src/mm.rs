//! Memory manager types and extern declarations.
//!
//! Derived from `sdk/include/ndk/mmtypes.h`, `ntoskrnl/include/internal/mm.h`,
//! and `ntoskrnl/mm/ARM3/miarm.h`.

use crate::base::*;
use crate::ke::KSPIN_LOCK;

// ============================================================================
// POOL_TYPE enum
// ============================================================================

pub type POOL_TYPE = ULONG;

pub const NonPagedPool: POOL_TYPE = 0;
pub const PagedPool: POOL_TYPE = 1;
pub const NonPagedPoolMustSucceed: POOL_TYPE = 2;
pub const DontUseThisType: POOL_TYPE = 3;
pub const NonPagedPoolCacheAligned: POOL_TYPE = 4;
pub const PagedPoolCacheAligned: POOL_TYPE = 5;
pub const NonPagedPoolCacheAlignedMustS: POOL_TYPE = 6;
pub const MaxPoolType: POOL_TYPE = 7;
pub const NonPagedPoolSession: POOL_TYPE = 32;
pub const PagedPoolSession: POOL_TYPE = 33;

// Pool type mask flags (from ntoskrnl/include/internal/mm.h)
pub const BASE_POOL_TYPE_MASK: ULONG = 1;
pub const PAGED_POOL_MASK: ULONG = 1;
pub const MUST_SUCCEED_POOL_MASK: ULONG = 2;
pub const CACHE_ALIGNED_POOL_MASK: ULONG = 4;
pub const QUOTA_POOL_MASK: ULONG = 8;
pub const SESSION_POOL_MASK: ULONG = 32;
pub const VERIFIER_POOL_MASK: ULONG = 64;

// ============================================================================
// Pool constants (from miarm.h)
// ============================================================================

#[cfg(target_pointer_width = "64")]
pub const POOL_BLOCK_SIZE: usize = 16;
#[cfg(target_pointer_width = "32")]
pub const POOL_BLOCK_SIZE: usize = 8;

pub const POOL_LISTS_PER_PAGE: usize = PAGE_SIZE / POOL_BLOCK_SIZE;
// POOL_MAX_ALLOC = PAGE_SIZE - (sizeof(POOL_HEADER) + POOL_BLOCK_SIZE)
pub const POOL_MAX_ALLOC: usize = PAGE_SIZE - (POOL_BLOCK_SIZE + POOL_BLOCK_SIZE);

// Pool debugging flags
pub const POOL_FLAG_CHECK_TIMERS: ULONG = 0x1;
pub const POOL_FLAG_CHECK_WORKERS: ULONG = 0x2;
pub const POOL_FLAG_CHECK_RESOURCES: ULONG = 0x4;
pub const POOL_FLAG_VERIFIER: ULONG = 0x8;
pub const POOL_FLAG_CHECK_DEADLOCK: ULONG = 0x10;
pub const POOL_FLAG_SPECIAL_POOL: ULONG = 0x20;
pub const POOL_FLAG_DBGPRINT_ON_FAILURE: ULONG = 0x40;
pub const POOL_FLAG_CRASH_ON_FAILURE: ULONG = 0x80;

// BAD_POOL_HEADER sub-codes
pub const POOL_CORRUPTED_LIST: ULONG = 3;
pub const POOL_SIZE_OR_INDEX_MISMATCH: ULONG = 5;
pub const POOL_ENTRIES_NOT_ALIGNED_PREVIOUS: ULONG = 6;
pub const POOL_HEADER_NOT_ALIGNED: ULONG = 7;
pub const POOL_HEADER_IS_ZERO: ULONG = 8;
pub const POOL_ENTRIES_NOT_ALIGNED_NEXT: ULONG = 9;
pub const POOL_ENTRY_NOT_FOUND: ULONG = 10;

// BAD_POOL_CALLER sub-codes
pub const POOL_ENTRY_CORRUPTED: ULONG = 1;
pub const POOL_ENTRY_ALREADY_FREE: ULONG = 6;
pub const POOL_ENTRY_NOT_ALLOCATED: ULONG = 7;
pub const POOL_ALLOC_IRQL_INVALID: ULONG = 8;
pub const POOL_FREE_IRQL_INVALID: ULONG = 9;
pub const POOL_BILLED_PROCESS_INVALID: ULONG = 13;
pub const POOL_HEADER_SIZE_INVALID: ULONG = 32;

// Big page table constants
pub const POOL_BIG_TABLE_ENTRY_FREE: ULONG_PTR = 0x1;
pub const POOL_BIG_TABLE_USE_RATE: usize = 4;

// ============================================================================
// POOL_HEADER — per-allocation metadata (one POOL_BLOCK_SIZE)
// ============================================================================

/// On 64-bit: 16 bytes (PreviousSize:8 PoolIndex:8 BlockSize:8 PoolType:8 PoolTag:32 ProcessBilled:64)
/// On 32-bit: 8 bytes (PreviousSize:9 PoolIndex:7 BlockSize:9 PoolType:7 PoolTag:32)
///
/// We store the raw words and provide accessor methods for the bitfields.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct POOL_HEADER {
    /// Low 32 bits: packed bitfields (PreviousSize, PoolIndex, BlockSize, PoolType).
    pub Ulong1: ULONG,
    /// On 64-bit: PoolTag (4 bytes).
    #[cfg(target_pointer_width = "64")]
    pub PoolTag: ULONG,
    /// On 64-bit: ProcessBilled pointer (8 bytes) — overlaps with AllocatorBackTraceIndex/PoolTagHash.
    /// On 32-bit: PoolTag (4 bytes) — overlaps with AllocatorBackTraceIndex/PoolTagHash.
    #[cfg(target_pointer_width = "64")]
    pub ProcessBilled: usize,
    #[cfg(target_pointer_width = "32")]
    pub PoolTag: ULONG,
}

pub type PPOOL_HEADER = *mut POOL_HEADER;

impl POOL_HEADER {
    // 64-bit: each field is 8 bits
    // 32-bit: PreviousSize:9, PoolIndex:7, BlockSize:9, PoolType:7

    #[inline]
    pub fn PreviousSize(&self) -> u16 {
        #[cfg(target_pointer_width = "64")]
        { (self.Ulong1 & 0xFF) as u16 }
        #[cfg(target_pointer_width = "32")]
        { (self.Ulong1 & 0x1FF) as u16 }
    }

    #[inline]
    pub fn set_PreviousSize(&mut self, val: u16) {
        #[cfg(target_pointer_width = "64")]
        {
            self.Ulong1 = (self.Ulong1 & !0xFF) | (val as ULONG & 0xFF);
        }
        #[cfg(target_pointer_width = "32")]
        {
            self.Ulong1 = (self.Ulong1 & !0x1FF) | (val as ULONG & 0x1FF);
        }
    }

    #[inline]
    pub fn PoolIndex(&self) -> u16 {
        #[cfg(target_pointer_width = "64")]
        { ((self.Ulong1 >> 8) & 0xFF) as u16 }
        #[cfg(target_pointer_width = "32")]
        { ((self.Ulong1 >> 9) & 0x7F) as u16 }
    }

    #[inline]
    pub fn set_PoolIndex(&mut self, val: u16) {
        #[cfg(target_pointer_width = "64")]
        {
            self.Ulong1 = (self.Ulong1 & !(0xFF << 8)) | ((val as ULONG & 0xFF) << 8);
        }
        #[cfg(target_pointer_width = "32")]
        {
            self.Ulong1 = (self.Ulong1 & !(0x7F << 9)) | ((val as ULONG & 0x7F) << 9);
        }
    }

    #[inline]
    pub fn BlockSize(&self) -> u16 {
        #[cfg(target_pointer_width = "64")]
        { ((self.Ulong1 >> 16) & 0xFF) as u16 }
        #[cfg(target_pointer_width = "32")]
        { ((self.Ulong1 >> 16) & 0x1FF) as u16 }
    }

    #[inline]
    pub fn set_BlockSize(&mut self, val: u16) {
        #[cfg(target_pointer_width = "64")]
        {
            self.Ulong1 = (self.Ulong1 & !(0xFF << 16)) | ((val as ULONG & 0xFF) << 16);
        }
        #[cfg(target_pointer_width = "32")]
        {
            self.Ulong1 = (self.Ulong1 & !(0x1FF << 16)) | ((val as ULONG & 0x1FF) << 16);
        }
    }

    #[inline]
    pub fn PoolType(&self) -> u16 {
        #[cfg(target_pointer_width = "64")]
        { ((self.Ulong1 >> 24) & 0xFF) as u16 }
        #[cfg(target_pointer_width = "32")]
        { ((self.Ulong1 >> 25) & 0x7F) as u16 }
    }

    #[inline]
    pub fn set_PoolType(&mut self, val: u16) {
        #[cfg(target_pointer_width = "64")]
        {
            self.Ulong1 = (self.Ulong1 & !(0xFF << 24)) | ((val as ULONG & 0xFF) << 24);
        }
        #[cfg(target_pointer_width = "32")]
        {
            self.Ulong1 = (self.Ulong1 & !(0x7F << 25)) | ((val as ULONG & 0x7F) << 25);
        }
    }
}

// ============================================================================
// POOL_DESCRIPTOR — per-pool allocation state
// ============================================================================

#[repr(C)]
pub struct POOL_DESCRIPTOR {
    pub PoolType: POOL_TYPE,
    pub PoolIndex: ULONG,
    pub RunningAllocs: ULONG,
    pub RunningDeAllocs: ULONG,
    pub TotalPages: ULONG,
    pub TotalBigPages: ULONG,
    pub Threshold: ULONG,
    pub LockAddress: PVOID,
    pub PendingFrees: PVOID,
    pub PendingFreeDepth: LONG,
    pub TotalBytes: SIZE_T,
    pub Spare0: SIZE_T,
    pub ListHeads: [LIST_ENTRY; POOL_LISTS_PER_PAGE],
}

pub type PPOOL_DESCRIPTOR = *mut POOL_DESCRIPTOR;

// ============================================================================
// POOL_TRACKER_TABLE — tag-based allocation tracking
// ============================================================================

#[repr(C)]
pub struct POOL_TRACKER_TABLE {
    pub Key: ULONG,
    pub NonPagedAllocs: LONG,
    pub NonPagedFrees: LONG,
    pub NonPagedBytes: SIZE_T,
    pub PagedAllocs: LONG,
    pub PagedFrees: LONG,
    pub PagedBytes: SIZE_T,
}

pub type PPOOL_TRACKER_TABLE = *mut POOL_TRACKER_TABLE;

// ============================================================================
// POOL_TRACKER_BIG_PAGES — large allocation tracking
// ============================================================================

#[repr(C)]
pub struct POOL_TRACKER_BIG_PAGES {
    pub Va: PVOID,
    pub Key: ULONG,
    pub NumberOfPages: ULONG,
    pub QuotaObject: PVOID,
}

pub type PPOOL_TRACKER_BIG_PAGES = *mut POOL_TRACKER_BIG_PAGES;

// ============================================================================
// MMFREE_POOL_ENTRY — free pool page tracking
// ============================================================================

pub const MM_FREE_POOL_SIGNATURE: ULONG = u32::from_le_bytes(*b"ARM3");
pub const MI_MAX_FREE_PAGE_LISTS: usize = 4;

#[repr(C)]
pub struct MMFREE_POOL_ENTRY {
    pub List: LIST_ENTRY,
    pub Size: PFN_COUNT,
    pub Signature: ULONG,
    pub Owner: *mut MMFREE_POOL_ENTRY,
}

pub type PMMFREE_POOL_ENTRY = *mut MMFREE_POOL_ENTRY;

// ============================================================================
// MM_PAGED_POOL_INFO
// ============================================================================

/// Paged pool bitmap tracking — must match C struct _MM_PAGED_POOL_INFO
/// (ntoskrnl/include/internal/mm.h).
#[repr(C)]
pub struct MM_PAGED_POOL_INFO {
    pub PagedPoolAllocationMap: *mut RTL_BITMAP,
    pub EndOfPagedPoolBitmap: *mut RTL_BITMAP,
    pub FirstPteForPagedPool: PVOID,           // PMMPTE
    pub LastPteForPagedPool: PVOID,            // PMMPTE
    pub NextPdeForPagedPoolExpansion: PVOID,   // PMMPDE
    pub PagedPoolHint: ULONG,
    pub PagedPoolCommit: SIZE_T,
    pub AllocatedPagedPool: SIZE_T,
}

// ============================================================================
// RTL_BITMAP
// ============================================================================

#[repr(C)]
pub struct RTL_BITMAP {
    pub SizeOfBitMap: ULONG,
    pub Buffer: *mut ULONG,
}

pub type PRTL_BITMAP = *mut RTL_BITMAP;

// ============================================================================
// MMPTE — page table entry (opaque wrapper)
// ============================================================================

/// We represent MMPTE as a wrapper around a pointer-width integer.
/// The pool code accesses u.Hard.Valid (bit 0), u.Hard.PageFrameNumber,
/// u.Soft.Prototype, and u.Long. We provide accessor methods.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MMPTE {
    pub Long: ULONG_PTR,
}

pub type PMMPTE = *mut MMPTE;
pub type PMMPDE = *mut MMPTE; // PDE has same layout as PTE

impl MMPTE {
    #[inline]
    pub fn valid(&self) -> bool {
        (self.Long & 1) != 0
    }

    #[inline]
    pub fn set_valid(&mut self, valid: bool) {
        if valid {
            self.Long |= 1;
        } else {
            self.Long &= !1;
        }
    }

    /// Prototype bit — bit 10 on all architectures.
    #[inline]
    pub fn prototype(&self) -> bool {
        (self.Long & (1 << 10)) != 0
    }

    #[inline]
    pub fn set_prototype(&mut self, proto: bool) {
        if proto {
            self.Long |= 1 << 10;
        } else {
            self.Long &= !(1 << 10);
        }
    }

    /// Extract PageFrameNumber from a valid hardware PTE.
    #[inline]
    pub fn page_frame_number(&self) -> PFN_NUMBER {
        // Architecture-dependent extraction
        #[cfg(target_arch = "aarch64")]
        { (self.Long >> 12) & 0xFFFFFFFFF } // bits [47:12]
        #[cfg(target_arch = "x86_64")]
        { (self.Long >> 12) & 0xFFFFFFFFF } // bits [47:12]
        #[cfg(target_arch = "x86")]
        { (self.Long >> 12) & 0xFFFFF }     // bits [31:12]
    }

    #[inline]
    pub fn set_page_frame_number(&mut self, pfn: PFN_NUMBER) {
        #[cfg(target_arch = "aarch64")]
        {
            self.Long = (self.Long & !0xFFFFFFFFF000) | ((pfn & 0xFFFFFFFFF) << 12);
        }
        #[cfg(target_arch = "x86_64")]
        {
            self.Long = (self.Long & !0xFFFFFFFFF000) | ((pfn & 0xFFFFFFFFF) << 12);
        }
        #[cfg(target_arch = "x86")]
        {
            self.Long = (self.Long & !0xFFFFF000) | ((pfn & 0xFFFFF) << 12);
        }
    }
}

/// Extract PFN from a PTE.
#[inline]
pub fn PFN_FROM_PTE(pte: PMMPTE) -> PFN_NUMBER {
    unsafe { (*pte).page_frame_number() }
}

// ============================================================================
// MMPFN — page frame number database entry (opaque)
// ============================================================================

/// MMPFN is extremely architecture-dependent with complex unions.
/// We define it as an opaque byte array. Access goes through helper
/// functions or via extern C wrappers.
///
/// Size: i386=36, amd64=72, arm64=72 (verified by bindgen size_check)
#[cfg(target_pointer_width = "64")]
#[repr(C, align(8))]
pub struct MMPFN {
    pub _opaque: [u8; 72],
}

#[cfg(target_pointer_width = "32")]
#[repr(C, align(4))]
pub struct MMPFN {
    pub _opaque: [u8; 36],
}

pub type PMMPFN = *mut MMPFN;

// ============================================================================
// PTE/Virtual address translation helpers
// ============================================================================

// PTE_BASE and related constants are architecture-dependent.
// These must match the values in ntoskrnl's mm headers.

#[cfg(target_arch = "x86")]
pub const PTE_BASE: usize = 0xC0000000;
#[cfg(target_arch = "x86")]
pub const PDE_BASE: usize = 0xC0300000;

#[cfg(target_arch = "x86_64")]
pub const PTE_BASE: usize = 0xFFFFF68000000000;
#[cfg(target_arch = "x86_64")]
pub const PDE_BASE: usize = 0xFFFFF6FB40000000;

#[cfg(target_arch = "aarch64")]
pub const PTE_BASE: usize = 0xFFFFF68000000000;
#[cfg(target_arch = "aarch64")]
pub const PDE_BASE: usize = 0xFFFFF6FB40000000;

/// Convert virtual address to PTE pointer (MiAddressToPte).
///
/// Matches the C inline function in ntoskrnl/include/internal/{arm64,amd64}/mm.h:
///   Offset = (ULONG64)Address >> (12 - 3);
///   Offset &= 0xFFFFFFFFFULL << 3;
///   return (PMMPTE)(PTE_BASE + Offset);
///
/// The mask is critical: without it, kernel VAs (upper bits all 1s) produce
/// offsets that overflow the PTE self-mapping range into user space.
#[inline]
pub fn MiAddressToPte(va: PVOID) -> PMMPTE {
    #[cfg(target_arch = "x86")]
    {
        // i386: 2-level paging, 4-byte PTEs, no mask needed (20-bit PFN fits)
        let addr = va as u32;
        (PTE_BASE as u32 + ((addr >> 12) << 2)) as PMMPTE
    }
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        // amd64/arm64: 4-level paging, 8-byte PTEs, mask to 39 bits
        let addr = va as u64;
        let offset = (addr >> (12 - 3)) & (0xFFFFFFFFFu64 << 3);
        (PTE_BASE as u64 + offset) as PMMPTE
    }
}

/// Convert virtual address to PDE pointer (MiAddressToPde).
///
/// Matches the C inline function:
///   Offset = (ULONG64)Address >> (21 - 3);
///   Offset &= 0x7FFFFFFULL << 3;   (arm64/amd64)
#[inline]
pub fn MiAddressToPde(va: PVOID) -> PMMPDE {
    #[cfg(target_arch = "x86")]
    {
        // i386: single page directory, 4-byte entries
        let addr = va as u32;
        (PDE_BASE as u32 + ((addr >> 22) << 2)) as PMMPDE
    }
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        let addr = va as u64;
        let offset = (addr >> (21 - 3)) & (0x7FFFFFFu64 << 3);
        (PDE_BASE as u64 + offset) as PMMPDE
    }
}

/// Convert PTE pointer to virtual address (MiPteToAddress).
///
/// Uses signed arithmetic to sign-extend kernel VAs (bit 47=1 → bits 63:48 all 1s).
/// Matches C: `(PVOID)(((LONG64)PointerPte << 25) >> 16)`
#[inline]
pub fn MiPteToAddress(pte: PMMPTE) -> PVOID {
    #[cfg(target_arch = "x86")]
    {
        let pte_addr = pte as u32;
        (((pte_addr - PTE_BASE as u32) / 4) << PAGE_SHIFT) as PVOID
    }
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        // Signed shift left discards PTE_BASE bits, signed shift right sign-extends
        (((pte as i64) << 25) >> 16) as PVOID
    }
}

/// Convert PDE pointer to virtual address (MiPdeToAddress).
#[inline]
pub fn MiPdeToAddress(pde: PMMPDE) -> PVOID {
    #[cfg(target_arch = "x86")]
    {
        let pde_addr = pde as u32;
        (((pde_addr - PDE_BASE as u32) / 4) << 22) as PVOID
    }
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        (((pde as i64) << 34) >> 16) as PVOID
    }
}

/// Convert PDE pointer to PTE pointer (MiPdeToPte).
/// C: `#define MiPdeToPte(_Pde) ((PMMPTE)MiPteToAddress(_Pde))`
#[inline]
pub fn MiPdeToPte(pde: PMMPDE) -> PMMPTE {
    MiPteToAddress(pde as PMMPTE) as PMMPTE
}

/// Convert PTE pointer to PDE pointer (MiPteToPde).
/// C: `#define MiPteToPde(_Pte) ((PMMPDE)MiAddressToPde(_Pte))`
#[inline]
pub fn MiPteToPde(pte: PMMPTE) -> PMMPDE {
    MiAddressToPde(pte as PVOID)
}

// ============================================================================
// Extern declarations — MM functions called by pool allocator
// ============================================================================

extern "system" {
    // Page-level allocator (used when pool.c is not being replaced,
    // or for internal calls that stay in C)

    /// Allocate physical page from free list.
    pub fn MiRemoveAnyPage(Color: ULONG) -> PFN_NUMBER;

    /// Insert page back into free list.
    pub fn MiInsertPageInFreeList(PageFrameIndex: PFN_NUMBER);

    /// Initialize PFN database entry for a page used by another process.
    pub fn MiInitializePfnForOtherProcess(
        PageFrameIndex: PFN_NUMBER,
        PointerPte: PMMPTE,
        PteFrame: PFN_NUMBER,
    );

    /// Reserve system PTEs for pool expansion.
    pub fn MiReserveSystemPtes(
        NumberOfPtes: ULONG,
        SystemPtePoolType: ULONG, // MMSYSTEM_PTE_POOL_TYPE
    ) -> PMMPTE;

    /// Release system PTEs.
    pub fn MiReleaseSystemPtes(
        StartingPte: PMMPTE,
        NumberOfPtes: ULONG,
        SystemPtePoolType: ULONG,
    );

    /// Initialize system PTE space.
    pub fn MiInitializeSystemPtes(
        StartingPte: PMMPTE,
        NumberOfPtes: ULONG,
        SystemPtePoolType: ULONG,
    );

    // MiAcquirePfnLock / MiReleasePfnLock — inline below
    // MI_IS_PHYSICAL_ADDRESS — inline below

    /// Special pool functions.
    pub fn MmAllocateSpecialPool(
        NumberOfBytes: SIZE_T,
        Tag: ULONG,
        PoolType: POOL_TYPE,
        SpecialType: ULONG,
    ) -> PVOID;

    pub fn MmFreeSpecialPool(P: PVOID);

    pub fn MmIsSpecialPoolAddress(P: PVOID) -> BOOLEAN;

    pub fn MmUseSpecialPool(NumberOfBytes: SIZE_T, Tag: ULONG) -> BOOLEAN;

    // MI_PFN_ELEMENT — inline below
}

// ============================================================================
// Extern global variables (defined in C, accessed from Rust)
// ============================================================================

extern "C" {
    // Pool boundaries
    pub static mut MmNonPagedPoolStart: PVOID;
    pub static mut MmNonPagedPoolEnd: PVOID;
    pub static mut MmNonPagedPoolExpansionStart: PVOID;
    pub static mut MmPagedPoolStart: PVOID;
    pub static mut MmPagedPoolEnd: PVOID;

    // Pool sizing
    pub static mut MmSizeOfNonPagedPoolInBytes: SIZE_T;
    pub static mut MmMaximumNonPagedPoolInBytes: SIZE_T;
    pub static mut MmSizeOfPagedPoolInPages: PFN_NUMBER;
    pub static mut MmMaximumNonPagedPoolInPages: PFN_NUMBER;
    pub static mut MmNumberOfPhysicalPages: PFN_NUMBER;

    // Pool thresholds
    pub static mut MiLowNonPagedPoolThreshold: PFN_NUMBER;
    pub static mut MiHighNonPagedPoolThreshold: PFN_NUMBER;
    pub static mut MiLowPagedPoolThreshold: PFN_NUMBER;
    pub static mut MiHighPagedPoolThreshold: PFN_NUMBER;

    // Pool events
    pub static mut MiLowNonPagedPoolEvent: PVOID;  // PKEVENT
    pub static mut MiHighNonPagedPoolEvent: PVOID;  // PKEVENT
    pub static mut MiLowPagedPoolEvent: PVOID;      // PKEVENT
    pub static mut MiHighPagedPoolEvent: PVOID;      // PKEVENT

    // PDE template
    pub static ValidKernelPde: MMPTE;
    pub static ValidKernelPte: MMPTE;

    // System page directory (i386 only)
    #[cfg(target_arch = "x86")]
    pub static mut MmSystemPagePtes: *mut MMPTE;
    #[cfg(target_arch = "x86")]
    pub static mut MmSystemPageDirectory: *mut PFN_NUMBER;

    // System color for page allocation
    pub static mut MmSystemPageColor: ULONG;
    pub static mut MmSecondaryColorMask: ULONG;

    // PsInitialSystemProcess (pointer to initial system EPROCESS)
    pub static mut PsInitialSystemProcess: PVOID; // PEPROCESS
}

// ============================================================================
// Helper macros as inline functions
// ============================================================================

/// MI_GET_PAGE_COLOR
#[inline]
pub fn MI_GET_PAGE_COLOR(x: ULONG) -> ULONG {
    unsafe { x & MmSecondaryColorMask }
}

/// MI_GET_NEXT_COLOR
#[inline]
pub unsafe fn MI_GET_NEXT_COLOR() -> ULONG {
    MmSystemPageColor = MmSystemPageColor.wrapping_add(1);
    MI_GET_PAGE_COLOR(MmSystemPageColor)
}

/// MI_SET_USAGE / MI_SET_PROCESS2 — no-ops in free builds.
#[inline]
pub fn MI_SET_USAGE(_usage: ULONG) {}
#[inline]
pub fn MI_SET_PROCESS2(_name: &str) {}

/// Usage constants
pub const MI_USAGE_PAGED_POOL: ULONG = 8;
pub const MI_USAGE_NONPAGED_POOL: ULONG = 9;

// PTE write helpers
#[inline]
pub unsafe fn MI_WRITE_VALID_PTE(pte: PMMPTE, value: MMPTE) {
    core::ptr::write_volatile(pte, value);
}

#[inline]
pub unsafe fn MI_WRITE_VALID_PDE(pde: PMMPDE, value: MMPTE) {
    core::ptr::write_volatile(pde, value);
}

#[inline]
pub unsafe fn MI_WRITE_INVALID_PTE(pte: PMMPTE, value: MMPTE) {
    core::ptr::write_volatile(pte, value);
}

#[inline]
pub unsafe fn MI_ERASE_PTE(pte: PMMPTE) {
    core::ptr::write_volatile(pte, MMPTE { Long: 0 });
}

/// MI_MAKE_SOFTWARE_PTE — create a demand-zero PTE with given protection.
#[inline]
pub fn MI_MAKE_SOFTWARE_PTE(pte: &mut MMPTE, protection: ULONG) {
    // MM_PTE_SOFTWARE_PROTECTION_BITS is 5 on all architectures
    pte.Long = (protection as usize) << 5;
}

/// PTE_PER_PAGE
#[cfg(target_arch = "x86")]
pub const PTE_PER_PAGE: usize = 1024;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PTE_PER_PAGE: usize = 512;

/// SYSTEM_PD_SIZE
#[cfg(target_arch = "x86")]
pub const SYSTEM_PD_SIZE: usize = PAGE_SIZE;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const SYSTEM_PD_SIZE: usize = PTE_PER_PAGE * PTE_PER_PAGE * core::mem::size_of::<MMPTE>();

/// MM_READWRITE protection value
pub const MM_READWRITE: ULONG = 4;

/// NonPagedPoolExpansion — MMSYSTEM_PTE_POOL_TYPE value
pub const NonPagedPoolExpansion: ULONG = 1;

// ============================================================================
// MMPFN field accessors
// ============================================================================

// MMPFN field offsets by architecture
#[cfg(target_pointer_width = "64")]
mod pfn_offsets {
    pub const PTE_ADDRESS: usize = 8;
    pub const U2: usize = 16;          // ShareCount (ULONG_PTR)
    pub const U3: usize = 24;          // ReferenceCount(u16) + MMPFNENTRY(u16)
    pub const U4: usize = 40;          // EntireFrame / PteFrame / VerifierAllocation
    pub const MI_PTE_FRAME_BITS: u32 = 57;
}

#[cfg(target_pointer_width = "32")]
mod pfn_offsets {
    pub const PTE_ADDRESS: usize = 4;
    pub const U2: usize = 8;
    pub const U3: usize = 12;
    pub const U4: usize = 20;
    pub const MI_PTE_FRAME_BITS: u32 = 25;
}

/// MMPFNENTRY bit positions within the u16 at u3+2
const MMPFNENTRY_START_OF_ALLOC: u16 = 1 << 1;   // ReadInProgress = StartOfAllocation
const MMPFNENTRY_END_OF_ALLOC: u16 = 1 << 2;     // WriteInProgress = EndOfAllocation
const MMPFNENTRY_PAGE_LOCATION_SHIFT: u32 = 8;
const MMPFNENTRY_PAGE_LOCATION_MASK: u16 = 0x7 << 8;
const MMPFNENTRY_CACHE_ATTRIBUTE_SHIFT: u32 = 12;

/// MMLISTS enum values
pub const ActiveAndValid: u16 = 6;

/// MI_PFN_CACHE_ATTRIBUTE
pub const MiCached: u16 = 1;

impl MMPFN {
    #[inline]
    pub unsafe fn pte_address(&self) -> PMMPTE {
        let p = (self as *const _ as *const u8).add(pfn_offsets::PTE_ADDRESS);
        *(p as *const PMMPTE)
    }

    #[inline]
    pub unsafe fn set_pte_address(&mut self, pte: PMMPTE) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets::PTE_ADDRESS);
        *(p as *mut PMMPTE) = pte;
    }

    #[inline]
    pub unsafe fn share_count(&self) -> ULONG_PTR {
        let p = (self as *const _ as *const u8).add(pfn_offsets::U2);
        *(p as *const ULONG_PTR)
    }

    #[inline]
    pub unsafe fn set_share_count(&mut self, val: ULONG_PTR) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets::U2);
        *(p as *mut ULONG_PTR) = val;
    }

    #[inline]
    pub unsafe fn reference_count(&self) -> USHORT {
        let p = (self as *const _ as *const u8).add(pfn_offsets::U3);
        *(p as *const USHORT)
    }

    #[inline]
    pub unsafe fn set_reference_count(&mut self, val: USHORT) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets::U3);
        *(p as *mut USHORT) = val;
    }

    #[inline]
    unsafe fn mmpfnentry(&self) -> USHORT {
        let p = (self as *const _ as *const u8).add(pfn_offsets::U3 + 2);
        *(p as *const USHORT)
    }

    #[inline]
    unsafe fn set_mmpfnentry(&mut self, val: USHORT) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets::U3 + 2);
        *(p as *mut USHORT) = val;
    }

    #[inline]
    pub unsafe fn start_of_allocation(&self) -> bool {
        (self.mmpfnentry() & MMPFNENTRY_START_OF_ALLOC) != 0
    }

    #[inline]
    pub unsafe fn set_start_of_allocation(&mut self, val: bool) {
        let mut e = self.mmpfnentry();
        if val { e |= MMPFNENTRY_START_OF_ALLOC; } else { e &= !MMPFNENTRY_START_OF_ALLOC; }
        self.set_mmpfnentry(e);
    }

    #[inline]
    pub unsafe fn end_of_allocation(&self) -> bool {
        (self.mmpfnentry() & MMPFNENTRY_END_OF_ALLOC) != 0
    }

    #[inline]
    pub unsafe fn set_end_of_allocation(&mut self, val: bool) {
        let mut e = self.mmpfnentry();
        if val { e |= MMPFNENTRY_END_OF_ALLOC; } else { e &= !MMPFNENTRY_END_OF_ALLOC; }
        self.set_mmpfnentry(e);
    }

    #[inline]
    pub unsafe fn page_location(&self) -> u16 {
        (self.mmpfnentry() & MMPFNENTRY_PAGE_LOCATION_MASK) >> MMPFNENTRY_PAGE_LOCATION_SHIFT
    }

    #[inline]
    pub unsafe fn set_page_location(&mut self, val: u16) {
        let mut e = self.mmpfnentry();
        e &= !MMPFNENTRY_PAGE_LOCATION_MASK;
        e |= (val << MMPFNENTRY_PAGE_LOCATION_SHIFT) & MMPFNENTRY_PAGE_LOCATION_MASK;
        self.set_mmpfnentry(e);
    }

    #[inline]
    pub unsafe fn set_cache_attribute(&mut self, val: u16) {
        let mut e = self.mmpfnentry();
        e &= !(0x3 << MMPFNENTRY_CACHE_ATTRIBUTE_SHIFT);
        e |= (val & 0x3) << MMPFNENTRY_CACHE_ATTRIBUTE_SHIFT;
        self.set_mmpfnentry(e);
    }

    /// u4.EntireFrame — full ULONG_PTR at u4 offset.
    #[inline]
    pub unsafe fn entire_frame(&self) -> ULONG_PTR {
        let p = (self as *const _ as *const u8).add(pfn_offsets::U4);
        *(p as *const ULONG_PTR)
    }

    #[inline]
    pub unsafe fn set_entire_frame(&mut self, val: ULONG_PTR) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets::U4);
        *(p as *mut ULONG_PTR) = val;
    }

    /// u4.PteFrame — lower MI_PTE_FRAME_BITS of u4.
    #[inline]
    pub unsafe fn pte_frame(&self) -> PFN_NUMBER {
        let mask = (1usize << pfn_offsets::MI_PTE_FRAME_BITS) - 1;
        self.entire_frame() & mask
    }

    #[inline]
    pub unsafe fn set_pte_frame(&mut self, val: PFN_NUMBER) {
        let mask = (1usize << pfn_offsets::MI_PTE_FRAME_BITS) - 1;
        let cur = self.entire_frame();
        self.set_entire_frame((cur & !mask) | (val & mask));
    }

    /// u4.VerifierAllocation — bit at MI_PTE_FRAME_BITS + 1.
    #[inline]
    pub unsafe fn verifier_allocation(&self) -> bool {
        (self.entire_frame() & (1usize << (pfn_offsets::MI_PTE_FRAME_BITS + 1))) != 0
    }

    #[inline]
    pub unsafe fn set_verifier_allocation(&mut self, val: bool) {
        let bit = 1usize << (pfn_offsets::MI_PTE_FRAME_BITS + 1);
        let cur = self.entire_frame();
        if val {
            self.set_entire_frame(cur | bit);
        } else {
            self.set_entire_frame(cur & !bit);
        }
    }
}

/// MiGetPfnEntry — returns PMMPFN from PFN number.
#[inline]
pub unsafe fn MiGetPfnEntry(pfn: PFN_NUMBER) -> PMMPFN {
    if pfn > MmHighestPhysicalPage {
        return core::ptr::null_mut();
    }
    MmPfnDatabase.add(pfn)
}

/// MiGetPfnEntryIndex — returns PFN number from PMMPFN.
#[inline]
pub unsafe fn MiGetPfnEntryIndex(pfn1: PMMPFN) -> PFN_NUMBER {
    pfn1.offset_from(MmPfnDatabase) as PFN_NUMBER
}

// ============================================================================
// Additional extern globals for pool.c
// ============================================================================

extern "C" {
    pub static mut MmPfnDatabase: PMMPFN;
    pub static mut MmHighestPhysicalPage: PFN_NUMBER;
    pub static mut MmAvailablePages: PFN_NUMBER;
    pub static mut MmResidentAvailablePages: isize; // SPFN_NUMBER (signed)
    pub static mut MmPfnLock: KSPIN_LOCK;
}

extern "system" {
    /// Delete system pageable VM pages.
    pub fn MiDeleteSystemPageableVm(
        PointerPte: PMMPTE,
        PageCount: PFN_NUMBER,
        Flags: ULONG,
        ResidentPages: *mut PFN_NUMBER,
    ) -> PFN_NUMBER;

    /// Initialize and charge PFN for session.
    pub fn MiInitializeAndChargePfn(
        PageFrameIndex: *mut PFN_NUMBER,
        PointerPde: PMMPDE,
        ContainingPageFrame: PFN_NUMBER,
        SessionAllocation: BOOLEAN,
    ) -> NTSTATUS;
}

/// MiAcquirePfnLockAtDpcLevel — acquires PFN lock when already at DISPATCH_LEVEL.
#[inline]
pub unsafe fn MiAcquirePfnLockAtDpcLevel() {
    extern "system" {
        fn KeAcquireSpinLockAtDpcLevel(SpinLock: *mut KSPIN_LOCK);
    }
    KeAcquireSpinLockAtDpcLevel(&raw mut MmPfnLock);
}

/// MiReleasePfnLockFromDpcLevel — releases PFN lock staying at DISPATCH_LEVEL.
#[inline]
pub unsafe fn MiReleasePfnLockFromDpcLevel() {
    extern "system" {
        fn KeReleaseSpinLockFromDpcLevel(SpinLock: *mut KSPIN_LOCK);
    }
    KeReleaseSpinLockFromDpcLevel(&raw mut MmPfnLock);
}

/// MiAcquirePfnLock — acquires PFN lock, returns old IRQL.
#[inline]
pub unsafe fn MiAcquirePfnLock() -> KIRQL {
    use crate::ke::KeAcquireSpinLock;
    let mut old_irql: KIRQL = 0;
    KeAcquireSpinLock(&raw mut MmPfnLock, &mut old_irql);
    old_irql
}

/// MiReleasePfnLock — releases PFN lock, restores IRQL.
#[inline]
pub unsafe fn MiReleasePfnLock(old_irql: KIRQL) {
    use crate::ke::KeReleaseSpinLock;
    KeReleaseSpinLock(&raw mut MmPfnLock, old_irql);
}

/// MI_IS_PHYSICAL_ADDRESS — checks if address is in the initial (non-expansion)
/// nonpaged pool range. These addresses are directly mapped to physical memory.
#[inline]
pub unsafe fn MI_IS_PHYSICAL_ADDRESS(va: PVOID) -> BOOLEAN {
    let addr = va as ULONG_PTR;
    let start = MmNonPagedPoolStart as ULONG_PTR;
    if addr >= start && addr < start + MmSizeOfNonPagedPoolInBytes {
        TRUE
    } else {
        FALSE
    }
}

/// MI_PFN_ELEMENT — get PFN database entry by page frame index.
/// Equivalent to `&MmPfnDatabase[PageFrameIndex]`.
#[inline]
pub unsafe fn MI_PFN_ELEMENT(page_frame_index: PFN_NUMBER) -> PMMPFN {
    MmPfnDatabase.add(page_frame_index)
}

// ============================================================================
// Pool quota constants
// ============================================================================

pub const MI_QUOTA_NON_PAGED_NEEDED_PAGES: PFN_NUMBER = 64;
pub const MI_NON_PAGED_QUOTA_MIN_RESIDENT_PAGES: isize = 200;
pub const MI_CHARGE_NON_PAGED_POOL_QUOTA: usize = 0x10000;
pub const MI_CHARGE_PAGED_POOL_QUOTA: usize = 0x80000;

// ============================================================================
// System PTE constants
// ============================================================================

pub const SystemPteSpace: ULONG = 0;

/// SYSTEM_PTE_MISUSE bugcheck code.
pub const SYSTEM_PTE_MISUSE: ULONG = 0xDA;

// PTE mapping bugcheck sub-codes
pub const PTE_MAPPING_NONE: ULONG_PTR = 0x100;
pub const PTE_MAPPING_NOT_OWNED: ULONG_PTR = 0x101;
pub const PTE_MAPPING_EMPTY: ULONG_PTR = 0x102;
pub const PTE_MAPPING_RESERVED: ULONG_PTR = 0x103;

/// MM_EMPTY_PTE_LIST — sentinel for PTE free list.
pub const MM_EMPTY_PTE_LIST: ULONG_PTR = 0xFFFFFFFF;

// ============================================================================
// PDE_PER_PAGE
// ============================================================================

#[cfg(target_arch = "x86")]
pub const PDE_PER_PAGE: usize = 1024;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PDE_PER_PAGE: usize = 512;

// ============================================================================
// Compile-time semantic assertions (size-vs-C assertions in size_check_*.rs)
// ============================================================================

const _: () = {
    // POOL_HEADER must be exactly POOL_BLOCK_SIZE (pool allocator invariant)
    assert!(core::mem::size_of::<POOL_HEADER>() == POOL_BLOCK_SIZE);
    // POOL_BLOCK_SIZE must equal LIST_ENTRY size (free list overlay invariant)
    assert!(POOL_BLOCK_SIZE == core::mem::size_of::<LIST_ENTRY>());
};
