use crate::sc::misc::ParseFailureArguments;
use crate::sc::sc::ReportLastError;
use crate::sc::usage::SetFailureUsage;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn QueryFailure(ServiceName: LPCTSTR) -> BOOL {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut cbBytesNeeded: DWORD = 0;
    let mut pServiceFailure: LPSERVICE_FAILURE_ACTIONS = core::ptr::null_mut();
    let mut i: INT = 0;

    if false {
        _tprintf((&[0x0073, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0074, 0x006f, 0x0020, 0x0073, 0x0068, 0x006f, 0x0077, 0x0020, 0x0066, 0x0061, 0x0069, 0x006c, 0x0075, 0x0072, 0x0065, 0x0020, 0x0061, 0x0063, 0x0074, 0x0069, 0x006f, 0x006e, 0x0020, 0x002d, 0x0020, 0x0025, 0x0073, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "service to show failure action - %s\n\n" */, ServiceName);
    }

    'done: {
        hManager = OpenSCManager(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            SC_MANAGER_CONNECT,
        );
        if hManager.is_null() {
            bResult = (FALSE) as i32;
            break 'done;
        }

        hService = OpenService(hManager, ServiceName, SERVICE_QUERY_CONFIG);
        if hService.is_null() {
            bResult = (FALSE) as i32;
            break 'done;
        }

        if QueryServiceConfig2(
            hService,
            SERVICE_CONFIG_FAILURE_ACTIONS,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(cbBytesNeeded),
        ) == 0
        {
            if cbBytesNeeded == 0 {
                bResult = (FALSE) as i32;
                break 'done;
            }
        }

        pServiceFailure = HeapAlloc(
            GetProcessHeap(),
            0,
            cbBytesNeeded as SIZE_T,
        ) as LPSERVICE_FAILURE_ACTIONS;
        if pServiceFailure.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            bResult = (FALSE) as i32;
            break 'done;
        }

        if QueryServiceConfig2(
            hService,
            SERVICE_CONFIG_FAILURE_ACTIONS,
            pServiceFailure as LPBYTE,
            cbBytesNeeded,
            core::ptr::addr_of_mut!(cbBytesNeeded),
        ) == 0
        {
            bResult = (FALSE) as i32;
            break 'done;
        }

        _tprintf((&[0x0053, 0x0045, 0x0052, 0x0056, 0x0049, 0x0043, 0x0045, 0x005f, 0x004e, 0x0041, 0x004d, 0x0045, 0x003a, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "SERVICE_NAME: %s\n" */, ServiceName);
        _tprintf(
            (&[0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0052, 0x0045, 0x0053, 0x0045, 0x0054, 0x005f, 0x0050, 0x0045, 0x0052, 0x0049, 0x004f, 0x0044, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x003a, 0x0020, 0x0025, 0x006c, 0x0075, 0x0020, 0x0073, 0x0065, 0x0063, 0x006f, 0x006e, 0x0064, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "        RESET_PERIOD       : %lu seconds\n" */,
            (*pServiceFailure).dwResetPeriod,
        );
        _tprintf(
            (&[0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0052, 0x0045, 0x0042, 0x004f, 0x004f, 0x0054, 0x005f, 0x004d, 0x0045, 0x0053, 0x0053, 0x0041, 0x0047, 0x0045, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x003a, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "        REBOOT_MESSAGE     : %s\n" */,
            if !(*pServiceFailure).lpRebootMsg.is_null() {
                (*pServiceFailure).lpRebootMsg
            } else {
                (&[0x0000u16]).as_ptr() /* _T "" */
            },
        );
        _tprintf(
            (&[0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0043, 0x004f, 0x004d, 0x004d, 0x0041, 0x004e, 0x0044, 0x005f, 0x004c, 0x0049, 0x004e, 0x0045, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x003a, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "        COMMAND_LINE       : %s\n" */,
            if !(*pServiceFailure).lpCommand.is_null() {
                (*pServiceFailure).lpCommand
            } else {
                (&[0x0000u16]).as_ptr() /* _T "" */
            },
        );
        _tprintf((&[0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0055, 0x0052, 0x0045, 0x005f, 0x0041, 0x0043, 0x0054, 0x0049, 0x004f, 0x004e, 0x0053, 0x0020, 0x0020, 0x0020, 0x0020, 0x003a, 0x0020, 0x0000u16]).as_ptr() /* _T "        FAILURE_ACTIONS    : " */);
        i = 0;
        while (i as DWORD) < (*pServiceFailure).cActions {
            'body: {
                if i != 0 {
                    _tprintf((&[0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0000u16]).as_ptr() /* _T "                             " */);
                }
                match (*(*pServiceFailure).lpsaActions.offset(i as isize)).Type {
                    SC_ACTION_NONE => {
                        break 'body;
                    }
                    SC_ACTION_RESTART => {
                        _tprintf(
                            (&[0x0052, 0x0045, 0x0053, 0x0054, 0x0041, 0x0052, 0x0054, 0x0020, 0x002d, 0x002d, 0x0020, 0x0044, 0x0065, 0x006c, 0x0061, 0x0079, 0x0020, 0x003d, 0x0020, 0x0025, 0x006c, 0x0075, 0x0020, 0x006d, 0x0069, 0x006c, 0x006c, 0x0069, 0x0073, 0x0065, 0x0063, 0x006f, 0x006e, 0x0064, 0x0073, 0x002e, 0x000a, 0x0000u16]).as_ptr() /* _T "RESTART -- Delay = %lu milliseconds.\n" */,
                            (*(*pServiceFailure).lpsaActions.offset(i as isize)).Delay,
                        );
                    }
                    SC_ACTION_REBOOT => {
                        _tprintf(
                            (&[0x0052, 0x0045, 0x0042, 0x004f, 0x004f, 0x0054, 0x0020, 0x002d, 0x002d, 0x0020, 0x0044, 0x0065, 0x006c, 0x0061, 0x0079, 0x0020, 0x003d, 0x0020, 0x0025, 0x006c, 0x0075, 0x0020, 0x006d, 0x0069, 0x006c, 0x006c, 0x0069, 0x0073, 0x0065, 0x0063, 0x006f, 0x006e, 0x0064, 0x0073, 0x002e, 0x000a, 0x0000u16]).as_ptr() /* _T "REBOOT -- Delay = %lu milliseconds.\n" */,
                            (*(*pServiceFailure).lpsaActions.offset(i as isize)).Delay,
                        );
                    }
                    SC_ACTION_RUN_COMMAND => {
                        _tprintf(
                            (&[0x0052, 0x0055, 0x004e, 0x005f, 0x0043, 0x004f, 0x004d, 0x004d, 0x0041, 0x004e, 0x0044, 0x0020, 0x002d, 0x002d, 0x0020, 0x0044, 0x0065, 0x006c, 0x0061, 0x0079, 0x0020, 0x003d, 0x0020, 0x0025, 0x006c, 0x0075, 0x0020, 0x006d, 0x0069, 0x006c, 0x006c, 0x0069, 0x0073, 0x0065, 0x0063, 0x006f, 0x006e, 0x0064, 0x0073, 0x002e, 0x000a, 0x0000u16]).as_ptr() /* _T "RUN_COMMAND -- Delay = %lu milliseconds.\n" */,
                            (*(*pServiceFailure).lpsaActions.offset(i as isize)).Delay,
                        );
                    }
                    _ => {
                        _tprintf((&[0x000a, 0x0000u16]).as_ptr() /* _T "\n" */);
                    }
                }
            }
            i += 1;
        }
    }

    if (bResult) as u32 == FALSE {
        ReportLastError();
    }

    if !pServiceFailure.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            pServiceFailure as PVOID,
        );
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }

    return bResult;
}
#[no_mangle]
pub unsafe extern "C" fn SetFailure(ServiceArgs: *mut LPCTSTR, ArgCount: INT) -> BOOL {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE) as i32;
    let mut FailureActions: SERVICE_FAILURE_ACTIONS = core::mem::zeroed();
    let mut lpServiceName: LPCTSTR = core::ptr::null();
    let mut Old: BOOLEAN = (FALSE) as u8;

    ZeroMemory!(core::ptr::addr_of_mut!(FailureActions) as *mut core::ffi::c_void, core::mem::size_of::<SERVICE_FAILURE_ACTIONS>());

    if ParseFailureArguments(ServiceArgs, ArgCount, core::ptr::addr_of_mut!(lpServiceName), core::ptr::addr_of_mut!(FailureActions)) == 0
    {
        SetFailureUsage();
        return (FALSE) as BOOL;
    }

    'done: {
        hManager = OpenSCManager(core::ptr::null_mut(),
                                 core::ptr::null_mut(),
                                 SC_MANAGER_CONNECT);
        if hManager.is_null()
        {
            let fmt: [u16; 33] = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'C' as u16, 'M' as u16,
                'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16, ' ' as u16,
                'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16,
                '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16];
            _tprintf(fmt.as_ptr(), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        hService = OpenService(hManager,
                               lpServiceName,
                               SERVICE_CHANGE_CONFIG | SERVICE_START);
        if hService.is_null()
        {
            let fmt: [u16; 31] = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                'O' as u16, 'p' as u16, 'e' as u16, 'n' as u16, 'S' as u16, 'e' as u16, 'r' as u16,
                'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, ' ' as u16,
                'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16,
                '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16];
            _tprintf(fmt.as_ptr(), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, (TRUE) as BOOLEAN, (FALSE) as BOOLEAN, core::ptr::addr_of_mut!(Old));

        if ChangeServiceConfig2(hService,
                                  SERVICE_CONFIG_FAILURE_ACTIONS,
                                  (core::ptr::addr_of_mut!(FailureActions) as LPBYTE).cast()) == 0
        {
            let fmt: [u16; 40] = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
                'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 'S' as u16,
                'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
                'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16,
                '2' as u16, ' ' as u16,
                'F' as u16, 'A' as u16, 'I' as u16, 'L' as u16, 'E' as u16, 'D' as u16, ' ' as u16,
                '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, '\n' as u16, '\n' as u16, 0u16];
            _tprintf(fmt.as_ptr(), GetLastError());
            bResult = (FALSE) as i32;
            break 'done;
        }

        let fmt: [u16; 36] = ['[' as u16, 'S' as u16, 'C' as u16, ']' as u16, ' ' as u16,
            'C' as u16, 'h' as u16, 'a' as u16, 'n' as u16, 'g' as u16, 'e' as u16, 'S' as u16,
            'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16,
            'C' as u16, 'o' as u16, 'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16,
            '2' as u16, ' ' as u16,
            'S' as u16, 'U' as u16, 'C' as u16, 'C' as u16, 'E' as u16, 'S' as u16, 'S' as u16,
            '\n' as u16, '\n' as u16, 0u16];
        _tprintf(fmt.as_ptr());
    }

    // done:
    RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, Old, (FALSE) as BOOLEAN, core::ptr::addr_of_mut!(Old));

    if (bResult) as u32 == FALSE
    {
        ReportLastError();
    }

    if !FailureActions.lpsaActions.is_null()
    {
        HeapFree(GetProcessHeap(), 0, FailureActions.lpsaActions as PVOID);
    }

    if !hService.is_null()
    {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null()
    {
        CloseServiceHandle(hManager);
    }

    return bResult;
}
