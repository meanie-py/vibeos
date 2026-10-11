use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn wmain(
    argc: ::core::ffi::c_int,
    argv: *mut *mut WCHAR,
) -> ::core::ffi::c_int {
    return LoadPerfCounterTextStringsW(GetCommandLineW() as LPCWSTR, (FALSE) as BOOL) as ::core::ffi::c_int;
}
