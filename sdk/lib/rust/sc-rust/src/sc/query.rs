use crate::sc::print::PrintService;
use crate::sc::sc::ReportLastError;
use crate::sc::sc::hSCManager;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn QueryService(ServiceName: LPCTSTR) -> LPSERVICE_STATUS_PROCESS {
    let mut h_sc_manager: SC_HANDLE = core::ptr::null_mut();
    let mut pServiceInfo: LPSERVICE_STATUS_PROCESS = core::ptr::null_mut();
    let mut hSc: SC_HANDLE = core::ptr::null_mut();
    let mut BufSiz: DWORD = 0;
    let mut BytesNeeded: DWORD = 0;
    let mut Ret: DWORD = 0;

    h_sc_manager = OpenSCManager(core::ptr::null_mut(),
                               core::ptr::null(),
                               SC_MANAGER_CONNECT);
    if h_sc_manager.is_null() {
        ReportLastError();
        return core::ptr::null_mut();
    }

    'fail: {
        hSc = OpenService(h_sc_manager,
                          ServiceName,
                          SERVICE_QUERY_STATUS);
        if hSc.is_null() {
            break 'fail;
        }

        Ret = (QueryServiceStatusEx(hSc,
                                   SC_STATUS_PROCESS_INFO,
                                   core::ptr::null_mut(),
                                   BufSiz,
                                   core::ptr::addr_of_mut!(BytesNeeded))) as u32;
        if (Ret != 0) || (GetLastError() != ERROR_INSUFFICIENT_BUFFER) {
            break 'fail;
        }

        pServiceInfo = HeapAlloc(GetProcessHeap(),
                                              0,
                                              BytesNeeded as SIZE_T) as LPSERVICE_STATUS_PROCESS;
        if pServiceInfo.is_null() {
            break 'fail;
        }

        if QueryServiceStatusEx(hSc,
                                  SC_STATUS_PROCESS_INFO,
                                  pServiceInfo as LPBYTE,
                                  BytesNeeded,
                                  core::ptr::addr_of_mut!(BytesNeeded)) == 0
        {
            break 'fail;
        }

        CloseServiceHandle(hSc);
        CloseServiceHandle(h_sc_manager);
        return pServiceInfo;
    }

    // fail:
    ReportLastError();
    if !pServiceInfo.is_null() {
        HeapFree(GetProcessHeap(), 0, pServiceInfo as PVOID);
    }
    if !hSc.is_null() {
        CloseServiceHandle(hSc);
    }
    if !h_sc_manager.is_null() {
        CloseServiceHandle(h_sc_manager);
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn EnumServices(
    pServiceStatus: *mut *mut ENUM_SERVICE_STATUS_PROCESS,
    dwServiceType: DWORD,
    dwServiceState: DWORD,
    mut dwBufferSize: DWORD,
    dwResumeIndex: DWORD,
    pszGroupName: LPCTSTR,
) -> DWORD {
    let mut hSCManagerLocal: SC_HANDLE = core::ptr::null_mut();
    let mut BytesNeeded: DWORD = 0;
    let mut ResumeHandle: DWORD = dwResumeIndex;
    let mut NumServices: DWORD = 0;
    let mut Ret: BOOL;

    hSCManagerLocal = OpenSCManager(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        SC_MANAGER_ENUMERATE_SERVICE,
    );
    if hSCManagerLocal.is_null() {
        ReportLastError();
        return 0;
    }

    if dwBufferSize == 0 {
        Ret = EnumServicesStatusEx(
            hSCManagerLocal,
            SC_ENUM_PROCESS_INFO,
            dwServiceType,
            dwServiceState,
            *pServiceStatus as LPBYTE,
            dwBufferSize,
            &raw mut BytesNeeded,
            &raw mut NumServices,
            &raw mut ResumeHandle,
            pszGroupName as LPCWSTR,
        );
        if (Ret == 0) && (GetLastError() != ERROR_MORE_DATA) {
            ReportLastError();
            CloseServiceHandle(hSCManagerLocal); // the C leaks the handle on this path
            return 0;
        }

        dwBufferSize = BytesNeeded;
    }

    *pServiceStatus = HeapAlloc(
        GetProcessHeap(),
        0,
        dwBufferSize as SIZE_T,
    ) as *mut ENUM_SERVICE_STATUS_PROCESS;
    if !(*pServiceStatus).is_null() {
        if EnumServicesStatusEx(
            hSCManagerLocal,
            SC_ENUM_PROCESS_INFO,
            dwServiceType,
            dwServiceState,
            *pServiceStatus as LPBYTE,
            dwBufferSize,
            &raw mut BytesNeeded,
            &raw mut NumServices,
            &raw mut ResumeHandle,
            pszGroupName as LPCWSTR,
        ) != 0
        {
            CloseServiceHandle(hSCManagerLocal);
            return NumServices;
        }
    }

    // Divergence from the C, which freed the buffer here but still returned NumServices: with
    // a small `bufsize=` the call fails ERROR_MORE_DATA having filled NumServices entries, so
    // Query walked the freed array and then freed it a second time. Those entries are the
    // point of `bufsize=` ("Enumerates with a 50 byte buffer"): keep them, and Query prints
    // them and frees the buffer once. Any other failure returns nothing and no buffer.
    let more_data = GetLastError() == ERROR_MORE_DATA;
    ReportLastError();
    if more_data && NumServices != 0 && !(*pServiceStatus).is_null() {
        CloseServiceHandle(hSCManagerLocal);
        return NumServices;
    }
    if !(*pServiceStatus).is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            *pServiceStatus as PVOID,
        );
        *pServiceStatus = core::ptr::null_mut();
    }

    CloseServiceHandle(hSCManagerLocal);

    return 0;
}

unsafe extern "C" fn ParseQueryArguments(
    mut ServiceArgs: *mut LPCTSTR,
    mut ArgCount: INT,
    mut pdwServiceType: PDWORD,
    mut pdwServiceState: PDWORD,
    mut pdwBufferSize: PDWORD,
    mut pdwResumeIndex: PDWORD,
    mut ppszGroupName: *mut LPCTSTR,
    mut ppszServiceName: *mut LPCTSTR,
) -> BOOL {
    let mut TmpCount: INT = 0;
    let mut TmpIndex: INT = 0;
    let mut dwValue: DWORD = 0;

    TmpCount = ArgCount;
    TmpIndex = 0;
    while TmpCount > 0 {
        if lstrcmpi(
            *ServiceArgs.offset(TmpIndex as isize),
            ['t' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, 0u16].as_ptr(),
        ) == 0
        {
            TmpIndex += 1;
            TmpCount -= 1;

            if TmpCount > 0 {
                if lstrcmpi(
                    *ServiceArgs.offset(TmpIndex as isize),
                    ['s' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 0u16].as_ptr(),
                ) == 0
                {
                    *pdwServiceType = SERVICE_WIN32;
                } else if lstrcmpi(
                    *ServiceArgs.offset(TmpIndex as isize),
                    ['d' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, 0u16].as_ptr(),
                ) == 0
                {
                    *pdwServiceType = SERVICE_DRIVER;
                } else if lstrcmpi(
                    *ServiceArgs.offset(TmpIndex as isize),
                    ['a' as u16, 'l' as u16, 'l' as u16, 0u16].as_ptr(),
                ) == 0
                {
                    *pdwServiceType = SERVICE_TYPE_ALL;
                } else if lstrcmpi(
                    *ServiceArgs.offset(TmpIndex as isize),
                    ['i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 't' as u16, 0u16].as_ptr(),
                ) == 0
                {
                    *pdwServiceType |= SERVICE_INTERACTIVE_PROCESS;
                } else {
                    _tprintf(
                        [
                            'E' as u16, 'R' as u16, 'R' as u16, 'O' as u16, 'R' as u16, ' ' as u16,
                            'f' as u16, 'o' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 'w' as u16,
                            'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16, '"' as u16,
                            't' as u16, 'y' as u16, 'p' as u16, 'e' as u16, '=' as u16, '"' as u16,
                            '!' as u16, '\n' as u16,
                            'M' as u16, 'u' as u16, 's' as u16, 't' as u16, ' ' as u16,
                            'b' as u16, 'e' as u16, ' ' as u16, 'o' as u16, 'n' as u16, 'e' as u16,
                            ' ' as u16, 'o' as u16, 'f' as u16, ':' as u16, ' ' as u16,
                            'a' as u16, 'l' as u16, 'l' as u16, ',' as u16, ' ' as u16,
                            'd' as u16, 'r' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16, ',' as u16, ' ' as u16,
                            'i' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 't' as u16, ',' as u16, ' ' as u16,
                            's' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, '.' as u16, '\n' as u16,
                            0u16,
                        ]
                        .as_ptr(),
                    );
                    return (FALSE) as BOOL;
                }

                TmpIndex += 1;
                TmpCount -= 1;
            }
        } else if lstrcmpi(
            *ServiceArgs.offset(TmpIndex as isize),
            ['s' as u16, 't' as u16, 'a' as u16, 't' as u16, 'e' as u16, '=' as u16, 0u16].as_ptr(),
        ) == 0
        {
            TmpIndex += 1;
            TmpCount -= 1;

            if TmpCount > 0 {
                if lstrcmpi(
                    *ServiceArgs.offset(TmpIndex as isize),
                    ['a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 0u16].as_ptr(),
                ) == 0
                {
                    *pdwServiceState = SERVICE_ACTIVE;
                } else if lstrcmpi(
                    *ServiceArgs.offset(TmpIndex as isize),
                    ['i' as u16, 'n' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 0u16].as_ptr(),
                ) == 0
                {
                    *pdwServiceState = SERVICE_INACTIVE;
                } else if lstrcmpi(
                    *ServiceArgs.offset(TmpIndex as isize),
                    ['a' as u16, 'l' as u16, 'l' as u16, 0u16].as_ptr(),
                ) == 0
                {
                    *pdwServiceState = SERVICE_STATE_ALL;
                } else {
                    _tprintf(
                        [
                            'E' as u16, 'R' as u16, 'R' as u16, 'O' as u16, 'R' as u16, ' ' as u16,
                            'f' as u16, 'o' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 'w' as u16,
                            'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16, '"' as u16,
                            's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'e' as u16, '=' as u16, '"' as u16,
                            '!' as u16, '\n' as u16,
                            'M' as u16, 'u' as u16, 's' as u16, 't' as u16, ' ' as u16,
                            'b' as u16, 'e' as u16, ' ' as u16, 'o' as u16, 'n' as u16, 'e' as u16,
                            ' ' as u16, 'o' as u16, 'f' as u16, ':' as u16, ' ' as u16,
                            'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, ',' as u16, ' ' as u16,
                            'a' as u16, 'l' as u16, 'l' as u16, ',' as u16, ' ' as u16,
                            'i' as u16, 'n' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, '.' as u16, '\n' as u16,
                            0u16,
                        ]
                        .as_ptr(),
                    );
                    return (FALSE) as BOOL;
                }

                TmpIndex += 1;
                TmpCount -= 1;
            }
        } else if lstrcmpi(
            *ServiceArgs.offset(TmpIndex as isize),
            ['b' as u16, 'u' as u16, 'f' as u16, 's' as u16, 'i' as u16, 'z' as u16, 'e' as u16, '=' as u16, 0u16].as_ptr(),
        ) == 0
        {
            TmpIndex += 1;
            TmpCount -= 1;

            if TmpCount > 0 {
                dwValue = _tcstoul(
                    *ServiceArgs.offset(TmpIndex as isize),
                    core::ptr::null_mut(),
                    10,
                );
                if dwValue > 0 {
                    *pdwBufferSize = dwValue;
                }

                TmpIndex += 1;
                TmpCount -= 1;
            }
        } else if lstrcmpi(
            *ServiceArgs.offset(TmpIndex as isize),
            ['r' as u16, 'i' as u16, '=' as u16, 0u16].as_ptr(),
        ) == 0
        {
            TmpIndex += 1;
            TmpCount -= 1;

            // The C tests `>= 0` here (every sibling tests `> 0`): `sc query ri=` as the last
            // argument read ServiceArgs[ArgCount], one past the array.
            if TmpCount > 0 {
                dwValue = _tcstoul(
                    *ServiceArgs.offset(TmpIndex as isize),
                    core::ptr::null_mut(),
                    10,
                );
                if dwValue > 0 {
                    *pdwResumeIndex = dwValue;
                }

                TmpIndex += 1;
                TmpCount -= 1;
            }
        } else if lstrcmpi(
            *ServiceArgs.offset(TmpIndex as isize),
            ['g' as u16, 'r' as u16, 'o' as u16, 'u' as u16, 'p' as u16, '=' as u16, 0u16].as_ptr(),
        ) == 0
        {
            TmpIndex += 1;
            TmpCount -= 1;

            if TmpCount > 0 {
                *ppszGroupName = *ServiceArgs.offset(TmpIndex as isize);

                TmpIndex += 1;
                TmpCount -= 1;
            }
        } else {
            *ppszServiceName = *ServiceArgs.offset(TmpIndex as isize);

            TmpIndex += 1;
            TmpCount -= 1;
        }
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn Query(mut ServiceArgs: *mut LPCTSTR, mut ArgCount: DWORD, mut bExtended: BOOL) -> BOOL {
    let mut pServiceStatus: LPENUM_SERVICE_STATUS_PROCESS = core::ptr::null_mut();
    let mut NumServices: DWORD = 0;
    let mut dwServiceType: DWORD = SERVICE_WIN32;
    let mut dwServiceState: DWORD = SERVICE_ACTIVE;
    let mut dwBufferSize: DWORD = 0;
    let mut dwResumeIndex: DWORD = 0;
    let mut pszGroupName: LPCTSTR = core::ptr::null();
    let mut pszServiceName: LPCTSTR = core::ptr::null();
    let mut i: DWORD;

    #[cfg(scdbg)]
    {
        let mut TmpArgs: *mut LPCTSTR = ServiceArgs;
        let mut TmpCnt: INT = ArgCount as INT;

        _tprintf((&[0x0041, 0x0072, 0x0067, 0x0075, 0x006d, 0x0065, 0x006e, 0x0074, 0x0073, 0x003a, 0x000a, 0x0000u16]).as_ptr() /* _T "Arguments:\n" */);
        while TmpCnt != 0 {
            _tprintf((&[0x0020, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "  %s\n" */, *TmpArgs);
            TmpArgs = TmpArgs.offset(1);
            TmpCnt -= 1;
        }
        _tprintf((&[0x000a, 0x0000u16]).as_ptr() /* _T "\n" */);
    }

    /* Parse arguments */
    if ParseQueryArguments(
        ServiceArgs,
        ArgCount as INT,
        &raw mut dwServiceType,
        &raw mut dwServiceState,
        &raw mut dwBufferSize,
        &raw mut dwResumeIndex,
        &raw mut pszGroupName,
        &raw mut pszServiceName,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    #[cfg(scdbg)]
    {
        _tprintf((&[0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x003a, 0x0020, 0x0025, 0x006c, 0x0078, 0x000a, 0x0000u16]).as_ptr() /* _T "Service type: %lx\n" */, dwServiceType);
        _tprintf((&[0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0073, 0x0074, 0x0061, 0x0074, 0x0065, 0x003a, 0x0020, 0x0025, 0x006c, 0x0078, 0x000a, 0x0000u16]).as_ptr() /* _T "Service state: %lx\n" */, dwServiceState);
        _tprintf((&[0x0042, 0x0075, 0x0066, 0x0066, 0x0065, 0x0072, 0x0020, 0x0073, 0x0069, 0x007a, 0x0065, 0x003a, 0x0020, 0x0025, 0x006c, 0x0075, 0x000a, 0x0000u16]).as_ptr() /* _T "Buffer size: %lu\n" */, dwBufferSize);
        _tprintf((&[0x0052, 0x0065, 0x0073, 0x0075, 0x006d, 0x0065, 0x0020, 0x0069, 0x006e, 0x0064, 0x0065, 0x0078, 0x003a, 0x0020, 0x0025, 0x006c, 0x0075, 0x000a, 0x0000u16]).as_ptr() /* _T "Resume index: %lu\n" */, dwResumeIndex);
        _tprintf((&[0x0047, 0x0072, 0x006f, 0x0075, 0x0070, 0x0020, 0x006e, 0x0061, 0x006d, 0x0065, 0x003a, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "Group name: %s\n" */, pszGroupName);
        _tprintf((&[0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x006e, 0x0061, 0x006d, 0x0065, 0x003a, 0x0020, 0x0025, 0x0073, 0x000a, 0x0000u16]).as_ptr() /* _T "Service name: %s\n" */, pszServiceName);
    }

    if !pszServiceName.is_null() {
        /* Print only the requested service */

        let mut pStatus: LPSERVICE_STATUS_PROCESS = core::ptr::null_mut();

        pStatus = QueryService(pszServiceName);
        if !pStatus.is_null() {
            PrintService(
                pszServiceName,
                core::ptr::null(),
                pStatus,
                bExtended,
            );

            HeapFree(
                GetProcessHeap(),
                0,
                pStatus as PVOID,
            );
        }
    } else {
        /* Print all matching services */

        NumServices = EnumServices(
            &raw mut pServiceStatus,
            dwServiceType,
            dwServiceState,
            dwBufferSize,
            dwResumeIndex,
            pszGroupName,
        );
        if NumServices == 0 {
            return (FALSE) as BOOL;
        }

        i = 0;
        while i < NumServices {
            PrintService(
                (*pServiceStatus.offset(i as isize)).lpServiceName as LPCTSTR,
                (*pServiceStatus.offset(i as isize)).lpDisplayName as LPCTSTR,
                &raw mut (*pServiceStatus.offset(i as isize)).ServiceStatusProcess,
                bExtended,
            );
            i = i.wrapping_add(1);
        }

        #[cfg(scdbg)]
        {
            _tprintf((&[0x006e, 0x0075, 0x006d, 0x0062, 0x0065, 0x0072, 0x0020, 0x003a, 0x0020, 0x0025, 0x006c, 0x0075, 0x000a, 0x0000u16]).as_ptr() /* _T "number : %lu\n" */, NumServices);
        }

        if !pServiceStatus.is_null() {
            HeapFree(
                GetProcessHeap(),
                0,
                pServiceStatus as PVOID,
            );
        }
    }

    return (TRUE) as BOOL;
}
