use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn NormalizeString(
    NormForm: NORM_FORM,
    lpSrcString: LPCWSTR,
    cwSrcLength: INT,
    lpDstString: LPWSTR,
    cwDstLength: INT,
) -> INT {
    DPRINT1!("%x %p %d %p %d\n", NormForm, lpSrcString, cwSrcLength, lpDstString, cwDstLength);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn IsNormalizedString(NormForm: NORM_FORM, lpString: LPCWSTR, cwLength: INT) -> BOOL {
    DPRINT1!("%x %p %d\n", NormForm, lpString, cwLength);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}
