use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn GetProcessID(lpszProcessName: LPCWSTR) -> DWORD {
    let mut Process: PROCESSENTRY32W = core::mem::zeroed();

    /* Create a snapshot and check if the given process name matches with the one from the process entry structure */
    let hSnapshot: HANDLE = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

    if hSnapshot == INVALID_HANDLE_VALUE {
        return 0;
    }

    /* Get the whole size of the structure */
    Process.dwSize = core::mem::size_of::<PROCESSENTRY32W>() as DWORD;

    /* Enumerate the processes */
    if Process32FirstW(hSnapshot, core::ptr::addr_of_mut!(Process)) != 0 {
        loop {
            if _wcsicmp(
                core::ptr::addr_of_mut!(Process.szExeFile) as *mut WCHAR,
                lpszProcessName as *const wchar_t,
            ) == 0
            {
                /* The names match, return the process ID we're interested */
                CloseHandle(hSnapshot);
                return Process.th32ProcessID;
            }
            if !(Process32NextW(hSnapshot, core::ptr::addr_of_mut!(Process)) != 0) {
                break;
            }
        }
    }

    CloseHandle(hSnapshot);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn IsProcessRunning(lpszProcessName: LPCWSTR) -> BOOL {
    let mut dwReturn: DWORD;
    let mut dwProcessID: DWORD;
    let mut hProcess: HANDLE;

    /* Get the process ID */
    dwProcessID = GetProcessID(lpszProcessName);
    if dwProcessID == 0 {
        return (FALSE) as BOOL;
    }

    /* Synchronize the process to get its signaling state */
    hProcess = OpenProcess(SYNCHRONIZE, (FALSE) as BOOL, dwProcessID);
    if hProcess.is_null() {
        DPRINT!("IsProcessRunning(): Failed to open the process! (Error: %lu)\n", GetLastError());
        return (FALSE) as BOOL;
    }

    /* Wait for the process */
    dwReturn = WaitForSingleObject(hProcess, 0);
    if dwReturn == WAIT_TIMEOUT {
        /* The process is still running */
        CloseHandle(hProcess);
        return (TRUE) as BOOL;
    }

    CloseHandle(hProcess);
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn LaunchProcess(lpszProcessName: LPCWSTR) -> BOOL {
    let mut si: STARTUPINFOW = core::mem::zeroed();
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut hUserToken: HANDLE = core::ptr::null_mut();
    let mut hProcessToken: HANDLE = core::ptr::null_mut();
    let mut bSuccess: BOOL;
    let mut ExpandedCmdLine: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* Expand the process path string */
    ExpandEnvironmentStringsW(lpszProcessName, core::ptr::addr_of_mut!(ExpandedCmdLine) as LPWSTR, ARRAYSIZE!(ExpandedCmdLine));

    ZeroMemory!(core::ptr::addr_of_mut!(pi) as *mut core::ffi::c_void, core::mem::size_of::<PROCESS_INFORMATION>());
    ZeroMemory!(core::ptr::addr_of_mut!(si) as *mut core::ffi::c_void, core::mem::size_of::<STARTUPINFOW>());
    si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = (SW_SHOWNORMAL as u16);

    /* Get the token of the parent (current) process of the application */
    bSuccess = OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY | TOKEN_DUPLICATE, core::ptr::addr_of_mut!(hUserToken));
    if bSuccess == 0 {
        DPRINT!("OpenProcessToken() failed with error -> %lu\n", GetLastError());
        return (FALSE) as BOOL;
    }

    /* Duplicate a new token so that we can use it to create our process */
    bSuccess = DuplicateTokenEx(
        hUserToken,
        TOKEN_ALL_ACCESS,
        core::ptr::null_mut(),
        SecurityIdentification,
        TokenPrimary,
        core::ptr::addr_of_mut!(hProcessToken),
    );
    if bSuccess == 0 {
        DPRINT!("DuplicateTokenEx() failed with error -> %lu\n", GetLastError());
        CloseHandle(hUserToken);
        return (FALSE) as BOOL;
    }

    /* Finally create the process */
    bSuccess = CreateProcessAsUserW(
        hProcessToken,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ExpandedCmdLine) as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (FALSE) as BOOL,
        0, // DETACHED_PROCESS, NORMAL_PRIORITY_CLASS
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(si),
        core::ptr::addr_of_mut!(pi),
    );

    if bSuccess == 0 {
        DPRINT!("CreateProcessAsUserW() failed with error -> %lu\n", GetLastError());
        CloseHandle(hUserToken);
        CloseHandle(hProcessToken);
        return (FALSE) as BOOL;
    }

    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    CloseHandle(hUserToken);
    CloseHandle(hProcessToken);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn CloseProcess(lpszProcessName: LPCWSTR) -> BOOL {
    let mut hProcess: HANDLE = core::ptr::null_mut();
    let mut dwProcessID: DWORD = 0;

    /* Get the process ID */
    dwProcessID = GetProcessID(lpszProcessName);
    if dwProcessID == 0 {
        return (FALSE) as BOOL;
    }

    /* Make sure that the given process ID is not ours, the parent process, so that we do not kill ourselves */
    if dwProcessID == GetCurrentProcessId() {
        return (FALSE) as BOOL;
    }

    /* Open the process so that we can terminate it */
    hProcess = OpenProcess(PROCESS_TERMINATE, (FALSE) as BOOL, dwProcessID);
    if hProcess.is_null() {
        DPRINT!("CloseProcess(): Failed to open the process for termination! (Error: %lu)\n", GetLastError());
        return (FALSE) as BOOL;
    }

    /* Terminate it */
    TerminateProcess(hProcess, 0);
    CloseHandle(hProcess);
    return (TRUE) as BOOL;
}
