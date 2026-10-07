//! glue.rs — #define constants + enum-variant aliases + driver globals the pciidex C
//! source/LLM reference by bare name. REAL values (clang-computed / from the bindgen
//! substrate), never LLM-guessed.
#![allow(non_upper_case_globals, dead_code, non_camel_case_types)]
use ntkrnl_sys::ddk::ULONG;

// pciidex.h: #define TAG_PCIIDEX 'XedI' → ('X'<<24)|('e'<<16)|('d'<<8)|'I' = 0x58656449.
pub const TAG_PCIIDEX: u32 = 0x58656449;

// miniport.c global: ULONG PciIdeDebug = 0; (debug print threshold).
pub static mut PciIdeDebug: ULONG = 0;

// LVL — the line-number arg type in the PAGED_CODE/RtlAssert expansion (the LLM cast
// `NNN as LVL`); it is ULONG.
pub type LVL = ULONG;
pub type size_t = usize;

// DDK / PCI / NTSTATUS / IRP_MN constants — clang-computed (bindgen --allowlist-var over
// ntddk.h + ide.h), NOT hand-derived.
pub const CM_RESOURCE_INTERRUPT_LATCHED: u32 = 1;
pub const CM_RESOURCE_PORT_16_BIT_DECODE: u32 = 16;
pub const CM_RESOURCE_PORT_IO: u32 = 1;
pub const FILE_DEVICE_CONTROLLER: u32 = 4;
pub const FILE_DEVICE_BUS_EXTENDER: u32 = 0x2a;
pub const FILE_DEVICE_SECURE_OPEN: u32 = 0x100;
pub const PCI_CLASS_MASS_STORAGE_CTLR: u32 = 1;
pub const PCI_SUBCLASS_MSC_IDE_CTLR: u32 = 1;
pub const PCI_SUBCLASS_MSC_RAID_CTLR: u32 = 4;
pub const PCI_WHICHSPACE_CONFIG: u32 = 0;
pub const PNP_DEVICE_NOT_DISABLEABLE: u32 = 32;
pub const STATUS_DEVICE_BUSY: i32 = 0x80000011u32 as i32;             // -2147483631
pub const STATUS_DEVICE_CONFIGURATION_ERROR: i32 = 0xC0000182u32 as i32;
pub const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32;
pub const STATUS_MORE_PROCESSING_REQUIRED: i32 = 0xC0000016u32 as i32;
pub const IRP_MN_QUERY_INTERFACE: u8 = 8;
// NOT in driver_ddk — without these the dispatch match arms silently become catch-all
// bindings (match-on-undefined-const hazard). Values from sdk/include wdm.h.
pub const IRP_MN_QUERY_PNP_DEVICE_STATE: u8 = 0x14;
pub const IRP_MN_DEVICE_USAGE_NOTIFICATION: u8 = 0x16;
pub const IRP_MN_QUERY_CAPABILITIES: u8 = 0x09;
pub const IRP_MN_QUERY_RESOURCES: u8 = 0x0A;
pub const IRP_MN_QUERY_RESOURCE_REQUIREMENTS: u8 = 0x0B;
pub const IRP_MN_QUERY_DEVICE_TEXT: u8 = 0x0C;
pub const IRP_MN_QUERY_ID: u8 = 0x13;

// CM_RESOURCE_TYPE enum values (the kernel bindgen leaves this enum out) — used vs the
// UCHAR CmResourceType field. Standard ntddk values.
pub const CmResourceTypePort: u8 = 1;
pub const CmResourceTypeInterrupt: u8 = 2;
pub const CmResourceTypeMemory: u8 = 3;

// enum-variant aliases — bindgen prefixes the variant; the C/LLM use the bare name.
pub use crate::types::IDE_CHANNEL_STATE_ChannelDisabled as ChannelDisabled;
pub use ntkrnl_sys::ddk::_DPFLTR_TYPE_DPFLTR_PCIIDE_ID as DPFLTR_PCIIDE_ID;
// FILE_DEVICE_* characteristics the LLM camelCased.
pub const FileDeviceBusExtender: u32 = 0x2a;
pub const FileDeviceSecureOpen: u32 = 0x100;

// additional bindgen-name aliases the LLM referenced.
pub use crate::types::IDE_CHANNEL_STATE_ChannelDisabled as _IDE_CHANNEL_STATE_ChannelDisabled;
pub use ntkrnl_sys::ddk::_CM_PARTIAL_RESOURCE_DESCRIPTOR as __CM_PARTIAL_RESOURCE_DESCRIPTOR;

// PnP/resource enum-variant aliases (bindgen _ENUM_Variant → bare name).
pub use ntkrnl_sys::ddk::{
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation,
    _BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID,
    _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs,
    _BUS_QUERY_ID_TYPE_BusQueryCompatibleIDs as BusQueryCompatibleIDs,
    _BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID,
    _CM_SHARE_DISPOSITION_CmResourceShareDeviceExclusive as CmResourceShareDeviceExclusive,
    _CM_SHARE_DISPOSITION_CmResourceShareShared as CmResourceShareShared,
    _DEVICE_TEXT_TYPE_DeviceTextLocationInformation as DeviceTextLocationInformation,
    _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeDumpFile as DeviceUsageTypeDumpFile,
    _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypePaging as DeviceUsageTypePaging,
    _INTERFACE_TYPE_InterfaceTypeUndefined as InterfaceTypeUndefined,
    _INTERFACE_TYPE_Isa as Isa,
    _INTERFACE_TYPE_PCIBus as PCIBus,
};
