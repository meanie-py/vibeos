use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CELL_DATA, CM_KEY_NODE, CmpAllocate, CmpFree, CmpLazyFlush, HBIN, HCELL, HCELL_INDEX, HHIVE, HSTORAGE_TYPE, HV_HIVE_CELL_PAIR, PCELL_DATA, PHBIN, PHCELL, PHCELL_INDEX, PHHIVE, PHV_HIVE_CELL_PAIR, PHV_TRACK_CELL_REF, HV_TRACK_CELL_REF, RtlSetBits, HSTORAGE_TYPE_Stable as Stable, HSTORAGE_TYPE_Volatile as Volatile, _HHIVE, _UNICODE_STRING};
use crate::glue::{HvGetCell, HvReleaseCell, HCELL_NIL, HBLOCK_SIZE};
use core::ffi::c_void;

// Pointer-width integer matching ULONG_PTR/SIZE_T on all arches
#[cfg(target_pointer_width = "32")]
type PtrInt = u32;
#[cfg(target_pointer_width = "64")]
type PtrInt = u64;

unsafe extern "C" {
    fn CMLTRACE(level: u32, format: *const i8, ...);
    fn HvGetCellBlock(cell: HCELL_INDEX) -> u32;
    fn HvGetCellType(cell: HCELL_INDEX) -> u32;
    unsafe fn HvpComputeFreeListIndex(Size: u32) -> u32;
    fn HvpAddBin(RegistryHive: *mut HHIVE, Size: u32, Storage: HSTORAGE_TYPE) -> *mut HBIN;
    unsafe fn HvpFindFree(RegistryHive: *mut HHIVE, Size: u32, Storage: HSTORAGE_TYPE) -> u32;
    fn HvFreeCell(RegistryHive: *mut _HHIVE, CellIndex: u32);
    fn HvGetCellSize(RegistryHive: *mut _HHIVE, Address: *mut core::ffi::c_void) -> i32;
}
unsafe extern "C" {
    fn RtlCheckBit(BitMapHeader: *mut core::ffi::c_void, BitNumber: u32) -> i32;
    fn PAGED_CODE();
    fn DbgPrint(format: *const u8, ...);
    fn HvpGetCellHeader(RegistryHive: *mut HHIVE, CellIndex: HCELL_INDEX) -> *mut HCELL;
    fn HvMarkCellDirty(RegistryHive: *mut HHIVE, CellIndex: HCELL_INDEX, HoldingLock: i32) -> i32;
    fn HvpAddFree(RegistryHive: *mut HHIVE, FreeBlock: *mut HCELL, FreeIndex: HCELL_INDEX) -> i32;
}

const CMLIB_HCELL_DEBUG: u32 = 0;

unsafe extern "C" fn HvpRemoveFree(
    RegistryHive: *mut HHIVE,
    CellBlock: *mut CELL_DATA,
    CellIndex: u32,
) {
    let mut FreeCellData: *mut u32;
    let mut pFreeCellOffset: *mut u32;
    let Storage: u32;
    let Index: u32;
    let mut FreeListIndex: u32;

    assert!((*RegistryHive).ReadOnly == 0);

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

    CMLTRACE(CMLIB_HCELL_DEBUG, b"-- beginning of HvpRemoveFree trace --\n\0".as_ptr() as *const i8);
    CMLTRACE(CMLIB_HCELL_DEBUG, b"block we are about to free: %08x\n\0".as_ptr() as *const i8, CellIndex);
    CMLTRACE(CMLIB_HCELL_DEBUG, b"chosen free list index: %u\n\0".as_ptr() as *const i8, Index);

    for FreeListIndex in 0..24u32 {
        CMLTRACE(CMLIB_HCELL_DEBUG, b"free list [%u]: \0".as_ptr() as *const i8, FreeListIndex);
        pFreeCellOffset = core::ptr::addr_of_mut!((*RegistryHive).Storage[Storage as usize].FreeDisplay[FreeListIndex as usize]);
        while *pFreeCellOffset != HCELL_NIL {
            CMLTRACE(CMLIB_HCELL_DEBUG, b"%08x \0".as_ptr() as *const i8, *pFreeCellOffset);
            FreeCellData = HvGetCell(RegistryHive, *pFreeCellOffset) as *mut u32;
            pFreeCellOffset = FreeCellData;
        }
        CMLTRACE(CMLIB_HCELL_DEBUG, b"\n\0".as_ptr() as *const i8);
    }
    CMLTRACE(CMLIB_HCELL_DEBUG, b"-- end of HvpRemoveFree trace --\n\0".as_ptr() as *const i8);

    assert!(false);
}
#[no_mangle]
pub unsafe extern "C" fn HvAllocateCell(
    RegistryHive: *mut _HHIVE,
    mut Size: u32,
    Storage: u32,
    Vicinity: u32,
) -> u32 {
    const HCELL_NIL: u32 = 0;
    const HCELL_TYPE_SHIFT: u32 = 30;
    const CMLIB_HCELL_DEBUG: u32 = 1;
    static __FUNCTION__: &[u8] = b"HvAllocateCell\0";

    let mut FreeCell: *mut HCELL;
    let mut FreeCellOffset: u32;
    let NewCell: *mut HCELL;
    let Bin: *mut HBIN;

    // ASSERT(RegistryHive->ReadOnly == FALSE);
    core::assert_eq!((*RegistryHive).ReadOnly, 0);

    // CMLTRACE(CMLIB_HCELL_DEBUG, "%s - Hive 0x%p, Size 0x%x, %s, Vicinity 0x%x\n",
    //          __FUNCTION__, RegistryHive, Size, (Storage == 0) ? "Stable" : "Volatile", Vicinity);
    CMLTRACE(
        CMLIB_HCELL_DEBUG,
        b"%s - Hive 0x%p, Size 0x%x, %s, Vicinity 0x%x\n\0".as_ptr() as *const i8,
        __FUNCTION__.as_ptr() as *const i8,
        RegistryHive,
        Size,
        if Storage == Stable {
            b"Stable\0".as_ptr() as *const i8
        } else {
            b"Volatile\0".as_ptr() as *const i8
        },
        Vicinity,
    );

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

    // CMLTRACE(CMLIB_HCELL_DEBUG, "%s - CellIndex 0x%x\n", __FUNCTION__, FreeCellOffset);
    CMLTRACE(
        CMLIB_HCELL_DEBUG,
        b"%s - CellIndex 0x%x\n\0".as_ptr() as *const i8,
        __FUNCTION__.as_ptr() as *const i8,
        FreeCellOffset,
    );

    FreeCellOffset
}

#[no_mangle]
pub unsafe extern "C" fn HvReallocateCell(RegistryHive: *mut _HHIVE, CellIndex: u32, Size: u32) -> u32 {
    let OldCell: *mut core::ffi::c_void;
    let NewCell: *mut core::ffi::c_void;
    let OldCellSize: i32;
    let NewCellIndex: u32;
    let Storage: u32;

    debug_assert!(CellIndex != HCELL_NIL);

    CMLTRACE(
        CMLIB_HCELL_DEBUG,
        b"%s - Hive 0x%p, CellIndex 0x%x, Size 0x%x\n\0".as_ptr() as *const i8,
        b"HvReallocateCell\0".as_ptr() as *const i8,
        RegistryHive as *const core::ffi::c_void,
        CellIndex,
        Size,
    );

    Storage = HvGetCellType(CellIndex);

    OldCell = HvGetCell(RegistryHive, CellIndex);
    OldCellSize = HvGetCellSize(RegistryHive, OldCell);
    debug_assert!(OldCellSize > 0);

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
pub unsafe extern "C" fn HvTrackCellRef(
    CellRef: *mut HV_TRACK_CELL_REF,
    Hive: *mut HHIVE,
    Cell: HCELL_INDEX,
) -> u8 {
    const STATIC_CELL_PAIR_COUNT: usize = 4;
    const CELL_REF_INCREMENT: u32 = 10;
    const TAG_CM: u32 = 0x434D;
    const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;

    PAGED_CODE();

    assert!(!CellRef.is_null());
    assert!(!Hive.is_null());
    assert!(Cell != HCELL_NIL);

    if ((*CellRef).StaticCount as u32) < (STATIC_CELL_PAIR_COUNT as u32) {
        (*CellRef).StaticArray[(*CellRef).StaticCount as usize].Hive = Hive;
        (*CellRef).StaticArray[(*CellRef).StaticCount as usize].Cell = Cell;
        (*CellRef).StaticCount += 1;
        return 1;
    }

    // DPRINT("HvTrackCellRef: Static array full, use dynamic array.\n");
    DbgPrint(b"HvTrackCellRef: Static array full, use dynamic array.\n\0".as_ptr() as *const u8);

    if (*CellRef).Max == 0 {
        assert!((*CellRef).CellArray.is_null());
        assert!((*CellRef).Count == 0);
    } else {
        assert!(!(*CellRef).CellArray.is_null());
    }
    assert!((*CellRef).Count <= (*CellRef).Max);

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
pub unsafe extern "C" fn HvReleaseFreeCellRefArray(
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
