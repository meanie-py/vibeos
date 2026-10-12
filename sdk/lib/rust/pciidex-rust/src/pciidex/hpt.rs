// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_158 {
    pub DeviceID: USHORT,
    pub MinimumRevisionID: UCHAR,
    pub Flags: UCHAR,
}

static mut HptControllerList: [C2RustUnnamed_158; 9] = [
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HPT366 as USHORT,
        MinimumRevisionID: 5 as UCHAR,
        Flags: TYPE_372 as UCHAR,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HPT366 as USHORT,
        MinimumRevisionID: 3 as UCHAR,
        Flags: (TYPE_370 | HW_FLAGS_UDMA5) as UCHAR,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HPT366 as USHORT,
        MinimumRevisionID: 0 as UCHAR,
        Flags: (TYPE_366 | HW_FLAGS_UDMA4 | HW_FLAGS_SINGLE_CHAN) as UCHAR,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HPT302 as USHORT,
        MinimumRevisionID: 0 as UCHAR,
        Flags: TYPE_372 as UCHAR,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HPT371 as USHORT,
        MinimumRevisionID: 2 as UCHAR,
        Flags: TYPE_372 as UCHAR,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HPT371 as USHORT,
        MinimumRevisionID: 0 as UCHAR,
        Flags: (TYPE_372 | HW_FLAGS_SINGLE_CHAN) as UCHAR,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HPT372 as USHORT,
        MinimumRevisionID: 0 as UCHAR,
        Flags: TYPE_372 as UCHAR,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HPT374 as USHORT,
        MinimumRevisionID: 0 as UCHAR,
        Flags: TYPE_374 as UCHAR,
    },
    C2RustUnnamed_158 {
        DeviceID: PCI_DEV_HPT372N as USHORT,
        MinimumRevisionID: 0 as UCHAR,
        Flags: TYPE_372 as UCHAR,
    },
];

static mut HptTimings: [[ULONG; 4]; 15] = [
    [
        0xd0a7aa as ::core::ffi::c_int as ULONG,
        0x6914e57 as ::core::ffi::c_int as ULONG,
        0xd029d5e as ::core::ffi::c_int as ULONG,
        0xac1f48a as ::core::ffi::c_int as ULONG,
    ],
    [
        0xd0a7a3 as ::core::ffi::c_int as ULONG,
        0x6914e43 as ::core::ffi::c_int as ULONG,
        0xd029d26 as ::core::ffi::c_int as ULONG,
        0xac1f465 as ::core::ffi::c_int as ULONG,
    ],
    [
        0xd0a753 as ::core::ffi::c_int as ULONG,
        0x6514e33 as ::core::ffi::c_int as ULONG,
        0xc829ca6 as ::core::ffi::c_int as ULONG,
        0xa81f454 as ::core::ffi::c_int as ULONG,
    ],
    [
        0xc8a742 as ::core::ffi::c_int as ULONG,
        0x6514e22 as ::core::ffi::c_int as ULONG,
        0xc829c84 as ::core::ffi::c_int as ULONG,
        0xa81f443 as ::core::ffi::c_int as ULONG,
    ],
    [
        0xc8a731 as ::core::ffi::c_int as ULONG,
        0x6514e21 as ::core::ffi::c_int as ULONG,
        0xc829c62 as ::core::ffi::c_int as ULONG,
        0xa81f442 as ::core::ffi::c_int as ULONG,
    ],
    [
        0x20c8a797 as ::core::ffi::c_int as ULONG,
        0x26514e97 as ::core::ffi::c_int as ULONG,
        0x2c82922e as ::core::ffi::c_int as ULONG,
        0x228082ea as ::core::ffi::c_int as ULONG,
    ],
    [
        0x20c8a732 as ::core::ffi::c_int as ULONG,
        0x26514e33 as ::core::ffi::c_int as ULONG,
        0x2c829266 as ::core::ffi::c_int as ULONG,
        0x22808254 as ::core::ffi::c_int as ULONG,
    ],
    [
        0x20c8a731 as ::core::ffi::c_int as ULONG,
        0x26514e21 as ::core::ffi::c_int as ULONG,
        0x2c829262 as ::core::ffi::c_int as ULONG,
        0x22808242 as ::core::ffi::c_int as ULONG,
    ],
    [
        0x10c8a731 as ::core::ffi::c_int as ULONG,
        0x16514e31 as ::core::ffi::c_int as ULONG,
        0x1c829c62 as ::core::ffi::c_int as ULONG,
        0x121882ea as ::core::ffi::c_int as ULONG,
    ],
    [
        0x10cba731 as ::core::ffi::c_int as ULONG,
        0x164d4e31 as ::core::ffi::c_int as ULONG,
        0x1c9a9c62 as ::core::ffi::c_int as ULONG,
        0x12148254 as ::core::ffi::c_int as ULONG,
    ],
    [
        0x10caa731 as ::core::ffi::c_int as ULONG,
        0x16494e31 as ::core::ffi::c_int as ULONG,
        0x1c929c62 as ::core::ffi::c_int as ULONG,
        0x120c8242 as ::core::ffi::c_int as ULONG,
    ],
    [
        0x10cfa731 as ::core::ffi::c_int as ULONG,
        0x166d4e31 as ::core::ffi::c_int as ULONG,
        0x1c8e9c62 as ::core::ffi::c_int as ULONG,
        0x128c8242 as ::core::ffi::c_int as ULONG,
    ],
    [
        0x10c9a731 as ::core::ffi::c_int as ULONG,
        0x16454e31 as ::core::ffi::c_int as ULONG,
        0x1c8a9c62 as ::core::ffi::c_int as ULONG,
        0x12ac8242 as ::core::ffi::c_int as ULONG,
    ],
    [
        0 as ::core::ffi::c_int as ULONG,
        0x16454e31 as ::core::ffi::c_int as ULONG,
        0x1c8a9c62 as ::core::ffi::c_int as ULONG,
        0x12848242 as ::core::ffi::c_int as ULONG,
    ],
    [
        0 as ::core::ffi::c_int as ULONG,
        0 as ::core::ffi::c_int as ULONG,
        0x1c869c62 as ::core::ffi::c_int as ULONG,
        0x12808242 as ::core::ffi::c_int as ULONG,
    ],
];

static mut HptMwDmaToPioMode: [ULONG; 3] = [
    1 as ::core::ffi::c_int as ULONG,
    3 as ::core::ffi::c_int as ULONG,
    4 as ::core::ffi::c_int as ULONG,
];

static mut Hpt37xEnableBits: [ATA_PCI_ENABLE_BITS; 2] = [
    _ATA_PCI_ENABLE_BITS {
        Register: 0x50 as UCHAR,
        Mask: 0x4 as UCHAR,
        ValueEnabled: 0x4 as UCHAR,
    },
    _ATA_PCI_ENABLE_BITS {
        Register: 0x54 as UCHAR,
        Mask: 0x4 as UCHAR,
        ValueEnabled: 0x4 as UCHAR,
    },
];

static mut Hpt366EnableBits: [ATA_PCI_ENABLE_BITS; 1] = [_ATA_PCI_ENABLE_BITS {
    Register: 0x50 as UCHAR,
    Mask: 0x30 as UCHAR,
    ValueEnabled: 0x30 as UCHAR,
}];

pub const PCI_DEV_HPT366: u16 = 0x0004;
pub const PCI_DEV_HPT372: u16 = 0x0005;
pub const PCI_DEV_HPT302: u16 = 0x0006;
pub const PCI_DEV_HPT371: u16 = 0x0007;
pub const PCI_DEV_HPT374: u16 = 0x0008;
pub const PCI_DEV_HPT372N: u16 = 0x0009;

pub const TYPE_366: u8 = 0;
pub const TYPE_370: u8 = 1;
pub const TYPE_372: u8 = 2;
pub const TYPE_374: u8 = 3;

pub const HW_FLAGS_TYPE_MASK: u8 = 0x03;
pub const HW_FLAGS_SINGLE_CHAN: u8 = 0x04;
pub const HW_FLAGS_UDMA4: u8 = 0x08;
pub const HW_FLAGS_UDMA5: u8 = 0x10;

pub const HPT366_TIM_DATA_HIGH_CYT_MASK: u32 = 0x0000000F;
pub const HPT366_TIM_DATA_LOW_CYT_MASK: u32 = 0x000000F0;
pub const HPT366_TIM_CMD_HIGH_CYT_MASK: u32 = 0x00000F00;
pub const HPT366_TIM_CMD_LOW_CYT_MASK: u32 = 0x0000F000;
pub const HPT366_TIM_UDMA_CYT_MASK: u32 = 0x00070000;
pub const HPT366_TIM_PRE_HIGH_CYT_MASK: u32 = 0x00380000;
pub const HPT366_TIM_CMD_PRE_HIGH_CYT_MASK: u32 = 0x01C00000;
pub const HPT366_TIM_UDMA_ENABLE: u32 = 0x10000000;
pub const HPT366_TIM_DMA_ENABLE: u32 = 0x20000000;
pub const HPT366_TIM_PIO_MST_ENABLE: u32 = 0x40000000;
pub const HPT366_TIM_PIO_BUFFER_ENABLE: u32 = 0x80000000;

pub const HPT37X_TIM_DATA_HIGH_CYT_MASK: u32 = 0x0000000F;
pub const HPT37X_TIM_DATA_LOW_CYT_MASK: u32 = 0x000001F0;
pub const HPT37X_TIM_CMD_HIGH_CYT_MASK: u32 = 0x00001E00;
pub const HPT37X_TIM_CMD_LOW_CYT_MASK: u32 = 0x0003E000;
pub const HPT37X_TIM_UDMA_CYT_MASK: u32 = 0x001C0000;
pub const HPT37X_TIM_UDMA_DUAL_CLOCK_ENABLE: u32 = 0x00200000;
pub const HPT37X_TIM_PRE_HIGH_CYT_MASK: u32 = 0x01C00000;
pub const HPT37X_TIM_CMD_PRE_HIGH_CYT_MASK: u32 = 0x0E000000;
pub const HPT37X_TIM_UDMA_ENABLE: u32 = 0x10000000;
pub const HPT37X_TIM_DMA_ENABLE: u32 = 0x20000000;
pub const HPT37X_TIM_PIO_MST_ENABLE: u32 = 0x40000000;
pub const HPT37X_TIM_PIO_BUFFER_ENABLE: u32 = 0x80000000;

#[repr(C)]
pub struct HPT_CONTROLLER_LIST {
    pub DeviceID: USHORT,
    pub MinimumRevisionID: UCHAR,
    pub Flags: UCHAR,
}
pub type PHPT_CONTROLLER_LIST = *mut HPT_CONTROLLER_LIST;
unsafe extern "C" fn HptChooseTimings(Device: PCHANNEL_DEVICE_CONFIG, HptType: ULONG) -> ULONG {
    let mut Mask: ULONG = 0;
    let mut Mode: ULONG;
    let PioTimings: ULONG;
    let DmaTimings: ULONG;

    Mode = (*Device).PioMode;
    PioTimings = HptTimings[Mode as usize][HptType as usize];

    Mode = (*Device).DmaMode;
    if Mode >= MWDMA_MODE!(0) {
        Mode = Mode.wrapping_sub(
            (MWDMA_MODE!(0) as ULONG).wrapping_sub(SWDMA_MODE!(0) as ULONG),
        );
    }
    DmaTimings = HptTimings[Mode as usize][HptType as usize];

    if (*Device).DmaMode >= UDMA_MODE!(0) {
        if HptType == (TYPE_366 as u32) {
            Mask = HPT366_TIM_UDMA_CYT_MASK
                | HPT366_TIM_UDMA_ENABLE
                | HPT366_TIM_PIO_MST_ENABLE;
        } else {
            Mask = HPT37X_TIM_UDMA_CYT_MASK
                | HPT37X_TIM_UDMA_DUAL_CLOCK_ENABLE
                | HPT37X_TIM_UDMA_ENABLE
                | HPT37X_TIM_PIO_MST_ENABLE;
        }
    } else if (*Device).DmaMode != PIO_MODE!(0) {
        Mask = 0xFFFFFFFF;

        Mode = (*Device).DmaMode;
        loop {
            if Mode <= PIO_MODE!(0) {
                break;
            }

            if ((*Device).SupportedModes & !(PIO_ALL) & (1 << Mode)) == 0 {
                Mode = Mode.wrapping_sub(1);
                continue;
            }

            let MinPio: ULONG =
                HptMwDmaToPioMode[Mode.wrapping_sub(MWDMA_MODE!(0)) as usize];

            if (*Device).SupportedModes & (1 << MinPio) != 0 {
                (*Device).PioMode = MinPio;
                break;
            }

            Mode = Mode.wrapping_sub(1);
        }

        if Mode != (*Device).DmaMode {
            if Mode == PIO_MODE!(0) {
                WARN!(
                    b"Too slow device '%s', disabling DMA\n\0".as_ptr() as PCSTR,
                    (*Device).FriendlyName,
                );
            } else {
                INFO!(
                    b"Downgrade DMA speed from %lu to %lu\n\0".as_ptr() as PCSTR,
                    (*Device).DmaMode,
                    Mode,
                );
            }
        }
        (*Device).DmaMode = Mode;
    }

    if (*Device).DmaMode == PIO_MODE!(0) {
        Mask = 0;
    }

    (PioTimings & !Mask) | (DmaTimings & Mask)
}
unsafe extern "C" fn HptSetTransferMode(
    Controller: PATA_CONTROLLER,
    Channel: ULONG,
    DeviceList: *mut PCHANNEL_DEVICE_CONFIG,
) {
    let ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[Channel as usize] as PCHANNEL_DATA_PATA;
    let mut i: ULONG = 0;
    while i < MAX_IDE_DEVICE {
        let Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let DeviceIndex: ULONG = (Channel << 1).wrapping_add(i);
        let mut Value: ULONG;
        if !Device.is_null() {
            Value = HptChooseTimings(Device, GET_TYPE!((*ChanData).HwFlags));

            INFO!(
                b"CH %lu: Dev %lu update timings from %08lx to %08lx\n\0".as_ptr() as PCHAR,
                Channel,
                i,
                PciRead32(Controller, HPT_REG_TIMING(DeviceIndex)),
                Value,
            );

            PciWrite32(Controller, HPT_REG_TIMING(DeviceIndex), Value);
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn HptControllerStart(Controller: PATA_CONTROLLER) {
    let mut ChanData: PCHANNEL_DATA_PATA =
        (*Controller).Channels[0 as usize] as PCHANNEL_DATA_PATA;
    let mut HwFlags: ULONG = (*ChanData).HwFlags;
    let mut Value: UCHAR = 0;

    if GET_TYPE!(HwFlags) == (TYPE_366 as u32) {
        /* Disable interrupt prediction */
        Value = PciRead8(Controller, 0x51);
        Value &= !0x80;
        PciWrite8(Controller, 0x51, Value);
    } else {
        /* Disable interrupt prediction */
        Value = PciRead8(Controller, 0x51);
        Value &= !0x03;
        PciWrite8(Controller, 0x51, Value);

        Value = PciRead8(Controller, 0x55);
        Value &= !0x03;
        PciWrite8(Controller, 0x55, Value);

        /* Enable interrupts */
        Value = PciRead8(Controller, 0x5A);
        Value &= !0x10;
        PciWrite8(Controller, 0x5A, Value);

        /* Set clocks */
        if GET_TYPE!(HwFlags) < (TYPE_372 as u32) {
            PciWrite8(Controller, 0x5B, 0x22);
        } else {
            Value = PciRead8(Controller, 0x5B);
            Value &= 0x01;
            Value |= 0x20;
            PciWrite8(Controller, 0x5B, Value);
        }
    }
}
unsafe extern "C" fn HptHasUdmaCable(
    Controller: PATA_CONTROLLER,
    HwFlags: ULONG,
    Channel: ULONG,
) -> BOOLEAN {
    let mut Register: UCHAR = 0;
    let mut Value: UCHAR = 0;
    let mut CableSelect: UCHAR = 0;

    PAGED_CODE!();

    if GET_TYPE!(HwFlags) != (TYPE_366 as u32) {
        if GET_TYPE!(HwFlags) == (TYPE_374 as u32) && ((*Controller).BusLocation & 1) != 0 {
            Register = (if Channel == 0 {
                0x53
            } else {
                0x57
            }) as UCHAR;
            Value = PciRead8(Controller, Register as ULONG);
            PciWrite8(
                Controller,
                Register as ULONG,
                (Value as ULONG | 0x80) as UCHAR,
            );
        } else {
            Register = 0x5B as UCHAR;
            Value = PciRead8(Controller, Register as ULONG);
            PciWrite8(
                Controller,
                Register as ULONG,
                (Value as ULONG & !(0x01)) as UCHAR,
            );
        }
    }

    CableSelect = PciRead8(Controller, 0x5A) as UCHAR;

    if GET_TYPE!(HwFlags) != (TYPE_366 as u32) {
        PciWrite8(Controller, Register as ULONG, Value);
    }

    return ((CableSelect as ULONG & (0x02 >> Channel)) == 0) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn HptGetControllerProperties(Controller: PATA_CONTROLLER) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;
    let mut HwFlags: ULONG = core::mem::zeroed();

    PAGED_CODE!();
    ASSERT!((*Controller).Pci.VendorID == PCI_VEN_HIGHPOINT as _);

    i = 0 as ULONG;
    while i < RTL_NUMBER_OF!(HptControllerList) as ULONG {
        HwFlags = HptControllerList[i as usize].Flags as ULONG;

        if (*Controller).Pci.DeviceID == HptControllerList[i as usize].DeviceID
            && (*Controller).Pci.RevisionID >= HptControllerList[i as usize].MinimumRevisionID
        {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == RTL_NUMBER_OF!(HptControllerList) as ULONG {
        return STATUS_NO_MATCH as NTSTATUS;
    }

    (*Controller).Flags |= CTRL_FLAG_DMA_INTERRUPT as ULONG;
    (*Controller).Start = Some(HptControllerStart);

    if HwFlags & HW_FLAGS_SINGLE_CHAN as ULONG != 0 {
        (*Controller).MaxChannels = 1 as ULONG;
    }

    if GET_TYPE!(HwFlags) == TYPE_366 as ULONG {
        (*Controller).__bindgen_anon_1.ChannelEnableBits =
            core::ptr::addr_of!(Hpt366EnableBits) as *const ATA_PCI_ENABLE_BITS;
    } else {
        (*Controller).__bindgen_anon_1.ChannelEnableBits =
            core::ptr::addr_of!(Hpt37xEnableBits) as *const ATA_PCI_ENABLE_BITS;
    }

    Status = PciIdeCreateChannelData(Controller, 0 as ULONG);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    i = 0 as ULONG;
    while i < (*Controller).MaxChannels {
        let ChanData: PCHANNEL_DATA_PATA =
            (*Controller).Channels[i as usize] as PCHANNEL_DATA_PATA;

        (*ChanData).HwFlags = HwFlags;
        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(HptSetTransferMode);

        (*ChanData).__bindgen_anon_1.TransferModeSupported =
            (PIO_ALL | MWDMA_ALL | UDMA_ALL) as ULONG;
        if HwFlags & HW_FLAGS_UDMA5 as ULONG != 0 {
            (*ChanData).__bindgen_anon_1.TransferModeSupported &= !(UDMA_MODE6 as ULONG);
        } else if HwFlags & HW_FLAGS_UDMA4 as ULONG != 0 {
            (*ChanData).__bindgen_anon_1.TransferModeSupported &=
                !((UDMA_MODE6 | UDMA_MODE5) as ULONG);
        }

        if HptHasUdmaCable(Controller, HwFlags, i) == 0 {
            INFO!(
                b"CH %lu: BIOS detected 40-conductor cable\n\0".as_ptr() as PCHAR,
                (*ChanData).Channel,
            );
            (*ChanData).__bindgen_anon_1.TransferModeSupported &=
                !(UDMA_80C_ALL as ULONG);
        }
        i = i.wrapping_add(1);
    }

    STATUS_SUCCESS as NTSTATUS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS ATA Bus Driver
//  * LICENSE:     BSD-2-Clause (https://spdx.org/licenses/BSD-2-Clause)
//  * PURPOSE:     HighPoint ATA controller minidriver
//  * COPYRIGHT:   Copyright 2026 Dmitry Borisov <di.sean@protonmail.com>
//  */
//
// /*
//  * Adapted from the FreeBSD ata-highpoint driver
//  * Copyright (c) 1998-2008 Søren Schmidt <sos@FreeBSD.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "pciidex.h"
//
// /* GLOBALS ********************************************************************/
//
// #define PCI_DEV_HPT366      0x0004
// #define PCI_DEV_HPT372      0x0005
// #define PCI_DEV_HPT302      0x0006
// #define PCI_DEV_HPT371      0x0007
// #define PCI_DEV_HPT374      0x0008
// #define PCI_DEV_HPT372N     0x0009
//
// #define TYPE_366             0
// #define TYPE_370             1
// #define TYPE_372             2
// #define TYPE_374             3
//
// #define GET_TYPE(Flags)  ((Flags) & HW_FLAGS_TYPE_MASK)
//
// #define HW_FLAGS_TYPE_MASK           0x03
// #define HW_FLAGS_SINGLE_CHAN         0x04
// #define HW_FLAGS_UDMA4               0x08
// #define HW_FLAGS_UDMA5               0x10
//
// #define HPT_REG_TIMING(Index)    (0x40 + (Index << 2))
#[inline(always)]
const fn HPT_REG_TIMING(Index: ULONG) -> ULONG {
    0x40 + (Index << 2)
}
//
// #define HPT366_TIM_DATA_HIGH_CYT_MASK      0x0000000F
// #define HPT366_TIM_DATA_LOW_CYT_MASK       0x000000F0
// #define HPT366_TIM_CMD_HIGH_CYT_MASK       0x00000F00
// #define HPT366_TIM_CMD_LOW_CYT_MASK        0x0000F000
// #define HPT366_TIM_UDMA_CYT_MASK           0x00070000
// #define HPT366_TIM_PRE_HIGH_CYT_MASK       0x00380000
// #define HPT366_TIM_CMD_PRE_HIGH_CYT_MASK   0x01C00000
// #define HPT366_TIM_UDMA_ENABLE             0x10000000
// #define HPT366_TIM_DMA_ENABLE              0x20000000
// #define HPT366_TIM_PIO_MST_ENABLE          0x40000000
// #define HPT366_TIM_PIO_BUFFER_ENABLE       0x80000000
//
// #define HPT37X_TIM_DATA_HIGH_CYT_MASK      0x0000000F
// #define HPT37X_TIM_DATA_LOW_CYT_MASK       0x000001F0
// #define HPT37X_TIM_CMD_HIGH_CYT_MASK       0x00001E00
// #define HPT37X_TIM_CMD_LOW_CYT_MASK        0x0003E000
// #define HPT37X_TIM_UDMA_CYT_MASK           0x001C0000
// #define HPT37X_TIM_UDMA_DUAL_CLOCK_ENABLE  0x00200000
// #define HPT37X_TIM_PRE_HIGH_CYT_MASK       0x01C00000
// #define HPT37X_TIM_CMD_PRE_HIGH_CYT_MASK   0x0E000000
// #define HPT37X_TIM_UDMA_ENABLE             0x10000000
// #define HPT37X_TIM_DMA_ENABLE              0x20000000
// #define HPT37X_TIM_PIO_MST_ENABLE          0x40000000
// #define HPT37X_TIM_PIO_BUFFER_ENABLE       0x80000000
//
//
// static const struct
// {
//     USHORT DeviceID;
//     UCHAR MinimumRevisionID;
//     UCHAR Flags;
// } HptControllerList[] =
// {
//     { PCI_DEV_HPT366,  5, TYPE_372 },
//     { PCI_DEV_HPT366,  3, TYPE_370 | HW_FLAGS_UDMA5 },
//     { PCI_DEV_HPT366,  0, TYPE_366 | HW_FLAGS_UDMA4 | HW_FLAGS_SINGLE_CHAN },
//
//     { PCI_DEV_HPT302,  0, TYPE_372 },
//
//     { PCI_DEV_HPT371,  2, TYPE_372 },
//     { PCI_DEV_HPT371,  0, TYPE_372 | HW_FLAGS_SINGLE_CHAN},
//
//     { PCI_DEV_HPT372,  0, TYPE_372 },
//     { PCI_DEV_HPT374,  0, TYPE_374 },
//     { PCI_DEV_HPT372N, 0, TYPE_372 },
// };
//
// static const ULONG HptTimings[][4] =
// {
//     //  HPT366      HPT370      HPT372      HPT374         mode
//     { 0x00d0a7aa, 0x06914e57, 0x0d029d5e, 0x0ac1f48a }, // PIO 0
//     { 0x00d0a7a3, 0x06914e43, 0x0d029d26, 0x0ac1f465 }, // PIO 1
//     { 0x00d0a753, 0x06514e33, 0x0c829ca6, 0x0a81f454 }, // PIO 2
//     { 0x00c8a742, 0x06514e22, 0x0c829c84, 0x0a81f443 }, // PIO 3
//     { 0x00c8a731, 0x06514e21, 0x0c829c62, 0x0a81f442 }, // PIO 4
//
//     { 0x20c8a797, 0x26514e97, 0x2c82922e, 0x228082ea }, // MWDMA 0
//     { 0x20c8a732, 0x26514e33, 0x2c829266, 0x22808254 }, // MWDMA 1
//     { 0x20c8a731, 0x26514e21, 0x2c829262, 0x22808242 }, // MWDMA 2
//
//     { 0x10c8a731, 0x16514e31, 0x1c829c62, 0x121882ea }, // UDMA 0
//     { 0x10cba731, 0x164d4e31, 0x1c9a9c62, 0x12148254 }, // UDMA 1
//     { 0x10caa731, 0x16494e31, 0x1c929c62, 0x120c8242 }, // UDMA 2
//     { 0x10cfa731, 0x166d4e31, 0x1c8e9c62, 0x128c8242 }, // UDMA 3
//     { 0x10c9a731, 0x16454e31, 0x1c8a9c62, 0x12ac8242 }, // UDMA 4
//     { 0,          0x16454e31, 0x1c8a9c62, 0x12848242 }, // UDMA 5
//     { 0,          0,          0x1c869c62, 0x12808242 }  // UDMA 6
// };
//
// static const ULONG HptMwDmaToPioMode[] =
// {
//     PIO_MODE(1), // MWDMA_MODE(0)
//     PIO_MODE(3), // MWDMA_MODE(1)
//     PIO_MODE(4), // MWDMA_MODE(2)
// };
//
//
// static const ATA_PCI_ENABLE_BITS Hpt37xEnableBits[MAX_IDE_CHANNEL] =
// {
//     { 0x50, 0x04, 0x04 },
//     { 0x54, 0x04, 0x04 },
// };
//
//
// static const ATA_PCI_ENABLE_BITS Hpt366EnableBits[1] =
// {
//     { 0x50, 0x30, 0x30 },
// };
//
// /* FUNCTIONS ******************************************************************/
//
// static
// ULONG
// HptChooseTimings(
//             PCHANNEL_DEVICE_CONFIG Device,
//          ULONG HptType)
// {
//     ULONG Mask, Mode, PioTimings, DmaTimings;
//
//     Mode = Device->PioMode;
//     PioTimings = HptTimings[Mode][HptType];
//
//     Mode = Device->DmaMode;
//     if (Mode >= MWDMA_MODE(0))
//         Mode -= MWDMA_MODE(0) - SWDMA_MODE(0);
//     DmaTimings = HptTimings[Mode][HptType];
//
//     if (Device->DmaMode >= UDMA_MODE(0))
//     {
//         if (HptType == TYPE_366)
//         {
//             Mask = HPT366_TIM_UDMA_CYT_MASK |
//                    HPT366_TIM_UDMA_ENABLE |
//                    HPT366_TIM_PIO_MST_ENABLE;
//         }
//         else
//         {
//             Mask = HPT37X_TIM_UDMA_CYT_MASK |
//                    HPT37X_TIM_UDMA_DUAL_CLOCK_ENABLE |
//                    HPT37X_TIM_UDMA_ENABLE |
//                    HPT37X_TIM_PIO_MST_ENABLE;
//         }
//     }
//     else if (Device->DmaMode != PIO_MODE(0))
//     {
//         Mask = 0xFFFFFFFF;
//
//         for (Mode = Device->DmaMode; Mode > PIO_MODE(0); Mode--)
//         {
//             ULONG MinPio;
//
//             if (!(Device->SupportedModes & ~PIO_ALL & (1 << Mode)))
//                 continue;
//
//             MinPio = HptMwDmaToPioMode[Mode - MWDMA_MODE(0)];
//
//             if (Device->SupportedModes & (1 << MinPio))
//             {
//                 Device->PioMode = MinPio;
//                 break;
//             }
//         }
//         if (Mode != Device->DmaMode)
//         {
//             if (Mode == PIO_MODE(0))
//                 WARN("Too slow device '%s', disabling DMA\n", Device->FriendlyName);
//             else
//                 INFO("Downgrade DMA speed from %lu to %lu\n", Device->DmaMode, Mode);
//         }
//         Device->DmaMode = Mode;
//     }
//
//     if (Device->DmaMode == PIO_MODE(0))
//         Mask = 0;
//
//     return (PioTimings & ~Mask) | (DmaTimings & Mask);
// }
//
// static
// VOID
// HptSetTransferMode(
//          PATA_CONTROLLER Controller,
//          ULONG Channel,
//                                PCHANNEL_DEVICE_CONFIG* DeviceList)
// {
//     PCHANNEL_DATA_PATA ChanData = Controller->Channels[Channel];
//     ULONG i;
//
//     for (i = 0; i < MAX_IDE_DEVICE; ++i)
//     {
//         PCHANNEL_DEVICE_CONFIG Device = DeviceList[i];
//         const ULONG DeviceIndex = (Channel << 1) + i;
//         ULONG Value;
//
//         if (!Device)
//             continue;
//
//         Value = HptChooseTimings(Device, GET_TYPE(ChanData->HwFlags));
//
//         INFO("CH %lu: Dev %lu update timings from %08lx to %08lx\n",
//              Channel,
//              i,
//              PciRead32(Controller, HPT_REG_TIMING(DeviceIndex)),
//              Value);
//
//         PciWrite32(Controller, HPT_REG_TIMING(DeviceIndex), Value);
//     }
// }
//
// static
// VOID
// HptControllerStart(
//          PATA_CONTROLLER Controller)
// {
//     PCHANNEL_DATA_PATA ChanData = Controller->Channels[0];
//     ULONG HwFlags = ChanData->HwFlags;
//     UCHAR Value;
//
//     if (GET_TYPE(HwFlags) == TYPE_366)
//     {
//         /* Disable interrupt prediction */
//         Value = PciRead8(Controller, 0x51);
//         Value &= ~0x80;
//         PciWrite8(Controller, 0x51, Value);
//     }
//     else
//     {
//         /* Disable interrupt prediction */
//         Value = PciRead8(Controller, 0x51);
//         Value &= ~0x03;
//         PciWrite8(Controller, 0x51, Value);
//
//         Value = PciRead8(Controller, 0x55);
//         Value &= ~0x03;
//         PciWrite8(Controller, 0x55, Value);
//
//         /* Enable interrupts */
//         Value = PciRead8(Controller, 0x5A);
//         Value &= ~0x10;
//         PciWrite8(Controller, 0x5A, Value);
//
//         /* Set clocks */
//         if (GET_TYPE(HwFlags) < TYPE_372)
//         {
//             PciWrite8(Controller, 0x5B, 0x22);
//         }
//         else
//         {
//             Value = PciRead8(Controller, 0x5B);
//             Value &= 0x01;
//             Value |= 0x20;
//             PciWrite8(Controller, 0x5B, Value);
//         }
//     }
// }
//
// static
// CODE_SEG("PAGE")
// BOOLEAN
// HptHasUdmaCable(
//          PATA_CONTROLLER Controller,
//          ULONG HwFlags,
//          ULONG Channel)
// {
//     UCHAR Register, Value, CableSelect;
//
//     PAGED_CODE();
//
//     /* Unlock the cable state pins */
//     if (GET_TYPE(HwFlags) != TYPE_366)
//     {
//         if ((GET_TYPE(HwFlags) == TYPE_374) && (Controller->BusLocation & 1))
//         {
//             Register = (Channel == 0) ? 0x53 : 0x57;
//             Value = PciRead8(Controller, Register);
//             PciWrite8(Controller, Register, Value | 0x80);
//         }
//         else
//         {
//             Register = 0x5B;
//             Value = PciRead8(Controller, Register);
//             PciWrite8(Controller, Register, Value & ~0x01);
//         }
//     }
//
//     CableSelect = PciRead8(Controller, 0x5A);
//
//     /* Restore settings */
//     if (GET_TYPE(HwFlags) != TYPE_366)
//         PciWrite8(Controller, Register, Value);
//
//     return !(CableSelect & (0x02 >> Channel));
// }
//
// CODE_SEG("PAGE")
// NTSTATUS
// HptGetControllerProperties(
//             PATA_CONTROLLER Controller)
// {
//     NTSTATUS Status;
//     ULONG i, HwFlags;
//
//     PAGED_CODE();
//     ASSERT(Controller->Pci.VendorID == PCI_VEN_HIGHPOINT);
//
//     for (i = 0; i < RTL_NUMBER_OF(HptControllerList); ++i)
//     {
//         HwFlags = HptControllerList[i].Flags;
//
//         if ((Controller->Pci.DeviceID == HptControllerList[i].DeviceID) &&
//             (Controller->Pci.RevisionID >= HptControllerList[i].MinimumRevisionID))
//         {
//             break;
//         }
//     }
//     if (i == RTL_NUMBER_OF(HptControllerList))
//         return STATUS_NO_MATCH;
//
//     Controller->Flags |= CTRL_FLAG_DMA_INTERRUPT;
//     Controller->Start = HptControllerStart;
//
//     if (HwFlags & HW_FLAGS_SINGLE_CHAN)
//         Controller->MaxChannels = 1;
//
//     if (GET_TYPE(HwFlags) == TYPE_366)
//         Controller->ChannelEnableBits = Hpt366EnableBits;
//     else
//         Controller->ChannelEnableBits = Hpt37xEnableBits;
//
//     Status = PciIdeCreateChannelData(Controller, 0);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     for (i = 0; i < Controller->MaxChannels; ++i)
//     {
//         PCHANNEL_DATA_PATA ChanData = Controller->Channels[i];
//
//         ChanData->HwFlags = HwFlags;
//         ChanData->SetTransferMode = HptSetTransferMode;
//
//         ChanData->TransferModeSupported = PIO_ALL | MWDMA_ALL | UDMA_ALL;
//         if (HwFlags & HW_FLAGS_UDMA5)
//             ChanData->TransferModeSupported &= ~UDMA_MODE6;
//         else if (HwFlags & HW_FLAGS_UDMA4)
//             ChanData->TransferModeSupported &= ~(UDMA_MODE6 | UDMA_MODE5);
//
//         /* Check for 80-conductor cable */
//         if (!HptHasUdmaCable(Controller, HwFlags, i))
//         {
//             INFO("CH %lu: BIOS detected 40-conductor cable\n", ChanData->Channel);
//             ChanData->TransferModeSupported &= ~UDMA_80C_ALL;
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
