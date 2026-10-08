//! startmnucust.cpp — the "Customize Classic Start Menu" dialog. A single
//! DLGPROC (`CustomizeClassicProc`) hosting a TreeView of checkable
//! advanced-start-menu options (each backed by a registry `Advanced` value via
//! GetAdvancedBool/SetAdvancedDword) plus Add/Remove/Advanced/Clear buttons.
//! Entry points `ShowCustomizeClassic()` (opens the modal dialog) and
//! `ClearRecentAndMru()` are called from the already-Rust startmnu.rs; precomp.h
//! already declares BOTH as `extern "C"`, so `#[no_mangle] extern "C"` replaces
//! the C++ definitions with no header edit. DIALOG code — runs only on user
//! action, NOT on the boot path (boot-safe: must link + not break the build).
//!
//! Modeling (mirrors trayprop.rs):
//! - The DLGPROC is `extern "system"` (== CALLBACK: stdcall on i386, cdecl on
//!   LLP64); `DialogBoxW(...)` = `DialogBoxParamW(..., 0)` (explorer_sys::externs).
//! - CUSTOM_ENTRY's `FN_CUSTOM_GET`/`FN_CUSTOM_SET` are CALLBACK fn-pointers →
//!   `extern "system"` fn types (definitions + type agree, so ABI-correct on all
//!   arches). The static table is rebuilt by a plain `fn` each use — behaviorally
//!   identical to the C++ `static const CUSTOM_ENTRY[]` (same fn pointers + wide
//!   literals) and sidesteps the "raw pointers aren't Sync" static hazard.
//! - TreeView_* are commctrl.h SendMessage macros → file-local inline helpers.
//! - The TVITEMW / TVITEMEXW / TVINSERTSTRUCTW structs + TVM_*/TVI_*/TVIF_*/
//!   TVIS_*/TVGN_*/TVS_CHECKBOXES constants are NOT in the winsrv/explorer-sys
//!   substrate, so they are declared FILE-LOCAL (like syspager's private surface).
//!   See the SUBSTRATE MANIFEST accompanying this file.

#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::{c_int, c_void};
use core::ptr::{null, null_mut};

use winsrv_sys::consts::{FAILED, SUCCEEDED};
use winsrv_sys::w;
// Types + user32/kernel32/advapi32 fns + WM_*/GWL_STYLE/KEY_WRITE/ERROR_SUCCESS/
// SW_SHOWNORMAL/MAX_PATH/SMTO_ABORTIFHUNG/WM_SETTINGCHANGE/WM_NEXTDLGCTL consts,
// LoadStringW/GetModuleHandleW/GetDlgItem/EnableWindow/GetFocus/EndDialog/
// SendMessageW/SendMessageTimeoutW/FindFirstFileW/FindClose/RegOpenKeyExW/
// RegCloseKey/GetWindowLongW/SetWindowLongW/WIN32_FIND_DATAW.
use winsrv_sys::winsrv::*;

use explorer_sys::externs::{
    SHGetFolderPathW, SHRestricted, StringCbCatW, HKEY_CURRENT_USER, INVALID_HANDLE_VALUE,
    REST_NOCONTROLPANEL, REST_NOFAVORITESMENU, REST_NONETWORKCONNECTIONS, REST_NORUN,
    REST_NOSETFOLDERS, REST_NOSMMYDOCS, REST_NOSMMYPICS, REST_STARTMENULOGOFF,
};
use explorer_sys::server::ShellExecuteW;
use explorer_sys::MAKEINTRESOURCEW;

use crate::explorer::settings::g_TaskbarSettings;
use crate::explorer::util::{GetAdvancedBool, SetAdvancedDword};

// ===========================================================================
// SUBSTRATE not in winsrv-sys / explorer-sys — declared FILE-LOCAL so this
// staging module is self-contained. explorer.exe already links comctl32/user32/
// shlwapi/shell32, so these resolve at link time.
// ===========================================================================

// ---- shlwapi.dll / shell32.dll (stdcall) gap-fill ------------------------
extern "system" {
    /// shlwapi.h `SHDeleteKeyW` — recursively delete a registry subkey.
    fn SHDeleteKeyW(hkey: HKEY, pszSubKey: LPCWSTR) -> DWORD;
    /// shlwapi.h `PathAppendW` — append `pszMore` to `pszPath` with a separator.
    fn PathAppendW(pszPath: LPWSTR, pszMore: LPCWSTR) -> BOOL;
    /// shlobj.h `SHAddToRecentDocs` — add/clear the Recent Documents list.
    fn SHAddToRecentDocs(uFlags: UINT, pv: *const c_void);
}
/// user32.dll `wsprintfW` — __cdecl variadic (NOT stdcall).
extern "C" {
    fn wsprintfW(lpOut: LPWSTR, lpFmt: LPCWSTR, ...) -> c_int;
}

// GetWindowLongPtrW / SetWindowLongPtrW are C macros over the *LongW variants on
// i386 (32-bit slot) and real imports on LLP64 — arch-split exactly like
// syspager.rs. Only used here to OR TVS_CHECKBOXES into GWL_STYLE (fits 32 bits).
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
}

// ---- shlobj.h CSIDL / SHARD --------------------------------------------
const CSIDL_PROGRAMS: c_int = 0x0002;
const CSIDL_RECENT: c_int = 0x0008;
const CSIDL_STARTMENU: c_int = 0x000b;
const CSIDL_FLAG_CREATE: c_int = 0x8000;
/// shlobj.h `SHARD_PIDL`.
const SHARD_PIDL: UINT = 0x0000_0001;

// ---- winuser.h -----------------------------------------------------------
/// winuser.h `HWND_BROADCAST` == (HWND)0xFFFF.
const HWND_BROADCAST: HWND = 0xFFFF_usize as HWND;

// ---- base/shell/explorer/resource.h dialog / control / string IDs --------
const IDD_CLASSICSTART_CUSTOMIZE: c_int = 9;

const IDC_CLASSICSTART_SETTINGS: c_int = 1135;
const IDC_CLASSICSTART_CLEAR: c_int = 1125;
const IDC_CLASSICSTART_ADD: c_int = 1126;
const IDC_CLASSICSTART_REMOVE: c_int = 1127;
const IDC_CLASSICSTART_ADVANCED: c_int = 1128;

// shell32.dll string-table IDs (loaded from shell32, not explorer.exe).
const IDS_ADVANCED_DISPLAY_FAVORITES: LPARAM = 30466;
const IDS_ADVANCED_DISPLAY_LOG_OFF: LPARAM = 30467;
const IDS_ADVANCED_EXPAND_CONTROL_PANEL: LPARAM = 30468;
const IDS_ADVANCED_EXPAND_MY_DOCUMENTS: LPARAM = 30469;
const IDS_ADVANCED_EXPAND_PRINTERS: LPARAM = 30470;
const IDS_ADVANCED_EXPAND_MY_PICTURES: LPARAM = 30472;
const IDS_ADVANCED_EXPAND_NET_CONNECTIONS: LPARAM = 30473;
const IDS_ADVANCED_DISPLAY_RUN: LPARAM = 30474;
const IDS_ADVANCED_DISPLAY_ADMINTOOLS: LPARAM = 30476;
const IDS_ADVANCED_SMALL_START_MENU: LPARAM = 30477;

// ---- commctrl.h TreeView substrate --------------------------------------
type HTREEITEM = *mut c_void;

const TV_FIRST: UINT = 0x1100;
const TVM_INSERTITEMW: UINT = TV_FIRST + 50;
const TVM_GETNEXTITEM: UINT = TV_FIRST + 10;
const TVM_GETITEMW: UINT = TV_FIRST + 62;

const TVGN_ROOT: WPARAM = 0x0;
const TVGN_NEXTVISIBLE: WPARAM = 0x6;

const TVIF_TEXT: UINT = 0x1;
const TVIF_PARAM: UINT = 0x4;
const TVIF_STATE: UINT = 0x8;

const TVIS_STATEIMAGEMASK: UINT = 0xF000;

const TVS_CHECKBOXES: LONG = 0x0100;

/// `TVI_ROOT` == (HTREEITEM)(ULONG_PTR)-0x10000.
const TVI_ROOT: HTREEITEM = (-0x10000_isize) as usize as HTREEITEM;
/// `TVI_LAST` == (HTREEITEM)(ULONG_PTR)-0xfffe.
const TVI_LAST: HTREEITEM = (-0xfffe_isize) as usize as HTREEITEM;

/// TreeView checkbox state indexes (use with INDEXTOSTATEIMAGEMASK).
const I_UNCHECKED: UINT = 1;
const I_CHECKED: UINT = 2;

/// commctrl.h `INDEXTOSTATEIMAGEMASK(i)` == `(i) << 12`.
#[inline]
const fn INDEXTOSTATEIMAGEMASK(i: UINT) -> UINT {
    i << 12
}

/// commctrl.h `tagTVITEMW` (exact layout).
#[repr(C)]
#[derive(Clone, Copy)]
struct TVITEMW {
    mask: UINT,
    hItem: HTREEITEM,
    state: UINT,
    stateMask: UINT,
    pszText: LPWSTR,
    cchTextMax: c_int,
    iImage: c_int,
    iSelectedImage: c_int,
    cChildren: c_int,
    lParam: LPARAM,
}

/// commctrl.h `tagTVITEMEXW` (the larger union arm — sizes the insert struct).
#[repr(C)]
#[derive(Clone, Copy)]
struct TVITEMEXW {
    mask: UINT,
    hItem: HTREEITEM,
    state: UINT,
    stateMask: UINT,
    pszText: LPWSTR,
    cchTextMax: c_int,
    iImage: c_int,
    iSelectedImage: c_int,
    cChildren: c_int,
    lParam: LPARAM,
    iIntegral: c_int,
    uStateEx: UINT,
    hwnd: HWND,
    iExpandedImage: c_int,
}

/// commctrl.h `tagTVINSERTSTRUCTW`. The `union { TVITEMEXW itemex; TV_ITEMW item }`
/// is modeled as its largest arm (`TVITEMEXW`); the TVITEMW prefix (mask..lParam)
/// aliases exactly, which is all the C++ touches.
#[repr(C)]
struct TVINSERTSTRUCTW {
    hParent: HTREEITEM,
    hInsertAfter: HTREEITEM,
    item: TVITEMEXW,
}

// TreeView_* commctrl.h macros → SendMessageW helpers.
#[inline]
unsafe fn TreeView_InsertItem(hwnd: HWND, lpis: *const TVINSERTSTRUCTW) -> HTREEITEM {
    SendMessageW(hwnd, TVM_INSERTITEMW, 0, lpis as LPARAM) as HTREEITEM
}
#[inline]
unsafe fn TreeView_GetNextItem(hwnd: HWND, hitem: HTREEITEM, code: WPARAM) -> HTREEITEM {
    SendMessageW(hwnd, TVM_GETNEXTITEM, code, hitem as LPARAM) as HTREEITEM
}
#[inline]
unsafe fn TreeView_GetRoot(hwnd: HWND) -> HTREEITEM {
    TreeView_GetNextItem(hwnd, null_mut(), TVGN_ROOT)
}
#[inline]
unsafe fn TreeView_GetNextVisible(hwnd: HWND, hitem: HTREEITEM) -> HTREEITEM {
    TreeView_GetNextItem(hwnd, hitem, TVGN_NEXTVISIBLE)
}
#[inline]
unsafe fn TreeView_GetItem(hwnd: HWND, pitem: *mut TVITEMW) -> BOOL {
    SendMessageW(hwnd, TVM_GETITEMW, 0, pitem as LPARAM) as BOOL
}

// ===========================================================================
// Add / Remove / Advanced start-menu-item handlers
// ===========================================================================

// TODO(upstream): Windows Explorer calls NewLinkHere / ConfigStartMenu directly.
unsafe fn OnAddStartMenuItems(hDlg: HWND) {
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if SUCCEEDED(SHGetFolderPathW(
        null_mut(),
        CSIDL_PROGRAMS,
        null_mut(),
        0,
        szPath.as_mut_ptr(),
    )) {
        let mut szCommand: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        // szCommand = L"appwiz.cpl,NewLinkHere "
        let prefix = w!("appwiz.cpl,NewLinkHere ");
        let plen = prefix.len() - 1; // exclude terminating NUL
        szCommand[..plen].copy_from_slice(&prefix[..plen]);

        // StringCchCatW(szCommand, _countof(szCommand), szPath) — cb = cch * 2.
        if SUCCEEDED(StringCbCatW(
            szCommand.as_mut_ptr(),
            (MAX_PATH as usize) * 2,
            szPath.as_ptr(),
        )) {
            ShellExecuteW(
                hDlg,
                w!("open").as_ptr(),
                w!("rundll32.exe").as_ptr(),
                szCommand.as_ptr(),
                null(),
                SW_SHOWNORMAL as c_int,
            );
        }
    }
}

unsafe fn OnRemoveStartmenuItems(hDlg: HWND) {
    ShellExecuteW(
        hDlg,
        w!("open").as_ptr(),
        w!("rundll32.exe").as_ptr(),
        w!("appwiz.cpl,ConfigStartMenu").as_ptr(),
        null(),
        SW_SHOWNORMAL as c_int,
    );
}

unsafe fn OnAdvancedStartMenuItems() {
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if SUCCEEDED(SHGetFolderPathW(
        null_mut(),
        CSIDL_STARTMENU,
        null_mut(),
        0,
        szPath.as_mut_ptr(),
    )) {
        ShellExecuteW(
            null_mut(),
            w!("explore").as_ptr(),
            szPath.as_ptr(),
            null(),
            null(),
            SW_SHOWNORMAL as c_int,
        );
    }
}

// ===========================================================================
// Recent / MRU clearing
// ===========================================================================

unsafe fn RecentHasShortcut(hwnd: HWND) -> BOOL {
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if FAILED(SHGetFolderPathW(
        hwnd,
        CSIDL_RECENT | CSIDL_FLAG_CREATE,
        null_mut(),
        0,
        szPath.as_mut_ptr(),
    )) {
        return 0;
    }

    // Find shortcut files in Recent.
    let mut find: WIN32_FIND_DATAW = core::mem::zeroed();
    PathAppendW(szPath.as_mut_ptr(), w!("*.lnk").as_ptr());
    let hFind = FindFirstFileW(szPath.as_ptr(), &mut find);
    if hFind == INVALID_HANDLE_VALUE {
        return 0;
    }

    FindClose(hFind);
    1
}

/// `g_MruKeys` (file-static). Keys not starting with 'S'[oftware] are relative
/// to "Software\Microsoft\Windows\CurrentVersion".
fn g_mru_keys() -> [LPCWSTR; 4] {
    [
        w!("Software\\Microsoft\\Internet Explorer\\TypedURLs").as_ptr(),
        w!("Explorer\\RunMRU").as_ptr(),
        w!("Explorer\\Comdlg32\\OpenSaveMRU").as_ptr(),
        w!("Explorer\\Comdlg32\\LastVisitedMRU").as_ptr(),
    ]
}

unsafe fn HandleMruData(Delete: BOOL) -> BOOL {
    let keys = g_mru_keys();
    for &key in keys.iter() {
        let mut szKey: [u16; 200] = [0; 200];
        let mut pszKey: LPCWSTR = key;
        if *pszKey != b'S' as u16 {
            // Keys not starting with S[oftware] are assumed relative to "SMWCV".
            wsprintfW(
                szKey.as_mut_ptr(),
                w!("%s\\%s").as_ptr(),
                w!("Software\\Microsoft\\Windows\\CurrentVersion").as_ptr(),
                pszKey,
            );
            pszKey = szKey.as_ptr();
        }

        let mut hKey: HKEY = null_mut();
        if Delete != 0 {
            SHDeleteKeyW(HKEY_CURRENT_USER, pszKey);
        } else if RegOpenKeyExW(HKEY_CURRENT_USER, pszKey, 0, KEY_WRITE, &mut hKey)
            == ERROR_SUCCESS as i32
        {
            RegCloseKey(hKey);
            return 1;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn ClearRecentAndMru() {
    SHAddToRecentDocs(SHARD_PIDL, null());
    HandleMruData(1);
}

unsafe fn InitializeClearButton(hwnd: HWND) {
    let hWndClear = GetDlgItem(hwnd, IDC_CLASSICSTART_CLEAR);
    let bHasData: BOOL = (RecentHasShortcut(hwnd) != 0 || HandleMruData(0) != 0) as BOOL;
    if bHasData == 0 && hWndClear == GetFocus() {
        SendMessageW(hwnd, WM_NEXTDLGCTL, 0, 0);
    }
    EnableWindow(hWndClear, bHasData);
}

// ===========================================================================
// CUSTOM_ENTRY table — one checkable TreeView option per advanced setting
// ===========================================================================

/// `FN_CUSTOM_GET` — `BOOL CALLBACK (*)(const CUSTOM_ENTRY*)`.
type FnCustomGet = unsafe extern "system" fn(*const CustomEntry) -> BOOL;
/// `FN_CUSTOM_SET` — `VOID CALLBACK (*)(const CUSTOM_ENTRY*, BOOL)`.
type FnCustomSet = unsafe extern "system" fn(*const CustomEntry, BOOL);

/// `struct CUSTOM_ENTRY`. `policy1`/`policy2` are RESTRICTIONS (shlobj.h enum → u32);
/// a zero policy means "no restriction" (SHRestricted(0) returns 0).
#[repr(C)]
struct CustomEntry {
    id: LPARAM,
    name: LPCWSTR,
    bDefaultValue: BOOL,
    fnGetValue: FnCustomGet,
    fnSetValue: FnCustomSet,
    policy1: u32,
    policy2: u32,
}

unsafe extern "system" fn CustomGetAdvanced(entry: *const CustomEntry) -> BOOL {
    GetAdvancedBool((*entry).name, (*entry).bDefaultValue)
}

unsafe extern "system" fn CustomSetAdvanced(entry: *const CustomEntry, bValue: BOOL) {
    SetAdvancedDword((*entry).name, bValue as u32);
}

unsafe extern "system" fn CustomGetSmallStartMenu(_entry: *const CustomEntry) -> BOOL {
    (*&raw const g_TaskbarSettings).sr.SmSmallIcons() as BOOL
}

unsafe extern "system" fn CustomSetSmallStartMenu(_entry: *const CustomEntry, bValue: BOOL) {
    (*&raw mut g_TaskbarSettings).sr.set_SmSmallIcons(bValue as DWORD);
}

/// `s_CustomEntries[]` (file-static). Rebuilt per use — same fn pointers + wide
/// literals as the C++ static table, so behaviorally identical.
fn s_custom_entries() -> [CustomEntry; 10] {
    [
        CustomEntry {
            id: IDS_ADVANCED_DISPLAY_ADMINTOOLS,
            name: w!("StartMenuAdminTools").as_ptr(),
            bDefaultValue: 1,
            fnGetValue: CustomGetAdvanced,
            fnSetValue: CustomSetAdvanced,
            policy1: 0,
            policy2: 0,
        },
        CustomEntry {
            id: IDS_ADVANCED_DISPLAY_FAVORITES,
            name: w!("StartMenuFavorites").as_ptr(),
            bDefaultValue: 0,
            fnGetValue: CustomGetAdvanced,
            fnSetValue: CustomSetAdvanced,
            policy1: REST_NOFAVORITESMENU,
            policy2: 0,
        },
        CustomEntry {
            id: IDS_ADVANCED_DISPLAY_LOG_OFF,
            name: w!("StartMenuLogoff").as_ptr(),
            bDefaultValue: 0,
            fnGetValue: CustomGetAdvanced,
            fnSetValue: CustomSetAdvanced,
            policy1: REST_STARTMENULOGOFF,
            policy2: 0,
        },
        CustomEntry {
            id: IDS_ADVANCED_DISPLAY_RUN,
            name: w!("StartMenuRun").as_ptr(),
            bDefaultValue: 1,
            fnGetValue: CustomGetAdvanced,
            fnSetValue: CustomSetAdvanced,
            policy1: REST_NORUN,
            policy2: 0,
        },
        CustomEntry {
            id: IDS_ADVANCED_EXPAND_MY_DOCUMENTS,
            name: w!("CascadeMyDocuments").as_ptr(),
            bDefaultValue: 0,
            fnGetValue: CustomGetAdvanced,
            fnSetValue: CustomSetAdvanced,
            policy1: REST_NOSMMYDOCS,
            policy2: 0,
        },
        CustomEntry {
            id: IDS_ADVANCED_EXPAND_MY_PICTURES,
            name: w!("CascadeMyPictures").as_ptr(),
            bDefaultValue: 0,
            fnGetValue: CustomGetAdvanced,
            fnSetValue: CustomSetAdvanced,
            policy1: REST_NOSMMYPICS,
            policy2: 0,
        },
        CustomEntry {
            id: IDS_ADVANCED_EXPAND_CONTROL_PANEL,
            name: w!("CascadeControlPanel").as_ptr(),
            bDefaultValue: 0,
            fnGetValue: CustomGetAdvanced,
            fnSetValue: CustomSetAdvanced,
            policy1: REST_NOSETFOLDERS,
            policy2: REST_NOCONTROLPANEL,
        },
        CustomEntry {
            id: IDS_ADVANCED_EXPAND_PRINTERS,
            name: w!("CascadePrinters").as_ptr(),
            bDefaultValue: 0,
            fnGetValue: CustomGetAdvanced,
            fnSetValue: CustomSetAdvanced,
            policy1: REST_NOSETFOLDERS,
            policy2: 0,
        },
        CustomEntry {
            id: IDS_ADVANCED_EXPAND_NET_CONNECTIONS,
            name: w!("CascadeNetworkConnections").as_ptr(),
            bDefaultValue: 0,
            fnGetValue: CustomGetAdvanced,
            fnSetValue: CustomSetAdvanced,
            policy1: REST_NOSETFOLDERS,
            policy2: REST_NONETWORKCONNECTIONS,
        },
        CustomEntry {
            id: IDS_ADVANCED_SMALL_START_MENU,
            name: null(),
            bDefaultValue: 0,
            fnGetValue: CustomGetSmallStartMenu,
            fnSetValue: CustomSetSmallStartMenu,
            policy1: 0,
            policy2: 0,
        },
    ]
}

unsafe fn AddCustomItem(hTreeView: HWND, entry: *const CustomEntry) {
    if SHRestricted((*entry).policy1) != 0 || SHRestricted((*entry).policy2) != 0 {
        return; // Restricted. Don't show.
    }

    let mut szText: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    LoadStringW(
        GetModuleHandleW(w!("shell32.dll").as_ptr()),
        (*entry).id as UINT,
        szText.as_mut_ptr(),
        MAX_PATH as c_int,
    );

    let bChecked = ((*entry).fnGetValue)(entry);

    let mut ins: TVINSERTSTRUCTW = core::mem::zeroed();
    ins.hParent = TVI_ROOT;
    ins.hInsertAfter = TVI_LAST;
    ins.item.mask = TVIF_TEXT | TVIF_STATE | TVIF_PARAM;
    ins.item.pszText = szText.as_mut_ptr();
    ins.item.lParam = (*entry).id;
    ins.item.stateMask = TVIS_STATEIMAGEMASK;
    ins.item.state = INDEXTOSTATEIMAGEMASK(if bChecked != 0 { I_CHECKED } else { I_UNCHECKED });
    TreeView_InsertItem(hTreeView, &ins);
}

// ===========================================================================
// Dialog proc + entry point
// ===========================================================================

unsafe fn set_checkbox_style(hTreeView: HWND) {
    #[cfg(target_arch = "x86")]
    {
        let style = GetWindowLongW(hTreeView, GWL_STYLE);
        SetWindowLongW(hTreeView, GWL_STYLE, style | TVS_CHECKBOXES);
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let style = GetWindowLongPtrW(hTreeView, GWL_STYLE);
        SetWindowLongPtrW(hTreeView, GWL_STYLE, style | TVS_CHECKBOXES as LONG_PTR);
    }
}

unsafe fn CustomizeClassic_OnInitDialog(hwnd: HWND) {
    InitializeClearButton(hwnd);

    let hTreeView = GetDlgItem(hwnd, IDC_CLASSICSTART_SETTINGS);
    set_checkbox_style(hTreeView);

    let entries = s_custom_entries();
    for entry in entries.iter() {
        AddCustomItem(hTreeView, entry as *const CustomEntry);
    }
}

unsafe fn CustomizeClassic_OnOK(hwnd: HWND) -> BOOL {
    let hTreeView = GetDlgItem(hwnd, IDC_CLASSICSTART_SETTINGS);
    let entries = s_custom_entries();

    let mut hItem = TreeView_GetRoot(hTreeView);
    while !hItem.is_null() {
        let mut item: TVITEMW = core::mem::zeroed();
        item.mask = TVIF_PARAM | TVIF_STATE;
        item.hItem = hItem;
        item.stateMask = TVIS_STATEIMAGEMASK;
        TreeView_GetItem(hTreeView, &mut item);

        let bChecked: BOOL = ((item.state & INDEXTOSTATEIMAGEMASK(I_CHECKED)) != 0) as BOOL;
        for entry in entries.iter() {
            if SHRestricted(entry.policy1) != 0 || SHRestricted(entry.policy2) != 0 {
                continue;
            }
            if item.lParam == entry.id {
                (entry.fnSetValue)(entry as *const CustomEntry, bChecked);
                break;
            }
        }

        hItem = TreeView_GetNextVisible(hTreeView, hItem);
    }

    SendMessageTimeoutW(
        HWND_BROADCAST,
        WM_SETTINGCHANGE,
        0,
        w!("TraySettings").as_ptr() as LPARAM,
        SMTO_ABORTIFHUNG,
        200,
        null_mut(),
    );
    1
}

unsafe extern "system" fn CustomizeClassicProc(
    hwnd: HWND,
    Message: UINT,
    wParam: WPARAM,
    _lParam: LPARAM,
) -> INT_PTR {
    match Message {
        WM_INITDIALOG => {
            CustomizeClassic_OnInitDialog(hwnd);
            return 1;
        }
        WM_COMMAND => {
            match (wParam & 0xFFFF) as c_int {
                IDC_CLASSICSTART_ADD => OnAddStartMenuItems(hwnd),
                IDC_CLASSICSTART_REMOVE => OnRemoveStartmenuItems(hwnd),
                IDC_CLASSICSTART_ADVANCED => OnAdvancedStartMenuItems(),
                IDC_CLASSICSTART_CLEAR => {
                    ClearRecentAndMru();
                    InitializeClearButton(hwnd);
                }
                x if x == IDOK as c_int => {
                    if CustomizeClassic_OnOK(hwnd) != 0 {
                        EndDialog(hwnd, IDOK as INT_PTR);
                    }
                }
                x if x == IDCANCEL as c_int => {
                    EndDialog(hwnd, IDCANCEL as INT_PTR);
                }
                _ => {}
            }
        }
        _ => {}
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn ShowCustomizeClassic(hInst: HINSTANCE, hExplorer: HWND) {
    explorer_sys::externs::DialogBoxParamW(
        hInst,
        MAKEINTRESOURCEW(IDD_CLASSICSTART_CUSTOMIZE as u16),
        hExplorer,
        Some(CustomizeClassicProc),
        0,
    );
}
