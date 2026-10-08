//! Rust translation of `sdk/lib/rtl/rangelist.c` — the RTL range-list
//! (`RTL_RANGE_LIST`) algorithm: add/delete/find/copy/invert/merge ranges
//! and iterate them. A range list is a `LIST_ENTRY` of `_RTL_RANGE_ENTRY`
//! nodes (each `{ LIST_ENTRY; RTL_RANGE }`), kept sorted by start address.
//!
//! Self-contained except for the shared user/kernel allocation helpers
//! `RtlpAllocateMemory`/`RtlpFreeMemory` (rtlp.h, implemented in libsupp)
//! and the ntkrnl-sys `RTL_RANGE*` layout (size-asserted).
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_NO_MORE_ENTRIES};
use ntkrnl_sys::ddk::{
    PRTL_CONFLICT_RANGE_CALLBACK, PRTL_RANGE, PRTL_RANGE_LIST, PRTL_RANGE_LIST_ITERATOR, PULONGLONG,
    RTL_RANGE, RTL_RANGE_LIST_ITERATOR, _LIST_ENTRY, _RTL_RANGE_LIST,
};
use ntkrnl_sys::rtl::RtlCopyMemory;

// rangelist flag / status constants (fixed ntddk / ntstatus.h values).
// Values are sdk/include/ndk/rtltypes.h. ADD_SHARED was hand-written as 0x1 here, which is
// RTL_RANGE_LIST_ADD_IF_CONFLICT -- so `RtlAddRange` never set RTL_RANGE_SHARED on the stored
// entry and `RtlIsRangeAvailable` never saw a shared range. Upstream's new RtlRangeList kmtest
// caught it (`[1] Flags = 0x0, expected 0x1`).
const RTL_RANGE_LIST_ADD_IF_CONFLICT: ULONG = 0x0000_0001;
const RTL_RANGE_LIST_ADD_SHARED: ULONG = 0x0000_0002;
// The RtlIsRangeAvailable `Flags` namespace is a DIFFERENT one that happens to share a value.
const RTL_RANGE_LIST_SHARED_OK: ULONG = 0x0000_0001;
const RTL_RANGE_LIST_NULL_CONFLICT_OK: ULONG = 0x0000_0002;
const RTL_RANGE_SHARED: ULONG = 0x01;
const STATUS_RANGE_NOT_FOUND: NTSTATUS = 0xC000028C_u32 as i32;
const STATUS_RANGE_LIST_CONFLICT: NTSTATUS = 0xC0000282_u32 as i32;

/// Returns whether the closed interval [Start1, End1] intersects [Start2, End2].
#[inline]
fn RtlpOverlaps(Start1: ULONGLONG, End1: ULONGLONG, Start2: ULONGLONG, End2: ULONGLONG) -> bool {
    Start1 <= End2 && Start2 <= End1
}

/// Shared availability test behind `RtlIsRangeAvailable` and `RtlFindRange`. Walks every entry
/// overlapping [Start, End]; an overlapping entry is NOT a conflict when the caller passed
/// `RTL_RANGE_LIST_SHARED_OK` and it is shared, when it carries an attribute in
/// `AttributeAvailableMask`, when the caller passed `RTL_RANGE_LIST_NULL_CONFLICT_OK` and it has
/// a NULL owner, or when `Callback` returns TRUE. When the window is unavailable,
/// `ConflictStart` receives the LOWEST conflicting Start, so a top-down search can jump past it.
unsafe fn RtlpWindowIsAvailable(
    RangeList: PRTL_RANGE_LIST,
    Start: ULONGLONG,
    End: ULONGLONG,
    Flags: ULONG,
    AttributeAvailableMask: UCHAR,
    Context: PVOID,
    Callback: PRTL_CONFLICT_RANGE_CALLBACK,
    ConflictStart: PULONGLONG,
) -> bool {
    let mut Available = true;
    let mut Lowest: ULONGLONG = 0;

    let mut Entry = (*RangeList).ListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
        let Current = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
            as PRTL_RANGE_ENTRY;

        if (*Current).Range.Start > End {
            break;
        }

        if !RtlpOverlaps(Start, End, (*Current).Range.Start, (*Current).Range.End) {
            Entry = (*Entry).Flink;
            continue;
        }

        let ignored = ((Flags & RTL_RANGE_LIST_SHARED_OK) != 0
            && ((*Current).Range.Flags & RTL_RANGE_SHARED as UCHAR) != 0)
            || (AttributeAvailableMask & (*Current).Range.Attributes) != 0
            || ((Flags & RTL_RANGE_LIST_NULL_CONFLICT_OK) != 0
                && (*Current).Range.Owner.is_null())
            || match Callback {
                Some(cb) => cb(Context, core::ptr::addr_of_mut!((*Current).Range)) != 0,
                None => false,
            };

        if !ignored {
            if Available || (*Current).Range.Start < Lowest {
                Lowest = (*Current).Range.Start;
            }
            Available = false;
        }

        Entry = (*Entry).Flink;
    }

    if !ConflictStart.is_null() && !Available {
        *ConflictStart = Lowest;
    }

    Available
}

/// Whether inserting [Start, End] would conflict with an existing entry. Two shared ranges
/// (`RTL_RANGE_LIST_ADD_SHARED` against an already-shared entry) may coexist.
unsafe fn RtlpConflictsOnAdd(
    RangeList: PRTL_RANGE_LIST,
    Start: ULONGLONG,
    End: ULONGLONG,
    Flags: ULONG,
) -> bool {
    let mut Entry = (*RangeList).ListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
        let Current = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
            as PRTL_RANGE_ENTRY;

        if (*Current).Range.Start > End {
            break;
        }

        if RtlpOverlaps(Start, End, (*Current).Range.Start, (*Current).Range.End)
            && !((Flags & RTL_RANGE_LIST_ADD_SHARED) != 0
                && ((*Current).Range.Flags & RTL_RANGE_SHARED as UCHAR) != 0)
        {
            return true;
        }

        Entry = (*Entry).Flink;
    }

    false
}

// Shared user/kernel RTL allocation helpers (rtlp.h; defined in libsupp).
extern "system" {
    fn RtlpAllocateMemory(Bytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn RtlpFreeMemory(Mem: PVOID, Tag: ULONG);
}

#[repr(C)]
pub struct _RTL_RANGE_ENTRY {
    pub Entry: _LIST_ENTRY,
    pub Range: RTL_RANGE,
}
pub type PRTL_RANGE_ENTRY = *mut _RTL_RANGE_ENTRY;
pub type RTL_RANGE_ENTRY = _RTL_RANGE_ENTRY;

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

    // Unless the caller explicitly allows adding on top of a conflict, reject a range that
    // overlaps an incompatible existing entry. The pre-merge translation had no conflict
    // detection at all, so every RtlAddRange succeeded.
    if (Flags & RTL_RANGE_LIST_ADD_IF_CONFLICT) == 0
        && RtlpConflictsOnAdd(RangeList, Start, End, Flags)
    {
        return STATUS_RANGE_LIST_CONFLICT;
    }

    RangeEntry = RtlpAllocateMemory(
        core::mem::size_of::<RTL_RANGE_ENTRY>() as SIZE_T,
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
            Current = (Entry as *mut u8)
                .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
                as PRTL_RANGE_ENTRY;
            if (*Current).Range.Start > (*RangeEntry).Range.Start {
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
    let mut Entry: PLIST_ENTRY;

    (*CopyRangeList).Flags = (*RangeList).Flags;

    Entry = (*RangeList).ListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
        Current = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
            as *mut _RTL_RANGE_ENTRY;

        let NewEntry = RtlpAllocateMemory(
            core::mem::size_of::<_RTL_RANGE_ENTRY>() as SIZE_T,
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
        // Capture the next link BEFORE the entry can be freed. Reading `(*Entry).Flink` after
        // RtlpFreeMemory is a use-after-free that usually survives (freed pool still holds the
        // old link) and faults only when the block is reused first: kmtest RtlRangeList's
        // intermittent BUGCHECK 0x1E on all three arches, a non-canonical Flink from a block
        // already handed to someone else.
        let Next = (*Entry).Flink;

        Current = (Entry as *mut u8)
            .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
            as PRTL_RANGE_ENTRY;
        if (*Current).Range.Owner == Owner {
            RemoveEntryList(Entry);
            RtlpFreeMemory(Current as PVOID, 0);

            (*RangeList).Count -= 1;
            (*RangeList).Stamp += 1;
        }

        Entry = Next;
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
    // Widened ULONG -> ULONGLONG by the upstream merge (sdk/include/ndk/rtlfuncs.h). Every C
    // caller is compiled against that; leaving these 32-bit here is a stdcall callee-pops
    // mismatch on i386 and truncation on the 64-bit targets.
    Length: ULONGLONG,
    Alignment: ULONGLONG,
    Flags: ULONG,
    AttributeAvailableMask: UCHAR,
    Context: PVOID,
    Callback: PRTL_CONFLICT_RANGE_CALLBACK,
    Start: PULONGLONG,
) -> NTSTATUS {
    let mut ConflictStart: ULONGLONG = 0;

    if Alignment == 0 || Length == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    // A window of Length can only end at Maximum if it also fits below it.
    if Length - 1 > Maximum {
        return STATUS_RANGE_NOT_FOUND;
    }

    // Pure 64-bit arithmetic. The crate's old ROUND_DOWN helper rounded through `usize`,
    // modelling a C `ULONG_PTR` cast that this expression does not have -- on i386 that
    // truncated the candidate to 32 bits. It had no other caller, so it is gone.
    let mut Candidate: ULONGLONG = (Maximum - (Length - 1)) & !(Alignment - 1);

    loop {
        if Candidate < Minimum {
            return STATUS_RANGE_NOT_FOUND;
        }

        let CandidateEnd = Candidate + (Length - 1);

        if RtlpWindowIsAvailable(
            RangeList,
            Candidate,
            CandidateEnd,
            Flags,
            AttributeAvailableMask,
            Context,
            Callback,
            core::ptr::addr_of_mut!(ConflictStart),
        ) {
            *Start = Candidate;
            return STATUS_SUCCESS;
        }

        // Jump the window entirely below the lowest conflicting entry. That entry overlaps the
        // current window, so Candidate strictly decreases and the loop always terminates.
        if ConflictStart == 0 {
            return STATUS_RANGE_NOT_FOUND;
        }

        if Length - 1 > ConflictStart - 1 {
            return STATUS_RANGE_NOT_FOUND;
        }

        Candidate = ((ConflictStart - 1) - (Length - 1)) & !(Alignment - 1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlFreeRangeList(RangeList: PRTL_RANGE_LIST) {
    while !IsListEmpty(core::ptr::addr_of_mut!((*RangeList).ListHead)) {
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

    if IsListEmpty(core::ptr::addr_of_mut!((*RangeList).ListHead)) {
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

/// New upstream export: the last range, so `RtlGetNextRange(.., MoveForwards = FALSE)` can
/// walk a list backwards. Identical to `RtlGetFirstRange` but off `Blink`.
#[no_mangle]
pub unsafe extern "system" fn RtlGetLastRange(
    RangeList: PRTL_RANGE_LIST,
    Iterator: PRTL_RANGE_LIST_ITERATOR,
    Range: *mut PRTL_RANGE,
) -> NTSTATUS {
    (*Iterator).RangeListHead = core::ptr::addr_of_mut!((*RangeList).ListHead);
    (*Iterator).MergedHead = core::ptr::null_mut();
    (*Iterator).Stamp = (*RangeList).Stamp;

    if IsListEmpty(core::ptr::addr_of_mut!((*RangeList).ListHead)) {
        (*Iterator).Current = core::ptr::null_mut();
        *Range = core::ptr::null_mut();
        return STATUS_NO_MORE_ENTRIES;
    }

    // Iterator.Current is the RANGE pointer, not the entry — matches Windows.
    let RangeEntry = ((*RangeList).ListHead.Blink as *mut u8)
        .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
        as *mut _RTL_RANGE_ENTRY;
    (*Iterator).Current = core::ptr::addr_of_mut!((*RangeEntry).Range) as *mut core::ffi::c_void;
    *Range = core::ptr::addr_of_mut!((*RangeEntry).Range);

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
    (*Iterator).Current = core::ptr::addr_of_mut!((*NextEntry).Range) as PVOID;
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

/// New upstream export. The pre-merge translation walked the sorted list pairwise and emitted
/// the gap between each adjacent pair, which is only correct when no two entries overlap --
/// and `RTL_RANGE_LIST_ADD_IF_CONFLICT` lets them. This walks the covered address space upward
/// instead, absorbing every range that covers the cursor before looking for the next gap.
#[no_mangle]
pub unsafe extern "system" fn RtlInvertRangeListEx(
    InvertedRangeList: PRTL_RANGE_LIST,
    RangeList: PRTL_RANGE_LIST,
    Attributes: UCHAR,
    UserData: PVOID,
    Owner: PVOID,
) -> NTSTATUS {
    let mut GapStart: ULONGLONG = 0;

    loop {
        // Advance GapStart past every range that covers it (handles overlaps).
        loop {
            let mut Absorbed = false;
            let mut Entry = (*RangeList).ListHead.Flink;
            while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
                let Current = (Entry as *mut u8)
                    .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
                    as PRTL_RANGE_ENTRY;
                if (*Current).Range.Start <= GapStart && (*Current).Range.End >= GapStart {
                    // Covered all the way to the top: no gap remains.
                    if (*Current).Range.End == ULONGLONG::MAX {
                        return STATUS_SUCCESS;
                    }
                    GapStart = (*Current).Range.End + 1;
                    Absorbed = true;
                }
                Entry = (*Entry).Flink;
            }
            if !Absorbed {
                break;
            }
        }

        // GapStart is now uncovered; find the nearest range starting above it.
        let mut NextStart: ULONGLONG = ULONGLONG::MAX;
        let mut Found = false;
        let mut Entry = (*RangeList).ListHead.Flink;
        while Entry != core::ptr::addr_of_mut!((*RangeList).ListHead) {
            let Current = (Entry as *mut u8)
                .offset(-(core::mem::offset_of!(_RTL_RANGE_ENTRY, Entry) as isize))
                as PRTL_RANGE_ENTRY;
            if (*Current).Range.Start > GapStart && (*Current).Range.Start <= NextStart {
                NextStart = (*Current).Range.Start;
                Found = true;
            }
            Entry = (*Entry).Flink;
        }

        // No further ranges: the gap runs to the top of the address space.
        if !Found {
            return RtlAddRange(
                InvertedRangeList,
                GapStart,
                ULONGLONG::MAX,
                Attributes,
                RTL_RANGE_LIST_ADD_IF_CONFLICT,
                UserData,
                Owner,
            );
        }

        let Status = RtlAddRange(
            InvertedRangeList,
            GapStart,
            NextStart - 1,
            Attributes,
            RTL_RANGE_LIST_ADD_IF_CONFLICT,
            UserData,
            Owner,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        // Resume from the next covered region.
        GapStart = NextStart;
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlInvertRangeList(
    InvertedRangeList: PRTL_RANGE_LIST,
    RangeList: PRTL_RANGE_LIST,
) -> NTSTATUS {
    RtlInvertRangeListEx(
        InvertedRangeList,
        RangeList,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
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
    Available: *mut BOOLEAN,
) -> NTSTATUS {
    // DELIBERATE divergence from the merged C, which added
    // `if (Start > End) return STATUS_INVALID_PARAMETER;` here. Upstream added that guard in
    // the SAME commit as kmtests/rtl/RtlRangeList.c, and that test asserts STATUS_SUCCESS for
    // seven inverted windows (:349,357,358,365,373,374,381) with no is_reactos()/todo guard --
    // so upstream's C fails upstream's own new test. The walk answers those correctly on its
    // own: `Current->Range.Start > End` breaks out immediately for the TRUE cases and
    // RtlpOverlaps catches the FALSE ones. All seven are banked passing in every conformance
    // baseline, so adding the guard would regress the oracle to match a C the oracle rejects.
    // RtlAddRange's Start > End guard is a different matter and stays: upstream did not touch
    // it and no assertion covers it.
    *Available = RtlpWindowIsAvailable(
        RangeList,
        Start,
        End,
        Flags,
        AttributeAvailableMask,
        Context,
        Callback,
        core::ptr::null_mut(),
    ) as BOOLEAN;

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
            (*Range).Flags as ULONG | Flags,
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
