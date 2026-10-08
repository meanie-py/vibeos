use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn shrink_main(argc: INT, argv: *mut PWSTR) -> EXIT_CODE {
    EXIT_SUCCESS
}
