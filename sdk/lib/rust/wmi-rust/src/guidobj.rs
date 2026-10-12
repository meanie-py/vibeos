use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    ACCESS_MASK, _GENERIC_MAPPING, GUID, HANDLE, KPROCESSOR_MODE, KeInitializeEvent,
    LPCGUID, _OBJECT_ATTRIBUTES, _UNICODE_STRING, OBJECT_TYPE_INITIALIZER,
    ObAssignObjectSecurityDescriptor, ObCreateObject, ObCreateObjectType,
    ObDeassignSecurity, ObOpenObjectByPointer, ObQuerySecurityDescriptorInfo,
    ObSetSecurityDescriptorInfo, PGENERIC_MAPPING, PGUID, PHANDLE,
    POBJECT_ATTRIBUTES, POBJECT_TYPE, PSECURITY_DESCRIPTOR,
    PSECURITY_INFORMATION, PUNICODE_STRING, RtlGUIDFromString,
    RtlPrefixUnicodeString, _EVENT_TYPE_NotificationEvent as _EVENT_TYPE_NotificationEvent,
    SECURITY_OPERATION_CODE, _SECURITY_OPERATION_CODE_QuerySecurityDescriptor as _SECURITY_OPERATION_CODE_QuerySecurityDescriptor,
    _SECURITY_OPERATION_CODE_SetSecurityDescriptor as _SECURITY_OPERATION_CODE_SetSecurityDescriptor,
    _SECURITY_OPERATION_CODE_AssignSecurityDescriptor as _SECURITY_OPERATION_CODE_AssignSecurityDescriptor,
    _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor as _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor,
};

use ntkrnl_sys::ddk::PEPROCESS;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "C" {
    fn InitializeObjectAttributes(
        p: *mut _OBJECT_ATTRIBUTES,
        n: PUNICODE_STRING,
        a: ULONG,
        r: HANDLE,
        s: PSECURITY_DESCRIPTOR,
    );
}

const GUID_STRING_LENGTH: usize = 36;

const WMIGUID_QUERY: ULONG = 0x0001;
const WMIGUID_SET: ULONG = 0x0002;
const WMIGUID_EXECUTE: ULONG = 0x10;
// psdk/wmistr.h; the translation had 0x000F, the low nibble of the 0xFFF (ObTypes.cpp:301).
const WMIGUID_ALL_ACCESS: ULONG = STANDARD_RIGHTS_READ | SYNCHRONIZE | 0xFFF;

#[repr(C)]
pub struct WMIP_GUID_OBJECT {
    pub Event: ntkrnl_sys::ddk::KEVENT,
    pub Guid: GUID,
    pub Irp: PVOID,
    pub IrpLink: ntkrnl_sys::ddk::_LIST_ENTRY,
}

pub type PWMIP_GUID_OBJECT = *mut WMIP_GUID_OBJECT;

#[no_mangle]
pub static mut WmipGuidObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut WmipGenericMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: WMIGUID_QUERY,
    GenericWrite: WMIGUID_SET,
    GenericExecute: WMIGUID_EXECUTE,
    GenericAll: WMIGUID_ALL_ACCESS,
};

#[no_mangle]
pub unsafe extern "system" fn WmipSecurityMethod(
    Object: PVOID,
    OperationType: SECURITY_OPERATION_CODE,
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    CapturedLength: *mut ULONG,
    ObjectSecurityDescriptor: *mut PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
    // upstream-sync 2026-09-20: the security-method prototype grew a trailing
    // KPROCESSOR_MODE AccessMode (ntoskrnl/include/internal/se.h), and ob-rust's
    // ObpCallSecurityProcedure now passes it to whatever sits in SecurityProcedure.
    // On i386 `extern "system"` is stdcall and the CALLEE pops, so an 8-parameter
    // method reached through a 9-argument call leaks four bytes of stack PER CALL --
    // which is how a boot that got all the way to the system-driver phase died with
    // BUGCHECK 0x1E on i386 while amd64 and arm64 were unaffected.
    _AccessMode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let SecurityDescriptor = opaque_ptr(SecurityDescriptor);
    let CapturedLength = opaque_ptr(CapturedLength);
    let ObjectSecurityDescriptor = opaque_ptr(ObjectSecurityDescriptor);
    let GenericMapping = opaque_ptr(GenericMapping);
    if OperationType == _SECURITY_OPERATION_CODE_QuerySecurityDescriptor {
        return ObQuerySecurityDescriptorInfo(
            Object,
            SecurityInformation,
            SecurityDescriptor,
            CapturedLength,
            ObjectSecurityDescriptor,
        );
    } else if OperationType == _SECURITY_OPERATION_CODE_SetSecurityDescriptor {
        return ObSetSecurityDescriptorInfo(
            Object,
            SecurityInformation,
            SecurityDescriptor,
            ObjectSecurityDescriptor,
            PoolType,
            GenericMapping,
        );
    } else if OperationType == _SECURITY_OPERATION_CODE_AssignSecurityDescriptor {
        ObAssignObjectSecurityDescriptor(Object, SecurityDescriptor, PoolType);
        return STATUS_SUCCESS;
    } else if OperationType == _SECURITY_OPERATION_CODE_DeleteSecurityDescriptor {
        return ObDeassignSecurity(ObjectSecurityDescriptor);
    }

    STATUS_INVALID_PARAMETER
}

#[no_mangle]
pub unsafe extern "system" fn WmipDeleteMethod(Object: PVOID) {
    let Object = opaque_ptr(Object);
    let guid_object = Object as PWMIP_GUID_OBJECT;

    if (*guid_object).Irp != core::ptr::null_mut() {
        // This is not supported yet
    }
}

#[no_mangle]
pub unsafe extern "system" fn WmipCloseMethod(
    _Process: PEPROCESS,
    _Object: PVOID,
    _GrantedAccess: ACCESS_MASK,
    _ProcessHandleCount: ULONG,
    _SystemHandleCount: ULONG,
) {
    let _Process = opaque_ptr(_Process);
    let _Object = opaque_ptr(_Object);
    // For now nothing
}

#[no_mangle]
pub unsafe extern "system" fn WmipInitializeGuidObjectType() -> NTSTATUS {
    #[no_mangle]
    pub static mut GUID_OBJECT_NAME: _UNICODE_STRING = _UNICODE_STRING {
        Length: 14,
        MaximumLength: 16,
        Buffer: w!("WmiGuid").as_ptr() as *mut u16,
    };
    let mut object_type_initializer: OBJECT_TYPE_INITIALIZER = core::mem::zeroed();

    object_type_initializer.Length = core::mem::size_of::<OBJECT_TYPE_INITIALIZER>() as u16;
    object_type_initializer.InvalidAttributes = OBJ_OPENLINK;
    object_type_initializer.GenericMapping = WmipGenericMapping;
    object_type_initializer.PoolType = NonPagedPool;
    object_type_initializer.MaintainHandleCount = 0u8;
    object_type_initializer.ValidAccessMask = STANDARD_RIGHTS_ALL | 0xFFF;
    object_type_initializer.SecurityRequired = 1u8;
    object_type_initializer.DefaultNonPagedPoolCharge = core::mem::size_of::<WMIP_GUID_OBJECT>() as ULONG;
    // SAFETY: transmute needed because extern "system" != extern "stdcall"/"C" in Rust's type system
    object_type_initializer.SecurityProcedure = core::mem::transmute(WmipSecurityMethod as usize);
    object_type_initializer.DeleteProcedure = core::mem::transmute(WmipDeleteMethod as usize);
    object_type_initializer.CloseProcedure = core::mem::transmute(WmipCloseMethod as usize);

    let status = ObCreateObjectType(
        &raw mut GUID_OBJECT_NAME,
        &mut object_type_initializer,
        core::ptr::null_mut(),
        &raw mut WmipGuidObjectType,
    );

    status
}

#[no_mangle]
pub unsafe extern "system" fn WmipGUIDFromString(
    GuidString: PUNICODE_STRING,
    Guid: PGUID,
) -> NTSTATUS {
    let GuidString = opaque_ptr(GuidString);
    let mut buffer: [u16; GUID_STRING_LENGTH + 2] = [0; GUID_STRING_LENGTH + 2];
    let mut string: _UNICODE_STRING;

    if (*GuidString).Length != (GUID_STRING_LENGTH * core::mem::size_of::<u16>()) as u16 {
        return STATUS_INVALID_PARAMETER;
    }

    core::ptr::copy_nonoverlapping(
        (*GuidString).Buffer as *const u8,
        buffer.as_mut_ptr().add(1) as *mut u8,
        (*GuidString).Length as usize,
    );
    buffer[0] = b'{' as u16;
    buffer[GUID_STRING_LENGTH + 1] = b'}' as u16;

    string = _UNICODE_STRING {
        Buffer: buffer.as_mut_ptr(),
        Length: core::mem::size_of_val(&buffer) as u16,
        MaximumLength: core::mem::size_of_val(&buffer) as u16,
    };

    RtlGUIDFromString(&mut string, Guid)
}

#[no_mangle]
pub unsafe extern "system" fn WmipCreateGuidObject(
    Guid: *const GUID,
    OutGuidObject: *mut PWMIP_GUID_OBJECT,
) -> NTSTATUS {
    let OutGuidObject = opaque_ptr(OutGuidObject);
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut guid_object: PWMIP_GUID_OBJECT = core::ptr::null_mut();

    InitializeObjectAttributes(
        &mut object_attributes,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let status = ObCreateObject(
        0i8,
        WmipGuidObjectType,
        &mut object_attributes,
        0i8,
        core::ptr::null_mut(),
        core::mem::size_of::<WMIP_GUID_OBJECT>() as ULONG,
        0,
        0,
        &mut guid_object as *mut PWMIP_GUID_OBJECT as *mut PVOID,
    );
    if status < 0 {
        return status;
    }

    core::ptr::write_bytes(guid_object as *mut u8, 0, core::mem::size_of::<WMIP_GUID_OBJECT>());
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*guid_object).Event),
        _EVENT_TYPE_NotificationEvent,
        0u8,
    );
    (*guid_object).Guid = *Guid;

    *OutGuidObject = guid_object;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WmipOpenGuidObject(
    Guid: LPCGUID,
    DesiredAccess: ACCESS_MASK,
    AccessMode: KPROCESSOR_MODE,
    OutGuidObjectHandle: PHANDLE,
    OutGuidObject: *mut PVOID,
) -> NTSTATUS {
    let OutGuidObjectHandle = opaque_ptr(OutGuidObjectHandle);
    let OutGuidObject = opaque_ptr(OutGuidObject);
    let mut guid_object: PWMIP_GUID_OBJECT = core::ptr::null_mut();
    let handle_attributes: ULONG;

    let mut status = WmipCreateGuidObject(Guid, &mut guid_object);
    if status < 0 {
        *OutGuidObject = core::ptr::null_mut();
        return status;
    }

    handle_attributes = if AccessMode == 0i8 { OBJ_KERNEL_HANDLE } else { 0 };

    status = ObOpenObjectByPointer(
        guid_object as PVOID,
        handle_attributes,
        core::ptr::null_mut(),
        DesiredAccess,
        WmipGuidObjectType,
        AccessMode,
        OutGuidObjectHandle,
    );
    if status < 0 {
        ObDereferenceObject(guid_object as PVOID);
        guid_object = core::ptr::null_mut();
    }

    *OutGuidObject = guid_object as PVOID;

    status
}

#[no_mangle]
pub unsafe extern "system" fn WmipOpenGuidObjectByName(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    DesiredAccess: ACCESS_MASK,
    AccessMode: KPROCESSOR_MODE,
    OutGuidObjectHandle: PHANDLE,
    OutGuidObject: *mut PVOID,
) -> NTSTATUS {
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let OutGuidObjectHandle = opaque_ptr(OutGuidObjectHandle);
    let OutGuidObject = opaque_ptr(OutGuidObject);
    #[no_mangle]
    pub static mut PREFIX: _UNICODE_STRING = _UNICODE_STRING {
        Length: 18,
        MaximumLength: 20,
        Buffer: w!("\\WmiGuid\\").as_ptr() as *mut u16,
    };
    let mut guid_string: _UNICODE_STRING;
    let mut guid: GUID = core::mem::zeroed();

    if RtlPrefixUnicodeString(&raw mut PREFIX, (*ObjectAttributes).ObjectName, 0u8) == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    guid_string = *(*ObjectAttributes).ObjectName;
    guid_string.Buffer = guid_string.Buffer.add((PREFIX.Length / core::mem::size_of::<u16>() as u16) as usize);
    guid_string.Length -= PREFIX.Length;

    let status = WmipGUIDFromString(&mut guid_string, &mut guid);
    if status < 0 {
        return status;
    }

    WmipOpenGuidObject(
        &guid,
        DesiredAccess,
        AccessMode,
        OutGuidObjectHandle,
        OutGuidObject,
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/wmi/guidobj.c
//  * PURPOSE:         I/O Windows Management Instrumentation (WMI) Support
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #include <wmistr.h>
// #include "wmip.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS ******************************************************************/
//
// POBJECT_TYPE WmipGuidObjectType;
// GENERIC_MAPPING WmipGenericMapping =
// {
//     WMIGUID_QUERY,
//     WMIGUID_SET,
//     WMIGUID_EXECUTE,
//     WMIGUID_ALL_ACCESS
// };
//
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// WmipSecurityMethod(
//     _In_ PVOID Object,
//     _In_ SECURITY_OPERATION_CODE OperationType,
//     _In_ PSECURITY_INFORMATION SecurityInformation,
//     _In_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _Inout_ PULONG CapturedLength,
//     _Inout_ PSECURITY_DESCRIPTOR *ObjectSecurityDescriptor,
//     _In_ POOL_TYPE PoolType,
//     _In_ PGENERIC_MAPPING GenericMapping)
// {
//     PAGED_CODE();
//
//     ASSERT((PoolType == PagedPool) || (PoolType == NonPagedPool));
//     ASSERT((OperationType == QuerySecurityDescriptor) ||
//            (OperationType == SetSecurityDescriptor) ||
//            (OperationType == AssignSecurityDescriptor) ||
//            (OperationType == DeleteSecurityDescriptor));
//
//     if (OperationType == QuerySecurityDescriptor)
//     {
//         return ObQuerySecurityDescriptorInfo(Object,
//                                              SecurityInformation,
//                                              SecurityDescriptor,
//                                              CapturedLength,
//                                              ObjectSecurityDescriptor);
//     }
//     else if (OperationType == SetSecurityDescriptor)
//     {
//         return ObSetSecurityDescriptorInfo(Object,
//                                              SecurityInformation,
//                                              SecurityDescriptor,
//                                              ObjectSecurityDescriptor,
//                                              PoolType,
//                                              GenericMapping);
//     }
//     else if (OperationType == AssignSecurityDescriptor)
//     {
//         ObAssignObjectSecurityDescriptor(Object, SecurityDescriptor, PoolType);
//         return STATUS_SUCCESS;
//     }
//     else if (OperationType == DeleteSecurityDescriptor)
//     {
//         return ObDeassignSecurity(ObjectSecurityDescriptor);
//     }
//
//     ASSERT(FALSE);
//     return STATUS_INVALID_PARAMETER;
// }
//
// VOID
// NTAPI
// WmipDeleteMethod(
//     _In_ PVOID Object)
// {
//     PWMIP_GUID_OBJECT GuidObject = Object;
//
//     /* Check if the object is attached to an IRP */
//     if (GuidObject->Irp != NULL)
//     {
//         /* This is not supported yet */
//         ASSERT(FALSE);
//     }
// }
//
// VOID
// NTAPI
// WmipCloseMethod(
//     _In_opt_ PEPROCESS Process,
//     _In_ PVOID Object,
//     _In_ ACCESS_MASK GrantedAccess,
//     _In_ ULONG ProcessHandleCount,
//     _In_ ULONG SystemHandleCount)
// {
//     /* For now nothing */
// }
//
// NTSTATUS
// NTAPI
// WmipInitializeGuidObjectType(
//     VOID)
// {
//     static UNICODE_STRING GuidObjectName = RTL_CONSTANT_STRING(L"WmiGuid");
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     NTSTATUS Status;
//
//     /* Setup the object type initializer */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.InvalidAttributes = OBJ_OPENLINK;
//     ObjectTypeInitializer.GenericMapping = WmipGenericMapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.MaintainHandleCount = FALSE;
//     ObjectTypeInitializer.ValidAccessMask = STANDARD_RIGHTS_ALL | 0xFFF;
//     ObjectTypeInitializer.SecurityRequired = TRUE;
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(WMIP_GUID_OBJECT);
//     ObjectTypeInitializer.SecurityProcedure = WmipSecurityMethod;
//     ObjectTypeInitializer.DeleteProcedure = WmipDeleteMethod;
//     ObjectTypeInitializer.CloseProcedure = WmipCloseMethod;
//
//     /* Create the object type */
//     Status = ObCreateObjectType(&GuidObjectName,
//                                 &ObjectTypeInitializer,
//                                 0,
//                                 &WmipGuidObjectType);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ObCreateObjectType failed: 0x%lx\n", Status);
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// WmipGUIDFromString(
//     _In_ PUNICODE_STRING GuidString,
//     _Out_ PGUID Guid)
// {
//     WCHAR Buffer[GUID_STRING_LENGTH + 2];
//     UNICODE_STRING String;
//
//     /* Validate string length */
//     if (GuidString->Length != GUID_STRING_LENGTH * sizeof(WCHAR))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Copy the string and wrap it in {} */
//     RtlCopyMemory(&Buffer[1], GuidString->Buffer, GuidString->Length);
//     Buffer[0] = L'{';
//     Buffer[GUID_STRING_LENGTH + 1] = L'}';
//
//     String.Buffer = Buffer;
//     String.Length = String.MaximumLength = sizeof(Buffer);
//
//     return RtlGUIDFromString(&String, Guid);
// }
//
// static
// NTSTATUS
// WmipCreateGuidObject(
//     _In_ const GUID *Guid,
//     _Out_ PWMIP_GUID_OBJECT *OutGuidObject)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PWMIP_GUID_OBJECT GuidObject;
//     NTSTATUS Status;
//
//     /* Initialize object attributes for an unnamed object */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                NULL,
//                                0,
//                                NULL,
//                                NULL); // FIXME: security descriptor!
//
//     /* Create the GUID object */
//     Status = ObCreateObject(KernelMode,
//                             WmipGuidObjectType,
//                             &ObjectAttributes,
//                             KernelMode,
//                             NULL,
//                             sizeof(WMIP_GUID_OBJECT),
//                             0,
//                             0,
//                             (PVOID*)&GuidObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("WMI: failed to create GUID object: 0x%lx\n", Status);
//         return Status;
//     }
//
//     RtlZeroMemory(GuidObject, sizeof(*GuidObject));
//     KeInitializeEvent(&GuidObject->Event, NotificationEvent, FALSE);
//     GuidObject->Guid = *Guid;
//
//     *OutGuidObject = GuidObject;
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// WmipOpenGuidObject(
//     _In_ LPCGUID Guid,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _Out_ PHANDLE OutGuidObjectHandle,
//     _Outptr_ PVOID *OutGuidObject)
// {
//     PWMIP_GUID_OBJECT GuidObject;
//     ULONG HandleAttributes;
//     NTSTATUS Status;
//
//     /* Create the GUID object */
//     Status = WmipCreateGuidObject(Guid, &GuidObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create GUID object: 0x%lx\n", Status);
//         *OutGuidObject = NULL;
//         return Status;
//     }
//
//     /* Set handle attributes */
//     HandleAttributes = (AccessMode == KernelMode) ? OBJ_KERNEL_HANDLE : 0;
//
//     /* Get a handle for the object */
//     Status = ObOpenObjectByPointer(GuidObject,
//                                    HandleAttributes,
//                                    0,
//                                    DesiredAccess,
//                                    WmipGuidObjectType,
//                                    AccessMode,
//                                    OutGuidObjectHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ObOpenObjectByPointer failed: 0x%lx\n", Status);
//         ObDereferenceObject(GuidObject);
//         GuidObject = NULL;
//     }
//
//     *OutGuidObject = GuidObject;
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// WmipOpenGuidObjectByName(
//     _In_ POBJECT_ATTRIBUTES ObjectAttributes,
//     _In_ ACCESS_MASK DesiredAccess,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _Out_ PHANDLE OutGuidObjectHandle,
//     _Outptr_ PVOID *OutGuidObject)
// {
//     static UNICODE_STRING Prefix = RTL_CONSTANT_STRING(L"\\WmiGuid\\");
//     UNICODE_STRING GuidString;
//     NTSTATUS Status;
//     GUID Guid;
//     PAGED_CODE();
//
//     /* Check if we have the expected prefix */
//     if (!RtlPrefixUnicodeString(&Prefix, ObjectAttributes->ObjectName, FALSE))
//     {
//         DPRINT1("WMI: Invalid prefix for guid object '%wZ'\n",
//                 ObjectAttributes->ObjectName);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Extract the GUID string */
//     GuidString = *ObjectAttributes->ObjectName;
//     GuidString.Buffer += Prefix.Length / sizeof(WCHAR);
//     GuidString.Length -= Prefix.Length;
//
//     /* Convert the string into a GUID structure */
//     Status = WmipGUIDFromString(&GuidString, &Guid);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("WMI: Invalid uuid format for guid '%wZ'\n", &GuidString);
//         return Status;
//     }
//
//     return WmipOpenGuidObject(&Guid,
//                               DesiredAccess,
//                               AccessMode,
//                               OutGuidObjectHandle,
//                               OutGuidObject);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<WMIP_GUID_OBJECT>() == 64);
    assert!(core::mem::offset_of!(WMIP_GUID_OBJECT, Event) == 0);
    assert!(core::mem::offset_of!(WMIP_GUID_OBJECT, Guid) == 24);
    assert!(core::mem::offset_of!(WMIP_GUID_OBJECT, Irp) == 40);
    assert!(core::mem::offset_of!(WMIP_GUID_OBJECT, IrpLink) == 48);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<WMIP_GUID_OBJECT>() == 44);
    assert!(core::mem::offset_of!(WMIP_GUID_OBJECT, Event) == 0);
    assert!(core::mem::offset_of!(WMIP_GUID_OBJECT, Guid) == 16);
    assert!(core::mem::offset_of!(WMIP_GUID_OBJECT, Irp) == 32);
    assert!(core::mem::offset_of!(WMIP_GUID_OBJECT, IrpLink) == 36);
};
// END OFFSET CHECKS
