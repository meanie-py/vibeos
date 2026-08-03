use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ExFreePool, EVENT_TYPE, FILE_ALLOCATION_INFORMATION, FILE_END_OF_FILE_INFORMATION, _IO_STATUS_BLOCK, IoSetThreadHardErrorMode, LARGE_INTEGER, _OBJECT_ATTRIBUTES, ObReferenceObjectByHandle, PCMHIVE, PHANDLE, PHHIVE, PKEVENT, PLARGE_INTEGER, ZwClose, ZwCreateEvent, ZwFlushBuffersFile, ZwReadFile, ZwSetInformationFile, ZwWriteFile, _FILE_INFORMATION_CLASS_FileAllocationInformation, _FILE_INFORMATION_CLASS_FileEndOfFileInformation};

use ntkrnl_sys::opaque_ptr;
// EVENT_ALL_ACCESS is defined in xdk/ketypes.h
const EVENT_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x3;
const FileEndOfFileInformation: u32 = _FILE_INFORMATION_CLASS_FileEndOfFileInformation;
const FileAllocationInformation: u32 = _FILE_INFORMATION_CLASS_FileAllocationInformation;

extern "C" {
    static mut CmpNoWrite: u8;
}

#[no_mangle]
pub unsafe extern "system" fn CmpCreateEvent(
    EventType: EVENT_TYPE,
    EventHandle: PHANDLE,
    Event: *mut PKEVENT,
) -> NTSTATUS {
    let EventHandle = opaque_ptr(EventHandle);
    let Event = opaque_ptr(Event);
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.ObjectName = core::ptr::null_mut();
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateEvent(
        EventHandle,
        EVENT_ALL_ACCESS,
        &mut ObjectAttributes,
        EventType,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ObReferenceObjectByHandle(
        *EventHandle,
        EVENT_ALL_ACCESS,
        core::ptr::null_mut(),
        0i32 as i8,
        Event as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        ZwClose(*EventHandle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CmpAllocate(
    Size: usize,
    Paged: u8,
    Tag: u32,
) -> *mut core::ffi::c_void {
    ExAllocatePoolWithTag(
        if Paged != 0 { PagedPool } else { NonPagedPool },
        Size,
        Tag,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CmpFree(
    Ptr: PVOID,
    Quota: ULONG,
) {
    let Ptr = opaque_ptr(Ptr);
    ExFreePool(Ptr);
}

#[no_mangle]
pub unsafe extern "system" fn CmpFileRead(
    RegistryHive: PHHIVE,
    FileType: u32,
    FileOffset: *mut u32,
    Buffer: *mut core::ffi::c_void,
    BufferLength: usize,
) -> u8 {
    let RegistryHive = opaque_ptr(RegistryHive);
    let FileOffset = opaque_ptr(FileOffset);
    let Buffer = opaque_ptr(Buffer);
    let CmHive = RegistryHive as PCMHIVE;
    let HiveHandle = (*CmHive).FileHandles[FileType as usize];
    let mut _FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Status: i32;

    if HiveHandle.is_null() {
        return 1u8;
    }

    _FileOffset.QuadPart =(*FileOffset) as i64;
    Status = ZwReadFile(
        HiveHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        Buffer,
        BufferLength as u32,
        &mut _FileOffset,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(Status) { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn CmpFileWrite(
    RegistryHive: PHHIVE,
    FileType: ULONG,
    FileOffset: PULONG,
    Buffer: PVOID,
    BufferLength: SIZE_T,
) -> BOOLEAN {
    let RegistryHive = opaque_ptr(RegistryHive);
    let FileOffset = opaque_ptr(FileOffset);
    let Buffer = opaque_ptr(Buffer);
    let CmHive = RegistryHive as PCMHIVE;
    let HiveHandle = (*CmHive).FileHandles[FileType as usize];
    let mut _FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Status: NTSTATUS;

    if HiveHandle.is_null() {
        return 1u8;
    }

    if CmpNoWrite != 0 {
        return 1u8;
    }

    _FileOffset.QuadPart =*FileOffset as i64;
    Status = ZwWriteFile(
        HiveHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        Buffer,
        BufferLength as ULONG,
        &mut _FileOffset,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(Status) { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn CmpFileSetSize(
    RegistryHive: PHHIVE,
    FileType: ULONG,
    FileSize: ULONG,
    OldFileSize: ULONG,
) -> BOOLEAN {
    let RegistryHive = opaque_ptr(RegistryHive);
    let CmHive = RegistryHive as PCMHIVE;
    let HiveHandle = (*CmHive).FileHandles[FileType as usize];
    let mut EndOfFileInfo: FILE_END_OF_FILE_INFORMATION = core::mem::zeroed();
    let mut FileAllocationInfo: FILE_ALLOCATION_INFORMATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let HardErrors: BOOLEAN;
    let mut Status: NTSTATUS;

    if HiveHandle == core::ptr::null_mut() {
        crate::dbg_print!(b"No hive handle associated with the given hive\n\0");
        return 1u8;
    }

    HardErrors = IoSetThreadHardErrorMode(0u8);

    EndOfFileInfo.EndOfFile.QuadPart = FileSize as i64;
    Status = ZwSetInformationFile(
        HiveHandle,
        &mut IoStatusBlock,
        &mut EndOfFileInfo as *mut _ as PVOID,
        core::mem::size_of::<FILE_END_OF_FILE_INFORMATION>() as ULONG,
        FileEndOfFileInformation,
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"ZwSetInformationFile failed to set new size of end of file (Status 0x%lx)\n\0",
            Status
        );
        IoSetThreadHardErrorMode(HardErrors);
        return 0u8;
    }

    FileAllocationInfo.AllocationSize.QuadPart = FileSize as i64;
    Status = ZwSetInformationFile(
        HiveHandle,
        &mut IoStatusBlock,
        &mut FileAllocationInfo as *mut _ as PVOID,
        core::mem::size_of::<FILE_ALLOCATION_INFORMATION>() as ULONG,
        FileAllocationInformation,
    );
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(
            b"ZwSetInformationFile failed to set new of allocation file (Status 0x%lx)\n\0",
            Status
        );
        IoSetThreadHardErrorMode(HardErrors);
        return 0u8;
    }

    IoSetThreadHardErrorMode(HardErrors);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CmpFileFlush(
    RegistryHive: PHHIVE,
    FileType: u32,
    FileOffset: PLARGE_INTEGER,
    Length: u32,
) -> u8 {
    let RegistryHive = opaque_ptr(RegistryHive);
    let FileOffset = opaque_ptr(FileOffset);
    let CmHive = RegistryHive as PCMHIVE;
    let HiveHandle = (*CmHive).FileHandles[FileType as usize];
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Status: i32;

    if HiveHandle.is_null() {
        return 1u8;
    }

    if CmpNoWrite != 0 {
        return 1u8;
    }

    Status = ZwFlushBuffersFile(HiveHandle, &mut IoStatusBlock);

    if NT_SUCCESS(Status) { 1u8 } else { 0u8 }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/config/cmwraprs.c
//  * PURPOSE:         Configuration Manager - Wrappers for Hive Operations
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "ntoskrnl.h"
// #define NDEBUG
// #include "debug.h"
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// CmpCreateEvent(IN EVENT_TYPE EventType,
//                OUT PHANDLE EventHandle,
//                OUT PKEVENT *Event)
// {
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* Create the event */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwCreateEvent(EventHandle,
//                            EVENT_ALL_ACCESS,
//                            &ObjectAttributes,
//                            EventType,
//                            FALSE);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get a pointer to the object itself */
//     Status = ObReferenceObjectByHandle(*EventHandle,
//                                        EVENT_ALL_ACCESS,
//                                        NULL,
//                                        KernelMode,
//                                        (PVOID*)Event,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) ZwClose(*EventHandle);
//
//     /* Return status */
//     return Status;
// }
//
// PVOID
// NTAPI
// CmpAllocate(IN SIZE_T Size,
//             IN BOOLEAN Paged,
//             IN ULONG Tag)
// {
//     return ExAllocatePoolWithTag(Paged ? PagedPool : NonPagedPool,
//                                  Size,
//                                  Tag);
// }
//
// VOID
// NTAPI
// CmpFree(IN PVOID Ptr,
//         IN ULONG Quota)
// {
//     ExFreePool(Ptr);
// }
//
// BOOLEAN
// NTAPI
// CmpFileRead(IN PHHIVE RegistryHive,
//             IN ULONG FileType,
//             IN PULONG FileOffset,
//             OUT PVOID Buffer,
//             IN SIZE_T BufferLength)
// {
//     PCMHIVE CmHive = (PCMHIVE)RegistryHive;
//     HANDLE HiveHandle = CmHive->FileHandles[FileType];
//     LARGE_INTEGER _FileOffset;
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//
//     /* Just return success if no file is associated with this hive */
//     if (HiveHandle == NULL)
//         return TRUE;
//
//     _FileOffset.QuadPart = *FileOffset;
//     Status = ZwReadFile(HiveHandle, NULL, NULL, NULL, &IoStatusBlock,
//                         Buffer, (ULONG)BufferLength, &_FileOffset, NULL);
//     /* We do synchronous I/O for simplicity - see CmpOpenHiveFiles. */
//     ASSERT(Status != STATUS_PENDING);
//     return NT_SUCCESS(Status) ? TRUE : FALSE;
// }
//
// BOOLEAN
// NTAPI
// CmpFileWrite(IN PHHIVE RegistryHive,
//              IN ULONG FileType,
//              IN PULONG FileOffset,
//              IN PVOID Buffer,
//              IN SIZE_T BufferLength)
// {
//     PCMHIVE CmHive = (PCMHIVE)RegistryHive;
//     HANDLE HiveHandle = CmHive->FileHandles[FileType];
//     LARGE_INTEGER _FileOffset;
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//
//     /* Just return success if no file is associated with this hive */
//     if (HiveHandle == NULL)
//         return TRUE;
//
//     /* Don't do anything if we're not supposed to */
//     if (CmpNoWrite)
//         return TRUE;
//
//     _FileOffset.QuadPart = *FileOffset;
//     Status = ZwWriteFile(HiveHandle, NULL, NULL, NULL, &IoStatusBlock,
//                          Buffer, (ULONG)BufferLength, &_FileOffset, NULL);
//     /* We do synchronous I/O for simplicity - see CmpOpenHiveFiles.
//      * Windows optimizes here by starting an async write for each 64k chunk,
//      * then waiting for all writes to complete at once.
//      */
//     ASSERT(Status != STATUS_PENDING);
//     return NT_SUCCESS(Status) ? TRUE : FALSE;
// }
//
// BOOLEAN
// NTAPI
// CmpFileSetSize(
//     _In_ PHHIVE RegistryHive,
//     _In_ ULONG FileType,
//     _In_ ULONG FileSize,
//     _In_ ULONG OldFileSize)
// {
//     PCMHIVE CmHive = (PCMHIVE)RegistryHive;
//     HANDLE HiveHandle = CmHive->FileHandles[FileType];
//     FILE_END_OF_FILE_INFORMATION EndOfFileInfo;
//     FILE_ALLOCATION_INFORMATION FileAllocationInfo;
//     IO_STATUS_BLOCK IoStatusBlock;
//     BOOLEAN HardErrors;
//     NTSTATUS Status;
//
//     /* Just return success if no file is associated with this hive */
//     if (HiveHandle == NULL)
//     {
//         DPRINT1("No hive handle associated with the given hive\n");
//         return TRUE;
//     }
//
//     /*
//      * Disable hard errors so that we don't deadlock
//      * when touching with the hive files.
//      */
//     HardErrors = IoSetThreadHardErrorMode(FALSE);
//
//     EndOfFileInfo.EndOfFile.QuadPart = FileSize;
//     Status = ZwSetInformationFile(HiveHandle,
//                                   &IoStatusBlock,
//                                   &EndOfFileInfo,
//                                   sizeof(FILE_END_OF_FILE_INFORMATION),
//                                   FileEndOfFileInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwSetInformationFile failed to set new size of end of file (Status 0x%lx)\n", Status);
//         IoSetThreadHardErrorMode(HardErrors);
//         return FALSE;
//     }
//
//     FileAllocationInfo.AllocationSize.QuadPart = FileSize;
//     Status = ZwSetInformationFile(HiveHandle,
//                                   &IoStatusBlock,
//                                   &FileAllocationInfo,
//                                   sizeof(FILE_ALLOCATION_INFORMATION),
//                                   FileAllocationInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwSetInformationFile failed to set new of allocation file (Status 0x%lx)\n", Status);
//         IoSetThreadHardErrorMode(HardErrors);
//         return FALSE;
//     }
//
//     /* Reset the hard errors back */
//     IoSetThreadHardErrorMode(HardErrors);
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// CmpFileFlush(IN PHHIVE RegistryHive,
//              IN ULONG FileType,
//              IN OUT PLARGE_INTEGER FileOffset,
//              IN ULONG Length)
// {
//     PCMHIVE CmHive = (PCMHIVE)RegistryHive;
//     HANDLE HiveHandle = CmHive->FileHandles[FileType];
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//
//     /* Just return success if no file is associated with this hive */
//     if (HiveHandle == NULL)
//         return TRUE;
//
//     /* Don't do anything if we're not supposed to */
//     if (CmpNoWrite)
//         return TRUE;
//
//     Status = ZwFlushBuffersFile(HiveHandle, &IoStatusBlock);
//
//     /* This operation is always synchronous */
//     ASSERT(Status != STATUS_PENDING);
//     ASSERT(Status == IoStatusBlock.Status);
//
//     return NT_SUCCESS(Status) ? TRUE : FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
