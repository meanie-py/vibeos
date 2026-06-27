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
