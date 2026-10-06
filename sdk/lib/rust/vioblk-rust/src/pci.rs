//! PCI configuration access for virtio-drivers over the HAL bus-data exports — the same
//! portable mechanism the netstack uses (netstack-rust/src/pci.rs), so it works whatever
//! the chipset's config mechanism is.

use virtio_drivers::transport::pci::bus::{ConfigurationAccess, DeviceFunction};

use ntkrnl_sys::ddk::{HalGetBusDataByOffset, HalSetBusDataByOffset, PVOID};

/// `_BUS_DATA_TYPE_PCIConfiguration`.
const PCI_CONFIGURATION: i32 = 4;

/// PCI_SLOT_NUMBER: device in bits 0..4, function in bits 5..7.
#[inline]
fn slot(df: DeviceFunction) -> u32 {
    (df.device as u32) | ((df.function as u32) << 5)
}

/// The inverse, for the SlotNumber scsiport hands HwFindAdapter.
pub fn device_function(bus: u32, slot_number: u32) -> DeviceFunction {
    DeviceFunction {
        bus: bus as u8,
        device: (slot_number & 0x1F) as u8,
        function: ((slot_number >> 5) & 0x7) as u8,
    }
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

/// Enable memory decoding and bus mastering, and ask for legacy INTx off (pci.sys turns
/// it back on when PnP starts the device; HwInterrupt acknowledges what gets through).
///
/// virtio-drivers' `PciTransport::new` does not enable bus mastering, and the device
/// cannot DMA without it. INTx goes off because this driver polls: a virtio-blk
/// function shares its INTx line with whatever else the chipset routed there, and an
/// interrupt nobody claims on a level-triggered shared line is a storm.
pub fn enable_polled_dma(df: DeviceFunction) {
    let mut access = HalPciAccess;
    let cmd_status = access.read_word(df, 0x04);
    // bit1 MEMORY_SPACE, bit2 BUS_MASTER, bit10 INTERRUPT_DISABLE. The status half of
    // the dword is write-1-to-clear, so write zeros there.
    access.write_word(df, 0x04, (cmd_status & 0xFFFF) | 0x0406);
}
