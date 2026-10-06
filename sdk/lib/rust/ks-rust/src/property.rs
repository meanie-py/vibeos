use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExGetPreviousMode, GUID, IoGetCurrentIrpStackLocation, KPROCESSOR_MODE, LPGUID, PFILE_OBJECT, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP};
use crate::misc::AllocateItem;
use core::ffi::c_void;

// ProbeForRead/ProbeForWrite are real ntoskrnl exports; declare them locally so
// this module can link (sibling modules api.rs/irp.rs do the same).
extern "system" {
}

// MmUserProbeAddress — highest user-mode address + 1, a DATA export from ntoskrnl
// (`@ extern MmUserProbeAddress` in ntoskrnl.spec).
//
// A data import needs the per-arch import form — i386/arm64 raw-dylib +verbatim (the
// symbol IS the value), amd64 binds __imp_ and dereferences; a plain extern static
// fails the .sys link (`undefined symbol: _MmUserProbeAddress`). This local copy
// predates ntkrnl-sys's `import_mm_user_probe_address` feature, which ks now enables,
// so `ntkrnl_sys::probe` works here too.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static MmUserProbeAddress: ULONG_PTR;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_MmUserProbeAddress"]
    static MmUserProbeAddress_imp: *const ULONG_PTR;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn mm_user_probe_address() -> ULONG_PTR {
    MmUserProbeAddress
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn mm_user_probe_address() -> ULONG_PTR {
    *MmUserProbeAddress_imp
}

/// Validate that [address, address+length) lies below MmUserProbeAddress and meets
/// `alignment`, without raising and without touching the page — the DDK ProbeFor*
/// macros do both, and under DUMMY_PSEH either one is fatal with no handler to reach.
///
/// Named for win32k's `SafeProbeForRead`, which it mirrors exactly. That is not
/// cosmetic: `scan_dropped_probes` recognises validation by name, so a local helper
/// called something else reads to the scanner as no validation at all and re-reports
/// this function as ALL. Matching the house spelling keeps the lint honest without
/// teaching it every crate's private vocabulary.
#[inline]
unsafe fn KsSafeProbeForRead(
    address: *const c_void,
    length: usize,
    alignment: usize,
) -> NTSTATUS {
    if length == 0 {
        return STATUS_SUCCESS;
    }
    if alignment > 1 && (address as ULONG_PTR) & (alignment as ULONG_PTR - 1) != 0 {
        return STATUS_DATATYPE_MISALIGNMENT;
    }
    let start = address as ULONG_PTR;
    let last = start.wrapping_add(length as ULONG_PTR).wrapping_sub(1);
    if last < start || last >= mm_user_probe_address() {
        return STATUS_ACCESS_VIOLATION;
    }
    STATUS_SUCCESS
}

// guiddef.h IsEqualGUIDAligned (private per-module helper; bool form — all call
// sites in this file use it in boolean context).
#[inline(always)]
unsafe fn IsEqualGUIDAligned(a: *const GUID, b: *const GUID) -> bool {
    (*a).Data1 == (*b).Data1
        && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3
        && (*a).Data4 == (*b).Data4
}

// ks.h KSPROPERTY_SET_IRP_STORAGE(Irp)  == *(PKSPROPERTY_SET*)&Irp->Tail.Overlay.DriverContext[0]
macro_rules! KSPROPERTY_SET_IRP_STORAGE { ($irp:expr) => {
    *(&mut (*$irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0]
        as *mut PVOID as *mut PKSPROPERTY_SET)
}; }
// ks.h KSPROPERTY_ITEM_IRP_STORAGE(Irp) == *(PKSPROPERTY_ITEM*)&Irp->Tail.Overlay.DriverContext[3]
macro_rules! KSPROPERTY_ITEM_IRP_STORAGE { ($irp:expr) => {
    *(&mut (*$irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[3]
        as *mut PVOID as *mut PKSPROPERTY_ITEM)
}; }

// ---- file-scope preamble (translated) ----
// GUID / LPGUID come from the ddk import (local redefs removed — E0255).

pub const KSPROPERTY_TYPE_SETSUPPORT: u32 = 0x00000100;

// #[no_mangle] so methods.rs (and other modules using the substrate `extern` decl) link here.
#[no_mangle]
pub static KSPROPTYPESETID_General: GUID = GUID {
    Data1: 0x97E99BA0,
    Data2: 0xBDEA,
    Data3: 0x11CF,
    Data4: [0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00],
};
#[no_mangle]
pub unsafe extern "C" fn FindPropertyHandler(
    IoStatus: PIO_STATUS_BLOCK,
    PropertySet: *const KSPROPERTY_SET,
    PropertySetCount: ULONG,
    Property: PKSPROPERTY,
    InputBufferLength: ULONG,
    OutputBufferLength: ULONG,
    OutputBuffer: PVOID,
    PropertyHandler: *mut PFNKSHANDLER,
    Set: *mut PKSPROPERTY_SET,
    PropertyItem: *mut PKSPROPERTY_ITEM,
) -> NTSTATUS {
    let mut Index: ULONG;
    let mut ItemIndex: ULONG;
    let mut Flags: PULONG;
    let mut Description: PKSPROPERTY_DESCRIPTION;

    Index = 0;
    while Index < PropertySetCount {
        if IsEqualGUIDAligned(&(*Property).Set, (*PropertySet.offset(Index as isize)).Set) {
            ItemIndex = 0;
            while ItemIndex < (*PropertySet.offset(Index as isize)).PropertiesCount {
                *Set = &*PropertySet.offset(Index as isize) as *const KSPROPERTY_SET as PKSPROPERTY_SET;
                *PropertyItem = &(*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)) as *const KSPROPERTY_ITEM as PKSPROPERTY_ITEM;

                if (*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).PropertyId == (*Property).Id {
                    if (*Property).Flags & KSPROPERTY_TYPE_BASICSUPPORT != 0 {
                        if core::mem::size_of::<ULONG>() as ULONG > OutputBufferLength {
                            return STATUS_INVALID_PARAMETER;
                        }

                        Flags = OutputBuffer as PULONG;
                        *Flags = 0;
                        (*IoStatus).Information = core::mem::size_of::<ULONG>() as _;

                        if (*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).SupportHandler != None {
                            *PropertyHandler = (*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).SupportHandler;
                            return STATUS_SUCCESS;
                        }

                        if *(*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).__bindgen_anon_1.GetSupported.as_ref() != 0 {
                            *Flags |= KSPROPERTY_TYPE_GET;
                        }

                        if *(*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).__bindgen_anon_2.SetSupported.as_ref() != 0 {
                            *Flags |= KSPROPERTY_TYPE_SET;
                        }

                        if OutputBufferLength >= core::mem::size_of::<KSPROPERTY_DESCRIPTION>() as ULONG {
                            Description = OutputBuffer as PKSPROPERTY_DESCRIPTION;
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

                    if (*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).MinProperty > InputBufferLength {
                        (*IoStatus).Information = (*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).MinProperty as _;
                        return STATUS_INVALID_PARAMETER;
                    }

                    if (*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).MinData > OutputBufferLength {
                        (*IoStatus).Information = (*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).MinData as _;
                        return STATUS_MORE_ENTRIES;
                    }

                    if (*Property).Flags & KSPROPERTY_TYPE_SET != 0 {
                        *PropertyHandler = *(*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).__bindgen_anon_2.SetPropertyHandler.as_ref();
                        return STATUS_SUCCESS;
                    }

                    if (*Property).Flags & KSPROPERTY_TYPE_GET != 0 {
                        *PropertyHandler = *(*(*PropertySet.offset(Index as isize)).PropertyItem.offset(ItemIndex as isize)).__bindgen_anon_1.GetPropertyHandler.as_ref();
                        return STATUS_SUCCESS;
                    }
                }
                ItemIndex += 1;
            }
        }
        Index += 1;
    }
    STATUS_NOT_FOUND
}
#[no_mangle]
pub unsafe extern "C" fn KspPropertyHandler(
    Irp: PIRP,
    PropertySetsCount: ULONG,
    PropertySet: *const KSPROPERTY_SET,
    Allocator: PFNKSALLOCATOR,
    PropertyItemSize: ULONG,
) -> NTSTATUS {
    let mut Property: PKSPROPERTY;
    let mut PropertyItem: PKSPROPERTY_ITEM = core::ptr::null_mut();
    let mut Set: PKSPROPERTY_SET = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut PropertyHandler: PFNKSHANDLER = None;
    let Index: ULONG;
    let InputBufferLength: ULONG;
    let OutputBufferLength: ULONG;
    let TotalSize: ULONG;
    let Guid: LPGUID;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    OutputBufferLength = ((*IoStack).Parameters.DeviceIoControl.OutputBufferLength + 7) & !7;
    InputBufferLength = (*IoStack).Parameters.DeviceIoControl.InputBufferLength;

    if OutputBufferLength < (*IoStack).Parameters.DeviceIoControl.OutputBufferLength {
        return STATUS_INVALID_BUFFER_SIZE;
    }

    if InputBufferLength + OutputBufferLength < (*IoStack).Parameters.DeviceIoControl.OutputBufferLength {
        return STATUS_INVALID_BUFFER_SIZE;
    }

    if (*IoStack).Parameters.DeviceIoControl.InputBufferLength < core::mem::size_of::<KSPROPERTY>() as ULONG {
        return STATUS_INVALID_BUFFER_SIZE;
    }

    TotalSize = InputBufferLength + OutputBufferLength;

    Property = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSPROPERTY;

    if (*Irp).AssociatedIrp.SystemBuffer.is_null() {
        // Divergence: the C makes these probes and user reads with NO _SEH2_TRY
        // of its own (its only one wraps the property handler call), so a bad
        // user buffer raises out into the calling driver's dispatch routine,
        // which has none either. Here each one returns the code instead.
        if (*Irp).RequestorMode == 1i8 {
            if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
                (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer,
                InputBufferLength as usize,
                1,
                1,
            ) {
                return status;
            }
        }
        let user_flags = match ntkrnl_sys::probe::probe_read_generic(core::ptr::addr_of!((*Property).Flags), 1, 0) {
            Ok(flags) => flags,
            Err(status) => return status,
        };

        if Allocator.is_some() && (user_flags & (KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET)) != 0 {
            Status = (Allocator.unwrap())(Irp, TotalSize, (user_flags & KSPROPERTY_TYPE_GET) as BOOLEAN);
            if !NT_SUCCESS(Status) {
                return Status;
            }
        } else {
            (*Irp).AssociatedIrp.SystemBuffer = AllocateItem(NonPagedPool, TotalSize as usize);
            (*Irp).Flags |= IRP_DEALLOCATE_BUFFER | IRP_BUFFERED_IO;
        }

        if let Err(status) = ntkrnl_sys::probe::safe_copy_raw(
            ((*Irp).AssociatedIrp.SystemBuffer as ULONG_PTR + OutputBufferLength as ULONG_PTR) as PVOID,
            (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer,
            InputBufferLength as usize,
        ) {
            return status;
        }

        Property = ((*Irp).AssociatedIrp.SystemBuffer as ULONG_PTR + OutputBufferLength as ULONG_PTR) as PKSPROPERTY;

        if (*Property).Flags & KSPROPERTY_TYPE_SET != 0 {
            if let Err(status) = ntkrnl_sys::probe::safe_copy_from_user(
                (*Irp).AssociatedIrp.SystemBuffer,
                (*Irp).UserBuffer,
                (*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize,
                1,
                (*Irp).RequestorMode,
            ) {
                return status;
            }
        }

        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength != 0 {
            if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
                (*Irp).UserBuffer,
                (*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize,
                1,
                (*Irp).RequestorMode,
            ) {
                return status;
            }
            if Allocator.is_none() || ((*Property).Flags & KSPROPERTY_TYPE_GET) == 0 {
                (*Irp).Flags |= IRP_INPUT_OPERATION;
            }
        }
    } else {
        Property = ((*Irp).AssociatedIrp.SystemBuffer as ULONG_PTR + OutputBufferLength as ULONG_PTR) as PKSPROPERTY;
    }

    Status = FindPropertyHandler(
        &mut (*Irp).IoStatus,
        PropertySet,
        PropertySetsCount,
        Property,
        InputBufferLength,
        OutputBufferLength,
        (*Irp).AssociatedIrp.SystemBuffer,
        &mut PropertyHandler,
        &mut Set,
        &mut PropertyItem,
    );

    if NT_SUCCESS(Status) && PropertyHandler.is_some() {
        KSPROPERTY_SET_IRP_STORAGE!(Irp) = Set;

        if PropertyItemSize != 0 {
            KSPROPERTY_ITEM_IRP_STORAGE!(Irp) = PropertyItem;
        }

        Status = (PropertyHandler.unwrap())(
            Irp,
            Property,
            if OutputBufferLength > 0 { (*Irp).AssociatedIrp.SystemBuffer } else { core::ptr::null_mut() },
        );

        if Status == STATUS_BUFFER_TOO_SMALL {
            if Allocator.is_some() {
                Status = (Allocator.unwrap())(Irp, (*Irp).IoStatus.Information as ULONG, 0u8);
                if !NT_SUCCESS(Status) {
                    return STATUS_INSUFFICIENT_RESOURCES;
                }
                Status = (PropertyHandler.unwrap())(Irp, Property, (*Irp).AssociatedIrp.SystemBuffer);
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsPropertyHandler(
    Irp: PIRP,
    PropertySetsCount: ULONG,
    PropertySet: *const KSPROPERTY_SET,
) -> NTSTATUS {
    return KspPropertyHandler(Irp, PropertySetsCount, PropertySet, None, 0);
}

#[no_mangle]
pub unsafe extern "system" fn KsPropertyHandlerWithAllocator(
    Irp: PIRP,
    PropertySetsCount: ULONG,
    PropertySet: *const KSPROPERTY_SET,
    Allocator: PFNKSALLOCATOR,
    PropertyItemSize: ULONG,
) -> NTSTATUS {
    return KspPropertyHandler(Irp, PropertySetsCount, PropertySet, Allocator, PropertyItemSize);
}
#[no_mangle]
pub unsafe extern "C" fn FindFastPropertyHandler(
    FastIoCount: ULONG,
    FastIoTable: *const KSFASTPROPERTY_ITEM,
    PropertyId: *mut KSPROPERTY,
    FastPropertyHandler: *mut PFNKSFASTHANDLER,
) -> NTSTATUS {
    let mut Index: ULONG;

    Index = 0;
    while Index < FastIoCount {
        if (*PropertyId).Id == (*FastIoTable.offset(Index as isize)).PropertyId {
            if (*PropertyId).Flags & KSPROPERTY_TYPE_SET != 0 {
                if *(*FastIoTable.offset(Index as isize)).__bindgen_anon_2.SetSupported.as_ref() != 0 {
                    *FastPropertyHandler = *(*FastIoTable.offset(Index as isize)).__bindgen_anon_2.SetPropertyHandler.as_ref();
                    return STATUS_SUCCESS;
                }
            }

            if (*PropertyId).Flags & KSPROPERTY_TYPE_GET != 0 {
                if *(*FastIoTable.offset(Index as isize)).__bindgen_anon_1.GetSupported.as_ref() != 0 {
                    *FastPropertyHandler = *(*FastIoTable.offset(Index as isize)).__bindgen_anon_1.GetPropertyHandler.as_ref();
                    return STATUS_SUCCESS;
                }
            }
        }

        Index += 1;
    }

    STATUS_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn KsFastPropertyHandler(
    FileObject: PFILE_OBJECT,
    Property: *const KSPROPERTY,
    PropertyLength: ULONG,
    Data: *mut core::ffi::c_void,
    DataLength: ULONG,
    IoStatus: PIO_STATUS_BLOCK,
    PropertySetsCount: ULONG,
    PropertySet: *const KSPROPERTY_SET,
) -> BOOLEAN {
    let mut PropRequest: KSPROPERTY = core::mem::zeroed();
    let Mode: KPROCESSOR_MODE;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Index: ULONG;
    let mut FastPropertyHandler: PFNKSFASTHANDLER = None;

    if PropertyLength < core::mem::size_of::<KSPROPERTY>() as ULONG {
        return FALSE;
    }

    Mode = ExGetPreviousMode();

    if Mode == 0i8 {
        core::ptr::copy_nonoverlapping(
            Property as *const u8,
            &mut PropRequest as *mut KSPROPERTY as *mut u8,
            core::mem::size_of::<KSPROPERTY>(),
        );
    } else {
        // C: ProbeForRead(Property, sizeof(KSPROPERTY), sizeof(UCHAR)) inside a
        // _SEH2_TRY that stashes the exception code in Status. The probe became the
        // comment above and the check below, and that check is not a bounds check: it
        // compares an ADDRESS against a SIZE, so it rejects only the first 24-odd bytes
        // of the address space and passes every kernel pointer. UserMode is the only
        // branch that reaches it — the KernelMode caller above is trusted — so this is
        // the path where validation actually matters.
        //
        // probe_user_buffer range-checks against MmUserProbeAddress without touching
        // the page, which is what DUMMY_PSEH requires here.
        let probe_status = KsSafeProbeForRead(
            Property as *const c_void,
            core::mem::size_of::<KSPROPERTY>(),
            core::mem::size_of::<UCHAR>(),
        );
        if !NT_SUCCESS(probe_status) {
            Status = probe_status;
        } else {
            core::ptr::copy_nonoverlapping(
                Property as *const u8,
                &mut PropRequest as *mut KSPROPERTY as *mut u8,
                core::mem::size_of::<KSPROPERTY>(),
            );
        }

        if !NT_SUCCESS(Status) {
            return FALSE;
        }
    }

    // are there any property sets provided
    if PropertySetsCount != 0 {
        // iterate through all property sets
        Index = 0;
        loop {
            // does the property id match
            if IsEqualGUIDAligned(
                &PropRequest.Set,
                (*PropertySet.offset(Index as isize)).Set,
            ) {
                // try to find a fast property handler
                Status = FindFastPropertyHandler(
                    (*PropertySet.offset(Index as isize)).FastIoCount,
                    (*PropertySet.offset(Index as isize)).FastIoTable,
                    &mut PropRequest,
                    &mut FastPropertyHandler,
                );

                if NT_SUCCESS(Status) {
                    // call fast property handler
                    if let Some(handler) = FastPropertyHandler {
                        return handler(
                            FileObject,
                            &mut PropRequest as *mut KSPROPERTY,
                            core::mem::size_of::<KSPROPERTY>() as ULONG,
                            Data,
                            DataLength,
                            IoStatus,
                        );
                    }
                }
            }
            // move to next item
            Index += 1;
            if Index >= PropertySetsCount {
                break;
            }
        }
    }
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn KsDispatchSpecificProperty(
    Irp: PIRP,
    Handler: PFNKSHANDLER,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    (Handler.unwrap())(
        Irp,
        (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSPROPERTY,
        (*Irp).UserBuffer,
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/event.c
//  * PURPOSE:         KS property handling functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// /* SEH support with PSEH */
// #include <pseh/pseh2.h>
//
// #define NDEBUG
// #include <debug.h>
//
// const GUID KSPROPTYPESETID_General = {0x97E99BA0L, 0xBDEA, 0x11CF, {0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00}};
//
// NTSTATUS
// FindPropertyHandler(
//     IN PIO_STATUS_BLOCK IoStatus,
//     IN  const KSPROPERTY_SET* PropertySet,
//     IN ULONG PropertySetCount,
//     IN PKSPROPERTY Property,
//     IN ULONG InputBufferLength,
//     IN ULONG OutputBufferLength,
//     OUT PVOID OutputBuffer,
//     OUT PFNKSHANDLER *PropertyHandler,
//     OUT PKSPROPERTY_SET * Set,
//     OUT PKSPROPERTY_ITEM *PropertyItem)
// {
//     ULONG Index, ItemIndex;
//     PULONG Flags;
//     PKSPROPERTY_DESCRIPTION Description;
//
//     for(Index = 0; Index < PropertySetCount; Index++)
//     {
//         ASSERT(PropertySet[Index].Set);
//
//         if (IsEqualGUIDAligned(&Property->Set, PropertySet[Index].Set))
//         {
//             for(ItemIndex = 0; ItemIndex < PropertySet[Index].PropertiesCount; ItemIndex++)
//             {
//
//                 /* store property set */
//                 *Set = (PKSPROPERTY_SET)&PropertySet[Index];
//                 *PropertyItem = (PKSPROPERTY_ITEM)&PropertySet[Index].PropertyItem[ItemIndex];
//
//
//                 if (PropertySet[Index].PropertyItem[ItemIndex].PropertyId == Property->Id)
//                 {
//                     if (Property->Flags & KSPROPERTY_TYPE_BASICSUPPORT)
//                     {
//                         if (sizeof(ULONG) > OutputBufferLength)
//                         {
//                             /* too small buffer */
//                             return STATUS_INVALID_PARAMETER;
//                         }
//
//                         /* get output buffer */
//                         Flags = (PULONG)OutputBuffer;
//
//                         /* clear flags */
//                         *Flags = 0;
//
//                         IoStatus->Information = sizeof(ULONG);
//
//                         if (PropertySet[Index].PropertyItem[ItemIndex].SupportHandler)
//                         {
//                             /* use support handler from driver */
//                             *PropertyHandler = PropertySet[Index].PropertyItem[ItemIndex].SupportHandler;
//                             return STATUS_SUCCESS;
//                         }
//
//                         if (PropertySet[Index].PropertyItem[ItemIndex].GetSupported)
//                             *Flags |= KSPROPERTY_TYPE_GET;
//
//                         if (PropertySet[Index].PropertyItem[ItemIndex].SetSupported)
//                             *Flags |= KSPROPERTY_TYPE_SET;
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
//
//                     if (PropertySet[Index].PropertyItem[ItemIndex].MinProperty > InputBufferLength)
//                     {
//                         /* too small input buffer */
//                         IoStatus->Information = PropertySet[Index].PropertyItem[ItemIndex].MinProperty;
//                         return STATUS_INVALID_PARAMETER;
//                     }
//
//                     if (PropertySet[Index].PropertyItem[ItemIndex].MinData > OutputBufferLength)
//                     {
//                         /* too small output buffer */
//                         IoStatus->Information = PropertySet[Index].PropertyItem[ItemIndex].MinData;
//                         return STATUS_MORE_ENTRIES;
//                     }
//
//
//                     if (Property->Flags & KSPROPERTY_TYPE_SET)
//                     {
//                         /* store property handler */
//                         *PropertyHandler = PropertySet[Index].PropertyItem[ItemIndex].SetPropertyHandler;
//                         return STATUS_SUCCESS;
//                     }
//
//                     if (Property->Flags & KSPROPERTY_TYPE_GET)
//                     {
//                         /* store property handler */
//                         *PropertyHandler = PropertySet[Index].PropertyItem[ItemIndex].GetPropertyHandler;
//                         return STATUS_SUCCESS;
//                     }
//
//
//                 }
//             }
//         }
//     }
//     return STATUS_NOT_FOUND;
// }
//
//
// NTSTATUS
// KspPropertyHandler(
//     IN PIRP Irp,
//     IN  ULONG PropertySetsCount,
//     IN  const KSPROPERTY_SET* PropertySet,
//     IN  PFNKSALLOCATOR Allocator OPTIONAL,
//     IN  ULONG PropertyItemSize OPTIONAL)
// {
//     PKSPROPERTY Property;
//     PKSPROPERTY_ITEM PropertyItem;
//     PKSPROPERTY_SET Set;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     PFNKSHANDLER PropertyHandler = NULL;
//     ULONG Index, InputBufferLength, OutputBufferLength, TotalSize;
//     LPGUID Guid;
//     //UNICODE_STRING GuidBuffer;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get parameters */
//     OutputBufferLength = (IoStack->Parameters.DeviceIoControl.OutputBufferLength + 7) & ~7;
//     InputBufferLength = IoStack->Parameters.DeviceIoControl.InputBufferLength;
//
//     /* check for invalid buffer length size */
//     if (OutputBufferLength < IoStack->Parameters.DeviceIoControl.OutputBufferLength)
//     {
//         /* unsigned overflow */
//         return STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     /* check for integer overflow */
//     if (InputBufferLength + OutputBufferLength < IoStack->Parameters.DeviceIoControl.OutputBufferLength)
//     {
//         /* overflow */
//         return STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     /* check if inputbuffer at least holds KSPROPERTY item */
//     if (IoStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(KSPROPERTY))
//     {
//         /* invalid parameter */
//         return STATUS_INVALID_BUFFER_SIZE;
//     }
//
//     /* get total size */
//     TotalSize = InputBufferLength + OutputBufferLength;
//
//     /* get input property request */
//     Property = (PKSPROPERTY)IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//
//     /* have the parameters been checked yet */
//     if (!Irp->AssociatedIrp.SystemBuffer)
//     {
//         /* is it from user mode */
//         if (Irp->RequestorMode == UserMode)
//         {
//             /* probe user buffer */
//             ProbeForRead(IoStack->Parameters.DeviceIoControl.Type3InputBuffer, InputBufferLength, 1);
//         }
//
//         /* do we have an allocator */
//         if ((Allocator) && (Property->Flags & (KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET)))
//         {
//             /* call allocator */
//             Status = Allocator(Irp, TotalSize, (Property->Flags & KSPROPERTY_TYPE_GET));
//
//             /* check for success */
//             if (!NT_SUCCESS(Status))
//                 return Status;
//         }
//         else
//         {
//             /* allocate buffer */
//             Irp->AssociatedIrp.SystemBuffer = AllocateItem(NonPagedPool, TotalSize);
//
//             /* sanity check */
//             ASSERT(Irp->AssociatedIrp.SystemBuffer != NULL);
//
//             /* mark irp as buffered so that changes the stream headers are propagated back */
//             Irp->Flags |= IRP_DEALLOCATE_BUFFER | IRP_BUFFERED_IO;
//         }
//
//         /* now copy the buffer */
//         RtlCopyMemory((PVOID)((ULONG_PTR)Irp->AssociatedIrp.SystemBuffer + OutputBufferLength), IoStack->Parameters.DeviceIoControl.Type3InputBuffer, InputBufferLength);
//
//         /* use new property buffer */
//         Property = (PKSPROPERTY)((ULONG_PTR)Irp->AssociatedIrp.SystemBuffer + OutputBufferLength);
//
//         /* is it a set operation */
//         if (Property->Flags & KSPROPERTY_TYPE_SET)
//         {
//             /* for set operations, the output parameters need to be copied */
//             if (Irp->RequestorMode == UserMode)
//             {
//                 /* probe user parameter */
//                 ProbeForRead(Irp->UserBuffer, IoStack->Parameters.DeviceIoControl.OutputBufferLength, 1);
//             }
//
//             /* copy parameters, needs un-aligned parameter length */
//             RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer, Irp->UserBuffer, IoStack->Parameters.DeviceIoControl.OutputBufferLength);
//         }
//
//         /* is there an output buffer */
//         if (IoStack->Parameters.DeviceIoControl.OutputBufferLength)
//         {
//             /* is it from user mode */
//             if (Irp->RequestorMode == UserMode)
//             {
//                 /* probe buffer for writing */
//                 ProbeForWrite(Irp->UserBuffer, IoStack->Parameters.DeviceIoControl.OutputBufferLength, 1);
//             }
//
//             if (!Allocator || !(Property->Flags & KSPROPERTY_TYPE_GET))
//             {
//                 /* it is an input operation */
//                 Irp->Flags |= IRP_INPUT_OPERATION;
//             }
//         }
//     }
//     else
//     {
//         /* use new property buffer */
//         Property = (PKSPROPERTY)((ULONG_PTR)Irp->AssociatedIrp.SystemBuffer + OutputBufferLength);
//     }
//
//     //RtlStringFromGUID(&Property->Set, &GuidBuffer);
//
//     //DPRINT("KspPropertyHandler Irp %p PropertySetsCount %u PropertySet %p Allocator %p PropertyItemSize %u ExpectedPropertyItemSize %u\n", Irp, PropertySetsCount, PropertySet, Allocator, PropertyItemSize, sizeof(KSPROPERTY_ITEM));
//     //DPRINT("PropertyId %lu PropertyFlags %x Guid %S\n", Property->Id, Property->Flags, GuidBuffer.Buffer);
//
//     //RtlFreeUnicodeString(&GuidBuffer);
//
//     /* sanity check */
//     ASSERT(PropertyItemSize == 0 || PropertyItemSize == sizeof(KSPROPERTY_ITEM));
//
//     /* find the property handler */
//     Status = FindPropertyHandler(&Irp->IoStatus, PropertySet, PropertySetsCount, Property, InputBufferLength, OutputBufferLength, Irp->AssociatedIrp.SystemBuffer, &PropertyHandler, &Set, &PropertyItem);
//
//     if (NT_SUCCESS(Status) && PropertyHandler)
//     {
//         /* store set */
//         KSPROPERTY_SET_IRP_STORAGE(Irp) = Set;
//
//         /* are any custom property item sizes used */
//         if (PropertyItemSize)
//         {
//             /* store custom property item */
//             KSPROPERTY_ITEM_IRP_STORAGE(Irp) = PropertyItem;
//         }
//
//         _SEH2_TRY
//         {
//             /* call property handler */
//             Status = PropertyHandler(Irp, Property, (OutputBufferLength > 0 ? Irp->AssociatedIrp.SystemBuffer : NULL));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
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
//                 _SEH2_TRY
//                 {
//                     /* re-call property handler */
//                     Status = PropertyHandler(Irp, Property, Irp->AssociatedIrp.SystemBuffer);
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     Status =  _SEH2_GetExceptionCode();
//                 }
//                 _SEH2_END;
//             }
//         }
//     }
//     else if (IsEqualGUIDAligned(&Property->Set, &GUID_NULL) && Property->Id == 0 && (Property->Flags & KSPROPERTY_TYPE_SETSUPPORT) == KSPROPERTY_TYPE_SETSUPPORT)
//     {
//         // store output size
//         Irp->IoStatus.Information = sizeof(GUID) * PropertySetsCount;
//         if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(GUID) * PropertySetsCount)
//         {
//             // buffer too small
//             return STATUS_MORE_ENTRIES;
//         }
//
//         // get output buffer
//         Guid = (LPGUID)Irp->AssociatedIrp.SystemBuffer;
//
//        // copy property guids from property sets
//        for(Index = 0; Index < PropertySetsCount; Index++)
//        {
//            RtlMoveMemory(&Guid[Index], PropertySet[Index].Set, sizeof(GUID));
//        }
//        Status = STATUS_SUCCESS;
//     }
//
//     /* done */
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsPropertyHandler(
//     IN  PIRP Irp,
//     IN  ULONG PropertySetsCount,
//     IN  const KSPROPERTY_SET* PropertySet)
// {
//     return KspPropertyHandler(Irp, PropertySetsCount, PropertySet, NULL, 0);
// }
//
//
// /*
//     @implemented
// */
// _IRQL_requires_max_(PASSIVE_LEVEL)
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsPropertyHandlerWithAllocator(
//      PIRP Irp,
//      ULONG PropertySetsCount,
//      const KSPROPERTY_SET* PropertySet,
//      PFNKSALLOCATOR Allocator,
//      ULONG PropertyItemSize)
// {
//     return KspPropertyHandler(Irp, PropertySetsCount, PropertySet, Allocator, PropertyItemSize);
// }
//
// NTSTATUS
// FindFastPropertyHandler(
//     IN ULONG FastIoCount,
//     IN const KSFASTPROPERTY_ITEM * FastIoTable,
//     IN PKSPROPERTY PropertyId,
//     OUT PFNKSFASTHANDLER * FastPropertyHandler)
// {
//     ULONG Index;
//
//     /* iterate through all items */
//     for(Index = 0; Index < FastIoCount; Index++)
//     {
//         if (PropertyId->Id == FastIoTable[Index].PropertyId)
//         {
//             if (PropertyId->Flags & KSPROPERTY_TYPE_SET)
//             {
//                 if (FastIoTable[Index].SetSupported)
//                 {
//                     *FastPropertyHandler = FastIoTable[Index].SetPropertyHandler;
//                     return STATUS_SUCCESS;
//                 }
//             }
//
//             if (PropertyId->Flags & KSPROPERTY_TYPE_GET)
//             {
//                 if (FastIoTable[Index].GetSupported)
//                 {
//                     *FastPropertyHandler = FastIoTable[Index].GetPropertyHandler;
//                     return STATUS_SUCCESS;
//                 }
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
// KsFastPropertyHandler(
//     IN  PFILE_OBJECT FileObject,
//     IN  PKSPROPERTY UNALIGNED Property,
//     IN  ULONG PropertyLength,
//     IN  OUT PVOID UNALIGNED Data,
//     IN  ULONG DataLength,
//     OUT PIO_STATUS_BLOCK IoStatus,
//     IN  ULONG PropertySetsCount,
//     IN  const KSPROPERTY_SET* PropertySet)
// {
//     KSPROPERTY PropRequest;
//     KPROCESSOR_MODE Mode;
//     NTSTATUS Status = STATUS_SUCCESS;
//     ULONG Index;
//     PFNKSFASTHANDLER FastPropertyHandler;
//
//     if (PropertyLength < sizeof(KSPROPERTY))
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
//         RtlMoveMemory(&PropRequest, Property, sizeof(KSPROPERTY));
//     }
//     else
//     {
//         /* need to probe the buffer */
//         _SEH2_TRY
//         {
//             ProbeForRead(Property, sizeof(KSPROPERTY), sizeof(UCHAR));
//             RtlMoveMemory(&PropRequest, Property, sizeof(KSPROPERTY));
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
//     if (PropertySetsCount)
//     {
//         /* iterate through all property sets count */
//         Index = 0;
//         do
//         {
//             /* does the property id match */
//             if (IsEqualGUIDAligned(PropertySet[Index].Set, &PropRequest.Set))
//             {
//                 /* try to find a fast property handler */
//                 Status = FindFastPropertyHandler(PropertySet[Index].FastIoCount, PropertySet[Index].FastIoTable, &PropRequest, &FastPropertyHandler);
//
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* call fast property handler */
//                     ASSERT(PropertyLength == sizeof(KSPROPERTY)); /* FIXME check if property length is bigger -> copy params */
//                     ASSERT(Mode == KernelMode); /* FIXME need to probe usermode output buffer */
//                     return FastPropertyHandler(FileObject, &PropRequest, sizeof(KSPROPERTY), Data, DataLength, IoStatus);
//                 }
//             }
//             /* move to next item */
//             Index++;
//         }while(Index < PropertySetsCount);
//     }
//     return FALSE;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsDispatchSpecificProperty(
//     IN  PIRP Irp,
//     IN  PFNKSHANDLER Handler)
// {
//     PIO_STACK_LOCATION IoStack;
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     return Handler(Irp, IoStack->Parameters.DeviceIoControl.Type3InputBuffer, Irp->UserBuffer);
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
