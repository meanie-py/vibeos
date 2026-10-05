use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ExGetPreviousMode, GUID, IoGetCurrentIrpStackLocation, KPROCESSOR_MODE, LPGUID, PFILE_OBJECT, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// ProbeForRead is a real ntoskrnl export; declare locally so this module links
// (sibling modules api.rs/irp.rs do the same). KSMETHOD/PKSMETHOD, KSMETHOD_SET/
// PKSMETHOD_SET, KSFASTMETHOD_ITEM and PFNKSFASTHANDLER come from crate::types.
extern "system" {
}

// guiddef.h IsEqualGUIDAligned (private per-module helper; int/BOOLEAN form — all
// call sites in this file use it in `!= 0` context).
#[inline(always)]
unsafe fn IsEqualGUIDAligned(a: *const GUID, b: *const GUID) -> i32 {
    ((*a).Data1 == (*b).Data1
        && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3
        && (*a).Data4 == (*b).Data4) as i32
}

// ks.h KSMETHOD_SET_IRP_STORAGE(Irp) == *(PKSMETHOD_SET*)&Irp->Tail.Overlay.DriverContext[0]
macro_rules! KSMETHOD_SET_IRP_STORAGE { ($irp:expr) => {
    *(&mut (*$irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0]
        as *mut PVOID as *mut PKSMETHOD_SET)
}; }
#[no_mangle]
pub unsafe extern "C" fn FindFastMethodHandler(
    FastIoCount: ULONG,
    FastIoTable: *const KSFASTMETHOD_ITEM,
    MethodId: PKSMETHOD,
    FastMethodHandler: *mut PFNKSFASTHANDLER,
) -> NTSTATUS {
    let mut Index: ULONG = 0;
    while Index < FastIoCount {
        let item = &*FastIoTable.offset(Index as isize);
        if (*MethodId).Id == item.MethodId {
            if *item.__bindgen_anon_1.MethodSupported.as_ref() != 0 {
                *FastMethodHandler = *item.__bindgen_anon_1.MethodHandler.as_ref();
                return STATUS_SUCCESS;
            }
        }
        Index += 1;
    }
    STATUS_NOT_FOUND
}
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
                        (*IoStatus).Information = (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).MinMethod as _;
                        return STATUS_INVALID_PARAMETER;
                    }

                    if (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).MinData > OutputBufferLength {
                        (*IoStatus).Information = (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).MinData as _;
                        return STATUS_MORE_ENTRIES;
                    }

                    if (*Method).Flags & KSMETHOD_TYPE_BASICSUPPORT != 0 {
                        let Flags: *mut ULONG;
                        let Description: *mut KSPROPERTY_DESCRIPTION;

                        if core::mem::size_of::<ULONG>() as ULONG > OutputBufferLength {
                            return STATUS_INVALID_PARAMETER;
                        }

                        Flags = OutputBuffer as *mut ULONG;
                        *Flags = (*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).Flags;
                        (*IoStatus).Information = core::mem::size_of::<ULONG>() as _;

                        if OutputBufferLength >= core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as ULONG {
                            Description = OutputBuffer as *mut KSPROPERTY_DESCRIPTION;
                            (*Description).DescriptionSize = core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as ULONG;
                            (*Description).PropTypeSet.Set = KSPROPTYPESETID_General;
                            (*Description).PropTypeSet.Id = 0;
                            (*Description).PropTypeSet.Flags = 0;
                            (*Description).MembersListCount = 0;
                            (*Description).Reserved = 0;
                            (*IoStatus).Information = core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as _;
                        }
                        return STATUS_SUCCESS;
                    }

                    *MethodHandler = *(*((*MethodSet.offset(Index as isize)).MethodItem.offset(ItemIndex as isize))).__bindgen_anon_1.MethodHandler.as_ref();
                    *Set = MethodSet.offset(Index as isize) as *mut KSMETHOD_SET;
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
    let mut Set: *mut KSMETHOD_SET = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let MethodHandler: PFNKSHANDLER = None;
    let mut Index: ULONG;
    let Guid: LPGUID;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<KSMETHOD>() as ULONG {
        (*Irp).IoStatus.Information = core::mem::size_of::<KSPROPERTY>() as _;
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

    if NT_SUCCESS(Status) && MethodHandler.is_some() {
        KSMETHOD_SET_IRP_STORAGE!(Irp) = Set;
        Status = (MethodHandler.unwrap())(Irp, Method, (*Irp).UserBuffer);

        if Status == STATUS_BUFFER_TOO_SMALL {
            if Allocator.is_some() {
                Status = (Allocator.unwrap())(Irp, (*Irp).IoStatus.Information as ULONG, 0u8);
                if !NT_SUCCESS(Status) {
                    return STATUS_INSUFFICIENT_RESOURCES;
                }
                Status = (MethodHandler.unwrap())(Irp, Method, (*Irp).UserBuffer);
            }
        }
    } else if IsEqualGUIDAligned(&(*Method).Set as *const GUID, &GUID_NULL as *const GUID) != 0
        && (*Method).Id == 0
        && (*Method).Flags == KSMETHOD_TYPE_SETSUPPORT
    {
        (*Irp).IoStatus.Information = ((core::mem::size_of::<GUID>() as ULONG) * MethodSetsCount) as _;
        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < (core::mem::size_of::<GUID>() as ULONG) * MethodSetsCount {
            return STATUS_MORE_ENTRIES;
        }

        Guid = (*Irp).UserBuffer as LPGUID;
        Index = 0;
        while Index < MethodSetsCount {
            core::ptr::copy_nonoverlapping(
                (*MethodSet.add(Index as usize)).Set,
                &mut *Guid.add(Index as usize) as *mut GUID,
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
    return KspMethodHandlerWithAllocator(Irp, MethodSetsCount, MethodSet, None, 0);
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
    IoStatus: PIO_STATUS_BLOCK,
    MethodSetsCount: u32,
    MethodSet: *const KSMETHOD_SET,
) -> u8 {
    let mut MethodRequest: KSMETHOD = core::mem::zeroed();
    let Mode: KPROCESSOR_MODE;
    let mut Status: i32 = 0i32;
    let mut Index: u32;
    let mut FastMethodHandler: PFNKSFASTHANDLER = None;

    if MethodLength < core::mem::size_of::<KSPROPERTY>() as u32 {
        return 0u8;
    }

    Mode = ExGetPreviousMode();

    if Mode == 0i8 {
        core::ptr::copy_nonoverlapping(
            Method as *const u8,
            &mut MethodRequest as *mut KSMETHOD as *mut u8,
            core::mem::size_of::<KSMETHOD>(),
        );
    } else {
        // C: probe + copy under _SEH2_TRY, whose handler stores the exception
        // code; failure returns FALSE. (The C probes sizeof(KSPROPERTY) and
        // copies sizeof(KSMETHOD): both are a KSIDENTIFIER.)
        if let Err(status) = ntkrnl_sys::probe::safe_copy_from_user(
            &mut MethodRequest as *mut KSMETHOD as PVOID,
            Method as *const core::ffi::c_void,
            core::mem::size_of::<KSMETHOD>(),
            1,
            1,
        ) {
            Status = status;
        }

        if !NT_SUCCESS(Status) {
            return 0u8;
        }
    }

    // are there any method sets provided
    if MethodSetsCount != 0 {
        // iterate through all method sets
        Index = 0;
        loop {
            // does the set id match
            if IsEqualGUIDAligned(
                (*MethodSet.offset(Index as isize)).Set,
                &MethodRequest.Set as *const GUID,
            ) != 0
            {
                // try to find a fast method handler
                Status = FindFastMethodHandler(
                    (*MethodSet.offset(Index as isize)).FastIoCount,
                    (*MethodSet.offset(Index as isize)).FastIoTable,
                    &mut MethodRequest,
                    &mut FastMethodHandler,
                );

                if NT_SUCCESS(Status) {
                    // call fast method handler
                    if let Some(handler) = FastMethodHandler {
                        return handler(
                            FileObject,
                            &mut MethodRequest,
                            core::mem::size_of::<KSMETHOD>() as u32,
                            Data,
                            DataLength,
                            IoStatus,
                        );
                    }
                }
            }
            // move to next item
            Index += 1;
            if Index >= MethodSetsCount {
                break;
            }
        }
    }
    0u8
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
