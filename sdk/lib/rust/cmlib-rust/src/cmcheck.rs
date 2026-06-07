use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CELL_DATA, CM_CHECK_REGISTRY_STATUS, CM_KEY_FAST_INDEX, CM_KEY_NODE, CM_KEY_VALUE, CmpAllocate, CmpFindSubKeyByNumber, CmpFree, CmpRepairClassOfNodeKey, CmpRepairKeyNodeSignature, CmpRepairParentKey, CmpRepairParentNode, CmpRepairSubKeyCounts, CmpRepairSubKeyList, CmpRepairValueList, CmpRepairValueListCount, HBIN, HCELL, HCELL_INDEX, HHIVE, PCELL_DATA, PCM_KEY_FAST_INDEX, PCM_KEY_INDEX, PCM_KEY_NODE, PCMHIVE, PHBIN, PHCELL, PHHIVE, RtlCompareUnicodeString, Stable, Volatile, _HHIVE, _UNICODE_STRING};
use crate::glue::{HvGetCell, HvReleaseCell, HCELL_NIL, HBLOCK_SIZE, TAG_CM, CM_KEY_INDEX_ROOT, CM_KEY_INDEX_LEAF, CM_KEY_FAST_LEAF, CM_KEY_HASH_LEAF, CM_KEY_NODE_SIGNATURE, HBOOT_TYPE_SELF_HEAL, STATUS_REGISTRY_CORRUPT, INVALID_INDEX, HV_HHIVE_SIGNATURE};
use core::ffi::c_void;

extern "C" {
    fn abs(x: i32) -> i32;
}
extern "C" {
    fn CmpCompareCompressedName( String: *const _UNICODE_STRING, Name: *const u16, NameLength: u16, ) -> i32;
    fn PAGED_CODE();
    fn DPRINT1(format: *const i8, ...);
    fn HvMarkCellDirty(Hive: *mut HHIVE, Cell: u32, Unk: i32);
    fn HvIsCellAllocated(Hive: *mut HHIVE, Cell: u32) -> i32;
    fn HvGetCellSize(Hive: *mut _HHIVE, Cell: *mut c_void) -> u32;
    fn ASSERT(condition: BOOLEAN);
    fn HvGetCellType(cell: HCELL_INDEX) -> u32;
    fn GET_CELL_BIN(Bin: PHBIN) -> PHCELL;
    fn IsFreeCell(Cell: PHCELL) -> i32;
    fn CM_CHECK_REGISTRY_SUCCESS(status: CM_CHECK_REGISTRY_STATUS) -> i32;
    unsafe fn CmpValidateValueListByCount( Hive: *mut HHIVE, CurrentCell: HCELL_INDEX, ListCount: u32, ValueListData: *mut CELL_DATA, ValuesRemoved: *mut u32, FixHive: i32, ) -> CM_CHECK_REGISTRY_STATUS;
    fn UNREFERENCED_PARAMETER(param: u8);
    fn FIELD_OFFSET(ty: CM_KEY_NODE, field: *const i8) -> u32;
    unsafe fn CmpPurgeVolatiles(Hive: *mut HHIVE, CurrentCell: HCELL_INDEX, CellData: *mut CELL_DATA, Flags: u32) ;
    unsafe fn CmpValidateClass(Hive: *mut HHIVE, CurrentCell: HCELL_INDEX, CellData: *mut CELL_DATA) -> CM_CHECK_REGISTRY_STATUS;
    unsafe fn CmpValidateSubKeyList(Hive: *mut HHIVE, CurrentCell: HCELL_INDEX, CellData: *mut CELL_DATA, FixHive: u8, DoRepair: *mut u8) -> CM_CHECK_REGISTRY_STATUS;
    unsafe fn CmpValidateValueList(Hive: *mut HHIVE, CurrentCell: HCELL_INDEX, CellData: *mut CELL_DATA, FixHive: u8) -> CM_CHECK_REGISTRY_STATUS;
    fn DPRINT(format: *const u8, ...);
    #[no_mangle] pub unsafe extern "system" fn HvValidateHive(Hive: *mut HHIVE) -> i32;
    unsafe fn CmpValidateRegistryInternal(Hive: *mut HHIVE, Flags: u32, SecurityDefaulted: i32, FixHive: i32) -> i32;
    static mut CmiVolatileHive: *mut CMHIVE;
}

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

const CM_KEY_VALUE_SIGNATURE: u32 = 0x6B765279;
const CM_CHECK_REGISTRY_GOOD: CM_CHECK_REGISTRY_STATUS = 0;
const CM_CHECK_REGISTRY_VALUE_CELL_NIL: CM_CHECK_REGISTRY_STATUS = 1;
const CM_CHECK_REGISTRY_VALUE_CELL_UNALLOCATED: CM_CHECK_REGISTRY_STATUS = 2;
const CM_CHECK_REGISTRY_VALUE_CELL_DATA_NOT_FOUND: CM_CHECK_REGISTRY_STATUS = 3;
const CM_CHECK_REGISTRY_VALUE_CELL_SIZE_NOT_SANE: CM_CHECK_REGISTRY_STATUS = 4;
const CM_CHECK_REGISTRY_CORRUPT_VALUE_DATA: CM_CHECK_REGISTRY_STATUS = 5;
const CM_CHECK_REGISTRY_DATA_CELL_NOT_ALLOCATED: CM_CHECK_REGISTRY_STATUS = 6;
const CM_CHECK_REGISTRY_BAD_KEY_VALUE_SIGNATURE: CM_CHECK_REGISTRY_STATUS = 7;

const CM_CHECK_REGISTRY_VALUE_LIST_UNALLOCATED: CM_CHECK_REGISTRY_STATUS = 0;
const CM_CHECK_REGISTRY_VALUE_LIST_DATA_NOT_FOUND: CM_CHECK_REGISTRY_STATUS = 0;
const CM_CHECK_REGISTRY_VALUE_LIST_SIZE_NOT_SANE: CM_CHECK_REGISTRY_STATUS = 0;

const CMP_KEY_SIZE_THRESHOLD: u32 = 0x45C;

// FIXME: The constants below require correct values from cmcheck.h
const CM_CHECK_REGISTRY_KEY_CELL_NOT_ALLOCATED: CM_CHECK_REGISTRY_STATUS = 1;
const CM_CHECK_REGISTRY_CELL_DATA_NOT_FOUND: CM_CHECK_REGISTRY_STATUS = 2;
const CM_CHECK_REGISTRY_CELL_SIZE_NOT_SANE: CM_CHECK_REGISTRY_STATUS = 3;
const CM_CHECK_REGISTRY_KEY_NAME_LENGTH_ZERO: CM_CHECK_REGISTRY_STATUS = 4;
const CM_CHECK_REGISTRY_KEY_TOO_BIG_THAN_CELL: CM_CHECK_REGISTRY_STATUS = 5;
const CM_CHECK_REGISTRY_BAD_KEY_NODE_PARENT: CM_CHECK_REGISTRY_STATUS = 6;
const CM_CHECK_REGISTRY_BAD_KEY_NODE_SIGNATURE: CM_CHECK_REGISTRY_STATUS = 7;

const CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES: u32 = 0x00000001;
const CM_CHECK_REGISTRY_PURGE_VOLATILES: u32 = 0x00000002;
const CM_CHECK_REGISTRY_BOOTLOADER_PURGE_VOLATILES: u32 = 0x00000004;
const CM_CHECK_REGISTRY_VALIDATE_HIVE: u32 = 0x00000008;
const CM_CHECK_REGISTRY_FIX_HIVE: u32 = 0x00000010;
const CM_CHECK_REGISTRY_INVALID_PARAMETER: i32 = 0xC0000003;

unsafe fn CmpValidateLexicalOrder(
    Hive: *mut _HHIVE,
    Child: HCELL_INDEX,
    Sibling: HCELL_INDEX,
) -> i32 {
    const KEY_COMP_NAME: u32 = 0x0020;

    let mut Result: i32;
    let mut ChildString: _UNICODE_STRING;
    let mut SiblingString: _UNICODE_STRING;
    let mut ChildNode: *mut CM_KEY_NODE;
    let mut SiblingNode: *mut CM_KEY_NODE;

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
        if ((*ChildNode).Flags & KEY_COMP_NAME) == 0
            && ((*SiblingNode).Flags & KEY_COMP_NAME) == 0
        {
            /* Build the sibling string */
            SiblingString.Buffer = (*SiblingNode).Name.as_mut_ptr();
            SiblingString.Length = (*SiblingNode).NameLength;
            SiblingString.MaximumLength = (*SiblingNode).NameLength;

            /* Build the child string */
            ChildString.Buffer = (*ChildNode).Name.as_mut_ptr();
            ChildString.Length = (*ChildNode).NameLength;
            ChildString.MaximumLength = (*ChildNode).NameLength;

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
        if ((*ChildNode).Flags & KEY_COMP_NAME) != 0
            && ((*SiblingNode).Flags & KEY_COMP_NAME) != 0
        {
            /* FIXME: Checks for two compressed names not implemented yet */
            // DPRINT!("Lexicographical order checks for two compressed names is UNIMPLEMENTED, assume the key is healthy...\n");
            return 1;
        }

        /* CASE 3: The child name is compressed but the sibling is not */
        if ((*ChildNode).Flags & KEY_COMP_NAME) != 0
            && ((*SiblingNode).Flags & KEY_COMP_NAME) == 0
        {
            SiblingString.Buffer = (*SiblingNode).Name.as_mut_ptr();
            SiblingString.Length = (*SiblingNode).NameLength;
            SiblingString.MaximumLength = (*SiblingNode).NameLength;
            Result = CmpCompareCompressedName(
                core::ptr::addr_of!(SiblingString),
                (*ChildNode).Name.as_ptr(),
                (*ChildNode).NameLength,
            );
            if Result >= 0 {
                // DPRINT1!("The sibling node name is greater or equal to that of the compressed child\n");
                return 0;
            }
        }

        /* CASE 4: The sibling name is compressed but the child is not */
        if ((*SiblingNode).Flags & KEY_COMP_NAME) != 0
            && ((*ChildNode).Flags & KEY_COMP_NAME) == 0
        {
            ChildString.Buffer = (*ChildNode).Name.as_mut_ptr();
            ChildString.Length = (*ChildNode).NameLength;
            ChildString.MaximumLength = (*ChildNode).NameLength;
            Result = CmpCompareCompressedName(
                core::ptr::addr_of!(ChildString),
                (*SiblingNode).Name.as_ptr(),
                (*SiblingNode).NameLength,
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

// Forward declarations for dependencies (caller dedups)
const CM_CHECK_REGISTRY_KEY_CLASS_UNALLOCATED: i32 = 1;

unsafe fn CmpValidateKey(
    Hive: *mut HHIVE,
    SecurityDefaulted: u8,
    ParentCell: HCELL_INDEX,
    CurrentCell: HCELL_INDEX,
    Flags: u32,
    FixHive: u8,
) -> CM_CHECK_REGISTRY_STATUS {
    let mut cm_status_code: CM_CHECK_REGISTRY_STATUS = 0;
    let mut cell_data: *mut CELL_DATA;
    let mut cell_size: u32;
    let mut do_subkeys_repair: u8 = 0u8;
    let mut total_key_name_length: u32;
    let mut name_length: u32;

    PAGED_CODE();

    assert!(CurrentCell != HCELL_NIL, "ASSERT: CurrentCell != HCELL_NIL");

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

    cell_size = HvGetCellSize(Hive, cell_data);
    if cell_size > CMP_KEY_SIZE_THRESHOLD {
        // DPRINT removed
        return CM_CHECK_REGISTRY_CELL_SIZE_NOT_SANE;
    }

    name_length = (*cell_data).u.KeyNode.NameLength;
    if name_length == 0 {
        // DPRINT removed
        return CM_CHECK_REGISTRY_KEY_NAME_LENGTH_ZERO;
    }

    total_key_name_length = name_length + FIELD_OFFSET(CM_KEY_NODE, b"Name\0".as_ptr() as *const i8);
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
    PAGED_CODE();
    if !(!Bin.is_null()) {
        panic!("ASSERT(Bin)");
    }
    if !(!Hive.is_null()) {
        panic!("ASSERT(Hive)");
    }
    if (*Bin).Signature != 0x6E696248 {
        // DPRINT removed
        return 1; // CM_CHECK_REGISTRY_BIN_SIGNATURE_HEADER_CORRUPT
    }
    let Basket = (Bin as *mut u8).add((*Bin).Size) as *mut HCELL;
    let mut Cell = GET_CELL_BIN(Bin);
    while Cell < Basket {
        if IsFreeCell(Cell) != 0 {
            if (*Cell).Size > (*Bin).Size || (*Cell).Size == 0 {
                // DPRINT removed
                return 2; // CM_CHECK_REGISTRY_BAD_FREE_CELL
            }
        } else {
            if abs((*Cell).Size) > (*Bin).Size {
                // DPRINT removed
                return 3; // CM_CHECK_REGISTRY_BAD_ALLOC_CELL
            }
        }
        Cell = (Cell as *mut u8).add(abs((*Cell).Size) as usize) as *mut HCELL;
    }
    return 0; // CM_CHECK_REGISTRY_GOOD
}

#[no_mangle]
pub unsafe extern "system" fn CmCheckRegistry(
    RegistryHive: *mut CMHIVE,
    Flags: u32,
) -> i32 {
    let mut CmStatusCode: i32;
    let Hive: *mut HHIVE;
    let mut ShouldFixHive: i32 = 0;

    PAGED_CODE();

    if RegistryHive.is_null() {
        // DPRINT removed
        return CM_CHECK_REGISTRY_INVALID_PARAMETER;
    }

    if RegistryHive == CmiVolatileHive {
        // DPRINT removed
        return CM_CHECK_REGISTRY_GOOD;
    }

    if Flags & !(CM_CHECK_REGISTRY_DONT_PURGE_VOLATILES |
                 CM_CHECK_REGISTRY_PURGE_VOLATILES |
                 CM_CHECK_REGISTRY_BOOTLOADER_PURGE_VOLATILES |
                 CM_CHECK_REGISTRY_VALIDATE_HIVE |
                 CM_CHECK_REGISTRY_FIX_HIVE) != 0 {
        // DPRINT removed
        return CM_CHECK_REGISTRY_INVALID_PARAMETER;
    }

    Hive = core::ptr::addr_of_mut!((*RegistryHive).Hive);

    if Flags & CM_CHECK_REGISTRY_VALIDATE_HIVE != 0 {
        CmStatusCode = HvValidateHive(Hive);
        if CM_CHECK_REGISTRY_SUCCESS(CmStatusCode) == 0 {
            // DPRINT removed
            return CmStatusCode;
        }
    }

    if Flags & CM_CHECK_REGISTRY_FIX_HIVE != 0 {
        ShouldFixHive = 1;
    }

    CmStatusCode = CmpValidateRegistryInternal(Hive, Flags, 0, ShouldFixHive);
    if CM_CHECK_REGISTRY_SUCCESS(CmStatusCode) == 0 {
        // DPRINT removed
        return CmStatusCode;
    }

    return CmStatusCode;
}
