use crate::prelude::*;
use ntkrnl_sys::ddk::{Channel, Timing};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut AmdControllerList: [C2RustUnnamed_158; 35] = [
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_AMD as USHORT,
        DeviceID: PCI_DEV_AMD_756 as USHORT,
        Flags: HW_FLAGS_UDMA4 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_AMD as USHORT,
        DeviceID: PCI_DEV_AMD_766 as USHORT,
        Flags: (HW_FLAGS_UDMA5 | HW_FLAGS_NO_PREFETCH) as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_AMD as USHORT,
        DeviceID: PCI_DEV_AMD_768 as USHORT,
        Flags: HW_FLAGS_UDMA5 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_AMD as USHORT,
        DeviceID: PCI_DEV_AMD_8111 as USHORT,
        Flags: HW_FLAGS_CHECK_SYSBOARD as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_AMD as USHORT,
        DeviceID: PCI_DEV_AMD_CS5536 as USHORT,
        Flags: HW_FLAGS_UDMA5 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_AMD as USHORT,
        DeviceID: PCI_DEV_AMD_CS5536_2 as USHORT,
        Flags: HW_FLAGS_UDMA5 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_NFORCE_IDE as USHORT,
        Flags: HW_FLAGS_UDMA5 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_NFORCE2_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_NFORCE2_IDE_2 as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_NFORCE3_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_NFORCE3_IDE_2 as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_CK804_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP04_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP51_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP55_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP61_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP65_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP67_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP73_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP77_IDE as USHORT,
        Flags: 0 as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_NFORCE2_SATA as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_NFORCE3_SATA as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_NFORCE3_SATA_2 as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_CK804_SATA as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_CK804_SATA_2 as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP04_SATA as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP04_SATA_2 as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP51_SATA as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP51_SATA_2 as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP55_SATA as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP55_SATA_2 as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP61_SATA as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP61_SATA_2 as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP61_SATA_3 as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
    C2RustUnnamed_158 {
        VendorID: PCI_VEN_NVIDIA as USHORT,
        DeviceID: PCI_DEV_MCP89_SATA as USHORT,
        Flags: HW_FLAGS_SATA as USHORT,
    },
];

static mut AmdUdmaSettings: [UCHAR; 7] = [
    (AMD_UDMA_EN | (2 as ::core::ffi::c_int) as u32) as UCHAR,
    (AMD_UDMA_EN | (1 as ::core::ffi::c_int) as u32) as UCHAR,
    (AMD_UDMA_EN | (0 as ::core::ffi::c_int) as u32) as UCHAR,
    (AMD_UDMA_EN | (4 as ::core::ffi::c_int) as u32) as UCHAR,
    (AMD_UDMA_EN | (5 as ::core::ffi::c_int) as u32) as UCHAR,
    (AMD_UDMA_EN | (6 as ::core::ffi::c_int) as u32) as UCHAR,
    (AMD_UDMA_EN | (7 as ::core::ffi::c_int) as u32) as UCHAR,
];

static mut AmdTimingControlShift: [[ULONG; 2]; 2] = [
    [
        24 as ::core::ffi::c_int as ULONG,
        16 as ::core::ffi::c_int as ULONG,
    ],
    [
        8 as ::core::ffi::c_int as ULONG,
        0 as ::core::ffi::c_int as ULONG,
    ],
];

static mut AmdAddressSetupShift: [[ULONG; 2]; 2] = [
    [
        6 as ::core::ffi::c_int as ULONG,
        4 as ::core::ffi::c_int as ULONG,
    ],
    [
        2 as ::core::ffi::c_int as ULONG,
        0 as ::core::ffi::c_int as ULONG,
    ],
];

static mut AmdPortTimShift: [ULONG; 2] = [
    24 as ::core::ffi::c_int as ULONG,
    16 as ::core::ffi::c_int as ULONG,
];

static mut AmdEnableBits: [ATA_PCI_ENABLE_BITS; 2] = [
    _ATA_PCI_ENABLE_BITS {
        Register: AMD_CONFIG_BASE as UCHAR,
        Mask: 0x2 as UCHAR,
        ValueEnabled: 0x2 as UCHAR,
    },
    _ATA_PCI_ENABLE_BITS {
        Register: AMD_CONFIG_BASE as UCHAR,
        Mask: 0x1 as UCHAR,
        ValueEnabled: 0x1 as UCHAR,
    },
];

static mut NvEnableBits: [ATA_PCI_ENABLE_BITS; 2] = [
    _ATA_PCI_ENABLE_BITS {
        Register: NV_CONFIG_BASE as UCHAR,
        Mask: 0x2 as UCHAR,
        ValueEnabled: 0x2 as UCHAR,
    },
    _ATA_PCI_ENABLE_BITS {
        Register: NV_CONFIG_BASE as UCHAR,
        Mask: 0x1 as UCHAR,
        ValueEnabled: 0x1 as UCHAR,
    },
];

pub const PCI_DEV_AMD_756: u16 = 0x7409;
pub const PCI_DEV_AMD_766: u16 = 0x7411;
pub const PCI_DEV_AMD_768: u16 = 0x7441;
pub const PCI_DEV_AMD_8111: u16 = 0x7469;
pub const PCI_DEV_AMD_CS5536: u16 = 0x2092;
pub const PCI_DEV_AMD_CS5536_2: u16 = 0x209A;
pub const PCI_DEV_NFORCE_IDE: u16 = 0x01BC;
pub const PCI_DEV_NFORCE2_IDE: u16 = 0x0065;
pub const PCI_DEV_NFORCE2_IDE_2: u16 = 0x0085;
pub const PCI_DEV_NFORCE3_IDE: u16 = 0x00D5;
pub const PCI_DEV_NFORCE3_IDE_2: u16 = 0x00E5;
pub const PCI_DEV_CK804_IDE: u16 = 0x0053;
pub const PCI_DEV_MCP04_IDE: u16 = 0x0035;
pub const PCI_DEV_MCP51_IDE: u16 = 0x0265;
pub const PCI_DEV_MCP55_IDE: u16 = 0x036E;
pub const PCI_DEV_MCP61_IDE: u16 = 0x03EC;
pub const PCI_DEV_MCP65_IDE: u16 = 0x0448;
pub const PCI_DEV_MCP67_IDE: u16 = 0x0560;
pub const PCI_DEV_MCP73_IDE: u16 = 0x056C;
pub const PCI_DEV_MCP77_IDE: u16 = 0x0759;
pub const PCI_DEV_NFORCE2_SATA: u16 = 0x008E;
pub const PCI_DEV_NFORCE3_SATA: u16 = 0x00E3;
pub const PCI_DEV_NFORCE3_SATA_2: u16 = 0x00EE;
pub const PCI_DEV_CK804_SATA: u16 = 0x0054;
pub const PCI_DEV_CK804_SATA_2: u16 = 0x0055;
pub const PCI_DEV_MCP04_SATA: u16 = 0x0036;
pub const PCI_DEV_MCP04_SATA_2: u16 = 0x003E;
pub const PCI_DEV_MCP51_SATA: u16 = 0x0266;
pub const PCI_DEV_MCP51_SATA_2: u16 = 0x0267;
pub const PCI_DEV_MCP55_SATA: u16 = 0x037E;
pub const PCI_DEV_MCP55_SATA_2: u16 = 0x037F;
pub const PCI_DEV_MCP61_SATA: u16 = 0x03E7;
pub const PCI_DEV_MCP61_SATA_2: u16 = 0x03F6;
pub const PCI_DEV_MCP61_SATA_3: u16 = 0x03F7;
pub const PCI_DEV_MCP89_SATA: u16 = 0x0D85;
pub const PCI_SUBSYSTEM_AMD_SERENADE: u16 = 0x36C0;
pub const AMD_CONFIG_BASE: u32 = 0x40;
pub const NV_CONFIG_BASE: u32 = 0x50;
pub const AMD_PCI_CLOCK: u32 = 30000;
pub const HW_FLAGS_UDMA4: u16 = 0x0001;
pub const HW_FLAGS_UDMA5: u16 = 0x0002;
pub const HW_FLAGS_CHECK_SYSBOARD: u16 = 0x0004;
pub const HW_FLAGS_NO_PREFETCH: u16 = 0x0008;
pub const HW_FLAGS_SATA: u16 = 0x0010;

#[repr(C)]
pub struct AmdControllerEntry {
    pub VendorID: u16,
    pub DeviceID: u16,
    pub Flags: u16,
}

unsafe extern "C" fn AmdGetPciConfigIoBase(Controller: PATA_CONTROLLER) -> ULONG {
    if ((*Controller).Pci.VendorID as ::core::ffi::c_int) as u32 == PCI_VEN_AMD {
        return AMD_CONFIG_BASE;
    }
    return NV_CONFIG_BASE;
}

unsafe extern "C" fn AmdEnablePostedWriteBuffer(
    mut Controller: PATA_CONTROLLER,
    mut ChanData: PCHANNEL_DATA_PATA,
    mut Channel: ULONG,
    mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG,
    mut IoBase: ULONG,
) {
    let mut ConfigReg: UCHAR = 0;
    let mut i: ULONG = 0;
    ConfigReg = PciRead8(Controller, AMD_REG_CONFIG_PREFETCH!(IoBase));
    if (*ChanData).HwFlags & (HW_FLAGS_NO_PREFETCH) as _ != 0 {
        ConfigReg = (ConfigReg as ::core::ffi::c_int
            & (!(AMD_CONFIG_PREFETCH!(0)
                | AMD_CONFIG_PREFETCH!(1))) as i32)
            as UCHAR;
    } else {
        ConfigReg = (ConfigReg as ::core::ffi::c_int | AMD_CONFIG_PREFETCH!(Channel)) as UCHAR;
        i = 0;
        while i < MAX_IDE_DEVICE {
            let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
            if !Device.is_null() && (*Device).IsFixedDisk == 0 {
                ConfigReg = (ConfigReg as ::core::ffi::c_int
                    & (!AMD_CONFIG_PREFETCH!(Channel)) as i32)
                    as UCHAR;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }
    PciWrite8(Controller, AMD_REG_CONFIG_PREFETCH!(IoBase), ConfigReg);
}

unsafe extern "C" fn AmdSetTransferMode(
    mut Controller: PATA_CONTROLLER,
    mut Channel: ULONG,
    mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG,
) {
    let mut ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[Channel as usize] as PCHANNEL_DATA_PATA;
    let IoBase: ULONG = AmdGetPciConfigIoBase(Controller);
    let mut DeviceTimings: [ATA_TIMING; (MAX_IDE_DEVICE) as _] = [ATA_TIMING {
        AddressSetup: 0,
        CmdActive: 0,
        CmdRecovery: 0,
        DataActive: 0,
        DataRecovery: 0,
    }; (MAX_IDE_DEVICE) as _];
    let mut i: ULONG;
    let mut DriveTimReg: ULONG;
    let mut PortTimReg: ULONG;
    let mut UdmaTimReg: ULONG;

    AtaSelectTimings(DeviceList, DeviceTimings.as_mut_ptr(), AMD_PCI_CLOCK, SHARED_CMD_TIMINGS);

    DriveTimReg = PciRead32(Controller, AMD_REG_TIMING_CTRL!(IoBase));
    PortTimReg = PciRead32(Controller, AMD_REG_ADDRESS_SETUP!(IoBase));
    UdmaTimReg = PciRead32(Controller, AMD_REG_UDMA!(IoBase, 1));

    INFO!(
        b"CH %lu: Config (before)\nDRV  %08lX\nPORT %08lX\nUDMA %08lX\nCFG  %08lX\n\0".as_ptr() as *const i8,
        Channel,
        DriveTimReg,
        PortTimReg,
        UdmaTimReg,
        PciRead32(Controller, IoBase),
    );

    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let mut Timing: PATA_TIMING = DeviceTimings.as_mut_ptr().offset(i as isize);
        let mut Value: ULONG;

        if !Device.is_null() && (*Device).DmaMode >= UDMA_MODE!(0) {
            Value = AmdUdmaSettings[(*Device).DmaMode.wrapping_sub(UDMA_MODE!(0)) as usize] as ULONG;
        } else {
            if (*Controller).Pci.VendorID as u32 == PCI_VEN_AMD {
                Value = AMD_UDMA_TIME!(3);
            } else {
                Value = 0;
            }
        }
        UdmaTimReg &= !((0xFF) << AmdTimingControlShift[Channel as usize][i as usize]);
        UdmaTimReg |= Value << AmdTimingControlShift[Channel as usize][i as usize];

        if Device.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        ViaClampTimings(Timing);

        Value = (*Timing).AddressSetup as ULONG;
        PortTimReg &= !((0x03) << AmdAddressSetupShift[Channel as usize][i as usize]);
        PortTimReg |= Value << AmdAddressSetupShift[Channel as usize][i as usize];

        Value = (((*Timing).CmdActive as u32) << 4) | ((*Timing).CmdRecovery as u32);
        PortTimReg &= !((0xFF) << AmdPortTimShift[Channel as usize]);
        PortTimReg |= Value << AmdPortTimShift[Channel as usize];

        Value = (((*Timing).DataActive as u32) << 4) | ((*Timing).DataRecovery as u32);
        DriveTimReg &= !((0xFF) << AmdTimingControlShift[Channel as usize][i as usize]);
        DriveTimReg |= Value << AmdTimingControlShift[Channel as usize][i as usize];

        i = i.wrapping_add(1);
    }

    AmdEnablePostedWriteBuffer(Controller, ChanData, Channel, DeviceList, IoBase);

    PciWrite32(Controller, AMD_REG_TIMING_CTRL!(IoBase), DriveTimReg);
    PciWrite32(Controller, AMD_REG_ADDRESS_SETUP!(IoBase), PortTimReg);
    PciWrite32(Controller, AMD_REG_UDMA!(IoBase, 1), UdmaTimReg);

    INFO!(
        b"CH %lu: Config (after)\nDRV  %08lX\nPORT %08lX\nUDMA %08lX\nCFG  %08lX\n\0".as_ptr() as *const i8,
        Channel,
        DriveTimReg,
        PortTimReg,
        UdmaTimReg,
        PciRead32(Controller, IoBase),
    );
}

unsafe extern "C" fn AmdControllerStart(mut Controller: PATA_CONTROLLER) {
    let mut ConfigReg: UCHAR;

    /* Initialize controller before issuing the identify command */
    ConfigReg = PciRead8(Controller, AMD_REG_CONFIG_PREFETCH!(AMD_CONFIG_BASE));
    ConfigReg &= (!(AMD_CONFIG_PREFETCH!(0) | AMD_CONFIG_PREFETCH!(1))) as u8;
    PciWrite8(Controller, AMD_REG_CONFIG_PREFETCH!(AMD_CONFIG_BASE), ConfigReg);
}

unsafe extern "C" fn NvHasUdmaCable(mut UdmaTimReg: USHORT, mut Drive: ULONG) -> BOOLEAN {
    if (KeGetCurrentIrql() as ::core::ffi::c_int) as u32 > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                b"drivers/storage/ide/pciidex/chipset/amd.c\0"
                    as *const u8 as *const ::core::ffi::c_char as PVOID,
                317 as ULONG,
                ::core::ptr::null_mut::<CHAR>(),
            );
        } else {
        };
    }
    if ((UdmaTimReg as ::core::ffi::c_int)
        << (1 as ULONG).wrapping_sub(Drive).wrapping_mul(8 as ULONG)
        & AMD_UDMA_EN as ::core::ffi::c_int) as u32
        != AMD_UDMA_EN
    {
        return FALSE;
    }
    return ((UdmaTimReg as ::core::ffi::c_int)
        << (1 as ULONG).wrapping_sub(Drive).wrapping_mul(8 as ULONG)
        & 0x7
        & AMD_UDMA_TIME!(4) as ::core::ffi::c_int
        != 0) as ::core::ffi::c_int as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn AmdGetControllerProperties(
    Controller: *mut ATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;
    let mut HwFlags: ULONG;
    let mut SupportedMode: ULONG;

    PAGED_CODE!();

    i = 0;
    while i < RTL_NUMBER_OF!(AmdControllerList) {
        HwFlags = AmdControllerList[i as usize].Flags as ULONG;

        if (*Controller).Pci.VendorID == AmdControllerList[i as usize].VendorID
            && (*Controller).Pci.DeviceID == AmdControllerList[i as usize].DeviceID
        {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == RTL_NUMBER_OF!(AmdControllerList) {
        return STATUS_NO_MATCH;
    }

    Status = PciIdeCreateChannelData((Controller).cast(), 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    if HwFlags & (HW_FLAGS_SATA) as _ != 0 {
        SupportedMode = SATA_ALL;
    } else {
        SupportedMode = PIO_ALL | MWDMA_ALL | UDMA_ALL;

        if HwFlags & (HW_FLAGS_CHECK_SYSBOARD) as _ != 0 {
            if (*Controller).Pci.SubSystemID == PCI_SUBSYSTEM_AMD_SERENADE {
                SupportedMode &= !UDMA_MODE6;
            }
        } else if HwFlags & (HW_FLAGS_UDMA5) as _ != 0 {
            SupportedMode &= !UDMA_MODE6;
        } else if HwFlags & (HW_FLAGS_UDMA4) as _ != 0 {
            SupportedMode &= !UDMA_MODES!(5, 6);
        }

        if ((*Controller).Pci.VendorID) as u32 == PCI_VEN_AMD {
            (*Controller).__bindgen_anon_1.ChannelEnableBits = &AmdEnableBits as *const ATA_PCI_ENABLE_BITS;
            (*Controller).Start = Some(AmdControllerStart as unsafe extern "C" fn(*mut ATA_CONTROLLER));
        } else {
            (*Controller).__bindgen_anon_1.ChannelEnableBits = &NvEnableBits as *const ATA_PCI_ENABLE_BITS;
        }
    }

    i = 0;
    while i < (*Controller).MaxChannels {
        let ChanData: *mut CHANNEL_DATA_PATA = ((*Controller).Channels[i as usize]).cast();

        (*ChanData).HwFlags = HwFlags;
        (*ChanData).TransferModeSupported = SupportedMode;

        if HwFlags & (HW_FLAGS_SATA) as _ != 0 {
            (*ChanData).ChanInfo |= CHANNEL_FLAG_NO_SLAVE;
            (*ChanData).SetTransferMode = Some(SataSetTransferMode as unsafe extern "C" fn(*mut ATA_CONTROLLER, ULONG, *mut CHANNEL_DEVICE_CONFIG));
        } else {
            (*ChanData).SetTransferMode = Some(AmdSetTransferMode as unsafe extern "C" fn(*mut ATA_CONTROLLER, ULONG, *mut CHANNEL_DEVICE_CONFIG));

            if ((*Controller).Pci.VendorID) as u32 == PCI_VEN_AMD {
                let ConfigReg: UCHAR = PciRead8(
                    Controller,
                    AMD_REG_CONFIG_CR!(AMD_CONFIG_BASE),
                );

                if ConfigReg & AMD_REG_CONFIG_CR!(i) == 0 {
                    INFO!("CH %lu: BIOS detected 40-conductor cable\n", i);
                    (*ChanData).TransferModeSupported &= !UDMA_80C_ALL;
                }
            } else {
                let UdmaTimReg: USHORT = PciRead16(
                    Controller,
                    AMD_REG_UDMA!(NV_CONFIG_BASE, i),
                );

                if NvHasUdmaCable(UdmaTimReg, 0) == 0
                    && NvHasUdmaCable(UdmaTimReg, 1) == 0
                {
                    INFO!(
                        "CH %lu: BIOS hasn't selected mode faster than UDMA 2, assume 40-conductor cable\n",
                        (*ChanData).Channel,
                    );
                    (*ChanData).TransferModeSupported &= !UDMA_80C_ALL;
                }
            }
        }
        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
