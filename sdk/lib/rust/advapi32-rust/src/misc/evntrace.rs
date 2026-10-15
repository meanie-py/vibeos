use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(advapi) - debug channel macro, no Rust equivalent needed

#[no_mangle]
pub unsafe extern "system" fn OpenTraceA(Logfile: PEVENT_TRACE_LOGFILEA) -> TRACEHANDLE {
    SetLastError(ERROR_ACCESS_DENIED);
    INVALID_PROCESSTRACE_HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn OpenTraceW(Logfile: PEVENT_TRACE_LOGFILEW) -> TRACEHANDLE {
    SetLastError(ERROR_ACCESS_DENIED);
    INVALID_PROCESSTRACE_HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn CloseTrace(TraceHandle: TRACEHANDLE) -> ULONG {
    // OpenTraceA/W always fail here, so no trace handle can ever be valid.
    ERROR_INVALID_HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn ProcessTrace(
    HandleArray: PTRACEHANDLE,
    HandleCount: ULONG,
    StartTime: LPFILETIME,
    EndTime: LPFILETIME,
) -> ULONG {
    ERROR_NOACCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS system libraries
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     advapi32.dll Event tracing stubs
//  * COPYRIGHT:   Copyright 2017 Mark Jansen (mark.jansen@reactos.org)
//  */
//
// #include <advapi32.h>
// #include <wmistr.h>
// #include <evntrace.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(advapi);
//
//
// TRACEHANDLE
// WINAPI
// OpenTraceA(IN PEVENT_TRACE_LOGFILEA Logfile)
// {
//     UNIMPLEMENTED;
//     SetLastError(ERROR_ACCESS_DENIED);
//     return INVALID_PROCESSTRACE_HANDLE;
// }
//
// TRACEHANDLE
// WINAPI
// OpenTraceW(IN PEVENT_TRACE_LOGFILEW Logfile)
// {
//     UNIMPLEMENTED;
//     SetLastError(ERROR_ACCESS_DENIED);
//     return INVALID_PROCESSTRACE_HANDLE;
// }
//
// ULONG
// WINAPI
// ProcessTrace(IN PTRACEHANDLE HandleArray,
//              IN ULONG HandleCount,
//              IN LPFILETIME StartTime,
//              IN LPFILETIME EndTime)
// {
//     UNIMPLEMENTED;
//     return ERROR_NOACCESS;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// ============================================================================
// EventActivityIdControl (ETW activity ids, evntprov.h EVENT_ACTIVITY_CTRL_*).
// The activity id is per thread. It lives in one TLS slot that points at a heap-allocated
// GUID, created on the first set. A thread that never set one reads the all-zero GUID,
// which is what Windows returns. ntdll's EtwEventActivityIdControl is only a 0x600+ stub
// (and ntdll is not ours to change here), so the control is implemented in this module.
// ============================================================================
use core::sync::atomic::{AtomicU32, Ordering};

const EVENT_ACTIVITY_CTRL_GET_ID: ULONG = 1;
const EVENT_ACTIVITY_CTRL_SET_ID: ULONG = 2;
const EVENT_ACTIVITY_CTRL_CREATE_ID: ULONG = 3;
const EVENT_ACTIVITY_CTRL_GET_SET_ID: ULONG = 4;
const EVENT_ACTIVITY_CTRL_CREATE_SET_ID: ULONG = 5;

const TLS_NO_INDEX: DWORD = 0xFFFF_FFFF;
static ACTIVITY_TLS_INDEX: AtomicU32 = AtomicU32::new(TLS_NO_INDEX);

extern "system" {
    fn TlsAlloc() -> DWORD;
    fn TlsFree(dwTlsIndex: DWORD) -> BOOL;
    fn TlsGetValue(dwTlsIndex: DWORD) -> LPVOID;
    fn TlsSetValue(dwTlsIndex: DWORD, lpTlsValue: LPVOID) -> BOOL;
}

unsafe fn activity_slot() -> Option<DWORD> {
    let cur = ACTIVITY_TLS_INDEX.load(Ordering::Acquire);
    if cur != TLS_NO_INDEX {
        return Some(cur);
    }
    let idx = TlsAlloc();
    if idx == TLS_NO_INDEX {
        return None;
    }
    match ACTIVITY_TLS_INDEX.compare_exchange(TLS_NO_INDEX, idx, Ordering::AcqRel, Ordering::Acquire) {
        Ok(_) => Some(idx),
        Err(existing) => {
            // Another thread published its index first; give ours back.
            TlsFree(idx);
            Some(existing)
        }
    }
}

unsafe fn current_activity() -> GUID {
    let mut out: GUID = core::mem::zeroed();
    if let Some(idx) = activity_slot() {
        let slot = TlsGetValue(idx) as *const GUID;
        if !slot.is_null() {
            out = *slot;
        }
    }
    out
}

unsafe fn set_current_activity(id: GUID) -> bool {
    let idx = match activity_slot() {
        Some(idx) => idx,
        None => return false,
    };
    let mut slot = TlsGetValue(idx) as *mut GUID;
    if slot.is_null() {
        slot = heap_alloc(core::mem::size_of::<GUID>()) as *mut GUID;
        if slot.is_null() {
            return false;
        }
        if TlsSetValue(idx, slot as LPVOID) == 0 {
            heap_free(slot as PVOID);
            return false;
        }
    }
    *slot = id;
    true
}

// A random version-4 GUID, as Windows generates activity ids. RtlGenRandom is this module's
// own SystemFunction036 (sysfunc.rs).
unsafe fn create_activity_id() -> Option<GUID> {
    let mut id: GUID = core::mem::zeroed();
    if crate::misc_sysfunc::SystemFunction036(&mut id as *mut GUID as PVOID, 16) == 0 {
        return None;
    }
    id.Data3 = (id.Data3 & 0x0FFF) | 0x4000;
    id.Data4[0] = (id.Data4[0] & 0x3F) | 0x80;
    Some(id)
}

#[no_mangle]
pub unsafe extern "system" fn EventActivityIdControl(ControlCode: ULONG, ActivityId: *mut GUID) -> ULONG {
    if ActivityId.is_null() {
        return ERROR_INVALID_PARAMETER as ULONG;
    }
    match ControlCode {
        EVENT_ACTIVITY_CTRL_GET_ID => {
            *ActivityId = current_activity();
        }
        EVENT_ACTIVITY_CTRL_SET_ID => {
            if !set_current_activity(*ActivityId) {
                return ERROR_NOT_ENOUGH_MEMORY as ULONG;
            }
        }
        EVENT_ACTIVITY_CTRL_CREATE_ID => {
            match create_activity_id() {
                Some(id) => *ActivityId = id,
                None => return ERROR_GEN_FAILURE as ULONG,
            }
        }
        EVENT_ACTIVITY_CTRL_GET_SET_ID => {
            // Swap: the new id comes in through ActivityId, the old one goes back out.
            let new = *ActivityId;
            let old = current_activity();
            if !set_current_activity(new) {
                return ERROR_NOT_ENOUGH_MEMORY as ULONG;
            }
            *ActivityId = old;
        }
        EVENT_ACTIVITY_CTRL_CREATE_SET_ID => {
            let id = match create_activity_id() {
                Some(id) => id,
                None => return ERROR_GEN_FAILURE as ULONG,
            };
            if !set_current_activity(id) {
                return ERROR_NOT_ENOUGH_MEMORY as ULONG;
            }
            *ActivityId = id;
        }
        _ => return ERROR_INVALID_PARAMETER as ULONG,
    }
    ERROR_SUCCESS as ULONG
}
