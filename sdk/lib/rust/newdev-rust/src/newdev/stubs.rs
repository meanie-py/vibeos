use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn InstallNewDevice(
    hwndParent: HWND,
    ClassGuid: LPGUID,
    Reboot: PDWORD,
) -> BOOL {
    UNIMPLEMENTED!();
    SetLastError(ERROR_GEN_FAILURE);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn InstallSelectedDriverW(
    hwndParent: HWND,
    DeviceInfoSet: HDEVINFO,
    Reserved: LPCWSTR,
    Backup: BOOL,
    pReboot: PDWORD,
) -> BOOL {
    UNIMPLEMENTED!();
    SetLastError(ERROR_GEN_FAILURE as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DiShowUpdateDevice(
    hwndParent: HWND,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Flags: DWORD,
    NeedReboot: PBOOL,
) -> BOOL {
    if Flags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return (FALSE) as BOOL;
    }

    UNIMPLEMENTED!();
    SetLastError(ERROR_GEN_FAILURE);
    return (FALSE) as BOOL;
}
