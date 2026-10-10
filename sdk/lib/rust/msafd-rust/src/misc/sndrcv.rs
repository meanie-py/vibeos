use crate::types::*;
use crate::glue::*;
use core::ffi::c_void;

// HeapFree is stdcall kernel32 -> extern "system" (i386 `_HeapFree@12`). ERR/TRACE shadowed
// by glue macros; SockReenableAsyncSelectEvent is msafd-internal (def is #[no_mangle] extern "C").
extern "system" {
    fn HeapFree(heap: HANDLE, flags: DWORD, mem: LPVOID) -> BOOL;
}
extern "C" {
    fn SockReenableAsyncSelectEvent(socket: PVOID, event: LONG);
}
#[no_mangle]
pub unsafe extern "system" fn WSPAsyncSelect(
    Handle: SOCKET,
    hWnd: HWND,
    wMsg: UINT,
    lEvent: LONG,
    lpErrno: LPINT,
) -> INT {
    let mut Socket: PSOCKET_INFORMATION = core::ptr::null_mut();
    let mut AsyncData: PASYNC_DATA;
    let mut BlockMode: BOOLEAN;

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        *lpErrno = WSAENOTSOCK as i32;
        return SOCKET_ERROR;
    }

    AsyncData = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<ASYNC_DATA>(),
    ) as PASYNC_DATA;
    if AsyncData.is_null() {
        MsafdReturnWithErrno(
            STATUS_INSUFFICIENT_RESOURCES,
            lpErrno,
            0,
            core::ptr::null_mut(),
        );
        return INVALID_SOCKET as i32;
    }

    BlockMode = 1u8;
    SetSocketInformation(
        Socket,
        AFD_INFO_BLOCKING_MODE,
        &mut BlockMode as *mut _ as PBOOLEAN,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        None,
    );
    (*(*Socket).SharedData).__bindgen_anon_1.set_NonBlocking(1u8);

    if (*(*Socket).SharedData).AsyncEvents != 0 {
        if WSPEventSelect(Handle, core::ptr::null_mut(), 0, lpErrno) == SOCKET_ERROR {
            HeapFree(GetProcessHeap(), 0, AsyncData as PVOID);
            return SOCKET_ERROR;
        }
    }

    SockCreateOrReferenceAsyncThread();

    SockGetAsyncSelectHelperAfdHandle();

    (*(*Socket).SharedData).hWnd = hWnd;
    (*(*Socket).SharedData).wMsg = wMsg;
    (*(*Socket).SharedData).AsyncEvents = lEvent;
    (*(*Socket).SharedData).AsyncDisabledEvents = 0;
    (*(*Socket).SharedData).SequenceNumber = (*(*Socket).SharedData).SequenceNumber.wrapping_add(1);

    if ((*(*Socket).SharedData).AsyncEvents & (!(*(*Socket).SharedData).AsyncDisabledEvents)) == 0 {
        HeapFree(GetProcessHeap(), 0, AsyncData as PVOID);
        return 0;
    }

    (*AsyncData).ParentSocket = Socket;
    (*AsyncData).SequenceNumber = (*(*Socket).SharedData).SequenceNumber;

    NtSetIoCompletion(
        SockAsyncCompletionPort,
        &SockProcessQueuedAsyncSelect as *const _ as PVOID,
        AsyncData as PVOID,
        0,
        0,
    );

    return ERROR_SUCCESS as i32;
}
#[no_mangle]
pub unsafe extern "system" fn WSPGetOverlappedResult(
    Handle: SOCKET,
    lpOverlapped: LPWSAOVERLAPPED,
    lpdwBytes: LPDWORD,
    fWait: BOOL,
    lpdwFlags: LPDWORD,
    lpErrno: LPINT,
) -> BOOL {
    let Socket: PSOCKET_INFORMATION;
    let IoStatusBlock: PIO_STATUS_BLOCK;
    let Ret: BOOL;

    TRACE!(b"Called (%x)\n\0".as_ptr() as *const i8, Handle);

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAENOTSOCK as i32;
        }
        return 0;
    }
    if lpOverlapped.is_null() || lpdwBytes.is_null() || lpdwFlags.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAEFAULT as i32;
        }
        return 0;
    }

    IoStatusBlock = lpOverlapped as PIO_STATUS_BLOCK;
    Ret = GetOverlappedResult(Handle as HANDLE, lpOverlapped, lpdwBytes, fWait);

    SleepEx(0, 1);

    if fWait == 0 && (*IoStatusBlock).__bindgen_anon_1.Status == STATUS_PENDING {
        if !lpErrno.is_null() {
            *lpErrno = WSA_IO_INCOMPLETE as i32;
        }
        return 0;
    }

    if Ret != 0 {
        SockReenableAsyncSelectEvent(Socket as PVOID, FD_OOB as LONG);
        SockReenableAsyncSelectEvent(Socket as PVOID, FD_WRITE as LONG);
        SockReenableAsyncSelectEvent(Socket as PVOID, FD_READ as LONG);
    }

    match (*IoStatusBlock).__bindgen_anon_1.Status {
        STATUS_RECEIVE_EXPEDITED => {
            *lpdwFlags = MSG_OOB;
        }
        STATUS_RECEIVE_PARTIAL_EXPEDITED => {
            *lpdwFlags = MSG_PARTIAL | MSG_OOB;
        }
        STATUS_RECEIVE_PARTIAL => {
            *lpdwFlags = MSG_PARTIAL;
        }
        _ => {
            *lpdwFlags = 0;
            if !lpErrno.is_null() {
                *lpErrno = TranslateNtStatusError((*IoStatusBlock).__bindgen_anon_1.Status);
            }
        }
    }

    Ret
}

unsafe extern "system" fn MsafdRecvAPC(
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Reserved: ULONG,
) {
    let Context = ApcContext as PMSAFD_RECV_APC_CONTEXT;

    TRACE!(
        b"MsafdRecvAPC(%p %lx %lx)\n\0".as_ptr() as PCSTR,
        ApcContext,
        (*IoStatusBlock).__bindgen_anon_1.Status,
        (*IoStatusBlock).Information,
    );

    if (*IoStatusBlock).__bindgen_anon_1.Status == STATUS_RECEIVE_EXPEDITED
        || (*IoStatusBlock).__bindgen_anon_1.Status == STATUS_RECEIVE_PARTIAL_EXPEDITED
    {
        SockReenableAsyncSelectEvent((*Context).lpSocket as PVOID, FD_OOB as LONG);
    } else {
        SockReenableAsyncSelectEvent((*Context).lpSocket as PVOID, FD_READ as LONG);
    }

    if (*Context).lpCompletionRoutine.is_some() {
        ((*Context).lpCompletionRoutine.unwrap())(
            (*IoStatusBlock).__bindgen_anon_1.Status as u32,
            (*IoStatusBlock).Information as u32,
            IoStatusBlock as LPWSAOVERLAPPED,
            0,
        );
    }

    HeapFree(GlobalHeap, 0, ApcContext);
}

unsafe extern "system" fn MsafdSendAPC(
    ApcContext: PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Reserved: ULONG,
) {
    let Context = ApcContext as PMSAFD_SEND_APC_CONTEXT;

    TRACE!(b"MsafdSendAPC(%p %lx %lx)\n\0".as_ptr() as PCHAR, ApcContext, (*IoStatusBlock).__bindgen_anon_1.Status, (*IoStatusBlock).Information);

    SockReenableAsyncSelectEvent((*Context).lpSocket as PVOID, FD_WRITE as LONG);

    if (*Context).lpCompletionRoutine.is_some() {
        ((*Context).lpCompletionRoutine.unwrap())((*IoStatusBlock).__bindgen_anon_1.Status as u32, (*IoStatusBlock).Information as u32, IoStatusBlock as LPWSAOVERLAPPED, 0);
    }

    HeapFree(GlobalHeap, 0, ApcContext);
}
#[no_mangle]
pub unsafe extern "system" fn WSPRecv(
    Handle: SOCKET,
    lpBuffers: LPWSABUF,
    dwBufferCount: DWORD,
    lpNumberOfBytesRead: LPDWORD,
    ReceiveFlags: LPDWORD,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
    lpThreadId: LPWSATHREADID,
    lpErrno: LPINT,
) -> i32 {
    let IOSB: PIO_STATUS_BLOCK;
    let mut DummyIOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut RecvInfo: AFD_RECV_INFO = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let APCContext: PMSAFD_RECV_APC_CONTEXT;
    let mut APCFunction: PIO_APC_ROUTINE = None;
    let mut Event: HANDLE = core::ptr::null_mut();
    let mut SockEvent: HANDLE = core::ptr::null_mut();
    let Socket: PSOCKET_INFORMATION;
    let NumberOfBytesRead: DWORD;

    TRACE!(b"WSPRecv(%x)\n\0".as_ptr() as _, Handle);

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAENOTSOCK as i32;
        }
        return SOCKET_ERROR;
    }
    if lpNumberOfBytesRead.is_null() && lpOverlapped.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAEFAULT as i32;
        }
        return SOCKET_ERROR;
    }
    if (*Socket).SharedData.is_null() == false
        && (*(*Socket).SharedData).__bindgen_anon_1.OobInline() != 0
        && !ReceiveFlags.is_null()
        && (*ReceiveFlags & MSG_OOB) != 0
    {
        if !lpErrno.is_null() {
            *lpErrno = WSAEINVAL as i32;
        }
        return SOCKET_ERROR;
    }

    memset(
        core::ptr::addr_of_mut!(RecvInfo) as *mut _,
        0,
        core::mem::size_of::<AFD_RECV_INFO>(),
    );

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

    RecvInfo.BufferArray = lpBuffers as PAFD_WSABUF;
    RecvInfo.BufferCount = dwBufferCount;
    RecvInfo.TdiFlags = 0;
    RecvInfo.AfdFlags = if (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
        AFD_IMMEDIATE
    } else {
        0
    };

    if ReceiveFlags.is_null() || *ReceiveFlags == 0 {
        RecvInfo.TdiFlags |= TDI_RECEIVE_NORMAL;
    } else {
        if (*ReceiveFlags & MSG_OOB) != 0 {
            RecvInfo.TdiFlags |= TDI_RECEIVE_EXPEDITED;
        }

        if (*ReceiveFlags & MSG_PEEK) != 0 {
            RecvInfo.TdiFlags |= TDI_RECEIVE_PEEK;
        }

        if (*ReceiveFlags & MSG_PARTIAL) != 0 {
            RecvInfo.TdiFlags |= TDI_RECEIVE_PARTIAL;
        }
    }

    if lpOverlapped.is_null() {
        APCContext = core::ptr::null_mut();
        APCFunction = None;
        Event = SockEvent;
        IOSB = core::ptr::addr_of_mut!(DummyIOSB);
    } else {
        if ((*(*Socket).SharedData).CreateFlags & SO_SYNCHRONOUS_NONALERT) != 0 {
            TRACE!(b"Opened without flag WSA_FLAG_OVERLAPPED. Do nothing.\n\0".as_ptr() as _);
            NtClose(SockEvent);
            return MsafdReturnWithErrno(STATUS_SUCCESS, lpErrno, 0, lpNumberOfBytesRead) as i32;
        }

        IOSB = lpOverlapped as PIO_STATUS_BLOCK;
        if lpCompletionRoutine.is_some() {
            RecvInfo.AfdFlags |= AFD_SKIP_FIO;
            APCFunction = as_apc(MsafdRecvAPC);
            APCContext = HeapAlloc(
                GlobalHeap,
                0,
                core::mem::size_of::<MSAFD_RECV_APC_CONTEXT>(),
            ) as PMSAFD_RECV_APC_CONTEXT;
            if APCContext.is_null() {
                ERR!("Not enough memory for APC Context\n");
                NtClose(SockEvent);
                return MsafdReturnWithErrno(
                    STATUS_INSUFFICIENT_RESOURCES,
                    lpErrno,
                    0,
                    lpNumberOfBytesRead,
                ) as i32;
            }
            (*APCContext).lpCompletionRoutine = lpCompletionRoutine;
            (*APCContext).lpOverlapped = lpOverlapped;
            (*APCContext).lpSocket = Socket;
            Event = core::ptr::null_mut();
        } else {
            Event = (*lpOverlapped).hEvent;

            APCFunction = None;
            if !Event.is_null() {
                APCContext = core::ptr::null_mut();
            } else {
                APCContext = lpOverlapped as PMSAFD_RECV_APC_CONTEXT;
            }
        }

        RecvInfo.AfdFlags |= AFD_OVERLAPPED;
    }

    (*IOSB).Information = 0;
    (*IOSB).__bindgen_anon_1.Status = STATUS_PENDING;

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        Event,
        APCFunction,
        APCContext as *mut _,
        IOSB,
        IOCTL_AFD_RECV,
        core::ptr::addr_of_mut!(RecvInfo) as *mut _,
        core::mem::size_of::<AFD_RECV_INFO>() as u32,
        core::ptr::null_mut(),
        0,
    );

    if lpOverlapped.is_null() && Status == STATUS_PENDING {
        Status = MsafdWaitForIo(Handle as HANDLE, SockEvent, IOSB, (*(*Socket).SharedData).RecvTimeout);
    }

    NtClose(SockEvent);

    if Status == STATUS_PENDING {
        TRACE!(b"Leaving (Pending)\n\0".as_ptr() as _);
        return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32;
    }

    NumberOfBytesRead = (*IOSB).Information as DWORD;

    if !ReceiveFlags.is_null() {
        match Status {
            STATUS_RECEIVE_EXPEDITED => {
                *ReceiveFlags = MSG_OOB;
            }
            STATUS_RECEIVE_PARTIAL_EXPEDITED => {
                *ReceiveFlags = MSG_PARTIAL | MSG_OOB;
            }
            STATUS_RECEIVE_PARTIAL => {
                *ReceiveFlags = MSG_PARTIAL;
            }
            _ => {
                *ReceiveFlags = 0;
            }
        }
    }

    if !ReceiveFlags.is_null() && (*ReceiveFlags & MSG_OOB) != 0 {
        SockReenableAsyncSelectEvent(Socket as PVOID, FD_OOB as LONG);
    } else {
        SockReenableAsyncSelectEvent(Socket as PVOID, FD_READ as LONG);
    }

    TRACE!(b"Leaving (%lx %ld)\n\0".as_ptr() as _, Status, NumberOfBytesRead);
    return MsafdReturnWithErrno(Status, lpErrno, NumberOfBytesRead, lpNumberOfBytesRead) as i32;
}
#[no_mangle]
pub unsafe extern "system" fn WSPRecvFrom(
    Handle: SOCKET,
    lpBuffers: LPWSABUF,
    dwBufferCount: DWORD,
    lpNumberOfBytesRead: LPDWORD,
    ReceiveFlags: LPDWORD,
    SocketAddress: *mut sockaddr,
    SocketAddressLength: *mut i32,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
    lpThreadId: LPWSATHREADID,
    lpErrno: LPINT,
) -> i32 {
    let mut IOSB: PIO_STATUS_BLOCK;
    let mut DummyIOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut RecvInfo: AFD_RECV_INFO_UDP = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut APCContext: PMSAFD_RECV_APC_CONTEXT;
    let mut APCFunction: PIO_APC_ROUTINE = None;
    let mut Event: HANDLE = core::ptr::null_mut();
    let mut SockEvent: HANDLE = core::ptr::null_mut();
    let Socket: PSOCKET_INFORMATION;
    let mut NumberOfBytesRead: DWORD;

    TRACE!(b"WSPRecvFrom(%x)\n\0".as_ptr() as *const i8, Handle);

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAENOTSOCK as i32;
        }
        return SOCKET_ERROR;
    }
    if lpNumberOfBytesRead.is_null() && lpOverlapped.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAEFAULT as i32;
        }
        return SOCKET_ERROR;
    }
    if (*Socket).SharedData != core::ptr::null_mut() && (*(*Socket).SharedData).__bindgen_anon_1.OobInline() != 0 && !ReceiveFlags.is_null() && (*ReceiveFlags & MSG_OOB) != 0 {
        if !lpErrno.is_null() {
            *lpErrno = WSAEINVAL as i32;
        }
        return SOCKET_ERROR;
    }

    if ((*(*Socket).SharedData).ServiceFlags1 & XP1_CONNECTIONLESS) == 0 {
        return WSPRecv(
            Handle,
            lpBuffers,
            dwBufferCount,
            lpNumberOfBytesRead,
            ReceiveFlags,
            lpOverlapped,
            lpCompletionRoutine,
            lpThreadId,
            lpErrno,
        );
    }

    if (*(*Socket).SharedData).State == SocketOpen {
        ((*(*Socket).HelperData).WSHGetWildcardSockaddr.unwrap())(
            (*Socket).HelperContext,
            SocketAddress,
            SocketAddressLength,
        );
        if WSPBind(Handle, SocketAddress, *SocketAddressLength, lpErrno) == SOCKET_ERROR {
            return SOCKET_ERROR;
        }
    }
    memset(
        &mut RecvInfo as *mut _ as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<AFD_RECV_INFO_UDP>(),
    );

    Status = NtCreateEvent(
        &mut SockEvent,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        _EVENT_TYPE_SynchronizationEvent,
        0,
    );

    if !NT_SUCCESS(Status) {
        return SOCKET_ERROR;
    }

    RecvInfo.BufferArray = lpBuffers as PAFD_WSABUF;
    RecvInfo.BufferCount = dwBufferCount;
    RecvInfo.TdiFlags = 0;
    RecvInfo.AfdFlags = if (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
        AFD_IMMEDIATE
    } else {
        0
    };
    RecvInfo.AddressLength = SocketAddressLength;
    RecvInfo.Address = SocketAddress as PVOID;

    if ReceiveFlags.is_null() || *ReceiveFlags == 0 {
        RecvInfo.TdiFlags |= TDI_RECEIVE_NORMAL;
    } else {
        if (*ReceiveFlags & MSG_OOB) != 0 {
            RecvInfo.TdiFlags |= TDI_RECEIVE_EXPEDITED;
        }

        if (*ReceiveFlags & MSG_PEEK) != 0 {
            RecvInfo.TdiFlags |= TDI_RECEIVE_PEEK;
        }

        if (*ReceiveFlags & MSG_PARTIAL) != 0 {
            RecvInfo.TdiFlags |= TDI_RECEIVE_PARTIAL;
        }
    }

    if lpOverlapped.is_null() {
        APCContext = core::ptr::null_mut();
        APCFunction = None;
        Event = SockEvent;
        IOSB = &mut DummyIOSB;
    } else {
        if ((*(*Socket).SharedData).CreateFlags & SO_SYNCHRONOUS_NONALERT) != 0 {
            TRACE!(b"Opened without flag WSA_FLAG_OVERLAPPED. Do nothing.\n\0".as_ptr() as *const i8);
            NtClose(SockEvent);
            return MsafdReturnWithErrno(STATUS_SUCCESS, lpErrno, 0, lpNumberOfBytesRead) as i32;
        }

        IOSB = lpOverlapped as PIO_STATUS_BLOCK;
        if lpCompletionRoutine.is_some() {
            RecvInfo.AfdFlags |= AFD_SKIP_FIO;
            APCFunction = as_apc(MsafdRecvAPC);
            APCContext = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<MSAFD_RECV_APC_CONTEXT>()) as PMSAFD_RECV_APC_CONTEXT;
            if APCContext.is_null() {
                ERR!("Not enough memory for APC Context\n");
                NtClose(SockEvent);
                return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, lpNumberOfBytesRead) as i32;
            }
            (*APCContext).lpCompletionRoutine = lpCompletionRoutine;
            (*APCContext).lpOverlapped = lpOverlapped;
            (*APCContext).lpSocket = Socket;
            Event = core::ptr::null_mut();
        } else {
            Event = (*lpOverlapped).hEvent;
            
            APCFunction = None;
            if !Event.is_null() {
                APCContext = core::ptr::null_mut();
            } else {
                APCContext = lpOverlapped as PMSAFD_RECV_APC_CONTEXT;
            }
        }

        RecvInfo.AfdFlags |= AFD_OVERLAPPED;
    }

    (*IOSB).Information = 0;
    (*IOSB).__bindgen_anon_1.Status = STATUS_PENDING;

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        Event,
        APCFunction,
        APCContext as *mut core::ffi::c_void,
        IOSB,
        IOCTL_AFD_RECV_DATAGRAM,
        &mut RecvInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<AFD_RECV_INFO_UDP>() as u32,
        core::ptr::null_mut(),
        0,
    );

    if lpOverlapped.is_null() && Status == STATUS_PENDING {
        Status = MsafdWaitForIo(Handle as HANDLE, SockEvent, IOSB, (*(*Socket).SharedData).RecvTimeout);
    }

    if !ReceiveFlags.is_null() {
        match Status {
            STATUS_RECEIVE_EXPEDITED => {
                *ReceiveFlags = MSG_OOB;
            }
            STATUS_RECEIVE_PARTIAL_EXPEDITED => {
                *ReceiveFlags = MSG_PARTIAL | MSG_OOB;
            }
            STATUS_RECEIVE_PARTIAL => {
                *ReceiveFlags = MSG_PARTIAL;
            }
            _ => {
                *ReceiveFlags = 0;
            }
        }
    }

    NtClose(SockEvent);

    if Status == STATUS_PENDING {
        TRACE!(b"Leaving (Pending)\n\0".as_ptr() as *const i8);
        *lpErrno = WSA_IO_PENDING as i32;
        return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32;
    }

    NumberOfBytesRead = (*IOSB).Information as DWORD;

    if !ReceiveFlags.is_null() && (*ReceiveFlags & MSG_OOB) != 0 {
        SockReenableAsyncSelectEvent(Socket as PVOID, FD_OOB as LONG);
    } else {
        SockReenableAsyncSelectEvent(Socket as PVOID, FD_READ as LONG);
    }

    TRACE!(b"Leaving (%lx %ld)\n\0".as_ptr() as *const i8, Status, NumberOfBytesRead);
    return MsafdReturnWithErrno(Status, lpErrno, NumberOfBytesRead, lpNumberOfBytesRead) as i32;
}
#[no_mangle]
pub unsafe extern "system" fn WSPSend(
    Handle: SOCKET,
    lpBuffers: LPWSABUF,
    dwBufferCount: DWORD,
    lpNumberOfBytesSent: LPDWORD,
    iFlags: DWORD,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
    lpThreadId: LPWSATHREADID,
    lpErrno: LPINT,
) -> i32 {
    let mut IOSB: PIO_STATUS_BLOCK;
    let mut DummyIOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut SendInfo: AFD_SEND_INFO = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut APCContext: PMSAFD_SEND_APC_CONTEXT;
    let mut APCFunction: PIO_APC_ROUTINE = None;
    let mut Event: HANDLE = core::ptr::null_mut();
    let mut SockEvent: HANDLE = core::ptr::null_mut();
    let Socket: PSOCKET_INFORMATION;
    let mut NumberOfBytesSent: DWORD;

    TRACE!(b"WSPSend(%x)\n\0".as_ptr() as _, Handle);

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAENOTSOCK as i32;
        }
        return SOCKET_ERROR;
    }
    if lpNumberOfBytesSent.is_null() && lpOverlapped.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAEFAULT as i32;
        }
        return SOCKET_ERROR;
    }

    memset(
        core::ptr::addr_of_mut!(SendInfo) as *mut _,
        0,
        core::mem::size_of::<AFD_SEND_INFO>(),
    );

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

    SendInfo.BufferArray = lpBuffers as PAFD_WSABUF;
    SendInfo.BufferCount = dwBufferCount;
    SendInfo.TdiFlags = 0;
    SendInfo.AfdFlags = if (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 {
        AFD_IMMEDIATE
    } else {
        0
    };

    if iFlags != 0 {
        if (iFlags & MSG_OOB) != 0 {
            SendInfo.TdiFlags |= TDI_SEND_EXPEDITED;
        }
        if (iFlags & MSG_PARTIAL) != 0 {
            SendInfo.TdiFlags |= TDI_SEND_PARTIAL;
        }
    }

    if lpOverlapped.is_null() {
        APCContext = core::ptr::null_mut();
        APCFunction = None;
        Event = SockEvent;
        IOSB = core::ptr::addr_of_mut!(DummyIOSB);
    } else {
        if ((*(*Socket).SharedData).CreateFlags & SO_SYNCHRONOUS_NONALERT) != 0 {
            TRACE!(b"Opened without flag WSA_FLAG_OVERLAPPED. Do nothing.\n\0".as_ptr() as _);
            NtClose(SockEvent);
            return MsafdReturnWithErrno(STATUS_SUCCESS, lpErrno, 0, lpNumberOfBytesSent) as i32;
        }

        IOSB = lpOverlapped as PIO_STATUS_BLOCK;
        if lpCompletionRoutine.is_some() {
            SendInfo.AfdFlags |= AFD_SKIP_FIO;
            APCFunction = as_apc(MsafdSendAPC);
            APCContext = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<MSAFD_SEND_APC_CONTEXT>())
                as PMSAFD_SEND_APC_CONTEXT;
            if APCContext.is_null() {
                ERR!("Not enough memory for APC Context\n");
                NtClose(SockEvent);
                return MsafdReturnWithErrno(
                    STATUS_INSUFFICIENT_RESOURCES,
                    lpErrno,
                    0,
                    lpNumberOfBytesSent,
                ) as i32;
            }
            (*APCContext).lpCompletionRoutine = lpCompletionRoutine;
            (*APCContext).lpOverlapped = lpOverlapped;
            (*APCContext).lpSocket = Socket;
            (*APCContext).lpRemoteAddress = core::ptr::null_mut();
            Event = core::ptr::null_mut();
        } else {
            Event = (*lpOverlapped).hEvent;

            APCFunction = None;
            if !Event.is_null() {
                APCContext = core::ptr::null_mut();
            } else {
                APCContext = lpOverlapped as PMSAFD_SEND_APC_CONTEXT;
            }
        }

        Event = (*lpOverlapped).hEvent;
        if !Event.is_null() {
            if lpCompletionRoutine.is_some() {
                HeapFree(GetProcessHeap(), 0, APCContext as *mut _);
            }
            APCContext = core::ptr::null_mut();
        }

        SendInfo.AfdFlags |= AFD_OVERLAPPED;
    }

    (*IOSB).Information = 0;
    (*IOSB).__bindgen_anon_1.Status = STATUS_PENDING;

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        Event,
        APCFunction,
        APCContext as *mut _,
        IOSB,
        IOCTL_AFD_SEND,
        core::ptr::addr_of_mut!(SendInfo) as *mut _,
        core::mem::size_of::<AFD_SEND_INFO>() as u32,
        core::ptr::null_mut(),
        0,
    );

    if lpOverlapped.is_null() && Status == STATUS_PENDING {
        Status = MsafdWaitForIo(Handle as HANDLE, SockEvent, IOSB, (*(*Socket).SharedData).SendTimeout);
    }

    NtClose(SockEvent);

    if Status == STATUS_PENDING {
        TRACE!(b"Leaving (Pending)\n\0".as_ptr() as _);
        return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32;
    }

    NumberOfBytesSent = (*IOSB).Information as DWORD;

    SockReenableAsyncSelectEvent(Socket as PVOID, FD_WRITE as LONG);

    TRACE!(b"Leaving (%lx %ld)\n\0".as_ptr() as _, Status, NumberOfBytesSent);
    let __rv = MsafdReturnWithErrno(Status, lpErrno, NumberOfBytesSent, lpNumberOfBytesSent) as i32;
    return __rv;
}
#[no_mangle]
pub unsafe extern "system" fn WSPSendTo(
    Handle: SOCKET,
    lpBuffers: LPWSABUF,
    dwBufferCount: DWORD,
    lpNumberOfBytesSent: LPDWORD,
    iFlags: DWORD,
    SocketAddress: *const sockaddr,
    SocketAddressLength: i32,
    lpOverlapped: LPWSAOVERLAPPED,
    lpCompletionRoutine: LPWSAOVERLAPPED_COMPLETION_ROUTINE,
    lpThreadId: LPWSATHREADID,
    lpErrno: LPINT,
) -> i32 {
    let mut IOSB: PIO_STATUS_BLOCK;
    let mut DummyIOSB: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut SendInfo: AFD_SEND_INFO_UDP = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut APCContext: PMSAFD_SEND_APC_CONTEXT;
    let mut APCFunction: PIO_APC_ROUTINE = None;
    let mut Event: HANDLE = core::ptr::null_mut();
    let RemoteAddress: PTRANSPORT_ADDRESS;
    let mut SockEvent: HANDLE = core::ptr::null_mut();
    let Socket: PSOCKET_INFORMATION;
    let NumberOfBytesSent: DWORD;
    let mut Buffer: [UCHAR; 128] = core::mem::zeroed();

    TRACE!(b"WSPSendTo(%x)\n\0".as_ptr() as PCHAR, Handle);

    Socket = GetSocketStructure(Handle);
    if Socket.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAENOTSOCK as i32;
        }
        return SOCKET_ERROR;
    }
    if lpNumberOfBytesSent.is_null() && lpOverlapped.is_null() {
        if !lpErrno.is_null() {
            *lpErrno = WSAEFAULT as i32;
        }
        return SOCKET_ERROR;
    }

    if ((*(*Socket).SharedData).ServiceFlags1 & XP1_CONNECTIONLESS) == 0 {
        return WSPSend(
            Handle,
            lpBuffers,
            dwBufferCount,
            lpNumberOfBytesSent,
            iFlags,
            lpOverlapped,
            lpCompletionRoutine,
            lpThreadId,
            lpErrno,
        );
    }

    if (*(*Socket).SharedData).State == SocketOpen {
        let mut BindAddressLength: INT;
        let BindAddress: PSOCKADDR;
        let BindError: INT;

        BindAddressLength = (*(*Socket).HelperData).MaxWSAddressLength;
        BindAddress = HeapAlloc(GlobalHeap, 0, BindAddressLength as usize) as PSOCKADDR;
        if BindAddress.is_null() {
            MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, core::ptr::null_mut());
            return INVALID_SOCKET as i32;
        }

        ((*(*Socket).HelperData).WSHGetWildcardSockaddr.unwrap())(
            (*Socket).HelperContext,
            BindAddress,
            core::ptr::addr_of_mut!(BindAddressLength),
        );
        BindError = WSPBind(Handle, BindAddress, BindAddressLength, lpErrno);
        HeapFree(GlobalHeap, 0, BindAddress as LPVOID);
        if BindError == SOCKET_ERROR {
            return SOCKET_ERROR;
        }
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

    RemoteAddress = Buffer.as_mut_ptr() as PTRANSPORT_ADDRESS;

    if SocketAddressLength > (128 - core::mem::size_of::<TRANSPORT_ADDRESS>() as i32) {
        *lpErrno = WSAEFAULT as i32;
        return SOCKET_ERROR;
    }

    (*RemoteAddress).TAAddressCount = 1;
    (*RemoteAddress).Address[0].AddressLength = (SocketAddressLength - core::mem::size_of::<u16>() as i32) as USHORT;
    RtlCopyMemory(
        core::ptr::addr_of_mut!((*RemoteAddress).Address[0].AddressType) as PVOID,
        SocketAddress as *const core::ffi::c_void,
        SocketAddressLength as usize,
    );

    SendInfo.BufferArray = lpBuffers as PAFD_WSABUF;
    SendInfo.AfdFlags = if (*(*Socket).SharedData).__bindgen_anon_1.NonBlocking() != 0 { AFD_IMMEDIATE } else { 0 };
    SendInfo.BufferCount = dwBufferCount;
    SendInfo.TdiConnection.RemoteAddress = RemoteAddress as PVOID;
    SendInfo.TdiConnection.RemoteAddressLength = (*(*Socket).HelperData).MaxTDIAddressLength;

    if lpOverlapped.is_null() {
        APCContext = core::ptr::null_mut();
        APCFunction = None;
        Event = SockEvent;
        IOSB = core::ptr::addr_of_mut!(DummyIOSB);
    } else {
        if ((*(*Socket).SharedData).CreateFlags & SO_SYNCHRONOUS_NONALERT) != 0 {
            TRACE!(b"Opened without flag WSA_FLAG_OVERLAPPED. Do nothing.\n\0".as_ptr() as PCHAR);
            NtClose(SockEvent);
            return MsafdReturnWithErrno(STATUS_SUCCESS, lpErrno, 0, lpNumberOfBytesSent) as i32;
        }

        IOSB = lpOverlapped as PIO_STATUS_BLOCK;
        if lpCompletionRoutine.is_some() {
            SendInfo.AfdFlags |= AFD_SKIP_FIO;
            APCFunction = as_apc(MsafdSendAPC);
            APCContext = HeapAlloc(GlobalHeap, 0, core::mem::size_of::<MSAFD_SEND_APC_CONTEXT>()) as PMSAFD_SEND_APC_CONTEXT;
            if APCContext.is_null() {
                ERR!("Not enough memory for APC Context\n");
                NtClose(SockEvent);
                return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, lpNumberOfBytesSent) as i32;
            }
            (*APCContext).lpCompletionRoutine = lpCompletionRoutine;
            (*APCContext).lpOverlapped = lpOverlapped;
            (*APCContext).lpSocket = Socket;
            (*APCContext).lpRemoteAddress = RemoteAddress;
            Event = core::ptr::null_mut();
        } else {
            Event = (*lpOverlapped).hEvent;

            APCFunction = None;
            if !Event.is_null() {
                APCContext = core::ptr::null_mut();
            } else {
                APCContext = lpOverlapped as PMSAFD_SEND_APC_CONTEXT;
            }
        }

        SendInfo.AfdFlags |= AFD_OVERLAPPED;
    }

    (*IOSB).Information = 0;
    (*IOSB).__bindgen_anon_1.Status = STATUS_PENDING;

    Status = NtDeviceIoControlFile(
        Handle as HANDLE,
        Event,
        APCFunction,
        APCContext as PVOID,
        IOSB,
        IOCTL_AFD_SEND_DATAGRAM,
        core::ptr::addr_of_mut!(SendInfo) as PVOID,
        core::mem::size_of::<AFD_SEND_INFO_UDP>() as ULONG,
        core::ptr::null_mut(),
        0,
    );

    if lpOverlapped.is_null() && Status == STATUS_PENDING {
        Status = MsafdWaitForIo(Handle as HANDLE, SockEvent, IOSB, (*(*Socket).SharedData).SendTimeout);
    }

    NtClose(SockEvent);

    if Status == STATUS_PENDING {
        TRACE!(b"Leaving (Pending)\n\0".as_ptr() as PCHAR);
        return MsafdReturnWithErrno(Status, lpErrno, 0, core::ptr::null_mut()) as i32;
    }

    NumberOfBytesSent = (*IOSB).Information as DWORD;

    SockReenableAsyncSelectEvent(Socket as PVOID, FD_WRITE as LONG);

    TRACE!(b"Leaving (%lx %ld)\n\0".as_ptr() as PCHAR, Status, NumberOfBytesSent);
    return MsafdReturnWithErrno(Status, lpErrno, NumberOfBytesSent, lpNumberOfBytesSent) as i32;
}
#[no_mangle]
pub unsafe extern "system" fn WSPRecvDisconnect(
    s: SOCKET,
    lpInboundDisconnectData: LPWSABUF,
    lpErrno: LPINT,
) -> INT {
    0
}
#[no_mangle]
pub unsafe extern "system" fn WSPSendDisconnect(
    s: SOCKET,
    lpOutboundDisconnectData: LPWSABUF,
    lpErrno: LPINT,
) -> INT {
    // C: `UNIMPLEMENTED; return 0;`. UNIMPLEMENTED is a debug print; `unimplemented!()`
    // here was a panic that took down any process whose socket was shut down this way.
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Ancillary Function Driver DLL
//  * FILE:        dll/win32/msafd/misc/sndrcv.c
//  * PURPOSE:     Send/receive routines
//  * PROGRAMMERS: Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *              Alex Ionescu (alex@relsoft.net)
//  * REVISIONS:
//  *              CSH 01/09-2000 Created
//  *              Alex 16/07/2004 - Complete Rewrite
//  */
//
// #include <msafd.h>
//
// INT
// WSPAPI
// WSPAsyncSelect(IN  SOCKET Handle,
//                IN  HWND hWnd,
//                IN  UINT wMsg,
//                IN  LONG lEvent,
//                OUT LPINT lpErrno)
// {
//     PSOCKET_INFORMATION Socket = NULL;
//     PASYNC_DATA                 AsyncData;
//     BOOLEAN                     BlockMode;
//
//     /* Get the Socket Structure associated to this Socket */
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//        *lpErrno = WSAENOTSOCK;
//        return SOCKET_ERROR;
//     }
//
//     /* Allocate the Async Data Structure to pass on to the Thread later */
//     AsyncData = HeapAlloc(GetProcessHeap(), 0, sizeof(*AsyncData));
//     if (!AsyncData)
//     {
//         MsafdReturnWithErrno( STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, NULL );
//         return INVALID_SOCKET;
//     }
//
//     /* Change the Socket to Non Blocking */
//     BlockMode = TRUE;
//     SetSocketInformation(Socket, AFD_INFO_BLOCKING_MODE, &BlockMode, NULL, NULL, NULL, NULL);
//     Socket->SharedData->NonBlocking = TRUE;
//
//     /* Deactivate WSPEventSelect */
//     if (Socket->SharedData->AsyncEvents)
//     {
//         if (WSPEventSelect(Handle, NULL, 0, lpErrno) == SOCKET_ERROR)
//         {
//             HeapFree(GetProcessHeap(), 0, AsyncData);
//             return SOCKET_ERROR;
//         }
//     }
//
//     /* Create the Asynch Thread if Needed */
//     SockCreateOrReferenceAsyncThread();
//
//     /* Open a Handle to AFD's Async Helper */
//     SockGetAsyncSelectHelperAfdHandle();
//
//     /* Store Socket Data */
//     Socket->SharedData->hWnd = hWnd;
//     Socket->SharedData->wMsg = wMsg;
//     Socket->SharedData->AsyncEvents = lEvent;
//     Socket->SharedData->AsyncDisabledEvents = 0;
//     Socket->SharedData->SequenceNumber++;
//
//     /* Return if there are no more Events */
//     if ((Socket->SharedData->AsyncEvents & (~Socket->SharedData->AsyncDisabledEvents)) == 0)
//     {
//         HeapFree(GetProcessHeap(), 0, AsyncData);
//         return 0;
//     }
//
//     /* Set up the Async Data */
//     AsyncData->ParentSocket = Socket;
//     AsyncData->SequenceNumber = Socket->SharedData->SequenceNumber;
//
//     /* Begin Async Select by using I/O Completion */
//     NtSetIoCompletion(SockAsyncCompletionPort,
//                       (PVOID)&SockProcessQueuedAsyncSelect,
//                       AsyncData,
//                       0,
//                       0);
//
//     /* Return */
//     return ERROR_SUCCESS;
// }
//
//
// BOOL
// WSPAPI
// WSPGetOverlappedResult(
//     IN  SOCKET Handle,
//     IN  LPWSAOVERLAPPED lpOverlapped,
//     OUT LPDWORD lpdwBytes,
//     IN  BOOL fWait,
//     OUT LPDWORD lpdwFlags,
//     OUT LPINT lpErrno)
// {
//     PSOCKET_INFORMATION     Socket;
//     PIO_STATUS_BLOCK        IoStatusBlock;
//     BOOL                    Ret;
//
//     TRACE("Called (%x)\n", Handle);
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//         if(lpErrno)
//             *lpErrno = WSAENOTSOCK;
//         return FALSE;
//     }
//     if (!lpOverlapped || !lpdwBytes || !lpdwFlags)
//     {
//         if (lpErrno)
//             *lpErrno = WSAEFAULT;
//         return FALSE;
//     }
//
//     IoStatusBlock = (PIO_STATUS_BLOCK)lpOverlapped;
//     Ret = GetOverlappedResult((HANDLE)Handle, lpOverlapped, lpdwBytes, fWait);
//
//     /* HACK: Allow APC to be processed */
//     SleepEx(0, TRUE);
//
//     if (!fWait && IoStatusBlock->Status == STATUS_PENDING)
//     {
//         if (lpErrno)
//             *lpErrno = WSA_IO_INCOMPLETE;
//         return FALSE;
//     }
//
//     if (Ret)
//     {
//
//         /* Re-enable Async Event */
//         SockReenableAsyncSelectEvent(Socket, FD_OOB);
//         SockReenableAsyncSelectEvent(Socket, FD_WRITE);
//         SockReenableAsyncSelectEvent(Socket, FD_READ);
//     }
//
//     switch (IoStatusBlock->Status)
//     {
//         case STATUS_RECEIVE_EXPEDITED:
//             *lpdwFlags = MSG_OOB;
//             break;
//         case STATUS_RECEIVE_PARTIAL_EXPEDITED:
//             *lpdwFlags = MSG_PARTIAL | MSG_OOB;
//             break;
//         case STATUS_RECEIVE_PARTIAL:
//             *lpdwFlags = MSG_PARTIAL;
//             break;
//         default:
//             *lpdwFlags = 0;
//             if (lpErrno)
//                 *lpErrno = TranslateNtStatusError(IoStatusBlock->Status);
//             break;
//     }
//
//     return Ret;
// }
//
// static
// VOID
// NTAPI
// MsafdRecvAPC(
//      PVOID ApcContext,
//      PIO_STATUS_BLOCK IoStatusBlock,
//      ULONG Reserved)
// {
//     PMSAFD_RECV_APC_CONTEXT Context = ApcContext;
//
//     TRACE("MsafdRecvAPC(%p %lx %lx)\n", ApcContext, IoStatusBlock->Status, IoStatusBlock->Information);
//
//     /* Re-enable Async Event */
//     if (IoStatusBlock->Status == STATUS_RECEIVE_EXPEDITED ||
//         IoStatusBlock->Status == STATUS_RECEIVE_PARTIAL_EXPEDITED)
//     {
//         SockReenableAsyncSelectEvent(Context->lpSocket, FD_OOB);
//     }
//     else
//     {
//         SockReenableAsyncSelectEvent(Context->lpSocket, FD_READ);
//     }
//
//     if (Context->lpCompletionRoutine)
//         Context->lpCompletionRoutine(IoStatusBlock->Status, IoStatusBlock->Information, (LPWSAOVERLAPPED)IoStatusBlock, 0);
//
//     HeapFree(GlobalHeap, 0, ApcContext);
// }
//
// static
// VOID
// NTAPI
// MsafdSendAPC(
//      PVOID ApcContext,
//      PIO_STATUS_BLOCK IoStatusBlock,
//      ULONG Reserved)
// {
//     PMSAFD_SEND_APC_CONTEXT Context = ApcContext;
//
//     TRACE("MsafdSendAPC(%p %lx %lx)\n", ApcContext, IoStatusBlock->Status, IoStatusBlock->Information);
//
//     /* Re-enable Async Event */
//     SockReenableAsyncSelectEvent(Context->lpSocket, FD_WRITE);
//
//     if (Context->lpCompletionRoutine)
//         Context->lpCompletionRoutine(IoStatusBlock->Status, IoStatusBlock->Information, (LPWSAOVERLAPPED)IoStatusBlock, 0);
//
//     /* Free IOCTL buffers */
//     HeapFree(GlobalHeap, 0, ApcContext);
// }
//
// int
// WSPAPI
// WSPRecv(SOCKET Handle,
//         LPWSABUF lpBuffers,
//         DWORD dwBufferCount,
//         LPDWORD lpNumberOfBytesRead,
//         LPDWORD ReceiveFlags,
//         LPWSAOVERLAPPED lpOverlapped,
//         LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine,
//         LPWSATHREADID lpThreadId,
//         LPINT lpErrno)
// {
//     PIO_STATUS_BLOCK        IOSB;
//     IO_STATUS_BLOCK         DummyIOSB;
//     AFD_RECV_INFO           RecvInfo;
//     NTSTATUS                Status;
//     PMSAFD_RECV_APC_CONTEXT APCContext;
//     PIO_APC_ROUTINE         APCFunction = NULL;
//     HANDLE                  Event = NULL;
//     HANDLE                  SockEvent;
//     PSOCKET_INFORMATION     Socket;
//     DWORD                   NumberOfBytesRead;
//
//     TRACE("WSPRecv(%x)\n", Handle);
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//         if (lpErrno)
//             *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//     if (!lpNumberOfBytesRead && !lpOverlapped)
//     {
//         if (lpErrno)
//             *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//     if (Socket->SharedData->OobInline && ReceiveFlags && (*ReceiveFlags & MSG_OOB) != 0)
//     {
//         if (lpErrno)
//             *lpErrno = WSAEINVAL;
//         return SOCKET_ERROR;
//     }
//
//     memset(&RecvInfo, 0, sizeof(RecvInfo));
//
//     Status = NtCreateEvent( &SockEvent, EVENT_ALL_ACCESS,
//                             NULL, SynchronizationEvent, FALSE );
//
//     if (!NT_SUCCESS(Status))
//     {
//         return SOCKET_ERROR;
//     }
//
//     /* Set up the Receive Structure */
//     RecvInfo.BufferArray = (PAFD_WSABUF)lpBuffers;
//     RecvInfo.BufferCount = dwBufferCount;
//     RecvInfo.TdiFlags = 0;
//     RecvInfo.AfdFlags = Socket->SharedData->NonBlocking ? AFD_IMMEDIATE : 0;
//
//     /* Set the TDI Flags */
//     if (!ReceiveFlags || *ReceiveFlags == 0)
//     {
//         RecvInfo.TdiFlags |= TDI_RECEIVE_NORMAL;
//     }
//     else
//     {
//         if (*ReceiveFlags & MSG_OOB)
//         {
//             RecvInfo.TdiFlags |= TDI_RECEIVE_EXPEDITED;
//         }
//
//         if (*ReceiveFlags & MSG_PEEK)
//         {
//             RecvInfo.TdiFlags |= TDI_RECEIVE_PEEK;
//         }
//
//         if (*ReceiveFlags & MSG_PARTIAL)
//         {
//             RecvInfo.TdiFlags |= TDI_RECEIVE_PARTIAL;
//         }
//     }
//
//     /* Verify if we should use APC */
//     if (!lpOverlapped)
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
//             return MsafdReturnWithErrno(STATUS_SUCCESS, lpErrno, 0, lpNumberOfBytesRead);
//         }
//
//         IOSB = (PIO_STATUS_BLOCK)lpOverlapped;
//         if (lpCompletionRoutine)
//         {
//             RecvInfo.AfdFlags |= AFD_SKIP_FIO;
//             APCFunction = &MsafdRecvAPC;
//             APCContext = HeapAlloc(GlobalHeap, 0, sizeof(*APCContext));
//             if (!APCContext)
//             {
//                 ERR("Not enough memory for APC Context\n");
//                 NtClose(SockEvent);
//                 return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, lpNumberOfBytesRead);
//             }
//             APCContext->lpCompletionRoutine = lpCompletionRoutine;
//             APCContext->lpOverlapped = lpOverlapped;
//             APCContext->lpSocket = Socket;
//             Event = NULL;
//         }
//         else
//         {
//             // .NET expects APCContext to be a pointer to lpOverlapped, because it adds some additional metadata past OVERLAPPED structure
//             // Do not use APC when there is no completion context
//             Event = lpOverlapped->hEvent;
//             
//             APCFunction = NULL;
//             if (Event)
//             {
//                 APCContext = NULL;
//             }
//             else
//             {
//                 APCContext = (PMSAFD_RECV_APC_CONTEXT)lpOverlapped;
//             }
//         }
//
//         RecvInfo.AfdFlags |= AFD_OVERLAPPED;
//     }
//
//     IOSB->Information = 0;
//     IOSB->Status = STATUS_PENDING;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                    Event,
//                                    APCFunction,
//                                    APCContext,
//                                    IOSB,
//                                    IOCTL_AFD_RECV,
//                                    &RecvInfo,
//                                    sizeof(RecvInfo),
//                                    NULL,
//                                    0);
//
//     /* Wait for completion if not overlapped */
//     if (!lpOverlapped && Status == STATUS_PENDING)
//     {
//         /* It's up to the protocol to time out recv. We must wait
//          * until the protocol decides it's had enough. */
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB->Status;
//     }
//
//     NtClose(SockEvent);
//
//     if (Status == STATUS_PENDING)
//     {
//         TRACE("Leaving (Pending)\n");
//         return MsafdReturnWithErrno(Status, lpErrno, 0, NULL);
//     }
//
//     NumberOfBytesRead = (DWORD)IOSB->Information;
//
//     /* Return the Flags */
//     if (ReceiveFlags)
//     {
//         switch (Status)
//         {
//             case STATUS_RECEIVE_EXPEDITED:
//                 *ReceiveFlags = MSG_OOB;
//                 break;
//             case STATUS_RECEIVE_PARTIAL_EXPEDITED:
//                 *ReceiveFlags = MSG_PARTIAL | MSG_OOB;
//                 break;
//             case STATUS_RECEIVE_PARTIAL:
//                 *ReceiveFlags = MSG_PARTIAL;
//                 break;
//             default:
//                 *ReceiveFlags = 0;
//                 break;
//         }
//     }
//
//     /* Re-enable Async Event */
//     if (ReceiveFlags && (*ReceiveFlags & MSG_OOB))
//     {
//         SockReenableAsyncSelectEvent(Socket, FD_OOB);
//     }
//     else
//     {
//         SockReenableAsyncSelectEvent(Socket, FD_READ);
//     }
//
//     TRACE("Leaving (%lx %ld)\n", Status, NumberOfBytesRead);
//     return MsafdReturnWithErrno(Status, lpErrno, NumberOfBytesRead, lpNumberOfBytesRead);
// }
//
// int
// WSPAPI
// WSPRecvFrom(SOCKET Handle,
//             LPWSABUF lpBuffers,
//             DWORD dwBufferCount,
//             LPDWORD lpNumberOfBytesRead,
//             LPDWORD ReceiveFlags,
//             struct sockaddr *SocketAddress,
//             int *SocketAddressLength,
//             LPWSAOVERLAPPED lpOverlapped,
//             LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine,
//             LPWSATHREADID lpThreadId,
//             LPINT lpErrno )
// {
//     PIO_STATUS_BLOCK            IOSB;
//     IO_STATUS_BLOCK             DummyIOSB;
//     AFD_RECV_INFO_UDP           RecvInfo;
//     NTSTATUS                    Status;
//     PMSAFD_RECV_APC_CONTEXT     APCContext;
//     PIO_APC_ROUTINE             APCFunction = NULL;
//     HANDLE                      Event = NULL;
//     HANDLE                      SockEvent;
//     PSOCKET_INFORMATION         Socket;
//     DWORD                       NumberOfBytesRead;
//
//     TRACE("WSPRecvFrom(%x)\n", Handle);
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//         if (lpErrno)
//             *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//     if (!lpNumberOfBytesRead && !lpOverlapped)
//     {
//         if (lpErrno)
//             *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//     if (Socket->SharedData->OobInline && ReceiveFlags && (*ReceiveFlags & MSG_OOB) != 0)
//     {
//         if (lpErrno)
//             *lpErrno = WSAEINVAL;
//         return SOCKET_ERROR;
//     }
//
//     if (!(Socket->SharedData->ServiceFlags1 & XP1_CONNECTIONLESS))
//     {
//         /* Call WSPRecv for a non-datagram socket */
//         return WSPRecv(Handle,
//                        lpBuffers,
//                        dwBufferCount,
//                        lpNumberOfBytesRead,
//                        ReceiveFlags,
//                        lpOverlapped,
//                        lpCompletionRoutine,
//                        lpThreadId,
//                        lpErrno);
//     }
//
//     /* Bind us First */
//     if (Socket->SharedData->State == SocketOpen)
//     {
//         Socket->HelperData->WSHGetWildcardSockaddr(Socket->HelperContext,
//             SocketAddress,
//             SocketAddressLength);
//         /* Bind it */
//         if (WSPBind(Handle, SocketAddress, *SocketAddressLength, lpErrno) == SOCKET_ERROR)
//             return SOCKET_ERROR;
//     }
//     memset(&RecvInfo, 0, sizeof(RecvInfo));
//
//     Status = NtCreateEvent( &SockEvent, EVENT_ALL_ACCESS,
//                             NULL, SynchronizationEvent, FALSE );
//
//     if( !NT_SUCCESS(Status) )
//     {
//         return SOCKET_ERROR;
//     }
//
//     /* Set up the Receive Structure */
//     RecvInfo.BufferArray = (PAFD_WSABUF)lpBuffers;
//     RecvInfo.BufferCount = dwBufferCount;
//     RecvInfo.TdiFlags = 0;
//     RecvInfo.AfdFlags = Socket->SharedData->NonBlocking ? AFD_IMMEDIATE : 0;
//     RecvInfo.AddressLength = SocketAddressLength;
//     RecvInfo.Address = SocketAddress;
//
//     /* Set the TDI Flags */
//     if (!ReceiveFlags || *ReceiveFlags == 0)
//     {
//         RecvInfo.TdiFlags |= TDI_RECEIVE_NORMAL;
//     }
//     else
//     {
//         if (*ReceiveFlags & MSG_OOB)
//         {
//             RecvInfo.TdiFlags |= TDI_RECEIVE_EXPEDITED;
//         }
//
//         if (*ReceiveFlags & MSG_PEEK)
//         {
//             RecvInfo.TdiFlags |= TDI_RECEIVE_PEEK;
//         }
//
//         if (*ReceiveFlags & MSG_PARTIAL)
//         {
//             RecvInfo.TdiFlags |= TDI_RECEIVE_PARTIAL;
//         }
//     }
//
//     /* Verify if we should use APC */
//     if (!lpOverlapped)
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
//             return MsafdReturnWithErrno(STATUS_SUCCESS, lpErrno, 0, lpNumberOfBytesRead);
//         }
//
//         IOSB = (PIO_STATUS_BLOCK)lpOverlapped;
//         if (lpCompletionRoutine)
//         {
//             RecvInfo.AfdFlags |= AFD_SKIP_FIO;
//             APCFunction = &MsafdRecvAPC;
//             APCContext = HeapAlloc(GlobalHeap, 0, sizeof(*APCContext));
//             if (!APCContext)
//             {
//                 ERR("Not enough memory for APC Context\n");
//                 NtClose(SockEvent);
//                 return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, lpNumberOfBytesRead);
//             }
//             APCContext->lpCompletionRoutine = lpCompletionRoutine;
//             APCContext->lpOverlapped = lpOverlapped;
//             APCContext->lpSocket = Socket;
//             Event = NULL;
//         }
//         else
//         {
//             // .NET expects APCContext to be a pointer to lpOverlapped, because it adds some additional metadata past OVERLAPPED structure
//             // Do not use APC when there is no completion context
//             Event = lpOverlapped->hEvent;
//             
//             APCFunction = NULL;
//             if (Event)
//             {
//                 APCContext = NULL;
//             }
//             else
//             {
//                 APCContext = (PMSAFD_RECV_APC_CONTEXT)lpOverlapped;
//             }
//         }
//
//         RecvInfo.AfdFlags |= AFD_OVERLAPPED;
//     }
//
//     IOSB->Information = 0;
//     IOSB->Status = STATUS_PENDING;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                     Event,
//                                     APCFunction,
//                                     APCContext,
//                                     IOSB,
//                                     IOCTL_AFD_RECV_DATAGRAM,
//                                     &RecvInfo,
//                                     sizeof(RecvInfo),
//                                     NULL,
//                                     0);
//
//     /* Wait for completion if not overlapped */
//     if (!lpOverlapped && Status == STATUS_PENDING)
//     {
//         /* FIXME: Shouldn't wait infinitely for receive... */
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB->Status;
//     }
//
//     /* Return the Flags */
//     if (ReceiveFlags)
//     {
//         switch (Status)
//         {
//             case STATUS_RECEIVE_EXPEDITED:
//                 *ReceiveFlags = MSG_OOB;
//                 break;
//             case STATUS_RECEIVE_PARTIAL_EXPEDITED:
//                 *ReceiveFlags = MSG_PARTIAL | MSG_OOB;
//                 break;
//             case STATUS_RECEIVE_PARTIAL:
//                 *ReceiveFlags = MSG_PARTIAL;
//                 break;
//             default:
//                 *ReceiveFlags = 0;
//                 break;
//         }
//     }
//
//     NtClose(SockEvent);
//
//     if (Status == STATUS_PENDING)
//     {
//         TRACE("Leaving (Pending)\n");
//         *lpErrno = WSA_IO_PENDING;
//         return MsafdReturnWithErrno(Status, lpErrno, 0, NULL);
//     }
//
//     NumberOfBytesRead = (DWORD)IOSB->Information;
//
//     /* Re-enable Async Event */
//     if (ReceiveFlags && (*ReceiveFlags & MSG_OOB))
//     {
//         SockReenableAsyncSelectEvent(Socket, FD_OOB);
//     }
//     else
//     {
//         SockReenableAsyncSelectEvent(Socket, FD_READ);
//     }
//
//     TRACE("Leaving (%lx %ld)\n", Status, NumberOfBytesRead);
//     return MsafdReturnWithErrno(Status, lpErrno, NumberOfBytesRead, lpNumberOfBytesRead);
// }
//
//
// int
// WSPAPI
// WSPSend(SOCKET Handle,
//         LPWSABUF lpBuffers,
//         DWORD dwBufferCount,
//         LPDWORD lpNumberOfBytesSent,
//         DWORD iFlags,
//         LPWSAOVERLAPPED lpOverlapped,
//         LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine,
//         LPWSATHREADID lpThreadId,
//         LPINT lpErrno)
// {
//     PIO_STATUS_BLOCK        IOSB;
//     IO_STATUS_BLOCK         DummyIOSB;
//     AFD_SEND_INFO          SendInfo;
//     NTSTATUS                Status;
//     PMSAFD_SEND_APC_CONTEXT APCContext;
//     PIO_APC_ROUTINE         APCFunction = NULL;
//     HANDLE                  Event = NULL;
//     HANDLE                  SockEvent;
//     PSOCKET_INFORMATION     Socket;
//     DWORD                   NumberOfBytesSent;
//     
//     TRACE("WSPSend(%x)\n", Handle);
//
//     /* Get the Socket Structure associate to this Socket*/
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//         if (lpErrno)
//             *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//     if (!lpNumberOfBytesSent && !lpOverlapped)
//     {
//         if (lpErrno)
//             *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//     memset(&SendInfo, 0, sizeof(SendInfo));
//
//     Status = NtCreateEvent( &SockEvent, EVENT_ALL_ACCESS,
//                             NULL, SynchronizationEvent, FALSE );
//
//     if( !NT_SUCCESS(Status) )
//     {
//         return SOCKET_ERROR;
//     }
//
//     /* Set up the Send Structure */
//     SendInfo.BufferArray = (PAFD_WSABUF)lpBuffers;
//     SendInfo.BufferCount = dwBufferCount;
//     SendInfo.TdiFlags = 0;
//     SendInfo.AfdFlags = Socket->SharedData->NonBlocking ? AFD_IMMEDIATE : 0;
//
//     /* Set the TDI Flags */
//     if (iFlags)
//     {
//         if (iFlags & MSG_OOB)
//         {
//             SendInfo.TdiFlags |= TDI_SEND_EXPEDITED;
//         }
//         if (iFlags & MSG_PARTIAL)
//         {
//             SendInfo.TdiFlags |= TDI_SEND_PARTIAL;
//         }
//     }
//
//     /* Verify if we should use APC */
//     if (!lpOverlapped)
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
//             return MsafdReturnWithErrno(STATUS_SUCCESS, lpErrno, 0, lpNumberOfBytesSent);
//         }
//
//         IOSB = (PIO_STATUS_BLOCK)lpOverlapped;
//         if (lpCompletionRoutine)
//         {
//             SendInfo.AfdFlags |= AFD_SKIP_FIO;
//             APCFunction = &MsafdSendAPC;
//             APCContext = HeapAlloc(GlobalHeap, 0, sizeof(*APCContext));
//             if (!APCContext)
//             {
//                 ERR("Not enough memory for APC Context\n");
//                 NtClose(SockEvent);
//                 return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, lpNumberOfBytesSent);
//             }
//             APCContext->lpCompletionRoutine = lpCompletionRoutine;
//             APCContext->lpOverlapped = lpOverlapped;
//             APCContext->lpSocket = Socket;
//             APCContext->lpRemoteAddress = NULL;
//             Event = NULL;
//         }
//         else
//         {
//             // .NET expects APCContext to be a pointer to lpOverlapped, because it adds some additional metadata past OVERLAPPED structure
//             // Do not use APC when there is no completion context
//             Event = lpOverlapped->hEvent;
//             
//             APCFunction = NULL;
//             if (Event)
//             {
//                 APCContext = NULL;
//             }
//             else
//             {
//                 APCContext = (PMSAFD_SEND_APC_CONTEXT)lpOverlapped;
//             }
//         }
//         
//         Event = lpOverlapped->hEvent;
//         // APCContext is unsupported when using event
//         if (Event)
//         {
//             if (lpCompletionRoutine)
//                 HeapFree(GetProcessHeap(), 0, APCContext);
//             APCContext = NULL;
//         }
//
//         SendInfo.AfdFlags |= AFD_OVERLAPPED;
//     }
//
//     IOSB->Information = 0;
//     IOSB->Status = STATUS_PENDING;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                     Event,
//                                     APCFunction,
//                                     APCContext,
//                                     IOSB,
//                                     IOCTL_AFD_SEND,
//                                     &SendInfo,
//                                     sizeof(SendInfo),
//                                     NULL,
//                                     0);
//
//     /* Wait for completion if not overlapped */
//     if (!lpOverlapped && Status == STATUS_PENDING)
//     {
//         /* FIXME: Shouldn't wait infinitely for send... */
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB->Status;
//     }
//
//     NtClose(SockEvent);
//
//     if (Status == STATUS_PENDING)
//     {
//         TRACE("Leaving (Pending)\n");
//         return MsafdReturnWithErrno(Status, lpErrno, 0, NULL);
//     }
//
//     NumberOfBytesSent = (DWORD)IOSB->Information;
//
//     /* Re-enable Async Event */
//     SockReenableAsyncSelectEvent(Socket, FD_WRITE);
//
//     TRACE("Leaving (%lx %ld)\n", Status, NumberOfBytesSent);
//     return MsafdReturnWithErrno(Status, lpErrno, NumberOfBytesSent, lpNumberOfBytesSent);
// }
//
// int
// WSPAPI
// WSPSendTo(SOCKET Handle,
//           LPWSABUF lpBuffers,
//           DWORD dwBufferCount,
//           LPDWORD lpNumberOfBytesSent,
//           DWORD iFlags,
//           const struct sockaddr *SocketAddress,
//           int SocketAddressLength,
//           LPWSAOVERLAPPED lpOverlapped,
//           LPWSAOVERLAPPED_COMPLETION_ROUTINE lpCompletionRoutine,
//           LPWSATHREADID lpThreadId,
//           LPINT lpErrno)
// {
//     PIO_STATUS_BLOCK        IOSB;
//     IO_STATUS_BLOCK         DummyIOSB;
//     AFD_SEND_INFO_UDP       SendInfo;
//     NTSTATUS                Status;
//     PMSAFD_SEND_APC_CONTEXT APCContext;
//     PIO_APC_ROUTINE         APCFunction = NULL;
//     HANDLE                  Event = NULL;
//     PTRANSPORT_ADDRESS      RemoteAddress;
//     HANDLE                  SockEvent;
//     PSOCKET_INFORMATION     Socket;
//     DWORD                   NumberOfBytesSent;
//     UCHAR                   Buffer[128];
//
//     TRACE("WSPSendTo(%x)\n", Handle);
//
//     /* Get the Socket Structure associate to this Socket */
//     Socket = GetSocketStructure(Handle);
//     if (!Socket)
//     {
//         if (lpErrno)
//             *lpErrno = WSAENOTSOCK;
//         return SOCKET_ERROR;
//     }
//     if (!lpNumberOfBytesSent && !lpOverlapped)
//     {
//         if (lpErrno)
//             *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//     if (!(Socket->SharedData->ServiceFlags1 & XP1_CONNECTIONLESS))
//     {
//         /* Use WSPSend for connection-oriented sockets */
//         return WSPSend(Handle,
//                        lpBuffers,
//                        dwBufferCount,
//                        lpNumberOfBytesSent,
//                        iFlags,
//                        lpOverlapped,
//                        lpCompletionRoutine,
//                        lpThreadId,
//                        lpErrno);
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
//         BindAddress = HeapAlloc(GlobalHeap, 0, BindAddressLength);
//         if (!BindAddress)
//         {
//             MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, NULL);
//             return INVALID_SOCKET;
//         }
//
//         Socket->HelperData->WSHGetWildcardSockaddr(Socket->HelperContext,
//                                                    BindAddress,
//                                                    &BindAddressLength);
//         /* Bind it */
//         BindError = WSPBind(Handle, BindAddress, BindAddressLength, lpErrno);
//         HeapFree(GlobalHeap, 0, BindAddress);
//         if (BindError == SOCKET_ERROR)
//             return SOCKET_ERROR;
//     }
//
//     Status = NtCreateEvent(&SockEvent,
//                         EVENT_ALL_ACCESS,
//                         NULL, SynchronizationEvent, FALSE);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return SOCKET_ERROR;
//     }
//
//     RemoteAddress = (PTRANSPORT_ADDRESS)Buffer;
//
//     if (SocketAddressLength > 128 - sizeof(TRANSPORT_ADDRESS)) {
//         *lpErrno = WSAEFAULT;
//         return SOCKET_ERROR;
//     }
//
//     /* Fill TDI address */
//     RemoteAddress->TAAddressCount = 1;
//     RemoteAddress->Address[0].AddressLength = SocketAddressLength - sizeof(SocketAddress->sa_family);
//     RtlCopyMemory(&RemoteAddress->Address[0].AddressType, SocketAddress, SocketAddressLength);
//
//     /* Set up Structure */
//     SendInfo.BufferArray = (PAFD_WSABUF)lpBuffers;
//     SendInfo.AfdFlags = Socket->SharedData->NonBlocking ? AFD_IMMEDIATE : 0;
//     SendInfo.BufferCount = dwBufferCount;
//     SendInfo.TdiConnection.RemoteAddress = RemoteAddress;
//     SendInfo.TdiConnection.RemoteAddressLength = Socket->HelperData->MaxTDIAddressLength;
//
//     /* Verify if we should use APC */
//     if (!lpOverlapped)
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
//             return MsafdReturnWithErrno(STATUS_SUCCESS, lpErrno, 0, lpNumberOfBytesSent);
//         }
//
//         IOSB = (PIO_STATUS_BLOCK)lpOverlapped;
//         if (lpCompletionRoutine)
//         {
//             SendInfo.AfdFlags |= AFD_SKIP_FIO;
//             APCFunction = &MsafdSendAPC;
//             APCContext = HeapAlloc(GlobalHeap, 0, sizeof(*APCContext));
//             if (!APCContext)
//             {
//                 ERR("Not enough memory for APC Context\n");
//                 NtClose(SockEvent);
//                 return MsafdReturnWithErrno(STATUS_INSUFFICIENT_RESOURCES, lpErrno, 0, lpNumberOfBytesSent);
//             }
//             APCContext->lpCompletionRoutine = lpCompletionRoutine;
//             APCContext->lpOverlapped = lpOverlapped;
//             APCContext->lpSocket = Socket;
//             APCContext->lpRemoteAddress = RemoteAddress;
//             Event = NULL;
//         }
//         else
//         {
//             // .NET expects APCContext to be a pointer to lpOverlapped, because it adds some additional metadata past OVERLAPPED structure
//             // Do not use APC when there is no completion context
//             Event = lpOverlapped->hEvent;
//             
//             APCFunction = NULL;
//             if (Event)
//             {
//                 APCContext = NULL;
//             }
//             else
//             {
//                 APCContext = (PMSAFD_SEND_APC_CONTEXT)lpOverlapped;
//             }
//         }
//
//         SendInfo.AfdFlags |= AFD_OVERLAPPED;
//     }
//
//     IOSB->Information = 0;
//     IOSB->Status = STATUS_PENDING;
//
//     /* Send IOCTL */
//     Status = NtDeviceIoControlFile((HANDLE)Handle,
//                                    Event,
//                                    APCFunction,
//                                    APCContext,
//                                    IOSB,
//                                    IOCTL_AFD_SEND_DATAGRAM,
//                                    &SendInfo,
//                                    sizeof(SendInfo),
//                                    NULL,
//                                    0);
//
//     /* Wait for completion if not overlapped */
//     if (!lpOverlapped && Status == STATUS_PENDING)
//     {
//         /* FIXME: Shouldn't wait infinitely for send... */
//         MsafdWaitForAlert(SockEvent);
//         Status = IOSB->Status;
//     }
//
//     NtClose(SockEvent);
//
//     if (Status == STATUS_PENDING)
//     {
//         TRACE("Leaving (Pending)\n");
//         return MsafdReturnWithErrno(Status, lpErrno, 0, NULL);
//     }
//
//     NumberOfBytesSent = (DWORD)IOSB->Information;
//
//     /* Re-enable Async Event */
//     SockReenableAsyncSelectEvent(Socket, FD_WRITE);
//
//     TRACE("Leaving (%lx %ld)\n", Status, NumberOfBytesSent);
//     return MsafdReturnWithErrno(Status, lpErrno, NumberOfBytesSent, lpNumberOfBytesSent);
// }
//
// INT
// WSPAPI
// WSPRecvDisconnect(IN  SOCKET s,
//                   OUT LPWSABUF lpInboundDisconnectData,
//                   OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
//
//
// INT
// WSPAPI
// WSPSendDisconnect(IN  SOCKET s,
//                   IN  LPWSABUF lpOutboundDisconnectData,
//                   OUT LPINT lpErrno)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
