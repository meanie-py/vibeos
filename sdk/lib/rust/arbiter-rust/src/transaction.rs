#[allow(unused_imports)]
use crate::prelude::*;
// rustconv-assembly: whole-tu
use crate::entry::ArbiterLibSortArbitrationList;
use ntkrnl_sys::ddk::{ARBITER_ALLOCATION_STATE, ARBITER_ALTERNATIVE, ARBITER_CONFLICT_INFO, ARBITER_LIST_ENTRY, INT32, IoGetDeviceProperty, PARBITER_ALLOCATION_STATE, PARBITER_ALTERNATIVE, PARBITER_CONFLICT_INFO, PARBITER_INSTANCE, PARBITER_LIST_ENTRY, PDEVICE_OBJECT, PIO_RESOURCE_DESCRIPTOR, PRTL_CONFLICT_RANGE_CALLBACK, PRTL_RANGE, PRTL_RANGE_LIST, RtlCopyRangeList, RtlDeleteOwnersRanges, RtlFreeRangeList, RtlInitializeRangeList, UINT64};

// ---- file-scope preamble (translated) ----
// No file-scope declarations: the preamble contains only function definitions and `#define NDEBUG` (valueless, so no constant is emitted).
unsafe extern "C" fn ArbpBuildAlternative(
    Arbiter: PARBITER_INSTANCE,
    Descriptor: PIO_RESOURCE_DESCRIPTOR,
    Alternative: PARBITER_ALTERNATIVE,
) -> NTSTATUS {
    let mut Length: UINT64 = 0;
    let mut Alignment: UINT64 = 0;
    let Status: NTSTATUS;

    (*Alternative).Descriptor = Descriptor;
    Status = ((*Arbiter).UnpackRequirement.unwrap_unchecked())(
        Descriptor,
        core::ptr::addr_of_mut!((*Alternative).Minimum),
        core::ptr::addr_of_mut!((*Alternative).Maximum),
        core::ptr::addr_of_mut!(Length),
        core::ptr::addr_of_mut!(Alignment),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*Alternative).Length = Length;
    (*Alternative).Alignment = Alignment;

    if Alignment != 0
        && ((*Alternative).Minimum.wrapping_rem(Alignment)) != 0
    {
        (*Alternative).Minimum = (*Alternative)
            .Minimum
            .wrapping_add(Alignment.wrapping_sub((*Alternative).Minimum.wrapping_rem(Alignment)));
    }

    (*Alternative).Flags = 0;
    (*Alternative).Priority = ARBITER_PRIORITY_NULL as i32;

    if (*Descriptor).ShareDisposition as ::core::ffi::c_int
        == CmResourceShareShared as ::core::ffi::c_int
    {
        (*Alternative).Flags |= ARBITER_ALTERNATIVE_FLAG_SHARED;
    }

    if (*Alternative).Maximum < (*Alternative).Minimum {
        (*Alternative).Flags |= ARBITER_ALTERNATIVE_FLAG_BADRANGE;
    } else if (*Alternative)
        .Maximum
        .wrapping_sub((*Alternative).Minimum)
        .wrapping_add(1)
        == (*Alternative).Length
    {
        (*Alternative).Flags |= ARBITER_ALTERNATIVE_FLAG_FIXED;
    }

    if ((*Descriptor).Type as ::core::ffi::c_int == CmResourceTypeMemory as ::core::ffi::c_int
        || (*Descriptor).Type as ::core::ffi::c_int == CmResourceTypeMemoryLarge as ::core::ffi::c_int)
        && (((*Descriptor).Flags as ::core::ffi::c_int & (CM_RESOURCE_MEMORY_COMPAT_FOR_INACCESSIBLE_RANGE as i32)) != 0)
    {
        (*Alternative).Flags |= ARBITER_ALTERNATIVE_FLAG_INACCESSIBLE_OK;
    }

    return STATUS_SUCCESS;
}
unsafe extern "C" fn ArbpBuildAllocationStack(
    Arbiter: PARBITER_INSTANCE,
    ArbitrationList: PLIST_ENTRY,
    EntryCount: ULONG,
) -> NTSTATUS {
    let mut State: PARBITER_ALLOCATION_STATE;
    let mut Alternative: PARBITER_ALTERNATIVE;
    let mut ListEntry: PLIST_ENTRY;
    let mut StateCount: ULONG = EntryCount.wrapping_add(1);
    let mut AlternativeCount: ULONG = 0;
    let Size: ULONG;

    PAGED_CODE!();

    ListEntry = (*ArbitrationList).Flink;
    while ListEntry != ArbitrationList {
        let Entry: PARBITER_LIST_ENTRY =
            CONTAINING_RECORD!(ListEntry, ARBITER_LIST_ENTRY, ListEntry);

        if (*Entry).AlternativeCount == 0 {
            StateCount = StateCount.wrapping_sub(1);
        } else {
            AlternativeCount = AlternativeCount.wrapping_add((*Entry).AlternativeCount);
        }
        ListEntry = (*ListEntry).Flink;
    }

    Size = (StateCount as usize)
        .wrapping_mul(core::mem::size_of::<ARBITER_ALLOCATION_STATE>())
        .wrapping_add(
            (AlternativeCount as usize)
                .wrapping_mul(core::mem::size_of::<ARBITER_ALTERNATIVE>()),
        ) as ULONG;

    if (*Arbiter).AllocationStackMaxSize < Size {
        let NewStack: PARBITER_ALLOCATION_STATE;

        NewStack = ExAllocatePoolWithTag(
            PagedPool,
            Size as SIZE_T,
            TAG_ARBITER,
        ) as PARBITER_ALLOCATION_STATE;
        if NewStack.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        if !(*Arbiter).AllocationStack.is_null() {
            ExFreePoolWithTag(
                (*Arbiter).AllocationStack as PVOID,
                TAG_ARBITER,
            );
        }

        (*Arbiter).AllocationStack = NewStack;
        (*Arbiter).AllocationStackMaxSize = Size;
    }
    core::ptr::write_bytes((*Arbiter).AllocationStack as *mut u8, 0, Size as usize);

    State = (*Arbiter).AllocationStack;
    Alternative = (*Arbiter).AllocationStack.add(StateCount as usize) as PARBITER_ALTERNATIVE;

    ListEntry = (*ArbitrationList).Flink;
    while ListEntry != ArbitrationList {
        let Entry: PARBITER_LIST_ENTRY =
            CONTAINING_RECORD!(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
        let mut Index: ULONG;

        if (*Entry).AlternativeCount == 0 {
            ListEntry = (*ListEntry).Flink;
            continue;
        }

        (*State).Entry = Entry;
        (*State).AlternativeCount = (*Entry).AlternativeCount;
        (*State).Alternatives = Alternative;
        (*State).Start = 1;

        Index = 0;
        while Index < (*Entry).AlternativeCount {
            let Status: NTSTATUS = ArbpBuildAlternative(
                Arbiter,
                (*Entry).Alternatives.add(Index as usize),
                Alternative,
            );
            if !NT_SUCCESS(Status) {
                return Status;
            }
            Alternative = Alternative.add(1);
            Index = Index.wrapping_add(1);
        }
        State = State.add(1);

        ListEntry = (*ListEntry).Flink;
    }

    STATUS_SUCCESS
}
unsafe extern "C" fn ArbpTestAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbitrationList: PLIST_ENTRY,
) -> NTSTATUS {
    let mut ListEntry: PLIST_ENTRY;
    let mut PreviousOwner: PVOID = core::ptr::null_mut();
    let mut EntryCount: ULONG = 0;
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    'failure: {
        RtlFreeRangeList((*Arbiter).PossibleAllocation);
        RtlInitializeRangeList((*Arbiter).PossibleAllocation);
        Status = RtlCopyRangeList((*Arbiter).PossibleAllocation, (*Arbiter).Allocation);
        if !NT_SUCCESS(Status) {
            break 'failure;
        }

        ListEntry = (*ArbitrationList).Flink;
        while ListEntry != ArbitrationList {
            let Entry: PARBITER_LIST_ENTRY =
                CONTAINING_RECORD!(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
            let mut Index: ULONG;

            EntryCount = EntryCount.wrapping_add(1);

            if (*Entry).PhysicalDeviceObject as PVOID != PreviousOwner {
                PreviousOwner = (*Entry).PhysicalDeviceObject as PVOID;
                RtlDeleteOwnersRanges((*Arbiter).PossibleAllocation, (*Entry).PhysicalDeviceObject as PVOID);
            }

            (*Entry).WorkSpace = 0;
            if let Some(score_requirement) = (*Arbiter).ScoreRequirement {
                Index = 0;
                while Index < (*Entry).AlternativeCount {
                    let Score: INT32 = score_requirement(
                        (*Entry).Alternatives.offset(Index as isize) as PIO_RESOURCE_DESCRIPTOR,
                    );
                    if Score < 0 {
                        Status = STATUS_DEVICE_CONFIGURATION_ERROR;
                        break 'failure;
                    }
                    (*Entry).WorkSpace += Score as LONG_PTR;
                    Index = Index.wrapping_add(1);
                }
            }

            ListEntry = (*ListEntry).Flink;
        }

        ArbiterLibSortArbitrationList(ArbitrationList);

        Status = ArbpBuildAllocationStack(Arbiter, ArbitrationList, EntryCount);
        if !NT_SUCCESS(Status) {
            break 'failure;
        }

        Status = (*Arbiter).AllocateEntry.unwrap_unchecked()(
            Arbiter,
            (*Arbiter).AllocationStack,
        );
        if !NT_SUCCESS(Status) {
            break 'failure;
        }

        return STATUS_SUCCESS;
    }

    RtlFreeRangeList((*Arbiter).PossibleAllocation);
    RtlInitializeRangeList((*Arbiter).PossibleAllocation);
    return Status;
}
unsafe extern "C" fn ArbpRetestAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbitrationList: PLIST_ENTRY,
) -> NTSTATUS {
    let mut State: ARBITER_ALLOCATION_STATE = core::mem::zeroed();
    let mut Alternative: ARBITER_ALTERNATIVE = core::mem::zeroed();
    let mut ListEntry: PLIST_ENTRY;
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    core::ptr::write_bytes(core::ptr::addr_of_mut!(State), 0, 1);
    core::ptr::write_bytes(core::ptr::addr_of_mut!(Alternative), 0, 1);
    State.Alternatives = core::ptr::addr_of_mut!(Alternative);
    State.CurrentAlternative = core::ptr::addr_of_mut!(Alternative);
    State.AlternativeCount = 1;

    RtlFreeRangeList((*Arbiter).PossibleAllocation);
    RtlInitializeRangeList((*Arbiter).PossibleAllocation);
    Status = RtlCopyRangeList((*Arbiter).PossibleAllocation, (*Arbiter).Allocation);

    'body: {
        if !NT_SUCCESS(Status) {
            break 'body;
        }

        ListEntry = (*ArbitrationList).Flink;
        while ListEntry != ArbitrationList {
            let Entry: PARBITER_LIST_ENTRY =
                CONTAINING_RECORD!(ListEntry, ARBITER_LIST_ENTRY, ListEntry);

            Status = RtlDeleteOwnersRanges((*Arbiter).PossibleAllocation, (*Entry).PhysicalDeviceObject as PVOID);
            if !NT_SUCCESS(Status) {
                break 'body;
            }
            ListEntry = (*ListEntry).Flink;
        }

        ListEntry = (*ArbitrationList).Flink;
        while ListEntry != ArbitrationList {
            let Entry: PARBITER_LIST_ENTRY =
                CONTAINING_RECORD!(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
            let mut Length: UINT64 = 0;

            if (*Entry).Result == ArbiterResultNullRequest {
                ListEntry = (*ListEntry).Flink;
                continue;
            }

            if (*Entry).SelectedAlternative.is_null() || (*Entry).Assignment.is_null() {
                Status = STATUS_INVALID_PARAMETER;
                break 'body;
            }

            Status = ArbpBuildAlternative(
                Arbiter,
                (*Entry).SelectedAlternative,
                core::ptr::addr_of_mut!(Alternative),
            );
            if !NT_SUCCESS(Status) {
                break 'body;
            }

            State.Entry = Entry;
            State.WorkSpace = 0;

            Status = (*Arbiter).UnpackResource.unwrap_unchecked()(
                (*Entry).Assignment,
                core::ptr::addr_of_mut!(State.Start),
                core::ptr::addr_of_mut!(Length),
            );
            if !NT_SUCCESS(Status) {
                break 'body;
            }
            State.End = State.Start + Length - 1;

            Status = (*Arbiter).PreprocessEntry.unwrap_unchecked()(
                Arbiter,
                core::ptr::addr_of_mut!(State),
            );
            if !NT_SUCCESS(Status) {
                break 'body;
            }

            if Length != 0 {
                (*Arbiter).AddAllocation.unwrap_unchecked()(
                    Arbiter,
                    core::ptr::addr_of_mut!(State),
                );
            }

            if (State.Flags as ::core::ffi::c_int) & (ARBITER_STATE_FLAG_WORKSPACE as i32) != 0 {
                ExFreePoolWithTag(State.WorkSpace as PVOID, TAG_ARBITER);
                State.WorkSpace = 0;
                State.Flags = ((State.Flags as ::core::ffi::c_int) & (!ARBITER_STATE_FLAG_WORKSPACE) as i32) as UINT16;
            }

            ListEntry = (*ListEntry).Flink;
        }

        return STATUS_SUCCESS;
    }

    // Failure:
    if (State.Flags as ::core::ffi::c_int) & (ARBITER_STATE_FLAG_WORKSPACE as i32) != 0 {
        ExFreePoolWithTag(State.WorkSpace as PVOID, TAG_ARBITER);
        // As the C does; State is a local never read again, so rustc calls these dead.
        #[allow(unused_assignments)]
        {
            State.WorkSpace = 0;
            State.Flags = ((State.Flags as ::core::ffi::c_int) & (!ARBITER_STATE_FLAG_WORKSPACE) as i32) as UINT16;
        }
    }
    RtlFreeRangeList((*Arbiter).PossibleAllocation);
    RtlInitializeRangeList((*Arbiter).PossibleAllocation);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibCommitAllocation(Arbiter: PARBITER_INSTANCE) -> NTSTATUS {
    let Old: PRTL_RANGE_LIST = (*Arbiter).Allocation;

    PAGED_CODE!();

    RtlFreeRangeList(Old);
    RtlInitializeRangeList(Old);
    (*Arbiter).Allocation = (*Arbiter).PossibleAllocation;
    (*Arbiter).PossibleAllocation = Old;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibRollbackAllocation(Arbiter: PARBITER_INSTANCE) -> NTSTATUS {
    PAGED_CODE!();

    RtlFreeRangeList((*Arbiter).PossibleAllocation);
    RtlInitializeRangeList((*Arbiter).PossibleAllocation);
    return STATUS_SUCCESS;
}
unsafe extern "C" fn ArbpBootAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbitrationList: PLIST_ENTRY,
) -> NTSTATUS {
    let mut ListEntry: PLIST_ENTRY;
    let mut State: ARBITER_ALLOCATION_STATE = core::mem::zeroed();
    let mut Alternative: ARBITER_ALTERNATIVE = core::mem::zeroed();
    let Old: PRTL_RANGE_LIST;
    let Status: NTSTATUS;

    PAGED_CODE!();

    RtlFreeRangeList((*Arbiter).PossibleAllocation);
    RtlInitializeRangeList((*Arbiter).PossibleAllocation);
    Status = RtlCopyRangeList((*Arbiter).PossibleAllocation, (*Arbiter).Allocation);
    if !NT_SUCCESS(Status) {
        RtlFreeRangeList((*Arbiter).PossibleAllocation);
        RtlInitializeRangeList((*Arbiter).PossibleAllocation);
        return Status;
    }

    ListEntry = (*ArbitrationList).Flink;
    loop {
        if ListEntry == ArbitrationList {
            break;
        }
        'body: {
            let Entry: PARBITER_LIST_ENTRY = CONTAINING_RECORD!(ListEntry, ARBITER_LIST_ENTRY, ListEntry);

            if (*Entry).AlternativeCount == 0 {
                break 'body;
            }

            core::ptr::write_bytes(core::ptr::addr_of_mut!(State), 0, 1);
            core::ptr::write_bytes(core::ptr::addr_of_mut!(Alternative), 0, 1);

            if !NT_SUCCESS(ArbpBuildAlternative(
                Arbiter,
                (*Entry).Alternatives.add(0 as usize),
                core::ptr::addr_of_mut!(Alternative),
            )) {
                break 'body;
            }

            if Alternative.Length == 0
                || Alternative.Alignment == 0
                || (Alternative.Flags & ARBITER_ALTERNATIVE_FLAG_BADRANGE) != 0
                || (Alternative.Flags & ARBITER_ALTERNATIVE_FLAG_FIXED) == 0
                || (Alternative.Minimum % Alternative.Alignment) != 0
            {
                break 'body;
            }

            State.Entry = Entry;
            State.AlternativeCount = 1;
            State.Alternatives = core::ptr::addr_of_mut!(Alternative);
            State.CurrentAlternative = core::ptr::addr_of_mut!(Alternative);
            State.Flags = ARBITER_STATE_FLAG_BOOT as u16;
            State.RangeAttributes = ARBITER_RANGE_BOOT_ALLOCATED as u8;
            State.Start = Alternative.Minimum;
            State.CurrentMinimum = State.Start;
            State.End = Alternative.Maximum;
            State.CurrentMaximum = State.End;

            if !NT_SUCCESS((*Arbiter).PreprocessEntry.unwrap_unchecked()(
                Arbiter,
                core::ptr::addr_of_mut!(State),
            )) {
                break 'body;
            }

            (*Arbiter).AddAllocation.unwrap_unchecked()(
                Arbiter,
                core::ptr::addr_of_mut!(State),
            );

            if (*Arbiter).PackResource.is_some() && !(*Entry).Assignment.is_null() {
                let _ = (*Arbiter).PackResource.unwrap_unchecked()(
                    Alternative.Descriptor,
                    State.Start,
                    (*Entry).Assignment,
                );
            }
        }
        ListEntry = (*ListEntry).Flink;
    }

    Old = (*Arbiter).Allocation;
    RtlFreeRangeList(Old);
    RtlInitializeRangeList(Old);
    (*Arbiter).Allocation = (*Arbiter).PossibleAllocation;
    (*Arbiter).PossibleAllocation = Old;

    STATUS_SUCCESS
}
unsafe extern "system" fn ArbpQueryConflictCallback(Context: PVOID, Range: PRTL_RANGE) -> BOOLEAN {
    *(Context as *mut PRTL_RANGE) = Range;
    return FALSE;
}
unsafe extern "C" fn ArbpQueryConflict(
    Arbiter: PARBITER_INSTANCE,
    PhysicalDeviceObject: PDEVICE_OBJECT,
    ConflictingResource: PIO_RESOURCE_DESCRIPTOR,
    ConflictCount: PULONG,
    Conflicts: *mut PARBITER_CONFLICT_INFO,
) -> NTSTATUS {
    let SavedCallback: PRTL_CONFLICT_RANGE_CALLBACK = (*Arbiter).ConflictCallback;
    let SavedContext: PVOID = (*Arbiter).ConflictCallbackContext;
    let mut ConflictingRange: PRTL_RANGE = core::ptr::null_mut();
    let mut State: ARBITER_ALLOCATION_STATE = core::mem::zeroed();
    let mut Entry: ARBITER_LIST_ENTRY = core::mem::zeroed();
    let mut Alternative: ARBITER_ALTERNATIVE = core::mem::zeroed();
    let mut List: PARBITER_CONFLICT_INFO;
    let mut Count: ULONG = 0;
    let mut Capacity: ULONG = 10;
    let mut ResultLength: ULONG = 0;
    let mut Status: NTSTATUS;

    *ConflictCount = 0;
    *Conflicts = core::ptr::null_mut();

    core::ptr::write_bytes(core::ptr::addr_of_mut!(State), 0, 1);

    (*Arbiter).ConflictCallback =
        Some(ArbpQueryConflictCallback);
    (*Arbiter).ConflictCallbackContext = core::ptr::addr_of_mut!(ConflictingRange) as PVOID;

    RtlFreeRangeList((*Arbiter).PossibleAllocation);
    RtlInitializeRangeList((*Arbiter).PossibleAllocation);

    'body: {
        Status = RtlCopyRangeList((*Arbiter).PossibleAllocation, (*Arbiter).Allocation);
        if !NT_SUCCESS(Status) {
            break 'body;
        }

        Status = ArbpBuildAlternative(
            Arbiter,
            ConflictingResource,
            core::ptr::addr_of_mut!(Alternative),
        );
        if !NT_SUCCESS(Status) {
            break 'body;
        }

        core::ptr::write_bytes(core::ptr::addr_of_mut!(Entry), 0, 1);
        State.CurrentMinimum = Alternative.Minimum;
        State.Start = State.CurrentMinimum;
        State.CurrentMaximum = Alternative.Maximum;
        State.End = State.CurrentMaximum;
        State.Alternatives = core::ptr::addr_of_mut!(Alternative) as PARBITER_ALTERNATIVE;
        State.CurrentAlternative = State.Alternatives;
        State.AlternativeCount = 1;
        Entry.RequestSource = ArbiterRequestPnpEnumerated;
        Entry.PhysicalDeviceObject = PhysicalDeviceObject;

        if !NT_SUCCESS(IoGetDeviceProperty(
            PhysicalDeviceObject,
            DevicePropertyLegacyBusType,
            core::mem::size_of_val(&Entry.InterfaceType) as ULONG,
            core::ptr::addr_of_mut!(Entry.InterfaceType) as PVOID,
            core::ptr::addr_of_mut!(ResultLength),
        )) {
            Entry.InterfaceType = Isa;
        }
        if !NT_SUCCESS(IoGetDeviceProperty(
            PhysicalDeviceObject,
            DevicePropertyBusNumber,
            core::mem::size_of_val(&Entry.BusNumber) as ULONG,
            core::ptr::addr_of_mut!(Entry.BusNumber) as PVOID,
            core::ptr::addr_of_mut!(ResultLength),
        )) {
            Entry.BusNumber = 0;
        }

        List = ExAllocatePoolWithTag(
            PagedPool,
            (Capacity as SIZE_T).wrapping_mul(core::mem::size_of::<ARBITER_CONFLICT_INFO>() as SIZE_T),
            TAG_ARBITER,
        ) as PARBITER_CONFLICT_INFO;
        if List.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'body;
        }

        // The C sets `State.Entry = &Entry` before filling Entry in. Here the pointer is taken
        // once Entry is complete: writing the local directly after its raw address is out
        // would invalidate that pointer under Stacked Borrows, and the writes looked dead to
        // rustc. Nothing reads State.Entry before this call.
        State.Entry = core::ptr::addr_of_mut!(Entry) as PARBITER_LIST_ENTRY;
        Status = (*Arbiter)
            .PreprocessEntry
            .unwrap_unchecked()(
            Arbiter,
            core::ptr::addr_of_mut!(State),
        );
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag(List as PVOID, TAG_ARBITER);
            break 'body;
        }

        /* The requester's own ranges are not conflicts. */
        RtlDeleteOwnersRanges((*Arbiter).PossibleAllocation, PhysicalDeviceObject as PVOID);

        loop {
            ConflictingRange = core::ptr::null_mut();
            State.CurrentMinimum = State.Start;
            State.CurrentMaximum = State.End;

            if (*Arbiter)
                .FindSuitableRange
                .unwrap_unchecked()(
                Arbiter,
                core::ptr::addr_of_mut!(State),
            ) != 0
            {
                break;
            }

            if Count == Capacity {
                let Grown: PARBITER_CONFLICT_INFO = ExAllocatePoolWithTag(
                    PagedPool,
                    ((Capacity + 5) as SIZE_T)
                        .wrapping_mul(core::mem::size_of::<ARBITER_CONFLICT_INFO>() as SIZE_T),
                    TAG_ARBITER,
                ) as PARBITER_CONFLICT_INFO;
                if Grown.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    ExFreePoolWithTag(List as PVOID, TAG_ARBITER);
                    break 'body;
                }
                core::ptr::copy_nonoverlapping(
                    List as *const u8,
                    Grown as *mut u8,
                    Count as usize * core::mem::size_of::<ARBITER_CONFLICT_INFO>(),
                );
                ExFreePoolWithTag(List as PVOID, TAG_ARBITER);
                List = Grown;
                Capacity += 5;
            }

            if ConflictingRange.is_null() {
                /* Blocked, but no specific owner: report one whole-range conflict. */
                (*List.add(Count as usize)).OwningObject = core::ptr::null_mut();
                (*List.add(Count as usize)).Start = 0;
                (*List.add(Count as usize)).End = ARBITER_MAXIMUM_ADDRESS;
                Count += 1;
                break;
            }

            (*List.add(Count as usize)).OwningObject =
                (*ConflictingRange).Owner as PDEVICE_OBJECT;
            (*List.add(Count as usize)).Start = (*ConflictingRange).Start;
            (*List.add(Count as usize)).End = (*ConflictingRange).End;
            Count += 1;

            /* Remove that owner and look for the next conflict. */
            Status = RtlDeleteOwnersRanges(
                (*Arbiter).PossibleAllocation,
                (*ConflictingRange).Owner,
            );
            if !NT_SUCCESS(Status) {
                ExFreePoolWithTag(List as PVOID, TAG_ARBITER);
                break 'body;
            }
        }

        *Conflicts = List;
        *ConflictCount = Count;
        Status = STATUS_SUCCESS;
    }

    /* Cleanup */
    if (State.Flags & (ARBITER_STATE_FLAG_WORKSPACE as UINT16)) != 0 {
        ExFreePoolWithTag(State.WorkSpace as PVOID, TAG_ARBITER);
        // As the C does; State is a local never read again, so rustc calls these dead.
        #[allow(unused_assignments)]
        {
            State.WorkSpace = 0;
            State.Flags &= !(ARBITER_STATE_FLAG_WORKSPACE as UINT16);
        }
    }
    RtlFreeRangeList((*Arbiter).PossibleAllocation);
    RtlInitializeRangeList((*Arbiter).PossibleAllocation);
    (*Arbiter).ConflictCallback = SavedCallback;
    (*Arbiter).ConflictCallbackContext = SavedContext;
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Arbitrartion Library
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Test/retest/commit/rollback transaction support
//  * COPYRIGHT:   Copyright 2026 Justin Miller <justin.miller@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntifs.h>
// #include <ndk/rtlfuncs.h>
// #include "arbiter.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* TRANSACTION SUPPORT ********************************************************/
//
// /**
//  * @brief
//  * Expands one IO_RESOURCE_DESCRIPTOR into an ARBITER_ALTERNATIVE
//  * through the arbiter's UnpackRequirement callback and identifies
//  * it.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose UnpackRequirement decodes the
//  * descriptor.
//  *
//  * @param[in] Descriptor
//  * The requirement descriptor to expand.
//  *
//  * @param[out] Alternative
//  * Receives the decoded window, length, alignment and the derived
//  * FIXED / SHARED / INVALID flags. The minimum is rounded up to the
//  * requested alignment first, so a misaligned window base does not
//  * later affect the range search.
//  *
//  * @return
//  * Returns STATUS_SUCCESS, or the UnpackRequirement failure status.
//  */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpBuildAlternative(
//          PARBITER_INSTANCE Arbiter,
//          PIO_RESOURCE_DESCRIPTOR Descriptor,
//           PARBITER_ALTERNATIVE Alternative)
// {
//     UINT64 Length, Alignment;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /*
//      * UnpackRequirement always writes 64-bit Length/Alignment; ARBITER_ALTERNATIVE
//      * narrows them to 32 bits pre-Vista, so decode through 64-bit temporaries and
//      * assign, just to keep both versions in source.
//      */
//     Alternative->Descriptor = Descriptor;
//     Status = Arbiter->UnpackRequirement(Descriptor,
//                                         &Alternative->Minimum,
//                                         &Alternative->Maximum,
//                                         &Length,
//                                         &Alignment);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
// #if (NTDDI_VERSION >= NTDDI_VISTA) || defined(__REACTOS__)
//     Alternative->Length = Length;
//     Alternative->Alignment = Alignment;
// #else
//     Alternative->Length = (UINT32)Length;
//     Alternative->Alignment = (UINT32)Alignment;
// #endif
//
//     if (Alignment != 0 && (Alternative->Minimum % Alignment) != 0)
//         Alternative->Minimum += Alignment - (Alternative->Minimum % Alignment);
//
//     Alternative->Flags = 0;
//     Alternative->Priority = ARBITER_PRIORITY_NULL;
//
//     if (Descriptor->ShareDisposition == CmResourceShareShared)
//         Alternative->Flags |= ARBITER_ALTERNATIVE_FLAG_SHARED;
//
//     if (Alternative->Maximum < Alternative->Minimum)
//         Alternative->Flags |= ARBITER_ALTERNATIVE_FLAG_BADRANGE;
//     else if ((Alternative->Maximum - Alternative->Minimum + 1) == Alternative->Length)
//         Alternative->Flags |= ARBITER_ALTERNATIVE_FLAG_FIXED;
//
//     if ((Descriptor->Type == CmResourceTypeMemory ||
//          Descriptor->Type == CmResourceTypeMemoryLarge) &&
//         (Descriptor->Flags & CM_RESOURCE_MEMORY_COMPAT_FOR_INACCESSIBLE_RANGE))
//     {
//         Alternative->Flags |= ARBITER_ALTERNATIVE_FLAG_INACCESSIBLE_OK;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Builds the allocation stack for an arbitration list into
//  * Arbiter->AllocationStack: one ARBITER_ALLOCATION_STATE per
//  * contributing entry, in list order, terminated by a state whose
//  * Entry is NULL, followed by the flattened ARBITER_ALTERNATIVE
//  * array the states point into.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose AllocationStack buffer is (re)used,
//  * growing it when the list needs more room.
//  *
//  * @param[in] ArbitrationList
//  * The (sorted) list of ARBITER_LIST_ENTRY nodes to flatten.
//  * Entries without alternatives contribute no state.
//  *
//  * @param[in] EntryCount
//  * The number of entries on the list.
//  *
//  * @return
//  * Returns STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES if the
//  * stack cannot grow, or an ArbpBuildAlternative failure status.
//  */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpBuildAllocationStack(
//          PARBITER_INSTANCE Arbiter,
//          PLIST_ENTRY ArbitrationList,
//          ULONG EntryCount)
// {
//     PARBITER_ALLOCATION_STATE State;
//     PARBITER_ALTERNATIVE Alternative;
//     PLIST_ENTRY ListEntry;
//     ULONG StateCount = EntryCount + 1;  /* + NULL terminator */
//     ULONG AlternativeCount = 0;
//     ULONG Size;
//
//     PAGED_CODE();
//
//     for (ListEntry = ArbitrationList->Flink;
//          ListEntry != ArbitrationList;
//          ListEntry = ListEntry->Flink)
//     {
//         PARBITER_LIST_ENTRY Entry = CONTAINING_RECORD(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
//
//         if (Entry->AlternativeCount == 0)
//             StateCount--;  /* an empty entry contributes no state */
//         else
//             AlternativeCount += Entry->AlternativeCount;
//     }
//
//     Size = StateCount * sizeof(ARBITER_ALLOCATION_STATE) +
//            AlternativeCount * sizeof(ARBITER_ALTERNATIVE);
//
//     if (Arbiter->AllocationStackMaxSize < Size)
//     {
//         PARBITER_ALLOCATION_STATE NewStack;
//
//         NewStack = ExAllocatePoolWithTag(PagedPool, Size, TAG_ARBITER);
//         if (NewStack == NULL)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         if (Arbiter->AllocationStack != NULL)
//             ExFreePoolWithTag(Arbiter->AllocationStack, TAG_ARBITER);
//
//         Arbiter->AllocationStack = NewStack;
//         Arbiter->AllocationStackMaxSize = Size;
//     }
//     RtlZeroMemory(Arbiter->AllocationStack, Size);
//
//     State = Arbiter->AllocationStack;
//     Alternative = (PARBITER_ALTERNATIVE)&Arbiter->AllocationStack[StateCount];
//
//     for (ListEntry = ArbitrationList->Flink;
//          ListEntry != ArbitrationList;
//          ListEntry = ListEntry->Flink)
//     {
//         PARBITER_LIST_ENTRY Entry = CONTAINING_RECORD(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
//         ULONG Index;
//
//         if (Entry->AlternativeCount == 0)
//             continue;
//
//         State->Entry = Entry;
//         State->AlternativeCount = Entry->AlternativeCount;
//         State->Alternatives = Alternative;
//         State->Start = 1;  /* Start(1) > End(0): nothing chosen yet */
//
//         for (Index = 0; Index < Entry->AlternativeCount; ++Index)
//         {
//             NTSTATUS Status = ArbpBuildAlternative(Arbiter, &Entry->Alternatives[Index], Alternative);
//             if (!NT_SUCCESS(Status))
//                 return Status;
//             Alternative++;
//         }
//         State++;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Test-allocates every entry on an arbitration list into the
//  * arbiter's tentative allocation. The shared implementation behind
//  * ArbiterLibAttemptAllocation.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose PossibleAllocation receives the
//  * tentative solution.
//  *
//  * @param[in] ArbitrationList
//  * The list of ARBITER_LIST_ENTRY nodes to place. Everything a
//  * listed device already owns is removed from the working list
//  * first, so a device does not conflict with itself (its boot
//  * configuration in particular).
//  *
//  * @return
//  * Returns STATUS_SUCCESS with the solution recorded in
//  * PossibleAllocation, STATUS_DEVICE_CONFIGURATION_ERROR for a
//  * malformed requirement, or the failure status of the stack build
//  * or the solver. On failure the tentative allocation is discarded.
//  */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpTestAllocation(
//          PARBITER_INSTANCE Arbiter,
//          PLIST_ENTRY ArbitrationList)
// {
//     PLIST_ENTRY ListEntry;
//     PVOID PreviousOwner = NULL;
//     ULONG EntryCount = 0;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Start the tentative allocation as a copy of the committed one. */
//     RtlFreeRangeList(Arbiter->PossibleAllocation);
//     RtlInitializeRangeList(Arbiter->PossibleAllocation);
//     Status = RtlCopyRangeList(Arbiter->PossibleAllocation, Arbiter->Allocation);
//     if (!NT_SUCCESS(Status))
//         goto Failure;
//
//     for (ListEntry = ArbitrationList->Flink;
//          ListEntry != ArbitrationList;
//          ListEntry = ListEntry->Flink)
//     {
//         PARBITER_LIST_ENTRY Entry = CONTAINING_RECORD(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
//         ULONG Index;
//
//         EntryCount++;
//
//         /*
//          * Everything a device on the arbitration list already owns is up for
//          * reassignment - remove it from the working list so the device does not
//          * conflict with itself (its boot config in particular).
//          */
//         if (Entry->PhysicalDeviceObject != PreviousOwner)
//         {
//             PreviousOwner = Entry->PhysicalDeviceObject;
//             RtlDeleteOwnersRanges(Arbiter->PossibleAllocation, Entry->PhysicalDeviceObject);
//         }
//
//         /* Score each entry: the sum of its alternatives' constrainedness. */
//         Entry->WorkSpace = 0;
//         if (Arbiter->ScoreRequirement != NULL)
//         {
//             for (Index = 0; Index < Entry->AlternativeCount; ++Index)
//             {
//                 INT32 Score = Arbiter->ScoreRequirement(&Entry->Alternatives[Index]);
//                 if (Score < 0)
//                 {
//                     Status = STATUS_DEVICE_CONFIGURATION_ERROR;
//                     goto Failure;
//                 }
//                 Entry->WorkSpace += Score;
//             }
//         }
//     }
//
//     ArbiterLibSortArbitrationList(ArbitrationList);
//
//     Status = ArbpBuildAllocationStack(Arbiter, ArbitrationList, EntryCount);
//     if (!NT_SUCCESS(Status))
//         goto Failure;
//
//     Status = Arbiter->AllocateEntry(Arbiter, Arbiter->AllocationStack);
//     if (!NT_SUCCESS(Status))
//         goto Failure;
//
//     return STATUS_SUCCESS;
//
// Failure:
//     RtlFreeRangeList(Arbiter->PossibleAllocation);
//     RtlInitializeRangeList(Arbiter->PossibleAllocation);
//     return Status;
// }
//
// /**
//  * @brief
//  * Re-establishes a previously tested solution in the arbiter's
//  * tentative allocation without searching again: every entry's
//  * SelectedAlternative is re-materialized at the position recorded
//  * in its Assignment.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose PossibleAllocation is rebuilt from
//  * the committed allocation.
//  *
//  * @param[in] ArbitrationList
//  * The list of entries from an earlier successful test: each must
//  * carry the SelectedAlternative and Assignment that test produced,
//  * except entries whose Result is ArbiterResultNullRequest, which
//  * are skipped.
//  *
//  * @return
//  * Returns STATUS_SUCCESS with the solution re-recorded,
//  * STATUS_INVALID_PARAMETER for an entry that was never tested, or
//  * the failure status of the copy, decode or preprocess step. On
//  * failure the tentative allocation is discarded.
//  *
//  * @remarks
//  * after every arbiter's Test has succeeded, each is re-driven
//  * through Retest so the exact chosen placements are laid down for the Commit that follows.
//  */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpRetestAllocation(
//          PARBITER_INSTANCE Arbiter,
//          PLIST_ENTRY ArbitrationList)
// {
//     ARBITER_ALLOCATION_STATE State;
//     ARBITER_ALTERNATIVE Alternative;
//     PLIST_ENTRY ListEntry;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     RtlZeroMemory(&State, sizeof(State));
//     RtlZeroMemory(&Alternative, sizeof(Alternative));
//     State.Alternatives = &Alternative;
//     State.CurrentAlternative = &Alternative;
//     State.AlternativeCount = 1;
//
//     /*
//      * Rebuild the tentative allocation from the committed one
//      * minus everything the listed devices already own.
//      */
//     RtlFreeRangeList(Arbiter->PossibleAllocation);
//     RtlInitializeRangeList(Arbiter->PossibleAllocation);
//     Status = RtlCopyRangeList(Arbiter->PossibleAllocation, Arbiter->Allocation);
//     if (!NT_SUCCESS(Status))
//         goto Failure;
//
//     for (ListEntry = ArbitrationList->Flink;
//          ListEntry != ArbitrationList;
//          ListEntry = ListEntry->Flink)
//     {
//         PARBITER_LIST_ENTRY Entry = CONTAINING_RECORD(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
//
//         Status = RtlDeleteOwnersRanges(Arbiter->PossibleAllocation,
//                                        Entry->PhysicalDeviceObject);
//         if (!NT_SUCCESS(Status))
//             goto Failure;
//     }
//
//     for (ListEntry = ArbitrationList->Flink;
//          ListEntry != ArbitrationList;
//          ListEntry = ListEntry->Flink)
//     {
//         PARBITER_LIST_ENTRY Entry = CONTAINING_RECORD(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
//         UINT64 Length;
//
//         if (Entry->Result == ArbiterResultNullRequest)
//             continue;
//
//         /* A retest without a preceding successful test is a caller bug. */
//         if (Entry->SelectedAlternative == NULL || Entry->Assignment == NULL)
//         {
//             Status = STATUS_INVALID_PARAMETER;
//             goto Failure;
//         }
//
//         Status = ArbpBuildAlternative(Arbiter, Entry->SelectedAlternative, &Alternative);
//         if (!NT_SUCCESS(Status))
//             goto Failure;
//
//         State.Entry = Entry;
//         State.WorkSpace = 0;
//
//         Status = Arbiter->UnpackResource(Entry->Assignment, &State.Start, &Length);
//         if (!NT_SUCCESS(Status))
//             goto Failure;
//         State.End = State.Start + Length - 1;
//
//         Status = Arbiter->PreprocessEntry(Arbiter, &State);
//         if (!NT_SUCCESS(Status))
//             goto Failure;
//
//         if (Length != 0)
//             Arbiter->AddAllocation(Arbiter, &State);
//
//         if (State.Flags & ARBITER_STATE_FLAG_WORKSPACE)
//         {
//             ExFreePoolWithTag((PVOID)State.WorkSpace, TAG_ARBITER);
//             State.WorkSpace = 0;
//             State.Flags &= ~ARBITER_STATE_FLAG_WORKSPACE;
//         }
//     }
//
//     return STATUS_SUCCESS;
//
// Failure:
//     if (State.Flags & ARBITER_STATE_FLAG_WORKSPACE)
//     {
//         ExFreePoolWithTag((PVOID)State.WorkSpace, TAG_ARBITER);
//         State.WorkSpace = 0;
//         State.Flags &= ~ARBITER_STATE_FLAG_WORKSPACE;
//     }
//     RtlFreeRangeList(Arbiter->PossibleAllocation);
//     RtlInitializeRangeList(Arbiter->PossibleAllocation);
//     return Status;
// }
//
// /**
//  * @brief
//  * The TestAllocation action: tentatively places every entry of the
//  * arbitration list, leaving the solution in PossibleAllocation for
//  * a later commit or rollback.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance performing the test.
//  *
//  * @param[in,out] Parameters
//  * The action parameters carrying the arbitration list (pre-Vista
//  * builds receive the list directly).
//  *
//  * @return
//  * Returns the ArbpTestAllocation status.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// #if (NTDDI_VERSION >= NTDDI_VISTA)
// ArbiterLibTestAllocation(
//          PARBITER_INSTANCE Arbiter,
//             PARBITER_TEST_ALLOCATION_PARAMETERS Parameters)
// {
//     PAGED_CODE();
//     return ArbpTestAllocation(Arbiter, Parameters->ArbitrationList);
// }
// #else
// ArbiterLibTestAllocation(
//          PARBITER_INSTANCE Arbiter,
//             PLIST_ENTRY ArbitrationList)
// {
//     PAGED_CODE();
//     return ArbpTestAllocation(Arbiter, ArbitrationList);
// }
// #endif
//
// /**
//  * @brief
//  * The RetestAllocation action: deterministically re-establishes
//  * the placements a previous test chose, without searching.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance performing the retest.
//  *
//  * @param[in,out] Parameters
//  * The action parameters carrying the arbitration list (pre-Vista
//  * builds receive the list directly).
//  *
//  * @return
//  * Returns the ArbpRetestAllocation status.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// #if (NTDDI_VERSION >= NTDDI_VISTA)
// ArbiterLibRetestAllocation(
//          PARBITER_INSTANCE Arbiter,
//             PARBITER_RETEST_ALLOCATION_PARAMETERS Parameters)
// {
//     PAGED_CODE();
//     return ArbpRetestAllocation(Arbiter, Parameters->ArbitrationList);
// }
// #else
// ArbiterLibRetestAllocation(
//          PARBITER_INSTANCE Arbiter,
//             PLIST_ENTRY ArbitrationList)
// {
//     PAGED_CODE();
//     return ArbpRetestAllocation(Arbiter, ArbitrationList);
// }
// #endif
//
// /**
//  * @brief
//  * The CommitAllocation action: the tentative PossibleAllocation
//  * becomes the committed Allocation, and the old committed list is
//  * recycled as the next scratch list.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose transaction is committed.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibCommitAllocation(
//          PARBITER_INSTANCE Arbiter)
// {
//     PRTL_RANGE_LIST Old = Arbiter->Allocation;
//
//     PAGED_CODE();
//
//     RtlFreeRangeList(Old);
//     RtlInitializeRangeList(Old);
//     Arbiter->Allocation = Arbiter->PossibleAllocation;
//     Arbiter->PossibleAllocation = Old;
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * The RollbackAllocation action: discards the tentative
//  * allocation; the committed one is untouched.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose transaction is rolled back.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibRollbackAllocation(
//          PARBITER_INSTANCE Arbiter)
// {
//     PAGED_CODE();
//
//     RtlFreeRangeList(Arbiter->PossibleAllocation);
//     RtlInitializeRangeList(Arbiter->PossibleAllocation);
//     return STATUS_SUCCESS;
// }
//
// /* BOOT ALLOCATION ************************************************************/
//
// /**
//  * @brief
//  * Reserves each entry's firmware boot configuration in the
//  * committed range list, so devices left where the firmware put
//  * them keep their resources. The shared implementation behind
//  * ArbiterLibBootAllocation.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose committed allocation receives the
//  * reservations. The ranges are added through the arbiter's own
//  * AddAllocation callback, owned by the device and tagged
//  * ARBITER_RANGE_BOOT_ALLOCATED so they read back as available to
//  * the owning device's later real allocation.
//  *
//  * @param[in] ArbitrationList
//  * The entries whose boot configurations are to be reserved.
//  * Placement is lenient: an entry whose configuration is not one
//  * sane, fixed, aligned, exact range is silently skipped, because
//  * a malformed firmware config must not block the rest of boot process.
//  *
//  * @return
//  * Returns STATUS_SUCCESS, or the failure status of staging the
//  * additions.
//  */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpBootAllocation(
//          PARBITER_INSTANCE Arbiter,
//          PLIST_ENTRY ArbitrationList)
// {
//     PLIST_ENTRY ListEntry;
//     ARBITER_ALLOCATION_STATE State;
//     ARBITER_ALTERNATIVE Alternative;
//     PRTL_RANGE_LIST Old;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Stage the additions in PossibleAllocation, then swap them in. */
//     RtlFreeRangeList(Arbiter->PossibleAllocation);
//     RtlInitializeRangeList(Arbiter->PossibleAllocation);
//     Status = RtlCopyRangeList(Arbiter->PossibleAllocation, Arbiter->Allocation);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeRangeList(Arbiter->PossibleAllocation);
//         RtlInitializeRangeList(Arbiter->PossibleAllocation);
//         return Status;
//     }
//
//     for (ListEntry = ArbitrationList->Flink;
//          ListEntry != ArbitrationList;
//          ListEntry = ListEntry->Flink)
//     {
//         PARBITER_LIST_ENTRY Entry = CONTAINING_RECORD(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
//
//         if (Entry->AlternativeCount == 0)
//             continue;
//
//         RtlZeroMemory(&State, sizeof(State));
//         RtlZeroMemory(&Alternative, sizeof(Alternative));
//
//         if (!NT_SUCCESS(ArbpBuildAlternative(Arbiter, &Entry->Alternatives[0], &Alternative)))
//             continue;
//
//         /* Only sane, fixed single-placement configurations are reserved. */
//         if (Alternative.Length == 0 ||
//             Alternative.Alignment == 0 ||
//             (Alternative.Flags & ARBITER_ALTERNATIVE_FLAG_BADRANGE) ||
//             !(Alternative.Flags & ARBITER_ALTERNATIVE_FLAG_FIXED) ||
//             (Alternative.Minimum % Alternative.Alignment) != 0)
//         {
//             continue;
//         }
//
//         State.Entry = Entry;
//         State.AlternativeCount = 1;
//         State.Alternatives = &Alternative;
//         State.CurrentAlternative = &Alternative;
//         State.Flags = ARBITER_STATE_FLAG_BOOT;
//         State.RangeAttributes = ARBITER_RANGE_BOOT_ALLOCATED;
//         State.Start = State.CurrentMinimum = Alternative.Minimum;
//         State.End = State.CurrentMaximum = Alternative.Maximum;
//
//         if (!NT_SUCCESS(Arbiter->PreprocessEntry(Arbiter, &State)))
//             continue;
//
//         Arbiter->AddAllocation(Arbiter, &State);
//
//         /*
//          * The reservation is also written back into the device's
//          * Assignment, or it is left holding an unfilled one.
//          */
//         if (Arbiter->PackResource != NULL && Entry->Assignment != NULL)
//         {
//             (VOID)Arbiter->PackResource(Alternative.Descriptor,
//                                         State.Start,
//                                         Entry->Assignment);
//         }
//     }
//
//     Old = Arbiter->Allocation;
//     RtlFreeRangeList(Old);
//     RtlInitializeRangeList(Old);
//     Arbiter->Allocation = Arbiter->PossibleAllocation;
//     Arbiter->PossibleAllocation = Old;
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * The BootAllocation action: records every entry's firmware
//  * boot configuration in the committed allocation.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance performing the reservation.
//  *
//  * @param[in,out] Parameters
//  * The action parameters carrying the arbitration list (pre-Vista
//  * builds receive the list directly).
//  *
//  * @return
//  * Returns the ArbpBootAllocation status.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// #if (NTDDI_VERSION >= NTDDI_VISTA)
// ArbiterLibBootAllocation(
//          PARBITER_INSTANCE Arbiter,
//             PARBITER_BOOT_ALLOCATION_PARAMETERS Parameters)
// {
//     PAGED_CODE();
//     return ArbpBootAllocation(Arbiter, Parameters->ArbitrationList);
// }
// #else
// ArbiterLibBootAllocation(
//          PARBITER_INSTANCE Arbiter,
//             PLIST_ENTRY ArbitrationList)
// {
//     PAGED_CODE();
//     return ArbpBootAllocation(Arbiter, ArbitrationList);
// }
// #endif
//
// /* CONFLICT QUERIES ***********************************************************/
//
// /**
//  * @brief
//  * RtlFindRange conflict callback installed while a conflict query
//  * runs: records the range that blocks placement and reports it as
//  * a real conflict, so FindSuitableRange fails and the caller can
//  * enumerate the owner.
//  *
//  * @param[in] Context
//  * Receives the conflicting range (a PRTL_RANGE pointer slot).
//  *
//  * @param[in] Range
//  * The range RtlFindRange collided with.
//  *
//  * @return
//  * Returns FALSE: the conflict stands.
//  */
// CODE_SEG("PAGE")
// static
// BOOLEAN
// NTAPI
// ArbpQueryConflictCallback(
//          PVOID Context,
//          PRTL_RANGE Range)
// {
//     PAGED_CODE();
//     *(PRTL_RANGE *)Context = Range;
//     return FALSE;
// }
//
// /**
//  * @brief
//  * Enumerates every committed range that conflicts with a candidate
//  * resource for a device. The shared implementation behind
//  * ArbiterLibQueryConflict.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose committed allocation is examined
//  * (through a scratch copy; the committed list is untouched).
//  *
//  * @param[in] PhysicalDeviceObject
//  * The device the resource is being probed for. Its own ranges are
//  * never reported as conflicts.
//  *
//  * @param[in] ConflictingResource
//  * The candidate requirement descriptor to probe.
//  *
//  * @param[out] ConflictCount
//  * Receives the number of conflicts found.
//  *
//  * @param[out] Conflicts
//  * Receives the ARBITER_CONFLICT_INFO array (pool tag TAG_ARBITER);
//  * the caller frees it.
//  *
//  * @return
//  * Returns STATUS_SUCCESS with the conflict array filled in, or a
//  * copy/decode/allocation failure status.
//  *
//  * @remarks
//  * FindSuitableRange is asked to place the resource in the scratch
//  * copy; each failure names one conflicting owner through the
//  * recording callback, which is then removed so the next round
//  * finds the next one. Going through FindSuitableRange keeps the
//  * share and boot semantics identical to assignment time.
//  */
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpQueryConflict(
//          PARBITER_INSTANCE Arbiter,
//          PDEVICE_OBJECT PhysicalDeviceObject,
//          PIO_RESOURCE_DESCRIPTOR ConflictingResource,
//           PULONG ConflictCount,
//           PARBITER_CONFLICT_INFO *Conflicts)
// {
//     PRTL_CONFLICT_RANGE_CALLBACK SavedCallback = Arbiter->ConflictCallback;
//     PVOID SavedContext = Arbiter->ConflictCallbackContext;
//     PRTL_RANGE ConflictingRange = NULL;
//     ARBITER_ALLOCATION_STATE State;
//     ARBITER_LIST_ENTRY Entry;
//     ARBITER_ALTERNATIVE Alternative;
//     PARBITER_CONFLICT_INFO List;
//     ULONG Count = 0;
//     ULONG Capacity = 10;
//     ULONG ResultLength;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     *ConflictCount = 0;
//     *Conflicts = NULL;
//
//     RtlZeroMemory(&State, sizeof(State));
//
//     Arbiter->ConflictCallback = ArbpQueryConflictCallback;
//     Arbiter->ConflictCallbackContext = &ConflictingRange;
//
//     RtlFreeRangeList(Arbiter->PossibleAllocation);
//     RtlInitializeRangeList(Arbiter->PossibleAllocation);
//     Status = RtlCopyRangeList(Arbiter->PossibleAllocation, Arbiter->Allocation);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     Status = ArbpBuildAlternative(Arbiter, ConflictingResource, &Alternative);
//     if (!NT_SUCCESS(Status))
//         goto Cleanup;
//
//     RtlZeroMemory(&Entry, sizeof(Entry));
//     State.Start = State.CurrentMinimum = Alternative.Minimum;
//     State.End = State.CurrentMaximum = Alternative.Maximum;
//     State.CurrentAlternative = State.Alternatives = &Alternative;
//     State.AlternativeCount = 1;
//     State.Entry = &Entry;
//     Entry.RequestSource = ArbiterRequestPnpEnumerated;
//     Entry.PhysicalDeviceObject = PhysicalDeviceObject;
//     if (!NT_SUCCESS(IoGetDeviceProperty(PhysicalDeviceObject,
//                                         DevicePropertyLegacyBusType,
//                                         sizeof(Entry.InterfaceType),
//                                         &Entry.InterfaceType,
//                                         &ResultLength)))
//     {
//         Entry.InterfaceType = Isa;
//     }
//     if (!NT_SUCCESS(IoGetDeviceProperty(PhysicalDeviceObject, DevicePropertyBusNumber,
//                                         sizeof(Entry.BusNumber), &Entry.BusNumber, &ResultLength)))
//     {
//         Entry.BusNumber = 0;
//     }
//
//     List = ExAllocatePoolWithTag(PagedPool, Capacity * sizeof(*List), TAG_ARBITER);
//     if (List == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Cleanup;
//     }
//
//     Status = Arbiter->PreprocessEntry(Arbiter, &State);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(List, TAG_ARBITER);
//         goto Cleanup;
//     }
//
//     /* The requester's own ranges are not conflicts. */
//     RtlDeleteOwnersRanges(Arbiter->PossibleAllocation, PhysicalDeviceObject);
//
//     for (;;)
//     {
//         ConflictingRange = NULL;
//         State.CurrentMinimum = State.Start;
//         State.CurrentMaximum = State.End;
//
//         if (Arbiter->FindSuitableRange(Arbiter, &State))
//             break;      /* placeable now: no more conflicts */
//
//         if (Count == Capacity)
//         {
//             PARBITER_CONFLICT_INFO Grown =
//                 ExAllocatePoolWithTag(PagedPool,
//                                       (Capacity + 5) * sizeof(*Grown),
//                                       TAG_ARBITER);
//             if (Grown == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 ExFreePoolWithTag(List, TAG_ARBITER);
//                 goto Cleanup;
//             }
//             RtlCopyMemory(Grown, List, Count * sizeof(ARBITER_CONFLICT_INFO));
//             ExFreePoolWithTag(List, TAG_ARBITER);
//             List = Grown;
//             Capacity += 5;
//         }
//
//         if (ConflictingRange == NULL)
//         {
//             /* Blocked, but no specific owner: report one whole-range conflict. */
//             List[Count].OwningObject = NULL;
//             List[Count].Start = 0;
//             List[Count].End = ARBITER_MAXIMUM_ADDRESS;
//             ++Count;
//             break;
//         }
//
//         List[Count].OwningObject = (PDEVICE_OBJECT)ConflictingRange->Owner;
//         List[Count].Start = ConflictingRange->Start;
//         List[Count].End = ConflictingRange->End;
//         ++Count;
//
//         /* Remove that owner and look for the next conflict. */
//         Status = RtlDeleteOwnersRanges(Arbiter->PossibleAllocation, ConflictingRange->Owner);
//         if (!NT_SUCCESS(Status))
//         {
//             ExFreePoolWithTag(List, TAG_ARBITER);
//             goto Cleanup;
//         }
//     }
//
//     *Conflicts = List;
//     *ConflictCount = Count;
//     Status = STATUS_SUCCESS;
//
// Cleanup:
//     if (State.Flags & ARBITER_STATE_FLAG_WORKSPACE)
//     {
//         ExFreePoolWithTag((PVOID)State.WorkSpace, TAG_ARBITER);
//         State.WorkSpace = 0;
//         State.Flags &= ~ARBITER_STATE_FLAG_WORKSPACE;
//     }
//     RtlFreeRangeList(Arbiter->PossibleAllocation);
//     RtlInitializeRangeList(Arbiter->PossibleAllocation);
//     Arbiter->ConflictCallback = SavedCallback;
//     Arbiter->ConflictCallbackContext = SavedContext;
//     return Status;
// }
//
// /**
//  * @brief
//  * The QueryConflict action: reports which devices' committed
//  * ranges collide with a candidate resource.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance performing the query.
//  *
//  * @param[in,out] Parameters
//  * The action parameters carrying the device, the candidate
//  * resource and the output slots (pre-Vista builds receive them
//  * directly).
//  *
//  * @return
//  * Returns the ArbpQueryConflict status.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// #if (NTDDI_VERSION >= NTDDI_VISTA)
// ArbiterLibQueryConflict(
//          PARBITER_INSTANCE Arbiter,
//             PARBITER_QUERY_CONFLICT_PARAMETERS Parameters)
// {
//     PAGED_CODE();
//     return ArbpQueryConflict(Arbiter,
//                              Parameters->PhysicalDeviceObject,
//                              Parameters->ConflictingResource,
//                              Parameters->ConflictCount,
//                              Parameters->Conflicts);
// }
// #else
// ArbiterLibQueryConflict(
//          PARBITER_INSTANCE Arbiter,
//          PDEVICE_OBJECT PhysicalDeviceObject,
//          PIO_RESOURCE_DESCRIPTOR ConflictingResource,
//           PULONG ConflictCount,
//           PARBITER_CONFLICT_INFO *Conflicts)
// {
//     PAGED_CODE();
//     return ArbpQueryConflict(Arbiter, PhysicalDeviceObject, ConflictingResource,
//                              ConflictCount, Conflicts);
// }
// #endif
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// The four action entry points below are written by hand (rustconv --skip-fn): transaction.c
// chooses between their signatures with `#if NTDDI_VERSION` placed between NTAPI and the name,
// which the scanner cannot parse. This build is NTDDI 0x502, so these are the pre-Vista arms.
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibTestAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbitrationList: PLIST_ENTRY,
) -> NTSTATUS {
    PAGED_CODE!();
    ArbpTestAllocation(Arbiter, ArbitrationList)
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibRetestAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbitrationList: PLIST_ENTRY,
) -> NTSTATUS {
    PAGED_CODE!();
    ArbpRetestAllocation(Arbiter, ArbitrationList)
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibBootAllocation(
    Arbiter: PARBITER_INSTANCE,
    ArbitrationList: PLIST_ENTRY,
) -> NTSTATUS {
    PAGED_CODE!();
    ArbpBootAllocation(Arbiter, ArbitrationList)
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibQueryConflict(
    Arbiter: PARBITER_INSTANCE,
    PhysicalDeviceObject: PDEVICE_OBJECT,
    ConflictingResource: PIO_RESOURCE_DESCRIPTOR,
    ConflictCount: PULONG,
    Conflicts: *mut PARBITER_CONFLICT_INFO,
) -> NTSTATUS {
    PAGED_CODE!();
    ArbpQueryConflict(Arbiter, PhysicalDeviceObject, ConflictingResource, ConflictCount, Conflicts)
}
