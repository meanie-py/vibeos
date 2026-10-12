use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn WinStationQueryLogonCredentialsW(A: PVOID) {
    // UNIMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "system" fn WinStationQueryInformationA(
    hServer: HANDLE,
    LogonId: ULONG,
    WinStationInformationClass: WINSTATIONINFOCLASS,
    pWinStationInformation: PVOID,
    WinStationInformationLength: ULONG,
    pReturnLength: PULONG,
) -> BOOLEAN {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    UNIMPLEMENTED!();
    return (FALSE) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "system" fn WinStationQueryInformationW(
    hServer: HANDLE,
    LogonId: ULONG,
    WinStationInformationClass: WINSTATIONINFOCLASS,
    pWinStationInformation: PVOID,
    WinStationInformationLength: ULONG,
    pReturnLength: PULONG,
) -> BOOLEAN {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    UNIMPLEMENTED!();
    return (FALSE) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationQueryAllowConcurrentConnections() {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationQueryEnforcementCore(
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
pub unsafe extern "system" fn WinStationQueryLicense(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationQueryUpdateRequired(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationEnumerateLicenses(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationEnumerateProcesses(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationEnumerateA(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationEnumerateW(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationEnumerate_IndexedA(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationEnumerate_IndexedW(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationRequestSessionsList(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationGetAllProcesses(A: PVOID, B: PVOID, C: PVOID, D: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationGetLanAdapterNameA(
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
pub unsafe extern "system" fn WinStationGetLanAdapterNameW(
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
pub unsafe extern "system" fn WinStationGetConnectionProperty(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationGetInitialApplication(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
    E: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationGetProcessSid(
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
pub unsafe extern "system" fn WinStationGetUserCertificates(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationGetUserCredentials(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationGetUserProfile(A: PVOID, B: PVOID, C: PVOID, D: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn _WinStationGetApplicationInfo(A: PVOID, B: PVOID, C: PVOID, D: PVOID) {
    // UNIMPLEMENTED;
}
