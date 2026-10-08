//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/cmlib/cmvalue.c.
//! Risk: HIGH. Behavior: 3 bugs found.
//! Findings:
//!   BUG1 CmpSetValueDataNew@144 — ASSERT uses 0x1000 as CM_KEY_VALUE_SMALL; correct value is 0x4 (cmdata.h).
//!     0x1000 is HBLOCK_SIZE. ASSERT fires incorrectly for DataSize in [5..4096]; passes DataSize==4 which
//!     violates the precondition DataSize > CM_KEY_VALUE_SMALL.
//!   BUG2 CmpRemoveValueFromList@197 — (*CellData).u.KeyList[Index as usize] direct-index on [HCELL_INDEX; 1]
//!     flex-array stub. Panics in debug / UB in release for Index >= 1. Must use pointer arithmetic
//!     (addr_of!(...).add(n)) as done correctly in cmname.rs.
//!   BUG3 CmpCopyKeyValueList@250,275 — same flex-array [Index as usize] direct-index bug on KeyList;
//!     any list with more than 1 entry is affected.
//! Unverified: HvMarkCellDirty local redecl uses Write:i32 vs glue.rs u8 — harmless on all 3 arches.
//!   CmpFindValueByName CellIndex init=0 (not HCELL_NIL) only matters if CmpFindNameInList skips the
//!   out-param write entirely on failure; functionally fine since caller checks HCELL_NIL on success path.
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CELL_DATA, CHILD_LIST, CM_KEY_VALUE, CmpFindNameInList, HCELL_INDEX, HHIVE, HSTORAGE_TYPE, PCELL_DATA, PCM_KEY_NODE, PCM_KEY_VALUE, PCUNICODE_STRING, PHHIVE, _HHIVE};
use crate::glue::{round_up, HvGetCell, HvGetCellSize, HvReleaseCell, HBLOCK_SIZE, HCELL_NIL};

// cmdata.h value constants.
const CM_KEY_VALUE_SIGNATURE: u16 = 0x6B76; // "vk"
const CM_KEY_VALUE_SMALL: u32 = 0x4;
const CM_KEY_VALUE_SPECIAL_SIZE: u32 = 0x8000_0000;
use core::ffi::c_void;

extern "system" {
    fn HvIsCellDirty(hive: *mut HHIVE, cell: HCELL_INDEX) -> u8;
    fn HvReallocateCell(hive: *mut HHIVE, cell: HCELL_INDEX, size: u32) -> HCELL_INDEX;
}

// CmpCopyCell (cmvalue.c:374) — copy a raw cell from one hive to another.
#[no_mangle]
pub unsafe extern "system" fn CmpCopyCell(
    SourceHive: *mut _HHIVE,
    SourceCell: HCELL_INDEX,
    DestinationHive: *mut _HHIVE,
    StorageType: HSTORAGE_TYPE,
) -> HCELL_INDEX {
    PAGED_CODE!();

    let SourceData = HvGetCell(SourceHive, SourceCell);
    let DataSize = HvGetCellSize(SourceHive, SourceData);

    let mut DestinationData: *mut c_void = core::ptr::null_mut();
    let DestinationCell = HvAllocateCell(DestinationHive, DataSize as u32, StorageType, HCELL_NIL);
    if DestinationCell != HCELL_NIL {
        DestinationData = HvGetCell(DestinationHive, DestinationCell);
        // RtlMoveMemory(DestinationData, SourceData, DataSize)
        core::ptr::copy(SourceData as *const u8, DestinationData as *mut u8, DataSize as usize);
    }

    if !DestinationData.is_null() {
        HvReleaseCell(DestinationHive, DestinationCell);
    }
    if !SourceData.is_null() {
        HvReleaseCell(SourceHive, SourceCell);
    }

    DestinationCell
}

use crate::glue::{
    CmpIsKeyValueSmall, HvAllocateCell, HvFreeCell, HvMarkCellDirty, ASSERT, };

// ---- file-scope preamble (translated) ----
// #define NDEBUG not converted (preprocessor macro without constant value)

const REGISTRY_ERROR: u32 = 0x51;

#[no_mangle]
pub unsafe extern "system" fn CmpMarkValueDataDirty(
    Hive: *mut HHIVE,
    Value: *const CM_KEY_VALUE,
) -> i32 {
    let mut KeySize: u32 = 0;
    PAGED_CODE!();
    if (*Value).Data != HCELL_NIL {
        if CmpIsKeyValueSmall(core::ptr::addr_of_mut!(KeySize), (*Value).DataLength) {
            return 1i32;
        }
        let _ = (&Hive, KeySize);
        HvMarkCellDirty(Hive, (*Value).Data, 0);
    }
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn CmpFreeValueData(
    Hive: *mut HHIVE,
    DataCell: u32,
    DataLength: u32,
) -> i32 {
    let mut KeySize: u32 = 0;
    // PAGED_CODE();
    if !CmpIsKeyValueSmall(core::ptr::addr_of_mut!(KeySize), DataLength) {
        if DataCell == HCELL_NIL {
            return 1;
        }
        // ASSERT(HvIsCellAllocated(Hive, DataCell));
        let _ = (&Hive, KeySize);
        HvFreeCell(Hive, DataCell);
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn CmpFindValueByName(
    Hive: PHHIVE,
    KeyNode: PCM_KEY_NODE,
    Name: PCUNICODE_STRING,
) -> HCELL_INDEX {
    let mut CellIndex: HCELL_INDEX = 0;
    if CmpFindNameInList(
        Hive,
        core::ptr::addr_of_mut!((*KeyNode).__bindgen_anon_1.__bindgen_anon_1.ValueList),
        Name,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(CellIndex),
    ) == 0
    {
        // ASSERT(CellIndex == HCELL_NIL);  /* omitted, NDEBUG */
    }
    CellIndex
}
#[no_mangle]
pub unsafe extern "system" fn CmpValueToData(
    Hive: *mut HHIVE,
    Value: *mut CM_KEY_VALUE,
    Length: *mut u32,
) -> *mut CELL_DATA {
    // PAGED_CODE();

    // Sanity check
    // ASSERT(Hive->ReleaseCellRoutine == NULL);

    let mut Buffer: *mut CELL_DATA = core::ptr::null_mut();
    let mut BufferAllocated: u8 = 0;
    let mut CellToRelease: HCELL_INDEX = 0;

    // Get the actual data
    if CmpGetValueData(
        Hive,
        Value,
        Length,
        core::ptr::addr_of_mut!(Buffer) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(BufferAllocated),
        core::ptr::addr_of_mut!(CellToRelease),
    ) == 0
    {
        // We failed
        // ASSERT(BufferAllocated == FALSE);
        // ASSERT(Buffer == NULL);
        return core::ptr::null_mut();
    }

    // This should never happen!
    if BufferAllocated != 0 {
        // Free the buffer and bugcheck
        ((*Hive).Free.unwrap())(Buffer as *mut c_void, 0);
        KeBugCheckEx(REGISTRY_ERROR, 8, 0, Hive as usize, Value as usize);
    }

    // Otherwise, return the cell data
    Buffer
}

#[no_mangle]
pub unsafe extern "system" fn CmpSetValueDataNew(
    hive: *mut _HHIVE,
    data: *const core::ffi::c_void,
    data_size: u32,
    storage_type: HSTORAGE_TYPE,
    _value_cell: HCELL_INDEX,
    data_cell: *mut HCELL_INDEX,
) -> NTSTATUS {
    PAGED_CODE!();
    ASSERT(data_size > 0x4 /* CM_KEY_VALUE_SMALL */);

    let _ = (&hive, data_size);

    *data_cell = HvAllocateCell(hive, data_size, storage_type, HCELL_NIL);
    if *data_cell == HCELL_NIL {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let cell_data = HvGetCell(hive, *data_cell) as PCELL_DATA;
    if cell_data.is_null() {
        ASSERT(false);
    }

    RtlCopyMemory(
        cell_data as *mut core::ffi::c_void,
        data,
        data_size as usize,
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmpRemoveValueFromList(
    Hive: *mut _HHIVE,
    mut Index: u32,
    ChildList: *mut CHILD_LIST
) -> i32 {
    let mut Count: u32 = 0;
    let mut CellData: *mut CELL_DATA = core::ptr::null_mut();
    let mut NewCell: HCELL_INDEX = 0;

    // PAGED_CODE();
    PAGED_CODE!();

    // ASSERT((((LONG)Index) >= 0) && (Index <= ChildList->Count));
    // (omitted due to NDEBUG)

    Count = (*ChildList).Count - 1;

    if Count > 0 {
        CellData = HvGetCell(Hive, (*ChildList).List) as *mut CELL_DATA;
        if CellData.is_null() {
            return 0xC000009Au32 as i32; // STATUS_INSUFFICIENT_RESOURCES
        }

        // ASSERT(HvIsCellDirty(Hive, ChildList->List));
        // (omitted due to NDEBUG)
        // ASSERT(HvIsCellDirty(Hive, CellData->u.KeyList[Index as usize]));
        // (omitted due to NDEBUG)

        while Index < Count {
            *(core::ptr::addr_of_mut!((*CellData).u.KeyList) as *mut HCELL_INDEX).add(Index as usize) = *(core::ptr::addr_of!((*CellData).u.KeyList) as *const HCELL_INDEX).add((Index + 1) as usize);
            Index += 1;
        }

        NewCell = HvReallocateCell(
            Hive,
            (*ChildList).List,
            Count * core::mem::size_of::<HCELL_INDEX>() as u32,
        );

        // ASSERT(NewCell != HCELL_NIL);
        // (omitted due to NDEBUG)

        HvReleaseCell(Hive, (*ChildList).List);
        (*ChildList).List = NewCell;
    } else {
        HvFreeCell(Hive, (*ChildList).List);
        (*ChildList).List = HCELL_NIL;
    }

    (*ChildList).Count = Count;
    return 0; // STATUS_SUCCESS
}

// Forward declarations for symbols not in the pre-resolved list

#[no_mangle]
pub unsafe extern "system" fn CmpCopyKeyValueList(
    SourceHive: *mut _HHIVE,
    SrcValueList: *mut CHILD_LIST,
    DestinationHive: *mut _HHIVE,
    DestValueList: *mut CHILD_LIST,
    StorageType: HSTORAGE_TYPE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut SrcListData: *mut CELL_DATA = core::ptr::null_mut();
    let mut DestListData: *mut CELL_DATA = core::ptr::null_mut();
    let mut NewValue: HCELL_INDEX;
    let mut Index: u32;

    (*DestValueList).Count = 0;
    (*DestValueList).List = HCELL_NIL;

    if (*SrcValueList).Count == 0 {
        return STATUS_SUCCESS;
    }

    SrcListData = HvGetCell(SourceHive, (*SrcValueList).List) as *mut CELL_DATA;

    Index = 0;
    while Index < (*SrcValueList).Count {
        NewValue = CmpCopyValue(
            SourceHive,
            *(core::ptr::addr_of!((*SrcListData).u.KeyList) as *const HCELL_INDEX).add(Index as usize),
            DestinationHive,
            StorageType,
        );
        if NewValue == HCELL_NIL {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break;
        }

        Status = CmpAddValueToList(DestinationHive, NewValue, Index, StorageType, DestValueList);
        if !NT_SUCCESS(Status) {
            if CmpFreeValue(DestinationHive, NewValue) == 0 {
                HvFreeCell(DestinationHive, NewValue);
            }
            break;
        }

        Index += 1;
    }

    if !NT_SUCCESS(Status) && ((*DestValueList).List != HCELL_NIL) {
        DestListData = HvGetCell(DestinationHive, (*DestValueList).List) as *mut CELL_DATA;

        while Index > 0 {
            Index -= 1;
            NewValue = *(core::ptr::addr_of!((*DestListData).u.KeyList) as *const HCELL_INDEX).add(Index as usize);
            if CmpFreeValue(DestinationHive, NewValue) == 0 {
                HvFreeCell(DestinationHive, NewValue);
            }
        }

        HvReleaseCell(DestinationHive, (*DestValueList).List);
        HvFreeCell(DestinationHive, (*DestValueList).List);
        (*DestValueList).Count = 0;
        (*DestValueList).List = HCELL_NIL;
    }

    HvReleaseCell(SourceHive, (*SrcValueList).List);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn CmpFreeValue(Hive: *mut _HHIVE, Cell: HCELL_INDEX) -> BOOLEAN {
    PAGED_CODE!();

    /* Get the cell data */
    let Value = HvGetCell(Hive, Cell) as PCM_KEY_VALUE;
    if Value.is_null() {
        ASSERT(false);
    }

    /* Free it */
    if CmpFreeValueData(Hive, (*Value).Data, (*Value).DataLength) == 0 {
        /* We failed to free the data, return failure */
        HvReleaseCell(Hive, Cell);
        return 0;
    }

    /* Release the cell and free it */
    HvReleaseCell(Hive, Cell);
    HvFreeCell(Hive, Cell);
    1
}

/*
 * NOTE: This function should support big values, contrary to CmpValueToData.
 */
#[no_mangle]
pub unsafe extern "system" fn CmpGetValueData(
    Hive: *mut HHIVE,
    Value: *mut CM_KEY_VALUE,
    Length: *mut u32,
    Buffer: *mut *mut core::ffi::c_void,
    BufferAllocated: *mut u8,
    CellToRelease: *mut HCELL_INDEX,
) -> u8 {
    PAGED_CODE!();

    /* Sanity check */
    ASSERT((*Value).Signature == CM_KEY_VALUE_SIGNATURE);

    /* Set failure defaults */
    *BufferAllocated = 0;
    *Buffer = core::ptr::null_mut();
    *CellToRelease = HCELL_NIL;

    /* Check if this is a small key */
    if CmpIsKeyValueSmall(Length, (*Value).DataLength) {
        /* Return the data immediately */
        *Buffer = core::ptr::addr_of_mut!((*Value).Data) as *mut core::ffi::c_void;
        return 1;
    }

    /* Unsupported at the moment */
    let _ = (&Hive, *Length); /* ASSERT_VALUE_BIG(Hive, *Length) — no-op (NDEBUG) */

    /* Get the data from the cell */
    *Buffer = HvGetCell(Hive, (*Value).Data);
    if (*Buffer).is_null() {
        return 0;
    }

    /* Return success and the cell to be released */
    *CellToRelease = (*Value).Data;
    1
}

#[no_mangle]
pub unsafe extern "system" fn CmpAddValueToList(
    Hive: *mut _HHIVE,
    ValueCell: HCELL_INDEX,
    Index: u32,
    StorageType: HSTORAGE_TYPE,
    ChildList: *mut CHILD_LIST,
) -> NTSTATUS {
    let ListCell: HCELL_INDEX;
    let mut ChildCount: u32;
    let Length: u32;
    let mut i: u32;
    PAGED_CODE!();

    /* Sanity check */
    ASSERT(Index <= (*ChildList).Count);

    /* Get the number of entries in the child list */
    ChildCount = (*ChildList).Count;
    ChildCount += 1;
    if ChildCount > 1 {
        ASSERT((*ChildList).List != HCELL_NIL);

        /* The cell should be dirty at this point */
        ASSERT(HvIsCellDirty(Hive, (*ChildList).List) != 0);

        /* Check if we have less then 100 children */
        if ChildCount < 100 {
            /* Allocate just enough as requested */
            Length = ChildCount * core::mem::size_of::<HCELL_INDEX>() as u32;
        } else {
            /* Otherwise, we have quite a few, so allocate a batch */
            let mut len = round_up(ChildCount, 100) * core::mem::size_of::<HCELL_INDEX>() as u32;
            if len > HBLOCK_SIZE {
                /* But make sure we don't allocate beyond our block size */
                len = round_up(len, HBLOCK_SIZE);
            }
            Length = len;
        }

        /* Perform the allocation */
        ListCell = HvReallocateCell(Hive, (*ChildList).List, Length);
    } else {
        /* This is our first child, so allocate a single cell */
        ASSERT((*ChildList).List == HCELL_NIL);
        ListCell = HvAllocateCell(
            Hive,
            core::mem::size_of::<HCELL_INDEX>() as u32,
            StorageType,
            HCELL_NIL,
        );
    }

    /* Fail if we couldn't get a cell */
    if ListCell == HCELL_NIL {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Set this cell as the child list's list cell */
    (*ChildList).List = ListCell;

    /* Get the actual key list memory */
    let CellData = HvGetCell(Hive, ListCell) as PCELL_DATA;
    ASSERT(!CellData.is_null());

    /* Loop all the children */
    let key_list = core::ptr::addr_of_mut!((*CellData).u.KeyList) as *mut HCELL_INDEX;
    i = ChildCount - 1;
    while i > Index {
        /* Move them all down */
        *key_list.add(i as usize) = *key_list.add((i - 1) as usize);
        i -= 1;
    }

    /* Insert us on top now */
    *key_list.add(Index as usize) = ValueCell;
    (*ChildList).Count = ChildCount;

    /* Release the list cell and make sure the value cell is dirty */
    HvReleaseCell(Hive, ListCell);
    ASSERT(HvIsCellDirty(Hive, ValueCell) != 0);

    /* We're done here */
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CmpCopyValue(
    SourceHive: *mut _HHIVE,
    SourceValueCell: HCELL_INDEX,
    DestinationHive: *mut _HHIVE,
    StorageType: HSTORAGE_TYPE,
) -> HCELL_INDEX {
    let Value: PCM_KEY_VALUE;
    let NewValue: PCM_KEY_VALUE;
    let mut NewValueCell: HCELL_INDEX;
    let NewDataCell: HCELL_INDEX;
    let CellData: PCELL_DATA;
    let SmallData: u32;
    let mut DataSize: u32 = 0;
    let IsSmall: bool;

    PAGED_CODE!();

    /* Get the actual source data */
    Value = HvGetCell(SourceHive, SourceValueCell) as PCM_KEY_VALUE;
    if Value.is_null() {
        ASSERT(false);
    }

    /* Copy the value cell body */
    NewValueCell = CmpCopyCell(SourceHive, SourceValueCell, DestinationHive, StorageType);
    if NewValueCell == HCELL_NIL {
        /* Not enough storage space */
        HvReleaseCell(SourceHive, SourceValueCell);
        return NewValueCell;
    }

    /* Copy the value data */
    IsSmall = CmpIsKeyValueSmall(core::ptr::addr_of_mut!(DataSize), (*Value).DataLength);
    if DataSize == 0 {
        /* Nothing to copy */
        NewValue = HvGetCell(DestinationHive, NewValueCell) as PCM_KEY_VALUE;
        ASSERT(!NewValue.is_null());
        (*NewValue).DataLength = 0;
        (*NewValue).Data = HCELL_NIL;
        HvReleaseCell(DestinationHive, NewValueCell);

        HvReleaseCell(SourceHive, SourceValueCell);
        return NewValueCell;
    }

    if DataSize <= CM_KEY_VALUE_SMALL {
        if IsSmall {
            /* Small value, copy directly */
            SmallData = (*Value).Data;
        } else {
            /* The value is small, but was stored in a regular cell. Get the data from it. */
            CellData = HvGetCell(SourceHive, (*Value).Data) as PCELL_DATA;
            ASSERT(!CellData.is_null());
            SmallData = *(CellData as *const u32);
            HvReleaseCell(SourceHive, (*Value).Data);
        }

        /* This is a small key, set the data directly inside */
        NewValue = HvGetCell(DestinationHive, NewValueCell) as PCM_KEY_VALUE;
        ASSERT(!NewValue.is_null());
        (*NewValue).DataLength = DataSize + CM_KEY_VALUE_SPECIAL_SIZE;
        (*NewValue).Data = SmallData;
        HvReleaseCell(DestinationHive, NewValueCell);
    } else {
        /* Big keys are currently unsupported */
        let _ = (&SourceHive, DataSize); /* ASSERT_VALUE_BIG — no-op (NDEBUG) */
        // Should use CmpGetValueData and CmpSetValueDataNew for big values!

        /* Regular value */

        /* Copy the data cell */
        NewDataCell = CmpCopyCell(SourceHive, (*Value).Data, DestinationHive, StorageType);
        if NewDataCell == HCELL_NIL {
            /* Not enough storage space */
            HvFreeCell(DestinationHive, NewValueCell);
            NewValueCell = HCELL_NIL;
            HvReleaseCell(SourceHive, SourceValueCell);
            return NewValueCell;
        }

        NewValue = HvGetCell(DestinationHive, NewValueCell) as PCM_KEY_VALUE;
        ASSERT(!NewValue.is_null());
        (*NewValue).DataLength = DataSize;
        (*NewValue).Data = NewDataCell;
        HvReleaseCell(DestinationHive, NewValueCell);
    }

    HvReleaseCell(SourceHive, SourceValueCell);

    /* Return the copied value body cell index */
    NewValueCell
}
