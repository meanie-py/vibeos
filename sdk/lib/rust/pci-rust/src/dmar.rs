//! DMA remapping: which devices may have an SMMU domain of their own, and DMA
//! adapters that know whose they are.
//!
//! Windows gives a device a DMA-remapping domain only when its driver says it is
//! compatible: `DmaRemappingCompatible` = 1 under its service's Parameters key, which
//! promises that every address it hands the device came from its DMA adapter. A
//! driver that hands out MmGetPhysicalAddress results would fault under one. The HAL
//! is told at START (HalVibeosDmaAttach), and each adapter carries the device's
//! requester ID so it can map what it hands out into that domain.

use core::mem::size_of;

use ntkrnl_sys::ddk::*;

use crate::msix::{query_dword, utf16, KEY_READ};
use crate::shared::PPDO_DEVICE_EXTENSION;

unsafe extern "system" {
    fn HalVibeosGetDmaAdapter(
        RequesterId: ULONG,
        DeviceDescription: PDEVICE_DESCRIPTION,
        NumberOfMapRegisters: *mut ULONG,
    ) -> PDMA_ADAPTER;
}

/// The device's requester ID with its PCI segment above it, as the HAL takes it.
pub unsafe fn requester(ext: PPDO_DEVICE_EXTENSION) -> u32 {
    let dev = (*ext).PciDevice;
    let slot = (*dev).SlotNumber.u.AsULONG;
    ((*dev).BusNumber & 0xFF_FFFF) << 8 | (slot & 0x1F) << 3 | (slot >> 5) & 7
}

/// Whether the driver of `pdo`'s device stack says it is DMA-remapping compatible:
/// the first driver above the PDO whose service key has a Parameters
/// `DmaRemappingCompatible` decides (filters normally have none).
pub unsafe fn remapping_compatible(pdo: PDEVICE_OBJECT) -> bool {
    const PREFIX: &[u16] = utf16!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\");
    const SUFFIX: &[u16] = utf16!("\\Parameters");
    let mut device = (*pdo).AttachedDevice;
    while !device.is_null() {
        let driver = (*device).DriverObject;
        let service = if driver.is_null() || (*driver).DriverExtension.is_null() {
            None
        } else {
            Some((*(*driver).DriverExtension).ServiceKeyName)
        };
        if let Some(name) = service.filter(|n| !n.Buffer.is_null() && n.Length != 0) {
            let mut path = [0u16; 256];
            let chars = name.Length as usize / 2;
            let total = PREFIX.len() + chars + SUFFIX.len();
            if total <= path.len() {
                path[..PREFIX.len()].copy_from_slice(PREFIX);
                path[PREFIX.len()..PREFIX.len() + chars].copy_from_slice(core::slice::from_raw_parts(name.Buffer, chars));
                path[PREFIX.len() + chars..total].copy_from_slice(SUFFIX);
                let mut key_name: UNICODE_STRING = core::mem::zeroed();
                key_name.Buffer = path.as_mut_ptr();
                key_name.Length = (total * 2) as u16;
                key_name.MaximumLength = key_name.Length;
                let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
                oa.Length = size_of::<OBJECT_ATTRIBUTES>() as ULONG;
                oa.ObjectName = &mut key_name;
                oa.Attributes = 0x240; // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
                let mut key: HANDLE = core::ptr::null_mut();
                if ZwOpenKey(&mut key, KEY_READ, &mut oa) >= 0 {
                    let value = query_dword(key, utf16!("DmaRemappingCompatible"));
                    ZwClose(key);
                    if let Some(v) = value {
                        return v == 1;
                    }
                }
            }
        }
        device = (*device).AttachedDevice;
    }
    false
}

/// BUS_INTERFACE_STANDARD.GetDmaAdapter: the HAL's adapter, told which device it is
/// for. `Context` is the PDO.
pub unsafe fn adapter(pdo: PDEVICE_OBJECT, description: PDEVICE_DESCRIPTION, map_registers: *mut ULONG) -> PDMA_ADAPTER {
    let ext = (*pdo).DeviceExtension as PPDO_DEVICE_EXTENSION;
    HalVibeosGetDmaAdapter(requester(ext), description, map_registers)
}
