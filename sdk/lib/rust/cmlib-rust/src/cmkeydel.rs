use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CmpFreeSecurityDescriptor, CmpFreeValue, CmpMarkIndexDirty, CmpMarkValueDataDirty, CmpRemoveSubKey, HCELL_INDEX, PCELL_DATA, PCM_KEY_NODE, PCM_KEY_SECURITY, PHHIVE, HSTORAGE_TYPE_Stable as Stable, HSTORAGE_TYPE_Volatile as Volatile};

// Hv* with correct per-fn ABI + KEY_HIVE_EXIT, from the shared glue.
use crate::glue::{HvFreeCell, HvGetCell, HvMarkCellDirty, HvReleaseCell, KEY_HIVE_ENTRY, KEY_HIVE_EXIT};

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate.
// The #define NDEBUG is a preprocessor directive that affects compilation
// and does not translate to runtime Rust code.

#[no_mangle]
pub unsafe extern "system" fn CmpMarkKeyDirty(
    Hive: PHHIVE,
    Cell: HCELL_INDEX,
    CheckNoSubkeys: BOOLEAN,
) -> BOOLEAN {
    let CellData: PCM_KEY_NODE;
    let _SecurityData: PCM_KEY_SECURITY;
    let _ListData: PCELL_DATA;
    let _ValueData: PCELL_DATA;
    let mut i: ULONG;

    CellData = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
    if CellData.is_null() {
        return 0u8;
    }

    if CheckNoSubkeys != 0 {
    }

    // The flag literals here were PERMUTED (0x4 for KEY_HIVE_EXIT, 0x2 for KEY_HIVE_ENTRY, and
    // 0x1 for KEY_HIVE_EXIT in CmpFreeKeyBody): an exit node in the master hive was walked
    // as an ordinary key, reached NIL cells, and unloading a hive bugchecked 0x1E.
    if ((*CellData).Flags & KEY_HIVE_EXIT) != 0 {
        HvReleaseCell(Hive, Cell);
        return 1u8;
    }

    HvMarkCellDirty(Hive, Cell, 0u8);
    HvReleaseCell(Hive, Cell);

    if (*CellData).Class != 0xFFFFFFFF {
        HvMarkCellDirty(Hive, (*CellData).Class, 0u8);
    }

    if (*CellData).Security != 0xFFFFFFFF {
        HvMarkCellDirty(Hive, (*CellData).Security, 0u8);

        let SecurityData = HvGetCell(Hive, (*CellData).Security) as PCM_KEY_SECURITY;
        HvReleaseCell(Hive, (*CellData).Security);

        HvMarkCellDirty(Hive, (*SecurityData).Flink, 0u8);
        HvMarkCellDirty(Hive, (*SecurityData).Blink, 0u8);
    }

    if (*CellData).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count > 0 {
        HvMarkCellDirty(Hive, (*CellData).__bindgen_anon_1.__bindgen_anon_1.ValueList.List, 0u8);

        let ListData = HvGetCell(Hive, (*CellData).__bindgen_anon_1.__bindgen_anon_1.ValueList.List) as PCELL_DATA;
        HvReleaseCell(Hive, (*CellData).__bindgen_anon_1.__bindgen_anon_1.ValueList.List);

        i = 0;
        while i < (*CellData).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count {
            HvMarkCellDirty(Hive, *(core::ptr::addr_of!((*ListData).u.KeyList) as *const HCELL_INDEX).add(i as usize), 0u8);

            let ValueData = HvGetCell(Hive, *(core::ptr::addr_of!((*ListData).u.KeyList) as *const HCELL_INDEX).add(i as usize)) as PCELL_DATA;
            HvReleaseCell(Hive, *(core::ptr::addr_of!((*ListData).u.KeyList) as *const HCELL_INDEX).add(i as usize));

            if CmpMarkValueDataDirty(Hive, core::ptr::addr_of_mut!((*ValueData).u.KeyValue)) == 0 {
                return 0u8;
            }
            i += 1;
        }
    }

    if ((*CellData).Flags & KEY_HIVE_ENTRY) != 0 {
        return 1u8;
    }

    if CmpMarkIndexDirty(Hive, (*CellData).Parent, Cell) == 0 {
        return 0u8;
    }

    HvMarkCellDirty(Hive, (*CellData).Parent, 0u8);
    return 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn CmpFreeKeyBody(
    Hive: PHHIVE,
    Cell: HCELL_INDEX,
) -> u8 {

    let CellData: PCM_KEY_NODE;

    CellData = HvGetCell(Hive, Cell) as PCM_KEY_NODE;

    if ((*CellData).Flags & KEY_HIVE_EXIT) == 0 {
        if (*CellData).Security != 0xFFFFFFFF {
            HvFreeCell(Hive, (*CellData).Security);
        }

        if (*CellData).ClassLength > 0 {
            HvFreeCell(Hive, (*CellData).Class);
        }
    }

    HvReleaseCell(Hive, Cell);
    HvFreeCell(Hive, Cell);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CmpFreeKeyByCell(
    Hive: PHHIVE,
    Cell: HCELL_INDEX,
    Unlink: BOOLEAN,
) -> NTSTATUS {
    let CellData: PCM_KEY_NODE;
    let ParentData: PCM_KEY_NODE;
    let ListData: PCELL_DATA;
    let mut i: ULONG;
    let mut Result: BOOLEAN;

    CmpMarkKeyDirty(Hive, Cell, 1u8);

    CellData = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
    HvReleaseCell(Hive, Cell);

    if Unlink != 0 {
        Result = CmpRemoveSubKey(Hive, (*CellData).Parent, Cell);
        if Result == 0 {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        ParentData = HvGetCell(Hive, (*CellData).Parent) as PCM_KEY_NODE;
        HvReleaseCell(Hive, (*CellData).Parent);

        if (*ParentData).SubKeyCounts[Stable as usize] + (*ParentData).SubKeyCounts[Volatile as usize] == 0 {
            (*ParentData).MaxNameLen = 0;
            (*ParentData).MaxClassLen = 0;
        }
    }

    if ((*CellData).Flags & KEY_HIVE_EXIT) == 0 {
        if (*CellData).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count > 0 {
            ListData = HvGetCell(Hive, (*CellData).__bindgen_anon_1.__bindgen_anon_1.ValueList.List) as PCELL_DATA;
            HvReleaseCell(Hive, (*CellData).__bindgen_anon_1.__bindgen_anon_1.ValueList.List);

            i = 0;
            while i < (*CellData).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count {
                Result = CmpFreeValue(Hive, *(core::ptr::addr_of!((*ListData).u.KeyList) as *const HCELL_INDEX).add(i as usize));
                ntkrnl_sys::ASSERT!(Result != 0);
                i += 1;
            }

            HvFreeCell(Hive, (*CellData).__bindgen_anon_1.__bindgen_anon_1.ValueList.List);
        }

        CmpFreeSecurityDescriptor(Hive, Cell);
    }

    if CmpFreeKeyBody(Hive, Cell) == 0 {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    STATUS_SUCCESS
}


// ============================================================================
