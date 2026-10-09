//! [correctness-review 2026-09-06] Reviewed vs sdk/lib/cmlib/cmindex.c.
//!
//! THIS FILE SHIPS IN THE KERNEL. `cmindex.c` is listed in `CMLIB_RUST_FLIPPED`
//! and REMOVE_ITEM'd from the kernel cmlib source list, so the C body does NOT win
//! at link -- its appearance at `sdk/lib/cmlib/CMakeLists.txt:10` is the SHARED list
//! that feeds the bootloader (`blcmlib`) and the host variant. Verified against the
//! artifact, not the CMake: `build-i386/.../libcmlib.a` is a 248-byte placeholder TU
//! and `_CmpSelectLeaf@20` is defined in `libcmlib_rust.a`. A prior review read the
//! CMake alone and filed every bug here as "latent, the C wins" -- it does not.
//!
//! FIXED 2026-09-06:
//!   `CmpSelectLeaf` wrote `*RootCell` via `addr_of_mut!((*IndexKey).List[i])` at
//!   three sites (i = SubKeyIndex, +1, -1). An array index inside a raw-ref is NOT
//!   plain address-taking: it still emits a bounds check, and bindgen renders the C
//!   flexible array as `[HCELL_INDEX; 1]`, so each one PANICS for every i >= 1 --
//!   that is, for any subkey list whose index root has more than one leaf. Measured
//!   in the shipped i386 object: 4 `panic_bounds_check` call sites in this function,
//!   three of them pushing an array length of 1; after routing them through
//!   `index_list_ptr` only the legitimate length-2 `SubKeyLists[Type]` check remains.
//!   Small hives boot fine, which is why it survived the smoke.
//!   Also restored the `ASSERT(IndexKey->Signature == CM_KEY_INDEX_ROOT)` the C makes
//!   after a split; the C's trailing unreachable `return HCELL_NIL` has no counterpart
//!   because Rust's `loop` with no `break` diverges.
//!
//! The 2026-06-07 header this replaces listed four REAL BUGS, and ALL FOUR had been
//! fixed by `85479bd0878` without anyone updating the text. It still cited line
//! numbers (`cmindex.rs:202`, `:258`) from a file less than a sixth of the current
//! length, and its claims that `CmpAddToLeaf` "inserts nothing" and that
//! `CmpSelectLeaf` is "missing ~100 lines including the split" were both false against
//! the code beneath it -- verified by table-diffing call sites with the C (HvGetCell
//! 8/8, CmpMaxIndexPerHblock 5/5, HCELL_NIL 13/13). The five NTAPI callees it says are
//! `extern "C"` are defined locally as `extern "system"`, and `CmpFindSubKeyByHash` is
//! a local Rust fn, so neither ABI complaint applies either.
//!
//! Keep this header true or delete it: a stale review that names line numbers reads as
//! current and outlives the defect it describes. This one cost a roadmap entry that
//! filed a live kernel panic as dead code.
//!
//! VERIFIED OK (re-checked against cmindex.c):
//!   CmpAddToLeaf -- complete; the RtlMoveMemory element counts are correct
//!     (`core::ptr::copy` counts ELEMENTS, so the C's `* sizeof(...)` is right to drop).
//!   CmpComputeHashKey -- correct (`<=` vs `<` on 'z' is equivalent: both yield 'Z').
//!   CmpFindSubKeyByNumber / CmpFindSubKeyByName -- correct logic and ABI.
//!   CmpMaxIndexPerHblock in glue.rs -- layout-derived, matches the C macro.
//!   Constants (INVALID_INDEX, HCELL_NIL, CM_KEY_*) -- correct vs cmdata.h/hivedata.h.
//!
//! STILL OPEN: CmpDoFindSubKeyByNumber is extern and not translated here.

use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{CM_KEY_NODE, CM_INDEX, CM_KEY_FAST_INDEX, CM_KEY_INDEX, CmpCompareCompressedName, CmpCompressedNameSize, CmpCopyCompressedName, HBIN, HCELL, HCELL_INDEX, HHIVE, HSTORAGE_TYPE, PCM_KEY_FAST_INDEX, PCM_KEY_INDEX, PCM_KEY_NODE, PCUNICODE_STRING, PHCELL_INDEX, PHHIVE, RtlCompareUnicodeString, HSTORAGE_TYPE_Stable as Stable, HSTORAGE_TYPE_Volatile as Volatile, _HHIVE, _UNICODE_STRING};
use crate::glue::{HvGetCell, HvReleaseCell, HvGetCellType, HCELL_NIL, HBLOCK_SIZE, CM_KEY_INDEX_ROOT, CM_KEY_INDEX_LEAF, CM_KEY_FAST_LEAF, CM_KEY_HASH_LEAF, INVALID_INDEX, CmpMaxIndexPerHblock, TAG_CM, KEY_COMP_NAME};
use core::ffi::c_void;

extern "system" {
    fn HvAllocateCell(Hive: PHHIVE, Size: u32, Type: HSTORAGE_TYPE, SourceCell: HCELL_INDEX) -> HCELL_INDEX;
    fn HvFreeCell(Hive: PHHIVE, Cell: HCELL_INDEX);
    fn HvMarkCellDirty(Hive: PHHIVE, Cell: HCELL_INDEX, Unk: u8) -> u8;
    fn HvGetCellSize(Hive: PHHIVE, Cell: *mut CM_KEY_INDEX) -> u32;
    fn HvReallocateCell(Hive: PHHIVE, Cell: HCELL_INDEX, NewSize: u32) -> HCELL_INDEX;
    fn RtlUpcaseUnicodeChar(ch: u16) -> u16;
}

// ── Flexible-array-member helpers (CM_KEY_INDEX.List is [HCELL_INDEX; 1],
//    CM_KEY_FAST_INDEX.List is [CM_INDEX; 1]). NEVER index `List[i]` for i>=1 —
//    bindgen renders the C flexible array as a 1-element array which panics at
//    boot for i>=1. Always go through these pointer-arith accessors. ───────────
#[inline]
unsafe fn index_list_ptr(idx: *const CM_KEY_INDEX, i: u32) -> *mut HCELL_INDEX {
    (core::ptr::addr_of!((*idx).List) as *mut HCELL_INDEX).add(i as usize)
}
#[inline]
unsafe fn index_list_get(idx: *const CM_KEY_INDEX, i: u32) -> HCELL_INDEX {
    core::ptr::read(index_list_ptr(idx, i))
}
#[inline]
unsafe fn fast_list_ptr(idx: *const CM_KEY_FAST_INDEX, i: u32) -> *mut CM_INDEX {
    (core::ptr::addr_of!((*idx).List) as *mut CM_INDEX).add(i as usize)
}

#[no_mangle]
pub unsafe extern "system" fn CmpDoCompareKeyName(
    Hive: PHHIVE,
    SearchName: PCUNICODE_STRING,
    Cell: HCELL_INDEX,
) -> i32 {
    /* Get the node */
    let Node = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
    if Node.is_null() {
        return 2;
    }

    let Result: i32;
    /* Check if it's compressed */
    if ((*Node).Flags & KEY_COMP_NAME) != 0 {
        /* Compare compressed names */
        Result = CmpCompareCompressedName(
            SearchName,
            core::ptr::addr_of_mut!((*Node).Name) as *mut u16,
            (*Node).NameLength as u32,
        );
    } else {
        /* Compare the Unicode name directly */
        let KeyName = _UNICODE_STRING {
            Length: (*Node).NameLength,
            MaximumLength: (*Node).NameLength,
            Buffer: core::ptr::addr_of_mut!((*Node).Name) as *mut u16,
        };
        Result = RtlCompareUnicodeString(SearchName, &KeyName, 1u8);
    }

    /* Release the cell and return the normalized result */
    HvReleaseCell(Hive, Cell);
    if Result == 0 {
        Result
    } else if Result > 0 {
        1
    } else {
        -1
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpCompareInIndex(
    Hive: PHHIVE,
    SearchName: PCUNICODE_STRING,
    Count: u32,
    Index: PCM_KEY_INDEX,
    SubKey: PHCELL_INDEX,
) -> i32 {
    let mut Result: i32;

    /* Assume failure */
    *SubKey = HCELL_NIL;

    /* Check if we are a fast or hashed leaf */
    if (*Index).Signature == CM_KEY_FAST_LEAF || (*Index).Signature == CM_KEY_HASH_LEAF {
        /* Get the Fast/Hash Index */
        let FastIndex = Index as PCM_KEY_FAST_INDEX;
        let FastEntry: *mut CM_INDEX = fast_list_ptr(FastIndex, Count);

        /* Check if we are a hash leaf, in which case we skip all this */
        if (*Index).Signature == CM_KEY_FAST_LEAF {
            let mut ActualNameLength: u32 = 4;

            /* Find out just how much of the name is there */
            for i in 0..4u32 {
                /* Check if this entry is empty */
                if (*FastEntry).__bindgen_anon_1.NameHint[i as usize] == 0 {
                    ActualNameLength = i;
                    break;
                }
            }

            /* How large is the name and how many characters to compare */
            let NameLength = (*SearchName).Length as u32 / core::mem::size_of::<u16>() as u32;
            let CompareLength = core::cmp::min(NameLength, ActualNameLength);

            /* Loop all the chars we'll test */
            for i in 0..CompareLength {
                /* Get one char from each buffer */
                let p = *(*SearchName).Buffer.add(i as usize);
                let pp = (*FastEntry).__bindgen_anon_1.NameHint[i as usize] as u16;

                /* See if they match and return result if they don't */
                Result = RtlUpcaseUnicodeChar(p) as i32 - RtlUpcaseUnicodeChar(pp) as i32;
                if Result != 0 {
                    return if Result > 0 { 1 } else { -1 };
                }
            }
        }

        /* If we got here then we have to do a full compare */
        Result = CmpDoCompareKeyName(Hive, SearchName, (*FastEntry).Cell);
        if Result == 2 {
            return Result;
        }
        if Result == 0 {
            *SubKey = (*FastEntry).Cell;
        }
    } else {
        /* We aren't, so do a name compare and return the subkey found */
        let entry = index_list_get(Index, Count);
        Result = CmpDoCompareKeyName(Hive, SearchName, entry);
        if Result == 2 {
            return Result;
        }
        if Result == 0 {
            *SubKey = entry;
        }
    }

    /* Return the comparison result */
    Result
}

#[no_mangle]
pub unsafe extern "system" fn CmpFindSubKeyInRoot(
    Hive: PHHIVE,
    Index: PCM_KEY_INDEX,
    SearchName: PCUNICODE_STRING,
    SubKey: PHCELL_INDEX,
) -> u32 {
    let mut High: u32;
    let mut Low: u32 = 0;
    let mut i: u32;
    let ReturnIndex: u32;
    let mut LeafCell: HCELL_INDEX;
    let mut Leaf: PCM_KEY_INDEX;
    let mut Result: i32;

    /* Verify Index for validity */
    ASSERT!((*Index).Count != 0);
    ASSERT!((*Index).Signature == CM_KEY_INDEX_ROOT);

    /* Set high limit and loop */
    High = (*Index).Count as u32 - 1;
    loop {
        /* Choose next entry */
        i = ((High - Low) / 2) + Low;

        /* Get the leaf cell and then the leaf itself */
        LeafCell = index_list_get(Index, i);
        Leaf = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;
        if !Leaf.is_null() {
            /* Do the compare */
            Result = CmpCompareInIndex(Hive, SearchName, (*Leaf).Count as u32 - 1, Leaf, SubKey);
            if Result == 2 {
                /* Big */
                ReturnIndex = INVALID_INDEX;
                *SubKey = HCELL_NIL;
                return ReturnIndex;
            }

            /* Check if we found the leaf */
            if Result == 0 {
                *SubKey = LeafCell;
                ReturnIndex = i;
                HvReleaseCell(Hive, LeafCell);
                return ReturnIndex;
            }

            /* Check for negative result */
            if Result < 0 {
                /* Do another lookup, since we might still be in the right leaf */
                Result = CmpCompareInIndex(Hive, SearchName, 0, Leaf, SubKey);
                if Result == 2 {
                    ReturnIndex = INVALID_INDEX;
                    *SubKey = HCELL_NIL;
                    return ReturnIndex;
                }

                /* Check if it's not below */
                if Result >= 0 {
                    *SubKey = LeafCell;
                    ReturnIndex = i;
                    HvReleaseCell(Hive, LeafCell);
                    return ReturnIndex;
                }

                /* Update the limit to this index, since we know it's not higher. */
                High = i;
            } else {
                /* Update the base to this index, since we know it's not lower. */
                Low = i;
            }
        } else {
            /* Big: This was some sort of special key */
            ReturnIndex = INVALID_INDEX;
            *SubKey = HCELL_NIL;
            return ReturnIndex;
        }

        /* Check if there is only one entry left */
        if (High - Low) <= 1 {
            break;
        }

        /* Release the leaf cell */
        HvReleaseCell(Hive, LeafCell);
    }

    /* Get the leaf cell and the leaf */
    LeafCell = index_list_get(Index, Low);
    Leaf = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;
    if Leaf.is_null() {
        return INVALID_INDEX;
    }

    /* Do the compare */
    Result = CmpCompareInIndex(Hive, SearchName, (*Leaf).Count as u32 - 1, Leaf, SubKey);
    if Result == 2 {
        if !Leaf.is_null() {
            HvReleaseCell(Hive, LeafCell);
        }
        return INVALID_INDEX;
    }

    /* Check if we found it */
    if Result == 0 {
        *SubKey = LeafCell;
        ReturnIndex = Low;
        HvReleaseCell(Hive, LeafCell);
        return ReturnIndex;
    }

    /* It's below, so could still be in this leaf */
    if Result < 0 {
        /* Do a search from the bottom */
        Result = CmpCompareInIndex(Hive, SearchName, 0, Leaf, SubKey);
        if Result == 2 {
            HvReleaseCell(Hive, LeafCell);
            return INVALID_INDEX;
        }

        if Result >= 0 {
            *SubKey = LeafCell;
            ReturnIndex = Low;
            HvReleaseCell(Hive, LeafCell);
            return ReturnIndex;
        }

        /* It's still below us, so fail */
        ReturnIndex = Low;
        *SubKey = HCELL_NIL;
        HvReleaseCell(Hive, LeafCell);
        return ReturnIndex;
    }

    /* Release the leaf cell */
    HvReleaseCell(Hive, LeafCell);

    /* Well the low didn't work too well, so try the high. */
    LeafCell = index_list_get(Index, High);
    Leaf = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;
    if Leaf.is_null() {
        return INVALID_INDEX;
    }

    /* Do the compare */
    Result = CmpCompareInIndex(Hive, SearchName, (*Leaf).Count as u32 - 1, Leaf, SubKey);
    if Result == 2 {
        if !Leaf.is_null() {
            HvReleaseCell(Hive, LeafCell);
        }
        return INVALID_INDEX;
    }

    /* Check if we found it */
    if Result == 0 {
        *SubKey = LeafCell;
        ReturnIndex = High;
        HvReleaseCell(Hive, LeafCell);
        return ReturnIndex;
    }

    /* Check if we are too high */
    if Result < 0 {
        *SubKey = LeafCell;
        ReturnIndex = High;
        HvReleaseCell(Hive, LeafCell);
        return ReturnIndex;
    }

    /* If we got here, then we are too low, again. */
    ReturnIndex = High;
    *SubKey = HCELL_NIL;
    HvReleaseCell(Hive, LeafCell);
    ReturnIndex
}

#[no_mangle]
pub unsafe extern "system" fn CmpFindSubKeyInLeaf(
    Hive: PHHIVE,
    Index: PCM_KEY_INDEX,
    SearchName: PCUNICODE_STRING,
    SubKey: PHCELL_INDEX,
) -> u32 {
    let mut High: u32;
    let mut Low: u32 = 0;
    let mut i: u32;
    let mut Result: i32;

    /* Get the upper bound and middle entry */
    High = (*Index).Count as u32 - 1;
    i = High / 2;

    /* Check if we don't actually have any entries */
    if (*Index).Count == 0 {
        *SubKey = HCELL_NIL;
        return 0;
    }

    /* Start compare loop */
    loop {
        /* Do the actual comparison and check the result */
        Result = CmpCompareInIndex(Hive, SearchName, i, Index, SubKey);
        if Result == 2 {
            *SubKey = HCELL_NIL;
            return INVALID_INDEX;
        }

        /* Check if we got lucky and found it */
        if Result == 0 {
            return i;
        }

        /* Check if the result is below us */
        if Result < 0 {
            High = i;
        } else {
            Low = i;
        }

        /* Check if this is the last entry, if so, break out and handle it */
        if (High - Low) <= 1 {
            break;
        }

        /* Set the new index */
        i = ((High - Low) / 2) + Low;
    }

    /*
     * If we get here, High - Low = 1 or High == Low
     * Simply look first at Low, then at High
     */
    Result = CmpCompareInIndex(Hive, SearchName, Low, Index, SubKey);
    if Result == 2 {
        *SubKey = HCELL_NIL;
        return INVALID_INDEX;
    }

    /* Check if we got lucky and found it */
    if Result == 0 {
        return Low;
    }

    /* Check if the result is below us */
    if Result < 0 {
        return Low;
    }

    /*
     * If we got here, then just check the high and return it no matter what
     * the result is (since we're a leaf, it has to be near there anyway).
     */
    Result = CmpCompareInIndex(Hive, SearchName, High, Index, SubKey);
    if Result == 2 {
        *SubKey = HCELL_NIL;
        return INVALID_INDEX;
    }

    /* Return the high */
    High
}

#[no_mangle]
pub unsafe extern "system" fn CmpDoFindSubKeyByNumber(
    Hive: PHHIVE,
    Index: PCM_KEY_INDEX,
    mut Number: u32,
) -> HCELL_INDEX {
    let mut LeafCell: HCELL_INDEX = 0;
    let mut Leaf: PCM_KEY_INDEX = core::ptr::null_mut();
    let Result: HCELL_INDEX;

    /* Check if this is a root */
    if (*Index).Signature == CM_KEY_INDEX_ROOT {
        /* Loop the index */
        let mut i: u32 = 0;
        while i < (*Index).Count as u32 {
            /* Check if this isn't the first iteration */
            if i != 0 {
                /* Make sure we had something valid, and release it */
                HvReleaseCell(Hive, LeafCell);
            }

            /* Get the leaf cell and the leaf for this index */
            LeafCell = index_list_get(Index, i);
            Leaf = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;
            if Leaf.is_null() {
                return HCELL_NIL;
            }

            /* Check if the index may be inside it */
            if Number < (*Leaf).Count as u32 {
                /* Check if this is a fast or hash leaf */
                if (*Leaf).Signature == CM_KEY_FAST_LEAF || (*Leaf).Signature == CM_KEY_HASH_LEAF {
                    /* Get the fast index */
                    let FastIndex = Leaf as PCM_KEY_FAST_INDEX;

                    /* Look inside the list to get our actual cell */
                    Result = (*fast_list_ptr(FastIndex, Number)).Cell;
                    HvReleaseCell(Hive, LeafCell);
                    return Result;
                } else {
                    /* Regular leaf, so just use the index directly */
                    Result = index_list_get(Leaf, Number);
                    HvReleaseCell(Hive, LeafCell);
                    return Result;
                }
            } else {
                /* Otherwise, skip this leaf */
                Number -= (*Leaf).Count as u32;
            }

            i += 1;
        }

        /* Should never get here */
        ASSERT!(false);
    }

    /* If we got here, then the cell is in this index */

    /* Check if we're a fast or hash leaf */
    if (*Index).Signature == CM_KEY_FAST_LEAF || (*Index).Signature == CM_KEY_HASH_LEAF {
        /* We are, get the fast index and get the cell from the list */
        let FastIndex = Index as PCM_KEY_FAST_INDEX;
        (*fast_list_ptr(FastIndex, Number)).Cell
    } else {
        /* We aren't, so use the index directly to grab the cell */
        index_list_get(Index, Number)
    }
}

/* C-internal (`static`) — keep crate-private, NO #[no_mangle]. */
unsafe fn CmpFindSubKeyByHash(
    Hive: *mut _HHIVE,
    FastIndex: *const CM_KEY_FAST_INDEX,
    SearchName: *const _UNICODE_STRING,
) -> u32 {
    /* Compute the hash key for the name */
    let HashKey = CmpComputeHashKey(0, SearchName, 0u8);

    /* Loop all the entries */
    let mut i: u32 = 0;
    while i < (*FastIndex).Count as u32 {
        /* Get the entry */
        let FastEntry: *mut CM_INDEX = fast_list_ptr(FastIndex, i);

        /* Compare the hash first */
        if (*FastEntry).__bindgen_anon_1.HashKey == HashKey {
            /* Go ahead for a full compare */
            if CmpDoCompareKeyName(Hive as PHHIVE, SearchName, (*FastEntry).Cell) == 0 {
                /* It matched, return the cell */
                return (*FastEntry).Cell;
            }
        }
        i += 1;
    }

    /* If we got here then we failed */
    HCELL_NIL
}

#[no_mangle]
pub unsafe extern "system" fn CmpMarkIndexDirty(
    Hive: PHHIVE,
    ParentKey: HCELL_INDEX,
    TargetKey: HCELL_INDEX,
) -> u8 {
    let mut SearchName = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let IsCompressed: bool;
    let mut Child: HCELL_INDEX;
    let mut CellToRelease: HCELL_INDEX = HCELL_NIL;

    /* Get the target key node */
    let mut Node = HvGetCell(Hive, TargetKey) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0;
    }

    /* Check if it's compressed */
    if ((*Node).Flags & KEY_COMP_NAME) != 0 {
        /* Remember this for later */
        IsCompressed = true;

        /* Build the search name */
        SearchName.Length = CmpCompressedNameSize(
            core::ptr::addr_of_mut!((*Node).Name) as *mut u16,
            (*Node).NameLength as u32,
        );
        SearchName.MaximumLength = SearchName.Length;
        SearchName.Buffer = (*Hive).Allocate.unwrap()(SearchName.Length as _, 1u8, TAG_CM) as *mut u16;
        if SearchName.Buffer.is_null() {
            /* Fail */
            HvReleaseCell(Hive, TargetKey);
            return 0;
        }

        /* Copy it */
        CmpCopyCompressedName(
            SearchName.Buffer,
            SearchName.MaximumLength as u32,
            core::ptr::addr_of_mut!((*Node).Name) as *mut u16,
            (*Node).NameLength as u32,
        );
    } else {
        /* Name isn't compressed, build it directly from the node */
        IsCompressed = false;
        SearchName.Length = (*Node).NameLength;
        SearchName.MaximumLength = (*Node).NameLength;
        SearchName.Buffer = core::ptr::addr_of_mut!((*Node).Name) as *mut u16;
    }

    /* We can release the target key now */
    HvReleaseCell(Hive, TargetKey);

    /* Now get the parent key node */
    Node = HvGetCell(Hive, ParentKey) as PCM_KEY_NODE;
    if !Node.is_null() {
        /* Loop all hive storage */
        let mut i: u32 = 0;
        'storage: while i < (*Hive).StorageTypeCount {
            /* Check if any subkeys are in this index */
            if (*Node).SubKeyCounts[i as usize] != 0 {
                /* Get the cell index */
                let mut IndexCell = (*Node).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[i as usize];

                /* Check if we had anything to release from before */
                if CellToRelease != HCELL_NIL {
                    HvReleaseCell(Hive, CellToRelease);
                    CellToRelease = HCELL_NIL;
                }

                /* Get the key index for the cell */
                let mut Index = HvGetCell(Hive, IndexCell) as PCM_KEY_INDEX;
                if Index.is_null() {
                    break 'storage;
                }

                /* Release it at the next iteration or below */
                CellToRelease = IndexCell;

                /* Check if this is a root */
                if (*Index).Signature == CM_KEY_INDEX_ROOT {
                    Child = HCELL_NIL;
                    let Result = CmpFindSubKeyInRoot(Hive, Index, &mut SearchName, &mut Child);
                    if (Result & INVALID_INDEX) != 0 {
                        break 'storage;
                    }
                    if Child == HCELL_NIL {
                        i += 1;
                        continue 'storage;
                    }

                    /* We found it, mark the cell dirty */
                    HvMarkCellDirty(Hive, IndexCell, 0u8);

                    /* Check if we had anything to release from before */
                    if CellToRelease != HCELL_NIL {
                        HvReleaseCell(Hive, CellToRelease);
                        CellToRelease = HCELL_NIL;
                    }

                    /* Now this child is the index, get the actual key index */
                    IndexCell = Child;
                    Index = HvGetCell(Hive, Child) as PCM_KEY_INDEX;
                    if Index.is_null() {
                        break 'storage;
                    }

                    /* Release it later */
                    CellToRelease = Child;
                }

                /* Find the child in the leaf */
                Child = HCELL_NIL;
                let Result = CmpFindSubKeyInLeaf(Hive, Index, &mut SearchName, &mut Child);
                if (Result & INVALID_INDEX) != 0 {
                    break 'storage;
                }
                if Child != HCELL_NIL {
                    /* We found it, free the name now */
                    if IsCompressed {
                        (*Hive).Free.unwrap()(SearchName.Buffer as *mut c_void, 0);
                    }

                    /* Release the parent key */
                    HvReleaseCell(Hive, ParentKey);

                    /* Check if we had a left over cell to release */
                    if CellToRelease != HCELL_NIL {
                        HvReleaseCell(Hive, CellToRelease);
                    }

                    /* And mark the index cell dirty */
                    HvMarkCellDirty(Hive, IndexCell, 0u8);
                    return 1;
                }
            }
            i += 1;
        }
    }

    /* Quickie: Release any cells that we still hold */
    if !Node.is_null() {
        HvReleaseCell(Hive, ParentKey);
    }
    if CellToRelease != HCELL_NIL {
        HvReleaseCell(Hive, CellToRelease);
    }

    /* Free the search name and return failure */
    if IsCompressed {
        (*Hive).Free.unwrap()(SearchName.Buffer as *mut c_void, 0);
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn CmpSplitLeaf(
    Hive: PHHIVE,
    mut RootCell: HCELL_INDEX,
    RootSelect: u32,
    Type: HSTORAGE_TYPE,
) -> HCELL_INDEX {
    let mut IndexKey: PCM_KEY_INDEX;
    let LeafKey: PCM_KEY_INDEX;
    let NewKey: PCM_KEY_INDEX;
    let LeafCell: HCELL_INDEX;
    let NewCell: HCELL_INDEX;
    let FirstHalf: u16;
    let LastHalf: u16;
    let EntrySize: u32;
    let mut TotalSize: u32;

    /* Get the cell */
    IndexKey = HvGetCell(Hive, RootCell) as PCM_KEY_INDEX;
    if IndexKey.is_null() {
        return HCELL_NIL;
    }

    /* Get the leaf cell and key */
    LeafCell = index_list_get(IndexKey, RootSelect);
    LeafKey = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;
    if LeafKey.is_null() {
        return HCELL_NIL;
    }

    /* We are going to divide this leaf into two halves */
    FirstHalf = (*LeafKey).Count / 2;
    LastHalf = (*LeafKey).Count - FirstHalf;

    /* Compute entry size based on hive version */
    if (*Hive).Version >= 5 {
        EntrySize = core::mem::size_of::<CM_INDEX>() as u32;
    } else {
        EntrySize = core::mem::size_of::<HCELL_INDEX>() as u32;
    }

    /* Compute the total size */
    let list_off = core::mem::offset_of!(CM_KEY_INDEX, List) as u32;
    TotalSize = (EntrySize * LastHalf as u32) + list_off + 1;

    /* Mark the leaf cell dirty */
    HvMarkCellDirty(Hive, LeafCell, 0u8);

    /* Make sure its type is the same */
    ASSERT!(HvGetCellType(LeafCell) == Type as u32);

    /* Allocate the cell, fail in case of error */
    NewCell = HvAllocateCell(Hive, TotalSize, Type, LeafCell);
    if NewCell == HCELL_NIL {
        return NewCell;
    }

    /* Get the key */
    NewKey = HvGetCell(Hive, NewCell) as PCM_KEY_INDEX;
    if NewKey.is_null() {
        /* Free the cell and exit - should not happen! */
        HvFreeCell(Hive, NewCell);
        return HCELL_NIL;
    }

    /* Release the newly created cell */
    HvReleaseCell(Hive, NewCell);

    /* Set its signature according to the version of the hive */
    if (*Hive).Version >= 5 {
        (*NewKey).Signature = CM_KEY_HASH_LEAF;
    } else {
        (*NewKey).Signature = CM_KEY_INDEX_LEAF;
    }

    /* Calculate the size of the free entries in the root key */
    TotalSize = HvGetCellSize(Hive, IndexKey)
        - ((*IndexKey).Count as u32 * core::mem::size_of::<HCELL_INDEX>() as u32)
        - list_off;

    /* Check if we're out of space */
    if TotalSize / (core::mem::size_of::<HCELL_INDEX>() as u32) < 1 {
        /* Grow the leaf by one more entry */
        TotalSize = HvGetCellSize(Hive, IndexKey) + core::mem::size_of::<HCELL_INDEX>() as u32;

        /* Re-allocate the root */
        RootCell = HvReallocateCell(Hive, RootCell, TotalSize);
        if RootCell == HCELL_NIL {
            HvFreeCell(Hive, NewCell);
            return HCELL_NIL;
        }

        /* Get the leaf cell */
        IndexKey = HvGetCell(Hive, RootCell) as PCM_KEY_INDEX;
        if IndexKey.is_null() {
            return HCELL_NIL;
        }
    }

    /* Splitting is done, now we need to copy the contents */
    if (*Hive).Version >= 5 {
        /* Copy the fast indexes */
        let FastLeaf = LeafKey as PCM_KEY_FAST_INDEX;
        core::ptr::copy(
            fast_list_ptr(FastLeaf, FirstHalf as u32) as *const u8,
            core::ptr::addr_of_mut!((*NewKey).List) as *mut u8,
            LastHalf as usize * EntrySize as usize,
        );
    } else {
        /* Copy the indexes themselves */
        core::ptr::copy(
            index_list_ptr(LeafKey, FirstHalf as u32) as *const u8,
            core::ptr::addr_of_mut!((*NewKey).List) as *mut u8,
            LastHalf as usize * EntrySize as usize,
        );
    }

    /* Shift the data inside the root key */
    if (RootSelect + 1) < (*IndexKey).Count as u32 {
        core::ptr::copy(
            index_list_ptr(IndexKey, RootSelect + 1) as *const u8,
            index_list_ptr(IndexKey, RootSelect + 2) as *mut u8,
            ((*IndexKey).Count as u32 - (RootSelect + 1)) as usize
                * core::mem::size_of::<HCELL_INDEX>(),
        );
    }

    /* Update the count value of old and new keys */
    (*LeafKey).Count = FirstHalf;
    (*NewKey).Count = LastHalf;

    /* Increase the count value of the root key */
    (*IndexKey).Count += 1;

    /* Set the new cell in root's list */
    *index_list_ptr(IndexKey, RootSelect + 1) = NewCell;

    /* Return the root cell */
    RootCell
}

#[no_mangle]
pub unsafe extern "system" fn CmpRemoveSubKey(
    Hive: PHHIVE,
    ParentKey: HCELL_INDEX,
    TargetKey: HCELL_INDEX,
) -> u8 {
    let mut SearchName = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let IsCompressed: bool;
    let mut Buffer: [u16; 50] = [0; 50];
    let mut RootCell: HCELL_INDEX = HCELL_NIL;
    let mut LeafCell: HCELL_INDEX;
    let mut ChildCell: HCELL_INDEX = 0;
    let mut Root: PCM_KEY_INDEX = core::ptr::null_mut();
    let mut Leaf: PCM_KEY_INDEX;
    let mut RootIndex: u32 = INVALID_INDEX;
    let LeafIndex: u32;
    let mut Result: u8 = 0;
    let mut CellToRelease1: HCELL_INDEX = HCELL_NIL;
    let mut CellToRelease2: HCELL_INDEX = HCELL_NIL;

    /* Get the target key node */
    let mut Node = HvGetCell(Hive, TargetKey) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0;
    }

    /* Make sure it's dirty, then release it */
    HvReleaseCell(Hive, TargetKey);

    /* Check if the name is compressed */
    if ((*Node).Flags & KEY_COMP_NAME) != 0 {
        /* Remember for later */
        IsCompressed = true;

        /* Build the search name */
        SearchName.Length = CmpCompressedNameSize(
            core::ptr::addr_of_mut!((*Node).Name) as *mut u16,
            (*Node).NameLength as u32,
        );
        SearchName.MaximumLength = SearchName.Length;

        /* Do we need an extra buffer? */
        if SearchName.MaximumLength as usize > core::mem::size_of::<[u16; 50]>() {
            /* Allocate one */
            SearchName.Buffer = (*Hive).Allocate.unwrap()(SearchName.Length as _, 1u8, TAG_CM) as *mut u16;
            if SearchName.Buffer.is_null() {
                return 0;
            }
        } else {
            /* Otherwise, use our local stack buffer */
            SearchName.Buffer = Buffer.as_mut_ptr();
        }

        /* Copy the compressed name */
        CmpCopyCompressedName(
            SearchName.Buffer,
            SearchName.MaximumLength as u32,
            core::ptr::addr_of_mut!((*Node).Name) as *mut u16,
            (*Node).NameLength as u32,
        );
    } else {
        /* It's not compressed, build the name directly from the node */
        IsCompressed = false;
        SearchName.Length = (*Node).NameLength;
        SearchName.MaximumLength = (*Node).NameLength;
        SearchName.Buffer = core::ptr::addr_of_mut!((*Node).Name) as *mut u16;
    }

    /* Now get the parent node */
    Node = HvGetCell(Hive, ParentKey) as PCM_KEY_NODE;
    if Node.is_null() {
        /* Exit */
        if CellToRelease1 != HCELL_NIL {
            HvReleaseCell(Hive, CellToRelease1);
        }
        if CellToRelease2 != HCELL_NIL {
            HvReleaseCell(Hive, CellToRelease2);
        }
        if IsCompressed && SearchName.MaximumLength as usize > core::mem::size_of::<[u16; 50]>() {
            (*Hive).Free.unwrap()(SearchName.Buffer as *mut c_void, 0);
        }
        return Result;
    }

    /* Make sure it's dirty, then release it */
    HvReleaseCell(Hive, ParentKey);

    /* Get the storage type and make sure it's not empty */
    let Storage = HvGetCellType(TargetKey);

    /* Get the leaf cell now */
    LeafCell = (*Node).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Storage as usize];
    Leaf = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;
    if Leaf.is_null() {
        /* Exit */
        if CellToRelease1 != HCELL_NIL {
            HvReleaseCell(Hive, CellToRelease1);
        }
        if CellToRelease2 != HCELL_NIL {
            HvReleaseCell(Hive, CellToRelease2);
        }
        if IsCompressed && SearchName.MaximumLength as usize > core::mem::size_of::<[u16; 50]>() {
            (*Hive).Free.unwrap()(SearchName.Buffer as *mut c_void, 0);
        }
        return Result;
    }

    /* Remember to release it later */
    CellToRelease1 = LeafCell;

    /* Check if the leaf is a root */
    if (*Leaf).Signature == CM_KEY_INDEX_ROOT {
        /* Find the child inside the root */
        RootIndex = CmpFindSubKeyInRoot(Hive, Leaf, &mut SearchName, &mut ChildCell);
        if (RootIndex & INVALID_INDEX) != 0 {
            /* Exit */
            if CellToRelease1 != HCELL_NIL {
                HvReleaseCell(Hive, CellToRelease1);
            }
            if CellToRelease2 != HCELL_NIL {
                HvReleaseCell(Hive, CellToRelease2);
            }
            if IsCompressed && SearchName.MaximumLength as usize > core::mem::size_of::<[u16; 50]>() {
                (*Hive).Free.unwrap()(SearchName.Buffer as *mut c_void, 0);
            }
            return Result;
        }

        /* The root cell is now this leaf */
        Root = Leaf;
        RootCell = LeafCell;

        /* And the new leaf is now this child */
        LeafCell = ChildCell;
        Leaf = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;
        if Leaf.is_null() {
            /* Exit */
            if CellToRelease1 != HCELL_NIL {
                HvReleaseCell(Hive, CellToRelease1);
            }
            if CellToRelease2 != HCELL_NIL {
                HvReleaseCell(Hive, CellToRelease2);
            }
            if IsCompressed && SearchName.MaximumLength as usize > core::mem::size_of::<[u16; 50]>() {
                (*Hive).Free.unwrap()(SearchName.Buffer as *mut c_void, 0);
            }
            return Result;
        }

        /* Remember to release it later */
        CellToRelease2 = LeafCell;
    }

    /* Now get the child in the leaf */
    LeafIndex = CmpFindSubKeyInLeaf(Hive, Leaf, &mut SearchName, &mut ChildCell);
    if (LeafIndex & INVALID_INDEX) != 0 {
        /* Exit */
        if CellToRelease1 != HCELL_NIL {
            HvReleaseCell(Hive, CellToRelease1);
        }
        if CellToRelease2 != HCELL_NIL {
            HvReleaseCell(Hive, CellToRelease2);
        }
        if IsCompressed && SearchName.MaximumLength as usize > core::mem::size_of::<[u16; 50]>() {
            (*Hive).Free.unwrap()(SearchName.Buffer as *mut c_void, 0);
        }
        return Result;
    }

    /* Decrement key counts and check if this was the last leaf entry */
    (*Node).SubKeyCounts[Storage as usize] -= 1;
    (*Leaf).Count -= 1;
    if (*Leaf).Count == 0 {
        /* Free the leaf */
        HvFreeCell(Hive, LeafCell);

        /* Check if we were inside a root */
        if !Root.is_null() {
            /* Decrease the root count too, since the leaf is going away */
            (*Root).Count -= 1;
            if (*Root).Count == 0 {
                /* The root is gone too, now */
                HvFreeCell(Hive, RootCell);
                (*Node).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Storage as usize] = HCELL_NIL;
            } else if RootIndex < (*Root).Count as u32 {
                /* Bring everything up by one */
                core::ptr::copy(
                    index_list_ptr(Root, RootIndex + 1) as *const u8,
                    index_list_ptr(Root, RootIndex) as *mut u8,
                    ((*Root).Count as u32 - RootIndex) as usize
                        * core::mem::size_of::<HCELL_INDEX>(),
                );
            }
        } else {
            /* Otherwise, just clear the cell */
            (*Node).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Storage as usize] = HCELL_NIL;
        }
    } else if LeafIndex < (*Leaf).Count as u32 {
        /* Was the leaf a normal index? */
        if (*Leaf).Signature == CM_KEY_INDEX_LEAF {
            /* Bring everything up by one */
            core::ptr::copy(
                index_list_ptr(Leaf, LeafIndex + 1) as *const u8,
                index_list_ptr(Leaf, LeafIndex) as *mut u8,
                ((*Leaf).Count as u32 - LeafIndex) as usize
                    * core::mem::size_of::<HCELL_INDEX>(),
            );
        } else {
            /* This is a fast index, bring everything up by one */
            let ChildFast = Leaf as PCM_KEY_FAST_INDEX;
            core::ptr::copy(
                fast_list_ptr(ChildFast, LeafIndex + 1) as *const u8,
                fast_list_ptr(ChildFast, LeafIndex) as *mut u8,
                ((*ChildFast).Count as u32 - LeafIndex) as usize
                    * core::mem::size_of::<CM_INDEX>(),
            );
        }
    }

    /* If we got here, now we're done */
    Result = 1;

    /* Exit: Release any cells we may have been holding */
    if CellToRelease1 != HCELL_NIL {
        HvReleaseCell(Hive, CellToRelease1);
    }
    if CellToRelease2 != HCELL_NIL {
        HvReleaseCell(Hive, CellToRelease2);
    }

    /* Check if the name was compressed and not inside our local buffer */
    if IsCompressed && SearchName.MaximumLength as usize > core::mem::size_of::<[u16; 50]>() {
        (*Hive).Free.unwrap()(SearchName.Buffer as *mut c_void, 0);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn CmpComputeHashKey(
    mut Hash: u32,
    Name: *const _UNICODE_STRING,
    AllowSeparators: u8,
) -> u32 {
    // ASSERT((Name->Length == 0) || (AllowSeparators) || (Name->Buffer[0] != OBJ_NAME_PATH_SEPARATOR));
    ntkrnl_sys::ASSERT!((*Name).Length == 0
            || AllowSeparators != 0
            || *(*Name).Buffer != OBJ_NAME_PATH_SEPARATOR as u16);

    // if (!Name->Length) return Hash;
    if (*Name).Length == 0 {
        return Hash;
    }

    // Set the buffer and loop every character
    let mut Cp: *mut u16 = (*Name).Buffer;
    let mut i: u32 = 0;
    while i < (*Name).Length as u32 {
        // ASSERT(AllowSeparators || (*Cp != OBJ_NAME_PATH_SEPARATOR));
        ntkrnl_sys::ASSERT!(AllowSeparators != 0 || *Cp != OBJ_NAME_PATH_SEPARATOR as u16);

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
    let _i: u32;
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

                ntkrnl_sys::ASSERT!(cell_to_release != HCELL_NIL);
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

            ntkrnl_sys::ASSERT!((*index_root).Signature == CM_KEY_INDEX_LEAF
                    || (*index_root).Signature == CM_KEY_FAST_LEAF
                    || (*index_root).Signature == CM_KEY_HASH_LEAF);

            if (*index_root).Signature != CM_KEY_HASH_LEAF {
                found = CmpFindSubKeyInLeaf(hive, index_root, search_name, &mut sub_key);

                ntkrnl_sys::ASSERT!(cell_to_release != HCELL_NIL);
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

                ntkrnl_sys::ASSERT!(cell_to_release != HCELL_NIL);
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
                    fast_list_ptr(FastLeaf, i),
                    fast_list_ptr(FastLeaf, i + 1),
                    ((*FastLeaf).Count as u32 - i) as usize,
                );
            } else {
                core::ptr::copy(
                    index_list_ptr(Leaf, i),
                    index_list_ptr(Leaf, i + 1),
                    ((*Leaf).Count as u32 - i) as usize,
                );
            }
        }
    }

    /* Check if this is a new-style leaf */
    if !FastLeaf.is_null() {
        let Entry = fast_list_ptr(FastLeaf, i);
        (*Entry).Cell = NewKey;

        if (*FastLeaf).Signature == CM_KEY_HASH_LEAF as u16 {
            (*Entry).__bindgen_anon_1.HashKey = CmpComputeHashKey(0, Name, 0u8);
        } else {
            (*Entry).__bindgen_anon_1.NameHint[0] = 0;
            (*Entry).__bindgen_anon_1.NameHint[1] = 0;
            (*Entry).__bindgen_anon_1.NameHint[2] = 0;
            (*Entry).__bindgen_anon_1.NameHint[3] = 0;

            if ((*Name).Length as usize / core::mem::size_of::<u16>()) < 4 {
                j = (*Name).Length as u32 / core::mem::size_of::<u16>() as u32;
            } else {
                j = 4;
            }

            loop {
                if (*(*Name).Buffer.offset((j - 1) as isize) as u16) > (u8::MAX as u16) {
                    break;
                }

                (*Entry).__bindgen_anon_1.NameHint[(j - 1) as usize] =
                    *(*Name).Buffer.offset((j - 1) as isize) as u8;

                j -= 1;
                if j == 0 {
                    break;
                }
            }
        }
    } else {
        core::ptr::write(index_list_ptr(Leaf, i), NewKey);
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
        ntkrnl_sys::ASSERT!(!IndexKey.is_null());
        return HCELL_NIL;
    }

    ntkrnl_sys::ASSERT!((*IndexKey).Signature == CM_KEY_INDEX_ROOT);

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
                *RootCell = index_list_ptr(IndexKey, SubKeyIndex) as PHCELL_INDEX;
                return LeafCell;
            }

            /* It didn't fit, so proceed to splitting */
        } else {
            LeafCell = *core::ptr::addr_of!((*IndexKey).List).cast::<HCELL_INDEX>().add(SubKeyIndex as usize);
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
                ntkrnl_sys::ASSERT!((*LeafKey).Signature == CM_KEY_INDEX_LEAF as u16);
                CurrentCell = (*LeafKey).List[0];
            }

            Result = CmpDoCompareKeyName(Hive, Name, CurrentCell);

            if Result == 2 {
                return HCELL_NIL;
            }

            ntkrnl_sys::ASSERT!(Result != 0);

            if Result >= 0 {
                LeafCell = *core::ptr::addr_of!((*IndexKey).List).cast::<HCELL_INDEX>().add(SubKeyIndex as usize);
                LeafKey = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;

                if LeafKey.is_null() {
                    return HCELL_NIL;
                }

                if ((*LeafKey).Count as u32) < CmpMaxIndexPerHblock {
                    // `addr_of_mut!((*IndexKey).List[i])` was the shape here, and it is
                    // NOT merely address-taking: an array index inside a raw-ref still
                    // emits a bounds check, so with bindgen rendering the C flexible
                    // array as `[HCELL_INDEX; 1]` it panics for every i >= 1 — i.e. for
                    // any root index past its first leaf. Verified: `&raw mut arr[i]`
                    // with a runtime i out of range aborts. Hence the accessor.
                    *RootCell = index_list_ptr(IndexKey, SubKeyIndex) as PHCELL_INDEX;
                    return LeafCell;
                }

                if (SubKeyIndex + 1) < (*IndexKey).Count as u32 {
                    LeafCell = *core::ptr::addr_of!((*IndexKey).List).cast::<HCELL_INDEX>().add((SubKeyIndex + 1) as usize);
                    LeafKey = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;

                    if LeafKey.is_null() {
                        return HCELL_NIL;
                    }

                    if ((*LeafKey).Count as u32) < CmpMaxIndexPerHblock {
                        *RootCell = index_list_ptr(IndexKey, SubKeyIndex + 1) as PHCELL_INDEX;
                        return LeafCell;
                    }
                }

                /* It didn't fit, so proceed to splitting */
            } else {
                /* No, it's below, check the subkey index */
                if SubKeyIndex > 0 {
                    LeafCell = *core::ptr::addr_of!((*IndexKey).List).cast::<HCELL_INDEX>().add((SubKeyIndex - 1) as usize);
                    LeafKey = HvGetCell(Hive, LeafCell) as PCM_KEY_INDEX;

                    if LeafKey.is_null() {
                        return HCELL_NIL;
                    }

                    if ((*LeafKey).Count as u32) < CmpMaxIndexPerHblock {
                        *RootCell = index_list_ptr(IndexKey, SubKeyIndex - 1) as PHCELL_INDEX;
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

        /* Make sure the new key became index root */
        ntkrnl_sys::ASSERT!((*IndexKey).Signature == CM_KEY_INDEX_ROOT);

        /* Now loop over with the new IndexKey value, which definitely has the
         * space now. The C's unreachable `return HCELL_NIL` after this loop has
         * no Rust counterpart: `loop` with no `break` diverges, so the tail is
         * not just dead but rejected. */
    }
}

// cmindex.c:19 macro — max CM_KEY_FAST_INDEX (fast-leaf) entries per hblock.
// C: ((HBLOCK_SIZE - (sizeof(HBIN)+sizeof(HCELL)+FIELD_OFFSET(CM_KEY_FAST_INDEX,List)))
//      / sizeof(CM_INDEX))
const CmpMaxFastIndexPerHblock: u32 = ((HBLOCK_SIZE as usize
    - (core::mem::size_of::<HBIN>()
        + core::mem::size_of::<HCELL>()
        + core::mem::offset_of!(CM_KEY_FAST_INDEX, List)))
    / core::mem::size_of::<CM_INDEX>()) as u32;

#[no_mangle]
pub unsafe extern "system" fn CmpAddSubKey(
    Hive: PHHIVE,
    Parent: HCELL_INDEX,
    Child: HCELL_INDEX,
) -> u8 {
    let mut KeyNode: PCM_KEY_NODE;
    let mut Index: PCM_KEY_INDEX;
    let OldIndex: PCM_KEY_FAST_INDEX;
    let mut Name = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut IndexCell: HCELL_INDEX = HCELL_NIL;
    let mut CellToRelease: HCELL_INDEX = HCELL_NIL;
    let mut LeafCell: HCELL_INDEX;
    let mut RootPointer: PHCELL_INDEX = core::ptr::null_mut();
    let Type: u32;
    let IsCompressed: bool;
    PAGED_CODE!();

    /* Get the key node */
    KeyNode = HvGetCell(Hive, Child) as PCM_KEY_NODE;
    if KeyNode.is_null() {
        /* Shouldn't happen */
        ASSERT!(false);
        return 0;
    }

    /* Check if the name is compressed */
    if ((*KeyNode).Flags & KEY_COMP_NAME) != 0 {
        /* Remember for later */
        IsCompressed = true;

        /* Create the compressed name and allocate it */
        Name.Length = CmpCompressedNameSize(
            core::ptr::addr_of_mut!((*KeyNode).Name) as *mut u16,
            (*KeyNode).NameLength as u32,
        );
        Name.MaximumLength = Name.Length;
        Name.Buffer = (*Hive).Allocate.unwrap()(Name.Length as _, 1u8, TAG_CM) as *mut u16;
        if Name.Buffer.is_null() {
            /* Release the cell and fail */
            HvReleaseCell(Hive, Child);
            ASSERT!(false);
            return 0;
        }

        /* Copy the compressed name */
        CmpCopyCompressedName(
            Name.Buffer,
            Name.MaximumLength as u32,
            core::ptr::addr_of_mut!((*KeyNode).Name) as *mut u16,
            (*KeyNode).NameLength as u32,
        );
    } else {
        /* Remember for later */
        IsCompressed = false;

        /* Build the unicode string */
        Name.Length = (*KeyNode).NameLength;
        Name.MaximumLength = (*KeyNode).NameLength;
        Name.Buffer = core::ptr::addr_of_mut!((*KeyNode).Name) as *mut u16;
    }

    /* Release the cell */
    HvReleaseCell(Hive, Child);

    /* Get the parent node */
    KeyNode = HvGetCell(Hive, Parent) as PCM_KEY_NODE;
    if KeyNode.is_null() {
        /* Not handled */
        ASSERT!(false);
    }

    /* Find out the type of the cell, and check if this is the first subkey */
    Type = HvGetCellType(Child);
    if (*KeyNode).SubKeyCounts[Type as usize] == 0 {
        /* Allocate a fast leaf */
        IndexCell = HvAllocateCell(
            Hive,
            core::mem::size_of::<CM_KEY_FAST_INDEX>() as u32,
            Type as HSTORAGE_TYPE,
            HCELL_NIL,
        );
        if IndexCell == HCELL_NIL {
            /* Not handled */
            ASSERT!(false);
        }

        /* Get the leaf cell */
        Index = HvGetCell(Hive, IndexCell) as PCM_KEY_INDEX;
        if Index.is_null() {
            /* Shouldn't happen */
            ASSERT!(false);
        }

        /* Now check what kind of hive we're dealing with */
        if (*Hive).Version >= 5 {
            /* XP Hive: Use hash leaf */
            (*Index).Signature = CM_KEY_HASH_LEAF;
        } else if (*Hive).Version >= 3 {
            /* Windows 2000 and ReactOS: Use fast leaf */
            (*Index).Signature = CM_KEY_FAST_LEAF;
        } else {
            /* NT 4: Use index leaf */
            (*Index).Signature = CM_KEY_INDEX_LEAF;
        }

        /* Setup the index list */
        (*Index).Count = 0;
        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize] = IndexCell;
    } else {
        /* We already have an index, get it */
        Index = HvGetCell(
            Hive,
            (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize],
        ) as PCM_KEY_INDEX;
        if Index.is_null() {
            /* Not handled */
            ASSERT!(false);
        }

        /* Remember to release the cell later */
        CellToRelease = (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize];

        /* Check if this is a fast leaf that's gotten too full */
        if (*Index).Signature == CM_KEY_FAST_LEAF
            && ((*Index).Count as u32) >= CmpMaxFastIndexPerHblock
        {
            /* Mark this cell as dirty */
            HvMarkCellDirty(Hive, CellToRelease, 0u8);

            /* Convert */
            OldIndex = Index as PCM_KEY_FAST_INDEX;

            let mut i: u32 = 0;
            while i < (*OldIndex).Count as u32 {
                *index_list_ptr(Index, i) = (*fast_list_ptr(OldIndex, i)).Cell;
                i += 1;
            }

            /* Set the new type value */
            (*Index).Signature = CM_KEY_INDEX_LEAF;
        } else if ((*Index).Signature == CM_KEY_INDEX_LEAF
            || (*Index).Signature == CM_KEY_HASH_LEAF)
            && ((*Index).Count as u32) >= CmpMaxIndexPerHblock
        {
            /* This is an old/hashed leaf that's gotten too large, root it */
            IndexCell = HvAllocateCell(
                Hive,
                (core::mem::size_of::<CM_KEY_INDEX>() + core::mem::size_of::<HCELL_INDEX>()) as u32,
                Type as HSTORAGE_TYPE,
                HCELL_NIL,
            );
            if IndexCell == HCELL_NIL {
                /* Not handled */
                ASSERT!(false);
            }

            /* Get the index cell */
            Index = HvGetCell(Hive, IndexCell) as PCM_KEY_INDEX;
            if Index.is_null() {
                /* Shouldn't happen */
                ASSERT!(false);
            }

            /* Mark the index as a root, and set the index cell */
            (*Index).Signature = CM_KEY_INDEX_ROOT;
            (*Index).Count = 1;
            *index_list_ptr(Index, 0) =
                (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize];
            (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize] = IndexCell;
        }
    }

    /* Now we can choose the leaf cell */
    LeafCell = (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize];

    /* Check if we turned the index into a root */
    if (*Index).Signature == CM_KEY_INDEX_ROOT {
        /* Get the leaf where to add the new entry (the routine will do
         * the splitting if necessary)
         */
        LeafCell = CmpSelectLeaf(Hive, KeyNode, &mut Name, Type as HSTORAGE_TYPE, &mut RootPointer);
        if LeafCell == HCELL_NIL {
            /* Not handled */
            ASSERT!(false);
        }
    }

    /* Add our leaf cell */
    LeafCell = CmpAddToLeaf(Hive, LeafCell, Child, &mut Name);
    if LeafCell == HCELL_NIL {
        /* Not handled */
        ASSERT!(false);
    }

    /* Update the key counts */
    (*KeyNode).SubKeyCounts[Type as usize] += 1;

    /* Check if caller wants us to return the leaf */
    if !RootPointer.is_null() {
        /* Return it */
        *RootPointer = LeafCell;
    } else {
        /* Otherwise, mark it as the list index for the cell */
        (*KeyNode).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Type as usize] = LeafCell;
    }

    /* If the name was compressed, free our copy */
    if IsCompressed {
        (*Hive).Free.unwrap()(Name.Buffer as *mut c_void, 0);
    }

    /* Release all our cells */
    if IndexCell != HCELL_NIL {
        HvReleaseCell(Hive, IndexCell);
    }
    if CellToRelease != HCELL_NIL {
        HvReleaseCell(Hive, CellToRelease);
    }
    HvReleaseCell(Hive, Parent);
    1
}
