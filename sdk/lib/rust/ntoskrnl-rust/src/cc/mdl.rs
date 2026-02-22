use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{IoFreeMdl, IoGetRelatedDeviceObject, MmUnlockPages, PDEVICE_OBJECT, PFAST_IO_DISPATCH, PFILE_OBJECT, PIO_STATUS_BLOCK, PLARGE_INTEGER, PMDL};
use core::ffi::c_void;

extern "C" {
    fn CCTRACE(level: u32, format: *const i8, ...);
}

#[no_mangle]
pub unsafe extern "system" fn CcMdlRead(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    MdlChain: *mut PMDL,
    IoStatus: PIO_STATUS_BLOCK
) {
    
    CCTRACE(
        0x00000001u32,
        b"FileObject=%p FileOffset=%I64d Length=%lu\n\0".as_ptr() as *const i8,
        FileObject,
        (*FileOffset).QuadPart,
        Length
    );
    
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "system" fn CcMdlReadComplete2(
    FileObject: PFILE_OBJECT,
    MemoryDescriptorList: PMDL,
) {
    let mut Mdl: PMDL;
    let mut MemoryDescriptorList = MemoryDescriptorList;

    loop {
        Mdl = MemoryDescriptorList;
        if Mdl.is_null() {
            break;
        }
        MemoryDescriptorList = (*Mdl).Next;
        MmUnlockPages(Mdl);
        IoFreeMdl(Mdl);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CcMdlReadComplete(
    FileObject: PFILE_OBJECT,
    MdlChain: PMDL,
) {
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let FastDispatch: PFAST_IO_DISPATCH;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && !(*FastDispatch).MdlReadComplete.is_null() {
        if ((*FastDispatch).MdlReadComplete.unwrap())(
            FileObject,
            MdlChain,
            DeviceObject,
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
    let mut DeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let FastDispatch: PFAST_IO_DISPATCH;

    DeviceObject = IoGetRelatedDeviceObject(FileObject);
    FastDispatch = (*(*DeviceObject).DriverObject).FastIoDispatch;

    if !FastDispatch.is_null() && (*FastDispatch).MdlWriteComplete.is_some() {
        if ((*FastDispatch).MdlWriteComplete.unwrap())(
            FileObject,
            FileOffset,
            MdlChain,
            DeviceObject,
        ) != 0
        {
            return;
        }
    }

    CcMdlWriteComplete2(FileObject, FileOffset, MdlChain);
}

#[no_mangle]
pub unsafe extern "system" fn CcMdlWriteComplete2(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    MdlChain: PMDL,
) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn CcMdlWriteAbort(
    FileObject: PFILE_OBJECT,
    MdlChain: PMDL,
) {
    unimplemented!("CcMdlWriteAbort");
}

#[no_mangle]
pub unsafe extern "system" fn CcPrepareMdlWrite(
    FileObject: PFILE_OBJECT,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
    MdlChain: *mut PMDL,
    IoStatus: PIO_STATUS_BLOCK,
) {
    
    CCTRACE(
        0x00000001,
        b"FileObject=%p FileOffset=%I64d Length=%lu\n\0".as_ptr() as *const i8,
        FileObject,
        (*FileOffset).QuadPart,
        Length,
    );
    
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
