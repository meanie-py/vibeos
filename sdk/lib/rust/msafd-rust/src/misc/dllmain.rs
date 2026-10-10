use crate::types::*;
use crate::glue::*;
use core::ffi::c_void;

// ---- SIO_* (ws2def.h) — computed by tools/gen_ioctl_consts.py through the _WSAIO* macro chain; a function-like macro is invisible to bindgen, so these arms were catch-all bindings (§9.112a) ----
pub const SIO_ADDRESS_LIST_QUERY: u32 = 0x48000016; // _WSAIOR(IOC_WS2,22)
pub const SIO_GET_EXTENSION_FUNCTION_POINTER: u32 = 0xc8000006; // _WSAIORW(IOC_WS2,6)

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// stdcall Win32/ntdll/ws2_32 imports: `extern "system"` so i386 gets the `_Name@N`
// decoration matching the DLL exports (extern "C" -> `_Name` cdecl -> undefined at link).
// Heap*/GetProcessHeap/SleepEx come from glue (extern "system") via `use crate::glue::*`;
// HandleToUlong/UlongToPtr/UlongToHandle/IS_INTRESOURCE are glue inline fns (not exports).
extern "system" {
    fn ntohl(netlong: u32) -> u32;
    fn ntohs(netshort: u16) -> u16;
    fn CreateFileMappingW( hFile: HANDLE, lpFileMappingAttributes: LPVOID, flProtect: DWORD, dwMaximumSizeHigh: DWORD, dwMaximumSizeLow: DWORD, lpName: LPVOID, ) -> HANDLE;
    fn DuplicateHandle( hSourceProcessHandle: HANDLE, hSourceHandle: HANDLE, hTargetProcessHandle: HANDLE, lpTargetHandle: LPHANDLE, dwDesiredAccess: DWORD, bInheritHandle: DWORD, dwOptions: DWORD, ) -> DWORD;
    fn GetCurrentProcess() -> HANDLE;
    fn MapViewOfFile( hFileMappingObject: HANDLE, dwDesiredAccess: DWORD, dwFileOffsetHigh: DWORD, dwFileOffsetLow: DWORD, dwNumberOfBytesToMap: usize, ) -> LPVOID;
    fn OpenProcess( dwDesiredAccess: DWORD, bInheritHandle: DWORD, dwProcessId: DWORD, ) -> HANDLE;
    fn UnmapViewOfFile(lpBaseAddress: PVOID) -> i32;
    fn EnterCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn WaitForSingleObjectEx(hObject: HANDLE, dwMilliseconds: DWORD, bAlertable: BOOL) -> DWORD;
    fn GetSystemTimeAsFileTime(lpSystemTimeAsFileTime: *mut FILETIME);
    fn SystemTimeToFileTime(lpSystemTime: *const SYSTEMTIME, lpFileTime: *mut FILETIME) -> i32;
    fn CreateThread( lpThreadAttributes: LPSECURITY_ATTRIBUTES, dwStackSize: SIZE_T, lpStartAddress: LPTHREAD_START_ROUTINE, lpParameter: LPVOID, dwCreationFlags: DWORD, lpThreadId: LPDWORD, ) -> HANDLE;
    fn GetCurrentThread() -> HANDLE;
    fn SetThreadPriority(hThread: HANDLE, nPriority: i32) -> i32;
    fn Sleep(dwMilliseconds: u32);
    fn DisableThreadLibraryCalls(hModule: HANDLE) -> BOOL;
    fn InitializeCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn DeleteCriticalSection(lpCriticalSection: *mut CRITICAL_SECTION);
    fn GetLastError() -> DWORD;
    fn PostQueuedCompletionStatus( CompletionPort: HANDLE, dwNumberOfBytesTransferred: DWORD, dwCompletionKey: ULONG_PTR, lpOverlapped: LPOVERLAPPED, ) -> BOOL;
    fn SetEvent(hEvent: HANDLE) -> BOOL;
}
// cdecl C-runtime (msvcrt) + msafd-internal SockGetTdiName (its def is extern "C").
extern "C" {
    fn swprintf(buffer: LPWSTR, format: *const u16, ...) -> i32;
    fn wcschr(s: *const u16, c: u16) -> *mut u16;
    fn wcscpy(dest: LPWSTR, src: *const u16) -> LPWSTR;
    fn wcslen(s: *const u16) -> usize;
    fn memset(s: *mut core::ffi::c_void, c: core::ffi::c_int, n: usize) -> *mut core::ffi::c_void;
    fn wcstol(nptr: *const u16, endptr: *mut *mut u16, base: core::ffi::c_int) -> core::ffi::c_long;
    fn SockGetTdiName( AddressFamily: *mut i32, SocketType: *mut i32, Protocol: *mut i32, g: GROUP, dwFlags: DWORD, TransportName: *mut _UNICODE_STRING, HelperDLLContext: *mut PVOID, HelperData: *mut PHELPER_DATA, HelperEvents: *mut DWORD, ) -> INT;
}

// ---- file-scope preamble (translated) ----
pub static mut GlobalHeap: HANDLE = core::ptr::null_mut();
pub static mut Upcalls: WSPUPCALLTABLE = WSPUPCALLTABLE {
    lpWPUCloseEvent: None,
    lpWPUCloseSocketHandle: None,
    lpWPUCreateEvent: None,
    lpWPUCreateSocketHandle: None,
    lpWPUFDIsSet: None,
    lpWPUGetProviderPath: None,
    lpWPUModifyIFSHandle: None,
    lpWPUPostMessage: None,
    lpWPUQueryBlockingCallback: None,
    lpWPUQuerySocketHandleContext: None,
    lpWPUQueueApc: None,
    lpWPUResetEvent: None,
    lpWPUSetEvent: None,
    lpWPUOpenCurrentThread: None,
    lpWPUCloseThread: None,
};
pub static mut CatalogEntryId: DWORD = 0;
pub static mut lpWPUCompleteOverlappedRequest: LPWPUCOMPLETEOVERLAPPEDREQUEST = None;
pub static mut SocketListHead: PSOCKET_INFORMATION = core::ptr::null_mut();
pub static mut SocketListLock: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: core::ptr::null_mut(),
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};
pub static mut SockHelpersListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
pub static mut SockAsyncThreadRefCount: ULONG = 0;
pub static mut SockAsyncHelperAfdHandle: HANDLE = core::ptr::null_mut();
pub static mut SockAsyncCompletionPort: HANDLE = core::ptr::null_mut();
pub static mut SockAsyncSelectCalled: BOOLEAN = 0;
#[no_mangle]
pub unsafe extern "system" fn WSPSocket(
    AddressFamily: i32,
    SocketType: i32,
    Protocol: i32,
    lpProtocolInfo: LPWSAPROTOCOL_INFOW,
    g: GROUP,
    dwFlags: DWORD,
    lpErrno: LPINT,
) -> SOCKET {
    let mut Object: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut SizeOfPacket: USHORT;
    let mut SizeOfEA: ULONG;
    let mut AfdPacket: PAFD_CREATE_PACKET;
    let mut Sock: HANDLE = core::ptr::null_mut();
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let mut EABuffer: PFILE_FULL_EA_INFORMATION = core::ptr::null_mut();
    let mut HelperData: PHELPER_DATA = core::ptr::null_mut();
    let mut HelperDLLContext: PVOID = core::ptr::null_mut();
    let mut HelperEvents: DWORD = 0;
    let mut TransportName: _UNICODE_STRING = core::mem::zeroed();
    let mut DevName: _UNICODE_STRING = core::mem::zeroed();
    let mut GroupData: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: INT;
    let mut SharedData: PSOCK_SHARED_INFO = core::ptr::null_mut();
    let mut AddressFamily = AddressFamily;
    let mut SocketType = SocketType;
    let mut Protocol = Protocol;

    TRACE!(b"Creating Socket, getting TDI Name - AddressFamily (%d)  SocketType (%d)  Protocol (%d).\n\0".as_ptr() as *const i8,
        AddressFamily, SocketType, Protocol);

    if !lpProtocolInfo.is_null() && (*lpProtocolInfo).dwServiceFlags3 != 0 && (*lpProtocolInfo).dwServiceFlags4 != 0 {
        if UlongToPtr((*lpProtocolInfo).dwServiceFlags3) == INVALID_HANDLE_VALUE {
            Status = (WSAEINVAL as i32);
            'error: {
                TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
                if !SharedData.is_null() {
                    UnmapViewOfFile(SharedData as PVOID);
                    NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
                } else {
                    if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                        HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                    }
                }
                if !Socket.is_null() {
                    HeapFree(GlobalHeap, 0, Socket as PVOID);
                }
                if !EABuffer.is_null() {
                    HeapFree(GlobalHeap, 0, EABuffer as PVOID);
                }
                if !lpErrno.is_null() {
                    *lpErrno = Status;
                }
                return INVALID_SOCKET;
            }
        }
        if UlongToPtr((*lpProtocolInfo).dwServiceFlags4) == INVALID_HANDLE_VALUE {
            Status = (WSAEINVAL as i32);
            'error: {
                TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
                if !SharedData.is_null() {
                    UnmapViewOfFile(SharedData as PVOID);
                    NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
                } else {
                    if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                        HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                    }
                }
                if !Socket.is_null() {
                    HeapFree(GlobalHeap, 0, Socket as PVOID);
                }
                if !EABuffer.is_null() {
                    HeapFree(GlobalHeap, 0, EABuffer as PVOID);
                }
                if !lpErrno.is_null() {
                    *lpErrno = Status;
                }
                return INVALID_SOCKET;
            }
        }
        SharedData = MapViewOfFile(UlongToHandle((*lpProtocolInfo).dwServiceFlags3),
                                   FILE_MAP_ALL_ACCESS,
                                   0,
                                   0,
                                   core::mem::size_of::<SOCK_SHARED_INFO>()) as PSOCK_SHARED_INFO;
        if SharedData.is_null() {
            Status = (WSAEINVAL as i32);
            'error: {
                TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
                if !SharedData.is_null() {
                    UnmapViewOfFile(SharedData as PVOID);
                    NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
                } else {
                    if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                        HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                    }
                }
                if !Socket.is_null() {
                    HeapFree(GlobalHeap, 0, Socket as PVOID);
                }
                if !EABuffer.is_null() {
                    HeapFree(GlobalHeap, 0, EABuffer as PVOID);
                }
                if !lpErrno.is_null() {
                    *lpErrno = Status;
                }
                return INVALID_SOCKET;
            }
        }
        InterlockedIncrement(core::ptr::addr_of_mut!((*SharedData).RefCount));
        AddressFamily = (*SharedData).AddressFamily;
        SocketType = (*SharedData).SocketType;
        Protocol = (*SharedData).Protocol;
    }

    if AddressFamily == (AF_UNSPEC as i32) && SocketType == 0 && Protocol == 0 {
        Status = (WSAEINVAL as i32);
        'error: {
            TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
            if !SharedData.is_null() {
                UnmapViewOfFile(SharedData as PVOID);
                NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
            } else {
                if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                    HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                }
            }
            if !Socket.is_null() {
                HeapFree(GlobalHeap, 0, Socket as PVOID);
            }
            if !EABuffer.is_null() {
                HeapFree(GlobalHeap, 0, EABuffer as PVOID);
            }
            if !lpErrno.is_null() {
                *lpErrno = Status;
            }
            return INVALID_SOCKET;
        }
    }

    if AddressFamily == (AF_UNSPEC as i32) {
        AddressFamily = (AF_INET as i32);
    }

    if SocketType == 0 {
        match Protocol {
            IPPROTO_TCP => SocketType = (SOCK_STREAM as i32),
            IPPROTO_UDP => SocketType = (SOCK_DGRAM as i32),
            IPPROTO_RAW => SocketType = (SOCK_RAW as i32),
            _ => {
                TRACE!(b"Unknown Protocol (%d). We will try SOCK_STREAM.\n\0".as_ptr() as *const i8, Protocol);
                SocketType = (SOCK_STREAM as i32);
            }
        }
    }

    if Protocol == 0 {
        match SocketType as u32 {
            SOCK_STREAM => Protocol = IPPROTO_TCP,
            SOCK_DGRAM => Protocol = IPPROTO_UDP,
            SOCK_RAW => Protocol = IPPROTO_RAW,
            _ => {
                TRACE!(b"Unknown SocketType (%d). We will try IPPROTO_TCP.\n\0".as_ptr() as *const i8, SocketType);
                Protocol = IPPROTO_TCP;
            }
        }
    }

    Status = SockGetTdiName(core::ptr::addr_of_mut!(AddressFamily),
                            core::ptr::addr_of_mut!(SocketType),
                            core::ptr::addr_of_mut!(Protocol),
                            g,
                            dwFlags,
                            core::ptr::addr_of_mut!(TransportName),
                            core::ptr::addr_of_mut!(HelperDLLContext),
                            core::ptr::addr_of_mut!(HelperData),
                            core::ptr::addr_of_mut!(HelperEvents));

    if Status != (NO_ERROR as i32) {
        ERR!("SockGetTdiName: Status %x\n", Status);
        'error: {
            TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
            if !SharedData.is_null() {
                UnmapViewOfFile(SharedData as PVOID);
                NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
            } else {
                if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                    HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                }
            }
            if !Socket.is_null() {
                HeapFree(GlobalHeap, 0, Socket as PVOID);
            }
            if !EABuffer.is_null() {
                HeapFree(GlobalHeap, 0, EABuffer as PVOID);
            }
            if !lpErrno.is_null() {
                *lpErrno = Status;
            }
            return INVALID_SOCKET;
        }
    }

    // \Device\Afd\Endpoint as REAL UTF-16 (the L"..." literal). A byte-string-as-u16 cast
    // mis-encodes it to ASCII pairs -> RtlInitUnicodeString over-scans / NtCreateFile gets a
    // garbage device name (same bug class as the afd.sys \Device\Afd device-name fix).
    static AFD_ENDPOINT: [u16; 21] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
        b'\\' as u16, b'A' as u16, b'f' as u16, b'd' as u16,
        b'\\' as u16, b'E' as u16, b'n' as u16, b'd' as u16, b'p' as u16, b'o' as u16, b'i' as u16,
        b'n' as u16, b't' as u16, 0,
    ];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(DevName), AFD_ENDPOINT.as_ptr());

    Socket = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<SOCKET_INFORMATION>()) as PSOCKET_INFORMATION;
    if Socket.is_null() {
        Status = (WSAENOBUFS as i32);
        'error: {
            TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
            if !SharedData.is_null() {
                UnmapViewOfFile(SharedData as PVOID);
                NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
            } else {
                if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                    HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                }
            }
            if !Socket.is_null() {
                HeapFree(GlobalHeap, 0, Socket as PVOID);
            }
            if !EABuffer.is_null() {
                HeapFree(GlobalHeap, 0, EABuffer as PVOID);
            }
            if !lpErrno.is_null() {
                *lpErrno = Status;
            }
            return INVALID_SOCKET;
        }
    }
    RtlZeroMemory(Socket as PVOID, core::mem::size_of::<SOCKET_INFORMATION>());
    if !SharedData.is_null() {
        (*Socket).SharedData = SharedData;
        (*Socket).SharedDataHandle = UlongToHandle((*lpProtocolInfo).dwServiceFlags3);
        Sock = UlongToHandle((*lpProtocolInfo).dwServiceFlags4);
        (*Socket).Handle = (*lpProtocolInfo).dwServiceFlags4 as SOCKET;
    } else {
        (*Socket).SharedDataHandle = INVALID_HANDLE_VALUE;
        (*Socket).SharedData = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<SOCK_SHARED_INFO>()) as PSOCK_SHARED_INFO;
        if (*Socket).SharedData.is_null() {
            Status = (WSAENOBUFS as i32);
            'error: {
                TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
                if !SharedData.is_null() {
                    UnmapViewOfFile(SharedData as PVOID);
                    NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
                } else {
                    if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                        HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                    }
                }
                if !Socket.is_null() {
                    HeapFree(GlobalHeap, 0, Socket as PVOID);
                }
                if !EABuffer.is_null() {
                    HeapFree(GlobalHeap, 0, EABuffer as PVOID);
                }
                if !lpErrno.is_null() {
                    *lpErrno = Status;
                }
                return INVALID_SOCKET;
            }
        }
        RtlZeroMemory((*Socket).SharedData as PVOID, core::mem::size_of::<SOCK_SHARED_INFO>());
        (*(*Socket).SharedData).State = SocketOpen;
        (*(*Socket).SharedData).RefCount = 1;
        (*(*Socket).SharedData).__bindgen_anon_1.set_Listening(0u8);
        (*(*Socket).SharedData).AddressFamily = AddressFamily;
        (*(*Socket).SharedData).SocketType = SocketType;
        (*(*Socket).SharedData).Protocol = Protocol;
        (*(*Socket).SharedData).SizeOfLocalAddress = (*HelperData).MaxWSAddressLength;
        (*(*Socket).SharedData).SizeOfRemoteAddress = (*HelperData).MaxWSAddressLength;
        (*(*Socket).SharedData).__bindgen_anon_1.set_UseDelayedAcceptance((*HelperData).UseDelayedAcceptance);
        (*(*Socket).SharedData).CreateFlags = dwFlags;
        (*(*Socket).SharedData).ServiceFlags1 = (*lpProtocolInfo).dwServiceFlags1;
        (*(*Socket).SharedData).ProviderFlags = (*lpProtocolInfo).dwProviderFlags;
        (*(*Socket).SharedData).__bindgen_anon_1.set_UseSAN(0u8);
        (*(*Socket).SharedData).__bindgen_anon_1.set_NonBlocking(0u8);
        (*(*Socket).SharedData).RecvTimeout = INFINITE;
        (*(*Socket).SharedData).SendTimeout = INFINITE;
        (*(*Socket).SharedData).__bindgen_anon_1.set_OobInline(0u8);

        if (*(*Socket).SharedData).SocketType == SOCK_DGRAM as i32 ||
           (*(*Socket).SharedData).SocketType == (SOCK_RAW as i32) {
            TRACE!(b"Connectionless socket\n\0".as_ptr() as *const i8);
            (*(*Socket).SharedData).ServiceFlags1 |= XP1_CONNECTIONLESS;
        }
        (*Socket).Handle = INVALID_SOCKET;
    }

    (*Socket).HelperContext = HelperDLLContext;
    (*Socket).HelperData = HelperData;
    (*Socket).HelperEvents = HelperEvents;
    (*Socket).LocalAddress = core::ptr::addr_of_mut!((*(*Socket).SharedData).WSLocalAddress);
    (*Socket).RemoteAddress = core::ptr::addr_of_mut!((*(*Socket).SharedData).WSRemoteAddress);
    (*Socket).SanData = core::ptr::null_mut();
    core::ptr::copy_nonoverlapping(lpProtocolInfo, core::ptr::addr_of_mut!((*Socket).ProtocolInfo), 1);
    if !SharedData.is_null() {
        'ok: {
            EnterCriticalSection(core::ptr::addr_of_mut!(SocketListLock));
            (*Socket).NextSocket = SocketListHead;
            SocketListHead = Socket;
            LeaveCriticalSection(core::ptr::addr_of_mut!(SocketListLock));

            CreateContext(Socket);

            (Upcalls.lpWPUModifyIFSHandle.unwrap())((*Socket).ProtocolInfo.dwCatalogEntryId, Sock as SOCKET, lpErrno);

            TRACE!(b"Success %x\n\0".as_ptr() as *const i8, Sock);

            return Sock as SOCKET;
        }
    }

    SizeOfPacket = TransportName.Length + core::mem::size_of::<AFD_CREATE_PACKET>() as u16 + core::mem::size_of::<u16>() as u16;

    SizeOfEA = SizeOfPacket as ULONG + core::mem::size_of::<FILE_FULL_EA_INFORMATION>() as ULONG + AFD_PACKET_COMMAND_LENGTH as ULONG;

    EABuffer = HeapAlloc(GlobalHeap, 0, SizeOfEA as usize) as PFILE_FULL_EA_INFORMATION;
    if EABuffer.is_null() {
        Status = (WSAENOBUFS as i32);
        'error: {
            TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
            if !SharedData.is_null() {
                UnmapViewOfFile(SharedData as PVOID);
                NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
            } else {
                if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                    HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                }
            }
            if !Socket.is_null() {
                HeapFree(GlobalHeap, 0, Socket as PVOID);
            }
            if !EABuffer.is_null() {
                HeapFree(GlobalHeap, 0, EABuffer as PVOID);
            }
            if !lpErrno.is_null() {
                *lpErrno = Status;
            }
            return INVALID_SOCKET;
        }
    }

    RtlZeroMemory(EABuffer as PVOID, SizeOfEA as usize);
    (*EABuffer).NextEntryOffset = 0;
    (*EABuffer).Flags = 0;
    (*EABuffer).EaNameLength = AFD_PACKET_COMMAND_LENGTH as u8;
    core::ptr::copy_nonoverlapping(AfdCommand.as_ptr(), (*EABuffer).EaName.as_mut_ptr() as *mut u8, (AFD_PACKET_COMMAND_LENGTH + 1) as usize);
    (*EABuffer).EaValueLength = SizeOfPacket as u16;

    AfdPacket = ((*EABuffer).EaName.as_mut_ptr() as *mut i8).offset((*EABuffer).EaNameLength as isize + 1) as PAFD_CREATE_PACKET;
    (*AfdPacket).SizeOfTransportName = (TransportName.Length as u32);
    core::ptr::copy_nonoverlapping(TransportName.Buffer, (*AfdPacket).TransportName.as_mut_ptr(), (TransportName.Length as usize + core::mem::size_of::<u16>()) / core::mem::size_of::<u16>());
    (*AfdPacket).GroupID = g;

    if ((*(*Socket).SharedData).ServiceFlags1 & XP1_CONNECTIONLESS) != 0 {
        if SocketType != (SOCK_DGRAM as i32) && SocketType != (SOCK_RAW as i32) {
            Status = (WSAEINVAL as i32);
            'error: {
                TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
                if !SharedData.is_null() {
                    UnmapViewOfFile(SharedData as PVOID);
                    NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
                } else {
                    if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                        HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                    }
                }
                if !Socket.is_null() {
                    HeapFree(GlobalHeap, 0, Socket as PVOID);
                }
                if !EABuffer.is_null() {
                    HeapFree(GlobalHeap, 0, EABuffer as PVOID);
                }
                if !lpErrno.is_null() {
                    *lpErrno = Status;
                }
                return INVALID_SOCKET;
            }
        }
        (*AfdPacket).EndpointFlags |= AFD_ENDPOINT_CONNECTIONLESS;
    }

    if ((*(*Socket).SharedData).ServiceFlags1 & XP1_MESSAGE_ORIENTED) != 0 {
        if SocketType == (SOCK_STREAM as i32) {
            if ((*(*Socket).SharedData).ServiceFlags1 & XP1_PSEUDO_STREAM) == 0 {
                Status = (WSAEINVAL as i32);
                'error: {
                    TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
                    if !SharedData.is_null() {
                        UnmapViewOfFile(SharedData as PVOID);
                        NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
                    } else {
                        if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                            HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                        }
                    }
                    if !Socket.is_null() {
                        HeapFree(GlobalHeap, 0, Socket as PVOID);
                    }
                    if !EABuffer.is_null() {
                        HeapFree(GlobalHeap, 0, EABuffer as PVOID);
                    }
                    if !lpErrno.is_null() {
                        *lpErrno = Status;
                    }
                    return INVALID_SOCKET;
                }
            }
        }
        (*AfdPacket).EndpointFlags |= AFD_ENDPOINT_MESSAGE_ORIENTED;
    }

    if SocketType == (SOCK_RAW as i32) {
        (*AfdPacket).EndpointFlags |= AFD_ENDPOINT_RAW;
    }

    if (dwFlags & (WSA_FLAG_MULTIPOINT_C_ROOT |
                   WSA_FLAG_MULTIPOINT_C_LEAF |
                   WSA_FLAG_MULTIPOINT_D_ROOT |
                   WSA_FLAG_MULTIPOINT_D_LEAF)) != 0 {
        if ((*(*Socket).SharedData).ServiceFlags1 & XP1_SUPPORT_MULTIPOINT) == 0 {
            Status = (WSAEINVAL as i32);
            'error: {
                TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
                if !SharedData.is_null() {
                    UnmapViewOfFile(SharedData as PVOID);
                    NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
                } else {
                    if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                        HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                    }
                }
                if !Socket.is_null() {
                    HeapFree(GlobalHeap, 0, Socket as PVOID);
                }
                if !EABuffer.is_null() {
                    HeapFree(GlobalHeap, 0, EABuffer as PVOID);
                }
                if !lpErrno.is_null() {
                    *lpErrno = Status;
                }
                return INVALID_SOCKET;
            }
        }
        (*AfdPacket).EndpointFlags |= AFD_ENDPOINT_MULTIPOINT;

        if (dwFlags & WSA_FLAG_MULTIPOINT_C_ROOT) != 0 {
            if (((*(*Socket).SharedData).ServiceFlags1 & XP1_MULTIPOINT_CONTROL_PLANE) == 0) ||
               ((dwFlags & WSA_FLAG_MULTIPOINT_C_LEAF) != 0) {
                Status = (WSAEINVAL as i32);
                'error: {
                    TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
                    if !SharedData.is_null() {
                        UnmapViewOfFile(SharedData as PVOID);
                        NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
                    } else {
                        if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                            HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                        }
                    }
                    if !Socket.is_null() {
                        HeapFree(GlobalHeap, 0, Socket as PVOID);
                    }
                    if !EABuffer.is_null() {
                        HeapFree(GlobalHeap, 0, EABuffer as PVOID);
                    }
                    if !lpErrno.is_null() {
                        *lpErrno = Status;
                    }
                    return INVALID_SOCKET;
                }
            }
            (*AfdPacket).EndpointFlags |= AFD_ENDPOINT_C_ROOT;
        }

        if (dwFlags & WSA_FLAG_MULTIPOINT_D_ROOT) != 0 {
            if (((*(*Socket).SharedData).ServiceFlags1 & XP1_MULTIPOINT_DATA_PLANE) == 0) ||
               ((dwFlags & WSA_FLAG_MULTIPOINT_D_LEAF) != 0) {
                Status = (WSAEINVAL as i32);
                'error: {
                    TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
                    if !SharedData.is_null() {
                        UnmapViewOfFile(SharedData as PVOID);
                        NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
                    } else {
                        if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                            HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                        }
                    }
                    if !Socket.is_null() {
                        HeapFree(GlobalHeap, 0, Socket as PVOID);
                    }
                    if !EABuffer.is_null() {
                        HeapFree(GlobalHeap, 0, EABuffer as PVOID);
                    }
                    if !lpErrno.is_null() {
                        *lpErrno = Status;
                    }
                    return INVALID_SOCKET;
                }
            }
            (*AfdPacket).EndpointFlags |= AFD_ENDPOINT_D_ROOT;
        }
    }

    Object.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    Object.RootDirectory = core::ptr::null_mut();
    Object.ObjectName = core::ptr::addr_of_mut!(DevName);
    Object.Attributes = OBJ_CASE_INSENSITIVE | OBJ_INHERIT;
    Object.SecurityDescriptor = core::ptr::null_mut();
    Object.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(core::ptr::addr_of_mut!(Sock),
                          GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
                          core::ptr::addr_of_mut!(Object),
                          core::ptr::addr_of_mut!(IOSB),
                          core::ptr::null_mut(),
                          0,
                          FILE_SHARE_READ | FILE_SHARE_WRITE,
                          FILE_OPEN_IF,
                          0,
                          EABuffer as PVOID,
                          SizeOfEA);

    HeapFree(GlobalHeap, 0, EABuffer as PVOID);

    if !NT_SUCCESS(Status) {
        ERR!("Failed to open socket. Status 0x%08x\n", Status);
        Status = TranslateNtStatusError(Status);
        'error: {
            TRACE!(b"Ending %x\n\0".as_ptr() as *const i8, Status);
            if !SharedData.is_null() {
                UnmapViewOfFile(SharedData as PVOID);
                NtClose(UlongToHandle((*lpProtocolInfo).dwServiceFlags3));
            } else {
                if !Socket.is_null() && !(*Socket).SharedData.is_null() {
                    HeapFree(GlobalHeap, 0, (*Socket).SharedData as PVOID);
                }
            }
            if !Socket.is_null() {
                HeapFree(GlobalHeap, 0, Socket as PVOID);
            }
            if !EABuffer.is_null() {
                HeapFree(GlobalHeap, 0, EABuffer as PVOID);
            }
            if !lpErrno.is_null() {
                *lpErrno = Status;
            }
            return INVALID_SOCKET;
        }
    }

    (*Socket).Handle = Sock as SOCKET;

    if g != 0 {
        GetSocketInformation(Socket,
                             AFD_INFO_GROUP_ID_TYPE,
                             core::ptr::null_mut(),
                             core::ptr::null_mut(),
                             core::ptr::addr_of_mut!(GroupData),
                             core::ptr::null_mut(),
                             None);
        (*(*Socket).SharedData).GroupID = GroupData.u.LowPart;
        (*(*Socket).SharedData).GroupType = (GroupData.u.HighPart as i32 as u32);
    }

    GetSocketInformation(Socket,
                         AFD_INFO_SEND_WINDOW_SIZE,
                         core::ptr::null_mut(),
                         core::ptr::addr_of_mut!((*(*Socket).SharedData).SizeOfSendBuffer),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         None);

    GetSocketInformation(Socket,
                         AFD_INFO_RECEIVE_WINDOW_SIZE,
                         core::ptr::null_mut(),
                         core::ptr::addr_of_mut!((*(*Socket).SharedData).SizeOfRecvBuffer),
                         core::ptr::null_mut(),
                         core::ptr::null_mut(),
                         None);

    EnterCriticalSection(core::ptr::addr_of_mut!(SocketListLock));
    (*Socket).NextSocket = SocketListHead;
    SocketListHead = Socket;
    LeaveCriticalSection(core::ptr::addr_of_mut!(SocketListLock));

    CreateContext(Socket);

    (Upcalls.lpWPUModifyIFSHandle.unwrap())((*Socket).ProtocolInfo.dwCatalogEntryId, Sock as SOCKET, lpErrno);

    TRACE!(b"Success %x\n\0".as_ptr() as *const i8, Sock);

    Sock as SOCKET
}
#[no_mangle]
pub unsafe extern "system" fn WSPDuplicateSocket(
    Handle: SOCKET,
    dwProcessId: DWORD,
    lpProtocolInfo: LPWSAPROTOCOL_INFOW,
    lpErrno: LPINT,
) -> INT {
    let mut hProcess: HANDLE;
    let mut hDuplicatedSharedData: HANDLE = core::ptr::null_mut();
    let mut hDuplicatedHandle: HANDLE = core::ptr::null_mut();
    let Socket: PSOCKET_INFORMATION;
    let mut pSharedData: PSOCK_SHARED_INFO;
    let pOldSharedData: PSOCK_SHARED_INFO;
    let mut bDuplicated: DWORD;

    if Handle == INVALID_SOCKET as SOCKET {
        return (MsafdReturnWithErrno(STATUS_INVALID_PARAMETER, lpErrno, 0, core::ptr::null_mut()) as i32);
    }
    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }
    hProcess = OpenProcess(PROCESS_DUP_HANDLE, 0, dwProcessId);
    if hProcess.is_null() {
        return (MsafdReturnWithErrno(STATUS_INVALID_PARAMETER, lpErrno, 0, core::ptr::null_mut()) as i32);
    }

    if (*Socket).SharedDataHandle == INVALID_HANDLE_VALUE {
        (*Socket).SharedDataHandle = CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            core::ptr::null_mut(),
            PAGE_READWRITE | SEC_COMMIT,
            0,
            (((core::mem::size_of::<SOCK_SHARED_INFO>() + (PAGE_SIZE as usize) - 1) & (!(PAGE_SIZE as usize - 1))) as u32),
            core::ptr::null_mut(),
        );
        if (*Socket).SharedDataHandle == INVALID_HANDLE_VALUE {
            return (MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, core::ptr::null_mut()) as i32);
        }
        pSharedData = MapViewOfFile(
            (*Socket).SharedDataHandle,
            FILE_MAP_ALL_ACCESS,
            0,
            0,
            core::mem::size_of::<SOCK_SHARED_INFO>(),
        ) as PSOCK_SHARED_INFO;

        RtlCopyMemory(
            pSharedData as LPVOID,
            (*Socket).SharedData as LPVOID,
            core::mem::size_of::<SOCK_SHARED_INFO>(),
        );
        pOldSharedData = (*Socket).SharedData;
        (*Socket).SharedData = pSharedData;
        HeapFree(GlobalHeap, 0, pOldSharedData as LPVOID);
    }

    bDuplicated = DuplicateHandle(
        GetCurrentProcess(),
        (*Socket).SharedDataHandle,
        hProcess,
        core::ptr::addr_of_mut!(hDuplicatedSharedData),
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    );
    if bDuplicated == 0 {
        NtClose(hProcess);
        return (MsafdReturnWithErrno(STATUS_ACCESS_DENIED, lpErrno, 0, core::ptr::null_mut()) as i32);
    }
    bDuplicated = DuplicateHandle(
        GetCurrentProcess(),
        (*Socket).Handle as HANDLE,
        hProcess,
        core::ptr::addr_of_mut!(hDuplicatedHandle),
        0,
        0,
        DUPLICATE_SAME_ACCESS,
    );
    NtClose(hProcess);
    if bDuplicated == 0 {
        return (MsafdReturnWithErrno(STATUS_ACCESS_DENIED, lpErrno, 0, core::ptr::null_mut()) as i32);
    }

    if lpProtocolInfo.is_null() {
        return (MsafdReturnWithErrno(STATUS_ACCESS_VIOLATION, lpErrno, 0, core::ptr::null_mut()) as i32);
    }

    RtlCopyMemory(
        lpProtocolInfo as LPVOID,
        core::ptr::addr_of!((*Socket).ProtocolInfo) as LPVOID,
        core::mem::size_of::<WSAPROTOCOL_INFOW>(),
    );

    (*lpProtocolInfo).iAddressFamily = (*(*Socket).SharedData).AddressFamily;
    (*lpProtocolInfo).iProtocol = (*(*Socket).SharedData).Protocol;
    (*lpProtocolInfo).iSocketType = (*(*Socket).SharedData).SocketType;
    (*lpProtocolInfo).dwServiceFlags3 = HandleToUlong(hDuplicatedSharedData);
    (*lpProtocolInfo).dwServiceFlags4 = HandleToUlong(hDuplicatedHandle);

    if !lpErrno.is_null() {
        *lpErrno = NO_ERROR as INT;
    }

    return NO_ERROR as INT;
}
// MsafdReturnWithErrno — FORCEINLINE helper in msafd.h (no .c body, so never translated by
// the per-function pass). Faithful translation of the header inline: map the NTSTATUS to a
// winsock errno via TranslateNtStatusError, publish the byte count, and return 0/SOCKET_ERROR.
#[no_mangle]
pub unsafe extern "C" fn MsafdReturnWithErrno(
    Status: NTSTATUS,
    Errno: LPINT,
    Received: DWORD,
    ReturnedBytes: LPDWORD,
) -> DWORD {
    if !Errno.is_null() {
        *Errno = TranslateNtStatusError(Status);
        if !ReturnedBytes.is_null() {
            *ReturnedBytes = if *Errno == 0 { Received } else { 0 };
        }
        return if *Errno == 0 { 0 } else { SOCKET_ERROR as DWORD };
    } else {
        if !ReturnedBytes.is_null() {
            *ReturnedBytes = if Status == STATUS_SUCCESS { Received } else { 0 };
        }
        return if Status == STATUS_SUCCESS { 0 } else { SOCKET_ERROR as DWORD };
    }
}
#[no_mangle]
pub unsafe extern "C" fn TranslateNtStatusError(Status: NTSTATUS) -> INT {
    // NOTE: these three NTSTATUS consts are missing from the central glue/types
    // (ntdll_sys::nt / ntdll-sys constants). Values from sdk/include/psdk/ntstatus.h.
    const STATUS_REMOTE_NOT_LISTENING: NTSTATUS = 0xC00000BCu32 as NTSTATUS;
    const STATUS_LOCAL_DISCONNECT: NTSTATUS = 0xC000013Bu32 as NTSTATUS;
    const STATUS_REMOTE_DISCONNECT: NTSTATUS = 0xC000013Cu32 as NTSTATUS;
    // Not in the C's table, so they fell to the "unhandled" WSAENETDOWN arm: a TCP
    // connect that timed out read as "network is down" and a refused one logged an
    // MSAFD error per call. Windows maps them to WSAETIMEDOUT / WSAECONNREFUSED, and
    // netstack completes connects with both.
    const STATUS_IO_TIMEOUT: NTSTATUS = 0xC00000B5u32 as NTSTATUS;
    const STATUS_CONNECTION_REFUSED: NTSTATUS = 0xC0000236u32 as NTSTATUS;
    match Status {
        STATUS_CANT_WAIT => (WSAEWOULDBLOCK as i32),
        STATUS_TIMEOUT | STATUS_IO_TIMEOUT => WSAETIMEDOUT as i32,
        STATUS_CONNECTION_REFUSED => WSAECONNREFUSED as i32,
        STATUS_SUCCESS => NO_ERROR as INT,
        STATUS_FILE_CLOSED => WSAECONNRESET as i32,
        STATUS_END_OF_FILE => WSAESHUTDOWN as i32,
        STATUS_PENDING => WSA_IO_PENDING as i32,
        STATUS_BUFFER_TOO_SMALL | STATUS_BUFFER_OVERFLOW => WSAEMSGSIZE as i32,
        STATUS_NO_MEMORY | STATUS_INSUFFICIENT_RESOURCES => WSAENOBUFS as i32,
        STATUS_INVALID_CONNECTION => WSAENOTCONN as i32,
        STATUS_PROTOCOL_NOT_SUPPORTED => WSAEAFNOSUPPORT as i32,
        STATUS_INVALID_ADDRESS => WSAEADDRNOTAVAIL as i32,
        STATUS_REMOTE_NOT_LISTENING | STATUS_REMOTE_DISCONNECT => WSAECONNREFUSED as i32,
        STATUS_NETWORK_UNREACHABLE => WSAENETUNREACH as i32,
        STATUS_HOST_UNREACHABLE => WSAEHOSTUNREACH as i32,
        STATUS_INVALID_PARAMETER => WSAEINVAL as i32,
        STATUS_CANCELLED => WSA_OPERATION_ABORTED as i32,
        STATUS_ADDRESS_ALREADY_EXISTS => WSAEADDRINUSE as i32,
        STATUS_LOCAL_DISCONNECT => WSAECONNABORTED as i32,
        STATUS_ACCESS_VIOLATION => WSAEFAULT as i32,
        STATUS_ACCESS_DENIED => WSAEACCES as i32,
        STATUS_NOT_IMPLEMENTED => WSAEOPNOTSUPP as i32,
        _ => {
            ERR!("MSAFD: Unhandled NTSTATUS value: 0x%x\n", Status);
            (WSAENETDOWN as i32)
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn MsafdWaitForAlert(hObject: HANDLE) {

    let mut Result: DWORD;

    loop {
        Result = WaitForSingleObjectEx(hObject, INFINITE, 1);
        if Result != WAIT_IO_COMPLETION {
            break;
        }
    }
}

/// Wait for a synchronous send/receive, honouring SO_SNDTIMEO/SO_RCVTIMEO.
///
/// Upstream waits forever ("FIXME: Shouldn't wait infinitely for receive..."), so a 3s
/// SO_RCVTIMEO blocked a recvfrom for 9 minutes. Windows instead cancels the request when
/// the timeout expires and fails the call with WSAETIMEDOUT. 0 and INFINITE both mean "no
/// timeout". If the request completes in the window between the timeout and the cancel, its
/// real result wins. Returns the request's final status.
pub unsafe fn MsafdWaitForIo(
    Handle: HANDLE,
    hObject: HANDLE,
    IOSB: PIO_STATUS_BLOCK,
    Timeout: DWORD,
) -> NTSTATUS {
    const WAIT_TIMEOUT: DWORD = 0x102;
    const STATUS_IO_TIMEOUT: NTSTATUS = 0xC00000B5u32 as NTSTATUS;

    if Timeout == 0 || Timeout == INFINITE {
        MsafdWaitForAlert(hObject);
        return (*IOSB).__bindgen_anon_1.Status;
    }

    let Start = GetTickCount();
    loop {
        // An APC (WAIT_IO_COMPLETION) restarts the wait with what is left of the timeout.
        let Elapsed = GetTickCount().wrapping_sub(Start);
        let Result = if Elapsed >= Timeout {
            WAIT_TIMEOUT
        } else {
            WaitForSingleObjectEx(hObject, Timeout - Elapsed, 1)
        };
        if Result == WAIT_IO_COMPLETION {
            continue;
        }
        if Result != WAIT_TIMEOUT {
            return (*IOSB).__bindgen_anon_1.Status;
        }
        break;
    }

    let mut CancelIosb: IO_STATUS_BLOCK = core::mem::zeroed();
    NtCancelIoFile(Handle, &mut CancelIosb);
    MsafdWaitForAlert(hObject);
    let Status = (*IOSB).__bindgen_anon_1.Status;
    if Status == STATUS_CANCELLED {
        STATUS_IO_TIMEOUT
    } else {
        Status
    }
}
#[no_mangle]
pub unsafe extern "system" fn WSPCloseSocket(
    Handle: SOCKET,
    lpErrno: LPINT,
) -> INT {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let mut CurrentSocket: PSOCKET_INFORMATION;
    let mut Status: NTSTATUS;
    let mut SockEvent: HANDLE = core::ptr::null_mut();
    let mut DisconnectInfo: AFD_DISCONNECT_INFO = core::mem::zeroed();
    let mut OldState: SOCKET_STATE;
    let mut LingerWait: LONG = -1;
    let References: DWORD;

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }

    SleepEx(0, 1);

    if ((*Socket).HelperEvents & WSH_NOTIFY_CLOSE) != 0 {
        Status = ((*(*Socket).HelperData).WSHNotify.unwrap())(
            (*Socket).HelperContext,
            (*Socket).Handle,
            (*Socket).TdiAddressHandle,
            (*Socket).TdiConnectionHandle,
            WSH_NOTIFY_CLOSE,
        );

        if Status != 0 {
            ERR!("WSHNotify failed. Error 0x%#x\n", Status);
            if !lpErrno.is_null() {
                *lpErrno = Status as INT;
            }
            return SOCKET_ERROR;
        }
    }

    if (*(*Socket).SharedData).State == SocketClosed {
        WARN!(b"Socket is closing.\n\0".as_ptr() as *const i8);
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }

    Status = NtCreateEvent(
        &mut SockEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0,
    );
    if !NT_SUCCESS(Status) {
        ERR!("NtCreateEvent failed: 0x%08x\n", Status);
        return SOCKET_ERROR;
    }

    References = InterlockedDecrement(&mut (*(*Socket).SharedData).RefCount) as DWORD;
    if References != 0 {
        // goto ok
    } else {
        OldState = (*(*Socket).SharedData).State;
        (*(*Socket).SharedData).State = SocketClosed;

        if (OldState == SocketConnected) && ((*(*Socket).SharedData).LingerData.l_onoff != 0) {
            let mut SendsInProgress: ULONG = 0;
            let SleepWait: ULONG;

            SleepWait = 100;
            LingerWait = ((((*(*Socket).SharedData).LingerData.l_linger * (1000 as i32 as i32) as u16)) as i32);

            while LingerWait != 0 {
                if GetSocketInformation(
                    Socket,
                    AFD_INFO_SENDS_IN_PROGRESS,
                    core::ptr::null_mut(),
                    &mut SendsInProgress,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    None,
                ) != 0
                {
                    LingerWait = 0;
                    break;
                }

                if SendsInProgress == 0 {
                    LingerWait = -1;
                    break;
                }

                if (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
                    WARN!(b"Would block!\n\0".as_ptr() as *const i8);
                    NtClose(SockEvent);
                    (*(*Socket).SharedData).State = OldState;
                    if !lpErrno.is_null() {
                        *lpErrno = (WSAEWOULDBLOCK as i32);
                    }
                    return SOCKET_ERROR;
                }

                Sleep(SleepWait);
                LingerWait -= SleepWait as LONG;
            }
        }

        if OldState == SocketConnected {
            if LingerWait <= 0 {
                DisconnectInfo.Timeout = RtlConvertLongToLargeInteger(0);
                DisconnectInfo.DisconnectType = if LingerWait < 0 {
                    AFD_DISCONNECT_SEND
                } else {
                    AFD_DISCONNECT_ABORT
                };

                if (((DisconnectInfo.DisconnectType & AFD_DISCONNECT_SEND) != 0) && ((*(*Socket).SharedData).__bindgen_anon_1.SendShutdown() == 0))
                    || (((DisconnectInfo.DisconnectType & AFD_DISCONNECT_ABORT) != 0) && ((*(*Socket).SharedData).__bindgen_anon_1.ReceiveShutdown() == 0))
                {
                    Status = NtDeviceIoControlFile(
                        Handle as HANDLE,
                        SockEvent,
                        None,
                        core::ptr::null_mut(),
                        &mut IoStatusBlock,
                        IOCTL_AFD_DISCONNECT,
                        &mut DisconnectInfo as *mut _ as PVOID,
                        core::mem::size_of::<AFD_DISCONNECT_INFO>() as ULONG,
                        core::ptr::null_mut(),
                        0,
                    );

                    if Status == STATUS_PENDING {
                        MsafdWaitForAlert(SockEvent);
                        Status = IoStatusBlock.__bindgen_anon_1.Status;
                    }
                }
            }
        }

        (*Socket).HelperContext = core::ptr::null_mut();
        (*(*Socket).SharedData).AsyncDisabledEvents = -1i32 as LONG;
        NtClose((*Socket).TdiAddressHandle);
        (*Socket).TdiAddressHandle = core::ptr::null_mut();
        NtClose((*Socket).TdiConnectionHandle);
        (*Socket).TdiConnectionHandle = core::ptr::null_mut();
    }

    EnterCriticalSection(&mut SocketListLock);
    if SocketListHead == Socket {
        SocketListHead = (*SocketListHead).NextSocket;
    } else {
        CurrentSocket = SocketListHead;
        while !(*CurrentSocket).NextSocket.is_null() {
            if (*CurrentSocket).NextSocket == Socket {
                (*CurrentSocket).NextSocket = (*(*CurrentSocket).NextSocket).NextSocket;
                break;
            }

            CurrentSocket = (*CurrentSocket).NextSocket;
        }
    }
    LeaveCriticalSection(&mut SocketListLock);

    NtClose(Handle as HANDLE);
    NtClose(SockEvent);

    if (*Socket).SharedDataHandle != INVALID_HANDLE_VALUE {
        UnmapViewOfFile((*Socket).SharedData as PVOID);
        NtClose((*Socket).SharedDataHandle);
        (*Socket).SharedData = core::ptr::null_mut();
    }
    if (References == 0) && !(*Socket).SharedData.is_null() {
        HeapFree(GlobalHeap, 0, (*Socket).SharedData as LPVOID);
    }
    HeapFree(GlobalHeap, 0, Socket as LPVOID);
    (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32)
}
#[no_mangle]
pub unsafe extern "system" fn WSPBind(
    Handle: SOCKET,
    SocketAddress: *const sockaddr,
    SocketAddressLength: i32,
    lpErrno: LPINT,
) -> INT {
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut BindData: PAFD_BIND_DATA;
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut SocketInfo: SOCKADDR_INFO = core::mem::zeroed();
    let mut SockEvent: HANDLE = core::ptr::null_mut();
    let mut connectSize: ULONG;

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }
    if (*(*Socket).SharedData).State != SocketOpen {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEINVAL as i32);
        }
        return SOCKET_ERROR;
    }
    if SocketAddress.is_null() || SocketAddressLength < (*(*Socket).SharedData).SizeOfLocalAddress as i32 {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEINVAL as i32);
        }
        return SOCKET_ERROR;
    }

    ((*(*Socket).HelperData).WSHGetSockaddrType.unwrap())(
        SocketAddress as PSOCKADDR,
        (SocketAddressLength as u32),
        &mut SocketInfo,
    );

    if SocketInfo.AddressInfo == SockaddrAddressInfoBroadcast && (*(*Socket).SharedData).__bindgen_anon_1.Broadcast() == 0 {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEADDRNOTAVAIL as i32);
        }
        return SOCKET_ERROR;
    }

    Status = NtCreateEvent(
        &mut SockEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    if !NT_SUCCESS(Status) {
        return SOCKET_ERROR;
    }

    // C: FIELD_OFFSET(AFD_BIND_DATA, Address.Address[SocketAddressLength]) — the
    // `[SocketAddressLength]` is a TA_ADDRESS array subscript (scale = sizeof(TA_ADDRESS)),
    // NOT a byte offset. The translator dropped the scaling (`+ SocketAddressLength`), which
    // under-allocates by 2 bytes and heap-overruns the inner Address[] copy below on every
    // bind() — corrupting the process heap. Restore the array-subscript scaling.
    connectSize = (FIELD_OFFSET!(AFD_BIND_DATA, Address.Address)
        + SocketAddressLength as usize * core::mem::size_of::<TA_ADDRESS>()) as ULONG;

    BindData = HeapAlloc(GlobalHeap, 0, (connectSize as usize)) as PAFD_BIND_DATA;
    if BindData.is_null() {
        return (MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, core::ptr::null_mut()) as i32);
    }

    (*BindData).Address.TAAddressCount = 1;
    (*BindData).Address.Address[0].AddressLength = (SocketAddressLength as u16) - (core::mem::size_of::<u16>() as u16);
    (*BindData).Address.Address[0].AddressType = (*SocketAddress).sa_family;
    core::ptr::copy_nonoverlapping(
        (*SocketAddress).sa_data.as_ptr(),
        (*BindData).Address.Address[0].Address.as_mut_ptr() as *mut i8,
        SocketAddressLength as usize - core::mem::size_of::<u16>(),
    );

    if (*(*Socket).SharedData).__bindgen_anon_1.ExclusiveAddressUse() != 0 {
        (*BindData).ShareType = AFD_SHARE_EXCLUSIVE;
    } else if SocketInfo.EndpointInfo == SockaddrEndpointInfoWildcard {
        (*BindData).ShareType = AFD_SHARE_WILDCARD;
    } else if (*(*Socket).SharedData).__bindgen_anon_1.ReuseAddresses() != 0 {
        (*BindData).ShareType = AFD_SHARE_REUSE;
    } else {
        (*BindData).ShareType = AFD_SHARE_UNIQUE;
    }

    Status = NtDeviceIoControlFile(
        (*Socket).Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        &mut IOSB,
        IOCTL_AFD_BIND,
        BindData as PVOID,
        connectSize,
        BindData as PVOID,
        connectSize,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    NtClose(SockEvent);
    HeapFree(GlobalHeap, 0, BindData as LPVOID);

    (*(*Socket).SharedData).SocketLastError = TranslateNtStatusError(Status);
    if Status != STATUS_SUCCESS {
        return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
    }

    (*(*Socket).SharedData).State = SocketBound;
    (*Socket).TdiAddressHandle = IOSB.Information as HANDLE;

    if ((*Socket).HelperEvents & WSH_NOTIFY_BIND) != 0 {
        Status = ((*(*Socket).HelperData).WSHNotify.unwrap())(
            (*Socket).HelperContext,
            (*Socket).Handle,
            (*Socket).TdiAddressHandle,
            (*Socket).TdiConnectionHandle,
            WSH_NOTIFY_BIND,
        );

        if Status != 0 {
            if !lpErrno.is_null() {
                *lpErrno = Status;
            }
            return SOCKET_ERROR;
        }
    }

    return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
}
#[no_mangle]
pub unsafe extern "system" fn WSPListen(
    Handle: SOCKET,
    Backlog: INT,
    lpErrno: LPINT,
) -> INT {
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ListenData: AFD_LISTEN_DATA = core::mem::zeroed();
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let mut SockEvent: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }

    if (*(*Socket).SharedData).__bindgen_anon_1.Listening() != 0 {
        return (NO_ERROR as i32);
    }

    Status = NtCreateEvent(
        &mut SockEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    if !NT_SUCCESS(Status) {
        return SOCKET_ERROR;
    }

    ListenData.UseSAN = 0u8;
    ListenData.UseDelayedAcceptance = (*(*Socket).SharedData).__bindgen_anon_1.UseDelayedAcceptance();
    ListenData.Backlog = Backlog as ULONG;

    Status = NtDeviceIoControlFile(
        (*Socket).Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        &mut IOSB,
        IOCTL_AFD_START_LISTEN,
        &mut ListenData as *mut _ as PVOID,
        core::mem::size_of::<AFD_LISTEN_DATA>() as ULONG,
        core::ptr::null_mut(),
        0,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    NtClose(SockEvent);

    (*(*Socket).SharedData).SocketLastError = TranslateNtStatusError(Status);
    if Status != STATUS_SUCCESS {
        return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
    }

    (*(*Socket).SharedData).__bindgen_anon_1.set_Listening(1u8);

    if (*Socket).HelperEvents & WSH_NOTIFY_LISTEN != 0 {
        Status = ((*(*Socket).HelperData).WSHNotify.unwrap())(
            (*Socket).HelperContext,
            (*Socket).Handle,
            (*Socket).TdiAddressHandle,
            (*Socket).TdiConnectionHandle,
            WSH_NOTIFY_LISTEN,
        );

        if Status != 0 {
            if !lpErrno.is_null() {
                *lpErrno = Status;
            }
            return SOCKET_ERROR;
        }
    }

    return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
}
#[no_mangle]
pub unsafe extern "system" fn WSPSelect(
    nfds: i32,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    exceptfds: *mut fd_set,
    timeout: *const timeval,
    lpErrno: LPINT,
) -> i32 {
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut PollInfo: PAFD_POLL_INFO;
    let mut Status: NTSTATUS;
    let mut HandleCount: ULONG;
    let mut PollBufferSize: ULONG;
    let mut PollBuffer: PVOID;
    let mut i: ULONG;
    let mut j: ULONG = 0;
    let mut x: ULONG;
    let mut SockEvent: HANDLE = core::ptr::null_mut();
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut Socket: PSOCKET_INFORMATION;
    let mut Handle: SOCKET;
    let mut Events: ULONG;
    let mut selectfds: fd_set = core::mem::zeroed();

    FD_ZERO(&mut selectfds);
    if !readfds.is_null() {
        i = 0;
        while i < (*readfds).fd_count {
            FD_SET((*readfds).fd_array[i as usize], &mut selectfds);
            i += 1;
        }
    }
    if !writefds.is_null() {
        i = 0;
        while i < (*writefds).fd_count {
            FD_SET((*writefds).fd_array[i as usize], &mut selectfds);
            i += 1;
        }
    }
    if !exceptfds.is_null() {
        i = 0;
        while i < (*exceptfds).fd_count {
            FD_SET((*exceptfds).fd_array[i as usize], &mut selectfds);
            i += 1;
        }
    }

    HandleCount = selectfds.fd_count;

    if HandleCount == 0 {
        WARN!(b"No handles! Returning SOCKET_ERROR\n\0".as_ptr() as *const i8, HandleCount);
        if !lpErrno.is_null() {
            *lpErrno = (WSAEINVAL as i32);
        }
        return SOCKET_ERROR;
    }

    PollBufferSize = core::mem::size_of::<AFD_POLL_INFO>() as ULONG + ((HandleCount - 1) * core::mem::size_of::<AFD_HANDLE>() as ULONG);

    TRACE!(b"HandleCount: %u BufferSize: %u\n\0".as_ptr() as *const i8, HandleCount, PollBufferSize);

    if timeout.is_null() {
        Timeout.u.LowPart = 0xFFFFFFFFu32;
        Timeout.u.HighPart = 0x7FFFFFFFu32 as i32;
        TRACE!(b"Infinite timeout\n\0".as_ptr() as *const i8);
    } else {
        Timeout = RtlEnlargedIntegerMultiply(
            ((*timeout).tv_sec * 1000) + ((*timeout).tv_usec / 1000),
            -10000,
        );
        if Timeout.QuadPart > 0 {
            if !lpErrno.is_null() {
                *lpErrno = (WSAEINVAL as i32);
            }
            return SOCKET_ERROR;
        }
        TRACE!(
            b"Timeout: Orig %d.%06d kernel %d\n\0".as_ptr() as *const i8,
            (*timeout).tv_sec,
            (*timeout).tv_usec,
            Timeout.u.__bindgen_anon_1.LowPart,
        );
    }

    Status = NtCreateEvent(
        &mut SockEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    if !NT_SUCCESS(Status) {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEFAULT as i32);
        }
        ERR!("NtCreateEvent failed, 0x%08x\n", Status);
        return SOCKET_ERROR;
    }

    PollBuffer = HeapAlloc(GlobalHeap, 0, PollBufferSize as usize);

    if PollBuffer.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEFAULT as i32);
        }
        NtClose(SockEvent);
        return SOCKET_ERROR;
    }

    PollInfo = PollBuffer as PAFD_POLL_INFO;

    RtlZeroMemory(PollInfo as PVOID, PollBufferSize as usize);

    (*PollInfo).Exclusive = (0u8 as _);
    (*PollInfo).Timeout = Timeout;

    // Handles is a C flexible array (`AFD_HANDLE Handles[1]`, drivers/afd/shared.h:59)
    // and PollBuffer above is sized sizeof(AFD_POLL_INFO) + (HandleCount - 1) *
    // sizeof(AFD_HANDLE), so bindgen's [AFD_HANDLE; 1] bounds-checks every walk below
    // and panics on any select() with more than one socket. Constant index 0 is
    // checked at compile time, so `[0]` + .add(n) carries no panic branch.
    i = 0;
    while i < selectfds.fd_count {
        (*(&raw mut (*PollInfo).Handles[0]).add(i as usize)).Handle = selectfds.fd_array[i as usize];
        i += 1;
    }

    if !readfds.is_null() {
        i = 0;
        while i < (*readfds).fd_count {
            j = 0;
            while j < HandleCount {
                if (*(&raw mut (*PollInfo).Handles[0]).add(j as usize)).Handle == (*readfds).fd_array[i as usize] {
                    break;
                }
                j += 1;
            }
            if j >= HandleCount {
                ERR!("Error while counting readfds %ld > %ld\n", j, HandleCount);
                if !lpErrno.is_null() {
                    *lpErrno = (WSAEFAULT as i32);
                }
                HeapFree(GlobalHeap, 0, PollBuffer);
                NtClose(SockEvent);
                return SOCKET_ERROR;
            }
            Socket = GetSocketStructure((*readfds).fd_array[i as usize]);
            if Socket.is_null() {
                ERR!("Invalid socket handle provided in readfds %d\n", (*readfds).fd_array[i as usize]);
                if !lpErrno.is_null() {
                    *lpErrno = (WSAENOTSOCK as i32);
                }
                HeapFree(GlobalHeap, 0, PollBuffer);
                NtClose(SockEvent);
                return SOCKET_ERROR;
            }
            (*(&raw mut (*PollInfo).Handles[0]).add(j as usize)).Events |= AFD_EVENT_RECEIVE | AFD_EVENT_DISCONNECT | AFD_EVENT_ABORT | AFD_EVENT_CLOSE | AFD_EVENT_ACCEPT;
            i += 1;
        }
    }

    if !writefds.is_null() {
        i = 0;
        while i < (*writefds).fd_count {
            j = 0;
            while j < HandleCount {
                if (*(&raw mut (*PollInfo).Handles[0]).add(j as usize)).Handle == (*writefds).fd_array[i as usize] {
                    break;
                }
                j += 1;
            }
            if j >= HandleCount {
                ERR!("Error while counting writefds %ld > %ld\n", j, HandleCount);
                if !lpErrno.is_null() {
                    *lpErrno = (WSAEFAULT as i32);
                }
                HeapFree(GlobalHeap, 0, PollBuffer);
                NtClose(SockEvent);
                return SOCKET_ERROR;
            }
            Socket = GetSocketStructure((*writefds).fd_array[i as usize]);
            if Socket.is_null() {
                ERR!("Invalid socket handle provided in writefds %d\n", (*writefds).fd_array[i as usize]);
                if !lpErrno.is_null() {
                    *lpErrno = (WSAENOTSOCK as i32);
                }
                HeapFree(GlobalHeap, 0, PollBuffer);
                NtClose(SockEvent);
                return SOCKET_ERROR;
            }
            (*(&raw mut (*PollInfo).Handles[0]).add(j as usize)).Handle = (*writefds).fd_array[i as usize];
            (*(&raw mut (*PollInfo).Handles[0]).add(j as usize)).Events |= AFD_EVENT_SEND;
            if (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
                (*(&raw mut (*PollInfo).Handles[0]).add(j as usize)).Events |= AFD_EVENT_CONNECT;
            }
            i += 1;
        }
    }

    if !exceptfds.is_null() {
        i = 0;
        while i < (*exceptfds).fd_count {
            j = 0;
            while j < HandleCount {
                if (*(&raw mut (*PollInfo).Handles[0]).add(j as usize)).Handle == (*exceptfds).fd_array[i as usize] {
                    break;
                }
                j += 1;
            }
            // `>` where the readfds and writefds branches above both use `>=`. When the
            // handle is not found, j == HandleCount, this guard passes, and the writes
            // below index PollInfo.Handles[HandleCount] -- one past the valid range.
            // Found by tools/scan_sibling_drift.py comparing the three branches; NOT by
            // the upstream oracle, because upstream STILL has it (dllmain.c:1279 today,
            // against >= at 1212 and 1247).
            if j >= HandleCount {
                ERR!("Error while counting exceptfds %ld > %ld\n", j, HandleCount);
                if !lpErrno.is_null() {
                    *lpErrno = (WSAEFAULT as i32);
                }
                HeapFree(GlobalHeap, 0, PollBuffer);
                NtClose(SockEvent);
                return SOCKET_ERROR;
            }
            Socket = GetSocketStructure((*exceptfds).fd_array[i as usize]);
            if Socket.is_null() {
                TRACE!(b"Invalid socket handle provided in exceptfds %d\n\0".as_ptr() as *const i8, (*exceptfds).fd_array[i as usize]);
                if !lpErrno.is_null() {
                    *lpErrno = (WSAENOTSOCK as i32);
                }
                HeapFree(GlobalHeap, 0, PollBuffer);
                NtClose(SockEvent);
                return SOCKET_ERROR;
            }
            (*(&raw mut (*PollInfo).Handles[0]).add(j as usize)).Handle = (*exceptfds).fd_array[i as usize];
            if (*(*Socket).SharedData).__bindgen_anon_1.OobInline() == 0 {
                (*(&raw mut (*PollInfo).Handles[0]).add(j as usize)).Events |= AFD_EVENT_OOB_RECEIVE;
            }
            if (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
                (*(&raw mut (*PollInfo).Handles[0]).add(j as usize)).Events |= AFD_EVENT_CONNECT_FAIL;
            }
            i += 1;
        }
    }

    (*PollInfo).HandleCount = HandleCount;
    PollBufferSize = FIELD_OFFSET!(AFD_POLL_INFO, Handles) as ULONG + (*PollInfo).HandleCount * core::mem::size_of::<AFD_HANDLE>() as ULONG;

    Status = NtDeviceIoControlFile(
        (*PollInfo).Handles[0].Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        &mut IOSB,
        IOCTL_AFD_SELECT,
        PollInfo as PVOID,
        PollBufferSize,
        PollInfo as PVOID,
        PollBufferSize,
    );

    TRACE!(b"DeviceIoControlFile => %x\n\0".as_ptr() as *const i8, Status);

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    if !readfds.is_null() {
        FD_ZERO(readfds);
    }
    if !writefds.is_null() {
        FD_ZERO(writefds);
    }
    if !exceptfds.is_null() {
        FD_ZERO(exceptfds);
    }

    HandleCount = (*PollInfo).HandleCount;

    i = 0;
    while i < HandleCount {
        Events = (*(&raw mut (*PollInfo).Handles[0]).add(i as usize)).Events;
        Handle = (*(&raw mut (*PollInfo).Handles[0]).add(i as usize)).Handle;
        x = 1;
        loop {
            if x == 0 {
                break;
            }
            Socket = GetSocketStructure(Handle);
            if Socket.is_null() {
                TRACE!(b"Invalid socket handle found %d\n\0".as_ptr() as *const i8, Handle);
                if !lpErrno.is_null() {
                    *lpErrno = (WSAENOTSOCK as i32);
                }
                HeapFree(GlobalHeap, 0, PollBuffer);
                NtClose(SockEvent);
                return SOCKET_ERROR;
            }
            match Events & x {
                AFD_EVENT_RECEIVE | AFD_EVENT_DISCONNECT | AFD_EVENT_ABORT | AFD_EVENT_ACCEPT | AFD_EVENT_CLOSE => {
                    TRACE!(b"Event %x on handle %x\n\0".as_ptr() as *const i8, Events, Handle);
                    if (Events & x) == AFD_EVENT_DISCONNECT || (Events & x) == AFD_EVENT_CLOSE {
                        (*(*Socket).SharedData).SocketLastError = (WSAECONNRESET as i32);
                    }
                    if (Events & x) == AFD_EVENT_ABORT {
                        (*(*Socket).SharedData).SocketLastError = (WSAECONNABORTED as i32);
                    }
                    if !readfds.is_null() {
                        FD_SET(Handle, readfds);
                    }
                }
                AFD_EVENT_SEND => {
                    TRACE!(b"Event %x on handle %x\n\0".as_ptr() as *const i8, Events, Handle);
                    if !writefds.is_null() {
                        FD_SET(Handle, writefds);
                    }
                }
                AFD_EVENT_CONNECT => {
                    TRACE!(b"Event %x on handle %x\n\0".as_ptr() as *const i8, Events, Handle);
                    if !writefds.is_null() && (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
                        FD_SET(Handle, writefds);
                    }
                }
                AFD_EVENT_OOB_RECEIVE => {
                    TRACE!(b"Event %x on handle %x\n\0".as_ptr() as *const i8, Events, Handle);
                    if !readfds.is_null() && (*(*Socket).SharedData).__bindgen_anon_1.OobInline() != 0 {
                        FD_SET(Handle, readfds);
                    }
                    if !exceptfds.is_null() && (*(*Socket).SharedData).__bindgen_anon_1.OobInline() == 0 {
                        FD_SET(Handle, exceptfds);
                    }
                }
                AFD_EVENT_CONNECT_FAIL => {
                    TRACE!(b"Event %x on handle %x\n\0".as_ptr() as *const i8, Events, Handle);
                    if !exceptfds.is_null() && (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
                        FD_SET(Handle, exceptfds);
                    }
                }
                _ => {}
            }
            x <<= 1;
        }
        i += 1;
    }

    HeapFree(GlobalHeap, 0, PollBuffer);
    NtClose(SockEvent);

    if !lpErrno.is_null() {
        match IOSB.__bindgen_anon_1.Status {
            STATUS_SUCCESS | STATUS_TIMEOUT => {
                *lpErrno = 0;
            }
            _ => {
                *lpErrno = (WSAEINVAL as i32);
            }
        }
        TRACE!(b"*lpErrno = %x\n\0".as_ptr() as *const i8, *lpErrno);
    }

    HandleCount = (if !readfds.is_null() { (*readfds).fd_count } else { 0 })
        + (if !writefds.is_null() && writefds != readfds {
            (*writefds).fd_count
        } else {
            0
        })
        + (if !exceptfds.is_null() && exceptfds != readfds && exceptfds != writefds {
            (*exceptfds).fd_count
        } else {
            0
        });

    TRACE!(b"%d events\n\0".as_ptr() as *const i8, HandleCount);

    HandleCount as i32
}
#[no_mangle]
pub unsafe extern "C" fn GetCurrentTimeInSeconds() -> DWORD {

    let st1970 = SYSTEMTIME {
        wYear: 1970,
        wMonth: 1,
        wDayOfWeek: 0,
        wDay: 1,
        wHour: 0,
        wMinute: 0,
        wSecond: 0,
        wMilliseconds: 0,
    };

    let mut u1970_ft: FILETIME = core::mem::zeroed();
    let mut time_ft: FILETIME = core::mem::zeroed();

    GetSystemTimeAsFileTime(&mut time_ft);
    SystemTimeToFileTime(&st1970, &mut u1970_ft);

    let u1970_ll = ((u1970_ft.dwHighDateTime as u64) << 32) | (u1970_ft.dwLowDateTime as u64);
    let time_ll = ((time_ft.dwHighDateTime as u64) << 32) | (time_ft.dwLowDateTime as u64);

    ((time_ll - u1970_ll) / 10000000u64) as DWORD
}
#[no_mangle]
pub unsafe extern "system" fn WSPAccept(
    Handle: SOCKET,
    SocketAddress: *mut sockaddr,
    SocketAddressLength: LPINT,
    lpfnCondition: LPCONDITIONPROC,
    dwCallbackData: DWORD_PTR,
    lpErrno: LPINT,
) -> SOCKET {
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ListenReceiveData: PAFD_RECEIVED_ACCEPT_DATA;
    let mut AcceptData: AFD_ACCEPT_DATA = core::mem::zeroed();
    let mut DeferData: AFD_DEFER_ACCEPT_DATA = core::mem::zeroed();
    let mut PendingAcceptData: AFD_PENDING_ACCEPT_DATA = core::mem::zeroed();
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ReadSet: fd_set = core::mem::zeroed();
    let mut Timeout: timeval = core::mem::zeroed();
    let mut PendingData: PVOID = core::ptr::null_mut();
    let mut PendingDataLength: ULONG = 0;
    let mut CalleeDataBuffer: PVOID;
    let mut CallerData: WSABUF = core::mem::zeroed();
    let mut CalleeID: WSABUF = core::mem::zeroed();
    let mut CallerID: WSABUF = core::mem::zeroed();
    let mut CalleeData: WSABUF = core::mem::zeroed();
    let mut RemoteAddress: PSOCKADDR = core::ptr::null_mut();
    let mut GroupID: GROUP = 0;
    let mut CallBack: ULONG;
    let mut AcceptSocket: SOCKET;
    let mut AcceptSocketInfo: PSOCKET_INFORMATION;
    let mut ReceiveBuffer: [UCHAR; 0x1A] = core::mem::zeroed();
    let mut SockEvent: HANDLE = core::ptr::null_mut();

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return (SOCKET_ERROR as _);
    }
    if (*(*Socket).SharedData).__bindgen_anon_1.Listening() == 0 {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEINVAL as i32);
        }
        return (SOCKET_ERROR as _);
    }
    if (!SocketAddress.is_null() && SocketAddressLength.is_null())
        || (!SocketAddressLength.is_null() && SocketAddress.is_null())
        || (!SocketAddressLength.is_null() && *SocketAddressLength < core::mem::size_of::<SOCKADDR>() as i32)
    {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEFAULT as i32);
        }
        return INVALID_SOCKET;
    }

    Status = NtCreateEvent(
        core::ptr::addr_of_mut!(SockEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    if !NT_SUCCESS(Status) {
        return (SOCKET_ERROR as _);
    }

    ListenReceiveData = ReceiveBuffer.as_mut_ptr() as PAFD_RECEIVED_ACCEPT_DATA;

    if (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
        FD_ZERO(core::ptr::addr_of_mut!(ReadSet));
        FD_SET((*Socket).Handle, core::ptr::addr_of_mut!(ReadSet));
        Timeout.tv_sec = 0;
        Timeout.tv_usec = 0;

        if WSPSelect(
            0,
            core::ptr::addr_of_mut!(ReadSet),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of!(Timeout),
            lpErrno,
        ) == SOCKET_ERROR
        {
            NtClose(SockEvent);
            return (SOCKET_ERROR as _);
        }

        if (*ReadSet.fd_array.as_ptr().add(0)) != (*Socket).Handle {
            NtClose(SockEvent);
            if !lpErrno.is_null() {
                *lpErrno = (WSAEWOULDBLOCK as i32);
            }
            return (SOCKET_ERROR as _);
        }
    }

    Status = NtDeviceIoControlFile(
        (*Socket).Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IOSB),
        IOCTL_AFD_WAIT_FOR_LISTEN,
        core::ptr::null_mut(),
        0,
        ListenReceiveData as PVOID,
        0xA + core::mem::size_of::<AFD_RECEIVED_ACCEPT_DATA>() as u32,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    if !NT_SUCCESS(Status) {
        NtClose(SockEvent);
        return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as _;
    }

    if lpfnCondition.is_some() {
        if ((*(*Socket).SharedData).ServiceFlags1 & XP1_CONNECT_DATA) != 0 {
            PendingAcceptData.SequenceNumber = (*ListenReceiveData).SequenceNumber;
            PendingAcceptData.ReturnSize = (1u8 as u32);

            Status = NtDeviceIoControlFile(
                (*Socket).Handle as HANDLE,
                SockEvent,
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(IOSB),
                IOCTL_AFD_GET_PENDING_CONNECT_DATA,
                core::ptr::addr_of_mut!(PendingAcceptData) as PVOID,
                core::mem::size_of::<AFD_PENDING_ACCEPT_DATA>() as u32,
                core::ptr::addr_of_mut!(PendingAcceptData) as PVOID,
                core::mem::size_of::<AFD_PENDING_ACCEPT_DATA>() as u32,
            );

            if Status == STATUS_PENDING {
                MsafdWaitForAlert(SockEvent);
                Status = IOSB.__bindgen_anon_1.Status;
            }

            if !NT_SUCCESS(Status) {
                NtClose(SockEvent);
                return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as _;
            }

            PendingDataLength = IOSB.Information as ULONG;

            if PendingDataLength != 0 {
                PendingData = HeapAlloc(GlobalHeap, 0, PendingDataLength as usize);
                if PendingData.is_null() {
                    return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, core::ptr::null_mut()) as _;
                }

                PendingAcceptData.ReturnSize = (0u8 as u32);

                Status = NtDeviceIoControlFile(
                    (*Socket).Handle as HANDLE,
                    SockEvent,
                    None,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(IOSB),
                    IOCTL_AFD_GET_PENDING_CONNECT_DATA,
                    core::ptr::addr_of_mut!(PendingAcceptData) as PVOID,
                    core::mem::size_of::<AFD_PENDING_ACCEPT_DATA>() as u32,
                    PendingData,
                    PendingDataLength,
                );

                if Status == STATUS_PENDING {
                    MsafdWaitForAlert(SockEvent);
                    Status = IOSB.__bindgen_anon_1.Status;
                }

                if !NT_SUCCESS(Status) {
                    NtClose(SockEvent);
                    return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as _;
                }
            }
        }

        if ((*(*Socket).SharedData).ServiceFlags1 & XP1_QOS_SUPPORTED) != 0 {
        }

        CalleeID.buf = (*Socket).LocalAddress as *mut i8;
        CalleeID.len = ((*(*Socket).SharedData).SizeOfLocalAddress as u32);

        RemoteAddress = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<SOCKADDR>()) as PSOCKADDR;
        if RemoteAddress.is_null() {
            return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, core::ptr::null_mut()) as _;
        }

        RtlCopyMemory(
            RemoteAddress as PVOID,
            core::ptr::addr_of!((*(*ListenReceiveData).Address.Address.as_ptr().add(0)).AddressType) as PVOID,
            core::mem::size_of::<SOCKADDR>(),
        );

        CallerID.buf = RemoteAddress as *mut i8;
        CallerID.len = core::mem::size_of::<SOCKADDR>() as u32;

        CallerData.buf = PendingData as *mut i8;
        CallerData.len = PendingDataLength;

        if (*(*Socket).SharedData).__bindgen_anon_1.UseDelayedAcceptance() != 0 {
            CalleeDataBuffer = HeapAlloc(GlobalHeap, 0, 4096);
            if CalleeDataBuffer.is_null() {
                return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, core::ptr::null_mut()) as _;
            }
            CalleeData.buf = CalleeDataBuffer as *mut i8;
            CalleeData.len = 4096;
        } else {
            CalleeData.buf = core::ptr::null_mut();
            CalleeData.len = 0;
        }

        CallBack = (lpfnCondition.unwrap()(
            core::ptr::addr_of_mut!(CallerID),
            if CallerData.buf.is_null() { core::ptr::null_mut() } else { core::ptr::addr_of_mut!(CallerData) },
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(CalleeID),
            if CalleeData.buf.is_null() { core::ptr::null_mut() } else { core::ptr::addr_of_mut!(CalleeData) },
            core::ptr::addr_of_mut!(GroupID),
            dwCallbackData,
        ) as u32);

        if ((CallBack == CF_ACCEPT) && GroupID != 0) {
        }

        if CallBack == CF_ACCEPT {
            if ((*(*Socket).SharedData).ServiceFlags1 & XP1_QOS_SUPPORTED) != 0 {
            }
            if !CalleeData.buf.is_null() {
            }
        } else {
            DeferData.SequenceNumber = (*ListenReceiveData).SequenceNumber;
            DeferData.RejectConnection = if CallBack == CF_REJECT { 1u8 } else { 0u8 };

            Status = NtDeviceIoControlFile(
                (*Socket).Handle as HANDLE,
                SockEvent,
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(IOSB),
                IOCTL_AFD_DEFER_ACCEPT,
                core::ptr::addr_of_mut!(DeferData) as PVOID,
                core::mem::size_of::<AFD_DEFER_ACCEPT_DATA>() as u32,
                core::ptr::null_mut(),
                0,
            );

            if Status == STATUS_PENDING {
                MsafdWaitForAlert(SockEvent);
                Status = IOSB.__bindgen_anon_1.Status;
            }

            NtClose(SockEvent);

            if !NT_SUCCESS(Status) {
                return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as _;
            }

            if CallBack == CF_REJECT {
                if !lpErrno.is_null() {
                    *lpErrno = (WSAECONNREFUSED as i32);
                }
                return (SOCKET_ERROR as _);
            } else {
                if !lpErrno.is_null() {
                    *lpErrno = (WSAECONNREFUSED as i32);
                }
                return (SOCKET_ERROR as _);
            }
        }
    }

    AcceptSocket = WSPSocket(
        (*(*Socket).SharedData).AddressFamily,
        (*(*Socket).SharedData).SocketType,
        (*(*Socket).SharedData).Protocol,
        core::ptr::addr_of_mut!((*Socket).ProtocolInfo),
        GroupID,
        (*(*Socket).SharedData).CreateFlags,
        lpErrno,
    );
    if AcceptSocket == INVALID_SOCKET {
        return (SOCKET_ERROR as _);
    }

    AcceptData.ListenHandle = AcceptSocket as HANDLE;
    AcceptData.SequenceNumber = (*ListenReceiveData).SequenceNumber;

    Status = NtDeviceIoControlFile(
        (*Socket).Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IOSB),
        IOCTL_AFD_ACCEPT,
        core::ptr::addr_of_mut!(AcceptData) as PVOID,
        core::mem::size_of::<AFD_ACCEPT_DATA>() as u32,
        core::ptr::null_mut(),
        0,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    (*(*Socket).SharedData).SocketLastError = TranslateNtStatusError(Status);
    if !NT_SUCCESS(Status) {
        NtClose(SockEvent);
        WSPCloseSocket(AcceptSocket, lpErrno);
        return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as _;
    }

    AcceptSocketInfo = GetSocketStructure(AcceptSocket);
    if AcceptSocketInfo.is_null() {
        NtClose(SockEvent);
        WSPCloseSocket(AcceptSocket, lpErrno);
        return MsafdReturnWithErrno(STATUS_PROTOCOL_NOT_SUPPORTED, lpErrno, 0, core::ptr::null_mut()) as _;
    }

    (*(*AcceptSocketInfo).SharedData).State = SocketConnected;
    (*(*AcceptSocketInfo).SharedData).ConnectTime = GetCurrentTimeInSeconds();
    (*(*AcceptSocketInfo).SharedData).__bindgen_anon_1.set_NonBlocking((*(*Socket).SharedData).__bindgen_anon_1.NonBlocking());

    if !SocketAddress.is_null() {
        RtlCopyMemory(
            SocketAddress as PVOID,
            core::ptr::addr_of!((*(*ListenReceiveData).Address.Address.as_ptr().add(0)).AddressType) as PVOID,
            core::mem::size_of::<SOCKADDR>(),
        );
        if !SocketAddressLength.is_null() {
            *SocketAddressLength = core::mem::size_of::<SOCKADDR>() as i32;
        }
    }

    NtClose(SockEvent);

    SockReenableAsyncSelectEvent(Socket, FD_ACCEPT);

    TRACE!(b"Socket %x\n\0".as_ptr() as PCHAR, AcceptSocket);

    if Status == STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_ACCEPT) != 0 {
        Status = ((*(*Socket).HelperData).WSHNotify.unwrap())(
            (*Socket).HelperContext,
            (*Socket).Handle,
            (*Socket).TdiAddressHandle,
            (*Socket).TdiConnectionHandle,
            WSH_NOTIFY_ACCEPT,
        );

        if Status != 0 {
            if !lpErrno.is_null() {
                *lpErrno = Status;
            }
            return (SOCKET_ERROR as _);
        }
    }

    if !lpErrno.is_null() {
        *lpErrno = (NO_ERROR as i32);
    }

    return AcceptSocket;
}

#[no_mangle]
pub unsafe extern "system" fn WSPAcceptEx(
    sListenSocket: SOCKET,
    sAcceptSocket: SOCKET,
    lpOutputBuffer: PVOID,
    dwReceiveDataLength: DWORD,
    dwLocalAddressLength: DWORD,
    dwRemoteAddressLength: DWORD,
    lpdwBytesReceived: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let ListenSocket = GetSocketStructure(sListenSocket);
    let AcceptSocket = GetSocketStructure(sAcceptSocket);
    let IoStatusBlock = lpOverlapped as PIO_STATUS_BLOCK;

    /* Validate sockets */
    if ListenSocket.is_null() || AcceptSocket.is_null() {
        SetLastError(WSAENOTSOCK);
        return 0;
    }
    if lpOverlapped.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    /* Check if this socket is listening and that AcceptEx hasn't already been invoked */
    if (*(*ListenSocket).SharedData).__bindgen_anon_1.Listening() == 0 {
        SetLastError(WSAEINVAL);
        return 0;
    }
    /* Make sure that the accept socket is not already opened or connected */
    if (*(*AcceptSocket).SharedData).State != SocketOpen {
        SetLastError(WSAEINVAL);
        return 0;
    }
    if lpOutputBuffer.is_null() || dwRemoteAddressLength == 0 {
        SetLastError(WSAEFAULT);
        return 0;
    }

    /* Populate IOCTL super accept structure */
    let mut AcceptInfo = AFD_SUPER_ACCEPT_INFO {
        SanActive: 0,
        FixAddressAlignment: 1,
        AcceptHandle: (*AcceptSocket).Handle as HANDLE,
        ReceiveDataLength: dwReceiveDataLength,
        LocalAddressLength: dwLocalAddressLength,
        RemoteAddressLength: dwRemoteAddressLength,
    };

    (*IoStatusBlock).Information = 0;
    (*IoStatusBlock).__bindgen_anon_1.Status = STATUS_PENDING;

    let Status = NtDeviceIoControlFile(
        (*ListenSocket).Handle as HANDLE,
        (*lpOverlapped).hEvent,
        None,
        if !(*lpOverlapped).hEvent.is_null() {
            core::ptr::null_mut()
        } else {
            lpOverlapped as PVOID
        },
        IoStatusBlock,
        IOCTL_AFD_SUPER_ACCEPT,
        &raw mut AcceptInfo as PVOID,
        core::mem::size_of::<AFD_SUPER_ACCEPT_INFO>() as ULONG,
        lpOutputBuffer,
        dwReceiveDataLength
            .wrapping_add(dwLocalAddressLength)
            .wrapping_add(dwRemoteAddressLength),
    );

    if Status == STATUS_PENDING {
        SetLastError(ERROR_IO_PENDING);
        return 0;
    }
    if NT_SUCCESS(Status) && !lpdwBytesReceived.is_null() {
        *lpdwBytesReceived = (*IoStatusBlock).Information as DWORD;
    }
    SetLastError(TranslateNtStatusError(Status) as u32);
    NT_SUCCESS(Status) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn WSPGetAcceptExSockaddrs(
    lpOutputBuffer: PVOID,
    dwReceiveDataLength: DWORD,
    dwLocalAddressLength: DWORD,
    dwRemoteAddressLength: DWORD,
    LocalSockaddr: *mut *mut sockaddr,
    LocalSockaddrLength: LPINT,
    RemoteSockaddr: *mut *mut sockaddr,
    RemoteSockaddrLength: LPINT,
) {
    /* AFD_SUPER_ACCEPT outputs receive data + local address + local address length + remote
     * address + remote address length; the length is located 2 bytes before the address length.
     * The C reads it as an unaligned UINT. */
    let LocalAddress = (lpOutputBuffer as *mut u8).add(dwReceiveDataLength as usize);
    if dwLocalAddressLength != 0 {
        *LocalSockaddr = LocalAddress as *mut sockaddr;
        *LocalSockaddrLength = (LocalAddress
            .add(dwLocalAddressLength as usize - core::mem::size_of::<WORD>())
            as *const u32)
            .read_unaligned() as INT;
    }
    if dwRemoteAddressLength != 0 {
        let RemoteAddress = LocalAddress.add(dwLocalAddressLength as usize);
        *RemoteSockaddr = RemoteAddress as *mut sockaddr;
        *RemoteSockaddrLength = (RemoteAddress
            .add(dwRemoteAddressLength as usize - core::mem::size_of::<WORD>())
            as *const u32)
            .read_unaligned() as INT;
    }
}

unsafe extern "system" fn MsafdConnectAPC(
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Reserved: ULONG,
) {
    let Context = ApcContext as PMSAFD_CONNECT_APC_CONTEXT;

    let Socket = GetSocketStructure((*Context).lpSocket);
    if Socket.is_null() {
        HeapFree(GlobalHeap, 0, ApcContext);
        return;
    }

    (*(*Socket).SharedData).SocketLastError = TranslateNtStatusError((*IoStatusBlock).__bindgen_anon_1.Status);
    if (*IoStatusBlock).__bindgen_anon_1.Status == STATUS_SUCCESS {
        (*(*Socket).SharedData).State = SocketConnected;
        (*Socket).TdiConnectionHandle = (*IoStatusBlock).Information as HANDLE;
        (*(*Socket).SharedData).ConnectTime = GetCurrentTimeInSeconds();
    }

    SockReenableAsyncSelectEvent(Socket, FD_WRITE);

    SockReenableAsyncSelectEvent(Socket, FD_CONNECT);

    if (*IoStatusBlock).__bindgen_anon_1.Status == STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT) != 0 {
        ((*(*Socket).HelperData).WSHNotify.unwrap())(
            (*Socket).HelperContext,
            (*Socket).Handle,
            (*Socket).TdiAddressHandle,
            (*Socket).TdiConnectionHandle,
            WSH_NOTIFY_CONNECT,
        );
    } else if (*IoStatusBlock).__bindgen_anon_1.Status != STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT_ERROR) != 0 {
        ((*(*Socket).HelperData).WSHNotify.unwrap())(
            (*Socket).HelperContext,
            (*Socket).Handle,
            (*Socket).TdiAddressHandle,
            (*Socket).TdiConnectionHandle,
            WSH_NOTIFY_CONNECT_ERROR,
        );
    }

    HeapFree(GlobalHeap, 0, ApcContext);
}
#[no_mangle]
pub unsafe extern "system" fn WSPConnect(
    Handle: SOCKET,
    SocketAddress: *const sockaddr,
    SocketAddressLength: i32,
    lpCallerData: LPWSABUF,
    lpCalleeData: LPWSABUF,
    lpSQOS: LPQOS,
    lpGQOS: LPQOS,
    lpErrno: LPINT,
) -> INT {
    let mut DummyIOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut IOSB: PIO_STATUS_BLOCK = &mut DummyIOSB;
    let ConnectInfo: PAFD_CONNECT_INFO;
    let Socket: PSOCKET_INFORMATION;
    let mut Status: NTSTATUS;
    let mut Errno: INT;
    let mut ConnectDataLength: ULONG;
    let InConnectDataLength: ULONG;
    let mut SockEvent: HANDLE = core::ptr::null_mut();
    let SocketDataLength: i32;
    let mut APCContext: PMSAFD_CONNECT_APC_CONTEXT = core::ptr::null_mut();
    let mut APCFunction: PIO_APC_ROUTINE = None;
    let mut Buffer: [UCHAR; 128] = [0; 128];

    TRACE!(b"WSPConnect(%x)\n\0".as_ptr() as *const i8, Handle);

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }

    if (*(*Socket).SharedData).State == SocketOpen {
        let BindAddressLength: INT;
        let BindAddress: PSOCKADDR;
        let BindError: INT;

        BindAddressLength = (*(*Socket).HelperData).MaxWSAddressLength;
        BindAddress = HeapAlloc(GetProcessHeap(), 0, BindAddressLength as usize) as PSOCKADDR;
        if BindAddress.is_null() {
            return (MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, core::ptr::null_mut()) as i32);
        }
        let mut BindAddrLen = BindAddressLength;
        ((*(*Socket).HelperData).WSHGetWildcardSockaddr.unwrap())(
            (*Socket).HelperContext,
            BindAddress,
            &mut BindAddrLen,
        );
        BindError = WSPBind(Handle, BindAddress, BindAddressLength, lpErrno);
        HeapFree(GetProcessHeap(), 0, BindAddress as LPVOID);
        if BindError == SOCKET_ERROR {
            return SOCKET_ERROR;
        }
    }

    Status = NtCreateEvent(
        &mut SockEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        return SOCKET_ERROR;
    }

    if !lpCallerData.is_null() {
        ConnectDataLength = (*lpCallerData).len;
        Status = NtDeviceIoControlFile(
            Handle as HANDLE,
            SockEvent,
            None,
            core::ptr::null_mut(),
            IOSB,
            IOCTL_AFD_SET_CONNECT_DATA,
            (*lpCallerData).buf as PVOID,
            ConnectDataLength,
            core::ptr::null_mut(),
            0,
        );
        if Status == STATUS_PENDING {
            MsafdWaitForAlert(SockEvent);
            Status = (*IOSB).__bindgen_anon_1.Status;
        }

        if Status != STATUS_SUCCESS {
            'Leave: {
                TRACE!(b"Ending %lx\n\0".as_ptr() as *const i8, Status);

                NtClose(SockEvent);

                SockReenableAsyncSelectEvent(Socket, FD_WRITE);

                SockReenableAsyncSelectEvent(Socket, FD_CONNECT);

                if Status == STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT != 0) {
                    Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                        (*Socket).HelperContext,
                        (*Socket).Handle,
                        (*Socket).TdiAddressHandle,
                        (*Socket).TdiConnectionHandle,
                        WSH_NOTIFY_CONNECT,
                    );

                    if Errno != 0 {
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return SOCKET_ERROR;
                    }
                } else if Status != STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT_ERROR != 0) {
                    Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                        (*Socket).HelperContext,
                        (*Socket).Handle,
                        (*Socket).TdiAddressHandle,
                        (*Socket).TdiConnectionHandle,
                        WSH_NOTIFY_CONNECT_ERROR,
                    );

                    if Errno != 0 {
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return SOCKET_ERROR;
                    }
                }

                return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
            }
        }
    }

    SocketDataLength = SocketAddressLength - FIELD_OFFSET!(sockaddr, sa_data) as i32;

    ConnectInfo = Buffer.as_mut_ptr() as PAFD_CONNECT_INFO;

    let connectionInfoSize = FIELD_OFFSET!(_AFD_CONNECT_INFO, RemoteAddress) 
        + FIELD_OFFSET!(TA_ADDRESS, Address) 
        + core::mem::size_of::<TA_ADDRESS>() 
        + SocketDataLength as usize;

    if connectionInfoSize > 128 {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEFAULT as i32);
        }
        return SOCKET_ERROR;
    }

    (*ConnectInfo).RemoteAddress.TAAddressCount = 1;
    (*ConnectInfo).RemoteAddress.Address[0].AddressLength = SocketDataLength as USHORT;
    (*ConnectInfo).RemoteAddress.Address[0].AddressType = (*SocketAddress).sa_family;
    core::ptr::copy_nonoverlapping(
        (*SocketAddress).sa_data.as_ptr(),
        (*ConnectInfo).RemoteAddress.Address[0].Address.as_mut_ptr() as *mut i8,
        SocketDataLength as usize,
    );

    if (*(*Socket).SharedData).AsyncEvents & (FD_CONNECT as i32) != 0 {
        (*(*Socket).SharedData).AsyncDisabledEvents |= (FD_CONNECT | FD_WRITE) as i32;
    }

    if !lpCalleeData.is_null() {
        InConnectDataLength = (*lpCalleeData).len;
        Status = NtDeviceIoControlFile(
            Handle as HANDLE,
            SockEvent,
            None,
            core::ptr::null_mut(),
            IOSB,
            IOCTL_AFD_SET_CONNECT_DATA_SIZE,
            &InConnectDataLength as *const ULONG as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
            core::ptr::null_mut(),
            0,
        );

        if Status == STATUS_PENDING {
            MsafdWaitForAlert(SockEvent);
            Status = (*IOSB).__bindgen_anon_1.Status;
        }

        if Status != STATUS_SUCCESS {
            'Leave: {
                TRACE!(b"Ending %lx\n\0".as_ptr() as *const i8, Status);

                NtClose(SockEvent);

                SockReenableAsyncSelectEvent(Socket, FD_WRITE);

                SockReenableAsyncSelectEvent(Socket, FD_CONNECT);

                if Status == STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT != 0) {
                    Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                        (*Socket).HelperContext,
                        (*Socket).Handle,
                        (*Socket).TdiAddressHandle,
                        (*Socket).TdiConnectionHandle,
                        WSH_NOTIFY_CONNECT,
                    );

                    if Errno != 0 {
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return SOCKET_ERROR;
                    }
                } else if Status != STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT_ERROR != 0) {
                    Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                        (*Socket).HelperContext,
                        (*Socket).Handle,
                        (*Socket).TdiAddressHandle,
                        (*Socket).TdiConnectionHandle,
                        WSH_NOTIFY_CONNECT_ERROR,
                    );

                    if Errno != 0 {
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return SOCKET_ERROR;
                    }
                }

                return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
            }
        }
    }

    (*ConnectInfo).Root = 0;
    (*ConnectInfo).UseSAN = 0u8;
    (*ConnectInfo).Unknown = 0;

    if (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
        APCContext = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<MSAFD_CONNECT_APC_CONTEXT>()) as PMSAFD_CONNECT_APC_CONTEXT;
        if APCContext.is_null() {
            ERR!("Not enough memory for APC Context\n");
            Status = STATUS_INSUFFICIENT_RESOURCES;
            'Leave: {
                TRACE!(b"Ending %lx\n\0".as_ptr() as *const i8, Status);

                NtClose(SockEvent);

                SockReenableAsyncSelectEvent(Socket, FD_WRITE);

                SockReenableAsyncSelectEvent(Socket, FD_CONNECT);

                if Status == STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT != 0) {
                    Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                        (*Socket).HelperContext,
                        (*Socket).Handle,
                        (*Socket).TdiAddressHandle,
                        (*Socket).TdiConnectionHandle,
                        WSH_NOTIFY_CONNECT,
                    );

                    if Errno != 0 {
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return SOCKET_ERROR;
                    }
                } else if Status != STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT_ERROR != 0) {
                    Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                        (*Socket).HelperContext,
                        (*Socket).Handle,
                        (*Socket).TdiAddressHandle,
                        (*Socket).TdiConnectionHandle,
                        WSH_NOTIFY_CONNECT_ERROR,
                    );

                    if Errno != 0 {
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return SOCKET_ERROR;
                    }
                }

                return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
            }
        }
        (*APCContext).lpSocket = Handle;
        APCFunction = as_apc(MsafdConnectAPC);

        IOSB = &mut (*APCContext).IoStatusBlock;
    }

    (*IOSB).__bindgen_anon_1.Status = STATUS_PENDING;
    (*IOSB).Information = 0;

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        if APCFunction.is_some() { core::ptr::null_mut() } else { SockEvent },
        APCFunction,
        APCContext as PVOID,
        IOSB,
        IOCTL_AFD_CONNECT,
        ConnectInfo as PVOID,
        connectionInfoSize as ULONG,
        core::ptr::null_mut(),
        0,
    );

    if (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
        if Status == STATUS_PENDING {
            Status = STATUS_CANT_WAIT;
            'Leave: {
                TRACE!(b"Ending %lx\n\0".as_ptr() as *const i8, Status);

                NtClose(SockEvent);

                SockReenableAsyncSelectEvent(Socket, FD_WRITE);

                SockReenableAsyncSelectEvent(Socket, FD_CONNECT);

                if Status == STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT != 0) {
                    Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                        (*Socket).HelperContext,
                        (*Socket).Handle,
                        (*Socket).TdiAddressHandle,
                        (*Socket).TdiConnectionHandle,
                        WSH_NOTIFY_CONNECT,
                    );

                    if Errno != 0 {
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return SOCKET_ERROR;
                    }
                } else if Status != STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT_ERROR != 0) {
                    Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                        (*Socket).HelperContext,
                        (*Socket).Handle,
                        (*Socket).TdiAddressHandle,
                        (*Socket).TdiConnectionHandle,
                        WSH_NOTIFY_CONNECT_ERROR,
                    );

                    if Errno != 0 {
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return SOCKET_ERROR;
                    }
                }

                return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
            }
        } else {
            SleepEx(0, 1);
        }
    } else {
        if Status == STATUS_PENDING {
            MsafdWaitForAlert(SockEvent);
            Status = (*IOSB).__bindgen_anon_1.Status;
        }

        (*(*Socket).SharedData).SocketLastError = TranslateNtStatusError(Status);
        if Status != STATUS_SUCCESS {
            'Leave: {
                TRACE!(b"Ending %lx\n\0".as_ptr() as *const i8, Status);

                NtClose(SockEvent);

                SockReenableAsyncSelectEvent(Socket, FD_WRITE);

                SockReenableAsyncSelectEvent(Socket, FD_CONNECT);

                if Status == STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT != 0) {
                    Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                        (*Socket).HelperContext,
                        (*Socket).Handle,
                        (*Socket).TdiAddressHandle,
                        (*Socket).TdiConnectionHandle,
                        WSH_NOTIFY_CONNECT,
                    );

                    if Errno != 0 {
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return SOCKET_ERROR;
                    }
                } else if Status != STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT_ERROR != 0) {
                    Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                        (*Socket).HelperContext,
                        (*Socket).Handle,
                        (*Socket).TdiAddressHandle,
                        (*Socket).TdiConnectionHandle,
                        WSH_NOTIFY_CONNECT_ERROR,
                    );

                    if Errno != 0 {
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return SOCKET_ERROR;
                    }
                }

                return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
            }
        }

        (*(*Socket).SharedData).State = SocketConnected;
        (*Socket).TdiConnectionHandle = (*IOSB).Information as HANDLE;
        (*(*Socket).SharedData).ConnectTime = GetCurrentTimeInSeconds();
    }

    if !lpCalleeData.is_null() && Status == STATUS_SUCCESS {
        IOSB = &mut DummyIOSB;
        Status = NtDeviceIoControlFile(
            Handle as HANDLE,
            SockEvent,
            None,
            core::ptr::null_mut(),
            IOSB,
            IOCTL_AFD_GET_CONNECT_DATA,
            core::ptr::null_mut(),
            0,
            (*lpCalleeData).buf as PVOID,
            (*lpCalleeData).len,
        );
        if Status == STATUS_PENDING {
            MsafdWaitForAlert(SockEvent);
            Status = (*IOSB).__bindgen_anon_1.Status;
        }
    }

    'Leave: {
        TRACE!(b"Ending %lx\n\0".as_ptr() as *const i8, Status);

        NtClose(SockEvent);

        SockReenableAsyncSelectEvent(Socket, FD_WRITE);

        SockReenableAsyncSelectEvent(Socket, FD_CONNECT);

        if Status == STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT != 0) {
            Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                (*Socket).HelperContext,
                (*Socket).Handle,
                (*Socket).TdiAddressHandle,
                (*Socket).TdiConnectionHandle,
                WSH_NOTIFY_CONNECT,
            );

            if Errno != 0 {
                if !lpErrno.is_null() {
                    *lpErrno = Errno;
                }
                return SOCKET_ERROR;
            }
        } else if Status != STATUS_SUCCESS && ((*Socket).HelperEvents & WSH_NOTIFY_CONNECT_ERROR != 0) {
            Errno = ((*(*Socket).HelperData).WSHNotify.unwrap())(
                (*Socket).HelperContext,
                (*Socket).Handle,
                (*Socket).TdiAddressHandle,
                (*Socket).TdiConnectionHandle,
                WSH_NOTIFY_CONNECT_ERROR,
            );

            if Errno != 0 {
                if !lpErrno.is_null() {
                    *lpErrno = Errno;
                }
                return SOCKET_ERROR;
            }
        }

        return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
    }
}
#[no_mangle]
pub unsafe extern "system" fn WSPConnectEx(
    Handle: SOCKET,
    SocketAddress: *const sockaddr,
    SocketAddressLength: INT,
    lpSendBuffer: PVOID,
    dwSendDataLength: DWORD,
    lpdwBytesSent: LPDWORD,
    lpOverlapped: LPOVERLAPPED,
) -> BOOL {
    let mut DummyIOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut IOSB: PIO_STATUS_BLOCK = &mut DummyIOSB;
    let mut ConnectInfo: PAFD_SUPER_CONNECT_INFO = core::ptr::null_mut();
    let Socket: PSOCKET_INFORMATION;
    let Status: NTSTATUS;
    let SocketDataLength: INT;
    let mut Buffer: [u8; 128] = [0; 128];

    if lpOverlapped.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if lpSendBuffer.is_null() && dwSendDataLength != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        SetLastError(WSAENOTSOCK);
        return 0;
    }

    if SocketAddressLength > (128 - core::mem::size_of::<AFD_CONNECT_INFO>() as INT)
        || ((*(*Socket).SharedData).ServiceFlags1 & XP1_CONNECTIONLESS) != 0
    {
        SetLastError(WSAEFAULT);
        return SOCKET_ERROR as BOOL;
    }

    if (*(*Socket).SharedData).State == SocketOpen {
        SetLastError(WSAEINVAL);
        return 0;
    }

    ConnectInfo = Buffer.as_mut_ptr() as PAFD_SUPER_CONNECT_INFO;
    (*ConnectInfo).SanActive = 0;

    SocketDataLength = SocketAddressLength - FIELD_OFFSET!(sockaddr, sa_data) as INT;

    (*ConnectInfo).RemoteAddress.TAAddressCount = 1;
    (*ConnectInfo).RemoteAddress.Address[0].AddressLength = SocketDataLength as USHORT;
    (*ConnectInfo).RemoteAddress.Address[0].AddressType = (*SocketAddress).sa_family;
    core::ptr::copy_nonoverlapping(
        (*SocketAddress).sa_data.as_ptr() as *const u8,
        (*ConnectInfo).RemoteAddress.Address[0].Address.as_mut_ptr(),
        SocketDataLength as usize,
    );

    if ((*(*Socket).SharedData).AsyncEvents & (FD_CONNECT as i32)) != 0 {
        (*(*Socket).SharedData).AsyncDisabledEvents |= (FD_CONNECT | FD_WRITE) as i32;
    }

    IOSB = lpOverlapped as PIO_STATUS_BLOCK;
    (*IOSB).__bindgen_anon_1.Status = STATUS_PENDING;

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        (*lpOverlapped).hEvent,
        None,
        if !(*lpOverlapped).hEvent.is_null() {
            core::ptr::null_mut()
        } else {
            lpOverlapped as PVOID
        },
        IOSB,
        IOCTL_AFD_SUPER_CONNECT,
        ConnectInfo as PVOID,
        (FIELD_OFFSET!(AFD_SUPER_CONNECT_INFO, RemoteAddress)
            + FIELD_OFFSET!(TA_IP_ADDRESS, Address)
            + FIELD_OFFSET!(TA_ADDRESS, Address)
            + SocketDataLength as usize) as ULONG,
        lpSendBuffer,
        dwSendDataLength,
    );

    SetLastError((TranslateNtStatusError(Status) as u32));

    if Status == STATUS_SUCCESS {
        1
    } else {
        0
    }
}
#[no_mangle]
pub unsafe extern "system" fn WSPShutdown(
    Handle: SOCKET,
    HowTo: INT,
    lpErrno: LPINT,
) -> INT {
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut DisconnectInfo: AFD_DISCONNECT_INFO = core::mem::zeroed();
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut SockEvent: HANDLE = core::ptr::null_mut();

    TRACE!(b"Called\n\0".as_ptr() as *const i8);

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }

    Status = NtCreateEvent(
        core::ptr::addr_of_mut!(SockEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    if !NT_SUCCESS(Status) {
        return SOCKET_ERROR;
    }

    match HowTo as u32 {
        SD_RECEIVE => {
            DisconnectInfo.DisconnectType = AFD_DISCONNECT_RECV;
            (*(*Socket).SharedData).__bindgen_anon_1.set_ReceiveShutdown(1u8);
        }
        SD_SEND => {
            DisconnectInfo.DisconnectType = AFD_DISCONNECT_SEND;
            (*(*Socket).SharedData).__bindgen_anon_1.set_SendShutdown(1u8);
        }
        SD_BOTH => {
            DisconnectInfo.DisconnectType = AFD_DISCONNECT_RECV | AFD_DISCONNECT_SEND;
            (*(*Socket).SharedData).__bindgen_anon_1.set_ReceiveShutdown(1u8);
            (*(*Socket).SharedData).__bindgen_anon_1.set_SendShutdown(1u8);
        }
        _ => {}
    }

    DisconnectInfo.Timeout = RtlConvertLongToLargeInteger(-1000000);

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IOSB),
        IOCTL_AFD_DISCONNECT,
        core::ptr::addr_of_mut!(DisconnectInfo) as PVOID,
        core::mem::size_of::<AFD_DISCONNECT_INFO>() as u32,
        core::ptr::null_mut(),
        0,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    TRACE!(b"Ending\n\0".as_ptr() as *const i8);

    NtClose(SockEvent);

    (*(*Socket).SharedData).SocketLastError = TranslateNtStatusError(Status);
    (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32)
}
#[no_mangle]
pub unsafe extern "system" fn WSPGetSockName(
    Handle: SOCKET,
    Name: LPSOCKADDR,
    NameLength: LPINT,
    lpErrno: LPINT,
) -> INT {
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let TdiAddressSize: ULONG;
    let TdiAddress: PTDI_ADDRESS_INFO;
    let SocketAddress: PTRANSPORT_ADDRESS;
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut SockEvent: HANDLE = core::ptr::null_mut();

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }

    if Name.is_null() || NameLength.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEFAULT as i32);
        }
        return SOCKET_ERROR;
    }

    let mut Status = NtCreateEvent(
        &mut SockEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    if !NT_SUCCESS(Status) {
        return SOCKET_ERROR;
    }

    TdiAddressSize = core::mem::size_of::<TRANSPORT_ADDRESS>() as ULONG
        + ((*(*Socket).SharedData).SizeOfLocalAddress as u32);
    TdiAddress = HeapAlloc(GlobalHeap, 0, TdiAddressSize as usize) as PTDI_ADDRESS_INFO;

    if TdiAddress.is_null() {
        NtClose(SockEvent);
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOBUFS as i32);
        }
        return SOCKET_ERROR;
    }

    SocketAddress = core::ptr::addr_of_mut!((*TdiAddress).Address);

    Status = NtDeviceIoControlFile(
        (*Socket).Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        &mut IOSB,
        IOCTL_AFD_GET_SOCK_NAME,
        core::ptr::null_mut(),
        0,
        TdiAddress as PVOID,
        TdiAddressSize,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    NtClose(SockEvent);

    if NT_SUCCESS(Status) {
        if *NameLength >= (*(*Socket).SharedData).SizeOfLocalAddress as INT {
            (*Name).sa_family = (*SocketAddress).Address[0].AddressType;
            RtlCopyMemory(
                (*Name).sa_data.as_mut_ptr() as PVOID,
                (*SocketAddress).Address[0].Address.as_ptr() as PVOID,
                (*SocketAddress).Address[0].AddressLength as usize,
            );
            *NameLength = (*(*Socket).SharedData).SizeOfLocalAddress as INT;
            TRACE!(
                b"NameLength %d Address: %x Port %x\n\0".as_ptr() as PCHAR,
                *NameLength,
                (*(Name as *mut sockaddr_in)).sin_addr.s_addr,
                (*(Name as *mut sockaddr_in)).sin_port,
            );
            HeapFree(GlobalHeap, 0, TdiAddress as PVOID);
            return 0;
        } else {
            HeapFree(GlobalHeap, 0, TdiAddress as PVOID);
            if !lpErrno.is_null() {
                *lpErrno = (WSAEFAULT as i32);
            }
            return SOCKET_ERROR;
        }
    }

    HeapFree(GlobalHeap, 0, TdiAddress as PVOID);

    (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32)
}
#[no_mangle]
pub unsafe extern "system" fn WSPGetPeerName(
    s: SOCKET,
    Name: LPSOCKADDR,
    NameLength: LPINT,
    lpErrno: LPINT,
) -> INT {
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let TdiAddressSize: ULONG;
    let SocketAddress: PTRANSPORT_ADDRESS;
    let Socket: PSOCKET_INFORMATION;
    let Status: NTSTATUS;
    let mut SockEvent: HANDLE = core::ptr::null_mut();

    Socket = GetSocketStructure(s);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }

    if (*Socket).SharedData.is_null() || (*(*Socket).SharedData).State != SocketConnected {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTCONN as i32);
        }
        return SOCKET_ERROR;
    }

    if Name.is_null() || NameLength.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEFAULT as i32);
        }
        return SOCKET_ERROR;
    }

    let mut Status = NtCreateEvent(
        core::ptr::addr_of_mut!(SockEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    if !NT_SUCCESS(Status) {
        return SOCKET_ERROR;
    }

    TdiAddressSize = core::mem::size_of::<TRANSPORT_ADDRESS>() as ULONG
        + ((*(*Socket).SharedData).SizeOfRemoteAddress as u32);
    SocketAddress = HeapAlloc(GlobalHeap, 0, TdiAddressSize as usize) as PTRANSPORT_ADDRESS;

    if SocketAddress.is_null() {
        NtClose(SockEvent);
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOBUFS as i32);
        }
        return SOCKET_ERROR;
    }

    Status = NtDeviceIoControlFile(
        (*Socket).Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IOSB),
        IOCTL_AFD_GET_PEER_NAME,
        core::ptr::null_mut(),
        0,
        SocketAddress as PVOID,
        TdiAddressSize,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    NtClose(SockEvent);

    if NT_SUCCESS(Status) {
        if *NameLength >= (*(*Socket).SharedData).SizeOfRemoteAddress as INT {
            (*Name).sa_family = (*SocketAddress).Address[0].AddressType;
            core::ptr::copy_nonoverlapping(
                (*SocketAddress).Address[0].Address.as_ptr(),
                (*Name).sa_data.as_mut_ptr() as *mut u8,
                (*SocketAddress).Address[0].AddressLength as usize,
            );
            *NameLength = (*(*Socket).SharedData).SizeOfRemoteAddress as INT;
            TRACE!(
                b"NameLength %d Address: %x Port %x\n\0".as_ptr() as PCHAR,
                *NameLength,
                (*(Name as *mut sockaddr_in)).sin_addr.s_addr,
                (*(Name as *mut sockaddr_in)).sin_port,
            );
            HeapFree(GlobalHeap, 0, SocketAddress as LPVOID);
            return 0;
        } else {
            HeapFree(GlobalHeap, 0, SocketAddress as LPVOID);
            if !lpErrno.is_null() {
                *lpErrno = (WSAEFAULT as i32);
            }
            return SOCKET_ERROR;
        }
    }

    HeapFree(GlobalHeap, 0, SocketAddress as LPVOID);

    return (MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32);
}
#[no_mangle]
pub unsafe extern "system" fn WSPIoctl(
    Handle: SOCKET,
    dwIoControlCode: DWORD,
    lpvInBuffer: LPVOID,
    cbInBuffer: DWORD,
    lpvOutBuffer: LPVOID,
    cbOutBuffer: DWORD,
    lpcbBytesReturned: LPDWORD,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
    lpThreadId: LPWSATHREADID,
    lpErrno: LPINT,
) -> INT {

    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let mut NeedsCompletion: BOOL = if !lpOverlapped.is_null() { 1 } else { 0 };
    let mut NonBlocking: BOOLEAN = 0;
    let mut Errno: INT = NO_ERROR as INT;
    let mut Ret: INT = SOCKET_ERROR;
    let mut cbRet: DWORD = 0;

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAENOTSOCK as INT;
        }
        return SOCKET_ERROR;
    }

    if lpcbBytesReturned.is_null() && lpOverlapped.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAEFAULT as INT;
        }
        return SOCKET_ERROR;
    }

    match dwIoControlCode {
        FIONBIO => {
            if cbInBuffer < core::mem::size_of::<INT>() as DWORD || IS_INTRESOURCE(lpvInBuffer) != 0 {
                Errno = WSAEFAULT as INT;
            } else {
                NonBlocking = if *(lpvInBuffer as PULONG) != 0 { 1 } else { 0 };
                if NonBlocking == 0 {
                    if ((*(*Socket).SharedData).AsyncEvents & (!((*(*Socket).SharedData).AsyncDisabledEvents))) != 0 {
                        Errno = WSAEINVAL as INT;
                    } else if (*Socket).NetworkEvents != 0 {
                        Errno = WSAEINVAL as INT;
                    } else {
                        (*(*Socket).SharedData).__bindgen_anon_1.set_NonBlocking(if NonBlocking != 0 { 1 } else { 0 });
                        NeedsCompletion = 0;
                        Errno = SetSocketInformation(
                            Socket,
                            AFD_INFO_BLOCKING_MODE,
                            core::ptr::addr_of_mut!(NonBlocking),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            lpOverlapped,
                            lpCompletionRoutine,
                        );
                        if Errno == NO_ERROR as INT {
                            Ret = NO_ERROR as INT;
                        }
                    }
                } else {
                    (*(*Socket).SharedData).__bindgen_anon_1.set_NonBlocking(if NonBlocking != 0 { 1 } else { 0 });
                    NeedsCompletion = 0;
                    Errno = SetSocketInformation(
                        Socket,
                        AFD_INFO_BLOCKING_MODE,
                        core::ptr::addr_of_mut!(NonBlocking),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        lpOverlapped,
                        lpCompletionRoutine,
                    );
                    if Errno == NO_ERROR as INT {
                        Ret = NO_ERROR as INT;
                    }
                }
            }
        }
        FIONREAD => {
            if IS_INTRESOURCE(lpvOutBuffer) != 0 || cbOutBuffer == 0 {
                cbRet = core::mem::size_of::<ULONG>() as DWORD;
                Errno = WSAEFAULT as INT;
            } else if cbOutBuffer < core::mem::size_of::<ULONG>() as DWORD {
                Errno = WSAEINVAL as INT;
            } else {
                NeedsCompletion = 0;
                Errno = GetSocketInformation(
                    Socket,
                    AFD_INFO_RECEIVE_CONTENT_SIZE,
                    core::ptr::null_mut(),
                    lpvOutBuffer as PULONG,
                    core::ptr::null_mut(),
                    lpOverlapped,
                    lpCompletionRoutine,
                );
                if Errno == NO_ERROR as INT {
                    cbRet = core::mem::size_of::<ULONG>() as DWORD;
                    Ret = NO_ERROR as INT;
                }
            }
        }
        SIOCATMARK => {
            if IS_INTRESOURCE(lpvOutBuffer) != 0 || cbOutBuffer == 0 {
                cbRet = core::mem::size_of::<BOOL>() as DWORD;
                Errno = WSAEFAULT as INT;
            } else if cbOutBuffer < core::mem::size_of::<BOOL>() as DWORD {
                Errno = WSAEINVAL as INT;
            } else if (*(*Socket).SharedData).SocketType != SOCK_STREAM as INT {
                Errno = WSAEINVAL as INT;
            } else {
                *(lpvOutBuffer as *mut BOOL) = if (*(*Socket).SharedData).__bindgen_anon_1.OobInline() == 0 { 1 } else { 0 };
                cbRet = core::mem::size_of::<BOOL>() as DWORD;
                Errno = NO_ERROR as INT;
                Ret = NO_ERROR as INT;
            }
        }
        SIO_GET_EXTENSION_FUNCTION_POINTER => {
            if cbOutBuffer == 0 {
                cbRet = core::mem::size_of::<PVOID>() as DWORD;
                Errno = WSAEFAULT as INT;
            } else if cbInBuffer < core::mem::size_of::<GUID>() as DWORD
                || cbOutBuffer < core::mem::size_of::<PVOID>() as DWORD
            {
                Errno = WSAEINVAL as INT;
            } else {
                let AcceptExGUID: GUID = WSAID_ACCEPTEX;
                let ConnectExGUID: GUID = WSAID_CONNECTEX;
                let DisconnectExGUID: GUID = WSAID_DISCONNECTEX;
                let GetAcceptExSockaddrsGUID: GUID = WSAID_GETACCEPTEXSOCKADDRS;

                if IsEqualGUID(core::ptr::addr_of!(AcceptExGUID), lpvInBuffer as *const GUID) {
                    *(lpvOutBuffer as *mut PVOID) = WSPAcceptEx as PVOID;
                    cbRet = core::mem::size_of::<PVOID>() as DWORD;
                    Errno = NO_ERROR as INT;
                    Ret = NO_ERROR as INT;
                } else if IsEqualGUID(core::ptr::addr_of!(ConnectExGUID), lpvInBuffer as *const GUID) {
                    *(lpvOutBuffer as *mut PVOID) = WSPConnectEx as PVOID;
                    cbRet = core::mem::size_of::<PVOID>() as DWORD;
                    Errno = NO_ERROR as INT;
                    Ret = NO_ERROR as INT;
                } else if IsEqualGUID(core::ptr::addr_of!(DisconnectExGUID), lpvInBuffer as *const GUID) {
                    *(lpvOutBuffer as *mut PVOID) = WSPDisconnectEx as PVOID;
                    cbRet = core::mem::size_of::<PVOID>() as DWORD;
                    Errno = NO_ERROR as INT;
                    Ret = NO_ERROR as INT;
                } else if IsEqualGUID(core::ptr::addr_of!(GetAcceptExSockaddrsGUID), lpvInBuffer as *const GUID) {
                    *(lpvOutBuffer as *mut PVOID) = WSPGetAcceptExSockaddrs as PVOID;
                    cbRet = core::mem::size_of::<PVOID>() as DWORD;
                    Errno = NO_ERROR as INT;
                    Ret = NO_ERROR as INT;
                } else {
                    ERR!(
                        "Querying unknown extension function: %x\n",
                        (*(lpvInBuffer as *const GUID)).Data1,
                    );
                    Errno = WSAEOPNOTSUPP as INT;
                }
            }
        }
        SIO_ADDRESS_LIST_QUERY => {
            if IS_INTRESOURCE(lpvOutBuffer) != 0 || cbOutBuffer == 0 {
                cbRet = (core::mem::size_of::<SOCKET_ADDRESS_LIST>()
                    + core::mem::size_of::<SOCKADDR>()) as DWORD;
                Errno = WSAEFAULT as INT;
            } else if cbOutBuffer < core::mem::size_of::<INT>() as DWORD {
                Errno = WSAEINVAL as INT;
            } else {
                cbRet = (core::mem::size_of::<SOCKET_ADDRESS_LIST>()
                    + core::mem::size_of::<SOCKADDR>()) as DWORD;

                (*(lpvOutBuffer as *mut SOCKET_ADDRESS_LIST)).iAddressCount = 1;

                if cbOutBuffer
                    < (core::mem::size_of::<SOCKET_ADDRESS_LIST>()
                        + core::mem::size_of::<SOCKADDR>()) as DWORD
                {
                    Errno = WSAEFAULT as INT;
                } else {
                    (*(lpvOutBuffer as *mut SOCKET_ADDRESS_LIST)).Address[0].iSockaddrLength =
                        core::mem::size_of::<SOCKADDR>() as INT;
                    (*(lpvOutBuffer as *mut SOCKET_ADDRESS_LIST)).Address[0].lpSockaddr =
                        core::ptr::addr_of_mut!((*(*Socket).SharedData).WSLocalAddress);

                    Errno = NO_ERROR as INT;
                    Ret = NO_ERROR as INT;
                }
            }
        }
        _ => {
            Errno = ((*(*Socket).HelperData).WSHIoctl.unwrap())(
                (*Socket).HelperContext,
                Handle,
                (*Socket).TdiAddressHandle,
                (*Socket).TdiConnectionHandle,
                dwIoControlCode,
                lpvInBuffer,
                cbInBuffer,
                lpvOutBuffer,
                cbOutBuffer,
                core::ptr::addr_of_mut!(cbRet),
                lpOverlapped,
                lpCompletionRoutine,
                core::ptr::addr_of_mut!(NeedsCompletion),
            );

            if Errno == NO_ERROR as INT {
                Ret = NO_ERROR as INT;
            }
        }
    }

    if !lpOverlapped.is_null() && NeedsCompletion != 0 {
        (*lpOverlapped).Internal = Errno as ULONG_PTR;
        (*lpOverlapped).InternalHigh = cbRet as ULONG_PTR;
        if lpCompletionRoutine.is_some() {
            lpCompletionRoutine.unwrap()(Errno as DWORD, cbRet, lpOverlapped, 0);
        }
        if !(*lpOverlapped).hEvent.is_null() {
            SetEvent((*lpOverlapped).hEvent);
        }
        if PostQueuedCompletionStatus(Handle as HANDLE, cbRet, 0, lpOverlapped as LPOVERLAPPED) == 0 {
            ERR!("PostQueuedCompletionStatus failed %d\n", GetLastError());
        }
        return NO_ERROR as INT;
    }

    if !lpErrno.is_null() {
        *lpErrno = Errno;
    }
    if !lpcbBytesReturned.is_null() {
        *lpcbBytesReturned = cbRet;
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn WSPGetSockOpt(
    Handle: SOCKET,
    Level: INT,
    OptionName: INT,
    OptionValue: *mut CHAR,
    OptionLength: LPINT,
    lpErrno: LPINT,
) -> INT {
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let mut Buffer: PVOID;
    let mut BufferSize: INT;
    let mut BoolBuffer: BOOL = 0;
    let mut DwordBuffer: DWORD = 0;
    let Errno: INT;

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }
    if OptionLength.is_null() || OptionValue.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEFAULT as i32);
        }
        return SOCKET_ERROR;
    }

    match Level as u32 {
        SOL_SOCKET => {
            match OptionName as u32 {
                SO_TYPE => {
                    Buffer = core::ptr::addr_of_mut!((*(*Socket).SharedData).SocketType) as PVOID;
                    BufferSize = core::mem::size_of::<INT>() as INT;
                }

                SO_RCVBUF => {
                    Buffer = core::ptr::addr_of_mut!((*(*Socket).SharedData).SizeOfRecvBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<ULONG>() as INT;
                }

                SO_SNDBUF => {
                    Buffer = core::ptr::addr_of_mut!((*(*Socket).SharedData).SizeOfSendBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<ULONG>() as INT;
                }

                SO_ACCEPTCONN => {
                    BoolBuffer = ((*(*Socket).SharedData).__bindgen_anon_1.Listening() as i32);
                    Buffer = core::ptr::addr_of_mut!(BoolBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<BOOL>() as INT;
                }

                SO_BROADCAST => {
                    BoolBuffer = ((*(*Socket).SharedData).__bindgen_anon_1.Broadcast() as i32);
                    Buffer = core::ptr::addr_of_mut!(BoolBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<BOOL>() as INT;
                }

                SO_DEBUG => {
                    BoolBuffer = ((*(*Socket).SharedData).__bindgen_anon_1.Debug() as i32);
                    Buffer = core::ptr::addr_of_mut!(BoolBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<BOOL>() as INT;
                }

                SO_DONTLINGER => {
                    BoolBuffer = if (*(*Socket).SharedData).LingerData.l_onoff == 0 { 1 } else { 0 };
                    Buffer = core::ptr::addr_of_mut!(BoolBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<BOOL>() as INT;
                }

                SO_LINGER => {
                    if (*(*Socket).SharedData).SocketType == (SOCK_DGRAM as i32) {
                        if !lpErrno.is_null() {
                            *lpErrno = (WSAENOPROTOOPT as i32);
                        }
                        return SOCKET_ERROR;
                    }
                    Buffer = core::ptr::addr_of_mut!((*(*Socket).SharedData).LingerData) as PVOID;
                    BufferSize = core::mem::size_of::<linger>() as INT;
                }

                SO_OOBINLINE => {
                    BoolBuffer = if (*(*Socket).SharedData).__bindgen_anon_1.OobInline() != 0 { 1 } else { 0 };
                    Buffer = core::ptr::addr_of_mut!(BoolBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<BOOL>() as INT;
                }

                SO_KEEPALIVE | SO_DONTROUTE => {
                    'SendToHelper: {
                        Errno = ((*(*Socket).HelperData).WSHGetSocketInformation.unwrap())(
                            (*Socket).HelperContext,
                            Handle,
                            (*Socket).TdiAddressHandle,
                            (*Socket).TdiConnectionHandle,
                            Level,
                            OptionName,
                            OptionValue,
                            OptionLength as LPINT,
                        );
                        if !lpErrno.is_null() {
                            *lpErrno = Errno;
                        }
                        return if Errno == (NO_ERROR as i32) { (NO_ERROR as i32) } else { SOCKET_ERROR };
                    }
                }

                SO_CONDITIONAL_ACCEPT => {
                    BoolBuffer = if (*(*Socket).SharedData).__bindgen_anon_1.UseDelayedAcceptance() != 0 { 1 } else { 0 };
                    Buffer = core::ptr::addr_of_mut!(BoolBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<BOOL>() as INT;
                }

                SO_REUSEADDR => {
                    BoolBuffer = if (*(*Socket).SharedData).__bindgen_anon_1.ReuseAddresses() != 0 { 1 } else { 0 };
                    Buffer = core::ptr::addr_of_mut!(BoolBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<BOOL>() as INT;
                }

                SO_EXCLUSIVEADDRUSE => {
                    BoolBuffer = if (*(*Socket).SharedData).__bindgen_anon_1.ExclusiveAddressUse() != 0 { 1 } else { 0 };
                    Buffer = core::ptr::addr_of_mut!(BoolBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<BOOL>() as INT;
                }

                SO_ERROR => {
                    Buffer = core::ptr::addr_of_mut!((*(*Socket).SharedData).SocketLastError) as PVOID;
                    BufferSize = core::mem::size_of::<INT>() as INT;
                }

                SO_CONNECT_TIME => {
                    // Not connected -> 0xFFFFFFFF, Windows' documented answer (upstream
                    // a68bfa2fb41); subtracting from 0 returned seconds-since-1970.
                    let ConnectTime = (*(*Socket).SharedData).ConnectTime;
                    DwordBuffer = if ConnectTime != 0 {
                        GetCurrentTimeInSeconds().wrapping_sub(ConnectTime)
                    } else {
                        0xFFFF_FFFF
                    };
                    Buffer = core::ptr::addr_of_mut!(DwordBuffer) as PVOID;
                    BufferSize = core::mem::size_of::<DWORD>() as INT;
                }

                SO_SNDTIMEO => {
                    Buffer = core::ptr::addr_of_mut!((*(*Socket).SharedData).SendTimeout) as PVOID;
                    BufferSize = core::mem::size_of::<DWORD>() as INT;
                }

                SO_RCVTIMEO => {
                    Buffer = core::ptr::addr_of_mut!((*(*Socket).SharedData).RecvTimeout) as PVOID;
                    BufferSize = core::mem::size_of::<DWORD>() as INT;
                }

                SO_PROTOCOL_INFOW => {
                    Buffer = core::ptr::addr_of_mut!((*Socket).ProtocolInfo) as PVOID;
                    BufferSize = core::mem::size_of::<WSAPROTOCOL_INFOW>() as INT;
                }

                SO_GROUP_ID | SO_GROUP_PRIORITY | SO_MAX_MSG_SIZE | _ => {
                    DbgPrint(b"MSAFD: Get unknown optname %x\n\0".as_ptr() as PCHAR, OptionName);
                    if !lpErrno.is_null() {
                        *lpErrno = (WSAENOPROTOOPT as i32);
                    }
                    return SOCKET_ERROR;
                }
            }

            if *OptionLength < BufferSize {
                if !lpErrno.is_null() {
                    *lpErrno = (WSAEFAULT as i32);
                }
                *OptionLength = BufferSize;
                return SOCKET_ERROR;
            }
            RtlCopyMemory(OptionValue as PVOID, Buffer, BufferSize as usize);

            return 0;
        }

        _ => {
            if !lpErrno.is_null() {
                *lpErrno = (WSAEINVAL as i32);
            }
            return SOCKET_ERROR;
        }
    }
}

unsafe fn MsafdUpdateConnectionContext(
    Handle: SOCKET,
    lpErrno: LPINT,
) -> INT {
    let mut DummyIOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let IOSB: PIO_STATUS_BLOCK = &mut DummyIOSB;
    let mut Status: NTSTATUS;
    let mut SharedData: SOCK_SHARED_INFO = core::mem::zeroed();
    let mut HandleData: AFD_TDI_HANDLE_DATA = core::mem::zeroed();
    let mut SockEvent: HANDLE = core::ptr::null_mut();
    let Socket: PSOCKET_INFORMATION = GetSocketStructure(Handle);

    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAENOTSOCK as i32);
        }
        return SOCKET_ERROR;
    }

    if (*(*Socket).SharedData).State == SocketConnected {
        return (NO_ERROR as i32);
    }

    Status = NtCreateEvent(
        &mut SockEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        'Quit: {
            if Status == STATUS_SUCCESS {
                return (NO_ERROR as i32);
            }
            if !lpErrno.is_null() {
                *lpErrno = TranslateNtStatusError(Status);
            }
            return SOCKET_ERROR;
        }
    }

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        IOSB,
        IOCTL_AFD_GET_CONTEXT,
        core::ptr::null_mut(),
        0,
        &mut SharedData as *mut _ as PVOID,
        core::mem::size_of::<SOCK_SHARED_INFO>() as ULONG,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = (*IOSB).__bindgen_anon_1.Status;
    }

    if !NT_SUCCESS(Status) {
        'Quit: {
            if Status == STATUS_SUCCESS {
                return (NO_ERROR as i32);
            }
            if !lpErrno.is_null() {
                *lpErrno = TranslateNtStatusError(Status);
            }
            return SOCKET_ERROR;
        }
    }

    let mut flags: ULONG = AFD_CONNECTION_HANDLE | AFD_ADDRESS_HANDLE;

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        IOSB,
        IOCTL_AFD_GET_TDI_HANDLES,
        &mut flags as *mut _ as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        &mut HandleData as *mut _ as PVOID,
        core::mem::size_of::<AFD_TDI_HANDLE_DATA>() as ULONG,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = (*IOSB).__bindgen_anon_1.Status;
    }

    if !NT_SUCCESS(Status) {
        'Quit: {
            if Status == STATUS_SUCCESS {
                return (NO_ERROR as i32);
            }
            if !lpErrno.is_null() {
                *lpErrno = TranslateNtStatusError(Status);
            }
            return SOCKET_ERROR;
        }
    }

    (*(*Socket).SharedData).State = SharedData.State;

    if SharedData.State == SocketConnected {
        (*Socket).TdiConnectionHandle = HandleData.TdiConnectionHandle;
        (*Socket).TdiAddressHandle = HandleData.TdiAddressHandle;
        (*(*Socket).SharedData).ConnectTime = SharedData.ConnectTime;

        SockReenableAsyncSelectEvent(Socket, FD_WRITE);

        SockReenableAsyncSelectEvent(Socket, FD_CONNECT);

        ((*(*Socket).HelperData).WSHNotify.unwrap())(
            (*Socket).HelperContext,
            (*Socket).Handle,
            (*Socket).TdiAddressHandle,
            (*Socket).TdiConnectionHandle,
            WSH_NOTIFY_CONNECT,
        );
    }

    NtClose(SockEvent);

    if Status == STATUS_SUCCESS {
        return (NO_ERROR as i32);
    }
    if !lpErrno.is_null() {
        *lpErrno = TranslateNtStatusError(Status);
    }
    return SOCKET_ERROR;
}
#[no_mangle]
pub unsafe extern "system" fn WSPStartup(
    wVersionRequested: WORD,
    lpWSPData: LPWSPDATA,
    lpProtocolInfo: LPWSAPROTOCOL_INFOW,
    UpcallTable: WSPUPCALLTABLE,
    lpProcTable: LPWSPPROC_TABLE,
) -> INT {
    let Status: NTSTATUS;

    if ((LOBYTE((wVersionRequested as usize)) == 2 && HIBYTE((wVersionRequested as usize)) < 2) ||
        LOBYTE((wVersionRequested as usize)) < 2)
    {
        ERR!("WSPStartup NOT SUPPORTED for version 0x%X\n", wVersionRequested as core::ffi::c_uint);
        return (WSAVERNOTSUPPORTED as i32);
    } else {
        Status = (NO_ERROR as i32);
    }

    Upcalls = UpcallTable;

    if Status == (NO_ERROR as i32) {
        (*lpProcTable).lpWSPAccept = Some(WSPAccept);
        (*lpProcTable).lpWSPAddressToString = Some(WSPAddressToString);
        (*lpProcTable).lpWSPAsyncSelect = Some(WSPAsyncSelect);
        (*lpProcTable).lpWSPBind = Some(WSPBind);
        (*lpProcTable).lpWSPCancelBlockingCall = Some(WSPCancelBlockingCall);
        (*lpProcTable).lpWSPCleanup = Some(WSPCleanup);
        (*lpProcTable).lpWSPCloseSocket = Some(WSPCloseSocket);
        (*lpProcTable).lpWSPConnect = Some(WSPConnect);
        (*lpProcTable).lpWSPDuplicateSocket = Some(WSPDuplicateSocket);
        (*lpProcTable).lpWSPEnumNetworkEvents = Some(WSPEnumNetworkEvents);
        (*lpProcTable).lpWSPEventSelect = Some(WSPEventSelect);
        (*lpProcTable).lpWSPGetOverlappedResult = Some(WSPGetOverlappedResult);
        (*lpProcTable).lpWSPGetPeerName = Some(WSPGetPeerName);
        (*lpProcTable).lpWSPGetSockName = Some(WSPGetSockName);
        (*lpProcTable).lpWSPGetSockOpt = Some(WSPGetSockOpt);
        (*lpProcTable).lpWSPGetQOSByName = Some(WSPGetQOSByName);
        (*lpProcTable).lpWSPIoctl = Some(WSPIoctl);
        (*lpProcTable).lpWSPJoinLeaf = Some(WSPJoinLeaf);
        (*lpProcTable).lpWSPListen = Some(WSPListen);
        (*lpProcTable).lpWSPRecv = Some(WSPRecv);
        (*lpProcTable).lpWSPRecvDisconnect = Some(WSPRecvDisconnect);
        (*lpProcTable).lpWSPRecvFrom = Some(WSPRecvFrom);
        (*lpProcTable).lpWSPSelect = Some(WSPSelect);
        (*lpProcTable).lpWSPSend = Some(WSPSend);
        (*lpProcTable).lpWSPSendDisconnect = Some(WSPSendDisconnect);
        (*lpProcTable).lpWSPSendTo = Some(WSPSendTo);
        (*lpProcTable).lpWSPSetSockOpt = Some(WSPSetSockOpt);
        (*lpProcTable).lpWSPShutdown = Some(WSPShutdown);
        (*lpProcTable).lpWSPSocket = Some(WSPSocket);
        (*lpProcTable).lpWSPStringToAddress = Some(WSPStringToAddress);
        (*lpWSPData).wVersion = MAKEWORD(2, 2);
        (*lpWSPData).wHighVersion = MAKEWORD(2, 2);
        CatalogEntryId = (*lpProtocolInfo).dwCatalogEntryId;
    }

    TRACE!(b"Status (%d).\n\0".as_ptr() as PCHAR, Status);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn WSPAddressToString(
    lpsaAddress: LPSOCKADDR,
    dwAddressLength: DWORD,
    lpProtocolInfo: LPWSAPROTOCOL_INFOW,
    lpszAddressString: LPWSTR,
    lpdwAddressStringLength: LPDWORD,
    lpErrno: LPINT,
) -> INT {

    let size: usize;
    let mut buffer: [u16; 54] = [0; 54];
    let p: *mut u16;

    if lpsaAddress.is_null() || lpszAddressString.is_null() || lpdwAddressStringLength.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEFAULT as i32);
        }
        return SOCKET_ERROR;
    }

    match (*lpsaAddress).sa_family as u32 {
        AF_INET => {
            if dwAddressLength < core::mem::size_of::<SOCKADDR_IN>() as DWORD {
                if !lpErrno.is_null() {
                    *lpErrno = (WSAEINVAL as i32);
                }
                return SOCKET_ERROR;
            }
            let sin_addr = (lpsaAddress as *const SOCKADDR_IN);
            let addr = ntohl((*sin_addr).sin_addr.S_un.S_addr);
            let port = ntohs((*sin_addr).sin_port);
            
            static FMT_IPV4: [u16; 15] = [
                0x25, 0x75, 0x2e, 0x25, 0x75, 0x2e, 0x25, 0x75, 0x2e, 0x25, 0x75, 0x3a, 0x25,
                0x75, 0x00,
            ];
            swprintf(
                buffer.as_mut_ptr(),
                FMT_IPV4.as_ptr(),
                (addr >> 24 & 0xff) as u32,
                (addr >> 16 & 0xff) as u32,
                (addr >> 8 & 0xff) as u32,
                (addr & 0xff) as u32,
                port as u32,
            );

            p = wcschr(buffer.as_ptr(), 0x3a_u16);
            if (*sin_addr).sin_port == 0 {
                *p = 0;
            }
        }
        _ => {
            if !lpErrno.is_null() {
                *lpErrno = (WSAEINVAL as i32);
            }
            return SOCKET_ERROR;
        }
    }

    size = wcslen(buffer.as_ptr()) + 1;

    if *lpdwAddressStringLength < size as DWORD {
        *lpdwAddressStringLength = size as DWORD;
        if !lpErrno.is_null() {
            *lpErrno = (WSAEFAULT as i32);
        }
        return SOCKET_ERROR;
    }

    *lpdwAddressStringLength = size as DWORD;
    wcscpy(lpszAddressString, buffer.as_ptr());
    0
}
#[no_mangle]
pub unsafe extern "system" fn WSPStringToAddress(
    mut AddressString: LPWSTR,
    AddressFamily: INT,
    lpProtocolInfo: LPWSAPROTOCOL_INFOW,
    lpAddress: LPSOCKADDR,
    lpAddressLength: LPINT,
    lpErrno: LPINT,
) -> INT {

    let mut numdots: core::ffi::c_int = 0;
    let mut port: USHORT;
    let mut inetaddr: LONG = 0;
    let mut ip_part: LONG;
    let mut bp: *mut LPWSTR = core::ptr::null_mut();
    let sockaddr: *mut SOCKADDR_IN;

    if lpAddressLength.is_null() || lpAddress.is_null() || AddressString.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = (WSAEINVAL as i32);
        }
        return SOCKET_ERROR;
    }

    sockaddr = lpAddress as *mut SOCKADDR_IN;

    if !lpProtocolInfo.is_null() {
        (*sockaddr).sin_family = ((*lpProtocolInfo).iAddressFamily as u16 as i16);
    } else {
        (*sockaddr).sin_family = (AddressFamily as u16 as i16);
    }

    if AddressFamily == (AF_INET as i32) {
        if *lpAddressLength < core::mem::size_of::<SOCKADDR_IN>() as INT {
            if !lpErrno.is_null() {
                *lpErrno = (WSAEFAULT as i32);
            }
        } else {
            bp = core::ptr::addr_of_mut!(AddressString);
            inetaddr = 0;

            while *bp < AddressString.add(wcslen(AddressString)) {
                ip_part = wcstol(*bp, bp, 10);
                if ip_part < 0 || ip_part > 255 {
                    if !lpErrno.is_null() {
                        *lpErrno = (WSAEINVAL as i32);
                    }
                    return SOCKET_ERROR;
                }
                inetaddr = (inetaddr << 8) + ip_part;
                if (**bp) == 0 || (**bp) == ':' as u16 {
                    break;
                }
                if (**bp) != '.' as u16 {
                    if !lpErrno.is_null() {
                        *lpErrno = (WSAEINVAL as i32);
                    }
                    return SOCKET_ERROR;
                }
                numdots += 1;
                *bp = (*bp).add(1);
            }

            if numdots != 3 {
                if !lpErrno.is_null() {
                    *lpErrno = (WSAEINVAL as i32);
                }
                return SOCKET_ERROR;
            }

            if (**bp) == ':' as u16 {
                *bp = (*bp).add(1);
                if (**bp) < '0' as u16 || (**bp) > '9' as u16 {
                    if !lpErrno.is_null() {
                        *lpErrno = (WSAEINVAL as i32);
                    }
                    return SOCKET_ERROR;
                }
                port = wcstol(*bp, bp, 10) as USHORT;
            } else {
                port = 0;
            }

            if !lpErrno.is_null() {
                *lpErrno = (NO_ERROR as i32);
            }
            *lpAddressLength = core::mem::size_of::<SOCKADDR_IN>() as INT;
            memset(lpAddress as *mut core::ffi::c_void, 0, core::mem::size_of::<SOCKADDR_IN>());
            (*sockaddr).sin_family = (AF_INET as u16 as i16);
            (*sockaddr).sin_addr.S_un.S_addr = inetaddr as u32;
            (*sockaddr).sin_port = port;
        }
    }

    if !lpErrno.is_null() && *lpErrno == 0 {
        return 0;
    }

    SOCKET_ERROR
}
#[no_mangle]
pub unsafe extern "system" fn WSPCleanup(lpErrno: LPINT) -> INT {
    
    TRACE!(b"Leaving.\n\0".as_ptr() as *const i8);

    if !lpErrno.is_null() {
        *lpErrno = (NO_ERROR as i32);
    }

    0
}
unsafe extern "system" fn MsafdInfoAPC(
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Reserved: ULONG) {
    let Context = ApcContext as PMSAFD_INFO_APC_CONTEXT;

    TRACE!(b"MsafdInfoAPC(%p %lx %lx)\n\0".as_ptr() as *const i8,
          ApcContext, 
          (*IoStatusBlock).__bindgen_anon_1.Status, 
          (*IoStatusBlock).Information);

    if (*Context).lpCompletionRoutine.is_some() {
        (*Context).lpCompletionRoutine.unwrap()(
            ((*IoStatusBlock).__bindgen_anon_1.Status as u32),
            (*IoStatusBlock).Information as u32,
            (*Context).lpOverlapped,
            0
        );
    }

    HeapFree(GlobalHeap, 0, (*Context).lpInfoData as LPVOID);

    HeapFree(GlobalHeap, 0, ApcContext);
}
// WSPSetSockOpt — faithful translation of dll/win32/msafd/misc/dllmain.c:WSPSetSockOpt.
// Dropped by the per-function pass (never emitted a body); reconstructed here 1:1. SOL_SOCKET
// options are handled locally (bitfields via __bindgen_anon_1 accessors, buffer sizes via
// Set/GetSocketInformation); everything else + SO_KEEPALIVE/SO_DONTROUTE goes to the helper DLL.
#[no_mangle]
pub unsafe extern "system" fn WSPSetSockOpt(
    s: SOCKET,
    level: INT,
    optname: INT,
    optval: *const CHAR,
    optlen: INT,
    lpErrno: LPINT,
) -> INT {
    let Socket: PSOCKET_INFORMATION;
    let Errno: INT;

    Socket = GetSocketStructure(s);
    if Socket.is_null() {
        if !lpErrno.is_null() { *lpErrno = WSAENOTSOCK as INT; }
        return SOCKET_ERROR;
    }
    if optval.is_null() && optlen != 0 {
        if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
        return SOCKET_ERROR;
    }

    /* FIXME: We should handle some more cases here */
    if level == SOL_SOCKET as INT {
        match optname as u32 {
            SO_BROADCAST => {
                if (optlen as usize) < core::mem::size_of::<BOOL>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                (*(*Socket).SharedData).__bindgen_anon_1.set_Broadcast(if *optval != 0 { 1 } else { 0 });
                return NO_ERROR as INT;
            }
            SO_OOBINLINE => {
                if (optlen as usize) < core::mem::size_of::<BOOL>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                (*(*Socket).SharedData).__bindgen_anon_1.set_OobInline(if *optval != 0 { 1 } else { 0 });
                return NO_ERROR as INT;
            }
            SO_DONTLINGER => {
                if (optlen as usize) < core::mem::size_of::<BOOL>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                (*(*Socket).SharedData).LingerData.l_onoff = if *optval != 0 { 0 } else { 1 };
                return NO_ERROR as INT;
            }
            SO_REUSEADDR => {
                if (optlen as usize) < core::mem::size_of::<BOOL>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                (*(*Socket).SharedData).__bindgen_anon_1.set_ReuseAddresses(if *optval != 0 { 1 } else { 0 });
                return NO_ERROR as INT;
            }
            SO_EXCLUSIVEADDRUSE => {
                if (optlen as usize) < core::mem::size_of::<BOOL>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                (*(*Socket).SharedData).__bindgen_anon_1.set_ExclusiveAddressUse(if *optval != 0 { 1 } else { 0 });
                return NO_ERROR as INT;
            }
            SO_LINGER => {
                if (optlen as usize) < core::mem::size_of::<linger>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*(*Socket).SharedData).LingerData) as *mut core::ffi::c_void,
                    optval as *const core::ffi::c_void,
                    core::mem::size_of::<linger>(),
                );
                return NO_ERROR as INT;
            }
            SO_SNDBUF => {
                if (optlen as usize) < core::mem::size_of::<ULONG>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                SetSocketInformation(Socket, AFD_INFO_SEND_WINDOW_SIZE, core::ptr::null_mut(),
                    optval as PULONG, core::ptr::null_mut(), core::ptr::null_mut(), None);
                GetSocketInformation(Socket, AFD_INFO_SEND_WINDOW_SIZE, core::ptr::null_mut(),
                    core::ptr::addr_of_mut!((*(*Socket).SharedData).SizeOfSendBuffer),
                    core::ptr::null_mut(), core::ptr::null_mut(), None);
                return NO_ERROR as INT;
            }
            SO_RCVBUF => {
                if (optlen as usize) < core::mem::size_of::<ULONG>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                /* FIXME: limit the packet receive buffer size — workaround for CORE-15804 */
                if *(optval as PULONG) > 0x2000 {
                    *(optval as *mut ULONG) = 0x2000;
                }
                SetSocketInformation(Socket, AFD_INFO_RECEIVE_WINDOW_SIZE, core::ptr::null_mut(),
                    optval as PULONG, core::ptr::null_mut(), core::ptr::null_mut(), None);
                GetSocketInformation(Socket, AFD_INFO_RECEIVE_WINDOW_SIZE, core::ptr::null_mut(),
                    core::ptr::addr_of_mut!((*(*Socket).SharedData).SizeOfRecvBuffer),
                    core::ptr::null_mut(), core::ptr::null_mut(), None);
                return NO_ERROR as INT;
            }
            SO_UPDATE_CONNECT_CONTEXT | SO_UPDATE_ACCEPT_CONTEXT => {
                return MsafdUpdateConnectionContext(s, lpErrno);
            }
            SO_ERROR => {
                if (optlen as usize) < core::mem::size_of::<INT>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*(*Socket).SharedData).SocketLastError) as *mut core::ffi::c_void,
                    optval as *const core::ffi::c_void, core::mem::size_of::<INT>());
                return NO_ERROR as INT;
            }
            SO_SNDTIMEO => {
                if (optlen as usize) < core::mem::size_of::<DWORD>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*(*Socket).SharedData).SendTimeout) as *mut core::ffi::c_void,
                    optval as *const core::ffi::c_void, core::mem::size_of::<DWORD>());
                return NO_ERROR as INT;
            }
            SO_RCVTIMEO => {
                if (optlen as usize) < core::mem::size_of::<DWORD>() {
                    if !lpErrno.is_null() { *lpErrno = WSAEFAULT as INT; }
                    return SOCKET_ERROR;
                }
                RtlCopyMemory(
                    core::ptr::addr_of_mut!((*(*Socket).SharedData).RecvTimeout) as *mut core::ffi::c_void,
                    optval as *const core::ffi::c_void, core::mem::size_of::<DWORD>());
                return NO_ERROR as INT;
            }
            SO_KEEPALIVE | SO_DONTROUTE => {
                /* These go directly to the helper dll — fall through to SendToHelper */
            }
            _ => {
                /* Obviously this is a hack */
                ERR!("MSAFD: Set unknown optname %x\n", optname);
                return NO_ERROR as INT;
            }
        }
    }

    // SendToHelper:
    Errno = ((*(*Socket).HelperData).WSHSetSocketInformation.unwrap())(
        (*Socket).HelperContext,
        s,
        (*Socket).TdiAddressHandle,
        (*Socket).TdiConnectionHandle,
        level,
        optname,
        optval as PCHAR,
        optlen,
    );
    if !lpErrno.is_null() { *lpErrno = Errno; }
    return if Errno == NO_ERROR as INT { NO_ERROR as INT } else { SOCKET_ERROR };
}
#[no_mangle]
pub unsafe extern "C" fn GetSocketInformation(
    Socket: PSOCKET_INFORMATION,
    AfdInformationClass: ULONG,
    Boolean: PBOOLEAN,
    Ulong: PULONG,
    LargeInteger: PLARGE_INTEGER,
    Overlapped: LPWSAOVERLAPPED,
    CompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
) -> INT {
    let mut IOSB: PIO_STATUS_BLOCK;
    let mut DummyIOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let InfoData: PAFD_INFO;
    let mut Status: NTSTATUS;
    let APCContext: PMSAFD_INFO_APC_CONTEXT;
    let APCFunction: PIO_APC_ROUTINE;
    let mut Event: HANDLE = core::ptr::null_mut();
    let mut SockEvent: HANDLE = core::ptr::null_mut();

    InfoData = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<AFD_INFO>()) as PAFD_INFO;
    if InfoData.is_null() {
        return SOCKET_ERROR;
    }

    Status = NtCreateEvent(
        &mut SockEvent as *mut _,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        HeapFree(GlobalHeap, 0, InfoData as LPVOID);
        return SOCKET_ERROR;
    }

    (*InfoData).InformationClass = AfdInformationClass;

    if Overlapped.is_null() {
        APCContext = core::ptr::null_mut();
        APCFunction = None;
        Event = SockEvent;
        IOSB = &mut DummyIOSB as *mut _;
    } else {
        if ((*(*Socket).SharedData).CreateFlags & SO_SYNCHRONOUS_NONALERT) != 0 {
            TRACE!(b"Opened without flag WSA_FLAG_OVERLAPPED. Do nothing.\n\0".as_ptr() as PCHAR);
            NtClose(SockEvent);
            HeapFree(GlobalHeap, 0, InfoData as LPVOID);
            return (MsafdReturnWithErrno(STATUS_SUCCESS, core::ptr::null_mut(), 0, core::ptr::null_mut()) as i32);
        }

        APCContext = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<MSAFD_INFO_APC_CONTEXT>()) as PMSAFD_INFO_APC_CONTEXT;
        if APCContext.is_null() {
            ERR!("Not enough memory for APC Context\n");
            NtClose(SockEvent);
            HeapFree(GlobalHeap, 0, InfoData as LPVOID);
            return (MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, core::ptr::null_mut(), 0, core::ptr::null_mut()) as i32);
        }
        (*APCContext).lpCompletionRoutine = CompletionRoutine;
        (*APCContext).lpOverlapped = Overlapped;
        (*APCContext).lpInfoData = InfoData;
        APCFunction = as_apc(MsafdInfoAPC);

        if CompletionRoutine.is_none() {
            Event = (*Overlapped).hEvent;
        }

        IOSB = &mut (*Overlapped).Internal as *mut _ as PIO_STATUS_BLOCK;
    }

    (*IOSB).__bindgen_anon_1.Status = STATUS_PENDING;

    Status = NtDeviceIoControlFile(
        (*Socket).Handle as HANDLE,
        Event,
        APCFunction,
        APCContext as PVOID,
        IOSB,
        IOCTL_AFD_GET_INFO,
        InfoData as PVOID,
        core::mem::size_of::<AFD_INFO>() as ULONG,
        InfoData as PVOID,
        core::mem::size_of::<AFD_INFO>() as ULONG,
    );

    if Overlapped.is_null() && Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = (*IOSB).__bindgen_anon_1.Status;
    }

    TRACE!(b"Status %lx\n\0".as_ptr() as PCHAR, Status);

    if Status == STATUS_SUCCESS {
        if !Ulong.is_null() {
            *Ulong = *(*InfoData).Information.Ulong.as_ref();
        }
        if !LargeInteger.is_null() {
            *LargeInteger = *(*InfoData).Information.LargeInteger.as_ref();
        }
        if !Boolean.is_null() {
            *Boolean = *(*InfoData).Information.Boolean.as_ref();
        }
    }

    NtClose(SockEvent);
    if APCFunction.is_none() {
        HeapFree(GlobalHeap, 0, InfoData as LPVOID);
    }

    (MsafdReturnWithErrno(Status, core::ptr::null_mut(), 0, core::ptr::null_mut()) as i32)
}
#[no_mangle]
pub unsafe extern "C" fn SetSocketInformation(
    Socket: PSOCKET_INFORMATION,
    AfdInformationClass: ULONG,
    Boolean: PBOOLEAN,
    Ulong: PULONG,
    LargeInteger: PLARGE_INTEGER,
    Overlapped: LPWSAOVERLAPPED,
    CompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
) -> INT {
    let IOSB: PIO_STATUS_BLOCK;
    let mut DummyIOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let InfoData: PAFD_INFO;
    let mut Status: NTSTATUS;
    let APCContext: PMSAFD_INFO_APC_CONTEXT;
    let APCFunction: PIO_APC_ROUTINE;
    let mut Event: HANDLE = core::ptr::null_mut();
    let mut SockEvent: HANDLE = core::ptr::null_mut();

    InfoData = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<AFD_INFO>()) as PAFD_INFO;
    if InfoData.is_null() {
        return SOCKET_ERROR;
    }

    Status = NtCreateEvent(
        core::ptr::addr_of_mut!(SockEvent),
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        HeapFree(GlobalHeap, 0, InfoData as LPVOID);
        return SOCKET_ERROR;
    }

    (*InfoData).InformationClass = AfdInformationClass;

    if !Ulong.is_null() {
        *(*InfoData).Information.Ulong.as_mut() = *Ulong;
    }
    if !LargeInteger.is_null() {
        *(*InfoData).Information.LargeInteger.as_mut() = *LargeInteger;
    }
    if !Boolean.is_null() {
        *(*InfoData).Information.Boolean.as_mut() = *Boolean;
    }

    if Overlapped.is_null() {
        APCContext = core::ptr::null_mut();
        APCFunction = None;
        Event = SockEvent;
        IOSB = core::ptr::addr_of_mut!(DummyIOSB);
    } else {
        if ((*(*Socket).SharedData).CreateFlags & SO_SYNCHRONOUS_NONALERT) != 0 {
            TRACE!(b"Opened without flag WSA_FLAG_OVERLAPPED. Do nothing.\n\0".as_ptr() as PCHAR);
            NtClose(SockEvent);
            HeapFree(GlobalHeap, 0, InfoData as LPVOID);
            return (MsafdReturnWithErrno(STATUS_SUCCESS, core::ptr::null_mut(), 0, core::ptr::null_mut()) as i32);
        }

        APCContext = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<MSAFD_INFO_APC_CONTEXT>()) as PMSAFD_INFO_APC_CONTEXT;
        if APCContext.is_null() {
            ERR!("Not enough memory for APC Context\n");
            NtClose(SockEvent);
            HeapFree(GlobalHeap, 0, InfoData as LPVOID);
            return (MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, core::ptr::null_mut(), 0, core::ptr::null_mut()) as i32);
        }
        (*APCContext).lpCompletionRoutine = CompletionRoutine;
        (*APCContext).lpOverlapped = Overlapped;
        (*APCContext).lpInfoData = InfoData;
        APCFunction = as_apc(MsafdInfoAPC);

        if CompletionRoutine.is_none() {
            Event = (*Overlapped).hEvent;
        }

        IOSB = core::ptr::addr_of_mut!((*Overlapped).Internal) as PIO_STATUS_BLOCK;
    }

    (*IOSB).__bindgen_anon_1.Status = STATUS_PENDING;

    Status = NtDeviceIoControlFile(
        (*Socket).Handle as HANDLE,
        Event,
        APCFunction,
        APCContext as PVOID,
        IOSB,
        IOCTL_AFD_SET_INFO,
        InfoData as PVOID,
        core::mem::size_of::<AFD_INFO>() as ULONG,
        core::ptr::null_mut(),
        0,
    );

    if Overlapped.is_null() && Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = (*IOSB).__bindgen_anon_1.Status;
    }

    TRACE!(b"Status %lx\n\0".as_ptr() as PCHAR, Status);

    NtClose(SockEvent);
    if APCFunction.is_none() {
        HeapFree(GlobalHeap, 0, InfoData as LPVOID);
    }

    (MsafdReturnWithErrno(Status, core::ptr::null_mut(), 0, core::ptr::null_mut()) as i32)
}
#[no_mangle]
pub unsafe extern "C" fn GetSocketStructure(Handle: SOCKET) -> PSOCKET_INFORMATION {

    let mut CurrentSocket: PSOCKET_INFORMATION;

    EnterCriticalSection(core::ptr::addr_of_mut!(SocketListLock));

    CurrentSocket = SocketListHead;
    while !CurrentSocket.is_null() {
        if (*CurrentSocket).Handle == Handle {
            LeaveCriticalSection(core::ptr::addr_of_mut!(SocketListLock));
            return CurrentSocket;
        }

        CurrentSocket = (*CurrentSocket).NextSocket;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(SocketListLock));

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn CreateContext(Socket: PSOCKET_INFORMATION) -> INT {
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ContextData: SOCKET_CONTEXT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut SockEvent: HANDLE = core::ptr::null_mut();

    Status = NtCreateEvent(
        &mut SockEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0u8,
    );

    if !NT_SUCCESS(Status) {
        return SOCKET_ERROR;
    }

    ContextData.SharedData = core::ptr::read((*Socket).SharedData);
    ContextData.SizeOfHelperData = 0;
    RtlCopyMemory(
        core::ptr::addr_of_mut!(ContextData.LocalAddress) as PVOID,
        (*Socket).LocalAddress as PVOID,
        (*(*Socket).SharedData).SizeOfLocalAddress as usize,
    );
    RtlCopyMemory(
        core::ptr::addr_of_mut!(ContextData.RemoteAddress) as PVOID,
        (*Socket).RemoteAddress as PVOID,
        (*(*Socket).SharedData).SizeOfRemoteAddress as usize,
    );

    Status = NtDeviceIoControlFile(
        (*Socket).Handle as HANDLE,
        SockEvent,
        None,
        core::ptr::null_mut(),
        &mut IOSB,
        IOCTL_AFD_SET_CONTEXT,
        core::ptr::addr_of_mut!(ContextData) as PVOID,
        core::mem::size_of::<SOCKET_CONTEXT>() as ULONG,
        core::ptr::null_mut(),
        0,
    );

    if Status == STATUS_PENDING {
        MsafdWaitForAlert(SockEvent);
        Status = IOSB.__bindgen_anon_1.Status;
    }

    NtClose(SockEvent);

    if Status == STATUS_SUCCESS {
        (NO_ERROR as i32)
    } else {
        SOCKET_ERROR
    }
}
#[no_mangle]
pub unsafe extern "C" fn SockCreateOrReferenceAsyncThread() -> u8 {
    let mut hAsyncThread: HANDLE;
    let mut AsyncThreadId: DWORD = 0;
    let mut AsyncEvent: HANDLE = core::ptr::null_mut();
    let mut HandleFlags: _OBJECT_HANDLE_FLAG_INFORMATION = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if SockAsyncThreadRefCount != 0 {
        return 1u8;
    }

    if SockAsyncCompletionPort == core::ptr::null_mut() {
        Status = NtCreateIoCompletion(
            &mut SockAsyncCompletionPort,
            IO_COMPLETION_ALL_ACCESS,
            core::ptr::null_mut(),
            2,
        );
        if !NT_SUCCESS(Status) {
            DbgPrint(b"Failed to create completion port: 0x%08x\n\0".as_ptr() as PCHAR, Status);
            return 0u8;
        }
        HandleFlags.ProtectFromClose = 1;
        HandleFlags.Inherit = 0;
        Status = NtSetInformationObject(
            SockAsyncCompletionPort,
            _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation,
            &mut HandleFlags as *mut _ as PVOID,
            core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as ULONG,
        );
    }

    Status = NtCreateEvent(
        &mut AsyncEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_NotificationEvent,
        0,
    );

    hAsyncThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(SockAsyncThread),
        core::ptr::null_mut(),
        0,
        &mut AsyncThreadId,
    );

    NtClose(hAsyncThread);

    SockAsyncThreadRefCount += 1;
    1u8
}

type LPSECURITY_ATTRIBUTES = PVOID;
type SIZE_T = usize;
type LPTHREAD_START_ROUTINE = Option<unsafe extern "system" fn(LPVOID) -> DWORD>;
#[no_mangle]
pub unsafe extern "system" fn SockAsyncThread(ThreadParam: PVOID) -> ULONG {
    let mut AsyncContext: PVOID = core::ptr::null_mut();
    let mut AsyncCompletionRoutine: PASYNC_COMPLETION_ROUTINE = None;
    let mut IOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Status: NTSTATUS;

    SetThreadPriority(GetCurrentThread(), (THREAD_PRIORITY_ABOVE_NORMAL as i32));

    loop {
        Status = NtRemoveIoCompletion(
            SockAsyncCompletionPort,
            &mut AsyncCompletionRoutine as *mut _ as *mut PVOID,
            &mut AsyncContext,
            &mut IOSB,
            core::ptr::null_mut(),
        );

        if NT_SUCCESS(Status) {
            (AsyncCompletionRoutine.unwrap())(AsyncContext, &mut IOSB);
        } else {
            Sleep(1000);
        }

        if Status == STATUS_TIMEOUT {
            break;
        }
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn SockGetAsyncSelectHelperAfdHandle() -> BOOLEAN {
    let mut AfdHelper: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoSb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut CompletionInfo: FILE_COMPLETION_INFORMATION = core::mem::zeroed();
    let mut HandleFlags: _OBJECT_HANDLE_FLAG_INFORMATION = core::mem::zeroed();

    if !SockAsyncHelperAfdHandle.is_null() {
        return 1u8;
    }

    RtlInitUnicodeString(&mut AfdHelper, w!("\\Device\\Afd\\AsyncSelectHlp").as_ptr() as LPWSTR);
    
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut AfdHelper;
    ObjectAttributes.Attributes = OBJ_INHERIT | OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    NtCreateFile(&mut SockAsyncHelperAfdHandle,
                 GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
                 &mut ObjectAttributes,
                 &mut IoSb,
                 core::ptr::null_mut(),
                 0,
                 FILE_SHARE_READ | FILE_SHARE_WRITE,
                 FILE_OPEN_IF,
                 0,
                 core::ptr::null_mut(),
                 0);

    CompletionInfo.Port = SockAsyncCompletionPort;
    CompletionInfo.Key = SockAsyncSelectCompletionRoutine as *mut _;
    NtSetInformationFile(SockAsyncHelperAfdHandle,
                         &mut IoSb,
                         &mut CompletionInfo as *mut _ as *mut _,
                         core::mem::size_of::<FILE_COMPLETION_INFORMATION>() as u32,
                         FileCompletionInformation);

    HandleFlags.ProtectFromClose = 1u8;
    HandleFlags.Inherit = 0u8;
    NtSetInformationObject(SockAsyncCompletionPort,
                           _OBJECT_INFORMATION_CLASS_ObjectHandleFlagInformation,
                           &mut HandleFlags as *mut _ as *mut _,
                           core::mem::size_of::<_OBJECT_HANDLE_FLAG_INFORMATION>() as u32);

    SockAsyncSelectCalled = 1u8;
    1u8
}
#[no_mangle]
pub unsafe extern "C" fn SockAsyncSelectCompletionRoutine(Context: PVOID, IoStatusBlock: PIO_STATUS_BLOCK) {
    let AsyncData = Context as PASYNC_DATA;
    let mut Socket: PSOCKET_INFORMATION;
    let mut x: ULONG;

    Socket = (*AsyncData).ParentSocket;

    if (*AsyncData).SequenceNumber != (*(*Socket).SharedData).SequenceNumber {
        return;
    }

    if !NT_SUCCESS((*IoStatusBlock).__bindgen_anon_1.Status) {
        return;
    }

    x = 1;
    while x != 0 {
        match (*(*AsyncData).AsyncSelectInfo.Handles.as_ptr()).Events & x {
            AFD_EVENT_RECEIVE => {
                if 0 != ((*(*Socket).SharedData).AsyncEvents & (FD_READ as i32))
                    && 0 == ((*(*Socket).SharedData).AsyncDisabledEvents & (FD_READ as i32))
                {
                    (Upcalls.lpWPUPostMessage.unwrap())(
                        (*(*Socket).SharedData).hWnd,
                        (*(*Socket).SharedData).wMsg,
                        (*Socket).Handle,
                        WSAMAKESELECTREPLY(FD_READ, 0) as _,
                    );
                    (*(*Socket).SharedData).AsyncDisabledEvents |= (FD_READ as i32);
                }
            }

            AFD_EVENT_OOB_RECEIVE => {
                if 0 != ((*(*Socket).SharedData).AsyncEvents & (FD_OOB as i32))
                    && 0 == ((*(*Socket).SharedData).AsyncDisabledEvents & (FD_OOB as i32))
                {
                    (Upcalls.lpWPUPostMessage.unwrap())(
                        (*(*Socket).SharedData).hWnd,
                        (*(*Socket).SharedData).wMsg,
                        (*Socket).Handle,
                        WSAMAKESELECTREPLY(FD_OOB, 0) as _,
                    );
                    (*(*Socket).SharedData).AsyncDisabledEvents |= (FD_OOB as i32);
                }
            }

            AFD_EVENT_SEND => {
                if 0 != ((*(*Socket).SharedData).AsyncEvents & (FD_WRITE as i32))
                    && 0 == ((*(*Socket).SharedData).AsyncDisabledEvents & (FD_WRITE as i32))
                {
                    (Upcalls.lpWPUPostMessage.unwrap())(
                        (*(*Socket).SharedData).hWnd,
                        (*(*Socket).SharedData).wMsg,
                        (*Socket).Handle,
                        WSAMAKESELECTREPLY(FD_WRITE, 0) as _,
                    );
                    (*(*Socket).SharedData).AsyncDisabledEvents |= (FD_WRITE as i32);
                }
            }

            AFD_EVENT_CONNECT | AFD_EVENT_CONNECT_FAIL => {
                if 0 != ((*(*Socket).SharedData).AsyncEvents & (FD_CONNECT as i32))
                    && 0 == ((*(*Socket).SharedData).AsyncDisabledEvents & (FD_CONNECT as i32))
                {
                    (Upcalls.lpWPUPostMessage.unwrap())(
                        (*(*Socket).SharedData).hWnd,
                        (*(*Socket).SharedData).wMsg,
                        (*Socket).Handle,
                        WSAMAKESELECTREPLY(FD_CONNECT, 0) as _,
                    );
                    (*(*Socket).SharedData).AsyncDisabledEvents |= (FD_CONNECT as i32);
                }
            }

            AFD_EVENT_ACCEPT => {
                if 0 != ((*(*Socket).SharedData).AsyncEvents & (FD_ACCEPT as i32))
                    && 0 == ((*(*Socket).SharedData).AsyncDisabledEvents & (FD_ACCEPT as i32))
                {
                    (Upcalls.lpWPUPostMessage.unwrap())(
                        (*(*Socket).SharedData).hWnd,
                        (*(*Socket).SharedData).wMsg,
                        (*Socket).Handle,
                        WSAMAKESELECTREPLY(FD_ACCEPT, 0) as _,
                    );
                    (*(*Socket).SharedData).AsyncDisabledEvents |= (FD_ACCEPT as i32);
                }
            }

            AFD_EVENT_DISCONNECT | AFD_EVENT_ABORT | AFD_EVENT_CLOSE => {
                if 0 != ((*(*Socket).SharedData).AsyncEvents & (FD_CLOSE as i32))
                    && 0 == ((*(*Socket).SharedData).AsyncDisabledEvents & (FD_CLOSE as i32))
                {
                    (Upcalls.lpWPUPostMessage.unwrap())(
                        (*(*Socket).SharedData).hWnd,
                        (*(*Socket).SharedData).wMsg,
                        (*Socket).Handle,
                        WSAMAKESELECTREPLY(FD_CLOSE, 0) as _,
                    );
                    (*(*Socket).SharedData).AsyncDisabledEvents |= (FD_CLOSE as i32);
                }
            }

            _ => {}
        }
        x <<= 1;
    }

    if ((*(*Socket).SharedData).AsyncEvents & (!(*(*Socket).SharedData).AsyncDisabledEvents)) == 0 {
        return;
    }

    SockProcessAsyncSelect(Socket, AsyncData);
    return;
}
#[no_mangle]
pub unsafe extern "C" fn SockProcessAsyncSelect(Socket: PSOCKET_INFORMATION, AsyncData: PASYNC_DATA) {
    let mut lNetworkEvents: ULONG;
    let Status: NTSTATUS;

    (*AsyncData).AsyncSelectInfo.Timeout.__bindgen_anon_1.HighPart = 0x7FFFFFFF;
    (*AsyncData).AsyncSelectInfo.Timeout.__bindgen_anon_1.LowPart = 0xFFFFFFFF;
    (*AsyncData).AsyncSelectInfo.HandleCount = 1;
    (*AsyncData).AsyncSelectInfo.Exclusive = (1u8 as _);
    (*AsyncData).AsyncSelectInfo.Handles[0].Handle = (*Socket).Handle;
    (*AsyncData).AsyncSelectInfo.Handles[0].Events = 0;

    lNetworkEvents = ((((*(*Socket).SharedData).AsyncEvents & ((!(*(*Socket).SharedData).AsyncDisabledEvents) as u32 as u32) as i32)) as u32);

    if (lNetworkEvents & FD_READ) != 0 {
        (*AsyncData).AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_RECEIVE;
    }

    if (lNetworkEvents & FD_WRITE) != 0 {
        (*AsyncData).AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_SEND;
    }

    if (lNetworkEvents & FD_OOB) != 0 {
        (*AsyncData).AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_OOB_RECEIVE;
    }

    if (lNetworkEvents & FD_ACCEPT) != 0 {
        (*AsyncData).AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_ACCEPT;
    }

    if (lNetworkEvents & FD_CONNECT) != 0 {
        (*AsyncData).AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_CONNECT | AFD_EVENT_CONNECT_FAIL;
    }

    if (lNetworkEvents & FD_CLOSE) != 0 {
        (*AsyncData).AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_DISCONNECT | AFD_EVENT_ABORT | AFD_EVENT_CLOSE;
    }

    if (lNetworkEvents & FD_QOS) != 0 {
        (*AsyncData).AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_QOS;
    }

    if (lNetworkEvents & FD_GROUP_QOS) != 0 {
        (*AsyncData).AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_GROUP_QOS;
    }

    Status = NtDeviceIoControlFile(
        SockAsyncHelperAfdHandle,
        core::ptr::null_mut(),
        None,
        AsyncData as PVOID,
        core::ptr::addr_of_mut!((*AsyncData).IoStatusBlock),
        IOCTL_AFD_SELECT,
        core::ptr::addr_of_mut!((*AsyncData).AsyncSelectInfo) as PVOID,
        core::mem::size_of_val(&(*AsyncData).AsyncSelectInfo) as ULONG,
        core::ptr::addr_of_mut!((*AsyncData).AsyncSelectInfo) as PVOID,
        core::mem::size_of_val(&(*AsyncData).AsyncSelectInfo) as ULONG,
    );

    if NT_SUCCESS(Status) {
        return;
    } else {
        (*AsyncData).IoStatusBlock.__bindgen_anon_1.Status = Status;
        SockAsyncSelectCompletionRoutine(AsyncData as PVOID, core::ptr::addr_of_mut!((*AsyncData).IoStatusBlock));
    }
}
#[no_mangle]
pub unsafe extern "C" fn SockProcessQueuedAsyncSelect(Context: PVOID, IoStatusBlock: PIO_STATUS_BLOCK) {
    let AsyncData = Context as PASYNC_DATA;
    let mut FreeContext = 1u8;
    let Socket: PSOCKET_INFORMATION;

    Socket = (*AsyncData).ParentSocket;

    if (*(*Socket).SharedData).State != SocketClosed {
        if (*AsyncData).SequenceNumber == (*(*Socket).SharedData).SequenceNumber {
            if ((*(*Socket).SharedData).AsyncEvents & (!(*(*Socket).SharedData).AsyncDisabledEvents)) != 0 {
                SockProcessAsyncSelect(Socket, AsyncData);
                FreeContext = 0u8;
            }
        }
    }

    if FreeContext != 0 {
        HeapFree(GetProcessHeap(), 0, AsyncData as PVOID);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SockReenableAsyncSelectEvent(
    Socket: PSOCKET_INFORMATION,
    Event: ULONG,
) {
    let AsyncData: PASYNC_DATA;

    if ((*(*Socket).SharedData).AsyncDisabledEvents & (Event as i32)) == 0 {
        return;
    }

    (*(*Socket).SharedData).AsyncDisabledEvents &= (!Event as i32);

    if ((*(*Socket).SharedData).AsyncEvents & (!(*(*Socket).SharedData).AsyncDisabledEvents)) == 0 {
        return;
    }

    AsyncData = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<ASYNC_DATA>()) as PASYNC_DATA;
    if AsyncData.is_null() {
        return;
    }

    SockCreateOrReferenceAsyncThread();

    (*(*Socket).SharedData).SequenceNumber += 1;

    (*AsyncData).ParentSocket = Socket;
    (*AsyncData).SequenceNumber = (*(*Socket).SharedData).SequenceNumber;

    NtSetIoCompletion(
        SockAsyncCompletionPort,
        SockProcessQueuedAsyncSelect as *const () as PVOID,
        AsyncData as PVOID,
        0,
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hInstDll: HANDLE,
    dwReason: ULONG,
    Reserved: PVOID,
) -> BOOL {

    match dwReason {
        DLL_PROCESS_ATTACH => {
            TRACE!(b"Loading MSAFD.DLL \n\0".as_ptr() as *const i8);

            DisableThreadLibraryCalls(hInstDll);

            InitializeListHead(core::ptr::addr_of_mut!(SockHelpersListHead));

            GlobalHeap = GetProcessHeap();

            InitializeCriticalSection(core::ptr::addr_of_mut!(SocketListLock));

            TRACE!(b"MSAFD.DLL has been loaded\n\0".as_ptr() as *const i8);
        }

        DLL_THREAD_ATTACH => {}

        DLL_THREAD_DETACH => {}

        DLL_PROCESS_DETACH => {
            DeleteCriticalSection(core::ptr::addr_of_mut!(SocketListLock));
        }

        _ => {}
    }

    TRACE!(b"DllMain of msafd.dll (leaving)\n\0".as_ptr() as *const i8);

    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Ancillary Function Driver DLL
//  * FILE:        dll/win32/msafd/misc/dllmain.c
//  * PURPOSE:     DLL entry point
//  * PROGRAMMERS: Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *              Alex Ionescu (alex@relsoft.net)
//  *              Pierre Schweitzer (pierre@reactos.org)
//  * REVISIONS:
//  *              CSH 01/09-2000 Created
//  *              Alex 16/07/2004 - Complete Rewrite
//  */
//
// #include <msafd.h>
//
// #include <winuser.h>
// #include <wchar.h>
//
// HANDLE GlobalHeap;
// WSPUPCALLTABLE Upcalls;
// DWORD CatalogEntryId; /* CatalogEntryId for upcalls */
// LPWPUCOMPLETEOVERLAPPEDREQUEST lpWPUCompleteOverlappedRequest;
// PSOCKET_INFORMATION SocketListHead = NULL;
// CRITICAL_SECTION SocketListLock;
// LIST_ENTRY SockHelpersListHead = { NULL, NULL };
// ULONG SockAsyncThreadRefCount;
// HANDLE SockAsyncHelperAfdHandle;
// HANDLE SockAsyncCompletionPort = NULL;
// BOOLEAN SockAsyncSelectCalled;
//
//
//
// /*
//  * FUNCTION: Creates a new socket
//  * ARGUMENTS:
//  *     af             = Address family
//  *     type           = Socket type
//  *     protocol       = Protocol type
//  *     lpProtocolInfo = Pointer to protocol information
//  *     g              = Reserved
//  *     dwFlags        = Socket flags
//  *     lpErrno        = Address of buffer for error information
//  * RETURNS:
//  *     Created socket, or INVALID_SOCKET if it could not be created
//  */
// SOCKET
// WSPAPI
// WSPSocket(int AddressFamily,
//           int SocketType,
//           int Protocol,
//           LPWSAPROTOCOL_INFOW lpProtocolInfo,
//           GROUP g,
//           DWORD dwFlags,
//           LPINT lpErrno)
// {
//     OBJECT_ATTRIBUTES           Object;
//     IO_STATUS_BLOCK             IOSB;
//     USHORT                      SizeOfPacket;
//     ULONG                       SizeOfEA;
//     PAFD_CREATE_PACKET          AfdPacket;
//     HANDLE                      Sock;
//     PSOCKET_INFORMATION         Socket = NULL;
//     PFILE_FULL_EA_INFORMATION   EABuffer = NULL;
//     PHELPER_DATA                HelperData;
//     PVOID                       HelperDLLContext;
//     DWORD                       HelperEvents;
//     UNICODE_STRING              TransportName;
//     UNICODE_STRING              DevName;
//     LARGE_INTEGER               GroupData;
//     INT                         Status;
//     PSOCK_SHARED_INFO           SharedData = NULL;
//
//     TRACE("Creating Socket, getting TDI Name - AddressFamily (%d)  SocketType (%d)  Protocol (%d).\n",
//         AddressFamily, SocketType, Protocol);
//
//     if (lpProtocolInfo && lpProtocolInfo->dwServiceFlags3 != 0 && lpProtocolInfo->dwServiceFlags4 != 0)
//     {
//         /* Duplpicating socket from different process */
//         if (UlongToPtr(lpProtocolInfo->dwServiceFlags3) == INVALID_HANDLE_VALUE)
//         {
//             Status = WSAEINVAL;
//             goto error;
//         }
//         if (UlongToPtr(lpProtocolInfo->dwServiceFlags4) == INVALID_HANDLE_VALUE)
//         {
//             Status = WSAEINVAL;
//             goto error;
//         }
//         SharedData = MapViewOfFile(UlongToPtr(lpProtocolInfo->dwServiceFlags3),
//                                    FILE_MAP_ALL_ACCESS,
//                                    0,
//                                    0,
//                                    sizeof(SOCK_SHARED_INFO));
//         if (!SharedData)
//         {
//             Status = WSAEINVAL;
//             goto error;
//         }
//         InterlockedIncrement(&SharedData->RefCount);
//         AddressFamily = SharedData->AddressFamily;
//         SocketType = SharedData->SocketType;
//         Protocol = SharedData->Protocol;
//     }
//
//     if (AddressFamily == AF_UNSPEC && SocketType == 0 && Protocol == 0)
//     {
//         Status = WSAEINVAL;
//         goto error;
//     }
//
//     /* Set the defaults */
//     if (AddressFamily == AF_UNSPEC)
//         AddressFamily = AF_INET;
//
//     if (SocketType == 0)
//     {
//         switch (Protocol)
//         {
//         case IPPROTO_TCP:
//             SocketType = SOCK_STREAM;
//             break;
//         case IPPROTO_UDP:
//             SocketType = SOCK_DGRAM;
//             break;
//         case IPPROTO_RAW:
//             SocketType = SOCK_RAW;
//             break;
//         default:
//             TRACE("Unknown Protocol (%d). We will try SOCK_STREAM.\n", Protocol);
//             SocketType = SOCK_STREAM;
//             break;
//         }
//     }
//
//     if (Protocol == 0)
//     {
//         switch (SocketType)
//         {
//         case SOCK_STREAM:
//             Protocol = IPPROTO_TCP;
//             break;
//         case SOCK_DGRAM:
//             Protocol = IPPROTO_UDP;
//             break;
//         case SOCK_RAW:
//             Protocol = IPPROTO_RAW;
//             break;
//         default:
//             TRACE("Unknown SocketType (%d). We will try IPPROTO_TCP.\n", SocketType);
//             Protocol = IPPROTO_TCP;
//             break;
//         }
//     }
//
//     /* Get Helper Data and Transport */
//     Status = SockGetTdiName (&AddressFamily,
//                              &SocketType,
//                              &Protocol,
//                              g,
//                              dwFlags,
//                              &TransportName,
//                              &HelperDLLContext,
//                              &HelperData,
//                              &HelperEvents);
//
//     /* Check for error */
//     if (Status != NO_ERROR)
//     {
//         ERR("SockGetTdiName: Status %x\n", Status);
//         goto error;
//     }
//
//     /* AFD Device Name */
//     RtlInitUnicodeString(&DevName, L"\\Device\\Afd\\Endpoint");
//
//     /* Set Socket Data */
//     Socket = HeapAlloc(GlobalHeap, 0, sizeof(*Socket));
//     if (!Socket)
//     {
//         Status = WSAENOBUFS;
//         goto error;
//     }
//     RtlZeroMemory(Socket, sizeof(*Socket));
//     if (SharedData)
//     {
//         Socket->SharedData = SharedData;
//         Socket->SharedDataHandle = UlongToHandle(lpProtocolInfo->dwServiceFlags3);
//         Sock = UlongToHandle(lpProtocolInfo->dwServiceFlags4);
//         Socket->Handle = (SOCKET)lpProtocolInfo->dwServiceFlags4;
//     }
//     else
//     {
//         Socket->SharedDataHandle = INVALID_HANDLE_VALUE;
//         Socket->SharedData = HeapAlloc(GlobalHeap, 0, sizeof(*Socket->SharedData));
//         if (!Socket->SharedData)
//         {
//             Status = WSAENOBUFS;
//             goto error;
//         }
//         RtlZeroMemory(Socket->SharedData, sizeof(*Socket->SharedData));
//         Socket->SharedData->State = SocketOpen;
//         Socket->SharedData->RefCount = 1L;
//         Socket->SharedData->Listening = FALSE;
//         Socket->SharedData->AddressFamily = AddressFamily;
//         Socket->SharedData->SocketType = SocketType;
//         Socket->SharedData->Protocol = Protocol;
//         Socket->SharedData->SizeOfLocalAddress = HelperData->MaxWSAddressLength;
//         Socket->SharedData->SizeOfRemoteAddress = HelperData->MaxWSAddressLength;
//         Socket->SharedData->UseDelayedAcceptance = HelperData->UseDelayedAcceptance;
//         Socket->SharedData->CreateFlags = dwFlags;
//         Socket->SharedData->ServiceFlags1 = lpProtocolInfo->dwServiceFlags1;
//         Socket->SharedData->ProviderFlags = lpProtocolInfo->dwProviderFlags;
//         Socket->SharedData->UseSAN = FALSE;
//         Socket->SharedData->NonBlocking = FALSE; /* Sockets start blocking */
//         Socket->SharedData->RecvTimeout = INFINITE;
//         Socket->SharedData->SendTimeout = INFINITE;
//         Socket->SharedData->OobInline = FALSE;
//
//         /* Ask alex about this */
//         if( Socket->SharedData->SocketType == SOCK_DGRAM ||
//             Socket->SharedData->SocketType == SOCK_RAW )
//         {
//             TRACE("Connectionless socket\n");
//             Socket->SharedData->ServiceFlags1 |= XP1_CONNECTIONLESS;
//         }
//         Socket->Handle = INVALID_SOCKET;
//     }
//
//     Socket->HelperContext = HelperDLLContext;
//     Socket->HelperData = HelperData;
//     Socket->HelperEvents = HelperEvents;
//     Socket->LocalAddress = &Socket->SharedData->WSLocalAddress;
//     Socket->RemoteAddress = &Socket->SharedData->WSRemoteAddress;
//     Socket->SanData = NULL;
//     RtlCopyMemory(&Socket->ProtocolInfo, lpProtocolInfo, sizeof(Socket->ProtocolInfo));
//     if (SharedData)
//         goto ok;
//
//     /* Packet Size */
//     SizeOfPacket = TransportName.Length + sizeof(AFD_CREATE_PACKET) + sizeof(WCHAR);
//
//     /* EA Size */
//     SizeOfEA = SizeOfPacket + sizeof(FILE_FULL_EA_INFORMATION) + AFD_PACKET_COMMAND_LENGTH;
//
//     /* Set up EA Buffer */
//     EABuffer = HeapAlloc(GlobalHeap, 0, SizeOfEA);
//     if (!EABuffer)
//     {
//         Status = WSAENOBUFS;
//         goto error;
//     }
//
//     RtlZeroMemory(EABuffer, SizeOfEA);
//     EABuffer->NextEntryOffset = 0;
//     EABuffer->Flags = 0;
//     EABuffer->EaNameLength = AFD_PACKET_COMMAND_LENGTH;
//     RtlCopyMemory (EABuffer->EaName,
//                    AfdCommand,
//                    AFD_PACKET_COMMAND_LENGTH + 1);
//     EABuffer->EaValueLength = SizeOfPacket;
//
//     /* Set up AFD Packet */
//     AfdPacket = (PAFD_CREATE_PACKET)(EABuffer->EaName + EABuffer->EaNameLength + 1);
//     AfdPacket->SizeOfTransportName = TransportName.Length;
//     RtlCopyMemory (AfdPacket->TransportName,
//                    TransportName.Buffer,
//                    TransportName.Length + sizeof(WCHAR));
//     AfdPacket->GroupID = g;
//
//     /* Set up Endpoint Flags */
//     if ((Socket->SharedData->ServiceFlags1 & XP1_CONNECTIONLESS) != 0)
//     {
//         if ((SocketType != SOCK_DGRAM) && (SocketType != SOCK_RAW))
//         {
//             /* Only RAW or UDP can be Connectionless */
//             Status = WSAEINVAL;
//             goto error;
//         }
//         AfdPacket->EndpointFlags |= AFD_ENDPOINT_CONNECTIONLESS;
//     }
//
//     if ((Socket->SharedData->ServiceFlags1 & XP1_MESSAGE_ORIENTED) != 0)
//     {
//         if (SocketType == SOCK_STREAM)
//         {
//             if ((Socket->SharedData->ServiceFlags1 & XP1_PSEUDO_STREAM) == 0)
//             {
//                 /* The Provider doesn't actually support Message Oriented Streams */
//                 Status = WSAEINVAL;
//                 goto error;
//             }
//         }
//         AfdPacket->EndpointFlags |= AFD_ENDPOINT_MESSAGE_ORIENTED;
//     }
//
//     if (SocketType == SOCK_RAW) AfdPacket->EndpointFlags |= AFD_ENDPOINT_RAW;
//
//     if (dwFlags & (WSA_FLAG_MULTIPOINT_C_ROOT |
//                    WSA_FLAG_MULTIPOINT_C_LEAF |
//                    WSA_FLAG_MULTIPOINT_D_ROOT |
//                    WSA_FLAG_MULTIPOINT_D_LEAF))
//     {
//         if ((Socket->SharedData->ServiceFlags1 & XP1_SUPPORT_MULTIPOINT) == 0)
//         {
//             /* The Provider doesn't actually support Multipoint */
//             Status = WSAEINVAL;
//             goto error;
//         }
//         AfdPacket->EndpointFlags |= AFD_ENDPOINT_MULTIPOINT;
//
//         if (dwFlags & WSA_FLAG_MULTIPOINT_C_ROOT)
//         {
//             if (((Socket->SharedData->ServiceFlags1 & XP1_MULTIPOINT_CONTROL_PLANE) == 0)
//                 || ((dwFlags & WSA_FLAG_MULTIPOINT_C_LEAF) != 0))
//             {
//                 /* The Provider doesn't support Control Planes, or you already gave a leaf */
//                 Status = WSAEINVAL;
//                 goto error;
//             }
//             AfdPacket->EndpointFlags |= AFD_ENDPOINT_C_ROOT;
//         }
//
//         if (dwFlags & WSA_FLAG_MULTIPOINT_D_ROOT)
//         {
//             if (((Socket->SharedData->ServiceFlags1 & XP1_MULTIPOINT_DATA_PLANE) == 0)
//                 || ((dwFlags & WSA_FLAG_MULTIPOINT_D_LEAF) != 0))
//             {
//                 /* The Provider doesn't support Data Planes, or you already gave a leaf */
//                 Status = WSAEINVAL;
//                 goto error;
//             }
//             AfdPacket->EndpointFlags |= AFD_ENDPOINT_D_ROOT;
//         }
//     }
//
//     /* Set up Object Attributes */
//     InitializeObjectAttributes (&Object,
//                                 &DevName,
//                                 OBJ_CASE_INSENSITIVE | OBJ_INHERIT,
//                                 0,
//                                 0);
//
//     /* Create the Socket as asynchronous. That means we have to block
//     ourselves after every call to NtDeviceIoControlFile. This is
//     because the kernel doesn't support overlapping synchronous I/O
//     requests (made from multiple threads) at this time (Sep 2005) */
//     Status = NtCreateFile(&Sock,
//                           GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
//                           &Object,
//                           &IOSB,
//                           NULL,
//                           0,
//                           FILE_SHARE_READ | FILE_SHARE_WRITE,
//                           FILE_OPEN_IF,
//                           0,
//                           EABuffer,
//                           SizeOfEA);
//
//     HeapFree(GlobalHeap, 0, EABuffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("Failed to open socket. Status 0x%08x\n", Status);
//         Status = TranslateNtStatusError(Status);
//         goto error;
//     }
//
//     /* Save Handle */
//     Socket->Handle = (SOCKET)Sock;
//
//     /* Save Group Info */
//     if (g != 0)
//     {
//         GetSocketInformation(Socket,
//                              AFD_INFO_GROUP_ID_TYPE,
//                              NULL,
//                              NULL,
//                              &GroupData,
//                              NULL,
//                              NULL);
//         Socket->SharedData->GroupID = GroupData.u.LowPart;
//         Socket->SharedData->GroupType = GroupData.u.HighPart;
//     }
//
//     /* Get Window Sizes and Save them */
//     GetSocketInformation (Socket,
//                           AFD_INFO_SEND_WINDOW_SIZE,
//                           NULL,
//                           &Socket->SharedData->SizeOfSendBuffer,
//                           NULL,
//                           NULL,
//                           NULL);
//
//     GetSocketInformation (Socket,
//                           AFD_INFO_RECEIVE_WINDOW_SIZE,
//                           NULL,
//                           &Socket->SharedData->SizeOfRecvBuffer,
//                           NULL,
//                           NULL,
//                           NULL);
// ok:
//
//     /* Save in Process Sockets List */
//     EnterCriticalSection(&SocketListLock);
//     Socket->NextSocket = SocketListHead;
//     SocketListHead = Socket;
//     LeaveCriticalSection(&SocketListLock);
//
//     /* Create the Socket Context */
//     CreateContext(Socket);
//
//     /* Notify Winsock */
//     Upcalls.lpWPUModifyIFSHandle(Socket->ProtocolInfo.dwCatalogEntryId, (SOCKET)Sock, lpErrno);
//
//     /* Return Socket Handle */
//     TRACE("Success %x\n", Sock);
//
//     return (SOCKET)Sock;
//
// error:
//     TRACE("Ending %x\n", Status);
//
//     if( SharedData )
//     {
//         UnmapViewOfFile(SharedData);
//         NtClose(UlongToHandle(lpProtocolInfo->dwServiceFlags3));
//     }
//     else
//     {
//         if( Socket && Socket->SharedData )
//             HeapFree(GlobalHeap, 0, Socket->SharedData);
//     }
//
//     if( Socket )
//         HeapFree(GlobalHeap, 0, Socket);
//
//     if( EABuffer )
//         HeapFree(GlobalHeap, 0, EABuffer);
//
//     if( lpErrno )
//         *lpErrno = Status;
//
//     return INVALID_SOCKET;
// }
//
//
// INT
// WSPAPI
// WSPDuplicateSocket(
//     IN  SOCKET Handle,
//     IN  DWORD dwProcessId,
//     OUT LPWSAPROTOCOL_INFOW lpProtocolInfo,
//     OUT LPINT lpErrno)
// {
//     HANDLE hProcess, hDuplicatedSharedData, hDuplicatedHandle;
//     PSOCKET_INFORMATION Socket;
//     PSOCK_SHARED_INFO pSharedData, pOldSharedData;
//     BOOL bDuplicated;
//
//     if (Handle == INVALID_SOCKET)
//         return MsafdReturnWithErrno(STATUS_INVALID_PARAMETER, lpErrno, 0, NULL);
//     Socket = GetSocketStructure(Handle);
//     if( !Socket )
//     {
//         if( lpErrno )
//             *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//     if ( !(hProcess = OpenProcess(PROCESS_DUP_HANDLE, FALSE, dwProcessId)) )
//         return MsafdReturnWithErrno(STATUS_INVALID_PARAMETER, lpErrno, 0, NULL);
//
//     /* It is a not yet duplicated socket, so map the memory, copy the SharedData and free heap */
//     if( Socket->SharedDataHandle == INVALID_HANDLE_VALUE )
//     {
//         Socket->SharedDataHandle = CreateFileMappingW(INVALID_HANDLE_VALUE,
//                                                      NULL,
//                                                      PAGE_READWRITE | SEC_COMMIT,
//                                                      0,
//                                                      (sizeof(SOCK_SHARED_INFO) + PAGE_SIZE - 1) & ~(PAGE_SIZE - 1),
//                                                      NULL);
//         if( Socket->SharedDataHandle == INVALID_HANDLE_VALUE )
//             return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, NULL);
//         pSharedData = MapViewOfFile(Socket->SharedDataHandle,
//                                     FILE_MAP_ALL_ACCESS,
//                                     0,
//                                     0,
//                                     sizeof(SOCK_SHARED_INFO));
//
//         RtlCopyMemory(pSharedData, Socket->SharedData, sizeof(SOCK_SHARED_INFO));
//         pOldSharedData = Socket->SharedData;
//         Socket->SharedData = pSharedData;
//         HeapFree(GlobalHeap, 0, pOldSharedData);
//     }
//     /* Duplicate the handles for the new process */
//     bDuplicated = DuplicateHandle(GetCurrentProcess(),
//                                   Socket->SharedDataHandle,
//                                   hProcess,
//                                   (LPHANDLE)&hDuplicatedSharedData,
//                                   0,
//                                   FALSE,
//                                   DUPLICATE_SAME_ACCESS);
//     if (!bDuplicated)
//     {
//         NtClose(hProcess);
//         return MsafdReturnWithErrno(STATUS_ACCESS_DENIED, lpErrno, 0, NULL);
//     }
//     bDuplicated = DuplicateHandle(GetCurrentProcess(),
//                                   (HANDLE)Socket->Handle,
//                                   hProcess,
//                                   (LPHANDLE)&hDuplicatedHandle,
//                                   0,
//                                   FALSE,
//                                   DUPLICATE_SAME_ACCESS);
//     NtClose(hProcess);
//     if( !bDuplicated )
//         return MsafdReturnWithErrno(STATUS_ACCESS_DENIED, lpErrno, 0, NULL);
//
//
//     if (!lpProtocolInfo)
//         return MsafdReturnWithErrno(STATUS_ACCESS_VIOLATION, lpErrno, 0, NULL);
//
//     RtlCopyMemory(lpProtocolInfo, &Socket->ProtocolInfo, sizeof(*lpProtocolInfo));
//
//     lpProtocolInfo->iAddressFamily = Socket->SharedData->AddressFamily;
//     lpProtocolInfo->iProtocol = Socket->SharedData->Protocol;
//     lpProtocolInfo->iSocketType = Socket->SharedData->SocketType;
//     lpProtocolInfo->dwServiceFlags3 = HandleToUlong(hDuplicatedSharedData);
//     lpProtocolInfo->dwServiceFlags4 = HandleToUlong(hDuplicatedHandle);
//
//     if( lpErrno )
//         *lpErrno = NO_ERROR;
//
//     return NO_ERROR;
// }
//
// INT
// TranslateNtStatusError(NTSTATUS Status)
// {
//     switch (Status)
//     {
//        case STATUS_CANT_WAIT:
//           return WSAEWOULDBLOCK;
//
//        case STATUS_TIMEOUT:
//           return WSAETIMEDOUT;
//
//        case STATUS_SUCCESS:
//           return NO_ERROR;
//
//        case STATUS_FILE_CLOSED:
//          return WSAECONNRESET;
//
//        case STATUS_END_OF_FILE:
//           return WSAESHUTDOWN;
//
//        case STATUS_PENDING:
//           return WSA_IO_PENDING;
//
//        case STATUS_BUFFER_TOO_SMALL:
//        case STATUS_BUFFER_OVERFLOW:
//           return WSAEMSGSIZE;
//
//        case STATUS_NO_MEMORY:
//        case STATUS_INSUFFICIENT_RESOURCES:
//           return WSAENOBUFS;
//
//        case STATUS_INVALID_CONNECTION:
//           return WSAENOTCONN;
//
//        case STATUS_PROTOCOL_NOT_SUPPORTED:
//           return WSAEAFNOSUPPORT;
//
//        case STATUS_INVALID_ADDRESS:
//           return WSAEADDRNOTAVAIL;
//
//        case STATUS_REMOTE_NOT_LISTENING:
//        case STATUS_REMOTE_DISCONNECT:
//           return WSAECONNREFUSED;
//
//        case STATUS_NETWORK_UNREACHABLE:
//           return WSAENETUNREACH;
//
//        case STATUS_HOST_UNREACHABLE:
//           return WSAEHOSTUNREACH;
//
//        case STATUS_INVALID_PARAMETER:
//           return WSAEINVAL;
//
//        case STATUS_CANCELLED:
//           return WSA_OPERATION_ABORTED;
//
//        case STATUS_ADDRESS_ALREADY_EXISTS:
//           return WSAEADDRINUSE;
//
//        case STATUS_LOCAL_DISCONNECT:
//           return WSAECONNABORTED;
//
//        case STATUS_ACCESS_VIOLATION:
//           return WSAEFAULT;
//
//        case STATUS_ACCESS_DENIED:
//           return WSAEACCES;
//
//        case STATUS_NOT_IMPLEMENTED:
//           return WSAEOPNOTSUPP;
//
//        default:
//           ERR("MSAFD: Unhandled NTSTATUS value: 0x%x\n", Status);
//           return WSAENETDOWN;
//     }
// }
//
// VOID
// MsafdWaitForAlert(
//      HANDLE hObject)
// {
//     DWORD Result;
//
//     for (;;)
//     {
//         Result = WaitForSingleObjectEx(hObject, INFINITE, TRUE);
//         if (Result != WAIT_IO_COMPLETION)
//             break;
//     }
// }
//
// /*
//  * FUNCTION: Closes an open socket
//  * ARGUMENTS:
//  *     s       = Socket descriptor
//  *     lpErrno = Address of buffer for error information
//  * RETURNS:
//  *     NO_ERROR, or SOCKET_ERROR if the socket could not be closed
//  */
// INT
// WSPAPI
// WSPCloseSocket(IN SOCKET Handle,
//                OUT LPINT lpErrno)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     PSOCKET_INFORMATION Socket = NULL, CurrentSocket;
//     NTSTATUS Status;
//     HANDLE SockEvent;
//     AFD_DISCONNECT_INFO DisconnectInfo;
//     SOCKET_STATE OldState;
//     LONG LingerWait = -1;
//     DWORD References;
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//        if (lpErrno) *lpErrno = WSAENOTSOCK;
//        return SOCKET_ERROR;
//     }
//
//     /* HACK: Allow APC to be processed */
//     SleepEx(0, TRUE);
//
//     if (Socket->HelperEvents & WSH_NOTIFY_CLOSE)
//     {
//         Status = Socket->HelperData->WSHNotify(Socket->HelperContext,
//                                                Socket->Handle,
//                                                Socket->TdiAddressHandle,
//                                                Socket->TdiConnectionHandle,
//                                                WSH_NOTIFY_CLOSE);
//
//         if (Status)
//         {
//             ERR("WSHNotify failed. Error 0x%#x\n", Status);
//             if (lpErrno) *lpErrno = Status;
//             return SOCKET_ERROR;
//         }
//     }
//
//     /* If a Close is already in Process, give up */
//     if (Socket->SharedData->State == SocketClosed)
//     {
//         WARN("Socket is closing.\n");
//         if (lpErrno) *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//
//     /* Create the Wait Event */
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR("NtCreateEvent failed: 0x%08x\n", Status);
//         return SOCKET_ERROR;
//     }
//
//     /* Decrement reference count on SharedData */
//     References = InterlockedDecrement(&Socket->SharedData->RefCount);
//     if (References)
//         goto ok;
//
//     /* Set the state to close */
//     OldState = Socket->SharedData->State;
//     Socket->SharedData->State = SocketClosed;
//
//     /* If SO_LINGER is ON and the Socket is connected, we need to disconnect */
//     /* FIXME: Should we do this on Datagram Sockets too? */
//     if ((OldState == SocketConnected) && (Socket->SharedData->LingerData.l_onoff))
//     {
//         ULONG SendsInProgress;
//         ULONG SleepWait;
//
//         /* We need to respect the timeout */
//         SleepWait = 100;
//         LingerWait = Socket->SharedData->LingerData.l_linger * 1000;
//
//         /* Loop until no more sends are pending, within the timeout */
//         while (LingerWait)
//         {
//             /* Find out how many Sends are in Progress */
//             if (GetSocketInformation(Socket,
//                                      AFD_INFO_SENDS_IN_PROGRESS,
//                                      NULL,
//                                      &SendsInProgress,
//                                      NULL,
//                                      NULL,
//                                      NULL))
//             {
//                 /* Bail out if anything but NO_ERROR */
//                 LingerWait = 0;
//                 break;
//             }
//
//             /* Bail out if no more sends are pending */
//             if (!SendsInProgress)
//             {
//                 LingerWait = -1;
//                 break;
//             }
//
//             /*
//              * We have to execute a sleep, so it's kind of like
//              * a block. If the socket is Nonblock, we cannot
//              * go on since asynchronous operation is expected
//              * and we cannot offer it
//              */
//             if (Socket->SharedData->NonBlocking)
//             {
//                 WARN("Would block!\n");
//                 NtClose(SockEvent);
//                 Socket->SharedData->State = OldState;
//                 if (lpErrno) *lpErrno = WSAEWOULDBLOCK;
//                 return SOCKET_ERROR;
//             }
//
//             /* Now we can sleep, and decrement the linger wait */
//             /*
//             * FIXME: It seems Windows does some funky acceleration
//             * since the waiting seems to be longer and longer. I
//             * don't think this improves performance so much, so we
//             * wait a fixed time instead.
//             */
//             Sleep(SleepWait);
//             LingerWait -= SleepWait;
//         }
//     }
//
//     if (OldState == SocketConnected)
//     {
//         if (LingerWait <= 0)
//         {
//             DisconnectInfo.Timeout = RtlConvertLongToLargeInteger(0);
//             DisconnectInfo.DisconnectType = LingerWait < 0 ? AFD_DISCONNECT_SEND : AFD_DISCONNECT_ABORT;
//
//             if (((DisconnectInfo.DisconnectType & AFD_DISCONNECT_SEND) && (!Socket->SharedData->SendShutdown)) ||
//                 ((DisconnectInfo.DisconnectType & AFD_DISCONNECT_ABORT) && (!Socket->SharedData->ReceiveShutdown)))
//             {
//                 /* Send IOCTL */
//                 Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                                SockEvent,
//                                                NULL,
//                                                NULL,
//                                                &IoStatusBlock,
//                                                IOCTL_AFD_DISCONNECT,
//                                                &DisconnectInfo,
//                                                sizeof(DisconnectInfo),
//                                                NULL,
//                                                0);
//
//                 /* Wait for return */
//                 if (Status == STATUS_PENDING)
//                 {
//                     MsafdWaitForAlert(SockEvent);
//                     Status = IoStatusBlock.Status;
//                 }
//             }
//         }
//     }
//
//     /* Cleanup Time! */
//     Socket->HelperContext = NULL;
//     Socket->SharedData->AsyncDisabledEvents = -1;
//     NtClose(Socket->TdiAddressHandle);
//     Socket->TdiAddressHandle = NULL;
//     NtClose(Socket->TdiConnectionHandle);
//     Socket->TdiConnectionHandle = NULL;
// ok:
//     EnterCriticalSection(&SocketListLock);
//     if (SocketListHead == Socket)
//     {
//         SocketListHead = SocketListHead->NextSocket;
//     }
//     else
//     {
//         CurrentSocket = SocketListHead;
//         while (CurrentSocket->NextSocket)
//         {
//             if (CurrentSocket->NextSocket == Socket)
//             {
//                 CurrentSocket->NextSocket = CurrentSocket->NextSocket->NextSocket;
//                 break;
//             }
//
//             CurrentSocket = CurrentSocket->NextSocket;
//         }
//     }
//     LeaveCriticalSection(&SocketListLock);
//
//     /* Close the handle */
//     NtClose((HANDLE)Handle);
//     NtClose(SockEvent);
//
//     if( Socket->SharedDataHandle != INVALID_HANDLE_VALUE )
//     {
//         /* It is a duplicated socket, so unmap the memory */
//         UnmapViewOfFile(Socket->SharedData);
//         NtClose(Socket->SharedDataHandle);
//         Socket->SharedData = NULL;
//     }
//     if( !References && Socket->SharedData )
//     {
//         HeapFree(GlobalHeap, 0, Socket->SharedData);
//     }
//     HeapFree(GlobalHeap, 0, Socket);
//     return MsafdReturnWithErrno(Status, lpErrno, 0, NULL);
// }
//
//
// /*
//  * FUNCTION: Associates a local address with a socket
//  * ARGUMENTS:
//  *     s       = Socket descriptor
//  *     name    = Pointer to local address
//  *     namelen = Length of name
//  *     lpErrno = Address of buffer for error information
//  * RETURNS:
//  *     0, or SOCKET_ERROR if the socket could not be bound
//  */
// INT
// WSPAPI
// WSPBind(SOCKET Handle,
//         const struct sockaddr *SocketAddress,
//         int SocketAddressLength,
//         LPINT lpErrno)
// {
//     IO_STATUS_BLOCK         IOSB;
//     PAFD_BIND_DATA          BindData;
//     PSOCKET_INFORMATION     Socket = NULL;
//     NTSTATUS                Status;
//     SOCKADDR_INFO           SocketInfo;
//     HANDLE                  SockEvent;
//     ULONG connectSize;
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//        if (lpErrno) *lpErrno = WSAENOTSOCK;
//        return SOCKET_ERROR;
//     }
//     if (Socket->SharedData->State != SocketOpen)
//     {
//        if (lpErrno) *lpErrno = WSAEINVAL;
//        return SOCKET_ERROR;
//     }
//     if (!SocketAddress || SocketAddressLength < Socket->SharedData->SizeOfLocalAddress)
//     {
//         if (lpErrno) *lpErrno = WSAEINVAL;
//         return SOCKET_ERROR;
//     }
//
//     /* Get Address Information */
//     Socket->HelperData->WSHGetSockaddrType ((PSOCKADDR)SocketAddress,
//                                             SocketAddressLength,
//                                             &SocketInfo);
//
//     if (SocketInfo.AddressInfo == SockaddrAddressInfoBroadcast && !Socket->SharedData->Broadcast)
//     {
//        if (lpErrno) *lpErrno = WSAEADDRNOTAVAIL;
//        return SOCKET_ERROR;
//     }
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return SOCKET_ERROR;
//     }
//
//     connectSize = FIELD_OFFSET(AFD_BIND_DATA, Address.Address[SocketAddressLength]);
//
//     /* See below */
//     BindData = HeapAlloc(GlobalHeap, 0, connectSize);
//     if (!BindData)
//     {
//         return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, NULL);
//     }
//
//     /* Set up Address in TDI Format */
//     BindData->Address.TAAddressCount = 1;
//     BindData->Address.Address[0].AddressLength = (USHORT)(SocketAddressLength - sizeof(SocketAddress->sa_family));
//     BindData->Address.Address[0].AddressType = SocketAddress->sa_family;
//     RtlCopyMemory (BindData->Address.Address[0].Address,
//                    SocketAddress->sa_data,
//                    SocketAddressLength - sizeof(SocketAddress->sa_family));
//
//     /* Set the Share Type */
//     if (Socket->SharedData->ExclusiveAddressUse)
//     {
//         BindData->ShareType = AFD_SHARE_EXCLUSIVE;
//     }
//     else if (SocketInfo.EndpointInfo == SockaddrEndpointInfoWildcard)
//     {
//         BindData->ShareType = AFD_SHARE_WILDCARD;
//     }
//     else if (Socket->SharedData->ReuseAddresses)
//     {
//         BindData->ShareType = AFD_SHARE_REUSE;
//     }
//     else
//     {
//         BindData->ShareType = AFD_SHARE_UNIQUE;
//     }
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_BIND,
//                                    BindData,
//                                    connectSize,
//                                    BindData,
//                                    connectSize);
//
//     /* Wait for return */
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     NtClose( SockEvent );
//     HeapFree(GlobalHeap, 0, BindData);
//
//     Socket->SharedData->SocketLastError = TranslateNtStatusError(Status);
//     if (Status != STATUS_SUCCESS)
//         return MsafdReturnWithErrno ( Status, lpErrno, 0, NULL );
//
//     /* Set up Socket Data */
//     Socket->SharedData->State = SocketBound;
//     Socket->TdiAddressHandle = (HANDLE)IOSB.Information;
//
//     if (Socket->HelperEvents & WSH_NOTIFY_BIND)
//     {
//         Status = Socket->HelperData->WSHNotify(Socket->HelperContext,
//                                                Socket->Handle,
//                                                Socket->TdiAddressHandle,
//                                                Socket->TdiConnectionHandle,
//                                                WSH_NOTIFY_BIND);
//
//         if (Status)
//         {
//             if (lpErrno) *lpErrno = Status;
//             return SOCKET_ERROR;
//         }
//     }
//
//     return MsafdReturnWithErrno ( Status, lpErrno, 0, NULL );
// }
//
// int
// WSPAPI
// WSPListen(SOCKET Handle,
//           int Backlog,
//           LPINT lpErrno)
// {
//     IO_STATUS_BLOCK         IOSB;
//     AFD_LISTEN_DATA         ListenData;
//     PSOCKET_INFORMATION     Socket = NULL;
//     HANDLE                  SockEvent;
//     NTSTATUS                Status;
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//        if (lpErrno) *lpErrno = WSAENOTSOCK;
//        return SOCKET_ERROR;
//     }
//
//     if (Socket->SharedData->Listening)
//         return NO_ERROR;
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//
//     if( !NT_SUCCESS(Status) )
//         return SOCKET_ERROR;
//
//     /* Set Up Listen Structure */
//     ListenData.UseSAN = FALSE;
//     ListenData.UseDelayedAcceptance = Socket->SharedData->UseDelayedAcceptance;
//     ListenData.Backlog = Backlog;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_START_LISTEN,
//                                    &ListenData,
//                                    sizeof(ListenData),
//                                    NULL,
//                                    0);
//
//     /* Wait for return */
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     NtClose( SockEvent );
//
//     Socket->SharedData->SocketLastError = TranslateNtStatusError(Status);
//     if (Status != STATUS_SUCCESS)
//        return MsafdReturnWithErrno ( Status, lpErrno, 0, NULL );
//
//     /* Set to Listening */
//     Socket->SharedData->Listening = TRUE;
//
//     if (Socket->HelperEvents & WSH_NOTIFY_LISTEN)
//     {
//         Status = Socket->HelperData->WSHNotify(Socket->HelperContext,
//                                                Socket->Handle,
//                                                Socket->TdiAddressHandle,
//                                                Socket->TdiConnectionHandle,
//                                                WSH_NOTIFY_LISTEN);
//
//         if (Status)
//         {
//            if (lpErrno) *lpErrno = Status;
//            return SOCKET_ERROR;
//         }
//     }
//
//     return MsafdReturnWithErrno ( Status, lpErrno, 0, NULL );
// }
//
//
// int
// WSPAPI
// WSPSelect(IN int nfds,
//           IN OUT fd_set *readfds OPTIONAL,
//           IN OUT fd_set *writefds OPTIONAL,
//           IN OUT fd_set *exceptfds OPTIONAL,
//           IN const struct timeval *timeout OPTIONAL,
//           OUT LPINT lpErrno)
// {
//     IO_STATUS_BLOCK     IOSB;
//     PAFD_POLL_INFO      PollInfo;
//     NTSTATUS            Status;
//     ULONG               HandleCount;
//     ULONG               PollBufferSize;
//     PVOID               PollBuffer;
//     ULONG               i, j = 0, x;
//     HANDLE              SockEvent;
//     LARGE_INTEGER       Timeout;
//     PSOCKET_INFORMATION Socket;
//     SOCKET              Handle;
//     ULONG               Events;
//     fd_set              selectfds;
//
//     /* Find out how many sockets we have, and how large the buffer needs
//      * to be */
//     FD_ZERO(&selectfds);
//     if (readfds != NULL)
//     {
//         for (i = 0; i < readfds->fd_count; i++)
//         {
//             FD_SET(readfds->fd_array[i], &selectfds);
//         }
//     }
//     if (writefds != NULL)
//     {
//         for (i = 0; i < writefds->fd_count; i++)
//         {
//             FD_SET(writefds->fd_array[i], &selectfds);
//         }
//     }
//     if (exceptfds != NULL)
//     {
//         for (i = 0; i < exceptfds->fd_count; i++)
//         {
//             FD_SET(exceptfds->fd_array[i], &selectfds);
//         }
//     }
//
//     HandleCount = selectfds.fd_count;
//
//     if ( HandleCount == 0 )
//     {
//         WARN("No handles! Returning SOCKET_ERROR\n", HandleCount);
//         if (lpErrno) *lpErrno = WSAEINVAL;
//         return SOCKET_ERROR;
//     }
//
//     PollBufferSize = sizeof(*PollInfo) + ((HandleCount - 1) * sizeof(AFD_HANDLE));
//
//     TRACE("HandleCount: %u BufferSize: %u\n", HandleCount, PollBufferSize);
//
//     /* Convert Timeout to NT Format */
//     if (timeout == NULL)
//     {
//         Timeout.u.LowPart = -1;
//         Timeout.u.HighPart = 0x7FFFFFFF;
//         TRACE("Infinite timeout\n");
//     }
//     else
//     {
//         Timeout = RtlEnlargedIntegerMultiply
//             ((timeout->tv_sec * 1000) + (timeout->tv_usec / 1000), -10000);
//         /* Negative timeouts are illegal.  Since the kernel represents an
//          * incremental timeout as a negative number, we check for a positive
//          * result.
//          */
//         if (Timeout.QuadPart > 0)
//         {
//             if (lpErrno) *lpErrno = WSAEINVAL;
//             return SOCKET_ERROR;
//         }
//         TRACE("Timeout: Orig %d.%06d kernel %d\n",
//                      timeout->tv_sec, timeout->tv_usec,
//                      Timeout.u.LowPart);
//     }
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//
//     if(!NT_SUCCESS(Status))
//     {
//         if (lpErrno)
//             *lpErrno = WSAEFAULT;
//
//         ERR("NtCreateEvent failed, 0x%08x\n", Status);
//         return SOCKET_ERROR;
//     }
//
//     /* Allocate */
//     PollBuffer = HeapAlloc(GlobalHeap, 0, PollBufferSize);
//
//     if (!PollBuffer)
//     {
//         if (lpErrno)
//             *lpErrno = WSAEFAULT;
//         NtClose(SockEvent);
//         return SOCKET_ERROR;
//     }
//
//     PollInfo = (PAFD_POLL_INFO)PollBuffer;
//
//     RtlZeroMemory( PollInfo, PollBufferSize );
//
//     /* Number of handles for AFD to Check */
//     PollInfo->Exclusive = FALSE;
//     PollInfo->Timeout = Timeout;
//
//     for (i = 0; i < selectfds.fd_count; i++)
//     {
//         PollInfo->Handles[i].Handle = selectfds.fd_array[i];
//     }
//     if (readfds != NULL) {
//         for (i = 0; i < readfds->fd_count; i++)
//         {
//             for (j = 0; j < HandleCount; j++)
//             {
//                 if (PollInfo->Handles[j].Handle == readfds->fd_array[i])
//                     break;
//             }
//             if (j >= HandleCount)
//             {
//                 ERR("Error while counting readfds %ld > %ld\n", j, HandleCount);
//                 if (lpErrno) *lpErrno = WSAEFAULT;
//                 HeapFree(GlobalHeap, 0, PollBuffer);
//                 NtClose(SockEvent);
//                 return SOCKET_ERROR;
//             }
//             Socket = GetSocketStructure(readfds->fd_array[i]);
//             if (!Socket)
//             {
//                 ERR("Invalid socket handle provided in readfds %d\n", readfds->fd_array[i]);
//                 if (lpErrno) *lpErrno = WSAENOTSOCK;
//                 HeapFree(GlobalHeap, 0, PollBuffer);
//                 NtClose(SockEvent);
//                 return SOCKET_ERROR;
//             }
//             PollInfo->Handles[j].Events |= AFD_EVENT_RECEIVE |
//                                            AFD_EVENT_DISCONNECT |
//                                            AFD_EVENT_ABORT |
//                                            AFD_EVENT_CLOSE |
//                                            AFD_EVENT_ACCEPT;
//             //if (Socket->SharedData->OobInline != 0)
//             //    PollInfo->Handles[j].Events |= AFD_EVENT_OOB_RECEIVE;
//         }
//     }
//     if (writefds != NULL)
//     {
//         for (i = 0; i < writefds->fd_count; i++)
//         {
//             for (j = 0; j < HandleCount; j++)
//             {
//                 if (PollInfo->Handles[j].Handle == writefds->fd_array[i])
//                     break;
//             }
//             if (j >= HandleCount)
//             {
//                 ERR("Error while counting writefds %ld > %ld\n", j, HandleCount);
//                 if (lpErrno) *lpErrno = WSAEFAULT;
//                 HeapFree(GlobalHeap, 0, PollBuffer);
//                 NtClose(SockEvent);
//                 return SOCKET_ERROR;
//             }
//             Socket = GetSocketStructure(writefds->fd_array[i]);
//             if (!Socket)
//             {
//                 ERR("Invalid socket handle provided in writefds %d\n", writefds->fd_array[i]);
//                 if (lpErrno) *lpErrno = WSAENOTSOCK;
//                 HeapFree(GlobalHeap, 0, PollBuffer);
//                 NtClose(SockEvent);
//                 return SOCKET_ERROR;
//             }
//             PollInfo->Handles[j].Handle = writefds->fd_array[i];
//             PollInfo->Handles[j].Events |= AFD_EVENT_SEND;
//             if (Socket->SharedData->NonBlocking != 0)
//                 PollInfo->Handles[j].Events |= AFD_EVENT_CONNECT;
//         }
//     }
//     if (exceptfds != NULL)
//     {
//         for (i = 0; i < exceptfds->fd_count; i++)
//         {
//             for (j = 0; j < HandleCount; j++)
//             {
//                 if (PollInfo->Handles[j].Handle == exceptfds->fd_array[i])
//                     break;
//             }
//             if (j > HandleCount)
//             {
//                 ERR("Error while counting exceptfds %ld > %ld\n", j, HandleCount);
//                 if (lpErrno) *lpErrno = WSAEFAULT;
//                 HeapFree(GlobalHeap, 0, PollBuffer);
//                 NtClose(SockEvent);
//                 return SOCKET_ERROR;
//             }
//             Socket = GetSocketStructure(exceptfds->fd_array[i]);
//             if (!Socket)
//             {
//                 TRACE("Invalid socket handle provided in exceptfds %d\n", exceptfds->fd_array[i]);
//                 if (lpErrno) *lpErrno = WSAENOTSOCK;
//                 HeapFree(GlobalHeap, 0, PollBuffer);
//                 NtClose(SockEvent);
//                 return SOCKET_ERROR;
//             }
//             PollInfo->Handles[j].Handle = exceptfds->fd_array[i];
//             if (Socket->SharedData->OobInline == 0)
//                 PollInfo->Handles[j].Events |= AFD_EVENT_OOB_RECEIVE;
//             if (Socket->SharedData->NonBlocking != 0)
//                 PollInfo->Handles[j].Events |= AFD_EVENT_CONNECT_FAIL;
//         }
//     }
//
//     PollInfo->HandleCount = HandleCount;
//     PollBufferSize = FIELD_OFFSET(AFD_POLL_INFO, Handles) + PollInfo->HandleCount * sizeof(AFD_HANDLE);
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)PollInfo->Handles[0].Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_SELECT,
//                                    PollInfo,
//                                    PollBufferSize,
//                                    PollInfo,
//                                    PollBufferSize);
//
//     TRACE("DeviceIoControlFile => %x\n", Status);
//
//     /* Wait for Completion */
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     /* Clear the Structures */
//     if( readfds )
//         FD_ZERO(readfds);
//     if( writefds )
//         FD_ZERO(writefds);
//     if( exceptfds )
//         FD_ZERO(exceptfds);
//
//     /* Loop through return structure */
//     HandleCount = PollInfo->HandleCount;
//
//     /* Return in FDSET Format */
//     for (i = 0; i < HandleCount; i++)
//     {
//         Events = PollInfo->Handles[i].Events;
//         Handle = PollInfo->Handles[i].Handle;
//         for(x = 1; x; x<<=1)
//         {
//             Socket = GetSocketStructure(Handle);
//             if (!Socket)
//             {
//                 TRACE("Invalid socket handle found %d\n", Handle);
//                 if (lpErrno) *lpErrno = WSAENOTSOCK;
//                 HeapFree(GlobalHeap, 0, PollBuffer);
//                 NtClose(SockEvent);
//                 return SOCKET_ERROR;
//             }
//             switch (Events & x)
//             {
//                 case AFD_EVENT_RECEIVE:
//                 case AFD_EVENT_DISCONNECT:
//                 case AFD_EVENT_ABORT:
//                 case AFD_EVENT_ACCEPT:
//                 case AFD_EVENT_CLOSE:
//                     TRACE("Event %x on handle %x\n",
//                         Events,
//                         Handle);
//                     if ((Events & x) == AFD_EVENT_DISCONNECT || (Events & x) == AFD_EVENT_CLOSE)
//                         Socket->SharedData->SocketLastError = WSAECONNRESET;
//                     if ((Events & x) == AFD_EVENT_ABORT)
//                         Socket->SharedData->SocketLastError = WSAECONNABORTED;
//                     if( readfds )
//                         FD_SET(Handle, readfds);
//                     break;
//                 case AFD_EVENT_SEND:
//                     TRACE("Event %x on handle %x\n",
//                         Events,
//                         Handle);
//                     if (writefds)
//                         FD_SET(Handle, writefds);
//                     break;
//                 case AFD_EVENT_CONNECT:
//                     TRACE("Event %x on handle %x\n",
//                         Events,
//                         Handle);
//                     if( writefds && Socket->SharedData->NonBlocking != 0 )
//                         FD_SET(Handle, writefds);
//                     break;
//                 case AFD_EVENT_OOB_RECEIVE:
//                     TRACE("Event %x on handle %x\n",
//                         Events,
//                         Handle);
//                     if( readfds && Socket->SharedData->OobInline != 0 )
//                         FD_SET(Handle, readfds);
//                     if( exceptfds && Socket->SharedData->OobInline == 0 )
//                         FD_SET(Handle, exceptfds);
//                     break;
//                 case AFD_EVENT_CONNECT_FAIL:
//                     TRACE("Event %x on handle %x\n",
//                         Events,
//                         Handle);
//                     if( exceptfds && Socket->SharedData->NonBlocking != 0 )
//                         FD_SET(Handle, exceptfds);
//                     break;
//             }
//         }
//     }
//
//     HeapFree( GlobalHeap, 0, PollBuffer );
//     NtClose( SockEvent );
//
//     if( lpErrno )
//     {
//         switch( IOSB.Status )
//         {
//             case STATUS_SUCCESS:
//             case STATUS_TIMEOUT:
//                 *lpErrno = 0;
//                 break;
//             default:
//                 *lpErrno = WSAEINVAL;
//                 break;
//         }
//         TRACE("*lpErrno = %x\n", *lpErrno);
//     }
//
//     HandleCount = (readfds ? readfds->fd_count : 0) +
//                   (writefds && writefds != readfds ? writefds->fd_count : 0) +
//                   (exceptfds && exceptfds != readfds && exceptfds != writefds ? exceptfds->fd_count : 0);
//
//     TRACE("%d events\n", HandleCount);
//
//     return HandleCount;
// }
//
// DWORD
// GetCurrentTimeInSeconds(VOID)
// {
//     SYSTEMTIME st1970 = { 1970, 1, 0, 1, 0, 0, 0, 0 };
//     union
//     {
//         FILETIME ft;
//         ULONGLONG ll;
//     } u1970, Time;
//
//     GetSystemTimeAsFileTime(&Time.ft);
//     SystemTimeToFileTime(&st1970, &u1970.ft);
//     return (DWORD)((Time.ll - u1970.ll) / 10000000ULL);
// }
//
//
// SOCKET
// WSPAPI
// WSPAccept(
//      SOCKET Handle,
//      struct sockaddr FAR *SocketAddress,
//      LPINT SocketAddressLength,
//      LPCONDITIONPROC lpfnCondition,
//      DWORD_PTR dwCallbackData,
//      LPINT lpErrno)
// {
//     IO_STATUS_BLOCK             IOSB;
//     PAFD_RECEIVED_ACCEPT_DATA   ListenReceiveData;
//     AFD_ACCEPT_DATA             AcceptData;
//     AFD_DEFER_ACCEPT_DATA       DeferData;
//     AFD_PENDING_ACCEPT_DATA     PendingAcceptData;
//     PSOCKET_INFORMATION         Socket = NULL;
//     NTSTATUS                    Status;
//     struct fd_set               ReadSet;
//     struct timeval              Timeout;
//     PVOID                       PendingData = NULL;
//     ULONG                       PendingDataLength = 0;
//     PVOID                       CalleeDataBuffer;
//     WSABUF                      CallerData, CalleeID, CallerID, CalleeData;
//     PSOCKADDR                   RemoteAddress =  NULL;
//     GROUP                       GroupID = 0;
//     ULONG                       CallBack;
//     SOCKET                      AcceptSocket;
//     PSOCKET_INFORMATION         AcceptSocketInfo;
//     UCHAR                       ReceiveBuffer[0x1A];
//     HANDLE                      SockEvent;
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//        if (lpErrno) *lpErrno = WSAENOTSOCK;
//        return SOCKET_ERROR;
//     }
//     if (!Socket->SharedData->Listening)
//     {
//        if (lpErrno) *lpErrno = WSAEINVAL;
//        return SOCKET_ERROR;
//     }
//     if ((SocketAddress && !SocketAddressLength) ||
//         (SocketAddressLength && !SocketAddress) ||
//         (SocketAddressLength && *SocketAddressLength < sizeof(SOCKADDR)))
//     {
//        if (lpErrno) *lpErrno = WSAEFAULT;
//        return INVALID_SOCKET;
//     }
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//
//     if( !NT_SUCCESS(Status) )
//     {
//         return SOCKET_ERROR;
//     }
//
//     /* Dynamic Structure...ugh */
//     ListenReceiveData = (PAFD_RECEIVED_ACCEPT_DATA)ReceiveBuffer;
//
//     /* If this is non-blocking, make sure there's something for us to accept */
//     if (Socket->SharedData->NonBlocking)
//     {
//         FD_ZERO(&ReadSet);
//         FD_SET(Socket->Handle, &ReadSet);
//         Timeout.tv_sec=0;
//         Timeout.tv_usec=0;
//
//         if (WSPSelect(0, &ReadSet, NULL, NULL, &Timeout, lpErrno) == SOCKET_ERROR)
//         {
//             NtClose(SockEvent);
//             return SOCKET_ERROR;
//         }
//
//         if (ReadSet.fd_array[0] != Socket->Handle)
//         {
//             NtClose(SockEvent);
//             if (lpErrno) *lpErrno = WSAEWOULDBLOCK;
//             return SOCKET_ERROR;
//         }
//     }
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_WAIT_FOR_LISTEN,
//                                    NULL,
//                                    0,
//                                    ListenReceiveData,
//                                    0xA + sizeof(*ListenReceiveData));
//
//     /* Wait for return */
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         NtClose( SockEvent );
//         return MsafdReturnWithErrno( Status, lpErrno, 0, NULL );
//     }
//
//     if (lpfnCondition != NULL)
//     {
//         if ((Socket->SharedData->ServiceFlags1 & XP1_CONNECT_DATA) != 0)
//         {
//             /* Find out how much data is pending */
//             PendingAcceptData.SequenceNumber = ListenReceiveData->SequenceNumber;
//             PendingAcceptData.ReturnSize = TRUE;
//
//             /* Send IOCTL */
//             Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                            SockEvent,
//                                            NULL,
//                                            NULL,
//                                            &IOSB,
//                                            IOCTL_AFD_GET_PENDING_CONNECT_DATA,
//                                            &PendingAcceptData,
//                                            sizeof(PendingAcceptData),
//                                            &PendingAcceptData,
//                                            sizeof(PendingAcceptData));
//
//             /* Wait for return */
//             if (Status == STATUS_PENDING)
//             {
//                 MsafdWaitForAlert(SockEvent);
//                 Status = IOSB.Status;
//             }
//
//             if (!NT_SUCCESS(Status))
//             {
//                 NtClose( SockEvent );
//                 return MsafdReturnWithErrno( Status, lpErrno, 0, NULL );
//             }
//
//             /* How much data to allocate */
//             PendingDataLength = IOSB.Information;
//
//             if (PendingDataLength)
//             {
//                 /* Allocate needed space */
//                 PendingData = HeapAlloc(GlobalHeap, 0, PendingDataLength);
//                 if (!PendingData)
//                 {
//                     return MsafdReturnWithErrno( STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, NULL );
//                 }
//
//                 /* We want the data now */
//                 PendingAcceptData.ReturnSize = FALSE;
//
//                 /* Send IOCTL */
//                 Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                                SockEvent,
//                                                NULL,
//                                                NULL,
//                                                &IOSB,
//                                                IOCTL_AFD_GET_PENDING_CONNECT_DATA,
//                                                &PendingAcceptData,
//                                                sizeof(PendingAcceptData),
//                                                PendingData,
//                                                PendingDataLength);
//
//                 /* Wait for return */
//                 if (Status == STATUS_PENDING)
//                 {
//                     MsafdWaitForAlert(SockEvent);
//                     Status = IOSB.Status;
//                 }
//
//                 if (!NT_SUCCESS(Status))
//                 {
//                     NtClose( SockEvent );
//                     return MsafdReturnWithErrno( Status, lpErrno, 0, NULL );
//                 }
//             }
//         }
//
//         if ((Socket->SharedData->ServiceFlags1 & XP1_QOS_SUPPORTED) != 0)
//         {
//             /* I don't support this yet */
//         }
//
//         /* Build Callee ID */
//         CalleeID.buf = (PVOID)Socket->LocalAddress;
//         CalleeID.len = Socket->SharedData->SizeOfLocalAddress;
//
//         RemoteAddress = HeapAlloc(GlobalHeap, 0, sizeof(*RemoteAddress));
//         if (!RemoteAddress)
//         {
//             return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, NULL);
//         }
//
//         /* Set up Address in SOCKADDR Format */
//         RtlCopyMemory (RemoteAddress,
//                        &ListenReceiveData->Address.Address[0].AddressType,
//                        sizeof(*RemoteAddress));
//
//         /* Build Caller ID */
//         CallerID.buf = (PVOID)RemoteAddress;
//         CallerID.len = sizeof(*RemoteAddress);
//
//         /* Build Caller Data */
//         CallerData.buf = PendingData;
//         CallerData.len = PendingDataLength;
//
//         /* Check if socket supports Conditional Accept */
//         if (Socket->SharedData->UseDelayedAcceptance != 0)
//         {
//             /* Allocate Buffer for Callee Data */
//             CalleeDataBuffer = HeapAlloc(GlobalHeap, 0, 4096);
//             if (!CalleeDataBuffer) {
//                 return MsafdReturnWithErrno( STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, NULL );
//             }
//             CalleeData.buf = CalleeDataBuffer;
//             CalleeData.len = 4096;
//         }
//         else
//         {
//             /* Nothing */
//             CalleeData.buf = 0;
//             CalleeData.len = 0;
//         }
//
//         /* Call the Condition Function */
//         CallBack = (lpfnCondition)(&CallerID,
//                                    CallerData.buf == NULL ? NULL : &CallerData,
//                                    NULL,
//                                    NULL,
//                                    &CalleeID,
//                                    CalleeData.buf == NULL ? NULL : &CalleeData,
//                                    &GroupID,
//                                    dwCallbackData);
//
//         if (((CallBack == CF_ACCEPT) && GroupID) != 0)
//         {
//             /* TBD: Check for Validity */
//         }
//
//         if (CallBack == CF_ACCEPT)
//         {
//             if ((Socket->SharedData->ServiceFlags1 & XP1_QOS_SUPPORTED) != 0)
//             {
//                 /* I don't support this yet */
//             }
//             if (CalleeData.buf)
//             {
//                 // SockSetConnectData Sockets(SocketID), IOCTL_AFD_SET_CONNECT_DATA, CalleeData.Buffer, CalleeData.BuffSize, 0
//             }
//         }
//         else
//         {
//             /* Callback rejected. Build Defer Structure */
//             DeferData.SequenceNumber = ListenReceiveData->SequenceNumber;
//             DeferData.RejectConnection = (CallBack == CF_REJECT);
//
//             /* Send IOCTL */
//             Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                            SockEvent,
//                                            NULL,
//                                            NULL,
//                                            &IOSB,
//                                            IOCTL_AFD_DEFER_ACCEPT,
//                                            &DeferData,
//                                            sizeof(DeferData),
//                                            NULL,
//                                            0);
//
//             /* Wait for return */
//             if (Status == STATUS_PENDING)
//             {
//                 MsafdWaitForAlert(SockEvent);
//                 Status = IOSB.Status;
//             }
//
//             NtClose( SockEvent );
//
//             if (!NT_SUCCESS(Status))
//             {
//                 return MsafdReturnWithErrno( Status, lpErrno, 0, NULL );
//             }
//
//             if (CallBack == CF_REJECT )
//             {
//                 if (lpErrno) *lpErrno = WSAECONNREFUSED;
//                 return SOCKET_ERROR;
//             }
//             else
//             {
//                 if (lpErrno) *lpErrno = WSAECONNREFUSED;
//                 return SOCKET_ERROR;
//             }
//         }
//     }
//
//     /* Create a new Socket */
//     AcceptSocket = WSPSocket (Socket->SharedData->AddressFamily,
//                               Socket->SharedData->SocketType,
//                               Socket->SharedData->Protocol,
//                               &Socket->ProtocolInfo,
//                               GroupID,
//                               Socket->SharedData->CreateFlags,
//                               lpErrno);
//     if (AcceptSocket == INVALID_SOCKET)
//         return SOCKET_ERROR;
//
//     /* Set up the Accept Structure */
//     AcceptData.ListenHandle = (HANDLE)AcceptSocket;
//     AcceptData.SequenceNumber = ListenReceiveData->SequenceNumber;
//
//     /* Send IOCTL to Accept */
//     Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_ACCEPT,
//                                    &AcceptData,
//                                    sizeof(AcceptData),
//                                    NULL,
//                                    0);
//
//     /* Wait for return */
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     Socket->SharedData->SocketLastError = TranslateNtStatusError(Status);
//     if (!NT_SUCCESS(Status))
//     {
//         NtClose(SockEvent);
//         WSPCloseSocket( AcceptSocket, lpErrno );
//         return MsafdReturnWithErrno( Status, lpErrno, 0, NULL );
//     }
//
//     AcceptSocketInfo = GetSocketStructure(AcceptSocket);
//     if (!AcceptSocketInfo)
//     {
//         NtClose(SockEvent);
//         WSPCloseSocket( AcceptSocket, lpErrno );
//         return MsafdReturnWithErrno( STATUS_PROTOCOL_NOT_SUPPORTED, lpErrno, 0, NULL );
//     }
//
//     AcceptSocketInfo->SharedData->State = SocketConnected;
//     AcceptSocketInfo->SharedData->ConnectTime = GetCurrentTimeInSeconds();
//     AcceptSocketInfo->SharedData->NonBlocking = Socket->SharedData->NonBlocking;
//
//     /* Return Address in SOCKADDR FORMAT */
//     if( SocketAddress )
//     {
//         RtlCopyMemory (SocketAddress,
//                        &ListenReceiveData->Address.Address[0].AddressType,
//                        sizeof(*RemoteAddress));
//         if( SocketAddressLength )
//             *SocketAddressLength = sizeof(*RemoteAddress);
//     }
//
//     NtClose( SockEvent );
//
//     /* Re-enable Async Event */
//     SockReenableAsyncSelectEvent(Socket, FD_ACCEPT);
//
//     TRACE("Socket %x\n", AcceptSocket);
//
//     if (Status == STATUS_SUCCESS && (Socket->HelperEvents & WSH_NOTIFY_ACCEPT))
//     {
//         Status = Socket->HelperData->WSHNotify(Socket->HelperContext,
//                                                Socket->Handle,
//                                                Socket->TdiAddressHandle,
//                                                Socket->TdiConnectionHandle,
//                                                WSH_NOTIFY_ACCEPT);
//
//         if (Status)
//         {
//             if (lpErrno) *lpErrno = Status;
//             return SOCKET_ERROR;
//         }
//     }
//
//     if (lpErrno) *lpErrno = NO_ERROR;
//
//     /* Return Socket */
//     return AcceptSocket;
// }
//
// static
// VOID
// NTAPI
// MsafdConnectAPC(
//      PVOID ApcContext,
//      PIO_STATUS_BLOCK IoStatusBlock,
//      ULONG Reserved)
// {
//     PMSAFD_CONNECT_APC_CONTEXT Context = ApcContext;
//
//     TRACE("MsafdConnectAPC(%p %lx %lx)\n", ApcContext, IoStatusBlock->Status, IoStatusBlock->Information);
//
//     PSOCKET_INFORMATION Socket = GetSocketStructure(Context->lpSocket);
//     if (!Socket)
//     {
//         // FIXME: Socket is closed before this APC could run
//         HeapFree(GlobalHeap, 0, ApcContext);
//         return;
//     }
//
//     Socket->SharedData->SocketLastError = TranslateNtStatusError(IoStatusBlock->Status);
//     if (IoStatusBlock->Status == STATUS_SUCCESS)
//     {
//         Socket->SharedData->State = SocketConnected;
//         Socket->TdiConnectionHandle = (HANDLE)IoStatusBlock->Information;
//         Socket->SharedData->ConnectTime = GetCurrentTimeInSeconds();
//     }
//
//     /* Re-enable Async Event */
//     SockReenableAsyncSelectEvent(Socket, FD_WRITE);
//
//     /* FIXME: THIS IS NOT RIGHT!!! HACK HACK HACK! */
//     SockReenableAsyncSelectEvent(Socket, FD_CONNECT);
//
//     if (IoStatusBlock->Status == STATUS_SUCCESS && (Socket->HelperEvents & WSH_NOTIFY_CONNECT))
//     {
//         Socket->HelperData->WSHNotify(Socket->HelperContext,
//                                       Socket->Handle,
//                                       Socket->TdiAddressHandle,
//                                       Socket->TdiConnectionHandle,
//                                       WSH_NOTIFY_CONNECT);
//     }
//     else if (IoStatusBlock->Status != STATUS_SUCCESS && (Socket->HelperEvents & WSH_NOTIFY_CONNECT_ERROR))
//     {
//         Socket->HelperData->WSHNotify(Socket->HelperContext,
//                                       Socket->Handle,
//                                       Socket->TdiAddressHandle,
//                                       Socket->TdiConnectionHandle,
//                                       WSH_NOTIFY_CONNECT_ERROR);
//     }
//
//     HeapFree(GlobalHeap, 0, ApcContext);
// }
//
// int
// WSPAPI
// WSPConnect(SOCKET Handle,
//            const struct sockaddr * SocketAddress,
//            int SocketAddressLength,
//            LPWSABUF lpCallerData,
//            LPWSABUF lpCalleeData,
//            LPQOS lpSQOS,
//            LPQOS lpGQOS,
//            LPINT lpErrno)
// {
//     IO_STATUS_BLOCK            DummyIOSB;
//     PIO_STATUS_BLOCK           IOSB = &DummyIOSB;
//     PAFD_CONNECT_INFO          ConnectInfo = NULL;
//     PSOCKET_INFORMATION        Socket;
//     NTSTATUS                   Status;
//     INT                        Errno;
//     ULONG                      ConnectDataLength;
//     ULONG                      InConnectDataLength;
//     HANDLE                     SockEvent;
//     int                        SocketDataLength;
//     PMSAFD_CONNECT_APC_CONTEXT APCContext = NULL;
//     PIO_APC_ROUTINE            APCFunction = NULL;
//     UCHAR                      Buffer[128];
//
//     TRACE("WSPConnect(%x)\n", Handle);
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//         if (lpErrno) *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//
//     /* Bind us First */
//     if (Socket->SharedData->State == SocketOpen)
//     {
//         INT BindAddressLength;
//         PSOCKADDR BindAddress;
//         INT BindError;
//
//         /* Get the Wildcard Address */
//         BindAddressLength = Socket->HelperData->MaxWSAddressLength;
//         BindAddress = HeapAlloc(GetProcessHeap(), 0, BindAddressLength);
//         if (!BindAddress)
//         {
//             return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, NULL);
//         }
//         Socket->HelperData->WSHGetWildcardSockaddr(Socket->HelperContext,
//                                                    BindAddress,
//                                                    &BindAddressLength);
//         /* Bind it */
//         BindError = WSPBind(Handle, BindAddress, BindAddressLength, lpErrno);
//         HeapFree(GetProcessHeap(), 0, BindAddress);
//         if (BindError == SOCKET_ERROR)
//             return SOCKET_ERROR;
//     }
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//     if (!NT_SUCCESS(Status))
//         return SOCKET_ERROR;
//
//     /* Set the Connect Data */
//     if (lpCallerData != NULL)
//     {
//         ConnectDataLength = lpCallerData->len;
//         Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                         SockEvent,
//                                         NULL,
//                                         NULL,
//                                         IOSB,
//                                         IOCTL_AFD_SET_CONNECT_DATA,
//                                         lpCallerData->buf,
//                                         ConnectDataLength,
//                                         NULL,
//                                         0);
//         /* Wait for return */
//         if (Status == STATUS_PENDING)
//         {
//             MsafdWaitForAlert(SockEvent);
//             Status = IOSB->Status;
//         }
//
//         if (Status != STATUS_SUCCESS)
//             goto Leave;
//     }
//
//     /* Calculate the size of SocketAddress->sa_data */
//     SocketDataLength = SocketAddressLength - FIELD_OFFSET(struct sockaddr, sa_data);
//
//     ConnectInfo = (PAFD_CONNECT_INFO)Buffer;
//
//     int connectionInfoSize = FIELD_OFFSET(AFD_CONNECT_INFO, RemoteAddress.Address[0].Address[SocketDataLength]);
//
//     if (connectionInfoSize > 128)
//     {
//         *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//     /* Set up Address in TDI Format */
//     ConnectInfo->RemoteAddress.TAAddressCount = 1;
//     ConnectInfo->RemoteAddress.Address[0].AddressLength = SocketDataLength;
//     ConnectInfo->RemoteAddress.Address[0].AddressType = SocketAddress->sa_family;
//     RtlCopyMemory(ConnectInfo->RemoteAddress.Address[0].Address,
//                   SocketAddress->sa_data,
//                   SocketDataLength);
//
//     /*
//     * Disable FD_WRITE and FD_CONNECT
//     * The latter fixes a race condition where the FD_CONNECT is re-enabled
//     * at the end of this function right after the Async Thread disables it.
//     * This should only happen at the *next* WSPConnect
//     */
//     if (Socket->SharedData->AsyncEvents & FD_CONNECT)
//     {
//         Socket->SharedData->AsyncDisabledEvents |= FD_CONNECT | FD_WRITE;
//     }
//
//     /* Tell AFD that we want Connection Data back, have it allocate a buffer */
//     if (lpCalleeData != NULL)
//     {
//         InConnectDataLength = lpCalleeData->len;
//         Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                         SockEvent,
//                                         NULL,
//                                         NULL,
//                                         IOSB,
//                                         IOCTL_AFD_SET_CONNECT_DATA_SIZE,
//                                         &InConnectDataLength,
//                                         sizeof(InConnectDataLength),
//                                         NULL,
//                                         0);
//
//         /* Wait for return */
//         if (Status == STATUS_PENDING)
//         {
//             MsafdWaitForAlert(SockEvent);
//             Status = IOSB->Status;
//         }
//
//         if (Status != STATUS_SUCCESS)
//             goto Leave;
//     }
//
//     /* AFD doesn't seem to care if these are invalid, but let's 0 them anyways */
//     ConnectInfo->Root = 0;
//     ConnectInfo->UseSAN = FALSE;
//     ConnectInfo->Unknown = 0;
//
//     /* Verify if we should use APC */
//     if (Socket->SharedData->NonBlocking)
//     {
//         APCContext = HeapAlloc(GlobalHeap, 0, sizeof(*APCContext));
//         if (!APCContext)
//         {
//             ERR("Not enough memory for APC Context\n");
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Leave;
//         }
//         APCContext->lpSocket = Handle;
//         APCFunction = &MsafdConnectAPC;
//
//         IOSB = &APCContext->IoStatusBlock;
//     }
//
//     IOSB->Status = STATUS_PENDING;
//     IOSB->Information = 0;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                    APCFunction ? NULL : SockEvent,
//                                    APCFunction,
//                                    APCContext,
//                                    IOSB,
//                                    IOCTL_AFD_CONNECT,
//                                    ConnectInfo,
//                                    connectionInfoSize,
//                                    NULL,
//                                    0);
//     if (Socket->SharedData->NonBlocking)
//     {
//         if (Status == STATUS_PENDING)
//         {
//             Status = STATUS_CANT_WAIT; // WSAEWOULDBLOCK
//             goto Leave;
//         }
//         else
//         {
//             /* HACK: Allow APC to be processed */
//             SleepEx(0, TRUE);
//         }
//     }
//     else
//     {
//         /* Wait for completion if blocking */
//         if (Status == STATUS_PENDING)
//         {
//             MsafdWaitForAlert(SockEvent);
//             Status = IOSB->Status;
//         }
//
//         Socket->SharedData->SocketLastError = TranslateNtStatusError(Status);
//         if (Status != STATUS_SUCCESS)
//             goto Leave;
//
//         Socket->SharedData->State = SocketConnected;
//         Socket->TdiConnectionHandle = (HANDLE)IOSB->Information;
//         Socket->SharedData->ConnectTime = GetCurrentTimeInSeconds();
//     }
//
//     /* Get any pending connect data */
//     if (lpCalleeData != NULL && Status == STATUS_SUCCESS)
//     {
//         IOSB = &DummyIOSB;
//         Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                        SockEvent,
//                                        NULL,
//                                        NULL,
//                                        IOSB,
//                                        IOCTL_AFD_GET_CONNECT_DATA,
//                                        NULL,
//                                        0,
//                                        lpCalleeData->buf,
//                                        lpCalleeData->len);
//         /* Wait for return */
//         if (Status == STATUS_PENDING)
//         {
//             MsafdWaitForAlert(SockEvent);
//             Status = IOSB->Status;
//         }
//     }
//
// Leave:
//     TRACE("Ending %lx\n", Status);
//
//     NtClose(SockEvent);
//
//     /* Re-enable Async Event */
//     SockReenableAsyncSelectEvent(Socket, FD_WRITE);
//
//     /* FIXME: THIS IS NOT RIGHT!!! HACK HACK HACK! */
//     SockReenableAsyncSelectEvent(Socket, FD_CONNECT);
//
//     if (Status == STATUS_SUCCESS && (Socket->HelperEvents & WSH_NOTIFY_CONNECT))
//     {
//         Errno = Socket->HelperData->WSHNotify(Socket->HelperContext,
//                                               Socket->Handle,
//                                               Socket->TdiAddressHandle,
//                                               Socket->TdiConnectionHandle,
//                                               WSH_NOTIFY_CONNECT);
//
//         if (Errno)
//         {
//             if (lpErrno) *lpErrno = Errno;
//             return SOCKET_ERROR;
//         }
//     }
//     else if (Status != STATUS_SUCCESS && (Socket->HelperEvents & WSH_NOTIFY_CONNECT_ERROR))
//     {
//         Errno = Socket->HelperData->WSHNotify(Socket->HelperContext,
//                                               Socket->Handle,
//                                               Socket->TdiAddressHandle,
//                                               Socket->TdiConnectionHandle,
//                                               WSH_NOTIFY_CONNECT_ERROR);
//
//         if (Errno)
//         {
//             if (lpErrno) *lpErrno = Errno;
//             return SOCKET_ERROR;
//         }
//     }
//
//     return MsafdReturnWithErrno(Status, lpErrno, 0, NULL);
// }
//
// BOOL
// WSPAPI
// WSPConnectEx(
//      SOCKET Handle,
//      const struct sockaddr *SocketAddress,
//      int SocketAddressLength,
//      PVOID lpSendBuffer,
//      DWORD dwSendDataLength,
//      LPDWORD lpdwBytesSent,
//      LPOVERLAPPED lpOverlapped)
// {
//     IO_STATUS_BLOCK DummyIOSB;
//     PIO_STATUS_BLOCK IOSB = &DummyIOSB;
//     PAFD_SUPER_CONNECT_INFO ConnectInfo = NULL;
//     PSOCKET_INFORMATION Socket;
//     NTSTATUS Status;
//     int SocketDataLength;
//     UCHAR Buffer[128];
//
//     if (!lpOverlapped)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (!lpSendBuffer && dwSendDataLength)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//         SetLastError(WSAENOTSOCK);
//         return FALSE;
//     }
//
//     
//     /* Cannot call on connectionless socket */
//     if (SocketAddressLength > ARRAYSIZE(Buffer) - sizeof(AFD_CONNECT_INFO) || Socket->SharedData->ServiceFlags1 & XP1_CONNECTIONLESS)
//     {
//         SetLastError(WSAEFAULT);
//         return SOCKET_ERROR;
//     }
//
//     if (Socket->SharedData->State == SocketOpen)
//     {
//         /* Socket already must be bound */
//         SetLastError(WSAEINVAL);
//         return FALSE;
//     }
//
//     ConnectInfo = (PAFD_SUPER_CONNECT_INFO)Buffer;
//     ConnectInfo->SanActive = FALSE;
//
//     /* Calculate the size of SocketAddress->sa_data */
//     SocketDataLength = SocketAddressLength - FIELD_OFFSET(struct sockaddr, sa_data);
//
//     /* Set up Address in TDI Format */
//     ConnectInfo->RemoteAddress.TAAddressCount = 1;
//     ConnectInfo->RemoteAddress.Address[0].AddressLength = SocketDataLength;
//     ConnectInfo->RemoteAddress.Address[0].AddressType = SocketAddress->sa_family;
//     RtlCopyMemory(ConnectInfo->RemoteAddress.Address[0].Address,
//                   SocketAddress->sa_data,
//                   SocketDataLength);
//
//     /*
//      * Disable FD_WRITE and FD_CONNECT.
//      * The latter fixes a race condition where the FD_CONNECT is re-enabled
//      * at the end of this function right after the Async Thread disables it.
//      * This should only happen at the *next* WSPConnect.
//      */
//     if (Socket->SharedData->AsyncEvents & FD_CONNECT)
//     {
//         Socket->SharedData->AsyncDisabledEvents |= (FD_CONNECT | FD_WRITE);
//     }
//
//     IOSB = (PIO_STATUS_BLOCK)lpOverlapped;
//     IOSB->Status = STATUS_PENDING;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                    lpOverlapped->hEvent,
//                                    NULL,
//                                    lpOverlapped->hEvent ? NULL : lpOverlapped,
//                                    IOSB,
//                                    IOCTL_AFD_SUPER_CONNECT,
//                                    ConnectInfo,
//                                    FIELD_OFFSET(AFD_SUPER_CONNECT_INFO, RemoteAddress.Address[0].Address[SocketDataLength]),
//                                    lpSendBuffer,
//                                    dwSendDataLength);
//
//     SetLastError(TranslateNtStatusError(Status));
//
//     return Status == STATUS_SUCCESS;
// }
//
// int
// WSPAPI
// WSPShutdown(SOCKET Handle,
//             int HowTo,
//             LPINT lpErrno)
//
// {
//     IO_STATUS_BLOCK         IOSB;
//     AFD_DISCONNECT_INFO     DisconnectInfo;
//     PSOCKET_INFORMATION     Socket = NULL;
//     NTSTATUS                Status;
//     HANDLE                  SockEvent;
//
//     TRACE("Called\n");
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//        if (lpErrno) *lpErrno = WSAENOTSOCK;
//        return SOCKET_ERROR;
//     }
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//
//     if( !NT_SUCCESS(Status) )
//         return SOCKET_ERROR;
//
//     /* Set AFD Disconnect Type */
//     switch (HowTo)
//     {
//         case SD_RECEIVE:
//             DisconnectInfo.DisconnectType = AFD_DISCONNECT_RECV;
//             Socket->SharedData->ReceiveShutdown = TRUE;
//             break;
//         case SD_SEND:
//             DisconnectInfo.DisconnectType= AFD_DISCONNECT_SEND;
//             Socket->SharedData->SendShutdown = TRUE;
//             break;
//         case SD_BOTH:
//             DisconnectInfo.DisconnectType = AFD_DISCONNECT_RECV | AFD_DISCONNECT_SEND;
//             Socket->SharedData->ReceiveShutdown = TRUE;
//             Socket->SharedData->SendShutdown = TRUE;
//             break;
//     }
//
//     DisconnectInfo.Timeout = RtlConvertLongToLargeInteger(-1000000);
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_DISCONNECT,
//                                    &DisconnectInfo,
//                                    sizeof(DisconnectInfo),
//                                    NULL,
//                                    0);
//
//     /* Wait for return */
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     TRACE("Ending\n");
//
//     NtClose( SockEvent );
//
//     Socket->SharedData->SocketLastError = TranslateNtStatusError(Status);
//     return MsafdReturnWithErrno( Status, lpErrno, 0, NULL );
// }
//
//
// INT
// WSPAPI
// WSPGetSockName(IN SOCKET Handle,
//                OUT LPSOCKADDR Name,
//                IN OUT LPINT NameLength,
//                OUT LPINT lpErrno)
// {
//     IO_STATUS_BLOCK         IOSB;
//     ULONG                   TdiAddressSize;
//     PTDI_ADDRESS_INFO       TdiAddress;
//     PTRANSPORT_ADDRESS      SocketAddress;
//     PSOCKET_INFORMATION     Socket = NULL;
//     NTSTATUS                Status;
//     HANDLE                  SockEvent;
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//        if (lpErrno) *lpErrno = WSAENOTSOCK;
//        return SOCKET_ERROR;
//     }
//
//     if (!Name || !NameLength)
//     {
//         if (lpErrno) *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//
//     if( !NT_SUCCESS(Status) )
//         return SOCKET_ERROR;
//
//     /* Allocate a buffer for the address */
//     TdiAddressSize =
//         sizeof(TRANSPORT_ADDRESS) + Socket->SharedData->SizeOfLocalAddress;
//     TdiAddress = HeapAlloc(GlobalHeap, 0, TdiAddressSize);
//
//     if ( TdiAddress == NULL )
//     {
//         NtClose( SockEvent );
//         if (lpErrno) *lpErrno = WSAENOBUFS;
//         return SOCKET_ERROR;
//     }
//
//     SocketAddress = &TdiAddress->Address;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_GET_SOCK_NAME,
//                                    NULL,
//                                    0,
//                                    TdiAddress,
//                                    TdiAddressSize);
//
//     /* Wait for return */
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     NtClose( SockEvent );
//
//     if (NT_SUCCESS(Status))
//     {
//         if (*NameLength >= Socket->SharedData->SizeOfLocalAddress)
//         {
//             Name->sa_family = SocketAddress->Address[0].AddressType;
//             RtlCopyMemory (Name->sa_data,
//                            SocketAddress->Address[0].Address,
//                            SocketAddress->Address[0].AddressLength);
//             *NameLength = Socket->SharedData->SizeOfLocalAddress;
//             TRACE("NameLength %d Address: %x Port %x\n",
//                           *NameLength, ((struct sockaddr_in *)Name)->sin_addr.s_addr,
//                           ((struct sockaddr_in *)Name)->sin_port);
//             HeapFree(GlobalHeap, 0, TdiAddress);
//             return 0;
//         }
//         else
//         {
//             HeapFree(GlobalHeap, 0, TdiAddress);
//             if (lpErrno) *lpErrno = WSAEFAULT;
//             return SOCKET_ERROR;
//         }
//     }
//
//     HeapFree(GlobalHeap, 0, TdiAddress);
//
//     return MsafdReturnWithErrno ( Status, lpErrno, 0, NULL );
// }
//
//
// INT
// WSPAPI
// WSPGetPeerName(IN SOCKET s,
//                OUT LPSOCKADDR Name,
//                IN OUT LPINT NameLength,
//                OUT LPINT lpErrno)
// {
//     IO_STATUS_BLOCK         IOSB;
//     ULONG                   TdiAddressSize;
//     PTRANSPORT_ADDRESS      SocketAddress;
//     PSOCKET_INFORMATION     Socket = NULL;
//     NTSTATUS                Status;
//     HANDLE                  SockEvent;
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(s);
//     if (!Socket)
//     {
//         if (lpErrno) *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//
//     if (Socket->SharedData->State != SocketConnected)
//     {
//         if (lpErrno) *lpErrno = WSAENOTCONN;
//         return SOCKET_ERROR;
//     }
//
//     if (!Name || !NameLength)
//     {
//         if (lpErrno) *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//
//     if( !NT_SUCCESS(Status) )
//         return SOCKET_ERROR;
//
//     /* Allocate a buffer for the address */
//     TdiAddressSize = sizeof(TRANSPORT_ADDRESS) + Socket->SharedData->SizeOfRemoteAddress;
//     SocketAddress = HeapAlloc(GlobalHeap, 0, TdiAddressSize);
//
//     if ( SocketAddress == NULL )
//     {
//         NtClose( SockEvent );
//         if (lpErrno) *lpErrno = WSAENOBUFS;
//         return SOCKET_ERROR;
//     }
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_GET_PEER_NAME,
//                                    NULL,
//                                    0,
//                                    SocketAddress,
//                                    TdiAddressSize);
//
//     /* Wait for return */
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     NtClose( SockEvent );
//
//     if (NT_SUCCESS(Status))
//     {
//         if (*NameLength >= Socket->SharedData->SizeOfRemoteAddress)
//         {
//             Name->sa_family = SocketAddress->Address[0].AddressType;
//             RtlCopyMemory (Name->sa_data,
//                            SocketAddress->Address[0].Address,
//                            SocketAddress->Address[0].AddressLength);
//             *NameLength = Socket->SharedData->SizeOfRemoteAddress;
//             TRACE("NameLength %d Address: %x Port %x\n",
//                           *NameLength, ((struct sockaddr_in *)Name)->sin_addr.s_addr,
//                           ((struct sockaddr_in *)Name)->sin_port);
//             HeapFree(GlobalHeap, 0, SocketAddress);
//             return 0;
//         }
//         else
//         {
//             HeapFree(GlobalHeap, 0, SocketAddress);
//             if (lpErrno) *lpErrno = WSAEFAULT;
//             return SOCKET_ERROR;
//         }
//     }
//
//     HeapFree(GlobalHeap, 0, SocketAddress);
//
//     return MsafdReturnWithErrno ( Status, lpErrno, 0, NULL );
// }
//
// INT
// WSPAPI
// WSPIoctl(IN  SOCKET Handle,
//          IN  DWORD dwIoControlCode,
//          IN  LPVOID lpvInBuffer,
//          IN  DWORD cbInBuffer,
//          OUT LPVOID lpvOutBuffer,
//          IN  DWORD cbOutBuffer,
//          OUT LPDWORD lpcbBytesReturned,
//          IN  LPWSAOVERLAPPED lpOverlapped,
//          IN  LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine,
//          IN  LPWSATHREADID lpThreadId,
//          OUT LPINT lpErrno)
// {
//     PSOCKET_INFORMATION Socket = NULL;
//     BOOL NeedsCompletion = lpOverlapped != NULL;
//     BOOLEAN NonBlocking;
//     INT Errno = NO_ERROR, Ret = SOCKET_ERROR;
//     DWORD cbRet = 0;
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//         if(lpErrno)
//             *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//
//     if (!lpcbBytesReturned && !lpOverlapped)
//     {
//         if(lpErrno)
//             *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//     switch( dwIoControlCode )
//     {
//         case FIONBIO:
//             if( cbInBuffer < sizeof(INT) || IS_INTRESOURCE(lpvInBuffer) )
//             {
//                 Errno = WSAEFAULT;
//                 break;
//             }
//             NonBlocking = *((PULONG)lpvInBuffer) ? TRUE : FALSE;
//             /* Don't allow to go in blocking mode if WSPAsyncSelect or WSPEventSelect is pending */
//             if (!NonBlocking)
//             {
//                 /* If there is an WSPAsyncSelect pending, fail with WSAEINVAL */
//                 if (Socket->SharedData->AsyncEvents & (~Socket->SharedData->AsyncDisabledEvents))
//                 {
//                     Errno = WSAEINVAL;
//                     break;
//                 }
//                 /* If there is an WSPEventSelect pending, fail with WSAEINVAL */
//                 if (Socket->NetworkEvents)
//                 {
//                     Errno = WSAEINVAL;
//                     break;
//                 }
//             }
//             Socket->SharedData->NonBlocking = NonBlocking ? 1 : 0;
//             NeedsCompletion = FALSE;
//             Errno = SetSocketInformation(Socket, AFD_INFO_BLOCKING_MODE, &NonBlocking, NULL, NULL, lpOverlapped, lpCompletionRoutine);
//             if (Errno == NO_ERROR)
//                 Ret = NO_ERROR;
//             break;
//         case FIONREAD:
//             if (IS_INTRESOURCE(lpvOutBuffer) || cbOutBuffer == 0)
//             {
//                 cbRet = sizeof(ULONG);
//                 Errno = WSAEFAULT;
//                 break;
//             }
//             if (cbOutBuffer < sizeof(ULONG))
//             {
//                 Errno = WSAEINVAL;
//                 break;
//             }
//             NeedsCompletion = FALSE;
//             Errno = GetSocketInformation(Socket, AFD_INFO_RECEIVE_CONTENT_SIZE, NULL, (PULONG)lpvOutBuffer, NULL, lpOverlapped, lpCompletionRoutine);
//             if (Errno == NO_ERROR)
//             {
//                 cbRet = sizeof(ULONG);
//                 Ret = NO_ERROR;
//             }
//             break;
//         case SIOCATMARK:
//             if (IS_INTRESOURCE(lpvOutBuffer) || cbOutBuffer == 0)
//             {
//                 cbRet = sizeof(BOOL);
//                 Errno = WSAEFAULT;
//                 break;
//             }
//             if (cbOutBuffer < sizeof(BOOL))
//             {
//                 Errno = WSAEINVAL;
//                 break;
//             }
//             if (Socket->SharedData->SocketType != SOCK_STREAM)
//             {
//                 Errno = WSAEINVAL;
//                 break;
//             }
//
//             /* FIXME: Return false if OOBINLINE is true for now
//                We should MSG_PEEK|MSG_OOB check with driver
//             */
//             *(BOOL*)lpvOutBuffer = !Socket->SharedData->OobInline;
//
//             cbRet = sizeof(BOOL);
//             Errno = NO_ERROR;
//             Ret = NO_ERROR;
//             break;
//         case SIO_GET_EXTENSION_FUNCTION_POINTER:
//             if (cbOutBuffer == 0)
//             {
//                 cbRet = sizeof(PVOID);
//                 Errno = WSAEFAULT;
//                 break;
//             }
//
//             if (cbInBuffer < sizeof(GUID) ||
//                 cbOutBuffer < sizeof(PVOID))
//             {
//                 Errno = WSAEINVAL;
//                 break;
//             }
//
//             {
//                 GUID AcceptExGUID = WSAID_ACCEPTEX;
//                 GUID ConnectExGUID = WSAID_CONNECTEX;
//                 GUID DisconnectExGUID = WSAID_DISCONNECTEX;
//                 GUID GetAcceptExSockaddrsGUID = WSAID_GETACCEPTEXSOCKADDRS;
//
//                 if (IsEqualGUID(&AcceptExGUID, lpvInBuffer))
//                 {
//                     *((PVOID *)lpvOutBuffer) = WSPAcceptEx;
//                     cbRet = sizeof(PVOID);
//                     Errno = NO_ERROR;
//                     Ret = NO_ERROR;
//                 }
//                 else if (IsEqualGUID(&ConnectExGUID, lpvInBuffer))
//                 {
//                     *((PVOID *)lpvOutBuffer) = WSPConnectEx;
//                     cbRet = sizeof(PVOID);
//                     Errno = NO_ERROR;
//                     Ret = NO_ERROR;
//                 }
//                 else if (IsEqualGUID(&DisconnectExGUID, lpvInBuffer))
//                 {
//                     *((PVOID *)lpvOutBuffer) = WSPDisconnectEx;
//                     cbRet = sizeof(PVOID);
//                     Errno = NO_ERROR;
//                     Ret = NO_ERROR;
//                 }
//                 else if (IsEqualGUID(&GetAcceptExSockaddrsGUID, lpvInBuffer))
//                 {
//                     *((PVOID *)lpvOutBuffer) = WSPGetAcceptExSockaddrs;
//                     cbRet = sizeof(PVOID);
//                     Errno = NO_ERROR;
//                     Ret = NO_ERROR;
//                 }
//                 else
//                 {
//                     ERR("Querying unknown extension function: %x\n", ((GUID*)lpvInBuffer)->Data1);
//                     Errno = WSAEOPNOTSUPP;
//                 }
//             }
//
//             break;
//         case SIO_ADDRESS_LIST_QUERY:
//             if (IS_INTRESOURCE(lpvOutBuffer) || cbOutBuffer == 0)
//             {
//                 cbRet = sizeof(SOCKET_ADDRESS_LIST) + sizeof(Socket->SharedData->WSLocalAddress);
//                 Errno = WSAEFAULT;
//                 break;
//             }
//             if (cbOutBuffer < sizeof(INT))
//             {
//                 Errno = WSAEINVAL;
//                 break;
//             }
//
//             cbRet = sizeof(SOCKET_ADDRESS_LIST) + sizeof(Socket->SharedData->WSLocalAddress);
//
//             ((SOCKET_ADDRESS_LIST*)lpvOutBuffer)->iAddressCount = 1;
//
//             if (cbOutBuffer < (sizeof(SOCKET_ADDRESS_LIST) + sizeof(Socket->SharedData->WSLocalAddress)))
//             {
//                 Errno = WSAEFAULT;
//                 break;
//             }
//
//             ((SOCKET_ADDRESS_LIST*)lpvOutBuffer)->Address[0].iSockaddrLength = sizeof(Socket->SharedData->WSLocalAddress);
//             ((SOCKET_ADDRESS_LIST*)lpvOutBuffer)->Address[0].lpSockaddr = &Socket->SharedData->WSLocalAddress;
//
//             Errno = NO_ERROR;
//             Ret = NO_ERROR;
//             break;
//         default:
//             Errno = Socket->HelperData->WSHIoctl(Socket->HelperContext,
//                                                  Handle,
//                                                  Socket->TdiAddressHandle,
//                                                  Socket->TdiConnectionHandle,
//                                                  dwIoControlCode,
//                                                  lpvInBuffer,
//                                                  cbInBuffer,
//                                                  lpvOutBuffer,
//                                                  cbOutBuffer,
//                                                  &cbRet,
//                                                  lpOverlapped,
//                                                  lpCompletionRoutine,
//                                                  &NeedsCompletion);
//
//             if (Errno == NO_ERROR)
//                 Ret = NO_ERROR;
//             break;
//     }
//     if (lpOverlapped && NeedsCompletion)
//     {
//         lpOverlapped->Internal = Errno;
//         lpOverlapped->InternalHigh = cbRet;
//         if (lpCompletionRoutine != NULL)
//         {
//             lpCompletionRoutine(Errno, cbRet, lpOverlapped, 0);
//         }
//         if (lpOverlapped->hEvent)
//             SetEvent(lpOverlapped->hEvent);
//         if (!PostQueuedCompletionStatus((HANDLE)Handle, cbRet, 0, lpOverlapped))
//         {
//             ERR("PostQueuedCompletionStatus failed %d\n", GetLastError());
//         }
//         return NO_ERROR;
//     }
//     if (lpErrno)
//         *lpErrno = Errno;
//     if (lpcbBytesReturned)
//         *lpcbBytesReturned = cbRet;
//     return Ret;
// }
//
//
// INT
// WSPAPI
// WSPGetSockOpt(IN SOCKET Handle,
//               IN INT Level,
//               IN INT OptionName,
//               OUT CHAR FAR* OptionValue,
//               IN OUT LPINT OptionLength,
//               OUT LPINT lpErrno)
// {
//     PSOCKET_INFORMATION Socket = NULL;
//     PVOID Buffer;
//     INT BufferSize;
//     BOOL BoolBuffer;
//     DWORD DwordBuffer;
//     INT Errno;
//
//     TRACE("Called\n");
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (Socket == NULL)
//     {
//         if (lpErrno) *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//     if (!OptionLength || !OptionValue)
//     {
//         if (lpErrno) *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//     switch (Level)
//     {
//         case SOL_SOCKET:
//             switch (OptionName)
//             {
//                 case SO_TYPE:
//                     Buffer = &Socket->SharedData->SocketType;
//                     BufferSize = sizeof(INT);
//                     break;
//
//                 case SO_RCVBUF:
//                     Buffer = &Socket->SharedData->SizeOfRecvBuffer;
//                     BufferSize = sizeof(ULONG);
//                     break;
//
//                 case SO_SNDBUF:
//                     Buffer = &Socket->SharedData->SizeOfSendBuffer;
//                     BufferSize = sizeof(ULONG);
//                     break;
//
//                 case SO_ACCEPTCONN:
//                     BoolBuffer = Socket->SharedData->Listening;
//                     Buffer = &BoolBuffer;
//                     BufferSize = sizeof(BOOL);
//                     break;
//
//                 case SO_BROADCAST:
//                     BoolBuffer = Socket->SharedData->Broadcast;
//                     Buffer = &BoolBuffer;
//                     BufferSize = sizeof(BOOL);
//                     break;
//
//                 case SO_DEBUG:
//                     BoolBuffer = Socket->SharedData->Debug;
//                     Buffer = &BoolBuffer;
//                     BufferSize = sizeof(BOOL);
//                     break;
//
//                 case SO_DONTLINGER:
//                     BoolBuffer = (Socket->SharedData->LingerData.l_onoff == 0);
//                     Buffer = &BoolBuffer;
//                     BufferSize = sizeof(BOOL);
//                     break;
//
//                 case SO_LINGER:
//                     if (Socket->SharedData->SocketType == SOCK_DGRAM)
//                     {
//                         if (lpErrno) *lpErrno = WSAENOPROTOOPT;
//                         return SOCKET_ERROR;
//                     }
//                     Buffer = &Socket->SharedData->LingerData;
//                     BufferSize = sizeof(struct linger);
//                     break;
//
//                 case SO_OOBINLINE:
//                     BoolBuffer = (Socket->SharedData->OobInline != 0);
//                     Buffer = &BoolBuffer;
//                     BufferSize = sizeof(BOOL);
//                     break;
//
//                 case SO_KEEPALIVE:
//                 case SO_DONTROUTE:
//                    /* These guys go directly to the helper */
//                    goto SendToHelper;
//
//                 case SO_CONDITIONAL_ACCEPT:
//                     BoolBuffer = (Socket->SharedData->UseDelayedAcceptance != 0);
//                     Buffer = &BoolBuffer;
//                     BufferSize = sizeof(BOOL);
//                     break;
//
//                 case SO_REUSEADDR:
//                     BoolBuffer = (Socket->SharedData->ReuseAddresses != 0);
//                     Buffer = &BoolBuffer;
//                     BufferSize = sizeof(BOOL);
//                     break;
//
//                 case SO_EXCLUSIVEADDRUSE:
//                     BoolBuffer = (Socket->SharedData->ExclusiveAddressUse != 0);
//                     Buffer = &BoolBuffer;
//                     BufferSize = sizeof(BOOL);
//                     break;
//
//                 case SO_ERROR:
//                     Buffer = &Socket->SharedData->SocketLastError;
//                     BufferSize = sizeof(INT);
//                     break;
//
//                 case SO_CONNECT_TIME:
//                     DwordBuffer = GetCurrentTimeInSeconds() - Socket->SharedData->ConnectTime;
//                     Buffer = &DwordBuffer;
//                     BufferSize = sizeof(DWORD);
//                     break;
//
//                 case SO_SNDTIMEO:
//                     Buffer = &Socket->SharedData->SendTimeout;
//                     BufferSize = sizeof(DWORD);
//                     break;
//                 case SO_RCVTIMEO:
//                     Buffer = &Socket->SharedData->RecvTimeout;
//                     BufferSize = sizeof(DWORD);
//                     break;
//                 case SO_PROTOCOL_INFOW:
//                     Buffer = &Socket->ProtocolInfo;
//                     BufferSize = sizeof(Socket->ProtocolInfo);
//                     break;
//
//                 case SO_GROUP_ID:
//                 case SO_GROUP_PRIORITY:
//                 case SO_MAX_MSG_SIZE:
//
//                 default:
//                     DbgPrint("MSAFD: Get unknown optname %x\n", OptionName);
//                     if (lpErrno) *lpErrno = WSAENOPROTOOPT;
//                     return SOCKET_ERROR;
//             }
//
//             if (*OptionLength < BufferSize)
//             {
//                 if (lpErrno) *lpErrno = WSAEFAULT;
//                 *OptionLength = BufferSize;
//                 return SOCKET_ERROR;
//             }
//             RtlCopyMemory(OptionValue, Buffer, BufferSize);
//
//             return 0;
//
//         default:
//             if (lpErrno) *lpErrno = WSAEINVAL;
//             return SOCKET_ERROR;
//     }
//
// SendToHelper:
//     Errno = Socket->HelperData->WSHGetSocketInformation(Socket->HelperContext,
//                                                         Handle,
//                                                         Socket->TdiAddressHandle,
//                                                         Socket->TdiConnectionHandle,
//                                                         Level,
//                                                         OptionName,
//                                                         OptionValue,
//                                                         (LPINT)OptionLength);
//     if (lpErrno) *lpErrno = Errno;
//     return (Errno == NO_ERROR) ? NO_ERROR : SOCKET_ERROR;
// }
//
// static
// INT
// NTAPI
// MsafdUpdateConnectionContext(
//      SOCKET Handle,
//      LPINT lpErrno)
// {
//     IO_STATUS_BLOCK            DummyIOSB;
//     PIO_STATUS_BLOCK           IOSB = &DummyIOSB;
//     NTSTATUS                   Status;
//     SOCK_SHARED_INFO SharedData;
//     AFD_TDI_HANDLE_DATA HandleData;
//     HANDLE SockEvent;
//     PSOCKET_INFORMATION Socket = GetSocketStructure(Handle);
//
//     if (!Socket)
//     {
//         if (lpErrno) *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//
//     if (Socket->SharedData->State == SocketConnected)
//     {
//         return NO_ERROR;
//     }
//
//     /* Get the socket context from AFD because when ConnectEx is used, this information is not updated. */
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    IOSB,
//                                    IOCTL_AFD_GET_CONTEXT,
//                                    NULL,
//                                    0,
//                                    &SharedData,
//                                    sizeof(SharedData));
//     
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB->Status;
//     }
//
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     ULONG flags = AFD_CONNECTION_HANDLE | AFD_ADDRESS_HANDLE;
//
//     /* Get TDI handles from AFD */
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    IOSB,
//                                    IOCTL_AFD_GET_TDI_HANDLES,
//                                    &flags,
//                                    sizeof(flags),
//                                    &HandleData,
//                                    sizeof(HandleData));
//     
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB->Status;
//     }
//
//     /* Bail out if the socket was destroyed */
//     if (!NT_SUCCESS(Status))
//         goto Quit;
//
//     Socket->SharedData->State = SharedData.State;
//
//     if (SharedData.State == SocketConnected)
//     {
//         /* Socket is now connected, update usermode msafd socket structure, same as the end of WSPConnect */
//         Socket->TdiConnectionHandle = HandleData.TdiConnectionHandle;
//         Socket->TdiAddressHandle = HandleData.TdiAddressHandle;
//         Socket->SharedData->ConnectTime = SharedData.ConnectTime;
//
//         /* Re-enable Async Event */
//         SockReenableAsyncSelectEvent(Socket, FD_WRITE);
//
//         /* FIXME: THIS IS NOT RIGHT!!! HACK HACK HACK! */
//         SockReenableAsyncSelectEvent(Socket, FD_CONNECT);
//
//         Socket->HelperData->WSHNotify(Socket->HelperContext,
//                                       Socket->Handle,
//                                       Socket->TdiAddressHandle,
//                                       Socket->TdiConnectionHandle,
//                                       WSH_NOTIFY_CONNECT);
//     }
//
//     NtClose(SockEvent);
//
// Quit:
//     if (Status == STATUS_SUCCESS)
//         return NO_ERROR;
//     if (lpErrno) *lpErrno = TranslateNtStatusError(Status);
//     return SOCKET_ERROR;
// }
//
// INT
// WSPAPI
// WSPSetSockOpt(
//     IN  SOCKET s,
//     IN  INT level,
//     IN  INT optname,
//     IN  CONST CHAR FAR* optval,
//     IN  INT optlen,
//     OUT LPINT lpErrno)
// {
//     PSOCKET_INFORMATION Socket;
//     INT Errno;
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(s);
//     if (Socket == NULL)
//     {
//         if (lpErrno) *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//     if (!optval && optlen)
//     {
//         if (lpErrno) *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//
//     /* FIXME: We should handle some more cases here */
//     if (level == SOL_SOCKET)
//     {
//         switch (optname)
//         {
//            case SO_BROADCAST:
//               if (optlen < sizeof(BOOL))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//               Socket->SharedData->Broadcast = (*optval != 0) ? 1 : 0;
//               return NO_ERROR;
//
//            case SO_OOBINLINE:
//               if (optlen < sizeof(BOOL))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//               Socket->SharedData->OobInline = (*optval != 0) ? 1 : 0;
//               return NO_ERROR;
//
//            case SO_DONTLINGER:
//               if (optlen < sizeof(BOOL))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//               Socket->SharedData->LingerData.l_onoff = (*optval != 0) ? 0 : 1;
//               return NO_ERROR;
//
//            case SO_REUSEADDR:
//               if (optlen < sizeof(BOOL))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//               Socket->SharedData->ReuseAddresses = (*optval != 0) ? 1 : 0;
//               return NO_ERROR;
//
//            case SO_EXCLUSIVEADDRUSE:
//               if (optlen < sizeof(BOOL))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//               Socket->SharedData->ExclusiveAddressUse = (*optval != 0) ? 1 : 0;
//               return NO_ERROR;
//
//            case SO_LINGER:
//               if (optlen < sizeof(struct linger))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//               RtlCopyMemory(&Socket->SharedData->LingerData,
//                             optval,
//                             sizeof(struct linger));
//               return NO_ERROR;
//
//            case SO_SNDBUF:
//               if (optlen < sizeof(ULONG))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//
//               SetSocketInformation(Socket,
//                                    AFD_INFO_SEND_WINDOW_SIZE,
//                                    NULL,
//                                    (PULONG)optval,
//                                    NULL,
//                                    NULL,
//                                    NULL);
//               GetSocketInformation(Socket,
//                                    AFD_INFO_SEND_WINDOW_SIZE,
//                                    NULL,
//                                    &Socket->SharedData->SizeOfSendBuffer,
//                                    NULL,
//                                    NULL,
//                                    NULL);
//
//               return NO_ERROR;
//
//            case SO_RCVBUF:
//               if (optlen < sizeof(ULONG))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//
//               /* FIXME: We should not have to limit the packet receive buffer size like this. workaround for CORE-15804 */
//               if (*(PULONG)optval > 0x2000)
//                   *(PULONG)optval = 0x2000;
//
//               SetSocketInformation(Socket,
//                                    AFD_INFO_RECEIVE_WINDOW_SIZE,
//                                    NULL,
//                                    (PULONG)optval,
//                                    NULL,
//                                    NULL,
//                                    NULL);
//               GetSocketInformation(Socket,
//                                    AFD_INFO_RECEIVE_WINDOW_SIZE,
//                                    NULL,
//                                    &Socket->SharedData->SizeOfRecvBuffer,
//                                    NULL,
//                                    NULL,
//                                    NULL);
//
//               return NO_ERROR;
//
//            case SO_UPDATE_CONNECT_CONTEXT:
//               return MsafdUpdateConnectionContext(s, lpErrno);
//
//            case SO_ERROR:
//               if (optlen < sizeof(INT))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//
//               RtlCopyMemory(&Socket->SharedData->SocketLastError,
//                             optval,
//                             sizeof(INT));
//               return NO_ERROR;
//
//            case SO_SNDTIMEO:
//               if (optlen < sizeof(DWORD))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//
//               RtlCopyMemory(&Socket->SharedData->SendTimeout,
//                             optval,
//                             sizeof(DWORD));
//               return NO_ERROR;
//
//            case SO_RCVTIMEO:
//               if (optlen < sizeof(DWORD))
//               {
//                   if (lpErrno) *lpErrno = WSAEFAULT;
//                   return SOCKET_ERROR;
//               }
//
//               RtlCopyMemory(&Socket->SharedData->RecvTimeout,
//                             optval,
//                             sizeof(DWORD));
//               return NO_ERROR;
//
//            case SO_KEEPALIVE:
//            case SO_DONTROUTE:
//               /* These go directly to the helper dll */
//               goto SendToHelper;
//
//            default:
//               /* Obviously this is a hack */
//               ERR("MSAFD: Set unknown optname %x\n", optname);
//               return NO_ERROR;
//         }
//     }
//
// SendToHelper:
//     Errno = Socket->HelperData->WSHSetSocketInformation(Socket->HelperContext,
//                                                         s,
//                                                         Socket->TdiAddressHandle,
//                                                         Socket->TdiConnectionHandle,
//                                                         level,
//                                                         optname,
//                                                         (PCHAR)optval,
//                                                         optlen);
//     if (lpErrno) *lpErrno = Errno;
//     return (Errno == NO_ERROR) ? NO_ERROR : SOCKET_ERROR;
// }
//
// /*
//  * FUNCTION: Initialize service provider for a client
//  * ARGUMENTS:
//  *     wVersionRequested = Highest WinSock SPI version that the caller can use
//  *     lpWSPData         = Address of WSPDATA structure to initialize
//  *     lpProtocolInfo    = Pointer to structure that defines the desired protocol
//  *     UpcallTable       = Pointer to upcall table of the WinSock DLL
//  *     lpProcTable       = Address of procedure table to initialize
//  * RETURNS:
//  *     Status of operation
//  */
//
// int
// WSPAPI
// WSPStartup(
//      WORD wVersionRequested,
//      LPWSPDATA lpWSPData,
//      LPWSAPROTOCOL_INFOW lpProtocolInfo,
//      WSPUPCALLTABLE UpcallTable,
//      LPWSPPROC_TABLE lpProcTable)
// {
//     NTSTATUS Status;
//
//     if (((LOBYTE(wVersionRequested) == 2) && (HIBYTE(wVersionRequested) < 2)) ||
//         (LOBYTE(wVersionRequested) < 2))
//     {
//         ERR("WSPStartup NOT SUPPORTED for version 0x%X\n", wVersionRequested);
//         return WSAVERNOTSUPPORTED;
//     }
//     else
//         Status = NO_ERROR;
//     /* FIXME: Enable all cases of WSPStartup status */
//     Upcalls = UpcallTable;
//
//     if (Status == NO_ERROR)
//     {
//         lpProcTable->lpWSPAccept = WSPAccept;
//         lpProcTable->lpWSPAddressToString = WSPAddressToString;
//         lpProcTable->lpWSPAsyncSelect = WSPAsyncSelect;
//         lpProcTable->lpWSPBind = WSPBind;
//         lpProcTable->lpWSPCancelBlockingCall = WSPCancelBlockingCall;
//         lpProcTable->lpWSPCleanup = WSPCleanup;
//         lpProcTable->lpWSPCloseSocket = WSPCloseSocket;
//         lpProcTable->lpWSPConnect = WSPConnect;
//         lpProcTable->lpWSPDuplicateSocket = WSPDuplicateSocket;
//         lpProcTable->lpWSPEnumNetworkEvents = WSPEnumNetworkEvents;
//         lpProcTable->lpWSPEventSelect = WSPEventSelect;
//         lpProcTable->lpWSPGetOverlappedResult = WSPGetOverlappedResult;
//         lpProcTable->lpWSPGetPeerName = WSPGetPeerName;
//         lpProcTable->lpWSPGetSockName = WSPGetSockName;
//         lpProcTable->lpWSPGetSockOpt = WSPGetSockOpt;
//         lpProcTable->lpWSPGetQOSByName = WSPGetQOSByName;
//         lpProcTable->lpWSPIoctl = WSPIoctl;
//         lpProcTable->lpWSPJoinLeaf = WSPJoinLeaf;
//         lpProcTable->lpWSPListen = WSPListen;
//         lpProcTable->lpWSPRecv = WSPRecv;
//         lpProcTable->lpWSPRecvDisconnect = WSPRecvDisconnect;
//         lpProcTable->lpWSPRecvFrom = WSPRecvFrom;
//         lpProcTable->lpWSPSelect = WSPSelect;
//         lpProcTable->lpWSPSend = WSPSend;
//         lpProcTable->lpWSPSendDisconnect = WSPSendDisconnect;
//         lpProcTable->lpWSPSendTo = WSPSendTo;
//         lpProcTable->lpWSPSetSockOpt = WSPSetSockOpt;
//         lpProcTable->lpWSPShutdown = WSPShutdown;
//         lpProcTable->lpWSPSocket = WSPSocket;
//         lpProcTable->lpWSPStringToAddress = WSPStringToAddress;
//         lpWSPData->wVersion     = MAKEWORD(2, 2);
//         lpWSPData->wHighVersion = MAKEWORD(2, 2);
//         /* Save CatalogEntryId for all upcalls */
//         CatalogEntryId = lpProtocolInfo->dwCatalogEntryId;
//     }
//
//     TRACE("Status (%d).\n", Status);
//     return Status;
// }
//
//
// INT
// WSPAPI
// WSPAddressToString(IN LPSOCKADDR lpsaAddress,
//                    IN DWORD dwAddressLength,
//                    IN LPWSAPROTOCOL_INFOW lpProtocolInfo,
//                    OUT LPWSTR lpszAddressString,
//                    IN OUT LPDWORD lpdwAddressStringLength,
//                    OUT LPINT lpErrno)
// {
//     SIZE_T size;
//     WCHAR buffer[54]; /* 32 digits + 7':' + '[' + '%" + 5 digits + ']:' + 5 digits + '\0' */
//     WCHAR *p;
//
//     if (!lpsaAddress || !lpszAddressString || !lpdwAddressStringLength)
//     {
//         if (lpErrno) *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//     switch (lpsaAddress->sa_family)
//     {
//         case AF_INET:
//             if (dwAddressLength < sizeof(SOCKADDR_IN))
//             {
//                 if (lpErrno) *lpErrno = WSAEINVAL;
//                 return SOCKET_ERROR;
//             }
//             swprintf(buffer,
//                      L"%u.%u.%u.%u:%u",
//                      (unsigned int)(ntohl(((SOCKADDR_IN *)lpsaAddress)->sin_addr.s_addr) >> 24 & 0xff),
//                      (unsigned int)(ntohl(((SOCKADDR_IN *)lpsaAddress)->sin_addr.s_addr) >> 16 & 0xff),
//                      (unsigned int)(ntohl(((SOCKADDR_IN *)lpsaAddress)->sin_addr.s_addr) >> 8 & 0xff),
//                      (unsigned int)(ntohl(((SOCKADDR_IN *)lpsaAddress)->sin_addr.s_addr) & 0xff),
//                      ntohs(((SOCKADDR_IN *)lpsaAddress)->sin_port));
//
//             p = wcschr(buffer, L':');
//             if (!((SOCKADDR_IN *)lpsaAddress)->sin_port)
//             {
//                 *p = 0;
//             }
//             break;
//         default:
//             if (lpErrno) *lpErrno = WSAEINVAL;
//             return SOCKET_ERROR;
//     }
//
//     size = wcslen(buffer) + 1;
//
//     if (*lpdwAddressStringLength < size)
//     {
//         *lpdwAddressStringLength = size;
//         if (lpErrno) *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//     *lpdwAddressStringLength = size;
//     wcscpy(lpszAddressString, buffer);
//     return 0;
// }
//
// INT
// WSPAPI
// WSPStringToAddress(IN LPWSTR AddressString,
//                    IN INT AddressFamily,
//                    IN LPWSAPROTOCOL_INFOW lpProtocolInfo,
//                    OUT LPSOCKADDR lpAddress,
//                    IN OUT LPINT lpAddressLength,
//                    OUT LPINT lpErrno)
// {
//     int numdots = 0;
//     USHORT port;
//     LONG inetaddr = 0, ip_part;
//     LPWSTR *bp = NULL;
//     SOCKADDR_IN *sockaddr;
//
//     if (!lpAddressLength || !lpAddress || !AddressString)
//     {
//         if (lpErrno) *lpErrno = WSAEINVAL;
//         return SOCKET_ERROR;
//     }
//
//     sockaddr = (SOCKADDR_IN *)lpAddress;
//
//     /* Set right address family */
//     if (lpProtocolInfo != NULL)
//     {
//         sockaddr->sin_family = lpProtocolInfo->iAddressFamily;
//     }
//     else
//     {
//         sockaddr->sin_family = AddressFamily;
//     }
//
//     /* Report size */
//     if (AddressFamily == AF_INET)
//     {
//         if (*lpAddressLength < (INT)sizeof(SOCKADDR_IN))
//         {
//             if (lpErrno) *lpErrno = WSAEFAULT;
//         }
//         else
//         {
//             // translate ip string to ip
//
//             /* Get ip number */
//             bp = &AddressString;
//             inetaddr = 0;
//
//             while (*bp < &AddressString[wcslen(AddressString)])
//             {
//                 ip_part = wcstol(*bp, bp, 10);
//                 /* ip part number should be in range 0-255 */
//                 if (ip_part < 0 || ip_part > 255)
//                 {
//                     if (lpErrno) *lpErrno = WSAEINVAL;
//                     return SOCKET_ERROR;
//                 }
//                 inetaddr = (inetaddr << 8) + ip_part;
//                 /* we end on string end or port separator */
//                 if ((*bp)[0] == 0 || (*bp)[0] == L':')
//                     break;
//                 /* ip parts are dot separated. verify it */
//                 if ((*bp)[0] != L'.')
//                 {
//                     if (lpErrno) *lpErrno = WSAEINVAL;
//                     return SOCKET_ERROR;
//                 }
//                 /* count the dots */
//                 numdots++;
//                 /* move over the dot to next ip part */
//                 (*bp)++;
//             }
//
//             /* check dots count */
//             if (numdots != 3)
//             {
//                 if (lpErrno) *lpErrno = WSAEINVAL;
//                 return SOCKET_ERROR;
//             }
//
//             /* Get port number */
//             if ((*bp)[0] == L':')
//             {
//                 /* move over the column to port part */
//                 (*bp)++;
//                 /* next char should be numeric */
//                 if ((*bp)[0] < L'0' || (*bp)[0] > L'9')
//                 {
//                     if (lpErrno) *lpErrno = WSAEINVAL;
//                     return SOCKET_ERROR;
//                 }
//                 port = wcstol(*bp, bp, 10);
//             }
//             else
//             {
//                 port = 0;
//             }
//
//             if (lpErrno) *lpErrno = NO_ERROR;
//             /* rest sockaddr.sin_addr.s_addr
//             for we need to be sure it is zero when we come to while */
//             *lpAddressLength = sizeof(*sockaddr);
//             memset(lpAddress, 0, sizeof(*sockaddr));
//             sockaddr->sin_family = AF_INET;
//             sockaddr->sin_addr.s_addr = inetaddr;
//             sockaddr->sin_port = port;
//         }
//     }
//
//     if (lpErrno && !*lpErrno)
//     {
//         return 0;
//     }
//
//     return SOCKET_ERROR;
// }
//
// /*
//  * FUNCTION: Cleans up service provider for a client
//  * ARGUMENTS:
//  *     lpErrno = Address of buffer for error information
//  * RETURNS:
//  *     0 if successful, or SOCKET_ERROR if not
//  */
// INT
// WSPAPI
// WSPCleanup(OUT LPINT lpErrno)
//
// {
//     TRACE("Leaving.\n");
//
//     if (lpErrno) *lpErrno = NO_ERROR;
//
//     return 0;
// }
//
// static
// VOID
// NTAPI
// MsafdInfoAPC(
//      PVOID ApcContext,
//      PIO_STATUS_BLOCK IoStatusBlock,
//      ULONG Reserved)
// {
//     PMSAFD_INFO_APC_CONTEXT Context = ApcContext;
//
//     TRACE("MsafdInfoAPC(%p %lx %lx)\n", ApcContext, IoStatusBlock->Status, IoStatusBlock->Information);
//
//     if (Context->lpCompletionRoutine)
//         Context->lpCompletionRoutine(IoStatusBlock->Status, IoStatusBlock->Information, Context->lpOverlapped, 0);
//
//     /* Free IOCTL buffer */
//     HeapFree(GlobalHeap, 0, Context->lpInfoData);
//
//     HeapFree(GlobalHeap, 0, ApcContext);
// }
//
// int
// GetSocketInformation(PSOCKET_INFORMATION Socket,
//                      ULONG AfdInformationClass,
//                      PBOOLEAN Boolean OPTIONAL,
//                      PULONG Ulong OPTIONAL,
//                      PLARGE_INTEGER LargeInteger OPTIONAL,
//                      LPWSAOVERLAPPED Overlapped OPTIONAL,
//                      LPWSAOVERLAPPED_COMPLETION_ROUTINE CompletionRoutine OPTIONAL)
// {
//     PIO_STATUS_BLOCK        IOSB;
//     IO_STATUS_BLOCK         DummyIOSB;
//     PAFD_INFO               InfoData;
//     NTSTATUS                Status;
//     PMSAFD_INFO_APC_CONTEXT APCContext;
//     PIO_APC_ROUTINE         APCFunction;
//     HANDLE                  Event = NULL;
//     HANDLE                  SockEvent;
//
//     InfoData = HeapAlloc(GlobalHeap, 0, sizeof(*InfoData));
//     if (!InfoData)
//         return SOCKET_ERROR;
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         HeapFree(GlobalHeap, 0, InfoData);
//         return SOCKET_ERROR;
//     }
//
//     /* Set Info Class */
//     InfoData->InformationClass = AfdInformationClass;
//
//     /* Verify if we should use APC */
//     if (!Overlapped)
//     {
//         /* Not using Overlapped structure, so use normal blocking on event */
//         APCContext = NULL;
//         APCFunction = NULL;
//         Event = SockEvent;
//         IOSB = &DummyIOSB;
//     }
//     else
//     {
//         /* Overlapped request for non overlapped opened socket */
//         if ((Socket->SharedData->CreateFlags & SO_SYNCHRONOUS_NONALERT) != 0)
//         {
//             TRACE("Opened without flag WSA_FLAG_OVERLAPPED. Do nothing.\n");
//             NtClose(SockEvent);
//             HeapFree(GlobalHeap, 0, InfoData);
//             return MsafdReturnWithErrno(STATUS_SUCCESS, NULL, 0, NULL);
//         }
//
//         APCContext = HeapAlloc(GlobalHeap, 0, sizeof(*APCContext));
//         if (!APCContext)
//         {
//             ERR("Not enough memory for APC Context\n");
//             NtClose(SockEvent);
//             HeapFree(GlobalHeap, 0, InfoData);
//             return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, NULL, 0, NULL);
//         }
//         APCContext->lpCompletionRoutine = CompletionRoutine;
//         APCContext->lpOverlapped = Overlapped;
//         APCContext->lpInfoData = InfoData;
//         APCFunction = &MsafdInfoAPC;
//
//         if (!CompletionRoutine)
//             Event = Overlapped->hEvent;
//
//         IOSB = (PIO_STATUS_BLOCK)&Overlapped->Internal;
//     }
//
//     IOSB->Status = STATUS_PENDING;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                    Event,
//                                    APCFunction,
//                                    APCContext,
//                                    IOSB,
//                                    IOCTL_AFD_GET_INFO,
//                                    InfoData,
//                                    sizeof(*InfoData),
//                                    InfoData,
//                                    sizeof(*InfoData));
//
//     /* Wait for completion if not overlapped */
//     if (!Overlapped && Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB->Status;
//     }
//
//     TRACE("Status %lx\n", Status);
//
//     if (Status == STATUS_SUCCESS)
//     {
//         /* Return Information */
//         if (Ulong != NULL)
//         {
//             *Ulong = InfoData->Information.Ulong;
//         }
//         if (LargeInteger != NULL)
//         {
//             *LargeInteger = InfoData->Information.LargeInteger;
//         }
//         if (Boolean != NULL)
//         {
//             *Boolean = InfoData->Information.Boolean;
//         }
//     }
//
//     NtClose(SockEvent);
//     if (!APCFunction)
//     {
//         /* When using APC, this will be freed by the APC function */
//         HeapFree(GlobalHeap, 0, InfoData);
//     }
//
//     return MsafdReturnWithErrno(Status, NULL, 0, NULL);
// }
//
//
// int
// SetSocketInformation(PSOCKET_INFORMATION Socket,
//                      ULONG AfdInformationClass,
//                      PBOOLEAN Boolean OPTIONAL,
//                      PULONG Ulong OPTIONAL,
//                      PLARGE_INTEGER LargeInteger OPTIONAL,
//                      LPWSAOVERLAPPED Overlapped OPTIONAL,
//                      LPWSAOVERLAPPED_COMPLETION_ROUTINE CompletionRoutine OPTIONAL)
// {
//     PIO_STATUS_BLOCK        IOSB;
//     IO_STATUS_BLOCK         DummyIOSB;
//     PAFD_INFO               InfoData;
//     NTSTATUS                Status;
//     PMSAFD_INFO_APC_CONTEXT APCContext;
//     PIO_APC_ROUTINE         APCFunction;
//     HANDLE                  Event = NULL;
//     HANDLE                  SockEvent;
//
//     InfoData = HeapAlloc(GlobalHeap, 0, sizeof(*InfoData));
//     if (!InfoData)
//         return SOCKET_ERROR;
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         HeapFree(GlobalHeap, 0, InfoData);
//         return SOCKET_ERROR;
//     }
//
//     /* Set Info Class */
//     InfoData->InformationClass = AfdInformationClass;
//
//     /* Set Information */
//     if (Ulong != NULL)
//     {
//         InfoData->Information.Ulong = *Ulong;
//     }
//     if (LargeInteger != NULL)
//     {
//         InfoData->Information.LargeInteger = *LargeInteger;
//     }
//     if (Boolean != NULL)
//     {
//         InfoData->Information.Boolean = *Boolean;
//     }
//
//     /* Verify if we should use APC */
//     if (!Overlapped)
//     {
//         /* Not using Overlapped structure, so use normal blocking on event */
//         APCContext = NULL;
//         APCFunction = NULL;
//         Event = SockEvent;
//         IOSB = &DummyIOSB;
//     }
//     else
//     {
//         /* Overlapped request for non overlapped opened socket */
//         if ((Socket->SharedData->CreateFlags & SO_SYNCHRONOUS_NONALERT) != 0)
//         {
//             TRACE("Opened without flag WSA_FLAG_OVERLAPPED. Do nothing.\n");
//             NtClose(SockEvent);
//             HeapFree(GlobalHeap, 0, InfoData);
//             return MsafdReturnWithErrno(STATUS_SUCCESS, NULL, 0, NULL);
//         }
//
//         APCContext = HeapAlloc(GlobalHeap, 0, sizeof(*APCContext));
//         if (!APCContext)
//         {
//             ERR("Not enough memory for APC Context\n");
//             NtClose(SockEvent);
//             HeapFree(GlobalHeap, 0, InfoData);
//             return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, NULL, 0, NULL);
//         }
//         APCContext->lpCompletionRoutine = CompletionRoutine;
//         APCContext->lpOverlapped = Overlapped;
//         APCContext->lpInfoData = InfoData;
//         APCFunction = &MsafdInfoAPC;
//
//         if (!CompletionRoutine)
//             Event = Overlapped->hEvent;
//
//         IOSB = (PIO_STATUS_BLOCK)&Overlapped->Internal;
//     }
//
//     IOSB->Status = STATUS_PENDING;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                    Event,
//                                    APCFunction,
//                                    APCContext,
//                                    IOSB,
//                                    IOCTL_AFD_SET_INFO,
//                                    InfoData,
//                                    sizeof(*InfoData),
//                                    NULL,
//                                    0);
//
//     /* Wait for completion if not overlapped */
//     if (!Overlapped && Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB->Status;
//     }
//
//     TRACE("Status %lx\n", Status);
//
//     NtClose(SockEvent);
//     if (!APCFunction)
//     {
//         /* When using APC, this will be freed by the APC function */
//         HeapFree(GlobalHeap, 0, InfoData);
//     }
//
//     return MsafdReturnWithErrno(Status, NULL, 0, NULL);
// }
//
// PSOCKET_INFORMATION
// GetSocketStructure(SOCKET Handle)
// {
//     PSOCKET_INFORMATION CurrentSocket;
//
//     EnterCriticalSection(&SocketListLock);
//
//     CurrentSocket = SocketListHead;
//     while (CurrentSocket)
//     {
//         if (CurrentSocket->Handle == Handle)
//         {
//             LeaveCriticalSection(&SocketListLock);
//             return CurrentSocket;
//         }
//
//         CurrentSocket = CurrentSocket->NextSocket;
//     }
//
//     LeaveCriticalSection(&SocketListLock);
//
//     return NULL;
// }
//
// int CreateContext(PSOCKET_INFORMATION Socket)
// {
//     IO_STATUS_BLOCK     IOSB;
//     SOCKET_CONTEXT      ContextData;
//     NTSTATUS            Status;
//     HANDLE              SockEvent;
//
//     Status = NtCreateEvent(&SockEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            SynchronizationEvent,
//                            FALSE);
//
//     if( !NT_SUCCESS(Status) )
//         return SOCKET_ERROR;
//
//     /* Create Context */
//     ContextData.SharedData = *Socket->SharedData;
//     ContextData.SizeOfHelperData = 0;
//     RtlCopyMemory (&ContextData.LocalAddress,
//                    Socket->LocalAddress,
//                    Socket->SharedData->SizeOfLocalAddress);
//     RtlCopyMemory (&ContextData.RemoteAddress,
//                    Socket->RemoteAddress,
//                    Socket->SharedData->SizeOfRemoteAddress);
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Socket->Handle,
//                                    SockEvent,
//                                    NULL,
//                                    NULL,
//                                    &IOSB,
//                                    IOCTL_AFD_SET_CONTEXT,
//                                    &ContextData,
//                                    sizeof(ContextData),
//                                    NULL,
//                                    0);
//
//     /* Wait for Completion */
//     if (Status == STATUS_PENDING)
//     {
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB.Status;
//     }
//
//     NtClose( SockEvent );
//
//     return Status == STATUS_SUCCESS ? NO_ERROR : SOCKET_ERROR;
// }
//
// BOOLEAN SockCreateOrReferenceAsyncThread(VOID)
// {
//     HANDLE hAsyncThread;
//     DWORD AsyncThreadId;
//     HANDLE AsyncEvent;
//     OBJECT_HANDLE_ATTRIBUTE_INFORMATION HandleFlags;
//     NTSTATUS Status;
//
//     /* Check if the Thread Already Exists */
//     if (SockAsyncThreadRefCount)
//     {
//         ASSERT(SockAsyncCompletionPort);
//         return TRUE;
//     }
//
//     /* Create the Completion Port */
//     if (!SockAsyncCompletionPort)
//     {
//         Status = NtCreateIoCompletion(&SockAsyncCompletionPort,
//                                       IO_COMPLETION_ALL_ACCESS,
//                                       NULL,
//                                       2); // Allow 2 threads only
//         if (!NT_SUCCESS(Status))
//         {
//              ERR("Failed to create completion port: 0x%08x\n", Status);
//              return FALSE;
//         }
//         /* Protect Handle */
//         HandleFlags.ProtectFromClose = TRUE;
//         HandleFlags.Inherit = FALSE;
//         Status = NtSetInformationObject(SockAsyncCompletionPort,
//                                         ObjectHandleFlagInformation,
//                                         &HandleFlags,
//                                         sizeof(HandleFlags));
//     }
//
//     /* Create the Async Event */
//     Status = NtCreateEvent(&AsyncEvent,
//                            EVENT_ALL_ACCESS,
//                            NULL,
//                            NotificationEvent,
//                            FALSE);
//
//     /* Create the Async Thread */
//     hAsyncThread = CreateThread(NULL,
//                                 0,
//                                 SockAsyncThread,
//                                 NULL,
//                                 0,
//                                 &AsyncThreadId);
//
//     /* Close the Handle */
//     NtClose(hAsyncThread);
//
//     /* Increase the Reference Count */
//     SockAsyncThreadRefCount++;
//     return TRUE;
// }
//
// ULONG
// NTAPI
// SockAsyncThread(PVOID ThreadParam)
// {
//     PVOID AsyncContext;
//     PASYNC_COMPLETION_ROUTINE AsyncCompletionRoutine;
//     IO_STATUS_BLOCK IOSB;
//     NTSTATUS Status;
//
//     /* Make the Thread Higher Priority */
//     SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_ABOVE_NORMAL);
//
//     /* Do a KQUEUE/WorkItem Style Loop, thanks to IoCompletion Ports */
//     do
//     {
//         Status =  NtRemoveIoCompletion (SockAsyncCompletionPort,
//                                         (PVOID*)&AsyncCompletionRoutine,
//                                         &AsyncContext,
//                                         &IOSB,
//                                         NULL);
//         /* Call the Async Function */
//         if (NT_SUCCESS(Status))
//         {
//             (*AsyncCompletionRoutine)(AsyncContext, &IOSB);
//         }
//         else
//         {
//             /* It Failed, sleep for a second */
//             Sleep(1000);
//         }
//     } while ((Status != STATUS_TIMEOUT));
//
//     /* The Thread has Ended */
//     return 0;
// }
//
// BOOLEAN SockGetAsyncSelectHelperAfdHandle(VOID)
// {
//     UNICODE_STRING AfdHelper;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoSb;
//     FILE_COMPLETION_INFORMATION CompletionInfo;
//     OBJECT_HANDLE_ATTRIBUTE_INFORMATION HandleFlags;
//
//     /* First, make sure we're not already initialized */
//     if (SockAsyncHelperAfdHandle)
//     {
//         return TRUE;
//     }
//
//     /* Set up Handle Name and Object */
//     RtlInitUnicodeString(&AfdHelper, L"\\Device\\Afd\\AsyncSelectHlp" );
//                          InitializeObjectAttributes(&ObjectAttributes,
//                          &AfdHelper,
//                          OBJ_INHERIT | OBJ_CASE_INSENSITIVE,
//                          NULL,
//                          NULL);
//
//     /* Open the Handle to AFD */
//     NtCreateFile(&SockAsyncHelperAfdHandle,
//                  GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
//                  &ObjectAttributes,
//                  &IoSb,
//                  NULL,
//                  0,
//                  FILE_SHARE_READ | FILE_SHARE_WRITE,
//                  FILE_OPEN_IF,
//                  0,
//                  NULL,
//                  0);
//
//     /*
//      * Now Set up the Completion Port Information
//      * This means that whenever a Poll is finished, the routine will be executed
//      */
//     CompletionInfo.Port = SockAsyncCompletionPort;
//     CompletionInfo.Key = SockAsyncSelectCompletionRoutine;
//     NtSetInformationFile(SockAsyncHelperAfdHandle,
//                          &IoSb,
//                          &CompletionInfo,
//                          sizeof(CompletionInfo),
//                          FileCompletionInformation);
//
//
//     /* Protect the Handle */
//     HandleFlags.ProtectFromClose = TRUE;
//     HandleFlags.Inherit = FALSE;
//     NtSetInformationObject(SockAsyncCompletionPort,
//                            ObjectHandleFlagInformation,
//                            &HandleFlags,
//                            sizeof(HandleFlags));
//
//
//     /* Set this variable to true so that Send/Recv/Accept will know wether to renable disabled events */
//     SockAsyncSelectCalled = TRUE;
//     return TRUE;
// }
//
// VOID SockAsyncSelectCompletionRoutine(PVOID Context, PIO_STATUS_BLOCK IoStatusBlock)
// {
//
//     PASYNC_DATA AsyncData = Context;
//     PSOCKET_INFORMATION Socket;
//     ULONG x;
//
//     /* Get the Socket */
//     Socket = AsyncData->ParentSocket;
//
//     /* Check if the Sequence  Number Changed behind our back */
//     if (AsyncData->SequenceNumber != Socket->SharedData->SequenceNumber )
//     {
//         return;
//     }
//
//     /* Check we were manually called b/c of a failure */
//     if (!NT_SUCCESS(IoStatusBlock->Status))
//     {
//         /* FIXME: Perform Upcall */
//         return;
//     }
//
//     for (x = 1; x; x<<=1)
//     {
//         switch (AsyncData->AsyncSelectInfo.Handles[0].Events & x)
//         {
//             case AFD_EVENT_RECEIVE:
//                 if (0 != (Socket->SharedData->AsyncEvents & FD_READ) &&
//                     0 == (Socket->SharedData->AsyncDisabledEvents & FD_READ))
//                 {
//                     /* Make the Notification */
//                     (Upcalls.lpWPUPostMessage.unwrap())(Socket->SharedData->hWnd,
//                                                Socket->SharedData->wMsg,
//                                                Socket->Handle,
//                                                WSAMAKESELECTREPLY(FD_READ, 0));
//                     /* Disable this event until the next read(); */
//                     Socket->SharedData->AsyncDisabledEvents |= FD_READ;
//                 }
//             break;
//
//             case AFD_EVENT_OOB_RECEIVE:
//             if (0 != (Socket->SharedData->AsyncEvents & FD_OOB) &&
//                 0 == (Socket->SharedData->AsyncDisabledEvents & FD_OOB))
//             {
//                 /* Make the Notification */
//                 (Upcalls.lpWPUPostMessage.unwrap())(Socket->SharedData->hWnd,
//                                            Socket->SharedData->wMsg,
//                                            Socket->Handle,
//                                            WSAMAKESELECTREPLY(FD_OOB, 0));
//                 /* Disable this event until the next read(); */
//                 Socket->SharedData->AsyncDisabledEvents |= FD_OOB;
//             }
//             break;
//
//             case AFD_EVENT_SEND:
//                 if (0 != (Socket->SharedData->AsyncEvents & FD_WRITE) &&
//                     0 == (Socket->SharedData->AsyncDisabledEvents & FD_WRITE))
//                 {
//                     /* Make the Notification */
//                     (Upcalls.lpWPUPostMessage.unwrap())(Socket->SharedData->hWnd,
//                                                Socket->SharedData->wMsg,
//                                                Socket->Handle,
//                                                WSAMAKESELECTREPLY(FD_WRITE, 0));
//                     /* Disable this event until the next write(); */
//                     Socket->SharedData->AsyncDisabledEvents |= FD_WRITE;
//                 }
//                 break;
//
//                 /* FIXME: THIS IS NOT RIGHT!!! HACK HACK HACK! */
//             case AFD_EVENT_CONNECT:
//             case AFD_EVENT_CONNECT_FAIL:
//                 if (0 != (Socket->SharedData->AsyncEvents & FD_CONNECT) &&
//                     0 == (Socket->SharedData->AsyncDisabledEvents & FD_CONNECT))
//                 {
//                     /* Make the Notification */
//                     (Upcalls.lpWPUPostMessage.unwrap())(Socket->SharedData->hWnd,
//                         Socket->SharedData->wMsg,
//                         Socket->Handle,
//                         WSAMAKESELECTREPLY(FD_CONNECT, 0));
//                     /* Disable this event forever; */
//                     Socket->SharedData->AsyncDisabledEvents |= FD_CONNECT;
//                 }
//                 break;
//
//             case AFD_EVENT_ACCEPT:
//                 if (0 != (Socket->SharedData->AsyncEvents & FD_ACCEPT) &&
//                     0 == (Socket->SharedData->AsyncDisabledEvents & FD_ACCEPT))
//                 {
//                     /* Make the Notification */
//                     (Upcalls.lpWPUPostMessage.unwrap())(Socket->SharedData->hWnd,
//                                                Socket->SharedData->wMsg,
//                                                Socket->Handle,
//                                                WSAMAKESELECTREPLY(FD_ACCEPT, 0));
//                     /* Disable this event until the next accept(); */
//                     Socket->SharedData->AsyncDisabledEvents |= FD_ACCEPT;
//                 }
//                 break;
//
//             case AFD_EVENT_DISCONNECT:
//             case AFD_EVENT_ABORT:
//             case AFD_EVENT_CLOSE:
//                 if (0 != (Socket->SharedData->AsyncEvents & FD_CLOSE) &&
//                     0 == (Socket->SharedData->AsyncDisabledEvents & FD_CLOSE))
//                 {
//                     /* Make the Notification */
//                     (Upcalls.lpWPUPostMessage.unwrap())(Socket->SharedData->hWnd,
//                                                Socket->SharedData->wMsg,
//                                                Socket->Handle,
//                                                WSAMAKESELECTREPLY(FD_CLOSE, 0));
//                     /* Disable this event forever; */
//                     Socket->SharedData->AsyncDisabledEvents |= FD_CLOSE;
//                 }
//                  break;
//             /* FIXME: Support QOS */
//         }
//     }
//
//     /* Check if there are any events left for us to check */
//     if ((Socket->SharedData->AsyncEvents & (~Socket->SharedData->AsyncDisabledEvents)) == 0 )
//     {
//         return;
//     }
//
//     /* Keep Polling */
//     SockProcessAsyncSelect(Socket, AsyncData);
//     return;
// }
//
// VOID SockProcessAsyncSelect(PSOCKET_INFORMATION Socket, PASYNC_DATA AsyncData)
// {
//
//     ULONG lNetworkEvents;
//     NTSTATUS Status;
//
//     /* Set up the Async Data Event Info */
//     AsyncData->AsyncSelectInfo.Timeout.HighPart = 0x7FFFFFFF;
//     AsyncData->AsyncSelectInfo.Timeout.LowPart = 0xFFFFFFFF;
//     AsyncData->AsyncSelectInfo.HandleCount = 1;
//     AsyncData->AsyncSelectInfo.Exclusive = TRUE;
//     AsyncData->AsyncSelectInfo.Handles[0].Handle = Socket->Handle;
//     AsyncData->AsyncSelectInfo.Handles[0].Events = 0;
//
//     /* Remove unwanted events */
//     lNetworkEvents = Socket->SharedData->AsyncEvents & (~Socket->SharedData->AsyncDisabledEvents);
//
//     /* Set Events to wait for */
//     if (lNetworkEvents & FD_READ)
//     {
//         AsyncData->AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_RECEIVE;
//     }
//
//     if (lNetworkEvents & FD_WRITE)
//     {
//         AsyncData->AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_SEND;
//     }
//
//     if (lNetworkEvents & FD_OOB)
//     {
//         AsyncData->AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_OOB_RECEIVE;
//     }
//
//     if (lNetworkEvents & FD_ACCEPT)
//     {
//         AsyncData->AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_ACCEPT;
//     }
//
//     /* FIXME: THIS IS NOT RIGHT!!! HACK HACK HACK! */
//     if (lNetworkEvents & FD_CONNECT)
//     {
//         AsyncData->AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_CONNECT | AFD_EVENT_CONNECT_FAIL;
//     }
//
//     if (lNetworkEvents & FD_CLOSE)
//     {
//         AsyncData->AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_DISCONNECT | AFD_EVENT_ABORT | AFD_EVENT_CLOSE;
//     }
//
//     if (lNetworkEvents & FD_QOS)
//     {
//         AsyncData->AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_QOS;
//     }
//
//     if (lNetworkEvents & FD_GROUP_QOS)
//     {
//         AsyncData->AsyncSelectInfo.Handles[0].Events |= AFD_EVENT_GROUP_QOS;
//     }
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile (SockAsyncHelperAfdHandle,
//                                     NULL,
//                                     NULL,
//                                     AsyncData,
//                                     &AsyncData->IoStatusBlock,
//                                     IOCTL_AFD_SELECT,
//                                     &AsyncData->AsyncSelectInfo,
//                                     sizeof(AsyncData->AsyncSelectInfo),
//                                     &AsyncData->AsyncSelectInfo,
//                                     sizeof(AsyncData->AsyncSelectInfo));
//
//     /* I/O Manager Won't call the completion routine, let's do it manually */
//     if (NT_SUCCESS(Status))
//     {
//         return;
//     }
//     else
//     {
//         AsyncData->IoStatusBlock.Status = Status;
//         SockAsyncSelectCompletionRoutine(AsyncData, &AsyncData->IoStatusBlock);
//     }
// }
//
// VOID SockProcessQueuedAsyncSelect(PVOID Context, PIO_STATUS_BLOCK IoStatusBlock)
// {
//     PASYNC_DATA AsyncData = Context;
//     BOOL FreeContext = TRUE;
//     PSOCKET_INFORMATION Socket;
//
//     /* Get the Socket */
//     Socket = AsyncData->ParentSocket;
//
//     /* If someone closed it, stop the function */
//     if (Socket->SharedData->State != SocketClosed)
//     {
//         /* Check if the Sequence Number changed by now, in which case quit */
//         if (AsyncData->SequenceNumber == Socket->SharedData->SequenceNumber)
//         {
//             /* Do the actual select, if needed */
//             if ((Socket->SharedData->AsyncEvents & (~Socket->SharedData->AsyncDisabledEvents)))
//             {
//                 SockProcessAsyncSelect(Socket, AsyncData);
//                 FreeContext = FALSE;
//             }
//         }
//     }
//
//     /* Free the Context */
//     if (FreeContext)
//     {
//         HeapFree(GetProcessHeap(), 0, AsyncData);
//     }
//
//     return;
// }
//
// VOID
// SockReenableAsyncSelectEvent (IN PSOCKET_INFORMATION Socket,
//                               IN ULONG Event)
// {
//     PASYNC_DATA AsyncData;
//
//     /* Make sure the event is actually disabled */
//     if (!(Socket->SharedData->AsyncDisabledEvents & Event))
//     {
//         return;
//     }
//
//     /* Re-enable it */
//     Socket->SharedData->AsyncDisabledEvents &= ~Event;
//
//     /* Return if no more events are being polled */
//     if ((Socket->SharedData->AsyncEvents & (~Socket->SharedData->AsyncDisabledEvents)) == 0 )
//     {
//         return;
//     }
//
//     /* Wait on new events */
//     AsyncData = HeapAlloc(GetProcessHeap(), 0, sizeof(ASYNC_DATA));
//     if (!AsyncData) return;
//
//     /* Create the Asynch Thread if Needed */
//     SockCreateOrReferenceAsyncThread();
//
//     /* Increase the sequence number to stop anything else */
//     Socket->SharedData->SequenceNumber++;
//
//     /* Set up the Async Data */
//     AsyncData->ParentSocket = Socket;
//     AsyncData->SequenceNumber = Socket->SharedData->SequenceNumber;
//
//     /* Begin Async Select by using I/O Completion */
//     NtSetIoCompletion(SockAsyncCompletionPort,
//                      (PVOID)&SockProcessQueuedAsyncSelect,
//                      AsyncData,
//                      0,
//                      0);
//
//     /* All done */
//     return;
// }
//
// BOOL
// WINAPI
// DllMain(HANDLE hInstDll,
//         ULONG dwReason,
//         PVOID Reserved)
// {
//
//     switch (dwReason)
//     {
//     case DLL_PROCESS_ATTACH:
//
//         TRACE("Loading MSAFD.DLL \n");
//
//         /* Don't need thread attach notifications
//         so disable them to improve performance */
//         DisableThreadLibraryCalls(hInstDll);
//
//         /* List of DLL Helpers */
//         InitializeListHead(&SockHelpersListHead);
//
//         /* Heap to use when allocating */
//         GlobalHeap = GetProcessHeap();
//
//         /* Initialize the lock that protects our socket list */
//         InitializeCriticalSection(&SocketListLock);
//
//         TRACE("MSAFD.DLL has been loaded\n");
//
//         break;
//
//     case DLL_THREAD_ATTACH:
//         break;
//
//     case DLL_THREAD_DETACH:
//         break;
//
//     case DLL_PROCESS_DETACH:
//
//         /* Delete the socket list lock */
//         DeleteCriticalSection(&SocketListLock);
//
//         break;
//     }
//
//     TRACE("DllMain of msafd.dll (leaving)\n");
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
