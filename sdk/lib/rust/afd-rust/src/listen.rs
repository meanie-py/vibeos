use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ddk::{IRP, ObReferenceObjectByHandle, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSetCancelRoutine};

unsafe fn SatisfyAccept(
    DeviceExt: PAFD_DEVICE_EXTENSION,
    Irp: PIRP,
    NewFileObject: PFILE_OBJECT,
    Qelt: PAFD_TDI_OBJECT_QELT,
) -> NTSTATUS {
    let FCB = (*NewFileObject).FsContext as PAFD_FCB;
    let mut Status: NTSTATUS;

    let _ = DeviceExt;

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    AFD_DbgPrint!(
        MID_TRACE,
        b"Completing a real accept (FCB %p)\n\0".as_ptr() as PCHAR,
        FCB,
    );

    (*FCB).Connection = core::ptr::read(core::ptr::addr_of!((*Qelt).Object));

    if !(*FCB).RemoteAddress.is_null() {
        ExFreePoolWithTag((*FCB).RemoteAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
    }

    (*FCB).RemoteAddress = TaCopyTransportAddress((*(*Qelt).ConnInfo).RemoteAddress as PTRANSPORT_ADDRESS);

    if (*FCB).RemoteAddress.is_null() {
        Status = STATUS_NO_MEMORY;
    } else {
        Status = MakeSocketIntoConnection(FCB);
    }

    if NT_SUCCESS(Status) {
        Status = TdiBuildConnectionInfo(
            core::ptr::addr_of_mut!((*FCB).ConnectCallInfo),
            (*FCB).RemoteAddress,
        );
    }

    if NT_SUCCESS(Status) {
        Status = TdiBuildConnectionInfo(
            core::ptr::addr_of_mut!((*FCB).ConnectReturnInfo),
            (*FCB).RemoteAddress,
        );
    }

    UnlockAndMaybeComplete(FCB, Status, Irp, 0)
}

unsafe fn SatisfyPreAccept(Irp: PIRP, Qelt: PAFD_TDI_OBJECT_QELT) -> NTSTATUS {
    let ListenReceive = (*Irp).AssociatedIrp.SystemBuffer as PAFD_RECEIVED_ACCEPT_DATA;
    let IPAddr: PTA_IP_ADDRESS;

    (*ListenReceive).SequenceNumber = (*Qelt).Seq;

    AFD_DbgPrint!(MID_TRACE, b"Giving SEQ %u to userland\n\0".as_ptr() as PCHAR, (*Qelt).Seq);
    AFD_DbgPrint!(MID_TRACE, b"Socket Address (K) %p (U) %p\n\0".as_ptr() as PCHAR,
                 core::ptr::addr_of_mut!((*ListenReceive).Address),
                 (*(*Qelt).ConnInfo).RemoteAddress);

    TaCopyTransportAddressInPlace(core::ptr::addr_of_mut!((*ListenReceive).Address),
                                  (*(*Qelt).ConnInfo).RemoteAddress as PTRANSPORT_ADDRESS);

    IPAddr = core::ptr::addr_of_mut!((*ListenReceive).Address) as PTA_IP_ADDRESS;

    AFD_DbgPrint!(MID_TRACE, b"IPAddr->TAAddressCount %d\n\0".as_ptr() as PCHAR,
                 (*IPAddr).TAAddressCount);
    AFD_DbgPrint!(MID_TRACE, b"IPAddr->Address[0].AddressType %u\n\0".as_ptr() as PCHAR,
                 (*IPAddr).Address[0].AddressType);
    AFD_DbgPrint!(MID_TRACE, b"IPAddr->Address[0].AddressLength %u\n\0".as_ptr() as PCHAR,
                 (*IPAddr).Address[0].AddressLength);
    AFD_DbgPrint!(MID_TRACE, b"IPAddr->Address[0].Address[0].sin_port %x\n\0".as_ptr() as PCHAR,
                 (*IPAddr).Address[0].Address[0].sin_port);
    AFD_DbgPrint!(MID_TRACE, b"IPAddr->Address[0].Address[0].sin_addr %x\n\0".as_ptr() as PCHAR,
                 (*IPAddr).Address[0].Address[0].in_addr);

    if !(*Irp).MdlAddress.is_null() {
        UnlockRequest(Irp, IoGetCurrentIrpStackLocation(Irp));
    }

    (*Irp).IoStatus.Information = (IPAddr.offset(1) as PCHAR).offset_from(ListenReceive as PCHAR) as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoSetCancelRoutine(Irp, None);
    IoCompleteRequest(Irp, IO_NETWORK_INCREMENT);
    STATUS_SUCCESS
}
unsafe extern "system" fn ListenComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    listen_complete_impl(DeviceObject, Irp, Context)
}

unsafe fn listen_complete_impl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let FCB = Context as PAFD_FCB;
    // Listen-path breadcrumb (see AfdListenSocket).
    DbgPrint(b"[AFD] ListenComplete FCB=%p status=0x%x\n\0".as_ptr() as PCHAR,
             FCB, (*Irp).IoStatus.__bindgen_anon_1.Status);
    let Qelt: PAFD_TDI_OBJECT_QELT;
    let mut NextIrpEntry: PLIST_ENTRY;
    let mut NextIrp: PIRP;

    let _ = DeviceObject;

    if SocketAcquireStateLock(FCB) == 0 {
        return STATUS_FILE_CLOSED;
    }

    ASSERT!((*FCB).ListenIrp.InFlightRequest == Irp);
    (*FCB).ListenIrp.InFlightRequest = core::ptr::null_mut();

    if (*FCB).SharedData.State == SOCKET_STATE_CLOSED {
        while !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_PREACCEPT as usize])) {
            NextIrpEntry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_PREACCEPT as usize]));
            NextIrp = (NextIrpEntry as usize - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP;
            (*NextIrp).IoStatus.__bindgen_anon_1.Status = STATUS_FILE_CLOSED;
            (*NextIrp).IoStatus.Information = 0;
            if !(*NextIrp).MdlAddress.is_null() {
                UnlockRequest(NextIrp, IoGetCurrentIrpStackLocation(NextIrp));
            }
            IoSetCancelRoutine(NextIrp, None);
            IoCompleteRequest(NextIrp, IO_NETWORK_INCREMENT);
        }

        if !(*FCB).ListenIrp.ConnectionReturnInfo.is_null() {
            ExFreePoolWithTag(
                (*FCB).ListenIrp.ConnectionReturnInfo as PVOID,
                TAG_AFD_TDI_CONNECTION_INFORMATION,
            );

            (*FCB).ListenIrp.ConnectionReturnInfo = core::ptr::null_mut();
        }

        if !(*FCB).ListenIrp.ConnectionCallInfo.is_null() {
            ExFreePoolWithTag(
                (*FCB).ListenIrp.ConnectionCallInfo as PVOID,
                TAG_AFD_TDI_CONNECTION_INFORMATION,
            );

            (*FCB).ListenIrp.ConnectionCallInfo = core::ptr::null_mut();
        }

        SocketStateUnlock(FCB);
        return STATUS_FILE_CLOSED;
    }

    AFD_DbgPrint!(MID_TRACE, b"Completing listen request.\n\0".as_ptr() as PCHAR);
    AFD_DbgPrint!(MID_TRACE, b"IoStatus was %x\n\0".as_ptr() as PCHAR, (*Irp).IoStatus.__bindgen_anon_1.Status);

    if (*Irp).IoStatus.__bindgen_anon_1.Status != STATUS_SUCCESS {
        SocketStateUnlock(FCB);
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    Qelt = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<AFD_TDI_OBJECT_QELT>(),
        TAG_AFD_ACCEPT_QUEUE,
    ) as PAFD_TDI_OBJECT_QELT;

    if Qelt.is_null() {
        Status = STATUS_NO_MEMORY;
    } else {
        let AddressType = (*(*FCB).LocalAddress).Address[0].AddressType;

        (*Qelt).Object = core::ptr::read(core::ptr::addr_of!((*FCB).Connection));
        (*Qelt).Seq = (*FCB).ConnSeq;
        (*FCB).ConnSeq = (*FCB).ConnSeq.wrapping_add(1);
        AFD_DbgPrint!(
            MID_TRACE,
            b"Address Type: %u (RA %p)\n\0".as_ptr() as PCHAR,
            AddressType,
            (*(*FCB).ListenIrp.ConnectionReturnInfo).RemoteAddress,
        );

        Status = TdiBuildNullConnectionInfo(core::ptr::addr_of_mut!((*Qelt).ConnInfo), AddressType as u32);
        if NT_SUCCESS(Status) {
            TaCopyTransportAddressInPlace(
                (*(*Qelt).ConnInfo).RemoteAddress as PTRANSPORT_ADDRESS,
                (*(*FCB).ListenIrp.ConnectionReturnInfo).RemoteAddress as PTRANSPORT_ADDRESS,
            );
            InsertTailList(
                core::ptr::addr_of_mut!((*FCB).PendingConnections),
                core::ptr::addr_of_mut!((*Qelt).ListEntry),
            );
        }
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_PREACCEPT as usize]))
        && !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingConnections))
    {
        let PendingIrp = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_PREACCEPT as usize]));
        let PendingConn = (*FCB).PendingConnections.Flink;
        SatisfyPreAccept(
            (PendingIrp as usize - core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)) as PIRP,
            (PendingConn as usize - core::mem::offset_of!(AFD_TDI_OBJECT_QELT, ListEntry)) as PAFD_TDI_OBJECT_QELT,
        );
    }

    Status = WarmSocketForConnection(FCB);

    if NT_SUCCESS(Status) {
        Status = TdiBuildNullConnectionInfoInPlace(
            (*FCB).ListenIrp.ConnectionCallInfo,
            (*(*FCB).LocalAddress).Address[0].AddressType as u32,
        );
        ASSERT!(Status == STATUS_SUCCESS);

        Status = TdiBuildNullConnectionInfoInPlace(
            (*FCB).ListenIrp.ConnectionReturnInfo,
            (*(*FCB).LocalAddress).Address[0].AddressType as u32,
        );
        ASSERT!(Status == STATUS_SUCCESS);

        Status = TdiListen(
            core::ptr::addr_of_mut!((*FCB).ListenIrp.InFlightRequest),
            (*FCB).Connection.Object,
            core::ptr::addr_of_mut!((*FCB).ListenIrp.ConnectionCallInfo),
            core::ptr::addr_of_mut!((*FCB).ListenIrp.ConnectionReturnInfo),
            Some(ListenComplete),
            FCB as PVOID,
        );

        if Status == STATUS_PENDING {
            Status = STATUS_SUCCESS;
        }
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingConnections)) {
        (*FCB).PollState |= AFD_EVENT_ACCEPT;
        (*FCB).PollStatus[FD_ACCEPT_BIT as usize] = STATUS_SUCCESS;
        PollReeval((*FCB).DeviceExt, (*FCB).FileObject);
    } else {
        (*FCB).PollState &= !AFD_EVENT_ACCEPT;
    }

    SocketStateUnlock(FCB);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn AfdListenSocket(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let ListenReq: PAFD_LISTEN_DATA;

    let _ = DeviceObject;

    AFD_DbgPrint!(MID_TRACE, b"Called on %p\n\0".as_ptr() as PCHAR, FCB);
    // Listen-path breadcrumb (once per listen socket — cheap). Paired with
    // netstack's [NETSTACK] TDI LISTEN markers it localizes any future
    // usermode-server break to msafd/AFD/netstack in one boot (the 2026-07-07
    // httpd bring-up needed exactly this).
    DbgPrint(b"[AFD] ListenSocket FCB=%p state=%u\n\0".as_ptr() as PCHAR,
             FCB, (*FCB).SharedData.State);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    ListenReq = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut()) as PAFD_LISTEN_DATA;
    if ListenReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if (*FCB).SharedData.State != SOCKET_STATE_BOUND {
        Status = STATUS_INVALID_PARAMETER;
        AFD_DbgPrint!(MIN_TRACE, b"Could not listen an unbound socket\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
    }

    (*FCB).DelayedAccept = (*ListenReq).UseDelayedAcceptance;

    AFD_DbgPrint!(MID_TRACE, b"ADDRESSFILE: %p\n\0".as_ptr() as PCHAR, (*FCB).AddressFile.Handle);

    Status = WarmSocketForConnection(FCB);

    AFD_DbgPrint!(MID_TRACE, b"Status from warmsocket %x\n\0".as_ptr() as PCHAR, Status);
    DbgPrint(b"[AFD] ListenSocket warm=0x%x connobj=%p\n\0".as_ptr() as PCHAR,
             Status, (*FCB).Connection.Object);

    if !NT_SUCCESS(Status) {
        return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
    }

    Status = TdiBuildNullConnectionInfo(
        core::ptr::addr_of_mut!((*FCB).ListenIrp.ConnectionCallInfo),
        (*(*FCB).LocalAddress).Address[0].AddressType as u32,
    );

    if !NT_SUCCESS(Status) {
        return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
    }

    Status = TdiBuildNullConnectionInfo(
        core::ptr::addr_of_mut!((*FCB).ListenIrp.ConnectionReturnInfo),
        (*(*FCB).LocalAddress).Address[0].AddressType as u32,
    );

    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(
            (*FCB).ListenIrp.ConnectionCallInfo as PVOID,
            TAG_AFD_TDI_CONNECTION_INFORMATION,
        );

        (*FCB).ListenIrp.ConnectionCallInfo = core::ptr::null_mut();
        return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
    }

    (*FCB).SharedData.State = SOCKET_STATE_LISTENING;

    Status = TdiListen(
        core::ptr::addr_of_mut!((*FCB).ListenIrp.InFlightRequest),
        (*FCB).Connection.Object,
        core::ptr::addr_of_mut!((*FCB).ListenIrp.ConnectionCallInfo),
        core::ptr::addr_of_mut!((*FCB).ListenIrp.ConnectionReturnInfo),
        Some(ListenComplete),
        FCB as PVOID,
    );

    DbgPrint(b"[AFD] ListenSocket TdiListen=0x%x\n\0".as_ptr() as PCHAR, Status);
    if Status == STATUS_PENDING {
        Status = STATUS_SUCCESS;
    }

    AFD_DbgPrint!(MID_TRACE, b"Returning %x\n\0".as_ptr() as PCHAR, Status);
    UnlockAndMaybeComplete(FCB, Status, Irp, 0)
}
#[no_mangle]
pub unsafe extern "C" fn AfdWaitForListen(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let Status: NTSTATUS;

    AFD_DbgPrint!(MID_TRACE, b"Called\n\0".as_ptr() as PCHAR);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingConnections)) {
        let PendingConn: PLIST_ENTRY = (*FCB).PendingConnections.Flink;

        Status = SatisfyPreAccept(
            Irp,
            ((PendingConn as usize) - core::mem::offset_of!(AFD_TDI_OBJECT_QELT, ListEntry)) as PAFD_TDI_OBJECT_QELT,
        );

        AFD_DbgPrint!(MID_TRACE, b"Completed a wait for accept\n\0".as_ptr() as PCHAR);

        if !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingConnections)) {
            (*FCB).PollState |= AFD_EVENT_ACCEPT;
            (*FCB).PollStatus[FD_ACCEPT_BIT as usize] = STATUS_SUCCESS;
            PollReeval((*FCB).DeviceExt, (*FCB).FileObject);
        } else {
            (*FCB).PollState &= !AFD_EVENT_ACCEPT;
        }

        SocketStateUnlock(FCB);
        return Status;
    } else if (*FCB).NonBlocking != 0 {
        AFD_DbgPrint!(MIN_TRACE, b"No connection ready on a non-blocking socket\n\0".as_ptr() as PCHAR);

        return UnlockAndMaybeComplete(FCB, STATUS_CANT_WAIT, Irp, 0);
    } else {
        AFD_DbgPrint!(MID_TRACE, b"Holding\n\0".as_ptr() as PCHAR);

        return LeaveIrpUntilLater(FCB, Irp, FUNCTION_PREACCEPT);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AfdAccept(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let DeviceExt: PAFD_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PAFD_DEVICE_EXTENSION;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let AcceptData: PAFD_ACCEPT_DATA = (*Irp).AssociatedIrp.SystemBuffer as PAFD_ACCEPT_DATA;

    AFD_DbgPrint!(MID_TRACE, b"Called\n\0".as_ptr() as PCHAR);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    (*FCB).EventSelectDisabled &= !AFD_EVENT_ACCEPT;

    let mut PendingConn = (*FCB).PendingConnections.Flink;
    while PendingConn != core::ptr::addr_of_mut!((*FCB).PendingConnections) {
        let PendingConnObj: PAFD_TDI_OBJECT_QELT =
            ((PendingConn as usize) - core::mem::offset_of!(AFD_TDI_OBJECT_QELT, ListEntry)) as PAFD_TDI_OBJECT_QELT;

        AFD_DbgPrint!(
            MID_TRACE,
            b"Comparing Seq %u to Q %u\n\0".as_ptr() as PCHAR,
            (*AcceptData).SequenceNumber,
            (*PendingConnObj).Seq,
        );

        if (*PendingConnObj).Seq == (*AcceptData).SequenceNumber {
            let mut NewFileObject: PFILE_OBJECT = core::ptr::null_mut();

            RemoveEntryList(PendingConn);

            Status = ObReferenceObjectByHandle(
                (*AcceptData).ListenHandle,
                FILE_ALL_ACCESS,
                core::ptr::null_mut(),
                0i32 as i8,
                core::ptr::addr_of_mut!(NewFileObject) as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );

            if !NT_SUCCESS(Status) {
                return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
            }

            Status = SatisfyAccept(DeviceExt, Irp, NewFileObject, PendingConnObj);

            ObDereferenceObject(NewFileObject as *mut core::ffi::c_void);

            AFD_DbgPrint!(MID_TRACE, b"Completed a wait for accept\n\0".as_ptr() as PCHAR);

            ExFreePoolWithTag(PendingConnObj as *mut core::ffi::c_void, TAG_AFD_ACCEPT_QUEUE);

            if !IsListEmpty(core::ptr::addr_of_mut!((*FCB).PendingConnections)) {
                (*FCB).PollState |= AFD_EVENT_ACCEPT;
                (*FCB).PollStatus[FD_ACCEPT_BIT as usize] = STATUS_SUCCESS;
                PollReeval((*FCB).DeviceExt, (*FCB).FileObject);
            } else {
                (*FCB).PollState &= !AFD_EVENT_ACCEPT;
            }

            SocketStateUnlock(FCB);
            return Status;
        }

        PendingConn = (*PendingConn).Flink;
    }

    AFD_DbgPrint!(MIN_TRACE, b"No connection waiting\n\0".as_ptr() as PCHAR);

    UnlockAndMaybeComplete(FCB, STATUS_UNSUCCESSFUL, Irp, 0)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/net/afd/afd/listen.c
//  * PURPOSE:          Ancillary functions driver
//  * PROGRAMMER:       Art Yerkes (ayerkes@speakeasy.net)
//  * UPDATE HISTORY:
//  * 20040708 Created
//  */
//
// #include "afd.h"
//
// static NTSTATUS SatisfyAccept( PAFD_DEVICE_EXTENSION DeviceExt,
//                                PIRP Irp,
//                                PFILE_OBJECT NewFileObject,
//                                PAFD_TDI_OBJECT_QELT Qelt ) {
//     PAFD_FCB FCB = NewFileObject->FsContext;
//     NTSTATUS Status;
//
//     UNREFERENCED_PARAMETER(DeviceExt);
//
//     if( !SocketAcquireStateLock( FCB ) )
//         return LostSocket( Irp );
//
//     /* Transfer the connection to the new socket, launch the opening read */
//     AFD_DbgPrint!(MID_TRACE,("Completing a real accept (FCB %p)\n", FCB));
//
//     FCB->Connection = Qelt->Object;
//
//     if (FCB->RemoteAddress)
//     {
//         ExFreePoolWithTag(FCB->RemoteAddress, TAG_AFD_TRANSPORT_ADDRESS);
//     }
//
//     FCB->RemoteAddress =
//         TaCopyTransportAddress( Qelt->ConnInfo->RemoteAddress );
//
//     if( !FCB->RemoteAddress )
//         Status = STATUS_NO_MEMORY;
//     else
//         Status = MakeSocketIntoConnection( FCB );
//
//     if (NT_SUCCESS(Status))
//         Status = TdiBuildConnectionInfo(&FCB->ConnectCallInfo, FCB->RemoteAddress);
//
//     if (NT_SUCCESS(Status))
//         Status = TdiBuildConnectionInfo(&FCB->ConnectReturnInfo, FCB->RemoteAddress);
//
//     return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
// }
//
// static NTSTATUS SatisfyPreAccept( PIRP Irp, PAFD_TDI_OBJECT_QELT Qelt ) {
//     PAFD_RECEIVED_ACCEPT_DATA ListenReceive =
//         (PAFD_RECEIVED_ACCEPT_DATA)Irp->AssociatedIrp.SystemBuffer;
//     PTA_IP_ADDRESS IPAddr;
//
//     ListenReceive->SequenceNumber = Qelt->Seq;
//
//     AFD_DbgPrint!(MID_TRACE,("Giving SEQ %u to userland\n", Qelt->Seq));
//     AFD_DbgPrint!(MID_TRACE,("Socket Address (K) %p (U) %p\n",
//                             &ListenReceive->Address,
//                             Qelt->ConnInfo->RemoteAddress));
//
//     TaCopyTransportAddressInPlace( &ListenReceive->Address,
//                                    Qelt->ConnInfo->RemoteAddress );
//
//     IPAddr = (PTA_IP_ADDRESS)&ListenReceive->Address;
//
//     AFD_DbgPrint!(MID_TRACE,("IPAddr->TAAddressCount %d\n",
//                             IPAddr->TAAddressCount));
//     AFD_DbgPrint!(MID_TRACE,("IPAddr->Address[0].AddressType %u\n",
//                             IPAddr->Address[0].AddressType));
//     AFD_DbgPrint!(MID_TRACE,("IPAddr->Address[0].AddressLength %u\n",
//                             IPAddr->Address[0].AddressLength));
//     AFD_DbgPrint!(MID_TRACE,("IPAddr->Address[0].Address[0].sin_port %x\n",
//                             IPAddr->Address[0].Address[0].sin_port));
//     AFD_DbgPrint!(MID_TRACE,("IPAddr->Address[0].Address[0].sin_addr %x\n",
//                             IPAddr->Address[0].Address[0].in_addr));
//
//     if( Irp->MdlAddress ) UnlockRequest( Irp, IoGetCurrentIrpStackLocation( Irp ) );
//
//     Irp->IoStatus.Information = ((PCHAR)&IPAddr[1]) - ((PCHAR)ListenReceive);
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     (void)IoSetCancelRoutine(Irp, NULL);
//     IoCompleteRequest( Irp, IO_NETWORK_INCREMENT );
//     return STATUS_SUCCESS;
// }
//
// static IO_COMPLETION_ROUTINE ListenComplete;
// static NTSTATUS NTAPI ListenComplete( PDEVICE_OBJECT DeviceObject,
//                                       PIRP Irp,
//                                       PVOID Context ) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAFD_FCB FCB = (PAFD_FCB)Context;
//     PAFD_TDI_OBJECT_QELT Qelt;
//     PLIST_ENTRY NextIrpEntry;
//     PIRP NextIrp;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if( !SocketAcquireStateLock( FCB ) )
//         return STATUS_FILE_CLOSED;
//
//     ASSERT(FCB->ListenIrp.InFlightRequest == Irp);
//     FCB->ListenIrp.InFlightRequest = NULL;
//
//     if( FCB->SharedData.State == SOCKET_STATE_CLOSED ) {
//         /* Cleanup our IRP queue because the FCB is being destroyed */
//         while( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_PREACCEPT] ) ) {
//            NextIrpEntry = RemoveHeadList(&FCB->PendingIrpList[FUNCTION_PREACCEPT]);
//            NextIrp = CONTAINING_RECORD(NextIrpEntry, IRP, Tail.Overlay.ListEntry);
//            NextIrp->IoStatus.Status = STATUS_FILE_CLOSED;
//            NextIrp->IoStatus.Information = 0;
//            if( NextIrp->MdlAddress ) UnlockRequest( NextIrp, IoGetCurrentIrpStackLocation( NextIrp ) );
//            (void)IoSetCancelRoutine(NextIrp, NULL);
//            IoCompleteRequest( NextIrp, IO_NETWORK_INCREMENT );
//         }
//
//         /* Free ConnectionReturnInfo and ConnectionCallInfo */
//         if (FCB->ListenIrp.ConnectionReturnInfo)
//         {
//             ExFreePoolWithTag(FCB->ListenIrp.ConnectionReturnInfo,
//                               TAG_AFD_TDI_CONNECTION_INFORMATION);
//
//             FCB->ListenIrp.ConnectionReturnInfo = NULL;
//         }
//
//         if (FCB->ListenIrp.ConnectionCallInfo)
//         {
//             ExFreePoolWithTag(FCB->ListenIrp.ConnectionCallInfo,
//                               TAG_AFD_TDI_CONNECTION_INFORMATION);
//
//             FCB->ListenIrp.ConnectionCallInfo = NULL;
//         }
//
//         SocketStateUnlock( FCB );
//         return STATUS_FILE_CLOSED;
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("Completing listen request.\n"));
//     AFD_DbgPrint!(MID_TRACE,("IoStatus was %x\n", Irp->IoStatus.Status));
//
//     if (Irp->IoStatus.Status != STATUS_SUCCESS)
//     {
//         SocketStateUnlock(FCB);
//         return Irp->IoStatus.Status;
//     }
//
//     Qelt = ExAllocatePoolWithTag(NonPagedPool,
//                                  sizeof(*Qelt),
//                                  TAG_AFD_ACCEPT_QUEUE);
//
//     if( !Qelt ) {
//         Status = STATUS_NO_MEMORY;
//     } else {
//         UINT AddressType =
//             FCB->LocalAddress->Address[0].AddressType;
//
//         Qelt->Object = FCB->Connection;
//         Qelt->Seq = FCB->ConnSeq++;
//         AFD_DbgPrint!(MID_TRACE,("Address Type: %u (RA %p)\n",
//                                 AddressType,
//                                 FCB->ListenIrp.
//                                 ConnectionReturnInfo->RemoteAddress));
//
//         Status = TdiBuildNullConnectionInfo( &Qelt->ConnInfo, AddressType );
//         if( NT_SUCCESS(Status) ) {
//             TaCopyTransportAddressInPlace
//                ( Qelt->ConnInfo->RemoteAddress,
//                  FCB->ListenIrp.ConnectionReturnInfo->RemoteAddress );
//             InsertTailList( &FCB->PendingConnections, &Qelt->ListEntry );
//         }
//     }
//
//     /* Satisfy a pre-accept request if one is available */
//     if( !IsListEmpty( &FCB->PendingIrpList[FUNCTION_PREACCEPT] ) &&
//         !IsListEmpty( &FCB->PendingConnections ) ) {
//         PLIST_ENTRY PendingIrp  =
//             RemoveHeadList( &FCB->PendingIrpList[FUNCTION_PREACCEPT] );
//         PLIST_ENTRY PendingConn = FCB->PendingConnections.Flink;
//         SatisfyPreAccept
//             ( CONTAINING_RECORD( PendingIrp, IRP,
//                                  Tail.Overlay.ListEntry ),
//               CONTAINING_RECORD( PendingConn, AFD_TDI_OBJECT_QELT,
//                                  ListEntry ) );
//     }
//
//     /* Launch new accept socket */
//     Status = WarmSocketForConnection( FCB );
//
//     if (NT_SUCCESS(Status))
//     {
//         Status = TdiBuildNullConnectionInfoInPlace(FCB->ListenIrp.ConnectionCallInfo,
//                                                    FCB->LocalAddress->Address[0].AddressType);
//         ASSERT(Status == STATUS_SUCCESS);
//
//         Status = TdiBuildNullConnectionInfoInPlace(FCB->ListenIrp.ConnectionReturnInfo,
//                                                    FCB->LocalAddress->Address[0].AddressType);
//         ASSERT(Status == STATUS_SUCCESS);
//
//         Status = TdiListen( &FCB->ListenIrp.InFlightRequest,
//                             FCB->Connection.Object,
//                             &FCB->ListenIrp.ConnectionCallInfo,
//                             &FCB->ListenIrp.ConnectionReturnInfo,
//                             ListenComplete,
//                             FCB );
//
//         if (Status == STATUS_PENDING)
//             Status = STATUS_SUCCESS;
//     }
//
//     /* Trigger a select return if appropriate */
//     if( !IsListEmpty( &FCB->PendingConnections ) ) {
//         FCB->PollState |= AFD_EVENT_ACCEPT;
//         FCB->PollStatus[FD_ACCEPT_BIT] = STATUS_SUCCESS;
//         PollReeval( FCB->DeviceExt, FCB->FileObject );
//     } else
//         FCB->PollState &= ~AFD_EVENT_ACCEPT;
//
//     SocketStateUnlock( FCB );
//
//     return Status;
// }
//
// NTSTATUS AfdListenSocket( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                           PIO_STACK_LOCATION IrpSp ) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PAFD_LISTEN_DATA ListenReq;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     AFD_DbgPrint!(MID_TRACE,("Called on %p\n", FCB));
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     if( !(ListenReq = LockRequest( Irp, IrpSp, FALSE, NULL )) )
//         return UnlockAndMaybeComplete( FCB, STATUS_NO_MEMORY, Irp,
//                                        0 );
//
//     if( FCB->SharedData.State != SOCKET_STATE_BOUND ) {
//         Status = STATUS_INVALID_PARAMETER;
//         AFD_DbgPrint!(MIN_TRACE,("Could not listen an unbound socket\n"));
//         return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
//     }
//
//     FCB->DelayedAccept = ListenReq->UseDelayedAcceptance;
//
//     AFD_DbgPrint!(MID_TRACE,("ADDRESSFILE: %p\n", FCB->AddressFile.Handle));
//
//     Status = WarmSocketForConnection( FCB );
//
//     AFD_DbgPrint!(MID_TRACE,("Status from warmsocket %x\n", Status));
//
//     if( !NT_SUCCESS(Status) ) return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
//
//     Status = TdiBuildNullConnectionInfo
//         ( &FCB->ListenIrp.ConnectionCallInfo,
//           FCB->LocalAddress->Address[0].AddressType );
//
//     if (!NT_SUCCESS(Status)) return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
//
//     Status = TdiBuildNullConnectionInfo
//         ( &FCB->ListenIrp.ConnectionReturnInfo,
//           FCB->LocalAddress->Address[0].AddressType );
//
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(FCB->ListenIrp.ConnectionCallInfo,
//                           TAG_AFD_TDI_CONNECTION_INFORMATION);
//
//         FCB->ListenIrp.ConnectionCallInfo = NULL;
//         return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
//     }
//
//     FCB->SharedData.State = SOCKET_STATE_LISTENING;
//
//     Status = TdiListen( &FCB->ListenIrp.InFlightRequest,
//                         FCB->Connection.Object,
//                         &FCB->ListenIrp.ConnectionCallInfo,
//                         &FCB->ListenIrp.ConnectionReturnInfo,
//                         ListenComplete,
//                         FCB );
//
//     if( Status == STATUS_PENDING )
//         Status = STATUS_SUCCESS;
//
//     AFD_DbgPrint!(MID_TRACE,("Returning %x\n", Status));
//     return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
// }
//
// NTSTATUS AfdWaitForListen( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                            PIO_STACK_LOCATION IrpSp ) {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     NTSTATUS Status;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     AFD_DbgPrint!(MID_TRACE,("Called\n"));
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     if( !IsListEmpty( &FCB->PendingConnections ) ) {
//         PLIST_ENTRY PendingConn = FCB->PendingConnections.Flink;
//
//         /* We have a pending connection ... complete this irp right away */
//         Status = SatisfyPreAccept
//             ( Irp,
//               CONTAINING_RECORD
//               ( PendingConn, AFD_TDI_OBJECT_QELT, ListEntry ) );
//
//         AFD_DbgPrint!(MID_TRACE,("Completed a wait for accept\n"));
//
//         if ( !IsListEmpty( &FCB->PendingConnections ) )
//         {
//              FCB->PollState |= AFD_EVENT_ACCEPT;
//              FCB->PollStatus[FD_ACCEPT_BIT] = STATUS_SUCCESS;
//              PollReeval( FCB->DeviceExt, FCB->FileObject );
//         } else
//              FCB->PollState &= ~AFD_EVENT_ACCEPT;
//
//         SocketStateUnlock( FCB );
//         return Status;
//     } else if (FCB->NonBlocking) {
//         AFD_DbgPrint!(MIN_TRACE,("No connection ready on a non-blocking socket\n"));
//
//         return UnlockAndMaybeComplete(FCB, STATUS_CANT_WAIT, Irp, 0);
//     } else {
//         AFD_DbgPrint!(MID_TRACE,("Holding\n"));
//
//         return LeaveIrpUntilLater( FCB, Irp, FUNCTION_PREACCEPT );
//     }
// }
//
// NTSTATUS AfdAccept( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                     PIO_STACK_LOCATION IrpSp ) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_DEVICE_EXTENSION DeviceExt =
//         (PAFD_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PAFD_ACCEPT_DATA AcceptData = Irp->AssociatedIrp.SystemBuffer;
//     PLIST_ENTRY PendingConn;
//
//     AFD_DbgPrint!(MID_TRACE,("Called\n"));
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     FCB->EventSelectDisabled &= ~AFD_EVENT_ACCEPT;
//
//     for( PendingConn = FCB->PendingConnections.Flink;
//          PendingConn != &FCB->PendingConnections;
//          PendingConn = PendingConn->Flink ) {
//         PAFD_TDI_OBJECT_QELT PendingConnObj =
//             CONTAINING_RECORD( PendingConn, AFD_TDI_OBJECT_QELT, ListEntry );
//
//         AFD_DbgPrint!(MID_TRACE,("Comparing Seq %u to Q %u\n",
//                                 AcceptData->SequenceNumber,
//                                 PendingConnObj->Seq));
//
//         if( PendingConnObj->Seq == AcceptData->SequenceNumber ) {
//             PFILE_OBJECT NewFileObject = NULL;
//
//             RemoveEntryList( PendingConn );
//
//             Status = ObReferenceObjectByHandle
//                 ( AcceptData->ListenHandle,
//                   FILE_ALL_ACCESS,
//                   NULL,
//                   KernelMode,
//                   (PVOID *)&NewFileObject,
//                   NULL );
//
//             if( !NT_SUCCESS(Status) ) return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
//
//             ASSERT(NewFileObject != FileObject);
//             ASSERT(NewFileObject->FsContext != FCB);
//
//             /* We have a pending connection ... complete this irp right away */
//             Status = SatisfyAccept( DeviceExt, Irp, NewFileObject, PendingConnObj );
//
//             ObDereferenceObject( NewFileObject );
//
//             AFD_DbgPrint!(MID_TRACE,("Completed a wait for accept\n"));
//
//             ExFreePoolWithTag(PendingConnObj, TAG_AFD_ACCEPT_QUEUE);
//
//             if( !IsListEmpty( &FCB->PendingConnections ) )
//             {
//                 FCB->PollState |= AFD_EVENT_ACCEPT;
//                 FCB->PollStatus[FD_ACCEPT_BIT] = STATUS_SUCCESS;
//                 PollReeval( FCB->DeviceExt, FCB->FileObject );
//             } else
//                 FCB->PollState &= ~AFD_EVENT_ACCEPT;
//
//             SocketStateUnlock( FCB );
//             return Status;
//         }
//     }
//
//     AFD_DbgPrint!(MIN_TRACE,("No connection waiting\n"));
//
//     return UnlockAndMaybeComplete( FCB, STATUS_UNSUCCESSFUL, Irp, 0 );
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
