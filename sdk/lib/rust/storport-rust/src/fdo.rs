use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{BUS_INTERFACE_STANDARD, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoBuildDeviceIoControlRequest, IoConnectInterrupt, IoForwardIrpSynchronously, KAFFINITY, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PGUID, PIO_RESOURCE_REQUIREMENTS_LIST, PIO_STACK_LOCATION, PIRP, PKINTERRUPT};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation};
use ntkrnl_sys::ddk::{PCSTR, CCHAR, INT, INTERFACE_TYPE, KINTERRUPT_MODE, KPROCESSOR_MODE, CM_FULL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PINTERFACE, PPHYSICAL_ADDRESS};
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
    fn MiniportHwInterrupt(Miniport: *mut core::ffi::c_void) -> BOOLEAN;
    fn GetResourceListInterrupt( DeviceExtension: PFDO_DEVICE_EXTENSION, Vector: *mut ULONG, Irql: *mut KIRQL, InterruptMode: *mut KINTERRUPT_MODE, ShareVector: *mut BOOLEAN, Affinity: *mut KAFFINITY, ) -> NTSTATUS;
    fn GetBusInterface(LowerDevice: PDEVICE_OBJECT) -> INTERFACE_TYPE;
    fn PortGetDriverInitData( DriverExtension: PDRIVER_OBJECT_EXTENSION, InterfaceType: INTERFACE_TYPE, ) -> PHW_INITIALIZATION_DATA;
    fn MiniportInitialize( Miniport: PMINIPORT, DeviceExtension: PFDO_DEVICE_EXTENSION, InitData: PHW_INITIALIZATION_DATA, ) -> NTSTATUS;
    fn MiniportFindAdapter(Miniport: PMINIPORT) -> NTSTATUS;
    fn MiniportHwInitialize(Miniport: PMINIPORT) -> NTSTATUS;
    fn CopyResourceList( PoolType: POOL_TYPE, ResourceList: PCM_RESOURCE_LIST, ) -> PCM_RESOURCE_LIST;
    fn QueryBusInterface( DeviceObject: PDEVICE_OBJECT, InterfaceType: PGUID, Size: USHORT, Version: USHORT, Interface: PINTERFACE, InterfaceSpecificData: PVOID, ) -> NTSTATUS;
}
unsafe extern "system" {
    fn ForwardIrpAndForget(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG
unsafe extern "system" fn PortFdoInterruptRoutine(
    mut Interrupt: PKINTERRUPT,
    mut ServiceContext: PVOID,
) -> BOOLEAN {
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    if DbgPrint(
        b"(%s:%d) PortFdoInterruptRoutine(%p %p)\n\0" as *const u8 as PCSTR,
        b"drivers/storage/port/storport/fdo.c\0"
            as *const u8 as *const core::ffi::c_char,
        28 as core::ffi::c_int,
        Interrupt,
        ServiceContext,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/storage/port/storport/fdo.c\0"
                as *const u8 as *const core::ffi::c_char,
            28 as core::ffi::c_int,
        );
    }
    DeviceExtension = ServiceContext as PFDO_DEVICE_EXTENSION;
    return MiniportHwInterrupt(core::ptr::addr_of_mut!((*DeviceExtension).Miniport) as *mut core::ffi::c_void);
}
unsafe fn PortFdoConnectInterrupt(
    DeviceExtension: PFDO_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Vector: ULONG = 0;
    let mut Irql: KIRQL = 0;
    let mut InterruptMode: KINTERRUPT_MODE = LevelSensitive;
    let mut ShareVector: BOOLEAN = 0;
    let mut Affinity: KAFFINITY = 0;
    let mut Status: NTSTATUS;

    if DbgPrint(
        b"(%s:%d) PortFdoConnectInterrupt(%p)\n\0" as *const u8 as PCSTR,
        b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
        49 as ::core::ffi::c_int,
        DeviceExtension,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
            49 as ::core::ffi::c_int,
        );
    }

    /* No resources, no interrupt. Done! */
    if (*DeviceExtension).AllocatedResources.is_null()
        || (*DeviceExtension).TranslatedResources.is_null()
    {
        if DbgPrint(
            b"(%s:%d) Checkpoint\n\0" as *const u8 as PCSTR,
            b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
            55 as ::core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
                55 as ::core::ffi::c_int,
            );
        }
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
    if !NT_SUCCESS(Status) {
        if DbgPrint(
            b"(%s:%d) GetResourceListInterrupt() failed (Status 0x%08lx)\n\0" as *const u8
                as PCSTR,
            b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
            68 as ::core::ffi::c_int,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
                68 as ::core::ffi::c_int,
            );
        }
        return Status;
    }

    if DbgPrint(
        b"(%s:%d) Vector: %lu\n\0" as *const u8 as PCSTR,
        b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
        72 as ::core::ffi::c_int,
        Vector,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
            72 as ::core::ffi::c_int,
        );
    }

    if DbgPrint(
        b"(%s:%d) Irql: %lu\n\0" as *const u8 as PCSTR,
        b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
        73 as ::core::ffi::c_int,
        Irql as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
            73 as ::core::ffi::c_int,
        );
    }

    if DbgPrint(
        b"(%s:%d) Affinity: 0x%08lx\n\0" as *const u8 as PCSTR,
        b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
        75 as ::core::ffi::c_int,
        Affinity,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
            75 as ::core::ffi::c_int,
        );
    }

    /* Connect the interrupt */
    Status = IoConnectInterrupt(
        core::ptr::addr_of_mut!((*DeviceExtension).Interrupt),
        core::mem::transmute(PortFdoInterruptRoutine as unsafe extern "system" fn(PKINTERRUPT, PVOID) -> BOOLEAN),
        DeviceExtension as PVOID,
        core::ptr::null_mut::<ULONG>() as *mut _,
        Vector,
        Irql,
        Irql,
        InterruptMode,
        ShareVector,
        Affinity,
        0 as BOOLEAN,
    );
    if NT_SUCCESS(Status) {
        (*DeviceExtension).InterruptIrql = Irql as ULONG;
    } else {
        if DbgPrint(
            b"(%s:%d) IoConnectInterrupt() failed (Status 0x%08lx)\n\0" as *const u8 as PCSTR,
            b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
            95 as ::core::ffi::c_int,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
                95 as ::core::ffi::c_int,
            );
        }
    }

    return Status;
}
unsafe fn PortFdoStartMiniport(
    DeviceExtension: PFDO_DEVICE_EXTENSION,
) -> NTSTATUS {

    let mut InitData: PHW_INITIALIZATION_DATA;
    let mut InterfaceType: INTERFACE_TYPE;
    let mut Status: NTSTATUS;

    InterfaceType = GetBusInterface((*DeviceExtension).LowerDevice);
    if InterfaceType as i32 == InterfaceTypeUndefined as i32 {
        return STATUS_NO_SUCH_DEVICE;
    }

    InitData = PortGetDriverInitData((*DeviceExtension).DriverExtension, InterfaceType);
    if InitData.is_null() {
        return STATUS_NO_SUCH_DEVICE;
    }

    Status = MiniportInitialize(
        core::ptr::addr_of_mut!((*DeviceExtension).Miniport),
        DeviceExtension,
        InitData,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = MiniportFindAdapter(core::ptr::addr_of_mut!((*DeviceExtension).Miniport));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = PortFdoConnectInterrupt(DeviceExtension);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = MiniportHwInitialize(core::ptr::addr_of_mut!((*DeviceExtension).Miniport));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*DeviceExtension).HwPassiveInitRoutine.is_some() {
        if (*DeviceExtension)
            .HwPassiveInitRoutine
            .expect("non-null function pointer")(
            core::ptr::addr_of_mut!(
                (*(*DeviceExtension).Miniport.MiniportExtension).HwDeviceExtension
            ) as PVOID,
        ) == 0
        {
            return STATUS_UNSUCCESSFUL;
        }
    }

    return STATUS_SUCCESS;
}
unsafe fn PortFdoStartDevice(
    DeviceExtension: PFDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {

    let mut Stack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    // DPRINT1("PortFdoStartDevice(%p %p)\n", DeviceExtension, Irp);

    // ASSERT(DeviceExtension->ExtensionType == FdoExtension);

    /* Get the current stack location */
    Stack = IoGetCurrentIrpStackLocation(Irp);

    /* Start the lower device if the FDO is in 'stopped' state */
    if (*DeviceExtension).PnpState as u32 == dsStopped as u32 {
        if IoForwardIrpSynchronously((*DeviceExtension).LowerDevice, Irp) != 0 {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        } else {
            Status = STATUS_UNSUCCESSFUL;
        }

        if !NT_SUCCESS(Status) {
            // DPRINT1("Lower device failed the IRP (Status 0x%08lx)\n", Status);
            return Status;
        }
    }

    /* Change to the 'started' state */
    (*DeviceExtension).PnpState = dsStarted;

    /* Copy the raw and translated resource lists into the device extension */
    if !(*Stack).Parameters.StartDevice.AllocatedResources.is_null()
        && !(*Stack)
            .Parameters
            .StartDevice
            .AllocatedResourcesTranslated
            .is_null()
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
        1 as USHORT,
        core::ptr::addr_of_mut!((*DeviceExtension).BusInterface) as PINTERFACE,
        core::ptr::null_mut(),
    );
    // DPRINT1("Status: 0x%08lx\n", Status);
    if NT_SUCCESS(Status) {
        // DPRINT1("Context: %p\n", DeviceExtension->BusInterface.Context);
        (*DeviceExtension).BusInitialized = 1 as BOOLEAN;
    }

    /* Start the miniport (FindAdapter & Initialize) */
    Status = PortFdoStartMiniport(DeviceExtension);
    if !NT_SUCCESS(Status) {
        // DPRINT1("FdoStartMiniport() failed (Status 0x%08lx)\n", Status);
        (*DeviceExtension).PnpState = dsStopped;
    }

    return Status;
}
unsafe fn PortSendInquiry(mut PdoExtension: PPDO_DEVICE_EXTENSION) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut IrpStack: PIO_STACK_LOCATION;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irp: PIRP;
    let mut Status: NTSTATUS = 0;
    let mut SenseBuffer: PSENSE_DATA;
    let mut KeepTrying: BOOLEAN = TRUE as BOOLEAN;
    let mut RetryCount: ULONG = 0 as ULONG;
    let mut Srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let mut Cdb: PCDB;

    if (*PdoExtension).InquiryBuffer.is_null() {
        (*PdoExtension).InquiryBuffer = ExAllocatePoolWithTag(
            NonPagedPool,
            INQUIRYDATABUFFERSIZE as SIZE_T,
            TAG_INQUIRY_DATA as ULONG,
        ) as PINQUIRYDATA;
        if (*PdoExtension).InquiryBuffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    SenseBuffer = ExAllocatePoolWithTag(
        NonPagedPool,
        SENSE_BUFFER_SIZE as SIZE_T,
        TAG_SENSE_DATA as ULONG,
    ) as PSENSE_DATA;
    if SenseBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    while KeepTrying != 0 {
        /* Initialize event for waiting */
        KeInitializeEvent(
            core::ptr::addr_of_mut!(Event),
            _EVENT_TYPE_NotificationEvent,
            FALSE as BOOLEAN,
        );

        /* Create an IRP */
        Irp = IoBuildDeviceIoControlRequest(
            IOCTL_SCSI_EXECUTE_IN as ULONG,
            (*PdoExtension).Device,
            core::ptr::null_mut(),
            0 as ULONG,
            (*PdoExtension).InquiryBuffer as PVOID,
            INQUIRYDATABUFFERSIZE as ULONG,
            TRUE as BOOLEAN,
            core::ptr::addr_of_mut!(Event),
            core::ptr::addr_of_mut!(IoStatusBlock),
        );
        if Irp.is_null() {
            /* Quit the loop */
            Status = STATUS_INSUFFICIENT_RESOURCES;
            KeepTrying = FALSE as BOOLEAN;
            continue;
        }

        /* Prepare SRB */
        RtlZeroMemory(
            core::ptr::addr_of_mut!(Srb) as PVOID,
            core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
        );

        Srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
        Srb.OriginalRequest = Irp as PVOID;
        Srb.PathId = (*PdoExtension).Bus as UCHAR;
        Srb.TargetId = (*PdoExtension).Target as UCHAR;
        Srb.Lun = (*PdoExtension).Lun as UCHAR;
        Srb.Function = SRB_FUNCTION_EXECUTE_SCSI as UCHAR;
        Srb.SrbFlags = (SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER) as ULONG;
        Srb.TimeOutValue = 4 as ULONG;
        Srb.CdbLength = 6 as UCHAR;

        Srb.SenseInfoBuffer = SenseBuffer as PVOID;
        Srb.SenseInfoBufferLength = SENSE_BUFFER_SIZE as UCHAR;

        Srb.DataBuffer = (*PdoExtension).InquiryBuffer as PVOID;
        Srb.DataTransferLength = INQUIRYDATABUFFERSIZE as ULONG;

        /* Attach Srb to the Irp */
        IrpStack = IoGetNextIrpStackLocation(Irp);
        (*IrpStack).Parameters.Scsi.Srb = core::ptr::addr_of_mut!(Srb) as *mut _;

        /* Fill in CDB */
        Cdb = core::ptr::addr_of_mut!(Srb.Cdb) as *mut UCHAR as PCDB;
        (*Cdb).CDB6INQUIRY.as_mut().OperationCode = SCSIOP_INQUIRY as UCHAR;
        (*Cdb)
            .CDB6INQUIRY
            .as_mut()
            .set_LogicalUnitNumber((*PdoExtension).Lun as UCHAR);
        (*Cdb).CDB6INQUIRY.as_mut().AllocationLength = INQUIRYDATABUFFERSIZE as UCHAR;

        /* Call the driver */
        Status = IoCallDriver((*PdoExtension).Device, Irp);

        /* Wait for it to complete */
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as PVOID,
                _KWAIT_REASON_Executive,
                0i32 as KPROCESSOR_MODE,
                FALSE as BOOLEAN,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if SRB_STATUS(Srb.SrbStatus) as u32 == SRB_STATUS_SUCCESS {
            /* Quit the loop */
            Status = STATUS_SUCCESS;
            KeepTrying = FALSE as BOOLEAN;
            continue;
        }

        /* Check if the queue is frozen */
        if Srb.SrbStatus as u32 & SRB_STATUS_QUEUE_FROZEN != 0 {
            /* Something weird happened, deal with it (unfreeze the queue) */
            KeepTrying = FALSE as BOOLEAN;
        }

        /* Check if data overrun happened */
        if SRB_STATUS(Srb.SrbStatus) as u32 == SRB_STATUS_DATA_OVERRUN {
            /* Quit the loop */
            Status = STATUS_SUCCESS;
            KeepTrying = FALSE as BOOLEAN;
        } else if (Srb.SrbStatus as u32 & SRB_STATUS_AUTOSENSE_VALID != 0)
            && (*SenseBuffer).SenseKey() as i32 == SCSI_SENSE_ILLEGAL_REQUEST as i32
        {
            /* LUN is not valid, but some device responds there.
                Mark it as invalid anyway */

            /* Quit the loop */
            Status = STATUS_INVALID_DEVICE_REQUEST;
            KeepTrying = FALSE as BOOLEAN;
        } else {
            /* Retry a couple of times if no timeout happened */
            if (RetryCount < 2)
                && (SRB_STATUS(Srb.SrbStatus) as u32 != SRB_STATUS_NO_DEVICE)
                && (SRB_STATUS(Srb.SrbStatus) as u32 != SRB_STATUS_SELECTION_TIMEOUT)
            {
                RetryCount = RetryCount.wrapping_add(1);
                KeepTrying = TRUE as BOOLEAN;
            } else {
                /* That's all, quit the loop */
                KeepTrying = FALSE as BOOLEAN;

                /* Set status according to SRB status */
                if SRB_STATUS(Srb.SrbStatus) as u32 == SRB_STATUS_BAD_FUNCTION
                    || SRB_STATUS(Srb.SrbStatus) as u32 == SRB_STATUS_BAD_SRB_BLOCK_LENGTH
                {
                    Status = STATUS_INVALID_DEVICE_REQUEST;
                } else {
                    Status = STATUS_IO_DEVICE_ERROR;
                }
            }
        }
    }

    /* Free the sense buffer */
    ExFreePoolWithTag(SenseBuffer as PVOID, TAG_SENSE_DATA as ULONG);

    return Status;
}

unsafe fn PortFdoScanBus(mut DeviceExtension: PFDO_DEVICE_EXTENSION) -> NTSTATUS {
    let mut PdoExtension: PPDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Bus: ULONG;
    let mut Target: ULONG;
    let mut Status: NTSTATUS;

    Bus = 0 as ULONG;
    while Bus < (*DeviceExtension).Miniport.PortConfig.NumberOfBuses as ULONG {
        Target = 0 as ULONG;
        while Target < (*DeviceExtension).Miniport.PortConfig.MaximumNumberOfTargets as ULONG {
            Status = PortCreatePdo(DeviceExtension, Bus, Target, 0 as ULONG, core::ptr::addr_of_mut!(PdoExtension));
            if NT_SUCCESS(Status) {
                Status = PortSendInquiry(PdoExtension);
                if !NT_SUCCESS(Status) {
                    PortDeletePdo(PdoExtension);
                } else {
                }
            }
            Target = Target.wrapping_add(1);
        }
        Bus = Bus.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
unsafe fn PortFdoQueryBusRelations(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut Information: PULONG_PTR,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    if DbgPrint(
        b"(%s:%d) PortFdoQueryBusRelations(%p %p)\n\0" as *const u8 as PCSTR,
        b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
        532 as ::core::ffi::c_int,
        DeviceExtension,
        Information,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
            532 as ::core::ffi::c_int,
        );
    }
    Status = PortFdoScanBus(DeviceExtension);
    if DbgPrint(
        b"(%s:%d) Units found: %lu\n\0" as *const u8 as PCSTR,
        b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
        536 as ::core::ffi::c_int,
        (*DeviceExtension).PdoCount,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"fdo.c\0" as *const u8 as *const ::core::ffi::c_char,
            536 as ::core::ffi::c_int,
        );
    }
    *Information = 0 as ::core::ffi::c_ulonglong as ULONG_PTR;
    return Status;
}
unsafe fn PortFdoFilterRequirements(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut RequirementsList: PIO_RESOURCE_REQUIREMENTS_LIST = core::ptr::null_mut();
    if DbgPrint(
        b"(%s:%d) PortFdoFilterRequirements(%p %p)\n\0" as *const u8 as PCSTR,
        b"drivers/storage/port/storport/fdo.c\0"
            as *const u8 as *const core::ffi::c_char,
        552 as core::ffi::c_int,
        DeviceExtension,
        Irp,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/storage/port/storport/fdo.c\0"
                as *const u8 as *const core::ffi::c_char,
            552 as core::ffi::c_int,
        );
    }
    RequirementsList = (*Irp).IoStatus.Information as PIO_RESOURCE_REQUIREMENTS_LIST;
    if !RequirementsList.is_null() {
        (*DeviceExtension).BusNumber = (*RequirementsList).BusNumber;
        (*DeviceExtension).SlotNumber = (*RequirementsList).SlotNumber;
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn PortFdoScsi(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut DeviceExtension: PFDO_DEVICE_EXTENSION =
        core::ptr::null_mut::<_FDO_DEVICE_EXTENSION>();
    let Information: ULONG_PTR = 0 as ULONG_PTR;
    let Status: NTSTATUS = STATUS_NOT_SUPPORTED;

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if DeviceExtension.is_null() {
        RtlAssert(
            b"DeviceExtension\0".as_ptr() as *const i8 as PVOID,
            b"drivers/storage/port/storport/fdo.c\0".as_ptr() as *const i8 as PVOID,
            580 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if !((*DeviceExtension).ExtensionType as core::ffi::c_uint
        == EXTENSION_TYPE_FdoExtension as core::ffi::c_int as core::ffi::c_uint)
    {
        RtlAssert(
            b"DeviceExtension->ExtensionType == FdoExtension\0".as_ptr() as *const i8 as PVOID,
            b"drivers/storage/port/storport/fdo.c\0".as_ptr() as *const i8 as PVOID,
            581 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    (*Irp).IoStatus.Information = Information as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn PortFdoPnp(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {

    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Stack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Information: ULONG_PTR = 0 as ULONG_PTR;
    let mut Status: NTSTATUS = STATUS_NOT_SUPPORTED;

    DbgPrint(
        b"PortFdoPnp(%p %p)\n\0".as_ptr() as *const i8,
        DeviceObject,
        Irp,
    );

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    if DeviceExtension.is_null() {
        RtlAssert(
            b"DeviceExtension\0".as_ptr() as *const i8 as PVOID,
            b"fdo.c\0".as_ptr() as *const i8 as PVOID,
            609 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if !((*DeviceExtension).ExtensionType as u32 == EXTENSION_TYPE_FdoExtension as i32 as u32) {
        RtlAssert(
            b"DeviceExtension->ExtensionType == FdoExtension\0".as_ptr() as *const i8 as PVOID,
            b"fdo.c\0".as_ptr() as *const i8 as PVOID,
            610 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    Stack = IoGetCurrentIrpStackLocation(Irp);

    match (*Stack).MinorFunction as u8 {
        IRP_MN_START_DEVICE => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_START_DEVICE\n\0".as_ptr() as *const i8);
            Status = PortFdoStartDevice(DeviceExtension, Irp);
        }
        IRP_MN_QUERY_REMOVE_DEVICE => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_QUERY_REMOVE_DEVICE\n\0".as_ptr() as *const i8);
        }
        IRP_MN_REMOVE_DEVICE => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_REMOVE_DEVICE\n\0".as_ptr() as *const i8);
        }
        IRP_MN_CANCEL_REMOVE_DEVICE => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_CANCEL_REMOVE_DEVICE\n\0".as_ptr() as *const i8);
        }
        IRP_MN_STOP_DEVICE => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_STOP_DEVICE\n\0".as_ptr() as *const i8);
        }
        IRP_MN_QUERY_STOP_DEVICE => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_QUERY_STOP_DEVICE\n\0".as_ptr() as *const i8);
        }
        IRP_MN_CANCEL_STOP_DEVICE => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_CANCEL_STOP_DEVICE\n\0".as_ptr() as *const i8);
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS\n\0".as_ptr() as *const i8);
            match (*Stack).Parameters.QueryDeviceRelations.Type as u32 {
                0 => {
                    DbgPrint(
                        b"    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / BusRelations\n\0".as_ptr()
                            as *const i8,
                    );
                    Status = PortFdoQueryBusRelations(DeviceExtension, core::ptr::addr_of_mut!(Information));
                }
                3 => {
                    DbgPrint(
                        b"    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / RemovalRelations\n\0"
                            .as_ptr() as *const i8,
                    );
                    return ForwardIrpAndForget((*DeviceExtension).LowerDevice, Irp);
                }
                _ => {
                    DbgPrint(
                        b"    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / Unknown type 0x%lx\n\0"
                            .as_ptr() as *const i8,
                        (*Stack).Parameters.QueryDeviceRelations.Type as u32,
                    );
                    return ForwardIrpAndForget((*DeviceExtension).LowerDevice, Irp);
                }
            }
        }
        IRP_MN_FILTER_RESOURCE_REQUIREMENTS => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n\0".as_ptr() as *const i8);
            PortFdoFilterRequirements(DeviceExtension, Irp);
            return ForwardIrpAndForget((*DeviceExtension).LowerDevice, Irp);
        }
        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_QUERY_PNP_DEVICE_STATE\n\0".as_ptr() as *const i8);
        }
        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_DEVICE_USAGE_NOTIFICATION\n\0".as_ptr() as *const i8);
        }
        IRP_MN_SURPRISE_REMOVAL => {
            DbgPrint(b"IRP_MJ_PNP / IRP_MN_SURPRISE_REMOVAL\n\0".as_ptr() as *const i8);
        }
        _ => {
            DbgPrint(
                b"IRP_MJ_PNP / Unknown IOCTL 0x%lx\n\0".as_ptr() as *const i8,
                (*Stack).MinorFunction as i32,
            );
            return ForwardIrpAndForget((*DeviceExtension).LowerDevice, Irp);
        }
    }

    (*Irp).IoStatus.Information = Information as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Storport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Storport FDO code
//  * COPYRIGHT:   Copyright 2017 Eric Kohl (eric.kohl@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* FUNCTIONS ******************************************************************/
//
// static
// BOOLEAN
// NTAPI
// PortFdoInterruptRoutine(
//      PKINTERRUPT Interrupt,
//      PVOID ServiceContext)
// {
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//
//     DPRINT1("PortFdoInterruptRoutine(%p %p)\n",
//             Interrupt, ServiceContext);
//
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)ServiceContext;
//
//     return MiniportHwInterrupt(&DeviceExtension->Miniport);
// }
//
//
// static
// NTSTATUS
// PortFdoConnectInterrupt(
//      PFDO_DEVICE_EXTENSION DeviceExtension)
// {
//     ULONG Vector;
//     KIRQL Irql;
//     KINTERRUPT_MODE InterruptMode;
//     BOOLEAN ShareVector;
//     KAFFINITY Affinity;
//     NTSTATUS Status;
//
//     DPRINT1("PortFdoConnectInterrupt(%p)\n",
//             DeviceExtension);
//
//     /* No resources, no interrupt. Done! */
//     if (DeviceExtension->AllocatedResources == NULL ||
//         DeviceExtension->TranslatedResources == NULL)
//     {
//         DPRINT1("Checkpoint\n");
//         return STATUS_SUCCESS;
//     }
//
//     /* Get the interrupt data from the resource list */
//     Status = GetResourceListInterrupt(DeviceExtension,
//                                       &Vector,
//                                       &Irql,
//                                       &InterruptMode,
//                                       &ShareVector,
//                                       &Affinity);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("GetResourceListInterrupt() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     DPRINT1("Vector: %lu\n", Vector);
//     DPRINT1("Irql: %lu\n", Irql);
//
//     DPRINT1("Affinity: 0x%08lx\n", Affinity);
//
//     /* Connect the interrupt */
//     Status = IoConnectInterrupt(&DeviceExtension->Interrupt,
//                                 PortFdoInterruptRoutine,
//                                 DeviceExtension,
//                                 NULL,
//                                 Vector,
//                                 Irql,
//                                 Irql,
//                                 InterruptMode,
//                                 ShareVector,
//                                 Affinity,
//                                 FALSE);
//     if (NT_SUCCESS(Status))
//     {
//         DeviceExtension->InterruptIrql = Irql;
//     }
//     else
//     {
//         DPRINT1("IoConnectInterrupt() failed (Status 0x%08lx)\n", Status);
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// PortFdoStartMiniport(
//      PFDO_DEVICE_EXTENSION DeviceExtension)
// {
//     PHW_INITIALIZATION_DATA InitData;
//     INTERFACE_TYPE InterfaceType;
//     NTSTATUS Status;
//
//     DPRINT1("PortFdoStartDevice(%p)\n", DeviceExtension);
//
//     /* Get the interface type of the lower device */
//     InterfaceType = GetBusInterface(DeviceExtension->LowerDevice);
//     if (InterfaceType == InterfaceTypeUndefined)
//         return STATUS_NO_SUCH_DEVICE;
//
//     /* Get the driver init data for the given interface type */
//     InitData = PortGetDriverInitData(DeviceExtension->DriverExtension,
//                                      InterfaceType);
//     if (InitData == NULL)
//         return STATUS_NO_SUCH_DEVICE;
//
//     /* Initialize the miniport */
//     Status = MiniportInitialize(&DeviceExtension->Miniport,
//                                 DeviceExtension,
//                                 InitData);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("MiniportInitialize() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Call the miniports FindAdapter function */
//     Status = MiniportFindAdapter(&DeviceExtension->Miniport);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("MiniportFindAdapter() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Connect the configured interrupt */
//     Status = PortFdoConnectInterrupt(DeviceExtension);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("PortFdoConnectInterrupt() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Call the miniports HwInitialize function */
//     Status = MiniportHwInitialize(&DeviceExtension->Miniport);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("MiniportHwInitialize() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     /* Call the HwPassiveInitRoutine function, if available */
//     if (DeviceExtension->HwPassiveInitRoutine != NULL)
//     {
//         DPRINT1("Calling HwPassiveInitRoutine()\n");
//         if (!DeviceExtension->HwPassiveInitRoutine(&DeviceExtension->Miniport.MiniportExtension->HwDeviceExtension))
//         {
//             DPRINT1("HwPassiveInitRoutine() failed\n");
//             return STATUS_UNSUCCESSFUL;
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// NTAPI
// PortFdoStartDevice(
//      PFDO_DEVICE_EXTENSION DeviceExtension,
//      PIRP Irp)
// {
//     PIO_STACK_LOCATION Stack;
//     NTSTATUS Status;
//
//     DPRINT1("PortFdoStartDevice(%p %p)\n",
//             DeviceExtension, Irp);
//
//     ASSERT(DeviceExtension->ExtensionType == FdoExtension);
//
//     /* Get the current stack location */
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* Start the lower device if the FDO is in 'stopped' state */
//     if (DeviceExtension->PnpState == dsStopped)
//     {
//         if (IoForwardIrpSynchronously(DeviceExtension->LowerDevice, Irp))
//         {
//             Status = Irp->IoStatus.Status;
//         }
//         else
//         {
//             Status = STATUS_UNSUCCESSFUL;
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Lower device failed the IRP (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//     }
//
//     /* Change to the 'started' state */
//     DeviceExtension->PnpState = dsStarted;
//
//     /* Copy the raw and translated resource lists into the device extension */
//     if (Stack->Parameters.StartDevice.AllocatedResources != NULL &&
//         Stack->Parameters.StartDevice.AllocatedResourcesTranslated != NULL)
//     {
//         DeviceExtension->AllocatedResources = CopyResourceList(NonPagedPool,
//                                                                Stack->Parameters.StartDevice.AllocatedResources);
//         if (DeviceExtension->AllocatedResources == NULL)
//             return STATUS_NO_MEMORY;
//
//         DeviceExtension->TranslatedResources = CopyResourceList(NonPagedPool,
//                                                                 Stack->Parameters.StartDevice.AllocatedResourcesTranslated);
//         if (DeviceExtension->TranslatedResources == NULL)
//             return STATUS_NO_MEMORY;
//     }
//
//     /* Get the bus interface of the lower (bus) device */
//     Status = QueryBusInterface(DeviceExtension->LowerDevice,
//                                (PGUID)&GUID_BUS_INTERFACE_STANDARD,
//                                sizeof(BUS_INTERFACE_STANDARD),
//                                1,
//                                &DeviceExtension->BusInterface,
//                                NULL);
//     DPRINT1("Status: 0x%08lx\n", Status);
//     if (NT_SUCCESS(Status))
//     {
//         DPRINT1("Context: %p\n", DeviceExtension->BusInterface.Context);
//         DeviceExtension->BusInitialized = TRUE;
//     }
//
//     /* Start the miniport (FindAdapter & Initialize) */
//     Status = PortFdoStartMiniport(DeviceExtension);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("FdoStartMiniport() failed (Status 0x%08lx)\n", Status);
//         DeviceExtension->PnpState = dsStopped;
//     }
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// PortSendInquiry(
//      PPDO_DEVICE_EXTENSION PdoExtension)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIO_STACK_LOCATION IrpStack;
//     KEVENT Event;
// //    KIRQL Irql;
//     PIRP Irp;
//     NTSTATUS Status;
//     PSENSE_DATA SenseBuffer;
//     BOOLEAN KeepTrying = TRUE;
//     ULONG RetryCount = 0;
//     SCSI_REQUEST_BLOCK Srb;
//     PCDB Cdb;
// //    PSCSI_PORT_LUN_EXTENSION LunExtension;
// //    PFDO_DEVICE_EXTENSION DeviceExtension;
//
//     DPRINT("PortSendInquiry(%p)\n", PdoExtension);
//
//     if (PdoExtension->InquiryBuffer == NULL)
//     {
//         PdoExtension->InquiryBuffer = ExAllocatePoolWithTag(NonPagedPool, INQUIRYDATABUFFERSIZE, TAG_INQUIRY_DATA);
//         if (PdoExtension->InquiryBuffer == NULL)
//             return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     SenseBuffer = ExAllocatePoolWithTag(NonPagedPool, SENSE_BUFFER_SIZE, TAG_SENSE_DATA);
//     if (SenseBuffer == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     while (KeepTrying)
//     {
//         /* Initialize event for waiting */
//         KeInitializeEvent(&Event,
//                           NotificationEvent,
//                           FALSE);
//
//         /* Create an IRP */
//         Irp = IoBuildDeviceIoControlRequest(IOCTL_SCSI_EXECUTE_IN,
//                                             PdoExtension->Device,
//                                             NULL,
//                                             0,
//                                             PdoExtension->InquiryBuffer,
//                                             INQUIRYDATABUFFERSIZE,
//                                             TRUE,
//                                             &Event,
//                                             &IoStatusBlock);
//         if (Irp == NULL)
//         {
//             DPRINT("IoBuildDeviceIoControlRequest() failed\n");
//
//             /* Quit the loop */
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             KeepTrying = FALSE;
//             continue;
//         }
//
//         /* Prepare SRB */
//         RtlZeroMemory(&Srb, sizeof(SCSI_REQUEST_BLOCK));
//
//         Srb.Length = sizeof(SCSI_REQUEST_BLOCK);
//         Srb.OriginalRequest = Irp;
//         Srb.PathId = PdoExtension->Bus;
//         Srb.TargetId = PdoExtension->Target;
//         Srb.Lun = PdoExtension->Lun;
//         Srb.Function = SRB_FUNCTION_EXECUTE_SCSI;
//         Srb.SrbFlags = SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
//         Srb.TimeOutValue = 4;
//         Srb.CdbLength = 6;
//
//         Srb.SenseInfoBuffer = SenseBuffer;
//         Srb.SenseInfoBufferLength = SENSE_BUFFER_SIZE;
//
//         Srb.DataBuffer = PdoExtension->InquiryBuffer;
//         Srb.DataTransferLength = INQUIRYDATABUFFERSIZE;
//
//         /* Attach Srb to the Irp */
//         IrpStack = IoGetNextIrpStackLocation(Irp);
//         IrpStack->Parameters.Scsi.Srb = &Srb;
//
//         /* Fill in CDB */
//         Cdb = (PCDB)Srb.Cdb;
//         Cdb->CDB6INQUIRY.OperationCode = SCSIOP_INQUIRY;
//         Cdb->CDB6INQUIRY.LogicalUnitNumber = PdoExtension->Lun;
//         Cdb->CDB6INQUIRY.AllocationLength = INQUIRYDATABUFFERSIZE;
//
//         /* Call the driver */
//         Status = IoCallDriver(PdoExtension->Device, Irp);
//
//         /* Wait for it to complete */
//         if (Status == STATUS_PENDING)
//         {
//             DPRINT1("PortSendInquiry(): Waiting for the driver to process request...\n");
//             KeWaitForSingleObject(&Event,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//             Status = IoStatusBlock.Status;
//         }
//
//         DPRINT("PortSendInquiry(): Request processed by driver, status = 0x%08X\n", Status);
//
//         if (SRB_STATUS(Srb.SrbStatus) == SRB_STATUS_SUCCESS)
//         {
//             DPRINT("Found a device!\n");
//
//             /* Quit the loop */
//             Status = STATUS_SUCCESS;
//             KeepTrying = FALSE;
//             continue;
//         }
//
//         DPRINT("Inquiry SRB failed with SrbStatus 0x%08X\n", Srb.SrbStatus);
//
//         /* Check if the queue is frozen */
//         if (Srb.SrbStatus & SRB_STATUS_QUEUE_FROZEN)
//         {
//             /* Something weird happened, deal with it (unfreeze the queue) */
//             KeepTrying = FALSE;
//
//             DPRINT("SpiSendInquiry(): the queue is frozen at TargetId %d\n", Srb.TargetId);
//
// //            LunExtension = SpiGetLunExtension(DeviceExtension,
// //                                              LunInfo->PathId,
// //                                              LunInfo->TargetId,
// //                                              LunInfo->Lun);
//
//             /* Clear frozen flag */
// //            LunExtension->Flags &= ~LUNEX_FROZEN_QUEUE;
//
//             /* Acquire the spinlock */
// //            KeAcquireSpinLock(&DeviceExtension->SpinLock, &Irql);
//
//             /* Process the request */
// //            SpiGetNextRequestFromLun(DeviceObject->DeviceExtension, LunExtension);
//
//             /* SpiGetNextRequestFromLun() releases the spinlock,
//                 so we just lower irql back to what it was before */
// //            KeLowerIrql(Irql);
//         }
//
//         /* Check if data overrun happened */
//         if (SRB_STATUS(Srb.SrbStatus) == SRB_STATUS_DATA_OVERRUN)
//         {
//             DPRINT("Data overrun at TargetId %d\n", PdoExtension->Target);
//
//             /* Quit the loop */
//             Status = STATUS_SUCCESS;
//             KeepTrying = FALSE;
//         }
//         else if ((Srb.SrbStatus & SRB_STATUS_AUTOSENSE_VALID) &&
//                  SenseBuffer->SenseKey == SCSI_SENSE_ILLEGAL_REQUEST)
//         {
//             /* LUN is not valid, but some device responds there.
//                 Mark it as invalid anyway */
//
//             /* Quit the loop */
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//             KeepTrying = FALSE;
//         }
//         else
//         {
//             /* Retry a couple of times if no timeout happened */
//             if ((RetryCount < 2) &&
//                 (SRB_STATUS(Srb.SrbStatus) != SRB_STATUS_NO_DEVICE) &&
//                 (SRB_STATUS(Srb.SrbStatus) != SRB_STATUS_SELECTION_TIMEOUT))
//             {
//                 RetryCount++;
//                 KeepTrying = TRUE;
//             }
//             else
//             {
//                 /* That's all, quit the loop */
//                 KeepTrying = FALSE;
//
//                 /* Set status according to SRB status */
//                 if (SRB_STATUS(Srb.SrbStatus) == SRB_STATUS_BAD_FUNCTION ||
//                     SRB_STATUS(Srb.SrbStatus) == SRB_STATUS_BAD_SRB_BLOCK_LENGTH)
//                 {
//                     Status = STATUS_INVALID_DEVICE_REQUEST;
//                 }
//                 else
//                 {
//                     Status = STATUS_IO_DEVICE_ERROR;
//                 }
//             }
//         }
//     }
//
//     /* Free the sense buffer */
//     ExFreePoolWithTag(SenseBuffer, TAG_SENSE_DATA);
//
//     DPRINT("PortSendInquiry() done with Status 0x%08X\n", Status);
//
//     return Status;
// }
//
//
//
// static
// NTSTATUS
// PortFdoScanBus(
//      PFDO_DEVICE_EXTENSION DeviceExtension)
// {
//     PPDO_DEVICE_EXTENSION PdoExtension;
//     ULONG Bus, Target; //, Lun;
//     NTSTATUS Status;
//
//     DPRINT("PortFdoScanBus(%p)\n", DeviceExtension);
//
//     DPRINT("NumberOfBuses: %lu\n", DeviceExtension->Miniport.PortConfig.NumberOfBuses);
//     DPRINT("MaximumNumberOfTargets: %lu\n", DeviceExtension->Miniport.PortConfig.MaximumNumberOfTargets);
//     DPRINT("MaximumNumberOfLogicalUnits: %lu\n", DeviceExtension->Miniport.PortConfig.MaximumNumberOfLogicalUnits);
//
//     /* Scan all buses */
//     for (Bus = 0; Bus < DeviceExtension->Miniport.PortConfig.NumberOfBuses; Bus++)
//     {
//         DPRINT("Scanning bus %ld\n", Bus);
//
//         /* Scan all targets */
//         for (Target = 0; Target < DeviceExtension->Miniport.PortConfig.MaximumNumberOfTargets; Target++)
//         {
//             DPRINT("  Scanning target %ld:%ld\n", Bus, Target);
//
//             DPRINT("    Scanning logical unit %ld:%ld:%ld\n", Bus, Target, 0);
//             Status = PortCreatePdo(DeviceExtension, Bus, Target, 0, &PdoExtension);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Scan LUN 0 */
//                 Status = PortSendInquiry(PdoExtension);
//                 DPRINT("PortSendInquiry returned 0x%08lx\n", Status);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     PortDeletePdo(PdoExtension);
//                 }
//                 else
//                 {
//                     DPRINT("VendorId: %.8s\n", PdoExtension->InquiryBuffer->VendorId);
//                     DPRINT("ProductId: %.16s\n", PdoExtension->InquiryBuffer->ProductId);
//                     DPRINT("ProductRevisionLevel: %.4s\n", PdoExtension->InquiryBuffer->ProductRevisionLevel);
//                     DPRINT("VendorSpecific: %.20s\n", PdoExtension->InquiryBuffer->VendorSpecific);
//                 }
//             }
//
//         }
//     }
//
//     DPRINT("PortFdoScanBus() done!\n");
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// PortFdoQueryBusRelations(
//      PFDO_DEVICE_EXTENSION DeviceExtension,
//      PULONG_PTR Information)
// {
//     NTSTATUS Status = STATUS_SUCCESS;;
//
//     DPRINT1("PortFdoQueryBusRelations(%p %p)\n",
//             DeviceExtension, Information);
//
//     Status = PortFdoScanBus(DeviceExtension);
//
//     DPRINT1("Units found: %lu\n", DeviceExtension->PdoCount);
//
//     *Information = 0;
//
//     return Status;
// }
//
//
// static
// NTSTATUS
// PortFdoFilterRequirements(
//     PFDO_DEVICE_EXTENSION DeviceExtension,
//     PIRP Irp)
// {
//     PIO_RESOURCE_REQUIREMENTS_LIST RequirementsList;
//
//     DPRINT1("PortFdoFilterRequirements(%p %p)\n", DeviceExtension, Irp);
//
//     /* Get the bus number and the slot number */
//     RequirementsList =(PIO_RESOURCE_REQUIREMENTS_LIST)Irp->IoStatus.Information;
//     if (RequirementsList != NULL)
//     {
//         DeviceExtension->BusNumber = RequirementsList->BusNumber;
//         DeviceExtension->SlotNumber = RequirementsList->SlotNumber;
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// PortFdoScsi(
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp)
// {
//     PFDO_DEVICE_EXTENSION DeviceExtension;
// //    PIO_STACK_LOCATION Stack;
//     ULONG_PTR Information = 0;
//     NTSTATUS Status = STATUS_NOT_SUPPORTED;
//
//     DPRINT("PortFdoScsi(%p %p)\n", DeviceObject, Irp);
//
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(DeviceExtension);
//     ASSERT(DeviceExtension->ExtensionType == FdoExtension);
//
// //    Stack = IoGetCurrentIrpStackLocation(Irp);
//
//
//     Irp->IoStatus.Information = Information;
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// PortFdoPnp(
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp)
// {
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//     PIO_STACK_LOCATION Stack;
//     ULONG_PTR Information = 0;
//     NTSTATUS Status = STATUS_NOT_SUPPORTED;
//
//     DPRINT1("PortFdoPnp(%p %p)\n",
//             DeviceObject, Irp);
//
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     ASSERT(DeviceExtension);
//     ASSERT(DeviceExtension->ExtensionType == FdoExtension);
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//
//     switch (Stack->MinorFunction)
//     {
//         case IRP_MN_START_DEVICE: /* 0x00 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_START_DEVICE\n");
//             Status = PortFdoStartDevice(DeviceExtension, Irp);
//             break;
//
//         case IRP_MN_QUERY_REMOVE_DEVICE: /* 0x01 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_QUERY_REMOVE_DEVICE\n");
//             break;
//
//         case IRP_MN_REMOVE_DEVICE: /* 0x02 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_REMOVE_DEVICE\n");
//             break;
//
//         case IRP_MN_CANCEL_REMOVE_DEVICE: /* 0x03 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_CANCEL_REMOVE_DEVICE\n");
//             break;
//
//         case IRP_MN_STOP_DEVICE: /* 0x04 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_STOP_DEVICE\n");
//             break;
//
//         case IRP_MN_QUERY_STOP_DEVICE: /* 0x05 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_QUERY_STOP_DEVICE\n");
//             break;
//
//         case IRP_MN_CANCEL_STOP_DEVICE: /* 0x06 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_CANCEL_STOP_DEVICE\n");
//             break;
//
//         case IRP_MN_QUERY_DEVICE_RELATIONS: /* 0x07 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS\n");
//             switch (Stack->Parameters.QueryDeviceRelations.Type)
//             {
//                 case BusRelations:
//                     DPRINT1("    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / BusRelations\n");
//                     Status = PortFdoQueryBusRelations(DeviceExtension, &Information);
//                     break;
//
//                 case RemovalRelations:
//                     DPRINT1("    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / RemovalRelations\n");
//                     return ForwardIrpAndForget(DeviceExtension->LowerDevice, Irp);
//
//                 default:
//                     DPRINT1("    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / Unknown type 0x%lx\n",
//                             Stack->Parameters.QueryDeviceRelations.Type);
//                     return ForwardIrpAndForget(DeviceExtension->LowerDevice, Irp);
//             }
//             break;
//
//         case IRP_MN_FILTER_RESOURCE_REQUIREMENTS: /* 0x0d */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n");
//             PortFdoFilterRequirements(DeviceExtension, Irp);
//             return ForwardIrpAndForget(DeviceExtension->LowerDevice, Irp);
//
//         case IRP_MN_QUERY_PNP_DEVICE_STATE: /* 0x14 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_QUERY_PNP_DEVICE_STATE\n");
//             break;
//
//         case IRP_MN_DEVICE_USAGE_NOTIFICATION: /* 0x16 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_DEVICE_USAGE_NOTIFICATION\n");
//             break;
//
//         case IRP_MN_SURPRISE_REMOVAL: /* 0x17 */
//             DPRINT1("IRP_MJ_PNP / IRP_MN_SURPRISE_REMOVAL\n");
//             break;
//
//         default:
//             DPRINT1("IRP_MJ_PNP / Unknown IOCTL 0x%lx\n", Stack->MinorFunction);
//             return ForwardIrpAndForget(DeviceExtension->LowerDevice, Irp);
//     }
//
//     Irp->IoStatus.Information = Information;
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
