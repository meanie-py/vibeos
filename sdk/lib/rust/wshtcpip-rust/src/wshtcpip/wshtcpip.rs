use crate::prelude::*;
use crate::wshtcpip::iflist::WSHIoctl_GetInterfaceList;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// wshtcpip.c file-scope preamble - EMPTY

// The C file contains only #define NDEBUG, which is a preprocessor directive
// that affects compilation but does not create any runtime declarations.
// There are no types, constants, static tables, or other file-scope declarations
// to translate.

macro_rules! RTL_CONSTANT_STRING {
    ($s:expr) => {{
        const S: &[u8] = $s;
        const LEN: usize = S.len() - 2;
        _UNICODE_STRING {
            Length: LEN as u16,
            MaximumLength: S.len() as u16,
            Buffer: S.as_ptr() as *mut u16,
        }
    }};
}
// `BOOL EXPORT DllMain` with EXPORT = WINAPI (wshtcpip.h). This was `extern "C"`, which on
// i386 defines `_DllMain` where the CRT calls `_DllMain@12`, so the link bound crt-startup's
// default DllMain instead and this one never ran (tools/scan_entry_point_cc.py).
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hInstDll: HANDLE,
    dwReason: ULONG,
    Reserved: PVOID,
) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hInstDll as HMODULE);
        }
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn WSHAddressToString(
    Address: LPSOCKADDR,
    AddressLength: INT,
    ProtocolInfo: LPWSAPROTOCOL_INFOW,
    AddressString: LPWSTR,
    AddressStringLength: LPDWORD,
) -> INT {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WSHAddressToString\0".as_ptr(),
        b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr(),
        51i32,
    );
    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "system" fn WSHEnumProtocols(
    lpiProtocols: LPINT,
    lpTransportKeyName: LPWSTR,
    lpProtocolBuffer: LPVOID,
    lpdwBufferLength: LPDWORD,
) -> INT {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WSHEnumProtocols\0".as_ptr() as *const core::ffi::c_char,
        b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const core::ffi::c_char,
        65 as core::ffi::c_int,
    );
    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "system" fn WSHGetBroadcastSockaddr(
    HelperDllSocketContext: PVOID,
    Sockaddr: PSOCKADDR,
    SockaddrLength: PINT,
) -> INT {
    let Size: INT = (2 * core::mem::size_of::<UINT>()) as INT;

    if *SockaddrLength < Size {
        return WSAEFAULT as INT;
    }

    RtlZeroMemory(Sockaddr as PVOID, *SockaddrLength as usize);

    (*Sockaddr).sa_family = AF_INET as USHORT;
    *((*Sockaddr).sa_data.as_mut_ptr() as PUINT) = INADDR_BROADCAST;

    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "system" fn WSHGetProviderGuid(
    ProviderName: LPWSTR,
    ProviderGuid: LPGUID,
) -> INT {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WSHGetProviderGuid\0".as_ptr() as *const core::ffi::c_char,
        b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const core::ffi::c_char,
        103 as core::ffi::c_int,
    );
    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "system" fn WSHGetSockaddrType(
    Sockaddr: PSOCKADDR,
    SockaddrLength: DWORD,
    SockaddrInfo: PSOCKADDR_INFO,
) -> INT {
    let ipv4 = Sockaddr as PSOCKADDR_IN;

    if ipv4.is_null()
        || SockaddrInfo.is_null()
        || (SockaddrLength as usize) < core::mem::size_of::<SOCKADDR_IN>()
        || (*ipv4).sin_family as i32 != AF_INET as i32
    {
        return WSAEINVAL as INT;
    }

    match ntohl((*ipv4).sin_addr.S_un.S_addr) {
        INADDR_ANY => {
            (*SockaddrInfo).AddressInfo = SockaddrAddressInfoWildcard;
        }
        INADDR_BROADCAST => {
            (*SockaddrInfo).AddressInfo = SockaddrAddressInfoBroadcast;
        }
        2130706433 => {
            (*SockaddrInfo).AddressInfo = SockaddrAddressInfoLoopback;
        }
        _ => {
            (*SockaddrInfo).AddressInfo = SockaddrAddressInfoNormal;
        }
    }

    if ntohs((*ipv4).sin_port) as i32 == 0 {
        (*SockaddrInfo).EndpointInfo = SockaddrEndpointInfoWildcard;
    } else if (ntohs((*ipv4).sin_port) as i32) < IPPORT_RESERVED as i32 {
        (*SockaddrInfo).EndpointInfo = SockaddrEndpointInfoReserved;
    } else {
        (*SockaddrInfo).EndpointInfo = SockaddrEndpointInfoNormal;
    }

    NO_ERROR as INT
}
// tcpioctl.h: #define TCP_SOCKET_KEEPALIVE 2 -- added to the header alongside this
// change (upstream 248aa75023a). The checked-in bindgen types predate it and
// gen_driver_types.sh cannot currently regenerate them (unrelated clang error in
// intrin_x86.h on the amd64 target), so it is declared here rather than left as a
// bare 2. Remove when types_*.rs are next regenerated.
const TCP_SOCKET_KEEPALIVE: ULONG = 2;

unsafe extern "C" fn GetTdiTypeId(
    Level: INT,
    OptionName: INT,
    TdiType: PULONG,
    TdiId: PULONG,
) {
    match Level as u32 {
        SOL_SOCKET => {
            // SO_KEEPALIVE is a CONNECTION-level TDI option, not an address-object
            // one, and this arm used to ASSERT rather than name its id -- so
            // setsockopt(SO_KEEPALIVE) reported success and did nothing at all.
            // Upstream-Commit: 248aa75023a
            *TdiType = INFO_TYPE_CONNECTION as ULONG;
            match OptionName as u32 {
                SO_KEEPALIVE => {
                    *TdiId = TCP_SOCKET_KEEPALIVE;
                    return;
                }
                _ => {}
            }
        }
        IPPROTO_IP => {
            *TdiType = INFO_TYPE_ADDRESS_OBJECT as ULONG;
            match OptionName as u32 {
                IP_TTL => {
                    *TdiId = AO_OPTION_TTL as ULONG;
                    return;
                }
                IP_DONTFRAGMENT => {
                    *TdiId = AO_OPTION_IP_DONTFRAGMENT as ULONG;
                    return;
                }
                IP_HDRINCL => {
                    *TdiId = AO_OPTION_IP_HDRINCL as ULONG;
                    return;
                }
                _ => {}
            }
        }
        6 => {
            *TdiType = INFO_TYPE_CONNECTION as ULONG;
            match OptionName as u32 {
                TCP_NODELAY => {
                    *TdiId = TCP_SOCKET_NODELAY as ULONG;
                    return;
                }
                _ => {}
            }
        }
        _ => {}
    }
    if DbgPrint(
        b"(%s:%d) Unknown level/option name: %d %d\n\0".as_ptr() as PCSTR,
        b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const ::core::ffi::c_char,
        224 as ::core::ffi::c_int,
        Level,
        OptionName,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const ::core::ffi::c_char,
            224 as ::core::ffi::c_int,
        );
    }
    *TdiType = 0 as ULONG;
    *TdiId = 0 as ULONG;
}
#[no_mangle]
pub unsafe extern "system" fn WSHGetSocketInformation(
    HelperDllSocketContext: PVOID,
    SocketHandle: SOCKET,
    TdiAddressObjectHandle: HANDLE,
    TdiConnectionObjectHandle: HANDLE,
    Level: INT,
    OptionName: INT,
    OptionValue: PCHAR,
    OptionLength: LPINT,
) -> INT {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WSHGetSocketInformation\0".as_ptr() as *const core::ffi::c_char,
        b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const core::ffi::c_char,
        241 as core::ffi::c_int,
    );
    if DbgPrint(
        b"(%s:%d) Get: Unknown level/option name: %d %d\n\0".as_ptr() as PCSTR,
        b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const core::ffi::c_char,
        243 as core::ffi::c_int,
        Level,
        OptionName,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const core::ffi::c_char,
            243 as core::ffi::c_int,
        );
    }
    *OptionLength = 0 as core::ffi::c_int;
    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "system" fn WSHGetWildcardSockaddr(
    HelperDllSocketContext: PVOID,
    Sockaddr: PSOCKADDR,
    SockaddrLength: PINT,
) -> INT {
    let Size: INT = (2 * core::mem::size_of::<UINT>()) as INT;

    if *SockaddrLength < Size {
        return WSAEFAULT as INT;
    }

    RtlZeroMemory(Sockaddr as PVOID, *SockaddrLength as usize);

    (*Sockaddr).sa_family = AF_INET as u16;
    *((*Sockaddr).sa_data.as_mut_ptr() as PUINT) = INADDR_ANY;

    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "system" fn WSHGetWinsockMapping(
    Mapping: PWINSOCK_MAPPING,
    MappingLength: DWORD,
) -> DWORD {
    let Rows: DWORD = 6;
    let Columns: DWORD = 3;
    let Size: DWORD = (2 * core::mem::size_of::<DWORD>() +
                       (Columns * Rows) as usize * core::mem::size_of::<DWORD>()) as DWORD;

    if MappingLength < Size {
        return Size;
    }

    (*Mapping).Rows = Rows;
    (*Mapping).Columns = Columns;

    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(0)).AddressFamily = AF_INET;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(0)).SocketType = SOCK_STREAM;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(0)).Protocol = 0;

    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(1)).AddressFamily = AF_INET;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(1)).SocketType = SOCK_STREAM;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(1)).Protocol = IPPROTO_TCP as DWORD;

    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(2)).AddressFamily = AF_INET;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(2)).SocketType = SOCK_DGRAM;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(2)).Protocol = 0;

    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(3)).AddressFamily = AF_INET;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(3)).SocketType = SOCK_DGRAM;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(3)).Protocol = IPPROTO_UDP as DWORD;

    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(4)).AddressFamily = AF_INET;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(4)).SocketType = SOCK_RAW;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(4)).Protocol = 0;

    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(5)).AddressFamily = AF_INET;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(5)).SocketType = SOCK_RAW;
    (*(core::ptr::addr_of_mut!((*Mapping).Mapping) as *mut WINSOCK_MAPPING_TRIPLE).offset(5)).Protocol = IPPROTO_ICMP as DWORD;

    NO_ERROR
}
#[no_mangle]
pub unsafe extern "system" fn WSHGetWSAProtocolInfo(
    ProviderName: LPWSTR,
    ProtocolInfo: *mut LPWSAPROTOCOL_INFOW,
    ProtocolInfoEntries: LPDWORD,
) -> INT {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WSHGetWSAProtocolInfo\0".as_ptr() as *const core::ffi::c_char,
        b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const core::ffi::c_char,
        331 as core::ffi::c_int,
    );
    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "system" fn WSHIoctl(
    HelperDllSocketContext: PVOID,
    SocketHandle: SOCKET,
    TdiAddressObjectHandle: HANDLE,
    TdiConnectionObjectHandle: HANDLE,
    IoControlCode: DWORD,
    InputBuffer: LPVOID,
    InputBufferLength: DWORD,
    OutputBuffer: LPVOID,
    OutputBufferLength: DWORD,
    NumberOfBytesReturned: LPDWORD,
    Overlapped: LPWSAOVERLAPPED,
    CompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
    NeedsCompletion: LPBOOL,
) -> INT {

    let res: INT;

    if IoControlCode == SIO_GET_INTERFACE_LIST {
        res = WSHIoctl_GetInterfaceList(
            OutputBuffer,
            OutputBufferLength,
            NumberOfBytesReturned,
            NeedsCompletion,
        );
        return res;
    }

    WSAEINVAL as INT
}
#[no_mangle]
pub unsafe extern "system" fn WSHJoinLeaf(
    HelperDllSocketContext: PVOID,
    SocketHandle: SOCKET,
    TdiAddressObjectHandle: HANDLE,
    TdiConnectionObjectHandle: HANDLE,
    LeafHelperDllSocketContext: PVOID,
    LeafSocketHandle: SOCKET,
    Sockaddr: PSOCKADDR,
    SockaddrLength: DWORD,
    CallerData: LPWSABUF,
    CalleeData: LPWSABUF,
    SocketQOS: LPQOS,
    GroupQOS: LPQOS,
    Flags: DWORD,
) -> INT {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WSHJoinLeaf\0".as_ptr() as *const core::ffi::c_char,
        b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const core::ffi::c_char,
        391 as core::ffi::c_int,
    );
    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "C" fn SendRequest(
    Request: PVOID,
    ReqSize: DWORD,
    IOCTL: DWORD,
) -> INT {
    let mut Success: BOOLEAN = 0;
    let mut TcpCC: HANDLE = core::ptr::null_mut();
    let mut BytesReturned: DWORD = 0;

    if openTcpFile(
        core::ptr::addr_of_mut!(TcpCC),
        (FILE_READ_DATA | FILE_WRITE_DATA) as ACCESS_MASK,
    ) != STATUS_SUCCESS
    {
        return WSAEINVAL as INT;
    }

    Success = DeviceIoControl(
        TcpCC,
        IOCTL,
        Request as LPVOID,
        ReqSize,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(BytesReturned),
        core::ptr::null_mut(),
    ) as BOOLEAN;

    closeTcpFile(TcpCC);

    if Success == 0 {
        return WSAEINVAL as INT;
    }

    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "system" fn WSHNotify(
    HelperDllSocketContext: PVOID,
    SocketHandle: SOCKET,
    TdiAddressObjectHandle: HANDLE,
    TdiConnectionObjectHandle: HANDLE,
    NotifyEvent: DWORD,
) -> INT {
    let Context: PSOCKET_CONTEXT = HelperDllSocketContext as PSOCKET_CONTEXT;
    let mut Status: NTSTATUS;
    let mut TcpCC: HANDLE = core::ptr::null_mut();
    let mut EntityIDs: *mut TDIEntityID = core::ptr::null_mut();
    let mut EntityCount: DWORD = 0;
    let mut i: DWORD;
    let mut QueuedRequest: PQUEUED_REQUEST;
    let mut NextQueuedRequest: PQUEUED_REQUEST;

    match NotifyEvent {
        WSH_NOTIFY_CLOSE => {
            QueuedRequest = (*Context).RequestQueue;
            while !QueuedRequest.is_null() {
                NextQueuedRequest = (*QueuedRequest).Next as *mut _QUEUED_REQUEST;

                HeapFree(GetProcessHeap(), 0, (*QueuedRequest).Info as LPVOID);
                HeapFree(GetProcessHeap(), 0, QueuedRequest as LPVOID);

                QueuedRequest = NextQueuedRequest;
            }
            HeapFree(GetProcessHeap(), 0, HelperDllSocketContext);
        }

        WSH_NOTIFY_BIND => {
            Status = openTcpFile(core::ptr::addr_of_mut!(TcpCC), FILE_READ_DATA);
            if Status != STATUS_SUCCESS {
                return WSAEINVAL as INT;
            }

            Status = tdiGetEntityIDSet(
                TcpCC,
                core::ptr::addr_of_mut!(EntityIDs),
                core::ptr::addr_of_mut!(EntityCount),
            );

            closeTcpFile(TcpCC);

            if Status != STATUS_SUCCESS {
                return WSAEINVAL as INT;
            }

            i = 0;
            while i < EntityCount {
                if (*EntityIDs.offset(i as isize)).tei_entity == CO_TL_ENTITY
                    || (*EntityIDs.offset(i as isize)).tei_entity == CL_TL_ENTITY
                    || (*EntityIDs.offset(i as isize)).tei_entity == ER_ENTITY
                {
                    (*Context).AddrFileInstance = (*EntityIDs.offset(i as isize)).tei_instance;
                    (*Context).AddrFileEntityType = (*EntityIDs.offset(i as isize)).tei_entity;
                }
                i = i.wrapping_add(1);
            }

            tdiFreeThingSet(EntityIDs as PVOID);

            (*Context).SocketState = SocketStateBound;

            QueuedRequest = (*Context).RequestQueue;
            while !QueuedRequest.is_null() {
                (*(*QueuedRequest).Info).ID.toi_entity.tei_entity = (*Context).AddrFileEntityType;
                (*(*QueuedRequest).Info).ID.toi_entity.tei_instance = (*Context).AddrFileInstance;

                SendRequest(
                    (*QueuedRequest).Info as PVOID,
                    (core::mem::size_of::<TCP_REQUEST_SET_INFORMATION_EX>() as DWORD)
                        .wrapping_add((*(*QueuedRequest).Info).BufferSize),
                    IOCTL_TCP_SET_INFORMATION_EX,
                );

                NextQueuedRequest = (*QueuedRequest).Next as *mut _QUEUED_REQUEST;

                HeapFree(GetProcessHeap(), 0, (*QueuedRequest).Info as LPVOID);
                HeapFree(GetProcessHeap(), 0, QueuedRequest as LPVOID);

                QueuedRequest = NextQueuedRequest;
            }
            (*Context).RequestQueue = core::ptr::null_mut();
        }

        _ => {}
    }

    NO_ERROR as INT
}
#[no_mangle]
pub unsafe extern "system" fn WSHOpenSocket(
    AddressFamily: PINT,
    SocketType: PINT,
    Protocol: PINT,
    TransportDeviceName: PUNICODE_STRING,
    HelperDllSocketContext: LPVOID,
    NotificationEvents: PDWORD,
) -> INT {
    return WSHOpenSocket2(
        AddressFamily,
        SocketType,
        Protocol,
        0,
        0,
        TransportDeviceName,
        HelperDllSocketContext as *mut LPVOID,
        NotificationEvents,
    );
}
#[no_mangle]
pub unsafe extern "system" fn WSHOpenSocket2(
    AddressFamily: PINT,
    SocketType: PINT,
    Protocol: PINT,
    Group: GROUP,
    Flags: DWORD,
    TransportDeviceName: PUNICODE_STRING,
    HelperDllSocketContext: *mut LPVOID,
    NotificationEvents: PDWORD,
) -> INT {
    let mut Context: PSOCKET_CONTEXT;
    let mut String: _UNICODE_STRING;
    let TcpDeviceName: _UNICODE_STRING = RTL_CONSTANT_STRING!(b"\\\0D\0e\0v\0i\0c\0e\0\\\0T\0c\0p\0\0\0");
    let UdpDeviceName: _UNICODE_STRING = RTL_CONSTANT_STRING!(b"\\\0D\0e\0v\0i\0c\0e\0\\\0U\0d\0p\0\0\0");
    let RawDeviceName: _UNICODE_STRING = RTL_CONSTANT_STRING!(b"\\\0D\0e\0v\0i\0c\0e\0\\\0R\0a\0w\0I\0p\0\0\0");

    match *SocketType as u32 {
        SOCK_STREAM => {
            String = TcpDeviceName;
        }
        SOCK_DGRAM => {
            String = UdpDeviceName;
        }
        SOCK_RAW => {
            if *Protocol < 0 || *Protocol > 255 {
                return WSAEINVAL as INT;
            }
            String = RawDeviceName;
        }
        _ => return WSAEINVAL as INT,
    }

    RtlInitUnicodeString(TransportDeviceName, core::ptr::null_mut());

    (*TransportDeviceName).MaximumLength = (String.Length as usize
        + (4 * core::mem::size_of::<u16>())
        + core::mem::size_of::<u16>()) as u16;

    (*TransportDeviceName).Buffer = HeapAlloc(
        GetProcessHeap(),
        0,
        (*TransportDeviceName).MaximumLength as usize,
    ) as *mut u16;

    if (*TransportDeviceName).Buffer.is_null() {
        return WSAENOBUFS as INT;
    }

    RtlAppendUnicodeStringToString(TransportDeviceName, &String);

    if *SocketType == SOCK_RAW as i32 {
        *(*TransportDeviceName).Buffer.offset(
            ((*TransportDeviceName).Length as usize / core::mem::size_of::<u16>()) as isize,
        ) = OBJ_NAME_PATH_SEPARATOR as u16;
        (*TransportDeviceName).Length += core::mem::size_of::<u16>() as u16;
        *(*TransportDeviceName).Buffer.offset(
            ((*TransportDeviceName).Length as usize / core::mem::size_of::<u16>()) as isize,
        ) = 0;

        String.Buffer = (*TransportDeviceName).Buffer.offset(
            ((*TransportDeviceName).Length as usize / core::mem::size_of::<u16>()) as isize,
        );
        String.Length = 0;
        String.MaximumLength = (*TransportDeviceName).MaximumLength - (*TransportDeviceName).Length;

        RtlIntegerToUnicodeString(*Protocol as u32, 10, &mut String);

        (*TransportDeviceName).Length += String.Length;
    }

    Context = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<SOCKET_CONTEXT>(),
    ) as PSOCKET_CONTEXT;

    if Context.is_null() {
        RtlFreeUnicodeString(TransportDeviceName);
        return WSAENOBUFS as INT;
    }

    (*Context).AddressFamily = *AddressFamily;
    (*Context).SocketType = *SocketType;
    (*Context).Protocol = *Protocol;
    (*Context).Flags = Flags;
    (*Context).SocketState = SocketStateCreated;

    *HelperDllSocketContext = Context as LPVOID;
    *NotificationEvents = WSH_NOTIFY_CLOSE | WSH_NOTIFY_BIND;

    return NO_ERROR as INT;
}
#[no_mangle]
pub unsafe extern "system" fn WSHSetSocketInformation(
    HelperDllSocketContext: PVOID,
    SocketHandle: SOCKET,
    TdiAddressObjectHandle: HANDLE,
    TdiConnectionObjectHandle: HANDLE,
    Level: INT,
    OptionName: INT,
    OptionValue: PCHAR,
    OptionLength: INT,
) -> INT {
    let Context: PSOCKET_CONTEXT = HelperDllSocketContext as PSOCKET_CONTEXT;
    let mut TdiType: ULONG = 0;
    let mut TdiId: ULONG = 0;
    let Status: INT;
    let Info: PTCP_REQUEST_SET_INFORMATION_EX;
    let mut Queued: PQUEUED_REQUEST;
    let mut NextQueued: PQUEUED_REQUEST;

    match Level as u32 {
        SOL_SOCKET => match OptionName as u32 {
            SO_DONTROUTE => {
                if (OptionLength as usize) < core::mem::size_of::<BOOL>() {
                    return WSAEFAULT as INT;
                }
                (*Context).DontRoute = *(OptionValue as *mut BOOL);
                return 0;
            }
            SO_KEEPALIVE => {
                // Was `return 0` -- a silent no-op that told the caller keepalive
                // was enabled. Validate and fall through to the TDI request path
                // below, the same one TCP_NODELAY uses. Upstream-Commit: 248aa75023a
                if (OptionLength as usize) < core::mem::size_of::<DWORD>() {
                    return WSAEFAULT as INT;
                }
            }
            _ => {
                return WSAENOPROTOOPT as INT;
            }
        },
        IPPROTO_IP => match OptionName as u32 {
            IP_TTL | IP_DONTFRAGMENT | IP_HDRINCL => {}
            _ => {
                return 0;
            }
        },
        IPPROTO_TCP => match OptionName as u32 {
            TCP_NODELAY => {
                if (OptionLength as usize) < core::mem::size_of::<CHAR>() {
                    return WSAEFAULT as INT;
                }
            }
            _ => {
                return 0;
            }
        },
        _ => {
            return 0;
        }
    }

    GetTdiTypeId(Level, OptionName, core::ptr::addr_of_mut!(TdiType), core::ptr::addr_of_mut!(TdiId));

    Info = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<_TCP_REQUEST_SET_INFORMATION_EX>().wrapping_add(OptionLength as usize),
    ) as PTCP_REQUEST_SET_INFORMATION_EX;
    if Info.is_null() {
        return WSAENOBUFS as INT;
    }

    (*Info).ID.toi_entity.tei_entity = (*Context).AddrFileEntityType as ULONG;
    (*Info).ID.toi_entity.tei_instance = (*Context).AddrFileInstance as ULONG;
    (*Info).ID.toi_class = INFO_CLASS_PROTOCOL as ULONG;
    (*Info).ID.toi_type = TdiType;
    (*Info).ID.toi_id = TdiId;
    (*Info).BufferSize = OptionLength as u32;
    memcpy(
        core::ptr::addr_of_mut!((*Info).Buffer) as *mut core::ffi::c_void,
        OptionValue as *const core::ffi::c_void,
        OptionLength as usize,
    );

    if (*Context).SocketState as u32 == SocketStateCreated as u32 {
        if !(*Context).RequestQueue.is_null() {
            Queued = (*Context).RequestQueue;
            loop {
                NextQueued = (*Queued).Next as PQUEUED_REQUEST;
                if NextQueued.is_null() {
                    break;
                }
                Queued = NextQueued;
            }

            (*Queued).Next = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<QUEUED_REQUEST>(),
            );
            if (*Queued).Next.is_null() {
                HeapFree(GetProcessHeap(), 0, Info as PVOID);
                return WSAENOBUFS as INT;
            }

            NextQueued = (*Queued).Next as PQUEUED_REQUEST;
            (*NextQueued).Next = core::ptr::null_mut();
            (*NextQueued).Info = Info;
        } else {
            (*Context).RequestQueue = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<QUEUED_REQUEST>(),
            ) as PQUEUED_REQUEST;
            if (*Context).RequestQueue.is_null() {
                HeapFree(GetProcessHeap(), 0, Info as PVOID);
                return WSAENOBUFS as INT;
            }

            (*(*Context).RequestQueue).Next = core::ptr::null_mut();
            (*(*Context).RequestQueue).Info = Info;
        }

        return 0;
    }

    Status = SendRequest(
        Info as PVOID,
        (core::mem::size_of::<_TCP_REQUEST_SET_INFORMATION_EX>()).wrapping_add((*Info).BufferSize as usize) as DWORD,
        IOCTL_TCP_SET_INFORMATION_EX as DWORD,
    );

    HeapFree(GetProcessHeap(), 0, Info as PVOID);

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn WSHStringToAddress(
    AddressString: LPWSTR,
    AddressFamily: DWORD,
    ProtocolInfo: LPWSAPROTOCOL_INFOW,
    Address: LPSOCKADDR,
    AddressStringLength: LPDWORD,
) -> INT {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"WSHStringToAddress\0".as_ptr() as *const core::ffi::c_char,
        b"dll/win32/wshtcpip/wshtcpip.c\0".as_ptr() as *const core::ffi::c_char,
        812 as core::ffi::c_int,
    );
    return NO_ERROR as INT;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS WinSock Helper DLL for TCP/IP
//  * FILE:        wshtcpip.c
//  * PURPOSE:     DLL entry
//  * PROGRAMMERS: Casper S. Hornstrup (chorns@users.sourceforge.net)
//  * REVISIONS:
//  *   CSH 01/09-2000 Created
//  */
// #include "wshtcpip.h"
// #define NDEBUG
// #include <debug.h>
//
// BOOL
// EXPORT
// DllMain(HANDLE hInstDll,
//         ULONG dwReason,
//         PVOID Reserved)
// {
//     DPRINT("DllMain of wshtcpip.dll\n");
//
//     switch (dwReason) {
//     case DLL_PROCESS_ATTACH:
//         /* Don't need thread attach notifications
//            so disable them to improve performance */
//         DisableThreadLibraryCalls(hInstDll);
//         break;
//
//     case DLL_THREAD_ATTACH:
//         break;
//
//     case DLL_THREAD_DETACH:
//         break;
//
//     case DLL_PROCESS_DETACH:
//         break;
//     }
//     return TRUE;
// }
//
//
// INT
// EXPORT
// WSHAddressToString(
//     IN      LPSOCKADDR Address,
//     IN      INT AddressLength,
//     IN      LPWSAPROTOCOL_INFOW ProtocolInfo    OPTIONAL,
//     OUT     LPWSTR AddressString,
//     IN OUT  LPDWORD AddressStringLength)
// {
//     UNIMPLEMENTED;
//
//     return NO_ERROR as INT;
// }
//
//
// INT
// EXPORT
// WSHEnumProtocols(
//     IN      LPINT lpiProtocols  OPTIONAL,
//     IN      LPWSTR lpTransportKeyName,
//     IN OUT  LPVOID lpProtocolBuffer,
//     IN OUT  LPDWORD lpdwBufferLength)
// {
//     UNIMPLEMENTED;
//
//     return NO_ERROR as INT;
// }
//
//
// INT
// EXPORT
// WSHGetBroadcastSockaddr(
//     IN  PVOID HelperDllSocketContext,
//     OUT PSOCKADDR Sockaddr,
//     OUT PINT SockaddrLength)
// {
//     INT Size = 2 * sizeof(UINT);
//
//     if (*SockaddrLength < Size)
//     {
//         DPRINT1("Socket address length too small: %d\n", *SockaddrLength);
//         return WSAEFAULT as INT;
//     }
//
//     RtlZeroMemory(Sockaddr, *SockaddrLength);
//
//     Sockaddr->sa_family = AF_INET;
//     *((PUINT)Sockaddr->sa_data) = INADDR_BROADCAST;
//
//     /* *SockaddrLength = Size; */
//
//     return NO_ERROR as INT;
// }
//
//
// INT
// EXPORT
// WSHGetProviderGuid(
//     IN  LPWSTR ProviderName,
//     OUT LPGUID ProviderGuid)
// {
//     UNIMPLEMENTED;
//
//     return NO_ERROR as INT;
// }
//
//
// /*
// Document from OSR how WSHGetSockaddrType works
// http://www.osronline.com/ddkx/network/37wshfun_5lyq.htm
// */
//
// INT
// EXPORT
// WSHGetSockaddrType(
//     IN  PSOCKADDR Sockaddr,
//     IN  DWORD SockaddrLength,
//     OUT PSOCKADDR_INFO SockaddrInfo)
// {
//     PSOCKADDR_IN ipv4 = (PSOCKADDR_IN)Sockaddr;
//
//     if (!ipv4 || !SockaddrInfo || SockaddrLength < sizeof(SOCKADDR_IN) ||
//         ipv4->sin_family != AF_INET)
//     {
//         DPRINT1("Invalid parameter: %x %x %d %u\n", ipv4, SockaddrInfo, SockaddrLength, (ipv4 ? ipv4->sin_family : 0));
//         return WSAEINVAL as INT;
//     }
//
//     switch (ntohl(ipv4->sin_addr.s_addr))
//     {
//       case INADDR_ANY:
//            SockaddrInfo->AddressInfo = SockaddrAddressInfoWildcard;
//            break;
//
//       case INADDR_BROADCAST:
//            SockaddrInfo->AddressInfo = SockaddrAddressInfoBroadcast;
//            break;
//
//       case INADDR_LOOPBACK:
//            SockaddrInfo->AddressInfo = SockaddrAddressInfoLoopback;
//            break;
//
//       default:
//            SockaddrInfo->AddressInfo = SockaddrAddressInfoNormal;
// 	   break;
//     }
//
//     if (ntohs(ipv4->sin_port) == 0)
//  	SockaddrInfo->EndpointInfo = SockaddrEndpointInfoWildcard;
//     else if (ntohs(ipv4->sin_port) < IPPORT_RESERVED)
// 	SockaddrInfo->EndpointInfo = SockaddrEndpointInfoReserved;
//     else
//         SockaddrInfo->EndpointInfo = SockaddrEndpointInfoNormal;
//
//     return NO_ERROR as INT;
// }
//
// static
// void
// GetTdiTypeId(
//      INT Level,
//      INT OptionName,
//      PULONG TdiType,
//      PULONG TdiId)
// {
//     switch (Level)
//     {
//        case SOL_SOCKET:
//           *TdiType = INFO_TYPE_ADDRESS_OBJECT;
//           switch (OptionName)
//           {
//              case SO_KEEPALIVE:
//                 /* FIXME: Return proper option */
//                 ASSERT(FALSE);
//                 break;
//              default:
//                 break;
//           }
//           break;
//
//        case IPPROTO_IP:
//           *TdiType = INFO_TYPE_ADDRESS_OBJECT;
//           switch (OptionName)
//           {
//              case IP_TTL:
//                 *TdiId = AO_OPTION_TTL;
//                 return;
//
//              case IP_DONTFRAGMENT:
//                  *TdiId = AO_OPTION_IP_DONTFRAGMENT;
//                  return;
//
//
//              case IP_HDRINCL:
//                  *TdiId = AO_OPTION_IP_HDRINCL;
//                  return;
//
//              default:
//                 break;
//           }
//           break;
//
//        case IPPROTO_TCP:
//           *TdiType = INFO_TYPE_CONNECTION;
//           switch (OptionName)
//           {
//              case TCP_NODELAY:
//                  *TdiId = TCP_SOCKET_NODELAY;
//                  return;
//              default:
//                  break;
//           }
//
//        default:
//           break;
//     }
//
//     DPRINT1("Unknown level/option name: %d %d\n", Level, OptionName);
//     *TdiType = 0;
//     *TdiId = 0;
// }
//
// INT
// EXPORT
// WSHGetSocketInformation(
//     IN  PVOID HelperDllSocketContext,
//     IN  SOCKET SocketHandle,
//     IN  HANDLE TdiAddressObjectHandle,
//     IN  HANDLE TdiConnectionObjectHandle,
//     IN  INT Level,
//     IN  INT OptionName,
//     OUT PCHAR OptionValue,
//     OUT LPINT OptionLength)
// {
//     UNIMPLEMENTED;
//
//     DPRINT1("Get: Unknown level/option name: %d %d\n", Level, OptionName);
//
//     *OptionLength = 0;
//
//     return NO_ERROR as INT;
// }
//
//
// INT
// EXPORT
// WSHGetWildcardSockaddr(
//     IN  PVOID HelperDllSocketContext,
//     OUT PSOCKADDR Sockaddr,
//     OUT PINT SockaddrLength)
// {
//     INT Size = 2 * sizeof(UINT);
//
//     if (*SockaddrLength < Size)
//     {
//         DPRINT1("Socket address length too small: %d\n", *SockaddrLength);
//         return WSAEFAULT as INT;
//     }
//
//     RtlZeroMemory(Sockaddr, *SockaddrLength);
//
//     Sockaddr->sa_family = AF_INET;
//     *((PUINT)Sockaddr->sa_data) = INADDR_ANY;
//
//     /* *SockaddrLength = Size; */
//
//     return NO_ERROR as INT;
// }
//
//
// DWORD
// EXPORT
// WSHGetWinsockMapping(
//     OUT PWINSOCK_MAPPING Mapping,
//     IN  DWORD MappingLength)
// {
//     DWORD Rows = 6;
//     DWORD Columns = 3;
//     DWORD Size = 2 * sizeof(DWORD) + Columns * Rows * sizeof(DWORD);
//
//     if (MappingLength < Size)
//     {
//         DPRINT1("Mapping length too small: %d\n", MappingLength);
//         return Size;
//     }
//
//     Mapping->Rows = Rows;
//     Mapping->Columns = Columns;
//
//     Mapping->Mapping[0].AddressFamily = AF_INET;
//     Mapping->Mapping[0].SocketType = SOCK_STREAM;
//     Mapping->Mapping[0].Protocol = 0;
//
//     Mapping->Mapping[1].AddressFamily = AF_INET;
//     Mapping->Mapping[1].SocketType = SOCK_STREAM;
//     Mapping->Mapping[1].Protocol = IPPROTO_TCP;
//
//     Mapping->Mapping[2].AddressFamily = AF_INET;
//     Mapping->Mapping[2].SocketType = SOCK_DGRAM;
//     Mapping->Mapping[2].Protocol = 0;
//
//     Mapping->Mapping[3].AddressFamily = AF_INET;
//     Mapping->Mapping[3].SocketType = SOCK_DGRAM;
//     Mapping->Mapping[3].Protocol = IPPROTO_UDP;
//
//     Mapping->Mapping[4].AddressFamily = AF_INET;
//     Mapping->Mapping[4].SocketType = SOCK_RAW;
//     Mapping->Mapping[4].Protocol = 0;
//
//     Mapping->Mapping[5].AddressFamily = AF_INET;
//     Mapping->Mapping[5].SocketType = SOCK_RAW;
//     Mapping->Mapping[5].Protocol = IPPROTO_ICMP;
//
//     return NO_ERROR as INT;
// }
//
//
// INT
// EXPORT
// WSHGetWSAProtocolInfo(
//     IN  LPWSTR ProviderName,
//     OUT LPWSAPROTOCOL_INFOW *ProtocolInfo,
//     OUT LPDWORD ProtocolInfoEntries)
// {
//     UNIMPLEMENTED;
//
//     return NO_ERROR as INT;
// }
//
//
// INT
// EXPORT
// WSHIoctl(
//     IN  PVOID HelperDllSocketContext,
//     IN  SOCKET SocketHandle,
//     IN  HANDLE TdiAddressObjectHandle,
//     IN  HANDLE TdiConnectionObjectHandle,
//     IN  DWORD IoControlCode,
//     IN  LPVOID InputBuffer,
//     IN  DWORD InputBufferLength,
//     IN  LPVOID OutputBuffer,
//     IN  DWORD OutputBufferLength,
//     OUT LPDWORD NumberOfBytesReturned,
//     IN  LPWSAOVERLAPPED Overlapped,
//     IN  LPWSAOVERLAPPED_COMPLETION_ROUTINE CompletionRoutine,
//     OUT LPBOOL NeedsCompletion)
// {
//     INT res;
//
//     if (IoControlCode == SIO_GET_INTERFACE_LIST)
//     {
//         res = WSHIoctl_GetInterfaceList(
//             OutputBuffer,
//             OutputBufferLength,
//             NumberOfBytesReturned,
//             NeedsCompletion);
//         return res;
//     }
//
//     UNIMPLEMENTED;
//
//     DPRINT1("Ioctl: Unknown IOCTL code: %x\n", IoControlCode);
//
//     return WSAEINVAL as INT;
// }
//
//
// INT
// EXPORT
// WSHJoinLeaf(
//     IN  PVOID HelperDllSocketContext,
//     IN  SOCKET SocketHandle,
//     IN  HANDLE TdiAddressObjectHandle,
//     IN  HANDLE TdiConnectionObjectHandle,
//     IN  PVOID LeafHelperDllSocketContext,
//     IN  SOCKET LeafSocketHandle,
//     IN  PSOCKADDR Sockaddr,
//     IN  DWORD SockaddrLength,
//     IN  LPWSABUF CallerData,
//     IN  LPWSABUF CalleeData,
//     IN  LPQOS SocketQOS,
//     IN  LPQOS GroupQOS,
//     IN  DWORD Flags)
// {
//     UNIMPLEMENTED;
//
//     return NO_ERROR as INT;
// }
//
// INT
// SendRequest(
//     IN PVOID Request,
//     IN DWORD ReqSize,
//     IN DWORD IOCTL)
// {
//     BOOLEAN Success;
//     HANDLE TcpCC;
//     DWORD BytesReturned;
//
//     if (openTcpFile(&TcpCC, FILE_READ_DATA | FILE_WRITE_DATA) != STATUS_SUCCESS)
//         return WSAEINVAL as INT;
//
//     Success = DeviceIoControl(TcpCC,
//                               IOCTL,
//                               Request,
//                               ReqSize,
//                               core::ptr::null_mut(),
//                               0,
//                               &BytesReturned,
//                               core::ptr::null_mut());
//
//     closeTcpFile(TcpCC);
//
//     DPRINT("DeviceIoControl: %ld\n", (Success ? NO_ERROR : GetLastError()));
//
//     if (!Success)
//         return WSAEINVAL as INT;
//
//     return NO_ERROR as INT;
// }
//
// INT
// EXPORT
// WSHNotify(
//     IN  PVOID HelperDllSocketContext,
//     IN  SOCKET SocketHandle,
//     IN  HANDLE TdiAddressObjectHandle,
//     IN  HANDLE TdiConnectionObjectHandle,
//     IN  DWORD NotifyEvent)
// {
//     PSOCKET_CONTEXT Context = HelperDllSocketContext;
//     NTSTATUS Status;
//     HANDLE TcpCC;
//     TDIEntityID *EntityIDs;
//     DWORD EntityCount, i;
//     PQUEUED_REQUEST QueuedRequest, NextQueuedRequest;
//
//     switch (NotifyEvent)
//     {
//         case WSH_NOTIFY_CLOSE:
//             DPRINT("WSHNotify: WSH_NOTIFY_CLOSE\n");
//             QueuedRequest = Context->RequestQueue;
//             while (QueuedRequest)
//             {
//                 NextQueuedRequest = QueuedRequest->Next;
//
//                 HeapFree(GetProcessHeap(), 0, QueuedRequest->Info);
//                 HeapFree(GetProcessHeap(), 0, QueuedRequest);
//
//                 QueuedRequest = NextQueuedRequest;
//             }
//             HeapFree(GetProcessHeap(), 0, HelperDllSocketContext);
//             break;
//
//
//         case WSH_NOTIFY_BIND:
//             DPRINT("WSHNotify: WSH_NOTIFY_BIND\n");
//             Status = openTcpFile(&TcpCC, FILE_READ_DATA);
//             if (Status != STATUS_SUCCESS)
//                 return WSAEINVAL as INT;
//
//             Status = tdiGetEntityIDSet(TcpCC,
//                                        &EntityIDs,
//                                        &EntityCount);
//
//             closeTcpFile(TcpCC);
//
//             if (Status != STATUS_SUCCESS)
//                 return WSAEINVAL as INT;
//
//             for (i = 0; i < EntityCount; i++)
//             {
//                 if (EntityIDs[i].tei_entity == CO_TL_ENTITY ||
//                     EntityIDs[i].tei_entity == CL_TL_ENTITY ||
//                     EntityIDs[i].tei_entity == ER_ENTITY)
//                 {
//                     Context->AddrFileInstance = EntityIDs[i].tei_instance;
//                     Context->AddrFileEntityType = EntityIDs[i].tei_entity;
//                 }
//             }
//
//             DPRINT("Instance: %lx Type: %lx\n", Context->AddrFileInstance, Context->AddrFileEntityType);
//
//             tdiFreeThingSet(EntityIDs);
//
//             Context->SocketState = SocketStateBound;
//
//             QueuedRequest = Context->RequestQueue;
//             while (QueuedRequest)
//             {
//                 QueuedRequest->Info->ID.toi_entity.tei_entity = Context->AddrFileEntityType;
//                 QueuedRequest->Info->ID.toi_entity.tei_instance = Context->AddrFileInstance;
//
//                 SendRequest(QueuedRequest->Info,
//                             sizeof(*QueuedRequest->Info) + QueuedRequest->Info->BufferSize,
//                             IOCTL_TCP_SET_INFORMATION_EX);
//
//                 NextQueuedRequest = QueuedRequest->Next;
//
//                 HeapFree(GetProcessHeap(), 0, QueuedRequest->Info);
//                 HeapFree(GetProcessHeap(), 0, QueuedRequest);
//
//                 QueuedRequest = NextQueuedRequest;
//             }
//             Context->RequestQueue = core::ptr::null_mut();
//             break;
//
//         default:
//             DPRINT1("Unwanted notification received! (%ld)\n", NotifyEvent);
//             break;
//     }
//
//     return NO_ERROR as INT;
// }
//
//
// INT
// EXPORT
// WSHOpenSocket(
//     IN OUT  PINT AddressFamily,
//     IN OUT  PINT SocketType,
//     IN OUT  PINT Protocol,
//     OUT     PUNICODE_STRING TransportDeviceName,
//     OUT     PVOID HelperDllSocketContext,
//     OUT     PDWORD NotificationEvents)
// /*
//  * FUNCTION: Opens a socket
//  */
// {
//     return WSHOpenSocket2(AddressFamily,
//                           SocketType,
//                           Protocol,
//                           0,
//                           0,
//                           TransportDeviceName,
//                           HelperDllSocketContext,
//                           NotificationEvents);
// }
//
//
// INT
// EXPORT
// WSHOpenSocket2(
//     OUT PINT AddressFamily,
//     IN  OUT PINT SocketType,
//     IN  OUT PINT Protocol,
//     IN  GROUP Group,
//     IN  DWORD Flags,
//     OUT PUNICODE_STRING TransportDeviceName,
//     OUT PVOID *HelperDllSocketContext,
//     OUT PDWORD NotificationEvents)
// /*
//  * FUNCTION: Opens a socket
//  * ARGUMENTS:
//  *     AddressFamily          = Address of buffer with address family (updated)
//  *     SocketType             = Address of buffer with type of socket (updated)
//  *     Protocol               = Address of buffer with protocol number (updated)
//  *     Group                  = Socket group
//  *     Flags                  = Socket flags
//  *     TransportDeviceName    = Address of buffer to place name of transport device
//  *     HelperDllSocketContext = Address of buffer to place socket context pointer
//  *     NotificationEvents     = Address of buffer to place flags for event notification
//  * RETURNS:
//  *     Status of operation
//  * NOTES:
//  *     Mapping tripple is returned in an canonicalized form
//  */
// {
//     PSOCKET_CONTEXT Context;
//     UNICODE_STRING String;
//     UNICODE_STRING TcpDeviceName = RTL_CONSTANT_STRING(DD_TCP_DEVICE_NAME);
//     UNICODE_STRING UdpDeviceName = RTL_CONSTANT_STRING(DD_UDP_DEVICE_NAME);
//     UNICODE_STRING RawDeviceName = RTL_CONSTANT_STRING(DD_RAW_IP_DEVICE_NAME);
//
//     DPRINT("WSHOpenSocket2 called\n");
//
//     switch (*SocketType) {
//     case SOCK_STREAM:
//         String = TcpDeviceName;
//         break;
//
//     case SOCK_DGRAM:
//         String = UdpDeviceName;
//         break;
//
//     case SOCK_RAW:
//         if ((*Protocol < 0) || (*Protocol > 255))
//           return WSAEINVAL as INT;
//
//         String = RawDeviceName;
//         break;
//
//     default:
//         return WSAEINVAL as INT;
//     }
//
//     RtlInitUnicodeString(TransportDeviceName, core::ptr::null_mut());
//
//     TransportDeviceName->MaximumLength = String.Length +        /* Transport device name */
//                                          (4 * sizeof(WCHAR) +   /* Separator and protocol */
//                                          sizeof(UNICODE_NULL)); /* Terminating null */
//
//     TransportDeviceName->Buffer = HeapAlloc(
//         GetProcessHeap(),
//         0,
//         TransportDeviceName->MaximumLength);
//
//     if (!TransportDeviceName->Buffer)
//         return WSAENOBUFS as INT;
//
//     /* Append the transport device name */
//     RtlAppendUnicodeStringToString(TransportDeviceName, &String);
//
//     if (*SocketType == SOCK_RAW) {
//         /* Append a separator */
//         TransportDeviceName->Buffer[TransportDeviceName->Length / sizeof(WCHAR)] = OBJ_NAME_PATH_SEPARATOR;
//         TransportDeviceName->Length += sizeof(WCHAR);
//         TransportDeviceName->Buffer[TransportDeviceName->Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//         /* Append the protocol number */
//         String.Buffer = TransportDeviceName->Buffer + (TransportDeviceName->Length / sizeof(WCHAR));
//         String.Length = 0;
//         String.MaximumLength = TransportDeviceName->MaximumLength - TransportDeviceName->Length;
//
//         RtlIntegerToUnicodeString((ULONG)*Protocol, 10, &String);
//
//         TransportDeviceName->Length += String.Length;
//     }
//
//     /* Setup a socket context area */
//
//     Context = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(SOCKET_CONTEXT));
//     if (!Context) {
//         RtlFreeUnicodeString(TransportDeviceName);
//         return WSAENOBUFS as INT;
//     }
//
//     Context->AddressFamily = *AddressFamily;
//     Context->SocketType    = *SocketType;
//     Context->Protocol      = *Protocol;
//     Context->Flags         = Flags;
//     Context->SocketState   = SocketStateCreated;
//
//     *HelperDllSocketContext = Context;
//     *NotificationEvents = WSH_NOTIFY_CLOSE | WSH_NOTIFY_BIND;
//
//     return NO_ERROR as INT;
// }
//
// INT
// EXPORT
// WSHSetSocketInformation(
//     IN  PVOID HelperDllSocketContext,
//     IN  SOCKET SocketHandle,
//     IN  HANDLE TdiAddressObjectHandle,
//     IN  HANDLE TdiConnectionObjectHandle,
//     IN  INT Level,
//     IN  INT OptionName,
//     IN  PCHAR OptionValue,
//     IN  INT OptionLength)
// {
//     PSOCKET_CONTEXT Context = HelperDllSocketContext;
//     ULONG TdiType, TdiId;
//     INT Status;
//     PTCP_REQUEST_SET_INFORMATION_EX Info;
//     PQUEUED_REQUEST Queued, NextQueued;
//
//     DPRINT("WSHSetSocketInformation\n");
//
//     /* FIXME: We only handle address file object here */
//
//     switch (Level)
//     {
//         case SOL_SOCKET:
//             switch (OptionName)
//             {
//                 case SO_DONTROUTE:
//                     if (OptionLength < sizeof(BOOL))
//                     {
//                         return WSAEFAULT as INT;
//                     }
//                     Context->DontRoute = *(BOOL*)OptionValue;
//                     /* This is silently ignored on Windows */
//                     return 0;
//
//                 case SO_KEEPALIVE:
//                     /* FIXME -- We'll send this to TCPIP */
//                     DPRINT1("Set: SO_KEEPALIVE not yet supported\n");
//                     return 0;
//
//                 default:
//                     /* Invalid option */
//                     DPRINT1("Set: Received unexpected SOL_SOCKET option %d\n", OptionName);
//                     return WSAENOPROTOOPT as INT;
//             }
//             break;
//
//         case IPPROTO_IP:
//             switch (OptionName)
//             {
//                 case IP_TTL:
//                 case IP_DONTFRAGMENT:
//                 case IP_HDRINCL:
//                     /* Send these to TCPIP */
//                     break;
//
//                 default:
//                     /* Invalid option -- FIXME */
//                     DPRINT1("Set: Received unsupported IPPROTO_IP option %d\n", OptionName);
//                     return 0;
//             }
//             break;
//
//         case IPPROTO_TCP:
//             switch (OptionName)
//             {
//                 case TCP_NODELAY:
//                     if (OptionLength < sizeof(CHAR))
//                     {
//                         return WSAEFAULT as INT;
//                     }
//                     break;
//
//                 default:
//                     /* Invalid option */
//                     DPRINT1("Set: Received unexpected IPPROTO_TCP option %d\n", OptionName);
//                     return 0;
//             }
//             break;
//
//         default:
//             DPRINT1("Set: Received unexpected %d option %d\n", Level, OptionName);
//             return 0;
//     }
//
//     /* If we get here, GetAddressOption must return something valid */
//     GetTdiTypeId(Level, OptionName, &TdiType, &TdiId);
//     ASSERT((TdiId != 0) && (TdiType != 0));
//
//     Info = HeapAlloc(GetProcessHeap(), 0, sizeof(*Info) + OptionLength);
//     if (!Info)
//         return WSAENOBUFS as INT;
//
//     Info->ID.toi_entity.tei_entity = Context->AddrFileEntityType;
//     Info->ID.toi_entity.tei_instance = Context->AddrFileInstance;
//     Info->ID.toi_class = INFO_CLASS_PROTOCOL;
//     Info->ID.toi_type = TdiType;
//     Info->ID.toi_id = TdiId;
//     Info->BufferSize = OptionLength;
//     memcpy(Info->Buffer, OptionValue, OptionLength);
//
//     if (Context->SocketState == SocketStateCreated)
//     {
//         if (Context->RequestQueue)
//         {
//             Queued = Context->RequestQueue;
//             while ((NextQueued = Queued->Next))
//             {
//                Queued = NextQueued;
//             }
//
//             Queued->Next = HeapAlloc(GetProcessHeap(), 0, sizeof(QUEUED_REQUEST));
//             if (!Queued->Next)
//             {
//                 HeapFree(GetProcessHeap(), 0, Info);
//                 return WSAENOBUFS as INT;
//             }
//
//             NextQueued = Queued->Next;
//             NextQueued->Next = core::ptr::null_mut();
//             NextQueued->Info = Info;
//         }
//         else
//         {
//             Context->RequestQueue = HeapAlloc(GetProcessHeap(), 0, sizeof(QUEUED_REQUEST));
//             if (!Context->RequestQueue)
//             {
//                 HeapFree(GetProcessHeap(), 0, Info);
//                 return WSAENOBUFS as INT;
//             }
//
//             Context->RequestQueue->Next = core::ptr::null_mut();
//             Context->RequestQueue->Info = Info;
//         }
//
//         return 0;
//     }
//
//     Status = SendRequest(Info, sizeof(*Info) + Info->BufferSize, IOCTL_TCP_SET_INFORMATION_EX);
//
//     HeapFree(GetProcessHeap(), 0, Info);
//
//     return Status;
// }
//
//
// INT
// EXPORT
// WSHStringToAddress(
//     IN      LPWSTR AddressString,
//     IN      DWORD AddressFamily,
//     IN      LPWSAPROTOCOL_INFOW ProtocolInfo    OPTIONAL,
//     OUT     LPSOCKADDR Address,
//     IN OUT  LPDWORD AddressStringLength)
// {
//     UNIMPLEMENTED;
//
//     return NO_ERROR as INT;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
