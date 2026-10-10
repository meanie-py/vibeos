use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{FILE_FS_DEVICE_INFORMATION, _FSINFOCLASS_FileFsDeviceInformation, IRP, IoCancelIrp, IoCreateDevice, IoReleaseCancelSpinLock, KeInitializeMutex, PDEVICE_OBJECT, PDRIVER_OBJECT, PFILE_FS_DEVICE_INFORMATION, PFILE_FULL_EA_INFORMATION, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PLARGE_INTEGER, PIO_COMPLETION_ROUTINE, PUNICODE_STRING, ZwClose};
// FORCEINLINE WDK macros: use the linkable inline versions, not the (non-exported)
// bindgen `ddk` extern stubs.
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSetCancelRoutine};
use ntkrnl_sys::inline::ke::KeInitializeSpinLock;

// The Afd* cross-module dispatch fns are declared (stdcall) by the per-arch
// `types` substrate (imported via `use crate::types::*`) — re-declaring them here
// as cdecl `extern "C"` shadowed those with the wrong ABI and broke the i386 link.
// UNREFERENCED_PARAMETER is now a no-op generic in glue. Only the genuinely-external
// (C tdihelpers / lock helpers) fns stay declared here.
extern "C" {
    fn LostSocket(Irp: PIRP) -> NTSTATUS;
    fn SocketAcquireStateLock(FCB: PAFD_FCB) -> BOOLEAN;
    fn SocketStateUnlock(FCB: PAFD_FCB);
    fn KillSelectsForFCB(DeviceExt: PAFD_DEVICE_EXTENSION, FileObject: PFILE_OBJECT, Force: BOOLEAN);
    fn TdiDisassociateAddressFile(Object: PVOID) -> NTSTATUS;
    fn TdiDisconnect( Request: *mut PIRP, ConnectionObject: PVOID, Timeout: PLARGE_INTEGER, Flags: USHORT, CompletionRoutine: PIO_COMPLETION_ROUTINE, CompletionContext: PVOID, CallInfo: PTDI_CONNECTION_INFORMATION, ReturnInfo: PTDI_CONNECTION_INFORMATION, ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub static mut DebugTraceLevel: DWORD = MIN_TRACE;
pub static mut AfdReceiveWindowSize: ULONG = 0x2000;
pub static mut AfdSendWindowSize: ULONG = 0x2000;
#[no_mangle]
pub unsafe extern "C" fn OskitDumpBuffer(Data: PCHAR, Len: u32) {
    let mut i: u32 = 0;
    while i < Len {
        if i != 0 && (i & 0xf) == 0 {
            DbgPrint(b"\n\0".as_ptr() as PCHAR);
        }
        if (i & 0xf) == 0 {
            DbgPrint(b"%p: \0".as_ptr() as PCHAR, Data.offset(i as isize));
        }
        DbgPrint(b" %02x\0".as_ptr() as PCHAR, (*Data.offset(i as isize)) as u32 & 0xff);
        i += 1;
    }
    DbgPrint(b"\n\0".as_ptr() as PCHAR);
}

#[no_mangle]
pub unsafe extern "system" fn AfdGetDisconnectOptions(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {

    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let mut BufferSize = (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if (*FCB).DisconnectOptionsSize == 0 {
        AFD_DbgPrint!(MIN_TRACE, b"Invalid parameter\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
    }

    if (*FCB).FilledDisconnectOptions < BufferSize {
        BufferSize = (*FCB).FilledDisconnectOptions;
    }

    core::ptr::copy_nonoverlapping(
        (*FCB).DisconnectOptions as *const u8,
        (*Irp).UserBuffer as *mut u8,
        BufferSize as usize,
    );

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, BufferSize as UINT)
}

#[no_mangle]
pub unsafe extern "system" fn AfdSetDisconnectOptions(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let DisconnectOptions = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut());
    let DisconnectOptionsSize = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if DisconnectOptions.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if !(*FCB).DisconnectOptions.is_null() {
        ExFreePoolWithTag((*FCB).DisconnectOptions, TAG_AFD_DISCONNECT_OPTIONS);
        (*FCB).DisconnectOptions = core::ptr::null_mut();
        (*FCB).DisconnectOptionsSize = 0;
        (*FCB).FilledDisconnectOptions = 0;
    }

    (*FCB).DisconnectOptions = ExAllocatePoolWithTag(
        PagedPool,
        DisconnectOptionsSize as usize,
        TAG_AFD_DISCONNECT_OPTIONS,
    );

    if (*FCB).DisconnectOptions.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    RtlCopyMemory(
        (*FCB).DisconnectOptions,
        DisconnectOptions,
        DisconnectOptionsSize as usize,
    );

    (*FCB).DisconnectOptionsSize = DisconnectOptionsSize;

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}

#[no_mangle]
pub unsafe extern "system" fn AfdSetDisconnectOptionsSize(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let DisconnectOptionsSize = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut()) as *mut UINT;
    let BufferSize = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if DisconnectOptionsSize.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if BufferSize < core::mem::size_of::<UINT>() as u32 {
        AFD_DbgPrint!(MIN_TRACE, b"Buffer too small\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, 0);
    }

    if !(*FCB).DisconnectOptions.is_null() {
        ExFreePoolWithTag((*FCB).DisconnectOptions as *mut core::ffi::c_void, TAG_AFD_DISCONNECT_OPTIONS);
        (*FCB).DisconnectOptionsSize = 0;
        (*FCB).FilledDisconnectOptions = 0;
    }

    (*FCB).DisconnectOptions = ExAllocatePoolWithTag(
        PagedPool,
        *DisconnectOptionsSize as usize,
        TAG_AFD_DISCONNECT_OPTIONS,
    ) as *mut core::ffi::c_void;

    if (*FCB).DisconnectOptions.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    (*FCB).DisconnectOptionsSize = *DisconnectOptionsSize;

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}

#[no_mangle]
pub unsafe extern "system" fn AfdGetDisconnectData(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {

    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let mut BufferSize = (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if (*FCB).DisconnectDataSize == 0 {
        AFD_DbgPrint!(MIN_TRACE, b"Invalid parameter\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
    }

    if (*FCB).FilledDisconnectData < BufferSize {
        BufferSize = (*FCB).FilledDisconnectData;
    }

    core::ptr::copy_nonoverlapping(
        (*FCB).DisconnectData as *const u8,
        (*Irp).UserBuffer as *mut u8,
        BufferSize as usize,
    );

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, BufferSize as UINT)
}

#[no_mangle]
pub unsafe extern "system" fn AfdSetDisconnectData(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let DisconnectData = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut());
    let DisconnectDataSize = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if DisconnectData.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if !(*FCB).DisconnectData.is_null() {
        ExFreePoolWithTag((*FCB).DisconnectData, TAG_AFD_DISCONNECT_DATA);
        (*FCB).DisconnectData = core::ptr::null_mut();
        (*FCB).DisconnectDataSize = 0;
        (*FCB).FilledDisconnectData = 0;
    }

    (*FCB).DisconnectData = ExAllocatePoolWithTag(
        PagedPool,
        DisconnectDataSize as usize,
        TAG_AFD_DISCONNECT_DATA,
    );

    if (*FCB).DisconnectData.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    RtlCopyMemory(
        (*FCB).DisconnectData,
        DisconnectData,
        DisconnectDataSize as usize,
    );

    (*FCB).DisconnectDataSize = DisconnectDataSize;

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}

#[no_mangle]
pub unsafe extern "system" fn AfdSetDisconnectDataSize(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let DisconnectDataSize = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut()) as *mut UINT;
    let BufferSize = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if DisconnectDataSize.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if BufferSize < core::mem::size_of::<UINT>() as u32 {
        AFD_DbgPrint!(MIN_TRACE, b"Buffer too small\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, 0);
    }

    if !(*FCB).DisconnectData.is_null() {
        ExFreePoolWithTag((*FCB).DisconnectData as *mut core::ffi::c_void, TAG_AFD_DISCONNECT_DATA);
        (*FCB).DisconnectDataSize = 0;
        (*FCB).FilledDisconnectData = 0;
    }

    (*FCB).DisconnectData = ExAllocatePoolWithTag(
        PagedPool,
        *DisconnectDataSize as usize,
        TAG_AFD_DISCONNECT_DATA,
    ) as *mut core::ffi::c_void;

    if (*FCB).DisconnectData.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    (*FCB).DisconnectDataSize = *DisconnectDataSize;

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}

unsafe fn AfdGetTdiHandles(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let HandleFlags = LockRequest(Irp, IrpSp, 1u8, core::ptr::null_mut()) as PULONG;
    let HandleData = (*Irp).UserBuffer as PAFD_TDI_HANDLE_DATA;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if HandleFlags.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if (*IrpSp).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<ULONG>() as u32
        || (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength < core::mem::size_of::<AFD_TDI_HANDLE_DATA>() as u32
    {
        AFD_DbgPrint!(MIN_TRACE, b"Buffer too small\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, 0);
    }

    if ((*HandleFlags) & AFD_ADDRESS_HANDLE) != 0 {
        (*HandleData).TdiAddressHandle = (*FCB).AddressFile.Handle;
    }

    if ((*HandleFlags) & AFD_CONNECTION_HANDLE) != 0 {
        (*HandleData).TdiConnectionHandle = (*FCB).Connection.Handle;
    }

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}

unsafe fn AfdCreateSocket(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FCB: PAFD_FCB;
    let FileObject: PFILE_OBJECT;
    let DeviceExt: PAFD_DEVICE_EXTENSION;
    let EaInfo: PFILE_FULL_EA_INFORMATION;
    let mut ConnectInfo: PAFD_CREATE_PACKET = core::ptr::null_mut();
    let _i: UINT;
    let Status: NTSTATUS = STATUS_SUCCESS;

    AFD_DbgPrint!(
        MID_TRACE,
        b"AfdCreate(DeviceObject %p Irp %p)\n\0".as_ptr() as PCHAR,
        DeviceObject,
        Irp,
    );

    DeviceExt = (*DeviceObject).DeviceExtension as PAFD_DEVICE_EXTENSION;
    FileObject = (*IrpSp).FileObject;
    (*FileObject).Flags |= FO_NAMED_PIPE;

    (*Irp).IoStatus.Information = 0;

    EaInfo = (*Irp).AssociatedIrp.SystemBuffer as PFILE_FULL_EA_INFORMATION;

    if !EaInfo.is_null() {
        ConnectInfo = ((*EaInfo).EaName.as_ptr() as usize + (*EaInfo).EaNameLength as usize + 1)
            as PAFD_CREATE_PACKET;

        AFD_DbgPrint!(
            MID_TRACE,
            b"EaInfo: %p\n\0".as_ptr() as PCHAR,
            EaInfo,
        );
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"About to allocate the new FCB\n\0".as_ptr() as PCHAR,
    );

    FCB = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<AFD_FCB>() as SIZE_T,
        TAG_AFD_FCB,
    ) as PAFD_FCB;
    if FCB.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
        return STATUS_NO_MEMORY;
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"Initializing the new FCB @ %p (FileObject %p Flags %x)\n\0".as_ptr() as PCHAR,
        FCB,
        FileObject,
        if !ConnectInfo.is_null() {
            (*ConnectInfo).EndpointFlags
        } else {
            0
        },
    );

    RtlZeroMemory(FCB as *mut core::ffi::c_void, core::mem::size_of::<AFD_FCB>() as SIZE_T);

    (*FCB).Flags = if !ConnectInfo.is_null() {
        (*ConnectInfo).EndpointFlags
    } else {
        0
    };
    (*FCB).GroupID = if !ConnectInfo.is_null() {
        (*ConnectInfo).GroupID
    } else {
        0
    };
    (*FCB).GroupType = 0;
    (*FCB).SharedData.State = SOCKET_STATE_CREATED;
    (*FCB).FileObject = FileObject;
    (*FCB).DeviceExt = DeviceExt;
    (*FCB).AddressFile.Handle = INVALID_HANDLE_VALUE as HANDLE;
    (*FCB).Connection.Handle = INVALID_HANDLE_VALUE as HANDLE;
    (*FCB).Recv.Size = AfdReceiveWindowSize;
    (*FCB).Send.Size = AfdSendWindowSize;

    KeInitializeMutex(core::ptr::addr_of_mut!((*FCB).Mutex), 0);

    for i in 0..MAX_FUNCTIONS {
        InitializeListHead(core::ptr::addr_of_mut!((*FCB).PendingIrpList[i as usize]));
    }

    InitializeListHead(core::ptr::addr_of_mut!((*FCB).DatagramList));
    InitializeListHead(core::ptr::addr_of_mut!((*FCB).PendingConnections));

    AFD_DbgPrint!(MID_TRACE, b"%p: Checking command channel\n\0".as_ptr() as PCHAR, FCB);

    if !ConnectInfo.is_null() {
        (*FCB).TdiDeviceName.Length = (*ConnectInfo).SizeOfTransportName as u16;
        (*FCB).TdiDeviceName.MaximumLength = (*FCB).TdiDeviceName.Length;
        (*FCB).TdiDeviceName.Buffer = ExAllocatePoolWithTag(
            NonPagedPool,
            (*FCB).TdiDeviceName.Length as SIZE_T,
            TAG_AFD_TRANSPORT_ADDRESS,
        ) as PWSTR;

        if (*FCB).TdiDeviceName.Buffer.is_null() {
            ExFreePoolWithTag(FCB as *mut core::ffi::c_void, TAG_AFD_FCB);
            AFD_DbgPrint!(
                MID_TRACE,
                b"Could not copy target string\n\0".as_ptr() as PCHAR,
            );
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NO_MEMORY;
            IoCompleteRequest(Irp, IO_NETWORK_INCREMENT);
            return STATUS_NO_MEMORY;
        }

        RtlCopyMemory(
            (*FCB).TdiDeviceName.Buffer as *mut core::ffi::c_void,
            (*ConnectInfo).TransportName.as_ptr() as *const core::ffi::c_void,
            (*FCB).TdiDeviceName.Length as SIZE_T,
        );

        AFD_DbgPrint!(
            MID_TRACE,
            b"Success: %s %wZ\n\0".as_ptr() as PCHAR,
            (*EaInfo).EaName.as_ptr(),
            core::ptr::addr_of!((*FCB).TdiDeviceName),
        );
    } else {
        AFD_DbgPrint!(
            MID_TRACE,
            b"Success: Control connection\n\0".as_ptr() as PCHAR,
        );
    }

    (*FileObject).FsContext = FCB as *mut core::ffi::c_void;

    if ((*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS) != 0 {
        AFD_DbgPrint!(
            MID_TRACE,
            b"Packet oriented socket\n\0".as_ptr() as PCHAR,
        );

        (*FCB).PollState |= AFD_EVENT_SEND;
        (*FCB).PollStatus[FD_WRITE_BIT as usize] = STATUS_SUCCESS;
        PollReeval((*FCB).DeviceExt, (*FCB).FileObject);
    }

    if !NT_SUCCESS(Status) {
        if !(*FCB).TdiDeviceName.Buffer.is_null() {
            ExFreePoolWithTag(
                (*FCB).TdiDeviceName.Buffer as *mut core::ffi::c_void,
                TAG_AFD_TRANSPORT_ADDRESS,
            );
        }
        ExFreePoolWithTag(FCB as *mut core::ffi::c_void, TAG_AFD_FCB);
        (*FileObject).FsContext = core::ptr::null_mut();
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NETWORK_INCREMENT);

    Status
}

unsafe fn AfdCleanupSocket(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut Function: UINT;
    let mut CurrentIrp: PIRP;

    let _ = DeviceObject;

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    Function = 0;
    while Function < MAX_FUNCTIONS {
        CurrentEntry = (*FCB).PendingIrpList[Function as usize].Flink;
        while CurrentEntry != core::ptr::addr_of_mut!((*FCB).PendingIrpList[Function as usize]) {
            NextEntry = (*CurrentEntry).Flink;
            CurrentIrp = (CurrentEntry as *mut u8)
                .offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;

            IoCancelIrp(CurrentIrp);

            CurrentEntry = NextEntry;
        }
        Function += 1;
    }

    KillSelectsForFCB((*FCB).DeviceExt, FileObject, 0u8);

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}

unsafe fn AfdCloseSocket(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {

    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let mut i: UINT;
    let mut InFlightRequest: [PAFD_IN_FLIGHT_REQUEST; 5] = [core::ptr::null_mut(); 5];
    let mut Qelt: PAFD_TDI_OBJECT_QELT;
    let mut QeltEntry: PLIST_ENTRY;

    AFD_DbgPrint!(
        MID_TRACE,
        b"AfdClose(DeviceObject %p Irp %p)\n\0".as_ptr() as PCHAR,
        DeviceObject,
        Irp,
    );

    if SocketAcquireStateLock(FCB) == 0 {
        return STATUS_FILE_CLOSED;
    }

    (*FCB).SharedData.State = SOCKET_STATE_CLOSED;

    InFlightRequest[0] = core::ptr::addr_of_mut!((*FCB).ListenIrp);
    InFlightRequest[1] = core::ptr::addr_of_mut!((*FCB).ReceiveIrp);
    InFlightRequest[2] = core::ptr::addr_of_mut!((*FCB).SendIrp);
    InFlightRequest[3] = core::ptr::addr_of_mut!((*FCB).ConnectIrp);
    InFlightRequest[4] = core::ptr::addr_of_mut!((*FCB).DisconnectIrp);

    i = 0;
    while i < 5 {
        if !(*InFlightRequest[i as usize]).InFlightRequest.is_null() {
            AFD_DbgPrint!(
                MID_TRACE,
                b"Cancelling in flight irp %u (%p)\n\0".as_ptr() as PCHAR,
                i,
                (*InFlightRequest[i as usize]).InFlightRequest,
            );
            IoCancelIrp((*InFlightRequest[i as usize]).InFlightRequest);
        }
        i += 1;
    }

    KillSelectsForFCB((*FCB).DeviceExt, FileObject, 0);

    while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingConnections)) {
        QeltEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingConnections));
        Qelt = (QeltEntry as *mut u8).offset(-(core::mem::offset_of!(AFD_TDI_OBJECT_QELT, ListEntry) as isize)) as PAFD_TDI_OBJECT_QELT;

        TdiDisassociateAddressFile((*Qelt).Object.Object as PVOID);
        ObDereferenceObject((*Qelt).Object.Object as PVOID);
        ZwClose((*Qelt).Object.Handle);

        ExFreePoolWithTag(Qelt as PVOID, TAG_AFD_ACCEPT_QUEUE);
    }

    SocketStateUnlock(FCB);

    if !(*FCB).EventSelect.is_null() {
        ObDereferenceObject((*FCB).EventSelect as PVOID);
    }

    if !(*FCB).Context.is_null() {
        ExFreePoolWithTag((*FCB).Context, TAG_AFD_SOCKET_CONTEXT);
    }

    if !(*FCB).Recv.Window.is_null() {
        ExFreePoolWithTag((*FCB).Recv.Window as PVOID, TAG_AFD_DATA_BUFFER);
    }

    if !(*FCB).Send.Window.is_null() {
        ExFreePoolWithTag((*FCB).Send.Window as PVOID, TAG_AFD_DATA_BUFFER);
    }

    if !(*FCB).AddressFrom.is_null() {
        ExFreePoolWithTag((*FCB).AddressFrom as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);
    }

    if !(*FCB).ConnectCallInfo.is_null() {
        ExFreePoolWithTag((*FCB).ConnectCallInfo as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);
    }

    if !(*FCB).ConnectReturnInfo.is_null() {
        ExFreePoolWithTag((*FCB).ConnectReturnInfo as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);
    }

    if !(*FCB).ConnectData.is_null() {
        ExFreePoolWithTag((*FCB).ConnectData as PVOID, TAG_AFD_CONNECT_DATA);
    }

    if !(*FCB).DisconnectData.is_null() {
        ExFreePoolWithTag((*FCB).DisconnectData as PVOID, TAG_AFD_DISCONNECT_DATA);
    }

    if !(*FCB).ConnectOptions.is_null() {
        ExFreePoolWithTag((*FCB).ConnectOptions as PVOID, TAG_AFD_CONNECT_OPTIONS);
    }

    if !(*FCB).DisconnectOptions.is_null() {
        ExFreePoolWithTag((*FCB).DisconnectOptions as PVOID, TAG_AFD_DISCONNECT_OPTIONS);
    }

    if !(*FCB).LocalAddress.is_null() {
        ExFreePoolWithTag((*FCB).LocalAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
    }

    if !(*FCB).RemoteAddress.is_null() {
        ExFreePoolWithTag((*FCB).RemoteAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
    }

    if !(*FCB).OnConnectSendBuffer.is_null() {
        ExFreePoolWithTag((*FCB).OnConnectSendBuffer as PVOID, TAG_AFD_SUPER_CONNECT_BUFFER);
    }

    if !(*FCB).Connection.Object.is_null() {
        TdiDisassociateAddressFile((*FCB).Connection.Object as PVOID);
        ObDereferenceObject((*FCB).Connection.Object as PVOID);
    }

    if !(*FCB).AddressFile.Object.is_null() {
        ObDereferenceObject((*FCB).AddressFile.Object as PVOID);
    }

    if (*FCB).AddressFile.Handle != INVALID_HANDLE_VALUE as HANDLE {
        if ZwClose((*FCB).AddressFile.Handle) == STATUS_INVALID_HANDLE {
            DbgPrint(
                b"INVALID ADDRESS FILE HANDLE VALUE: %p %p\n\0".as_ptr() as PCHAR,
                (*FCB).AddressFile.Handle,
                (*FCB).AddressFile.Object,
            );
        }
    }

    if (*FCB).Connection.Handle != INVALID_HANDLE_VALUE as HANDLE {
        if ZwClose((*FCB).Connection.Handle) == STATUS_INVALID_HANDLE {
            DbgPrint(
                b"INVALID CONNECTION HANDLE VALUE: %p %p\n\0".as_ptr() as PCHAR,
                (*FCB).Connection.Handle,
                (*FCB).Connection.Object,
            );
        }
    }

    if !(*FCB).TdiDeviceName.Buffer.is_null() {
        ExFreePoolWithTag((*FCB).TdiDeviceName.Buffer as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
    }

    ExFreePoolWithTag(FCB as PVOID, TAG_AFD_FCB);

    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_NETWORK_INCREMENT as i8);

    AFD_DbgPrint!(MID_TRACE, b"Returning success.\n\0".as_ptr() as PCHAR);

    STATUS_SUCCESS
}
unsafe extern "system" fn DisconnectComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    disconnect_complete_impl(DeviceObject, Irp, Context)
}

unsafe fn disconnect_complete_impl(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    let FCB = Context as PAFD_FCB;
    let mut CurrentIrp: PIRP;
    let mut CurrentEntry: PLIST_ENTRY;

    if SocketAcquireStateLock(FCB) == 0 {
        return STATUS_FILE_CLOSED;
    }

    (*FCB).DisconnectIrp.InFlightRequest = core::ptr::null_mut();

    if NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status)
        && ((*FCB).DisconnectFlags & TDI_DISCONNECT_RELEASE as u16) != 0
    {
        (*FCB).FilledDisconnectData = MIN(
            (*FCB).DisconnectDataSize,
            (*(*FCB).ConnectReturnInfo).UserDataLength as u32,
        );
        if (*FCB).FilledDisconnectData != 0 {
            RtlCopyMemory(
                (*FCB).DisconnectData as *mut core::ffi::c_void,
                (*(*FCB).ConnectReturnInfo).UserData as *const core::ffi::c_void,
                (*FCB).FilledDisconnectData as usize,
            );
        }

        (*FCB).FilledDisconnectOptions = MIN(
            (*FCB).DisconnectOptionsSize,
            (*(*FCB).ConnectReturnInfo).OptionsLength as u32,
        );
        if (*FCB).FilledDisconnectOptions != 0 {
            RtlCopyMemory(
                (*FCB).DisconnectOptions as *mut core::ffi::c_void,
                (*(*FCB).ConnectReturnInfo).Options as *const core::ffi::c_void,
                (*FCB).FilledDisconnectOptions as usize,
            );
        }
    }

    (*FCB).DisconnectPending = 0u8;

    while !IsListEmpty(core::ptr::addr_of_mut!(
        (*FCB).PendingIrpList[FUNCTION_DISCONNECT as usize]
    ))
    {
        CurrentEntry = RemoveHeadList(core::ptr::addr_of_mut!(
            (*FCB).PendingIrpList[FUNCTION_DISCONNECT as usize]
        ));
        CurrentIrp = (CurrentEntry as *mut u8).sub(
            core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)
        ) as PIRP;
        (*CurrentIrp).IoStatus.__bindgen_anon_1.Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        (*CurrentIrp).IoStatus.Information = 0;
        UnlockRequest(CurrentIrp, IoGetCurrentIrpStackLocation(CurrentIrp));
        IoSetCancelRoutine(CurrentIrp, None);
        IoCompleteRequest(CurrentIrp, IO_NETWORK_INCREMENT);
    }

    if ((*FCB).DisconnectFlags & TDI_DISCONNECT_RELEASE as u16) == 0 {
        (*FCB).PollState |= AFD_EVENT_ABORT;
        (*FCB).PollStatus[FD_CLOSE_BIT as usize] = (*Irp).IoStatus.__bindgen_anon_1.Status;
        (*FCB).LastReceiveStatus = STATUS_FILE_CLOSED;
        PollReeval((*FCB).DeviceExt, (*FCB).FileObject);
    }

    SocketStateUnlock(FCB);

    (*Irp).IoStatus.__bindgen_anon_1.Status
}

unsafe fn DoDisconnect(FCB: PAFD_FCB) -> NTSTATUS {

    let Status: NTSTATUS;

    (*(*FCB).ConnectCallInfo).UserData = (*FCB).DisconnectData;
    (*(*FCB).ConnectCallInfo).UserDataLength = (*FCB).DisconnectDataSize as i32;
    (*(*FCB).ConnectCallInfo).Options = (*FCB).DisconnectOptions;
    (*(*FCB).ConnectCallInfo).OptionsLength = (*FCB).DisconnectOptionsSize as i32;

    if (*FCB).DisconnectIrp.InFlightRequest != core::ptr::null_mut() {
        return STATUS_PENDING;
    }

    Status = TdiDisconnect(
        core::ptr::addr_of_mut!((*FCB).DisconnectIrp.InFlightRequest),
        (*FCB).Connection.Object as PVOID,
        core::ptr::addr_of_mut!((*FCB).DisconnectTimeout),
        (*FCB).DisconnectFlags,
        Some(DisconnectComplete),
        FCB as PVOID,
        (*FCB).ConnectCallInfo,
        (*FCB).ConnectReturnInfo,
    );

    if Status != STATUS_PENDING {
        (*FCB).DisconnectPending = 0u8;
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn RetryDisconnectCompletion(FCB: PAFD_FCB) {
    if IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize]))
        && (*FCB).SendIrp.InFlightRequest.is_null()
        && (*FCB).DisconnectPending != 0
    {
        DoDisconnect(FCB);
    }
}

unsafe fn AfdDisconnect(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let DisReq: PAFD_DISCONNECT_INFO;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Flags: USHORT;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut CurrentIrp: PIRP;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    DisReq = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut()) as PAFD_DISCONNECT_INFO;
    if DisReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if ((*DisReq).DisconnectType & AFD_DISCONNECT_SEND) != 0
        && ((*DisReq).DisconnectType & AFD_DISCONNECT_RECV) == 0
    {
        Flags = TDI_DISCONNECT_RELEASE as u16;
    } else {
        (*FCB).TdiReceiveClosed = 1u8;

        if !(*FCB).ReceiveIrp.InFlightRequest.is_null() {
            IoCancelIrp((*FCB).ReceiveIrp.InFlightRequest);
        }

        (*FCB).Recv.Content = 0;
        (*FCB).Recv.BytesUsed = 0;

        (*FCB).LastReceiveStatus = STATUS_SUCCESS;

        (*FCB).PollState &= !AFD_EVENT_RECEIVE;

        if ((*DisReq).DisconnectType & AFD_DISCONNECT_RECV) != 0
            && ((*DisReq).DisconnectType & AFD_DISCONNECT_SEND) == 0
        {
            return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
        } else {
            Flags = TDI_DISCONNECT_ABORT as u16;
        }
    }

    if ((*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS) == 0 {
        if (*FCB).ConnectCallInfo.is_null() {
            AFD_DbgPrint!(MIN_TRACE, b"Invalid parameter\n\0".as_ptr() as PCHAR);
            return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
        }

        if (*FCB).DisconnectPending != 0 {
            if !(*FCB).DisconnectIrp.InFlightRequest.is_null() {
                IoCancelIrp((*FCB).DisconnectIrp.InFlightRequest);
            } else {
                while !IsListEmpty(core::ptr::addr_of_mut!(
                    (*FCB).PendingIrpList[FUNCTION_DISCONNECT as usize]
                ))
                {
                    CurrentEntry = RemoveHeadList(core::ptr::addr_of_mut!(
                        (*FCB).PendingIrpList[FUNCTION_DISCONNECT as usize]
                    ));
                    CurrentIrp = (CurrentEntry as *mut u8)
                        .offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize))
                        as PIRP;
                    (*CurrentIrp).IoStatus.Information = 0;
                    (*CurrentIrp).IoStatus.__bindgen_anon_1.Status = STATUS_CANCELLED;
                    UnlockRequest(CurrentIrp, IoGetCurrentIrpStackLocation(CurrentIrp));
                    IoSetCancelRoutine(CurrentIrp, None);
                    IoCompleteRequest(CurrentIrp, IO_NETWORK_INCREMENT);
                }
            }
        }

        (*FCB).DisconnectFlags = Flags;
        (*FCB).DisconnectTimeout = (*DisReq).Timeout;
        (*FCB).DisconnectPending = 1u8;
        (*FCB).SendClosed = 1u8;
        (*FCB).PollState &= !AFD_EVENT_SEND;

        Status = QueueUserModeIrp(FCB, Irp, FUNCTION_DISCONNECT);
        if Status == STATUS_PENDING {
            if (IsListEmpty(core::ptr::addr_of_mut!(
                (*FCB).PendingIrpList[FUNCTION_SEND as usize]
            ))
                && (*FCB).SendIrp.InFlightRequest.is_null())
                || ((*FCB).DisconnectFlags & TDI_DISCONNECT_ABORT as u16) != 0
            {
                Status = DoDisconnect(FCB);
            }

            if Status != STATUS_PENDING {
                RemoveEntryList(core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry));
            }
        }

        if Status == STATUS_PENDING {
            SocketStateUnlock(FCB);

            return Status;
        }
    } else {
        if (Flags & TDI_DISCONNECT_RELEASE as u16) == 0 {
            if (*FCB).RemoteAddress.is_null() {
                AFD_DbgPrint!(MIN_TRACE, b"Invalid parameter\n\0".as_ptr() as PCHAR);
                return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
            }

            ExFreePoolWithTag(
                (*FCB).RemoteAddress as *mut core::ffi::c_void,
                TAG_AFD_TRANSPORT_ADDRESS,
            );

            (*FCB).RemoteAddress = core::ptr::null_mut();
        }

        (*FCB).PollState &= !AFD_EVENT_SEND;
        (*FCB).SendClosed = 1u8;
    }

    UnlockAndMaybeComplete(FCB, Status, Irp, 0)
}
#[no_mangle]
pub unsafe extern "system" fn AfdQueryFsDeviceInfo(
    _DeviceObject: PDEVICE_OBJECT,
    Buffer: PFILE_FS_DEVICE_INFORMATION,
    Length: *mut u32,
) -> i32 {
    if *Length >= core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as u32 {
        (*Buffer).Characteristics = 0;
        (*Buffer).DeviceType = 0x00000012;

        *Length -= core::mem::size_of::<FILE_FS_DEVICE_INFORMATION>() as u32;

        return STATUS_SUCCESS;
    } else {
        return STATUS_INFO_LENGTH_MISMATCH;
    }
}

unsafe fn AfdQueryVolumeInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> i32 {
    let InfoClass: i32;
    let Buffer: *mut core::ffi::c_void;
    let mut Length: u32;
    let mut Status: i32 = STATUS_INVALID_INFO_CLASS;

    Buffer = (*Irp).AssociatedIrp.SystemBuffer;
    Length = (*IrpSp).Parameters.QueryVolume.Length;
    InfoClass = (*IrpSp).Parameters.QueryVolume.FsInformationClass as i32;

    match InfoClass {
        _ if InfoClass == _FSINFOCLASS_FileFsDeviceInformation as i32 => {
            Status = AfdQueryFsDeviceInfo(DeviceObject, Buffer as PFILE_FS_DEVICE_INFORMATION, &mut Length);
        }
        _ => {}
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = ((*IrpSp).Parameters.QueryVolume.Length - Length) as _;
    IoCompleteRequest(Irp, IO_NETWORK_INCREMENT as i8);

    Status
}

const MID_TRACE: u32 = 0x00000002; // afd debug.h: MID_TRACE 2; was 1
const MIN_TRACE: u32 = 0x00000001; // afd debug.h: MIN_TRACE 1; was 0
// AFD IOCTLs — _AFD_CONTROL_CODE(Op, Method) = (FILE_DEVICE_NETWORK<<12)|(Op<<2)|Method
// per drivers/afd/shared.h. The grind hardcoded many of these with WRONG (roughly
// sequential) values, so msafd's real codes were misrouted — e.g. SEND (0x1201F)
// landed on the RECV_DATAGRAM arm, DISCONNECT (0x1202B) on GET_INFO, and the dominant
// GET_INFO(0x1207B)/SET_CONTEXT(0x12047) polls fell through to default. Compute them
// from the canonical (op, method) so they match msafd exactly and can't drift again.
// METHOD_NEITHER = 3, METHOD_BUFFERED = 0.
const fn afd_ctl(op: u32, method: u32) -> u32 { (0x12u32 << 12) | (op << 2) | method }
const IOCTL_AFD_BIND: u32 = afd_ctl(0, 3);
const IOCTL_AFD_CONNECT: u32 = afd_ctl(1, 3);
const IOCTL_AFD_START_LISTEN: u32 = afd_ctl(2, 3);
const IOCTL_AFD_WAIT_FOR_LISTEN: u32 = afd_ctl(3, 0);
const IOCTL_AFD_ACCEPT: u32 = afd_ctl(4, 0);
const IOCTL_AFD_RECV: u32 = afd_ctl(5, 3);
const IOCTL_AFD_RECV_DATAGRAM: u32 = afd_ctl(6, 3);
const IOCTL_AFD_SEND: u32 = afd_ctl(7, 3);
const IOCTL_AFD_SEND_DATAGRAM: u32 = afd_ctl(8, 3);
const IOCTL_AFD_SELECT: u32 = afd_ctl(9, 0);
const IOCTL_AFD_DISCONNECT: u32 = afd_ctl(10, 3);
const IOCTL_AFD_GET_SOCK_NAME: u32 = afd_ctl(11, 3);
const IOCTL_AFD_GET_PEER_NAME: u32 = afd_ctl(12, 3);
const IOCTL_AFD_GET_TDI_HANDLES: u32 = afd_ctl(13, 3);
const IOCTL_AFD_SET_INFO: u32 = afd_ctl(14, 3);
const IOCTL_AFD_GET_CONTEXT_SIZE: u32 = afd_ctl(15, 3);
const IOCTL_AFD_GET_CONTEXT: u32 = afd_ctl(16, 3);
const IOCTL_AFD_SET_CONTEXT: u32 = afd_ctl(17, 3);
const IOCTL_AFD_SET_CONNECT_DATA: u32 = afd_ctl(18, 3);
const IOCTL_AFD_SET_CONNECT_OPTIONS: u32 = afd_ctl(19, 3);
const IOCTL_AFD_SET_DISCONNECT_DATA: u32 = afd_ctl(20, 3);
const IOCTL_AFD_SET_DISCONNECT_OPTIONS: u32 = afd_ctl(21, 3);
const IOCTL_AFD_GET_CONNECT_DATA: u32 = afd_ctl(22, 3);
const IOCTL_AFD_GET_CONNECT_OPTIONS: u32 = afd_ctl(23, 3);
const IOCTL_AFD_GET_DISCONNECT_DATA: u32 = afd_ctl(24, 3);
const IOCTL_AFD_GET_DISCONNECT_OPTIONS: u32 = afd_ctl(25, 3);
const IOCTL_AFD_SET_CONNECT_DATA_SIZE: u32 = afd_ctl(26, 3);
const IOCTL_AFD_SET_CONNECT_OPTIONS_SIZE: u32 = afd_ctl(27, 3);
const IOCTL_AFD_SET_DISCONNECT_DATA_SIZE: u32 = afd_ctl(28, 3);
const IOCTL_AFD_SET_DISCONNECT_OPTIONS_SIZE: u32 = afd_ctl(29, 3);
const IOCTL_AFD_GET_INFO: u32 = afd_ctl(30, 3);
const IOCTL_AFD_EVENT_SELECT: u32 = afd_ctl(33, 3);
const IOCTL_AFD_ENUM_NETWORK_EVENTS: u32 = afd_ctl(34, 3);
const IOCTL_AFD_DEFER_ACCEPT: u32 = afd_ctl(35, 3);
const IOCTL_AFD_GET_PENDING_CONNECT_DATA: u32 = afd_ctl(41, 3);
const IOCTL_AFD_VALIDATE_GROUP: u32 = afd_ctl(42, 3);
const IOCTL_AFD_SUPER_CONNECT: u32 = afd_ctl(49, 3);
unsafe extern "system" fn AfdDispatch(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    let mut Status: NTSTATUS = STATUS_NOT_IMPLEMENTED;
    let _FileObject = (*IrpSp).FileObject;

    AFD_DbgPrint!(MID_TRACE, b"AfdDispatch: %u\n\0".as_ptr() as *const i8, (*IrpSp).MajorFunction);
    if (*IrpSp).MajorFunction != IRP_MJ_CREATE {
        AFD_DbgPrint!(MID_TRACE, b"FO %p, IrpSp->FO %p\n\0".as_ptr() as *const i8, FileObject, (*IrpSp).FileObject);
    }

    (*Irp).IoStatus.Information = 0;

    match (*IrpSp).MajorFunction {
        IRP_MJ_CREATE => {
            return AfdCreateSocket(DeviceObject, Irp, IrpSp);
        }

        IRP_MJ_CLOSE => {
            return AfdCloseSocket(DeviceObject, Irp, IrpSp);
        }

        IRP_MJ_CLEANUP => {
            return AfdCleanupSocket(DeviceObject, Irp, IrpSp);
        }

        IRP_MJ_WRITE => {
            return AfdConnectedSocketWriteData(DeviceObject, Irp, IrpSp, 1u8);
        }

        IRP_MJ_READ => {
            return AfdConnectedSocketReadData(DeviceObject, Irp, IrpSp, 1u8);
        }

        0x0A => {
            return AfdQueryVolumeInformation(DeviceObject, Irp, IrpSp);
        }

        IRP_MJ_DEVICE_CONTROL => {
            match (*IrpSp).Parameters.DeviceIoControl.IoControlCode {
                IOCTL_AFD_BIND => {
                    return AfdBindSocket(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_CONNECT => {
                    return AfdStreamSocketConnect(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SUPER_CONNECT => {
                    return AfdStreamSocketSuperConnect(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_START_LISTEN => {
                    return AfdListenSocket(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_RECV => {
                    return AfdConnectedSocketReadData(DeviceObject, Irp, IrpSp, 0u8);
                }

                IOCTL_AFD_SELECT => {
                    return AfdSelect(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_EVENT_SELECT => {
                    return AfdEventSelect(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_ENUM_NETWORK_EVENTS => {
                    return AfdEnumEvents(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_RECV_DATAGRAM => {
                    return AfdPacketSocketReadData(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SEND => {
                    return AfdConnectedSocketWriteData(DeviceObject, Irp, IrpSp, 0u8);
                }

                IOCTL_AFD_SEND_DATAGRAM => {
                    return AfdPacketSocketWriteData(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_GET_INFO => {
                    return AfdGetInfo(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SET_INFO => {
                    return AfdSetInfo(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_GET_CONTEXT_SIZE => {
                    return AfdGetContextSize(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_GET_CONTEXT => {
                    return AfdGetContext(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SET_CONTEXT => {
                    return AfdSetContext(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_WAIT_FOR_LISTEN => {
                    return AfdWaitForListen(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_ACCEPT => {
                    return AfdAccept(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_DISCONNECT => {
                    return AfdDisconnect(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_GET_SOCK_NAME => {
                    return AfdGetSockName(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_GET_PEER_NAME => {
                    return AfdGetPeerName(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_GET_CONNECT_DATA => {
                    return AfdGetConnectData(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SET_CONNECT_DATA => {
                    return AfdSetConnectData(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SET_DISCONNECT_DATA => {
                    return AfdSetDisconnectData(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_GET_DISCONNECT_DATA => {
                    return AfdGetDisconnectData(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SET_CONNECT_DATA_SIZE => {
                    return AfdSetConnectDataSize(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SET_DISCONNECT_DATA_SIZE => {
                    return AfdSetDisconnectDataSize(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SET_CONNECT_OPTIONS => {
                    return AfdSetConnectOptions(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SET_DISCONNECT_OPTIONS => {
                    return AfdSetDisconnectOptions(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_GET_CONNECT_OPTIONS => {
                    return AfdGetConnectOptions(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_GET_DISCONNECT_OPTIONS => {
                    return AfdGetDisconnectOptions(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SET_CONNECT_OPTIONS_SIZE => {
                    return AfdSetConnectOptionsSize(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_SET_DISCONNECT_OPTIONS_SIZE => {
                    return AfdSetDisconnectOptionsSize(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_GET_TDI_HANDLES => {
                    return AfdGetTdiHandles(DeviceObject, Irp, IrpSp);
                }

                IOCTL_AFD_DEFER_ACCEPT => {
                    DbgPrint(b"IOCTL_AFD_DEFER_ACCEPT is unimplemented!()\n\0".as_ptr() as *const i8);
                }

                IOCTL_AFD_GET_PENDING_CONNECT_DATA => {
                    DbgPrint(b"IOCTL_AFD_GET_PENDING_CONNECT_DATA is unimplemented!()\n\0".as_ptr() as *const i8);
                }

                IOCTL_AFD_VALIDATE_GROUP => {
                    DbgPrint(b"IOCTL_AFD_VALIDATE_GROUP is unimplemented!()\n\0".as_ptr() as *const i8);
                }

                _ => {
                    Status = STATUS_NOT_SUPPORTED;
                    DbgPrint(b"Unknown IOCTL (0x%x)\n\0".as_ptr() as *const i8, (*IrpSp).Parameters.DeviceIoControl.IoControlCode);
                }
            }
        }

        _ => {
            Status = STATUS_NOT_IMPLEMENTED;
            AFD_DbgPrint!(MIN_TRACE, b"Irp: Unknown Major code was %x\n\0".as_ptr() as *const i8, (*IrpSp).MajorFunction);
        }
    }

    AFD_DbgPrint!(MID_TRACE, b"Returning %x\n\0".as_ptr() as *const i8, Status);
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn CheckUnlockExtraBuffers(FCB: PAFD_FCB, IrpSp: PIO_STACK_LOCATION) -> BOOLEAN {
    if (*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS != 0 {
        if (*IrpSp).MajorFunction == IRP_MJ_READ || (*IrpSp).MajorFunction == IRP_MJ_WRITE {
            return 0u8;
        } else if (*IrpSp).MajorFunction == IRP_MJ_DEVICE_CONTROL {
            if (*IrpSp).Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_RECV_DATAGRAM {
                return 1u8;
            } else if (*IrpSp).Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_RECV {
                return 0u8;
            } else if (*IrpSp).Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_SEND ||
                      (*IrpSp).Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_SEND_DATAGRAM {
                return 0u8;
            } else {
                ntkrnl_sys::ASSERT!(false);
                return 0u8;
            }
        } else {
            ntkrnl_sys::ASSERT!(false);
            return 0u8;
        }
    } else {
        return 0u8;
    }
}
#[no_mangle]
pub unsafe extern "C" fn CleanupPendingIrp(FCB: PAFD_FCB, Irp: PIRP, IrpSp: PIO_STACK_LOCATION, Poll: PAFD_ACTIVE_POLL) {
    let RecvReq: PAFD_RECV_INFO;
    let SendReq: PAFD_SEND_INFO;
    let PollReq: PAFD_POLL_INFO;

    if (*IrpSp).MajorFunction == IRP_MJ_READ as u8 {
        RecvReq = GetLockedData(Irp, IrpSp) as PAFD_RECV_INFO;
        UnlockBuffers((*RecvReq).BufferArray, (*RecvReq).BufferCount, CheckUnlockExtraBuffers(FCB, IrpSp) as i32);
    } else if (*IrpSp).MajorFunction == IRP_MJ_WRITE as u8 {
        SendReq = GetLockedData(Irp, IrpSp) as PAFD_SEND_INFO;
        UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, CheckUnlockExtraBuffers(FCB, IrpSp) as i32);
    } else {
        ntkrnl_sys::ASSERT!((*IrpSp).MajorFunction == IRP_MJ_DEVICE_CONTROL as u8);

        if (*IrpSp).Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_RECV {
            RecvReq = GetLockedData(Irp, IrpSp) as PAFD_RECV_INFO;
            UnlockBuffers((*RecvReq).BufferArray, (*RecvReq).BufferCount, CheckUnlockExtraBuffers(FCB, IrpSp) as i32);
        } else if (*IrpSp).Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_SEND {
            SendReq = GetLockedData(Irp, IrpSp) as PAFD_SEND_INFO;
            UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, CheckUnlockExtraBuffers(FCB, IrpSp) as i32);
        } else if (*IrpSp).Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_SELECT {
            ntkrnl_sys::ASSERT!(!Poll.is_null());

            PollReq = (*Irp).AssociatedIrp.SystemBuffer as PAFD_POLL_INFO;
            ZeroEvents(core::ptr::addr_of_mut!((*PollReq).Handles) as PAFD_HANDLE, (*PollReq).HandleCount);
            SignalSocket(Poll, core::ptr::null_mut(), PollReq, STATUS_CANCELLED);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn AfdCancelHandler(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let IrpSp = IoGetCurrentIrpStackLocation(Irp);
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let Function: ULONG;
    let IoctlCode: ULONG;
    let mut CurrentIrp: PIRP;
    let mut CurrentEntry: PLIST_ENTRY;
    let DeviceExt = (*DeviceObject).DeviceExtension as PAFD_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;
    let mut Poll: PAFD_ACTIVE_POLL;

    IoReleaseCancelSpinLock((*Irp).CancelIrql);

    if SocketAcquireStateLock(FCB) == 0 {
        return;
    }

    match (*IrpSp).MajorFunction {
        IRP_MJ_DEVICE_CONTROL => {
            IoctlCode = (*IrpSp).Parameters.DeviceIoControl.IoControlCode;
        }

        IRP_MJ_READ => {
            IoctlCode = IOCTL_AFD_RECV;
        }

        IRP_MJ_WRITE => {
            IoctlCode = IOCTL_AFD_SEND;
        }

        _ => {
            SocketStateUnlock(FCB);
            return;
        }
    }

    match IoctlCode {
        IOCTL_AFD_RECV | IOCTL_AFD_RECV_DATAGRAM => {
            Function = FUNCTION_RECV;
        }

        IOCTL_AFD_SEND | IOCTL_AFD_SEND_DATAGRAM => {
            Function = FUNCTION_SEND;
        }

        IOCTL_AFD_CONNECT => {
            Function = FUNCTION_CONNECT;
        }

        IOCTL_AFD_SUPER_CONNECT => {
            Function = FUNCTION_CONNECTEX;
        }

        IOCTL_AFD_WAIT_FOR_LISTEN => {
            Function = FUNCTION_PREACCEPT;
        }

        IOCTL_AFD_SELECT => {
            KeAcquireSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize, core::ptr::addr_of_mut!(OldIrql));

            CurrentEntry = (*DeviceExt).Polls.Flink;
            while CurrentEntry != core::ptr::addr_of_mut!((*DeviceExt).Polls) {
                Poll = (CurrentEntry as *mut u8).offset(-(core::mem::offset_of!(_AFD_ACTIVE_POLL, ListEntry) as isize)) as PAFD_ACTIVE_POLL;

                if Irp == (*Poll).Irp {
                    CleanupPendingIrp(FCB, Irp, IrpSp, Poll);
                    KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize, OldIrql);
                    SocketStateUnlock(FCB);
                    return;
                } else {
                    CurrentEntry = (*CurrentEntry).Flink;
                }
            }

            KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut usize, OldIrql);

            SocketStateUnlock(FCB);

            DbgPrint(b"WARNING!!! IRP cancellation race could lead to a process hang! (IOCTL_AFD_SELECT)\n\0".as_ptr() as PCHAR);
            return;
        }

        IOCTL_AFD_DISCONNECT => {
            Function = FUNCTION_DISCONNECT;
        }

        _ => {
            UnlockAndMaybeComplete(FCB, STATUS_CANCELLED, Irp, 0);
            return;
        }
    }

    CurrentEntry = (*FCB).PendingIrpList[Function as usize].Flink;
    while CurrentEntry != core::ptr::addr_of_mut!((*FCB).PendingIrpList[Function as usize]) {
        CurrentIrp = (CurrentEntry as *mut u8).offset(-(core::mem::offset_of!(_IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;

        if CurrentIrp == Irp {
            RemoveEntryList(CurrentEntry);
            CleanupPendingIrp(FCB, Irp, IrpSp, core::ptr::null_mut());
            UnlockAndMaybeComplete(FCB, STATUS_CANCELLED, Irp, 0);
            return;
        } else {
            CurrentEntry = (*CurrentEntry).Flink;
        }
    }

    SocketStateUnlock(FCB);

    DbgPrint(b"WARNING!!! IRP cancellation race could lead to a process hang! (Function: %u)\n\0".as_ptr() as PCHAR, Function);
}

unsafe extern "system" fn AfdUnload(DriverObject: PDRIVER_OBJECT) {
    let _ = DriverObject;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    // RTL_CONSTANT_STRING(L"\\Device\\Afd") — a UTF-16 (wide) device name. The grind
    // emitted an ASCII byte-string reinterpreted as *mut u16 (so each pair of ASCII
    // bytes became one garbage wide char) with doubled Length/MaximumLength. That
    // registered a mis-named device, so msafd's NtCreateFile on \Device\Afd returned
    // STATUS_OBJECT_PATH_NOT_FOUND. Use a real UTF-16 literal: 11 chars -> Length 22,
    // MaximumLength 24 (incl the trailing NUL). IoCreateDevice copies the name, so a
    // read-only static backing buffer is fine.
    static AFD_DEVICE_NAME: [u16; 12] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
        b'e' as u16, b'\\' as u16, b'A' as u16, b'f' as u16, b'd' as u16, 0,
    ];
    let mut wstrDeviceName = _UNICODE_STRING {
        Length: 22,
        MaximumLength: 24,
        Buffer: AFD_DEVICE_NAME.as_ptr() as *mut u16,
    };
    let DeviceExt: PAFD_DEVICE_EXTENSION;
    let Status: NTSTATUS;

    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = Some(core::mem::transmute(AfdDispatch as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = Some(core::mem::transmute(AfdDispatch as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP as usize] = Some(core::mem::transmute(AfdDispatch as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] = Some(core::mem::transmute(AfdDispatch as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] = Some(core::mem::transmute(AfdDispatch as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(core::mem::transmute(AfdDispatch as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION as usize] = Some(core::mem::transmute(AfdDispatch as *const ()));
    (*DriverObject).DriverUnload = Some(core::mem::transmute(AfdUnload as *const ()));

    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<AFD_DEVICE_EXTENSION>() as u32,
        &mut wstrDeviceName,
        FILE_DEVICE_NAMED_PIPE,
        0,
        0u8,
        &mut DeviceObject,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    DeviceExt = (*DeviceObject).DeviceExtension as PAFD_DEVICE_EXTENSION;
    // inline KeInitializeSpinLock takes *mut ULONG_PTR; the driver-types Lock field
    // is a KSPIN_LOCK (u32 on i386, usize on 64-bit) — cast to the param's pointer type.
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExt).Lock) as *mut _);
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceExt).Polls));

    AFD_DbgPrint!(
        MID_TRACE,
        b"Device created: object %p ext %p\n\0".as_ptr() as PCHAR,
        DeviceObject,
        DeviceExt,
    );

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/net/afd/afd/main.c
//  * PURPOSE:          Ancillary functions driver
//  * PROGRAMMER:       Art Yerkes (ayerkes@speakeasy.net)
//  * UPDATE HISTORY:
//  * 20040630 Created
//  *
//  * Suggestions: Uniform naming (AfdXxx)
//  */
//
// /* INCLUDES */
//
// #include "afd.h"
//
//
// /* See debug.h for debug/trace constants */
// //DWORD DebugTraceLevel = DEBUG_ULTRA;
// DWORD DebugTraceLevel = MIN_TRACE;
//
//
// /* FIXME: should depend on SystemSize */
// ULONG AfdReceiveWindowSize = 0x2000;
// ULONG AfdSendWindowSize = 0x2000;
//
// void OskitDumpBuffer( PCHAR Data, UINT Len ) {
//     unsigned int i;
//
//     for( i = 0; i < Len; i++ ) {
//         if( i && !(i & 0xf) ) DbgPrint( "\n" );
//         if( !(i & 0xf) ) DbgPrint( "%p: ", (Data + i) );
//         DbgPrint( " %02x", Data[i] & 0xff );
//     }
//     DbgPrint("\n");
// }
//
// /* FUNCTIONS */
//
// NTSTATUS NTAPI
// DriverEntry(PDRIVER_OBJECT DriverObject, PUNICODE_STRING RegistryPath);
//
// NTSTATUS NTAPI
// AfdGetDisconnectOptions(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                         PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     UINT BufferSize = IrpSp->Parameters.DeviceIoControl.OutputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (FCB->DisconnectOptionsSize == 0)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Invalid parameter\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
//     }
//
//     ASSERT(FCB->DisconnectOptions);
//
//     if (FCB->FilledDisconnectOptions < BufferSize) BufferSize = FCB->FilledDisconnectOptions;
//
//     RtlCopyMemory(Irp->UserBuffer,
//                   FCB->DisconnectOptions,
//                   BufferSize);
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, BufferSize);
// }
//
// NTSTATUS
// NTAPI
// AfdSetDisconnectOptions(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                         PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PVOID DisconnectOptions = LockRequest(Irp, IrpSp, FALSE, NULL);
//     UINT DisconnectOptionsSize = IrpSp->Parameters.DeviceIoControl.InputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!DisconnectOptions)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if (FCB->DisconnectOptions)
//     {
//         ExFreePoolWithTag(FCB->DisconnectOptions, TAG_AFD_DISCONNECT_OPTIONS);
//         FCB->DisconnectOptions = NULL;
//         FCB->DisconnectOptionsSize = 0;
//         FCB->FilledDisconnectOptions = 0;
//     }
//
//     FCB->DisconnectOptions = ExAllocatePoolWithTag(PagedPool,
//                                                    DisconnectOptionsSize,
//                                                    TAG_AFD_DISCONNECT_OPTIONS);
//
//     if (!FCB->DisconnectOptions)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     RtlCopyMemory(FCB->DisconnectOptions,
//                   DisconnectOptions,
//                   DisconnectOptionsSize);
//
//     FCB->DisconnectOptionsSize = DisconnectOptionsSize;
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
// }
//
// NTSTATUS
// NTAPI
// AfdSetDisconnectOptionsSize(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                       PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PUINT DisconnectOptionsSize = LockRequest(Irp, IrpSp, FALSE, NULL);
//     UINT BufferSize = IrpSp->Parameters.DeviceIoControl.InputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!DisconnectOptionsSize)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if (BufferSize < sizeof(UINT))
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Buffer too small\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, 0);
//     }
//
//     if (FCB->DisconnectOptions)
//     {
//         ExFreePoolWithTag(FCB->DisconnectOptions, TAG_AFD_DISCONNECT_OPTIONS);
//         FCB->DisconnectOptionsSize = 0;
//         FCB->FilledDisconnectOptions = 0;
//     }
//
//     FCB->DisconnectOptions = ExAllocatePoolWithTag(PagedPool,
//                                                    *DisconnectOptionsSize,
//                                                    TAG_AFD_DISCONNECT_OPTIONS);
//
//     if (!FCB->DisconnectOptions) return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     FCB->DisconnectOptionsSize = *DisconnectOptionsSize;
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
// }
//
// NTSTATUS NTAPI
// AfdGetDisconnectData(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                   PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     UINT BufferSize = IrpSp->Parameters.DeviceIoControl.OutputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (FCB->DisconnectDataSize == 0)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Invalid parameter\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
//     }
//
//     ASSERT(FCB->DisconnectData);
//
//     if (FCB->FilledDisconnectData < BufferSize)
//         BufferSize = FCB->FilledDisconnectData;
//
//     RtlCopyMemory(Irp->UserBuffer,
//                   FCB->DisconnectData,
//                   BufferSize);
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, BufferSize);
// }
//
// NTSTATUS
// NTAPI
// AfdSetDisconnectData(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                      PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PVOID DisconnectData = LockRequest(Irp, IrpSp, FALSE, NULL);
//     UINT DisconnectDataSize = IrpSp->Parameters.DeviceIoControl.InputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!DisconnectData)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if (FCB->DisconnectData)
//     {
//         ExFreePoolWithTag(FCB->DisconnectData, TAG_AFD_DISCONNECT_DATA);
//         FCB->DisconnectData = NULL;
//         FCB->DisconnectDataSize = 0;
//         FCB->FilledDisconnectData = 0;
//     }
//
//     FCB->DisconnectData = ExAllocatePoolWithTag(PagedPool,
//                                                 DisconnectDataSize,
//                                                 TAG_AFD_DISCONNECT_DATA);
//
//     if (!FCB->DisconnectData)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     RtlCopyMemory(FCB->DisconnectData,
//                   DisconnectData,
//                   DisconnectDataSize);
//
//     FCB->DisconnectDataSize = DisconnectDataSize;
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
// }
//
// NTSTATUS
// NTAPI
// AfdSetDisconnectDataSize(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                       PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PUINT DisconnectDataSize = LockRequest(Irp, IrpSp, FALSE, NULL);
//     UINT BufferSize = IrpSp->Parameters.DeviceIoControl.InputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!DisconnectDataSize)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if (BufferSize < sizeof(UINT))
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Buffer too small\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, 0);
//     }
//
//     if (FCB->DisconnectData)
//     {
//         ExFreePoolWithTag(FCB->DisconnectData, TAG_AFD_DISCONNECT_DATA);
//         FCB->DisconnectDataSize = 0;
//         FCB->FilledDisconnectData = 0;
//     }
//
//     FCB->DisconnectData = ExAllocatePoolWithTag(PagedPool,
//                                                 *DisconnectDataSize,
//                                                 TAG_AFD_DISCONNECT_DATA);
//
//     if (!FCB->DisconnectData)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     FCB->DisconnectDataSize = *DisconnectDataSize;
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
// }
//
// static NTSTATUS NTAPI
// AfdGetTdiHandles(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                 PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PULONG HandleFlags = LockRequest(Irp, IrpSp, TRUE, NULL);
//     PAFD_TDI_HANDLE_DATA HandleData = Irp->UserBuffer;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!HandleFlags)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if (IrpSp->Parameters.DeviceIoControl.InputBufferLength < sizeof(ULONG) ||
//         IrpSp->Parameters.DeviceIoControl.OutputBufferLength < sizeof(*HandleData))
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Buffer too small\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, 0);
//     }
//
//     if ((*HandleFlags) & AFD_ADDRESS_HANDLE)
//         HandleData->TdiAddressHandle = FCB->AddressFile.Handle;
//
//     if ((*HandleFlags) & AFD_CONNECTION_HANDLE)
//         HandleData->TdiConnectionHandle = FCB->Connection.Handle;
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
// }
//
// static NTSTATUS NTAPI
// AfdCreateSocket(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                 PIO_STACK_LOCATION IrpSp) {
//     PAFD_FCB FCB;
//     PFILE_OBJECT FileObject;
//     PAFD_DEVICE_EXTENSION DeviceExt;
//     PFILE_FULL_EA_INFORMATION EaInfo;
//     PAFD_CREATE_PACKET ConnectInfo = NULL;
//     //ULONG EaLength;
//     PWCHAR EaInfoValue = NULL;
//     //UINT Disposition;
//     UINT i;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     AFD_DbgPrint!(MID_TRACE,("AfdCreate(DeviceObject %p Irp %p)\n",
//                             DeviceObject, Irp));
//
//     DeviceExt = DeviceObject->DeviceExtension;
//     FileObject = IrpSp->FileObject;
//     FileObject->Flags |= FO_NAMED_PIPE;
//     //Disposition = (IrpSp->Parameters.Create.Options >> 24) & 0xff;
//
//     Irp->IoStatus.Information = 0;
//
//     EaInfo = Irp->AssociatedIrp.SystemBuffer;
//
//     if( EaInfo ) {
//         ConnectInfo = (PAFD_CREATE_PACKET)(EaInfo->EaName + EaInfo->EaNameLength + 1);
//         EaInfoValue = (PWCHAR)(((PCHAR)ConnectInfo) + sizeof(AFD_CREATE_PACKET));
//
//         //EaLength = sizeof(FILE_FULL_EA_INFORMATION) + EaInfo->EaNameLength + EaInfo->EaValueLength;
//
//         AFD_DbgPrint!(MID_TRACE,("EaInfo: %p, EaInfoValue: %p\n",
//                                 EaInfo, EaInfoValue));
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("About to allocate the new FCB\n"));
//
//     FCB = ExAllocatePoolWithTag(NonPagedPool, sizeof(AFD_FCB), TAG_AFD_FCB);
//     if( FCB == NULL ) {
//         Irp->IoStatus.Status = STATUS_NO_MEMORY;
//         IoCompleteRequest(Irp, IO_NO_INCREMENT);
//         return STATUS_NO_MEMORY;
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("Initializing the new FCB @ %p (FileObject %p Flags %x)\n",
//                             FCB, FileObject, ConnectInfo ? ConnectInfo->EndpointFlags : 0));
//
//     RtlZeroMemory( FCB, sizeof( *FCB ) );
//
//     FCB->Flags = ConnectInfo ? ConnectInfo->EndpointFlags : 0;
//     FCB->GroupID = ConnectInfo ? ConnectInfo->GroupID : 0;
//     FCB->GroupType = 0; /* FIXME */
//     FCB->SharedData.State = SOCKET_STATE_CREATED;
//     FCB->FileObject = FileObject;
//     FCB->DeviceExt = DeviceExt;
//     FCB->AddressFile.Handle = INVALID_HANDLE_VALUE;
//     FCB->Connection.Handle = INVALID_HANDLE_VALUE;
//     FCB->Recv.Size = AfdReceiveWindowSize;
//     FCB->Send.Size = AfdSendWindowSize;
//
//     KeInitializeMutex( &FCB->Mutex, 0 );
//
//     for( i = 0; i < MAX_FUNCTIONS; i++ ) {
//         InitializeListHead( &FCB->PendingIrpList[i] );
//     }
//
//     InitializeListHead( &FCB->DatagramList );
//     InitializeListHead( &FCB->PendingConnections );
//
//     AFD_DbgPrint!(MID_TRACE,("%p: Checking command channel\n", FCB));
//
//     if( ConnectInfo ) {
//         FCB->TdiDeviceName.Length = ConnectInfo->SizeOfTransportName;
//         FCB->TdiDeviceName.MaximumLength = FCB->TdiDeviceName.Length;
//         FCB->TdiDeviceName.Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                                           FCB->TdiDeviceName.Length,
//                                                           TAG_AFD_TRANSPORT_ADDRESS);
//
//         if( !FCB->TdiDeviceName.Buffer ) {
//             ExFreePoolWithTag(FCB, TAG_AFD_FCB);
//             AFD_DbgPrint!(MID_TRACE,("Could not copy target string\n"));
//             Irp->IoStatus.Status = STATUS_NO_MEMORY;
//             IoCompleteRequest( Irp, IO_NETWORK_INCREMENT );
//             return STATUS_NO_MEMORY;
//         }
//
//         RtlCopyMemory( FCB->TdiDeviceName.Buffer,
//                        ConnectInfo->TransportName,
//                        FCB->TdiDeviceName.Length );
//
//         AFD_DbgPrint!(MID_TRACE,("Success: %s %wZ\n",
//                                 EaInfo->EaName, &FCB->TdiDeviceName));
//     } else {
//         AFD_DbgPrint!(MID_TRACE,("Success: Control connection\n"));
//     }
//
//     FileObject->FsContext = FCB;
//
//     /* It seems that UDP sockets are writable from inception */
//     if( FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS ) {
//         AFD_DbgPrint!(MID_TRACE,("Packet oriented socket\n"));
//
//         /* A datagram socket is always sendable */
//         FCB->PollState |= AFD_EVENT_SEND;
//         FCB->PollStatus[FD_WRITE_BIT] = STATUS_SUCCESS;
//         PollReeval( FCB->DeviceExt, FCB->FileObject );
//     }
//
//     if( !NT_SUCCESS(Status) ) {
//         if (FCB->TdiDeviceName.Buffer)
//         {
//             ExFreePoolWithTag(FCB->TdiDeviceName.Buffer, TAG_AFD_TRANSPORT_ADDRESS);
//         }
//         ExFreePoolWithTag(FCB, TAG_AFD_FCB);
//         FileObject->FsContext = NULL;
//     }
//
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest( Irp, IO_NETWORK_INCREMENT );
//
//     return Status;
// }
//
// static NTSTATUS NTAPI
// AfdCleanupSocket(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                  PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PLIST_ENTRY CurrentEntry, NextEntry;
//     UINT Function;
//     PIRP CurrentIrp;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket(Irp);
//
//     for (Function = 0; Function < MAX_FUNCTIONS; Function++)
//     {
//         CurrentEntry = FCB->PendingIrpList[Function].Flink;
//         while (CurrentEntry != &FCB->PendingIrpList[Function])
//         {
//            NextEntry = CurrentEntry->Flink;
//            CurrentIrp = CONTAINING_RECORD(CurrentEntry, IRP, Tail.Overlay.ListEntry);
//
//            /* The cancel routine will remove the IRP from the list */
//            IoCancelIrp(CurrentIrp);
//
//            CurrentEntry = NextEntry;
//         }
//     }
//
//     KillSelectsForFCB( FCB->DeviceExt, FileObject, FALSE );
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
// }
//
// static NTSTATUS NTAPI
// AfdCloseSocket(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     UINT i;
//     PAFD_IN_FLIGHT_REQUEST InFlightRequest[IN_FLIGHT_REQUESTS];
//     PAFD_TDI_OBJECT_QELT Qelt;
//     PLIST_ENTRY QeltEntry;
//
//
//     AFD_DbgPrint!(MID_TRACE,("AfdClose(DeviceObject %p Irp %p)\n",
//                             DeviceObject, Irp));
//
//     if( !SocketAcquireStateLock( FCB ) ) return STATUS_FILE_CLOSED;
//
//     FCB->SharedData.State = SOCKET_STATE_CLOSED;
//
//     InFlightRequest[0] = &FCB->ListenIrp;
//     InFlightRequest[1] = &FCB->ReceiveIrp;
//     InFlightRequest[2] = &FCB->SendIrp;
//     InFlightRequest[3] = &FCB->ConnectIrp;
//     InFlightRequest[4] = &FCB->DisconnectIrp;
//
//     /* Cancel our pending requests */
//     for( i = 0; i < IN_FLIGHT_REQUESTS; i++ ) {
//         if( InFlightRequest[i]->InFlightRequest ) {
//             AFD_DbgPrint!(MID_TRACE,("Cancelling in flight irp %u (%p)\n",
//                                     i, InFlightRequest[i]->InFlightRequest));
//             IoCancelIrp(InFlightRequest[i]->InFlightRequest);
//         }
//     }
//
//     KillSelectsForFCB( FCB->DeviceExt, FileObject, FALSE );
//
//     ASSERT(IsListEmpty(&FCB->PendingIrpList[FUNCTION_CONNECT]));
//     ASSERT(IsListEmpty(&FCB->PendingIrpList[FUNCTION_CONNECTEX]));
//     ASSERT(IsListEmpty(&FCB->PendingIrpList[FUNCTION_SEND]));
//     ASSERT(IsListEmpty(&FCB->PendingIrpList[FUNCTION_RECV]));
//     ASSERT(IsListEmpty(&FCB->PendingIrpList[FUNCTION_PREACCEPT]));
//     ASSERT(IsListEmpty(&FCB->PendingIrpList[FUNCTION_DISCONNECT]));
//
//     while (!IsListEmpty(&FCB->PendingConnections))
//     {
//         QeltEntry = RemoveHeadList(&FCB->PendingConnections);
//         Qelt = CONTAINING_RECORD(QeltEntry, AFD_TDI_OBJECT_QELT, ListEntry);
//
//         /* We have to close all pending connections or the listen won't get closed */
//         TdiDisassociateAddressFile(Qelt->Object.Object);
//         ObDereferenceObject(Qelt->Object.Object);
//         ZwClose(Qelt->Object.Handle);
//
//         ExFreePoolWithTag(Qelt, TAG_AFD_ACCEPT_QUEUE);
//     }
//
//     SocketStateUnlock( FCB );
//
//     if( FCB->EventSelect )
//         ObDereferenceObject( FCB->EventSelect );
//
//     if (FCB->Context)
//         ExFreePoolWithTag(FCB->Context, TAG_AFD_SOCKET_CONTEXT);
//
//     if (FCB->Recv.Window)
//         ExFreePoolWithTag(FCB->Recv.Window, TAG_AFD_DATA_BUFFER);
//
//     if (FCB->Send.Window)
//         ExFreePoolWithTag(FCB->Send.Window, TAG_AFD_DATA_BUFFER);
//
//     if (FCB->AddressFrom)
//         ExFreePoolWithTag(FCB->AddressFrom, TAG_AFD_TDI_CONNECTION_INFORMATION);
//
//     if (FCB->ConnectCallInfo)
//         ExFreePoolWithTag(FCB->ConnectCallInfo, TAG_AFD_TDI_CONNECTION_INFORMATION);
//
//     if (FCB->ConnectReturnInfo)
//         ExFreePoolWithTag(FCB->ConnectReturnInfo, TAG_AFD_TDI_CONNECTION_INFORMATION);
//
//     if (FCB->ConnectData)
//         ExFreePoolWithTag(FCB->ConnectData, TAG_AFD_CONNECT_DATA);
//
//     if (FCB->DisconnectData)
//         ExFreePoolWithTag(FCB->DisconnectData, TAG_AFD_DISCONNECT_DATA);
//
//     if (FCB->ConnectOptions)
//         ExFreePoolWithTag(FCB->ConnectOptions, TAG_AFD_CONNECT_OPTIONS);
//
//     if (FCB->DisconnectOptions)
//         ExFreePoolWithTag(FCB->DisconnectOptions, TAG_AFD_DISCONNECT_OPTIONS);
//
//     if (FCB->LocalAddress)
//         ExFreePoolWithTag(FCB->LocalAddress, TAG_AFD_TRANSPORT_ADDRESS);
//
//     if (FCB->RemoteAddress)
//         ExFreePoolWithTag(FCB->RemoteAddress, TAG_AFD_TRANSPORT_ADDRESS);
//
//     if (FCB->OnConnectSendBuffer)
//         ExFreePoolWithTag(FCB->OnConnectSendBuffer, TAG_AFD_SUPER_CONNECT_BUFFER);
//
//     if( FCB->Connection.Object )
//     {
//         TdiDisassociateAddressFile(FCB->Connection.Object);
//         ObDereferenceObject(FCB->Connection.Object);
//     }
//
//     if( FCB->AddressFile.Object )
//         ObDereferenceObject(FCB->AddressFile.Object);
//
//     if( FCB->AddressFile.Handle != INVALID_HANDLE_VALUE )
//     {
//         if (ZwClose(FCB->AddressFile.Handle) == STATUS_INVALID_HANDLE)
//         {
//             DbgPrint("INVALID ADDRESS FILE HANDLE VALUE: %p %p\n", FCB->AddressFile.Handle, FCB->AddressFile.Object);
//         }
//     }
//
//     if( FCB->Connection.Handle != INVALID_HANDLE_VALUE )
//     {
//         if (ZwClose(FCB->Connection.Handle) == STATUS_INVALID_HANDLE)
//         {
//             DbgPrint("INVALID CONNECTION HANDLE VALUE: %p %p\n", FCB->Connection.Handle, FCB->Connection.Object);
//         }
//     }
//
//     if (FCB->TdiDeviceName.Buffer)
//     {
//         ExFreePoolWithTag(FCB->TdiDeviceName.Buffer, TAG_AFD_TRANSPORT_ADDRESS);
//     }
//
//     ExFreePoolWithTag(FCB, TAG_AFD_FCB);
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = 0;
//     IoCompleteRequest(Irp, IO_NETWORK_INCREMENT);
//
//     AFD_DbgPrint!(MID_TRACE, ("Returning success.\n"));
//
//     return STATUS_SUCCESS;
// }
//
// static IO_COMPLETION_ROUTINE DisconnectComplete;
// static
// NTSTATUS
// NTAPI
// DisconnectComplete(PDEVICE_OBJECT DeviceObject,
//                    PIRP Irp,
//                    PVOID Context)
// {
//     PAFD_FCB FCB = Context;
//     PIRP CurrentIrp;
//     PLIST_ENTRY CurrentEntry;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB))
//         return STATUS_FILE_CLOSED;
//
//     ASSERT(FCB->DisconnectIrp.InFlightRequest == Irp);
//     FCB->DisconnectIrp.InFlightRequest = NULL;
//
//     ASSERT(FCB->DisconnectPending);
//     ASSERT((IsListEmpty(&FCB->PendingIrpList[FUNCTION_SEND]) && !FCB->SendIrp.InFlightRequest) ||
//            (FCB->DisconnectFlags & TDI_DISCONNECT_ABORT));
//
//     if (NT_SUCCESS(Irp->IoStatus.Status) && (FCB->DisconnectFlags & TDI_DISCONNECT_RELEASE))
//     {
//         FCB->FilledDisconnectData = MIN(FCB->DisconnectDataSize, FCB->ConnectReturnInfo->UserDataLength);
//         if (FCB->FilledDisconnectData)
//         {
//             RtlCopyMemory(FCB->DisconnectData,
//                           FCB->ConnectReturnInfo->UserData,
//                           FCB->FilledDisconnectData);
//         }
//
//         FCB->FilledDisconnectOptions = MIN(FCB->DisconnectOptionsSize, FCB->ConnectReturnInfo->OptionsLength);
//         if (FCB->FilledDisconnectOptions)
//         {
//             RtlCopyMemory(FCB->DisconnectOptions,
//                           FCB->ConnectReturnInfo->Options,
//                           FCB->FilledDisconnectOptions);
//         }
//     }
//
//     FCB->DisconnectPending = FALSE;
//
//     while (!IsListEmpty(&FCB->PendingIrpList[FUNCTION_DISCONNECT]))
//     {
//         CurrentEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_DISCONNECT]);
//         CurrentIrp = CONTAINING_RECORD(CurrentEntry, IRP, Tail.Overlay.ListEntry);
//         CurrentIrp->IoStatus.Status = Irp->IoStatus.Status;
//         CurrentIrp->IoStatus.Information = 0;
//         UnlockRequest(CurrentIrp, IoGetCurrentIrpStackLocation(CurrentIrp));
//         (void)IoSetCancelRoutine(CurrentIrp, NULL);
//         IoCompleteRequest(CurrentIrp, IO_NETWORK_INCREMENT );
//     }
//
//     if (!(FCB->DisconnectFlags & TDI_DISCONNECT_RELEASE))
//     {
//         /* Signal complete connection closure immediately */
//         FCB->PollState |= AFD_EVENT_ABORT;
//         FCB->PollStatus[FD_CLOSE_BIT] = Irp->IoStatus.Status;
//         FCB->LastReceiveStatus = STATUS_FILE_CLOSED;
//         PollReeval(FCB->DeviceExt, FCB->FileObject);
//     }
//
//     SocketStateUnlock(FCB);
//
//     return Irp->IoStatus.Status;
// }
//
// static
// NTSTATUS
// DoDisconnect(PAFD_FCB FCB)
// {
//     NTSTATUS Status;
//
//     ASSERT(FCB->DisconnectPending);
//     ASSERT((IsListEmpty(&FCB->PendingIrpList[FUNCTION_SEND]) && !FCB->SendIrp.InFlightRequest) ||
//            (FCB->DisconnectFlags & TDI_DISCONNECT_ABORT));
//
//     if (FCB->DisconnectIrp.InFlightRequest)
//     {
//         return STATUS_PENDING;
//     }
//
//     FCB->ConnectCallInfo->UserData = FCB->DisconnectData;
//     FCB->ConnectCallInfo->UserDataLength = FCB->DisconnectDataSize;
//     FCB->ConnectCallInfo->Options = FCB->DisconnectOptions;
//     FCB->ConnectCallInfo->OptionsLength = FCB->DisconnectOptionsSize;
//
//     Status = TdiDisconnect(&FCB->DisconnectIrp.InFlightRequest,
//                            FCB->Connection.Object,
//                            &FCB->DisconnectTimeout,
//                            FCB->DisconnectFlags,
//                            DisconnectComplete,
//                            FCB,
//                            FCB->ConnectCallInfo,
//                            FCB->ConnectReturnInfo);
//     if (Status != STATUS_PENDING)
//     {
//         FCB->DisconnectPending = FALSE;
//     }
//
//     return Status;
// }
//
// VOID
// RetryDisconnectCompletion(PAFD_FCB FCB)
// {
//     ASSERT(FCB->RemoteAddress);
//
//     if (IsListEmpty(&FCB->PendingIrpList[FUNCTION_SEND]) && !FCB->SendIrp.InFlightRequest && FCB->DisconnectPending)
//     {
//         /* Sends are done; fire off a TDI_DISCONNECT request */
//         DoDisconnect(FCB);
//     }
// }
//
// static NTSTATUS NTAPI
// AfdDisconnect(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//               PIO_STACK_LOCATION IrpSp) {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PAFD_DISCONNECT_INFO DisReq;
//     NTSTATUS Status = STATUS_SUCCESS;
//     USHORT Flags;
//     PLIST_ENTRY CurrentEntry;
//     PIRP CurrentIrp;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!(DisReq = LockRequest(Irp, IrpSp, FALSE, NULL)))
//         return UnlockAndMaybeComplete( FCB, STATUS_NO_MEMORY,
//                                        Irp, 0 );
//
//     /* Send direction only */
//     if ((DisReq->DisconnectType & AFD_DISCONNECT_SEND) &&
//         !(DisReq->DisconnectType & AFD_DISCONNECT_RECV))
//     {
//         /* Perform a controlled disconnect */
//         Flags = TDI_DISCONNECT_RELEASE;
//     }
//     /* Receive direction or both */
//     else
//     {
//         /* Mark that we can't issue another receive request */
//         FCB->TdiReceiveClosed = TRUE;
//
//         /* Try to cancel a pending TDI receive IRP if there was one in progress */
//         if (FCB->ReceiveIrp.InFlightRequest)
//             IoCancelIrp(FCB->ReceiveIrp.InFlightRequest);
//
//         /* Discard any pending data */
//         FCB->Recv.Content = 0;
//         FCB->Recv.BytesUsed = 0;
//
//         /* Set a successful receive status to indicate a shutdown on overread */
//         FCB->LastReceiveStatus = STATUS_SUCCESS;
//
//         /* Clear the receive event */
//         FCB->PollState &= ~AFD_EVENT_RECEIVE;
//
//         /* Receive direction only */
//         if ((DisReq->DisconnectType & AFD_DISCONNECT_RECV) &&
//             !(DisReq->DisconnectType & AFD_DISCONNECT_SEND))
//         {
//             /* No need to tell the transport driver for receive direction only */
//             return UnlockAndMaybeComplete( FCB, STATUS_SUCCESS, Irp, 0 );
//         }
//         else
//         {
//             /* Perform an abortive disconnect */
//             Flags = TDI_DISCONNECT_ABORT;
//         }
//     }
//
//     if (!(FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS))
//     {
//         if (!FCB->ConnectCallInfo)
//         {
//             AFD_DbgPrint!(MIN_TRACE,("Invalid parameter\n"));
//             return UnlockAndMaybeComplete( FCB, STATUS_INVALID_PARAMETER,
//                                            Irp, 0 );
//         }
//
//         if (FCB->DisconnectPending)
//         {
//             if (FCB->DisconnectIrp.InFlightRequest)
//             {
//                 IoCancelIrp(FCB->DisconnectIrp.InFlightRequest);
//                 ASSERT(!FCB->DisconnectIrp.InFlightRequest);
//             }
//             else
//             {
//                 while (!IsListEmpty(&FCB->PendingIrpList[FUNCTION_DISCONNECT]))
//                 {
//                     CurrentEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_DISCONNECT]);
//                     CurrentIrp = CONTAINING_RECORD(CurrentEntry, IRP, Tail.Overlay.ListEntry);
//                     CurrentIrp->IoStatus.Status = STATUS_CANCELLED;
//                     CurrentIrp->IoStatus.Information = 0;
//                     UnlockRequest(CurrentIrp, IoGetCurrentIrpStackLocation(CurrentIrp));
//                     (void)IoSetCancelRoutine(CurrentIrp, NULL);
//                     IoCompleteRequest(CurrentIrp, IO_NETWORK_INCREMENT );
//                 }
//             }
//         }
//
//         FCB->DisconnectFlags = Flags;
//         FCB->DisconnectTimeout = DisReq->Timeout;
//         FCB->DisconnectPending = TRUE;
//         FCB->SendClosed = TRUE;
//         FCB->PollState &= ~AFD_EVENT_SEND;
//
//         Status = QueueUserModeIrp(FCB, Irp, FUNCTION_DISCONNECT);
//         if (Status == STATUS_PENDING)
//         {
//             if ((IsListEmpty(&FCB->PendingIrpList[FUNCTION_SEND]) && !FCB->SendIrp.InFlightRequest) ||
//                 (FCB->DisconnectFlags & TDI_DISCONNECT_ABORT))
//             {
//                 /* Go ahead and execute the disconnect because we're ready for it */
//                 Status = DoDisconnect(FCB);
//             }
//
//             if (Status != STATUS_PENDING)
//                 RemoveEntryList(&Irp->Tail.Overlay.ListEntry);
//         }
//
//         if (Status == STATUS_PENDING)
//         {
//             SocketStateUnlock(FCB);
//
//             return Status;
//         }
//     }
//     else
//     {
//         if (!(Flags & TDI_DISCONNECT_RELEASE))
//         {
//             if (!FCB->RemoteAddress)
//             {
//                 AFD_DbgPrint!(MIN_TRACE,("Invalid parameter\n"));
//                 return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
//             }
//
//             ExFreePoolWithTag(FCB->RemoteAddress, TAG_AFD_TRANSPORT_ADDRESS);
//
//             FCB->RemoteAddress = NULL;
//         }
//
//         FCB->PollState &= ~AFD_EVENT_SEND;
//         FCB->SendClosed = TRUE;
//     }
//
//     return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
// }
//
// NTSTATUS
// AfdQueryFsDeviceInfo(PDEVICE_OBJECT DeviceObject, PFILE_FS_DEVICE_INFORMATION Buffer, PULONG Length)
// {
//     if (*Length >= sizeof(FILE_FS_DEVICE_INFORMATION))
//     {
//         Buffer->Characteristics = 0;
//         Buffer->DeviceType = FILE_DEVICE_NAMED_PIPE;
//
//         *Length -= sizeof(FILE_FS_DEVICE_INFORMATION);
//
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         ASSERT(*Length >= sizeof(FILE_FS_DEVICE_INFORMATION));
//         return STATUS_INFO_LENGTH_MISMATCH;
//     }
// }
//
// static NTSTATUS NTAPI
// AfdQueryVolumeInformation(PDEVICE_OBJECT DeviceObject, PIRP Irp, PIO_STACK_LOCATION IrpSp)
// {
//     FS_INFORMATION_CLASS InfoClass;
//     PVOID Buffer;
//     ULONG Length;
//     NTSTATUS Status = STATUS_INVALID_INFO_CLASS;
//
//     Buffer = Irp->AssociatedIrp.SystemBuffer;
//     Length = IrpSp->Parameters.QueryVolume.Length;
//     InfoClass = IrpSp->Parameters.QueryVolume.FsInformationClass;
//
//     switch (InfoClass)
//     {
//     case FileFsDeviceInformation:
//         Status = AfdQueryFsDeviceInfo(DeviceObject, Buffer, &Length);
//         break;
//     default:
//         break;
//     }
//
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = IrpSp->Parameters.QueryVolume.Length - Length;
//     IoCompleteRequest(Irp, IO_NETWORK_INCREMENT);
//
//     return Status;
// }
//
// static DRIVER_DISPATCH AfdDispatch;
// static NTSTATUS NTAPI
// AfdDispatch(PDEVICE_OBJECT DeviceObject, PIRP Irp)
// {
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation(Irp);
//     NTSTATUS Status = STATUS_NOT_IMPLEMENTED;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//
//     AFD_DbgPrint!(MID_TRACE,("AfdDispatch: %u\n", IrpSp->MajorFunction));
//     if( IrpSp->MajorFunction != IRP_MJ_CREATE) {
//         AFD_DbgPrint!(MID_TRACE,("FO %p, IrpSp->FO %p\n",
//                                 FileObject, IrpSp->FileObject));
//         ASSERT(FileObject == IrpSp->FileObject);
//     }
//
//     Irp->IoStatus.Information = 0;
//
//     switch(IrpSp->MajorFunction)
//     {
//         /* opening and closing handles to the device */
//     case IRP_MJ_CREATE:
//         /* Mostly borrowed from the named pipe file system */
//         return AfdCreateSocket(DeviceObject, Irp, IrpSp);
//
//     case IRP_MJ_CLOSE:
//         /* Ditto the borrowing */
//         return AfdCloseSocket(DeviceObject, Irp, IrpSp);
//
//     case IRP_MJ_CLEANUP:
//         return AfdCleanupSocket(DeviceObject, Irp, IrpSp);
//
//     /* write data */
//     case IRP_MJ_WRITE:
//         return AfdConnectedSocketWriteData( DeviceObject, Irp, IrpSp, TRUE );
//
//     /* read data */
//     case IRP_MJ_READ:
//         return AfdConnectedSocketReadData( DeviceObject, Irp, IrpSp, TRUE );
//
//     /* query volume info */
//     case IRP_MJ_QUERY_VOLUME_INFORMATION:
//         return AfdQueryVolumeInformation(DeviceObject, Irp, IrpSp);
//
//     case IRP_MJ_DEVICE_CONTROL:
//     {
//         switch( IrpSp->Parameters.DeviceIoControl.IoControlCode ) {
//         case IOCTL_AFD_BIND:
//             return AfdBindSocket( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_CONNECT:
//             return AfdStreamSocketConnect( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_SUPER_CONNECT:
//             return AfdStreamSocketSuperConnect(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_START_LISTEN:
//             return AfdListenSocket( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_RECV:
//             return AfdConnectedSocketReadData( DeviceObject, Irp, IrpSp,
//                                                FALSE );
//
//         case IOCTL_AFD_SELECT:
//             return AfdSelect( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_EVENT_SELECT:
//             return AfdEventSelect( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_ENUM_NETWORK_EVENTS:
//             return AfdEnumEvents( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_RECV_DATAGRAM:
//             return AfdPacketSocketReadData( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_SEND:
//             return AfdConnectedSocketWriteData( DeviceObject, Irp, IrpSp,
//                                                 FALSE );
//
//         case IOCTL_AFD_SEND_DATAGRAM:
//             return AfdPacketSocketWriteData( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_GET_INFO:
//             return AfdGetInfo( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_SET_INFO:
//             return AfdSetInfo( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_GET_CONTEXT_SIZE:
//             return AfdGetContextSize( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_GET_CONTEXT:
//             return AfdGetContext( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_SET_CONTEXT:
//             return AfdSetContext( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_WAIT_FOR_LISTEN:
//             return AfdWaitForListen( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_ACCEPT:
//             return AfdAccept( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_DISCONNECT:
//             return AfdDisconnect( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_GET_SOCK_NAME:
//             return AfdGetSockName( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_GET_PEER_NAME:
//             return AfdGetPeerName( DeviceObject, Irp, IrpSp );
//
//         case IOCTL_AFD_GET_CONNECT_DATA:
//             return AfdGetConnectData(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_SET_CONNECT_DATA:
//             return AfdSetConnectData(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_SET_DISCONNECT_DATA:
//             return AfdSetDisconnectData(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_GET_DISCONNECT_DATA:
//             return AfdGetDisconnectData(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_SET_CONNECT_DATA_SIZE:
//             return AfdSetConnectDataSize(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_SET_DISCONNECT_DATA_SIZE:
//             return AfdSetDisconnectDataSize(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_SET_CONNECT_OPTIONS:
//             return AfdSetConnectOptions(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_SET_DISCONNECT_OPTIONS:
//             return AfdSetDisconnectOptions(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_GET_CONNECT_OPTIONS:
//             return AfdGetConnectOptions(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_GET_DISCONNECT_OPTIONS:
//             return AfdGetDisconnectOptions(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_SET_CONNECT_OPTIONS_SIZE:
//             return AfdSetConnectOptionsSize(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_SET_DISCONNECT_OPTIONS_SIZE:
//             return AfdSetDisconnectOptionsSize(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_GET_TDI_HANDLES:
//             return AfdGetTdiHandles(DeviceObject, Irp, IrpSp);
//
//         case IOCTL_AFD_DEFER_ACCEPT:
//             DbgPrint("IOCTL_AFD_DEFER_ACCEPT is UNIMPLEMENTED!\n");
//             break;
//
//         case IOCTL_AFD_GET_PENDING_CONNECT_DATA:
//             DbgPrint("IOCTL_AFD_GET_PENDING_CONNECT_DATA is UNIMPLEMENTED!\n");
//             break;
//
//         case IOCTL_AFD_VALIDATE_GROUP:
//             DbgPrint("IOCTL_AFD_VALIDATE_GROUP is UNIMPLEMENTED!\n");
//             break;
//
//         default:
//             Status = STATUS_NOT_SUPPORTED;
//             DbgPrint("Unknown IOCTL (0x%x)\n",
//                      IrpSp->Parameters.DeviceIoControl.IoControlCode);
//             break;
//         }
//         break;
//     }
//
// /* unsupported operations */
//     default:
//     {
//         Status = STATUS_NOT_IMPLEMENTED;
//         AFD_DbgPrint!(MIN_TRACE,
//                      ("Irp: Unknown Major code was %x\n",
//                       IrpSp->MajorFunction));
//         break;
//     }
//     }
//
//     AFD_DbgPrint!(MID_TRACE, ("Returning %x\n", Status));
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// BOOLEAN CheckUnlockExtraBuffers(PAFD_FCB FCB, PIO_STACK_LOCATION IrpSp)
// {
//     if (FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS)
//     {
//         if (IrpSp->MajorFunction == IRP_MJ_READ || IrpSp->MajorFunction == IRP_MJ_WRITE)
//         {
//             /* read()/write() call - no extra buffers */
//             return FALSE;
//         }
//         else if (IrpSp->MajorFunction == IRP_MJ_DEVICE_CONTROL)
//         {
//             if (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_RECV_DATAGRAM)
//             {
//                 /* recvfrom() call - extra buffers */
//                 return TRUE;
//             }
//             else if (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_RECV)
//             {
//                 /* recv() call - no extra buffers */
//                 return FALSE;
//             }
//             else if (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_SEND ||
//                      IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_SEND_DATAGRAM)
//             {
//                 /* send()/sendto() call - no extra buffers */
//                 return FALSE;
//             }
//             else
//             {
//                 /* Unknown IOCTL */
//                 ASSERT(FALSE);
//                 return FALSE;
//             }
//         }
//         else
//         {
//             /* Unknown IRP_MJ code */
//             ASSERT(FALSE);
//             return FALSE;
//         }
//     }
//     else
//     {
//         /* Connection-oriented never has extra buffers */
//         return FALSE;
//     }
// }
//
// VOID
// CleanupPendingIrp(PAFD_FCB FCB, PIRP Irp, PIO_STACK_LOCATION IrpSp, PAFD_ACTIVE_POLL Poll)
// {
//     PAFD_RECV_INFO RecvReq;
//     PAFD_SEND_INFO SendReq;
//     PAFD_POLL_INFO PollReq;
//
//     if (IrpSp->MajorFunction == IRP_MJ_READ)
//     {
//         RecvReq = GetLockedData(Irp, IrpSp) as PAFD_RECV_INFO;
//         UnlockBuffers(RecvReq->BufferArray, RecvReq->BufferCount, CheckUnlockExtraBuffers(FCB, IrpSp) as i32);
//     }
//     else if (IrpSp->MajorFunction == IRP_MJ_WRITE)
//     {
//         SendReq = GetLockedData(Irp, IrpSp) as PAFD_SEND_INFO;
//         UnlockBuffers(SendReq->BufferArray, SendReq->BufferCount, CheckUnlockExtraBuffers(FCB, IrpSp) as i32);
//     }
//     else
//     {
//         ASSERT(IrpSp->MajorFunction == IRP_MJ_DEVICE_CONTROL);
//
//         if (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_RECV)
//         {
//             RecvReq = GetLockedData(Irp, IrpSp) as PAFD_RECV_INFO;
//             UnlockBuffers(RecvReq->BufferArray, RecvReq->BufferCount, CheckUnlockExtraBuffers(FCB, IrpSp) as i32);
//         }
//         else if (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_SEND)
//         {
//             SendReq = GetLockedData(Irp, IrpSp) as PAFD_SEND_INFO;
//             UnlockBuffers(SendReq->BufferArray, SendReq->BufferCount, CheckUnlockExtraBuffers(FCB, IrpSp) as i32);
//         }
//         else if (IrpSp->Parameters.DeviceIoControl.IoControlCode == IOCTL_AFD_SELECT)
//         {
//             ASSERT(Poll);
//
//             PollReq = Irp->AssociatedIrp.SystemBuffer;
//             ZeroEvents(PollReq->Handles, PollReq->HandleCount);
//             SignalSocket(Poll, NULL, PollReq, STATUS_CANCELLED);
//         }
//     }
// }
//
// VOID
// NTAPI
// AfdCancelHandler(PDEVICE_OBJECT DeviceObject,
//                  PIRP Irp)
// {
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation(Irp);
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     ULONG Function, IoctlCode;
//     PIRP CurrentIrp;
//     PLIST_ENTRY CurrentEntry;
//     PAFD_DEVICE_EXTENSION DeviceExt = DeviceObject->DeviceExtension;
//     KIRQL OldIrql;
//     PAFD_ACTIVE_POLL Poll;
//
//     IoReleaseCancelSpinLock(Irp->CancelIrql);
//
//     if (!SocketAcquireStateLock(FCB))
//         return;
//
//     switch (IrpSp->MajorFunction)
//     {
//         case IRP_MJ_DEVICE_CONTROL:
//             IoctlCode = IrpSp->Parameters.DeviceIoControl.IoControlCode;
//             break;
//
//         case IRP_MJ_READ:
//             IoctlCode = IOCTL_AFD_RECV;
//             break;
//
//         case IRP_MJ_WRITE:
//             IoctlCode = IOCTL_AFD_SEND;
//             break;
//
//         default:
//             ASSERT(FALSE);
//             SocketStateUnlock(FCB);
//             return;
//     }
//
//     switch (IoctlCode)
//     {
//         case IOCTL_AFD_RECV:
//         case IOCTL_AFD_RECV_DATAGRAM:
//             Function = FUNCTION_RECV;
//             break;
//
//         case IOCTL_AFD_SEND:
//         case IOCTL_AFD_SEND_DATAGRAM:
//             Function = FUNCTION_SEND;
//             break;
//
//         case IOCTL_AFD_CONNECT:
//             Function = FUNCTION_CONNECT;
//             break;
//
//         case IOCTL_AFD_SUPER_CONNECT:
//             Function = FUNCTION_CONNECTEX;
//             break;
//
//         case IOCTL_AFD_WAIT_FOR_LISTEN:
//             Function = FUNCTION_PREACCEPT;
//             break;
//
//         case IOCTL_AFD_SELECT:
//             KeAcquireSpinLock(&DeviceExt->Lock, &OldIrql);
//
//             CurrentEntry = DeviceExt->Polls.Flink;
//             while (CurrentEntry != &DeviceExt->Polls)
//             {
//                 Poll = CONTAINING_RECORD(CurrentEntry, AFD_ACTIVE_POLL, ListEntry);
//
//                 if (Irp == Poll->Irp)
//                 {
//                     CleanupPendingIrp(FCB, Irp, IrpSp, Poll);
//                     KeReleaseSpinLock(&DeviceExt->Lock, OldIrql);
//                     SocketStateUnlock(FCB);
//                     return;
//                 }
//                 else
//                 {
//                     CurrentEntry = CurrentEntry->Flink;
//                 }
//             }
//
//             KeReleaseSpinLock(&DeviceExt->Lock, OldIrql);
//
//             SocketStateUnlock(FCB);
//
//             DbgPrint("WARNING!!! IRP cancellation race could lead to a process hang! (IOCTL_AFD_SELECT)\n");
//             return;
//
//         case IOCTL_AFD_DISCONNECT:
//             Function = FUNCTION_DISCONNECT;
//             break;
//
//         default:
//             ASSERT(FALSE);
//             UnlockAndMaybeComplete(FCB, STATUS_CANCELLED, Irp, 0);
//             return;
//     }
//
//     CurrentEntry = FCB->PendingIrpList[Function].Flink;
//     while (CurrentEntry != &FCB->PendingIrpList[Function])
//     {
//         CurrentIrp = CONTAINING_RECORD(CurrentEntry, IRP, Tail.Overlay.ListEntry);
//
//         if (CurrentIrp == Irp)
//         {
//             RemoveEntryList(CurrentEntry);
//             CleanupPendingIrp(FCB, Irp, IrpSp, NULL);
//             UnlockAndMaybeComplete(FCB, STATUS_CANCELLED, Irp, 0);
//             return;
//         }
//         else
//         {
//             CurrentEntry = CurrentEntry->Flink;
//         }
//     }
//
//     SocketStateUnlock(FCB);
//
//     DbgPrint("WARNING!!! IRP cancellation race could lead to a process hang! (Function: %u)\n", Function);
// }
//
// static DRIVER_UNLOAD AfdUnload;
// static VOID NTAPI
// AfdUnload(PDRIVER_OBJECT DriverObject)
// {
//     UNREFERENCED_PARAMETER(DriverObject);
// }
//
// NTSTATUS NTAPI
// DriverEntry(PDRIVER_OBJECT DriverObject, PUNICODE_STRING RegistryPath)
// {
//     PDEVICE_OBJECT DeviceObject;
//     UNICODE_STRING wstrDeviceName = RTL_CONSTANT_STRING(L"\\Device\\Afd");
//     PAFD_DEVICE_EXTENSION DeviceExt;
//     NTSTATUS Status;
//
//     UNREFERENCED_PARAMETER(RegistryPath);
//     /* register driver routines */
//     DriverObject->MajorFunction[IRP_MJ_CLOSE] = AfdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_CREATE] = AfdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_CLEANUP] = AfdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_WRITE] = AfdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_READ] = AfdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = AfdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION] = AfdDispatch;
//     DriverObject->DriverUnload = AfdUnload;
//
//     Status = IoCreateDevice(DriverObject,
//                             sizeof(AFD_DEVICE_EXTENSION),
//                             &wstrDeviceName,
//                             FILE_DEVICE_NAMED_PIPE,
//                             0,
//                             FALSE,
//                             &DeviceObject);
//
//     /* failure */
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     DeviceExt = DeviceObject->DeviceExtension;
//     KeInitializeSpinLock( &DeviceExt->Lock );
//     InitializeListHead( &DeviceExt->Polls );
//
//     AFD_DbgPrint!(MID_TRACE,("Device created: object %p ext %p\n",
//                             DeviceObject, DeviceExt));
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
