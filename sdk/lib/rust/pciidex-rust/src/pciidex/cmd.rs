use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut CmdPrefetchDisable: [[UCHAR; 2]; 2] = [
    [
        0x40 as ::core::ffi::c_int as UCHAR,
        0x80 as ::core::ffi::c_int as UCHAR,
    ],
    [
        0x4 as ::core::ffi::c_int as UCHAR,
        0x8 as ::core::ffi::c_int as UCHAR,
    ],
];

static mut CmdPrefetchRegs: [ULONG; 2] = [CMD_REG_CNTRL as ULONG, CMD_REG_ARTTIM23 as ULONG];

static mut CmdDrwTimRegs: [[ULONG; 2]; 2] = [
    [CMD_REG_DRWTIM0 as ULONG, CMD_REG_DRWTIM1 as ULONG],
    [CMD_REG_DRWTIM2 as ULONG, CMD_REG_DRWTIM3 as ULONG],
];

static mut CmdArtTimRegs: [[ULONG; 2]; 2] = [
    [CMD_REG_ARTTIM0 as ULONG, CMD_REG_ARTTIM1 as ULONG],
    [CMD_REG_ARTTIM23 as ULONG, CMD_REG_ARTTIM23 as ULONG],
];

static mut CmdArtTimings: [UCHAR; 6] = [
    0x40 as ::core::ffi::c_int as UCHAR,
    0x40 as ::core::ffi::c_int as UCHAR,
    0x40 as ::core::ffi::c_int as UCHAR,
    0x80 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0xc0 as ::core::ffi::c_int as UCHAR,
];

static mut CmdUdmaTimings: [[UCHAR; 2]; 6] = [
    [
        0x31 as ::core::ffi::c_int as UCHAR,
        0xc2 as ::core::ffi::c_int as UCHAR,
    ],
    [
        0x21 as ::core::ffi::c_int as UCHAR,
        0x82 as ::core::ffi::c_int as UCHAR,
    ],
    [
        0x11 as ::core::ffi::c_int as UCHAR,
        0x42 as ::core::ffi::c_int as UCHAR,
    ],
    [
        0x25 as ::core::ffi::c_int as UCHAR,
        0x8a as ::core::ffi::c_int as UCHAR,
    ],
    [
        0x15 as ::core::ffi::c_int as UCHAR,
        0x4a as ::core::ffi::c_int as UCHAR,
    ],
    [
        0x5 as ::core::ffi::c_int as UCHAR,
        0xa as ::core::ffi::c_int as UCHAR,
    ],
];

pub const PCI_DEV_PCI0640: u32 = 0x0640;
pub const PCI_DEV_PCI0643: u32 = 0x0643;
pub const PCI_DEV_PCI0646: u32 = 0x0646;
pub const PCI_DEV_PCI0648: u32 = 0x0648;
pub const PCI_DEV_CMD0649: u32 = 0x0649;
pub const HW_FLAGS_PRIMARY_ENABLED: u32 = 0x0001;
pub const HW_FLAGS_HAS_UDMA_REG: u32 = 0x0002; // nolint: const-contradiction (cmd.c-local flag)
pub const HW_FLAGS_NEED_PIO_FIX: u32 = 0x0004;
pub const HW_FLAGS_NO_PREFETCH: u32 = 0x0008;
pub const CMD_PCI_CLOCK: u32 = 30000;
pub const CMD_REG_CFR: u32 = 0x50;
pub const CMD_REG_CNTRL: u32 = 0x51;
pub const CMD_REG_CMDTIM: u32 = 0x52;
pub const CMD_REG_ARTTIM0: u32 = 0x53;
pub const CMD_REG_DRWTIM0: u32 = 0x54;
pub const CMD_REG_ARTTIM1: u32 = 0x55;
pub const CMD_REG_DRWTIM1: u32 = 0x56;
pub const CMD_REG_ARTTIM23: u32 = 0x57;
pub const CMD_REG_DRWTIM2: u32 = 0x58;
pub const CMD_REG_DRWTIM23: u32 = 0x58;
pub const CMD_REG_DRWTIM3: u32 = 0x5B;
pub const CMD_REG_MRDMODE: u32 = 0x71;
pub const CMD_REG_BMIDECSR: u32 = 0x79;
pub const CMD_CFR_INTR_CH0: u32 = 0x04;
pub const CMD_CFR_INTR_CH1: u32 = 0x10;
pub const CMD_CNTRL_CHAN_EN_CH0: u32 = 0x04;
pub const CMD_CNTRL_CHAN_EN_CH1: u32 = 0x08;
pub const CMD_ARTTIM_ART_MASK: u32 = 0xC0;
pub const CMD_MRDMODE_READ_MULTIPLE: u32 = 0x01;
pub const CMD_MRDMODE_INTR_CH0: u32 = 0x04;
pub const CMD_MRDMODE_INTR_CH1: u32 = 0x08;
pub const CMD_MRDMODE_INTR_CH0_BLOCK: u32 = 0x10;
pub const CMD_MRDMODE_INTR_CH1_BLOCK: u32 = 0x20;
pub const CMD_BMIDECSR_CR_CH0: u32 = 0x01;
pub const CMD_BMIDECSR_CR_CH1: u32 = 0x02;
pub const CMD_UDIDETCR_CLEAR_DRIVE0: u32 = 0x35;
pub const CMD_UDIDETCR_CLEAR_DRIVE1: u32 = 0xCA;
pub const CMD_UDIDETCR_EN_DRIVE0: u32 = 0x01;
pub const CMD_UDIDETCR_EN_DRIVE1: u32 = 0x02;

#[repr(C)]
pub struct _CMD_HW_EXTENSION {
    pub CmdActive: [ATATIM; (MAX_IDE_CHANNEL * MAX_IDE_DEVICE) as _],
    pub CmdRecovery: [ATATIM; (MAX_IDE_CHANNEL * MAX_IDE_DEVICE) as _],
}
pub type CMD_HW_EXTENSION = _CMD_HW_EXTENSION;
pub type PCMD_HW_EXTENSION = *mut _CMD_HW_EXTENSION;

unsafe extern "C" fn CmdFixRecoveryTiming(
    mut Device: PCHANNEL_DEVICE_CONFIG,
    mut Timing: PATA_TIMING,
    mut CycleTimeClocks: ATATIM,
    mut CmdActiveClocks: ATATIM,
    mut CmdRecoveryClocks: ATATIM,
    mut DataActiveClocks: ATATIM,
    mut DataRecoveryClocks: ATATIM,
) {
    if (*Timing).DataActive as ::core::ffi::c_int + (*Timing).DataRecovery as ::core::ffi::c_int
        > CycleTimeClocks as ::core::ffi::c_int
        || (*Device).DmaMode == MWDMA_MODE!(0)
    {
        if ((*Timing).CmdActive as ::core::ffi::c_int) < CmdActiveClocks as ::core::ffi::c_int {
            (*Timing).CmdActive = CmdActiveClocks;
        }
        if ((*Timing).CmdRecovery as ::core::ffi::c_int) < CmdRecoveryClocks as ::core::ffi::c_int {
            (*Timing).CmdRecovery = CmdRecoveryClocks;
        }
        if ((*Timing).DataActive as ::core::ffi::c_int) < DataActiveClocks as ::core::ffi::c_int {
            (*Timing).DataActive = DataActiveClocks;
        }
        if ((*Timing).DataRecovery as ::core::ffi::c_int) < DataRecoveryClocks as ::core::ffi::c_int
        {
            (*Timing).DataRecovery = DataRecoveryClocks;
        }
    } else {
        (*Timing).CmdActive = CmdActiveClocks;
        (*Timing).CmdRecovery = CmdRecoveryClocks;
        (*Timing).DataActive = DataActiveClocks;
        (*Timing).DataRecovery = DataRecoveryClocks;
    };
}

unsafe extern "C" fn CmdDerateTimings(
    Device: PCHANNEL_DEVICE_CONFIG,
    Timing: PATA_TIMING,
) {
    match (*Device).PioMode {
        PIO_MODE!(0) => {
            CmdFixRecoveryTiming(
                Device,
                Timing,
                20,
                16,
                3,
                16,
                3,
            );
        }
        PIO_MODE!(1) => {
            CmdFixRecoveryTiming(
                Device,
                Timing,
                13,
                10,
                3,
                10,
                3,
            );
        }
        PIO_MODE!(2) => {
            CmdFixRecoveryTiming(
                Device,
                Timing,
                8,
                10,
                3,
                5,
                3,
            );
        }
        _ => {}
    }
}

unsafe extern "C" fn CmdPackTimings(
    mut Controller: PATA_CONTROLLER,
    mut Active: ATATIM,
    mut Recovery: ATATIM,
) -> UCHAR {
    if (*Controller).Pci.DeviceID as ::core::ffi::c_int == (PCI_DEV_PCI0640 as i32) {
        if Active as ::core::ffi::c_int >= 16 {
            Active = 0 as ATATIM;
        } else if (Active as ::core::ffi::c_int) < 2 {
            Active = 2 as ATATIM;
        }
        if (*Controller).Pci.RevisionID as ::core::ffi::c_int > 1 {
            if Recovery as ::core::ffi::c_int >= 17 {
                Recovery = 0 as ATATIM;
            } else if Recovery as ::core::ffi::c_int > 2 {
                Recovery = Recovery.wrapping_sub(1);
            } else {
                Recovery = 2 as ATATIM;
            }
        } else if Recovery as ::core::ffi::c_int >= 16 {
            Recovery = 0 as ATATIM;
        } else if (Recovery as ::core::ffi::c_int) < 2 {
            Recovery = 2 as ATATIM;
        }
    } else {
        if Active as ::core::ffi::c_int >= 16 {
            Active = 0 as ATATIM;
        }
        if Recovery as ::core::ffi::c_int >= 16 {
            Recovery = 0 as ATATIM;
        } else if Recovery as ::core::ffi::c_int > 1 {
            Recovery = Recovery.wrapping_sub(1);
        } else {
            Recovery = 15 as ATATIM;
        }
    }
    return ((Active as ::core::ffi::c_int) << 4
        | Recovery as ::core::ffi::c_int) as UCHAR;
}

unsafe extern "C" fn CmdSetTransferMode(mut Controller: PATA_CONTROLLER, mut Channel: ULONG, mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    let mut ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[Channel as usize] as PCHANNEL_DATA_PATA;
    let mut HwExt: PCMD_HW_EXTENSION = (*Controller).HwExt as PCMD_HW_EXTENSION;
    let mut DeviceTimings: [ATA_TIMING; (MAX_IDE_DEVICE as usize)] = [_ATA_TIMING {
        AddressSetup: 0,
        CmdActive: 0,
        CmdRecovery: 0,
        DataActive: 0,
        DataRecovery: 0,
    }; (MAX_IDE_DEVICE as usize)];
    let mut i: ULONG = 0;
    let mut TimingFlags: ULONG = 0;
    let mut Value: UCHAR = 0;
    TimingFlags = 0;
    if Channel != 0 {
        TimingFlags |= SHARED_ADDR_TIMINGS;
        if (*Controller).Pci.DeviceID as ::core::ffi::c_int == (PCI_DEV_PCI0640 as i32) {
            TimingFlags |= SHARED_DATA_TIMINGS;
        }
    }
    AtaSelectTimings(
        DeviceList,
        &raw mut DeviceTimings as PATA_TIMING,
        CMD_PCI_CLOCK,
        TimingFlags,
    );
    Value = PciRead8(Controller, CmdPrefetchRegs[Channel as usize]);
    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        if !Device.is_null() {
            if (*ChanData).HwFlags & HW_FLAGS_NO_PREFETCH != 0 || (*Device).IsFixedDisk == 0 {
                Value = (Value as ::core::ffi::c_int
                    | CmdPrefetchDisable[Channel as usize][i as usize] as ::core::ffi::c_int)
                    as UCHAR;
            } else {
                Value = (Value as ::core::ffi::c_int
                    & !(CmdPrefetchDisable[Channel as usize][i as usize] as ::core::ffi::c_int))
                    as UCHAR;
            }
        }
        i = i.wrapping_add(1);
    }
    PciWrite8(Controller, CmdPrefetchRegs[Channel as usize], Value);
    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut Device_0: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let mut Timing: PATA_TIMING =
            (&raw mut DeviceTimings as *mut ATA_TIMING).offset(i as isize) as PATA_TIMING;
        let mut UdmaTimReg: UCHAR = 0;
        let mut Register: ULONG = 0;
        if (*ChanData).HwFlags & HW_FLAGS_HAS_UDMA_REG != 0 {
            UdmaTimReg = PciRead8(Controller, CMD_REG_UDIDETCR!(Channel));
            if !Device_0.is_null()
                && (*Device_0).DmaMode >= UDMA_MODE!(0)
            {
                UdmaTimReg = (UdmaTimReg as ::core::ffi::c_int
                    & (!CMD_UDIDETCR_CLEAR!(i)) as i32) as UCHAR;
                UdmaTimReg = (UdmaTimReg as ::core::ffi::c_int
                    | CmdUdmaTimings[(*Device_0)
                        .DmaMode
                        .wrapping_sub(UDMA_MODE!(0)) as usize][i as usize]
                        as ::core::ffi::c_int) as UCHAR;
            } else {
                UdmaTimReg = (UdmaTimReg as ::core::ffi::c_int
                    & (!CMD_UDIDETCR_EN!(i)) as i32) as UCHAR;
            }
            PciWrite8(Controller, CMD_REG_UDIDETCR!(Channel), UdmaTimReg);
        }
        if Device_0.is_null() {
            (*HwExt).CmdActive
                [i.wrapping_add(Channel.wrapping_mul(MAX_IDE_DEVICE)) as usize] =
                0;
            (*HwExt).CmdRecovery
                [i.wrapping_add(Channel.wrapping_mul(MAX_IDE_DEVICE)) as usize] =
                0;
        } else {
            if (*ChanData).HwFlags & HW_FLAGS_NEED_PIO_FIX != 0 {
                CmdDerateTimings(Device_0, Timing);
            }
            (*HwExt).CmdActive
                [i.wrapping_add(Channel.wrapping_mul(MAX_IDE_DEVICE)) as usize] =
                (*Timing).CmdActive;
            (*HwExt).CmdRecovery
                [i.wrapping_add(Channel.wrapping_mul(MAX_IDE_DEVICE)) as usize] =
                (*Timing).CmdRecovery;
            if (*Timing).AddressSetup as usize
                > (RTL_NUMBER_OF!(CmdArtTimings) - 1) as _
            {
                (*Timing).AddressSetup =
                    (RTL_NUMBER_OF!(CmdArtTimings) - 1) as ATATIM;
            }
            Value = PciRead8(Controller, CmdArtTimRegs[Channel as usize][i as usize]);
            Value = (Value as ::core::ffi::c_int & (!CMD_ARTTIM_ART_MASK) as i32) as UCHAR;
            Value = (Value as ::core::ffi::c_int
                | CmdArtTimings[(*Timing).AddressSetup as usize] as ::core::ffi::c_int)
                as UCHAR;
            PciWrite8(
                Controller,
                CmdArtTimRegs[Channel as usize][i as usize],
                Value,
            );
            Value = CmdPackTimings(Controller, (*Timing).DataActive, (*Timing).DataRecovery);
            if (*Controller).Pci.DeviceID as ::core::ffi::c_int == (PCI_DEV_PCI0640 as i32)
                && Channel != 0
            {
                Register = CMD_REG_DRWTIM23;
            } else {
                Register = CmdDrwTimRegs[Channel as usize][i as usize];
            }
            PciWrite8(Controller, Register, Value);
        }
        i = i.wrapping_add(1);
    }
    i = 0;
    while i < MAX_IDE_CHANNEL * MAX_IDE_DEVICE {
        if (*HwExt).CmdActive[i as usize] as ::core::ffi::c_int
            > DeviceTimings[0].CmdActive as ::core::ffi::c_int
        {
            DeviceTimings[0].CmdActive = (*HwExt).CmdActive[i as usize];
        }
        if (*HwExt).CmdRecovery[i as usize] as ::core::ffi::c_int
            > DeviceTimings[0].CmdRecovery as ::core::ffi::c_int
        {
            DeviceTimings[0].CmdRecovery = (*HwExt).CmdRecovery[i as usize];
        }
        i = i.wrapping_add(1);
    }
    if DeviceTimings[0].CmdActive as ::core::ffi::c_int
        != 0
        || DeviceTimings[0].CmdRecovery as ::core::ffi::c_int
            != 0
    {
        Value = CmdPackTimings(
            Controller,
            DeviceTimings[0].CmdActive,
            DeviceTimings[0].CmdRecovery,
        );
        PciWrite8(Controller, CMD_REG_CMDTIM, Value);
    }
}

unsafe extern "C" fn CmdChannelEnabledTest(Controller: PATA_CONTROLLER, Channel: ULONG) -> IDE_CHANNEL_STATE {
    let ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[Channel as usize] as PCHANNEL_DATA_PATA;
    let mut Control: UCHAR;

    if (Channel == 0) && ((*ChanData).HwFlags & HW_FLAGS_PRIMARY_ENABLED) != 0 {
        return ChannelEnabled;
    }

    Control = PciRead8(Controller, CMD_REG_CNTRL);
    if (Control as i32 & CMD_CNTRL_CHAN_EN!(Channel) as i32) != 0 {
        return ChannelEnabled;
    }

    return ChannelDisabled;
}

unsafe extern "C" fn CmdCheckInterruptMrdMode(ChanData: PCHANNEL_DATA_PATA) -> BOOLEAN {
    let Controller: *mut ATA_CONTROLLER = (*ChanData).__bindgen_anon_1.Controller;
    let mut Control: UCHAR;
    let InterruptMask: UCHAR;

    if !(*ChanData).Regs.Dma.is_null() {
        Control = ATA_READ!(
            (*ChanData).Regs.Dma.add(1),
            ChanData,
            MRES_DMA,
        );
    } else {
        Control = PciRead8(Controller, CMD_REG_MRDMODE);
    }

    InterruptMask = CMD_MRDMODE_INTR!((*ChanData).__bindgen_anon_1.Channel);
    if Control & InterruptMask == 0 {
        return FALSE;
    }

    Control &= (!(CMD_MRDMODE_INTR!(0) | CMD_MRDMODE_INTR!(1))) as u8;
    Control |= InterruptMask;
    Control |= (CMD_MRDMODE_READ_MULTIPLE as u8);

    if !(*ChanData).Regs.Dma.is_null() {
        ATA_WRITE!(
            (*ChanData).Regs.Dma.add(1),
            Control,
            ChanData,
            MRES_DMA,
        );
    } else {
        PciWrite8(Controller, CMD_REG_MRDMODE, Control);
    }

    return TRUE;
}

unsafe extern "C" fn CmdCheckInterruptPci(ChanData: PCHANNEL_DATA_PATA) -> BOOLEAN {
    let Controller: PATA_CONTROLLER = (*ChanData).__bindgen_anon_1.Controller;
    let mut Status: UCHAR;
    let mut InterruptMask: UCHAR;
    let mut Register: ULONG;

    InterruptMask = CMD_CFR_INTR!((*ChanData).__bindgen_anon_1.Channel);
    Register = if (*ChanData).__bindgen_anon_1.Channel == 0 {
        CMD_REG_CFR
    } else {
        CMD_REG_ARTTIM23
    };

    /* Read CFR or REG_ARTTIM23 to clear the interrupt */
    Status = PciRead8(Controller, Register);
    return ((Status as i32 & InterruptMask as i32) != 0) as BOOLEAN;
}

unsafe extern "C" fn CmdControllerStart(mut Controller: PATA_CONTROLLER) {
    let mut i: ULONG = 0;
    let mut Control: UCHAR = 0;
    PciWrite8(Controller, CMD_REG_CMDTIM, 0);
    if (*Controller).Pci.DeviceID as ::core::ffi::c_int == (PCI_DEV_PCI0640 as i32) {
        i = 0;
        while (i as usize)
            < (::core::mem::size_of::<[ULONG; 2]>() as usize)
                .wrapping_div(::core::mem::size_of::<ULONG>() as usize)
        {
            Control = PciRead8(Controller, CmdPrefetchRegs[i as usize]);
            Control = (Control as ::core::ffi::c_int
                | (CmdPrefetchDisable[i as usize][0 as usize]
                    as ::core::ffi::c_int
                    | CmdPrefetchDisable[i as usize][1 as usize]
                        as ::core::ffi::c_int)) as UCHAR;
            PciWrite8(Controller, CmdPrefetchRegs[i as usize], Control);
            i = i.wrapping_add(1);
        }
    } else {
        Control = PciRead8(Controller, CMD_REG_MRDMODE);
        Control = (Control as ::core::ffi::c_int
            & (!(CMD_MRDMODE_INTR_CH0_BLOCK | CMD_MRDMODE_INTR_CH1_BLOCK)) as i32)
            as UCHAR;
        Control = (Control as ::core::ffi::c_int | (CMD_MRDMODE_READ_MULTIPLE as i32)) as UCHAR;
        PciWrite8(Controller, CMD_REG_MRDMODE, Control);
    };
}
#[no_mangle]
pub unsafe extern "C" fn CmdGetControllerProperties(
    Controller: *mut ATA_CONTROLLER,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut i: ULONG;
    let mut SupportedMode: ULONG;
    let mut HwFlags: ULONG = 0;
    let mut CheckInterrupt: PCHANNEL_CHECK_INTERRUPT = Some(CmdCheckInterruptPci);

    PAGED_CODE!();
    ASSERT!((*Controller).Pci.VendorID as u32 == PCI_VEN_CMD);

    match ((*Controller).Pci.DeviceID) as u32 {
        PCI_DEV_PCI0640 => {
            (*Controller).Flags |= CTRL_FLAG_IS_SIMPLEX;
            SupportedMode = PIO_ALL;
            HwFlags |= HW_FLAGS_PRIMARY_ENABLED | HW_FLAGS_NO_PREFETCH;
        }
        PCI_DEV_PCI0643 => {
            SupportedMode = PIO_ALL | SWDMA_ALL | MWDMA_ALL;
            if (*Controller).Pci.RevisionID < 6 {
                HwFlags |= HW_FLAGS_PRIMARY_ENABLED;
            }
        }
        PCI_DEV_PCI0646 => {
            SupportedMode = PIO_ALL | SWDMA_ALL | MWDMA_ALL;
            HwFlags |= HW_FLAGS_HAS_UDMA_REG;
            if (*Controller).Pci.RevisionID < 3 {
                HwFlags |= HW_FLAGS_PRIMARY_ENABLED;
            } else {
                CheckInterrupt = Some(CmdCheckInterruptMrdMode);
            }
            if (*Controller).Pci.RevisionID == 5
                || (*Controller).Pci.RevisionID == 6
            {
                SupportedMode |= UDMA_MODES!(0, 1);
            } else if (*Controller).Pci.RevisionID > 6 {
                SupportedMode |= UDMA_MODES!(0, 2);
            }
        }
        PCI_DEV_PCI0648 => {
            SupportedMode = PIO_ALL
                | MWDMA_ALL
                | UDMA_MODES!(0, 4);
            HwFlags |= HW_FLAGS_HAS_UDMA_REG | HW_FLAGS_NO_PREFETCH;
            CheckInterrupt = Some(CmdCheckInterruptMrdMode);
        }
        PCI_DEV_CMD0649 => {
            SupportedMode = PIO_ALL
                | MWDMA_ALL
                | UDMA_MODES!(0, 5);
            HwFlags |= HW_FLAGS_HAS_UDMA_REG;
            if (*Controller).Pci.RevisionID == 2 {
                HwFlags |= HW_FLAGS_NEED_PIO_FIX;
            }
            CheckInterrupt = Some(CmdCheckInterruptMrdMode);
        }
        _ => {
            return STATUS_NO_MATCH;
        }
    }

    (*Controller).Start = Some(CmdControllerStart);
    (*Controller).Flags |= CTRL_FLAG_USE_TEST_FUNCTION;
    (*Controller).__bindgen_anon_1.ChannelEnabledTest = Some(CmdChannelEnabledTest);

    Status = PciIdeCreateChannelData(Controller, 0);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    (*Controller).HwExt = ExAllocatePoolZero(
        NonPagedPool,
        core::mem::size_of::<CMD_HW_EXTENSION>(),
        TAG_PCIIDEX,
    );
    if (*Controller).HwExt.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    i = 0;
    while i < (*Controller).MaxChannels {
        let ChanData: PCHANNEL_DATA_PATA = ((*Controller).Channels[i as usize]).cast();

        (*ChanData).HwFlags = HwFlags;
        (*ChanData).CheckInterrupt = CheckInterrupt;
        (*ChanData).__bindgen_anon_1.SetTransferMode = Some(CmdSetTransferMode);
        (*ChanData).__bindgen_anon_1.TransferModeSupported = SupportedMode;

        if (*Controller).Pci.DeviceID == (PCI_DEV_PCI0640 as u16) {
            (*ChanData).__bindgen_anon_1.ChanInfo &= !CHANNEL_FLAG_IO32;
        } else {
            if (*ChanData).__bindgen_anon_1.TransferModeSupported & UDMA_80C_ALL != 0 {
                let BmControl: UCHAR = PciRead8(Controller, CMD_REG_BMIDECSR);
                if BmControl & CMD_BMIDECSR_CR!(i) == 0 {
                    INFO!(
                        b"CH %lu: BIOS detected 40-conductor cable\n\0".as_ptr() as *const i8,
                        (*ChanData).Channel,
                    );
                    (*ChanData).__bindgen_anon_1.TransferModeSupported &= !UDMA_80C_ALL;
                }
            }
        }

        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
