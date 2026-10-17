//! Shared cmlib constants + correct-ABI extern decls for the cmlib hive
//! primitives the translated modules call. Centralized so the per-function ABI
//! (the #1 cmlib hazard) is declared once and right.
//!
//! ABI (verified via llvm-nm on libcmlib.a):
//!   - HvGetCell / HvReleaseCell are **cdecl** (`_HvGetCell`, no `@N`).
//!   - HvAllocateCell@16 / HvFreeCell@8 / HvMarkCellDirty@12 are **stdcall**.
//! HvpHiveHeaderChecksum lives in our hivesum.rs (call it directly, not here).

use ntkrnl_sys::ddk::{HCELL_INDEX, PHHIVE, HBIN, HCELL, CM_KEY_INDEX};
// The crate's own functions: the definition is the declaration.
pub use crate::hivecell::HvGetCellSize;

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

// ── Phase-2 authoritative constants (harvested from sdk/lib/cmlib/hivedata.h,
//    cmlib.h, ndk/ntstatus) — canonical home so the deepseek-mechanical modules
//    import REAL values instead of fabricating them. Never redefine these
//    locally with a guess (the mechanical pass produced wrong HCELL_NIL=0 etc.).
pub const HV_HHIVE_SIGNATURE: u32 = 0xbee0_bee0;
pub const HCELL_TYPE_MASK: u32 = 0x8000_0000;
pub const INVALID_INDEX: u32 = 0x8000_0000;
// hive file types (hivedata.h)
pub const HFILE_TYPE_PRIMARY: u32 = 0;
pub const HFILE_TYPE_LOG: u32 = 1;
pub const HFILE_TYPE_ALTERNATE: u32 = 3;
// hive storage / boot types
pub const HIVE_VOLATILE: u32 = 1;
pub const HSYS_MINOR: u32 = 3;
pub const HTYPE_COUNT: u32 = 2;
pub const HBOOT_TYPE_REGULAR: u32 = 0;
pub const HBOOT_TYPE_SELF_HEAL: u32 = 4;
// CM_KEY_INDEX cell signatures (u16, compared against the Signature field)
pub const CM_KEY_INDEX_ROOT: u16 = 0x6972; // "ri"
pub const CM_KEY_INDEX_LEAF: u16 = 0x696C; // "li"
pub const CM_KEY_FAST_LEAF: u16 = 0x666C; // "lf"
pub const CM_KEY_HASH_LEAF: u16 = 0x686C; // "lh"
// versioning (sdkddkver.h): NTDDI_VISTA = NTDDI_WIN6
pub const NTDDI_VISTA: u32 = 0x0600_0000;
// registry NTSTATUS codes (ndk/ntstatus.h)
pub const STATUS_REGISTRY_CORRUPT: i32 = 0xC000_014Cu32 as i32;
pub const STATUS_REGISTRY_IO_FAILED: i32 = 0xC000_014Du32 as i32;
pub const STATUS_NOT_REGISTRY_FILE: i32 = 0xC000_015Cu32 as i32;
pub const STATUS_REGISTRY_RECOVERED: i32 = 0x4000_0009;

// cmindex.c:23 macro — max CM_KEY_INDEX entries that fit in one hblock. Kept as
// a layout-DERIVED const expression (not a guessed literal): a wrong value
// breaks leaf-split capacity logic → registry corruption on large keys.
// C: ((HBLOCK_SIZE - (sizeof(HBIN)+sizeof(HCELL)+FIELD_OFFSET(CM_KEY_INDEX,List)))
//      / sizeof(HCELL_INDEX) - 1)
pub const CmpMaxIndexPerHblock: u32 = ((HBLOCK_SIZE as usize
    - (core::mem::size_of::<HBIN>()
        + core::mem::size_of::<HCELL>()
        + core::mem::offset_of!(CM_KEY_INDEX, List)))
    / core::mem::size_of::<HCELL_INDEX>()
    - 1) as u32;

/// `ROUND_UP(a,b) = ((((a)+(b)-1)/(b))*(b))` from cmlib.h.
#[inline]
pub fn round_up(a: u32, b: u32) -> u32 {
    ((a + b - 1) / b) * b
}

// ── cmlib.h MACROS, as crate-local inlines ────────────────────────────────
// `HvGetCell`/`HvReleaseCell` are the cmlib.h macros
// `(Hive)->GetCellRoutine(Hive, Cell)` / `(Hive)->ReleaseCellRoutine(...)` —
// NOT real exported symbols. In the kernel ntoskrnl/config/inlines.rs exports a
// real one; the loader's blcmlib has none (the C loader inlined the macro). We
// provide crate-local inlines (NOT #[no_mangle]) so every cmlib-rust call site
// inlines the function-pointer dispatch and needs NO external symbol on either
// the kernel OR the loader link — identical to cmboot-rust's local copy.
// Modules MUST `use crate::glue::{HvGetCell, HvReleaseCell, ...}` and must NOT
// re-declare these as `extern` (that would reference a non-existent symbol).
#[inline]
pub(crate) unsafe fn HvGetCell(hive: PHHIVE, cell: HCELL_INDEX) -> *mut core::ffi::c_void {
    if let Some(get) = (*hive).GetCellRoutine {
        get(hive, cell) as *mut core::ffi::c_void
    } else {
        core::ptr::null_mut()
    }
}

#[inline]
pub(crate) unsafe fn HvReleaseCell(hive: PHHIVE, cell: HCELL_INDEX) {
    if let Some(release) = (*hive).ReleaseCellRoutine {
        release(hive, cell);
    }
}

// HvGetCellType / HvGetCellBlock — hivedata.h bitfield macros over HCELL_INDEX.
#[inline]
pub(crate) fn HvGetCellType(cell: HCELL_INDEX) -> u32 {
    (cell & 0x8000_0000) >> 31
}
#[inline]
pub(crate) fn HvGetCellBlock(cell: HCELL_INDEX) -> u32 {
    (cell & 0x7fff_f000) >> 12
}

// IsFreeCell(Cell) — hivedata.h:360 `((Cell)->Size >= 0)`.
#[inline]
pub(crate) unsafe fn IsFreeCell(cell: *const HCELL) -> bool {
    (*cell).Size >= 0
}

// CmpIsKeyValueSmall — cmlib.h:393 static inline. Returns TRUE if the value is
// "small" (data inlined into the cell) and writes the real length.
#[inline]
pub(crate) unsafe fn CmpIsKeyValueSmall(real_length: *mut u32, length: u32) -> bool {
    const CM_KEY_VALUE_SPECIAL_SIZE: u32 = 0x8000_0000;
    if length >= CM_KEY_VALUE_SPECIAL_SIZE {
        *real_length = length - CM_KEY_VALUE_SPECIAL_SIZE;
        true
    } else {
        *real_length = length;
        false
    }
}

// ASSERT — a debug macro, a no-op in the NDEBUG cmlib build. Provided
// as a crate-local no-op (PAGED_CODE is the macro in lib.rs) so translated bodies
// that call it need no extern sym.

extern "system" {
    // Real cmlib functions defined IN this crate (hivecell.rs). stdcall (_Name@N
    // on i386), cdecl on amd64/arm64. Declared here so other modules call them
    // with the correct ABI; they resolve intra-crate to the hivecell.rs defs.
    pub fn HvAllocateCell(
        Hive: PHHIVE,
        Size: u32,
        Storage: u32,
        Vicinity: HCELL_INDEX,
    ) -> HCELL_INDEX;
    pub fn HvFreeCell(Hive: PHHIVE, Cell: HCELL_INDEX);
    pub fn HvMarkCellDirty(Hive: PHHIVE, Cell: HCELL_INDEX, HoldBufferLock: u8) -> u8;
    pub fn HvpGetCellData(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
}
