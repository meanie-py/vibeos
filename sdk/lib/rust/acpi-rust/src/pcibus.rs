//! GUID_PCI_BUS_INTERFACE_STANDARD for a PCI root bridge (VibeOS; no C original).
//!
//! The NT PCI bus driver (pcix.sys) gets config-space access for a root bus from the
//! device below it: it sends IRP_MN_QUERY_INTERFACE for this GUID down the root's stack
//! and, only if nobody answers, falls back to a HAL bus handler. On Windows acpi.sys
//! answers for every PNP0A03/PNP0A08. ReactOS's acpi.sys never did, and an ACPI HAL has
//! no PCI bus handler either, so pcix's root AddDevice failed and it unloaded. Only the
//! legacy HAL's own root PDO (hal/halx86/legacy/halpnpdd.c) provides the interface.
//!
//! ReadConfig/WriteConfig carry the absolute bus number (pcix passes the FDO's BaseBus),
//! so they map directly onto the HAL's PCIConfiguration bus data, which already knows
//! the platform's access mechanism (type-1 ports or MCFG ECAM). RootBusCapability (_OSC)
//! is left NULL; pcix only calls it when present.
#![allow(non_snake_case)]

use core::ffi::c_void;
use ntkrnl_sys::ddk::{
    HalGetBusDataByOffset, HalSetBusDataByOffset, _BUS_DATA_TYPE_PCIConfiguration as PCIConfiguration,
    GUID, NTSTATUS, PCI_BUS_INTERFACE_STANDARD, PVOID, ULONG, USHORT,
};

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC000_0023u32 as NTSTATUS;

extern "C" {
    pub static GUID_PCI_BUS_INTERFACE_STANDARD: GUID;
    fn wcsstr(s1: *const u16, s2: *const u16) -> *mut u16;
}

/// Is this PDO a PCI root bridge? The same test buspdo.rs uses for its resources.
pub unsafe fn is_pci_root(hardware_ids: *const u16) -> bool {
    !hardware_ids.is_null()
        && (!wcsstr(hardware_ids, ntkrnl_sys::w!("PNP0A03").as_ptr()).is_null()
            || !wcsstr(hardware_ids, ntkrnl_sys::w!("PNP0A08").as_ptr()).is_null())
}

unsafe extern "system" fn interface_reference(_context: PVOID) {}
unsafe extern "system" fn interface_dereference(_context: PVOID) {}

unsafe extern "system" fn read_config(
    _context: PVOID,
    bus: ULONG,
    slot: ULONG,
    buffer: PVOID,
    offset: ULONG,
    length: ULONG,
) -> ULONG {
    HalGetBusDataByOffset(PCIConfiguration, bus & 0xFF, slot, buffer, offset, length)
}

unsafe extern "system" fn write_config(
    _context: PVOID,
    bus: ULONG,
    slot: ULONG,
    buffer: PVOID,
    offset: ULONG,
    length: ULONG,
) -> ULONG {
    HalSetBusDataByOffset(PCIConfiguration, bus & 0xFF, slot, buffer, offset, length)
}

/// Fill the caller's PCI_BUS_INTERFACE_STANDARD (the IRP's QueryInterface buffer).
pub unsafe fn query_pci_bus_interface(context: PVOID, interface: *mut c_void, size: USHORT) -> NTSTATUS {
    if (size as usize) < core::mem::size_of::<PCI_BUS_INTERFACE_STANDARD>() {
        return STATUS_BUFFER_TOO_SMALL;
    }
    let out = interface as *mut PCI_BUS_INTERFACE_STANDARD;
    core::ptr::write_bytes(out, 0, 1);
    (*out).Size = core::mem::size_of::<PCI_BUS_INTERFACE_STANDARD>() as USHORT;
    (*out).Version = 1;
    (*out).Context = context;
    (*out).InterfaceReference = Some(interface_reference);
    (*out).InterfaceDereference = Some(interface_dereference);
    (*out).ReadConfig = Some(read_config);
    (*out).WriteConfig = Some(write_config);
    STATUS_SUCCESS
}
