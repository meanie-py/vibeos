#[allow(unused_imports)]
use crate::prelude::*;
// rustconv-assembly: whole-tu
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_AEEE8D02E860: [u16; 10] = [0x0028u16, 0x0075, 0x006E, 0x006E, 0x0061, 0x006D, 0x0065, 0x0064, 0x0029, 0x0000];

use ntkrnl_sys::ddk::{ARBITER_LIST_ENTRY, LIST_ENTRY, PARBITER_ALLOCATION_STATE, PARBITER_ALTERNATIVE, PARBITER_INSTANCE, PARBITER_LIST_ENTRY, PRTL_RANGE, RTL_RANGE_LIST_ITERATOR, RtlGetFirstRange, RtlGetNextRange};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const ARBITER_ALLOCATE_DEADLINE: u64 = 10_000_000;

pub const ARBITER_ALLOCATE_HARD_DEADLINE: u64 = 100_000_000;

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibPreprocessEntry(
    Arbiter: PARBITER_INSTANCE,
    ArbState: PARBITER_ALLOCATION_STATE,
) -> NTSTATUS {
    UNREFERENCED_PARAMETER!(Arbiter);
    UNREFERENCED_PARAMETER!(ArbState);
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibSortArbitrationList(ArbitrationList: PLIST_ENTRY) -> NTSTATUS {
    let mut Sorted: LIST_ENTRY = core::mem::zeroed();

    PAGED_CODE!();

    InitializeListHead(&raw mut Sorted);

    while !IsListEmpty(ArbitrationList) {
        let ListEntry: PLIST_ENTRY = RemoveHeadList(ArbitrationList);
        let Entry: PARBITER_LIST_ENTRY = CONTAINING_RECORD!(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
        let mut Position: PLIST_ENTRY;

        Position = Sorted.Flink;
        while Position != &raw mut Sorted {
            if (*CONTAINING_RECORD!(Position, ARBITER_LIST_ENTRY, ListEntry)).WorkSpace
                > (*Entry).WorkSpace
            {
                break;
            }
            Position = (*Position).Flink;
        }

        InsertTailList(Position, ListEntry);
    }

    if !IsListEmpty(&raw mut Sorted) {
        (*ArbitrationList).Flink = Sorted.Flink;
        (*Sorted.Flink).Blink = ArbitrationList;
        (*ArbitrationList).Blink = Sorted.Blink;
        (*Sorted.Blink).Flink = ArbitrationList;
    }

    return STATUS_SUCCESS;
}
unsafe extern "C" fn ArbpFreeAllocationWorkSpaces(ArbState: PARBITER_ALLOCATION_STATE) {
    let mut Current: PARBITER_ALLOCATION_STATE;

    PAGED_CODE!();

    Current = ArbState;
    while !(*Current).Entry.is_null() {
        if ((*Current).Flags & (ARBITER_STATE_FLAG_WORKSPACE as UINT16)) != 0 {
            ExFreePoolWithTag((*Current).WorkSpace as PVOID, TAG_ARBITER as ULONG);
            (*Current).WorkSpace = 0;
            (*Current).Flags &= !(ARBITER_STATE_FLAG_WORKSPACE as UINT16);
        }
        Current = Current.offset(1);
    }
}
unsafe extern "C" fn ArbpReportConflictingRanges(
    Arbiter: PARBITER_INSTANCE,
    Minimum: ULONGLONG,
    Maximum: ULONGLONG,
) {
    let mut Iterator: RTL_RANGE_LIST_ITERATOR = core::mem::zeroed();
    let mut Range: PRTL_RANGE = core::ptr::null_mut();
    let mut Overlapped: BOOLEAN = FALSE;

    if !NT_SUCCESS(RtlGetFirstRange(
        (*Arbiter).PossibleAllocation,
        &raw mut Iterator,
        &raw mut Range,
    )) {
        return;
    }

    while !Range.is_null() {
        if (*Range).Start <= Maximum && (*Range).End >= Minimum {
            Overlapped = TRUE;
            DbgPrint(
                b"  0x%I64x..0x%I64x held by %p, attributes 0x%x%s\n\0".as_ptr() as PCSTR,
                (*Range).Start,
                (*Range).End,
                (*Range).Owner,
                (*Range).Attributes as ::core::ffi::c_int,
                if ((*Range).Flags & (RTL_RANGE_SHARED as u8)) != 0 {
                    b" (shared)\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
        }

        if !NT_SUCCESS(RtlGetNextRange(
            &raw mut Iterator,
            &raw mut Range,
            TRUE,
        )) {
            break;
        }
    }

    if Overlapped == 0 {
        DbgPrint(
            b"  nothing holds 0x%I64x..0x%I64x, so it is outside what this arbiter was given\n\0"
                .as_ptr() as PCSTR,
            Minimum,
            Maximum,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibAllocateEntry(
    Arbiter: PARBITER_INSTANCE,
    ArbState: PARBITER_ALLOCATION_STATE,
) -> NTSTATUS {
    let mut Current: PARBITER_ALLOCATION_STATE = ArbState;
    let mut Backtracking: BOOLEAN = FALSE;
    let mut McfgConflict: BOOLEAN = FALSE;
    let mut DeadlineReported: BOOLEAN = FALSE;
    let mut Deadline: ULONGLONG;
    let mut HardDeadline: ULONGLONG;
    let mut Status: NTSTATUS;

    Deadline = KeQueryInterruptTime().wrapping_add(ARBITER_ALLOCATE_DEADLINE);
    HardDeadline = KeQueryInterruptTime().wrapping_add(ARBITER_ALLOCATE_HARD_DEADLINE);

    'entries: while Current >= ArbState && !(*Current).Entry.is_null() {
        let mut Found: BOOLEAN = FALSE;
        let mut RetrySameRange: BOOLEAN = FALSE;

        if KeQueryInterruptTime() > HardDeadline {
            DbgPrint(
                b"Arbiter %ws: abandoning allocation at entry %u, the search is not terminating\n\0"
                    as *const u8 as PCSTR,
                if !(*Arbiter).Name.is_null() {
                    (*Arbiter).Name as *const u16
                } else {
                    RUSTCONV_WSTR_AEEE8D02E860.as_ptr()
                },
                Current.offset_from(ArbState) as ULONG,
            );

            if !(*ArbState).Entry.is_null() {
                (*(*ArbState).Entry).Result = ArbiterResultExternalConflict;
            }

            ArbpFreeAllocationWorkSpaces(ArbState);
            return STATUS_UNSUCCESSFUL;
        }

        Status = (*Arbiter).PreprocessEntry.expect("non-null function pointer")(
            Arbiter, Current,
        );
        if !NT_SUCCESS(Status) {
            ArbpFreeAllocationWorkSpaces(ArbState);
            return Status;
        }

        'attempt: {
            if Backtracking != 0 {
                let FailedAlternative: PARBITER_ALTERNATIVE = (*Current).CurrentAlternative;

                (*Current.offset(1 as isize)).CurrentAlternative =
                    core::ptr::null_mut();
                Backtracking = FALSE;

                if FailedAlternative.is_null()
                    || (*FailedAlternative).Length == 0
                {
                    break 'attempt;
                }

                (*Arbiter).BacktrackAllocation.expect("non-null function pointer")(
                    Arbiter, Current,
                );

                if (*Current).Start > (*Current).CurrentMinimum
                    && ((*FailedAlternative).Flags & ARBITER_ALTERNATIVE_FLAG_FIXED) == 0
                {
                    if KeQueryInterruptTime() <= Deadline {
                        (*Current).CurrentMaximum =
                            (*Current).Start.wrapping_sub(1);
                        RetrySameRange = TRUE;
                    } else if DeadlineReported == 0 {
                        DeadlineReported = TRUE;
                        DbgPrint(
                            b"Arbiter %ws: allocation deadline expired, giving up range refinement\n\0"
                                as *const u8 as PCSTR,
                            if !(*Arbiter).Name.is_null() {
                                (*Arbiter).Name as *const u16
                            } else {
                                RUSTCONV_WSTR_AEEE8D02E860.as_ptr()
                            },
                        );
                    }
                }
            }

            loop {
                if RetrySameRange == 0 {
                    if (*Arbiter).GetNextAllocationRange.expect("non-null function pointer")(
                        Arbiter, Current,
                    ) == 0
                    {
                        break;
                    }
                }
                RetrySameRange = FALSE;

                if (*Arbiter).FindSuitableRange.expect("non-null function pointer")(
                    Arbiter, Current,
                ) != 0
                {
                    Found = TRUE;
                    break;
                }
            }

            if Found != 0 {
                if (*(*Current).CurrentAlternative).Length != 0 {
                    (*Arbiter).AddAllocation.expect("non-null function pointer")(
                        Arbiter, Current,
                    );
                } else {
                    (*(*Current).Entry).Result = ArbiterResultNullRequest;
                }

                Current = Current.offset(1);
                continue 'entries;
            }
        }

        // Backtrack:
        if ((*Current).Flags & (ARBITER_STATE_FLAG_MCFG_CONFLICT as u16)) != 0 {
            McfgConflict = TRUE;
        }

        if Current == ArbState {
            if !(*Current).Entry.is_null() {
                (*(*Current).Entry).Result = ArbiterResultExternalConflict;
            }

            DbgPrint(
                b"Arbiter %ws: nowhere to put 0x%I64x..0x%I64x\n\0" as *const u8 as PCSTR,
                if !(*Arbiter).Name.is_null() {
                    (*Arbiter).Name as *const u16
                } else {
                    RUSTCONV_WSTR_AEEE8D02E860.as_ptr()
                },
                (*Current).CurrentMinimum,
                (*Current).CurrentMaximum,
            );
            ArbpReportConflictingRanges(
                Arbiter,
                (*Current).CurrentMinimum,
                (*Current).CurrentMaximum,
            );

            ArbpFreeAllocationWorkSpaces(ArbState);
            return if McfgConflict != 0 {
                STATUS_BAD_MCFG_TABLE
            } else {
                STATUS_UNSUCCESSFUL
            };
        }
        Backtracking = TRUE;
        Current = Current.offset(-1);
    }

    Current = ArbState;
    while !(*Current).Entry.is_null() {
        (*(*Current).Entry).SelectedAlternative = (*(*Current).CurrentAlternative).Descriptor;

        if !(*(*Current).Entry).Assignment.is_null() && (*Arbiter).PackResource.is_some() {
            Status = (*Arbiter).PackResource.expect("non-null function pointer")(
                (*(*Current).CurrentAlternative).Descriptor,
                (*Current).Start,
                (*(*Current).Entry).Assignment,
            );
            if !NT_SUCCESS(Status) {
                ArbpFreeAllocationWorkSpaces(ArbState);
                return Status;
            }
        }

        if (*(*Current).Entry).Result != ArbiterResultNullRequest {
            (*(*Current).Entry).Result = ArbiterResultSuccess;
        }

        Current = Current.offset(1);
    }

    ArbpFreeAllocationWorkSpaces(ArbState);
    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Arbitration Library
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Entry allocation pipeline
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
// /* Bound the search */
// #define ARBITER_ALLOCATE_DEADLINE  (10ULL * 1000 * 1000)   /* 1 second */
// #define ARBITER_ALLOCATE_HARD_DEADLINE  (10ULL * 1000 * 1000 * 10)   /* 10 seconds */
//
// /**
//  * @brief
//  * Per-entry hook invoked before each placement attempt. The
//  * library default does nothing and succeeds.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance placing the entry.
//  *
//  * @param[in,out] ArbState
//  * The allocation state of the entry about to be placed.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibPreprocessEntry(
//          PARBITER_INSTANCE Arbiter,
//             PARBITER_ALLOCATION_STATE ArbState)
// {
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER(Arbiter);
//     UNREFERENCED_PARAMETER(ArbState);
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Orders an arbitration list most-constrained first, so fixed
//  * requirements are placed before flexible ones can steal their
//  * ranges.
//  *
//  * @param[in,out] ArbitrationList
//  * The list of ARBITER_LIST_ENTRY nodes to sort, keyed on the
//  * WorkSpace constrainedness score in ascending order. The sort is
//  * stable: equally scored entries keep their relative order.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  *
//  * @remarks
//  * Without this ordering a wide requirement placed first keeps
//  * landing on the one range a later fixed requirement must have,
//  * forcing a backtrack for every such collision.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibSortArbitrationList(
//             PLIST_ENTRY ArbitrationList)
// {
//     LIST_ENTRY Sorted;
//
//     PAGED_CODE();
//
//     InitializeListHead(&Sorted);
//
//     while (!IsListEmpty(ArbitrationList))
//     {
//         PLIST_ENTRY ListEntry = RemoveHeadList(ArbitrationList);
//         PARBITER_LIST_ENTRY Entry = CONTAINING_RECORD(ListEntry, ARBITER_LIST_ENTRY, ListEntry);
//         PLIST_ENTRY Position;
//
//         for (Position = Sorted.Flink; Position != &Sorted; Position = Position->Flink)
//         {
//             if (CONTAINING_RECORD(Position, ARBITER_LIST_ENTRY, ListEntry)->WorkSpace >
//                 Entry->WorkSpace)
//             {
//                 break;
//             }
//         }
//
//         InsertTailList(Position, ListEntry);  /* Insert before Position */
//     }
//
//     /* Return the data back to ArbitrationList! */
//     if (!IsListEmpty(&Sorted))
//     {
//         ArbitrationList->Flink = Sorted.Flink;
//         Sorted.Flink->Blink = ArbitrationList;
//         ArbitrationList->Blink = Sorted.Blink;
//         Sorted.Blink->Flink = ArbitrationList;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /**
//  * @brief
//  * Releases the per-entry workspace a PreprocessEntry override
//  * allocated, across the whole allocation stack.
//  *
//  * @param[in,out] ArbState
//  * The first state of the allocation stack.
//  */
// CODE_SEG("PAGE")
// static
// VOID
// ArbpFreeAllocationWorkSpaces(
//             PARBITER_ALLOCATION_STATE ArbState)
// {
//     PARBITER_ALLOCATION_STATE Current;
//
//     PAGED_CODE();
//
//     for (Current = ArbState; Current->Entry != NULL; ++Current)
//     {
//         if (Current->Flags & ARBITER_STATE_FLAG_WORKSPACE)
//         {
//             ExFreePoolWithTag((PVOID)Current->WorkSpace, TAG_ARBITER);
//             Current->WorkSpace = 0;
//             Current->Flags &= ~ARBITER_STATE_FLAG_WORKSPACE;
//         }
//     }
// }
//
// /**
//  * @brief
//  * Says what an arbiter is already holding across a window.
//  *
//  * The bare "arbitration failed" a caller reports names the device that could
//  * not be placed but not what stopped it, which is the half worth knowing: a
//  * fixed requirement can fail either because the window belongs to somebody
//  * else or because it was never the arbiter's to give. Both look the same from
//  * outside.
//  *
//  * @param[in] Arbiter
//  * The arbiter whose bookkeeping to read.
//  *
//  * @param[in] Minimum
//  * The bottom of the window that could not be satisfied.
//  *
//  * @param[in] Maximum
//  * The top of it.
//  */
// CODE_SEG("PAGE")
// static
// VOID
// ArbpReportConflictingRanges(
//          PARBITER_INSTANCE Arbiter,
//          ULONGLONG Minimum,
//          ULONGLONG Maximum)
// {
//     RTL_RANGE_LIST_ITERATOR Iterator;
//     PRTL_RANGE Range;
//     BOOLEAN Overlapped = FALSE;
//
//     PAGED_CODE();
//
//     if (!NT_SUCCESS(RtlGetFirstRange(Arbiter->PossibleAllocation, &Iterator, &Range)))
//         return;
//
//     while (Range != NULL)
//     {
//         if ((Range->Start <= Maximum) && (Range->End >= Minimum))
//         {
//             Overlapped = TRUE;
//             DPRINT1("  0x%I64x..0x%I64x held by %p, attributes 0x%x%s\n",
//                     Range->Start, Range->End, Range->Owner, Range->Attributes,
//                     (Range->Flags & RTL_RANGE_SHARED) ? " (shared)" : "");
//         }
//
//         if (!NT_SUCCESS(RtlGetNextRange(&Iterator, &Range, TRUE)))
//             break;
//     }
//
//     if (!Overlapped)
//     {
//         DPRINT1("  nothing holds 0x%I64x..0x%I64x, so it is outside what this "
//                 "arbiter was given\n",
//                 Minimum, Maximum);
//     }
// }
//
// /**
//  * @brief
//  * Places the whole stack of entries with backtracking: when an
//  * entry cannot be placed, the previous entry's tentative choice is
//  * withdrawn and varied, first lower within its window, then via
//  * its next window or alternative.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose walker callbacks
//  * (GetNextAllocationRange, FindSuitableRange, AddAllocation,
//  * BacktrackAllocation) drive each placement.
//  *
//  * @param[in,out] ArbState
//  * The first ARBITER_ALLOCATION_STATE of the allocation stack, one
//  * per entry in most-constrained-first order, terminated by a state
//  * whose Entry is NULL.
//  *
//  * @return
//  * Returns STATUS_SUCCESS once every entry has a placement (the
//  * results are packed into each entry's Assignment and Result),
//  * STATUS_UNSUCCESSFUL if even the first entry has no solution, or
//  * a PreprocessEntry / PackResource failure status.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibAllocateEntry(
//          PARBITER_INSTANCE Arbiter,
//             PARBITER_ALLOCATION_STATE ArbState)
// {
//     PARBITER_ALLOCATION_STATE Current = ArbState;
//     BOOLEAN Backtracking = FALSE;
//     BOOLEAN McfgConflict = FALSE;
//     BOOLEAN DeadlineReported = FALSE;
//     ULONGLONG Deadline;
//     ULONGLONG HardDeadline;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     Deadline = KeQueryInterruptTime() + ARBITER_ALLOCATE_DEADLINE;
//     HardDeadline = KeQueryInterruptTime() + ARBITER_ALLOCATE_HARD_DEADLINE;
//
//     while (Current >= ArbState && Current->Entry != NULL)
//     {
//         BOOLEAN Found = FALSE;
//         BOOLEAN RetrySameRange = FALSE;
//
//         /*
//          * A search that cannot terminate must fail rather than wedge the
//          * machine.
//          */
//         if (KeQueryInterruptTime() > HardDeadline)
//         {
//             DPRINT1("Arbiter %ws: abandoning allocation at entry %u, the search "
//                     "is not terminating\n",
//                     Arbiter->Name ? Arbiter->Name : L"(unnamed)",
//                     (ULONG)(Current - ArbState));
//
//             if (ArbState->Entry != NULL)
//                 ArbState->Entry->Result = ArbiterResultExternalConflict;
//
//             ArbpFreeAllocationWorkSpaces(ArbState);
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         Status = Arbiter->PreprocessEntry(Arbiter, Current);
//         if (!NT_SUCCESS(Status))
//         {
//             ArbpFreeAllocationWorkSpaces(ArbState);
//             return Status;
//         }
//
//         if (Backtracking)
//         {
//             PARBITER_ALTERNATIVE FailedAlternative = Current->CurrentAlternative;
//
//             /*
//              * The entry after this one could not be placed. Withdraw this
//              * entry's tentative choice and vary it: first lower within the same
//              * window, then via the next window / alternative.
//              */
//             (Current + 1)->CurrentAlternative = NULL;
//             Backtracking = FALSE;
//
//             if (FailedAlternative == NULL || FailedAlternative->Length == 0)
//                 goto Backtrack;  /* Nothing to vary here */
//
//             Arbiter->BacktrackAllocation(Arbiter, Current);
//
//             /*
//              * Retrying one placement lower walks the window down an alignment
//              * unit at a time, so it has to be bounded, or searching a bridge's
//              * multi-gigabyte window will hang the system.
//              */
//             if (Current->Start > Current->CurrentMinimum &&
//                 !(FailedAlternative->Flags & ARBITER_ALTERNATIVE_FLAG_FIXED))
//             {
//                 if (KeQueryInterruptTime() <= Deadline)
//                 {
//                     Current->CurrentMaximum = Current->Start - 1;
//                     RetrySameRange = TRUE;
//                 }
//                 else if (!DeadlineReported)
//                 {
//                     DeadlineReported = TRUE;
//                     DPRINT1("Arbiter %ws: allocation deadline expired, giving up "
//                             "range refinement\n",
//                             Arbiter->Name ? Arbiter->Name : L"(unnamed)");
//                 }
//             }
//         }
//
//         for (;;)
//         {
//             if (!RetrySameRange)
//             {
//                 if (!Arbiter->GetNextAllocationRange(Arbiter, Current))
//                     break;
//             }
//             RetrySameRange = FALSE;
//
//             if (Arbiter->FindSuitableRange(Arbiter, Current))
//             {
//                 Found = TRUE;
//                 break;
//             }
//         }
//
//         if (Found)
//         {
//             if (Current->CurrentAlternative->Length != 0)
//                 Arbiter->AddAllocation(Arbiter, Current);
//             else
//                 Current->Entry->Result = ArbiterResultNullRequest;
//
//             Current++;
//             continue;
//         }
//
// Backtrack:
//         if (Current->Flags & ARBITER_STATE_FLAG_MCFG_CONFLICT)
//             McfgConflict = TRUE;
//
//         if (Current == ArbState)
//         {
//             /* Even the first entry has no solution. */
//             if (Current->Entry != NULL)
//                 Current->Entry->Result = ArbiterResultExternalConflict;
//
//             DPRINT1("Arbiter %ws: nowhere to put 0x%I64x..0x%I64x\n",
//                     Arbiter->Name ? Arbiter->Name : L"(unnamed)",
//                     Current->CurrentMinimum, Current->CurrentMaximum);
//             ArbpReportConflictingRanges(Arbiter,
//                                         Current->CurrentMinimum,
//                                         Current->CurrentMaximum);
//
//             ArbpFreeAllocationWorkSpaces(ArbState);
//             return McfgConflict ? STATUS_BAD_MCFG_TABLE : STATUS_UNSUCCESSFUL;
//         }
//         Backtracking = TRUE;
//         Current--;
//     }
//
//     /* Complete solution found; report it back to the requesters. */
//     for (Current = ArbState; Current->Entry != NULL; ++Current)
//     {
//         Current->Entry->SelectedAlternative = Current->CurrentAlternative->Descriptor;
//
//         if (Current->Entry->Assignment != NULL && Arbiter->PackResource != NULL)
//         {
//             Status = Arbiter->PackResource(Current->CurrentAlternative->Descriptor,
//                                            Current->Start,
//                                            Current->Entry->Assignment);
//             if (!NT_SUCCESS(Status))
//             {
//                 ArbpFreeAllocationWorkSpaces(ArbState);
//                 return Status;
//             }
//         }
//
//         if (Current->Entry->Result != ArbiterResultNullRequest)
//             Current->Entry->Result = ArbiterResultSuccess;
//     }
//
//     ArbpFreeAllocationWorkSpaces(ArbState);
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
