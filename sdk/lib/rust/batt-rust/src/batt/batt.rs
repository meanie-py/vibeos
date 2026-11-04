// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_080ED0C6B7A3: [u16; 18] = [
    0x0043u16, 0x004F, 0x004D, 0x0050, 0x004F, 0x0053, 0x0049, 0x0054, 0x0045, 0x005F,
    0x0042, 0x0041, 0x0054, 0x0054, 0x0045, 0x0052, 0x0059, 0x0000,
];
pub static RUSTCONV_WSTR_6A82B430F54F: [u16; 28] = [
    0x0052u16, 0x006F, 0x006F, 0x0074, 0x005C, 0x0043, 0x004F, 0x004D, 0x0050, 0x004F,
    0x0053, 0x0049, 0x0054, 0x0045, 0x005F, 0x0042, 0x0041, 0x0054, 0x0054, 0x0045,
    0x0052, 0x0059, 0x005C, 0x0030, 0x0030, 0x0030, 0x0030, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GUID_DEVCLASS_SYSTEM: GUID = _GUID {
    Data1: 0x4d36e97d as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe325 as ::core::ffi::c_ushort,
    Data3: 0x11ce as ::core::ffi::c_ushort,
    Data4: [
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

unsafe extern "C" fn InstallCompositeBattery(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DeviceInstallParams: PSP_DEVINSTALL_PARAMS_W,
) -> DWORD {
    let mut szDeviceId: [WCHAR; 32] = [0; 32];
    let mut DriverInfoData: SP_DRVINFO_DATA = core::mem::zeroed();
    let mut NewDeviceInfoSet: HDEVINFO = INVALID_HANDLE_VALUE;
    let mut NewDeviceInfoData: PSP_DEVINFO_DATA = core::ptr::null_mut();
    let mut bDeviceRegistered: BOOL = (FALSE as i32);
    let mut bHaveDriverInfoList: BOOL = (FALSE as i32);
    let mut dwError: DWORD = ERROR_SUCCESS;

    DPRINT!(
        "InstallCompositeBattery(%p %p %p)\n",
        DeviceInfoSet,
        DeviceInfoData,
        DeviceInstallParams
    );

    NewDeviceInfoSet = SetupDiCreateDeviceInfoList(
        core::ptr::addr_of!(GUID_DEVCLASS_SYSTEM),
        (*DeviceInstallParams).hwndParent,
    );
    if NewDeviceInfoSet == INVALID_HANDLE_VALUE {
        DPRINT1!(
            "SetupDiCreateDeviceInfoList() failed (Error %lu)\n",
            GetLastError()
        );
        return GetLastError();
    }

    NewDeviceInfoData = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<SP_DEVINFO_DATA>()) as SIZE_T,
    ) as PSP_DEVINFO_DATA;
    if NewDeviceInfoData.is_null() {
        dwError = ERROR_OUTOFMEMORY;
        return dwError;
    }

    'done: {
        if NewDeviceInfoData.is_null() {
            dwError = ERROR_OUTOFMEMORY;
            break 'done;
        }

        (*NewDeviceInfoData).cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD;
        if SetupDiCreateDeviceInfoW(
            NewDeviceInfoSet,
            RUSTCONV_WSTR_6A82B430F54F.as_ptr(),
            core::ptr::addr_of!(GUID_DEVCLASS_SYSTEM),
            core::ptr::null_mut(),
            (*DeviceInstallParams).hwndParent,
            0,
            NewDeviceInfoData,
        ) == 0
        {
            dwError = GetLastError();
            if dwError == ERROR_DEVINST_ALREADY_EXISTS {
                dwError = ERROR_SUCCESS;
                break 'done;
            }

            DPRINT1!(
                "SetupDiCreateDeviceInfoW() failed (Error %lu 0x%08lx)\n",
                dwError,
                dwError
            );
            break 'done;
        }

        if SetupDiRegisterDeviceInfo(
            NewDeviceInfoSet,
            NewDeviceInfoData,
            0,
            None,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            dwError = GetLastError();
            DPRINT1!(
                "SetupDiRegisterDeviceInfo() failed (Error %lu 0x%08lx)\n",
                dwError,
                dwError
            );
            break 'done;
        }

        bDeviceRegistered = (TRUE as i32);

        ZeroMemory!(
            core::ptr::addr_of_mut!(szDeviceId) as *mut core::ffi::c_void,
            core::mem::size_of::<[WCHAR; 32]>(),
        );
        wcscpy(
            core::ptr::addr_of_mut!(szDeviceId) as *mut WCHAR,
            RUSTCONV_WSTR_080ED0C6B7A3.as_ptr(),
        );

        if SetupDiSetDeviceRegistryPropertyW(
            NewDeviceInfoSet,
            NewDeviceInfoData,
            SPDRP_HARDWAREID,
            core::ptr::addr_of_mut!(szDeviceId) as *mut WCHAR as PBYTE,
            ((wcslen(core::ptr::addr_of!(szDeviceId) as *const WCHAR) as usize
                + 2 as usize)
                * core::mem::size_of::<WCHAR>()) as DWORD,
        ) == 0
        {
            dwError = GetLastError();
            DPRINT1!(
                "SetupDiSetDeviceRegistryPropertyW() failed (Error %lu 0x%08lx)\n",
                dwError,
                dwError
            );
            break 'done;
        }

        if SetupDiBuildDriverInfoList(
            NewDeviceInfoSet,
            NewDeviceInfoData,
            SPDIT_COMPATDRIVER,
        ) == 0
        {
            dwError = GetLastError();
            DPRINT1!(
                "SetupDiBuildDriverInfoList() failed (Error %lu 0x%08lx)\n",
                dwError,
                dwError
            );
            break 'done;
        }

        bHaveDriverInfoList = (TRUE as i32);

        DriverInfoData.cbSize = core::mem::size_of::<SP_DRVINFO_DATA>() as DWORD;
        if SetupDiEnumDriverInfo(
            NewDeviceInfoSet,
            NewDeviceInfoData,
            SPDIT_COMPATDRIVER,
            0,
            core::ptr::addr_of_mut!(DriverInfoData),
        ) == 0
        {
            dwError = GetLastError();
            DPRINT1!(
                "SetupDiEnumDriverInfo() failed (Error %lu 0x%08lx)\n",
                dwError,
                dwError
            );
            break 'done;
        }

        if SetupDiSetSelectedDriver(
            NewDeviceInfoSet,
            NewDeviceInfoData,
            core::ptr::addr_of_mut!(DriverInfoData),
        ) == 0
        {
            dwError = GetLastError();
            DPRINT1!(
                "SetupDiSetSelectedDriver() failed (Error %lu 0x%08lx)\n",
                dwError,
                dwError
            );
            break 'done;
        }

        if SetupDiInstallDevice(NewDeviceInfoSet, NewDeviceInfoData) == 0 {
            dwError = GetLastError();
            DPRINT1!(
                "SetupDiInstallDevice() failed (Error %lu 0x%08lx)\n",
                dwError,
                dwError
            );
            break 'done;
        }

        dwError = ERROR_SUCCESS;
    }

    // done:
    if bHaveDriverInfoList != 0 {
        SetupDiDestroyDriverInfoList(
            NewDeviceInfoSet,
            NewDeviceInfoData,
            SPDIT_COMPATDRIVER,
        );
    }

    if bDeviceRegistered != 0 {
        SetupDiDeleteDeviceInfo(NewDeviceInfoSet, NewDeviceInfoData);
    }

    if !NewDeviceInfoData.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            NewDeviceInfoData as PVOID,
        );
    }

    if NewDeviceInfoSet != INVALID_HANDLE_VALUE {
        SetupDiDestroyDeviceInfoList(NewDeviceInfoSet);
    }

    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn BatteryClassInstall(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    let mut DeviceInstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
    let mut dwError: DWORD;

    DPRINT!(
        "BatteryClassInstall(%u %p %p)\n",
        InstallFunction, DeviceInfoSet, DeviceInfoData
    );

    if InstallFunction != DIF_INSTALLDEVICE {
        return ERROR_DI_DO_DEFAULT;
    }

    DeviceInstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
    if SetupDiGetDeviceInstallParamsW(
        DeviceInfoSet,
        DeviceInfoData,
        core::ptr::addr_of_mut!(DeviceInstallParams),
    ) == 0
    {
        DPRINT1!(
            "SetupDiGetDeviceInstallParamsW() failed (Error %lu)\n",
            GetLastError()
        );
        return GetLastError();
    }

    /* Install the composite battery device */
    dwError = InstallCompositeBattery(
        DeviceInfoSet,
        DeviceInfoData,
        core::ptr::addr_of_mut!(DeviceInstallParams),
    );
    if dwError == ERROR_SUCCESS {
        /* Install the battery device */
        dwError = ERROR_DI_DO_DEFAULT;
    }

    return dwError;
}

#[no_mangle]
pub unsafe extern "system" fn BatteryClassCoInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Context: PCOINSTALLER_CONTEXT_DATA,
) -> DWORD {
    match InstallFunction {
        _ => {
            DPRINT!("Install function %u ignored\n", InstallFunction);
            return ERROR_DI_DO_DEFAULT;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDll: HINSTANCE,
    dwReason: DWORD,
    reserved: LPVOID,
) -> BOOL {
    if dwReason == DLL_PROCESS_ATTACH {
        DisableThreadLibraryCalls(hinstDll as HMODULE);
    } else if dwReason == DLL_PROCESS_DETACH {
    }

    return (TRUE) as BOOL;
}
