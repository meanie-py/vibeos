use crate::prelude::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_RESOURCE_LIST, _KWAIT_REASON_Executive, GUID, _IO_STATUS_BLOCK, IoBuildSynchronousFsdRequest, IoGetDeviceProperty, KEVENT, KeInitializeEvent, KeWaitForSingleObject, MAPPED_ADDRESS, _EVENT_TYPE_NotificationEvent, PBUS_INTERFACE_STANDARD, PCM_FULL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PCSTR, PDEVICE_OBJECT, PGUID, PINTERFACE, PIO_STACK_LOCATION, PIRP, PKAFFINITY, PKIRQL, PMAPPED_ADDRESS, PPHYSICAL_ADDRESS, RtlCompareMemory};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn ForwardIrpAndForget(
    LowerDevice: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    ASSERT!(!LowerDevice.is_null());

    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver(LowerDevice, Irp);
}
#[no_mangle]
pub unsafe extern "C" fn GetBusInterface(DeviceObject: PDEVICE_OBJECT) -> INTERFACE_TYPE {
    let mut Guid: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS;

    Status = IoGetDeviceProperty(
        DeviceObject,
        DevicePropertyBusTypeGuid,
        core::mem::size_of::<GUID>() as ULONG,
        core::ptr::addr_of_mut!(Guid) as PVOID,
        core::ptr::addr_of_mut!(Length),
    );
    if !NT_SUCCESS!(Status) {
        return InterfaceTypeUndefined;
    }

    if RtlCompareMemory(
        core::ptr::addr_of!(Guid) as *const core::ffi::c_void,
        core::ptr::addr_of!(GUID_BUS_TYPE_PCMCIA) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as SIZE_T,
    ) as usize
        == core::mem::size_of::<GUID>() as usize
    {
        return PCMCIABus;
    } else if RtlCompareMemory(
        core::ptr::addr_of!(Guid) as *const core::ffi::c_void,
        core::ptr::addr_of!(GUID_BUS_TYPE_PCI) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as SIZE_T,
    ) as usize
        == core::mem::size_of::<GUID>() as usize
    {
        return PCIBus;
    } else if RtlCompareMemory(
        core::ptr::addr_of!(Guid) as *const core::ffi::c_void,
        core::ptr::addr_of!(GUID_BUS_TYPE_ISAPNP) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as SIZE_T,
    ) as usize
        == core::mem::size_of::<GUID>() as usize
    {
        return PNPISABus;
    }

    return InterfaceTypeUndefined;
}

unsafe extern "C" fn GetResourceListSize(mut ResourceList: PCM_RESOURCE_LIST) -> ULONG {
    let mut Descriptor: PCM_FULL_RESOURCE_DESCRIPTOR;
    let mut Size: ULONG;

    DPRINT1!("GetResourceListSize(%p)\n", ResourceList);

    Size = ::core::mem::size_of::<CM_RESOURCE_LIST>() as ULONG;
    if (*ResourceList).Count == 0 {
        DPRINT1!("Size: 0x%lx (%u)\n", Size, Size);
        return Size;
    }

    DPRINT1!("ResourceList->Count: %lu\n", (*ResourceList).Count);

    Descriptor = (&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
        .offset(0 as isize)
        as *mut CM_FULL_RESOURCE_DESCRIPTOR as PCM_FULL_RESOURCE_DESCRIPTOR;

    DPRINT1!("PartialResourceList->Count: %lu\n", (*Descriptor).PartialResourceList.Count);

    /* Add the size of the partial descriptors */
    if (*Descriptor).PartialResourceList.Count > 1 {
        Size = Size.wrapping_add(
            (((*Descriptor).PartialResourceList.Count.wrapping_sub(1 as ULONG)) as usize)
                .wrapping_mul(::core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as usize)
                as ULONG,
        ) as ULONG;
    }

    DPRINT1!("Size: 0x%lx (%u)\n", Size, Size);
    return Size;
}
#[no_mangle]
pub unsafe extern "C" fn CopyResourceList(
    PoolType: POOL_TYPE,
    Source: PCM_RESOURCE_LIST,
) -> PCM_RESOURCE_LIST {
    let mut Destination: PCM_RESOURCE_LIST = core::ptr::null_mut();
    let mut Size: ULONG = 0;

    DPRINT1!("CopyResourceList(%lu %p)\n", PoolType, Source);

    /* Get the size of the resource list */
    Size = GetResourceListSize(Source);

    /* Allocate a new buffer */
    Destination = ExAllocatePoolWithTag(PoolType, Size as SIZE_T, TAG_RESOURCE_LIST)
        as PCM_RESOURCE_LIST;
    if Destination.is_null() {
        return core::ptr::null_mut();
    }

    /* Copy the resource list */
    RtlCopyMemory!(Destination as *mut core::ffi::c_void, Source as *const core::ffi::c_void, Size as SIZE_T);

    return Destination;
}
#[no_mangle]
pub unsafe extern "C" fn QueryBusInterface(
    DeviceObject: PDEVICE_OBJECT,
    Guid: PGUID,
    Size: USHORT,
    Version: USHORT,
    Interface: PBUS_INTERFACE_STANDARD,
    InterfaceSpecificData: PVOID,
) -> NTSTATUS {
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Irp: PIRP;
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Stack: PIO_STACK_LOCATION;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, FALSE);

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as ULONG,
        DeviceObject,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatus),
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Stack = IoGetNextIrpStackLocation(Irp);

    (*Stack).MajorFunction = IRP_MJ_PNP as UCHAR;
    (*Stack).MinorFunction = IRP_MN_QUERY_INTERFACE as UCHAR;
    (*Stack).Parameters.QueryInterface.InterfaceType = Guid as *const GUID;
    (*Stack).Parameters.QueryInterface.Size = Size;
    (*Stack).Parameters.QueryInterface.Version = Version;
    (*Stack).Parameters.QueryInterface.Interface = Interface as PINTERFACE;
    (*Stack).Parameters.QueryInterface.InterfaceSpecificData = InterfaceSpecificData;

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as KPROCESSOR_MODE,
            FALSE as BOOLEAN,
            core::ptr::null_mut(),
        );

        Status = IoStatus.__bindgen_anon_1.Status;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn TranslateResourceListAddress(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut BusType: INTERFACE_TYPE,
    mut SystemIoBusNumber: ULONG,
    mut IoAddress: STOR_PHYSICAL_ADDRESS,
    mut NumberOfBytes: ULONG,
    mut InIoSpace: BOOLEAN,
    mut TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    let mut FullDescriptorA: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut FullDescriptorT: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut PartialDescriptorA: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut PartialDescriptorT: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut i: INT;
    let mut j: INT;

    DPRINT1!("TranslateResourceListAddress(%p)\n", DeviceExtension);

    FullDescriptorA = &raw mut (*(*DeviceExtension).AllocatedResources).List
        as *mut CM_FULL_RESOURCE_DESCRIPTOR as PCM_FULL_RESOURCE_DESCRIPTOR;
    FullDescriptorT = &raw mut (*(*DeviceExtension).TranslatedResources).List
        as *mut CM_FULL_RESOURCE_DESCRIPTOR as PCM_FULL_RESOURCE_DESCRIPTOR;
    i = 0 as INT;
    while (i as ULONG) < (*(*DeviceExtension).AllocatedResources).Count {
        j = 0 as INT;
        while (j as ULONG) < (*FullDescriptorA).PartialResourceList.Count {
            PartialDescriptorA = (&raw mut (*FullDescriptorA).PartialResourceList.PartialDescriptors
                as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(j as isize)
                as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
            PartialDescriptorT = (&raw mut (*FullDescriptorT).PartialResourceList.PartialDescriptors
                as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(j as isize)
                as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

            match (*PartialDescriptorA).Type as INT {
                __c if __c == CmResourceTypePort as _ => {
                    DPRINT1!(
                        "Port: 0x%I64x (0x%lx)\n",
                        (*PartialDescriptorA).u.Port.Start.QuadPart,
                        (*PartialDescriptorA).u.Port.Length
                    );
                    if InIoSpace as INT != 0
                        && IoAddress.QuadPart >= (*PartialDescriptorA).u.Port.Start.QuadPart
                    {
                    }
                    if InIoSpace as INT != 0
                        && IoAddress.QuadPart >= (*PartialDescriptorA).u.Port.Start.QuadPart
                        && IoAddress.QuadPart + NumberOfBytes as LONGLONG
                            <= (*PartialDescriptorA).u.Port.Start.QuadPart
                                + (*PartialDescriptorA).u.Port.Length as LONGLONG
                    {
                        (*TranslatedAddress).QuadPart = (*PartialDescriptorT).u.Port.Start.QuadPart
                            + (IoAddress.QuadPart - (*PartialDescriptorA).u.Port.Start.QuadPart);
                        return TRUE as BOOLEAN;
                    }
                }
                __c if __c == CmResourceTypeMemory as _ => {
                    DPRINT1!(
                        "Memory: 0x%I64x (0x%lx)\n",
                        (*PartialDescriptorA).u.Memory.Start.QuadPart,
                        (*PartialDescriptorA).u.Memory.Length
                    );
                    if InIoSpace == 0
                        && IoAddress.QuadPart >= (*PartialDescriptorA).u.Memory.Start.QuadPart
                        && IoAddress.QuadPart + NumberOfBytes as LONGLONG
                            <= (*PartialDescriptorA).u.Memory.Start.QuadPart
                                + (*PartialDescriptorA).u.Memory.Length as LONGLONG
                    {
                        (*TranslatedAddress).QuadPart =
                            (*PartialDescriptorT).u.Memory.Start.QuadPart
                                + (IoAddress.QuadPart
                                    - (*PartialDescriptorA).u.Memory.Start.QuadPart);
                        return TRUE as BOOLEAN;
                    }
                }
                _ => {}
            }
            j += 1;
        }

        /* Advance to next CM_FULL_RESOURCE_DESCRIPTOR block in memory. */
        FullDescriptorA = (&raw mut (*FullDescriptorA).PartialResourceList.PartialDescriptors
            as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
            .offset((*FullDescriptorA).PartialResourceList.Count as isize)
            as PCM_FULL_RESOURCE_DESCRIPTOR;

        FullDescriptorT = (&raw mut (*FullDescriptorT).PartialResourceList.PartialDescriptors
            as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
            .offset((*FullDescriptorT).PartialResourceList.Count as isize)
            as PCM_FULL_RESOURCE_DESCRIPTOR;
        i += 1;
    }

    return FALSE as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn GetResourceListInterrupt(
    DeviceExtension: PFDO_DEVICE_EXTENSION,
    Vector: PULONG,
    Irql: PKIRQL,
    InterruptMode: *mut KINTERRUPT_MODE,
    ShareVector: PBOOLEAN,
    Affinity: PKAFFINITY,
) -> NTSTATUS {
    let mut FullDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut j: INT = 0;

    DPRINT1!("GetResourceListInterrupt(%p)\n", DeviceExtension);

    FullDescriptor = &raw mut (*(*DeviceExtension).TranslatedResources).List
        as *mut CM_FULL_RESOURCE_DESCRIPTOR as PCM_FULL_RESOURCE_DESCRIPTOR;
    i = 0 as INT;
    while (i as ULONG) < (*(*DeviceExtension).TranslatedResources).Count {
        j = 0 as INT;
        while (j as ULONG) < (*FullDescriptor).PartialResourceList.Count {
            PartialDescriptor = (&raw mut (*FullDescriptor).PartialResourceList.PartialDescriptors
                as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(j as isize)
                as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

            match (*PartialDescriptor).Type as ::core::ffi::c_int {
                __c if __c == CmResourceTypeInterrupt as _ => {
                    DPRINT1!(
                        "Interrupt: Level %lu  Vector %lu\n",
                        (*PartialDescriptor).u.Interrupt.Level,
                        (*PartialDescriptor).u.Interrupt.Vector
                    );

                    *Vector = (*PartialDescriptor).u.Interrupt.Vector as ULONG;
                    *Irql = (*PartialDescriptor).u.Interrupt.Level as KIRQL;
                    *InterruptMode = (if ((*PartialDescriptor).Flags as ULONG & CM_RESOURCE_INTERRUPT_LATCHED as ULONG) != 0 {
                        Latched
                    } else {
                        LevelSensitive
                    }) as KINTERRUPT_MODE;
                    *ShareVector = (if (*PartialDescriptor).ShareDisposition as ::core::ffi::c_int
                        == CmResourceShareShared as ::core::ffi::c_int
                    {
                        TRUE
                    } else {
                        FALSE
                    }) as UCHAR;
                    *Affinity = (*PartialDescriptor).u.Interrupt.Affinity;

                    return STATUS_SUCCESS;
                }
                _ => {}
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

    return STATUS_NOT_FOUND;
}
#[no_mangle]
pub unsafe extern "C" fn AllocateAddressMapping(
    MappedAddressList: *mut PMAPPED_ADDRESS,
    IoAddress: STOR_PHYSICAL_ADDRESS,
    MappedAddress: PVOID,
    NumberOfBytes: ULONG,
    BusNumber: ULONG,
) -> NTSTATUS {
    let mut Mapping: PMAPPED_ADDRESS;

    DPRINT1!("AllocateAddressMapping()\n");

    Mapping = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<MAPPED_ADDRESS>() as SIZE_T,
        TAG_ADDRESS_MAPPING as ULONG,
    ) as PMAPPED_ADDRESS;
    if Mapping.is_null() {
        DPRINT1!("No memory!\n");
        return STATUS_NO_MEMORY;
    }

    RtlZeroMemory!(
        Mapping as PVOID,
        core::mem::size_of::<MAPPED_ADDRESS>() as SIZE_T,
    );

    (*Mapping).NextMappedAddress = *MappedAddressList;
    *MappedAddressList = Mapping;

    (*Mapping).IoAddress = IoAddress;
    (*Mapping).MappedAddress = MappedAddress;
    (*Mapping).NumberOfBytes = NumberOfBytes;
    (*Mapping).BusNumber = BusNumber;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn SimpleTimerCallbackDpcRoutine(Dpc: PKDPC, DeferredContext: PVOID, SystemArgument1: PVOID, SystemArgument2: PVOID) {
    UNREFERENCED_PARAMETER!(Dpc);
    UNREFERENCED_PARAMETER!(SystemArgument1);
    UNREFERENCED_PARAMETER!(SystemArgument2);

    let FdoExtension: PFDO_DEVICE_EXTENSION = DeferredContext as PFDO_DEVICE_EXTENSION;

    NT_ASSERT!(!FdoExtension.is_null());
    NT_ASSERT!(!(*FdoExtension).Miniport.MiniportExtension.is_null());

    ((*FdoExtension)
        .TimerCallback
        .unwrap_unchecked())(
        core::ptr::addr_of_mut!(
            (*(*FdoExtension).Miniport.MiniportExtension).HwDeviceExtension
        ) as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "system" fn TimerCallbackDpcRoutine(Dpc: PKDPC, DeferredContext: PVOID, SystemArgument1: PVOID, SystemArgument2: PVOID) {
    UNREFERENCED_PARAMETER!(Dpc);
    UNREFERENCED_PARAMETER!(SystemArgument1);
    UNREFERENCED_PARAMETER!(SystemArgument2);

    let TimerEntry: PTIMER_ENTRY = DeferredContext as PTIMER_ENTRY;

    /*
     * Miniport can schedule a new timer event inside of callback, we should clear timer activated
     * status here to ensure timer can be correctly scheduled
     */
    (*TimerEntry).TimerAlreadySet = 0;

    ((*TimerEntry).TimerCallback.unwrap_unchecked())(
        (*TimerEntry).HwDeviceExtension,
        (*TimerEntry).Context,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GetDeviceType(InquiryData: PINQUIRYDATA) -> PCSTR {
    match ((*InquiryData).DeviceType() as ::core::ffi::c_int) as u32 {
        DIRECT_ACCESS_DEVICE => return b"Disk\0" as *const u8 as PCSTR,
        SEQUENTIAL_ACCESS_DEVICE => return b"Sequential\0" as *const u8 as PCSTR,
        PRINTER_DEVICE => return b"Printer\0" as *const u8 as PCSTR,
        PROCESSOR_DEVICE => return b"Processor\0" as *const u8 as PCSTR,
        WRITE_ONCE_READ_MULTIPLE_DEVICE => return b"Worm\0" as *const u8 as PCSTR,
        READ_ONLY_DIRECT_ACCESS_DEVICE => return b"CdRom\0" as *const u8 as PCSTR,
        SCANNER_DEVICE => return b"Scanner\0" as *const u8 as PCSTR,
        OPTICAL_DEVICE => return b"Optical\0" as *const u8 as PCSTR,
        MEDIUM_CHANGER => return b"Changer\0" as *const u8 as PCSTR,
        COMMUNICATION_DEVICE => return b"Net\0" as *const u8 as PCSTR,
        ARRAY_CONTROLLER_DEVICE => return b"Array\0" as *const u8 as PCSTR,
        SCSI_ENCLOSURE_DEVICE => return b"Enclosure\0" as *const u8 as PCSTR,
        REDUCED_BLOCK_DEVICE => return b"RBC\0" as *const u8 as PCSTR,
        OPTICAL_CARD_READER_WRITER_DEVICE => return b"CardReader\0" as *const u8 as PCSTR,
        BRIDGE_CONTROLLER_DEVICE => return b"Bridge\0" as *const u8 as PCSTR,
        _ => return b"Other\0" as *const u8 as PCSTR,
    };
}
#[no_mangle]
pub unsafe extern "C" fn GetGenericType(mut InquiryData: PINQUIRYDATA) -> PCSTR {
    match ((*InquiryData).DeviceType() as ::core::ffi::c_int) as u32 {
        DIRECT_ACCESS_DEVICE => return b"GenDisk\0" as *const u8 as PCSTR,
        PRINTER_DEVICE => return b"GenPrinter\0" as *const u8 as PCSTR,
        WRITE_ONCE_READ_MULTIPLE_DEVICE => return b"GenWorm\0" as *const u8 as PCSTR,
        READ_ONLY_DIRECT_ACCESS_DEVICE => return b"GenCdRom\0" as *const u8 as PCSTR,
        SCANNER_DEVICE => return b"GenScanner\0" as *const u8 as PCSTR,
        OPTICAL_DEVICE => return b"GenOptical\0" as *const u8 as PCSTR,
        MEDIUM_CHANGER => return b"ScsiChanger\0" as *const u8 as PCSTR,
        COMMUNICATION_DEVICE => return b"ScsiNet\0" as *const u8 as PCSTR,
        ARRAY_CONTROLLER_DEVICE => return b"ScsiArray\0" as *const u8 as PCSTR,
        SCSI_ENCLOSURE_DEVICE => return b"ScsiEnclosure\0" as *const u8 as PCSTR,
        REDUCED_BLOCK_DEVICE => return b"ScsiRBC\0" as *const u8 as PCSTR,
        OPTICAL_CARD_READER_WRITER_DEVICE => {
            return b"ScsiCardReader\0" as *const u8 as PCSTR;
        }
        BRIDGE_CONTROLLER_DEVICE => return b"ScsiBridge\0" as *const u8 as PCSTR,
        _ => return b"ScsiOther\0" as *const u8 as PCSTR,
    };
}
