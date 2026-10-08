//! PCI configuration access for virtio-drivers, over the HAL bus-data exports.
//!
//! `HalGetBusDataByOffset`/`HalSetBusDataByOffset` with `BusDataType =
//! PCIConfiguration` is the portable kernel API for PCI config space — it hides
//! the 0xCF8/0xCFC (i440fx) vs ECAM mechanism, so this same code works on every
//! arch without per-arch port I/O. We implement virtio-drivers'
//! `ConfigurationAccess` on top of it.

use virtio_drivers::transport::pci::bus::{ConfigurationAccess, DeviceFunction};

use ntkrnl_sys::ddk::{HalGetBusDataByOffset, HalSetBusDataByOffset, PVOID};

/// _BUS_DATA_TYPE_PCIConfiguration
const PCI_CONFIGURATION: i32 = 4;

const VIRTIO_VENDOR_ID: u16 = 0x1AF4;
/// Modern virtio-net (virtio 1.x). We force `disable-legacy=on` in the QEMU
/// config, so this is what enumerates; 0x1000 (transitional) accepted as a
/// fallback in case the harness ever runs without that flag.
const VIRTIO_NET_MODERN: u16 = 0x1041;
const VIRTIO_NET_TRANSITIONAL: u16 = 0x1000;

/// PCI_SLOT_NUMBER: device in bits 0..4, function in bits 5..7.
#[inline]
fn slot(df: DeviceFunction) -> u32 {
    (df.device as u32) | ((df.function as u32) << 5)
}

#[derive(Clone)]
pub struct HalPciAccess;

impl ConfigurationAccess for HalPciAccess {
    fn read_word(&self, df: DeviceFunction, register_offset: u8) -> u32 {
        let mut val: u32 = 0xFFFF_FFFF;
        unsafe {
            HalGetBusDataByOffset(
                PCI_CONFIGURATION,
                df.bus as u32,
                slot(df),
                &mut val as *mut u32 as PVOID,
                register_offset as u32,
                4,
            );
        }
        val
    }

    fn write_word(&mut self, df: DeviceFunction, register_offset: u8, data: u32) {
        let mut v = data;
        unsafe {
            HalSetBusDataByOffset(
                PCI_CONFIGURATION,
                df.bus as u32,
                slot(df),
                &mut v as *mut u32 as PVOID,
                register_offset as u32,
                4,
            );
        }
    }

    unsafe fn unsafe_clone(&self) -> Self {
        HalPciAccess
    }
}

/// Scan PCI bus 0 (QEMU i440fx keeps everything on bus 0) for the virtio-net
/// device function.
pub fn find_virtio_net() -> Option<DeviceFunction> {
    let access = HalPciAccess;
    for device in 0..32u8 {
        for function in 0..8u8 {
            let df = DeviceFunction { bus: 0, device, function };
            let id = access.read_word(df, 0x00);
            let vendor = (id & 0xFFFF) as u16;
            let dev = (id >> 16) as u16;
            if vendor == VIRTIO_VENDOR_ID
                && (dev == VIRTIO_NET_MODERN || dev == VIRTIO_NET_TRANSITIONAL)
            {
                return Some(df);
            }
        }
    }
    None
}

/// Set MEMORY_SPACE | BUS_MASTER in the PCI command register. virtio-drivers'
/// `PciTransport::new` does NOT do this, and the device cannot DMA without bus
/// mastering enabled. INTx stays DISABLED (bit 10) until a driver that services
/// it asks for it (`unmask_intx`, x86's M10): the line is shared (QEMU's arm64
/// board routes INTx by slot, so virtio-net in slot 1 shares INTID 36 with slots
/// 5, 9, 13, ...), and a virtio device raises it on its own, for a config change
/// such as link-up, until someone reads its ISR register. Unserviced, that was a
/// level-triggered interrupt nobody claimed, forever: the arm64 boot wedged the
/// day nvme.sys connected INTID 36 (2026-10-06).
pub fn enable_bus_master(df: DeviceFunction) {
    let mut access = HalPciAccess;
    let cmd_status = access.read_word(df, 0x04);
    access.write_word(df, 0x04, cmd_status | 0x0006 | 0x0400);
}

/// Clear INTERRUPT_DISABLE (command bit 10) so the function may assert legacy
/// INTx, for a driver about to connect an ISR to it.
pub fn unmask_intx(df: DeviceFunction) {
    let mut access = HalPciAccess;
    let cmd_status = access.read_word(df, 0x04);
    access.write_word(df, 0x04, cmd_status & !0x0400);
}


/// Read the PCI Interrupt Line (config byte 0x3C) — the IRQ the firmware routed
/// this function's INTx pin to. 0xFF means "unrouted" → None. M10 (amd64) feeds
/// this to `HalGetInterruptVector` to connect the RX interrupt.
pub fn read_interrupt_line(df: DeviceFunction) -> Option<u8> {
    let line = (HalPciAccess.read_word(df, 0x3C) & 0xFF) as u8;
    if line == 0xFF {
        None
    } else {
        Some(line)
    }
}
