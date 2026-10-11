use crate::servman::progress::IncrementProgressBar;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_WAIT_TIME: i32 = 30000;
#[no_mangle]
pub unsafe extern "C" fn DoStartService(mut ServiceName: LPWSTR, mut hProgress: HANDLE, mut lpStartParams: LPWSTR) -> DWORD {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut ServiceStatus: SERVICE_STATUS_PROCESS = core::mem::zeroed::<SERVICE_STATUS_PROCESS>();
    let mut BytesNeeded: DWORD = 0 as DWORD;
    let mut StartTickCount: DWORD;
    let mut OldCheckPoint: DWORD;
    let mut WaitTime: DWORD;
    let mut MaxWait: DWORD;
    let mut Result: BOOL = FALSE as BOOL;

    let mut bWhiteSpace: BOOL = TRUE as BOOL;
    let mut lpChar: LPWSTR;
    let mut dwArgsCount: DWORD = 0 as DWORD;
    let mut dwResult: DWORD = ERROR_SUCCESS as DWORD;
    let mut lpArgsVector: *mut LPCWSTR = core::ptr::null_mut();

    if !lpStartParams.is_null() {
        /* Count the number of arguments */
        lpChar = lpStartParams;
        while *lpChar != 0 as WCHAR {
            if iswspace(*lpChar) != 0 {
                bWhiteSpace = TRUE as BOOL;
            } else {
                if bWhiteSpace != FALSE as BOOL {
                    dwArgsCount = dwArgsCount.wrapping_add(1);
                    bWhiteSpace = FALSE as BOOL;
                }
            }

            lpChar = lpChar.offset(1);
        }

        /*
         * Allocate the arguments vector.
         * Do not add the service name here because services.exe does it for us!
         */
        lpArgsVector = LocalAlloc(
            LMEM_FIXED as UINT,
            (dwArgsCount as SIZE_T).wrapping_mul(core::mem::size_of::<LPCWSTR>() as SIZE_T),
        ) as *mut LPCWSTR;
        if lpArgsVector.is_null() {
            return GetLastError();
        }

        /* Fill the arguments vector */
        dwArgsCount = 0 as DWORD;
        bWhiteSpace = TRUE as BOOL;
        lpChar = lpStartParams;
        while *lpChar != 0 as WCHAR {
            if iswspace(*lpChar) != 0 {
                *lpChar = 0 as WCHAR;
                bWhiteSpace = TRUE as BOOL;
            } else {
                if bWhiteSpace != FALSE as BOOL {
                    *lpArgsVector.offset(dwArgsCount as isize) = lpChar as LPCWSTR;
                    dwArgsCount = dwArgsCount.wrapping_add(1);
                    bWhiteSpace = FALSE as BOOL;
                }
            }

            lpChar = lpChar.offset(1);
        }
    }

    hSCManager = OpenSCManagerW(
        core::ptr::null(),
        core::ptr::null(),
        SC_MANAGER_CONNECT as DWORD,
    );
    if hSCManager.is_null() {
        dwResult = GetLastError();
        if !lpArgsVector.is_null() {
            LocalFree(lpArgsVector as LPVOID);
        }
        return dwResult;
    }

    hService = OpenServiceW(
        hSCManager,
        ServiceName as LPCWSTR,
        (SERVICE_START | SERVICE_QUERY_STATUS) as DWORD,
    );
    if hService.is_null() {
        dwResult = GetLastError();
        CloseServiceHandle(hSCManager);
        if !lpArgsVector.is_null() {
            LocalFree(lpArgsVector as LPVOID);
        }
        return dwResult;
    }

    /* Start the service */
    Result = StartServiceW(hService, dwArgsCount, lpArgsVector);
    if Result == 0 && GetLastError() == ERROR_SERVICE_ALREADY_RUNNING as DWORD {
        /* If it's already running, just return TRUE */
        Result = TRUE as BOOL;
    } else if Result != 0 {
        if !hProgress.is_null() {
            /* Increment the progress bar */
            IncrementProgressBar(hProgress, DEFAULT_STEP as UINT);
        }

        /* Get the service status to check if it's running */
        Result = QueryServiceStatusEx(
            hService,
            SC_STATUS_PROCESS_INFO as DWORD,
            core::ptr::addr_of_mut!(ServiceStatus) as LPBYTE,
            core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
            core::ptr::addr_of_mut!(BytesNeeded),
        );
        if Result != 0 {
            Result = FALSE as BOOL;
            MaxWait = MAX_WAIT_TIME as DWORD;
            OldCheckPoint = ServiceStatus.dwCheckPoint;
            StartTickCount = GetTickCount();

            /* Loop until it's running */
            while ServiceStatus.dwCurrentState != SERVICE_RUNNING as DWORD {
                let mut i: ::core::ffi::c_int;
                /* Fixup the wait time */
                WaitTime = ServiceStatus.dwWaitHint / (10 as DWORD);

                if WaitTime < 1000 as DWORD {
                    WaitTime = 1000 as DWORD;
                } else if WaitTime > 10000 as DWORD {
                    WaitTime = 10000 as DWORD;
                }

                /* We don't wanna wait for up to 10 secs without incrementing */
                i = (WaitTime / (1000 as DWORD)) as ::core::ffi::c_int;
                while i > 0 as ::core::ffi::c_int {
                    Sleep(1000 as DWORD);
                    if !hProgress.is_null() {
                        /* Increment the progress bar */
                        IncrementProgressBar(hProgress, DEFAULT_STEP as UINT);
                    }
                    i -= 1;
                }

                /* Get the latest status info */
                if QueryServiceStatusEx(
                    hService,
                    SC_STATUS_PROCESS_INFO as DWORD,
                    core::ptr::addr_of_mut!(ServiceStatus) as LPBYTE,
                    core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
                    core::ptr::addr_of_mut!(BytesNeeded),
                ) == 0
                {
                    /* Something went wrong... */
                    dwResult = GetLastError();
                    DPRINT1!("QueryServiceStatusEx failed: %d\n", dwResult);
                    break;
                }

                /* Is the service making progress? */
                if ServiceStatus.dwCheckPoint > OldCheckPoint {
                    /* It is, get the latest tickcount to reset the max wait time */
                    StartTickCount = GetTickCount();
                    OldCheckPoint = ServiceStatus.dwCheckPoint;
                } else {
                    /* It's not, make sure we haven't exceeded our wait time */
                    if GetTickCount() >= StartTickCount.wrapping_add(MaxWait) {
                        /* We have, give up */
                        DPRINT1!("Timeout\n");
                        dwResult = ERROR_SERVICE_REQUEST_TIMEOUT as DWORD;
                        break;
                    }
                }
            }
        } else {
            dwResult = GetLastError();
        }

        if ServiceStatus.dwCurrentState == SERVICE_RUNNING as DWORD {
            dwResult = ERROR_SUCCESS as DWORD;
        }
    }

    CloseServiceHandle(hService);
    CloseServiceHandle(hSCManager);

    if !lpArgsVector.is_null() {
        LocalFree(lpArgsVector as LPVOID);
    }

    return dwResult;
}
