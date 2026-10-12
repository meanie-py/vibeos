use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInst: HINSTANCE,
    hPrevInst: HINSTANCE,
    lpCmdLine: LPWSTR,
    nCmdShow: INT,
) -> INT {
    DPRINT1!("dllhost: %S\n", lpCmdLine);
    0
}
