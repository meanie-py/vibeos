use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoGetConfigurationInformation, PCM_FULL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_LIST, PCM_RESOURCE_LIST, PCONFIGURATION_INFORMATION};
use ntkrnl_sys::ddk::{PCSTR, CCHAR, INT, INTERFACE_TYPE, KINTERRUPT_MODE, CM_FULL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_DESCRIPTOR};
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG
unsafe fn InitializeConfiguration(
    mut PortConfig: PPORT_CONFIGURATION_INFORMATION,
    mut InitData: PHW_INITIALIZATION_DATA,
    mut BusNumber: ULONG,
    mut SlotNumber: ULONG,
) -> NTSTATUS {
    let mut ConfigInfo: PCONFIGURATION_INFORMATION;
    let mut i: ULONG;

    // DPRINT1("InitializeConfiguration(%p %p %lu %lu)\n", ...)

    /* Get the configurration information */
    ConfigInfo = IoGetConfigurationInformation();

    /* Initialize the port configuration */
    RtlZeroMemory(
        PortConfig as *mut core::ffi::c_void,
        core::mem::size_of::<PORT_CONFIGURATION_INFORMATION>() as SIZE_T,
    );

    (*PortConfig).Length = core::mem::size_of::<PORT_CONFIGURATION_INFORMATION>() as ULONG;
    (*PortConfig).SystemIoBusNumber = BusNumber;
    (*PortConfig).SlotNumber = SlotNumber;
    (*PortConfig).AdapterInterfaceType = (*InitData).AdapterInterfaceType;

    (*PortConfig).MaximumTransferLength = SP_UNINITIALIZED_VALUE;
    (*PortConfig).DmaChannel = SP_UNINITIALIZED_VALUE;
    (*PortConfig).DmaPort = SP_UNINITIALIZED_VALUE;

    (*PortConfig).InterruptMode = LevelSensitive;

    (*PortConfig).Master = 1 as BOOLEAN;
    (*PortConfig).AtdiskPrimaryClaimed = (*ConfigInfo).AtDiskPrimaryAddressClaimed;
    (*PortConfig).AtdiskSecondaryClaimed = (*ConfigInfo).AtDiskSecondaryAddressClaimed;
    (*PortConfig).Dma32BitAddresses = 1 as BOOLEAN;
    (*PortConfig).DemandMode = 0 as BOOLEAN;
    (*PortConfig).MapBuffers = (*InitData).MapBuffers;

    (*PortConfig).NeedPhysicalAddresses = 1 as BOOLEAN;
    (*PortConfig).TaggedQueuing = 1 as BOOLEAN;
    (*PortConfig).AutoRequestSense = 1 as BOOLEAN;
    (*PortConfig).MultipleRequestPerLu = 1 as BOOLEAN;
    (*PortConfig).ReceiveEvent = (*InitData).ReceiveEvent;
    (*PortConfig).RealModeInitialized = 0 as BOOLEAN;
    (*PortConfig).BufferAccessScsiPortControlled = 1 as BOOLEAN;
    (*PortConfig).MaximumNumberOfTargets = SCSI_MAXIMUM_TARGETS_PER_BUS as UCHAR;

    (*PortConfig).SpecificLuExtensionSize = (*InitData).SpecificLuExtensionSize;
    (*PortConfig).SrbExtensionSize = (*InitData).SrbExtensionSize;
    (*PortConfig).MaximumNumberOfLogicalUnits = SCSI_MAXIMUM_LOGICAL_UNITS as UCHAR;
    (*PortConfig).WmiDataProvider = 1 as BOOLEAN;

    (*PortConfig).NumberOfAccessRanges = (*InitData).NumberOfAccessRanges;
    // DPRINT1("NumberOfAccessRanges: %lu\n", PortConfig->NumberOfAccessRanges);
    if (*PortConfig).NumberOfAccessRanges != 0 {
        (*PortConfig).AccessRanges = ExAllocatePoolWithTag(
            NonPagedPool,
            ((*PortConfig).NumberOfAccessRanges as SIZE_T)
                .wrapping_mul(core::mem::size_of::<ACCESS_RANGE>() as SIZE_T),
            TAG_ACCRESS_RANGE as ULONG,
        ) as *mut [ACCESS_RANGE; 0];
        if (*PortConfig).AccessRanges.is_null() {
            return STATUS_NO_MEMORY;
        }

        RtlZeroMemory(
            (*PortConfig).AccessRanges as *mut core::ffi::c_void,
            ((*PortConfig).NumberOfAccessRanges as SIZE_T)
                .wrapping_mul(core::mem::size_of::<ACCESS_RANGE>() as SIZE_T),
        );
    }

    i = 0 as ULONG;
    while (i as usize)
        < (core::mem::size_of::<[CCHAR; 8]>() as usize)
            .wrapping_div(core::mem::size_of::<CCHAR>() as usize)
    {
        (*PortConfig).InitiatorBusId[i as usize] = SP_UNINITIALIZED_VALUE as CCHAR;
        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
unsafe fn AssignResourcesToConfiguration(
    PortConfiguration: PPORT_CONFIGURATION_INFORMATION,
    ResourceList: PCM_RESOURCE_LIST,
    NumberOfAccessRanges: ULONG,
) {
    const CmResourceTypePort: ::core::ffi::c_int = 1;
    const CmResourceTypeInterrupt: ::core::ffi::c_int = 2;
    const CmResourceTypeMemory: ::core::ffi::c_int = 3;
    const CmResourceTypeDma: ::core::ffi::c_int = 4;
    const CM_RESOURCE_INTERRUPT_LATCHED: ::core::ffi::c_int = 1;
    const CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE: ::core::ffi::c_int = 0;
    const CM_RESOURCE_DMA_8: ::core::ffi::c_int = 0x0001;
    const CM_RESOURCE_DMA_16: ::core::ffi::c_int = 0x0002;
    const CM_RESOURCE_DMA_32: ::core::ffi::c_int = 0x0004;
    const CM_RESOURCE_DMA_8_AND_16: ::core::ffi::c_int = 0x0008;
    const FALSE: ::core::ffi::c_int = 0;
    const TRUE: ::core::ffi::c_int = 1;

    let mut FullDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut PartialResourceList: PCM_PARTIAL_RESOURCE_LIST = core::ptr::null_mut();
    let mut AccessRange: PACCESS_RANGE = core::ptr::null_mut();
    let mut i: INT;
    let mut j: INT;
    let mut RangeNumber: ULONG = 0 as ULONG;
    let mut Interrupt: ULONG = 0 as ULONG;
    let mut Dma: ULONG = 0 as ULONG;

    if DbgPrint(
        b"(%s:%d) AssignResourceToConfiguration(%p %p %lu)\n\0" as *const u8 as PCSTR,
        b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
        109 as ::core::ffi::c_int,
        PortConfiguration,
        ResourceList,
        NumberOfAccessRanges,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
            109 as ::core::ffi::c_int,
        );
    }

    FullDescriptor = (core::ptr::addr_of_mut!((*ResourceList).List) as *mut CM_FULL_RESOURCE_DESCRIPTOR)
        .offset(0 as ::core::ffi::c_int as isize) as PCM_FULL_RESOURCE_DESCRIPTOR;
    i = 0 as ::core::ffi::c_int as INT;
    while (i as ::core::ffi::c_ulong) < (*ResourceList).Count {
        PartialResourceList =
            core::ptr::addr_of_mut!((*FullDescriptor).PartialResourceList) as PCM_PARTIAL_RESOURCE_LIST;
        j = 0 as ::core::ffi::c_int as INT;
        while (j as ::core::ffi::c_ulong) < (*PartialResourceList).Count {
            PartialDescriptor = (core::ptr::addr_of_mut!((*PartialResourceList).PartialDescriptors)
                as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(j as isize) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
            match (*PartialDescriptor).Type as ::core::ffi::c_int {
                CmResourceTypePort => {
                    if DbgPrint(
                        b"(%s:%d) Port: 0x%I64x (0x%lx)\n\0" as *const u8 as PCSTR,
                        b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
                        125 as ::core::ffi::c_int,
                        (*PartialDescriptor).u.Port.Start.QuadPart,
                        (*PartialDescriptor).u.Port.Length,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                            b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
                            125 as ::core::ffi::c_int,
                        );
                    }
                    if RangeNumber < NumberOfAccessRanges {
                        AccessRange = (core::ptr::addr_of_mut!(*(*PortConfiguration).AccessRanges)
                            as *mut ACCESS_RANGE)
                            .offset(RangeNumber as isize) as PACCESS_RANGE;
                        (*AccessRange).RangeStart = (*PartialDescriptor).u.Port.Start;
                        (*AccessRange).RangeLength = (*PartialDescriptor).u.Port.Length;
                        (*AccessRange).RangeInMemory = FALSE as BOOLEAN;
                        RangeNumber = RangeNumber.wrapping_add(1);
                    }
                }
                CmResourceTypeMemory => {
                    if DbgPrint(
                        b"(%s:%d) Memory: 0x%I64x (0x%lx)\n\0" as *const u8 as PCSTR,
                        b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
                        139 as ::core::ffi::c_int,
                        (*PartialDescriptor).u.Memory.Start.QuadPart,
                        (*PartialDescriptor).u.Memory.Length,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                            b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
                            139 as ::core::ffi::c_int,
                        );
                    }
                    if RangeNumber < NumberOfAccessRanges {
                        AccessRange = (core::ptr::addr_of_mut!(*(*PortConfiguration).AccessRanges)
                            as *mut ACCESS_RANGE)
                            .offset(RangeNumber as isize) as PACCESS_RANGE;
                        (*AccessRange).RangeStart = (*PartialDescriptor).u.Memory.Start;
                        (*AccessRange).RangeLength = (*PartialDescriptor).u.Memory.Length;
                        (*AccessRange).RangeInMemory = TRUE as BOOLEAN;
                        RangeNumber = RangeNumber.wrapping_add(1);
                    }
                }
                CmResourceTypeInterrupt => {
                    if DbgPrint(
                        b"(%s:%d) Interrupt: Level %lu  Vector %lu\n\0" as *const u8 as PCSTR,
                        b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
                        153 as ::core::ffi::c_int,
                        (*PartialDescriptor).u.Interrupt.Level,
                        (*PartialDescriptor).u.Interrupt.Vector,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                            b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
                            153 as ::core::ffi::c_int,
                        );
                    }
                    if Interrupt == 0 as ::core::ffi::c_ulong {
                        (*PortConfiguration).BusInterruptLevel = (*PartialDescriptor).u.Interrupt.Level;
                        (*PortConfiguration).BusInterruptVector = (*PartialDescriptor).u.Interrupt.Vector;
                        if (*PartialDescriptor).Flags as ::core::ffi::c_int == CM_RESOURCE_INTERRUPT_LATCHED {
                            (*PortConfiguration).InterruptMode = Latched;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            == CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE
                        {
                            (*PortConfiguration).InterruptMode = LevelSensitive;
                        }
                    } else if Interrupt == 1 as ::core::ffi::c_ulong {
                        (*PortConfiguration).BusInterruptLevel2 = (*PartialDescriptor).u.Interrupt.Level;
                        (*PortConfiguration).BusInterruptVector2 = (*PartialDescriptor).u.Interrupt.Vector;
                        if (*PartialDescriptor).Flags as ::core::ffi::c_int == CM_RESOURCE_INTERRUPT_LATCHED {
                            (*PortConfiguration).InterruptMode2 = Latched;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int
                            == CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE
                        {
                            (*PortConfiguration).InterruptMode2 = LevelSensitive;
                        }
                    }
                    Interrupt = Interrupt.wrapping_add(1);
                }
                CmResourceTypeDma => {
                    if DbgPrint(
                        b"(%s:%d) Dma: Channel: %lu  Port: %lu\n\0" as *const u8 as PCSTR,
                        b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
                        192 as ::core::ffi::c_int,
                        (*PartialDescriptor).u.Dma.Channel,
                        (*PartialDescriptor).u.Dma.Port,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                            b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
                            192 as ::core::ffi::c_int,
                        );
                    }
                    if Dma == 0 as ::core::ffi::c_ulong {
                        (*PortConfiguration).DmaChannel = (*PartialDescriptor).u.Dma.Channel;
                        (*PortConfiguration).DmaPort = (*PartialDescriptor).u.Dma.Port;
                        if (*PartialDescriptor).Flags as ::core::ffi::c_int & CM_RESOURCE_DMA_8 != 0 {
                            (*PortConfiguration).DmaWidth = Width8Bits;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int & CM_RESOURCE_DMA_16 != 0
                            || (*PartialDescriptor).Flags as ::core::ffi::c_int & CM_RESOURCE_DMA_8_AND_16 != 0
                        {
                            (*PortConfiguration).DmaWidth = Width16Bits;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int & CM_RESOURCE_DMA_32 != 0 {
                            (*PortConfiguration).DmaWidth = Width32Bits;
                        }
                    } else if Dma == 1 as ::core::ffi::c_ulong {
                        (*PortConfiguration).DmaChannel2 = (*PartialDescriptor).u.Dma.Channel;
                        (*PortConfiguration).DmaPort2 = (*PartialDescriptor).u.Dma.Port;
                        if (*PartialDescriptor).Flags as ::core::ffi::c_int & CM_RESOURCE_DMA_8 != 0 {
                            (*PortConfiguration).DmaWidth2 = Width8Bits;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int & CM_RESOURCE_DMA_16 != 0
                            || (*PartialDescriptor).Flags as ::core::ffi::c_int & CM_RESOURCE_DMA_8_AND_16 != 0
                        {
                            (*PortConfiguration).DmaWidth2 = Width16Bits;
                        } else if (*PartialDescriptor).Flags as ::core::ffi::c_int & CM_RESOURCE_DMA_32 != 0 {
                            (*PortConfiguration).DmaWidth2 = Width32Bits;
                        }
                    }
                    Dma = Dma.wrapping_add(1);
                }
                _ => {
                    if DbgPrint(
                        b"(%s:%d) Other: %u\n\0" as *const u8 as PCSTR,
                        b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
                        223 as ::core::ffi::c_int,
                        (*PartialDescriptor).Type as ::core::ffi::c_int,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                            b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
                            223 as ::core::ffi::c_int,
                        );
                    }
                }
            }
            j += 1;
        }
        FullDescriptor = (core::ptr::addr_of_mut!((*FullDescriptor).PartialResourceList.PartialDescriptors)
            as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
            .offset((*FullDescriptor).PartialResourceList.Count as isize)
            as PCM_FULL_RESOURCE_DESCRIPTOR;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MiniportInitialize(
    Miniport: PMINIPORT,
    DeviceExtension: PFDO_DEVICE_EXTENSION,
    InitData: PHW_INITIALIZATION_DATA,
) -> NTSTATUS {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION;
    let mut Size: ULONG;
    let mut Status: NTSTATUS;

    (*Miniport).DeviceExtension = DeviceExtension;
    (*Miniport).InitData = InitData;

    /* Calculate the miniport device extension size */
    Size = (core::mem::size_of::<MINIPORT_DEVICE_EXTENSION>() as ULONG)
        .wrapping_add((*(*Miniport).InitData).DeviceExtensionSize);

    /* Allocate and initialize the miniport device extension */
    MiniportExtension = ExAllocatePoolWithTag(
        NonPagedPool,
        Size as SIZE_T,
        TAG_MINIPORT_DATA as ULONG,
    ) as PMINIPORT_DEVICE_EXTENSION;
    if MiniportExtension.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlZeroMemory(MiniportExtension as *mut core::ffi::c_void, Size as SIZE_T);

    (*MiniportExtension).Miniport = Miniport;
    (*Miniport).MiniportExtension = MiniportExtension;

    /* Initialize the port configuration */
    Status = InitializeConfiguration(
        core::ptr::addr_of_mut!((*Miniport).PortConfig),
        InitData,
        (*DeviceExtension).BusNumber,
        (*DeviceExtension).SlotNumber,
    );
    if !NT_SUCCESS(Status) {
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
    const SP_RETURN_NOT_FOUND: ULONG = 0;
    const SP_RETURN_FOUND: ULONG = 1;
    const SP_RETURN_ERROR: ULONG = 2;
    const SP_RETURN_BAD_CONFIG: ULONG = 3;
    const STATUS_ADAPTER_HARDWARE_ERROR: NTSTATUS = 0xC00000A1u32 as NTSTATUS;
    const STATUS_DEVICE_CONFIGURATION_ERROR: NTSTATUS = 0xC0000182u32 as NTSTATUS;
    const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5u32 as NTSTATUS;

    let mut Reserved: BOOLEAN = 0 as BOOLEAN;
    let mut Result: ULONG;
    let Status: NTSTATUS;

    DbgPrint(
        b"MiniportFindAdapter(%p)\n\0".as_ptr() as *const i8,
        Miniport,
    );

    /* Call the miniport HwFindAdapter routine */
    Result = ((*(*Miniport).InitData).HwFindAdapter.expect("non-null function pointer"))(
        core::ptr::addr_of_mut!((*(*Miniport).MiniportExtension).HwDeviceExtension) as PVOID,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut::<CHAR>(),
        core::ptr::addr_of_mut!((*Miniport).PortConfig),
        core::ptr::addr_of_mut!(Reserved),
    );
    DbgPrint(
        b"HwFindAdapter() returned %lu\n\0".as_ptr() as *const i8,
        Result,
    );

    /* Convert the result to a status code */
    match Result {
        SP_RETURN_NOT_FOUND => {
            DbgPrint(b"SP_RETURN_NOT_FOUND\n\0".as_ptr() as *const i8);
            Status = STATUS_NOT_FOUND;
        }
        SP_RETURN_FOUND => {
            DbgPrint(b"SP_RETURN_FOUND\n\0".as_ptr() as *const i8);
            Status = STATUS_SUCCESS;
        }
        SP_RETURN_ERROR => {
            DbgPrint(b"SP_RETURN_ERROR\n\0".as_ptr() as *const i8);
            Status = STATUS_ADAPTER_HARDWARE_ERROR;
        }
        SP_RETURN_BAD_CONFIG => {
            DbgPrint(b"SP_RETURN_BAD_CONFIG\n\0".as_ptr() as *const i8);
            Status = STATUS_DEVICE_CONFIGURATION_ERROR;
        }
        _ => {
            DbgPrint(
                b"Unknown result: %lu\n\0".as_ptr() as *const i8,
                Result,
            );
            Status = STATUS_INTERNAL_ERROR;
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn MiniportHwInitialize(mut Miniport: PMINIPORT) -> NTSTATUS {

    let mut Result: BOOLEAN;

    if DbgPrint(
        b"(%s:%d) MiniportHwInitialize(%p)\n\0" as *const u8 as PCSTR,
        b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/storage/port/storport/miniport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        342 as ::core::ffi::c_int,
        Miniport,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/storage/port/storport/miniport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            342 as ::core::ffi::c_int,
        );
    }

    Result = ((*(*Miniport).InitData)
        .HwInitialize
        .expect("non-null function pointer"))(
        core::ptr::addr_of_mut!((*(*Miniport).MiniportExtension).HwDeviceExtension) as PVOID,
    );

    if DbgPrint(
        b"(%s:%d) HwInitialize() returned %u\n\0" as *const u8 as PCSTR,
        b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/storage/port/storport/miniport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        346 as ::core::ffi::c_int,
        Result as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"/home/curi/Projects/os-playground/vibeos-wt-filesystems/drivers/storage/port/storport/miniport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            346 as ::core::ffi::c_int,
        );
    }

    return if Result as ::core::ffi::c_int != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    };
}

#[no_mangle]
pub unsafe extern "C" fn MiniportHwInterrupt(mut Miniport: PMINIPORT) -> BOOLEAN {
    let mut Result: BOOLEAN;

    if DbgPrint(
        b"(%s:%d) MiniportHwInterrupt(%p)\n\0" as *const u8 as PCSTR,
        b"miniport.c\0" as *const u8 as *const core::ffi::c_char,
        359 as core::ffi::c_int,
        Miniport,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"miniport.c\0" as *const u8 as *const core::ffi::c_char,
            359 as core::ffi::c_int,
        );
    }

    Result = (*(*Miniport).InitData)
        .HwInterrupt
        .expect("non-null function pointer")(
        core::ptr::addr_of_mut!((*(*Miniport).MiniportExtension).HwDeviceExtension) as PVOID,
    );

    if DbgPrint(
        b"(%s:%d) HwInterrupt() returned %u\n\0" as *const u8 as PCSTR,
        b"miniport.c\0" as *const u8 as *const core::ffi::c_char,
        362 as core::ffi::c_int,
        Result as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"miniport.c\0" as *const u8 as *const core::ffi::c_char,
            362 as core::ffi::c_int,
        );
    }

    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn MiniportStartIo(
    mut Miniport: PMINIPORT,
    mut Srb: PSCSI_REQUEST_BLOCK,
) -> BOOLEAN {
    let mut Result: BOOLEAN = 0;

    if DbgPrint(
        b"(%s:%d) MiniportHwStartIo(%p %p)\n\0" as *const u8 as PCSTR,
        b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
        376 as ::core::ffi::c_int,
        Miniport,
        Srb,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
            376 as ::core::ffi::c_int,
        );
    }

    Result = (*(*Miniport).InitData)
        .HwStartIo
        .expect("non-null function pointer")(
        core::ptr::addr_of_mut!((*(*Miniport).MiniportExtension).HwDeviceExtension) as PVOID,
        Srb,
    );

    if DbgPrint(
        b"(%s:%d) HwStartIo() returned %u\n\0" as *const u8 as PCSTR,
        b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
        379 as ::core::ffi::c_int,
        Result as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"miniport.c\0" as *const u8 as *const ::core::ffi::c_char,
            379 as ::core::ffi::c_int,
        );
    }

    return Result;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Storport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Miniport interface code
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
// NTSTATUS
// InitializeConfiguration(
//      PPORT_CONFIGURATION_INFORMATION PortConfig,
//      PHW_INITIALIZATION_DATA InitData,
//      ULONG BusNumber,
//      ULONG SlotNumber)
// {
//     PCONFIGURATION_INFORMATION ConfigInfo;
//     ULONG i;
//
//     DPRINT1("InitializeConfiguration(%p %p %lu %lu)\n",
//             PortConfig, InitData, BusNumber, SlotNumber);
//
//     /* Get the configurration information */
//     ConfigInfo = IoGetConfigurationInformation();
//
//     /* Initialize the port configuration */
//     RtlZeroMemory(PortConfig,
//                   sizeof(PORT_CONFIGURATION_INFORMATION));
//
//     PortConfig->Length = sizeof(PORT_CONFIGURATION_INFORMATION);
//     PortConfig->SystemIoBusNumber = BusNumber;
//     PortConfig->SlotNumber = SlotNumber;
//     PortConfig->AdapterInterfaceType = InitData->AdapterInterfaceType;
//
//     PortConfig->MaximumTransferLength = SP_UNINITIALIZED_VALUE;
//     PortConfig->DmaChannel = SP_UNINITIALIZED_VALUE;
//     PortConfig->DmaPort = SP_UNINITIALIZED_VALUE;
//
//     PortConfig->InterruptMode = LevelSensitive;
//
//     PortConfig->Master = TRUE;
//     PortConfig->AtdiskPrimaryClaimed = ConfigInfo->AtDiskPrimaryAddressClaimed;
//     PortConfig->AtdiskSecondaryClaimed = ConfigInfo->AtDiskSecondaryAddressClaimed;
//     PortConfig->Dma32BitAddresses = TRUE;
//     PortConfig->DemandMode = FALSE;
//     PortConfig->MapBuffers = InitData->MapBuffers;
//
//     PortConfig->NeedPhysicalAddresses = TRUE;
//     PortConfig->TaggedQueuing = TRUE;
//     PortConfig->AutoRequestSense = TRUE;
//     PortConfig->MultipleRequestPerLu = TRUE;
//     PortConfig->ReceiveEvent = InitData->ReceiveEvent;
//     PortConfig->RealModeInitialized = FALSE;
//     PortConfig->BufferAccessScsiPortControlled = TRUE;
//     PortConfig->MaximumNumberOfTargets = SCSI_MAXIMUM_TARGETS_PER_BUS;
//
//     PortConfig->SpecificLuExtensionSize = InitData->SpecificLuExtensionSize;
//     PortConfig->SrbExtensionSize = InitData->SrbExtensionSize;
//     PortConfig->MaximumNumberOfLogicalUnits = SCSI_MAXIMUM_LOGICAL_UNITS;
//     PortConfig->WmiDataProvider = TRUE;
//
//     PortConfig->NumberOfAccessRanges = InitData->NumberOfAccessRanges;
//     DPRINT1("NumberOfAccessRanges: %lu\n", PortConfig->NumberOfAccessRanges);
//     if (PortConfig->NumberOfAccessRanges != 0)
//     {
//         PortConfig->AccessRanges = ExAllocatePoolWithTag(NonPagedPool,
//                                                          PortConfig->NumberOfAccessRanges * sizeof(ACCESS_RANGE),
//                                                          TAG_ACCRESS_RANGE);
//         if (PortConfig->AccessRanges == NULL)
//             return STATUS_NO_MEMORY;
//
//         RtlZeroMemory(PortConfig->AccessRanges,
//                       PortConfig->NumberOfAccessRanges * sizeof(ACCESS_RANGE));
//     }
//
//     for (i = 0; i < RTL_NUMBER_OF(PortConfig->InitiatorBusId); i++)
//     {
//         PortConfig->InitiatorBusId[i] = (CCHAR)SP_UNINITIALIZED_VALUE;
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// VOID
// AssignResourcesToConfiguration(
//      PPORT_CONFIGURATION_INFORMATION PortConfiguration,
//      PCM_RESOURCE_LIST ResourceList,
//      ULONG NumberOfAccessRanges)
// {
//     PCM_FULL_RESOURCE_DESCRIPTOR FullDescriptor;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     PCM_PARTIAL_RESOURCE_LIST PartialResourceList;
//     PACCESS_RANGE AccessRange;
//     INT i, j;
//     ULONG RangeNumber = 0, Interrupt = 0, Dma = 0;
//
//     DPRINT1("AssignResourceToConfiguration(%p %p %lu)\n",
//             PortConfiguration, ResourceList, NumberOfAccessRanges);
//
//     FullDescriptor = &ResourceList->List[0];
//     for (i = 0; i < ResourceList->Count; i++)
//     {
//         PartialResourceList = &FullDescriptor->PartialResourceList;
//
//         for (j = 0; j < PartialResourceList->Count; j++)
//         {
//             PartialDescriptor = &PartialResourceList->PartialDescriptors[j];
//
//             switch (PartialDescriptor->Type)
//             {
//                 case CmResourceTypePort:
//                     DPRINT1("Port: 0x%I64x (0x%lx)\n",
//                             PartialDescriptor->u.Port.Start.QuadPart,
//                             PartialDescriptor->u.Port.Length);
//                     if (RangeNumber < NumberOfAccessRanges)
//                     {
//                         AccessRange = &((*(PortConfiguration->AccessRanges))[RangeNumber]);
//                         AccessRange->RangeStart = PartialDescriptor->u.Port.Start;
//                         AccessRange->RangeLength = PartialDescriptor->u.Port.Length;
//                         AccessRange->RangeInMemory = FALSE;
//                         RangeNumber++;
//                     }
//                     break;
//
//                 case CmResourceTypeMemory:
//                     DPRINT1("Memory: 0x%I64x (0x%lx)\n",
//                             PartialDescriptor->u.Memory.Start.QuadPart,
//                             PartialDescriptor->u.Memory.Length);
//                     if (RangeNumber < NumberOfAccessRanges)
//                     {
//                         AccessRange = &((*(PortConfiguration->AccessRanges))[RangeNumber]);
//                         AccessRange->RangeStart = PartialDescriptor->u.Memory.Start;
//                         AccessRange->RangeLength = PartialDescriptor->u.Memory.Length;
//                         AccessRange->RangeInMemory = TRUE;
//                         RangeNumber++;
//                     }
//                     break;
//
//                 case CmResourceTypeInterrupt:
//                     DPRINT1("Interrupt: Level %lu  Vector %lu\n",
//                             PartialDescriptor->u.Interrupt.Level,
//                             PartialDescriptor->u.Interrupt.Vector);
//                     if (Interrupt == 0)
//                     {
//                         /* Copy interrupt data */
//                         PortConfiguration->BusInterruptLevel = PartialDescriptor->u.Interrupt.Level;
//                         PortConfiguration->BusInterruptVector = PartialDescriptor->u.Interrupt.Vector;
//
//                         /* Set interrupt mode accordingly to the resource */
//                         if (PartialDescriptor->Flags == CM_RESOURCE_INTERRUPT_LATCHED)
//                         {
//                             PortConfiguration->InterruptMode = Latched;
//                         }
//                         else if (PartialDescriptor->Flags == CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE)
//                         {
//                             PortConfiguration->InterruptMode = LevelSensitive;
//                         }
//                     }
//                     else if (Interrupt == 1)
//                     {
//                         /* Copy interrupt data */
//                         PortConfiguration->BusInterruptLevel2 = PartialDescriptor->u.Interrupt.Level;
//                         PortConfiguration->BusInterruptVector2 = PartialDescriptor->u.Interrupt.Vector;
//
//                         /* Set interrupt mode accordingly to the resource */
//                         if (PartialDescriptor->Flags == CM_RESOURCE_INTERRUPT_LATCHED)
//                         {
//                             PortConfiguration->InterruptMode2 = Latched;
//                         }
//                         else if (PartialDescriptor->Flags == CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE)
//                         {
//                             PortConfiguration->InterruptMode2 = LevelSensitive;
//                         }
//                     }
//                     Interrupt++;
//                     break;
//
//                 case CmResourceTypeDma:
//                     DPRINT1("Dma: Channel: %lu  Port: %lu\n",
//                             PartialDescriptor->u.Dma.Channel,
//                             PartialDescriptor->u.Dma.Port);
//                     if (Dma == 0)
//                     {
//                         PortConfiguration->DmaChannel = PartialDescriptor->u.Dma.Channel;
//                         PortConfiguration->DmaPort = PartialDescriptor->u.Dma.Port;
//
//                         if (PartialDescriptor->Flags & CM_RESOURCE_DMA_8)
//                             PortConfiguration->DmaWidth = Width8Bits;
//                         else if ((PartialDescriptor->Flags & CM_RESOURCE_DMA_16) ||
//                                  (PartialDescriptor->Flags & CM_RESOURCE_DMA_8_AND_16))
//                             PortConfiguration->DmaWidth = Width16Bits;
//                         else if (PartialDescriptor->Flags & CM_RESOURCE_DMA_32)
//                             PortConfiguration->DmaWidth = Width32Bits;
//                     }
//                     else if (Dma == 1)
//                     {
//                         PortConfiguration->DmaChannel2 = PartialDescriptor->u.Dma.Channel;
//                         PortConfiguration->DmaPort2 = PartialDescriptor->u.Dma.Port;
//
//                         if (PartialDescriptor->Flags & CM_RESOURCE_DMA_8)
//                             PortConfiguration->DmaWidth2 = Width8Bits;
//                         else if ((PartialDescriptor->Flags & CM_RESOURCE_DMA_16) ||
//                                  (PartialDescriptor->Flags & CM_RESOURCE_DMA_8_AND_16))
//                             PortConfiguration->DmaWidth2 = Width16Bits;
//                         else if (PartialDescriptor->Flags & CM_RESOURCE_DMA_32)
//                             PortConfiguration->DmaWidth2 = Width32Bits;
//                     }
//                     Dma++;
//                     break;
//
//                 default:
//                     DPRINT1("Other: %u\n", PartialDescriptor->Type);
//                     break;
//             }
//         }
//
//         /* Advance to next CM_FULL_RESOURCE_DESCRIPTOR block in memory. */
//         FullDescriptor = (PCM_FULL_RESOURCE_DESCRIPTOR)(FullDescriptor->PartialResourceList.PartialDescriptors +
//                                                         FullDescriptor->PartialResourceList.Count);
//     }
// }
//
//
// NTSTATUS
// MiniportInitialize(
//      PMINIPORT Miniport,
//      PFDO_DEVICE_EXTENSION DeviceExtension,
//      PHW_INITIALIZATION_DATA InitData)
// {
//     PMINIPORT_DEVICE_EXTENSION MiniportExtension;
//     ULONG Size;
//     NTSTATUS Status;
//
//     DPRINT1("MiniportInitialize(%p %p %p)\n",
//             Miniport, DeviceExtension, InitData);
//
//     Miniport->DeviceExtension = DeviceExtension;
//     Miniport->InitData = InitData;
//
//     /* Calculate the miniport device extension size */
//     Size = sizeof(MINIPORT_DEVICE_EXTENSION) +
//            Miniport->InitData->DeviceExtensionSize;
//
//     /* Allocate and initialize the miniport device extension */
//     MiniportExtension = ExAllocatePoolWithTag(NonPagedPool,
//                                               Size,
//                                               TAG_MINIPORT_DATA);
//     if (MiniportExtension == NULL)
//         return STATUS_NO_MEMORY;
//
//     RtlZeroMemory(MiniportExtension, Size);
//
//     MiniportExtension->Miniport = Miniport;
//     Miniport->MiniportExtension = MiniportExtension;
//
//     /* Initialize the port configuration */
//     Status = InitializeConfiguration(&Miniport->PortConfig,
//                                      InitData,
//                                      DeviceExtension->BusNumber,
//                                      DeviceExtension->SlotNumber);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Assign the resources to the port configuration */
//     AssignResourcesToConfiguration(&Miniport->PortConfig,
//                                    DeviceExtension->AllocatedResources,
//                                    InitData->NumberOfAccessRanges);
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// MiniportFindAdapter(
//      PMINIPORT Miniport)
// {
//     BOOLEAN Reserved = FALSE;
//     ULONG Result;
//     NTSTATUS Status;
//
//     DPRINT1("MiniportFindAdapter(%p)\n", Miniport);
//
//     /* Call the miniport HwFindAdapter routine */
//     Result = Miniport->InitData->HwFindAdapter(&Miniport->MiniportExtension->HwDeviceExtension,
//                                                NULL,
//                                                NULL,
//                                                NULL,
//                                                &Miniport->PortConfig,
//                                                &Reserved);
//     DPRINT1("HwFindAdapter() returned %lu\n", Result);
//
//     /* Convert the result to a status code */
//     switch (Result)
//     {
//         case SP_RETURN_NOT_FOUND:
//             DPRINT1("SP_RETURN_NOT_FOUND\n");
//             Status = STATUS_NOT_FOUND;
//             break;
//
//         case SP_RETURN_FOUND:
//             DPRINT1("SP_RETURN_FOUND\n");
//             Status = STATUS_SUCCESS;
//             break;
//
//         case SP_RETURN_ERROR:
//             DPRINT1("SP_RETURN_ERROR\n");
//             Status = STATUS_ADAPTER_HARDWARE_ERROR;
//             break;
//
//         case SP_RETURN_BAD_CONFIG:
//             DPRINT1("SP_RETURN_BAD_CONFIG\n");
//             Status = STATUS_DEVICE_CONFIGURATION_ERROR;
//             break;
//
//         default:
//             DPRINT1("Unknown result: %lu\n", Result);
//             Status = STATUS_INTERNAL_ERROR;
//             break;
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// MiniportHwInitialize(
//      PMINIPORT Miniport)
// {
//     BOOLEAN Result;
//
//     DPRINT1("MiniportHwInitialize(%p)\n", Miniport);
//
//     /* Call the miniport HwInitialize routine */
//     Result = Miniport->InitData->HwInitialize(&Miniport->MiniportExtension->HwDeviceExtension);
//     DPRINT1("HwInitialize() returned %u\n", Result);
//
//     return Result ? STATUS_SUCCESS : STATUS_UNSUCCESSFUL;
// }
//
//
// BOOLEAN
// MiniportHwInterrupt(
//      PMINIPORT Miniport)
// {
//     BOOLEAN Result;
//
//     DPRINT1("MiniportHwInterrupt(%p)\n",
//             Miniport);
//
//     Result = Miniport->InitData->HwInterrupt(&Miniport->MiniportExtension->HwDeviceExtension);
//     DPRINT1("HwInterrupt() returned %u\n", Result);
//
//     return Result;
// }
//
//
// BOOLEAN
// MiniportStartIo(
//      PMINIPORT Miniport,
//      PSCSI_REQUEST_BLOCK Srb)
// {
//     BOOLEAN Result;
//
//     DPRINT1("MiniportHwStartIo(%p %p)\n",
//             Miniport, Srb);
//
//     Result = Miniport->InitData->HwStartIo(&Miniport->MiniportExtension->HwDeviceExtension, Srb);
//     DPRINT1("HwStartIo() returned %u\n", Result);
//
//     return Result;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
