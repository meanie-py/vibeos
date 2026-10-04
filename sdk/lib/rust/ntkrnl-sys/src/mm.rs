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

    /// MMPTE_PROTOTYPE.ReadOnly — bit 8 on all arches.
    ///
    /// Only meaningful when the PTE is in prototype format. Kept as a named
    /// accessor (not a raw mask at call sites) because the *software*-protection
    /// ReadOnly bit sits at a different position on ARM64 — don't conflate the two.
    /// (The prior per-arch cfg here had byte-identical branches — spurious.)
    #[inline]
    pub fn proto_read_only(&self) -> bool {
        (self.Long & (1 << 8)) != 0
    }

    #[inline]
    pub fn set_proto_read_only(&mut self, read_only: bool) {
        if read_only {
            self.Long |= 1 << 8;
        } else {
            self.Long &= !(1 << 8);
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

    // ========================================================================
    // MMPTE_LIST accessors (system PTE free list management)
    // ========================================================================

    /// OneEntry — bit 1 on amd64/i386, bit 6 on ARM64
    #[inline]
    pub fn list_one_entry(&self) -> ULONG_PTR {
        #[cfg(target_arch = "aarch64")]
        { (self.Long >> 6) & 1 }
        #[cfg(not(target_arch = "aarch64"))]
        { (self.Long >> 1) & 1 }
    }

    #[inline]
    pub fn set_list_one_entry(&mut self, val: ULONG_PTR) {
        #[cfg(target_arch = "aarch64")]
        { self.Long = (self.Long & !(1 << 6)) | ((val & 1) << 6); }
        #[cfg(not(target_arch = "aarch64"))]
        { self.Long = (self.Long & !(1 << 1)) | ((val & 1) << 1); }
    }

    /// NextEntry — bits [31:12] on i386, bits [63:32] on amd64, bits [63:28] on ARM64
    #[inline]
    pub fn list_next_entry(&self) -> ULONG_PTR {
        #[cfg(target_arch = "aarch64")]
        { (self.Long >> 28) as ULONG_PTR }
        #[cfg(target_arch = "x86_64")]
        { (self.Long >> 32) as ULONG_PTR }
        #[cfg(target_arch = "x86")]
        { (self.Long >> 12) & 0xFFFFF }
    }

    #[inline]
    pub fn set_list_next_entry(&mut self, val: ULONG_PTR) {
        #[cfg(target_arch = "aarch64")]
        {
            self.Long = (self.Long & 0xFFFFFFF) | ((val as ULONG_PTR) << 28);
        }
        #[cfg(target_arch = "x86_64")]
        {
            self.Long = (self.Long & 0xFFFFFFFF) | ((val as ULONG_PTR) << 32);
        }
        #[cfg(target_arch = "x86")]
        {
            self.Long = (self.Long & !0xFFFFF000) | ((val & 0xFFFFF) << 12);
        }
    }

    // ========================================================================
    // MMPTE_SOFTWARE accessors (soft/invalid PTE fields)
    // ========================================================================

    /// Protection field bit layout in MMPTE_SOFTWARE differs by architecture.
    /// amd64/i386: bits 5..=9 (Valid:1, PageFileLow:4, Protection:5)
    /// arm64: bits 1..=5 (Valid:1, Protection:5, PageFileLow:4) — see ndk/arm64/mmtypes.h
    #[cfg(target_arch = "aarch64")]
    const SOFT_PROTECTION_SHIFT: u32 = 1;
    #[cfg(not(target_arch = "aarch64"))]
    const SOFT_PROTECTION_SHIFT: u32 = 5;

    #[inline]
    pub fn soft_protection(&self) -> ULONG {
        ((self.Long >> Self::SOFT_PROTECTION_SHIFT) & 0x1F) as ULONG
    }

    #[inline]
    pub fn set_soft_protection(&mut self, val: ULONG) {
        let shift = Self::SOFT_PROTECTION_SHIFT;
        self.Long = (self.Long & !(0x1F << shift)) | (((val as ULONG_PTR) & 0x1F) << shift);
    }

    /// UsedPageTableEntries in a software PTE.
    /// amd64: bits 12..=21; arm64: bits 14..=23; i386 reuses PageFileHigh.
    #[inline]
    pub fn soft_used_page_table_entries(&self) -> ULONG {
        #[cfg(target_arch = "aarch64")]
        { ((self.Long >> 14) & 0x3FF) as ULONG }
        #[cfg(target_arch = "x86_64")]
        { ((self.Long >> 12) & 0x3FF) as ULONG }
        #[cfg(target_arch = "x86")]
        { ((self.Long >> 12) & 0xFFFFF) as ULONG }
    }

    #[inline]
    pub fn set_soft_used_page_table_entries(&mut self, val: ULONG) {
        #[cfg(target_arch = "aarch64")]
        {
            self.Long = (self.Long & !(0x3FFusize << 14)) | (((val as ULONG_PTR) & 0x3FF) << 14);
        }
        #[cfg(target_arch = "x86_64")]
        {
            self.Long = (self.Long & !(0x3FFusize << 12)) | (((val as ULONG_PTR) & 0x3FF) << 12);
        }
        #[cfg(target_arch = "x86")]
        {
            self.Long = (self.Long & !0xFFFFF000usize) | (((val as ULONG_PTR) & 0xFFFFF) << 12);
        }
    }

    /// PageFileHigh — bits [31:12] on i386, bits [63:32] on 64-bit
    #[inline]
    pub fn soft_page_file_high(&self) -> ULONG_PTR {
        #[cfg(target_pointer_width = "64")]
        { (self.Long >> 32) as ULONG_PTR }
        #[cfg(target_pointer_width = "32")]
        { (self.Long >> 12) & 0xFFFFF }
    }

    #[inline]
    pub fn set_soft_page_file_high(&mut self, val: ULONG_PTR) {
        #[cfg(target_pointer_width = "64")]
        {
            self.Long = (self.Long & 0xFFFFFFFF) | ((val as ULONG_PTR) << 32);
        }
        #[cfg(target_pointer_width = "32")]
        {
            self.Long = (self.Long & !0xFFFFF000) | ((val & 0xFFFFF) << 12);
        }
    }

    /// Transition bit — bit 11
    #[inline]
    pub fn transition(&self) -> bool {
        (self.Long & (1 << 11)) != 0
    }

    #[inline]
    pub fn set_transition(&mut self, val: bool) {
        if val {
            self.Long |= 1 << 11;
        } else {
            self.Long &= !(1 << 11);
        }
    }

    /// Write bit — bit 1 in hardware PTE
    #[inline]
    pub fn write(&self) -> bool {
        (self.Long & (1 << 1)) != 0
    }

    #[inline]
    pub fn set_write(&mut self, val: bool) {
        if val {
            self.Long |= 1 << 1;
        } else {
            self.Long &= !(1 << 1);
        }
    }

    /// Owner bit — bit 2
    #[inline]
    pub fn owner(&self) -> bool {
        (self.Long & (1 << 2)) != 0
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
///
/// One self-map recursion step: the PDE of X *is* the PTE of (the PTE of X),
/// so `MiPteToPde(pte) == MiAddressToPte(pte)` on every self-mapped arch.
/// (An earlier aarch64 branch used `MiAddressToPde(pte)` here — that returns
/// the **PPE** of the original VA, which made `MiDeletePte` decrement the
/// page-*directory*'s share count instead of the page table's: BUGCHECK 0x4E
/// subcode 0x99 at process teardown. See field-manual §9.16.)
#[inline]
pub fn MiPteToPde(pte: PMMPTE) -> PMMPDE {
    #[cfg(target_arch = "x86")]
    {
        MiAddressToPte(pte as PVOID) as PMMPDE
    }
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        MiAddressToPte(pte as PVOID) as PMMPDE
    }
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
    pub static mut MmSystemPageDirectory: [PFN_NUMBER; PPE_PER_PAGE];

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

// MI_USAGE_PAGED_POOL / MI_USAGE_NONPAGED_POOL — see MI_PFN_USAGES enum below

// PTE write helpers
/// Make a just-written translation-table entry visible to the HARDWARE page
/// table walker.
///
/// On x86 this is free: a store to a page table is guaranteed visible to the
/// walker, so making an invalid entry valid needs nothing but the store. On
/// AArch64 the walker is a SEPARATE OBSERVER — the store may still be sitting
/// in the store buffer, where this core's own loads see it and the walker does
/// not. Software then walks the table, reads "valid", and reports the fault
/// resolved, while the MMU keeps taking a translation fault on the same
/// address. Both are telling the truth about different copies.
///
/// `dsb ishst` is the architecturally required publish (inner-shareable, stores
/// only — the minimum that orders the entry write ahead of any subsequent
/// walk). No TLBI: invalid -> valid needs no invalidate, and the callers that
/// change a LIVE mapping already do their own break-before-make. No `isb`
/// either — the return to the faulting instruction is context-synchronising.
///
/// This lives in the shared helper rather than at the call sites because the
/// arch-neutral fault path (`ARM3/pagfault.rs`) writes valid entries at ~15
/// places and had the barrier at none of them; `arm64/page.rs` paired it by
/// hand at exactly two (its `dsb ish`/`isb` pairs are now redundant, and
/// harmless). Centralising it is the only way the next new call site is right
/// by default.
/// Refuse to build a VALID user PTE for a protection that grants no access.
///
/// The C original opens with three ASSERTs whose entire job is this check —
/// "check that we are not setting valid a page that should not be":
///
/// ```c
/// ASSERT(ProtectionMask & MM_PROTECT_ACCESS);
/// ASSERT((ProtectionMask & MM_PROTECT_SPECIAL) != MM_GUARDPAGE);
/// ASSERT(ProtectionMask != MM_OUTSWAPPED_KSTACK && ...);
/// ```
///
/// The translation dropped all three. On x86 that loss is survivable: the mask
/// contributes nothing, so the PTE comes out Valid|User|PFN — a wrongly
/// READABLE page, but a page the hardware can translate. On arm64 it freezes
/// the machine, because `MmProtectToPteMask` is the only source of bit 1
/// (NotLargePage) and bit 10 (AF) there. A no-access mask therefore yields
/// Valid=1 with descriptor type 0b01, which is RESERVED at level 3: the MMU
/// raises a translation fault (DFSC 0x07) that software cannot see, because
/// `MmAccessFault` reads Valid=1 and reports STATUS_SUCCESS. The faulting
/// instruction retries forever with DAIF masked — no timer, no IPI, no
/// bugcheck, a dead machine whose PC never moves.
///
/// So this is not a lost debug assert; it is the difference between a wrong
/// page and an unkillable fault loop. Returning `false` leaves the entry
/// not-present, which is what the caller's own protection mask asked for: the
/// fault then takes the invalid path and the thread gets a clean access
/// violation instead of taking the box down with it.
#[inline]
#[track_caller]
pub unsafe fn mi_user_pte_may_be_valid(protection_mask: usize) -> bool {
    // MM_PROTECT_ACCESS is the three access bits; a mask with none of them set
    // names MM_ZERO_ACCESS (0), MM_NOACCESS (8), MM_GUARDPAGE/MM_DECOMMIT (0x10)
    // or 0x18 — exactly the four zero rows of MmProtectToPteMask.
    if protection_mask & MM_PROTECT_ACCESS as usize != 0 {
        return true;
    }
    let loc = core::panic::Location::caller();
    mi_report_no_access_valid_pte(protection_mask, loc.file(), loc.line());
    false
}

/// One line per distinct offending mask per boot — enough to name the caller's
/// protection without turning a hot fault path into a serial-console flood.
///
/// The call site comes from `#[track_caller]` rather than a read of x30 or a tag
/// threaded through every caller: it is stable Rust, exact, and costs nothing at
/// the sites. It only reaches the real ARM3 caller because
/// `MI_MAKE_HARDWARE_PTE_USER` is `#[track_caller]` too — without that the
/// location stops at the helper and names itself.
#[inline(never)]
unsafe fn mi_report_no_access_valid_pte(protection_mask: usize, file: &str, line: u32) {
    static mut REPORTED: u32 = 0;
    let bit = 1u32 << (protection_mask & 31);
    if REPORTED & bit != 0 {
        return;
    }
    REPORTED |= bit;

    // Location::file() is a plain &str, not NUL-terminated, so it cannot be
    // handed to DbgPrint's %s directly. Copy the TAIL (the interesting half:
    // ".../ARM3/pagfault.rs") into a NUL-terminated stack buffer.
    let mut buf = [0u8; 64];
    let src = file.as_bytes();
    let take = if src.len() > 63 { 63 } else { src.len() };
    let start = src.len() - take;
    let mut i = 0;
    while i < take {
        buf[i] = src[start + i];
        i += 1;
    }
    buf[take] = 0;

    crate::driver_ddk::DbgPrint(
        b"[NOACCESSPTE] refused a VALID user PTE for protection mask %u at %s:%u \
          (grants no access; would be a hardware-invalid L3 descriptor on arm64)\n\0"
            .as_ptr() as *const i8,
        protection_mask as u32,
        buf.as_ptr() as *const i8,
        line,
    );
}

#[inline(always)]
unsafe fn mi_publish_pte() {
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("dsb ishst", options(nostack, preserves_flags));
}

#[inline]
pub unsafe fn MI_WRITE_VALID_PTE(pte: PMMPTE, value: MMPTE) {
    core::ptr::write_volatile(pte, value);
    mi_publish_pte();
}

#[inline]
pub unsafe fn MI_WRITE_VALID_PDE(pde: PMMPDE, value: MMPTE) {
    core::ptr::write_volatile(pde, value);
    mi_publish_pte();
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
/// Uses arch-aware shift since ARM64's MMPTE_SOFTWARE puts Protection at bit 1
/// while amd64/i386 put it at bit 5.
#[inline]
pub fn MI_MAKE_SOFTWARE_PTE(pte: &mut MMPTE, protection: ULONG) {
    pte.Long = 0;
    pte.set_soft_protection(protection);
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

// MMPFN field offsets — derived from bindgen (which offset_check_<arch>.rs
// asserts against C __builtin_offsetof), so they CANNOT drift, on any arch.
// The hand-maintained per-arch numeric tables here caused the 2026-07-05
// one-slot-shift bug (0xBE); never reintroduce numerals.
mod pfn_offsets {
    use core::mem::offset_of;
    use crate::ddk::_MMPFN;
    pub const PTE_ADDRESS: usize = offset_of!(_MMPFN, PteAddress);
    pub const U2: usize = offset_of!(_MMPFN, u2); // ShareCount (ULONG_PTR)
    pub const U3: usize = offset_of!(_MMPFN, u3); // ReferenceCount(u16) + MMPFNENTRY(u16)
    pub const U4: usize = offset_of!(_MMPFN, u4); // EntireFrame / PteFrame / VerifierAllocation
    #[cfg(target_pointer_width = "64")]
    pub const MI_PTE_FRAME_BITS: u32 = 57;
    #[cfg(target_pointer_width = "32")]
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

    /// u4.InPageError — bit immediately after PteFrame.
    #[inline]
    pub unsafe fn in_page_error(&self) -> bool {
        (self.entire_frame() & (1usize << pfn_offsets::MI_PTE_FRAME_BITS)) != 0
    }

    #[inline]
    pub unsafe fn set_in_page_error(&mut self, val: bool) {
        let bit = 1usize << pfn_offsets::MI_PTE_FRAME_BITS;
        let cur = self.entire_frame();
        if val {
            self.set_entire_frame(cur | bit);
        } else {
            self.set_entire_frame(cur & !bit);
        }
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

// ============================================================================
// MMPFN OriginalPte accessors
// ============================================================================

// OriginalPte — the {OriginalPte|AweReferenceCount|RmapListHead} union, which
// bindgen names __bindgen_anon_1. Derived from bindgen (see pfn_offsets banner).
mod pfn_offsets_orig_pte {
    pub const ORIGINAL_PTE: usize = core::mem::offset_of!(crate::ddk::_MMPFN, __bindgen_anon_1);
}

impl MMPFN {
    /// OriginalPte — MMPTE stored in the PFN entry (between u3 and u4)
    #[inline]
    pub unsafe fn original_pte(&self) -> MMPTE {
        let p = (self as *const _ as *const u8).add(pfn_offsets_orig_pte::ORIGINAL_PTE);
        *(p as *const MMPTE)
    }

    #[inline]
    pub unsafe fn set_original_pte(&mut self, val: MMPTE) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets_orig_pte::ORIGINAL_PTE);
        *(p as *mut MMPTE) = val;
    }
}

/// MiGetPfnEntry — returns PMMPFN from PFN number.
/// Matches C version in mm.h: checks MmHighestPhysicalPage and MiPfnBitMap.
#[inline]
pub unsafe fn MiGetPfnEntry(pfn: PFN_NUMBER) -> PMMPFN {
    extern "C" {
        static mut MiPfnBitMap: RTL_BITMAP;
    }
    if pfn > MmHighestPhysicalPage {
        return core::ptr::null_mut();
    }
    if !MiPfnBitMap.Buffer.is_null()
        && crate::rtl::RtlTestBit(core::ptr::addr_of_mut!(MiPfnBitMap), pfn as ULONG) == 0
    {
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
/// i386: 20-bit NextEntry field → 0xFFFFF
/// amd64/arm64: 32-bit NextEntry field → 0xFFFFFFFF
#[cfg(target_arch = "x86")]
pub const MM_EMPTY_PTE_LIST: ULONG_PTR = 0xFFFFF;
#[cfg(not(target_arch = "x86"))]
pub const MM_EMPTY_PTE_LIST: ULONG_PTR = 0xFFFFFFFF;

// ============================================================================
// PDE_PER_PAGE
// ============================================================================

#[cfg(target_arch = "x86")]
pub const PDE_PER_PAGE: usize = 1024;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PDE_PER_PAGE: usize = 512;

// ============================================================================
// PPE_PER_PAGE, PXE_PER_PAGE (64-bit only)
// ============================================================================

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PPE_PER_PAGE: usize = 512;
#[cfg(target_arch = "x86")]
pub const PPE_PER_PAGE: usize = 1;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PXE_PER_PAGE: usize = 512;

// ============================================================================
// _MI_PAGING_LEVELS
// ============================================================================

#[cfg(target_arch = "x86")]
pub const _MI_PAGING_LEVELS: usize = 2;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const _MI_PAGING_LEVELS: usize = 4;

// ============================================================================
// PPE_BASE, PXE_BASE (64-bit only, for 4-level paging)
// ============================================================================

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PPE_BASE: usize = 0xFFFFF6FB7DA00000;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PXE_BASE: usize = 0xFFFFF6FB7DBED000;

// PTE_TOP — end of PTE self-map range
#[cfg(target_arch = "x86")]
pub const PTE_TOP: usize = PTE_BASE + (((1u64 << 32) / PAGE_SIZE as u64) as usize * 4) - 1;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const PTE_TOP: usize = 0xFFFFF6FFFFFFFFFF;

// ============================================================================
// PPE/PXE address translation (64-bit only)
// ============================================================================

/// MiAddressToPpe — PPE pointer from VA (4-level paging only)
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiAddressToPpe(va: PVOID) -> PMMPTE {
    let addr = va as u64;
    let offset = (addr >> (30 - 3)) & (0x3FFFFu64 << 3);
    (PPE_BASE as u64 + offset) as PMMPTE
}

/// MiAddressToPxe — PXE pointer from VA (4-level paging only)
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiAddressToPxe(va: PVOID) -> PMMPTE {
    let addr = va as u64;
    let offset = (addr >> (39 - 3)) & (0x1FFu64 << 3);
    (PXE_BASE as u64 + offset) as PMMPTE
}

/// MiPpeToAddress — VA from PPE pointer (signed shift for sign extension)
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiPpeToAddress(ppe: PMMPTE) -> PVOID {
    (((ppe as i64) << 43) >> 16) as PVOID
}

/// MiPxeToAddress — VA from PXE pointer (signed shift for sign extension)
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiPxeToAddress(pxe: PMMPTE) -> PVOID {
    (((pxe as i64) << 52) >> 16) as PVOID
}

/// MiPpeToPte — PPE to its lowest PTE
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiPpeToPte(ppe: PMMPTE) -> PMMPTE {
    MiPdeToAddress(ppe) as PMMPTE
}

/// MiPxeToPte — PXE to its lowest PTE
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiPxeToPte(pxe: PMMPTE) -> PMMPTE {
    MiPpeToAddress(pxe) as PMMPTE
}

/// MiPteToPpe — PTE to its corresponding PPE
///
/// Two recursion steps: PPE-of-X = PDE-of-(PTE-of-X) = `MiAddressToPde(pte)`.
/// Arch-independent on 4-level self-map (the old aarch64 branch was one level
/// too high — see MiPteToPde above / field-manual §9.16).
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiPteToPpe(pte: PMMPTE) -> PMMPTE {
    MiAddressToPde(pte as PVOID)
}

/// MiPteToPxe — PTE to its corresponding PXE
///
/// Three recursion steps: PXE-of-X = PPE-of-(PTE-of-X) = `MiAddressToPpe(pte)`.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiPteToPxe(pte: PMMPTE) -> PMMPTE {
    MiAddressToPpe(pte as PVOID)
}

/// MiPdeToPpe — PDE to its corresponding PPE
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiPdeToPpe(pde: PMMPDE) -> PMMPTE {
    MiAddressToPte(pde as PVOID)
}

/// MiPdeToPxe — PDE to its corresponding PXE
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiPdeToPxe(pde: PMMPDE) -> PMMPTE {
    MiAddressToPde(pde as PVOID)
}

// P*E boundary checks
#[inline]
pub fn MiIsPteOnPdeBoundary(pte: PMMPTE) -> bool {
    ((pte as usize) & (PAGE_SIZE - 1)) == 0
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiIsPteOnPpeBoundary(pte: PMMPTE) -> bool {
    ((pte as usize) & (PDE_PER_PAGE * PAGE_SIZE - 1)) == 0
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[inline]
pub fn MiIsPteOnPxeBoundary(pte: PMMPTE) -> bool {
    ((pte as usize) & (PPE_PER_PAGE * PDE_PER_PAGE * PAGE_SIZE - 1)) == 0
}

// ============================================================================
// MI_SET_PFN_DELETED / MI_IS_PFN_DELETED
// ============================================================================

/// Mark PFN as deleted by OR-ing bit 0 into PteAddress.
#[inline]
pub unsafe fn MI_SET_PFN_DELETED(pfn: PMMPFN) {
    let p = (pfn as *mut u8).add(pfn_offsets::PTE_ADDRESS);
    let addr = *(p as *mut usize);
    *(p as *mut usize) = addr | 1;
}

/// Check if PFN is marked deleted (bit 0 of PteAddress).
#[inline]
pub unsafe fn MI_IS_PFN_DELETED(pfn: PMMPFN) -> bool {
    let p = (pfn as *const u8).add(pfn_offsets::PTE_ADDRESS);
    let addr = *(p as *const usize);
    (addr & 1) != 0
}

// ============================================================================
// MM_WAIT_ENTRY
// ============================================================================

#[cfg(target_arch = "x86")]
pub const MM_WAIT_ENTRY: ULONG_PTR = 0x7ffffc00;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const MM_WAIT_ENTRY: ULONG_PTR = 0x7FFFFFFFFFFFFC00;

// ============================================================================
// MI_PTE_LOOKUP_NEEDED
// ============================================================================

#[cfg(target_pointer_width = "64")]
pub const MI_PTE_LOOKUP_NEEDED: ULONG_PTR = 0xFFFFFFFF;
#[cfg(target_pointer_width = "32")]
pub const MI_PTE_LOOKUP_NEEDED: ULONG_PTR = 0xFFFFF;

// ============================================================================
// Memory consumer IDs (from mm.h)
// ============================================================================

pub const MC_USER: ULONG = 0;
pub const MC_SYSTEM: ULONG = 1;
pub const MC_MAXIMUM: ULONG = 2;

// ============================================================================
// Page location values (MMLISTS enum)
// ============================================================================

pub const ZeroedPageList: u16 = 0;
pub const FreePageList: u16 = 1;
pub const StandbyPageList: u16 = 2;
pub const ModifiedPageList: u16 = 3;
pub const ModifiedNoWritePageList: u16 = 4;
pub const BadPageList: u16 = 5;
// ActiveAndValid = 6 already defined above
pub const TransitionPage: u16 = 7;

// ============================================================================
// MMPFNLIST — PFN list head structure
// ============================================================================

pub type MMLISTS = ULONG;

#[repr(C)]
pub struct MMPFNLIST {
    pub Total: PFN_NUMBER,
    pub ListName: MMLISTS,
    pub Flink: PFN_NUMBER,
    pub Blink: PFN_NUMBER,
}

pub type PMMPFNLIST = *mut MMPFNLIST;

// ============================================================================
// MM_MEMORY_CONSUMER — page consumer balancing
// ============================================================================

/// Trim callback function type.
/// The C declaration uses no calling convention (cdecl on i386):
///   NTSTATUS (*Trim)(ULONG Target, ULONG Priority, PULONG NrFreed)
pub type PBALANCE_TRIM_ROUTINE = Option<
    unsafe extern "C" fn(Target: ULONG, Priority: ULONG, NrFreed: *mut ULONG) -> NTSTATUS,
>;

#[repr(C)]
pub struct MM_MEMORY_CONSUMER {
    pub PagesUsed: ULONG,
    pub PagesTarget: ULONG,
    pub Trim: PBALANCE_TRIM_ROUTINE,
}

pub type PMM_MEMORY_CONSUMER = *mut MM_MEMORY_CONSUMER;

// ============================================================================
// MM_REGION — memory region descriptor
// ============================================================================

#[repr(C)]
pub struct MM_REGION {
    pub Type: ULONG,
    pub Protect: ULONG,
    pub Length: SIZE_T,
    pub RegionListEntry: LIST_ENTRY,
}

pub type PMM_REGION = *mut MM_REGION;

// ============================================================================
// PHYSICAL_MEMORY_RUN / PHYSICAL_MEMORY_DESCRIPTOR
// ============================================================================

#[repr(C)]
pub struct PHYSICAL_MEMORY_RUN {
    pub BasePage: PFN_NUMBER,
    pub PageCount: PFN_NUMBER,
}

#[repr(C)]
pub struct PHYSICAL_MEMORY_DESCRIPTOR {
    pub NumberOfRuns: ULONG,
    pub NumberOfPages: PFN_NUMBER,
    pub Run: [PHYSICAL_MEMORY_RUN; 1],
}

pub type PPHYSICAL_MEMORY_DESCRIPTOR = *mut PHYSICAL_MEMORY_DESCRIPTOR;

// ============================================================================
// MMCOLOR_TABLES — page coloring
// ============================================================================

#[repr(C)]
pub struct MMCOLOR_TABLES {
    pub Flink: PFN_NUMBER,
    pub Blink: PVOID,
    pub Count: PFN_NUMBER,
}

pub type PMMCOLOR_TABLES = *mut MMCOLOR_TABLES;

// ============================================================================
// MM protection constants (from miarm.h)
// ============================================================================

pub const MM_ZERO_ACCESS: ULONG = 0;
pub const MM_READONLY: ULONG = 1;
pub const MM_EXECUTE: ULONG = 2;
pub const MM_EXECUTE_READ: ULONG = 3;
// MM_READWRITE = 4 already defined above
pub const MM_WRITECOPY: ULONG = 5;
pub const MM_EXECUTE_READWRITE: ULONG = 6;
pub const MM_EXECUTE_WRITECOPY: ULONG = 7;
pub const MM_PROTECT_ACCESS: ULONG = 7;

pub const MM_NOCACHE: ULONG = 0x08;
pub const MM_GUARDPAGE: ULONG = 0x10;
pub const MM_WRITECOMBINE: ULONG = 0x18;
pub const MM_PROTECT_SPECIAL: ULONG = 0x18;

pub const MM_DECOMMIT: ULONG = MM_ZERO_ACCESS | MM_GUARDPAGE;
pub const MM_NOACCESS: ULONG = MM_ZERO_ACCESS | MM_WRITECOMBINE;
pub const MM_OUTSWAPPED_KSTACK: ULONG = MM_EXECUTE_WRITECOPY | MM_WRITECOMBINE;
pub const MM_INVALID_PROTECTION: ULONG = 0xFFFFFFFF;

// ============================================================================
// Memory area types (from mm.h)
// ============================================================================

pub const MEMORY_AREA_SECTION_VIEW: ULONG = 1;
pub const MEMORY_AREA_OWNED_BY_ARM3: ULONG = 15;
pub const MEMORY_AREA_STATIC: ULONG = 0x80000000;

// ============================================================================
// Misc MM constants
// ============================================================================

pub const STATUS_MM_RESTART_OPERATION: NTSTATUS = 0xD0000001u32 as NTSTATUS;
pub const MM_NOIRQL: KIRQL = 0xFF;
pub const MM_VIRTMEM_GRANULARITY: usize = 64 * 1024;
pub const MAX_PAGING_FILES: usize = 16;

pub const LIST_HEAD_SENTINEL: ULONG_PTR = ULONG_PTR::MAX;

/// COLORED_LIST_HEAD — sentinel for colored page lists
#[cfg(target_pointer_width = "64")]
pub const COLORED_LIST_HEAD: PFN_NUMBER = (1usize << 57) - 1;
#[cfg(target_pointer_width = "32")]
pub const COLORED_LIST_HEAD: PFN_NUMBER = (1usize << 25) - 1;

// Special LoadedImports values (from miarm.h)
pub const MM_SYSLDR_NO_IMPORTS: PVOID = (-2isize) as PVOID;
pub const MM_SYSLDR_BOOT_LOADED: PVOID = (-1isize) as PVOID;
pub const MM_SYSLDR_SINGLE_ENTRY: ULONG_PTR = 0x1;

// MI_INITIAL_SESSION_IDS
pub const MI_INITIAL_SESSION_IDS: ULONG = 64;

// MmDbgCopyMemory flags
pub const MMDBG_COPY_WRITE: ULONG = 0x01;
pub const MMDBG_COPY_PHYSICAL: ULONG = 0x02;
pub const MMDBG_COPY_UNSAFE: ULONG = 0x04;
pub const MMDBG_COPY_CACHED: ULONG = 0x08;
pub const MMDBG_COPY_UNCACHED: ULONG = 0x10;
pub const MMDBG_COPY_WRITE_COMBINED: ULONG = 0x20;
pub const MMDBG_COPY_MAX_SIZE: usize = 0x8;

// MiCheckSecuredVad constants
pub const MM_READ_WRITE_ALLOWED: ULONG = 11;
pub const MM_READ_ONLY_ALLOWED: ULONG = 10;
pub const MM_NO_ACCESS_ALLOWED: ULONG = 1;
pub const MM_DELETE_CHECK: ULONG = 85;

// Session data page limits
pub const MI_SESSION_DATA_PAGES_MAXIMUM: usize = MM_VIRTMEM_GRANULARITY / PAGE_SIZE;
pub const MI_SESSION_TAG_PAGES_MAXIMUM: usize = MM_VIRTMEM_GRANULARITY / PAGE_SIZE;

// System view bucket
pub const MI_SYSTEM_VIEW_BUCKET_SIZE: usize = 64 * 1024;

// ============================================================================
// PTE flags (arch-specific, from miarm.h)
// ============================================================================

#[cfg(target_arch = "x86")]
pub const PTE_VALID: ULONG_PTR = 0x1;
#[cfg(target_arch = "x86")]
pub const PTE_ACCESSED: ULONG_PTR = 0x20;
#[cfg(target_arch = "x86")]
pub const PTE_DIRTY: ULONG_PTR = 0x40;

#[cfg(target_arch = "x86_64")]
pub const PTE_VALID: ULONG_PTR = 0x1;
#[cfg(target_arch = "x86_64")]
pub const PTE_ACCESSED: ULONG_PTR = 0x20;
#[cfg(target_arch = "x86_64")]
pub const PTE_DIRTY: ULONG_PTR = 0x40;

#[cfg(target_arch = "aarch64")]
pub const PTE_VALID: ULONG_PTR = 0x1;
#[cfg(target_arch = "aarch64")]
pub const PTE_ACCESSED: ULONG_PTR = 0x400;
#[cfg(target_arch = "aarch64")]
pub const PTE_DIRTY: ULONG_PTR = 0x80;

// ============================================================================
// MI_PFN_USAGES enum
// ============================================================================

pub const MI_USAGE_NOT_SET: ULONG = 0;
pub const MI_USAGE_PAGED_POOL: ULONG = 1;
pub const MI_USAGE_NONPAGED_POOL: ULONG = 2;
pub const MI_USAGE_NONPAGED_POOL_EXPANSION: ULONG = 3;
pub const MI_USAGE_KERNEL_STACK: ULONG = 4;
pub const MI_USAGE_KERNEL_STACK_EXPANSION: ULONG = 5;
pub const MI_USAGE_SYSTEM_PTE: ULONG = 6;
pub const MI_USAGE_VAD: ULONG = 7;
pub const MI_USAGE_PEB_TEB: ULONG = 8;
pub const MI_USAGE_SECTION: ULONG = 9;
pub const MI_USAGE_PAGE_TABLE: ULONG = 10;
pub const MI_USAGE_PAGE_DIRECTORY: ULONG = 11;
pub const MI_USAGE_LEGACY_PAGE_DIRECTORY: ULONG = 12;
pub const MI_USAGE_DRIVER_PAGE: ULONG = 13;
pub const MI_USAGE_CONTINOUS_ALLOCATION: ULONG = 14;
pub const MI_USAGE_MDL: ULONG = 15;
pub const MI_USAGE_DEMAND_ZERO: ULONG = 16;
pub const MI_USAGE_ZERO_LOOP: ULONG = 17;
pub const MI_USAGE_CACHE: ULONG = 18;
pub const MI_USAGE_PFN_DATABASE: ULONG = 19;
pub const MI_USAGE_BOOT_DRIVER: ULONG = 20;
pub const MI_USAGE_INIT_MEMORY: ULONG = 21;
pub const MI_USAGE_PAGE_FILE: ULONG = 22;
pub const MI_USAGE_COW: ULONG = 23;
pub const MI_USAGE_WSLE: ULONG = 24;
pub const MI_USAGE_FREE_PAGE: ULONG = 25;

// ============================================================================
// Additional extern globals needed by MM subsystem
// ============================================================================

extern "C" {
    // PFN lists
    pub static mut MmZeroedPageListHead: MMPFNLIST;
    pub static mut MmFreePageListHead: MMPFNLIST;
    pub static mut MmStandbyPageListHead: MMPFNLIST;
    pub static mut MmModifiedPageListHead: MMPFNLIST;
    pub static mut MmModifiedNoWritePageListHead: MMPFNLIST;

    // Physical memory descriptor
    pub static mut MmPhysicalMemoryBlock: PPHYSICAL_MEMORY_DESCRIPTOR;

    // Memory consumers
    pub static mut MiMemoryConsumers: [MM_MEMORY_CONSUMER; MC_MAXIMUM as usize];

    // Session space boundaries
    pub static mut MmSessionBase: PVOID;
    pub static mut MiSessionSpaceEnd: PVOID;
    pub static mut MiSessionBasePte: PMMPTE;
    pub static mut MiSessionLastPte: PMMPTE;
    pub static mut MiSessionImageStart: PVOID;
    pub static mut MiSessionImageEnd: PVOID;

    // Working set limits
    pub static mut MmMinimumWorkingSetSize: SIZE_T;
    pub static mut MmMaximumWorkingSetSize: SIZE_T;

    // Expansion lock
    pub static mut MmExpansionLock: KSPIN_LOCK;
    pub static mut MiExpansionLockOwner: PVOID; // PETHREAD

    // Commit tracking
    pub static mut MmTotalCommitLimit: SIZE_T;
    pub static mut MmTotalCommittedPages: SIZE_T;
    pub static mut MmSharedCommit: SIZE_T;
    pub static mut MmDriverCommit: SIZE_T;
    pub static mut MmProcessCommit: SIZE_T;
    pub static mut MmPagedPoolCommit: SIZE_T;
    pub static mut MmPeakCommitment: SIZE_T;
    pub static mut MmtotalCommitLimitMaximum: SIZE_T;

    // Paging
    pub static mut MmNumberOfPagingFiles: ULONG;

    // Debug mapping
    pub static mut MiDebugMapping: PVOID;
    pub static mut MmDebugPte: PMMPTE;

    // Verifier
    pub static mut MmVerifierData: [u8; 256]; // opaque, MM_DRIVER_VERIFIER_DATA
    pub static mut KernelVerifier: PVOID;
    pub static mut MmTriageActionTaken: PVOID;
    pub static mut MmUnloadedDrivers: PVOID;
    pub static mut MmLastUnloadedDrivers: PVOID;

    // Misc
    pub static mut MmDisablePagingExecutive: UCHAR;
    pub static mut MmLowestPhysicalPage: PFN_NUMBER;
    pub static mut MmThrottleTop: ULONG;
    pub static mut MmThrottleBottom: ULONG;
    // MmTotalNonPagedPoolQuota/MmTotalPagedPoolQuota — owned by reactos-pool globals
    pub static mut MmLoadedUserImageList: LIST_ENTRY;
    pub static mut MmSystemLoadLock: [u8; 64]; // opaque KMUTANT

    // Highest user PTE (used by HW PTE creation)
    pub static mut MiHighestUserPte: PMMPTE;

    // Protection to PTE mask table
    pub static MmProtectToPteMask: [ULONG_PTR; 32];
    pub static MmProtectToValue: [ULONG; 32];
}

// ============================================================================
// Additional extern MM functions
// ============================================================================

extern "system" {
    /// Insert page into a specific PFN list
    pub fn MiInsertPageInList(ListHead: PMMPFNLIST, PageFrameIndex: PFN_NUMBER);

    /// Unlink a page from free or zeroed list
    pub fn MiUnlinkFreeOrZeroedPage(Pfn: PMMPFN);

    /// Map pages in zero space (for zero page thread)
    pub fn MiMapPagesInZeroSpace(Pfn1: PMMPFN, NumberOfPages: PFN_NUMBER) -> PVOID;

    /// Unmap pages from zero space
    pub fn MiUnmapPagesInZeroSpace(VirtualAddress: PVOID, NumberOfPages: PFN_NUMBER);

    /// Zero a physical page
    pub fn MiZeroPhysicalPage(PageFrameIndex: PFN_NUMBER);

    /// Remove a zeroed page
    pub fn MiRemoveZeroPage(Color: ULONG) -> PFN_NUMBER;

    /// Find/free initialization code
    pub fn MiFindInitializationCode(StartVa: *mut PVOID, EndVa: *mut PVOID);
    pub fn MiFreeInitializationCode(StartVa: PVOID, EndVa: PVOID);

}

extern "system" {
    /// Lookup data table entry by address.
    /// Defined as NTAPI in miarm.h/sysldr.c — stdcall on i386.
    pub fn MiLookupDataTableEntry(Address: PVOID) -> PVOID; // PLDR_DATA_TABLE_ENTRY
}

// ============================================================================
// Session range check helpers
// ============================================================================

/// MI_IS_SESSION_ADDRESS — check if VA is in session space
#[inline]
pub unsafe fn MI_IS_SESSION_ADDRESS(va: PVOID) -> bool {
    let addr = va as ULONG_PTR;
    addr >= MmSessionBase as ULONG_PTR && addr < MiSessionSpaceEnd as ULONG_PTR
}

/// MI_IS_SESSION_PTE — check if PTE is in session PTE range
#[inline]
pub unsafe fn MI_IS_SESSION_PTE(pte: PMMPTE) -> bool {
    pte >= MiSessionBasePte && pte < MiSessionLastPte
}

/// MI_IS_PAGE_TABLE_ADDRESS — check if address is in page table self-map
#[inline]
pub fn MI_IS_PAGE_TABLE_ADDRESS(va: PVOID) -> bool {
    let addr = va as ULONG_PTR;
    addr >= PTE_BASE && addr <= PTE_TOP
}

// ============================================================================
// Additional MM constants needed by translated MM files
// ============================================================================

// Bugcheck codes (from bugcodes.mc)
pub const MEMORY_MANAGEMENT: ULONG = 0x1A;
pub const NO_PAGES_AVAILABLE: ULONG = 0x4D;
pub const PFN_LIST_CORRUPT: ULONG = 0x4E;

// NTSTATUS codes
pub const STATUS_WAIT_0: NTSTATUS = 0x00000000;
pub const STATUS_WAIT_1: NTSTATUS = 0x00000001;
pub const STATUS_WORKING_SET_LIMIT_RANGE: NTSTATUS = 0x40000002u32 as NTSTATUS;
pub const STATUS_BAD_WORKING_SET_LIMIT: NTSTATUS = 0xC000004Cu32 as NTSTATUS;

// Pool tags
pub const TAG_MDL: ULONG = u32::from_le_bytes(*b" ldM");
pub const TAG_MM: ULONG = u32::from_le_bytes(*b"  mM");

// MDL flags
pub const MDL_PAGES_LOCKED: ULONG = 0x0002;

// MM constants
pub const MM_DATAFILE_SEGMENT: ULONG = 0x2;
pub const MI_ZERO_PTES: ULONG = 32;

// MI_DEBUG_MAPPING — architecture-dependent debug mapping address
#[cfg(target_arch = "x86")]
pub const MI_DEBUG_MAPPING: ULONG_PTR = 0xFFBFF000;
#[cfg(target_arch = "x86_64")]
pub const MI_DEBUG_MAPPING: ULONG_PTR = 0xFFFFF89FFFFFF000;
#[cfg(target_arch = "aarch64")]
pub const MI_DEBUG_MAPPING: ULONG_PTR = 0xFFFFFB0000000000;

// ULONG64 type alias (Windows type, maps to u64)
pub type ULONG64 = u64;

// HANDLE type alias
pub type HANDLE = PVOID;

// SSIZE_T type alias (signed size)
pub type SSIZE_T = isize;

// MI_PFN_CACHE_ATTRIBUTE enum values
pub type MI_PFN_CACHE_ATTRIBUTE = u32;
pub const MiNonCached: MI_PFN_CACHE_ATTRIBUTE = 0;
// MiCached already defined above as u16 (0x1) — this is the u32 version for the enum
pub const MiCached_u32: MI_PFN_CACHE_ATTRIBUTE = 1;
pub const MiWriteCombined: MI_PFN_CACHE_ATTRIBUTE = 2;
pub const MiNotMapped: MI_PFN_CACHE_ATTRIBUTE = 3;

// MEMORY_CACHING_TYPE enum values
pub type MEMORY_CACHING_TYPE = u32;
pub const MmNonCached: MEMORY_CACHING_TYPE = 0;
pub const MmCached_val: MEMORY_CACHING_TYPE = 1;
pub const MmWriteCombined: MEMORY_CACHING_TYPE = 2;
pub const MmHardwareCoherentCached: MEMORY_CACHING_TYPE = 3;
pub const MmNonCachedUnordered: MEMORY_CACHING_TYPE = 4;
pub const MmUSWCCached: MEMORY_CACHING_TYPE = 5;
pub const MmMaximumCacheType: MEMORY_CACHING_TYPE = 6;

// MM_SYSTEMSIZE enum values
pub type MM_SYSTEMSIZE = u32;
pub const MmSmallSystem: MM_SYSTEMSIZE = 0;
pub const MmMediumSystem: MM_SYSTEMSIZE = 1;
pub const MmLargeSystem: MM_SYSTEMSIZE = 2;

// LIST_HEAD sentinel for PFN lists
pub const LIST_HEAD: PFN_NUMBER = PFN_NUMBER::MAX;

// ============================================================================
// Additional MMPFN field accessors (u1.Flink, u1.SwapEntry, NextLRU, PreviousLRU, RmapListHead)
// ============================================================================

// Derived from bindgen (see pfn_offsets banner). The hand-maintained numeric
// version of THIS table was the 2026-07-05 one-slot-shift bug: NextLRU collided
// with the embedded Wsle, PreviousLRU with NextLRU, and RmapListHead landed on
// PreviousLRU instead of overlaying OriginalPte as the C union intends.
mod pfn_offsets_extra {
    use core::mem::offset_of;
    use crate::ddk::_MMPFN;
    pub const U1: usize = offset_of!(_MMPFN, u1); // Flink/SwapEntry/WsIndex union
    pub const NEXT_LRU: usize = offset_of!(_MMPFN, NextLRU);
    pub const PREVIOUS_LRU: usize = offset_of!(_MMPFN, PreviousLRU);
    // RmapListHead — C union overlay with OriginalPte (bindgen __bindgen_anon_1)
    pub const RMAP_LIST_HEAD: usize = offset_of!(_MMPFN, __bindgen_anon_1);
}

impl MMPFN {
    /// u1.Flink — PFN of next entry in PFN list (overlaps with SwapEntry/WsIndex)
    #[inline]
    pub unsafe fn flink(&self) -> PFN_NUMBER {
        let p = (self as *const _ as *const u8).add(pfn_offsets_extra::U1);
        *(p as *const PFN_NUMBER)
    }

    #[inline]
    pub unsafe fn set_flink(&mut self, val: PFN_NUMBER) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets_extra::U1);
        *(p as *mut PFN_NUMBER) = val;
    }

    /// u1.SwapEntry — swap entry (overlaps with Flink/WsIndex)
    #[inline]
    pub unsafe fn swap_entry(&self) -> ULONG_PTR {
        let p = (self as *const _ as *const u8).add(pfn_offsets_extra::U1);
        *(p as *const ULONG_PTR)
    }

    #[inline]
    pub unsafe fn set_swap_entry(&mut self, val: ULONG_PTR) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets_extra::U1);
        *(p as *mut ULONG_PTR) = val;
    }

    /// u4.AweAllocation bit — just past VerifierAllocation
    #[inline]
    pub unsafe fn awe_allocation(&self) -> bool {
        (self.entire_frame() & (1usize << (pfn_offsets::MI_PTE_FRAME_BITS + 2))) != 0
    }

    #[inline]
    pub unsafe fn set_awe_allocation(&mut self, val: bool) {
        let bit = 1usize << (pfn_offsets::MI_PTE_FRAME_BITS + 2);
        let cur = self.entire_frame();
        if val {
            self.set_entire_frame(cur | bit);
        } else {
            self.set_entire_frame(cur & !bit);
        }
    }

    /// u4.Priority — 3 bits after AweAllocation.
    #[inline]
    pub unsafe fn priority(&self) -> u16 {
        ((self.entire_frame() >> (pfn_offsets::MI_PTE_FRAME_BITS + 3)) & 0x7) as u16
    }

    #[inline]
    pub unsafe fn set_priority(&mut self, val: u16) {
        let shift = pfn_offsets::MI_PTE_FRAME_BITS + 3;
        let mask = 0x7usize << shift;
        let cur = self.entire_frame();
        self.set_entire_frame((cur & !mask) | (((val as usize) & 0x7) << shift));
    }

    /// u4.MustBeCached bit
    #[inline]
    pub unsafe fn must_be_cached(&self) -> bool {
        (self.entire_frame() & (1usize << (pfn_offsets::MI_PTE_FRAME_BITS + 6))) != 0
    }

    #[inline]
    pub unsafe fn set_must_be_cached(&mut self, val: bool) {
        let bit = 1usize << (pfn_offsets::MI_PTE_FRAME_BITS + 6);
        let cur = self.entire_frame();
        if val {
            self.set_entire_frame(cur | bit);
        } else {
            self.set_entire_frame(cur & !bit);
        }
    }

    /// NextLRU — ReactOS-specific field (PMMPFN pointer)
    #[inline]
    pub unsafe fn next_lru(&self) -> PMMPFN {
        let p = (self as *const _ as *const u8).add(pfn_offsets_extra::NEXT_LRU);
        *(p as *const PMMPFN)
    }

    #[inline]
    pub unsafe fn set_next_lru(&mut self, val: PMMPFN) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets_extra::NEXT_LRU);
        *(p as *mut PMMPFN) = val;
    }

    /// PreviousLRU — ReactOS-specific field (PMMPFN pointer)
    #[inline]
    pub unsafe fn previous_lru(&self) -> PMMPFN {
        let p = (self as *const _ as *const u8).add(pfn_offsets_extra::PREVIOUS_LRU);
        *(p as *const PMMPFN)
    }

    #[inline]
    pub unsafe fn set_previous_lru(&mut self, val: PMMPFN) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets_extra::PREVIOUS_LRU);
        *(p as *mut PMMPFN) = val;
    }

    /// u2.Blink — PFN of previous entry in PFN list (overlaps with ShareCount)
    #[inline]
    pub unsafe fn blink(&self) -> PFN_NUMBER {
        self.share_count() as PFN_NUMBER
    }

    /// RmapListHead — ReactOS-specific field (pointer)
    #[inline]
    pub unsafe fn rmap_list_head(&self) -> PVOID {
        let p = (self as *const _ as *const u8).add(pfn_offsets_extra::RMAP_LIST_HEAD);
        *(p as *const PVOID)
    }

    #[inline]
    pub unsafe fn set_rmap_list_head(&mut self, val: PVOID) {
        let p = (self as *mut _ as *mut u8).add(pfn_offsets_extra::RMAP_LIST_HEAD);
        *(p as *mut PVOID) = val;
    }

    /// u3.e1.Modified bit (bit 0 of MMPFNENTRY)
    #[inline]
    pub unsafe fn modified(&self) -> bool {
        (self.mmpfnentry() & 1) != 0
    }

    #[inline]
    pub unsafe fn set_modified(&mut self, val: bool) {
        let mut e = self.mmpfnentry();
        if val { e |= 1; } else { e &= !1; }
        self.set_mmpfnentry(e);
    }

    /// u3.e1.RemovalRequested bit (bit 11)
    #[inline]
    pub unsafe fn removal_requested(&self) -> bool {
        (self.mmpfnentry() & (1 << 11)) != 0
    }

    /// u3.e1.Rom bit (bit 14)
    #[inline]
    pub unsafe fn rom(&self) -> bool {
        (self.mmpfnentry() & (1 << 14)) != 0
    }

    #[inline]
    pub unsafe fn set_rom(&mut self, val: bool) {
        let mut e = self.mmpfnentry();
        if val { e |= 1 << 14; } else { e &= !(1 << 14); }
        self.set_mmpfnentry(e);
    }

    /// u3.e1.ParityError bit (bit 15)
    #[inline]
    pub unsafe fn parity_error(&self) -> bool {
        (self.mmpfnentry() & (1 << 15)) != 0
    }

    /// u3.e2.ReferenceCount — same as reference_count(), alias for C naming
    #[inline]
    pub unsafe fn e2_reference_count(&self) -> USHORT {
        self.reference_count()
    }

    #[inline]
    pub unsafe fn set_e2_reference_count(&mut self, val: USHORT) {
        self.set_reference_count(val);
    }

    /// Cache attribute accessor (read)
    #[inline]
    pub unsafe fn cache_attribute(&self) -> u16 {
        (self.mmpfnentry() >> MMPFNENTRY_CACHE_ATTRIBUTE_SHIFT) & 0x3
    }

    /// u3.e1.PrototypePte — bit 3 of MMPFNENTRY
    #[inline]
    pub unsafe fn prototype_pte(&self) -> bool {
        (self.mmpfnentry() & (1 << 3)) != 0
    }

    #[inline]
    pub unsafe fn set_prototype_pte(&mut self, val: bool) {
        let mut e = self.mmpfnentry();
        if val { e |= 1 << 3; } else { e &= !(1 << 3); }
        self.set_mmpfnentry(e);
    }
}

// ============================================================================
// Additional extern globals needed by MM translated files
// ============================================================================

extern "C" {
    // MmPagingFile array
    pub static mut MmPagingFile: [PVOID; MAX_PAGING_FILES]; // PMMPAGING_FILE[]

    // Product type and system size
    pub static mut MmProductType: ULONG;
    pub static mut MmSystemSize: MM_SYSTEMSIZE;

    // System lock pages count
    pub static mut MmSystemLockPagesCount: SIZE_T;

    // Boot image size
    pub static mut MmBootImageSize: SIZE_T;

    // Cache attributes table
    pub static mut MiPlatformCacheAttributes: [[MI_PFN_CACHE_ATTRIBUTE; 6]; 2]; // [2][MmMaximumCacheType]
}

// ============================================================================
// Helper inline functions needed by MM translated files
// ============================================================================

/// ADDRESS_AND_SIZE_TO_SPAN_PAGES — calculate number of pages spanned by an address+size
#[inline]
pub fn ADDRESS_AND_SIZE_TO_SPAN_PAGES(va: PVOID, size: SIZE_T) -> PFN_NUMBER {
    let va_addr = va as ULONG_PTR;
    ((va_addr & (PAGE_SIZE - 1)) + size + (PAGE_SIZE - 1)) / PAGE_SIZE
}

/// MI_PAGE_DISABLE_CACHE — set cache-disable bit on a PTE
#[inline]
pub fn MI_PAGE_DISABLE_CACHE(pte: &mut MMPTE) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        pte.Long |= 1 << 4; // PCD bit
    }
    #[cfg(target_arch = "aarch64")]
    {
        pte.Long = (pte.Long & !ARM64_PTE_ATTRINDX_MASK) | ARM64_PTE_ATTR_DEVICE;
    }
}

// PTE AttrIndx (bits [4:2]) selects a MAIR_EL1 slot. freeldr programs
// MAIR_EL1 = 0x004400FF (WinLdrSetProcessorContext): slot 0 = Normal write-back,
// slot 1 = Device-nGnRnE, slot 2 = Normal non-cacheable. This helper used to
// write slot 0 under a comment saying "device", so every MmMapIoSpace(MmNonCached)
// on arm64 - the UART, the GIC - was mapped as ordinary CACHED memory. QEMU TCG
// models no caches and never noticed; on a Raspberry Pi 4 the first UART write
// through the Phase-1 remap vanished into the D-cache. The same bit-swap had
// already been fixed in PTE_DISABLE_CACHE (ARM3/helpers.rs) and the protection
// table in ke/arm64/stubs.rs; this was the third copy.
#[cfg(target_arch = "aarch64")]
const ARM64_PTE_ATTRINDX_MASK: ULONG_PTR = 0x7 << 2;
#[cfg(target_arch = "aarch64")]
const ARM64_PTE_ATTR_DEVICE: ULONG_PTR = 1 << 2;
#[cfg(target_arch = "aarch64")]
const ARM64_PTE_ATTR_NORMAL_NC: ULONG_PTR = 2 << 2;

/// MI_PAGE_WRITE_THROUGH — set write-through bit on a PTE
#[inline]
pub fn MI_PAGE_WRITE_THROUGH(pte: &mut MMPTE) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        pte.Long |= 1 << 3; // PWT bit
    }
    #[cfg(target_arch = "aarch64")]
    {
        // ARM64: write-through via MAIR
        let _ = pte; // architecture-specific handling
    }
}

/// MI_PAGE_WRITE_COMBINED — set write-combined caching on a PTE
#[inline]
pub fn MI_PAGE_WRITE_COMBINED(pte: &mut MMPTE) {
    // On x86/x86_64: PCD + PWT
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        pte.Long |= (1 << 4) | (1 << 3); // PCD + PWT
    }
    #[cfg(target_arch = "aarch64")]
    {
        // Normal non-cacheable: write-combining is allowed, and so are unaligned
        // accesses (a framebuffer blit), which Device memory would fault.
        pte.Long = (pte.Long & !ARM64_PTE_ATTRINDX_MASK) | ARM64_PTE_ATTR_NORMAL_NC;
    }
}

/// KeGetCurrentThread — returns pointer to current ETHREAD
///
/// On i386, reads fs:0x124 directly (KPCR.PrcbData.CurrentThread).
/// On amd64, reads gs:0x188 directly (KPCR.CurrentPrcb→CurrentThread).
/// On arm64, calls KeGetCurrentPrcb()→CurrentThread.
/// This avoids the PsGetCurrentThread → KeGetCurrentThread → KeGetCurrentPrcb
/// function call chain which is expensive in trap handlers.
#[inline(always)]
pub unsafe fn KeGetCurrentThread() -> PVOID {
    #[cfg(target_arch = "x86")]
    {
        let result: PVOID;
        core::arch::asm!(
            "mov {}, fs:[0x124]",
            out(reg) result,
            options(nostack, preserves_flags, readonly),
        );
        result
    }
    #[cfg(target_arch = "x86_64")]
    {
        let result: PVOID;
        core::arch::asm!(
            "mov {}, gs:[0x188]",
            out(reg) result,
            options(nostack, preserves_flags, readonly),
        );
        result
    }
    #[cfg(target_arch = "aarch64")]
    {
        use crate::ke::KeGetCurrentPrcb;
        (*KeGetCurrentPrcb()).CurrentThread as PVOID
    }
}

/// MmLockSectionSegment / MmUnlockSectionSegment — ReactOS internal MM functions
extern "system" {
    pub fn MmLockSectionSegment(Segment: PVOID);
    pub fn MmUnlockSectionSegment(Segment: PVOID);
    pub fn MmGetPageEntrySectionSegment(Segment: PVOID, Offset: *const u64) -> ULONG_PTR;
}

/// IS_SWAP_FROM_SSE — check if section segment entry indicates swap
#[inline]
pub fn IS_SWAP_FROM_SSE(entry: ULONG_PTR) -> bool {
    (entry & 0x00000001) != 0
}

/// IS_DIRTY_SSE — check if section segment entry is dirty
#[inline]
pub fn IS_DIRTY_SSE(entry: ULONG_PTR) -> bool {
    (entry & 0x00000008) != 0
}

/// CONTAINING_RECORD macro equivalent
/// Calculates the base address of a struct from a pointer to one of its fields.
#[inline]
pub unsafe fn containing_record<Container, Field>(
    ptr: *const Field,
    offset: usize,
) -> *mut Container {
    (ptr as *mut u8).sub(offset) as *mut Container
}

// MMPTE PTE_ACCESSED bit check helper
impl MMPTE {
    /// Check the Accessed bit
    #[inline]
    pub fn accessed(&self) -> bool {
        (self.Long & PTE_ACCESSED) != 0
    }
}

// ============================================================================
// MM Tier 2 — additional constants, globals, and helpers
// ============================================================================

// Bugcheck codes
pub const SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION: ULONG = 0xC1;

// NTSTATUS codes
pub const STATUS_CONFLICTING_ADDRESSES: NTSTATUS = 0xC0000018u32 as NTSTATUS;
pub const STATUS_INVALID_PAGE_PROTECTION: NTSTATUS = 0xC0000045u32 as NTSTATUS;
pub const STATUS_TOO_MANY_PAGING_FILES: NTSTATUS = 0xC0000097u32 as NTSTATUS;

// Page protection constants (from winnt.h)
pub const PAGE_NOACCESS: ULONG = 0x01;
pub const PAGE_READONLY: ULONG = 0x02;
pub const PAGE_WRITECOPY: ULONG = 0x08;
pub const PAGE_EXECUTE: ULONG = 0x10;
pub const PAGE_EXECUTE_READ: ULONG = 0x20;
pub const PAGE_EXECUTE_READWRITE: ULONG = 0x40;
pub const PAGE_EXECUTE_WRITECOPY: ULONG = 0x80;
pub const PAGE_GUARD: ULONG = 0x100;
pub const PAGE_NOCACHE: ULONG = 0x200;
pub const PAGE_WRITECOMBINE: ULONG = 0x400;

// Memory allocation flags
pub const MEM_TOP_DOWN: ULONG_PTR = 0x100000;

// Pool tags
pub const TAG_RMAP: ULONG = u32::from_le_bytes(*b"pMrM"); // MrMp - rmap tag

// Type alias for system PTE pool type
pub type MMSYSTEM_PTE_POOL_TYPE = ULONG;

// Address space constants — architecture-dependent
#[cfg(target_arch = "x86")]
pub const MM_LOWEST_USER_ADDRESS: ULONG_PTR = 0x10000;
#[cfg(target_arch = "x86")]
pub const MM_HIGHEST_USER_ADDRESS: ULONG_PTR = 0x7FFEFFFF;
#[cfg(target_arch = "x86")]
pub const MM_HIGHEST_VAD_ADDRESS: ULONG_PTR = 0x7FFEFFFF;
#[cfg(target_arch = "x86")]
pub const MI_LOWEST_VAD_ADDRESS: ULONG_PTR = 0x10000;
#[cfg(target_arch = "x86")]
pub const MI_SYSTEM_PTE_BASE: ULONG_PTR = 0xC0200000;
#[cfg(target_arch = "x86")]
pub const MM_SHARED_USER_DATA_VA: ULONG_PTR = 0x7FFE0000;
#[cfg(target_arch = "x86")]
pub const KI_USER_SHARED_DATA: ULONG_PTR = 0xFFDF0000;
#[cfg(target_arch = "x86")]
pub const KSEG0_BASE: ULONG_PTR = 0x80000000;
#[cfg(target_arch = "x86")]
pub const HYPER_SPACE: ULONG_PTR = 0xC0400000;
#[cfg(target_arch = "x86")]
pub const HYPER_SPACE_END: ULONG_PTR = 0xC07FFFFF;

#[cfg(target_arch = "x86_64")]
pub const MM_LOWEST_USER_ADDRESS: ULONG_PTR = 0x10000;
#[cfg(target_arch = "x86_64")]
pub const MM_HIGHEST_USER_ADDRESS: ULONG_PTR = 0x000007FFFFFEFFFF;
#[cfg(target_arch = "x86_64")]
pub const MM_HIGHEST_VAD_ADDRESS: ULONG_PTR = 0x000007FFFFFEFFFF;
#[cfg(target_arch = "x86_64")]
pub const MI_LOWEST_VAD_ADDRESS: ULONG_PTR = 0x10000;
#[cfg(target_arch = "x86_64")]
// On amd64: MiAddressToPte(KSEG0_BASE) = ((0xFFFFF80000000000 >> 9) & 0x7FFFFFFFF8) + PTE_BASE
// = 0xFFFFF6FC00000000  (NOT PTE_BASE — that's for i386/arm64 where MI_SYSTEM_PTE_BASE = MiAddressToPte(NULL))
pub const MI_SYSTEM_PTE_BASE: ULONG_PTR = 0xFFFFF6FC00000000;
#[cfg(target_arch = "x86_64")]
pub const MM_SHARED_USER_DATA_VA: ULONG_PTR = 0x7FFE0000;
#[cfg(target_arch = "x86_64")]
pub const KI_USER_SHARED_DATA: ULONG_PTR = 0xFFFFF78000000000;
#[cfg(target_arch = "x86_64")]
pub const KSEG0_BASE: ULONG_PTR = 0xFFFFF80000000000;
#[cfg(target_arch = "x86_64")]
pub const HYPER_SPACE: ULONG_PTR = 0xFFFFF70000000000;
#[cfg(target_arch = "x86_64")]
pub const HYPER_SPACE_END: ULONG_PTR = 0xFFFFF77FFFFFFFFF;

#[cfg(target_arch = "aarch64")]
pub const MM_LOWEST_USER_ADDRESS: ULONG_PTR = 0x10000;
#[cfg(target_arch = "aarch64")]
pub const MM_HIGHEST_USER_ADDRESS: ULONG_PTR = 0x7FFFFFFEFFFF;
#[cfg(target_arch = "aarch64")]
pub const MM_HIGHEST_VAD_ADDRESS: ULONG_PTR = 0x7FFFFFFEFFFF;
#[cfg(target_arch = "aarch64")]
pub const MI_LOWEST_VAD_ADDRESS: ULONG_PTR = 0x10000;
#[cfg(target_arch = "aarch64")]
pub const MI_SYSTEM_PTE_BASE: ULONG_PTR = PTE_BASE;
#[cfg(target_arch = "aarch64")]
pub const MM_SHARED_USER_DATA_VA: ULONG_PTR = 0x7FFE0000;
#[cfg(target_arch = "aarch64")]
pub const KI_USER_SHARED_DATA: ULONG_PTR = 0xFFFFF78000000000;
#[cfg(target_arch = "aarch64")]
pub const KSEG0_BASE: ULONG_PTR = 0xFFFFF80000000000;
#[cfg(target_arch = "aarch64")]
pub const HYPER_SPACE: ULONG_PTR = 0xFFFFF70000000000;
#[cfg(target_arch = "aarch64")]
pub const HYPER_SPACE_END: ULONG_PTR = 0xFFFFF77FFFFFFFFF;

// LockQueueSystemSpaceLock — KSPIN_LOCK_QUEUE_NUMBER enum value
pub const LockQueueSystemSpaceLock: ULONG = 0x03;

// MM_SEGMENT_INDELETE — mm.h:242 says (0x4). This carried 0x1, which is
// MM_PHYSICALMEMORY_SEGMENT's bit; mm/section.rs shadows it with a correct
// local 0x4, so nothing consumed the wrong value yet.
pub const MM_SEGMENT_INDELETE: ULONG_PTR = 0x4;

// Event type
pub const _EVENT_TYPE_SynchronizationEvent: ULONG = 1;

extern "C" {
    // System range start
    pub static mut MmSystemRangeStart: PVOID;

    // System PTE count
    pub static mut MmNumberOfSystemPtes: ULONG;

    // Special pool tag
    pub static mut MmSpecialPoolTag: ULONG;

    // System cache working set (MMSUPPORT struct, NOT a pointer)
    pub static mut MmSystemCacheWs: crate::ddk::_MMSUPPORT;

    // Non-paged system start
    pub static mut MmNonPagedSystemStart: PVOID;

    // Paged pool in bytes
    pub static mut MmSizeOfPagedPoolInBytes: SIZE_T;

    // System view
    pub static mut MiSystemViewStart: PVOID;
    pub static mut MmSystemViewSize: SIZE_T;

    // PFN allocation
    pub static mut MxPfnAllocation: PFN_NUMBER;

    // Idle process
    pub static mut PsIdleProcess: PVOID; // PEPROCESS

    // Loaded modules resource
    pub static mut PsLoadedModuleResource: PVOID; // PERESOURCE

    // Paging file object type
    pub static mut IoFileObjectType: PVOID; // POBJECT_TYPE

}

// Paged pool info (MM_PAGED_POOL_INFO global)
extern "C" {
    pub static mut MmPagedPoolInfo: MM_PAGED_POOL_INFO;
}

/// ALIGN_UP_BY — align a value up to the nearest multiple of alignment
#[inline]
pub fn ALIGN_UP_BY(addr: ULONG_PTR, align: ULONG_PTR) -> ULONG_PTR {
    (addr + align - 1) & !(align - 1)
}

/// ALIGN_DOWN_BY — align a value down to the nearest multiple of alignment
#[inline]
pub fn ALIGN_DOWN_BY(addr: ULONG_PTR, align: ULONG_PTR) -> ULONG_PTR {
    addr & !(align - 1)
}

/// ROUND_UP — round up to multiple
#[inline]
pub fn ROUND_UP(val: ULONG_PTR, align: ULONG_PTR) -> ULONG_PTR {
    ALIGN_UP_BY(val, align)
}

/// ROUND_DOWN — round down to multiple
#[inline]
pub fn ROUND_DOWN(val: ULONG_PTR, align: ULONG_PTR) -> ULONG_PTR {
    ALIGN_DOWN_BY(val, align)
}

/// PAGE_ROUND_DOWN — round address down to page boundary
#[inline]
pub fn PAGE_ROUND_DOWN(addr: ULONG_PTR) -> ULONG_PTR {
    addr & !(PAGE_SIZE - 1)
}

// PAGE_ALIGN is in base.rs — returns usize. Use `PAGE_ALIGN(x) as PVOID` if you need PVOID.

/// InterlockedIncrementUL — atomic increment on ULONG
#[inline]
pub unsafe fn InterlockedIncrementUL(addend: *mut ULONG) -> ULONG {
    let atomic = &*(addend as *const core::sync::atomic::AtomicU32);
    atomic.fetch_add(1, core::sync::atomic::Ordering::SeqCst).wrapping_add(1)
}

/// InterlockedDecrementUL — atomic decrement on ULONG
#[inline]
pub unsafe fn InterlockedDecrementUL(addend: *mut ULONG) -> ULONG {
    let atomic = &*(addend as *const core::sync::atomic::AtomicU32);
    atomic.fetch_sub(1, core::sync::atomic::Ordering::SeqCst).wrapping_sub(1)
}

/// InterlockedExchangeAddUL — atomic add on ULONG, returns old value
#[inline]
pub unsafe fn InterlockedExchangeAddUL(addend: *mut ULONG, value: ULONG) -> ULONG {
    let atomic = &*(addend as *const core::sync::atomic::AtomicU32);
    atomic.fetch_add(value, core::sync::atomic::Ordering::SeqCst)
}

/// UpdateTotalCommittedPages — atomically add to MmTotalCommittedPages
#[inline]
pub unsafe fn UpdateTotalCommittedPages(delta: isize) {
    if delta >= 0 {
        MmTotalCommittedPages = MmTotalCommittedPages.wrapping_add(delta as SIZE_T);
    } else {
        MmTotalCommittedPages = MmTotalCommittedPages.wrapping_sub((-delta) as SIZE_T);
    }
}

/// MI_IS_NOT_PRESENT_FAULT — check if fault is a not-present fault
#[inline]
pub fn MI_IS_NOT_PRESENT_FAULT(fault_code: ULONG_PTR) -> bool {
    (fault_code & 1) == 0
}

/// MI_IS_INSTRUCTION_FETCH — check if fault is an instruction fetch
/// MI_IS_INSTRUCTION_FETCH — check if page fault was due to instruction fetch (NX/XN violation)
/// C: `#define MI_IS_INSTRUCTION_FETCH(FaultCode) BooleanFlagOn(FaultCode, 0x00000010)`
#[inline]
pub fn MI_IS_INSTRUCTION_FETCH(fault_code: ULONG_PTR) -> bool {
    (fault_code & 0x10) != 0
}

/// MM_IS_WAIT_PTE — check if a section-segment entry (SSE) is a wait entry.
/// C macro (ntoskrnl/include/internal/mm.h):
///   `(IS_SWAP_FROM_SSE(E) && SWAPENTRY_FROM_SSE(E) == MM_WAIT_ENTRY)`
///   = `((E) & 0x1) && ((E) >> 1) == MM_WAIT_ENTRY`
/// A wait SSE is written as `MAKE_SWAP_SSE(MM_WAIT_ENTRY) = (MM_WAIT_ENTRY << 1) | 1`,
/// NOT `MM_WAIT_ENTRY` raw — the prior `pte_long == MM_WAIT_ENTRY` never matched a
/// real wait SSE, so MmMakeSegmentResident's collided-fault wait loops never
/// recognized an in-flight page-in → section page-in livelock (arm64 fastfat).
#[inline]
pub fn MM_IS_WAIT_PTE(pte_long: ULONG_PTR) -> bool {
    (pte_long & 0x1) != 0 && (pte_long >> 1) == MM_WAIT_ENTRY
}

/// MI_MAKE_HARDWARE_PTE_KERNEL — create a valid kernel PTE
#[inline]
pub unsafe fn MI_MAKE_HARDWARE_PTE_KERNEL(
    pte: &mut MMPTE,
    _pointer_pte: PMMPTE,
    protection: ULONG,
    pfn: PFN_NUMBER,
) {
    *pte = ValidKernelPte;
    pte.Long &= !(0x1F << 5); // Clear protection bits
    pte.Long |= MmProtectToPteMask[protection as usize];
    pte.set_page_frame_number(pfn);
}

/// MI_IS_ROSMM_VAD — check if a VAD is a ReactOS MM VAD (memory area)
/// C: `#define MI_IS_ROSMM_VAD(Vad) (((Vad)->u.VadFlags.Spare & 2) != 0)`
#[inline]
pub unsafe fn MI_IS_ROSMM_VAD(vad: PVOID) -> bool {
    let vad = vad as *const crate::ddk::_MMVAD;
    ((*vad).u.VadFlags.Spare() & 2) != 0
}

/// RMAP_SEGMENT_MASK — `~((ULONG_PTR)0xff)` on every arch
/// (ntoskrnl/include/internal/mm.h). The low 8 bits of a segment rmap
/// `Address` hold the page index within the section page table, so the mask
/// must clear exactly those 8 bits — it is NOT pointer-width dependent, and it
/// is NOT a low-bit flag.
pub const RMAP_SEGMENT_MASK: ULONG_PTR = !0xffusize;

/// RMAP_IS_SEGMENT — `(((ULONG_PTR)Address & RMAP_SEGMENT_MASK) == RMAP_SEGMENT_MASK)`.
///
/// This used to test `addr & 1`, which is a different predicate entirely: a
/// segment address is `RMAP_SEGMENT_MASK | page_index`, so `& 1` answers
/// "is the page index odd?" and misses every even-indexed segment entry.
/// balance.rs had that test inlined and so walked segment rmap entries as if
/// they were process mappings — treating the page-table pointer parked in
/// `Entry->Process` as an EPROCESS. See the field manual.
#[inline]
pub fn RMAP_IS_SEGMENT(addr: PVOID) -> bool {
    (addr as ULONG_PTR & RMAP_SEGMENT_MASK) == RMAP_SEGMENT_MASK
}

/// PFN_FROM_SSE — extract PFN from section segment entry
#[inline]
pub fn PFN_FROM_SSE(entry: ULONG_PTR) -> PFN_NUMBER {
    entry >> PAGE_SHIFT
}

/// MA_GetStartingAddress — get the starting address of a memory area
/// This is a ReactOS-internal function
#[inline]
pub unsafe fn MA_GetStartingAddress(_marea: PVOID) -> ULONG_PTR {
    // Memory area structure: offset 0 is MMVAD_NODE, starting address at specific offset
    // For now, use extern declaration
    extern "system" {
        fn MmLocateMemoryAreaByAddress(
            AddressSpace: PVOID,
            Address: PVOID,
        ) -> PVOID;
    }
    0 // stub
}

// ============================================================================
// Compile-time semantic assertions (size-vs-C assertions in size_check_*.rs)
// ============================================================================

const _: () = {
    // POOL_HEADER must be exactly POOL_BLOCK_SIZE (pool allocator invariant)
    assert!(core::mem::size_of::<POOL_HEADER>() == POOL_BLOCK_SIZE);
    // POOL_BLOCK_SIZE must equal LIST_ENTRY size (free list overlay invariant)
    assert!(POOL_BLOCK_SIZE == core::mem::size_of::<LIST_ENTRY>());
};
