use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _COMPONENT_INFO {
    pub pszFullInfPath: PWSTR,
    pub pszInfPath: PWSTR,
    pub pszInfSection: PWSTR,
    pub pszComponentId: PWSTR,
    pub pszDescription: PWSTR,
    pub pszClassGuid: PWSTR,
    pub dwCharacteristics: DWORD,
}

pub type COMPONENT_INFO = _COMPONENT_INFO;
pub type PCOMPONENT_INFO = *mut _COMPONENT_INFO;
unsafe extern "C" fn InstallInfSections(
    hWnd: HWND,
    hKey: HKEY,
    InfFile: LPCWSTR,
    InfSection: LPCWSTR,
) -> BOOL {
    let mut Buffer: [u16; MAX_PATH] = core::mem::zeroed();
    let mut hInf = INVALID_HANDLE_VALUE;
    let mut BufferSize: u32;
    let mut Context: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ret: BOOL = FALSE;

    if InfSection.is_null() {
        return FALSE;
    }

    BufferSize = MAX_PATH - 5 - wcslen(InfFile);
    if GetWindowsDirectoryW(Buffer.as_mut_ptr(), BufferSize) > BufferSize {
        SetLastError(ERROR_GEN_FAILURE);
        ret = FALSE;
    } else {
        if Buffer[wcslen(Buffer.as_ptr()) as usize - 1] != '\\' as u16 {
            wcscat(Buffer.as_mut_ptr(), b"\\\0".as_ptr() as *const u16);
        }
        wcscat(Buffer.as_mut_ptr(), b"I\0n\0f\0\\\0\0".as_ptr() as *const u16);
        wcscat(Buffer.as_mut_ptr(), InfFile);

        hInf = SetupOpenInfFileW(Buffer.as_ptr(), core::ptr::null_mut(), INF_STYLE_WIN4, core::ptr::null_mut());
        if hInf == INVALID_HANDLE_VALUE {
            ret = FALSE;
        } else {
            Context = SetupInitDefaultQueueCallback(hWnd);
            if Context.is_null() {
                ret = FALSE;
            } else {
                ret = SetupInstallFromInfSectionW(
                    hWnd,
                    hInf,
                    InfSection,
                    SPINST_ALL,
                    hKey,
                    core::ptr::null_mut(),
                    SP_COPY_NEWER,
                    Some(SetupDefaultQueueCallbackW),
                    Context,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if ret == FALSE {
                    ret = FALSE;
                } else {
                    wcscpy(Buffer.as_mut_ptr(), InfSection);
                    wcscat(Buffer.as_mut_ptr(), b".\0S\0e\0r\0v\0i\0c\0e\0s\0\0".as_ptr() as *const u16);

                    ret = SetupInstallServicesFromInfSectionW(hInf, Buffer.as_ptr(), 0);
                    if ret == FALSE {
                        ret = FALSE;
                    }
                }
            }
        }
    }

    if !Context.is_null() {
        SetupTermDefaultQueueCallback(Context);
    }

    if hInf != INVALID_HANDLE_VALUE {
        SetupCloseInfFile(hInf);
    }

    ret
}
unsafe extern "C" fn CreateInstanceKey(
    pComponentInfo: *mut COMPONENT_INFO,
    pInstanceKey: PHKEY,
) -> i32 {
    let mut szKeyBuffer: [u16; 128] = [0; 128];
    let mut UuidString: LPWSTR = core::ptr::null_mut();
    let mut Uuid: UUID = core::mem::zeroed();
    let mut RpcStatus: u32;
    let mut hInstanceKey: HKEY = core::ptr::null_mut();
    let mut rc: u32;
    let mut ret: i32 = FALSE;

    *pInstanceKey = core::ptr::null_mut();

    wcscpy(szKeyBuffer.as_mut_ptr(), b"SYSTEM\\CurrentControlSet\\Control\\Network\\\0".as_ptr() as LPCWSTR);
    wcscat(szKeyBuffer.as_mut_ptr(), (*pComponentInfo).pszClassGuid);
    wcscat(szKeyBuffer.as_mut_ptr(), b"\\{\0".as_ptr() as LPCWSTR);

    RpcStatus = UuidCreate(&mut Uuid);
    if RpcStatus != RPC_S_OK && RpcStatus != RPC_S_UUID_LOCAL_ONLY {
        return ret;
    }

    RpcStatus = UuidToStringW(&mut Uuid, &mut UuidString);
    if RpcStatus != RPC_S_OK {
        return ret;
    }

    wcscat(szKeyBuffer.as_mut_ptr(), UuidString);
    wcscat(szKeyBuffer.as_mut_ptr(), b"}\0".as_ptr() as LPCWSTR);

    RpcStringFreeW(&mut UuidString);

    rc = RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        szKeyBuffer.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
        core::ptr::null_mut(),
        &mut hInstanceKey,
        core::ptr::null_mut(),
    );
    if rc != ERROR_SUCCESS {
        return ret;
    }

    rc = RegSetValueExW(
        hInstanceKey,
        b"Characteristics\0".as_ptr() as LPCWSTR,
        0,
        REG_DWORD,
        core::ptr::addr_of_mut!((*pComponentInfo).dwCharacteristics) as LPBYTE,
        core::mem::size_of::<u32>() as u32,
    );
    if rc != ERROR_SUCCESS {
        RegCloseKey(hInstanceKey);
        return ret;
    }

    rc = RegSetValueExW(
        hInstanceKey,
        b"ComponentId\0".as_ptr() as LPCWSTR,
        0,
        REG_SZ,
        (*pComponentInfo).pszComponentId as LPBYTE,
        (wcslen((*pComponentInfo).pszComponentId) + 1) * core::mem::size_of::<u16>() as u32,
    );
    if rc != ERROR_SUCCESS {
        RegCloseKey(hInstanceKey);
        return ret;
    }

    rc = RegSetValueExW(
        hInstanceKey,
        b"Description\0".as_ptr() as LPCWSTR,
        0,
        REG_SZ,
        (*pComponentInfo).pszDescription as LPBYTE,
        (wcslen((*pComponentInfo).pszDescription) + 1) * core::mem::size_of::<u16>() as u32,
    );
    if rc != ERROR_SUCCESS {
        RegCloseKey(hInstanceKey);
        return ret;
    }

    rc = RegSetValueExW(
        hInstanceKey,
        b"InfPath\0".as_ptr() as LPCWSTR,
        0,
        REG_SZ,
        (*pComponentInfo).pszInfPath as LPBYTE,
        (wcslen((*pComponentInfo).pszInfPath) + 1) * core::mem::size_of::<u16>() as u32,
    );
    if rc != ERROR_SUCCESS {
        RegCloseKey(hInstanceKey);
        return ret;
    }

    rc = RegSetValueExW(
        hInstanceKey,
        b"InfSection\0".as_ptr() as LPCWSTR,
        0,
        REG_SZ,
        (*pComponentInfo).pszInfSection as LPBYTE,
        (wcslen((*pComponentInfo).pszInfSection) + 1) * core::mem::size_of::<u16>() as u32,
    );
    if rc != ERROR_SUCCESS {
        RegCloseKey(hInstanceKey);
        return ret;
    }

    *pInstanceKey = hInstanceKey;
    ret = TRUE;

    if ret == FALSE {
        RegCloseKey(hInstanceKey);
    }

    ret
}
unsafe extern "C" fn CheckInfFile(
    pszFullInfName: PWSTR,
    pszComponentId: PCWSTR,
    pComponentInfo: *mut COMPONENT_INFO,
) -> u32 {
    let mut szLineBuffer: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut hInf = INVALID_HANDLE_VALUE;
    let mut MfgContext: INFCONTEXT = core::mem::zeroed();
    let mut DevContext: INFCONTEXT = core::mem::zeroed();
    let mut MiscContext: INFCONTEXT = core::mem::zeroed();
    let mut dwLength: u32 = 0;

    hInf = SetupOpenInfFileW(
        pszFullInfName,
        core::ptr::null_mut(),
        INF_STYLE_WIN4,
        core::ptr::null_mut(),
    );
    if hInf == INVALID_HANDLE_VALUE {
        return FALSE;
    }

    if SetupFindFirstLineW(
        hInf,
        b"M\0a\0n\0u\0f\0a\0c\0t\0u\0r\0e\0r\0\0\0".as_ptr() as PCWSTR,
        core::ptr::null_mut(),
        &mut MfgContext,
    ) == 0
    {
        'done: {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            return FALSE;
        }
    }

    'outer: loop {
        if SetupGetStringFieldW(
            &mut MfgContext,
            1,
            szLineBuffer.as_mut_ptr(),
            MAX_PATH,
            core::ptr::null_mut(),
        ) == 0
        {
            break 'outer;
        }

        if SetupFindFirstLineW(
            hInf,
            szLineBuffer.as_ptr(),
            core::ptr::null_mut(),
            &mut DevContext,
        ) == 0
        {
            break 'outer;
        }

        'inner: loop {
            if SetupGetStringFieldW(
                &mut DevContext,
                2,
                szLineBuffer.as_mut_ptr(),
                MAX_PATH,
                core::ptr::null_mut(),
            ) == 0
            {
                break 'inner;
            }

            if _wcsicmp(szLineBuffer.as_ptr(), pszComponentId) == 0 {
                SetupGetStringFieldW(
                    &mut DevContext,
                    1,
                    core::ptr::null_mut(),
                    0,
                    &mut dwLength,
                );

                (*pComponentInfo).pszInfSection = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    dwLength * core::mem::size_of::<u16>() as u32,
                ) as PWSTR;
                if !(*pComponentInfo).pszInfSection.is_null() {
                    SetupGetStringFieldW(
                        &mut DevContext,
                        1,
                        (*pComponentInfo).pszInfSection,
                        dwLength,
                        &mut dwLength,
                    );
                }

                SetupGetStringFieldW(
                    &mut DevContext,
                    0,
                    core::ptr::null_mut(),
                    0,
                    &mut dwLength,
                );

                (*pComponentInfo).pszDescription = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    dwLength * core::mem::size_of::<u16>() as u32,
                ) as PWSTR;
                if !(*pComponentInfo).pszDescription.is_null() {
                    SetupGetStringFieldW(
                        &mut DevContext,
                        0,
                        (*pComponentInfo).pszDescription,
                        dwLength,
                        &mut dwLength,
                    );
                }

                if SetupFindFirstLineW(
                    hInf,
                    b"V\0e\0r\0s\0i\0o\0n\0\0\0".as_ptr() as PCWSTR,
                    b"C\0l\0a\0s\0s\0G\0u\0i\0d\0\0\0".as_ptr() as PCWSTR,
                    &mut MiscContext,
                ) != 0
                {
                    SetupGetStringFieldW(
                        &mut MiscContext,
                        1,
                        core::ptr::null_mut(),
                        0,
                        &mut dwLength,
                    );

                    (*pComponentInfo).pszClassGuid = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        dwLength * core::mem::size_of::<u16>() as u32,
                    ) as PWSTR;
                    if !(*pComponentInfo).pszInfSection.is_null() {
                        SetupGetStringFieldW(
                            &mut MiscContext,
                            1,
                            (*pComponentInfo).pszClassGuid,
                            dwLength,
                            &mut dwLength,
                        );
                    }
                }

                if SetupFindFirstLineW(
                    hInf,
                    (*pComponentInfo).pszInfSection,
                    b"C\0h\0a\0r\0a\0c\0t\0e\0r\0i\0s\0t\0i\0c\0s\0\0\0".as_ptr() as PCWSTR,
                    &mut MiscContext,
                ) != 0
                {
                    SetupGetIntField(
                        &mut MiscContext,
                        1,
                        &mut (*pComponentInfo).dwCharacteristics as *mut u32 as PINT,
                    );
                }

                SetupCloseInfFile(hInf);
                return TRUE;
            }

            if SetupFindNextLine(&mut DevContext, &mut DevContext) == 0 {
                break 'inner;
            }
        }

        if SetupFindNextLine(&mut MfgContext, &mut MfgContext) == 0 {
            break 'outer;
        }
    }

    if hInf != INVALID_HANDLE_VALUE {
        SetupCloseInfFile(hInf);
    }

    return FALSE;
}
unsafe extern "C" fn ScanForInfFile(
    pszComponentId: PCWSTR,
    pComponentInfo: *mut COMPONENT_INFO,
) -> u32 {
    let mut szInfPath: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut szFullInfName: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut szPathBuffer: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut fdw: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFindFile = INVALID_HANDLE_VALUE;
    let mut bFound = FALSE;

    GetWindowsDirectoryW(szInfPath.as_mut_ptr(), MAX_PATH);
    wcscat(szInfPath.as_mut_ptr(), b"\\inf\0".as_ptr() as LPCWSTR);

    wcscpy(szPathBuffer.as_mut_ptr(), szInfPath.as_ptr());
    wcscat(szPathBuffer.as_mut_ptr(), b"\\*.inf\0".as_ptr() as LPCWSTR);

    hFindFile = FindFirstFileW(szPathBuffer.as_ptr(), &mut fdw);
    if hFindFile == INVALID_HANDLE_VALUE {
        return FALSE;
    }

    loop {
        // Faithful to netinstall.c, INCLUDING the bug: this `continue` skips the
        // FindNextFileW at the bottom of the loop, so taking it once spins forever. It is
        // unreachable in practice -- the search pattern is `\inf\*.inf`, which cannot
        // match `.` or `..` -- and repairing an upstream bug inside a translation is how
        // a divergence ships. Three paid samplings all "fixed" it by calling FindNextFileW
        // on this path; the call seal caught every one.
        if wcscmp(fdw.cFileName.as_ptr(), b".\0".as_ptr() as LPCWSTR) == 0 ||
           wcscmp(fdw.cFileName.as_ptr(), b"..\0".as_ptr() as LPCWSTR) == 0 {
            continue;
        }

        wcscpy(szFullInfName.as_mut_ptr(), szInfPath.as_ptr());
        wcscat(szFullInfName.as_mut_ptr(), b"\\\0".as_ptr() as LPCWSTR);
        wcscat(szFullInfName.as_mut_ptr(), fdw.cFileName.as_ptr());

        if CheckInfFile(szFullInfName.as_mut_ptr(), pszComponentId, pComponentInfo) != 0 {
            (*pComponentInfo).pszFullInfPath = HeapAlloc(
                GetProcessHeap(),
                0,
                (wcslen(szFullInfName.as_ptr()) + 1) * core::mem::size_of::<u16>(),
            ) as PWSTR;
            if !(*pComponentInfo).pszFullInfPath.is_null() {
                wcscpy((*pComponentInfo).pszFullInfPath, szFullInfName.as_ptr());
            }

            (*pComponentInfo).pszInfPath = HeapAlloc(
                GetProcessHeap(),
                0,
                (wcslen(fdw.cFileName.as_ptr()) + 1) * core::mem::size_of::<u16>(),
            ) as PWSTR;
            if !(*pComponentInfo).pszInfPath.is_null() {
                wcscpy((*pComponentInfo).pszInfPath, fdw.cFileName.as_ptr());
            }

            (*pComponentInfo).pszComponentId = HeapAlloc(
                GetProcessHeap(),
                0,
                (wcslen(pszComponentId) + 1) * core::mem::size_of::<u16>(),
            ) as PWSTR;
            if !(*pComponentInfo).pszComponentId.is_null() {
                wcscpy((*pComponentInfo).pszComponentId, pszComponentId);
            }

            bFound = TRUE;
            break;
        }

        if FindNextFileW(hFindFile, &mut fdw) == 0 {
            break;
        }
    }

    if hFindFile != INVALID_HANDLE_VALUE {
        FindClose(hFindFile);
    }

    bFound
}
unsafe extern "C" fn StartNetworkServices(
    pComponentInfo: *mut COMPONENT_INFO,
) {
    let mut szServiceName: [u16; 64] = [0; 64];
    let mut hInf = INVALID_HANDLE_VALUE;
    let mut Context: INFCONTEXT = core::mem::zeroed();
    let mut pServiceSection: PWSTR = core::ptr::null_mut();

    let mut hManager = core::ptr::null_mut();
    let mut hService = core::ptr::null_mut();

    hManager = OpenSCManagerW(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        SC_MANAGER_ALL_ACCESS,
    );
    if hManager == core::ptr::null_mut() {
        'done: loop {
            if !pServiceSection.is_null() {
                HeapFree(GetProcessHeap(), 0, pServiceSection as *mut _);
            }

            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }

            if hManager != core::ptr::null_mut() {
                CloseServiceHandle(hManager);
            }
            break 'done;
        }
        return;
    }

    hInf = SetupOpenInfFileW(
        (*pComponentInfo).pszFullInfPath,
        core::ptr::null_mut(),
        INF_STYLE_WIN4,
        core::ptr::null_mut(),
    );
    if hInf == INVALID_HANDLE_VALUE {
        'done: loop {
            if !pServiceSection.is_null() {
                HeapFree(GetProcessHeap(), 0, pServiceSection as *mut _);
            }

            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }

            if hManager != core::ptr::null_mut() {
                CloseServiceHandle(hManager);
            }
            break 'done;
        }
        return;
    }

    pServiceSection = HeapAlloc(
        GetProcessHeap(),
        0,
        (wcslen((*pComponentInfo).pszInfSection) + wcslen(b".Service\0".as_ptr() as LPCWSTR) + 1) * core::mem::size_of::<u16>(),
    ) as PWSTR;
    if pServiceSection == core::ptr::null_mut() {
        'done: loop {
            if !pServiceSection.is_null() {
                HeapFree(GetProcessHeap(), 0, pServiceSection as *mut _);
            }

            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }

            if hManager != core::ptr::null_mut() {
                CloseServiceHandle(hManager);
            }
            break 'done;
        }
        return;
    }

    wcscpy(pServiceSection, (*pComponentInfo).pszInfSection);
    wcscat(pServiceSection, b".Service\0".as_ptr() as LPCWSTR);

    if SetupFindFirstLineW(
        hInf,
        pServiceSection,
        b"AddService\0".as_ptr() as LPCWSTR,
        &mut Context,
    ) == 0
    {
        'done: loop {
            if !pServiceSection.is_null() {
                HeapFree(GetProcessHeap(), 0, pServiceSection as *mut _);
            }

            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }

            if hManager != core::ptr::null_mut() {
                CloseServiceHandle(hManager);
            }
            break 'done;
        }
        return;
    }

    loop {
        SetupGetStringFieldW(
            &mut Context,
            1,
            szServiceName.as_mut_ptr(),
            64,
            core::ptr::null_mut(),
        );

        hService = OpenServiceW(
            hManager,
            szServiceName.as_ptr(),
            SERVICE_START,
        );
        if !hService.is_null() {
            if StartService(hService, 0, core::ptr::null_mut()) == 0 {
            }

            CloseServiceHandle(hService);
        }

        if SetupFindNextLine(&Context, &mut Context) == 0 {
            break;
        }
    }

    'done: loop {
        if !pServiceSection.is_null() {
            HeapFree(GetProcessHeap(), 0, pServiceSection as *mut _);
        }

        if hInf != INVALID_HANDLE_VALUE {
            SetupCloseInfFile(hInf);
        }

        if hManager != core::ptr::null_mut() {
            CloseServiceHandle(hManager);
        }
        break 'done;
    }
}
#[no_mangle]
pub unsafe extern "C" fn InstallNetworkComponent(
    pszComponentId: PCWSTR
) -> BOOL {
    let mut ComponentInfo: COMPONENT_INFO = core::mem::zeroed();
    let mut hInstanceKey: HKEY = core::ptr::null_mut();
    let mut bResult: BOOL = FALSE;

    ZeroMemory(
        core::ptr::addr_of_mut!(ComponentInfo) as *mut core::ffi::c_void,
        core::mem::size_of_val(&COMPONENT_INFO) as usize
    );

    if ScanForInfFile(pszComponentId, core::ptr::addr_of_mut!(ComponentInfo)) == 0 {
        return 'done: {
            if !hInstanceKey.is_null() {
                RegCloseKey(hInstanceKey);
            }

            if !ComponentInfo.pszFullInfPath.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszFullInfPath as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszInfPath.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszInfPath as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszInfSection.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszInfSection as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszComponentId.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszComponentId as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszDescription.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszDescription as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszClassGuid.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszClassGuid as *mut core::ffi::c_void);
            }

            break 'done bResult;
        };
    }

    if CreateInstanceKey(
        core::ptr::addr_of_mut!(ComponentInfo),
        core::ptr::addr_of_mut!(hInstanceKey)
    ) == 0 {
        return 'done: {
            if !hInstanceKey.is_null() {
                RegCloseKey(hInstanceKey);
            }

            if !ComponentInfo.pszFullInfPath.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszFullInfPath as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszInfPath.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszInfPath as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszInfSection.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszInfSection as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszComponentId.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszComponentId as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszDescription.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszDescription as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszClassGuid.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszClassGuid as *mut core::ffi::c_void);
            }

            break 'done bResult;
        };
    }

    if InstallInfSections(
        core::ptr::null_mut(),
        hInstanceKey,
        ComponentInfo.pszInfPath,
        ComponentInfo.pszInfSection
    ) == 0 {
        return 'done: {
            if !hInstanceKey.is_null() {
                RegCloseKey(hInstanceKey);
            }

            if !ComponentInfo.pszFullInfPath.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszFullInfPath as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszInfPath.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszInfPath as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszInfSection.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszInfSection as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszComponentId.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszComponentId as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszDescription.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszDescription as *mut core::ffi::c_void);
            }

            if !ComponentInfo.pszClassGuid.is_null() {
                HeapFree(GetProcessHeap(), 0, ComponentInfo.pszClassGuid as *mut core::ffi::c_void);
            }

            break 'done bResult;
        };
    }

    bResult = TRUE;

    StartNetworkServices(core::ptr::addr_of_mut!(ComponentInfo));

    if !hInstanceKey.is_null() {
        RegCloseKey(hInstanceKey);
    }

    if !ComponentInfo.pszFullInfPath.is_null() {
        HeapFree(GetProcessHeap(), 0, ComponentInfo.pszFullInfPath as *mut core::ffi::c_void);
    }

    if !ComponentInfo.pszInfPath.is_null() {
        HeapFree(GetProcessHeap(), 0, ComponentInfo.pszInfPath as *mut core::ffi::c_void);
    }

    if !ComponentInfo.pszInfSection.is_null() {
        HeapFree(GetProcessHeap(), 0, ComponentInfo.pszInfSection as *mut core::ffi::c_void);
    }

    if !ComponentInfo.pszComponentId.is_null() {
        HeapFree(GetProcessHeap(), 0, ComponentInfo.pszComponentId as *mut core::ffi::c_void);
    }

    if !ComponentInfo.pszDescription.is_null() {
        HeapFree(GetProcessHeap(), 0, ComponentInfo.pszDescription as *mut core::ffi::c_void);
    }

    if !ComponentInfo.pszClassGuid.is_null() {
        HeapFree(GetProcessHeap(), 0, ComponentInfo.pszClassGuid as *mut core::ffi::c_void);
    }

    bResult
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS system libraries
//  * PURPOSE:           Network configuration
//  * FILE:              dll/win32/netcfgx/netinstall.c
//  * PROGRAMER:         Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
// #include <rpc.h>
//
//
// typedef struct _COMPONENT_INFO
// {
//     PWSTR pszFullInfPath;
//     PWSTR pszInfPath;
//     PWSTR pszInfSection;
//     PWSTR pszComponentId;
//     PWSTR pszDescription;
//     PWSTR pszClassGuid;
//     DWORD dwCharacteristics;
// } COMPONENT_INFO, *PCOMPONENT_INFO;
//
//
// /* GLOBALS ******************************************************************/
//
//
// /* FUNCTIONS ****************************************************************/
//
// static
// BOOL
// InstallInfSections(
//      HWND hWnd,
//      HKEY hKey,
//      LPCWSTR InfFile,
//      LPCWSTR InfSection)
// {
//     WCHAR Buffer[MAX_PATH];
//     HINF hInf = INVALID_HANDLE_VALUE;
//     UINT BufferSize;
//     PVOID Context = NULL;
//     BOOL ret = FALSE;
//
//     TRACE("InstallInfSections()\n");
//
//     if (InfSection == NULL)
//         return FALSE;
//
//     /* Get Windows directory */
//     BufferSize = ARRAYSIZE(Buffer) - 5 - wcslen(InfFile);
//     if (GetWindowsDirectoryW(Buffer, BufferSize) > BufferSize)
//     {
//         /* Function failed */
//         SetLastError(ERROR_GEN_FAILURE);
//         goto cleanup;
//     }
//
//     /* We have enough space to add some information in the buffer */
//     if (Buffer[wcslen(Buffer) - 1] != '\\')
//         wcscat(Buffer, L"\\");
//     wcscat(Buffer, L"Inf\\");
//     wcscat(Buffer, InfFile);
//
//     /* Install specified section */
//     hInf = SetupOpenInfFileW(Buffer, NULL, INF_STYLE_WIN4, NULL);
//     if (hInf == INVALID_HANDLE_VALUE)
//         goto cleanup;
//
//     Context = SetupInitDefaultQueueCallback(hWnd);
//     if (Context == NULL)
//         goto cleanup;
//
//     ret = SetupInstallFromInfSectionW(
//                 hWnd, hInf,
//                 InfSection, SPINST_ALL,
//                 hKey, NULL, SP_COPY_NEWER,
//                 SetupDefaultQueueCallbackW, Context,
//                 NULL, NULL);
//     if (ret == FALSE)
//     {
//         ERR("SetupInstallFromInfSectionW(%S) failed (Error %lx)\n", InfSection, GetLastError());
//         goto cleanup;
//     }
//
//     wcscpy(Buffer, InfSection);
//     wcscat(Buffer, L".Services");
//
//     ret = SetupInstallServicesFromInfSectionW(hInf, Buffer, 0);
//     if (ret == FALSE)
//     {
//         ERR("SetupInstallServicesFromInfSectionW(%S) failed (Error %lx)\n", Buffer, GetLastError());
//         goto cleanup;
//     }
//
// cleanup:
//     if (Context)
//         SetupTermDefaultQueueCallback(Context);
//
//     if (hInf != INVALID_HANDLE_VALUE)
//         SetupCloseInfFile(hInf);
//
//     TRACE("InstallInfSections() done %u\n", ret);
//
//     return ret;
// }
//
//
// static
// BOOL
// CreateInstanceKey(
//      PCOMPONENT_INFO pComponentInfo,
//      PHKEY pInstanceKey)
// {
//     WCHAR szKeyBuffer[128];
//     LPWSTR UuidString = NULL;
//     UUID Uuid;
//     RPC_STATUS RpcStatus;
//     HKEY hInstanceKey;
//     DWORD rc;
//     BOOL ret = FALSE;
//
//     TRACE("CreateInstanceKey()\n");
//
//     *pInstanceKey = NULL;
//
//     wcscpy(szKeyBuffer, L"SYSTEM\\CurrentControlSet\\Control\\Network\\");
//     wcscat(szKeyBuffer, pComponentInfo->pszClassGuid);
//     wcscat(szKeyBuffer, L"\\{");
//
//     /* Create a new UUID */
//     RpcStatus = UuidCreate(&Uuid);
//     if (RpcStatus != RPC_S_OK && RpcStatus != RPC_S_UUID_LOCAL_ONLY)
//     {
//         ERR("UuidCreate() failed with RPC status 0x%lx\n", RpcStatus);
//         goto done;
//     }
//
//     RpcStatus = UuidToStringW(&Uuid, &UuidString);
//     if (RpcStatus != RPC_S_OK)
//     {
//         ERR("UuidToStringW() failed with RPC status 0x%lx\n", RpcStatus);
//         goto done;
//     }
//
//     wcscat(szKeyBuffer, UuidString);
//     wcscat(szKeyBuffer, L"}");
//
//     RpcStringFreeW(&UuidString);
//
//     TRACE("szKeyBuffer %S\n", szKeyBuffer);
//
//     rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                          szKeyBuffer,
//                          0,
//                          NULL,
//                          REG_OPTION_NON_VOLATILE,
//                          KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
//                          NULL,
//                          &hInstanceKey,
//                          NULL);
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegCreateKeyExW() failed with error 0x%lx\n", rc);
//         goto done;
//     }
//
//     rc = RegSetValueExW(hInstanceKey,
//                         L"Characteristics",
//                         0,
//                         REG_DWORD,
//                         (LPBYTE)&pComponentInfo->dwCharacteristics,
//                         sizeof(DWORD));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto done;
//     }
//
//     rc = RegSetValueExW(hInstanceKey,
//                         L"ComponentId",
//                         0,
//                         REG_SZ,
//                         (LPBYTE)pComponentInfo->pszComponentId,
//                         (wcslen(pComponentInfo->pszComponentId) + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto done;
//     }
//
//     rc = RegSetValueExW(hInstanceKey,
//                         L"Description",
//                         0,
//                         REG_SZ,
//                         (LPBYTE)pComponentInfo->pszDescription,
//                         (wcslen(pComponentInfo->pszDescription) + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto done;
//     }
//
//     rc = RegSetValueExW(hInstanceKey,
//                         L"InfPath",
//                         0,
//                         REG_SZ,
//                         (LPBYTE)pComponentInfo->pszInfPath,
//                         (wcslen(pComponentInfo->pszInfPath) + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto done;
//     }
//
//     rc = RegSetValueExW(hInstanceKey,
//                         L"InfSection",
//                         0,
//                         REG_SZ,
//                         (LPBYTE)pComponentInfo->pszInfSection,
//                         (wcslen(pComponentInfo->pszInfSection) + 1) * sizeof(WCHAR));
//     if (rc != ERROR_SUCCESS)
//     {
//         ERR("RegSetValueExW() failed with error 0x%lx\n", rc);
//         goto done;
//     }
//
//     *pInstanceKey = hInstanceKey;
//     ret = TRUE;
//
// done:
//     if (ret == FALSE)
//         RegCloseKey(hInstanceKey);
//
//     TRACE("CreateInstanceKey() done %u\n", ret);
//
//     return ret;
// }
//
//
// static
// BOOL
// CheckInfFile(
//      PWSTR pszFullInfName,
//      PCWSTR pszComponentId,
//      PCOMPONENT_INFO pComponentInfo)
// {
//     WCHAR szLineBuffer[MAX_PATH];
//     HINF hInf = INVALID_HANDLE_VALUE;
//     INFCONTEXT MfgContext, DevContext, MiscContext;
//     DWORD dwLength;
//
//     hInf = SetupOpenInfFileW(pszFullInfName,
//                              NULL,
//                              INF_STYLE_WIN4,
//                              NULL);
//     if (hInf == INVALID_HANDLE_VALUE)
//     {
//         ERR("\n");
//         return FALSE;
//     }
//
//     if (!SetupFindFirstLineW(hInf,
//                              L"Manufacturer",
//                              NULL,
//                              &MfgContext))
//     {
//         ERR("No Manufacurer section found!\n");
//         goto done;
//     }
//
//     for (;;)
//     {
//         if (!SetupGetStringFieldW(&MfgContext,
//                                   1,
//                                   szLineBuffer,
//                                   MAX_PATH,
//                                   NULL))
//             break;
//
//         TRACE("Manufacturer: %S\n", szLineBuffer);
//         if (!SetupFindFirstLineW(hInf,
//                                  szLineBuffer,
//                                  NULL,
//                                  &DevContext))
//             break;
//
//         for (;;)
//         {
//             if (!SetupGetStringFieldW(&DevContext,
//                                       2,
//                                       szLineBuffer,
//                                       MAX_PATH,
//                                       NULL))
//                 break;
//
//             TRACE("Device: %S\n", szLineBuffer);
//             if (_wcsicmp(szLineBuffer, pszComponentId) == 0)
//             {
//                 TRACE("Found it!\n");
//
//                 /* Get the section name*/
//                 SetupGetStringFieldW(&DevContext,
//                                      1,
//                                      NULL,
//                                      0,
//                                      &dwLength);
//
//                 pComponentInfo->pszInfSection = HeapAlloc(GetProcessHeap(),
//                                                           0,
//                                                           dwLength * sizeof(WCHAR));
//                 if (pComponentInfo->pszInfSection)
//                 {
//                     SetupGetStringFieldW(&DevContext,
//                                          1,
//                                          pComponentInfo->pszInfSection,
//                                          dwLength,
//                                          &dwLength);
//                 }
//
//                 /* Get the description*/
//                 SetupGetStringFieldW(&DevContext,
//                                      0,
//                                      NULL,
//                                      0,
//                                      &dwLength);
//
//                 pComponentInfo->pszDescription = HeapAlloc(GetProcessHeap(),
//                                                            0,
//                                                            dwLength * sizeof(WCHAR));
//                 if (pComponentInfo->pszDescription)
//                 {
//                     SetupGetStringFieldW(&DevContext,
//                                          0,
//                                          pComponentInfo->pszDescription,
//                                          dwLength,
//                                          &dwLength);
//                 }
//
//                 /* Get the class GUID */
//                 if (SetupFindFirstLineW(hInf,
//                                         L"Version",
//                                         L"ClassGuid",
//                                         &MiscContext))
//                 {
//                     SetupGetStringFieldW(&MiscContext,
//                                          1,
//                                          NULL,
//                                          0,
//                                          &dwLength);
//
//                     pComponentInfo->pszClassGuid = HeapAlloc(GetProcessHeap(),
//                                                              0,
//                                                              dwLength * sizeof(WCHAR));
//                     if (pComponentInfo->pszInfSection)
//                     {
//                         SetupGetStringFieldW(&MiscContext,
//                                              1,
//                                              pComponentInfo->pszClassGuid,
//                                              dwLength,
//                                              &dwLength);
//                     }
//                 }
//
//                 /* Get the Characteristics value */
//                 if (SetupFindFirstLineW(hInf,
//                                         pComponentInfo->pszInfSection,
//                                         L"Characteristics",
//                                         &MiscContext))
//                 {
//                     SetupGetIntField(&MiscContext,
//                                      1,
//                                      (PINT)&pComponentInfo->dwCharacteristics);
//                 }
//
//                 SetupCloseInfFile(hInf);
//                 return TRUE;
//             }
//
//             if (!SetupFindNextLine(&DevContext, &DevContext))
//                 break;
//         }
//
//         if (!SetupFindNextLine(&MfgContext, &MfgContext))
//             break;
//     }
//
// done:
//     if (hInf != INVALID_HANDLE_VALUE)
//         SetupCloseInfFile(hInf);
//
//     return FALSE;
// }
//
//
// static
// BOOL
// ScanForInfFile(
//      PCWSTR pszComponentId,
//      PCOMPONENT_INFO pComponentInfo)
// {
//     WCHAR szInfPath[MAX_PATH];
//     WCHAR szFullInfName[MAX_PATH];
//     WCHAR szPathBuffer[MAX_PATH];
//     WIN32_FIND_DATAW fdw;
//     HANDLE hFindFile = INVALID_HANDLE_VALUE;
//     BOOL bFound = FALSE;
//
//     GetWindowsDirectoryW(szInfPath, MAX_PATH);
//     wcscat(szInfPath, L"\\inf");
//
//     wcscpy(szPathBuffer, szInfPath);
//     wcscat(szPathBuffer, L"\\*.inf");
//
//     hFindFile = FindFirstFileW(szPathBuffer, &fdw);
//     if (hFindFile == INVALID_HANDLE_VALUE)
//         return FALSE;
//
//     for (;;)
//     {
//         if (wcscmp(fdw.cFileName, L".") == 0 ||
//             wcscmp(fdw.cFileName, L"..") == 0)
//             continue;
//
//         TRACE("FileName: %S\n", fdw.cFileName);
//
//         wcscpy(szFullInfName, szInfPath);
//         wcscat(szFullInfName, L"\\");
//         wcscat(szFullInfName, fdw.cFileName);
//
//         TRACE("Full Inf Name: %S\n", szFullInfName);
//         if (CheckInfFile(szFullInfName,
//                          pszComponentId,
//                          pComponentInfo))
//         {
//             pComponentInfo->pszFullInfPath = HeapAlloc(GetProcessHeap(),
//                                                        0,
//                                                        (wcslen(szFullInfName) + 1) * sizeof(WCHAR));
//             if (pComponentInfo->pszFullInfPath)
//                 wcscpy(pComponentInfo->pszFullInfPath, szFullInfName);
//
//             pComponentInfo->pszInfPath = HeapAlloc(GetProcessHeap(),
//                                                    0,
//                                                    (wcslen(fdw.cFileName) + 1) * sizeof(WCHAR));
//             if (pComponentInfo->pszInfPath)
//                 wcscpy(pComponentInfo->pszInfPath, fdw.cFileName);
//
//             pComponentInfo->pszComponentId = HeapAlloc(GetProcessHeap(),
//                                                        0,
//                                                        (wcslen(pszComponentId) + 1) * sizeof(WCHAR));
//             if (pComponentInfo->pszComponentId)
//                 wcscpy(pComponentInfo->pszComponentId, pszComponentId);
//
//             bFound = TRUE;
//             break;
//         }
//
//         if (!FindNextFileW(hFindFile, &fdw))
//             break;
//     }
//
//     if (hFindFile != INVALID_HANDLE_VALUE)
//         FindClose(hFindFile);
//
//     return bFound;
// }
//
// static
// VOID
// StartNetworkServices(
//      PCOMPONENT_INFO pComponentInfo)
// {
//     WCHAR szServiceName[64];
//     HINF hInf = INVALID_HANDLE_VALUE;
//     INFCONTEXT Context;
//     PWSTR pServiceSection = NULL;
//
//     SC_HANDLE hManager = NULL;
//     SC_HANDLE hService = NULL;
//
//     hManager = OpenSCManagerW(NULL,
//                               NULL,
//                               SC_MANAGER_ALL_ACCESS);
//     if (hManager == NULL)
//     {
//         ERR("\n");
//         goto done;
//     }
//
//     hInf = SetupOpenInfFileW(pComponentInfo->pszFullInfPath,
//                              NULL,
//                              INF_STYLE_WIN4,
//                              NULL);
//     if (hInf == INVALID_HANDLE_VALUE)
//     {
//         ERR("\n");
//         goto done;
//     }
//
//     pServiceSection = HeapAlloc(GetProcessHeap(), 0,
//                                 (wcslen(pComponentInfo->pszInfSection) + wcslen(L".Service") + 1) * sizeof(WCHAR));
//     if (pServiceSection == NULL)
//     {
//         ERR("\n");
//         goto done;
//     }
//
//     wcscpy(pServiceSection, pComponentInfo->pszInfSection);
//     wcscat(pServiceSection, L".Service");
//
//     TRACE("ServiceSection: %S\n", pServiceSection);
//
//     if (!SetupFindFirstLineW(hInf,
//                              pServiceSection,
//                              L"AddService",
//                              &Context))
//     {
//         ERR("\n");
//         goto done;
//     }
//
//     for (;;)
//     {
//         SetupGetStringFieldW(&Context,
//                              1,
//                              szServiceName,
//                              64,
//                              NULL);
//
//         hService = OpenServiceW(hManager,
//                                 szServiceName,
//                                 SERVICE_START);
//         if (hService)
//         {
//             if (!StartService(hService, 0, NULL))
//             {
//                 ERR("StartService() %S failed %lu\n", szServiceName, GetLastError());
//             }
//
//             CloseServiceHandle(hService);
//         }
//
//         if (!SetupFindNextLine(&Context, &Context))
//             break;
//     }
//
// done:
//     if (pServiceSection)
//         HeapFree(GetProcessHeap(), 0, pServiceSection);
//
//     if (hInf != INVALID_HANDLE_VALUE)
//         SetupCloseInfFile(hInf);
//
//     if (hManager != NULL)
//         CloseServiceHandle(hManager);
// }
//
//
// BOOL
// InstallNetworkComponent(
//      PCWSTR pszComponentId)
// {
//     COMPONENT_INFO ComponentInfo;
//     HKEY hInstanceKey = NULL;
//     BOOL bResult = FALSE;
//
//     TRACE("InstallNetworkComponent(%S)\n", pszComponentId);
//
//     ZeroMemory(&ComponentInfo, sizeof(COMPONENT_INFO));
//
//     if (!ScanForInfFile(pszComponentId, &ComponentInfo))
//         goto done;
//
//     TRACE("Characteristics: 0x%lx\n", ComponentInfo.dwCharacteristics);
//     TRACE("ComponentId: %S\n", ComponentInfo.pszComponentId);
//     TRACE("Description: %S\n", ComponentInfo.pszDescription);
//     TRACE("InfPath: %S\n", ComponentInfo.pszInfPath);
//     TRACE("InfSection: %S\n", ComponentInfo.pszInfSection);
//     TRACE("ClassGuid: %S\n", ComponentInfo.pszClassGuid);
//
//     if (!CreateInstanceKey(&ComponentInfo,
//                            &hInstanceKey))
//     {
//         ERR("CreateInstanceKey() failed (Error %lx)\n", GetLastError());
//         goto done;
//     }
//
//     if (!InstallInfSections(NULL,
//                             hInstanceKey,
//                             ComponentInfo.pszInfPath,
//                             ComponentInfo.pszInfSection))
//     {
//         ERR("InstallInfSections() failed (Error %lx)\n", GetLastError());
//         goto done;
//     }
//
//     bResult = TRUE;
//
//     StartNetworkServices(&ComponentInfo);
//
// done:
//     if (hInstanceKey != NULL)
//         RegCloseKey(hInstanceKey);
//
//     if (ComponentInfo.pszFullInfPath)
//         HeapFree(GetProcessHeap(), 0, ComponentInfo.pszFullInfPath);
//
//     if (ComponentInfo.pszInfPath)
//         HeapFree(GetProcessHeap(), 0, ComponentInfo.pszInfPath);
//
//     if (ComponentInfo.pszInfSection)
//         HeapFree(GetProcessHeap(), 0, ComponentInfo.pszInfSection);
//
//     if (ComponentInfo.pszComponentId)
//         HeapFree(GetProcessHeap(), 0, ComponentInfo.pszComponentId);
//
//     if (ComponentInfo.pszDescription)
//         HeapFree(GetProcessHeap(), 0, ComponentInfo.pszDescription);
//
//     if (ComponentInfo.pszClassGuid)
//         HeapFree(GetProcessHeap(), 0, ComponentInfo.pszClassGuid);
//
//     return bResult;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
