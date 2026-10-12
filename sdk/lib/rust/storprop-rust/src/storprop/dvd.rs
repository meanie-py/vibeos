use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn CdromDisableDigitalPlayback(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> LONG {
    DPRINT1!("CdromDisableDigitalPlayback(%p %p)\n",
           DeviceInfoSet, DeviceInfoData);
    (ERROR_SUCCESS as i32)
}

#[no_mangle]
pub unsafe extern "system" fn CdromEnableDigitalPlayback(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    ForceUnknown: BOOLEAN,
) -> LONG {
    DPRINT1!("CdromEnableDigitalPlayback(%p %p %u)\n",
           DeviceInfoSet, DeviceInfoData, ForceUnknown as core::ffi::c_uint);
    (ERROR_SUCCESS as i32)
}

#[no_mangle]
pub unsafe extern "system" fn CdromIsDigitalPlaybackEnabled(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    Enabled: PBOOLEAN,
) -> LONG {
    DPRINT1!("CdromIsDigitalPlaybackEnabled(%p %p %p)\n",
           DeviceInfoSet, DeviceInfoData, Enabled);
    (ERROR_SUCCESS as i32)
}

#[no_mangle]
pub unsafe extern "system" fn CdromKnownGoodDigitalPlayback(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    DPRINT1!("CdromKnownGoodDigitalPlayback(%p %p)\n",
           DeviceInfoSet, DeviceInfoData);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn DvdClassInstaller(
    InstallFunction: DI_FUNCTION,
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> DWORD {
    DPRINT1!("DvdClassInstaller(%u %p %p)\n",
           InstallFunction, DeviceInfoSet, DeviceInfoData);
    ERROR_DI_DO_DEFAULT
}

#[no_mangle]
pub unsafe extern "system" fn DvdPropPageProvider(
    lpPropSheetPageRequest: PSP_PROPSHEETPAGE_REQUEST,
    lpfnAddPropSheetPageProc: LPFNADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> BOOL {
    DPRINT1!("DvdPropPageProvider(%p %p %lx)\n",
           lpPropSheetPageRequest, lpfnAddPropSheetPageProc, lParam);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn DvdLauncher(HWnd: HWND, DriveLetter: CHAR) -> BOOL {
    DPRINT1!("DvdLauncher(%p %c)\n", HWnd, DriveLetter as core::ffi::c_int);
    (FALSE as i32)
}
