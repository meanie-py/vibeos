//! Feature configuration (Rtl*Feature*) with nothing configured.
//!
//! Windows' feature staging: a driver built with the WDK's feature-staging static library
//! (115-118 of 478 vendor driver packages) asks the kernel whether each of its staged
//! features is overridden, and reports usage for telemetry. This kernel stages nothing, so
//! the honest answers are: no feature has a configuration (STATUS_NOT_FOUND, and the
//! library then uses the feature's compiled-in default), the change stamp never moves, a
//! change notification never fires, and usage reports go nowhere.
//!
//! Signatures for the query and notification calls follow phnt's ntrtl.h. The four
//! usage-provider calls (RtlRegisterFeatureUsageProvider, RtlUnregisterFeatureUsageProvider,
//! RtlRecordFeatureUsage, RtlArmFeatureUsageProviderFlushNotification) have no public
//! prototype; they take no arguments here, are exported on 64-bit only (the x64 calling
//! convention lets a callee ignore what it was passed, i386 stdcall does not), and
//! registration reports STATUS_NOT_SUPPORTED, so a caller never holds a handle we did
//! not hand out.

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_NOT_FOUND, STATUS_NOT_SUPPORTED};

/// RtlRegisterFeatureConfigurationChangeNotification's handle. Never dereferenced; a
/// distinct non-null value so a caller that checks for NULL sees a registration.
static CHANGE_REGISTRATION: u8 = 0;

#[no_mangle]
pub unsafe extern "system" fn RtlQueryFeatureConfiguration(
    _feature_id: ULONG,
    _configuration_type: ULONG,
    change_stamp: *mut u64,
    _feature_configuration: PVOID,
) -> NTSTATUS {
    if !change_stamp.is_null() {
        *change_stamp = 0;
    }
    STATUS_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryFeatureConfigurationChangeStamp() -> u64 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RtlRegisterFeatureConfigurationChangeNotification(
    callback: PVOID,
    _context: PVOID,
    _observed_change_stamp: *mut u64,
    registration_handle: *mut PVOID,
) -> NTSTATUS {
    if callback.is_null() || registration_handle.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    *registration_handle = &CHANGE_REGISTRATION as *const u8 as PVOID;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlUnregisterFeatureConfigurationChangeNotification(
    _registration_handle: PVOID,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlNotifyFeatureUsage(_feature_usage_report: PVOID) -> NTSTATUS {
    STATUS_SUCCESS
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "system" fn RtlRegisterFeatureUsageProvider() -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "system" fn RtlUnregisterFeatureUsageProvider() -> NTSTATUS {
    STATUS_SUCCESS
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "system" fn RtlRecordFeatureUsage() -> NTSTATUS {
    STATUS_SUCCESS
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern "system" fn RtlArmFeatureUsageProviderFlushNotification() -> NTSTATUS {
    STATUS_SUCCESS
}
