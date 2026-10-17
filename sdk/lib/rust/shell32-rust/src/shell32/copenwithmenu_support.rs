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
// One declaration per external symbol: island.rs.
pub use crate::island::DialogBoxParamW;

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

// ── substrate the TU calls that the island's closure does not carry ───────────────────────
//
// Every name here already has ONE definition elsewhere in the crate; this module re-exports
// rather than re-declares, so there is no second spelling to drift. (Several of these are
// per-module re-implementations across the crate — `StringCbCopyW` and `HResultFromWin32`
// each exist in three files — which is a consolidation worth doing separately, not here.)
pub use crate::win32::{PathFindFileNameW, SHELL_ErrorBox};
pub use crate::default_context_menu_support::CmicFlagsToSeeFlags;
pub use crate::types::HRESULT_FROM_WIN32;

/// shlwapi's `StringCbCopyW` — BYTE count, not character count. Named `Cb` in the C for
/// exactly that reason; passing `_countof` here truncates every string to half its length.
#[inline]
pub unsafe fn StringCbCopyW(dst: LPWSTR, bytes: usize, src: LPCWSTR) -> HRESULT {
    crate::cfs_folder::StringCchCopyW(dst, bytes / core::mem::size_of::<WCHAR>(), src)
}

// ── DialogBoxParamW, declared against the island ──────────────────────────────────────────
//
// The island's own DialogBoxParamW takes ntdll-sys's `DLGPROC`, which bindgen spells
// `extern "stdcall"` on i386 and `extern "C"` on 64-bit: one ABI, three Rust types, and a
// dialog procedure can only be written to match one of them. The usual escape is a
// `#[cfg]`-duplicated copy of the whole procedure per arch, which is how brfolder.rs and
// recycler_prop.rs ended up with two copies that then drifted apart under separate repair
// passes.
//
// `crate::prelude::DLGPROC` is the fix already in the tree: `extern "system"`, which IS
// stdcall on i386-windows and IS the C convention on 64-bit, so one spelling is correct
// everywhere. Re-declaring the import against it — and against island handle types, since
// these TUs are island-first — lets the thunks be written once.
//
// Not taken from `crate::win32`: that module types against `crate::types`, where `HWND` is
// `*mut c_void` rather than the island's `*mut HWND__`, which just moves the mismatch from
// the callback slot to every handle argument.
extern "system" {
}

// ── notification codes the island does not carry ──────────────────────────────────────────
//
// bindgen keeps `TVN_SELCHANGEDW` and `NM_FIRST` but drops `TVN_SELCHANGED` (a macro alias
// for the W form) and the `NM_*` codes (macros defined by ARITHMETIC on NM_FIRST, which
// bindgen does not evaluate). All three appear as `match` arms on `NMHDR::code` in
// COpenWithDialog_DialogProc, and an unresolved name in a match arm is not an error — it is
// an irrefutable BINDING that swallows every case (field manual [§9.112]). This one shipped
// three of them in a single match, so every WM_NOTIFY took the first arm and the double-click
// and Enter-key paths were unreachable. rustc only objected because two of them shared an
// or-pattern, which cannot bind; a lone bad arm compiles silently.
//
// Derived from the island rather than transcribed: the arithmetic is the macro's own, so a
// header change moves these with it.
pub const TVN_SELCHANGED: UINT = crate::island::TVN_SELCHANGEDW as UINT;
pub const NM_DBLCLK: UINT = crate::island::NM_FIRST.wrapping_sub(3);
pub const NM_RETURN: UINT = crate::island::NM_FIRST.wrapping_sub(4);

/// `RegSetString` — dll/win32/shell32/utils.h:81, a `static inline` in a HEADER.
///
/// Header helpers have no TU of their own, so no translation pass ever produces one: it is
/// neither in the module's source list nor an import the island can resolve. Ported here
/// verbatim rather than open-coded at each call site, so the byte-count arithmetic — the
/// part that is easy to get wrong — exists once.
///
/// `(lstrlenW + 1) * sizeof(WCHAR)`: bytes INCLUDING the NUL. Dropping the `+ 1` stores an
/// unterminated value that later reads walk off the end of.
#[inline]
pub unsafe fn RegSetString(hKey: HKEY, Name: LPCWSTR, Str: LPCWSTR, Type: DWORD) -> LSTATUS {
    RegSetValueExW(
        hKey,
        Name,
        0,
        Type,
        Str as *const BYTE,
        ((lstrlenW(Str) + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD,
    )
}

/// `SHRestricted(REST_NOFILEASSOCIATE)` — COpenWithMenu.cpp:1207.
///
/// bindgen renders a C enum constant PREFIXED with its enum's tag, so `RESTRICTIONS`'
/// enumerators arrive as `RESTRICTIONS_REST_*` and none of them answer to the name the C
/// uses. Aliased, never re-spelled: the value stays the island's.
pub const REST_NOFILEASSOCIATE: RESTRICTIONS = crate::island::RESTRICTIONS_REST_NOFILEASSOCIATE;

/// `HRESULT_FROM_WIN32` applied to a `DWORD` error, the spelling the C++ uses.
#[inline]
pub fn HResultFromWin32(error: DWORD) -> HRESULT {
    HRESULT_FROM_WIN32(error as LONG)
}
