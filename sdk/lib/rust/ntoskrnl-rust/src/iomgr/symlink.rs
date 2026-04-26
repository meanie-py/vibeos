use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;

use ntkrnl_sys::ddk::{
    _OBJECT_ATTRIBUTES,
    PUNICODE_STRING,
    _SECURITY_DESCRIPTOR,
    HANDLE, PSECURITY_DESCRIPTOR, PACL,
    ZwClose,
    ZwCreateSymbolicLinkObject,
    ZwMakeTemporaryObject,
    ZwOpenSymbolicLinkObject,
    RtlCreateSecurityDescriptor,
    RtlSetDaclSecurityDescriptor,
};

use ntkrnl_sys::inline::common::InitializeObjectAttributes;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static mut SePublicDefaultSd: *mut core::ffi::c_void;
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateSymbolicLink(
    SymbolicLinkName: PUNICODE_STRING,
    DeviceName: PUNICODE_STRING,
) -> NTSTATUS {
    let SymbolicLinkName = opaque_ptr(SymbolicLinkName);
    let DeviceName = opaque_ptr(DeviceName);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        SymbolicLinkName,
        OBJ_KERNEL_HANDLE | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        SePublicDefaultSd,
    );

    let Status = ZwCreateSymbolicLinkObject(
        &mut Handle,
        SYMBOLIC_LINK_ALL_ACCESS,
        &mut ObjectAttributes,
        DeviceName,
    );

    if Status >= 0 {
        ZwClose(Handle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoCreateUnprotectedSymbolicLink(
    SymbolicLinkName: PUNICODE_STRING,
    DeviceName: PUNICODE_STRING,
) -> NTSTATUS {
    let SymbolicLinkName = opaque_ptr(SymbolicLinkName);
    let DeviceName = opaque_ptr(DeviceName);
    let mut SecurityDescriptor: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();

    let mut Status = RtlCreateSecurityDescriptor(
        &mut SecurityDescriptor as *mut _SECURITY_DESCRIPTOR as PSECURITY_DESCRIPTOR,
        SECURITY_DESCRIPTOR_REVISION,
    );
    if Status < 0 {
        return Status;
    }

    Status = RtlSetDaclSecurityDescriptor(
        &mut SecurityDescriptor as *mut _SECURITY_DESCRIPTOR as PSECURITY_DESCRIPTOR,
        1u8,
        core::ptr::null_mut(),
        1u8,
    );
    if Status < 0 {
        return Status;
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        SymbolicLinkName,
        OBJ_KERNEL_HANDLE | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        &mut SecurityDescriptor as *mut _SECURITY_DESCRIPTOR as *mut core::ffi::c_void,
    );

    Status = ZwCreateSymbolicLinkObject(
        &mut Handle,
        SYMBOLIC_LINK_ALL_ACCESS,
        &mut ObjectAttributes,
        DeviceName,
    );

    if Status >= 0 {
        ZwClose(Handle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IoDeleteSymbolicLink(
    SymbolicLinkName: PUNICODE_STRING,
) -> NTSTATUS {
    let SymbolicLinkName = opaque_ptr(SymbolicLinkName);
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Handle: HANDLE = core::ptr::null_mut();

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        SymbolicLinkName,
        OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let mut Status = ZwOpenSymbolicLinkObject(
        &mut Handle,
        DELETE,
        &mut ObjectAttributes,
    );
    if Status < 0 {
        return Status;
    }

    Status = ZwMakeTemporaryObject(Handle);
    if Status >= 0 {
        ZwClose(Handle);
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/symlink.c
//  * PURPOSE:         I/O Wrappers for Symbolic Links
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ****************************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoCreateSymbolicLink(IN PUNICODE_STRING SymbolicLinkName,
//                      IN PUNICODE_STRING DeviceName)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE Handle;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Initialize the object attributes and create the link */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                SymbolicLinkName,
//                                OBJ_KERNEL_HANDLE | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                SePublicDefaultSd);
//     Status = ZwCreateSymbolicLinkObject(&Handle,
//                                         SYMBOLIC_LINK_ALL_ACCESS,
//                                         &ObjectAttributes,
//                                         DeviceName);
//     if (NT_SUCCESS(Status)) ZwClose(Handle);
//
//     /* Return status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoCreateUnprotectedSymbolicLink(IN PUNICODE_STRING SymbolicLinkName,
//                                 IN PUNICODE_STRING DeviceName)
// {
//     SECURITY_DESCRIPTOR SecurityDescriptor;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE Handle;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Create an SD */
//     Status = RtlCreateSecurityDescriptor(&SecurityDescriptor,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Set the DACL */
//     Status = RtlSetDaclSecurityDescriptor(&SecurityDescriptor,
//                                           TRUE,
//                                           NULL,
//                                           TRUE);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Initialize the object attributes and create the link */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                SymbolicLinkName,
//                                OBJ_KERNEL_HANDLE | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                &SecurityDescriptor);
//     Status = ZwCreateSymbolicLinkObject(&Handle,
//                                         SYMBOLIC_LINK_ALL_ACCESS,
//                                         &ObjectAttributes,
//                                         DeviceName);
//     if (NT_SUCCESS(Status)) ZwClose(Handle);
//
//     /* Return status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoDeleteSymbolicLink(IN PUNICODE_STRING SymbolicLinkName)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE Handle;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Initialize the object attributes and open the link */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                SymbolicLinkName,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenSymbolicLinkObject(&Handle, DELETE, &ObjectAttributes);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Make the link temporary and close its handle */
//     Status = ZwMakeTemporaryObject(Handle);
//     if (NT_SUCCESS(Status)) ZwClose(Handle);
//
//     /* Return status */
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
