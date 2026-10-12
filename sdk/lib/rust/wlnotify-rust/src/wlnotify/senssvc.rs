use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn SensDisconnectEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensDisconnectEvent\n");
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
pub unsafe extern "system" fn SensLockEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensLockEvent\n");
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
pub unsafe extern "system" fn SensLogoffEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensLogoffEvent\n");
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
pub unsafe extern "system" fn SensLogonEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensLogonEvent\n");
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
pub unsafe extern "system" fn SensPostShellEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensPostShellEvent\n");
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
pub unsafe extern "system" fn SensReconnectEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensReconnectEvent\n");
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
pub unsafe extern "system" fn SensShutdownEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensShutdownEvent\n");
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
pub unsafe extern "system" fn SensStartScreenSaverEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensStartScreenSaverEvent\n");
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
pub unsafe extern "system" fn SensStartShellEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensStartShellEvent\n");
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
pub unsafe extern "system" fn SensStartupEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensStartupEvent\n");
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
pub unsafe extern "system" fn SensStopScreenSaverEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensStopScreenSaverEvent\n");
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
pub unsafe extern "system" fn SensUnlockEvent(pInfo: PWLX_NOTIFICATION_INFO) {
    // TRACE("SensUnlockEvent\n");
    // TRACE("Size: %lu\n", pInfo->Size);
    // TRACE("Flags: %lx\n", pInfo->Flags);
    // TRACE("UserName: %S\n", pInfo->UserName);
    // TRACE("Domain: %S\n", pInfo->Domain);
    // TRACE("WindowStation: %S\n", pInfo->WindowStation);
    // TRACE("hToken: %p\n", pInfo->hToken);
    // TRACE("hDesktop: %p\n", pInfo->hDesktop);
    // TRACE("pStatusCallback: %p\n", pInfo->pStatusCallback);
}
