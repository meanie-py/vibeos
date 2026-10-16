use crate::prelude::*;
use ntkrnl_sys::ddk::{BUS_INTERFACE_STANDARD, DEVICE_RELATIONS, ExDeleteNPagedLookasideList, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IRP, IoBuildDeviceIoControlRequest, IoConnectInterrupt, IoCsqInitialize, IoCsqInsertIrp, IoCsqRemoveNextIrp, IoForwardIrpSynchronously, IoGetConfigurationInformation, KAFFINITY, KEVENT, KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeInitializeEvent, KeReleaseInStackQueuedSpinLock, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PCONFIGURATION_INFORMATION, PDEVICE_OBJECT, PDEVICE_RELATIONS, PGUID, PIO_CSQ, PIO_RESOURCE_REQUIREMENTS_LIST, PIO_STACK_LOCATION, PIRP, PKINTERRUPT, PKIRQL};
use core::ffi::c_void;
unsafe extern "system" fn PortStarvedCsqInsertIrp(Csq: PIO_CSQ, Irp: PIRP) {
    let Queue: PSTARVED_REQUEST_QUEUE = CONTAINING_RECORD!(Csq, STARVED_REQUEST_QUEUE, Csq);

    InsertTailList(
        core::ptr::addr_of_mut!((*Queue).ListHead),
        core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry),
    );
}
unsafe extern "system" fn PortStarvedCsqRemoveIrp(Csq: PIO_CSQ, Irp: PIRP) {
    let _ = Csq;

    RemoveEntryList(core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry));
}
unsafe extern "system" fn PortStarvedCsqPeekNextIrp(
    Csq: PIO_CSQ,
    Irp: PIRP,
    PeekContext: PVOID,
) -> PIRP {
    let Queue: PSTARVED_REQUEST_QUEUE = CONTAINING_RECORD!(Csq, STARVED_REQUEST_QUEUE, Csq);
    let mut Entry: PLIST_ENTRY;

    UNREFERENCED_PARAMETER!(PeekContext);

    Entry = if Irp.is_null() {
        (*Queue).ListHead.Flink
    } else {
        (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry.Flink
    };

    if Entry == core::ptr::addr_of_mut!((*Queue).ListHead) {
        return core::ptr::null_mut();
    }

    return CONTAINING_RECORD!(Entry, IRP, Tail.Overlay.__bindgen_anon_2.ListEntry);
}

unsafe extern "system" fn PortStarvedCsqAcquireLock(Csq: PIO_CSQ, Irql: PKIRQL) {
    let Queue: PSTARVED_REQUEST_QUEUE = (Csq as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(STARVED_REQUEST_QUEUE, Csq) as ULONG_PTR)
        as PSTARVED_REQUEST_QUEUE;

    KeAcquireSpinLock(core::ptr::addr_of_mut!((*Queue).Lock), Irql);
}
unsafe extern "system" fn PortStarvedCsqReleaseLock(mut Csq: PIO_CSQ, mut Irql: KIRQL) {
    let mut Queue: PSTARVED_REQUEST_QUEUE = (Csq as usize)
        .wrapping_sub(core::mem::offset_of!(STARVED_REQUEST_QUEUE, Csq))
        as PSTARVED_REQUEST_QUEUE;
    KeReleaseSpinLock(core::ptr::addr_of_mut!((*Queue).Lock), Irql);
}

unsafe extern "system" fn PortStarvedCsqCompleteCanceledIrp(Csq: PIO_CSQ, Irp: PIRP) {
    UNREFERENCED_PARAMETER!(Csq);

    StorpCompleteRequest(Irp, (SRB_STATUS_ABORTED) as UCHAR, STATUS_CANCELLED);
}
#[no_mangle]
pub unsafe extern "C" fn PortInitializeStarvedRequestQueue(Queue: PSTARVED_REQUEST_QUEUE) -> NTSTATUS {
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Queue).Lock));
    InitializeListHead(core::ptr::addr_of_mut!((*Queue).ListHead));

    return IoCsqInitialize(
        core::ptr::addr_of_mut!((*Queue).Csq),
        Some(PortStarvedCsqInsertIrp),
        Some(PortStarvedCsqRemoveIrp),
        Some(PortStarvedCsqPeekNextIrp),
        Some(PortStarvedCsqAcquireLock),
        Some(PortStarvedCsqReleaseLock),
        Some(PortStarvedCsqCompleteCanceledIrp),
    );
}
#[no_mangle]
pub unsafe extern "C" fn PortQueueStarvedRequest(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) {
    IoCsqInsertIrp(
        core::ptr::addr_of_mut!((*FdoExtension).StarvedRequests.Csq),
        Irp,
        core::ptr::null_mut(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn PortRetryStarvedRequest(FdoExtension: PFDO_DEVICE_EXTENSION) {
    let mut Irp: PIRP = IoCsqRemoveNextIrp(core::ptr::addr_of_mut!((*FdoExtension).StarvedRequests.Csq), core::ptr::null_mut());
    if Irp.is_null() {
        return;
    }
    PortPdoScsi((*IoGetCurrentIrpStackLocation(Irp)).DeviceObject, Irp);
}
#[no_mangle]
pub unsafe extern "C" fn PortFlushStarvedRequests(mut FdoExtension: PFDO_DEVICE_EXTENSION) {
    let mut Irp: PIRP;
    loop {
        Irp = IoCsqRemoveNextIrp(core::ptr::addr_of_mut!((*FdoExtension).StarvedRequests.Csq), core::ptr::null_mut());
        if Irp.is_null() {
            break;
        }
        StorpCompleteRequest(Irp, SRB_STATUS_NO_DEVICE as UCHAR, STATUS_NO_SUCH_DEVICE);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PortAllocateRequestReserve(FdoExtension: PFDO_DEVICE_EXTENSION) {
    let SrbExtensionSize: ULONG = (*(*FdoExtension).HwInitData).SrbExtensionSize;

    (*FdoExtension).ReservedRequestReference = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_QUEUED_REQUEST_REFERENCE>() as SIZE_T,
        TAG_QUEUED_REQUEST as ULONG,
    ) as *mut _QUEUED_REQUEST_REFERENCE;
    if (*FdoExtension).ReservedRequestReference.is_null() {
        DPRINT1!("Failed to reserve a request packet\n");
        return;
    }

    if SrbExtensionSize == 0 as ULONG {
        return;
    }

    (*FdoExtension).ReservedSrbExtension = ExAllocatePoolWithTag(
        NonPagedPool,
        SrbExtensionSize as SIZE_T,
        TAG_SRB_EXTENSION as ULONG,
    );
    if (*FdoExtension).ReservedSrbExtension.is_null() {
        DPRINT1!("Failed to reserve an SRB extension\n");

        /* A packet alone is useless if the miniport wants an SRB extension */
        ExFreePoolWithTag(
            (*FdoExtension).ReservedRequestReference as PVOID,
            TAG_QUEUED_REQUEST as ULONG,
        );
        (*FdoExtension).ReservedRequestReference =
            core::ptr::null_mut::<_QUEUED_REQUEST_REFERENCE>();
    }
}
#[no_mangle]
pub unsafe extern "C" fn PortFreeRequestReserve(FdoExtension: PFDO_DEVICE_EXTENSION) {
    if !(*FdoExtension).ReservedSrbExtension.is_null() {
        ExFreePoolWithTag((*FdoExtension).ReservedSrbExtension, TAG_SRB_EXTENSION);
        (*FdoExtension).ReservedSrbExtension = core::ptr::null_mut();
    }

    if !(*FdoExtension).ReservedRequestReference.is_null() {
        ExFreePoolWithTag(((*FdoExtension).ReservedRequestReference).cast(), TAG_QUEUED_REQUEST);
        (*FdoExtension).ReservedRequestReference = core::ptr::null_mut();
    }
}

unsafe extern "system" fn PortFdoInterruptRoutine(
    Interrupt: PKINTERRUPT,
    ServiceContext: PVOID,
) -> BOOLEAN {
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION;

    DeviceExtension = ServiceContext as PFDO_DEVICE_EXTENSION;

    return MiniportHwInterrupt(core::ptr::addr_of_mut!((*DeviceExtension).Miniport));
}

unsafe extern "C" fn PortFdoConnectInterrupt(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Vector: ULONG = 0;
    let mut Irql: KIRQL = 0;
    let mut InterruptMode: KINTERRUPT_MODE = LevelSensitive;
    let mut ShareVector: BOOLEAN = 0;
    let mut Affinity: KAFFINITY = 0;
    let mut Status: NTSTATUS = 0;

    DPRINT!("PortFdoConnectInterrupt(%p)\n", DeviceExtension);

    /* No resources, no interrupt. Done! */
    if (*DeviceExtension).AllocatedResources.is_null()
        || (*DeviceExtension).TranslatedResources.is_null()
    {
        DPRINT!("Checkpoint\n");
        return STATUS_SUCCESS;
    }

    /* Get the interrupt data from the resource list */
    Status = GetResourceListInterrupt(
        DeviceExtension,
        core::ptr::addr_of_mut!(Vector),
        core::ptr::addr_of_mut!(Irql),
        core::ptr::addr_of_mut!(InterruptMode),
        core::ptr::addr_of_mut!(ShareVector),
        core::ptr::addr_of_mut!(Affinity),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT!("GetResourceListInterrupt() failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    DPRINT1!("Vector: %lu\n", Vector);
    DPRINT1!("Irql: %lu\n", Irql as core::ffi::c_uint);

    DPRINT1!("Affinity: 0x%08lx\n", Affinity);

    /* Connect the interrupt */
    Status = IoConnectInterrupt(
        core::ptr::addr_of_mut!((*DeviceExtension).Interrupt),
        Some(PortFdoInterruptRoutine),
        DeviceExtension as PVOID,
        core::ptr::null_mut::<ULONG_PTR>(),
        Vector,
        Irql,
        Irql,
        InterruptMode,
        ShareVector,
        Affinity,
        FALSE,
    );
    if NT_SUCCESS!(Status) {
        (*DeviceExtension).InterruptIrql = Irql as ULONG;
    } else {
        DPRINT1!("IoConnectInterrupt() failed (Status 0x%08lx)\n", Status);
    }

    return Status;
}

unsafe extern "C" fn PortFdoStartMiniport(mut DeviceExtension: PFDO_DEVICE_EXTENSION) -> NTSTATUS {
    let mut InitData: PHW_INITIALIZATION_DATA = core::ptr::null_mut();
    let mut InterfaceType: INTERFACE_TYPE;
    let mut Status: NTSTATUS;

    DPRINT1!("PortFdoStartMiniport(%p)\n", DeviceExtension);

    /* Get the interface type of the lower device */
    InterfaceType = GetBusInterface((*DeviceExtension).LowerDevice);
    if InterfaceType as ::core::ffi::c_int == InterfaceTypeUndefined as ::core::ffi::c_int {
        return STATUS_NO_SUCH_DEVICE;
    }

    /* Get the driver init data for the given interface type */
    InitData = PortGetDriverInitData((*DeviceExtension).DriverExtension, InterfaceType);
    if InitData.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    /* Initialize the miniport */
    Status = MiniportInitialize(
        core::ptr::addr_of_mut!((*DeviceExtension).Miniport),
        DeviceExtension,
        InitData,
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!("MiniportInitialize() failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    /* Call the miniports FindAdapter function */
    Status = MiniportFindAdapter(core::ptr::addr_of_mut!((*DeviceExtension).Miniport));
    if !NT_SUCCESS!(Status) {
        DPRINT1!("MiniportFindAdapter() failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    /* Connect the configured interrupt */
    Status = PortFdoConnectInterrupt(DeviceExtension);
    if !NT_SUCCESS!(Status) {
        DPRINT1!("PortFdoConnectInterrupt() failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    // FIXME: Find an appropriate place
    (*DeviceExtension).OutstandingRequestMax = (*DeviceExtension).Miniport.PortConfig.MaxNumberOfIO;

    /* Call the miniports HwInitialize function */
    Status = MiniportHwInitialize(core::ptr::addr_of_mut!((*DeviceExtension).Miniport));
    if !NT_SUCCESS!(Status) {
        DPRINT1!("MiniportHwInitialize() failed (Status 0x%08lx)\n", Status);
        return Status;
    }

    /* Call the HwPassiveInitRoutine function, if available */
    if (*DeviceExtension).HwPassiveInitRoutine.is_some() {
        DPRINT1!("Calling HwPassiveInitRoutine()\n");
        if (*DeviceExtension)
            .HwPassiveInitRoutine
            .unwrap_unchecked()(
            core::ptr::addr_of_mut!((*(*DeviceExtension).Miniport.MiniportExtension).HwDeviceExtension) as PVOID,
        ) == 0
        {
            DPRINT1!("HwPassiveInitRoutine() failed\n");
            return STATUS_UNSUCCESSFUL;
        }
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn PortFdoStartDevice(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut ConfigInfo: PCONFIGURATION_INFORMATION = core::ptr::null_mut();
    let mut Stack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    DPRINT1!("PortFdoStartDevice(%p %p)\n", DeviceExtension, Irp);

    ASSERT!((*DeviceExtension).ExtensionType == EXTENSION_TYPE_FdoExtension);

    /* Get the current stack location */
    Stack = IoGetCurrentIrpStackLocation(Irp);

    /* Start the lower device if the FDO is in 'stopped' state */
    if (*DeviceExtension).PnpState == dsStopped {
        if IoForwardIrpSynchronously((*DeviceExtension).LowerDevice, Irp) != 0 {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        } else {
            Status = STATUS_UNSUCCESSFUL;
        }

        if !NT_SUCCESS!(Status) {
            DPRINT1!("Lower device failed the IRP (Status 0x%08lx)\n", Status);
            return Status;
        }
    }

    /* Change to the 'started' state */
    (*DeviceExtension).PnpState = dsStarted;

    /* Copy the raw and translated resource lists into the device extension */
    if !(*Stack).Parameters.StartDevice.AllocatedResources.is_null()
        && !(*Stack).Parameters.StartDevice.AllocatedResourcesTranslated.is_null()
    {
        (*DeviceExtension).AllocatedResources = CopyResourceList(
            NonPagedPool,
            (*Stack).Parameters.StartDevice.AllocatedResources,
        );
        if (*DeviceExtension).AllocatedResources.is_null() {
            return STATUS_NO_MEMORY;
        }

        (*DeviceExtension).TranslatedResources = CopyResourceList(
            NonPagedPool,
            (*Stack).Parameters.StartDevice.AllocatedResourcesTranslated,
        );
        if (*DeviceExtension).TranslatedResources.is_null() {
            return STATUS_NO_MEMORY;
        }
    }

    /* Get the bus interface of the lower (bus) device */
    Status = QueryBusInterface(
        (*DeviceExtension).LowerDevice,
        core::ptr::addr_of!(GUID_BUS_INTERFACE_STANDARD) as PGUID,
        core::mem::size_of::<BUS_INTERFACE_STANDARD>() as USHORT,
        1,
        core::ptr::addr_of_mut!((*DeviceExtension).BusInterface),
        core::ptr::null_mut(),
    );
    DPRINT1!("Status: 0x%08lx\n", Status);
    if NT_SUCCESS!(Status) {
        DPRINT1!("Context: %p\n", (*DeviceExtension).BusInterface.Context);
        (*DeviceExtension).BusInitialized = TRUE;
    }

    /* Start the miniport (FindAdapter & Initialize) */
    Status = PortFdoStartMiniport(DeviceExtension);
    if !NT_SUCCESS!(Status) {
        DPRINT1!("FdoStartMiniport() failed (Status 0x%08lx)\n", Status);
        (*DeviceExtension).PnpState = dsStopped;
    } else {
        /* SrbExtensionSize is only known once the miniport is up */
        PortAllocateRequestReserve(DeviceExtension);
    }

    /* Claim and increase SCSI port number */
    /* TODO: Reverse this when stopping adapter */
    ConfigInfo = IoGetConfigurationInformation();
    (*DeviceExtension).ScsiPortNumber = (*ConfigInfo).ScsiPortCount as LONG;
    (*ConfigInfo).ScsiPortCount = (*ConfigInfo).ScsiPortCount.wrapping_add(1);

    return Status;
}

unsafe extern "C" fn PortSendReportLuns(
    mut PdoExtension_local: PPDO_DEVICE_EXTENSION,
    mut LunCount: PULONG,
) -> NTSTATUS {
    // C leaves it unset; `while (KeepTrying)` always runs once, which rustc cannot see.
    let mut Status: NTSTATUS = 0;
    let mut Srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let mut Cdb: PCDB;
    let mut Irp: PIRP;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut IrpStack: PIO_STACK_LOCATION;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut SenseBuffer: PSENSE_DATA;
    let mut BufferSize: ULONG;
    let mut ReportLunsData: PREPORT_LUNS_DATA;
    let mut KeepTrying: BOOLEAN = TRUE;
    let mut BufferReallocated: BOOLEAN = FALSE;
    let mut RetryCount: ULONG = 0;

    DPRINT!("PortSendReportLuns(%p)\n", PdoExtension_local);

    /* Allocate sense buffer */
    /* TODO: Reuse sense buffers */
    /* FIXME: Do we need sense buffer here? Can I pass a NULL? */
    SenseBuffer = ExAllocatePoolWithTag(NonPagedPool, (SENSE_BUFFER_SIZE as _), TAG_SENSE_DATA) as PSENSE_DATA;
    if SenseBuffer.is_null()
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    /* Allocate REPORT LUNS return buffer. Use buffer size for the case of a single LUN */
    BufferSize = core::mem::size_of::<REPORT_LUNS_DATA>() as ULONG;
    ReportLunsData =
        ExAllocatePoolWithTag(NonPagedPool, (BufferSize) as _, TAG_REPORT_LUN_DATA) as PREPORT_LUNS_DATA;
    if ReportLunsData.is_null()
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    while KeepTrying != 0
    {
        /* Initialize event for waiting */
        KeInitializeEvent(core::ptr::addr_of_mut!(Event),
                          _EVENT_TYPE_NotificationEvent,
                          FALSE);

        /* Create an IRP */
        Irp = IoBuildDeviceIoControlRequest(IOCTL_SCSI_EXECUTE_IN,
                                            (*PdoExtension_local).Device,
                                            core::ptr::null_mut(),
                                            0,
                                            ReportLunsData as PVOID,
                                            BufferSize,
                                            TRUE,
                                            core::ptr::addr_of_mut!(Event),
                                            core::ptr::addr_of_mut!(IoStatusBlock));
        if Irp.is_null()
        {
            DPRINT!("IoBuildDeviceIoControlRequest() failed\n");

            /* Quit the loop */
            Status = STATUS_INSUFFICIENT_RESOURCES;
            KeepTrying = FALSE;
            continue;
        }

        /* Prepare SRB */
        RtlZeroMemory!(core::ptr::addr_of_mut!(Srb) as PVOID, core::mem::size_of::<SCSI_REQUEST_BLOCK>());

        Srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
        Srb.OriginalRequest = Irp as PVOID;
        Srb.PathId = ((*PdoExtension_local).Bus) as u8;
        Srb.TargetId = ((*PdoExtension_local).Target) as u8;
        Srb.Lun = ((*PdoExtension_local).Lun) as u8;
        Srb.Function = (SRB_FUNCTION_EXECUTE_SCSI as u8);
        Srb.SrbFlags = SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
        Srb.TimeOutValue = 4;
        Srb.CdbLength = 12;

        Srb.SenseInfoBuffer = SenseBuffer as PVOID;
        Srb.SenseInfoBufferLength = (SENSE_BUFFER_SIZE as u8);

        Srb.DataBuffer = ReportLunsData as PVOID;
        Srb.DataTransferLength = BufferSize;

        /* Attach Srb to the Irp */
        IrpStack = IoGetNextIrpStackLocation(Irp);
        (*IrpStack).Parameters.Scsi.Srb = core::ptr::addr_of_mut!(Srb).cast();

        /* Fill in CDB */
        Cdb = core::ptr::addr_of_mut!(Srb.Cdb) as PCDB;
        (*Cdb).REPORT_LUNS.OperationCode = (SCSIOP_REPORT_LUNS as u8);
        (*Cdb).REPORT_LUNS.AllocationLength[0] = ((BufferSize >> (0*8)) & 0xFF) as UCHAR;
        (*Cdb).REPORT_LUNS.AllocationLength[1] = ((BufferSize >> (1*8)) & 0xFF) as UCHAR;
        (*Cdb).REPORT_LUNS.AllocationLength[2] = ((BufferSize >> (2*8)) & 0xFF) as UCHAR;
        (*Cdb).REPORT_LUNS.AllocationLength[3] = ((BufferSize >> (3*8)) & 0xFF) as UCHAR;

        /* Call the driver */
        Status = IoCallDriver((*PdoExtension_local).Device, Irp);

        /* Wait for it to complete */
        if Status == STATUS_PENDING || Srb.SrbStatus == (SRB_STATUS_PENDING as u8)
        {
            DPRINT1!("PortSendReportLuns(): Waiting for the driver to process request...\n");
            KeWaitForSingleObject(core::ptr::addr_of_mut!(Event) as PVOID,
                                  _KWAIT_REASON_Executive,
                                  (0i32) as i8,
                                  FALSE,
                                  core::ptr::null_mut());
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        DPRINT!("PortSendReportLuns(): Request processed by driver, status = 0x%08X\n", Status);

        if SRB_STATUS!(Srb.SrbStatus) == (SRB_STATUS_SUCCESS as i32)
        {
            let mut DataSize: ULONG = 0;

            /* Extract LUN count from the return data. This is the sole purpose of this function */
            DataSize |= (((*ReportLunsData).LunListLength[0] as ULONG) << (8*0));
            DataSize |= (((*ReportLunsData).LunListLength[1] as ULONG) << (8*1));
            DataSize |= (((*ReportLunsData).LunListLength[2] as ULONG) << (8*2));
            DataSize |= (((*ReportLunsData).LunListLength[3] as ULONG) << (8*3));
            *LunCount = ((DataSize - (FIELD_OFFSET!(REPORT_LUNS_DATA, LunDescriptor)) as u32) /
                         (core::mem::size_of::<LUN_DESCRIPTOR>() as ULONG));

            /* Quit the loop */
            Status = STATUS_SUCCESS;
            KeepTrying = FALSE;
            continue;
        }

        DPRINT1!("REPORT_LUNS SRB failed with SrbStatus 0x%08X Status %08X\n",
                Srb.SrbStatus as core::ffi::c_uint, Status);

        /* Check if the queue is frozen */
        if Srb.SrbStatus & (SRB_STATUS_QUEUE_FROZEN as u8) != 0
        {
            /* Something weird happened, deal with it (unfreeze the queue) */
            KeepTrying = FALSE;

            DPRINT1!("PortSendReportLuns(): the queue is frozen at TargetId %d\n", Srb.TargetId as core::ffi::c_uint);
            /* TODO: What do we do with this crap */
        }

        /* Check if data overrun happened, then resize buffer once */
        /* FIXME: Needs coverage test */
        if SRB_STATUS!(Srb.SrbStatus) == (SRB_STATUS_DATA_OVERRUN as i32)
        {
            /* If the buffer has already been reallocated we ditch the device */
            if BufferReallocated != 0
            {
                DPRINT1!("Data overrun again (!) at TargetId %d, give up\n", (*PdoExtension_local).Target);

                Status = STATUS_IO_DEVICE_ERROR;
                KeepTrying = FALSE;
                continue;
            }

            /* Miniport will return how many bytes should we actually allocate */
            BufferSize = 0;
            BufferSize |= (((*ReportLunsData).LunListLength[0] as ULONG) << (8*0));
            BufferSize |= (((*ReportLunsData).LunListLength[1] as ULONG) << (8*1));
            BufferSize |= (((*ReportLunsData).LunListLength[2] as ULONG) << (8*2));
            BufferSize |= (((*ReportLunsData).LunListLength[3] as ULONG) << (8*3));

            DPRINT1!("Data overrun at TargetId %d, %d bytes needed\n",
                   (*PdoExtension_local).Target,
                   BufferSize);

            /* Allocate a larger buffer */
            ExFreePoolWithTag(ReportLunsData as PVOID, TAG_REPORT_LUN_DATA);
            ReportLunsData = ExAllocatePoolWithTag(NonPagedPool, (BufferSize) as _, TAG_REPORT_LUN_DATA) as PREPORT_LUNS_DATA;
            if ReportLunsData.is_null()
            {
                DPRINT1!("Cannot reallocate REPORT_LUNS buffer\n");

                Status = STATUS_INSUFFICIENT_RESOURCES;
                KeepTrying = FALSE;
                continue;
            }
            BufferReallocated = TRUE;

            /* Continue trying with a larger buffer */
            continue;
        } else
        {
            /* Retry a couple of times if no timeout happened */
            if (RetryCount < 2) &&
                (SRB_STATUS!(Srb.SrbStatus) != (SRB_STATUS_NO_DEVICE as i32)) &&
                (SRB_STATUS!(Srb.SrbStatus) != (SRB_STATUS_SELECTION_TIMEOUT as i32))
            {
                RetryCount += 1;
                KeepTrying = TRUE;
            }
            else
            {
                /* That's all, quit the loop */
                KeepTrying = FALSE;

                /* Set status according to SRB status */
                if SRB_STATUS!(Srb.SrbStatus) == (SRB_STATUS_BAD_FUNCTION as i32) ||
                    SRB_STATUS!(Srb.SrbStatus) == (SRB_STATUS_BAD_SRB_BLOCK_LENGTH as i32)
                {
                    Status = STATUS_INVALID_DEVICE_REQUEST;
                }
                else
                {
                    Status = STATUS_IO_DEVICE_ERROR;
                }
            }
        }
    }

    /* Free Sense data and Report LUNs buffer */
    ExFreePoolWithTag(ReportLunsData as PVOID, TAG_REPORT_LUN_DATA);
    ExFreePoolWithTag(SenseBuffer as PVOID, TAG_SENSE_DATA);

    DPRINT1!("PortSendReportLuns() done with Status 0x%08X\n", Status);

    return Status;
}

unsafe extern "C" fn PortSendInquiry(PdoExtension: PPDO_DEVICE_EXTENSION) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut IrpStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut SenseBuffer: PSENSE_DATA = core::ptr::null_mut();
    let mut KeepTrying: BOOLEAN = TRUE;
    let mut RetryCount: ULONG = 0;
    let mut Srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let mut Cdb: PCDB = core::ptr::null_mut();

    DPRINT!("PortSendInquiry(%p)\n", PdoExtension);

    if (*PdoExtension).InquiryBuffer.is_null() {
        (*PdoExtension).InquiryBuffer = ExAllocatePoolWithTag(
            NonPagedPool,
            (INQUIRYDATABUFFERSIZE as _),
            TAG_INQUIRY_DATA,
        ) as PINQUIRYDATA;
        if (*PdoExtension).InquiryBuffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    SenseBuffer = ExAllocatePoolWithTag(
        NonPagedPool,
        (SENSE_BUFFER_SIZE as _),
        TAG_SENSE_DATA,
    ) as PSENSE_DATA;
    if SenseBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    while KeepTrying != 0 {
        /* Initialize event for waiting */
        KeInitializeEvent(
            core::ptr::addr_of_mut!(Event),
            _EVENT_TYPE_NotificationEvent,
            FALSE,
        );

        /* Create an IRP */
        Irp = IoBuildDeviceIoControlRequest(
            IOCTL_SCSI_EXECUTE_IN,
            (*PdoExtension).Device,
            core::ptr::null_mut(),
            0,
            (*PdoExtension).InquiryBuffer as PVOID,
            INQUIRYDATABUFFERSIZE,
            TRUE,
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(IoStatusBlock),
        );
        if Irp.is_null() {
            DPRINT1!("IoBuildDeviceIoControlRequest() failed\n");

            /* Quit the loop */
            Status = STATUS_INSUFFICIENT_RESOURCES;
            KeepTrying = FALSE;
            continue;
        }

        /* Prepare SRB */
        RtlZeroMemory!(
            core::ptr::addr_of_mut!(Srb) as PVOID,
            core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
        );

        Srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
        Srb.OriginalRequest = Irp as PVOID;
        Srb.PathId = (*PdoExtension).Bus as UCHAR;
        Srb.TargetId = (*PdoExtension).Target as UCHAR;
        Srb.Lun = (*PdoExtension).Lun as UCHAR;
        Srb.Function = (SRB_FUNCTION_EXECUTE_SCSI as u8);
        Srb.SrbFlags = (SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER) as ULONG;
        Srb.TimeOutValue = 4;
        Srb.CdbLength = 6;

        Srb.SenseInfoBuffer = SenseBuffer as PVOID;
        Srb.SenseInfoBufferLength = (SENSE_BUFFER_SIZE as u8);

        Srb.DataBuffer = (*PdoExtension).InquiryBuffer as PVOID;
        Srb.DataTransferLength = INQUIRYDATABUFFERSIZE;

        /* Attach Srb to the Irp */
        IrpStack = IoGetNextIrpStackLocation(Irp);
        (*IrpStack).Parameters.Scsi.Srb = core::ptr::addr_of_mut!(Srb).cast();

        /* Fill in CDB */
        Cdb = core::ptr::addr_of_mut!(Srb.Cdb) as *mut UCHAR as PCDB;
        (*Cdb).CDB6INQUIRY.OperationCode = (SCSIOP_INQUIRY as u8);
        (*Cdb).CDB6INQUIRY.set_LogicalUnitNumber(((*PdoExtension).Lun as UCHAR) as _);
        (*Cdb).CDB6INQUIRY.AllocationLength = (INQUIRYDATABUFFERSIZE as u8);

        /* Call the driver */
        Status = IoCallDriver((*PdoExtension).Device, Irp);

        /* Wait for it to complete */
        if Status == STATUS_PENDING {
            DPRINT1!("PortSendInquiry(): Waiting for the driver to process request...\n");
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as PVOID,
                _KWAIT_REASON_Executive,
                (0i32) as i8,
                FALSE,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        DPRINT1!("PortSendInquiry(): Request processed by driver, status = 0x%08X\n", Status);

        if SRB_STATUS!(Srb.SrbStatus) == (SRB_STATUS_SUCCESS as i32) {
            DPRINT1!("Found a device!\n");

            /* Quit the loop */
            Status = STATUS_SUCCESS;
            KeepTrying = FALSE;
            continue;
        }

        DPRINT1!("Inquiry SRB failed with SrbStatus 0x%08X\n", Srb.SrbStatus as core::ffi::c_uint);

        /* Check if the queue is frozen */
        if Srb.SrbStatus & (SRB_STATUS_QUEUE_FROZEN as u8) != 0 {
            /* Something weird happened, deal with it (unfreeze the queue) */
            KeepTrying = FALSE;

            DPRINT1!("SpiSendInquiry(): the queue is frozen at TargetId %d\n", Srb.TargetId as core::ffi::c_uint);
        }

        /* Check if data overrun happened */
        if SRB_STATUS!(Srb.SrbStatus) == (SRB_STATUS_DATA_OVERRUN as i32) {
            DPRINT!("Data overrun at TargetId %d\n", (*PdoExtension).Target);

            /* Quit the loop */
            Status = STATUS_SUCCESS;
            KeepTrying = FALSE;
        } else if (Srb.SrbStatus & (SRB_STATUS_AUTOSENSE_VALID as u8)) != 0
            && (*SenseBuffer).SenseKey() == (SCSI_SENSE_ILLEGAL_REQUEST as u8)
        {
            /* LUN is not valid, but some device responds there.
               Mark it as invalid anyway */

            /* Quit the loop */
            Status = STATUS_INVALID_DEVICE_REQUEST;
            KeepTrying = FALSE;
        } else {
            /* Retry a couple of times if no timeout happened */
            if (RetryCount < 2)
                && (SRB_STATUS!(Srb.SrbStatus) != (SRB_STATUS_NO_DEVICE as i32))
                && (SRB_STATUS!(Srb.SrbStatus) != (SRB_STATUS_SELECTION_TIMEOUT as i32))
            {
                RetryCount = RetryCount.wrapping_add(1);
                KeepTrying = TRUE;
            } else {
                /* That's all, quit the loop */
                KeepTrying = FALSE;

                /* Set status according to SRB status */
                if SRB_STATUS!(Srb.SrbStatus) == (SRB_STATUS_BAD_FUNCTION as i32)
                    || SRB_STATUS!(Srb.SrbStatus) == (SRB_STATUS_BAD_SRB_BLOCK_LENGTH as i32)
                {
                    Status = STATUS_INVALID_DEVICE_REQUEST;
                } else {
                    Status = STATUS_IO_DEVICE_ERROR;
                }
            }
        }
    }

    /* Free the sense buffer */
    ExFreePoolWithTag(SenseBuffer as PVOID, TAG_SENSE_DATA);

    DPRINT1!("PortSendInquiry() done with Status 0x%08X\n", Status);

    return Status;
}

unsafe extern "C" fn PortFdoScanBus(mut DeviceExtension: PFDO_DEVICE_EXTENSION) -> NTSTATUS {
    let mut PdoExtension: PPDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Bus: ULONG = 0;
    let mut Target: ULONG = 0;
    let mut Status: NTSTATUS;
    let mut LunCount: ULONG = 0;

    DPRINT!("PortFdoScanBus(%p)\n", DeviceExtension);

    DPRINT!("NumberOfBuses: %lu\n", (*DeviceExtension).Miniport.PortConfig.NumberOfBuses);
    DPRINT!("MaximumNumberOfTargets: %lu\n", (*DeviceExtension).Miniport.PortConfig.MaximumNumberOfTargets);
    DPRINT!("MaximumNumberOfLogicalUnits: %lu\n", (*DeviceExtension).Miniport.PortConfig.MaximumNumberOfLogicalUnits);

    /* Scan all buses */
    Bus = 0;
    while Bus < (*DeviceExtension).Miniport.PortConfig.NumberOfBuses as ULONG {
        DPRINT!("Scanning bus %ld\n", Bus);

        /* Scan all targets */
        Target = 0;
        while Target < (*DeviceExtension).Miniport.PortConfig.MaximumNumberOfTargets as ULONG {
            DPRINT!("  Scanning target %ld:%ld\n", Bus, Target);

            DPRINT!("    Scanning logical unit %ld:%ld:%ld\n", Bus, Target, 0);
            Status = PortCreatePdo(
                DeviceExtension,
                Bus,
                Target,
                0,
                core::ptr::addr_of_mut!(PdoExtension),
            );
            if NT_SUCCESS!(Status) {
                /* Send Report LUNs first */
                PortSendReportLuns(PdoExtension, core::ptr::addr_of_mut!(LunCount));

                /* Scan LUN 0 */
                Status = PortSendInquiry(PdoExtension);
                DPRINT!("PortSendInquiry returned 0x%08lx\n", Status);
                if !NT_SUCCESS!(Status) {
                    PortDeletePdo(PdoExtension);
                } else {
                    DPRINT!("VendorId: %.8s\n", (*(*PdoExtension).InquiryBuffer).VendorId.as_ptr());
                    DPRINT!("ProductId: %.16s\n", (*(*PdoExtension).InquiryBuffer).ProductId.as_ptr());
                    DPRINT!("ProductRevisionLevel: %.4s\n", (*(*PdoExtension).InquiryBuffer).ProductRevisionLevel.as_ptr());
                    DPRINT!("VendorSpecific: %.20s\n", (*(*PdoExtension).InquiryBuffer).VendorSpecific.as_ptr());
                }
            }

            Target = Target.wrapping_add(1);
        }
        Bus = Bus.wrapping_add(1);
    }

    DPRINT!("PortFdoScanBus() done!\n");

    return STATUS_SUCCESS;
}

unsafe extern "C" fn PortFdoQueryBusRelations(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut Information: PULONG_PTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut DeviceRelations: PDEVICE_RELATIONS = core::ptr::null_mut();
    let mut PdoExtension_local: PPDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut PdoEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut PdoCount: ULONG = 0;
    let mut PdoIndex: ULONG = 0 as ULONG;

    DPRINT!("PortFdoQueryBusRelations(%p %p)\n",
            DeviceExtension, Information);

    Status = PortFdoScanBus(DeviceExtension);

    PdoCount = (*DeviceExtension).PdoCount;

    DPRINT1!("Units found: %lu\n", PdoCount);

    /* Following part referred to SCSIport */
    DeviceRelations = ExAllocatePoolWithTag(
        (PagedPool),
        (core::mem::offset_of!(DEVICE_RELATIONS, Objects)
            + (PdoCount as usize) * core::mem::size_of::<_DEVICE_OBJECT>()) as SIZE_T,
        (TAG_DEVICE_RELATION) as ULONG,
    ) as PDEVICE_RELATIONS;
    if DeviceRelations.is_null()
    {
        *Information = 0 as ULONG_PTR;
        return STATUS_INSUFFICIENT_RESOURCES as NTSTATUS;
    }

    /* Set PDO pointers */
    KeAcquireInStackQueuedSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).PdoListLock), core::ptr::addr_of_mut!(LockHandle));

    PdoEntry = (*DeviceExtension).PdoListHead.Flink as PLIST_ENTRY;
    while PdoEntry != core::ptr::addr_of_mut!((*DeviceExtension).PdoListHead) as PLIST_ENTRY
        && PdoIndex < PdoCount
    {
        PdoExtension_local = (PdoEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(PDO_DEVICE_EXTENSION, PdoListEntry) as ULONG_PTR,
        ) as PPDO_DEVICE_EXTENSION;

        if !((*PdoExtension_local).PnpState as ::core::ffi::c_uint
            == dsRemoved as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*PdoExtension_local).PnpState as ::core::ffi::c_uint
                == dsSurpriseRemoved as ::core::ffi::c_int as ::core::ffi::c_uint)
        {
            ObReferenceObject(((*PdoExtension_local).Device).cast());

            *(core::ptr::addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT)
                .offset(PdoIndex as isize) = (*PdoExtension_local).Device;

            /* Next one */
            PdoIndex = PdoIndex.wrapping_add(1);
        }

        PdoEntry = (*PdoEntry).Flink as PLIST_ENTRY;
    }

    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));

    (*DeviceRelations).Count = PdoIndex;

    *Information = DeviceRelations as ULONG_PTR;

    return Status;
}
unsafe extern "C" fn PortFdoFilterRequirements(
    DeviceExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut RequirementsList: PIO_RESOURCE_REQUIREMENTS_LIST = core::ptr::null_mut();

    DPRINT!("PortFdoFilterRequirements(%p %p)\n", DeviceExtension, Irp);

    /* Get the bus number and the slot number */
    RequirementsList = (*Irp).IoStatus.Information as PIO_RESOURCE_REQUIREMENTS_LIST;
    if !RequirementsList.is_null() {
        (*DeviceExtension).BusNumber = (*RequirementsList).BusNumber;
        (*DeviceExtension).SlotNumber = (*RequirementsList).SlotNumber;
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn FdoFindLun(
    FdoExtension: PFDO_DEVICE_EXTENSION,
    Bus: ULONG,
    Target: ULONG,
    Lun: ULONG,
) -> PPDO_DEVICE_EXTENSION {
    let mut PdoExtension: PPDO_DEVICE_EXTENSION;
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut PdoEntry: PLIST_ENTRY;

    KeAcquireInStackQueuedSpinLock(core::ptr::addr_of_mut!((*FdoExtension).PdoListLock), core::ptr::addr_of_mut!(LockHandle));

    PdoEntry = (*FdoExtension).PdoListHead.Flink as PLIST_ENTRY;
    while PdoEntry != core::ptr::addr_of_mut!((*FdoExtension).PdoListHead) {
        PdoExtension = (PdoEntry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(PDO_DEVICE_EXTENSION, PdoListEntry) as ULONG_PTR,
        ) as PPDO_DEVICE_EXTENSION;

        if (*PdoExtension).Bus == Bus
            && (*PdoExtension).Target == Target
            && (*PdoExtension).Lun == Lun
        {
            KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));
            return PdoExtension;
        }

        PdoEntry = (*PdoEntry).Flink as PLIST_ENTRY;
    }

    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn PortFdoScsi(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Information: ULONG_PTR = 0;
    let mut Status: NTSTATUS = STATUS_NOT_SUPPORTED;

    DPRINT!("PortFdoScsi(%p %p)\n", DeviceObject, Irp);

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    ASSERT!(!DeviceExtension.is_null());
    ASSERT!((*DeviceExtension).ExtensionType == EXTENSION_TYPE_FdoExtension);

    (*Irp).IoStatus.Information = Information;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn PortFdoPnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Stack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Information: ULONG_PTR = 0 as ULONG_PTR;
    let mut Status: NTSTATUS = STATUS_NOT_SUPPORTED as NTSTATUS;

    DPRINT1!("PortFdoPnp(%p %p)\n", DeviceObject, Irp);

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    ASSERT!(!DeviceExtension.is_null());
    ASSERT!((*DeviceExtension).ExtensionType as u32 == EXTENSION_TYPE_FdoExtension as u32);

    Stack = IoGetCurrentIrpStackLocation(Irp);

    match (*Stack).MinorFunction as i32 {
        __c if __c == IRP_MN_START_DEVICE as _ => {
            DPRINT!("IRP_MJ_PNP / IRP_MN_START_DEVICE\n");
            Status = PortFdoStartDevice(DeviceExtension, Irp);
        }

        __c if __c == IRP_MN_QUERY_REMOVE_DEVICE as _ => {
            DPRINT1!("IRP_MJ_PNP / IRP_MN_QUERY_REMOVE_DEVICE\n");
        }

        __c if __c == IRP_MN_REMOVE_DEVICE as _ => {
            DPRINT1!("IRP_MJ_PNP / IRP_MN_REMOVE_DEVICE\n");
            PortFlushStarvedRequests(DeviceExtension);
            PortFreeRequestReserve(DeviceExtension);
            ExDeleteNPagedLookasideList(core::ptr::addr_of_mut!((*DeviceExtension).RequestReferenceLookaside));
        }

        __c if __c == IRP_MN_CANCEL_REMOVE_DEVICE as _ => {
            DPRINT1!("IRP_MJ_PNP / IRP_MN_CANCEL_REMOVE_DEVICE\n");
        }

        __c if __c == IRP_MN_STOP_DEVICE as _ => {
            DPRINT1!("IRP_MJ_PNP / IRP_MN_STOP_DEVICE\n");
        }

        __c if __c == IRP_MN_QUERY_STOP_DEVICE as _ => {
            DPRINT1!("IRP_MJ_PNP / IRP_MN_QUERY_STOP_DEVICE\n");
        }

        __c if __c == IRP_MN_CANCEL_STOP_DEVICE as _ => {
            DPRINT1!("IRP_MJ_PNP / IRP_MN_CANCEL_STOP_DEVICE\n");
        }

        __c if __c == IRP_MN_QUERY_DEVICE_RELATIONS as _ => {
            match (*Stack).Parameters.QueryDeviceRelations.Type {
                BusRelations => {
                    DPRINT!("    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / BusRelations\n");
                    Status = PortFdoQueryBusRelations(DeviceExtension, core::ptr::addr_of_mut!(Information));
                }

                RemovalRelations => {
                    DPRINT1!("    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / RemovalRelations\n");
                    return ForwardIrpAndForget((*DeviceExtension).LowerDevice, Irp);
                }

                _ => {
                    DPRINT1!(
                        "    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / Unknown type 0x%lx\n",
                        (*Stack).Parameters.QueryDeviceRelations.Type
                    );
                    return ForwardIrpAndForget((*DeviceExtension).LowerDevice, Irp);
                }
            }
        }

        __c if __c == IRP_MN_FILTER_RESOURCE_REQUIREMENTS as _ => {
            DPRINT!("IRP_MJ_PNP / IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n");
            PortFdoFilterRequirements(DeviceExtension, Irp);
            return ForwardIrpAndForget((*DeviceExtension).LowerDevice, Irp);
        }

        __c if __c == IRP_MN_QUERY_PNP_DEVICE_STATE as _ => {
            DPRINT1!("IRP_MJ_PNP / IRP_MN_QUERY_PNP_DEVICE_STATE\n");
        }

        __c if __c == IRP_MN_DEVICE_USAGE_NOTIFICATION as _ => {
            DPRINT1!("IRP_MJ_PNP / IRP_MN_DEVICE_USAGE_NOTIFICATION\n");
        }

        __c if __c == IRP_MN_SURPRISE_REMOVAL as _ => {
            DPRINT1!("IRP_MJ_PNP / IRP_MN_SURPRISE_REMOVAL\n");
        }

        _ => {
            DPRINT1!("IRP_MJ_PNP / Unknown IOCTL 0x%lx\n", (*Stack).MinorFunction as i32);
            return ForwardIrpAndForget((*DeviceExtension).LowerDevice, Irp);
        }
    }

    (*Irp).IoStatus.Information = Information;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}
