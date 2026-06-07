use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{CM_INDEX, CM_KEY_FAST_INDEX, CM_KEY_INDEX, CmpAddSubKey, CmpCompareCompressedName, CmpCompressedNameSize, CmpCopyCompressedName, CmpMarkIndexDirty, CmpRemoveSubKey, HCELL_INDEX, HSTORAGE_TYPE, LPWSTR, PCM_INDEX, PCM_KEY_FAST_INDEX, PCM_KEY_INDEX, PCM_KEY_NODE, PCUNICODE_STRING, PHCELL_INDEX, PHHIVE, Root, RtlCompareUnicodeString, Stable, Type, Volatile};
use core::ffi::c_void;

extern "C" {
    #[no_mangle] pub unsafe extern "system" fn CmpDoCompareKeyName(hive: *mut HHIVE, name: *const _UNICODE_STRING, cell: HCELL_INDEX) -> i32;
    #[no_mangle] pub unsafe extern "system" fn CmpFindSubKeyInRoot( hive: *mut _HHIVE, index: *const CM_KEY_INDEX, name: *const _UNICODE_STRING, subkey: *mut u32, ) -> u32;
    #[no_mangle] pub unsafe extern "system" fn CmpFindSubKeyInLeaf( hive: *mut _HHIVE, index: *const CM_KEY_INDEX, name: *const _UNICODE_STRING, subkey: *mut u32, ) -> u32;
    unsafe fn CmpFindSubKeyByHash( hive: *mut _HHIVE, index: *const CM_KEY_FAST_INDEX, name: *const _UNICODE_STRING, ) -> u32;
    fn HvGetCell(hive: *mut _HHIVE, cell: u32) -> *mut core::ffi::c_void;
    fn HvReleaseCell(hive: *mut _HHIVE, cell: u32);
}
extern "system" {
    fn HvAllocateCell(Hive: PHHIVE, Size: u32, Type: HSTORAGE_TYPE, SourceCell: HCELL_INDEX) -> HCELL_INDEX;
    fn HvFreeCell(Hive: PHHIVE, Cell: HCELL_INDEX);
    fn HvMarkCellDirty(Hive: PHHIVE, Cell: HCELL_INDEX, Unk: BOOL);
    fn HvGetCellSize(Hive: PHHIVE, Cell: *mut CM_KEY_INDEX) -> u32;
    fn HvReallocateCell(Hive: PHHIVE, Cell: HCELL_INDEX, NewSize: u32) -> HCELL_INDEX;
    fn RtlUpcaseUnicodeChar(ch: u16) -> u16;
    #[no_mangle] pub unsafe extern "system" fn CmpComputeHashKey(arg1: u32, arg2: *const _UNICODE_STRING, arg3: u8) -> u32;
    fn CmpDoCompareKeyName(hive: *mut HHIVE, name: *const _UNICODE_STRING, cell: HCELL_INDEX) -> i32;
    #[no_mangle] pub unsafe extern "system" fn CmpDoFindSubKeyByNumber( Hive: *mut HHIVE, Index: *mut CM_KEY_INDEX, Number: u32, ) -> u32;
    #[no_mangle] pub unsafe extern "system" fn CmpCompareInIndex( Hive: *mut HHIVE, SearchName: *const _UNICODE_STRING, Index: u32, KeyIndex: *const CM_KEY_INDEX, SubKey: *mut HCELL_INDEX, ) -> i32;
    fn CmpFindSubKeyInRoot( hive: *mut HHIVE, index_key: *mut CM_KEY_INDEX, name: *const _UNICODE_STRING, leaf_cell: *mut HCELL_INDEX, ) -> u32;
    #[no_mangle] pub unsafe extern "system" fn CmpSplitLeaf( hive: *mut HHIVE, cell: HCELL_INDEX, root_select: u32, storage_type: HSTORAGE_TYPE, ) -> HCELL_INDEX;
    fn CmpFindSubKeyInLeaf( hive: *mut _HHIVE, leaf: *mut CM_KEY_INDEX, name: *const _UNICODE_STRING, child: *mut HCELL_INDEX ) -> u32;
}
) -> u32 {
    // ASSERT((Name->Length == 0) || (AllowSeparators) || (Name->Buffer[0] != OBJ_NAME_PATH_SEPARATOR));
    debug_assert!(
        (*Name).Length == 0
            || AllowSeparators != 0
            || *(*Name).Buffer != OBJ_NAME_PATH_SEPARATOR as u16
    );

    // if (!Name->Length) return Hash;
    if (*Name).Length == 0 {
        return Hash;
    }

    // Set the buffer and loop every character
    let mut Cp: *mut u16 = (*Name).Buffer;
    let mut i: u32 = 0;
    while i < (*Name).Length as u32 {
        // ASSERT(AllowSeparators || (*Cp != OBJ_NAME_PATH_SEPARATOR));
        debug_assert!(AllowSeparators != 0 || *Cp != OBJ_NAME_PATH_SEPARATOR as u16);

        let Value: u32;
        // if (*Cp >= L'a')
        if *Cp >= 0x61u16 {
            // if (*Cp < L'z')
            if *Cp <= 0x7Au16 {
                // Value = *Cp - L'a' + L'A';
                Value = ((*Cp - 0x61u16 + 0x41u16) as u32);
            } else {
                // Value = RtlUpcaseUnicodeChar(*Cp);
                Value = RtlUpcaseUnicodeChar(*Cp) as u32;
            }
        } else {
            // Value = *Cp;
            Value = *Cp as u32;
        }

        // Hash *= 37;
        Hash = Hash.wrapping_mul(37);
        // Hash += Value;
        Hash = Hash.wrapping_add(Value);

        // i += sizeof(WCHAR), Cp++
        i += core::mem::size_of::<u16>() as u32;
        Cp = Cp.add(1);
    }

    // return Hash
    Hash
}

#[no_mangle]
pub unsafe extern "system" fn CmpFindSubKeyByNumber(
    Hive: *mut HHIVE,
    Node: *mut CM_KEY_NODE,
    Number: u32,
) -> u32 {
    let mut Index: *mut CM_KEY_INDEX = core::ptr::null_mut();
    let mut Result: u32 = HCELL_NIL;

    // Check if it's in the stable list
    if Number < (*Node).SubKeyCounts[Stable as usize] {
        // Get the actual key index
        Index = HvGetCell(Hive, (*Node).SubKeyLists[Stable as usize]) as *mut CM_KEY_INDEX;
        if Index.is_null() {
            return HCELL_NIL;
        }

        // Do a search inside it
        Result = CmpDoFindSubKeyByNumber(Hive, Index, Number);

        // Release the cell and return the result
        HvReleaseCell(Hive, (*Node).SubKeyLists[Stable as usize]);
        return Result;
    } else if (*Hive).StorageTypeCount > Volatile {
        // It's in the volatile list
        let Number = Number - (*Node).SubKeyCounts[Stable as usize];
        if Number < (*Node).SubKeyCounts[Volatile as usize] {
            // Get the actual key index
            Index = HvGetCell(Hive, (*Node).SubKeyLists[Volatile as usize]) as *mut CM_KEY_INDEX;
            if Index.is_null() {
                return HCELL_NIL;
            }

            // Do a search inside it
            Result = CmpDoFindSubKeyByNumber(Hive, Index, Number);

            // Release the cell and return the result
            HvReleaseCell(Hive, (*Node).SubKeyLists[Volatile as usize]);
            return Result;
        }
    }

    // Nothing was found
    return HCELL_NIL;
}

// Forward declarations for callees (not yet translated, caller will deduplicate)

#[no_mangle]
pub unsafe extern "system" fn CmpFindSubKeyByName(
    hive: *mut _HHIVE,
    parent: *mut CM_KEY_NODE,
    search_name: *const _UNICODE_STRING,
) -> HCELL_INDEX {
    let mut i: u32;
    let mut index_root: *mut CM_KEY_INDEX;
    let mut sub_key: HCELL_INDEX = 0;
    let mut cell_to_release: HCELL_INDEX;
    let mut found: u32;

    for i in 0..(*hive).StorageTypeCount {
        if (*parent).SubKeyCounts[i as usize] != 0 {
            index_root = HvGetCell(hive, (*parent).SubKeyLists[i as usize]) as *mut CM_KEY_INDEX;
            if index_root.is_null() {
                return HCELL_NIL;
            }
            cell_to_release = (*parent).SubKeyLists[i as usize];

            if (*index_root).Signature == CM_KEY_INDEX_ROOT {
                found = CmpFindSubKeyInRoot(hive, index_root, search_name, &mut sub_key);

                debug_assert!(cell_to_release != HCELL_NIL);
                HvReleaseCell(hive, cell_to_release);

                if found & INVALID_INDEX != 0 {
                    break;
                }

                if sub_key == HCELL_NIL {
                    continue;
                }
                cell_to_release = sub_key;
                index_root = HvGetCell(hive, sub_key) as *mut CM_KEY_INDEX;
            }

            debug_assert!(
                (*index_root).Signature == CM_KEY_INDEX_LEAF
                    || (*index_root).Signature == CM_KEY_FAST_LEAF
                    || (*index_root).Signature == CM_KEY_HASH_LEAF
            );

            if (*index_root).Signature != CM_KEY_HASH_LEAF {
                found = CmpFindSubKeyInLeaf(hive, index_root, search_name, &mut sub_key);

                debug_assert!(cell_to_release != HCELL_NIL);
                HvReleaseCell(hive, cell_to_release);

                if found & INVALID_INDEX != 0 {
                    break;
                }
            } else {
                sub_key = CmpFindSubKeyByHash(
                    hive,
                    index_root as *const CM_KEY_FAST_INDEX,
                    search_name,
                );

                debug_assert!(cell_to_release != HCELL_NIL);
                HvReleaseCell(hive, cell_to_release);
            }

            if sub_key != HCELL_NIL {
                return sub_key;
            }
        }
    }

    return HCELL_NIL;
}

#[no_mangle]
pub unsafe extern "system" fn CmpAddToLeaf(
    hive: *mut _HHIVE,
    leaf_cell: HCELL_INDEX,
    new_key: HCELL_INDEX,
    name: *const _UNICODE_STRING,
) -> HCELL_INDEX {
    let leaf: *mut CM_KEY_INDEX;
    let fast_leaf: *mut CM_KEY_FAST_INDEX;
    let mut size: u32;
    let old_size: u32;
    let entry_size: usize;
    let mut i: u32;
    let mut j: u32;
    let mut new_cell: HCELL_INDEX;
    let mut child: HCELL_INDEX = 0;
    let result: i32;

    HvMarkCellDirty(hive, leaf_cell, 0);

    leaf = HvGetCell(hive, leaf_cell) as *mut CM_KEY_INDEX;
    if leaf.is_null() {
        return HCELL_NIL;
    }

    HvReleaseCell(hive, leaf_cell);

    if (*leaf).Signature == CM_KEY_INDEX_LEAF {
        fast_leaf = core::ptr::null_mut();
        entry_size = core::mem::size_of::<HCELL_INDEX>();
    } else {
        // Sanity check: CM_KEY_FAST_LEAF or CM_KEY_HASH_LEAF assumed
        fast_leaf = leaf as *mut CM_KEY_FAST_INDEX;
        entry_size = core::mem::size_of::<CM_INDEX>();
    }

    old_size = HvGetCellSize(hive, leaf as *mut core::ffi::c_void);
    size = old_size;
    size -= (entry_size as u32) * (*leaf).Count
        + (core::mem::offset_of!(CM_KEY_INDEX, List) as u32);

    new_cell = leaf_cell;

    if (size / entry_size as u32) < 1 {
        size = old_size + old_size / 2;
        if size < (old_size + entry_size as u32) {
            size = old_size + entry_size as u32;
        }

        new_cell = HvReallocateCell(hive, leaf_cell, size);
        if new_cell == HCELL_NIL {

#[no_mangle]
pub unsafe extern "system" fn CmpSelectLeaf(
    Hive: *mut HHIVE,
    KeyNode: *mut CM_KEY_NODE,
    Name: *const _UNICODE_STRING,
    Type: HSTORAGE_TYPE,
    RootCell: *mut *mut HCELL_INDEX,
) -> HCELL_INDEX {

    unsafe {
        let mut IndexKey: *mut CM_KEY_INDEX;
        let mut LeafKey: *mut CM_KEY_INDEX;
        let mut FastLeaf: *mut CM_KEY_FAST_INDEX;
        let mut LeafCell: HCELL_INDEX;
        let mut CurrentCell: HCELL_INDEX;
        let mut SubKeyIndex: u32;
        let mut Result: i32;

        // Mark it as dirty
        HvMarkCellDirty(Hive, (*KeyNode).SubKeyLists[Type as usize], 0);

        // Get the cell
        IndexKey = HvGetCell(Hive, (*KeyNode).SubKeyLists[Type as usize]) as *mut CM_KEY_INDEX;

        // Check if we've got a valid key
        if IndexKey.is_null() {
            // Should not happen!
            debug_assert!(!IndexKey.is_null());
            return 0; // HCELL_NIL
        }

        // Sanity check
        debug_assert!((*IndexKey).Signature == CM_KEY_INDEX_ROOT);

        loop {
            // Find subkey
            SubKeyIndex = CmpFindSubKeyInRoot(
                Hive,
                IndexKey,
                Name,
                core::ptr::addr_of_mut!(LeafCell),
            );

            // Make sure we found something valid
            if SubKeyIndex & 0x80000000 != 0 {
                return 0; // HCELL_NIL
            } // INVALID_INDEX

            // Try to fit it into the LeafCell, if it was found
            if LeafCell != 0 {
                // Get the leaf key
                LeafKey = HvGetCell(Hive, LeafCell) as *mut CM_KEY_INDEX;

                // Check for failure
                if LeafKey.is_null() {
                    return 0; // HCELL_NIL
                }

                // Check if it fits into this leaf and break
                if (*LeafKey).Count < CmpMaxIndexPerHblock {
                    // Fill in the result and return it
                    *RootCell = core::ptr::addr_of_mut!((*IndexKey).List[SubKeyIndex as usize]);
                    return LeafCell;
                }

                // It didn't fit, so proceed to splitting
            } else {
                // Get the leaf cell at the very end
                LeafCell = (*IndexKey).List[SubKeyIndex as usize];
                LeafKey = HvGetCell(Hive, LeafCell) as *mut CM_KEY_INDEX;

                // Return an error in case of problems
                if LeafKey.is_null() {
                    return 0; // HCELL_NIL
                }

                // Choose the cell to search from depending on the key type
                if ((*LeafKey).Signature == CM_KEY_FAST_LEAF)
                    || ((*LeafKey).Signature == CM_KEY_HASH_LEAF)
                {
                    FastLeaf = LeafKey as *mut CM_KEY_FAST_INDEX;
                    CurrentCell = (*FastLeaf).List[0].Cell;
                } else {
                    // Make sure it's an index leaf
                    debug_assert!((*LeafKey).Signature == CM_KEY_INDEX_LEAF);
                    CurrentCell = (*LeafKey).List[0];
                }

                // Do a name compare
                Result = CmpDoCompareKeyName(Hive, Name, CurrentCell);

                // Check for failure
                if Result == 2 {
                    return 0; // HCELL_NIL
                }

                // Names can't be equal, ensure that
                debug_assert!(Result != 0);

                // Check if it's above
                if Result >= 0 {
                    // Get the cell in the index
                    LeafCell = (*IndexKey).List[SubKeyIndex as usize];
                    LeafKey = HvGetCell(Hive, LeafCell) as *mut CM_KEY_INDEX;

                    // Return an error in case of problems
                    if LeafKey.is_null() {
                        return 0; // HCELL_NIL
                    }

                    // Check if it fits into this leaf
                    if (*LeafKey).Count < CmpMaxIndexPerHblock {
                        // Fill in the result and return the cell
                        *RootCell = core::ptr::addr_of_mut!((*IndexKey).List[SubKeyIndex as usize]);
                        return LeafCell;
                    }

                    // No, it doesn't fit, check the next adjacent leaf
                    if (SubKeyIndex + 1) <

