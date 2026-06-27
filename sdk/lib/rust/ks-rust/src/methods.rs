use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ExGetPreviousMode, GUID, IoGetCurrentIrpStackLocation, KPROCESSOR_MODE, LPGUID, PFILE_OBJECT, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _KSMETHOD {
    pub Set: GUID,
    pub Id: u32,
    pub Flags: u32,
}

pub type PKSMETHOD = *mut _KSMETHOD;

pub const NDEBUG: u32 = 0;

pub type PFAST_METHOD_HANDLER = Option<unsafe extern "system" fn(
    FileObject: PFILE_OBJECT,
    MethodRequest: PKSMETHOD,
    MethodLength: u32,
    Data: *mut core::ffi::c_void,
    DataLength: u32,
    IoStatus: PIO_STATUS_BLOCK,
) -> BOOLEAN>;

#[repr(C)]
pub struct _KSMETHOD_SET {
    pub Set: GUID,
    pub MethodsCount: u32,
    pub Method: *mut _KSMETHOD,
    pub FastIoCount: u32,
    pub FastIoTable: *mut PFAST_METHOD_HANDLER,
}

pub type PKSMETHOD_SET = *mut _KSMETHOD_SET;
#[no_mangle]
pub unsafe extern "C" fn FindFastMethodHandler(
    FastIoCount: u32,
    FastIoTable: *mut PFAST_METHOD_HANDLER,
    MethodRequest: PKSMETHOD,
    FastMethodHandler: *mut PFAST_METHOD_HANDLER,
) -> NTSTATUS {
    let mut Index: u32 = 0;
    while Index < FastIoCount {
        if let Some(handler) = *FastIoTable.add(Index as usize) {
            *FastMethodHandler = Some(handler);
            return STATUS_SUCCESS;
        }
        Index += 1;
    }
    STATUS_NOT_FOUND
}```
#[no_mangle]
pub unsafe extern "C" fn FindMethodHandler(
    IoStatus: PIO_STATUS_BLOCK,
    MethodSet: *const KSMETHOD_SET,
    MethodSetCount: ULONG,
    Method: *const KSMETHOD,
    InputBufferLength: ULONG,
    OutputBufferLength: ULONG,
    OutputBuffer: *mut core::ffi::c_void,
    MethodHandler: *mut PFNKSHANDLER,
    Set: *mut *mut KSMETHOD_SET,
) -> NTSTATUS {
    let mut Index: ULONG;
    let mut ItemIndex: ULONG;

    Index = 0;
    while Index < MethodSetCount {
        if IsEqualGUIDAligned(&(*Method).Set, (*MethodSet.offset(Index as isize)).Set) != 0 {
            ItemIndex = 0;
            while ItemIndex < (*MethodSet.offset(Index as isize)).MethodsCount {
                if (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).MethodId == (*Method).Id {
                    if (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).MinMethod > InputBufferLength {
                        (*IoStatus).Information = (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).MinMethod;
                        return STATUS_INVALID_PARAMETER;
                    }

                    if (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).MinData > OutputBufferLength {
                        (*IoStatus).Information = (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).MinData;
                        return STATUS_MORE_ENTRIES;
                    }

                    if (*Method).Flags & KSMETHOD_TYPE_BASICSUPPORT != 0 {
                        let Flags: *mut ULONG;
                        let Description: *mut KSPROPERTY_DESCRIPTION;

                        if core::mem::size_of::<ULONG>() > OutputBufferLength {
                            return STATUS_INVALID_PARAMETER;
                        }

                        Flags = OutputBuffer as *mut ULONG;
                        *Flags = (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).Flags;
                        (*IoStatus).Information = core::mem::size_of::<ULONG>() as ULONG;

                        if OutputBufferLength >= core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as ULONG {
                            Description = OutputBuffer as *mut KSPROPERTY_DESCRIPTION;
                            (*Description).DescriptionSize = core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as ULONG;
                            (*Description).PropTypeSet.Set = KSPROPTYPESETID_General;
                            (*Description).PropTypeSet.Id = 0;
                            (*Description).PropTypeSet.Flags = 0;
                            (*Description).MembersListCount = 0;
                            (*Description).Reserved = 0;
                            (*IoStatus).Information = core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as ULONG;
                        }
                        return STATUS_SUCCESS;
                    }

                    *MethodHandler = (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).MethodHandler;
                    *Set = &mut (*MethodSet.offset(Index as isize)) as *mut KSMETHOD_SET;
                    return STATUS_SUCCESS;
                }
                ItemIndex += 1;
            }
        }
        Index += 1;
    }
    STATUS_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn KspMethodHandlerWithAllocator(
    Irp: PIRP,
    MethodSetsCount: ULONG,
    MethodSet: *const KSMETHOD_SET,
    Allocator: PFNKSALLOCATOR,
    MethodItemSize: ULONG,
) -> NTSTATUS {
    let Method: PKSMETHOD;
    let Set: *mut KSMETHOD_SET;
    let IoStack: PIO_STACK_LOCATION;
    let Status: NTSTATUS;
    let MethodHandler: PFNKSHANDLER = core::ptr::null_mut();
    let Index: ULONG;
    let Guid: LPGUID;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<KSMETHOD>() as ULONG {
        (*Irp).IoStatus.Information = core::mem::size_of::<KSPROPERTY>() as ULONG_PTR;
        return STATUS_INVALID_BUFFER_SIZE;
    }

    Method = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSMETHOD;

    Status = FindMethodHandler(
        &mut (*Irp).IoStatus,
        MethodSet,
        MethodSetsCount,
        Method,
        (*IoStack).Parameters.DeviceIoControl.InputBufferLength,
        (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
        (*Irp).UserBuffer,
        &MethodHandler as *const PFNKSHANDLER as *mut PFNKSHANDLER,
        &mut Set,
    );

    if NT_SUCCESS(Status) && !MethodHandler.is_null() {
        KSMETHOD_SET_IRP_STORAGE(Irp) = Set;
        Status = MethodHandler(Irp, Method, (*Irp).UserBuffer);

        if Status == STATUS_BUFFER_TOO_SMALL {
            if !Allocator.is_null() {
                Status = Allocator(Irp, (*Irp).IoStatus.Information as ULONG, 0u8);
                if !NT_SUCCESS(Status) {
                    return STATUS_INSUFFICIENT_RESOURCES;
                }
                Status = MethodHandler(Irp, Method, (*Irp).UserBuffer);
            }
        }
    } else if IsEqualGUIDAligned(&(*Method).Set as *const GUID, &GUID_NULL as *const GUID) != 0
        && (*Method).Id == 0
        && (*Method).Flags == KSMETHOD_TYPE_SETSUPPORT
    {
        (*Irp).IoStatus.Information = (core::mem::size_of::<GUID>() as ULONG) * MethodSetsCount;
        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < (core::mem::size_of::<GUID>() as ULONG) * MethodSetsCount {
            return STATUS_MORE_ENTRIES;
        }

        Guid = (*Irp).UserBuffer as LPGUID;
        Index = 0;
        while Index < MethodSetsCount {
            core::ptr::copy_nonoverlapping(
                &(*MethodSet.add(Index as usize)).Set as *const GUID,
                &mut (*Guid.add(Index as usize)) as *mut GUID,
                1,
            );
            Index += 1;
        }
        return STATUS_SUCCESS;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsMethodHandler(
    Irp: PIRP,
    MethodSetsCount: ULONG,
    MethodSet: *const KSMETHOD_SET,
) -> NTSTATUS {
    return KspMethodHandlerWithAllocator(Irp, MethodSetsCount, MethodSet, core::ptr::null_mut(), 0);
}

#[no_mangle]
pub unsafe extern "system" fn KsMethodHandlerWithAllocator(
    Irp: PIRP,
    MethodSetsCount: ULONG,
    MethodSet: *const KSMETHOD_SET,
    Allocator: PFNKSALLOCATOR,
    MethodItemSize: ULONG,
) -> NTSTATUS {
    return KspMethodHandlerWithAllocator(Irp, MethodSetsCount, MethodSet, Allocator, MethodItemSize);
}

#[no_mangle]
pub unsafe extern "system" fn KsFastMethodHandler(
    FileObject: PFILE_OBJECT,
    Method: *const KSMETHOD,
    MethodLength: u32,
    Data: *mut core::ffi::c_void,
    DataLength: u32,
    IoStatus: *mut _IO_STATUS_BLOCK,
    MethodSetsCount: u32,
    MethodSet: *const KSMETHOD_SET,
) -> u8 {
    let mut MethodRequest: KSMETHOD = core::mem::zeroed();
    let Mode: KPROCESSOR_MODE;
    let mut Status: i32 = 0i32;
    let mut Index: u32;
    let mut FastMethodHandler: PFNKSFASTHANDLER;

    if MethodLength < core::mem::size_of::<KSPROPERTY>() as u32 {
        return 0u8;
    }

    Mode = ExGetPreviousMode();

    if Mode == 0i32 {
        core::ptr::copy_nonoverlapping(
            Method as *const u8,
            &mut MethodRequest as *mut KSMETHOD as *mut u8,
            core::mem::size_of::<KSMETHOD>(),
        );
    } else {
        // SEH2_TRY body executed directly
        ProbeForRead(Method as *const core::ffi::c_void, core::mem::size_of::<KSPROPERTY>() as u32, core::mem::size_of::<u8>() as u32);
        core::ptr::copy_nonoverlapping(
            Method as *const u8,
            &mut MethodRequest as *mut KSMETHOD as *mut u8,
            core::mem::size_of::<KSMETHOD>(),
        );
        // SEH2_EXCEPT handler omitted per rules (dead code)

        if !NT_SUCCESS(Status) {
            return 0u8;
        }
    }


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/methods.c
//  * PURPOSE:         KS Allocator functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// NTSTATUS
// FindMethodHandler(
//     IN PIO_STATUS_BLOCK IoStatus,
//     IN  const KSMETHOD_SET* MethodSet,
//     IN ULONG MethodSetCount,
//     IN PKSMETHOD Method,
//     IN ULONG InputBufferLength,
//     IN ULONG OutputBufferLength,
//     OUT PVOID OutputBuffer,
//     OUT PFNKSHANDLER *MethodHandler,
//     OUT PKSMETHOD_SET * Set)
// {
//     ULONG Index, ItemIndex;
//
//     for(Index = 0; Index < MethodSetCount; Index++)
//     {
//         ASSERT(MethodSet[Index].Set);
//
//         if (IsEqualGUIDAligned(&Method->Set, MethodSet[Index].Set))
//         {
//             for(ItemIndex = 0; ItemIndex < MethodSet[Index].MethodsCount; ItemIndex++)
//             {
//                 if (MethodSet[Index].MethodItem[ItemIndex].MethodId == Method->Id)
//                 {
//                     if (MethodSet[Index].MethodItem[ItemIndex].MinMethod > InputBufferLength)
//                     {
//                         /* too small input buffer */
//                         IoStatus->Information = MethodSet[Index].MethodItem[ItemIndex].MinMethod;
//                         return STATUS_INVALID_PARAMETER;
//                     }
//
//                     if (MethodSet[Index].MethodItem[ItemIndex].MinData > OutputBufferLength)
//                     {
//                         /* too small output buffer */
//                         IoStatus->Information = MethodSet[Index].MethodItem[ItemIndex].MinData;
//                         return STATUS_MORE_ENTRIES;
//                     }
//                     if (Method->Flags & KSMETHOD_TYPE_BASICSUPPORT)
//                     {
//                         PULONG Flags;
//                         PKSPROPERTY_DESCRIPTION Description;
//
//                         if (sizeof(ULONG) > OutputBufferLength)
//                         {
//                             /* too small buffer */
//                             return STATUS_INVALID_PARAMETER;
//                         }
//
//                         /* get output buffer */
//                         Flags = (PULONG)OutputBuffer;
//
//                         /* set flags flags */
//                         *Flags = MethodSet[Index].MethodItem[ItemIndex].Flags;
//
//                         IoStatus->Information = sizeof(ULONG);
//
//                         if (OutputBufferLength >= sizeof(KSPROPERTY_DESCRIPTION))
//                         {
//                             /* get output buffer */
//                             Description = (PKSPROPERTY_DESCRIPTION)OutputBuffer;
//
//                             /* store result */
//                             Description->DescriptionSize = sizeof(KSPROPERTY_DESCRIPTION);
//                             Description->PropTypeSet.Set = KSPROPTYPESETID_General;
//                             Description->PropTypeSet.Id = 0;
//                             Description->PropTypeSet.Flags = 0;
//                             Description->MembersListCount = 0;
//                             Description->Reserved = 0;
//
//                             IoStatus->Information = sizeof(KSPROPERTY_DESCRIPTION);
//                         }
//                         return STATUS_SUCCESS;
//                     }
//                     *MethodHandler = MethodSet[Index].MethodItem[ItemIndex].MethodHandler;
//                     *Set = (PKSMETHOD_SET)&MethodSet[Index];
//                     return STATUS_SUCCESS;
//                 }
//             }
//         }
//     }
//     return STATUS_NOT_FOUND;
// }
//
// NTSTATUS
// NTAPI
// KspMethodHandlerWithAllocator(
//     IN  PIRP Irp,
//     IN  ULONG MethodSetsCount,
//     IN  const KSMETHOD_SET *MethodSet,
//     IN  PFNKSALLOCATOR Allocator OPTIONAL,
//     IN  ULONG MethodItemSize OPTIONAL)
// {
//     PKSMETHOD Method;
//     PKSMETHOD_SET Set;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     PFNKSHANDLER MethodHandler = NULL;
//     ULONG Index;
//     LPGUID Guid;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* check if inputbuffer at least holds KSMETHOD item */
//     if (IoStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(KSMETHOD))
//     {
//         /* invalid parameter */
//         Irp->IoStatus.Information = sizeof(KSPROPERTY);
//         return STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     /* FIXME probe the input / output buffer if from user mode */
//
//     /* get input property request */
//     Method = (PKSMETHOD)IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//
// //    DPRINT("KspMethodHandlerWithAllocator Irp %p PropertySetsCount %u PropertySet %p Allocator %p PropertyItemSize %u ExpectedPropertyItemSize %u\n", Irp, PropertySetsCount, PropertySet, Allocator, PropertyItemSize, sizeof(KSPROPERTY_ITEM));
//
//     /* sanity check */
//     ASSERT(MethodItemSize == 0 || MethodItemSize == sizeof(KSMETHOD_ITEM));
//
//     /* find the method handler */
//     Status = FindMethodHandler(&Irp->IoStatus, MethodSet, MethodSetsCount, Method, IoStack->Parameters.DeviceIoControl.InputBufferLength, IoStack->Parameters.DeviceIoControl.OutputBufferLength, Irp->UserBuffer, &MethodHandler, &Set);
//
//     if (NT_SUCCESS(Status) && MethodHandler)
//     {
//         /* call method handler */
//         KSMETHOD_SET_IRP_STORAGE(Irp) = Set;
//         Status = MethodHandler(Irp, Method, Irp->UserBuffer);
//
//         if (Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             /* output buffer is too small */
//             if (Allocator)
//             {
//                 /* allocate the requested amount */
//                 Status = Allocator(Irp, (ULONG)Irp->IoStatus.Information, FALSE);
//
//                 /* check if the block was allocated */
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* no memory */
//                     return STATUS_INSUFFICIENT_RESOURCES;
//                 }
//
//                 /* re-call method handler */
//                 Status = MethodHandler(Irp, Method, Irp->UserBuffer);
//             }
//         }
//     }
//     else if (IsEqualGUIDAligned(&Method->Set, &GUID_NULL) && Method->Id == 0 && Method->Flags == KSMETHOD_TYPE_SETSUPPORT)
//     {
//         // store output size
//         Irp->IoStatus.Information = sizeof(GUID) * MethodSetsCount;
//         if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(GUID) * MethodSetsCount)
//         {
//             // buffer too small
//             return STATUS_MORE_ENTRIES;
//         }
//
//         // get output buffer
//         Guid = (LPGUID)Irp->UserBuffer;
//
//        // copy property guids from property sets
//        for(Index = 0; Index < MethodSetsCount; Index++)
//        {
//            RtlMoveMemory(&Guid[Index], MethodSet[Index].Set, sizeof(GUID));
//        }
//        return STATUS_SUCCESS;
//     }
//
//     /* done */
//     return Status;
// }
//
// /*
//     @implemented
// */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsMethodHandler(
//      PIRP Irp,
//      ULONG MethodSetsCount,
//      const KSMETHOD_SET* MethodSet)
// {
//     return KspMethodHandlerWithAllocator(Irp, MethodSetsCount, MethodSet, NULL, 0);
// }
//
// /*
//     @implemented
// */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsMethodHandlerWithAllocator(
//      PIRP Irp,
//      ULONG MethodSetsCount,
//      const KSMETHOD_SET* MethodSet,
//      PFNKSALLOCATOR Allocator,
//      ULONG MethodItemSize)
// {
//     return KspMethodHandlerWithAllocator(Irp, MethodSetsCount, MethodSet, Allocator, MethodItemSize);
// }
//
//
// NTSTATUS
// FindFastMethodHandler(
//     IN ULONG FastIoCount,
//     IN const KSFASTMETHOD_ITEM * FastIoTable,
//     IN PKSMETHOD MethodId,
//     OUT PFNKSFASTHANDLER * FastPropertyHandler)
// {
//     ULONG Index;
//
//     /* iterate through all items */
//     for(Index = 0; Index < FastIoCount; Index++)
//     {
//         if (MethodId->Id == FastIoTable[Index].MethodId)
//         {
//             if (FastIoTable[Index].MethodSupported)
//             {
//                 *FastPropertyHandler = FastIoTable[Index].MethodHandler;
//                 return STATUS_SUCCESS;
//             }
//         }
//
//     }
//     /* no fast property handler found */
//     return STATUS_NOT_FOUND;
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI
// BOOLEAN
// NTAPI
// KsFastMethodHandler(
//     IN  PFILE_OBJECT FileObject,
//     IN  PKSMETHOD UNALIGNED Method,
//     IN  ULONG MethodLength,
//     IN  OUT PVOID UNALIGNED Data,
//     IN  ULONG DataLength,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN  ULONG MethodSetsCount,
//     IN  const KSMETHOD_SET* MethodSet)
// {
//     KSMETHOD MethodRequest;
//     KPROCESSOR_MODE Mode;
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG Index;
//     PFNKSFASTHANDLER FastMethodHandler;
//
//     if (MethodLength < sizeof(KSPROPERTY))
//     {
//         /* invalid request */
//         return FALSE;
//     }
//
//     /* get previous mode */
//     Mode = ExGetPreviousMode();
//
//     if (Mode == KernelMode)
//     {
//         /* just copy it */
//         RtlMoveMemory(&MethodRequest, Method, sizeof(KSMETHOD));
//     }
//     else
//     {
//         /* need to probe the buffer */
//         _SEH2_TRY
//         {
//             ProbeForRead(Method, sizeof(KSPROPERTY), sizeof(UCHAR));
//             RtlMoveMemory(&MethodRequest, Method, sizeof(KSMETHOD));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Exception, get the error code */
//             Status = _SEH2_GetExceptionCode();
//         }_SEH2_END;
//
//         if (!NT_SUCCESS(Status))
//             return FALSE;
//     }
//
//     /* are there any property sets provided */
//     if (MethodSetsCount)
//     {
//         /* iterate through all property sets count */
//         Index = 0;
//         do
//         {
//             /* does the property id match */
//             if (IsEqualGUIDAligned(MethodSet[Index].Set, &MethodRequest.Set))
//             {
//                 /* try to find a fast property handler */
//                 Status = FindFastMethodHandler(MethodSet[Index].FastIoCount, MethodSet[Index].FastIoTable, &MethodRequest, &FastMethodHandler);
//
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* call fast property handler */
//                     ASSERT(MethodLength == sizeof(KSMETHOD)); /* FIXME check if property length is bigger -> copy params */
//                     ASSERT(Mode == KernelMode); /* FIXME need to probe usermode output buffer */
//                     return FastMethodHandler(FileObject, &MethodRequest, sizeof(KSMETHOD), Data, DataLength, IoStatus);
//                 }
//             }
//             /* move to next item */
//             Index++;
//         }while(Index < MethodSetsCount);
//     }
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
