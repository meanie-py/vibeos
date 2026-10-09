use crate::prelude::*;
use ntkrnl_sys::ddk::{PCI_SLOT_NUMBER, PPCI_COMMON_HEADER};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_163 {
    pub DeviceID: USHORT,
    pub MinimumRevisionID: UCHAR,
    pub Flags: UCHAR,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_162 {
    pub DefaultValue: UCHAR,
    pub Data: [UCHAR; 7],
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_164 {
    pub DeviceID: USHORT,
    pub Flags: USHORT,
}

static mut ViaControllerList: [C2RustUnnamed_164; 17] = [
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT82C576M_IDE as USHORT,
        Flags: TYPE_MWDMA as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT82C586A_IDE as USHORT,
        Flags: HW_FLAGS_CHECK_BRIDGE as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT6410_IDE_RAID as USHORT,
        Flags: TYPE_133 as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT6415_IDE as USHORT,
        Flags: (TYPE_133 | HW_FLAGS_SINGLE_CHAN) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT6420_IDE as USHORT,
        Flags: (TYPE_133 | HW_FLAGS_SINGLE_CHAN) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VX855_IDE as USHORT,
        Flags: (TYPE_133 | HW_FLAGS_SINGLE_CHAN) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT8237_SATA as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_SINGLE_PORT) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT8237A_SATA as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_SINGLE_PORT) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT8237A_SATA_2 as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_SINGLE_PORT) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT8237S_SATA as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_SINGLE_PORT) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT8237S_SATA_RAID as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_SINGLE_PORT | HW_FLAGS_PCI_SCR) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT8251_SATA_AHCI as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_PCI_SCR) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT8251_SATA_2 as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_PCI_SCR) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT8261_SATA as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_PCI_SCR) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VT8261_SATA_RAID as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_PCI_SCR) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VX900_SATA as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_PCI_SCR) as USHORT,
    },
    C2RustUnnamed_164 {
        DeviceID: PCI_DEV_VX900_SATA_RAID as USHORT,
        Flags: (TYPE_SATA | HW_FLAGS_PCI_SCR) as USHORT,
    },
];

static mut ViaUdmaTimings: [C2RustUnnamed_162; 4] = [
    C2RustUnnamed_162 {
        DefaultValue: 0x3 as UCHAR,
        Data: [
            0xe2 as ::core::ffi::c_int as UCHAR,
            0xe1 as ::core::ffi::c_int as UCHAR,
            0xe0 as ::core::ffi::c_int as UCHAR,
            0,
            0,
            0,
            0,
        ],
    },
    C2RustUnnamed_162 {
        DefaultValue: 0x3 as UCHAR,
        Data: [
            0xe6 as ::core::ffi::c_int as UCHAR,
            0xe4 as ::core::ffi::c_int as UCHAR,
            0xe2 as ::core::ffi::c_int as UCHAR,
            0xe1 as ::core::ffi::c_int as UCHAR,
            0xe0 as ::core::ffi::c_int as UCHAR,
            0,
            0,
        ],
    },
    C2RustUnnamed_162 {
        DefaultValue: 0x7 as UCHAR,
        Data: [
            0xea as ::core::ffi::c_int as UCHAR,
            0xe6 as ::core::ffi::c_int as UCHAR,
            0xe4 as ::core::ffi::c_int as UCHAR,
            0xe2 as ::core::ffi::c_int as UCHAR,
            0xe1 as ::core::ffi::c_int as UCHAR,
            0xe0 as ::core::ffi::c_int as UCHAR,
            0,
        ],
    },
    C2RustUnnamed_162 {
        DefaultValue: 0x7 as UCHAR,
        Data: [
            0xee as ::core::ffi::c_int as UCHAR,
            0xe8 as ::core::ffi::c_int as UCHAR,
            0xe6 as ::core::ffi::c_int as UCHAR,
            0xe4 as ::core::ffi::c_int as UCHAR,
            0xe2 as ::core::ffi::c_int as UCHAR,
            0xe1 as ::core::ffi::c_int as UCHAR,
            0xe0 as ::core::ffi::c_int as UCHAR,
        ],
    },
];

static mut ViaPataEnableBits: [ATA_PCI_ENABLE_BITS; 2] = [
    _ATA_PCI_ENABLE_BITS {
        Register: 0x40 as UCHAR,
        Mask: 0x2 as UCHAR,
        ValueEnabled: 0x2 as UCHAR,
    },
    _ATA_PCI_ENABLE_BITS {
        Register: 0x40 as UCHAR,
        Mask: 0x1 as UCHAR,
        ValueEnabled: 0x1 as UCHAR,
    },
];

static mut ViaCx700EnableBits: [ATA_PCI_ENABLE_BITS; 2] = [
    _ATA_PCI_ENABLE_BITS {
        Register: 0x40 as UCHAR,
        Mask: 0x2 as UCHAR,
        ValueEnabled: 0x2 as UCHAR,
    },
    _ATA_PCI_ENABLE_BITS {
        Register: 0xc0 as UCHAR,
        Mask: 0x1 as UCHAR,
        ValueEnabled: 0x1 as UCHAR,
    },
];

pub const PCI_DEV_VT82C576M_IDE: u16 = 0x1571;
pub const PCI_DEV_VT82C586A_IDE: u16 = 0x0571;
pub const PCI_DEV_VT6410_IDE_RAID: u16 = 0x3164;
pub const PCI_DEV_VT6415_IDE: u16 = 0x0415;
pub const PCI_DEV_VT6420_IDE: u16 = 0x4149;
pub const PCI_DEV_VT6421A_RAID: u16 = 0x3249;
pub const PCI_DEV_CX700M2_IDE_SATA: u16 = 0x5324;
pub const PCI_DEV_CX700M2_IDE_SATA_RAID: u16 = 0x0581;
pub const PCI_DEV_VX855_IDE: u16 = 0xC409;
pub const PCI_DEV_VT8237_SATA: u16 = 0x3149;
pub const PCI_DEV_VT8237A_SATA: u16 = 0x0591;
pub const PCI_DEV_VT8237A_SATA_2: u16 = 0x5337;
pub const PCI_DEV_VT8237S_SATA: u16 = 0x5372;
pub const PCI_DEV_VT8237S_SATA_RAID: u16 = 0x7372;
pub const PCI_DEV_VT8251_SATA_AHCI: u16 = 0x3349;
pub const PCI_DEV_VT8251_SATA_2: u16 = 0x5287;
pub const PCI_DEV_VT8261_SATA: u16 = 0x9000;
pub const PCI_DEV_VT8261_SATA_RAID: u16 = 0x9040;
pub const PCI_DEV_VX900_SATA: u16 = 0x9001;
pub const PCI_DEV_VX900_SATA_RAID: u16 = 0x9041;
pub const PCI_DEV_BRIDGE_VT82C586: u16 = 0x0586;
pub const PCI_DEV_BRIDGE_VT82C596A: u16 = 0x0596;
pub const PCI_DEV_BRIDGE_VT82C686A: u16 = 0x0686;
pub const PCI_DEV_BRIDGE_VT8231: u16 = 0x8231;
pub const PCI_DEV_BRIDGE_VT8233: u16 = 0x3074;
pub const PCI_DEV_BRIDGE_VT8233A: u16 = 0x3147;
pub const PCI_DEV_BRIDGE_VT8233C: u16 = 0x3109;
pub const PCI_DEV_BRIDGE_VT8235: u16 = 0x3177;
pub const PCI_DEV_BRIDGE_VT8237: u16 = 0x3227;
pub const PCI_DEV_BRIDGE_VT8237A: u16 = 0x3337;
pub const PCI_DEV_BRIDGE_VT8237S: u16 = 0x3372;
pub const PCI_DEV_BRIDGE_VT8251: u16 = 0x3287;
pub const PCI_DEV_BRIDGE_VT8261: u16 = 0x3402;

pub const HW_FLAGS_TYPE_MASK: u16 = 0x000F;
pub const HW_FLAGS_CHECK_BRIDGE: u16 = 0x0010;
pub const HW_FLAGS_HAS_UDMA_CLOCK: u16 = 0x0020;
pub const HW_FLAGS_SINGLE_CHAN: u16 = 0x0040;
pub const HW_FLAGS_SINGLE_PORT: u16 = 0x0080;
pub const HW_FLAGS_PCI_SCR: u16 = 0x0100;

pub const TYPE_33: u16 = 0;
pub const TYPE_66: u16 = 1;
pub const TYPE_100: u16 = 2;
pub const TYPE_133: u16 = 3;
pub const TYPE_MWDMA: u16 = 4;
pub const TYPE_SATA: u16 = 5;

pub const VIA_PCI_CLOCK: u32 = 30000;

pub const VIA_REG_ENABLE_CTRL: u8 = 0x40;
pub const VIA_REG_DRIVE_TIMING: u8 = 0x48;
pub const VIA_REG_ADDRESS_SETUP: u8 = 0x4C;
pub const VIA_REG_UDMA_CTRL: u8 = 0x50;
pub const VIA_REG_SSTATUS: u8 = 0xA0;
pub const VIA_REG_SCONTROL: u8 = 0xA4;
pub const VIA_REG_SERROR: u8 = 0xA8;
pub const VIA_REG_SERROR_VT8237: u8 = 0xB0;

pub const VIA_UDMA_CLOCK_UDMA66: u8 = 0x08;
pub const VIA_UDMA_CABLE_BITS: u8 = 0x10;
pub const VIA_UDMA_CLEAR_MASK: u8 = 0xEF;

pub const VIA_REG_SATA_PORT_MAP: u8 = 0x49;
pub const VT6421A_REG_SATA_CTRL: u8 = 0x52;
pub const VT6421A_UDMA_SLOW: u8 = 0x0F;
pub const VT6421A_FIFO_WATERMARK_64DW: u8 = 0x04;

#[repr(C)]
pub struct ViaControllerEntry {
    pub DeviceID: u16,
    pub Flags: u16,
}

#[repr(C)]
pub struct ViaUdmaTimingEntry {
    pub DefaultValue: u8,
    pub Data: [u8; 7],
}
#[no_mangle]
pub unsafe extern "C" fn ViaClampTimings(Timing: PATA_TIMING) {
    (*Timing).AddressSetup = (CLAMP_TIMING((*Timing).AddressSetup, 1, 4) as i32 - 1) as ATATIM;
    (*Timing).CmdRecovery = (CLAMP_TIMING((*Timing).CmdRecovery, 1, 16) as i32 - 1) as ATATIM;
    (*Timing).CmdActive = (CLAMP_TIMING((*Timing).CmdActive, 1, 16) as i32 - 1) as ATATIM;
    (*Timing).DataRecovery = (CLAMP_TIMING((*Timing).DataRecovery, 1, 16) as i32 - 1) as ATATIM;
    (*Timing).DataActive = (CLAMP_TIMING((*Timing).DataActive, 1, 16) as i32 - 1) as ATATIM;
}

unsafe extern "C" fn ViaSetTransferMode(mut Controller: PATA_CONTROLLER, mut Channel: ULONG, mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    let mut ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[Channel as usize] as PCHANNEL_DATA_PATA;
    let mut DeviceTimings: [ATA_TIMING; (MAX_IDE_DEVICE as usize)] = [ATA_TIMING {
        AddressSetup: 0,
        CmdActive: 0,
        CmdRecovery: 0,
        DataActive: 0,
        DataRecovery: 0,
    }; (MAX_IDE_DEVICE as usize)];
    let mut i: ULONG = 0;
    let mut DriveTimReg: ULONG = 0;
    let mut UdmaTimReg: ULONG = 0;
    let mut PortTimReg: UCHAR = 0;

    AtaSelectTimings(
        DeviceList,
        &raw mut DeviceTimings as PATA_TIMING,
        VIA_PCI_CLOCK,
        SHARED_CMD_TIMINGS,
    );

    INFO!(
        b"CH %lu: Config (before)\nDRV  %08lX\nPORT %08lX\nUDMA %08lX\nCFG  %08lX\n\0".as_ptr() as *const i8,
        Channel,
        PciRead32(Controller, 0x48),
        PciRead32(Controller, 0x4C),
        PciRead32(Controller, 0x50),
        PciRead32(Controller, 0x40),
    );

    DriveTimReg = PciRead32(Controller, (VIA_REG_DRIVE_TIMING) as ULONG);
    PortTimReg = PciRead8(Controller, VIA_REG_PORT_TIMING!(Channel));

    if GET_TYPE!((*ChanData).HwFlags) != (TYPE_MWDMA as u32) {
        UdmaTimReg = PciRead32(Controller, (VIA_REG_UDMA_CTRL) as ULONG);
    }

    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let mut Timing: PATA_TIMING = (&raw mut DeviceTimings as *mut ATA_TIMING).offset(i as isize) as PATA_TIMING;
        let DeviceIndex: ULONG = (Channel << 1).wrapping_add(i);
        let Shift: ULONG = (3u32).wrapping_sub(DeviceIndex) << 3;
        let mut Value: ULONG = 0;

        if GET_TYPE!((*ChanData).HwFlags) != (TYPE_MWDMA as u32) {
            if !Device.is_null() && (*Device).DmaMode >= UDMA_MODE!(0) {
                let mut ModeIndex: ULONG = (*Device).DmaMode.wrapping_sub(UDMA_MODE!(0));
                Value = ViaUdmaTimings[GET_TYPE!((*ChanData).HwFlags) as usize].Data[ModeIndex as usize] as ULONG;
            } else {
                Value = ViaUdmaTimings[GET_TYPE!((*ChanData).HwFlags) as usize].DefaultValue as ULONG;
            }

            if i != 0 && GET_TYPE!((*ChanData).HwFlags) == (TYPE_66 as u32) {
                Value |= (VIA_UDMA_CLOCK_UDMA66 as u32);
            }

            UdmaTimReg &= !(VIA_UDMA_CLEAR_MASK << Shift) as ULONG;
            UdmaTimReg |= Value << Shift;
        }

        if Device.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        ViaClampTimings(Timing);

        Value = (((*Timing).DataActive << 4) | (*Timing).DataRecovery) as ULONG;
        DriveTimReg &= !(0xFF << Shift);
        DriveTimReg |= Value << Shift;

        PortTimReg = (((*Timing).CmdActive << 4) | (*Timing).CmdRecovery) as u8;

        i = i.wrapping_add(1);
    }

    PciWrite8(Controller, VIA_REG_PORT_TIMING!(Channel), PortTimReg);
    PciWrite32(Controller, (VIA_REG_DRIVE_TIMING) as ULONG, DriveTimReg);

    if GET_TYPE!((*ChanData).HwFlags) != (TYPE_MWDMA as u32) {
        PciWrite32(Controller, (VIA_REG_UDMA_CTRL) as ULONG, UdmaTimReg);
    }

    INFO!(
        b"CH %lu: Config (after)\nDRV  %08lX\nPORT %08lX\nUDMA %08lX\nCFG  %08lX\n\0".as_ptr() as *const i8,
        Channel,
        PciRead32(Controller, 0x48),
        PciRead32(Controller, 0x4C),
        PciRead32(Controller, 0x50),
        PciRead32(Controller, 0x40),
    );
}

unsafe extern "C" fn ViaSataSetTransferMode(mut Controller: PATA_CONTROLLER, mut Channel: ULONG, mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    let mut SataControl: UCHAR = PciRead8(Controller, (VT6421A_REG_SATA_CTRL) as ULONG);
    let mut i: ULONG = 0;
    if SataControl as ::core::ffi::c_int & (VT6421A_FIFO_WATERMARK_64DW as i32) == 0 {
        i = 0;
        while i < MAX_IDE_DEVICE {
            let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
            if !Device.is_null() {
                if _strnicmp(
                    (*Device).FriendlyName as *const ::core::ffi::c_char,
                    b"WD\0" as *const u8 as *const ::core::ffi::c_char,
                    2,
                ) == 0
                {
                    WARN!(
                        b"Enabling workaround for the '%s' drive\n\0".as_ptr() as *const i8,
                        (*Device).FriendlyName,
                    );
                    SataControl =
                        (SataControl as ::core::ffi::c_int | (VT6421A_FIFO_WATERMARK_64DW as i32)) as UCHAR;
                    PciWrite8(Controller, (VT6421A_REG_SATA_CTRL) as ULONG, SataControl);
                    break;
                }
            }
            i = i.wrapping_add(1);
        }
    }
    SataSetTransferMode(Controller as PATA_CONTROLLER, Channel, DeviceList);
}

unsafe extern "C" fn Via6421SetTransferMode(mut Controller: PATA_CONTROLLER, mut Channel: ULONG, mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    let mut DeviceTimings: [ATA_TIMING; (MAX_IDE_DEVICE as usize)] = [ATA_TIMING {
        AddressSetup: 0,
        CmdActive: 0,
        CmdRecovery: 0,
        DataActive: 0,
        DataRecovery: 0,
    }; (MAX_IDE_DEVICE as usize)];
    let mut i: ULONG = 0;

    UNREFERENCED_PARAMETER!(Channel);

    AtaSelectTimings(
        DeviceList,
        &raw mut DeviceTimings as PATA_TIMING,
        VIA_PCI_CLOCK,
        SHARED_CMD_TIMINGS,
    );

    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let mut Timing: PATA_TIMING =
            (&raw mut DeviceTimings as *mut ATA_TIMING).offset(i as isize) as PATA_TIMING;
        let mut UdmaTimReg: UCHAR = 0;

        UdmaTimReg = PciRead8(Controller, VT6421A_REG_UDMA_CTRL!(i));
        UdmaTimReg = (UdmaTimReg as ::core::ffi::c_int & (VIA_UDMA_CABLE_BITS as i32)) as UCHAR;
        if !Device.is_null()
            && (*Device).DmaMode >= UDMA_MODE!(0)
        {
            UdmaTimReg = (UdmaTimReg as ::core::ffi::c_int
                | ViaUdmaTimings[(TYPE_133) as usize].Data[(*Device)
                    .DmaMode
                    .wrapping_sub(UDMA_MODE!(0))
                    as usize] as ::core::ffi::c_int) as UCHAR;
        } else {
            UdmaTimReg = (UdmaTimReg as ::core::ffi::c_int | (VT6421A_UDMA_SLOW as i32)) as UCHAR;
        }
        PciWrite8(Controller, VT6421A_REG_UDMA_CTRL!(i), UdmaTimReg);

        if !Device.is_null() {
            ViaClampTimings(Timing);

            PciWrite8(
                Controller,
                VT6421A_REG_TIMING_CTRL!(i),
                (((*Timing).DataActive as ::core::ffi::c_int) << 4
                    | (*Timing).DataRecovery as ::core::ffi::c_int) as UCHAR,
            );
        }

        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn ViaGetSerrOffset(Controller: PATA_CONTROLLER) -> ULONG {
    if (*Controller).Pci.DeviceID == PCI_DEV_VT8237S_SATA_RAID {
        return (VIA_REG_SERROR_VT8237) as ULONG;
    }
    return (VIA_REG_SERROR) as ULONG;
}

unsafe extern "C" fn ViaScrReadPci(ChanData: PCHANNEL_DATA_PATA, Register: SATA_SCR_REGISTER, PortNumber: ULONG, Result: PULONG) -> BOOLEAN {
    let Controller: PATA_CONTROLLER = (*ChanData).__bindgen_anon_1.Controller;
    let mut Value: ULONG;
    let Offset: ULONG = 2 * (*ChanData).__bindgen_anon_1.Channel + PortNumber;
    let Reg: UCHAR;

    match Register {
        ATA_SSTATUS => {
            Reg = PciRead8(Controller, (VIA_REG_SSTATUS + (Offset) as u8) as ULONG);

            Value = (Reg & 0x03) as ULONG;

            if Reg & 0x04 != 0 {
                Value |= AHCI_PXSSTS_IPM_PARTIAL;
            } else if Reg & 0x08 != 0 {
                Value |= AHCI_PXSSTS_IPM_SLUMBER;
            } else {
                Value |= AHCI_PXSSTS_IPM_ACTIVE;
            }

            if Reg & 0x10 != 0 {
                Value |= AHCI_PXSSTS_SPD_SATA2;
            } else {
                Value |= AHCI_PXSSTS_SPD_SATA1;
            }
        }
        ATA_SERROR => {
            Value = PciRead32(
                Controller,
                ViaGetSerrOffset(Controller) + Offset * 4,
            );
        }
        ATA_SCONTROL => {
            Value = 0;
            Reg = PciRead8(Controller, (VIA_REG_SCONTROL + (Offset) as u8) as ULONG);
            if Reg & 0x01 != 0 {
                Value |= AHCI_PXCTL_DET_RESET;
            }
            if Reg & 0x02 != 0 {
                Value |= AHCI_PXCTL_DET_DISABLE_SATA;
            }
            if Reg & 0x04 != 0 {
                Value |= AHCI_PXCTL_IPM_DISABLE_PARTIAL;
            }
            if Reg & 0x08 != 0 {
                Value |= AHCI_PXCTL_IPM_DISABLE_SLUMBER;
            }
        }
        _ => {
            return FALSE;
        }
    }

    *Result = Value;
    return TRUE;
}

unsafe extern "C" fn ViaScrWritePci(mut ChanData: PCHANNEL_DATA_PATA, mut Register: SATA_SCR_REGISTER, mut PortNumber: ULONG, mut Value: ULONG) -> BOOLEAN {
    let mut Controller: PATA_CONTROLLER = (*ChanData).__bindgen_anon_1.Controller;
    let mut Offset: ULONG = (2 as ULONG)
        .wrapping_mul((*ChanData).__bindgen_anon_1.Channel)
        .wrapping_add(PortNumber);
    match Register as ::core::ffi::c_uint {
        ATA_SERROR => {
            PciWrite32(
                Controller,
                ViaGetSerrOffset(Controller)
                    .wrapping_add(Offset.wrapping_mul(4)),
                Value,
            );
        }
        ATA_SCONTROL => {
            let mut Val: UCHAR = 0;
            if Value & AHCI_PXCTL_DET_RESET != 0 {
                Val = (Val as ::core::ffi::c_int | 0x01 as ::core::ffi::c_int)
                    as UCHAR;
            }
            if Value & AHCI_PXCTL_DET_DISABLE_SATA != 0 {
                Val = (Val as ::core::ffi::c_int | 0x02 as ::core::ffi::c_int)
                    as UCHAR;
            }
            if Value & AHCI_PXCTL_IPM_DISABLE_PARTIAL != 0 {
                Val = (Val as ::core::ffi::c_int | 0x04 as ::core::ffi::c_int)
                    as UCHAR;
            }
            if Value & AHCI_PXCTL_IPM_DISABLE_SLUMBER != 0 {
                Val = (Val as ::core::ffi::c_int | 0x08 as ::core::ffi::c_int)
                    as UCHAR;
            }
            PciWrite8(
                Controller,
                (VIA_REG_SCONTROL.wrapping_add(Offset as UCHAR)) as ULONG,
                Val,
            );
        }
        _ => return FALSE,
    }
    return TRUE;
}

unsafe extern "C" fn ViaScrReadIoPort(ChanData: PCHANNEL_DATA_PATA, Register: SATA_SCR_REGISTER, PortNumber: ULONG, Result: PULONG) -> BOOLEAN {
    UNREFERENCED_PARAMETER!(PortNumber);

    if Register as u32 > ATA_SCONTROL {
        return FALSE;
    }

    *Result = READ_PORT_ULONG(
        (*ChanData).Regs.Scr.offset((Register as u32).wrapping_mul(4) as isize) as PULONG
    ) as u32;
    return TRUE;
}

unsafe extern "C" fn ViaScrWriteIoPort(ChanData: PCHANNEL_DATA_PATA, Register: SATA_SCR_REGISTER, PortNumber: ULONG, Value: ULONG) -> BOOLEAN {
    UNREFERENCED_PARAMETER!(PortNumber);

    if Register as u32 > ATA_SCONTROL {
        return FALSE;
    }

    WRITE_PORT_ULONG(
        (*ChanData).Regs.Scr.offset((Register as u32).wrapping_mul(4) as isize) as PULONG,
        Value,
    );
    return TRUE;
}

unsafe extern "C" fn Via6421ParseResources(
    ChanData: PCHANNEL_DATA_PATA,
    PortIndex: ULONG,
) -> NTSTATUS {
    let mut CommandPortBase: PUCHAR = core::ptr::null_mut::<core::ffi::c_uchar>();
    let mut ControlPortBase: PUCHAR = core::ptr::null_mut::<core::ffi::c_uchar>();
    let mut DmaBase: PUCHAR = core::ptr::null_mut::<core::ffi::c_uchar>();
    let mut ScrBase: PUCHAR = core::ptr::null_mut::<core::ffi::c_uchar>();

    PAGED_CODE!();

    CommandPortBase = AtaCtrlPciMapBar(
        (*ChanData).__bindgen_anon_1.Controller,
        PortIndex,
        16,
    ) as PUCHAR;
    if CommandPortBase.is_null() {
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    DmaBase = AtaCtrlPciMapBar(
        (*ChanData).__bindgen_anon_1.Controller,
        4,
        32,
    ) as PUCHAR;
    if DmaBase.is_null() {
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    ScrBase = AtaCtrlPciMapBar(
        (*ChanData).__bindgen_anon_1.Controller,
        5,
        128,
    ) as PUCHAR;
    if ScrBase.is_null() {
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }

    (*ChanData).Regs.Dma = DmaBase.offset(PortIndex.wrapping_mul(8) as isize);
    (*ChanData).Regs.Scr = ScrBase.offset(PortIndex.wrapping_mul(64) as isize);

    ControlPortBase = CommandPortBase.offset(PCIIDE_COMMAND_IO_RANGE_LENGTH as isize);

    PciIdeInitTaskFileIoResources(
        ChanData,
        (CommandPortBase as ULONG_PTR) as _,
        ((ControlPortBase as ULONG_PTR).wrapping_add(PCIIDE_CONTROL_IO_BAR_OFFSET as ULONG_PTR)) as _,
        1,
    );

    return STATUS_SUCCESS;
}

unsafe extern "C" fn Via6421GetControllerProperties(mut Controller: PATA_CONTROLLER) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut i: ULONG = 0;

    PAGED_CODE!();

    (*Controller).MaxChannels = 3;
    (*Controller).Flags |= CTRL_FLAG_MANUAL_RES;

    Status = PciIdeCreateChannelData(Controller, 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    i = 0;
    while i < (*Controller).MaxChannels {
        let mut ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[i as usize] as PCHANNEL_DATA_PATA;

        if i == 2 {
            (*ChanData).__bindgen_anon_1.TransferModeSupported = PIO_ALL | UDMA_ALL;
            (*ChanData).__bindgen_anon_1.SetTransferMode = Some(
                Via6421SetTransferMode
                    as unsafe extern "C" fn(
                        PATA_CONTROLLER,
                        ULONG,
                        *mut PCHANNEL_DEVICE_CONFIG,
                    ) -> (),
            ) as PCHANNEL_SET_MODE_EX;

            if !VT6421A_UDMA_CABLE_PRESENT!(Controller) {
                INFO!("CH %lu: BIOS detected 40-conductor cable\n", (*ChanData).Channel);
                (*ChanData).__bindgen_anon_1.TransferModeSupported &= !UDMA_80C_ALL;
            }
        } else {
            (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_NO_SLAVE;
            (*ChanData).__bindgen_anon_1.SetTransferMode = Some(
                ViaSataSetTransferMode
                    as unsafe extern "C" fn(
                        PATA_CONTROLLER,
                        ULONG,
                        *mut PCHANNEL_DEVICE_CONFIG,
                    ) -> (),
            ) as PCHANNEL_SET_MODE_EX;
            (*ChanData).__bindgen_anon_1.TransferModeSupported = SATA_ALL;
            (*ChanData).ScrRead = Some(
                ViaScrReadIoPort
                    as unsafe extern "C" fn(
                        PCHANNEL_DATA_PATA,
                        SATA_SCR_REGISTER,
                        ULONG,
                        PULONG,
                    ) -> BOOLEAN,
            ) as PCHANNEL_READ_SCR;
            (*ChanData).ScrWrite = Some(
                ViaScrWriteIoPort
                    as unsafe extern "C" fn(
                        PCHANNEL_DATA_PATA,
                        SATA_SCR_REGISTER,
                        ULONG,
                        ULONG,
                    ) -> BOOLEAN,
            ) as PCHANNEL_WRITE_SCR;
        }

        Status = Via6421ParseResources(ChanData, i);
        if !NT_SUCCESS!(Status) {
            return Status;
        }

        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn ViaCx700GetControllerProperties(
    mut Controller: PATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut i: ULONG = 0;

    PAGED_CODE!();

    Status = PciIdeCreateChannelData(Controller, 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    (*Controller).__bindgen_anon_1.ChannelEnableBits = &ViaCx700EnableBits as *const ATA_PCI_ENABLE_BITS;

    i = 0;
    while i < (*Controller).MaxChannels {
        let mut ChanData: PCHANNEL_DATA_PATA =
            (*Controller).Channels[i as usize] as PCHANNEL_DATA_PATA;

        if i == 0 {
            (*ChanData).__bindgen_anon_1.SetTransferMode = Some(
                SataSetTransferMode
                    as unsafe extern "C" fn(
                        PATA_CONTROLLER,
                        ULONG,
                        *mut PCHANNEL_DEVICE_CONFIG,
                    ) -> (),
            ) as PCHANNEL_SET_MODE_EX;
            (*ChanData).__bindgen_anon_1.TransferModeSupported = SATA_ALL;
            (*ChanData).ScrRead = Some(
                ViaScrReadPci
                    as unsafe extern "C" fn(
                        PCHANNEL_DATA_PATA,
                        SATA_SCR_REGISTER,
                        ULONG,
                        PULONG,
                    ) -> BOOLEAN,
            ) as PCHANNEL_READ_SCR;
            (*ChanData).ScrWrite = Some(
                ViaScrWritePci
                    as unsafe extern "C" fn(
                        PCHANNEL_DATA_PATA,
                        SATA_SCR_REGISTER,
                        ULONG,
                        ULONG,
                    ) -> BOOLEAN,
            ) as PCHANNEL_WRITE_SCR;
        } else {
            (*ChanData).HwFlags |= (TYPE_133 as u32);
            (*ChanData).__bindgen_anon_1.SetTransferMode = Some(
                ViaSetTransferMode
                    as unsafe extern "C" fn(
                        PATA_CONTROLLER,
                        ULONG,
                        *mut PCHANNEL_DEVICE_CONFIG,
                    ) -> (),
            ) as PCHANNEL_SET_MODE_EX;
            (*ChanData).__bindgen_anon_1.TransferModeSupported =
                PIO_ALL | MWDMA_ALL | UDMA_ALL;

            if !VIA_SINGLE_CHAN_UDMA_CABLE_PRESENT!(Controller) {
                INFO!(
                    b"CH %lu: BIOS detected 40-conductor cable\n\0" as *const u8 as *const i8,
                    (*ChanData).Channel,
                );
                (*ChanData).__bindgen_anon_1.TransferModeSupported &= !UDMA_80C_ALL;
            }
        }

        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn Via6410ControllerStart(Controller: PATA_CONTROLLER) {
    let mut Value: UCHAR;

    Value = PciRead8(Controller, (VIA_REG_ENABLE_CTRL) as ULONG);
    Value |= 0x03;
    PciWrite8(Controller, (VIA_REG_ENABLE_CTRL) as ULONG, Value);
}

unsafe extern "C" fn ViaQuerySouthBridgeInformation(mut Context: PVOID, mut BusNumber: ULONG, mut PciSlot: PCI_SLOT_NUMBER, mut PciConfig: PPCI_COMMON_HEADER) -> BOOLEAN {
    let mut HwFlags: PULONG = Context as PULONG;
    let mut i: ULONG = 0;
    #[link_section = "PAGECONS"]
    static mut ViaBridgeList: [C2RustUnnamed_163; 16] = [
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT82C586 as USHORT,
            MinimumRevisionID: 0x20 as UCHAR,
            Flags: TYPE_33 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT82C586 as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_MWDMA as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT82C596A as USHORT,
            MinimumRevisionID: 0x10 as UCHAR,
            Flags: (TYPE_66 | HW_FLAGS_HAS_UDMA_CLOCK) as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT82C596A as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_33 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT82C686A as USHORT,
            MinimumRevisionID: 0x40 as UCHAR,
            Flags: TYPE_100 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT82C686A as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: (TYPE_66 | HW_FLAGS_HAS_UDMA_CLOCK) as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT8231 as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_100 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT8233 as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_100 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT8233A as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_133 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT8233C as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_100 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT8235 as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_133 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT8237 as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_133 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT8237A as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_133 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT8237S as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_133 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT8251 as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_133 as UCHAR,
        },
        C2RustUnnamed_163 {
            DeviceID: PCI_DEV_BRIDGE_VT8261 as USHORT,
            MinimumRevisionID: 0 as UCHAR,
            Flags: TYPE_133 as UCHAR,
        },
    ];
    UNREFERENCED_PARAMETER!(BusNumber);
    UNREFERENCED_PARAMETER!(PciSlot);
    PAGED_CODE!();
    if (*PciConfig).VendorID as ::core::ffi::c_int != (PCI_VEN_VIA as i32) {
        return FALSE as BOOLEAN;
    }
    i = 0 as ULONG;
    while (i as usize)
        < (::core::mem::size_of::<[C2RustUnnamed_163; 16]>() as usize)
            .wrapping_div(::core::mem::size_of::<C2RustUnnamed_163>() as usize)
    {
        if (*PciConfig).DeviceID as ::core::ffi::c_int
            == ViaBridgeList[i as usize].DeviceID as ::core::ffi::c_int
            && (*PciConfig).RevisionID as ::core::ffi::c_int
                >= ViaBridgeList[i as usize].MinimumRevisionID as ::core::ffi::c_int
        {
            INFO!(
                b"Found %04X:%04X.%02X VIA bridge\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*PciConfig).VendorID,
                (*PciConfig).DeviceID,
                (*PciConfig).RevisionID,
            );
            *HwFlags = ViaBridgeList[i as usize].Flags as ::core::ffi::c_ulong;
            return TRUE as BOOLEAN;
        }
        i = i.wrapping_add(1);
    }
    return FALSE as BOOLEAN;
}

unsafe extern "C" fn ViaPataGetControllerProperties(
    mut Controller: PATA_CONTROLLER,
    mut HwFlags: ULONG,
) -> NTSTATUS {
    let mut i: ULONG = 0;
    let mut UdmaTimReg: ULONG = 0;

    PAGED_CODE!();

    if HwFlags & (HW_FLAGS_CHECK_BRIDGE as u32) != 0 {
        if ((PciFindDevice(Some(ViaQuerySouthBridgeInformation), (&raw mut HwFlags).cast())) == 0) {
            ERR!(b"Unable to find the VIA bridge\n\0".as_ptr() as *const i8);
            return STATUS_NO_MATCH;
        }
    }

    if (*Controller).Pci.DeviceID == PCI_DEV_VT6410_IDE_RAID {
        (*Controller).Start = Some(Via6410ControllerStart);
    }

    if (*Controller).Pci.DeviceID != PCI_DEV_VT6415_IDE {
        (*Controller).__bindgen_anon_1.ChannelEnableBits = ViaPataEnableBits.as_ptr();
    }

    if GET_TYPE!(HwFlags) != (TYPE_MWDMA as u32) {
        UdmaTimReg = PciRead32(Controller, (VIA_REG_UDMA_CTRL) as ULONG);
    }

    i = 0;
    while i < (*Controller).MaxChannels {
        let ChanData: PCHANNEL_DATA_PATA = ((*Controller).Channels[i as usize]).cast();

        (*ChanData).HwFlags = HwFlags;
        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(ViaSetTransferMode);

        match (GET_TYPE!((*ChanData).HwFlags)) as u16 {
            TYPE_MWDMA => {
                (*ChanData).__bindgen_anon_1.TransferModeSupported = PIO_ALL | MWDMA_ALL;
            }
            TYPE_33 => {
                (*ChanData).__bindgen_anon_1.TransferModeSupported = PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 2);
            }
            TYPE_66 => {
                (*ChanData).__bindgen_anon_1.TransferModeSupported = PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 4);

                if !VIA_UDMA_CLOCK_ENABLED!(UdmaTimReg, i) {
                    INFO!(
                        b"CH %lu: BIOS detected 40-conductor cable or this chip is ATA/33 capable\n\0"
                            .as_ptr() as *const i8,
                        (*ChanData).Channel,
                    );
                    (*ChanData).__bindgen_anon_1.TransferModeSupported &= !UDMA_80C_ALL;
                    (*ChanData).HwFlags &= !(HW_FLAGS_TYPE_MASK as u32);
                    (*ChanData).HwFlags |= (TYPE_33 as u32);
                }
            }
            TYPE_100 => {
                (*ChanData).__bindgen_anon_1.TransferModeSupported = PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 5);

                if !VIA_UDMA_CABLE_PRESENT!(UdmaTimReg, i) {
                    INFO!(
                        b"CH %lu: BIOS detected 40-conductor cable\n\0".as_ptr() as *const i8,
                        (*ChanData).Channel,
                    );
                    (*ChanData).__bindgen_anon_1.TransferModeSupported &= !UDMA_80C_ALL;
                }
            }
            TYPE_133 => {
                (*ChanData).__bindgen_anon_1.TransferModeSupported = PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 6);

                if !VIA_UDMA_CABLE_PRESENT!(UdmaTimReg, i) {
                    INFO!(
                        b"CH %lu: BIOS detected 40-conductor cable\n\0".as_ptr() as *const i8,
                        (*ChanData).Channel,
                    );
                    (*ChanData).__bindgen_anon_1.TransferModeSupported &= !UDMA_80C_ALL;
                }
            }
            _ => {
                ASSERT!(FALSE);
                UNREACHABLE!();
            }
        }

        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn ViaSataGetControllerProperties(
    mut Controller: PATA_CONTROLLER,
    mut HwFlags: ULONG,
) -> NTSTATUS {
    let mut i: ULONG = 0;
    PAGED_CODE!();
    INFO!(
        b"Port map: %02X\n\0" as *const u8 as *const i8,
        PciRead8(Controller, VIA_REG_SATA_PORT_MAP),
    );
    i = 0;
    while i < (*Controller).MaxChannels {
        let mut ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[i as usize] as PCHANNEL_DATA_PATA;
        (*ChanData).HwFlags = HwFlags;
        (*ChanData).__bindgen_anon_1.TransferModeSupported = SATA_ALL;
        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(
            SataSetTransferMode
                as unsafe extern "C" fn(PATA_CONTROLLER, ULONG, *mut PCHANNEL_DEVICE_CONFIG) -> (),
        ) as PCHANNEL_SET_MODE_EX;
        if HwFlags & (HW_FLAGS_SINGLE_PORT as u32) != 0 {
            (*ChanData).__bindgen_anon_1.ChanInfo |= CHANNEL_FLAG_NO_SLAVE;
        }
        if HwFlags & (HW_FLAGS_PCI_SCR as u32) != 0 {
            (*ChanData).ScrRead = Some(
                ViaScrReadPci
                    as unsafe extern "C" fn(
                        PCHANNEL_DATA_PATA,
                        SATA_SCR_REGISTER,
                        ULONG,
                        PULONG,
                    ) -> BOOLEAN,
            ) as PCHANNEL_READ_SCR;
            (*ChanData).ScrWrite = Some(
                ViaScrWritePci
                    as unsafe extern "C" fn(
                        PCHANNEL_DATA_PATA,
                        SATA_SCR_REGISTER,
                        ULONG,
                        ULONG,
                    ) -> BOOLEAN,
            ) as PCHANNEL_WRITE_SCR;
        }
        i = i.wrapping_add(1);
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn ViaGetControllerProperties(
    Controller: PATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;
    let mut HwFlags: ULONG = 0;

    PAGED_CODE!();

    if (*Controller).Pci.DeviceID == PCI_DEV_VT6421A_RAID {
        return Via6421GetControllerProperties(Controller);
    } else if (*Controller).Pci.DeviceID == PCI_DEV_CX700M2_IDE_SATA
        || (*Controller).Pci.DeviceID == PCI_DEV_CX700M2_IDE_SATA_RAID
    {
        return ViaCx700GetControllerProperties(Controller);
    }

    i = 0;
    while i < RTL_NUMBER_OF!(ViaControllerList) {
        HwFlags = ViaControllerList[i as usize].Flags as ULONG;
        if (*Controller).Pci.DeviceID == ViaControllerList[i as usize].DeviceID {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == RTL_NUMBER_OF!(ViaControllerList) {
        return STATUS_NO_MATCH;
    }

    if HwFlags & (HW_FLAGS_SINGLE_CHAN as u32) != 0 {
        (*Controller).MaxChannels = 1;
    }

    Status = PciIdeCreateChannelData(Controller, 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    if GET_TYPE!(HwFlags as ULONG) as u32 == (TYPE_SATA as u32) {
        return ViaSataGetControllerProperties(Controller, HwFlags);
    }

    return ViaPataGetControllerProperties(Controller, HwFlags);
}
