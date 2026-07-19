use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// `int kickoff_telnetd(void);` — a forward declaration of the function defined in the sibling
// module (telnetd.c's body), so it is a re-export here, not a bodiless fn.
pub use crate::telnetd::telnetd::kickoff_telnetd;

static telnetdW: [u16; 8] = [
    b't' as u16,
    b'e' as u16,
    b'l' as u16,
    b'n' as u16,
    b'e' as u16,
    b't' as u16,
    b'd' as u16,
    0u16,
];

static mut service_handle: SERVICE_STATUS_HANDLE = core::ptr::null_mut();
static mut stop_event: HANDLE = core::ptr::null_mut();

// WINAPI. The x86 (`extern "stdcall"`) and non-x86 (`extern "C"`) translations had identical
// bodies, so they are collapsed into one `extern "system"` definition — which *is* stdcall on
// i386 and cdecl elsewhere. Same ABI, one body, and no fn-pointer transmute at the call site.
unsafe extern "system" fn service_handler(
    ctrl: DWORD,
    _event_type: DWORD,
    _event_data: LPVOID,
    _context: LPVOID,
) -> DWORD {
    let mut status: SERVICE_STATUS = core::mem::zeroed();
    status.dwServiceType = SERVICE_WIN32;
    status.dwControlsAccepted = SERVICE_ACCEPT_STOP;
    status.dwWin32ExitCode = 0;
    status.dwServiceSpecificExitCode = 0;
    status.dwCheckPoint = 0;
    status.dwWaitHint = 0;

    match ctrl {
        SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
            WINE_TRACE!(b"shutting down\n\0".as_ptr() as *const c_char);
            status.dwCurrentState = SERVICE_STOP_PENDING;
            status.dwControlsAccepted = 0;
            SetServiceStatus(service_handle, core::ptr::addr_of_mut!(status));
            SetEvent(stop_event);
            NO_ERROR
        }
        _ => {
            WINE_FIXME!(b"got service ctrl %x\n\0".as_ptr() as *const c_char, ctrl);
            status.dwCurrentState = SERVICE_RUNNING;
            SetServiceStatus(service_handle, core::ptr::addr_of_mut!(status));
            NO_ERROR
        }
    }
}

unsafe extern "system" fn serv_main(_argc: DWORD, _argv: *mut *mut u16) {
    let mut status: SERVICE_STATUS = core::mem::zeroed();

    WINE_TRACE!(b"starting service\n\0".as_ptr() as *const c_char);

    stop_event = CreateEventW(core::ptr::null_mut(), TRUE as BOOL, FALSE as BOOL, core::ptr::null());

    service_handle = RegisterServiceCtrlHandlerExW(
        telnetdW.as_ptr(),
        Some(service_handler),
        core::ptr::null_mut(),
    );
    if service_handle.is_null() {
        return;
    }

    status.dwServiceType = SERVICE_WIN32;
    status.dwCurrentState = SERVICE_RUNNING;
    status.dwControlsAccepted = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
    status.dwWin32ExitCode = 0;
    status.dwServiceSpecificExitCode = 0;
    status.dwCheckPoint = 0;
    status.dwWaitHint = 10000;
    SetServiceStatus(service_handle, core::ptr::addr_of_mut!(status));

    kickoff_telnetd();

    WaitForSingleObject(stop_event, INFINITE);

    status.dwCurrentState = SERVICE_STOPPED;
    status.dwControlsAccepted = 0;
    SetServiceStatus(service_handle, core::ptr::addr_of_mut!(status));
    WINE_TRACE!(b"service stopped\n\0".as_ptr() as *const c_char);
}

#[no_mangle]
pub unsafe extern "C" fn main(argc: c_int, argv: *mut *mut c_char) -> c_int {
    // C had this `static const`; a `const` cannot name a `static` (telnetdW) in Rust, so it is a
    // stack local. It only has to outlive the dispatcher call, and it does — that call blocks
    // until every service in the table has stopped.
    let servtbl: [SERVICE_TABLE_ENTRYW; 2] = [
        SERVICE_TABLE_ENTRYW {
            lpServiceName: telnetdW.as_ptr() as *mut u16,
            lpServiceProc: Some(serv_main),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: core::ptr::null_mut(),
            lpServiceProc: None,
        },
    ];
    WINE_TRACE!(b"(%d %p)\n\0".as_ptr() as *const c_char, argc, argv);
    StartServiceCtrlDispatcherW(servtbl.as_ptr());
    0
}



// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 2007 Jacek Caban for CodeWeavers
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <stdarg.h>
// #include <windef.h>
// #include <winbase.h>
// #include <winsvc.h>
// #include <wine/debug.h>
//
// int kickoff_telnetd(void);
//
// WINE_DEFAULT_DEBUG_CHANNEL(telnetd);
//
// static WCHAR telnetdW[] = {'t','e','l','n','e','t','d',0};
//
// static SERVICE_STATUS_HANDLE service_handle;
// static HANDLE stop_event;
//
// static DWORD WINAPI service_handler( DWORD ctrl, DWORD event_type, LPVOID event_data, LPVOID context )
// {
//     SERVICE_STATUS status;
//
//     status.dwServiceType             = SERVICE_WIN32;
//     status.dwControlsAccepted        = SERVICE_ACCEPT_STOP;
//     status.dwWin32ExitCode           = 0;
//     status.dwServiceSpecificExitCode = 0;
//     status.dwCheckPoint              = 0;
//     status.dwWaitHint                = 0;
//
//     switch(ctrl)
//     {
//     case SERVICE_CONTROL_STOP:
//     case SERVICE_CONTROL_SHUTDOWN:
//         WINE_TRACE( "shutting down\n" );
//         status.dwCurrentState     = SERVICE_STOP_PENDING;
//         status.dwControlsAccepted = 0;
//         SetServiceStatus( service_handle, &status );
//         SetEvent( stop_event );
//         return NO_ERROR;
//     default:
//         WINE_FIXME( "got service ctrl %x\n", ctrl );
//         status.dwCurrentState = SERVICE_RUNNING;
//         SetServiceStatus( service_handle, &status );
//         return NO_ERROR;
//     }
// }
//
// static void WINAPI serv_main(DWORD argc, LPWSTR *argv)
// {
//     SERVICE_STATUS status;
//
//     WINE_TRACE( "starting service\n" );
//
//     stop_event = CreateEventW( NULL, TRUE, FALSE, NULL );
//
//     service_handle = RegisterServiceCtrlHandlerExW( telnetdW, service_handler, NULL );
//     if (!service_handle)
//         return;
//
//     status.dwServiceType             = SERVICE_WIN32;
//     status.dwCurrentState            = SERVICE_RUNNING;
//     status.dwControlsAccepted        = SERVICE_ACCEPT_STOP | SERVICE_ACCEPT_SHUTDOWN;
//     status.dwWin32ExitCode           = 0;
//     status.dwServiceSpecificExitCode = 0;
//     status.dwCheckPoint              = 0;
//     status.dwWaitHint                = 10000;
//     SetServiceStatus( service_handle, &status );
//
//     /* Argument Ignored for now */
//     kickoff_telnetd();
//
//     WaitForSingleObject( stop_event, INFINITE );
//
//     status.dwCurrentState     = SERVICE_STOPPED;
//     status.dwControlsAccepted = 0;
//     SetServiceStatus( service_handle, &status );
//     WINE_TRACE( "service stopped\n" );
// }
//
// int main(int argc, char **argv)
// {
//     static const SERVICE_TABLE_ENTRYW servtbl[] = {
//         {telnetdW, serv_main},
//         {NULL, NULL}
//     };
//
//     WINE_TRACE("(%d %p)\n", argc, argv);
//
//     StartServiceCtrlDispatcherW(servtbl);
//     return 0;
// }
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
