use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn GRPFILE_ReadGroupFile(lpszPath: LPCWSTR, bIsCommonGroup: BOOL) -> DWORD {
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn GRPFILE_WriteGroupFile(hGroup: *mut PROGGROUP) -> BOOL {
    TRUE as BOOL
}
