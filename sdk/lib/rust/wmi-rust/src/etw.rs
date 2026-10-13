//! Kernel ETW provider API (Etw*) with no trace sessions behind it.
//!
//! Not in the 2003 tree: written from the WDK declarations in sdk/include/xdk/wmifuncs.h
//! and the WDK documentation, because ETW registration is in nearly every modern driver
//! (EtwRegister: 146 of 478 vendor driver packages; the write calls, 50-124). This kernel
//! has no ETW consumers, so every provider is correctly NEVER enabled: registration
//! succeeds with a real handle, the enable callback is never called (that only happens
//! when a session enables the provider), "is it enabled" answers FALSE, and a write
//! succeeds having written to no one -- which is what Windows does for a provider
//! nobody is listening to. A real ETW is the day a consumer needs events.

use core::sync::atomic::{AtomicU32, Ordering};
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;

/// Registration handles. Never zero: zero is the "no handle" a failed EtwRegister leaves.
static NEXT_REGHANDLE: AtomicU32 = AtomicU32::new(1);
/// Activity ids handed out by EVENT_ACTIVITY_CTRL_CREATE_ID.
static NEXT_ACTIVITY: AtomicU32 = AtomicU32::new(1);

// EVENT_ACTIVITY_CTRL_* (sdk/include/psdk/evntprov.h).
const EVENT_ACTIVITY_CTRL_GET_ID: ULONG = 1;
const EVENT_ACTIVITY_CTRL_SET_ID: ULONG = 2;
const EVENT_ACTIVITY_CTRL_CREATE_ID: ULONG = 3;
const EVENT_ACTIVITY_CTRL_GET_SET_ID: ULONG = 4;
const EVENT_ACTIVITY_CTRL_CREATE_SET_ID: ULONG = 5;

#[no_mangle]
pub unsafe extern "system" fn EtwRegister(
    provider_id: PVOID,
    _enable_callback: PVOID,
    _callback_context: PVOID,
    reg_handle: *mut u64,
) -> NTSTATUS {
    if provider_id.is_null() || reg_handle.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    *reg_handle = NEXT_REGHANDLE.fetch_add(1, Ordering::Relaxed) as u64;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn EtwUnregister(_reg_handle: u64) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn EtwEventEnabled(_reg_handle: u64, _event_descriptor: PVOID) -> BOOLEAN {
    0
}

#[no_mangle]
pub unsafe extern "system" fn EtwProviderEnabled(_reg_handle: u64, _level: UCHAR, _keyword: u64) -> BOOLEAN {
    0
}

#[no_mangle]
pub unsafe extern "system" fn EtwWrite(
    _reg_handle: u64,
    _event_descriptor: PVOID,
    _activity_id: PVOID,
    _user_data_count: ULONG,
    _user_data: PVOID,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn EtwWriteTransfer(
    _reg_handle: u64,
    _event_descriptor: PVOID,
    _activity_id: PVOID,
    _related_activity_id: PVOID,
    _user_data_count: ULONG,
    _user_data: PVOID,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn EtwWriteEx(
    _reg_handle: u64,
    _event_descriptor: PVOID,
    _filter: u64,
    _flags: ULONG,
    _activity_id: PVOID,
    _related_activity_id: PVOID,
    _user_data_count: ULONG,
    _user_data: PVOID,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn EtwWriteString(
    _reg_handle: u64,
    _level: UCHAR,
    _keyword: u64,
    _activity_id: PVOID,
    _string: PVOID,
) -> NTSTATUS {
    STATUS_SUCCESS
}

/// Provider traits, decode GUIDs and the like: stored by Windows for consumers to read.
#[no_mangle]
pub unsafe extern "system" fn EtwSetInformation(
    _reg_handle: u64,
    _information_class: ULONG,
    _event_information: PVOID,
    _information_length: ULONG,
) -> NTSTATUS {
    STATUS_SUCCESS
}

/// The calling thread's activity id. With no consumer nothing reads it, so there is no
/// per-thread slot: GET answers the null GUID, SET is accepted, and CREATE hands out a
/// distinct non-null id, because callers use it to correlate their own events.
#[no_mangle]
pub unsafe extern "system" fn EtwActivityIdControl(control_code: ULONG, activity_id: *mut [u8; 16]) -> NTSTATUS {
    if activity_id.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    match control_code {
        EVENT_ACTIVITY_CTRL_GET_ID | EVENT_ACTIVITY_CTRL_GET_SET_ID => *activity_id = [0; 16],
        EVENT_ACTIVITY_CTRL_SET_ID => {}
        EVENT_ACTIVITY_CTRL_CREATE_ID | EVENT_ACTIVITY_CTRL_CREATE_SET_ID => {
            let mut id = [0u8; 16];
            id[..4].copy_from_slice(&NEXT_ACTIVITY.fetch_add(1, Ordering::Relaxed).to_le_bytes());
            *activity_id = id;
        }
        _ => return STATUS_INVALID_PARAMETER,
    }
    STATUS_SUCCESS
}
