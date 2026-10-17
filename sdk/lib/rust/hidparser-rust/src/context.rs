use ntkrnl_sys::base::*;


use crate::parser_h::{
    HID_COLLECTION, PHID_COLLECTION,
    HID_REPORT, PHID_REPORT,
    HID_REPORT_ITEM,
};

// ---- file-scope preamble (translated) ----
pub const NDEBUG: bool = true;

#[repr(C)]
pub union HID_COLLECTION_CONTEXT_anon_0 {
    pub RawData: [u8; 1],
}

#[repr(C)]
pub struct HID_COLLECTION_CONTEXT {
    pub Size: u32,
    pub __anon_0: HID_COLLECTION_CONTEXT_anon_0,
}

pub type PHID_COLLECTION_CONTEXT = *mut HID_COLLECTION_CONTEXT;

// Translated from context.c (grinder left it as an extern stub — body dropped).
#[no_mangle]
pub unsafe extern "C" fn HidParser_CalculateCollectionSize(Collection: *mut HID_COLLECTION) -> u32 {
    let mut Size: u32 = core::mem::size_of::<HID_COLLECTION>() as u32;

    // add size required for the number of report items
    let mut Index: u32 = 0;
    while Index < (*Collection).ReportCount {
        // ASSERT(Collection->Reports[Index]->ItemCount);
        let Report = *(*Collection).Reports.add(Index as usize);
        Size = Size.wrapping_add(
            (core::mem::size_of::<HID_REPORT>() as u32).wrapping_add(
                (*Report).ItemCount.wrapping_mul(core::mem::size_of::<HID_REPORT_ITEM>() as u32),
            ),
        );
        Index = Index.wrapping_add(1);
    }

    // calculate size for sub collections
    Index = 0;
    while Index < (*Collection).NodeCount {
        Size = Size.wrapping_add(HidParser_CalculateCollectionSize(
            *(*Collection).Nodes.add(Index as usize) as PHID_COLLECTION,
        ));
        Index = Index.wrapping_add(1);
    }

    // append size for the offset
    Size = Size.wrapping_add(
        ((*Collection).ReportCount.wrapping_add((*Collection).NodeCount))
            .wrapping_mul(core::mem::size_of::<ULONG>() as u32),
    );

    Size
}

// NTAPI (stdcall on i386): CopyFunction (hidparse.sys callback) + RtlAssert.
unsafe extern "system" {
    fn CopyFunction(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, size: u32);
    fn RtlAssert( FailedAssertion: *const core::ffi::c_void, FileName: *const core::ffi::c_void, LineNumber: ULONG, Message: *mut core::ffi::c_void, );
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_CalculateContextSize(Collection: *mut HID_COLLECTION) -> u32 {
    let mut Size: u32 = 0;
    Size = HidParser_CalculateCollectionSize(Collection);
    Size = Size.wrapping_add(core::mem::size_of::<HID_COLLECTION_CONTEXT>() as u32);
    return Size;
}

#[no_mangle]
pub unsafe extern "C" fn HidParser_StoreCollection(
    Collection: PHID_COLLECTION,
    CollectionContext: PHID_COLLECTION_CONTEXT,
    mut CurrentOffset: u32,
) -> u32 {
    let mut Index: u32 = 0;
    let mut ReportSize: u32 = 0;
    let mut InitialOffset: u32 = 0;
    let mut CollectionSize: u32 = 0;
    let mut TargetCollection: PHID_COLLECTION = core::ptr::null_mut::<HID_COLLECTION>();

    InitialOffset = CurrentOffset;
    TargetCollection = ((&raw mut (*CollectionContext).__anon_0.RawData) as *mut u8)
        .add(CurrentOffset as usize) as PHID_COLLECTION;

    CopyFunction(
        TargetCollection as *mut core::ffi::c_void,
        Collection as *mut core::ffi::c_void,
        core::mem::size_of::<HID_COLLECTION>() as u32,
    );

    CollectionSize = (core::mem::size_of::<HID_COLLECTION>() as u32).wrapping_add(
        (core::mem::size_of::<u32>() as u32).wrapping_mul(
            ((*Collection).ReportCount).wrapping_add((*Collection).NodeCount)
        )
    );

    CurrentOffset = CurrentOffset.wrapping_add(CollectionSize);

    if !(CurrentOffset < (*CollectionContext).Size) {
        RtlAssert(
            b"CurrentOffset < CollectionContext->Size\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/context.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            122u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }

    Index = 0u32;
    while Index < (*Collection).ReportCount {
        ReportSize = (core::mem::size_of::<HID_REPORT>() as u32).wrapping_add(
            ((*(*(*Collection).Reports.add(Index as usize))).ItemCount as u32).wrapping_mul(
                core::mem::size_of::<HID_REPORT_ITEM>() as u32,
            ),
        );

        if !(CurrentOffset.wrapping_add(ReportSize) < (*CollectionContext).Size) {
            RtlAssert(
                b"CurrentOffset + ReportSize < CollectionContext->Size\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/hidparser/context.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                137u32,
                core::ptr::null_mut::<core::ffi::c_void>(),
            );
        }

        CopyFunction(
            ((&raw mut (*CollectionContext).__anon_0.RawData) as *mut u8)
                .add(CurrentOffset as usize) as *mut core::ffi::c_void,
            *(*Collection).Reports.add(Index as usize) as *mut core::ffi::c_void,
            ReportSize,
        );

        *(&raw mut (*TargetCollection).Offsets as *mut u32).add(Index as usize) = CurrentOffset;
        CurrentOffset = CurrentOffset.wrapping_add(ReportSize);

        Index = Index.wrapping_add(1);
    }

    if !(CurrentOffset <= (*CollectionContext).Size) {
        RtlAssert(
            b"CurrentOffset <= CollectionContext->Size\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/context.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            155u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }

    Index = 0u32;
    while Index < (*Collection).NodeCount {
        *(&raw mut (*TargetCollection).Offsets as *mut u32)
            .add((*Collection).ReportCount.wrapping_add(Index) as usize) = CurrentOffset;

        CurrentOffset = CurrentOffset.wrapping_add(
            HidParser_StoreCollection(
                *(*Collection).Nodes.add(Index as usize) as PHID_COLLECTION,
                CollectionContext,
                CurrentOffset,
            )
        );

        if !(CurrentOffset < (*CollectionContext).Size) {
            RtlAssert(
                b"CurrentOffset < CollectionContext->Size\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"sdk/lib/drivers/hidparser/context.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                175u32,
                core::ptr::null_mut::<core::ffi::c_void>(),
            );
        }

        Index = Index.wrapping_add(1);
    }

    CurrentOffset.wrapping_sub(InitialOffset)
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_BuildCollectionContext(
    RootCollection: *mut HID_COLLECTION,
    Context: *mut core::ffi::c_void,
    ContextSize: u32,
) -> i32 {
    let CollectionContext: *mut HID_COLLECTION_CONTEXT;
    let CollectionSize: u32;

    CollectionContext = Context as *mut HID_COLLECTION_CONTEXT;
    (*CollectionContext).Size = ContextSize;
    CollectionSize = HidParser_StoreCollection(RootCollection, CollectionContext, 0u32);
    if !((CollectionSize as usize).wrapping_add(core::mem::size_of::<HID_COLLECTION_CONTEXT>()) == ContextSize as usize) {
        RtlAssert(
            b"CollectionSize + sizeof(HID_COLLECTION_CONTEXT) == ContextSize\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/context.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            207u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    // FIXED 2026-07-06: was `return 0; // HIDP_STATUS_SUCCESS` — but
    // HIDP_STATUS_SUCCESS is 0x00110000, NOT 0. Callers compare against the
    // real value, so 0 read as FAILURE in HidParser_GetCollectionDescription,
    // which then returned the raw 0 upward — hidclass saw "NT success" with
    // zero collections and never created kbd/mouse PDOs.
    return hid_sys::HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_SearchReportInCollection(
    CollectionContext: PHID_COLLECTION_CONTEXT,
    Collection: PHID_COLLECTION,
    ReportType: UCHAR,
) -> PHID_REPORT {
    let mut Index: ULONG = 0;
    let mut Report: PHID_REPORT = core::ptr::null_mut::<HID_REPORT>();
    let mut SubCollection: PHID_COLLECTION = core::ptr::null_mut::<HID_COLLECTION>();
    Index = 0 as ULONG;
    while Index < (*Collection).ReportCount {
        Report = (&raw mut (*CollectionContext).__anon_0.RawData as *mut UCHAR)
            .offset(*(&raw mut (*Collection).Offsets as *mut ULONG).offset(Index as isize) as isize)
            as PHID_REPORT;
        if (*Report).Type as core::ffi::c_int == ReportType as core::ffi::c_int {
            return Report;
        }
        Index = Index.wrapping_add(1);
    }
    Index = 0 as ULONG;
    while Index < (*Collection).NodeCount {
        SubCollection = (&raw mut (*CollectionContext).__anon_0.RawData as *mut UCHAR).offset(
            *(&raw mut (*Collection).Offsets as *mut ULONG)
                .offset((*Collection).ReportCount.wrapping_add(Index) as isize)
                as isize,
        ) as PHID_COLLECTION;
        Report = HidParser_SearchReportInCollection(CollectionContext, SubCollection, ReportType);
        if !Report.is_null() {
            return Report;
        }
        Index = Index.wrapping_add(1);
    }
    return core::ptr::null_mut::<HID_REPORT>();
}
/// Calls `f` on every report stored in a collection context: a collection's own reports
/// before its sub-collections', which is the order the descriptor declared them.
pub unsafe fn HidParser_VisitReports(Context: PVOID, f: &mut dyn FnMut(PHID_REPORT)) {
    let CollectionContext = Context as PHID_COLLECTION_CONTEXT;
    visit_reports(
        CollectionContext,
        &raw mut (*CollectionContext).__anon_0.RawData as PHID_COLLECTION,
        f,
    );
}

unsafe fn visit_reports(
    CollectionContext: PHID_COLLECTION_CONTEXT,
    Collection: PHID_COLLECTION,
    f: &mut dyn FnMut(PHID_REPORT),
) {
    let base = &raw mut (*CollectionContext).__anon_0.RawData as *mut UCHAR;
    let offsets = &raw mut (*Collection).Offsets as *mut ULONG;
    for Index in 0..(*Collection).ReportCount {
        f(base.add(*offsets.add(Index as usize) as usize) as PHID_REPORT);
    }
    for Index in 0..(*Collection).NodeCount {
        let at = *offsets.add((*Collection).ReportCount.wrapping_add(Index) as usize);
        visit_reports(CollectionContext, base.add(at as usize) as PHID_COLLECTION, f);
    }
}

#[no_mangle]
pub unsafe extern "C" fn HidParser_GetReportInCollection(
    Context: PVOID,
    ReportType: UCHAR,
) -> PHID_REPORT {
    let CollectionContext: PHID_COLLECTION_CONTEXT = Context as PHID_COLLECTION_CONTEXT;
    return HidParser_SearchReportInCollection(
        CollectionContext,
        &raw mut (*CollectionContext).__anon_0.RawData as PHID_COLLECTION,
        ReportType,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetCollectionFromContext(
    Context: PVOID,
) -> PHID_COLLECTION {
    let CollectionContext: PHID_COLLECTION_CONTEXT = Context as PHID_COLLECTION_CONTEXT;
    // return root collection
    return &raw mut (*CollectionContext).__anon_0.RawData as *mut u8 as PHID_COLLECTION;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetCollectionCount(
    CollectionContext: *mut HID_COLLECTION_CONTEXT,
    Collection: *mut HID_COLLECTION,
) -> u32 {
    let mut Index: u32 = 0;
    let mut Count: u32 = (*Collection).NodeCount;
    let mut SubCollection: *mut HID_COLLECTION = core::ptr::null_mut();

    Index = 0;
    while Index < (*Collection).NodeCount {
        let base = core::ptr::addr_of!((*CollectionContext).__anon_0.RawData) as *const u8;
        let offset = *((*Collection).Offsets.as_ptr().add(((*Collection).ReportCount.wrapping_add(Index)) as usize));
        SubCollection = base.add(offset as usize) as *mut HID_COLLECTION;
        Count = Count.wrapping_add(HidParser_GetCollectionCount(CollectionContext, SubCollection));
        Index = Index.wrapping_add(1);
    }

    Count
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetTotalCollectionCount(
    Context: *mut core::ffi::c_void,
) -> u32 {
    unsafe {
        let mut CollectionContext: *mut HID_COLLECTION_CONTEXT = core::ptr::null_mut();
        CollectionContext = Context as *mut HID_COLLECTION_CONTEXT;
        return HidParser_GetCollectionCount(
            CollectionContext,
            &raw mut (*CollectionContext).__anon_0.RawData as *mut u8 as PHID_COLLECTION,
        );
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS HID Parser Library
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        lib/drivers/hidparser/context.c
//  * PURPOSE:     HID Parser
//  * PROGRAMMERS:
//  *              Michael Martin (michael.martin@reactos.org)
//  *              Johannes Anderwald (johannes.anderwald@reactos.org)
//  */
//
// #include "parser.h"
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct
// {
//     ULONG Size;
//     union
//     {
//         UCHAR RawData[1];
//     };
// }HID_COLLECTION_CONTEXT, *PHID_COLLECTION_CONTEXT;
//
// ULONG
// HidParser_CalculateCollectionSize(
//     IN PHID_COLLECTION Collection)
// {
//     ULONG Size = 0, Index;
//
//     Size = sizeof(HID_COLLECTION);
//
//     //
//     // add size required for the number of report items
//     //
//     for(Index = 0; Index < Collection->ReportCount; Index++)
//     {
//         //
//         // get report size
//         //
//         ASSERT(Collection->Reports[Index]->ItemCount);
//         Size += sizeof(HID_REPORT) + Collection->Reports[Index]->ItemCount * sizeof(HID_REPORT_ITEM);
//     }
//
//     //
//     // calculate size for sub collections
//     //
//     for(Index = 0; Index < Collection->NodeCount; Index++)
//     {
//         Size += HidParser_CalculateCollectionSize(Collection->Nodes[Index]);
//     }
//
//     //
//     // append size for the offset
//     //
//     Size += (Collection->ReportCount + Collection->NodeCount) * sizeof(ULONG);
//
//     //
//     // done
//     //
//     return Size;
// }
//
// ULONG
// HidParser_CalculateContextSize(
//     IN PHID_COLLECTION Collection)
// {
//     ULONG Size;
//
//     //
//     // minimum size is the size of the collection
//     //
//     Size = HidParser_CalculateCollectionSize(Collection);
//
//     //
//     // append collection context size
//     //
//     Size += sizeof(HID_COLLECTION_CONTEXT);
//     return Size;
// }
//
// ULONG
// HidParser_StoreCollection(
//     IN PHID_COLLECTION Collection,
//     IN PHID_COLLECTION_CONTEXT CollectionContext,
//     IN ULONG CurrentOffset)
// {
//     ULONG Index;
//     ULONG ReportSize;
//     ULONG InitialOffset;
//     ULONG CollectionSize;
//     PHID_COLLECTION TargetCollection;
//
//     //
//     // backup initial offset
//     //
//     InitialOffset = CurrentOffset;
//
//     //
//     // get target collection
//     //
//     TargetCollection = (PHID_COLLECTION)(&CollectionContext->RawData[CurrentOffset]);
//
//     //
//     // first copy the collection details
//     //
//     CopyFunction(TargetCollection, Collection, sizeof(HID_COLLECTION));
//
//     //
//     // calulcate collection size
//     //
//     CollectionSize = sizeof(HID_COLLECTION) + sizeof(ULONG) * (Collection->ReportCount + Collection->NodeCount);
//
//     //
//     // increase offset
//     //
//     CurrentOffset += CollectionSize;
//
//     //
//     // sanity check
//     //
//     ASSERT(CurrentOffset < CollectionContext->Size);
//
//     //
//     // first store the report items
//     //
//     for(Index = 0; Index < Collection->ReportCount; Index++)
//     {
//         //
//         // calculate report size
//         //
//         ReportSize = sizeof(HID_REPORT) + Collection->Reports[Index]->ItemCount * sizeof(HID_REPORT_ITEM);
//
//         //
//         // sanity check
//         //
//         ASSERT(CurrentOffset + ReportSize < CollectionContext->Size);
//
//         //
//         // copy report item
//         //
//         CopyFunction(&CollectionContext->RawData[CurrentOffset], Collection->Reports[Index], ReportSize);
//
//         //
//         // store offset to report item
//         //
//         TargetCollection->Offsets[Index] = CurrentOffset;
//
//         //
//         // move to next offset
//         //
//         CurrentOffset += ReportSize;
//     }
//
//     ASSERT(CurrentOffset <= CollectionContext->Size);
//
//     //
//     // now store the sub collections
//     //
//     for(Index = 0; Index < Collection->NodeCount; Index++)
//     {
//         //
//         // store offset
//         //
//         TargetCollection->Offsets[Collection->ReportCount + Index] = CurrentOffset;
//
//         //
//         // store sub collections
//         //
//         CurrentOffset += HidParser_StoreCollection(Collection->Nodes[Index], CollectionContext, CurrentOffset);
//
//         //
//         // sanity check
//         //
//         ASSERT(CurrentOffset < CollectionContext->Size);
//     }
//
//     //
//     // return size of collection
//     //
//     return CurrentOffset - InitialOffset;
// }
//
// NTSTATUS
// HidParser_BuildCollectionContext(
//     IN PHID_COLLECTION RootCollection,
//     IN PVOID Context,
//     IN ULONG ContextSize)
// {
//     PHID_COLLECTION_CONTEXT CollectionContext;
//     ULONG CollectionSize;
//
//     //
//     // init context
//     //
//     CollectionContext = (PHID_COLLECTION_CONTEXT)Context;
//     CollectionContext->Size = ContextSize;
//
//     //
//     // store collections
//     //
//     CollectionSize = HidParser_StoreCollection(RootCollection, CollectionContext, 0);
//
//     //
//     // sanity check
//     //
//     ASSERT(CollectionSize + sizeof(HID_COLLECTION_CONTEXT) == ContextSize);
//
//     DPRINT("CollectionContext %p\n", CollectionContext);
//     DPRINT("CollectionContext RawData %p\n", CollectionContext->RawData);
//     DPRINT("CollectionContext Size %lu\n", CollectionContext->Size);
//
//     //
//     // done
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// PHID_REPORT
// HidParser_SearchReportInCollection(
//     IN PHID_COLLECTION_CONTEXT CollectionContext,
//     IN PHID_COLLECTION Collection,
//     IN UCHAR ReportType)
// {
//     ULONG Index;
//     PHID_REPORT Report;
//     PHID_COLLECTION SubCollection;
//
//     //
//     // search first in local array
//     //
//     for(Index = 0; Index < Collection->ReportCount; Index++)
//     {
//         //
//         // get report
//         //
//         Report = (PHID_REPORT)(CollectionContext->RawData + Collection->Offsets[Index]);
//         if (Report->Type == ReportType)
//         {
//             //
//             // found report
//             //
//             return Report;
//         }
//     }
//
//     //
//     // now search in sub collections
//     //
//     for(Index = 0; Index < Collection->NodeCount; Index++)
//     {
//         //
//         // get collection
//         //
//         SubCollection = (PHID_COLLECTION)(CollectionContext->RawData + Collection->Offsets[Collection->ReportCount + Index]);
//
//         //
//         // recursively search collection
//         //
//         Report = HidParser_SearchReportInCollection(CollectionContext, SubCollection, ReportType);
//         if (Report)
//         {
//             //
//             // found report
//             //
//             return Report;
//         }
//     }
//
//     //
//     // not found
//     //
//     return NULL;
// }
//
// PHID_REPORT
// HidParser_GetReportInCollection(
//     IN PVOID Context,
//     IN UCHAR ReportType)
// {
//     PHID_COLLECTION_CONTEXT CollectionContext = (PHID_COLLECTION_CONTEXT)Context;
//
//     //
//     // done
//     //
//     return HidParser_SearchReportInCollection(CollectionContext, (PHID_COLLECTION)&CollectionContext->RawData, ReportType);
// }
//
// PHID_COLLECTION
// HidParser_GetCollectionFromContext(
//     IN PVOID Context)
// {
//     PHID_COLLECTION_CONTEXT CollectionContext = (PHID_COLLECTION_CONTEXT)Context;
//
//     //
//     // return root collection
//     //
//     return (PHID_COLLECTION)CollectionContext->RawData;
// }
//
// ULONG
// HidParser_GetCollectionCount(
//     IN PHID_COLLECTION_CONTEXT CollectionContext,
//     IN PHID_COLLECTION Collection)
// {
//     ULONG Index;
//     ULONG Count = Collection->NodeCount;
//     PHID_COLLECTION SubCollection;
//
//     for(Index = 0; Index < Collection->NodeCount; Index++)
//     {
//         //
//         // get offset to sub collection
//         //
//         SubCollection = (PHID_COLLECTION)(CollectionContext->RawData + Collection->Offsets[Collection->ReportCount + Index]);
//
//         //
//         // count collection for sub nodes
//         //
//         Count += HidParser_GetCollectionCount(CollectionContext, SubCollection);
//     }
//
//     //
//     // done
//     //
//     return Count;
// }
//
// ULONG
// HidParser_GetTotalCollectionCount(
//     IN PVOID Context)
// {
//     PHID_COLLECTION_CONTEXT CollectionContext;
//
//     //
//     // get parser context
//     //
//     CollectionContext = (PHID_COLLECTION_CONTEXT)Context;
//
//     //
//     // count collections
//     //
//     return HidParser_GetCollectionCount(CollectionContext, (PHID_COLLECTION)CollectionContext->RawData);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
