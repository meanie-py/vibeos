//! Rust translation of `dll/ntdll/etw/trace.c`.
//!
//! ntdll.dll Event Tracing (ETW) user-mode API. Almost entirely
//! `@unimplemented` stubs that log a FIXME (NDEBUG-elided in release) and
//! return `ERROR_SUCCESS`.
//!
//! Type note: the ETW user types (`TRACEHANDLE`, `EVENT_TRACE_PROPERTIES`,
//! `WMIDPREQUEST`, `TRACE_GUID_REGISTRATION`, …) live in evntrace.h /
//! wmistr.h, which are NOT part of the (kernel-focused) ntdll-sys bindgen
//! surface — so the handful these stubs touch are declared locally.
//!
//! ABI note: decorations match `def/ntdll.spec`. There, `TRACEHANDLE` is
//! spelled `double` — an 8-byte by-value argument that counts 8 toward the
//! i386 stdcall `@N` (e.g. `EtwTraceEvent(double ptr)` → `_EtwTraceEvent@12`).
//! `EtwTraceMessage` is `@ varargs` (cdecl) — see its note below.

use core::ffi::c_void;
use core::ptr::write_volatile;

// --- Local ETW aliases (absent from ntdll-sys) ----------------------
type TRACEHANDLE = u64; // ULONG64
type PTRACEHANDLE = *mut u64;
type ULONG = u32;
type UCHAR = u8;
type USHORT = u16;
type LPCGUID = *const c_void; // const GUID* — unread by these stubs
type LPCSTR = *const i8;
type LPCWSTR = *const u16;
type PVOID = *mut c_void;
// PEVENT_TRACE_HEADER / PEVENT_TRACE_PROPERTIES / PTRACE_GUID_REGISTRATION /
// WMIDPREQUEST are opaque to these stubs and carried as raw pointers.

const ERROR_SUCCESS: ULONG = 0;
const ERROR_INVALID_PARAMETER: ULONG = 87;

// EVENT_TRACE_CONTROL_* (evntrace.h) — passed only to the EtwControlTrace
// stubs (which ignore them); kept exact for fidelity.
const EVENT_TRACE_CONTROL_QUERY: ULONG = 0;
const EVENT_TRACE_CONTROL_STOP: ULONG = 1;
const EVENT_TRACE_CONTROL_UPDATE: ULONG = 2;
const EVENT_TRACE_CONTROL_FLUSH: ULONG = 3;

// sizeof(EVENT_TRACE_HEADER) — fixed at 48 by the evntrace.h layout on all
// targets (max field alignment is 8 from LARGE_INTEGER/ULONG64; total is
// already a multiple of 8). The struct is opaque in bindgen, so we read the
// leading USHORT `Size` field (offset 0) directly.
const SIZEOF_EVENT_TRACE_HEADER: USHORT = 48;

/// C: `ULONG CDECL EtwTraceMessage(TRACEHANDLE, ULONG, LPCGUID, USHORT, ...)`.
/// @unimplemented.
///
/// The C is variadic; Rust stable cannot *define* a C-variadic function. This
/// is safe to drop for a stub under cdecl: the caller pushes the varargs and
/// cleans the stack, and this callee never reads past its fixed parameters.
/// cdecl on i386 → `_EtwTraceMessage` (matches the `@ varargs` spec entry).
#[no_mangle]
pub unsafe extern "C" fn EtwTraceMessage(
    _session_handle: TRACEHANDLE,
    _message_flags: ULONG,
    _message_guid: LPCGUID,
    _message_number: USHORT,
) -> ULONG {
    ERROR_SUCCESS
}

/// C: `TRACEHANDLE NTAPI EtwGetTraceLoggerHandle(PVOID Buffer)`. Stub: -1.
#[no_mangle]
pub unsafe extern "system" fn EtwGetTraceLoggerHandle(_buffer: PVOID) -> TRACEHANDLE {
    -1i64 as u64 // (TRACEHANDLE)-1
}

/// C: `ULONG NTAPI EtwTraceEvent(TRACEHANDLE, PEVENT_TRACE_HEADER)`. Stub.
#[no_mangle]
pub unsafe extern "system" fn EtwTraceEvent(
    session_handle: TRACEHANDLE,
    event_trace: PVOID, // PEVENT_TRACE_HEADER
) -> ULONG {
    if session_handle == 0 || event_trace.is_null() {
        // invalid parameters
        return ERROR_INVALID_PARAMETER;
    }

    // EventTrace->Size — the leading USHORT field (offset 0).
    if *(event_trace as *const USHORT) != SIZEOF_EVENT_TRACE_HEADER {
        // invalid parameter
        return ERROR_INVALID_PARAMETER;
    }

    ERROR_SUCCESS
}

/// C: `ULONG NTAPI EtwGetTraceEnableFlags(TRACEHANDLE)`. Stub: 0xFF.
#[no_mangle]
pub unsafe extern "system" fn EtwGetTraceEnableFlags(_trace_handle: TRACEHANDLE) -> ULONG {
    0xFF
}

/// C: `UCHAR NTAPI EtwGetTraceEnableLevel(TRACEHANDLE)`. Stub: 0xFF.
#[no_mangle]
pub unsafe extern "system" fn EtwGetTraceEnableLevel(_trace_handle: TRACEHANDLE) -> UCHAR {
    0xFF
}

/// C: `ULONG NTAPI EtwUnregisterTraceGuids(TRACEHANDLE)`. Stub.
#[no_mangle]
pub unsafe extern "system" fn EtwUnregisterTraceGuids(_registration_handle: TRACEHANDLE) -> ULONG {
    ERROR_SUCCESS
}

/// C: `ULONG NTAPI EtwRegisterTraceGuidsA(WMIDPREQUEST, PVOID, LPCGUID, ULONG,
/// PTRACE_GUID_REGISTRATION, LPCSTR, LPCSTR, PTRACEHANDLE)`. Stub.
#[no_mangle]
pub unsafe extern "system" fn EtwRegisterTraceGuidsA(
    _request_address: PVOID,
    _request_context: PVOID,
    _control_guid: LPCGUID,
    _guid_count: ULONG,
    _trace_guid_reg: PVOID,
    _mof_image_path: LPCSTR,
    _mof_resource_name: LPCSTR,
    _registration_handle: PTRACEHANDLE,
) -> ULONG {
    ERROR_SUCCESS
}

/// C: `ULONG NTAPI EtwRegisterTraceGuidsW(WMIDPREQUEST, PVOID, LPCGUID, ULONG,
/// PTRACE_GUID_REGISTRATION, LPCWSTR, LPCWSTR, PTRACEHANDLE)`. Stub.
#[no_mangle]
pub unsafe extern "system" fn EtwRegisterTraceGuidsW(
    _request_address: PVOID,
    _request_context: PVOID,
    _control_guid: LPCGUID,
    _guid_count: ULONG,
    _trace_guid_reg: PVOID,
    _mof_image_path: LPCWSTR,
    _mof_resource_name: LPCWSTR,
    _registration_handle: PTRACEHANDLE,
) -> ULONG {
    ERROR_SUCCESS
}

/// C: `ULONG WINAPI EtwStartTraceW(PTRACEHANDLE, LPCWSTR, PEVENT_TRACE_PROPERTIES)`.
#[no_mangle]
pub unsafe extern "system" fn EtwStartTraceW(
    p_session_handle: PTRACEHANDLE,
    _session_name: LPCWSTR,
    _properties: PVOID,
) -> ULONG {
    if !p_session_handle.is_null() {
        write_volatile(p_session_handle, 0xcafe4242);
    }
    ERROR_SUCCESS
}

/// C: `ULONG WINAPI EtwStartTraceA(PTRACEHANDLE, LPCSTR, PEVENT_TRACE_PROPERTIES)`.
#[no_mangle]
pub unsafe extern "system" fn EtwStartTraceA(
    p_session_handle: PTRACEHANDLE,
    _session_name: LPCSTR,
    _properties: PVOID,
) -> ULONG {
    if !p_session_handle.is_null() {
        write_volatile(p_session_handle, 0xcafe4242);
    }
    ERROR_SUCCESS
}

/// C: `ULONG WINAPI EtwControlTraceW(TRACEHANDLE, LPCWSTR, PEVENT_TRACE_PROPERTIES, ULONG)`.
#[no_mangle]
pub unsafe extern "system" fn EtwControlTraceW(
    _h_session: TRACEHANDLE,
    _session_name: LPCWSTR,
    _properties: PVOID,
    _control: ULONG,
) -> ULONG {
    ERROR_SUCCESS
}

/// C: `ULONG WINAPI EtwControlTraceA(TRACEHANDLE, LPCSTR, PEVENT_TRACE_PROPERTIES, ULONG)`.
#[no_mangle]
pub unsafe extern "system" fn EtwControlTraceA(
    _h_session: TRACEHANDLE,
    _session_name: LPCSTR,
    _properties: PVOID,
    _control: ULONG,
) -> ULONG {
    ERROR_SUCCESS
}

/// C: `ULONG WINAPI EtwEnableTrace(ULONG, ULONG, ULONG, LPCGUID, TRACEHANDLE)`.
#[no_mangle]
pub unsafe extern "system" fn EtwEnableTrace(
    _enable: ULONG,
    _flag: ULONG,
    _level: ULONG,
    _guid: LPCGUID,
    _h_session: TRACEHANDLE,
) -> ULONG {
    ERROR_SUCCESS
}

/// C: `ULONG WINAPI EtwQueryAllTracesW(PEVENT_TRACE_PROPERTIES*, ULONG, PULONG)`.
#[no_mangle]
pub unsafe extern "system" fn EtwQueryAllTracesW(
    _parray: PVOID,
    _arraycount: ULONG,
    psessioncount: *mut ULONG,
) -> ULONG {
    if !psessioncount.is_null() {
        write_volatile(psessioncount, 0);
    }
    ERROR_SUCCESS
}

/// C: `ULONG WINAPI EtwQueryAllTracesA(PEVENT_TRACE_PROPERTIES*, ULONG, PULONG)`.
#[no_mangle]
pub unsafe extern "system" fn EtwQueryAllTracesA(
    _parray: PVOID,
    _arraycount: ULONG,
    psessioncount: *mut ULONG,
) -> ULONG {
    if !psessioncount.is_null() {
        write_volatile(psessioncount, 0);
    }
    ERROR_SUCCESS
}

/// C: `ULONG WINAPI EtwFlushTraceA(TRACEHANDLE, LPCSTR, PEVENT_TRACE_PROPERTIES)`.
#[no_mangle]
pub unsafe extern "system" fn EtwFlushTraceA(
    h_session: TRACEHANDLE,
    session_name: LPCSTR,
    properties: PVOID,
) -> ULONG {
    EtwControlTraceA(h_session, session_name, properties, EVENT_TRACE_CONTROL_FLUSH)
}

/// C: `ULONG WINAPI EtwFlushTraceW(TRACEHANDLE, LPCWSTR, PEVENT_TRACE_PROPERTIES)`.
#[no_mangle]
pub unsafe extern "system" fn EtwFlushTraceW(
    h_session: TRACEHANDLE,
    session_name: LPCWSTR,
    properties: PVOID,
) -> ULONG {
    EtwControlTraceW(h_session, session_name, properties, EVENT_TRACE_CONTROL_FLUSH)
}

/// C: `ULONG WINAPI EtwQueryTraceA(TRACEHANDLE, LPCSTR, PEVENT_TRACE_PROPERTIES)`.
#[no_mangle]
pub unsafe extern "system" fn EtwQueryTraceA(
    h_session: TRACEHANDLE,
    session_name: LPCSTR,
    properties: PVOID,
) -> ULONG {
    EtwControlTraceA(h_session, session_name, properties, EVENT_TRACE_CONTROL_QUERY)
}

/// C: `ULONG WINAPI EtwQueryTraceW(TRACEHANDLE, LPCWSTR, PEVENT_TRACE_PROPERTIES)`.
#[no_mangle]
pub unsafe extern "system" fn EtwQueryTraceW(
    h_session: TRACEHANDLE,
    session_name: LPCWSTR,
    properties: PVOID,
) -> ULONG {
    EtwControlTraceW(h_session, session_name, properties, EVENT_TRACE_CONTROL_QUERY)
}

/// C: `ULONG WINAPI EtwStopTraceA(TRACEHANDLE, LPCSTR, PEVENT_TRACE_PROPERTIES)`.
#[no_mangle]
pub unsafe extern "system" fn EtwStopTraceA(
    h_session: TRACEHANDLE,
    session_name: LPCSTR,
    properties: PVOID,
) -> ULONG {
    EtwControlTraceA(h_session, session_name, properties, EVENT_TRACE_CONTROL_STOP)
}

/// C: `ULONG WINAPI EtwStopTraceW(TRACEHANDLE, LPCWSTR, PEVENT_TRACE_PROPERTIES)`.
#[no_mangle]
pub unsafe extern "system" fn EtwStopTraceW(
    h_session: TRACEHANDLE,
    session_name: LPCWSTR,
    properties: PVOID,
) -> ULONG {
    EtwControlTraceW(h_session, session_name, properties, EVENT_TRACE_CONTROL_STOP)
}

/// C: `ULONG WINAPI EtwUpdateTraceA(TRACEHANDLE, LPCSTR, PEVENT_TRACE_PROPERTIES)`.
#[no_mangle]
pub unsafe extern "system" fn EtwUpdateTraceA(
    h_session: TRACEHANDLE,
    session_name: LPCSTR,
    properties: PVOID,
) -> ULONG {
    EtwControlTraceA(h_session, session_name, properties, EVENT_TRACE_CONTROL_UPDATE)
}

/// C: `ULONG WINAPI EtwUpdateTraceW(TRACEHANDLE, LPCWSTR, PEVENT_TRACE_PROPERTIES)`.
#[no_mangle]
pub unsafe extern "system" fn EtwUpdateTraceW(
    h_session: TRACEHANDLE,
    session_name: LPCWSTR,
    properties: PVOID,
) -> ULONG {
    EtwControlTraceW(h_session, session_name, properties, EVENT_TRACE_CONTROL_UPDATE)
}
