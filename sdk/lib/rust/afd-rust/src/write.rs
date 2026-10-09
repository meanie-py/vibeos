use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IRP, KPROCESSOR_MODE, MmMapLockedPages, MmUnmapLockedPages, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSetCancelRoutine};

extern "C" {
    pub fn GetLockedData(Irp: PIRP, IrpSp: PIO_STACK_LOCATION) -> PVOID;
    pub fn PollReeval(DeviceObject: PAFD_DEVICE_EXTENSION, FileObject: PFILE_OBJECT);
    fn SocketAcquireStateLock(fcb: PAFD_FCB) -> BOOLEAN;
    fn SocketStateUnlock(fcb: PAFD_FCB);
    fn UnlockBuffers(buffer_array: PAFD_WSABUF, buffer_count: u32, write: u32);
    fn UnlockRequest(irp: PIRP, stack: PIO_STACK_LOCATION);
    fn LeaveIrpUntilLater(fcb: PAFD_FCB, irp: PIRP, function: u32) -> NTSTATUS;
    pub fn LockBuffers( Buf: PAFD_WSABUF, Count: UINT, AddressBuf: PVOID, AddressLen: PINT, Write: BOOLEAN, LockAddress: BOOLEAN, LockMode: KPROCESSOR_MODE, ) -> PAFD_WSABUF;
    pub fn LockRequest( Irp: PIRP, IrpSp: PIO_STACK_LOCATION, Output: BOOLEAN, LockMode: *mut KPROCESSOR_MODE, ) -> PVOID;
    fn LostSocket(irp: PIRP) -> NTSTATUS;
    fn QueueUserModeIrp(fcb: PAFD_FCB, irp: PIRP, function: u32) -> NTSTATUS;
    fn TdiBuildConnectionInfo(out: *mut PTDI_CONNECTION_INFORMATION, addr: PTRANSPORT_ADDRESS) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
unsafe extern "system" fn SendComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    send_complete_impl(DeviceObject, Irp, Context)
}

unsafe fn send_complete_impl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    let FCB = Context as PAFD_FCB;
    let mut NextIrpEntry: PLIST_ENTRY;
    let mut NextIrp: PIRP;
    let mut NextIrpSp: PIO_STACK_LOCATION;
    let mut SendReq: PAFD_SEND_INFO;
    let Map: PAFD_MAPBUF;
    let mut TotalBytesCopied: SIZE_T;
    let mut SpaceAvail: SIZE_T;
    let mut i: SIZE_T;
    let mut SendLength: UINT;
    let mut BytesCopied: UINT;
    let mut HaltSendQueue: BOOLEAN;

    UNREFERENCED_PARAMETER(DeviceObject as PVOID);

    AFD_DbgPrint!(
        MID_TRACE,
        b"Called, status %x, %u bytes used\n\0".as_ptr() as PCHAR,
        (*Irp).IoStatus.__bindgen_anon_1.Status,
        (*Irp).IoStatus.Information,
    );

    if SocketAcquireStateLock(FCB) == 0 {
        return STATUS_FILE_CLOSED;
    }

    ASSERT!((*FCB).SendIrp.InFlightRequest == Irp);
    (*FCB).SendIrp.InFlightRequest = core::ptr::null_mut();

    if (*FCB).SharedData.State == SOCKET_STATE_CLOSED {
        while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize])) {
            NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize]));
            NextIrp = (NextIrpEntry as usize - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP;
            NextIrpSp = IoGetCurrentIrpStackLocation(NextIrp);
            SendReq = (GetLockedData(NextIrp, NextIrpSp)) as _;
            (*NextIrp).IoStatus.__bindgen_anon_1.Status = STATUS_FILE_CLOSED;
            (*NextIrp).IoStatus.Information = 0;
            UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, (0u8) as _);
            if !(*NextIrp).MdlAddress.is_null() {
                UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
            }
            IoSetCancelRoutine(NextIrp, None);
            IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
        }

        RetryDisconnectCompletion(FCB);

        SocketStateUnlock(FCB);
        return STATUS_FILE_CLOSED;
    }

    if !NT_SUCCESS(Status) {
        while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize])) {
            NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize]));
            NextIrp = (NextIrpEntry as usize - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP;
            NextIrpSp = IoGetCurrentIrpStackLocation(NextIrp);
            SendReq = (GetLockedData(NextIrp, NextIrpSp)) as _;

            UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, (0u8) as _);

            (*NextIrp).IoStatus.__bindgen_anon_1.Status = Status;
            (*NextIrp).IoStatus.Information = 0;

            if !(*NextIrp).MdlAddress.is_null() {
                UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
            }
            IoSetCancelRoutine(NextIrp, None);
            IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
        }

        RetryDisconnectCompletion(FCB);

        SocketStateUnlock(FCB);

        return STATUS_SUCCESS;
    }

    core::ptr::copy(
        (*FCB).Send.Window.offset((*Irp).IoStatus.Information as isize),
        (*FCB).Send.Window,
        ((*FCB).Send.BytesUsed as usize) - ((*Irp).IoStatus.Information as usize),
    );

    SendLength = (*Irp).IoStatus.Information as UINT;
    HaltSendQueue = 0u8;
    while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize])) && SendLength > 0 {
        NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize]));
        NextIrp = (NextIrpEntry as usize - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP;
        NextIrpSp = IoGetCurrentIrpStackLocation(NextIrp);
        SendReq = (GetLockedData(NextIrp, NextIrpSp)) as _;

        TotalBytesCopied = (*NextIrp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[3] as SIZE_T;
        ASSERT!(TotalBytesCopied != 0);

        if TotalBytesCopied > SendLength as SIZE_T {
            TotalBytesCopied -= SendLength as SIZE_T;
            (*NextIrp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[3] = TotalBytesCopied as PVOID;

            (*FCB).Send.BytesUsed -= SendLength;
            SendLength = 0;

            InsertHeadList(
                core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize]),
                core::ptr::addr_of_mut!((*NextIrp).Tail.Overlay.__bindgen_anon_2.ListEntry),
            );
            HaltSendQueue = 1u8;
            break;
        }

        ASSERT!((*NextIrp).IoStatus.Information != 0);

        (*NextIrp).IoStatus.__bindgen_anon_1.Status = (*Irp).IoStatus.__bindgen_anon_1.Status;

        (*FCB).Send.BytesUsed -= TotalBytesCopied as u32;
        SendLength -= TotalBytesCopied as UINT;

        IoSetCancelRoutine(NextIrp, None);

        UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, (0u8) as _);

        if !(*NextIrp).MdlAddress.is_null() {
            UnlockRequest(NextIrp, NextIrpSp);
        }

        IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
    }

    ASSERT!(SendLength == 0);

    if HaltSendQueue == 0 && !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize])) {
        NextIrpEntry = (*FCB).PendingIrpList[FUNCTION_SEND as usize].Flink;
        NextIrp = (NextIrpEntry as usize - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP;
        NextIrpSp = IoGetCurrentIrpStackLocation(NextIrp);
        SendReq = (GetLockedData(NextIrp, NextIrpSp)) as _;
        Map = ((*SendReq).BufferArray as usize + (*SendReq).BufferCount as usize * core::mem::size_of::<AFD_WSABUF>()) as PAFD_MAPBUF;

        AFD_DbgPrint!(MID_TRACE, b"SendReq @ %p\n\0".as_ptr() as PCHAR, SendReq);

        SpaceAvail = ((*FCB).Send.Size - (*FCB).Send.BytesUsed) as SIZE_T;
        TotalBytesCopied = 0;

        SendLength = 0;
        i = 0;
        while i < (*SendReq).BufferCount as SIZE_T {
            SendLength += (*(*SendReq).BufferArray.offset(i as isize)).len;
            i += 1;
        }

        if SendLength as SIZE_T > SpaceAvail {
            if SendLength as SIZE_T <= (*FCB).Send.Size as SIZE_T
                && ((*SendReq).AfdFlags & AFD_IMMEDIATE) == 0
                && (*FCB).NonBlocking == 0
            {
                (*FCB).PollState &= !AFD_EVENT_SEND;

                NextIrp = core::ptr::null_mut();
            }

            if SpaceAvail == 0 {
                (*FCB).PollState &= !AFD_EVENT_SEND;

                ASSERT!(((*SendReq).AfdFlags & AFD_OVERLAPPED) != 0);

                NextIrp = core::ptr::null_mut();
            }
        }

        if !NextIrp.is_null() {
            i = 0;
            while i < (*SendReq).BufferCount as SIZE_T {
                BytesCopied = MIN((*(*SendReq).BufferArray.offset(i as isize)).len, SpaceAvail as UINT);

                (*Map.offset(i as isize)).BufferAddress =
                    MmMapLockedPages((*Map.offset(i as isize)).Mdl, 0i32 as KPROCESSOR_MODE);

                RtlCopyMemory(
                    (*FCB).Send.Window.offset((*FCB).Send.BytesUsed as isize) as PVOID,
                    (*Map.offset(i as isize)).BufferAddress,
                    BytesCopied as SIZE_T,
                );

                MmUnmapLockedPages(
                    (*Map.offset(i as isize)).BufferAddress,
                    (*Map.offset(i as isize)).Mdl,
                );

                TotalBytesCopied += BytesCopied as SIZE_T;
                SpaceAvail -= BytesCopied as SIZE_T;
                (*FCB).Send.BytesUsed += BytesCopied;
                i += 1;
            }

            (*NextIrp).IoStatus.Information = TotalBytesCopied as _;
            (*NextIrp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[3] =
                (*NextIrp).IoStatus.Information as PVOID;
        }
    }

    if (*FCB).Send.Size - (*FCB).Send.BytesUsed != 0
        && (*FCB).SendClosed == 0
        && IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize]))
    {
        (*FCB).PollState |= AFD_EVENT_SEND;
        (*FCB).PollStatus[FD_WRITE_BIT as usize] = STATUS_SUCCESS;
        PollReeval(((*FCB).DeviceExt as PVOID) as _, (*FCB).FileObject);
    } else {
        (*FCB).PollState &= !AFD_EVENT_SEND;
    }

    if (*FCB).Send.BytesUsed != 0 {
        let _Status = TdiSend( // nolint: let-shadow (C never reads Status; returns STATUS_SUCCESS)
            core::ptr::addr_of_mut!((*FCB).SendIrp.InFlightRequest),
            (*FCB).Connection.Object,
            0,
            (*FCB).Send.Window,
            (*FCB).Send.BytesUsed as ULONG,
            Some(SendComplete),
            FCB as PVOID,
        );
    } else {
        RetryDisconnectCompletion(FCB);
    }

    SocketStateUnlock(FCB);

    STATUS_SUCCESS
}
unsafe extern "system" fn PacketSocketSendComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    packet_socket_send_complete_impl(DeviceObject, Irp, Context)
}

unsafe fn packet_socket_send_complete_impl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let FCB = Context as PAFD_FCB;
    let mut NextIrpEntry: PLIST_ENTRY;
    let mut NextIrp: PIRP;
    let mut SendReq: PAFD_SEND_INFO;

    UNREFERENCED_PARAMETER(DeviceObject as PVOID);

    AFD_DbgPrint!(
        MID_TRACE,
        b"Called, status %x, %u bytes used\n\0".as_ptr(),
        (*Irp).IoStatus.__bindgen_anon_1.Status,
        (*Irp).IoStatus.Information,
    );

    if SocketAcquireStateLock(FCB) == 0 {
        return STATUS_FILE_CLOSED;
    }

    (*FCB).SendIrp.InFlightRequest = core::ptr::null_mut();

    if (*FCB).SharedData.State == SOCKET_STATE_CLOSED {
        while !IsListEmpty(&mut (*FCB).PendingIrpList[FUNCTION_SEND as usize]) {
            NextIrpEntry = RemoveHeadList(&mut (*FCB).PendingIrpList[FUNCTION_SEND as usize]);
            NextIrp = (NextIrpEntry as *mut u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;
            SendReq = (GetLockedData(NextIrp, IoGetCurrentIrpStackLocation(NextIrp))) as _;
            (*NextIrp).IoStatus.__bindgen_anon_1.Status = STATUS_FILE_CLOSED;
            (*NextIrp).IoStatus.Information = 0;
            IoSetCancelRoutine(NextIrp, None);
            UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, (0u8) as _);
            UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
            IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
        }
        SocketStateUnlock(FCB);
        return STATUS_FILE_CLOSED;
    }

    NextIrpEntry = RemoveHeadList(&mut (*FCB).PendingIrpList[FUNCTION_SEND as usize]);
    NextIrp = (NextIrpEntry as *mut u8).offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;

    SendReq = (GetLockedData(NextIrp, IoGetCurrentIrpStackLocation(NextIrp))) as _;

    (*NextIrp).IoStatus.__bindgen_anon_1.Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    (*NextIrp).IoStatus.Information = (*Irp).IoStatus.Information;

    IoSetCancelRoutine(NextIrp, None);

    UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, (0u8) as _);

    UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));

    IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);

    (*FCB).PollState |= AFD_EVENT_SEND;
    (*FCB).PollStatus[FD_WRITE_BIT as usize] = STATUS_SUCCESS;
    PollReeval(((*FCB).DeviceExt as PVOID) as _, (*FCB).FileObject);

    SocketStateUnlock(FCB);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn AfdConnectedSocketWriteData(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
    Short: u8,
) -> NTSTATUS {

    let mut Status: NTSTATUS;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let SendReq: PAFD_SEND_INFO;
    let mut TotalBytesCopied: u32 = 0;
    let mut i: u32;
    let mut SpaceAvail: u32;
    let mut BytesCopied: u32;
    let mut SendLength: u32;
    let mut LockMode: KPROCESSOR_MODE = 0;

    UNREFERENCED_PARAMETER(DeviceObject as PVOID);
    UNREFERENCED_PARAMETER(&Short as *const u8 as PVOID);

    AFD_DbgPrint!(MID_TRACE, b"Called on %p\n\0".as_ptr() as PCHAR, FCB);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    (*FCB).EventSelectDisabled &= !AFD_EVENT_SEND;

    if ((*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS) != 0 {
        let SendReq: PAFD_SEND_INFO_UDP;
        let mut TargetAddress: PTDI_CONNECTION_INFORMATION = core::ptr::null_mut();

        if (*FCB).SharedData.State != SOCKET_STATE_BOUND || (*FCB).RemoteAddress.is_null() {
            AFD_DbgPrint!(MIN_TRACE, b"Invalid parameter\n\0".as_ptr() as PCHAR);
            return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
        }

        SendReq = LockRequest(Irp, IrpSp, 0u8, &mut LockMode) as PAFD_SEND_INFO_UDP;
        if SendReq.is_null() {
            return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
        }

        (*SendReq).BufferArray = LockBuffers(
            (*SendReq).BufferArray,
            (*SendReq).BufferCount,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0u8,
            0u8,
            LockMode,
        );

        if (*SendReq).BufferArray.is_null() {
            return UnlockAndMaybeComplete(FCB, STATUS_ACCESS_VIOLATION, Irp, 0);
        }

        Status = TdiBuildConnectionInfo(&mut TargetAddress, (*FCB).RemoteAddress);

        if NT_SUCCESS(Status) {
            (*FCB).PollState &= !AFD_EVENT_SEND;

            Status = QueueUserModeIrp(FCB, Irp, FUNCTION_SEND);
            if Status == STATUS_PENDING {
                if (*SendReq).BufferCount > 1 {
                    AFD_DbgPrint!(
                        MIN_TRACE,
                        b"WARN: More than one buffer %ld\n\0".as_ptr() as PCHAR,
                        (*SendReq).BufferCount,
                    );
                }
                Status = TdiSendDatagram(
                    core::ptr::addr_of_mut!((*FCB).SendIrp.InFlightRequest),
                    (*FCB).AddressFile.Object,
                    (*(*SendReq).BufferArray.offset(0)).buf,
                    (*(*SendReq).BufferArray.offset(0)).len,
                    TargetAddress,
                    Some(PacketSocketSendComplete),
                    FCB as PVOID,
                );
                if Status != STATUS_PENDING {
                    NT_VERIFY((RemoveHeadList(
                        core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize]),
                    ) == core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry))
                        as u8);
                    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                    (*Irp).IoStatus.Information = if Status == STATUS_SUCCESS {
                        (*(*SendReq).BufferArray.offset(0)).len as _
                    } else {
                        0
                    };
                    IoSetCancelRoutine(Irp, None);
                    UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, (0u8) as _);
                    UnlockRequest(Irp, IoGetCurrentIrpStackLocation(Irp));
                    IoCompleteRequest(Irp, IO_NETWORK_INCREMENT as i8);
                }
            }

            ExFreePoolWithTag(TargetAddress as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);

            SocketStateUnlock(FCB);

            return STATUS_PENDING;
        } else {
            UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, (0u8) as _);
            return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
        }
    }

    if ((*FCB).PollState & AFD_EVENT_CLOSE) != 0 {
        AFD_DbgPrint!(MIN_TRACE, b"Connection reset by remote peer\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, (*FCB).PollStatus[FD_CLOSE_BIT as usize], Irp, 0);
    }

    if ((*FCB).PollState & AFD_EVENT_ABORT) != 0 {
        AFD_DbgPrint!(MIN_TRACE, b"Connection aborted\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, (*FCB).PollStatus[FD_CLOSE_BIT as usize], Irp, 0);
    }

    if (*FCB).SendClosed != 0 {
        AFD_DbgPrint!(MIN_TRACE, b"No more sends\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_FILE_CLOSED, Irp, 0);
    }

    SendReq = (LockRequest(Irp, IrpSp, 0u8, &mut LockMode)) as _;
    if SendReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    (*SendReq).BufferArray = LockBuffers(
        (*SendReq).BufferArray,
        (*SendReq).BufferCount,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0u8,
        0u8,
        LockMode,
    );

    if (*SendReq).BufferArray.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_ACCESS_VIOLATION, Irp, 0);
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"Socket state %u\n\0".as_ptr() as PCHAR,
        (*FCB).SharedData.State,
    );

    if (*FCB).SharedData.State != SOCKET_STATE_CONNECTED {
        AFD_DbgPrint!(MIN_TRACE, b"Socket not connected\n\0".as_ptr() as PCHAR);
        UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, (0u8) as _);
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_CONNECTION, Irp, 0);
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"FCB->Send.BytesUsed = %u\n\0".as_ptr() as PCHAR,
        (*FCB).Send.BytesUsed,
    );

    SpaceAvail = (*FCB).Send.Size - (*FCB).Send.BytesUsed;

    AFD_DbgPrint!(
        MID_TRACE,
        b"We can accept %u bytes\n\0".as_ptr() as PCHAR,
        SpaceAvail,
    );

    SendLength = 0;
    i = 0;
    while i < (*SendReq).BufferCount {
        SendLength += (*(*SendReq).BufferArray.offset(i as isize)).len;
        i += 1;
    }

    if SendLength > SpaceAvail {
        if SendLength <= (*FCB).Send.Size
            && !(((*SendReq).AfdFlags & AFD_IMMEDIATE) != 0 || (*FCB).NonBlocking != 0)
        {
            (*FCB).PollState &= !AFD_EVENT_SEND;
            return LeaveIrpUntilLater(FCB, Irp, FUNCTION_SEND);
        }

        if SpaceAvail == 0 {
            (*FCB).PollState &= !AFD_EVENT_SEND;

            if ((*SendReq).AfdFlags & AFD_OVERLAPPED) == 0 {
                UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, (0u8) as _);
                return UnlockAndMaybeComplete(FCB, STATUS_CANT_WAIT, Irp, 0);
            } else {
                return LeaveIrpUntilLater(FCB, Irp, FUNCTION_SEND);
            }
        }
    }

    i = 0;
    while SpaceAvail > 0 && i < (*SendReq).BufferCount {
        BytesCopied = MIN((*(*SendReq).BufferArray.offset(i as isize)).len, SpaceAvail);

        AFD_DbgPrint!(
            MID_TRACE,
            b"Copying Buffer %u, %p:%u to %p\n\0".as_ptr() as PCHAR,
            i,
            (*(*SendReq).BufferArray.offset(i as isize)).buf,
            BytesCopied,
            (*FCB).Send.Window.offset((*FCB).Send.BytesUsed as isize),
        );

        RtlCopyMemory(
            (*FCB).Send.Window.offset((*FCB).Send.BytesUsed as isize) as PVOID,
            (*(*SendReq).BufferArray.offset(i as isize)).buf as *const core::ffi::c_void,
            BytesCopied as usize,
        );

        TotalBytesCopied += BytesCopied;
        SpaceAvail -= BytesCopied;
        (*FCB).Send.BytesUsed += BytesCopied;

        i += 1;
    }

    (*Irp).IoStatus.Information = TotalBytesCopied as _;

    if TotalBytesCopied == 0 {
        AFD_DbgPrint!(MIN_TRACE, b"Empty send\n\0".as_ptr() as PCHAR);
        UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, (0u8) as _);
        return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, TotalBytesCopied);
    }

    if SpaceAvail != 0 {
        (*FCB).PollState |= AFD_EVENT_SEND;
        (*FCB).PollStatus[FD_WRITE_BIT as usize] = STATUS_SUCCESS;
        PollReeval(((*FCB).DeviceExt as PVOID) as _, (*FCB).FileObject);
    } else {
        (*FCB).PollState &= !AFD_EVENT_SEND;
    }

    (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[3] = (*Irp).IoStatus.Information as PVOID;

    Status = QueueUserModeIrp(FCB, Irp, FUNCTION_SEND);
    if Status == STATUS_PENDING && (*FCB).SendIrp.InFlightRequest.is_null() {
        TdiSend(
            core::ptr::addr_of_mut!((*FCB).SendIrp.InFlightRequest),
            (*FCB).Connection.Object,
            0,
            (*FCB).Send.Window,
            (*FCB).Send.BytesUsed,
            Some(SendComplete),
            FCB as PVOID,
        );
    }

    SocketStateUnlock(FCB);

    STATUS_PENDING
}
#[no_mangle]
pub unsafe extern "system" fn AfdPacketSocketWriteData(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut TargetAddress: PTDI_CONNECTION_INFORMATION = core::ptr::null_mut();
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let SendReq: PAFD_SEND_INFO_UDP;
    let mut LockMode: KPROCESSOR_MODE = 0;

    UNREFERENCED_PARAMETER(DeviceObject as PVOID);

    AFD_DbgPrint!(MID_TRACE, b"Called on %p\n\0".as_ptr() as PCHAR, FCB);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    (*FCB).EventSelectDisabled &= !AFD_EVENT_SEND;

    if (*FCB).SharedData.State != SOCKET_STATE_BOUND &&
       (*FCB).SharedData.State != SOCKET_STATE_CREATED
    {
        AFD_DbgPrint!(MIN_TRACE, b"Invalid socket state\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
    }

    if (*FCB).SendClosed != 0 {
        AFD_DbgPrint!(MIN_TRACE, b"No more sends\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_FILE_CLOSED, Irp, 0);
    }

    SendReq = LockRequest(Irp, IrpSp, 0, core::ptr::addr_of_mut!(LockMode)) as PAFD_SEND_INFO_UDP;
    if SendReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if (*FCB).SharedData.State == SOCKET_STATE_CREATED {
        if !(*FCB).LocalAddress.is_null() {
            ExFreePoolWithTag((*FCB).LocalAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
        }

        (*FCB).LocalAddress = TaBuildNullTransportAddress(
            (*((*SendReq).TdiConnection.RemoteAddress as PTRANSPORT_ADDRESS))
                .Address[0]
                .AddressType as u32
        );

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
    }

    (*SendReq).BufferArray = LockBuffers(
        (*SendReq).BufferArray,
        (*SendReq).BufferCount,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        0,
        LockMode
    );

    if (*SendReq).BufferArray.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_ACCESS_VIOLATION, Irp, 0);
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"RemoteAddress #%d Type %u\n\0".as_ptr() as PCHAR,
        (*((*SendReq).TdiConnection.RemoteAddress as PTRANSPORT_ADDRESS)).TAAddressCount,
        (*((*SendReq).TdiConnection.RemoteAddress as PTRANSPORT_ADDRESS)).Address[0].AddressType
    );

    Status = TdiBuildConnectionInfo(
        core::ptr::addr_of_mut!(TargetAddress),
        (*SendReq).TdiConnection.RemoteAddress as PTRANSPORT_ADDRESS
    );

    if NT_SUCCESS(Status) {
        (*FCB).PollState &= !AFD_EVENT_SEND;

        Status = QueueUserModeIrp(FCB, Irp, FUNCTION_SEND);
        if Status == STATUS_PENDING {
            if (*SendReq).BufferCount > 1 {
                AFD_DbgPrint!(MIN_TRACE, b"WARN: More than one buffer %ld\n\0".as_ptr() as PCHAR, (*SendReq).BufferCount);
            }
            Status = TdiSendDatagram(
                core::ptr::addr_of_mut!((*FCB).SendIrp.InFlightRequest),
                (*FCB).AddressFile.Object,
                (*(*SendReq).BufferArray.offset(0)).buf,
                (*(*SendReq).BufferArray.offset(0)).len,
                TargetAddress,
                Some(PacketSocketSendComplete),
                FCB as PVOID
            );
            if Status != STATUS_PENDING {
                NT_VERIFY((RemoveHeadList((*FCB).PendingIrpList.as_mut_ptr().offset(FUNCTION_SEND as isize)) == core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry) as PLIST_ENTRY) as u8);
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                (*Irp).IoStatus.Information = if Status == STATUS_SUCCESS {
                    (*(*SendReq).BufferArray.offset(0)).len as _
                } else {
                    0
                };
                IoSetCancelRoutine(Irp, None);
                UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, 0);
                UnlockRequest(Irp, IoGetCurrentIrpStackLocation(Irp));
                IoCompleteRequest(Irp, IO_NETWORK_INCREMENT as CCHAR);
            }
        }

        ExFreePoolWithTag(TargetAddress as PVOID, TAG_AFD_TDI_CONNECTION_INFORMATION);

        SocketStateUnlock(FCB);

        return STATUS_PENDING;
    } else {
        UnlockBuffers((*SendReq).BufferArray, (*SendReq).BufferCount, 0);
        return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/net/afd/afd/write.c
//  * PURPOSE:          Ancillary functions driver
//  * PROGRAMMER:       Art Yerkes (ayerkes@speakeasy.net)
//  * UPDATE HISTORY:
//  * 20040708 Created
//  */
//
// #include "afd.h"
//
// static IO_COMPLETION_ROUTINE SendComplete;
// static NTSTATUS NTAPI SendComplete
// ( PDEVICE_OBJECT DeviceObject,
//   PIRP Irp,
//   PVOID Context ) {
//     NTSTATUS Status = Irp->IoStatus.Status;
//     PAFD_FCB FCB = (PAFD_FCB)Context;
//     PLIST_ENTRY NextIrpEntry;
//     PIRP NextIrp = NULL;
//     PIO_STACK_LOCATION NextIrpSp;
//     PAFD_SEND_INFO SendReq = NULL;
//     PAFD_MAPBUF Map;
//     SIZE_T TotalBytesCopied = 0, TotalBytesProcessed = 0, SpaceAvail, i;
//     UINT SendLength, BytesCopied;
//     BOOLEAN HaltSendQueue;
//
//     UNREFERENCED_PARAMETER(DeviceObject as PVOID);
//
//     /*
//      * The Irp parameter passed in is the IRP of the stream between AFD and
//      * TDI driver. It's not very useful to us. We need the IRPs of the stream
//      * between usermode and AFD. Those are chained from
//      * FCB->PendingIrpList[FUNCTION_SEND as usize] and you'll see them in the code
//      * below as "NextIrp" ('cause they are the next usermode IRP to be
//      * processed).
//      */
//
//     AFD_DbgPrint!(MID_TRACE,("Called, status %x, %u bytes used\n",
//                             Irp->IoStatus.Status,
//                             Irp->IoStatus.Information));
//
//     if( !SocketAcquireStateLock( FCB ) )
//         return STATUS_FILE_CLOSED;
//
//     ASSERT(FCB->SendIrp.InFlightRequest == Irp);
//     FCB->SendIrp.InFlightRequest = NULL;
//     /* Request is not in flight any longer */
//
//     if( FCB->SharedData.State == SOCKET_STATE_CLOSED ) {
//         /* Cleanup our IRP queue because the FCB is being destroyed */
//         while( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_SEND as usize] ) ) {
//             NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_SEND as usize]);
//             NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//             NextIrpSp = IoGetCurrentIrpStackLocation( NextIrp );
//             SendReq = GetLockedData(NextIrp, NextIrpSp);
//             NextIrp->IoStatus.Status = STATUS_FILE_CLOSED;
//             NextIrp->IoStatus.Information = 0;
//             UnlockBuffers(SendReq->BufferArray, SendReq->BufferCount, FALSE);
//             if( NextIrp->MdlAddress ) UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//             (void)IoSetCancelRoutine(NextIrp, NULL);
//             IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//         }
//
//         RetryDisconnectCompletion(FCB);
//
//         SocketStateUnlock( FCB );
//         return STATUS_FILE_CLOSED;
//     }
//
//     if( !NT_SUCCESS(Status) ) {
//         /* Complete all following send IRPs with error */
//
//         while( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_SEND as usize] ) ) {
//             NextIrpEntry =
//                 RemoveHeadList(&FCB->PendingIrpList[FUNCTION_SEND as usize]);
//             NextIrp =
//                 CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//             NextIrpSp = IoGetCurrentIrpStackLocation( NextIrp );
//             SendReq = GetLockedData(NextIrp, NextIrpSp);
//
//             UnlockBuffers( SendReq->BufferArray,
//                            SendReq->BufferCount,
//                            FALSE );
//
//             NextIrp->IoStatus.Status = Status;
//             NextIrp->IoStatus.Information = 0;
//
//             if ( NextIrp->MdlAddress ) UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//             (void)IoSetCancelRoutine(NextIrp, NULL);
//             IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//         }
//
//         RetryDisconnectCompletion(FCB);
//
//         SocketStateUnlock( FCB );
//
//         return STATUS_SUCCESS;
//     }
//
//     RtlMoveMemory( FCB->Send.Window,
//                    FCB->Send.Window + Irp->IoStatus.Information,
//                    FCB->Send.BytesUsed - Irp->IoStatus.Information );
//
//     TotalBytesProcessed = 0;
//     SendLength = Irp->IoStatus.Information;
//     HaltSendQueue = FALSE;
//     while (!IsListEmpty(&FCB->PendingIrpList[FUNCTION_SEND as usize]) && SendLength > 0) {
//         NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_SEND as usize]);
//         NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//         NextIrpSp = IoGetCurrentIrpStackLocation( NextIrp );
//         SendReq = GetLockedData(NextIrp, NextIrpSp);
//         Map = (PAFD_MAPBUF)(SendReq->BufferArray + SendReq->BufferCount);
//
//         TotalBytesCopied = (ULONG_PTR)NextIrp->Tail.Overlay.DriverContext[3];
//         ASSERT(TotalBytesCopied != 0);
//
//         /* If we didn't get enough, keep waiting */
//         if (TotalBytesCopied > SendLength)
//         {
//             /* Update the bytes left to copy */
//             TotalBytesCopied -= SendLength;
//             NextIrp->Tail.Overlay.DriverContext[3] = (PVOID)TotalBytesCopied;
//
//             /* Update the state variables */
//             FCB->Send.BytesUsed -= SendLength;
//             TotalBytesProcessed += SendLength;
//             SendLength = 0;
//
//             /* Pend the IRP */
//             InsertHeadList(&FCB->PendingIrpList[FUNCTION_SEND as usize],
//                            &NextIrp->Tail.Overlay.ListEntry);
//             HaltSendQueue = TRUE;
//             break;
//         }
//
//         ASSERT(NextIrp->IoStatus.Information != 0);
//
//         NextIrp->IoStatus.Status = Irp->IoStatus.Status;
//
//         FCB->Send.BytesUsed -= TotalBytesCopied;
//         TotalBytesProcessed += TotalBytesCopied;
//         SendLength -= TotalBytesCopied;
//
//         (void)IoSetCancelRoutine(NextIrp, NULL);
//
//         UnlockBuffers( SendReq->BufferArray,
//                        SendReq->BufferCount,
//                        FALSE );
//
//         if (NextIrp->MdlAddress) UnlockRequest(NextIrp, NextIrpSp);
//
//         IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
//     }
//
//     ASSERT(SendLength == 0);
//
//    if ( !HaltSendQueue && !IsListEmpty( &FCB->PendingIrpList[FUNCTION_SEND as usize] ) ) {
//         NextIrpEntry = FCB->PendingIrpList[FUNCTION_SEND as usize].Flink;
//         NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//         NextIrpSp = IoGetCurrentIrpStackLocation( NextIrp );
//         SendReq = GetLockedData(NextIrp, NextIrpSp);
//         Map = (PAFD_MAPBUF)(SendReq->BufferArray + SendReq->BufferCount);
//
//         AFD_DbgPrint!(MID_TRACE,("SendReq @ %p\n", SendReq));
//
//         SpaceAvail = FCB->Send.Size - FCB->Send.BytesUsed;
//         TotalBytesCopied = 0;
//
//         /* Count the total transfer size */
//         SendLength = 0;
//         for (i = 0; i < SendReq->BufferCount; i++)
//         {
//             SendLength += SendReq->BufferArray[i].len;
//         }
//
//         /* Make sure we've got the space */
//         if (SendLength > SpaceAvail)
//         {
//            /* Blocking sockets have to wait here */
//            if (SendLength <= FCB->Send.Size && !((SendReq->AfdFlags & AFD_IMMEDIATE) || (FCB->NonBlocking)))
//            {
//                FCB->PollState &= ~AFD_EVENT_SEND;
//
//                NextIrp = NULL;
//            }
//
//            /* Check if we can send anything */
//            if (SpaceAvail == 0)
//            {
//                FCB->PollState &= ~AFD_EVENT_SEND;
//
//                /* We should never be non-overlapped and get to this point */
//                ASSERT(SendReq->AfdFlags & AFD_OVERLAPPED);
//
//                NextIrp = NULL;
//            }
//         }
//
//         if (NextIrp != NULL)
//         {
//             for( i = 0; i < SendReq->BufferCount; i++ ) {
//                 BytesCopied = MIN(SendReq->BufferArray[i].len, SpaceAvail);
//
//                 Map[i].BufferAddress =
//                    MmMapLockedPages( Map[i].Mdl, KernelMode );
//
//                 RtlCopyMemory( FCB->Send.Window + FCB->Send.BytesUsed,
//                                Map[i].BufferAddress,
//                                BytesCopied );
//
//                 MmUnmapLockedPages( Map[i].BufferAddress, Map[i].Mdl );
//
//                 TotalBytesCopied += BytesCopied;
//                 SpaceAvail -= BytesCopied;
//                 FCB->Send.BytesUsed += BytesCopied;
//             }
//
//             NextIrp->IoStatus.Information = TotalBytesCopied;
//             NextIrp->Tail.Overlay.DriverContext[3] = (PVOID)NextIrp->IoStatus.Information;
//         }
//     }
//
//     if (FCB->Send.Size - FCB->Send.BytesUsed != 0 && !FCB->SendClosed &&
//         IsListEmpty(&FCB->PendingIrpList[FUNCTION_SEND as usize]))
//     {
//         FCB->PollState |= AFD_EVENT_SEND;
//         FCB->PollStatus[FD_WRITE_BIT as usize] = STATUS_SUCCESS;
//         PollReeval( FCB->DeviceExt, FCB->FileObject );
//     }
//     else
//     {
//         FCB->PollState &= ~AFD_EVENT_SEND;
//     }
//
//
//     /* Some data is still waiting */
//     if( FCB->Send.BytesUsed )
//     {
//         Status = TdiSend( &FCB->SendIrp.InFlightRequest,
//                           FCB->Connection.Object,
//                           0,
//                           FCB->Send.Window,
//                           FCB->Send.BytesUsed,
//                           SendComplete,
//                           FCB );
//     }
//     else
//     {
//         /* Nothing is waiting so try to complete a pending disconnect */
//         RetryDisconnectCompletion(FCB);
//     }
//
//     SocketStateUnlock( FCB );
//
//     return STATUS_SUCCESS;
// }
//
// static IO_COMPLETION_ROUTINE PacketSocketSendComplete;
// static NTSTATUS NTAPI PacketSocketSendComplete
// ( PDEVICE_OBJECT DeviceObject,
//   PIRP Irp,
//   PVOID Context ) {
//     PAFD_FCB FCB = (PAFD_FCB)Context;
//     PLIST_ENTRY NextIrpEntry;
//     PIRP NextIrp;
//     PAFD_SEND_INFO SendReq;
//
//     UNREFERENCED_PARAMETER(DeviceObject as PVOID);
//
//     AFD_DbgPrint!(MID_TRACE,("Called, status %x, %u bytes used\n",
//                             Irp->IoStatus.Status,
//                             Irp->IoStatus.Information));
//
//     if( !SocketAcquireStateLock( FCB ) )
//         return STATUS_FILE_CLOSED;
//
//     ASSERT(FCB->SendIrp.InFlightRequest == Irp);
//     FCB->SendIrp.InFlightRequest = NULL;
//     /* Request is not in flight any longer */
//
//     if( FCB->SharedData.State == SOCKET_STATE_CLOSED ) {
//         /* Cleanup our IRP queue because the FCB is being destroyed */
//         while( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_SEND as usize] ) ) {
//             NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_SEND as usize]);
//             NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//             SendReq = GetLockedData(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
//             NextIrp->IoStatus.Status = STATUS_FILE_CLOSED;
//             NextIrp->IoStatus.Information = 0;
//             (void)IoSetCancelRoutine(NextIrp, NULL);
//             UnlockBuffers(SendReq->BufferArray, SendReq->BufferCount, FALSE);
//             UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//             IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//         }
//         SocketStateUnlock( FCB );
//         return STATUS_FILE_CLOSED;
//     }
//
//     ASSERT(!IsListEmpty(&FCB->PendingIrpList[FUNCTION_SEND as usize]));
//
//     /* TDI spec guarantees FIFO ordering on IRPs */
//     NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_SEND as usize]);
//     NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//
//     SendReq = GetLockedData(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
//
//     NextIrp->IoStatus.Status = Irp->IoStatus.Status;
//     NextIrp->IoStatus.Information = Irp->IoStatus.Information;
//
//     (void)IoSetCancelRoutine(NextIrp, NULL);
//
//     UnlockBuffers(SendReq->BufferArray, SendReq->BufferCount, FALSE);
//
//     UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
//
//     IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
//
//     FCB->PollState |= AFD_EVENT_SEND;
//     FCB->PollStatus[FD_WRITE_BIT as usize] = STATUS_SUCCESS;
//     PollReeval(FCB->DeviceExt, FCB->FileObject);
//
//     SocketStateUnlock(FCB);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS NTAPI
// AfdConnectedSocketWriteData(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                             PIO_STACK_LOCATION IrpSp, BOOLEAN Short) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PAFD_SEND_INFO SendReq;
//     UINT TotalBytesCopied = 0, i, SpaceAvail = 0, BytesCopied, SendLength;
//     KPROCESSOR_MODE LockMode;
//
//     UNREFERENCED_PARAMETER(DeviceObject as PVOID);
//     UNREFERENCED_PARAMETER(Short);
//
//     AFD_DbgPrint!(MID_TRACE,("Called on %p\n", FCB));
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     FCB->EventSelectDisabled &= ~AFD_EVENT_SEND;
//
//     if( FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS )
//     {
//         PAFD_SEND_INFO_UDP SendReq;
//         PTDI_CONNECTION_INFORMATION TargetAddress;
//
//         /* Check that the socket is bound */
//         if( FCB->SharedData.State != SOCKET_STATE_BOUND || !FCB->RemoteAddress )
//         {
//             AFD_DbgPrint!(MIN_TRACE,("Invalid parameter\n"));
//             return UnlockAndMaybeComplete( FCB, STATUS_INVALID_PARAMETER, Irp,
//                                            0 );
//         }
//
//         if( !(SendReq = LockRequest( Irp, IrpSp, FALSE, &LockMode )) )
//             return UnlockAndMaybeComplete( FCB, STATUS_NO_MEMORY, Irp, 0 );
//
//         /* Must lock buffers before handing off user data */
//         SendReq->BufferArray = LockBuffers( SendReq->BufferArray,
//                                             SendReq->BufferCount,
//                                             NULL, NULL,
//                                             FALSE, FALSE, LockMode );
//
//         if( !SendReq->BufferArray ) {
//             return UnlockAndMaybeComplete( FCB, STATUS_ACCESS_VIOLATION,
//                                            Irp, 0 );
//         }
//
//         Status = TdiBuildConnectionInfo( &TargetAddress, FCB->RemoteAddress );
//
//         if( NT_SUCCESS(Status) ) {
//             FCB->PollState &= ~AFD_EVENT_SEND;
//
//             Status = QueueUserModeIrp(FCB, Irp, FUNCTION_SEND);
//             if (Status == STATUS_PENDING)
//             {
//                 if (SendReq->BufferCount > 1)
//                 {
//                     AFD_DbgPrint!(MIN_TRACE,("WARN: More than one buffer %ld\n", SendReq->BufferCount));
//                 }
//                 Status = TdiSendDatagram(&FCB->SendIrp.InFlightRequest,
//                                          FCB->AddressFile.Object,
//                                          SendReq->BufferArray[0].buf,
//                                          SendReq->BufferArray[0].len,
//                                          TargetAddress,
//                                          PacketSocketSendComplete,
//                                          FCB);
//                 if (Status != STATUS_PENDING)
//                 {
//                     NT_VERIFY(RemoveHeadList(&FCB->PendingIrpList[FUNCTION_SEND as usize]) == &Irp->Tail.Overlay.ListEntry);
//                     Irp->IoStatus.Status = Status;
//                     Irp->IoStatus.Information = ((Status == STATUS_SUCCESS) ? SendReq->BufferArray[0].len : 0);
//                     (void)IoSetCancelRoutine(Irp, NULL);
//                     UnlockBuffers(SendReq->BufferArray, SendReq->BufferCount, FALSE);
//                     UnlockRequest(Irp, IoGetCurrentIrpStackLocation(Irp));
//                     IoCompleteRequest(Irp, IO_NETWORK_INCREMENT);
//                 }
//             }
//
//             ExFreePoolWithTag(TargetAddress, TAG_AFD_TDI_CONNECTION_INFORMATION);
//
//             SocketStateUnlock(FCB);
//
//             return STATUS_PENDING;
//         }
//         else
//         {
//             UnlockBuffers(SendReq->BufferArray, SendReq->BufferCount, FALSE);
//             return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
//         }
//     }
//
//     if (FCB->PollState & AFD_EVENT_CLOSE)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Connection reset by remote peer\n"));
//
//         /* This is an unexpected remote disconnect */
//         return UnlockAndMaybeComplete(FCB, FCB->PollStatus[FD_CLOSE_BIT], Irp, 0);
//     }
//
//     if (FCB->PollState & AFD_EVENT_ABORT)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Connection aborted\n"));
//
//         /* This is an abortive socket closure on our side */
//         return UnlockAndMaybeComplete(FCB, FCB->PollStatus[FD_CLOSE_BIT], Irp, 0);
//     }
//
//     if (FCB->SendClosed)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("No more sends\n"));
//
//         /* This is a graceful send closure */
//         return UnlockAndMaybeComplete(FCB, STATUS_FILE_CLOSED, Irp, 0);
//     }
//
//     if( !(SendReq = LockRequest( Irp, IrpSp, FALSE, &LockMode )) )
//         return UnlockAndMaybeComplete
//             ( FCB, STATUS_NO_MEMORY, Irp, 0 );
//
//     SendReq->BufferArray = LockBuffers( SendReq->BufferArray,
//                                         SendReq->BufferCount,
//                                         NULL, NULL,
//                                         FALSE, FALSE, LockMode );
//
//     if( !SendReq->BufferArray ) {
//         return UnlockAndMaybeComplete( FCB, STATUS_ACCESS_VIOLATION,
//                                        Irp, 0 );
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("Socket state %u\n", FCB->SharedData.State));
//
//     if( FCB->SharedData.State != SOCKET_STATE_CONNECTED ) {
//         AFD_DbgPrint!(MID_TRACE,("Socket not connected\n"));
//         UnlockBuffers( SendReq->BufferArray, SendReq->BufferCount, FALSE );
//         return UnlockAndMaybeComplete( FCB, STATUS_INVALID_CONNECTION, Irp, 0 );
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("FCB->Send.BytesUsed = %u\n",
//                             FCB->Send.BytesUsed));
//
//     SpaceAvail = FCB->Send.Size - FCB->Send.BytesUsed;
//
//     AFD_DbgPrint!(MID_TRACE,("We can accept %u bytes\n",
//                             SpaceAvail));
//
//     /* Count the total transfer size */
//     SendLength = 0;
//     for (i = 0; i < SendReq->BufferCount; i++)
//     {
//         SendLength += SendReq->BufferArray[i].len;
//     }
//
//     /* Make sure we've got the space */
//     if (SendLength > SpaceAvail)
//     {
//         /* Blocking sockets have to wait here */
//         if (SendLength <= FCB->Send.Size && !((SendReq->AfdFlags & AFD_IMMEDIATE) || (FCB->NonBlocking)))
//         {
//             FCB->PollState &= ~AFD_EVENT_SEND;
//             return LeaveIrpUntilLater(FCB, Irp, FUNCTION_SEND);
//         }
//
//         /* Check if we can send anything */
//         if (SpaceAvail == 0)
//         {
//             FCB->PollState &= ~AFD_EVENT_SEND;
//
//             /* Non-overlapped sockets will fail if we can send nothing */
//             if (!(SendReq->AfdFlags & AFD_OVERLAPPED))
//             {
//                 UnlockBuffers( SendReq->BufferArray, SendReq->BufferCount, FALSE );
//                 return UnlockAndMaybeComplete( FCB, STATUS_CANT_WAIT, Irp, 0 );
//             }
//             else
//             {
//                 /* Overlapped sockets just pend */
//                 return LeaveIrpUntilLater(FCB, Irp, FUNCTION_SEND);
//             }
//         }
//     }
//
//     for ( i = 0; SpaceAvail > 0 && i < SendReq->BufferCount; i++ )
//     {
//         BytesCopied = MIN(SendReq->BufferArray[i].len, SpaceAvail);
//
//         AFD_DbgPrint!(MID_TRACE,("Copying Buffer %u, %p:%u to %p\n",
//                                 i,
//                                 SendReq->BufferArray[i].buf,
//                                 BytesCopied,
//                                 FCB->Send.Window + FCB->Send.BytesUsed));
//
//         RtlCopyMemory(FCB->Send.Window + FCB->Send.BytesUsed,
//                       SendReq->BufferArray[i].buf,
//                       BytesCopied);
//
//         TotalBytesCopied += BytesCopied;
//         SpaceAvail -= BytesCopied;
//         FCB->Send.BytesUsed += BytesCopied;
//     }
//
//     Irp->IoStatus.Information = TotalBytesCopied;
//
//     if( TotalBytesCopied == 0 ) {
//         AFD_DbgPrint!(MID_TRACE,("Empty send\n"));
//         UnlockBuffers( SendReq->BufferArray, SendReq->BufferCount, FALSE );
//         return UnlockAndMaybeComplete
//         ( FCB, STATUS_SUCCESS, Irp, TotalBytesCopied );
//     }
//
//     if (SpaceAvail)
//     {
//         FCB->PollState |= AFD_EVENT_SEND;
//         FCB->PollStatus[FD_WRITE_BIT as usize] = STATUS_SUCCESS;
//         PollReeval( FCB->DeviceExt, FCB->FileObject );
//     }
//     else
//     {
//         FCB->PollState &= ~AFD_EVENT_SEND;
//     }
//
//     /* We use the IRP tail for some temporary storage here */
//     Irp->Tail.Overlay.DriverContext[3] = (PVOID)Irp->IoStatus.Information;
//
//     Status = QueueUserModeIrp(FCB, Irp, FUNCTION_SEND);
//     if (Status == STATUS_PENDING && !FCB->SendIrp.InFlightRequest)
//     {
//         TdiSend(&FCB->SendIrp.InFlightRequest,
//                 FCB->Connection.Object,
//                 0,
//                 FCB->Send.Window,
//                 FCB->Send.BytesUsed,
//                 SendComplete,
//                 FCB);
//     }
//
//     SocketStateUnlock(FCB);
//
//     return STATUS_PENDING;
// }
//
// NTSTATUS NTAPI
// AfdPacketSocketWriteData(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                          PIO_STACK_LOCATION IrpSp) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PTDI_CONNECTION_INFORMATION TargetAddress;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PAFD_SEND_INFO_UDP SendReq;
//     KPROCESSOR_MODE LockMode;
//
//     UNREFERENCED_PARAMETER(DeviceObject as PVOID);
//
//     AFD_DbgPrint!(MID_TRACE,("Called on %p\n", FCB));
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     FCB->EventSelectDisabled &= ~AFD_EVENT_SEND;
//
//     /* Check that the socket is bound */
//     if( FCB->SharedData.State != SOCKET_STATE_BOUND &&
//         FCB->SharedData.State != SOCKET_STATE_CREATED)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Invalid socket state\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
//     }
//
//     if (FCB->SendClosed)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("No more sends\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_FILE_CLOSED, Irp, 0);
//     }
//
//     if( !(SendReq = LockRequest( Irp, IrpSp, FALSE, &LockMode )) )
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if (FCB->SharedData.State == SOCKET_STATE_CREATED)
//     {
//         if (FCB->LocalAddress)
//         {
//             ExFreePoolWithTag(FCB->LocalAddress, TAG_AFD_TRANSPORT_ADDRESS);
//         }
//
//         FCB->LocalAddress =
//         TaBuildNullTransportAddress( ((PTRANSPORT_ADDRESS)SendReq->TdiConnection.RemoteAddress)->
//                                       Address[0].AddressType );
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
//             ( FCB, STATUS_NO_MEMORY, Irp, 0 );
//     }
//
//     SendReq->BufferArray = LockBuffers( SendReq->BufferArray,
//                                         SendReq->BufferCount,
//                                         NULL, NULL,
//                                         FALSE, FALSE, LockMode );
//
//     if( !SendReq->BufferArray )
//         return UnlockAndMaybeComplete( FCB, STATUS_ACCESS_VIOLATION,
//                                        Irp, 0 );
//
//     AFD_DbgPrint
//         (MID_TRACE,("RemoteAddress #%d Type %u\n",
//                     ((PTRANSPORT_ADDRESS)SendReq->TdiConnection.RemoteAddress)->
//                     TAAddressCount,
//                     ((PTRANSPORT_ADDRESS)SendReq->TdiConnection.RemoteAddress)->
//                     Address[0].AddressType));
//
//     Status = TdiBuildConnectionInfo( &TargetAddress,
//                             ((PTRANSPORT_ADDRESS)SendReq->TdiConnection.RemoteAddress) );
//
//     /* Check the size of the Address given ... */
//
//     if( NT_SUCCESS(Status) ) {
//         FCB->PollState &= ~AFD_EVENT_SEND;
//
//         Status = QueueUserModeIrp(FCB, Irp, FUNCTION_SEND);
//         if (Status == STATUS_PENDING)
//         {
//             if (SendReq->BufferCount > 1)
//             {
//                 AFD_DbgPrint!(MIN_TRACE,("WARN: More than one buffer %ld\n", SendReq->BufferCount));
//             }
//             Status = TdiSendDatagram(&FCB->SendIrp.InFlightRequest,
//                                      FCB->AddressFile.Object,
//                                      SendReq->BufferArray[0].buf,
//                                      SendReq->BufferArray[0].len,
//                                      TargetAddress,
//                                      PacketSocketSendComplete,
//                                      FCB);
//             if (Status != STATUS_PENDING)
//             {
//                 NT_VERIFY(RemoveHeadList(&FCB->PendingIrpList[FUNCTION_SEND as usize]) == &Irp->Tail.Overlay.ListEntry);
//                 Irp->IoStatus.Status = Status;
//                 Irp->IoStatus.Information = ((Status == STATUS_SUCCESS) ? SendReq->BufferArray[0].len : 0);
//                 (void)IoSetCancelRoutine(Irp, NULL);
//                 UnlockBuffers(SendReq->BufferArray, SendReq->BufferCount, FALSE);
//                 UnlockRequest(Irp, IoGetCurrentIrpStackLocation(Irp));
//                 IoCompleteRequest(Irp, IO_NETWORK_INCREMENT);
//             }
//         }
//
//         ExFreePoolWithTag(TargetAddress, TAG_AFD_TDI_CONNECTION_INFORMATION);
//
//         SocketStateUnlock(FCB);
//
//         return STATUS_PENDING;
//     }
//     else
//     {
//         UnlockBuffers(SendReq->BufferArray, SendReq->BufferCount, FALSE);
//         return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
