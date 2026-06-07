use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CELL_DATA, CM_KEY_NODE, CmpFindNameInList, HCELL_INDEX, HHIVE, HSTORAGE_TYPE, PCELL_DATA, PCHILD_LIST, PCM_KEY_NODE, PCM_KEY_VALUE, PCUNICODE_STRING, PHCELL_INDEX, PHHIVE, _HHIVE, _UNICODE_STRING};
use crate::glue::{HvGetCell, HvReleaseCell};
use core::ffi::c_void;

extern "C" {
}
extern "system" {
    fn CmpCopyCell( SourceHive: *mut _HHIVE, SourceCell: HCELL_INDEX, DestinationHive: *mut _HHIVE, StorageType: HSTORAGE_TYPE, ) -> HCELL_INDEX;
    fn PAGED_CODE();
    fn HvMarkCellDirty(Hive: *mut HHIVE, Cell: u32, Write: i32);
    fn HvFreeCell(hive: *mut HHIVE, cell: u32);
    fn HvAllocateCell(hive: *mut HHIVE, size: u32, storage_type: HSTORAGE_TYPE, hint: HCELL_INDEX) -> HCELL_INDEX;
    fn HvIsCellDirty(hive: *mut HHIVE, cell: HCELL_INDEX) -> u8;
    fn HvReallocateCell(hive: *mut HHIVE, cell: HCELL_INDEX, size: u32) -> HCELL_INDEX;
    fn ASSERT(condition: i32);
    fn HvGetCellSize( Hive: *mut _HHIVE, Data: *mut CELL_DATA, ) -> i32;
    #[no_mangle] pub unsafe extern "system" fn CmpGetValueData( Hive: *mut HHIVE, Value: *mut CM_KEY_VALUE, Length: *mut u32, Buffer: *mut *mut core::ffi::c_void, BufferAllocated: *mut u8, CellToRelease: *mut HCELL_INDEX, ) -> u8;
    #[no_mangle] pub unsafe extern "system" fn CmpCopyValue( SourceHive: *mut _HHIVE, SourceValueCell: HCELL_INDEX, DestinationHive: *mut _HHIVE, StorageType: HSTORAGE_TYPE, ) -> HCELL_INDEX;
    #[no_mangle] pub unsafe extern "system" fn CmpAddValueToList( Hive: *mut _HHIVE, Cell: HCELL_INDEX, Index: u32, StorageType: HSTORAGE_TYPE, ChildList: *mut CHILD_LIST, ) -> NTSTATUS;
    #[no_mangle] pub unsafe extern "system" fn CmpFreeValue(Hive: *mut _HHIVE, Cell: HCELL_INDEX) -> BOOLEAN;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG not converted (preprocessor macro without constant value)

const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;
const HBLOCK_SIZE: u32 = 4096;

const REGISTRY_ERROR: u32 = 0x51;

#[no_mangle]
pub unsafe extern "system" fn CmpMarkValueDataDirty(
    Hive: *mut HHIVE,
    Value: *const CM_KEY_VALUE,
) -> i32 {
    let mut KeySize: u32 = 0;
    PAGED_CODE();
    if (*Value).Data != HCELL_NIL {
        if CmpIsKeyValueSmall(core::ptr::addr_of_mut!(KeySize), (*Value).DataLength) != 0 {
            return 1i32;
        }
        ASSERT_VALUE_BIG(Hive, KeySize);
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
    if CmpIsKeyValueSmall(core::ptr::addr_of_mut!(KeySize), DataLength) == 0 {
        if DataCell == HCELL_NIL {
            return 1;
        }
        // ASSERT(HvIsCellAllocated(Hive, DataCell));
        ASSERT_VALUE_BIG(Hive, KeySize);
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
        core::ptr::addr_of_mut!((*KeyNode).ValueList),
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

    let mut Buffer: *mut CELL_DATA;
    let mut BufferAllocated: u8;
    let mut CellToRelease: HCELL_INDEX;

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
        ((*Hive).Free)(Buffer, 0);
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
    value_cell: HCELL_INDEX,
    data_cell: *mut HCELL_INDEX,
) -> NTSTATUS {
    PAGED_CODE();
    ASSERT((data_size > 0x1000 /* CM_KEY_VALUE_SMALL */) as i32);

    ASSERT_VALUE_BIG(hive, data_size);

    *data_cell = HvAllocateCell(hive, data_size, storage_type, 0 /* HCELL_NIL */);
    if *data_cell == 0 /* HCELL_NIL */ {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let cell_data: PCELL_DATA = HvGetCell(hive, *data_cell);
    if cell_data.is_null() {
        ASSERT(0);
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
    ChildList: *mut _CHILD_LIST
) -> i32 {
    let mut Count: u32 = 0;
    let mut CellData: *mut CELL_DATA = core::ptr::null_mut();
    let mut NewCell: HCELL_INDEX = 0;

    // PAGED_CODE();
    PAGED_CODE();

    // ASSERT((((LONG)Index) >= 0) && (Index <= ChildList->Count));
    // (omitted due to NDEBUG)

    Count = (*ChildList).Count - 1;

    if Count > 0 {
        CellData = HvGetCell(Hive, (*ChildList).List) as *mut CELL_DATA;
        if CellData.is_null() {
            return 0xC000009A; // STATUS_INSUFFICIENT_RESOURCES
        }

        // ASSERT(HvIsCellDirty(Hive, ChildList->List));
        // (omitted due to NDEBUG)
        // ASSERT(HvIsCellDirty(Hive, CellData->u.KeyList[Index]));
        // (omitted due to NDEBUG)

        while Index < Count {
            (*CellData).u.KeyList[Index as usize] = (*CellData).u.KeyList[(Index + 1) as usize];
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
            (*SrcListData).u.KeyList[Index],
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
            NewValue = (*DestListData).u.KeyList[Index];
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
