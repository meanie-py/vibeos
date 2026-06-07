use crate::types::*;
use core::ffi::c_void;

extern "system" {
    fn RosSymFreeMem(RosSymInfo: *mut ROSSYM_INFO);
}
#[no_mangle]
pub unsafe extern "C" fn RosSymDelete(RosSymInfo: *mut ROSSYM_INFO) {
    RosSymFreeMem(RosSymInfo);
}
