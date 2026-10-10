use crate::sc::usage::CreateUsage;
use crate::sc::misc::ParseCreateConfigArguments;
use crate::sc::sc::ReportLastError;
use crate::sc::sc::hSCManager;
use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn Create(ServiceArgs: *mut LPCTSTR, ArgCount: INT) -> BOOL {
    let mut h_sc_manager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut bRet: BOOL = (TRUE) as i32;

    let mut i: INT = 0;
    let mut Length: INT = 0;
    let mut lpBuffer: LPTSTR = core::ptr::null_mut();
    let mut ServiceInfo: SERVICE_CREATE_INFO = core::mem::zeroed();

    if ParseCreateConfigArguments(
        ServiceArgs,
        ArgCount,
        (FALSE) as BOOL,
        core::ptr::addr_of_mut!(ServiceInfo),
    ) == 0
    {
        CreateUsage();
        return (FALSE) as BOOL;
    }

    if ServiceInfo.dwServiceType == 0 {
        ServiceInfo.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    }

    if ServiceInfo.dwStartType == 0 {
        ServiceInfo.dwStartType = SERVICE_DEMAND_START;
    }

    if ServiceInfo.dwErrorControl == 0 {
        ServiceInfo.dwErrorControl = SERVICE_ERROR_NORMAL;
    }

    if !ServiceInfo.lpDependencies.is_null() {
        Length = lstrlen(ServiceInfo.lpDependencies as LPCTSTR) as INT;

        lpBuffer = HeapAlloc(
            GetProcessHeap(),
            0,
            ((Length + 2) * (core::mem::size_of::<TCHAR>()) as i32) as SIZE_T,
        ) as LPTSTR;

        i = 0;
        while i < Length {
            if *ServiceInfo.lpDependencies.offset(i as isize) == ('/' as u16) {
                *lpBuffer.offset(i as isize) = 0;
            } else {
                *lpBuffer.offset(i as isize) = *ServiceInfo.lpDependencies.offset(i as isize);
            }
            i += 1;
        }

        *lpBuffer.offset(Length as isize) = 0;
        *lpBuffer.offset((Length + 1) as isize) = 0;

        ServiceInfo.lpDependencies = lpBuffer as LPCTSTR;
    }

    // SCDBG debug block (compiled out in the C source; kept as dead code)
    if false {
        _tprintf((&[0x0073, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x006e, 0x0061, 0x006d, 0x0065, 0x0020, 0x002d, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "service name - %s\n" */, ServiceInfo.lpServiceName);
        _tprintf((&[0x0064, 0x0069, 0x0073, 0x0070, 0x006c, 0x0061, 0x0079, 0x0020, 0x006e, 0x0061, 0x006d, 0x0065, 0x0020, 0x002d, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "display name - %s\n" */, ServiceInfo.lpDisplayName);
        _tprintf((&[0x0073, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x0020, 0x002d, 0x0020, 0x0025, 0x006c, 0x0075, 0x000a, 0x0000u16]).as_ptr() /* _T "service type - %lu\n" */, ServiceInfo.dwServiceType);
        _tprintf((&[0x0073, 0x0074, 0x0061, 0x0072, 0x0074, 0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x0020, 0x002d, 0x0020, 0x0025, 0x006c, 0x0075, 0x000a, 0x0000u16]).as_ptr() /* _T "start type - %lu\n" */, ServiceInfo.dwStartType);
        _tprintf((&[0x0065, 0x0072, 0x0072, 0x006f, 0x0072, 0x0020, 0x0063, 0x006f, 0x006e, 0x0074, 0x0072, 0x006f, 0x006c, 0x0020, 0x002d, 0x0020, 0x0025, 0x006c, 0x0075, 0x000a, 0x0000u16]).as_ptr() /* _T "error control - %lu\n" */, ServiceInfo.dwErrorControl);
        _tprintf((&[0x0042, 0x0069, 0x006e, 0x0061, 0x0072, 0x0079, 0x0020, 0x0070, 0x0061, 0x0074, 0x0068, 0x0020, 0x002d, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "Binary path - %s\n" */, ServiceInfo.lpBinaryPathName);
        _tprintf((&[0x006c, 0x006f, 0x0061, 0x0064, 0x0020, 0x006f, 0x0072, 0x0064, 0x0065, 0x0072, 0x0020, 0x0067, 0x0072, 0x006f, 0x0075, 0x0070, 0x0020, 0x002d, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "load order group - %s\n" */, ServiceInfo.lpLoadOrderGroup);
        _tprintf((&[0x0074, 0x0061, 0x0067, 0x0020, 0x002d, 0x0020, 0x0025, 0x006c, 0x0075, 0x000a, 0x0000u16]).as_ptr() /* _T "tag - %lu\n" */, ServiceInfo.dwTagId);
        _tprintf((&[0x0064, 0x0065, 0x0070, 0x0065, 0x006e, 0x0064, 0x0065, 0x006e, 0x0063, 0x0069, 0x0065, 0x0073, 0x0020, 0x002d, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "dependencies - %s\n" */, ServiceInfo.lpDependencies);
        _tprintf((&[0x0061, 0x0063, 0x0063, 0x006f, 0x0075, 0x006e, 0x0074, 0x0020, 0x0073, 0x0074, 0x0061, 0x0072, 0x0074, 0x0020, 0x006e, 0x0061, 0x006d, 0x0065, 0x0020, 0x002d, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "account start name - %s\n" */, ServiceInfo.lpServiceStartName);
        _tprintf((&[0x0061, 0x0063, 0x0063, 0x006f, 0x0075, 0x006e, 0x0074, 0x0020, 0x0070, 0x0061, 0x0073, 0x0073, 0x0077, 0x006f, 0x0072, 0x0064, 0x0020, 0x002d, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "account password - %s\n" */, ServiceInfo.lpPassword);
    }

    h_sc_manager = OpenSCManager(
        core::ptr::null(),
        core::ptr::null(),
        SC_MANAGER_CREATE_SERVICE,
    );

    'done: {
        if h_sc_manager.is_null() {
            _tprintf(
                (&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x004f, 0x0070, 0x0065, 0x006e, 0x0053, 0x0043, 0x004d, 0x0061, 0x006e, 0x0061, 0x0067, 0x0065, 0x0072, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] OpenSCManager FAILED %lu:\n\n" */,
                GetLastError(),
            );
            bRet = (FALSE) as i32;
            break 'done;
        }

        hService = CreateService(
            h_sc_manager,
            ServiceInfo.lpServiceName as LPCTSTR,
            ServiceInfo.lpDisplayName as LPCTSTR,
            SERVICE_ALL_ACCESS,
            ServiceInfo.dwServiceType,
            ServiceInfo.dwStartType,
            ServiceInfo.dwErrorControl,
            ServiceInfo.lpBinaryPathName as LPCTSTR,
            ServiceInfo.lpLoadOrderGroup as LPCTSTR,
            if ServiceInfo.bTagId != 0 {
                core::ptr::addr_of_mut!(ServiceInfo.dwTagId)
            } else {
                core::ptr::null_mut()
            },
            ServiceInfo.lpDependencies as LPCTSTR,
            ServiceInfo.lpServiceStartName as LPCTSTR,
            ServiceInfo.lpPassword as LPCTSTR,
        );
        if hService.is_null() {
            _tprintf(
                (&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x0043, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0046, 0x0041, 0x0049, 0x004c, 0x0045, 0x0044, 0x0020, 0x0025, 0x006c, 0x0075, 0x003a, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] CreateService FAILED %lu:\n\n" */,
                GetLastError(),
            );
            bRet = (FALSE) as i32;
            break 'done;
        }

        _tprintf((&[0x005b, 0x0053, 0x0043, 0x005d, 0x0020, 0x0043, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0053, 0x0055, 0x0043, 0x0043, 0x0045, 0x0053, 0x0053, 0x000a, 0x000a, 0x0000u16]).as_ptr() /* _T "[SC] CreateService SUCCESS\n\n" */);
    }

    if (bRet) as u32 == FALSE {
        ReportLastError();
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !h_sc_manager.is_null() {
        CloseServiceHandle(h_sc_manager);
    }

    if !lpBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, lpBuffer as PVOID);
    }

    return bRet;
}
