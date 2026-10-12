use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn TSEventDisconnect(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("TSEventDisconnect\n");
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
pub unsafe extern "system" fn TSEventLogoff(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("TSEventLogoff\n");
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
pub unsafe extern "system" fn TSEventLogon(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("TSEventLogon\n");
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
pub unsafe extern "system" fn TSEventPostShell(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("TSEventPostShell\n");
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
pub unsafe extern "system" fn TSEventReconnect(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("TSEventReconnect\n");
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
pub unsafe extern "system" fn TSEventShutdown(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("TSEventShutdown\n");
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
pub unsafe extern "system" fn TSEventStartShell(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("TSEventStartShell\n");
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
pub unsafe extern "system" fn TSEventStartup(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("TSEventStartup\n");
    // TRACE("Size: %lu\n", pInfo->Size);
    // TRACE("Flags: %lx\n", pInfo->Flags);
    // TRACE("UserName: %S\n", pInfo->UserName);
    // TRACE("Domain: %S\n", pInfo->Domain);
    // TRACE("WindowStation: %S\n", pInfo->WindowStation);
    // TRACE("hToken: %p\n", pInfo->hToken);
    // TRACE("hDesktop: %p\n", pInfo->hDesktop);
    // TRACE("pStatusCallback: %p\n", pInfo->pStatusCallback);
}
