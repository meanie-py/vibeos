use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ObOpenObjectByPointer, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP};
#[no_mangle]
pub unsafe extern "C" fn WarmSocketForBind(FCB: PAFD_FCB, ShareType: ULONG) -> NTSTATUS {
    let mut Status: NTSTATUS;

    AFD_DbgPrint!(
        MID_TRACE,
        b"Called (AF %u)\n\0".as_ptr() as PCHAR,
        (*(*FCB).LocalAddress).Address[0].AddressType,
    );

    if (*FCB).TdiDeviceName.Length == 0 || (*FCB).TdiDeviceName.Buffer.is_null() {
        AFD_DbgPrint!(MIN_TRACE, b"Null Device\n\0".as_ptr() as PCHAR);
        return STATUS_NO_SUCH_DEVICE;
    }
    if (*FCB).LocalAddress.is_null() {
        AFD_DbgPrint!(MIN_TRACE, b"No local address\n\0".as_ptr() as PCHAR);
        return STATUS_INVALID_PARAMETER;
    }

    Status = TdiOpenAddressFile(
        core::ptr::addr_of_mut!((*FCB).TdiDeviceName),
        (*FCB).LocalAddress,
        ShareType,
        core::ptr::addr_of_mut!((*FCB).AddressFile.Handle),
        core::ptr::addr_of_mut!((*FCB).AddressFile.Object),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if ((*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS) != 0 {
        if (*FCB).Recv.Size == 0 {
            Status = TdiQueryMaxDatagramLength(
                (*FCB).AddressFile.Object,
                core::ptr::addr_of_mut!((*FCB).Recv.Size),
            );
        }

        if NT_SUCCESS(Status) && (*FCB).Recv.Window.is_null() {
            (*FCB).Recv.Window = ExAllocatePoolWithTag(
                PagedPool,
                (*FCB).Recv.Size as usize,
                TAG_AFD_DATA_BUFFER,
            ) as PCHAR;

            if (*FCB).Recv.Window.is_null() {
                Status = STATUS_NO_MEMORY;
            }
        }

        if NT_SUCCESS(Status) && (*FCB).Recv.Content < (*FCB).Recv.Size {
            Status = TdiReceiveDatagram(
                core::ptr::addr_of_mut!((*FCB).ReceiveIrp.InFlightRequest),
                (*FCB).AddressFile.Object,
                0,
                (*FCB).Recv.Window,
                (*FCB).Recv.Size,
                (*FCB).AddressFrom,
                Some(PacketSocketRecvComplete),
                FCB as *mut core::ffi::c_void,
            );

            if Status == STATUS_PENDING {
                Status = STATUS_SUCCESS;
            }
        }
    }

    AFD_DbgPrint!(MID_TRACE, b"Returning %x\n\0".as_ptr() as PCHAR, Status);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn AfdBindSocket(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let BindReq: PAFD_BIND_DATA;
    let mut UserHandle: HANDLE = core::ptr::null_mut();

    let _ = DeviceObject;

    AFD_DbgPrint!(MID_TRACE, b"Called\n\0".as_ptr() as PCHAR);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    BindReq = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut()) as PAFD_BIND_DATA;
    if BindReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if (*FCB).SharedData.State != SOCKET_STATE_CREATED {
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
    }

    if !(*FCB).LocalAddress.is_null() {
        ExFreePoolWithTag((*FCB).LocalAddress as PVOID, TAG_AFD_TRANSPORT_ADDRESS);
    }

    (*FCB).LocalAddress = TaCopyTransportAddress(core::ptr::addr_of_mut!((*BindReq).Address));

    if !(*FCB).LocalAddress.is_null() {
        Status = TdiBuildConnectionInfo(
            core::ptr::addr_of_mut!((*FCB).AddressFrom),
            (*FCB).LocalAddress,
        );
    }

    if NT_SUCCESS(Status) {
        Status = WarmSocketForBind(FCB, (*BindReq).ShareType);
    }
    AFD_DbgPrint!(MID_TRACE, b"FCB->Flags %x\n\0".as_ptr() as PCHAR);

    if NT_SUCCESS(Status) {
        Status = ObOpenObjectByPointer(
            (*FCB).AddressFile.Object as PVOID,
            0,
            core::ptr::null_mut(),
            MAXIMUM_ALLOWED,
            // raw-dylib reads through the import thunk: the static IS the POBJECT_TYPE,
            // so pass it directly (no deref — see glue.rs). The C `*IoFileObjectType`
            // deref'd the IAT-alias, which Rust doesn't have.
            crate::glue::IoFileObjectType as ntkrnl_sys::ddk::POBJECT_TYPE,
            (*Irp).RequestorMode,
            core::ptr::addr_of_mut!(UserHandle),
        );
        if NT_SUCCESS(Status) {
            (*FCB).SharedData.State = SOCKET_STATE_BOUND;
        }
    }

    UnlockAndMaybeComplete(FCB, Status, Irp, UserHandle as ULONG_PTR as u32)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/net/afd/afd/bind.c
//  * PURPOSE:          Ancillary functions driver
//  * PROGRAMMER:       Art Yerkes (ayerkes@speakeasy.net)
//  * UPDATE HISTORY:
//  * 20040708 Created
//  */
//
// #include "afd.h"
//
// NTSTATUS WarmSocketForBind( PAFD_FCB FCB, ULONG ShareType ) {
//     NTSTATUS Status;
//
//     AFD_DbgPrint!(MID_TRACE,("Called (AF %u)\n",
//                             FCB->LocalAddress->Address[0].AddressType));
//
//     if( !FCB->TdiDeviceName.Length || !FCB->TdiDeviceName.Buffer ) {
//         AFD_DbgPrint!(MIN_TRACE,("Null Device\n"));
//         return STATUS_NO_SUCH_DEVICE;
//     }
//     if( !FCB->LocalAddress ) {
//         AFD_DbgPrint!(MIN_TRACE,("No local address\n"));
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Status = TdiOpenAddressFile(&FCB->TdiDeviceName,
//                                 FCB->LocalAddress,
//                                 ShareType,
//                                 &FCB->AddressFile.Handle,
//                                 &FCB->AddressFile.Object );
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS)
//     {
//         if (!FCB->Recv.Size)
//         {
//             Status = TdiQueryMaxDatagramLength(FCB->AddressFile.Object,
//                                                &FCB->Recv.Size);
//         }
//
//         if (NT_SUCCESS(Status) && !FCB->Recv.Window)
//         {
//             FCB->Recv.Window = ExAllocatePoolWithTag(PagedPool,
//                                                      FCB->Recv.Size,
//                                                      TAG_AFD_DATA_BUFFER);
//
//             if (!FCB->Recv.Window)
//                 Status = STATUS_NO_MEMORY;
//         }
//
//         if (NT_SUCCESS(Status) && FCB->Recv.Content < FCB->Recv.Size)
//         {
//             Status = TdiReceiveDatagram(&FCB->ReceiveIrp.InFlightRequest,
//                                         FCB->AddressFile.Object,
//                                         0,
//                                         FCB->Recv.Window,
//                                         FCB->Recv.Size,
//                                         FCB->AddressFrom,
//                                         PacketSocketRecvComplete,
//                                         FCB);
//
//             /* We don't want to wait for this read to complete. */
//             if( Status == STATUS_PENDING ) Status = STATUS_SUCCESS;
//         }
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("Returning %x\n", Status));
//
//     return Status;
// }
//
// NTSTATUS NTAPI
// AfdBindSocket(PDEVICE_OBJECT DeviceObject, PIRP Irp,
//               PIO_STACK_LOCATION IrpSp) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PAFD_BIND_DATA BindReq;
//     HANDLE UserHandle = NULL;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     AFD_DbgPrint!(MID_TRACE,("Called\n"));
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//     if( !(BindReq = LockRequest( Irp, IrpSp, FALSE, NULL )) )
//         return UnlockAndMaybeComplete( FCB, STATUS_NO_MEMORY,
//                                        Irp, 0 );
//
//     if (FCB->LocalAddress)
//     {
//         ExFreePoolWithTag(FCB->LocalAddress, TAG_AFD_TRANSPORT_ADDRESS);
//     }
//
//     FCB->LocalAddress = TaCopyTransportAddress( &BindReq->Address );
//
//     if( FCB->LocalAddress )
//         Status = TdiBuildConnectionInfo( &FCB->AddressFrom,
//                                          FCB->LocalAddress );
//
//     if( NT_SUCCESS(Status) )
//         Status = WarmSocketForBind( FCB, BindReq->ShareType );
//     AFD_DbgPrint!(MID_TRACE,("FCB->Flags %x\n", FCB->Flags));
//
//     if (NT_SUCCESS(Status))
//     {
//         Status = ObOpenObjectByPointer(FCB->AddressFile.Object,
//                                        0,
//                                        NULL,
//                                        MAXIMUM_ALLOWED,
//                                        *IoFileObjectType,
//                                        Irp->RequestorMode,
//                                        &UserHandle);
//         if (NT_SUCCESS(Status))
//             FCB->SharedData.State = SOCKET_STATE_BOUND;
//     }
//
//     /* MSAFD relies on us returning the address file handle in the IOSB */
//     return UnlockAndMaybeComplete( FCB, Status, Irp,
//                                    (ULONG_PTR)UserHandle);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================