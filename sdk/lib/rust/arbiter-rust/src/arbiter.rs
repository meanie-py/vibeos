#[allow(unused_imports)]
use crate::prelude::*;
// Rust translation of sdk/lib/drivers/arbiter/arbiter.c (upstream #9484). The built arm is
// pre-Vista (NTDDI 0x502): no OrderingName, TransactionEvent or InitializeRangeList.
use crate::range::ArbiterLibAddAllocation;
use crate::entry::ArbiterLibAllocateEntry;
use crate::range::ArbiterLibBacktrackAllocation;
use crate::ordering::ArbiterLibDefaultAssignmentOrdering;
use crate::range::ArbiterLibFindSuitableRange;
use crate::ordering::ArbiterLibFreeOrderingList;
use crate::range::ArbiterLibGetNextAllocationRange;
use crate::range::ArbiterLibOverrideConflict;
use crate::entry::ArbiterLibPreprocessEntry;
use crate::handler::{ArbiterLibAddReserved, ArbiterLibQueryArbitrate, ArbiterLibStartArbiter};
use crate::transaction::{ArbiterLibBootAllocation, ArbiterLibCommitAllocation, ArbiterLibQueryConflict,
    ArbiterLibRetestAllocation, ArbiterLibRollbackAllocation, ArbiterLibTestAllocation};
use ntkrnl_sys::ddk::{CM_RESOURCE_TYPE, KEVENT, KeInitializeEvent, PARBITER_INSTANCE, PARB_TRANSLATE_ORDERING, PDEVICE_OBJECT, RTL_RANGE_LIST, RtlFreeRangeList, RtlInitializeRangeList, _EVENT_TYPE_SynchronizationEvent};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const ARBITER_SIG: u32 = 0x73627241;
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibDeleteInstance(Arbiter: PARBITER_INSTANCE) {
    PAGED_CODE!();

    if !(*Arbiter).PossibleAllocation.is_null() {
        RtlFreeRangeList((*Arbiter).PossibleAllocation);
        ExFreePoolWithTag((*Arbiter).PossibleAllocation as PVOID, TAG_ARBITER);
        (*Arbiter).PossibleAllocation = core::ptr::null_mut();
    }

    if !(*Arbiter).Allocation.is_null() {
        RtlFreeRangeList((*Arbiter).Allocation);
        ExFreePoolWithTag((*Arbiter).Allocation as PVOID, TAG_ARBITER);
        (*Arbiter).Allocation = core::ptr::null_mut();
    }

    if !(*Arbiter).AllocationStack.is_null() {
        ExFreePoolWithTag((*Arbiter).AllocationStack as PVOID, TAG_ARBITER);
        (*Arbiter).AllocationStack = core::ptr::null_mut();
        (*Arbiter).AllocationStackMaxSize = 0;
    }

    ArbiterLibFreeOrderingList(core::ptr::addr_of_mut!((*Arbiter).OrderingList));
    ArbiterLibFreeOrderingList(core::ptr::addr_of_mut!((*Arbiter).ReservedList));

    if !(*Arbiter).MutexEvent.is_null() {
        ExFreePoolWithTag((*Arbiter).MutexEvent as PVOID, TAG_ARBITER);
        (*Arbiter).MutexEvent = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "system" fn ArbiterLibInitializeInstance(
    Arbiter: PARBITER_INSTANCE,
    BusDeviceObject: PDEVICE_OBJECT,
    ResourceType: CM_RESOURCE_TYPE,
    ArbiterName: PCWSTR,
    OrderName: PCWSTR,
    TranslateOrderingFunction: PARB_TRANSLATE_ORDERING,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    PAGED_CODE!();

    ntkrnl_sys::ASSERT!((*Arbiter).UnpackRequirement.is_some());
    ntkrnl_sys::ASSERT!((*Arbiter).PackResource.is_some());
    ntkrnl_sys::ASSERT!((*Arbiter).UnpackResource.is_some());
    ntkrnl_sys::ASSERT!((*Arbiter).MutexEvent.is_null());
    ntkrnl_sys::ASSERT!((*Arbiter).Allocation.is_null());
    ntkrnl_sys::ASSERT!((*Arbiter).PossibleAllocation.is_null());
    ntkrnl_sys::ASSERT!((*Arbiter).AllocationStack.is_null());

    (*Arbiter).Signature = ARBITER_SIG;
    (*Arbiter).BusDeviceObject = BusDeviceObject;
    (*Arbiter).Name = ArbiterName;
    (*Arbiter).ResourceType = ResourceType;
    (*Arbiter).TransactionInProgress = FALSE;

    'failure: {
        (*Arbiter).MutexEvent = ExAllocatePoolWithTag(
            NonPagedPool,
            ::core::mem::size_of::<KEVENT>() as SIZE_T,
            TAG_ARBITER,
        ) as PKEVENT;
        if (*Arbiter).MutexEvent.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'failure;
        }
        KeInitializeEvent(
            (*Arbiter).MutexEvent as PRKEVENT,
            _EVENT_TYPE_SynchronizationEvent,
            TRUE,
        );

        (*Arbiter).AllocationStack = ExAllocatePoolWithTag(
            PagedPool,
            (PAGE_SIZE) as _,
            TAG_ARBITER,
        ) as PARBITER_ALLOCATION_STATE;
        if (*Arbiter).AllocationStack.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'failure;
        }
        (*Arbiter).AllocationStackMaxSize = (PAGE_SIZE) as _;

        (*Arbiter).Allocation = ExAllocatePoolWithTag(
            PagedPool,
            ::core::mem::size_of::<RTL_RANGE_LIST>() as SIZE_T,
            TAG_ARBITER,
        ) as PRTL_RANGE_LIST;
        if (*Arbiter).Allocation.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'failure;
        }
        RtlInitializeRangeList((*Arbiter).Allocation);

        (*Arbiter).PossibleAllocation = ExAllocatePoolWithTag(
            PagedPool,
            ::core::mem::size_of::<RTL_RANGE_LIST>() as SIZE_T,
            TAG_ARBITER,
        ) as PRTL_RANGE_LIST;
        if (*Arbiter).PossibleAllocation.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'failure;
        }
        RtlInitializeRangeList((*Arbiter).PossibleAllocation);

        if (*Arbiter).TestAllocation.is_none() {
            (*Arbiter).TestAllocation = Some(ArbiterLibTestAllocation);
        }
        if (*Arbiter).RetestAllocation.is_none() {
            (*Arbiter).RetestAllocation = Some(ArbiterLibRetestAllocation);
        }
        if (*Arbiter).CommitAllocation.is_none() {
            (*Arbiter).CommitAllocation = Some(ArbiterLibCommitAllocation);
        }
        if (*Arbiter).RollbackAllocation.is_none() {
            (*Arbiter).RollbackAllocation = Some(ArbiterLibRollbackAllocation);
        }
        if (*Arbiter).BootAllocation.is_none() {
            (*Arbiter).BootAllocation = Some(ArbiterLibBootAllocation);
        }
        if (*Arbiter).AddReserved.is_none() {
            (*Arbiter).AddReserved = Some(ArbiterLibAddReserved);
        }
        if (*Arbiter).QueryConflict.is_none() {
            (*Arbiter).QueryConflict = Some(ArbiterLibQueryConflict);
        }
        if (*Arbiter).QueryArbitrate.is_none() {
            (*Arbiter).QueryArbitrate = Some(ArbiterLibQueryArbitrate);
        }
        if (*Arbiter).StartArbiter.is_none() {
            (*Arbiter).StartArbiter = Some(ArbiterLibStartArbiter);
        }
        if (*Arbiter).PreprocessEntry.is_none() {
            (*Arbiter).PreprocessEntry = Some(ArbiterLibPreprocessEntry);
        }
        if (*Arbiter).AllocateEntry.is_none() {
            (*Arbiter).AllocateEntry = Some(ArbiterLibAllocateEntry);
        }
        if (*Arbiter).GetNextAllocationRange.is_none() {
            (*Arbiter).GetNextAllocationRange = Some(ArbiterLibGetNextAllocationRange);
        }
        if (*Arbiter).FindSuitableRange.is_none() {
            (*Arbiter).FindSuitableRange = Some(ArbiterLibFindSuitableRange);
        }
        if (*Arbiter).AddAllocation.is_none() {
            (*Arbiter).AddAllocation = Some(ArbiterLibAddAllocation);
        }
        if (*Arbiter).BacktrackAllocation.is_none() {
            (*Arbiter).BacktrackAllocation = Some(ArbiterLibBacktrackAllocation);
        }
        if (*Arbiter).OverrideConflict.is_none() {
            (*Arbiter).OverrideConflict = Some(ArbiterLibOverrideConflict);
        }

        Status = ArbiterLibDefaultAssignmentOrdering(Arbiter, OrderName, OrderName, TranslateOrderingFunction);
        if !NT_SUCCESS(Status) {
            DPRINT1!(
                "ArbiterLibInitializeInstance: ArbiterLibDefaultAssignmentOrdering failed, Status %X\n",
                Status,
            );
            break 'failure;
        }

        return STATUS_SUCCESS;
    }

    DPRINT1!("ArbiterLibInitializeInstance: '%S' failed, Status %X\n", ArbiterName, Status);
    ArbiterLibDeleteInstance(Arbiter);
    return Status;
}
