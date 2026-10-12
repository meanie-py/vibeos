use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn wmain(argc: core::ffi::c_int, argv: *mut *const WCHAR) -> core::ffi::c_int {
    WINE_FIXME!(b"stub\n\0".as_ptr() as *const i8);
    return 0;
}
