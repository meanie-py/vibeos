use crate::prelude::*;
use ntkrnl_sys::ddk::{HalGetBusDataByOffset, PCI_SLOT_NUMBER, PPCI_COMMON_HEADER};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut IntelControllerList: [INTEL_CONTROLLER_INFO; 88] = [
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PIIX_82371FB as USHORT,
        Type: TYPE_PIIX as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_MPIIX_82371MX as USHORT,
        Type: TYPE_MPIIX as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PIIX3_82371SB as USHORT,
        Type: TYPE_PIIX3 as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PIIX4_82371AB as USHORT,
        Type: TYPE_PIIX4 as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PIIX4E_82372FB as USHORT,
        Type: TYPE_PIIX4 as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PIIX4E_82443MX as USHORT,
        Type: TYPE_PIIX4 as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PIIX4E_82451NX as USHORT,
        Type: TYPE_PIIX4 as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH_82801AA as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH0_82801AB as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH2_82801BAM as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH2_82801BA as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_C_ICH_82801E as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH3_M_82801CAM as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH3_S_82801CA as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH4_L_82801DBL as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH4_M_82801DBM as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH4_82801DB as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH5_82801EB_SATA as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH5_82801EB_IDE as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH5_R_82801ER as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH5_6300ESB_IDE as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH5_6300ESB_SATA as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH5_6300ESB_RAID as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH6_82801FB_SATA as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH6_R_82801FB as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH6_M_82801FBM as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH6_82801FB_IDE as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ESB2_63XXESB as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH7_6321ESB as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH7_82801GB as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH7_M_82801GBM as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH7_82801G as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH8_82801H as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH8_R_82801HR as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH8_M_82801HM as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH8_M_82801HBM as USHORT,
        Type: TYPE_ICH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH9_R_82801IR_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH9_R_82801IR_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH9_82801I as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH9_M_82801IBM_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH9_M_82801IBM_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH9_M_82801IBM_3 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH10_82801JD_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH10_82801JD_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH10_82801JI_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ICH10_82801JI_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_SCH_ATOM_Z5XX as USHORT,
        Type: TYPE_SCH as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_EP80579 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_5SERIES_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_5SERIES_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_5SERIES_3 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_5SERIES_4 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_5SERIES_5 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_5SERIES_6 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_6SERIES_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_6SERIES_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_6SERIES_3 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_6SERIES_4 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_7SERIES_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_7SERIES_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_7SERIES_3 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_7SERIES_4 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_X79_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_X79_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_8900_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_8900_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ATOM_C2000_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ATOM_C2000_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ATOM_C2000_3 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ATOM_C2000_4 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_8SERIES_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_8SERIES_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_8SERIES_3 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_8SERIES_4 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_8SERIES_5 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_8SERIES_6 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_8SERIES_7 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_8SERIES_8 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ATOM_E3800_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_ATOM_E3800_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_X99_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_X99_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_X99_3 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_X99_4 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_9SERIES_1 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_9SERIES_2 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_9SERIES_3 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
    _INTEL_CONTROLLER_INFO {
        DeviceID: PCI_DEV_PCH_9SERIES_4 as USHORT,
        Type: TYPE_SATA as USHORT,
    },
];

static mut IntelPiixEnableBits: [ATA_PCI_ENABLE_BITS; 2] = [
    _ATA_PCI_ENABLE_BITS {
        Register: 0x41 as UCHAR,
        Mask: 0x80 as UCHAR,
        ValueEnabled: 0x80 as UCHAR,
    },
    _ATA_PCI_ENABLE_BITS {
        Register: 0x43 as UCHAR,
        Mask: 0x80 as UCHAR,
        ValueEnabled: 0x80 as UCHAR,
    },
];

static mut IntelMpiixEnableBits: [ATA_PCI_ENABLE_BITS; 2] = [
    _ATA_PCI_ENABLE_BITS {
        Register: 0x6d as UCHAR,
        Mask: 0xc0 as UCHAR,
        ValueEnabled: 0x80 as UCHAR,
    },
    _ATA_PCI_ENABLE_BITS {
        Register: 0x6d as UCHAR,
        Mask: 0xc0 as UCHAR,
        ValueEnabled: 0xc0 as UCHAR,
    },
];

static mut IntelClockSettings: [[UCHAR; 2]; 5] = [
    [
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
    ],
    [
        0 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
    ],
    [
        1 as ::core::ffi::c_int as UCHAR,
        0 as ::core::ffi::c_int as UCHAR,
    ],
    [
        2 as ::core::ffi::c_int as UCHAR,
        1 as ::core::ffi::c_int as UCHAR,
    ],
    [
        2 as ::core::ffi::c_int as UCHAR,
        3 as ::core::ffi::c_int as UCHAR,
    ],
];

static mut IntelTimingModeToCycleTime: [ULONG; 5] = [
    900 as ::core::ffi::c_int as ULONG,
    900 as ::core::ffi::c_int as ULONG,
    240 as ::core::ffi::c_int as ULONG,
    180 as ::core::ffi::c_int as ULONG,
    120 as ::core::ffi::c_int as ULONG,
];

static mut IntelModeSettings: [UCHAR; 5] = [
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    ((0x1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int) as UCHAR,
    ((0x1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int
        | (0x2 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int)
        as UCHAR,
    ((0x1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int
        | (0x2 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int * 0 as ::core::ffi::c_int)
        as UCHAR,
];

static mut IntelUdmaSettings: [UCHAR; 6] = [
    0 as ::core::ffi::c_int as UCHAR,
    1 as ::core::ffi::c_int as UCHAR,
    2 as ::core::ffi::c_int as UCHAR,
    1 as ::core::ffi::c_int as UCHAR,
    2 as ::core::ffi::c_int as UCHAR,
    1 as ::core::ffi::c_int as UCHAR,
];

static mut IntelDmaModeToTimingMode: [ULONG; 4] = [
    2 as ::core::ffi::c_int as ULONG,
    0 as ::core::ffi::c_int as ULONG,
    3 as ::core::ffi::c_int as ULONG,
    4 as ::core::ffi::c_int as ULONG,
];

pub const PCI_DEV_PIIX_82371FB: u16 = 0x1230;
pub const PCI_DEV_MPIIX_82371MX: u16 = 0x1234;
pub const PCI_DEV_PIIX3_82371SB: u16 = 0x7010;
pub const PCI_DEV_PIIX4_82371AB: u16 = 0x7111;
pub const PCI_DEV_PIIX4E_82372FB: u16 = 0x7601;
pub const PCI_DEV_PIIX4E_82443MX: u16 = 0x7199;
pub const PCI_DEV_PIIX4E_82451NX: u16 = 0x84CA;
pub const PCI_DEV_ICH_82801AA: u16 = 0x2411;
pub const PCI_DEV_ICH0_82801AB: u16 = 0x2421;
pub const PCI_DEV_ICH2_82801BAM: u16 = 0x244A;
pub const PCI_DEV_ICH2_82801BA: u16 = 0x244B;
pub const PCI_DEV_C_ICH_82801E: u16 = 0x245B;
pub const PCI_DEV_ICH3_M_82801CAM: u16 = 0x248A;
pub const PCI_DEV_ICH3_S_82801CA: u16 = 0x248B;
pub const PCI_DEV_ICH4_L_82801DBL: u16 = 0x24C1;
pub const PCI_DEV_ICH4_M_82801DBM: u16 = 0x24CA;
pub const PCI_DEV_ICH4_82801DB: u16 = 0x24CB;
pub const PCI_DEV_ICH5_82801EB_SATA: u16 = 0x24D1;
pub const PCI_DEV_ICH5_82801EB_IDE: u16 = 0x24DB;
pub const PCI_DEV_ICH5_R_82801ER: u16 = 0x24DF;
pub const PCI_DEV_ICH5_6300ESB_IDE: u16 = 0x25A2;
pub const PCI_DEV_ICH5_6300ESB_SATA: u16 = 0x25A3;
pub const PCI_DEV_ICH5_6300ESB_RAID: u16 = 0x25B0;
pub const PCI_DEV_ICH6_82801FB_SATA: u16 = 0x2651;
pub const PCI_DEV_ICH6_R_82801FB: u16 = 0x2652;
pub const PCI_DEV_ICH6_M_82801FBM: u16 = 0x2653;
pub const PCI_DEV_ICH6_82801FB_IDE: u16 = 0x266F;
pub const PCI_DEV_ESB2_63XXESB: u16 = 0x2680;
pub const PCI_DEV_ICH7_6321ESB: u16 = 0x269E;
pub const PCI_DEV_ICH7_82801GB: u16 = 0x27C0;
pub const PCI_DEV_ICH7_M_82801GBM: u16 = 0x27C4;
pub const PCI_DEV_ICH7_82801G: u16 = 0x27DF;
pub const PCI_DEV_ICH8_82801H: u16 = 0x2820;
pub const PCI_DEV_ICH8_R_82801HR: u16 = 0x2825;
pub const PCI_DEV_ICH8_M_82801HM: u16 = 0x2828;
pub const PCI_DEV_ICH8_M_82801HBM: u16 = 0x2850;
pub const PCI_DEV_ICH9_R_82801IR_1: u16 = 0x2920;
pub const PCI_DEV_ICH9_R_82801IR_2: u16 = 0x2921;
pub const PCI_DEV_ICH9_82801I: u16 = 0x2926;
pub const PCI_DEV_ICH9_M_82801IBM_1: u16 = 0x2928;
pub const PCI_DEV_ICH9_M_82801IBM_2: u16 = 0x292D;
pub const PCI_DEV_ICH9_M_82801IBM_3: u16 = 0x292E;
pub const PCI_DEV_ICH10_82801JD_1: u16 = 0x3A00;
pub const PCI_DEV_ICH10_82801JD_2: u16 = 0x3A06;
pub const PCI_DEV_ICH10_82801JI_1: u16 = 0x3A20;
pub const PCI_DEV_ICH10_82801JI_2: u16 = 0x3A26;
pub const PCI_DEV_SCH_ATOM_Z5XX: u16 = 0x811A;
pub const PCI_DEV_EP80579: u16 = 0x5028;
pub const PCI_DEV_PCH_5SERIES_1: u16 = 0x3B20;
pub const PCI_DEV_PCH_5SERIES_2: u16 = 0x3B21;
pub const PCI_DEV_PCH_5SERIES_3: u16 = 0x3B26;
pub const PCI_DEV_PCH_5SERIES_4: u16 = 0x3B28;
pub const PCI_DEV_PCH_5SERIES_5: u16 = 0x3B2D;
pub const PCI_DEV_PCH_5SERIES_6: u16 = 0x3B2E;
pub const PCI_DEV_PCH_6SERIES_1: u16 = 0x1C00;
pub const PCI_DEV_PCH_6SERIES_2: u16 = 0x1C01;
pub const PCI_DEV_PCH_6SERIES_3: u16 = 0x1C08;
pub const PCI_DEV_PCH_6SERIES_4: u16 = 0x1C09;
pub const PCI_DEV_PCH_7SERIES_1: u16 = 0x1E00;
pub const PCI_DEV_PCH_7SERIES_2: u16 = 0x1E01;
pub const PCI_DEV_PCH_7SERIES_3: u16 = 0x1E08;
pub const PCI_DEV_PCH_7SERIES_4: u16 = 0x1E09;
pub const PCI_DEV_PCH_X79_1: u16 = 0x1D00;
pub const PCI_DEV_PCH_X79_2: u16 = 0x1D08;
pub const PCI_DEV_PCH_8900_1: u16 = 0x2326;
pub const PCI_DEV_PCH_8900_2: u16 = 0x23A6;
pub const PCI_DEV_ATOM_C2000_1: u16 = 0x1F20;
pub const PCI_DEV_ATOM_C2000_2: u16 = 0x1F21;
pub const PCI_DEV_ATOM_C2000_3: u16 = 0x1F30;
pub const PCI_DEV_ATOM_C2000_4: u16 = 0x1F31;
pub const PCI_DEV_PCH_8SERIES_1: u16 = 0x8C00;
pub const PCI_DEV_PCH_8SERIES_2: u16 = 0x8C01;
pub const PCI_DEV_PCH_8SERIES_3: u16 = 0x8C08;
pub const PCI_DEV_PCH_8SERIES_4: u16 = 0x8C09;
pub const PCI_DEV_PCH_8SERIES_5: u16 = 0x9C00;
pub const PCI_DEV_PCH_8SERIES_6: u16 = 0x9C01;
pub const PCI_DEV_PCH_8SERIES_7: u16 = 0x9C08;
pub const PCI_DEV_PCH_8SERIES_8: u16 = 0x9C09;
pub const PCI_DEV_ATOM_E3800_1: u16 = 0x0F20;
pub const PCI_DEV_ATOM_E3800_2: u16 = 0x0F21;
pub const PCI_DEV_PCH_X99_1: u16 = 0x8D00;
pub const PCI_DEV_PCH_X99_2: u16 = 0x8D08;
pub const PCI_DEV_PCH_X99_3: u16 = 0x8D60;
pub const PCI_DEV_PCH_X99_4: u16 = 0x8D68;
pub const PCI_DEV_PCH_9SERIES_1: u16 = 0x8C80;
pub const PCI_DEV_PCH_9SERIES_2: u16 = 0x8C81;
pub const PCI_DEV_PCH_9SERIES_3: u16 = 0x8C88;
pub const PCI_DEV_PCH_9SERIES_4: u16 = 0x8C89;
pub const PCI_DEV_BRIDGE_450KX: u16 = 0x84C4;
pub const PCI_DEV_BRIDGE_450NX: u16 = 0x84CB;

pub const BRIDGE_450KX_REV_B0: u8 = 0x04;
pub const BRIDGE_450NX_REV_B0: u8 = 0x00;
pub const BRIDGE_450NX_REV_B1: u8 = 0x02;
pub const BRIDGE_450NX_REV_C0: u8 = 0x04;

pub const PXB_REG_CONFIG: u8 = 0x40;
pub const PXB_CONFIG_BUF_RESTREAM: u16 = 0x0040;
pub const PXB_CONFIG_PCI_BUS_LOCK: u16 = 0x4000;

pub const MPIIX_REG_IDETIM: u8 = 0x6C;

pub const PIIX_IDETIM_RCT_MASK: u16 = 0x0300;
pub const PIIX_IDETIM_RSV_MASK: u16 = 0x0C00;
pub const PIIX_IDETIM_ISP_MASK: u16 = 0x3000;
pub const PIIX_IDETIM_SITRE: u16 = 0x4000;
pub const PIIX_IDETIM_SECONDARY: u16 = 0x4000;
pub const PIIX_IDETIM_IDE: u16 = 0x8000;

pub const PIIX_REG_SIDETIM: u8 = 0x44;
pub const PIIX_REG_UDMACTL: u8 = 0x48;
pub const PIIX_UDMATIM_CT_MASK: u8 = 0x33;
pub const PIIX_REG_CONFIG: u8 = 0x54;
pub const PIIX_CONFIG_KEEP_MASK: u16 = 0x0FF0;
pub const PIIX_CONFIG_WR_PING_PONG: u16 = 0x0400;

pub const SCH_DTIM_PM_MASK: u32 = 0x00000007;
pub const SCH_DTIM_MDM_MASK: u32 = 0x00000300;
pub const SCH_DTIM_UDM_MASK: u32 = 0x00070000;
pub const SCH_DTIM_PPE: u32 = 0x40000000;
pub const SCH_DTIM_USD: u32 = 0x80000000;

pub const HW_FLAGS_TYPE_MASK: u16 = 0x000F;
pub const HW_FLAGS_DISABLE_DMA: u16 = 0x0010;
pub const HW_FLAGS_HAS_CFG_REG: u16 = 0x0080;
pub const HW_FLAGS_HAS_UDMA_REG: u16 = 0x8000; // nolint: const-contradiction (intel.c-local flag)

pub const TYPE_MPIIX: u16 = 0;
pub const TYPE_PIIX: u16 = 1;
pub const TYPE_PIIX3: u16 = 2;
pub const TYPE_PIIX4: u16 = 3 | HW_FLAGS_HAS_UDMA_REG;
pub const TYPE_ICH: u16 = 4 | HW_FLAGS_HAS_UDMA_REG | HW_FLAGS_HAS_CFG_REG;
pub const TYPE_SATA: u16 = 5;
pub const TYPE_SCH: u16 = 6;

pub type PINTEL_CONTROLLER_INFO = *mut INTEL_CONTROLLER_INFO;

#[repr(C)]
pub struct INTEL_HW_EXTENSION {
    pub Register: u32,
    pub CurrentTiming: u16,
    pub Device: [INTEL_HW_EXTENSION_Device; (MAX_IDE_DEVICE as usize)],
}

#[repr(C)]
pub struct INTEL_HW_EXTENSION_Device {
    pub Timing: [u16; 2],
}

pub type PINTEL_HW_EXTENSION = *mut INTEL_HW_EXTENSION;

unsafe extern "C" fn IntelPiixChooseDeviceSpeed(
    mut Channel: u32,
    mut Device: *mut CHANNEL_DEVICE_CONFIG,
) {
    let mut Mode: u32;

    Mode = (*Device).PioMode;
    while Mode > PIO_MODE!(0) {
        if ((*Device).SupportedModes & (1 << Mode)) == 0 {
            Mode = Mode.wrapping_sub(1);
            continue;
        }
        if IntelTimingModeToCycleTime[Mode as usize] >= (*Device).MinPioCycleTime {
            break;
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
        let mut MinimumCycleTime: u32;
        let mut TimingMode: u32;

        if ((*Device).SupportedModes & !PIO_ALL & (1 << Mode)) == 0 {
            Mode = Mode.wrapping_sub(1);
            continue;
        }

        if (*Device).DmaMode >= MWDMA_MODE!(0) {
            MinimumCycleTime = (*Device).MinMwDmaCycleTime;
        } else {
            MinimumCycleTime = (*Device).MinSwDmaCycleTime;
        }

        TimingMode = IntelDmaModeToTimingMode[(Mode - SWDMA_MODE!(2)) as usize];

        if IntelTimingModeToCycleTime[TimingMode as usize] >= MinimumCycleTime {
            break;
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

unsafe extern "C" fn IntelPiixSetTransferMode(mut Controller: PATA_CONTROLLER, mut Channel: ULONG, mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    let mut ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[Channel as usize] as PCHANNEL_DATA_PATA;
    let mut IdeTimReg: USHORT = 0;
    let mut IdeConfigReg: USHORT = 0;
    let mut IdeSlaveTimReg: UCHAR = 0;
    let mut IdeUdmaCtrlReg: UCHAR = 0;
    let mut IdeUdmaTimReg: UCHAR = 0;
    let mut i: ULONG = 0;

    IdeUdmaCtrlReg = 0 as UCHAR;
    IdeUdmaTimReg = 0 as UCHAR;
    IdeConfigReg = 0 as USHORT;

    IdeTimReg = PciRead16(Controller, PIIX_REG_IDETIM!(Channel));
    IdeSlaveTimReg = PciRead8(Controller, (PIIX_REG_SIDETIM) as ULONG);
    if (*ChanData).HwFlags & (HW_FLAGS_HAS_UDMA_REG) as u32 != 0 {
        IdeUdmaCtrlReg = PciRead8(Controller, (PIIX_REG_UDMACTL) as ULONG);
        IdeUdmaTimReg = PciRead8(Controller, PIIX_REG_UDMATIM!(Channel));
        if (*ChanData).HwFlags & (HW_FLAGS_HAS_CFG_REG as u32) != 0 {
            IdeConfigReg = PciRead16(Controller, (PIIX_REG_CONFIG) as ULONG);
        }
    }

    INFO!(
        b"CH %lu: Config (before)\nIDETIM  %04X\nSIDETIM   %02X\nUDMACTL   %02X\nUDMATIM   %02X\nCONFIG  %04X\n\0"
            .as_ptr() as *const i8,
        Channel,
        IdeTimReg,
        IdeSlaveTimReg,
        IdeUdmaCtrlReg,
        IdeUdmaTimReg,
        IdeConfigReg,
    );

    IdeTimReg = (IdeTimReg as i32 & ((PIIX_IDETIM_IDE | PIIX_IDETIM_RSV_MASK)) as i32) as USHORT;
    IdeSlaveTimReg = (IdeSlaveTimReg as i32 & (!PIIX_SIDETIM_ISPRCT_MASK!(Channel)) as i32) as UCHAR;
    IdeUdmaCtrlReg = (IdeUdmaCtrlReg as i32 & (!PIIX_UDMACTL_EN_MASK!(Channel)) as i32) as UCHAR;
    IdeUdmaTimReg = (IdeUdmaTimReg as i32 & (!PIIX_UDMATIM_CT_MASK) as i32) as UCHAR;
    IdeConfigReg = (IdeConfigReg as i32 & (PIIX_CONFIG_KEEP_MASK as i32)) as USHORT;

    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let mut TimingMode: ULONG = 0;

        if !Device.is_null() {
            IntelPiixChooseDeviceSpeed(Channel, Device);

            TimingMode = (*Device).PioMode;

            if (*Device).DmaMode >= UDMA_MODE!(0) {
                IdeUdmaTimReg = (IdeUdmaTimReg as i32
                    | PIIX_UDMATIM_CT!(
                        i,
                        IntelUdmaSettings[(*Device)
                            .DmaMode
                            .wrapping_sub(UDMA_MODE!(0))
                            as usize],
                    )) as UCHAR;
                IdeUdmaCtrlReg = (IdeUdmaCtrlReg as i32 | PIIX_UDMACTL_EN!(Channel, i)) as UCHAR;

                if (*Device).DmaMode == UDMA_MODE!(5) {
                    IdeConfigReg = (IdeConfigReg as i32 | PIIX_CONFIG_CLOCK_UDMA100!(Channel, i)) as USHORT;
                } else if (*Device).DmaMode > UDMA_MODE!(2) {
                    IdeConfigReg = (IdeConfigReg as i32 | PIIX_CONFIG_CLOCK_UDMA66!(Channel, i)) as USHORT;
                }
            } else if (*Device).DmaMode != PIO_MODE!(0) {
                TimingMode = IntelDmaModeToTimingMode[(*Device)
                    .DmaMode
                    .wrapping_sub(SWDMA_MODE!(2))
                    as usize];
                if TimingMode < (*Device).PioMode {
                    IdeTimReg = (IdeTimReg as i32 | PIIX_IDETIM_DTE!(i)) as USHORT;
                }
            }

            if i == 0 {
                IdeTimReg = (IdeTimReg as i32
                    | PIIX_IDETIM_ISP!(IntelClockSettings[TimingMode as usize][0]))
                    as USHORT;
                IdeTimReg = (IdeTimReg as i32
                    | PIIX_IDETIM_RCT!(IntelClockSettings[TimingMode as usize][1]))
                    as USHORT;
            } else {
                IdeSlaveTimReg = (IdeSlaveTimReg as i32
                    | PIIX_SIDETIM_ISP!(
                        Channel,
                        IntelClockSettings[TimingMode as usize][0],
                    )) as UCHAR;
                IdeSlaveTimReg = (IdeSlaveTimReg as i32
                    | PIIX_SIDETIM_RCT!(
                        Channel,
                        IntelClockSettings[TimingMode as usize][1],
                    )) as UCHAR;

                IdeTimReg = (IdeTimReg as i32 | (PIIX_IDETIM_SITRE as i32)) as USHORT;
            }

            IdeTimReg = (IdeTimReg as i32
                | PIIX_IDETIM_SETTINGS!(i, IntelModeSettings[TimingMode as usize]))
                as USHORT;

            if TimingMode == PIO_MODE!(2) && (*Device).IoReadySupported != 0 {
                IdeTimReg = (IdeTimReg as i32 | PIIX_IDETIM_IE!(i)) as USHORT;
            }

            if (*Device).IsFixedDisk != 0 {
                IdeTimReg = (IdeTimReg as i32 | PIIX_IDETIM_PPE!(i)) as USHORT;
            }
        }

        i = i.wrapping_add(1);
    }

    PciWrite8(Controller, (PIIX_REG_SIDETIM) as ULONG, IdeSlaveTimReg);
    PciWrite16(Controller, PIIX_REG_IDETIM!(Channel), IdeTimReg);
    if (*ChanData).HwFlags & (HW_FLAGS_HAS_UDMA_REG) as u32 != 0 {
        PciWrite8(Controller, (PIIX_REG_UDMACTL) as ULONG, IdeUdmaCtrlReg);
        PciWrite8(Controller, PIIX_REG_UDMATIM!(Channel), IdeUdmaTimReg);
        if (*ChanData).HwFlags & (HW_FLAGS_HAS_CFG_REG as u32) != 0 {
            IdeConfigReg = (IdeConfigReg as i32 | (PIIX_CONFIG_WR_PING_PONG as i32)) as USHORT;

            PciWrite16(Controller, (PIIX_REG_CONFIG) as ULONG, IdeConfigReg);
        }
    }

    INFO!(
        b"CH %lu: Config (after)\nIDETIM  %04X\nSIDETIM   %02X\nUDMACTL   %02X\nUDMATIM   %02X\nCONFIG  %04X\n\0"
            .as_ptr() as *const i8,
        Channel,
        IdeTimReg,
        IdeSlaveTimReg,
        IdeUdmaCtrlReg,
        IdeUdmaTimReg,
        IdeConfigReg,
    );
}

unsafe extern "C" fn IntelSchSetTransferMode(mut Controller: PATA_CONTROLLER, mut Channel: ULONG, mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    let mut i: ULONG = 0;
    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        let mut DeviceTimingReg: ULONG = 0;
        if !Device.is_null() {
            DeviceTimingReg = PciRead32(Controller, SCH_REG_DTIM!(i));
            DeviceTimingReg &= !(SCH_DTIM_PM_MASK
                | SCH_DTIM_MDM_MASK
                | SCH_DTIM_UDM_MASK
                | SCH_DTIM_PPE
                | SCH_DTIM_USD);
            if (*Device).IsFixedDisk != 0 {
                DeviceTimingReg |= SCH_DTIM_PPE;
            }
            if (*Device).DmaMode >= UDMA_MODE!(0) {
                DeviceTimingReg |= SCH_DTIM_USD
                    | SCH_DTIM_UDM!(
                        (*Device).DmaMode.wrapping_sub(UDMA_MODE!(0)),
                    );
            } else if (*Device).DmaMode >= MWDMA_MODE!(0) {
                DeviceTimingReg |= SCH_DTIM_MDM!(
                    (*Device).DmaMode.wrapping_sub(MWDMA_MODE!(0)),
                );
            }
            DeviceTimingReg |= SCH_DTIM_PM!((*Device).PioMode);
            PciWrite32(Controller, SCH_REG_DTIM!(i), DeviceTimingReg);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn IntelPiixLegacyPrepareIo(mut ChannelContext: PVOID, mut Request: PATA_DEVICE_REQUEST) {
    let mut ChanData: PCHANNEL_DATA_PATA = ChannelContext as PCHANNEL_DATA_PATA;
    let mut HwExt: PINTEL_HW_EXTENSION = ((*ChanData).HwExt).as_mut_ptr() as PINTEL_HW_EXTENSION;
    let mut Index: ULONG = if (*Request).Flags & REQUEST_FLAG_PROGRAM_DMA != 0 {
        1
    } else {
        0
    };
    let mut NewTimingReg: USHORT = (*HwExt).Device[DEV_NUMBER!((*Request).Device) as usize]
        .Timing[Index as usize];
    if (*HwExt).CurrentTiming as i32 != NewTimingReg as i32 {
        (*HwExt).CurrentTiming = NewTimingReg;
        PciWrite16((*ChanData).__bindgen_anon_1.Controller, (*HwExt).Register, NewTimingReg);
    }
    PataPrepareIo(ChanData as PVOID, Request);
}

unsafe extern "C" fn IntelPiixLegacyComputeIdeTiming(
    Device: PCHANNEL_DEVICE_CONFIG,
    Number: ULONG,
    Mode: ULONG,
) -> USHORT {
    let mut IdeTimReg: USHORT;
    let mut TimingMode: ULONG;

    if Mode > PIO_MODE!(4) {
        TimingMode = IntelDmaModeToTimingMode[(Mode - SWDMA_MODE!(2)) as usize];
    } else {
        TimingMode = Mode;
    }

    IdeTimReg = PIIX_IDETIM_ISP!(IntelClockSettings[TimingMode as usize][0]) as USHORT;
    IdeTimReg |= PIIX_IDETIM_RCT!(IntelClockSettings[TimingMode as usize][1]) as USHORT;
    IdeTimReg |= PIIX_IDETIM_SETTINGS!(Number, IntelModeSettings[TimingMode as usize]) as USHORT;

    if (*Device).IsFixedDisk != 0 {
        IdeTimReg |= PIIX_IDETIM_PPE!(Number) as USHORT;
    }

    if TimingMode == PIO_MODE!(2) && (*Device).IoReadySupported != 0 {
        IdeTimReg |= PIIX_IDETIM_IE!(Number) as USHORT;
    }

    return IdeTimReg;
}

unsafe extern "C" fn IntelPiixLegacySetTransferMode(mut Controller: PATA_CONTROLLER, mut Channel: ULONG, mut DeviceList: *mut PCHANNEL_DEVICE_CONFIG) {
    let mut ChanData: PCHANNEL_DATA_PATA = (*Controller).Channels[Channel as usize] as PCHANNEL_DATA_PATA;
    let mut HwExt: PINTEL_HW_EXTENSION = &raw mut (*ChanData).HwExt as *mut PUCHAR as PVOID as PINTEL_HW_EXTENSION;
    let mut IdeTimReg: USHORT = 0;
    let mut i: ULONG = 0;

    IdeTimReg = PciRead16(Controller, (*HwExt).Register);
    IdeTimReg = (IdeTimReg as ::core::ffi::c_int
        & ((PIIX_IDETIM_IDE | PIIX_IDETIM_SITRE | PIIX_IDETIM_RSV_MASK)) as ::core::ffi::c_int) as USHORT;
    PciWrite16(Controller, (*HwExt).Register, IdeTimReg);

    i = 0;
    while i < MAX_IDE_DEVICE {
        let mut Device: PCHANNEL_DEVICE_CONFIG = *DeviceList.offset(i as isize);
        if !Device.is_null() {
            IntelPiixChooseDeviceSpeed(Channel, Device);

            (*HwExt).Device[i as usize].Timing[0 as usize] =
                IntelPiixLegacyComputeIdeTiming(Device, i, (*Device).PioMode);
            (*HwExt).Device[i as usize].Timing[1 as usize] =
                IntelPiixLegacyComputeIdeTiming(Device, i, (*Device).DmaMode);

            INFO!(
                b"CH %lu: Drive #%lu PIO:%04X DMA:04X\n\0".as_ptr() as *const i8,
                Channel,
                i,
                (*HwExt).Device[i as usize].Timing[0 as usize],
                (*HwExt).Device[i as usize].Timing[1 as usize],
            );
        }
        i = i.wrapping_add(1);
    }

    (*HwExt).CurrentTiming = 0;
}

unsafe extern "C" fn IntelPciBridgeErrataMatch(mut Context: PVOID, mut BusNumber: ULONG, mut PciSlot: PCI_SLOT_NUMBER, mut PciConfig: PPCI_COMMON_HEADER) -> BOOLEAN {
    UNREFERENCED_PARAMETER!(Context);
    PAGED_CODE!();
    if (*PciConfig).VendorID as ::core::ffi::c_int != (PCI_VEN_INTEL as i32) {
        return FALSE;
    }
    if (*PciConfig).DeviceID as ::core::ffi::c_int == (PCI_DEV_BRIDGE_450KX as i32)
        && ((*PciConfig).RevisionID as ::core::ffi::c_int) < (BRIDGE_450KX_REV_B0 as i32)
    {
        WARN!(b"Enabling workaround for the 450KX #2\n\0".as_ptr() as *const i8);
        return TRUE;
    }
    if (*PciConfig).DeviceID as ::core::ffi::c_int == (PCI_DEV_BRIDGE_450NX as i32) {
        let mut ConfigReg: USHORT = 0;
        if (*PciConfig).RevisionID as ::core::ffi::c_int == (BRIDGE_450NX_REV_B0 as i32) {
            WARN!(b"Enabling workaround for the 450NX #19\n\0".as_ptr() as *const i8);
            return TRUE;
        }
        HalGetBusDataByOffset(
            PCIConfiguration,
            BusNumber,
            PciSlot.u.AsULONG,
            &raw mut ConfigReg as PVOID,
            (PXB_REG_CONFIG) as ULONG,
            ::core::mem::size_of::<USHORT>() as ULONG,
        );
        INFO!(
            b"PXB CFG: %04X\n\0" as *const u8 as *const ::core::ffi::c_char,
            ConfigReg,
        );
        if (*PciConfig).RevisionID as ::core::ffi::c_int != (BRIDGE_450NX_REV_B1 as i32)
            && ConfigReg as ::core::ffi::c_int & (PXB_CONFIG_PCI_BUS_LOCK as i32) != 0
        {
            WARN!(b"Enabling workaround for the 450NX #20\n\0".as_ptr() as *const i8);
            return TRUE;
        }
        if (*PciConfig).RevisionID as ::core::ffi::c_int == (BRIDGE_450NX_REV_C0 as i32)
            && ConfigReg as ::core::ffi::c_int & (PXB_CONFIG_BUF_RESTREAM as i32) != 0
        {
            WARN!(b"Enabling workaround for the 450NX #25\n\0".as_ptr() as *const i8);
            return TRUE;
        }
    }
    return FALSE;
}

unsafe extern "C" fn IntelInitChannel(
    mut Controller: PATA_CONTROLLER,
    mut ControllerInfo: *const INTEL_CONTROLLER_INFO,
    mut ChanData: PCHANNEL_DATA_PATA,
) {
    let mut SupportedMode: ULONG = 0;

    PAGED_CODE!();

    match ((*ControllerInfo).Type as ::core::ffi::c_int) as u16 {
        TYPE_MPIIX | TYPE_PIIX => {
            let mut HwExt: PINTEL_HW_EXTENSION =
                &raw mut (*ChanData).HwExt as *mut PUCHAR as PVOID as PINTEL_HW_EXTENSION;
            if (*ControllerInfo).Type as ::core::ffi::c_int == (TYPE_MPIIX as i32) {
                (*HwExt).Register = MPIIX_REG_IDETIM as ULONG;
            } else {
                (*HwExt).Register = PIIX_REG_IDETIM!((*ChanData).__bindgen_anon_1.Channel) as ULONG;
            }
            (*ChanData).__bindgen_anon_1.SetTransferMode = Some(
                IntelPiixLegacySetTransferMode
                    as unsafe extern "C" fn(
                        PATA_CONTROLLER,
                        ULONG,
                        *mut PCHANNEL_DEVICE_CONFIG,
                    ) -> (),
            ) as PCHANNEL_SET_MODE_EX;
            (*ChanData).__bindgen_anon_1.PrepareIo = Some(
                IntelPiixLegacyPrepareIo as unsafe extern "C" fn(PVOID, PATA_DEVICE_REQUEST) -> (),
            ) as PCHANNEL_PREPARE_IO;
        }
        TYPE_PIIX3 | TYPE_PIIX4 | TYPE_ICH => {
            (*ChanData).__bindgen_anon_1.SetTransferMode = Some(
                IntelPiixSetTransferMode
                    as unsafe extern "C" fn(
                        PATA_CONTROLLER,
                        ULONG,
                        *mut PCHANNEL_DEVICE_CONFIG,
                    ) -> (),
            ) as PCHANNEL_SET_MODE_EX;
        }
        TYPE_SCH => {
            (*ChanData).__bindgen_anon_1.SetTransferMode = Some(
                IntelSchSetTransferMode
                    as unsafe extern "C" fn(
                        PATA_CONTROLLER,
                        ULONG,
                        *mut PCHANNEL_DEVICE_CONFIG,
                    ) -> (),
            ) as PCHANNEL_SET_MODE_EX;
        }
        _ => {
            (*ChanData).__bindgen_anon_1.SetTransferMode = Some(
                SataSetTransferMode
                    as unsafe extern "C" fn(
                        PATA_CONTROLLER,
                        ULONG,
                        *mut PCHANNEL_DEVICE_CONFIG,
                    ) -> (),
            ) as PCHANNEL_SET_MODE_EX;
        }
    }

    match ((*ControllerInfo).Type as ::core::ffi::c_int) as u16 {
        TYPE_MPIIX => {
            SupportedMode = PIO_ALL as ULONG;
        }
        TYPE_PIIX | TYPE_PIIX3 => {
            SupportedMode = (PIO_ALL | SWDMA_MODE2 | MWDMA_MODES!(1, 2)) as ULONG;
        }
        TYPE_PIIX4 => {
            SupportedMode = (PIO_ALL | SWDMA_MODE2 | MWDMA_MODES!(1, 2) | UDMA_MODES!(0, 2)) as ULONG;
        }
        TYPE_ICH => {
            SupportedMode = (PIO_ALL | MWDMA_MODE2 | UDMA_MODES!(0, 5)) as ULONG;

            if (*Controller).Pci.DeviceID as ::core::ffi::c_int == (PCI_DEV_ICH_82801AA as i32)
                || (*Controller).Pci.DeviceID as ::core::ffi::c_int == (PCI_DEV_ICH0_82801AB as i32)
            {
                if (*Controller).Pci.RevisionID as ::core::ffi::c_int == 0 {
                    SupportedMode = PIO_ALL as ULONG;
                } else {
                    SupportedMode &= !(UDMA_MODE5 as ULONG);

                    if (*Controller).Pci.DeviceID as ::core::ffi::c_int == (PCI_DEV_ICH0_82801AB as i32) {
                        SupportedMode &= !((UDMA_MODE3 | UDMA_MODE4) as ULONG);
                    }
                }
            }

            if SupportedMode & (UDMA_80C_ALL as ULONG) != 0 {
                let mut IdeConfigReg: USHORT = PciRead16(Controller, PIIX_REG_CONFIG as ULONG);
                if IdeConfigReg as ::core::ffi::c_int & PIIX_CONFIG_CR!((*ChanData).__bindgen_anon_1.Channel) == 0 {
                    INFO!(
                        b"CH %lu: BIOS detected 40-conductor cable\n\0" as *const u8 as *const ::core::ffi::c_char,
                        (*ChanData).Channel,
                    );
                    SupportedMode &= !(UDMA_80C_ALL as ULONG);
                }
            }
        }
        TYPE_SCH => {
            let mut DeviceTimingReg: [ULONG; 2] = [0; 2];

            SupportedMode = (PIO_ALL | MWDMA_ALL | UDMA_MODES!(0, 5)) as ULONG;

            PciRead(
                Controller,
                &raw mut DeviceTimingReg as PVOID,
                SCH_REG_DTIM!(0) as ULONG,
                ::core::mem::size_of::<[ULONG; 2]>() as ULONG,
            );
            if DeviceTimingReg[0 as usize] & (SCH_DTIM_UDM_MASK as ULONG)
                <= SCH_DTIM_UDM!(2) as ULONG
                && DeviceTimingReg[1 as usize] & (SCH_DTIM_UDM_MASK as ULONG)
                    <= SCH_DTIM_UDM!(2) as ULONG
            {
                INFO!(
                    b"CH %lu: BIOS hasn't selected mode faster than UDMA 2, assume 40-conductor cable\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    (*ChanData).Channel,
                );
                SupportedMode &= !(UDMA_80C_ALL as ULONG);
            }
        }
        TYPE_SATA => {
            SupportedMode = SATA_ALL as ULONG;

            // TODO: Read the port map register and set CHANNEL_FLAG_NO_SLAVE when appropriate
            ERR!(
                b"CH %lu: PMR %02X PCS %04X\n\0".as_ptr() as *const i8,
                (*ChanData).__bindgen_anon_1.Channel,
                PciRead8(Controller, 0x90) as u32,
                PciRead16(Controller, 0x92) as u32,
            );
        }
        _ => {
            ASSERT!(FALSE);
            UNREACHABLE!();
        }
    }

    (*ChanData).__bindgen_anon_1.TransferModeSupported = SupportedMode;
}
#[no_mangle]
pub unsafe extern "C" fn IntelGetControllerProperties(
    Controller: PATA_CONTROLLER,
) -> NTSTATUS {
    let mut ControllerInfo: *const INTEL_CONTROLLER_INFO = core::ptr::null();
    let mut i: u32;
    let mut ExtensionSize: u32;
    let mut HwFlags: u32;
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    i = 0;
    while i < RTL_NUMBER_OF!(IntelControllerList) {
        ControllerInfo = &IntelControllerList[i as usize];
        if (*Controller).Pci.DeviceID == (*ControllerInfo).DeviceID {
            break;
        }
        i = i.wrapping_add(1);
    }
    if i == RTL_NUMBER_OF!(IntelControllerList) {
        return STATUS_NO_MATCH;
    }

    (*Controller).Flags |= CTRL_FLAG_DMA_INTERRUPT;

    if (*ControllerInfo).Type == TYPE_SCH {
        (*Controller).MaxChannels = 1;
    }

    if (*ControllerInfo).Type == TYPE_PIIX
        || (*ControllerInfo).Type == TYPE_MPIIX
    {
        ExtensionSize = core::mem::size_of::<INTEL_HW_EXTENSION>() as u32;
    } else {
        ExtensionSize = 0;
    }

    Status = PciIdeCreateChannelData(Controller, ExtensionSize);
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    match (*ControllerInfo).Type {
        TYPE_MPIIX => {
            (*Controller).Flags &= !CTRL_FLAG_NATIVE_PCI;
            (*Controller).__bindgen_anon_1.ChannelEnableBits = IntelMpiixEnableBits.as_ptr();
        }
        TYPE_PIIX
        | TYPE_PIIX3
        | TYPE_PIIX4
        | TYPE_ICH => {
            (*Controller).__bindgen_anon_1.ChannelEnableBits = IntelPiixEnableBits.as_ptr();
        }
        TYPE_SATA => {}
        _ => {}
    }
    HwFlags = ((*ControllerInfo).Type) as _;

    if PciFindDevice(Some(IntelPciBridgeErrataMatch), core::ptr::null_mut()) != 0 {
        HwFlags |= (HW_FLAGS_DISABLE_DMA as u32);
    }

    i = 0;
    while i < (*Controller).MaxChannels {
        let ChanData: PCHANNEL_DATA_PATA = ((*Controller).Channels[i as usize]).cast();

        (*ChanData).HwFlags = HwFlags;

        IntelInitChannel(Controller, ControllerInfo, ChanData);

        if HwFlags & (HW_FLAGS_DISABLE_DMA as u32) != 0 {
            (*ChanData).__bindgen_anon_1.TransferModeSupported &= PIO_ALL;
        }
        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
