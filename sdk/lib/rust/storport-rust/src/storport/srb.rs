use crate::prelude::*;
use ntkrnl_sys::ddk::{KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeReleaseInStackQueuedSpinLock, MmProbeAndLockPages, MmUnlockPages, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PSCATTER_GATHER_LIST};
use core::ffi::c_void;

unsafe extern "C" fn StorpSrbInitRequestReference(
    RequestReference: PQUEUED_REQUEST_REFERENCE,
    Srb: PSCSI_REQUEST_BLOCK,
    Irp: PIRP,
    PdoExtension: PPDO_DEVICE_EXTENSION,
) {
    /* Both the lookaside entries and the reserve are recycled, so they need zeroing */
    core::ptr::write_bytes(RequestReference as *mut u8, 0, core::mem::size_of::<QUEUED_REQUEST_REFERENCE>());

    (*Srb).OriginalRequest = RequestReference as PVOID;
    (*RequestReference).Irp = Irp;
    (*RequestReference).Srb = Srb;
    (*RequestReference).TimeoutCounter = (*Srb).TimeOutValue;

    (*RequestReference).PdoExtension = PdoExtension;
}

unsafe extern "C" fn StorpSrbAllocateRequestReference(
    Srb: PSCSI_REQUEST_BLOCK,
    Irp: PIRP,
    PdoExtension: PPDO_DEVICE_EXTENSION,
) -> PQUEUED_REQUEST_REFERENCE {
    let mut RequestReference: PQUEUED_REQUEST_REFERENCE;

    RequestReference = ExAllocateFromNPagedLookasideList(
        core::ptr::addr_of_mut!((*(*PdoExtension).FdoExtension).RequestReferenceLookaside),
    ) as PQUEUED_REQUEST_REFERENCE;
    if RequestReference.is_null() {
        return core::ptr::null_mut();
    }

    StorpSrbInitRequestReference(RequestReference, Srb, Irp, PdoExtension);

    return RequestReference;
}

unsafe extern "C" fn StorpSrbAcquireReservedRequestReference(
    Srb: PSCSI_REQUEST_BLOCK,
    Irp: PIRP,
    PdoExtension: PPDO_DEVICE_EXTENSION,
) -> PQUEUED_REQUEST_REFERENCE {
    let FdoExtension: PFDO_DEVICE_EXTENSION = (*PdoExtension).FdoExtension;
    let mut RequestReference: PQUEUED_REQUEST_REFERENCE = core::ptr::null_mut();

    if (*FdoExtension).ReservedRequestReference.is_null()
        || InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*FdoExtension).ReserveInUse) as *mut core::ffi::c_long,
            1,
            0,
        ) != 0
    {
        return core::ptr::null_mut();
    }

    RequestReference = (*FdoExtension).ReservedRequestReference as PQUEUED_REQUEST_REFERENCE;
    StorpSrbInitRequestReference(RequestReference, Srb, Irp, PdoExtension);
    (*RequestReference).FromReserve = TRUE;

    return RequestReference;
}
#[no_mangle]
pub unsafe extern "C" fn StorpSrbFreeRequestReference(mut Srb: PSCSI_REQUEST_BLOCK) {
    let mut RequestReference: PQUEUED_REQUEST_REFERENCE =
        core::ptr::null_mut::<_QUEUED_REQUEST_REFERENCE>();
    let mut FdoExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut::<_FDO_DEVICE_EXTENSION>();
    RequestReference = (*Srb).OriginalRequest as PQUEUED_REQUEST_REFERENCE;
    (*Srb).OriginalRequest = (*RequestReference).Irp as PVOID;
    FdoExtension = (*(*RequestReference).PdoExtension).FdoExtension;
    if (*RequestReference).FromReserve != 0 {
        InterlockedExchange(
            core::ptr::addr_of_mut!((*FdoExtension).ReserveInUse) as *mut ::core::ffi::c_long,
            0 as ::core::ffi::c_long,
        );
    } else {
        ExFreeToNPagedLookasideList(
            core::ptr::addr_of_mut!((*FdoExtension).RequestReferenceLookaside),
            RequestReference as PVOID,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn StorpAssignTagToSrb(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
) -> BOOLEAN {
    let mut NewTag: ULONG;

    /* Bail out if the request is untagged */
    if TEST_FLAG!((*Srb).SrbFlags, SRB_FLAGS_QUEUE_ACTION_ENABLE) == false {
        return FALSE;
    }

    /* FIXME: What if the counter overflows? */
    NewTag = InterlockedIncrement(
        core::ptr::addr_of_mut!((*PdoExtension).TagCounter) as *mut core::ffi::c_long,
    ) as ULONG;

    /*
     * We use ULONG for tag counter, and here seems to do a stupid truncation by casting to UCHAR;
     * But since Storport has a default maximum 254 requests/LUN limitation, this is actually OK for
     * our use case. If a miniport needs to support more than 254 tags it must use
     * STORAGE_REQUEST_BLOCK which we don't support yet :D
     */
    (*Srb).QueueTag = (NewTag % (*PdoExtension).OutstandingRequestMax) as UCHAR;

    DPRINT!("StorpAssignTagToSrb: Assigned Tag %x, TagCounter was %x\n", (*Srb).QueueTag, NewTag);

    return TRUE;
}
#[no_mangle]
pub unsafe extern "system" fn PortPdoAfterBuildingScatterGatherList(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    ScatterGatherList: PSCATTER_GATHER_LIST,
    Context: PVOID,
) {
    let mut FdoExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut MiniportExtension: PVOID = core::ptr::null_mut();
    let mut HwInitData: PHW_INITIALIZATION_DATA = core::ptr::null_mut();
    let mut RequestReference: PQUEUED_REQUEST_REFERENCE = core::ptr::null_mut();
    let mut CanStartIo: BOOLEAN = TRUE;
    let mut OldIrql: KIRQL = 0;

    RequestReference = Context as PQUEUED_REQUEST_REFERENCE;
    FdoExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    MiniportExtension = core::ptr::addr_of_mut!(
        (*(*FdoExtension).Miniport.MiniportExtension).HwDeviceExtension
    ) as PVOID;
    HwInitData = (*FdoExtension).HwInitData;

    /* Save our SG List */
    (*RequestReference).ScatterGatherList = ScatterGatherList as PSTOR_SCATTER_GATHER_LIST;

    /* BuildIo and StartIo are called on DISPATCH_LEVEL */
    OldIrql = KfRaiseIrql((DISPATCH_LEVEL) as KIRQL);

    loop {
        /* Call BuildIo, if one is provided */
        if (*HwInitData).HwBuildIo.is_some() {
            CanStartIo = (*HwInitData).HwBuildIo.unwrap_unchecked()(
                MiniportExtension,
                (*RequestReference).Srb,
            );
        }

        /* If BuildIo says the request processing shall stop here, no more processing is needed */
        if CanStartIo == 0 {
            break;
        }

        (*HwInitData).HwStartIo.unwrap_unchecked()(
            MiniportExtension,
            (*RequestReference).Srb,
        );

        if (0) == 0 {
            break;
        }
    }

    /* Restore IRQL */
    KeLowerIrql(OldIrql);
}

unsafe extern "C" fn PortPdoStarveRequest(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Srb: PSCSI_REQUEST_BLOCK,
    Irp: PIRP,
) -> NTSTATUS {
    /* Without a reserve nothing guarantees a completion that would retry the request */
    if (*FdoExtension).ReservedRequestReference.is_null() {
        (*Srb).__bindgen_anon_1.InternalStatus = (STATUS_INSUFFICIENT_RESOURCES) as u32;
        StorpCompleteRequest(Irp, (SRB_STATUS_INTERNAL_ERROR) as UCHAR, STATUS_INSUFFICIENT_RESOURCES);
    } else {
        PortQueueStarvedRequest(FdoExtension, Irp);
    }

    return STATUS_PENDING;
}
#[no_mangle]
pub unsafe extern "C" fn PortPdoSrbAllocatePrivateContexts(
    Srb: PSCSI_REQUEST_BLOCK,
    PdoExtension: PPDO_DEVICE_EXTENSION,
    FdoExtension: PFDO_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut RequestReference: PQUEUED_REQUEST_REFERENCE = core::ptr::null_mut();
    let mut SrbExtensionSize: ULONG = (*(*FdoExtension).HwInitData).SrbExtensionSize;
    let mut Irp: PIRP = core::ptr::null_mut();

    Irp = (*Srb).OriginalRequest as PIRP;

    /* Allocate our private data area, falling back on the reserve when pool is exhausted */
    RequestReference = StorpSrbAllocateRequestReference(Srb, Irp, PdoExtension);
    if RequestReference.is_null() {
        RequestReference = StorpSrbAcquireReservedRequestReference(Srb, Irp, PdoExtension);
        if RequestReference.is_null() {
            return PortPdoStarveRequest(FdoExtension, Srb, Irp);
        }
    }

    /* Allocate SRB extension */
    if SrbExtensionSize == 0 {
        (*Srb).SrbExtension = core::ptr::null_mut();
        return STATUS_SUCCESS;
    }

    if (*RequestReference).FromReserve != 0 {
        (*Srb).SrbExtension = (*FdoExtension).ReservedSrbExtension;
        RtlZeroMemory!((*Srb).SrbExtension, SrbExtensionSize as SIZE_T);
        return STATUS_SUCCESS;
    }

    (*Srb).SrbExtension = ExAllocatePoolWithTag(
        NonPagedPool,
        SrbExtensionSize as SIZE_T,
        TAG_SRB_EXTENSION,
    );
    if (*Srb).SrbExtension.is_null() {
        StorpSrbFreeRequestReference(Srb);
        return PortPdoStarveRequest(FdoExtension, Srb, Irp);
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PortPdoIssueRequest(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    RequestReference: PQUEUED_REQUEST_REFERENCE,
) -> NTSTATUS {
    let mut Irp: PIRP;
    let mut Status: NTSTATUS;
    let mut FdoExtension: PFDO_DEVICE_EXTENSION;
    let mut HwInitData: PHW_INITIALIZATION_DATA;
    let mut OldIrql: KIRQL;

    Irp = (*RequestReference).Irp;
    FdoExtension = (*PdoExtension).FdoExtension;
    HwInitData = (*FdoExtension).Miniport.InitData;

    NT_ASSERT!(!HwInitData.is_null());

    /* Fill in SCSI address data */
    /* FIXME: Not sure if this is the best place for it. */
    (*(*RequestReference).Srb).PathId = ((*PdoExtension).Bus) as u8;
    (*(*RequestReference).Srb).TargetId = ((*PdoExtension).Target) as u8;
    (*(*RequestReference).Srb).Lun = ((*PdoExtension).Lun) as u8;

    /* Start an DMA if IO subsystem allocated an MDL for us */
    /* FIXME: Does this strategy work? */
    if !(*(*RequestReference).Irp).MdlAddress.is_null() {
        /*
         * Lock the buffer so that it stays in physical memory.
         * In the two cases avoided here the memory has already been locked
         */
        if !TEST_FLAG!((*(*Irp).MdlAddress).MdlFlags, (MDL_SOURCE_IS_NONPAGED_POOL as i16))
            && !TEST_FLAG!((*(*Irp).MdlAddress).MdlFlags, (MDL_MAPPED_TO_SYSTEM_VA as i16))
            && !TEST_FLAG!((*(*Irp).MdlAddress).MdlFlags, (MDL_PAGES_LOCKED as i16))
            && !TEST_FLAG!((*(*Irp).MdlAddress).MdlFlags, (MDL_PARTIAL as i16))
        {
            (*RequestReference).MappedSystemVa = 1 as PVOID; /* FIXME: CHANGE ITS NAME */
            // Divergence: the C calls the raising MmProbeAndLockPages with no SEH frame, so a
            // failure there is a bugcheck. Fail this one request through the normal
            // completion path instead (nothing was locked, so nothing is unlocked).
            let LockStatus = ntkrnl_sys::probe::MmTryProbeAndLockPages(
                (*Irp).MdlAddress.cast(),
                (0i32) as KPROCESSOR_MODE,
                IoModifyAccess as _,
            );
            if !NT_SUCCESS(LockStatus) {
                (*RequestReference).MappedSystemVa = core::ptr::null_mut();
                (*(*RequestReference).Srb).SrbStatus = SRB_STATUS_ERROR as UCHAR;
                PortCompleteRequest(RequestReference);
                return LockStatus;
            }
        }

        /* Following call requires DISPATCH_LEVEL. */
        OldIrql = KfRaiseIrql(DISPATCH_LEVEL as KIRQL);

        /* Make up the scatter gather list */
        /* FIXME: I don't know if we actually hit it */
        NT_ASSERT!(
            ((*(*RequestReference).Srb).SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION)
                != SRB_FLAGS_UNSPECIFIED_DIRECTION,
        );
        (*RequestReference).WriteToDevice = TEST_FLAG!(
            (*(*RequestReference).Srb).SrbFlags,
            SRB_FLAGS_DATA_OUT,
        ) as BOOLEAN;

        (*(*(*FdoExtension).DmaAdapter).DmaOperations)
            .GetScatterGatherList
            .unwrap_unchecked()(
            (*FdoExtension).DmaAdapter,
            (*FdoExtension).Device,
            (*(*RequestReference).Irp).MdlAddress,
            MmGetMdlVirtualAddress!((*(*RequestReference).Irp).MdlAddress),
            (*(*RequestReference).Srb).DataTransferLength,
            Some(
                PortPdoAfterBuildingScatterGatherList
                    as unsafe extern "system" fn(
                        PDEVICE_OBJECT,
                        PIRP,
                        PSCATTER_GATHER_LIST,
                        PVOID,
                    ),
            ),
            RequestReference as PVOID,
            (*RequestReference).WriteToDevice,
        );

        /* Restore IRQL */
        KeLowerIrql(OldIrql);
    } else {
        /* Otherwise just start the IO with a NULL scatter gather list */
        PortPdoAfterBuildingScatterGatherList(
            (*FdoExtension).Device,
            (*RequestReference).Irp,
            core::ptr::null_mut(),
            RequestReference as PVOID,
        );
    }

    /* FIXME: Is this correct? */
    Status = STATUS_PENDING;

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn PortPdoScheduleRequestNoFlowControl(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    RequestReference: PQUEUED_REQUEST_REFERENCE,
) {
    /* Ensure outstanding flag is unset */
    (*RequestReference).IsOutstanding = FALSE as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn PortPdoScheduleRequestFlowControlled(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    FdoExtension: PFDO_DEVICE_EXTENSION,
    RequestReference: PQUEUED_REQUEST_REFERENCE,
) -> BOOLEAN {
    let mut FlowControl: PPDO_IO_FLOW_CONTROL = core::ptr::addr_of_mut!((*PdoExtension).FlowControl);
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut Srb: PSCSI_REQUEST_BLOCK = (*RequestReference).Srb;
    let mut StrongOrderedRequest: BOOLEAN = FALSE;
    let mut IssueRequest: BOOLEAN = TRUE;

    /* Set outstanding flag */
    (*RequestReference).IsOutstanding = TRUE;

    /* Check if we can issue this request. */
    /* Acquire the HBA global flow control lock first */
    KeAcquireInStackQueuedSpinLock(
        core::ptr::addr_of_mut!((*FdoExtension).FlowControl.Lock),
        core::ptr::addr_of_mut!(LockHandle),
    );
    StorpCheckFlowControl(FdoExtension);

    loop {
        /* Check PDO flow control conditions */

        /* Sanity check */
        NT_ASSERT!((*FlowControl).OutstandingRequestCount <= (*PdoExtension).OutstandingRequestMax);

        /* If PDO is not suitable to receive a request, make it wait */
        if (*FlowControl).IsBusy != 0
            || (*FlowControl).IsPaused != 0
            || ((*FlowControl).IsFrozen != 0
                && TEST_FLAG!((*Srb).SrbFlags, SRB_FLAGS_BYPASS_FROZEN_QUEUE) == false)
            || ((*FlowControl).IsLocked != 0
                && TEST_FLAG!((*Srb).SrbFlags, SRB_FLAGS_BYPASS_LOCKED_QUEUE) == false)
            || (*FlowControl).StrongOrderedCount != 0
            || (*FlowControl).OutstandingRequestCount == (*PdoExtension).OutstandingRequestMax
            || IsListEmpty(core::ptr::addr_of_mut!((*FlowControl).AwaitingRequestsListHead)) == 0
        {
            InsertHeadList(
                core::ptr::addr_of_mut!((*FlowControl).AwaitingRequestsListHead),
                core::ptr::addr_of_mut!((*RequestReference).PdoEntry),
            );
            IssueRequest = FALSE;
            break;
        }

        /* Check if this request is strong-ordered */
        if TEST_FLAG!((*Srb).SrbFlags, SRB_FLAGS_QUEUE_ACTION_ENABLE) == false
            || ((*Srb).QueueAction == (SRB_ORDERED_QUEUE_TAG_REQUEST as u8))
        {
            (*RequestReference).StrongOrdered = TRUE;
            StrongOrderedRequest = TRUE;
            (*FlowControl).StrongOrderedCount = (*FlowControl).StrongOrderedCount.wrapping_add(1);
        }

        /*
        * If this request is strong ordered and the LUN's active/approved list is not
        * drained, the request must wait.
        */
        if StrongOrderedRequest != 0
            && IsListEmpty(core::ptr::addr_of_mut!((*FlowControl).ActiveRequestsListHead)) == 0
        {
            InsertHeadList(
                core::ptr::addr_of_mut!((*FlowControl).AwaitingRequestsListHead),
                core::ptr::addr_of_mut!((*RequestReference).PdoEntry),
            );
            IssueRequest = FALSE;
            break;
        }

        /*
        * If any break's triggered before this point, the request is in PDO waiting queue.
        * Going past this point meant PDO approves this request. Put it into active list.
        */
        if (*Srb).QueueAction == (SRB_HEAD_OF_QUEUE_TAG_REQUEST as u8) {
            InsertTailList(
                core::ptr::addr_of_mut!((*FlowControl).ActiveRequestsListHead),
                core::ptr::addr_of_mut!((*RequestReference).PdoEntry),
            );
        } else {
            InsertHeadList(
                core::ptr::addr_of_mut!((*FlowControl).ActiveRequestsListHead),
                core::ptr::addr_of_mut!((*RequestReference).PdoEntry),
            );
        }

        /* A new outstanding request is added to this LUN */
        (*FlowControl).OutstandingRequestCount = (*FlowControl).OutstandingRequestCount.wrapping_add(1);

        /* Check FDO flow control status. */

        /* If the FDO can't handle more requests, then the request should be queued. */
        if (*FlowControl).IsBusy != 0
            || (*FlowControl).IsPaused != 0
            || (*FlowControl).OutstandingRequestCount == (*FdoExtension).OutstandingRequestMax
        {
            /* Put into FDO queue */
            InsertHeadList(
                core::ptr::addr_of_mut!((*FdoExtension).FlowControl.FdoBlockedRequestsListHead),
                core::ptr::addr_of_mut!((*RequestReference).FdoEntry),
            );

            /*
            * Now the request is in PDO active list, in FDO awaiting queue, not issued.
            * Let it wait until another completion event schedules it.
            */
            break;
        }

        /* Seems good, increment FDO request count */
        (*FdoExtension).FlowControl.OutstandingRequestCount =
            (*FdoExtension).FlowControl.OutstandingRequestCount.wrapping_add(1);

        /*
        * Now the request is counted in FDO & PDO, and is put into PDO active list,
        * next step we're going to issue it.
        */
        if (0) == 0 {
            break;
        }
    }

    // FIXME: The port driver indicates that an LU-specific queue has been frozen by
    // returning a request with SRB_STATUS_QUEUE_FROZEN in the SrbStatus member. New
    // requests from the class driver can be inserted into the queue, but only autosense
    // requests are sent to the logical unit as long as its queue is frozen.
    // https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/storage-class-driver-s-releasequeue-routine

    /* We've determined where the request goes. Release the flow control lock */
    StorpCheckFlowControl(FdoExtension);
    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));

    return IssueRequest;
}

#[no_mangle]
pub unsafe extern "system" fn PortPdoScsi(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Stack: PIO_STACK_LOCATION;
    let mut Srb: PSCSI_REQUEST_BLOCK;
    let mut PdoExtension: PPDO_DEVICE_EXTENSION;
    let mut FdoExtension: PFDO_DEVICE_EXTENSION;
    let mut Miniport: PMINIPORT;
    let mut HwInitData: PHW_INITIALIZATION_DATA;

    DPRINT!("PortPdoScsi(%p %p)\n", DeviceObject, Irp);

    Stack = IoGetCurrentIrpStackLocation(Irp);
    Srb = ((*Stack).Parameters.Scsi.Srb).cast();

    if Srb.is_null() {
        DPRINT1!("Srb is NULL!");
        DbgBreakPoint();
        Status = STATUS_UNSUCCESSFUL;

        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;

        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
        return Status;
    }

    PdoExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    NT_ASSERT!(!PdoExtension.is_null());
    FdoExtension = (*PdoExtension).FdoExtension;
    NT_ASSERT!(!FdoExtension.is_null());
    Miniport = core::ptr::addr_of_mut!((*FdoExtension).Miniport);
    HwInitData = (*Miniport).InitData;

    'sw: {
        match ((*Srb).Function as ::core::ffi::c_int) as u32 {
            /* Both of these should be sent to device and let themselves process this */
            SRB_FUNCTION_IO_CONTROL | SRB_FUNCTION_EXECUTE_SCSI => {
                let mut RequestReference: PQUEUED_REQUEST_REFERENCE;
                let mut IssueRequest: BOOLEAN;

                /* Mark IRP as pending */
                IoMarkIrpPending(Irp);

                /* Allocate private contexts */
                Status = PortPdoSrbAllocatePrivateContexts(Srb, PdoExtension, FdoExtension);
                if Status != STATUS_SUCCESS {
                    break 'sw;
                }
                RequestReference = (*Srb).OriginalRequest as PQUEUED_REQUEST_REFERENCE;

                /* Assign a tag to the SRB if it supports tagging */
                StorpAssignTagToSrb(PdoExtension, Srb);

                DPRINT!(
                    "New request (%d:%d:%d) ReqRef %p Srb %p Irp %p Tag %x SrbExt %p(%d)\n",
                    (*Srb).PathId,
                    (*Srb).TargetId,
                    (*Srb).Lun,
                    RequestReference,
                    Srb,
                    Irp,
                    (*Srb).QueueTag,
                    (*Srb).SrbExtension,
                    (*HwInitData).SrbExtensionSize
                );

                /*
                 * Do the scheduling. If returns TRUE, we should issue this request to the hardware.
                 * Otherwise it will keep waiting until next completion schedules it, or be cancelled on
                 * an error, etc.
                 */
                IssueRequest = PortPdoScheduleRequestFlowControlled(PdoExtension, FdoExtension, RequestReference);

                /* This request is viable to be issued. */
                if IssueRequest != 0 {
                    Status = PortPdoIssueRequest(PdoExtension, RequestReference);
                } else {
                    Status = STATUS_PENDING;
                }
            }

            /*
             * These requests are actually sent to adapters and are not outstanding requests for devices
             * so we simply pass them down to miniport
             */
            SRB_FUNCTION_FLUSH | SRB_FUNCTION_SHUTDOWN => {
                let mut RequestReference: PQUEUED_REQUEST_REFERENCE;

                /* Mark IRP as pending */
                IoMarkIrpPending(Irp);

                /* Allocate private contexts */
                Status = PortPdoSrbAllocatePrivateContexts(Srb, PdoExtension, FdoExtension);
                if Status != STATUS_SUCCESS {
                    break 'sw;
                }
                RequestReference = (*Srb).OriginalRequest as PQUEUED_REQUEST_REFERENCE;

                /*
                 * Call the non-flow-controlled scheduling function (it does nothing but the model is
                 * preserved for consistency)
                 */
                PortPdoScheduleRequestNoFlowControl(FdoExtension, RequestReference);

                /* Fire up the request immediately */
                PortPdoIssueRequest(PdoExtension, RequestReference);
            }

            SRB_FUNCTION_CLAIM_DEVICE => {
                if InterlockedCompareExchange(
                    core::ptr::addr_of_mut!((*PdoExtension).IsClaimed),
                    1,
                    0,
                ) != 0
                {
                    Status = STATUS_DEVICE_BUSY;
                    StorpCompleteRequest(Irp, (SRB_STATUS_BUSY) as UCHAR, Status);
                    break 'sw;
                }

                /* The claimed device object is handed back to the class driver here */
                (*Srb).DataBuffer = (*PdoExtension).Device as PVOID;
                Status = STATUS_SUCCESS;
                StorpCompleteRequest(Irp, (SRB_STATUS_SUCCESS) as UCHAR, Status);
            }

            SRB_FUNCTION_RELEASE_DEVICE => {
                InterlockedExchange(
                    core::ptr::addr_of_mut!((*PdoExtension).IsClaimed),
                    0,
                );

                Status = STATUS_SUCCESS;
                StorpCompleteRequest(Irp, (SRB_STATUS_SUCCESS) as UCHAR, Status);
            }

            _ => {
                DPRINT1!("Unsupported Srb Function %02X!", (*Srb).Function as core::ffi::c_uint);
                // DbgBreakPoint();
                Status = STATUS_NOT_IMPLEMENTED;
                StorpCompleteRequest(Irp, (SRB_STATUS_INVALID_REQUEST) as UCHAR, Status);
            }
        }
    }

    return Status;
}

unsafe extern "C" fn PortScheduleComingRequestsOnCompletion(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    FdoExtension: PFDO_DEVICE_EXTENSION,
) {
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut RequestReference: PQUEUED_REQUEST_REFERENCE = core::ptr::null_mut();

    /* Sanity check again whether FDO has resource to handle the request */
    NT_ASSERT!(
        (*FdoExtension).FlowControl.OutstandingRequestCount < (*FdoExtension).OutstandingRequestMax
    );

    /* If FDO has waiting requests, issue one. */
    if IsListEmpty(core::ptr::addr_of_mut!(
        (*FdoExtension).FlowControl.FdoBlockedRequestsListHead
    )) == 0
    {
        /* Sanity check */
        NT_ASSERT!(
            (*FdoExtension).FlowControl.OutstandingRequestCount < (*FdoExtension).OutstandingRequestMax
        );

        /* Take it out of FDO list */
        Entry = RemoveTailList(core::ptr::addr_of_mut!(
            (*FdoExtension).FlowControl.FdoBlockedRequestsListHead
        ));
        RequestReference = ((Entry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(QUEUED_REQUEST_REFERENCE, FdoEntry) as ULONG_PTR,
        )) as PQUEUED_REQUEST_REFERENCE;

        /* Increment FDO request counter */
        (*FdoExtension).FlowControl.OutstandingRequestCount =
            (*FdoExtension).FlowControl.OutstandingRequestCount.wrapping_add(1);
        NT_ASSERT!(
            (*FdoExtension).FlowControl.OutstandingRequestCount <= (*FdoExtension).OutstandingRequestMax
        );

        /* Call the start routine */
        PortPdoIssueRequest(PdoExtension, RequestReference);

        /* Our task is done */
        return;
    }

    /* FDO's got no request, check the PDO of the just-finished request for awaiting requests */
    if IsListEmpty(core::ptr::addr_of_mut!(
        (*PdoExtension).FlowControl.AwaitingRequestsListHead
    )) == 0
    {
        /* Sanity check */
        NT_ASSERT!(
            (*PdoExtension).FlowControl.OutstandingRequestCount < (*PdoExtension).OutstandingRequestMax
        );

        /* Take one request */
        Entry = RemoveTailList(core::ptr::addr_of_mut!(
            (*PdoExtension).FlowControl.AwaitingRequestsListHead
        ));
        RequestReference = ((Entry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(QUEUED_REQUEST_REFERENCE, PdoEntry) as ULONG_PTR,
        )) as PQUEUED_REQUEST_REFERENCE;

        /* This one goes to active list */
        InsertHeadList(
            core::ptr::addr_of_mut!((*PdoExtension).FlowControl.ActiveRequestsListHead),
            Entry,
        );

        /* Increment PDO request counter */
        (*PdoExtension).FlowControl.OutstandingRequestCount =
            (*PdoExtension).FlowControl.OutstandingRequestCount.wrapping_add(1);
        NT_ASSERT!(
            (*PdoExtension).FlowControl.OutstandingRequestCount <= (*PdoExtension).OutstandingRequestMax
        );

        /* FIXME: should be false. Allocate resource if it is determined required */
        if (*(*RequestReference).Srb).SrbExtension.is_null()
        {
            __debugbreak();
        }

        /* Call the start routine */
        PortPdoIssueRequest(PdoExtension, RequestReference);

        /* Our task is done */
        return;
    }
}

unsafe extern "C" fn PortCompleteRequest(mut RequestReference: PQUEUED_REQUEST_REFERENCE) {
    let mut Srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut FdoExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut PdoExtension: PPDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut IoStatus: NTSTATUS = 0;
    let mut FromReserve: BOOLEAN = 0;

    /* Take out our information */
    Srb = (*RequestReference).Srb;
    Irp = (*RequestReference).Irp;
    PdoExtension = (*RequestReference).PdoExtension;
    FdoExtension = (*PdoExtension).FdoExtension;

    /* The reference is released further down, so remember this while it is still valid */
    FromReserve = (*RequestReference).FromReserve;

    DPRINT!("Completing request: (%d:%d:%d) ReqRef %p Srb %p Irp %p Tag %x\n",
            (*Srb).PathId, (*Srb).TargetId, (*Srb).Lun, RequestReference, Srb, (*RequestReference).Irp, (*Srb).QueueTag);

    /* Return SG list resources back to system */
    if !(*RequestReference).ScatterGatherList.is_null() {
        (*(*(*FdoExtension).DmaAdapter).DmaOperations)
            .PutScatterGatherList
            .unwrap_unchecked()(
            (*FdoExtension).DmaAdapter,
            (*RequestReference).ScatterGatherList as PSCATTER_GATHER_LIST,
            (*RequestReference).WriteToDevice,
        );
        (*RequestReference).ScatterGatherList = core::ptr::null_mut();

        /* Unlock the pages locked for DMA */
        if !(*RequestReference).MappedSystemVa.is_null() {
            MmUnlockPages((*(*RequestReference).Irp).MdlAddress);
        }
    }

    /* Check if we need to interact with flow control. */
    if (*RequestReference).IsOutstanding != 0 {
        /* Take the request out of FDO and PDO structures, and update flow control data */
        KeAcquireInStackQueuedSpinLock(
            core::ptr::addr_of_mut!((*FdoExtension).FlowControl.Lock),
            core::ptr::addr_of_mut!(LockHandle),
        );
        StorpCheckFlowControl(FdoExtension);

        (*FdoExtension).FlowControl.OutstandingRequestCount =
            (*FdoExtension).FlowControl.OutstandingRequestCount.wrapping_sub(1);
        if (*FdoExtension).FlowControl.RemainingBusyRequests != 0 {
            (*FdoExtension).FlowControl.RemainingBusyRequests =
                (*FdoExtension).FlowControl.RemainingBusyRequests.wrapping_sub(1);

            /* Clear busy state as needed */
            if (*FdoExtension).FlowControl.OutstandingRequestCount == 0
                || (*FdoExtension).FlowControl.RemainingBusyRequests == 0
            {
                (*FdoExtension).FlowControl.IsBusy = FALSE;
            }
        }

        RemoveEntryList(core::ptr::addr_of_mut!((*RequestReference).PdoEntry));
        (*PdoExtension).FlowControl.OutstandingRequestCount =
            (*PdoExtension).FlowControl.OutstandingRequestCount.wrapping_sub(1);
        if (*PdoExtension).FlowControl.RemainingBusyRequests != 0 {
            (*PdoExtension).FlowControl.RemainingBusyRequests =
                (*PdoExtension).FlowControl.RemainingBusyRequests.wrapping_sub(1);

            /* Clear busy state as needed */
            if (*PdoExtension).FlowControl.OutstandingRequestCount == 0
                || (*PdoExtension).FlowControl.RemainingBusyRequests == 0
            {
                (*PdoExtension).FlowControl.IsBusy = FALSE;
            }
        }

        /* Decrement strong ordered request count if needed */
        if (*RequestReference).StrongOrdered != 0 {
            (*PdoExtension).FlowControl.StrongOrderedCount =
                (*PdoExtension).FlowControl.StrongOrderedCount.wrapping_sub(1);
        }

        /* Free the private data structure. No one should use it anymore */
        StorpSrbFreeRequestReference(Srb);
        RequestReference = core::ptr::null_mut();

        /* Schedule next IO before we release flow control lock */
        PortScheduleComingRequestsOnCompletion(PdoExtension, FdoExtension);

        /* We're done, release flow control lock */
        StorpCheckFlowControl(FdoExtension);
        KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));
    }

    /*
     * Free SRB extension as this request will never go back to miniport again.
     */
    if !(*Srb).SrbExtension.is_null() {
        /* The reserved one is handed back along with its packet, not freed */
        if FromReserve == 0 {
            ExFreePoolWithTag((*Srb).SrbExtension, TAG_SRB_EXTENSION);
        }
        (*Srb).SrbExtension = core::ptr::null_mut();
    }

    /*
     * After done with internal accountings, we can finally complete this IO.
     */
    match (SRB_STATUS!((*Srb).SrbStatus)) as u32 {
        SRB_STATUS_SUCCESS => {
            IoStatus = STATUS_SUCCESS;
        }
        SRB_STATUS_BUSY => {
            IoStatus = STATUS_DEVICE_BUSY;
        }
        _ => {
            IoStatus = STATUS_UNSUCCESSFUL; // FIXME:
        }
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = IoStatus;
    (*Irp).IoStatus.Information = (*Srb).DataTransferLength as ULONG_PTR; /* FIXME: hopefully this is correct */

    /* We just gave memory back, so let one request that ran out of it through */
    PortRetryStarvedRequest(FdoExtension);

    DPRINT!("Calling IoCompleteRequest\n");
    IoCompleteRequest(Irp, (IO_DISK_INCREMENT as i8));
}
#[no_mangle]
pub unsafe extern "system" fn PortCompletionDpc(Dpc: PKDPC, Context: PVOID, SystemArgument1: PVOID, SystemArgument2: PVOID) {
    let mut RequestReference: PQUEUED_REQUEST_REFERENCE;
    let mut FdoExtension: PFDO_DEVICE_EXTENSION;
    let mut Entry: PSLIST_ENTRY;
    let mut Next: PSLIST_ENTRY;
    let mut Count: ULONG = 0;

    DPRINT!("PortCompletionDpc(%p %p %p %p)\n", Dpc, Context, SystemArgument1, SystemArgument2);

    /* Take context pointer as FDO extension */
    FdoExtension = Context as PFDO_DEVICE_EXTENSION;

    /* Flush all entries in the list and complete all requests */
    Entry = InterlockedFlushSList(core::ptr::addr_of_mut!((*FdoExtension).CompletionList));
    while !Entry.is_null() {
        /* Prepare Next pointer first (Completion routine will free the memory that contains it) */
        Next = (*Entry).Next;

        /* Get original request */
        RequestReference = CONTAINING_RECORD!(Entry, QUEUED_REQUEST_REFERENCE, CompletionEntry);

        /* Call real completion routine */
        PortCompleteRequest(RequestReference);

        /* Move on to next one */
        Entry = Next;
        Count = Count.wrapping_add(1);
    }

    DPRINT!("Completed %d requests\n", Count);
}
