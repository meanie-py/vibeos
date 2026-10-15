use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn main() -> i32 {
    /* 5x Sleep(0) = distinctive NtDelayExecution pattern in SVC trace */
    Sleep(0);
    Sleep(0);
    Sleep(0);
    Sleep(0);
    Sleep(0);

    /* Stay alive briefly so winlogon can observe us */
    Sleep(3000);

    ExitProcess(42);
    return 0;
}
