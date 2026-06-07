//! Shared cmlib constants + correct-ABI extern decls for the cmlib hive
//! primitives the translated modules call. Centralized so the per-function ABI
//! (the #1 cmlib hazard) is declared once and right.
//!
//! ABI (verified via llvm-nm on libcmlib.a):
//!   - HvGetCell / HvReleaseCell are **cdecl** (`_HvGetCell`, no `@N`).
//!   - HvAllocateCell@16 / HvFreeCell@8 / HvMarkCellDirty@12 are **stdcall**.
//! HvpHiveHeaderChecksum lives in our hivesum.rs (call it directly, not here).

use ntkrnl_sys::ddk::{HCELL_INDEX, PHHIVE};

// cmlib.h constants.
pub const HCELL_NIL: HCELL_INDEX = 0xFFFF_FFFF; // MAXULONG
pub const HBLOCK_SIZE: u32 = 0x1000;
pub const HV_HBIN_SIGNATURE: u32 = 0x6e69_6268; // "hbin"
pub const TAG_CM: u32 = 0x2020_4D43; // '  MC' (reads "CM  " in a pool dump)
pub const CM_KEY_NODE_SIGNATURE: u16 = 0x6B6E; // "nk"
pub const CM_KEY_SECURITY_SIGNATURE: u16 = 0x6B73; // "sk"
pub const KEY_HIVE_EXIT: u16 = 0x0002;
pub const KEY_HIVE_ENTRY: u16 = 0x0004;
pub const KEY_NO_DELETE: u16 = 0x0008;
pub const KEY_COMP_NAME: u16 = 0x0020;

/// `ROUND_UP(a,b) = ((((a)+(b)-1)/(b))*(b))` from cmlib.h.
#[inline]
pub fn round_up(a: u32, b: u32) -> u32 {
    ((a + b - 1) / b) * b
}

extern "C" {
    // cdecl (no @N decoration on i386)
    pub fn HvGetCell(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
    pub fn HvReleaseCell(Hive: PHHIVE, Cell: HCELL_INDEX);
}

extern "system" {
    // stdcall (_Name@N on i386), cdecl on amd64/arm64
    pub fn HvAllocateCell(
        Hive: PHHIVE,
        Size: u32,
        Storage: u32,
        Vicinity: HCELL_INDEX,
    ) -> HCELL_INDEX;
    pub fn HvFreeCell(Hive: PHHIVE, Cell: HCELL_INDEX);
    pub fn HvMarkCellDirty(Hive: PHHIVE, Cell: HCELL_INDEX, HoldBufferLock: u8) -> u8;
}
