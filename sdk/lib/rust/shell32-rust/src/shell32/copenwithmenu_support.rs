//! Hand-written substrate for the COpenWithMenu.cpp flip.
//!
//! Lives in its own file because a re-run of `rustconv translate` overwrites each generated
//! class half wholesale, so anything hand-written parked inside them is lost next time.
//!
//! What is here and why the generator could not supply it:
//!
//! * `SApp` — a POD nested INSIDE `class COpenWithList`. `_discover_pod_state_types` only
//!   captures a nested type reached through a by-VALUE member; `COpenWithList` holds
//!   `SApp *m_pApp`, a pointer, so the type is referenced everywhere and defined nowhere.

use crate::prelude::*;

/// `COpenWithList::SApp` — COpenWithMenu.cpp:154.
///
/// Layout is load-bearing: `m_pApp` is a heap ARRAY walked by pointer arithmetic
/// (`COpenWithList_GetAt_RustBridge` strides by `size_of::<SApp>()`), so a field width or
/// order change here silently reads the wrong app entry rather than failing to compile.
/// The commented-out `wszManufacturer` is commented out in the C too — keep it that way.
#[repr(C)]
pub struct SApp {
    pub wszFilename: [WCHAR; MAX_PATH as usize],
    pub wszCmd: [WCHAR; MAX_PATH as usize],
    // WCHAR wszManufacturer[256];  // disabled in the C++ source, not dropped in translation
    pub wszName: [WCHAR; 256],
    pub bHidden: BOOL,
    pub bRecommended: BOOL,
    pub bMRUList: BOOL,
    pub hIcon: HICON,
}

// ── registry status constants, typed as the Reg* functions actually return ────────────────
//
// The island types every `ERROR_*` as `u32` (WinError.h spells them `0L`, but they arrive
// unsigned), while `RegOpenKeyExW` and friends return `LSTATUS` = `LONG` = `i32`. Every
// `RegX(...) == ERROR_SUCCESS` comparison is therefore an i32/u32 mismatch — 36 of them in
// this TU alone. The other ERROR_* here are SEALED into the generated files with their
// values; those are retyped in place so sealing stays the single source of their value.
//
// Re-typing the constant once beats casting at each site: an explicit `use` of these shadows
// the prelude's glob-imported versions inside the importing module, so the comparisons read
// exactly as the C does. Values are pinned to the island's, never re-spelled — re-typing a
// constant must not become an opportunity to re-enter its value.
pub const ERROR_SUCCESS: LSTATUS = crate::prelude::ERROR_SUCCESS as LSTATUS;
