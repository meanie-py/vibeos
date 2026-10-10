use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IRP, KPROCESSOR_MODE, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSetCancelRoutine};

extern "C" {
    fn LockRequest(Irp: PIRP, IrpSp: PIO_STACK_LOCATION, Write: u8, SizeOut: *mut KPROCESSOR_MODE) -> *mut core::ffi::c_void;
    fn LostSocket(Irp: PIRP) -> i32;
    fn SocketAcquireStateLock(FCB: PAFD_FCB) -> u8;
    fn TdiAssociateAddressFile(Handle: HANDLE, Object: PFILE_OBJECT) -> NTSTATUS;
    fn TdiOpenConnectionEndpointFile( DeviceName: *mut _UNICODE_STRING, Handle: *mut HANDLE, Object: *mut PFILE_OBJECT ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn AfdGetConnectOptions(
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

    if (*FCB).ConnectOptionsSize == 0 {
        AFD_DbgPrint!(MIN_TRACE, b"Invalid parameter\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
    }

    ASSERT!((*FCB).ConnectOptions);

    if (*FCB).FilledConnectOptions < BufferSize {
        BufferSize = (*FCB).FilledConnectOptions;
    }

    core::ptr::copy_nonoverlapping(
        (*FCB).ConnectOptions as *const u8,
        (*Irp).UserBuffer as *mut u8,
        BufferSize as usize,
    );

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, BufferSize as UINT)
}

#[no_mangle]
pub unsafe extern "system" fn AfdSetConnectOptions(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let ConnectOptions = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut());
    let ConnectOptionsSize = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if ConnectOptions.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if !(*FCB).ConnectOptions.is_null() {
        ExFreePoolWithTag((*FCB).ConnectOptions, TAG_AFD_CONNECT_OPTIONS);
        (*FCB).ConnectOptions = core::ptr::null_mut();
        (*FCB).ConnectOptionsSize = 0;
        (*FCB).FilledConnectOptions = 0;
    }

    (*FCB).ConnectOptions = ExAllocatePoolWithTag(
        PagedPool,
        ConnectOptionsSize as usize,
        TAG_AFD_CONNECT_OPTIONS,
    );

    if (*FCB).ConnectOptions.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    core::ptr::copy_nonoverlapping(
        ConnectOptions as *const u8,
        (*FCB).ConnectOptions as *mut u8,
        ConnectOptionsSize as usize,
    );

    (*FCB).ConnectOptionsSize = ConnectOptionsSize;

    return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
}

#[no_mangle]
pub unsafe extern "system" fn AfdSetConnectOptionsSize(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> i32 {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let ConnectOptionsSize = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut()) as *mut u32;
    let BufferSize = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if ConnectOptionsSize.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if BufferSize < core::mem::size_of::<u32>() as u32 {
        AFD_DbgPrint!(0, b"Buffer too small\n\0".as_ptr() as *const i8);
        return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, 0);
    }

    if !(*FCB).ConnectOptions.is_null() {
        ExFreePoolWithTag((*FCB).ConnectOptions as *mut core::ffi::c_void, TAG_AFD_CONNECT_OPTIONS);
        (*FCB).ConnectOptionsSize = 0;
        (*FCB).FilledConnectOptions = 0;
    }

    (*FCB).ConnectOptions = ExAllocatePoolWithTag(
        PagedPool,
        *ConnectOptionsSize as usize,
        TAG_AFD_CONNECT_OPTIONS,
    );

    if (*FCB).ConnectOptions.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    (*FCB).ConnectOptionsSize = *ConnectOptionsSize;

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}

#[no_mangle]
pub unsafe extern "system" fn AfdGetConnectData(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let mut BufferSize = (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength;

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if (*FCB).ConnectDataSize == 0 {
        AFD_DbgPrint!(MIN_TRACE, b"Invalid parameter\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
    }

    if (*FCB).FilledConnectData < BufferSize {
        BufferSize = (*FCB).FilledConnectData;
    }

    core::ptr::copy_nonoverlapping(
        (*FCB).ConnectData as *const u8,
        (*Irp).UserBuffer as *mut u8,
        BufferSize as usize,
    );

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, BufferSize as UINT)
}

#[no_mangle]
pub unsafe extern "system" fn AfdSetConnectData(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let ConnectData = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut());
    let ConnectDataSize = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if ConnectData.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if !(*FCB).ConnectData.is_null() {
        ExFreePoolWithTag((*FCB).ConnectData, TAG_AFD_CONNECT_DATA);
        (*FCB).ConnectData = core::ptr::null_mut();
        (*FCB).ConnectDataSize = 0;
        (*FCB).FilledConnectData = 0;
    }

    (*FCB).ConnectData = ExAllocatePoolWithTag(
        PagedPool,
        ConnectDataSize as usize,
        TAG_AFD_CONNECT_DATA,
    );

    if (*FCB).ConnectData.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    core::ptr::copy_nonoverlapping(
        ConnectData as *const u8,
        (*FCB).ConnectData as *mut u8,
        ConnectDataSize as usize,
    );

    (*FCB).ConnectDataSize = ConnectDataSize;

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}

#[no_mangle]
pub unsafe extern "system" fn AfdSetConnectDataSize(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let ConnectDataSize = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut()) as *mut UINT;
    let BufferSize = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if ConnectDataSize.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if BufferSize < core::mem::size_of::<UINT>() as u32 {
        AFD_DbgPrint!(MIN_TRACE, b"Buffer too small\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, 0);
    }

    if !(*FCB).ConnectData.is_null() {
        ExFreePoolWithTag((*FCB).ConnectData as PVOID, TAG_AFD_CONNECT_DATA);
        (*FCB).ConnectDataSize = 0;
        (*FCB).FilledConnectData = 0;
    }

    (*FCB).ConnectData = ExAllocatePoolWithTag(
        PagedPool,
        *ConnectDataSize as usize,
        TAG_AFD_CONNECT_DATA,
    );

    if (*FCB).ConnectData.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    (*FCB).ConnectDataSize = *ConnectDataSize;

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}
#[no_mangle]
pub unsafe extern "C" fn WarmSocketForConnection(FCB: PAFD_FCB) -> NTSTATUS {
    let mut Status: NTSTATUS;

    if (*FCB).TdiDeviceName.Length == 0 || (*FCB).TdiDeviceName.Buffer.is_null() {
        AFD_DbgPrint!(MIN_TRACE, b"Null Device\n\0".as_ptr() as PCHAR);
        return STATUS_NO_SUCH_DEVICE;
    }

    Status = TdiOpenConnectionEndpointFile(
        core::ptr::addr_of_mut!((*FCB).TdiDeviceName),
        core::ptr::addr_of_mut!((*FCB).Connection.Handle),
        core::ptr::addr_of_mut!((*FCB).Connection.Object)
    );

    if NT_SUCCESS(Status) {
        Status = TdiAssociateAddressFile(
            (*FCB).AddressFile.Handle,
            (*FCB).Connection.Object
        );
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn MakeSocketIntoConnection(FCB: PAFD_FCB, Receive: BOOL) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if (*FCB).Recv.Size == 0 {
        Status = TdiQueryMaxDatagramLength(
            (*FCB).Connection.Object,
            core::ptr::addr_of_mut!((*FCB).Recv.Size),
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    if (*FCB).Send.Size == 0 {
        Status = TdiQueryMaxDatagramLength(
            (*FCB).Connection.Object,
            core::ptr::addr_of_mut!((*FCB).Send.Size),
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    if (*FCB).Recv.Window.is_null() {
        (*FCB).Recv.Window = ExAllocatePoolWithTag(
            PagedPool,
            (*FCB).Recv.Size as usize,
            TAG_AFD_DATA_BUFFER,
        ) as *mut i8;

        if (*FCB).Recv.Window.is_null() {
            return STATUS_NO_MEMORY;
        }
    }

    if (*FCB).Send.Window.is_null() {
        (*FCB).Send.Window = ExAllocatePoolWithTag(
            PagedPool,
            (*FCB).Send.Size as usize,
            TAG_AFD_DATA_BUFFER,
        ) as *mut i8;

        if (*FCB).Send.Window.is_null() {
            return STATUS_NO_MEMORY;
        }
    }

    (*FCB).SharedData.State = SOCKET_STATE_CONNECTED;
    (*FCB).SharedData.ConnectTime = 0;

    if Receive != 0 {
        Status = TdiReceive(
            core::ptr::addr_of_mut!((*FCB).ReceiveIrp.InFlightRequest),
            (*FCB).Connection.Object,
            TDI_RECEIVE_NORMAL as u16,
            (*FCB).Recv.Window,
            (*FCB).Recv.Size,
            Some(ReceiveComplete),
            FCB as *mut core::ffi::c_void,
        );

        if Status == STATUS_PENDING {
            Status = STATUS_SUCCESS;
        }
    }

    (*FCB).PollState |= AFD_EVENT_CONNECT | AFD_EVENT_SEND;
    (*FCB).PollStatus[FD_CONNECT_BIT as usize] = STATUS_SUCCESS;
    (*FCB).PollStatus[FD_WRITE_BIT as usize] = STATUS_SUCCESS;
    PollReeval((*FCB).DeviceExt, (*FCB).FileObject);

    Status
}

unsafe extern "system" fn StreamSocketConnectSendComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    stream_socket_connect_send_complete_impl(DeviceObject, Irp, Context)
}

unsafe fn stream_socket_connect_send_complete_impl(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let FCB = Context as PAFD_FCB;
    let mut NextIrpEntry: PLIST_ENTRY;
    let mut NextIrp: PIRP;

    if SocketAcquireStateLock(FCB) == 0 {
        return STATUS_FILE_CLOSED;
    }

    (*FCB).SendIrp.InFlightRequest = core::ptr::null_mut();

    while !IsListEmpty(&mut (*FCB).PendingIrpList[FUNCTION_CONNECTEX as usize]) {
        NextIrpEntry = RemoveHeadList(&mut (*FCB).PendingIrpList[FUNCTION_CONNECTEX as usize]);
        NextIrp = (NextIrpEntry as *mut u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;

        AFD_DbgPrint!(MID_TRACE, b"Completing connectex %p\n\0".as_ptr() as PCHAR, NextIrp);
        (*NextIrp).IoStatus.__bindgen_anon_1.Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        (*NextIrp).IoStatus.Information = (*Irp).IoStatus.Information;
        if !(*NextIrp).MdlAddress.is_null() {
            UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
        }
        IoSetCancelRoutine(NextIrp, None);
        IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT as i8);
    }

    if !(*FCB).OnConnectSendBuffer.is_null() {
        ExFreePoolWithTag((*FCB).OnConnectSendBuffer as PVOID, TAG_AFD_SUPER_CONNECT_BUFFER);
        (*FCB).OnConnectSendBuffer = core::ptr::null_mut();
        (*FCB).OnConnectSendBufferSize = 0;
    }

    SocketStateUnlock(FCB);

    STATUS_SUCCESS
}
unsafe extern "system" fn StreamSocketConnectComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    stream_socket_connect_complete_impl(DeviceObject, Irp, Context)
}

unsafe fn stream_socket_connect_complete_impl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let mut Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    let FCB = Context as PAFD_FCB;
    let mut NextIrpEntry: PLIST_ENTRY;
    let mut NextIrp: PIRP;

    AFD_DbgPrint!(
        MID_TRACE,
        b"Called: FCB %p, FO %p\n\0".as_ptr() as PCHAR,
        Context,
        (*FCB).FileObject,
    );

    if SocketAcquireStateLock(FCB) == 0 {
        return STATUS_FILE_CLOSED;
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"Irp->IoStatus.__bindgen_anon_1.Status = %x\n\0".as_ptr() as PCHAR,
        (*Irp).IoStatus.__bindgen_anon_1.Status,
    );

    (*FCB).ConnectIrp.InFlightRequest = core::ptr::null_mut();

    if (*FCB).SharedData.State == SOCKET_STATE_CLOSED {
        while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_CONNECT as usize])) {
            NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_CONNECT as usize]));
            NextIrp = (NextIrpEntry as *mut u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;
            (*NextIrp).IoStatus.__bindgen_anon_1.Status = STATUS_FILE_CLOSED;
            (*NextIrp).IoStatus.Information = 0;
            if !(*NextIrp).MdlAddress.is_null() {
                UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
            }
            IoSetCancelRoutine(NextIrp, None);
            IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
        }
        while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_CONNECTEX as usize])) {
            NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_CONNECTEX as usize]));
            NextIrp = (NextIrpEntry as *mut u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;
            (*NextIrp).IoStatus.__bindgen_anon_1.Status = STATUS_FILE_CLOSED;
            (*NextIrp).IoStatus.Information = (*Irp).IoStatus.Information;
            if !(*NextIrp).MdlAddress.is_null() {
                UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
            }
            IoSetCancelRoutine(NextIrp, None);
            IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
        }
        SocketStateUnlock(FCB);
        return STATUS_FILE_CLOSED;
    }

    if !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
        (*FCB).PollState |= AFD_EVENT_CONNECT_FAIL;
        (*FCB).PollStatus[FD_CONNECT_BIT as usize] = (*Irp).IoStatus.__bindgen_anon_1.Status;
        AFD_DbgPrint!(MID_TRACE, b"Going to bound state\n\0".as_ptr() as PCHAR);
        (*FCB).SharedData.State = SOCKET_STATE_BOUND;
        PollReeval((*FCB).DeviceExt, (*FCB).FileObject);
    }

    while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_CONNECT as usize])) {
        NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_CONNECT as usize]));
        NextIrp = (NextIrpEntry as *mut u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;
        AFD_DbgPrint!(MID_TRACE, b"Completing connect %p\n\0".as_ptr() as PCHAR, NextIrp);
        (*NextIrp).IoStatus.__bindgen_anon_1.Status = Status;
        (*NextIrp).IoStatus.Information = if NT_SUCCESS(Status) {
            (*FCB).Connection.Handle as ULONG_PTR as _
        } else {
            0
        };
        if !(*NextIrp).MdlAddress.is_null() {
            UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
        }
        IoSetCancelRoutine(NextIrp, None);
        IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
    }

    if NT_SUCCESS(Status) {
        Status = MakeSocketIntoConnection(FCB, 1);

        if !NT_SUCCESS(Status) {
            'end: loop {
                while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_CONNECTEX as usize])) {
                    NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_CONNECTEX as usize]));
                    NextIrp = (NextIrpEntry as *mut u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;

                    AFD_DbgPrint!(MID_TRACE, b"Completing connectex %p\n\0".as_ptr() as PCHAR, NextIrp);
                    (*NextIrp).IoStatus.__bindgen_anon_1.Status = Status;
                    (*NextIrp).IoStatus.Information = 0;
                    if !(*NextIrp).MdlAddress.is_null() {
                        UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
                    }
                    IoSetCancelRoutine(NextIrp, None);
                    IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
                }
                SocketStateUnlock(FCB);

                AFD_DbgPrint!(MID_TRACE, b"Returning %x\n\0".as_ptr() as PCHAR, Status);

                return Status;
            }
        }

        (*FCB).FilledConnectData = MIN((*(*FCB).ConnectReturnInfo).UserDataLength as u32, (*FCB).ConnectDataSize);
        if (*FCB).FilledConnectData != 0 {
            core::ptr::copy_nonoverlapping(
                (*(*FCB).ConnectReturnInfo).UserData,
                (*FCB).ConnectData,
                (*FCB).FilledConnectData as usize,
            );
        }

        (*FCB).FilledConnectOptions = MIN((*(*FCB).ConnectReturnInfo).OptionsLength as u32, (*FCB).ConnectOptionsSize);
        if (*FCB).FilledConnectOptions != 0 {
            core::ptr::copy_nonoverlapping(
                (*(*FCB).ConnectReturnInfo).Options,
                (*FCB).ConnectOptions,
                (*FCB).FilledConnectOptions as usize,
            );
        }

        if !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize])) {
            NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize]));
            NextIrp = (NextIrpEntry as *mut u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;
            AFD_DbgPrint!(MID_TRACE, b"Launching send request %p\n\0".as_ptr() as PCHAR, NextIrp);
            Status = AfdConnectedSocketWriteData(
                DeviceObject,
                NextIrp,
                IoGetCurrentIrpStackLocation(NextIrp),
                0u8,
            );
        }

        if Status == STATUS_PENDING {
            Status = STATUS_SUCCESS;
        }

        if !(*FCB).OnConnectSendBuffer.is_null() {
            let object = (*FCB).Connection.Object;
            let sendBuffer = (*FCB).OnConnectSendBuffer;
            let sendBufferLength = (*FCB).OnConnectSendBufferSize;
            SocketStateUnlock(FCB);

            Status = TdiSend(
                core::ptr::addr_of_mut!((*FCB).SendIrp.InFlightRequest),
                object,
                AFD_OVERLAPPED as u16,
                sendBuffer as PCHAR,
                sendBufferLength,
                Some(StreamSocketConnectSendComplete),
                FCB as PVOID,
            );

            return if Status == STATUS_PENDING { STATUS_SUCCESS } else { Status };
        }
    }

    while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_CONNECTEX as usize])) {
        NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_CONNECTEX as usize]));
        NextIrp = (NextIrpEntry as *mut u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;

        AFD_DbgPrint!(MID_TRACE, b"Completing connectex %p\n\0".as_ptr() as PCHAR, NextIrp);
        (*NextIrp).IoStatus.__bindgen_anon_1.Status = Status;
        (*NextIrp).IoStatus.Information = 0;
        if !(*NextIrp).MdlAddress.is_null() {
            UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
        }
        IoSetCancelRoutine(NextIrp, None);
        IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
    }
    SocketStateUnlock(FCB);

    AFD_DbgPrint!(MID_TRACE, b"Returning %x\n\0".as_ptr() as PCHAR, Status);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn AfdStreamSocketConnect(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_INVALID_PARAMETER;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let ConnectReq: PAFD_CONNECT_INFO;

    AFD_DbgPrint!(MID_TRACE, b"Called on %p\n\0".as_ptr() as PCHAR, FCB);

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    ConnectReq = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut()) as PAFD_CONNECT_INFO;
    if ConnectReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    AFD_DbgPrint!(MID_TRACE, b"Connect request:\n\0".as_ptr() as PCHAR);

    if ((*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS) != 0 {
        if !(*FCB).RemoteAddress.is_null() {
            ExFreePoolWithTag((*FCB).RemoteAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
        }

        (*FCB).RemoteAddress = TaCopyTransportAddress(core::ptr::addr_of_mut!((*ConnectReq).RemoteAddress));

        if (*FCB).RemoteAddress.is_null() {
            Status = STATUS_NO_MEMORY;
        } else {
            Status = STATUS_SUCCESS;
        }

        return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
    }

    match (*FCB).SharedData.State {
        SOCKET_STATE_CONNECTED => {
            Status = STATUS_INVALID_PARAMETER;
        }

        SOCKET_STATE_CONNECTING => {
            return LeaveIrpUntilLater(FCB, Irp, FUNCTION_CONNECT);
        }

        SOCKET_STATE_CREATED => {
            if !(*FCB).LocalAddress.is_null() {
                ExFreePoolWithTag((*FCB).LocalAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
            }

            (*FCB).LocalAddress = TaBuildNullTransportAddress((*ConnectReq).RemoteAddress.Address[0].AddressType as u32);

            if !(*FCB).LocalAddress.is_null() {
                Status = WarmSocketForBind(FCB, AFD_SHARE_WILDCARD);

                if NT_SUCCESS(Status) {
                    (*FCB).SharedData.State = SOCKET_STATE_BOUND;
                } else {
                    return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
                }
            } else {
                return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
            }

            // Fall through to SOCKET_STATE_BOUND
            if !(*FCB).RemoteAddress.is_null() {
                ExFreePoolWithTag((*FCB).RemoteAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
            }

            (*FCB).RemoteAddress = TaCopyTransportAddress(core::ptr::addr_of_mut!((*ConnectReq).RemoteAddress));

            if (*FCB).RemoteAddress.is_null() {
                Status = STATUS_NO_MEMORY;
            } else {
                Status = WarmSocketForConnection(FCB);

                if NT_SUCCESS(Status) {
                    if !(*FCB).ConnectReturnInfo.is_null() {
                        ExFreePoolWithTag((*FCB).ConnectReturnInfo as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);
                    }

                    Status = TdiBuildConnectionInfo(
                        core::ptr::addr_of_mut!((*FCB).ConnectReturnInfo),
                        core::ptr::addr_of_mut!((*ConnectReq).RemoteAddress),
                    );

                    if NT_SUCCESS(Status) {
                        if !(*FCB).ConnectCallInfo.is_null() {
                            ExFreePoolWithTag((*FCB).ConnectCallInfo as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);
                        }

                        Status = TdiBuildConnectionInfo(
                            core::ptr::addr_of_mut!((*FCB).ConnectCallInfo),
                            core::ptr::addr_of_mut!((*ConnectReq).RemoteAddress),
                        );
                    }

                    if NT_SUCCESS(Status) {
                        (*(*FCB).ConnectCallInfo).UserData = (*FCB).ConnectData;
                        (*(*FCB).ConnectCallInfo).UserDataLength = (*FCB).ConnectDataSize as i32;
                        (*(*FCB).ConnectCallInfo).Options = (*FCB).ConnectOptions;
                        (*(*FCB).ConnectCallInfo).OptionsLength = (*FCB).ConnectOptionsSize as i32;

                        (*FCB).SharedData.State = SOCKET_STATE_CONNECTING;

                        AFD_DbgPrint!(MID_TRACE, b"Queueing IRP %p\n\0".as_ptr() as PCHAR, Irp);
                        Status = QueueUserModeIrp(FCB, Irp, FUNCTION_CONNECT);
                        if Status == STATUS_PENDING {
                            Status = TdiConnect(
                                core::ptr::addr_of_mut!((*FCB).ConnectIrp.InFlightRequest),
                                (*FCB).Connection.Object,
                                (*FCB).ConnectCallInfo,
                                (*FCB).ConnectReturnInfo,
                                Some(StreamSocketConnectComplete),
                                FCB as PVOID,
                            );
                        }

                        if Status != STATUS_PENDING {
                            (*FCB).SharedData.State = SOCKET_STATE_BOUND;
                        }

                        SocketStateUnlock(FCB);

                        return Status;
                    }
                }
            }
        }

        SOCKET_STATE_BOUND => {
            if !(*FCB).RemoteAddress.is_null() {
                ExFreePoolWithTag((*FCB).RemoteAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
            }

            (*FCB).RemoteAddress = TaCopyTransportAddress(core::ptr::addr_of_mut!((*ConnectReq).RemoteAddress));

            if (*FCB).RemoteAddress.is_null() {
                Status = STATUS_NO_MEMORY;
            } else {
                Status = WarmSocketForConnection(FCB);

                if NT_SUCCESS(Status) {
                    if !(*FCB).ConnectReturnInfo.is_null() {
                        ExFreePoolWithTag((*FCB).ConnectReturnInfo as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);
                    }

                    Status = TdiBuildConnectionInfo(
                        core::ptr::addr_of_mut!((*FCB).ConnectReturnInfo),
                        core::ptr::addr_of_mut!((*ConnectReq).RemoteAddress),
                    );

                    if NT_SUCCESS(Status) {
                        if !(*FCB).ConnectCallInfo.is_null() {
                            ExFreePoolWithTag((*FCB).ConnectCallInfo as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);
                        }

                        Status = TdiBuildConnectionInfo(
                            core::ptr::addr_of_mut!((*FCB).ConnectCallInfo),
                            core::ptr::addr_of_mut!((*ConnectReq).RemoteAddress),
                        );
                    }

                    if NT_SUCCESS(Status) {
                        (*(*FCB).ConnectCallInfo).UserData = (*FCB).ConnectData;
                        (*(*FCB).ConnectCallInfo).UserDataLength = (*FCB).ConnectDataSize as i32;
                        (*(*FCB).ConnectCallInfo).Options = (*FCB).ConnectOptions;
                        (*(*FCB).ConnectCallInfo).OptionsLength = (*FCB).ConnectOptionsSize as i32;

                        (*FCB).SharedData.State = SOCKET_STATE_CONNECTING;

                        AFD_DbgPrint!(MID_TRACE, b"Queueing IRP %p\n\0".as_ptr() as PCHAR, Irp);
                        Status = QueueUserModeIrp(FCB, Irp, FUNCTION_CONNECT);
                        if Status == STATUS_PENDING {
                            Status = TdiConnect(
                                core::ptr::addr_of_mut!((*FCB).ConnectIrp.InFlightRequest),
                                (*FCB).Connection.Object,
                                (*FCB).ConnectCallInfo,
                                (*FCB).ConnectReturnInfo,
                                Some(StreamSocketConnectComplete),
                                FCB as PVOID,
                            );
                        }

                        if Status != STATUS_PENDING {
                            (*FCB).SharedData.State = SOCKET_STATE_BOUND;
                        }

                        SocketStateUnlock(FCB);

                        return Status;
                    }
                }
            }
        }

        _ => {
            AFD_DbgPrint!(
                MIN_TRACE,
                b"Inappropriate socket state %u for connect\n\0".as_ptr() as PCHAR,
                (*FCB).SharedData.State,
            );
        }
    }

    return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
}

#[no_mangle]
pub unsafe extern "system" fn AfdStreamSocketSuperConnect(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_INVALID_PARAMETER;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let ConnectReq: PAFD_SUPER_CONNECT_INFO;
    let mut LockMode: KPROCESSOR_MODE = 0;
    let BufferSize: u32 = (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength;

    AFD_DbgPrint!(MID_TRACE, b"Called on %p\n\0".as_ptr() as PCHAR, FCB);

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    ConnectReq = LockRequest(Irp, IrpSp, 0u8, core::ptr::addr_of_mut!(LockMode)) as PAFD_SUPER_CONNECT_INFO;
    if ConnectReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    AFD_DbgPrint!(MID_TRACE, b"Connect request:\n\0".as_ptr() as PCHAR);
    OskitDumpBuffer(
        ConnectReq as PCHAR,
        (*IrpSp).Parameters.DeviceIoControl.InputBufferLength,
    );

    if !(*FCB).OnConnectSendBuffer.is_null() {
        ExFreePoolWithTag((*FCB).OnConnectSendBuffer as PVOID, TAG_AFD_SUPER_CONNECT_BUFFER);
        (*FCB).OnConnectSendBuffer = core::ptr::null_mut();
        (*FCB).OnConnectSendBufferSize = 0;
    }

    if !(*Irp).UserBuffer.is_null() {
        (*FCB).OnConnectSendBufferSize = BufferSize;
        (*FCB).OnConnectSendBuffer = ExAllocatePoolWithTag(
            PagedPool,
            BufferSize as usize,
            TAG_AFD_SUPER_CONNECT_BUFFER,
        ) as PVOID;

        if (*FCB).OnConnectSendBuffer.is_null() {
            return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
        }

        RtlCopyMemory(
            (*FCB).OnConnectSendBuffer,
            (*Irp).UserBuffer,
            BufferSize as usize,
        );
    }

    if ((*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS) != 0 {
        AFD_DbgPrint!(
            MIN_TRACE,
            b"Cannot call IOCTL_CONNECTEX on connectionless socket\n\0".as_ptr() as PCHAR,
            (*FCB).SharedData.State,
        );
        if !(*FCB).RemoteAddress.is_null() {
            ExFreePoolWithTag((*FCB).RemoteAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
        }

        (*FCB).RemoteAddress = TaCopyTransportAddress(core::ptr::addr_of_mut!(
            (*ConnectReq).RemoteAddress
        ));

        if (*FCB).RemoteAddress.is_null() {
            Status = STATUS_NO_MEMORY;
        } else {
            Status = STATUS_SUCCESS;
        }

        return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
    }

    match (*FCB).SharedData.State {
        SOCKET_STATE_CONNECTED => {
            Status = STATUS_SUCCESS;
        }

        SOCKET_STATE_CONNECTING => {
            return LeaveIrpUntilLater(FCB, Irp, FUNCTION_CONNECTEX);
        }

        SOCKET_STATE_BOUND => {
            if !(*FCB).RemoteAddress.is_null() {
                ExFreePoolWithTag((*FCB).RemoteAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
            }

            (*FCB).RemoteAddress = TaCopyTransportAddress(core::ptr::addr_of_mut!(
                (*ConnectReq).RemoteAddress
            ));
            if (*FCB).RemoteAddress.is_null() {
                Status = STATUS_NO_MEMORY;
            } else {
                Status = WarmSocketForConnection(FCB);
                if NT_SUCCESS(Status) {
                    if !(*FCB).ConnectReturnInfo.is_null() {
                        ExFreePoolWithTag(
                            (*FCB).ConnectReturnInfo as PVOID,
                            TAG_AFD_TDI_CONNECTION_INFORMATION,
                        );
                    }

                    Status = TdiBuildConnectionInfo(
                        core::ptr::addr_of_mut!((*FCB).ConnectReturnInfo),
                        core::ptr::addr_of_mut!((*ConnectReq).RemoteAddress),
                    );
                    if NT_SUCCESS(Status) {
                        if !(*FCB).ConnectCallInfo.is_null() {
                            ExFreePoolWithTag(
                                (*FCB).ConnectCallInfo as PVOID,
                                TAG_AFD_TDI_CONNECTION_INFORMATION,
                            );
                        }

                        Status = TdiBuildConnectionInfo(
                            core::ptr::addr_of_mut!((*FCB).ConnectCallInfo),
                            core::ptr::addr_of_mut!((*ConnectReq).RemoteAddress),
                        );

                        if NT_SUCCESS(Status) {
                            (*(*FCB).ConnectCallInfo).UserData = if !(*FCB).ConnectData.is_null() {
                                (*FCB).ConnectData
                            } else {
                                (*FCB).OnConnectSendBuffer
                            };
                            (*(*FCB).ConnectCallInfo).UserDataLength =
                                if !(*FCB).ConnectData.is_null() {
                                    (*FCB).ConnectDataSize as i32
                                } else {
                                    (*FCB).OnConnectSendBufferSize as i32
                                };
                            (*(*FCB).ConnectCallInfo).Options = (*FCB).ConnectOptions;
                            (*(*FCB).ConnectCallInfo).OptionsLength = (*FCB).ConnectOptionsSize as i32;
                            (*FCB).SharedData.State = SOCKET_STATE_CONNECTING;

                            AFD_DbgPrint!(MID_TRACE, b"Queueing IRP %p\n\0".as_ptr() as PCHAR, Irp);
                            Status = QueueUserModeIrp(FCB, Irp, FUNCTION_CONNECTEX);
                            if Status == STATUS_PENDING {
                                Status = TdiConnect(
                                    core::ptr::addr_of_mut!((*FCB).ConnectIrp.InFlightRequest),
                                    (*FCB).Connection.Object,
                                    (*FCB).ConnectCallInfo,
                                    (*FCB).ConnectReturnInfo,
                                    Some(StreamSocketConnectComplete),
                                    FCB as PVOID,
                                );
                            }

                            if Status != STATUS_PENDING {
                                (*FCB).SharedData.State = SOCKET_STATE_BOUND;
                            }

                            SocketStateUnlock(FCB);

                            return Status;
                        }
                    }
                }
            }
        }

        _ => {
            AFD_DbgPrint!(
                MIN_TRACE,
                b"Inappropriate socket state %u for connect\n\0".as_ptr() as PCHAR,
                (*FCB).SharedData.State,
            );
        }
    }

    UnlockAndMaybeComplete(FCB, Status, Irp, 0)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/net/afd/afd/connect.c
//  * PURPOSE:          Ancillary functions driver
//  * PROGRAMMER:       Art Yerkes (ayerkes@speakeasy.net)
//  * UPDATE HISTORY:
//  * 20040708 Created
//  */
//
// #include "afd.h"
//
// NTSTATUS
// NTAPI
// AfdGetConnectOptions(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                      PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     UINT BufferSize = IrpSp->Parameters.DeviceIoControl.OutputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (FCB->ConnectOptionsSize == 0)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Invalid parameter\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
//     }
//
//     ASSERT(FCB->ConnectOptions);
//
//     if (FCB->FilledConnectOptions < BufferSize) BufferSize = FCB->FilledConnectOptions;
//
//     RtlCopyMemory(Irp->UserBuffer,
//                   FCB->ConnectOptions,
//                   BufferSize);
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, BufferSize);
// }
//
// NTSTATUS
// NTAPI
// AfdSetConnectOptions(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                      PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PVOID ConnectOptions = LockRequest(Irp, IrpSp, FALSE, NULL);
//     UINT ConnectOptionsSize = IrpSp->Parameters.DeviceIoControl.InputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!ConnectOptions)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if (FCB->ConnectOptions)
//     {
//         ExFreePoolWithTag(FCB->ConnectOptions, TAG_AFD_CONNECT_OPTIONS);
//         FCB->ConnectOptions = NULL;
//         FCB->ConnectOptionsSize = 0;
//         FCB->FilledConnectOptions = 0;
//     }
//
//     FCB->ConnectOptions = ExAllocatePoolWithTag(PagedPool,
//                                                 ConnectOptionsSize,
//                                                 TAG_AFD_CONNECT_OPTIONS);
//
//     if (!FCB->ConnectOptions)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     RtlCopyMemory(FCB->ConnectOptions,
//                   ConnectOptions,
//                   ConnectOptionsSize);
//
//     FCB->ConnectOptionsSize = ConnectOptionsSize;
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
// }
//
// NTSTATUS
// NTAPI
// AfdSetConnectOptionsSize(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                          PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PUINT ConnectOptionsSize = LockRequest(Irp, IrpSp, FALSE, NULL);
//     UINT BufferSize = IrpSp->Parameters.DeviceIoControl.InputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!ConnectOptionsSize)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if (BufferSize < sizeof(UINT))
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Buffer too small\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, 0);
//     }
//
//     if (FCB->ConnectOptions)
//     {
//         ExFreePoolWithTag(FCB->ConnectOptions, TAG_AFD_CONNECT_OPTIONS);
//         FCB->ConnectOptionsSize = 0;
//         FCB->FilledConnectOptions = 0;
//     }
//
//     FCB->ConnectOptions = ExAllocatePoolWithTag(PagedPool,
//                                                 *ConnectOptionsSize,
//                                                 TAG_AFD_CONNECT_OPTIONS);
//
//     if (!FCB->ConnectOptions) return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     FCB->ConnectOptionsSize = *ConnectOptionsSize;
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
// }
//
// NTSTATUS
// NTAPI
// AfdGetConnectData(PDEVICE_OBJECT DeviceObject, PIRP Irp,
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
//     if (FCB->ConnectDataSize == 0)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Invalid parameter\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
//     }
//
//     ASSERT(FCB->ConnectData);
//
//     if (FCB->FilledConnectData < BufferSize) BufferSize = FCB->FilledConnectData;
//
//     RtlCopyMemory(Irp->UserBuffer,
//                   FCB->ConnectData,
//                   BufferSize);
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, BufferSize);
// }
//
// NTSTATUS
// NTAPI
// AfdSetConnectData(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                   PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PVOID ConnectData = LockRequest(Irp, IrpSp, FALSE, NULL);
//     UINT ConnectDataSize = IrpSp->Parameters.DeviceIoControl.InputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!ConnectData)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if (FCB->ConnectData)
//     {
//         ExFreePoolWithTag(FCB->ConnectData, TAG_AFD_CONNECT_DATA);
//         FCB->ConnectData = NULL;
//         FCB->ConnectDataSize = 0;
//         FCB->FilledConnectData = 0;
//     }
//
//     FCB->ConnectData = ExAllocatePoolWithTag(PagedPool,
//                                              ConnectDataSize,
//                                              TAG_AFD_CONNECT_DATA);
//
//     if (!FCB->ConnectData) return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     RtlCopyMemory(FCB->ConnectData,
//                   ConnectData,
//                   ConnectDataSize);
//
//     FCB->ConnectDataSize = ConnectDataSize;
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
// }
//
// NTSTATUS
// NTAPI
// AfdSetConnectDataSize(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                       PIO_STACK_LOCATION IrpSp)
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PUINT ConnectDataSize = LockRequest(Irp, IrpSp, FALSE, NULL);
//     UINT BufferSize = IrpSp->Parameters.DeviceIoControl.InputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!ConnectDataSize)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if (BufferSize < sizeof(UINT))
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Buffer too small\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, 0);
//     }
//
//     if (FCB->ConnectData)
//     {
//         ExFreePoolWithTag(FCB->ConnectData, TAG_AFD_CONNECT_DATA);
//         FCB->ConnectDataSize = 0;
//         FCB->FilledConnectData = 0;
//     }
//
//     FCB->ConnectData = ExAllocatePoolWithTag(PagedPool,
//                                              *ConnectDataSize,
//                                              TAG_AFD_CONNECT_DATA);
//
//     if (!FCB->ConnectData) return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     FCB->ConnectDataSize = *ConnectDataSize;
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0);
// }
//
//
// NTSTATUS
// WarmSocketForConnection(PAFD_FCB FCB) {
//     NTSTATUS Status;
//
//     if( !FCB->TdiDeviceName.Length || !FCB->TdiDeviceName.Buffer ) {
//         AFD_DbgPrint!(MIN_TRACE,("Null Device\n"));
//         return STATUS_NO_SUCH_DEVICE;
//     }
//
//     Status = TdiOpenConnectionEndpointFile(&FCB->TdiDeviceName,
//                                            &FCB->Connection.Handle,
//                                            &FCB->Connection.Object );
//
//     if( NT_SUCCESS(Status) ) {
//         Status = TdiAssociateAddressFile( FCB->AddressFile.Handle,
//                                           FCB->Connection.Object );
//     }
//
//     return Status;
// }
//
// NTSTATUS
// MakeSocketIntoConnection(PAFD_FCB FCB) {
//     NTSTATUS Status;
//
//     ASSERT(!FCB->Recv.Window);
//     ASSERT(!FCB->Send.Window);
//
//     if (!FCB->Recv.Size)
//     {
//         Status = TdiQueryMaxDatagramLength(FCB->Connection.Object,
//                                            &FCB->Recv.Size);
//         if (!NT_SUCCESS(Status))
//             return Status;
//     }
//
//     if (!FCB->Send.Size)
//     {
//         Status = TdiQueryMaxDatagramLength(FCB->Connection.Object,
//                                            &FCB->Send.Size);
//         if (!NT_SUCCESS(Status))
//             return Status;
//     }
//
//     /* Allocate the receive area and start receiving */
//     if (!FCB->Recv.Window)
//     {
//         FCB->Recv.Window = ExAllocatePoolWithTag(PagedPool,
//                                                  FCB->Recv.Size,
//                                                  TAG_AFD_DATA_BUFFER);
//
//         if( !FCB->Recv.Window ) return STATUS_NO_MEMORY;
//     }
//
//     if (!FCB->Send.Window)
//     {
//         FCB->Send.Window = ExAllocatePoolWithTag(PagedPool,
//                                                  FCB->Send.Size,
//                                                  TAG_AFD_DATA_BUFFER);
//
//         if( !FCB->Send.Window ) return STATUS_NO_MEMORY;
//     }
//
//     FCB->SharedData.State = SOCKET_STATE_CONNECTED;
//     FCB->SharedData.ConnectTime = 0; // Not used
//
//     Status = TdiReceive( &FCB->ReceiveIrp.InFlightRequest,
//                          FCB->Connection.Object,
//                          TDI_RECEIVE_NORMAL,
//                          FCB->Recv.Window,
//                          FCB->Recv.Size,
//                          ReceiveComplete,
//                          FCB );
//
//    if( Status == STATUS_PENDING ) Status = STATUS_SUCCESS;
//
//    FCB->PollState |= AFD_EVENT_CONNECT | AFD_EVENT_SEND;
//    FCB->PollStatus[FD_CONNECT_BIT] = STATUS_SUCCESS;
//    FCB->PollStatus[FD_WRITE_BIT] = STATUS_SUCCESS;
//    PollReeval( FCB->DeviceExt, FCB->FileObject );
//
//    return Status;
// }
//
// static IO_COMPLETION_ROUTINE StreamSocketConnectSendComplete;
//
// static
// NTSTATUS
// NTAPI
// StreamSocketConnectSendComplete(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PVOID Context)
// {
//     PAFD_FCB FCB = (PAFD_FCB)Context;
//     PLIST_ENTRY NextIrpEntry;
//     PIRP NextIrp;
//
//     if (!SocketAcquireStateLock(FCB))
//         return STATUS_FILE_CLOSED;
//
//     ASSERT(FCB->SendIrp.InFlightRequest == Irp);
//     FCB->SendIrp.InFlightRequest = NULL;
//
//     while (!IsListEmpty(&FCB->PendingIrpList[FUNCTION_CONNECTEX as usize]))
//     {
//         NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_CONNECTEX as usize]);
//         NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//
//         AFD_DbgPrint!(MID_TRACE,("Completing connectex %p\n", NextIrp));
//         NextIrp->IoStatus.Status = Irp->IoStatus.Status;
//         NextIrp->IoStatus.Information = Irp->IoStatus.Information;
//         if (NextIrp->MdlAddress)
//             UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
//         (void)IoSetCancelRoutine(NextIrp, NULL);
//         IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
//     }
//
//     if (FCB->OnConnectSendBuffer)
//     {
//         ExFreePoolWithTag(FCB->OnConnectSendBuffer, TAG_AFD_SUPER_CONNECT_BUFFER);
//         FCB->OnConnectSendBuffer = NULL;
//         FCB->OnConnectSendBufferSize = 0;
//     }
//
//     SocketStateUnlock(FCB);
//
//     return STATUS_SUCCESS;
// }
//
// static IO_COMPLETION_ROUTINE StreamSocketConnectComplete;
// static
// NTSTATUS
// NTAPI
// StreamSocketConnectComplete(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                             PVOID Context) {
//     NTSTATUS Status = Irp->IoStatus.Status;
//     PAFD_FCB FCB = (PAFD_FCB)Context;
//     PLIST_ENTRY NextIrpEntry;
//     PIRP NextIrp;
//
//     AFD_DbgPrint!(MID_TRACE,("Called: FCB %p, FO %p\n",
//                             Context, FCB->FileObject));
//
//     /* I was wrong about this before as we can have pending writes to a not
//      * yet connected socket */
//     if( !SocketAcquireStateLock( FCB ) )
//         return STATUS_FILE_CLOSED;
//
//     AFD_DbgPrint!(MID_TRACE,("Irp->IoStatus.Status = %x\n",
//                             Irp->IoStatus.Status));
//
//     ASSERT(FCB->ConnectIrp.InFlightRequest == Irp);
//     FCB->ConnectIrp.InFlightRequest = NULL;
//
//     if( FCB->SharedData.State == SOCKET_STATE_CLOSED ) {
//         /* Cleanup our IRP queue because the FCB is being destroyed */
//         while( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_CONNECT as usize] ) ) {
//                NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_CONNECT as usize]);
//                NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//                NextIrp->IoStatus.Status = STATUS_FILE_CLOSED;
//                NextIrp->IoStatus.Information = 0;
//                if( NextIrp->MdlAddress ) UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//                (void)IoSetCancelRoutine(NextIrp, NULL);
//                IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//         }
//         while (!IsListEmpty(&FCB->PendingIrpList[FUNCTION_CONNECTEX as usize]))
//         {
//                NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_CONNECTEX as usize]);
//                NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//                NextIrp->IoStatus.Status = STATUS_FILE_CLOSED;
//                NextIrp->IoStatus.Information = Irp->IoStatus.Information;
//                if (NextIrp->MdlAddress)
//                    UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
//                (void)IoSetCancelRoutine(NextIrp, NULL);
//                IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
//         }
//         SocketStateUnlock( FCB );
//         return STATUS_FILE_CLOSED;
//     }
//
//     if( !NT_SUCCESS(Irp->IoStatus.Status) ) {
//         FCB->PollState |= AFD_EVENT_CONNECT_FAIL;
//         FCB->PollStatus[FD_CONNECT_BIT] = Irp->IoStatus.Status;
//         AFD_DbgPrint!(MID_TRACE,("Going to bound state\n"));
//         FCB->SharedData.State = SOCKET_STATE_BOUND;
//         PollReeval( FCB->DeviceExt, FCB->FileObject );
//     }
//
//     /* Succeed pending irps on the FUNCTION_CONNECT list */
//     while( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_CONNECT as usize] ) ) {
//         NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_CONNECT as usize]);
//         NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//         AFD_DbgPrint!(MID_TRACE,("Completing connect %p\n", NextIrp));
//         NextIrp->IoStatus.Status = Status;
//         NextIrp->IoStatus.Information = NT_SUCCESS(Status) ? ((ULONG_PTR)FCB->Connection.Handle) : 0;
//         if( NextIrp->MdlAddress ) UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//         (void)IoSetCancelRoutine(NextIrp, NULL);
//         IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//     }
//
//     if( NT_SUCCESS(Status) ) {
//         Status = MakeSocketIntoConnection( FCB );
//
//         if( !NT_SUCCESS(Status) ) {
//             goto end;
//         }
//
//         FCB->FilledConnectData = MIN(FCB->ConnectReturnInfo->UserDataLength, FCB->ConnectDataSize);
//         if (FCB->FilledConnectData)
//         {
//             RtlCopyMemory(FCB->ConnectData,
//                           FCB->ConnectReturnInfo->UserData,
//                           FCB->FilledConnectData);
//         }
//
//         FCB->FilledConnectOptions = MIN(FCB->ConnectReturnInfo->OptionsLength, FCB->ConnectOptionsSize);
//         if (FCB->FilledConnectOptions)
//         {
//             RtlCopyMemory(FCB->ConnectOptions,
//                           FCB->ConnectReturnInfo->Options,
//                           FCB->FilledConnectOptions);
//         }
//
//         if( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_SEND as usize] ) ) {
//             NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_SEND as usize]);
//             NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP,
//                                         Tail.Overlay.ListEntry);
//             AFD_DbgPrint!(MID_TRACE,("Launching send request %p\n", NextIrp));
//             Status = AfdConnectedSocketWriteData
//                 ( DeviceObject,
//                   NextIrp,
//                   IoGetCurrentIrpStackLocation( NextIrp ),
//                   FALSE );
//         }
//
//         if( Status == STATUS_PENDING )
//             Status = STATUS_SUCCESS;
//
//         if (FCB->OnConnectSendBuffer)
//         {
//             PFILE_OBJECT object = FCB->Connection.Object;
//             PVOID sendBuffer = FCB->OnConnectSendBuffer;
//             UINT sendBufferLength = FCB->OnConnectSendBufferSize;
//             SocketStateUnlock(FCB);
//
//             Status = TdiSend(&FCB->SendIrp.InFlightRequest,
//                              object,
//                              AFD_OVERLAPPED,
//                              sendBuffer,
//                              sendBufferLength,
//                              StreamSocketConnectSendComplete,
//                              FCB);
//
//             return Status == STATUS_PENDING ? STATUS_SUCCESS : Status;
//         }
//     }
//
// end:
//     while (!IsListEmpty(&FCB->PendingIrpList[FUNCTION_CONNECTEX as usize]))
//     {
//         NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_CONNECTEX as usize]);
//         NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//
//         AFD_DbgPrint!(MID_TRACE,("Completing connectex %p\n", NextIrp));
//         NextIrp->IoStatus.Status = Status;
//         NextIrp->IoStatus.Information = 0;
//         if (NextIrp->MdlAddress)
//             UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
//         (void)IoSetCancelRoutine(NextIrp, NULL);
//         IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
//     }
//     SocketStateUnlock( FCB );
//
//     AFD_DbgPrint!(MID_TRACE,("Returning %x\n", Status));
//
//     return Status;
// }
//
// /* Return the socket object for ths request only if it is a connected or
//    stream type. */
// NTSTATUS
// NTAPI
// AfdStreamSocketConnect(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                        PIO_STACK_LOCATION IrpSp) {
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PAFD_CONNECT_INFO ConnectReq;
//     AFD_DbgPrint!(MID_TRACE,("Called on %p\n", FCB));
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//     if( !(ConnectReq = LockRequest( Irp, IrpSp, FALSE, NULL )) )
//         return UnlockAndMaybeComplete( FCB, STATUS_NO_MEMORY, Irp,
//                                        0 );
//
//     AFD_DbgPrint!(MID_TRACE,("Connect request:\n"));
//
//    if( FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS )
//    {
//         if (FCB->RemoteAddress)
//         {
//             ExFreePoolWithTag(FCB->RemoteAddress, TAG_AFD_TRANSPORT_ADDRESS);
//         }
//
//         FCB->RemoteAddress =
//             TaCopyTransportAddress( &ConnectReq->RemoteAddress );
//
//         if( !FCB->RemoteAddress )
//             Status = STATUS_NO_MEMORY;
//         else
//             Status = STATUS_SUCCESS;
//
//         return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
//    }
//
//     switch(FCB->SharedData.State)
//     {
//     case SOCKET_STATE_CONNECTED:
//         Status = STATUS_SUCCESS;
//         break;
//
//     case SOCKET_STATE_CONNECTING:
//         return LeaveIrpUntilLater( FCB, Irp, FUNCTION_CONNECT );
//
//     case SOCKET_STATE_CREATED:
//         if (FCB->LocalAddress)
//         {
//             ExFreePoolWithTag(FCB->LocalAddress, TAG_AFD_TRANSPORT_ADDRESS);
//         }
//
//         FCB->LocalAddress =
//             TaBuildNullTransportAddress( ConnectReq->RemoteAddress.Address[0].AddressType );
//
//         if( FCB->LocalAddress ) {
//             Status = WarmSocketForBind( FCB, AFD_SHARE_WILDCARD );
//
//             if( NT_SUCCESS(Status) )
//                 FCB->SharedData.State = SOCKET_STATE_BOUND;
//             else
//                 return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
//         } else
//             return UnlockAndMaybeComplete
//                 ( FCB, STATUS_NO_MEMORY, Irp, 0 );
//
//     /* Drop through to SOCKET_STATE_BOUND */
//
//     case SOCKET_STATE_BOUND:
//         if (FCB->RemoteAddress)
//         {
//             ExFreePoolWithTag(FCB->RemoteAddress, TAG_AFD_TRANSPORT_ADDRESS);
//         }
//
//         FCB->RemoteAddress =
//             TaCopyTransportAddress( &ConnectReq->RemoteAddress );
//
//         if( !FCB->RemoteAddress ) {
//             Status = STATUS_NO_MEMORY;
//             break;
//         }
//
//         Status = WarmSocketForConnection( FCB );
//
//         if( !NT_SUCCESS(Status) )
//             break;
//
//         if (FCB->ConnectReturnInfo)
//         {
//             ExFreePoolWithTag(FCB->ConnectReturnInfo, TAG_AFD_TDI_CONNECTION_INFORMATION);
//         }
//
//         Status = TdiBuildConnectionInfo
//             ( &FCB->ConnectReturnInfo,
//               &ConnectReq->RemoteAddress );
//
//         if( NT_SUCCESS(Status) )
//         {
//             if (FCB->ConnectCallInfo)
//             {
//                 ExFreePoolWithTag(FCB->ConnectCallInfo, TAG_AFD_TDI_CONNECTION_INFORMATION);
//             }
//
//             Status = TdiBuildConnectionInfo(&FCB->ConnectCallInfo,
//                                               &ConnectReq->RemoteAddress);
//         }
//         else break;
//
//
//         if( NT_SUCCESS(Status) ) {
//             FCB->ConnectCallInfo->UserData = FCB->ConnectData;
//             FCB->ConnectCallInfo->UserDataLength = FCB->ConnectDataSize;
//             FCB->ConnectCallInfo->Options = FCB->ConnectOptions;
//             FCB->ConnectCallInfo->OptionsLength = FCB->ConnectOptionsSize;
//
//             FCB->SharedData.State = SOCKET_STATE_CONNECTING;
//
//             AFD_DbgPrint!(MID_TRACE,("Queueing IRP %p\n", Irp));
//             Status = QueueUserModeIrp(FCB, Irp, FUNCTION_CONNECT);
//             if (Status == STATUS_PENDING)
//             {
//                 Status = TdiConnect(&FCB->ConnectIrp.InFlightRequest,
//                                     FCB->Connection.Object,
//                                     FCB->ConnectCallInfo,
//                                     FCB->ConnectReturnInfo,
//                                     StreamSocketConnectComplete,
//                                     FCB);
//             }
//
//             if (Status != STATUS_PENDING)
//                 FCB->SharedData.State = SOCKET_STATE_BOUND;
//
//             SocketStateUnlock(FCB);
//
//             return Status;
//         }
//         break;
//
//     default:
//         AFD_DbgPrint!(MIN_TRACE,("Inappropriate socket state %u for connect\n",
//                                 FCB->SharedData.State));
//         break;
//     }
//
//     return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
// }
//
// /* Return the socket object for ths request only if it is a connected or stream type. */
// NTSTATUS
// NTAPI
// AfdStreamSocketSuperConnect(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     PIO_STACK_LOCATION IrpSp)
// {
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PAFD_SUPER_CONNECT_INFO ConnectReq;
//     KPROCESSOR_MODE LockMode;
//     UINT BufferSize = IrpSp->Parameters.DeviceIoControl.OutputBufferLength;
//     AFD_DbgPrint!(MID_TRACE,("Called on %p\n", FCB));
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB))
//         return LostSocket(Irp);
//
//     ConnectReq = LockRequest(Irp, IrpSp, FALSE, &LockMode);
//     if (!ConnectReq)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     AFD_DbgPrint!(MID_TRACE,("Connect request:\n"));
//     OskitDumpBuffer((PCHAR)ConnectReq,
//                     IrpSp->Parameters.DeviceIoControl.InputBufferLength);
//
//     if (FCB->OnConnectSendBuffer)
//     {
//         ExFreePoolWithTag(FCB->OnConnectSendBuffer, TAG_AFD_SUPER_CONNECT_BUFFER);
//         FCB->OnConnectSendBuffer = NULL;
//         FCB->OnConnectSendBufferSize = 0;
//     }
//
//     if (Irp->UserBuffer)
//     {
//          FCB->OnConnectSendBufferSize = BufferSize;
//          FCB->OnConnectSendBuffer = ExAllocatePoolWithTag(PagedPool,
//                                                         BufferSize,
//                                                         TAG_AFD_SUPER_CONNECT_BUFFER);
//
//          if (!FCB->OnConnectSendBuffer)
//              return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//          RtlCopyMemory(FCB->OnConnectSendBuffer, Irp->UserBuffer, BufferSize);
//     }
//
//     if (FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Cannot call IOCTL_CONNECTEX on connectionless socket\n",
//                                  FCB->SharedData.State));
//          if (FCB->RemoteAddress)
//          {
//              ExFreePoolWithTag(FCB->RemoteAddress, TAG_AFD_TRANSPORT_ADDRESS);
//          }
//
//          FCB->RemoteAddress =
//              TaCopyTransportAddress(&ConnectReq->RemoteAddress);
//
//          if(!FCB->RemoteAddress)
//              Status = STATUS_NO_MEMORY;
//          else
//              Status = STATUS_SUCCESS;
//
//          return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
//     }
//
//     switch (FCB->SharedData.State)
//     {
//     case SOCKET_STATE_CONNECTED:
//         Status = STATUS_SUCCESS;
//         break;
//
//     case SOCKET_STATE_CONNECTING:
//         return LeaveIrpUntilLater( FCB, Irp, FUNCTION_CONNECTEX );
//
//     case SOCKET_STATE_BOUND:
//         if (FCB->RemoteAddress)
//         {
//             ExFreePoolWithTag(FCB->RemoteAddress, TAG_AFD_TRANSPORT_ADDRESS);
//         }
//
//         FCB->RemoteAddress =
//             TaCopyTransportAddress(&ConnectReq->RemoteAddress);
//         if (!FCB->RemoteAddress)
//         {
//             Status = STATUS_NO_MEMORY;
//             break;
//         }
//
//         Status = WarmSocketForConnection(FCB);
//         if (!NT_SUCCESS(Status))
//             break;
//
//         if (FCB->ConnectReturnInfo)
//         {
//             ExFreePoolWithTag(FCB->ConnectReturnInfo, TAG_AFD_TDI_CONNECTION_INFORMATION);
//         }
//
//         Status = TdiBuildConnectionInfo(&FCB->ConnectReturnInfo,
//                                         &ConnectReq->RemoteAddress);
//         if (!NT_SUCCESS(Status))
//             break;
//
//         if (FCB->ConnectCallInfo)
//         {
//             ExFreePoolWithTag(FCB->ConnectCallInfo, TAG_AFD_TDI_CONNECTION_INFORMATION);
//         }
//
//         Status = TdiBuildConnectionInfo(&FCB->ConnectCallInfo,
//                                         &ConnectReq->RemoteAddress);
//
//
//         if (!NT_SUCCESS(Status))
//             break;
//
//         FCB->ConnectCallInfo->UserData = FCB->ConnectData ?  FCB->ConnectData : FCB->OnConnectSendBuffer;
//         FCB->ConnectCallInfo->UserDataLength = FCB->ConnectData ?  FCB->ConnectDataSize : FCB->OnConnectSendBufferSize;
//         FCB->ConnectCallInfo->Options = FCB->ConnectOptions;
//         FCB->ConnectCallInfo->OptionsLength = FCB->ConnectOptionsSize;
//         FCB->SharedData.State = SOCKET_STATE_CONNECTING;
//
//         AFD_DbgPrint!(MID_TRACE,("Queueing IRP %p\n", Irp));
//         Status = QueueUserModeIrp( FCB, Irp, FUNCTION_CONNECTEX );
//         if (Status == STATUS_PENDING)
//         {
//             Status = TdiConnect(&FCB->ConnectIrp.InFlightRequest,
//                                 FCB->Connection.Object,
//                                 FCB->ConnectCallInfo,
//                                 FCB->ConnectReturnInfo,
//                                 StreamSocketConnectComplete,
//                                 FCB );
//         }
//
//         if (Status != STATUS_PENDING)
//             FCB->SharedData.State = SOCKET_STATE_BOUND;
//
//         SocketStateUnlock(FCB);
//
//         return Status;
//
//     default:
//         AFD_DbgPrint!(MIN_TRACE,("Inappropriate socket state %u for connect\n",
//                                 FCB->SharedData.State));
//         break;
//     }
//
//     return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
