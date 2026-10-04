//! wlx.c — the GINA interface: the WLX_DISPATCH_VERSION_1_4 function table
//! winlogon hands to the GINA DLL (msgina.dll), the GINA loader
//! (LoadGina/GinaInit, GetProcAddress over ~30 WlxXxx exports), the dialog
//! tracking list, and CreateWindowStationAndDesktops. Third GINA/SAS/RPC core
//! flipped (after rpcserver.c + winlogon.c).
//!
//! The ~27 dispatch routines + the 2 default callbacks are WINAPI; GINA calls
//! them through the typed dispatch table, whose bindgen fields are
//! `Option<unsafe extern "stdcall" fn>` on i386 / `extern "C"` on amd64+arm64.
//! `winapi_fns!` defines each with the matching per-arch ABI so `Some(WlxXxx)`
//! slots straight into the table with no transmute. The GINA-provided function
//! pointers (GetProcAddress results) are transmuted from FARPROC into the
//! PFWLX* slots (same machine ABI, distinct Rust type).
//!
//! GinaInit / CreateWindowStationAndDesktops / InitDialogListHead /
//! CloseAllDialogWindows are #[no_mangle] (winlogon.c + sas.c call them).
//! CreateWindowStationAndDesktops calls the (Rust) security module + the
//! sas.c-owned LuidNone (extern for now). Like winlogon.c, only the live
//! preprocessed paths are translated (`#ifdef _M_ARM64` → cfg).

use crate::preamble::{
    wcsncpy, InitializeListHead, InsertHeadList, RemoveEntryList, RtlGetProcessHeap,
    SetWindowStationUser, HKEY_LOCAL_MACHINE, WLSession,
};
use crate::wstr;
use core::mem::{offset_of, size_of, transmute};
use core::ptr::{addr_of, addr_of_mut, null_mut};
use winlogon_sys::nt::*;

// Define a WINAPI fn with the ABI the bindgen WLX/Win32 callback typedefs use, so
// `Some(name)` slots into a typed dispatch field directly.
//
// This used to emit the body TWICE under complementary cfgs — `stdcall` on i386, `C` on
// amd64/arm64 — because the island spelled each slot with the arch's literal convention and
// Rust compares fn-pointer ABIs nominally. normalize_island_abi.py now spells those slots
// `system`, which IS that per-arch convention under one portable name, so one definition
// unifies everywhere. The fork also meant only half of these 29 routines was ever compiled:
// a fix to one arch's copy was invisible until someone built another.
macro_rules! winapi_fns {
    ($(
        $(#[$attr:meta])*
        fn $name:ident ( $($a:ident: $t:ty),* $(,)? ) $(-> $r:ty)? $body:block
    )*) => {
        $(
            $(#[$attr])*
            unsafe extern "system" fn $name($($a: $t),*) $(-> $r)? $body
        )*
    };
}

#[repr(C)]
struct DIALOG_LIST_ENTRY {
    Entry: LIST_ENTRY,
    hWnd: HWND,
    DlgProc: DLGPROC,
    lParam: LPARAM,
}

static mut DialogListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: null_mut(),
    Blink: null_mut(),
};

// sas.c (still C) owns LuidNone (`LUID LuidNone = {0,0};`).
extern "C" {
    static LuidNone: LUID;
}

#[no_mangle]
pub unsafe extern "C" fn InitDialogListHead() {
    InitializeListHead(addr_of_mut!(DialogListHead));
}

unsafe fn AddDialogListEntry() -> *mut DIALOG_LIST_ENTRY {
    let ListEntry = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        size_of::<DIALOG_LIST_ENTRY>() as SIZE_T,
    ) as *mut DIALOG_LIST_ENTRY;
    if ListEntry.is_null() {
        return null_mut();
    }

    InsertHeadList(addr_of_mut!(DialogListHead), addr_of_mut!((*ListEntry).Entry));

    ListEntry
}

unsafe fn RemoveDialogListEntry(ListEntry: *mut DIALOG_LIST_ENTRY) {
    RemoveEntryList(addr_of_mut!((*ListEntry).Entry));
    RtlFreeHeap(RtlGetProcessHeap(), 0, ListEntry as PVOID);
}

unsafe fn GetDialogListEntry(hwndDlg: HWND) -> *mut DIALOG_LIST_ENTRY {
    let head = addr_of_mut!(DialogListHead);
    let mut ListEntry = (*addr_of!(DialogListHead)).Flink;
    while ListEntry != head {
        let Current =
            (ListEntry as *mut u8).sub(offset_of!(DIALOG_LIST_ENTRY, Entry)) as *mut DIALOG_LIST_ENTRY;
        if (*Current).hWnd == hwndDlg {
            return Current;
        }
        ListEntry = (*ListEntry).Flink;
    }

    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn CloseAllDialogWindows() {
    let head = addr_of_mut!(DialogListHead);
    let mut ListEntry = (*addr_of!(DialogListHead)).Flink;
    while ListEntry != head {
        let Current =
            (ListEntry as *mut u8).sub(offset_of!(DIALOG_LIST_ENTRY, Entry)) as *mut DIALOG_LIST_ENTRY;
        // winlogon is non-UNICODE → PostMessage == PostMessageA (no string arg).
        PostMessageA((*Current).hWnd, WLX_WM_SAS, WLX_SAS_TYPE_TIMEOUT as WPARAM, 0);
        ListEntry = (*ListEntry).Flink;
    }
}

winapi_fns! {

fn DefaultWlxWindowProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam_in: LPARAM) -> INT_PTR {
    let ListEntry: *mut DIALOG_LIST_ENTRY;
    let mut lParam: LPARAM = lParam_in;

    if uMsg == WM_INITDIALOG {
        ListEntry = lParam as *mut DIALOG_LIST_ENTRY;
        (*ListEntry).hWnd = hwndDlg;
        lParam = (*ListEntry).lParam;
    } else {
        ListEntry = GetDialogListEntry(hwndDlg);
        if ListEntry.is_null() {
            return 0; // FALSE
        }
    }

    if uMsg == WLX_WM_SAS {
        let ret: INT_PTR = match wParam as u32 {
            WLX_SAS_TYPE_TIMEOUT => WLX_DLG_INPUT_TIMEOUT as INT_PTR,
            WLX_SAS_TYPE_SCRNSVR_TIMEOUT => WLX_DLG_SCREEN_SAVER_TIMEOUT as INT_PTR,
            WLX_SAS_TYPE_USER_LOGOFF => WLX_DLG_USER_LOGOFF as INT_PTR,
            // WLX_SAS_TYPE_CTRL_ALT_DEL and default
            _ => WLX_DLG_SAS as INT_PTR,
        };
        EndDialog(hwndDlg, ret);
        return 1; // TRUE
    }

    (*ListEntry).DlgProc.unwrap()(hwndDlg, uMsg, wParam, lParam)
}

fn WlxUseCtrlAltDel(hWlx: HANDLE) {
    let mut OldValue: ULONG_PTR = 0;
    WlxSetOption(hWlx, WLX_OPTION_USE_CTRL_ALT_DEL, TRUE as ULONG_PTR, &mut OldValue);
}

fn WlxSetContextPointer(hWlx: HANDLE, pWlxContext: PVOID) {
    let mut OldValue: ULONG_PTR = 0;
    WlxSetOption(hWlx, WLX_OPTION_CONTEXT_POINTER, pWlxContext as ULONG_PTR, &mut OldValue);
}

fn WlxSasNotify(hWlx: HANDLE, dwSasType: DWORD) {
    let Session = hWlx as PWLSESSION;
    if dwSasType == WLX_SAS_TYPE_CTRL_ALT_DEL || dwSasType > WLX_SAS_TYPE_MAX_MSFT_VALUE {
        PostMessageW((*Session).SASWindow, WLX_WM_SAS, dwSasType as WPARAM, 0);
    }
}

fn WlxSetTimeout(hWlx: HANDLE, Timeout: DWORD) -> BOOL {
    let Session = hWlx as PWLSESSION;
    (*Session).DialogTimeout = Timeout;
    1 // TRUE
}

fn WlxAssignShellProtection(_hWlx: HANDLE, _hToken: HANDLE, _hProcess: HANDLE, _hThread: HANDLE) -> i32 {
    0 // UNIMPLEMENTED
}

fn WlxMessageBox(_hWlx: HANDLE, hwndOwner: HWND, lpszText: LPWSTR, lpszTitle: LPWSTR, fuStyle: UINT) -> i32 {
    MessageBoxW(hwndOwner, lpszText, lpszTitle, fuStyle)
}

fn WlxDialogBox(hWlx: HANDLE, hInst: HANDLE, lpszTemplate: LPWSTR, hwndOwner: HWND, dlgprc: DLGPROC) -> i32 {
    WlxDialogBoxParam(hWlx, hInst, lpszTemplate, hwndOwner, dlgprc, 0) as i32
}

fn WlxDialogBoxParam(
    _hWlx: HANDLE,
    hInst: HANDLE,
    lpszTemplate: LPWSTR,
    hwndOwner: HWND,
    dlgprc: DLGPROC,
    dwInitParam: LPARAM,
) -> i32 {
    let ListEntry = AddDialogListEntry();
    if ListEntry.is_null() {
        return -1;
    }

    (*ListEntry).DlgProc = dlgprc;
    (*ListEntry).lParam = dwInitParam;

    let ret = DialogBoxParamW(
        hInst as HINSTANCE,
        lpszTemplate,
        hwndOwner,
        Some(DefaultWlxWindowProc),
        ListEntry as LPARAM,
    ) as i32;

    RemoveDialogListEntry(ListEntry);

    ret
}

fn WlxDialogBoxIndirect(hWlx: HANDLE, hInst: HANDLE, hDialogTemplate: LPCDLGTEMPLATEW, hwndOwner: HWND, dlgprc: DLGPROC) -> i32 {
    WlxDialogBoxIndirectParam(hWlx, hInst, hDialogTemplate, hwndOwner, dlgprc, 0) as i32
}

fn WlxDialogBoxIndirectParam(
    _hWlx: HANDLE,
    hInst: HANDLE,
    hDialogTemplate: LPCDLGTEMPLATEW,
    hwndOwner: HWND,
    dlgprc: DLGPROC,
    dwInitParam: LPARAM,
) -> i32 {
    let ListEntry = AddDialogListEntry();
    if ListEntry.is_null() {
        return -1;
    }

    (*ListEntry).DlgProc = dlgprc;
    (*ListEntry).lParam = dwInitParam;

    let ret = DialogBoxIndirectParamW(
        hInst as HINSTANCE,
        hDialogTemplate,
        hwndOwner,
        Some(DefaultWlxWindowProc),
        ListEntry as LPARAM,
    ) as i32;

    RemoveDialogListEntry(ListEntry);

    ret
}

fn WlxSwitchDesktopToUser(hWlx: HANDLE) -> i32 {
    let Session = hWlx as PWLSESSION;
    let bRet = SwitchDesktop((*Session).ApplicationDesktop);
    if bRet != 0 {
        SetThreadDesktop((*Session).ApplicationDesktop);
    }
    bRet
}

fn WlxSwitchDesktopToWinlogon(hWlx: HANDLE) -> i32 {
    let Session = hWlx as PWLSESSION;
    let bRet = SwitchDesktop((*Session).WinlogonDesktop);
    if bRet != 0 {
        SetThreadDesktop((*Session).WinlogonDesktop);
    }
    bRet
}

fn WlxChangePasswordNotify(_hWlx: HANDLE, _pMprInfo: PWLX_MPR_NOTIFY_INFO, _dwChangeInfo: DWORD) -> i32 {
    0 // UNIMPLEMENTED
}

fn WlxGetSourceDesktop(_hWlx: HANDLE, _ppDesktop: *mut PWLX_DESKTOP) -> BOOL {
    0 // UNIMPLEMENTED / FALSE
}

fn WlxSetReturnDesktop(_hWlx: HANDLE, _pDesktop: PWLX_DESKTOP) -> BOOL {
    0
}

fn WlxCreateUserDesktop(_hWlx: HANDLE, _hToken: HANDLE, _Flags: DWORD, _pszDesktopName: PWSTR, _ppDesktop: *mut PWLX_DESKTOP) -> BOOL {
    0
}

fn WlxChangePasswordNotifyEx(_hWlx: HANDLE, _pMprInfo: PWLX_MPR_NOTIFY_INFO, _dwChangeInfo: DWORD, _ProviderName: PWSTR, _Reserved: PVOID) -> i32 {
    0
}

fn WlxCloseUserDesktop(_hWlx: HANDLE, _pDesktop: PWLX_DESKTOP, _hToken: HANDLE) -> BOOL {
    0
}

fn WlxSetOption(hWlx: HANDLE, Option: DWORD, Value: ULONG_PTR, OldValue: *mut ULONG_PTR) -> BOOL {
    let Session = hWlx as PWLSESSION;
    match Option {
        WLX_OPTION_USE_CTRL_ALT_DEL => {
            *OldValue = (*Session).Gina.UseCtrlAltDelete as ULONG_PTR;
            (*Session).Gina.UseCtrlAltDelete = Value as BOOL;
            1 // TRUE
        }
        WLX_OPTION_CONTEXT_POINTER => {
            *OldValue = (*Session).Gina.Context as ULONG_PTR;
            (*Session).Gina.Context = Value as PVOID;
            1
        }
        // WLX_OPTION_USE_SMART_CARD → UNIMPLEMENTED
        _ => 0, // FALSE
    }
}

fn WlxGetOption(hWlx: HANDLE, Option: DWORD, Value: *mut ULONG_PTR) -> BOOL {
    let Session = hWlx as PWLSESSION;
    match Option {
        WLX_OPTION_USE_CTRL_ALT_DEL => {
            *Value = (*Session).Gina.UseCtrlAltDelete as ULONG_PTR;
            1
        }
        WLX_OPTION_CONTEXT_POINTER => {
            *Value = (*Session).Gina.Context as ULONG_PTR;
            1
        }
        WLX_OPTION_DISPATCH_TABLE_SIZE => {
            *Value = match (*Session).Gina.Version {
                WLX_VERSION_1_0 => size_of::<WLX_DISPATCH_VERSION_1_0>() as ULONG_PTR,
                WLX_VERSION_1_1 => size_of::<WLX_DISPATCH_VERSION_1_1>() as ULONG_PTR,
                WLX_VERSION_1_2 => size_of::<WLX_DISPATCH_VERSION_1_2>() as ULONG_PTR,
                WLX_VERSION_1_3 => size_of::<WLX_DISPATCH_VERSION_1_3>() as ULONG_PTR,
                WLX_VERSION_1_4 => size_of::<WLX_DISPATCH_VERSION_1_4>() as ULONG_PTR,
                _ => return 0, // FALSE
            };
            1
        }
        // smart-card options → UNIMPLEMENTED
        _ => 0,
    }
}

fn WlxWin31Migrate(_hWlx: HANDLE) {
    // UNIMPLEMENTED
}

fn WlxQueryClientCredentials(_pCred: PWLX_CLIENT_CREDENTIALS_INFO_V1_0) -> BOOL {
    0
}

fn WlxQueryInetConnectorCredentials(_pCred: PWLX_CLIENT_CREDENTIALS_INFO_V1_0) -> BOOL {
    0
}

fn WlxDisconnect() -> BOOL {
    0
}

fn WlxQueryTerminalServicesData(_hWlx: HANDLE, _pTSData: PWLX_TERMINAL_SERVICES_DATA, _UserName: *mut WCHAR, _Domain: *mut WCHAR) -> DWORD {
    0
}

fn WlxQueryConsoleSwitchCredentials(_pCred: PWLX_CONSOLESWITCH_CREDENTIALS_INFO_V1_0) -> DWORD {
    0
}

fn WlxQueryTsLogonCredentials(_pCred: PWLX_CLIENT_CREDENTIALS_INFO_V2_0) -> BOOL {
    0
}

fn DefaultWlxScreenSaverNotify(pWlxContext: PVOID, pSecure: *mut BOOL) -> BOOL {
    if *pSecure != 0 {
        *pSecure = (*WLSession).Gina.Functions.WlxIsLogoffOk.unwrap()(pWlxContext);
    }
    1 // TRUE
}

} // winapi_fns!

static FunctionTable: WLX_DISPATCH_VERSION_1_4 = WLX_DISPATCH_VERSION_1_4 {
    WlxUseCtrlAltDel: Some(WlxUseCtrlAltDel),
    WlxSetContextPointer: Some(WlxSetContextPointer),
    WlxSasNotify: Some(WlxSasNotify),
    WlxSetTimeout: Some(WlxSetTimeout),
    WlxAssignShellProtection: Some(WlxAssignShellProtection),
    WlxMessageBox: Some(WlxMessageBox),
    WlxDialogBox: Some(WlxDialogBox),
    WlxDialogBoxParam: Some(WlxDialogBoxParam),
    WlxDialogBoxIndirect: Some(WlxDialogBoxIndirect),
    WlxDialogBoxIndirectParam: Some(WlxDialogBoxIndirectParam),
    WlxSwitchDesktopToUser: Some(WlxSwitchDesktopToUser),
    WlxSwitchDesktopToWinlogon: Some(WlxSwitchDesktopToWinlogon),
    WlxChangePasswordNotify: Some(WlxChangePasswordNotify),
    WlxGetSourceDesktop: Some(WlxGetSourceDesktop),
    WlxSetReturnDesktop: Some(WlxSetReturnDesktop),
    WlxCreateUserDesktop: Some(WlxCreateUserDesktop),
    WlxChangePasswordNotifyEx: Some(WlxChangePasswordNotifyEx),
    WlxCloseUserDesktop: Some(WlxCloseUserDesktop),
    WlxSetOption: Some(WlxSetOption),
    WlxGetOption: Some(WlxGetOption),
    WlxWin31Migrate: Some(WlxWin31Migrate),
    WlxQueryClientCredentials: Some(WlxQueryClientCredentials),
    WlxQueryInetConnectorCredentials: Some(WlxQueryInetConnectorCredentials),
    WlxDisconnect: Some(WlxDisconnect),
    WlxQueryTerminalServicesData: Some(WlxQueryTerminalServicesData),
    WlxQueryConsoleSwitchCredentials: Some(WlxQueryConsoleSwitchCredentials),
    WlxQueryTsLogonCredentials: Some(WlxQueryTsLogonCredentials),
};

// --- GINA loader ------------------------------------------------------------

unsafe fn GetGinaPath(Path: LPWSTR, Len: DWORD) -> BOOL {
    let mut hKey: HKEY = null_mut();

    let Status = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wstr!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon").as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    );
    if Status != ERROR_SUCCESS as LONG {
        // Default value
        wcsncpy(Path, wstr!("msgina.dll").as_ptr(), Len as usize);
        return 1; // TRUE
    }

    let mut Type: DWORD = 0;
    let mut Size: DWORD = Len * size_of::<WCHAR>() as DWORD;
    let Status = RegQueryValueExW(
        hKey,
        wstr!("GinaDLL").as_ptr(),
        null_mut(),
        &mut Type,
        Path as LPBYTE,
        &mut Size,
    );
    if Status != ERROR_SUCCESS as LONG || Type != REG_SZ || Size == 0 {
        wcsncpy(Path, wstr!("msgina.dll").as_ptr(), Len as usize);
    }
    RegCloseKey(hKey);
    1 // TRUE
}

unsafe fn proc_addr(hGina: HMODULE, name: *const u8) -> FARPROC {
    GetProcAddress(hGina, name as *const i8)
}

unsafe fn LoadGina(
    Functions: PGINAFUNCTIONS,
    DllVersion: *mut DWORD,
    GinaInstance: *mut HMODULE,
) -> BOOL {
    let mut hGina: HMODULE = null_mut();
    let mut GinaDll: [u16; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let mut ret: BOOL = 0; // FALSE

    GinaDll[0] = 0;
    'cleanup: {
        if GetGinaPath(GinaDll.as_mut_ptr(), MAX_PATH) == 0 {
            break 'cleanup;
        }
        GinaDll[MAX_PATH as usize] = 0;

        hGina = LoadLibraryW(GinaDll.as_ptr());
        if hGina.is_null() {
            break 'cleanup;
        }

        (*Functions).WlxNegotiate = transmute(proc_addr(hGina, b"WlxNegotiate\0".as_ptr()));
        (*Functions).WlxInitialize = transmute(proc_addr(hGina, b"WlxInitialize\0".as_ptr()));

        if (*Functions).WlxInitialize.is_none() {
            break 'cleanup;
        }

        if (*Functions).WlxNegotiate.is_none() {
            // Assume current version
            *DllVersion = WLX_CURRENT_VERSION;
        } else if (*Functions).WlxNegotiate.unwrap()(WLX_CURRENT_VERSION, DllVersion) == 0 {
            break 'cleanup;
        }

        if *DllVersion >= WLX_VERSION_1_0 {
            macro_rules! load_required {
                ($field:ident, $sym:literal) => {{
                    (*Functions).$field = transmute(proc_addr(hGina, concat!($sym, "\0").as_ptr()));
                    if (*Functions).$field.is_none() {
                        break 'cleanup;
                    }
                }};
            }
            load_required!(WlxActivateUserShell, "WlxActivateUserShell");
            load_required!(WlxDisplayLockedNotice, "WlxDisplayLockedNotice");
            load_required!(WlxDisplaySASNotice, "WlxDisplaySASNotice");
            load_required!(WlxIsLockOk, "WlxIsLockOk");
            load_required!(WlxIsLogoffOk, "WlxIsLogoffOk");
            load_required!(WlxLoggedOnSAS, "WlxLoggedOnSAS");
            load_required!(WlxLoggedOutSAS, "WlxLoggedOutSAS");
            load_required!(WlxLogoff, "WlxLogoff");
            load_required!(WlxShutdown, "WlxShutdown");
            load_required!(WlxWkstaLockedSAS, "WlxWkstaLockedSAS");
        }

        if *DllVersion >= WLX_VERSION_1_1 {
            (*Functions).WlxScreenSaverNotify = transmute(proc_addr(hGina, b"WlxScreenSaverNotify\0".as_ptr()));
            (*Functions).WlxStartApplication = transmute(proc_addr(hGina, b"WlxStartApplication\0".as_ptr()));
        }

        if *DllVersion >= WLX_VERSION_1_3 {
            macro_rules! load_required {
                ($field:ident, $sym:literal) => {{
                    (*Functions).$field = transmute(proc_addr(hGina, concat!($sym, "\0").as_ptr()));
                    if (*Functions).$field.is_none() {
                        break 'cleanup;
                    }
                }};
            }
            load_required!(WlxDisplayStatusMessage, "WlxDisplayStatusMessage");
            load_required!(WlxGetStatusMessage, "WlxGetStatusMessage");
            load_required!(WlxNetworkProviderLoad, "WlxNetworkProviderLoad");
            load_required!(WlxRemoveStatusMessage, "WlxRemoveStatusMessage");
        }

        // Provide some default functions
        if (*Functions).WlxScreenSaverNotify.is_none() {
            (*Functions).WlxScreenSaverNotify = Some(DefaultWlxScreenSaverNotify);
        }

        ret = 1; // TRUE
    }

    // cleanup:
    if ret == 0 {
        if !hGina.is_null() {
            FreeLibrary(hGina);
        }
    } else {
        *GinaInstance = hGina;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn GinaInit(Session: PWLSESSION) -> BOOL {
    let mut GinaDllVersion: DWORD = 0;

    if LoadGina(
        addr_of_mut!((*Session).Gina.Functions),
        &mut GinaDllVersion,
        addr_of_mut!((*Session).Gina.hDllInstance),
    ) == 0
    {
        return 0; // FALSE
    }

    (*Session).Gina.Context = null_mut();
    (*Session).Gina.Version = GinaDllVersion;
    (*Session).Gina.UseCtrlAltDelete = 0; // FALSE
    (*Session).SuppressStatus = 0; // FALSE

    (*Session).Gina.Functions.WlxInitialize.unwrap()(
        (*Session).InteractiveWindowStationName,
        Session as HANDLE,
        null_mut(),
        addr_of!(FunctionTable) as PVOID,
        addr_of_mut!((*Session).Gina.Context),
    )
}

#[no_mangle]
pub unsafe extern "C" fn CreateWindowStationAndDesktops(Session: PWLSESSION) -> BOOL {
    let mut ret: BOOL = 0; // FALSE

    let mut WinstaSecurity: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut ApplicationDesktopSecurity: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut WinlogonDesktopSecurity: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut ScreenSaverDesktopSecurity: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut WlWinstaSD: PSECURITY_DESCRIPTOR = null_mut();
    let mut WlApplicationDesktopSD: PSECURITY_DESCRIPTOR = null_mut();
    let mut WlWinlogonDesktopSD: PSECURITY_DESCRIPTOR = null_mut();
    let mut WlScreenSaverDesktopSD: PSECURITY_DESCRIPTOR = null_mut();

    'cleanup: {
        {
            if crate::security::CreateWinstaSecurity(&mut WlWinstaSD) == 0 {
                return ret;
            }
            WinstaSecurity.nLength = size_of::<SECURITY_ATTRIBUTES>() as DWORD;
            WinstaSecurity.lpSecurityDescriptor = WlWinstaSD;
            WinstaSecurity.bInheritHandle = 1; // TRUE

            if crate::security::CreateApplicationDesktopSecurity(&mut WlApplicationDesktopSD) == 0 {
                break 'cleanup;
            }
            ApplicationDesktopSecurity.nLength = size_of::<SECURITY_ATTRIBUTES>() as DWORD;
            ApplicationDesktopSecurity.lpSecurityDescriptor = WlApplicationDesktopSD;
            ApplicationDesktopSecurity.bInheritHandle = 1;

            if crate::security::CreateWinlogonDesktopSecurity(&mut WlWinlogonDesktopSD) == 0 {
                break 'cleanup;
            }
            WinlogonDesktopSecurity.nLength = size_of::<SECURITY_ATTRIBUTES>() as DWORD;
            WinlogonDesktopSecurity.lpSecurityDescriptor = WlWinlogonDesktopSD;
            WinlogonDesktopSecurity.bInheritHandle = 0; // FALSE

            if crate::security::CreateScreenSaverSecurity(&mut WlScreenSaverDesktopSD) == 0 {
                break 'cleanup;
            }
            ScreenSaverDesktopSecurity.nLength = size_of::<SECURITY_ATTRIBUTES>() as DWORD;
            ScreenSaverDesktopSecurity.lpSecurityDescriptor = WlScreenSaverDesktopSD;
            ScreenSaverDesktopSecurity.bInheritHandle = 1;
        }

        // Create the interactive window station
        (*Session).InteractiveWindowStationName = wstr!("WinSta0").as_ptr() as PWSTR;
        let winsta_sa: *mut SECURITY_ATTRIBUTES = &mut WinstaSecurity;
        (*Session).InteractiveWindowStation =
            CreateWindowStationW((*Session).InteractiveWindowStationName, 0, MAXIMUM_ALLOWED, winsta_sa);
        if (*Session).InteractiveWindowStation.is_null() {
            break 'cleanup;
        }

        if SetProcessWindowStation((*Session).InteractiveWindowStation) == 0 {
            break 'cleanup;
        }

        let (app_sa, wl_sa, ss_sa): (
            *mut SECURITY_ATTRIBUTES,
            *mut SECURITY_ATTRIBUTES,
            *mut SECURITY_ATTRIBUTES,
        ) = (
            &mut ApplicationDesktopSecurity,
            &mut WinlogonDesktopSecurity,
            &mut ScreenSaverDesktopSecurity,
        );

        // Create the application desktop
        (*Session).ApplicationDesktop = CreateDesktopW(
            wstr!("Default").as_ptr(),
            null_mut(),
            null_mut(),
            0,
            MAXIMUM_ALLOWED,
            app_sa,
        );
        if (*Session).ApplicationDesktop.is_null() {
            break 'cleanup;
        }

        // Create the winlogon desktop
        (*Session).WinlogonDesktop = CreateDesktopW(
            wstr!("Winlogon").as_ptr(),
            null_mut(),
            null_mut(),
            0,
            MAXIMUM_ALLOWED,
            wl_sa,
        );
        if (*Session).WinlogonDesktop.is_null() {
            break 'cleanup;
        }

        {
            // Create the screen saver desktop
            (*Session).ScreenSaverDesktop = CreateDesktopW(
                wstr!("Screen-Saver").as_ptr(),
                null_mut(),
                null_mut(),
                0,
                MAXIMUM_ALLOWED,
                ss_sa,
            );
            if (*Session).ScreenSaverDesktop.is_null() {
                break 'cleanup;
            }

            // Switch to winlogon desktop
            if SetThreadDesktop((*Session).WinlogonDesktop) == 0
                || SwitchDesktop((*Session).WinlogonDesktop) == 0
            {
                break 'cleanup;
            }
        }

        SetWindowStationUser(
            (*Session).InteractiveWindowStation,
            addr_of!(LuidNone) as PLUID,
            null_mut(),
            0,
        );

        ret = 1; // TRUE
    }

    // cleanup:
    if ret == 0 {
        if !(*Session).ApplicationDesktop.is_null() {
            CloseDesktop((*Session).ApplicationDesktop);
            (*Session).ApplicationDesktop = null_mut();
        }
        if !(*Session).WinlogonDesktop.is_null() {
            CloseDesktop((*Session).WinlogonDesktop);
            (*Session).WinlogonDesktop = null_mut();
        }
        if !(*Session).ScreenSaverDesktop.is_null() {
            CloseDesktop((*Session).ScreenSaverDesktop);
            (*Session).ScreenSaverDesktop = null_mut();
        }
        if !(*Session).InteractiveWindowStation.is_null() {
            CloseWindowStation((*Session).InteractiveWindowStation);
            (*Session).InteractiveWindowStation = null_mut();
        }
    }

    {
        // Free security descriptors regardless of success or failure
        if !WlWinstaSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, WlWinstaSD);
        }
        if !WlApplicationDesktopSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, WlApplicationDesktopSD);
        }
        if !WlWinlogonDesktopSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, WlWinlogonDesktopSD);
        }
        if !WlScreenSaverDesktopSD.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, WlScreenSaverDesktopSD);
        }
    }

    ret
}
