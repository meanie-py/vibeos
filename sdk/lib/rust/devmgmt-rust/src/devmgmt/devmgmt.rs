use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hThisInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPWSTR,
    nCmdShow: i32,
) -> i32 {
    if DeviceManager_ExecuteW(
        core::ptr::null_mut(),
        hThisInstance,
        core::ptr::null(),
        nCmdShow,
    ) == 0
    {
        return GetLastError() as i32;
    }

    return 0;
}
