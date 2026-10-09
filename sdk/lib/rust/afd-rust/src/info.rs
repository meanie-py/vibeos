use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoAllocateMdl, IoFreeMdl, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PMDL};

#[no_mangle]
pub unsafe extern "system" fn AfdGetInfo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let InfoReq: PAFD_INFO = LockRequest(Irp, IrpSp, 1u8, core::ptr::null_mut()) as PAFD_INFO;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let mut CurrentEntry: PLIST_ENTRY;

    let _ = DeviceObject;

    AFD_DbgPrint!(
        MID_TRACE,
        b"Called %p %x\n\0".as_ptr() as PCHAR,
        InfoReq,
        if !InfoReq.is_null() {
            (*InfoReq).InformationClass
        } else {
            0
        },
    );

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if InfoReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    {
        match (*InfoReq).InformationClass {
            AFD_INFO_RECEIVE_WINDOW_SIZE => {
                *(*InfoReq).Information.Ulong.as_mut() = (*FCB).Recv.Size;
            }

            AFD_INFO_SEND_WINDOW_SIZE => {
                *(*InfoReq).Information.Ulong.as_mut() = (*FCB).Send.Size;
                AFD_DbgPrint!(
                    MID_TRACE,
                    b"Send window size %u\n\0".as_ptr() as PCHAR,
                    (*FCB).Send.Size,
                );
            }

            AFD_INFO_GROUP_ID_TYPE => {
                (*InfoReq).Information.LargeInteger.as_mut().u.HighPart = (*FCB).GroupType as LONG;
                (*InfoReq).Information.LargeInteger.as_mut().u.LowPart = (*FCB).GroupID;
                AFD_DbgPrint!(
                    MID_TRACE,
                    b"Group ID: %u Group Type: %u\n\0".as_ptr() as PCHAR,
                    (*FCB).GroupID,
                    (*FCB).GroupType,
                );
            }

            AFD_INFO_BLOCKING_MODE => {
                *(*InfoReq).Information.Boolean.as_mut() = (*FCB).NonBlocking;
            }

            AFD_INFO_INLINING_MODE => {
                *(*InfoReq).Information.Boolean.as_mut() = (*FCB).OobInline;
            }

            AFD_INFO_RECEIVE_CONTENT_SIZE => {
                *(*InfoReq).Information.Ulong.as_mut() = (*FCB).Recv.Content - (*FCB).Recv.BytesUsed;
            }

            AFD_INFO_SENDS_IN_PROGRESS => {
                *(*InfoReq).Information.Ulong.as_mut() = 0;

                CurrentEntry = (*FCB).PendingIrpList[FUNCTION_SEND as usize].Flink;
                while CurrentEntry != core::ptr::addr_of_mut!((*FCB).PendingIrpList[FUNCTION_SEND as usize]) {
                    *(*InfoReq).Information.Ulong.as_mut() += 1;
                    CurrentEntry = (*CurrentEntry).Flink;
                }

                if !(*FCB).SendIrp.InFlightRequest.is_null() {
                    *(*InfoReq).Information.Ulong.as_mut() += 1;
                }
            }

            _ => {
                AFD_DbgPrint!(
                    MIN_TRACE,
                    b"Unknown info id %x\n\0".as_ptr() as PCHAR,
                    (*InfoReq).InformationClass,
                );
                Status = STATUS_INVALID_PARAMETER;
            }
        }
    }

    AFD_DbgPrint!(MID_TRACE, b"Returning %x\n\0".as_ptr() as PCHAR, Status);

    UnlockAndMaybeComplete(FCB, Status, Irp, 0)
}

#[no_mangle]
pub unsafe extern "system" fn AfdSetInfo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let InfoReq: PAFD_INFO = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut()) as PAFD_INFO;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let NewBuffer: PCHAR;

    let _ = DeviceObject;

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if InfoReq.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    match (*InfoReq).InformationClass {
        AFD_INFO_BLOCKING_MODE => {
            AFD_DbgPrint!(
                MID_TRACE,
                b"Blocking mode set to %u\n\0".as_ptr() as PCHAR,
                *(*InfoReq).Information.Boolean.as_ref(),
            );
            (*FCB).NonBlocking = *(*InfoReq).Information.Boolean.as_ref();
        }
        AFD_INFO_INLINING_MODE => {
            (*FCB).OobInline = *(*InfoReq).Information.Boolean.as_ref();
        }
        AFD_INFO_RECEIVE_WINDOW_SIZE => {
            if (*FCB).SharedData.State == SOCKET_STATE_CONNECTED
                || ((*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS) != 0
            {
                if *(*InfoReq).Information.Ulong.as_ref() > 0
                    && *(*InfoReq).Information.Ulong.as_ref() < 0xFFFF
                    && *(*InfoReq).Information.Ulong.as_ref() != (*FCB).Recv.Size
                {
                    NewBuffer = ExAllocatePoolWithTag(
                        PagedPool,
                        *(*InfoReq).Information.Ulong.as_ref() as usize,
                        TAG_AFD_DATA_BUFFER,
                    ) as PCHAR;

                    if !NewBuffer.is_null() {
                        if (*FCB).Recv.Content > *(*InfoReq).Information.Ulong.as_ref() {
                            (*FCB).Recv.Content = *(*InfoReq).Information.Ulong.as_ref();
                        }

                        if !(*FCB).Recv.Window.is_null() {
                            RtlCopyMemory(
                                NewBuffer as *mut core::ffi::c_void,
                                (*FCB).Recv.Window as *const core::ffi::c_void,
                                (*FCB).Recv.Content as usize,
                            );

                            ExFreePoolWithTag((*FCB).Recv.Window as *mut core::ffi::c_void, TAG_AFD_DATA_BUFFER);
                        }

                        (*FCB).Recv.Size = *(*InfoReq).Information.Ulong.as_ref();
                        (*FCB).Recv.Window = NewBuffer;

                        Status = STATUS_SUCCESS;
                    } else {
                        Status = STATUS_NO_MEMORY;
                    }
                } else {
                    Status = STATUS_SUCCESS;
                }
            } else {
                Status = STATUS_INVALID_PARAMETER;
            }
        }
        AFD_INFO_SEND_WINDOW_SIZE => {
            if (*FCB).SharedData.State == SOCKET_STATE_CONNECTED
                || ((*FCB).Flags & AFD_ENDPOINT_CONNECTIONLESS) != 0
            {
                if *(*InfoReq).Information.Ulong.as_ref() > 0
                    && *(*InfoReq).Information.Ulong.as_ref() < 0xFFFF
                    && *(*InfoReq).Information.Ulong.as_ref() != (*FCB).Send.Size
                {
                    NewBuffer = ExAllocatePoolWithTag(
                        PagedPool,
                        *(*InfoReq).Information.Ulong.as_ref() as usize,
                        TAG_AFD_DATA_BUFFER,
                    ) as PCHAR;

                    if !NewBuffer.is_null() {
                        if (*FCB).Send.BytesUsed > *(*InfoReq).Information.Ulong.as_ref() {
                            (*FCB).Send.BytesUsed = *(*InfoReq).Information.Ulong.as_ref();
                        }

                        if !(*FCB).Send.Window.is_null() {
                            RtlCopyMemory(
                                NewBuffer as *mut core::ffi::c_void,
                                (*FCB).Send.Window as *const core::ffi::c_void,
                                (*FCB).Send.BytesUsed as usize,
                            );

                            ExFreePoolWithTag((*FCB).Send.Window as *mut core::ffi::c_void, TAG_AFD_DATA_BUFFER);
                        }

                        (*FCB).Send.Size = *(*InfoReq).Information.Ulong.as_ref();
                        (*FCB).Send.Window = NewBuffer;

                        Status = STATUS_SUCCESS;
                    } else {
                        Status = STATUS_NO_MEMORY;
                    }
                } else {
                    Status = STATUS_SUCCESS;
                }
            } else {
                Status = STATUS_INVALID_PARAMETER;
            }
        }
        _ => {
            AFD_DbgPrint!(
                MIN_TRACE,
                b"Unknown request %u\n\0".as_ptr() as PCHAR,
                (*InfoReq).InformationClass,
            );
        }
    }

    AFD_DbgPrint!(MID_TRACE, b"Returning %x\n\0".as_ptr() as PCHAR, Status);

    UnlockAndMaybeComplete(FCB, Status, Irp, 0)
}

#[no_mangle]
pub unsafe extern "system" fn AfdGetSockName(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let Mdl: PMDL;

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if (*FCB).AddressFile.Object == core::ptr::null_mut() && (*FCB).Connection.Object == core::ptr::null_mut() {
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
    }

    Mdl = IoAllocateMdl(
        (*Irp).UserBuffer,
        (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength,
        0u8,
        0u8,
        core::ptr::null_mut(),
    );

    if Mdl != core::ptr::null_mut() {
        // C: _SEH2_TRY { MmProbeAndLockPages } _SEH2_EXCEPT { Status = code; }
        Status = ntkrnl_sys::probe::MmTryProbeAndLockPages(Mdl, (*Irp).RequestorMode, 2 /* IoModifyAccess */);

        if NT_SUCCESS(Status) {
            Status = TdiQueryInformation(
                if (*FCB).Connection.Object != core::ptr::null_mut() {
                    (*FCB).Connection.Object
                } else {
                    (*FCB).AddressFile.Object
                },
                TDI_QUERY_ADDRESS_INFO as LONG,
                Mdl,
            );
        }

        if !NT_SUCCESS(Status) && (*Irp).MdlAddress != Mdl {
            IoFreeMdl(Mdl);
        }
    } else {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    UnlockAndMaybeComplete(FCB, Status, Irp, 0)
}

#[no_mangle]
pub unsafe extern "system" fn AfdGetPeerName(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if (*FCB).RemoteAddress == core::ptr::null_mut() {
        AFD_DbgPrint!(MIN_TRACE, b"Invalid parameter\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_INVALID_PARAMETER, Irp, 0);
    }

    let Status: NTSTATUS;
    if (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength >= TaLengthOfTransportAddress((*FCB).RemoteAddress) {
        RtlCopyMemory(
            (*Irp).UserBuffer,
            (*FCB).RemoteAddress as *mut core::ffi::c_void,
            TaLengthOfTransportAddress((*FCB).RemoteAddress) as usize,
        );
        Status = STATUS_SUCCESS;
    } else {
        AFD_DbgPrint!(MIN_TRACE, b"Buffer too small\n\0".as_ptr() as PCHAR);
        Status = STATUS_BUFFER_TOO_SMALL;
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
//  * FILE:             drivers/net/afd/afd/info.c
//  * PURPOSE:          Ancillary functions driver
//  * PROGRAMMER:       Art Yerkes (ayerkes@speakeasy.net)
//  * UPDATE HISTORY:
//  * 20040708 Created
//  */
//
// #include "afd.h"
//
// NTSTATUS NTAPI
// AfdGetInfo( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//             PIO_STACK_LOCATION IrpSp ) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAFD_INFO InfoReq = LockRequest(Irp, IrpSp, TRUE, NULL);
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PLIST_ENTRY CurrentEntry;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     AFD_DbgPrint!(MID_TRACE,("Called %p %x\n", InfoReq,
//                             InfoReq ? InfoReq->InformationClass : 0));
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     if (!InfoReq)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     _SEH2_TRY {
//         switch( InfoReq->InformationClass ) {
//         case AFD_INFO_RECEIVE_WINDOW_SIZE:
//             InfoReq->Information.Ulong = FCB->Recv.Size;
//             break;
//
//         case AFD_INFO_SEND_WINDOW_SIZE:
//             InfoReq->Information.Ulong = FCB->Send.Size;
//             AFD_DbgPrint!(MID_TRACE,("Send window size %u\n", FCB->Send.Size));
//             break;
//
//         case AFD_INFO_GROUP_ID_TYPE:
//             InfoReq->Information.LargeInteger.u.HighPart = FCB->GroupType;
//             InfoReq->Information.LargeInteger.u.LowPart = FCB->GroupID;
//             AFD_DbgPrint!(MID_TRACE, ("Group ID: %u Group Type: %u\n", FCB->GroupID, FCB->GroupType));
//             break;
//
//         case AFD_INFO_BLOCKING_MODE:
//             InfoReq->Information.Boolean = FCB->NonBlocking;
//             break;
//
//     case AFD_INFO_INLINING_MODE:
//         InfoReq->Information.Boolean = FCB->OobInline;
//         break;
//
//     case AFD_INFO_RECEIVE_CONTENT_SIZE:
//         InfoReq->Information.Ulong = FCB->Recv.Content - FCB->Recv.BytesUsed;
//         break;
//
//         case AFD_INFO_SENDS_IN_PROGRESS:
//             InfoReq->Information.Ulong = 0;
//
//             /* Count the queued sends */
//             CurrentEntry = FCB->PendingIrpList[FUNCTION_SEND].Flink;
//             while (CurrentEntry != &FCB->PendingIrpList[FUNCTION_SEND])
//             {
//                  InfoReq->Information.Ulong++;
//                  CurrentEntry = CurrentEntry->Flink;
//             }
//
//         /* This needs to count too because when this is dispatched
//          * the user-mode IRP has already been completed and therefore
//          * will NOT be in our pending IRP list. We count this as one send
//          * outstanding although it could be multiple since we batch sends
//          * when waiting for the in flight request to return, so this number
//          * may not be accurate but it really doesn't matter that much since
//          * it's more or less a zero/non-zero comparison to determine whether
//          * we can shutdown the socket
//          */
//         if (FCB->SendIrp.InFlightRequest)
//             InfoReq->Information.Ulong++;
//         break;
//
//         default:
//             AFD_DbgPrint!(MIN_TRACE,("Unknown info id %x\n",
//                                     InfoReq->InformationClass));
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//         }
//     } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//         AFD_DbgPrint!(MIN_TRACE,("Exception executing GetInfo\n"));
//         Status = STATUS_INVALID_PARAMETER;
//     } _SEH2_END;
//
//     AFD_DbgPrint!(MID_TRACE,("Returning %x\n", Status));
//
//     return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
// }
//
// NTSTATUS NTAPI
// AfdSetInfo( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//             PIO_STACK_LOCATION IrpSp ) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAFD_INFO InfoReq = LockRequest(Irp, IrpSp, FALSE, NULL);
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PCHAR NewBuffer;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if (!SocketAcquireStateLock(FCB)) return LostSocket(Irp);
//
//     if (!InfoReq)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     _SEH2_TRY {
//         switch (InfoReq->InformationClass) {
//             case AFD_INFO_BLOCKING_MODE:
//                 AFD_DbgPrint!(MID_TRACE,("Blocking mode set to %u\n", InfoReq->Information.Boolean));
//                 FCB->NonBlocking = InfoReq->Information.Boolean;
//                 break;
//             case AFD_INFO_INLINING_MODE:
//                 FCB->OobInline = InfoReq->Information.Boolean;
//                 break;
//             case AFD_INFO_RECEIVE_WINDOW_SIZE:
//                 if (FCB->SharedData.State == SOCKET_STATE_CONNECTED ||
//                     FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS)
//                 {
//                     /* FIXME: likely not right, check tcpip.sys for TDI_QUERY_MAX_DATAGRAM_INFO */
//                     if (InfoReq->Information.Ulong > 0 && InfoReq->Information.Ulong < 0xFFFF &&
//                         InfoReq->Information.Ulong != FCB->Recv.Size)
//                     {
//                         NewBuffer = ExAllocatePoolWithTag(PagedPool,
//                                                           InfoReq->Information.Ulong,
//                                                           TAG_AFD_DATA_BUFFER);
//
//                         if (NewBuffer)
//                         {
//                             if (FCB->Recv.Content > InfoReq->Information.Ulong)
//                                 FCB->Recv.Content = InfoReq->Information.Ulong;
//
//                             if (FCB->Recv.Window)
//                             {
//                                 RtlCopyMemory(NewBuffer,
//                                               FCB->Recv.Window,
//                                               FCB->Recv.Content);
//
//                                 ExFreePoolWithTag(FCB->Recv.Window, TAG_AFD_DATA_BUFFER);
//                             }
//
//                             FCB->Recv.Size = InfoReq->Information.Ulong;
//                             FCB->Recv.Window = NewBuffer;
//
//                             Status = STATUS_SUCCESS;
//                         }
//                         else
//                         {
//                             Status = STATUS_NO_MEMORY;
//                         }
//                     }
//                     else
//                     {
//                         Status = STATUS_SUCCESS;
//                     }
//                 }
//                 else
//                 {
//                     Status = STATUS_INVALID_PARAMETER;
//                 }
//                 break;
//             case AFD_INFO_SEND_WINDOW_SIZE:
//                 if (FCB->SharedData.State == SOCKET_STATE_CONNECTED ||
//                     FCB->Flags & AFD_ENDPOINT_CONNECTIONLESS)
//                 {
//                     if (InfoReq->Information.Ulong > 0 && InfoReq->Information.Ulong < 0xFFFF &&
//                         InfoReq->Information.Ulong != FCB->Send.Size)
//                     {
//                         NewBuffer = ExAllocatePoolWithTag(PagedPool,
//                                                           InfoReq->Information.Ulong,
//                                                           TAG_AFD_DATA_BUFFER);
//
//                         if (NewBuffer)
//                         {
//                             if (FCB->Send.BytesUsed > InfoReq->Information.Ulong)
//                                 FCB->Send.BytesUsed = InfoReq->Information.Ulong;
//
//                             if (FCB->Send.Window)
//                             {
//                                 RtlCopyMemory(NewBuffer,
//                                               FCB->Send.Window,
//                                               FCB->Send.BytesUsed);
//
//                                 ExFreePoolWithTag(FCB->Send.Window, TAG_AFD_DATA_BUFFER);
//                             }
//
//                             FCB->Send.Size = InfoReq->Information.Ulong;
//                             FCB->Send.Window = NewBuffer;
//
//                             Status = STATUS_SUCCESS;
//                         }
//                         else
//                         {
//                             Status = STATUS_NO_MEMORY;
//                         }
//                     }
//                     else
//                     {
//                         Status = STATUS_SUCCESS;
//                     }
//                 }
//                 else
//                 {
//                     Status = STATUS_INVALID_PARAMETER;
//                 }
//                 break;
//             default:
//                 AFD_DbgPrint!(MIN_TRACE,("Unknown request %u\n", InfoReq->InformationClass));
//                 break;
//         }
//     } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//         AFD_DbgPrint!(MIN_TRACE,("Exception executing SetInfo\n"));
//         Status = STATUS_INVALID_PARAMETER;
//     } _SEH2_END;
//
//     AFD_DbgPrint!(MID_TRACE,("Returning %x\n", Status));
//
//     return UnlockAndMaybeComplete(FCB, Status, Irp, 0);
// }
//
// NTSTATUS NTAPI
// AfdGetSockName( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                 PIO_STACK_LOCATION IrpSp ) {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PMDL Mdl = NULL;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//     ASSERT(Irp->MdlAddress == NULL);
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     if( FCB->AddressFile.Object == NULL && FCB->Connection.Object == NULL ) {
//          return UnlockAndMaybeComplete( FCB, STATUS_INVALID_PARAMETER, Irp, 0 );
//     }
//
//     Mdl = IoAllocateMdl( Irp->UserBuffer,
//                          IrpSp->Parameters.DeviceIoControl.OutputBufferLength,
//                          FALSE,
//                          FALSE,
//                          NULL );
//
//     if( Mdl != NULL ) {
//         _SEH2_TRY {
//             MmProbeAndLockPages( Mdl, Irp->RequestorMode, IoModifyAccess );
//         } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
//             AFD_DbgPrint!(MIN_TRACE, ("MmProbeAndLockPages() failed.\n"));
//             Status = _SEH2_GetExceptionCode();
//         } _SEH2_END;
//
//         if( NT_SUCCESS(Status) ) {
//                 Status = TdiQueryInformation( FCB->Connection.Object
//                                                 ? FCB->Connection.Object
//                                                 : FCB->AddressFile.Object,
//                                               TDI_QUERY_ADDRESS_INFO,
//                                               Mdl );
//         }
//
//         /* Check if MmProbeAndLockPages or TdiQueryInformation failed and
//          * clean up Mdl */
//         if (!NT_SUCCESS(Status) && Irp->MdlAddress != Mdl)
//             IoFreeMdl(Mdl);
//     } else
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//
//     return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
// }
//
// NTSTATUS NTAPI
// AfdGetPeerName( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                 PIO_STACK_LOCATION IrpSp ) {
//     NTSTATUS Status;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     if (FCB->RemoteAddress == NULL) {
//         AFD_DbgPrint!(MIN_TRACE,("Invalid parameter\n"));
//         return UnlockAndMaybeComplete( FCB, STATUS_INVALID_PARAMETER, Irp, 0 );
//     }
//
//     if (IrpSp->Parameters.DeviceIoControl.OutputBufferLength >= TaLengthOfTransportAddress(FCB->RemoteAddress))
//     {
//         RtlCopyMemory(Irp->UserBuffer, FCB->RemoteAddress, TaLengthOfTransportAddress(FCB->RemoteAddress));
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Buffer too small\n"));
//         Status = STATUS_BUFFER_TOO_SMALL;
//     }
//
//     return UnlockAndMaybeComplete( FCB, Status, Irp, 0 );
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================