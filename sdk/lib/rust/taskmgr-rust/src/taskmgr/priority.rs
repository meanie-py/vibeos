use crate::taskmgr::taskmgr::ConfirmMessageBox;
use crate::taskmgr::taskmgr::GetLastErrorText;
use crate::taskmgr::procpage::GetSelectedProcessId;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn DoSetPriority(priority: DWORD) {
    let mut dwProcessId: DWORD;
    let mut hProcess: HANDLE;
    let mut szText: [WCHAR; 260] = [0; 260];
    let mut szTitle: [WCHAR; 256] = [0; 256];

    dwProcessId = GetSelectedProcessId();

    if dwProcessId == 0 {
        return;
    }

    LoadStringW(hInst, IDS_MSG_TASKMGRWARNING, core::ptr::addr_of_mut!(szTitle) as LPWSTR, 256);
    LoadStringW(hInst, IDS_MSG_WARNINGCHANGEPRIORITY, core::ptr::addr_of_mut!(szText) as LPWSTR, 260);
    if ConfirmMessageBox(hMainWnd, core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, (MB_YESNO | MB_ICONWARNING) as UINT) == 0 {
        return;
    }

    hProcess = OpenProcess(PROCESS_SET_INFORMATION as DWORD, (FALSE) as BOOL, dwProcessId);

    if hProcess.is_null() {
        GetLastErrorText(core::ptr::addr_of_mut!(szText) as LPTSTR, 260 as DWORD);
        LoadStringW(hInst, IDS_MSG_UNABLECHANGEPRIORITY, core::ptr::addr_of_mut!(szTitle) as LPWSTR, 256);
        MessageBoxW(hMainWnd, core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, (MB_OK | MB_ICONSTOP) as UINT);
        return;
    }

    if SetPriorityClass(hProcess, priority) == 0 {
        GetLastErrorText(core::ptr::addr_of_mut!(szText) as LPTSTR, 260 as DWORD);
        LoadStringW(hInst, IDS_MSG_UNABLECHANGEPRIORITY, core::ptr::addr_of_mut!(szTitle) as LPWSTR, 256);
        MessageBoxW(hMainWnd, core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, (MB_OK | MB_ICONSTOP) as UINT);
    }

    CloseHandle(hProcess);
}
