use crate::prelude::*;
use ntkrnl_sys::ddk::{IoGetConfigurationInformation, PCM_FULL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_LIST, PCM_RESOURCE_LIST, PCONFIGURATION_INFORMATION};
use core::ffi::c_void;

unsafe extern "C" fn InitializeConfiguration(
    PortConfig: PPORT_CONFIGURATION_INFORMATION,
    InitData: PHW_INITIALIZATION_DATA,
    BusNumber: ULONG,
    SlotNumber: ULONG,
) -> NTSTATUS {
    let mut ConfigInfo: PCONFIGURATION_INFORMATION = core::ptr::null_mut();
    let mut i: ULONG;

    DPRINT1!(
        "InitializeConfiguration(%p %p %lu %lu)\n",
        PortConfig,
        InitData,
        BusNumber,
        SlotNumber
    );

    /* Get the configurration information */
    ConfigInfo = IoGetConfigurationInformation();

    /* Initialize the port configuration */
    RtlZeroMemory!(
        PortConfig as *mut core::ffi::c_void,
        core::mem::size_of::<PORT_CONFIGURATION_INFORMATION>(),
    );

    (*PortConfig).Length = core::mem::size_of::<PORT_CONFIGURATION_INFORMATION>() as ULONG;
    (*PortConfig).SystemIoBusNumber = BusNumber;
    (*PortConfig).SlotNumber = SlotNumber;
    (*PortConfig).AdapterInterfaceType = (*InitData).AdapterInterfaceType;

    (*PortConfig).MaximumTransferLength = SP_UNINITIALIZED_VALUE;
    (*PortConfig).DmaChannel = SP_UNINITIALIZED_VALUE;
    (*PortConfig).DmaPort = SP_UNINITIALIZED_VALUE;

    (*PortConfig).InterruptMode = LevelSensitive;

    (*PortConfig).Master = TRUE;
    (*PortConfig).AtdiskPrimaryClaimed = (*ConfigInfo).AtDiskPrimaryAddressClaimed;
    (*PortConfig).AtdiskSecondaryClaimed = (*ConfigInfo).AtDiskSecondaryAddressClaimed;
    (*PortConfig).Dma32BitAddresses = TRUE;
    (*PortConfig).DemandMode = FALSE;
    (*PortConfig).MapBuffers = (*InitData).MapBuffers;

    (*PortConfig).NeedPhysicalAddresses = TRUE;
    (*PortConfig).TaggedQueuing = TRUE;
    (*PortConfig).AutoRequestSense = TRUE;
    (*PortConfig).MultipleRequestPerLu = TRUE;
    (*PortConfig).ReceiveEvent = (*InitData).ReceiveEvent;
    (*PortConfig).RealModeInitialized = FALSE;
    (*PortConfig).BufferAccessScsiPortControlled = TRUE;
    (*PortConfig).MaximumNumberOfTargets = (SCSI_MAXIMUM_TARGETS_PER_BUS as u8);

    (*PortConfig).SpecificLuExtensionSize = (*InitData).SpecificLuExtensionSize;
    (*PortConfig).SrbExtensionSize = (*InitData).SrbExtensionSize;
    (*PortConfig).MaximumNumberOfLogicalUnits = (SCSI_MAXIMUM_LOGICAL_UNITS as u8);
    (*PortConfig).WmiDataProvider = TRUE;

    (*PortConfig).NumberOfAccessRanges = (*InitData).NumberOfAccessRanges;
    DPRINT1!(
        "NumberOfAccessRanges: %lu\n",
        (*PortConfig).NumberOfAccessRanges
    );
    if (*PortConfig).NumberOfAccessRanges != 0 {
        (*PortConfig).AccessRanges = ExAllocatePoolWithTag(
            NonPagedPool,
            ((*PortConfig).NumberOfAccessRanges as SIZE_T)
                * (core::mem::size_of::<ACCESS_RANGE>() as SIZE_T),
            TAG_ACCRESS_RANGE,
        ) as _;
        if (*PortConfig).AccessRanges.is_null() {
            return STATUS_NO_MEMORY;
        }

        RtlZeroMemory!(
            (*PortConfig).AccessRanges as *mut core::ffi::c_void,
            ((*PortConfig).NumberOfAccessRanges as SIZE_T)
                * (core::mem::size_of::<ACCESS_RANGE>() as SIZE_T),
        );
    }

    i = 0;
    while i < RTL_NUMBER_OF!((*PortConfig).InitiatorBusId) {
        (*PortConfig).InitiatorBusId[i as usize] = SP_UNINITIALIZED_VALUE as CCHAR;
        i = i.wrapping_add(1);
    }

    /* Default per-adapter outstanding IO limit, as per storport documentation. */
    (*PortConfig).MaxNumberOfIO = 1000;

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AssignResourcesToConfiguration(
    mut PortConfiguration: PPORT_CONFIGURATION_INFORMATION,
    mut ResourceList: PCM_RESOURCE_LIST,
    mut NumberOfAccessRanges: ULONG,
) {
    let mut FullDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut PartialResourceList: PCM_PARTIAL_RESOURCE_LIST = core::ptr::null_mut();
    let mut AccessRange: PACCESS_RANGE = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut j: INT = 0;
    let mut RangeNumber: ULONG = 0;
    let mut Interrupt: ULONG = 0;
    let mut Dma: ULONG = 0;

    DPRINT1!(
        "AssignResourceToConfiguration(%p %p %lu)\n",
        PortConfiguration,
        ResourceList,
        NumberOfAccessRanges
    );

    FullDescriptor = (&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
        .offset(0 as isize) as PCM_FULL_RESOURCE_DESCRIPTOR;
    i = 0 as INT;
    while (i as ULONG) < (*ResourceList).Count {
        PartialResourceList =
            &raw mut (*FullDescriptor).PartialResourceList as PCM_PARTIAL_RESOURCE_LIST;

        j = 0 as INT;
        while (j as ULONG) < (*PartialResourceList).Count {
            PartialDescriptor = (&raw mut (*PartialResourceList).PartialDescriptors
                as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(j as isize) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

            match (*PartialDescriptor).Type as ::core::ffi::c_int {
                __c if __c == CmResourceTypePort as _ => {
                    DPRINT1!(
                        "Port: 0x%I64x (0x%lx)\n",
                        (*PartialDescriptor).u.Port.Start.QuadPart,
                        (*PartialDescriptor).u.Port.Length
                    );
                    if RangeNumber < NumberOfAccessRanges {
                        AccessRange = (&raw mut *(*PortConfiguration).AccessRanges
                            as *mut ACCESS_RANGE)
                            .offset(RangeNumber as isize) as PACCESS_RANGE;
                        (*AccessRange).RangeStart = (*PartialDescriptor).u.Port.Start;
                        (*AccessRange).RangeLength = (*PartialDescriptor).u.Port.Length;
                        (*AccessRange).RangeInMemory = FALSE;
                        RangeNumber = RangeNumber.wrapping_add(1);
                    }
                }

                __c if __c == CmResourceTypeMemory as _ => {
                    DPRINT1!(
                        "Memory: 0x%I64x (0x%lx)\n",
                        (*PartialDescriptor).u.Memory.Start.QuadPart,
                        (*PartialDescriptor).u.Memory.Length
                    );
                    if RangeNumber < NumberOfAccessRanges {
                        AccessRange = (&raw mut *(*PortConfiguration).AccessRanges
                            as *mut ACCESS_RANGE)
                            .offset(RangeNumber as isize) as PACCESS_RANGE;
                        (*AccessRange).RangeStart = (*PartialDescriptor).u.Memory.Start;
                        (*AccessRange).RangeLength = (*PartialDescriptor).u.Memory.Length;
                        (*AccessRange).RangeInMemory = TRUE;
                        RangeNumber = RangeNumber.wrapping_add(1);
                    }
                }

                __c if __c == CmResourceTypeInterrupt as _ => {
                    DPRINT1!(
                        "Interrupt: Level %lu  Vector %lu\n",
                        (*PartialDescriptor).u.Interrupt.Level,
                        (*PartialDescriptor).u.Interrupt.Vector
                    );
                    if Interrupt == 0 {
                        /* Copy interrupt data */
                        (*PortConfiguration).BusInterruptLevel =
                            (*PartialDescriptor).u.Interrupt.Level;
                        (*PortConfiguration).BusInterruptVector =
                            (*PartialDescriptor).u.Interrupt.Vector;

                        /* Set interrupt mode accordingly to the resource */
                        if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            == (CM_RESOURCE_INTERRUPT_LATCHED) as _
                        {
                            (*PortConfiguration).InterruptMode = Latched;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            == (CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE) as _
                        {
                            (*PortConfiguration).InterruptMode = LevelSensitive;
                        }
                    } else if Interrupt == 1 {
                        /* Copy interrupt data */
                        (*PortConfiguration).BusInterruptLevel2 =
                            (*PartialDescriptor).u.Interrupt.Level;
                        (*PortConfiguration).BusInterruptVector2 =
                            (*PartialDescriptor).u.Interrupt.Vector;

                        /* Set interrupt mode accordingly to the resource */
                        if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            == (CM_RESOURCE_INTERRUPT_LATCHED) as _
                        {
                            (*PortConfiguration).InterruptMode2 = Latched;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            == (CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE) as _
                        {
                            (*PortConfiguration).InterruptMode2 = LevelSensitive;
                        }
                    }
                    Interrupt = Interrupt.wrapping_add(1);
                }

                __c if __c == CmResourceTypeDma as _ => {
                    DPRINT1!(
                        "Dma: Channel: %lu  Port: %lu\n",
                        (*PartialDescriptor).u.Dma.Channel,
                        (*PartialDescriptor).u.Dma.Port
                    );
                    if Dma == 0 {
                        (*PortConfiguration).DmaChannel = (*PartialDescriptor).u.Dma.Channel;
                        (*PortConfiguration).DmaPort = (*PartialDescriptor).u.Dma.Port;

                        if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            & CM_RESOURCE_DMA_8 as ::core::ffi::c_int != 0
                        {
                            (*PortConfiguration).DmaWidth = Width8Bits;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            & CM_RESOURCE_DMA_16 as ::core::ffi::c_int != 0
                            || (*PartialDescriptor).Flags as ::core::ffi::c_int
                                & CM_RESOURCE_DMA_8_AND_16 as ::core::ffi::c_int != 0
                        {
                            (*PortConfiguration).DmaWidth = Width16Bits;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            & CM_RESOURCE_DMA_32 as ::core::ffi::c_int != 0
                        {
                            (*PortConfiguration).DmaWidth = Width32Bits;
                        }
                    } else if Dma == 1 {
                        (*PortConfiguration).DmaChannel2 = (*PartialDescriptor).u.Dma.Channel;
                        (*PortConfiguration).DmaPort2 = (*PartialDescriptor).u.Dma.Port;

                        if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            & CM_RESOURCE_DMA_8 as ::core::ffi::c_int != 0
                        {
                            (*PortConfiguration).DmaWidth2 = Width8Bits;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            & CM_RESOURCE_DMA_16 as ::core::ffi::c_int != 0
                            || (*PartialDescriptor).Flags as ::core::ffi::c_int
                                & CM_RESOURCE_DMA_8_AND_16 as ::core::ffi::c_int != 0
                        {
                            (*PortConfiguration).DmaWidth2 = Width16Bits;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            & CM_RESOURCE_DMA_32 as ::core::ffi::c_int != 0
                        {
                            (*PortConfiguration).DmaWidth2 = Width32Bits;
                        }
                    }
                    Dma = Dma.wrapping_add(1);
                }

                _ => {
                    DPRINT1!(
                        "Other: %u\n",
                        (*PartialDescriptor).Type as ::core::ffi::c_int
                    );
                }
            }
            j += 1;
        }

        /* Advance to next CM_FULL_RESOURCE_DESCRIPTOR block in memory. */
        FullDescriptor = (&raw mut (*FullDescriptor).PartialResourceList.PartialDescriptors
            as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
            .offset((*FullDescriptor).PartialResourceList.Count as isize)
            as PCM_FULL_RESOURCE_DESCRIPTOR;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MiniportInitialize(
    mut Miniport: PMINIPORT,
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut InitData: PHW_INITIALIZATION_DATA,
) -> NTSTATUS {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    DPRINT1!("MiniportInitialize(%p %p %p)\n", Miniport, DeviceExtension, InitData);

    (*Miniport).DeviceExtension = DeviceExtension as *mut _FDO_DEVICE_EXTENSION;
    (*Miniport).InitData = InitData;

    /* Calculate the miniport device extension size */
    Status = RtlULongAdd(
        core::mem::size_of::<_MINIPORT_DEVICE_EXTENSION>() as ULONG,
        (*(*Miniport).InitData).DeviceExtensionSize,
        core::ptr::addr_of_mut!(Size),
    );
    if !NT_SUCCESS!(Status) {
        DPRINT1!(
            "Invalid miniport extension size %lx\n",
            (*(*Miniport).InitData).DeviceExtensionSize
        );
        return Status;
    }

    /* Allocate and initialize the miniport device extension */
    MiniportExtension = ExAllocatePoolWithTag(
        NonPagedPool,
        Size as SIZE_T,
        TAG_MINIPORT_DATA,
    ) as PMINIPORT_DEVICE_EXTENSION;
    if MiniportExtension.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlZeroMemory!(MiniportExtension as *mut core::ffi::c_void, Size as SIZE_T);

    (*MiniportExtension).Miniport = Miniport as *mut _MINIPORT;
    (*Miniport).MiniportExtension = MiniportExtension;

    /* Initialize the port configuration */
    Status = InitializeConfiguration(
        core::ptr::addr_of_mut!((*Miniport).PortConfig),
        InitData,
        (*DeviceExtension).BusNumber,
        (*DeviceExtension).SlotNumber,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    /* Assign the resources to the port configuration */
    AssignResourcesToConfiguration(
        core::ptr::addr_of_mut!((*Miniport).PortConfig),
        (*DeviceExtension).AllocatedResources,
        (*InitData).NumberOfAccessRanges,
    );

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MiniportFindAdapter(Miniport: PMINIPORT) -> NTSTATUS {
    let mut Reserved: BOOLEAN = FALSE;
    let mut Result: ULONG;
    let mut Status: NTSTATUS;

    DPRINT1!("MiniportFindAdapter(%p)\n", Miniport);

    /* Call the miniport HwFindAdapter routine */
    Result = (*(*Miniport).InitData)
        .HwFindAdapter
        .unwrap_unchecked()(
        core::ptr::addr_of_mut!((*(*Miniport).MiniportExtension).HwDeviceExtension) as PVOID,
        NULL,
        NULL,
        core::ptr::null_mut::<CHAR>(),
        core::ptr::addr_of_mut!((*Miniport).PortConfig),
        core::ptr::addr_of_mut!(Reserved),
    );
    DPRINT1!("HwFindAdapter() returned %lu\n", Result);

    /* Convert the result to a status code */
    match Result {
        SP_RETURN_NOT_FOUND => {
            DPRINT1!("SP_RETURN_NOT_FOUND\n");
            Status = STATUS_NOT_FOUND;
        }

        SP_RETURN_FOUND => {
            DPRINT1!("SP_RETURN_FOUND\n");
            Status = STATUS_SUCCESS;
        }

        SP_RETURN_ERROR => {
            DPRINT1!("SP_RETURN_ERROR\n");
            Status = STATUS_ADAPTER_HARDWARE_ERROR;
        }

        SP_RETURN_BAD_CONFIG => {
            DPRINT1!("SP_RETURN_BAD_CONFIG\n");
            Status = STATUS_DEVICE_CONFIGURATION_ERROR;
        }

        _ => {
            DPRINT1!("Unknown result: %lu\n", Result);
            Status = STATUS_INTERNAL_ERROR;
        }
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn MiniportHwInitialize(Miniport: PMINIPORT) -> NTSTATUS {
    let mut Result: BOOLEAN = 0;

    DPRINT!("MiniportHwInitialize(%p)\n", Miniport);

    /* Call the miniport HwInitialize routine */
    Result = (*(*Miniport).InitData)
        .HwInitialize
        .unwrap_unchecked()(
        &raw mut (*(*Miniport).MiniportExtension).HwDeviceExtension as PVOID,
    );
    DPRINT!("HwInitialize() returned %u\n", Result);

    return if Result != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    };
}
#[no_mangle]
pub unsafe extern "C" fn MiniportHwInterrupt(Miniport: PMINIPORT) -> BOOLEAN {
    let mut Result: BOOLEAN;

    DPRINT!("MiniportHwInterrupt(%p)\n", Miniport);

    Result = (*(*Miniport).InitData).HwInterrupt.unwrap_unchecked()(
        core::ptr::addr_of_mut!((*(*Miniport).MiniportExtension).HwDeviceExtension) as PVOID,
    );
    DPRINT!("HwInterrupt() returned %u\n", Result);

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn MiniportStartIo(
    Miniport: PMINIPORT,
    Srb: PSCSI_REQUEST_BLOCK,
) -> BOOLEAN {
    let mut Result: BOOLEAN;

    DPRINT!("MiniportHwStartIo(%p %p)\n", Miniport, Srb);

    Result = (*(*Miniport).InitData).HwStartIo.unwrap_unchecked()(
        &raw mut (*(*Miniport).MiniportExtension).HwDeviceExtension as PVOID,
        Srb,
    );
    DPRINT!("HwStartIo() returned %u\n", Result);

    return Result;
}
