use crate::taskmgr::taskmgr::ConfirmMessageBox;
use crate::taskmgr::taskmgr::GetLastErrorText;
use crate::taskmgr::procpage::GetSelectedProcessId;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn ProcessPage_OnEndProcess() {
    let mut dwProcessId: DWORD;
    let mut hProcess: HANDLE;
    let mut szTitle: [WCHAR; 256] = [0; 256];
    let mut strErrorText: [WCHAR; 260] = [0; 260];

    dwProcessId = GetSelectedProcessId();

    if dwProcessId == 0 {
        return;
    }

    hProcess = OpenProcess(PROCESS_TERMINATE | PROCESS_QUERY_INFORMATION, (FALSE) as BOOL, dwProcessId);

    /* forbid killing system processes even if we have privileges -- sigh, windows kludge! */
    if !hProcess.is_null() && IsCriticalProcess(hProcess) != 0 {
        LoadStringW(hInst, IDS_MSG_UNABLETERMINATEPRO, szTitle.as_mut_ptr(), 256);
        LoadStringW(hInst, IDS_MSG_CLOSESYSTEMPROCESS, strErrorText.as_mut_ptr(), 256);
        MessageBoxW(hMainWnd, strErrorText.as_ptr(), szTitle.as_ptr(), MB_OK | MB_ICONWARNING | MB_TOPMOST);
        CloseHandle(hProcess);
        return;
    }

    /* if this is a standard process just ask for confirmation before doing it */
    LoadStringW(hInst, IDS_MSG_WARNINGTERMINATING, strErrorText.as_mut_ptr(), 256);
    LoadStringW(hInst, IDS_MSG_TASKMGRWARNING, szTitle.as_mut_ptr(), 256);
    if ConfirmMessageBox(hMainWnd, strErrorText.as_ptr(), szTitle.as_ptr(), MB_YESNO | MB_ICONWARNING | MB_TOPMOST) == 0 {
        if !hProcess.is_null() {
            CloseHandle(hProcess);
        }
        return;
    }

    /* no such process or not enough privileges to open its token */
    if hProcess.is_null() {
        GetLastErrorText(strErrorText.as_mut_ptr(), 260);
        LoadStringW(hInst, IDS_MSG_UNABLETERMINATEPRO, szTitle.as_mut_ptr(), 256);
        MessageBoxW(hMainWnd, strErrorText.as_ptr(), szTitle.as_ptr(), MB_OK | MB_ICONSTOP | MB_TOPMOST);
        return;
    }

    /* try to kill it, and notify the user if didn't work */
    if TerminateProcess(hProcess, 1) == 0 {
        GetLastErrorText(strErrorText.as_mut_ptr(), 260);
        LoadStringW(hInst, IDS_MSG_UNABLETERMINATEPRO, szTitle.as_mut_ptr(), 256);
        MessageBoxW(hMainWnd, strErrorText.as_ptr(), szTitle.as_ptr(), MB_OK | MB_ICONSTOP | MB_TOPMOST);
    }

    CloseHandle(hProcess);
}
#[no_mangle]
pub unsafe extern "C" fn IsCriticalProcess(hProcess: HANDLE) -> BOOL {
    let mut status: NTSTATUS;
    let mut BreakOnTermination: ULONG = 0;

    /* return early if the process handle does not exist */
    if hProcess.is_null() {
        return (FALSE) as BOOL;
    }

    /* the important system processes that we don't want to let the user
       kill come marked as critical, this simplifies the check greatly.

       a critical process brings the system down when is terminated:
       <http://www.geoffchappell.com/studies/windows/win32/ntdll/api/rtl/peb/setprocessiscritical.htm> */

    status = NtQueryInformationProcess(
        hProcess,
        ProcessBreakOnTermination,
        core::ptr::addr_of_mut!(BreakOnTermination) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );

    if NT_SUCCESS!(status) && BreakOnTermination != 0 {
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ShutdownProcessTreeHelper(hSnapshot: HANDLE, hParentProcess: HANDLE, dwParentPID: DWORD) -> BOOL {
    let mut hChildHandle: HANDLE;
    let mut ProcessEntry: PROCESSENTRY32W = core::mem::zeroed();
    ProcessEntry.dwSize = core::mem::size_of::<PROCESSENTRY32W>() as DWORD;

    if Process32FirstW(hSnapshot, core::ptr::addr_of_mut!(ProcessEntry)) != 0 {
        loop {
            'body: {
                if ProcessEntry.th32ParentProcessID == dwParentPID {
                    hChildHandle = OpenProcess(
                        (PROCESS_TERMINATE | PROCESS_QUERY_INFORMATION) as DWORD,
                        (FALSE) as BOOL,
                        ProcessEntry.th32ProcessID,
                    );
                    if hChildHandle.is_null() || IsCriticalProcess(hChildHandle) != 0 {
                        if !hChildHandle.is_null() {
                            CloseHandle(hChildHandle);
                        }
                        break 'body;
                    }
                    if ShutdownProcessTreeHelper(hSnapshot, hChildHandle, ProcessEntry.th32ProcessID) == 0 {
                        CloseHandle(hChildHandle);
                        return (FALSE) as BOOL;
                    }
                    CloseHandle(hChildHandle);
                }
            }
            if !(Process32NextW(hSnapshot, core::ptr::addr_of_mut!(ProcessEntry)) != 0) {
                break;
            }
        }
    }

    return TerminateProcess(hParentProcess, 0 as UINT);
}
#[no_mangle]
pub unsafe extern "C" fn ShutdownProcessTree(hParentProcess: HANDLE, dwParentPID: DWORD) -> BOOL {
    let hSnapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS as DWORD, 0 as DWORD);
    let mut bResult: BOOL;

    if hSnapshot.is_null() {
        return (FALSE) as BOOL;
    }

    bResult = ShutdownProcessTreeHelper(hSnapshot, hParentProcess, dwParentPID);
    CloseHandle(hSnapshot);
    return bResult;
}
#[no_mangle]
pub unsafe extern "C" fn ProcessPage_OnEndProcessTree() {
    let mut dwProcessId: DWORD = 0;
    let mut hProcess: HANDLE = core::ptr::null_mut();
    let mut szTitle: [WCHAR; 256] = [0; 256];
    let mut strErrorText: [WCHAR; 260] = [0; 260];

    dwProcessId = GetSelectedProcessId();

    if dwProcessId == 0 {
        return;
    }

    hProcess = OpenProcess(PROCESS_TERMINATE | PROCESS_QUERY_INFORMATION, (FALSE) as BOOL, dwProcessId);

    /* forbid killing system processes even if we have privileges -- sigh, windows kludge! */
    if !hProcess.is_null() && IsCriticalProcess(hProcess) != 0 {
        LoadStringW(hInst, IDS_MSG_UNABLETERMINATEPRO, core::ptr::addr_of_mut!(szTitle) as LPWSTR, 256);
        LoadStringW(hInst, IDS_MSG_CLOSESYSTEMPROCESS, core::ptr::addr_of_mut!(strErrorText) as LPWSTR, 256);
        MessageBoxW(hMainWnd, core::ptr::addr_of_mut!(strErrorText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, MB_OK | MB_ICONWARNING | MB_TOPMOST);
        CloseHandle(hProcess);
        return;
    }

    LoadStringW(hInst, IDS_MSG_WARNINGTERMINATING, core::ptr::addr_of_mut!(strErrorText) as LPWSTR, 256);
    LoadStringW(hInst, IDS_MSG_TASKMGRWARNING, core::ptr::addr_of_mut!(szTitle) as LPWSTR, 256);
    if ConfirmMessageBox(hMainWnd, core::ptr::addr_of_mut!(strErrorText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, MB_YESNO | MB_ICONWARNING) == 0
    {
        if !hProcess.is_null() {
            CloseHandle(hProcess);
        }
        return;
    }

    if hProcess.is_null() {
        GetLastErrorText(core::ptr::addr_of_mut!(strErrorText) as LPTSTR, 260);
        LoadStringW(hInst, IDS_MSG_UNABLETERMINATEPRO, core::ptr::addr_of_mut!(szTitle) as LPWSTR, 256);
        MessageBoxW(hMainWnd, core::ptr::addr_of_mut!(strErrorText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, MB_OK | MB_ICONSTOP);
        return;
    }

    if ShutdownProcessTree(hProcess, dwProcessId) == 0 {
        GetLastErrorText(core::ptr::addr_of_mut!(strErrorText) as LPTSTR, 260);
        LoadStringW(hInst, IDS_MSG_UNABLETERMINATEPRO, core::ptr::addr_of_mut!(szTitle) as LPWSTR, 256);
        MessageBoxW(hMainWnd, core::ptr::addr_of_mut!(strErrorText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, MB_OK | MB_ICONSTOP);
    }

    CloseHandle(hProcess);
}
