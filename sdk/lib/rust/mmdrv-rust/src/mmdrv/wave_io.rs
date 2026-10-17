use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn CompleteWaveOverlap(
    error_code: DWORD,
    bytes_transferred: DWORD,
    overlapped: LPOVERLAPPED,
) {
    // DPRINT("Complete partial wave overlap\n");
}

#[no_mangle]
pub unsafe extern "C" fn UpdateWaveLoop(session_info: *mut SessionInfo) {}

#[no_mangle]
pub unsafe extern "C" fn PerformWaveIO(session_info: *mut SessionInfo) {}
