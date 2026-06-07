use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CELL_DATA, CM_KEY_NODE, CM_INDEX, HCELL_INDEX, HHIVE, PCELL_DATA, PCM_KEY_FAST_INDEX, PCM_KEY_INDEX, PCM_KEY_NODE, PHHIVE, HSTORAGE_TYPE_Stable as Stable, _HHIVE, _UNICODE_STRING};
use crate::glue::{HvGetCell, HvReleaseCell, HCELL_NIL, HBOOT_TYPE_SELF_HEAL, CM_KEY_INDEX_ROOT, CM_KEY_INDEX_LEAF, CM_KEY_FAST_LEAF, CM_KEY_HASH_LEAF};
use core::ffi::c_void;

extern "C" {
    // conditionally compiled in original (#if !defined(CMLIB_HOST) && !defined(_BLDR_)) pub static mut CmpSelfHeal: u8;   // BOOLEAN pub static mut CmpBootType: u32;  // ULONG
    unsafe fn CmpRemoveFastIndexKeyCell(FastIndex: *mut CM_KEY_FAST_INDEX, Index: u32, UpdateCount: u8) ;
    unsafe fn CmpRemoveIndexKeyCell(KeyIndex: *mut CM_KEY_INDEX, Index: u32) ;
    fn PAGED_CODE();
    fn HvMarkCellDirty(hive: PHHIVE, cell: HCELL_INDEX, dirty: i32);
    fn DPRINT1(format: *const i8, ...);
    unsafe fn CmpRemoveValueFromValueList(value_list_node: *mut CM_KEY_NODE, value_list_data: *mut CELL_DATA, index: u32) ;
}
extern "C" {
    fn CmIsSelfHealEnabled(FixHive: u8) -> u8;
    fn DbgPrint(format: *const i8, ...);
    unsafe fn CmpRemoveSubKeyInLeaf( Hive: PHHIVE, KeyNode: PCM_KEY_NODE, Leaf: PCM_KEY_INDEX, TargetKey: HCELL_INDEX, ) -> BOOLEAN;
    unsafe fn CmpRemoveSubkeyInRoot( Hive: PHHIVE, RootIndex: PCM_KEY_INDEX, TargetKey: HCELL_INDEX, ) -> BOOLEAN;
    fn ASSERT(cond: i32);
}
#[no_mangle]
pub unsafe extern "C" fn CmpRepairParentKey(
    Hive: PHHIVE,
    TargetKey: HCELL_INDEX,
    ParentKey: HCELL_INDEX,
    FixHive: BOOLEAN,
) -> BOOLEAN {
    let mut KeyIndex: PCM_KEY_INDEX;
    let mut KeyNode: PCM_KEY_NODE;
    let mut ParentRepaired: BOOLEAN;

    PAGED_CODE();

    // ASSERT(TargetKey != HCELL_NIL);

    ParentRepaired = 0;

    if CmIsSelfHealEnabled(FixHive) == 0 {
        DPRINT1(b"Self healing not possible\n\0".as_ptr() as *const i8);
        return ParentRepaired;
    }

    KeyNode = HvGetCell(Hive, ParentKey) as PCM_KEY_NODE;
    if KeyNode.is_null() {
        DPRINT1(b"Couldn't get the parent key node\n\0".as_ptr() as *const i8);
        return ParentRepaired;
    }

    KeyIndex = HvGetCell(Hive, (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize]) as PCM_KEY_INDEX;
    if KeyIndex.is_null() {
        DPRINT1(b"Couldn't get the key index from parent node\n\0".as_ptr() as *const i8);
        HvReleaseCell(Hive, ParentKey);
        return ParentRepaired;
        }

        if (*KeyIndex).Signature == CM_KEY_INDEX_ROOT {
            ParentRepaired = CmpRemoveSubkeyInRoot(Hive, KeyIndex, TargetKey);
        } else if ((*KeyIndex).Signature == CM_KEY_INDEX_LEAF)
            || ((*KeyIndex).Signature == CM_KEY_FAST_LEAF)
            || ((*KeyIndex).Signature == CM_KEY_HASH_LEAF)
        {
            ParentRepaired = CmpRemoveSubKeyInLeaf(Hive, KeyNode, KeyIndex, TargetKey);
        } else {
            // DPRINT removed
            ASSERT(0);
        }

        if ParentRepaired != 0 {
            HvMarkCellDirty(Hive, ParentKey, 0);
            (*KeyNode).SubKeyCounts[Stable as usize] -= 1;
            (*(*Hive).BaseBlock).BootType |= HBOOT_TYPE_SELF_HEAL;
            // DPRINT removed
        }

        HvReleaseCell(Hive, (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize]);
        HvReleaseCell(Hive, ParentKey);
        ParentRepaired
}
#[no_mangle]
pub unsafe extern "C" fn CmpRepairParentNode(
    Hive: PHHIVE,
    CurrentCell: HCELL_INDEX,
    ParentCell: HCELL_INDEX,
    CellData: PCELL_DATA,
    FixHive: i32,
) -> i32 {
    PAGED_CODE();
    if CmIsSelfHealEnabled(FixHive) == 0 {
        // DPRINT removed
        return 0;
    }
    HvMarkCellDirty(Hive, CurrentCell, 0);
    (*CellData).u.KeyNode.Parent = ParentCell;
    (*(*Hive).BaseBlock).BootType |= HBOOT_TYPE_SELF_HEAL;
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn CmpRepairKeyNodeSignature(
    Hive: *mut HHIVE,
    CurrentCell: u32,
    CellData: *mut CELL_DATA,
    FixHive: u8,
) -> u8 {
    // PAGED_CODE();

    if CmIsSelfHealEnabled(FixHive) == 0 {
        // DPRINT removed
        return 0u8;
    }

    HvMarkCellDirty(Hive, CurrentCell, 0u8);
    (*CellData).u.KeyNode.Signature = 0x6e4b6e48; // CM_KEY_NODE_SIGNATURE

    let base = (*Hive).BaseBlock;
    (*base).BootType |= 0x200; // HBOOT_TYPE_SELF_HEAL

    1u8
}
#[no_mangle] #[no_mangle]
#[no_mangle] pub unsafe extern "C" fn CmpRepairClassOfNodeKey(Hive: *mut HHIVE, CurrentCell: u32, CellData: *mut CELL_DATA, FixHive: i32) -> i32 {
    // PAGED_CODE();
    if CmIsSelfHealEnabled(FixHive) == 0 {
        // DPRINT1("Self healing not possible\n");
        return 0;
    }
    HvMarkCellDirty(Hive, CurrentCell, 0);
    (*CellData).u.KeyNode.Class = HCELL_NIL;
    (*CellData).u.KeyNode.ClassLength = 0;
    (*(*Hive).BaseBlock).BootType |= 0x8; // HBOOT_TYPE_SELF_HEAL
    1
}
#[no_mangle]
pub unsafe extern "C" fn CmpRepairValueListCount(
    hive: *mut HHIVE,
    current_cell: u32,
    list_count_index: u32,
    value_list_data: *mut CELL_DATA,
    fix_hive: i32,
) -> i32 {
    let value_list_node: *mut CM_KEY_NODE;

    // PAGED_CODE();

    /* Is self healing possible? */
    if CmIsSelfHealEnabled(fix_hive) == 0 {
        // DPRINT1("Self healing not possible\n");
        return 0;
    }

    /*
     * Obtain a node from the cell that we mark it as dirty.
     * The node is that of the current cell of which its
     * value list is being validated.
     */
    value_list_node = HvGetCell(hive, current_cell) as *mut CM_KEY_NODE;
    if value_list_node.is_null() {
        // DPRINT1("Could not get a node from the current cell\n");
        return 0;
    }

    /*
     * Mark the current cell and value list as dirty
     * as we will be making changes onto them.
     */
    HvMarkCellDirty(hive, current_cell, 0);
    HvMarkCellDirty(hive, (*value_list_node).__bindgen_anon_1.__bindgen_anon_1.ValueList.List, 0);

    /*
     * Now remove the value from the list and mark the
     * hive as in self healing mode.
     */
    CmpRemoveValueFromValueList(value_list_node, value_list_data, list_count_index);
    let base_block = (*hive).BaseBlock;
    if !base_block.is_null() {
        (*base_block).BootType |= HBOOT_TYPE_SELF_HEAL;
    }
    HvReleaseCell(hive, current_cell);
    return 1;
}

#[no_mangle]
pub unsafe extern "C" fn CmpRepairValueList(
    Hive: *mut HHIVE,
    CurrentCell: HCELL_INDEX,
    FixHive: BOOLEAN,
) -> BOOLEAN {
    let value_list_node: *mut CM_KEY_NODE;

    // PAGED_CODE();

    if CmIsSelfHealEnabled(FixHive) == 0 {
        DbgPrint(b"Self healing not possible\n\0".as_ptr() as *const i8);
        return 0;
    }

    value_list_node = HvGetCell(Hive, CurrentCell) as *mut CM_KEY_NODE;
    if value_list_node.is_null() {
        DbgPrint(b"Could not get a node from the current cell\n\0".as_ptr() as *const i8);
        return 0;
    }

    HvMarkCellDirty(Hive, CurrentCell, 0);
    (*value_list_node).__bindgen_anon_1.__bindgen_anon_1.ValueList.List = HCELL_NIL;
    (*value_list_node).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count = 0;

    (*(*Hive).BaseBlock).BootType |= HBOOT_TYPE_SELF_HEAL;
    HvReleaseCell(Hive, CurrentCell);
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn CmpRepairSubKeyCounts(
    Hive: *mut HHIVE,
    CurrentCell: HCELL_INDEX,
    Count: u32,
    CellData: *mut CELL_DATA,
    FixHive: u8,
) -> u8 {
    // PAGED_CODE();
    if CmIsSelfHealEnabled(FixHive) == 0 {
        // DPRINT1("Self healing not possible\n");
        return 0u8;
    }

    HvMarkCellDirty(Hive, CurrentCell, 0u8);
    const Stable: usize = 1; // Stable index
    (*CellData).u.KeyNode.SubKeyCounts[Stable] = Count;
    (*(*Hive).BaseBlock).BootType |= 0x00000002u32; // HBOOT_TYPE_SELF_HEAL
    return 1u8;
}
#[no_mangle]
pub unsafe extern "C" fn CmpRepairSubKeyList(
    Hive: PHHIVE,
    CurrentCell: HCELL_INDEX,
    CellData: PCELL_DATA,
    FixHive: i32,
) -> i32 {
    // PAGED_CODE();

    /* Is self healing possible? */
    if CmIsSelfHealEnabled(FixHive) == 0 {
        // DPRINT removed
        return 0;
    }

    /*
     * Mark the cell where we got the actual
     * cell data as dirty and fix the subkey
     * list.
     */
    HvMarkCellDirty(Hive, CurrentCell, 0);
    (*CellData).u.KeyNode.__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable] = HCELL_NIL;
    (*CellData).u.KeyNode.SubKeyCounts[Stable] = 0;

    /* Mark the hive as in self healing mode since we repaired it */
    let base_block = (*Hive).BaseBlock;
    (*base_block).BootType |= HBOOT_TYPE_SELF_HEAL;
    return 1;
}
