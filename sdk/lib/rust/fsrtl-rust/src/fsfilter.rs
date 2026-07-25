use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    ACCESS_MASK,
    KeBugCheck,
    PFILE_OBJECT,
    PHANDLE,
    PLARGE_INTEGER,
    POBJECT_ATTRIBUTES,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub unsafe extern "system" fn FsRtlCreateSectionForDataScan(
    SectionHandle: PHANDLE,
    SectionObject: *mut PVOID,
    SectionFileSize: PLARGE_INTEGER,
    FileObject: PFILE_OBJECT,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    MaximumSize: PLARGE_INTEGER,
    SectionPageProtection: ULONG,
    AllocationAttributes: ULONG,
    Flags: ULONG,
) -> NTSTATUS {
    let SectionHandle = opaque_ptr(SectionHandle);
    let SectionObject = opaque_ptr(SectionObject);
    let SectionFileSize = opaque_ptr(SectionFileSize);
    let FileObject = opaque_ptr(FileObject);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let MaximumSize = opaque_ptr(MaximumSize);
    KeBugCheck(ntkrnl_sys::bugcodes::FILE_SYSTEM);
    STATUS_NOT_IMPLEMENTED
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/fsfilter.c
//  * PURPOSE:         Provides support for the Filter Manager
//  * PROGRAMMERS:     None.
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlCreateSectionForDataScan
//  * @unimplemented
//  *
//  * FILLME
//  *
//  * @param SectionHandle
//  *        FILLME
//  *
//  * @param SectionObject
//  *        FILLME
//  *
//  * @param SectionFileSize
//  *        FILLME
//  *
//  * @param FileObject
//  *        FILLME
//  *
//  * @param DesiredAccess
//  *        FILLME
//  *
//  * @param ObjectAttributes
//  *        FILLME
//  *
//  * @param MaximumSize
//  *        FILLME
//  *
//  * @param SectionPageProtection
//  *        FILLME
//  *
//  * @param AllocationAttributes
//  *        FILLME
//  *
//  * @param Flags
//  *        FILLME
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// NTSTATUS
// NTAPI
// FsRtlCreateSectionForDataScan(OUT PHANDLE SectionHandle,
//                               OUT PVOID *SectionObject,
//                               OUT PLARGE_INTEGER SectionFileSize OPTIONAL,
//                               IN PFILE_OBJECT FileObject,
//                               IN ACCESS_MASK DesiredAccess,
//                               IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
//                               IN PLARGE_INTEGER MaximumSize OPTIONAL,
//                               IN ULONG SectionPageProtection,
//                               IN ULONG AllocationAttributes,
//                               IN ULONG Flags)
// {
//     /* Unimplemented */
//     KeBugCheck(FILE_SYSTEM);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
