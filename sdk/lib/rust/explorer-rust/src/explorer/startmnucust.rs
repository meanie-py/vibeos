use core::ffi::c_void;
use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use winsrv_sys::consts::*;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

extern "C" {
    fn ShellExecuteW( hwnd: HWND, lpOperation: PCWSTR, lpFile: PCWSTR, lpParameters: PCWSTR, lpDirectory: PCWSTR, nShowCmd: i32, ) -> DWORD_PTR;
    fn SHGetFolderPathW(hwnd: HWND, csidl: i32, hToken: *mut core::ffi::c_void, dwFlags: DWORD, pszPath: *mut u16) -> HRESULT;
    fn wsprintfW(buf: *mut u16, fmt: PCWSTR, ...) -> i32;
    fn SHDeleteKeyW(hKey: HKEY, pszSubKey: PCWSTR) -> i32;
    fn GetAdvancedBool(name: LPCWSTR, bDefaultValue: BOOL) -> BOOL;
    fn SetAdvancedDword(name: PCWSTR, value: u8);
    fn SHRestricted(rest: RESTRICTIONS) -> i32;
    fn TRACE(fmt: *const i8, ...);
    fn TreeView_InsertItem(hwnd: HWND, lpis: *const TV_INSERTSTRUCT) -> HTREEITEM;
    fn DialogBoxW(hInstance: HINSTANCE, lpTemplateName: PCWSTR, hWndParent: HWND, lpDialogFunc: *const core::ffi::c_void) -> INT_PTR;
    fn MAKEINTRESOURCEW(i: u16) -> PCWSTR;
    fn SHAddToRecentDocs(uFlags: DWORD_PTR, pv: *const core::ffi::c_void);
}

// ---- file-scope preamble (translated) ----
pub const I_UNCHECKED: i32 = 1;

pub const I_CHECKED: i32 = 2;

pub static g_MruKeys: [PCWSTR; 4] = [
    b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0I\0n\0t\0e\0r\0n\0e\0t\0 \0E\0x\0p\0l\0o\0r\0e\0r\0\\\0T\0y\0p\0e\0d\0U\0R\0L\0s\0\0\0".as_ptr() as PCWSTR,
    b"E\0x\0p\0l\0o\0r\0e\0r\0\\\0R\0u\0n\0M\0R\0U\0\0\0".as_ptr() as PCWSTR,
    b"E\0x\0p\0l\0o\0r\0e\0r\0\\\0C\0o\0m\0d\0l\0g\x003\x002\0\\\0O\0p\0e\0n\0S\0a\0v\0e\0M\0R\0U\0\0\0".as_ptr() as PCWSTR,
    b"E\0x\0p\0l\0o\0r\0e\0r\0\\\0C\0o\0m\0d\0l\0g\x003\x002\0\\\0L\0a\0s\0t\0V\0i\0s\0i\0t\0e\0d\0M\0R\0U\0\0\0".as_ptr() as PCWSTR,
];

pub type FN_CUSTOM_GET = Option<unsafe extern "system" fn(entry: *const CUSTOM_ENTRY) -> BOOL>;
pub type FN_CUSTOM_SET = Option<unsafe extern "system" fn(entry: *const CUSTOM_ENTRY, bValue: BOOL)>;

#[repr(C)]
pub struct CUSTOM_ENTRY {
    pub id: LPARAM,
    pub name: LPCWSTR,
    pub bDefaultValue: BOOL,
    pub fnGetValue: FN_CUSTOM_GET,
    pub fnSetValue: FN_CUSTOM_SET,
    pub policy1: RESTRICTIONS,
    pub policy2: RESTRICTIONS,
}

pub static s_CustomEntries: [CUSTOM_ENTRY; 10] = [
    CUSTOM_ENTRY {
        id: IDS_ADVANCED_DISPLAY_ADMINTOOLS,
        name: b"S\0t\0a\0r\0t\0M\0e\0n\0u\0A\0d\0m\0i\0n\0T\0o\0o\0l\0s\0\0\0".as_ptr() as LPCWSTR,
        bDefaultValue: TRUE,
        fnGetValue: Some(CustomGetAdvanced),
        fnSetValue: Some(CustomSetAdvanced),
        policy1: 0,
        policy2: 0,
    },
    CUSTOM_ENTRY {
        id: IDS_ADVANCED_DISPLAY_FAVORITES,
        name: b"S\0t\0a\0r\0t\0M\0e\0n\0u\0F\0a\0v\0o\0r\0i\0t\0e\0s\0\0\0".as_ptr() as LPCWSTR,
        bDefaultValue: FALSE,
        fnGetValue: Some(CustomGetAdvanced),
        fnSetValue: Some(CustomSetAdvanced),
        policy1: REST_NOFAVORITESMENU,
        policy2: 0,
    },
    CUSTOM_ENTRY {
        id: IDS_ADVANCED_DISPLAY_LOG_OFF,
        name: b"S\0t\0a\0r\0t\0M\0e\0n\0u\0L\0o\0g\0o\0f\0f\0\0\0".as_ptr() as LPCWSTR,
        bDefaultValue: FALSE,
        fnGetValue: Some(CustomGetAdvanced),
        fnSetValue: Some(CustomSetAdvanced),
        policy1: REST_STARTMENULOGOFF,
        policy2: 0,
    },
    CUSTOM_ENTRY {
        id: IDS_ADVANCED_DISPLAY_RUN,
        name: b"S\0t\0a\0r\0t\0M\0e\0n\0u\0R\0u\0n\0\0\0".as_ptr() as LPCWSTR,
        bDefaultValue: TRUE,
        fnGetValue: Some(CustomGetAdvanced),
        fnSetValue: Some(CustomSetAdvanced),
        policy1: REST_NORUN,
        policy2: 0,
    },
    CUSTOM_ENTRY {
        id: IDS_ADVANCED_EXPAND_MY_DOCUMENTS,
        name: b"C\0a\0s\0c\0a\0d\0e\0M\0y\0D\0o\0c\0u\0m\0e\0n\0t\0s\0\0\0".as_ptr() as LPCWSTR,
        bDefaultValue: FALSE,
        fnGetValue: Some(CustomGetAdvanced),
        fnSetValue: Some(CustomSetAdvanced),
        policy1: REST_NOSMMYDOCS,
        policy2: 0,
    },
    CUSTOM_ENTRY {
        id: IDS_ADVANCED_EXPAND_MY_PICTURES,
        name: b"C\0a\0s\0c\0a\0d\0e\0M\0y\0P\0i\0c\0t\0u\0r\0e\0s\0\0\0".as_ptr() as LPCWSTR,
        bDefaultValue: FALSE,
        fnGetValue: Some(CustomGetAdvanced),
        fnSetValue: Some(CustomSetAdvanced),
        policy1: REST_NOSMMYPICS,
        policy2: 0,
    },
    CUSTOM_ENTRY {
        id: IDS_ADVANCED_EXPAND_CONTROL_PANEL,
        name: b"C\0a\0s\0c\0a\0d\0e\0C\0o\0n\0t\0r\0o\0l\0P\0a\0n\0e\0l\0\0\0".as_ptr() as LPCWSTR,
        bDefaultValue: FALSE,
        fnGetValue: Some(CustomGetAdvanced),
        fnSetValue: Some(CustomSetAdvanced),
        policy1: REST_NOSETFOLDERS,
        policy2: REST_NOCONTROLPANEL,
    },
    CUSTOM_ENTRY {
        id: IDS_ADVANCED_EXPAND_PRINTERS,
        name: b"C\0a\0s\0c\0a\0d\0e\0P\0r\0i\0n\0t\0e\0r\0s\0\0\0".as_ptr() as LPCWSTR,
        bDefaultValue: FALSE,
        fnGetValue: Some(CustomGetAdvanced),
        fnSetValue: Some(CustomSetAdvanced),
        policy1: REST_NOSETFOLDERS,
        policy2: 0,
    },
    CUSTOM_ENTRY {
        id: IDS_ADVANCED_EXPAND_NET_CONNECTIONS,
        name: b"C\0a\0s\0c\0a\0d\0e\0N\0e\0t\0w\0o\0r\0k\0C\0o\0n\0n\0e\0c\0t\0i\0o\0n\0s\0\0\0".as_ptr() as LPCWSTR,
        bDefaultValue: FALSE,
        fnGetValue: Some(CustomGetAdvanced),
        fnSetValue: Some(CustomSetAdvanced),
        policy1: REST_NOSETFOLDERS,
        policy2: REST_NONETWORKCONNECTIONS,
    },
    CUSTOM_ENTRY {
        id: IDS_ADVANCED_SMALL_START_MENU,
        name: core::ptr::null(),
        bDefaultValue: FALSE,
        fnGetValue: Some(CustomGetSmallStartMenu),
        fnSetValue: Some(CustomSetSmallStartMenu),
        policy1: 0,
        policy2: 0,
    },
];

#[repr(C)]
struct SHELLEXECUTEINFOW {
    cbSize: DWORD,
    fMask: ULONG,
    hwnd: HWND,
    lpVerb: PCWSTR,
    lpFile: PCWSTR,
    lpParameters: PCWSTR,
    lpDirectory: PCWSTR,
    nShow: i32,
    hInstApp: HINSTANCE,
    lpIDList: *mut core::ffi::c_void,
    lpClass: PCWSTR,
    hkeyClass: HKEY,
    dwHotKey: DWORD,
    hIcon: *mut core::ffi::c_void,
    hProcess: *mut core::ffi::c_void,
}

type DWORD = u32;
type ULONG = u32;
type HRESULT = i32;

const CSIDL_STARTMENU: i32 = 0x000b;
#[no_mangle]
pub unsafe extern "C" fn OnAddStartMenuItems(hDlg: HWND) {
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if SUCCEEDED(SHGetFolderPathW(
        core::ptr::null_mut(),
        CSIDL_PROGRAMS,
        core::ptr::null_mut(),
        0,
        szPath.as_mut_ptr(),
    )) != 0
    {
        let mut szCommand: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let prefix = ['a' as u16, 'p' as u16, 'p' as u16, 'w' as u16, 'i' as u16, 'z' as u16, '.' as u16, 'c' as u16, 'p' as u16, 'l' as u16, ',' as u16, 'N' as u16, 'e' as u16, 'w' as u16, 'L' as u16, 'i' as u16, 'n' as u16, 'k' as u16, 'H' as u16, 'e' as u16, 'r' as u16, 'e' as u16, ' ' as u16, 0];
        core::ptr::copy_nonoverlapping(prefix.as_ptr(), szCommand.as_mut_ptr(), prefix.len());
        
        if SUCCEEDED(StringCchCatW(
            szCommand.as_mut_ptr(),
            MAX_PATH as usize,
            szPath.as_ptr(),
        )) != 0
        {
            ShellExecuteW(
                hDlg,
                ['o' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 0].as_ptr(),
                ['r' as u16, 'u' as u16, 'n' as u16, 'd' as u16, 'l' as u16, 'l' as u16, '3' as u16, '2' as u16, '.' as u16, 'e' as u16, 'x' as u16, 'e' as u16, 0].as_ptr(),
                szCommand.as_ptr(),
                core::ptr::null(),
                SW_SHOWNORMAL,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn OnRemoveStartmenuItems(hDlg: HWND) {
    
    ShellExecuteW(
        hDlg,
        b"o\0p\0e\0n\0\0\0".as_ptr() as PCWSTR,
        b"r\0u\0n\0d\0l\0l\03\02\0.\0e\0x\0e\0\0\0".as_ptr() as PCWSTR,
        b"a\0p\0p\0w\0i\0z\0.\0c\0p\0l\0,\0C\0o\0n\0f\0i\0g\0S\0t\0a\0r\0t\0M\0e\0n\0u\0\0\0".as_ptr() as PCWSTR,
        core::ptr::null(),
        SW_SHOWNORMAL,
    );
}

unsafe fn SUCCEEDED(hr: HRESULT) -> bool {
    hr >= 0
}
#[no_mangle]
pub unsafe extern "C" fn OnAdvancedStartMenuItems() {
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if SUCCEEDED(SHGetFolderPathW(
        core::ptr::null_mut(),
        CSIDL_STARTMENU,
        core::ptr::null_mut(),
        0,
        szPath.as_mut_ptr(),
    )) {
        ShellExecuteW(
            core::ptr::null_mut(),
            b"e\0x\0p\0l\0o\0r\0e\0\0\0".as_ptr() as PCWSTR,
            szPath.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SW_SHOWNORMAL,
        );
    }
}

unsafe fn RecentHasShortcut(hwnd: HWND) -> BOOL {
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if FAILED(SHGetFolderPathW(
        hwnd,
        CSIDL_RECENT | CSIDL_FLAG_CREATE,
        core::ptr::null_mut(),
        0,
        szPath.as_mut_ptr(),
    )) {
        return FALSE;
    }

    let mut find: WIN32_FIND_DATAW = core::mem::zeroed();
    PathAppendW(szPath.as_mut_ptr(), b"*\0.\0l\0n\0k\0\0".as_ptr() as PCWSTR);
    let hFind = FindFirstFileW(szPath.as_ptr(), &mut find);
    if hFind == INVALID_HANDLE_VALUE {
        return FALSE;
    }

    FindClose(hFind);
    TRUE
}

unsafe fn HandleMruData(Delete: u8) -> u8 {
    let mut i: u32 = 0;
    while i < (core::mem::size_of_val(&g_MruKeys) / core::mem::size_of::<PCWSTR>()) as u32 {
        let mut szKey: [u16; 200] = [0; 200];
        let pszKey_orig: PCWSTR = g_MruKeys[i as usize];
        let mut pszKey: PCWSTR = pszKey_orig;
        
        if *pszKey_orig != 'S' as u16 {
            wsprintfW(
                szKey.as_mut_ptr(),
                b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0%\0s\0\0\0".as_ptr() as PCWSTR,
                pszKey_orig
            );
            pszKey = szKey.as_ptr();
        }

        let mut hKey: HKEY = core::ptr::null_mut();
        if Delete != 0 {
            SHDeleteKeyW(0x80000001u32 as HKEY, pszKey);
        } else if RegOpenKeyExW(0x80000001u32 as HKEY, pszKey, 0, KEY_WRITE, &mut hKey) == ERROR_SUCCESS {
            RegCloseKey(hKey);
            return 1u8;
        }
        
        i += 1;
    }
    return 0u8;
}
#[no_mangle]
pub unsafe extern "C" fn ClearRecentAndMru() {
    
    SHAddToRecentDocs(0x00000001, core::ptr::null());
    HandleMruData(1);
}

unsafe fn InitializeClearButton(hwnd: HWND) {
    let hWndClear = GetDlgItem(hwnd, IDC_CLASSICSTART_CLEAR);
    let bHasData = (RecentHasShortcut(hwnd) != 0) || (HandleMruData(0u8) != 0);
    if !bHasData && hWndClear == GetFocus() {
        SendMessage(hwnd, WM_NEXTDLGCTL, 0, 0);
    }
    EnableWindow(hWndClear, if bHasData { 1 } else { 0 });
}
unsafe fn CustomGetAdvanced(entry: *const CUSTOM_ENTRY) -> BOOL {
    GetAdvancedBool((*entry).name, (*entry).bDefaultValue)
}
unsafe fn CustomSetAdvanced(entry: *const CUSTOM_ENTRY, bValue: u8) {
    SetAdvancedDword((*entry).name, bValue);
}
unsafe fn CustomGetSmallStartMenu(entry: *const CUSTOM_ENTRY) -> u8 {
    g_TaskbarSettings.sr.SmSmallIcons as u8
}

unsafe fn CustomSetSmallStartMenu(entry: *const CUSTOM_ENTRY, bValue: u8) {
    g_TaskbarSettings.sr.SmSmallIcons = bValue;
}

unsafe fn AddCustomItem(hTreeView: HWND, entry: *const CUSTOM_ENTRY) {
    if SHRestricted((*entry).policy1) != 0 || SHRestricted((*entry).policy2) != 0 {
        TRACE!(b"%p: Restricted\n\0".as_ptr() as *const i8, (*entry).id);
        return;
    }

    let mut szText: [u16; MAX_PATH as usize] = core::mem::zeroed();
    LoadStringW(
        GetModuleHandleW(b"shell32.dll\0".as_ptr()),
        (*entry).id as u32,
        szText.as_mut_ptr(),
        MAX_PATH as i32,
    );

    let bChecked = ((*entry).fnGetValue)(entry);
    TRACE!(b"%p: %d\n\0".as_ptr() as *const i8, (*entry).id, bChecked as i32);

    let mut Insert: TV_INSERTSTRUCT = core::mem::zeroed();
    Insert.hParent = TVI_ROOT;
    Insert.hInsertAfter = TVI_LAST;
    Insert.item.mask = TVIF_TEXT | TVIF_STATE | TVIF_PARAM;
    Insert.item.pszText = szText.as_mut_ptr();
    Insert.item.lParam = (*entry).id;
    Insert.item.stateMask = TVIS_STATEIMAGEMASK;
    Insert.item.state = INDEXTOSTATEIMAGEMASK(if bChecked != 0 { I_CHECKED } else { I_UNCHECKED });
    TreeView_InsertItem(hTreeView, &mut Insert);
}

const I_UNCHECKED: u32 = 1;
const I_CHECKED: u32 = 2;

fn INDEXTOSTATEIMAGEMASK(i: u32) -> u32 {
    i << 12
}

unsafe fn CustomizeClassic_OnInitDialog(hwnd: HWND) {
    InitializeClearButton(hwnd);

    let hTreeView = GetDlgItem(hwnd, IDC_CLASSICSTART_SETTINGS);

    let style = GetWindowLongPtrW(hTreeView, GWL_STYLE);
    SetWindowLongPtrW(hTreeView, GWL_STYLE, style | TVS_CHECKBOXES);

    for entry in &s_CustomEntries {
        AddCustomItem(hTreeView, entry);
    }
}

unsafe fn CustomizeClassic_OnOK(hwnd: HWND) -> i32 {
    let hTreeView = GetDlgItem(hwnd, IDC_CLASSICSTART_SETTINGS);

    let mut hItem = TreeView_GetRoot(hTreeView);
    while !hItem.is_null() {
        let mut item: TV_ITEM = core::mem::zeroed();
        item.mask = TVIF_PARAM | TVIF_STATE;
        item.hItem = hItem;
        item.stateMask = TVIS_STATEIMAGEMASK;
        TreeView_GetItem(hTreeView, &mut item);

        let bChecked = (item.state & INDEXTOSTATEIMAGEMASK(I_CHECKED)) != 0;
        
        for entry in &s_CustomEntries {
            if SHRestricted(entry.policy1) != 0 || SHRestricted(entry.policy2) != 0 {
                continue;
            }

            if item.lParam == entry.id {
                (entry.fnSetValue)(entry, if bChecked { 1u8 } else { 0u8 });
                break;
            }
        }

        hItem = TreeView_GetNextVisible(hTreeView, hItem);
    }

    SendMessageTimeoutW(
        HWND_BROADCAST,
        WM_SETTINGCHANGE,
        0,
        b"TraySettings\0".as_ptr() as LPARAM,
        SMTO_ABORTIFHUNG,
        200,
        core::ptr::null_mut()
    );
    
    1
}
#[no_mangle]
pub unsafe extern "C" fn CustomizeClassicProc(hwnd: HWND, Message: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match Message {
        WM_INITDIALOG => {
            CustomizeClassic_OnInitDialog(hwnd);
            return 1;
        }
        WM_COMMAND => {
            match (wParam & 0xFFFF) as u32 {
                IDC_CLASSICSTART_ADD => {
                    OnAddStartMenuItems(hwnd);
                }
                IDC_CLASSICSTART_REMOVE => {
                    OnRemoveStartmenuItems(hwnd);
                }
                IDC_CLASSICSTART_ADVANCED => {
                    OnAdvancedStartMenuItems();
                }
                IDC_CLASSICSTART_CLEAR => {
                    ClearRecentAndMru();
                    InitializeClearButton(hwnd);
                }
                IDOK => {
                    if CustomizeClassic_OnOK(hwnd) != 0 {
                        EndDialog(hwnd, IDOK as INT_PTR);
                    }
                }
                IDCANCEL => {
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
    
    DialogBoxW(hInst, MAKEINTRESOURCEW(IDD_CLASSICSTART_CUSTOMIZE), hExplorer, CustomizeClassicProc as *const core::ffi::c_void);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Explorer
//  * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
//  * PURPOSE:     "Customize Start Menu" dialog
//  * COPYRIGHT:   Copyright 2006-2007 Thomas Weidenmueller <w3seek@reactos.org>
//  *              Copyright 2015 Robert Naumann <gonzomdx@gmail.com>
//  *              Copyright 2024 Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  */
//
// #include "precomp.h"
//
// // TreeView checkbox state indexes (Use with INDEXTOSTATEIMAGEMASK macro)
// #define I_UNCHECKED 1
// #define I_CHECKED   2
//
// // TODO: Windows Explorer appears to be calling NewLinkHere / ConfigStartMenu directly for both items.
// VOID OnAddStartMenuItems(HWND hDlg)
// {
//     WCHAR szPath[MAX_PATH];
//
//     if (SUCCEEDED(SHGetFolderPathW(NULL, CSIDL_PROGRAMS, NULL, 0, szPath)))
//     {
//         WCHAR szCommand[MAX_PATH] = L"appwiz.cpl,NewLinkHere ";
//         if (SUCCEEDED(StringCchCatW(szCommand, _countof(szCommand), szPath)))
//             ShellExecuteW(hDlg, L"open", L"rundll32.exe", szCommand, NULL, SW_SHOWNORMAL);
//     }
// }
//
// VOID OnRemoveStartmenuItems(HWND hDlg)
// {
//     ShellExecuteW(hDlg, L"open", L"rundll32.exe", L"appwiz.cpl,ConfigStartMenu", NULL, SW_SHOWNORMAL);
// }
//
// VOID OnAdvancedStartMenuItems()
// {
//     WCHAR szPath[MAX_PATH];
//
//     if (SUCCEEDED(SHGetFolderPathW(NULL, CSIDL_STARTMENU, NULL, 0, szPath)))
//     {
//         ShellExecuteW(NULL, L"explore", szPath, NULL, NULL, SW_SHOWNORMAL);
//     }
// }
//
// static BOOL RecentHasShortcut(HWND hwnd)
// {
//     WCHAR szPath[MAX_PATH];
//     if (FAILED(SHGetFolderPathW(hwnd, CSIDL_RECENT | CSIDL_FLAG_CREATE, NULL, 0, szPath)))
//         return FALSE;
//
//     // Find shortcut files in Recent
//     WIN32_FIND_DATAW find;
//     PathAppendW(szPath, L"*.lnk");
//     HANDLE hFind = FindFirstFileW(szPath, &find);
//     if (hFind == INVALID_HANDLE_VALUE)
//         return FALSE;
//
//     FindClose(hFind);
//     return TRUE;
// }
//
// static const PCWSTR g_MruKeys[] = 
// {
//     L"Software\\Microsoft\\Internet Explorer\\TypedURLs",
//     L"Explorer\\RunMRU",
//     L"Explorer\\Comdlg32\\OpenSaveMRU",
//     L"Explorer\\Comdlg32\\LastVisitedMRU",
// };
//
// static BOOL HandleMruData(BOOL Delete)
// {
//     for (UINT i = 0; i < _countof(g_MruKeys); ++i)
//     {
//         WCHAR szKey[200];
//         PCWSTR pszKey = g_MruKeys[i];
//         if (*pszKey != 'S') // Keys not starting with S[oftware] are assumed to be relative to "SMWCV"
//         {
//             wsprintfW(szKey, L"%s\\%s", L"Software\\Microsoft\\Windows\\CurrentVersion", pszKey);
//             pszKey = szKey;
//         }
//
//         HKEY hKey;
//         if (Delete)
//         {
//             SHDeleteKeyW(HKEY_CURRENT_USER, pszKey);
//         }
//         else if (RegOpenKeyExW(HKEY_CURRENT_USER, pszKey, 0, KEY_WRITE, &hKey) == ERROR_SUCCESS)
//         {
//             RegCloseKey(hKey);
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
//
// VOID ClearRecentAndMru()
// {
//     SHAddToRecentDocs(SHARD_PIDL, NULL);
//     HandleMruData(TRUE);
// }
//
// static VOID InitializeClearButton(HWND hwnd)
// {
//     HWND hWndClear = GetDlgItem(hwnd, IDC_CLASSICSTART_CLEAR);
//     BOOL bHasData = RecentHasShortcut(hwnd) || HandleMruData(FALSE);
//     if (!bHasData && hWndClear == GetFocus())
//         SendMessage(hwnd, WM_NEXTDLGCTL, 0, FALSE);
//     EnableWindow(hWndClear, bHasData);
// }
//
// struct CUSTOM_ENTRY;
//
// typedef BOOL (__stdcall *FN_CUSTOM_GET)(const CUSTOM_ENTRY *entry);
// typedef VOID (__stdcall *FN_CUSTOM_SET)(const CUSTOM_ENTRY *entry, BOOL bValue);
//
// struct CUSTOM_ENTRY
// {
//     LPARAM id;
//     LPCWSTR name;
//     BOOL bDefaultValue;
//     FN_CUSTOM_GET fnGetValue;
//     FN_CUSTOM_SET fnSetValue;
//     RESTRICTIONS policy1, policy2;
// };
//
// static BOOL __stdcall CustomGetAdvanced(const CUSTOM_ENTRY *entry)
// {
//     return GetAdvancedBool(entry->name, entry->bDefaultValue);
// }
//
// static VOID __stdcall CustomSetAdvanced(const CUSTOM_ENTRY *entry, BOOL bValue)
// {
//     SetAdvancedDword(entry->name, bValue);
// }
//
// static BOOL __stdcall CustomGetSmallStartMenu(const CUSTOM_ENTRY *entry)
// {
//     return g_TaskbarSettings.sr.SmSmallIcons;
// }
//
// static VOID __stdcall CustomSetSmallStartMenu(const CUSTOM_ENTRY *entry, BOOL bValue)
// {
//     g_TaskbarSettings.sr.SmSmallIcons = bValue;
// }
//
// static const CUSTOM_ENTRY s_CustomEntries[] =
// {
//     {
//         IDS_ADVANCED_DISPLAY_ADMINTOOLS, L"StartMenuAdminTools", TRUE,
//         CustomGetAdvanced, CustomSetAdvanced,
//     },
//     {
//         IDS_ADVANCED_DISPLAY_FAVORITES, L"StartMenuFavorites", FALSE,
//         CustomGetAdvanced, CustomSetAdvanced,
//         REST_NOFAVORITESMENU,
//     },
//     {
//         IDS_ADVANCED_DISPLAY_LOG_OFF, L"StartMenuLogoff", FALSE,
//         CustomGetAdvanced, CustomSetAdvanced,
//         REST_STARTMENULOGOFF,
//     },
//     {
//         IDS_ADVANCED_DISPLAY_RUN, L"StartMenuRun", TRUE,
//         CustomGetAdvanced, CustomSetAdvanced,
//         REST_NORUN,
//     },
//     {
//         IDS_ADVANCED_EXPAND_MY_DOCUMENTS, L"CascadeMyDocuments", FALSE,
//         CustomGetAdvanced, CustomSetAdvanced,
//         REST_NOSMMYDOCS,
//     },
//     {
//         IDS_ADVANCED_EXPAND_MY_PICTURES, L"CascadeMyPictures", FALSE,
//         CustomGetAdvanced, CustomSetAdvanced,
//         REST_NOSMMYPICS,
//     },
//     {
//         IDS_ADVANCED_EXPAND_CONTROL_PANEL, L"CascadeControlPanel", FALSE,
//         CustomGetAdvanced, CustomSetAdvanced,
//         REST_NOSETFOLDERS, REST_NOCONTROLPANEL,
//     },
//     {
//         IDS_ADVANCED_EXPAND_PRINTERS, L"CascadePrinters", FALSE,
//         CustomGetAdvanced, CustomSetAdvanced,
//         REST_NOSETFOLDERS,
//     },
//     {
//         IDS_ADVANCED_EXPAND_NET_CONNECTIONS, L"CascadeNetworkConnections", FALSE,
//         CustomGetAdvanced, CustomSetAdvanced,
//         REST_NOSETFOLDERS, REST_NONETWORKCONNECTIONS,
//     },
//     {
//         IDS_ADVANCED_SMALL_START_MENU, NULL, FALSE,
//         CustomGetSmallStartMenu, CustomSetSmallStartMenu,
//     },
// };
//
// static VOID AddCustomItem(HWND hTreeView, const CUSTOM_ENTRY *entry)
// {
//     if (SHRestricted(entry->policy1) || SHRestricted(entry->policy2))
//     {
//         TRACE("%p: Restricted\n", entry->id);
//         return; // Restricted. Don't show
//     }
//
//     WCHAR szText[MAX_PATH];
//     LoadStringW(GetModuleHandleW(L"shell32.dll"), entry->id, szText, _countof(szText));
//
//     BOOL bChecked = entry->fnGetValue(entry);
//     TRACE("%p: %d\n", entry->id, bChecked);
//
//     TV_INSERTSTRUCT Insert = { TVI_ROOT, TVI_LAST, { TVIF_TEXT | TVIF_STATE | TVIF_PARAM } };
//     Insert.item.pszText = szText;
//     Insert.item.lParam = entry->id;
//     Insert.item.stateMask = TVIS_STATEIMAGEMASK;
//     Insert.item.state = INDEXTOSTATEIMAGEMASK(bChecked ? I_CHECKED : I_UNCHECKED);
//     TreeView_InsertItem(hTreeView, &Insert);
// }
//
// static void CustomizeClassic_OnInitDialog(HWND hwnd)
// {
//     InitializeClearButton(hwnd);
//
//     HWND hTreeView = GetDlgItem(hwnd, IDC_CLASSICSTART_SETTINGS);
//
//     DWORD_PTR style = GetWindowLongPtrW(hTreeView, GWL_STYLE);
//     SetWindowLongPtrW(hTreeView, GWL_STYLE, style | TVS_CHECKBOXES);
//
//     for (auto& entry : s_CustomEntries)
//     {
//         AddCustomItem(hTreeView, &entry);
//     }
// }
//
// static BOOL CustomizeClassic_OnOK(HWND hwnd)
// {
//     HWND hTreeView = GetDlgItem(hwnd, IDC_CLASSICSTART_SETTINGS);
//
//     for (HTREEITEM hItem = TreeView_GetRoot(hTreeView);
//          hItem != NULL;
//          hItem = TreeView_GetNextVisible(hTreeView, hItem))
//     {
//         TV_ITEM item = { TVIF_PARAM | TVIF_STATE, hItem };
//         item.stateMask = TVIS_STATEIMAGEMASK;
//         TreeView_GetItem(hTreeView, &item);
//
//         BOOL bChecked = !!(item.state & INDEXTOSTATEIMAGEMASK(I_CHECKED));
//         for (auto& entry : s_CustomEntries)
//         {
//             if (SHRestricted(entry.policy1) || SHRestricted(entry.policy2))
//                 continue;
//
//             if (item.lParam == entry.id)
//             {
//                 TRACE("%p: %d\n", item.lParam, bChecked);
//                 entry.fnSetValue(&entry, bChecked);
//                 break;
//             }
//         }
//     }
//
//     SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, (LPARAM)L"TraySettings",
//                         SMTO_ABORTIFHUNG, 200, NULL);
//     return TRUE;
// }
//
// INT_PTR __stdcall CustomizeClassicProc(HWND hwnd, UINT Message, WPARAM wParam, LPARAM lParam)
// {
//     switch (Message)
//     {
//         case WM_INITDIALOG:
//             CustomizeClassic_OnInitDialog(hwnd);
//             return TRUE;
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_CLASSICSTART_ADD:
//                     OnAddStartMenuItems(hwnd);
//                     break;
//                 case IDC_CLASSICSTART_REMOVE:
//                     OnRemoveStartmenuItems(hwnd);
//                     break;
//                 case IDC_CLASSICSTART_ADVANCED:
//                     OnAdvancedStartMenuItems();
//                     break;
//                 case IDC_CLASSICSTART_CLEAR:
//                     ClearRecentAndMru();
//                     InitializeClearButton(hwnd);
//                     break;
//                 case IDOK:
//                     if (CustomizeClassic_OnOK(hwnd))
//                         EndDialog(hwnd, IDOK);
//                     break;
//                 case IDCANCEL:
//                     EndDialog(hwnd, IDCANCEL);
//                     break;
//             }
//             break;
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
// VOID ShowCustomizeClassic(HINSTANCE hInst, HWND hExplorer)
// {
//     DialogBoxW(hInst, MAKEINTRESOURCEW(IDD_CLASSICSTART_CUSTOMIZE), hExplorer, CustomizeClassicProc);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
