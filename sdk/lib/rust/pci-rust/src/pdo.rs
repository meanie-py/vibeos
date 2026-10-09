//! pdo.c — Child (Physical) Device Object dispatch routines for pci.sys.
//!
//! Faithful translation of `drivers/bus/pci/pdo.c` (ReactOS PCI bus driver).
//! No SEH in this driver — straightforward PnP/Power IRP dispatch.
//!
//! ## Substrate notes (banked while translating)
//!
//! * **`PCI_COMMON_CONFIG` is a broken bindgen type.** In the C headers
//!   `PCI_COMMON_CONFIG` = `{ PCI_COMMON_HEADER hdr; UCHAR DeviceSpecific[192] }`
//!   (256 bytes). bindgen, however, emitted `_PCI_COMMON_CONFIG { DeviceSpecific:
//!   [UCHAR; 192] }` — it dropped the inlined header entirely (192 bytes, no
//!   VendorID/Command/u.type0/...). `shared::PCI_DEVICE.PciConfig` re-exports that
//!   broken type, so we cannot read header fields through it directly. Everywhere
//!   the C accesses header fields (`PciConfig.VendorID`, `PciConfig.u.type0.*`,
//!   `PciConfig.BaseClass`, …) we instead view the storage as a `PCI_COMMON_HEADER`
//!   (`pci_config_header(ptr)`), whose bindgen layout *is* correct (64 bytes,
//!   verified by size_check). For the locals the C declares as
//!   `PCI_COMMON_CONFIG PciConfig`, we use a `PCI_COMMON_HEADER` local instead —
//!   `HalGetBusData` only writes `PCI_COMMON_HDR_LENGTH` (= 64) bytes into it,
//!   which is exactly the header. See TODO(pci-rust) markers below.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use core::mem::{offset_of, size_of, transmute};
use core::ptr::{addr_of, addr_of_mut};

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::{
    IoCompleteRequest, PoStartNextPowerIrp,
};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, ObReferenceObject};
use ntkrnl_sys::interlocked::{InterlockedDecrement, InterlockedIncrement};
use ntkrnl_sys::ke::{
    KeAcquireSpinLock, KeAcquireSpinLockAtDpcLevel, KeReleaseSpinLock,
    KeReleaseSpinLockFromDpcLevel,
};

use crate::shared::{
    PCI_DEVICE, PFDO_DEVICE_EXTENSION, PPDO_DEVICE_EXTENSION, TAG_PCI,
    // Use the corrected 256-byte PCI_COMMON_CONFIG (shared.rs) — shadows the
    // broken 192-byte one in the `ntkrnl_sys::ddk::*` glob below.
    PCI_COMMON_CONFIG,
};

// NT_SUCCESS + most STATUS_* live in ntkrnl_sys::base (re-exported at crate root);
// the rest in ntkrnl_sys::constants. NOT in the ddk bindgen glob.
use ntkrnl_sys::{STATUS_SUCCESS, STATUS_NOT_IMPLEMENTED, STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::constants::{
    STATUS_UNSUCCESSFUL, STATUS_INVALID_PARAMETER, STATUS_NOT_SUPPORTED,
    STATUS_BUFFER_TOO_SMALL, STATUS_PENDING,
};

// pdo.c's own DBGPRINT, not debug.h's: it is `#define DBGPRINT(...)` behind an
// `#if 0`, so it is off in the C too. DPRINT/DPRINT1 come from `crate::dprint`.
macro_rules! DBGPRINT { ($($t:tt)*) => {} }

// ---------------------------------------------------------------------------
// Constants not present in ntkrnl-sys (#defines / enum variants the bindgen
// pass didn't capture). Values are header-authoritative (sdk/include/xdk/*).
// ---------------------------------------------------------------------------

// drivers/bus/pci/pdo.c local #defines
const PCI_ADDRESS_MEMORY_ADDRESS_MASK_64: u64 = 0xffff_ffff_ffff_fff0;
const PCI_ADDRESS_IO_ADDRESS_MASK_64: u64 = 0xffff_ffff_ffff_fffc;

// xdk/iotypes.h
const PCI_COMMON_HDR_LENGTH: u32 = size_of::<PCI_COMMON_HEADER>() as u32; // 64
const PCI_TYPE0_ADDRESSES: u8 = 6;
const PCI_TYPE1_ADDRESSES: u8 = 2;
const PCI_MULTIFUNCTION: u8 = 0x80;
const PCI_DEVICE_TYPE: u8 = 0x00;
const PCI_BRIDGE_TYPE: u8 = 0x01;
const PCI_CARDBUS_BRIDGE_TYPE: u8 = 0x02;
const PCI_ENABLE_IO_SPACE: u16 = 0x0001;
const PCI_ENABLE_MEMORY_SPACE: u16 = 0x0002;
const PCI_ENABLE_BUS_MASTER: u16 = 0x0004;
const PCI_WHICHSPACE_CONFIG: u32 = 0x0;
const PCI_CLASS_BRIDGE_DEV: u8 = 0x06;
const PCI_ADDRESS_IO_SPACE: u32 = 0x0000_0001;
const PCI_ADDRESS_MEMORY_TYPE_MASK: u32 = 0x0000_0006;
const PCI_ADDRESS_MEMORY_PREFETCHABLE: u32 = 0x0000_0008;
const PCI_TYPE_32BIT: u32 = 0;
const PCI_TYPE_20BIT: u32 = 2;
const PCI_TYPE_64BIT: u32 = 4;
const PCI_USE_SUBSYSTEM_IDS: u32 = 0x0000_0001;
const PCI_USE_REVISION: u32 = 0x0000_0002;
const PCI_USE_VENDEV_IDS: u32 = 0x0000_0004;
const PCI_USE_CLASS_SUBCLASS: u32 = 0x0000_0008;
const PCI_USE_PROGIF: u32 = 0x0000_0010;
const PCI_USE_LOCAL_BUS: u32 = 0x0000_0020;
const PCI_USE_LOCAL_DEVICE: u32 = 0x0000_0040;

// xdk/cmtypes.h — CM_PARTIAL_RESOURCE_DESCRIPTOR.Type / .ShareDisposition / .Flags
const CmResourceTypePort: u8 = 1;
const CmResourceTypeInterrupt: u8 = 2;
const CmResourceTypeMemory: u8 = 3;
const CmResourceTypeBusNumber: u8 = 6;
const CmResourceShareDeviceExclusive: u8 = 1;
const CmResourceShareShared: u8 = 3;
const CM_RESOURCE_PORT_IO: u16 = 0x0001;
const CM_RESOURCE_PORT_16_BIT_DECODE: u16 = 0x0010;
const CM_RESOURCE_PORT_POSITIVE_DECODE: u16 = 0x0020;
const CM_RESOURCE_MEMORY_READ_WRITE: u16 = 0x0000;
const CM_RESOURCE_MEMORY_PREFETCHABLE: u16 = 0x0004;
const CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE: u16 = 0x0000;
// xdk/iotypes.h — IO_RESOURCE_DESCRIPTOR.Option
const IO_RESOURCE_PREFERRED: u8 = 0x01;
const IO_RESOURCE_ALTERNATIVE: u8 = 0x08;

// ndk/rtltypes.h
const RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE: u32 = 1;

// Enum variant values (bindgen names them `_ENUM_Variant`; we want plain ints).
const BUS_DATA_TYPE_PCIConfiguration: BUS_DATA_TYPE = 4; // _BUS_DATA_TYPE_PCIConfiguration
const INTERFACE_TYPE_PCIBus: INTERFACE_TYPE = 5; // _INTERFACE_TYPE_PCIBus

const DeviceTextDescription: DEVICE_TEXT_TYPE = 0;
const DeviceTextLocationInformation: DEVICE_TEXT_TYPE = 1;

const BusQueryDeviceID: BUS_QUERY_ID_TYPE = 0;
const BusQueryHardwareIDs: BUS_QUERY_ID_TYPE = 1;
const BusQueryCompatibleIDs: BUS_QUERY_ID_TYPE = 2;
const BusQueryInstanceID: BUS_QUERY_ID_TYPE = 3;

const TargetDeviceRelation: DEVICE_RELATION_TYPE = 4;

// IRP_MN_* minor function codes (xdk/iotypes.h). u8 to match IrpSp.MinorFunction.
const IRP_MN_START_DEVICE: u8 = 0x00;
const IRP_MN_QUERY_REMOVE_DEVICE: u8 = 0x01;
const IRP_MN_REMOVE_DEVICE: u8 = 0x02;
const IRP_MN_CANCEL_REMOVE_DEVICE: u8 = 0x03;
const IRP_MN_STOP_DEVICE: u8 = 0x04;
const IRP_MN_QUERY_STOP_DEVICE: u8 = 0x05;
const IRP_MN_CANCEL_STOP_DEVICE: u8 = 0x06;
const IRP_MN_QUERY_DEVICE_RELATIONS: u8 = 0x07;
const IRP_MN_QUERY_INTERFACE: u8 = 0x08;
const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;
const IRP_MN_QUERY_RESOURCES: u8 = 0x0A;
const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: u8 = 0x0B;
const IRP_MN_QUERY_DEVICE_TEXT: u8 = 0x0C;
const IRP_MN_FILTER_RESOURCE_REQUIREMENTS: u8 = 0x0D;
const IRP_MN_READ_CONFIG: u8 = 0x0F;
const IRP_MN_WRITE_CONFIG: u8 = 0x10;
const IRP_MN_EJECT: u8 = 0x11;
const IRP_MN_SET_LOCK: u8 = 0x12;
const IRP_MN_QUERY_ID: u8 = 0x13;
const IRP_MN_QUERY_PNP_DEVICE_STATE: u8 = 0x14;
const IRP_MN_QUERY_BUS_INFORMATION: u8 = 0x15;
const IRP_MN_DEVICE_USAGE_NOTIFICATION: u8 = 0x16;
const IRP_MN_SURPRISE_REMOVAL: u8 = 0x17;

const IRP_MN_QUERY_POWER: u8 = 0x03;
const IRP_MN_SET_POWER: u8 = 0x02;

const IO_NO_INCREMENT: CCHAR = 0;
const MAXULONG: ULONG = 0xFFFF_FFFF;
const FALSE: BOOLEAN = 0;
const TRUE: BOOLEAN = 1;

// _POOL_TYPE_PagedPool (bindgen names the enum variant `_POOL_TYPE_PagedPool`).
const PagedPool: POOL_TYPE = 1;

// ---------------------------------------------------------------------------
// GUIDs. wdmguid.h DEFINE_GUID values (GUID_BUS_INTERFACE_STANDARD,
// GUID_PCI_DEVICE_PRESENT_INTERFACE, GUID_BUS_TYPE_PCI) — these are normally
// pulled in via <initguid.h>/<wdmguid.h>; ntkrnl-sys does NOT export them, so
// we materialize them as `static GUID` constants from the header byte values.
// (sdk/include/ddk/wdmguid.h lines 47/67/86.)
// ---------------------------------------------------------------------------
const fn guid(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> GUID {
    GUID { Data1: d1, Data2: d2, Data3: d3, Data4: d4 }
}

static GUID_BUS_INTERFACE_STANDARD: GUID = guid(
    0x496b8280, 0x6f25, 0x11d0,
    [0xbe, 0xaf, 0x08, 0x00, 0x2b, 0xe2, 0x09, 0x2f],
);
static GUID_PCI_DEVICE_PRESENT_INTERFACE: GUID = guid(
    0xd1b82c26, 0xbf49, 0x45ef,
    [0xb2, 0x16, 0x71, 0xcb, 0xd7, 0x88, 0x9b, 0x57],
);
static GUID_BUS_TYPE_PCI: GUID = guid(
    0xc8ebdfb0, 0xb510, 0x11d0,
    [0x80, 0xe5, 0x00, 0xa0, 0xc9, 0x25, 0x42, 0xe3],
);

// ---------------------------------------------------------------------------
// Cross-file C functions exported by pci.c (declared in pci.h), called here.
// NTAPI = extern "system" per arch.
// ---------------------------------------------------------------------------
extern "system" {
    fn PciDuplicateUnicodeString(
        Flags: ULONG,
        SourceString: PCUNICODE_STRING,
        DestinationString: PUNICODE_STRING,
    ) -> NTSTATUS;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// `NT_SUCCESS(Status)` — Status >= 0.
#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

/// View a `*PCI_COMMON_CONFIG` (the broken 192-byte bindgen type) — or any
/// PCI config storage — as a correctly-laid-out `*PCI_COMMON_HEADER`. See the
/// module docs: this is the workaround for the substrate dropping the inlined
/// header from `_PCI_COMMON_CONFIG`.
/// TODO(pci-rust): if bindgen is fixed to emit the full `PCI_COMMON_CONFIG`
/// layout (header + DeviceSpecific[192]), drop this cast and read fields direct.
#[inline]
unsafe fn pci_config_header(p: *const PCI_COMMON_CONFIG) -> *const PCI_COMMON_HEADER {
    p as *const PCI_COMMON_HEADER
}

/// PCI_CONFIGURATION_TYPE(PciData) — `((PPCI_COMMON_CONFIG)PciData)->HeaderType
/// & ~PCI_MULTIFUNCTION`. We read HeaderType through the header view.
#[inline]
unsafe fn PCI_CONFIGURATION_TYPE(hdr: *const PCI_COMMON_HEADER) -> u8 {
    (*hdr).HeaderType & !PCI_MULTIFUNCTION
}

/*** PRIVATE *****************************************************************/

unsafe fn PdoQueryDeviceText(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    DPRINT!("Called\n");

    let DeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    let mut String: UNICODE_STRING = core::mem::zeroed();
    let Status: NTSTATUS;

    match (*IrpSp).Parameters.QueryDeviceText.DeviceTextType {
        x if x == DeviceTextDescription => {
            Status = PciDuplicateUnicodeString(
                RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
                &(*DeviceExtension).DeviceDescription as *const _ as PCUNICODE_STRING,
                &mut String,
            );
            DPRINT!("DeviceTextDescription\n");
            (*Irp).IoStatus.Information = String.Buffer as ULONG_PTR;
        }
        x if x == DeviceTextLocationInformation => {
            Status = PciDuplicateUnicodeString(
                RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
                &(*DeviceExtension).DeviceLocation as *const _ as PCUNICODE_STRING,
                &mut String,
            );
            DPRINT!("DeviceTextLocationInformation\n");
            (*Irp).IoStatus.Information = String.Buffer as ULONG_PTR;
        }
        _ => {
            (*Irp).IoStatus.Information = 0;
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    Status
}

unsafe fn PdoQueryId(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    DPRINT!("Called\n");

    let DeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    let mut String: UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    RtlInitUnicodeString(&mut String, core::ptr::null::<WCHAR>());

    match (*IrpSp).Parameters.QueryId.IdType {
        x if x == BusQueryDeviceID => {
            Status = PciDuplicateUnicodeString(
                RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
                &(*DeviceExtension).DeviceID as *const _ as PCUNICODE_STRING,
                &mut String,
            );
            DPRINT!("DeviceID: %S\n", String.Buffer);
            (*Irp).IoStatus.Information = String.Buffer as ULONG_PTR;
        }
        x if x == BusQueryHardwareIDs => {
            Status = PciDuplicateUnicodeString(
                RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
                &(*DeviceExtension).HardwareIDs as *const _ as PCUNICODE_STRING,
                &mut String,
            );
            (*Irp).IoStatus.Information = String.Buffer as ULONG_PTR;
        }
        x if x == BusQueryCompatibleIDs => {
            Status = PciDuplicateUnicodeString(
                RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
                &(*DeviceExtension).CompatibleIDs as *const _ as PCUNICODE_STRING,
                &mut String,
            );
            (*Irp).IoStatus.Information = String.Buffer as ULONG_PTR;
        }
        x if x == BusQueryInstanceID => {
            Status = PciDuplicateUnicodeString(
                RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
                &(*DeviceExtension).InstanceID as *const _ as PCUNICODE_STRING,
                &mut String,
            );
            DPRINT!("InstanceID: %S\n", String.Buffer);
            (*Irp).IoStatus.Information = String.Buffer as ULONG_PTR;
        }
        // BusQueryDeviceSerialNumber and default
        _ => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    Status
}

unsafe fn PdoQueryBusInformation(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    _IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    DPRINT!("Called\n");

    let DeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    let BusInformation =
        ExAllocatePoolWithTag(PagedPool, size_of::<PNP_BUS_INFORMATION>() as SIZE_T, TAG_PCI)
            as PPNP_BUS_INFORMATION;
    (*Irp).IoStatus.Information = BusInformation as ULONG_PTR;
    if !BusInformation.is_null() {
        (*BusInformation).BusTypeGuid = GUID_BUS_TYPE_PCI;
        (*BusInformation).LegacyBusType = INTERFACE_TYPE_PCIBus;
        (*BusInformation).BusNumber = (*(*DeviceExtension).PciDevice).BusNumber;

        return STATUS_SUCCESS;
    }

    STATUS_INSUFFICIENT_RESOURCES
}

unsafe fn PdoQueryCapabilities(
    DeviceObject: PDEVICE_OBJECT,
    _Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    DPRINT!("Called\n");

    let DeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;
    let DeviceCapabilities = (*IrpSp).Parameters.DeviceCapabilities.Capabilities;

    if (*DeviceCapabilities).Version != 1 {
        return STATUS_UNSUCCESSFUL;
    }

    let DeviceNumber =
        (*(*DeviceExtension).PciDevice).SlotNumber.u.bits.DeviceNumber();
    let FunctionNumber =
        (*(*DeviceExtension).PciDevice).SlotNumber.u.bits.FunctionNumber();

    (*DeviceCapabilities).set_UniqueID(FALSE as ULONG);
    (*DeviceCapabilities).Address =
        ((DeviceNumber << 16) & 0xFFFF_0000) + (FunctionNumber & 0xFFFF);
    (*DeviceCapabilities).UINumber = MAXULONG; /* FIXME */

    STATUS_SUCCESS
}

unsafe fn PdoReadPciBar(
    DeviceExtension: PPDO_DEVICE_EXTENSION,
    Offset: ULONG,
    OriginalValue: *mut ULONG,
    NewValue: *mut ULONG,
) -> BOOLEAN {
    /* Read the original value */
    let mut Size = HalGetBusDataByOffset(
        BUS_DATA_TYPE_PCIConfiguration,
        (*(*DeviceExtension).PciDevice).BusNumber,
        (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG,
        OriginalValue as PVOID,
        Offset,
        size_of::<ULONG>() as ULONG,
    );
    if Size != size_of::<ULONG>() as ULONG {
        DPRINT1!("Wrong size %lu\n", Size);
        return FALSE;
    }

    /* Write all ones to determine which bits are held to zero */
    let mut AllOnes: ULONG = MAXULONG;
    Size = HalSetBusDataByOffset(
        BUS_DATA_TYPE_PCIConfiguration,
        (*(*DeviceExtension).PciDevice).BusNumber,
        (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG,
        &mut AllOnes as *mut ULONG as PVOID,
        Offset,
        size_of::<ULONG>() as ULONG,
    );
    if Size != size_of::<ULONG>() as ULONG {
        DPRINT1!("Wrong size %lu\n", Size);
        return FALSE;
    }

    /* Get the range length */
    Size = HalGetBusDataByOffset(
        BUS_DATA_TYPE_PCIConfiguration,
        (*(*DeviceExtension).PciDevice).BusNumber,
        (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG,
        NewValue as PVOID,
        Offset,
        size_of::<ULONG>() as ULONG,
    );
    if Size != size_of::<ULONG>() as ULONG {
        DPRINT1!("Wrong size %lu\n", Size);
        return FALSE;
    }

    /* Restore original value */
    Size = HalSetBusDataByOffset(
        BUS_DATA_TYPE_PCIConfiguration,
        (*(*DeviceExtension).PciDevice).BusNumber,
        (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG,
        OriginalValue as PVOID,
        Offset,
        size_of::<ULONG>() as ULONG,
    );
    if Size != size_of::<ULONG>() as ULONG {
        DPRINT1!("Wrong size %lu\n", Size);
        return FALSE;
    }

    TRUE
}

/// The `union { struct { ULONG Bar0, Bar1; } Bars; ULONGLONG Bar; }` from the C.
/// Modeled as two u32 lanes with a combined u64 accessor (little-endian: Bar0 is
/// the low dword, Bar1 the high dword — matches the x86/arm64 LE layout the C
/// union relies on).
#[derive(Copy, Clone)]
struct BarUnion {
    bar0: u32,
    bar1: u32,
}
impl BarUnion {
    #[inline]
    fn new() -> Self {
        BarUnion { bar0: 0, bar1: 0 }
    }
    #[inline]
    fn bar(&self) -> u64 {
        (self.bar0 as u64) | ((self.bar1 as u64) << 32)
    }
}

unsafe fn PdoGetRangeLength(
    DeviceExtension: PPDO_DEVICE_EXTENSION,
    Bar: u8,
    Base: *mut u64,
    Length: *mut u64,
    Flags: *mut ULONG,
    NextBar: *mut u8,
    MaximumAddress: *mut u64,
) -> BOOLEAN {
    let mut OriginalValue = BarUnion::new();
    let mut NewValue = BarUnion::new();

    /* Compute the offset of this BAR in PCI config space */
    let Offset: ULONG = 0x10 + (Bar as ULONG) * 4;

    /* Assume this is a 32-bit BAR until we find wrong */
    *NextBar = Bar + 1;

    /* Read the first BAR */
    if PdoReadPciBar(
        DeviceExtension,
        Offset,
        &mut OriginalValue.bar0,
        &mut NewValue.bar0,
    ) == FALSE
    {
        return FALSE;
    }

    /* Check if this is a memory BAR */
    if (OriginalValue.bar0 & PCI_ADDRESS_IO_SPACE) == 0 {
        /* Write the maximum address if the caller asked for it */
        if !MaximumAddress.is_null() {
            if (OriginalValue.bar0 & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_32BIT {
                *MaximumAddress = 0x0000_0000_FFFF_FFFF;
            } else if (OriginalValue.bar0 & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_20BIT {
                *MaximumAddress = 0x0000_0000_000F_FFFF;
            } else if (OriginalValue.bar0 & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_64BIT {
                *MaximumAddress = 0xFFFF_FFFF_FFFF_FFFF;
            }
        }

        /* Check if this is a 64-bit BAR */
        if (OriginalValue.bar0 & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_64BIT {
            /* We've now consumed the next BAR too */
            *NextBar = Bar + 2;

            /* Read the next BAR */
            if PdoReadPciBar(
                DeviceExtension,
                Offset + 4,
                &mut OriginalValue.bar1,
                &mut NewValue.bar1,
            ) == FALSE
            {
                return FALSE;
            }
        }
    } else {
        /* Write the maximum I/O port address */
        if !MaximumAddress.is_null() {
            *MaximumAddress = 0x0000_0000_FFFF_FFFF;
        }
    }

    if NewValue.bar() == 0 {
        DPRINT!("Unused address register\n");
        *Base = 0;
        *Length = 0;
        *Flags = 0;
        return TRUE;
    }

    *Base = if (OriginalValue.bar() & PCI_ADDRESS_IO_SPACE as u64) != 0 {
        OriginalValue.bar() & PCI_ADDRESS_IO_ADDRESS_MASK_64
    } else {
        OriginalValue.bar() & PCI_ADDRESS_MEMORY_ADDRESS_MASK_64
    };

    let Size: u64 = if (NewValue.bar() & PCI_ADDRESS_IO_SPACE as u64) != 0 {
        NewValue.bar() & PCI_ADDRESS_IO_ADDRESS_MASK_64
    } else {
        NewValue.bar() & PCI_ADDRESS_MEMORY_ADDRESS_MASK_64
    };
    *Length = Size & !(Size - 1);

    *Flags = if (NewValue.bar() & PCI_ADDRESS_IO_SPACE as u64) != 0 {
        (NewValue.bar() & !PCI_ADDRESS_IO_ADDRESS_MASK_64) as ULONG
    } else {
        (NewValue.bar() & !PCI_ADDRESS_MEMORY_ADDRESS_MASK_64) as ULONG
    };

    TRUE
}

unsafe fn PdoQueryResourceRequirements(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    _IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    DPRINT!("PdoQueryResourceRequirements() called\n");

    let DeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    // C local `PCI_COMMON_CONFIG PciConfig`; HalGetBusData writes only
    // PCI_COMMON_HDR_LENGTH (64) bytes = the header — use the correct header type.
    // TODO(pci-rust): substrate's PCI_COMMON_CONFIG is the 192-byte (header-less)
    // bindgen blob; a header-typed local is the faithful storage for the bytes
    // HalGetBusData fills and the fields the code reads.
    let mut PciConfig: PCI_COMMON_HEADER = core::mem::zeroed();
    let PciConfigPtr: *const PCI_COMMON_HEADER = &PciConfig;

    /* Get PCI configuration space */
    let Size = HalGetBusData(
        BUS_DATA_TYPE_PCIConfiguration,
        (*(*DeviceExtension).PciDevice).BusNumber,
        (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG,
        &mut PciConfig as *mut PCI_COMMON_HEADER as PVOID,
        PCI_COMMON_HDR_LENGTH,
    );
    DPRINT!("Size %lu\n", Size);
    if Size < PCI_COMMON_HDR_LENGTH {
        (*Irp).IoStatus.Information = 0;
        return STATUS_UNSUCCESSFUL;
    }

    DPRINT!("Command register: 0x%04hx\n", PciConfig.Command);

    let mut Base: u64 = 0;
    let mut Length: u64 = 0;
    let mut Flags: ULONG = 0;
    let mut MaximumAddress: u64 = 0;

    /* Count required resource descriptors */
    let mut ResCount: ULONG = 0;
    if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_DEVICE_TYPE {
        let mut Bar: u8 = 0;
        while Bar < PCI_TYPE0_ADDRESSES {
            if PdoGetRangeLength(
                DeviceExtension,
                Bar,
                &mut Base,
                &mut Length,
                &mut Flags,
                &mut Bar,
                core::ptr::null_mut(),
            ) == FALSE
            {
                break;
            }
            if Length != 0 {
                ResCount += 2;
            }
        }

        /* FIXME: Check ROM address */

        if (*PciConfigPtr).u.type0.InterruptPin != 0 {
            ResCount += 1;
        }
    } else if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_BRIDGE_TYPE {
        let mut Bar: u8 = 0;
        while Bar < PCI_TYPE1_ADDRESSES {
            if PdoGetRangeLength(
                DeviceExtension,
                Bar,
                &mut Base,
                &mut Length,
                &mut Flags,
                &mut Bar,
                core::ptr::null_mut(),
            ) == FALSE
            {
                break;
            }
            if Length != 0 {
                ResCount += 2;
            }
        }

        if (*pci_config_header(addr_of!((*(*DeviceExtension).PciDevice).PciConfig)))
            .BaseClass
            == PCI_CLASS_BRIDGE_DEV
        {
            ResCount += 1;
        }
    } else if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_CARDBUS_BRIDGE_TYPE {
        /* FIXME: Count Cardbus bridge resources */
    } else {
        DPRINT1!(
            "Unsupported header type %d\n",
            PCI_CONFIGURATION_TYPE(PciConfigPtr) as core::ffi::c_uint
        );
    }

    if ResCount == 0 {
        (*Irp).IoStatus.Information = 0;
        return STATUS_SUCCESS;
    }

    /* Calculate the resource list size */
    // FIELD_OFFSET(IO_RESOURCE_REQUIREMENTS_LIST, List[0].Descriptors)
    let ListSize: ULONG = (offset_of!(IO_RESOURCE_REQUIREMENTS_LIST, List)
        + offset_of!(IO_RESOURCE_LIST, Descriptors)) as ULONG
        + ResCount * size_of::<IO_RESOURCE_DESCRIPTOR>() as ULONG;

    DPRINT!("ListSize %lu (0x%lx)\n", ListSize, ListSize);

    /* Allocate the resource requirements list */
    let ResourceList =
        ExAllocatePoolWithTag(PagedPool, ListSize as SIZE_T, TAG_PCI)
            as PIO_RESOURCE_REQUIREMENTS_LIST;
    if ResourceList.is_null() {
        (*Irp).IoStatus.Information = 0;
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // RtlZeroMemory — core::ptr::write_bytes (RtlZeroMemory is a DDK macro).
    core::ptr::write_bytes(ResourceList as *mut u8, 0, ListSize as usize);
    (*ResourceList).ListSize = ListSize;
    (*ResourceList).InterfaceType = INTERFACE_TYPE_PCIBus;
    (*ResourceList).BusNumber = (*(*DeviceExtension).PciDevice).BusNumber;
    (*ResourceList).SlotNumber =
        (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG;
    (*ResourceList).AlternativeLists = 1;

    let list0 = addr_of_mut!((*ResourceList).List) as *mut IO_RESOURCE_LIST;
    (*list0).Version = 1;
    (*list0).Revision = 1;
    (*list0).Count = ResCount;

    // Descriptor = &ResourceList->List[0].Descriptors[0]; advancing raw pointer.
    let mut Descriptor =
        addr_of_mut!((*list0).Descriptors) as *mut IO_RESOURCE_DESCRIPTOR;

    if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_DEVICE_TYPE {
        let mut Bar: u8 = 0;
        while Bar < PCI_TYPE0_ADDRESSES {
            if PdoGetRangeLength(
                DeviceExtension,
                Bar,
                &mut Base,
                &mut Length,
                &mut Flags,
                &mut Bar,
                &mut MaximumAddress,
            ) == FALSE
            {
                DPRINT1!("PdoGetRangeLength() failed\n");
                break;
            }

            if Length == 0 {
                DPRINT!("Unused address register\n");
                continue;
            }

            /* Set preferred descriptor */
            (*Descriptor).Option = IO_RESOURCE_PREFERRED;
            if (Flags & PCI_ADDRESS_IO_SPACE) != 0 {
                (*Descriptor).Type = CmResourceTypePort;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = CM_RESOURCE_PORT_IO
                    | CM_RESOURCE_PORT_16_BIT_DECODE
                    | CM_RESOURCE_PORT_POSITIVE_DECODE;

                (*Descriptor).u.Port.Length = Length as ULONG;
                (*Descriptor).u.Port.Alignment = 1;
                (*Descriptor).u.Port.MinimumAddress.QuadPart = Base as LONGLONG;
                (*Descriptor).u.Port.MaximumAddress.QuadPart =
                    (Base + Length - 1) as LONGLONG;
            } else {
                (*Descriptor).Type = CmResourceTypeMemory;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                // NOTE: faithful to the C operator-precedence bug — the `?:` binds
                // tighter than `|`, so this whole expression is just the ternary.
                (*Descriptor).Flags = if (CM_RESOURCE_MEMORY_READ_WRITE
                    | (Flags as u16 & PCI_ADDRESS_MEMORY_PREFETCHABLE as u16))
                    != 0
                {
                    CM_RESOURCE_MEMORY_PREFETCHABLE
                } else {
                    0
                };

                (*Descriptor).u.Memory.Length = Length as ULONG;
                (*Descriptor).u.Memory.Alignment = 1;
                (*Descriptor).u.Memory.MinimumAddress.QuadPart = Base as LONGLONG;
                (*Descriptor).u.Memory.MaximumAddress.QuadPart =
                    (Base + Length - 1) as LONGLONG;
            }
            Descriptor = Descriptor.add(1);

            /* Set alternative descriptor */
            (*Descriptor).Option = IO_RESOURCE_ALTERNATIVE;
            if (Flags & PCI_ADDRESS_IO_SPACE) != 0 {
                (*Descriptor).Type = CmResourceTypePort;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = CM_RESOURCE_PORT_IO
                    | CM_RESOURCE_PORT_16_BIT_DECODE
                    | CM_RESOURCE_PORT_POSITIVE_DECODE;

                (*Descriptor).u.Port.Length = Length as ULONG;
                (*Descriptor).u.Port.Alignment = Length as ULONG;
                (*Descriptor).u.Port.MinimumAddress.QuadPart = 0;
                (*Descriptor).u.Port.MaximumAddress.QuadPart =
                    MaximumAddress as LONGLONG;
            } else {
                (*Descriptor).Type = CmResourceTypeMemory;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = if (CM_RESOURCE_MEMORY_READ_WRITE
                    | (Flags as u16 & PCI_ADDRESS_MEMORY_PREFETCHABLE as u16))
                    != 0
                {
                    CM_RESOURCE_MEMORY_PREFETCHABLE
                } else {
                    0
                };

                (*Descriptor).u.Memory.Length = Length as ULONG;
                (*Descriptor).u.Memory.Alignment = Length as ULONG;
                // NOTE: the C writes u.Port here (not u.Memory) — faithful copy.
                (*Descriptor).u.Port.MinimumAddress.QuadPart = 0;
                (*Descriptor).u.Port.MaximumAddress.QuadPart =
                    MaximumAddress as LONGLONG;
            }
            Descriptor = Descriptor.add(1);
        }

        /* FIXME: Check ROM address */

        if (*PciConfigPtr).u.type0.InterruptPin != 0 {
            (*Descriptor).Option = 0; /* Required */
            (*Descriptor).Type = CmResourceTypeInterrupt;
            (*Descriptor).ShareDisposition = CmResourceShareShared;
            (*Descriptor).Flags = CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE;

            (*Descriptor).u.Interrupt.MinimumVector = 0;
            (*Descriptor).u.Interrupt.MaximumVector = 0xFF;
        }
    } else if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_BRIDGE_TYPE {
        let mut Bar: u8 = 0;
        while Bar < PCI_TYPE1_ADDRESSES {
            if PdoGetRangeLength(
                DeviceExtension,
                Bar,
                &mut Base,
                &mut Length,
                &mut Flags,
                &mut Bar,
                &mut MaximumAddress,
            ) == FALSE
            {
                DPRINT1!("PdoGetRangeLength() failed\n");
                break;
            }

            if Length == 0 {
                DPRINT!("Unused address register\n");
                continue;
            }

            /* Set preferred descriptor */
            (*Descriptor).Option = IO_RESOURCE_PREFERRED;
            if (Flags & PCI_ADDRESS_IO_SPACE) != 0 {
                (*Descriptor).Type = CmResourceTypePort;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = CM_RESOURCE_PORT_IO
                    | CM_RESOURCE_PORT_16_BIT_DECODE
                    | CM_RESOURCE_PORT_POSITIVE_DECODE;

                (*Descriptor).u.Port.Length = Length as ULONG;
                (*Descriptor).u.Port.Alignment = 1;
                (*Descriptor).u.Port.MinimumAddress.QuadPart = Base as LONGLONG;
                (*Descriptor).u.Port.MaximumAddress.QuadPart =
                    (Base + Length - 1) as LONGLONG;
            } else {
                (*Descriptor).Type = CmResourceTypeMemory;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = if (CM_RESOURCE_MEMORY_READ_WRITE
                    | (Flags as u16 & PCI_ADDRESS_MEMORY_PREFETCHABLE as u16))
                    != 0
                {
                    CM_RESOURCE_MEMORY_PREFETCHABLE
                } else {
                    0
                };

                (*Descriptor).u.Memory.Length = Length as ULONG;
                (*Descriptor).u.Memory.Alignment = 1;
                (*Descriptor).u.Memory.MinimumAddress.QuadPart = Base as LONGLONG;
                (*Descriptor).u.Memory.MaximumAddress.QuadPart =
                    (Base + Length - 1) as LONGLONG;
            }
            Descriptor = Descriptor.add(1);

            /* Set alternative descriptor */
            (*Descriptor).Option = IO_RESOURCE_ALTERNATIVE;
            if (Flags & PCI_ADDRESS_IO_SPACE) != 0 {
                (*Descriptor).Type = CmResourceTypePort;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = CM_RESOURCE_PORT_IO
                    | CM_RESOURCE_PORT_16_BIT_DECODE
                    | CM_RESOURCE_PORT_POSITIVE_DECODE;

                (*Descriptor).u.Port.Length = Length as ULONG;
                (*Descriptor).u.Port.Alignment = Length as ULONG;
                (*Descriptor).u.Port.MinimumAddress.QuadPart = 0;
                (*Descriptor).u.Port.MaximumAddress.QuadPart =
                    MaximumAddress as LONGLONG;
            } else {
                (*Descriptor).Type = CmResourceTypeMemory;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = if (CM_RESOURCE_MEMORY_READ_WRITE
                    | (Flags as u16 & PCI_ADDRESS_MEMORY_PREFETCHABLE as u16))
                    != 0
                {
                    CM_RESOURCE_MEMORY_PREFETCHABLE
                } else {
                    0
                };

                (*Descriptor).u.Memory.Length = Length as ULONG;
                (*Descriptor).u.Memory.Alignment = Length as ULONG;
                // NOTE: C writes u.Port here (not u.Memory) — faithful copy.
                (*Descriptor).u.Port.MinimumAddress.QuadPart = 0;
                (*Descriptor).u.Port.MaximumAddress.QuadPart =
                    MaximumAddress as LONGLONG;
            }
            Descriptor = Descriptor.add(1);
        }

        if (*pci_config_header(addr_of!((*(*DeviceExtension).PciDevice).PciConfig)))
            .BaseClass
            == PCI_CLASS_BRIDGE_DEV
        {
            (*Descriptor).Option = 0; /* Required */
            (*Descriptor).Type = CmResourceTypeBusNumber;
            (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;

            let secondary = (*pci_config_header(addr_of!(
                (*(*DeviceExtension).PciDevice).PciConfig
            )))
            .u
            .type1
            .SecondaryBus as ULONG
                // The segment rides above the bus (PCI_SEGMENT_BUS_NUMBER), so the
                // bridge's own FDO stays on its segment and two segments' bus 1 are
                // different resources to the PnP manager.
                | (*(*DeviceExtension).PciDevice).BusNumber & !0xFF;
            (*ResourceList).BusNumber = secondary;
            (*Descriptor).u.BusNumber.MinBusNumber = secondary;
            (*Descriptor).u.BusNumber.MaxBusNumber = secondary;
            (*Descriptor).u.BusNumber.Length = 1;
            (*Descriptor).u.BusNumber.Reserved = 0;
        }
    } else if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_CARDBUS_BRIDGE_TYPE {
        /* FIXME: Add Cardbus bridge resources */
    }

    (*Irp).IoStatus.Information = ResourceList as ULONG_PTR;

    STATUS_SUCCESS
}

unsafe fn PdoQueryResources(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    _IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    DPRINT!("PdoQueryResources() called\n");

    let DeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    // See PdoQueryResourceRequirements re: header-typed local.
    let mut PciConfig: PCI_COMMON_HEADER = core::mem::zeroed();
    let PciConfigPtr: *const PCI_COMMON_HEADER = &PciConfig;

    /* Get PCI configuration space */
    let Size = HalGetBusData(
        BUS_DATA_TYPE_PCIConfiguration,
        (*(*DeviceExtension).PciDevice).BusNumber,
        (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG,
        &mut PciConfig as *mut PCI_COMMON_HEADER as PVOID,
        PCI_COMMON_HDR_LENGTH,
    );
    DPRINT!("Size %lu\n", Size);
    if Size < PCI_COMMON_HDR_LENGTH {
        (*Irp).IoStatus.Information = 0;
        return STATUS_UNSUCCESSFUL;
    }

    DPRINT!("Command register: 0x%04hx\n", PciConfig.Command);

    let mut Base: u64 = 0;
    let mut Length: u64 = 0;
    let mut Flags: ULONG = 0;

    /* Count required resource descriptors */
    let mut ResCount: ULONG = 0;
    if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_DEVICE_TYPE {
        let mut Bar: u8 = 0;
        while Bar < PCI_TYPE0_ADDRESSES {
            if PdoGetRangeLength(
                DeviceExtension,
                Bar,
                &mut Base,
                &mut Length,
                &mut Flags,
                &mut Bar,
                core::ptr::null_mut(),
            ) == FALSE
            {
                break;
            }
            if Length != 0 {
                ResCount += 1;
            }
        }

        if (*PciConfigPtr).u.type0.InterruptPin != 0
            && (*PciConfigPtr).u.type0.InterruptLine != 0
            && (*PciConfigPtr).u.type0.InterruptLine != 0xFF
        {
            ResCount += 1;
        }
    } else if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_BRIDGE_TYPE {
        let mut Bar: u8 = 0;
        while Bar < PCI_TYPE1_ADDRESSES {
            if PdoGetRangeLength(
                DeviceExtension,
                Bar,
                &mut Base,
                &mut Length,
                &mut Flags,
                &mut Bar,
                core::ptr::null_mut(),
            ) == FALSE
            {
                break;
            }
            if Length != 0 {
                ResCount += 1;
            }
        }

        if (*pci_config_header(addr_of!((*(*DeviceExtension).PciDevice).PciConfig)))
            .BaseClass
            == PCI_CLASS_BRIDGE_DEV
        {
            ResCount += 1;
        }
    } else if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_CARDBUS_BRIDGE_TYPE {
        /* FIXME: Count Cardbus bridge resources */
    } else {
        DPRINT1!(
            "Unsupported header type %d\n",
            PCI_CONFIGURATION_TYPE(PciConfigPtr) as core::ffi::c_uint
        );
    }

    if ResCount == 0 {
        (*Irp).IoStatus.Information = 0;
        return STATUS_SUCCESS;
    }

    /* Calculate the resource list size */
    // FIELD_OFFSET(CM_RESOURCE_LIST, List[0].PartialResourceList.PartialDescriptors)
    let ListSize: ULONG = (offset_of!(CM_RESOURCE_LIST, List)
        + offset_of!(CM_FULL_RESOURCE_DESCRIPTOR, PartialResourceList)
        + offset_of!(CM_PARTIAL_RESOURCE_LIST, PartialDescriptors))
        as ULONG
        + ResCount * size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as ULONG;

    /* Allocate the resource list */
    let ResourceList =
        ExAllocatePoolWithTag(PagedPool, ListSize as SIZE_T, TAG_PCI)
            as PCM_RESOURCE_LIST;
    if ResourceList.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // RtlZeroMemory — core::ptr::write_bytes (RtlZeroMemory is a DDK macro).
    core::ptr::write_bytes(ResourceList as *mut u8, 0, ListSize as usize);
    (*ResourceList).Count = 1;
    let full0 = addr_of_mut!((*ResourceList).List) as *mut CM_FULL_RESOURCE_DESCRIPTOR;
    (*full0).InterfaceType = INTERFACE_TYPE_PCIBus;
    (*full0).BusNumber = (*(*DeviceExtension).PciDevice).BusNumber;

    let PartialList = addr_of_mut!((*full0).PartialResourceList);
    (*PartialList).Version = 1;
    (*PartialList).Revision = 1;
    (*PartialList).Count = ResCount;

    let mut Descriptor =
        addr_of_mut!((*PartialList).PartialDescriptors) as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR;

    if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_DEVICE_TYPE {
        let mut Bar: u8 = 0;
        while Bar < PCI_TYPE0_ADDRESSES {
            if PdoGetRangeLength(
                DeviceExtension,
                Bar,
                &mut Base,
                &mut Length,
                &mut Flags,
                &mut Bar,
                core::ptr::null_mut(),
            ) == FALSE
            {
                break;
            }

            if Length == 0 {
                DPRINT!("Unused address register\n");
                continue;
            }

            if (Flags & PCI_ADDRESS_IO_SPACE) != 0 {
                (*Descriptor).Type = CmResourceTypePort;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = CM_RESOURCE_PORT_IO
                    | CM_RESOURCE_PORT_16_BIT_DECODE
                    | CM_RESOURCE_PORT_POSITIVE_DECODE;
                (*Descriptor).u.Port.Start.QuadPart = Base as LONGLONG;
                (*Descriptor).u.Port.Length = Length as ULONG;

                /* Enable IO space access */
                (*(*DeviceExtension).PciDevice).EnableIoSpace = TRUE;
            } else {
                (*Descriptor).Type = CmResourceTypeMemory;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = if (CM_RESOURCE_MEMORY_READ_WRITE
                    | (Flags as u16 & PCI_ADDRESS_MEMORY_PREFETCHABLE as u16))
                    != 0
                {
                    CM_RESOURCE_MEMORY_PREFETCHABLE
                } else {
                    0
                };
                (*Descriptor).u.Memory.Start.QuadPart = Base as LONGLONG;
                (*Descriptor).u.Memory.Length = Length as ULONG;

                /* Enable memory space access */
                (*(*DeviceExtension).PciDevice).EnableMemorySpace = TRUE;
            }

            Descriptor = Descriptor.add(1);
        }

        /* Add interrupt resource */
        if (*PciConfigPtr).u.type0.InterruptPin != 0
            && (*PciConfigPtr).u.type0.InterruptLine != 0
            && (*PciConfigPtr).u.type0.InterruptLine != 0xFF
        {
            (*Descriptor).Type = CmResourceTypeInterrupt;
            (*Descriptor).ShareDisposition = CmResourceShareShared;
            (*Descriptor).Flags = CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE;
            (*Descriptor).u.Interrupt.Level =
                (*PciConfigPtr).u.type0.InterruptLine as ULONG;
            (*Descriptor).u.Interrupt.Vector =
                (*PciConfigPtr).u.type0.InterruptLine as ULONG;
            (*Descriptor).u.Interrupt.Affinity = 0xFFFF_FFFF;
        }

        /* Allow bus master mode */
        (*(*DeviceExtension).PciDevice).EnableBusMaster = TRUE;
    } else if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_BRIDGE_TYPE {
        let mut Bar: u8 = 0;
        while Bar < PCI_TYPE1_ADDRESSES {
            if PdoGetRangeLength(
                DeviceExtension,
                Bar,
                &mut Base,
                &mut Length,
                &mut Flags,
                &mut Bar,
                core::ptr::null_mut(),
            ) == FALSE
            {
                break;
            }

            if Length == 0 {
                DPRINT!("Unused address register\n");
                continue;
            }

            if (Flags & PCI_ADDRESS_IO_SPACE) != 0 {
                (*Descriptor).Type = CmResourceTypePort;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = CM_RESOURCE_PORT_IO
                    | CM_RESOURCE_PORT_16_BIT_DECODE
                    | CM_RESOURCE_PORT_POSITIVE_DECODE;
                (*Descriptor).u.Port.Start.QuadPart = Base as LONGLONG;
                (*Descriptor).u.Port.Length = Length as ULONG;

                /* Enable IO space access */
                (*(*DeviceExtension).PciDevice).EnableIoSpace = TRUE;
            } else {
                (*Descriptor).Type = CmResourceTypeMemory;
                (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;
                (*Descriptor).Flags = if (CM_RESOURCE_MEMORY_READ_WRITE
                    | (Flags as u16 & PCI_ADDRESS_MEMORY_PREFETCHABLE as u16))
                    != 0
                {
                    CM_RESOURCE_MEMORY_PREFETCHABLE
                } else {
                    0
                };
                (*Descriptor).u.Memory.Start.QuadPart = Base as LONGLONG;
                (*Descriptor).u.Memory.Length = Length as ULONG;

                /* Enable memory space access */
                (*(*DeviceExtension).PciDevice).EnableMemorySpace = TRUE;
            }

            Descriptor = Descriptor.add(1);
        }

        if (*pci_config_header(addr_of!((*(*DeviceExtension).PciDevice).PciConfig)))
            .BaseClass
            == PCI_CLASS_BRIDGE_DEV
        {
            (*Descriptor).Type = CmResourceTypeBusNumber;
            (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;

            let secondary = (*pci_config_header(addr_of!(
                (*(*DeviceExtension).PciDevice).PciConfig
            )))
            .u
            .type1
            .SecondaryBus as ULONG
                // The segment rides above the bus (PCI_SEGMENT_BUS_NUMBER), so the
                // bridge's own FDO stays on its segment and two segments' bus 1 are
                // different resources to the PnP manager.
                | (*(*DeviceExtension).PciDevice).BusNumber & !0xFF;
            (*full0).BusNumber = secondary;
            (*Descriptor).u.BusNumber.Start = secondary;
            (*Descriptor).u.BusNumber.Length = 1;
            (*Descriptor).u.BusNumber.Reserved = 0;
        }
    } else if PCI_CONFIGURATION_TYPE(PciConfigPtr) == PCI_CARDBUS_BRIDGE_TYPE {
        /* FIXME: Add Cardbus bridge resources */
    }

    (*Irp).IoStatus.Information = ResourceList as ULONG_PTR;

    STATUS_SUCCESS
}

// ---------------------------------------------------------------------------
// Interface callbacks — installed into BUS_INTERFACE_STANDARD /
// PCI_DEVICE_PRESENT_INTERFACE / INTERFACE callback slots via transmute
// fn-ptr (BRIEF gotcha #1). Kept `#[no_mangle] extern "system"` (NTAPI).
// ---------------------------------------------------------------------------

#[no_mangle]
unsafe extern "system" fn InterfaceReference(Context: PVOID) {
    DPRINT!("InterfaceReference(%p)\n", Context);

    let DeviceExtension =
        (*(Context as PDEVICE_OBJECT)).DeviceExtension as PPDO_DEVICE_EXTENSION;
    InterlockedIncrement(addr_of_mut!((*DeviceExtension).References));
}

#[no_mangle]
unsafe extern "system" fn InterfaceDereference(Context: PVOID) {
    DPRINT!("InterfaceDereference(%p)\n", Context);

    let DeviceExtension =
        (*(Context as PDEVICE_OBJECT)).DeviceExtension as PPDO_DEVICE_EXTENSION;
    InterlockedDecrement(addr_of_mut!((*DeviceExtension).References));
}

#[no_mangle]
unsafe extern "system" fn InterfaceBusTranslateBusAddress(
    Context: PVOID,
    BusAddress: PHYSICAL_ADDRESS,
    Length: ULONG,
    AddressSpace: *mut ULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    let _ = Length;
    DPRINT!("InterfaceBusTranslateBusAddress(...)\n");

    let DeviceExtension =
        (*(Context as PDEVICE_OBJECT)).DeviceExtension as PPDO_DEVICE_EXTENSION;

    HalTranslateBusAddress(
        INTERFACE_TYPE_PCIBus,
        (*(*DeviceExtension).PciDevice).BusNumber,
        BusAddress,
        AddressSpace,
        TranslatedAddress,
    )
}

#[no_mangle]
unsafe extern "system" fn InterfaceBusGetDmaAdapter(
    Context: PVOID,
    DeviceDescription: PDEVICE_DESCRIPTION,
    NumberOfMapRegisters: *mut ULONG,
) -> PDMA_ADAPTER {
    let _ = Context;
    DPRINT!("InterfaceBusGetDmaAdapter(...)\n");
    HalGetAdapter(DeviceDescription, NumberOfMapRegisters) as PDMA_ADAPTER
}

#[no_mangle]
unsafe extern "system" fn InterfaceBusSetBusData(
    Context: PVOID,
    DataType: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    DPRINT!("InterfaceBusSetBusData(...)\n");

    // PCIConfiguration too (upstream e7518a9c796): StorPort miniports ask for it by that name.
    if DataType != PCI_WHICHSPACE_CONFIG && DataType != BUS_DATA_TYPE_PCIConfiguration as ULONG {
        DPRINT!("Unknown DataType %lu\n", DataType);
        return 0;
    }

    let DeviceExtension =
        (*(Context as PDEVICE_OBJECT)).DeviceExtension as PPDO_DEVICE_EXTENSION;

    /* Get PCI configuration space */
    HalSetBusDataByOffset(
        BUS_DATA_TYPE_PCIConfiguration,
        (*(*DeviceExtension).PciDevice).BusNumber,
        (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG,
        Buffer,
        Offset,
        Length,
    )
}

#[no_mangle]
unsafe extern "system" fn InterfaceBusGetBusData(
    Context: PVOID,
    DataType: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    DPRINT!("InterfaceBusGetBusData(...) called\n");

    // PCIConfiguration too (upstream e7518a9c796): StorPort miniports ask for it by that name.
    if DataType != PCI_WHICHSPACE_CONFIG && DataType != BUS_DATA_TYPE_PCIConfiguration as ULONG {
        DPRINT!("Unknown DataType %lu\n", DataType);
        return 0;
    }

    let DeviceExtension =
        (*(Context as PDEVICE_OBJECT)).DeviceExtension as PPDO_DEVICE_EXTENSION;

    /* Get PCI configuration space */
    HalGetBusDataByOffset(
        BUS_DATA_TYPE_PCIConfiguration,
        (*(*DeviceExtension).PciDevice).BusNumber,
        (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG,
        Buffer,
        Offset,
        Length,
    )
}

#[no_mangle]
unsafe extern "system" fn InterfacePciDevicePresent(
    VendorID: USHORT,
    DeviceID: USHORT,
    RevisionID: UCHAR,
    SubVendorID: USHORT,
    SubSystemID: USHORT,
    Flags: ULONG,
) -> BOOLEAN {
    let mut Found: BOOLEAN = FALSE;
    let mut OldIrql: KIRQL = 0;

    KeAcquireSpinLock(
        addr_of_mut!((*crate::shared::DriverExtension).BusListLock).cast(),
        &mut OldIrql,
    );
    let mut CurrentBus = (*crate::shared::DriverExtension).BusListHead.Flink;
    while Found == FALSE
        && CurrentBus != addr_of_mut!((*crate::shared::DriverExtension).BusListHead)
    {
        // CONTAINING_RECORD(CurrentBus, FDO_DEVICE_EXTENSION, ListEntry)
        let FdoDeviceExtension = (CurrentBus as *mut u8)
            .sub(offset_of!(crate::shared::FDO_DEVICE_EXTENSION, ListEntry))
            as PFDO_DEVICE_EXTENSION;

        KeAcquireSpinLockAtDpcLevel(addr_of_mut!((*FdoDeviceExtension).DeviceListLock).cast());
        let mut CurrentEntry = (*FdoDeviceExtension).DeviceListHead.Flink;
        while Found == FALSE
            && CurrentEntry != addr_of_mut!((*FdoDeviceExtension).DeviceListHead)
        {
            // CONTAINING_RECORD(CurrentEntry, PCI_DEVICE, ListEntry)
            let PciDevice = (CurrentEntry as *mut u8)
                .sub(offset_of!(PCI_DEVICE, ListEntry))
                as *mut PCI_DEVICE;

            let cfg = pci_config_header(addr_of!((*PciDevice).PciConfig));
            if (*cfg).VendorID == VendorID && (*cfg).DeviceID == DeviceID {
                if (Flags & PCI_USE_SUBSYSTEM_IDS) == 0
                    || ((*cfg).u.type0.SubVendorID == SubVendorID
                        && (*cfg).u.type0.SubSystemID == SubSystemID)
                {
                    if (Flags & PCI_USE_REVISION) == 0
                        || (*cfg).RevisionID == RevisionID
                    {
                        DPRINT!("Found the PCI device\n");
                        Found = TRUE;
                    }
                }
            }

            CurrentEntry = (*CurrentEntry).Flink;
        }

        KeReleaseSpinLockFromDpcLevel(addr_of_mut!((*FdoDeviceExtension).DeviceListLock).cast());
        CurrentBus = (*CurrentBus).Flink;
    }
    KeReleaseSpinLock(
        addr_of_mut!((*crate::shared::DriverExtension).BusListLock).cast(),
        OldIrql,
    );

    Found
}

unsafe fn CheckPciDevice(
    PciConfig: *const PCI_COMMON_HEADER,
    Parameters: *const PCI_DEVICE_PRESENCE_PARAMETERS,
) -> BOOLEAN {
    if ((*Parameters).Flags & PCI_USE_VENDEV_IDS) != 0
        && ((*PciConfig).VendorID != (*Parameters).VendorID
            || (*PciConfig).DeviceID != (*Parameters).DeviceID)
    {
        return FALSE;
    }

    if ((*Parameters).Flags & PCI_USE_CLASS_SUBCLASS) != 0
        && ((*PciConfig).BaseClass != (*Parameters).BaseClass
            || (*PciConfig).SubClass != (*Parameters).SubClass)
    {
        return FALSE;
    }

    if ((*Parameters).Flags & PCI_USE_PROGIF) != 0
        && (*PciConfig).ProgIf != (*Parameters).ProgIf
    {
        return FALSE;
    }

    if ((*Parameters).Flags & PCI_USE_SUBSYSTEM_IDS) != 0
        && ((*PciConfig).u.type0.SubVendorID != (*Parameters).SubVendorID
            || (*PciConfig).u.type0.SubSystemID != (*Parameters).SubSystemID)
    {
        return FALSE;
    }

    if ((*Parameters).Flags & PCI_USE_REVISION) != 0
        && (*PciConfig).RevisionID != (*Parameters).RevisionID
    {
        return FALSE;
    }

    TRUE
}

#[no_mangle]
unsafe extern "system" fn InterfacePciDevicePresentEx(
    Context: PVOID,
    Parameters: *mut PCI_DEVICE_PRESENCE_PARAMETERS,
) -> BOOLEAN {
    let mut Found: BOOLEAN = FALSE;
    let mut OldIrql: KIRQL = 0;

    DPRINT!("InterfacePciDevicePresentEx(%p %p) called\n", Context, Parameters);

    if Parameters.is_null()
        || (*Parameters).Size != size_of::<PCI_DEVICE_PRESENCE_PARAMETERS>() as ULONG
    {
        return FALSE;
    }

    let DeviceExtension =
        (*(Context as PDEVICE_OBJECT)).DeviceExtension as PPDO_DEVICE_EXTENSION;
    let MyFdoDeviceExtension =
        (*(*DeviceExtension).Fdo).DeviceExtension as PFDO_DEVICE_EXTENSION;

    if ((*Parameters).Flags & PCI_USE_LOCAL_DEVICE) != 0 {
        return CheckPciDevice(
            pci_config_header(addr_of!((*(*DeviceExtension).PciDevice).PciConfig)),
            Parameters,
        );
    }

    KeAcquireSpinLock(
        addr_of_mut!((*crate::shared::DriverExtension).BusListLock).cast(),
        &mut OldIrql,
    );
    let mut CurrentBus = (*crate::shared::DriverExtension).BusListHead.Flink;
    while Found == FALSE
        && CurrentBus != addr_of_mut!((*crate::shared::DriverExtension).BusListHead)
    {
        let FdoDeviceExtension = (CurrentBus as *mut u8)
            .sub(offset_of!(crate::shared::FDO_DEVICE_EXTENSION, ListEntry))
            as PFDO_DEVICE_EXTENSION;
        if ((*Parameters).Flags & PCI_USE_LOCAL_BUS) == 0
            || FdoDeviceExtension == MyFdoDeviceExtension
        {
            KeAcquireSpinLockAtDpcLevel(addr_of_mut!((*FdoDeviceExtension).DeviceListLock).cast());
            let mut CurrentEntry = (*FdoDeviceExtension).DeviceListHead.Flink;
            while Found == FALSE
                && CurrentEntry != addr_of_mut!((*FdoDeviceExtension).DeviceListHead)
            {
                let PciDevice = (CurrentEntry as *mut u8)
                    .sub(offset_of!(PCI_DEVICE, ListEntry))
                    as *mut PCI_DEVICE;

                if CheckPciDevice(
                    pci_config_header(addr_of!((*PciDevice).PciConfig)),
                    Parameters,
                ) != FALSE
                {
                    DPRINT!("Found the PCI device\n");
                    Found = TRUE;
                }

                CurrentEntry = (*CurrentEntry).Flink;
            }

            KeReleaseSpinLockFromDpcLevel(addr_of_mut!((*FdoDeviceExtension).DeviceListLock).cast());
        }
        CurrentBus = (*CurrentBus).Flink;
    }
    KeReleaseSpinLock(
        addr_of_mut!((*crate::shared::DriverExtension).BusListLock).cast(),
        OldIrql,
    );

    Found
}

unsafe fn PdoQueryInterface(
    DeviceObject: PDEVICE_OBJECT,
    _Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if RtlCompareMemory(
        (*IrpSp).Parameters.QueryInterface.InterfaceType as *const core::ffi::c_void,
        &GUID_BUS_INTERFACE_STANDARD as *const GUID as *const core::ffi::c_void,
        size_of::<GUID>() as SIZE_T,
    ) == size_of::<GUID>() as SIZE_T
    {
        /* BUS_INTERFACE_STANDARD */
        if (*IrpSp).Parameters.QueryInterface.Version < 1 {
            Status = STATUS_NOT_SUPPORTED;
        } else if ((*IrpSp).Parameters.QueryInterface.Size as usize)
            < size_of::<BUS_INTERFACE_STANDARD>()
        {
            Status = STATUS_BUFFER_TOO_SMALL;
        } else {
            let BusInterface =
                (*IrpSp).Parameters.QueryInterface.Interface as PBUS_INTERFACE_STANDARD;
            (*BusInterface).Size = size_of::<BUS_INTERFACE_STANDARD>() as USHORT;
            (*BusInterface).Version = 1;
            (*BusInterface).TranslateBusAddress =
                Some(transmute(InterfaceBusTranslateBusAddress as *const ()));
            (*BusInterface).GetDmaAdapter =
                Some(transmute(InterfaceBusGetDmaAdapter as *const ()));
            (*BusInterface).SetBusData =
                Some(transmute(InterfaceBusSetBusData as *const ()));
            (*BusInterface).GetBusData =
                Some(transmute(InterfaceBusGetBusData as *const ()));
            Status = STATUS_SUCCESS;
        }
    } else if RtlCompareMemory(
        (*IrpSp).Parameters.QueryInterface.InterfaceType as *const core::ffi::c_void,
        &GUID_PCI_DEVICE_PRESENT_INTERFACE as *const GUID as *const core::ffi::c_void,
        size_of::<GUID>() as SIZE_T,
    ) == size_of::<GUID>() as SIZE_T
    {
        /* PCI_DEVICE_PRESENT_INTERFACE */
        if (*IrpSp).Parameters.QueryInterface.Version < 1 {
            Status = STATUS_NOT_SUPPORTED;
        } else if ((*IrpSp).Parameters.QueryInterface.Size as usize)
            < size_of::<PCI_DEVICE_PRESENT_INTERFACE>()
        {
            Status = STATUS_BUFFER_TOO_SMALL;
        } else {
            let PciDevicePresentInterface =
                (*IrpSp).Parameters.QueryInterface.Interface as PPCI_DEVICE_PRESENT_INTERFACE;
            (*PciDevicePresentInterface).Size =
                size_of::<PCI_DEVICE_PRESENT_INTERFACE>() as USHORT;
            (*PciDevicePresentInterface).Version = 1;
            (*PciDevicePresentInterface).IsDevicePresent =
                Some(transmute(InterfacePciDevicePresent as *const ()));
            (*PciDevicePresentInterface).IsDevicePresentEx =
                Some(transmute(InterfacePciDevicePresentEx as *const ()));
            Status = STATUS_SUCCESS;
        }
    } else {
        /* Not a supported interface */
        return STATUS_NOT_SUPPORTED;
    }

    if NT_SUCCESS(Status) {
        /* Add a reference for the returned interface */
        let Interface = (*IrpSp).Parameters.QueryInterface.Interface as PINTERFACE;
        (*Interface).Context = DeviceObject as PVOID;
        (*Interface).InterfaceReference =
            Some(transmute(InterfaceReference as *const ()));
        (*Interface).InterfaceDereference =
            Some(transmute(InterfaceDereference as *const ()));
        // Interface->InterfaceReference(Interface->Context)
        ((*Interface).InterfaceReference.unwrap())((*Interface).Context);
    }

    Status
}

/// CmiGetNextResourceDescriptor — advance past a variable-size full resource
/// descriptor (CM_FULL_RESOURCE_DESCRIPTOR with a trailing
/// PartialResourceList.PartialDescriptors[Count]).
///
/// This helper is not exported by ntkrnl-sys, so it is reproduced inline.
/// Layout (from cmreslist.h CmiGetNextResourceDescriptor):
///   next = (CM_FULL_RESOURCE_DESCRIPTOR*)((PCHAR)(desc + 1)
///          + (desc->PartialResourceList.Count - 1)
///            * sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR));
/// i.e. one base record + (Count - 1) extra partial descriptors (the record
/// already includes PartialDescriptors[1]).
/// TODO(pci-rust): assumes no variable-size CmResourceTypeDeviceSpecific entries
/// expand the trailing partial descriptors (same assumption as the C macro).
#[inline]
unsafe fn CmiGetNextResourceDescriptor(
    desc: *mut CM_FULL_RESOURCE_DESCRIPTOR,
) -> *mut CM_FULL_RESOURCE_DESCRIPTOR {
    let count = (*desc).PartialResourceList.Count;
    let base = desc.add(1) as *mut u8;
    base.add(
        (count.wrapping_sub(1)) as usize
            * size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(),
    ) as *mut CM_FULL_RESOURCE_DESCRIPTOR
}

unsafe fn PdoStartDevice(
    DeviceObject: PDEVICE_OBJECT,
    _Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    let RawResList = (*IrpSp).Parameters.StartDevice.AllocatedResources;
    let DeviceExtension = (*DeviceObject).DeviceExtension as PPDO_DEVICE_EXTENSION;

    if RawResList.is_null() {
        return STATUS_SUCCESS;
    }

    /* TODO: Assign the other resources we get to the card */

    let mut RawFullDesc =
        addr_of_mut!((*RawResList).List) as *mut CM_FULL_RESOURCE_DESCRIPTOR;
    let mut i: ULONG = 0;
    while i < (*RawResList).Count {
        let mut ii: ULONG = 0;
        while ii < (*RawFullDesc).PartialResourceList.Count {
            /* Partial resource descriptors can be of variable size
               (CmResourceTypeDeviceSpecific), but only one is allowed and it
               must be the last one in the list! */
            let RawPartialDesc =
                (addr_of_mut!((*RawFullDesc).PartialResourceList.PartialDescriptors)
                    as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                    .add(ii as usize);

            // A message's raw vector is a token, not a line to write back.
            if (*RawPartialDesc).Type == CmResourceTypeInterrupt
                && (*RawPartialDesc).Flags & crate::msix::CM_RESOURCE_INTERRUPT_MESSAGE == 0
            {
                DPRINT!("Assigning IRQ ...\n");

                let Irq: UCHAR = (*RawPartialDesc).u.Interrupt.Vector as UCHAR;
                HalSetBusDataByOffset(
                    BUS_DATA_TYPE_PCIConfiguration,
                    (*(*DeviceExtension).PciDevice).BusNumber,
                    (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG,
                    &Irq as *const UCHAR as PVOID,
                    0x3c, /* PCI_INTERRUPT_LINE */
                    size_of::<UCHAR>() as ULONG,
                );
            }
            ii += 1;
        }
        RawFullDesc = CmiGetNextResourceDescriptor(RawFullDesc);
        i += 1;
    }

    let mut Command: USHORT = 0;

    DBGPRINT!("pci!PdoStartDevice: Enabling command flags ...");
    if (*(*DeviceExtension).PciDevice).EnableBusMaster != FALSE {
        Command |= PCI_ENABLE_BUS_MASTER;
        DBGPRINT!("[Bus master] ");
    }

    if (*(*DeviceExtension).PciDevice).EnableMemorySpace != FALSE {
        Command |= PCI_ENABLE_MEMORY_SPACE;
        DBGPRINT!("[Memory space enable] ");
    }

    if (*(*DeviceExtension).PciDevice).EnableIoSpace != FALSE {
        Command |= PCI_ENABLE_IO_SPACE;
        DBGPRINT!("[I/O space enable] ");
    }

    if Command != 0 {
        DBGPRINT!("\n");

        /* OR with the previous value */
        Command |= (*pci_config_header(addr_of!(
            (*(*DeviceExtension).PciDevice).PciConfig
        )))
        .Command;

        HalSetBusDataByOffset(
            BUS_DATA_TYPE_PCIConfiguration,
            (*(*DeviceExtension).PciDevice).BusNumber,
            (*(*DeviceExtension).PciDevice).SlotNumber.u.AsULONG,
            &mut Command as *mut USHORT as PVOID,
            offset_of!(PCI_COMMON_HEADER, Command) as ULONG,
            size_of::<USHORT>() as ULONG,
        );
    } else {
        DBGPRINT!("None\n");
    }

    // After the decodes, not before: an MSI-X table lives in a memory BAR, and writes
    // to a BAR the device is not decoding go nowhere. A device the firmware left
    // decoding (QEMU's NVMe) hid that; QEMU's xHCI, which UEFI turns off at exit, sat
    // with every table entry masked and the boot hung on its first USB transfer.
    crate::msix::program(
        DeviceExtension,
        RawResList,
        (*IrpSp).Parameters.StartDevice.AllocatedResourcesTranslated,
    );

    STATUS_SUCCESS
}

unsafe fn PdoReadConfig(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    DPRINT!("PdoReadConfig() called\n");

    let Size = InterfaceBusGetBusData(
        DeviceObject as PVOID,
        (*IrpSp).Parameters.ReadWriteConfig.WhichSpace,
        (*IrpSp).Parameters.ReadWriteConfig.Buffer,
        (*IrpSp).Parameters.ReadWriteConfig.Offset,
        (*IrpSp).Parameters.ReadWriteConfig.Length,
    );

    if Size != (*IrpSp).Parameters.ReadWriteConfig.Length {
        DPRINT1!("Size %lu  Length %lu\n", Size, (*IrpSp).Parameters.ReadWriteConfig.Length);
        (*Irp).IoStatus.Information = 0;
        return STATUS_UNSUCCESSFUL;
    }

    (*Irp).IoStatus.Information = Size as ULONG_PTR;

    STATUS_SUCCESS
}

unsafe fn PdoWriteConfig(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    DPRINT1!("PdoWriteConfig() called\n");

    /* Get PCI configuration space */
    let Size = InterfaceBusSetBusData(
        DeviceObject as PVOID,
        (*IrpSp).Parameters.ReadWriteConfig.WhichSpace,
        (*IrpSp).Parameters.ReadWriteConfig.Buffer,
        (*IrpSp).Parameters.ReadWriteConfig.Offset,
        (*IrpSp).Parameters.ReadWriteConfig.Length,
    );

    if Size != (*IrpSp).Parameters.ReadWriteConfig.Length {
        DPRINT1!("Size %lu  Length %lu\n", Size, (*IrpSp).Parameters.ReadWriteConfig.Length);
        (*Irp).IoStatus.Information = 0;
        return STATUS_UNSUCCESSFUL;
    }

    (*Irp).IoStatus.Information = Size as ULONG_PTR;

    STATUS_SUCCESS
}

unsafe fn PdoQueryDeviceRelations(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    IrpSp: PIO_STACK_LOCATION,
) -> NTSTATUS {
    /* We only support TargetDeviceRelation for child PDOs */
    if (*IrpSp).Parameters.QueryDeviceRelations.Type != TargetDeviceRelation {
        return (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    /* We can do this because we only return 1 PDO for TargetDeviceRelation */
    let DeviceRelations =
        ExAllocatePoolWithTag(PagedPool, size_of::<DEVICE_RELATIONS>() as SIZE_T, TAG_PCI)
            as PDEVICE_RELATIONS;
    if DeviceRelations.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*DeviceRelations).Count = 1;
    // Objects is [PDEVICE_OBJECT; 1] flexible array — write [0] via raw ptr.
    *(addr_of_mut!((*DeviceRelations).Objects) as *mut PDEVICE_OBJECT) = DeviceObject;

    /* The PnP manager will remove this when it is done with the PDO */
    ObReferenceObject(DeviceObject as PVOID);

    (*Irp).IoStatus.Information = DeviceRelations as ULONG_PTR;

    STATUS_SUCCESS
}

/*** PUBLIC ******************************************************************/

/// Handle Plug and Play IRPs for the child device.
#[no_mangle]
pub unsafe extern "system" fn PdoPnpControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    DPRINT!("Called\n");

    let mut Status: NTSTATUS = (*Irp).IoStatus.__bindgen_anon_1.Status;

    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    match (*IrpSp).MinorFunction {
        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            DPRINT!("Unimplemented IRP_MN_DEVICE_USAGE_NOTIFICATION received\n");
        }
        IRP_MN_EJECT => {
            DPRINT!("Unimplemented IRP_MN_EJECT received\n");
        }
        IRP_MN_QUERY_BUS_INFORMATION => {
            Status = PdoQueryBusInformation(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_QUERY_CAPABILITIES => {
            Status = PdoQueryCapabilities(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            Status = PdoQueryDeviceRelations(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_QUERY_DEVICE_TEXT => {
            DPRINT!("IRP_MN_QUERY_DEVICE_TEXT received\n");
            Status = PdoQueryDeviceText(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_QUERY_ID => {
            DPRINT!("IRP_MN_QUERY_ID received\n");
            Status = PdoQueryId(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            DPRINT!("Unimplemented IRP_MN_QUERY_ID received\n");
        }
        IRP_MN_QUERY_RESOURCE_REQUIREMENTS => {
            DPRINT!("IRP_MN_QUERY_RESOURCE_REQUIREMENTS received\n");
            Status = PdoQueryResourceRequirements(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_QUERY_RESOURCES => {
            DPRINT!("IRP_MN_QUERY_RESOURCES received\n");
            Status = PdoQueryResources(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_SET_LOCK => {
            DPRINT!("Unimplemented IRP_MN_SET_LOCK received\n");
        }
        IRP_MN_START_DEVICE => {
            Status = PdoStartDevice(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_STOP_DEVICE | IRP_MN_REMOVE_DEVICE | IRP_MN_SURPRISE_REMOVAL => {
            crate::msix::disable((*DeviceObject).DeviceExtension as crate::shared::PPDO_DEVICE_EXTENSION);
            Status = STATUS_SUCCESS;
        }
        IRP_MN_QUERY_STOP_DEVICE
        | IRP_MN_CANCEL_STOP_DEVICE
        | IRP_MN_QUERY_REMOVE_DEVICE
        | IRP_MN_CANCEL_REMOVE_DEVICE => {
            Status = STATUS_SUCCESS;
        }
        IRP_MN_QUERY_INTERFACE => {
            DPRINT!("IRP_MN_QUERY_INTERFACE received\n");
            Status = PdoQueryInterface(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_READ_CONFIG => {
            DPRINT!("IRP_MN_READ_CONFIG received\n");
            Status = PdoReadConfig(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_WRITE_CONFIG => {
            DPRINT!("IRP_MN_WRITE_CONFIG received\n");
            Status = PdoWriteConfig(DeviceObject, Irp, IrpSp);
        }
        IRP_MN_FILTER_RESOURCE_REQUIREMENTS => {
            DPRINT!("IRP_MN_FILTER_RESOURCE_REQUIREMENTS received\n");
            Status = crate::msix::filter_requirements(DeviceObject, Irp, IrpSp);
        }
        _ => {
            DPRINT1!("Unknown IOCTL 0x%lx\n", (*IrpSp).MinorFunction as core::ffi::c_uint);
        }
    }

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
    }

    DPRINT!("Leaving. Status 0x%X\n", Status);

    Status
}

/// Handle power management IRPs for the child device.
#[no_mangle]
pub unsafe extern "system" fn PdoPowerControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let _ = DeviceObject;
    let mut Status: NTSTATUS = (*Irp).IoStatus.__bindgen_anon_1.Status;

    DPRINT!("Called\n");

    let IrpSp = IoGetCurrentIrpStackLocation(Irp);

    match (*IrpSp).MinorFunction {
        IRP_MN_QUERY_POWER | IRP_MN_SET_POWER => {
            Status = STATUS_SUCCESS;
        }
        _ => {}
    }

    PoStartNextPowerIrp(Irp);
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    DPRINT!("Leaving. Status 0x%X\n", Status);

    Status
}

/* EOF */
