use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut Sil680TaskFileTimings: [USHORT; 5] = [
    0x328a as ::core::ffi::c_int as USHORT,
    0x2283 as ::core::ffi::c_int as USHORT,
    0x1281 as ::core::ffi::c_int as USHORT,
    0x10c3 as ::core::ffi::c_int as USHORT,
    0x10c1 as ::core::ffi::c_int as USHORT,
];

static mut Sil680PioTimings: [USHORT; 5] = [
    0x328a as ::core::ffi::c_int as USHORT,
    0x2283 as ::core::ffi::c_int as USHORT,
    0x1104 as ::core::ffi::c_int as USHORT,
    0x10c3 as ::core::ffi::c_int as USHORT,
    0x10c1 as ::core::ffi::c_int as USHORT,
];

static mut Sil680MwDmaTimings: [USHORT; 3] = [
    0x2208 as ::core::ffi::c_int as USHORT,
    0x10c2 as ::core::ffi::c_int as USHORT,
    0x10c1 as ::core::ffi::c_int as USHORT,
];

static mut Sil680UdmaTimings: [[UCHAR; 7]; 2] = [
    [
        0xb as ::core::ffi::c_int as UCHAR,
        0x7 as ::core::ffi::c_int as UCHAR,
        0x5 as ::core::ffi::c_int as UCHAR,
        0x4 as ::core::ffi::c_int as UCHAR,
        0x2 as ::core::ffi::c_int as UCHAR,
        0x1 as ::core::ffi::c_int as UCHAR,
        0,
    ],
    [
        0xf as ::core::ffi::c_int as UCHAR,
        0xb as ::core::ffi::c_int as UCHAR,
        0x7 as ::core::ffi::c_int as UCHAR,
        0x5 as ::core::ffi::c_int as UCHAR,
        0x3 as ::core::ffi::c_int as UCHAR,
        0x2 as ::core::ffi::c_int as UCHAR,
        0x1 as ::core::ffi::c_int as UCHAR,
    ],
];

pub const PCI_DEV_SIL0680: u32 = 0x0680;

pub const SIL_CFG_CABLE_80: u32 = 0x00000001;

pub const SIL_CFG_MONITOR_IORDY: u32 = 0x00000200;

pub const SIL_STATUS_INTR: u32 = 0x08;

pub const SIL_CLK_MASK: u32 = 0x30;

pub const SIL_CLK_100MHZ: u32 = 0x00;

pub const SIL_CLK_133MHZ: u32 = 0x10;

pub const SIL_CLK_X2: u32 = 0x20;

pub const SIL_CLK_DISABLED: u32 = 0x30;

unsafe extern "C" fn Sil680SetTransferMode(mut Controller: PATA_CONTROLLER, mut Channel: ULONG, mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    unsafe {
        let mut i: ULONG;
        let mut SlowestPioMode: ULONG;
        let mut Config: ULONG;
        let mut ModeReg: UCHAR;
        let mut MonitorIoReady: BOOLEAN = FALSE;

        INFO!(
            b"CH %lu: Config (before)\nMODE %08lX\nCFG  %08lX\nPIO  %08lX\nDMA  %08lX\nUDMA %08lX\n\0".as_ptr() as *const i8,
            Channel,
            PciRead32(Controller, SIL_REG_XFER_MODE!(Channel)),
            PciRead32(Controller, SIL_REG_CFG!(Channel)),
            PciRead32(Controller, SIL_REG_PIO_TIMING!(Channel, 0)),
            PciRead32(Controller, SIL_REG_DMA_TIMING!(Channel, 0)),
            PciRead32(Controller, SIL_REG_UDMA_TIMING!(Channel, 0)),
        );

        SlowestPioMode = UDMA_MODE!(0);
        ModeReg = PciRead8(Controller, SIL_REG_XFER_MODE!(Channel));

        i = 0;
        while i < MAX_IDE_DEVICE {
            let Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
            let mut XferMode: UCHAR = 0;
            let mut SysConfig: UCHAR = 0;

            if !Device.is_null() {
                SlowestPioMode = min(SlowestPioMode, (*Device).PioMode);
                PciWrite16(
                    Controller,
                    SIL_REG_PIO_TIMING!(Channel, i),
                    Sil680PioTimings[(*Device).PioMode as usize],
                );

                if (*Device).PioMode >= PIO_MODE!(3) {
                    MonitorIoReady = TRUE;
                }

                if (*Device).DmaMode >= UDMA_MODE!(0) {
                    SysConfig = PciRead8(Controller, SIL_REG_SYS_CFG);

                    if (SysConfig as u32 & SIL_CLK_MASK) == SIL_CLK_100MHZ {
                        PciWrite8(
                            Controller,
                            SIL_REG_SYS_CFG,
                            (SysConfig as u32 | SIL_CLK_133MHZ) as UCHAR,
                        );
                        SysConfig = PciRead8(Controller, SIL_REG_SYS_CFG);
                    }

                    if ((*Device).DmaMode == UDMA_MODE!(6))
                        && ((SysConfig as u32 & SIL_CLK_MASK) == SIL_CLK_100MHZ)
                    {
                        let mut SupportedModes: ULONG = (*Device).SupportedModes
                            & !(PIO_ALL as u32 | UDMA_MODE6 as u32);
                        if _BitScanReverse(
                            core::ptr::addr_of_mut!((*Device).DmaMode),
                            SupportedModes,
                        ) == 0
                        {
                            (*Device).DmaMode = PIO_MODE!(0);
                        }
                    }
                }

                if (*Device).DmaMode >= UDMA_MODE!(0) {
                    let mut ClockIndex: ULONG = if (SysConfig as u32 & SIL_CLK_MASK)
                        == SIL_CLK_100MHZ
                    {
                        0
                    } else {
                        1
                    };
                    let mut UdmaTim: USHORT;

                    XferMode = 3 as UCHAR;

                    UdmaTim = PciRead16(Controller, SIL_REG_UDMA_TIMING!(Channel, i));
                    UdmaTim = (UdmaTim as u32 & !(0x003F as u32)) as USHORT;
                    UdmaTim = (UdmaTim as u32
                        | Sil680UdmaTimings[ClockIndex as usize]
                            [(*Device).DmaMode.wrapping_sub(UDMA_MODE!(0)) as usize]
                            as u32) as USHORT;
                    PciWrite16(Controller, SIL_REG_UDMA_TIMING!(Channel, i), UdmaTim);
                } else if (*Device).DmaMode != PIO_MODE!(0) {
                    let mut ModeIndex: ULONG = (*Device)
                        .DmaMode
                        .wrapping_sub(MWDMA_MODE!(0));

                    XferMode = 2 as UCHAR;

                    PciWrite16(
                        Controller,
                        SIL_REG_DMA_TIMING!(Channel, i),
                        Sil680MwDmaTimings[ModeIndex as usize],
                    );
                } else {
                    if (*Device).IoReadySupported != 0 {
                        XferMode = 1 as UCHAR;
                    } else {
                        XferMode = 0 as UCHAR;
                    }
                }

                ModeReg = (ModeReg as u32
                    & !((0x03 as u32)
                        << i.wrapping_mul(4))) as UCHAR;
                ModeReg = (ModeReg as u32
                    | (XferMode as u32) << i.wrapping_mul(4)) as UCHAR;
            }

            i = i.wrapping_add(1);
        }

        PciWrite8(Controller, SIL_REG_XFER_MODE!(Channel), ModeReg);

        Config = PciRead32(Controller, SIL_REG_CFG!(Channel));
        if MonitorIoReady != 0 {
            Config |= SIL_CFG_MONITOR_IORDY as u32;
        } else {
            Config &= !(SIL_CFG_MONITOR_IORDY as u32);
        }

        if SlowestPioMode != UDMA_MODE!(0) {
            Config &= !(0xFFFF0000 as u32);
            Config |= (Sil680TaskFileTimings[SlowestPioMode as usize] as u32)
                << 16;
        }

        PciWrite32(Controller, SIL_REG_CFG!(Channel), Config);

        INFO!(
            b"CH %lu: Config (after)\nMODE %08lX\nCFG  %08lX\nPIO  %08lX\nDMA  %08lX\nUDMA %08lX\n\0".as_ptr() as *const i8,
            Channel,
            PciRead32(Controller, SIL_REG_XFER_MODE!(Channel)),
            Config,
            PciRead32(Controller, SIL_REG_PIO_TIMING!(Channel, 0)),
            PciRead32(Controller, SIL_REG_DMA_TIMING!(Channel, 0)),
            PciRead32(Controller, SIL_REG_UDMA_TIMING!(Channel, 0)),
        );
    }
}

unsafe extern "C" fn Sil680CheckInterrupt(ChanData: PCHANNEL_DATA_PATA) -> BOOLEAN {
    let Controller: PATA_CONTROLLER = (*ChanData).__bindgen_anon_1.Controller;
    let Status: UCHAR;

    Status = PciRead8(Controller, SIL_REG_STATUS!((*ChanData).__bindgen_anon_1.Channel));
    return ((Status & (SIL_STATUS_INTR as u8)) != 0) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn Sil680GetControllerProperties(
    Controller: PATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;

    PAGED_CODE!();

    if (*Controller).Pci.VendorID != (PCI_DEV_SIL0680 as u16) {
        return STATUS_NO_MATCH;
    }

    Status = PciIdeCreateChannelData(Controller, 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    i = 0;
    while i < (*Controller).MaxChannels {
        let ChanData: PCHANNEL_DATA_PATA = ((*Controller).Channels[i as usize]).cast();
        let mut Config: UCHAR;

        (*ChanData).CheckInterrupt = Some(Sil680CheckInterrupt);
        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(Sil680SetTransferMode);
        (*ChanData).__bindgen_anon_1.TransferModeSupported = PIO_ALL
            | MWDMA_ALL
            | UDMA_MODES!(0, 6);

        Config = PciRead8(Controller, SIL_REG_CFG!(i));
        if Config & (SIL_CFG_CABLE_80 as u8) == 0 {
            INFO!(
                b"CH %lu: BIOS detected 40-conductor cable\n\0".as_ptr() as *const i8,
                (*ChanData).Channel,
            );
            (*ChanData).__bindgen_anon_1.TransferModeSupported &= !UDMA_80C_ALL;
        }

        i = i.wrapping_add(1);
    }

    return STATUS_NOT_IMPLEMENTED;
}
