use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::macros::*;

use ntkrnl_sys::ddk::{
    PZONE_HEADER, PZONE_SEGMENT_HEADER, ZONE_SEGMENT_HEADER,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

/// Inline replacement for the C macro PushEntryList.
#[inline(always)]
unsafe fn PushEntryList(ListHead: PSINGLE_LIST_ENTRY, Entry: PSINGLE_LIST_ENTRY) {
    (*Entry).Next = (*ListHead).Next;
    (*ListHead).Next = Entry;
}

#[no_mangle]
pub unsafe extern "system" fn ExExtendZone(
    Zone: PZONE_HEADER,
    Segment: PVOID,
    SegmentSize: ULONG,
) -> NTSTATUS {
    let Segment = opaque_ptr(Segment);
    let mut Entry: ULONG_PTR;
    let mut i: ULONG;

    if ((Segment as ULONG_PTR & 7) != 0) ||
       ((SegmentSize & 7) != 0) ||
       ((*Zone).BlockSize > SegmentSize)
    {
        return STATUS_INVALID_PARAMETER;
    }

    PushEntryList(
        core::ptr::addr_of_mut!((*Zone).SegmentList) as PSINGLE_LIST_ENTRY,
        core::ptr::addr_of_mut!((*(Segment as PZONE_SEGMENT_HEADER)).SegmentList) as PSINGLE_LIST_ENTRY,
    );

    Entry = Segment as ULONG_PTR + core::mem::size_of::<ZONE_SEGMENT_HEADER>();

    i = core::mem::size_of::<ZONE_SEGMENT_HEADER>() as ULONG;
    while i <= SegmentSize - (*Zone).BlockSize {
        PushEntryList(
            core::ptr::addr_of_mut!((*Zone).FreeList) as PSINGLE_LIST_ENTRY,
            Entry as PSINGLE_LIST_ENTRY,
        );

        Entry += (*Zone).BlockSize as ULONG_PTR;
        i += (*Zone).BlockSize;
    }

    (*Zone).TotalSegmentSize += i;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ExInterlockedExtendZone(
    Zone: PZONE_HEADER,
    Segment: PVOID,
    SegmentSize: ULONG,
    Lock: PKSPIN_LOCK,
) -> NTSTATUS {
    let Segment = opaque_ptr(Segment);
    let Status: NTSTATUS;
    let mut OldIrql: KIRQL = 0;

    // [guard] KeAcquireSpinLock(Lock, core::ptr::addr_of_mut!(OldIrql));
    let spin_guard = unsafe { ntkrnl_sys::guards::SpinLockGuard::acquire(Lock) };

    Status = ExExtendZone(Zone, Segment, SegmentSize);

    drop(spin_guard);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn ExInitializeZone(
    Zone: PZONE_HEADER,
    BlockSize: ULONG,
    InitialSegment: PVOID,
    InitialSegmentSize: ULONG,
) -> NTSTATUS {
    let InitialSegment = opaque_ptr(InitialSegment);
    let mut i: ULONG;
    let mut Entry: ULONG_PTR;

    if ((InitialSegment as ULONG_PTR & 7) != 0) ||
       ((InitialSegmentSize & 7) != 0) ||
       (BlockSize > InitialSegmentSize)
    {
        return STATUS_INVALID_PARAMETER;
    }

    (*Zone).BlockSize = BlockSize;

    (*Zone).FreeList.Next = core::ptr::null_mut();
    (*Zone).SegmentList.Next = core::ptr::null_mut();
    PushEntryList(
        core::ptr::addr_of_mut!((*Zone).SegmentList) as PSINGLE_LIST_ENTRY,
        core::ptr::addr_of_mut!((*(InitialSegment as PZONE_SEGMENT_HEADER)).SegmentList) as PSINGLE_LIST_ENTRY,
    );
    (*(InitialSegment as PZONE_SEGMENT_HEADER)).Reserved = core::ptr::null_mut();

    Entry = InitialSegment as ULONG_PTR + core::mem::size_of::<ZONE_SEGMENT_HEADER>();

    i = core::mem::size_of::<ZONE_SEGMENT_HEADER>() as ULONG;
    while i <= InitialSegmentSize - BlockSize {
        PushEntryList(
            core::ptr::addr_of_mut!((*Zone).FreeList) as PSINGLE_LIST_ENTRY,
            Entry as PSINGLE_LIST_ENTRY,
        );

        Entry += (*Zone).BlockSize as ULONG_PTR;
        i += (*Zone).BlockSize;
    }

    (*Zone).TotalSegmentSize = i;

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/ex/zone.c
//  * PURPOSE:         Implements zone buffers
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  *                  David Welch (welch@mcmail.com)
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ***************************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// ExExtendZone(PZONE_HEADER Zone,
//              PVOID Segment,
//              ULONG SegmentSize)
// {
//     ULONG_PTR Entry;
//     ULONG i;
//
//     /*
//      * BlockSize and Segment must be 8-byte aligned.
//      * Blocksize cannot exceed Segment Size.
//      */
//     if (((ULONG_PTR)Segment & 7) ||
//         (SegmentSize & 7) ||
//         (Zone->BlockSize > SegmentSize))
//     {
//         DPRINT1("Invalid ExExtendZone Alignment and/or Size\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Link the Zone and Segment */
//     PushEntryList(&Zone->SegmentList,
//                   &((PZONE_SEGMENT_HEADER)Segment)->SegmentList);
//
//     /* Get to the first entry */
//     Entry = (ULONG_PTR)Segment + sizeof(ZONE_SEGMENT_HEADER);
//
//     /* Loop through the segments */
//     for (i = sizeof(ZONE_SEGMENT_HEADER);
//          i <= SegmentSize - Zone->BlockSize;
//          i+= Zone->BlockSize)
//     {
//         /* Link the Free and Segment Lists */
//         PushEntryList(&Zone->FreeList, (PSINGLE_LIST_ENTRY)Entry);
//
//         /* Go to the next entry */
//         Entry += Zone->BlockSize;
//     }
//
//     /* Update Segment Size */
//     Zone->TotalSegmentSize += i;
//
//     /* Return Success */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// ExInterlockedExtendZone(PZONE_HEADER Zone,
//                         PVOID Segment,
//                         ULONG SegmentSize,
//                         PKSPIN_LOCK Lock)
// {
//     NTSTATUS Status;
//     KIRQL OldIrql;
//
//     /* Get the lock */
//     KeAcquireSpinLock(Lock, &OldIrql);
//
//     /* Extend the Zone */
//     Status = ExExtendZone(Zone, Segment, SegmentSize);
//
//     /* Release lock and return status */
//     KeReleaseSpinLock(Lock, OldIrql);
//     return Status;
// }
//
// /*
//  * FUNCTION: Initializes a zone header
//  * ARGUMENTS:
//  *          Zone = zone header to be initialized
//  *          BlockSize = Size (in bytes) of the allocation size of the zone
//  *          InitialSegment = Initial segment of storage allocated by the
//  *                           caller
//  *          InitialSegmentSize = Initial size of the segment
//  *
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// ExInitializeZone(PZONE_HEADER Zone,
//                  ULONG BlockSize,
//                  PVOID InitialSegment,
//                  ULONG InitialSegmentSize)
// {
//     ULONG i;
//     ULONG_PTR Entry;
//
//     /*
//      * BlockSize and Segment must be 8-byte aligned.
//      * Blocksize cannot exceed Segment Size.
//      */
//     if (((ULONG_PTR)InitialSegment & 7) ||
//         (InitialSegmentSize & 7) ||
//         (BlockSize > InitialSegmentSize))
//     {
//         DPRINT1("Invalid ExInitializeZone Alignment and/or Size\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Set the Zone Header */
//     Zone->BlockSize = BlockSize;
//
//     /* Link empty list */
//     Zone->FreeList.Next = NULL;
//     Zone->SegmentList.Next = NULL;
//     PushEntryList(&Zone->SegmentList,
//                   &((PZONE_SEGMENT_HEADER)InitialSegment)->SegmentList);
//     ((PZONE_SEGMENT_HEADER)InitialSegment)->Reserved = NULL;
//
//     /* Get first entry */
//     Entry = (ULONG_PTR)InitialSegment + sizeof(ZONE_SEGMENT_HEADER);
//
//     /* Loop through the segments */
//     for (i = sizeof(ZONE_SEGMENT_HEADER);
//          i <= InitialSegmentSize - BlockSize;
//          i+= BlockSize)
//     {
//         /* Link the Free and Segment Lists */
//         PushEntryList(&Zone->FreeList, (PSINGLE_LIST_ENTRY)Entry);
//
//         /* Go to the next entry */
//         Entry += Zone->BlockSize;
//     }
//
//     /* Set Segment Size */
//     Zone->TotalSegmentSize = i;
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
