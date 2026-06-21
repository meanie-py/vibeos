use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, PDEVICE_OBJECT, PFILE_OBJECT, PSECURITY_DESCRIPTOR, HANDLE};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();
#[no_mangle]
pub unsafe extern "system" fn FltBuildDefaultSecurityDescriptor(
    SecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    DesiredAccess: ACCESS_MASK,
) -> NTSTATUS {
    let _ = DesiredAccess;
    *SecurityDescriptor = core::ptr::null_mut();
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn FltFreeSecurityDescriptor(SecurityDescriptor: PSECURITY_DESCRIPTOR) {
    let _ = SecurityDescriptor;
}
#[no_mangle]
pub unsafe extern "system" fn FltGetDiskDeviceObject(
    Volume: PFLT_VOLUME,
    DiskDeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let _ = Volume;
    let _ = DiskDeviceObject;
    return 0 as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn FltGetFileNameInformationUnsafe(
    mut FileObject: PFILE_OBJECT,
    mut Instance: PFLT_INSTANCE,
    mut NameOptions: FLT_FILE_NAME_OPTIONS,
    mut FileNameInformation: *mut PFLT_FILE_NAME_INFORMATION,
) -> NTSTATUS {
    *FileNameInformation = core::ptr::null_mut();
    return 0 as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn FltGetFileNameInformation(
    mut CallbackData: PFLT_CALLBACK_DATA,
    mut NameOptions: FLT_FILE_NAME_OPTIONS,
    mut FileNameInformation: *mut PFLT_FILE_NAME_INFORMATION,
) -> NTSTATUS {
    let _ = CallbackData;
    let _ = NameOptions;
    *FileNameInformation = core::ptr::null_mut();
    return 0 as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn FltGetDestinationFileNameInformation(
    Instance: PFLT_INSTANCE,
    FileObject: PFILE_OBJECT,
    RootDirectory: HANDLE,
    FileName: PWSTR,
    FileNameLength: ULONG,
    NameOptions: FLT_FILE_NAME_OPTIONS,
    RetFileNameInformation: *mut PFLT_FILE_NAME_INFORMATION,
) -> NTSTATUS {
    let _ = Instance;
    let _ = FileObject;
    let _ = RootDirectory;
    let _ = FileName;
    let _ = FileNameLength;
    let _ = NameOptions;
    *RetFileNameInformation = core::ptr::null_mut();
    return 0 as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn FltReleaseFileNameInformation(
    FileNameInformation: PFLT_FILE_NAME_INFORMATION,
) {
    let _ = FileNameInformation;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         Filesystem Filter Manager
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            drivers/filters/fltmgr/Misc.c
//  * PURPOSE:         Uncataloged functions
//  * PROGRAMMERS:     Ged Murphy (gedmurphy@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "fltmgr.h"
// #include "fltmgrint.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* DATA *********************************************************************/
//
//
//
//
// /* EXPORTED FUNCTIONS ******************************************************/
//
// NTSTATUS
// FLTAPI
// FltBuildDefaultSecurityDescriptor(
//      PSECURITY_DESCRIPTOR *SecurityDescriptor,
//      ACCESS_MASK DesiredAccess
// )
// {
//     UNREFERENCED_PARAMETER(DesiredAccess);
//     *SecurityDescriptor = NULL;
//     return 0;
// }
//
// VOID
// FLTAPI
// FltFreeSecurityDescriptor(
//      PSECURITY_DESCRIPTOR SecurityDescriptor
// )
// {
//     UNREFERENCED_PARAMETER(SecurityDescriptor);
// }
//
// NTSTATUS
// FLTAPI
// FltGetDiskDeviceObject(
//      PFLT_VOLUME Volume,
//      PDEVICE_OBJECT *DiskDeviceObject
// )
// {
//     UNREFERENCED_PARAMETER(Volume);
//     UNREFERENCED_PARAMETER(DiskDeviceObject);
//     return 0;
// }
//
// NTSTATUS
// FLTAPI
// FltGetFileNameInformationUnsafe(
//      PFILE_OBJECT FileObject,
//      PFLT_INSTANCE Instance,
//      FLT_FILE_NAME_OPTIONS NameOptions,
//      PFLT_FILE_NAME_INFORMATION *FileNameInformation)
// {
//     UNREFERENCED_PARAMETER(FileObject);
//     UNREFERENCED_PARAMETER(Instance);
//     UNREFERENCED_PARAMETER(NameOptions);
//     *FileNameInformation = NULL;
//     return 0;
// }
//
// NTSTATUS
// FLTAPI
// FltGetFileNameInformation(
//      PFLT_CALLBACK_DATA CallbackData,
//      FLT_FILE_NAME_OPTIONS NameOptions,
//      PFLT_FILE_NAME_INFORMATION *FileNameInformation)
// {
//     UNREFERENCED_PARAMETER(CallbackData);
//     UNREFERENCED_PARAMETER(NameOptions);
//     *FileNameInformation = NULL;
//     return 0;
// }
//
// NTSTATUS
// FLTAPI
// FltGetDestinationFileNameInformation(
//      PFLT_INSTANCE Instance,
//      PFILE_OBJECT FileObject,
//      HANDLE RootDirectory,
//      PWSTR FileName,
//      ULONG FileNameLength,
//      FLT_FILE_NAME_OPTIONS NameOptions,
//      PFLT_FILE_NAME_INFORMATION *RetFileNameInformation)
// {
//     UNREFERENCED_PARAMETER(Instance);
//     UNREFERENCED_PARAMETER(FileObject);
//     UNREFERENCED_PARAMETER(RootDirectory);
//     UNREFERENCED_PARAMETER(FileName);
//     UNREFERENCED_PARAMETER(FileNameLength);
//     UNREFERENCED_PARAMETER(NameOptions);
//     *RetFileNameInformation = NULL;
//     return 0;
// }
//
// VOID
// FLTAPI
// FltReleaseFileNameInformation(
//      PFLT_FILE_NAME_INFORMATION FileNameInformation)
// {
//     UNREFERENCED_PARAMETER(FileNameInformation);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
