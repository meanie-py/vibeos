use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    PBOOT_ENTRY, PBOOT_OPTIONS, PEFI_DRIVER_ENTRY, PFILE_PATH,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub unsafe extern "system" fn NtAddBootEntry(
    Entry: PBOOT_ENTRY,
    Id: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtAddDriverEntry(
    Entry: PEFI_DRIVER_ENTRY,
    Id: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtDeleteBootEntry(Id: ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtDeleteDriverEntry(Id: ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtEnumerateBootEntries(
    Buffer: PVOID,
    BufferLength: *mut ULONG,
) -> NTSTATUS {
    let Buffer = opaque_ptr(Buffer);
    let BufferLength = opaque_ptr(BufferLength);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtEnumerateDriverEntries(
    Buffer: PVOID,
    BufferLength: *mut ULONG,
) -> NTSTATUS {
    let Buffer = opaque_ptr(Buffer);
    let BufferLength = opaque_ptr(BufferLength);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtModifyBootEntry(BootEntry: PBOOT_ENTRY) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtModifyDriverEntry(
    DriverEntry: PEFI_DRIVER_ENTRY,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryBootEntryOrder(
    Ids: *mut ULONG,
    Count: *mut ULONG,
) -> NTSTATUS {
    let Ids = opaque_ptr(Ids);
    let Count = opaque_ptr(Count);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryDriverEntryOrder(
    Ids: *mut ULONG,
    Count: *mut ULONG,
) -> NTSTATUS {
    let Ids = opaque_ptr(Ids);
    let Count = opaque_ptr(Count);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryBootOptions(
    BootOptions: PBOOT_OPTIONS,
    BootOptionsLength: *mut ULONG,
) -> NTSTATUS {
    let BootOptionsLength = opaque_ptr(BootOptionsLength);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtSetBootEntryOrder(
    Ids: *mut ULONG,
    Count: *mut ULONG,
) -> NTSTATUS {
    let Ids = opaque_ptr(Ids);
    let Count = opaque_ptr(Count);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtSetDriverEntryOrder(
    Ids: *mut ULONG,
    Count: *mut ULONG,
) -> NTSTATUS {
    let Ids = opaque_ptr(Ids);
    let Count = opaque_ptr(Count);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtSetBootOptions(
    BootOptions: PBOOT_OPTIONS,
    FieldsToChange: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtTranslateFilePath(
    InputFilePath: PFILE_PATH,
    OutputType: ULONG,
    OutputFilePath: PFILE_PATH,
    OutputFilePathLength: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/efi.c
//  * PURPOSE:         I/O Functions for EFI Machines
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// NtAddBootEntry(IN PBOOT_ENTRY Entry,
//                IN ULONG Id)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtAddDriverEntry(IN PEFI_DRIVER_ENTRY Entry,
//                  IN ULONG Id)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtDeleteBootEntry(IN ULONG Id)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtDeleteDriverEntry(IN ULONG Id)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtEnumerateBootEntries(IN PVOID Buffer,
//                        IN PULONG BufferLength)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtEnumerateDriverEntries(IN PVOID Buffer,
//                         IN PULONG BufferLength)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtModifyBootEntry(IN PBOOT_ENTRY BootEntry)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtModifyDriverEntry(IN PEFI_DRIVER_ENTRY DriverEntry)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtQueryBootEntryOrder(IN PULONG Ids,
//                       IN PULONG Count)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtQueryDriverEntryOrder(IN PULONG Ids,
//                         IN PULONG Count)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtQueryBootOptions(IN PBOOT_OPTIONS BootOptions,
//                    IN PULONG BootOptionsLength)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtSetBootEntryOrder(IN PULONG Ids,
//                     IN PULONG Count)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtSetDriverEntryOrder(IN PULONG Ids,
//                       IN PULONG Count)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtSetBootOptions(IN PBOOT_OPTIONS BootOptions,
//                  IN ULONG FieldsToChange)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtTranslateFilePath(PFILE_PATH InputFilePath,
//                     ULONG OutputType,
//                     PFILE_PATH OutputFilePath,
//                     ULONG OutputFilePathLength)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
