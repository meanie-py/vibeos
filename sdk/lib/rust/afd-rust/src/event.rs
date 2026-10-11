//! ORPHAN-BY-DESIGN: `event.c` is not in afd's build — it is dead upstream
//! (its `AfdRegisterEventHandlers` is called from nowhere). Translated and kept
//! per the never-delete rule; see drivers/network/afd/CMakeLists.txt.
//!
#[allow(unused_imports)] use crate::{AFD_DbgPrint};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{PIRP, PMDL};
use core::ffi::c_void;

extern "C" {
    fn TdiSetEventHandler( Handle: PVOID, EventType: ULONG, EventHandler: PVOID, EventContext: PVOID, ) -> NTSTATUS;
}
#[no_mangle]
pub unsafe extern "C" fn AfdEventError(
    TdiEventContext: PVOID,
    Status: NTSTATUS,
) -> NTSTATUS {

    AFD_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCHAR);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn AfdEventDisconnect(
    TdiEventContext: PVOID,
    ConnectionContext: CONNECTION_CONTEXT,
    DisconnectDataLength: LONG,
    DisconnectData: PVOID,
    DisconnectInformationLength: LONG,
    DisconnectInformation: PVOID,
    DisconnectFlags: ULONG,
) -> NTSTATUS {

    AFD_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCHAR);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn AfdEventReceive(
    TdiEventContext: PVOID,
    ConnectionContext: CONNECTION_CONTEXT,
    ReceiveFlags: ULONG,
    BytesIndicated: ULONG,
    BytesAvailable: ULONG,
    BytesTaken: *mut ULONG,
    Tsdu: PVOID,
    IoRequestPacket: *mut PIRP,
) -> NTSTATUS {
    let FCB = TdiEventContext as PAFDFCB;
    let mut ReceiveBuffer: PVOID;
    let mut Buffer: PAFD_BUFFER;
    let mut OldIrql: KIRQL = 0;

    AFD_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCHAR);

    AFD_DbgPrint!(MID_TRACE, b"Receiving (%d) bytes on socket\n\0".as_ptr() as PCHAR, BytesAvailable);

    ReceiveBuffer = ExAllocatePoolWithTag(
        NonPagedPool,
        BytesAvailable as usize,
        TAG_AFD_DATA_BUFFER,
    );

    if ReceiveBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<AFD_BUFFER>(),
        TAG_AFD_DATA_BUFFER,
    ) as PAFD_BUFFER;

    if Buffer.is_null() {
        ExFreePoolWithTag(ReceiveBuffer, TAG_AFD_DATA_BUFFER);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(
        Tsdu as *const u8,
        ReceiveBuffer as *mut u8,
        BytesAvailable as usize,
    );

    (*Buffer).Buffer.len = BytesAvailable;
    (*Buffer).Buffer.buf = ReceiveBuffer as PCHAR;
    (*Buffer).Offset = 0;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*FCB).ReceiveQueueLock), core::ptr::addr_of_mut!(OldIrql));

    InsertTailList(
        core::ptr::addr_of_mut!((*FCB).ReceiveQueue),
        core::ptr::addr_of_mut!((*Buffer).ListEntry),
    );

    TryToSatisfyRecvRequest(FCB, 1u8);

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*FCB).ReceiveQueueLock), OldIrql);

    *BytesTaken = BytesAvailable;

    AFD_DbgPrint!(MAX_TRACE, b"Leaving.\n\0".as_ptr() as PCHAR);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClientEventReceiveExpedited(
    TdiEventContext: PVOID,
    ConnectionContext: CONNECTION_CONTEXT,
    ReceiveFlags: ULONG,
    BytesIndicated: ULONG,
    BytesAvailable: ULONG,
    BytesTaken: *mut ULONG,
    Tsdu: PVOID,
    IoRequestPacket: *mut PIRP,
) -> TDI_STATUS {
    AFD_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCHAR);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClientEventChainedReceive(
    TdiEventContext: PVOID,
    ConnectionContext: CONNECTION_CONTEXT,
    ReceiveFlags: ULONG,
    ReceiveLength: ULONG,
    StartingOffset: ULONG,
    Tsdu: PMDL,
    TsduDescriptor: PVOID,
) -> NTSTATUS {
    AFD_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCHAR);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn AfdEventReceiveDatagramHandler(
    TdiEventContext: PVOID,
    SourceAddressLength: LONG,
    SourceAddress: PVOID,
    OptionsLength: LONG,
    Options: PVOID,
    ReceiveDatagramFlags: ULONG,
    BytesIndicated: ULONG,
    BytesAvailable: ULONG,
    BytesTaken: *mut ULONG,
    Tsdu: PVOID,
    IoRequestPacket: *mut PIRP,
) -> NTSTATUS {
    let FCB = TdiEventContext as PAFDFCB;
    let mut ReadRequest: PAFD_READ_REQUEST;
    let ReceiveBuffer: PVOID;
    let Buffer: PAFD_BUFFER;
    let mut Entry: PLIST_ENTRY;
    let mut Status: NTSTATUS;
    let mut OldIrql: KIRQL = 0;
    let mut Count: ULONG = 0;
    let CompleteIrp: BOOLEAN;

    AFD_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCHAR);

    AFD_DbgPrint!(MID_TRACE, b"Receiving (%d) bytes from (0x%X).\n\0".as_ptr() as PCHAR, BytesAvailable, *(SourceAddress as PULONG));

    ReceiveBuffer = ExAllocatePoolWithTag(
        NonPagedPool,
        BytesAvailable as SIZE_T,
        TAG_AFD_DATA_BUFFER,
    );

    if ReceiveBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<AFD_BUFFER>() as SIZE_T,
        TAG_AFD_DATA_BUFFER,
    ) as PAFD_BUFFER;

    if Buffer.is_null() {
        ExFreePoolWithTag(ReceiveBuffer, TAG_AFD_DATA_BUFFER);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyMemory(ReceiveBuffer, Tsdu, BytesAvailable as SIZE_T);

    (*Buffer).Buffer.len = BytesAvailable;
    (*Buffer).Buffer.buf = ReceiveBuffer as PCHAR;
    (*Buffer).Offset = 0;

    ExInterlockedInsertTailList(
        core::ptr::addr_of_mut!((*FCB).ReceiveQueue),
        core::ptr::addr_of_mut!((*Buffer).ListEntry),
        core::ptr::addr_of_mut!((*FCB).ReceiveQueueLock),
    );

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*FCB).ReceiveQueueLock),
        core::ptr::addr_of_mut!(OldIrql),
    );

    CompleteIrp = if IsListEmpty(core::ptr::addr_of_mut!((*FCB).ReadRequestQueue)) == 0 {
        AFD_DbgPrint!(MAX_TRACE, b"Satisfying read request.\n\0".as_ptr() as PCHAR);

        Entry = RemoveHeadList(core::ptr::addr_of_mut!((*FCB).ReceiveQueue));
        ReadRequest = (Entry as *mut u8).offset(-(core::mem::offset_of!(AFD_READ_REQUEST, ListEntry) as isize)) as PAFD_READ_REQUEST;

        Status = FillWSABuffers(
            FCB,
            (*(*ReadRequest).RecvFromRequest).Buffers,
            (*(*ReadRequest).RecvFromRequest).BufferCount,
            core::ptr::addr_of_mut!(Count),
            0u8,
        );
        (*(*ReadRequest).RecvFromReply).NumberOfBytesRecvd = Count;
        (*(*ReadRequest).RecvFromReply).__bindgen_anon_1.Status = NO_ERROR;

        (*(*ReadRequest).Irp).IoStatus.Information = 0;
        (*(*ReadRequest).Irp).IoStatus.__bindgen_anon_1.Status = Status;

        1u8
    } else {
        0u8
    };

    KeReleaseSpinLock(core::ptr::addr_of_mut!((*FCB).ReceiveQueueLock), OldIrql);

    if CompleteIrp != 0 {
        AFD_DbgPrint!(MAX_TRACE, b"Completing IRP at (0x%X).\n\0".as_ptr() as PCHAR, (*ReadRequest).Irp);
        IoCompleteRequest((*ReadRequest).Irp, IO_NETWORK_INCREMENT as CCHAR);
        *BytesTaken = BytesAvailable;
    }

    AFD_DbgPrint!(MAX_TRACE, b"Leaving.\n\0".as_ptr() as PCHAR);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn AfdRegisterEventHandlers(FCB: PAFDFCB) -> NTSTATUS {

    let mut Status: NTSTATUS;

    Status = TdiSetEventHandler(
        (*FCB).TdiAddressObject,
        TDI_EVENT_ERROR,
        AfdEventError as PVOID,
        FCB as PVOID,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    match (*FCB).SocketType {
        SOCK_STREAM => {
            Status = TdiSetEventHandler(
                (*FCB).TdiAddressObject,
                TDI_EVENT_DISCONNECT,
                AfdEventDisconnect as PVOID,
                FCB as PVOID,
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }

            Status = TdiSetEventHandler(
                (*FCB).TdiAddressObject,
                TDI_EVENT_RECEIVE,
                AfdEventReceive as PVOID,
                FCB as PVOID,
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }

            Status = TdiSetEventHandler(
                (*FCB).TdiAddressObject,
                TDI_EVENT_RECEIVE_EXPEDITED,
                ClientEventReceiveExpedited as PVOID,
                FCB as PVOID,
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }

            Status = TdiSetEventHandler(
                (*FCB).TdiAddressObject,
                TDI_EVENT_CHAINED_RECEIVE,
                ClientEventChainedReceive as PVOID,
                FCB as PVOID,
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }
        }

        SOCK_DGRAM | SOCK_RAW => {
            Status = TdiSetEventHandler(
                (*FCB).TdiAddressObject,
                TDI_EVENT_RECEIVE_DATAGRAM,
                AfdEventReceiveDatagramHandler as PVOID,
                FCB as PVOID,
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }
        }

        _ => {}
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn AfdDeregisterEventHandlers(FCB: PAFDFCB) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = TdiSetEventHandler(
        (*FCB).TdiAddressObject,
        TDI_EVENT_ERROR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    match (*FCB).SocketType {
        SOCK_STREAM => {
            Status = TdiSetEventHandler(
                (*FCB).TdiAddressObject,
                TDI_EVENT_DISCONNECT,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }

            Status = TdiSetEventHandler(
                (*FCB).TdiAddressObject,
                TDI_EVENT_RECEIVE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }

            Status = TdiSetEventHandler(
                (*FCB).TdiAddressObject,
                TDI_EVENT_RECEIVE_EXPEDITED,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }

            Status = TdiSetEventHandler(
                (*FCB).TdiAddressObject,
                TDI_EVENT_CHAINED_RECEIVE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }
        }

        SOCK_DGRAM | SOCK_RAW => {
            Status = TdiSetEventHandler(
                (*FCB).TdiAddressObject,
                TDI_EVENT_RECEIVE_DATAGRAM,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }
        }

        _ => {}
    }

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Ancillary Function Driver
//  * FILE:        afd/event.c
//  * PURPOSE:     TDI event handlers
//  * PROGRAMMERS: Casper S. Hornstrup (chorns@users.sourceforge.net)
//  * REVISIONS:
//  *   CSH 01/09-2000 Created
//  */
//
// #include <afd.h>
//
// NTSTATUS AfdEventError(
//     IN PVOID TdiEventContext,
//     IN NTSTATUS Status)
// {
//     AFD_DbgPrint(MAX_TRACE, ("Called.\n"));
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS AfdEventDisconnect(
//     IN PVOID TdiEventContext,
//     IN CONNECTION_CONTEXT ConnectionContext,
//     IN LONG DisconnectDataLength,
//     IN PVOID DisconnectData,
//     IN LONG DisconnectInformationLength,
//     IN PVOID DisconnectInformation,
//     IN ULONG DisconnectFlags)
// {
//     AFD_DbgPrint(MAX_TRACE, ("Called.\n"));
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS AfdEventReceive(
//     IN PVOID TdiEventContext,
//     IN CONNECTION_CONTEXT ConnectionContext,
//     IN ULONG ReceiveFlags,
//     IN ULONG BytesIndicated,
//     IN ULONG BytesAvailable,
//     OUT ULONG *BytesTaken,
//     IN PVOID Tsdu,
//     OUT PIRP *IoRequestPacket)
// {
//     PAFDFCB FCB = (PAFDFCB)TdiEventContext;
//     PVOID ReceiveBuffer;
//     PAFD_BUFFER Buffer;
//     KIRQL OldIrql;
//
//     AFD_DbgPrint(MAX_TRACE, ("Called.\n"));
//
//     AFD_DbgPrint(MID_TRACE, ("Receiving (%d) bytes on socket\n",
//                              BytesAvailable));
//
//     ReceiveBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                           BytesAvailable,
//                                           TAG_AFD_DATA_BUFFER);
//
//     if (!ReceiveBuffer)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /*Buffer = (PAFD_BUFFER)ExAllocateFromNPagedLookasideList(
//       &BufferLookasideList);*/
//     Buffer = (PAFD_BUFFER)ExAllocatePoolWithTag(NonPagedPool,
//                                                 sizeof(AFD_BUFFER),
//                                                 TAG_AFD_DATA_BUFFER);
//
//     if (!Buffer) {
//         ExFreePoolWithTag(ReceiveBuffer, TAG_AFD_DATA_BUFFER);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Copy the data to a local buffer */
//     RtlCopyMemory(ReceiveBuffer, Tsdu, BytesAvailable);
//
//     Buffer->Buffer.len = BytesAvailable;
//     Buffer->Buffer.buf = ReceiveBuffer;
//     Buffer->Offset = 0;
//
//     KeAcquireSpinLock(&FCB->ReceiveQueueLock, &OldIrql);
//
//     InsertTailList( &FCB->ReceiveQueue, &Buffer->ListEntry );
//
//     TryToSatisfyRecvRequest( FCB, TRUE );
//
//     KeReleaseSpinLock(&FCB->ReceiveQueueLock, OldIrql);
//
//     *BytesTaken = BytesAvailable;
//
//     AFD_DbgPrint(MAX_TRACE, ("Leaving.\n"));
//
//     return STATUS_SUCCESS;
// }
//
//
// TDI_STATUS ClientEventReceiveExpedited(
//     IN PVOID TdiEventContext,
//     IN CONNECTION_CONTEXT ConnectionContext,
//     IN ULONG ReceiveFlags,
//     IN ULONG BytesIndicated,
//     IN ULONG BytesAvailable,
//     OUT ULONG *BytesTaken,
//     IN PVOID Tsdu,
//     OUT PIRP *IoRequestPacket)
// {
//     AFD_DbgPrint(MAX_TRACE, ("Called.\n"));
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS ClientEventChainedReceive(
//     IN PVOID TdiEventContext,
//     IN CONNECTION_CONTEXT ConnectionContext,
//     IN ULONG ReceiveFlags,
//     IN ULONG ReceiveLength,
//     IN ULONG StartingOffset,
//     IN PMDL Tsdu,
//     IN PVOID TsduDescriptor)
// {
//     AFD_DbgPrint(MAX_TRACE, ("Called.\n"));
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS AfdEventReceiveDatagramHandler(
//     IN PVOID TdiEventContext,
//     IN LONG SourceAddressLength,
//     IN PVOID SourceAddress,
//     IN LONG OptionsLength,
//     IN PVOID Options,
//     IN ULONG ReceiveDatagramFlags,
//     IN ULONG BytesIndicated,
//     IN ULONG BytesAvailable,
//     OUT ULONG *BytesTaken,
//     IN PVOID Tsdu,
//     OUT PIRP *IoRequestPacket)
// {
//     PAFDFCB FCB = (PAFDFCB)TdiEventContext;
//     PAFD_READ_REQUEST ReadRequest;
//     PVOID ReceiveBuffer;
//     PAFD_BUFFER Buffer;
//     PLIST_ENTRY Entry;
//     NTSTATUS Status;
//     KIRQL OldIrql;
//     ULONG Count;
//     BOOLEAN CompleteIrp;
//
//     AFD_DbgPrint(MAX_TRACE, ("Called.\n"));
//
//     AFD_DbgPrint(MID_TRACE, ("Receiving (%d) bytes from (0x%X).\n",
//                              BytesAvailable, *(PULONG)SourceAddress));
//
//     ReceiveBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                           BytesAvailable,
//                                           TAG_AFD_DATA_BUFFER);
//
//     if (!ReceiveBuffer)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /*Buffer = (PAFD_BUFFER)ExAllocateFromNPagedLookasideList(
//       &BufferLookasideList);*/
//     Buffer = (PAFD_BUFFER)ExAllocatePoolWithTag(NonPagedPool,
//                                                 sizeof(AFD_BUFFER),
//                                                 TAG_AFD_DATA_BUFFER);
//
//     if (!Buffer) {
//         ExFreePoolWithTag(ReceiveBuffer, TAG_AFD_DATA_BUFFER);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Copy the data to a local buffer */
//     RtlCopyMemory(ReceiveBuffer, Tsdu, BytesAvailable);
//
//     Buffer->Buffer.len = BytesAvailable;
//     Buffer->Buffer.buf = ReceiveBuffer;
//     Buffer->Offset = 0;
//
//     ExInterlockedInsertTailList(&FCB->ReceiveQueue,
//                                 &Buffer->ListEntry,
//                                 &FCB->ReceiveQueueLock);
//
//     KeAcquireSpinLock(&FCB->ReceiveQueueLock, &OldIrql);
//
//     if (CompleteIrp = !IsListEmpty(&FCB->ReadRequestQueue)) {
//         AFD_DbgPrint(MAX_TRACE, ("Satisfying read request.\n"));
//
//         Entry = RemoveHeadList(&FCB->ReceiveQueue);
//         ReadRequest = CONTAINING_RECORD(Entry, AFD_READ_REQUEST, ListEntry);
//
//         Status = FillWSABuffers(FCB,
//                                 ReadRequest->RecvFromRequest->Buffers,
//                                 ReadRequest->RecvFromRequest->BufferCount,
//                                 &Count,
//                                 FALSE); /* Packet oriented */
//         ReadRequest->RecvFromReply->NumberOfBytesRecvd = Count;
//         ReadRequest->RecvFromReply->Status = NO_ERROR;
//
//         ReadRequest->Irp->IoStatus.Information = 0;
//         ReadRequest->Irp->IoStatus.Status = Status;
//     }
//
//     KeReleaseSpinLock(&FCB->ReceiveQueueLock, OldIrql);
//
//     if (CompleteIrp) {
//         AFD_DbgPrint(MAX_TRACE, ("Completing IRP at (0x%X).\n", ReadRequest->Irp));
//         IoCompleteRequest(ReadRequest->Irp, IO_NETWORK_INCREMENT);
//         *BytesTaken = BytesAvailable;
//     }
//
//     AFD_DbgPrint(MAX_TRACE, ("Leaving.\n"));
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS AfdRegisterEventHandlers(
//     PAFDFCB FCB)
// {
//     NTSTATUS Status;
//
//     assert(FCB->TdiAddressObject);
//
//     /* Report errors for all types of sockets */
//     Status = TdiSetEventHandler(FCB->TdiAddressObject,
//         TDI_EVENT_ERROR,
//         (PVOID)AfdEventError,
//         (PVOID)FCB);
//     if (!NT_SUCCESS(Status)) { return Status; }
//
//     switch (FCB->SocketType) {
//     case SOCK_STREAM:
//         Status = TdiSetEventHandler(FCB->TdiAddressObject,
//             TDI_EVENT_DISCONNECT,
//             (PVOID)AfdEventDisconnect,
//             (PVOID)FCB);
//         if (!NT_SUCCESS(Status)) { return Status; }
//
//         Status = TdiSetEventHandler(FCB->TdiAddressObject,
//             TDI_EVENT_RECEIVE,
//             (PVOID)AfdEventReceive,
//             (PVOID)FCB);
//         if (!NT_SUCCESS(Status)) { return Status; }
//
//         Status = TdiSetEventHandler(FCB->TdiAddressObject,
//             TDI_EVENT_RECEIVE_EXPEDITED,
//             (PVOID)ClientEventReceiveExpedited,
//             (PVOID)FCB);
//         if (!NT_SUCCESS(Status)) { return Status; }
//
//         Status = TdiSetEventHandler(FCB->TdiAddressObject,
//             TDI_EVENT_CHAINED_RECEIVE,
//             (PVOID)ClientEventChainedReceive,
//             (PVOID)FCB);
//         if (!NT_SUCCESS(Status)) { return Status; }
//         break;
//
//     case SOCK_DGRAM:
//     case SOCK_RAW:
//         Status = TdiSetEventHandler(FCB->TdiAddressObject,
//             TDI_EVENT_RECEIVE_DATAGRAM,
//             (PVOID)AfdEventReceiveDatagramHandler,
//             (PVOID)FCB);
//         if (!NT_SUCCESS(Status)) { return Status; }
//     }
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS AfdDeregisterEventHandlers(
//     PAFDFCB FCB)
// {
//     NTSTATUS Status;
//
//     Status = TdiSetEventHandler(FCB->TdiAddressObject,
//         TDI_EVENT_ERROR,
//         NULL,
//         NULL);
//     if (!NT_SUCCESS(Status)) { return Status; }
//
//     switch (FCB->SocketType) {
//     case SOCK_STREAM:
//         Status = TdiSetEventHandler(FCB->TdiAddressObject,
//             TDI_EVENT_DISCONNECT,
//             NULL,
//             NULL);
//         if (!NT_SUCCESS(Status)) { return Status; }
//
//         Status = TdiSetEventHandler(FCB->TdiAddressObject,
//             TDI_EVENT_RECEIVE,
//             NULL,
//             NULL);
//         if (!NT_SUCCESS(Status)) { return Status; }
//
//         Status = TdiSetEventHandler(FCB->TdiAddressObject,
//             TDI_EVENT_RECEIVE_EXPEDITED,
//             NULL,
//             NULL);
//         if (!NT_SUCCESS(Status)) { return Status; }
//
//         Status = TdiSetEventHandler(FCB->TdiAddressObject,
//             TDI_EVENT_CHAINED_RECEIVE,
//             NULL,
//             NULL);
//         if (!NT_SUCCESS(Status)) { return Status; }
//         break;
//
//     case SOCK_DGRAM:
//     case SOCK_RAW:
//         Status = TdiSetEventHandler(FCB->TdiAddressObject,
//             TDI_EVENT_RECEIVE_DATAGRAM,
//             NULL,
//             NULL);
//         if (!NT_SUCCESS(Status)) { return Status; }
//     }
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
