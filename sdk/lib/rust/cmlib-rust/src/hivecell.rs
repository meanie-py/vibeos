//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/cmlib/hivecell.c.
//! Risk: HIGH. Behavior: 4 real bugs found.
//! Findings:
//!   BUG1 (CRITICAL) HvAllocateCell:131 — HCELL_TYPE_SHIFT=30 but hivedata.h defines it as 31;
//!     `FreeCellOffset |= Storage << 30` sets bit 30 (0x40000000) instead of bit 31 (0x80000000),
//!     misencoding every newly-added volatile cell's type bit → corrupt HCELL_INDEX on hive extension.
//!   BUG2 (HIGH) HvAllocateCell/HvReallocateCell/HvTrackCellRef/HvReleaseFreeCellRefArray:
//!     exported as `extern "C"` (cdecl) but CMAPI=NTAPI=__stdcall; on i386 callers use stdcall
//!     calling convention → stack corruption / mismatched argument pop on every call from C.
//!     Must be `pub unsafe extern "system"`.
//!   BUG3 (LOW) HvTrackCellRef:264 — CmpFree(ptr, TAG_CM) passes TAG_CM=0x434D as Quota arg;
//!     C passes 0 (both CmpFree call sites in hivecell.c use 0). TAG_CM value itself is also
//!     wrong (0x434D vs correct 0x2020_4D43 in glue.rs); import from glue::TAG_CM instead.
//!   BUG4 (cosmetic) HvAllocateCell:91 — local const CMLIB_HCELL_DEBUG=1 shadows module-level
//!     CMLIB_HCELL_DEBUG=0; C has a single #define CMLIB_HCELL_DEBUG 0x01; net effect is that
//!     trace calls inside HvAllocateCell fire (level=1) while HvpRemoveFree traces use level=0
//!     (suppressed). Inconsistent but non-crashing.
//! Unverified: HvpRemoveFree is Rust-dead (only called by C HvFreeCell which is not translated);
//!   7 of 9 CMAPI functions in hivecell.c remain C (HvIsCellAllocated, HvpGetCellData,
//!   HvGetCellSize, HvMarkCellDirty, HvIsCellDirty, HvpCreateHiveFreeCellList, HvFreeCell).

use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CELL_DATA, CmpAllocate, CmpFree, CmpLazyFlush, HBIN, HCELL, HCELL_INDEX, HHIVE, HSTORAGE_TYPE, HV_HIVE_CELL_PAIR, PHV_TRACK_CELL_REF, HV_TRACK_CELL_REF, RtlSetBits, HSTORAGE_TYPE_Stable as Stable, HSTORAGE_TYPE_Volatile as Volatile, _HHIVE};
use crate::glue::{HvGetCell, HvReleaseCell, HvGetCellType, HvGetCellBlock, HCELL_NIL, HBLOCK_SIZE};
use core::ffi::c_void;
use crate::hivelock::{HvpLockHiveExclusive, HvpLockHiveShared};

// Pointer-width integer matching ULONG_PTR/SIZE_T on all arches
#[cfg(target_pointer_width = "32")]
type PtrInt = u32;
#[cfg(target_pointer_width = "64")]
type PtrInt = u64;

unsafe extern "system" {
    // CMAPI/NTAPI (stdcall on i386). HvpAddBin lives in this crate's hivebin.rs
    // (`_HvpAddBin@12`).
    fn HvpAddBin(RegistryHive: *mut HHIVE, Size: u32, Storage: HSTORAGE_TYPE) -> *mut HBIN;
}
unsafe extern "C" {
    fn DbgPrint(format: *const u8, ...);
}

// RtlCheckBit is a FORCEINLINE in the SDK headers (no exported symbol), so we
// inline the bit test rather than calling it: test BitNumber in an RTL_BITMAP.
#[inline]
unsafe fn rtl_check_bit(bitmap: *const ntkrnl_sys::ddk::RTL_BITMAP, bit_number: u32) -> bool {
    let word = *(*bitmap).Buffer.add((bit_number >> 5) as usize);
    (word >> (bit_number & 0x1f)) & 1 != 0
}

// hivedata.h cell-index field masks.
const HCELL_OFFSET_MASK: u32 = 0x0000_0fff;

const CMLIB_HCELL_DEBUG: u32 = 0;

// ── HvpGetCellHeader (static __inline) — resolve an HCELL_INDEX to its PHCELL ──
unsafe fn HvpGetCellHeader(RegistryHive: *mut HHIVE, CellIndex: HCELL_INDEX) -> *mut HCELL {
    ntkrnl_sys::ASSERT!(CellIndex != HCELL_NIL);
    if (*RegistryHive).Flat == 0 {
        let CellType = HvGetCellType(CellIndex);
        let CellBlock = HvGetCellBlock(CellIndex);
        let CellOffset = CellIndex & HCELL_OFFSET_MASK; // HCELL_OFFSET_SHIFT = 0
        // A concurrent HvpAddBin may swap and free BlockList; hold it while indexing.
        let _map = HvpLockHiveShared(RegistryHive);
        let entry = (*RegistryHive).Storage[CellType as usize]
            .BlockList
            .add(CellBlock as usize);
        let Block = (*entry).BlockAddress;
        (Block as usize + CellOffset as usize) as *mut HCELL
    } else {
        ((*RegistryHive).BaseBlock as usize + HBLOCK_SIZE as usize + CellIndex as usize)
            as *mut HCELL
    }
}

// PCELL_DATA = (PHCELL)(HvpGetCellHeader(...) + 1) — header + 1 HCELL.
#[no_mangle]
pub unsafe extern "system" fn HvpGetCellData(
    Hive: *mut HHIVE,
    CellIndex: HCELL_INDEX,
) -> *mut c_void {
    HvpGetCellHeader(Hive, CellIndex).add(1) as *mut c_void
}

// static __inline LONG HvpGetCellFullSize — ((PHCELL)Cell - 1)->Size.
unsafe fn HvpGetCellFullSize(_RegistryHive: *mut HHIVE, Cell: *mut c_void) -> i32 {
    (*(Cell as *mut HCELL).offset(-1)).Size
}

#[no_mangle]
pub unsafe extern "system" fn HvGetCellSize(_Hive: *mut HHIVE, Address: *mut c_void) -> i32 {
    let CellHeader = (Address as *mut HCELL).offset(-1);
    let mut Size = (*CellHeader).Size * -1;
    Size -= core::mem::size_of::<HCELL>() as i32;
    Size
}

#[no_mangle]
pub unsafe extern "system" fn HvIsCellAllocated(
    RegistryHive: *mut HHIVE,
    CellIndex: HCELL_INDEX,
) -> u8 {
    if (*RegistryHive).Flat != 0 {
        return 1;
    }
    let Type = HvGetCellType(CellIndex);
    let Block = HvGetCellBlock(CellIndex);
    if Block >= (*RegistryHive).Storage[Type as usize].Length {
        return 0;
    }
    let _map = HvpLockHiveShared(RegistryHive);
    let entry = (*RegistryHive).Storage[Type as usize].BlockList.add(Block as usize);
    if (*entry).BlockAddress != 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn HvMarkCellDirty(
    RegistryHive: *mut HHIVE,
    CellIndex: HCELL_INDEX,
    _HoldingLock: u8,
) -> u8 {
    ntkrnl_sys::ASSERT!((*RegistryHive).ReadOnly == 0);

    if HvGetCellType(CellIndex) != Stable {
        return 1;
    }

    let CellBlock = HvGetCellBlock(CellIndex);
    let CellLastBlock = HvGetCellBlock(CellIndex + HBLOCK_SIZE - 1);

    // RtlSetBits is a read-modify-write of a vector HvpAddBin may reallocate.
    let _hive = HvpLockHiveExclusive(RegistryHive);
    RtlSetBits(
        core::ptr::addr_of_mut!((*RegistryHive).DirtyVector),
        CellBlock,
        CellLastBlock - CellBlock,
    );
    (*RegistryHive).DirtyCount += 1;

    // C HvMarkCellDirty queues a lazy flush of the dirty data (unless the hive
    // opted out via HIVE_NOLAZYFLUSH), gated `#if !defined(CMLIB_HOST) &&
    // !defined(_BLDR_)`. The `bldr` feature mirrors _BLDR_ (the loader has no
    // lazy-flush worker), so the kernel build calls it, the loader build omits
    // it. CmpLazyFlush is _CmpLazyFlush@0 (ntoskrnl config). hivedata.h:24
    // HIVE_NOLAZYFLUSH = 2.
    #[cfg(not(feature = "bldr"))]
    {
        const HIVE_NOLAZYFLUSH: u32 = 2;
        if ((*RegistryHive).HiveFlags & HIVE_NOLAZYFLUSH) == 0 {
            CmpLazyFlush();
        }
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn HvIsCellDirty(Hive: *mut HHIVE, Cell: HCELL_INDEX) -> u8 {
    ntkrnl_sys::ASSERT!((*Hive).ReadOnly == 0);
    if HvGetCellType(Cell) == Volatile {
        return 1;
    }
    let _hive = HvpLockHiveShared(Hive);
    if rtl_check_bit(
        core::ptr::addr_of!((*Hive).DirtyVector),
        Cell / HBLOCK_SIZE,
    ) {
        1
    } else {
        0
    }
}

// static __inline ULONG HvpComputeFreeListIndex(Size).
unsafe fn HvpComputeFreeListIndex(Size: u32) -> u32 {
    const FIND_FIRST_SET: [u8; 128] = [
        0, 0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3,
        4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
        6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
    ];
    ntkrnl_sys::ASSERT!(Size >= (1 << 3));
    let mut Index = (Size >> 3) - 1;
    if Index >= 16 {
        if Index > 127 {
            Index = 23;
        } else {
            Index = FIND_FIRST_SET[Index as usize] as u32 + 16;
        }
    }
    Index
}

// static NTSTATUS HvpAddFree — push a free block onto its free-list bucket.
unsafe fn HvpAddFree(
    RegistryHive: *mut HHIVE,
    FreeBlock: *mut HCELL,
    FreeIndex: HCELL_INDEX,
) -> i32 {
    let Storage = HvGetCellType(FreeIndex);
    let Index = HvpComputeFreeListIndex((*FreeBlock).Size as u32);

    let FreeBlockData = FreeBlock.add(1) as *mut HCELL_INDEX;
    *FreeBlockData = (*RegistryHive).Storage[Storage as usize].FreeDisplay[Index as usize];
    (*RegistryHive).Storage[Storage as usize].FreeDisplay[Index as usize] = FreeIndex;

    0 // STATUS_SUCCESS
}

// static HCELL_INDEX HvpFindFree — first-fit search of the free-list buckets.
unsafe fn HvpFindFree(RegistryHive: *mut HHIVE, Size: u32, Storage: HSTORAGE_TYPE) -> u32 {
    let mut Index = HvpComputeFreeListIndex(Size);
    while Index < 24 {
        let mut pFreeCellOffset =
            core::ptr::addr_of_mut!((*RegistryHive).Storage[Storage as usize].FreeDisplay[Index as usize]);
        while *pFreeCellOffset != HCELL_NIL {
            let FreeCellData = HvGetCell(RegistryHive, *pFreeCellOffset) as *mut HCELL_INDEX;
            if (HvpGetCellFullSize(RegistryHive, FreeCellData as *mut c_void) as u32) >= Size {
                let FreeCellOffset = *pFreeCellOffset;
                *pFreeCellOffset = *FreeCellData;
                return FreeCellOffset;
            }
            pFreeCellOffset = FreeCellData;
        }
        Index += 1;
    }
    HCELL_NIL
}

unsafe extern "C" fn HvpRemoveFree(
    RegistryHive: *mut HHIVE,
    CellBlock: *mut CELL_DATA,
    CellIndex: u32,
) {
    let mut FreeCellData: *mut u32;
    let mut pFreeCellOffset: *mut u32;
    let Storage: u32;
    let Index: u32;

    ntkrnl_sys::ASSERT!((*RegistryHive).ReadOnly == 0);

    Storage = HvGetCellType(CellIndex);
    Index = HvpComputeFreeListIndex((*(CellBlock as *mut HCELL)).Size as u32);

    pFreeCellOffset = core::ptr::addr_of_mut!((*RegistryHive).Storage[Storage as usize].FreeDisplay[Index as usize]);

    while *pFreeCellOffset != HCELL_NIL {
        FreeCellData = HvGetCell(RegistryHive, *pFreeCellOffset) as *mut u32;
        if *pFreeCellOffset == CellIndex {
            *pFreeCellOffset = *FreeCellData;
            return;
        }
        pFreeCellOffset = FreeCellData;
    }

    // Something bad happened (the block to free was not on its free list).
    // C dumps the free lists via CMLTRACE then ASSERT(FALSE); the trace is
    // NDEBUG-disabled, so we just assert.
    ntkrnl_sys::ASSERT!(false);
}
#[no_mangle]
pub unsafe extern "system" fn HvAllocateCell(
    RegistryHive: *mut _HHIVE,
    mut Size: u32,
    Storage: u32,
    Vicinity: u32,
) -> u32 {
    // NOTE: HCELL_NIL comes from the glue import (0xFFFFFFFF). The earlier
    // translation shadowed it with a local `const HCELL_NIL: u32 = 0;`, which
    // broke the "no free cell found" test against HvpFindFree's real return.
    const HCELL_TYPE_SHIFT: u32 = 31;
    let _ = Vicinity;

    let FreeCell: *mut HCELL;
    let mut FreeCellOffset: u32;
    let NewCell: *mut HCELL;
    let Bin: *mut HBIN;

    // ASSERT(RegistryHive->ReadOnly == FALSE);
    ntkrnl_sys::ASSERT!((*RegistryHive).ReadOnly == 0);

    // Free lists, HvpAddBin's map swap and the dirty vector: one writer per hive.
    let _hive = HvpLockHiveExclusive(RegistryHive);

    // Size = ROUND_UP(Size + sizeof(HCELL), 16);
    Size = ROUND_UP(Size as usize + core::mem::size_of::<HCELL>(), 16) as u32;

    // First search in free blocks.
    FreeCellOffset = HvpFindFree(RegistryHive, Size, Storage);

    // If no free cell was found we need to extend the hive file.
    if FreeCellOffset == HCELL_NIL {
        Bin = HvpAddBin(RegistryHive, Size, Storage);
        if Bin.is_null() {
            return HCELL_NIL;
        }
        FreeCellOffset = (*Bin).FileOffset + core::mem::size_of::<HBIN>() as u32;
        FreeCellOffset |= Storage << HCELL_TYPE_SHIFT;
    }

    FreeCell = HvpGetCellHeader(RegistryHive, FreeCellOffset);

    // Split the block in two parts.
    // The free block that is created has to be at least
    // sizeof(HCELL) + sizeof(HCELL_INDEX) big...
    if ((*FreeCell).Size as u32) > Size + 16 {
        NewCell = (FreeCell as usize + Size as usize) as *mut HCELL; // (PHCELL)((ULONG_PTR)FreeCell + Size)
        (*NewCell).Size = (*FreeCell).Size - Size as i32;
        (*FreeCell).Size = Size as i32;
        HvpAddFree(RegistryHive, NewCell, FreeCellOffset + Size);
        if Storage == Stable {
            HvMarkCellDirty(RegistryHive, FreeCellOffset + Size, 0);
        }
    }

    if Storage == Stable {
        HvMarkCellDirty(RegistryHive, FreeCellOffset, 0);
    }

    (*FreeCell).Size = -((*FreeCell).Size);
    // RtlZeroMemory(FreeCell + 1, Size - sizeof(HCELL));
    core::ptr::write_bytes(
        (FreeCell as *mut u8).add(core::mem::size_of::<HCELL>()),
        0,
        (Size - core::mem::size_of::<HCELL>() as u32) as usize,
    );

    FreeCellOffset
}

#[no_mangle]
pub unsafe extern "system" fn HvReallocateCell(RegistryHive: *mut _HHIVE, CellIndex: u32, Size: u32) -> u32 {
    let OldCell: *mut core::ffi::c_void;
    let NewCell: *mut core::ffi::c_void;
    let OldCellSize: i32;
    let NewCellIndex: u32;
    let Storage: u32;

    ntkrnl_sys::ASSERT!(CellIndex != HCELL_NIL);

    let _hive = HvpLockHiveExclusive(RegistryHive);
    Storage = HvGetCellType(CellIndex);

    OldCell = HvGetCell(RegistryHive, CellIndex);
    OldCellSize = HvGetCellSize(RegistryHive, OldCell);
    ntkrnl_sys::ASSERT!(OldCellSize > 0);

    if Size > (OldCellSize as u32) {
        NewCellIndex = HvAllocateCell(RegistryHive, Size, Storage, HCELL_NIL);
        if NewCellIndex == HCELL_NIL {
            return HCELL_NIL;
        }

        NewCell = HvGetCell(RegistryHive, NewCellIndex);
        RtlCopyMemory(NewCell, OldCell as *const core::ffi::c_void, OldCellSize as usize);

        HvFreeCell(RegistryHive, CellIndex);

        return NewCellIndex;
    }

    return CellIndex;
}

#[no_mangle]
pub unsafe extern "system" fn HvFreeCell(RegistryHive: *mut HHIVE, CellIndex: HCELL_INDEX) {
    const HCELL_TYPE_MASK: u32 = 0x8000_0000;

    ntkrnl_sys::ASSERT!((*RegistryHive).ReadOnly == 0);

    let _hive = HvpLockHiveExclusive(RegistryHive);
    let Free: *mut HCELL = HvpGetCellHeader(RegistryHive, CellIndex);
    ntkrnl_sys::ASSERT!((*Free).Size < 0);
    (*Free).Size = -((*Free).Size);

    let CellType = HvGetCellType(CellIndex);
    let CellBlock = HvGetCellBlock(CellIndex);

    // FIXME (as in C): Merge free blocks
    let entry = (*RegistryHive).Storage[CellType as usize].BlockList.add(CellBlock as usize);
    let Bin: *mut HBIN = (*entry).BinAddress as *mut HBIN;

    let mut Neighbor: *mut HCELL;

    if (CellIndex & !HCELL_TYPE_MASK) + ((*Free).Size as u32) < (*Bin).FileOffset + (*Bin).Size {
        Neighbor = (Free as usize + (*Free).Size as usize) as *mut HCELL;
        if (*Neighbor).Size > 0 {
            let nci = ((Neighbor as usize - Bin as usize) as u32 + (*Bin).FileOffset)
                | (CellIndex & HCELL_TYPE_MASK);
            HvpRemoveFree(RegistryHive, Neighbor as *mut CELL_DATA, nci);
            (*Free).Size += (*Neighbor).Size;
        }
    }

    Neighbor = Bin.add(1) as *mut HCELL; // (PHCELL)(Bin + 1)
    while Neighbor < Free {
        if (*Neighbor).Size > 0 {
            if (Neighbor as usize + (*Neighbor).Size as usize) == Free as usize {
                let NeighborCellIndex = ((Neighbor as usize - Bin as usize) as u32
                    + (*Bin).FileOffset)
                    | (CellIndex & HCELL_TYPE_MASK);

                if HvpComputeFreeListIndex((*Neighbor).Size as u32)
                    != HvpComputeFreeListIndex(((*Neighbor).Size + (*Free).Size) as u32)
                {
                    HvpRemoveFree(RegistryHive, Neighbor as *mut CELL_DATA, NeighborCellIndex);
                    (*Neighbor).Size += (*Free).Size;
                    HvpAddFree(RegistryHive, Neighbor, NeighborCellIndex);
                } else {
                    (*Neighbor).Size += (*Free).Size;
                }

                if CellType == Stable {
                    HvMarkCellDirty(RegistryHive, NeighborCellIndex, 0);
                }
                return;
            }
            Neighbor = (Neighbor as usize + (*Neighbor).Size as usize) as *mut HCELL;
        } else {
            // Size < 0: step by its absolute value.
            Neighbor = (Neighbor as usize).wrapping_add((-(*Neighbor).Size) as usize) as *mut HCELL;
        }
    }

    // Add block to the list of free blocks
    HvpAddFree(RegistryHive, Free, CellIndex);
    if CellType == Stable {
        HvMarkCellDirty(RegistryHive, CellIndex, 0);
    }
}

#[no_mangle]
pub unsafe extern "system" fn HvpCreateHiveFreeCellList(Hive: *mut HHIVE) -> i32 {
    // Initialize the free cell list (24 buckets × 2 storage types).
    let mut Index: u32 = 0;
    while Index < 24 {
        (*Hive).Storage[Stable as usize].FreeDisplay[Index as usize] = HCELL_NIL;
        (*Hive).Storage[Volatile as usize].FreeDisplay[Index as usize] = HCELL_NIL;
        Index += 1;
    }

    let mut BlockIndex: u32 = 0;
    while BlockIndex < (*Hive).Storage[Stable as usize].Length {
        let entry = (*Hive).Storage[Stable as usize].BlockList.add(BlockIndex as usize);
        let Bin: *mut HBIN = (*entry).BinAddress as *mut HBIN;

        // Search free blocks and add to list
        let mut FreeOffset: u32 = core::mem::size_of::<HBIN>() as u32;
        while FreeOffset < (*Bin).Size {
            let FreeBlock = (Bin as usize + FreeOffset as usize) as *mut HCELL;
            if (*FreeBlock).Size > 0 {
                let Status = HvpAddFree(Hive, FreeBlock, (*Bin).FileOffset + FreeOffset);
                if Status < 0 {
                    return Status;
                }
                FreeOffset += (*FreeBlock).Size as u32;
            } else {
                // C: FreeOffset -= FreeBlock->Size (Size < 0 → advance forward).
                FreeOffset = FreeOffset.wrapping_sub((*FreeBlock).Size as u32);
            }
        }

        BlockIndex += (*Bin).Size / HBLOCK_SIZE;
    }

    0 // STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn HvTrackCellRef(
    CellRef: *mut HV_TRACK_CELL_REF,
    Hive: *mut HHIVE,
    Cell: HCELL_INDEX,
) -> u8 {
    const STATIC_CELL_PAIR_COUNT: usize = 4;
    const CELL_REF_INCREMENT: u32 = 10;
    // C: cmlib.h `#define TAG_CM '  MC'`. A multi-char literal is BIG-endian.
    const TAG_CM: u32 = u32::from_be_bytes(*b"  MC");
    const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;

    PAGED_CODE!();

    ntkrnl_sys::ASSERT!(!CellRef.is_null());
    ntkrnl_sys::ASSERT!(!Hive.is_null());
    ntkrnl_sys::ASSERT!(Cell != HCELL_NIL);

    if ((*CellRef).StaticCount as u32) < (STATIC_CELL_PAIR_COUNT as u32) {
        (*CellRef).StaticArray[(*CellRef).StaticCount as usize].Hive = Hive;
        (*CellRef).StaticArray[(*CellRef).StaticCount as usize].Cell = Cell;
        (*CellRef).StaticCount += 1;
        return 1;
    }

    // DPRINT("HvTrackCellRef: Static array full, use dynamic array.\n");
    DbgPrint(b"HvTrackCellRef: Static array full, use dynamic array.\n\0".as_ptr() as *const u8);

    if (*CellRef).Max == 0 {
        ntkrnl_sys::ASSERT!((*CellRef).CellArray.is_null());
        ntkrnl_sys::ASSERT!((*CellRef).Count == 0);
    } else {
        ntkrnl_sys::ASSERT!(!(*CellRef).CellArray.is_null());
    }
    ntkrnl_sys::ASSERT!((*CellRef).Count <= (*CellRef).Max);

    if (*CellRef).Count == (*CellRef).Max {
        let size = ((*CellRef).Max as usize + CELL_REF_INCREMENT as usize) * core::mem::size_of::<HV_HIVE_CELL_PAIR>();
        let new_cell_array = CmpAllocate(size as PtrInt, 1u8, TAG_CM) as *mut HV_HIVE_CELL_PAIR;
        if new_cell_array.is_null() {
            // DPRINT1("HvTrackCellRef: Cannot reallocate the reference table.\n");
            DbgPrint(b"HvTrackCellRef: Cannot reallocate the reference table.\n\0".as_ptr() as *const u8);
            HvReleaseCell(Hive, Cell);
            return 0;
        }

        if !(*CellRef).CellArray.is_null() {
            core::ptr::copy_nonoverlapping(
                (*CellRef).CellArray,
                new_cell_array,
                (*CellRef).Max as usize,
            );
            CmpFree((*CellRef).CellArray as *mut c_void, TAG_CM);
        }
        (*CellRef).CellArray = new_cell_array;
        (*CellRef).Max += CELL_REF_INCREMENT as u16;
    }

    // ASSERT(CellRef->Count < CellRef->Max);

    (*(*CellRef).CellArray.add((*CellRef).Count as usize)).Hive = Hive;
    (*(*CellRef).CellArray.add((*CellRef).Count as usize)).Cell = Cell;
    (*CellRef).Count += 1;
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn HvReleaseFreeCellRefArray(
    CellRef: PHV_TRACK_CELL_REF,
) {
    // PAGED_CODE();
    // ASSERT(CellRef);

    let mut i: u32;

    /* Any references in the static array? */
    if (*CellRef).StaticCount > 0 {
        /* Sanity check */
        // ASSERT(CellRef->StaticCount <= STATIC_CELL_PAIR_COUNT);

        /* Loop over them and release them */
        i = 0;
        while i < (*CellRef).StaticCount as u32 {
            let pair_ptr = core::ptr::addr_of!((*CellRef).StaticArray[i as usize]);
            HvReleaseCell((*pair_ptr).Hive, (*pair_ptr).Cell);
            i += 1;
        }

        /* We can reuse the static array */
        (*CellRef).StaticCount = 0;
    }

    /* Any references in the dynamic array? */
    if (*CellRef).Count > 0 {
        /* Sanity checks */
        // ASSERT(CellRef->Count <= CellRef->Max);
        // ASSERT(CellRef->CellArray);

        let cell_array: *mut HV_HIVE_CELL_PAIR = (*CellRef).CellArray;

        /* Loop over them and release them */
        i = 0;
        while i < (*CellRef).Count as u32 {
            let pair_ptr = cell_array.add(i as usize);
            HvReleaseCell((*pair_ptr).Hive, (*pair_ptr).Cell);
            i += 1;
        }

        /* We can reuse the dynamic array */
        CmpFree(cell_array as *mut core::ffi::c_void, 0); // TAG_CM
        (*CellRef).CellArray = core::ptr::null_mut();
        (*CellRef).Max = 0;
        (*CellRef).Count = 0;
    }
}
