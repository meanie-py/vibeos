use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn TouchFileTimes(FileHandle: HANDLE, mut lpSystemTime: LPSYSTEMTIME) -> BOOL {
    let mut FileTime: FILETIME = core::mem::zeroed();
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();

    if lpSystemTime.is_null() {
        GetSystemTime(core::ptr::addr_of_mut!(SystemTime));
        lpSystemTime = core::ptr::addr_of_mut!(SystemTime);
    }

    return (SystemTimeToFileTime(lpSystemTime, core::ptr::addr_of_mut!(FileTime)) != 0
        && SetFileTime(
            FileHandle,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FileTime),
        ) != 0) as BOOL;
}
