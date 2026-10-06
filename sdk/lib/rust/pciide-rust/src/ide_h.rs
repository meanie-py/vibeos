//! Hand-written translation of the `ide.h` / pciidex framework surface used by
//! pciide.c (not in the ntkrnl-sys bindgen). Layouts must match `sdk/include/
//! ddk/ide.h` exactly — these structs are shared with pciidex.sys across the
//! IoAllocate/registration boundary.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use ntkrnl_sys::base::*;

pub const MAX_IDE_CHANNEL: usize = 2;
pub const MAX_IDE_DEVICE: usize = 2;
pub const MAX_IDE_LINE: usize = 2;
// MAX_IDE_DEVICE * MAX_IDE_LINE
const N_LINES: usize = MAX_IDE_DEVICE * MAX_IDE_LINE;

// typedef enum { ChannelDisabled=0, ChannelEnabled, ChannelStateUnknown }
//     IDE_CHANNEL_STATE;  (C enum == int)
pub type IDE_CHANNEL_STATE = u32;
pub const ChannelDisabled: IDE_CHANNEL_STATE = 0;
pub const ChannelEnabled: IDE_CHANNEL_STATE = 1;
pub const ChannelStateUnknown: IDE_CHANNEL_STATE = 2;

// IDENTIFY_DATA is the 512-byte (256-word) ATA IDENTIFY block. pciide never
// touches it (it only writes DevicePresent), so it is kept opaque/size-correct.
pub type IDENTIFY_DATA = [u16; 256];

// PCI_COMMON_HDR_LENGTH = FIELD_OFFSET(PCI_COMMON_CONFIG, DeviceSpecific) = 0x40.
pub const PCI_COMMON_HDR_LENGTH: u32 = 0x40;
// PCI_MULTIFUNCTION bit masked off by PCI_CONFIGURATION_TYPE().
pub const PCI_MULTIFUNCTION: u8 = 0x80;

// PCI class/type constants (the bindgen ones live in the hal_bindgen module).
pub const PCI_DEVICE_TYPE: u32 = 0x0;
pub const PCI_CLASS_MASS_STORAGE_CTLR: u32 = 0x01;
pub const PCI_SUBCLASS_MSC_IDE_CTLR: u32 = 0x01;

// Standard PCI configuration-space header. pciide only reads SubClass (0x0A),
// BaseClass (0x0B), HeaderType (0x0E); PciIdeXGetBusData fills the first
// PCI_COMMON_HDR_LENGTH (0x40) bytes, so the struct is padded to that size. The
// ntkrnl-sys bindgen `_PCI_COMMON_CONFIG` exposes only an opaque blob, hence
// this faithful header here.
#[repr(C)]
pub struct PCI_COMMON_CONFIG {
    pub VendorID: u16,        // 0x00
    pub DeviceID: u16,        // 0x02
    pub Command: u16,         // 0x04
    pub Status: u16,          // 0x06
    pub RevisionID: u8,       // 0x08
    pub ProgIf: u8,           // 0x09
    pub SubClass: u8,         // 0x0A
    pub BaseClass: u8,        // 0x0B
    pub CacheLineSize: u8,    // 0x0C
    pub LatencyTimer: u8,     // 0x0D
    pub HeaderType: u8,       // 0x0E
    pub BIST: u8,             // 0x0F
    pub Rest: [u8; 0x40 - 0x10], // type-specific + DeviceSpecific start (to 0x40)
}

// Transfer-mode bitmasks (ide.h) — SupportedTransferMode /
// DeviceTransferModeSupported.
pub const PIO_MODE0: ULONG = 1 << 0;
pub const PIO_MODE1: ULONG = 1 << 1;
pub const PIO_MODE2: ULONG = 1 << 2;
pub const PIO_MODE3: ULONG = 1 << 3;
pub const PIO_MODE4: ULONG = 1 << 4;
pub const SWDMA_MODE0: ULONG = 1 << 5;
pub const SWDMA_MODE1: ULONG = 1 << 6;
pub const SWDMA_MODE2: ULONG = 1 << 7;
pub const MWDMA_MODE0: ULONG = 1 << 8;
pub const MWDMA_MODE1: ULONG = 1 << 9;
pub const MWDMA_MODE2: ULONG = 1 << 10;
pub const UDMA_MODE0: ULONG = 1 << 11;
pub const UDMA_MODE1: ULONG = 1 << 12;
pub const UDMA_MODE2: ULONG = 1 << 13;
pub const UDMA_MODE3: ULONG = 1 << 14;
pub const UDMA_MODE4: ULONG = 1 << 15;
pub const UDMA_MODE5: ULONG = 1 << 16;
pub const UDMA_MODE6: ULONG = 1 << 17;

// pci.h: PCI_COMMON_HEADER.Command lives at offset 4; bit 2 enables bus mastering.
pub const PCI_COMMAND_OFFSET: ULONG = 4;
pub const PCI_ENABLE_BUS_MASTER: u16 = 0x0004;

// typedef struct _PCIIDE_TRANSFER_MODE_SELECT { ... }
#[repr(C)]
pub struct PCIIDE_TRANSFER_MODE_SELECT {
    pub Channel: ULONG,
    pub DevicePresent: [BOOLEAN; N_LINES],
    pub FixedDisk: [BOOLEAN; N_LINES],
    pub IoReadySupported: [BOOLEAN; N_LINES],
    pub DeviceTransferModeSupported: [ULONG; N_LINES],
    pub BestPioCycleTime: [ULONG; N_LINES],
    pub BestSwDmaCycleTime: [ULONG; N_LINES],
    pub BestMwDmaCycleTime: [ULONG; N_LINES],
    pub BestUDmaCycleTime: [ULONG; N_LINES],
    pub DeviceTransferModeCurrent: [ULONG; N_LINES],
    pub UserChoiceTransferMode: [ULONG; N_LINES],
    pub EnableUDMA66: ULONG,
    pub IdentifyData: [IDENTIFY_DATA; MAX_IDE_DEVICE],
    pub DeviceTransferModeSelected: [ULONG; N_LINES],
    pub TransferModeTimingTable: *mut ULONG,
    pub TransferModeTableLength: ULONG,
}
pub type PPCIIDE_TRANSFER_MODE_SELECT = *mut PCIIDE_TRANSFER_MODE_SELECT;

// Controller-properties callback function pointers (NTAPI == extern "system").
pub type PCIIDE_CHANNEL_ENABLED =
    Option<unsafe extern "system" fn(DeviceExtension: PVOID, Channel: ULONG) -> IDE_CHANNEL_STATE>;
pub type PCIIDE_SYNC_ACCESS_REQUIRED =
    Option<unsafe extern "system" fn(DeviceExtension: PVOID) -> BOOLEAN>;
pub type PCIIDE_TRANSFER_MODE_SELECT_FUNC = Option<
    unsafe extern "system" fn(DeviceExtension: PVOID, XferMode: PPCIIDE_TRANSFER_MODE_SELECT) -> NTSTATUS,
>;
pub type PCIIDE_USEDMA_FUNC = Option<
    unsafe extern "system" fn(DeviceExtension: PVOID, CdbCommand: PUCHAR, Slave: PUCHAR) -> ULONG,
>;
pub type PCIIDE_UDMA_MODES_SUPPORTED = Option<
    unsafe extern "system" fn(
        IdentifyData: IDENTIFY_DATA,
        BestXferMode: PULONG,
        CurrentXferMode: PULONG,
    ) -> NTSTATUS,
>;

// typedef struct _IDE_CONTROLLER_PROPERTIES { ... }
#[repr(C)]
pub struct IDE_CONTROLLER_PROPERTIES {
    pub Size: ULONG,
    pub ExtensionSize: ULONG,
    pub SupportedTransferMode: [[ULONG; MAX_IDE_DEVICE]; MAX_IDE_CHANNEL],
    pub PciIdeChannelEnabled: PCIIDE_CHANNEL_ENABLED,
    pub PciIdeSyncAccessRequired: PCIIDE_SYNC_ACCESS_REQUIRED,
    pub PciIdeTransferModeSelect: PCIIDE_TRANSFER_MODE_SELECT_FUNC,
    pub IgnoreActiveBitForAtaDevice: BOOLEAN,
    pub AlwaysClearBusMasterInterrupt: BOOLEAN,
    pub PciIdeUseDma: PCIIDE_USEDMA_FUNC,
    pub AlignmentRequirement: ULONG,
    pub DefaultPIO: ULONG,
    pub PciIdeUdmaModesSupported: PCIIDE_UDMA_MODES_SUPPORTED,
}
pub type PIDE_CONTROLLER_PROPERTIES = *mut IDE_CONTROLLER_PROPERTIES;

// HwGetControllerProperties callback passed to PciIdeXInitialize.
pub type PCONTROLLER_PROPERTIES = Option<
    unsafe extern "system" fn(
        DeviceExtension: PVOID,
        ControllerProperties: PIDE_CONTROLLER_PROPERTIES,
    ) -> NTSTATUS,
>;

// pciidex.sys exports (NTAPI). Imported via the pciidex spec import lib.
extern "system" {
    pub fn PciIdeXInitialize(
        DriverObject: ntkrnl_sys::ddk::PDRIVER_OBJECT,
        RegistryPath: ntkrnl_sys::ddk::PUNICODE_STRING,
        HwGetControllerProperties: PCONTROLLER_PROPERTIES,
        ExtensionSize: ULONG,
    ) -> NTSTATUS;
    pub fn PciIdeXGetBusData(
        DeviceExtension: PVOID,
        Buffer: PVOID,
        ConfigDataOffset: ULONG,
        BufferLength: ULONG,
    ) -> NTSTATUS;
}
