//! ntdll.dll user-mode ETW provider surface (the `EtwEvent*` family that
//! advapi32's `Event*` forwarders resolve to).
//!
//! No trace session exists in this OS, so every provider registers
//! successfully and every event is dropped, which is exactly what Windows
//! does when nothing is listening: `EtwEventRegister` hands back a handle,
//! `EtwEventEnabled` reports FALSE, and the write calls succeed without
//! delivering anything. The provider callback is never invoked, since no
//! session can enable it.

use core::ffi::c_void;
use core::sync::atomic::{AtomicU64, Ordering};

type ULONG = u32;
type BOOLEAN = u8;
type PVOID = *mut c_void;
type PCWSTR = *const u16;
type GUID = c_void; // only ever passed by pointer and never read

const ERROR_SUCCESS: ULONG = 0;
const ERROR_INVALID_PARAMETER: ULONG = 87;

/// Source of unique, nonzero REGHANDLE values. Starts at 1 so 0 stays free
/// as the "no registration" value.
static NEXT_REG_HANDLE: AtomicU64 = AtomicU64::new(1);

/// C: `ULONG EtwEventRegister(LPCGUID ProviderId, PENABLECALLBACK EnableCallback,
/// PVOID CallbackContext, PREGHANDLE RegHandle)`.
#[no_mangle]
pub unsafe extern "system" fn EtwEventRegister(
    ProviderId: *const GUID,
    EnableCallback: PVOID,
    CallbackContext: PVOID,
    RegHandle: *mut u64,
) -> ULONG {
    let _ = (EnableCallback, CallbackContext);
    if ProviderId.is_null() || RegHandle.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    let handle = NEXT_REG_HANDLE.fetch_add(1, Ordering::Relaxed);
    core::ptr::write_volatile(RegHandle, handle);
    ERROR_SUCCESS
}

/// C: `ULONG EtwEventUnregister(REGHANDLE RegHandle)`. Unknown handles are
/// not an error on Windows, so this always succeeds.
#[no_mangle]
pub unsafe extern "system" fn EtwEventUnregister(RegHandle: u64) -> ULONG {
    let _ = RegHandle;
    ERROR_SUCCESS
}

/// C: `BOOLEAN EtwEventEnabled(REGHANDLE RegHandle, PCEVENT_DESCRIPTOR EventDescriptor)`.
/// No session ever enables a provider, so this is always FALSE.
#[no_mangle]
pub unsafe extern "system" fn EtwEventEnabled(RegHandle: u64, EventDescriptor: PVOID) -> BOOLEAN {
    let _ = (RegHandle, EventDescriptor);
    0
}

/// C: `ULONG EtwEventWrite(REGHANDLE RegHandle, PCEVENT_DESCRIPTOR EventDescriptor,
/// ULONG UserDataCount, PEVENT_DATA_DESCRIPTOR UserData)`. Event is dropped.
#[no_mangle]
pub unsafe extern "system" fn EtwEventWrite(
    RegHandle: u64,
    EventDescriptor: PVOID,
    UserDataCount: ULONG,
    UserData: PVOID,
) -> ULONG {
    let _ = (RegHandle, EventDescriptor, UserDataCount, UserData);
    ERROR_SUCCESS
}

/// C: `ULONG EtwEventWriteTransfer(REGHANDLE RegHandle, PCEVENT_DESCRIPTOR Descriptor,
/// LPCGUID ActivityId, LPCGUID RelatedActivityId, ULONG UserDataCount,
/// PEVENT_DATA_DESCRIPTOR UserData)`. Event is dropped.
#[no_mangle]
pub unsafe extern "system" fn EtwEventWriteTransfer(
    RegHandle: u64,
    Descriptor: PVOID,
    ActivityId: *const GUID,
    RelatedActivityId: *const GUID,
    UserDataCount: ULONG,
    UserData: PVOID,
) -> ULONG {
    let _ = (
        RegHandle,
        Descriptor,
        ActivityId,
        RelatedActivityId,
        UserDataCount,
        UserData,
    );
    ERROR_SUCCESS
}

/// C: `ULONG EtwEventWriteString(REGHANDLE RegHandle, UCHAR Level, ULONGLONG Keyword,
/// PCWSTR String)`. Event is dropped.
#[no_mangle]
pub unsafe extern "system" fn EtwEventWriteString(
    RegHandle: u64,
    Level: u8,
    Keyword: u64,
    String: PCWSTR,
) -> ULONG {
    let _ = (RegHandle, Level, Keyword, String);
    ERROR_SUCCESS
}

/// C: `ULONG EtwEventWriteEx(REGHANDLE RegHandle, PCEVENT_DESCRIPTOR Descriptor,
/// ULONG64 Filter, ULONG Flags, LPCGUID ActivityId, LPCGUID RelatedActivityId,
/// ULONG UserDataCount, PEVENT_DATA_DESCRIPTOR UserData)`. Event is dropped.
#[no_mangle]
pub unsafe extern "system" fn EtwEventWriteEx(
    RegHandle: u64,
    Descriptor: PVOID,
    Filter: u64,
    Flags: ULONG,
    ActivityId: *const GUID,
    RelatedActivityId: *const GUID,
    UserDataCount: ULONG,
    UserData: PVOID,
) -> ULONG {
    let _ = (
        RegHandle,
        Descriptor,
        Filter,
        Flags,
        ActivityId,
        RelatedActivityId,
        UserDataCount,
        UserData,
    );
    ERROR_SUCCESS
}

/// C: `ULONG EtwEventSetInformation(REGHANDLE RegHandle, EVENT_INFO_CLASS InformationClass,
/// PVOID EventInformation, ULONG InformationLength)`. Accepted and ignored.
#[no_mangle]
pub unsafe extern "system" fn EtwEventSetInformation(
    RegHandle: u64,
    InformationClass: u32,
    Information: PVOID,
    Length: ULONG,
) -> ULONG {
    let _ = (RegHandle, InformationClass, Information, Length);
    ERROR_SUCCESS
}
