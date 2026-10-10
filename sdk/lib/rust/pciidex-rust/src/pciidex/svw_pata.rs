use crate::prelude::*;
use ntkrnl_sys::ddk::{HalGetBusDataByOffset, HalSetBusDataByOffset, PCI_SLOT_NUMBER, PPCI_COMMON_HEADER};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_162 {
    pub Value: UCHAR,
    pub CycleTime: ULONG,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_163 {
    pub Value: UCHAR,
    pub CycleTime: ULONG,
}

static mut SvwPioTimings: [C2RustUnnamed_162; 5] = [
    C2RustUnnamed_162 {
        Value: 0x5d as UCHAR,
        CycleTime: 600 as ULONG,
    },
    C2RustUnnamed_162 {
        Value: 0x47 as UCHAR,
        CycleTime: 390 as ULONG,
    },
    C2RustUnnamed_162 {
        Value: 0x34 as UCHAR,
        CycleTime: 270 as ULONG,
    },
    C2RustUnnamed_162 {
        Value: 0x22 as UCHAR,
        CycleTime: 180 as ULONG,
    },
    C2RustUnnamed_162 {
        Value: 0x20 as UCHAR,
        CycleTime: 120 as ULONG,
    },
];

static mut SvwMwDmaTimings: [C2RustUnnamed_163; 3] = [
    C2RustUnnamed_163 {
        Value: 0x77 as UCHAR,
        CycleTime: 480 as ULONG,
    },
    C2RustUnnamed_163 {
        Value: 0x21 as UCHAR,
        CycleTime: 150 as ULONG,
    },
    C2RustUnnamed_163 {
        Value: 0x20 as UCHAR,
        CycleTime: 120 as ULONG,
    },
];

pub const PCI_DEV_OSB4_IDE: u32 = 0x0211;
pub const PCI_DEV_CSB5_IDE: u32 = 0x0212;
pub const PCI_DEV_CSB6_IDE: u32 = 0x0213;
pub const PCI_DEV_CSB6_IDE_THIRD: u32 = 0x0217;
pub const PCI_DEV_HT1000_IDE: u32 = 0x0214;
pub const PCI_DEV_OSB4_BRIDGE: u32 = 0x0210;
pub const SVW_REG_PIO_TIMING: u32 = 0x40;
pub const SVW_REG_DMA_TIMING: u32 = 0x44;
pub const SVW_REG_PIO_MODE: u32 = 0x4A;
pub const SVW_REG_UDMA_ENABLE: u32 = 0x54;
pub const SVW_REG_UDMA_MODE: u32 = 0x56;
pub const SVW_REG_UDMA_CONTROL: u32 = 0x5A;
pub const SVW_UDMA_CTRL_MODE_MASK: u32 = 0x03;
pub const SVW_UDMA_CTRL_DISABLE: u32 = 0x40;
pub const SVW_UDMA_CTRL_MODE_UDMA4: u32 = 0x02;
pub const SVW_UDMA_CTRL_MODE_UDMA5: u32 = 0x03;

#[repr(C)]
pub struct SvwPioTimingEntry {
    pub Value: u8,
    pub CycleTime: u32,
}

#[repr(C)]
pub struct SvwMwDmaTimingEntry {
    pub Value: u8,
    pub CycleTime: u32,
}

unsafe extern "C" fn SvwChooseDeviceSpeed(
    mut Channel: ULONG,
    mut Device: PCHANNEL_DEVICE_CONFIG,
) {
    let mut Mode: ULONG = 0;

    Mode = (*Device).PioMode;
    while Mode > PIO_MODE!(0) {
        if !((*Device).SupportedModes & (1 << Mode) == 0) {
            if SvwPioTimings[Mode as usize].CycleTime >= (*Device).MinPioCycleTime {
                break;
            }
        }
        Mode = Mode.wrapping_sub(1);
    }
    if Mode != (*Device).PioMode {
        INFO!(
            b"CH %lu: Downgrade PIO speed from %lu to %lu\n\0".as_ptr() as *const i8,
            Channel,
            (*Device).PioMode,
            Mode,
        );
    }
    (*Device).PioMode = Mode;

    if (*Device).DmaMode == PIO_MODE!(0)
        || (*Device).DmaMode >= UDMA_MODE!(0)
    {
        return;
    }

    Mode = (*Device).DmaMode;
    while Mode > PIO_MODE!(0) {
        if !((*Device).SupportedModes
            & !PIO_ALL
            & (1 << Mode)
            == 0)
        {
            ASSERT!((1 << Mode) & MWDMA_ALL);

            if SvwMwDmaTimings[Mode.wrapping_sub(MWDMA_MODE!(0)) as usize]
                .CycleTime
                >= (*Device).MinMwDmaCycleTime
            {
                break;
            }
        }
        Mode = Mode.wrapping_sub(1);
    }
    if Mode != (*Device).DmaMode {
        if Mode == PIO_MODE!(0) {
            WARN!(
                b"CH %lu: Too slow device '%s', disabling DMA\n\0".as_ptr() as *const i8,
                Channel,
                (*Device).FriendlyName,
            );
        } else {
            INFO!(
                b"CH %lu: Downgrade DMA speed from %lu to %lu\n\0".as_ptr() as *const i8,
                Channel,
                (*Device).DmaMode,
                Mode,
            );
        }
    }
    (*Device).DmaMode = Mode;
}
#[no_mangle]
pub unsafe extern "C" fn SvwSetTransferMode(Controller: PATA_CONTROLLER, Channel: ULONG, DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    let mut PioModeReg: USHORT;
    let mut UdmaModeReg: USHORT;
    let mut UdmaEnableReg: UCHAR;
    let mut i: ULONG;
    let mut PioTimReg: ULONG;
    let mut DmaTimReg: ULONG;

    if (*Controller).Pci.DeviceID != (PCI_DEV_OSB4_IDE as u16) {
        PioModeReg = PciRead16(Controller, SVW_REG_PIO_MODE);
    } else {
        PioModeReg = 0;
    }

    PioTimReg = PciRead32(Controller, SVW_REG_PIO_TIMING);
    DmaTimReg = PciRead32(Controller, SVW_REG_DMA_TIMING);
    UdmaEnableReg = PciRead8(Controller, SVW_REG_UDMA_ENABLE);
    UdmaModeReg = PciRead16(Controller, SVW_REG_UDMA_MODE);

    INFO!(
        b"CH %lu: Config (before)\nDMA Timing   %08lX\nPIO Timing   %08lX\nPIO Mode         %04X\nUDMA Mode        %04X\nUDMA Enable        %02X\n\0"
            .as_ptr() as *const i8,
        Channel,
        DmaTimReg,
        PioTimReg,
        PioModeReg,
        UdmaModeReg,
        UdmaEnableReg,
    );

    i = 0;
    while i < MAX_IDE_DEVICE {
        let Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let DeviceIndex: ULONG = (Channel << 1).wrapping_add(i);
        let Shift: ULONG = (DeviceIndex ^ 1) << 3;

        UdmaModeReg = (UdmaModeReg as ULONG
            & !((0x0F as ULONG) << (DeviceIndex << 2)))
            as USHORT;
        UdmaEnableReg = (UdmaEnableReg as ULONG
            & !((0x01 as ULONG) << DeviceIndex))
            as UCHAR;

        if !Device.is_null() {
            SvwChooseDeviceSpeed(Channel, Device);

            if (*Device).DmaMode >= UDMA_MODE!(0) {
                let ModeIndex: ULONG = (*Device).DmaMode.wrapping_sub(UDMA_MODE!(0));

                UdmaModeReg = (UdmaModeReg as ULONG
                    | ModeIndex << (DeviceIndex << 2))
                    as USHORT;
                UdmaEnableReg = (UdmaEnableReg as ULONG
                    | (1 as ULONG) << DeviceIndex)
                    as UCHAR;

                DmaTimReg &= !((0xFF as ULONG) << Shift);
                DmaTimReg |= (SvwMwDmaTimings[2 as usize].Value as ULONG) << Shift;
            } else if (*Device).DmaMode != PIO_MODE!(0) {
                let ModeIndex: ULONG = (*Device).DmaMode.wrapping_sub(MWDMA_MODE!(0));

                DmaTimReg &= !((0xFF as ULONG) << Shift);
                DmaTimReg |= (SvwMwDmaTimings[ModeIndex as usize].Value as ULONG) << Shift;
            }

            PioModeReg = (PioModeReg as ULONG
                & !((0x0F as ULONG) << (DeviceIndex << 2)))
                as USHORT;
            PioModeReg = (PioModeReg as ULONG
                | (*Device).PioMode << (DeviceIndex << 2))
                as USHORT;

            PioTimReg &= !((0xFF as ULONG) << Shift);
            PioTimReg |= (SvwPioTimings[(*Device).PioMode as usize].Value as ULONG) << Shift;
        }

        i = i.wrapping_add(1);
    }

    PciWrite32(Controller, SVW_REG_PIO_TIMING, PioTimReg);

    if (*Controller).Pci.DeviceID != (PCI_DEV_OSB4_IDE as u16) {
        PciWrite16(Controller, SVW_REG_PIO_MODE, PioModeReg);
    }

    PciWrite32(Controller, SVW_REG_DMA_TIMING, DmaTimReg);
    PciWrite16(Controller, SVW_REG_UDMA_MODE, UdmaModeReg);
    PciWrite8(Controller, SVW_REG_UDMA_ENABLE, UdmaEnableReg);

    INFO!(
        b"CH %lu: Config (after)\nDMA Timing   %08lX\nPIO Timing   %08lX\nPIO Mode         %04X\nUDMA Mode        %04X\nUDMA Enable        %02X\n\0"
            .as_ptr() as *const i8,
        Channel,
        DmaTimReg,
        PioTimReg,
        PioModeReg,
        UdmaModeReg,
        UdmaEnableReg,
    );
}
#[no_mangle]
pub unsafe extern "C" fn SvwHasUdmaCable(
    Controller: PATA_CONTROLLER,
    Channel: ULONG,
) -> BOOLEAN {
    let mut UdmaModeReg: UCHAR;

    PAGED_CODE!();

    UdmaModeReg = PciRead8(Controller, SVW_REG_UDMA_MODE + Channel);

    return (((UdmaModeReg & 0x07) > 2)
        || ((UdmaModeReg & 0x70) > 0x20)) as u8;
}

unsafe extern "C" fn SvwOsb4PciBridgeInit(mut Context: PVOID, mut BusNumber: ULONG, mut PciSlot: PCI_SLOT_NUMBER, mut PciConfig: PPCI_COMMON_HEADER) -> BOOLEAN {
    let mut Value: ULONG = 0;
    UNREFERENCED_PARAMETER!(Context);
    if (*PciConfig).VendorID as ::core::ffi::c_int != (PCI_VEN_SERVERWORKS as i32)
        || (*PciConfig).DeviceID as ::core::ffi::c_int != (PCI_DEV_OSB4_BRIDGE as i32)
    {
        return FALSE;
    }
    HalGetBusDataByOffset(
        PCIConfiguration,
        BusNumber,
        PciSlot.u.AsULONG,
        &raw mut Value as PVOID,
        0x64,
        ::core::mem::size_of::<ULONG>() as ULONG,
    );
    Value &= !(0x00002000 as ::core::ffi::c_int) as ULONG;
    Value |= 0x00004000 as ULONG;
    HalSetBusDataByOffset(
        PCIConfiguration,
        BusNumber,
        PciSlot.u.AsULONG,
        &raw mut Value as PVOID,
        0x64,
        ::core::mem::size_of::<ULONG>() as ULONG,
    );
    return TRUE;
}

unsafe extern "C" fn SvwPataControllerStart(mut Controller: PATA_CONTROLLER) {
    if (*Controller).Pci.DeviceID as ::core::ffi::c_int == (PCI_DEV_OSB4_IDE as i32) {
        PciFindDevice(
            Some(
                SvwOsb4PciBridgeInit
                    as unsafe extern "C" fn(
                        PVOID,
                        ULONG,
                        PCI_SLOT_NUMBER,
                        PPCI_COMMON_HEADER,
                    ) -> BOOLEAN,
            ),
            core::ptr::null_mut(),
        );
    } else {
        let mut ChanData: PCHANNEL_DATA_COMMON =
            (*Controller).Channels[0 as usize] as PCHANNEL_DATA_COMMON;
        let mut UdmaCtrlReg: UCHAR = PciRead8(Controller, SVW_REG_UDMA_CONTROL);
        UdmaCtrlReg = (UdmaCtrlReg as ::core::ffi::c_int
            & (!(SVW_UDMA_CTRL_DISABLE | SVW_UDMA_CTRL_MODE_MASK)) as i32) as UCHAR;
        if (*ChanData).TransferModeSupported & UDMA_MODE5 != 0 {
            UdmaCtrlReg = (UdmaCtrlReg as ::core::ffi::c_int | (SVW_UDMA_CTRL_MODE_UDMA5 as i32)) as UCHAR;
        } else {
            UdmaCtrlReg = (UdmaCtrlReg as ::core::ffi::c_int | (SVW_UDMA_CTRL_MODE_UDMA4 as i32)) as UCHAR;
        }
        PciWrite8(Controller, SVW_REG_UDMA_CONTROL, UdmaCtrlReg);
    };
}
#[no_mangle]
pub unsafe extern "C" fn SvwPataGetControllerProperties(
    Controller: PATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;
    let mut SupportedMode: ULONG;
    let mut No64KDma: BOOLEAN = FALSE;

    PAGED_CODE!();
    ASSERT!((*Controller).Pci.VendorID as u32 == PCI_VEN_SERVERWORKS);

    match ((*Controller).Pci.DeviceID) as u32 {
        PCI_DEV_OSB4_IDE => {
            SupportedMode = PIO_ALL | MWDMA_ALL;
            No64KDma = TRUE;
        }
        PCI_DEV_CSB5_IDE => {
            if (*Controller).Pci.RevisionID < 0x92 {
                SupportedMode = PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 4);
            } else {
                SupportedMode = PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 5);
            }
            (*Controller).Flags |= CTRL_FLAG_DMA_INTERRUPT;
        }
        PCI_DEV_CSB6_IDE_THIRD => {
            SupportedMode = PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 4);
            (*Controller).MaxChannels = 1;
        }
        PCI_DEV_CSB6_IDE => {
            SupportedMode = PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 5);
        }
        PCI_DEV_HT1000_IDE => {
            SupportedMode = PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 5);
            (*Controller).MaxChannels = 1;
            No64KDma = TRUE;
        }
        _ => {
            return STATUS_NO_MATCH;
        }
    }

    Status = PciIdeCreateChannelData(Controller, 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    (*Controller).Start = Some(SvwPataControllerStart);

    i = 0;
    while i < (*Controller).MaxChannels {
        let ChanData: PCHANNEL_DATA_PATA = ((*Controller).Channels[i as usize]).cast();

        if No64KDma != 0 {
            (*ChanData).__bindgen_anon_1.MaximumTransferLength = PCIIDE_PRD_LIMIT - 0x4000;
        }

        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(SvwSetTransferMode);
        (*ChanData).__bindgen_anon_1.TransferModeSupported = SupportedMode;

        if ((*ChanData).__bindgen_anon_1.TransferModeSupported & UDMA_80C_ALL) != 0
            && SvwHasUdmaCable(Controller, i) == 0
        {
            INFO!(
                b"CH %lu: BIOS hasn't selected mode faster than UDMA 2, assume 40-conductor cable\n\0"
                    .as_ptr() as *const i8,
                (*ChanData).Channel,
            );
            (*ChanData).__bindgen_anon_1.TransferModeSupported &= !UDMA_80C_ALL;
        }

        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
