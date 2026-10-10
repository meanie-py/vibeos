// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_00E24D9CDAB6: [u16; 16] = [
    0x0041u16, 0x0054, 0x0041, 0x0020, 0x0043, 0x0068, 0x0061, 0x006E, 0x006E, 0x0065,
    0x006C, 0x0020, 0x0025, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_0C2E535E467F: [u16; 4] = [0x0041u16, 0x004C, 0x0069, 0x0000];
pub static RUSTCONV_WSTR_21EDDD208D0D: [u16; 8] = [0x0057u16, 0x0069, 0x006E, 0x0042, 0x006F, 0x006E, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_243D27BD4597: [u16; 6] = [0x0049u16, 0x006E, 0x0074, 0x0065, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_39384F78B5AA: [u16; 4] = [0x0053u16, 0x0069, 0x0053, 0x0000];
pub static RUSTCONV_WSTR_3D1E9615F258: [u16; 7] = [0x0043u16, 0x006F, 0x006D, 0x0070, 0x0061, 0x0071, 0x0000];
pub static RUSTCONV_WSTR_5473B84409F2: [u16; 10] = [0x0025u16, 0x0030, 0x0034, 0x0078, 0x002D, 0x0025, 0x0030, 0x0034, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_605D248A6F22: [u16; 8] = [0x0025u16, 0x006C, 0x0073, 0x002D, 0x0025, 0x006C, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_7A763495A66E: [u16; 4] = [0x0043u16, 0x004D, 0x0044, 0x0000];
pub static RUSTCONV_WSTR_7D85A404D734: [u16; 12] = [
    0x0043u16, 0x0068, 0x0061, 0x006E, 0x006E, 0x0065, 0x006C, 0x0020, 0x0025, 0x006C,
    0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_8D44D4323B7D: [u16; 4] = [0x0025u16, 0x0068, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_9EF1A8EFD445: [u16; 6] = [0x0050u16, 0x0049, 0x0049, 0x0058, 0x0034, 0x0000];
pub static RUSTCONV_WSTR_C10F74EBA383: [u16; 9] = [0x0025u16, 0x006C, 0x0073, 0x002D, 0x0025, 0x0030, 0x0034, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_C534EDFF6A6F: [u16; 6] = [0x0050u16, 0x0049, 0x0049, 0x0058, 0x0033, 0x0000];
pub static RUSTCONV_WSTR_D217C45C6798: [u16; 4] = [0x0025u16, 0x006C, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_FD751CC90415: [u16; 5] = [0x0050u16, 0x0049, 0x0049, 0x0058, 0x0000];

use crate::prelude::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_RESOURCE_LIST, DEVICE_CAPABILITIES, DEVICE_DESCRIPTION, ExAcquireFastMutex, ExReleaseFastMutex, IO_RESOURCE_DESCRIPTOR, IO_RESOURCE_REQUIREMENTS_LIST, IoDeleteDevice, IoGetDmaAdapter, KAFFINITY, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PDEVICE_RELATIONS, PDMA_ADAPTER, PIO_RESOURCE_DESCRIPTOR, PIO_RESOURCE_REQUIREMENTS_LIST, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GUID_PCIIDE_INTERFACE_ROS: GUID = _GUID {
    Data1: 0xd677fbcf as ::core::ffi::c_uint,
    Data2: 0xabed as ::core::ffi::c_ushort,
    Data3: 0x47c8 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

unsafe extern "C" fn PciIdeXPdoFreeDmaResources(mut ChanData: PCHANNEL_DATA_COMMON) {
    if KeGetCurrentIrql() as ::core::ffi::c_int > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                17 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }
    if !(*ChanData).DmaAdapter.is_null() {
        (*ChanData).FreeMemory.unwrap_unchecked()(ChanData as PVOID);
        (*(*(*ChanData).DmaAdapter).DmaOperations)
            .PutDmaAdapter
            .unwrap_unchecked()((*ChanData).DmaAdapter);
        (*ChanData).DmaAdapter = ::core::ptr::null_mut::<_DMA_ADAPTER>();
    }
}

unsafe extern "C" fn PciIdeXPdoGetDmaAdapter(
    FdoExt: PFDO_DEVICE_EXTENSION,
    ChanData: PCHANNEL_DATA_COMMON,
) -> PDMA_ADAPTER {
    let mut DeviceDescription: DEVICE_DESCRIPTION = core::mem::zeroed();

    PAGED_CODE!();

    DeviceDescription.Version = DEVICE_DESCRIPTION_VERSION;
    DeviceDescription.Master = TRUE;
    DeviceDescription.ScatterGather = TRUE;
    DeviceDescription.Dma32BitAddresses = ((*ChanData).ChanInfo & CHANNEL_FLAG_64_BIT_DMA == 0) as i32 as u8;
    DeviceDescription.InterfaceType = PCIBus;
    DeviceDescription.MaximumLength = (*ChanData).MaximumTransferLength;

    return IoGetDmaAdapter(
        (*FdoExt).Common.__bindgen_anon_1.LowerDeviceObject,
        core::ptr::addr_of_mut!(DeviceDescription),
        core::ptr::addr_of_mut!((*ChanData).MaximumPhysicalPages),
    );
}

unsafe extern "C" fn PciIdeXPdoInitDma(
    FdoExt: PFDO_DEVICE_EXTENSION,
    ChanData: PCHANNEL_DATA_COMMON,
) -> BOOLEAN {
    let mut Status: NTSTATUS;

    (*ChanData).DmaAdapter = PciIdeXPdoGetDmaAdapter(FdoExt, ChanData);
    if (*ChanData).DmaAdapter.is_null() {
        WARN!(b"CH %lu: Unable to get DMA adapter\n\0".as_ptr() as *const i8, (*ChanData).Channel);
        return FALSE;
    }

    Status = (((*ChanData).AllocateMemory).unwrap_unchecked())((ChanData).cast());
    if !NT_SUCCESS!(Status) {
        WARN!(b"CH %lu: Unable to allocate DMA memory %lx\n\0".as_ptr() as *const i8, (*ChanData).Channel, Status);
        PciIdeXPdoFreeDmaResources(ChanData);
        return FALSE;
    }

    (*ChanData).Current.MaximumTransferLength = min(
        (*ChanData).Current.MaximumTransferLength,
        (*ChanData).MaximumPhysicalPages << PAGE_SHIFT,
    );
    INFO!(
        b"CH %lu: Allocated %lu PRD pages, maximum transfer length 0x%lx\n\0".as_ptr() as *const i8,
        (*ChanData).Channel,
        (*ChanData).MaximumPhysicalPages,
        (*ChanData).Current.MaximumTransferLength,
    );

    return TRUE;
}

unsafe extern "C" fn PciIdeXPdoStartDevice(
    PdoExt: PPDO_DEVICE_EXTENSION,
    ResourcesTranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let FdoExt: PFDO_DEVICE_EXTENSION = ((*PdoExt).Common.__bindgen_anon_1.FdoExt).cast();
    let Controller: PATA_CONTROLLER = &raw mut (*FdoExt).Controller;
    let ChanData: PCHANNEL_DATA_COMMON = (*PdoExt).ChanData;
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    if (*ChanData).MaximumTransferLength == 0 {
        (*ChanData).MaximumTransferLength = ATA_MAX_TRANSFER_LENGTH;
    }
    (*ChanData).Current.MaximumTransferLength = (*ChanData).MaximumTransferLength;
    (*ChanData).Current.TransferModeSupported = (*ChanData).TransferModeSupported;

    if (*Controller).Flags & CTRL_FLAG_IS_AHCI == 0 {
        let PataData: PCHANNEL_DATA_PATA = ChanData as PCHANNEL_DATA_PATA;

        if (*Controller).Flags & CTRL_FLAG_MANUAL_RES == 0 {
            Status = PciIdeParseResources(PataData, ResourcesTranslated);
            if !NT_SUCCESS!(Status) {
                ERR!(b"CH %lu: Failed to parse resources 0x%lx\n\0".as_ptr() as *const i8, (*ChanData).Channel, Status);
                return Status;
            }
        }

        if (*Controller).Flags & CTRL_FLAG_SATA_HBA_ACPI == 0
            && (*ChanData).ChanInfo & CHANNEL_FLAG_HAS_ACPI_GTM == 0
        {
            if AtaAcpiGetTimingMode((*PdoExt).Common.Self_, &raw mut (*PataData).CurrentTimingMode) != 0 {
                INFO!("CH %lu: ACPI available\n", (*ChanData).Channel);
                (*ChanData).ChanInfo |= CHANNEL_FLAG_HAS_ACPI_GTM;
            }
        }
    }

    if (*ChanData).Current.TransferModeSupported & !PIO_ALL != 0 {
        if PciIdeXPdoInitDma(FdoExt, ChanData) == 0 {
            (*ChanData).Current.TransferModeSupported &= PIO_ALL;
            INFO!("CH %lu: Non DMA channel\n", (*ChanData).Channel);
        }
    }

    if (*ChanData).DmaAdapter.is_null() {
        if (*ChanData).ChanInfo & CHANNEL_FLAG_PIO_VIA_DMA != 0 {
            ERR!(b"CH %lu: Unable to fall back to PIO mode\n\0".as_ptr() as *const i8, (*ChanData).Channel);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*ChanData).MaximumPhysicalPages = BYTES_TO_PAGES((*ChanData).Current.MaximumTransferLength);
    }

    if (*Controller).Flags & CTRL_FLAG_IS_AHCI == 0 {
        Status = PciIdeConnectInterrupt((ChanData).cast());
        if !NT_SUCCESS!(Status) {
            // The C passes Status with no conversion for it in the format; mirrored as-is.
            ERR!(b"CH %lu: Could not connect to interrupt\n\0".as_ptr() as *const i8, (*ChanData).Channel, Status);
            return Status;
        }

        if (*ChanData).ChanInfo & CHANNEL_FLAG_IO32 == 0 {
            INFO!("CH %lu: 32-bit I/O not supported\n", (*ChanData).Channel);
        }
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXPdoStopDevice(PdoExt: PPDO_DEVICE_EXTENSION) -> NTSTATUS {
    let ChanData: PCHANNEL_DATA_COMMON = (*PdoExt).ChanData;
    let Controller: PATA_CONTROLLER = (*ChanData).Controller;

    PAGED_CODE!();

    if (*Controller).Flags & CTRL_FLAG_IS_AHCI == 0 {
        PciIdeFreeResources((ChanData).cast());
    }

    PciIdeXPdoFreeDmaResources(ChanData);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn PciIdeXPdoRemoveDevice(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
    FinalRemove: BOOLEAN,
) -> NTSTATUS {
    let FdoExtension: PFDO_DEVICE_EXTENSION = ((*PdoExtension).Common.__bindgen_anon_1.FdoExt).cast();
    let mut Status: NTSTATUS;
    let mut WasRemoved: BOOLEAN = FALSE;

    PAGED_CODE!();

    PciIdeXPdoStopDevice(PdoExtension);

    if FinalRemove != 0 && ((*PdoExtension).Flags & PDO_FLAG_REPORTED_MISSING) != 0 {
        IoReleaseRemoveLockAndWait(&raw mut (*PdoExtension).Common.RemoveLock, (Irp).cast());

        if !Irp.is_null() {
            ExAcquireFastMutex(&raw mut (*FdoExtension).PdoListSyncMutex);
        }

        RemoveEntryList(&raw mut (*PdoExtension).ListEntry);

        if !Irp.is_null() {
            ExReleaseFastMutex(&raw mut (*FdoExtension).PdoListSyncMutex);
        }

        IoDeleteDevice((*PdoExtension).Common.Self_);
        WasRemoved = TRUE;
    }

    Status = STATUS_SUCCESS;
    if !Irp.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));
    }

    if WasRemoved == 0 {
        IoReleaseRemoveLock(&raw mut (*PdoExtension).Common.RemoveLock, (Irp).cast());
    }
    return Status;
}

unsafe extern "C" fn PciIdeXPdoQueryStopRemoveDevice(
    mut PdoExtension: PPDO_DEVICE_EXTENSION,
) -> NTSTATUS {
    if KeGetCurrentIrql() as ::core::ffi::c_int > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/storage/ide/pciidex/pdo.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                240 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }
    if (*PdoExtension).Common.PageFiles != 0
        || (*PdoExtension).Common.HibernateFiles != 0
        || (*PdoExtension).Common.DumpFiles != 0
    {
        return STATUS_DEVICE_BUSY;
    }
    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXPdoQueryTargetDeviceRelations(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceRelations: PDEVICE_RELATIONS;

    PAGED_CODE!();

    DeviceRelations = ExAllocatePoolUninitialized(
        PagedPool,
        core::mem::size_of::<DEVICE_RELATIONS>(),
        TAG_PCIIDEX,
    ) as PDEVICE_RELATIONS;
    if DeviceRelations.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DeviceRelations).Count = 1;
    *((*DeviceRelations).Objects.as_mut_ptr().add(0)) =
        (*PdoExtension).Common.Self_;
    ObReferenceObject(((*PdoExtension).Common.Self_).cast());

    (*Irp).IoStatus.Information = DeviceRelations as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXPdoQueryCapabilities(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut FdoExt: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Controller: PATA_CONTROLLER = core::ptr::null_mut();
    let mut ParentCapabilities: DEVICE_CAPABILITIES = core::mem::zeroed();
    let mut DeviceCapabilities: PDEVICE_CAPABILITIES = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    PAGED_CODE!();

    RtlZeroMemory!(
        &mut ParentCapabilities as *mut DEVICE_CAPABILITIES as *mut core::ffi::c_void,
        core::mem::size_of::<DEVICE_CAPABILITIES>(),
    );
    ParentCapabilities.Size = core::mem::size_of::<DEVICE_CAPABILITIES>() as u16;
    ParentCapabilities.Version = 1;
    ParentCapabilities.Address = MAXULONG;
    ParentCapabilities.UINumber = MAXULONG;
    Status = PciIdeXPnpRepeatRequest(
        &mut (*PdoExtension).Common,
        Irp,
        &mut ParentCapabilities,
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    DeviceCapabilities = (*IoStack).Parameters.DeviceCapabilities.Capabilities;
    *DeviceCapabilities = ParentCapabilities;

    (*DeviceCapabilities).set_UniqueID((FALSE) as _);
    (*DeviceCapabilities).Address = (*PdoExtension).Channel;

    FdoExt = ((*PdoExtension).Common.__bindgen_anon_1.FdoExt).cast();
    Controller = &mut (*FdoExt).Controller;
    if (*Controller).Flags & CTRL_FLAG_SATA_HBA_ACPI != 0 {
        (*DeviceCapabilities).Address <<= 16;
        (*DeviceCapabilities).Address |= 0xFFFF;
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXMakePortResource(
    Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    IoBase: u32,
    Length: u32,
) {
    (*Descriptor).Type = CmResourceTypePort;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
    (*Descriptor).Flags = (CM_RESOURCE_PORT_IO | CM_RESOURCE_PORT_16_BIT_DECODE) as u16;
    (*Descriptor).u.Port.Length = Length;
    (*Descriptor).u.Port.Start.__bindgen_anon_1.LowPart = IoBase;
}

unsafe extern "C" fn PciIdeXMakePortRequirement(
    mut Descriptor: PIO_RESOURCE_DESCRIPTOR,
    mut IoBase: u32,
    mut Length: u32,
) {
    (*Descriptor).Type = CmResourceTypePort;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
    (*Descriptor).Flags = (CM_RESOURCE_PORT_IO | CM_RESOURCE_PORT_16_BIT_DECODE) as u16;
    (*Descriptor).u.Port.Length = Length;
    (*Descriptor).u.Port.Alignment = 1;
    (*Descriptor).u.Port.MinimumAddress.__bindgen_anon_1.LowPart = IoBase;
    (*Descriptor).u.Port.MaximumAddress.__bindgen_anon_1.LowPart = IoBase.wrapping_add(Length).wrapping_sub(1);
}

unsafe extern "C" fn PciIdeXPdoQueryResources(
    mut PdoExtension: PPDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut FdoExtension: PFDO_DEVICE_EXTENSION = ((*PdoExtension).Common.__bindgen_anon_1.FdoExt).cast();
    let mut Controller: PATA_CONTROLLER = &raw mut (*FdoExtension).Controller;
    let mut ChannelState: IDE_CHANNEL_STATE;
    let mut ResourceList: PCM_RESOURCE_LIST = core::ptr::null_mut();
    let mut Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut CommandPortBase: ULONG = 0;
    let mut ControlPortBase: ULONG = 0;
    let mut InterruptVector: ULONG = 0;
    let mut ListSize: ULONG = 0;
    let mut AppendSecondaryResources: BOOLEAN = 0;

    PAGED_CODE!();

    if (*Controller).Flags & (CTRL_FLAG_NATIVE_PCI | CTRL_FLAG_NON_PNP) != 0 {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    ChannelState = PciIdeXGetChannelState(Controller, (*PdoExtension).Channel);
    if ChannelState == ChannelDisabled {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    AppendSecondaryResources = ((*Controller).MaxChannels == 1) as BOOLEAN;

    ListSize = (FIELD_OFFSET!(CM_RESOURCE_LIST, List[0].PartialResourceList.PartialDescriptors)
        + (core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() * (PCIIDE_LEGACY_RESOURCE_COUNT as usize)) as i32) as ULONG;
    if AppendSecondaryResources != 0 {
        ListSize = (ListSize as usize
            + core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() * 2) as ULONG;
    }
    ResourceList = ExAllocatePoolZero(PagedPool, (ListSize) as _, TAG_PCIIDEX) as PCM_RESOURCE_LIST;
    if ResourceList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*ResourceList).Count = 1;
    (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR).offset(0 as isize))
        .InterfaceType = Isa;
    (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR).offset(0 as isize))
        .PartialResourceList.Version = 1;
    (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR).offset(0 as isize))
        .PartialResourceList.Revision = 1;
    (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR).offset(0 as isize))
        .PartialResourceList.Count = PCIIDE_LEGACY_RESOURCE_COUNT;
    if AppendSecondaryResources != 0 {
        (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR).offset(0 as isize))
            .PartialResourceList.Count += 2;
    }

    if IS_PRIMARY_CHANNEL!(PdoExtension) {
        CommandPortBase = PCIIDE_LEGACY_PRIMARY_COMMAND_BASE;
        ControlPortBase = PCIIDE_LEGACY_PRIMARY_CONTROL_BASE;
        InterruptVector = PCIIDE_LEGACY_PRIMARY_IRQ;
    } else {
        CommandPortBase = PCIIDE_LEGACY_SECONDARY_COMMAND_BASE;
        ControlPortBase = PCIIDE_LEGACY_SECONDARY_CONTROL_BASE;
        InterruptVector = PCIIDE_LEGACY_SECONDARY_IRQ;
    }

    Descriptor = (&raw mut (*(&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
        .offset(0 as isize))
        .PartialResourceList.PartialDescriptors as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
        .offset(0 as isize) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

    let fresh0 = Descriptor;
    Descriptor = Descriptor.offset(1);
    PciIdeXMakePortResource(fresh0, CommandPortBase, PCIIDE_LEGACY_COMMAND_IO_RANGE_LENGTH);

    let fresh1 = Descriptor;
    Descriptor = Descriptor.offset(1);
    PciIdeXMakePortResource(fresh1, ControlPortBase, PCIIDE_LEGACY_CONTROL_IO_RANGE_LENGTH);

    (*Descriptor).Type = CmResourceTypeInterrupt;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
    (*Descriptor).Flags = (CM_RESOURCE_INTERRUPT_LATCHED as u16);
    (*Descriptor).u.Interrupt.Level = InterruptVector;
    (*Descriptor).u.Interrupt.Vector = InterruptVector;
    (*Descriptor).u.Interrupt.Affinity = (0 as KAFFINITY).wrapping_sub(1);

    if AppendSecondaryResources != 0 {
        Descriptor = Descriptor.offset(1);
        PciIdeXMakePortResource(Descriptor, PCIIDE_LEGACY_SECONDARY_COMMAND_BASE, PCIIDE_LEGACY_COMMAND_IO_RANGE_LENGTH);

        Descriptor = Descriptor.offset(1);
        PciIdeXMakePortResource(Descriptor, PCIIDE_LEGACY_SECONDARY_CONTROL_BASE, PCIIDE_LEGACY_CONTROL_IO_RANGE_LENGTH);
    }

    (*Irp).IoStatus.Information = ResourceList as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXPdoQueryResourceRequirements(
    mut PdoExtension: PPDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut FdoExtension: PFDO_DEVICE_EXTENSION = ((*PdoExtension).Common.__bindgen_anon_1.FdoExt).cast();
    let mut Controller: PATA_CONTROLLER = &raw mut (*FdoExtension).Controller;
    let mut RequirementsList: PIO_RESOURCE_REQUIREMENTS_LIST = core::ptr::null_mut();
    let mut Descriptor: PIO_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut ChannelState: IDE_CHANNEL_STATE = 0;
    let mut CommandPortBase: ULONG = 0;
    let mut ControlPortBase: ULONG = 0;
    let mut InterruptVector: ULONG = 0;
    let mut ListSize: ULONG = 0;
    let mut AppendSecondaryResources: BOOLEAN = 0;

    PAGED_CODE!();

    if (*Controller).Flags & (CTRL_FLAG_NATIVE_PCI | CTRL_FLAG_NON_PNP) != 0 {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    ChannelState = PciIdeXGetChannelState(Controller, (*PdoExtension).Channel);
    if ChannelState == ChannelDisabled {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    AppendSecondaryResources = ((*Controller).MaxChannels == 1) as BOOLEAN;

    ListSize = (FIELD_OFFSET!(IO_RESOURCE_REQUIREMENTS_LIST, List[0].Descriptors)
        + (core::mem::size_of::<IO_RESOURCE_DESCRIPTOR>() * (PCIIDE_LEGACY_RESOURCE_COUNT as usize)) as i32) as ULONG;
    if AppendSecondaryResources != 0 {
        ListSize = (ListSize as usize
            + core::mem::size_of::<IO_RESOURCE_DESCRIPTOR>() * 2) as ULONG;
    }

    RequirementsList = ExAllocatePoolZero(
        PagedPool,
        ListSize as usize,
        TAG_PCIIDEX,
    ) as PIO_RESOURCE_REQUIREMENTS_LIST;
    if RequirementsList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*RequirementsList).InterfaceType = Isa;
    (*RequirementsList).ListSize = ListSize;
    (*RequirementsList).AlternativeLists = 1;
    (*(&raw mut (*RequirementsList).List as *mut IO_RESOURCE_LIST)
        .offset(0 as isize))
    .Version = 1;
    (*(&raw mut (*RequirementsList).List as *mut IO_RESOURCE_LIST)
        .offset(0 as isize))
    .Revision = 1;
    (*(&raw mut (*RequirementsList).List as *mut IO_RESOURCE_LIST)
        .offset(0 as isize))
    .Count = PCIIDE_LEGACY_RESOURCE_COUNT;
    if AppendSecondaryResources != 0 {
        let ref mut fresh = (*(&raw mut (*RequirementsList).List as *mut IO_RESOURCE_LIST)
            .offset(0 as isize))
        .Count;
        *fresh = (*fresh).wrapping_add(2);
    }

    if IS_PRIMARY_CHANNEL!(PdoExtension) {
        CommandPortBase = PCIIDE_LEGACY_PRIMARY_COMMAND_BASE;
        ControlPortBase = PCIIDE_LEGACY_PRIMARY_CONTROL_BASE;
        InterruptVector = PCIIDE_LEGACY_PRIMARY_IRQ;
    } else {
        CommandPortBase = PCIIDE_LEGACY_SECONDARY_COMMAND_BASE;
        ControlPortBase = PCIIDE_LEGACY_SECONDARY_CONTROL_BASE;
        InterruptVector = PCIIDE_LEGACY_SECONDARY_IRQ;
    }

    Descriptor = (&raw mut (*(&raw mut (*RequirementsList).List as *mut IO_RESOURCE_LIST)
        .offset(0 as isize))
    .Descriptors as *mut IO_RESOURCE_DESCRIPTOR)
        .offset(0 as isize) as PIO_RESOURCE_DESCRIPTOR;

    let fresh = Descriptor;
    Descriptor = Descriptor.offset(1);
    PciIdeXMakePortRequirement(fresh, CommandPortBase, PCIIDE_LEGACY_COMMAND_IO_RANGE_LENGTH);

    let fresh = Descriptor;
    Descriptor = Descriptor.offset(1);
    PciIdeXMakePortRequirement(fresh, ControlPortBase, PCIIDE_LEGACY_CONTROL_IO_RANGE_LENGTH);

    (*Descriptor).Type = CmResourceTypeInterrupt;
    (*Descriptor).ShareDisposition = (CmResourceShareShared) as u8;
    (*Descriptor).Flags = (CM_RESOURCE_INTERRUPT_LATCHED as u16);
    (*Descriptor).u.Interrupt.MinimumVector = InterruptVector;
    (*Descriptor).u.Interrupt.MaximumVector = InterruptVector;

    if AppendSecondaryResources != 0 {
        Descriptor = Descriptor.offset(1);
        PciIdeXMakePortRequirement(Descriptor, PCIIDE_LEGACY_SECONDARY_COMMAND_BASE, PCIIDE_LEGACY_COMMAND_IO_RANGE_LENGTH);

        Descriptor = Descriptor.offset(1);
        PciIdeXMakePortRequirement(Descriptor, PCIIDE_LEGACY_SECONDARY_CONTROL_BASE, PCIIDE_LEGACY_CONTROL_IO_RANGE_LENGTH);
    }

    (*Irp).IoStatus.Information = RequirementsList as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXGetControllerVendorId(
    FdoExtension: PFDO_DEVICE_EXTENSION,
) -> PCWSTR {
    match (*FdoExtension).Controller.Pci.VendorID {
        0x0E11 => return RUSTCONV_WSTR_3D1E9615F258.as_ptr(),
        0x1039 => return RUSTCONV_WSTR_39384F78B5AA.as_ptr(),
        0x1050 => return RUSTCONV_WSTR_21EDDD208D0D.as_ptr(),
        0x1095 => return RUSTCONV_WSTR_7A763495A66E.as_ptr(),
        0x10B9 => return RUSTCONV_WSTR_0C2E535E467F.as_ptr(),
        0x8086 => return RUSTCONV_WSTR_243D27BD4597.as_ptr(),
        _ => {}
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn PciIdeXGetControllerDeviceId(
    FdoExtension: PFDO_DEVICE_EXTENSION,
) -> PCWSTR {
    if (*FdoExtension).Controller.Pci.VendorID == 0x8086 {
        match (*FdoExtension).Controller.Pci.DeviceID {
            0x1230 => {
                return RUSTCONV_WSTR_FD751CC90415.as_ptr();
            }
            0x7010 => {
                return RUSTCONV_WSTR_C534EDFF6A6F.as_ptr();
            }
            0x7111 => {
                return RUSTCONV_WSTR_9EF1A8EFD445.as_ptr();
            }
            _ => {}
        }
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn PciIdeXPdoQueryId(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut Buffer: PWCHAR;
    let mut End: PWCHAR;
    let mut CharCount: usize;
    let mut Remaining: usize;
    static IdeCompatibleId: [u16; 9] = [0x002Au16, 0x0050, 0x004E, 0x0050, 0x0030, 0x0036, 0x0030, 0x0030, 0x0000];

    PAGED_CODE!();

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).Parameters.QueryId.IdType {
        BusQueryDeviceID => {
            static PciIdeDeviceId: [u16; 18] = [
    0x0050u16, 0x0043, 0x0049, 0x0049, 0x0044, 0x0045, 0x005C, 0x0049, 0x0044, 0x0045,
    0x0043, 0x0068, 0x0061, 0x006E, 0x006E, 0x0065, 0x006C, 0x0000,
];

            Buffer = ExAllocatePoolUninitialized(
                PagedPool,
                core::mem::size_of::<[u16; 18]>(),
                TAG_PCIIDEX,
            ) as PWCHAR;
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            RtlCopyMemory!(
                Buffer as *mut _,
                PciIdeDeviceId.as_ptr() as *const _,
                core::mem::size_of::<[u16; 18]>(),
            );

            INFO!("Device ID: '%S'\n", Buffer);
        }

        BusQueryHardwareIDs => {
            let mut FdoExtension: PFDO_DEVICE_EXTENSION;
            let mut IdStart: PWCHAR = core::ptr::null_mut();

            DBG_UNREFERENCED_LOCAL_VARIABLE!(IdStart);

            CharCount = core::mem::size_of::<[u8; 13]>()
                + core::mem::size_of::<[u8; 21]>()
                + RTL_NUMBER_OF!(IdeCompatibleId) as usize
                + core::mem::size_of::<u8>();

            Buffer = ExAllocatePoolUninitialized(
                PagedPool,
                CharCount * core::mem::size_of::<u16>(),
                TAG_PCIIDEX,
            ) as PWCHAR;
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            INFO!("HardwareIDs:\n");

            FdoExtension = ((*PdoExtension).Common.__bindgen_anon_1.FdoExt).cast();
            End = Buffer;
            Remaining = CharCount;

            if ((*FdoExtension).Controller.Flags & CTRL_FLAG_NON_PNP) == 0 {
                let VendorString: PCWSTR = PciIdeXGetControllerVendorId(FdoExtension);

                if !VendorString.is_null() {
                    let DeviceString: PCWSTR = PciIdeXGetControllerDeviceId(FdoExtension);

                    if !DeviceString.is_null() {
                        Status = RtlStringCchPrintfExW!(
                            End,
                            Remaining,
                            &mut End,
                            &mut Remaining,
                            0,
                            RUSTCONV_WSTR_605D248A6F22.as_ptr(),
                            VendorString,
                            DeviceString,
                        );
                    } else {
                        Status = RtlStringCchPrintfExW!(
                            End,
                            Remaining,
                            &mut End,
                            &mut Remaining,
                            0,
                            RUSTCONV_WSTR_C10F74EBA383.as_ptr(),
                            VendorString,
                            (*FdoExtension).Controller.Pci.DeviceID as core::ffi::c_uint,
                        );
                    }
                } else {
                    Status = RtlStringCchPrintfExW!(
                        End,
                        Remaining,
                        &mut End,
                        &mut Remaining,
                        0,
                        RUSTCONV_WSTR_5473B84409F2.as_ptr(),
                        (*FdoExtension).Controller.Pci.VendorID as core::ffi::c_uint,
                        (*FdoExtension).Controller.Pci.DeviceID as core::ffi::c_uint,
                    );
                }

                INFO!("  '%S'\n", Buffer);

                End = End.add(1);
                Remaining -= 1;
            }

            IdStart = End;
            Status = RtlStringCchPrintfExW!(
                End,
                Remaining,
                &mut End,
                &mut Remaining,
                0,
                RUSTCONV_WSTR_8D44D4323B7D.as_ptr(),
                b"Internal_IDE_Channel\0".as_ptr() as *const i8,
            );

            INFO!("  '%S'\n", IdStart);

            End = End.add(1);
            Remaining -= 1;

            IdStart = End;
            Status = RtlStringCchPrintfExW!(
                End,
                Remaining,
                &mut End,
                &mut Remaining,
                0,
                RUSTCONV_WSTR_D217C45C6798.as_ptr(),
                IdeCompatibleId.as_ptr(),
            );

            INFO!("  '%S'\n", IdStart);

            End = End.add(1);
            *End = UNICODE_NULL;
        }

        BusQueryCompatibleIDs => {
            Buffer = ExAllocatePoolUninitialized(
                PagedPool,
                core::mem::size_of::<[u16; 9]>() + core::mem::size_of_val(&UNICODE_NULL),
                TAG_PCIIDEX,
            ) as PWCHAR;
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            RtlCopyMemory!(
                Buffer as *mut _,
                IdeCompatibleId.as_ptr() as *const _,
                core::mem::size_of::<[u16; 9]>(),
            );

            *Buffer.add(core::mem::size_of::<[u16; 9]>() / core::mem::size_of::<u16>()) =
                UNICODE_NULL;

            INFO!("Compatible ID: '%S'\n", Buffer);
        }

        BusQueryInstanceID => {
            CharCount = core::mem::size_of::<[u8; 3]>();

            Buffer = ExAllocatePoolUninitialized(
                PagedPool,
                CharCount * core::mem::size_of::<u16>(),
                TAG_PCIIDEX,
            ) as PWCHAR;
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            Status = RtlStringCchPrintfExW!(
                Buffer,
                CharCount,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
                (*PdoExtension).Channel,
            );

            INFO!("Instance ID: '%S'\n", Buffer);
        }

        _ => return (*Irp).IoStatus.__bindgen_anon_1.Status,
    }

    (*Irp).IoStatus.Information = Buffer as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXPdoQueryDeviceText(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let Status: NTSTATUS;
    let Buffer: PWCHAR;
    let CharCount: usize;

    PAGED_CODE!();

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).Parameters.QueryDeviceText.DeviceTextType {
        DeviceTextDescription => {
            CharCount = core::mem::size_of::<[u8; 15]>();

            Buffer = (ExAllocatePoolUninitialized(
                PagedPool,
                CharCount * core::mem::size_of::<u16>(),
                TAG_PCIIDEX,
            )).cast();
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            Status = RtlStringCchPrintfExW!(
                Buffer,
                CharCount,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                RUSTCONV_WSTR_00E24D9CDAB6.as_ptr(),
                (*PdoExtension).Channel,
            );

            INFO!("Device Description: '%S'\n\0".as_ptr() as *const i8, Buffer);
        }
        DeviceTextLocationInformation => {
            CharCount = core::mem::size_of::<[u8; 11]>();

            Buffer = (ExAllocatePoolUninitialized(
                PagedPool,
                CharCount * core::mem::size_of::<u16>(),
                TAG_PCIIDEX,
            )).cast();
            if Buffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            Status = RtlStringCchPrintfExW!(
                Buffer,
                CharCount,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                RUSTCONV_WSTR_7D85A404D734.as_ptr(),
                (*PdoExtension).Channel,
            );

            INFO!("Device Location: '%S'\n\0".as_ptr() as *const i8, Buffer);
        }
        _ => {
            return (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    (*Irp).IoStatus.Information = Buffer as ULONG_PTR;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXQueryPciIdeInterface(
    mut PdoExt: PPDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
    mut IoStack: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let mut FdoExt: PFDO_DEVICE_EXTENSION = ((*PdoExt).Common.__bindgen_anon_1.FdoExt).cast();
    let mut Controller: PATA_CONTROLLER = &raw mut (*FdoExt).Controller;
    let mut ChanData: PCHANNEL_DATA_COMMON;
    let mut ChannelInterface: PPCIIDEX_CHANNEL_INTERFACE;

    PAGED_CODE!();

    if ((*IoStack).Parameters.QueryInterface.Size as usize)
        < core::mem::size_of::<PCIIDEX_CHANNEL_INTERFACE>() as usize
    {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    if (*IoStack).Parameters.QueryInterface.Version as i32 != (PCIIDEX_INTERFACE_VERSION as i32) {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    ChanData = (*PdoExt).ChanData;

    ChannelInterface = (*IoStack).Parameters.QueryInterface.Interface as PPCIIDEX_CHANNEL_INTERFACE;
    (*ChannelInterface).AttachChannel = Some(AtaCtrlAttachChannel);
    (*ChannelInterface).ChannelContext = ChanData as PVOID;
    (*ChannelInterface).Channel = (*ChanData).Channel;
    (*ChannelInterface).MaximumTransferLength = (*ChanData).Current.MaximumTransferLength;
    (*ChannelInterface).TransferModeSupported = (*ChanData).Current.TransferModeSupported;
    (*ChannelInterface).SetTransferMode = Some(AtaCtrlSetTransferMode);
    (*ChannelInterface).SetDeviceData = Some(AtaCtrlSetDeviceData);
    (*ChannelInterface).GetInitTaskFile = Some(AtaCtrlGetInitTaskFile);
    (*ChannelInterface).DowngradeInterfaceSpeed = Some(AtaCtrlDowngradeInterfaceSpeed);
    (*ChannelInterface).PreparePrdTable = (*ChanData).PreparePrdTable;
    (*ChannelInterface).PrepareIo = (*ChanData).PrepareIo;
    (*ChannelInterface).StartIo = (*ChanData).StartIo;
    (*ChannelInterface).MaximumPhysicalPages = (*ChanData).MaximumPhysicalPages;
    (*ChannelInterface).QueueDepth = (*Controller).QueueDepth;
    (*ChannelInterface).DmaAdapter = (*ChanData).DmaAdapter;
    (*ChannelInterface).ChannelObject = (*PdoExt).Common.Self_;
    (*ChannelInterface).AbortChannel = Some(AtaCtrlAbortChannel);

    if (*Controller).Flags & CTRL_FLAG_IS_AHCI != 0 {
        (*ChannelInterface).AllocateSlot = Some(AtaAhciAllocateSlot);
        (*ChannelInterface).ResetChannel = Some(AtaAhciResetChannel);
        (*ChannelInterface).EnumerateChannel = Some(AtaAhciEnumerateChannel);
        (*ChannelInterface).IdentifyDevice = Some(AtaAhciIdentifyDevice);
        (*ChannelInterface).MaxTargetId = AtaAhciChannelGetMaximumDeviceCount((ChanData).cast());
    } else {
        (*ChannelInterface).AllocateSlot = Some(PataAllocateSlot);
        (*ChannelInterface).ResetChannel = Some(PataResetChannel);
        (*ChannelInterface).EnumerateChannel = Some(PataEnumerateChannel);
        (*ChannelInterface).IdentifyDevice = Some(PataIdentifyDevice);
        (*ChannelInterface).MaxTargetId = PataChannelGetMaximumDeviceCount((ChanData).cast());
    }

    (*ChannelInterface).PortContext = &raw mut (*ChanData).PortContext;
    (*ChannelInterface).PortNotification = &raw mut (*ChanData).PortNotification;
    (*ChannelInterface).Slots = &raw mut (*ChanData).Slots;

    if (*Controller).Flags & CTRL_FLAG_IS_AHCI != 0 {
        (*ChannelInterface).InterruptObject = (*Controller).InterruptObject;
    } else {
        (*ChannelInterface).InterruptObject = (*ChanData).InterruptObject;
    }

    if (*ChanData).ChanInfo & CHANNEL_FLAG_PIO_VIA_DMA != 0 {
        (*ChannelInterface).Flags |= ATA_CHANNEL_FLAG_PIO_VIA_DMA;
    }

    if (*ChanData).ChanInfo & CHANNEL_FLAG_IS_EXTERNAL != 0 {
        (*ChannelInterface).Flags |= ATA_CHANNEL_FLAG_IS_EXTERNAL;
    }

    if (*ChanData).ChanInfo & CHANNEL_FLAG_HAS_NCQ != 0 {
        (*ChannelInterface).Flags |= ATA_CHANNEL_FLAG_NCQ;
    }

    if (*Controller).Flags & CTRL_FLAG_IS_AHCI != 0 {
        (*ChannelInterface).Flags |= ATA_CHANNEL_FLAG_IS_AHCI;
    }

    if (*ChanData).ChanInfo & CHANNEL_FLAG_PIO_FOR_LBA48_XFER != 0 {
        (*ChannelInterface).Flags |= ATA_CHANNEL_FLAG_PIO_FOR_LBA48_XFER;
    }

    if (*Controller).Flags & CTRL_FLAG_IS_SIMPLEX != 0 {
        (*ChannelInterface).HwSyncObject = (*Controller).HwSyncObject;
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn PciIdeXPdoQueryInterface(
    mut PdoExtension: PPDO_DEVICE_EXTENSION,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();

    PAGED_CODE!();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if IsEqualGUIDAligned(
        (*IoStack).Parameters.QueryInterface.InterfaceType,
        &GUID_PCIIDE_INTERFACE_ROS,
    ) != false {
        Status = PciIdeXQueryPciIdeInterface(PdoExtension, Irp, IoStack);
    } else {
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    return Status;
}

unsafe extern "C" fn PciIdeXPdoDispatchPnp(
    PdoExtension: PPDO_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let IoStack: PIO_STACK_LOCATION;

    PAGED_CODE!();

    Status = IoAcquireRemoveLock(&mut (*PdoExtension).Common.RemoveLock, (Irp).cast());
    if !NT_SUCCESS!(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

        return Status;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).MinorFunction {
        IRP_MN_START_DEVICE => {
            Status = PciIdeXPdoStartDevice(
                PdoExtension,
                (*IoStack).Parameters.StartDevice.AllocatedResourcesTranslated,
            );
        }
        IRP_MN_STOP_DEVICE => {
            Status = PciIdeXPdoStopDevice(PdoExtension);
        }
        IRP_MN_QUERY_STOP_DEVICE | IRP_MN_QUERY_REMOVE_DEVICE => {
            Status = PciIdeXPdoQueryStopRemoveDevice(PdoExtension);
        }
        IRP_MN_CANCEL_REMOVE_DEVICE | IRP_MN_CANCEL_STOP_DEVICE => {
            Status = STATUS_SUCCESS;
        }
        IRP_MN_SURPRISE_REMOVAL | IRP_MN_REMOVE_DEVICE => {
            return PciIdeXPdoRemoveDevice(
                PdoExtension,
                Irp,
                ((*IoStack).MinorFunction == IRP_MN_REMOVE_DEVICE) as u8,
            );
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            if (*IoStack).Parameters.QueryDeviceRelations.Type == TargetDeviceRelation {
                Status = PciIdeXPdoQueryTargetDeviceRelations(PdoExtension, Irp);
            } else {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            }
        }
        IRP_MN_QUERY_CAPABILITIES => {
            Status = PciIdeXPdoQueryCapabilities(PdoExtension, Irp);
        }
        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            Status = PciIdeXPnpQueryPnpDeviceState(&mut (*PdoExtension).Common, Irp);
        }
        IRP_MN_QUERY_RESOURCES => {
            Status = PciIdeXPdoQueryResources(PdoExtension, Irp);
        }
        IRP_MN_QUERY_RESOURCE_REQUIREMENTS => {
            Status = PciIdeXPdoQueryResourceRequirements(PdoExtension, Irp);
        }
        IRP_MN_QUERY_ID => {
            Status = PciIdeXPdoQueryId(PdoExtension, Irp);
        }
        IRP_MN_QUERY_DEVICE_TEXT => {
            Status = PciIdeXPdoQueryDeviceText(PdoExtension, Irp);
        }
        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            Status = PciIdeXPnpQueryDeviceUsageNotification(&mut (*PdoExtension).Common, Irp);
        }
        IRP_MN_QUERY_INTERFACE => {
            Status = PciIdeXPdoQueryInterface(PdoExtension, Irp);
        }
        _ => {
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT as i8));

    IoReleaseRemoveLock(&mut (*PdoExtension).Common.RemoveLock, (Irp).cast());

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXDispatchPnp(DeviceObject: *mut _DEVICE_OBJECT, Irp: *mut _IRP) -> NTSTATUS {
    PAGED_CODE!();

    if IS_FDO!((*DeviceObject).DeviceExtension) != false {
        return PciIdeXFdoDispatchPnp(((*DeviceObject).DeviceExtension).cast(), Irp);
    } else {
        return PciIdeXPdoDispatchPnp(((*DeviceObject).DeviceExtension).cast(), Irp);
    }
}
