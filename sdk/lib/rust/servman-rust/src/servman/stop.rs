use crate::servman::progress::IncrementProgressBar;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_WAIT_TIME: u32 = 30000;
#[no_mangle]
pub unsafe extern "C" fn DoStopService(mut ServiceName: LPWSTR, mut hProgress: HANDLE) -> DWORD {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut ServiceStatus: SERVICE_STATUS_PROCESS = core::mem::zeroed();
    let mut BytesNeeded: DWORD = 0;
    let mut StartTime: DWORD = 0;
    let mut WaitTime: DWORD = 0;
    let mut Timeout: DWORD = 0;
    let mut dwResult: DWORD = ERROR_SUCCESS;

    hSCManager = OpenSCManagerW(core::ptr::null_mut(),
                                core::ptr::null_mut(),
                                SC_MANAGER_CONNECT);
    if hSCManager.is_null() { return GetLastError(); }

    hService = OpenServiceW(hSCManager,
                            ServiceName as LPCWSTR,
                            SERVICE_STOP | SERVICE_QUERY_STATUS);
    if hService.is_null()
    {
        dwResult = GetLastError();
        CloseServiceHandle(hSCManager);
        return dwResult;
    }

    if !hProgress.is_null()
    {
        /* Increment the progress bar */
        IncrementProgressBar(hProgress, DEFAULT_STEP);
    }

    /* Set the start and max wait times */
    StartTime = GetTickCount();
    Timeout = MAX_WAIT_TIME;

    /* Send the service the stop code */
    if ControlService(hService,
                        SERVICE_CONTROL_STOP,
                        core::ptr::addr_of_mut!(ServiceStatus) as LPSERVICE_STATUS) != 0
    {
        if !hProgress.is_null()
        {
            /* Increment the progress bar */
            IncrementProgressBar(hProgress, DEFAULT_STEP);
        }

        while ServiceStatus.dwCurrentState != SERVICE_STOPPED
        {
            let mut i: core::ffi::c_int = 0;
            /* Fixup the wait time */
            WaitTime = ServiceStatus.dwWaitHint / 10;

            if WaitTime < 1000 { WaitTime = 1000; }
            else if WaitTime > 10000 { WaitTime = 10000; }

            /* We don't wanna wait for up to 10 secs without incrementing */
            i = (WaitTime / 1000) as core::ffi::c_int;
            while i > 0
            {
                Sleep(1000);
                if !hProgress.is_null()
                {
                    /* Increment the progress bar */
                    IncrementProgressBar(hProgress, DEFAULT_STEP);
                }
                i -= 1;
            }

            if QueryServiceStatusEx(hService,
                                        SC_STATUS_PROCESS_INFO,
                                        core::ptr::addr_of_mut!(ServiceStatus) as LPBYTE,
                                        core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
                                        core::ptr::addr_of_mut!(BytesNeeded)) != 0
            {
                /* Have we exceeded our wait time? */
                if GetTickCount().wrapping_sub(StartTime) > Timeout
                {
                    /* Yep, give up */
                    DPRINT1!("Timeout\n");
                    dwResult = ERROR_SERVICE_REQUEST_TIMEOUT;
                    break;
                }
            }
            else
            {
                dwResult = GetLastError();
                DPRINT1!("QueryServiceStatusEx failed: %d\n", dwResult);
            }
        }

        /* If the service is stopped, return TRUE */
        if ServiceStatus.dwCurrentState == SERVICE_STOPPED
        {
            dwResult = ERROR_SUCCESS;
        }
    }
    else
    {
        dwResult = GetLastError();
    }

    CloseServiceHandle(hService);
    CloseServiceHandle(hSCManager);

    return dwResult;
}
