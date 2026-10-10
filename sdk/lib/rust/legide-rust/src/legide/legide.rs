// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_6CC06C2CA9FD: [u16; 22] = [
    0x0044u16, 0x0069, 0x0073, 0x0061, 0x0062, 0x006C, 0x0065, 0x0046, 0x0069, 0x0072,
    0x006D, 0x0077, 0x0061, 0x0072, 0x0065, 0x004D, 0x0061, 0x0070, 0x0070, 0x0065,
    0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_9EADC9D3A146: [u16; 11] = [
    0x0050u16, 0x0061, 0x0072, 0x0061, 0x006D, 0x0065, 0x0074, 0x0065, 0x0072, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_D1C33AAC335D: [u16; 16] = [
    0x004Cu16, 0x0065, 0x0067, 0x0061, 0x0063, 0x0079, 0x0044, 0x0065, 0x0074, 0x0065,
    0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_RESOURCE_LIST, HalGetInterruptVector, HalTranslateBusAddress, IoGetConfigurationInformation, IoReportDetectedDevice, IoReportResourceForDetection, KAFFINITY, KeStallExecutionProcessor, MmMapIoSpace, MmUnmapIoSpace, _OBJECT_ATTRIBUTES, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PCONFIGURATION_INFORMATION, PDEVICE_OBJECT, PDRIVER_OBJECT, PHANDLE, PHYSICAL_ADDRESS, PUNICODE_STRING, RTL_QUERY_REGISTRY_TABLE, RtlQueryRegistryValues, RtlWriteRegistryValue, ZwClose, ZwOpenKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

pub const ATAPORT_TAG: u32 = 0x50656449 /* 'PedI' */ as u32;

pub const IDE_DRIVE_SELECT: u32 = 0xA0;

pub const ATA_TIME_BUSY_SELECT: u32 = 3000; // nolint: const-contradiction (legide.c: 30 ms; freeldr hwidep.h has its own 20 ms)

pub const ATA_TIME_BUSY_IDENTIFY: u32 = 500000;

pub const ATA_TIME_DRQ_CLEAR: u32 = 1000;

pub const ATA_TIME_RESET_SELECT: u32 = 200000;

pub const ATA_TIME_BUSY_RESET: u32 = 1000000;

#[repr(C)]
pub struct _ATA_LEGACY_CHANNEL
{
    pub IoBase: u32,
    pub Irq: u32,
}
pub type ATA_LEGACY_CHANNEL = _ATA_LEGACY_CHANNEL;
pub type PATA_LEGACY_CHANNEL = *mut _ATA_LEGACY_CHANNEL;

unsafe extern "C" fn AtaWait(
    Registers: PIDE_REGISTERS,
    Timeout: ULONG,
    Mask: UCHAR,
    Value: UCHAR,
) -> UCHAR {
    let mut IdeStatus: UCHAR = core::mem::zeroed();
    let mut i: ULONG;

    i = 0;
    while i < Timeout {
        IdeStatus = ATA_READ!((*Registers).__bindgen_anon_5.Status);
        if (IdeStatus & Mask) == Value {
            break;
        }

        if IdeStatus == 0xFF {
            break;
        }

        KeStallExecutionProcessor(10);
        i += 1;
    }

    return IdeStatus;
}

unsafe extern "C" fn AtaLegacyFetchIdentifyData(Registers: PIDE_REGISTERS) {
    let mut Buffer: [USHORT; 256] = [0; 256];
    READ_PORT_BUFFER_USHORT(
        (*Registers).Data as PUSHORT,
        Buffer.as_mut_ptr() as PUSHORT,
        RTL_NUMBER_OF!(Buffer),
    );
}

unsafe extern "C" fn AtaLegacyIdentifyDevice(
    Registers: PIDE_REGISTERS,
    DeviceNumber: ULONG,
    Command: UCHAR,
) -> BOOLEAN {
    let mut IdeStatus: UCHAR;

    ATA_WRITE!(
        (*Registers).Device,
        ((DeviceNumber << 4) | IDE_DRIVE_SELECT) as UCHAR,
    );
    ATA_IO_WAIT!();
    IdeStatus = AtaWait(
        Registers,
        ATA_TIME_BUSY_SELECT,
        (IDE_STATUS_BUSY) as UCHAR,
        0,
    );
    if IdeStatus & (IDE_STATUS_BUSY as u8) != 0 {
        DPRINT!("Timeout, status 0x%02x\n", IdeStatus);
        return FALSE;
    }

    ATA_WRITE!((*Registers).__bindgen_anon_1.Features, 0);
    ATA_WRITE!((*Registers).__bindgen_anon_2.SectorCount, 0);
    ATA_WRITE!((*Registers).LbaLow, 0);
    ATA_WRITE!((*Registers).__bindgen_anon_3.LbaMid, 0);
    ATA_WRITE!((*Registers).__bindgen_anon_4.LbaHigh, 0);
    ATA_WRITE!((*Registers).__bindgen_anon_5.Command, Command);

    ATA_IO_WAIT!();

    IdeStatus = AtaWait(
        Registers,
        ATA_TIME_BUSY_IDENTIFY,
        (IDE_STATUS_BUSY) as UCHAR,
        0,
    );
    if IdeStatus & (IDE_STATUS_BUSY as u8) != 0 {
        DPRINT!("Timeout, status 0x%02x\n", IdeStatus);
        return FALSE;
    }
    if IdeStatus & ((IDE_STATUS_ERROR | IDE_STATUS_DEVICE_FAULT) as u8) != 0 {
        DPRINT!(
            "Command 0x%02x aborted, status 0x%02x, error 0x%02x\n",
            Command,
            IdeStatus,
            ATA_READ!((*Registers).__bindgen_anon_1.Error),
        );
        return FALSE;
    }
    if IdeStatus & (IDE_STATUS_DRQ as u8) == 0 {
        DPRINT1!("DRQ not set, status 0x%02x\n", IdeStatus as c_uint);
        return FALSE;
    }

    AtaLegacyFetchIdentifyData(Registers);

    IdeStatus = AtaWait(
        Registers,
        ATA_TIME_DRQ_CLEAR,
        (IDE_STATUS_BUSY | IDE_STATUS_DRQ) as UCHAR,
        0,
    );
    if IdeStatus & ((IDE_STATUS_BUSY | IDE_STATUS_DRQ) as u8) != 0 {
        DPRINT1!("DRQ not cleared, status 0x%02x\n", IdeStatus as c_uint);
        return FALSE;
    }

    return TRUE;
}

unsafe extern "C" fn AtaLegacyPerformSoftwareReset(
    Registers: PIDE_REGISTERS,
    DeviceNumber: ULONG,
) -> BOOLEAN {
    let mut IdeStatus: UCHAR;
    let mut i: ULONG;

    ATA_WRITE!((*Registers).__bindgen_anon_6.Control, (IDE_DC_RESET_CONTROLLER as u8));
    KeStallExecutionProcessor(20);
    ATA_WRITE!((*Registers).__bindgen_anon_6.Control, (IDE_DC_DISABLE_INTERRUPTS as u8));
    KeStallExecutionProcessor(20);

    if (DeviceNumber & 1) != 0 {
        i = ATA_TIME_RESET_SELECT;
        while i > 0 {
            ATA_WRITE!(
                (*Registers).Device,
                (((DeviceNumber << 4) | IDE_DRIVE_SELECT)) as u8,
            );
            ATA_IO_WAIT!();

            ATA_WRITE!((*Registers).__bindgen_anon_3.ByteCountLow, 0xAA);
            ATA_WRITE!((*Registers).__bindgen_anon_3.ByteCountLow, 0x55);
            ATA_WRITE!((*Registers).__bindgen_anon_3.ByteCountLow, 0xAA);
            if ATA_READ!((*Registers).__bindgen_anon_3.ByteCountLow) == 0xAA {
                break;
            }

            KeStallExecutionProcessor(10);
            i = i.wrapping_sub(1);
        }
        if i == 0 {
            DPRINT!("Selection timeout\n");
            return FALSE;
        }
    }

    IdeStatus = AtaWait(
        Registers,
        ATA_TIME_BUSY_RESET,
        (IDE_STATUS_BUSY) as UCHAR,
        0,
    );
    if IdeStatus & (IDE_STATUS_BUSY as u8) != 0 {
        DPRINT1!("Timeout, status 0x%02x\n", IdeStatus as c_uint);
        return FALSE;
    }

    return FALSE;
}

unsafe extern "C" fn AtaLegacyFindAtaDevice(
    Registers: PIDE_REGISTERS,
    DeviceNumber: ULONG,
) -> BOOLEAN {
    let mut IdeStatus: UCHAR;
    let mut SignatureLow: UCHAR;
    let mut SignatureHigh: UCHAR;

    ATA_WRITE!(
        (*Registers).Device,
        ((DeviceNumber << 4) | IDE_DRIVE_SELECT) as UCHAR,
    );
    ATA_IO_WAIT!();

    IdeStatus = ATA_READ!((*Registers).__bindgen_anon_5.Status);
    if IdeStatus == 0xFF || IdeStatus == 0x7F {
        return FALSE;
    }

    ATA_WRITE!((*Registers).__bindgen_anon_3.ByteCountLow, 0x55);
    ATA_WRITE!((*Registers).__bindgen_anon_3.ByteCountLow, 0xAA);
    ATA_WRITE!((*Registers).__bindgen_anon_3.ByteCountLow, 0x55);
    if ATA_READ!((*Registers).__bindgen_anon_3.ByteCountLow) != 0x55 {
        return FALSE;
    }
    ATA_WRITE!((*Registers).__bindgen_anon_4.ByteCountHigh, 0xAA);
    ATA_WRITE!((*Registers).__bindgen_anon_4.ByteCountHigh, 0x55);
    ATA_WRITE!((*Registers).__bindgen_anon_4.ByteCountHigh, 0xAA);
    if ATA_READ!((*Registers).__bindgen_anon_4.ByteCountHigh) != 0xAA {
        return FALSE;
    }

    IdeStatus = AtaWait(
        Registers,
        ATA_TIME_BUSY_SELECT,
        (IDE_STATUS_BUSY) as UCHAR,
        0,
    );
    if IdeStatus & (IDE_STATUS_BUSY as u8) != 0 {
        DPRINT1!("Device is busy, attempting to recover %02x\n", IdeStatus as c_uint);

        if AtaLegacyPerformSoftwareReset(Registers, DeviceNumber) == 0 {
            DPRINT1!("Failed to reset device %02x\n", ATA_READ!((*Registers).__bindgen_anon_5.Status) as c_uint);
            return FALSE;
        }
    }

    if AtaLegacyIdentifyDevice(Registers, DeviceNumber, (IDE_COMMAND_IDENTIFY) as UCHAR) != 0 {
        return TRUE;
    }

    SignatureLow = ATA_READ!((*Registers).__bindgen_anon_3.SignatureLow);
    SignatureHigh = ATA_READ!((*Registers).__bindgen_anon_4.SignatureHigh);

    DPRINT!("SL = 0x%02x, SH = 0x%02x\n", SignatureLow, SignatureHigh);

    if SignatureLow == 0x14 && SignatureHigh == 0xEB {
        return TRUE;
    }

    return AtaLegacyIdentifyDevice(Registers, DeviceNumber, (IDE_COMMAND_ATAPI_IDENTIFY) as UCHAR);
}

unsafe extern "C" fn AtaLegacyChannelPresent(Registers: PIDE_REGISTERS) -> BOOLEAN {
    let mut i: ULONG;

    if IsNEC_98 != 0 {
        return (ATA_READ!(0x432 as *mut u8) as i32 != 0xFF) as BOOLEAN;
    }

    i = 0;
    while i < MAX_IDE_DEVICE {
        if AtaLegacyFindAtaDevice(Registers, i) != 0 {
            DPRINT!(b"Found IDE device\n\0".as_ptr() as *const i8);
            return TRUE;
        }
        i = i.wrapping_add(1);
    }

    return FALSE;
}

unsafe extern "C" fn AtaLegacyMakePortResource(
    Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    IoBase: u32,
    Length: u32,
) {
    (*Descriptor).Type = CmResourceTypePort;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
    (*Descriptor).Flags = (CM_RESOURCE_PORT_IO | CM_RESOURCE_PORT_16_BIT_DECODE) as u16;
    (*Descriptor).u.Port.Start.u.LowPart = IoBase;
    (*Descriptor).u.Port.Length = Length;
}

unsafe extern "C" fn AtaLegacyMakeInterruptResource(
    Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    Level: u32,
) {
    (*Descriptor).Type = CmResourceTypeInterrupt;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
    (*Descriptor).Flags = (CM_RESOURCE_INTERRUPT_LATCHED as u16);
    (*Descriptor).u.Interrupt.Level = Level;
    (*Descriptor).u.Interrupt.Vector = Level;
    (*Descriptor).u.Interrupt.Affinity = -(1 as i32) as KAFFINITY;
}

unsafe extern "C" fn AtaLegacyChannelBuildResources(
    LegacyChannel: *mut ATA_LEGACY_CHANNEL,
    ResourceList: *mut CM_RESOURCE_LIST,
) -> i32 {
    let ConfigInfo: *mut CONFIGURATION_INFORMATION = IoGetConfigurationInformation();
    let mut Descriptor: *mut CM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut i: u32;

    Descriptor = &mut (*ResourceList).List[0 as usize]
        .PartialResourceList
        .PartialDescriptors[0 as usize];

    if IsNEC_98 != 0 {
        if (*ConfigInfo).AtDiskPrimaryAddressClaimed != 0
            || (*ConfigInfo).AtDiskSecondaryAddressClaimed != 0
        {
            return (FALSE as i32);
        }

        AtaLegacyMakePortResource(Descriptor, 0x640, 1);
        Descriptor = Descriptor.add(1);
        AtaLegacyMakePortResource(Descriptor, 0x74C, 1);
        Descriptor = Descriptor.add(1);
        AtaLegacyMakeInterruptResource(Descriptor, 9);
        Descriptor = Descriptor.add(1);
        i = 0;
        while i < 7 {
            AtaLegacyMakePortResource(
                Descriptor,
                0x642 + i * 2,
                1,
            );
            Descriptor = Descriptor.add(1);
            i += 1;
        }
        AtaLegacyMakePortResource(Descriptor, 0x432, 2);
        Descriptor = Descriptor.add(1);
        AtaLegacyMakePortResource(Descriptor, 0x435, 1);
    } else {
        // vibeos, LEGIDE_DETECT only: detection was forced on an ACPI machine, where the two
        // compatibility channels belong to the PCI IDE function whether or not pciidex has
        // started it and set the claimed flags yet (legide runs first here).
        let forced = FORCED_DETECTION.load(core::sync::atomic::Ordering::Relaxed);
        if (*LegacyChannel).IoBase == 0x1F0
            && ((*ConfigInfo).AtDiskPrimaryAddressClaimed != 0 || forced)
        {
            return (FALSE as i32);
        }

        if (*LegacyChannel).IoBase == 0x170
            && ((*ConfigInfo).AtDiskSecondaryAddressClaimed != 0 || forced)
        {
            return (FALSE as i32);
        }

        AtaLegacyMakePortResource(Descriptor, (*LegacyChannel).IoBase, 8);
        Descriptor = Descriptor.add(1);
        AtaLegacyMakePortResource(
            Descriptor,
            (*LegacyChannel).IoBase + 0x206,
            1,
        );
        Descriptor = Descriptor.add(1);
        AtaLegacyMakeInterruptResource(Descriptor, (*LegacyChannel).Irq);
    }

    return (TRUE as i32);
}

unsafe extern "C" fn AtaLegacyChannelTranslateResources(
    ResourceList: PCM_RESOURCE_LIST,
    CommandPortBase: PUCHAR,
    ControlPortBase: PUCHAR,
) {
    let mut Descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut Irql: KIRQL = core::mem::zeroed();

    Descriptor = &raw mut (*ResourceList).List[0]
        .PartialResourceList
        .PartialDescriptors[0];

    if ((IsNEC_98) != 0) {
        Descriptor = Descriptor.add(2);
    } else {
        (*Descriptor).u.Port.Start.QuadPart = CommandPortBase as ULONG_PTR as LONGLONG;
        Descriptor = Descriptor.add(1);

        (*Descriptor).u.Port.Start.QuadPart = ControlPortBase as ULONG_PTR as LONGLONG;
        Descriptor = Descriptor.add(1);
    }

    (*Descriptor).u.Interrupt.Vector = HalGetInterruptVector(
        Isa,
        0,
        (*Descriptor).u.Interrupt.Level,
        (*Descriptor).u.Interrupt.Vector,
        &raw mut Irql,
        &raw mut (*Descriptor).u.Interrupt.Affinity,
    );
    (*Descriptor).u.Interrupt.Level = (Irql) as _;
}

unsafe extern "C" fn AtaLegacyTranslateBusAddress(
    mut Address: ULONG,
    mut NumberOfBytes: ULONG,
    mut IsAddressMmio: PBOOLEAN,
) -> PVOID {
    let mut AddressSpace: ULONG = 0;
    let mut Success: BOOLEAN = 0;
    let mut BusAddress: PHYSICAL_ADDRESS = PHYSICAL_ADDRESS { QuadPart: 0 };
    let mut TranslatedAddress: PHYSICAL_ADDRESS = PHYSICAL_ADDRESS { QuadPart: 0 };
    BusAddress.QuadPart = Address as LONGLONG;
    AddressSpace = 1;
    Success = HalTranslateBusAddress(
        Isa,
        0,
        BusAddress,
        &raw mut AddressSpace,
        &raw mut TranslatedAddress,
    );
    if Success == 0 {
        return core::ptr::null_mut();
    }
    if AddressSpace != 0 {
        *IsAddressMmio = FALSE;
        return TranslatedAddress.QuadPart as ULONG_PTR as PVOID;
    } else {
        *IsAddressMmio = TRUE;
        return MmMapIoSpace(TranslatedAddress, NumberOfBytes as SIZE_T, MmNonCached);
    };
}

unsafe extern "C" fn AtaLegacyClaimHardwareResources(
    DriverObject: PDRIVER_OBJECT,
    ResourceList: PCM_RESOURCE_LIST,
    ResourceListSize: u32,
) -> i32 {
    let mut Status: i32 = 0;
    let mut ConflictDetected: i32 = 0;

    Status = IoReportResourceForDetection(
        DriverObject,
        ResourceList,
        ResourceListSize,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        (core::ptr::addr_of_mut!(ConflictDetected)).cast(),
    );

    if IsNEC_98 != 0 {
        return (TRUE as i32);
    }

    if !NT_SUCCESS!(Status) || ConflictDetected != 0 {
        return (FALSE as i32);
    }

    return (TRUE as i32);
}

unsafe extern "C" fn AtaLegacyReleaseHardwareResources(DriverObject: PDRIVER_OBJECT) {
    let mut Dummy: BOOLEAN = 0;
    IoReportResourceForDetection(
        DriverObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        &mut Dummy,
    );
}

unsafe extern "C" fn AtaLegacyDetectChannel(
    DriverObject: PDRIVER_OBJECT,
    ControllerInferface: PPCIIDEX_LEGACY_CONTROLLER_INTERFACE,
    LegacyChannel: PATA_LEGACY_CHANNEL,
    ResourceList: PCM_RESOURCE_LIST,
    ResourceListSize: ULONG,
) -> BOOLEAN {
    let mut IsAddressMmio: [BOOLEAN; 2 as usize] =
        [0; 2 as usize];
    let mut PhysicalDeviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut ControllerContext: PVOID = core::ptr::null_mut();
    let mut Registers: IDE_REGISTERS = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;
    let mut Spare: ULONG = 0;

    DPRINT!(
        b"IDE Channel IO %lx, Irq %lu\n\0".as_ptr() as *const i8,
        (*LegacyChannel).IoBase,
        (*LegacyChannel).Irq,
    );

    if ((AtaLegacyChannelBuildResources(LegacyChannel, ResourceList)) == 0) {
        DPRINT!(b"Failed to build the resource list\n\0".as_ptr() as *const i8);
        return FALSE as BOOLEAN;
    }

    if ((AtaLegacyClaimHardwareResources(DriverObject, ResourceList, ResourceListSize)) == 0) {
        DPRINT!(b"Failed to claim resources\n\0".as_ptr() as *const i8);
        return FALSE as BOOLEAN;
    }

    'release_resources: {
        Status = STATUS_SUCCESS as NTSTATUS;
        IsAddressMmio[0 as usize] = FALSE as BOOLEAN;
        IsAddressMmio[1 as usize] = FALSE as BOOLEAN;

        if IsNEC_98 != 0 {
            Registers.Data = 0x640 as usize as *mut u8;
            Registers.__bindgen_anon_6.Control = 0x74C as usize as *mut u8;
            Spare = 2 as ULONG;
        } else {
            Registers.Data = AtaLegacyTranslateBusAddress(
                (*LegacyChannel).IoBase,
                8 as ULONG,
                core::ptr::addr_of_mut!(IsAddressMmio[0 as usize]),
            ) as *mut u8;
            if Registers.Data.is_null() {
                DPRINT!(b"Failed to map command port\n\0".as_ptr() as *const i8);
                Status = STATUS_UNSUCCESSFUL as NTSTATUS;
                break 'release_resources;
            }

            Registers.__bindgen_anon_6.Control = AtaLegacyTranslateBusAddress(
                (*LegacyChannel)
                    .IoBase
                    .wrapping_add(0x206 as ULONG),
                1 as ULONG,
                core::ptr::addr_of_mut!(IsAddressMmio[1 as usize]),
            ) as *mut u8;
            if Registers.__bindgen_anon_6.Control.is_null() {
                DPRINT!(b"Failed to map control port\n\0".as_ptr() as *const i8);
                Status = STATUS_UNSUCCESSFUL as NTSTATUS;
                break 'release_resources;
            }

            Spare = 1 as ULONG;
        }

        Registers.__bindgen_anon_1.Error = (Registers.Data as usize)
            .wrapping_add((1 as usize).wrapping_mul(Spare as usize))
            as *mut u8;
        Registers.__bindgen_anon_2.SectorCount = (Registers.Data as usize)
            .wrapping_add((2 as usize).wrapping_mul(Spare as usize))
            as *mut u8;
        Registers.LbaLow = (Registers.Data as usize)
            .wrapping_add((3 as usize).wrapping_mul(Spare as usize))
            as *mut u8;
        Registers.__bindgen_anon_3.LbaMid = (Registers.Data as usize)
            .wrapping_add((4 as usize).wrapping_mul(Spare as usize))
            as *mut u8;
        Registers.__bindgen_anon_4.LbaHigh = (Registers.Data as usize)
            .wrapping_add((5 as usize).wrapping_mul(Spare as usize))
            as *mut u8;
        Registers.Device = (Registers.Data as usize)
            .wrapping_add((6 as usize).wrapping_mul(Spare as usize))
            as *mut u8;
        Registers.__bindgen_anon_5.Status = (Registers.Data as usize)
            .wrapping_add((7 as usize).wrapping_mul(Spare as usize))
            as *mut u8;

        if ((AtaLegacyChannelPresent(core::ptr::addr_of_mut!(Registers))) == 0) {
            DPRINT!(b"No IDE devices found\n\0".as_ptr() as *const i8);
            Status = STATUS_UNSUCCESSFUL as NTSTATUS;
        }
    }

    AtaLegacyReleaseHardwareResources(DriverObject);

    'cleanup: {
        if !NT_SUCCESS!(Status) {
            break 'cleanup;
        }

        PhysicalDeviceObject = core::ptr::null_mut();
        Status = IoReportDetectedDevice(
            DriverObject,
            (InterfaceTypeUndefined) as INTERFACE_TYPE,
            (-(1 as i32)) as ULONG,
            (-(1 as i32)) as ULONG,
            ResourceList,
            core::ptr::null_mut(),
            0 as BOOLEAN,
            core::ptr::addr_of_mut!(PhysicalDeviceObject),
        );
        if !NT_SUCCESS!(Status) {
            DPRINT!(
                b"IoReportDetectedDevice() failed with status 0x%lx\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'cleanup;
        }

        if (*ControllerInferface).Version != PCIIDEX_INTERFACE_VERSION as ULONG {
            Status = PciIdeXInitialize(
                DriverObject,
                core::ptr::null_mut(),
                // ReactOS-specific overload: the C passes the interface buffer (as PVOID)
                // through the HwGetControllerProperties slot when the signature asks for it.
                core::mem::transmute::<PVOID, PCONTROLLER_PROPERTIES>(ControllerInferface as PVOID),
                PCIIDEX_GET_CONTROLLER_INTERFACE_SIGNATURE as ULONG,
            );
            if !NT_SUCCESS!(Status) {
                DPRINT1!(
                    b"Failed to register the legacy channel 0x%lx, status 0x%lx\n\0".as_ptr()
                        as *const i8,
                    (*LegacyChannel).IoBase,
                    Status,
                );
                break 'cleanup;
            }

            if (*ControllerInferface).Version != PCIIDEX_INTERFACE_VERSION as ULONG {
                DPRINT1!(
                    b"Unknown interface version 0x%lx\n\0".as_ptr() as *const i8,
                    (*ControllerInferface).Version,
                );
                Status = STATUS_REVISION_MISMATCH as NTSTATUS;
                break 'cleanup;
            }
        }

        Status = (((*ControllerInferface).AddDevice).unwrap_unchecked())(
            DriverObject,
            PhysicalDeviceObject,
            core::ptr::addr_of_mut!(ControllerContext),
        );
        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                b"Failed to add the legacy channel 0x%lx, status 0x%lx\n\0".as_ptr()
                    as *const i8,
                (*LegacyChannel).IoBase,
                Status,
            );
            break 'cleanup;
        }

        AtaLegacyChannelTranslateResources(ResourceList, Registers.Data, Registers.__bindgen_anon_6.Control);

        Status = (((*ControllerInferface).StartDevice).unwrap_unchecked())(ControllerContext, ResourceList);
        if !NT_SUCCESS!(Status) {
            DPRINT1!(
                b"Failed to start the legacy channel 0x%lx, status 0x%lx\n\0".as_ptr()
                    as *const i8,
                (*LegacyChannel).IoBase,
                Status,
            );
            (((*ControllerInferface).RemoveDevice).unwrap_unchecked())(ControllerContext);
        }
    }

    if IsAddressMmio[0 as usize] != 0 && !Registers.Data.is_null() {
        MmUnmapIoSpace(Registers.Data as PVOID, (8 as usize) as _);
    }

    if IsAddressMmio[1 as usize] != 0 && !Registers.__bindgen_anon_6.Control.is_null() {
        MmUnmapIoSpace(Registers.__bindgen_anon_6.Control as PVOID, (1 as usize) as _);
    }

    return NT_SUCCESS!(Status) as BOOLEAN;
}

unsafe extern "C" fn AtaOpenRegistryKey(
    KeyHandle: PHANDLE,
    RootKey: HANDLE,
    KeyName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    if KeGetCurrentIrql() as i32 > (APC_LEVEL as i32) {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/storage/ide/legide/legide.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                680 as u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = RootKey;
    ObjectAttributes.ObjectName = KeyName;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();

    return ZwOpenKey(KeyHandle, KEY_ALL_ACCESS, &mut ObjectAttributes);
}

/// Set when the LEGIDE_DETECT tripwire forced detection past DisableFirmwareMapper.
static FORCED_DETECTION: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

extern "system" {
    // ntoskrnl (tripwire.rs): whether this boot's tripwire config holds the exact token.
    fn VibeTripwireHasToken(needle: *const u8, len: usize) -> u8;
}

unsafe extern "C" fn AtaLegacyShouldDetectChannels(
    mut RegistryPath: PUNICODE_STRING,
) -> BOOLEAN {
    let mut Status: NTSTATUS = 0;
    let mut KeyValue: ULONG = 0;
    let mut PerformDetection: BOOLEAN = 0;
    let mut SoftKeyHandle: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut ParamsKeyHandle: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 2] =
        [RTL_QUERY_REGISTRY_TABLE {
            QueryRoutine: None,
            Flags: 0,
            Name: core::ptr::null::<u16>(),
            EntryContext: core::ptr::null_mut::<core::ffi::c_void>(),
            DefaultType: 0,
            DefaultData: core::ptr::null_mut::<core::ffi::c_void>(),
            DefaultLength: 0,
        }; 2];
    let mut ParametersKeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 11]>() as usize)
            .wrapping_sub(core::mem::size_of::<u16>() as usize) as u16,
        MaximumLength: (core::mem::size_of::<[u16; 11]>() as usize)
            .wrapping_div(core::mem::size_of::<core::ffi::c_char>() as usize) as u16,
        Buffer: RUSTCONV_WSTR_9EADC9D3A146.as_ptr() as *mut u16,
    };
    static mut MapperKeyPath: [u16; 55] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
    0x005C, 0x0050, 0x006E, 0x0070, 0x0000,
];

    PAGED_CODE!();

    KeyValue = 0;
    RtlZeroMemory!(
        &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE as *mut core::ffi::c_void,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>() as usize,
    );
    QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT;
    QueryTable[0].Name = RUSTCONV_WSTR_6CC06C2CA9FD.as_ptr();
    QueryTable[0].EntryContext = &raw mut KeyValue as *mut core::ffi::c_void;
    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_ABSOLUTE,
        &raw const MapperKeyPath as *const u16,
        &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    // vibeos divergence, test rigs only: the LEGIDE_DETECT tripwire probes anyway. The
    // ACPI HAL always sets DisableFirmwareMapper and we ship no PC/AT HAL, so without it
    // nothing below this line can run on any boot (`smoke --isa-ide`).
    if NT_SUCCESS!(Status) != false && KeyValue != 0 {
        if VibeTripwireHasToken(b"LEGIDE_DETECT".as_ptr(), b"LEGIDE_DETECT".len()) == 0 {
            DPRINT!("Skipping legacy detection on a PnP system\n");
            return FALSE;
        }
        FORCED_DETECTION.store(true, core::sync::atomic::Ordering::Relaxed);
        DPRINT1!("LEGIDE_DETECT: probing the non-compatibility channels on a PnP system\n");
    }

    Status = AtaOpenRegistryKey(
        &raw mut SoftKeyHandle,
        core::ptr::null_mut(),
        RegistryPath,
    );
    if NT_SUCCESS!(Status) == false {
        DPRINT1!(
            "Failed to open the '%wZ' key, status 0x%lx\n",
            RegistryPath,
            Status,
        );
        return FALSE;
    }

    Status = AtaOpenRegistryKey(
        &raw mut ParamsKeyHandle,
        SoftKeyHandle,
        &raw mut ParametersKeyName,
    );
    if NT_SUCCESS!(Status) == false {
        DPRINT1!("Failed to open the 'Parameters' key, status 0x%lx\n", Status);

        ZwClose(SoftKeyHandle);
        return FALSE;
    }

    KeyValue = 0;
    RtlZeroMemory!(
        &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE as *mut core::ffi::c_void,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 2]>() as usize,
    );
    QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_REQUIRED;
    QueryTable[0].Name = RUSTCONV_WSTR_D1C33AAC335D.as_ptr();
    QueryTable[0].EntryContext = &raw mut KeyValue as *mut core::ffi::c_void;
    RtlQueryRegistryValues(
        RTL_REGISTRY_HANDLE,
        ParamsKeyHandle as PWSTR,
        &raw mut QueryTable as *mut RTL_QUERY_REGISTRY_TABLE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    PerformDetection = (KeyValue != 0) as BOOLEAN;

    if PerformDetection != 0 {
        KeyValue = 0;
        RtlWriteRegistryValue(
            RTL_REGISTRY_HANDLE,
            ParamsKeyHandle as PWSTR,
            RUSTCONV_WSTR_D1C33AAC335D.as_ptr(),
            REG_DWORD,
            &raw mut KeyValue as *mut ULONG as *mut core::ffi::c_void,
            core::mem::size_of::<ULONG>() as ULONG,
        );
    }

    ZwClose(ParamsKeyHandle);
    ZwClose(SoftKeyHandle);

    return PerformDetection;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut ControllerInferface: PCIIDEX_LEGACY_CONTROLLER_INTERFACE = core::mem::zeroed();
    let mut LegacyChannel: [ATA_LEGACY_CHANNEL; 4 + 1] = core::mem::zeroed();
    let mut ResourceList: PCM_RESOURCE_LIST;
    let mut i: u32;
    let mut ListSize: u32;
    let mut ResourceCount: u32;
    let mut Status: NTSTATUS;

    if AtaLegacyShouldDetectChannels(RegistryPath) == 0 {
        return STATUS_CANCELLED;
    }

    if IsNEC_98 != 0 {
        LegacyChannel[0].IoBase = 0x640;
        LegacyChannel[0].Irq = 9;
        ResourceCount = CHANNEL_PC98_RESOURCE_COUNT;
    } else {
        LegacyChannel[0].IoBase = 0x1F0;
        LegacyChannel[0].Irq = 14;
        LegacyChannel[1].IoBase = 0x170;
        LegacyChannel[1].Irq = 15;
        LegacyChannel[2].IoBase = 0x1E8;
        LegacyChannel[2].Irq = 11;
        LegacyChannel[3].IoBase = 0x168;
        LegacyChannel[3].Irq = 10;
        ResourceCount = CHANNEL_PCAT_RESOURCE_COUNT;
    }

    ListSize = (FIELD_OFFSET!(CM_RESOURCE_LIST, List[0].PartialResourceList.PartialDescriptors)
        + (core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as u32 * ResourceCount) as i32) as u32;

    ResourceList = ExAllocatePoolZero(PagedPool, (ListSize) as _, ATAPORT_TAG) as PCM_RESOURCE_LIST;
    if ResourceList.is_null() {
        DPRINT1!("Failed to allocate the resource list\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*ResourceList).Count = 1;
    (*ResourceList).List[0].InterfaceType = Isa;
    (*ResourceList).List[0].BusNumber = 0;
    (*ResourceList).List[0].PartialResourceList.Version = 1;
    (*ResourceList).List[0].PartialResourceList.Revision = 1;
    (*ResourceList).List[0].PartialResourceList.Count = ResourceCount;

    Status = STATUS_NOT_FOUND;

    i = 0;
    while LegacyChannel[i as usize].IoBase != 0 {
        if AtaLegacyDetectChannel(
            DriverObject,
            &mut ControllerInferface,
            &mut LegacyChannel[i as usize],
            ResourceList,
            ListSize,
        ) != 0 {
            Status = STATUS_SUCCESS;
        }
        i += 1;
    }

    ExFreePoolWithTag((ResourceList).cast(), ATAPORT_TAG);
    Status
}
