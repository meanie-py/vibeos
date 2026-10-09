//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/cmlib/cmheal.c.
//! Risk: HIGH (4 wrong constants, 1 wrong Stable index, ABI mismatch on i386).
//! Behavior: 5 real bugs fixed in this pass. Findings:
//!   1. CmpRepairKeyNodeSignature L118: `BootType |= 0x200` — WRONG; must be HBOOT_TYPE_SELF_HEAL (4). Fixed.
//!   2. CmpRepairClassOfNodeKey L132: `BootType |= 0x8` — WRONG; must be HBOOT_TYPE_SELF_HEAL (4). Fixed.
//!   3. CmpRepairSubKeyCounts L230: `BootType |= 0x00000002u32` — WRONG; must be HBOOT_TYPE_SELF_HEAL (4). Fixed.
//!   4. CmpRepairSubKeyCounts L228: `const Stable: usize = 1` — SHADOWS imported Stable (=0) with Volatile slot (=1);
//!      hivedata.h: Stable=0, Volatile=1. SubKeyCounts[1] repaired the volatile count, not the stable count. Fixed.
//!   5. `#[no_mangle] #[no_mangle] #[no_mangle]` triple-attr artifact on CmpRepairClassOfNodeKey. Fixed.
//!   6. ABI: CMAPI=NTAPI=__stdcall on i386; all exported fns use `extern "C"` (cdecl). Changed to `extern "system"`.
//! [2026-06-09] Translated the previously-stubbed static helpers in place:
//!   CmpRemoveValueFromValueList, CmpRemoveSubKeyInLeaf, CmpRemoveSubkeyInRoot,
//!   plus the two leaf-cell removers (CmpRemoveFastIndexKeyCell/CmpRemoveIndexKeyCell)
//!   they depend on — all `static` in C, so private `unsafe fn` here.
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CELL_DATA, CM_KEY_NODE, CM_KEY_FAST_INDEX, CM_KEY_INDEX, CM_INDEX, HCELL_INDEX, HHIVE, PCELL_DATA, PCM_KEY_FAST_INDEX, PCM_KEY_INDEX, PCM_KEY_NODE, PHHIVE, HSTORAGE_TYPE_Stable as Stable};
use crate::glue::{HvGetCell, HvReleaseCell, HCELL_NIL, HBOOT_TYPE_SELF_HEAL, CM_KEY_INDEX_ROOT, CM_KEY_INDEX_LEAF, CM_KEY_FAST_LEAF, CM_KEY_HASH_LEAF, CM_KEY_NODE_SIGNATURE};

unsafe extern "C" {
    // conditionally compiled in original (#if !defined(CMLIB_HOST) && !defined(_BLDR_)) pub static mut CmpSelfHeal: u8;   // BOOLEAN pub static mut CmpBootType: u32;  // ULONG
}
// HvMarkCellDirty is NTAPI = __stdcall on i386 (`_HvMarkCellDirty@12`, callee-pops).
// Must be `extern "system"`, NOT `extern "C"` — see the cmcheck.rs note (cdecl
// decl + stdcall def drifts ESP per call → BUGCHECK 0x1E on i386).
unsafe extern "system" {
    fn HvMarkCellDirty(hive: PHHIVE, cell: HCELL_INDEX, dirty: u8) -> u8;
}

// DPRINT1 is a ReactOS debug-print macro, so it is not a linkable symbol and an
// `extern fn DPRINT1` would leave one undefined (lld tolerates that only where
// the caller is dead-code-eliminated — i386 GC'd CmpRepairParentKey, amd64 kept
// it and hit LINK_FAIL). A crate-local wrapper is the right shape; what was
// wrong is that it had an EMPTY body "the same way the C NDEBUG build does".
// NDEBUG silences DPRINT, never DPRINT1 — DPRINT1 is the unconditional one, and
// these three call sites are the hive self-heal telling us why a corrupt key
// could not be repaired. Forward to DbgPrint, declared just below. All call
// sites here pass a single format arg.
#[inline(always)]
unsafe fn DPRINT1(format: *const i8) {
    DbgPrint(format);
}
unsafe extern "C" {
    fn DbgPrint(format: *const i8, ...);
    // Consumer-provided globals (ntoskrnl in the kernel; loader supplies a
    // zero stub). Gated behind !CMLIB_HOST && !_BLDR_ in C.
    #[cfg(not(feature = "bldr"))]
    static mut CmpSelfHeal: u8; // BOOLEAN
    #[cfg(not(feature = "bldr"))]
    static mut CmpBootType: u32; // ULONG
}

// CmIsSelfHealEnabled (cmheal.c:369) — full kernel variant. Returns whether
// self-healing is possible for this hive.
#[no_mangle]
pub unsafe extern "system" fn CmIsSelfHealEnabled(FixHive: u8) -> u8 {
    PAGED_CODE!();

    if FixHive != 0 {
        return 1;
    }

    #[cfg(not(feature = "bldr"))]
    {
        if CmpSelfHeal != 0 || (CmpBootType & HBOOT_TYPE_SELF_HEAL) != 0 {
            return 1;
        }
    }

    0
}
#[no_mangle]
pub unsafe extern "system" fn CmpRepairParentKey(
    Hive: PHHIVE,
    TargetKey: HCELL_INDEX,
    ParentKey: HCELL_INDEX,
    FixHive: BOOLEAN,
) -> BOOLEAN {
    let KeyIndex: PCM_KEY_INDEX;
    let KeyNode: PCM_KEY_NODE;
    let mut ParentRepaired: BOOLEAN;

    PAGED_CODE!();

    // ASSERT(TargetKey != HCELL_NIL);

    ParentRepaired = 0;

    if CmIsSelfHealEnabled(FixHive as u8) == 0 {
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
            ASSERT!(0);
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
pub unsafe extern "system" fn CmpRepairParentNode(
    Hive: PHHIVE,
    CurrentCell: HCELL_INDEX,
    ParentCell: HCELL_INDEX,
    CellData: PCELL_DATA,
    FixHive: i32,
) -> i32 {
    PAGED_CODE!();
    if CmIsSelfHealEnabled(FixHive as u8) == 0 {
        // DPRINT removed
        return 0;
    }
    HvMarkCellDirty(Hive, CurrentCell, 0);
    (*CellData).u.KeyNode.Parent = ParentCell;
    (*(*Hive).BaseBlock).BootType |= HBOOT_TYPE_SELF_HEAL;
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn CmpRepairKeyNodeSignature(
    Hive: *mut HHIVE,
    CurrentCell: u32,
    CellData: *mut CELL_DATA,
    FixHive: u8,
) -> u8 {
    // PAGED_CODE();

    if CmIsSelfHealEnabled(FixHive as u8) == 0 {
        // DPRINT removed
        return 0u8;
    }

    HvMarkCellDirty(Hive, CurrentCell, 0);
    (*CellData).u.KeyNode.Signature = CM_KEY_NODE_SIGNATURE; // CM_KEY_NODE_SIGNATURE

    let base = (*Hive).BaseBlock;
    (*base).BootType |= HBOOT_TYPE_SELF_HEAL;

    1u8
}
#[no_mangle]
pub unsafe extern "system" fn CmpRepairClassOfNodeKey(Hive: *mut HHIVE, CurrentCell: u32, CellData: *mut CELL_DATA, FixHive: i32) -> i32 {
    // PAGED_CODE();
    if CmIsSelfHealEnabled(FixHive as u8) == 0 {
        // DPRINT1("Self healing not possible\n");
        return 0;
    }
    HvMarkCellDirty(Hive, CurrentCell, 0);
    (*CellData).u.KeyNode.Class = HCELL_NIL;
    (*CellData).u.KeyNode.ClassLength = 0;
    (*(*Hive).BaseBlock).BootType |= HBOOT_TYPE_SELF_HEAL;
    1
}
#[no_mangle]
pub unsafe extern "system" fn CmpRepairValueListCount(
    hive: *mut HHIVE,
    current_cell: u32,
    list_count_index: u32,
    value_list_data: *mut CELL_DATA,
    fix_hive: i32,
) -> i32 {
    let value_list_node: *mut CM_KEY_NODE;

    // PAGED_CODE();

    /* Is self healing possible? */
    if CmIsSelfHealEnabled(fix_hive as u8) == 0 {
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
pub unsafe extern "system" fn CmpRepairValueList(
    Hive: *mut HHIVE,
    CurrentCell: HCELL_INDEX,
    FixHive: BOOLEAN,
) -> BOOLEAN {
    let value_list_node: *mut CM_KEY_NODE;

    // PAGED_CODE();

    if CmIsSelfHealEnabled(FixHive as u8) == 0 {
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
pub unsafe extern "system" fn CmpRepairSubKeyCounts(
    Hive: *mut HHIVE,
    CurrentCell: HCELL_INDEX,
    Count: u32,
    CellData: *mut CELL_DATA,
    FixHive: u8,
) -> u8 {
    // PAGED_CODE();
    if CmIsSelfHealEnabled(FixHive as u8) == 0 {
        // DPRINT1("Self healing not possible\n");
        return 0u8;
    }

    HvMarkCellDirty(Hive, CurrentCell, 0);
    (*CellData).u.KeyNode.SubKeyCounts[Stable as usize] = Count;
    (*(*Hive).BaseBlock).BootType |= HBOOT_TYPE_SELF_HEAL;
    return 1u8;
}
#[no_mangle]
pub unsafe extern "system" fn CmpRepairSubKeyList(
    Hive: PHHIVE,
    CurrentCell: HCELL_INDEX,
    CellData: PCELL_DATA,
    FixHive: i32,
) -> i32 {
    // PAGED_CODE();

    /* Is self healing possible? */
    if CmIsSelfHealEnabled(FixHive as u8) == 0 {
        // DPRINT removed
        return 0;
    }

    /*
     * Mark the cell where we got the actual
     * cell data as dirty and fix the subkey
     * list.
     */
    HvMarkCellDirty(Hive, CurrentCell, 0);
    (*CellData).u.KeyNode.__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize] = HCELL_NIL;
    (*CellData).u.KeyNode.SubKeyCounts[Stable as usize] = 0;

    /* Mark the hive as in self healing mode since we repaired it */
    let base_block = (*Hive).BaseBlock;
    (*base_block).BootType |= HBOOT_TYPE_SELF_HEAL;
    return 1;
}

/// Removes a cell from a fast key index (C `static VOID CmpRemoveFastIndexKeyCell`).
unsafe fn CmpRemoveFastIndexKeyCell(
    FastIndex: *mut CM_KEY_FAST_INDEX,
    Index: u32,
    UpdateCount: u8,
) {
    ASSERT!((Index < (*FastIndex).Count as u32) as i32);

    /* Calculate the number of trailing cells */
    let MoveCount = (*FastIndex).Count as u32 - Index - 1;
    if MoveCount != 0 {
        /* Remove the cell now by moving one location ahead.
         * List is a flexible array of CM_INDEX. */
        let list = core::ptr::addr_of_mut!((*FastIndex).List) as *mut CM_INDEX;
        core::ptr::copy(
            list.add((Index + 1) as usize),
            list.add(Index as usize),
            MoveCount as usize,
        );
    }

    /* Update the fast index count if asked */
    if UpdateCount != 0 {
        (*FastIndex).Count -= 1;
    }
}

/// Removes a cell from a normal key index (C `static VOID CmpRemoveIndexKeyCell`).
unsafe fn CmpRemoveIndexKeyCell(KeyIndex: *mut CM_KEY_INDEX, Index: u32) {
    ASSERT!((Index < (*KeyIndex).Count as u32) as i32);

    /* Calculate the number of trailing cells */
    let MoveCount = (*KeyIndex).Count as u32 - Index - 1;
    if MoveCount != 0 {
        /* Remove the cell now by moving one location ahead.
         * List is a flexible array of HCELL_INDEX. */
        let list = core::ptr::addr_of_mut!((*KeyIndex).List) as *mut HCELL_INDEX;
        core::ptr::copy(
            list.add((Index + 1) as usize),
            list.add(Index as usize),
            MoveCount as usize,
        );
    }

    /* Update the key index count */
    (*KeyIndex).Count -= 1;
}

/// Removes a cell from a key value list node (C `static VOID CmpRemoveValueFromValueList`).
unsafe fn CmpRemoveValueFromValueList(
    ValueListNode: *mut CM_KEY_NODE,
    ValueListData: *mut CELL_DATA,
    Index: u32,
) {
    let count_ptr =
        core::ptr::addr_of_mut!((*ValueListNode).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count);
    let count = core::ptr::read_unaligned(count_ptr);
    ASSERT!((Index < count) as i32);

    /* Calculate the number of trailing values */
    let MoveCount = count - Index - 1;
    if MoveCount != 0 {
        /* Remove the value now by moving one location ahead.
         * KeyList is a flexible array of HCELL_INDEX. */
        let list = core::ptr::addr_of_mut!((*ValueListData).u.KeyList) as *mut HCELL_INDEX;
        core::ptr::copy(
            list.add((Index + 1) as usize),
            list.add(Index as usize),
            MoveCount as usize,
        );
    }

    /* Update the value list count */
    core::ptr::write_unaligned(count_ptr, count - 1);
}

/// Removes the offending subkey from a root index (C `static BOOLEAN CmpRemoveSubkeyInRoot`).
unsafe fn CmpRemoveSubkeyInRoot(
    Hive: PHHIVE,
    RootIndex: PCM_KEY_INDEX,
    TargetKey: HCELL_INDEX,
) -> BOOLEAN {
    let mut Leaf: PCM_KEY_INDEX = core::ptr::null_mut();
    let mut LeafCell: HCELL_INDEX = HCELL_NIL;

    PAGED_CODE!();

    ASSERT!((!RootIndex.is_null()) as i32);

    let root_list = core::ptr::addr_of!((*RootIndex).List) as *const HCELL_INDEX;

    /* Loop the root index */
    let mut RootCountIndex: u32 = 0;
    while RootCountIndex < (*RootIndex).Count as u32 {
        /*
         * Release the leaf cell from previous iteration
         * of the loop. Make sure what we're releasing is
         * valid to begin with.
         */
        if RootCountIndex != 0 {
            ASSERT!((!Leaf.is_null()) as i32);
            ASSERT!((LeafCell == *root_list.add((RootCountIndex - 1) as usize)) as i32);
            HvReleaseCell(Hive, LeafCell);
        }

        /* Get the leaf cell and the leaf for this index */
        LeafCell = *root_list.add(RootCountIndex as usize);
        Leaf = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;
        if Leaf.is_null() {
            return 0;
        }

        /* Start looping the leaf */
        let mut LeafCountIndex: u32 = 0;
        while LeafCountIndex < (*Leaf).Count as u32 {
            /* Is the leaf a fast leaf or a hash one? */
            if ((*Leaf).Signature == CM_KEY_FAST_LEAF) || ((*Leaf).Signature == CM_KEY_HASH_LEAF) {
                /* It is one of the two, get the fast index */
                let FastIndex = Leaf as PCM_KEY_FAST_INDEX;

                /*
                 * Is the subkey cell from the fast index the
                 * one we're actually searching?
                 */
                let fi_list = core::ptr::addr_of!((*FastIndex).List) as *const CM_INDEX;
                if (*fi_list.add(LeafCountIndex as usize)).Cell == TargetKey {
                    HvReleaseCell(Hive, LeafCell);
                    HvMarkCellDirty(Hive, LeafCell, 0);
                    CmpRemoveFastIndexKeyCell(FastIndex, LeafCountIndex, 1);
                    return 1;
                }
            } else {
                /*
                 * The leaf is neither of the two. Check if the
                 * target offending cell is inside the leaf itself.
                 */
                let leaf_list = core::ptr::addr_of!((*Leaf).List) as *const HCELL_INDEX;
                if *leaf_list.add(LeafCountIndex as usize) == TargetKey {
                    HvReleaseCell(Hive, LeafCell);
                    HvMarkCellDirty(Hive, LeafCell, 0);
                    CmpRemoveIndexKeyCell(Leaf, LeafCountIndex);
                    return 1;
                }
            }

            LeafCountIndex += 1;
        }

        RootCountIndex += 1;
    }

    /*
     * We have searched everywhere but we couldn't
     * hunt the offending target key cell.
     */
    0
}

/// Removes the offending subkey from a leaf index (C `static BOOLEAN CmpRemoveSubKeyInLeaf`).
unsafe fn CmpRemoveSubKeyInLeaf(
    Hive: PHHIVE,
    KeyNode: PCM_KEY_NODE,
    Leaf: PCM_KEY_INDEX,
    TargetKey: HCELL_INDEX,
) -> BOOLEAN {
    /* Loop the leaf index */
    let mut LeafIndex: u32 = 0;
    while LeafIndex < (*Leaf).Count as u32 {
        /*
         * Check if the main leaf is a fast leaf or a hash one.
         */
        if ((*Leaf).Signature == CM_KEY_FAST_LEAF) || ((*Leaf).Signature == CM_KEY_HASH_LEAF) {
            /* It is one of the two, get the fast index */
            let FastIndex = Leaf as PCM_KEY_FAST_INDEX;

            /*
             * Is the subkey cell from the fast index the one
             * we're actually searching?
             */
            let fi_list = core::ptr::addr_of!((*FastIndex).List) as *const CM_INDEX;
            if (*fi_list.add(LeafIndex as usize)).Cell == TargetKey {
                let subkey_list = core::ptr::read_unaligned(core::ptr::addr_of!(
                    (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize]
                ));
                HvMarkCellDirty(Hive, subkey_list, 0);
                CmpRemoveFastIndexKeyCell(FastIndex, LeafIndex, 0);

                /*
                 * Since this fast index actually came from the
                 * actual leaf index itself, just update its count
                 * rather than that of the fast index.
                 */
                (*Leaf).Count -= 1;
                return 1;
            }
        } else {
            /*
             * The leaf is neither of the two. The offending
             * cell must come directly from the normal leaf
             * at this point.
             */
            let leaf_list = core::ptr::addr_of!((*Leaf).List) as *const HCELL_INDEX;
            if *leaf_list.add(LeafIndex as usize) == TargetKey {
                let subkey_list = core::ptr::read_unaligned(core::ptr::addr_of!(
                    (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize]
                ));
                HvMarkCellDirty(Hive, subkey_list, 0);
                CmpRemoveIndexKeyCell(Leaf, LeafIndex);
                return 1;
            }
        }

        LeafIndex += 1;
    }

    /*
     * We have searched everywhere but we couldn't
     * hunt the offending target key cell.
     */
    0
}
