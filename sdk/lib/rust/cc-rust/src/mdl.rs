use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    IoFreeMdl, IoGetRelatedDeviceObject, MmUnlockPages,
    PDEVICE_OBJECT, PFAST_IO_DISPATCH, PFILE_OBJECT, PIO_STATUS_BLOCK,
    PLARGE_INTEGER, PMDL,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub unsafe extern "system" fn CcMdlReadComplete2(
    FileObject: PFILE_OBJECT,
    MemoryDescriptorList: PMDL,
) {
    let FileObject = opaque_ptr(FileObject);
    let MemoryDescriptorList = opaque_ptr(MemoryDescriptorList);
    let mut mdl = MemoryDescriptorList;
    
    while !mdl.is_null() {
        let next = (*mdl).Next;
        MmUnlockPages(mdl);
        IoFreeMdl(mdl);
        mdl = next;
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcMdlWriteComplete2(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    MdlChain: PMDL,
) {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let MdlChain = opaque_ptr(MdlChain);
    // UNIMPLEMENTED in original C source
}

#[no_mangle]
pub unsafe extern "system" fn CcMdlRead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    MdlChain: *mut PMDL,
    IoStatus: PIO_STATUS_BLOCK,
) {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let MdlChain = opaque_ptr(MdlChain);
    let IoStatus = opaque_ptr(IoStatus);
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "system" fn CcMdlReadComplete(
    FileObject: PFILE_OBJECT,
    MdlChain: PMDL,
) {
    let FileObject = opaque_ptr(FileObject);
    let MdlChain = opaque_ptr(MdlChain);
    let mut device_object: PDEVICE_OBJECT = core::ptr::null_mut();
    let fast_dispatch: PFAST_IO_DISPATCH;

    device_object = IoGetRelatedDeviceObject(FileObject);
    fast_dispatch = (*(*device_object).DriverObject).FastIoDispatch;

    if !fast_dispatch.is_null() && (*fast_dispatch).MdlReadComplete.is_some() {
        if ((*fast_dispatch).MdlReadComplete.unwrap())(
            FileObject,
            MdlChain,
            device_object,
        ) != 0
        {
            return;
        }
    }

    CcMdlReadComplete2(FileObject, MdlChain);
}

#[no_mangle]
pub unsafe extern "system" fn CcMdlWriteComplete(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    MdlChain: PMDL,
) {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let MdlChain = opaque_ptr(MdlChain);
    let mut device_object: PDEVICE_OBJECT = core::ptr::null_mut();
    let fast_dispatch: PFAST_IO_DISPATCH;

    device_object = IoGetRelatedDeviceObject(FileObject);
    fast_dispatch = (*(*device_object).DriverObject).FastIoDispatch;

    if !fast_dispatch.is_null() && (*fast_dispatch).MdlWriteComplete.is_some() {
        if ((*fast_dispatch).MdlWriteComplete.unwrap())(
            FileObject,
            FileOffset,
            MdlChain,
            device_object,
        ) != 0
        {
            return;
        }
    }

    CcMdlWriteComplete2(FileObject, FileOffset, MdlChain);
}

#[no_mangle]
pub unsafe extern "system" fn CcMdlWriteAbort(
    FileObject: PFILE_OBJECT,
    MdlChain: PMDL,
) {
    let FileObject = opaque_ptr(FileObject);
    let MdlChain = opaque_ptr(MdlChain);
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "system" fn CcPrepareMdlWrite(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: ULONG,
    MdlChain: *mut PMDL,
    IoStatus: PIO_STATUS_BLOCK,
) {
    let FileObject = opaque_ptr(FileObject);
    let FileOffset = opaque_ptr(FileOffset);
    let MdlChain = opaque_ptr(MdlChain);
    let IoStatus = opaque_ptr(IoStatus);
    unimplemented!();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/cc/mdl.c
//  * PURPOSE:         Implements MDL Cache Manager Functions
//  *
//  * PROGRAMMERS:     Alex Ionescu
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// CcMdlRead (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER FileOffset,
//     IN ULONG Length,
//     OUT PMDL * MdlChain,
//     OUT PIO_STATUS_BLOCK IoStatus
//     )
// {
//     CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%I64d Length=%lu\n",
//         FileObject, FileOffset->QuadPart, Length);
//
//     UNIMPLEMENTED;
// }
//
// /*
//  * NAME                            INTERNAL
//  * CcMdlReadComplete2@8
//  *
//  * DESCRIPTION
//  *
//  * ARGUMENTS
//  * MdlChain
//  * DeviceObject
//  *
//  * RETURN VALUE
//  *  None.
//  *
//  * NOTE
//  *     Used by CcMdlReadComplete@8 and FsRtl
//  *
//  */
// VOID
// NTAPI
// CcMdlReadComplete2 (
//     IN PFILE_OBJECT FileObject,
//     IN PMDL MemoryDescriptorList
// )
// {
//     PMDL Mdl;
//
//     /* Free MDLs */
//     while ((Mdl = MemoryDescriptorList))
//     {
//         MemoryDescriptorList = Mdl->Next;
//         MmUnlockPages(Mdl);
//         IoFreeMdl(Mdl);
//     }
// }
//
// /*
//  * NAME    EXPORTED
//  * CcMdlReadComplete@8
//  *
//  * DESCRIPTION
//  *
//  * ARGUMENTS
//  *
//  * RETURN VALUE
//  * None.
//  *
//  * NOTE
//  * From Bo Branten's ntifs.h v13.
//  *
//  * @implemented
//  */
// VOID
// NTAPI
// CcMdlReadComplete (
//     IN PFILE_OBJECT FileObject,
//     IN PMDL MdlChain)
// {
//     PDEVICE_OBJECT DeviceObject = NULL;
//     PFAST_IO_DISPATCH FastDispatch;
//
//     /* Get Fast Dispatch Data */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if we support Fast Calls, and check this one */
//     if (FastDispatch && FastDispatch->MdlReadComplete)
//     {
//         /* Use the fast path */
//         if (FastDispatch->MdlReadComplete(FileObject,
//                                           MdlChain,
//                                           DeviceObject))
//         {
//             /* Request was handled */
//             return;
//         }
//     }
//
//     /* Use slow path */
//     CcMdlReadComplete2(FileObject, MdlChain);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// CcMdlWriteComplete (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER FileOffset,
//     IN PMDL MdlChain)
// {
//     PDEVICE_OBJECT DeviceObject = NULL;
//     PFAST_IO_DISPATCH FastDispatch;
//
//     /* Get Fast Dispatch Data */
//     DeviceObject = IoGetRelatedDeviceObject(FileObject);
//     FastDispatch = DeviceObject->DriverObject->FastIoDispatch;
//
//     /* Check if we support Fast Calls, and check this one */
//     if (FastDispatch && FastDispatch->MdlWriteComplete)
//     {
//         /* Use the fast path */
//         if (FastDispatch->MdlWriteComplete(FileObject,
//                                            FileOffset,
//                                            MdlChain,
//                                            DeviceObject))
//         {
//             /* Request was handled */
//             return;
//         }
//     }
//
//     /* Use slow path */
//     CcMdlWriteComplete2(FileObject,FileOffset, MdlChain);
// }
//
// VOID
// NTAPI
// CcMdlWriteComplete2 (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER FileOffset,
//     IN PMDL MdlChain)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// CcMdlWriteAbort (
//     IN PFILE_OBJECT FileObject,
//     IN PMDL MdlChain)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// CcPrepareMdlWrite (
//     IN PFILE_OBJECT FileObject,
//     IN PLARGE_INTEGER FileOffset,
//     IN ULONG Length,
//     OUT PMDL * MdlChain,
//     OUT PIO_STATUS_BLOCK IoStatus)
// {
//     CCTRACE(CC_API_DEBUG, "FileObject=%p FileOffset=%I64d Length=%lu\n",
//         FileObject, FileOffset->QuadPart, Length);
//
//     UNIMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
