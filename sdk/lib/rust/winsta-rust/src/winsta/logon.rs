use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn WinStationDisconnect(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationFreeUserCredentials(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationFreeUserCertificates(A: PVOID) {
    // UNIMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "system" fn WinStationIsSessionPermitted() -> BOOL {
    UNIMPLEMENTED!();
    SetLastError(ERROR_SUCCESS);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationNegotiateSession(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
    F: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationRedirectErrorMessage(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationReportUIResult(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationGetLoggedOnCount(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationUserLoginAccessCheck(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationWaitForConnect() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationReset(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationNotifyLogoff() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationNotifyLogon(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
    F: PVOID,
    G: PVOID,
    H: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationCanLogonProceed(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationOpenSessionDirectory(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}
