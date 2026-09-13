//! Translation of `sdk/lib/cmlib/cmse.c` — CM security-cell list management.
//! (rustconv Sonnet, hand-integrated: ABI fixes + extern decls + HCELL_NIL.)

use ntkrnl_sys::ddk::{HCELL_INDEX, PCM_KEY_NODE, PCM_KEY_SECURITY, PHHIVE};

const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;

// cmlib callees with their REAL per-function ABI (llvm-nm on libcmlib.a):
//   HvGetCell / HvReleaseCell are cdecl (_HvGetCell, no @N) — extern "C".
//   HvFreeCell is stdcall (_HvFreeCell@8) — extern "system".
extern "C" {
    fn HvGetCell(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
    fn HvReleaseCell(Hive: PHHIVE, Cell: HCELL_INDEX);
}
extern "system" {
    fn HvFreeCell(Hive: PHHIVE, Cell: HCELL_INDEX);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn CmpRemoveSecurityCellList(
    Hive: PHHIVE,
    SecurityCell: HCELL_INDEX,
) {
    let SecurityData = HvGetCell(Hive, SecurityCell) as PCM_KEY_SECURITY;
    if SecurityData.is_null() {
        return;
    }

    let FlinkCell = HvGetCell(Hive, (*SecurityData).Flink) as PCM_KEY_SECURITY;
    if FlinkCell.is_null() {
        HvReleaseCell(Hive, SecurityCell);
        return;
    }

    let BlinkCell = HvGetCell(Hive, (*SecurityData).Blink) as PCM_KEY_SECURITY;
    if BlinkCell.is_null() {
        HvReleaseCell(Hive, (*SecurityData).Flink);
        HvReleaseCell(Hive, SecurityCell);
        return;
    }

    /* Unlink the security block and free it */
    (*FlinkCell).Blink = (*SecurityData).Blink;
    (*BlinkCell).Flink = (*SecurityData).Flink;

    /* Release the cells */
    HvReleaseCell(Hive, (*SecurityData).Blink);
    HvReleaseCell(Hive, (*SecurityData).Flink);
    HvReleaseCell(Hive, SecurityCell);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn CmpFreeSecurityDescriptor(Hive: PHHIVE, Cell: HCELL_INDEX) {
    let CellData = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
    if CellData.is_null() {
        return;
    }

    // FIXME (ReactOS-specific, from cmse.c): keys without a security block.
    if (*CellData).Security == HCELL_NIL {
        HvReleaseCell(Hive, Cell);
        return;
    }

    let SecurityCell = (*CellData).Security;
    let SecurityData = HvGetCell(Hive, SecurityCell) as PCM_KEY_SECURITY;
    if SecurityData.is_null() {
        HvReleaseCell(Hive, Cell);
        return;
    }

    let FreeSecurityCell = (*SecurityData).ReferenceCount <= 1;
    if !FreeSecurityCell {
        (*SecurityData).ReferenceCount -= 1;
    } else {
        CmpRemoveSecurityCellList(Hive, SecurityCell);
    }

    (*CellData).Security = HCELL_NIL;
    HvReleaseCell(Hive, SecurityCell);
    if FreeSecurityCell {
        HvFreeCell(Hive, SecurityCell);
    }
    HvReleaseCell(Hive, Cell);
}
