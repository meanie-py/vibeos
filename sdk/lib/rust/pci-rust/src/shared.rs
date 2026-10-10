//! Driver-private types + module globals for pci.sys, translated from
//! `drivers/bus/pci/pci.h`.
//!
//! These structs are driver-PRIVATE (defined in pci.h, not the DDK). Because the
//! whole driver is Rust now, there is no C cohabitant reading them and thus no
//! external layout constraint — but we mirror the C layout exactly anyway (every
//! kernel field type resolved to the single `ntkrnl_sys::ddk` identity), so the
//! `IoCreateDevice(sizeof(FDO_DEVICE_EXTENSION))` allocations and the embedded
//! `PCI_COMMON_CONFIG` that `HalGetBusData` fills stay byte-compatible with the
//! original C driver.
//!
//! The three module-global variables (`DriverExtension`, `HasDebuggingDevice`,
//! `PciDebuggingDevice`) are `extern` in pci.h and shared across all three .c
//! files; here they are `pub static mut` in this one module, referenced as
//! `crate::shared::*` from pci/fdo/pdo.
#![allow(non_camel_case_types, non_snake_case, dead_code)]

pub use ntkrnl_sys::ddk::{
    BOOLEAN, KSPIN_LOCK, LIST_ENTRY, LONG, PDEVICE_OBJECT, ULONG, UCHAR, UNICODE_STRING,
    DEVICE_POWER_STATE, PCI_COMMON_HEADER, PCI_SLOT_NUMBER,
};

// ---------------------------------------------------------------------------
// PCI_COMMON_CONFIG — corrected 256-byte layout.
//
// SUBSTRATE BUG: `ntkrnl_sys::ddk::PCI_COMMON_CONFIG` is mis-generated — bindgen
// could not expand the C `PCI_COMMON_HEADER_LAYOUT` macro that splices the 64-byte
// header inline (sdk/include/xdk/iotypes.h:3585), so it emitted only
// `{ DeviceSpecific: [u8; 192] }` (192 bytes, NO header fields), with a self-
// consistent-but-wrong `size_of == 192` assertion. The real C type is
// PCI_COMMON_HEADER (64) + DeviceSpecific[192] = 256 bytes, and the header fields
// (VendorID, BaseClass, u.type0/type1, ...) are accessed flat. We define the
// correct layout here (Header first, so a `*const PCI_COMMON_HEADER` overlay on
// `&PciConfig` reads the header fields exactly as the C flat-field access does)
// and use THIS type everywhere, never the broken ddk one.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _PCI_COMMON_CONFIG {
    pub Header: PCI_COMMON_HEADER,
    pub DeviceSpecific: [UCHAR; 192],
}
pub type PCI_COMMON_CONFIG = _PCI_COMMON_CONFIG;
pub type PPCI_COMMON_CONFIG = *mut _PCI_COMMON_CONFIG;

pub const TAG_PCI: ULONG = u32::from_ne_bytes([b'P', b'C', b'I', b'0']); // 'PCI0' = '0ICP' little-endian

// ---------------------------------------------------------------------------
// PCI_DEVICE — one discovered device on a PCI bus (pci.h).
// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _PCI_DEVICE {
    pub ListEntry: LIST_ENTRY,
    pub Pdo: PDEVICE_OBJECT,
    pub BusNumber: ULONG,
    pub SlotNumber: PCI_SLOT_NUMBER,
    pub PciConfig: PCI_COMMON_CONFIG,
    pub EnableMemorySpace: BOOLEAN,
    pub EnableIoSpace: BOOLEAN,
    pub EnableBusMaster: BOOLEAN,
    pub IsDebuggingDevice: BOOLEAN,
    /// VibeOS: the ACPI routing of this function's INTx (irqroute.rs).
    pub IrqRouteState: UCHAR,
    pub IrqRouteGsi: ULONG,
}
pub type PCI_DEVICE = _PCI_DEVICE;
pub type PPCI_DEVICE = *mut _PCI_DEVICE;

// PCI_DEVICE_STATE — C enum, driver-private (pci.h).
pub type PCI_DEVICE_STATE = i32;
pub const dsStopped: PCI_DEVICE_STATE = 0;
pub const dsStarted: PCI_DEVICE_STATE = 1;
pub const dsPaused: PCI_DEVICE_STATE = 2;
pub const dsRemoved: PCI_DEVICE_STATE = 3;
pub const dsSurpriseRemoved: PCI_DEVICE_STATE = 4;

// ---------------------------------------------------------------------------
// Device extensions (pci.h).
// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _COMMON_DEVICE_EXTENSION {
    pub DeviceObject: PDEVICE_OBJECT,
    pub IsFDO: BOOLEAN,
    pub Removed: BOOLEAN,
    pub DevicePowerState: DEVICE_POWER_STATE,
}
pub type COMMON_DEVICE_EXTENSION = _COMMON_DEVICE_EXTENSION;
pub type PCOMMON_DEVICE_EXTENSION = *mut _COMMON_DEVICE_EXTENSION;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _PDO_DEVICE_EXTENSION {
    pub Common: COMMON_DEVICE_EXTENSION,
    pub Fdo: PDEVICE_OBJECT,
    pub PciDevice: PPCI_DEVICE,
    pub DeviceID: UNICODE_STRING,
    pub InstanceID: UNICODE_STRING,
    pub HardwareIDs: UNICODE_STRING,
    pub CompatibleIDs: UNICODE_STRING,
    pub DeviceDescription: UNICODE_STRING,
    pub DeviceLocation: UNICODE_STRING,
    pub References: LONG,
}
pub type PDO_DEVICE_EXTENSION = _PDO_DEVICE_EXTENSION;
pub type PPDO_DEVICE_EXTENSION = *mut _PDO_DEVICE_EXTENSION;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _FDO_DEVICE_EXTENSION {
    pub Common: COMMON_DEVICE_EXTENSION,
    pub ListEntry: LIST_ENTRY,
    pub BusNumber: ULONG,
    pub State: PCI_DEVICE_STATE,
    pub DeviceListHead: LIST_ENTRY,
    pub DeviceListCount: ULONG,
    pub DeviceListLock: KSPIN_LOCK,
    pub Ldo: PDEVICE_OBJECT,
    /// VibeOS: acpi.sys's INTx routing, on a root bus (irqroute.rs).
    pub IrqRouting: crate::irqroute::IrqRoutingInterface,
}
pub type FDO_DEVICE_EXTENSION = _FDO_DEVICE_EXTENSION;
pub type PFDO_DEVICE_EXTENSION = *mut _FDO_DEVICE_EXTENSION;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _PCI_DRIVER_EXTENSION {
    pub BusListHead: LIST_ENTRY,
    pub BusListLock: KSPIN_LOCK,
}
pub type PCI_DRIVER_EXTENSION = _PCI_DRIVER_EXTENSION;
pub type PPCI_DRIVER_EXTENSION = *mut _PCI_DRIVER_EXTENSION;

// ---------------------------------------------------------------------------
// PCI_TYPE1_CFG_CYCLE_BITS — union { bitfields; ULONG AsULONG } (pci.h).
// Bit layout (LSB first): InUse:2, RegisterNumber:6, FunctionNumber:3,
// DeviceNumber:5, BusNumber:8, Reserved2:8. Modeled as a u32 with bitfield
// accessor methods (the field-style C writes become set_*; reads become *()).
// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PCI_TYPE1_CFG_CYCLE_BITS {
    pub AsULONG: ULONG,
}
pub type PPCI_TYPE1_CFG_CYCLE_BITS = *mut PCI_TYPE1_CFG_CYCLE_BITS;

impl PCI_TYPE1_CFG_CYCLE_BITS {
    #[inline]
    pub const fn new() -> Self { Self { AsULONG: 0 } }

    #[inline]
    fn get(&self, shift: u32, width: u32) -> ULONG {
        (self.AsULONG >> shift) & ((1u32 << width) - 1)
    }
    #[inline]
    fn set(&mut self, shift: u32, width: u32, value: ULONG) {
        let mask = ((1u32 << width) - 1) << shift;
        self.AsULONG = (self.AsULONG & !mask) | ((value << shift) & mask);
    }

    #[inline] pub fn InUse(&self) -> ULONG { self.get(0, 2) }
    #[inline] pub fn set_InUse(&mut self, v: ULONG) { self.set(0, 2, v) }
    #[inline] pub fn RegisterNumber(&self) -> ULONG { self.get(2, 6) }
    #[inline] pub fn set_RegisterNumber(&mut self, v: ULONG) { self.set(2, 6, v) }
    #[inline] pub fn FunctionNumber(&self) -> ULONG { self.get(8, 3) }
    #[inline] pub fn set_FunctionNumber(&mut self, v: ULONG) { self.set(8, 3, v) }
    #[inline] pub fn DeviceNumber(&self) -> ULONG { self.get(11, 5) }
    #[inline] pub fn set_DeviceNumber(&mut self, v: ULONG) { self.set(11, 5, v) }
    #[inline] pub fn BusNumber(&self) -> ULONG { self.get(16, 8) }
    #[inline] pub fn set_BusNumber(&mut self, v: ULONG) { self.set(16, 8, v) }
    #[inline] pub fn Reserved2(&self) -> ULONG { self.get(24, 8) }
    #[inline] pub fn set_Reserved2(&mut self, v: ULONG) { self.set(24, 8, v) }
}

// ---------------------------------------------------------------------------
// Module globals (pci.h `extern` — defined in pci.c, used by fdo.c/pdo.c).
// ---------------------------------------------------------------------------
pub static mut DriverExtension: PPCI_DRIVER_EXTENSION = core::ptr::null_mut();
pub static mut HasDebuggingDevice: BOOLEAN = 0; // FALSE
pub static mut PciDebuggingDevice: [PCI_TYPE1_CFG_CYCLE_BITS; 2] =
    [PCI_TYPE1_CFG_CYCLE_BITS::new(); 2];
