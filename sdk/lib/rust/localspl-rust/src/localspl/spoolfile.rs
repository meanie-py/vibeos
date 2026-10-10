use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn LocalGetSpoolFileInfo(hPrinter: HANDLE, pSpoolDir: *mut LPWSTR, phFile: LPHANDLE, hSpoolerProcess: HANDLE, hAppProcess: HANDLE) -> BOOL {
    // FIXME("LocalGetSpoolFileInfo(%p, %S, %p, %p, %p)\n", hPrinter, pSpoolDir, phFile, hSpoolerProcess, hAppProcess);
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn LocalCommitSpoolData(hPrinter: HANDLE, cbCommit: DWORD) -> BOOL {
    // FIXME("LocalCommitSpoolData(%p, %lu)\n", hPrinter, cbCommit);
    FALSE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn LocalCloseSpoolFileHandle(hPrinter: HANDLE) -> BOOL {
    // FIXME("LocalCloseSpoolFileHandle(%p)\n", hPrinter);
    FALSE as BOOL
}
