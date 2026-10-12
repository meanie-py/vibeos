// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_2F006FE06135: [u16; 9] = [0x0053u16, 0x0063, 0x0068, 0x0065, 0x0064, 0x0075, 0x006C, 0x0065, 0x0000];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn SchedEventLogoff(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SchedEventLogoff\n");
    // TRACE("Size: %lu\n", pInfo->Size);
    // TRACE("Flags: %lx\n", pInfo->Flags);
    // TRACE("UserName: %S\n", pInfo->UserName);
    // TRACE("Domain: %S\n", pInfo->Domain);
    // TRACE("WindowStation: %S\n", pInfo->WindowStation);
    // TRACE("hToken: %p\n", pInfo->hToken);
    // TRACE("hDesktop: %p\n", pInfo->hDesktop);
    // TRACE("pStatusCallback: %p\n", pInfo->pStatusCallback);
}

#[no_mangle]
pub unsafe extern "system" fn SchedStartShell(pInfo: PWLX_NOTIFICATION_INFO) {
    let mut hManager: SC_HANDLE = core::ptr::null_mut();
    let mut hService: SC_HANDLE = core::ptr::null_mut();
    let mut ServiceStatus: SERVICE_STATUS = core::mem::zeroed();

    'done: {
        hManager = OpenSCManagerW(core::ptr::null_mut(), core::ptr::null_mut(), SC_MANAGER_CONNECT);
        if hManager.is_null() {
            WARN!("OpenSCManagerW() failed (Error %lu)\n", GetLastError());
            break 'done;
        }

        hService = OpenServiceW(hManager, RUSTCONV_WSTR_2F006FE06135.as_ptr(), SERVICE_USER_DEFINED_CONTROL);
        if hManager.is_null() {
            WARN!("OpenServiceW() failed (Error %lu)\n", GetLastError());
            break 'done;
        }

        if ControlService(hService, 128, core::ptr::addr_of_mut!(ServiceStatus)) == 0 {
            WARN!("ControlService() failed (Error %lu)\n", GetLastError());
        }
    }

    if !hService.is_null() {
        CloseServiceHandle(hService);
    }

    if !hManager.is_null() {
        CloseServiceHandle(hManager);
    }
}
