use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP};

extern "C" {
    fn LockRequest(Irp: PIRP, IrpSp: PIO_STACK_LOCATION, Force: BOOLEAN, Size: PULONG) -> PVOID;
    fn LostSocket(Irp: PIRP) -> NTSTATUS;
    fn SocketAcquireStateLock(FCB: PAFD_FCB) -> BOOLEAN;
}
#[no_mangle]
pub unsafe extern "system" fn AfdGetContext(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_INVALID_PARAMETER;
    let FileObject: PFILE_OBJECT = (*IrpSp).FileObject;
    let FCB: PAFD_FCB = (*FileObject).FsContext as PAFD_FCB;
    let mut ContextSize: UINT = (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if ContextSize == core::mem::size_of_val(&(*FCB).SharedData) as UINT {
        RtlCopyMemory(
            (*Irp).UserBuffer,
            core::ptr::addr_of_mut!((*FCB).SharedData) as PVOID,
            ContextSize as usize,
        );
        Status = STATUS_SUCCESS;
    } else if !(*FCB).Context.is_null() {
        ContextSize = core::cmp::min(ContextSize, (*FCB).ContextSize);
        RtlCopyMemory(
            (*Irp).UserBuffer,
            (*FCB).Context as PVOID,
            ContextSize as usize,
        );
        Status = STATUS_SUCCESS;
    }

    AFD_DbgPrint!(MID_TRACE, b"Returning %x\n\0".as_ptr() as PCHAR, Status);

    return UnlockAndMaybeComplete(FCB, Status, Irp, ContextSize);
}

#[no_mangle]
pub unsafe extern "system" fn AfdGetContextSize(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if (*IrpSp).Parameters.DeviceIoControl.OutputBufferLength < core::mem::size_of::<ULONG>() as u32 {
        AFD_DbgPrint!(MIN_TRACE, b"Buffer too small\n\0".as_ptr() as PCHAR);
        return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, core::mem::size_of::<ULONG>() as u32);
    }

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*FCB).ContextSize) as *const u8,
        (*Irp).UserBuffer as *mut u8,
        core::mem::size_of::<ULONG>(),
    );

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, core::mem::size_of::<ULONG>() as u32)
}

#[no_mangle]
pub unsafe extern "system" fn AfdSetContext(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {

    let FileObject = (*IrpSp).FileObject;
    let FCB = (*FileObject).FsContext as PAFD_FCB;
    let Context = LockRequest(Irp, IrpSp, 0u8, core::ptr::null_mut());

    UNREFERENCED_PARAMETER!(DeviceObject);

    if SocketAcquireStateLock(FCB) == 0 {
        return LostSocket(Irp);
    }

    if Context.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    if !(*FCB).Context.is_null() {
        ExFreePoolWithTag((*FCB).Context, TAG_AFD_SOCKET_CONTEXT);
        (*FCB).ContextSize = 0;
    }

    (*FCB).Context = ExAllocatePoolWithTag(
        PagedPool,
        (*IrpSp).Parameters.DeviceIoControl.InputBufferLength as usize,
        TAG_AFD_SOCKET_CONTEXT,
    );

    if (*FCB).Context.is_null() {
        return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
    }

    (*FCB).ContextSize = (*IrpSp).Parameters.DeviceIoControl.InputBufferLength;

    core::ptr::copy_nonoverlapping(
        Context as *const u8,
        (*FCB).Context as *mut u8,
        (*FCB).ContextSize as usize,
    );

    UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, 0)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             drivers/net/afd/afd/context.c
//  * PURPOSE:          Ancillary functions driver
//  * PROGRAMMER:       Art Yerkes (ayerkes@speakeasy.net)
//  * UPDATE HISTORY:
//  * 20040708 Created
//  */
//
// #include "afd.h"
//
// NTSTATUS NTAPI
// AfdGetContext( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                PIO_STACK_LOCATION IrpSp ) {
//     NTSTATUS Status = STATUS_INVALID_PARAMETER;
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     UINT ContextSize = IrpSp->Parameters.DeviceIoControl.OutputBufferLength;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     if (ContextSize == sizeof(FCB->SharedData))
//     {
//         RtlCopyMemory(Irp->UserBuffer,
//                       &FCB->SharedData,
//                       ContextSize);
//         Status = STATUS_SUCCESS;
//     }
//     else if (FCB->Context)
//     {
//         ContextSize = min(ContextSize, FCB->ContextSize);
//         RtlCopyMemory( Irp->UserBuffer,
//                        FCB->Context,
//                        ContextSize );
//         Status = STATUS_SUCCESS;
//     }
//
//     AFD_DbgPrint!(MID_TRACE,("Returning %x\n", Status));
//
//     return UnlockAndMaybeComplete( FCB, Status, Irp, ContextSize );
// }
//
// NTSTATUS NTAPI
// AfdGetContextSize( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                    PIO_STACK_LOCATION IrpSp )
// {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     if (IrpSp->Parameters.DeviceIoControl.OutputBufferLength < sizeof(ULONG))
//     {
//         AFD_DbgPrint!(MIN_TRACE,("Buffer too small\n"));
//         return UnlockAndMaybeComplete(FCB, STATUS_BUFFER_TOO_SMALL, Irp, sizeof(ULONG));
//     }
//
//     RtlCopyMemory(Irp->UserBuffer,
//                   &FCB->ContextSize,
//                   sizeof(ULONG));
//
//     return UnlockAndMaybeComplete(FCB, STATUS_SUCCESS, Irp, sizeof(ULONG));
// }
//
// NTSTATUS NTAPI
// AfdSetContext( PDEVICE_OBJECT DeviceObject, PIRP Irp,
//                PIO_STACK_LOCATION IrpSp ) {
//     PFILE_OBJECT FileObject = IrpSp->FileObject;
//     PAFD_FCB FCB = FileObject->FsContext;
//     PVOID Context = LockRequest(Irp, IrpSp, FALSE, NULL);
//
//     UNREFERENCED_PARAMETER(DeviceObject);
//
//     if( !SocketAcquireStateLock( FCB ) ) return LostSocket( Irp );
//
//     if (!Context)
//         return UnlockAndMaybeComplete(FCB, STATUS_NO_MEMORY, Irp, 0);
//
//     if( FCB->Context ) {
//         ExFreePoolWithTag(FCB->Context, TAG_AFD_SOCKET_CONTEXT);
//         FCB->ContextSize = 0;
//     }
//
//     FCB->Context = ExAllocatePoolWithTag(PagedPool,
//                                          IrpSp->Parameters.DeviceIoControl.InputBufferLength,
//                                          TAG_AFD_SOCKET_CONTEXT);
//
//     if( !FCB->Context ) return UnlockAndMaybeComplete( FCB, STATUS_NO_MEMORY, Irp, 0 );
//
//     FCB->ContextSize = IrpSp->Parameters.DeviceIoControl.InputBufferLength;
//
//     RtlCopyMemory( FCB->Context,
//                    Context,
//                    FCB->ContextSize );
//
//     return UnlockAndMaybeComplete( FCB, STATUS_SUCCESS, Irp, 0 );
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================