//! Shell/System Policies — translation of dll/win32/shell32/wine/shpolicy.c.
//!
//! Provides the policy/restriction query API (SHRestricted/SHSettingsChanged) plus the
//! cached global-counter machinery and the StrCmpLogical indirection. The static policy
//! table (s_PolicyTable) is a faithful reproduction of `PolicyData.h` as compiled for the
//! ReactOS shell32 build configuration (dll/win32/shell32/CMakeLists.txt forces
//! `-D_WIN32_WINNT=0x600` and `-D_SHELL32_`): WINE_FILEVERSION_MAJOR == 6 and
//! NTDDI_VERSION == NTDDI_WIN6 (0x06000000); since NTDDI_LONGHORN == NTDDI_VISTA == NTDDI_WIN6,
//! the guard `NTDDI_VERSION < NTDDI_LONGHORN` is FALSE here. Under that config the
//! `#include "PolicyData.h"` macro-expansion yields 159 rows (verified byte-for-byte against
//! the compiled shpolicy.c.obj); the C array adds a `{0,NULL,NULL}` terminator → 160 entries
//! == `_countof(s_PolicyTable)`, which is the length of `g_RestValues`.
//!
//! Exports (i386 decoration): `_SHRestricted@4`, `_SHSettingsChanged@8`,
//! `_SHELL_StrCmpLogicalInit@8` (all WINAPI/extern "system"); `_SHELL_GetCachedGlobalCounter`,
//! `_SH32_InternalRestricted` (EXTERN_C/cdecl). Exported data globals: `SHELL_StrCmpLogical`,
//! `g_hRestGlobalCounter`, `g_nRestCountValue`, `g_RestValues`.

use crate::types::*;
// One declaration per external symbol: island.rs.
use crate::island::CloseHandle;
use core::sync::atomic::{AtomicPtr, Ordering};

// ── file-local types not (yet) in crate::types — see ADDED-TO-TYPES note at EOF ──────────
// PCWSTR (guiddef/winnt.h) — same as LPCWSTR.
pub type PCWSTR = *const WCHAR;

/// GUID (guiddef.h) — { Data1: DWORD, Data2: WORD, Data3: WORD, Data4: [BYTE;8] }.
pub use crate::island::GUID;
/// REFGUID (guiddef.h) — `const GUID *`.
pub type REFGUID = *const GUID;

/// POLICYDATA (reactos/shlwapi_undoc.h):
///   { DWORD policy; LPCWSTR appstr; LPCWSTR keystr; }
#[repr(C)]
#[derive(Copy, Clone)]
pub struct POLICYDATA {
    pub policy: DWORD,
    pub appstr: LPCWSTR,
    pub keystr: LPCWSTR,
}

/// `WINAPI int(*)(PCWSTR, PCWSTR)` — type of the exported `SHELL_StrCmpLogical` fn pointer.
pub type StrCmpLogicalFn = unsafe extern "system" fn(s1: PCWSTR, s2: PCWSTR) -> INT;

/// Sync wrapper so the read-only `s_PolicyTable` (which holds raw `*const WCHAR`) can live in
/// an immutable `static`. The table is never mutated; the pointers are to other statics.
#[repr(transparent)]
struct PolicyTable([POLICYDATA; POLICY_TABLE_LEN]);
unsafe impl Sync for PolicyTable {}

// ── constants pinned from headers ────────────────────────────────────────────────────────
// REST_NOSTRCMPLOGICAL (psdk/shlobj.h: REST_NOSTRCMPLOGICAL = 0x4000007E).
const REST_NOSTRCMPLOGICAL: DWORD = 0x4000007E;
// SHELL_NO_POLICY (reactos/shlwapi_undoc.h: ((DWORD)-1)) — value FillMemory(0xFF) writes.
// (used implicitly: g_RestValues is filled with 0xFF bytes == 0xFFFFFFFF per slot)

// GUID_Restrictions — DEFINE_GUID(GUID_Restrictions,
//   0xA48F1A32,0xA340,0x11D1,0xBC,0x6B,0x00,0xA0,0xC9,0x03,0x12,0xE1) (shpolicy.c).
static GUID_RESTRICTIONS: GUID = GUID {
    Data1: 0xA48F1A32,
    Data2: 0xA340,
    Data3: 0x11D1,
    Data4: [0xBC, 0x6B, 0x00, 0xA0, 0xC9, 0x03, 0x12, 0xE1],
};

// ── imports (resolved via shell32's existing import libs at link) ─────────────────────────
extern "system" {
    // shlwapi (undoc) — global counters + restriction lookup
    fn SHGlobalCounterCreate(guid: REFGUID) -> HANDLE; // SHGlobalCounterCreate@4
    fn SHGlobalCounterGetValue(hGlobalCounter: HANDLE) -> LONG; // SHGlobalCounterGetValue@4
    fn SHGlobalCounterIncrement(hGlobalCounter: HANDLE) -> LONG; // SHGlobalCounterIncrement@4
    fn SHRestrictionLookup(
        policy: DWORD,
        key: LPCWSTR,
        polTable: *const POLICYDATA,
        polArr: LPDWORD,
    ) -> DWORD; // SHRestrictionLookup@16
    // shlwapi — string compares
    fn StrCmpIW(s1: PCWSTR, s2: PCWSTR) -> INT; // StrCmpIW@8
    fn StrCmpLogicalW(s1: PCWSTR, s2: PCWSTR) -> INT; // StrCmpLogicalW@8
    // kernel32
    fn lstrcmpiW(s1: LPCWSTR, s2: LPCWSTR) -> INT; // lstrcmpiW@8
// CloseHandle@4
}

// ---- wide-string statics (UTF-16, NUL-terminated) — dedup of PolicyData.h app/key strings ----
static W_0_Explorer: [WCHAR; 9] = [0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x0000]; // "Explorer"
static W_1_NoRun: [WCHAR; 6] = [0x004E, 0x006F, 0x0052, 0x0075, 0x006E, 0x0000]; // "NoRun"
static W_2_NoClose: [WCHAR; 8] = [0x004E, 0x006F, 0x0043, 0x006C, 0x006F, 0x0073, 0x0065, 0x0000]; // "NoClose"
static W_3_NoSaveSettings: [WCHAR; 15] = [0x004E, 0x006F, 0x0053, 0x0061, 0x0076, 0x0065, 0x0053, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x0000]; // "NoSaveSettings"
static W_4_NoFileMenu: [WCHAR; 11] = [0x004E, 0x006F, 0x0046, 0x0069, 0x006C, 0x0065, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "NoFileMenu"
static W_5_NoSetFolders: [WCHAR; 13] = [0x004E, 0x006F, 0x0053, 0x0065, 0x0074, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0073, 0x0000]; // "NoSetFolders"
static W_6_NoSetTaskbar: [WCHAR; 13] = [0x004E, 0x006F, 0x0053, 0x0065, 0x0074, 0x0054, 0x0061, 0x0073, 0x006B, 0x0062, 0x0061, 0x0072, 0x0000]; // "NoSetTaskbar"
static W_7_NoDesktop: [WCHAR; 10] = [0x004E, 0x006F, 0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0000]; // "NoDesktop"
static W_8_NoFind: [WCHAR; 7] = [0x004E, 0x006F, 0x0046, 0x0069, 0x006E, 0x0064, 0x0000]; // "NoFind"
static W_9_NoDrives: [WCHAR; 9] = [0x004E, 0x006F, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0073, 0x0000]; // "NoDrives"
static W_10_NoDriveAutoRun: [WCHAR; 15] = [0x004E, 0x006F, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0041, 0x0075, 0x0074, 0x006F, 0x0052, 0x0075, 0x006E, 0x0000]; // "NoDriveAutoRun"
static W_11_NoDriveTypeAutoRun: [WCHAR; 19] = [0x004E, 0x006F, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0054, 0x0079, 0x0070, 0x0065, 0x0041, 0x0075, 0x0074, 0x006F, 0x0052, 0x0075, 0x006E, 0x0000]; // "NoDriveTypeAutoRun"
static W_12_NoNetHood: [WCHAR; 10] = [0x004E, 0x006F, 0x004E, 0x0065, 0x0074, 0x0048, 0x006F, 0x006F, 0x0064, 0x0000]; // "NoNetHood"
static W_13_NoStartBanner: [WCHAR; 14] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0042, 0x0061, 0x006E, 0x006E, 0x0065, 0x0072, 0x0000]; // "NoStartBanner"
static W_14_RestrictRun: [WCHAR; 12] = [0x0052, 0x0065, 0x0073, 0x0074, 0x0072, 0x0069, 0x0063, 0x0074, 0x0052, 0x0075, 0x006E, 0x0000]; // "RestrictRun"
static W_15_NoPrinterTabs: [WCHAR; 14] = [0x004E, 0x006F, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0054, 0x0061, 0x0062, 0x0073, 0x0000]; // "NoPrinterTabs"
static W_16_NoDeletePrinter: [WCHAR; 16] = [0x004E, 0x006F, 0x0044, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0000]; // "NoDeletePrinter"
static W_17_NoAddPrinter: [WCHAR; 13] = [0x004E, 0x006F, 0x0041, 0x0064, 0x0064, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x0065, 0x0072, 0x0000]; // "NoAddPrinter"
static W_18_NoStartMenuSubFolders: [WCHAR; 22] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0053, 0x0075, 0x0062, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0073, 0x0000]; // "NoStartMenuSubFolders"
static W_19_MyDocsOnNet: [WCHAR; 12] = [0x004D, 0x0079, 0x0044, 0x006F, 0x0063, 0x0073, 0x004F, 0x006E, 0x004E, 0x0065, 0x0074, 0x0000]; // "MyDocsOnNet"
static W_20_WinOldApp: [WCHAR; 10] = [0x0057, 0x0069, 0x006E, 0x004F, 0x006C, 0x0064, 0x0041, 0x0070, 0x0070, 0x0000]; // "WinOldApp"
static W_21_NoRealMode: [WCHAR; 11] = [0x004E, 0x006F, 0x0052, 0x0065, 0x0061, 0x006C, 0x004D, 0x006F, 0x0064, 0x0065, 0x0000]; // "NoRealMode"
static W_22_EnforceShellExtensionSecurity: [WCHAR; 30] = [0x0045, 0x006E, 0x0066, 0x006F, 0x0072, 0x0063, 0x0065, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0045, 0x0078, 0x0074, 0x0065, 0x006E, 0x0073, 0x0069, 0x006F, 0x006E, 0x0053, 0x0065, 0x0063, 0x0075, 0x0072, 0x0069, 0x0074, 0x0079, 0x0000]; // "EnforceShellExtensionSecurity"
static W_23_LinkResolveIgnoreLinkInfo: [WCHAR; 26] = [0x004C, 0x0069, 0x006E, 0x006B, 0x0052, 0x0065, 0x0073, 0x006F, 0x006C, 0x0076, 0x0065, 0x0049, 0x0067, 0x006E, 0x006F, 0x0072, 0x0065, 0x004C, 0x0069, 0x006E, 0x006B, 0x0049, 0x006E, 0x0066, 0x006F, 0x0000]; // "LinkResolveIgnoreLinkInfo"
static W_24_NoCommonGroups: [WCHAR; 15] = [0x004E, 0x006F, 0x0043, 0x006F, 0x006D, 0x006D, 0x006F, 0x006E, 0x0047, 0x0072, 0x006F, 0x0075, 0x0070, 0x0073, 0x0000]; // "NoCommonGroups"
static W_25_SeparateProcess: [WCHAR; 16] = [0x0053, 0x0065, 0x0070, 0x0061, 0x0072, 0x0061, 0x0074, 0x0065, 0x0050, 0x0072, 0x006F, 0x0063, 0x0065, 0x0073, 0x0073, 0x0000]; // "SeparateProcess"
static W_26_NoWebMenu: [WCHAR; 10] = [0x004E, 0x006F, 0x0057, 0x0065, 0x0062, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "NoWebMenu"
static W_27_NoTrayContextMenu: [WCHAR; 18] = [0x004E, 0x006F, 0x0054, 0x0072, 0x0061, 0x0079, 0x0043, 0x006F, 0x006E, 0x0074, 0x0065, 0x0078, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "NoTrayContextMenu"
static W_28_NoViewContextMenu: [WCHAR; 18] = [0x004E, 0x006F, 0x0056, 0x0069, 0x0065, 0x0077, 0x0043, 0x006F, 0x006E, 0x0074, 0x0065, 0x0078, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "NoViewContextMenu"
static W_29_NoNetConnectDisconnect: [WCHAR; 23] = [0x004E, 0x006F, 0x004E, 0x0065, 0x0074, 0x0043, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0044, 0x0069, 0x0073, 0x0063, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0000]; // "NoNetConnectDisconnect"
static W_30_StartMenuLogoff: [WCHAR; 16] = [0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x004C, 0x006F, 0x0067, 0x006F, 0x0066, 0x0066, 0x0000]; // "StartMenuLogoff"
static W_31_NoSettingsWizards: [WCHAR; 18] = [0x004E, 0x006F, 0x0053, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x0057, 0x0069, 0x007A, 0x0061, 0x0072, 0x0064, 0x0073, 0x0000]; // "NoSettingsWizards"
static W_32_NoInternetIcon: [WCHAR; 15] = [0x004E, 0x006F, 0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0065, 0x0074, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000]; // "NoInternetIcon"
static W_33_NoRecentDocsHistory: [WCHAR; 20] = [0x004E, 0x006F, 0x0052, 0x0065, 0x0063, 0x0065, 0x006E, 0x0074, 0x0044, 0x006F, 0x0063, 0x0073, 0x0048, 0x0069, 0x0073, 0x0074, 0x006F, 0x0072, 0x0079, 0x0000]; // "NoRecentDocsHistory"
static W_34_NoRecentDocsMenu: [WCHAR; 17] = [0x004E, 0x006F, 0x0052, 0x0065, 0x0063, 0x0065, 0x006E, 0x0074, 0x0044, 0x006F, 0x0063, 0x0073, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "NoRecentDocsMenu"
static W_35_NoActiveDesktop: [WCHAR; 16] = [0x004E, 0x006F, 0x0041, 0x0063, 0x0074, 0x0069, 0x0076, 0x0065, 0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0000]; // "NoActiveDesktop"
static W_36_NoActiveDesktopChanges: [WCHAR; 23] = [0x004E, 0x006F, 0x0041, 0x0063, 0x0074, 0x0069, 0x0076, 0x0065, 0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0043, 0x0068, 0x0061, 0x006E, 0x0067, 0x0065, 0x0073, 0x0000]; // "NoActiveDesktopChanges"
static W_37_NoFavoritesMenu: [WCHAR; 16] = [0x004E, 0x006F, 0x0046, 0x0061, 0x0076, 0x006F, 0x0072, 0x0069, 0x0074, 0x0065, 0x0073, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "NoFavoritesMenu"
static W_38_ClearRecentDocsOnExit: [WCHAR; 22] = [0x0043, 0x006C, 0x0065, 0x0061, 0x0072, 0x0052, 0x0065, 0x0063, 0x0065, 0x006E, 0x0074, 0x0044, 0x006F, 0x0063, 0x0073, 0x004F, 0x006E, 0x0045, 0x0078, 0x0069, 0x0074, 0x0000]; // "ClearRecentDocsOnExit"
static W_39_ClassicShell: [WCHAR; 13] = [0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0069, 0x0063, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0000]; // "ClassicShell"
static W_40_NoCustomizeWebView: [WCHAR; 19] = [0x004E, 0x006F, 0x0043, 0x0075, 0x0073, 0x0074, 0x006F, 0x006D, 0x0069, 0x007A, 0x0065, 0x0057, 0x0065, 0x0062, 0x0056, 0x0069, 0x0065, 0x0077, 0x0000]; // "NoCustomizeWebView"
static W_41_ActiveDesktop: [WCHAR; 14] = [0x0041, 0x0063, 0x0074, 0x0069, 0x0076, 0x0065, 0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0000]; // "ActiveDesktop"
static W_42_NoHTMLWallpaper: [WCHAR; 16] = [0x004E, 0x006F, 0x0048, 0x0054, 0x004D, 0x004C, 0x0057, 0x0061, 0x006C, 0x006C, 0x0070, 0x0061, 0x0070, 0x0065, 0x0072, 0x0000]; // "NoHTMLWallpaper"
static W_43_NoChangingWallpaper: [WCHAR; 20] = [0x004E, 0x006F, 0x0043, 0x0068, 0x0061, 0x006E, 0x0067, 0x0069, 0x006E, 0x0067, 0x0057, 0x0061, 0x006C, 0x006C, 0x0070, 0x0061, 0x0070, 0x0065, 0x0072, 0x0000]; // "NoChangingWallpaper"
static W_44_NoComponents: [WCHAR; 13] = [0x004E, 0x006F, 0x0043, 0x006F, 0x006D, 0x0070, 0x006F, 0x006E, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000]; // "NoComponents"
static W_45_NoAddingComponents: [WCHAR; 19] = [0x004E, 0x006F, 0x0041, 0x0064, 0x0064, 0x0069, 0x006E, 0x0067, 0x0043, 0x006F, 0x006D, 0x0070, 0x006F, 0x006E, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000]; // "NoAddingComponents"
static W_46_NoDeletingComponents: [WCHAR; 21] = [0x004E, 0x006F, 0x0044, 0x0065, 0x006C, 0x0065, 0x0074, 0x0069, 0x006E, 0x0067, 0x0043, 0x006F, 0x006D, 0x0070, 0x006F, 0x006E, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000]; // "NoDeletingComponents"
static W_47_NoClosingComponents: [WCHAR; 20] = [0x004E, 0x006F, 0x0043, 0x006C, 0x006F, 0x0073, 0x0069, 0x006E, 0x0067, 0x0043, 0x006F, 0x006D, 0x0070, 0x006F, 0x006E, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000]; // "NoClosingComponents"
static W_48_NoCloseDragDropBands: [WCHAR; 21] = [0x004E, 0x006F, 0x0043, 0x006C, 0x006F, 0x0073, 0x0065, 0x0044, 0x0072, 0x0061, 0x0067, 0x0044, 0x0072, 0x006F, 0x0070, 0x0042, 0x0061, 0x006E, 0x0064, 0x0073, 0x0000]; // "NoCloseDragDropBands"
static W_49_NoMovingBands: [WCHAR; 14] = [0x004E, 0x006F, 0x004D, 0x006F, 0x0076, 0x0069, 0x006E, 0x0067, 0x0042, 0x0061, 0x006E, 0x0064, 0x0073, 0x0000]; // "NoMovingBands"
static W_50_NoEditingComponents: [WCHAR; 20] = [0x004E, 0x006F, 0x0045, 0x0064, 0x0069, 0x0074, 0x0069, 0x006E, 0x0067, 0x0043, 0x006F, 0x006D, 0x0070, 0x006F, 0x006E, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000]; // "NoEditingComponents"
static W_51_NoResolveSearch: [WCHAR; 16] = [0x004E, 0x006F, 0x0052, 0x0065, 0x0073, 0x006F, 0x006C, 0x0076, 0x0065, 0x0053, 0x0065, 0x0061, 0x0072, 0x0063, 0x0068, 0x0000]; // "NoResolveSearch"
static W_52_NoResolveTrack: [WCHAR; 15] = [0x004E, 0x006F, 0x0052, 0x0065, 0x0073, 0x006F, 0x006C, 0x0076, 0x0065, 0x0054, 0x0072, 0x0061, 0x0063, 0x006B, 0x0000]; // "NoResolveTrack"
static W_53_ForceCopyACLWithFile: [WCHAR; 21] = [0x0046, 0x006F, 0x0072, 0x0063, 0x0065, 0x0043, 0x006F, 0x0070, 0x0079, 0x0041, 0x0043, 0x004C, 0x0057, 0x0069, 0x0074, 0x0068, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000]; // "ForceCopyACLWithFile"
static W_54_NoForgetSoftwareUpdate: [WCHAR; 23] = [0x004E, 0x006F, 0x0046, 0x006F, 0x0072, 0x0067, 0x0065, 0x0074, 0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x0055, 0x0070, 0x0064, 0x0061, 0x0074, 0x0065, 0x0000]; // "NoForgetSoftwareUpdate"
static W_55_NoSetActiveDesktop: [WCHAR; 19] = [0x004E, 0x006F, 0x0053, 0x0065, 0x0074, 0x0041, 0x0063, 0x0074, 0x0069, 0x0076, 0x0065, 0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x0000]; // "NoSetActiveDesktop"
static W_56_NoWindowsUpdate: [WCHAR; 16] = [0x004E, 0x006F, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0055, 0x0070, 0x0064, 0x0061, 0x0074, 0x0065, 0x0000]; // "NoWindowsUpdate"
static W_57_NoChangeStartMenu: [WCHAR; 18] = [0x004E, 0x006F, 0x0043, 0x0068, 0x0061, 0x006E, 0x0067, 0x0065, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "NoChangeStartMenu"
static W_58_NoFolderOptions: [WCHAR; 16] = [0x004E, 0x006F, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x004F, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000]; // "NoFolderOptions"
static W_59_FindComputers: [WCHAR; 14] = [0x0046, 0x0069, 0x006E, 0x0064, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000]; // "FindComputers"
static W_60_IntelliMenus: [WCHAR; 13] = [0x0049, 0x006E, 0x0074, 0x0065, 0x006C, 0x006C, 0x0069, 0x004D, 0x0065, 0x006E, 0x0075, 0x0073, 0x0000]; // "IntelliMenus"
static W_61_MemCheckBoxInRunDlg: [WCHAR; 20] = [0x004D, 0x0065, 0x006D, 0x0043, 0x0068, 0x0065, 0x0063, 0x006B, 0x0042, 0x006F, 0x0078, 0x0049, 0x006E, 0x0052, 0x0075, 0x006E, 0x0044, 0x006C, 0x0067, 0x0000]; // "MemCheckBoxInRunDlg"
static W_62_Uninstall: [WCHAR; 10] = [0x0055, 0x006E, 0x0069, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0000]; // "Uninstall"
static W_63_ShowPostSetup: [WCHAR; 14] = [0x0053, 0x0068, 0x006F, 0x0077, 0x0050, 0x006F, 0x0073, 0x0074, 0x0053, 0x0065, 0x0074, 0x0075, 0x0070, 0x0000]; // "ShowPostSetup"
static W_64_NoSyncAll: [WCHAR; 10] = [0x004E, 0x006F, 0x0053, 0x0079, 0x006E, 0x0063, 0x0041, 0x006C, 0x006C, 0x0000]; // "NoSyncAll"
static W_65_NoControlPanel: [WCHAR; 15] = [0x004E, 0x006F, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0050, 0x0061, 0x006E, 0x0065, 0x006C, 0x0000]; // "NoControlPanel"
static W_66_EnumWorkgroup: [WCHAR; 14] = [0x0045, 0x006E, 0x0075, 0x006D, 0x0057, 0x006F, 0x0072, 0x006B, 0x0067, 0x0072, 0x006F, 0x0075, 0x0070, 0x0000]; // "EnumWorkgroup"
static W_67_NoAddRemovePrograms: [WCHAR; 20] = [0x004E, 0x006F, 0x0041, 0x0064, 0x0064, 0x0052, 0x0065, 0x006D, 0x006F, 0x0076, 0x0065, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0073, 0x0000]; // "NoAddRemovePrograms"
static W_68_NoRemovePage: [WCHAR; 13] = [0x004E, 0x006F, 0x0052, 0x0065, 0x006D, 0x006F, 0x0076, 0x0065, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000]; // "NoRemovePage"
static W_69_NoAddPage: [WCHAR; 10] = [0x004E, 0x006F, 0x0041, 0x0064, 0x0064, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000]; // "NoAddPage"
static W_70_NoWindowsSetupPage: [WCHAR; 19] = [0x004E, 0x006F, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0053, 0x0065, 0x0074, 0x0075, 0x0070, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000]; // "NoWindowsSetupPage"
static W_71_GreyMSIAds: [WCHAR; 11] = [0x0047, 0x0072, 0x0065, 0x0079, 0x004D, 0x0053, 0x0049, 0x0041, 0x0064, 0x0073, 0x0000]; // "GreyMSIAds"
static W_72_NoChangeMappedDriveLabel: [WCHAR; 25] = [0x004E, 0x006F, 0x0043, 0x0068, 0x0061, 0x006E, 0x0067, 0x0065, 0x004D, 0x0061, 0x0070, 0x0070, 0x0065, 0x0064, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x004C, 0x0061, 0x0062, 0x0065, 0x006C, 0x0000]; // "NoChangeMappedDriveLabel"
static W_73_NoChangeMappedDriveComment: [WCHAR; 27] = [0x004E, 0x006F, 0x0043, 0x0068, 0x0061, 0x006E, 0x0067, 0x0065, 0x004D, 0x0061, 0x0070, 0x0070, 0x0065, 0x0064, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0043, 0x006F, 0x006D, 0x006D, 0x0065, 0x006E, 0x0074, 0x0000]; // "NoChangeMappedDriveComment"
static W_74_MaxRecentDocs: [WCHAR; 14] = [0x004D, 0x0061, 0x0078, 0x0052, 0x0065, 0x0063, 0x0065, 0x006E, 0x0074, 0x0044, 0x006F, 0x0063, 0x0073, 0x0000]; // "MaxRecentDocs"
static W_75_NoNetworkConnections: [WCHAR; 21] = [0x004E, 0x006F, 0x004E, 0x0065, 0x0074, 0x0077, 0x006F, 0x0072, 0x006B, 0x0043, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000]; // "NoNetworkConnections"
static W_76_ForceStartMenuLogoff: [WCHAR; 21] = [0x0046, 0x006F, 0x0072, 0x0063, 0x0065, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x004C, 0x006F, 0x0067, 0x006F, 0x0066, 0x0066, 0x0000]; // "ForceStartMenuLogoff"
static W_77_NoWebView: [WCHAR; 10] = [0x004E, 0x006F, 0x0057, 0x0065, 0x0062, 0x0056, 0x0069, 0x0065, 0x0077, 0x0000]; // "NoWebView"
static W_78_NoCustomizeThisFolder: [WCHAR; 22] = [0x004E, 0x006F, 0x0043, 0x0075, 0x0073, 0x0074, 0x006F, 0x006D, 0x0069, 0x007A, 0x0065, 0x0054, 0x0068, 0x0069, 0x0073, 0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000]; // "NoCustomizeThisFolder"
static W_79_NoEncryption: [WCHAR; 13] = [0x004E, 0x006F, 0x0045, 0x006E, 0x0063, 0x0072, 0x0079, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000]; // "NoEncryption"
static W_80_DontShowSuperHidden: [WCHAR; 20] = [0x0044, 0x006F, 0x006E, 0x0074, 0x0053, 0x0068, 0x006F, 0x0077, 0x0053, 0x0075, 0x0070, 0x0065, 0x0072, 0x0048, 0x0069, 0x0064, 0x0064, 0x0065, 0x006E, 0x0000]; // "DontShowSuperHidden"
static W_81_NoShellSearchButton: [WCHAR; 20] = [0x004E, 0x006F, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0053, 0x0065, 0x0061, 0x0072, 0x0063, 0x0068, 0x0042, 0x0075, 0x0074, 0x0074, 0x006F, 0x006E, 0x0000]; // "NoShellSearchButton"
static W_82_NoHardwareTab: [WCHAR; 14] = [0x004E, 0x006F, 0x0048, 0x0061, 0x0072, 0x0064, 0x0077, 0x0061, 0x0072, 0x0065, 0x0054, 0x0061, 0x0062, 0x0000]; // "NoHardwareTab"
static W_83_NoManageMyComputerVerb: [WCHAR; 23] = [0x004E, 0x006F, 0x004D, 0x0061, 0x006E, 0x0061, 0x0067, 0x0065, 0x004D, 0x0079, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x0056, 0x0065, 0x0072, 0x0062, 0x0000]; // "NoManageMyComputerVerb"
static W_84_DisallowRun: [WCHAR; 12] = [0x0044, 0x0069, 0x0073, 0x0061, 0x006C, 0x006C, 0x006F, 0x0077, 0x0052, 0x0075, 0x006E, 0x0000]; // "DisallowRun"
static W_85_NoWelcomeScreen: [WCHAR; 16] = [0x004E, 0x006F, 0x0057, 0x0065, 0x006C, 0x0063, 0x006F, 0x006D, 0x0065, 0x0053, 0x0063, 0x0072, 0x0065, 0x0065, 0x006E, 0x0000]; // "NoWelcomeScreen"
static W_86_RestrictCpl: [WCHAR; 12] = [0x0052, 0x0065, 0x0073, 0x0074, 0x0072, 0x0069, 0x0063, 0x0074, 0x0043, 0x0070, 0x006C, 0x0000]; // "RestrictCpl"
static W_87_DisallowCpl: [WCHAR; 12] = [0x0044, 0x0069, 0x0073, 0x0061, 0x006C, 0x006C, 0x006F, 0x0077, 0x0043, 0x0070, 0x006C, 0x0000]; // "DisallowCpl"
static W_88_NoSMBalloonTip: [WCHAR; 15] = [0x004E, 0x006F, 0x0053, 0x004D, 0x0042, 0x0061, 0x006C, 0x006C, 0x006F, 0x006F, 0x006E, 0x0054, 0x0069, 0x0070, 0x0000]; // "NoSMBalloonTip"
static W_89_NoSMHelp: [WCHAR; 9] = [0x004E, 0x006F, 0x0053, 0x004D, 0x0048, 0x0065, 0x006C, 0x0070, 0x0000]; // "NoSMHelp"
static W_90_NoWinKeys: [WCHAR; 10] = [0x004E, 0x006F, 0x0057, 0x0069, 0x006E, 0x004B, 0x0065, 0x0079, 0x0073, 0x0000]; // "NoWinKeys"
static W_91_NoEncryptOnMove: [WCHAR; 16] = [0x004E, 0x006F, 0x0045, 0x006E, 0x0063, 0x0072, 0x0079, 0x0070, 0x0074, 0x004F, 0x006E, 0x004D, 0x006F, 0x0076, 0x0065, 0x0000]; // "NoEncryptOnMove"
static W_92_DisableLocalMachineRun: [WCHAR; 23] = [0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x004C, 0x006F, 0x0063, 0x0061, 0x006C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x0052, 0x0075, 0x006E, 0x0000]; // "DisableLocalMachineRun"
static W_93_DisableCurrentUserRun: [WCHAR; 22] = [0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0055, 0x0073, 0x0065, 0x0072, 0x0052, 0x0075, 0x006E, 0x0000]; // "DisableCurrentUserRun"
static W_94_DisableLocalMachineRunOnce: [WCHAR; 27] = [0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x004C, 0x006F, 0x0063, 0x0061, 0x006C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x0052, 0x0075, 0x006E, 0x004F, 0x006E, 0x0063, 0x0065, 0x0000]; // "DisableLocalMachineRunOnce"
static W_95_DisableCurrentUserRunOnce: [WCHAR; 26] = [0x0044, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0055, 0x0073, 0x0065, 0x0072, 0x0052, 0x0075, 0x006E, 0x004F, 0x006E, 0x0063, 0x0065, 0x0000]; // "DisableCurrentUserRunOnce"
static W_96_ForceActiveDesktopOn: [WCHAR; 21] = [0x0046, 0x006F, 0x0072, 0x0063, 0x0065, 0x0041, 0x0063, 0x0074, 0x0069, 0x0076, 0x0065, 0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x004F, 0x006E, 0x0000]; // "ForceActiveDesktopOn"
static W_97_NoViewOnDrive: [WCHAR; 14] = [0x004E, 0x006F, 0x0056, 0x0069, 0x0065, 0x0077, 0x004F, 0x006E, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0000]; // "NoViewOnDrive"
static W_98_NoNetCrawling: [WCHAR; 14] = [0x004E, 0x006F, 0x004E, 0x0065, 0x0074, 0x0043, 0x0072, 0x0061, 0x0077, 0x006C, 0x0069, 0x006E, 0x0067, 0x0000]; // "NoNetCrawling"
static W_99_NoSharedDocuments: [WCHAR; 18] = [0x004E, 0x006F, 0x0053, 0x0068, 0x0061, 0x0072, 0x0065, 0x0064, 0x0044, 0x006F, 0x0063, 0x0075, 0x006D, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000]; // "NoSharedDocuments"
static W_100_NoSMMyDocs: [WCHAR; 11] = [0x004E, 0x006F, 0x0053, 0x004D, 0x004D, 0x0079, 0x0044, 0x006F, 0x0063, 0x0073, 0x0000]; // "NoSMMyDocs"
static W_101_NoSMMyPictures: [WCHAR; 15] = [0x004E, 0x006F, 0x0053, 0x004D, 0x004D, 0x0079, 0x0050, 0x0069, 0x0063, 0x0074, 0x0075, 0x0072, 0x0065, 0x0073, 0x0000]; // "NoSMMyPictures"
static W_102_RecycleBinDrives: [WCHAR; 17] = [0x0052, 0x0065, 0x0063, 0x0079, 0x0063, 0x006C, 0x0065, 0x0042, 0x0069, 0x006E, 0x0044, 0x0072, 0x0069, 0x0076, 0x0065, 0x0073, 0x0000]; // "RecycleBinDrives"
static W_103_NoneLegacyShellMode: [WCHAR; 20] = [0x004E, 0x006F, 0x006E, 0x0065, 0x004C, 0x0065, 0x0067, 0x0061, 0x0063, 0x0079, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x004D, 0x006F, 0x0064, 0x0065, 0x0000]; // "NoneLegacyShellMode"
static W_104_NoControlPanelBarricade: [WCHAR; 24] = [0x004E, 0x006F, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0050, 0x0061, 0x006E, 0x0065, 0x006C, 0x0042, 0x0061, 0x0072, 0x0072, 0x0069, 0x0063, 0x0061, 0x0064, 0x0065, 0x0000]; // "NoControlPanelBarricade"
static W_105_NoAutoTrayNotify: [WCHAR; 17] = [0x004E, 0x006F, 0x0041, 0x0075, 0x0074, 0x006F, 0x0054, 0x0072, 0x0061, 0x0079, 0x004E, 0x006F, 0x0074, 0x0069, 0x0066, 0x0079, 0x0000]; // "NoAutoTrayNotify"
static W_106_NoTaskGrouping: [WCHAR; 15] = [0x004E, 0x006F, 0x0054, 0x0061, 0x0073, 0x006B, 0x0047, 0x0072, 0x006F, 0x0075, 0x0070, 0x0069, 0x006E, 0x0067, 0x0000]; // "NoTaskGrouping"
static W_107_NoCDBurning: [WCHAR; 12] = [0x004E, 0x006F, 0x0043, 0x0044, 0x0042, 0x0075, 0x0072, 0x006E, 0x0069, 0x006E, 0x0067, 0x0000]; // "NoCDBurning"
static W_108_NoPropertiesMyComputer: [WCHAR; 23] = [0x004E, 0x006F, 0x0050, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0069, 0x0065, 0x0073, 0x004D, 0x0079, 0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x0000]; // "NoPropertiesMyComputer"
static W_109_NoPropertiesMyDocuments: [WCHAR; 24] = [0x004E, 0x006F, 0x0050, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0069, 0x0065, 0x0073, 0x004D, 0x0079, 0x0044, 0x006F, 0x0063, 0x0075, 0x006D, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000]; // "NoPropertiesMyDocuments"
static W_110_NoSimpleStartMenu: [WCHAR; 18] = [0x004E, 0x006F, 0x0053, 0x0069, 0x006D, 0x0070, 0x006C, 0x0065, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "NoSimpleStartMenu"
static W_111_System: [WCHAR; 7] = [0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000]; // "System"
static W_112_NoDispAppearancePage: [WCHAR; 21] = [0x004E, 0x006F, 0x0044, 0x0069, 0x0073, 0x0070, 0x0041, 0x0070, 0x0070, 0x0065, 0x0061, 0x0072, 0x0061, 0x006E, 0x0063, 0x0065, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000]; // "NoDispAppearancePage"
static W_113_NoThemesTab: [WCHAR; 12] = [0x004E, 0x006F, 0x0054, 0x0068, 0x0065, 0x006D, 0x0065, 0x0073, 0x0054, 0x0061, 0x0062, 0x0000]; // "NoThemesTab"
static W_114_NoVisualStyleChoice: [WCHAR; 20] = [0x004E, 0x006F, 0x0056, 0x0069, 0x0073, 0x0075, 0x0061, 0x006C, 0x0053, 0x0074, 0x0079, 0x006C, 0x0065, 0x0043, 0x0068, 0x006F, 0x0069, 0x0063, 0x0065, 0x0000]; // "NoVisualStyleChoice"
static W_115_NoSizeChoice: [WCHAR; 13] = [0x004E, 0x006F, 0x0053, 0x0069, 0x007A, 0x0065, 0x0043, 0x0068, 0x006F, 0x0069, 0x0063, 0x0065, 0x0000]; // "NoSizeChoice"
static W_116_NoColorChoice: [WCHAR; 14] = [0x004E, 0x006F, 0x0043, 0x006F, 0x006C, 0x006F, 0x0072, 0x0043, 0x0068, 0x006F, 0x0069, 0x0063, 0x0065, 0x0000]; // "NoColorChoice"
static W_117_SetVisualStyle: [WCHAR; 15] = [0x0053, 0x0065, 0x0074, 0x0056, 0x0069, 0x0073, 0x0075, 0x0061, 0x006C, 0x0053, 0x0074, 0x0079, 0x006C, 0x0065, 0x0000]; // "SetVisualStyle"
static W_118_StartRunNoHOMEPATH: [WCHAR; 19] = [0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0052, 0x0075, 0x006E, 0x004E, 0x006F, 0x0048, 0x004F, 0x004D, 0x0045, 0x0050, 0x0041, 0x0054, 0x0048, 0x0000]; // "StartRunNoHOMEPATH"
static W_119_NoUserNameInStartMenu: [WCHAR; 22] = [0x004E, 0x006F, 0x0055, 0x0073, 0x0065, 0x0072, 0x004E, 0x0061, 0x006D, 0x0065, 0x0049, 0x006E, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0000]; // "NoUserNameInStartMenu"
static W_120_NonEnum: [WCHAR; 8] = [0x004E, 0x006F, 0x006E, 0x0045, 0x006E, 0x0075, 0x006D, 0x0000]; // "NonEnum"
static W_121__20D04FE0_3AEA_1069_A2D8_08002B30309D_: [WCHAR; 39] = [0x007B, 0x0032, 0x0030, 0x0044, 0x0030, 0x0034, 0x0046, 0x0045, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0038, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x0000]; // "{20D04FE0-3AEA-1069-A2D8-08002B30309D}"
static W_122_NoStartMenuNetworkPlaces: [WCHAR; 25] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x004E, 0x0065, 0x0074, 0x0077, 0x006F, 0x0072, 0x006B, 0x0050, 0x006C, 0x0061, 0x0063, 0x0065, 0x0073, 0x0000]; // "NoStartMenuNetworkPlaces"
static W_123_NoStartMenuPinnedList: [WCHAR; 22] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0050, 0x0069, 0x006E, 0x006E, 0x0065, 0x0064, 0x004C, 0x0069, 0x0073, 0x0074, 0x0000]; // "NoStartMenuPinnedList"
static W_124_NoStartMenuMyMusic: [WCHAR; 19] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x004D, 0x0079, 0x004D, 0x0075, 0x0073, 0x0069, 0x0063, 0x0000]; // "NoStartMenuMyMusic"
static W_125_NoStartMenuEjectPC: [WCHAR; 19] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x0045, 0x006A, 0x0065, 0x0063, 0x0074, 0x0050, 0x0043, 0x0000]; // "NoStartMenuEjectPC"
static W_126_NoStartMenuMorePrograms: [WCHAR; 24] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x004D, 0x006F, 0x0072, 0x0065, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0073, 0x0000]; // "NoStartMenuMorePrograms"
static W_127_NoStartMenuMFUprogramsList: [WCHAR; 27] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x004D, 0x0065, 0x006E, 0x0075, 0x004D, 0x0046, 0x0055, 0x0070, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0073, 0x004C, 0x0069, 0x0073, 0x0074, 0x0000]; // "NoStartMenuMFUprogramsList"
static W_128_NoTrayItemsDisplay: [WCHAR; 19] = [0x004E, 0x006F, 0x0054, 0x0072, 0x0061, 0x0079, 0x0049, 0x0074, 0x0065, 0x006D, 0x0073, 0x0044, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x0000]; // "NoTrayItemsDisplay"
static W_129_NoToolbarsOnTaskbar: [WCHAR; 20] = [0x004E, 0x006F, 0x0054, 0x006F, 0x006F, 0x006C, 0x0062, 0x0061, 0x0072, 0x0073, 0x004F, 0x006E, 0x0054, 0x0061, 0x0073, 0x006B, 0x0062, 0x0061, 0x0072, 0x0000]; // "NoToolbarsOnTaskbar"
static W_130_NoSMConfigurePrograms: [WCHAR; 22] = [0x004E, 0x006F, 0x0053, 0x004D, 0x0043, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x0075, 0x0072, 0x0065, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0073, 0x0000]; // "NoSMConfigurePrograms"
static W_131_HideClock: [WCHAR; 10] = [0x0048, 0x0069, 0x0064, 0x0065, 0x0043, 0x006C, 0x006F, 0x0063, 0x006B, 0x0000]; // "HideClock"
static W_132_NoLowDiskSpaceChecks: [WCHAR; 21] = [0x004E, 0x006F, 0x004C, 0x006F, 0x0077, 0x0044, 0x0069, 0x0073, 0x006B, 0x0053, 0x0070, 0x0061, 0x0063, 0x0065, 0x0043, 0x0068, 0x0065, 0x0063, 0x006B, 0x0073, 0x0000]; // "NoLowDiskSpaceChecks"
static W_133_NoRecycleFiles: [WCHAR; 15] = [0x004E, 0x006F, 0x0052, 0x0065, 0x0063, 0x0079, 0x0063, 0x006C, 0x0065, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0000]; // "NoRecycleFiles"
static W_134_ConfirmFileDelete: [WCHAR; 18] = [0x0043, 0x006F, 0x006E, 0x0066, 0x0069, 0x0072, 0x006D, 0x0046, 0x0069, 0x006C, 0x0065, 0x0044, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000]; // "ConfirmFileDelete"
static W_135_NoPropertiesRecycleBin: [WCHAR; 23] = [0x004E, 0x006F, 0x0050, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0069, 0x0065, 0x0073, 0x0052, 0x0065, 0x0063, 0x0079, 0x0063, 0x006C, 0x0065, 0x0042, 0x0069, 0x006E, 0x0000]; // "NoPropertiesRecycleBin"
static W_136_NoDispBackgroundPage: [WCHAR; 21] = [0x004E, 0x006F, 0x0044, 0x0069, 0x0073, 0x0070, 0x0042, 0x0061, 0x0063, 0x006B, 0x0067, 0x0072, 0x006F, 0x0075, 0x006E, 0x0064, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000]; // "NoDispBackgroundPage"
static W_137_NoDispScrSavPage: [WCHAR; 17] = [0x004E, 0x006F, 0x0044, 0x0069, 0x0073, 0x0070, 0x0053, 0x0063, 0x0072, 0x0053, 0x0061, 0x0076, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000]; // "NoDispScrSavPage"
static W_138_NoDispSettingsPage: [WCHAR; 19] = [0x004E, 0x006F, 0x0044, 0x0069, 0x0073, 0x0070, 0x0053, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E, 0x0067, 0x0073, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000]; // "NoDispSettingsPage"
static W_139_NoScreenSavePreview: [WCHAR; 20] = [0x004E, 0x006F, 0x0053, 0x0063, 0x0072, 0x0065, 0x0065, 0x006E, 0x0053, 0x0061, 0x0076, 0x0065, 0x0050, 0x0072, 0x0065, 0x0076, 0x0069, 0x0065, 0x0077, 0x0000]; // "NoScreenSavePreview"
static W_140_NoDispCPL: [WCHAR; 10] = [0x004E, 0x006F, 0x0044, 0x0069, 0x0073, 0x0070, 0x0043, 0x0050, 0x004C, 0x0000]; // "NoDispCPL"
static W_141_HideRunAsVerb: [WCHAR; 14] = [0x0048, 0x0069, 0x0064, 0x0065, 0x0052, 0x0075, 0x006E, 0x0041, 0x0073, 0x0056, 0x0065, 0x0072, 0x0062, 0x0000]; // "HideRunAsVerb"
static W_142_NoThumbnailCache: [WCHAR; 17] = [0x004E, 0x006F, 0x0054, 0x0068, 0x0075, 0x006D, 0x0062, 0x006E, 0x0061, 0x0069, 0x006C, 0x0043, 0x0061, 0x0063, 0x0068, 0x0065, 0x0000]; // "NoThumbnailCache"
static W_143_NoStrCmpLogical: [WCHAR; 16] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0072, 0x0043, 0x006D, 0x0070, 0x004C, 0x006F, 0x0067, 0x0069, 0x0063, 0x0061, 0x006C, 0x0000]; // "NoStrCmpLogical"
static W_144_NoPublishingWizard: [WCHAR; 19] = [0x004E, 0x006F, 0x0050, 0x0075, 0x0062, 0x006C, 0x0069, 0x0073, 0x0068, 0x0069, 0x006E, 0x0067, 0x0057, 0x0069, 0x007A, 0x0061, 0x0072, 0x0064, 0x0000]; // "NoPublishingWizard"
static W_145_NoOnlinePrintsWizard: [WCHAR; 21] = [0x004E, 0x006F, 0x004F, 0x006E, 0x006C, 0x0069, 0x006E, 0x0065, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x0073, 0x0057, 0x0069, 0x007A, 0x0061, 0x0072, 0x0064, 0x0000]; // "NoOnlinePrintsWizard"
static W_146_NoWebServices: [WCHAR; 14] = [0x004E, 0x006F, 0x0057, 0x0065, 0x0062, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x0000]; // "NoWebServices"
static W_147_AllowUnhashedWebView: [WCHAR; 21] = [0x0041, 0x006C, 0x006C, 0x006F, 0x0077, 0x0055, 0x006E, 0x0068, 0x0061, 0x0073, 0x0068, 0x0065, 0x0064, 0x0057, 0x0065, 0x0062, 0x0056, 0x0069, 0x0065, 0x0077, 0x0000]; // "AllowUnhashedWebView"
static W_148_AllowLegacyWebView: [WCHAR; 19] = [0x0041, 0x006C, 0x006C, 0x006F, 0x0077, 0x004C, 0x0065, 0x0067, 0x0061, 0x0063, 0x0079, 0x0057, 0x0065, 0x0062, 0x0056, 0x0069, 0x0065, 0x0077, 0x0000]; // "AllowLegacyWebView"
static W_149_RevertWebViewSecurity: [WCHAR; 22] = [0x0052, 0x0065, 0x0076, 0x0065, 0x0072, 0x0074, 0x0057, 0x0065, 0x0062, 0x0056, 0x0069, 0x0065, 0x0077, 0x0053, 0x0065, 0x0063, 0x0075, 0x0072, 0x0069, 0x0074, 0x0079, 0x0000]; // "RevertWebViewSecurity"
static W_150_InheritConsoleHandles: [WCHAR; 22] = [0x0049, 0x006E, 0x0068, 0x0065, 0x0072, 0x0069, 0x0074, 0x0043, 0x006F, 0x006E, 0x0073, 0x006F, 0x006C, 0x0065, 0x0048, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0073, 0x0000]; // "InheritConsoleHandles"
static W_151_NoRemoteRecursiveEvents: [WCHAR; 24] = [0x004E, 0x006F, 0x0052, 0x0065, 0x006D, 0x006F, 0x0074, 0x0065, 0x0052, 0x0065, 0x0063, 0x0075, 0x0072, 0x0073, 0x0069, 0x0076, 0x0065, 0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x0073, 0x0000]; // "NoRemoteRecursiveEvents"
static W_152_NoRemoteChangeNotify: [WCHAR; 21] = [0x004E, 0x006F, 0x0052, 0x0065, 0x006D, 0x006F, 0x0074, 0x0065, 0x0043, 0x0068, 0x0061, 0x006E, 0x0067, 0x0065, 0x004E, 0x006F, 0x0074, 0x0069, 0x0066, 0x0079, 0x0000]; // "NoRemoteChangeNotify"
static W_153_NoEnumEntireNetwork: [WCHAR; 20] = [0x004E, 0x006F, 0x0045, 0x006E, 0x0075, 0x006D, 0x0045, 0x006E, 0x0074, 0x0069, 0x0072, 0x0065, 0x004E, 0x0065, 0x0074, 0x0077, 0x006F, 0x0072, 0x006B, 0x0000]; // "NoEnumEntireNetwork"
static W_154_NoInternetOpenWith: [WCHAR; 19] = [0x004E, 0x006F, 0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0065, 0x0074, 0x004F, 0x0070, 0x0065, 0x006E, 0x0057, 0x0069, 0x0074, 0x0068, 0x0000]; // "NoInternetOpenWith"
static W_155_DontRetryBadNetName: [WCHAR; 20] = [0x0044, 0x006F, 0x006E, 0x0074, 0x0052, 0x0065, 0x0074, 0x0072, 0x0079, 0x0042, 0x0061, 0x0064, 0x004E, 0x0065, 0x0074, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000]; // "DontRetryBadNetName"
static W_156_AllowFileCLSIDJunctions: [WCHAR; 24] = [0x0041, 0x006C, 0x006C, 0x006F, 0x0077, 0x0046, 0x0069, 0x006C, 0x0065, 0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x004A, 0x0075, 0x006E, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000]; // "AllowFileCLSIDJunctions"
static W_157_NoUPnPInstallTask: [WCHAR; 18] = [0x004E, 0x006F, 0x0055, 0x0050, 0x006E, 0x0050, 0x0049, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0054, 0x0061, 0x0073, 0x006B, 0x0000]; // "NoUPnPInstallTask"
static W_158_DontGroupPatches: [WCHAR; 17] = [0x0044, 0x006F, 0x006E, 0x0074, 0x0047, 0x0072, 0x006F, 0x0075, 0x0070, 0x0050, 0x0061, 0x0074, 0x0063, 0x0068, 0x0065, 0x0073, 0x0000]; // "DontGroupPatches"
static W_159_NoChooseProgramsPage: [WCHAR; 21] = [0x004E, 0x006F, 0x0043, 0x0068, 0x006F, 0x006F, 0x0073, 0x0065, 0x0050, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0073, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000]; // "NoChooseProgramsPage"
static W_160_NoDisconnect: [WCHAR; 13] = [0x004E, 0x006F, 0x0044, 0x0069, 0x0073, 0x0063, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0000]; // "NoDisconnect"
static W_161_NoNTSecurity: [WCHAR; 13] = [0x004E, 0x006F, 0x004E, 0x0054, 0x0053, 0x0065, 0x0063, 0x0075, 0x0072, 0x0069, 0x0074, 0x0079, 0x0000]; // "NoNTSecurity"
static W_162_NoFileAssociate: [WCHAR; 16] = [0x004E, 0x006F, 0x0046, 0x0069, 0x006C, 0x0065, 0x0041, 0x0073, 0x0073, 0x006F, 0x0063, 0x0069, 0x0061, 0x0074, 0x0065, 0x0000]; // "NoFileAssociate"
static W_163_ToggleCommentPosition: [WCHAR; 22] = [0x0054, 0x006F, 0x0067, 0x0067, 0x006C, 0x0065, 0x0043, 0x006F, 0x006D, 0x006D, 0x0065, 0x006E, 0x0074, 0x0050, 0x006F, 0x0073, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000]; // "ToggleCommentPosition"
static W_164_EnableShellExecuteHooks: [WCHAR; 24] = [0x0045, 0x006E, 0x0061, 0x0062, 0x006C, 0x0065, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0045, 0x0078, 0x0065, 0x0063, 0x0075, 0x0074, 0x0065, 0x0048, 0x006F, 0x006F, 0x006B, 0x0073, 0x0000]; // "EnableShellExecuteHooks"

// ---- 159 policy rows + NULL terminator = 160 entries (== _countof(s_PolicyTable)) ----
const POLICY_TABLE_LEN: usize = 160;
static S_POLICY_TABLE: PolicyTable = PolicyTable([
    POLICYDATA { policy: 0x00000001, appstr: W_0_Explorer.as_ptr(), keystr: W_1_NoRun.as_ptr() }, // REST_NORUN
    POLICYDATA { policy: 0x00000002, appstr: W_0_Explorer.as_ptr(), keystr: W_2_NoClose.as_ptr() }, // REST_NOCLOSE
    POLICYDATA { policy: 0x00000004, appstr: W_0_Explorer.as_ptr(), keystr: W_3_NoSaveSettings.as_ptr() }, // REST_NOSAVESET
    POLICYDATA { policy: 0x00000008, appstr: W_0_Explorer.as_ptr(), keystr: W_4_NoFileMenu.as_ptr() }, // REST_NOFILEMENU
    POLICYDATA { policy: 0x00000010, appstr: W_0_Explorer.as_ptr(), keystr: W_5_NoSetFolders.as_ptr() }, // REST_NOSETFOLDERS
    POLICYDATA { policy: 0x00000020, appstr: W_0_Explorer.as_ptr(), keystr: W_6_NoSetTaskbar.as_ptr() }, // REST_NOSETTASKBAR
    POLICYDATA { policy: 0x00000040, appstr: W_0_Explorer.as_ptr(), keystr: W_7_NoDesktop.as_ptr() }, // REST_NODESKTOP
    POLICYDATA { policy: 0x00000080, appstr: W_0_Explorer.as_ptr(), keystr: W_8_NoFind.as_ptr() }, // REST_NOFIND
    POLICYDATA { policy: 0x00000100, appstr: W_0_Explorer.as_ptr(), keystr: W_9_NoDrives.as_ptr() }, // REST_NODRIVES
    POLICYDATA { policy: 0x00000200, appstr: W_0_Explorer.as_ptr(), keystr: W_10_NoDriveAutoRun.as_ptr() }, // REST_NODRIVEAUTORUN
    POLICYDATA { policy: 0x00000400, appstr: W_0_Explorer.as_ptr(), keystr: W_11_NoDriveTypeAutoRun.as_ptr() }, // REST_NODRIVETYPEAUTORUN
    POLICYDATA { policy: 0x00000800, appstr: W_0_Explorer.as_ptr(), keystr: W_12_NoNetHood.as_ptr() }, // REST_NONETHOOD
    POLICYDATA { policy: 0x00001000, appstr: W_0_Explorer.as_ptr(), keystr: W_13_NoStartBanner.as_ptr() }, // REST_STARTBANNER
    POLICYDATA { policy: 0x00002000, appstr: W_0_Explorer.as_ptr(), keystr: W_14_RestrictRun.as_ptr() }, // REST_RESTRICTRUN
    POLICYDATA { policy: 0x00004000, appstr: W_0_Explorer.as_ptr(), keystr: W_15_NoPrinterTabs.as_ptr() }, // REST_NOPRINTERTABS
    POLICYDATA { policy: 0x00008000, appstr: W_0_Explorer.as_ptr(), keystr: W_16_NoDeletePrinter.as_ptr() }, // REST_NOPRINTERDELETE
    POLICYDATA { policy: 0x00010000, appstr: W_0_Explorer.as_ptr(), keystr: W_17_NoAddPrinter.as_ptr() }, // REST_NOPRINTERADD
    POLICYDATA { policy: 0x00020000, appstr: W_0_Explorer.as_ptr(), keystr: W_18_NoStartMenuSubFolders.as_ptr() }, // REST_NOSTARTMENUSUBFOLDERS
    POLICYDATA { policy: 0x00040000, appstr: W_0_Explorer.as_ptr(), keystr: W_19_MyDocsOnNet.as_ptr() }, // REST_MYDOCSONNET
    POLICYDATA { policy: 0x00080000, appstr: W_20_WinOldApp.as_ptr(), keystr: W_21_NoRealMode.as_ptr() }, // REST_NOEXITTODOS
    POLICYDATA { policy: 0x00100000, appstr: W_0_Explorer.as_ptr(), keystr: W_22_EnforceShellExtensionSecurity.as_ptr() }, // REST_ENFORCESHELLEXTSECURITY
    POLICYDATA { policy: 0x00200000, appstr: W_0_Explorer.as_ptr(), keystr: W_23_LinkResolveIgnoreLinkInfo.as_ptr() }, // REST_LINKRESOLVEIGNORELINKINFO
    POLICYDATA { policy: 0x00400000, appstr: W_0_Explorer.as_ptr(), keystr: W_24_NoCommonGroups.as_ptr() }, // REST_NOCOMMONGROUPS
    POLICYDATA { policy: 0x00800000, appstr: W_0_Explorer.as_ptr(), keystr: W_25_SeparateProcess.as_ptr() }, // REST_SEPARATEDESKTOPPROCESS
    POLICYDATA { policy: 0x01000000, appstr: W_0_Explorer.as_ptr(), keystr: W_26_NoWebMenu.as_ptr() }, // REST_NOWEB
    POLICYDATA { policy: 0x02000000, appstr: W_0_Explorer.as_ptr(), keystr: W_27_NoTrayContextMenu.as_ptr() }, // REST_NOTRAYCONTEXTMENU
    POLICYDATA { policy: 0x04000000, appstr: W_0_Explorer.as_ptr(), keystr: W_28_NoViewContextMenu.as_ptr() }, // REST_NOVIEWCONTEXTMENU
    POLICYDATA { policy: 0x08000000, appstr: W_0_Explorer.as_ptr(), keystr: W_29_NoNetConnectDisconnect.as_ptr() }, // REST_NONETCONNECTDISCONNECT
    POLICYDATA { policy: 0x10000000, appstr: W_0_Explorer.as_ptr(), keystr: W_30_StartMenuLogoff.as_ptr() }, // REST_STARTMENULOGOFF
    POLICYDATA { policy: 0x20000000, appstr: W_0_Explorer.as_ptr(), keystr: W_31_NoSettingsWizards.as_ptr() }, // REST_NOSETTINGSASSIST
    POLICYDATA { policy: 0x40000001, appstr: W_0_Explorer.as_ptr(), keystr: W_32_NoInternetIcon.as_ptr() }, // REST_NOINTERNETICON
    POLICYDATA { policy: 0x40000002, appstr: W_0_Explorer.as_ptr(), keystr: W_33_NoRecentDocsHistory.as_ptr() }, // REST_NORECENTDOCSHISTORY
    POLICYDATA { policy: 0x40000003, appstr: W_0_Explorer.as_ptr(), keystr: W_34_NoRecentDocsMenu.as_ptr() }, // REST_NORECENTDOCSMENU
    POLICYDATA { policy: 0x40000004, appstr: W_0_Explorer.as_ptr(), keystr: W_35_NoActiveDesktop.as_ptr() }, // REST_NOACTIVEDESKTOP
    POLICYDATA { policy: 0x40000005, appstr: W_0_Explorer.as_ptr(), keystr: W_36_NoActiveDesktopChanges.as_ptr() }, // REST_NOACTIVEDESKTOPCHANGES
    POLICYDATA { policy: 0x40000006, appstr: W_0_Explorer.as_ptr(), keystr: W_37_NoFavoritesMenu.as_ptr() }, // REST_NOFAVORITESMENU
    POLICYDATA { policy: 0x40000007, appstr: W_0_Explorer.as_ptr(), keystr: W_38_ClearRecentDocsOnExit.as_ptr() }, // REST_CLEARRECENTDOCSONEXIT
    POLICYDATA { policy: 0x40000008, appstr: W_0_Explorer.as_ptr(), keystr: W_39_ClassicShell.as_ptr() }, // REST_CLASSICSHELL
    POLICYDATA { policy: 0x40000009, appstr: W_0_Explorer.as_ptr(), keystr: W_40_NoCustomizeWebView.as_ptr() }, // REST_NOCUSTOMIZEWEBVIEW
    POLICYDATA { policy: 0x40000010, appstr: W_41_ActiveDesktop.as_ptr(), keystr: W_42_NoHTMLWallpaper.as_ptr() }, // REST_NOHTMLWALLPAPER
    POLICYDATA { policy: 0x40000011, appstr: W_41_ActiveDesktop.as_ptr(), keystr: W_43_NoChangingWallpaper.as_ptr() }, // REST_NOCHANGINGWALLPAPER
    POLICYDATA { policy: 0x40000012, appstr: W_41_ActiveDesktop.as_ptr(), keystr: W_44_NoComponents.as_ptr() }, // REST_NODESKCOMP
    POLICYDATA { policy: 0x40000013, appstr: W_41_ActiveDesktop.as_ptr(), keystr: W_45_NoAddingComponents.as_ptr() }, // REST_NOADDDESKCOMP
    POLICYDATA { policy: 0x40000014, appstr: W_41_ActiveDesktop.as_ptr(), keystr: W_46_NoDeletingComponents.as_ptr() }, // REST_NODELDESKCOMP
    POLICYDATA { policy: 0x40000015, appstr: W_41_ActiveDesktop.as_ptr(), keystr: W_47_NoClosingComponents.as_ptr() }, // REST_NOCLOSEDESKCOMP
    POLICYDATA { policy: 0x40000016, appstr: W_0_Explorer.as_ptr(), keystr: W_48_NoCloseDragDropBands.as_ptr() }, // REST_NOCLOSE_DRAGDROPBAND
    POLICYDATA { policy: 0x40000017, appstr: W_0_Explorer.as_ptr(), keystr: W_49_NoMovingBands.as_ptr() }, // REST_NOMOVINGBAND
    POLICYDATA { policy: 0x40000018, appstr: W_41_ActiveDesktop.as_ptr(), keystr: W_50_NoEditingComponents.as_ptr() }, // REST_NOEDITDESKCOMP
    POLICYDATA { policy: 0x40000019, appstr: W_0_Explorer.as_ptr(), keystr: W_51_NoResolveSearch.as_ptr() }, // REST_NORESOLVESEARCH
    POLICYDATA { policy: 0x4000001A, appstr: W_0_Explorer.as_ptr(), keystr: W_52_NoResolveTrack.as_ptr() }, // REST_NORESOLVETRACK
    POLICYDATA { policy: 0x4000001B, appstr: W_0_Explorer.as_ptr(), keystr: W_53_ForceCopyACLWithFile.as_ptr() }, // REST_FORCECOPYACLWITHFILE
    POLICYDATA { policy: 0x4000001D, appstr: W_0_Explorer.as_ptr(), keystr: W_54_NoForgetSoftwareUpdate.as_ptr() }, // REST_NOFORGETSOFTWAREUPDATE
    POLICYDATA { policy: 0x4000001E, appstr: W_0_Explorer.as_ptr(), keystr: W_55_NoSetActiveDesktop.as_ptr() }, // REST_NOSETACTIVEDESKTOP
    POLICYDATA { policy: 0x4000001F, appstr: W_0_Explorer.as_ptr(), keystr: W_56_NoWindowsUpdate.as_ptr() }, // REST_NOUPDATEWINDOWS
    POLICYDATA { policy: 0x40000020, appstr: W_0_Explorer.as_ptr(), keystr: W_57_NoChangeStartMenu.as_ptr() }, // REST_NOCHANGESTARMENU
    POLICYDATA { policy: 0x40000021, appstr: W_0_Explorer.as_ptr(), keystr: W_58_NoFolderOptions.as_ptr() }, // REST_NOFOLDEROPTIONS
    POLICYDATA { policy: 0x40000022, appstr: W_0_Explorer.as_ptr(), keystr: W_59_FindComputers.as_ptr() }, // REST_HASFINDCOMPUTERS
    POLICYDATA { policy: 0x40000023, appstr: W_0_Explorer.as_ptr(), keystr: W_60_IntelliMenus.as_ptr() }, // REST_INTELLIMENUS
    POLICYDATA { policy: 0x40000024, appstr: W_0_Explorer.as_ptr(), keystr: W_61_MemCheckBoxInRunDlg.as_ptr() }, // REST_RUNDLGMEMCHECKBOX
    POLICYDATA { policy: 0x40000025, appstr: W_62_Uninstall.as_ptr(), keystr: W_63_ShowPostSetup.as_ptr() }, // REST_ARP_ShowPostSetup
    POLICYDATA { policy: 0x40000026, appstr: W_0_Explorer.as_ptr(), keystr: W_64_NoSyncAll.as_ptr() }, // REST_NOCSC
    POLICYDATA { policy: 0x40000027, appstr: W_0_Explorer.as_ptr(), keystr: W_65_NoControlPanel.as_ptr() }, // REST_NOCONTROLPANEL
    POLICYDATA { policy: 0x40000028, appstr: W_0_Explorer.as_ptr(), keystr: W_66_EnumWorkgroup.as_ptr() }, // REST_ENUMWORKGROUP
    POLICYDATA { policy: 0x40000029, appstr: W_62_Uninstall.as_ptr(), keystr: W_67_NoAddRemovePrograms.as_ptr() }, // REST_ARP_NOARP
    POLICYDATA { policy: 0x4000002A, appstr: W_62_Uninstall.as_ptr(), keystr: W_68_NoRemovePage.as_ptr() }, // REST_ARP_NOREMOVEPAGE
    POLICYDATA { policy: 0x4000002B, appstr: W_62_Uninstall.as_ptr(), keystr: W_69_NoAddPage.as_ptr() }, // REST_ARP_NOADDPAGE
    POLICYDATA { policy: 0x4000002C, appstr: W_62_Uninstall.as_ptr(), keystr: W_70_NoWindowsSetupPage.as_ptr() }, // REST_ARP_NOWINSETUPPAGE
    POLICYDATA { policy: 0x4000002D, appstr: W_0_Explorer.as_ptr(), keystr: W_71_GreyMSIAds.as_ptr() }, // REST_GREYMSIADS
    POLICYDATA { policy: 0x4000002E, appstr: W_0_Explorer.as_ptr(), keystr: W_72_NoChangeMappedDriveLabel.as_ptr() }, // REST_NOCHANGEMAPPEDDRIVELABEL
    POLICYDATA { policy: 0x4000002F, appstr: W_0_Explorer.as_ptr(), keystr: W_73_NoChangeMappedDriveComment.as_ptr() }, // REST_NOCHANGEMAPPEDDRIVECOMMENT
    POLICYDATA { policy: 0x40000030, appstr: W_0_Explorer.as_ptr(), keystr: W_74_MaxRecentDocs.as_ptr() }, // REST_MaxRecentDocs
    POLICYDATA { policy: 0x40000031, appstr: W_0_Explorer.as_ptr(), keystr: W_75_NoNetworkConnections.as_ptr() }, // REST_NONETWORKCONNECTIONS
    POLICYDATA { policy: 0x40000032, appstr: W_0_Explorer.as_ptr(), keystr: W_76_ForceStartMenuLogoff.as_ptr() }, // REST_FORCESTARTMENULOGOFF
    POLICYDATA { policy: 0x40000033, appstr: W_0_Explorer.as_ptr(), keystr: W_77_NoWebView.as_ptr() }, // REST_NOWEBVIEW
    POLICYDATA { policy: 0x40000034, appstr: W_0_Explorer.as_ptr(), keystr: W_78_NoCustomizeThisFolder.as_ptr() }, // REST_NOCUSTOMIZETHISFOLDER
    POLICYDATA { policy: 0x40000035, appstr: W_0_Explorer.as_ptr(), keystr: W_79_NoEncryption.as_ptr() }, // REST_NOENCRYPTION
    POLICYDATA { policy: 0x40000037, appstr: W_0_Explorer.as_ptr(), keystr: W_80_DontShowSuperHidden.as_ptr() }, // REST_DONTSHOWSUPERHIDDEN
    POLICYDATA { policy: 0x40000038, appstr: W_0_Explorer.as_ptr(), keystr: W_81_NoShellSearchButton.as_ptr() }, // REST_NOSHELLSEARCHBUTTON
    POLICYDATA { policy: 0x40000039, appstr: W_0_Explorer.as_ptr(), keystr: W_82_NoHardwareTab.as_ptr() }, // REST_NOHARDWARETAB
    POLICYDATA { policy: 0x4000003C, appstr: W_0_Explorer.as_ptr(), keystr: W_83_NoManageMyComputerVerb.as_ptr() }, // REST_NOMANAGEMYCOMPUTERVERB
    POLICYDATA { policy: 0x4000003E, appstr: W_0_Explorer.as_ptr(), keystr: W_84_DisallowRun.as_ptr() }, // REST_DISALLOWRUN
    POLICYDATA { policy: 0x4000003F, appstr: W_0_Explorer.as_ptr(), keystr: W_85_NoWelcomeScreen.as_ptr() }, // REST_NOWELCOMESCREEN
    POLICYDATA { policy: 0x40000040, appstr: W_0_Explorer.as_ptr(), keystr: W_86_RestrictCpl.as_ptr() }, // REST_RESTRICTCPL
    POLICYDATA { policy: 0x40000041, appstr: W_0_Explorer.as_ptr(), keystr: W_87_DisallowCpl.as_ptr() }, // REST_DISALLOWCPL
    POLICYDATA { policy: 0x40000042, appstr: W_0_Explorer.as_ptr(), keystr: W_88_NoSMBalloonTip.as_ptr() }, // REST_NOSMBALLOONTIP
    POLICYDATA { policy: 0x40000043, appstr: W_0_Explorer.as_ptr(), keystr: W_89_NoSMHelp.as_ptr() }, // REST_NOSMHELP
    POLICYDATA { policy: 0x40000044, appstr: W_0_Explorer.as_ptr(), keystr: W_90_NoWinKeys.as_ptr() }, // REST_NOWINKEYS
    POLICYDATA { policy: 0x40000045, appstr: W_0_Explorer.as_ptr(), keystr: W_91_NoEncryptOnMove.as_ptr() }, // REST_NOENCRYPTONMOVE
    POLICYDATA { policy: 0x40000046, appstr: W_0_Explorer.as_ptr(), keystr: W_92_DisableLocalMachineRun.as_ptr() }, // REST_NOLOCALMACHINERUN
    POLICYDATA { policy: 0x40000047, appstr: W_0_Explorer.as_ptr(), keystr: W_93_DisableCurrentUserRun.as_ptr() }, // REST_NOCURRENTUSERRUN
    POLICYDATA { policy: 0x40000048, appstr: W_0_Explorer.as_ptr(), keystr: W_94_DisableLocalMachineRunOnce.as_ptr() }, // REST_NOLOCALMACHINERUNONCE
    POLICYDATA { policy: 0x40000049, appstr: W_0_Explorer.as_ptr(), keystr: W_95_DisableCurrentUserRunOnce.as_ptr() }, // REST_NOCURRENTUSERRUNONCE
    POLICYDATA { policy: 0x4000004A, appstr: W_0_Explorer.as_ptr(), keystr: W_96_ForceActiveDesktopOn.as_ptr() }, // REST_FORCEACTIVEDESKTOPON
    POLICYDATA { policy: 0x4000004C, appstr: W_0_Explorer.as_ptr(), keystr: W_97_NoViewOnDrive.as_ptr() }, // REST_NOVIEWONDRIVE
    POLICYDATA { policy: 0x4000004D, appstr: W_0_Explorer.as_ptr(), keystr: W_98_NoNetCrawling.as_ptr() }, // REST_NONETCRAWL
    POLICYDATA { policy: 0x4000004E, appstr: W_0_Explorer.as_ptr(), keystr: W_99_NoSharedDocuments.as_ptr() }, // REST_NOSHAREDDOCUMENTS
    POLICYDATA { policy: 0x4000004F, appstr: W_0_Explorer.as_ptr(), keystr: W_100_NoSMMyDocs.as_ptr() }, // REST_NOSMMYDOCS
    POLICYDATA { policy: 0x40000050, appstr: W_0_Explorer.as_ptr(), keystr: W_101_NoSMMyPictures.as_ptr() }, // REST_NOSMMYPICS
    POLICYDATA { policy: 0x40000051, appstr: W_0_Explorer.as_ptr(), keystr: W_102_RecycleBinDrives.as_ptr() }, // REST_ALLOWBITBUCKDRIVES
    POLICYDATA { policy: 0x40000052, appstr: W_0_Explorer.as_ptr(), keystr: W_103_NoneLegacyShellMode.as_ptr() }, // REST_NONLEGACYSHELLMODE
    POLICYDATA { policy: 0x40000053, appstr: W_0_Explorer.as_ptr(), keystr: W_104_NoControlPanelBarricade.as_ptr() }, // REST_NOCONTROLPANELBARRICADE
    POLICYDATA { policy: 0x40000055, appstr: W_0_Explorer.as_ptr(), keystr: W_105_NoAutoTrayNotify.as_ptr() }, // REST_NOAUTOTRAYNOTIFY
    POLICYDATA { policy: 0x40000056, appstr: W_0_Explorer.as_ptr(), keystr: W_106_NoTaskGrouping.as_ptr() }, // REST_NOTASKGROUPING
    POLICYDATA { policy: 0x40000057, appstr: W_0_Explorer.as_ptr(), keystr: W_107_NoCDBurning.as_ptr() }, // REST_NOCDBURNING
    POLICYDATA { policy: 0x40000058, appstr: W_0_Explorer.as_ptr(), keystr: W_108_NoPropertiesMyComputer.as_ptr() }, // REST_MYCOMPNOPROP
    POLICYDATA { policy: 0x40000059, appstr: W_0_Explorer.as_ptr(), keystr: W_109_NoPropertiesMyDocuments.as_ptr() }, // REST_MYDOCSNOPROP
    POLICYDATA { policy: 0x4000005A, appstr: W_0_Explorer.as_ptr(), keystr: W_110_NoSimpleStartMenu.as_ptr() }, // REST_NOSTARTPANEL
    POLICYDATA { policy: 0x4000005B, appstr: W_111_System.as_ptr(), keystr: W_112_NoDispAppearancePage.as_ptr() }, // REST_NODISPLAYAPPEARANCEPAGE
    POLICYDATA { policy: 0x4000005C, appstr: W_0_Explorer.as_ptr(), keystr: W_113_NoThemesTab.as_ptr() }, // REST_NOTHEMESTAB
    POLICYDATA { policy: 0x4000005D, appstr: W_111_System.as_ptr(), keystr: W_114_NoVisualStyleChoice.as_ptr() }, // REST_NOVISUALSTYLECHOICE
    POLICYDATA { policy: 0x4000005E, appstr: W_111_System.as_ptr(), keystr: W_115_NoSizeChoice.as_ptr() }, // REST_NOSIZECHOICE
    POLICYDATA { policy: 0x4000005F, appstr: W_111_System.as_ptr(), keystr: W_116_NoColorChoice.as_ptr() }, // REST_NOCOLORCHOICE
    POLICYDATA { policy: 0x40000060, appstr: W_111_System.as_ptr(), keystr: W_117_SetVisualStyle.as_ptr() }, // REST_SETVISUALSTYLE
    POLICYDATA { policy: 0x40000061, appstr: W_0_Explorer.as_ptr(), keystr: W_118_StartRunNoHOMEPATH.as_ptr() }, // REST_STARTRUNNOHOMEPATH
    POLICYDATA { policy: 0x40000062, appstr: W_0_Explorer.as_ptr(), keystr: W_119_NoUserNameInStartMenu.as_ptr() }, // REST_NOUSERNAMEINSTARTPANEL
    POLICYDATA { policy: 0x40000063, appstr: W_120_NonEnum.as_ptr(), keystr: W_121__20D04FE0_3AEA_1069_A2D8_08002B30309D_.as_ptr() }, // REST_NOMYCOMPUTERICON
    POLICYDATA { policy: 0x40000064, appstr: W_0_Explorer.as_ptr(), keystr: W_122_NoStartMenuNetworkPlaces.as_ptr() }, // REST_NOSMNETWORKPLACES
    POLICYDATA { policy: 0x40000065, appstr: W_0_Explorer.as_ptr(), keystr: W_123_NoStartMenuPinnedList.as_ptr() }, // REST_NOSMPINNEDLIST
    POLICYDATA { policy: 0x40000066, appstr: W_0_Explorer.as_ptr(), keystr: W_124_NoStartMenuMyMusic.as_ptr() }, // REST_NOSMMYMUSIC
    POLICYDATA { policy: 0x40000067, appstr: W_0_Explorer.as_ptr(), keystr: W_125_NoStartMenuEjectPC.as_ptr() }, // REST_NOSMEJECTPC
    POLICYDATA { policy: 0x40000068, appstr: W_0_Explorer.as_ptr(), keystr: W_126_NoStartMenuMorePrograms.as_ptr() }, // REST_NOSMMOREPROGRAMS
    POLICYDATA { policy: 0x40000069, appstr: W_0_Explorer.as_ptr(), keystr: W_127_NoStartMenuMFUprogramsList.as_ptr() }, // REST_NOSMMFUPROGRAMS
    POLICYDATA { policy: 0x4000006A, appstr: W_0_Explorer.as_ptr(), keystr: W_128_NoTrayItemsDisplay.as_ptr() }, // REST_NOTRAYITEMSDISPLAY
    POLICYDATA { policy: 0x4000006B, appstr: W_0_Explorer.as_ptr(), keystr: W_129_NoToolbarsOnTaskbar.as_ptr() }, // REST_NOTOOLBARSONTASKBAR
    POLICYDATA { policy: 0x4000006F, appstr: W_0_Explorer.as_ptr(), keystr: W_130_NoSMConfigurePrograms.as_ptr() }, // REST_NOSMCONFIGUREPROGRAMS
    POLICYDATA { policy: 0x40000070, appstr: W_0_Explorer.as_ptr(), keystr: W_131_HideClock.as_ptr() }, // REST_HIDECLOCK
    POLICYDATA { policy: 0x40000071, appstr: W_0_Explorer.as_ptr(), keystr: W_132_NoLowDiskSpaceChecks.as_ptr() }, // REST_NOLOWDISKSPACECHECKS
    POLICYDATA { policy: 0x40000074, appstr: W_0_Explorer.as_ptr(), keystr: W_133_NoRecycleFiles.as_ptr() }, // REST_BITBUCKNUKEONDELETE
    POLICYDATA { policy: 0x40000075, appstr: W_0_Explorer.as_ptr(), keystr: W_134_ConfirmFileDelete.as_ptr() }, // REST_BITBUCKCONFIRMDELETE
    POLICYDATA { policy: 0x40000076, appstr: W_0_Explorer.as_ptr(), keystr: W_135_NoPropertiesRecycleBin.as_ptr() }, // REST_BITBUCKNOPROP
    POLICYDATA { policy: 0x40000077, appstr: W_111_System.as_ptr(), keystr: W_136_NoDispBackgroundPage.as_ptr() }, // REST_NODISPBACKGROUND
    POLICYDATA { policy: 0x40000078, appstr: W_111_System.as_ptr(), keystr: W_137_NoDispScrSavPage.as_ptr() }, // REST_NODISPSCREENSAVEPG
    POLICYDATA { policy: 0x40000079, appstr: W_111_System.as_ptr(), keystr: W_138_NoDispSettingsPage.as_ptr() }, // REST_NODISPSETTINGSPG
    POLICYDATA { policy: 0x4000007A, appstr: W_111_System.as_ptr(), keystr: W_139_NoScreenSavePreview.as_ptr() }, // REST_NODISPSCREENSAVEPREVIEW
    POLICYDATA { policy: 0x4000007B, appstr: W_111_System.as_ptr(), keystr: W_140_NoDispCPL.as_ptr() }, // REST_NODISPLAYCPL
    POLICYDATA { policy: 0x4000007C, appstr: W_0_Explorer.as_ptr(), keystr: W_141_HideRunAsVerb.as_ptr() }, // REST_HIDERUNASVERB
    POLICYDATA { policy: 0x4000007D, appstr: W_0_Explorer.as_ptr(), keystr: W_142_NoThumbnailCache.as_ptr() }, // REST_NOTHUMBNAILCACHE
    POLICYDATA { policy: 0x4000007E, appstr: W_0_Explorer.as_ptr(), keystr: W_143_NoStrCmpLogical.as_ptr() }, // REST_NOSTRCMPLOGICAL
    POLICYDATA { policy: 0x4000007F, appstr: W_0_Explorer.as_ptr(), keystr: W_144_NoPublishingWizard.as_ptr() }, // REST_NOPUBLISHWIZARD
    POLICYDATA { policy: 0x40000080, appstr: W_0_Explorer.as_ptr(), keystr: W_145_NoOnlinePrintsWizard.as_ptr() }, // REST_NOONLINEPRINTSWIZARD
    POLICYDATA { policy: 0x40000081, appstr: W_0_Explorer.as_ptr(), keystr: W_146_NoWebServices.as_ptr() }, // REST_NOWEBSERVICES
    POLICYDATA { policy: 0x40000082, appstr: W_0_Explorer.as_ptr(), keystr: W_147_AllowUnhashedWebView.as_ptr() }, // REST_ALLOWUNHASHEDWEBVIEW
    POLICYDATA { policy: 0x40000083, appstr: W_0_Explorer.as_ptr(), keystr: W_148_AllowLegacyWebView.as_ptr() }, // REST_ALLOWLEGACYWEBVIEW
    POLICYDATA { policy: 0x40000084, appstr: W_0_Explorer.as_ptr(), keystr: W_149_RevertWebViewSecurity.as_ptr() }, // REST_REVERTWEBVIEWSECURITY
    POLICYDATA { policy: 0x40000086, appstr: W_0_Explorer.as_ptr(), keystr: W_150_InheritConsoleHandles.as_ptr() }, // REST_INHERITCONSOLEHANDLES
    POLICYDATA { policy: 0x40000089, appstr: W_0_Explorer.as_ptr(), keystr: W_151_NoRemoteRecursiveEvents.as_ptr() }, // REST_NOREMOTERECURSIVEEVENTS
    POLICYDATA { policy: 0x40000091, appstr: W_0_Explorer.as_ptr(), keystr: W_152_NoRemoteChangeNotify.as_ptr() }, // REST_NOREMOTECHANGENOTIFY
    POLICYDATA { policy: 0x40000093, appstr: W_0_Explorer.as_ptr(), keystr: W_153_NoEnumEntireNetwork.as_ptr() }, // REST_NOENUMENTIRENETWORK
    POLICYDATA { policy: 0x40000095, appstr: W_0_Explorer.as_ptr(), keystr: W_154_NoInternetOpenWith.as_ptr() }, // REST_NOINTERNETOPENWITH
    POLICYDATA { policy: 0x4000009B, appstr: W_0_Explorer.as_ptr(), keystr: W_155_DontRetryBadNetName.as_ptr() }, // REST_DONTRETRYBADNETNAME
    POLICYDATA { policy: 0x4000009C, appstr: W_0_Explorer.as_ptr(), keystr: W_156_AllowFileCLSIDJunctions.as_ptr() }, // REST_ALLOWFILECLSIDJUNCTIONS
    POLICYDATA { policy: 0x4000009D, appstr: W_0_Explorer.as_ptr(), keystr: W_157_NoUPnPInstallTask.as_ptr() }, // REST_NOUPNPINSTALL
    POLICYDATA { policy: 0x400000AC, appstr: W_62_Uninstall.as_ptr(), keystr: W_158_DontGroupPatches.as_ptr() }, // REST_ARP_DONTGROUPPATCHES
    POLICYDATA { policy: 0x400000AD, appstr: W_62_Uninstall.as_ptr(), keystr: W_159_NoChooseProgramsPage.as_ptr() }, // REST_ARP_NOCHOOSEPROGRAMSPAGE
    POLICYDATA { policy: 0x41000001, appstr: W_0_Explorer.as_ptr(), keystr: W_160_NoDisconnect.as_ptr() }, // REST_NODISCONNECT
    POLICYDATA { policy: 0x41000002, appstr: W_0_Explorer.as_ptr(), keystr: W_161_NoNTSecurity.as_ptr() }, // REST_NOSECURITY
    POLICYDATA { policy: 0x41000003, appstr: W_0_Explorer.as_ptr(), keystr: W_162_NoFileAssociate.as_ptr() }, // REST_NOFILEASSOCIATE
    POLICYDATA { policy: 0x41000004, appstr: W_0_Explorer.as_ptr(), keystr: W_163_ToggleCommentPosition.as_ptr() }, // REST_ALLOWCOMMENTTOGGLE
    POLICYDATA { policy: 0x00060001, appstr: W_0_Explorer.as_ptr(), keystr: W_164_EnableShellExecuteHooks.as_ptr() }, // REST_SH32_ENABLESHELLEXECUTEHOOKS
    POLICYDATA { policy: 0, appstr: core::ptr::null(), keystr: core::ptr::null() }, // terminator { 0, NULL, NULL }
]);

// ── exported restriction-related data globals (C-linkage names) ───────────────────────────
// HANDLE g_hRestGlobalCounter = NULL;
#[no_mangle]
pub static mut g_hRestGlobalCounter: HANDLE = core::ptr::null_mut();
// LONG g_nRestCountValue = -1;
#[no_mangle]
pub static mut g_nRestCountValue: LONG = -1;
// DWORD g_RestValues[_countof(s_PolicyTable)] = { 0 };
#[no_mangle]
pub static mut g_RestValues: [DWORD; POLICY_TABLE_LEN] = [0; POLICY_TABLE_LEN];

// int (WINAPI* SHELL_StrCmpLogical)(PCWSTR, PCWSTR) = SHELL_StrCmpLogicalInit;
#[no_mangle]
pub static mut SHELL_StrCmpLogical: StrCmpLogicalFn = SHELL_StrCmpLogicalInit;

// ── StrCmpLogical indirection (#ifdef __REACTOS__) ────────────────────────────────────────
#[no_mangle]
pub unsafe extern "system" fn SHELL_StrCmpLogicalInit(s1: PCWSTR, s2: PCWSTR) -> INT {
    // SHELL_StrCmpLogical = SHRestricted(REST_NOSTRCMPLOGICAL) ? StrCmpIW : StrCmpLogicalW;
    SHELL_StrCmpLogical = if SHRestricted(REST_NOSTRCMPLOGICAL) != 0 {
        StrCmpIW
    } else {
        StrCmpLogicalW
    };
    (SHELL_StrCmpLogical)(s1, s2)
}

// static void SH32_RestrictionsChanged()
unsafe fn SH32_RestrictionsChanged() {
    SHELL_StrCmpLogical = SHELL_StrCmpLogicalInit;
}

// ── global-counter caching ────────────────────────────────────────────────────────────────
/// SHELL_GetCachedGlobalCounter — EXTERN_C HANDLE (cdecl, no @N decoration).
#[no_mangle]
pub unsafe extern "C" fn SHELL_GetCachedGlobalCounter(
    phGlobalCounter: *mut HANDLE,
    rguid: REFGUID,
) -> HANDLE {
    if !(*phGlobalCounter).is_null() {
        return *phGlobalCounter;
    }
    let hGlobalCounter = SHGlobalCounterCreate(rguid);
    // InterlockedCompareExchangePointer(phGlobalCounter, hGlobalCounter, NULL):
    //   atomically install hGlobalCounter if *phGlobalCounter == NULL; returns the prior value.
    //   prior != NULL  => another thread won  => close our handle.
    let slot = &*(phGlobalCounter as *const AtomicPtr<core::ffi::c_void>);
    if slot
        .compare_exchange(
            core::ptr::null_mut(),
            hGlobalCounter,
            Ordering::SeqCst,
            Ordering::SeqCst,
        )
        .is_err()
    {
        CloseHandle(hGlobalCounter);
    }
    *phGlobalCounter
}

// static HANDLE SHELL_GetRestrictionsCounter(VOID)
unsafe fn SHELL_GetRestrictionsCounter() -> HANDLE {
    SHELL_GetCachedGlobalCounter(
        core::ptr::addr_of_mut!(g_hRestGlobalCounter),
        core::ptr::addr_of!(GUID_RESTRICTIONS),
    )
}

// static BOOL SHELL_QueryRestrictionsChanged(VOID)
unsafe fn SHELL_QueryRestrictionsChanged() -> BOOL {
    let value: LONG = SHGlobalCounterGetValue(SHELL_GetRestrictionsCounter());
    if g_nRestCountValue == value {
        return FALSE;
    }
    g_nRestCountValue = value;
    TRUE
}

// ── exported policy query API ─────────────────────────────────────────────────────────────
/// SHRestricted [SHELL32.100] — WINAPI (`_SHRestricted@4`).
#[no_mangle]
pub unsafe extern "system" fn SHRestricted(rest: DWORD) -> DWORD {
    // If restrictions from registry have changed, reset all cached values to SHELL_NO_POLICY.
    if SHELL_QueryRestrictionsChanged() != FALSE {
        // FillMemory(&g_RestValues, sizeof(g_RestValues), 0xFF);
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(g_RestValues) as *mut DWORD,
            0xFF,
            POLICY_TABLE_LEN,
        );
        SH32_RestrictionsChanged();
    }

    SHRestrictionLookup(
        rest,
        core::ptr::null(),
        core::ptr::addr_of!(S_POLICY_TABLE.0) as *const POLICYDATA,
        core::ptr::addr_of_mut!(g_RestValues) as LPDWORD,
    )
}

/// SH32_InternalRestricted — cdecl (`_SH32_InternalRestricted`, no @N).
#[no_mangle]
pub unsafe extern "C" fn SH32_InternalRestricted(rest: DWORD) -> DWORD {
    /* assert((rest & 0x00060000) == 0x00060000) */
    SHRestricted(rest)
}

// L"Policy"
static WSZ_POLICY: [WCHAR; 7] = [0x0050, 0x006F, 0x006C, 0x0069, 0x0063, 0x0079, 0x0000];
// L"Software\\Microsoft\\Windows\\CurrentVersion\\Policies"
static WSZ_POLICIES_KEY: [WCHAR; 51] = [
    0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063,
    0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F,
    0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065,
    0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0050, 0x006F, 0x006C, 0x0069, 0x0063, 0x0069,
    0x0065, 0x0073, 0x0000,
];

/// SHSettingsChanged [SHELL32.244] — WINAPI (`_SHSettingsChanged@8`).
#[no_mangle]
pub unsafe extern "system" fn SHSettingsChanged(unused: LPCVOID, pszKey: LPCWSTR) -> BOOL {
    let _ = unused;
    if !pszKey.is_null()
        && lstrcmpiW(WSZ_POLICY.as_ptr(), pszKey) != 0
        && lstrcmpiW(WSZ_POLICIES_KEY.as_ptr(), pszKey) != 0
    {
        return FALSE;
    }

    SHGlobalCounterIncrement(SHELL_GetRestrictionsCounter()) as BOOL
}

// ADDED-TO-TYPES?: GUID + REFGUID (guiddef.h), PCWSTR (winnt.h), POLICYDATA
// (reactos/shlwapi_undoc.h) — these recur across the shell32 wave and likely belong in
// crate::types. StrCmpLogicalFn / PolicyTable are shpolicy-specific and should stay local.
