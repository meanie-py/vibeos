// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_62A39F1BCBB9: [u16; 12] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0055, 0x0064,
    0x0070, 0x0000,
];
pub static RUSTCONV_WSTR_9112492AFDA5: [u16; 12] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0054, 0x0063,
    0x0070, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoAllocateIrp, IoCancelIrp, IoFreeIrp, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, ObReferenceObjectByHandle, PDEVICE_OBJECT, PDRIVER_OBJECT, PETHREAD, PIRP, PKEVENT, PLARGE_INTEGER, PSECURITY_DESCRIPTOR, PUNICODE_STRING, PsCreateSystemThread, PsGetCurrentThread, PsTerminateSystemThread, RtlInitUnicodeString, _EVENT_TYPE_SynchronizationEvent, ZwClose};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut SocketsToDereference: PWSK_SOCKET_INTERNAL =
    ::core::ptr::null::<_WSK_SOCKET_INTERNAL>() as *mut _WSK_SOCKET_INTERNAL;

static mut SocketsToDereferenceListLock: KSPIN_LOCK = 0 as KSPIN_LOCK;

static mut DereferenceSocketsThreadShouldRun: BOOLEAN = FALSE as BOOLEAN;

// C: `static KEVENT DereferenceSocketsEvent;` -- all zeros until KeInitializeEvent.
static mut DereferenceSocketsEvent: KEVENT = unsafe { core::mem::zeroed() };

static mut DereferenceSocketsThreadHandle: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

// wsk.h embeds the basic dispatch as an anonymous member named only by its typedef
// (`WSK_PROVIDER_BASIC_DISPATCH;`, an MS extension): bindgen has no field to emit for it and
// pads the island's struct instead, so a table that fills it cannot be written as the island
// type. These mirror the C layout, are size-checked against the island's, and are handed out
// only as pointers, as the C does.
#[repr(C)]
pub struct WskProviderDatagramDispatch {
    pub Basic: WSK_PROVIDER_BASIC_DISPATCH,
    pub WskBind: PFN_WSK_BIND,
    pub WskSendTo: PFN_WSK_SEND_TO,
    pub WskReceiveFrom: PFN_WSK_RECEIVE_FROM,
    pub WskRelease: PFN_WSK_RELEASE_DATAGRAM_INDICATION_LIST,
    pub WskGetLocalAddress: PFN_WSK_GET_LOCAL_ADDRESS,
}
#[repr(C)]
pub struct WskProviderConnectionDispatch {
    pub Basic: WSK_PROVIDER_BASIC_DISPATCH,
    pub WskBind: PFN_WSK_BIND,
    pub WskConnect: PFN_WSK_CONNECT,
    pub WskGetLocalAddress: PFN_WSK_GET_LOCAL_ADDRESS,
    pub WskGetRemoteAddress: PFN_WSK_GET_REMOTE_ADDRESS,
    pub WskSend: PFN_WSK_SEND,
    pub WskReceive: PFN_WSK_RECEIVE,
    pub WskDisconnect: PFN_WSK_DISCONNECT,
    pub WskRelease: PFN_WSK_RELEASE_DATA_INDICATION_LIST,
}
const _: () = assert!(
    core::mem::size_of::<WskProviderDatagramDispatch>()
        == core::mem::size_of::<WSK_PROVIDER_DATAGRAM_DISPATCH>()
);
const _: () = assert!(
    core::mem::size_of::<WskProviderConnectionDispatch>()
        == core::mem::size_of::<WSK_PROVIDER_CONNECTION_DISPATCH>()
);

static mut UdpDispatch: WskProviderDatagramDispatch = unsafe {
    WskProviderDatagramDispatch {
        Basic: _WSK_PROVIDER_BASIC_DISPATCH {
            WskControlSocket: Some(
                WskControlSocket,
            ),
            WskCloseSocket: Some(
                WskCloseSocket,
            ),
        },
        WskBind: Some(
            WskBind,
        ),
        WskSendTo: Some(
            WskSendTo,
        ),
        WskReceiveFrom: Some(
            WskReceiveFrom,
        ),
        WskRelease: Some(
            WskReleaseUdp,
        ),
        WskGetLocalAddress: Some(
            WskGetLocalAddress,
        ),
    }
};

static mut TcpDispatch: WskProviderConnectionDispatch = unsafe {
    WskProviderConnectionDispatch {
        Basic: _WSK_PROVIDER_BASIC_DISPATCH {
            WskControlSocket: Some(
                WskControlSocket,
            ),
            WskCloseSocket: Some(
                WskCloseSocket,
            ),
        },
        WskBind: Some(
            WskBind,
        ),
        WskConnect: Some(
            WskConnect,
        ),
        WskGetLocalAddress: Some(
            WskGetLocalAddress,
        ),
        WskGetRemoteAddress: Some(
            WskGetRemoteAddress,
        ),
        WskSend: Some(
            WskSend,
        ),
        WskReceive: Some(
            WskReceive,
        ),
        WskDisconnect: Some(
            WskDisconnect,
        ),
        WskRelease: Some(
            WskReleaseTcp,
        ),
    }
};

static mut provider_dispatch: WSK_PROVIDER_DISPATCH = unsafe {
    _WSK_PROVIDER_DISPATCH {
        Version: 0 as USHORT,
        Reserved: 0 as USHORT,
        WskSocket: Some(
            WskSocket,
        ),
        WskSocketConnect: Some(
            WskSocketConnect,
        ),
        WskControlClient: Some(
            WskControlClient,
        ),
    }
};

pub const NDEBUG: i32 = 1; // nolint: const-contradiction (netio.c's own #define NDEBUG)

pub const TAG_NETIO: u32 = 0x4F49_454E; // 'OIEN'

pub const TAG_AFD_TDI_CONNECTION_INFORMATION: u32 = 0x6354_6641; // 'cTfA'

pub const AFD_SHARE_UNIQUE: i32 = 0x0;

pub const AFD_SHARE_REUSE: i32 = 0x1;

pub const AFD_SHARE_WILDCARD: i32 = 0x2;

pub const AFD_SHARE_EXCLUSIVE: i32 = 0x3;

#[repr(C)]
pub struct _WSK_SOCKET_INTERNAL {
    pub Socket: WSK_SOCKET,
    pub family: ADDRESS_FAMILY,
    pub type_: USHORT,
    pub proto: ULONG,
    pub WskFlags: ULONG,
    pub user_context: PVOID,
    pub TdiName: _UNICODE_STRING,
    pub LocalAddress: sockaddr,
    pub LocalAddressFile: PFILE_OBJECT,
    pub LocalAddressHandle: HANDLE,
    pub RemoteAddress: sockaddr,
    pub RemoteAddressFile: PFILE_OBJECT,
    pub RemoteAddressHandle: HANDLE,
    pub ConnectionFile: PFILE_OBJECT,
    pub ConnectionHandle: HANDLE,
    pub ConnectionFileAssociated: BOOLEAN,
    pub ListenDispatch: *const WSK_CLIENT_LISTEN_DISPATCH,
    pub CallbackMask: UINT,
    pub Flags: UINT,
    pub RefCount: LONG,
    pub ListenIrp: PIRP,
    pub ListenCancelled: BOOLEAN,
    pub ListenThreadHandle: HANDLE,
    pub ListenThread: PKTHREAD,
    pub StartListenEvent: KEVENT,
    pub ListenThreadShouldRun: BOOLEAN,
    pub ListenSocket: *mut _WSK_SOCKET_INTERNAL,
    pub NextSocketToDereference: *mut _WSK_SOCKET_INTERNAL,
    pub NumSocketDereferences: ULONG,
}
pub type WSK_SOCKET_INTERNAL = _WSK_SOCKET_INTERNAL;
pub type PWSK_SOCKET_INTERNAL = *mut WSK_SOCKET_INTERNAL;

#[repr(C)]
pub struct _NETIO_CONTEXT {
    pub UserIrp: PIRP,
    pub socket: PWSK_SOCKET_INTERNAL,
    pub TargetConnectionInfo: PTDI_CONNECTION_INFORMATION,
    pub PeerAddrRet: PTDI_CONNECTION_INFORMATION,
}
pub type NETIO_CONTEXT = _NETIO_CONTEXT;
pub type PNETIO_CONTEXT = *mut NETIO_CONTEXT;

#[repr(C)]
pub struct _LISTEN_CONTEXT {
    pub ListenSocket: PWSK_SOCKET_INTERNAL,
    pub AcceptSocket: PWSK_SOCKET_INTERNAL,
    pub RequestConnectionInfo: PTDI_CONNECTION_INFORMATION,
    pub ReturnConnectionInfo: PTDI_CONNECTION_INFORMATION,
}
pub type LISTEN_CONTEXT = _LISTEN_CONTEXT;
pub type PLISTEN_CONTEXT = *mut LISTEN_CONTEXT;

pub const DIR_SEND: i32 = 0;
pub const DIR_RECEIVE: i32 = 1;

unsafe extern "C" fn ReferenceSocket(mut Socket: PWSK_SOCKET_INTERNAL) {
    FUNCTION_TRACE;

    InterlockedIncrement(core::ptr::addr_of_mut!((*Socket).RefCount) as *mut ::core::ffi::c_long);
}

unsafe extern "C" fn SocketShutdown(mut Socket: PWSK_SOCKET_INTERNAL) {
    let mut status: NTSTATUS = 0;

    FUNCTION_TRACE;

    if !(*Socket).ListenSocket.is_null() {
        DereferenceSocket((*Socket).ListenSocket);
        (*Socket).ListenSocket = core::ptr::null_mut();
    }
    if !(*Socket).ListenThreadHandle.is_null() {
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*Socket).ListenThreadShouldRun),
            FALSE,
        );
        KeSetEvent(
            core::ptr::addr_of_mut!((*Socket).StartListenEvent),
            (IO_NO_INCREMENT) as KPRIORITY,
            FALSE,
        );
        status = KeWaitForSingleObject(
            (*Socket).ListenThread as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS!(status) {
            DPRINT1!("KeWaitForSingleObject failed with status 0x%08x!\n", status);
        }
        ObDereferenceObject((*Socket).ListenThread as PVOID);
        (*Socket).ListenThread = core::ptr::null_mut();
        (*Socket).ListenThreadHandle = core::ptr::null_mut();
    }
    if !(*Socket).ListenIrp.is_null() {
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*Socket).ListenCancelled),
            TRUE,
        );
        IoCancelIrp((*Socket).ListenIrp);
        (*Socket).ListenIrp = core::ptr::null_mut();
    }
    if !(*Socket).ConnectionFile.is_null() {
        if (*Socket).ConnectionFileAssociated != 0 {
            /* This fails with error STATUS_CONNECTION_ACTIVE on Windows 2003 */
            status = TdiDisassociateAddressFile((*Socket).ConnectionFile);
            if !NT_SUCCESS!(status) {
                DPRINT1!("Warning: TdiDisassociateAddressFile returned status %08x\n", status);
            }
            (*Socket).ConnectionFileAssociated = FALSE;
        }
        ObDereferenceObject((*Socket).ConnectionFile as PVOID);
        (*Socket).ConnectionFile = core::ptr::null_mut();
    }
    if !(*Socket).ConnectionHandle.is_null() {
        ZwClose((*Socket).ConnectionHandle);
        (*Socket).ConnectionHandle = core::ptr::null_mut();
    }
}

unsafe extern "C" fn SocketDestroy(Socket: PWSK_SOCKET_INTERNAL) {
    SocketShutdown(Socket);

    /* Especially listen sockets must keep the LocalAddressFile
     * open when being closed and there are still accepted sockets
     * somewhere. Else the accepted socket I/O will fail with
     * STATUS_INVALID_DEVICE_STATE. So do not close the
     * address file in Shutdown only close it here when all AcceptSockets
     * are gone (AcceptSockets hold a reference to the listen socket).
     */

    if !(*Socket).LocalAddressFile.is_null() {
        ObDereferenceObject((*Socket).LocalAddressFile as PVOID);
        (*Socket).LocalAddressFile = core::ptr::null_mut();
    }
    if !(*Socket).LocalAddressHandle.is_null() {
        ZwClose((*Socket).LocalAddressHandle);
        (*Socket).LocalAddressHandle = core::ptr::null_mut();
    }
    ExFreePoolWithTag(Socket as PVOID, TAG_NETIO as ULONG);
}

unsafe extern "C" fn DereferenceSocketSynchronous(Socket: PWSK_SOCKET_INTERNAL) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*Socket).RefCount)) == 0 {
        SocketDestroy(Socket);
    }
}

unsafe extern "system" fn DereferenceSocketsThread(p: *mut core::ffi::c_void) {
    let mut status: NTSTATUS = 0;
    let mut SocketToDereference: PWSK_SOCKET_INTERNAL = core::ptr::null_mut();
    let mut OldIrql: KIRQL = 0;
    let mut NumSocketDereferences: ULONG = 0;

    FUNCTION_TRACE;

    while DereferenceSocketsThreadShouldRun != 0 {
        status = KeWaitForSingleObject(
            &raw mut DereferenceSocketsEvent as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS!(status) {
            DPRINT1!("KeWaitForSingleObject failed with status 0x%08x!\n", status);
        }

        KeAcquireSpinLock(&raw mut SocketsToDereferenceListLock, &raw mut OldIrql);
        while !SocketsToDereference.is_null() {
            SocketToDereference = SocketsToDereference;
            SocketsToDereference = (*SocketToDereference).NextSocketToDereference;
            NumSocketDereferences = (*SocketToDereference).NumSocketDereferences;

            while NumSocketDereferences > 0 {
                (*SocketToDereference).NumSocketDereferences =
                    (*SocketToDereference).NumSocketDereferences.wrapping_sub(1);
                NumSocketDereferences = (*SocketToDereference).NumSocketDereferences;

                KeReleaseSpinLock(&raw mut SocketsToDereferenceListLock, OldIrql);
                DereferenceSocketSynchronous(SocketToDereference);
                KeAcquireSpinLock(&raw mut SocketsToDereferenceListLock, &raw mut OldIrql);
            }
        }
        KeReleaseSpinLock(&raw mut SocketsToDereferenceListLock, OldIrql);
    }

    PsTerminateSystemThread(STATUS_SUCCESS);
}

unsafe extern "C" fn StartSocketDereferenceThread() -> NTSTATUS {
    let mut status: NTSTATUS;

    KeInitializeSpinLock(core::ptr::addr_of_mut!(SocketsToDereferenceListLock));
    KeInitializeEvent(core::ptr::addr_of_mut!(DereferenceSocketsEvent), _EVENT_TYPE_SynchronizationEvent, FALSE);
    DereferenceSocketsThreadShouldRun = TRUE;

    status = PsCreateSystemThread(core::ptr::addr_of_mut!(DereferenceSocketsThreadHandle), THREAD_ALL_ACCESS, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), Some(DereferenceSocketsThread), core::ptr::null_mut());
    if !NT_SUCCESS!(status) {
        DPRINT1!("Could not start put sockets thread, status is %x\n", status);
    }
    return status;
}

unsafe extern "C" fn StopSocketDereferenceThread() {
    ::core::ptr::write_volatile(
        core::ptr::addr_of_mut!(DereferenceSocketsThreadShouldRun),
        FALSE as BOOLEAN,
    );
    KeSetEvent(
        core::ptr::addr_of_mut!(DereferenceSocketsEvent),
        IO_NO_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );

    /* eventually it will terminate, no need to wait for that. */
}

unsafe extern "C" fn DereferenceSocket(Socket: PWSK_SOCKET_INTERNAL) {
    let mut OldIrql: KIRQL = 0;

    FUNCTION_TRACE;

    if KeGetCurrentIrql() as ::core::ffi::c_int > (PASSIVE_LEVEL as i32) {
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!(SocketsToDereferenceListLock),
            core::ptr::addr_of_mut!(OldIrql),
        );
        if (*Socket).NumSocketDereferences == 0 {
            (*Socket).NextSocketToDereference = SocketsToDereference;
            SocketsToDereference = Socket;
            (*Socket).NumSocketDereferences = 1;
        } else {
            (*Socket).NumSocketDereferences = (*Socket).NumSocketDereferences.wrapping_add(1);
        }
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!(SocketsToDereferenceListLock),
            OldIrql,
        );

        KeSetEvent(
            core::ptr::addr_of_mut!(DereferenceSocketsEvent),
            (IO_NO_INCREMENT) as KPRIORITY,
            FALSE,
        );

        return;
    }
    DereferenceSocketSynchronous(Socket);
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    FUNCTION_TRACE;

    return STATUS_SUCCESS;
}

unsafe extern "system" fn NetioComplete(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
    mut ContextParam: PVOID,
) -> NTSTATUS {
    let mut Context: PNETIO_CONTEXT = ContextParam as PNETIO_CONTEXT;
    let mut UserIrp: PIRP = (*Context).UserIrp;

    let _ = FUNCTION_TRACE;

    if (*Irp).IoStatus.__bindgen_anon_1.Status != STATUS_CANCELLED {
        (*UserIrp).IoStatus.__bindgen_anon_1.Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        (*UserIrp).IoStatus.Information = (*Irp).IoStatus.Information;
    }
    if !(*Context).PeerAddrRet.is_null() {
        let mut RemoteAddress: PSOCKADDR =
            &raw mut (*(&raw mut (*((*(*Context).PeerAddrRet).RemoteAddress as PTRANSPORT_ADDRESS))
                .Address as *mut TA_ADDRESS)
                .offset(0 as ::core::ffi::c_int as isize))
            .AddressType as PSOCKADDR;
        memcpy(
            &raw mut (*(*Context).socket).RemoteAddress as *mut ::core::ffi::c_void,
            RemoteAddress as *const ::core::ffi::c_void,
            ::core::mem::size_of::<sockaddr>() as size_t,
        );
    }
    if (*Irp).IoStatus.__bindgen_anon_1.Status != STATUS_CANCELLED {
        IoCompleteRequest(UserIrp, IO_NETWORK_INCREMENT as CCHAR);
    }

    DereferenceSocket((*Context).socket);
    if !(*Context).TargetConnectionInfo.is_null() {
        ExFreePoolWithTag(
            (*Context).TargetConnectionInfo as PVOID,
            TAG_AFD_TDI_CONNECTION_INFORMATION as ULONG,
        );
    }
    if !(*Context).PeerAddrRet.is_null() {
        ExFreePoolWithTag(
            (*Context).PeerAddrRet as PVOID,
            TAG_AFD_TDI_CONNECTION_INFORMATION as ULONG,
        );
    }
    ExFreePoolWithTag(Context as PVOID, TAG_NETIO as ULONG);

    return STATUS_SUCCESS;
}

unsafe extern "system" fn CompletionFireEvent(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
    mut Context: PVOID,
) -> NTSTATUS {
    let mut Event: PKEVENT = Context as PKEVENT;

    FUNCTION_TRACE;

    KeSetEvent(Event as PRKEVENT, IO_NO_INCREMENT as KPRIORITY, FALSE as BOOLEAN);

    return STATUS_MORE_PROCESSING_REQUIRED;
}

unsafe extern "C" fn CreateSocket(
    mut AddressFamily: ADDRESS_FAMILY,
    mut SocketType: USHORT,
    mut Protocol: ULONG,
    mut Flags: ULONG,
    mut TheSocket: *mut PWSK_SOCKET_INTERNAL,
) -> NTSTATUS {
    let mut NewSocketIrp: PIRP = core::ptr::null_mut();
    let mut CompletionEvent: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;

    FUNCTION_TRACE;

    NewSocketIrp = IoAllocateIrp(1, FALSE);
    if NewSocketIrp.is_null() {
        DPRINT1!("Out of memory?\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    KeInitializeEvent(&raw mut CompletionEvent, _EVENT_TYPE_NotificationEvent, FALSE);
    IoSetCompletionRoutine(
        NewSocketIrp,
        Some(CompletionFireEvent),
        &raw mut CompletionEvent as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );
    (*NewSocketIrp).Tail.Overlay.Thread = PsGetCurrentThread();

    Status = WskSocket(
        core::ptr::null_mut(),
        AddressFamily,
        SocketType,
        Protocol,
        Flags,
        core::ptr::null_mut(),
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        NewSocketIrp,
    );

    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut CompletionEvent as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        Status = (*NewSocketIrp).IoStatus.__bindgen_anon_1.Status;
    }

    if NT_SUCCESS!(Status) {
        *TheSocket = (*NewSocketIrp).IoStatus.Information as PWSK_SOCKET_INTERNAL;
    }

    IoFreeIrp(NewSocketIrp);
    return Status;
}

unsafe extern "C" fn QueueListening(mut ListenSocket: PWSK_SOCKET_INTERNAL) {
    let _ = FUNCTION_TRACE;

    KeSetEvent(
        core::ptr::addr_of_mut!((*ListenSocket).StartListenEvent),
        IO_NO_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );
}

unsafe extern "system" fn ListenComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let ListenContext: PLISTEN_CONTEXT = Context as PLISTEN_CONTEXT;
    let ListenSocket: PWSK_SOCKET_INTERNAL = (*ListenContext).ListenSocket;
    let AcceptSocket: PWSK_SOCKET_INTERNAL = (*ListenContext).AcceptSocket;
    let ListenDispatch: PWSK_CLIENT_LISTEN_DISPATCH =
        (*ListenSocket).ListenDispatch as PWSK_CLIENT_LISTEN_DISPATCH;

    let RemoteAddress: PSOCKADDR = core::ptr::addr_of_mut!(
        (*((*(*ListenContext).ReturnConnectionInfo).RemoteAddress as PTRANSPORT_ADDRESS))
            .Address[0]
            .AddressType
    ) as PSOCKADDR;
    let mut AcceptSocketContext: PVOID = core::ptr::null_mut();
    let mut AcceptSocketDispatch: *const WSK_CLIENT_CONNECTION_DISPATCH = core::ptr::null();
    let mut Status: NTSTATUS;

    FUNCTION_TRACE;

    if ((*ListenSocket).CallbackMask & WSK_EVENT_ACCEPT) != 0
        && (*ListenDispatch).WskAcceptEvent.is_some()
        && !(*ListenSocket).ListenIrp.is_null()
        && (*ListenSocket).ListenCancelled == 0
        && (*Irp).IoStatus.__bindgen_anon_1.Status != STATUS_CANCELLED
    {
        (*ListenSocket).ListenIrp = core::ptr::null_mut();

        Status = (*ListenDispatch)
            .WskAcceptEvent
            .unwrap_unchecked()(
            (*ListenSocket).user_context,
            0,
            core::ptr::addr_of_mut!((*ListenSocket).LocalAddress),
            RemoteAddress,
            core::ptr::addr_of_mut!((*AcceptSocket).Socket),
            core::ptr::addr_of_mut!(AcceptSocketContext),
            core::ptr::addr_of_mut!(AcceptSocketDispatch),
        );

        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                "ListenDispatch->WskAcceptEvent returned non-successful status 0x%08x\n",
                Status
            );
        } else {
            memcpy(
                core::ptr::addr_of_mut!((*AcceptSocket).RemoteAddress) as *mut core::ffi::c_void,
                RemoteAddress as *const core::ffi::c_void,
                core::mem::size_of::<SOCKADDR>(),
            );
        }
    } else {
        DereferenceSocket(AcceptSocket); /* close the AcceptSocket */
    }
    (*ListenSocket).ListenIrp = core::ptr::null_mut();

    DereferenceSocket(AcceptSocket);
    DereferenceSocket(ListenSocket);

    ExFreePoolWithTag(
        (*ListenContext).ReturnConnectionInfo as PVOID,
        TAG_AFD_TDI_CONNECTION_INFORMATION,
    );
    ExFreePoolWithTag(
        (*ListenContext).RequestConnectionInfo as PVOID,
        TAG_AFD_TDI_CONNECTION_INFORMATION,
    );
    ExFreePoolWithTag(ListenContext as PVOID, TAG_NETIO);

    /* And wait for the next incoming connection.
     * This is done in a separate thread at IRQL = PASSIVE_LEVEL
     */

    QueueListening(ListenSocket);

    return STATUS_SUCCESS;
}

unsafe extern "C" fn StartListening(mut ListenSocket: PWSK_SOCKET_INTERNAL) -> NTSTATUS {
    let mut tdiIrp: PIRP = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;
    let mut lc: PLISTEN_CONTEXT = core::ptr::null_mut();
    let mut AcceptSocket: PWSK_SOCKET_INTERNAL = core::ptr::null_mut();

    FUNCTION_TRACE;

    if (*ListenSocket).LocalAddressHandle.is_null() {
        DPRINT1!("LocalAddressHandle is not set, need to bind your socket before listening\n");
        return STATUS_INVALID_PARAMETER;
    }

    status = CreateSocket(
        (*ListenSocket).family,
        (*ListenSocket).type_,
        (*ListenSocket).proto,
        WSK_FLAG_CONNECTION_SOCKET,
        core::ptr::addr_of_mut!(AcceptSocket),
    );
    if !NT_SUCCESS!(status) {
        DPRINT1!("Could not create AcceptSocket, status is 0x%08x\n", status);
        return status;
    }
    (*AcceptSocket).ListenSocket = ListenSocket;
    /* Dereferenced when the AcceptSocket is closed */
    ReferenceSocket((*AcceptSocket).ListenSocket);

    'free_accept: {
        status = STATUS_INSUFFICIENT_RESOURCES;
        lc = ExAllocatePoolUninitialized(
            NonPagedPool,
            (core::mem::size_of::<LISTEN_CONTEXT>() as SIZE_T) as _,
            TAG_NETIO,
        ) as PLISTEN_CONTEXT;
        if lc.is_null() {
            break 'free_accept;
        }
        (*lc).ListenSocket = ListenSocket;
        (*lc).AcceptSocket = AcceptSocket;

        tdiIrp = core::ptr::null_mut();

        'free_lc: {
            status = TdiAssociateAddressFile((*ListenSocket).LocalAddressHandle, (*AcceptSocket).ConnectionFile);
            if !NT_SUCCESS!(status) {
                break 'free_lc;
            }
            (*AcceptSocket).ConnectionFileAssociated = TRUE;

            (*lc).RequestConnectionInfo = core::ptr::null_mut();
            TdiBuildNullConnectionInfo(core::ptr::addr_of_mut!((*lc).RequestConnectionInfo), TDI_ADDRESS_TYPE_IP);

            'free_lc_and_disassociate: {
                if (*lc).RequestConnectionInfo.is_null() {
                    break 'free_lc_and_disassociate;
                }
                (*(*lc).RequestConnectionInfo).OptionsLength = 0;

                'free_lc_and_req_conn_info: {
                    (*lc).ReturnConnectionInfo = core::ptr::null_mut();
                    TdiBuildNullConnectionInfo(core::ptr::addr_of_mut!((*lc).ReturnConnectionInfo), TDI_ADDRESS_TYPE_IP);
                    if (*lc).ReturnConnectionInfo.is_null() {
                        break 'free_lc_and_req_conn_info;
                    }
                    (*(*lc).ReturnConnectionInfo).OptionsLength = 0;

                    ReferenceSocket(ListenSocket);
                    ReferenceSocket(AcceptSocket);

                    core::ptr::write_volatile(
                        core::ptr::addr_of_mut!((*ListenSocket).ListenCancelled),
                        FALSE,
                    );
                    /* pass the ConnectionFile from accept socket, not from ListenSocket ... */
                    status = TdiListen(
                        core::ptr::addr_of_mut!(tdiIrp),
                        (*AcceptSocket).ConnectionFile,
                        core::ptr::addr_of_mut!((*lc).RequestConnectionInfo),
                        core::ptr::addr_of_mut!((*lc).ReturnConnectionInfo),
                        Some(ListenComplete),
                        lc as PVOID,
                    );

                    if !NT_SUCCESS!(status) {
                        ExFreePoolWithTag((*lc).ReturnConnectionInfo as PVOID, TAG_NETIO);
                        DereferenceSocket(ListenSocket);
                        DereferenceSocket(AcceptSocket);
                        break 'free_lc_and_req_conn_info;
                    }
                    (*ListenSocket).ListenIrp = tdiIrp;

                    return STATUS_PENDING;
                }
                /* err_out_free_lc_and_req_conn_info */
                ExFreePoolWithTag((*lc).RequestConnectionInfo as PVOID, TAG_NETIO);
            }
            /* err_out_free_lc_and_disassociate */
            status = TdiDisassociateAddressFile((*AcceptSocket).ConnectionFile);
            if !NT_SUCCESS!(status) {
                DPRINT1!("Warning: TdiDisassociateAddressFile returned status %08x\n", status);
            }
            (*AcceptSocket).ConnectionFileAssociated = FALSE;
        }
        /* err_out_free_lc */
        ExFreePoolWithTag(lc as PVOID, TAG_NETIO);
    }
    /* err_out_free_accept_socket */
    DereferenceSocket(AcceptSocket);

    return status;
}

unsafe extern "system" fn RequeueListenThread(p: PVOID) {
    let ListenSocket: PWSK_SOCKET_INTERNAL = p as PWSK_SOCKET_INTERNAL;
    let mut status: NTSTATUS;

    FUNCTION_TRACE;

    while (*ListenSocket).ListenThreadShouldRun != 0 {
        status = KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*ListenSocket).StartListenEvent) as PVOID,
            _KWAIT_REASON_Executive,
            (0i32) as KPROCESSOR_MODE,
            FALSE,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS!(status) {
            DPRINT1!("KeWaitForSingleObject failed with status 0x%08x!\n", status);
        }
        if (*ListenSocket).ListenThreadShouldRun == 0 {
            break;
        }
        StartListening(ListenSocket);
    }
    PsTerminateSystemThread(STATUS_SUCCESS);
}
unsafe extern "system" fn WskControlSocket(
    SocketParam: PWSK_SOCKET,
    RequestType: WSK_CONTROL_SOCKET_TYPE,
    ControlCode: ULONG,
    Level: ULONG,
    InputSize: SIZE_T,
    InputBuffer: PVOID,
    OutputSize: SIZE_T,
    OutputBuffer: PVOID,
    OutputSizeReturned: *mut SIZE_T,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Socket: PWSK_SOCKET_INTERNAL = CONTAINING_RECORD!(SocketParam, WSK_SOCKET_INTERNAL, Socket);
    let mut status: NTSTATUS = STATUS_NOT_IMPLEMENTED;

    FUNCTION_TRACE;

    if Socket.is_null() {
        status = STATUS_INVALID_PARAMETER;
    } else {
        IoSetNextIrpStackLocation(Irp);

        match RequestType as ::core::ffi::c_uint {
            WskSetOption => {
                match Level {
                    SOL_SOCKET => {
                        match ControlCode {
                            SO_REUSEADDR => {
                                /* add more supported flags here */
                                if InputBuffer.is_null() {
                                    DPRINT1!("WskControlSocket: Need an InputBuffer for this operation\n");
                                    status = STATUS_INVALID_PARAMETER;
                                } else if InputSize < (core::mem::size_of::<UINT>()) as _ {
                                    DPRINT1!("WskControlSocket: InputBuffer too small for this operation\n");
                                    status = STATUS_INVALID_PARAMETER;
                                } else {
                                    let mut enable: UINT = *(InputBuffer as PUINT);
                                    if enable != 0 {
                                        (*Socket).Flags = ((*Socket).Flags as ULONG | ControlCode) as UINT;
                                    } else {
                                        (*Socket).Flags = ((*Socket).Flags as ULONG & !ControlCode) as UINT;
                                    }
                                    status = STATUS_SUCCESS;
                                }
                            }

                            /* Windows specific. This sets the mask for callback functions. */
                            SO_WSK_EVENT_CALLBACK => {
                                if InputBuffer.is_null() {
                                    DPRINT1!("WskControlSocket: Need an InputBuffer for this operation\n");
                                    status = STATUS_INVALID_PARAMETER;
                                } else if InputSize < (core::mem::size_of::<WSK_EVENT_CALLBACK_CONTROL>()) as _ {
                                    DPRINT1!("WskControlSocket: InputBuffer too small for this operation\n");
                                    status = STATUS_INVALID_PARAMETER;
                                } else {
                                    let mut Control: PWSK_EVENT_CALLBACK_CONTROL =
                                        InputBuffer as PWSK_EVENT_CALLBACK_CONTROL;

                                    if ((*Socket).CallbackMask & (WSK_EVENT_ACCEPT as UINT)
                                        == (0 as UINT))
                                        && (((*Control).EventMask & (WSK_EVENT_ACCEPT as ULONG))
                                            == (WSK_EVENT_ACCEPT as ULONG))
                                    {
                                        (*Socket).CallbackMask = (*Control).EventMask as UINT;
                                        QueueListening(Socket);
                                    } else {
                                        (*Socket).CallbackMask = (*Control).EventMask as UINT;
                                    }

                                    status = STATUS_SUCCESS;
                                }
                            }

                            _ => {
                                DPRINT1!("WskControlSocket: ControlCode %d Not implemented\n", ControlCode);
                            }
                        }
                    }
                    _ => {
                        DPRINT1!("WskControlSocket: Level %d Not implemented\n", Level);
                    }
                }
            }

            WskGetOption | WskIoctl | _ => {
                DPRINT1!("WskControlSocket: Option %d Not implemented\n", RequestType);
            }
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));

    return STATUS_PENDING;
}
unsafe extern "system" fn WskCloseSocket(SocketParam: PWSK_SOCKET, Irp: PIRP) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut Socket: PWSK_SOCKET_INTERNAL = CONTAINING_RECORD!(SocketParam, WSK_SOCKET_INTERNAL, Socket);

    IoSetNextIrpStackLocation(Irp);

    /* There might be a reference from (for example) a pending
     * receive. Shutdown the socket here explicitly. We expect
     * all pending I/O operations to be cancelled, then.
     */

    SocketShutdown(Socket);
    DereferenceSocket(Socket);

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));

    return STATUS_PENDING;
}

unsafe extern "C" fn TdiTransportAddressFromSocketAddress(mut SocketAddress: PSOCKADDR) -> PTRANSPORT_ADDRESS {
    let mut ta: PTRANSPORT_ADDRESS = core::ptr::null_mut();
    let mut Size: ULONG;
    Size = (core::mem::offset_of!(TRANSPORT_ADDRESS, Address)
        + core::mem::offset_of!(TA_ADDRESS, Address)
        + core::mem::size_of::<[CHAR; 14]>()) as ULONG;

    ta = ExAllocatePoolUninitialized(NonPagedPool, (Size as SIZE_T) as _, TAG_NETIO as ULONG)
        as PTRANSPORT_ADDRESS;
    if ta.is_null() {
        DPRINT1!("TdiTransportAddressFromSocketAddress: Out of memory\n");
        return core::ptr::null_mut();
    }

    (*ta).TAAddressCount = 1 as LONG;
    (*(&raw mut (*ta).Address as *mut TA_ADDRESS).offset(0 as isize))
        .AddressLength = core::mem::size_of::<[CHAR; 14]>() as USHORT;
    (*(&raw mut (*ta).Address as *mut TA_ADDRESS).offset(0 as isize))
        .AddressType = TDI_ADDRESS_TYPE_IP as USHORT;

    memcpy(
        (&raw mut (*(&raw mut (*ta).Address as *mut TA_ADDRESS)
            .offset(0 as isize))
        .Address as *mut UCHAR)
            .offset(0 as isize) as *mut core::ffi::c_void,
        &raw const (*SocketAddress).sa_data as *const core::ffi::c_void,
        (*(&raw mut (*ta).Address as *mut TA_ADDRESS).offset(0 as isize))
            .AddressLength as usize,
    );

    return ta;
}

unsafe extern "C" fn TdiConnectionInfoFromSocketAddress(
    SocketAddress: PSOCKADDR,
) -> PTDI_CONNECTION_INFORMATION {
    let mut TargetAddress: PTRANSPORT_ADDRESS = core::ptr::null_mut();
    let mut ConnectionInformation: PTDI_CONNECTION_INFORMATION = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;

    TargetAddress = TdiTransportAddressFromSocketAddress(SocketAddress);
    if TargetAddress.is_null() {
        return core::ptr::null_mut();
    }

    status = TdiBuildConnectionInfo(core::ptr::addr_of_mut!(ConnectionInformation), TargetAddress);
    ExFreePoolWithTag(TargetAddress as PVOID, TAG_NETIO as ULONG);

    if !NT_SUCCESS!(status) {
        if !ConnectionInformation.is_null() {
            ExFreePoolWithTag(ConnectionInformation as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION as ULONG);
        }
        return core::ptr::null_mut();
    }
    (*ConnectionInformation).OptionsLength = 0 as LONG;
    return ConnectionInformation;
}
unsafe extern "system" fn WskBind(
    SocketParam: PWSK_SOCKET,
    LocalAddress: PSOCKADDR,
    Flags: ULONG,
    Irp: PIRP,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let Socket: PWSK_SOCKET_INTERNAL = CONTAINING_RECORD!(SocketParam, WSK_SOCKET_INTERNAL, Socket);

    let ta: PTRANSPORT_ADDRESS = TdiTransportAddressFromSocketAddress(LocalAddress);

    FUNCTION_TRACE;
    IoSetNextIrpStackLocation(Irp);

    'err_out: {
        if ta.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            break 'err_out;
        }
        if !(*Socket).LocalAddressHandle.is_null() {
            status = STATUS_INVALID_PARAMETER;
            break 'err_out;
        }

        status = TdiOpenAddressFile(
            &raw mut (*Socket).TdiName,
            ta,
            (AFD_SHARE_REUSE) as ULONG,
            &raw mut (*Socket).LocalAddressHandle,
            &raw mut (*Socket).LocalAddressFile,
        );

        ExFreePoolWithTag(ta as PVOID, TAG_NETIO);
    }
    // err_out:
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));

    return STATUS_PENDING;
}
unsafe extern "system" fn WskSendTo(
    SocketParam: PWSK_SOCKET,
    Buffer: PWSK_BUF,
    Flags: ULONG,
    RemoteAddress: PSOCKADDR,
    ControlInfoLength: ULONG,
    ControlInfo: PCMSGHDR,
    Irp: PIRP,
) -> NTSTATUS {
    let mut tdiIrp: PIRP = core::ptr::null_mut();
    let mut Socket: PWSK_SOCKET_INTERNAL = CONTAINING_RECORD!(SocketParam, WSK_SOCKET_INTERNAL, Socket);
    let mut TargetConnectionInfo: PTDI_CONNECTION_INFORMATION = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;
    let mut BufferData: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut NetioContext: PNETIO_CONTEXT = core::ptr::null_mut();

    FUNCTION_TRACE;

    IoSetNextIrpStackLocation(Irp);

    status = STATUS_INSUFFICIENT_RESOURCES;
    'err_out: {
        'err_out_free_nc: {
            'err_out_free_nc_and_tci: {
                NetioContext = ExAllocatePoolUninitialized(
                    NonPagedPool,
                    (core::mem::size_of::<NETIO_CONTEXT>() as SIZE_T) as _,
                    TAG_NETIO,
                ) as PNETIO_CONTEXT;
                if NetioContext.is_null() {
                    break 'err_out;
                }
                (*NetioContext).socket = Socket;
                (*NetioContext).UserIrp = Irp;

                TargetConnectionInfo = TdiConnectionInfoFromSocketAddress(RemoteAddress);
                if TargetConnectionInfo.is_null() {
                    break 'err_out_free_nc;
                }
                (*NetioContext).TargetConnectionInfo = TargetConnectionInfo;
                (*NetioContext).PeerAddrRet = core::ptr::null_mut();

                BufferData = MmGetSystemAddressForMdlSafe!((*Buffer).Mdl, NormalPagePriority);
                if BufferData.is_null() {
                    DPRINT1!("Error mapping MDL\n");
                    break 'err_out_free_nc_and_tci;
                }

                IoMarkIrpPending(Irp);
                ReferenceSocket(Socket);

                /* This will create a tdiIrp */
                status = TdiSendDatagram(
                    core::ptr::addr_of_mut!(tdiIrp),
                    (*Socket).LocalAddressFile,
                    (BufferData as *mut core::ffi::c_char).add((*Buffer).Offset as usize),
                    ((*Buffer).Length) as UINT,
                    TargetConnectionInfo,
                    Some(NetioComplete),
                    (NetioContext).cast(),
                );

                /* If allocating tdiIrp fails we get here.
                 * Call the IoCompletion of the application's Irp so this Irp
                 * gets freed.
                 */
                if !NT_SUCCESS!(status) {
                    DereferenceSocket(Socket);
                    break 'err_out_free_nc_and_tci; /* caller has to clean up mdl mapping */
                }
                return STATUS_PENDING;
            }
            ExFreePoolWithTag(TargetConnectionInfo as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);
        }
        ExFreePoolWithTag(NetioContext as PVOID, TAG_NETIO);
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));

    return status;
}
unsafe extern "system" fn WskReceiveFrom(
    SocketParam: PWSK_SOCKET,
    Buffer: PWSK_BUF,
    Flags: ULONG,
    RemoteAddress: PSOCKADDR,
    ControlLength: PULONG,
    ControlInfo: PCMSGHDR,
    ControlFlags: PULONG,
    Irp: PIRP,
) -> NTSTATUS {
    FUNCTION_TRACE;

    UNIMPLEMENTED!();
    if !Irp.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));
    }
    return STATUS_NOT_IMPLEMENTED;
}

unsafe extern "system" fn WskReleaseUdp(
    mut SocketParam: PWSK_SOCKET,
    mut DatagramIndication: PWSK_DATAGRAM_INDICATION,
) -> NTSTATUS {
    FUNCTION_TRACE;

    UNIMPLEMENTED!();

    return STATUS_NOT_IMPLEMENTED;
}
unsafe extern "system" fn WskReleaseTcp(
    mut SocketParam: PWSK_SOCKET,
    mut DataIndication: PWSK_DATA_INDICATION,
) -> NTSTATUS {
    FUNCTION_TRACE;

    UNIMPLEMENTED!();

    return STATUS_NOT_IMPLEMENTED;
}
unsafe extern "system" fn WskGetLocalAddress(
    mut SocketParam: PWSK_SOCKET,
    mut LocalAddress: PSOCKADDR,
    mut Irp: PIRP,
) -> NTSTATUS {
    FUNCTION_TRACE;

    UNIMPLEMENTED!();
    if !Irp.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));
    }
    return STATUS_NOT_IMPLEMENTED;
}
unsafe extern "system" fn WskGetRemoteAddress(
    SocketParam: PWSK_SOCKET,
    RemoteAddress: PSOCKADDR,
    Irp: PIRP,
) -> NTSTATUS {
    FUNCTION_TRACE;

    let Socket: PWSK_SOCKET_INTERNAL = (SocketParam as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(WSK_SOCKET_INTERNAL, Socket) as ULONG_PTR,
    ) as PWSK_SOCKET_INTERNAL;
    let mut Status: NTSTATUS = STATUS_INVALID_PARAMETER;

    IoSetNextIrpStackLocation(Irp);
    if !Socket.is_null() {
        memcpy(
            RemoteAddress as *mut ::core::ffi::c_void,
            &raw mut (*Socket).RemoteAddress as *const ::core::ffi::c_void,
            ::core::mem::size_of::<SOCKADDR>() as size_t,
        );
        Status = STATUS_SUCCESS;
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NETWORK_INCREMENT as CCHAR);

    return STATUS_PENDING;
}
unsafe extern "system" fn WskSocketConnect(
    Client: PWSK_CLIENT,
    SocketType: USHORT,
    Protocol: ULONG,
    LocalAddress: PSOCKADDR,
    RemoteAddress: PSOCKADDR,
    Flags: ULONG,
    SocketContext: PVOID,
    Dispatch: *const WSK_CLIENT_CONNECTION_DISPATCH,
    OwningProcess: PEPROCESS,
    OwningThread: PETHREAD,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Irp: PIRP,
) -> NTSTATUS {
    FUNCTION_TRACE;

    UNIMPLEMENTED!();
    if !Irp.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        IoCompleteRequest(Irp, IO_NETWORK_INCREMENT as CCHAR);
    }
    return STATUS_NOT_IMPLEMENTED;
}
unsafe extern "system" fn WskControlClient(
    Client: PWSK_CLIENT,
    ControlCode: ULONG,
    InputSize: SIZE_T,
    InputBuffer: PVOID,
    OutputSize: SIZE_T,
    OutputBuffer: PVOID,
    OutputSizeReturned: *mut SIZE_T,
    Irp: PIRP,
) -> NTSTATUS {
    FUNCTION_TRACE;

    UNIMPLEMENTED!();
    if !Irp.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));
    }
    return STATUS_NOT_IMPLEMENTED;
}
unsafe extern "system" fn WskConnect(
    SocketParam: PWSK_SOCKET,
    RemoteAddress: PSOCKADDR,
    Flags: ULONG,
    Irp: PIRP,
) -> NTSTATUS {
    let mut TargetConnectionInfo: PTDI_CONNECTION_INFORMATION = core::ptr::null_mut();
    let mut PeerAddrRet: PTDI_CONNECTION_INFORMATION = core::ptr::null_mut();
    let mut tdiIrp: PIRP = core::ptr::null_mut();
    let mut Socket: PWSK_SOCKET_INTERNAL =
        CONTAINING_RECORD!(SocketParam, WSK_SOCKET_INTERNAL, Socket);
    let mut status: NTSTATUS = 0;
    let mut status2: NTSTATUS = 0;
    let mut NetioContext: PNETIO_CONTEXT = core::ptr::null_mut();

    FUNCTION_TRACE;

    IoSetNextIrpStackLocation(Irp);

    status = STATUS_INVALID_PARAMETER;
    'err_out: {
        if (*Socket).LocalAddressHandle.is_null() {
            DPRINT1!("LocalAddressHandle is not set, need to bind your socket before connecting\n");
            break 'err_out;
        }

        status = STATUS_INSUFFICIENT_RESOURCES;
        'err_out_free_nc: {
            'err_out_free_nc_and_disassociate: {
                'err_out_free_nc_and_tci: {
                    NetioContext = ExAllocatePoolUninitialized(
                        NonPagedPool,
                        (core::mem::size_of::<NETIO_CONTEXT>() as SIZE_T) as _,
                        TAG_NETIO,
                    ) as PNETIO_CONTEXT;
                    if NetioContext.is_null() {
                        break 'err_out;
                    }
                    (*NetioContext).socket = Socket;
                    (*NetioContext).UserIrp = Irp;

                    tdiIrp = core::ptr::null_mut();

                    status = TdiAssociateAddressFile((*Socket).LocalAddressHandle, (*Socket).ConnectionFile);
                    if !NT_SUCCESS!(status) {
                        break 'err_out_free_nc;
                    }
                    (*Socket).ConnectionFileAssociated = TRUE;

                    TargetConnectionInfo = TdiConnectionInfoFromSocketAddress(RemoteAddress);
                    if TargetConnectionInfo.is_null() {
                        break 'err_out_free_nc_and_disassociate;
                    }
                    (*NetioContext).TargetConnectionInfo = TargetConnectionInfo;

                    PeerAddrRet = core::ptr::null_mut();
                    TdiBuildNullConnectionInfo(
                        core::ptr::addr_of_mut!(PeerAddrRet),
                        TDI_ADDRESS_TYPE_IP,
                    );
                    if PeerAddrRet.is_null() {
                        break 'err_out_free_nc_and_tci;
                    }
                    (*PeerAddrRet).OptionsLength = 0;
                    (*NetioContext).PeerAddrRet = PeerAddrRet;

                    IoMarkIrpPending(Irp);
                    ReferenceSocket(Socket);

                    status = TdiConnect(
                        core::ptr::addr_of_mut!(tdiIrp),
                        (*Socket).ConnectionFile,
                        TargetConnectionInfo,
                        PeerAddrRet,
                        Some(
                            NetioComplete,
                        ),
                        NetioContext as PVOID,
                    );

                    if !NT_SUCCESS!(status) {
                        /* If allocating tdiIrp fails we get here.
                         * Call the IoCompletion of the application's Irp so this Irp
                         * gets freed.
                         */
                        ExFreePoolWithTag(PeerAddrRet as PVOID, TAG_NETIO);
                        DereferenceSocket(Socket);
                        break 'err_out_free_nc_and_tci;
                    }
                    return STATUS_PENDING;
                }

                // err_out_free_nc_and_tci:
                ExFreePoolWithTag(TargetConnectionInfo as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);
            }

            // err_out_free_nc_and_disassociate:
            status2 = TdiDisassociateAddressFile((*Socket).ConnectionFile);
            if !NT_SUCCESS!(status2) {
                DPRINT1!("Warning: TdiDisassociateAddressFile returned status %08x\n", status);
            }
            (*Socket).ConnectionFileAssociated = FALSE;
        }

        // err_out_free_nc:
        ExFreePoolWithTag(NetioContext as PVOID, TAG_NETIO);
    }

    // err_out:
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));

    return status;
}
unsafe extern "system" fn WskStreamIo(
    mut SocketParam: PWSK_SOCKET,
    mut Buffer: PWSK_BUF,
    mut Flags: ULONG,
    mut Irp: PIRP,
    mut Direction: direction,
) -> NTSTATUS {
    let mut tdiIrp: PIRP = core::ptr::null_mut();
    let mut Socket: PWSK_SOCKET_INTERNAL = CONTAINING_RECORD!(SocketParam, WSK_SOCKET_INTERNAL, Socket);
    let mut status: NTSTATUS;
    let mut BufferData: *mut core::ffi::c_void;
    let mut NetioContext: PNETIO_CONTEXT;

    FUNCTION_TRACE;

    IoSetNextIrpStackLocation(Irp);
    status = STATUS_INSUFFICIENT_RESOURCES;

    NetioContext = ExAllocatePoolUninitialized(
        NonPagedPool,
        (core::mem::size_of::<NETIO_CONTEXT>() as SIZE_T) as _,
        TAG_NETIO as ULONG,
    ) as PNETIO_CONTEXT;

    'err_out: {
        if NetioContext.is_null() {
            break 'err_out;
        }

        'err_out_free_nc: {
            (*NetioContext).socket = Socket;
            (*NetioContext).UserIrp = Irp;

            (*NetioContext).TargetConnectionInfo = core::ptr::null_mut();
            (*NetioContext).PeerAddrRet = core::ptr::null_mut();

            BufferData = MmGetSystemAddressForMdlSafe!((*Buffer).Mdl, NormalPagePriority);
            if BufferData.is_null() {
                DPRINT1!("Error mapping MDL\n");
                break 'err_out_free_nc;
            }

            IoMarkIrpPending(Irp);
            ReferenceSocket(Socket);

            if Direction as ::core::ffi::c_uint == DIR_SEND as ::core::ffi::c_uint {
                /* This will create a tdiIrp */
                status = TdiSend(
                    core::ptr::addr_of_mut!(tdiIrp),
                    (*Socket).ConnectionFile,
                    0 as USHORT,
                    (BufferData as *mut ::core::ffi::c_char).offset((*Buffer).Offset as isize),
                    (*Buffer).Length as UINT,
                    Some(
                        NetioComplete,
                    ),
                    NetioContext as PVOID,
                );
            } else {
                status = TdiReceive(
                    core::ptr::addr_of_mut!(tdiIrp),
                    (*Socket).ConnectionFile,
                    0 as USHORT,
                    (BufferData as *mut ::core::ffi::c_char).offset((*Buffer).Offset as isize),
                    (*Buffer).Length as UINT,
                    Some(
                        NetioComplete,
                    ),
                    NetioContext as PVOID,
                );
            }

            /* If allocating tdiIrp fails we get here.
             * Call the IoCompletion of the application's Irp so this Irp
             * gets freed.
             */
            if !NT_SUCCESS!(status) {
                break 'err_out_free_nc; /* caller has to clean up mdl mapping */
            }
            return STATUS_PENDING;
        }

        // err_out_free_nc:
        ExFreePoolWithTag(NetioContext as PVOID, TAG_NETIO);
    }

    // err_out:
    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));

    return status;
}
unsafe extern "system" fn WskSend(
    mut SocketParam: PWSK_SOCKET,
    mut Buffer: PWSK_BUF,
    mut Flags: ULONG,
    mut Irp: PIRP,
) -> NTSTATUS {
    FUNCTION_TRACE;

    return WskStreamIo(SocketParam, Buffer, Flags, Irp, (DIR_SEND) as direction);
}
unsafe extern "system" fn WskReceive(
    SocketParam: PWSK_SOCKET,
    Buffer: PWSK_BUF,
    Flags: ULONG,
    Irp: PIRP,
) -> NTSTATUS {
    FUNCTION_TRACE;

    return WskStreamIo(SocketParam, Buffer, Flags, Irp, (DIR_RECEIVE) as direction);
}
unsafe extern "system" fn WskDisconnect(
    SocketParam: PWSK_SOCKET,
    Buffer: PWSK_BUF,
    Flags: ULONG,
    Irp: PIRP,
) -> NTSTATUS {
    let Socket: PWSK_SOCKET_INTERNAL = (SocketParam as *mut u8)
        .sub(core::mem::offset_of!(WSK_SOCKET_INTERNAL, Socket)) as PWSK_SOCKET_INTERNAL;

    FUNCTION_TRACE;

    SocketShutdown(Socket);

    IoSetNextIrpStackLocation(Irp);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));

    return STATUS_PENDING;
}
unsafe extern "system" fn WskSocket(
    Client: PWSK_CLIENT,
    AddressFamily: ADDRESS_FAMILY,
    SocketType: USHORT,
    Protocol: ULONG,
    Flags: ULONG,
    SocketContext: PVOID,
    Dispatch: *const core::ffi::c_void,
    OwningProcess: PEPROCESS,
    OwningThread: PETHREAD,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Socket: PWSK_SOCKET_INTERNAL = core::ptr::null_mut();
    let mut status: NTSTATUS = 0;

    FUNCTION_TRACE;

    IoSetNextIrpStackLocation(Irp);

    'err_out: {
        if AddressFamily != (AF_INET as u16) {
            DPRINT1!("Address family %d not supported\n", AddressFamily as core::ffi::c_uint);
            status = STATUS_NOT_SUPPORTED;
            break 'err_out;
        }
        match (SocketType) as u32 {
            SOCK_DGRAM => {
                if Protocol != IPPROTO_UDP {
                    DPRINT1!("SOCK_DGRAM only supports IPPROTO_UDP\n");
                    status = STATUS_INVALID_PARAMETER;
                    break 'err_out;
                }
                if Flags != WSK_FLAG_DATAGRAM_SOCKET {
                    DPRINT1!("SOCK_DGRAM flags must be WSK_FLAG_DATAGRAM_SOCKET\n");
                    status = STATUS_INVALID_PARAMETER;
                    break 'err_out;
                }
            }
            SOCK_STREAM => {
                if Protocol != IPPROTO_TCP {
                    DPRINT1!("SOCK_STREAM only supports IPPROTO_TCP\n");
                    status = STATUS_INVALID_PARAMETER;
                    break 'err_out;
                }
                if (Flags != WSK_FLAG_CONNECTION_SOCKET) && (Flags != WSK_FLAG_LISTEN_SOCKET) {
                    DPRINT1!("SOCK_STREAM flags must be either WSK_FLAG_CONNECTION_SOCKET or WSK_FLAG_LISTEN_SOCKET\n");
                    status = STATUS_INVALID_PARAMETER;
                    break 'err_out;
                }
            }
            SOCK_RAW => {
                DPRINT1!("SOCK_RAW not supported\n");
                status = STATUS_NOT_SUPPORTED;
                break 'err_out;
            }
            _ => {
                status = STATUS_INVALID_PARAMETER;
                break 'err_out;
            }
        }

        Socket = ExAllocatePoolZero(
            NonPagedPool,
            (core::mem::size_of::<WSK_SOCKET_INTERNAL>() as SIZE_T) as _,
            TAG_NETIO,
        ) as PWSK_SOCKET_INTERNAL;
        if Socket.is_null() {
            DPRINT1!("WskSocket: Out of memory\n");
            status = STATUS_INSUFFICIENT_RESOURCES;
            break 'err_out;
        }
        (*Socket).family = AddressFamily;
        (*Socket).type_ = SocketType;
        (*Socket).proto = Protocol;
        (*Socket).WskFlags = Flags;
        (*Socket).user_context = SocketContext;
        (*Socket).ListenDispatch = Dispatch as *const WSK_CLIENT_LISTEN_DISPATCH;
        (*Socket).RefCount = 1; /* DereferenceSocket() is in WskCloseSocket */

        match (SocketType) as u32 {
            SOCK_DGRAM => {
                (*Socket).Socket.Dispatch = core::ptr::addr_of_mut!(UdpDispatch) as *const core::ffi::c_void;
                RtlInitUnicodeString(core::ptr::addr_of_mut!((*Socket).TdiName), RUSTCONV_WSTR_62A39F1BCBB9.as_ptr());
            }
            SOCK_STREAM => {
                (*Socket).Socket.Dispatch = core::ptr::addr_of_mut!(TcpDispatch) as *const core::ffi::c_void;
                RtlInitUnicodeString(core::ptr::addr_of_mut!((*Socket).TdiName), RUSTCONV_WSTR_9112492AFDA5.as_ptr());

                if Flags != WSK_FLAG_LISTEN_SOCKET {
                    status = TdiOpenConnectionEndpointFile(
                        core::ptr::addr_of_mut!((*Socket).TdiName),
                        core::ptr::addr_of_mut!((*Socket).ConnectionHandle),
                        core::ptr::addr_of_mut!((*Socket).ConnectionFile),
                    );
                    if !NT_SUCCESS!(status) {
                        DPRINT1!("Could not open TDI handle, status is %x\n", status);
                        ExFreePoolWithTag((Socket).cast(), TAG_NETIO);
                        break 'err_out;
                    }
                } else {
                    KeInitializeEvent(
                        core::ptr::addr_of_mut!((*Socket).StartListenEvent),
                        _EVENT_TYPE_SynchronizationEvent,
                        FALSE,
                    );
                    core::ptr::write_volatile(
                        core::ptr::addr_of_mut!((*Socket).ListenThreadShouldRun),
                        TRUE,
                    );

                    status = PsCreateSystemThread(
                        core::ptr::addr_of_mut!((*Socket).ListenThreadHandle),
                        THREAD_ALL_ACCESS,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        Some(RequeueListenThread),
                        Socket as PVOID,
                    );
                    if !NT_SUCCESS!(status) {
                        DPRINT1!("Could not start listen thread, status is %x\n", status);
                        ExFreePoolWithTag((Socket).cast(), TAG_NETIO);
                        break 'err_out;
                    }
                    status = ObReferenceObjectByHandle(
                        (*Socket).ListenThreadHandle,
                        THREAD_ALL_ACCESS,
                        core::ptr::null_mut(),
                        (0i32) as KPROCESSOR_MODE,
                        core::ptr::addr_of_mut!((*Socket).ListenThread) as *mut PVOID,
                        core::ptr::null_mut(),
                    );
                    if !NT_SUCCESS!(status) {
                        DPRINT1!("Could not get a PKTHREAD object, status is %x\n", status);
                        ExFreePoolWithTag((Socket).cast(), TAG_NETIO);
                        break 'err_out;
                    }
                    ZwClose((*Socket).ListenThreadHandle);
                }

                if Flags == WSK_FLAG_LISTEN_SOCKET && (*Socket).ListenDispatch.is_null() {
                    DPRINT1!("Warning: no callbacks given for listen socket\n");
                }
            }
            _ => {
                DPRINT1!("Socket type not yet supported\n");
                status = STATUS_NOT_SUPPORTED;
                break 'err_out;
            }
        }

        (*Irp).IoStatus.Information = Socket as ULONG_PTR;
        status = STATUS_SUCCESS;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(Irp, (IO_NETWORK_INCREMENT as i8));

    return STATUS_PENDING;
}
#[no_mangle]
pub unsafe extern "system" fn WskRegister(client_npi: PWSK_CLIENT_NPI, reg: PWSK_REGISTRATION) -> NTSTATUS {
    FUNCTION_TRACE;

    (*reg).ReservedRegistrationState = 0;
    (*reg).ReservedRegistrationContext = core::ptr::null_mut();
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*reg).ReservedRegistrationLock));

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn WskCaptureProviderNPI(
    reg: PWSK_REGISTRATION,
    wait: ULONG,
    npi: PWSK_PROVIDER_NPI,
) -> NTSTATUS {
    let _ = FUNCTION_TRACE;

    (*npi).Client = core::ptr::null_mut();
    (*npi).Dispatch = core::ptr::addr_of_mut!(provider_dispatch) as _;

    return StartSocketDereferenceThread();
}
#[no_mangle]
pub unsafe extern "system" fn WskReleaseProviderNPI(reg: PWSK_REGISTRATION) {
    let _ = FUNCTION_TRACE;

    StopSocketDereferenceThread();
}
#[no_mangle]
pub unsafe extern "system" fn WskDeregister(reg: PWSK_REGISTRATION) {
    FUNCTION_TRACE;

    (*reg).ReservedRegistrationState = 0;
    (*reg).ReservedRegistrationContext = core::ptr::null_mut();
}
