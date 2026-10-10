use crate::servman::misc::GetError;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn GetSelectedService(Info: PMAIN_WND_INFO) -> *mut ENUM_SERVICE_STATUS_PROCESS {
    let mut lvItem: LVITEM = core::mem::zeroed();
    lvItem.mask = LVIF_PARAM as UINT;
    lvItem.iItem = (*Info).SelectedItem as i32;
    SendMessage(
        (*Info).hListView,
        LVM_GETITEM as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(lvItem) as LPARAM,
    );

    /* return pointer to selected service */
    return lvItem.lParam as *mut ENUM_SERVICE_STATUS_PROCESS;
}

#[no_mangle]
pub unsafe extern "C" fn GetServiceConfig(lpServiceName: LPWSTR) -> LPQUERY_SERVICE_CONFIG {
    let mut lpServiceConfig: LPQUERY_SERVICE_CONFIGW = core::ptr::null_mut();
    let mut hSCManager: SC_HANDLE;
    let mut hService: SC_HANDLE;
    let mut dwBytesNeeded: DWORD = 0;

    hSCManager = OpenSCManagerW(
        core::ptr::null::<WCHAR>(),
        core::ptr::null::<WCHAR>(),
        SC_MANAGER_CONNECT as DWORD,
    );
    if !hSCManager.is_null() {
        hService = OpenServiceW(
            hSCManager,
            lpServiceName as LPCWSTR,
            SERVICE_QUERY_CONFIG as DWORD,
        );
        if !hService.is_null() {
            if QueryServiceConfigW(
                hService,
                core::ptr::null_mut::<_QUERY_SERVICE_CONFIGW>(),
                0 as DWORD,
                core::ptr::addr_of_mut!(dwBytesNeeded),
            ) == 0
            {
                if GetLastError() == ERROR_INSUFFICIENT_BUFFER as DWORD {
                    lpServiceConfig = HeapAlloc(
                        GetProcessHeap(),
                        0 as DWORD,
                        dwBytesNeeded as SIZE_T,
                    ) as LPQUERY_SERVICE_CONFIG as LPQUERY_SERVICE_CONFIGW;
                    if !lpServiceConfig.is_null() {
                        if QueryServiceConfigW(
                            hService,
                            lpServiceConfig,
                            dwBytesNeeded,
                            core::ptr::addr_of_mut!(dwBytesNeeded),
                        ) == 0
                        {
                            HeapFree(
                                GetProcessHeap(),
                                0 as DWORD,
                                lpServiceConfig as PVOID,
                            );
                            lpServiceConfig = core::ptr::null_mut::<_QUERY_SERVICE_CONFIGW>();
                        }
                    }
                }
            }

            CloseServiceHandle(hService);
        }

        CloseServiceHandle(hSCManager);
    }

    return lpServiceConfig as LPQUERY_SERVICE_CONFIG;
}
#[no_mangle]
pub unsafe extern "C" fn SetServiceConfig(mut pServiceConfig: LPQUERY_SERVICE_CONFIG, mut lpServiceName: LPWSTR, mut lpPassword: LPWSTR) -> BOOL {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut::<SC_HANDLE__>();
    let mut hSc: SC_HANDLE = core::ptr::null_mut::<SC_HANDLE__>();
    let mut scLock: SC_LOCK = core::ptr::null_mut::<core::ffi::c_void>();
    let mut bRet: BOOL = (FALSE) as i32;

    hSCManager = OpenSCManagerW(
        core::ptr::null::<WCHAR>(),
        core::ptr::null::<WCHAR>(),
        SC_MANAGER_LOCK as DWORD,
    );
    if !hSCManager.is_null() {
        scLock = LockServiceDatabase(hSCManager);
        if !scLock.is_null() {
            hSc = OpenServiceW(
                hSCManager,
                lpServiceName as LPCWSTR,
                SERVICE_CHANGE_CONFIG as DWORD,
            );
            if !hSc.is_null() {
                if ChangeServiceConfigW(
                    hSc,
                    (*pServiceConfig).dwServiceType,
                    (*pServiceConfig).dwStartType,
                    (*pServiceConfig).dwErrorControl,
                    (*pServiceConfig).lpBinaryPathName as LPCWSTR,
                    (*pServiceConfig).lpLoadOrderGroup as LPCWSTR,
                    if (*pServiceConfig).dwTagId != 0 {
                        core::ptr::addr_of_mut!((*pServiceConfig).dwTagId)
                    } else {
                        core::ptr::null_mut::<DWORD>()
                    },
                    (*pServiceConfig).lpDependencies as LPCWSTR,
                    (*pServiceConfig).lpServiceStartName as LPCWSTR,
                    lpPassword as LPCWSTR,
                    (*pServiceConfig).lpDisplayName as LPCWSTR,
                ) != 0
                {
                    bRet = (TRUE) as i32;
                }

                CloseServiceHandle(hSc);
            }

            UnlockServiceDatabase(scLock);
        }

        CloseServiceHandle(hSCManager);
    }

    if bRet == 0 {
        GetError();
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn GetServiceDescription(lpServiceName: LPWSTR) -> LPWSTR {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hSc: SC_HANDLE = core::ptr::null_mut();
    let mut pServiceDescription: *mut SERVICE_DESCRIPTIONW = core::ptr::null_mut();
    let mut lpDescription: LPWSTR = core::ptr::null_mut();
    let mut BytesNeeded: DWORD = 0;
    let mut dwSize: DWORD = 0;

    hSCManager = OpenSCManagerW(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        SC_MANAGER_ENUMERATE_SERVICE,
    );
    if hSCManager.is_null() {
        GetError();
        return core::ptr::null_mut();
    }

    hSc = OpenServiceW(
        hSCManager,
        lpServiceName,
        SERVICE_QUERY_CONFIG,
    );
    if !hSc.is_null() {
        if QueryServiceConfig2W(
            hSc,
            SERVICE_CONFIG_DESCRIPTION,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(BytesNeeded),
        ) == 0
        {
            if GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                'quit: {
                    pServiceDescription = HeapAlloc(
                        ProcessHeap,
                        0,
                        BytesNeeded as SIZE_T,
                    ) as *mut SERVICE_DESCRIPTIONW;
                    if pServiceDescription.is_null() {
                        break 'quit;
                    }

                    if QueryServiceConfig2W(
                        hSc,
                        SERVICE_CONFIG_DESCRIPTION,
                        pServiceDescription as LPBYTE,
                        BytesNeeded,
                        core::ptr::addr_of_mut!(BytesNeeded),
                    ) != 0
                    {
                        if !(*pServiceDescription).lpDescription.is_null() {
                            dwSize = (wcslen((*pServiceDescription).lpDescription) as DWORD)
                                + 1;
                            lpDescription = HeapAlloc(
                                ProcessHeap,
                                0,
                                (dwSize as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T),
                            ) as LPWSTR;
                            if !lpDescription.is_null() {
                                StringCchCopyW(
                                    lpDescription,
                                    dwSize as usize,
                                    (*pServiceDescription).lpDescription,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // cleanup:
    if !pServiceDescription.is_null() {
        HeapFree(
            ProcessHeap,
            0,
            (pServiceDescription).cast(),
        );
    }
    if !hSCManager.is_null() {
        CloseServiceHandle(hSCManager);
    }
    if !hSc.is_null() {
        CloseServiceHandle(hSc);
    }

    return lpDescription;
}
#[no_mangle]
pub unsafe extern "C" fn SetServiceDescription(lpServiceName: LPWSTR, lpDescription: LPWSTR) -> BOOL {
    let mut hSCManager: SC_HANDLE = core::ptr::null_mut();
    let mut hSc: SC_HANDLE = core::ptr::null_mut();
    let mut scLock: SC_LOCK = core::ptr::null_mut();
    let mut ServiceDescription: SERVICE_DESCRIPTION = SERVICE_DESCRIPTION {
        lpDescription: core::ptr::null_mut(),
    };
    let mut bRet: BOOL = (FALSE) as i32;

    hSCManager = OpenSCManagerW(
        core::ptr::null(),
        core::ptr::null(),
        SC_MANAGER_LOCK as DWORD,
    );
    if !hSCManager.is_null() {
        scLock = LockServiceDatabase(hSCManager);
        if !scLock.is_null() {
            hSc = OpenServiceW(
                hSCManager,
                lpServiceName as LPCWSTR,
                SERVICE_CHANGE_CONFIG as DWORD,
            );
            if !hSc.is_null() {
                ServiceDescription.lpDescription = lpDescription;

                if ChangeServiceConfig2W(
                    hSc,
                    SERVICE_CONFIG_DESCRIPTION as DWORD,
                    core::ptr::addr_of_mut!(ServiceDescription) as LPVOID,
                ) != 0
                {
                    bRet = (TRUE) as i32;
                }

                CloseServiceHandle(hSc);
            }

            UnlockServiceDatabase(scLock);
        }

        CloseServiceHandle(hSCManager);
    }

    if bRet == 0 {
        GetError();
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn FreeServiceList(mut Info: PMAIN_WND_INFO) {
    let mut i: DWORD = 0;
    if !(*Info).pAllServices.is_null() {
        i = 0;
        while i < (*Info).NumServices {
            if !(*(*Info).pAllServices.offset(i as isize))
                .lpServiceName
                .is_null()
            {
                HeapFree(
                    ProcessHeap,
                    0,
                    (*(*Info).pAllServices.offset(i as isize)).lpServiceName as PVOID,
                );
            }
            if !(*(*Info).pAllServices.offset(i as isize))
                .lpDisplayName
                .is_null()
            {
                HeapFree(
                    ProcessHeap,
                    0,
                    (*(*Info).pAllServices.offset(i as isize)).lpDisplayName as PVOID,
                );
            }
            i = i.wrapping_add(1);
        }
        HeapFree(ProcessHeap, 0, (*Info).pAllServices as PVOID);
        (*Info).pAllServices = ::core::ptr::null_mut::<ENUM_SERVICE_STATUS_PROCESS>();
        (*Info).NumServices = 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetServiceList(mut Info: PMAIN_WND_INFO) -> BOOL {
    let mut pServices: *mut ENUM_SERVICE_STATUS_PROCESS = core::ptr::null_mut();
    let mut ScHandle: SC_HANDLE = core::ptr::null_mut();
    let mut bRet: BOOL = (FALSE) as i32;
    let mut BytesNeeded: DWORD = 0;
    let mut ResumeHandle: DWORD = 0;
    let mut NumServices: DWORD = 0;
    let mut i: DWORD = 0;

    FreeServiceList(Info);

    ScHandle = OpenSCManagerW(
        core::ptr::null(),
        core::ptr::null(),
        SC_MANAGER_ENUMERATE_SERVICE,
    );
    if !ScHandle.is_null() {
        if EnumServicesStatusEx(
            ScHandle,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32,
            SERVICE_STATE_ALL,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(BytesNeeded),
            core::ptr::addr_of_mut!(NumServices),
            core::ptr::addr_of_mut!(ResumeHandle),
            core::ptr::null(),
        ) == 0
        {
            /* Call function again if required size was returned */
            if GetLastError() == ERROR_MORE_DATA {
                /* reserve memory for service info array */
                pServices = HeapAlloc(
                    ProcessHeap,
                    0,
                    BytesNeeded as SIZE_T,
                ) as *mut ENUM_SERVICE_STATUS_PROCESS;
                if !pServices.is_null() {
                    /* fill array with service info */
                    if EnumServicesStatusEx(
                        ScHandle,
                        SC_ENUM_PROCESS_INFO,
                        SERVICE_WIN32,
                        SERVICE_STATE_ALL,
                        pServices as LPBYTE,
                        BytesNeeded,
                        core::ptr::addr_of_mut!(BytesNeeded),
                        core::ptr::addr_of_mut!(NumServices),
                        core::ptr::addr_of_mut!(ResumeHandle),
                        core::ptr::null(),
                    ) != 0
                    {
                        bRet = (TRUE) as i32;
                    }
                }
            }
        }
    }

    if !ScHandle.is_null() {
        CloseServiceHandle(ScHandle);
    }

    (*Info).pAllServices = HeapAlloc(
        ProcessHeap,
        HEAP_ZERO_MEMORY,
        (NumServices as SIZE_T)
            .wrapping_mul(core::mem::size_of::<ENUM_SERVICE_STATUS_PROCESS>() as SIZE_T),
    ) as *mut ENUM_SERVICE_STATUS_PROCESS;
    if !(*Info).pAllServices.is_null() {
        (*Info).NumServices = NumServices;

        i = 0;
        while i < NumServices {
            (*(*Info).pAllServices.offset(i as isize)).lpServiceName = HeapAlloc(
                ProcessHeap,
                HEAP_ZERO_MEMORY,
                (wcslen((*pServices.offset(i as isize)).lpServiceName as *const WCHAR) as SIZE_T)
                    .wrapping_add(1 as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;
            if !(*(*Info).pAllServices.offset(i as isize)).lpServiceName.is_null() {
                wcscpy(
                    (*(*Info).pAllServices.offset(i as isize)).lpServiceName,
                    (*pServices.offset(i as isize)).lpServiceName,
                );
            }

            (*(*Info).pAllServices.offset(i as isize)).lpDisplayName = HeapAlloc(
                ProcessHeap,
                HEAP_ZERO_MEMORY,
                (wcslen((*pServices.offset(i as isize)).lpDisplayName as *const WCHAR) as SIZE_T)
                    .wrapping_add(1 as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;
            if !(*(*Info).pAllServices.offset(i as isize)).lpDisplayName.is_null() {
                wcscpy(
                    (*(*Info).pAllServices.offset(i as isize)).lpDisplayName,
                    (*pServices.offset(i as isize)).lpDisplayName,
                );
            }

            CopyMemory!(
                core::ptr::addr_of_mut!((*(*Info).pAllServices.offset(i as isize)).ServiceStatusProcess)
                    as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!((*pServices.offset(i as isize)).ServiceStatusProcess)
                    as *const core::ffi::c_void,
                core::mem::size_of::<SERVICE_STATUS_PROCESS>(),
            );

            i = i.wrapping_add(1);
        }
    }

    if !pServices.is_null() {
        HeapFree(ProcessHeap, 0, pServices as *mut core::ffi::c_void);
    }

    return bRet;
}
#[no_mangle]
pub unsafe extern "C" fn UpdateServiceStatus(pService: *mut ENUM_SERVICE_STATUS_PROCESS) -> BOOL {
    let mut hScm: SC_HANDLE = core::ptr::null_mut();
    let mut bRet: BOOL = FALSE as BOOL;
    hScm = OpenSCManagerW(
        core::ptr::null::<WCHAR>(),
        core::ptr::null::<WCHAR>(),
        SC_MANAGER_ENUMERATE_SERVICE as DWORD,
    );
    if !hScm.is_null() {
        let mut hService: SC_HANDLE = core::ptr::null_mut();
        hService = OpenServiceW(
            hScm,
            (*pService).lpServiceName as LPCWSTR,
            SERVICE_QUERY_STATUS as DWORD,
        );
        if !hService.is_null() {
            let mut size: DWORD = 0;
            QueryServiceStatusEx(
                hService,
                SC_STATUS_PROCESS_INFO,
                &raw mut (*pService).ServiceStatusProcess as LPBYTE,
                core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
                &raw mut size,
            );

            CloseServiceHandle(hService);
            bRet = TRUE as BOOL;
        }

        CloseServiceHandle(hScm);
    }

    return bRet;
}
