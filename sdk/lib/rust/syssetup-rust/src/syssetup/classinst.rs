// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from this preamble

#[no_mangle]
pub unsafe extern "system" fn ComputerClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    match InstallFunction {
        _ => {
            return (ERROR_DI_DO_DEFAULT) as u32;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn CriticalDeviceCoInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Context: PCOINSTALLER_CONTEXT_DATA,
) -> DWORD {
    let mut szDeviceId: [WCHAR; 256] = [0; 256];
    let mut szServiceName: [WCHAR; 256] = [0; 256];
    let mut szClassGUID: [WCHAR; 64] = [0; 64];
    let mut dwRequiredSize: DWORD = 0;
    let mut hDriverKey: HKEY = core::ptr::null_mut();
    let mut hDatabaseKey: HKEY = core::ptr::null_mut();
    let mut hDeviceKey: HKEY = core::ptr::null_mut();
    let mut dwDisposition: DWORD = 0;
    let mut Ptr: PWSTR;
    let mut dwError: DWORD = ERROR_SUCCESS;

    // DPRINT removed

    if InstallFunction != (DIF_INSTALLDEVICE) as u32 {
        return ERROR_SUCCESS;
    }

    hDriverKey = SetupDiOpenDevRegKey(
        DeviceInfoSet,
        DeviceInfoData,
        (DICS_FLAG_GLOBAL) as u32,
        0,
        (DIREG_DRV) as u32,
        KEY_READ,
    );
    if hDriverKey == (INVALID_HANDLE_VALUE).cast() {
        if (*Context).PostProcessing != 0 {
            dwError = GetLastError();
            // DPRINT removed
            return 'done: {
                break 'done dwError;
            };
        } else {
            // DPRINT removed
            return (ERROR_DI_POSTPROCESSING_REQUIRED) as u32;
        }
    }

    dwRequiredSize = core::mem::size_of_val(&szDeviceId) as DWORD;
    dwError = (RegQueryValueExW(
        hDriverKey,
        b"M\0a\0t\0c\0h\0i\0n\0g\0D\0e\0v\0i\0c\0e\0I\0d\0\0\0".as_ptr() as _,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        szDeviceId.as_mut_ptr() as PBYTE,
        &mut dwRequiredSize,
    )) as u32;
    RegCloseKey(hDriverKey);
    if dwError != ERROR_SUCCESS {
        if (*Context).PostProcessing != 0 {
            dwError = GetLastError();
            // DPRINT removed
            return 'done: {
                break 'done dwError;
            };
        } else {
            // DPRINT removed
            return (ERROR_DI_POSTPROCESSING_REQUIRED) as u32;
        }
    }

    // DPRINT removed

    dwRequiredSize = 0;
    if SetupDiGetDeviceRegistryPropertyW(
        DeviceInfoSet,
        DeviceInfoData,
        (SPDRP_CLASSGUID) as u32,
        core::ptr::null_mut(),
        szClassGUID.as_mut_ptr() as PBYTE,
        core::mem::size_of_val(&szClassGUID) as DWORD,
        &mut dwRequiredSize,
    ) == 0
    {
        if (*Context).PostProcessing != 0 {
            dwError = GetLastError();
            // DPRINT removed
            return 'done: {
                break 'done dwError;
            };
        } else {
            // DPRINT removed
            return (ERROR_DI_POSTPROCESSING_REQUIRED) as u32;
        }
    }

    // DPRINT removed

    dwRequiredSize = 0;
    if SetupDiGetDeviceRegistryPropertyW(
        DeviceInfoSet,
        DeviceInfoData,
        (SPDRP_SERVICE) as u32,
        core::ptr::null_mut(),
        szServiceName.as_mut_ptr() as PBYTE,
        core::mem::size_of_val(&szServiceName) as DWORD,
        &mut dwRequiredSize,
    ) == 0
    {
        if (*Context).PostProcessing != 0 {
            dwError = GetLastError();
            if dwError != ERROR_FILE_NOT_FOUND {
                // DPRINT removed
                return 'done: {
                    break 'done dwError;
                };
            } else {
                szServiceName[0] = UNICODE_NULL;
                dwError = ERROR_SUCCESS;
            }
        } else {
            // DPRINT removed
            return (ERROR_DI_POSTPROCESSING_REQUIRED) as u32;
        }
    }

    // DPRINT removed

    Ptr = wcschr(szDeviceId.as_mut_ptr(), '\\' as u16);
    if Ptr != core::ptr::null_mut() {
        *Ptr = '#' as u16;

        Ptr = wcschr(Ptr, '\\' as u16);
        if Ptr != core::ptr::null_mut() {
            *Ptr = UNICODE_NULL;
        }
    }

    // DPRINT removed

    dwError = (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        b"S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0C\0r\0i\0t\0i\0c\0a\0l\0D\0e\0v\0i\0c\0e\0D\0a\0t\0a\0b\0a\0s\0e\0\0\0".as_ptr() as _,
        0,
        KEY_WRITE,
        &mut hDatabaseKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return 'done: {
            break 'done dwError;
        };
    }

    dwError = (RegCreateKeyExW(
        hDatabaseKey,
        szDeviceId.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hDeviceKey,
        &mut dwDisposition,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return 'done: {
            break 'done dwError;
        };
    }

    dwError = (RegSetValueExW(
        hDeviceKey,
        b"C\0l\0a\0s\0s\0G\0U\0I\0D\0\0\0".as_ptr() as _,
        0,
        (REG_SZ) as u32,
        szClassGUID.as_ptr() as PBYTE,
        ((wcslen(szClassGUID.as_ptr()) + 1)
            * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return 'done: {
            break 'done dwError;
        };
    }

    if szServiceName[0] != UNICODE_NULL {
        dwError = (RegSetValueExW(
            hDeviceKey,
            b"S\0e\0r\0v\0i\0c\0e\0\0\0".as_ptr() as _,
            0,
            (REG_SZ) as u32,
            szServiceName.as_ptr() as PBYTE,
            ((wcslen(szServiceName.as_ptr()) + 1)
                * (core::mem::size_of::<WCHAR>() as u32) as usize) as u32,
        )) as u32;
        if dwError != ERROR_SUCCESS {
            // DPRINT removed
            return 'done: {
                break 'done dwError;
            };
        }
    }

    'done: {
        if hDeviceKey != core::ptr::null_mut() {
            RegCloseKey(hDeviceKey);
        }

        if hDatabaseKey != core::ptr::null_mut() {
            RegCloseKey(hDatabaseKey);
        }

        // DPRINT removed

        break 'done dwError;
    }
}

#[no_mangle]
pub unsafe extern "system" fn DeviceBayClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    match InstallFunction {
        _ => {
            return (ERROR_DI_DO_DEFAULT) as u32;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn EisaUpHalCoInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Context: PCOINSTALLER_CONTEXT_DATA,
) -> DWORD {
    match InstallFunction {
        _ => {
            return ERROR_SUCCESS;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn HdcClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    // DPRINT!("HdcClassInstaller()\n");
    (ERROR_DI_DO_DEFAULT) as u32
}

#[no_mangle]
pub unsafe extern "system" fn KeyboardClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    match InstallFunction {
        _ => {
            return (ERROR_DI_DO_DEFAULT) as u32;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn MouseClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    match InstallFunction {
        _ => {
            return (ERROR_DI_DO_DEFAULT) as u32;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtApmClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    match InstallFunction {
        _ => {
            return (ERROR_DI_DO_DEFAULT) as u32;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ScsiClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    match InstallFunction {
        _ => {
            return (ERROR_DI_DO_DEFAULT) as u32;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn StorageCoInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Context: PCOINSTALLER_CONTEXT_DATA,
) -> DWORD {
    let mut ulStatus: ULONG = 0;
    let mut ulProblem: ULONG = 0;
    let mut dwBufferSize: DWORD = 0;
    let pszDeviceDescription: PWSTR;
    let ret: CONFIGRET;

    // DPRINT removed

    if InstallFunction != (DIF_INSTALLDEVICE) as u32 {
        return ERROR_SUCCESS;
    }

    if (*Context).PostProcessing != 0 {
        if (*Context).PrivateData != core::ptr::null_mut() {
            pszDeviceDescription = (*Context).PrivateData as PWSTR;

            SetupDiSetDeviceRegistryPropertyW(
                DeviceInfoSet,
                DeviceInfoData,
                (SPDRP_FRIENDLYNAME) as u32,
                pszDeviceDescription as PBYTE,
                ((wcslen(pszDeviceDescription) + 1)
                    * (core::mem::size_of::<WCHAR>() as DWORD) as usize) as u32,
            );

            HeapFree(
                GetProcessHeap(),
                0,
                (*Context).PrivateData,
            );
            (*Context).PrivateData = core::ptr::null_mut();
        }
    } else {
        if DeviceInfoData == core::ptr::null_mut() {
            return ERROR_SUCCESS;
        }

        ret = CM_Get_DevNode_Status(
            &mut ulStatus,
            &mut ulProblem,
            (*DeviceInfoData).DevInst,
            0,
        );
        if ret != (CR_SUCCESS) as u32 {
            return ERROR_SUCCESS;
        }

        if ulStatus & DN_ROOT_ENUMERATED != 0 {
            return ERROR_SUCCESS;
        }

        SetupDiGetDeviceRegistryPropertyW(
            DeviceInfoSet,
            DeviceInfoData,
            (SPDRP_DEVICEDESC) as u32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            &mut dwBufferSize,
        );
        if dwBufferSize == 0 {
            return ERROR_SUCCESS;
        }

        pszDeviceDescription =
            HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (dwBufferSize) as SIZE_T)
                as PWSTR;
        if pszDeviceDescription == core::ptr::null_mut() {
            return ERROR_SUCCESS;
        }

        SetupDiGetDeviceRegistryPropertyW(
            DeviceInfoSet,
            DeviceInfoData,
            (SPDRP_DEVICEDESC) as u32,
            core::ptr::null_mut(),
            pszDeviceDescription as PBYTE,
            dwBufferSize,
            &mut dwBufferSize,
        );

        (*Context).PrivateData = pszDeviceDescription as PVOID;
        return (ERROR_DI_POSTPROCESSING_REQUIRED) as u32;
    }

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn TapeClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    // DPRINT!("TapeClassInstaller()\n");
    (ERROR_DI_DO_DEFAULT) as u32
}

#[no_mangle]
pub unsafe extern "system" fn VolumeClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    // DPRINT!("VolumeClassInstaller()\n");
    (ERROR_DI_DO_DEFAULT) as u32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS system libraries
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/win32/syssetup/classinst.c
//  * PURPOSE:     Class installers
//  * PROGRAMMERS: Copyright 2006 Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// ComputerClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     switch (InstallFunction)
//     {
//         default:
//             DPRINT1("Install function %u ignored\n", InstallFunction);
//             return ERROR_DI_DO_DEFAULT;
//     }
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// CriticalDeviceCoInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData ,
//       PCOINSTALLER_CONTEXT_DATA Context)
// {
//     WCHAR szDeviceId[256];
//     WCHAR szServiceName[256];
//     WCHAR szClassGUID[64];
//     DWORD dwRequiredSize;
//     HKEY hDriverKey = NULL;
//     HKEY hDatabaseKey = NULL, hDeviceKey = NULL;
//     DWORD dwDisposition;
//     PWSTR Ptr;
//     DWORD dwError = ERROR_SUCCESS;
//
//     DPRINT("CriticalDeviceCoInstaller(%lu %p %p %p)\n",
//            InstallFunction, DeviceInfoSet, DeviceInfoData, Context);
//
//     if (InstallFunction != DIF_INSTALLDEVICE)
//         return ERROR_SUCCESS;
//
//     /* Get the MatchingDeviceId property */
//     hDriverKey = SetupDiOpenDevRegKey(DeviceInfoSet,
//                                       DeviceInfoData,
//                                       DICS_FLAG_GLOBAL,
//                                       0,
//                                       DIREG_DRV,
//                                       KEY_READ);
//     if (hDriverKey == INVALID_HANDLE_VALUE)
//     {
//         if (Context->PostProcessing)
//         {
//             dwError = GetLastError();
//             DPRINT1("Failed to open the driver key! (Error %lu)\n", dwError);
//             goto done;
//         }
//         else
//         {
//             DPRINT("Failed to open the driver key! Postprocessing required!\n");
//             return ERROR_DI_POSTPROCESSING_REQUIRED;
//         }
//     }
//
//     dwRequiredSize = sizeof(szDeviceId);
//     dwError = RegQueryValueExW(hDriverKey,
//                                L"MatchingDeviceId",
//                                NULL,
//                                NULL,
//                                (PBYTE)szDeviceId,
//                                &dwRequiredSize);
//     RegCloseKey(hDriverKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         if (Context->PostProcessing)
//         {
//             dwError = GetLastError();
//             DPRINT1("Failed to read the MatchingDeviceId value! (Error %lu)\n", dwError);
//             goto done;
//         }
//         else
//         {
//             DPRINT("Failed to read the MatchingDeviceId value! Postprocessing required!\n");
//             return ERROR_DI_POSTPROCESSING_REQUIRED;
//         }
//     }
//
//     DPRINT("MatchingDeviceId: %S\n", szDeviceId);
//
//     /* Get the ClassGUID property */
//     dwRequiredSize = 0;
//     if (!SetupDiGetDeviceRegistryPropertyW(DeviceInfoSet,
//                                            DeviceInfoData,
//                                            SPDRP_CLASSGUID,
//                                            NULL,
//                                            (PBYTE)szClassGUID,
//                                            sizeof(szClassGUID),
//                                            &dwRequiredSize))
//     {
//         if (Context->PostProcessing)
//         {
//             dwError = GetLastError();
//             DPRINT1("Failed to read the ClassGUID! (Error %lu)\n", dwError);
//             goto done;
//         }
//         else
//         {
//             DPRINT("Failed to read the ClassGUID! Postprocessing required!\n");
//             return ERROR_DI_POSTPROCESSING_REQUIRED;
//         }
//     }
//
//     DPRINT("ClassGUID %S\n", szClassGUID);
//
//     /* Get the Service property (optional) */
//     dwRequiredSize = 0;
//     if (!SetupDiGetDeviceRegistryPropertyW(DeviceInfoSet,
//                                            DeviceInfoData,
//                                            SPDRP_SERVICE,
//                                            NULL,
//                                            (PBYTE)szServiceName,
//                                            sizeof(szServiceName),
//                                            &dwRequiredSize))
//     {
//         if (Context->PostProcessing)
//         {
//             dwError = GetLastError();
//             if (dwError != ERROR_FILE_NOT_FOUND)
//             {
//                 DPRINT1("Failed to read the Service name! (Error %lu)\n", dwError);
//                 goto done;
//             }
//             else
//             {
//                 szServiceName[0] = UNICODE_NULL;
//                 dwError = ERROR_SUCCESS;
//             }
//         }
//         else
//         {
//             DPRINT("Failed to read the Service name! Postprocessing required!\n");
//             return ERROR_DI_POSTPROCESSING_REQUIRED;
//         }
//     }
//
//     DPRINT("Service %S\n", szServiceName);
//
//     /* Replace the first backslash by a number sign */
//     Ptr = wcschr(szDeviceId, '\\');
//     if (Ptr != NULL)
//     {
//         *Ptr = '#';
//
//         /* Terminate the device id at the second backslash */
//         Ptr = wcschr(Ptr, '\\');
//         if (Ptr != NULL)
//             *Ptr = UNICODE_NULL;
//     }
//
//     DPRINT("DeviceId: %S\n", szDeviceId);
//
//     /* Open the critical device database key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SYSTEM\\CurrentControlSet\\Control\\CriticalDeviceDatabase",
//                             0,
//                             KEY_WRITE,
//                             &hDatabaseKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("RegOpenKeyExW failed (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     /* Create a new key for the device */
//     dwError = RegCreateKeyExW(hDatabaseKey,
//                               szDeviceId,
//                               0,
//                               NULL,
//                               REG_OPTION_NON_VOLATILE,
//                               KEY_WRITE,
//                               NULL,
//                               &hDeviceKey,
//                               &dwDisposition);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("RegCreateKeyExW failed (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     /* Set the ClassGUID value */
//     dwError = RegSetValueExW(hDeviceKey,
//                              L"ClassGUID",
//                              0,
//                              REG_SZ,
//                              (PBYTE)szClassGUID,
//                              (wcslen(szClassGUID) + 1) * sizeof(WCHAR));
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("RegSetValueExW failed (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     /* If available, set the Service value */
//     if (szServiceName[0] != UNICODE_NULL)
//     {
//         dwError = RegSetValueExW(hDeviceKey,
//                                  L"Service",
//                                  0,
//                                  REG_SZ,
//                                  (PBYTE)szServiceName,
//                                  (wcslen(szServiceName) + 1) * sizeof(WCHAR));
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("RegSetValueExW failed (Error %lu)\n", dwError);
//             goto done;
//         }
//     }
//
// done:
//     if (hDeviceKey != NULL)
//         RegCloseKey(hDeviceKey);
//
//     if (hDatabaseKey != NULL)
//         RegCloseKey(hDatabaseKey);
//
//     DPRINT("CriticalDeviceCoInstaller() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// DeviceBayClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     switch (InstallFunction)
//     {
//         default:
//             DPRINT("Install function %u ignored\n", InstallFunction);
//             return ERROR_DI_DO_DEFAULT;
//     }
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// EisaUpHalCoInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData ,
//       PCOINSTALLER_CONTEXT_DATA Context)
// {
//     switch (InstallFunction)
//     {
//         default:
//             DPRINT1("Install function %u ignored\n", InstallFunction);
//             return ERROR_SUCCESS;
//     }
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// HdcClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     DPRINT("HdcClassInstaller()\n");
//     return ERROR_DI_DO_DEFAULT;
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// KeyboardClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     switch (InstallFunction)
//     {
//         default:
//             DPRINT("Install function %u ignored\n", InstallFunction);
//             return ERROR_DI_DO_DEFAULT;
//     }
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// MouseClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     switch (InstallFunction)
//     {
//         default:
//             DPRINT("Install function %u ignored\n", InstallFunction);
//             return ERROR_DI_DO_DEFAULT;
//     }
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// NtApmClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     switch (InstallFunction)
//     {
//         default:
//             DPRINT("Install function %u ignored\n", InstallFunction);
//             return ERROR_DI_DO_DEFAULT;
//     }
// }
//
//
// /*
//  * @unimplemented
//  */
// DWORD
// WINAPI
// ScsiClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     switch (InstallFunction)
//     {
//         default:
//             DPRINT("Install function %u ignored\n", InstallFunction);
//             return ERROR_DI_DO_DEFAULT;
//     }
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// StorageCoInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData ,
//       PCOINSTALLER_CONTEXT_DATA Context)
// {
//     ULONG ulStatus, ulProblem;
//     DWORD dwBufferSize = 0;
//     PWSTR pszDeviceDescription;
//     CONFIGRET ret;
//
//     DPRINT("StorageCoInstaller(%u %p %p %p)\n",
//            InstallFunction, DeviceInfoSet, DeviceInfoData, Context);
//
//     if (InstallFunction != DIF_INSTALLDEVICE)
//         return ERROR_SUCCESS;
//
//     if (Context->PostProcessing)
//     {
//         if (Context->PrivateData != NULL)
//         {
//             pszDeviceDescription = (PWSTR)Context->PrivateData;
//
//             /* Store the device description as the friendly name */
//             SetupDiSetDeviceRegistryPropertyW(DeviceInfoSet,
//                                               DeviceInfoData,
//                                               SPDRP_FRIENDLYNAME,
//                                               (PBYTE)pszDeviceDescription,
//                                               (wcslen(pszDeviceDescription) + 1) * sizeof(WCHAR));
//
//             /* Free the device description */
//             HeapFree(GetProcessHeap(), 0, Context->PrivateData);
//             Context->PrivateData = NULL;
//         }
//     }
//     else
//     {
//         if (DeviceInfoData == NULL)
//             return ERROR_SUCCESS;
//
//         ret = CM_Get_DevNode_Status(&ulStatus, &ulProblem, DeviceInfoData->DevInst, 0);
//         if (ret != CR_SUCCESS)
//             return ERROR_SUCCESS;
//
//         if (ulStatus & DN_ROOT_ENUMERATED)
//             return ERROR_SUCCESS;
//
//         /* Get the device description size */
//         SetupDiGetDeviceRegistryPropertyW(DeviceInfoSet,
//                                           DeviceInfoData,
//                                           SPDRP_DEVICEDESC,
//                                           NULL,
//                                           NULL,
//                                           0,
//                                           &dwBufferSize);
//         if (dwBufferSize == 0)
//             return ERROR_SUCCESS;
//
//         /* Allocate the device description buffer */
//         pszDeviceDescription = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwBufferSize);
//         if (pszDeviceDescription == NULL)
//             return ERROR_SUCCESS;
//
//         /* Get the device description */
//         SetupDiGetDeviceRegistryPropertyW(DeviceInfoSet,
//                                           DeviceInfoData,
//                                           SPDRP_DEVICEDESC,
//                                           NULL,
//                                           (PBYTE)pszDeviceDescription,
//                                           dwBufferSize,
//                                           &dwBufferSize);
//
//         Context->PrivateData = (PVOID)pszDeviceDescription;
//         return ERROR_DI_POSTPROCESSING_REQUIRED;
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// TapeClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     DPRINT("TapeClassInstaller()\n");
//     return ERROR_DI_DO_DEFAULT;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// VolumeClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     DPRINT("VolumeClassInstaller()\n");
//     return ERROR_DI_DO_DEFAULT;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
