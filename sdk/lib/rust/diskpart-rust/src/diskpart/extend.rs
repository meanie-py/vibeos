use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn extend_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    EXIT_SUCCESS
}
