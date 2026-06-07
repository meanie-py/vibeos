use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{Available, Next, PRTL_CONFLICT_RANGE_CALLBACK, PRTL_RANGE, PRTL_RANGE_LIST, PRTL_RANGE_LIST_ITERATOR, PULONGLONG, RTL_RANGE, RTL_RANGE_LIST, RTL_RANGE_LIST_ITERATOR};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _RTL_RANGE_ENTRY {
    pub Entry: _LIST_ENTRY,
    pub Range: RTL_RANGE,
}
pub type PRTL_RANGE_ENTRY = *mut _RTL_RANGE_ENTRY;

#[no_mangle]
pub unsafe extern "system" fn RtlAddRange(
    RangeList: PRTL_RANGE_LIST,
    Start: ULONGLONG,
    End: ULONGLONG,
    Attributes: UCHAR,
    Flags: ULONG,
    UserData: PVOID,
    Owner: PVOID,
) -> NTSTATUS {
    let RangeEntry: PRTL_RANGE_ENTRY;
    let mut Current: PRTL_RANGE_ENTRY;
    let mut Entry: PLIST_ENTRY;

    if Start > End {
        return STATUS_INVALID_PARAMETER;
    }

    RangeEntry = RtlpAllocateMemory(
        core::mem::size_of::<RTL_RANGE_ENTRY>() as ULONG,
        u32::from_le_bytes(*b"RRle"),
    ) as PRTL_RANGE_ENTRY;
    if RangeEntry.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*RangeEntry).Range.Start = Start;
    (*RangeEntry).Range.End = End;
    (*RangeEntry).Range.Attributes = Attributes;
    (*RangeEntry).Range.UserData = UserData;
    (*RangeEntry).Range.Owner = Owner;

    (*RangeEntry).Range.Flags = 0;
    if (Flags & RTL_RANGE_LIST_ADD_SHARED) != 0 {
        (*RangeEntry).Range.Flags |= RTL_RANGE_SHARED as UCHAR;
    }

    if (*RangeList).Count == 0 {
        InsertTailList(
            core::ptr::addr_of_mut!((*RangeList).ListHead),
            core::ptr::addr_of_mut!((*RangeEntry).Entry),
        );
        (*RangeList).Count += 1;
        (*RangeList).Stamp += 1;
        return STATUS_SUCCESS;
    } else {
        Entry = (*RangeList).ListHead.Flink;
        while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
            Current = CONTAINING_RECORD!(Entry, RTL_RANGE_ENTRY, Entry);
            if (*Current).Range.Start > (*RangeEntry).Range.End {
                InsertTailList(
                    core::ptr::addr_of_mut!((*Current).Entry),
                    core::ptr::addr_of_mut!((*RangeEntry).Entry),
                );

                (*RangeList).Count += 1;
                (*RangeList).Stamp += 1;
                return STATUS_SUCCESS;
            }

            Entry = (*Entry).Flink;
        }

        InsertTailList(
            core::ptr::addr_of_mut!((*RangeList).ListHead),
            core::ptr::addr_of_mut!((*RangeEntry).Entry),
        );
        (*RangeList).Count += 1;
        (*RangeList).Stamp += 1;
        return STATUS_SUCCESS;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlCopyRangeList(
    CopyRangeList: PRTL_RANGE_LIST,
    RangeList: PRTL_RANGE_LIST,
) -> NTSTATUS {
    let mut Current: *mut _RTL_RANGE_ENTRY;
    let NewEntry: *mut _RTL_RANGE_ENTRY;
    let mut Entry: PLIST_ENTRY;

    (*CopyRangeList).Flags = (*RangeList).Flags;

    Entry = (*RangeList).ListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
        Current = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
            as *mut _RTL_RANGE_ENTRY;

        let NewEntry = RtlpAllocateMemory(
            core::mem::size_of::<_RTL_RANGE_ENTRY>() as ULONG,
            u32::from_le_bytes(*b"RRle"),
        ) as *mut _RTL_RANGE_ENTRY;
        if NewEntry.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlCopyMemory(
            core::ptr::addr_of_mut!((*NewEntry).Range) as PVOID,
            core::ptr::addr_of_mut!((*Current).Range) as PVOID,
            core::mem::size_of::<RTL_RANGE>() as SIZE_T,
        );

        InsertTailList(
            core::ptr::addr_of_mut!((*CopyRangeList).ListHead),
            core::ptr::addr_of_mut!((*NewEntry).Entry),
        );

        (*CopyRangeList).Count += 1;

        Entry = (*Entry).Flink;
    }

    (*CopyRangeList).Stamp += 1;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteOwnersRanges(
    RangeList: PRTL_RANGE_LIST,
    Owner: PVOID,
) -> NTSTATUS {
    let mut Current: PRTL_RANGE_ENTRY;
    let mut Entry: PLIST_ENTRY;

    Entry = (*RangeList).ListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
        Current = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
            as PRTL_RANGE_ENTRY;
        if (*Current).Range.Owner == Owner {
            RemoveEntryList(Entry);
            RtlpFreeMemory(Current as PVOID, 0);

            (*RangeList).Count -= 1;
            (*RangeList).Stamp += 1;
        }

        Entry = (*Entry).Flink;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlDeleteRange(
    RangeList: PRTL_RANGE_LIST,
    Start: ULONGLONG,
    End: ULONGLONG,
    Owner: PVOID,
) -> NTSTATUS {
    let mut Entry: PLIST_ENTRY = (*RangeList).ListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
        let Current: PRTL_RANGE_ENTRY = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
            as PRTL_RANGE_ENTRY;
        if (*Current).Range.Start == Start
            && (*Current).Range.End == End
            && (*Current).Range.Owner == Owner
        {
            RemoveEntryList(Entry);

            RtlpFreeMemory(Current as PVOID, 0);

            (*RangeList).Count -= 1;
            (*RangeList).Stamp += 1;
            return STATUS_SUCCESS;
        }

        Entry = (*Entry).Flink;
    }

    STATUS_RANGE_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn RtlFindRange(
    RangeList: PRTL_RANGE_LIST,
    Minimum: ULONGLONG,
    Maximum: ULONGLONG,
    Length: ULONG,
    Alignment: ULONG,
    _Flags: ULONG,
    _AttributeAvailableMask: UCHAR,
    _Context: PVOID,
    _Callback: PRTL_CONFLICT_RANGE_CALLBACK,
    Start: PULONGLONG,
) -> NTSTATUS {
    let mut CurrentEntry: PRTL_RANGE_ENTRY;
    let mut NextEntry: PRTL_RANGE_ENTRY;
    let mut Entry: PLIST_ENTRY;
    let mut RangeMin: ULONGLONG;
    let mut RangeMax: ULONGLONG;

    if Alignment == 0 || Length == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if IsListEmpty(core::ptr::addr_of_mut!((*RangeList).ListHead)) != 0 {
        *Start = ROUND_DOWN(Maximum - (Length as ULONGLONG - 1), Alignment as ULONGLONG);
        return STATUS_SUCCESS;
    }

    NextEntry = core::ptr::null_mut();
    Entry = (*RangeList).ListHead.Blink;
    while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
        CurrentEntry = (Entry as *mut u8)
            .sub(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry)) as PRTL_RANGE_ENTRY;

        RangeMax = if !NextEntry.is_null() {
            (*NextEntry).Range.Start - 1
        } else {
            Maximum
        };
        if RangeMax + (Length as ULONGLONG - 1) < Minimum {
            return STATUS_RANGE_NOT_FOUND;
        }

        RangeMin = ROUND_DOWN(RangeMax - (Length as ULONGLONG - 1), Alignment as ULONGLONG);
        if RangeMin < Minimum || (RangeMax - RangeMin) < (Length as ULONGLONG - 1) {
            return STATUS_RANGE_NOT_FOUND;
        }

        if RangeMin > (*CurrentEntry).Range.End {
            *Start = RangeMin;
            return STATUS_SUCCESS;
        }

        NextEntry = CurrentEntry;
        Entry = (*Entry).Blink;
    }

    RangeMax = if !NextEntry.is_null() {
        (*NextEntry).Range.Start - 1
    } else {
        Maximum
    };
    if RangeMax + (Length as ULONGLONG - 1) < Minimum {
        return STATUS_RANGE_NOT_FOUND;
    }

    RangeMin = ROUND_DOWN(RangeMax - (Length as ULONGLONG - 1), Alignment as ULONGLONG);
    if RangeMin < Minimum || (RangeMax - RangeMin) < (Length as ULONGLONG - 1) {
        return STATUS_RANGE_NOT_FOUND;
    }

    *Start = RangeMin;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlFreeRangeList(RangeList: PRTL_RANGE_LIST) {
    while IsListEmpty(core::ptr::addr_of_mut!((*RangeList).ListHead)) == 0 {
        let Entry = RemoveHeadList(core::ptr::addr_of_mut!((*RangeList).ListHead));
        let Current = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
            as PRTL_RANGE_ENTRY;

        RtlpFreeMemory(Current as PVOID, 0);
    }

    (*RangeList).Flags = 0;
    (*RangeList).Count = 0;
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetFirstRange(
    RangeList: PRTL_RANGE_LIST,
    Iterator: PRTL_RANGE_LIST_ITERATOR,
    Range: *mut PRTL_RANGE,
) -> NTSTATUS {
    (*Iterator).RangeListHead = core::ptr::addr_of_mut!((*RangeList).ListHead);
    (*Iterator).MergedHead = core::ptr::null_mut();
    (*Iterator).Stamp = (*RangeList).Stamp;

    if IsListEmpty(core::ptr::addr_of_mut!((*RangeList).ListHead)) != 0 {
        (*Iterator).Current = core::ptr::null_mut();
        *Range = core::ptr::null_mut();
        return STATUS_NO_MORE_ENTRIES;
    }

    {
        let RangeEntry = ((*RangeList).ListHead.Flink as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
            as *mut _RTL_RANGE_ENTRY;
        (*Iterator).Current = core::ptr::addr_of_mut!((*RangeEntry).Range) as *mut core::ffi::c_void;
        *Range = core::ptr::addr_of_mut!((*RangeEntry).Range);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlGetNextRange(
    Iterator: PRTL_RANGE_LIST_ITERATOR,
    Range: *mut PRTL_RANGE,
    MoveForwards: u8,
) -> NTSTATUS {
    let RangeList: PRTL_RANGE_LIST = ((*Iterator).RangeListHead as *mut u8)
        .sub(core::mem::offset_of!(_RTL_RANGE_LIST, ListHead)) as PRTL_RANGE_LIST;

    if (*Iterator).Stamp != (*RangeList).Stamp {
        return STATUS_INVALID_PARAMETER;
    }

    if (*Iterator).Current.is_null() {
        *Range = core::ptr::null_mut();
        return STATUS_NO_MORE_ENTRIES;
    }

    let CurEntry: PRTL_RANGE_ENTRY = ((*Iterator).Current as *mut u8)
        .sub(core::mem::offset_of!(_RTL_RANGE_ENTRY, Range)) as PRTL_RANGE_ENTRY;

    let Next: PLIST_ENTRY = if MoveForwards != 0 {
        (*CurEntry).Entry.Flink
    } else {
        (*CurEntry).Entry.Blink
    };

    if Next == (*Iterator).RangeListHead {
        (*Iterator).Current = core::ptr::null_mut();
        *Range = core::ptr::null_mut();
        return STATUS_NO_MORE_ENTRIES;
    }

    let NextEntry: PRTL_RANGE_ENTRY = (Next as *mut u8)
        .sub(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry)) as PRTL_RANGE_ENTRY;
    (*Iterator).Current = core::ptr::addr_of_mut!((*NextEntry).Range);
    *Range = core::ptr::addr_of_mut!((*NextEntry).Range);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlInitializeRangeList(RangeList: PRTL_RANGE_LIST) {
    InitializeListHead(core::ptr::addr_of_mut!((*RangeList).ListHead));
    (*RangeList).Flags = 0;
    (*RangeList).Count = 0;
    (*RangeList).Stamp = 0;
}

#[no_mangle]
pub unsafe extern "system" fn RtlInvertRangeList(
    InvertedRangeList: PRTL_RANGE_LIST,
    RangeList: PRTL_RANGE_LIST,
) -> NTSTATUS {
    let mut Previous: PRTL_RANGE_ENTRY = core::ptr::null_mut();
    let mut Current: PRTL_RANGE_ENTRY;
    let mut Entry: PLIST_ENTRY;
    let mut Status: NTSTATUS;

    Entry = (*RangeList).ListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
        Current = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(RTL_RANGE_ENTRY, Entry) as isize))
            as PRTL_RANGE_ENTRY;

        if Previous.is_null() {
            if (*Current).Range.Start != 0u64 {
                Status = RtlAddRange(
                    InvertedRangeList,
                    0u64,
                    (*Current).Range.Start - 1,
                    0,
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if !NT_SUCCESS(Status) {
                    return Status;
                }
            }
        } else {
            if (*Previous).Range.End + 1 != (*Current).Range.Start {
                Status = RtlAddRange(
                    InvertedRangeList,
                    (*Previous).Range.End + 1,
                    (*Current).Range.Start - 1,
                    0,
                    0,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if !NT_SUCCESS(Status) {
                    return Status;
                }
            }
        }

        Previous = Current;
        Entry = (*Entry).Flink;
    }

    if Previous.is_null() {
        return STATUS_SUCCESS;
    }

    if (*Previous).Range.End + 1 != (-1i64) as u64 {
        Status = RtlAddRange(
            InvertedRangeList,
            (*Previous).Range.End + 1,
            (-1i64) as u64,
            0,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsRangeAvailable(
    RangeList: PRTL_RANGE_LIST,
    Start: ULONGLONG,
    End: ULONGLONG,
    Flags: ULONG,
    AttributeAvailableMask: UCHAR,
    Context: PVOID,
    Callback: PRTL_CONFLICT_RANGE_CALLBACK,
    Available: PBOOLEAN,
) -> NTSTATUS {
    let mut Current: PRTL_RANGE_ENTRY;
    let mut Entry: PLIST_ENTRY;

    *Available = 1u8;

    Entry = (*RangeList).ListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
        Current = (Entry as *mut u8).offset(
            -(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize),
        ) as PRTL_RANGE_ENTRY;

        if (*Current).Range.Start > End || (*Current).Range.End < Start {
            /* No overlap */
        } else if AttributeAvailableMask != 0
            && ((*Current).Range.Attributes & AttributeAvailableMask) != 0
        {
            /* Attributes match */
        } else if (Flags & RTL_RANGE_SHARED) != 0
            && ((*Current).Range.Flags & RTL_RANGE_SHARED as UCHAR) != 0
        {
            /* Both shared */
        } else {
            if let Some(cb) = Callback {
                *Available = cb(Context, core::ptr::addr_of_mut!((*Current).Range));
            } else {
                *Available = 0u8;
            }
        }

        Entry = (*Entry).Flink;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlMergeRangeLists(
    MergedRangeList: PRTL_RANGE_LIST,
    RangeList1: PRTL_RANGE_LIST,
    RangeList2: PRTL_RANGE_LIST,
    Flags: ULONG,
) -> NTSTATUS {
    let mut Iterator: RTL_RANGE_LIST_ITERATOR = core::mem::zeroed();
    let mut Range: PRTL_RANGE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = RtlCopyRangeList(MergedRangeList, RangeList1);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = RtlGetFirstRange(RangeList2, &mut Iterator, &mut Range);
    if !NT_SUCCESS(Status) {
        return if Status == STATUS_NO_MORE_ENTRIES {
            STATUS_SUCCESS
        } else {
            Status
        };
    }

    loop {
        Status = RtlAddRange(
            MergedRangeList,
            (*Range).Start,
            (*Range).End,
            (*Range).Attributes,
            (*Range).Flags | Flags,
            (*Range).UserData,
            (*Range).Owner,
        );
        if !NT_SUCCESS(Status) {
            break;
        }

        Status = RtlGetNextRange(&mut Iterator, &mut Range, 1u8);
        if !NT_SUCCESS(Status) {
            break;
        }
    }

    if Status == STATUS_NO_MORE_ENTRIES {
        STATUS_SUCCESS
    } else {
        Status
    }
}
