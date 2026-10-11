use crate::taskmgr::taskmgr::ConfirmMessageBox;
use crate::taskmgr::taskmgr::GetLastErrorText;
use crate::taskmgr::procpage::GetSelectedProcessId;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_D7CB07DBF8E2: [u16; 9] = [0x0044u16, 0x0065, 0x0062, 0x0075, 0x0067, 0x0067, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_DEB4EFA22C30: [u16; 53] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0041, 0x0065, 0x0044, 0x0065, 0x0062,
    0x0075, 0x0067, 0x0000,
];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn ProcessPage_OnDebug() {
    let mut dwProcessId: DWORD = core::mem::zeroed();
    let mut strErrorText: [WCHAR; 260] = core::mem::zeroed();
    let mut hKey: HKEY = core::mem::zeroed();
    let mut strDebugPath: [WCHAR; 260] = core::mem::zeroed();
    let mut strDebugger: [WCHAR; 260] = core::mem::zeroed();
    let mut dwDebuggerSize: DWORD = core::mem::zeroed();
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut si: STARTUPINFOW = core::mem::zeroed();
    let mut hDebugEvent: HANDLE = core::mem::zeroed();
    let mut szTemp: [WCHAR; 256] = core::mem::zeroed();
    let mut szTempA: [WCHAR; 256] = core::mem::zeroed();

    dwProcessId = GetSelectedProcessId();

    if dwProcessId == 0 {
        return;
    }

    LoadStringW(
        hInst,
        IDS_MSG_WARNINGDEBUG as UINT,
        &raw mut szTemp as LPWSTR,
        _countof!(szTemp) as ::core::ffi::c_int,
    );
    LoadStringW(
        hInst,
        IDS_MSG_TASKMGRWARNING as UINT,
        &raw mut szTempA as LPWSTR,
        _countof!(szTempA) as ::core::ffi::c_int,
    );

    if ConfirmMessageBox(
        hMainWnd,
        &raw mut szTemp as *mut WCHAR as LPCWSTR,
        &raw mut szTempA as *mut WCHAR as LPCWSTR,
        (MB_YESNO | MB_ICONWARNING) as UINT,
    ) == 0
    {
        GetLastErrorText(
            &raw mut strErrorText as LPTSTR,
            _countof!(strErrorText) as DWORD,
        );
        LoadStringW(
            hInst,
            IDS_MSG_UNABLEDEBUGPROCESS as UINT,
            &raw mut szTemp as LPWSTR,
            _countof!(szTemp) as ::core::ffi::c_int,
        );
        MessageBoxW(
            hMainWnd,
            &raw mut strErrorText as *mut WCHAR as LPCWSTR,
            &raw mut szTemp as *mut WCHAR as LPCWSTR,
            (MB_OK | MB_ICONSTOP) as UINT,
        );
        return;
    }

    if (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_DEB4EFA22C30.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &raw mut hKey,
    )) as u32 != ERROR_SUCCESS
    {
        GetLastErrorText(
            &raw mut strErrorText as LPTSTR,
            _countof!(strErrorText) as DWORD,
        );
        LoadStringW(
            hInst,
            IDS_MSG_UNABLEDEBUGPROCESS as UINT,
            &raw mut szTemp as LPWSTR,
            _countof!(szTemp) as ::core::ffi::c_int,
        );
        MessageBoxW(
            hMainWnd,
            &raw mut strErrorText as *mut WCHAR as LPCWSTR,
            &raw mut szTemp as *mut WCHAR as LPCWSTR,
            (MB_OK | MB_ICONSTOP) as UINT,
        );
        return;
    }

    dwDebuggerSize = core::mem::size_of::<[WCHAR; 260]>() as DWORD;
    if (RegQueryValueExW(
        hKey,
        RUSTCONV_WSTR_D7CB07DBF8E2.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut strDebugger as *mut WCHAR as LPBYTE,
        &raw mut dwDebuggerSize,
    )) as u32 != ERROR_SUCCESS
    {
        GetLastErrorText(
            &raw mut strErrorText as LPTSTR,
            _countof!(strErrorText) as DWORD,
        );
        LoadStringW(
            hInst,
            IDS_MSG_UNABLEDEBUGPROCESS as UINT,
            &raw mut szTemp as LPWSTR,
            _countof!(szTemp) as ::core::ffi::c_int,
        );
        MessageBoxW(
            hMainWnd,
            &raw mut strErrorText as *mut WCHAR as LPCWSTR,
            &raw mut szTemp as *mut WCHAR as LPCWSTR,
            (MB_OK | MB_ICONSTOP) as UINT,
        );
        RegCloseKey(hKey);
        return;
    }

    RegCloseKey(hKey);

    hDebugEvent = CreateEventW(
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null_mut(),
    );
    if hDebugEvent.is_null() {
        GetLastErrorText(
            &raw mut strErrorText as LPTSTR,
            _countof!(strErrorText) as DWORD,
        );
        LoadStringW(
            hInst,
            IDS_MSG_UNABLEDEBUGPROCESS as UINT,
            &raw mut szTemp as LPWSTR,
            _countof!(szTemp) as ::core::ffi::c_int,
        );
        MessageBoxW(
            hMainWnd,
            &raw mut strErrorText as *mut WCHAR as LPCWSTR,
            &raw mut szTemp as *mut WCHAR as LPCWSTR,
            (MB_OK | MB_ICONSTOP) as UINT,
        );
        return;
    }

    wsprintfW(
        &raw mut strDebugPath as LPWSTR,
        &raw mut strDebugger as *mut WCHAR as LPCWSTR,
        dwProcessId,
        hDebugEvent,
    );

    ZeroMemory!(
        &raw mut pi as *mut ::core::ffi::c_void,
        core::mem::size_of::<PROCESS_INFORMATION>(),
    );
    ZeroMemory!(
        &raw mut si as *mut ::core::ffi::c_void,
        core::mem::size_of::<STARTUPINFOW>(),
    );
    si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
    if CreateProcessW(
        core::ptr::null_mut(),
        &raw mut strDebugPath as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        0 as DWORD,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &raw mut si,
        &raw mut pi,
    ) == 0
    {
        GetLastErrorText(
            &raw mut strErrorText as LPTSTR,
            _countof!(strErrorText) as DWORD,
        );
        LoadStringW(
            hInst,
            IDS_MSG_UNABLEDEBUGPROCESS as UINT,
            &raw mut szTemp as LPWSTR,
            _countof!(szTemp) as ::core::ffi::c_int,
        );
        MessageBoxW(
            hMainWnd,
            &raw mut strErrorText as *mut WCHAR as LPCWSTR,
            &raw mut szTemp as *mut WCHAR as LPCWSTR,
            (MB_OK | MB_ICONSTOP) as UINT,
        );
    } else {
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
    }

    CloseHandle(hDebugEvent);
}
