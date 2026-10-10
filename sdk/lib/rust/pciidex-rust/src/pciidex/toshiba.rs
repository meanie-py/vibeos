use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut PiccoloPioTimings: [ULONG; 5] = [
    0x566 as ::core::ffi::c_int as ULONG,
    0x433 as ::core::ffi::c_int as ULONG,
    0x311 as ::core::ffi::c_int as ULONG,
    0x201 as ::core::ffi::c_int as ULONG,
    0x200 as ::core::ffi::c_int as ULONG,
];

static mut PiccoloMwDmaTimings: [ULONG; 3] = [
    0x655 as ::core::ffi::c_int as ULONG,
    0x200 as ::core::ffi::c_int as ULONG,
    0x200 as ::core::ffi::c_int as ULONG,
];

static mut PiccoloUdmaTimings: [ULONG; 3] = [
    0x84000222 as ::core::ffi::c_uint as ULONG,
    0x83000111 as ::core::ffi::c_uint as ULONG,
    0x82000000 as ::core::ffi::c_uint as ULONG,
];

pub const PCI_DEV_PICCOLO_1: u32 = 0x0101;
pub const PCI_DEV_PICCOLO_2: u32 = 0x0102;
pub const PCI_DEV_PICCOLO_3: u32 = 0x0103;
pub const PCI_DEV_PICCOLO_5: u32 = 0x0105;
pub const PICCOLO_REG_PIO_TIMING: u32 = 0x50;
pub const PICCOLO_REG_DMA_TIMING: u32 = 0x5C;
pub const PICCOLO_PIO_MASK: u32 = 0xFFFFE088;
pub const PICCOLO_DMA_MASK: u32 = 0xFFFFE088;
pub const PICCOLO_UDMA_MASK: u32 = 0x78FFE088;

unsafe extern "C" fn ToshibaSetTransferMode(mut Controller: PATA_CONTROLLER, mut Channel: ULONG, mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    let mut i: ULONG = 0;
    UNREFERENCED_PARAMETER!(Channel);
    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let mut TimingReg: ULONG = 0;
        if !Device.is_null() {
            TimingReg = PciRead32(Controller, PICCOLO_REG_PIO_TIMING);
            TimingReg &= PICCOLO_PIO_MASK;
            TimingReg |= PiccoloPioTimings[(*Device).PioMode as usize];
            PciWrite32(Controller, PICCOLO_REG_PIO_TIMING, TimingReg);
            if (*Device).DmaMode != PIO_MODE!(0) {
                TimingReg = PciRead32(Controller, PICCOLO_REG_DMA_TIMING);
                if (*Device).DmaMode >= UDMA_MODE!(0) {
                    TimingReg &= PICCOLO_UDMA_MASK;
                    TimingReg |= PiccoloUdmaTimings[(*Device)
                        .DmaMode
                        .wrapping_sub(UDMA_MODE!(0))
                        as usize];
                } else {
                    TimingReg &= PICCOLO_DMA_MASK;
                    TimingReg |= PiccoloMwDmaTimings[(*Device)
                        .DmaMode
                        .wrapping_sub(MWDMA_MODE!(0))
                        as usize];
                }
                PciWrite32(Controller, PICCOLO_REG_DMA_TIMING, TimingReg);
            }
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ToshibaGetControllerProperties(
    Controller: *mut ATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;

    PAGED_CODE!();
    ASSERT!((*Controller).Pci.VendorID as u32 == PCI_VEN_TOSHIBA);

    if (*Controller).Pci.DeviceID != (PCI_DEV_PICCOLO_1 as u16) &&
        (*Controller).Pci.DeviceID != (PCI_DEV_PICCOLO_2 as u16) &&
        (*Controller).Pci.DeviceID != (PCI_DEV_PICCOLO_3 as u16) &&
        (*Controller).Pci.DeviceID != (PCI_DEV_PICCOLO_5 as u16)
    {
        return STATUS_NO_MATCH;
    }

    (*Controller).MaxChannels = 1;

    Status = PciIdeCreateChannelData(Controller, 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    i = 0;
    while i < (*Controller).MaxChannels {
        let ChanData: *mut CHANNEL_DATA_PATA = (*Controller).Channels[i as usize] as *mut CHANNEL_DATA_PATA;

        (*ChanData).__bindgen_anon_1.TransferModeSupported = PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 2);
        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(ToshibaSetTransferMode);

        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
