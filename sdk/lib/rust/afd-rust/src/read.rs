use crate::types::*;
// The crate's own functions: the definition is the declaration.
use crate::lock::UnlockBuffers;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IRP, KPROCESSOR_MODE, MmMapLockedPages, MmUnmapLockedPages, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoMarkIrpPending, IoSetCancelRoutine};
use core::ffi::c_void;
use ntkrnl_sys::ddk::RtlAssert;

extern "C" {
    fn CheckUnlockExtraBuffers(FCB: PAFD_FCB, IrpSp: PIO_STACK_LOCATION) -> BOOLEAN;
    pub fn GetLockedData(Irp: PIRP, IrpSp: PIO_STACK_LOCATION) -> PVOID;
    pub fn PollReeval(DeviceObject: PAFD_DEVICE_EXTENSION, FileObject: PFILE_OBJECT);
    fn SocketAcquireStateLock(FCB: PAFD_FCB) -> BOOLEAN;
    fn SocketStateUnlock(FCB: PAFD_FCB);
    pub fn TaCopyTransportAddress(OtherAddress: PTRANSPORT_ADDRESS) -> PTRANSPORT_ADDRESS;
    fn UnlockRequest(Irp: PIRP, IrpSp: PIO_STACK_LOCATION);
}

unsafe fn RefillSocketBuffer(FCB: PAFD_FCB) {
    if (*FCB).ReceiveIrp.InFlightRequest != core::ptr::null_mut() {
        return;
    }

    if (*FCB).TdiReceiveClosed != 0 {
        return;
    }

    if (*FCB).Recv.Content == (*FCB).Recv.Size {
        if (*FCB).Recv.BytesUsed != 0 {
            core::ptr::copy(
                (*FCB).Recv.Window.add((*FCB).Recv.BytesUsed as usize),
                (*FCB).Recv.Window,
                ((*FCB).Recv.Content - (*FCB).Recv.BytesUsed) as usize,
            );

            (*FCB).Recv.Content -= (*FCB).Recv.BytesUsed;
            (*FCB).Recv.BytesUsed = 0;
        } else {
            return;
        }
    }

    AFD_DbgPrint!(MID_TRACE, b"Replenishing buffer\n\0".as_ptr() as PCHAR);

    TdiReceive(
        core::ptr::addr_of_mut!((*FCB).ReceiveIrp.InFlightRequest),
        (*FCB).Connection.Object,
        TDI_RECEIVE_NORMAL as USHORT,
        (*FCB).Recv.Window.add((*FCB).Recv.Content as usize),
        (*FCB).Recv.Size - (*FCB).Recv.Content,
        Some(ReceiveComplete),
        FCB as PVOID,
    );
}

unsafe fn HandleReceiveComplete(FCB: PAFD_FCB, Status: NTSTATUS, Information: ULONG_PTR) {
    (*FCB).LastReceiveStatus = Status;

    if (*FCB).TdiReceiveClosed != 0 {
    } else if Status == STATUS_SUCCESS {
        (*FCB).Recv.Content += Information as UINT;
        ntkrnl_sys::ASSERT!((*FCB).Recv.Content <= (*FCB).Recv.Size);

        if Information == 0 {
            (*FCB).TdiReceiveClosed = 1u8;
        } else {
            RefillSocketBuffer(FCB);
        }
    } else {
        (*FCB).TdiReceiveClosed = 1u8;
    }
}

unsafe fn CantReadMore(FCB: PAFD_FCB) -> BOOLEAN {
    let BytesAvailable: UINT = (*FCB).Recv.Content - (*FCB).Recv.BytesUsed;

    if BytesAvailable == 0 && (*FCB).TdiReceiveClosed != 0 {
        1u8
    } else {
        0u8
    }
}

unsafe fn TryToSatisfyRecvRequestFromBuffer(
    FCB: PAFD_FCB,
    RecvReq: PAFD_RECV_INFO,
    TotalBytesCopied: *mut UINT,
) -> NTSTATUS {
    let mut BytesToCopy: UINT;
    let mut FcbBytesCopied: UINT = (*FCB).Recv.BytesUsed;
    let mut BytesAvailable: UINT = (*FCB).Recv.Content - (*FCB).Recv.BytesUsed;
    let Map: PAFD_MAPBUF;

    *TotalBytesCopied = 0;

    AFD_DbgPrint!(
        MID_TRACE,
        b"Called, BytesAvailable = %u\n\0".as_ptr() as PCHAR,
        BytesAvailable,
    );

    if CantReadMore(FCB) != 0 {
        return STATUS_SUCCESS;
    }
    if BytesAvailable == 0 {
        return STATUS_PENDING;
    }

    Map = ((*RecvReq).BufferArray.offset((*RecvReq).BufferCount as isize)) as PAFD_MAPBUF;

    AFD_DbgPrint!(
        MID_TRACE,
        b"Buffer Count: %u @ %p\n\0".as_ptr() as PCHAR,
        (*RecvReq).BufferCount,
        (*RecvReq).BufferArray,
    );

    let mut i: UINT = 0;
    while !(*RecvReq).BufferArray.is_null()
        && BytesAvailable != 0
        && i < (*RecvReq).BufferCount
    {
        BytesToCopy = if (*(*RecvReq).BufferArray.offset(i as isize)).len < BytesAvailable {
            (*(*RecvReq).BufferArray.offset(i as isize)).len
        } else {
            BytesAvailable
        };

        if !(*Map.offset(i as isize)).Mdl.is_null() {
            (*Map.offset(i as isize)).BufferAddress =
                MmMapLockedPages((*Map.offset(i as isize)).Mdl, 0i32 as KPROCESSOR_MODE);

            AFD_DbgPrint!(
                MID_TRACE,
                b"Buffer %u: %p:%u\n\0".as_ptr() as PCHAR,
                i,
                (*Map.offset(i as isize)).BufferAddress,
                BytesToCopy,
            );

            RtlCopyMemory(
                (*Map.offset(i as isize)).BufferAddress,
                (*FCB).Recv.Window.offset(FcbBytesCopied as isize) as PVOID,
                BytesToCopy as usize,
            );

            MmUnmapLockedPages(
                (*Map.offset(i as isize)).BufferAddress,
                (*Map.offset(i as isize)).Mdl,
            );

            *TotalBytesCopied += BytesToCopy;
            FcbBytesCopied += BytesToCopy;
            BytesAvailable -= BytesToCopy;

            if ((*RecvReq).TdiFlags & TDI_RECEIVE_PEEK) == 0 {
                (*FCB).Recv.BytesUsed += BytesToCopy;
            }
        }

        i += 1;
    }

    RefillSocketBuffer(FCB);

    STATUS_SUCCESS
}

unsafe fn ReceiveActivity(FCB: PAFD_FCB, Irp: PIRP) -> NTSTATUS {
    let mut NextIrpEntry: PLIST_ENTRY;
    let mut NextIrp: PIRP;
    let mut NextIrpSp: PIO_STACK_LOCATION;
    let mut RecvReq: PAFD_RECV_INFO;
    let mut TotalBytesCopied: UINT = 0;
    let mut Status: NTSTATUS;
    let mut RetStatus: NTSTATUS = STATUS_PENDING;

    AFD_DbgPrint!(MID_TRACE, b"%p %p\n\0".as_ptr() as PCHAR, FCB, Irp);

    AFD_DbgPrint!(
        MID_TRACE,
        b"FCB %p Receive data waiting %u\n\0".as_ptr() as PCHAR,
        FCB,
        (*FCB).Recv.Content,
    );

    if CantReadMore(FCB) != 0 {
        while !IsListEmpty(&(*FCB).PendingIrpList[FUNCTION_RECV as usize]) {
            NextIrpEntry = RemoveHeadList(&mut (*FCB).PendingIrpList[FUNCTION_RECV as usize]);
            NextIrp = (NextIrpEntry as *mut u8)
                .offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;
            NextIrpSp = IoGetCurrentIrpStackLocation(NextIrp);
            RecvReq = (GetLockedData(NextIrp, NextIrpSp)) as _;

            AFD_DbgPrint!(
                MID_TRACE,
                b"Completing recv %p (%u)\n\0".as_ptr() as PCHAR,
                NextIrp,
                TotalBytesCopied,
            );
            UnlockBuffers(((*RecvReq).BufferArray as PVOID).cast(), (*RecvReq).BufferCount, (0u8) as _);

            Status = (*FCB).LastReceiveStatus;

            (*NextIrp).IoStatus.__bindgen_anon_1.Status = Status;
            (*NextIrp).IoStatus.Information = 0;
            if NextIrp == Irp {
                RetStatus = Status;
            }
            if !(*NextIrp).MdlAddress.is_null() {
                UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
            }
            IoSetCancelRoutine(NextIrp, None);
            IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT as i8);
        }
    } else {
        AFD_DbgPrint!(
            MID_TRACE,
            b"FCB %p Receive data waiting %u\n\0".as_ptr() as PCHAR,
            FCB,
            (*FCB).Recv.Content,
        );

        while !IsListEmpty(&(*FCB).PendingIrpList[FUNCTION_RECV as usize]) {
            NextIrpEntry = RemoveHeadList(&mut (*FCB).PendingIrpList[FUNCTION_RECV as usize]);
            NextIrp = (NextIrpEntry as *mut u8)
                .offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;
            NextIrpSp = IoGetCurrentIrpStackLocation(NextIrp);
            RecvReq = (GetLockedData(NextIrp, NextIrpSp)) as _;

            AFD_DbgPrint!(MID_TRACE, b"RecvReq @ %p\n\0".as_ptr() as PCHAR, RecvReq);

            Status = TryToSatisfyRecvRequestFromBuffer(FCB, RecvReq, &mut TotalBytesCopied);

            if Status == STATUS_PENDING {
                AFD_DbgPrint!(
                    MID_TRACE,
                    b"Ran out of data for %p\n\0".as_ptr() as PCHAR,
                    NextIrp,
                );
                InsertHeadList(
                    &mut (*FCB).PendingIrpList[FUNCTION_RECV as usize],
                    &mut (*NextIrp).Tail.Overlay.__bindgen_anon_2.ListEntry,
                );
                break;
            } else {
                AFD_DbgPrint!(
                    MID_TRACE,
                    b"Completing recv %p (%u)\n\0".as_ptr() as PCHAR,
                    NextIrp,
                    TotalBytesCopied,
                );
                UnlockBuffers(((*RecvReq).BufferArray as PVOID).cast(), (*RecvReq).BufferCount, (0u8) as _);
                (*NextIrp).IoStatus.__bindgen_anon_1.Status = Status;
                (*NextIrp).IoStatus.Information = TotalBytesCopied as _;
                if NextIrp == Irp {
                    RetStatus = Status;
                }
                if !(*NextIrp).MdlAddress.is_null() {
                    UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
                }
                IoSetCancelRoutine(NextIrp, None);
                IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT as i8);
            }
        }
    }

    if ((*FCB).Recv.Content - (*FCB).Recv.BytesUsed) != 0
        && IsListEmpty(&(*FCB).PendingIrpList[FUNCTION_RECV as usize])
    {
        (*FCB).PollState |= AFD_EVENT_RECEIVE;
        (*FCB).PollStatus[FD_READ_BIT as usize] = STATUS_SUCCESS;
        PollReeval(((*FCB).DeviceExt as PVOID) as _, (*FCB).FileObject);
    } else {
        (*FCB).PollState &= !AFD_EVENT_RECEIVE;
    }

    if CantReadMore(FCB) != 0 {
        if (*FCB).LastReceiveStatus == STATUS_SUCCESS {
            (*FCB).PollState |= AFD_EVENT_DISCONNECT;
        } else {
            (*FCB).PollState |= AFD_EVENT_CLOSE;
        }
        (*FCB).PollStatus[FD_CLOSE_BIT as usize] = (*FCB).LastReceiveStatus;
        PollReeval(((*FCB).DeviceExt as PVOID) as _, (*FCB).FileObject);
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"RetStatus for irp %p is %x\n\0".as_ptr() as PCHAR,
        Irp,
        RetStatus,
    );

    RetStatus
}

// AcceptEx receive completion: the early data that followed the accepted connection has
// arrived, so complete the held AcceptEx IRP (FCB->AcceptIrp) and resume receive activity.
#[no_mangle]
pub unsafe extern "system" fn AcceptExReceiveComplete(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let FCB: PAFD_FCB = Context as PAFD_FCB;
    let AcceptIrp: PIRP;

    if SocketAcquireStateLock(FCB) == 0 {
        return STATUS_FILE_CLOSED;
    }

    ASSERT!((*FCB).ReceiveIrp.InFlightRequest == Irp);
    ASSERT!(!(*FCB).AcceptIrp.is_null());
    (*FCB).ReceiveIrp.InFlightRequest = core::ptr::null_mut();
    (*FCB).SharedData.State = SOCKET_STATE_CONNECTED;

    // Complete the AcceptEx IRP as the data has been received
    AcceptIrp = (*FCB).AcceptIrp;
    ntkrnl_sys::ddk::MmUnlockPages(
        (*AcceptIrp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[3] as ntkrnl_sys::ddk::PMDL,
    );
    ntkrnl_sys::ddk::IoFreeMdl(
        (*AcceptIrp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[3] as ntkrnl_sys::ddk::PMDL,
    );
    (*AcceptIrp).IoStatus.Information = (*Irp).IoStatus.Information;
    (*AcceptIrp).IoStatus.__bindgen_anon_1.Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    if !(*AcceptIrp).MdlAddress.is_null() {
        UnlockRequest(AcceptIrp, IoGetCurrentIrpStackLocation(AcceptIrp));
    }
    IoSetCancelRoutine(AcceptIrp, None);
    IoCompleteRequest(AcceptIrp, IO_NETWORK_INCREMENT);
    (*FCB).AcceptIrp = core::ptr::null_mut();

    // Continue receive activity
    ReceiveActivity(FCB, core::ptr::null_mut());

    SocketStateUnlock(FCB);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ReceiveComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    receive_complete_impl(DeviceObject, Irp, Context)
}

unsafe fn receive_complete_impl(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let FCB: PAFD_FCB = Context as PAFD_FCB;
    let mut NextIrpEntry: PLIST_ENTRY;
    let mut NextIrp: PIRP;
    let mut RecvReq: PAFD_RECV_INFO;
    let mut NextIrpSp: PIO_STACK_LOCATION;

    AFD_DbgPrint!(MID_TRACE, b"Called\n\0".as_ptr() as PCHAR);

    if SocketAcquireStateLock(FCB) == 0 {
        return STATUS_FILE_CLOSED;
    }

    (*FCB).ReceiveIrp.InFlightRequest = core::ptr::null_mut();

    if (*FCB).SharedData.State == SOCKET_STATE_CLOSED {
        while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_RECV as usize])) {
            NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_RECV as usize]));
            NextIrp = (NextIrpEntry as *mut u8)
                .offset(-(core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry) as isize)) as PIRP;
            NextIrpSp = IoGetCurrentIrpStackLocation(NextIrp);
            RecvReq = (GetLockedData(NextIrp, NextIrpSp)) as _;
            (*NextIrp).IoStatus.__bindgen_anon_1.Status = STATUS_FILE_CLOSED;
            (*NextIrp).IoStatus.Information = 0;
            UnlockBuffers(((*RecvReq).BufferArray as PVOID).cast(), (*RecvReq).BufferCount, (0u8) as _);
            if !(*NextIrp).MdlAddress.is_null() {
                UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
            }
            IoSetCancelRoutine(NextIrp, None);
            IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT as i8);
        }
        SocketStateUnlock(FCB);
        return STATUS_FILE_CLOSED;
    } else if (*FCB).SharedData.State == SOCKET_STATE_LISTENING {
        AFD_DbgPrint!(MIN_TRACE, b"!!! LISTENER GOT A RECEIVE COMPLETE !!!\n\0".as_ptr() as PCHAR);
        SocketStateUnlock(FCB);
        return STATUS_INVALID_PARAMETER;
    }

    HandleReceiveComplete(FCB, (*Irp).IoStatus.__bindgen_anon_1.Status, (*Irp).IoStatus.Information as usize);

    ReceiveActivity(FCB, core::ptr::null_mut());

    SocketStateUnlock(FCB);

    STATUS_SUCCESS
}

unsafe fn SatisfyPacketRecvRequest(
    FCB: PAFD_FCB,
    Irp: PIRP,
    DatagramRecv: PAFD_STORED_DATAGRAM,
    TotalBytesCopied: PUINT,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let RecvReq: PAFD_RECV_INFO = (GetLockedData(Irp, IrpSp)) as _;
    let BytesToCopy: UINT;
    let BytesAvailable: UINT = (*DatagramRecv).Len;
    let AddrLen: UINT;
    let Map: PAFD_MAPBUF;
    let ExtraBuffers: BOOLEAN = CheckUnlockExtraBuffers(FCB, IrpSp);

    Map = ((*RecvReq).BufferArray.add((*RecvReq).BufferCount as usize)
        .add(if ExtraBuffers != 0 { EXTRA_LOCK_BUFFERS as usize } else { 0 })) as PAFD_MAPBUF;

    BytesToCopy = MIN((*(*RecvReq).BufferArray.add(0)).len, BytesAvailable);

    AFD_DbgPrint!(
        MID_TRACE,
        b"BytesToCopy: %u len %u\n\0".as_ptr() as PCHAR,
        BytesToCopy,
        (*(*RecvReq).BufferArray.add(0)).len,
    );

    if !(*Map.add(0)).Mdl.is_null() {
        if ExtraBuffers != 0 && !(*Map.add(1)).Mdl.is_null() && !(*Map.add(2)).Mdl.is_null() {
            AFD_DbgPrint!(MID_TRACE, b"Checking TAAddressCount\n\0".as_ptr() as PCHAR);

            if (*(*DatagramRecv).Address).TAAddressCount != 1 {
                AFD_DbgPrint!(
                    MIN_TRACE,
                    b"Wierd address count %d\n\0".as_ptr() as PCHAR,
                    (*(*DatagramRecv).Address).TAAddressCount,
                );
            }

            AFD_DbgPrint!(MID_TRACE, b"Computing addr len\n\0".as_ptr() as PCHAR);

            AddrLen = MIN(
                (*(*(*DatagramRecv).Address).Address.as_ptr().add(0)).AddressLength as UINT
                    + core::mem::size_of::<USHORT>() as UINT,
                (*(*RecvReq).BufferArray.add(1)).len,
            );

            AFD_DbgPrint!(
                MID_TRACE,
                b"Copying %u bytes of address\n\0".as_ptr() as PCHAR,
                AddrLen,
            );

            (*Map.add(1)).BufferAddress = MmMapLockedPages((*Map.add(1)).Mdl, 0i8);

            AFD_DbgPrint!(MID_TRACE, b"Done mapping, copying address\n\0".as_ptr() as PCHAR);

            RtlCopyMemory(
                (*Map.add(1)).BufferAddress,
                core::ptr::addr_of!((*(*(*DatagramRecv).Address).Address.as_ptr().add(0)).AddressType) as PVOID,
                AddrLen as usize,
            );

            MmUnmapLockedPages((*Map.add(1)).BufferAddress, (*Map.add(1)).Mdl);

            AFD_DbgPrint!(MID_TRACE, b"Copying address len\n\0".as_ptr() as PCHAR);

            (*Map.add(2)).BufferAddress = MmMapLockedPages((*Map.add(2)).Mdl, 0i8);
            *((*Map.add(2)).BufferAddress as PINT) = AddrLen as INT;
            MmUnmapLockedPages((*Map.add(2)).BufferAddress, (*Map.add(2)).Mdl);
        }

        AFD_DbgPrint!(MID_TRACE, b"Mapping data buffer pages\n\0".as_ptr() as PCHAR);

        (*Map.add(0)).BufferAddress = MmMapLockedPages((*Map.add(0)).Mdl, 0i8);

        AFD_DbgPrint!(
            MID_TRACE,
            b"Buffer %d: %p:%u\n\0".as_ptr() as PCHAR,
            0,
            (*Map.add(0)).BufferAddress,
            BytesToCopy,
        );

        RtlCopyMemory(
            (*Map.add(0)).BufferAddress,
            (*DatagramRecv).Buffer.as_ptr() as *const c_void,
            BytesToCopy as usize,
        );

        MmUnmapLockedPages((*Map.add(0)).BufferAddress, (*Map.add(0)).Mdl);

        *TotalBytesCopied = BytesToCopy;
    }

    if *TotalBytesCopied == (*DatagramRecv).Len {
        Status = STATUS_SUCCESS;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    } else {
        Status = STATUS_BUFFER_OVERFLOW;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_OVERFLOW;
    }

    (*Irp).IoStatus.Information = *TotalBytesCopied as _;

    if ((*RecvReq).TdiFlags & TDI_RECEIVE_PEEK) == 0 {
        (*FCB).Recv.Content -= (*DatagramRecv).Len;
        ExFreePoolWithTag((*DatagramRecv).Address as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
        ExFreePoolWithTag(DatagramRecv as PVOID, TAG_AFD_STORED_DATAGRAM);
    }

    AFD_DbgPrint!(MID_TRACE, b"Done\n\0".as_ptr() as PCHAR);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn AfdConnectedSocketReadData(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
    Short: u8,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let RecvReq: PAFD_RECV_INFO;
    let TotalBytesCopied: UINT;
    let DatagramRecv: PAFD_STORED_DATAGRAM;
    let ListEntry: PLIST_ENTRY;
    let mut LockMode: KPROCESSOR_MODE = 0;

    UNREFERENCED_PARAMETER!(DeviceObject as PVOID);
    UNREFERENCED_PARAMETER!(Short as usize as PVOID);

    AFD_DbgPrint!(MID_TRACE, b"Called on %p\n\0".as_ptr() as PCHAR, FCB);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    (*FCB).EventSelectDisabled &= !AFD_EVENT_RECEIVE;

    if ((*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS) == 0
        && (*FCB).SharedData.State != SOCKET_STATE_CONNECTED
        && (*FCB).SharedData.State != SOCKET_STATE_CONNECTING
    {
        AFD_DbgPrint!(
            MIN_TRACE,
            b"Called recv on wrong kind of socket (s%x)\n\0".as_ptr() as PCHAR,
            (*FCB).SharedData.State,
        );
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
    }

    RecvReq = LockRequest(Irp, IrpSp, 0u8, &mut LockMode) as PAFD_RECV_INFO;
    if RecvReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"Recv flags %x\n\0".as_ptr() as PCHAR,
        (*RecvReq).AfdFlags,
    );

    (*RecvReq).BufferArray = LockBuffers(
        (*RecvReq).BufferArray,
        (*RecvReq).BufferCount,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        1u8,
        0u8,
        LockMode,
    );

    if (*RecvReq).BufferArray.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_ACCESS_VIOLATION, Irp, 0);
    }

    if ((*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS) != 0 {
        if !IsListEmpty(&mut (*FCB).DatagramList) {
            ListEntry = RemoveHeadList(&mut (*FCB).DatagramList);
            DatagramRecv = (ListEntry as *mut u8).offset(
                -(core::mem::offset_of!(_AFD_STORED_DATAGRAM, ListEntry) as isize)
            ) as PAFD_STORED_DATAGRAM;
            Status = SatisfyPacketRecvRequest(
                FCB,
                Irp,
                DatagramRecv,
                core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as PUINT,
            );

            if ((*RecvReq).TdiFlags & TDI_RECEIVE_PEEK) != 0 {
                InsertHeadList(&mut (*FCB).DatagramList, &mut (*DatagramRecv).ListEntry);
            }

            if !IsListEmpty(&mut (*FCB).DatagramList) {
                (*FCB).PollState |= AFD_EVENT_RECEIVE;
                (*FCB).PollStatus[FD_READ_BIT as usize] = STATUS_SUCCESS;
                PollReeval(((*FCB).DeviceExt as PVOID) as _, (*FCB).FileObject);
            } else {
                (*FCB).PollState &= !AFD_EVENT_RECEIVE;
            }

            UnlockBuffers(((*RecvReq).BufferArray as PVOID).cast(), (*RecvReq).BufferCount, (0u8) as _);

            return UnlockAndMaybeComplete(FCB, Status, Irp, (*Irp).IoStatus.Information as u32);
        } else if ((*RecvReq).AfdFlags & AFD_OVERLAPPED) == 0
            && (((*RecvReq).AfdFlags & AFD_IMMEDIATE) != 0 || (*FCB).NonBlocking != 0)
        {
            AFD_DbgPrint!(MID_TRACE, b"Nonblocking\n\0".as_ptr() as PCHAR);
            Status = STATUS_CANT_WAIT;
            (*FCB).PollState &= !AFD_EVENT_RECEIVE;
            UnlockBuffers(((*RecvReq).BufferArray as PVOID).cast(), (*RecvReq).BufferCount, (0u8) as _);
            return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
        } else {
            (*FCB).PollState &= !AFD_EVENT_RECEIVE;
            return LeaveIrpUntilLater(FCB, Irp, FUNCTION_RECV);
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_PENDING;
    (*Irp).IoStatus.Information = 0;

    InsertTailList(
        &mut (*FCB).PendingIrpList[FUNCTION_RECV as usize],
        &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
    );

    Status = ReceiveActivity(FCB, Irp);

    if Status == STATUS_PENDING
        && ((*RecvReq).AfdFlags & AFD_OVERLAPPED) == 0
        && (((*RecvReq).AfdFlags & AFD_IMMEDIATE) != 0 || (*FCB).NonBlocking != 0)
    {
        AFD_DbgPrint!(MID_TRACE, b"Nonblocking\n\0".as_ptr() as PCHAR);
        Status = STATUS_CANT_WAIT;
        TotalBytesCopied = 0;
        RemoveEntryList(
            &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry
        );
        UnlockBuffers(((*RecvReq).BufferArray as PVOID).cast(), (*RecvReq).BufferCount, (0u8) as _);
        return UnlockAndMaybeComplete(FCB, Status, Irp, TotalBytesCopied);
    } else if Status == STATUS_PENDING {
        AFD_DbgPrint!(MID_TRACE, b"Leaving read irp\n\0".as_ptr() as PCHAR);
        IoMarkIrpPending(Irp);
        IoSetCancelRoutine(Irp, Some(AfdCancelHandler));
    } else {
        AFD_DbgPrint!(
            MID_TRACE,
            b"Completed with status %x\n\0".as_ptr() as PCHAR,
            Status,
        );
    }

    SocketStateUnlock(FCB);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn PacketSocketRecvComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    packet_socket_recv_complete_impl(DeviceObject, Irp, Context)
}

unsafe fn packet_socket_recv_complete_impl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let FCB: PAFD_FCB = Context as PAFD_FCB;
    let mut NextIrp: PIRP;
    let mut NextIrpSp: PIO_STACK_LOCATION;
    let mut ListEntry: PLIST_ENTRY;
    let mut RecvReq: PAFD_RECV_INFO;
    let mut DatagramRecv: PAFD_STORED_DATAGRAM;
    let DGSize: UINT = (*Irp).IoStatus.Information as UINT + core::mem::size_of::<AFD_STORED_DATAGRAM>() as UINT;
    let mut NextIrpEntry: PLIST_ENTRY;
    let mut DatagramRecvEntry: PLIST_ENTRY;

    UNREFERENCED_PARAMETER!(DeviceObject as PVOID);

    AFD_DbgPrint!(MID_TRACE, b"Called on %p\n\0".as_ptr() as *const i8, FCB);

    if SocketAcquireStateLock(FCB) == 0 {
        return STATUS_FILE_CLOSED;
    }

    if (*FCB).ReceiveIrp.InFlightRequest != Irp {
        RtlAssert(
            b"FCB->ReceiveIrp.InFlightRequest == Irp\0".as_ptr() as *mut core::ffi::c_void,
            b"read.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            core::ptr::null_mut::<i8>(),
        );
    }
    (*FCB).ReceiveIrp.InFlightRequest = core::ptr::null_mut();

    if (*FCB).SharedData.State == SOCKET_STATE_CLOSED {
        while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_RECV as usize])) {
            NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_RECV as usize]));
            NextIrp = (NextIrpEntry as usize - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP;
            NextIrpSp = IoGetCurrentIrpStackLocation(NextIrp);
            RecvReq = (GetLockedData(NextIrp, NextIrpSp)) as _;
            (*NextIrp).IoStatus.__bindgen_anon_1.Status = STATUS_FILE_CLOSED;
            (*NextIrp).IoStatus.Information = 0;
            UnlockBuffers(((*RecvReq).BufferArray as PVOID).cast(), (*RecvReq).BufferCount, (CheckUnlockExtraBuffers(FCB, NextIrpSp)) as _);
            if (*NextIrp).MdlAddress != core::ptr::null_mut() {
                UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
            }
            IoSetCancelRoutine(NextIrp, None);
            IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
        }

        while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).DatagramList)) {
            DatagramRecvEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).DatagramList));
            DatagramRecv = (DatagramRecvEntry as usize - core::mem::offset_of!(AFD_STORED_DATAGRAM, ListEntry)) as PAFD_STORED_DATAGRAM;
            ExFreePoolWithTag((*DatagramRecv).Address as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
            ExFreePoolWithTag(DatagramRecv as PVOID, TAG_AFD_STORED_DATAGRAM);
        }

        SocketStateUnlock(FCB);
        return STATUS_FILE_CLOSED;
    }

    if (*Irp).IoStatus.__bindgen_anon_1.Status != STATUS_SUCCESS {
        SocketStateUnlock(FCB);
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    if (*FCB).TdiReceiveClosed != 0 {
        SocketStateUnlock(FCB);
        return STATUS_FILE_CLOSED;
    }

    DatagramRecv = ExAllocatePoolWithTag(
        NonPagedPool,
        DGSize as SIZE_T,
        TAG_AFD_STORED_DATAGRAM,
    ) as PAFD_STORED_DATAGRAM;

    if DatagramRecv != core::ptr::null_mut() {
        (*DatagramRecv).Len = (*Irp).IoStatus.Information as UINT;
        RtlCopyMemory(
            core::ptr::addr_of_mut!((*DatagramRecv).Buffer) as PVOID,
            (*FCB).Recv.Window as *const c_void,
            (*DatagramRecv).Len as SIZE_T,
        );
        AFD_DbgPrint!(
            MID_TRACE,
            b"Received (A %p)\n\0".as_ptr() as *const i8,
            (*(*FCB).AddressFrom).RemoteAddress,
        );
        (*DatagramRecv).Address = TaCopyTransportAddress(((*(*FCB).AddressFrom).RemoteAddress) as _) as PTRANSPORT_ADDRESS;

        if (*DatagramRecv).Address == core::ptr::null_mut() {
            Status = STATUS_NO_MEMORY;
        }
    } else {
        Status = STATUS_NO_MEMORY;
    }

    if !NT_SUCCESS(Status) {
        if DatagramRecv != core::ptr::null_mut() {
            ExFreePoolWithTag(DatagramRecv as PVOID, TAG_AFD_STORED_DATAGRAM);
        }

        SocketStateUnlock(FCB);
        return Status;
    } else {
        (*FCB).Recv.Content += (*DatagramRecv).Len;
        InsertTailList(
            core::ptr::addr_of_mut!((*FCB).DatagramList),
            core::ptr::addr_of_mut!((*DatagramRecv).ListEntry),
        );
    }

    while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).DatagramList))
        && !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_RECV as usize]))
    {
        AFD_DbgPrint!(MID_TRACE, b"Looping trying to satisfy request\n\0".as_ptr() as *const i8);
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).DatagramList));
        DatagramRecv = (ListEntry as usize - core::mem::offset_of!(AFD_STORED_DATAGRAM, ListEntry)) as PAFD_STORED_DATAGRAM;
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_RECV as usize]));
        NextIrp = (ListEntry as usize - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP;
        NextIrpSp = IoGetCurrentIrpStackLocation(NextIrp);
        RecvReq = (GetLockedData(NextIrp, NextIrpSp)) as _;

        AFD_DbgPrint!(
            MID_TRACE,
            b"RecvReq: %p, DatagramRecv: %p\n\0".as_ptr() as *const i8,
            RecvReq,
            DatagramRecv,
        );

        AFD_DbgPrint!(MID_TRACE, b"Satisfying\n\0".as_ptr() as *const i8);
        Status = SatisfyPacketRecvRequest(
            FCB,
            NextIrp,
            DatagramRecv,
            core::ptr::addr_of_mut!((*NextIrp).IoStatus.Information) as PUINT,
        );

        if (*RecvReq).TdiFlags & TDI_RECEIVE_PEEK != 0 {
            InsertHeadList(
                core::ptr::addr_of_mut!((*FCB).DatagramList),
                core::ptr::addr_of_mut!((*DatagramRecv).ListEntry),
            );
        }

        AFD_DbgPrint!(MID_TRACE, b"Unlocking\n\0".as_ptr() as *const i8);
        UnlockBuffers(((*RecvReq).BufferArray as PVOID).cast(), (*RecvReq).BufferCount, (CheckUnlockExtraBuffers(FCB, NextIrpSp)) as _);
        if (*NextIrp).MdlAddress != core::ptr::null_mut() {
            UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
        }

        AFD_DbgPrint!(MID_TRACE, b"Completing\n\0".as_ptr() as *const i8);
        IoSetCancelRoutine(NextIrp, None);
        (*NextIrp).IoStatus.__bindgen_anon_1.Status = Status;

        IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*FCB).DatagramList)) && IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_RECV as usize])) {
        AFD_DbgPrint!(MID_TRACE, b"Signalling\n\0".as_ptr() as *const i8);
        (*FCB).PollState |= AFD_EVENT_RECEIVE;
        (*FCB).PollStatus[FD_READ_BIT as usize] = STATUS_SUCCESS;
        PollReeval(((*FCB).DeviceExt as PVOID) as _, (*FCB).FileObject);
    } else {
        (*FCB).PollState &= !AFD_EVENT_RECEIVE;
    }

    if NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) && (*FCB).Recv.Content < (*FCB).Recv.Size {
        TdiReceiveDatagram(
            core::ptr::addr_of_mut!((*FCB).ReceiveIrp.InFlightRequest),
            (*FCB).AddressFile.Object,
            0,
            (*FCB).Recv.Window,
            (*FCB).Recv.Size,
            (*FCB).AddressFrom,
            Some(PacketSocketRecvComplete),
            FCB as PVOID,
        );
    }

    SocketStateUnlock(FCB);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn AfdPacketSocketReadData(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let RecvReq: PAFD_RECV_INFO_UDP;
    let ListEntry: PLIST_ENTRY;
    let DatagramRecv: PAFD_STORED_DATAGRAM;
    let mut LockMode: KPROCESSOR_MODE = 0;

    AFD_DbgPrint!(
        MID_TRACE,
        b"Called on %p\n\0".as_ptr() as PCHAR,
        FCB,
    );

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    (*FCB).EventSelectDisabled &= !AFD_EVENT_RECEIVE;

    if (*FCB).SharedData.State != SOCKET_STATE_BOUND {
        AFD_DbgPrint!(
            MIN_TRACE,
            b"Invalid socket state\n\0".as_ptr() as PCHAR,
        );
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
    }

    if (*FCB).TdiReceiveClosed != 0 {
        AFD_DbgPrint!(
            MIN_TRACE,
            b"Receive closed\n\0".as_ptr() as PCHAR,
        );
        return UnlockAndMaybeComplete(FCB, STATUS_FILE_CLOSED, Irp, 0);
    }

    RecvReq = LockRequest(Irp, IrpSp, 0u8, core::ptr::addr_of_mut!(LockMode)) as PAFD_RECV_INFO_UDP;
    if RecvReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"Recv flags %x\n\0".as_ptr() as PCHAR,
        (*RecvReq).AfdFlags,
    );

    (*RecvReq).BufferArray = LockBuffers(
        (*RecvReq).BufferArray,
        (*RecvReq).BufferCount,
        (*RecvReq).Address,
        (*RecvReq).AddressLength,
        1u8,
        1u8,
        LockMode,
    );

    if (*RecvReq).BufferArray.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_ACCESS_VIOLATION, Irp, 0);
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*FCB).DatagramList)) {
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).DatagramList));
        DatagramRecv = (ListEntry as *mut u8).offset(
            -(core::mem::offset_of!(_AFD_STORED_DATAGRAM, ListEntry) as isize)
        ) as PAFD_STORED_DATAGRAM;
        Status = SatisfyPacketRecvRequest(
            FCB,
            Irp,
            DatagramRecv,
            core::ptr::addr_of_mut!((*Irp).IoStatus.Information) as PUINT,
        );

        if (*RecvReq).TdiFlags & TDI_RECEIVE_PEEK != 0 {
            InsertHeadList(
                core::ptr::addr_of_mut!((*FCB).DatagramList),
                core::ptr::addr_of_mut!((*DatagramRecv).ListEntry),
            );
        }

        if !IsListEmpty(core::ptr::addr_of_mut!((*FCB).DatagramList)) {
            (*FCB).PollState |= AFD_EVENT_RECEIVE;
            (*FCB).PollStatus[FD_READ_BIT as usize] = STATUS_SUCCESS;
            PollReeval(((*FCB).DeviceExt as PVOID) as _, (*FCB).FileObject);
        } else {
            (*FCB).PollState &= !AFD_EVENT_RECEIVE;
        }

        UnlockBuffers(((*RecvReq).BufferArray as PVOID).cast(), (*RecvReq).BufferCount, (1u8) as _);

        return UnlockAndMaybeComplete(FCB, Status, Irp, (*Irp).IoStatus.Information as u32);
    } else if ((*RecvReq).AfdFlags & AFD_OVERLAPPED) == 0
        && (((*RecvReq).AfdFlags & AFD_IMMEDIATE) != 0 || (*FCB).NonBlocking != 0)
    {
        AFD_DbgPrint!(
            MID_TRACE,
            b"Nonblocking\n\0".as_ptr() as PCHAR,
        );
        Status = STATUS_CANT_WAIT;
        (*FCB).PollState &= !AFD_EVENT_RECEIVE;
        UnlockBuffers(((*RecvReq).BufferArray as PVOID).cast(), (*RecvReq).BufferCount, (1u8) as _);
        return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
    } else {
        (*FCB).PollState &= !AFD_EVENT_RECEIVE;
        return LeaveIrpUntilLater(FCB, Irp, FUNCTION_RECV);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/net/afd/afd/read.c
//  * PURPOSE:          Ancillary functions driver
//  * PROGRAMMER:       Art Yerkes (ayerkes@speakeasy.net)
//  * UPDATE HISTORY:
//  * 20040708 Created
//  */
//
// #include "afd.h"
//
// static VOID RefillSocketBuffer( PAFD_FCB FCB )
// {
//     /* Make sure nothing's in flight first */
//     if (FCB->ReceiveIrp.InFlightRequest) return;
//
//     /* Now ensure that receive is still allowed */
//     if (FCB->TdiReceiveClosed) return;
//
//     /* Check if the buffer is full */
//     if (FCB->Recv.Content == FCB->Recv.Size)
//     {
//         /* If there are bytes used, we can solve this problem */
//         if (FCB->Recv.BytesUsed != 0)
//         {
//             /* Reposition the unused portion to the beginning of the receive window */
//             RtlMoveMemory(FCB->Recv.Window,
//                           FCB->Recv.Window + FCB->Recv.BytesUsed,
//                           FCB->Recv.Content - FCB->Recv.BytesUsed);
//
//             FCB->Recv.Content -= FCB->Recv.BytesUsed;
//             FCB->Recv.BytesUsed = 0;
//         }
//         else
//         {
//             /* No space in the buffer to receive */
//             return;
//         }
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("Replenishing buffer\n"));
//
//     TdiReceive( &FCB->ReceiveIrp.InFlightRequest,
//                 FCB->Connection.Object,
//                 TDI_RECEIVE_NORMAL,
//                 FCB->Recv.Window + FCB->Recv.Content,
//                 FCB->Recv.Size - FCB->Recv.Content,
//                 ReceiveComplete,
//                 FCB );
// }
//
// static VOID HandleReceiveComplete( PAFD_FCB FCB, NTSTATUS Status, ULONG_PTR Information )
// {
//     FCB->LastReceiveStatus = Status;
//
//     /* We got closed while the receive was in progress */
//     if (FCB->TdiReceiveClosed)
//     {
//         /* The received data is discarded */
//     }
//     /* Receive successful */
//     else if (Status == STATUS_SUCCESS)
//     {
//         FCB->Recv.Content += Information;
//         ASSERT(FCB->Recv.Content <= FCB->Recv.Size);
//
//         /* Check for graceful closure */
//         if (Information == 0)
//         {
//             /* Receive is closed */
//             FCB->TdiReceiveClosed = TRUE;
//         }
//         else
//         {
//             /* Issue another receive IRP to keep the buffer well stocked */
//             RefillSocketBuffer(FCB);
//         }
//     }
//     /* Receive failed with no data (unexpected closure) */
//     else
//     {
//         /* Previously received data remains intact */
//         FCB->TdiReceiveClosed = TRUE;
//     }
// }
//
// static BOOLEAN CantReadMore( PAFD_FCB FCB ) {
//     UINT BytesAvailable = FCB->Recv.Content - FCB->Recv.BytesUsed;
//
//     return !BytesAvailable && FCB->TdiReceiveClosed;
// }
//
// static NTSTATUS TryToSatisfyRecvRequestFromBuffer( PAFD_FCB FCB,
//                                                    PAFD_RECV_INFO RecvReq,
//                                                    PUINT TotalBytesCopied ) {
//     UINT i, BytesToCopy = 0, FcbBytesCopied = FCB->Recv.BytesUsed,
//         BytesAvailable =
//         FCB->Recv.Content - FCB->Recv.BytesUsed;
//     PAFD_MAPBUF Map;
//     *TotalBytesCopied = 0;
//
//
//     AFD_DbgPrint!(MID_TRACE,("Called, BytesAvailable = %u\n", BytesAvailable));
//
//     if( CantReadMore(FCB) ) return STATUS_SUCCESS;
//     if( !BytesAvailable ) return STATUS_PENDING;
//
//     Map = (PAFD_MAPBUF)(RecvReq->BufferArray + RecvReq->BufferCount);
//
//     AFD_DbgPrint!(MID_TRACE,("Buffer Count: %u @ %p\n",
//                             RecvReq->BufferCount,
//                             RecvReq->BufferArray));
//     for( i = 0;
//          RecvReq->BufferArray &&
//              BytesAvailable &&
//              i < RecvReq->BufferCount;
//          i++ ) {
//         BytesToCopy =
//             MIN( RecvReq->BufferArray[i].len, BytesAvailable );
//
//         if( Map[i].Mdl ) {
//             Map[i].BufferAddress = MmMapLockedPages( Map[i].Mdl, KernelMode );
//
//             AFD_DbgPrint!(MID_TRACE,("Buffer %u: %p:%u\n",
//                                     i,
//                                     Map[i].BufferAddress,
//                                     BytesToCopy));
//
//             RtlCopyMemory( Map[i].BufferAddress,
//                            FCB->Recv.Window + FcbBytesCopied,
//                            BytesToCopy );
//
//             MmUnmapLockedPages( Map[i].BufferAddress, Map[i].Mdl );
//
//             *TotalBytesCopied += BytesToCopy;
//             FcbBytesCopied += BytesToCopy;
//             BytesAvailable -= BytesToCopy;
//
//             if (!(RecvReq->TdiFlags & TDI_RECEIVE_PEEK))
//                 FCB->Recv.BytesUsed += BytesToCopy;
//         }
//     }
//
//     /* Issue another receive IRP to keep the buffer well stocked */
//     RefillSocketBuffer(FCB);
//
//     return STATUS_SUCCESS;
// }
//
// static NTSTATUS ReceiveActivity( PAFD_FCB FCB, PIRP Irp ) {
//     PLIST_ENTRY NextIrpEntry;
//     PIRP NextIrp;
//     PIO_STACK_LOCATION NextIrpSp;
//     PAFD_RECV_INFO RecvReq;
//     UINT TotalBytesCopied = 0;
//     NTSTATUS Status = STATUS_SUCCESS, RetStatus = STATUS_PENDING;
//
//     AFD_DbgPrint!(MID_TRACE,("%p %p\n", FCB, Irp));
//
//     AFD_DbgPrint!(MID_TRACE,("FCB %p Receive data waiting %u\n",
//                             FCB, FCB->Recv.Content));
//
//     if( CantReadMore( FCB ) ) {
//         /* Success here means that we got an EOF.  Complete a pending read
//          * with zero bytes if we haven't yet overread, then kill the others.
//          */
//         while( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_RECV as usize] ) ) {
//             NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_RECV as usize]);
//             NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//             NextIrpSp = IoGetCurrentIrpStackLocation( NextIrp );
//             RecvReq = GetLockedData(NextIrp, NextIrpSp);
//
//             AFD_DbgPrint!(MID_TRACE,("Completing recv %p (%u)\n", NextIrp,
//                                     TotalBytesCopied));
//             UnlockBuffers( RecvReq->BufferArray,
//                            RecvReq->BufferCount, FALSE );
//
//             /* Unexpected disconnect by the remote host or graceful disconnect */
//             Status = FCB->LastReceiveStatus;
//
//             NextIrp->IoStatus.Status = Status;
//             NextIrp->IoStatus.Information = 0;
//             if( NextIrp == Irp ) RetStatus = Status;
//             if( NextIrp->MdlAddress ) UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//             (void)IoSetCancelRoutine(NextIrp, NULL);
//             IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//         }
//     } else {
//         /* Kick the user that receive would be possible now */
//         /* XXX Not implemented yet */
//
//         AFD_DbgPrint!(MID_TRACE,("FCB %p Receive data waiting %u\n",
//                                 FCB, FCB->Recv.Content));
//         /*OskitDumpBuffer( FCB->Recv.Window, FCB->Recv.Content );*/
//
//         /* Try to clear some requests */
//         while( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_RECV as usize] ) ) {
//             NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_RECV as usize]);
//             NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//             NextIrpSp = IoGetCurrentIrpStackLocation( NextIrp );
//             RecvReq = GetLockedData(NextIrp, NextIrpSp);
//
//             AFD_DbgPrint!(MID_TRACE,("RecvReq @ %p\n", RecvReq));
//
//             Status = TryToSatisfyRecvRequestFromBuffer
//             ( FCB, RecvReq, &TotalBytesCopied );
//
//             if( Status == STATUS_PENDING ) {
//                 AFD_DbgPrint!(MID_TRACE,("Ran out of data for %p\n", NextIrp));
//                 InsertHeadList(&FCB->PendingIrpList[FUNCTION_RECV as usize],
//                                &NextIrp->Tail.Overlay.ListEntry);
//                 break;
//             } else {
//                 AFD_DbgPrint!(MID_TRACE,("Completing recv %p (%u)\n", NextIrp,
//                                         TotalBytesCopied));
//                 UnlockBuffers( RecvReq->BufferArray,
//                                RecvReq->BufferCount, FALSE );
//                 NextIrp->IoStatus.Status = Status;
//                 NextIrp->IoStatus.Information = TotalBytesCopied;
//                 if( NextIrp == Irp ) {
//                     RetStatus = Status;
//                 }
//                 if( NextIrp->MdlAddress ) UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//                 (void)IoSetCancelRoutine(NextIrp, NULL);
//                 IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//             }
//         }
//     }
//
//     if( FCB->Recv.Content - FCB->Recv.BytesUsed &&
//         IsListEmpty(&FCB->PendingIrpList[FUNCTION_RECV as usize]) ) {
//         FCB->PollState |= AFD_EVENT_RECEIVE;
//         FCB->PollStatus[FD_READ_BIT] = STATUS_SUCCESS;
//         PollReeval( FCB->DeviceExt, FCB->FileObject );
//     }
//     else
//     {
//         FCB->PollState &= ~AFD_EVENT_RECEIVE;
//     }
//
//     /* Signal FD_CLOSE if no buffered data remains and the socket can't receive any more */
//     if (CantReadMore(FCB))
//     {
//         if (FCB->LastReceiveStatus == STATUS_SUCCESS)
//         {
//             FCB->PollState |= AFD_EVENT_DISCONNECT;
//         }
//         else
//         {
//             FCB->PollState |= AFD_EVENT_CLOSE;
//         }
//         FCB->PollStatus[FD_CLOSE_BIT] = FCB->LastReceiveStatus;
//         PollReeval(FCB->DeviceExt, FCB->FileObject);
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("RetStatus for irp %p is %x\n", Irp, RetStatus));
//
//     return RetStatus;
// }
//
// NTSTATUS NTAPI ReceiveComplete
// ( PDEVICE_OBJECT DeviceObject,
//   PIRP Irp,
//   PVOID Context ) {
//     PAFD_FCB FCB = (PAFD_FCB)Context;
//     PLIST_ENTRY NextIrpEntry;
//     PIRP NextIrp;
//     PAFD_RECV_INFO RecvReq;
//     PIO_STACK_LOCATION NextIrpSp;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     AFD_DbgPrint!(MID_TRACE,("Called\n"));
//
//     if( !SocketAcquireStateLock( FCB ) )
//         return STATUS_FILE_CLOSED;
//
//     ASSERT(FCB->ReceiveIrp.InFlightRequest == Irp);
//     FCB->ReceiveIrp.InFlightRequest = NULL;
//
//     if( FCB->SharedData.State == SOCKET_STATE_CLOSED ) {
//         /* Cleanup our IRP queue because the FCB is being destroyed */
//         while( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_RECV as usize] ) ) {
//             NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_RECV as usize]);
//             NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//             NextIrpSp = IoGetCurrentIrpStackLocation(NextIrp);
//             RecvReq = GetLockedData(NextIrp, NextIrpSp);
//             NextIrp->IoStatus.Status = STATUS_FILE_CLOSED;
//             NextIrp->IoStatus.Information = 0;
//             UnlockBuffers(RecvReq->BufferArray, RecvReq->BufferCount, FALSE);
//             if( NextIrp->MdlAddress ) UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//             (void)IoSetCancelRoutine(NextIrp, NULL);
//             IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//         }
//         SocketStateUnlock( FCB );
//         return STATUS_FILE_CLOSED;
//     } else if( FCB->SharedData.State == SOCKET_STATE_LISTENING ) {
//         AFD_DbgPrint!(MIN_TRACE,("!!! LISTENER GOT A RECEIVE COMPLETE !!!\n"));
//         SocketStateUnlock( FCB );
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     HandleReceiveComplete( FCB, Irp->IoStatus.Status, Irp->IoStatus.Information );
//
//     ReceiveActivity( FCB, NULL );
//
//     SocketStateUnlock( FCB );
//
//     return STATUS_SUCCESS;
// }
//
// static NTSTATUS NTAPI
// SatisfyPacketRecvRequest( PAFD_FCB FCB, PIRP Irp,
//                          PAFD_STORED_DATAGRAM DatagramRecv,
//                          PUINT TotalBytesCopied ) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//     PAFD_RECV_INFO RecvReq =
//     GetLockedData(Irp, IrpSp);
//     UINT BytesToCopy = 0, BytesAvailable = DatagramRecv->Len, AddrLen = 0;
//     PAFD_MAPBUF Map;
//     BOOLEAN ExtraBuffers = CheckUnlockExtraBuffers(FCB, IrpSp);
//
//     Map = (PAFD_MAPBUF)(RecvReq->BufferArray +
//                         RecvReq->BufferCount +
//                         (ExtraBuffers ? EXTRA_LOCK_BUFFERS : 0));
//
//     BytesToCopy = MIN( RecvReq->BufferArray[0].len, BytesAvailable );
//
//     AFD_DbgPrint!(MID_TRACE,("BytesToCopy: %u len %u\n", BytesToCopy,
//                             RecvReq->BufferArray[0].len));
//
//     if( Map[0].Mdl ) {
//         /* Copy the address */
//         if( ExtraBuffers && Map[1].Mdl && Map[2].Mdl ) {
//             AFD_DbgPrint!(MID_TRACE,("Checking TAAddressCount\n"));
//
//             if( DatagramRecv->Address->TAAddressCount != 1 ) {
//                 AFD_DbgPrint
//                 (MIN_TRACE,
//                  ("Wierd address count %d\n",
//                   DatagramRecv->Address->TAAddressCount));
//             }
//
//             AFD_DbgPrint!(MID_TRACE,("Computing addr len\n"));
//
//             AddrLen = MIN(DatagramRecv->Address->Address->AddressLength +
//                           sizeof(USHORT),
//                           RecvReq->BufferArray[1].len);
//
//             AFD_DbgPrint!(MID_TRACE,("Copying %u bytes of address\n", AddrLen));
//
//             Map[1].BufferAddress = MmMapLockedPages( Map[1].Mdl, KernelMode );
//
//             AFD_DbgPrint!(MID_TRACE,("Done mapping, copying address\n"));
//
//             RtlCopyMemory( Map[1].BufferAddress,
//                           &DatagramRecv->Address->Address->AddressType,
//                           AddrLen );
//
//             MmUnmapLockedPages( Map[1].BufferAddress, Map[1].Mdl );
//
//             AFD_DbgPrint!(MID_TRACE,("Copying address len\n"));
//
//             Map[2].BufferAddress = MmMapLockedPages( Map[2].Mdl, KernelMode );
//             *((PINT)Map[2].BufferAddress) = AddrLen;
//             MmUnmapLockedPages( Map[2].BufferAddress, Map[2].Mdl );
//         }
//
//         AFD_DbgPrint!(MID_TRACE,("Mapping data buffer pages\n"));
//
//         Map[0].BufferAddress = MmMapLockedPages( Map[0].Mdl, KernelMode );
//
//         AFD_DbgPrint!(MID_TRACE,("Buffer %d: %p:%u\n",
//                                 0,
//                                 Map[0].BufferAddress,
//                                 BytesToCopy));
//
//         RtlCopyMemory( Map[0].BufferAddress,
//                       DatagramRecv->Buffer,
//                       BytesToCopy );
//
//         MmUnmapLockedPages( Map[0].BufferAddress, Map[0].Mdl );
//
//         *TotalBytesCopied = BytesToCopy;
//     }
//
//     if (*TotalBytesCopied == DatagramRecv->Len)
//     {
//         /* We copied the whole datagram */
//         Status = Irp->IoStatus.Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* We only copied part of the datagram */
//         Status = Irp->IoStatus.Status = STATUS_BUFFER_OVERFLOW;
//     }
//
//     Irp->IoStatus.Information = *TotalBytesCopied;
//
//     if (!(RecvReq->TdiFlags & TDI_RECEIVE_PEEK))
//     {
//         FCB->Recv.Content -= DatagramRecv->Len;
//         ExFreePoolWithTag(DatagramRecv->Address, TAG_AFD_TRANSPORT_ADDRESS);
//         ExFreePoolWithTag(DatagramRecv, TAG_AFD_STORED_DATAGRAM);
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("Done\n"));
//
//     return Status;
// }
//
// NTSTATUS NTAPI
// AfdConnectedSocketReadData(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                            PIO_STACK_LOCATION IrpSp, BOOLEAN Short) {
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PAFD_RECV_INFO RecvReq;
//     UINT TotalBytesCopied = 0;
//     PAFD_STORED_DATAGRAM DatagramRecv;
//     PLIST_ENTRY ListEntry;
//     KPROCESSOR_MODE LockMode;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//     UNREFERENCED_PARAMETER(Short);
//
//     AFD_DbgPrint!(MID_TRACE,("Called on %p\n", FCB));
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     FCB->EventSelectDisabled &= ~AFD_EVENT_RECEIVE;
//
//     if( !(FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS) &&
//         FCB->SharedData.State != SOCKET_STATE_CONNECTED &&
//         FCB->SharedData.State != SOCKET_STATE_CONNECTING ) {
//         AFD_DbgPrint!(MIN_TRACE,("Called recv on wrong kind of socket (s%x)\n",
//                                 FCB->SharedData.State));
//         return UnlockAndMaybeComplete( FCB, STATUS_INVALID_PARAMETER,
//                                        Irp, 0 );
//     }
//
//     if( !(RecvReq = LockRequest( Irp, IrpSp, FALSE, &LockMode )) )
//         return UnlockAndMaybeComplete( FCB, STATUS_NO_MEMORY,
//                                        Irp, 0 );
//
//     AFD_DbgPrint!(MID_TRACE,("Recv flags %x\n", RecvReq->AfdFlags));
//
//     RecvReq->BufferArray = LockBuffers( RecvReq->BufferArray,
//                                        RecvReq->BufferCount,
//                                        NULL, NULL,
//                                        TRUE, FALSE, LockMode );
//
//     if( !RecvReq->BufferArray ) {
//         return UnlockAndMaybeComplete( FCB, STATUS_ACCESS_VIOLATION,
//                                       Irp, 0 );
//     }
//
//     if( FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS )
//     {
//         if (!IsListEmpty(&FCB->DatagramList))
//         {
//             ListEntry = RemoveHeadList(&FCB->DatagramList);
//             DatagramRecv = CONTAINING_RECORD(ListEntry, AFD_STORED_DATAGRAM, ListEntry);
//             Status = SatisfyPacketRecvRequest(FCB, Irp, DatagramRecv,
//                                               (PUINT)&Irp->IoStatus.Information);
//
//             if (RecvReq->TdiFlags & TDI_RECEIVE_PEEK)
//             {
//                 InsertHeadList(&FCB->DatagramList,
//                                &DatagramRecv->ListEntry);
//             }
//
//             if (!IsListEmpty(&FCB->DatagramList))
//             {
//                 FCB->PollState |= AFD_EVENT_RECEIVE;
//                 FCB->PollStatus[FD_READ_BIT] = STATUS_SUCCESS;
//                 PollReeval( FCB->DeviceExt, FCB->FileObject );
//             }
//             else
//                 FCB->PollState &= ~AFD_EVENT_RECEIVE;
//
//             UnlockBuffers(RecvReq->BufferArray, RecvReq->BufferCount, FALSE);
//
//             return UnlockAndMaybeComplete(FCB, Status, Irp, Irp->IoStatus.Information);
//         }
//         else if (!(RecvReq->AfdFlags & AFD_OVERLAPPED) &&
//                 ((RecvReq->AfdFlags & AFD_IMMEDIATE) || (FCB->NonBlocking)))
//         {
//             AFD_DbgPrint!(MID_TRACE,("Nonblocking\n"));
//             Status = STATUS_CANT_WAIT;
//             FCB->PollState &= ~AFD_EVENT_RECEIVE;
//             UnlockBuffers( RecvReq->BufferArray, RecvReq->BufferCount, FALSE );
//             return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
//         }
//         else
//         {
//             FCB->PollState &= ~AFD_EVENT_RECEIVE;
//             return LeaveIrpUntilLater( FCB, Irp, FUNCTION_RECV );
//         }
//     }
//
//     Irp->IoStatus.Status = STATUS_PENDING;
//     Irp->IoStatus.Information = 0;
//
//     InsertTailList( &FCB->PendingIrpList[FUNCTION_RECV as usize],
//                     &Irp->Tail.Overlay.ListEntry );
//
//     /************ From this point, the IRP is not ours ************/
//
//     Status = ReceiveActivity( FCB, Irp );
//
//     if( Status == STATUS_PENDING &&
//         !(RecvReq->AfdFlags & AFD_OVERLAPPED) &&
//         ((RecvReq->AfdFlags & AFD_IMMEDIATE) || (FCB->NonBlocking))) {
//         AFD_DbgPrint!(MID_TRACE,("Nonblocking\n"));
//         Status = STATUS_CANT_WAIT;
//         TotalBytesCopied = 0;
//         RemoveEntryList( &Irp->Tail.Overlay.ListEntry );
//         UnlockBuffers( RecvReq->BufferArray, RecvReq->BufferCount, FALSE );
//         return UnlockAndMaybeComplete( FCB, Status, Irp,
//                                        TotalBytesCopied );
//     } else if( Status == STATUS_PENDING ) {
//         AFD_DbgPrint!(MID_TRACE,("Leaving read irp\n"));
//         IoMarkIrpPending( Irp );
//         (void)IoSetCancelRoutine(Irp, AfdCancelHandler);
//     } else {
//         AFD_DbgPrint!(MID_TRACE,("Completed with status %x\n", Status));
//     }
//
//     SocketStateUnlock( FCB );
//     return Status;
// }
//
// NTSTATUS NTAPI
// PacketSocketRecvComplete(
//         PDEVICE_OBJECT DeviceObject,
//         PIRP Irp,
//         PVOID Context ) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAFD_FCB FCB = Context;
//     PIRP NextIrp;
//     PIO_STACK_LOCATION NextIrpSp;
//     PLIST_ENTRY ListEntry;
//     PAFD_RECV_INFO RecvReq;
//     PAFD_STORED_DATAGRAM DatagramRecv;
//     UINT DGSize = Irp->IoStatus.Information + sizeof( AFD_STORED_DATAGRAM );
//     PLIST_ENTRY NextIrpEntry, DatagramRecvEntry;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     AFD_DbgPrint!(MID_TRACE,("Called on %p\n", FCB));
//
//     if( !SocketAcquireStateLock( FCB ) )
//         return STATUS_FILE_CLOSED;
//
//     ASSERT(FCB->ReceiveIrp.InFlightRequest == Irp);
//     FCB->ReceiveIrp.InFlightRequest = NULL;
//
//     if( FCB->SharedData.State == SOCKET_STATE_CLOSED ) {
//         /* Cleanup our IRP queue because the FCB is being destroyed */
//         while( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_RECV as usize] ) ) {
//             NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_RECV as usize]);
//             NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//             NextIrpSp = IoGetCurrentIrpStackLocation( NextIrp );
//             RecvReq = GetLockedData(NextIrp, NextIrpSp);
//             NextIrp->IoStatus.Status = STATUS_FILE_CLOSED;
//             NextIrp->IoStatus.Information = 0;
//             UnlockBuffers(RecvReq->BufferArray, RecvReq->BufferCount, CheckUnlockExtraBuffers(FCB, NextIrpSp));
//             if( NextIrp->MdlAddress ) UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//             (void)IoSetCancelRoutine(NextIrp, NULL);
//             IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//         }
//
//         /* Free all items on the datagram list */
//         while( !IsListEmpty( &FCB->DatagramList ) ) {
//                DatagramRecvEntry = RemoveHeadList(&FCB->DatagramList);
//                DatagramRecv = CONTAINING_RECORD(DatagramRecvEntry, AFD_STORED_DATAGRAM, ListEntry);
//                ExFreePoolWithTag(DatagramRecv->Address, TAG_AFD_TRANSPORT_ADDRESS);
//                ExFreePoolWithTag(DatagramRecv, TAG_AFD_STORED_DATAGRAM);
//         }
//
//         SocketStateUnlock( FCB );
//         return STATUS_FILE_CLOSED;
//     }
//
//     if (Irp->IoStatus.Status != STATUS_SUCCESS)
//     {
//         SocketStateUnlock(FCB);
//         return Irp->IoStatus.Status;
//     }
//
//     if (FCB->TdiReceiveClosed)
//     {
//         SocketStateUnlock(FCB);
//         return STATUS_FILE_CLOSED;
//     }
//
//     DatagramRecv = ExAllocatePoolWithTag(NonPagedPool,
//                                          DGSize,
//                                          TAG_AFD_STORED_DATAGRAM);
//
//     if( DatagramRecv ) {
//         DatagramRecv->Len = Irp->IoStatus.Information;
//         RtlCopyMemory( DatagramRecv->Buffer, FCB->Recv.Window,
//                        DatagramRecv->Len );
//         AFD_DbgPrint!(MID_TRACE,("Received (A %p)\n",
//                                 FCB->AddressFrom->RemoteAddress));
//         DatagramRecv->Address =
//             TaCopyTransportAddress( FCB->AddressFrom->RemoteAddress );
//
//         if( !DatagramRecv->Address ) Status = STATUS_NO_MEMORY;
//
//     } else Status = STATUS_NO_MEMORY;
//
//     if( !NT_SUCCESS( Status ) ) {
//
//         if (DatagramRecv)
//         {
//             ExFreePoolWithTag(DatagramRecv, TAG_AFD_STORED_DATAGRAM);
//         }
//
//         SocketStateUnlock( FCB );
//         return Status;
//     } else {
//         FCB->Recv.Content += DatagramRecv->Len;
//         InsertTailList( &FCB->DatagramList, &DatagramRecv->ListEntry );
//     }
//
//     /* Satisfy as many requests as we can */
//
//     while( !IsListEmpty( &FCB->DatagramList ) &&
//            !IsListEmpty( &FCB->PendingIrpList[FUNCTION_RECV as usize] ) ) {
//         AFD_DbgPrint!(MID_TRACE,("Looping trying to satisfy request\n"));
//         ListEntry = RemoveHeadList( &FCB->DatagramList );
//         DatagramRecv = CONTAINING_RECORD( ListEntry, AFD_STORED_DATAGRAM,
//                                           ListEntry );
//         ListEntry = RemoveHeadList( &FCB->PendingIrpList[FUNCTION_RECV as usize] );
//         NextIrp = CONTAINING_RECORD( ListEntry, IRP, Tail.Overlay.ListEntry );
//         NextIrpSp = IoGetCurrentIrpStackLocation( NextIrp );
//         RecvReq = GetLockedData(NextIrp, NextIrpSp);
//
//         AFD_DbgPrint!(MID_TRACE,("RecvReq: %p, DatagramRecv: %p\n",
//                                 RecvReq, DatagramRecv));
//
//         AFD_DbgPrint!(MID_TRACE,("Satisfying\n"));
//         Status = SatisfyPacketRecvRequest
//         ( FCB, NextIrp, DatagramRecv,
//          (PUINT)&NextIrp->IoStatus.Information );
//
//         if (RecvReq->TdiFlags & TDI_RECEIVE_PEEK)
//         {
//             InsertHeadList(&FCB->DatagramList,
//                            &DatagramRecv->ListEntry);
//         }
//
//         AFD_DbgPrint!(MID_TRACE,("Unlocking\n"));
//         UnlockBuffers( RecvReq->BufferArray, RecvReq->BufferCount, CheckUnlockExtraBuffers(FCB, NextIrpSp) );
//         if ( NextIrp->MdlAddress ) UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//
//         AFD_DbgPrint!(MID_TRACE,("Completing\n"));
//         (void)IoSetCancelRoutine(NextIrp, NULL);
//         NextIrp->IoStatus.Status = Status;
//
//         IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//     }
//
//     if( !IsListEmpty( &FCB->DatagramList ) && IsListEmpty(&FCB->PendingIrpList[FUNCTION_RECV as usize]) ) {
//         AFD_DbgPrint!(MID_TRACE,("Signalling\n"));
//         FCB->PollState |= AFD_EVENT_RECEIVE;
//         FCB->PollStatus[FD_READ_BIT] = STATUS_SUCCESS;
//         PollReeval( FCB->DeviceExt, FCB->FileObject );
//     } else
//         FCB->PollState &= ~AFD_EVENT_RECEIVE;
//
//     if( NT_SUCCESS(Irp->IoStatus.Status) && FCB->Recv.Content < FCB->Recv.Size ) {
//         /* Now relaunch the datagram request */
//         Status = TdiReceiveDatagram
//             ( &FCB->ReceiveIrp.InFlightRequest,
//               FCB->AddressFile.Object,
//               0,
//               FCB->Recv.Window,
//               FCB->Recv.Size,
//               FCB->AddressFrom,
//               PacketSocketRecvComplete,
//               FCB );
//     }
//
//     SocketStateUnlock( FCB );
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS NTAPI
// AfdPacketSocketReadData(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                         PIO_STACK_LOCATION IrpSp ) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PAFD_RECV_INFO_UDP RecvReq;
//     PLIST_ENTRY ListEntry;
//     PAFD_STORED_DATAGRAM DatagramRecv;
//     KPROCESSOR_MODE LockMode;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     AFD_DbgPrint!(MID_TRACE,("Called on %p\n", FCB));
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     FCB->EventSelectDisabled &= ~AFD_EVENT_RECEIVE;
//
//     /* Check that the socket is bound */
//     if( FCB->SharedData.State != SOCKET_STATE_BOUND )
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Invalid socket state\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
//     }
//
//     if (FCB->TdiReceiveClosed)
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Receive closed\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_FILE_CLOSED, Irp, 0);
//     }
//
//     if( !(RecvReq = LockRequest( Irp, IrpSp, FALSE, &LockMode )) )
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     AFD_DbgPrint!(MID_TRACE,("Recv flags %x\n", RecvReq->AfdFlags));
//
//     RecvReq->BufferArray = LockBuffers( RecvReq->BufferArray,
//                                         RecvReq->BufferCount,
//                                         RecvReq->Address,
//                                         RecvReq->AddressLength,
//                                         TRUE, TRUE, LockMode );
//
//     if( !RecvReq->BufferArray ) { /* access violation in userspace */
//         return UnlockAndMaybeComplete(FCB, STATUS_ACCESS_VIOLATION, Irp, 0);
//     }
//
//     if (!IsListEmpty(&FCB->DatagramList))
//     {
//         ListEntry = RemoveHeadList(&FCB->DatagramList);
//         DatagramRecv = CONTAINING_RECORD(ListEntry, AFD_STORED_DATAGRAM, ListEntry);
//         Status = SatisfyPacketRecvRequest(FCB, Irp, DatagramRecv,
//                                           (PUINT)&Irp->IoStatus.Information);
//
//         if (RecvReq->TdiFlags & TDI_RECEIVE_PEEK)
//         {
//             InsertHeadList(&FCB->DatagramList,
//                            &DatagramRecv->ListEntry);
//         }
//
//         if (!IsListEmpty(&FCB->DatagramList))
//         {
//             FCB->PollState |= AFD_EVENT_RECEIVE;
//             FCB->PollStatus[FD_READ_BIT] = STATUS_SUCCESS;
//             PollReeval( FCB->DeviceExt, FCB->FileObject );
//         }
//         else
//             FCB->PollState &= ~AFD_EVENT_RECEIVE;
//
//         UnlockBuffers(RecvReq->BufferArray, RecvReq->BufferCount, TRUE);
//
//         return UnlockAndMaybeComplete(FCB, Status, Irp, Irp->IoStatus.Information);
//     }
//     else if (!(RecvReq->AfdFlags & AFD_OVERLAPPED) &&
//             ((RecvReq->AfdFlags & AFD_IMMEDIATE) || (FCB->NonBlocking)))
//     {
//         AFD_DbgPrint!(MID_TRACE,("Nonblocking\n"));
//         Status = STATUS_CANT_WAIT;
//         FCB->PollState &= ~AFD_EVENT_RECEIVE;
//         UnlockBuffers( RecvReq->BufferArray, RecvReq->BufferCount, TRUE );
//         return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
//     }
//     else
//     {
//         FCB->PollState &= ~AFD_EVENT_RECEIVE;
//         return LeaveIrpUntilLater( FCB, Irp, FUNCTION_RECV );
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
