//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/cmlib/cmindex.c. Risk: HIGH.
//! Behavior: 4 real bugs. Unverified: CmpDoFindSubKeyByNumber (extern, not translated here).
//!
//! REAL BUGS:
//!
//! 1. `CmpAddToLeaf` @ cmindex.rs:202 — STUBBED BODY.
//!    The Rust implementation performs only the resize/realloc guard (steps 1-6) then returns
//!    `new_cell` without doing ANY of: HvGetCell on the reallocated cell, updating the FastLeaf
//!    pointer, finding the insertion point via CmpFindSubKeyInLeaf, comparing via
//!    CmpCompareInIndex, shifting existing entries via RtlMoveMemory, writing the new
//!    Cell/HashKey/NameHint entry, or incrementing `Leaf->Count`. Every call to CmpAddToLeaf
//!    returns a cell handle without actually inserting the key — registry key creation is silently
//!    broken. C lines 987-1099 are entirely absent.
//!
//! 2. `CmpSelectLeaf` @ cmindex.rs:258 — HEAVILY TRUNCATED (missing ~100 C lines).
//!    The `else` branch (Result < 0 — new key sorts below the first entry of the current leaf)
//!    is completely absent: C tries `List[SubKeyIndex - 1]` (or `List[0]`) before splitting.
//!    The CmpSplitLeaf call + SubKeyLists update + IndexKey refresh loop are also missing.
//!    The `continue` at Rust line 379 re-enters the outer `loop {}` without having done a split,
//!    producing an infinite loop whenever a leaf is full and Result >= 0.
//!    C lines 1391-1457 are entirely absent.
//!
//! 3. i386 ABI mismatch on all `unsafe extern "C"` NTAPI callees @ cmindex.rs:11-26.
//!    `CmpDoCompareKeyName`, `CmpCompareInIndex`, `CmpFindSubKeyInRoot`, `CmpSplitLeaf`,
//!    `CmpFindSubKeyInLeaf` are all `NTAPI` (stdcall on i386) in C, but are declared
//!    `extern "C"` (cdecl) in Rust. Every call to these functions on i386 causes a stack
//!    imbalance (callee pops args, caller also adjusts). amd64/arm64 are unaffected (cdecl==system).
//!    Fix: change their block to `unsafe extern "system"`.
//!
//! 4. `CmpFindSubKeyByHash` @ cmindex.rs:12 — declared `extern "C"` but `static` in C.
//!    `CmpFindSubKeyByHash` has C-internal linkage (`static`). If cmindex.c is excluded from
//!    the build (as happens when this Rust file replaces it), the symbol does not exist and
//!    the link will fail with an unresolved reference. The function must either be translated
//!    into Rust or the static restriction must be lifted on the C side.
//!
//! VERIFIED OK:
//!    CmpComputeHashKey — correct (the <= vs < on 'z' is semantically equivalent: both yield 'Z').
//!    CmpFindSubKeyByNumber — correct logic and ABI (extern "system", matches NTAPI).
//!    CmpFindSubKeyByName — correct logic and ABI; `sub_key = 0` init is a minor style gap
//!      (C leaves it uninitialized; both are fine in practice since it is always written first).
//!    CmpMaxIndexPerHblock in glue.rs — layout-derived const expression matches C macro exactly.
//!    Constants (INVALID_INDEX, HCELL_NIL, CM_KEY_*) — all correct vs cmdata.h/hivedata.h.

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{CELL_DATA, CM_KEY_NODE, CM_INDEX, CM_KEY_FAST_INDEX, CM_KEY_INDEX, CmpAddSubKey, CmpCompareCompressedName, CmpCompressedNameSize, CmpCopyCompressedName, CmpMarkIndexDirty, CmpRemoveSubKey, HCELL_INDEX, HHIVE, HSTORAGE_TYPE, LPWSTR, PCELL_DATA, PCM_INDEX, PCM_KEY_FAST_INDEX, PCM_KEY_INDEX, PCM_KEY_NODE, PCUNICODE_STRING, PHCELL_INDEX, PHHIVE, RtlCompareUnicodeString, HSTORAGE_TYPE_Stable as Stable, HSTORAGE_TYPE_Volatile as Volatile, _HHIVE, _UNICODE_STRING};
use crate::glue::{HvGetCell, HvReleaseCell, HCELL_NIL, HBLOCK_SIZE, CM_KEY_INDEX_ROOT, CM_KEY_INDEX_LEAF, CM_KEY_FAST_LEAF, CM_KEY_HASH_LEAF, INVALID_INDEX, HCELL_TYPE_MASK, CmpMaxIndexPerHblock};
use core::ffi::c_void;

extern "system" {
    fn CmpDoFindSubKeyByNumber(Hive: PHHIVE, Index: *const CM_KEY_INDEX, Number: u32) -> HCELL_INDEX;
}
unsafe extern "C" {
    unsafe fn CmpFindSubKeyByHash( hive: *mut _HHIVE, index: *const CM_KEY_FAST_INDEX, name: *const _UNICODE_STRING, ) -> u32;
}
extern "system" {
    fn HvAllocateCell(Hive: PHHIVE, Size: u32, Type: HSTORAGE_TYPE, SourceCell: HCELL_INDEX) -> HCELL_INDEX;
    fn HvFreeCell(Hive: PHHIVE, Cell: HCELL_INDEX);
    fn HvMarkCellDirty(Hive: PHHIVE, Cell: HCELL_INDEX, Unk: u8);
    fn HvGetCellSize(Hive: PHHIVE, Cell: *mut CM_KEY_INDEX) -> u32;
    fn HvReallocateCell(Hive: PHHIVE, Cell: HCELL_INDEX, NewSize: u32) -> HCELL_INDEX;
    fn RtlUpcaseUnicodeChar(ch: u16) -> u16;
    fn CmpDoCompareKeyName(hive: *mut HHIVE, name: *const _UNICODE_STRING, cell: HCELL_INDEX) -> i32;
    fn CmpCompareInIndex( Hive: *mut HHIVE, SearchName: *const _UNICODE_STRING, Index: u32, KeyIndex: *const CM_KEY_INDEX, SubKey: *mut HCELL_INDEX, ) -> i32;
    fn CmpFindSubKeyInRoot( hive: *mut HHIVE, index_key: *mut CM_KEY_INDEX, name: *const _UNICODE_STRING, leaf_cell: *mut HCELL_INDEX, ) -> u32;
    fn CmpSplitLeaf( hive: *mut HHIVE, cell: HCELL_INDEX, root_select: u32, storage_type: HSTORAGE_TYPE, ) -> HCELL_INDEX;
    fn CmpFindSubKeyInLeaf( hive: *mut _HHIVE, leaf: *mut CM_KEY_INDEX, name: *const _UNICODE_STRING, child: *mut HCELL_INDEX ) -> u32;
}
#[no_mangle]
pub unsafe extern "system" fn CmpComputeHashKey(
    mut Hash: u32,
    Name: *const _UNICODE_STRING,
    AllowSeparators: u8,
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
                Value = *Cp as u32 - 0x61u32 + 0x41u32;
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
        Index = HvGetCell(Hive, (*Node).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize]) as *mut CM_KEY_INDEX;
        if Index.is_null() {
            return HCELL_NIL;
        }

        // Do a search inside it
        Result = CmpDoFindSubKeyByNumber(Hive, Index, Number);

        // Release the cell and return the result
        HvReleaseCell(Hive, (*Node).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize]);
        return Result;
    } else if (*Hive).StorageTypeCount > Volatile {
        // It's in the volatile list
        let Number = Number - (*Node).SubKeyCounts[Stable as usize];
        if Number < (*Node).SubKeyCounts[Volatile as usize] {
            // Get the actual key index
            Index = HvGetCell(Hive, (*Node).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Volatile as usize]) as *mut CM_KEY_INDEX;
            if Index.is_null() {
                return HCELL_NIL;
            }

            // Do a search inside it
            Result = CmpDoFindSubKeyByNumber(Hive, Index, Number);

            // Release the cell and return the result
            HvReleaseCell(Hive, (*Node).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Volatile as usize]);
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
            index_root = HvGetCell(hive, (*parent).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[i as usize]) as *mut CM_KEY_INDEX;
            if index_root.is_null() {
                return HCELL_NIL;
            }
            cell_to_release = (*parent).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[i as usize];

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
    Hive: PHHIVE,
    LeafCell: HCELL_INDEX,
    NewKey: HCELL_INDEX,
    Name: PCUNICODE_STRING,
) -> HCELL_INDEX {
    let mut Leaf: PCM_KEY_INDEX;
    let mut FastLeaf: PCM_KEY_FAST_INDEX;
    let mut Size: u32;
    let OldSize: u32;
    let EntrySize: u32;
    let mut i: u32;
    let mut j: u32;
    let mut NewCell: HCELL_INDEX;
    let mut Child: HCELL_INDEX = 0;
    let Result: i32;

    /* Mark the leaf dirty */
    HvMarkCellDirty(Hive, LeafCell, 0u8);

    /* Get the leaf cell */
    Leaf = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;
    if Leaf.is_null() {
        return HCELL_NIL;
    }

    /* Release it */
    HvReleaseCell(Hive, LeafCell);

    /* Check if this is an index leaf */
    if (*Leaf).Signature == CM_KEY_INDEX_LEAF as u16 {
        FastLeaf = core::ptr::null_mut();
        EntrySize = core::mem::size_of::<HCELL_INDEX>() as u32;
    } else {
        FastLeaf = Leaf as PCM_KEY_FAST_INDEX;
        EntrySize = core::mem::size_of::<CM_INDEX>() as u32;
    }

    /* Get the current size of the leaf */
    OldSize = HvGetCellSize(Hive, Leaf as *mut CM_KEY_INDEX);

    /* Calculate the size of the free entries */
    Size = OldSize;
    Size -= EntrySize * (*Leaf).Count as u32
        + (core::ptr::addr_of!((*Leaf).List) as usize - Leaf as usize) as u32;

    /* Assume we'll re-use the same leaf */
    NewCell = LeafCell;

    /* Check if we're out of space */
    if (Size / EntrySize) < 1 {
        Size = OldSize + OldSize / 2;
        if Size < (OldSize + EntrySize) {
            Size = OldSize + EntrySize;
        }

        NewCell = HvReallocateCell(Hive, LeafCell, Size);
        if NewCell == HCELL_NIL {
            return HCELL_NIL;
        }

        Leaf = HvGetCell(Hive, NewCell) as PCM_KEY_INDEX;
        if Leaf.is_null() {
            return HCELL_NIL;
        }

        HvReleaseCell(Hive, NewCell);

        if !FastLeaf.is_null() {
            FastLeaf = Leaf as PCM_KEY_FAST_INDEX;
        }
    }

    /* Find the insertion point for our entry */
    i = CmpFindSubKeyInLeaf(Hive, Leaf, Name, &mut Child);
    if (i & INVALID_INDEX) != 0 {
        return HCELL_NIL;
    }

    /* Check if we're not last */
    if i != (*Leaf).Count as u32 {
        Result = CmpCompareInIndex(Hive, Name, i, Leaf, &mut Child);
        if Result == 2 {
            return HCELL_NIL;
        }

        if Result > 0 {
            i += 1;
        }

        if i != (*Leaf).Count as u32 {
            if !FastLeaf.is_null() {
                core::ptr::copy(
                    core::ptr::addr_of!((*FastLeaf).List[i as usize]),
                    core::ptr::addr_of_mut!((*FastLeaf).List[(i + 1) as usize]),
                    ((*FastLeaf).Count as u32 - i) as usize,
                );
            } else {
                core::ptr::copy(
                    core::ptr::addr_of!((*Leaf).List[i as usize]),
                    core::ptr::addr_of_mut!((*Leaf).List[(i + 1) as usize]),
                    ((*Leaf).Count as u32 - i) as usize,
                );
            }
        }
    }

    /* Check if this is a new-style leaf */
    if !FastLeaf.is_null() {
        (*FastLeaf).List[i as usize].Cell = NewKey;

        if (*FastLeaf).Signature == CM_KEY_HASH_LEAF as u16 {
            (*FastLeaf).List[i as usize].__bindgen_anon_1.HashKey = CmpComputeHashKey(0, Name, 0u8);
        } else {
            (*FastLeaf).List[i as usize].__bindgen_anon_1.NameHint[0] = 0;
            (*FastLeaf).List[i as usize].__bindgen_anon_1.NameHint[1] = 0;
            (*FastLeaf).List[i as usize].__bindgen_anon_1.NameHint[2] = 0;
            (*FastLeaf).List[i as usize].__bindgen_anon_1.NameHint[3] = 0;

            if ((*Name).Length as usize / core::mem::size_of::<u16>()) < 4 {
                j = (*Name).Length as u32 / core::mem::size_of::<u16>() as u32;
            } else {
                j = 4;
            }

            loop {
                if (*(*Name).Buffer.offset((j - 1) as isize) as u16) > (u8::MAX as u16) {
                    break;
                }

                (*FastLeaf).List[i as usize].__bindgen_anon_1.NameHint[(j - 1) as usize] =
                    *(*Name).Buffer.offset((j - 1) as isize) as u8;

                j -= 1;
                if j == 0 {
                    break;
                }
            }
        }
    } else {
        (*Leaf).List[i as usize] = NewKey;
    }

    /* Update the leaf count and return the new cell */
    (*Leaf).Count += 1;
    NewCell
}

#[no_mangle]
pub unsafe extern "system" fn CmpSelectLeaf(
    Hive: PHHIVE,
    KeyNode: PCM_KEY_NODE,
    Name: PCUNICODE_STRING,
    Type: HSTORAGE_TYPE,
    RootCell: *mut PHCELL_INDEX,
) -> HCELL_INDEX {
    let mut IndexKey: PCM_KEY_INDEX;
    let mut LeafKey: PCM_KEY_INDEX;
    let mut FastLeaf: PCM_KEY_FAST_INDEX;
    let mut LeafCell: HCELL_INDEX = HCELL_NIL;
    let mut CurrentCell: HCELL_INDEX;
    let mut SubKeyIndex: u32;
    let mut Result: i32;

    HvMarkCellDirty(Hive, (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize], 0u8);

    IndexKey = HvGetCell(Hive, (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize]) as PCM_KEY_INDEX;

    if IndexKey.is_null() {
        debug_assert!(!IndexKey.is_null());
        return HCELL_NIL;
    }

    debug_assert!((*IndexKey).Signature == CM_KEY_INDEX_ROOT);

    loop {
        SubKeyIndex = CmpFindSubKeyInRoot(Hive, IndexKey, Name, &mut LeafCell);

        if (SubKeyIndex & INVALID_INDEX) != 0 {
            return HCELL_NIL;
        }

        if LeafCell != HCELL_NIL {
            LeafKey = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;

            if LeafKey.is_null() {
                return HCELL_NIL;
            }

            if ((*LeafKey).Count as u32) < CmpMaxIndexPerHblock {
                *RootCell = core::ptr::addr_of_mut!((*IndexKey).List[SubKeyIndex as usize]) as PHCELL_INDEX;
                return LeafCell;
            }

            /* It didn't fit, so proceed to splitting */
        } else {
            LeafCell = (*IndexKey).List[SubKeyIndex as usize];
            LeafKey = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;

            if LeafKey.is_null() {
                return HCELL_NIL;
            }

            if ((*LeafKey).Signature == CM_KEY_FAST_LEAF as u16)
                || ((*LeafKey).Signature == CM_KEY_HASH_LEAF as u16)
            {
                FastLeaf = LeafKey as PCM_KEY_FAST_INDEX;
                CurrentCell = (*FastLeaf).List[0].Cell;
            } else {
                debug_assert!((*LeafKey).Signature == CM_KEY_INDEX_LEAF as u16);
                CurrentCell = (*LeafKey).List[0];
            }

            Result = CmpDoCompareKeyName(Hive, Name, CurrentCell);

            if Result == 2 {
                return HCELL_NIL;
            }

            debug_assert!(Result != 0);

            if Result >= 0 {
                LeafCell = (*IndexKey).List[SubKeyIndex as usize];
                LeafKey = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;

                if LeafKey.is_null() {
                    return HCELL_NIL;
                }

                if ((*LeafKey).Count as u32) < CmpMaxIndexPerHblock {
                    *RootCell = core::ptr::addr_of_mut!((*IndexKey).List[SubKeyIndex as usize]) as PHCELL_INDEX;
                    return LeafCell;
                }

                if (SubKeyIndex + 1) < (*IndexKey).Count as u32 {
                    LeafCell = (*IndexKey).List[(SubKeyIndex + 1) as usize];
                    LeafKey = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;

                    if LeafKey.is_null() {
                        return HCELL_NIL;
                    }

                    if ((*LeafKey).Count as u32) < CmpMaxIndexPerHblock {
                        *RootCell = core::ptr::addr_of_mut!((*IndexKey).List[(SubKeyIndex + 1) as usize]) as PHCELL_INDEX;
                        return LeafCell;
                    }
                }

                /* It didn't fit, so proceed to splitting */
            } else {
                /* No, it's below, check the subkey index */
                if SubKeyIndex > 0 {
                    LeafCell = (*IndexKey).List[(SubKeyIndex - 1) as usize];
                    LeafKey = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;

                    if LeafKey.is_null() {
                        return HCELL_NIL;
                    }

                    if ((*LeafKey).Count as u32) < CmpMaxIndexPerHblock {
                        *RootCell = core::ptr::addr_of_mut!((*IndexKey).List[(SubKeyIndex - 1) as usize]) as PHCELL_INDEX;
                        return LeafCell;
                    }
                } else {
                    LeafCell = (*IndexKey).List[0];
                    LeafKey = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;

                    if LeafKey.is_null() {
                        return HCELL_NIL;
                    }

                    if ((*LeafKey).Count as u32) < CmpMaxIndexPerHblock {
                        *RootCell = core::ptr::addr_of_mut!((*IndexKey).List[0]) as PHCELL_INDEX;
                        return LeafCell;
                    }
                }

                /* It didn't fit into either, so proceed to splitting */
            }
        }

        /* We need to split */
        CurrentCell = CmpSplitLeaf(
            Hive,
            (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize],
            SubKeyIndex,
            Type,
        );

        if CurrentCell == HCELL_NIL {
            return CurrentCell;
        }

        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize] = CurrentCell;

        IndexKey = HvGetCell(Hive, (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize]) as PCM_KEY_INDEX;

        if IndexKey.is_null() {
            return HCELL_NIL;
        }
    }
}
