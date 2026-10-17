#[allow(unused_imports)]
use crate::prelude::*;
// rustconv-assembly: whole-tu
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, KeWaitForSingleObject, PARBITER_INSTANCE, PARBITER_PARAMETERS, PCM_FULL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PIO_RESOURCE_DESCRIPTOR, PRTL_RANGE_LIST, RtlAddRange, RtlDeleteOwnersRanges, RtlFreeRangeList, RtlInitializeRangeList};

// ---- file-scope preamble (translated) ----
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _ARBITER_SEED_WINDOW {
    pub Start: ULONGLONG,
    pub End: ULONGLONG,
}
pub type ARBITER_SEED_WINDOW = _ARBITER_SEED_WINDOW;
pub type PARBITER_SEED_WINDOW = *mut _ARBITER_SEED_WINDOW;
unsafe extern "C" fn ArbpAcquireLock(Arbiter: PARBITER_INSTANCE) {
    if KeGetCurrentIrql() as ::core::ffi::c_int > (APC_LEVEL) as _ {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        RtlAssert(
            b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"sdk/lib/drivers/arbiter/handler.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            33 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }
    KeWaitForSingleObject(
        (*Arbiter).MutexEvent as PVOID,
        _KWAIT_REASON_Executive,
        (0i32) as i8,
        FALSE,
        core::ptr::null_mut::<_LARGE_INTEGER>(),
    );
}
unsafe extern "C" fn ArbpReleaseLock(Arbiter: PARBITER_INSTANCE) {
    KeSetEvent(
        (*Arbiter).MutexEvent as PRKEVENT,
        IO_NO_INCREMENT as KPRIORITY,
        FALSE as BOOLEAN,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibHandler(
    Context: PVOID,
    Action: ARBITER_ACTION,
    Parameters: PARBITER_PARAMETERS,
) -> NTSTATUS {
    let Arbiter: PARBITER_INSTANCE = Context as PARBITER_INSTANCE;
    let Status: NTSTATUS;

    ArbpAcquireLock(Arbiter);

    // NTDDI 0x502: the pre-Vista arms. Each action takes its parameters unpacked, and the
    // Vista-only TransactionEvent bookkeeping after the switch does not exist.
    let P = core::ptr::addr_of_mut!((*Parameters).Parameters);
    match Action {
        ArbiterActionTestAllocation => {
            Status = (*Arbiter).TestAllocation.unwrap_unchecked()(Arbiter, (*P).TestAllocation.ArbitrationList);
        }
        ArbiterActionRetestAllocation => {
            Status = (*Arbiter).RetestAllocation.unwrap_unchecked()(Arbiter, (*P).RetestAllocation.ArbitrationList);
        }
        ArbiterActionCommitAllocation => {
            Status = (*Arbiter).CommitAllocation.unwrap_unchecked()(Arbiter);
        }
        ArbiterActionRollbackAllocation => {
            Status = (*Arbiter).RollbackAllocation.unwrap_unchecked()(Arbiter);
        }
        ArbiterActionBootAllocation => {
            Status = (*Arbiter).BootAllocation.unwrap_unchecked()(Arbiter, (*P).BootAllocation.ArbitrationList);
        }
        ArbiterActionQueryConflict => {
            Status = (*Arbiter).QueryConflict.unwrap_unchecked()(
                Arbiter,
                (*P).QueryConflict.PhysicalDeviceObject,
                (*P).QueryConflict.ConflictingResource,
                (*P).QueryConflict.ConflictCount,
                (*P).QueryConflict.Conflicts,
            );
        }
        ArbiterActionQueryArbitrate => {
            Status = (*Arbiter).QueryArbitrate.unwrap_unchecked()(Arbiter, (*P).QueryArbitrate.ArbitrationList);
        }
        // Actions the interface defines but this arbiter does not carry out. AddReserved is
        // one: the instance slot exists and is defaulted, but nothing routes to it.
        ArbiterActionQueryAllocatedResources | ArbiterActionWriteReservedResources | ArbiterActionAddReserved => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        _ => {
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    ArbpReleaseLock(Arbiter);
    return Status;
}
unsafe extern "C" fn ArbpSeedRangeList(
    Arbiter: PARBITER_INSTANCE,
    ResourceCount: ULONG,
    Resources: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    RangeList: PRTL_RANGE_LIST,
) -> NTSTATUS {
    let Windows: PARBITER_SEED_WINDOW;
    let mut WindowCount: ULONG = 0;
    let mut Next: ULONGLONG;
    let mut Index: ULONG;

    PAGED_CODE!();

    RtlFreeRangeList(RangeList);
    RtlInitializeRangeList(RangeList);

    if (*Arbiter).UnpackResource.is_none() || ResourceCount == 0 {
        return STATUS_SUCCESS;
    }

    Windows = ExAllocatePoolWithTag(
        PagedPool,
        (ResourceCount as SIZE_T)
            .wrapping_mul(core::mem::size_of::<ARBITER_SEED_WINDOW>() as SIZE_T),
        TAG_ARBITER,
    ) as PARBITER_SEED_WINDOW;
    if Windows.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Index = 0;
    while Index < ResourceCount {
        let mut Start: ULONGLONG = 0;
        let mut Length: ULONGLONG = 0;
        let mut Insert: ULONG;

        if !((*Resources.add(Index as usize)).Type != (*Arbiter).ResourceType as UCHAR
            && !((*Resources.add(Index as usize)).Type == (CmResourceTypeMemoryLarge as u8)
                && (*Arbiter).ResourceType == (CmResourceTypeMemory as i32)))
        {
            (*Arbiter)
                .UnpackResource
                .unwrap_unchecked()(
                Resources.add(Index as usize),
                &mut Start,
                &mut Length,
            );
            if Length != 0 {
                Insert = WindowCount;
                while Insert > 0 {
                    if (*Windows.add((Insert - 1) as usize)).Start <= Start {
                        break;
                    }
                    *Windows.add(Insert as usize) =
                        *Windows.add((Insert - 1) as usize);
                    Insert -= 1;
                }
                (*Windows.add(Insert as usize)).Start = Start;
                (*Windows.add(Insert as usize)).End =
                    Start + Length - 1;
                WindowCount += 1;
            }
        }
        Index += 1;
    }

    if WindowCount == 0 {
        ExFreePoolWithTag(Windows as PVOID, TAG_ARBITER);
        return STATUS_SUCCESS;
    }

    Next = 0;

    Index = 0;
    while Index < WindowCount {
        if (*Windows.add(Index as usize)).Start > Next {
            RtlAddRange(
                RangeList,
                Next,
                (*Windows.add(Index as usize)).Start - 1,
                0 as UCHAR,
                RTL_RANGE_LIST_ADD_IF_CONFLICT as ULONG,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }

        if (*Windows.add(Index as usize)).End >= Next {
            Next = (*Windows.add(Index as usize)).End + 1;
        }

        if (*Windows.add(Index as usize)).End == ARBITER_MAXIMUM_ADDRESS {
            ExFreePoolWithTag(Windows as PVOID, TAG_ARBITER);
            return STATUS_SUCCESS;
        }
        Index += 1;
    }

    RtlAddRange(
        RangeList,
        Next,
        ARBITER_MAXIMUM_ADDRESS,
        0 as UCHAR,
        RTL_RANGE_LIST_ADD_IF_CONFLICT as ULONG,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    ExFreePoolWithTag(Windows as PVOID, TAG_ARBITER);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibStartArbiter(
    Arbiter: PARBITER_INSTANCE,
    StartResources: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let Full: PCM_FULL_RESOURCE_DESCRIPTOR;

    if StartResources.is_null() || (*StartResources).Count == 0 {
        return STATUS_SUCCESS;
    }

    Full = core::ptr::addr_of_mut!((*StartResources).List[0])
        as PCM_FULL_RESOURCE_DESCRIPTOR;

    ArbpSeedRangeList(
        Arbiter,
        (*Full).PartialResourceList.Count,
        core::ptr::addr_of_mut!((*Full).PartialResourceList.PartialDescriptors)
            as PCM_PARTIAL_RESOURCE_DESCRIPTOR,
        (*Arbiter).Allocation,
    )
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibReserveRange(
    Arbiter: PARBITER_INSTANCE,
    Start: ULONGLONG,
    End: ULONGLONG,
    Owner: PVOID,
    Shared: BOOLEAN,
) {
    ArbpAcquireLock(Arbiter);
    RtlAddRange(
        (*Arbiter).Allocation,
        Start,
        End,
        0 as UCHAR,
        (RTL_RANGE_LIST_ADD_IF_CONFLICT
            | (if Shared as ::core::ffi::c_int != 0 {
                RTL_RANGE_LIST_ADD_SHARED
            } else {
                0
            })) as ULONG,
        core::ptr::null_mut(),
        Owner,
    );
    ArbpReleaseLock(Arbiter);
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibReleaseResources(Arbiter: PARBITER_INSTANCE, Owner: PVOID) {
    ArbpAcquireLock(Arbiter);
    RtlDeleteOwnersRanges((*Arbiter).Allocation, Owner);
    ArbpReleaseLock(Arbiter);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Arbitration Library
//  * LICENSE:     MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Action dispatch and arbiter startup
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
// /* DISPATCH *******************************************************************/
//
// /**
//  * @brief
//  * Acquires the arbiter's per-instance lock: one arbitration at a
//  * time.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance to lock.
//  */
// CODE_SEG("PAGE")
// static
// VOID
// ArbpAcquireLock(
//          PARBITER_INSTANCE Arbiter)
// {
//     PAGED_CODE();
//     KeWaitForSingleObject(Arbiter->MutexEvent, Executive, KernelMode, FALSE, NULL);
// }
//
// /**
//  * @brief
//  * Releases the arbiter's per-instance lock.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance to unlock.
//  */
// CODE_SEG("PAGE")
// static
// VOID
// ArbpReleaseLock(
//          PARBITER_INSTANCE Arbiter)
// {
//     PAGED_CODE();
//     KeSetEvent(Arbiter->MutexEvent, IO_NO_INCREMENT, FALSE);
// }
//
// /**
//  * @brief
//  * The QueryArbitrate action default: reports that this arbiter is
//  * willing to arbitrate the given list.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance being queried.
//  *
//  * @param[in,out] Parameters
//  * The action parameters carrying the arbitration list (pre-Vista
//  * builds receive the list directly).
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// #if (NTDDI_VERSION >= NTDDI_VISTA)
// ArbiterLibQueryArbitrate(
//          PARBITER_INSTANCE Arbiter,
//             PARBITER_QUERY_ARBITRATE_PARAMETERS Parameters)
// {
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(Arbiter);
//     UNREFERENCED_PARAMETER(Parameters);
//     return STATUS_SUCCESS;
// }
// #else
// ArbiterLibQueryArbitrate(
//          PARBITER_INSTANCE Arbiter,
//          PLIST_ENTRY ArbitrationList)
// {
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(Arbiter);
//     UNREFERENCED_PARAMETER(ArbitrationList);
//     return STATUS_SUCCESS;
// }
// #endif
//
// /**
//  * @brief
//  * The AddReserved action default: nothing to reserve.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance the reservation was aimed at.
//  *
//  * @param[in,out] Parameters
//  * The action parameters (pre-Vista builds receive the requirement
//  * and resource descriptors directly).
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// #if (NTDDI_VERSION >= NTDDI_VISTA)
// ArbiterLibAddReserved(
//          PARBITER_INSTANCE Arbiter,
//             PARBITER_ADD_RESERVED_PARAMETERS Parameters)
// {
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(Arbiter);
//     UNREFERENCED_PARAMETER(Parameters);
//     return STATUS_SUCCESS;
// }
// #else
// ArbiterLibAddReserved(
//          PARBITER_INSTANCE Arbiter,
//              PIO_RESOURCE_DESCRIPTOR Requirement,
//              PCM_PARTIAL_RESOURCE_DESCRIPTOR Resource)
// {
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER(Arbiter);
//     UNREFERENCED_PARAMETER(Requirement);
//     UNREFERENCED_PARAMETER(Resource);
//     return STATUS_SUCCESS;
// }
// #endif
//
// /**
//  * @brief
//  * The single ARBITER_ACTION dispatcher every exported
//  * ARBITER_INTERFACE points its ArbiterHandler at; Context is the
//  * ARBITER_INSTANCE.
//  *
//  * @param[in] Context
//  * The ARBITER_INSTANCE the interface was created for.
//  *
//  * @param[in] Action
//  * The ARBITER_ACTION to perform.
//  *
//  * @param[in,out] Parameters
//  * The parameters union matching the action.
//  *
//  * @return
//  * Returns the dispatched action's status,
//  * STATUS_NOT_IMPLEMENTED for a known but unrouted action, or
//  * STATUS_INVALID_PARAMETER for an unrecognized one.
//  *
//  * @remarks
//  * The call runs under the instance lock, so a Test and its later
//  * Commit or Rollback apply to the same transaction. On Vista+ the
//  * transaction event tracks that window: Test and Retest arm it,
//  * Commit and Rollback resolve it and wake waiters.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibHandler(
//          PVOID Context,
//          ARBITER_ACTION Action,
//             PARBITER_PARAMETERS Parameters)
// {
//     PARBITER_INSTANCE Arbiter = (PARBITER_INSTANCE)Context;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     ArbpAcquireLock(Arbiter);
//
//     switch (Action)
//     {
//         case ArbiterActionTestAllocation:
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//             Status = Arbiter->TestAllocation(Arbiter, &Parameters->Parameters.TestAllocation);
// #else
//             Status = Arbiter->TestAllocation(
//                 Arbiter,
//                 Parameters->Parameters.TestAllocation.ArbitrationList);
// #endif
//             break;
//
//         case ArbiterActionRetestAllocation:
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//             Status = Arbiter->RetestAllocation(Arbiter, &Parameters->Parameters.RetestAllocation);
// #else
//             Status = Arbiter->RetestAllocation(
//                 Arbiter,
//                 Parameters->Parameters.RetestAllocation.ArbitrationList);
// #endif
//             break;
//
//         case ArbiterActionCommitAllocation:
//             Status = Arbiter->CommitAllocation(Arbiter);
//             break;
//
//         case ArbiterActionRollbackAllocation:
//             Status = Arbiter->RollbackAllocation(Arbiter);
//             break;
//
//         case ArbiterActionBootAllocation:
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//             Status = Arbiter->BootAllocation(Arbiter, &Parameters->Parameters.BootAllocation);
// #else
//             Status = Arbiter->BootAllocation(
//                 Arbiter,
//                 Parameters->Parameters.BootAllocation.ArbitrationList);
// #endif
//             break;
//
//         case ArbiterActionQueryConflict:
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//             Status = Arbiter->QueryConflict(Arbiter, &Parameters->Parameters.QueryConflict);
// #else
//             Status = Arbiter->QueryConflict(
//                 Arbiter,
//                 Parameters->Parameters.QueryConflict.PhysicalDeviceObject,
//                 Parameters->Parameters.QueryConflict.ConflictingResource,
//                 Parameters->Parameters.QueryConflict.ConflictCount,
//                 Parameters->Parameters.QueryConflict.Conflicts);
// #endif
//             break;
//
//         case ArbiterActionQueryArbitrate:
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//             Status = Arbiter->QueryArbitrate(Arbiter, &Parameters->Parameters.QueryArbitrate);
// #else
//             Status = Arbiter->QueryArbitrate(
//                 Arbiter,
//                 Parameters->Parameters.QueryArbitrate.ArbitrationList);
// #endif
//             break;
//
//         /*
//          * Actions the interface defines but this arbiter does not carry
//          * out. AddReserved is one: the instance slot exists and is
//          * defaulted, but nothing routes to it.
//          */
//         case ArbiterActionQueryAllocatedResources:
//         case ArbiterActionWriteReservedResources:
//         case ArbiterActionAddReserved:
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         default:
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//     }
//
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//     /* Track the tested-but-not-yet-committed window for transaction waiters. */
//     if (NT_SUCCESS(Status))
//     {
//         if (Action == ArbiterActionTestAllocation ||
//             Action == ArbiterActionRetestAllocation)
//         {
//             Arbiter->TransactionInProgress = TRUE;
//             KeClearEvent(Arbiter->TransactionEvent);
//         }
//         else if (Action == ArbiterActionCommitAllocation ||
//                  Action == ArbiterActionRollbackAllocation)
//         {
//             Arbiter->TransactionInProgress = FALSE;
//             KeSetEvent(Arbiter->TransactionEvent, IO_NO_INCREMENT, FALSE);
//         }
//     }
// #endif
//
//     ArbpReleaseLock(Arbiter);
//     return Status;
// }
//
// /* STARTUP ********************************************************************/
//
// /**
//  * @brief
//  * Seeds a range list with the resources the bus actually decodes:
//  * every descriptor of the arbiter's resource type becomes one
//  * unowned range.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance whose UnpackResource decodes the
//  * descriptors and whose resource type filters them.
//  *
//  * @param[in] ResourceCount
//  * The number of descriptors in Resources.
//  *
//  * @param[in] Resources
//  * The CM partial resource descriptors describing the decoded
//  * windows.
//  *
//  * @param[in,out] RangeList
//  * The range list to rebuild from the descriptors. What lands in it is
//  * everything outside those windows, because a range list records what is
//  * unavailable.
//  *
//  * @return
//  * STATUS_SUCCESS, or STATUS_INSUFFICIENT_RESOURCES.
//  */
//
// /**
//  * @brief
//  * One window a bus decodes, while the windows are being sorted.
//  */
// typedef struct _ARBITER_SEED_WINDOW
// {
//     ULONGLONG Start;
//     ULONGLONG End;
// } ARBITER_SEED_WINDOW, *PARBITER_SEED_WINDOW;
//
// CODE_SEG("PAGE")
// static
// NTSTATUS
// ArbpSeedRangeList(
//          PARBITER_INSTANCE Arbiter,
//          ULONG ResourceCount,
//          PCM_PARTIAL_RESOURCE_DESCRIPTOR Resources,
//             PRTL_RANGE_LIST RangeList)
// {
//     PARBITER_SEED_WINDOW Windows;
//     ULONG WindowCount = 0;
//     ULONGLONG Next;
//     ULONG Index;
//
//     PAGED_CODE();
//
//     RtlFreeRangeList(RangeList);
//     RtlInitializeRangeList(RangeList);
//
//     if ((Arbiter->UnpackResource == NULL) || (ResourceCount == 0))
//         return STATUS_SUCCESS;
//
//     Windows = ExAllocatePoolWithTag(PagedPool,
//                                     ResourceCount * sizeof(ARBITER_SEED_WINDOW),
//                                     TAG_ARBITER);
//     if (Windows == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Collect the windows of our own resource type, lowest first */
//     for (Index = 0; Index < ResourceCount; ++Index)
//     {
//         ULONGLONG Start, Length;
//         ULONG Insert;
//
//         /* A memory window of 4GB or more comes in the large memory form */
//         if ((Resources[Index].Type != (UCHAR)Arbiter->ResourceType) &&
//             !((Resources[Index].Type == CmResourceTypeMemoryLarge) &&
//               (Arbiter->ResourceType == CmResourceTypeMemory)))
//         {
//             continue;
//         }
//
//         Arbiter->UnpackResource(&Resources[Index], &Start, &Length);
//         if (Length == 0)
//             continue;
//
//         for (Insert = WindowCount; Insert > 0; --Insert)
//         {
//             if (Windows[Insert - 1].Start <= Start)
//                 break;
//
//             Windows[Insert] = Windows[Insert - 1];
//         }
//
//         Windows[Insert].Start = Start;
//         Windows[Insert].End = Start + Length - 1;
//         WindowCount++;
//     }
//
//     /*
//      * A bus that reports no window of this kind is not saying it decodes
//      * nothing; it is saying it has nothing to add. Leaving the list empty
//      * leaves every address available, which is how the root arbiters run.
//      */
//     if (WindowCount == 0)
//     {
//         ExFreePoolWithTag(Windows, TAG_ARBITER);
//         return STATUS_SUCCESS;
//     }
//
//     /*
//      * What goes in the list is what the arbiter may NOT hand out, so the
//      * windows themselves are the gaps in it. Recording the windows instead
//      * would wall off exactly the addresses the bus exists to give away.
//      */
//     Next = 0;
//
//     for (Index = 0; Index < WindowCount; ++Index)
//     {
//         if (Windows[Index].Start > Next)
//         {
//             RtlAddRange(RangeList, Next, Windows[Index].Start - 1, 0,
//                         RTL_RANGE_LIST_ADD_IF_CONFLICT, NULL, NULL);
//         }
//
//         /* Overlapping windows merge, so only ever move the mark forward */
//         if (Windows[Index].End >= Next)
//             Next = Windows[Index].End + 1;
//
//         /* A window running to the top of the space leaves no tail to block */
//         if (Windows[Index].End == ARBITER_MAXIMUM_ADDRESS)
//         {
//             ExFreePoolWithTag(Windows, TAG_ARBITER);
//             return STATUS_SUCCESS;
//         }
//     }
//
//     RtlAddRange(RangeList, Next, ARBITER_MAXIMUM_ADDRESS, 0,
//                 RTL_RANGE_LIST_ADD_IF_CONFLICT, NULL, NULL);
//
//     ExFreePoolWithTag(Windows, TAG_ARBITER);
//
//     return STATUS_SUCCESS;
// }
//
// #if (NTDDI_VERSION >= NTDDI_VISTA)
// /**
//  * @brief
//  * The InitializeRangeList callback default.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance the range list belongs to.
//  *
//  * @param[in] ResourceCount
//  * The number of descriptors in Resources.
//  *
//  * @param[in] Resources
//  * The CM partial resource descriptors describing the decoded
//  * windows.
//  *
//  * @param[in,out] RangeList
//  * The range list to rebuild from the descriptors.
//  *
//  * @return
//  * Returns STATUS_SUCCESS.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibInitializeRangeList(
//          PARBITER_INSTANCE Arbiter,
//          ULONG ResourceCount,
//          PCM_PARTIAL_RESOURCE_DESCRIPTOR Resources,
//             PRTL_RANGE_LIST RangeList)
// {
//     PAGED_CODE();
//     return ArbpSeedRangeList(Arbiter, ResourceCount, Resources, RangeList);
// }
// #endif
//
// /**
//  * @brief
//  * The StartArbiter action default: seeds the committed allocation
//  * with the windows the bus decodes, so arbitration starts from
//  * the hardware's real extent.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance being started.
//  *
//  * @param[in] StartResources
//  * The bus's decoded resources; NULL or empty means nothing to
//  * seed.
//  *
//  * @return
//  * Returns the range-list initialization status.
//  */
// CODE_SEG("PAGE")
// NTSTATUS
// NTAPI
// ArbiterLibStartArbiter(
//          PARBITER_INSTANCE Arbiter,
//          PCM_RESOURCE_LIST StartResources)
// {
//     PCM_FULL_RESOURCE_DESCRIPTOR Full;
//
//     PAGED_CODE();
//
//     if (StartResources == NULL || StartResources->Count == 0)
//         return STATUS_SUCCESS;
//
//     Full = &StartResources->List[0];
//
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//     /* Vista+ routes through the slot, so an arbiter may override the seeding. */
//     return Arbiter->InitializeRangeList(Arbiter,
//                                         Full->PartialResourceList.Count,
//                                         Full->PartialResourceList.PartialDescriptors,
//                                         Arbiter->Allocation);
// #else
//     return ArbpSeedRangeList(Arbiter,
//                              Full->PartialResourceList.Count,
//                              Full->PartialResourceList.PartialDescriptors,
//                              Arbiter->Allocation);
// #endif
// }
//
// /* EXTERNAL RESERVATIONS ******************************************************/
//
// /**
//  * @brief
//  * Commits a single [Start, End] range into the arbiter as an
//  * ordinary allocation owned by Owner.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance to reserve the range in.
//  *
//  * @param[in] Start
//  * The first address (or unit) of the range.
//  *
//  * @param[in] End
//  * The last address (or unit) of the range, inclusive.
//  *
//  * @param[in] Owner
//  * The owning device object, or NULL for a system reservation.
//  *
//  * @param[in] Shared
//  * TRUE if the range may be shared with other shareable owners.
//  *
//  * @remarks
//  * Used to record resources assigned outside the PnP transaction
//  * (e.g. a legacy device's IoAssignResources), so later arbitration
//  * sees them. Unlike a boot reservation this carries no
//  * BOOT_ALLOCATED attribute, so it is not treated as reclaimable.
//  */
// CODE_SEG("PAGE")
// VOID
// NTAPI
// ArbiterLibReserveRange(
//          PARBITER_INSTANCE Arbiter,
//          ULONGLONG Start,
//          ULONGLONG End,
//              PVOID Owner,
//          BOOLEAN Shared)
// {
//     PAGED_CODE();
//
//     ArbpAcquireLock(Arbiter);
//     RtlAddRange(Arbiter->Allocation, Start, End, 0,
//                 RTL_RANGE_LIST_ADD_IF_CONFLICT | (Shared ? RTL_RANGE_LIST_ADD_SHARED : 0),
//                 NULL, Owner);
//     ArbpReleaseLock(Arbiter);
// }
//
// /**
//  * @brief
//  * Releases every committed range owned by Owner.
//  *
//  * @param[in] Arbiter
//  * The arbiter instance to release the ranges from.
//  *
//  * @param[in] Owner
//  * The owning device object (a removed device's PDO).
//  *
//  * @remarks
//  * Without this a torn-down device's ranges stay reserved forever
//  * and later devices see phantom conflicts.
//  */
// CODE_SEG("PAGE")
// VOID
// NTAPI
// ArbiterLibReleaseResources(
//          PARBITER_INSTANCE Arbiter,
//          PVOID Owner)
// {
//     PAGED_CODE();
//
//     ArbpAcquireLock(Arbiter);
//     RtlDeleteOwnersRanges(Arbiter->Allocation, Owner);
//     ArbpReleaseLock(Arbiter);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// The two action defaults below are written by hand (rustconv --skip-fn): handler.c chooses
// between their signatures with `#if NTDDI_VERSION` placed between NTAPI and the name, which
// the scanner cannot parse. This build is NTDDI 0x502, so these are the pre-Vista arms.
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibQueryArbitrate(
    Arbiter: PARBITER_INSTANCE,
    ArbitrationList: PLIST_ENTRY,
) -> NTSTATUS {
    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(Arbiter);
    UNREFERENCED_PARAMETER!(ArbitrationList);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ArbiterLibAddReserved(
    Arbiter: PARBITER_INSTANCE,
    Requirement: PIO_RESOURCE_DESCRIPTOR,
    Resource: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(Arbiter);
    UNREFERENCED_PARAMETER!(Requirement);
    UNREFERENCED_PARAMETER!(Resource);
    STATUS_SUCCESS
}
