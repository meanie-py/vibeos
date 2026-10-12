use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingOpenA(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingOpenW(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingSetPolicy(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingClose(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingDeactivateCurrentPolicy(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingFreePolicyInformation(A: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingGetAvailablePolicyIds(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingGetPolicy(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingGetPolicyInformationA(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingGetPolicyInformationW(
    A: PVOID,
    B: PVOID,
    C: PVOID,
    D: PVOID,
) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingLoadPolicy(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn ServerLicensingUnloadPolicy(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationActivateLicense(A: PVOID, B: PVOID, C: PVOID, D: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationGenerateLicense(A: PVOID, B: PVOID, C: PVOID, D: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationRemoveLicense(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationInstallLicense(A: PVOID, B: PVOID, C: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WinStationGetMachinePolicy(A: PVOID, B: PVOID) {
    // UNIMPLEMENTED;
}
