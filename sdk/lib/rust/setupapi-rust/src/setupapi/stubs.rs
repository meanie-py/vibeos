use crate::setupapi::queue::file_queue;
use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn pSetupRegistryDelnode(x: DWORD, y: DWORD) -> BOOL {
    // FIXME("%08x %08x: stub\n", x, y);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupPromptReboot(
    file_queue: HSPFILEQ,
    owner: HWND,
    scan_only: BOOL,
) -> INT {
    // FIXME("%p, %p, %d\n", file_queue, owner, scan_only);
    0
}

#[no_mangle]
pub unsafe extern "system" fn SetupAddToSourceListA(flags: DWORD, source: PCSTR) -> BOOL {
    // FIXME("0x%08x %s: stub\n", flags, debugstr_a(source));
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupAddToSourceListW(flags: DWORD, source: PCWSTR) -> BOOL {
    // FIXME("0x%08x %s: stub\n", flags, debugstr_w(source));
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupSetSourceListA(
    flags: DWORD,
    list: *mut PCSTR,
    count: UINT,
) -> BOOL {
    // FIXME("0x%08x %p %d\n", flags, list, count);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupSetSourceListW(
    flags: DWORD,
    list: *mut PCWSTR,
    count: UINT,
) -> BOOL {
    // FIXME("0x%08x %p %d\n", flags, list, count);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiRemoveDevice(DeviceInfoSet: HDEVINFO, DeviceInfoData: PSP_DEVINFO_DATA) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiUnremoveDevice(DeviceInfoSet: HDEVINFO, DeviceInfoData: PSP_DEVINFO_DATA) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiRemoveDeviceInterface(info: HDEVINFO, data: PSP_DEVICE_INTERFACE_DATA) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDeviceInterfaceAlias(
    DeviceInfoSet: HDEVINFO,
    DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
    AliasInterfaceClassGuid: *const GUID,
    AliasDeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
) -> BOOL {
    SetLastError(ERROR_INVALID_PARAMETER);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupVerifyInfFileA(mut InfName: PCSTR, mut AltPlatformInfo: PSP_ALTPLATFORM_INFO, mut InfFileName: PSP_INF_SIGNER_INFO_A) -> BOOL {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/stubs.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupVerifyInfFileA\0" as *const u8 as *const ::core::ffi::c_char,
            133 as ::core::ffi::c_int,
            b"Stub %s %p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_a(InfName as *const ::core::ffi::c_char),
            AltPlatformInfo,
            InfFileName,
        );
    }
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupVerifyInfFileW(
    InfName: PCWSTR,
    AltPlatformInfo: PSP_ALTPLATFORM_INFO,
    InfFileName: PSP_INF_SIGNER_INFO_W,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetDriverInstallParamsA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverInfoData: PSP_DRVINFO_DATA_A,
    DriverInstallParams: PSP_DRVINSTALL_PARAMS,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetDriverInstallParamsW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverInfoData: PSP_DRVINFO_DATA_W,
    DriverInstallParams: PSP_DRVINSTALL_PARAMS,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}
