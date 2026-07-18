//! wshirda.rs — the IrDA WinSock Helper DLL entry points (dll/win32/wshirda/wshirda.c).
//!
//! Every WSH* export is `UNIMPLEMENTED` in ReactOS: in the release (non-DBG) build the
//! `WSH_DbgPrint`/`UNIMPLEMENTED` macros (dll/win32/wshirda/debug.h) expand to nothing, so
//! each body is just `return 0`. `wsh_dbg_print!` below is that release no-op, kept so the
//! trace call sites stay visible (a future DBG build could wire it to DbgPrint). DllMain
//! disables thread-library-call notifications and returns TRUE.

use crate::prelude::*;

// WSH_DbgPrint / UNIMPLEMENTED are DBG-only in the C (debug.h `#else` defines them empty).
// This mirrors the release build: a no-op that discards its args.
macro_rules! wsh_dbg_print {
    ($level:expr, $($arg:tt)*) => {{
        let _ = $level;
    }};
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstDll: HANDLE, dwReason: DWORD, _Reserved: PVOID) -> BOOL {
    wsh_dbg_print!(1u32, "DllMain of wshirda.dll\n");

    match dwReason {
        // Don't need thread attach notifications, so disable them to improve performance.
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hInstDll);
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn WSHEnumProtocols(
    _lpiProtocols: LPINT,
    _lpTransportKeyName: LPWSTR,
    _lpProtocolBuffer: LPVOID,
    _lpdwBufferLength: LPDWORD,
) -> INT {
    wsh_dbg_print!(1u32, "WSHEnumProtocols unimplemented\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn WSHGetProviderGuid(
    _ProviderName: LPWSTR,
    _ProviderGuid: LPGUID,
) -> INT {
    wsh_dbg_print!(1u32, "WSHGetProviderGuid unimplemented\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn WSHGetSockaddrType(
    _Sockaddr: PSOCKADDR,
    _SockaddrLength: DWORD,
    _SockaddrInfo: PSOCKADDR_INFO,
) -> INT {
    wsh_dbg_print!(1u32, "WSHGetSockaddrType unimplemented\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn WSHGetSocketInformation(
    _HelperDllSocketContext: PVOID,
    _SocketHandle: SOCKET,
    _TdiAddressObjectHandle: HANDLE,
    _TdiConnectionObjectHandle: HANDLE,
    _Level: INT,
    _OptionName: INT,
    _OptionValue: PCHAR,
    _OptionLength: LPINT,
) -> INT {
    wsh_dbg_print!(1u32, "WSHGetSocketInformation unimplemented\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn WSHGetWSAProtocolInfo(
    _ProviderName: LPWSTR,
    _ProtocolInfo: *mut LPWSAPROTOCOL_INFOW,
    _ProtocolInfoEntries: LPDWORD,
) -> INT {
    wsh_dbg_print!(1u32, "WSHGetWSAProtocolInfo unimplemented\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn WSHGetWildcardSockaddr(
    _HelperDllSocketContext: PVOID,
    _Sockaddr: PSOCKADDR,
    _SockaddrLength: PINT,
) -> INT {
    wsh_dbg_print!(1u32, "WSHGetWildcardSockaddr unimplemented\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn WSHGetWinsockMapping(
    _Mapping: PWINSOCK_MAPPING,
    _MappingLength: DWORD,
) -> DWORD {
    wsh_dbg_print!(1u32, "WSHGetWinsockMapping unimplemented\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn WSHIoctl(
    _HelperDllSocketContext: PVOID,
    _SocketHandle: SOCKET,
    _TdiAddressObjectHandle: HANDLE,
    _TdiConnectionObjectHandle: HANDLE,
    _IoControlCode: DWORD,
    _InputBuffer: LPVOID,
    _InputBufferLength: DWORD,
    _OutputBuffer: LPVOID,
    _OutputBufferLength: DWORD,
    _NumberOfBytesReturned: LPDWORD,
    _Overlapped: LPWSAOVERLAPPED,
    _CompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
    _NeedsCompletion: LPBOOL,
) -> INT {
    wsh_dbg_print!(1u32, "WSHIoctl unimplemented\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn WSHNotify(
    _HelperDllSocketContext: PVOID,
    _SocketHandle: SOCKET,
    _TdiAddressObjectHandle: HANDLE,
    _TdiConnectionObjectHandle: HANDLE,
    _NotifyEvent: DWORD,
) -> INT {
    wsh_dbg_print!(1u32, "WSHNotify unimplemented\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn WSHOpenSocket(
    _AddressFamily: PINT,
    _SocketType: PINT,
    _Protocol: PINT,
    _TransportDeviceName: PUNICODE_STRING,
    _HelperDllSocketContext: PVOID,
    _NotificationEvents: PDWORD,
) -> INT {
    wsh_dbg_print!(1u32, "WSHOpenSocket unimplemented\n");
    0
}

#[no_mangle]
pub unsafe extern "system" fn WSHSetSocketInformation(
    _HelperDllSocketContext: PVOID,
    _SocketHandle: SOCKET,
    _TdiAddressObjectHandle: HANDLE,
    _TdiConnectionObjectHandle: HANDLE,
    _Level: INT,
    _OptionName: INT,
    _OptionValue: PCHAR,
    _OptionLength: INT,
) -> INT {
    wsh_dbg_print!(1u32, "WSHSetSocketInformation unimplemented\n");
    0
}
