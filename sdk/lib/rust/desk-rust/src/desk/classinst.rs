// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn DisplayClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> u32 {
    let mut InstallParams: SP_DEVINSTALL_PARAMS = core::mem::zeroed();
    let mut DriverInfoData: SP_DRVINFO_DATA = core::mem::zeroed();
    let mut hInf: HINF = INVALID_HANDLE_VALUE;
    let mut SectionName: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut ServiceName: [TCHAR; MAX_SERVICE_NAME_LEN] = [0; MAX_SERVICE_NAME_LEN];
    let mut DeviceName: [TCHAR; 12] = [0; 12];
    let mut DriverInfoDetailData: SP_DRVINFO_DETAIL_DATA = core::mem::zeroed();
    let mut DisplayDevice: DISPLAY_DEVICE = core::mem::zeroed();
    let mut hDriverKey: HKEY = INVALID_HANDLE_VALUE;
    let mut hSettingsKey: HKEY = core::ptr::null_mut();
    let mut hServicesKey: HKEY = core::ptr::null_mut();
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut hDeviceSubKey: HKEY = core::ptr::null_mut();
    let mut disposition: u32 = 0;
    let mut cchMax: u32;
    let mut cbData: u32;
    let mut wIndex: u16;
    let mut result: i32;
    let mut rc: i32;
    let mut hr: HRESULT;

    if InstallFunction != DIF_INSTALLDEVICE {
        return ERROR_DI_DO_DEFAULT;
    }

    InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS>() as u32;
    result = SetupDiGetDeviceInstallParams(DeviceInfoSet, DeviceInfoData, &mut InstallParams);
    if result == 0 {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    InstallParams.Flags |= DI_DONOTCALLCONFIGMG;

    result = SetupDiSetDeviceInstallParams(DeviceInfoSet, DeviceInfoData, &mut InstallParams);
    if result == 0 {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    result = SetupDiInstallDevice(DeviceInfoSet, DeviceInfoData);
    if result == 0 {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    DriverInfoData.cbSize = core::mem::size_of::<SP_DRVINFO_DATA>() as u32;
    result = SetupDiGetSelectedDriver(DeviceInfoSet, DeviceInfoData, &mut DriverInfoData);
    if result == 0 {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    DriverInfoDetailData.cbSize = core::mem::size_of::<SP_DRVINFO_DETAIL_DATA>() as u32;
    result = SetupDiGetDriverInfoDetail(
        DeviceInfoSet,
        DeviceInfoData,
        &mut DriverInfoData,
        &mut DriverInfoDetailData,
        core::mem::size_of::<SP_DRVINFO_DETAIL_DATA>() as u32,
        core::ptr::null_mut(),
    );
    if result == 0 && GetLastError() != ERROR_INSUFFICIENT_BUFFER {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    hInf = SetupOpenInfFile(
        DriverInfoDetailData.InfFileName.as_ptr(),
        core::ptr::null_mut(),
        INF_STYLE_WIN4,
        core::ptr::null_mut(),
    );
    if hInf == INVALID_HANDLE_VALUE {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    // C: `MAX_PATH - (sizeof(_T(".SoftwareSettings")) / sizeof(TCHAR))` — the ARRAY size of the
    // literal, i.e. 17 characters plus the terminator. `wide!` is the array form of `w!` and
    // counts the literal itself; `size_of_val` on `w!`'s POINTER would have measured 4 or 8.
    cchMax = MAX_PATH - wide!(".SoftwareSettings").len() as u32;
    result = SetupDiGetActualSectionToInstall(
        hInf,
        DriverInfoDetailData.SectionName.as_ptr(),
        SectionName.as_mut_ptr(),
        cchMax,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if result == 0 {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }
    hr = StringCbCat(
        SectionName.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of_val(&SectionName),
        w!(".SoftwareSettings"),
    );
    if hr < 0 {
        rc = ERROR_INSUFFICIENT_BUFFER;
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    hDriverKey = SetupDiOpenDevRegKey(
        DeviceInfoSet,
        DeviceInfoData,
        DICS_FLAG_GLOBAL,
        0,
        DIREG_DRV,
        KEY_CREATE_SUB_KEY,
    );
    if hDriverKey == INVALID_HANDLE_VALUE {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }
    rc = RegCreateKeyEx(
        hDriverKey,
        w!("Settings") as *const u16,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        // The C's `#if _WIN32_WINNT >= 0x502` — a WINDOWS VERSION test, not an architecture
        // one. desk compiles with -D_WIN32_WINNT=0x502 on all three targets, so the >= 0x502
        // arm is the only live branch and KEY_ALL_ACCESS is dead source. The translation had
        // rendered it as `#[cfg(target_pointer_width = ...)]` on the two argument expressions,
        // which is wrong twice: attributes on expressions are not stable Rust at all, and had
        // they been, i386 would have asked for ALL_ACCESS on a key it only reads and writes.
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        &mut hSettingsKey,
        &mut disposition,
    );
    if rc != ERROR_SUCCESS {
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    result = SetupInstallFromInfSection(
        InstallParams.hwndParent,
        hInf,
        SectionName.as_ptr(),
        SPINST_REGISTRY,
        hSettingsKey,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if result == 0 {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    result = SetupDiGetDeviceRegistryProperty(
        DeviceInfoSet,
        DeviceInfoData,
        SPDRP_SERVICE,
        core::ptr::null_mut(),
        ServiceName.as_mut_ptr() as PBYTE,
        MAX_PATH * core::mem::size_of::<TCHAR>() as u32,
        core::ptr::null_mut(),
    );
    if result == 0 {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    rc = RegOpenKeyEx(
        HKEY_LOCAL_MACHINE,
        w!("SYSTEM\\CurrentControlSet\\Services"),
        0,
        KEY_ENUMERATE_SUB_KEYS,
        &mut hServicesKey,
    );
    if rc != ERROR_SUCCESS {
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }
    rc = RegOpenKeyEx(
        hServicesKey,
        ServiceName.as_ptr(),
        0,
        KEY_CREATE_SUB_KEY,
        &mut hServiceKey,
    );
    if rc != ERROR_SUCCESS {
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    wIndex = 0;
    'device_loop: loop {
        if wIndex >= 9999 {
            break;
        }

        _stprintf(DeviceName.as_mut_ptr(), w!("Device%hu"), wIndex as c_int);

        rc = RegCreateKeyEx(
            hServiceKey,
            DeviceName.as_ptr(),
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            core::ptr::null_mut(),
            &mut hDeviceSubKey,
            &mut disposition,
        );
        if rc != ERROR_SUCCESS {
            // DPRINT removed
            'cleanup: loop {
                if hInf != INVALID_HANDLE_VALUE {
                    SetupCloseInfFile(hInf);
                }
                if hDriverKey != INVALID_HANDLE_VALUE {
                    RegCloseKey(hDriverKey);
                }
                if !hSettingsKey.is_null() {
                    RegCloseKey(hSettingsKey);
                }
                if !hServicesKey.is_null() {
                    RegCloseKey(hServicesKey);
                }
                if !hServiceKey.is_null() {
                    RegCloseKey(hServiceKey);
                }
                if !hDeviceSubKey.is_null() {
                    RegCloseKey(hDeviceSubKey);
                }
                return rc as u32;
            }
        }

        if disposition == REG_CREATED_NEW_KEY {
            break 'device_loop;
        }

        if wIndex == 9999 {
            rc = ERROR_GEN_FAILURE;
            // DPRINT removed
            'cleanup: loop {
                if hInf != INVALID_HANDLE_VALUE {
                    SetupCloseInfFile(hInf);
                }
                if hDriverKey != INVALID_HANDLE_VALUE {
                    RegCloseKey(hDriverKey);
                }
                if !hSettingsKey.is_null() {
                    RegCloseKey(hSettingsKey);
                }
                if !hServicesKey.is_null() {
                    RegCloseKey(hServicesKey);
                }
                if !hServiceKey.is_null() {
                    RegCloseKey(hServiceKey);
                }
                if !hDeviceSubKey.is_null() {
                    RegCloseKey(hDeviceSubKey);
                }
                return rc as u32;
            }
        }

        RegCloseKey(hDeviceSubKey);
        hDeviceSubKey = core::ptr::null_mut();

        wIndex = wIndex.wrapping_add(1);
    }

    result = SetupInstallFromInfSection(
        InstallParams.hwndParent,
        hInf,
        SectionName.as_ptr(),
        SPINST_REGISTRY,
        hDeviceSubKey,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if result == 0 {
        rc = GetLastError();
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    cbData = (_tcslen(DriverInfoData.Description.as_ptr()) + 1) as u32 * core::mem::size_of::<TCHAR>() as u32;
    rc = RegSetValueEx(
        hDeviceSubKey,
        w!("Device Description"),
        0,
        REG_SZ,
        DriverInfoData.Description.as_ptr() as *const u8,
        cbData,
    );
    if rc != ERROR_SUCCESS {
        // DPRINT removed
        'cleanup: loop {
            if hInf != INVALID_HANDLE_VALUE {
                SetupCloseInfFile(hInf);
            }
            if hDriverKey != INVALID_HANDLE_VALUE {
                RegCloseKey(hDriverKey);
            }
            if !hSettingsKey.is_null() {
                RegCloseKey(hSettingsKey);
            }
            if !hServicesKey.is_null() {
                RegCloseKey(hServicesKey);
            }
            if !hServiceKey.is_null() {
                RegCloseKey(hServiceKey);
            }
            if !hDeviceSubKey.is_null() {
                RegCloseKey(hDeviceSubKey);
            }
            return rc as u32;
        }
    }

    DisplayDevice.cb = core::mem::size_of::<DISPLAY_DEVICE>() as u32;
    EnumDisplayDevices(
        core::ptr::null_mut(),
        0,
        &mut DisplayDevice,
        0,
    );

    rc = ERROR_SUCCESS;

    if hInf != INVALID_HANDLE_VALUE {
        SetupCloseInfFile(hInf);
    }
    if hDriverKey != INVALID_HANDLE_VALUE {
        RegCloseKey(hDriverKey);
    }
    if !hSettingsKey.is_null() {
        RegCloseKey(hSettingsKey);
    }
    if !hServicesKey.is_null() {
        RegCloseKey(hServicesKey);
    }
    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }
    if !hDeviceSubKey.is_null() {
        RegCloseKey(hDeviceSubKey);
    }

    rc as u32
}

#[no_mangle]
pub unsafe extern "system" fn MonitorClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    ERROR_DI_DO_DEFAULT
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * FILE:            dll/cpl/desk/classinst.c
//  * PURPOSE:         Class installers
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "desk.h"
//
// //#define NDEBUG
// #include <debug.h>
//
// DWORD WINAPI
// DisplayClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     SP_DEVINSTALL_PARAMS InstallParams;
//     SP_DRVINFO_DATA DriverInfoData;
//     HINF hInf = INVALID_HANDLE_VALUE;
//     TCHAR SectionName[MAX_PATH];
//     TCHAR ServiceName[MAX_SERVICE_NAME_LEN];
//     TCHAR DeviceName[12];
//     SP_DRVINFO_DETAIL_DATA DriverInfoDetailData;
//     DISPLAY_DEVICE DisplayDevice;
//     HKEY hDriverKey = INVALID_HANDLE_VALUE; /* SetupDiOpenDevRegKey returns INVALID_HANDLE_VALUE in case of error! */
//     HKEY hSettingsKey = NULL;
//     HKEY hServicesKey = NULL;
//     HKEY hServiceKey = NULL;
//     HKEY hDeviceSubKey = NULL;
//     DWORD disposition, cchMax, cbData;
//     WORD wIndex;
//     BOOL result;
//     LONG rc;
//     HRESULT hr;
//
//     if (InstallFunction != DIF_INSTALLDEVICE)
//         return ERROR_DI_DO_DEFAULT;
//
//     /* Set DI_DONOTCALLCONFIGMG flag */
//     InstallParams.cbSize = sizeof(InstallParams);
//     result = SetupDiGetDeviceInstallParams(DeviceInfoSet, DeviceInfoData, &InstallParams);
//     if (!result)
//     {
//         rc = GetLastError();
//         DPRINT("SetupDiGetDeviceInstallParams() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     InstallParams.Flags |= DI_DONOTCALLCONFIGMG;
//
//     result = SetupDiSetDeviceInstallParams(DeviceInfoSet, DeviceInfoData, &InstallParams);
//     if (!result)
//     {
//         rc = GetLastError();
//         DPRINT("SetupDiSetDeviceInstallParams() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* Do normal install */
//     result = SetupDiInstallDevice(DeviceInfoSet, DeviceInfoData);
//     if (!result)
//     {
//         rc = GetLastError();
//         DPRINT("SetupDiInstallDevice() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* Get .inf file name and section name */
//     DriverInfoData.cbSize = sizeof(DriverInfoData);
//     result = SetupDiGetSelectedDriver(DeviceInfoSet, DeviceInfoData, &DriverInfoData);
//     if (!result)
//     {
//         rc = GetLastError();
//         DPRINT("SetupDiGetSelectedDriver() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     DriverInfoDetailData.cbSize = sizeof(DriverInfoDetailData);
//     result = SetupDiGetDriverInfoDetail(DeviceInfoSet, DeviceInfoData,
//                                         &DriverInfoData, &DriverInfoDetailData,
//                                         sizeof(DriverInfoDetailData), NULL);
//     if (!result && GetLastError() != ERROR_INSUFFICIENT_BUFFER)
//     {
//         rc = GetLastError();
//         DPRINT("SetupDiGetDriverInfoDetail() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     hInf = SetupOpenInfFile(DriverInfoDetailData.InfFileName, NULL, INF_STYLE_WIN4, NULL);
//     if (hInf == INVALID_HANDLE_VALUE)
//     {
//         rc = GetLastError();
//         DPRINT("SetupOpenInfFile() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     cchMax = MAX_PATH - (sizeof(_T(".SoftwareSettings")) / sizeof(TCHAR));
//     result = SetupDiGetActualSectionToInstall(hInf,
//                                               DriverInfoDetailData.SectionName,
//                                               SectionName,
//                                               cchMax,
//                                               NULL,
//                                               NULL);
//     if (!result)
//     {
//         rc = GetLastError();
//         DPRINT("SetupDiGetActualSectionToInstall() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//     hr = StringCbCat(SectionName, sizeof(SectionName), _T(".SoftwareSettings"));
//     if (FAILED(hr))
//     {
//         rc = ERROR_INSUFFICIENT_BUFFER;
//         goto cleanup;
//     }
//
//     /* Open driver registry key and create Settings subkey */
//     hDriverKey = SetupDiOpenDevRegKey(
//         DeviceInfoSet, DeviceInfoData,
//         DICS_FLAG_GLOBAL, 0, DIREG_DRV,
//         KEY_CREATE_SUB_KEY);
//     if (hDriverKey == INVALID_HANDLE_VALUE)
//     {
//         rc = GetLastError();
//         DPRINT("SetupDiOpenDevRegKey() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//     rc = RegCreateKeyEx(
//         hDriverKey, L"Settings",
//         0, NULL, REG_OPTION_NON_VOLATILE,
// #if _WIN32_WINNT >= 0x502
//         KEY_READ | KEY_WRITE,
// #else
//         KEY_ALL_ACCESS,
// #endif
//         NULL, &hSettingsKey, &disposition);
//     if (rc != ERROR_SUCCESS)
//     {
//         DPRINT("RegCreateKeyEx() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* Install .SoftwareSettings to Settings subkey */
//     result = SetupInstallFromInfSection(
//         InstallParams.hwndParent, hInf, SectionName,
//         SPINST_REGISTRY, hSettingsKey,
//         NULL, 0, NULL, NULL,
//         NULL, NULL);
//     if (!result)
//     {
//         rc = GetLastError();
//         DPRINT("SetupInstallFromInfSection() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* Get service name and open service registry key */
//     result = SetupDiGetDeviceRegistryProperty(
//         DeviceInfoSet, DeviceInfoData,
//         SPDRP_SERVICE, NULL,
//         (PBYTE)ServiceName, MAX_SERVICE_NAME_LEN * sizeof(TCHAR), NULL);
//     if (!result)
//     {
//         rc = GetLastError();
//         DPRINT("SetupDiGetDeviceRegistryProperty() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegOpenKeyEx(
//         HKEY_LOCAL_MACHINE, _T("SYSTEM\\CurrentControlSet\\Services"),
//         0, KEY_ENUMERATE_SUB_KEYS, &hServicesKey);
//     if (rc != ERROR_SUCCESS)
//     {
//         DPRINT("RegOpenKeyEx() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//     rc = RegOpenKeyEx(
//         hServicesKey, ServiceName,
//         0, KEY_CREATE_SUB_KEY, &hServiceKey);
//     if (rc != ERROR_SUCCESS)
//     {
//         DPRINT("RegOpenKeyEx() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* Create a new DeviceX subkey */
//     for (wIndex = 0; wIndex < 9999; wIndex++)
//     {
//         _stprintf(DeviceName, _T("Device%hu"), wIndex);
//
//         rc = RegCreateKeyEx(
//             hServiceKey, DeviceName, 0, NULL,
//             REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, NULL,
//             &hDeviceSubKey, &disposition);
//         if (rc != ERROR_SUCCESS)
//         {
//             DPRINT("RegCreateKeyEx() failed with error 0x%lx\n", rc);
//             goto cleanup;
//         }
//
//         if (disposition == REG_CREATED_NEW_KEY)
//             break;
//
//         if (wIndex == 9999)
//         {
//             rc = ERROR_GEN_FAILURE;
//             DPRINT("RegCreateKeyEx() failed\n");
//             goto cleanup;
//         }
//
//         RegCloseKey(hDeviceSubKey);
//         hDeviceSubKey = NULL;
//     }
//
//     /* Install SoftwareSettings section */
//     /* Yes, we're installing this section for the second time.
//      * We don't want to create a link to Settings subkey */
//     result = SetupInstallFromInfSection(
//         InstallParams.hwndParent, hInf, SectionName,
//         SPINST_REGISTRY, hDeviceSubKey,
//         NULL, 0, NULL, NULL,
//         NULL, NULL);
//     if (!result)
//     {
//         rc = GetLastError();
//         DPRINT("SetupInstallFromInfSection() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* Add Device Description string */
//     cbData = (DWORD)(_tcslen(DriverInfoData.Description) + 1) * sizeof(TCHAR);
//     rc = RegSetValueEx(hDeviceSubKey,
//                        _T("Device Description"),
//                        0,
//                        REG_SZ,
//                        (const BYTE*)DriverInfoData.Description,
//                        cbData);
//     if (rc != ERROR_SUCCESS)
//     {
//         DPRINT("RegSetValueEx() failed with error 0x%lx\n", rc);
//         goto cleanup;
//     }
//
//     /* FIXME: install OpenGLSoftwareSettings section */
//
//     /* Reenumerate display devices ; this will rescan for potential new devices */
//     DisplayDevice.cb = sizeof(DISPLAY_DEVICE);
//     EnumDisplayDevices(NULL, 0, &DisplayDevice, 0);
//
//     rc = ERROR_SUCCESS;
//
// cleanup:
//     if (hInf != INVALID_HANDLE_VALUE)
//         SetupCloseInfFile(hInf);
//     if (hDriverKey != INVALID_HANDLE_VALUE)
//     {
//         /* SetupDiOpenDevRegKey returns INVALID_HANDLE_VALUE in case of error! */
//         RegCloseKey(hDriverKey);
//     }
//     if (hSettingsKey != NULL)
//         RegCloseKey(hSettingsKey);
//     if (hServicesKey != NULL)
//         RegCloseKey(hServicesKey);
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//     if (hDeviceSubKey != NULL)
//         RegCloseKey(hDeviceSubKey);
//
//     return rc;
// }
//
// DWORD WINAPI
// MonitorClassInstaller(
//      DI_FUNCTION InstallFunction,
//      HDEVINFO DeviceInfoSet,
//      PSP_DEVINFO_DATA DeviceInfoData )
// {
//     return ERROR_DI_DO_DEFAULT;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
