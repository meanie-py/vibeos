// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _COMPONENT_INFO {
    pub pszInfPath: PWSTR,
    pub pszInfSection: PWSTR,
    pub pszComponentId: PWSTR,
    pub pszDescription: PWSTR,
    pub pszClassGuid: PWSTR,
    pub dwCharacteristics: u32,
}

pub type COMPONENT_INFO = _COMPONENT_INFO;
pub type PCOMPONENT_INFO = *mut _COMPONENT_INFO;
unsafe extern "C" fn InstallInfSections(
    hWnd: HWND,
    hKey: HKEY,
    InfFile: LPCWSTR,
    InfSection: LPCWSTR,
) -> u8 {
    let mut Buffer: [u16; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut hInf = INVALID_HANDLE_VALUE;
    let mut BufferSize: u32;
    let mut Context: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut ret: u8 = (FALSE) as u8;

    // DPRINT removed

    if InfSection == core::ptr::null() {
        return (FALSE) as u8;
    }

    BufferSize = ARRAYSIZE!(Buffer) - 5 - (wcslen(InfFile)) as u32;
    if GetWindowsDirectoryW(Buffer.as_mut_ptr(), BufferSize) > BufferSize {
        SetLastError(ERROR_GEN_FAILURE);
        'cleanup: loop {
            if !Context.is_null() {
                SetupTermDefaultQueueCallback(Context);
            }

            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }

            // DPRINT removed

            return ret;
        }
    }

    if Buffer[wcslen(Buffer.as_ptr()) - 1] != '\\' as u16 {
        wcscat(Buffer.as_mut_ptr(), w!("\\") as LPCWSTR);
    }
    wcscat(Buffer.as_mut_ptr(), w!("Inf\\") as LPCWSTR);
    wcscat(Buffer.as_mut_ptr(), InfFile);

    hInf = SetupOpenInfFileW(Buffer.as_ptr(), core::ptr::null(), (INF_STYLE_WIN4) as u32, core::ptr::null_mut());
    if hInf == INVALID_HANDLE_VALUE {
        'cleanup: loop {
            if !Context.is_null() {
                SetupTermDefaultQueueCallback(Context);
            }

            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }

            // DPRINT removed

            return ret;
        }
    }

    Context = SetupInitDefaultQueueCallback(hWnd);
    if Context.is_null() {
        'cleanup: loop {
            if !Context.is_null() {
                SetupTermDefaultQueueCallback(Context);
            }

            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }

            // DPRINT removed

            return ret;
        }
    }

    ret = (SetupInstallFromInfSectionW(
        hWnd,
        hInf,
        InfSection,
        (SPINST_ALL) as u32,
        hKey,
        core::ptr::null(),
        (SP_COPY_NEWER) as u32,
        Some(SetupDefaultQueueCallbackW),
        Context,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )) as u8;
    if ret == (FALSE) as u8 {
        // DPRINT removed
        'cleanup: loop {
            if !Context.is_null() {
                SetupTermDefaultQueueCallback(Context);
            }

            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }

            // DPRINT removed

            return ret;
        }
    }

    wcscpy(Buffer.as_mut_ptr(), InfSection);
    wcscat(Buffer.as_mut_ptr(), w!(".Services") as LPCWSTR);

    ret = (SetupInstallServicesFromInfSectionW(hInf, Buffer.as_ptr(), 0)) as u8;
    if ret == (FALSE) as u8 {
        // DPRINT removed
        'cleanup: loop {
            if !Context.is_null() {
                SetupTermDefaultQueueCallback(Context);
            }

            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }

            // DPRINT removed

            return ret;
        }
    }

    if !Context.is_null() {
        SetupTermDefaultQueueCallback(Context);
    }

    if hInf != INVALID_HANDLE_VALUE {
        SetupCloseInfFile(hInf);
    }

    // DPRINT removed

    ret
}
unsafe extern "C" fn CreateInstanceKey(
    pComponentInfo: *mut COMPONENT_INFO,
    pInstanceKey: PHKEY,
) -> i32 {
    let mut szKeyBuffer: [u16; 128] = [0; 128];
    let mut UuidString: LPWSTR = core::ptr::null_mut();
    let mut Uuid: UUID = core::mem::zeroed();
    let mut RpcStatus: RPC_STATUS;
    let mut hInstanceKey: HKEY = core::ptr::null_mut();
    let mut rc: u32;
    let mut ret: i32 = (FALSE) as i32;

    // DPRINT removed

    *pInstanceKey = core::ptr::null_mut();

    wcscpy(szKeyBuffer.as_mut_ptr(), w!("SYSTEM\\CurrentControlSet\\Control\\Network\\") as LPCWSTR);
    wcscat(szKeyBuffer.as_mut_ptr(), (*pComponentInfo).pszClassGuid);
    wcscat(szKeyBuffer.as_mut_ptr(), w!("\\{") as LPCWSTR);

    RpcStatus = UuidCreate(&mut Uuid);
    if RpcStatus != RPC_S_OK && RpcStatus != (RPC_S_UUID_LOCAL_ONLY) as i32 {
        // DPRINT removed
        'done: loop {
            if ret == (FALSE) as i32 {
                RegCloseKey(hInstanceKey);
            }
            // DPRINT removed
            return ret;
        }
    }

    RpcStatus = UuidToStringW(&mut Uuid, &mut UuidString);
    if RpcStatus != RPC_S_OK {
        // DPRINT removed
        'done: loop {
            if ret == (FALSE) as i32 {
                RegCloseKey(hInstanceKey);
            }
            // DPRINT removed
            return ret;
        }
    }

    wcscat(szKeyBuffer.as_mut_ptr(), UuidString);
    wcscat(szKeyBuffer.as_mut_ptr(), w!("}") as LPCWSTR);

    RpcStringFreeW(&mut UuidString);

    // DPRINT removed

    rc = (RegCreateKeyExW(
        HKEY_LOCAL_MACHINE,
        szKeyBuffer.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
        core::ptr::null_mut(),
        &mut hInstanceKey,
        core::ptr::null_mut(),
    )) as u32;
    if rc != ERROR_SUCCESS {
        // DPRINT removed
        'done: loop {
            if ret == (FALSE) as i32 {
                RegCloseKey(hInstanceKey);
            }
            // DPRINT removed
            return ret;
        }
    }

    rc = (RegSetValueExW(
        hInstanceKey,
        w!("Characteristics") as LPCWSTR,
        0,
        (REG_DWORD) as u32,
        core::ptr::addr_of_mut!((*pComponentInfo).dwCharacteristics) as LPBYTE,
        core::mem::size_of::<u32>() as u32,
    )) as u32;
    if rc != ERROR_SUCCESS {
        // DPRINT removed
        'done: loop {
            if ret == (FALSE) as i32 {
                RegCloseKey(hInstanceKey);
            }
            // DPRINT removed
            return ret;
        }
    }

    rc = (RegSetValueExW(
        hInstanceKey,
        w!("ComponentId") as LPCWSTR,
        0,
        (REG_SZ) as u32,
        (*pComponentInfo).pszComponentId as LPBYTE,
        ((wcslen((*pComponentInfo).pszComponentId) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32,
    )) as u32;
    if rc != ERROR_SUCCESS {
        // DPRINT removed
        'done: loop {
            if ret == (FALSE) as i32 {
                RegCloseKey(hInstanceKey);
            }
            // DPRINT removed
            return ret;
        }
    }

    rc = (RegSetValueExW(
        hInstanceKey,
        w!("Description") as LPCWSTR,
        0,
        (REG_SZ) as u32,
        (*pComponentInfo).pszDescription as LPBYTE,
        ((wcslen((*pComponentInfo).pszDescription) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32,
    )) as u32;
    if rc != ERROR_SUCCESS {
        // DPRINT removed
        'done: loop {
            if ret == (FALSE) as i32 {
                RegCloseKey(hInstanceKey);
            }
            // DPRINT removed
            return ret;
        }
    }

    rc = (RegSetValueExW(
        hInstanceKey,
        w!("InfPath") as LPCWSTR,
        0,
        (REG_SZ) as u32,
        (*pComponentInfo).pszInfPath as LPBYTE,
        ((wcslen((*pComponentInfo).pszInfPath) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32,
    )) as u32;
    if rc != ERROR_SUCCESS {
        // DPRINT removed
        'done: loop {
            if ret == (FALSE) as i32 {
                RegCloseKey(hInstanceKey);
            }
            // DPRINT removed
            return ret;
        }
    }

    rc = (RegSetValueExW(
        hInstanceKey,
        w!("InfSection") as LPCWSTR,
        0,
        (REG_SZ) as u32,
        (*pComponentInfo).pszInfSection as LPBYTE,
        ((wcslen((*pComponentInfo).pszInfSection) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32,
    )) as u32;
    if rc != ERROR_SUCCESS {
        // DPRINT removed
        'done: loop {
            if ret == (FALSE) as i32 {
                RegCloseKey(hInstanceKey);
            }
            // DPRINT removed
            return ret;
        }
    }

    *pInstanceKey = hInstanceKey;
    ret = (TRUE) as i32;

    if ret == (FALSE) as i32 {
        RegCloseKey(hInstanceKey);
    }

    // DPRINT removed

    return ret;
}
unsafe extern "C" fn CheckInfFile(
    pszFullInfName: PWSTR,
    pszComponentId: PWSTR,
    pComponentInfo: *mut COMPONENT_INFO,
) -> i32 {
    let mut szLineBuffer: [u16; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut hInf = INVALID_HANDLE_VALUE;
    let mut MfgContext: INFCONTEXT = core::mem::zeroed();
    let mut DevContext: INFCONTEXT = core::mem::zeroed();
    let mut MiscContext: INFCONTEXT = core::mem::zeroed();
    let mut dwLength: u32 = 0;

    hInf = SetupOpenInfFileW(
        pszFullInfName,
        core::ptr::null_mut(),
        (INF_STYLE_WIN4) as u32,
        core::ptr::null_mut(),
    );
    if hInf == INVALID_HANDLE_VALUE {
        // DPRINT removed
        return (FALSE) as i32;
    }

    if SetupFindFirstLineW(
        hInf,
        w!("Manufacturer") as LPCWSTR,
        core::ptr::null_mut(),
        &mut MfgContext,
    ) == 0
    {
        // DPRINT removed
        if hInf != INVALID_HANDLE_VALUE {
            SetupCloseInfFile(hInf);
        }
        return (FALSE) as i32;
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
            break;
        }

        // DPRINT removed
        if SetupFindFirstLineW(
            hInf,
            szLineBuffer.as_ptr(),
            core::ptr::null_mut(),
            &mut DevContext,
        ) == 0
        {
            break;
        }

        loop {
            if SetupGetStringFieldW(
                &mut DevContext,
                2,
                szLineBuffer.as_mut_ptr(),
                MAX_PATH,
                core::ptr::null_mut(),
            ) == 0
            {
                break;
            }

            // DPRINT removed
            if _wcsicmp(szLineBuffer.as_ptr(), pszComponentId) == 0
            {
                // DPRINT removed

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
                    ((dwLength as usize) * core::mem::size_of::<u16>()) as SIZE_T,
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
                    ((dwLength as usize) * core::mem::size_of::<u16>()) as SIZE_T,
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
                    w!("Version") as LPCWSTR,
                    w!("ClassGuid") as LPCWSTR,
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
                        ((dwLength as usize) * core::mem::size_of::<u16>()) as SIZE_T,
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
                    w!("Characteristics") as LPCWSTR,
                    &mut MiscContext,
                ) != 0
                {
                    SetupGetIntField(
                        &mut MiscContext,
                        1,
                        core::ptr::addr_of_mut!((*pComponentInfo).dwCharacteristics) as PINT,
                    );
                }

                SetupCloseInfFile(hInf);
                return (TRUE) as i32;
            }

            if SetupFindNextLine(&mut DevContext, &mut DevContext) == 0 {
                break;
            }
        }

        if SetupFindNextLine(&mut MfgContext, &mut MfgContext) == 0 {
            break;
        }
    }

    if hInf != INVALID_HANDLE_VALUE {
        SetupCloseInfFile(hInf);
    }

    return (FALSE) as i32;
}
// HAND-WRITTEN (netinstall.c:401). `--skip-fn`'d: the model kept damaging the deterministic
// call envelope and a retry did not help.
//
// The `continue` on "." / ".." is the C's OWN latent infinite loop — it skips FindNextFileW, so
// the enumeration never advances and the loop spins on that entry forever. It is unreachable in
// practice because the search pattern is `\\*.inf`, which never matches "." or "..". Reproduced
// exactly rather than quietly repaired: a translation that fixes a bug is a translation whose
// behaviour no longer matches the C it is supposed to be replacing, and the difference would
// surface as an unexplained divergence later.
unsafe extern "C" fn ScanForInfFile(
    pszComponentId: PWSTR,
    pComponentInfo: *mut COMPONENT_INFO,
) -> i32 {
    let mut szInfPath: [u16; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut szFullInfName: [u16; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut szPathBuffer: [u16; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut fdw: WIN32_FIND_DATAW = core::mem::zeroed();
    let hFindFile: HANDLE;
    let mut bFound: i32 = FALSE as i32;

    GetWindowsDirectoryW(szInfPath.as_mut_ptr(), MAX_PATH as UINT);
    wcscat(szInfPath.as_mut_ptr(), w!("\\inf"));

    wcscpy(szPathBuffer.as_mut_ptr(), szInfPath.as_ptr());
    wcscat(szPathBuffer.as_mut_ptr(), w!("\\*.inf"));

    hFindFile = FindFirstFileW(szPathBuffer.as_ptr(), &mut fdw);
    if hFindFile == INVALID_HANDLE_VALUE {
        return FALSE as i32;
    }

    loop {
        if wcscmp(fdw.cFileName.as_ptr(), w!(".")) == 0 || wcscmp(fdw.cFileName.as_ptr(), w!("..")) == 0 {
            continue; // the C's own non-advancing continue — see the note above
        }

        wcscpy(szFullInfName.as_mut_ptr(), szInfPath.as_ptr());
        wcscat(szFullInfName.as_mut_ptr(), w!("\\"));
        wcscat(szFullInfName.as_mut_ptr(), fdw.cFileName.as_ptr());

        if CheckInfFile(szFullInfName.as_mut_ptr(), pszComponentId, pComponentInfo) != 0 {
            (*pComponentInfo).pszInfPath = HeapAlloc(
                GetProcessHeap(),
                0,
                ((wcslen(fdw.cFileName.as_ptr()) + 1) * core::mem::size_of::<u16>()) as SIZE_T,
            ) as PWSTR;
            if !(*pComponentInfo).pszInfPath.is_null() {
                wcscpy((*pComponentInfo).pszInfPath, fdw.cFileName.as_ptr());
            }

            (*pComponentInfo).pszComponentId = HeapAlloc(
                GetProcessHeap(),
                0,
                ((wcslen(pszComponentId) + 1) * core::mem::size_of::<u16>()) as SIZE_T,
            ) as PWSTR;
            if !(*pComponentInfo).pszComponentId.is_null() {
                wcscpy((*pComponentInfo).pszComponentId, pszComponentId);
            }

            bFound = TRUE as i32;
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

#[no_mangle]
pub unsafe extern "C" fn InstallNetworkComponent(
    pszComponentId: PWSTR,
) -> BOOL {
    let mut ComponentInfo: COMPONENT_INFO = core::mem::zeroed();
    let mut hInstanceKey: HKEY = core::ptr::null_mut();
    let mut bResult: BOOL = FALSE;

    // DPRINT removed

    ZeroMemory!(&mut ComponentInfo as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<COMPONENT_INFO>());

    if ScanForInfFile(pszComponentId, &mut ComponentInfo) == 0 {
        return 'done: {
            if !hInstanceKey.is_null() {
                RegCloseKey(hInstanceKey);
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
        };
    }

    // DPRINT removed
    // DPRINT removed
    // DPRINT removed
    // DPRINT removed
    // DPRINT removed
    // DPRINT removed

    if CreateInstanceKey(&mut ComponentInfo, &mut hInstanceKey) == 0 {
        // DPRINT removed
        return 'done: {
            if !hInstanceKey.is_null() {
                RegCloseKey(hInstanceKey);
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
        };
    }

    if InstallInfSections(core::ptr::null_mut(), hInstanceKey, ComponentInfo.pszInfPath, ComponentInfo.pszInfSection) == 0 {
        // DPRINT removed
        return 'done: {
            if !hInstanceKey.is_null() {
                RegCloseKey(hInstanceKey);
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
        };
    }

    bResult = (TRUE) as BOOL;

    if !hInstanceKey.is_null() {
        RegCloseKey(hInstanceKey);
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
//  * PURPOSE:           System setup
//  * FILE:              dll/win32/syssetup/netinstall.c
//  * PROGRAMER:         Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// #include <rpc.h>
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _COMPONENT_INFO
// {
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
//     DPRINT("InstallInfSections()\n");
//
//     if (InfSection == NULL)
//         return FALSE;
//
//     /* Get Windows directory */
//     BufferSize = ARRAYSIZE!(Buffer) - 5 - wcslen(InfFile);
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
//         DPRINT1("SetupInstallFromInfSectionW(%S) failed (Error %lx)\n", InfSection, GetLastError());
//         goto cleanup;
//     }
//
//     wcscpy(Buffer, InfSection);
//     wcscat(Buffer, L".Services");
//
//     ret = SetupInstallServicesFromInfSectionW(hInf, Buffer, 0);
//     if (ret == FALSE)
//     {
//         DPRINT1("SetupInstallServicesFromInfSectionW(%S) failed (Error %lx)\n", Buffer, GetLastError());
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
//     DPRINT("InstallInfSections() done %u\n", ret);
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
//     DPRINT("CreateInstanceKey()\n");
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
//         DPRINT1("UuidCreate() failed with RPC status 0x%lx\n", RpcStatus);
//         goto done;
//     }
//
//     RpcStatus = UuidToStringW(&Uuid, &UuidString);
//     if (RpcStatus != RPC_S_OK)
//     {
//         DPRINT1("UuidToStringW() failed with RPC status 0x%lx\n", RpcStatus);
//         goto done;
//     }
//
//     wcscat(szKeyBuffer, UuidString);
//     wcscat(szKeyBuffer, L"}");
//
//     RpcStringFreeW(&UuidString);
//
//     DPRINT("szKeyBuffer %S\n", szKeyBuffer);
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
//         DPRINT1("RegCreateKeyExW() failed with error 0x%lx\n", rc);
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
//         DPRINT1("RegSetValueExW() failed with error 0x%lx\n", rc);
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
//         DPRINT1("RegSetValueExW() failed with error 0x%lx\n", rc);
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
//         DPRINT1("RegSetValueExW() failed with error 0x%lx\n", rc);
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
//         DPRINT1("RegSetValueExW() failed with error 0x%lx\n", rc);
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
//         DPRINT1("RegSetValueExW() failed with error 0x%lx\n", rc);
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
//     DPRINT("CreateInstanceKey() done %u\n", ret);
//
//     return ret;
// }
//
//
// static
// BOOL
// CheckInfFile(
//      PWSTR pszFullInfName,
//      PWSTR pszComponentId,
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
//         DPRINT1("\n");
//         return FALSE;
//     }
//
//     if (!SetupFindFirstLineW(hInf,
//                              L"Manufacturer",
//                              NULL,
//                              &MfgContext))
//     {
//         DPRINT("No Manufacurer section found!\n");
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
//         DPRINT("Manufacturer: %S\n", szLineBuffer);
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
//             DPRINT("Device: %S\n", szLineBuffer);
//             if (_wcsicmp(szLineBuffer, pszComponentId) == 0)
//             {
//                 DPRINT("Found it!\n");
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
//      PWSTR pszComponentId,
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
//         DPRINT("FileName: %S\n", fdw.cFileName);
//
//         wcscpy(szFullInfName, szInfPath);
//         wcscat(szFullInfName, L"\\");
//         wcscat(szFullInfName, fdw.cFileName);
//
//         DPRINT("Full Inf Name: %S\n", szFullInfName);
//         if (CheckInfFile(szFullInfName,
//                          pszComponentId,
//                          pComponentInfo))
//         {
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
//
// BOOL
// InstallNetworkComponent(
//      PWSTR pszComponentId)
// {
//     COMPONENT_INFO ComponentInfo;
//     HKEY hInstanceKey = NULL;
//     BOOL bResult = FALSE;
//
//     DPRINT("InstallNetworkComponent(%S)\n", pszComponentId);
//
//     ZeroMemory!(&ComponentInfo, sizeof(COMPONENT_INFO));
//
//     if (!ScanForInfFile(pszComponentId, &ComponentInfo))
//         goto done;
//
//     DPRINT("Characteristics: 0x%lx\n", ComponentInfo.dwCharacteristics);
//     DPRINT("ComponentId: %S\n", ComponentInfo.pszComponentId);
//     DPRINT("Description: %S\n", ComponentInfo.pszDescription);
//     DPRINT("InfPath: %S\n", ComponentInfo.pszInfPath);
//     DPRINT("InfSection: %S\n", ComponentInfo.pszInfSection);
//     DPRINT("ClassGuid: %S\n", ComponentInfo.pszClassGuid);
//
//     if (!CreateInstanceKey(&ComponentInfo,
//                            &hInstanceKey))
//     {
//         DPRINT1("CreateInstanceKey() failed (Error %lx)\n", GetLastError());
//         goto done;
//     }
//
//     if (!InstallInfSections(NULL,
//                             hInstanceKey,
//                             ComponentInfo.pszInfPath,
//                             ComponentInfo.pszInfSection))
//     {
//         DPRINT1("InstallInfSections() failed (Error %lx)\n", GetLastError());
//         goto done;
//     }
//
//     bResult = TRUE;
//
// done:
//     if (hInstanceKey != NULL)
//         RegCloseKey(hInstanceKey);
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
