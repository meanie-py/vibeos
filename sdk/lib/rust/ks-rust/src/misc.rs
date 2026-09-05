use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, IoGetCurrentIrpStackLocation, LPWSTR, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _KSIDENTIFIER {
    pub Anonymous: _KSIDENTIFIER__bindgen_ty_1,
}

#[repr(C)]
pub union _KSIDENTIFIER__bindgen_ty_1 {
    pub Value: ULONG,
    pub Anonymous: _KSIDENTIFIER__bindgen_ty_1__bindgen_ty_1,
    pub Alignment: [LONGLONG; 2usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _KSIDENTIFIER__bindgen_ty_1__bindgen_ty_1 {
    pub Set: GUID,
    pub Id: ULONG,
    pub Flags: ULONG,
}

pub type KSOBJECTTYPE = ULONG;

pub const KsObjectTypeDevice: KSOBJECTTYPE = 0;
pub const KsObjectTypeFilterFactory: KSOBJECTTYPE = 1;
pub const KsObjectTypePinFactory: KSOBJECTTYPE = 2;
pub const KsObjectTypePin: KSOBJECTTYPE = 3;
pub const KsObjectTypeTopology: KSOBJECTTYPE = 4;
pub const KsObjectTypeClock: KSOBJECTTYPE = 5;
pub const KsObjectTypeAllocator: KSOBJECTTYPE = 6;
pub const KsObjectTypeFdo: KSOBJECTTYPE = 7;
pub const KsObjectTypeFilter: KSOBJECTTYPE = 8;

pub const TAG_KS: u32 = 0x53534B4B; // 'ssKK'
#[no_mangle]
pub unsafe extern "C" fn CompleteRequest(
    Irp: PIRP,
    PriorityBoost: CCHAR,
) {
    if (*Irp).IoStatus.__bindgen_anon_1.Status != STATUS_PENDING {
        IoCompleteRequest(Irp, PriorityBoost);
    }
}
#[no_mangle]
pub unsafe extern "C" fn AllocateItem(
    PoolType: POOL_TYPE,
    NumberOfBytes: usize,
) -> *mut core::ffi::c_void {
    // ExAllocatePoolZero is not exported by this ntoskrnl; use ExAllocatePoolWithTag + zero
    // (the zeroed-pool semantics the KS AllocateItem relies on).
    let p = ExAllocatePoolWithTag(PoolType, NumberOfBytes as _, TAG_KS);
    if !p.is_null() {
        core::ptr::write_bytes(p as *mut u8, 0, NumberOfBytes);
    }
    p
}
#[no_mangle]
pub unsafe extern "C" fn FreeItem(Item: PVOID) {
    ExFreePoolWithTag(Item, TAG_KS);
}
#[no_mangle]
pub unsafe extern "C" fn KspCopyCreateRequest(
    Irp: PIRP,
    ObjectClass: LPWSTR,
    Size: *mut u32,
    Result: *mut *mut core::ffi::c_void,
) -> i32 {
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let ObjectLength: usize = (wcslen(ObjectClass as *const u16) + 1) * 2;
    if ObjectLength + *Size as usize > (*((*IoStack).FileObject)).FileName.MaximumLength as usize {
        return STATUS_UNSUCCESSFUL;
    }
    let ParametersLength: usize = (*((*IoStack).FileObject)).FileName.MaximumLength as usize - ObjectLength;
    let Buffer: *mut core::ffi::c_void = AllocateItem(NonPagedPool, ParametersLength);
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::copy_nonoverlapping(
        (*((*IoStack).FileObject)).FileName.Buffer.add(ObjectLength / 2) as *const core::ffi::c_void,
        Buffer,
        ParametersLength,
    );
    *Result = Buffer;
    *Size = ParametersLength as u32;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsGetObjectFromFileObject(
    FileObject: PFILE_OBJECT,
) -> PVOID {
    let ObjectHeader: PKSIOBJECT_HEADER = (*FileObject).FsContext2 as PKSIOBJECT_HEADER;
    return (*ObjectHeader).ObjectType;
}

#[no_mangle]
pub unsafe extern "system" fn KsGetObjectTypeFromFileObject(
    FileObject: PFILE_OBJECT,
) -> KSOBJECTTYPE {
    let ObjectHeader: PKSIOBJECT_HEADER = (*FileObject).FsContext2 as PKSIOBJECT_HEADER;
    return (*ObjectHeader).Type;
}

#[no_mangle]
pub unsafe extern "system" fn KsGetObjectTypeFromIrp(
    Irp: PIRP,
) -> KSOBJECTTYPE {
    let ObjectHeader: PKSIOBJECT_HEADER;
    let IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    ObjectHeader = (*(*IoStack).FileObject).FsContext2 as PKSIOBJECT_HEADER;
    return (*ObjectHeader).Type;
}

#[no_mangle]
pub unsafe extern "system" fn KsGetOuterUnknown(
    Object: PVOID,
) -> PUNKNOWN {
    let BasicHeader: PKSBASIC_HEADER = (Object as usize)
        .wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>())
        as PKSBASIC_HEADER;

    let type_val = (*BasicHeader).Type as u32;
    ntkrnl_sys::ASSERT!(type_val == KsObjectTypeDevice as u32
            || type_val == KsObjectTypeFilterFactory as u32
            || type_val == KsObjectTypeFilter as u32
            || type_val == KsObjectTypePin as u32);

    (*BasicHeader).OuterUnknown
}

#[no_mangle]
pub unsafe extern "system" fn KsGetParent(Object: PVOID) -> PVOID {
    let BasicHeader: PKSBASIC_HEADER = (Object as usize)
        .wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>())
        as PKSBASIC_HEADER;
    (*BasicHeader).Parent.KsDevice as PVOID
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/misc.c
//  * PURPOSE:         KS Allocator functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define TAG_KS 'ssKK'
//
// VOID
// CompleteRequest(
//     PIRP Irp,
//     CCHAR PriorityBoost)
// {
//     DPRINT("Completing IRP %p Status %x\n", Irp, Irp->IoStatus.Status);
//
//     ASSERT(Irp->IoStatus.Status != STATUS_PENDING);
//
//     IoCompleteRequest(Irp, PriorityBoost);
// }
//
// PVOID
// AllocateItem(
//     IN POOL_TYPE PoolType,
//     IN SIZE_T NumberOfBytes)
// {
//     return ExAllocatePoolZero(PoolType, NumberOfBytes, TAG_KS);
// }
//
// VOID
// FreeItem(
//     IN PVOID Item)
// {
//     ExFreePoolWithTag(Item, TAG_KS);
// }
//
// NTSTATUS
// KspCopyCreateRequest(
//     IN PIRP Irp,
//     IN LPWSTR ObjectClass,
//     IN OUT PULONG Size,
//     OUT PVOID * Result)
// {
//     PIO_STACK_LOCATION IoStack;
//     SIZE_T ObjectLength, ParametersLength;
//     PVOID Buffer;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get object class length */
//     ObjectLength = (wcslen(ObjectClass) + 1) * sizeof(WCHAR);
//
//     /* check for minium length requirement */
//     if (ObjectLength  + *Size > IoStack->FileObject->FileName.MaximumLength)
//         return STATUS_UNSUCCESSFUL;
//
//     /* extract parameters length */
//     ParametersLength = IoStack->FileObject->FileName.MaximumLength - ObjectLength;
//
//     /* allocate buffer */
//     Buffer = AllocateItem(NonPagedPool, ParametersLength);
//     if (!Buffer)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* copy parameters */
//     RtlMoveMemory(Buffer, &IoStack->FileObject->FileName.Buffer[ObjectLength / sizeof(WCHAR)], ParametersLength);
//
//     /* store result */
//     *Result = Buffer;
//     *Size = (ULONG)ParametersLength;
//
//     return STATUS_SUCCESS;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// PVOID
// NTAPI
// KsGetObjectFromFileObject(
//     IN PFILE_OBJECT FileObject)
// {
//     PKSIOBJECT_HEADER ObjectHeader;
//
//     /* get object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)FileObject->FsContext2;
//
//     /* return associated object */
//     return ObjectHeader->ObjectType;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// KSOBJECTTYPE
// NTAPI
// KsGetObjectTypeFromFileObject(
//     IN PFILE_OBJECT FileObject)
// {
//     PKSIOBJECT_HEADER ObjectHeader;
//
//     /* get object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)FileObject->FsContext2;
//     /* return type */
//     return ObjectHeader->Type;
// }
//
// /*
//     @implemented
// */
// KSOBJECTTYPE
// NTAPI
// KsGetObjectTypeFromIrp(
//     IN PIRP  Irp)
// {
//     PKSIOBJECT_HEADER ObjectHeader;
//     PIO_STACK_LOCATION IoStack;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     /* get object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//     /* return type */
//     return ObjectHeader->Type;
// }
//
// /*
//     @implemented
// */
// PUNKNOWN
// NTAPI
// KsGetOuterUnknown(
//     IN PVOID  Object)
// {
//     PKSBASIC_HEADER BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)Object - sizeof(KSBASIC_HEADER));
//
//     /* sanity check */
//     ASSERT(BasicHeader->Type == KsObjectTypeDevice || BasicHeader->Type == KsObjectTypeFilterFactory ||
//            BasicHeader->Type == KsObjectTypeFilter || BasicHeader->Type == KsObjectTypePin);
//
//     /* return objects outer unknown */
//     return BasicHeader->OuterUnknown;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// PVOID
// NTAPI
// KsGetParent(
//     IN PVOID Object)
// {
//     PKSBASIC_HEADER BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)Object - sizeof(KSBASIC_HEADER));
//     /* sanity check */
//     ASSERT(BasicHeader->Parent.KsDevice != NULL);
//     /* return object type */
//     return (PVOID)BasicHeader->Parent.KsDevice;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
