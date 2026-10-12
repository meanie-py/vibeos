use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn RegisterTicketExpiredNotificationEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("RegisterTicketExpiredNotificationEvent\n");
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
pub unsafe extern "system" fn UnregisterTicketExpiredNotificationEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("UnregisterTicketExpiredNotificationEvent\n");
    // TRACE("Size: %lu\n", pInfo->Size);
    // TRACE("Flags: %lx\n", pInfo->Flags);
    // TRACE("UserName: %S\n", pInfo->UserName);
    // TRACE("Domain: %S\n", pInfo->Domain);
    // TRACE("WindowStation: %S\n", pInfo->WindowStation);
    // TRACE("hToken: %p\n", pInfo->hToken);
    // TRACE("hDesktop: %p\n", pInfo->hDesktop);
    // TRACE("pStatusCallback: %p\n", pInfo->pStatusCallback);
}
