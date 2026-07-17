use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CmpCopyName, CmpNameSize, HCELL_INDEX, PCM_KEY_NODE, _CM_KEY_NODE, PHHIVE, RtlInitUnicodeString, HSTORAGE_TYPE_Stable as Stable, HSTORAGE_TYPE_Volatile as Volatile, _UNICODE_STRING};
// Hv* with their correct per-fn ABI (cdecl HvGetCell/HvReleaseCell; stdcall
// HvAllocateCell/HvFreeCell); the header checksum is our own hivesum.rs.
use crate::glue::{HvAllocateCell, HvFreeCell, HvGetCell, HvReleaseCell};
use crate::hivesum::HvpHiveHeaderChecksum;

// ---- file-scope preamble (translated) ----
// Exported (other cmlib C files reference _CmlibTraceLevel for debug tracing).
#[unsafe(no_mangle)]
pub static mut CmlibTraceLevel: ULONG = 0;

// CMAPI = NTAPI → stdcall on i386 (_CmCreateRootNode@8): extern "system".
#[no_mangle]
pub unsafe extern "system" fn CmCreateRootNode(
    Hive: PHHIVE,
    Name: PCWSTR,
) -> u8 {
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let KeyCell: PCM_KEY_NODE;
    let RootCellIndex: HCELL_INDEX;

    const CM_KEY_NODE_SIGNATURE: u16 = 0x6B6E;
    const KEY_HIVE_ENTRY: u16 = 0x0004;
    const KEY_NO_DELETE: u16 = 0x0008;
    const KEY_COMP_NAME: u16 = 0x0020;
    const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;

    RtlInitUnicodeString(core::ptr::addr_of_mut!(KeyName), Name);
    RootCellIndex = HvAllocateCell(
        Hive,
        (core::mem::offset_of!(_CM_KEY_NODE, Name) as u32)
            + CmpNameSize(Hive, core::ptr::addr_of_mut!(KeyName)) as u32,
        Stable,
        HCELL_NIL,
    );
    if RootCellIndex == HCELL_NIL {
        return 0u8;
    }

    (*(*Hive).BaseBlock).RootCell = RootCellIndex;
    (*(*Hive).BaseBlock).CheckSum = HvpHiveHeaderChecksum((*Hive).BaseBlock as *const u32);

    KeyCell = HvGetCell(Hive, RootCellIndex) as PCM_KEY_NODE;
    if KeyCell.is_null() {
        HvFreeCell(Hive, RootCellIndex);
        return 0u8;
    }

    (*KeyCell).Signature = CM_KEY_NODE_SIGNATURE;
    (*KeyCell).Flags = KEY_HIVE_ENTRY | KEY_NO_DELETE;
    (*KeyCell).LastWriteTime.QuadPart = 0i64;
    (*KeyCell).Parent = HCELL_NIL;
    (*KeyCell).SubKeyCounts[Stable as usize] = 0;
    (*KeyCell).SubKeyCounts[Volatile as usize] = 0;
    (*KeyCell).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Stable as usize] = HCELL_NIL;
    (*KeyCell).__bindgen_anon_1.__bindgen_anon_1.SubKeyLists[Volatile as usize] = HCELL_NIL;
    (*KeyCell).__bindgen_anon_1.__bindgen_anon_1.ValueList.Count = 0;
    (*KeyCell).__bindgen_anon_1.__bindgen_anon_1.ValueList.List = HCELL_NIL;
    (*KeyCell).Security = HCELL_NIL;
    (*KeyCell).Class = HCELL_NIL;
    (*KeyCell).ClassLength = 0;
    (*KeyCell).MaxNameLen = 0;
    (*KeyCell).MaxClassLen = 0;
    (*KeyCell).MaxValueNameLen = 0;
    (*KeyCell).MaxValueDataLen = 0;
    (*KeyCell).NameLength = CmpCopyName(Hive, core::ptr::addr_of_mut!((*KeyCell).Name) as *mut u16, core::ptr::addr_of_mut!(KeyName));
    if (*KeyCell).NameLength < KeyName.Length {
        (*KeyCell).Flags |= KEY_COMP_NAME;
    }

    HvReleaseCell(Hive, RootCellIndex);
    1u8
}


// ============================================================================
