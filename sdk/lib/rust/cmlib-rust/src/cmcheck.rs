//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/cmlib/cmcheck.c.
//! Risk: HIGH (3 real bugs in the two live #[no_mangle] fns; private fn bodies are dead code).
//! Behavior: 3 confirmed deviations — see BUGS below.
//!
//! ARCHITECTURE NOTE: CmpValidateRegistryInternal, CmpValidateKey, CmpValidateLexicalOrder,
//! CmpValidateClass, CmpValidateSubKeyList, CmpValidateValueList, CmpValidateValueListByCount,
//! and CmpPurgeVolatiles are ALL declared as extern C in this file (lines 23–35). Their Rust
//! bodies (lines 83–290) are therefore DEAD CODE at runtime — the C linker picks the C symbols.
//! The only live Rust code is HvValidateBin (line 293) and CmCheckRegistry (line 325).
//!
//! BUG 1 — HvValidateBin @ line 301: WRONG HBIN SIGNATURE (CRITICAL).
//!   Rust literal: 0x6E696248 ("Hbin"), correct: 0x6E696268 ("hbin") from glue::HV_HBIN_SIGNATURE.
//!   Effect: every valid bin is flagged as corrupt; CM_CHECK_REGISTRY_VALIDATE_HIVE always fails.
//!   Fix: replace the literal with the HV_HBIN_SIGNATURE constant from crate::glue.
//!
//! BUG 2 — CM_CHECK_REGISTRY_INVALID_PARAMETER @ line 81: WRONG VALUE.
//!   Rust: 0xC0000003u32 as i32 (-1073741821). C: 1 (cmlib.h:238).
//!   Effect: callers that compare the specific error code see a completely wrong value.
//!   CM_CHECK_REGISTRY_SUCCESS-only callers see non-zero either way (same flow), but the
//!   returned integer from CmCheckRegistry is wrong for any caller that logs or pattern-matches.
//!   Fix: set to 1.
//!
//! BUG 3 — CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES @ line 76: WRONG VALUE.
//!   Rust: 0x00000001. C: 0x0 (cmlib.h:228, intentionally 0 — it means "no purge flags set").
//!   Effect: the flags validation bitmask in CmCheckRegistry (line 345) accepts bit 0 as a
//!   valid flag (harmless in practice) but also shifts the no-op bit meaning.
//!   Fix: set to 0x0.
//!
//! WRONG BUT DEAD (constants used only in extern-C bodies, no runtime effect here):
//!   CM_KEY_VALUE_SIGNATURE line 53: Rust 0x6B765279, C 0x6B76 (u16). Unused in live code.
//!   CM_CHECK_REGISTRY_VALUE_LIST_UNALLOCATED/DATA_NOT_FOUND/SIZE_NOT_SANE lines 63-65: all 0
//!     (C: 22/23/24). Used only in CmpValidateValueList which is extern C.
//!   CM_CHECK_REGISTRY_KEY_CELL_NOT_ALLOCATED…BAD_KEY_NODE_SIGNATURE lines 68-74: wrong sequential
//!     values (C: 14-20). Used only in CmpValidateKey whose Rust body is dead code.
//!   HvValidateBin error codes (1/2/3 for BIN_SIGNATURE/BAD_FREE/BAD_ALLOC, C: 5/6/7): wrong enum
//!     values, but caller CM_CHECK_REGISTRY_SUCCESS check is == 0 so flow is unaffected.
//!   CM_CHECK_REGISTRY_KEY_CLASS_UNALLOCATED line 194: 1 (C: 21). Dead in extern CmpValidateClass.
//!
//! Unverified: CmpValidateRegistryInternal, CmpValidateLexicalOrder, CmpValidateKey body logic
//!   (all dead code; C bodies win at link time). HvValidateHive (extern C, not translated here).

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CELL_DATA, CM_CHECK_REGISTRY_STATUS, CM_KEY_NODE, CM_KEY_VALUE, CmpAllocate, CmpFindSubKeyByNumber, CmpFree, CmpRepairClassOfNodeKey, CmpRepairKeyNodeSignature, CmpRepairParentKey, CmpRepairParentNode, CmpRepairSubKeyCounts, CmpRepairSubKeyList, CmpRepairValueList, CmpRepairValueListCount, HBIN, HCELL, HCELL_INDEX, HHIVE, PCM_KEY_INDEX, CMHIVE, PHBIN, PHCELL, RtlCompareUnicodeString, SIZE_T, HSTORAGE_TYPE_Stable as Stable, HSTORAGE_TYPE_Volatile as Volatile, _HHIVE, _UNICODE_STRING};
use crate::glue::{HvGetCell, HvGetCellType, CmpIsKeyValueSmall, HCELL_NIL, HBLOCK_SIZE, CM_KEY_INDEX_ROOT, CM_KEY_INDEX_LEAF, CM_KEY_FAST_LEAF, CM_KEY_HASH_LEAF, CM_KEY_NODE_SIGNATURE, HV_HHIVE_SIGNATURE, HV_HBIN_SIGNATURE};
use core::ffi::c_void;

unsafe extern "C" {
    fn abs(x: i32) -> i32;
}
// CMAPI / Hv* are NTAPI = __stdcall on i386 (callee-pops, `_Name@N`). Declaring
// them `extern "C"` (cdecl, caller-pops) makes BOTH sides pop the args on i386,
// drifting ESP by N bytes per call — in the deep CmCheckRegistry tree-walk this
// accumulates until a `ret` grabs a stack data word (e.g. 0x20) → BUGCHECK 0x1E.
// (LLD's fuzzy `_foo`↔`_foo@N` match hides it at link time; invisible on
// amd64/arm64 where "system"==cdecl.) These MUST be `extern "system"`.
unsafe extern "system" {
    fn CmpCompareCompressedName(String: *const _UNICODE_STRING, Name: *const u16, NameLength: u32) -> i32;
    fn HvMarkCellDirty(Hive: *mut HHIVE, Cell: u32, Unk: u8) -> u8;
    fn HvIsCellAllocated(Hive: *mut HHIVE, Cell: u32) -> u8;
    fn HvGetCellSize(Hive: *mut _HHIVE, Cell: *mut c_void) -> u32;
}
unsafe extern "system" {
    // NTAPI. Resolvable in BOTH of cmlib's link contexts, which is why an assert here can
    // be loud rather than fatal: ntoskrnl exports RtlAssert, and freeldr defines its own
    // #[no_mangle] one in debug.rs. Declared locally rather than imported from
    // ntkrnl_sys::ddk because cmlib takes that crate with default-features = false.
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: u32, Message: *mut i8);
}
unsafe extern "C" {
    fn DPRINT1(format: *const i8, ...);
    fn ASSERT(condition: u32);
    fn DPRINT(format: *const u8, ...);
    #[cfg(not(feature = "bldr"))]
    static mut CmiVolatileHive: *mut CMHIVE;
}

// cmlib.h / hivedata.h macros, inlined (no external symbol):
//   GET_CELL_BIN(Bin)              = (PHCELL)((PUCHAR)Bin + sizeof(HBIN))
//   IsFreeCell(Cell)              = ((Cell)->Size >= 0)
//   CM_CHECK_REGISTRY_SUCCESS(s)  = ((ULONG)(s) == CM_CHECK_REGISTRY_GOOD)  [GOOD=0]
//   UNREFERENCED_PARAMETER(x)     = no-op
#[inline]
unsafe fn GET_CELL_BIN(Bin: PHBIN) -> PHCELL {
    (Bin as *mut u8).add(core::mem::size_of::<HBIN>()) as PHCELL
}
#[inline]
unsafe fn IsFreeCell(Cell: PHCELL) -> i32 {
    ((*Cell).Size >= 0) as i32
}
#[inline]
fn CM_CHECK_REGISTRY_SUCCESS(status: CM_CHECK_REGISTRY_STATUS) -> i32 {
    (status == 0) as i32
}
#[inline]
fn UNREFERENCED_PARAMETER(_param: u8) {}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct CMP_REGISTRY_STACK_WORK_STATE {
    pub ChildCellIndex: u32,
    pub Parent: HCELL_INDEX,
    pub Current: HCELL_INDEX,
    pub Sibling: HCELL_INDEX,
}
pub type PCMP_REGISTRY_STACK_WORK_STATE = *mut CMP_REGISTRY_STACK_WORK_STATE;

pub const CMP_PRIOR_STACK: u32 = 1;
pub const CMP_REGISTRY_MAX_LEVELS_TREE_DEPTH: u32 = 512;
pub const CMP_KEY_SIZE_THRESHOLD: u32 = 0x45C;
pub const CMP_VOLATILE_LIST_UNINTIALIZED: u32 = 0xBAADF00D;

// CM_KEY_VALUE.Signature is a USHORT (CM_KEY_VALUE_SIGNATURE = 'kv' = 0x6B76).
const CM_KEY_VALUE_SIGNATURE: u16 = 0x6B76;
const CM_CHECK_REGISTRY_GOOD: CM_CHECK_REGISTRY_STATUS = 0;
// Authoritative CM_CHECK_REGISTRY_STATUS codes (cmlib.h:237-276). These are now
// LIVE (returned by the real CmpValidate* bodies below), so they carry the real
// enum values, not the placeholder sequence the dead-code era used.
const CM_CHECK_REGISTRY_VALUE_CELL_NIL: CM_CHECK_REGISTRY_STATUS = 25;
const CM_CHECK_REGISTRY_VALUE_CELL_UNALLOCATED: CM_CHECK_REGISTRY_STATUS = 26;
const CM_CHECK_REGISTRY_VALUE_CELL_DATA_NOT_FOUND: CM_CHECK_REGISTRY_STATUS = 27;
const CM_CHECK_REGISTRY_VALUE_CELL_SIZE_NOT_SANE: CM_CHECK_REGISTRY_STATUS = 28;
const CM_CHECK_REGISTRY_CORRUPT_VALUE_DATA: CM_CHECK_REGISTRY_STATUS = 29;
const CM_CHECK_REGISTRY_DATA_CELL_NOT_ALLOCATED: CM_CHECK_REGISTRY_STATUS = 30;
const CM_CHECK_REGISTRY_BAD_KEY_VALUE_SIGNATURE: CM_CHECK_REGISTRY_STATUS = 31;

const CM_CHECK_REGISTRY_KEY_CLASS_UNALLOCATED: CM_CHECK_REGISTRY_STATUS = 21;

const CM_CHECK_REGISTRY_VALUE_LIST_UNALLOCATED: CM_CHECK_REGISTRY_STATUS = 22;
const CM_CHECK_REGISTRY_VALUE_LIST_DATA_NOT_FOUND: CM_CHECK_REGISTRY_STATUS = 23;
const CM_CHECK_REGISTRY_VALUE_LIST_SIZE_NOT_SANE: CM_CHECK_REGISTRY_STATUS = 24;

const CM_CHECK_REGISTRY_STABLE_KEYS_ON_VOLATILE: CM_CHECK_REGISTRY_STATUS = 32;
const CM_CHECK_REGISTRY_SUBKEYS_LIST_UNALLOCATED: CM_CHECK_REGISTRY_STATUS = 33;
const CM_CHECK_REGISTRY_CORRUPT_SUBKEYS_INDEX: CM_CHECK_REGISTRY_STATUS = 34;
const CM_CHECK_REGISTRY_BAD_SUBKEY_COUNT: CM_CHECK_REGISTRY_STATUS = 35;
const CM_CHECK_REGISTRY_KEY_INDEX_CELL_UNALLOCATED: CM_CHECK_REGISTRY_STATUS = 36;
const CM_CHECK_REGISTRY_CORRUPT_LEAF_ON_ROOT: CM_CHECK_REGISTRY_STATUS = 37;
const CM_CHECK_REGISTRY_CORRUPT_LEAF_SIGNATURE: CM_CHECK_REGISTRY_STATUS = 38;
const CM_CHECK_REGISTRY_CORRUPT_KEY_INDEX_SIGNATURE: CM_CHECK_REGISTRY_STATUS = 39;

// hivedata.h:71 — HSYS_WHISTLER_BETA1.
const HSYS_WHISTLER_BETA1: u32 = 4;

const CM_CHECK_REGISTRY_KEY_CELL_NOT_ALLOCATED: CM_CHECK_REGISTRY_STATUS = 14;
const CM_CHECK_REGISTRY_CELL_DATA_NOT_FOUND: CM_CHECK_REGISTRY_STATUS = 15;
const CM_CHECK_REGISTRY_CELL_SIZE_NOT_SANE: CM_CHECK_REGISTRY_STATUS = 16;
const CM_CHECK_REGISTRY_KEY_NAME_LENGTH_ZERO: CM_CHECK_REGISTRY_STATUS = 17;
const CM_CHECK_REGISTRY_KEY_TOO_BIG_THAN_CELL: CM_CHECK_REGISTRY_STATUS = 18;
const CM_CHECK_REGISTRY_BAD_KEY_NODE_PARENT: CM_CHECK_REGISTRY_STATUS = 19;
const CM_CHECK_REGISTRY_BAD_KEY_NODE_SIGNATURE: CM_CHECK_REGISTRY_STATUS = 20;

const CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES: u32 = 0x00000000; // BUG3 FIXED: was 0x1, C=0x0
const CM_CHECK_REGISTRY_PURGE_VOLATILES: u32 = 0x00000002;
const CM_CHECK_REGISTRY_BOOTLOADER_PURGE_VOLATILES: u32 = 0x00000004;
const CM_CHECK_REGISTRY_VALIDATE_HIVE: u32 = 0x00000008;
const CM_CHECK_REGISTRY_FIX_HIVE: u32 = 0x00000010;
const CM_CHECK_REGISTRY_INVALID_PARAMETER: CM_CHECK_REGISTRY_STATUS = 1; // BUG2 FIXED: was 0xC0000003u32 as i32, C=1

// cmlib.h status codes (authoritative values, cmlib.h:237-250)
const CM_CHECK_REGISTRY_HIVE_CORRUPT_SIGNATURE: CM_CHECK_REGISTRY_STATUS = 3;
const CM_CHECK_REGISTRY_BIN_SIZE_OR_OFFSET_CORRUPT: CM_CHECK_REGISTRY_STATUS = 4;
const CM_CHECK_REGISTRY_ALLOCATE_MEM_STACK_FAIL: CM_CHECK_REGISTRY_STATUS = 8;
const CM_CHECK_REGISTRY_ROOT_CELL_NOT_FOUND: CM_CHECK_REGISTRY_STATUS = 9;
const CM_CHECK_REGISTRY_BAD_LEXICOGRAPHICAL_ORDER: CM_CHECK_REGISTRY_STATUS = 10;
const CM_CHECK_REGISTRY_NODE_NOT_FOUND: CM_CHECK_REGISTRY_STATUS = 11;
const CM_CHECK_REGISTRY_SUBKEY_NOT_FOUND: CM_CHECK_REGISTRY_STATUS = 12;
const CM_CHECK_REGISTRY_TREE_TOO_MANY_LEVELS: CM_CHECK_REGISTRY_STATUS = 13;

// cmlib.h:216 — TAG_REGISTRY_STACK = 'sRMC' (little-endian DWORD)
const TAG_REGISTRY_STACK: u32 = u32::from_le_bytes([b'C', b'M', b'R', b's']);

#[inline(never)]
unsafe fn CmpValidateLexicalOrder(
    Hive: *mut _HHIVE,
    Child: HCELL_INDEX,
    Sibling: HCELL_INDEX,
) -> i32 {
    const KEY_COMP_NAME: u16 = 0x0020;

    let mut Result: i32;
    let mut ChildString: _UNICODE_STRING = core::mem::zeroed();
    let mut SiblingString: _UNICODE_STRING = core::mem::zeroed();
    let ChildNode: *mut CM_KEY_NODE;
    let SiblingNode: *mut CM_KEY_NODE;

    // PAGED_CODE();

    unsafe {
        /* Obtain the child node */
        ChildNode = HvGetCell(Hive, Child) as *mut CM_KEY_NODE;
        if ChildNode.is_null() {
            /* Couldn't get the child node, bail out */
            // DPRINT1!("Failed to get the child node\n");
            return 0;
        }

        /* Obtain the sibling node */
        SiblingNode = HvGetCell(Hive, Sibling) as *mut CM_KEY_NODE;
        if SiblingNode.is_null() {
            /* Couldn't get the sibling node, bail out */
            // DPRINT1!("Failed to get the sibling node\n");
            return 0;
        }

        /* CASE 1: Two distinct non-compressed Unicode names */
        if (core::ptr::addr_of!((*ChildNode).Flags).read_unaligned() & KEY_COMP_NAME) == 0
            && (core::ptr::addr_of!((*SiblingNode).Flags).read_unaligned() & KEY_COMP_NAME) == 0
        {
            /* Build the sibling string */
            SiblingString.Buffer = core::ptr::addr_of!((*SiblingNode).Name) as *mut u16;
            SiblingString.Length = core::ptr::addr_of!((*SiblingNode).NameLength).read_unaligned();
            SiblingString.MaximumLength = core::ptr::addr_of!((*SiblingNode).NameLength).read_unaligned();

            /* Build the child string */
            ChildString.Buffer = core::ptr::addr_of!((*ChildNode).Name) as *mut u16;
            ChildString.Length = core::ptr::addr_of!((*ChildNode).NameLength).read_unaligned();
            ChildString.MaximumLength = core::ptr::addr_of!((*ChildNode).NameLength).read_unaligned();

            Result = RtlCompareUnicodeString(
                core::ptr::addr_of!(SiblingString),
                core::ptr::addr_of!(ChildString),
                1,
            );
            if Result >= 0 {
                // DPRINT1!("The sibling node name is greater or equal to that of the child\n");
                return 0;
            }
        }

        /* CASE 2: Both compressed Unicode names */
        if (core::ptr::addr_of!((*ChildNode).Flags).read_unaligned() & KEY_COMP_NAME) != 0
            && (core::ptr::addr_of!((*SiblingNode).Flags).read_unaligned() & KEY_COMP_NAME) != 0
        {
            /* FIXME: Checks for two compressed names not implemented yet */
            // DPRINT!("Lexicographical order checks for two compressed names is UNIMPLEMENTED, assume the key is healthy...\n");
            return 1;
        }

        /* CASE 3: The child name is compressed but the sibling is not */
        if (core::ptr::addr_of!((*ChildNode).Flags).read_unaligned() & KEY_COMP_NAME) != 0
            && (core::ptr::addr_of!((*SiblingNode).Flags).read_unaligned() & KEY_COMP_NAME) == 0
        {
            SiblingString.Buffer = core::ptr::addr_of!((*SiblingNode).Name) as *mut u16;
            SiblingString.Length = core::ptr::addr_of!((*SiblingNode).NameLength).read_unaligned();
            SiblingString.MaximumLength = core::ptr::addr_of!((*SiblingNode).NameLength).read_unaligned();
            Result = CmpCompareCompressedName(
                core::ptr::addr_of!(SiblingString),
                core::ptr::addr_of!((*ChildNode).Name) as *const u16,
                (core::ptr::addr_of!((*ChildNode).NameLength).read_unaligned()) as _,
            );
            if Result >= 0 {
                // DPRINT1!("The sibling node name is greater or equal to that of the compressed child\n");
                return 0;
            }
        }

        /* CASE 4: The sibling name is compressed but the child is not */
        if (core::ptr::addr_of!((*SiblingNode).Flags).read_unaligned() & KEY_COMP_NAME) != 0
            && (core::ptr::addr_of!((*ChildNode).Flags).read_unaligned() & KEY_COMP_NAME) == 0
        {
            ChildString.Buffer = core::ptr::addr_of!((*ChildNode).Name) as *mut u16;
            ChildString.Length = core::ptr::addr_of!((*ChildNode).NameLength).read_unaligned();
            ChildString.MaximumLength = core::ptr::addr_of!((*ChildNode).NameLength).read_unaligned();
            Result = CmpCompareCompressedName(
                core::ptr::addr_of!(ChildString),
                core::ptr::addr_of!((*SiblingNode).Name) as *const u16,
                (core::ptr::addr_of!((*SiblingNode).NameLength).read_unaligned()) as _,
            );
            if Result <= 0 {
                // DPRINT1!("The compressed sibling node name is lesser or equal to that of the child\n");
                return 0;
            }
        }

        /*
         * One of the cases above has met the conditions
         * successfully, the lexicographical order is legal.
         */
        1
    }
}

/// C `static CmpValidateClass` — validates the class cell of a key node.
unsafe fn CmpValidateClass(
    Hive: *mut HHIVE,
    CurrentCell: HCELL_INDEX,
    CellData: *mut CELL_DATA,
) -> CM_CHECK_REGISTRY_STATUS {
    PAGED_CODE!();

    ASSERT(((CurrentCell != HCELL_NIL) as BOOLEAN) as _);
    ASSERT(((!CellData.is_null()) as BOOLEAN) as _);

    /* Cache the class cell and validate it (if any). CM_KEY_NODE is packed. */
    let class_cell: HCELL_INDEX =
        core::ptr::addr_of!((*CellData).u.KeyNode.Class).read_unaligned();
    let class_length: u32 =
        core::ptr::addr_of!((*CellData).u.KeyNode.ClassLength).read_unaligned() as u32;
    if class_length > 0 {
        if class_cell == HCELL_NIL {
            /*
             * Somebody has freed the class but left the
             * length as is, reset it.
             */
            HvMarkCellDirty(Hive, CurrentCell, 0);
            core::ptr::addr_of_mut!((*CellData).u.KeyNode.ClassLength).write_unaligned(0);
            return CM_CHECK_REGISTRY_GOOD;
        }

        if HvIsCellAllocated(Hive, class_cell) == 0 {
            return CM_CHECK_REGISTRY_KEY_CLASS_UNALLOCATED;
        }
    }

    CM_CHECK_REGISTRY_GOOD
}

/// C `static CmpValidateValueListByCount` — validates each value in a value list
/// by count, removing (self-healing) damaged values where possible.
unsafe fn CmpValidateValueListByCount(
    Hive: *mut HHIVE,
    CurrentCell: HCELL_INDEX,
    mut ListCount: u32,
    ValueListData: *mut CELL_DATA,
    ValuesRemoved: *mut u32,
    FixHive: u8,
) -> CM_CHECK_REGISTRY_STATUS {
    PAGED_CODE!();

    ASSERT(((!ValueListData.is_null()) as BOOLEAN) as _);
    ASSERT(((ListCount != 0) as BOOLEAN) as _);

    /* Assume we haven't removed any value counts for now */
    *ValuesRemoved = 0;

    /*
     * Begin looping each value in the list and validate it accordingly.
     * KeyList is the flexible array of HCELL_INDEX in CELL_DATA — pointer arith.
     */
    let key_list = core::ptr::addr_of!((*ValueListData).u.KeyList) as *const HCELL_INDEX;
    let mut list_count_index: u32 = 0;
    while list_count_index < ListCount {
        let value_cell: HCELL_INDEX = *key_list.add(list_count_index as usize);
        if value_cell == HCELL_NIL {
            if CmpRepairValueListCount(
                Hive,
                CurrentCell,
                list_count_index,
                ValueListData,
                FixHive,
            ) == 0
            {
                return CM_CHECK_REGISTRY_VALUE_CELL_NIL;
            }

            /* Decrease the list count and go to the next value */
            ListCount -= 1;
            *ValuesRemoved += 1;
            continue;
        }

        if HvIsCellAllocated(Hive, value_cell) == 0 {
            if CmpRepairValueListCount(
                Hive,
                CurrentCell,
                list_count_index,
                ValueListData,
                FixHive,
            ) == 0
            {
                return CM_CHECK_REGISTRY_VALUE_CELL_UNALLOCATED;
            }

            /* Decrease the list count and go to the next value */
            ListCount -= 1;
            *ValuesRemoved += 1;
            continue;
        }

        /* Obtain a cell data from this value */
        let value_data = HvGetCell(Hive, value_cell) as *mut CELL_DATA;
        if value_data.is_null() {
            return CM_CHECK_REGISTRY_VALUE_CELL_DATA_NOT_FOUND;
        }

        /* Check that the value size is sane */
        let value_data_size: u32 = HvGetCellSize(Hive, value_data as *mut c_void);
        let value_name_length: u32 =
            core::ptr::addr_of!((*value_data).u.KeyValue.NameLength).read_unaligned() as u32;
        let total_value_name_length: u32 =
            value_name_length + core::mem::offset_of!(CM_KEY_VALUE, Name) as u32;
        if total_value_name_length > value_data_size {
            if CmpRepairValueListCount(
                Hive,
                CurrentCell,
                list_count_index,
                ValueListData,
                FixHive,
            ) == 0
            {
                return CM_CHECK_REGISTRY_VALUE_CELL_SIZE_NOT_SANE;
            }

            /* Decrease the list count and go to the next value */
            ListCount -= 1;
            *ValuesRemoved += 1;
            continue;
        }

        /*
         * The value cell has a sane size. Validate the actual data of the
         * value cell. Small values are inlined into the cell and need no
         * further validation; normal values point at a data cell.
         */
        let data_cell: HCELL_INDEX =
            core::ptr::addr_of!((*value_data).u.KeyValue.Data).read_unaligned();
        let data_length: u32 =
            core::ptr::addr_of!((*value_data).u.KeyValue.DataLength).read_unaligned();
        let mut data_size: u32 = 0;
        if !CmpIsKeyValueSmall(core::ptr::addr_of_mut!(data_size), data_length) {
            /* Validate the actual data based on size */
            if data_size == 0 {
                if data_cell != HCELL_NIL {
                    if CmpRepairValueListCount(
                        Hive,
                        CurrentCell,
                        list_count_index,
                        ValueListData,
                        FixHive,
                    ) == 0
                    {
                        return CM_CHECK_REGISTRY_CORRUPT_VALUE_DATA;
                    }

                    /* Decrease the list count and go to the next value */
                    ListCount -= 1;
                    *ValuesRemoved += 1;
                    continue;
                }
            } else {
                if HvIsCellAllocated(Hive, data_cell) == 0 {
                    if CmpRepairValueListCount(
                        Hive,
                        CurrentCell,
                        list_count_index,
                        ValueListData,
                        FixHive,
                    ) == 0
                    {
                        return CM_CHECK_REGISTRY_DATA_CELL_NOT_ALLOCATED;
                    }

                    /* Decrease the list count and go to the next value */
                    ListCount -= 1;
                    *ValuesRemoved += 1;
                    continue;
                }
            }

            /* FIXME: Big values not supported yet — ASSERT_VALUE_BIG no-op (NDEBUG) */
            let _ = (&Hive, data_size);
        }

        /* Is the signature valid? (CM_KEY_VALUE is packed) */
        if core::ptr::addr_of!((*value_data).u.KeyValue.Signature).read_unaligned()
            != CM_KEY_VALUE_SIGNATURE
        {
            if CmpRepairValueListCount(
                Hive,
                CurrentCell,
                list_count_index,
                ValueListData,
                FixHive,
            ) == 0
            {
                return CM_CHECK_REGISTRY_BAD_KEY_VALUE_SIGNATURE;
            }

            /* Decrease the list count and go to the next value */
            ListCount -= 1;
            *ValuesRemoved += 1;
            continue;
        }

        /* Advance to the next value */
        list_count_index += 1;
    }

    CM_CHECK_REGISTRY_GOOD
}

/// C `static CmpValidateValueList` — validates the value list of a key,
/// delegating per-value checks to CmpValidateValueListByCount.
unsafe fn CmpValidateValueList(
    Hive: *mut HHIVE,
    CurrentCell: HCELL_INDEX,
    CellData: *mut CELL_DATA,
    FixHive: u8,
) -> CM_CHECK_REGISTRY_STATUS {
    PAGED_CODE!();

    ASSERT(((CurrentCell != HCELL_NIL) as BOOLEAN) as _);
    ASSERT(((!CellData.is_null()) as BOOLEAN) as _);

    /* Cache the value list and validate it (CM_KEY_NODE is packed) */
    let value_list_cell: HCELL_INDEX = core::ptr::addr_of!(
        (*CellData).u.KeyNode.__bindgen_anon_1.__bindgen_anon_1.ValueList.List
    )
    .read_unaligned();
    let value_list_count: u32 = core::ptr::addr_of!(
        (*CellData).u.KeyNode.__bindgen_anon_1.__bindgen_anon_1.ValueList.Count
    )
    .read_unaligned();
    if value_list_count > 0 {
        if HvIsCellAllocated(Hive, value_list_cell) == 0 {
            return CM_CHECK_REGISTRY_VALUE_LIST_UNALLOCATED;
        }

        /* Obtain cell data from the value list cell */
        let value_list_data = HvGetCell(Hive, value_list_cell) as *mut CELL_DATA;
        if value_list_data.is_null() {
            return CM_CHECK_REGISTRY_VALUE_LIST_DATA_NOT_FOUND;
        }

        /*
         * Cache the value size and total length and ensure this is a sane
         * value list to begin with.
         */
        let value_size: u32 = HvGetCellSize(Hive, value_list_data as *mut c_void);
        let total_value_length: u32 =
            value_list_count.wrapping_mul(core::mem::size_of::<HCELL_INDEX>() as u32);
        if total_value_length > value_size {
            return CM_CHECK_REGISTRY_VALUE_LIST_SIZE_NOT_SANE;
        }

        /*
         * The value list is sane, validate the actual list by its count.
         */
        let mut values_removed: u32 = 0;
        let cm_status_code = CmpValidateValueListByCount(
            Hive,
            CurrentCell,
            value_list_count,
            value_list_data,
            core::ptr::addr_of_mut!(values_removed),
            FixHive,
        );
        if CM_CHECK_REGISTRY_SUCCESS(cm_status_code) == 0 {
            return cm_status_code;
        }

        /* (values_removed logged in C via DPRINT1 — stripped here) */
    }

    CM_CHECK_REGISTRY_GOOD
}

/// C `static CmpValidateSubKeyList` — validates the subkey list of a key. May
/// flag the list as repairable (DoRepair) so the caller can purge it.
unsafe fn CmpValidateSubKeyList(
    Hive: *mut HHIVE,
    CurrentCell: HCELL_INDEX,
    CellData: *mut CELL_DATA,
    FixHive: u8,
    DoRepair: *mut u8,
) -> CM_CHECK_REGISTRY_STATUS {
    PAGED_CODE!();

    ASSERT(((CurrentCell != HCELL_NIL) as BOOLEAN) as _);
    ASSERT(((!CellData.is_null()) as BOOLEAN) as _);

    let mut total_leaf_count: u32 = 0;

    /*
     * Assume for now that the caller should not do any kind of repairs on the
     * subkeys list, unless explicitly given the consent by us.
     */
    *DoRepair = 0;

    /*
     * For volatile keys there's nothing we can validate, but we still want to
     * ensure the volatile key is not damaged (e.g. stable keys on a volatile
     * space). IS_CELL_VOLATILE(CurrentCell) = HvGetCellType(CurrentCell) == Volatile.
     */
    if HvGetCellType(CurrentCell) == Volatile as u32 {
        let stable_subkey_counts: u32 = core::ptr::addr_of!(
            (*CellData).u.KeyNode.SubKeyCounts[Stable as usize]
        )
        .read_unaligned();
        if stable_subkey_counts != 0 {
            return CM_CHECK_REGISTRY_STABLE_KEYS_ON_VOLATILE;
        }

        return CM_CHECK_REGISTRY_GOOD;
    }

    /*
     * This is not a volatile key, cache the (stable) subkeys list and validate
     * it. SubKeyLists/SubKeyCounts live in the packed CM_KEY_NODE.
     */
    let sub_keys_list_cell: HCELL_INDEX = core::ptr::addr_of!(
        (*CellData).u.KeyNode.__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize]
    )
    .read_unaligned();
    let sub_key_counts: u32 = core::ptr::addr_of!(
        (*CellData).u.KeyNode.SubKeyCounts[Stable as usize]
    )
    .read_unaligned();
    if sub_key_counts > 0 {
        if HvIsCellAllocated(Hive, sub_keys_list_cell) == 0 {
            *DoRepair = 1;
            return CM_CHECK_REGISTRY_SUBKEYS_LIST_UNALLOCATED;
        }

        /* Obtain a root index and validate it */
        let root_key_index = HvGetCell(Hive, sub_keys_list_cell) as PCM_KEY_INDEX;
        if root_key_index.is_null() {
            return CM_CHECK_REGISTRY_CORRUPT_SUBKEYS_INDEX;
        }

        let root_signature: u16 =
            core::ptr::addr_of!((*root_key_index).Signature).read_unaligned();
        let root_count: u32 =
            core::ptr::addr_of!((*root_key_index).Count).read_unaligned() as u32;

        /*
         * For simple, fast and hashed leaves we want the root index count to
         * match the subkey counts. If not, isolate and fix the count.
         */
        if root_signature == CM_KEY_INDEX_LEAF
            || root_signature == CM_KEY_FAST_LEAF
            || root_signature == CM_KEY_HASH_LEAF
        {
            if sub_key_counts != root_count {
                if CmpRepairSubKeyCounts(Hive, CurrentCell, root_count, CellData, FixHive) == 0 {
                    return CM_CHECK_REGISTRY_BAD_SUBKEY_COUNT;
                }
            }

            return CM_CHECK_REGISTRY_GOOD;
        }

        /*
         * The root index is not a leaf, check if it's an actual root then.
         */
        if root_signature == CM_KEY_INDEX_ROOT {
            /*
             * For the root we loop each leaf, validate it and accumulate the
             * total leaf count so we can compare against the subkey counts.
             * CM_KEY_INDEX.List is a flexible array of HCELL_INDEX — pointer arith.
             */
            let root_list = core::ptr::addr_of!((*root_key_index).List) as *const HCELL_INDEX;
            let mut root_index: u32 = 0;
            while root_index < root_count {
                let key_index_cell: HCELL_INDEX = *root_list.add(root_index as usize);
                if HvIsCellAllocated(Hive, key_index_cell) == 0 {
                    *DoRepair = 1;
                    return CM_CHECK_REGISTRY_KEY_INDEX_CELL_UNALLOCATED;
                }

                /* Obtain a leaf from the root */
                let leaf_key_index = HvGetCell(Hive, key_index_cell) as PCM_KEY_INDEX;
                if leaf_key_index.is_null() {
                    return CM_CHECK_REGISTRY_CORRUPT_LEAF_ON_ROOT;
                }

                let leaf_signature: u16 =
                    core::ptr::addr_of!((*leaf_key_index).Signature).read_unaligned();
                /* Check that the leaf has a valid signature */
                if leaf_signature != CM_KEY_INDEX_LEAF
                    && leaf_signature != CM_KEY_FAST_LEAF
                    && leaf_signature != CM_KEY_HASH_LEAF
                {
                    *DoRepair = 1;
                    return CM_CHECK_REGISTRY_CORRUPT_LEAF_SIGNATURE;
                }

                /* Add up the count of the leaf */
                total_leaf_count +=
                    core::ptr::addr_of!((*leaf_key_index).Count).read_unaligned() as u32;

                root_index += 1;
            }

            /*
             * We have built up the total leaf count; it must match the subkey
             * counts. Otherwise just fix it.
             */
            if sub_key_counts != total_leaf_count {
                if CmpRepairSubKeyCounts(Hive, CurrentCell, total_leaf_count, CellData, FixHive)
                    == 0
                {
                    return CM_CHECK_REGISTRY_BAD_SUBKEY_COUNT;
                }
            }

            return CM_CHECK_REGISTRY_GOOD;
        }

        /*
         * None of the valid signatures match the root key index. The whole
         * subkey list is total toast.
         */
        *DoRepair = 1;
        return CM_CHECK_REGISTRY_CORRUPT_KEY_INDEX_SIGNATURE;
    }

    /* If we reach here then this key has no subkeys */
    CM_CHECK_REGISTRY_GOOD
}

/// C `static CmpPurgeVolatiles` — purges the volatile storage of a hive during
/// boot when the caller requests it via Flags.
#[inline(never)]
unsafe fn CmpPurgeVolatiles(
    Hive: *mut HHIVE,
    CurrentCell: HCELL_INDEX,
    CellData: *mut CELL_DATA,
    Flags: u32,
) {
    PAGED_CODE!();

    ASSERT(((!CellData.is_null()) as BOOLEAN) as _);

    /* Did the caller ask to purge volatiles, and are there any? */
    let volatile_subkey_counts: u32 = core::ptr::addr_of!(
        (*CellData).u.KeyNode.SubKeyCounts[Volatile as usize]
    )
    .read_unaligned();
    if ((Flags & CM_CHECK_REGISTRY_PURGE_VOLATILES) != 0
        || (Flags & CM_CHECK_REGISTRY_BOOTLOADER_PURGE_VOLATILES) != 0)
        && volatile_subkey_counts != 0
    {
        /*
         * The caller wants them cleaned. For XP Beta 1 or newer hives we
         * uninitialize the whole volatile subkeys list; for older hives we
         * just do a cleanup. (The C `#if !defined(_BLDR_)` kernel build always
         * marks the cell dirty.)
         */
        #[cfg(not(feature = "bldr"))]
        HvMarkCellDirty(Hive, CurrentCell, 0);

        let new_list: HCELL_INDEX = if (Flags & CM_CHECK_REGISTRY_BOOTLOADER_PURGE_VOLATILES) != 0
            && core::ptr::addr_of!((*Hive).Version).read_unaligned() >= HSYS_WHISTLER_BETA1
        {
            CMP_VOLATILE_LIST_UNINTIALIZED
        } else {
            HCELL_NIL
        };
        core::ptr::addr_of_mut!(
            (*CellData).u.KeyNode.__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Volatile as usize]
        )
        .write_unaligned(new_list);

        /* Clear the count */
        core::ptr::addr_of_mut!((*CellData).u.KeyNode.SubKeyCounts[Volatile as usize])
            .write_unaligned(0);
    }
}

#[inline(never)]
unsafe fn CmpValidateKey(
    Hive: *mut HHIVE,
    SecurityDefaulted: u8,
    ParentCell: HCELL_INDEX,
    CurrentCell: HCELL_INDEX,
    Flags: u32,
    FixHive: u8,
) -> CM_CHECK_REGISTRY_STATUS {
    let mut cm_status_code: CM_CHECK_REGISTRY_STATUS = 0;
    let cell_data: *mut CELL_DATA;
    let cell_size: u32;
    let mut do_subkeys_repair: u8 = 0u8;
    let total_key_name_length: u32;
    let name_length: u32;

    PAGED_CODE!();

    ASSERT(((CurrentCell != HCELL_NIL) as BOOLEAN) as _);

    UNREFERENCED_PARAMETER(SecurityDefaulted);

    if HvIsCellAllocated(Hive, CurrentCell) == 0 {
        // DPRINT removed
        return CM_CHECK_REGISTRY_KEY_CELL_NOT_ALLOCATED;
    }

    cell_data = HvGetCell(Hive, CurrentCell) as *mut CELL_DATA;
    if cell_data.is_null() {
        // DPRINT removed
        return CM_CHECK_REGISTRY_CELL_DATA_NOT_FOUND;
    }

    cell_size = HvGetCellSize(Hive, cell_data as *mut c_void);
    if cell_size > CMP_KEY_SIZE_THRESHOLD {
        // DPRINT removed
        return CM_CHECK_REGISTRY_CELL_SIZE_NOT_SANE;
    }

    name_length = (*cell_data).u.KeyNode.NameLength as u32;
    if name_length == 0 {
        // DPRINT removed
        return CM_CHECK_REGISTRY_KEY_NAME_LENGTH_ZERO;
    }

    total_key_name_length = name_length + core::mem::offset_of!(CM_KEY_NODE, Name) as u32;
    if total_key_name_length > cell_size {
        // DPRINT removed
        return CM_CHECK_REGISTRY_KEY_TOO_BIG_THAN_CELL;
    }

    if ParentCell != HCELL_NIL && ParentCell != (*cell_data).u.KeyNode.Parent {
        if CmpRepairParentNode(Hive, CurrentCell, ParentCell, cell_data, FixHive) == 0 {
            // DPRINT removed
            return CM_CHECK_REGISTRY_BAD_KEY_NODE_PARENT;
        }
    }

    if (*cell_data).u.KeyNode.Signature != CM_KEY_NODE_SIGNATURE {
        if CmpRepairKeyNodeSignature(Hive, CurrentCell, cell_data, FixHive) == 0 {
            // DPRINT removed
            return CM_CHECK_REGISTRY_BAD_KEY_NODE_SIGNATURE;
        }
    }

    cm_status_code = CmpValidateClass(Hive, CurrentCell, cell_data);
    if CM_CHECK_REGISTRY_SUCCESS(cm_status_code) == 0 {
        if CmpRepairClassOfNodeKey(Hive, CurrentCell, cell_data, FixHive) == 0 {
            // DPRINT removed
            return cm_status_code;
        }
    }

    cm_status_code = CmpValidateValueList(Hive, CurrentCell, cell_data, FixHive);
    if CM_CHECK_REGISTRY_SUCCESS(cm_status_code) == 0 {
        if CmpRepairValueList(Hive, CurrentCell, FixHive) == 0 {
            // DPRINT removed
            return cm_status_code;
        }
    }

    cm_status_code = CmpValidateSubKeyList(Hive, CurrentCell, cell_data, FixHive, core::ptr::addr_of_mut!(do_subkeys_repair) as *mut u8);
    if CM_CHECK_REGISTRY_SUCCESS(cm_status_code) == 0 {
        if do_subkeys_repair == 0 {
            // DPRINT removed
            return cm_status_code;
        }
        if CmpRepairSubKeyList(Hive, CurrentCell, cell_data, FixHive) == 0 {
            // DPRINT removed
            return cm_status_code;
        }
    }

    CmpPurgeVolatiles(Hive, CurrentCell, cell_data, Flags);
    return CM_CHECK_REGISTRY_GOOD;
}

#[no_mangle]
pub unsafe extern "system" fn HvValidateBin(Hive: *mut HHIVE, Bin: *mut HBIN) -> i32 {
    PAGED_CODE!();
    if !(!Bin.is_null()) {
        RtlAssert(
            b"Bin\0".as_ptr() as *mut core::ffi::c_void,
            b"cmcheck.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            core::ptr::null_mut::<i8>(),
        );
    }
    if !(!Hive.is_null()) {
        RtlAssert(
            b"Hive\0".as_ptr() as *mut core::ffi::c_void,
            b"cmcheck.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            core::ptr::null_mut::<i8>(),
        );
    }
    if (*Bin).Signature != HV_HBIN_SIGNATURE { // BUG1 FIXED: was 0x6E696248 ("Hbin"), correct is 0x6E696268 ("hbin")
        // DPRINT removed
        return 1; // CM_CHECK_REGISTRY_BIN_SIGNATURE_HEADER_CORRUPT
    }
    let Basket = (Bin as *mut u8).add((*Bin).Size as usize) as *mut HCELL;
    let mut Cell = GET_CELL_BIN(Bin);
    while Cell < Basket {
        if IsFreeCell(Cell) != 0 {
            if (*Cell).Size as u32 > (*Bin).Size || (*Cell).Size == 0 {
                // DPRINT removed
                return 2; // CM_CHECK_REGISTRY_BAD_FREE_CELL
            }
        } else {
            if abs((*Cell).Size) as u32 > (*Bin).Size {
                // DPRINT removed
                return 3; // CM_CHECK_REGISTRY_BAD_ALLOC_CELL
            }
        }
        Cell = (Cell as *mut u8).add(abs((*Cell).Size) as usize) as *mut HCELL;
    }
    return 0; // CM_CHECK_REGISTRY_GOOD
}

/// C `static CmpValidateRegistryInternal` — deep registry tree walk using a
/// stack-based pool buffer. Private to this module.
#[inline(never)]
unsafe fn CmpValidateRegistryInternal(
    Hive: *mut HHIVE,
    Flags: u32,
    SecurityDefaulted: i32,
    FixHive: i32,
) -> CM_CHECK_REGISTRY_STATUS {
    let mut CmStatusCode: CM_CHECK_REGISTRY_STATUS;
    let RootCell: HCELL_INDEX;
    let mut ParentCell: HCELL_INDEX;
    let mut CurrentCell: HCELL_INDEX;
    let mut ChildSubKeyCell: HCELL_INDEX;
    let mut KeyNode: *mut CM_KEY_NODE;
    let WorkStateLength: u32;
    let mut StackDepth: i32;
    let mut AllChildrenChecked: i32;

    PAGED_CODE!();

    ASSERT(((!Hive.is_null()) as BOOLEAN) as _);

    /*
     * Allocate some memory blocks for the stack state structure so we can
     * walk the hive tree iteratively without blowing the kernel stack.
     */
    WorkStateLength = CMP_REGISTRY_MAX_LEVELS_TREE_DEPTH
        * core::mem::size_of::<CMP_REGISTRY_STACK_WORK_STATE>() as u32;
    let WorkState = CmpAllocate(WorkStateLength as SIZE_T, 1u8, TAG_REGISTRY_STACK)
        as *mut CMP_REGISTRY_STACK_WORK_STATE;
    if WorkState.is_null() {
        // DPRINT removed
        return CM_CHECK_REGISTRY_ALLOCATE_MEM_STACK_FAIL;
    }

    /* Obtain the root cell of the hive. */
    RootCell = (*(*Hive).BaseBlock).RootCell;
    if RootCell == HCELL_NIL {
        // DPRINT removed
        CmpFree(WorkState as *mut c_void, WorkStateLength);
        return CM_CHECK_REGISTRY_ROOT_CELL_NOT_FOUND;
    }

    'restart: loop {
        /*
         * Prepare the stack state and start from the root cell.
         */
        StackDepth = 0;
        (*WorkState.add(StackDepth as usize)).ChildCellIndex = 0;
        (*WorkState.add(StackDepth as usize)).Current = RootCell;
        (*WorkState.add(StackDepth as usize)).Parent = HCELL_NIL;
        (*WorkState.add(StackDepth as usize)).Sibling = HCELL_NIL;

        AllChildrenChecked = 0;

        while StackDepth >= 0 {
            /* Cache the current and parent cells */
            CurrentCell = (*WorkState.add(StackDepth as usize)).Current;
            ParentCell = (*WorkState.add(StackDepth as usize)).Parent;

            /* Do we still have children to validate? */
            if AllChildrenChecked == 0 {
                /* Check that the key is OK */
                CmStatusCode = CmpValidateKey(
                    Hive,
                    SecurityDefaulted as u8,
                    ParentCell,
                    CurrentCell,
                    Flags,
                    FixHive as u8,
                );
                if CM_CHECK_REGISTRY_SUCCESS(CmStatusCode) == 0 {
                    /*
                     * The key cell is damaged. If this is the root cell the
                     * damage is fatal.
                     */
                    if CurrentCell == RootCell {
                        // DPRINT removed
                        CmpFree(WorkState as *mut c_void, WorkStateLength);
                        return CmStatusCode;
                    }

                    /*
                     * It is not the root, remove the faulting damaged cell
                     * from the parent so that we can heal the hive.
                     */
                    if CmpRepairParentKey(Hive, CurrentCell, ParentCell, FixHive as u8) == 0 {
                        // DPRINT removed
                        CmpFree(WorkState as *mut c_void, WorkStateLength);
                        return CmStatusCode;
                    }

                    /* Damaged cell removed, restart the loop */
                    continue 'restart;
                }

                /*
                 * The key is in perfect shape. If we have advanced the stack
                 * depth then check the lexicographical order of the keys.
                 */
                if StackDepth > 0 && CM_CHECK_REGISTRY_SUCCESS(CmStatusCode) != 0 {
                    let prior = StackDepth - CMP_PRIOR_STACK as i32;
                    if (*WorkState.add(prior as usize)).Sibling != HCELL_NIL {
                        if CmpValidateLexicalOrder(
                            Hive,
                            CurrentCell,
                            (*WorkState.add(prior as usize)).Sibling,
                        ) == 0
                        {
                            /*
                             * The lexicographical order is bad, attempt to
                             * heal the hive.
                             */
                            if CmpRepairParentKey(Hive, CurrentCell, ParentCell, FixHive as u8) == 0
                            {
                                // DPRINT removed
                                CmpFree(WorkState as *mut c_void, WorkStateLength);
                                return CM_CHECK_REGISTRY_BAD_LEXICOGRAPHICAL_ORDER;
                            }

                            /* Damaged cell removed, restart the loop */
                            continue 'restart;
                        }
                    }

                    /* Assign the prior sibling for the upcoming iteration */
                    (*WorkState.add(prior as usize)).Sibling = CurrentCell;
                }
            }

            /* Obtain a node for this key */
            KeyNode = HvGetCell(Hive, CurrentCell) as *mut CM_KEY_NODE;
            if KeyNode.is_null() {
                // DPRINT removed
                CmpFree(WorkState as *mut c_void, WorkStateLength);
                return CM_CHECK_REGISTRY_NODE_NOT_FOUND;
            }

            /*
             * If we have processed all the children from this node, go back
             * up the stack. Otherwise descend into the next child subkey.
             * SubKeyCounts[Stable] == index 0.
             */
            if (*WorkState.add(StackDepth as usize)).ChildCellIndex
                < (*KeyNode).SubKeyCounts[Stable as usize]
            {
                /* Obtain the child subkey in question */
                ChildSubKeyCell = CmpFindSubKeyByNumber(
                    Hive,
                    KeyNode,
                    (*WorkState.add(StackDepth as usize)).ChildCellIndex,
                );
                if ChildSubKeyCell == HCELL_NIL {
                    // DPRINT removed
                    CmpFree(WorkState as *mut c_void, WorkStateLength);
                    return CM_CHECK_REGISTRY_SUBKEY_NOT_FOUND;
                }

                /*
                 * Advance the child index and the stack depth. Ensure we do
                 * not exceed the maximum tree level depth (512).
                 */
                (*WorkState.add(StackDepth as usize)).ChildCellIndex += 1;
                StackDepth += 1;
                if StackDepth >= CMP_REGISTRY_MAX_LEVELS_TREE_DEPTH as i32 - 1 {
                    // DPRINT removed
                    CmpFree(WorkState as *mut c_void, WorkStateLength);
                    return CM_CHECK_REGISTRY_TREE_TOO_MANY_LEVELS;
                }

                /* Prepare the work state for the next key */
                let prior = StackDepth - CMP_PRIOR_STACK as i32;
                (*WorkState.add(StackDepth as usize)).ChildCellIndex = 0;
                (*WorkState.add(StackDepth as usize)).Current = ChildSubKeyCell;
                (*WorkState.add(StackDepth as usize)).Parent =
                    (*WorkState.add(prior as usize)).Current;
                (*WorkState.add(StackDepth as usize)).Sibling = HCELL_NIL;

                AllChildrenChecked = 0;
                continue;
            }

            /*
             * We have validated all the child subkeys of the node. Decrease
             * the stack depth and signal that all children were checked.
             */
            AllChildrenChecked = 1;
            StackDepth -= 1;
        }

        CmpFree(WorkState as *mut c_void, WorkStateLength);
        return CM_CHECK_REGISTRY_GOOD;
    }
}

/// C `HvValidateHive` (NTAPI/CMAPI) — validates each bin in every storage.
#[no_mangle]
pub unsafe extern "system" fn HvValidateHive(Hive: *mut HHIVE) -> CM_CHECK_REGISTRY_STATUS {
    let mut CmStatusCode: CM_CHECK_REGISTRY_STATUS;
    let mut StorageIndex: u32;
    let mut BlockIndex: u32;
    let mut StorageLength: u32;
    let mut Bin: PHBIN;

    PAGED_CODE!();

    ASSERT(((!Hive.is_null()) as BOOLEAN) as _);

    /* Is the hive signature valid? */
    if (*Hive).Signature != HV_HHIVE_SIGNATURE {
        // DPRINT removed
        return CM_CHECK_REGISTRY_HIVE_CORRUPT_SIGNATURE;
    }

    /* Loop each bin in the storage of this hive */
    StorageIndex = 0;
    while StorageIndex < (*Hive).StorageTypeCount {
        StorageLength = (*Hive).Storage[StorageIndex as usize].Length;

        BlockIndex = 0;
        while BlockIndex < StorageLength {
            let block_list = (*Hive).Storage[StorageIndex as usize].BlockList;
            /* Go to the next if this bin does not exist */
            if (*block_list.add(BlockIndex as usize)).BinAddress == 0 {
                // NOTE: faithful to C — the C loop `continue`s without
                // advancing BlockIndex here (BlockIndex only advances at the
                // bottom via Bin->Size). A genuinely NULL BinAddress would spin,
                // but in practice BinAddress is populated for in-range blocks.
                continue;
            }

            /*
             * Capture this bin: GET_HHIVE_BIN(Hive, StorageIndex, BlockIndex)
             * = (PHBIN)Hive->Storage[StorageIndex].BlockList[BlockIndex].BinAddress
             */
            Bin = (*block_list.add(BlockIndex as usize)).BinAddress as PHBIN;
            if (*Bin).Size > StorageLength.wrapping_mul(HBLOCK_SIZE)
                || ((*Bin).FileOffset / HBLOCK_SIZE) != BlockIndex
            {
                // DPRINT removed
                return CM_CHECK_REGISTRY_BIN_SIZE_OR_OFFSET_CORRUPT;
            }

            /* Validate the rest of the bin */
            CmStatusCode = HvValidateBin(Hive, Bin) as CM_CHECK_REGISTRY_STATUS;
            if CM_CHECK_REGISTRY_SUCCESS(CmStatusCode) == 0 {
                // DPRINT removed
                return CmStatusCode;
            }

            /* Go to the next block */
            BlockIndex += (*Bin).Size / HBLOCK_SIZE;
        }

        StorageIndex += 1;
    }

    CM_CHECK_REGISTRY_GOOD
}

#[no_mangle]
pub unsafe extern "system" fn CmCheckRegistry(
    RegistryHive: *mut CMHIVE,
    Flags: u32,
) -> i32 {
    let mut CmStatusCode: i32;
    let Hive: *mut HHIVE;
    let mut ShouldFixHive: i32 = 0;

    PAGED_CODE!();

    if RegistryHive.is_null() {
        // DPRINT removed
        return CM_CHECK_REGISTRY_INVALID_PARAMETER as i32;
    }

    #[cfg(not(feature = "bldr"))]
    {
        if RegistryHive == CmiVolatileHive {
            // DPRINT removed
            return CM_CHECK_REGISTRY_GOOD as i32;
        }
    }

    if Flags & !(CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES |
                 CM_CHECK_REGISTRY_PURGE_VOLATILES |
                 CM_CHECK_REGISTRY_BOOTLOADER_PURGE_VOLATILES |
                 CM_CHECK_REGISTRY_VALIDATE_HIVE |
                 CM_CHECK_REGISTRY_FIX_HIVE) != 0 {
        // DPRINT removed
        return CM_CHECK_REGISTRY_INVALID_PARAMETER as i32;
    }

    Hive = core::ptr::addr_of_mut!((*RegistryHive).Hive);

    if Flags & CM_CHECK_REGISTRY_VALIDATE_HIVE != 0 {
        CmStatusCode = HvValidateHive(Hive) as i32;
        if CM_CHECK_REGISTRY_SUCCESS(CmStatusCode as CM_CHECK_REGISTRY_STATUS) == 0 {
            // DPRINT removed
            return CmStatusCode;
        }
    }

    if Flags & CM_CHECK_REGISTRY_FIX_HIVE != 0 {
        ShouldFixHive = 1;
    }

    CmStatusCode = CmpValidateRegistryInternal(Hive, Flags, 0, ShouldFixHive) as i32;
    if CM_CHECK_REGISTRY_SUCCESS(CmStatusCode as CM_CHECK_REGISTRY_STATUS) == 0 {
        // DPRINT removed
        return CmStatusCode;
    }

    return CmStatusCode;
}
