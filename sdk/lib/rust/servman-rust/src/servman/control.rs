use crate::servman::progress::IncrementProgressBar;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_WAIT_TIME: u32 = 30000;
#[no_mangle]
pub unsafe extern "C" fn DoControlService(ServiceName: LPWSTR, hProgress: HWND, Control: DWORD) -> DWORD {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut ServiceStatus: SERVICE_STATUS_PROCESS = core::mem::zeroed();
    let mut Status: SERVICE_STATUS = core::mem::zeroed();
    let mut BytesNeeded: DWORD = 0;
    let mut StartTickCount: DWORD;
    let mut OldCheckPoint: DWORD;
    let mut WaitTime: DWORD;
    let mut MaxWait: DWORD;
    let mut ReqState: DWORD = 0;
    let mut i: DWORD;
    let mut Result: BOOL;
    let mut dwResult: DWORD = ERROR_SUCCESS;

    /* Set the state we're interested in */
    if Control == SERVICE_CONTROL_PAUSE {
        ReqState = SERVICE_PAUSED;
    } else if Control == SERVICE_CONTROL_CONTINUE {
        ReqState = SERVICE_RUNNING;
    } else {
        /* Unhandled control code */
        DPRINT1!("Unknown control command: 0x%X\n", Control);
        return ERROR_INVALID_SERVICE_CONTROL;
    }

    hSCManager = OpenSCManagerW(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        SC_MANAGER_CONNECT,
    );
    if hSCManager.is_null() {
        return GetLastError();
    }

    hService = OpenServiceW(
        hSCManager,
        ServiceName,
        SERVICE_PAUSE_CONTINUE | SERVICE_INTERROGATE | SERVICE_QUERY_STATUS,
    );
    if hService.is_null() {
        dwResult = GetLastError();
        CloseServiceHandle(hSCManager);
        return dwResult;
    }

    /* Send the control message to the service */
    Result = ControlService(
        hService,
        Control,
        core::ptr::addr_of_mut!(Status),
    );
    if Result != 0 {
        if !hProgress.is_null() {
            /* Increment the progress bar */
            IncrementProgressBar(hProgress as HANDLE, DEFAULT_STEP as UINT);
        }

        /* Get the service status */
        Result = QueryServiceStatusEx(
            hService,
            SC_STATUS_PROCESS_INFO,
            core::ptr::addr_of_mut!(ServiceStatus) as LPBYTE,
            core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
            core::ptr::addr_of_mut!(BytesNeeded),
        );
        if Result != 0 {
            Result = (FALSE) as i32;
            MaxWait = MAX_WAIT_TIME;
            OldCheckPoint = ServiceStatus.dwCheckPoint;
            StartTickCount = GetTickCount();

            /* Loop until it's at the correct state */
            while ServiceStatus.dwCurrentState != ReqState {
                /* Fixup the wait time */
                WaitTime = ServiceStatus.dwWaitHint / 10;

                if WaitTime < 1000 {
                    WaitTime = 1000;
                } else if WaitTime > 10000 {
                    WaitTime = 10000;
                }

                /* We don't wanna wait for up to 10 secs without incrementing */
                i = WaitTime / 1000;
                while i > 0 {
                    Sleep(1000);
                    if !hProgress.is_null() {
                        /* Increment the progress bar */
                        IncrementProgressBar(hProgress as HANDLE, DEFAULT_STEP as UINT);
                    }
                    i = i.wrapping_sub(1);
                }

                /* Get the latest status info */
                if QueryServiceStatusEx(
                    hService,
                    SC_STATUS_PROCESS_INFO,
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
                    IncrementProgressBar(hProgress as HANDLE, DEFAULT_STEP as UINT);
                } else {
                    /* It's not, make sure we haven't exceeded our wait time */
                    if GetTickCount() >= StartTickCount.wrapping_add(MaxWait) {
                        /* We have, give up */
                        DPRINT1!("Timeout\n");
                        dwResult = ERROR_SERVICE_REQUEST_TIMEOUT;
                        break;
                    }
                }
            }
        } else {
            dwResult = GetLastError();
        }

        if ServiceStatus.dwCurrentState == ReqState {
            dwResult = ERROR_SUCCESS;
        }
    } else {
        dwResult = GetLastError();
    }

    CloseServiceHandle(hService);
    CloseServiceHandle(hSCManager);

    return dwResult;
}
