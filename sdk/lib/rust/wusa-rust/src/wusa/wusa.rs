use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hCurInst: HINSTANCE,
    hPrevInst: HINSTANCE,
    lpsCmdLine: LPWSTR,
    nCmdShow: i32,
) -> i32 {
    0
}
